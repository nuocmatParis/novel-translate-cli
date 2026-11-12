use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use nt_application::workspace::{WorkspaceError, WorkspaceRepository};
use nt_domain::workspace::{GenerationId, SeriesId, SeriesMetadata, VolumeId, WorkspaceMetadata};
use rusqlite::types::Value;
use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior};

const DATABASE_FILE: &str = "series.sqlite3";
const LEGACY_GO_MARKER: &str = "workspace.yaml";
const SCHEMA_VERSION: u32 = 1;
const APPLICATION_ID: i64 = 0x4E54_5253;
const BOOTSTRAP_GENERATION: &str = "g0";
const BUSY_TIMEOUT: Duration = Duration::from_secs(2);
static NEXT_VALIDATION_SNAPSHOT: AtomicU64 = AtomicU64::new(0);

const SCHEMA_V1: &str = r#"
CREATE TABLE series (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    series_id TEXT NOT NULL UNIQUE
        CHECK (length(series_id) = 32 AND series_id NOT GLOB '*[^0-9a-f]*'),
    created_at_utc_ms INTEGER NOT NULL CHECK (created_at_utc_ms >= 0)
);

CREATE TABLE volumes (
    series_id TEXT NOT NULL,
    volume_id TEXT NOT NULL
        CHECK (length(volume_id) = 32 AND volume_id NOT GLOB '*[^0-9a-f]*'),
    display_title TEXT,
    created_at_utc_ms INTEGER NOT NULL CHECK (created_at_utc_ms >= 0),
    PRIMARY KEY (series_id, volume_id),
    FOREIGN KEY (series_id) REFERENCES series(series_id) ON DELETE RESTRICT
);

CREATE TABLE generations (
    series_id TEXT NOT NULL,
    generation_id TEXT NOT NULL
        CHECK (generation_id = 'g0' OR
               (length(generation_id) = 32 AND generation_id NOT GLOB '*[^0-9a-f]*')),
    parent_generation_id TEXT,
    created_at_utc_ms INTEGER NOT NULL CHECK (created_at_utc_ms >= 0),
    PRIMARY KEY (series_id, generation_id),
    FOREIGN KEY (series_id) REFERENCES series(series_id) ON DELETE RESTRICT,
    FOREIGN KEY (series_id, parent_generation_id)
        REFERENCES generations(series_id, generation_id) ON DELETE RESTRICT,
    CHECK (
        (generation_id = 'g0' AND parent_generation_id IS NULL) OR
        (generation_id <> 'g0' AND parent_generation_id IS NOT NULL)
    )
);

CREATE TABLE current_generation (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    series_id TEXT NOT NULL,
    generation_id TEXT NOT NULL,
    FOREIGN KEY (series_id, generation_id)
        REFERENCES generations(series_id, generation_id) ON DELETE RESTRICT
);
"#;

#[derive(Clone, Copy, Debug, Default)]
pub struct SqliteWorkspaceRepository;

impl WorkspaceRepository for SqliteWorkspaceRepository {
    fn create(&self, root: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
        let prepared = prepare_create_root(root)?;
        let database_path = prepared.path.join(DATABASE_FILE);

        let database_file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&database_path)
        {
            Ok(file) => file,
            Err(error) => {
                remove_created_root_if_empty(&prepared);
                return Err(map_io_error(error));
            }
        };
        drop(database_file);

        let result = (|| {
            let mut connection = Connection::open_with_flags(
                &database_path,
                OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
            .map_err(map_sqlite_error)?;
            configure_new_database(&connection)?;
            let metadata = bootstrap_v1(&mut connection)?;
            validate_schema(&connection)?;
            Ok(metadata)
        })();

        match result {
            Ok(metadata) => Ok(metadata),
            Err(error) => {
                remove_new_database_artifacts(&database_path);
                remove_created_root_if_empty(&prepared);
                Err(error)
            }
        }
    }

