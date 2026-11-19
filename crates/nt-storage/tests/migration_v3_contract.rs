use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};

use nt_application::generation::GenerationRepository;
use nt_application::object_store::ImmutableObjectStore;
use nt_application::workspace::{WorkspaceError, WorkspaceRepository, create_workspace};
use nt_domain::object::{ObjectHash, ObjectReference};
use nt_domain::workspace::{GenerationId, GenerationWriteRequest, OperationId};
use nt_storage::{
    FileObjectStore, SqliteGenerationReader, SqliteGenerationWriter, SqliteWorkspaceRepository,
};
use rusqlite::Connection;
use sha2::{Digest, Sha256};

static NEXT: AtomicUsize = AtomicUsize::new(0);
const VOLUME: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const GENERATION: &str = "11111111111111111111111111111111";

struct TestDir(PathBuf);
impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "nt-cp05-migration-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn workspace(&self) -> PathBuf {
        let root = self.0.join("series");
        create_workspace(&SqliteWorkspaceRepository, &root).unwrap();
        root
    }
}
impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn prepare_v2(root: &Path) -> (String, ObjectReference, Vec<u8>) {
    let metadata = SqliteWorkspaceRepository.open(root).unwrap();
    let bytes = b"preexisting referenced source bytes".to_vec();
    let store = FileObjectStore::new(root).unwrap();
    let object = store.put(&mut bytes.as_slice()).unwrap();
    let writer = SqliteGenerationWriter::open(root).unwrap();
    let request = GenerationWriteRequest::new(
        OperationId::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap(),
        GenerationId::parse("g0").unwrap(),
        GenerationId::parse(GENERATION).unwrap(),
        vec![object],
    )
    .unwrap();
    writer.commit_generation(request).unwrap();
    drop(writer);

    let connection = Connection::open(root.join("series.sqlite3")).unwrap();
    connection.execute(
        "INSERT INTO volumes(series_id, volume_id, display_title, created_at_utc_ms) VALUES (?1, ?2, 'kept', 12)",
        (metadata.series.id.as_str(), VOLUME),
    ).unwrap();
    connection
        .execute_batch(
            "DROP TABLE cp06_volume_registrations;
         DROP TABLE cp05_resource_refs;
         DROP TABLE cp05_volume_versions;
         DROP TABLE cp05_series_defaults;
         PRAGMA user_version = 2;
         PRAGMA wal_checkpoint(TRUNCATE);",
        )
        .unwrap();
    let tables: i64 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(tables, 6);
    drop(connection);
    (metadata.series.id.as_str().to_owned(), object, bytes)
}

fn schema_version(root: &Path) -> i64 {
    let connection = Connection::open(root.join("series.sqlite3")).unwrap();
    connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap()
}

#[test]
fn public_v2_reader_is_read_only_and_preserves_exact_generation_objects() {
    let temp = TestDir::new();
    let root = temp.workspace();
    let (_, object, _) = prepare_v2(&root);
    let before = fs::read(root.join("series.sqlite3")).unwrap();

    let mut reader = SqliteGenerationReader::open(&root).unwrap();
    let snapshot = reader.begin_snapshot().unwrap();
    assert_eq!(snapshot.generation_id().as_str(), GENERATION);
    assert_eq!(snapshot.objects(), &[object]);
    snapshot.finish().unwrap();
    drop(reader);

    assert_eq!(schema_version(&root), 2);
    assert_eq!(fs::read(root.join("series.sqlite3")).unwrap(), before);
    assert!(!fs::read_dir(&root).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".nt-migration-backup-")
    }));
}

#[test]
fn v2_migration_backfills_actual_current_generation_and_preserves_verified_backup() {
    let temp = TestDir::new();
    let root = temp.workspace();
    let (series, object, bytes) = prepare_v2(&root);
    let opened = SqliteWorkspaceRepository.open(&root).unwrap();
    assert_eq!(opened.schema_version, 4);
    assert_eq!(opened.current_generation.as_str(), GENERATION);

    let db = Connection::open(root.join("series.sqlite3")).unwrap();
    let row: (String, String, String, Option<Vec<u8>>) = db.query_row(
        "SELECT generation_id, state, mode, payload FROM cp05_volume_versions WHERE series_id=?1 AND volume_id=?2",
        (&series, VOLUME), |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    ).unwrap();
    assert_eq!(
        row,
        (
            GENERATION.to_owned(),
            "unimported".to_owned(),
            "all-text".to_owned(),
            None
        )
    );
    let default: (String, String) = db
        .query_row(
            "SELECT generation_id, mode FROM cp05_series_defaults WHERE series_id=?1",
            [&series],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(default, (GENERATION.to_owned(), "all-text".to_owned()));
    let old_g0: i64 = db
        .query_row(
            "SELECT count(*) FROM cp05_volume_versions WHERE generation_id='g0'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(old_g0, 0);
    drop(db);

    let backup = fs::read_dir(&root)
        .unwrap()
        .map(Result::unwrap)
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".nt-migration-backup-")
                && schema_version(path) == 2
        })
        .unwrap();
    let backed_up = fs::read(backup.join("objects").join(format!("{}.blob", object.hash))).unwrap();
    assert_eq!(backed_up, bytes);
    let db_backup = Connection::open(backup.join("series.sqlite3")).unwrap();
    let backup_version: i64 = db_backup
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(backup_version, 2);
    let backup_current: String = db_backup
        .query_row("SELECT generation_id FROM current_generation", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(backup_current, GENERATION);
    drop(db_backup);

    let backup_db_bytes = fs::read(backup.join("series.sqlite3")).unwrap();
    let backup_digest = ObjectHash::from_bytes(Sha256::digest(&backup_db_bytes).into()).to_string();
    let manifest = fs::read_to_string(backup.join("manifest.sha256")).unwrap();
    assert!(manifest.contains(&format!(
        "{backup_digest}  {}  series.sqlite3",
        backup_db_bytes.len()
    )));
    assert!(manifest.contains(&format!(
        "{}  {}  objects/{}.blob",
        object.hash,
        bytes.len(),
        object.hash
    )));
}

