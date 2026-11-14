use std::collections::HashMap;

use roxmltree::{Document, Node};

use crate::inventory::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, MetadataValue, ResourceEncryption,
};
use crate::xml::{XmlFailure, node_position, with_xml_document};

const OPF_NAMESPACE: &str = "http://www.idpf.org/2007/opf";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ManifestItem {
    pub id: String,
    pub href: String,
    pub media_type: String,
    pub properties: Vec<String>,
    pub fallback: Option<String>,
    pub media_overlay: Option<String>,
    pub path: Option<String>,
    pub available: bool,
    pub encryption: ResourceEncryption,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SpineRef {
    pub idref: String,
    pub linear: bool,
    pub layout_override: Option<String>,
    pub order: usize,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PackageData {
    pub path: String,
    pub version: Option<String>,
    pub unique_identifier: Option<String>,
    pub metadata: Vec<MetadataValue>,
    pub resources: Vec<ManifestItem>,
    pub spine: Vec<SpineRef>,
    pub toc_id: Option<String>,
    pub layout: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedHref {
    pub path: Option<String>,
    pub fragment: Option<String>,
    pub external: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HrefError {
    Empty,
    MalformedPercentEncoding,
    EncodedSeparator,
    InvalidUtf8,
    ParentTraversal,
    AbsolutePath,
    QueryNotSupported,
    InvalidArchivePath,
}

pub(crate) fn parse_package(bytes: &[u8], path: &str) -> Result<PackageData, XmlFailure> {
    with_xml_document(
        bytes,
        super::xml::DEFAULT_XML_MAX_BYTES,
        super::xml::DEFAULT_XML_MAX_NODES,
        |document| extract_package(document, path),
    )
}

fn extract_package(document: &Document<'_>, path: &str) -> PackageData {
    let root = document.root_element();
    let mut diagnostics = Vec::new();
    if root.tag_name().name() != "package" || root.tag_name().namespace() != Some(OPF_NAMESPACE) {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::PackageRootInvalid,
            DiagnosticSeverity::Incomplete,
            Some(path.to_owned()),
            Some(1),
            Some(1),
            None,
        ));
    }
    let package_namespace = root.tag_name().namespace();
    let metadata_node = child_element(root, package_namespace, "metadata");
    let manifest_node = child_element(root, package_namespace, "manifest");
    let spine_node = child_element(root, package_namespace, "spine");

    let (metadata, layout) = metadata_node
        .map(extract_metadata)
        .unwrap_or_else(|| (Vec::new(), None));

    let mut resources = Vec::new();
    let mut seen_ids = HashMap::<String, (u32, u32)>::new();
    if let Some(manifest) = manifest_node {
        for node in manifest
            .children()
            .filter(|node| is_element_named(*node, package_namespace, "item"))
        {
            let (line, column) = node_position(node);
            let id = node.attribute("id").unwrap_or_default().trim().to_owned();
            let href = node.attribute("href").unwrap_or_default().trim().to_owned();
            let media_type = node
                .attribute("media-type")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase();
            if id.is_empty() {
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ManifestIdMissing,
                    DiagnosticSeverity::Incomplete,
                    Some(path.to_owned()),
                    Some(line),
                    Some(column),
                    None,
                ));
            } else if seen_ids.insert(id.clone(), (line, column)).is_some() {
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ManifestIdDuplicate,
                    DiagnosticSeverity::Incomplete,
                    Some(path.to_owned()),
                    Some(line),
                    Some(column),
                    Some(id.clone()),
                ));
            }
            if href.is_empty() || media_type.is_empty() {
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ManifestItemInvalid,
                    DiagnosticSeverity::Incomplete,
                    Some(path.to_owned()),
                    Some(line),
                    Some(column),
                    None,
                ));
            }
            resources.push(ManifestItem {
                id,
                href,
                media_type,
                properties: split_tokens(node.attribute("properties")),
                fallback: node.attribute("fallback").map(str::to_owned),
                media_overlay: node.attribute("media-overlay").map(str::to_owned),
                path: None,
                available: false,
                encryption: ResourceEncryption::NotEncrypted,
            });
        }
    } else {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::ManifestMissing,
            DiagnosticSeverity::Incomplete,
            Some(path.to_owned()),
            None,
            None,
            None,
        ));
    }

    let mut spine = Vec::new();
    let (toc_id, unique_identifier, version) = (
        spine_node
            .and_then(|node| node.attribute("toc"))
            .map(str::to_owned),
        root.attribute("unique-identifier").map(str::to_owned),
        root.attribute("version").map(str::to_owned),
    );
    if let Some(spine_node) = spine_node {
        for (order, node) in spine_node
            .children()
            .filter(|node| is_element_named(*node, package_namespace, "itemref"))
            .enumerate()
        {
            let (line, column) = node_position(node);
            let idref = node
                .attribute("idref")
                .unwrap_or_default()
                .trim()
                .to_owned();
            if idref.is_empty() {
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::SpineReferenceInvalid,
                    DiagnosticSeverity::Incomplete,
                    Some(path.to_owned()),
                    Some(line),
                    Some(column),
                    None,
                ));
            }
            let linear = match node.attribute("linear").map(str::trim) {
                None | Some("yes") => true,
                Some("no") => false,
                Some(_) => {
                    diagnostics.push(Diagnostic::new(
                        DiagnosticCode::SpineLinearInvalid,
                        DiagnosticSeverity::Warning,
                        Some(path.to_owned()),
                        Some(line),
                        Some(column),
                        Some(idref.clone()),
                    ));
                    true
                }
            };
            let layout_override = split_tokens(node.attribute("properties"))
                .into_iter()
                .find_map(|property| {
                    property
                        .strip_prefix("rendition:layout-")
                        .map(str::to_owned)
                });
            spine.push(SpineRef {
                idref,
                linear,
                layout_override,
                order,
                line,
                column,
            });
        }
    } else {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::SpineMissing,
            DiagnosticSeverity::Incomplete,
            Some(path.to_owned()),
            None,
            None,
            None,
        ));
    }

    PackageData {
        path: path.to_owned(),
        version,
        unique_identifier,
        metadata,
        resources,
        spine,
        toc_id,
        layout,
        diagnostics,
    }
}

