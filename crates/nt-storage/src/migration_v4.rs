//! Record the first generation of volumes registered by the import command.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use nt_application::object_store::ImmutableObjectStore;
use nt_application::workspace::WorkspaceError;
use nt_domain::object::{ObjectHash, ObjectReference};
use nt_domain::workspace::VolumeId;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior};

use super::{create_private_directory, ensure_same_workspace, migration_v3::hash_file};
use crate::import::{Budget, verify_import_integrity};
use crate::workspace::{configure_connection, map_io_error, map_sqlite_error, validate_database};
use crate::{FileImportSourceStore, FileObjectStore, PinnedImportSource};

const M004_SCHEMA: &str = "
CREATE TABLE cp06_volume_registrations (
    series_id TEXT NOT NULL,
    volume_id TEXT NOT NULL,
    generation_id TEXT NOT NULL CHECK (generation_id <> 'g0'),
    PRIMARY KEY (series_id, volume_id),
    FOREIGN KEY (series_id, generation_id, volume_id)
        REFERENCES cp05_volume_versions(series_id, generation_id, volume_id) ON DELETE RESTRICT
);";

pub(crate) fn apply_m004(transaction: &Transaction<'_>) -> Result<(), WorkspaceError> {
    transaction
        .execute_batch(M004_SCHEMA)
        .map_err(map_sqlite_error)?;
    transaction
        .pragma_update(None, "user_version", 4)
        .map_err(map_sqlite_error)
}

pub(crate) fn validate_m004(connection: &Connection) -> Result<(), WorkspaceError> {
    let definition: Option<String> = connection
        .query_row(
            "SELECT sql FROM sqlite_schema WHERE type = 'table' AND name = 'cp06_volume_registrations'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(map_sqlite_error)?;
    if definition.as_deref() != Some(M004_SCHEMA.trim().trim_end_matches(';')) {
        return Err(WorkspaceError::CorruptWorkspace);
    }
    let invalid: bool = connection
        .query_row(
            "SELECT EXISTS (
                 SELECT 1 FROM cp06_volume_registrations r
                 JOIN cp05_volume_versions v USING (series_id, generation_id, volume_id)
                 WHERE v.state <> 'imported'
             )",
            [],
            |row| row.get(0),
        )
        .map_err(map_sqlite_error)?;
    if invalid {
        return Err(WorkspaceError::CorruptWorkspace);
    }
    Ok(())
}

pub(crate) fn migrate_v3_to_v4(
    root: &Path,
    connection: &mut Connection,
) -> Result<(), WorkspaceError> {
    migrate_with_hook(root, connection, || Ok(()), || Ok(()))
}

fn migrate_with_hook(
    root: &Path,
    connection: &mut Connection,
    after_staging: impl FnOnce() -> Result<(), WorkspaceError>,
    after_ddl: impl FnOnce() -> Result<(), WorkspaceError>,
) -> Result<(), WorkspaceError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(map_sqlite_error)?;
    let before = validate_database(&transaction)?;
    if before.schema_version == 4 {
        return Ok(());
    }
    if before.schema_version != 3 {
        return Err(WorkspaceError::UnsupportedFormat);
    }
    let backup_root = create_private_directory(root, "migration-backup")?;
    let backup_path = backup_root.join("series.sqlite3");
    super::create_migration_backup(&transaction, &backup_path)?;
    let backup = open_candidate(&backup_path)?;
    let saved = validate_database(&backup)?;
    ensure_same_workspace(&before, &saved)?;
    if saved.schema_version != 3 {
        return Err(WorkspaceError::Busy);
    }
    let _pins = copy_backup_files(root, &backup_root, &backup)?;
    verify_import_integrity(&backup_root, &backup, &saved.series.id)?;
    drop(backup);

    let stage_root = create_private_directory(root, "migration-stage")?;
    let stage_path = stage_root.join("series.sqlite3");
    let staged = (|| {
        fs::copy(&backup_path, &stage_path).map_err(map_io_error)?;
        if hash_file(&backup_path)? != hash_file(&stage_path)? {
            return Err(WorkspaceError::CorruptWorkspace);
        }
        let mut stage = open_candidate(&stage_path)?;
        let transaction = stage
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(map_sqlite_error)?;
        apply_m004(&transaction)?;
        transaction.commit().map_err(map_sqlite_error)?;
        let validated = validate_database(&stage)?;
        ensure_same_workspace(&before, &validated)?;
        Ok::<_, WorkspaceError>(validated)
    })();
    fs::remove_dir_all(stage_root).map_err(map_io_error)?;
    let staged = staged?;
    after_staging()?;

    apply_m004(&transaction)?;
    after_ddl()?;
    let ready = validate_database(&transaction)?;
    ensure_same_workspace(&staged, &ready)?;
    transaction.commit().map_err(map_sqlite_error)?;
    validate_database(connection).map(|_| ())
}

