//! Application port for prepared volume/default generation writes.

pub use nt_domain::object::{ObjectHash, ObjectReference};
pub use nt_domain::volume_import::{
    CapturedBlock, CapturedBlockPlacement, CapturedImagePlacement, CapturedNavigationLabel,
    CapturedScopeElement, CapturedSourceError, CapturedSourceRequest, CapturedUnitRun,
    DetectedImageFormat, ImageInformation, ImportOperation, ImportedVolumeState,
    InvalidImportRequest, MAX_IMPORT_PAYLOAD_BYTES, MAX_IMPORT_RECORDS, MAX_IMPORT_WORKING_BYTES,
    StagedPayload, VolumeImportVersion, VolumeImportWriteRequest, VolumeResource, VolumeState,
};
pub use nt_domain::workspace::{
    GenerationCommitError, GenerationCommitResult, GenerationId, OperationId, SeriesId, VolumeId,
};
pub use nt_domain::{chapter_map, source_range, source_scope};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportWriteError {
    Generation(GenerationCommitError),
    InvalidVolume,
    InvalidRequest,
    ResourceLimit,
    PayloadMismatch,
}

pub type ImportReadError = ImportWriteError;

impl From<GenerationCommitError> for ImportWriteError {
    fn from(value: GenerationCommitError) -> Self {
        Self::Generation(value)
    }
}

/// The implementation serializes these operations with generic generation
/// writes. Callers retain the request/operation ID for outcome reconciliation.
pub trait VolumeImportRepository: Send + Sync {
    fn commit_volume_import(
        &self,
        request: VolumeImportWriteRequest,
    ) -> Result<GenerationCommitResult, ImportWriteError>;

    fn reconcile_volume_import(
        &self,
        request: &VolumeImportWriteRequest,
    ) -> Result<GenerationCommitResult, ImportWriteError>;
}

pub fn commit_volume_import(
    repository: &impl VolumeImportRepository,
    request: VolumeImportWriteRequest,
) -> Result<GenerationCommitResult, ImportWriteError> {
    repository.commit_volume_import(request)
}
