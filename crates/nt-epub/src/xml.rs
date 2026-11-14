use std::collections::{HashMap, HashSet};

use roxmltree::{Document, Error, Node, ParsingOptions, TextPos};

use crate::encoding::decode_xml;

pub(crate) const DEFAULT_XML_MAX_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const DEFAULT_XML_MAX_NODES: u32 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum XmlFailureKind {
    ByteLimit,
    NodeLimit,
    UnsupportedEncoding,
    ExternalEntity,
    UnknownEntity,
    EntityExpansionLimit,
    NavigationDepthLimit,
    Malformed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct XmlFailure {
    pub kind: XmlFailureKind,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

impl XmlFailure {
    pub(crate) fn new(kind: XmlFailureKind, position: Option<TextPos>) -> Self {
        Self {
            kind,
            line: position.map(|value| value.row),
            column: position.map(|value| value.col),
        }
    }
}

pub(crate) fn node_position(node: Node<'_, '_>) -> (u32, u32) {
    let position = node.document().text_pos_at(node.range().start);
    (position.row, position.col)
}

pub(crate) fn with_xml_document<T>(
    bytes: &[u8],
    max_bytes: usize,
    max_nodes: u32,
    visit: impl FnOnce(&Document<'_>) -> T,
) -> Result<T, XmlFailure> {
    with_xml_document_with_expansion_limit(bytes, max_bytes, max_nodes, max_bytes, visit)
}

fn with_xml_document_with_expansion_limit<T>(
    bytes: &[u8],
    max_bytes: usize,
    max_nodes: u32,
    max_expanded_bytes: usize,
    visit: impl FnOnce(&Document<'_>) -> T,
) -> Result<T, XmlFailure> {
    let text = decode_xml(bytes, max_bytes)?;
    preflight_entity_expansion(&text, max_expanded_bytes)?;
    let deny_external = |_public_id: Option<&str>, _uri: &str| -> Result<Option<&str>, String> {
        Err("external entity access is disabled".to_owned())
    };
    let options = ParsingOptions {
        allow_dtd: true,
        nodes_limit: max_nodes,
        entity_resolver: Some(&deny_external),
    };
    let document = Document::parse_with_options(&text, options).map_err(map_parser_error)?;
    Ok(visit(&document))
}

#[derive(Clone, Copy)]
enum EntityDefinition<'a> {
    Internal(&'a str),
    External,
}

#[derive(Clone, Copy)]
struct DoctypeSpan {
    start: usize,
    end: usize,
    subset: Option<(usize, usize)>,
}

const MAX_ENTITY_REFERENCE_DEPTH: usize = 10;

#[derive(Default)]
struct ScanMetrics {
    #[cfg(test)]
    reference_scan_bytes: usize,
}

impl ScanMetrics {
    #[cfg(test)]
    fn record_reference_scan(&mut self, scanned_bytes: usize) {
        self.reference_scan_bytes = self.reference_scan_bytes.saturating_add(scanned_bytes);
    }

    #[cfg(not(test))]
    fn record_reference_scan(&mut self) {}
}

struct EntityExpansion<'a, 'doc> {
    max_expanded_bytes: usize,
    entities: &'a HashMap<String, EntityDefinition<'doc>>,
    memo: &'a mut HashMap<String, usize>,
    active: &'a mut HashSet<String>,
    metrics: &'a mut ScanMetrics,
}

struct EntityReferenceEnd {
    semicolon: Option<usize>,
    #[cfg(test)]
    scanned_bytes: usize,
}

fn find_entity_reference_end(bytes: &[u8], start: usize) -> EntityReferenceEnd {
    let mut cursor = start + 1;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b';' => {
                return EntityReferenceEnd {
                    semicolon: Some(cursor),
                    #[cfg(test)]
                    scanned_bytes: cursor + 1 - start,
                };
            }
            b'&' | b'<' => {
                return EntityReferenceEnd {
                    semicolon: None,
                    #[cfg(test)]
                    scanned_bytes: cursor + 1 - start,
                };
            }
            byte if byte.is_ascii_whitespace() => {
                return EntityReferenceEnd {
                    semicolon: None,
                    #[cfg(test)]
                    scanned_bytes: cursor + 1 - start,
                };
            }
            _ => cursor += 1,
        }
    }
    EntityReferenceEnd {
        semicolon: None,
        #[cfg(test)]
        scanned_bytes: bytes.len() - start,
    }
}

fn preflight_entity_expansion(text: &str, max_expanded_bytes: usize) -> Result<(), XmlFailure> {
    let mut metrics = ScanMetrics::default();
    preflight_entity_expansion_with_metrics(text, max_expanded_bytes, &mut metrics)
}

fn preflight_entity_expansion_with_metrics(
    text: &str,
    max_expanded_bytes: usize,
    metrics: &mut ScanMetrics,
) -> Result<(), XmlFailure> {
    let doctype = find_doctype_span(text);
    let entities = match doctype.and_then(|span| span.subset.map(|subset| (span, subset))) {
        Some((_, (start, end))) => parse_internal_entities(text, start, end).map_err(|offset| {
            XmlFailure::new(
                XmlFailureKind::EntityExpansionLimit,
                Some(text_position_at(text, offset)),
            )
        })?,
        None => HashMap::new(),
    };
    let mut memo = HashMap::new();
    let mut active = HashSet::new();
    let skip = doctype.map(|span| (span.start, span.end));
    let mut expansion = EntityExpansion {
        max_expanded_bytes,
        entities: &entities,
        memo: &mut memo,
        active: &mut active,
        metrics,
    };
    estimate_referenced_size(text, skip, 0, &mut expansion)
        .map(|_| ())
        .map_err(|offset| {
            XmlFailure::new(
                XmlFailureKind::EntityExpansionLimit,
                Some(text_position_at(text, offset)),
            )
        })
}

fn find_doctype_span(text: &str) -> Option<DoctypeSpan> {
    let bytes = text.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if starts_with_at(bytes, cursor, b"<!--") {
            cursor = skip_through(bytes, cursor + 4, b"-->");
        } else if starts_with_at(bytes, cursor, b"<?") {
            cursor = skip_through(bytes, cursor + 2, b"?>");
        } else if starts_with_at(bytes, cursor, b"<!DOCTYPE") {
            let start = cursor;
            cursor += b"<!DOCTYPE".len();
            let mut quote = None;
            let mut subset_depth = 0usize;
            let mut subset_start = None;
            let mut subset_end = None;
            while cursor < bytes.len() {
                if quote.is_none() && starts_with_at(bytes, cursor, b"<!--") {
                    cursor = skip_through(bytes, cursor + 4, b"-->");
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
                            if subset_depth == 0 {
                                subset_start = Some(cursor + 1);
                            }
                            subset_depth = subset_depth.saturating_add(1);
                        }
                        b']' if subset_depth > 0 => {
                            subset_depth -= 1;
                            if subset_depth == 0 {
                                subset_end = Some(cursor);
                            }
                        }
                        b'>' if subset_depth == 0 => {
                            return Some(DoctypeSpan {
                                start,
                                end: cursor + 1,
                                subset: subset_start.zip(subset_end),
                            });
                        }
                        _ => {}
                    }
                }
                cursor += 1;
            }
            return Some(DoctypeSpan {
                start,
                end: bytes.len(),
                subset: subset_start.zip(subset_end),
            });
        } else {
            cursor += 1;
        }
    }
    None
}

