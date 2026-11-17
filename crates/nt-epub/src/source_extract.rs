//! Source blocks with stable identities and offsets into decoded XHTML text.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::{Read, Seek};

use nt_domain::chapter_map::{
    ChapterMapInput, ConfirmedBoundary, CoverageDomain as MapCoverageDomain,
    NavigationItem as MapNavigationItem, NavigationSource as MapNavigationSource,
    NavigationTarget as MapNavigationTarget, NavigationTree as MapNavigationTree,
    NavigationTreeState, SourceAnchor as MapSourceAnchor, SourceDocument as MapSourceDocument,
    SourceInputDiagnostic,
};
use nt_domain::source_range::{
    AttributeSourceText, BlockIdentityRecord, BlockLocator, ElementLocator, ImageSource,
    InlineMetadataKind, InlineSourceMetadata, ProtectedResourceHash, ProtectedSourceUnit,
    RubySource, SourceBlock, SourceBlockId, SourceSpan, SourceUnit, SourceUnitKind,
    TextNodeLocator, UnitRange,
};
use roxmltree::{Document, Node, NodeId, ParsingOptions};
use sha2::{Digest, Sha256};
use unicode_segmentation::UnicodeSegmentation;

use crate::archive::{ArchiveError, EpubZip};
use crate::inventory::{
    Diagnostic, EpubInventory, InventoryCoverage, ResourceEncryption, SpineContentKind,
};
use crate::package::resolve_href;
use crate::xml::{
    DEFAULT_XML_MAX_BYTES, DEFAULT_XML_MAX_NODES, XmlFailure, XmlFailureKind, with_xml_document,
};

const XHTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";
const XLINK_NAMESPACE: &str = "http://www.w3.org/1999/xlink";
const EPUB_NAMESPACE: &str = "http://www.idpf.org/2007/ops";
pub const PARSER_VERSION: &str = "nt-epub/1.5.0";
const MAX_XML_ELEMENT_DEPTH: usize = 256;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CoverageDomain {
    MainFlow,
    Auxiliary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceDocument {
    pub canonical_path: String,
    pub domain: CoverageDomain,
    /// Every spine occurrence is kept, including repeated and nonlinear items.
    pub spine_positions: Vec<usize>,
    pub linear_spine_positions: Vec<usize>,
    pub blocks: Vec<SourceBlock>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceAnchor {
    pub canonical_path: String,
    pub fragment: String,
    pub locator: ElementLocator,
    pub block_id: SourceBlockId,
    /// A valid insertion boundary in the block's grapheme/protected-unit list.
    pub unit_offset: u32,
    pub is_unique: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceDiagnostic {
    InventoryPartial,
    InvalidOptionalNavigation { canonical_path: String },
    Inventory(Diagnostic),
    UnsupportedFixedLayout { canonical_path: String },
    UnsupportedEncryption { canonical_path: String },
    DuplicateXhtmlId { canonical_path: String, id: String },
    ProtectedResourceMissing { canonical_path: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceExtraction {
    pub epub_sha256: [u8; 32],
    pub parser_version: String,
    pub documents: Vec<SourceDocument>,
    pub anchors: Vec<SourceAnchor>,
    pub navigation: Vec<MapNavigationTree>,
    pub navigation_diagnostics: Vec<SourceInputDiagnostic>,
    pub diagnostics: Vec<SourceDiagnostic>,
    pub complete: bool,
}

impl SourceExtraction {
    /// Adapt the verified parser output to the application-owned chapter-map contract.
    pub fn chapter_map_input(
        &self,
        confirmed_boundaries: Vec<ConfirmedBoundary>,
    ) -> ChapterMapInput {
        ChapterMapInput {
            epub_sha256: self.epub_sha256,
            parser_version: self.parser_version.clone(),
            documents: self
                .documents
                .iter()
                .map(|document| MapSourceDocument {
                    canonical_path: document.canonical_path.clone(),
                    domain: match document.domain {
                        CoverageDomain::MainFlow => MapCoverageDomain::MainFlow,
                        CoverageDomain::Auxiliary => MapCoverageDomain::Auxiliary,
                    },
                    spine_positions: document.spine_positions.clone(),
                    linear_spine_positions: document.linear_spine_positions.clone(),
                    blocks: document.blocks.clone(),
                })
                .collect(),
            anchors: self
                .anchors
                .iter()
                .map(|anchor| MapSourceAnchor {
                    canonical_path: anchor.canonical_path.clone(),
                    fragment: anchor.fragment.clone(),
                    block_id: anchor.block_id,
                    unit_offset: anchor.unit_offset,
                    is_unique: anchor.is_unique,
                })
                .collect(),
            navigation: self.navigation.clone(),
            confirmed_boundaries,
            diagnostics: self.navigation_diagnostics.clone(),
        }
    }
}

#[derive(Debug)]
pub enum SourceExtractionError {
    EmptyParserVersion,
    Archive(ArchiveError),
    Xml {
        canonical_path: String,
    },
    InvalidXhtmlRoot {
        canonical_path: String,
    },
    InvalidSvgRoot {
        canonical_path: String,
    },
    InvalidIdentity,
    InvalidSourceSpan,
    XmlDepthLimit {
        canonical_path: String,
        max_depth: usize,
    },
}

impl fmt::Display for SourceExtractionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyParserVersion => formatter.write_str("parser version must not be empty"),
            Self::Archive(error) => write!(formatter, "EPUB archive read failed: {error}"),
            Self::Xml { canonical_path } => {
                write!(
                    formatter,
                    "XML resource `{canonical_path}` is not safe valid XML"
                )
            }
            Self::InvalidXhtmlRoot { canonical_path } => write!(
                formatter,
                "resource `{canonical_path}` does not have an XHTML html root"
            ),
            Self::InvalidSvgRoot { canonical_path } => write!(
                formatter,
                "resource `{canonical_path}` does not have an SVG root"
            ),
            Self::InvalidIdentity => formatter.write_str("source block identity is invalid"),
            Self::InvalidSourceSpan => formatter.write_str("source text span is invalid"),
            Self::XmlDepthLimit {
                canonical_path,
                max_depth,
            } => write!(
                formatter,
                "XML resource {canonical_path} exceeds the safe element nesting limit of {max_depth}"
            ),
        }
    }
}

impl std::error::Error for SourceExtractionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Archive(error) => Some(error),
            _ => None,
        }
    }
}

