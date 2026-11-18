//! Real SQLite and filesystem tests for the typed CP05 writer.
//! Payloads here are opaque storage fixtures, not a claim of EPUB validity.
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};

use nt_application::generation::GenerationRepository;
use nt_application::object_store::ImmutableObjectStore;
use nt_application::volume_import::{
    GenerationCommitError, GenerationId, ImportOperation, ImportWriteError, ImportedVolumeState,
    ObjectReference, OperationId, SeriesId, VolumeId, VolumeImportRepository,
    VolumeImportWriteRequest, VolumeResource, VolumeState,
};
use nt_application::workspace::{WorkspaceError, WorkspaceRepository, create_workspace};
use nt_domain::source_scope::ScopeMode;
use nt_domain::workspace::GenerationWriteRequest;
use nt_storage::{
    FileImportSourceStore, FileObjectStore, ImportDigestSink, SqliteGenerationReader,
    SqliteGenerationWriter, SqliteVolumeImportReader, SqliteWorkspaceRepository,
    stage_import_payload,
};
use rusqlite::Connection;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Workspace {
    base: PathBuf,
    root: PathBuf,
    series: SeriesId,
    volume: VolumeId,
    second: VolumeId,
}
impl Workspace {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!(
            "nt-cp05-import-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&base).unwrap();
        let root = base.join("series");
        let metadata = create_workspace(&SqliteWorkspaceRepository, &root).unwrap();
        let volume = VolumeId::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
        let second = VolumeId::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
        let connection = Connection::open(root.join("series.sqlite3")).unwrap();
        connection.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        for volume in [&volume, &second] {
            // Volume registration is a T12 use case. This explicitly seeds only
            // the synthetic storage fixture; it is not an app registration path.
            connection.execute(
                "INSERT INTO volumes(series_id,volume_id,display_title,created_at_utc_ms) VALUES (?1,?2,'fixture',1)",
                (metadata.series.id.as_str(), volume.as_str()),
            ).unwrap();
        }
        drop(connection);
        Self {
            base,
            root,
            series: metadata.series.id,
            volume,
            second,
        }
    }
    fn store(&self) -> FileObjectStore {
        FileObjectStore::new(&self.root).unwrap()
    }
    fn writer(&self) -> SqliteGenerationWriter {
        SqliteGenerationWriter::open(&self.root).unwrap()
    }
    fn connection(&self) -> Connection {
        Connection::open(self.root.join("series.sqlite3")).unwrap()
    }
    fn current(&self) -> String {
        self.connection()
            .query_row("SELECT generation_id FROM current_generation", [], |r| {
                r.get(0)
            })
            .unwrap()
    }
    fn operation(
        &self,
        ordinal: u128,
        parent: &str,
        operation: ImportOperation,
        objects: Vec<ObjectReference>,
    ) -> VolumeImportWriteRequest {
        VolumeImportWriteRequest::new(
            self.series.clone(),
            op(ordinal),
            generation(parent),
            generation(&format!("{ordinal:032x}")),
            operation,
            objects,
        )
        .unwrap()
    }
    fn prepared(
        &self,
        ordinal: u128,
        parent: &str,
        volume: &VolumeId,
        bytes: &[u8],
    ) -> (
        VolumeImportWriteRequest,
        ImportedVolumeState,
        Vec<VolumeResource>,
    ) {
        let store = self.store();
        let original = store
            .put(&mut &b"same captured opaque original"[..])
            .unwrap();
        let image = store.put(&mut &b"opaque image source bytes"[..]).unwrap();
        let payload = stage_import_payload(&store, &mut &bytes[..]).unwrap();
        let materializer = FileImportSourceStore::new(&self.root).unwrap();
        let pin = materializer
            .materialize(
                volume,
                *original.hash.as_bytes(),
                "OPS/Images/a.png",
                image,
                &mut &b"opaque image source bytes"[..],
            )
            .unwrap();
        let state = ImportedVolumeState {
            mode: ScopeMode::AllText,
            epub_sha256: *original.hash.as_bytes(),
            mapping_version: [0x77; 32],
            payload,
        };
        let resources = vec![VolumeResource {
            role: "Image".into(),
            canonical_path: "OPS/Images/a.png".into(),
            object: image,
            source_relative_path: Some(pin.relative_path.clone()),
        }];
        let operation = ImportOperation::Publish {
            volume_id: volume.clone(),
            state: state.clone(),
            resources: resources.clone(),
            default_delta: None,
        };
        let request = self.operation(
            ordinal,
            parent,
            operation,
            vec![original, image, payload.object],
        );
        (request, state, resources)
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}
fn op(value: u128) -> OperationId {
    OperationId::parse(format!("{value:032x}")).unwrap()
}
fn generation(value: &str) -> GenerationId {
    GenerationId::parse(value).unwrap()
}
fn generation_error(error: GenerationCommitError) -> ImportWriteError {
    error.into()
}

#[test]
fn import_reopens_complete_state_blob_refs_and_read_snapshot_after_default_changes() {
    let workspace = Workspace::new();
    let (request, state, resources) =
        workspace.prepared(1, "g0", &workspace.volume, b"first payload");
    let writer = workspace.writer();
    let committed = writer.commit_volume_import(request).unwrap();
    assert!(!committed.replayed);
    let mut reader = SqliteVolumeImportReader::open(&workspace.root).unwrap();
    let snapshot = reader.begin_snapshot().unwrap();
    assert_eq!(snapshot.generation_id(), &committed.generation_id);
    let volume = snapshot.volume(&workspace.volume).unwrap();
    assert_eq!(volume.state, VolumeState::Imported(state.clone()));
    assert_eq!(volume.resources, resources);
    assert_eq!(
        snapshot.volume(&workspace.second).unwrap().state,
        VolumeState::Unimported {
            mode: ScopeMode::AllText
        }
    );
    let mut payload = snapshot.open_payload(&workspace.volume).unwrap().unwrap();
    assert_eq!(payload.object(), state.payload.object);
    let mut bytes = Vec::new();
    payload.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"first payload");
    payload.seek(SeekFrom::Start(6)).unwrap();
    bytes.clear();
    payload.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"payload");
    drop(payload);

    let changed = workspace.operation(
        2,
        committed.generation_id.as_str(),
        ImportOperation::SetSeriesDefault {
            mode: ScopeMode::Story,
        },
        vec![],
    );
    writer.commit_volume_import(changed).unwrap();
    assert_eq!(snapshot.series_default().unwrap(), ScopeMode::AllText);
    snapshot.finish().unwrap();
    let latest = reader.begin_snapshot().unwrap();
    assert_eq!(latest.series_default().unwrap(), ScopeMode::Story);
    assert_eq!(
        latest.volume(&workspace.volume).unwrap().state,
        VolumeState::Imported(state)
    );
    // Existing volumes retain their captured scope; default is not re-inherited.
    assert_eq!(
        latest.volume(&workspace.second).unwrap().state,
        VolumeState::Unimported {
            mode: ScopeMode::AllText
        }
    );
    latest.finish().unwrap();
    drop(reader);
    drop(writer);
    assert_eq!(
        SqliteWorkspaceRepository
            .open(&workspace.root)
            .unwrap()
            .schema_version,
        3
    );
}