fn parse_internal_entities<'a>(
    text: &'a str,
    start: usize,
    end: usize,
) -> Result<HashMap<String, EntityDefinition<'a>>, usize> {
    let bytes = text.as_bytes();
    let mut cursor = start;
    let mut entities = HashMap::new();
    while cursor < end {
        if starts_with_at(bytes, cursor, b"<!--") {
            cursor = skip_through_bounded(bytes, cursor + 4, b"-->", end);
            continue;
        }
        if starts_with_at(bytes, cursor, b"<?") {
            cursor = skip_through_bounded(bytes, cursor + 2, b"?>", end);
            continue;
        }
        if starts_with_at(bytes, cursor, b"<!ENTITY") {
            let declaration_start = cursor;
            let declaration_end = find_declaration_end(bytes, cursor, end);
            let mut position = cursor + b"<!ENTITY".len();
            skip_ascii_whitespace(bytes, &mut position, declaration_end);
            if bytes.get(position) == Some(&b'%') {
                return Err(declaration_start);
            }
            let name_start = position;
            while position < declaration_end
                && !bytes[position].is_ascii_whitespace()
                && bytes[position] != b'>'
            {
                position += 1;
            }
            if name_start == position {
                cursor = declaration_end.max(cursor + 1);
                continue;
            }
            let name = &text[name_start..position];
            skip_ascii_whitespace(bytes, &mut position, declaration_end);
            let definition = match bytes.get(position).copied() {
                Some(delimiter @ (b'\'' | b'"')) => {
                    let value_start = position + 1;
                    let value_end = bytes[value_start..declaration_end]
                        .iter()
                        .position(|byte| *byte == delimiter)
                        .map(|offset| value_start + offset)
                        .unwrap_or(declaration_end);
                    if let Some(offset) = first_parameter_reference(text, value_start, value_end) {
                        return Err(offset);
                    }
                    EntityDefinition::Internal(&text[value_start..value_end])
                }
                _ => EntityDefinition::External,
            };
            entities.entry(name.to_owned()).or_insert(definition);
            cursor = declaration_end.max(cursor + 1);
            continue;
        }
        if bytes[cursor] == b'\'' || bytes[cursor] == b'"' {
            cursor = skip_quoted(bytes, cursor, end);
            continue;
        }
        if bytes[cursor] == b'%' && is_parameter_reference(bytes, cursor, end) {
            return Err(cursor);
        }
        cursor += 1;
    }
    Ok(entities)
}