/// Extract source blocks from the already-inspected archive using its inventory.
/// The caller supplies the hash of the original EPUB bytes; no file is reopened.
pub fn extract_source<R: Read + Seek>(
    archive: &mut EpubZip<R>,
    inventory: &EpubInventory,
    epub_sha256: [u8; 32],
    parser_version: &str,
) -> Result<SourceExtraction, SourceExtractionError> {
    if parser_version.trim().is_empty() {
        return Err(SourceExtractionError::EmptyParserVersion);
    }

    let mut positions_by_resource = HashMap::<usize, (Vec<usize>, Vec<usize>, bool)>::new();
    for item in &inventory.spine {
        if let Some(index) = item.resource_index {
            let entry = positions_by_resource.entry(index).or_default();
            entry.0.push(item.reading_order);
            if item.linear {
                entry.1.push(item.reading_order);
            }
            if item.kind == SpineContentKind::FixedLayoutDocument {
                entry.2 = true;
            }
        }
    }

    let mut resource_order: Vec<usize> = inventory
        .resources
        .iter()
        .enumerate()
        .filter_map(|(index, resource)| {
            (resource.media_type == "application/xhtml+xml"
                || inventory.spine.iter().any(|item| {
                    item.resource_index == Some(index)
                        && item.linear
                        && item.kind == SpineContentKind::ImagePage
                }))
            .then_some(index)
        })
        .collect();
    resource_order.sort_by_key(|index| {
        let linear_position = positions_by_resource
            .get(index)
            .and_then(|entry| entry.1.iter().min().copied());
        (
            linear_position.is_none(),
            linear_position.unwrap_or(*index),
            *index,
        )
    });

    let mut documents = Vec::new();
    let mut anchors = Vec::new();
    let mut diagnostics = Vec::new();
    let mut optional_navigation_diagnostics = Vec::new();
    if inventory.coverage == InventoryCoverage::Partial {
        diagnostics.push(SourceDiagnostic::InventoryPartial);
    }
    diagnostics.extend(
        inventory
            .diagnostics
            .iter()
            .cloned()
            .map(SourceDiagnostic::Inventory),
    );

    for resource_index in resource_order {
        let resource = &inventory.resources[resource_index];
        let Some(canonical_path) = resource.path.as_deref() else {
            continue;
        };
        let positions = positions_by_resource
            .get(&resource_index)
            .cloned()
            .unwrap_or_default();
        let (spine_positions, linear_positions, fixed_layout) = positions;
        if fixed_layout || resource.encryption == ResourceEncryption::Unsupported {
            diagnostics.push(if fixed_layout {
                SourceDiagnostic::UnsupportedFixedLayout {
                    canonical_path: canonical_path.to_owned(),
                }
            } else {
                SourceDiagnostic::UnsupportedEncryption {
                    canonical_path: canonical_path.to_owned(),
                }
            });
            continue;
        }
        if !resource.available {
            continue;
        }

        let Some(entry) = archive
            .entry_by_path(canonical_path)
            .map_err(SourceExtractionError::Archive)?
        else {
            continue;
        };
        let bytes = archive
            .read_entry(entry.id())
            .map_err(SourceExtractionError::Archive)?;
        let is_navigation_document = resource.properties.iter().any(|property| property == "nav");
        let is_linear_image_page = inventory.spine.iter().any(|item| {
            item.resource_index == Some(resource_index)
                && item.linear
                && item.kind == SpineContentKind::ImagePage
        });
        let parsed = if is_linear_image_page && resource.media_type == "image/svg+xml" {
            parse_svg_image_page(
                &bytes,
                canonical_path,
                epub_sha256,
                parser_version,
                linear_positions.iter().copied().min().unwrap_or_default(),
            )?
        } else if is_linear_image_page && resource.media_type != "application/xhtml+xml" {
            ParsedDocument {
                blocks: vec![image_page_block(
                    canonical_path,
                    epub_sha256,
                    parser_version,
                    linear_positions.iter().copied().min().unwrap_or_default(),
                )?],
                anchors: Vec::new(),
                diagnostics: Vec::new(),
            }
        } else {
            match parse_xhtml(
                &bytes,
                canonical_path,
                epub_sha256,
                parser_version,
                is_navigation_document,
            ) {
                Ok(parsed) => parsed,
                Err(XhtmlParseFailure::Xml(failure))
                    if is_navigation_document
                        && spine_positions.is_empty()
                        && !is_xml_resource_limit(failure.kind) =>
                {
                    diagnostics.push(SourceDiagnostic::InvalidOptionalNavigation {
                        canonical_path: canonical_path.to_owned(),
                    });
                    optional_navigation_diagnostics.push(SourceInputDiagnostic {
                        source: Some(MapNavigationSource::Epub3Nav),
                        resource_path: Some(canonical_path.to_owned()),
                        code: if failure.kind == XmlFailureKind::Malformed {
                            "NavigationInvalidXml"
                        } else {
                            "NavigationXmlUnavailable"
                        }
                        .to_owned(),
                        reason: format!(
                            "optional navigation XHTML could not be parsed: {:?}",
                            failure.kind
                        ),
                        line: failure.line,
                        column: failure.column,
                    });
                    continue;
                }
                Err(XhtmlParseFailure::Source(SourceExtractionError::InvalidXhtmlRoot {
                    ..
                })) if is_navigation_document && spine_positions.is_empty() => {
                    diagnostics.push(SourceDiagnostic::InvalidOptionalNavigation {
                        canonical_path: canonical_path.to_owned(),
                    });
                    optional_navigation_diagnostics.push(SourceInputDiagnostic {
                        source: Some(MapNavigationSource::Epub3Nav),
                        resource_path: Some(canonical_path.to_owned()),
                        code: "NavigationInvalidXhtmlRoot".to_owned(),
                        reason: "optional navigation resource is not an XHTML document".to_owned(),
                        line: None,
                        column: None,
                    });
                    continue;
                }
                Err(XhtmlParseFailure::Xml(_)) => {
                    return Err(SourceExtractionError::Xml {
                        canonical_path: canonical_path.to_owned(),
                    });
                }
                Err(XhtmlParseFailure::Source(error)) => return Err(error),
            }
        };
        diagnostics.extend(parsed.diagnostics);
        anchors.extend(parsed.anchors);
        let blocks = parsed.blocks;
        let is_linear_readable = !linear_positions.is_empty()
            && inventory.spine.iter().any(|item| {
                item.resource_index == Some(resource_index)
                    && item.linear
                    && matches!(
                        item.kind,
                        SpineContentKind::ReflowableDocument | SpineContentKind::ImagePage
                    )
            });
        let domain = if is_linear_readable && !is_navigation_document {
            CoverageDomain::MainFlow
        } else {
            CoverageDomain::Auxiliary
        };
        documents.push(SourceDocument {
            canonical_path: canonical_path.to_owned(),
            domain,
            spine_positions,
            linear_spine_positions: linear_positions,
            blocks,
        });
    }

    let (navigation, mut navigation_diagnostics) = extract_navigation(archive, inventory)?;
    navigation_diagnostics.extend(optional_navigation_diagnostics);

    let mut resource_hashes = HashMap::<String, [u8; 32]>::new();
    for document in &mut documents {
        for block in &mut document.blocks {
            let image_paths: HashSet<_> = block
                .units
                .iter()
                .filter_map(|unit| match &unit.kind {
                    SourceUnitKind::Protected(ProtectedSourceUnit::Image(image)) => {
                        image.resource_path.clone()
                    }
                    _ => None,
                })
                .collect();
            for path in image_paths {
                let digest = if let Some(digest) = resource_hashes.get(&path) {
                    Some(*digest)
                } else if let Some(entry) = archive
                    .entry_by_path(&path)
                    .map_err(SourceExtractionError::Archive)?
                {
                    let bytes = archive
                        .read_entry(entry.id())
                        .map_err(SourceExtractionError::Archive)?;
                    let digest: [u8; 32] = Sha256::digest(&bytes).into();
                    resource_hashes.insert(path.clone(), digest);
                    Some(digest)
                } else {
                    None
                };
                if let Some(sha256) = digest {
                    block.protected_resource_hashes.push(ProtectedResourceHash {
                        canonical_path: path,
                        sha256,
                    });
                } else {
                    diagnostics.push(SourceDiagnostic::ProtectedResourceMissing {
                        canonical_path: path,
                    });
                }
            }
            block
                .protected_resource_hashes
                .sort_by(|left, right| left.canonical_path.cmp(&right.canonical_path));
        }
    }

    documents.sort_by_key(|document| {
        let first_linear = document.linear_spine_positions.iter().min().copied();
        (
            document.domain != CoverageDomain::MainFlow,
            first_linear.unwrap_or(usize::MAX),
        )
    });
    anchors.sort_by(|left, right| {
        (&left.canonical_path, &left.fragment, left.unit_offset).cmp(&(
            &right.canonical_path,
            &right.fragment,
            right.unit_offset,
        ))
    });
    let complete = diagnostics.iter().all(|diagnostic| match diagnostic {
        SourceDiagnostic::InventoryPartial
        | SourceDiagnostic::InvalidOptionalNavigation { .. }
        | SourceDiagnostic::UnsupportedFixedLayout { .. }
        | SourceDiagnostic::UnsupportedEncryption { .. }
        | SourceDiagnostic::ProtectedResourceMissing { .. } => false,
        SourceDiagnostic::Inventory(diagnostic) => {
            diagnostic.severity != crate::inventory::DiagnosticSeverity::Incomplete
        }
        SourceDiagnostic::DuplicateXhtmlId { .. } => true,
    });
    Ok(SourceExtraction {
        epub_sha256,
        parser_version: parser_version.to_owned(),
        documents,
        anchors,
        navigation,
        navigation_diagnostics,
        diagnostics,
        complete,
    })
}

struct RawNavigationItem {
    label: String,
    href: Option<String>,
    children: Vec<RawNavigationItem>,
}

fn extract_navigation<R: Read + Seek>(
    archive: &mut EpubZip<R>,
    inventory: &EpubInventory,
) -> Result<(Vec<MapNavigationTree>, Vec<SourceInputDiagnostic>), SourceExtractionError> {
    let mut trees = Vec::new();
    let mut diagnostics = Vec::new();
    for tree in &inventory.navigation {
        let source = match tree.source {
            crate::inventory::NavigationSource::Epub3Nav => MapNavigationSource::Epub3Nav,
            crate::inventory::NavigationSource::Epub2Ncx => MapNavigationSource::Epub2Ncx,
        };
        let entry_id = archive
            .entry_by_path(&tree.resource_path)
            .map_err(SourceExtractionError::Archive)?
            .map(|entry| entry.id());
        let raw_items = match entry_id {
            Some(entry_id) => {
                let bytes = archive
                    .read_entry(entry_id)
                    .map_err(SourceExtractionError::Archive)?;
                match with_xml_document(
                    &bytes,
                    DEFAULT_XML_MAX_BYTES,
                    DEFAULT_XML_MAX_NODES,
                    |document| {
                        validate_xml_depth(document.root(), &tree.resource_path)?;
                        Ok(parse_raw_navigation(document, source))
                    },
                ) {
                    Ok(result) => result?,
                    Err(failure) if is_xml_resource_limit(failure.kind) => {
                        return Err(SourceExtractionError::Xml {
                            canonical_path: tree.resource_path.clone(),
                        });
                    }
                    Err(_) => None,
                }
            }
            None => None,
        };
        let parsed_raw = raw_items.is_some();
        let (items, aligned, unresolved) =
            merge_navigation_items(&tree.items, raw_items.as_deref());
        let state = if parsed_raw && aligned && !unresolved {
            NavigationTreeState::Usable
        } else if !parsed_raw && tree.items.is_empty() {
            NavigationTreeState::Invalid
        } else {
            NavigationTreeState::Partial
        };
        if !parsed_raw {
            diagnostics.push(SourceInputDiagnostic {
                source: Some(source), resource_path: Some(tree.resource_path.clone()),
                code: "NavigationXmlUnavailable".to_owned(),
                reason: "the navigation resource could not be read and parsed with the bounded XML parser".to_owned(),
                line: None, column: None,
            });
        } else if !aligned {
            diagnostics.push(SourceInputDiagnostic {
                source: Some(source), resource_path: Some(tree.resource_path.clone()),
                code: "NavigationTreeMismatch".to_owned(),
                reason: "raw navigation items do not align with the inventory tree; both are retained as partial evidence".to_owned(),
                line: None, column: None,
            });
        }
        trees.push(MapNavigationTree {
            source,
            resource_path: tree.resource_path.clone(),
            state,
            items,
        });
    }

    for diagnostic in &inventory.diagnostics {
        if !is_navigation_diagnostic(diagnostic.code) {
            continue;
        }
        let source = inventory
            .navigation
            .iter()
            .find(|tree| diagnostic.resource_path.as_deref() == Some(tree.resource_path.as_str()))
            .map(|tree| match tree.source {
                crate::inventory::NavigationSource::Epub3Nav => MapNavigationSource::Epub3Nav,
                crate::inventory::NavigationSource::Epub2Ncx => MapNavigationSource::Epub2Ncx,
            });
        diagnostics.push(SourceInputDiagnostic {
            source,
            resource_path: diagnostic.resource_path.clone(),
            code: format!("{:?}", diagnostic.code),
            reason: format!("EPUB inventory reported {:?}", diagnostic.code),
            line: diagnostic.line,
            column: diagnostic.column,
        });
    }
    Ok((trees, diagnostics))
}

fn is_xml_resource_limit(kind: XmlFailureKind) -> bool {
    matches!(
        kind,
        XmlFailureKind::ByteLimit
            | XmlFailureKind::NodeLimit
            | XmlFailureKind::EntityExpansionLimit
            | XmlFailureKind::NavigationDepthLimit
    )
}

fn is_navigation_diagnostic(code: crate::inventory::DiagnosticCode) -> bool {
    use crate::inventory::DiagnosticCode;
    matches!(
        code,
        DiagnosticCode::NavigationMissing
            | DiagnosticCode::NavigationInvalidXml
            | DiagnosticCode::NavigationStructureUnsupported
            | DiagnosticCode::NavigationDepthLimitExceeded
            | DiagnosticCode::NavigationTargetMissing
            | DiagnosticCode::NavigationTargetUnlisted
            | DiagnosticCode::NavigationTargetAmbiguous
            | DiagnosticCode::NavigationSourcesConflict
            | DiagnosticCode::ResourceHrefInvalid
            | DiagnosticCode::ExternalReferenceNotRead
    )
}

fn parse_raw_navigation(
    document: &Document<'_>,
    source: MapNavigationSource,
) -> Option<Vec<RawNavigationItem>> {
    match source {
        MapNavigationSource::Epub3Nav => {
            let toc = document.descendants().find(|node| {
                is_xhtml_element(*node, "nav")
                    && node
                        .attribute((EPUB_NAMESPACE, "type"))
                        .is_some_and(|value| {
                            value.split_ascii_whitespace().any(|token| token == "toc")
                        })
            })?;
            let list = toc
                .descendants()
                .find(|node| is_xhtml_element(*node, "ol"))?;
            Some(parse_raw_html_list(list))
        }
        MapNavigationSource::Epub2Ncx => {
            let nav_map = document
                .descendants()
                .find(|node| node.is_element() && node.tag_name().name() == "navMap")?;
            Some(
                nav_map
                    .children()
                    .filter(|node| node.is_element() && node.tag_name().name() == "navPoint")
                    .map(parse_raw_ncx_item)
                    .collect(),
            )
        }
    }
}

