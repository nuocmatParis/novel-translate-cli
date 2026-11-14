use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek};

use roxmltree::{Document, Node};

use crate::archive::{EntryId, EpubZip};
use crate::container::parse_container;
use crate::encryption::parse_encryption;
use crate::navigation::{RawNavigationItem, parse_navigation};
use crate::package::{ManifestItem, PackageData, SpineRef, parse_package, resolve_href};
use crate::xml::{XmlFailure, XmlFailureKind, node_position, with_xml_document};

const CONTAINER_PATH: &str = "META-INF/container.xml";
const ENCRYPTION_PATH: &str = "META-INF/encryption.xml";
const PACKAGE_MEDIA_TYPE: &str = "application/oebps-package+xml";
const XHTML_MEDIA_TYPE: &str = "application/xhtml+xml";
const NCX_MEDIA_TYPE: &str = "application/x-dtbncx+xml";
const SVG_MEDIA_TYPE: &str = "image/svg+xml";
const DC_NAMESPACE: &str = "http://purl.org/dc/elements/1.1/";
const XHTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InventoryCoverage {
    Complete,
    Partial,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Incomplete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticCode {
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    /// An archive-relative validated resource path, never an absolute path or URL.
    pub resource_path: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    /// An internal manifest id or archive-relative resource path when useful.
    pub related_resource: Option<String>,
}

fn sanitize_related_resource(value: String) -> Option<String> {
    if value.is_empty()
        || value.len() > 4096
        || value.chars().any(char::is_control)
        || value
            .chars()
            .any(|character| matches!(character, '?' | '#' | '\\'))
        || value.starts_with('/')
    {
        return None;
    }

    let is_relative_path = !value.contains(':')
        && value.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment.chars().all(|character| {
                    character.is_alphanumeric()
                        || matches!(
                            character,
                            ' ' | '-'
                                | '_'
                                | '.'
                                | '~'
                                | '+'
                                | '('
                                | ')'
                                | '\''
                                | '$'
                                | '@'
                                | '&'
                                | ','
                                | '!'
                        )
                })
        });
    let mut characters = value.chars();
    let first = characters.next().unwrap_or_default();
    let is_safe_id = (first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        });
    if is_relative_path || is_safe_id {
        Some(value)
    } else {
        None
    }
}