    fn open(&self, root: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
        let root = prepare_open_root(root)?;
        reject_legacy_go_workspace(&root)?;

        let database_path = root.join(DATABASE_FILE);
        let database_metadata = fs::symlink_metadata(&database_path).map_err(map_io_error)?;
        if database_metadata.file_type().is_symlink() || !database_metadata.is_file() {
            return Err(WorkspaceError::UnsupportedFormat);
        }

        let metadata = {
            let snapshot = create_validation_snapshot(&database_path)?;
            let validation = Connection::open_with_flags(
                &snapshot.database_path,
                OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
            .map_err(map_corrupt_or_sqlite_error)?;
            validation
                .busy_timeout(BUSY_TIMEOUT)
                .map_err(map_sqlite_error)?;
            validation
                .execute_batch("PRAGMA foreign_keys = ON;")
                .map_err(map_sqlite_error)?;
            let metadata = validate_database(&validation)?;
            verify_wal_mode(&validation)?;
            metadata
        };

        // Check write access without changing the file before SQLite opens a writer connection.
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&database_path)
            .map_err(map_io_error)?;

        let connection = Connection::open_with_flags(
            &database_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(map_sqlite_error)?;
        configure_existing_database(&connection)?;
        Ok(metadata)
    }
}

#[derive(Debug)]
struct PreparedRoot {
    path: PathBuf,
    created_by_call: bool,
}

struct ValidationSnapshot {
    directory: PathBuf,
    database_path: PathBuf,
    _source_files: Vec<(&'static str, File)>,
}

impl ValidationSnapshot {
    fn create(database_path: &Path) -> Result<Self, WorkspaceError> {
        let mut source_files = Vec::new();
        for suffix in ["", "-wal", "-journal"] {
            let source_path = append_path_suffix(database_path, suffix);
            let metadata = match fs::symlink_metadata(&source_path) {
                Ok(metadata) => metadata,
                Err(error) if suffix.is_empty() => return Err(map_io_error(error)),
                Err(error) if error.kind() == ErrorKind::NotFound => continue,
                Err(error) => return Err(map_io_error(error)),
            };
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(WorkspaceError::UnsupportedFormat);
            }
            let source_file = if suffix.is_empty() {
                open_snapshot_database(&source_path)?
            } else {
                OpenOptions::new()
                    .read(true)
                    .open(&source_path)
                    .map_err(map_io_error)?
            };
            source_files.push((suffix, source_file));
        }

        let directory = create_validation_directory()?;
        let mut snapshot = Self {
            database_path: directory.join(DATABASE_FILE),
            directory,
            _source_files: source_files,
        };

        for (suffix, source) in &mut snapshot._source_files {
            let destination_path = append_path_suffix(&snapshot.database_path, suffix);
            let mut destination = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination_path)
                .map_err(map_io_error)?;
            std::io::copy(source, &mut destination).map_err(map_io_error)?;
            destination.flush().map_err(map_io_error)?;
        }

        Ok(snapshot)
    }
}

impl Drop for ValidationSnapshot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn create_validation_snapshot(database_path: &Path) -> Result<ValidationSnapshot, WorkspaceError> {
    ValidationSnapshot::create(database_path)
}

fn create_validation_directory() -> Result<PathBuf, WorkspaceError> {
    let temporary_root = std::env::temp_dir();
    for _ in 0..128 {
        let sequence = NEXT_VALIDATION_SNAPSHOT.fetch_add(1, Ordering::Relaxed);
        let directory = temporary_root.join(format!(
            "novel-translate-open-{}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(map_io_error(error)),
        }
    }
    Err(WorkspaceError::StorageFailure)
}

fn append_path_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut suffixed = path.as_os_str().to_os_string();
    suffixed.push(suffix);
    PathBuf::from(suffixed)
}

#[cfg(windows)]
fn open_snapshot_database(path: &Path) -> Result<File, WorkspaceError> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;

    OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(path)
        .map_err(map_io_error)
}

#[cfg(not(windows))]
fn open_snapshot_database(_path: &Path) -> Result<File, WorkspaceError> {
    Err(WorkspaceError::UnsupportedFilesystem)
}