fn parse_raw_html_list(list: Node<'_, '_>) -> Vec<RawNavigationItem> {
    list.children()
        .filter(|node| is_xhtml_element(*node, "li"))
        .map(parse_raw_html_item)
        .collect()
}

fn parse_raw_html_item(item: Node<'_, '_>) -> RawNavigationItem {
    let label_node = item
        .children()
        .find(|node| node.is_element() && matches!(node.tag_name().name(), "a" | "span"));
    let href = label_node
        .and_then(|node| node.attribute("href"))
        .map(str::to_owned);
    let label = label_node.map(collect_node_text).unwrap_or_default();
    let children = item
        .children()
        .find(|node| is_xhtml_element(*node, "ol"))
        .map(parse_raw_html_list)
        .unwrap_or_default();
    RawNavigationItem {
        label,
        href,
        children,
    }
}

fn parse_raw_ncx_item(item: Node<'_, '_>) -> RawNavigationItem {
    let content = item
        .children()
        .find(|node| node.is_element() && node.tag_name().name() == "content");
    let href = content
        .and_then(|node| node.attribute("src"))
        .map(str::to_owned);
    let label = item
        .children()
        .find(|node| node.is_element() && node.tag_name().name() == "navLabel")
        .map(collect_node_text)
        .unwrap_or_default();
    let children = item
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "navPoint")
        .map(parse_raw_ncx_item)
        .collect();
    RawNavigationItem {
        label,
        href,
        children,
    }
}

fn collect_node_text(node: Node<'_, '_>) -> String {
    node.descendants()
        .filter_map(|child| child.text())
        .collect::<String>()
        .trim()
        .to_owned()
}

fn merge_navigation_items(
    inventory: &[crate::inventory::NavigationItem],
    raw: Option<&[RawNavigationItem]>,
) -> (Vec<MapNavigationItem>, bool, bool) {
    let raw = raw.unwrap_or_default();
    let mut result = Vec::new();
    let mut aligned = inventory.len() == raw.len();
    let mut unresolved = false;
    for index in 0..inventory.len().max(raw.len()) {
        let inventory_item = inventory.get(index);
        let raw_item = raw.get(index);
        let label = inventory_item
            .map(|item| item.label.clone())
            .filter(|label| !label.trim().is_empty())
            .or_else(|| raw_item.map(|item| item.label.clone()))
            .unwrap_or_default();
        let raw_href = raw_item.and_then(|item| item.href.clone());
        let target = match inventory_item.and_then(|item| item.target.as_ref()) {
            Some(target) => match target.resource_path.clone() {
                Some(canonical_path) => Some(MapNavigationTarget::Resolved {
                    canonical_path,
                    fragment: target.fragment.clone(),
                }),
                None => {
                    unresolved = true;
                    Some(MapNavigationTarget::Invalid {
                        reason: "inventory target has no canonical resource path".to_owned(),
                    })
                }
            },
            None if raw_href.is_some() => {
                unresolved = true;
                Some(MapNavigationTarget::Invalid {
                    reason: "inventory could not resolve the original navigation href".to_owned(),
                })
            }
            None => None,
        };
        let inventory_children = inventory_item
            .map(|item| item.children.as_slice())
            .unwrap_or_default();
        let raw_children = raw_item.map(|item| item.children.as_slice());
        let (children, child_aligned, child_unresolved) =
            merge_navigation_items(inventory_children, raw_children);
        aligned &= child_aligned;
        unresolved |= child_unresolved;
        if inventory_item.is_none() || raw_item.is_none() {
            aligned = false;
            if raw_href.is_some() {
                unresolved = true;
            }
        }
        result.push(MapNavigationItem {
            label,
            raw_href,
            target,
            children,
        });
    }
    (result, aligned, unresolved)
}

struct ParsedDocument {
    blocks: Vec<SourceBlock>,
    anchors: Vec<SourceAnchor>,
    diagnostics: Vec<SourceDiagnostic>,
}

enum XhtmlParseFailure {
    Xml(XmlFailure),
    Source(SourceExtractionError),
}

fn validate_xml_depth(
    root: Node<'_, '_>,
    canonical_path: &str,
) -> Result<(), SourceExtractionError> {
    let mut pending = vec![(root, 0_usize)];
    while let Some((node, parent_depth)) = pending.pop() {
        let depth = if node.is_element() {
            parent_depth.saturating_add(1)
        } else {
            parent_depth
        };
        if depth > MAX_XML_ELEMENT_DEPTH {
            return Err(SourceExtractionError::XmlDepthLimit {
                canonical_path: canonical_path.to_owned(),
                max_depth: MAX_XML_ELEMENT_DEPTH,
            });
        }
        for child in node.children() {
            pending.push((child, depth));
        }
    }
    Ok(())
}

fn image_page_block(
    canonical_path: &str,
    epub_sha256: [u8; 32],
    parser_version: &str,
    spine_position: usize,
) -> Result<SourceBlock, SourceExtractionError> {
    let locator = BlockLocator::NodePath {
        element_ordinals: Vec::new(),
        order: 0,
    };
    let record = BlockIdentityRecord {
        epub_sha256,
        parser_version: parser_version.to_owned(),
        canonical_path: canonical_path.to_owned(),
        locator: locator.clone(),
    };
    let identity: [u8; 32] = Sha256::digest(
        record
            .canonical_bytes()
            .map_err(|_| SourceExtractionError::InvalidIdentity)?,
    )
    .into();
    let units = vec![SourceUnit::protected(
        ProtectedSourceUnit::Image(ImageSource {
            raw_source: Some(canonical_path.to_owned()),
            resource_path: Some(canonical_path.to_owned()),
            alt: None,
            occurrence: spine_position,
            node_path: Vec::new(),
            external: false,
        }),
        Vec::new(),
    )];
    let content_hash = hash_block_content(&units, &[], &[]);
    Ok(SourceBlock {
        id: SourceBlockId::from_bytes(identity),
        canonical_path: canonical_path.to_owned(),
        element_name: "image-page".to_owned(),
        locator,
        parser_version: parser_version.to_owned(),
        content_hash,
        units,
        inline_metadata: Vec::new(),
        attribute_text: Vec::new(),
        protected_resource_hashes: Vec::new(),
    })
}

fn parse_svg_image_page(
    bytes: &[u8],
    canonical_path: &str,
    epub_sha256: [u8; 32],
    parser_version: &str,
    order: usize,
) -> Result<ParsedDocument, SourceExtractionError> {
    with_xml_document(
        bytes,
        DEFAULT_XML_MAX_BYTES,
        DEFAULT_XML_MAX_NODES,
        |document| {
            validate_xml_depth(document.root(), canonical_path)?;
            let root = document.root_element();
            if root.tag_name().name() != "svg" || root.tag_name().namespace() != Some(SVG_NAMESPACE)
            {
                return Err(SourceExtractionError::InvalidSvgRoot {
                    canonical_path: canonical_path.to_owned(),
                });
            }

            let fragment_counts = element_fragment_counts(root);
            let block = image_page_block(canonical_path, epub_sha256, parser_version, order)?;
            let anchors = svg_anchors(root, canonical_path, &block, &fragment_counts);
            Ok(ParsedDocument {
                blocks: vec![block],
                anchors,
                diagnostics: Vec::new(),
            })
        },
    )
    .map_err(|_| SourceExtractionError::Xml {
        canonical_path: canonical_path.to_owned(),
    })?
}

fn svg_anchors(
    root: Node<'_, '_>,
    canonical_path: &str,
    block: &SourceBlock,
    fragment_counts: &HashMap<String, usize>,
) -> Vec<SourceAnchor> {
    fn visit(
        node: Node<'_, '_>,
        path: &mut Vec<u32>,
        canonical_path: &str,
        block: &SourceBlock,
        fragment_counts: &HashMap<String, usize>,
        anchors: &mut Vec<SourceAnchor>,
    ) {
        for fragment in element_fragments(node) {
            if fragment_counts.get(&fragment) != Some(&1) {
                continue;
            }
            anchors.push(SourceAnchor {
                canonical_path: canonical_path.to_owned(),
                fragment,
                locator: ElementLocator {
                    block: block.locator.clone(),
                    element_ordinals: path.clone(),
                },
                block_id: block.id,
                unit_offset: 0,
                is_unique: true,
            });
        }

        let mut ordinal = 0_u32;
        for child in node.children().filter(|child| child.is_element()) {
            path.push(ordinal);
            ordinal = ordinal.saturating_add(1);
            visit(child, path, canonical_path, block, fragment_counts, anchors);
            path.pop();
        }
    }

    let mut anchors = Vec::new();
    visit(
        root,
        &mut Vec::new(),
        canonical_path,
        block,
        fragment_counts,
        &mut anchors,
    );
    anchors
}

fn parse_xhtml(
    bytes: &[u8],
    canonical_path: &str,
    epub_sha256: [u8; 32],
    parser_version: &str,
    is_navigation_document: bool,
) -> Result<ParsedDocument, XhtmlParseFailure> {
    with_xml_document(
        bytes,
        DEFAULT_XML_MAX_BYTES,
        DEFAULT_XML_MAX_NODES,
        |document| {
            parse_document_tree(
                document,
                canonical_path,
                epub_sha256,
                parser_version,
                is_navigation_document,
            )
        },
    )
    .map_err(XhtmlParseFailure::Xml)?
    .map_err(XhtmlParseFailure::Source)
}

// The validated DOM merges character data across CDATA markup. Build a second,
// bounded view solely to recover those boundaries; all element paths and source
// diagnostics continue to use the original validated document.
fn lexical_text_segments(
    original: &Document<'_>,
    canonical_path: &str,
) -> Result<HashMap<NodeId, Vec<String>>, SourceExtractionError> {
    let input = original.input_text();
    let mut boundaries = Vec::new();
    scan_cdata_content(input.as_bytes(), 0, input.len(), true, &mut boundaries);
    boundaries.sort_unstable();
    boundaries.dedup();
    if boundaries.is_empty() {
        return Ok(HashMap::new());
    }

    const SEPARATOR: &str = "<!---->";
    let virtual_len = boundaries
        .len()
        .checked_mul(SEPARATOR.len())
        .and_then(|extra| input.len().checked_add(extra))
        .filter(|length| *length <= DEFAULT_XML_MAX_BYTES * 3)
        .ok_or_else(|| SourceExtractionError::Xml {
            canonical_path: canonical_path.to_owned(),
        })?;
    let mut augmented = String::with_capacity(virtual_len);
    let mut separator_positions = HashSet::new();
    let mut previous = 0;
    for boundary in boundaries {
        augmented.push_str(&input[previous..boundary]);
        separator_positions.insert(augmented.len());
        augmented.push_str(SEPARATOR);
        previous = boundary;
    }
    augmented.push_str(&input[previous..]);

    // Each CDATA syntax takes at least 12 bytes in the validated input or its
    // bounded entity expansion. Each of its two separators adds at most a
    // comment node and a text node in this virtual view.
    let virtual_node_limit =
        DEFAULT_XML_MAX_NODES.saturating_add(((DEFAULT_XML_MAX_BYTES / 12 + 1) * 4) as u32);
    let deny_external = |_public_id: Option<&str>, _uri: &str| -> Result<Option<&str>, String> {
        Err("external entity access is disabled".to_owned())
    };
    let virtual_document = Document::parse_with_options(
        &augmented,
        ParsingOptions {
            allow_dtd: true,
            nodes_limit: virtual_node_limit,
            entity_resolver: Some(&deny_external),
        },
    )
    .map_err(|_| SourceExtractionError::Xml {
        canonical_path: canonical_path.to_owned(),
    })?;
    let mut segments = HashMap::new();
    align_text_segments(
        original.root(),
        virtual_document.root(),
        &separator_positions,
        &mut segments,
    )
    .map_err(|_| SourceExtractionError::Xml {
        canonical_path: canonical_path.to_owned(),
    })?;
    Ok(segments)
}