fn first_parameter_reference(text: &str, start: usize, end: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut cursor = start;
    while cursor < end {
        if bytes[cursor] == b'%' && is_parameter_reference(bytes, cursor, end) {
            return Some(cursor);
        }
        cursor += 1;
    }
    None
}

fn is_parameter_reference(bytes: &[u8], start: usize, end: usize) -> bool {
    let mut cursor = start + 1;
    let name_start = cursor;
    while cursor < end && is_xml_name_byte(bytes[cursor]) {
        cursor += 1;
    }
    cursor > name_start && cursor < end && bytes[cursor] == b';'
}

fn is_xml_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b':' | b'-' | b'.') || byte >= 0x80
}

fn find_declaration_end(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut cursor = start;
    let mut quote = None;
    while cursor < end {
        let byte = bytes[cursor];
        if let Some(delimiter) = quote {
            if byte == delimiter {
                quote = None;
            }
        } else {
            match byte {
                b'\'' | b'"' => quote = Some(byte),
                b'>' => return cursor + 1,
                _ => {}
            }
        }
        cursor += 1;
    }
    end
}

fn skip_ascii_whitespace(bytes: &[u8], cursor: &mut usize, end: usize) {
    while *cursor < end && bytes[*cursor].is_ascii_whitespace() {
        *cursor += 1;
    }
}

fn skip_quoted(bytes: &[u8], start: usize, end: usize) -> usize {
    let delimiter = bytes[start];
    bytes[start + 1..end]
        .iter()
        .position(|byte| *byte == delimiter)
        .map(|offset| start + 2 + offset)
        .unwrap_or(end)
}

