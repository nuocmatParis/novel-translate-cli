//! Inventory DTOs keep source order and flatten the bounded Nav/NCX trees.
use super::CodecError;
use super::records::{Budget, safe_path};
use nt_epub as epub;
use serde::{Deserialize, Serialize};

macro_rules! names {
    ($name:ident,$source:path,$($variant:ident),+ $(,)?)=>{
        #[derive(Clone,Copy,Debug,Eq,PartialEq,Serialize,Deserialize)]
        pub enum $name {$($variant),+}
        $crate::import_codec::memory::heap_enum!($name {$($variant),+});
        impl From<$source> for $name {
            fn from(value:$source)->Self {use $source as Source;match value {$(Source::$variant=>Self::$variant),+}}
        }
        impl From<$name> for $source {
            fn from(value:$name)->Self {use $source as Source;match value {$($name::$variant=>Source::$variant),+}}
        }
    };
}
pub(super) use names;

names!(Coverage, epub::InventoryCoverage, Complete, Partial);
names!(
    Severity,
    epub::DiagnosticSeverity,
    Info,
    Warning,
    Incomplete
);
names!(
    Encryption,
    epub::ResourceEncryption,
    NotEncrypted,
    IdpfFontObfuscation,
    AdobeFontObfuscation,
    Unsupported
);
names!(
    SpineKind,
    epub::SpineContentKind,
    ReflowableDocument,
    FixedLayoutDocument,
    ImagePage,
    Unsupported
);
names!(NavigationSource, epub::NavigationSource, Epub3Nav, Epub2Ncx);
names!(
    DiagnosticCode,
    epub::DiagnosticCode,
    ArchiveEntryUnreadable,
    ContainerMissing,
    ContainerInvalidXml,
    ContainerRootfileMissing,
    ContainerRootfileInvalid,
    PackageMissing,
    PackageInvalidXml,
    PackageRootInvalid,
    PackageVersionMissing,
    PackageVersionUnsupported,
    UniqueIdentifierMissing,
    UniqueIdentifierUnresolved,
    MetadataMissing,
    ManifestMissing,
    ManifestIdMissing,
    ManifestIdDuplicate,
    ManifestItemInvalid,
    ResourceHrefInvalid,
    ExternalResourceNotRead,
    ResourceMissing,
    ResourcePathAmbiguous,
    SpineMissing,
    SpineEmpty,
    SpineReferenceInvalid,
    SpineResourceMissing,
    SpineLinearInvalid,
    SpineTocReferenceInvalid,
    XhtmlNamespaceInvalid,
    UnsupportedSpineMediaType,
    ReferencedImageMissing,
    ReferencedImageUnlisted,
    ReferencedImageAmbiguous,
    ImageOnlyContentNeedsImageWorkflow,
    FixedLayoutTextUnsupported,
    FixedLayoutPageUnsupported,
    NavigationMissing,
    NavigationInvalidXml,
    NavigationStructureUnsupported,
    NavigationDepthLimitExceeded,
    NavigationTargetMissing,
    NavigationTargetUnlisted,
    NavigationTargetAmbiguous,
    NavigationSourcesConflict,
    ExternalReferenceNotRead,
    EncryptionMetadataInvalid,
    EncryptedResourceMissing,
    EncryptedResourceUnlisted,
    UnsupportedEncryption,
    FontObfuscationDetected,
    ScriptedContentUnsupported,
    RemoteResourcesUnsupported,
    XmlByteLimitExceeded,
    XmlNodeLimitExceeded,
    XmlEncodingUnsupported,
    XmlExternalEntityRejected,
    XmlEntityReferenceUnknown,
    XmlEntityExpansionLimit,
    XmlMalformed,
    BaseResolutionLimitExceeded
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryDiagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub resource_path: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub related_resource: Option<String>,
}
impl InventoryDiagnostic {
    pub(super) fn capture(d: &epub::Diagnostic, budget: &mut Budget) -> Result<Self, CodecError> {
        Ok(Self {
            code: d.code.into(),
            severity: d.severity.into(),
            resource_path: budget.optional_text(&d.resource_path)?,
            line: d.line,
            column: d.column,
            related_resource: budget.optional_text(&d.related_resource)?,
        })
    }
    fn restore(self) -> epub::Diagnostic {
        epub::Diagnostic {
            code: self.code.into(),
            severity: self.severity.into(),
            resource_path: self.resource_path,
            line: self.line,
            column: self.column,
            related_resource: self.related_resource,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataValue {
    pub namespace: Option<String>,
    pub name: String,
    pub value: String,
    pub id: Option<String>,
    pub property: Option<String>,
    pub refines: Option<String>,
    pub scheme: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub values: Vec<MetadataValue>,
    pub unique_identifier_id: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageUse {
    pub resource_path: Option<String>,
    pub manifest_resource_index: Option<u64>,
    pub occurrence: u64,
    pub external: bool,
    pub line: u32,
    pub column: u32,
}
fn usize64(n: usize) -> Result<u64, CodecError> {
    u64::try_from(n).map_err(|_| CodecError::ResourceLimit)
}
fn width(n: u64) -> Result<usize, CodecError> {
    usize::try_from(n).map_err(|_| CodecError::ResourceLimit)
}
impl ImageUse {
    fn capture(d: &epub::ImageUse, budget: &mut Budget) -> Result<Self, CodecError> {
        Ok(Self {
            resource_path: budget.optional_text(&d.resource_path)?,
            manifest_resource_index: d.manifest_resource_index.map(usize64).transpose()?,
            occurrence: usize64(d.occurrence)?,
            external: d.external,
            line: d.line,
            column: d.column,
        })
    }
    fn restore(self) -> Result<epub::ImageUse, CodecError> {
        Ok(epub::ImageUse {
            resource_path: self.resource_path,
            manifest_resource_index: self.manifest_resource_index.map(width).transpose()?,
            occurrence: width(self.occurrence)?,
            external: self.external,
            line: self.line,
            column: self.column,
        })
    }
    fn capture_all(
        values: &[epub::ImageUse],
        budget: &mut Budget,
    ) -> Result<Vec<Self>, CodecError> {
        let mut out = budget.vector(values.len())?;
        for v in values {
            out.push(Self::capture(v, budget)?);
        }
        Ok(out)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub id: String,
    pub path: Option<String>,
    pub media_type: String,
    pub properties: Vec<String>,
    pub fallback: Option<String>,
    pub media_overlay: Option<String>,
    pub available: bool,
    pub encryption: Encryption,
    pub image_uses: Vec<ImageUse>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpineItem {
    pub idref: String,
    pub resource_index: Option<u64>,
    pub reading_order: u64,
    pub linear: bool,
    pub kind: SpineKind,
    pub image_uses: Vec<ImageUse>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigationTarget {
    pub resource_path: Option<String>,
    pub fragment: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigationNode {
    pub parent_index: Option<u32>,
    pub sibling_index: u32,
    pub label: String,
    pub target: Option<NavigationTarget>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Navigation {
    pub source: NavigationSource,
    pub resource_path: String,
    pub items: Vec<NavigationNode>,
}

impl Navigation {
    fn capture(value: &epub::NavigationTree, budget: &mut Budget) -> Result<Self, CodecError> {
        fn count(items: &[epub::NavigationItem], depth: usize) -> Result<usize, CodecError> {
            if items.is_empty() {
                return Ok(0);
            }
            if depth >= 256 {
                return Err(CodecError::ResourceLimit);
            }
            items.iter().try_fold(0usize, |n, item| {
                n.checked_add(1)
                    .and_then(|n| {
                        count(&item.children, depth + 1)
                            .ok()
                            .and_then(|m| n.checked_add(m))
                    })
                    .ok_or(CodecError::ResourceLimit)
            })
        }
        fn flatten(
            items: &[epub::NavigationItem],
            parent: Option<u32>,
            out: &mut Vec<NavigationNode>,
            budget: &mut Budget,
        ) -> Result<(), CodecError> {
            for (sibling, item) in items.iter().enumerate() {
                let current = u32::try_from(out.len()).map_err(|_| CodecError::ResourceLimit)?;
                let target = item
                    .target
                    .as_ref()
                    .map(|t| {
                        Ok(NavigationTarget {
                            resource_path: budget.optional_text(&t.resource_path)?,
                            fragment: budget.optional_text(&t.fragment)?,
                        })
                    })
                    .transpose()?;
                out.push(NavigationNode {
                    parent_index: parent,
                    sibling_index: u32::try_from(sibling).map_err(|_| CodecError::ResourceLimit)?,
                    label: budget.text(&item.label)?,
                    target,
                });
                flatten(&item.children, Some(current), out, budget)?;
            }
            Ok(())
        }
        let mut items = budget.vector(count(&value.items, 0)?)?;
        flatten(&value.items, None, &mut items, budget)?;
        Ok(Self {
            source: value.source.into(),
            resource_path: budget.text(&value.resource_path)?,
            items,
        })
    }
    fn restore(self) -> Result<epub::NavigationTree, CodecError> {
        validate_parents(self.items.iter().map(|n| (n.parent_index, n.sibling_index)))?;
        let mut children: Vec<Vec<epub::NavigationItem>> =
            (0..=self.items.len()).map(|_| Vec::new()).collect();
        for (index, node) in self.items.into_iter().enumerate().rev() {
            let branch = std::mem::take(&mut children[index + 1]);
            let mut branch = branch;
            branch.reverse();
            let item = epub::NavigationItem {
                label: node.label,
                target: node.target.map(|t| epub::NavigationTarget {
                    resource_path: t.resource_path,
                    fragment: t.fragment,
                }),
                children: branch,
            };
            children[node.parent_index.map_or(0, |p| p as usize + 1)].push(item);
        }
        let mut items = std::mem::take(&mut children[0]);
        items.reverse();
        Ok(epub::NavigationTree {
            source: self.source.into(),
            resource_path: self.resource_path,
            items,
        })
    }
}

/// Preorder, parent-before-child and consecutive sibling indices prevent
/// cycles, disconnected parents and re-entering a closed subtree.
pub(super) fn validate_parents(
    items: impl IntoIterator<Item = (Option<u32>, u32)>,
) -> Result<(), CodecError> {
    let mut stack = Vec::<(u32, u32)>::new();
    let mut roots = 0u32;
    for (index, (parent, sibling)) in items.into_iter().enumerate() {
        if let Some(parent) = parent {
            while stack.last().is_some_and(|(node, _)| *node != parent) {
                stack.pop();
            }
            let (_, next) = stack.last_mut().ok_or(CodecError::InvalidRecord)?;
            if sibling != *next {
                return Err(CodecError::InvalidRecord);
            }
            *next = next.checked_add(1).ok_or(CodecError::ResourceLimit)?;
        } else {
            stack.clear();
            if sibling != roots {
                return Err(CodecError::InvalidRecord);
            }
            roots = roots.checked_add(1).ok_or(CodecError::ResourceLimit)?;
        }
        if stack.len() >= 256 {
            return Err(CodecError::ResourceLimit);
        }
        stack.push((
            u32::try_from(index).map_err(|_| CodecError::ResourceLimit)?,
            0,
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredInventory {
    pub package_path: Option<String>,
    pub package_version: Option<String>,
    pub metadata: Metadata,
    pub resources: Vec<Resource>,
    pub spine: Vec<SpineItem>,
    pub navigation: Vec<Navigation>,
    pub primary_navigation: Option<u64>,
    pub coverage: Coverage,
    pub diagnostics: Vec<InventoryDiagnostic>,
}
impl StoredInventory {
    pub fn capture(
        value: &epub::EpubInventory,
        reserved_working_bytes: usize,
    ) -> Result<Self, CodecError> {
        Self::capture_with_budget(value, &mut Budget::new(reserved_working_bytes)?)
    }
    pub(super) fn capture_with_budget(
        value: &epub::EpubInventory,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        let mut values = budget.vector(value.metadata.values.len())?;
        for v in &value.metadata.values {
            values.push(MetadataValue {
                namespace: budget.optional_text(&v.namespace)?,
                name: budget.text(&v.name)?,
                value: budget.text(&v.value)?,
                id: budget.optional_text(&v.id)?,
                property: budget.optional_text(&v.property)?,
                refines: budget.optional_text(&v.refines)?,
                scheme: budget.optional_text(&v.scheme)?,
            });
        }
        let metadata = Metadata {
            values,
            unique_identifier_id: budget.optional_text(&value.metadata.unique_identifier_id)?,
        };
        let mut resources = budget.vector(value.resources.len())?;
        for r in &value.resources {
            let mut properties = budget.vector(r.properties.len())?;
            for p in &r.properties {
                properties.push(budget.text(p)?);
            }
            resources.push(Resource {
                id: budget.text(&r.id)?,
                path: budget.optional_text(&r.path)?,
                media_type: budget.text(&r.media_type)?,
                properties,
                fallback: budget.optional_text(&r.fallback)?,
                media_overlay: budget.optional_text(&r.media_overlay)?,
                available: r.available,
                encryption: r.encryption.into(),
                image_uses: ImageUse::capture_all(&r.image_uses, budget)?,
            });
        }
        let mut spine = budget.vector(value.spine.len())?;
        for s in &value.spine {
            spine.push(SpineItem {
                idref: budget.text(&s.idref)?,
                resource_index: s.resource_index.map(usize64).transpose()?,
                reading_order: usize64(s.reading_order)?,
                linear: s.linear,
                kind: s.kind.into(),
                image_uses: ImageUse::capture_all(&s.image_uses, budget)?,
            });
        }
        let mut navigation = budget.vector(value.navigation.len())?;
        for n in &value.navigation {
            navigation.push(Navigation::capture(n, budget)?);
        }
        let mut diagnostics = budget.vector(value.diagnostics.len())?;
        for d in &value.diagnostics {
            diagnostics.push(InventoryDiagnostic::capture(d, budget)?);
        }
        Ok(Self {
            package_path: budget.optional_text(&value.package_path)?,
            package_version: budget.optional_text(&value.package_version)?,
            metadata,
            resources,
            spine,
            navigation,
            primary_navigation: value.primary_navigation.map(usize64).transpose()?,
            coverage: value.coverage.into(),
            diagnostics,
        })
    }
    /// Consume the DTO to rebuild the exact public inventory contract. Its data
    /// remains untrusted until the original EPUB and owning snapshot are checked.
    pub fn into_inventory(self) -> Result<epub::EpubInventory, CodecError> {
        if self.package_path.as_deref().is_some_and(|p| !safe_path(p))
            || self
                .primary_navigation
                .is_some_and(|i| i >= self.navigation.len() as u64)
        {
            return Err(CodecError::InvalidRecord);
        }
        let mut resources = Vec::with_capacity(self.resources.len());
        for r in self.resources {
            if r.path.as_deref().is_some_and(|p| !safe_path(p)) {
                return Err(CodecError::InvalidRecord);
            }
            resources.push(epub::Resource {
                id: r.id,
                path: r.path,
                media_type: r.media_type,
                properties: r.properties,
                fallback: r.fallback,
                media_overlay: r.media_overlay,
                available: r.available,
                encryption: r.encryption.into(),
                image_uses: r
                    .image_uses
                    .into_iter()
                    .map(ImageUse::restore)
                    .collect::<Result<_, _>>()?,
            });
        }
        let mut spine = Vec::with_capacity(self.spine.len());
        for s in self.spine {
            if s.resource_index
                .is_some_and(|i| i >= resources.len() as u64)
            {
                return Err(CodecError::InvalidRecord);
            }
            spine.push(epub::SpineItem {
                idref: s.idref,
                resource_index: s.resource_index.map(width).transpose()?,
                reading_order: width(s.reading_order)?,
                linear: s.linear,
                kind: s.kind.into(),
                image_uses: s
                    .image_uses
                    .into_iter()
                    .map(ImageUse::restore)
                    .collect::<Result<_, _>>()?,
            });
        }
        Ok(epub::EpubInventory {
            package_path: self.package_path,
            package_version: self.package_version,
            metadata: epub::Metadata {
                values: self
                    .metadata
                    .values
                    .into_iter()
                    .map(|v| epub::MetadataValue {
                        namespace: v.namespace,
                        name: v.name,
                        value: v.value,
                        id: v.id,
                        property: v.property,
                        refines: v.refines,
                        scheme: v.scheme,
                    })
                    .collect(),
                unique_identifier_id: self.metadata.unique_identifier_id,
            },
            resources,
            spine,
            navigation: self
                .navigation
                .into_iter()
                .map(Navigation::restore)
                .collect::<Result<_, _>>()?,
            primary_navigation: self.primary_navigation.map(width).transpose()?,
            coverage: self.coverage.into(),
            diagnostics: self
                .diagnostics
                .into_iter()
                .map(InventoryDiagnostic::restore)
                .collect(),
        })
    }
}

// Retained allocation accounting.
crate::import_codec::memory::heap_struct!(InventoryDiagnostic {
    code,
    severity,
    resource_path,
    line,
    column,
    related_resource
});
crate::import_codec::memory::heap_struct!(MetadataValue {
    namespace,
    name,
    value,
    id,
    property,
    refines,
    scheme
});
crate::import_codec::memory::heap_struct!(Metadata {
    values,
    unique_identifier_id
});
crate::import_codec::memory::heap_struct!(ImageUse {
    resource_path,
    manifest_resource_index,
    occurrence,
    external,
    line,
    column
});
crate::import_codec::memory::heap_struct!(Resource {
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
crate::import_codec::memory::heap_struct!(SpineItem {
    idref,
    resource_index,
    reading_order,
    linear,
    kind,
    image_uses
});
crate::import_codec::memory::heap_struct!(NavigationTarget {
    resource_path,
    fragment
});
crate::import_codec::memory::heap_struct!(NavigationNode {
    parent_index,
    sibling_index,
    label,
    target
});
crate::import_codec::memory::heap_struct!(Navigation {
    source,
    resource_path,
    items
});
crate::import_codec::memory::heap_struct!(StoredInventory {
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