fn prepare_create_root(path: &Path) -> Result<PreparedRoot, WorkspaceError> {
    if path.as_os_str().is_empty() {
        return Err(WorkspaceError::InvalidPath);
    }
    reject_unc_path(path)?;

    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(WorkspaceError::InvalidPath);
            }
            let root = fs::canonicalize(path).map_err(map_io_error)?;
            ensure_local_volume(&root)?;
            reject_legacy_go_workspace(&root)?;

            match fs::symlink_metadata(root.join(DATABASE_FILE)) {
                Ok(_) => return Err(WorkspaceError::AlreadyExists),
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => return Err(map_io_error(error)),
            }

            if fs::read_dir(&root)
                .map_err(map_io_error)?
                .next()
                .transpose()
                .map_err(map_io_error)?
                .is_some()
            {
                return Err(WorkspaceError::DirectoryNotEmpty);
            }

            Ok(PreparedRoot {
                path: root,
                created_by_call: false,
            })
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let name = path.file_name().ok_or(WorkspaceError::InvalidPath)?;
            let requested_parent = path.parent().unwrap_or_else(|| Path::new("."));
            let requested_parent = if requested_parent.as_os_str().is_empty() {
                Path::new(".")
            } else {
                requested_parent
            };
            let parent = match fs::canonicalize(requested_parent) {
                Ok(parent) => parent,
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    return Err(WorkspaceError::InvalidPath);
                }
                Err(error) => return Err(map_io_error(error)),
            };
            if !parent.is_dir() {
                return Err(WorkspaceError::InvalidPath);
            }
            ensure_local_volume(&parent)?;

            let root = parent.join(name);
            fs::create_dir(&root).map_err(map_io_error)?;
            let canonical_root = fs::canonicalize(&root).map_err(map_io_error)?;
            ensure_local_volume(&canonical_root)?;
            Ok(PreparedRoot {
                path: canonical_root,
                created_by_call: true,
            })
        }
        Err(error) => Err(map_io_error(error)),
    }
}

fn prepare_open_root(path: &Path) -> Result<PathBuf, WorkspaceError> {
    if path.as_os_str().is_empty() {
        return Err(WorkspaceError::InvalidPath);
    }
    reject_unc_path(path)?;

    let metadata = fs::symlink_metadata(path).map_err(map_io_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(WorkspaceError::InvalidPath);
    }
    let root = fs::canonicalize(path).map_err(map_io_error)?;
    ensure_local_volume(&root)?;
    Ok(root)
}

fn reject_legacy_go_workspace(root: &Path) -> Result<(), WorkspaceError> {
    match fs::symlink_metadata(root.join(LEGACY_GO_MARKER)) {
        Ok(_) => Err(WorkspaceError::LegacyGoWorkspace),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(map_io_error(error)),
    }
}

#[cfg(windows)]
fn reject_unc_path(path: &Path) -> Result<(), WorkspaceError> {
    use std::path::{Component, Prefix};

    if let Some(Component::Prefix(prefix)) = path.components().next()
        && matches!(prefix.kind(), Prefix::UNC(..) | Prefix::VerbatimUNC(..))
    {
        return Err(WorkspaceError::UnsupportedFilesystem);
    }
    Ok(())
}

#[cfg(not(windows))]
fn reject_unc_path(_path: &Path) -> Result<(), WorkspaceError> {
    Ok(())
}

fn remove_created_root_if_empty(root: &PreparedRoot) {
    if root.created_by_call {
        let _ = fs::remove_dir(&root.path);
    }
}

fn remove_new_database_artifacts(database_path: &Path) {
    for suffix in ["", "-wal", "-shm", "-journal"] {
        let mut artifact = database_path.as_os_str().to_owned();
        artifact.push(suffix);
        let _ = fs::remove_file(PathBuf::from(artifact));
    }
}

#[cfg(windows)]
fn ensure_local_volume(path: &Path) -> Result<(), WorkspaceError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{GetDriveTypeW, GetVolumePathNameW};

    const DRIVE_REMOVABLE: u32 = 2;
    const DRIVE_FIXED: u32 = 3;
    const DRIVE_RAMDISK: u32 = 6;

    let input: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut volume_path = vec![0u16; 32_768];
    let found_volume = unsafe {
        GetVolumePathNameW(
            input.as_ptr(),
            volume_path.as_mut_ptr(),
            volume_path.len() as u32,
        )
    };
    if found_volume == 0 {
        return Err(map_io_error(std::io::Error::last_os_error()));
    }

    let drive_type = unsafe { GetDriveTypeW(volume_path.as_ptr()) };
    if matches!(drive_type, DRIVE_FIXED | DRIVE_REMOVABLE | DRIVE_RAMDISK) {
        Ok(())
    } else {
        Err(WorkspaceError::UnsupportedFilesystem)
    }
}

