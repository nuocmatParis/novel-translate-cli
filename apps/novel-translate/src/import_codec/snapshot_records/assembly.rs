//! Assemble captured producer outputs into the existing V2 storage format.

use nt_application::object_store::ObjectStoreError;
use nt_application::source_scope::ScopeEvaluation;
use nt_application::volume_import::{
    ImportedVolumeState, ObjectReference, SeriesId, VolumeResource, chapter_map,
    source_scope::DecisionContext,
};
use nt_epub::{EpubInventory, EpubZip, ReferenceIndex, SourceExtraction};
use nt_storage::FileObjectStore;

use super::super::{ResourceFileError, ResourceFiles};
use super::{
    Budget, CodecError, Digest, ResourceRecord, StoredInventory, StoredMapping, StoredScope,
    StoredSourceIndex, VERSION, VolumeDiagnostics, VolumeFormat, VolumeRecord, VolumeSource,
    mapping_digest,
};
use std::io::{self, Read, Seek};

pub struct SnapshotInputs<'a> {
    pub original: ObjectReference,
    pub series: &'a SeriesId,
    pub inventory: &'a EpubInventory,
    pub source: &'a SourceExtraction,
    pub mapping: &'a chapter_map::ChapterMap,
    pub confirmed_boundaries: &'a [chapter_map::ConfirmedBoundary],
    pub references: &'a ReferenceIndex,
    pub mapping_rule_version: &'a str,
    pub classification_rule_version: &'a str,
}

pub struct SnapshotDraft {
    budget: Budget,
    source: VolumeSource,
    inventory: StoredInventory,
    source_index: StoredSourceIndex,
    mapping: StoredMapping,
    resources: Vec<ResourceRecord>,
    diagnostics: VolumeDiagnostics,
    context: DecisionContext,
}

#[derive(Debug)]
pub enum SnapshotStageError {
    Codec(CodecError),
    Resources(ResourceFileError),
    Storage(ObjectStoreError),
    ContextMismatch,
}

impl From<CodecError> for SnapshotStageError {
    fn from(error: CodecError) -> Self {
        Self::Codec(error)
    }
}

pub struct StagedVolume {
    pub record: VolumeRecord,
    pub state: ImportedVolumeState,
    pub resources: Vec<VolumeResource>,
    pub objects: Vec<ObjectReference>,
}

impl SnapshotDraft {
    /// Capture the outputs from one pinned original EPUB. The caller checks
    /// mandatory eligibility and keeps the parsing/archive reservation live.
    pub fn capture(
        inputs: SnapshotInputs<'_>,
        files: &ResourceFiles<'_>,
        archive: &mut EpubZip<impl Read + Seek>,
        reserved_working_bytes: usize,
    ) -> Result<Self, SnapshotStageError> {
        if inputs.source.epub_sha256 != *inputs.original.hash.as_bytes()
            || inputs.source.epub_sha256 != files.epub.0
            || inputs.original.byte_len == 0
        {
            return Err(SnapshotStageError::ContextMismatch);
        }
        let mut budget = Budget::new(reserved_working_bytes)?;
        let inventory = StoredInventory::capture_with_budget(inputs.inventory, &mut budget)?;
        let source_index = StoredSourceIndex::capture_with_budget(inputs.source, &mut budget)?;
        let mapping = StoredMapping::capture_with_budget(
            inputs.mapping,
            inputs.confirmed_boundaries,
            &mut budget,
        )?;
        mapping.validate_coverage(&source_index, files.epub, &inputs.source.parser_version)?;
        let diagnostics = VolumeDiagnostics::capture_with_budget(
            inputs.source,
            inputs.mapping,
            inputs.references,
            &mut budget,
        )?;
        let resources = files
            .stage_with_budget(archive, inputs.inventory, inputs.references, &mut budget)
            .map_err(SnapshotStageError::Resources)?;
        let mut source = VolumeSource {
            epub_sha256: files.epub,
            original_object_sha256: files.epub,
            original_object_byte_len: inputs.original.byte_len,
            parser_version: budget.text(&inputs.source.parser_version)?,
            mapping_rule_version: budget.text(inputs.mapping_rule_version)?,
            classification_rule_version: budget.text(inputs.classification_rule_version)?,
            mapping_version: Digest([0; 32]),
        };
        source.mapping_version = mapping_digest(&source, &source_index, &mapping, &resources)?;
        let context = DecisionContext {
            series_id: inputs.series.clone(),
            volume_id: files.volume.clone(),
            epub_sha256: files.epub.0,
            mapping_version: source.mapping_version.0,
        };
        Ok(Self {
            budget,
            source,
            inventory,
            source_index,
            mapping,
            resources,
            diagnostics,
            context,
        })
    }