fn open_candidate(path: &Path) -> Result<Connection, WorkspaceError> {
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(map_sqlite_error)?;
    configure_connection(&connection)?;
    Ok(connection)
}

#[derive(Default)]
struct BackupPins {
    objects: Vec<Box<dyn Read + Send>>,
    images: Vec<PinnedImportSource>,
}

fn copy_backup_files(
    root: &Path,
    backup: &Path,
    database: &Connection,
) -> Result<BackupPins, WorkspaceError> {
    let originals = FileObjectStore::new(root).map_err(|_| WorkspaceError::CorruptWorkspace)?;
    let images = FileImportSourceStore::new(root).map_err(|_| WorkspaceError::CorruptWorkspace)?;
    let copies = FileObjectStore::new(backup).map_err(|_| WorkspaceError::StorageFailure)?;
    let image_copies =
        FileImportSourceStore::new(backup).map_err(|_| WorkspaceError::StorageFailure)?;
    let mut manifest = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(backup.join("manifest.sha256"))
        .map_err(map_io_error)?;
    let (hash, bytes) = hash_file(&backup.join("series.sqlite3"))?;
    writeln!(manifest, "{hash}  {bytes}  series.sqlite3").map_err(map_io_error)?;
    let mut pins = BackupPins::default();
    let mut budget = Budget::default();
    let mut statement = database
        .prepare("SELECT DISTINCT object_hash, byte_len FROM generation_object_refs ORDER BY object_hash, byte_len")
        .map_err(map_sqlite_error)?;
    let mut rows = statement.query([]).map_err(map_sqlite_error)?;
    while let Some(row) = rows.next().map_err(map_sqlite_error)? {
        budget
            .add(512)
            .map_err(|_| WorkspaceError::StorageFailure)?;
        let expected = object_reference(row, 0, 1)?;
        let mut file = originals
            .open_verified(expected.hash)
            .map_err(|_| WorkspaceError::CorruptWorkspace)?;
        if copies
            .put(&mut file)
            .map_err(|_| WorkspaceError::StorageFailure)?
            != expected
        {
            return Err(WorkspaceError::CorruptWorkspace);
        }
        writeln!(
            manifest,
            "{}  {}  objects/{}.blob",
            expected.hash, expected.byte_len, expected.hash
        )
        .map_err(map_io_error)?;
        pins.objects
            .try_reserve_exact(1)
            .map_err(|_| WorkspaceError::StorageFailure)?;
        pins.objects.push(file);
    }

    let mut statement = database
        .prepare(
            "SELECT DISTINCT r.volume_id, v.epub_sha256, r.canonical_path,
                         r.source_relative_path, r.object_hash, r.byte_len
         FROM cp05_resource_refs r
         JOIN cp05_volume_versions v USING (series_id, generation_id, volume_id)
         WHERE r.source_relative_path IS NOT NULL ORDER BY r.source_relative_path",
        )
        .map_err(map_sqlite_error)?;
    let mut rows = statement.query([]).map_err(map_sqlite_error)?;
    while let Some(row) = rows.next().map_err(map_sqlite_error)? {
        let volume = VolumeId::parse(row.get::<_, String>(0).map_err(map_sqlite_error)?)
            .map_err(|_| WorkspaceError::CorruptWorkspace)?;
        let epub: [u8; 32] = row.get(1).map_err(map_sqlite_error)?;
        let path: String = row.get(2).map_err(map_sqlite_error)?;
        let relative: String = row.get(3).map_err(map_sqlite_error)?;
        let reference = object_reference(row, 4, 5)?;
        let source = images
            .open_verified(&volume, epub, &path, reference)
            .map_err(|_| WorkspaceError::CorruptWorkspace)?;
        if source.relative_path != relative {
            return Err(WorkspaceError::CorruptWorkspace);
        }
        let allocation = relative
            .len()
            .checked_mul(8)
            .and_then(|size| size.checked_add(512))
            .ok_or(WorkspaceError::StorageFailure)?;
        budget
            .add(allocation)
            .map_err(|_| WorkspaceError::StorageFailure)?;
        let mut object = copies
            .open_import_object(reference)
            .map_err(|_| WorkspaceError::CorruptWorkspace)?;
        image_copies
            .materialize(&volume, epub, &path, reference, &mut object)
            .map_err(|_| WorkspaceError::StorageFailure)?;
        writeln!(
            manifest,
            "{}  {}  {relative}",
            reference.hash, reference.byte_len
        )
        .map_err(map_io_error)?;
        pins.images
            .try_reserve_exact(1)
            .map_err(|_| WorkspaceError::StorageFailure)?;
        pins.images.push(source);
    }
    manifest.sync_all().map_err(map_io_error)?;
    Ok(pins)
}