fn estimate_referenced_size(
    text: &str,
    skip: Option<(usize, usize)>,
    depth: usize,
    expansion: &mut EntityExpansion<'_, '_>,
) -> Result<usize, usize> {
    let bytes = text.as_bytes();
    let mut projected_size = text.len();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if let Some((start, end)) = skip
            && cursor == start
        {
            cursor = end;
            continue;
        }
        if starts_with_at(bytes, cursor, b"<!--") {
            cursor = skip_through(bytes, cursor + 4, b"-->");
            continue;
        }
        if starts_with_at(bytes, cursor, b"<![CDATA[") {
            cursor = skip_through(bytes, cursor + b"<![CDATA[".len(), b"]]>");
            continue;
        }
        if starts_with_at(bytes, cursor, b"<?") {
            cursor = skip_through(bytes, cursor + 2, b"?>");
            continue;
        }
        if bytes[cursor] != b'&' {
            cursor += 1;
            continue;
        }
        let reference = find_entity_reference_end(bytes, cursor);
        #[cfg(test)]
        expansion
            .metrics
            .record_reference_scan(reference.scanned_bytes);
        #[cfg(not(test))]
        expansion.metrics.record_reference_scan();
        let Some(semicolon) = reference.semicolon else {
            cursor += 1;
            continue;
        };
        if bytes[cursor + 1..semicolon]
            .iter()
            .any(|byte| byte.is_ascii_whitespace() || matches!(byte, b'<' | b'&'))
        {
            cursor += 1;
            continue;
        }
        let name = &text[cursor + 1..semicolon];
        match estimate_entity_reference(name, depth, expansion) {
            Ok(Some(expanded_size)) => {
                projected_size = projected_size
                    .saturating_sub(semicolon + 1 - cursor)
                    .saturating_add(expanded_size);
                if projected_size > expansion.max_expanded_bytes {
                    return Err(cursor);
                }
            }
            Ok(None) => {}
            Err(()) => return Err(cursor),
        }
        cursor = semicolon + 1;
    }
    if projected_size > expansion.max_expanded_bytes {
        Err(0)
    } else {
        Ok(projected_size)
    }
}

fn estimate_entity_reference(
    name: &str,
    depth: usize,
    expansion: &mut EntityExpansion<'_, '_>,
) -> Result<Option<usize>, ()> {
    if let Some(size) = character_reference_size(name) {
        return Ok(Some(size));
    }
    if let Some(size) = expansion.memo.get(name) {
        return Ok(Some(*size));
    }
    if depth >= MAX_ENTITY_REFERENCE_DEPTH {
        return Err(());
    }
    let Some(EntityDefinition::Internal(replacement)) = expansion.entities.get(name).copied()
    else {
        return Ok(None);
    };
    if !expansion.active.insert(name.to_owned()) {
        return Err(());
    }
    let result = estimate_referenced_size(replacement, None, depth + 1, expansion);
    expansion.active.remove(name);
    let expanded_size = result.map_err(|_| ())?;
    expansion.memo.insert(name.to_owned(), expanded_size);
    Ok(Some(expanded_size))
}

fn character_reference_size(name: &str) -> Option<usize> {
    let predefined = match name {
        "amp" | "lt" | "gt" | "apos" | "quot" => Some(1),
        _ => None,
    };
    if predefined.is_some() {
        return predefined;
    }
    let numeric = name.strip_prefix("#")?;
    let value = if let Some(hexadecimal) = numeric.strip_prefix(['x', 'X']) {
        u32::from_str_radix(hexadecimal, 16).ok()?
    } else {
        numeric.parse::<u32>().ok()?
    };
    char::from_u32(value).map(char::len_utf8)
}

fn starts_with_at(bytes: &[u8], offset: usize, value: &[u8]) -> bool {
    bytes
        .get(offset..offset.saturating_add(value.len()))
        .is_some_and(|candidate| candidate == value)
}

fn find_sequence(bytes: &[u8], start: usize, value: &[u8]) -> Option<usize> {
    bytes
        .get(start..)?
        .windows(value.len())
        .position(|candidate| candidate == value)
        .map(|offset| start + offset)
}

fn skip_through(bytes: &[u8], start: usize, ending: &[u8]) -> usize {
    find_sequence(bytes, start, ending)
        .map(|offset| offset + ending.len())
        .unwrap_or(bytes.len())
}

fn skip_through_bounded(bytes: &[u8], start: usize, ending: &[u8], end: usize) -> usize {
    find_sequence(&bytes[..end], start, ending)
        .map(|offset| offset + ending.len())
        .unwrap_or(end)
}