#[cfg(not(windows))]
fn ensure_local_volume(_path: &Path) -> Result<(), WorkspaceError> {
    Err(WorkspaceError::UnsupportedFilesystem)
}

fn configure_new_database(connection: &Connection) -> Result<(), WorkspaceError> {
    configure_connection(connection)?;
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(map_sqlite_error)?;
    verify_wal_mode(connection)
}

fn configure_existing_database(connection: &Connection) -> Result<(), WorkspaceError> {
    configure_connection(connection)?;
    verify_wal_mode(connection)
}

fn configure_connection(connection: &Connection) -> Result<(), WorkspaceError> {
    connection
        .busy_timeout(BUSY_TIMEOUT)
        .map_err(map_sqlite_error)?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL;")
        .map_err(map_sqlite_error)?;

    let foreign_keys: i64 = connection
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .map_err(map_sqlite_error)?;
    if foreign_keys != 1 {
        return Err(WorkspaceError::StorageFailure);
    }
    Ok(())
}

fn verify_wal_mode(connection: &Connection) -> Result<(), WorkspaceError> {
    let mode: String = connection
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .map_err(map_sqlite_error)?;
    if mode.eq_ignore_ascii_case("wal") {
        Ok(())
    } else {
        Err(WorkspaceError::UnsupportedFilesystem)
    }
}

fn bootstrap_v1(connection: &mut Connection) -> Result<WorkspaceMetadata, WorkspaceError> {
    let created_at_utc_ms = now_utc_ms()?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(map_sqlite_error)?;

    transaction
        .execute_batch(SCHEMA_V1)
        .map_err(map_sqlite_error)?;
    let series_id: String = transaction
        .query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
        .map_err(map_sqlite_error)?;
    transaction
        .execute(
            "INSERT INTO series (singleton, series_id, created_at_utc_ms) VALUES (1, ?1, ?2)",
            (&series_id, created_at_utc_ms),
        )
        .map_err(map_sqlite_error)?;
    transaction
        .execute(
            "INSERT INTO generations (series_id, generation_id, parent_generation_id, created_at_utc_ms) \
             VALUES (?1, ?2, NULL, ?3)",
            (&series_id, BOOTSTRAP_GENERATION, created_at_utc_ms),
        )
        .map_err(map_sqlite_error)?;
    transaction
        .execute(
            "INSERT INTO current_generation (singleton, series_id, generation_id) VALUES (1, ?1, ?2)",
            (&series_id, BOOTSTRAP_GENERATION),
        )
        .map_err(map_sqlite_error)?;
    transaction
        .execute_batch(&format!(
            "PRAGMA application_id = {APPLICATION_ID}; PRAGMA user_version = {SCHEMA_VERSION};"
        ))
        .map_err(map_sqlite_error)?;
    transaction.commit().map_err(map_sqlite_error)?;

    let series_id = SeriesId::parse(series_id).map_err(|_| WorkspaceError::CorruptWorkspace)?;
    let current_generation =
        GenerationId::parse(BOOTSTRAP_GENERATION).map_err(|_| WorkspaceError::CorruptWorkspace)?;
    Ok(WorkspaceMetadata {
        series: SeriesMetadata {
            id: series_id,
            created_at_utc_ms,
        },
        schema_version: SCHEMA_VERSION,
        current_generation,
    })
}

