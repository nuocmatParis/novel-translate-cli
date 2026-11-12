use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use nt_application::workspace::{WorkspaceError, create_workspace, open_workspace};
use nt_storage::SqliteWorkspaceRepository;
use rusqlite::{Connection, OpenFlags};

const DATABASE_FILE: &str = "series.sqlite3";
const APPLICATION_ID: i64 = 0x4E54_5253;
static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nt-storage-p02-t01-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create isolated test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn directory_snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fs::read_dir(root)
        .expect("read fixture directory")
        .map(|entry| {
            let entry = entry.expect("read fixture entry");
            let name = entry.file_name().to_string_lossy().into_owned();
            let contents = fs::read(entry.path()).expect("read fixture file");
            (name, contents)
        })
        .collect()
}

fn sidecar_path(database_path: &Path, suffix: &str) -> PathBuf {
    let mut path = database_path.as_os_str().to_os_string();
    path.push(suffix);
    PathBuf::from(path)
}

fn create_workspace_without_volume_length_check(root: &Path, volume_ids: &[&str]) {
    let created =
        create_workspace(&SqliteWorkspaceRepository, root).expect("create valid workspace");
    let database_path = root.join(DATABASE_FILE);
    let connection = Connection::open(&database_path).expect("open schema fixture");
    connection
        .execute_batch(
            "PRAGMA foreign_keys = OFF;
             DROP TABLE volumes;
             CREATE TABLE volumes (
                 series_id TEXT NOT NULL,
                 volume_id TEXT NOT NULL CHECK(volume_id NOT GLOB '*[^0-9a-f]*'),
                 display_title TEXT,
                 created_at_utc_ms INTEGER NOT NULL CHECK(created_at_utc_ms >= 0),
                 PRIMARY KEY (series_id, volume_id),
                 FOREIGN KEY (series_id) REFERENCES series(series_id) ON DELETE RESTRICT
             );",
        )
        .expect("replace Volume table without the length check");
    for volume_id in volume_ids {
        connection
            .execute(
                "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms) \
                 VALUES (?1, ?2, ?3, 42)",
                (
                    created.series.id.as_str(),
                    volume_id,
                    Some("fixture Volume"),
                ),
            )
            .expect("insert fixture Volume metadata");
    }
    drop(connection);
}