fn object_reference(
    row: &rusqlite::Row<'_>,
    hash: usize,
    length: usize,
) -> Result<ObjectReference, WorkspaceError> {
    let hash: String = row.get(hash).map_err(map_sqlite_error)?;
    let length: i64 = row.get(length).map_err(map_sqlite_error)?;
    Ok(ObjectReference {
        hash: ObjectHash::parse(&hash).map_err(|_| WorkspaceError::CorruptWorkspace)?,
        byte_len: u64::try_from(length).map_err(|_| WorkspaceError::CorruptWorkspace)?,
    })
}

#[cfg(test)]
mod recovery_tests {
    use std::cell::Cell;
    use std::path::PathBuf;

    use nt_application::generation::GenerationRepository;
    use nt_application::object_store::ImmutableObjectStore;
    use nt_application::workspace::{WorkspaceRepository, create_workspace};
    use nt_domain::workspace::{GenerationId, GenerationWriteRequest, OperationId};

    use super::{Connection, OpenFlags, WorkspaceError, fs, hash_file, migrate_with_hook};
    use crate::{FileObjectStore, SqliteGenerationWriter, SqliteWorkspaceRepository};

    struct Fixture(PathBuf);

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn m004_rollback_preserves_the_old_snapshot_and_verified_backup_before_retry() {
        let parent =
            std::env::temp_dir().join(format!("nt-cp07-m004-rollback-{}", std::process::id()));
        fs::create_dir(&parent).unwrap();
        let fixture = Fixture(parent);
        let root = fixture.0.join("series");
        let metadata = create_workspace(&SqliteWorkspaceRepository, &root).unwrap();
        assert_eq!(metadata.schema_version, 4);

        let content = b"schema three object fixture";
        let store = FileObjectStore::new(&root).unwrap();
        let object = store.put(&mut &content[..]).unwrap();
        let hash = object.hash;
        let generation = GenerationId::parse("33333333333333333333333333333333").unwrap();
        let writer = SqliteGenerationWriter::open(&root).unwrap();
        writer
            .commit_generation(
                GenerationWriteRequest::new(
                    OperationId::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(),
                    metadata.current_generation,
                    generation.clone(),
                    vec![object],
                )
                .unwrap(),
            )
            .unwrap();
        drop(writer);

        let mut database = Connection::open(root.join("series.sqlite3")).unwrap();
        database
            .execute_batch(
                "BEGIN IMMEDIATE;
             DROP TABLE cp06_volume_registrations;
             PRAGMA user_version = 3;
             COMMIT;",
            )
            .unwrap();
        let staged = Cell::new(false);
        let after_ddl = Cell::new(false);
        assert_eq!(
            migrate_with_hook(
                &root,
                &mut database,
                || {
                    staged.set(true);
                    Ok(())
                },
                || {
                    after_ddl.set(true);
                    Err(WorkspaceError::StorageFailure)
                },
            ),
            Err(WorkspaceError::StorageFailure),
        );
        assert!(staged.get() && after_ddl.get());
        assert_generation_and_schema(&database, &generation, 3);
        let table: i64 = database
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name = 'cp06_volume_registrations'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table, 0);
        drop(database);

