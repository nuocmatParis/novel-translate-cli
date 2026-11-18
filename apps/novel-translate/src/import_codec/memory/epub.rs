use nt_application::volume_import::CapturedNavigationLabel;
use nt_application::volume_import::chapter_map as map;
use nt_epub::{
    Diagnostic, EpubInventory, ImageUse, Metadata, MetadataValue, NavigationItem, NavigationTarget,
    NavigationTree, ReferenceDiagnostic, ReferenceIndex, ReferenceIndexError, ReferenceLocation,
    ReferenceTarget, Resource, ResourceReference, SourceDiagnostic, SpineItem,
};

use super::{CodecError, Heap};
use crate::import_codec::records::Budget;

scalar!(
    nt_epub::DiagnosticCode,
    nt_epub::DiagnosticSeverity,
    nt_epub::InventoryCoverage,
    nt_epub::NavigationSource,
    nt_epub::ResourceEncryption,
    nt_epub::SpineContentKind,
    nt_epub::ReferenceKind,
    nt_epub::ReferenceCoverage
);

heap_struct!(EpubInventory {
    package_path,
    package_version,
    metadata,
    resources,
    spine,
    navigation,
    primary_navigation,
    coverage,
    diagnostics
});
heap_struct!(Metadata {
    values,
    unique_identifier_id
});
heap_struct!(MetadataValue {
    namespace,
    name,
    value,
    id,
    property,
    refines,
    scheme
});
heap_struct!(Resource {
    id,
    path,
    media_type,
    properties,
    fallback,
    media_overlay,
    available,
    encryption,
    image_uses
});
heap_struct!(SpineItem {
    idref,
    resource_index,
    reading_order,
    linear,
    kind,
    image_uses
});
heap_struct!(ImageUse {
    resource_path,
    manifest_resource_index,
    occurrence,
    external,
    line,
    column
});
heap_struct!(Diagnostic {
    code,
    severity,
    resource_path,
    line,
    column,
    related_resource
});
heap_struct!(NavigationTree {
    source,
    resource_path,
    items
});
heap_struct!(NavigationItem {
    label,
    target,
    children
});
heap_struct!(NavigationTarget {
    resource_path,
    fragment
});
heap_struct!(ReferenceIndex {
    references,
    diagnostics,
    coverage
});
heap_struct!(ReferenceDiagnostic {
    source_path,
    location,
    code
});
heap_struct!(ReferenceLocation {
    element_ordinals,
    attribute_namespace,
    attribute_local_name,
    candidate_ordinal,
    line,
    column,
    decoded_byte_range
});
heap_struct!(ResourceReference {
    source_path,
    location,
    kind,
    target,
    raw_reference,
    spine_positions,
    inventory_occurrence,
    source_occurrences
});
heap_enum!(ReferenceTarget { Internal { path, fragment }, Missing { path, fragment }, Ambiguous,
    External { scheme }, Invalid { reason } });
heap_enum!(SourceDiagnostic { InventoryPartial, InvalidOptionalNavigation { canonical_path },
    Inventory(diagnostic), UnsupportedFixedLayout { canonical_path },
    UnsupportedEncryption { canonical_path }, DuplicateXhtmlId { canonical_path, id },
    ProtectedResourceMissing { canonical_path } });
heap_struct!(CapturedNavigationLabel {
    canonical_path,
    item_ordinals,
    element_ordinals,
    text,
    raw_href
});
use map::{
    NavigationItem as MapItem, NavigationTarget as MapTarget, NavigationTree as MapTree,
    SourceInputDiagnostic,
};
scalar!(map::NavigationSource, map::NavigationTreeState);
heap_struct!(MapTree {
    source,
    resource_path,
    state,
    items
});
heap_struct!(MapItem {
    label,
    raw_href,
    target,
    children
});
heap_enum!(MapTarget { Resolved { canonical_path, fragment }, Invalid { reason } });
heap_struct!(SourceInputDiagnostic {
    source,
    resource_path,
    code,
    reason,
    line,
    column
});
heap_enum!(ReferenceIndexError { InvalidInput, Archive, ArchiveEntryLimit, ArchiveCumulativeLimit,
    XmlMalformed, XmlByteLimit, XmlNodeLimit, XmlDepthLimit, ExternalEntity, UnknownEntity,
    EntityExpansionLimit, UnsupportedEncoding, FileCountLimit, ReferenceCountLimit, ReferenceByteLimit,
    RawEvidenceLimit, WorkingMemoryLimit, CssByteLimit, CssTokenLimit, CssFunctionDepthLimit, CssImportDepthLimit, InvalidCss,
    InvalidSrcset, UnsafeUri, SourceMismatch, Missing, Ambiguous, External, ImportCycle, OptionalNavigationSkipped,
    BaseByteLimit, BaseNodeLimit, BaseWorkLimit, At { source_path, line, column, reason } });

#[cfg(test)]
mod tests {
    use super::{EpubInventory, ReferenceIndex};
    use crate::import_codec::{CodecError, memory::working};
    use nt_application::volume_import::MAX_IMPORT_WORKING_BYTES;
    use nt_epub::{ReferenceCoverage, ResourceReference};

    #[test]
    fn live_inventory_and_reference_vectors_share_the_same_remaining_space() {
        let inventory = EpubInventory {
            package_path: Some(String::with_capacity(4096)),
            ..Default::default()
        };
        let references = ReferenceIndex {
            references: Vec::with_capacity(8),
            diagnostics: Vec::new(),
            coverage: ReferenceCoverage::Complete,
        };
        let inventory_bytes =
            size_of::<EpubInventory>() + inventory.package_path.as_ref().unwrap().capacity();
        let graph_bytes = size_of::<ReferenceIndex>()
            + references.references.capacity() * size_of::<ResourceReference>();
        let caller = MAX_IMPORT_WORKING_BYTES - inventory_bytes - graph_bytes;
        let with_inventory = working(&inventory, caller).unwrap();
        assert_eq!(
            working(&references, with_inventory),
            Ok(MAX_IMPORT_WORKING_BYTES)
        );
        assert_eq!(
            working(&references, with_inventory + 1),
            Err(CodecError::ResourceLimit)
        );
    }
}
