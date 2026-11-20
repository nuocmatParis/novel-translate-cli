//! Version-three schema and staged migration from an actual version-two database.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use nt_application::object_store::ImmutableObjectStore;
use nt_application::workspace::WorkspaceError;
use nt_domain::object::ObjectHash;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};

use super::{create_private_directory, ensure_same_workspace};
use crate::immutable_store::FileObjectStore;
use crate::workspace::{configure_connection, map_io_error, map_sqlite_error, validate_database};

pub(super) const M003_SCHEMA: &str = r#"
CREATE TABLE cp05_volume_versions (
    series_id TEXT NOT NULL,
    generation_id TEXT NOT NULL,
    volume_id TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('unimported', 'imported')),
    mode TEXT NOT NULL CHECK (mode IN ('all-text', 'story')),
    epub_sha256 BLOB,
    mapping_version BLOB,
    payload BLOB,
    payload_sha256 BLOB,
    PRIMARY KEY (series_id, generation_id, volume_id),
    FOREIGN KEY (series_id, generation_id)
        REFERENCES generations(series_id, generation_id) ON DELETE RESTRICT,
    FOREIGN KEY (series_id, volume_id)
        REFERENCES volumes(series_id, volume_id) ON DELETE RESTRICT,
    CHECK (
        (state = 'unimported' AND epub_sha256 IS NULL AND mapping_version IS NULL
         AND payload IS NULL AND payload_sha256 IS NULL) OR
        (state = 'imported' AND typeof(epub_sha256) = 'blob' AND length(epub_sha256) = 32
         AND typeof(mapping_version) = 'blob' AND length(mapping_version) = 32
         AND typeof(payload) = 'blob' AND length(payload) BETWEEN 1 AND 536870912
         AND typeof(payload_sha256) = 'blob' AND length(payload_sha256) = 32)
    )
);
CREATE TABLE cp05_resource_refs (
    series_id TEXT NOT NULL,
    generation_id TEXT NOT NULL,
    volume_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK (length(role) > 0),
    canonical_path TEXT NOT NULL CHECK (length(canonical_path) > 0),
    object_hash TEXT NOT NULL
        CHECK (length(object_hash) = 64 AND object_hash NOT GLOB '*[^0-9a-f]*'),
    byte_len INTEGER NOT NULL CHECK (byte_len >= 0),
    source_relative_path TEXT,
    PRIMARY KEY (series_id, generation_id, volume_id, role, canonical_path),
    FOREIGN KEY (series_id, generation_id, volume_id)
        REFERENCES cp05_volume_versions(series_id, generation_id, volume_id) ON DELETE RESTRICT,
    FOREIGN KEY (series_id, generation_id, object_hash)
        REFERENCES generation_object_refs(series_id, generation_id, object_hash) ON DELETE RESTRICT,
    CHECK (source_relative_path IS NULL OR length(source_relative_path) > 0)
);
CREATE TABLE cp05_series_defaults (
    series_id TEXT NOT NULL,
    generation_id TEXT NOT NULL,
    mode TEXT NOT NULL CHECK (mode IN ('all-text', 'story')),
    PRIMARY KEY (series_id, generation_id),
    FOREIGN KEY (series_id, generation_id)
        REFERENCES generations(series_id, generation_id) ON DELETE RESTRICT
);
"#;

pub(crate) fn apply_m003(transaction: &Transaction<'_>) -> Result<(), WorkspaceError> {
    transaction
        .execute_batch(M003_SCHEMA)
        .map_err(map_sqlite_error)?;
    transaction
        .execute_batch(
            "INSERT INTO cp05_volume_versions
             (series_id, generation_id, volume_id, state, mode)
             SELECT v.series_id, c.generation_id, v.volume_id, 'unimported', 'all-text'
             FROM volumes v JOIN current_generation c ON c.series_id = v.series_id;
             INSERT INTO cp05_series_defaults (series_id, generation_id, mode)
             SELECT series_id, generation_id, 'all-text' FROM current_generation;",
        )
        .map_err(map_sqlite_error)?;
    transaction
        .pragma_update(None, "user_version", 3_u32)
        .map_err(map_sqlite_error)
}

