use std::collections::BTreeMap;
use std::path::Path;

use nt_application::object_store::ObjectStoreError;
use nt_application::volume_import::{
    GenerationId, ImportReadError, ImportedVolumeState, SeriesId, VolumeId, VolumeImportVersion,
    VolumeResource, VolumeState, source_scope::DecisionContext,
};
use nt_storage::{
    FileImportSourceStore, FileObjectStore, SqliteVolumeImportReader, VolumeImportSnapshot,
};

use crate::import_codec::{
    CaptureVerificationError, CodecError, Digest, ResourceFiles, StoredScope, VolumeRecord,
};

use super::eligibility::{BlockingSource, check_dependencies};
use super::readiness::ImportReadiness;

#[derive(Debug)]
pub enum ImportFlowError {
    Capture(super::CaptureError),
    Archive(nt_epub::ArchiveError),
    Source(nt_application::volume_import::CapturedSourceError),
    Extraction(nt_epub::SourceExtractionError),
    Mapping(nt_application::volume_import::chapter_map::ChapterMapError),
    References(nt_epub::ReferenceIndexError),
    Scope(nt_application::source_scope::ScopeError),
    Stage(crate::import_codec::SnapshotStageError),
    Storage(ImportReadError),
    Files(ObjectStoreError),
    Codec(CodecError),
    Verification(CaptureVerificationError),
    Blocked(BlockingSource),
    InvalidWorkspace,
    InvalidSnapshot,
}

impl From<nt_application::volume_import::CapturedSourceError> for ImportFlowError {
    fn from(value: nt_application::volume_import::CapturedSourceError) -> Self {
        Self::Source(value)
    }
}

impl From<ImportReadError> for ImportFlowError {
    fn from(value: ImportReadError) -> Self {
        Self::Storage(value)
    }
}

impl From<CodecError> for ImportFlowError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}

impl From<nt_application::volume_import::InvalidImportRequest> for ImportFlowError {
    fn from(value: nt_application::volume_import::InvalidImportRequest) -> Self {
        use nt_application::volume_import::{ImportWriteError, InvalidImportRequest};
        let error = match value {
            InvalidImportRequest::Invalid => ImportWriteError::InvalidRequest,
            InvalidImportRequest::ResourceLimit => ImportWriteError::ResourceLimit,
        };
        Self::Storage(error)
    }
}

/// A volume version read from a single pinned workspace generation.
pub struct SavedVolume {
    pub generation: GenerationId,
    pub version: VolumeImportVersion,
    pub record: Option<VolumeRecord>,
}

impl SavedVolume {
    /// Readiness is derived from the verified snapshot on each read. A
    /// registered volume without an import has no source readiness yet.
    pub fn readiness(&self) -> Option<ImportReadiness> {
        self.record.as_ref().map(ImportReadiness::from_record)
    }
}

/// Read the current or an explicitly specified historical generation. The
/// persisted EPUB object, its resource uses and the scope are checked before
/// returning an imported volume.
pub fn read_saved_volume(
    workspace: &Path,
    series: &SeriesId,
    volume: &VolumeId,
    generation: Option<&GenerationId>,
) -> Result<SavedVolume, ImportFlowError> {
    let mut reader = SqliteVolumeImportReader::open(workspace)?;
    let snapshot = match generation {
        Some(generation) => reader.begin_generation_snapshot(generation)?,
        None => reader.begin_snapshot()?,
    };
    if snapshot.series_id() != series {
        return Err(ImportFlowError::InvalidWorkspace);
    }
    let version = snapshot.volume(volume)?;
    let record = match &version.state {
        VolumeState::Imported(state) => {
            let mut payload = snapshot
                .open_payload(volume)?
                .ok_or(ImportFlowError::InvalidSnapshot)?;
            if payload.object() != state.payload.object {
                return Err(ImportFlowError::InvalidSnapshot);
            }
            let record = VolumeRecord::read_unverified(&mut payload)?;
            drop(payload);
            Some(verify_import_record(
                workspace,
                &snapshot,
                volume,
                state,
                &version.resources,
                record,
            )?)
        }
        VolumeState::Unimported { .. } => None,
    };
    let generation = snapshot.generation_id().clone();
    snapshot.finish()?;
    Ok(SavedVolume {
        generation,
        version,
        record,
    })
}