#[test]
fn replay_and_reconcile_return_historical_generation_after_generic_and_typed_writes() {
    let workspace = Workspace::new();
    let (request, state, _) = workspace.prepared(1, "g0", &workspace.volume, b"first");
    let writer = workspace.writer();
    let first = writer.commit_volume_import(request.clone()).unwrap();
    let generic = GenerationWriteRequest::new(
        op(20),
        first.generation_id.clone(),
        generation("00000000000000000000000000000002"),
        vec![],
    )
    .unwrap();
    writer.commit_generation(generic).unwrap();
    let next = workspace.operation(
        3,
        "00000000000000000000000000000002",
        ImportOperation::SetSeriesDefault {
            mode: ScopeMode::Story,
        },
        vec![],
    );
    writer.commit_volume_import(next).unwrap();
    for replay in [
        writer.commit_volume_import(request.clone()).unwrap(),
        writer.reconcile_volume_import(&request).unwrap(),
    ] {
        assert_eq!(replay.generation_id, first.generation_id);
        assert!(replay.replayed);
    }
    assert_eq!(workspace.current(), "00000000000000000000000000000003");
    let mut reader = SqliteVolumeImportReader::open(&workspace.root).unwrap();
    let history = reader
        .begin_generation_snapshot(&first.generation_id)
        .unwrap();
    assert_eq!(history.series_default().unwrap(), ScopeMode::AllText);
    assert_eq!(
        history.volume(&workspace.volume).unwrap().state,
        VolumeState::Imported(state)
    );
    history.finish().unwrap();
    let latest = reader.begin_snapshot().unwrap();
    assert_eq!(latest.series_default().unwrap(), ScopeMode::Story);
    latest.finish().unwrap();

    let mut generic_reader = SqliteGenerationReader::open(&workspace.root).unwrap();
    let generic_view = generic_reader.begin_snapshot().unwrap();
    // The existing reader lists only exact current-generation refs.
    assert!(generic_view.objects().is_empty());
    generic_view.finish().unwrap();
}