fn extract_metadata(node: Node<'_, '_>) -> (Vec<MetadataValue>, Option<String>) {
    let mut values = Vec::new();
    let mut layout = None;
    for child in node.children().filter(Node::is_element) {
        let tag = child.tag_name();
        let property = child.attribute("property").map(str::to_owned);
        let content = child.attribute("content");
        let value = content
            .or_else(|| child.text())
            .unwrap_or_default()
            .trim()
            .to_owned();
        if property.as_deref() == Some("rendition:layout") {
            layout = Some(value.clone());
        }
        values.push(MetadataValue {
            namespace: tag.namespace().map(str::to_owned),
            name: tag.name().to_owned(),
            value,
            id: child.attribute("id").map(str::to_owned),
            property,
            refines: child.attribute("refines").map(str::to_owned),
            scheme: child.attribute("scheme").map(str::to_owned),
        });
    }
    (values, layout)
}

fn child_element<'a, 'input>(
    parent: Node<'a, 'input>,
    namespace: Option<&str>,
    name: &str,
) -> Option<Node<'a, 'input>> {
    parent
        .children()
        .find(|node| is_element_named(*node, namespace, name))
}

fn is_element_named(node: Node<'_, '_>, namespace: Option<&str>, name: &str) -> bool {
    node.is_element() && node.tag_name().name() == name && node.tag_name().namespace() == namespace
}

fn split_tokens(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split_ascii_whitespace()
        .map(str::to_owned)
        .collect()
}