fn text_position_at(text: &str, offset: usize) -> TextPos {
    let offset = offset.min(text.len());
    let prefix = &text[..offset];
    let row = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32 + 1;
    let column = prefix
        .rsplit('\n')
        .next()
        .map(|line| line.chars().count() as u32 + 1)
        .unwrap_or(1);
    TextPos::new(row, column)
}
fn map_parser_error(error: Error) -> XmlFailure {
    let kind = match error {
        Error::NodesLimitReached
        | Error::AttributesLimitReached
        | Error::NamespacesLimitReached => XmlFailureKind::NodeLimit,
        Error::EntityReferenceLoop(_) => XmlFailureKind::EntityExpansionLimit,
        Error::EntityResolver(_, _) => XmlFailureKind::ExternalEntity,
        Error::UnknownEntityReference(_, _) => XmlFailureKind::UnknownEntity,
        _ => XmlFailureKind::Malformed,
    };
    XmlFailure::new(kind, Some(error.pos()))
}

#[cfg(test)]
mod tests {
    use roxmltree::Document;
    use std::collections::{HashMap, HashSet};

    use super::{
        EntityExpansion, ScanMetrics, XmlFailureKind, estimate_referenced_size,
        preflight_entity_expansion, preflight_entity_expansion_with_metrics, with_xml_document,
        with_xml_document_with_expansion_limit,
    };

    #[test]
    fn bounds_node_count_and_rejects_external_entities_without_resolution() {
        let error = with_xml_document(b"<root><a/><b/></root>", 1024, 2, |_| ()).unwrap_err();
        assert_eq!(error.kind, XmlFailureKind::NodeLimit);
        assert_eq!((error.line, error.column), (Some(1), Some(1)));

        let xml =
            b"<!DOCTYPE root [<!ENTITY ext SYSTEM 'file:///private/sentinel'>]><root>&ext;</root>";
        let error = with_xml_document(xml, 1024, 32, |_| ()).unwrap_err();
        assert_eq!(error.kind, XmlFailureKind::ExternalEntity);
        assert_eq!(error.line, Some(1));
    }

    #[test]
    fn accepts_xml_node_counts_below_and_at_the_cap_and_rejects_one_above() {
        let xml = b"<root><child/></root>";
        let count = Document::parse(std::str::from_utf8(xml).unwrap())
            .unwrap()
            .descendants()
            .count() as u32;
        assert!(with_xml_document(xml, 1024, count + 1, |_| ()).is_ok());
        assert!(with_xml_document(xml, 1024, count, |_| ()).is_ok());
        assert_eq!(
            with_xml_document(xml, 1024, count - 1, |_| ())
                .unwrap_err()
                .kind,
            XmlFailureKind::NodeLimit
        );
    }

    #[test]
    fn bounds_entity_expansion_and_keeps_internal_dtd_syntax() {
        let valid = b"<!DOCTYPE root [<!ENTITY name 'safe'>]><root>&name;</root>";
        let root_text = with_xml_document(valid, 1024, 32, |document| {
            document.root_element().text().unwrap().to_owned()
        })
        .unwrap();
        assert_eq!(root_text, "safe");

        let mut expansion = String::from("<!DOCTYPE r [<!ENTITY e0 'safe'>");
        for index in 1..=10 {
            expansion.push_str(&format!("<!ENTITY e{index} '&e{};'>", index - 1));
        }
        expansion.push_str("]><r>&e10;</r>");
        let error = with_xml_document(expansion.as_bytes(), 1024, 64, |_| ()).unwrap_err();
        assert_eq!(error.kind, XmlFailureKind::EntityExpansionLimit);
    }

    #[test]
    fn rejects_large_root_repeated_expansion_before_xml_parse() {
        let entity = "x".repeat(32 * 1024);
        let mut xml = format!("<!DOCTYPE r [<!ENTITY e '{entity}'>]><r>");
        for _ in 0..1024 {
            xml.push_str("&e;");
        }
        xml.push_str("</r>");
        assert!(xml.len() < 64 * 1024);
        assert_eq!(
            with_xml_document_with_expansion_limit(
                xml.as_bytes(),
                64 * 1024,
                4096,
                16 * 1024 * 1024,
                |_| (),
            )
            .unwrap_err()
            .kind,
            XmlFailureKind::EntityExpansionLimit
        );
    }

