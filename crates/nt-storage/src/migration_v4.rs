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
    // Compare the version on this same connection inside the final write
    // transaction. Any intervening commit, including a change outside the
    // generation writer, invalidates the backup.
    let data_version = pragma(connection, "data_version")?;
    let before = validate_database(connection)?;
    if before.schema_version == 4 {
        return Ok(());
    }
    if before.schema_version != 3 {
        return Err(WorkspaceError::UnsupportedFormat);
    }
    let backup_root = create_private_directory(root, "migration-backup")?;
    let backup_path = backup_root.join("series.sqlite3");
    connection
        .execute(
            "VACUUM INTO ?1",
            [backup_path
                .to_str()
                .ok_or(WorkspaceError::UnsupportedFilesystem)?],
        )
        .map_err(map_sqlite_error)?;
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

    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(map_sqlite_error)?;
    let version = pragma(&transaction, "user_version")?;
    if version == 4 {
        transaction.commit().map_err(map_sqlite_error)?;
        return validate_database(connection).map(|_| ());
    }
    if version != 3 || pragma(&transaction, "data_version")? != data_version {
        return Err(WorkspaceError::Busy);
    }
    apply_m004(&transaction)?;
    after_ddl()?;
    let ready = validate_database(&transaction)?;
    ensure_same_workspace(&staged, &ready)?;
    transaction.commit().map_err(map_sqlite_error)?;
    validate_database(connection).map(|_| ())
}

fn pragma(connection: &Connection, name: &str) -> Result<i64, WorkspaceError> {
    connection
        .pragma_query_value(None, name, |row| row.get(0))
        .map_err(map_sqlite_error)
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