fn align_text_segments(
    original: Node<'_, '_>,
    virtual_node: Node<'_, '_>,
    separator_positions: &HashSet<usize>,
    segments: &mut HashMap<NodeId, Vec<String>>,
) -> Result<(), ()> {
    let mut virtual_children = virtual_node
        .children()
        .filter(|child| !(child.is_comment() && separator_positions.contains(&child.range().start)))
        .peekable();
    for source_child in original.children() {
        if source_child.is_text() {
            let text = source_child.text().unwrap_or_default();
            let mut offset = 0;
            let mut pieces = Vec::new();
            while virtual_children.peek().is_some_and(Node::is_text) {
                let child = virtual_children.next().ok_or(())?;
                let part = child.text().unwrap_or_default();
                if !text[offset..].starts_with(part) {
                    return Err(());
                }
                offset += part.len();
                pieces.push(part.to_owned());
            }
            if pieces.is_empty() || offset != text.len() {
                return Err(());
            }
            if pieces.len() > 1 {
                segments.insert(source_child.id(), pieces);
            }
            continue;
        }
        let other = virtual_children.next().ok_or(())?;
        if source_child.node_type() != other.node_type() {
            return Err(());
        }
        if source_child.is_element() {
            if source_child.tag_name() != other.tag_name()
                || !source_child
                    .attributes()
                    .map(|item| (item.namespace(), item.name(), item.value()))
                    .eq(other
                        .attributes()
                        .map(|item| (item.namespace(), item.name(), item.value())))
            {
                return Err(());
            }
            align_text_segments(source_child, other, separator_positions, segments)?;
        } else if (source_child.is_comment() && source_child.text() != other.text())
            || (source_child.is_pi()
                && source_child.pi().map(|pi| (pi.target, pi.value))
                    != other.pi().map(|pi| (pi.target, pi.value)))
        {
            return Err(());
        }
    }
    if virtual_children.next().is_some() {
        return Err(());
    }
    Ok(())
}

enum TextPieces<'a> {
    Original(Option<&'a str>),
    Split(std::slice::Iter<'a, String>),
}

impl<'a> Iterator for TextPieces<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Original(value) => value.take(),
            Self::Split(values) => values.next().map(String::as_str),
        }
    }
}

fn text_pieces<'a, 'input: 'a>(
    node: Node<'a, 'input>,
    segments: &'a HashMap<NodeId, Vec<String>>,
) -> TextPieces<'a> {
    match segments.get(&node.id()) {
        Some(pieces) => TextPieces::Split(pieces.iter()),
        None => TextPieces::Original(Some(node.text().unwrap_or_default())),
    }
}

fn scan_cdata_content(
    bytes: &[u8],
    start: usize,
    end: usize,
    allow_doctype: bool,
    boundaries: &mut Vec<usize>,
) {
    let mut cursor = start;
    while cursor < end {
        if bytes[cursor..end].starts_with(b"<!--") {
            cursor = find_sequence_end(bytes, cursor + 4, end, b"-->");
        } else if bytes[cursor..end].starts_with(b"<?") {
            cursor = find_sequence_end(bytes, cursor + 2, end, b"?>");
        } else if bytes[cursor..end].starts_with(b"<![CDATA[") {
            let close = find_sequence_end(bytes, cursor + 9, end, b"]]>");
            if close == end && !bytes[start..end].ends_with(b"]]>") {
                break;
            }
            boundaries.push(cursor);
            boundaries.push(close);
            cursor = close;
        } else if allow_doctype && bytes[cursor..end].starts_with(b"<!DOCTYPE") {
            let (after, subset) = doctype_bounds(bytes, cursor, end);
            if let Some((subset_start, subset_end)) = subset {
                scan_entity_values(bytes, subset_start, subset_end, boundaries);
            }
            cursor = after;
        } else if bytes[cursor] == b'<' {
            cursor = markup_end(bytes, cursor, end);
        } else {
            cursor += 1;
        }
    }
}

fn find_sequence_end(bytes: &[u8], start: usize, end: usize, sequence: &[u8]) -> usize {
    bytes[start.min(end)..end]
        .windows(sequence.len())
        .position(|window| window == sequence)
        .map_or(end, |offset| start + offset + sequence.len())
}

fn markup_end(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut cursor = start + 1;
    let mut quote = None;
    while cursor < end {
        let byte = bytes[cursor];
        if let Some(delimiter) = quote {
            if byte == delimiter {
                quote = None;
            }
        } else if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        } else if byte == b'>' {
            return cursor + 1;
        }
        cursor += 1;
    }
    end
}

fn doctype_bounds(bytes: &[u8], start: usize, end: usize) -> (usize, Option<(usize, usize)>) {
    let mut cursor = start + b"<!DOCTYPE".len();
    let mut quote = None;
    let mut depth = 0_usize;
    let mut subset_start = None;
    let mut subset_end = None;
    while cursor < end {
        if quote.is_none() && bytes[cursor..end].starts_with(b"<!--") {
            cursor = find_sequence_end(bytes, cursor + 4, end, b"-->");
            continue;
        }
        if quote.is_none() && bytes[cursor..end].starts_with(b"<?") {
            cursor = find_sequence_end(bytes, cursor + 2, end, b"?>");
            continue;
        }
        let byte = bytes[cursor];
        if let Some(delimiter) = quote {
            if byte == delimiter {
                quote = None;
            }
        } else {
            match byte {
                b'\'' | b'"' => quote = Some(byte),
                b'[' => {
                    if depth == 0 {
                        subset_start = Some(cursor + 1);
                    }
                    depth = depth.saturating_add(1);
                }
                b']' if depth > 0 => {
                    depth -= 1;
                    if depth == 0 {
                        subset_end = Some(cursor);
                    }
                }
                b'>' if depth == 0 => return (cursor + 1, subset_start.zip(subset_end)),
                _ => {}
            }
        }
        cursor += 1;
    }
    (end, subset_start.zip(subset_end))
}

fn scan_entity_values(bytes: &[u8], start: usize, end: usize, boundaries: &mut Vec<usize>) {
    let mut cursor = start;
    while cursor < end {
        if bytes[cursor..end].starts_with(b"<!--") {
            cursor = find_sequence_end(bytes, cursor + 4, end, b"-->");
        } else if bytes[cursor..end].starts_with(b"<?") {
            cursor = find_sequence_end(bytes, cursor + 2, end, b"?>");
        } else if bytes[cursor..end].starts_with(b"<!ENTITY") {
            let declaration_end = markup_end(bytes, cursor, end);
            let mut position = cursor + b"<!ENTITY".len();
            while position < declaration_end && bytes[position].is_ascii_whitespace() {
                position += 1;
            }
            while position < declaration_end && !bytes[position].is_ascii_whitespace() {
                position += 1;
            }
            while position < declaration_end && bytes[position].is_ascii_whitespace() {
                position += 1;
            }
            if position < declaration_end && matches!(bytes[position], b'\'' | b'"') {
                let delimiter = bytes[position];
                let value_start = position + 1;
                if let Some(offset) = bytes[value_start..declaration_end]
                    .iter()
                    .position(|byte| *byte == delimiter)
                {
                    scan_cdata_content(bytes, value_start, value_start + offset, false, boundaries);
                }
            }
            cursor = declaration_end;
        } else if bytes[cursor] == b'<' {
            cursor = markup_end(bytes, cursor, end);
        } else {
            cursor += 1;
        }
    }
}

fn parse_document_tree(
    document: &Document<'_>,
    canonical_path: &str,
    epub_sha256: [u8; 32],
    parser_version: &str,
    is_navigation_document: bool,
) -> Result<ParsedDocument, SourceExtractionError> {
    validate_xml_depth(document.root(), canonical_path)?;
    let root = document.root_element();
    if root.tag_name().name() != "html" || root.tag_name().namespace() != Some(XHTML_NAMESPACE) {
        return Err(SourceExtractionError::InvalidXhtmlRoot {
            canonical_path: canonical_path.to_owned(),
        });
    }
    let text_segments = lexical_text_segments(document, canonical_path)?;
    let id_counts = element_id_counts(root);
    let fragment_counts = element_fragment_counts(root);
    let body = root
        .descendants()
        .find(|node| is_xhtml_element(*node, "body"))
        .unwrap_or(root);
    let body_path = element_path(root, body).unwrap_or_default();
    let excluded_navigation_list = is_navigation_document
        .then(|| {
            let toc = root.descendants().find(|node| {
                is_xhtml_element(*node, "nav")
                    && node
                        .attribute((EPUB_NAMESPACE, "type"))
                        .is_some_and(|value| {
                            value.split_ascii_whitespace().any(|token| token == "toc")
                        })
            })?;
            let list = toc
                .descendants()
                .find(|node| is_xhtml_element(*node, "ol"))?;
            element_path(root, list)
        })
        .flatten();
    let mut collector = BlockCollector {
        id_counts: &id_counts,
        fragment_counts: &fragment_counts,
        text_segments: &text_segments,
        canonical_path,
        epub_sha256,
        parser_version,
        excluded_navigation_list,
        blocks: Vec::new(),
        anchors: Vec::new(),
    };
    let mut diagnostics = Vec::new();
    collect_blocks(body, &body_path, &[], &mut collector)?;
    if let Some(first) = collector
        .blocks
        .iter()
        .find(|block| !block.units.is_empty())
    {
        for fragment in element_fragments(root) {
            if fragment_counts.get(&fragment) == Some(&1) {
                collector.anchors.push(SourceAnchor {
                    canonical_path: canonical_path.to_owned(),
                    fragment,
                    locator: ElementLocator {
                        block: BlockLocator::NodePath {
                            element_ordinals: Vec::new(),
                            order: 0,
                        },
                        element_ordinals: Vec::new(),
                    },
                    block_id: first.id,
                    unit_offset: 0,
                    is_unique: true,
                });
            }
        }
    }
    for (id, count) in &id_counts {
        if *count > 1 {
            diagnostics.push(SourceDiagnostic::DuplicateXhtmlId {
                canonical_path: canonical_path.to_owned(),
                id: id.clone(),
            });
        }
    }
    Ok(ParsedDocument {
        blocks: collector.blocks,
        anchors: collector.anchors,
        diagnostics,
    })
}

