//! Safe, bounded access to EPUB ZIP entries.
//!
//! This crate validates the container and archive paths before exposing entry
//! bytes. It deliberately does not parse the EPUB package or materialize files.

mod archive;
mod container;
mod effective_base;
mod encoding;
mod encryption;
mod inventory;
mod navigation;
mod package;
mod path;
mod reference_index;
mod source_extract;
mod xml;

pub use archive::{ArchiveEntry, ArchiveError, ArchiveLimits, EntryId, EpubZip};
pub use inventory::inspect;
pub use inventory::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, EpubInventory, ImageUse, InventoryCoverage,
    Metadata, MetadataValue, NavigationItem, NavigationSource, NavigationTarget, NavigationTree,
    Resource, ResourceEncryption, SpineContentKind, SpineItem,
};
pub use path::{ArchivePathError, canonical_archive_path, windows_materialization_key};
pub use reference_index::{
    ReferenceCoverage, ReferenceDiagnostic, ReferenceIndex, ReferenceIndexError,
    ReferenceIndexLimits, ReferenceKind, ReferenceLocation, ReferenceTarget, ResourceReference,
    index_references,
};
pub use source_extract::{
    CoverageDomain, PARSER_VERSION, SourceAnchor, SourceDiagnostic, SourceDocument,
    SourceExtraction, SourceExtractionError, extract_source,
};