/// Read the decisions and source locations for the exact displayed volume
/// version before preparing a classification or mode edit.
pub fn read_volume_scope(
    workspace: &Path,
    series: &SeriesId,
    volume: &VolumeId,
    generation: Option<&GenerationId>,
) -> Result<Option<nt_application::source_scope::ScopeEvaluation>, ImportFlowError> {
    let saved = read_saved_volume(workspace, series, volume, generation)?;
    let Some(mut record) = saved.record else {
        return Ok(None);
    };
    read_scope(workspace, series, volume, &saved.generation, &mut record).map(Some)
}

pub(super) fn verify_import_record(
    workspace: &Path,
    snapshot: &VolumeImportSnapshot<'_>,
    volume: &VolumeId,
    state: &ImportedVolumeState,
    resources: &[VolumeResource],
    mut record: VolumeRecord,
) -> Result<VolumeRecord, ImportFlowError> {
    check_identity(&record, state)?;
    let objects = FileObjectStore::new(workspace).map_err(ImportFlowError::Files)?;
    let images = FileImportSourceStore::new(workspace).map_err(ImportFlowError::Files)?;
    let files = ResourceFiles {
        objects: &objects,
        source_images: &images,
        volume,
        epub: Digest(state.epub_sha256),
    };
    let evidence = record
        .verify_capture(&files, resources, 0)
        .map_err(ImportFlowError::Verification)?;
    check_dependencies(
        &evidence.inventory,
        &evidence.references,
        &evidence.diagnostics,
    )
    .map_err(ImportFlowError::Blocked)?;
    let (actual_scope, scope_working) = super::scope_proof::rebuild_scope(&record, &evidence)?;
    drop(evidence);

    let scope = take_scope(snapshot, volume, &files, &mut record, scope_working)?;
    super::scope_proof::verify_observations(actual_scope, &scope)?;
    let base = record.retained_working_bytes()?;
    let mut working = nt_application::source_scope::ScopeWorking::new(
        nt_application::volume_import::MAX_IMPORT_WORKING_BYTES
            .checked_sub(base)
            .ok_or(CodecError::ResourceLimit)?,
    );
    working.result(&scope).map_err(ImportFlowError::Scope)?;
    record.scope = StoredScope::capture(
        &scope,
        base.checked_add(working.used())
            .ok_or(CodecError::ResourceLimit)?,
    )?;
    Ok(record)
}

pub(super) fn read_scope(
    workspace: &Path,
    series: &SeriesId,
    volume: &VolumeId,
    generation: &GenerationId,
    record: &mut VolumeRecord,
) -> Result<nt_application::source_scope::ScopeEvaluation, ImportFlowError> {
    let mut reader = SqliteVolumeImportReader::open(workspace)?;
    let snapshot = reader.begin_generation_snapshot(generation)?;
    if snapshot.series_id() != series {
        return Err(ImportFlowError::InvalidWorkspace);
    }
    let version = snapshot.volume(volume)?;
    let VolumeState::Imported(state) = version.state else {
        return Err(ImportFlowError::InvalidSnapshot);
    };
    check_identity(record, &state)?;
    let objects = FileObjectStore::new(workspace).map_err(ImportFlowError::Files)?;
    let source_images = FileImportSourceStore::new(workspace).map_err(ImportFlowError::Files)?;
    let files = ResourceFiles {
        objects: &objects,
        source_images: &source_images,
        volume,
        epub: Digest(state.epub_sha256),
    };
    let scope = take_scope(&snapshot, volume, &files, record, 0)?;
    snapshot.finish()?;
    Ok(scope)
}