#[test]
fn create_and_open_round_trip_series_schema_and_bootstrap_generation() {
    let temp = TestDirectory::new();
    let root = temp.path().join("series");
    let repository = SqliteWorkspaceRepository;

    let created = create_workspace(&repository, &root).expect("create Rust workspace");
    assert_eq!(created.schema_version, 1);
    assert_eq!(created.current_generation.as_str(), "g0");
    assert_eq!(created.series.id.as_str().len(), 32);
    assert!(created.series.created_at_utc_ms > 0);

    let database_path = root.join(DATABASE_FILE);
    let connection = Connection::open_with_flags(
        &database_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("open the created database through a fresh connection");
    let application_id: i64 = connection
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .expect("read the Rust format marker");
    let schema_version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("read schema version");
    let series_count: i64 = connection
        .query_row("SELECT count(*) FROM series", [], |row| row.get(0))
        .expect("count Series rows");
    let bootstrap_count: i64 = connection
        .query_row(
            "SELECT count(*) FROM generations WHERE generation_id = 'g0' AND parent_generation_id IS NULL",
            [],
            |row| row.get(0),
        )
        .expect("count bootstrap generations");
    let current_count: i64 = connection
        .query_row(
            "SELECT count(*) FROM current_generation WHERE singleton = 1 AND generation_id = 'g0'",
            [],
            |row| row.get(0),
        )
        .expect("count current pointers");
    assert_eq!(application_id, APPLICATION_ID);
    assert_eq!(schema_version, 1);
    assert_eq!(series_count, 1);
    assert_eq!(bootstrap_count, 1);
    assert_eq!(current_count, 1);
    drop(connection);

    let opened = open_workspace(&repository, &root).expect("open workspace on a new connection");
    assert_eq!(opened, created);
}

#[test]
fn create_rejects_a_sentinel_directory_without_changing_its_bytes() {
    let temp = TestDirectory::new();
    let root = temp.path().join("occupied");
    fs::create_dir(&root).expect("create occupied root");
    let sentinel = root.join("sentinel.bin");
    let original = b"keep these bytes exactly";
    fs::write(&sentinel, original).expect("write sentinel");

    let error = create_workspace(&SqliteWorkspaceRepository, &root)
        .expect_err("refuse to create inside a non-empty directory");
    assert_eq!(error, WorkspaceError::DirectoryNotEmpty);
    assert_eq!(
        fs::read(&sentinel).expect("read sentinel after rejection"),
        original
    );
    assert!(!root.join(DATABASE_FILE).exists());
}

#[test]
fn open_distinguishes_missing_legacy_corrupt_and_newer_workspaces_without_mutation() {
    let temp = TestDirectory::new();
    let repository = SqliteWorkspaceRepository;

    let missing = temp.path().join("missing");
    assert_eq!(
        open_workspace(&repository, &missing).expect_err("missing root must fail"),
        WorkspaceError::NotFound
    );
    assert!(!missing.exists(), "Open must not create a root");

    let legacy = temp.path().join("legacy");
    fs::create_dir(&legacy).expect("create legacy fixture root");
    let legacy_marker = legacy.join("workspace.yaml");
    let legacy_bytes = b"go workspace fixture";
    fs::write(&legacy_marker, legacy_bytes).expect("write legacy marker fixture");
    assert_eq!(
        open_workspace(&repository, &legacy).expect_err("legacy workspace must be rejected"),
        WorkspaceError::LegacyGoWorkspace
    );
    assert_eq!(
        fs::read(&legacy_marker).expect("read legacy marker"),
        legacy_bytes
    );
    assert!(!legacy.join(DATABASE_FILE).exists());

    let corrupt = temp.path().join("corrupt");
    fs::create_dir(&corrupt).expect("create corrupt fixture root");
    let corrupt_path = corrupt.join(DATABASE_FILE);
    let corrupt_bytes = b"not a sqlite database";
    fs::write(&corrupt_path, corrupt_bytes).expect("write corrupt fixture");
    assert_eq!(
        open_workspace(&repository, &corrupt).expect_err("corrupt database must fail"),
        WorkspaceError::CorruptWorkspace
    );
    assert_eq!(
        fs::read(&corrupt_path).expect("read corrupt fixture"),
        corrupt_bytes
    );

    let newer = temp.path().join("newer");
    fs::create_dir(&newer).expect("create newer fixture root");
    let newer_path = newer.join(DATABASE_FILE);
    let connection = Connection::open(&newer_path).expect("create newer schema fixture");
    connection
        .pragma_update(None, "application_id", APPLICATION_ID)
        .expect("set Rust marker");
    connection
        .pragma_update(None, "user_version", 2)
        .expect("set future schema version");
    drop(connection);
    let newer_bytes = fs::read(&newer_path).expect("capture future schema bytes");
    let shm_path = sidecar_path(&newer_path, "-shm");
    let shm_sentinel = b"preserve pre-existing shared-memory bytes";
    fs::write(&shm_path, shm_sentinel).expect("write SHM sentinel");
    let newer_files = directory_snapshot(&newer);
    assert_eq!(
        open_workspace(&repository, &newer).expect_err("newer schema must fail"),
        WorkspaceError::NewerSchema {
            found: 2,
            supported: 1,
        }
    );
    assert_eq!(
        fs::read(&newer_path).expect("read future schema after rejection"),
        newer_bytes
    );
    assert_eq!(directory_snapshot(&newer), newer_files);
}

#[cfg(windows)]
#[test]
fn rejected_newer_wal_workspace_preserves_every_source_file() {
    let temp = TestDirectory::new();
    let root = temp.path().join("newer-wal");
    let created = create_workspace(&SqliteWorkspaceRepository, &root).expect("create workspace");
    let database_path = root.join(DATABASE_FILE);

    let writer = Connection::open(&database_path).expect("open WAL writer fixture");
    writer
        .execute_batch("PRAGMA journal_mode = WAL;")
        .expect("keep fixture in WAL mode");
    let reader = Connection::open_with_flags(
        &database_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("open read-only WAL reader");
    reader
        .execute_batch("BEGIN;")
        .expect("begin reader snapshot");
    let old_version: i64 = reader
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("read version from reader snapshot");
    assert_eq!(old_version, 1);

    writer
        .pragma_update(None, "user_version", 2)
        .expect("commit newer schema version into WAL");
    drop(writer);
    let wal_path = sidecar_path(&database_path, "-wal");
    assert!(
        fs::metadata(&wal_path).expect("read WAL metadata").len() > 32,
        "committed version must remain in WAL while the reader holds its snapshot"
    );

    let before = directory_snapshot(&root);
    assert_eq!(
        open_workspace(&SqliteWorkspaceRepository, &root)
            .expect_err("Open must read the newer committed WAL state"),
        WorkspaceError::NewerSchema {
            found: 2,
            supported: 1,
        }
    );
    assert_eq!(directory_snapshot(&root), before);
    drop(reader);
    assert_eq!(created.schema_version, 1);
}

#[test]
fn volume_foreign_key_rejects_a_different_series_on_a_real_connection() {
    let temp = TestDirectory::new();
    let root = temp.path().join("series");
    let created = create_workspace(&SqliteWorkspaceRepository, &root).expect("create workspace");
    let connection = Connection::open(root.join(DATABASE_FILE)).expect("open real SQLite database");
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .expect("enable foreign keys on this connection");
    let foreign_keys: i64 = connection
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .expect("read connection foreign-key setting");
    assert_eq!(foreign_keys, 1);

    let valid_volume_id = "11111111111111111111111111111111";
    connection
        .execute(
            "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms) \
             VALUES (?1, ?2, ?3, ?4)",
            (
                created.series.id.as_str(),
                valid_volume_id,
                Some("Volume 1"),
                1_i64,
            ),
        )
        .expect("insert valid Series Volume metadata");
    let invalid_volume_id = "22222222222222222222222222222222";
    assert!(
        connection
            .execute(
                "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms) \
             VALUES (?1, ?2, ?3, ?4)",
                (
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    invalid_volume_id,
                    Some("Wrong Series"),
                    2_i64,
                ),
            )
            .is_err()
    );

    let (volume_id, title): (String, Option<String>) = connection
        .query_row("SELECT volume_id, display_title FROM volumes", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .expect("read the valid Volume metadata back");
    assert_eq!(volume_id, valid_volume_id);
    assert_eq!(title.as_deref(), Some("Volume 1"));
    let count: i64 = connection
        .query_row("SELECT count(*) FROM volumes", [], |row| row.get(0))
        .expect("count valid Volume metadata");
    assert_eq!(count, 1);
}

#[test]
fn open_rejects_volume_schema_without_contract_constraints() {
    let temp = TestDirectory::new();
    let root = temp.path().join("invalid-volumes");
    create_workspace(&SqliteWorkspaceRepository, &root).expect("create valid workspace");
    let database_path = root.join(DATABASE_FILE);
    let connection = Connection::open(&database_path).expect("open schema fixture");
    connection
        .execute_batch(
            "PRAGMA foreign_keys = OFF;
             DROP TABLE volumes;
             CREATE TABLE volumes (
                 series_id TEXT REFERENCES series(series_id),
                 volume_id TEXT,
                 display_title TEXT,
                 created_at_utc_ms INTEGER
             );
             INSERT INTO volumes VALUES (NULL, 'invalid-id', 'orphan Volume', -1);",
        )
        .expect("remove required Volume constraints and insert invalid metadata");
    drop(connection);

    let before = directory_snapshot(&root);
    assert_eq!(
        open_workspace(&SqliteWorkspaceRepository, &root)
            .expect_err("Open must reject a schema without its Volume contract"),
        WorkspaceError::CorruptWorkspace
    );
    assert_eq!(directory_snapshot(&root), before);
}

#[test]
fn open_classifies_a_missing_required_volume_column_as_corrupt() {
    let temp = TestDirectory::new();
    let root = temp.path().join("missing-volume-column");
    create_workspace(&SqliteWorkspaceRepository, &root).expect("create valid workspace");
    let database_path = root.join(DATABASE_FILE);
    let connection = Connection::open(&database_path).expect("open schema fixture");
    connection
        .execute_batch("ALTER TABLE volumes RENAME COLUMN display_title TO broken_title;")
        .expect("remove a required schema column");
    drop(connection);

    assert_eq!(
        open_workspace(&SqliteWorkspaceRepository, &root)
            .expect_err("missing schema column is workspace corruption"),
        WorkspaceError::CorruptWorkspace
    );
}

#[test]
fn open_accepts_a_valid_volume_row_and_preserves_its_metadata() {
    let temp = TestDirectory::new();
    let root = temp.path().join("valid-volume");
    let created =
        create_workspace(&SqliteWorkspaceRepository, &root).expect("create valid workspace");
    let database_path = root.join(DATABASE_FILE);
    let volume_id = "11111111111111111111111111111111";
    let connection = Connection::open(&database_path).expect("open workspace writer");
    connection
        .execute(
            "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms) \
             VALUES (?1, ?2, ?3, ?4)",
            (
                created.series.id.as_str(),
                volume_id,
                Some("Volume 1"),
                1_i64,
            ),
        )
        .expect("insert valid Volume metadata");
    drop(connection);

    let opened = open_workspace(&SqliteWorkspaceRepository, &root)
        .expect("open workspace with a valid Volume row");
    assert_eq!(opened, created);
    let connection = Connection::open_with_flags(
        &database_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("read accepted Volume metadata");
    let volume: (String, Option<String>, i64) = connection
        .query_row(
            "SELECT volume_id, display_title, created_at_utc_ms FROM volumes",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("read valid Volume row");
    assert_eq!(
        volume,
        (volume_id.to_owned(), Some("Volume 1".to_owned()), 1)
    );
}

#[test]
fn open_rejects_volume_schema_missing_only_id_length_check() {
    let temp = TestDirectory::new();
    let root = temp.path().join("volume-no-length-check");
    create_workspace_without_volume_length_check(&root, &[]);
    let before = directory_snapshot(&root);

    assert_eq!(
        open_workspace(&SqliteWorkspaceRepository, &root)
            .expect_err("Open must require the exact Volume ID length check"),
        WorkspaceError::CorruptWorkspace
    );
    assert_eq!(directory_snapshot(&root), before);
}

#[test]
fn open_rejects_short_and_long_volume_ids_in_stored_metadata() {
    let temp = TestDirectory::new();
    let short_id = "a";
    let long_id = "a".repeat(33);
    for (name, volume_id) in [("short", short_id), ("long", long_id.as_str())] {
        let root = temp.path().join(format!("invalid-volume-id-{name}"));
        create_workspace_without_volume_length_check(&root, &[volume_id]);
        let before = directory_snapshot(&root);

        assert_eq!(
            open_workspace(&SqliteWorkspaceRepository, &root)
                .expect_err("Open must reject a stored Volume ID outside the format"),
            WorkspaceError::CorruptWorkspace
        );
        assert_eq!(directory_snapshot(&root), before);
    }
}

#[cfg(windows)]
#[test]
fn open_reports_permission_denied_when_a_process_blocks_write_access() {
    use std::os::windows::fs::OpenOptionsExt;

    let temp = TestDirectory::new();
    let root = temp.path().join("locked");
    create_workspace(&SqliteWorkspaceRepository, &root).expect("create workspace");
    let database_path = root.join(DATABASE_FILE);
    let original = fs::read(&database_path).expect("capture database bytes");
    let held = OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0x0000_0001)
        .open(&database_path)
        .expect("hold a read handle that denies later write opens");

    assert_eq!(
        open_workspace(&SqliteWorkspaceRepository, &root)
            .expect_err("open must report the unavailable writer access"),
        WorkspaceError::PermissionDenied
    );
    drop(held);
    assert_eq!(
        fs::read(&database_path).expect("read database after rejection"),
        original
    );
}

#[cfg(windows)]
#[test]
fn open_reports_permission_denied_when_a_process_blocks_read_access() {
    use std::os::windows::fs::OpenOptionsExt;

    let temp = TestDirectory::new();
    let root = temp.path().join("read-locked");
    create_workspace(&SqliteWorkspaceRepository, &root).expect("create workspace");
    let database_path = root.join(DATABASE_FILE);
    let original = fs::read(&database_path).expect("capture database bytes");
    let held = OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(&database_path)
        .expect("hold an exclusive handle");

    assert_eq!(
        open_workspace(&SqliteWorkspaceRepository, &root)
            .expect_err("open must report the unavailable read access"),
        WorkspaceError::PermissionDenied
    );
    drop(held);
    assert_eq!(
        fs::read(&database_path).expect("read database after rejection"),
        original
    );
}

#[cfg(windows)]
#[test]
fn create_rejects_unc_paths_before_touching_the_network() {
    let error = create_workspace(
        &SqliteWorkspaceRepository,
        Path::new(r"\\example.invalid\share\workspace"),
    )
    .expect_err("UNC/network paths are unsupported");
    assert_eq!(error, WorkspaceError::UnsupportedFilesystem);
}