#[test]
fn missing_v2_referenced_object_blocks_migration_without_source_ddl() {
    let temp = TestDir::new();
    let root = temp.workspace();
    let (_, object, _) = prepare_v2(&root);
    fs::remove_file(root.join("objects").join(format!("{}.blob", object.hash))).unwrap();
    assert_eq!(
        SqliteWorkspaceRepository.open(&root),
        Err(WorkspaceError::CorruptWorkspace)
    );
    let db = Connection::open(root.join("series.sqlite3")).unwrap();
    let version: i64 = db
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    let tables: i64 = db
        .query_row(
            "SELECT count(*) FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let refs: i64 = db
        .query_row("SELECT count(*) FROM generation_object_refs", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!((version, tables, refs), (2, 6, 1));
}

#[test]
fn m003_enforces_state_and_same_generation_reference_constraints() {
    let temp = TestDir::new();
    let root = temp.workspace();
    let metadata = SqliteWorkspaceRepository.open(&root).unwrap();
    let db = Connection::open(root.join("series.sqlite3")).unwrap();
    db.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    db.execute(
        "INSERT INTO volumes VALUES (?1, ?2, NULL, 1)",
        (metadata.series.id.as_str(), VOLUME),
    )
    .unwrap();

    assert!(db.execute(
        "INSERT INTO cp05_volume_versions(series_id,generation_id,volume_id,state,mode) VALUES (?1,'g0',?2,'imported','story')",
        (metadata.series.id.as_str(), VOLUME)
    ).is_err());
    assert!(db.execute(
        "INSERT INTO cp05_volume_versions(series_id,generation_id,volume_id,state,mode) VALUES (?1,'g0',?2,'unimported','wrong')",
        (metadata.series.id.as_str(), VOLUME)
    ).is_err());
    assert!(db.execute(
        "INSERT INTO cp05_volume_versions(series_id,generation_id,volume_id,state,mode) VALUES (?1,'g0','cccccccccccccccccccccccccccccccc','unimported','all-text')",
        [metadata.series.id.as_str()]
    ).is_err());
    db.execute(
        "INSERT INTO cp05_volume_versions(series_id,generation_id,volume_id,state,mode) VALUES (?1,'g0',?2,'unimported','all-text')",
        (metadata.series.id.as_str(), VOLUME)
    ).unwrap();
    assert!(db.execute(
        "INSERT INTO cp05_resource_refs(series_id,generation_id,volume_id,role,canonical_path,object_hash,byte_len) VALUES (?1,'g0',?2,'image','OPS/x.png',?3,1)",
        (metadata.series.id.as_str(), VOLUME, "0".repeat(64))
    ).is_err());
}

#[test]
fn concurrent_v2_openers_leave_a_single_complete_schema() {
    let temp = TestDir::new();
    let root = temp.workspace();
    let _ = prepare_v2(&root);
    let barrier = Arc::new(Barrier::new(3));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let root = root.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                SqliteWorkspaceRepository.open(&root)
            })
        })
        .collect();
    barrier.wait();
    let outcomes: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert!(outcomes.iter().any(Result::is_ok), "{outcomes:?}");
    assert!(
        outcomes
            .iter()
            .all(|outcome| matches!(outcome, Ok(_) | Err(WorkspaceError::Busy))),
        "{outcomes:?}"
    );
    let final_open = SqliteWorkspaceRepository.open(&root).unwrap();
    assert_eq!(final_open.schema_version, 4);
    assert_eq!(final_open.current_generation.as_str(), GENERATION);
    let db = Connection::open(root.join("series.sqlite3")).unwrap();
    let tables: i64 = db
        .query_row(
            "SELECT count(*) FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(tables, 10);
}