struct BlockCollector<'a> {
    id_counts: &'a HashMap<String, usize>,
    fragment_counts: &'a HashMap<String, usize>,
    text_segments: &'a HashMap<NodeId, Vec<String>>,
    canonical_path: &'a str,
    epub_sha256: [u8; 32],
    parser_version: &'a str,
    excluded_navigation_list: Option<Vec<u32>>,
    blocks: Vec<SourceBlock>,
    anchors: Vec<SourceAnchor>,
}

fn collect_blocks(
    node: Node<'_, '_>,
    path: &[u32],
    inherited_metadata: &[MetadataBuilder],
    collector: &mut BlockCollector<'_>,
) -> Result<(), SourceExtractionError> {
    if collector.excluded_navigation_list.as_deref() == Some(path) {
        return Ok(());
    }
    if is_source_block(node) {
        let provisional_locator = block_locator(node, path, 0, collector.id_counts);
        let RawBlock { tokens, metadata } = collect_block_tokens(
            node,
            &provisional_locator,
            collector.fragment_counts,
            collector.canonical_path,
            inherited_metadata,
            collector.text_segments,
        )?;
        let mut segment_tokens = Vec::new();
        let mut next_segment_order = 0;
        for token in tokens {
            match token {
                Token::NestedBlock {
                    relative_path,
                    metadata: active_metadata,
                } => {
                    emit_block_segment(
                        node,
                        path,
                        std::mem::take(&mut segment_tokens),
                        &metadata,
                        &mut next_segment_order,
                        collector,
                    )?;
                    let child = node_at_path(node, &relative_path)
                        .ok_or(SourceExtractionError::InvalidIdentity)?;
                    let mut child_path = path.to_vec();
                    child_path.extend(relative_path);
                    let inherited = active_metadata
                        .iter()
                        .map(|index| {
                            let mut item = metadata[*index].clone();
                            item.rebase_to_segment = false;
                            item
                        })
                        .collect::<Vec<_>>();
                    collect_blocks(child, &child_path, &inherited, collector)?;
                }
                other => segment_tokens.push(other),
            }
        }
        emit_block_segment(
            node,
            path,
            segment_tokens,
            &metadata,
            &mut next_segment_order,
            collector,
        )?;
        return Ok(());
    }

    let mut element_ordinal = 0_u32;
    for child in node.children().filter(|child| child.is_element()) {
        let mut child_path = path.to_vec();
        child_path.push(element_ordinal);
        element_ordinal = element_ordinal.saturating_add(1);
        collect_blocks(child, &child_path, inherited_metadata, collector)?;
    }
    Ok(())
}

fn emit_block_segment(
    node: Node<'_, '_>,
    path: &[u32],
    mut tokens: Vec<Token>,
    metadata: &[MetadataBuilder],
    next_segment_order: &mut u32,
    collector: &mut BlockCollector<'_>,
) -> Result<(), SourceExtractionError> {
    if !tokens.iter().any(|token| {
        matches!(
            token,
            Token::Text { .. } | Token::Atomic { .. } | Token::Anchor { .. }
        )
    }) {
        return Ok(());
    }

    let order = *next_segment_order;
    *next_segment_order = next_segment_order.saturating_add(1);
    let locator = if order == 0 {
        block_locator(node, path, order, collector.id_counts)
    } else {
        BlockLocator::NodePath {
            element_ordinals: path.to_vec(),
            order,
        }
    };
    rebase_tokens(&mut tokens, &locator);

    let mut used_metadata = HashSet::new();
    for token in &tokens {
        match token {
            Token::Text { metadata, .. } | Token::Atomic { metadata, .. } => {
                used_metadata.extend(metadata.iter().copied());
            }
            Token::Anchor { .. } | Token::AttributeText(_) | Token::NestedBlock { .. } => {}
        }
    }
    let mut metadata_indices = HashMap::new();
    let mut segment_metadata = Vec::new();
    for (source_index, builder) in metadata.iter().enumerate() {
        if !used_metadata.contains(&source_index) {
            continue;
        }
        let mut builder = builder.clone();
        if builder.rebase_to_segment {
            builder.locator.block = locator.clone();
        }
        builder.unit_indices.clear();
        metadata_indices.insert(source_index, segment_metadata.len());
        segment_metadata.push(builder);
    }
    for token in &mut tokens {
        let source_indices = match token {
            Token::Text { metadata, .. } | Token::Atomic { metadata, .. } => metadata,
            Token::Anchor { .. } | Token::AttributeText(_) | Token::NestedBlock { .. } => continue,
        };
        for index in source_indices {
            *index = *metadata_indices
                .get(index)
                .ok_or(SourceExtractionError::InvalidIdentity)?;
        }
    }
    let mut attribute_text = Vec::new();
    let mut content_tokens = Vec::with_capacity(tokens.len());
    for token in tokens {
        match token {
            Token::AttributeText(value) => attribute_text.push(value),
            other => content_tokens.push(other),
        }
    }
    let materialized = materialize_tokens(content_tokens, segment_metadata)?;
    let record = BlockIdentityRecord {
        epub_sha256: collector.epub_sha256,
        parser_version: collector.parser_version.to_owned(),
        canonical_path: collector.canonical_path.to_owned(),
        locator: locator.clone(),
    };
    let identity: [u8; 32] = Sha256::digest(
        record
            .canonical_bytes()
            .map_err(|_| SourceExtractionError::InvalidIdentity)?,
    )
    .into();
    let id = SourceBlockId::from_bytes(identity);
    for (fragment, element_locator, unit_offset) in materialized.anchors {
        let unit_offset =
            u32::try_from(unit_offset).map_err(|_| SourceExtractionError::InvalidSourceSpan)?;
        let is_unique = collector.fragment_counts.get(&fragment).copied() == Some(1);
        collector.anchors.push(SourceAnchor {
            canonical_path: collector.canonical_path.to_owned(),
            fragment,
            locator: element_locator,
            block_id: id,
            unit_offset,
            is_unique,
        });
    }
    let content_hash = hash_block_content(
        &materialized.units,
        &materialized.inline_metadata,
        &attribute_text,
    );
    collector.blocks.push(SourceBlock {
        id,
        canonical_path: collector.canonical_path.to_owned(),
        element_name: node.tag_name().name().to_ascii_lowercase(),
        locator,
        parser_version: collector.parser_version.to_owned(),
        content_hash,
        units: materialized.units,
        inline_metadata: materialized.inline_metadata,
        attribute_text,
        protected_resource_hashes: Vec::new(),
    });
    Ok(())
}

fn rebase_tokens(tokens: &mut [Token], block: &BlockLocator) {
    for token in tokens {
        match token {
            Token::Text { locator, .. } => locator.block = block.clone(),
            Token::Atomic { unit, spans, .. } => {
                for span in spans {
                    *span = span.with_block_locator(block.clone());
                }
                if let ProtectedSourceUnit::Ruby(ruby) = unit {
                    for span in ruby.base_spans.iter_mut().chain(&mut ruby.annotation_spans) {
                        *span = span.with_block_locator(block.clone());
                    }
                }
            }
            Token::Anchor { locator, .. } => locator.block = block.clone(),
            Token::AttributeText(value) => value.locator.block = block.clone(),
            Token::NestedBlock { .. } => {}
        }
    }
}

fn node_at_path<'doc, 'input>(
    root: Node<'doc, 'input>,
    path: &[u32],
) -> Option<Node<'doc, 'input>> {
    let mut current = root;
    for ordinal in path {
        current = current
            .children()
            .filter(|child| child.is_element())
            .nth(*ordinal as usize)?;
    }
    Some(current)
}
fn is_source_block(node: Node<'_, '_>) -> bool {
    if !node.is_element() || node.tag_name().namespace() != Some(XHTML_NAMESPACE) {
        return false;
    }
    matches!(
        node.tag_name().name(),
        "body"
            | "address"
            | "article"
            | "aside"
            | "blockquote"
            | "caption"
            | "dd"
            | "div"
            | "dl"
            | "dt"
            | "figcaption"
            | "figure"
            | "footer"
            | "header"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "li"
            | "main"
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "table"
            | "tbody"
            | "td"
            | "tfoot"
            | "th"
            | "thead"
            | "tr"
            | "ul"
    )
}

fn is_nested_source_block(node: Node<'_, '_>, block_root: Node<'_, '_>) -> bool {
    node != block_root && is_source_block(node)
}

fn block_locator(
    node: Node<'_, '_>,
    path: &[u32],
    order: u32,
    id_counts: &HashMap<String, usize>,
) -> BlockLocator {
    node.attribute("id")
        .filter(|id| !id.is_empty() && id_counts.get(*id) == Some(&1))
        .map(|id| BlockLocator::UniqueXhtmlId(id.to_owned()))
        .unwrap_or_else(|| BlockLocator::NodePath {
            element_ordinals: path.to_vec(),
            order,
        })
}

fn element_id_counts(root: Node<'_, '_>) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for node in root.descendants().filter(|node| node.is_element()) {
        if let Some(id) = node.attribute("id").filter(|id| !id.is_empty()) {
            *counts.entry(id.to_owned()).or_insert(0) += 1;
        }
    }
    counts
}

// Match inventory: count each id and XHTML a@name, but count equal
// id and name on the same element only once.
fn element_fragments(node: Node<'_, '_>) -> Vec<String> {
    let id = node.attribute("id").filter(|value| !value.is_empty());
    let name = is_xhtml_element(node, "a")
        .then(|| node.attribute("name"))
        .flatten()
        .filter(|value| !value.is_empty() && Some(*value) != id);
    id.into_iter().chain(name).map(str::to_owned).collect()
}

fn element_fragment_counts(root: Node<'_, '_>) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for node in root.descendants().filter(|node| node.is_element()) {
        for fragment in element_fragments(node) {
            *counts.entry(fragment).or_insert(0) += 1;
        }
    }
    counts
}

fn element_path(root: Node<'_, '_>, target: Node<'_, '_>) -> Option<Vec<u32>> {
    fn visit(node: Node<'_, '_>, target: Node<'_, '_>, path: &mut Vec<u32>) -> bool {
        if node == target {
            return true;
        }
        let mut ordinal = 0_u32;
        for child in node.children().filter(|child| child.is_element()) {
            path.push(ordinal);
            ordinal = ordinal.saturating_add(1);
            if visit(child, target, path) {
                return true;
            }
            path.pop();
        }
        false
    }
    let mut path = Vec::new();
    visit(root, target, &mut path).then_some(path)
}