impl Diagnostic {
    pub(crate) fn new(
        code: DiagnosticCode,
        severity: DiagnosticSeverity,
        resource_path: Option<String>,
        line: Option<u32>,
        column: Option<u32>,
        related_resource: Option<String>,
    ) -> Self {
        Self {
            code,
            severity,
            resource_path,
            line,
            column,
            related_resource: related_resource.and_then(sanitize_related_resource),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataValue {
    pub namespace: Option<String>,
    pub name: String,
    pub value: String,
    pub id: Option<String>,
    pub property: Option<String>,
    pub refines: Option<String>,
    pub scheme: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Metadata {
    pub values: Vec<MetadataValue>,
    pub unique_identifier_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceEncryption {
    NotEncrypted,
    IdpfFontObfuscation,
    AdobeFontObfuscation,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resource {
    pub id: String,
    pub path: Option<String>,
    pub media_type: String,
    pub properties: Vec<String>,
    pub fallback: Option<String>,
    pub media_overlay: Option<String>,
    pub available: bool,
    pub encryption: ResourceEncryption,
    pub image_uses: Vec<ImageUse>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpineContentKind {
    ReflowableDocument,
    FixedLayoutDocument,
    ImagePage,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpineItem {
    pub idref: String,
    pub resource_index: Option<usize>,
    pub reading_order: usize,
    pub linear: bool,
    pub kind: SpineContentKind,
    pub image_uses: Vec<ImageUse>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageUse {
    pub resource_path: Option<String>,
    pub manifest_resource_index: Option<usize>,
    pub occurrence: usize,
    pub external: bool,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationSource {
    Epub3Nav,
    Epub2Ncx,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationTarget {
    pub resource_path: Option<String>,
    pub fragment: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationItem {
    pub label: String,
    pub target: Option<NavigationTarget>,
    pub children: Vec<NavigationItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationTree {
    pub source: NavigationSource,
    pub resource_path: String,
    pub items: Vec<NavigationItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpubInventory {
    pub package_path: Option<String>,
    pub package_version: Option<String>,
    pub metadata: Metadata,
    pub resources: Vec<Resource>,
    pub spine: Vec<SpineItem>,
    pub navigation: Vec<NavigationTree>,
    pub primary_navigation: Option<usize>,
    pub coverage: InventoryCoverage,
    pub diagnostics: Vec<Diagnostic>,
}

impl Default for EpubInventory {
    fn default() -> Self {
        Self {
            package_path: None,
            package_version: None,
            metadata: Metadata::default(),
            resources: Vec::new(),
            spine: Vec::new(),
            navigation: Vec::new(),
            primary_navigation: None,
            coverage: InventoryCoverage::Complete,
            diagnostics: Vec::new(),
        }
    }
}

impl EpubInventory {
    fn record(&mut self, diagnostic: Diagnostic, affects_coverage: bool) {
        if affects_coverage {
            self.coverage = InventoryCoverage::Partial;
        }
        self.diagnostics.push(diagnostic);
    }

    #[allow(clippy::too_many_arguments)]
    fn issue(
        &mut self,
        code: DiagnosticCode,
        severity: DiagnosticSeverity,
        path: Option<String>,
        line: Option<u32>,
        column: Option<u32>,
        related: Option<String>,
        affects_coverage: bool,
    ) {
        self.record(
            Diagnostic::new(code, severity, path, line, column, related),
            affects_coverage,
        );
    }
}

/// Read the OCF container and OPF inventory through the bounded T04 archive API.
/// External URI references are represented only by diagnostics and are never opened.
pub fn inspect<R: Read + Seek>(archive: &mut EpubZip<R>) -> EpubInventory {
    let mut inventory = EpubInventory::default();
    let container_bytes = match read_entry(archive, CONTAINER_PATH) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => {
            inventory.issue(
                DiagnosticCode::ContainerMissing,
                DiagnosticSeverity::Incomplete,
                Some(CONTAINER_PATH.to_owned()),
                None,
                None,
                None,
                true,
            );
            return inventory;
        }
        Err(()) => {
            archive_failure(&mut inventory, CONTAINER_PATH);
            return inventory;
        }
    };
    let rootfiles = match parse_container(&container_bytes) {
        Ok(rootfiles) => rootfiles,
        Err(failure) => {
            xml_failure(
                &mut inventory,
                CONTAINER_PATH,
                failure,
                DiagnosticCode::ContainerInvalidXml,
            );
            return inventory;
        }
    };
    if rootfiles.is_empty() {
        inventory.issue(
            DiagnosticCode::ContainerRootfileMissing,
            DiagnosticSeverity::Incomplete,
            Some(CONTAINER_PATH.to_owned()),
            None,
            None,
            None,
            true,
        );
        return inventory;
    }

    let mut package_path = None;
    for rootfile in &rootfiles {
        if rootfile.media_type.as_deref() != Some(PACKAGE_MEDIA_TYPE) {
            inventory.issue(
                DiagnosticCode::ContainerRootfileInvalid,
                DiagnosticSeverity::Warning,
                Some(CONTAINER_PATH.to_owned()),
                None,
                None,
                None,
                true,
            );
            continue;
        }
        match resolve_href("", &rootfile.full_path) {
            Ok(target) if !target.external => {
                if let Some(path) = target.path {
                    package_path = Some(path);
                    break;
                }
            }
            _ => inventory.issue(
                DiagnosticCode::ContainerRootfileInvalid,
                DiagnosticSeverity::Incomplete,
                Some(CONTAINER_PATH.to_owned()),
                None,
                None,
                None,
                true,
            ),
        }
    }
    let Some(package_path) = package_path else {
        inventory.issue(
            DiagnosticCode::ContainerRootfileMissing,
            DiagnosticSeverity::Incomplete,
            Some(CONTAINER_PATH.to_owned()),
            None,
            None,
            None,
            true,
        );
        return inventory;
    };
    inventory.package_path = Some(package_path.clone());

    let package_bytes = match read_entry(archive, &package_path) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => {
            inventory.issue(
                DiagnosticCode::PackageMissing,
                DiagnosticSeverity::Incomplete,
                Some(package_path),
                None,
                None,
                None,
                true,
            );
            return inventory;
        }
        Err(()) => {
            archive_failure(&mut inventory, &package_path);
            return inventory;
        }
    };
    let package = match parse_package(&package_bytes, &package_path) {
        Ok(package) => package,
        Err(failure) => {
            xml_failure(
                &mut inventory,
                &package_path,
                failure,
                DiagnosticCode::PackageInvalidXml,
            );
            return inventory;
        }
    };
    inventory.package_version = package.version.clone();
    inventory.metadata = Metadata {
        values: package.metadata.clone(),
        unique_identifier_id: package.unique_identifier.clone(),
    };
    for diagnostic in package.diagnostics.iter().cloned() {
        inventory.record(diagnostic, true);
    }
    if package.version.is_none() {
        inventory.issue(
            DiagnosticCode::PackageVersionMissing,
            DiagnosticSeverity::Warning,
            Some(package_path.clone()),
            None,
            None,
            None,
            true,
        );
    } else if !matches!(package.version.as_deref(), Some("2.0" | "3.0")) {
        inventory.issue(
            DiagnosticCode::PackageVersionUnsupported,
            DiagnosticSeverity::Warning,
            Some(package_path.clone()),
            None,
            None,
            None,
            true,
        );
    }
    check_metadata(&mut inventory, &package);

    let mut resources = package.resources.clone();
    let mut id_to_index = HashMap::<String, usize>::new();
    let mut paths_to_indices = HashMap::<String, Vec<usize>>::new();
    for (index, source) in resources.iter_mut().enumerate() {
        if !source.id.is_empty() {
            id_to_index.entry(source.id.clone()).or_insert(index);
        }
        match resolve_href(&package.path, &source.href) {
            Ok(target) if target.external => {
                inventory.issue(
                    DiagnosticCode::ExternalResourceNotRead,
                    DiagnosticSeverity::Warning,
                    Some(package.path.clone()),
                    None,
                    None,
                    Some(source.id.clone()),
                    true,
                );
            }
            Ok(target) => {
                source.path = target.path;
                if let Some(path) = source.path.as_deref() {
                    match entry_exists(archive, path) {
                        Ok(true) => source.available = true,
                        Ok(false) => inventory.issue(
                            DiagnosticCode::ResourceMissing,
                            DiagnosticSeverity::Incomplete,
                            Some(package.path.clone()),
                            None,
                            None,
                            Some(path.to_owned()),
                            true,
                        ),
                        Err(()) => {
                            archive_failure(&mut inventory, path);
                            return inventory;
                        }
                    }
                    paths_to_indices
                        .entry(path.to_owned())
                        .or_default()
                        .push(index);
                }
            }
            Err(_) => {
                inventory.issue(
                    DiagnosticCode::ResourceHrefInvalid,
                    DiagnosticSeverity::Incomplete,
                    Some(package.path.clone()),
                    None,
                    None,
                    Some(source.id.clone()),
                    true,
                );
            }
        }
    }
    let mut duplicate_paths = paths_to_indices
        .iter()
        .filter(|(_, indices)| indices.len() > 1)
        .collect::<Vec<_>>();
    duplicate_paths.sort_by_key(|(path, _)| path.as_str());
    for (path, indices) in duplicate_paths {
        if indices.len() > 1 {
            inventory.issue(
                DiagnosticCode::ResourcePathAmbiguous,
                DiagnosticSeverity::Incomplete,
                Some(package.path.clone()),
                None,
                None,
                Some(path.clone()),
                true,
            );
        }
    }
    inventory.resources = public_resources(&resources);

    let layout_fixed = package.layout.as_deref() == Some("pre-paginated");
    let has_usable_spine_item = package.spine.iter().any(|spine| {
        id_to_index
            .get(&spine.idref)
            .and_then(|index| resources.get(*index))
            .is_some_and(|resource| resource.available)
    });
    if !has_usable_spine_item {
        inventory.issue(
            DiagnosticCode::SpineEmpty,
            DiagnosticSeverity::Incomplete,
            Some(package.path.clone()),
            None,
            None,
            None,
            true,
        );
    }
    for spine in &package.spine {
        let item = build_spine_item(
            spine,
            &id_to_index,
            &resources,
            layout_fixed,
            &package.path,
            &mut inventory,
        );
        inventory.spine.push(item);
    }
    diagnose_manifest_properties(&mut inventory, &package, &resources);

    if let Some(toc_id) = package.toc_id.as_deref() {
        match package
            .resources
            .iter()
            .find(|resource| resource.id == toc_id)
        {
            Some(resource) if resource.media_type == NCX_MEDIA_TYPE => {}
            Some(_) => inventory.issue(
                DiagnosticCode::SpineTocReferenceInvalid,
                DiagnosticSeverity::Warning,
                Some(package.path.clone()),
                None,
                None,
                Some(toc_id.to_owned()),
                true,
            ),
            None => inventory.issue(
                DiagnosticCode::SpineTocReferenceInvalid,
                DiagnosticSeverity::Incomplete,
                Some(package.path.clone()),
                None,
                None,
                Some(toc_id.to_owned()),
                true,
            ),
        }
    }
    let navigation_resources = select_navigation_resources(&package);
    if navigation_resources.is_empty() {
        inventory.issue(
            DiagnosticCode::NavigationMissing,
            DiagnosticSeverity::Warning,
            Some(package.path.clone()),
            None,
            None,
            None,
            true,
        );
    }
    let mut navigation_anchor_cache = HashMap::new();
    for (resource_index, source) in &navigation_resources {
        let resource = &resources[*resource_index];
        let Some(path) = resource.path.as_deref() else {
            continue;
        };
        if !resource.available {
            continue;
        }
        let bytes = match read_entry(archive, path) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                inventory.issue(
                    DiagnosticCode::NavigationTargetMissing,
                    DiagnosticSeverity::Incomplete,
                    Some(path.to_owned()),
                    None,
                    None,
                    None,
                    true,
                );
                continue;
            }
            Err(()) => {
                archive_failure(&mut inventory, path);
                return inventory;
            }
        };
        match parse_navigation(&bytes, *source) {
            Ok(parsed) if parsed.has_toc && !parsed.items.is_empty() => {
                let items = resolve_navigation_items(
                    &parsed.items,
                    path,
                    archive,
                    &paths_to_indices,
                    &resources,
                    &mut navigation_anchor_cache,
                    &mut inventory,
                );
                inventory.navigation.push(NavigationTree {
                    source: *source,
                    resource_path: path.to_owned(),
                    items,
                });
            }
            Ok(_) => inventory.issue(
                DiagnosticCode::NavigationStructureUnsupported,
                DiagnosticSeverity::Warning,
                Some(path.to_owned()),
                None,
                None,
                None,
                true,
            ),
            Err(failure) => xml_failure(
                &mut inventory,
                path,
                failure,
                DiagnosticCode::NavigationInvalidXml,
            ),
        }
        if archive_failed(&inventory) {
            return inventory;
        }
    }
    if inventory.navigation.is_empty() && !navigation_resources.is_empty() {
        inventory.issue(
            DiagnosticCode::NavigationMissing,
            DiagnosticSeverity::Warning,
            Some(package.path.clone()),
            None,
            None,
            None,
            true,
        );
    }
    inventory.primary_navigation = inventory
        .navigation
        .iter()
        .position(|tree| tree.source == NavigationSource::Epub3Nav)
        .or_else(|| (!inventory.navigation.is_empty()).then_some(0));
    compare_navigation_sources(&mut inventory, &package.path);

    inspect_encryption(archive, &mut inventory, &mut resources, &paths_to_indices);
    if archive_failed(&inventory) {
        return inventory;
    }
    inventory.resources = public_resources(&resources);
    inspect_spine_markup(archive, &mut inventory);
    inventory
}

fn public_resources(resources: &[ManifestItem]) -> Vec<Resource> {
    resources
        .iter()
        .map(|resource| Resource {
            id: resource.id.clone(),
            path: resource.path.clone(),
            media_type: resource.media_type.clone(),
            properties: resource.properties.clone(),
            fallback: resource.fallback.clone(),
            media_overlay: resource.media_overlay.clone(),
            available: resource.available,
            encryption: resource.encryption,
            image_uses: Vec::new(),
        })
        .collect()
}

fn check_metadata(inventory: &mut EpubInventory, package: &PackageData) {
    let has_dc_value = |name: &str| {
        package.metadata.iter().any(|value| {
            value.namespace.as_deref() == Some(DC_NAMESPACE)
                && value.name == name
                && !value.value.is_empty()
        })
    };
    let has_title = has_dc_value("title");
    let has_language = has_dc_value("language");
    let has_creator = has_dc_value("creator");
    if !has_title || !has_language || !has_creator {
        inventory.issue(
            DiagnosticCode::MetadataMissing,
            DiagnosticSeverity::Warning,
            Some(package.path.clone()),
            None,
            None,
            None,
            false,
        );
    }
    let Some(identifier_id) = package.unique_identifier.as_deref() else {
        inventory.issue(
            DiagnosticCode::UniqueIdentifierMissing,
            DiagnosticSeverity::Warning,
            Some(package.path.clone()),
            None,
            None,
            None,
            true,
        );
        return;
    };
    let resolved = package.metadata.iter().any(|value| {
        value.namespace.as_deref() == Some(DC_NAMESPACE)
            && value.name == "identifier"
            && value.id.as_deref() == Some(identifier_id)
    });
    if !resolved {
        inventory.issue(
            DiagnosticCode::UniqueIdentifierUnresolved,
            DiagnosticSeverity::Incomplete,
            Some(package.path.clone()),
            None,
            None,
            Some(identifier_id.to_owned()),
            true,
        );
    }
}

fn build_spine_item(
    source: &SpineRef,
    id_to_index: &HashMap<String, usize>,
    resources: &[ManifestItem],
    layout_fixed: bool,
    package_path: &str,
    inventory: &mut EpubInventory,
) -> SpineItem {
    let resource_index = id_to_index.get(&source.idref).copied();
    let fixed_layout = match source.layout_override.as_deref() {
        Some("pre-paginated") => true,
        Some("reflowable") => false,
        _ => layout_fixed,
    };
    let kind = match resource_index.and_then(|index| resources.get(index)) {
        None => {
            inventory.issue(
                DiagnosticCode::SpineResourceMissing,
                DiagnosticSeverity::Incomplete,
                Some(package_path.to_owned()),
                Some(source.line),
                Some(source.column),
                Some(source.idref.clone()),
                true,
            );
            SpineContentKind::Unsupported
        }
        Some(resource) if !resource.available => SpineContentKind::Unsupported,
        Some(resource) if resource.media_type == XHTML_MEDIA_TYPE && fixed_layout => {
            SpineContentKind::FixedLayoutDocument
        }
        Some(resource) if resource.media_type == SVG_MEDIA_TYPE && fixed_layout => {
            SpineContentKind::FixedLayoutDocument
        }
        Some(resource)
            if resource.media_type.starts_with("image/")
                || resource.media_type == SVG_MEDIA_TYPE =>
        {
            SpineContentKind::ImagePage
        }
        Some(resource) if resource.media_type == XHTML_MEDIA_TYPE => {
            SpineContentKind::ReflowableDocument
        }
        Some(_) => {
            inventory.issue(
                DiagnosticCode::UnsupportedSpineMediaType,
                DiagnosticSeverity::Warning,
                Some(package_path.to_owned()),
                Some(source.line),
                Some(source.column),
                Some(source.idref.clone()),
                true,
            );
            SpineContentKind::Unsupported
        }
    };
    SpineItem {
        idref: source.idref.clone(),
        resource_index,
        reading_order: source.order,
        linear: source.linear,
        kind,
        image_uses: Vec::new(),
    }
}

fn diagnose_manifest_properties(
    inventory: &mut EpubInventory,
    package: &PackageData,
    resources: &[ManifestItem],
) {
    for resource in resources {
        if resource
            .properties
            .iter()
            .any(|property| property == "scripted")
        {
            inventory.issue(
                DiagnosticCode::ScriptedContentUnsupported,
                DiagnosticSeverity::Warning,
                resource.path.clone().or_else(|| Some(package.path.clone())),
                None,
                None,
                Some(resource.id.clone()),
                true,
            );
        }
        if resource
            .properties
            .iter()
            .any(|property| property == "remote-resources")
        {
            inventory.issue(
                DiagnosticCode::RemoteResourcesUnsupported,
                DiagnosticSeverity::Warning,
                resource.path.clone().or_else(|| Some(package.path.clone())),
                None,
                None,
                Some(resource.id.clone()),
                true,
            );
        }
    }
}

fn select_navigation_resources(package: &PackageData) -> Vec<(usize, NavigationSource)> {
    let mut selected = Vec::new();
    for (index, resource) in package.resources.iter().enumerate() {
        if resource.properties.iter().any(|property| property == "nav") {
            selected.push((index, NavigationSource::Epub3Nav));
        }
    }
    let ncx_index = match package.toc_id.as_deref() {
        Some(id) => package
            .resources
            .iter()
            .position(|resource| resource.id == id),
        None => package
            .resources
            .iter()
            .position(|resource| resource.media_type == NCX_MEDIA_TYPE),
    };
    if let Some(index) = ncx_index
        && !selected.iter().any(|(candidate, _)| *candidate == index)
    {
        selected.push((index, NavigationSource::Epub2Ncx));
    }
    selected
}

fn resolve_navigation_items<R: Read + Seek>(
    source_items: &[RawNavigationItem],
    nav_path: &str,
    archive: &mut EpubZip<R>,
    paths_to_indices: &HashMap<String, Vec<usize>>,
    resources: &[ManifestItem],
    anchor_cache: &mut HashMap<String, AnchorScan>,
    inventory: &mut EpubInventory,
) -> Vec<NavigationItem> {
    source_items
        .iter()
        .map(|item| {
            let target = item.target.as_ref().map(|raw| {
                let (line, column) = (Some(raw.line), Some(raw.column));
                let resolved = match resolve_href(nav_path, &raw.href) {
                    Ok(target) if target.external => {
                        inventory.issue(
                            DiagnosticCode::ExternalReferenceNotRead,
                            DiagnosticSeverity::Warning,
                            Some(nav_path.to_owned()),
                            line,
                            column,
                            None,
                            true,
                        );
                        return NavigationTarget {
                            resource_path: None,
                            fragment: None,
                        };
                    }
                    Ok(target) => target,
                    Err(_) => {
                        inventory.issue(
                            DiagnosticCode::ResourceHrefInvalid,
                            DiagnosticSeverity::Incomplete,
                            Some(nav_path.to_owned()),
                            line,
                            column,
                            None,
                            true,
                        );
                        return NavigationTarget {
                            resource_path: None,
                            fragment: None,
                        };
                    }
                };
                let Some(path) = resolved.path else {
                    return NavigationTarget {
                        resource_path: None,
                        fragment: resolved.fragment,
                    };
                };
                let target_exists = match entry_exists(archive, &path) {
                    Ok(true) => true,
                    Ok(false) => {
                        inventory.issue(
                            DiagnosticCode::NavigationTargetMissing,
                            DiagnosticSeverity::Incomplete,
                            Some(nav_path.to_owned()),
                            line,
                            column,
                            Some(path.clone()),
                            true,
                        );
                        false
                    }
                    Err(()) => {
                        inventory.issue(
                            DiagnosticCode::ArchiveEntryUnreadable,
                            DiagnosticSeverity::Incomplete,
                            Some(path.clone()),
                            line,
                            column,
                            None,
                            true,
                        );
                        false
                    }
                };
                if target_exists {
                    let listed_xhtml = match paths_to_indices.get(&path).map(Vec::as_slice) {
                        None | Some([]) => {
                            inventory.issue(
                                DiagnosticCode::NavigationTargetUnlisted,
                                DiagnosticSeverity::Warning,
                                Some(nav_path.to_owned()),
                                line,
                                column,
                                Some(path.clone()),
                                true,
                            );
                            false
                        }
                        Some(indices) if indices.len() > 1 => {
                            inventory.issue(
                                DiagnosticCode::NavigationTargetAmbiguous,
                                DiagnosticSeverity::Incomplete,
                                Some(nav_path.to_owned()),
                                line,
                                column,
                                Some(path.clone()),
                                true,
                            );
                            false
                        }
                        Some([index]) => resources.get(*index).is_some_and(|resource| {
                            matches!(
                                resource.media_type.as_str(),
                                XHTML_MEDIA_TYPE | SVG_MEDIA_TYPE
                            )
                        }),
                        Some(_) => false,
                    };
                    if listed_xhtml
                        && let Some(fragment) =
                            resolved.fragment.as_deref().filter(|f| !f.is_empty())
                    {
                        match fragment_exists(
                            archive,
                            &path,
                            fragment,
                            nav_path,
                            Some((raw.line, raw.column)),
                            anchor_cache,
                            inventory,
                        ) {
                            Some(0) => inventory.issue(
                                DiagnosticCode::NavigationTargetMissing,
                                DiagnosticSeverity::Incomplete,
                                Some(nav_path.to_owned()),
                                line,
                                column,
                                Some(path.clone()),
                                true,
                            ),
                            Some(1) | None => {}
                            Some(_) => inventory.issue(
                                DiagnosticCode::NavigationTargetAmbiguous,
                                DiagnosticSeverity::Incomplete,
                                Some(nav_path.to_owned()),
                                line,
                                column,
                                Some(path.clone()),
                                true,
                            ),
                        }
                    }
                }
                NavigationTarget {
                    resource_path: Some(path),
                    fragment: resolved.fragment,
                }
            });
            NavigationItem {
                label: item.label.clone(),
                target,
                children: resolve_navigation_items(
                    &item.children,
                    nav_path,
                    archive,
                    paths_to_indices,
                    resources,
                    anchor_cache,
                    inventory,
                ),
            }
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum AnchorScan {
    Parsed(HashMap<String, usize>),
    Unavailable,
}

fn fragment_exists<R: Read + Seek>(
    archive: &mut EpubZip<R>,
    path: &str,
    fragment: &str,
    nav_path: &str,
    position: Option<(u32, u32)>,
    cache: &mut HashMap<String, AnchorScan>,
    inventory: &mut EpubInventory,
) -> Option<usize> {
    let (line, column) = position
        .map(|(line, column)| (Some(line), Some(column)))
        .unwrap_or((None, None));
    if !cache.contains_key(path) {
        let scan = match read_entry(archive, path) {
            Ok(Some(bytes)) => match with_xml_document(
                &bytes,
                super::xml::DEFAULT_XML_MAX_BYTES,
                super::xml::DEFAULT_XML_MAX_NODES,
                collect_fragment_ids,
            ) {
                Ok(ids) => AnchorScan::Parsed(ids),
                Err(failure) => {
                    xml_failure(inventory, path, failure, DiagnosticCode::XmlMalformed);
                    AnchorScan::Unavailable
                }
            },
            Ok(None) => {
                inventory.issue(
                    DiagnosticCode::NavigationTargetMissing,
                    DiagnosticSeverity::Incomplete,
                    Some(nav_path.to_owned()),
                    line,
                    column,
                    Some(path.to_owned()),
                    true,
                );
                AnchorScan::Unavailable
            }
            Err(()) => {
                archive_failure(inventory, path);
                AnchorScan::Unavailable
            }
        };
        cache.insert(path.to_owned(), scan);
    }
    match cache.get(path) {
        Some(AnchorScan::Parsed(ids)) => Some(ids.get(fragment).copied().unwrap_or(0)),
        Some(AnchorScan::Unavailable) | None => None,
    }
}

fn collect_fragment_ids(document: &Document<'_>) -> HashMap<String, usize> {
    let mut fragments = HashMap::<String, usize>::new();
    for node in document.descendants().filter(Node::is_element) {
        let id = node.attribute("id");
        let legacy_name = (node.tag_name().namespace() == Some(XHTML_NAMESPACE)
            && node.tag_name().name() == "a")
            .then(|| node.attribute("name"))
            .flatten();
        if let Some(id) = id {
            *fragments.entry(id.to_owned()).or_default() += 1;
        }
        if let Some(name) = legacy_name
            && Some(name) != id
        {
            *fragments.entry(name.to_owned()).or_default() += 1;
        }
    }
    fragments
}
fn compare_navigation_sources(inventory: &mut EpubInventory, package_path: &str) {
    if inventory.navigation.len() < 2 {
        return;
    }
    let primary_index = inventory.primary_navigation.unwrap_or(0);
    let primary = navigation_targets(&inventory.navigation[primary_index].items);
    let has_conflict = inventory
        .navigation
        .iter()
        .enumerate()
        .any(|(index, tree)| index != primary_index && navigation_targets(&tree.items) != primary);
    if has_conflict {
        inventory.issue(
            DiagnosticCode::NavigationSourcesConflict,
            DiagnosticSeverity::Warning,
            Some(package_path.to_owned()),
            None,
            None,
            None,
            true,
        );
    }
}

fn navigation_targets(items: &[NavigationItem]) -> Vec<(usize, String, String, Option<String>)> {
    fn visit(
        items: &[NavigationItem],
        depth: usize,
        result: &mut Vec<(usize, String, String, Option<String>)>,
    ) {
        for item in items {
            if let Some(target) = &item.target
                && let Some(path) = &target.resource_path
            {
                result.push((
                    depth,
                    item.label.clone(),
                    path.clone(),
                    target.fragment.clone(),
                ));
            }
            visit(&item.children, depth + 1, result);
        }
    }
    let mut result = Vec::new();
    visit(items, 0, &mut result);
    result
}

fn inspect_encryption<R: Read + Seek>(
    archive: &mut EpubZip<R>,
    inventory: &mut EpubInventory,
    resources: &mut [ManifestItem],
    paths_to_indices: &HashMap<String, Vec<usize>>,
) {
    let bytes = match read_entry(archive, ENCRYPTION_PATH) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return,
        Err(()) => {
            archive_failure(inventory, ENCRYPTION_PATH);
            return;
        }
    };
    let records = match parse_encryption(&bytes) {
        Ok(records) => records,
        Err(failure) => {
            xml_failure(
                inventory,
                ENCRYPTION_PATH,
                failure,
                DiagnosticCode::EncryptionMetadataInvalid,
            );
            return;
        }
    };
    let mut seen = HashSet::new();
    for record in records {
        let (path, line, column) = (
            ENCRYPTION_PATH.to_owned(),
            Some(record.line),
            Some(record.column),
        );
        let target = match resolve_href("", &record.href) {
            Ok(target) if target.external => {
                inventory.issue(
                    DiagnosticCode::ExternalReferenceNotRead,
                    DiagnosticSeverity::Warning,
                    Some(path),
                    line,
                    column,
                    None,
                    true,
                );
                continue;
            }
            Ok(target) => target,
            Err(_) => {
                inventory.issue(
                    DiagnosticCode::ResourceHrefInvalid,
                    DiagnosticSeverity::Incomplete,
                    Some(path),
                    line,
                    column,
                    None,
                    true,
                );
                continue;
            }
        };
        let Some(resource_path) = target.path else {
            continue;
        };
        if !seen.insert(resource_path.clone()) {
            inventory.issue(
                DiagnosticCode::ResourcePathAmbiguous,
                DiagnosticSeverity::Incomplete,
                Some(path),
                line,
                column,
                Some(resource_path),
                true,
            );
            continue;
        }
        match entry_exists(archive, &resource_path) {
            Ok(true) => {}
            Ok(false) => {
                inventory.issue(
                    DiagnosticCode::EncryptedResourceMissing,
                    DiagnosticSeverity::Incomplete,
                    Some(path),
                    line,
                    column,
                    Some(resource_path),
                    true,
                );
                continue;
            }
            Err(()) => {
                archive_failure(inventory, &resource_path);
                return;
            }
        }
        let Some(indices) = paths_to_indices.get(&resource_path) else {
            inventory.issue(
                DiagnosticCode::EncryptedResourceUnlisted,
                DiagnosticSeverity::Warning,
                Some(path),
                line,
                column,
                Some(resource_path),
                true,
            );
            continue;
        };
        if indices.len() != 1 {
            inventory.issue(
                DiagnosticCode::ResourcePathAmbiguous,
                DiagnosticSeverity::Incomplete,
                Some(path),
                line,
                column,
                Some(resource_path),
                true,
            );
            continue;
        }
        let index = indices[0];
        let kind = match (resources.get(index), record.kind) {
            (
                Some(resource),
                ResourceEncryption::IdpfFontObfuscation | ResourceEncryption::AdobeFontObfuscation,
            ) if !is_font_core_media_type(&resource.media_type) => ResourceEncryption::Unsupported,
            (_, kind) => kind,
        };
        if let Some(resource) = resources.get_mut(index) {
            resource.encryption = kind;
        }
        match kind {
            ResourceEncryption::IdpfFontObfuscation | ResourceEncryption::AdobeFontObfuscation => {
                inventory.issue(
                    DiagnosticCode::FontObfuscationDetected,
                    DiagnosticSeverity::Info,
                    Some(resource_path.clone()),
                    line,
                    column,
                    Some(resource_path),
                    false,
                )
            }
            ResourceEncryption::Unsupported => inventory.issue(
                DiagnosticCode::UnsupportedEncryption,
                DiagnosticSeverity::Incomplete,
                Some(path),
                line,
                column,
                Some(resource_path),
                true,
            ),
            ResourceEncryption::NotEncrypted => {}
        }
    }
}

fn is_font_core_media_type(media_type: &str) -> bool {
    matches!(
        media_type,
        "font/ttf"
            | "application/font-sfnt"
            | "font/otf"
            | "application/vnd.ms-opentype"
            | "font/woff"
            | "application/font-woff"
            | "font/woff2"
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RawImageUse {
    href: String,
    line: u32,
    column: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MarkupScan {
    scripted: bool,
    expected_root: bool,
    has_readable_text: bool,
    image_uses: Vec<RawImageUse>,
}

fn inspect_spine_markup<R: Read + Seek>(archive: &mut EpubZip<R>, inventory: &mut EpubInventory) {
    let candidates = inventory
        .resources
        .iter()
        .enumerate()
        .filter(|(_, resource)| {
            resource.available
                && matches!(
                    resource.media_type.as_str(),
                    XHTML_MEDIA_TYPE | SVG_MEDIA_TYPE
                )
        })
        .filter_map(|(resource_index, resource)| {
            resource.path.clone().map(|path| {
                (
                    resource_index,
                    resource.id.clone(),
                    resource.media_type.clone(),
                    path,
                )
            })
        })
        .collect::<Vec<_>>();
    let linear_count = inventory.spine.iter().filter(|item| item.linear).count();
    let mut any_readable_text = false;
    let mut any_images = inventory
        .spine
        .iter()
        .any(|item| item.linear && item.kind == SpineContentKind::ImagePage);
    let mut image_readability = HashMap::<EntryId, bool>::new();

    for (resource_index, id, media_type, path) in candidates {
        let bytes = match read_entry(archive, &path) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                inventory.issue(
                    DiagnosticCode::ResourceMissing,
                    DiagnosticSeverity::Incomplete,
                    Some(path),
                    None,
                    None,
                    Some(id),
                    true,
                );
                continue;
            }
            Err(()) => {
                archive_failure(inventory, &path);
                return;
            }
        };
        let scan = with_xml_document(
            &bytes,
            super::xml::DEFAULT_XML_MAX_BYTES,
            super::xml::DEFAULT_XML_MAX_NODES,
            |document| {
                let root = document.root_element();
                let is_svg = media_type == SVG_MEDIA_TYPE;
                let expected_root = if is_svg {
                    root.tag_name().name() == "svg"
                        && root.tag_name().namespace() == Some(SVG_NAMESPACE)
                } else {
                    root.tag_name().name() == "html"
                        && root.tag_name().namespace() == Some(XHTML_NAMESPACE)
                };
                let scripted = document.descendants().any(|node| {
                    node.is_element()
                        && ((node.tag_name().name() == "script"
                            && matches!(
                                node.tag_name().namespace(),
                                Some(XHTML_NAMESPACE | SVG_NAMESPACE)
                            ))
                            || node
                                .attributes()
                                .any(|attribute| is_event_handler_attribute(attribute.name())))
                });
                let body = document
                    .descendants()
                    .find(|node| {
                        node.is_element()
                            && node.tag_name().name() == "body"
                            && node.tag_name().namespace() == Some(XHTML_NAMESPACE)
                    })
                    .unwrap_or_else(|| document.root_element());
                let has_readable_text = if is_svg {
                    document.descendants().any(|content_element| {
                        content_element.is_element()
                            && content_element.tag_name().namespace() == Some(SVG_NAMESPACE)
                            && matches!(content_element.tag_name().name(), "text" | "foreignObject")
                            && content_element
                                .descendants()
                                .filter(Node::is_text)
                                .filter(|text_node| {
                                    if content_element.tag_name().name() != "foreignObject" {
                                        return true;
                                    }
                                    let mut ancestor = text_node.parent();
                                    while let Some(element) = ancestor {
                                        if element == content_element {
                                            break;
                                        }
                                        if element.is_element()
                                            && element.tag_name().namespace() == Some(SVG_NAMESPACE)
                                            && matches!(element.tag_name().name(), "title" | "desc")
                                        {
                                            return false;
                                        }
                                        ancestor = element.parent();
                                    }
                                    true
                                })
                                .filter_map(|node| node.text())
                                .any(|text| !text.trim().is_empty())
                    })
                } else {
                    body.descendants()
                        .filter(Node::is_text)
                        .filter_map(|node| node.text())
                        .any(|text| !text.trim().is_empty())
                };
                let image_uses = document
                    .descendants()
                    .filter(|node| {
                        node.is_element()
                            && ((node.tag_name().name() == "img"
                                && node.tag_name().namespace() == Some(XHTML_NAMESPACE))
                                || (node.tag_name().name() == "image"
                                    && node.tag_name().namespace() == Some(SVG_NAMESPACE)))
                    })
                    .filter_map(|node| {
                        let href = node
                            .attribute("src")
                            .or_else(|| node.attribute("href"))
                            .or_else(|| {
                                namespaced_attribute(node, "http://www.w3.org/1999/xlink", "href")
                            })?;
                        let (line, column) = node_position(node);
                        Some(RawImageUse {
                            href: href.to_owned(),
                            line,
                            column,
                        })
                    })
                    .collect();
                MarkupScan {
                    scripted,
                    expected_root,
                    has_readable_text,
                    image_uses,
                }
            },
        );
        let scan = match scan {
            Ok(scan) => scan,
            Err(failure) => {
                xml_failure(inventory, &path, failure, DiagnosticCode::XmlMalformed);
                continue;
            }
        };

        let linear_spine_resource = inventory
            .spine
            .iter()
            .any(|item| item.resource_index == Some(resource_index) && item.linear);
        if linear_spine_resource {
            any_readable_text |= scan.has_readable_text;
            any_images |= !scan.image_uses.is_empty();
        }
        if !scan.expected_root && media_type == XHTML_MEDIA_TYPE {
            inventory.issue(
                DiagnosticCode::XhtmlNamespaceInvalid,
                DiagnosticSeverity::Warning,
                Some(path.clone()),
                Some(1),
                Some(1),
                Some(id.clone()),
                true,
            );
        }
        if scan.scripted
            && !inventory.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == DiagnosticCode::ScriptedContentUnsupported
                    && diagnostic.related_resource.as_deref() == Some(&id)
            })
        {
            inventory.issue(
                DiagnosticCode::ScriptedContentUnsupported,
                DiagnosticSeverity::Warning,
                Some(path.clone()),
                None,
                None,
                Some(id.clone()),
                true,
            );
        }

        let mut uses = Vec::new();
        for (occurrence, raw) in scan.image_uses.into_iter().enumerate() {
            let mut image_use = ImageUse {
                resource_path: None,
                manifest_resource_index: None,
                occurrence,
                external: false,
                line: raw.line,
                column: raw.column,
            };
            match resolve_href(&path, &raw.href) {
                Ok(target) if target.external => {
                    image_use.external = true;
                    inventory.issue(
                        DiagnosticCode::ExternalReferenceNotRead,
                        DiagnosticSeverity::Warning,
                        Some(path.clone()),
                        Some(raw.line),
                        Some(raw.column),
                        None,
                        true,
                    );
                }
                Ok(target) => {
                    if let Some(target_path) = target.path {
                        image_use.resource_path = Some(target_path.clone());
                        let target_entry_id = match archive.entry_by_path(&target_path) {
                            Ok(Some(entry)) if !entry.is_directory() => Some(entry.id()),
                            Ok(_) => {
                                inventory.issue(
                                    DiagnosticCode::ReferencedImageMissing,
                                    DiagnosticSeverity::Incomplete,
                                    Some(path.clone()),
                                    Some(raw.line),
                                    Some(raw.column),
                                    Some(target_path.clone()),
                                    true,
                                );
                                None
                            }
                            Err(_) => {
                                archive_failure(inventory, &target_path);
                                None
                            }
                        };
                        if let Some(entry_id) = target_entry_id {
                            if let std::collections::hash_map::Entry::Vacant(entry) =
                                image_readability.entry(entry_id)
                            {
                                let readable = archive.read_entry(entry_id).is_ok();
                                entry.insert(readable);
                                if !readable {
                                    inventory.issue(
                                        DiagnosticCode::ArchiveEntryUnreadable,
                                        DiagnosticSeverity::Incomplete,
                                        Some(path.clone()),
                                        Some(raw.line),
                                        Some(raw.column),
                                        Some(target_path.clone()),
                                        true,
                                    );
                                }
                            }
                            match inventory
                                .resources
                                .iter()
                                .enumerate()
                                .filter(|(_, resource)| {
                                    resource.path.as_deref() == Some(target_path.as_str())
                                })
                                .map(|(index, _)| index)
                                .collect::<Vec<_>>()
                                .as_slice()
                            {
                                [] => inventory.issue(
                                    DiagnosticCode::ReferencedImageUnlisted,
                                    DiagnosticSeverity::Warning,
                                    Some(path.clone()),
                                    Some(raw.line),
                                    Some(raw.column),
                                    Some(target_path),
                                    true,
                                ),
                                [index] => image_use.manifest_resource_index = Some(*index),
                                _ => inventory.issue(
                                    DiagnosticCode::ReferencedImageAmbiguous,
                                    DiagnosticSeverity::Incomplete,
                                    Some(path.clone()),
                                    Some(raw.line),
                                    Some(raw.column),
                                    Some(target_path),
                                    true,
                                ),
                            }
                        }
                    }
                }
                Err(_) => inventory.issue(
                    DiagnosticCode::ResourceHrefInvalid,
                    DiagnosticSeverity::Incomplete,
                    Some(path.clone()),
                    Some(raw.line),
                    Some(raw.column),
                    None,
                    true,
                ),
            }
            uses.push(image_use);
        }

        if let Some(resource) = inventory.resources.get_mut(resource_index) {
            resource.image_uses = uses.clone();
        }
        let fixed_layout_unsupported = inventory.spine.iter().any(|item| {
            item.resource_index == Some(resource_index)
                && item.kind == SpineContentKind::FixedLayoutDocument
        }) && (scan.has_readable_text || uses.is_empty());
        if fixed_layout_unsupported {
            inventory.issue(
                if scan.has_readable_text {
                    DiagnosticCode::FixedLayoutTextUnsupported
                } else {
                    DiagnosticCode::FixedLayoutPageUnsupported
                },
                DiagnosticSeverity::Incomplete,
                Some(path.clone()),
                None,
                None,
                Some(id.clone()),
                true,
            );
        }
        for item in inventory
            .spine
            .iter_mut()
            .filter(|item| item.resource_index == Some(resource_index))
        {
            item.image_uses = uses.clone();
            if item.kind == SpineContentKind::FixedLayoutDocument {
                item.kind = if fixed_layout_unsupported {
                    SpineContentKind::Unsupported
                } else {
                    SpineContentKind::ImagePage
                };
            }
        }
    }
    if linear_count > 0 && !any_readable_text && any_images {
        inventory.issue(
            DiagnosticCode::ImageOnlyContentNeedsImageWorkflow,
            DiagnosticSeverity::Warning,
            inventory.package_path.clone(),
            None,
            None,
            None,
            true,
        );
    }
}

fn is_event_handler_attribute(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "onabort"
            | "onauxclick"
            | "onbeforeinput"
            | "onbeforeprint"
            | "onbeforeunload"
            | "onbeforematch"
            | "onbeforetoggle"
            | "onblur"
            | "oncancel"
            | "oncanplay"
            | "oncanplaythrough"
            | "onchange"
            | "onanimationcancel"
            | "onanimationend"
            | "onanimationiteration"
            | "onanimationstart"
            | "onclick"
            | "onclose"
            | "oncontextlost"
            | "oncontextmenu"
            | "oncontextrestored"
            | "oncopy"
            | "oncuechange"
            | "oncut"
            | "ondblclick"
            | "ondrag"
            | "ondragend"
            | "ondragenter"
            | "ondragleave"
            | "ondragover"
            | "ondragstart"
            | "ondrop"
            | "ondurationchange"
            | "onemptied"
            | "onended"
            | "onerror"
            | "onfocus"
            | "onfocusin"
            | "onfocusout"
            | "onformdata"
            | "onfullscreenchange"
            | "onfullscreenerror"
            | "onhashchange"
            | "oninput"
            | "oninvalid"
            | "onkeydown"
            | "onkeypress"
            | "onkeyup"
            | "onload"
            | "onloadeddata"
            | "onloadedmetadata"
            | "onloadstart"
            | "onmessage"
            | "onmessageerror"
            | "onmousedown"
            | "onmouseenter"
            | "onmouseleave"
            | "onmousemove"
            | "onmouseout"
            | "onmouseover"
            | "onmouseup"
            | "ononline"
            | "onoffline"
            | "onpagehide"
            | "onpageshow"
            | "onpaste"
            | "onpointercancel"
            | "onpointerdown"
            | "ongotpointercapture"
            | "onlostpointercapture"
            | "onpointerenter"
            | "onpointerleave"
            | "onpointermove"
            | "onpointerout"
            | "onpointerover"
            | "onpointerrawupdate"
            | "onpointerup"
            | "ontouchcancel"
            | "ontouchend"
            | "ontouchmove"
            | "ontouchstart"
            | "onpause"
            | "onplay"
            | "onplaying"
            | "onpopstate"
            | "onprogress"
            | "onratechange"
            | "onreset"
            | "onresize"
            | "onscroll"
            | "onsecuritypolicyviolation"
            | "onseeked"
            | "onseeking"
            | "onselect"
            | "onslotchange"
            | "onstalled"
            | "onsubmit"
            | "onsuspend"
            | "ontimeupdate"
            | "ontoggle"
            | "ontransitioncancel"
            | "ontransitionend"
            | "ontransitionrun"
            | "ontransitionstart"
            | "onvolumechange"
            | "onbeforecopy"
            | "onbeforecut"
            | "onbeforepaste"
            | "onafterprint"
            | "onunload"
            | "onstorage"
            | "onlanguagechange"
            | "onrejectionhandled"
            | "onunhandledrejection"
            | "onbegin"
            | "onend"
            | "onrepeat"
            | "onwaiting"
            | "onwheel"
    )
}
fn namespaced_attribute<'a>(node: Node<'a, 'a>, namespace: &str, name: &str) -> Option<&'a str> {
    node.attributes()
        .find(|attribute| attribute.name() == name && attribute.namespace() == Some(namespace))
        .map(|attribute| attribute.value())
}

fn read_entry<R: Read + Seek>(archive: &mut EpubZip<R>, path: &str) -> Result<Option<Vec<u8>>, ()> {
    let entry = archive.entry_by_path(path).map_err(|_| ())?;
    let Some(entry) = entry else { return Ok(None) };
    if entry.is_directory() {
        return Ok(None);
    }
    archive.read_entry(entry.id()).map(Some).map_err(|_| ())
}

fn entry_exists<R: Read + Seek>(archive: &EpubZip<R>, path: &str) -> Result<bool, ()> {
    archive
        .entry_by_path(path)
        .map(|entry| entry.is_some_and(|entry| !entry.is_directory()))
        .map_err(|_| ())
}

fn archive_failure(inventory: &mut EpubInventory, path: &str) {
    inventory.issue(
        DiagnosticCode::ArchiveEntryUnreadable,
        DiagnosticSeverity::Incomplete,
        Some(path.to_owned()),
        None,
        None,
        None,
        true,
    );
}

fn xml_failure(
    inventory: &mut EpubInventory,
    path: &str,
    failure: XmlFailure,
    fallback: DiagnosticCode,
) {
    let code = match failure.kind {
        XmlFailureKind::ByteLimit => DiagnosticCode::XmlByteLimitExceeded,
        XmlFailureKind::NodeLimit => DiagnosticCode::XmlNodeLimitExceeded,
        XmlFailureKind::UnsupportedEncoding => DiagnosticCode::XmlEncodingUnsupported,
        XmlFailureKind::ExternalEntity => DiagnosticCode::XmlExternalEntityRejected,
        XmlFailureKind::UnknownEntity => DiagnosticCode::XmlEntityReferenceUnknown,
        XmlFailureKind::EntityExpansionLimit => DiagnosticCode::XmlEntityExpansionLimit,
        XmlFailureKind::NavigationDepthLimit => DiagnosticCode::NavigationDepthLimitExceeded,
        XmlFailureKind::Malformed => fallback,
    };
    inventory.issue(
        code,
        DiagnosticSeverity::Incomplete,
        Some(path.to_owned()),
        failure.line,
        failure.column,
        None,
        true,
    );
}

fn archive_failed(inventory: &EpubInventory) -> bool {
    inventory
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::ArchiveEntryUnreadable)
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, DiagnosticCode, DiagnosticSeverity};

    fn related(value: &str) -> Option<String> {
        Diagnostic::new(
            DiagnosticCode::ResourceMissing,
            DiagnosticSeverity::Incomplete,
            None,
            None,
            None,
            Some(value.to_owned()),
        )
        .related_resource
    }

    #[test]
    fn sanitizes_related_resources_and_keeps_safe_ids_and_paths() {
        assert_eq!(related("chapter-1"), Some("chapter-1".to_owned()));
        assert_eq!(
            related("OPS/text/chapter one.xhtml"),
            Some("OPS/text/chapter one.xhtml".to_owned())
        );
        for unsafe_value in [
            "https://user:secret@example.test/book?token=private",
            "chapter?token=private",
            "C:/Users/Phun/private.epub",
            "/private/book.epub",
            "../outside",
            "OPS/%2Fsecret",
        ] {
            assert_eq!(related(unsafe_value), None, "{unsafe_value}");
        }
    }
}
