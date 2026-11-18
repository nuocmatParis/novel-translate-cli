//! Compact source records for the private CP05 volume payload.
//! The envelope and verified original-source reader compose these leaf records.
mod canonical_json;
mod inventory_records;
mod mapping_records;
pub(crate) mod memory;
mod records;
mod resource_records;
mod scope_records;
mod snapshot_records;
mod source_records;
mod text_runs;

pub use text_runs::{ProjectedSpan, TextRun, TextUnit, TextUnits, ValidatedTextRun};

pub(crate) use inventory_records::Severity as InventorySeverity;
pub use inventory_records::StoredInventory;
pub use mapping_records::StoredMapping;
pub(crate) use mapping_records::{MainFlow as StoredMainFlow, MapIssue as StoredMapIssue};
pub(crate) use records::Budget;
pub use records::{BlockLocator, Digest, ElementLocator, Span, TextLocator, UnitRange};
pub use resource_records::{
    ReferenceIssue, ResourceFileError, ResourceFiles, ResourceInput, ResourceRecord, ResourceRole,
    ResourceUse,
};
pub use scope_records::{
    ClassificationEvidence as ScopeEvidence, ContentKind as ScopeContentKind,
    Context as ScopeContext, Handling as ScopeHandling, Location as ScopeLocation, RangeLookup,
    RelatedText as ScopeRelatedText, ScopeRecord, StoredScope,
};
pub(crate) use scope_records::{
    DecisionMatch as ScopeDecisionMatch, RelationTarget as ScopeTarget,
};
pub(crate) use snapshot_records::SourceIssue;
pub use snapshot_records::{
    CaptureEvidence, CaptureVerificationError, SnapshotDraft, SnapshotInputs, SnapshotStageError,
    StagedVolume, VolumeDiagnostics, VolumeFormat, VolumeRecord, VolumeSource,
};
pub use source_records::{
    ArchiveSourceError, ReplayedSource, StoredSourceBlock, StoredSourceIndex, StoredUnitRun,
};
pub(crate) use source_records::{
    InlineKind as StoredInlineKind, ProtectedUnit as StoredProtectedUnit,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecError {
    Io,
    InvalidJson,
    NonCanonicalJson,
    InvalidTextRun,
    InvalidRecord,
    InvalidEnvelope,
    UnsupportedWireVersion,
    ResourceLimit,
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Io => "The stored source could not be read.",
            Self::InvalidJson => "The stored source JSON is invalid.",
            Self::NonCanonicalJson => "The stored source JSON is not canonical.",
            Self::InvalidTextRun => "The stored source text run is invalid.",
            Self::InvalidRecord => "The stored source record is invalid.",
            Self::InvalidEnvelope => "The stored volume envelope is invalid.",
            Self::UnsupportedWireVersion => "The stored volume format version is unsupported.",
            Self::ResourceLimit => "The stored source exceeds its safe resource limit.",
        })
    }
}
impl std::error::Error for CodecError {}