pub(crate) fn resolve_href(base_file: &str, href: &str) -> Result<ResolvedHref, HrefError> {
    if href.is_empty() {
        return Err(HrefError::Empty);
    }
    if has_uri_scheme(href) || href.starts_with("//") {
        return Ok(ResolvedHref {
            path: None,
            fragment: None,
            external: true,
        });
    }
    let (raw_path, raw_fragment) = href.split_once('#').unwrap_or((href, ""));
    if raw_path.contains('?') {
        return Err(HrefError::QueryNotSupported);
    }
    if has_encoded_path_separator(raw_path) {
        return Err(HrefError::EncodedSeparator);
    }
    let fragment = if href.contains('#') {
        Some(percent_decode(raw_fragment)?)
    } else {
        None
    };
    let decoded_path = percent_decode(raw_path)?;
    if decoded_path.starts_with('/') || decoded_path.starts_with('\\') {
        return Err(HrefError::AbsolutePath);
    }
    let path_to_resolve = if decoded_path.is_empty() {
        base_file.to_owned()
    } else {
        let mut components = base_file
            .rsplit_once('/')
            .map(|(parent, _)| parent.split('/').map(str::to_owned).collect::<Vec<_>>())
            .unwrap_or_default();
        for component in decoded_path.split('/') {
            match component {
                "" | "." => {}
                ".." => {
                    if components.pop().is_none() {
                        return Err(HrefError::ParentTraversal);
                    }
                }
                _ => components.push(component.to_owned()),
            }
        }
        components.join("/")
    };
    let path = crate::canonical_archive_path(&path_to_resolve)
        .map_err(|_| HrefError::InvalidArchivePath)?;
    Ok(ResolvedHref {
        path: Some(path),
        fragment,
        external: false,
    })
}

fn has_uri_scheme(value: &str) -> bool {
    let Some((scheme, _)) = value.split_once(':') else {
        return false;
    };
    !scheme.is_empty()
        && scheme
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.'))
}

fn has_encoded_path_separator(value: &str) -> bool {
    value.as_bytes().windows(3).any(|sequence| {
        sequence[0] == b'%'
            && ((sequence[1] == b'2' && sequence[2].eq_ignore_ascii_case(&b'f'))
                || (sequence[1] == b'5' && sequence[2].eq_ignore_ascii_case(&b'c')))
    })
}

fn percent_decode(value: &str) -> Result<String, HrefError> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] == b'%' {
            if cursor + 2 >= bytes.len() {
                return Err(HrefError::MalformedPercentEncoding);
            }
            let high = from_hex(bytes[cursor + 1]).ok_or(HrefError::MalformedPercentEncoding)?;
            let low = from_hex(bytes[cursor + 2]).ok_or(HrefError::MalformedPercentEncoding)?;
            decoded.push((high << 4) | low);
            cursor += 3;
        } else {
            decoded.push(bytes[cursor]);
            cursor += 1;
        }
    }
    String::from_utf8(decoded).map_err(|_| HrefError::InvalidUtf8)
}

fn from_hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{HrefError, resolve_href};

    #[test]
    fn resolves_relative_percent_encoded_paths_and_fragments_once() {
        let target = resolve_href("OPS/package.opf", "Text/ch%20one.xhtml#p%2D1").unwrap();
        assert_eq!(target.path.as_deref(), Some("OPS/Text/ch one.xhtml"));
        assert_eq!(target.fragment.as_deref(), Some("p-1"));
        assert_eq!(
            resolve_href("OPS/package.opf", "../Images/cover%20a.png")
                .unwrap()
                .path
                .as_deref(),
            Some("Images/cover a.png")
        );
    }

    #[test]
    fn rejects_archive_escape_bad_encoding_and_never_resolves_external_urls() {
        assert_eq!(
            resolve_href("OPS/package.opf", "../../outside"),
            Err(HrefError::ParentTraversal)
        );
        assert_eq!(
            resolve_href("OPS/package.opf", "%2E%2E/%2E%2E/outside"),
            Err(HrefError::ParentTraversal)
        );
        assert_eq!(
            resolve_href("OPS/package.opf", "Text/%GG"),
            Err(HrefError::MalformedPercentEncoding)
        );
        assert!(
            resolve_href("OPS/package.opf", "https://example.invalid/book")
                .unwrap()
                .external
        );
        assert!(
            resolve_href("OPS/package.opf", "//example.invalid/book")
                .unwrap()
                .external
        );
    }
    #[test]
    fn rejects_encoded_path_separators_and_decodes_percent_once() {
        for href in [
            "text%2Fchapter.xhtml",
            "text%2fchapter.xhtml",
            "text%5Cchapter.xhtml",
            "text%5cchapter.xhtml",
        ] {
            assert_eq!(
                resolve_href("OPS/book.opf", href),
                Err(HrefError::EncodedSeparator),
                "{href}"
            );
        }
        assert_eq!(
            resolve_href("OPS/book.opf", "text%252Fchapter.xhtml")
                .unwrap()
                .path
                .as_deref(),
            Some("OPS/text%2Fchapter.xhtml")
        );
    }
}