struct RawBlock {
    tokens: Vec<Token>,
    metadata: Vec<MetadataBuilder>,
}

enum Token {
    Text {
        text: String,
        locator: TextNodeLocator,
        metadata: Vec<usize>,
    },
    Atomic {
        unit: ProtectedSourceUnit,
        spans: Vec<SourceSpan>,
        metadata: Vec<usize>,
    },
    Anchor {
        id: String,
        locator: ElementLocator,
    },
    AttributeText(AttributeSourceText),
    NestedBlock {
        relative_path: Vec<u32>,
        metadata: Vec<usize>,
    },
}

#[derive(Clone)]
struct MetadataBuilder {
    locator: ElementLocator,
    kind: InlineMetadataKind,
    unit_indices: Vec<usize>,
    rebase_to_segment: bool,
}

struct MaterializedBlock {
    units: Vec<SourceUnit>,
    inline_metadata: Vec<InlineSourceMetadata>,
    anchors: Vec<(String, ElementLocator, usize)>,
}

fn collect_block_tokens(
    block_root: Node<'_, '_>,
    block_locator: &BlockLocator,
    fragment_counts: &HashMap<String, usize>,
    canonical_path: &str,
    inherited_metadata: &[MetadataBuilder],
    text_segments: &HashMap<NodeId, Vec<String>>,
) -> Result<RawBlock, SourceExtractionError> {
    let mut raw = RawBlock {
        tokens: Vec::new(),
        metadata: inherited_metadata.to_vec(),
    };
    let mut text_node_order = 0_u32;
    let mut active_metadata = (0..inherited_metadata.len()).collect();
    let mut image_occurrence = 0_usize;
    let mut state = ContentWalkState {
        block_root,
        block_locator,
        fragment_counts,
        text_segments,
        canonical_path,
        text_node_order: &mut text_node_order,
        image_occurrence: &mut image_occurrence,
        active_metadata: &mut active_metadata,
        output: &mut raw,
    };
    walk_content(block_root, &[], &mut state)?;
    Ok(raw)
}

struct ContentWalkState<'doc, 'input, 'data> {
    block_root: Node<'doc, 'input>,
    block_locator: &'data BlockLocator,
    fragment_counts: &'data HashMap<String, usize>,
    text_segments: &'data HashMap<NodeId, Vec<String>>,
    canonical_path: &'data str,
    text_node_order: &'data mut u32,
    image_occurrence: &'data mut usize,
    active_metadata: &'data mut Vec<usize>,
    output: &'data mut RawBlock,
}

fn walk_content(
    node: Node<'_, '_>,
    relative_path: &[u32],
    state: &mut ContentWalkState<'_, '_, '_>,
) -> Result<(), SourceExtractionError> {
    if node.is_text() {
        for text in text_pieces(node, state.text_segments) {
            let current_order = *state.text_node_order;
            *state.text_node_order = state.text_node_order.saturating_add(1);
            if !text.is_empty() {
                let locator = TextNodeLocator {
                    block: state.block_locator.clone(),
                    text_node_order: current_order,
                };
                state.output.tokens.push(Token::Text {
                    text: text.to_owned(),
                    locator,
                    metadata: state.active_metadata.clone(),
                });
            }
        }
        return Ok(());
    }
    if !node.is_element() {
        return Ok(());
    }
    if is_nested_source_block(node, state.block_root) {
        count_text_descendants(node, state.text_node_order, state.text_segments);
        state.output.tokens.push(Token::NestedBlock {
            relative_path: relative_path.to_vec(),
            metadata: state.active_metadata.clone(),
        });
        return Ok(());
    }

    let tag = node.tag_name().name();
    if matches!(tag, "script" | "style") {
        count_text_descendants(node, state.text_node_order, state.text_segments);
        return Ok(());
    }

    let element_locator = ElementLocator {
        block: state.block_locator.clone(),
        element_ordinals: if node == state.block_root {
            Vec::new()
        } else {
            relative_path.to_vec()
        },
    };
    for fragment in element_fragments(node) {
        if state.fragment_counts.get(&fragment) == Some(&1) {
            state.output.tokens.push(Token::Anchor {
                id: fragment,
                locator: element_locator.clone(),
            });
        }
    }

    if tag == "ruby" {
        let mut ruby_metadata = state.active_metadata.clone();
        let mut ruby_path = relative_path.to_vec();
        collect_ruby_descendant_evidence(node, &mut ruby_path, state, &mut ruby_metadata);
        let (base, annotation, base_spans, annotation_spans) = collect_ruby(
            node,
            state.block_locator,
            state.text_node_order,
            state.text_segments,
        )?;
        let spans = base_spans
            .iter()
            .chain(&annotation_spans)
            .cloned()
            .collect();
        state.output.tokens.push(Token::Atomic {
            unit: ProtectedSourceUnit::Ruby(RubySource {
                base,
                annotation,
                base_spans,
                annotation_spans,
                node_path: element_locator.element_ordinals,
            }),
            spans,
            metadata: ruby_metadata,
        });
        return Ok(());
    }
    let is_xhtml_image = tag == "img" && node.tag_name().namespace() == Some(XHTML_NAMESPACE);
    let is_svg_image = tag == "image" && node.tag_name().namespace() == Some(SVG_NAMESPACE);
    if is_xhtml_image || is_svg_image {
        let raw_source = if is_xhtml_image {
            node.attribute("src")
        } else {
            node.attribute("href")
                .or_else(|| node.attribute((XLINK_NAMESPACE, "href")))
        }
        .map(str::to_owned);
        let resolved = raw_source
            .as_deref()
            .map(|source| resolve_href(state.canonical_path, source));
        let (resource_path, external) = match resolved {
            Some(Ok(resolved)) => (resolved.path, resolved.external),
            Some(Err(_)) => (None, false),
            None => (None, false),
        };
        let alt_attribute = if is_xhtml_image { "alt" } else { "aria-label" };
        let alt = node.attribute(alt_attribute).map(str::to_owned);
        if let Some(value) = &alt {
            state
                .output
                .tokens
                .push(Token::AttributeText(AttributeSourceText {
                    locator: element_locator.clone(),
                    attribute: alt_attribute.to_owned(),
                    value: value.clone(),
                    resource_path: resource_path.clone(),
                }));
        }
        let occurrence = *state.image_occurrence;
        *state.image_occurrence = state.image_occurrence.saturating_add(1);
        state.output.tokens.push(Token::Atomic {
            unit: ProtectedSourceUnit::Image(ImageSource {
                raw_source,
                resource_path,
                alt,
                occurrence,
                node_path: element_locator.element_ordinals,
                external,
            }),
            spans: Vec::new(),
            metadata: state.active_metadata.clone(),
        });
        return Ok(());
    }
    if tag == "br" || tag == "hr" {
        let unit = if tag == "br" {
            ProtectedSourceUnit::LineBreak {
                element_path: element_locator.element_ordinals,
            }
        } else {
            ProtectedSourceUnit::SceneBreak {
                element_path: element_locator.element_ordinals,
            }
        };
        state.output.tokens.push(Token::Atomic {
            unit,
            spans: Vec::new(),
            metadata: state.active_metadata.clone(),
        });
        return Ok(());
    }

    let metadata_index = metadata_for_element(node, &element_locator, state.output);
    if let Some(index) = metadata_index {
        state.active_metadata.push(index);
    }
    let mut element_ordinal = 0_u32;
    for child in node.children() {
        if child.is_element() {
            let mut child_path = if node == state.block_root {
                Vec::new()
            } else {
                relative_path.to_vec()
            };
            child_path.push(element_ordinal);
            element_ordinal = element_ordinal.saturating_add(1);
            walk_content(child, &child_path, state)?;
        } else {
            walk_content(child, relative_path, state)?;
        }
    }
    if metadata_index.is_some() {
        state.active_metadata.pop();
    }
    Ok(())
}

fn collect_ruby_descendant_evidence(
    parent: Node<'_, '_>,
    path: &mut Vec<u32>,
    state: &mut ContentWalkState<'_, '_, '_>,
    metadata: &mut Vec<usize>,
) {
    let mut ordinal = 0_u32;
    for child in parent.children().filter(Node::is_element) {
        path.push(ordinal);
        ordinal = ordinal.saturating_add(1);
        let locator = ElementLocator {
            block: state.block_locator.clone(),
            element_ordinals: path.clone(),
        };
        for fragment in element_fragments(child) {
            if state.fragment_counts.get(&fragment) == Some(&1) {
                state.output.tokens.push(Token::Anchor {
                    id: fragment,
                    locator: locator.clone(),
                });
            }
        }
        if let Some(index) = metadata_for_element(child, &locator, state.output) {
            metadata.push(index);
        }
        collect_ruby_descendant_evidence(child, path, state, metadata);
        path.pop();
    }
}

fn count_text_descendants(
    node: Node<'_, '_>,
    text_node_order: &mut u32,
    text_segments: &HashMap<NodeId, Vec<String>>,
) {
    for descendant in node.descendants().filter(|node| node.is_text()) {
        let count = text_segments.get(&descendant.id()).map_or(1, Vec::len);
        *text_node_order = text_node_order.saturating_add(count as u32);
    }
}

fn collect_ruby(
    ruby: Node<'_, '_>,
    block_locator: &BlockLocator,
    text_node_order: &mut u32,
    text_segments: &HashMap<NodeId, Vec<String>>,
) -> Result<(String, String, Vec<SourceSpan>, Vec<SourceSpan>), SourceExtractionError> {
    struct RubyCollector<'a> {
        block_locator: &'a BlockLocator,
        text_node_order: &'a mut u32,
        text_segments: &'a HashMap<NodeId, Vec<String>>,
        base: String,
        annotation: String,
        base_spans: Vec<SourceSpan>,
        annotation_spans: Vec<SourceSpan>,
    }

    impl RubyCollector<'_> {
        fn visit(
            &mut self,
            node: Node<'_, '_>,
            in_annotation: bool,
            in_parentheses: bool,
        ) -> Result<(), SourceExtractionError> {
            if node.is_text() {
                for text in text_pieces(node, self.text_segments) {
                    let order = *self.text_node_order;
                    *self.text_node_order = self.text_node_order.saturating_add(1);
                    if !in_parentheses {
                        if in_annotation {
                            self.annotation.push_str(text);
                        } else {
                            self.base.push_str(text);
                        }
                        if !text.is_empty() {
                            let locator = TextNodeLocator {
                                block: self.block_locator.clone(),
                                text_node_order: order,
                            };
                            let span = SourceSpan::new(locator, 0, text.len(), text)
                                .map_err(|_| SourceExtractionError::InvalidSourceSpan)?;
                            if in_annotation {
                                self.annotation_spans.push(span);
                            } else {
                                self.base_spans.push(span);
                            }
                        }
                    }
                }
                return Ok(());
            }
            if node.is_element() && is_xhtml_element(node, "rp") {
                for child in node.children() {
                    self.visit(child, in_annotation, true)?;
                }
                return Ok(());
            }
            let is_annotation =
                in_annotation || (node.is_element() && is_xhtml_element(node, "rt"));
            for child in node.children() {
                self.visit(child, is_annotation, in_parentheses)?;
            }
            Ok(())
        }
    }

    let mut collector = RubyCollector {
        block_locator,
        text_node_order,
        text_segments,
        base: String::new(),
        annotation: String::new(),
        base_spans: Vec::new(),
        annotation_spans: Vec::new(),
    };
    collector.visit(ruby, false, false)?;
    Ok((
        collector.base,
        collector.annotation,
        collector.base_spans,
        collector.annotation_spans,
    ))
}