fn validate_database(connection: &Connection) -> Result<WorkspaceMetadata, WorkspaceError> {
    let application_id: i64 = connection
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(map_corrupt_or_sqlite_error)?;
    if application_id != APPLICATION_ID {
        return Err(WorkspaceError::UnsupportedFormat);
    }

    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(map_corrupt_or_sqlite_error)?;
    if version > i64::from(SCHEMA_VERSION) {
        return Err(WorkspaceError::NewerSchema {
            found: u32::try_from(version).unwrap_or(u32::MAX),
            supported: SCHEMA_VERSION,
        });
    }
    if version != i64::from(SCHEMA_VERSION) {
        return Err(WorkspaceError::CorruptWorkspace);
    }

    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(map_corrupt_or_sqlite_error)?;
    if integrity != "ok" {
        return Err(WorkspaceError::CorruptWorkspace);
    }

    let mut foreign_keys = connection
        .prepare("PRAGMA foreign_key_check")
        .map_err(map_corrupt_or_sqlite_error)?;
    if foreign_keys
        .query([])
        .map_err(map_corrupt_or_sqlite_error)?
        .next()
        .map_err(map_corrupt_or_sqlite_error)?
        .is_some()
    {
        return Err(WorkspaceError::CorruptWorkspace);
    }

    let table_count: i64 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .map_err(map_corrupt_or_sqlite_error)?;
    if table_count != 4 {
        return Err(WorkspaceError::CorruptWorkspace);
    }
    validate_schema_columns(connection)?;

    if !has_foreign_key_group(
        connection,
        "volumes",
        "series",
        &[("series_id", "series_id")],
    )? || !has_foreign_key_group(
        connection,
        "generations",
        "series",
        &[("series_id", "series_id")],
    )? || !has_foreign_key_group(
        connection,
        "generations",
        "generations",
        &[
            ("series_id", "series_id"),
            ("parent_generation_id", "generation_id"),
        ],
    )? || !has_foreign_key_group(
        connection,
        "current_generation",
        "generations",
        &[
            ("series_id", "series_id"),
            ("generation_id", "generation_id"),
        ],
    )? {
        return Err(WorkspaceError::CorruptWorkspace);
    }

    connection
        .query_row(
            "SELECT series_id, created_at_utc_ms FROM series WHERE singleton = 1",
            [],
            |row| {
                let id: String = row.get(0)?;
                let created_at_utc_ms: i64 = row.get(1)?;
                Ok((id, created_at_utc_ms))
            },
        )
        .optional()
        .map_err(map_corrupt_or_sqlite_error)?
        .ok_or(WorkspaceError::CorruptWorkspace)?;

    connection
        .query_row(
            "SELECT generation_id FROM generations WHERE generation_id = 'g0' AND parent_generation_id IS NULL",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(map_corrupt_or_sqlite_error)?
        .ok_or(WorkspaceError::CorruptWorkspace)?;

    let series_count: i64 = connection
        .query_row("SELECT count(*) FROM series", [], |row| row.get(0))
        .map_err(map_corrupt_or_sqlite_error)?;
    let generation_count: i64 = connection
        .query_row("SELECT count(*) FROM generations", [], |row| row.get(0))
        .map_err(map_corrupt_or_sqlite_error)?;
    let current_count: i64 = connection
        .query_row("SELECT count(*) FROM current_generation", [], |row| {
            row.get(0)
        })
        .map_err(map_corrupt_or_sqlite_error)?;
    if series_count != 1 || generation_count != 1 || current_count != 1 {
        return Err(WorkspaceError::CorruptWorkspace);
    }

    let (series_id, created_at_utc_ms): (String, i64) = connection
        .query_row(
            "SELECT series_id, created_at_utc_ms FROM series WHERE singleton = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(map_corrupt_or_sqlite_error)?;
    let current: (String, String) = connection
        .query_row(
            "SELECT series_id, generation_id FROM current_generation WHERE singleton = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(map_corrupt_or_sqlite_error)?;
    if current.0 != series_id || current.1 != BOOTSTRAP_GENERATION || created_at_utc_ms < 0 {
        return Err(WorkspaceError::CorruptWorkspace);
    }
    let series_id = SeriesId::parse(series_id).map_err(|_| WorkspaceError::CorruptWorkspace)?;
    validate_stored_volumes(connection, &series_id)?;
    validate_volume_constraints(connection, series_id.as_str())?;

    drop(
        connection
            .prepare(
                "SELECT series_id, volume_id, display_title, created_at_utc_ms FROM volumes LIMIT 0",
            )
            .map_err(map_corrupt_or_sqlite_error)?,
    );

    Ok(WorkspaceMetadata {
        series: SeriesMetadata {
            id: series_id,
            created_at_utc_ms,
        },
        schema_version: SCHEMA_VERSION,
        current_generation: GenerationId::parse(BOOTSTRAP_GENERATION)
            .map_err(|_| WorkspaceError::CorruptWorkspace)?,
    })
}

fn validate_schema_columns(connection: &Connection) -> Result<(), WorkspaceError> {
    validate_table_columns(
        connection,
        "series",
        &[
            ("singleton", "INTEGER", 0, 1),
            ("series_id", "TEXT", 1, 0),
            ("created_at_utc_ms", "INTEGER", 1, 0),
        ],
    )?;
    validate_table_columns(
        connection,
        "volumes",
        &[
            ("series_id", "TEXT", 1, 1),
            ("volume_id", "TEXT", 1, 2),
            ("display_title", "TEXT", 0, 0),
            ("created_at_utc_ms", "INTEGER", 1, 0),
        ],
    )?;
    validate_table_columns(
        connection,
        "generations",
        &[
            ("series_id", "TEXT", 1, 1),
            ("generation_id", "TEXT", 1, 2),
            ("parent_generation_id", "TEXT", 0, 0),
            ("created_at_utc_ms", "INTEGER", 1, 0),
        ],
    )?;
    validate_table_columns(
        connection,
        "current_generation",
        &[
            ("singleton", "INTEGER", 0, 1),
            ("series_id", "TEXT", 1, 0),
            ("generation_id", "TEXT", 1, 0),
        ],
    )
}

fn validate_table_columns(
    connection: &Connection,
    table: &str,
    expected: &[(&str, &str, i64, i64)],
) -> Result<(), WorkspaceError> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(map_corrupt_or_sqlite_error)?;
    let mut rows = statement.query([]).map_err(map_corrupt_or_sqlite_error)?;
    let mut actual = Vec::new();
    while let Some(row) = rows.next().map_err(map_corrupt_or_sqlite_error)? {
        let name: String = row.get(1).map_err(map_corrupt_or_sqlite_error)?;
        let declared_type: String = row.get(2).map_err(map_corrupt_or_sqlite_error)?;
        let not_null: i64 = row.get(3).map_err(map_corrupt_or_sqlite_error)?;
        let default_value: Option<String> = row.get(4).map_err(map_corrupt_or_sqlite_error)?;
        let primary_key_order: i64 = row.get(5).map_err(map_corrupt_or_sqlite_error)?;
        actual.push((
            name,
            declared_type,
            not_null,
            default_value,
            primary_key_order,
        ));
    }

    if actual.len() != expected.len()
        || actual.iter().zip(expected).any(
            |((name, declared_type, not_null, default_value, primary_key_order), expected)| {
                !name.eq_ignore_ascii_case(expected.0)
                    || !declared_type.eq_ignore_ascii_case(expected.1)
                    || *not_null != expected.2
                    || default_value.is_some()
                    || *primary_key_order != expected.3
            },
        )
    {
        return Err(WorkspaceError::CorruptWorkspace);
    }
    Ok(())
}

fn validate_stored_volumes(
    connection: &Connection,
    series_id: &SeriesId,
) -> Result<(), WorkspaceError> {
    let mut statement = connection
        .prepare("SELECT series_id, volume_id, display_title, created_at_utc_ms FROM volumes")
        .map_err(map_corrupt_or_sqlite_error)?;
    let mut rows = statement.query([]).map_err(map_corrupt_or_sqlite_error)?;
    while let Some(row) = rows.next().map_err(map_corrupt_or_sqlite_error)? {
        let stored_series_id: Value = row.get(0).map_err(map_corrupt_or_sqlite_error)?;
        let stored_series_id = match stored_series_id {
            Value::Text(value) => {
                SeriesId::parse(value).map_err(|_| WorkspaceError::CorruptWorkspace)?
            }
            _ => return Err(WorkspaceError::CorruptWorkspace),
        };
        if &stored_series_id != series_id {
            return Err(WorkspaceError::CorruptWorkspace);
        }

        let volume_id: Value = row.get(1).map_err(map_corrupt_or_sqlite_error)?;
        match volume_id {
            Value::Text(value) => {
                VolumeId::parse(value).map_err(|_| WorkspaceError::CorruptWorkspace)?;
            }
            _ => return Err(WorkspaceError::CorruptWorkspace),
        }

        let display_title: Value = row.get(2).map_err(map_corrupt_or_sqlite_error)?;
        if !matches!(display_title, Value::Null | Value::Text(_)) {
            return Err(WorkspaceError::CorruptWorkspace);
        }

        let created_at_utc_ms: Value = row.get(3).map_err(map_corrupt_or_sqlite_error)?;
        if !matches!(created_at_utc_ms, Value::Integer(value) if value >= 0) {
            return Err(WorkspaceError::CorruptWorkspace);
        }
    }
    Ok(())
}

fn validate_volume_constraints(
    connection: &Connection,
    series_id: &str,
) -> Result<(), WorkspaceError> {
    const SAVEPOINT: &str = "p02_t01_volume_contract_probe";
    connection
        .execute_batch(&format!("SAVEPOINT {SAVEPOINT};"))
        .map_err(map_corrupt_or_sqlite_error)?;

    let validation_result = (|| {
        let valid_volume_id: String = connection
            .query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
            .map_err(map_corrupt_or_sqlite_error)?;
        connection
            .execute(
                "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms) \
                 VALUES (?1, ?2, NULL, 1)",
                rusqlite::params![series_id, valid_volume_id],
            )
            .map_err(map_corrupt_or_sqlite_error)?;

        expect_constraint_rejection(connection.execute(
            "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms) \
             VALUES (?1, ?2, NULL, 1)",
            rusqlite::params![series_id, valid_volume_id],
        ))?;

        let short_volume_id = "a";
        expect_constraint_rejection(connection.execute(
            "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms) \
             VALUES (?1, ?2, NULL, 1)",
            rusqlite::params![series_id, short_volume_id],
        ))?;

        let long_volume_id = format!("{valid_volume_id}0");
        expect_constraint_rejection(connection.execute(
            "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms) \
             VALUES (?1, ?2, NULL, 1)",
            rusqlite::params![series_id, long_volume_id],
        ))?;

        let invalid_volume_id = format!("g{}", &valid_volume_id[1..]);
        expect_constraint_rejection(connection.execute(
            "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms) \
             VALUES (?1, ?2, NULL, 1)",
            rusqlite::params![series_id, invalid_volume_id],
        ))?;

        let negative_time_id: String = connection
            .query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
            .map_err(map_corrupt_or_sqlite_error)?;
        expect_constraint_rejection(connection.execute(
            "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms) \
             VALUES (?1, ?2, NULL, -1)",
            rusqlite::params![series_id, negative_time_id],
        ))?;

        let null_series_volume_id: String = connection
            .query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
            .map_err(map_corrupt_or_sqlite_error)?;
        expect_constraint_rejection(connection.execute(
            "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms) \
             VALUES (NULL, ?1, NULL, 1)",
            [null_series_volume_id],
        ))?;

        expect_constraint_rejection(connection.execute(
            "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms) \
             VALUES (?1, NULL, NULL, 1)",
            [series_id],
        ))?;

        let mut orphan_series_id: String = connection
            .query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
            .map_err(map_corrupt_or_sqlite_error)?;
        if orphan_series_id == series_id {
            orphan_series_id.replace_range(0..1, "f");
        }
        let orphan_volume_id: String = connection
            .query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
            .map_err(map_corrupt_or_sqlite_error)?;
        expect_constraint_rejection(connection.execute(
            "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms) \
             VALUES (?1, ?2, NULL, 1)",
            rusqlite::params![orphan_series_id, orphan_volume_id],
        ))?;
        Ok(())
    })();

    let rollback_result = connection
        .execute_batch(&format!("ROLLBACK TO {SAVEPOINT}; RELEASE {SAVEPOINT};"))
        .map_err(map_corrupt_or_sqlite_error);
    match (validation_result, rollback_result) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(()),
    }
}

fn expect_constraint_rejection(
    result: Result<usize, rusqlite::Error>,
) -> Result<(), WorkspaceError> {
    match result {
        Err(rusqlite::Error::SqliteFailure(code, _))
            if code.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            Ok(())
        }
        Err(error) => Err(map_corrupt_or_sqlite_error(error)),
        Ok(_) => Err(WorkspaceError::CorruptWorkspace),
    }
}