#[test]
fn same_operation_changed_payload_mode_or_kind_conflicts_in_commit_and_reconciliation() {
    let workspace = Workspace::new();
    let (request, mut state, resources) = workspace.prepared(1, "g0", &workspace.volume, b"first");
    let writer = workspace.writer();
    writer.commit_volume_import(request.clone()).unwrap();
    state.mode = ScopeMode::Story;
    let changed_mode = workspace.operation(
        1,
        "g0",
        ImportOperation::Publish {
            volume_id: workspace.volume.clone(),
            state,
            resources,
            default_delta: None,
        },
        request.objects().to_vec(),
    );
    let (changed_payload, _, _) = workspace.prepared(1, "g0", &workspace.volume, b"other");
    let changed_kind = workspace.operation(
        1,
        "g0",
        ImportOperation::SetSeriesDefault {
            mode: ScopeMode::AllText,
        },
        request.objects().to_vec(),
    );
    for changed in [changed_mode, changed_payload, changed_kind] {
        assert_eq!(
            writer.commit_volume_import(changed.clone()).unwrap_err(),
            generation_error(GenerationCommitError::OperationConflict)
        );
        assert_eq!(
            writer.reconcile_volume_import(&changed).unwrap_err(),
            generation_error(GenerationCommitError::OperationConflict)
        );
    }
    assert_eq!(workspace.current(), "00000000000000000000000000000001");
    assert_eq!(
        workspace
            .connection()
            .query_row("SELECT count(*) FROM generations", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        2
    );
}

#[test]
fn new_operation_same_bytes_keeps_both_versions_and_stale_write_does_not_change_history() {
    let workspace = Workspace::new();
    let writer = workspace.writer();
    let (first, state, _) = workspace.prepared(1, "g0", &workspace.volume, b"same");
    writer.commit_volume_import(first.clone()).unwrap();
    let (next, next_state, _) = workspace.prepared(
        2,
        first.generation_id().as_str(),
        &workspace.volume,
        b"same",
    );
    assert_eq!(state, next_state);
    let result = writer.commit_volume_import(next).unwrap();
    assert!(!result.replayed);
    let (stale, _, _) = workspace.prepared(3, "g0", &workspace.volume, b"stale");
    assert_eq!(
        writer.commit_volume_import(stale).unwrap_err(),
        generation_error(GenerationCommitError::StaleGeneration)
    );
    assert_eq!(workspace.current(), result.generation_id.as_str());
    assert_eq!(
        workspace
            .connection()
            .query_row("SELECT count(*) FROM cp05_volume_versions", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        2
    );
    let mut reader = SqliteVolumeImportReader::open(&workspace.root).unwrap();
    let old = reader
        .begin_generation_snapshot(first.generation_id())
        .unwrap();
    assert_eq!(
        old.volume(&workspace.volume).unwrap().state,
        VolumeState::Imported(state)
    );
    old.finish().unwrap();
}

#[test]
fn source_image_corruption_in_ancestor_blocks_read_open_and_later_writes() {
    let workspace = Workspace::new();
    let writer = workspace.writer();
    let (first, _, resources) = workspace.prepared(1, "g0", &workspace.volume, b"first");
    writer.commit_volume_import(first.clone()).unwrap();
    writer
        .commit_volume_import(workspace.operation(
            2,
            first.generation_id().as_str(),
            ImportOperation::SetSeriesDefault {
                mode: ScopeMode::Story,
            },
            vec![],
        ))
        .unwrap();
    let source = workspace
        .root
        .join(resources[0].source_relative_path.as_ref().unwrap());
    fs::write(&source, b"wrong source bytes").unwrap();
    assert!(SqliteVolumeImportReader::open(&workspace.root).is_err());
    let typed = workspace.operation(
        3,
        "00000000000000000000000000000002",
        ImportOperation::SetSeriesDefault {
            mode: ScopeMode::AllText,
        },
        vec![],
    );
    assert_eq!(
        writer.commit_volume_import(typed).unwrap_err(),
        generation_error(GenerationCommitError::CorruptWorkspace)
    );
    let generic = GenerationWriteRequest::new(
        op(4),
        generation("00000000000000000000000000000002"),
        generation("00000000000000000000000000000004"),
        vec![],
    )
    .unwrap();
    assert_eq!(
        writer.commit_generation(generic).unwrap_err(),
        GenerationCommitError::CorruptWorkspace
    );
    assert_eq!(workspace.current(), "00000000000000000000000000000002");
    // Reconciliation does not depend on the original user path or current files.
    assert!(writer.reconcile_volume_import(&first).unwrap().replayed);
    drop(writer);
    assert_eq!(
        SqliteWorkspaceRepository.open(&workspace.root).unwrap_err(),
        WorkspaceError::CorruptWorkspace
    );
}

#[test]
fn opaque_blob_and_original_object_corruption_fail_closed_without_changing_current() {
    for damage_blob in [true, false] {
        let workspace = Workspace::new();
        let (first, state, _) = workspace.prepared(1, "g0", &workspace.volume, b"first");
        let writer = workspace.writer();
        writer.commit_volume_import(first).unwrap();
        drop(writer);
        if damage_blob {
            workspace
                .connection()
                .execute(
                    "UPDATE cp05_volume_versions SET payload=?1",
                    [&b"other"[..]],
                )
                .unwrap();
        } else {
            let original = nt_domain::object::ObjectHash::from_bytes(state.epub_sha256);
            fs::write(
                workspace
                    .root
                    .join("objects")
                    .join(format!("{original}.blob")),
                b"wrong",
            )
            .unwrap();
        }
        assert!(SqliteVolumeImportReader::open(&workspace.root).is_err());
        assert_eq!(workspace.current(), "00000000000000000000000000000001");
    }
}

#[test]
fn missing_and_foreign_volume_and_unknown_operation_leave_no_rows() {
    let workspace = Workspace::new();
    let writer = workspace.writer();
    let (request, state, resources) = workspace.prepared(1, "g0", &workspace.volume, b"first");
    let missing = VolumeId::parse("cccccccccccccccccccccccccccccccc").unwrap();
    let bad_volume = workspace.operation(
        1,
        "g0",
        ImportOperation::Publish {
            volume_id: missing,
            state: state.clone(),
            resources: resources.clone(),
            default_delta: None,
        },
        request.objects().to_vec(),
    );
    assert_eq!(
        writer.commit_volume_import(bad_volume).unwrap_err(),
        ImportWriteError::InvalidVolume
    );
    let foreign = VolumeImportWriteRequest::new(
        SeriesId::parse("dddddddddddddddddddddddddddddddd").unwrap(),
        op(1),
        generation("g0"),
        generation("00000000000000000000000000000001"),
        ImportOperation::Publish {
            volume_id: workspace.volume.clone(),
            state,
            resources,
            default_delta: None,
        },
        request.objects().to_vec(),
    )
    .unwrap();
    assert_eq!(
        writer.commit_volume_import(foreign).unwrap_err(),
        ImportWriteError::InvalidVolume
    );
    assert_eq!(
        writer.reconcile_volume_import(&request).unwrap_err(),
        generation_error(GenerationCommitError::CommitOutcomeUnknown)
    );
    assert_eq!(workspace.current(), "g0");
    assert_eq!(
        workspace
            .connection()
            .query_row("SELECT count(*) FROM generation_operations", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn concurrent_imports_on_one_parent_commit_once_and_preserve_the_other_volume() {
    let workspace = Workspace::new();
    let (one, _, _) = workspace.prepared(1, "g0", &workspace.volume, b"first");
    let (two, _, _) = workspace.prepared(2, "g0", &workspace.second, b"second");
    let barrier = Arc::new(Barrier::new(3));
    let mut threads = Vec::new();
    for request in [one, two] {
        let writer = workspace.writer();
        let barrier = Arc::clone(&barrier);
        threads.push(std::thread::spawn(move || {
            barrier.wait();
            writer.commit_volume_import(request)
        }));
    }
    barrier.wait();
    let results = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(
                |result| **result == Err(generation_error(GenerationCommitError::StaleGeneration))
            )
            .count(),
        1
    );
    let mut reader = SqliteVolumeImportReader::open(&workspace.root).unwrap();
    let view = reader.begin_snapshot().unwrap();
    let states =
        [&workspace.volume, &workspace.second].map(|volume| view.volume(volume).unwrap().state);
    assert_eq!(
        states
            .iter()
            .filter(|state| matches!(state, VolumeState::Imported(_)))
            .count(),
        1
    );
    view.finish().unwrap();
}

#[test]
fn digest_sink_checks_exact_bound_sticky_error_and_sha_without_retaining_payload() {
    let mut sink = ImportDigestSink::new(3).unwrap();
    sink.write_all(b"abc").unwrap();
    let digest = sink.finish().unwrap();
    assert_eq!(digest.byte_len, 3);
    assert_eq!(
        digest.hash.to_string(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let mut sink = ImportDigestSink::new(3).unwrap();
    sink.write_all(b"abc").unwrap();
    assert!(sink.write_all(b"d").is_err());
    assert!(sink.write(b"").is_err());
    assert_eq!(sink.finish().unwrap_err(), ImportWriteError::ResourceLimit);
}