        let entries = fs::read_dir(&root)
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        let backups = entries
            .iter()
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".nt-migration-backup-")
            })
            .collect::<Vec<_>>();
        assert_eq!(backups.len(), 1);
        assert!(!entries.iter().any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".nt-migration-stage-")
        }));
        let backup_root = backups[0].path();
        let backup_db = Connection::open_with_flags(
            backup_root.join("series.sqlite3"),
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        assert_generation_and_schema(&backup_db, &generation, 3);
        let backup_store = FileObjectStore::new(&backup_root).unwrap();
        let mut backup_content = Vec::new();
        std::io::Read::read_to_end(
            &mut backup_store.open_verified(hash).unwrap(),
            &mut backup_content,
        )
        .unwrap();
        assert_eq!(backup_content, content);
        drop(backup_db);

        let backup_path = backup_root.join("series.sqlite3");
        let (digest, length) = hash_file(&backup_path).unwrap();
        let manifest = fs::read_to_string(backup_root.join("manifest.sha256")).unwrap();
        assert_eq!(
            manifest,
            format!(
                "{digest}  {length}  series.sqlite3\n{hash}  {}  objects/{hash}.blob\n",
                content.len(),
            )
        );

        for (name, object_bytes) in [
            ("restore-missing-object", None),
            ("restore-wrong-hash", Some(vec![b'x'; content.len()])),
        ] {
            let restore_root = fixture.0.join(name);
            let objects = restore_root.join("objects");
            fs::create_dir_all(&objects).unwrap();
            let restore_path = restore_root.join("series.sqlite3");
            fs::copy(&backup_path, &restore_path).unwrap();
            // The offline backup uses DELETE journaling. A restored workspace uses WAL.
            let restored = Connection::open(&restore_path).unwrap();
            restored.pragma_update(None, "journal_mode", "WAL").unwrap();
            drop(restored);
            if let Some(bytes) = object_bytes {
                fs::write(objects.join(format!("{hash}.blob")), bytes).unwrap();
            }

            assert_eq!(
                SqliteWorkspaceRepository.open(&restore_root),
                Err(WorkspaceError::CorruptWorkspace),
                "{name}",
            );
            let restore_db =
                Connection::open_with_flags(&restore_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                    .unwrap();
            assert_generation_and_schema(&restore_db, &generation, 3);
            assert!(
                !restore_db
                    .prepare(
                        "SELECT name FROM sqlite_schema WHERE name = 'cp06_volume_registrations'"
                    )
                    .unwrap()
                    .exists([])
                    .unwrap()
            );
        }

        let upgraded = SqliteWorkspaceRepository.open(&root).unwrap();
        assert_eq!(upgraded.schema_version, 4);
        assert_eq!(upgraded.current_generation, generation);
        let mut after_retry = Vec::new();
        std::io::Read::read_to_end(&mut store.open_verified(hash).unwrap(), &mut after_retry)
            .unwrap();
        assert_eq!(after_retry, content);
    }

    fn assert_generation_and_schema(
        connection: &Connection,
        generation: &GenerationId,
        schema: i64,
    ) {
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, schema);
        let current: String = connection
            .query_row(
                "SELECT generation_id FROM current_generation WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(current, generation.as_str());
        let integrity: String = connection
            .pragma_query_value(None, "integrity_check", |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");
        assert!(
            !connection
                .prepare("PRAGMA foreign_key_check")
                .unwrap()
                .exists([])
                .unwrap()
        );
        let references: i64 = connection
            .query_row("SELECT count(*) FROM generation_object_refs", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(references, 1);
    }
}
