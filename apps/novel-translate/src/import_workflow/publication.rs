use std::path::{Path, PathBuf};

use nt_application::volume_import::{
    GenerationCommitError, GenerationCommitResult, GenerationId, ImportOperation, ImportWriteError,
    OperationId, SeriesId, VolumeId, VolumeImportRepository, VolumeImportWriteRequest, VolumeState,
    source_scope::ScopeMode,
};
use nt_storage::{
    FileObjectStore, SqliteGenerationWriter, SqliteVolumeImportReader, VolumeImportSnapshot,
};

use crate::import_codec::{StagedVolume, VolumeRecord};

use super::persisted::{ImportFlowError, SavedVolume, read_saved_volume, verify_import_record};

pub struct ImportTarget {
    pub series: SeriesId,
    pub volume: VolumeId,
    pub expected_parent: GenerationId,
    pub operation: OperationId,
    pub generation: GenerationId,
}

/// An exact request retained across submission, retry and acknowledgement
/// reconciliation. Replays use this request without reopening the user's EPUB.
pub struct PreparedImport {
    workspace: PathBuf,
    volume: VolumeId,
    request: VolumeImportWriteRequest,
}

pub enum Publication {
    Published {
        saved: Box<SavedVolume>,
        replayed: bool,
    },
    CommitOutcomeUnknown {
        operation: OperationId,
        fingerprint: [u8; 32],
    },
    SavedButUnreadable {
        generation: GenerationId,
        error: ImportFlowError,
    },
}

/// Finish application preparation with a closed payload that has already
/// been staged. Verify the exact immutable object the writer will consume.
pub fn prepare_staged_import(
    workspace: &Path,
    target: ImportTarget,
    staged: StagedVolume,
) -> Result<PreparedImport, ImportFlowError> {
    prepare_staged_change(workspace, target, staged, ChangeKind::Import)
}

pub(super) fn prepare_staged_scope(
    workspace: &Path,
    target: ImportTarget,
    staged: StagedVolume,
    default_delta: Option<ScopeMode>,
) -> Result<PreparedImport, ImportFlowError> {
    prepare_staged_change(
        workspace,
        target,
        staged,
        ChangeKind::Scope { default_delta },
    )
}

pub(super) fn prepare_staged_registration(
    workspace: &Path,
    target: ImportTarget,
    staged: StagedVolume,
    display_title: Option<String>,
) -> Result<PreparedImport, ImportFlowError> {
    prepare_staged_change(
        workspace,
        target,
        staged,
        ChangeKind::Registration { display_title },
    )
}

pub(super) fn registration_mode(
    workspace: &Path,
    target: &ImportTarget,
) -> Result<ScopeMode, ImportFlowError> {
    let mut reader = SqliteVolumeImportReader::open(workspace)?;
    let snapshot = reader.begin_snapshot()?;
    validate_parent(&snapshot, target)?;
    let mode = vacant_volume_mode(&snapshot, target)?;
    snapshot.finish()?;
    Ok(mode)
}

fn vacant_volume_mode(
    snapshot: &VolumeImportSnapshot<'_>,
    target: &ImportTarget,
) -> Result<ScopeMode, ImportFlowError> {
    match snapshot.volume(&target.volume) {
        Err(ImportWriteError::InvalidVolume) => Ok(snapshot.series_default()?),
        Ok(_) => Err(ImportWriteError::InvalidVolume.into()),
        Err(error) => Err(error.into()),
    }
}

enum ChangeKind {
    Import,
    Registration { display_title: Option<String> },
    Scope { default_delta: Option<ScopeMode> },
}

fn validate_parent(
    snapshot: &VolumeImportSnapshot<'_>,
    target: &ImportTarget,
) -> Result<(), ImportFlowError> {
    if snapshot.series_id() != &target.series {
        return Err(ImportFlowError::InvalidWorkspace);
    }
    if snapshot.generation_id() != &target.expected_parent {
        return Err(ImportWriteError::from(GenerationCommitError::StaleGeneration).into());
    }
    Ok(())
}