/// Require the exact version-three owned table definitions, including CHECK and
/// composite FK clauses. Existing v1/v2 validation remains version-specific.
pub(crate) fn validate_m003(
    connection: &Connection,
    _probe_write_constraints: bool,
) -> Result<(), WorkspaceError> {
    let expected = Connection::open_in_memory().map_err(map_sqlite_error)?;
    expected
        .execute_batch(M003_SCHEMA)
        .map_err(map_sqlite_error)?;
    for name in [
        "cp05_volume_versions",
        "cp05_resource_refs",
        "cp05_series_defaults",
    ] {
        let sql = |database: &Connection| -> Result<Option<String>, WorkspaceError> {
            database
                .query_row(
                    "SELECT sql FROM sqlite_schema WHERE type = 'table' AND name = ?1",
                    [name],
                    |row| row.get(0),
                )
                .optional()
                .map_err(map_sqlite_error)
        };
        let normalize = |value: String| value.split_whitespace().collect::<Vec<_>>().join(" ");
        if sql(connection)?.map(normalize) != sql(&expected)?.map(normalize) {
            return Err(WorkspaceError::CorruptWorkspace);
        }
    }
    let mismatched_lengths: i64 = connection
        .query_row(
            "SELECT count(*) FROM cp05_resource_refs r
             LEFT JOIN generation_object_refs o
             ON o.series_id = r.series_id AND o.generation_id = r.generation_id
                AND o.object_hash = r.object_hash
             WHERE o.object_hash IS NULL OR o.byte_len <> r.byte_len",
            [],
            |row| row.get(0),
        )
        .map_err(map_sqlite_error)?;
    if mismatched_lengths != 0 {
        return Err(WorkspaceError::CorruptWorkspace);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MigrationFailpoint {
    Backup,
    Staging,
    SourceDdl,
}

pub(crate) fn migrate_v2_to_v3(
    root: &Path,
    connection: &mut Connection,
) -> Result<(), WorkspaceError> {
    migrate_v2_to_v3_with_failpoint(root, connection, None)
}

fn migrate_v2_to_v3_with_failpoint(
    root: &Path,
    connection: &mut Connection,
    failpoint: Option<MigrationFailpoint>,
) -> Result<(), WorkspaceError> {
    migrate_with_hook(root, connection, failpoint, || Ok(()))
}

fn migrate_with_hook(
    root: &Path,
    connection: &mut Connection,
    failpoint: Option<MigrationFailpoint>,
    after_staging: impl FnOnce() -> Result<(), WorkspaceError>,
) -> Result<(), WorkspaceError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(map_sqlite_error)?;
    let version: i64 = transaction
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(map_sqlite_error)?;
    if matches!(version, 3 | 4) {
        return validate_database(&transaction).map(|_| ());
    }
    if version != 2 {
        return Err(WorkspaceError::UnsupportedFormat);
    }
    let before = validate_database(&transaction)?;
    let backup_directory = create_private_directory(root, "migration-backup")?;
    let stage_directory = create_private_directory(root, "migration-stage")?;
    let backup_path = backup_directory.join("series.sqlite3");
    let stage_path = stage_directory.join("series.sqlite3");

    let prepared = (|| {
        super::create_migration_backup(&transaction, &backup_path)?;
        let backup = Connection::open_with_flags(
            &backup_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(map_sqlite_error)?;
        configure_connection(&backup)?;
        let backup_metadata = validate_database(&backup)?;
        if matches!(backup_metadata.schema_version, 3 | 4) {
            return Err(WorkspaceError::Busy);
        }
        ensure_same_workspace(&before, &backup_metadata)?;
        if backup_metadata.schema_version != 2 {
            return Err(WorkspaceError::CorruptWorkspace);
        }
        let expected_digest = v2_content_digest(&backup)?;
        let pinned_objects = backup_referenced_objects(root, &backup_directory, &backup)?;
        let mut manifest = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(backup_directory.join("manifest.sha256"))
            .map_err(map_io_error)?;
        let (database_hash, database_len) = hash_file(&backup_path)?;
        writeln!(manifest, "{database_hash}  {database_len}  series.sqlite3")
            .map_err(map_io_error)?;
        for (hash, length) in referenced_objects(&backup)? {
            writeln!(manifest, "{hash}  {length}  objects/{hash}.blob").map_err(map_io_error)?;
        }
        manifest.sync_all().map_err(map_io_error)?;

        if failpoint == Some(MigrationFailpoint::Backup) {
            return Err(WorkspaceError::StorageFailure);
        }
        fs::copy(&backup_path, &stage_path).map_err(map_io_error)?;
        let mut stage = Connection::open_with_flags(
            &stage_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(map_sqlite_error)?;
        configure_connection(&stage)?;
        let transaction = stage
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(map_sqlite_error)?;
        apply_m003(&transaction)?;
        transaction.commit().map_err(map_sqlite_error)?;
        let staged = validate_database(&stage)?;
        ensure_same_workspace(&backup_metadata, &staged)?;
        if staged.schema_version != 3 {
            return Err(WorkspaceError::CorruptWorkspace);
        }
        Ok((expected_digest, staged, pinned_objects))
    })();
    let _ = fs::remove_dir_all(&stage_directory);
    let (expected_digest, staged, _pinned_objects) = prepared?;

    if failpoint == Some(MigrationFailpoint::Staging) {
        return Err(WorkspaceError::StorageFailure);
    }
    after_staging()?;
    if v2_content_digest(&transaction)? != expected_digest {
        return Err(WorkspaceError::Busy);
    }
    apply_m003(&transaction)?;
    #[cfg(test)]
    if std::env::var_os("NT_CP05_TEST_CRASH_AFTER_M003_DDL").is_some() {
        std::process::exit(93);
    }
    if failpoint == Some(MigrationFailpoint::SourceDdl) {
        return Err(WorkspaceError::StorageFailure);
    }
    let actual = validate_database(&transaction)?;
    ensure_same_workspace(&staged, &actual)?;
    transaction.commit().map_err(map_sqlite_error)?;
    let final_metadata = validate_database(connection)?;
    ensure_same_workspace(&staged, &final_metadata)
}

fn referenced_objects(
    connection: &Connection,
) -> Result<BTreeMap<ObjectHash, u64>, WorkspaceError> {
    let mut query = connection
        .prepare("SELECT object_hash, byte_len FROM generation_object_refs ORDER BY object_hash")
        .map_err(map_sqlite_error)?;
    let mut rows = query.query([]).map_err(map_sqlite_error)?;
    let mut objects = BTreeMap::new();
    while let Some(row) = rows.next().map_err(map_sqlite_error)? {
        let hash: String = row.get(0).map_err(map_sqlite_error)?;
        let length: i64 = row.get(1).map_err(map_sqlite_error)?;
        let hash = ObjectHash::parse(&hash).map_err(|_| WorkspaceError::CorruptWorkspace)?;
        let length = u64::try_from(length).map_err(|_| WorkspaceError::CorruptWorkspace)?;
        if objects
            .insert(hash, length)
            .is_some_and(|old| old != length)
        {
            return Err(WorkspaceError::CorruptWorkspace);
        }
    }
    Ok(objects)
}

fn backup_referenced_objects(
    root: &Path,
    backup: &Path,
    connection: &Connection,
) -> Result<Vec<Box<dyn Read + Send>>, WorkspaceError> {
    let references = referenced_objects(connection)?;
    let mut pinned = Vec::with_capacity(references.len());
    if references.is_empty() {
        return Ok(pinned);
    }
    let destination = backup.join("objects");
    fs::create_dir(&destination).map_err(map_io_error)?;
    let store = FileObjectStore::new(root).map_err(|_| WorkspaceError::CorruptWorkspace)?;
    for (hash, expected_len) in references {
        let mut source = store
            .open_verified(hash)
            .map_err(|_| WorkspaceError::CorruptWorkspace)?;
        let path = destination.join(format!("{hash}.blob"));
        let mut copy = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(map_io_error)?;
        let mut digest = Sha256::new();
        let mut total = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let count = source.read(&mut buffer).map_err(map_io_error)?;
            if count == 0 {
                break;
            }
            total = total
                .checked_add(count as u64)
                .ok_or(WorkspaceError::StorageFailure)?;
            digest.update(&buffer[..count]);
            copy.write_all(&buffer[..count]).map_err(map_io_error)?;
        }
        copy.sync_all().map_err(map_io_error)?;
        let actual: [u8; 32] = digest.finalize().into();
        if actual != *hash.as_bytes()
            || total != expected_len
            || hash_file(&path)? != (hash.to_string(), total)
        {
            return Err(WorkspaceError::CorruptWorkspace);
        }
        pinned.push(source);
    }
    Ok(pinned)
}

pub(super) fn hash_file(path: &Path) -> Result<(String, u64), WorkspaceError> {
    let mut file = File::open(path).map_err(map_io_error)?;
    let mut hash = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(map_io_error)?;
        if count == 0 {
            break;
        }
        total = total
            .checked_add(count as u64)
            .ok_or(WorkspaceError::StorageFailure)?;
        hash.update(&buffer[..count]);
    }
    let digest: [u8; 32] = hash.finalize().into();
    Ok((ObjectHash::from_bytes(digest).to_string(), total))
}

/// Hash every v2 table row and schema definition in a deterministic order;
/// metadata alone would miss concurrent Volume or ledger changes.
fn v2_content_digest(connection: &Connection) -> Result<[u8; 32], WorkspaceError> {
    use rusqlite::types::ValueRef;
    let mut digest = Sha256::new();
    for sql in [
        "SELECT type, name, tbl_name, sql FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name",
        "SELECT singleton, series_id, created_at_utc_ms FROM series ORDER BY singleton",
        "SELECT series_id, volume_id, display_title, created_at_utc_ms FROM volumes ORDER BY series_id, volume_id",
        "SELECT series_id, generation_id, parent_generation_id, created_at_utc_ms FROM generations ORDER BY series_id, generation_id",
        "SELECT singleton, series_id, generation_id FROM current_generation ORDER BY singleton",
        "SELECT series_id, operation_id, request_fingerprint, expected_parent_generation_id, result_generation_id, committed_at_utc_ms FROM generation_operations ORDER BY series_id, operation_id",
        "SELECT series_id, generation_id, object_hash, byte_len FROM generation_object_refs ORDER BY series_id, generation_id, object_hash",
    ] {
        digest.update((sql.len() as u64).to_be_bytes());
        digest.update(sql.as_bytes());
        let mut statement = connection.prepare(sql).map_err(map_sqlite_error)?;
        let columns = statement.column_count();
        let mut rows = statement.query([]).map_err(map_sqlite_error)?;
        while let Some(row) = rows.next().map_err(map_sqlite_error)? {
            digest.update([0xff]);
            for index in 0..columns {
                match row.get_ref(index).map_err(map_sqlite_error)? {
                    ValueRef::Null => digest.update([0]),
                    ValueRef::Integer(value) => {
                        digest.update([1]);
                        digest.update(value.to_be_bytes());
                    }
                    ValueRef::Real(value) => {
                        digest.update([2]);
                        digest.update(value.to_bits().to_be_bytes());
                    }
                    ValueRef::Text(value) | ValueRef::Blob(value) => {
                        let tag = if matches!(row.get_ref(index), Ok(ValueRef::Text(_))) {
                            3
                        } else {
                            4
                        };
                        digest.update([tag]);
                        digest.update((value.len() as u64).to_be_bytes());
                        digest.update(value);
                    }
                }
            }
        }
    }
    Ok(digest.finalize().into())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use nt_application::workspace::{WorkspaceError, WorkspaceRepository, create_workspace};
    use rusqlite::Connection;

    use super::{MigrationFailpoint, migrate_v2_to_v3_with_failpoint, migrate_with_hook};
    use crate::SqliteWorkspaceRepository;

    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "nt-cp05-m003-probes-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn v2(root: &Path) -> Connection {
        create_workspace(&SqliteWorkspaceRepository, root).unwrap();
        let connection = Connection::open(root.join("series.sqlite3")).unwrap();
        connection
            .execute_batch(
                "DROP TABLE cp06_volume_registrations;
             DROP TABLE cp05_resource_refs;
             DROP TABLE cp05_volume_versions;
             DROP TABLE cp05_series_defaults;
             PRAGMA user_version=2;
             PRAGMA wal_checkpoint(TRUNCATE);",
            )
            .unwrap();
        connection
    }

    fn assert_v2(connection: &Connection) {
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        let tables: i64 = connection.query_row(
            "SELECT count(*) FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            [], |row| row.get(0)
        ).unwrap();
        assert_eq!((version, tables), (2, 6));
    }

    #[test]
    fn failures_at_backup_staging_and_source_ddl_leave_complete_v2() {
        for failpoint in [
            MigrationFailpoint::Backup,
            MigrationFailpoint::Staging,
            MigrationFailpoint::SourceDdl,
        ] {
            let temp = Temp::new();
            let root = temp.0.join("series");
            let mut connection = v2(&root);
            let result = migrate_v2_to_v3_with_failpoint(&root, &mut connection, Some(failpoint));
            assert_eq!(result, Err(WorkspaceError::StorageFailure));
            assert_v2(&connection);
            assert!(fs::read_dir(&root).unwrap().any(|entry| {
                let path = entry.unwrap().path();
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".nt-migration-backup-")
                    && path.join("manifest.sha256").is_file()
            }));
        }
    }

    #[test]
    fn volume_write_is_blocked_after_staging_until_the_upgrade_commits() {
        let temp = Temp::new();
        let root = temp.0.join("series");
        let mut connection = v2(&root);
        let series: String = connection
            .query_row("SELECT series_id FROM series", [], |row| row.get(0))
            .unwrap();
        let changed = root.join("series.sqlite3");
        let result = migrate_with_hook(&root, &mut connection, None, || {
            let other = Connection::open(changed).unwrap();
            other
                .busy_timeout(std::time::Duration::from_millis(25))
                .unwrap();
            let error = other
                .execute(
                    "INSERT INTO volumes(series_id, volume_id, display_title, created_at_utc_ms) VALUES (?1, 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'new', 1)",
                    [&series],
                )
                .unwrap_err();
            assert_eq!(
                error.sqlite_error_code(),
                Some(rusqlite::ErrorCode::DatabaseBusy)
            );
            Ok(())
        });
        assert_eq!(result, Ok(()));
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 3);
        let volumes: i64 = connection
            .query_row("SELECT count(*) FROM volumes", [], |row| row.get(0))
            .unwrap();
        assert_eq!(volumes, 0);
    }

    #[test]
    fn process_exit_child() {
        let Some(root) = std::env::var_os("NT_CP05_TEST_CHILD_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        let mut connection = Connection::open(root.join("series.sqlite3")).unwrap();
        super::migrate_v2_to_v3(&root, &mut connection).unwrap();
        panic!("crash failpoint did not terminate");
    }

    #[test]
    fn process_exit_after_m003_ddl_rolls_back_on_reopen() {
        let temp = Temp::new();
        let root = temp.0.join("series");
        drop(v2(&root));
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "migration::migration_v3::tests::process_exit_child",
                "--nocapture",
            ])
            .env("NT_CP05_TEST_CHILD_ROOT", &root)
            .env("NT_CP05_TEST_CRASH_AFTER_M003_DDL", "1")
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(93));
        let connection = Connection::open(root.join("series.sqlite3")).unwrap();
        assert_v2(&connection);
        drop(connection);
        let reopened = SqliteWorkspaceRepository.open(&root).unwrap();
        assert_eq!(reopened.schema_version, 4);
    }
}