    #[test]
    fn applies_one_aggregate_budget_to_text_attributes_and_nested_entities() {
        let xml =
            "<!DOCTYPE r [<!ENTITY e '12345'><!ENTITY pair '&e;&e;'>]><r a='&e;'>&pair;&e;</r>";
        let projected = xml.len() + 2 + 4 + 2;
        assert!(preflight_entity_expansion(xml, projected - 1).is_err());
        assert!(preflight_entity_expansion(xml, projected).is_ok());
        assert!(preflight_entity_expansion(xml, projected + 1).is_ok());

        let mut quadratic = String::from("<!DOCTYPE r [<!ENTITY e0 'abcdefgh'>");
        for index in 1..=9 {
            quadratic.push_str(&format!(
                "<!ENTITY e{index} '&e{};&e{};'>",
                index - 1,
                index - 1
            ));
        }
        quadratic.push_str("]><r>&e9;&e9;</r>");
        let quadratic_projected = quadratic.len() + 2 * (4096 - "&e9;".len());
        assert!(preflight_entity_expansion(&quadratic, quadratic_projected - 1).is_err());
        assert!(preflight_entity_expansion(&quadratic, quadratic_projected).is_ok());
        assert!(preflight_entity_expansion(&quadratic, quadratic_projected + 1).is_ok());
    }

    #[test]
    fn malformed_entity_reference_runs_have_linear_preflight_work_and_located_errors() {
        let ampersands = "&".repeat(16 * 1024);
        let malformed = [
            format!("<r>{ampersands}</r>"),
            format!("<r a='{ampersands}'/>"),
            format!("<!DOCTYPE r [<!ENTITY broken '{ampersands}'>]><r>&broken;</r>"),
        ];

        for text in malformed {
            let mut metrics = ScanMetrics::default();
            assert!(
                preflight_entity_expansion_with_metrics(&text, 64 * 1024, &mut metrics,).is_ok()
            );
            assert!(
                metrics.reference_scan_bytes <= text.len().saturating_mul(4),
                "scanned {} bytes for {} decoded bytes",
                metrics.reference_scan_bytes,
                text.len(),
            );

            let failure = with_xml_document(text.as_bytes(), 64 * 1024, 4096, |_| ()).unwrap_err();
            assert!(!matches!(
                failure.kind,
                XmlFailureKind::EntityExpansionLimit
            ));
            assert!(failure.line.is_some());
            assert!(failure.column.is_some());
        }

        let mut metrics = ScanMetrics::default();
        let mut memo = HashMap::new();
        let mut active = HashSet::new();
        let replacement = format!("{ampersands}tail");
        {
            let entities = HashMap::new();
            let mut expansion = EntityExpansion {
                max_expanded_bytes: 64 * 1024,
                entities: &entities,
                memo: &mut memo,
                active: &mut active,
                metrics: &mut metrics,
            };
            assert!(estimate_referenced_size(&replacement, None, 0, &mut expansion).is_ok());
        }
        assert!(metrics.reference_scan_bytes <= replacement.len().saturating_mul(4));
    }

    #[test]
    fn accepts_public_external_dtd_syntax_without_resolving_it() {
        let xml = br#"<!DOCTYPE root PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd" [<!ENTITY title "safe">]><root>&title;</root>"#;
        let value = with_xml_document(xml, 1024, 32, |document| {
            document.root_element().text().unwrap().to_owned()
        })
        .unwrap();
        assert_eq!(value, "safe");
    }

    #[test]
    fn ignores_reference_lookalikes_in_comments_cdata_and_processing_instructions() {
        let xml = "<!DOCTYPE r [<!ENTITY e 'x'>]><r><!-- &e; --><![CDATA[&e;]]><?probe &e;?>ok</r>";
        assert!(preflight_entity_expansion(xml, xml.len()).is_ok());
    }
}
