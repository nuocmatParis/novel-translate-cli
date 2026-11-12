use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use nt_application::workspace::WorkspaceError;
use nt_domain::workspace::WorkspaceMetadata;
use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};

use super::workspace::{
    SCHEMA_VERSION, configure_connection, map_io_error, map_sqlite_error, validate_database,
};

static NEXT_MIGRATION_DIRECTORY: AtomicU64 = AtomicU64::new(0);

pub(super) const M002_SCHEMA: &str = r#"
CREATE TABLE generation_operations (
    series_id TEXT NOT NULL,
    operation_id TEXT NOT NULL
        CHECK (length(operation_id) = 32 AND operation_id NOT GLOB '*[^0-9a-f]*'),
    request_fingerprint BLOB NOT NULL CHECK (length(request_fingerprint) = 32),
    expected_parent_generation_id TEXT NOT NULL,
    result_generation_id TEXT NOT NULL,
    committed_at_utc_ms INTEGER NOT NULL CHECK (committed_at_utc_ms >= 0),
    PRIMARY KEY (series_id, operation_id),
    UNIQUE (series_id, result_generation_id),
    FOREIGN KEY (series_id, expected_parent_generation_id)
        REFERENCES generations(series_id, generation_id) ON DELETE RESTRICT,
    FOREIGN KEY (series_id, result_generation_id)
        REFERENCES generations(series_id, generation_id) ON DELETE RESTRICT
);

CREATE TABLE generation_object_refs (
    series_id TEXT NOT NULL,
    generation_id TEXT NOT NULL,
    object_hash TEXT NOT NULL
        CHECK (length(object_hash) = 64 AND object_hash NOT GLOB '*[^0-9a-f]*'),
    byte_len INTEGER NOT NULL CHECK (byte_len >= 0),
    PRIMARY KEY (series_id, generation_id, object_hash),
    FOREIGN KEY (series_id, generation_id)
        REFERENCES generations(series_id, generation_id) ON DELETE RESTRICT
);

CREATE INDEX generation_object_refs_hash_idx
    ON generation_object_refs(object_hash);
"#;

pub(super) fn apply_m002(transaction: &Transaction<'_>) -> Result<(), WorkspaceError> {
    transaction
        .execute_batch(M002_SCHEMA)
        .map_err(map_sqlite_error)?;
    transaction
        .pragma_update(None, "user_version", SCHEMA_VERSION)
        .map_err(map_sqlite_error)
}