fn validate_schema(connection: &Connection) -> Result<(), WorkspaceError> {
    validate_database(connection).map(|_| ())
}

fn has_foreign_key_group(
    connection: &Connection,
    table: &str,
    referenced_table: &str,
    columns: &[(&str, &str)],
) -> Result<bool, WorkspaceError> {
    let mut statement = connection
        .prepare(&format!("PRAGMA foreign_key_list({table})"))
        .map_err(map_corrupt_or_sqlite_error)?;
    let mut rows = statement.query([]).map_err(map_corrupt_or_sqlite_error)?;
    let mut groups: BTreeMap<i64, Vec<(i64, String, String, String)>> = BTreeMap::new();
    while let Some(row) = rows.next().map_err(map_corrupt_or_sqlite_error)? {
        let id: i64 = row.get(0).map_err(map_corrupt_or_sqlite_error)?;
        let sequence: i64 = row.get(1).map_err(map_corrupt_or_sqlite_error)?;
        let target: String = row.get(2).map_err(map_corrupt_or_sqlite_error)?;
        let from: String = row.get(3).map_err(map_corrupt_or_sqlite_error)?;
        let to: String = row.get(4).map_err(map_corrupt_or_sqlite_error)?;
        groups
            .entry(id)
            .or_default()
            .push((sequence, target, from, to));
    }

    let mut expected: Vec<_> = columns
        .iter()
        .enumerate()
        .map(|(sequence, (from, to))| {
            (
                sequence as i64,
                referenced_table.to_owned(),
                (*from).to_owned(),
                (*to).to_owned(),
            )
        })
        .collect();
    expected.sort();
    Ok(groups.into_values().any(|mut group| {
        group.sort();
        group == expected
    }))
}

