use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

use nt_application::generation::GenerationRepository;
use nt_application::object_store::ImmutableObjectStore;
use nt_application::workspace::{WorkspaceRepository, create_workspace};
use nt_domain::object::ObjectReference;
use nt_domain::workspace::{
    GenerationCommitError, GenerationId, GenerationWriteRequest, OperationId,
};
use nt_storage::{
    FileObjectStore, SqliteGenerationReader, SqliteGenerationWriter, SqliteWorkspaceRepository,
};
use rusqlite::{Connection, OpenFlags};

const DATABASE_FILE: &str = "series.sqlite3";
static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nt-storage-p02-generation-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create isolated generation directory");
        Self(path)
    }

    fn workspace(&self) -> PathBuf {
        let root = self.0.join("series");
        create_workspace(&SqliteWorkspaceRepository, &root).expect("create schema v2 workspace");
        root
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn request(
    operation_id: &str,
    expected_parent: &str,
    generation_id: &str,
    objects: Vec<ObjectReference>,
) -> GenerationWriteRequest {
    GenerationWriteRequest::new(
        OperationId::parse(operation_id).expect("valid operation ID"),
        GenerationId::parse(expected_parent).expect("valid parent generation"),
        GenerationId::parse(generation_id).expect("valid result generation"),
        objects,
    )
    .expect("valid generation write request")
}

fn open_database(root: &Path) -> Connection {
    Connection::open_with_flags(
        root.join(DATABASE_FILE),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("open workspace database read-only")
}

fn current_generation(connection: &Connection) -> String {
    connection
        .query_row(
            "SELECT generation_id FROM current_generation WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .expect("read current generation")
}

#[test]
fn writer_acknowledges_only_committed_data_and_replay_keeps_original_result() {
    let temp = TestDirectory::new();
    let root = temp.workspace();
    let store = FileObjectStore::new(&root).expect("open immutable object store");
    let first_object = store
        .put(&mut &b"first complete generation"[..])
        .expect("publish first object before DB write");
    let second_object = store
        .put(&mut &b"second complete generation"[..])
        .expect("publish second object before DB write");
    let writer = SqliteGenerationWriter::open(&root).expect("open Series writer");

    let first = request(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "g0",
        "11111111111111111111111111111111",
        vec![first_object],
    );
    let committed = writer
        .commit_generation(first.clone())
        .expect("commit first generation");
    assert_eq!(
        committed.generation_id.as_str(),
        "11111111111111111111111111111111"
    );
    assert!(!committed.replayed);

    let connection = open_database(&root);
    assert_eq!(
        current_generation(&connection),
        committed.generation_id.as_str()
    );
    let ledger_result: String = connection
        .query_row(
            "SELECT result_generation_id FROM generation_operations WHERE operation_id = ?1",
            [first.operation_id().as_str()],
            |row| row.get(0),
        )
        .expect("operation ledger is durable before ACK");
    assert_eq!(ledger_result, committed.generation_id.as_str());
    drop(connection);

    let second = request(
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "11111111111111111111111111111111",
        "22222222222222222222222222222222",
        vec![second_object],
    );
    writer
        .commit_generation(second)
        .expect("advance current generation");

    let replayed = writer
        .commit_generation(first.clone())
        .expect("replay the first operation after current advanced");
    assert_eq!(replayed.generation_id, committed.generation_id);
    assert!(replayed.replayed);

    let connection = open_database(&root);
    assert_eq!(
        current_generation(&connection),
        "22222222222222222222222222222222"
    );
    let generation_count: i64 = connection
        .query_row("SELECT count(*) FROM generations", [], |row| row.get(0))
        .expect("count generation history");
    let operation_count: i64 = connection
        .query_row("SELECT count(*) FROM generation_operations", [], |row| {
            row.get(0)
        })
        .expect("count committed operations");
    assert_eq!(generation_count, 3);
    assert_eq!(operation_count, 2);
}

#[test]
fn operation_conflict_and_stale_parent_leave_history_and_current_unchanged() {
    let temp = TestDirectory::new();
    let root = temp.workspace();
    let store = FileObjectStore::new(&root).expect("open immutable object store");
    let original = store
        .put(&mut &b"original operation"[..])
        .expect("publish original object");
    let changed = store
        .put(&mut &b"changed operation"[..])
        .expect("publish changed object");
    let writer = SqliteGenerationWriter::open(&root).expect("open Series writer");

    let accepted = request(
        "cccccccccccccccccccccccccccccccc",
        "g0",
        "33333333333333333333333333333333",
        vec![original],
    );
    writer
        .commit_generation(accepted.clone())
        .expect("commit accepted operation");

    let conflict = request(
        "cccccccccccccccccccccccccccccccc",
        "g0",
        "44444444444444444444444444444444",
        vec![changed],
    );
    assert_eq!(
        writer.commit_generation(conflict),
        Err(GenerationCommitError::OperationConflict)
    );

    let stale = request(
        "dddddddddddddddddddddddddddddddd",
        "g0",
        "55555555555555555555555555555555",
        vec![changed],
    );
    assert_eq!(
        writer.commit_generation(stale),
        Err(GenerationCommitError::StaleGeneration)
    );

    let connection = open_database(&root);
    assert_eq!(
        current_generation(&connection),
        "33333333333333333333333333333333"
    );
    let generation_count: i64 = connection
        .query_row("SELECT count(*) FROM generations", [], |row| row.get(0))
        .expect("count generations after rejected requests");
    let operation_count: i64 = connection
        .query_row("SELECT count(*) FROM generation_operations", [], |row| {
            row.get(0)
        })
        .expect("count ledger after rejected requests");
    let reference_count: i64 = connection
        .query_row("SELECT count(*) FROM generation_object_refs", [], |row| {
            row.get(0)
        })
        .expect("count references after rejected requests");
    assert_eq!(generation_count, 2);
    assert_eq!(operation_count, 1);
    assert_eq!(reference_count, 1);
}

#[test]
fn concurrent_writers_cas_once_and_reader_keeps_a_stable_snapshot() {
    let temp = TestDirectory::new();
    let root = temp.workspace();
    let store = FileObjectStore::new(&root).expect("open immutable object store");
    let first_object = store
        .put(&mut &b"writer one object"[..])
        .expect("publish first object");
    let second_object = store
        .put(&mut &b"writer two object"[..])
        .expect("publish second object");
    let writer_one = Arc::new(SqliteGenerationWriter::open(&root).expect("open writer one"));
    let writer_two = Arc::new(SqliteGenerationWriter::open(&root).expect("open writer two"));
    let mut reader = SqliteGenerationReader::open(&root).expect("open read-only generation reader");
    let old_snapshot = reader.begin_snapshot().expect("begin old snapshot");
    assert_eq!(old_snapshot.generation_id().as_str(), "g0");
    assert!(old_snapshot.objects().is_empty());

    let barrier = Arc::new(Barrier::new(3));
    let first_barrier = Arc::clone(&barrier);
    let first_writer = Arc::clone(&writer_one);
    let first_thread = thread::spawn(move || {
        let request = request(
            "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
            "g0",
            "66666666666666666666666666666666",
            vec![first_object],
        );
        first_barrier.wait();
        first_writer.commit_generation(request)
    });
    let second_barrier = Arc::clone(&barrier);
    let second_writer = Arc::clone(&writer_two);
    let second_thread = thread::spawn(move || {
        let request = request(
            "ffffffffffffffffffffffffffffffff",
            "g0",
            "77777777777777777777777777777777",
            vec![second_object],
        );
        second_barrier.wait();
        second_writer.commit_generation(request)
    });

    barrier.wait();
    let first_result = first_thread.join().expect("join first writer thread");
    let second_result = second_thread.join().expect("join second writer thread");
    let winner = match (first_result, second_result) {
        (Ok(winner), Err(GenerationCommitError::StaleGeneration | GenerationCommitError::Busy))
        | (Err(GenerationCommitError::StaleGeneration | GenerationCommitError::Busy), Ok(winner)) => {
            winner
        }
        results => panic!("exactly one writer must win the CAS: {results:?}"),
    };
    assert!(!winner.replayed);

    assert_eq!(old_snapshot.generation_id().as_str(), "g0");
    assert!(old_snapshot.objects().is_empty());
    old_snapshot.finish().expect("finish pinned read snapshot");
    let current_snapshot = reader.begin_snapshot().expect("begin new snapshot");
    assert_eq!(current_snapshot.generation_id(), &winner.generation_id);
    assert_eq!(current_snapshot.objects().len(), 1);
    let winning_object = if winner.generation_id.as_str() == "66666666666666666666666666666666" {
        first_object
    } else {
        second_object
    };
    assert_eq!(current_snapshot.objects(), &[winning_object]);

    let connection = open_database(&root);
    assert_eq!(
        current_generation(&connection),
        winner.generation_id.as_str()
    );
    let generation_count: i64 = connection
        .query_row("SELECT count(*) FROM generations", [], |row| row.get(0))
        .expect("count generation history after CAS");
    let operation_count: i64 = connection
        .query_row("SELECT count(*) FROM generation_operations", [], |row| {
            row.get(0)
        })
        .expect("count operation ledger after CAS");
    assert_eq!(generation_count, 2);
    assert_eq!(operation_count, 1);
}

#[test]
fn v1_open_migrates_through_staging_and_preserves_workspace_and_objects() {
    let temp = TestDirectory::new();
    let root = temp.workspace();
    let created = SqliteWorkspaceRepository
        .open(&root)
        .expect("open created workspace");
    let store = FileObjectStore::new(&root).expect("open immutable object store");
    let object = store
        .put(&mut &b"existing object stays byte-for-byte"[..])
        .expect("publish pre-migration object");
    let object_path = root
        .join("objects")
        .join(format!("{}.blob", object.hash.to_hex()));
    let object_bytes = fs::read(&object_path).expect("capture existing object bytes");

    {
        let connection = Connection::open(root.join(DATABASE_FILE)).expect("open v2 fixture");
        connection
            .execute(
                "INSERT INTO volumes(series_id, volume_id, display_title, created_at_utc_ms) VALUES (?1, ?2, ?3, ?4)",
                (
                    created.series.id.as_str(),
                    "88888888888888888888888888888888",
                    Some("Existing Volume"),
                    42_i64,
                ),
            )
            .expect("insert existing Volume metadata");
        connection
            .execute_batch(
                "DROP TABLE generation_operations; DROP TABLE generation_object_refs; PRAGMA user_version = 1;",
            )
            .expect("restore the published v1 schema for migration fixture");
        connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .expect("checkpoint the restored v1 fixture");
    }

    assert_eq!(
        SqliteGenerationReader::open(&root).err(),
        Some(GenerationCommitError::CorruptWorkspace),
        "read-only reader must not migrate schema v1"
    );
    let before_migration = open_database(&root);
    let old_version: i64 = before_migration
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("read v1 version after read-only open");
    assert_eq!(old_version, 1);
    drop(before_migration);

    let opened = SqliteWorkspaceRepository
        .open(&root)
        .expect("migrate and open v1 workspace");
    assert_eq!(opened.series, created.series);
    assert_eq!(opened.current_generation.as_str(), "g0");
    assert_eq!(opened.schema_version, 2);
    assert_eq!(
        fs::read(&object_path).expect("read preserved object"),
        object_bytes
    );

    let connection = open_database(&root);
    let volume: (String, Option<String>, i64) = connection
        .query_row(
            "SELECT volume_id, display_title, created_at_utc_ms FROM volumes",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("read preserved Volume metadata");
    assert_eq!(
        volume,
        (
            "88888888888888888888888888888888".to_owned(),
            Some("Existing Volume".to_owned()),
            42
        )
    );
    let operation_tables: i64 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_schema WHERE type='table' AND name IN ('generation_operations', 'generation_object_refs')",
            [],
            |row| row.get(0),
        )
        .expect("count migrated tables");
    assert_eq!(operation_tables, 2);

    let backup = fs::read_dir(&root)
        .expect("read workspace backup entries")
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".nt-migration-backup-")
        })
        .expect("successful migration retains its source backup");
    let backup_database = Connection::open_with_flags(
        backup.path().join(DATABASE_FILE),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("open retained source backup");
    let backup_version: i64 = backup_database
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("read backup schema version");
    assert_eq!(backup_version, 1);
    let backup_series: String = backup_database
        .query_row(
            "SELECT series_id FROM series WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .expect("read Series from source backup");
    assert_eq!(backup_series, created.series.id.as_str());
}

#[test]
fn sqlite_busy_rejects_generation_without_partial_rows_or_pointer_change() {
    let temp = TestDirectory::new();
    let root = temp.workspace();
    let store = FileObjectStore::new(&root).expect("open immutable object store");
    let object = store
        .put(&mut &b"busy request object"[..])
        .expect("publish immutable object before acquiring the database lock");
    let writer = SqliteGenerationWriter::open(&root).expect("open Series writer");
    let lock = Connection::open(root.join(DATABASE_FILE)).expect("open lock connection");
    lock.execute_batch("BEGIN IMMEDIATE")
        .expect("hold the SQLite write lock");

    assert_eq!(
        writer.commit_generation(request(
            "99999999999999999999999999999999",
            "g0",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            vec![object],
        )),
        Err(GenerationCommitError::Busy)
    );

    assert_eq!(current_generation(&lock), "g0");
    let generation_count: i64 = lock
        .query_row("SELECT count(*) FROM generations", [], |row| row.get(0))
        .expect("count generations while lock is held");
    let operation_count: i64 = lock
        .query_row("SELECT count(*) FROM generation_operations", [], |row| {
            row.get(0)
        })
        .expect("count operation ledger while lock is held");
    let reference_count: i64 = lock
        .query_row("SELECT count(*) FROM generation_object_refs", [], |row| {
            row.get(0)
        })
        .expect("count object refs while lock is held");
    assert_eq!(generation_count, 1);
    assert_eq!(operation_count, 0);
    assert_eq!(reference_count, 0);

    lock.execute_batch("ROLLBACK")
        .expect("release the SQLite write lock");
}