fn metadata_for_element(
    node: Node<'_, '_>,
    locator: &ElementLocator,
    output: &mut RawBlock,
) -> Option<usize> {
    let tag = node.tag_name().name();
    let kind = if tag == "a" {
        let href = node.attribute("href").map(str::to_owned);
        let epub_type = node.attribute((EPUB_NAMESPACE, "type")).unwrap_or_default();
        if epub_type.split_whitespace().any(|kind| kind == "noteref") {
            InlineMetadataKind::NoteReference { href }
        } else {
            InlineMetadataKind::Link { href }
        }
    } else if matches!(
        tag,
        "b" | "em" | "i" | "s" | "small" | "span" | "strong" | "sub" | "sup" | "u"
    ) {
        InlineMetadataKind::Formatting {
            element: tag.to_owned(),
        }
    } else {
        return None;
    };
    let index = output.metadata.len();
    output.metadata.push(MetadataBuilder {
        locator: locator.clone(),
        kind,
        unit_indices: Vec::new(),
        rebase_to_segment: true,
    });
    Some(index)
}

fn materialize_tokens(
    tokens: Vec<Token>,
    mut metadata: Vec<MetadataBuilder>,
) -> Result<MaterializedBlock, SourceExtractionError> {
    let mut units = Vec::new();
    let mut anchors = Vec::new();
    let mut cursor = 0;
    while cursor < tokens.len() {
        match &tokens[cursor] {
            Token::Atomic { .. } => {
                let Token::Atomic {
                    unit,
                    spans,
                    metadata: metadata_ids,
                } = &tokens[cursor]
                else {
                    unreachable!()
                };
                let unit_index = units.len();
                for index in metadata_ids {
                    if let Some(builder) = metadata.get_mut(*index) {
                        builder.unit_indices.push(unit_index);
                    }
                }
                units.push(SourceUnit::protected(unit.clone(), spans.clone()));
                cursor += 1;
            }
            Token::AttributeText(_) | Token::NestedBlock { .. } => unreachable!(),
            Token::Text { .. } | Token::Anchor { .. } => {
                let run_start = cursor;
                let mut run_end = cursor;
                while run_end < tokens.len()
                    && matches!(tokens[run_end], Token::Text { .. } | Token::Anchor { .. })
                {
                    run_end += 1;
                }
                let unit_start = units.len();
                let mut text = String::new();
                let mut segments = Vec::<(usize, usize, TextNodeLocator, Vec<usize>)>::new();
                let mut pending_anchors = Vec::<(usize, String, ElementLocator)>::new();
                for token in &tokens[run_start..run_end] {
                    match token {
                        Token::Text {
                            text: value,
                            locator,
                            metadata,
                        } => {
                            let start = text.len();
                            text.push_str(value);
                            segments.push((start, text.len(), locator.clone(), metadata.clone()));
                        }
                        Token::Anchor { id, locator } => {
                            pending_anchors.push((text.len(), id.clone(), locator.clone()));
                        }
                        Token::Atomic { .. }
                        | Token::AttributeText(_)
                        | Token::NestedBlock { .. } => unreachable!(),
                    }
                }
                let graphemes: Vec<_> = text.grapheme_indices(true).collect();
                let mut segment_cursor = 0;
                for grapheme in &graphemes {
                    let start = grapheme.0;
                    let end = start + grapheme.1.len();
                    while segment_cursor < segments.len() && segments[segment_cursor].1 <= start {
                        segment_cursor += 1;
                    }
                    let unit_index = units.len();
                    let mut spans = Vec::new();
                    let mut metadata_ids = HashSet::new();
                    for (segment_start, segment_end, locator, segment_metadata) in
                        &segments[segment_cursor..]
                    {
                        if *segment_start >= end {
                            break;
                        }
                        let overlap_start = start.max(*segment_start);
                        let overlap_end = end.min(*segment_end);
                        if overlap_start < overlap_end {
                            let node_text = &text[*segment_start..*segment_end];
                            let local_start = overlap_start - segment_start;
                            let local_end = overlap_end - segment_start;
                            spans.push(
                                SourceSpan::new(locator.clone(), local_start, local_end, node_text)
                                    .map_err(|_| SourceExtractionError::InvalidSourceSpan)?,
                            );
                            metadata_ids.extend(segment_metadata.iter().copied());
                        }
                    }
                    for index in metadata_ids {
                        if let Some(builder) = metadata.get_mut(index) {
                            builder.unit_indices.push(unit_index);
                        }
                    }
                    units.push(SourceUnit::grapheme(grapheme.1.to_owned(), spans));
                }
                for (byte_offset, id, locator) in pending_anchors {
                    let unit_offset = byte_offset_to_unit_offset(&graphemes, byte_offset);
                    anchors.push((id, locator, unit_start + unit_offset));
                }
                cursor = run_end;
            }
        }
    }

    let inline_metadata = metadata
        .into_iter()
        .map(|builder| InlineSourceMetadata {
            locator: builder.locator,
            kind: builder.kind,
            ranges: compress_unit_indices(builder.unit_indices, units.len()),
        })
        .collect();
    Ok(MaterializedBlock {
        units,
        inline_metadata,
        anchors,
    })
}

fn byte_offset_to_unit_offset(graphemes: &[(usize, &str)], byte_offset: usize) -> usize {
    if byte_offset == 0 {
        return 0;
    }
    let index = graphemes.partition_point(|(start, value)| start + value.len() < byte_offset);
    if let Some((start, value)) = graphemes.get(index) {
        return index + usize::from(byte_offset == start + value.len());
    }
    graphemes.len()
}

fn compress_unit_indices(mut indices: Vec<usize>, unit_count: usize) -> Vec<UnitRange> {
    indices.sort_unstable();
    indices.dedup();
    let mut output = Vec::new();
    let mut start = None;
    let mut previous = 0;
    for index in indices {
        if index >= unit_count {
            continue;
        }
        match start {
            None => {
                start = Some(index);
                previous = index;
            }
            Some(_) if index == previous + 1 => previous = index,
            Some(begin) => {
                output.push(UnitRange {
                    start: begin as u32,
                    end: (previous + 1) as u32,
                });
                start = Some(index);
                previous = index;
            }
        }
    }
    if let Some(begin) = start {
        output.push(UnitRange {
            start: begin as u32,
            end: (previous + 1) as u32,
        });
    }
    output
}

fn hash_block_content(
    units: &[SourceUnit],
    inline_metadata: &[InlineSourceMetadata],
    attribute_text: &[AttributeSourceText],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"NT-SOURCE-CONTENT\0");
    hash.update(1_u16.to_be_bytes());
    for unit in units {
        match &unit.kind {
            SourceUnitKind::Grapheme(text) => {
                hash.update([0x01]);
                hash_bytes(&mut hash, text.as_bytes());
            }
            SourceUnitKind::Protected(ProtectedSourceUnit::Ruby(ruby)) => {
                hash.update([0x02]);
                hash_bytes(&mut hash, ruby.base.as_bytes());
                hash_bytes(&mut hash, ruby.annotation.as_bytes());
            }
            SourceUnitKind::Protected(ProtectedSourceUnit::Image(image)) => {
                hash.update([0x03]);
                hash_optional_bytes(&mut hash, image.resource_path.as_deref().map(str::as_bytes));
                hash_optional_bytes(&mut hash, image.raw_source.as_deref().map(str::as_bytes));
                hash_optional_bytes(&mut hash, image.alt.as_deref().map(str::as_bytes));
            }
            SourceUnitKind::Protected(ProtectedSourceUnit::MarkerPair(pair)) => {
                hash.update([0x04]);
                hash_bytes(&mut hash, pair.kind.as_bytes());
                hash_endpoint(&mut hash, &pair.start.node_path, &pair.start.attributes);
                hash_endpoint(&mut hash, &pair.end.node_path, &pair.end.attributes);
            }
            SourceUnitKind::Protected(ProtectedSourceUnit::LineBreak { .. }) => {
                hash.update([0x05]);
            }
            SourceUnitKind::Protected(ProtectedSourceUnit::SceneBreak { .. }) => {
                hash.update([0x06]);
            }
        }
    }
    for metadata in inline_metadata {
        hash.update([0x10]);
        match &metadata.kind {
            InlineMetadataKind::Formatting { element } => {
                hash.update([0x01]);
                hash_bytes(&mut hash, element.as_bytes());
            }
            InlineMetadataKind::Link { href } => {
                hash.update([0x02]);
                hash_optional_bytes(&mut hash, href.as_deref().map(str::as_bytes));
            }
            InlineMetadataKind::NoteReference { href } => {
                hash.update([0x03]);
                hash_optional_bytes(&mut hash, href.as_deref().map(str::as_bytes));
            }
        }
        for range in &metadata.ranges {
            hash.update(range.start.to_be_bytes());
            hash.update(range.end.to_be_bytes());
        }
    }
    for attribute in attribute_text {
        hash.update([0x20]);
        hash_bytes(&mut hash, attribute.attribute.as_bytes());
        hash_bytes(&mut hash, attribute.value.as_bytes());
        hash_optional_bytes(
            &mut hash,
            attribute.resource_path.as_deref().map(str::as_bytes),
        );
    }
    hash.finalize().into()
}

fn hash_endpoint(hash: &mut Sha256, path: &[u32], attributes: &[(String, String)]) {
    hash.update((path.len() as u32).to_be_bytes());
    for ordinal in path {
        hash.update(ordinal.to_be_bytes());
    }
    for (name, value) in attributes {
        hash_bytes(hash, name.as_bytes());
        hash_bytes(hash, value.as_bytes());
    }
}

fn hash_optional_bytes(hash: &mut Sha256, value: Option<&[u8]>) {
    match value {
        Some(value) => {
            hash.update([1]);
            hash_bytes(hash, value);
        }
        None => {
            hash.update([0]);
        }
    }
}

fn hash_bytes(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_be_bytes());
    hash.update(value);
}