pub(super) fn migrate_v1_to_v2(
    root: &Path,
    connection: &mut Connection,
) -> Result<(), WorkspaceError> {
    migrate_v1_to_v2_with_failpoint(root, connection, None)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MigrationFailpoint {
    AfterStaging,
    AfterSourceDdl,
}

fn migrate_v1_to_v2_with_failpoint(
    root: &Path,
    connection: &mut Connection,
    failpoint: Option<MigrationFailpoint>,
) -> Result<(), WorkspaceError> {
    let current_version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(map_sqlite_error)?;
    if current_version == i64::from(SCHEMA_VERSION) {
        return Ok(());
    }
    if current_version != 1 {
        return Err(WorkspaceError::UnsupportedFormat);
    }

    let before = validate_database(connection)?;
    let backup_directory = create_private_directory(root, "migration-backup")?;
    let staging_directory = create_private_directory(root, "migration-stage")?;
    let backup_path = backup_directory.join("series.sqlite3");
    let staging_path = staging_directory.join("series.sqlite3");

    let staged = (|| {
        let backup_name = backup_path
            .to_str()
            .ok_or(WorkspaceError::UnsupportedFilesystem)?;
        connection
            .execute("VACUUM INTO ?1", [backup_name])
            .map_err(map_sqlite_error)?;

        let backup = Connection::open_with_flags(
            &backup_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(map_sqlite_error)?;
        configure_connection(&backup)?;
        let backup_metadata = validate_database(&backup)?;
        if backup_metadata != before {
            return Err(WorkspaceError::CorruptWorkspace);
        }

        fs::copy(&backup_path, &staging_path).map_err(map_io_error)?;
        let mut staging = Connection::open_with_flags(
            &staging_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(map_sqlite_error)?;
        configure_connection(&staging)?;
        let transaction = staging
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(map_sqlite_error)?;
        apply_m002(&transaction)?;
        transaction.commit().map_err(map_sqlite_error)?;

        let staged_metadata = validate_database(&staging)?;
        ensure_same_workspace(&before, &staged_metadata)?;
        Ok(staged_metadata)
    })();
    let _ = fs::remove_dir_all(&staging_directory);
    let staged_metadata = staged?;

    if failpoint == Some(MigrationFailpoint::AfterStaging) {
        return Err(WorkspaceError::StorageFailure);
    }

    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(map_sqlite_error)?;
    let version_inside_transaction: i64 = transaction
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(map_sqlite_error)?;
    if version_inside_transaction == i64::from(SCHEMA_VERSION) {
        transaction.commit().map_err(map_sqlite_error)?;
        return Ok(());
    }
    if version_inside_transaction != 1 {
        return Err(WorkspaceError::UnsupportedFormat);
    }
    apply_m002(&transaction)?;
    if failpoint == Some(MigrationFailpoint::AfterSourceDdl) {
        return Err(WorkspaceError::StorageFailure);
    }
    let migrated_metadata = validate_database(&transaction)?;
    ensure_same_workspace(&staged_metadata, &migrated_metadata)?;
    transaction.commit().map_err(map_sqlite_error)?;

    let final_metadata = validate_database(connection)?;
    ensure_same_workspace(&staged_metadata, &final_metadata)
}

fn create_private_directory(root: &Path, role: &str) -> Result<PathBuf, WorkspaceError> {
    for _ in 0..128 {
        let sequence = NEXT_MIGRATION_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let directory = root.join(format!(".nt-{role}-{}-{sequence}", std::process::id()));
        match fs::create_dir(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(map_io_error(error)),
        }
    }
    Err(WorkspaceError::StorageFailure)
}

fn ensure_same_workspace(
    before: &WorkspaceMetadata,
    after: &WorkspaceMetadata,
) -> Result<(), WorkspaceError> {
    if before.series != after.series || before.current_generation != after.current_generation {
        return Err(WorkspaceError::CorruptWorkspace);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use nt_application::workspace::WorkspaceError;
    use rusqlite::Connection;

    use super::{MigrationFailpoint, migrate_v1_to_v2_with_failpoint};
    use crate::workspace::{APPLICATION_ID, SCHEMA_V1};

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "nt-storage-p02-migration-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create temporary migration directory");
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

    fn create_v1(root: &Path) -> Connection {
        let path = root.join("series.sqlite3");
        let connection = Connection::open(&path).expect("open v1 fixture");
        connection
            .execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
            .expect("configure v1 fixture");
        connection
            .execute_batch(SCHEMA_V1)
            .expect("create v1 tables");
        connection
            .execute(
                "INSERT INTO series(singleton, series_id, created_at_utc_ms) VALUES (1, ?1, 7)",
                ["0123456789abcdef0123456789abcdef"],
            )
            .expect("insert Series");
        connection
            .execute(
                "INSERT INTO generations(series_id, generation_id, parent_generation_id, created_at_utc_ms) VALUES (?1, 'g0', NULL, 7)",
                ["0123456789abcdef0123456789abcdef"],
            )
            .expect("insert bootstrap generation");
        connection
            .execute(
                "INSERT INTO current_generation(singleton, series_id, generation_id) VALUES (1, ?1, 'g0')",
                ["0123456789abcdef0123456789abcdef"],
            )
            .expect("insert current generation");
        connection
            .pragma_update(None, "application_id", APPLICATION_ID)
            .expect("set format marker");
        connection
            .pragma_update(None, "user_version", 1)
            .expect("set schema version");
        connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .expect("checkpoint v1 fixture");
        connection
    }

    fn user_version(connection: &Connection) -> i64 {
        connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read schema version")
    }

    #[test]
    fn migration_failure_after_staging_preserves_v1_and_existing_files() {
        let temp = TestDirectory::new();
        let mut connection = create_v1(temp.path());
        let objects = temp.path().join("objects");
        fs::create_dir(&objects).expect("create object fixture directory");
        let object = objects.join("sentinel.blob");
        fs::write(&object, b"preserve this existing object").expect("write object fixture");

        let error = migrate_v1_to_v2_with_failpoint(
            temp.path(),
            &mut connection,
            Some(MigrationFailpoint::AfterStaging),
        )
        .expect_err("injected staging failure must stop migration");
        assert_eq!(error, WorkspaceError::StorageFailure);
        assert_eq!(user_version(&connection), 1);
        assert_eq!(
            fs::read(&object).expect("read object after failed migration"),
            b"preserve this existing object"
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT count(*) FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .expect("count original tables"),
            4
        );
        assert!(fs::read_dir(temp.path()).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".nt-migration-backup-")
        }));
    }

    #[test]
    fn migration_failure_inside_source_transaction_rolls_back_all_ddl() {
        let temp = TestDirectory::new();
        let mut connection = create_v1(temp.path());

        let error = migrate_v1_to_v2_with_failpoint(
            temp.path(),
            &mut connection,
            Some(MigrationFailpoint::AfterSourceDdl),
        )
        .expect_err("injected DDL failure must stop migration");
        assert_eq!(error, WorkspaceError::StorageFailure);
        assert_eq!(user_version(&connection), 1);
        let operation_table_exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='generation_operations')",
                [],
                |row| row.get(0),
            )
            .expect("check migration rollback");
        assert!(!operation_table_exists);
    }
}
