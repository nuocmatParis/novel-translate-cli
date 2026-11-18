use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use nt_application::generation::GenerationRepository;
use nt_application::object_store::ImmutableObjectStore;
use nt_application::volume_import::{ImportWriteError, VolumeImportRepository};
use nt_application::workspace::{WorkspaceRepository, create_workspace};
use nt_domain::source_scope::ScopeMode;
use nt_domain::volume_import::{ImportOperation, ImportedVolumeState, VolumeImportWriteRequest};
use nt_domain::workspace::{
    GenerationCommitError, GenerationId, GenerationWriteRequest, OperationId, SeriesId, VolumeId,
};
use rusqlite::Connection;

use super::{CommitStage, commit_with_hook, fingerprint_import};
use crate::generation::writer::{SqliteGenerationWriter, WriterMessage};
use crate::{
    FileObjectStore, SqliteVolumeImportReader, SqliteWorkspaceRepository, stage_import_payload,
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
const FIRST: &str = "11111111111111111111111111111111";
const SECOND: &str = "22222222222222222222222222222222";
const VOLUME: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ORIGINAL: &[u8] = b"captured storage-only source";
const PAYLOAD: &[u8] = b"opaque bounded payload";
struct Fixture {
    base: PathBuf,
    root: PathBuf,
    series: SeriesId,
}
impl Fixture {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!(
            "nt-cp05-import-fault-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&base).unwrap();
        let root = base.join("series");
        let metadata = create_workspace(&SqliteWorkspaceRepository, &root).unwrap();
        let connection = Connection::open(root.join("series.sqlite3")).unwrap();
        connection.execute(
            "INSERT INTO volumes(series_id,volume_id,display_title,created_at_utc_ms) VALUES (?1,?2,'fixture',1)",
            (metadata.series.id.as_str(),VOLUME)).unwrap();
        drop(connection);
        Self {
            base,
            root,
            series: metadata.series.id,
        }
    }
    fn request(&self) -> VolumeImportWriteRequest {
        request(&self.root, &self.series)
    }
    fn assert_unpublished(&self) {
        let connection = Connection::open(self.root.join("series.sqlite3")).unwrap();
        assert_eq!(
            connection
                .query_row("SELECT generation_id FROM current_generation", [], |r| {
                    r.get::<_, String>(0)
                })
                .unwrap(),
            "g0"
        );
        for (table, expected) in [
            ("generations", 1),
            ("generation_operations", 0),
            ("cp05_volume_versions", 0),
        ] {
            assert_eq!(
                connection
                    .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                        .get::<_, i64>(0))
                    .unwrap(),
                expected
            );
        }
        assert_eq!(
            connection
                .pragma_query_value::<String, _>(None, "integrity_check", |r| r.get(0))
                .unwrap(),
            "ok"
        );
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}
fn request(root: &Path, series: &SeriesId) -> VolumeImportWriteRequest {
    let store = FileObjectStore::new(root).unwrap();
    let original = store.put(&mut &ORIGINAL[..]).unwrap();
    let payload = stage_import_payload(&store, &mut &PAYLOAD[..]).unwrap();
    VolumeImportWriteRequest::new(
        series.clone(),
        OperationId::parse(FIRST).unwrap(),
        GenerationId::parse("g0").unwrap(),
        GenerationId::parse(FIRST).unwrap(),
        ImportOperation::Publish {
            volume_id: VolumeId::parse(VOLUME).unwrap(),
            state: ImportedVolumeState {
                mode: ScopeMode::AllText,
                epub_sha256: *original.hash.as_bytes(),
                mapping_version: [0x44; 32],
                payload,
            },
            resources: vec![],
            default_delta: Some(ScopeMode::Story),
        },
        vec![original, payload.object],
    )
    .unwrap()
}

#[test]
fn failure_after_each_publication_stage_rolls_back_rows_current_and_ledger() {
    for stage in [
        CommitStage::AfterReferences,
        CommitStage::AfterState,
        CommitStage::BeforeCommit,
    ] {
        let fixture = Fixture::new();
        let request = fixture.request();
        let refs = request.objects().to_vec();
        let mut connection = Connection::open(fixture.root.join("series.sqlite3")).unwrap();
        let store = FileObjectStore::new(&fixture.root).unwrap();
        let mut reached = false;
        let result = commit_with_hook(
            &mut connection,
            &store,
            &fixture.series,
            request.clone(),
            fingerprint_import(&request),
            |at| {
                if at == stage {
                    reached = true;
                    Err(GenerationCommitError::StorageFailure.into())
                } else {
                    Ok(())
                }
            },
        );
        assert_eq!(
            result.unwrap_err(),
            ImportWriteError::from(GenerationCommitError::StorageFailure)
        );
        assert!(reached);
        fixture.assert_unpublished();
        // Complete files may remain unreferenced; failure must not delete them.
        for object in refs {
            let mut bytes = Vec::new();
            store
                .open_verified(object.hash)
                .unwrap()
                .read_to_end(&mut bytes)
                .unwrap();
            assert_eq!(bytes.len() as u64, object.byte_len);
        }
    }
}

#[test]
fn file_verification_finishes_before_write_lock_and_cas_rechecks_a_racing_generation() {
    for race in [false, true] {
        let fixture = Fixture::new();
        let request = fixture.request();
        let store = FileObjectStore::new(&fixture.root).unwrap();
        let mut connection = Connection::open(fixture.root.join("series.sqlite3")).unwrap();
        let mut reached = false;
        let result = commit_with_hook(
            &mut connection,
            &store,
            &fixture.series,
            request.clone(),
            fingerprint_import(&request),
            |at| {
                if at == CommitStage::BeforeTransaction {
                    reached = true;
                    // This would be Busy if the file/hash preparation still
                    // held a BEGIN IMMEDIATE transaction.
                    let probe = Connection::open(fixture.root.join("series.sqlite3")).unwrap();
                    probe.execute_batch("BEGIN IMMEDIATE; ROLLBACK").unwrap();
                    if race {
                        let writer = SqliteGenerationWriter::open(&fixture.root).unwrap();
                        writer
                            .commit_generation(
                                GenerationWriteRequest::new(
                                    OperationId::parse(SECOND).unwrap(),
                                    GenerationId::parse("g0").unwrap(),
                                    GenerationId::parse(SECOND).unwrap(),
                                    vec![],
                                )
                                .unwrap(),
                            )
                            .unwrap();
                    }
                }
                Ok(())
            },
        );
        assert!(reached);
        if race {
            assert_eq!(
                result.unwrap_err(),
                ImportWriteError::from(GenerationCommitError::StaleGeneration)
            );
            let db = fixture.root.join("series.sqlite3");
            let check = Connection::open(db).unwrap();
            assert_eq!(
                check
                    .query_row("SELECT generation_id FROM current_generation", [], |r| {
                        r.get::<_, String>(0)
                    })
                    .unwrap(),
                SECOND
            );
            assert_eq!(
                check
                    .query_row("SELECT count(*) FROM cp05_volume_versions", [], |r| r
                        .get::<_, i64>(0))
                    .unwrap(),
                0
            );
        } else {
            assert!(!result.unwrap().replayed);
        }
    }
}

#[test]
fn dropped_import_ack_is_reconciled_after_next_generic_write_through_the_same_queue() {
    let fixture = Fixture::new();
    let request = fixture.request();
    let writer = SqliteGenerationWriter::open(&fixture.root).unwrap();
    let (reply, receiver) = std::sync::mpsc::sync_channel(1);
    drop(receiver);
    assert!(
        writer
            .inner
            .sender
            .send(WriterMessage::CommitImport {
                fingerprint: fingerprint_import(&request),
                request: request.clone(),
                reply,
            })
            .is_ok()
    );
    let later = GenerationWriteRequest::new(
        OperationId::parse(SECOND).unwrap(),
        GenerationId::parse(FIRST).unwrap(),
        GenerationId::parse(SECOND).unwrap(),
        vec![],
    )
    .unwrap();
    writer.commit_generation(later).unwrap();
    let result = writer.reconcile_volume_import(&request).unwrap();
    assert!(result.replayed);
    assert_eq!(result.generation_id.as_str(), FIRST);
    let mut reader = SqliteVolumeImportReader::open(&fixture.root).unwrap();
    let snapshot = reader.begin_snapshot().unwrap();
    assert_eq!(snapshot.generation_id().as_str(), SECOND);
    snapshot.finish().unwrap();
}

#[test]
fn process_exit_inside_import_before_commit_leaves_previous_state_usable() {
    let fixture = Fixture::new();
    fixture.request();
    let marker = fixture.base.join("reached-before-commit");
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "generation::writer::import_writer::fault_tests::import_process_exit_child",
            "--nocapture",
        ])
        .env("NT_CP05_IMPORT_CRASH_ROOT", &fixture.root)
        .env("NT_CP05_IMPORT_CRASH_SERIES", fixture.series.as_str())
        .env("NT_CP05_IMPORT_CRASH_MARKER", &marker)
        .output()
        .unwrap();
    assert_eq!(
        child.status.code(),
        Some(42),
        "{:?}",
        String::from_utf8_lossy(&child.stderr)
    );
    assert_eq!(
        fs::read(marker).unwrap(),
        b"rows-and-cas-written-not-committed"
    );
    fixture.assert_unpublished();
    assert_eq!(
        SqliteWorkspaceRepository
            .open(&fixture.root)
            .unwrap()
            .schema_version,
        3
    );
}

#[test]
fn import_process_exit_child() {
    let Some(root) = std::env::var_os("NT_CP05_IMPORT_CRASH_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    let series = SeriesId::parse(std::env::var("NT_CP05_IMPORT_CRASH_SERIES").unwrap()).unwrap();
    let request = request(&root, &series);
    let store = FileObjectStore::new(&root).unwrap();
    let mut connection = Connection::open(root.join("series.sqlite3")).unwrap();
    commit_with_hook(
        &mut connection,
        &store,
        &series,
        request.clone(),
        fingerprint_import(&request),
        |at| {
            if at == CommitStage::BeforeCommit {
                fs::write(
                    std::env::var_os("NT_CP05_IMPORT_CRASH_MARKER").unwrap(),
                    b"rows-and-cas-written-not-committed",
                )
                .unwrap();
                std::process::exit(42);
            }
            Ok(())
        },
    )
    .unwrap();
    panic!("child did not reach the controlled crash");
}

use std::io::Read;