fn is_xhtml_element(node: Node<'_, '_>, name: &str) -> bool {
    node.is_element()
        && node.tag_name().name() == name
        && node.tag_name().namespace() == Some(XHTML_NAMESPACE)
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use nt_domain::source_range::{BlockLocator, ProtectedSourceUnit, SourceUnitKind};
    use sha2::Digest;
    use zip::write::{SimpleFileOptions, ZipWriter};

    use super::{CoverageDomain, extract_source};
    use crate::archive::EpubZip;
    use crate::inventory::inspect;

    const CONTAINER: &str = r#"<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OPS/book.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#;
    const PACKAGE: &str = r#"<package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/" version="3.0" unique-identifier="uid"><metadata><dc:identifier id="uid">source-test</dc:identifier><dc:title>Source test</dc:title><dc:language>ja</dc:language><dc:creator>Author</dc:creator></metadata><manifest><item id="chapter" href="text/chapter.xhtml" media-type="application/xhtml+xml"/><item id="appendix" href="appendix.xhtml" media-type="application/xhtml+xml"/><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="picture" href="img/picture.png" media-type="image/png"/></manifest><spine><itemref idref="chapter"/></spine></package>"#;
    const NAV: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="text/chapter.xhtml#start">Chapter</a></li><li><a href="../../outside.xhtml#bad">Broken</a></li></ol></nav></body></html>"#;
    const CHAPTER: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><p id="start">ab<span>cd</span></p><p id="duplicate">same</p><p id="duplicate">same</p><p id="cross">e<em>&#x301;</em></p><p id="protected"><ruby><rb>漢</rb><rt>かん</rt><rp>(</rp><rp>)</rp></ruby><img src="../img/picture.png" alt="cover"/><br/><hr/></p><script>not visible</script><style>also hidden</style><p id="emoji">👩‍👩‍👧‍👦 &amp; &#x1f600;</p></body></html>"#;
    const APPENDIX: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="aux">Appendix.</p></body></html>"#;

    fn epub_bytes() -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let stored =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (path, bytes) in [
            ("mimetype", "application/epub+zip".as_bytes()),
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OPS/book.opf", PACKAGE.as_bytes()),
            ("OPS/nav.xhtml", NAV.as_bytes()),
            ("OPS/text/chapter.xhtml", CHAPTER.as_bytes()),
            ("OPS/appendix.xhtml", APPENDIX.as_bytes()),
            ("OPS/img/picture.png", b"fixture-image".as_slice()),
        ] {
            writer.start_file(path, stored).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    fn extract(bytes: &[u8], epub_hash: [u8; 32], parser_version: &str) -> super::SourceExtraction {
        let mut archive = EpubZip::new(Cursor::new(bytes.to_vec())).unwrap();
        let inventory = inspect(&mut archive);
        extract_source(&mut archive, &inventory, epub_hash, parser_version).unwrap()
    }

    fn block<'a>(
        document: &'a super::SourceDocument,
        id: &str,
    ) -> &'a nt_domain::source_range::SourceBlock {
        document
            .blocks
            .iter()
            .find(|block| block.locator == BlockLocator::UniqueXhtmlId(id.to_owned()))
            .unwrap()
    }

    #[test]
    fn extracts_real_epub_text_graphemes_spans_protected_units_and_auxiliary_scope() {
        let bytes = epub_bytes();
        let result = extract(&bytes, [7; 32], super::PARSER_VERSION);
        assert_eq!(result.navigation.len(), 1);
        assert_eq!(
            result.navigation[0].state,
            nt_domain::chapter_map::NavigationTreeState::Partial
        );
        assert_eq!(
            result.navigation[0].items[0].raw_href.as_deref(),
            Some("text/chapter.xhtml#start")
        );
        assert!(matches!(
            result.navigation[0].items[1].target,
            Some(nt_domain::chapter_map::NavigationTarget::Invalid { .. })
        ));
        assert_eq!(
            result.navigation[0].items[1].raw_href.as_deref(),
            Some("../../outside.xhtml#bad")
        );
        let map_input = result.chapter_map_input(Vec::new());
        assert_eq!(map_input.documents.len(), 3);
        assert_eq!(
            map_input.navigation[0].items[0].raw_href.as_deref(),
            Some("text/chapter.xhtml#start")
        );
        assert!(
            map_input
                .anchors
                .iter()
                .any(|anchor| anchor.fragment == "start" && anchor.is_unique)
        );
        assert!(
            !result.complete,
            "unsupported scripted content must stay visible as incomplete"
        );
        assert!(
            result
                .diagnostics
                .contains(&super::SourceDiagnostic::InventoryPartial)
        );

        let chapter = result
            .documents
            .iter()
            .find(|document| document.canonical_path == "OPS/text/chapter.xhtml")
            .unwrap();
        assert_eq!(chapter.domain, CoverageDomain::MainFlow);
        let text = block(chapter, "start");
        assert_eq!(
            text.units
                .iter()
                .map(|unit| match &unit.kind {
                    SourceUnitKind::Grapheme(value) => value.as_str(),
                    SourceUnitKind::Protected(_) => panic!("unexpected protected source unit"),
                })
                .collect::<String>(),
            "abcd"
        );
        assert_eq!(text.units[0].spans[0].node().text_node_order, 0);
        assert_eq!(text.units[0].spans[0].start_byte(), 0);
        assert_eq!(text.units[3].spans[0].node().text_node_order, 1);
        assert_eq!(text.units[3].spans[0].start_byte(), 1);

        let cross = block(chapter, "cross");
        assert_eq!(cross.units.len(), 1);
        assert_eq!(
            cross.units[0].kind,
            SourceUnitKind::Grapheme("e\u{301}".to_owned())
        );
        assert_eq!(cross.units[0].spans.len(), 2);
        assert_eq!(cross.inline_metadata.len(), 1);
        assert_eq!(
            cross.inline_metadata[0].ranges[0],
            nt_domain::source_range::UnitRange { start: 0, end: 1 }
        );

        let emoji = block(chapter, "emoji");
        assert_eq!(
            emoji
                .units
                .iter()
                .map(|unit| match &unit.kind {
                    SourceUnitKind::Grapheme(value) => value.as_str(),
                    SourceUnitKind::Protected(_) =>
                        panic!("unexpected protected unit in emoji fixture"),
                })
                .collect::<Vec<_>>(),
            vec!["👩‍👩‍👧‍👦", " ", "&", " ", "😀"]
        );
        assert_eq!(emoji.units[2].spans[0].start_byte(), "👩‍👩‍👧‍👦 ".len() as u32);
        assert_eq!(emoji.units[2].spans[0].end_byte(), "👩‍👩‍👧‍👦 &".len() as u32);

        let protected = block(chapter, "protected");
        assert_eq!(protected.units.len(), 4);
        let SourceUnitKind::Protected(ProtectedSourceUnit::Ruby(ruby)) = &protected.units[0].kind
        else {
            panic!("ruby must remain one protected unit");
        };
        assert_eq!(ruby.base, "漢");
        assert_eq!(ruby.annotation, "かん");
        assert_eq!(ruby.base_spans.len(), 1);
        assert_eq!(ruby.annotation_spans.len(), 1);
        let SourceUnitKind::Protected(ProtectedSourceUnit::Image(image)) = &protected.units[1].kind
        else {
            panic!("image must remain one protected unit");
        };
        assert_eq!(image.resource_path.as_deref(), Some("OPS/img/picture.png"));
        assert_eq!(image.alt.as_deref(), Some("cover"));
        assert_eq!(protected.attribute_text[0].attribute, "alt");
        assert_eq!(protected.protected_resource_hashes.len(), 1);
        let expected_image_hash: [u8; 32] = sha2::Sha256::digest(b"fixture-image").into();
        assert_eq!(
            protected.protected_resource_hashes[0].sha256,
            expected_image_hash
        );
        assert!(matches!(
            protected.units[2].kind,
            SourceUnitKind::Protected(ProtectedSourceUnit::LineBreak { .. })
        ));
        assert!(matches!(
            protected.units[3].kind,
            SourceUnitKind::Protected(ProtectedSourceUnit::SceneBreak { .. })
        ));

        let duplicate_blocks: Vec<_> = chapter
            .blocks
            .iter()
            .filter(|candidate| {
                candidate
                    .units
                    .iter()
                    .any(|unit| unit.kind == SourceUnitKind::Grapheme("s".to_owned()))
            })
            .collect();
        assert_eq!(duplicate_blocks.len(), 2);
        assert_ne!(duplicate_blocks[0].id, duplicate_blocks[1].id);
        assert!(
            duplicate_blocks
                .iter()
                .all(|candidate| matches!(candidate.locator, BlockLocator::NodePath { .. }))
        );
        assert!(
            !result
                .anchors
                .iter()
                .any(|anchor| anchor.fragment == "duplicate")
        );
        assert!(
            result
                .anchors
                .iter()
                .any(|anchor| anchor.fragment == "start" && anchor.unit_offset == 0)
        );

        let auxiliary = result
            .documents
            .iter()
            .find(|document| document.canonical_path == "OPS/appendix.xhtml")
            .unwrap();
        assert_eq!(auxiliary.domain, CoverageDomain::Auxiliary);
        assert_eq!(block(auxiliary, "aux").units.len(), 9);
        let nav_document = result
            .documents
            .iter()
            .find(|document| document.canonical_path == "OPS/nav.xhtml")
            .unwrap();
        assert_eq!(nav_document.domain, CoverageDomain::Auxiliary);
        assert!(nav_document.blocks.is_empty());
    }

    #[test]
    fn computes_the_accepted_block_id_hash_vector() {
        use nt_domain::source_range::{BlockIdentityRecord, BlockLocator, SourceBlockId};
        use sha2::{Digest, Sha256};

        let record = BlockIdentityRecord {
            epub_sha256: core::array::from_fn(|index| index as u8),
            parser_version: "nt-epub/1.2.0".to_owned(),
            canonical_path: "EPUB/text/ch01.xhtml".to_owned(),
            locator: BlockLocator::UniqueXhtmlId("p-1".to_owned()),
        };
        let digest: [u8; 32] = Sha256::digest(record.canonical_bytes().unwrap()).into();
        assert_eq!(
            SourceBlockId::from_bytes(digest).to_string(),
            "12979669a4ef112098cad9666389dca1db8c32003ecfe29071ef555f708e3520"
        );
    }
    #[test]
    fn namespaces_ids_by_snapshot_hash_and_parser_version() {
        let bytes = epub_bytes();
        let first = extract(&bytes, [7; 32], super::PARSER_VERSION);
        let repeated = extract(&bytes, [7; 32], super::PARSER_VERSION);
        let changed_snapshot = extract(&bytes, [8; 32], super::PARSER_VERSION);
        let changed_parser = extract(&bytes, [7; 32], "nt-epub/1.2.1");
        let id = |result: &super::SourceExtraction| {
            block(
                result
                    .documents
                    .iter()
                    .find(|document| document.canonical_path == "OPS/text/chapter.xhtml")
                    .unwrap(),
                "start",
            )
            .id
        };
        assert_eq!(id(&first), id(&repeated));
        assert_ne!(id(&first), id(&changed_snapshot));
        assert_ne!(id(&first), id(&changed_parser));
    }
}