    pub fn decision_context(&self) -> &DecisionContext {
        &self.context
    }

    pub(crate) fn remaining_working_bytes(&self) -> usize {
        self.budget.remaining_bytes()
    }

    pub(crate) fn reserve_scope_evaluation(&mut self, bytes: usize) -> Result<(), CodecError> {
        self.budget.charge(bytes, 0)
    }

    /// Seal the payload and files needed by the volume writer. The caller
    /// submits them through the shared writer and verifies the saved result.
    pub fn stage(
        mut self,
        scope: &ScopeEvaluation,
        objects: &FileObjectStore,
    ) -> Result<StagedVolume, SnapshotStageError> {
        if &scope.context != self.decision_context() {
            return Err(SnapshotStageError::ContextMismatch);
        }
        let stored_scope = StoredScope::capture_with_budget(scope, &mut self.budget)?;
        let record = VolumeRecord {
            format: VolumeFormat::Volume,
            version: VERSION,
            source: self.source,
            inventory: self.inventory,
            source_index: self.source_index,
            mapping: self.mapping,
            scope: stored_scope,
            resources: self.resources,
            diagnostics: self.diagnostics,
        };
        record.stage_with_budget(objects, self.budget)
    }
}

impl VolumeRecord {
    /// Seal a new scope version of an already verified source record.
    pub fn stage(
        self,
        objects: &FileObjectStore,
        reserved_working_bytes: usize,
    ) -> Result<StagedVolume, SnapshotStageError> {
        let live = super::super::memory::working(&self, reserved_working_bytes)?;
        self.stage_with_budget(objects, Budget::new(live)?)
    }

    fn stage_with_budget(
        self,
        objects: &FileObjectStore,
        mut budget: Budget,
    ) -> Result<StagedVolume, SnapshotStageError> {
        self.validate_structure()?;
        let mut refs = budget.vector(
            self.resources
                .len()
                .checked_add(2)
                .ok_or(CodecError::ResourceLimit)?,
        )?;
        refs.push(ObjectReference {
            hash: nt_application::volume_import::ObjectHash::from_bytes(
                self.source.original_object_sha256.0,
            ),
            byte_len: self.source.original_object_byte_len,
        });
        let mut resources = budget.vector(self.resources.len())?;
        for resource in &self.resources {
            let object = resource.object();
            refs.push(object);
            resources.push(VolumeResource {
                role: budget.text(resource.role.storage_name())?,
                canonical_path: budget.text(&resource.canonical_path)?,
                object,
                source_relative_path: budget.optional_text(&resource.source_relative_path)?,
            });
        }

        budget.charge(128 * 1024, 0)?;
        let payload = objects
            .stage_encoded_import(|output| {
                self.write_unverified(output)
                    .map(|_| ())
                    .map_err(io::Error::other)
            })
            .map_err(SnapshotStageError::Storage)?;
        {
            let mut sealed = objects
                .open_import_object(payload.object)
                .map_err(SnapshotStageError::Storage)?;
            VolumeRecord::check_staged_read_limits(&mut sealed)?;
        }
        refs.push(payload.object);
        refs.sort_unstable_by_key(|object| (object.hash, object.byte_len));
        for pair in refs.windows(2) {
            if pair[0].hash == pair[1].hash && pair[0].byte_len != pair[1].byte_len {
                return Err(CodecError::InvalidRecord.into());
            }
        }
        refs.dedup();
        Ok(StagedVolume {
            state: ImportedVolumeState {
                mode: self.scope.mode(),
                epub_sha256: self.source.epub_sha256.0,
                mapping_version: self.source.mapping_version.0,
                payload,
            },
            record: self,
            resources,
            objects: refs,
        })
    }
}
