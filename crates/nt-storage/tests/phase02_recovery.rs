use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use nt_application::generation::GenerationRepository;
use nt_application::object_store::ImmutableObjectStore;
use nt_application::workspace::{WorkspaceRepository, create_workspace};
use nt_domain::workspace::{GenerationId, GenerationWriteRequest, OperationId};
use nt_storage::{FileObjectStore, SqliteGenerationWriter, SqliteWorkspaceRepository};
use rusqlite::{Connection, OpenFlags};

const OLD: &[u8] = b"verified source still referenced";
const NEW: &[u8] = b"complete but not referenced";
const PARTIAL: &[u8] = b"partial";
const PREVIOUS: &str = "11111111111111111111111111111111";
const OPERATION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CRASH_EXIT: i32 = 74;
static SEQUENCE: AtomicUsize = AtomicUsize::new(0);

struct TestRoot(PathBuf);

impl TestRoot {
    fn new() -> Self {
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nt-phase02-recovery-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create a synthetic recovery fixture");
        Self(path)
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct ExitAfterPartialWrite {
    marker: PathBuf,
    sent: bool,
}

impl Read for ExitAfterPartialWrite {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.sent {
            fs::write(&self.marker, b"partial-object-bytes-written").unwrap();
            std::process::exit(CRASH_EXIT);
        }
        buffer[..PARTIAL.len()].copy_from_slice(PARTIAL);
        self.sent = true;
        Ok(PARTIAL.len())
    }
}

fn reopen_database(root: &Path) -> Connection {
    Connection::open_with_flags(
        root.join("series.sqlite3"),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .unwrap()
}

#[test]
fn process_exit_during_temp_write_preserves_previous_current_and_marks_temp() {
    verify_file_stage("during-file", b"partial-object-bytes-written");
}

#[test]
fn process_exit_after_file_publish_preserves_previous_current_and_lists_orphan() {
    verify_file_stage("after-file", b"object-published-before-database");
}

fn verify_file_stage(stage: &str, expected_marker: &[u8]) {
    let fixture = TestRoot::new();
    let root = fixture.0.join("series");
    create_workspace(&SqliteWorkspaceRepository, &root).unwrap();
    let store = FileObjectStore::new(&root).unwrap();
    let previous_object = store.put(&mut &OLD[..]).unwrap();
    let expected_hash = previous_object.hash;
    let writer = SqliteGenerationWriter::open(&root).unwrap();
    let request = GenerationWriteRequest::new(
        OperationId::parse(OPERATION).unwrap(),
        GenerationId::parse("g0").unwrap(),
        GenerationId::parse(PREVIOUS).unwrap(),
        vec![previous_object],
    )
    .unwrap();
    writer.commit_generation(request).unwrap();
    drop(writer);

    let marker = fixture.0.join("marker");
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "file_crash_child", "--nocapture"])
        .env("NT_PHASE02_FILE_CRASH_STAGE", stage)
        .env("NT_PHASE02_FILE_CRASH_ROOT", &root)
        .env("NT_PHASE02_FILE_CRASH_MARKER", &marker)
        .output()
        .unwrap();
    assert_eq!(
        child.status.code(),
        Some(CRASH_EXIT),
        "{}",
        String::from_utf8_lossy(&child.stderr)
    );
    assert_eq!(fs::read(&marker).unwrap(), expected_marker);

    let reopened = SqliteWorkspaceRepository.open(&root).unwrap();
    assert_eq!(reopened.current_generation.as_str(), PREVIOUS);
    let database = reopen_database(&root);
    let integrity: String = database
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    assert!(
        !database
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .exists([])
            .unwrap()
    );
    let history: i64 = database
        .query_row("SELECT count(*) FROM generations", [], |row| row.get(0))
        .unwrap();
    assert_eq!(history, 2);
    let (operations, result): (i64, String) = database
        .query_row(
            "SELECT count(*), max(result_generation_id) FROM generation_operations",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!((operations, result.as_str()), (1, PREVIOUS));
    let (references, hash): (i64, String) = database
        .query_row(
            "SELECT count(*), max(object_hash) FROM generation_object_refs",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!((references, hash), (1, expected_hash.to_hex()));

    let mut prior = Vec::new();
    store
        .open_verified(expected_hash)
        .unwrap()
        .read_to_end(&mut prior)
        .unwrap();
    assert_eq!(prior, OLD);
    let inventory = store.inventory(&BTreeSet::from([expected_hash])).unwrap();
    assert_eq!(inventory.referenced.len(), 1);
    assert!(inventory.missing_references.is_empty());
    assert!(inventory.invalid_entries.is_empty());
    match stage {
        "during-file" => {
            assert_eq!(inventory.incomplete_temps.len(), 1);
            assert!(inventory.unreferenced.is_empty());
            let temp = root.join("objects").join(&inventory.incomplete_temps[0]);
            assert_eq!(fs::read(temp).unwrap(), PARTIAL);
        }
        "after-file" => {
            assert!(inventory.incomplete_temps.is_empty());
            assert_eq!(inventory.unreferenced.len(), 1);
            let mut object = store.open_verified(inventory.unreferenced[0].hash).unwrap();
            let mut bytes = Vec::new();
            object.read_to_end(&mut bytes).unwrap();
            assert_eq!(bytes, NEW);
        }
        _ => unreachable!("exact fixture stage"),
    }
}

#[test]
fn file_crash_child() {
    let Ok(stage) = std::env::var("NT_PHASE02_FILE_CRASH_STAGE") else {
        return;
    };
    let root = PathBuf::from(std::env::var_os("NT_PHASE02_FILE_CRASH_ROOT").unwrap());
    let marker = PathBuf::from(std::env::var_os("NT_PHASE02_FILE_CRASH_MARKER").unwrap());
    let store = FileObjectStore::new(&root).unwrap();
    match stage.as_str() {
        "during-file" => {
            store
                .put(&mut ExitAfterPartialWrite {
                    marker,
                    sent: false,
                })
                .unwrap();
        }
        "after-file" => {
            let published = store.put(&mut &NEW[..]).unwrap();
            let mut verified = Vec::new();
            store
                .open_verified(published.hash)
                .unwrap()
                .read_to_end(&mut verified)
                .unwrap();
            assert_eq!(verified, NEW);
            fs::write(marker, b"object-published-before-database").unwrap();
            std::process::exit(CRASH_EXIT);
        }
        _ => panic!("unknown test-only crash point"),
    }
    panic!("the child did not hit the requested crash point");
}