fn now_utc_ms() -> Result<i64, WorkspaceError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| WorkspaceError::StorageFailure)?;
    i64::try_from(duration.as_millis()).map_err(|_| WorkspaceError::StorageFailure)
}

fn map_io_error(error: std::io::Error) -> WorkspaceError {
    #[cfg(windows)]
    if matches!(error.raw_os_error(), Some(5) | Some(32) | Some(33)) {
        return WorkspaceError::PermissionDenied;
    }

    match error.kind() {
        ErrorKind::NotFound => WorkspaceError::NotFound,
        ErrorKind::PermissionDenied => WorkspaceError::PermissionDenied,
        ErrorKind::AlreadyExists => WorkspaceError::AlreadyExists,
        _ => WorkspaceError::StorageFailure,
    }
}

fn map_corrupt_or_sqlite_error(error: rusqlite::Error) -> WorkspaceError {
    match &error {
        rusqlite::Error::SqliteFailure(code, _)
            if code.code == rusqlite::ErrorCode::NotADatabase =>
        {
            WorkspaceError::CorruptWorkspace
        }
        rusqlite::Error::SqliteFailure(code, _)
            if code.code == rusqlite::ErrorCode::DatabaseCorrupt =>
        {
            WorkspaceError::CorruptWorkspace
        }
        rusqlite::Error::SqliteFailure(code, _) if code.code == rusqlite::ErrorCode::Unknown => {
            WorkspaceError::CorruptWorkspace
        }
        _ => map_sqlite_error(error),
    }
}

fn map_sqlite_error(error: rusqlite::Error) -> WorkspaceError {
    match &error {
        rusqlite::Error::SqliteFailure(code, _)
            if code.code == rusqlite::ErrorCode::PermissionDenied
                || code.code == rusqlite::ErrorCode::ReadOnly =>
        {
            WorkspaceError::PermissionDenied
        }
        rusqlite::Error::SqliteFailure(code, _)
            if code.code == rusqlite::ErrorCode::DatabaseBusy =>
        {
            WorkspaceError::StorageFailure
        }
        _ => WorkspaceError::StorageFailure,
    }
}