fn take_scope(
    snapshot: &VolumeImportSnapshot<'_>,
    volume: &VolumeId,
    files: &ResourceFiles<'_>,
    record: &mut VolumeRecord,
    reserved: usize,
) -> Result<nt_application::source_scope::ScopeEvaluation, ImportFlowError> {
    let context = DecisionContext {
        series_id: snapshot.series_id().clone(),
        volume_id: volume.clone(),
        epub_sha256: record.source.epub_sha256.0,
        mapping_version: record.source.mapping_version.0,
    };
    let mode = record.scope.effective_mode;
    let current_working = record.scope_working_bytes(reserved)?;
    let stored = std::mem::replace(
        &mut record.scope,
        StoredScope {
            effective_mode: mode,
            records: Vec::new(),
            notes: Vec::new(),
        },
    );
    let mut current = BTreeMap::new();
    for document in &record.source_index.documents {
        for block in &document.blocks {
            if current
                .insert(
                    block.id,
                    (document.canonical_path.as_str(), block.unit_count()?),
                )
                .is_some()
            {
                return Err(ImportFlowError::InvalidSnapshot);
            }
        }
    }
    let mut previous: Option<(DecisionContext, HistoricalRanges)> = None;
    let scope = {
        let mut lookup = |key: &DecisionContext, path: &str, block: Digest| {
            if key.series_id != context.series_id || key.volume_id != context.volume_id {
                return Err(CodecError::InvalidRecord);
            }
            if key == &context {
                return current
                    .get(&block)
                    .filter(|(actual, _)| actual == &path)
                    .map(|(_, count)| *count)
                    .ok_or(CodecError::InvalidRecord);
            }
            if previous.as_ref().is_none_or(|(cached, _)| key != cached) {
                drop(previous.take());
                previous = Some((
                    key.clone(),
                    historical_ranges(snapshot, key, files, current_working)?,
                ));
            }
            previous
                .as_ref()
                .and_then(|(_, counts)| counts.get(&block))
                .filter(|(actual, _)| actual == path)
                .map(|(_, count)| *count)
                .ok_or(CodecError::InvalidRecord)
        };
        stored.restore(context.clone(), &mut lookup)?
    };
    // Historical range tables are no longer needed after restoring the keys.
    drop(previous);
    drop(current);
    StoredScope::verify_evaluation(
        &scope,
        nt_application::volume_import::MAX_IMPORT_WORKING_BYTES
            .checked_sub(current_working)
            .ok_or(CodecError::ResourceLimit)?,
    )?;
    Ok(scope)
}

type HistoricalRanges = BTreeMap<Digest, (String, u32)>;

fn historical_ranges(
    snapshot: &VolumeImportSnapshot<'_>,
    key: &DecisionContext,
    files: &ResourceFiles<'_>,
    current_working: usize,
) -> Result<HistoricalRanges, CodecError> {
    let (version, mut payload) = snapshot
        .open_payload_for_context(&key.volume_id, &key.epub_sha256, &key.mapping_version)
        .map_err(|_| CodecError::Io)?
        .ok_or(CodecError::InvalidRecord)?;
    let VolumeState::Imported(state) = &version.state else {
        return Err(CodecError::InvalidRecord);
    };
    let original = VolumeRecord::read_with_reservation(&mut payload, current_working)?;
    drop(payload);
    check_identity(&original, state).map_err(|_| CodecError::InvalidRecord)?;
    let history_files = ResourceFiles {
        epub: Digest(key.epub_sha256),
        ..*files
    };
    original
        .verify_capture(&history_files, &version.resources, current_working)
        .map_err(|error| match error {
            CaptureVerificationError::Codec(error)
            | CaptureVerificationError::Source(crate::import_codec::ArchiveSourceError::Codec(
                error,
            ))
            | CaptureVerificationError::Resources(crate::import_codec::ResourceFileError::Codec(
                error,
            )) => error,
            CaptureVerificationError::Source(
                crate::import_codec::ArchiveSourceError::XmlProof(
                    nt_application::volume_import::CapturedSourceError::ResourceLimit,
                ),
            ) => CodecError::ResourceLimit,
            _ => CodecError::InvalidRecord,
        })?;
    let mut retained = crate::import_codec::Budget::new(crate::import_codec::memory::working(
        &original,
        current_working,
    )?)?;
    let mut ranges = BTreeMap::new();
    for document in original.source_index.documents {
        for block in document.blocks {
            // Include the key/value, tree node space and a separate path for
            // each block before extending the historical lookup.
            retained.charge(
                2 * size_of::<(Digest, String, u32)>() + document.canonical_path.len(),
                1,
            )?;
            if ranges
                .insert(
                    block.id,
                    (document.canonical_path.clone(), block.unit_count()?),
                )
                .is_some()
            {
                return Err(CodecError::InvalidRecord);
            }
        }
    }
    Ok(ranges)
}

fn check_identity(
    record: &VolumeRecord,
    state: &ImportedVolumeState,
) -> Result<(), ImportFlowError> {
    if record.source.epub_sha256.0 != state.epub_sha256
        || record.source.mapping_version.0 != state.mapping_version
        || record.scope.mode() != state.mode
    {
        return Err(ImportFlowError::InvalidSnapshot);
    }
    Ok(())
}