fn prepare_staged_change(
    workspace: &Path,
    target: ImportTarget,
    mut staged: StagedVolume,
    kind: ChangeKind,
) -> Result<PreparedImport, ImportFlowError> {
    let mut reader = SqliteVolumeImportReader::open(workspace)?;
    let snapshot = reader.begin_snapshot()?;
    validate_parent(&snapshot, &target)?;
    let order = |left: &nt_application::volume_import::VolumeResource,
                 right: &nt_application::volume_import::VolumeResource| {
        (&left.role, &left.canonical_path).cmp(&(&right.role, &right.canonical_path))
    };
    staged.resources.sort_unstable_by(order);
    match &kind {
        ChangeKind::Import => {
            if snapshot.volume(&target.volume)?.state.mode() != staged.state.mode {
                return Err(ImportFlowError::InvalidSnapshot);
            }
        }
        ChangeKind::Registration { .. } => {
            if vacant_volume_mode(&snapshot, &target)? != staged.state.mode {
                return Err(ImportFlowError::InvalidSnapshot);
            }
        }
        ChangeKind::Scope { .. } => {
            let mut current = snapshot.volume(&target.volume)?;
            current.resources.sort_unstable_by(order);
            let VolumeState::Imported(before) = &current.state else {
                return Err(ImportFlowError::InvalidSnapshot);
            };
            if before.epub_sha256 != staged.state.epub_sha256
                || before.mapping_version != staged.state.mapping_version
                || current.resources != staged.resources
            {
                return Err(ImportFlowError::InvalidSnapshot);
            }
        }
    }
    let StagedVolume {
        record,
        state,
        resources,
        objects,
    } = staged;
    // Use the sealed object as the proof input. This also exercises the same
    // bounded decoder a reader will use, before any SQLite generation moves.
    drop(record);
    let store = FileObjectStore::new(workspace).map_err(ImportFlowError::Files)?;
    let mut payload = store
        .open_import_object(state.payload.object)
        .map_err(ImportFlowError::Files)?;
    let record = VolumeRecord::read_unverified(&mut payload)?;
    drop(payload);
    let mut expected_refs = Vec::with_capacity(record.resources.len() + 2);
    expected_refs.push(state.payload.object);
    expected_refs.push(nt_application::volume_import::ObjectReference {
        hash: nt_application::volume_import::ObjectHash::from_bytes(
            record.source.original_object_sha256.0,
        ),
        byte_len: record.source.original_object_byte_len,
    });
    expected_refs.extend(
        record
            .resources
            .iter()
            .map(crate::import_codec::ResourceRecord::object),
    );
    expected_refs.sort_unstable_by_key(|object| object.hash);
    expected_refs.dedup();
    if expected_refs != objects {
        return Err(ImportFlowError::InvalidSnapshot);
    }
    verify_import_record(
        workspace,
        &snapshot,
        &target.volume,
        &state,
        &resources,
        record,
    )?;
    snapshot.finish()?;
    let operation = match &kind {
        ChangeKind::Import | ChangeKind::Registration { .. } => ImportOperation::Publish {
            volume_id: target.volume.clone(),
            state,
            resources,
            default_delta: None,
        },
        ChangeKind::Scope { default_delta } => ImportOperation::SetVolumeScope {
            volume_id: target.volume.clone(),
            state: VolumeState::Imported(state),
            resources,
            default_delta: *default_delta,
        },
    };
    let request = VolumeImportWriteRequest::new(
        target.series,
        target.operation,
        target.expected_parent,
        target.generation,
        operation,
        objects,
    )
    .map_err(ImportFlowError::from)?;
    let request = if let ChangeKind::Registration { display_title } = kind {
        request.with_registration(display_title)?
    } else {
        request
    };
    Ok(PreparedImport::new(workspace, target.volume, request))
}

impl PreparedImport {
    pub(super) fn new(
        workspace: &Path,
        volume: VolumeId,
        request: VolumeImportWriteRequest,
    ) -> Self {
        Self {
            workspace: workspace.to_path_buf(),
            volume,
            request,
        }
    }

    pub fn fingerprint(&self) -> [u8; 32] {
        SqliteGenerationWriter::import_fingerprint(&self.request)
    }

    pub fn submit(&self) -> Result<Publication, ImportFlowError> {
        let writer = self.writer()?;
        let result = writer.commit_volume_import(self.request.clone());
        self.finish(result)
    }

    pub fn reconcile(&self) -> Result<Publication, ImportFlowError> {
        let writer = self.writer()?;
        let result = writer.reconcile_volume_import(&self.request);
        self.finish(result)
    }

    fn writer(&self) -> Result<SqliteGenerationWriter, ImportFlowError> {
        SqliteGenerationWriter::open(&self.workspace)
            .map_err(|error| ImportWriteError::from(error).into())
    }

    fn finish(
        &self,
        result: Result<GenerationCommitResult, ImportWriteError>,
    ) -> Result<Publication, ImportFlowError> {
        let saved = match result {
            Ok(value) => value,
            Err(ImportWriteError::Generation(GenerationCommitError::CommitOutcomeUnknown)) => {
                return Ok(Publication::CommitOutcomeUnknown {
                    operation: self.request.operation_id().clone(),
                    fingerprint: self.fingerprint(),
                });
            }
            Err(error) => return Err(error.into()),
        };
        let read = read_saved_volume(
            &self.workspace,
            self.request.series_id(),
            &self.volume,
            Some(&saved.generation_id),
        )
        .and_then(|volume| {
            let (state, resources) = match self.request.operation() {
                ImportOperation::Publish {
                    state, resources, ..
                } => (VolumeState::Imported(state.clone()), resources),
                ImportOperation::SetVolumeScope {
                    state, resources, ..
                } => (state.clone(), resources),
                ImportOperation::SetSeriesDefault { .. } => {
                    return Err(ImportFlowError::InvalidSnapshot);
                }
            };
            if volume.version.generation_id.as_ref() != Some(&saved.generation_id)
                || volume.version.state != state
                || &volume.version.resources != resources
            {
                return Err(ImportFlowError::InvalidSnapshot);
            }
            Ok(volume)
        });
        Ok(match read {
            Ok(value) => Publication::Published {
                saved: Box::new(value),
                replayed: saved.replayed,
            },
            Err(error) => Publication::SavedButUnreadable {
                generation: saved.generation_id,
                error,
            },
        })
    }
}
