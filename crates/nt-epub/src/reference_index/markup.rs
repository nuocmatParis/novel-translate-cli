use std::collections::{HashMap, HashSet};

use crate::effective_base::{BaseContext, BaseError, BaseLimit, ReferenceKind as BaseKind};
use nt_domain::source_range::{BlockLocator, ImageSource, ProtectedSourceUnit, SourceUnitKind};
use roxmltree::{Document, Node};

use super::{
    Budget, ImageEvidence, OPF, RawReference, ReferenceIndexError, ReferenceKind,
    ReferenceLocation, Resolution, Resource, SVG, XHTML, XLINK, at, css, node_position, scheme,
    srcset,
};

pub(super) fn scan(
    document: &Document<'_>,
    source: &str,
    resource: Option<&Resource>,
    images: Option<ImageEvidence<'_>>,
    budget: &mut Budget,
) -> Result<Vec<RawReference>, ReferenceIndexError> {
    let root = document.root_element();
    let html = root.tag_name().namespace() == Some(XHTML) && root.tag_name().name() == "html";
    let svg = root.tag_name().namespace() == Some(SVG) && root.tag_name().name() == "svg";
    let opf = root.tag_name().namespace() == Some(OPF) && root.tag_name().name() == "package";
    if !html && !svg && !opf {
        return Err(ReferenceIndexError::SourceMismatch);
    }
    let bases = BaseContext::from_document(source, document).map_err(base_limit)?;
    fn id_for(locator: &BlockLocator) -> Option<&str> {
        match locator {
            BlockLocator::UniqueXhtmlId(id) => Some(id.as_str()),
            _ => None,
        }
    }
    let wanted_ids: HashSet<&str> = match &images {
        Some(ImageEvidence::Extracted(doc)) => doc
            .blocks
            .iter()
            .filter_map(|block| id_for(&block.locator))
            .collect(),
        Some(ImageEvidence::Replayed(placements)) => placements
            .iter()
            .filter_map(|entry| id_for(&entry.block_locator))
            .collect(),
        None => HashSet::new(),
    };
    let mut id_paths: HashMap<String, Option<Vec<u32>>> = HashMap::new();
    let mut output = Vec::new();
    let mut frames = Vec::<(Node<'_, '_>, u32)>::new();
    let mut path = Vec::<u32>::new();
    for node in root.descendants().filter(Node::is_element) {
        if node != root {
            while frames
                .last()
                .is_some_and(|frame| Some(frame.0) != node.parent_element())
            {
                frames.pop();
                path.pop();
            }
            let frame = frames
                .last_mut()
                .ok_or(ReferenceIndexError::SourceMismatch)?;
            let ordinal = frame.1;
            frame.1 = frame
                .1
                .checked_add(1)
                .ok_or(ReferenceIndexError::XmlNodeLimit)?;
            path.push(ordinal);
        }
        if path.len() > budget.limits.max_xml_depth {
            return Err(located(node, ReferenceIndexError::XmlDepthLimit));
        }
        frames.push((node, 0));
        if let Some(id) = unqualified(node, "id").filter(|id| wanted_ids.contains(id)) {
            match id_paths.entry(id.to_owned()) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(Some(path.clone()));
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    entry.insert(None);
                }
            }
        }
        if opf {
            continue;
        }
        let tag = node.tag_name();
        if named(node, XHTML, "img") {
            if let Some(value) = unqualified(node, "src") {
                simple(
                    &mut output,
                    budget,
                    node,
                    &path,
                    ReferenceKind::XhtmlImageSrc,
                    None,
                    "src",
                    value,
                    &bases,
                )?;
            }
        } else if named(node, SVG, "image") {
            if let Some(value) = unqualified(node, "href") {
                simple(
                    &mut output,
                    budget,
                    node,
                    &path,
                    ReferenceKind::SvgImageHref,
                    None,
                    "href",
                    value,
                    &bases,
                )?;
            } else if let Some(value) = node.attribute((XLINK, "href")) {
                simple(
                    &mut output,
                    budget,
                    node,
                    &path,
                    ReferenceKind::SvgImageHref,
                    Some(XLINK),
                    "href",
                    value,
                    &bases,
                )?;
            }
        }
        if (named(node, XHTML, "img") || named(node, XHTML, "source"))
            && unqualified(node, "srcset").is_some()
        {
            let value = unqualified(node, "srcset").unwrap_or_default();
            for candidate in srcset::scan(value, budget).map_err(|error| located(node, error))? {
                let mut location = location(node, &path, None, Some("srcset"));
                location.candidate_ordinal = candidate.ordinal;
                location.decoded_byte_range = Some(range(candidate.range)?);
                output.push(RawReference {
                    location,
                    kind: ReferenceKind::SrcsetCandidate,
                    raw: candidate.value.clone(),
                    value: candidate.value.clone(),
                    resolved: Some(resolve_node(&bases, node, &candidate.value)?),
                    spine_override: None,
                    inventory_occurrence: None,
                    source_occurrences: Vec::new(),
                });
            }
        }
        if named(node, XHTML, "link") {
            if let Some(value) = unqualified(node, "href") {
                let stylesheet = unqualified(node, "rel").is_some_and(|rel| {
                    rel.split_ascii_whitespace()
                        .any(|part| part.eq_ignore_ascii_case("stylesheet"))
                });
                simple(
                    &mut output,
                    budget,
                    node,
                    &path,
                    if stylesheet {
                        ReferenceKind::StylesheetLink
                    } else {
                        ReferenceKind::XhtmlLink
                    },
                    None,
                    "href",
                    value,
                    &bases,
                )?;
            }
        } else if (named(node, XHTML, "a") || named(node, XHTML, "area") || named(node, SVG, "a"))
            && unqualified(node, "href").is_some()
        {
            simple(
                &mut output,
                budget,
                node,
                &path,
                ReferenceKind::XhtmlLink,
                None,
                "href",
                unqualified(node, "href").unwrap_or_default(),
                &bases,
            )?;
        } else if named(node, SVG, "a") && node.attribute((XLINK, "href")).is_some() {
            simple(
                &mut output,
                budget,
                node,
                &path,
                ReferenceKind::XhtmlLink,
                Some(XLINK),
                "href",
                node.attribute((XLINK, "href")).unwrap_or_default(),
                &bases,
            )?;
        }
        if matches!(tag.namespace(), Some(XHTML | SVG)) {
            if let Some(value) = unqualified(node, "style") {
                css_references(
                    &mut output,
                    budget,
                    node,
                    &path,
                    Some("style"),
                    value,
                    &bases,
                    false,
                )?;
            }
            if tag.name() == "style" {
                let mut text = String::new();
                for child in node.children() {
                    if child.is_element() {
                        return Err(ReferenceIndexError::InvalidCss);
                    }
                    if child.is_text() {
                        let value = child.text().unwrap_or_default();
                        if text
                            .len()
                            .checked_add(value.len())
                            .is_none_or(|size| size > budget.limits.max_css_bytes)
                        {
                            return Err(ReferenceIndexError::CssByteLimit);
                        }
                        text.push_str(value);
                    }
                }
                css_references(&mut output, budget, node, &path, None, &text, &bases, true)?;
            }
        }
    }
    if let Some(resource) = resource {
        reconcile_inventory(source, resource, &mut output)?;
    }
    if let Some(images) = images {
        reconcile_extraction(images, &id_paths, &mut output)?;
    }
    Ok(output)
}

fn named(node: Node<'_, '_>, namespace: &str, name: &str) -> bool {
    node.is_element()
        && node.tag_name().namespace() == Some(namespace)
        && node.tag_name().name() == name
}
// roxmltree's string-only attribute lookup matches local names in any
// namespace. URL attributes without a prefix must remain distinct from XLink
// and foreign namespace lookalikes.
pub(super) fn unqualified<'a>(node: Node<'a, '_>, name: &str) -> Option<&'a str> {
    node.attributes()
        .find(|attribute| attribute.namespace().is_none() && attribute.name() == name)
        .map(|attribute| attribute.value())
}
fn range((start, end): (usize, usize)) -> Result<(u32, u32), ReferenceIndexError> {
    Ok((
        u32::try_from(start).map_err(|_| ReferenceIndexError::ReferenceByteLimit)?,
        u32::try_from(end).map_err(|_| ReferenceIndexError::ReferenceByteLimit)?,
    ))
}
fn location(
    node: Node<'_, '_>,
    path: &[u32],
    namespace: Option<&str>,
    name: Option<&str>,
) -> ReferenceLocation {
    let (line, column) = node_position(node);
    ReferenceLocation {
        element_ordinals: path.to_vec(),
        attribute_namespace: namespace.map(str::to_owned),
        attribute_local_name: name.map(str::to_owned),
        candidate_ordinal: 0,
        line,
        column,
        decoded_byte_range: None,
    }
}
fn located(node: Node<'_, '_>, reason: ReferenceIndexError) -> ReferenceIndexError {
    let (line, column) = node_position(node);
    at("", Some(line), Some(column), reason)
}
#[allow(clippy::too_many_arguments)]
fn simple(
    output: &mut Vec<RawReference>,
    budget: &mut Budget,
    node: Node<'_, '_>,
    path: &[u32],
    kind: ReferenceKind,
    namespace: Option<&str>,
    name: &str,
    value: &str,
    bases: &BaseContext,
) -> Result<(), ReferenceIndexError> {
    budget
        .reference(value.len())
        .map_err(|error| located(node, error))?;
    output.push(RawReference {
        location: location(node, path, namespace, Some(name)),
        kind,
        value: value.to_owned(),
        raw: value.to_owned(),
        resolved: Some(resolve_node(bases, node, value)?),
        spine_override: None,
        inventory_occurrence: None,
        source_occurrences: Vec::new(),
    });
    Ok(())
}
#[allow(clippy::too_many_arguments)]
fn css_references(
    output: &mut Vec<RawReference>,
    budget: &mut Budget,
    node: Node<'_, '_>,
    path: &[u32],
    attribute: Option<&str>,
    text: &str,
    bases: &BaseContext,
    block: bool,
) -> Result<(), ReferenceIndexError> {
    if text.len() > budget.limits.max_css_bytes {
        return Err(located(node, ReferenceIndexError::CssByteLimit));
    }
    for (ordinal, reference) in css::scan(text, budget)
        .map_err(|error| located(node, error))?
        .into_iter()
        .enumerate()
    {
        let mut location = location(node, path, None, attribute);
        location.candidate_ordinal =
            u32::try_from(ordinal).map_err(|_| ReferenceIndexError::ReferenceCountLimit)?;
        location.decoded_byte_range = Some(range(reference.range)?);
        if block {
            let start = node
                .children()
                .find(Node::is_text)
                .map(|n| n.document().text_pos_at(n.range().start));
            if let Some(start) = start {
                let (line, column) = css_position(text, reference.position, start.row, start.col);
                location.line = line;
                location.column = column;
            }
        }
        output.push(RawReference {
            location,
            kind: reference.kind,
            value: reference.value.clone(),
            raw: reference.raw,
            resolved: Some(
                resolve_css(bases, &reference.value).map_err(|reason| located(node, reason))?,
            ),
            spine_override: None,
            inventory_occurrence: None,
            source_occurrences: Vec::new(),
        });
    }
    Ok(())
}
pub(super) fn css_position(
    text: &str,
    position: usize,
    first_line: u32,
    first_column: u32,
) -> (u32, u32) {
    let (mut line, mut column, mut cr) = (first_line, first_column, false);
    for value in text[..position].chars() {
        match value {
            '\n' if cr => {}
            '\r' | '\n' | '\u{c}' => {
                line += 1;
                column = 1;
            }
            _ => {
                column += 1;
            }
        }
        cr = value == '\r';
    }
    (line, column)
}

fn reconcile_inventory(
    _source: &str,
    resource: &Resource,
    output: &mut [RawReference],
) -> Result<(), ReferenceIndexError> {
    let mut images = output.iter_mut().filter(|item| {
        matches!(
            item.kind,
            ReferenceKind::XhtmlImageSrc | ReferenceKind::SvgImageHref
        )
    });
    for (ordinal, expected) in resource.image_uses.iter().enumerate() {
        let item = images.next().ok_or(ReferenceIndexError::SourceMismatch)?;
        if expected.occurrence != ordinal
            || expected.line != item.location.line
            || expected.column != item.location.column
        {
            return Err(ReferenceIndexError::SourceMismatch);
        }
        let actual = item
            .resolved
            .clone()
            .ok_or(ReferenceIndexError::SourceMismatch)?;
        let matches = match actual {
            Resolution::Internal(path, _) => {
                expected.resource_path.as_deref() == Some(path.as_str()) && !expected.external
            }
            Resolution::External(_) => expected.resource_path.is_none() && expected.external,
            Resolution::Invalid => expected.resource_path.is_none() && !expected.external,
            Resolution::MissingIdentifier | Resolution::Ambiguous => false,
        };
        if !matches {
            return Err(ReferenceIndexError::SourceMismatch);
        }
        item.inventory_occurrence = Some(ordinal);
    }
    if images.next().is_some() {
        return Err(ReferenceIndexError::SourceMismatch);
    }
    Ok(())
}
fn reconcile_extraction(
    images: ImageEvidence<'_>,
    ids: &HashMap<String, Option<Vec<u32>>>,
    output: &mut [RawReference],
) -> Result<(), ReferenceIndexError> {
    let mut by_path = HashMap::new();
    for (index, item) in output.iter().enumerate().filter(|(_, item)| {
        matches!(
            item.kind,
            ReferenceKind::XhtmlImageSrc | ReferenceKind::SvgImageHref
        )
    }) {
        by_path.insert(item.location.element_ordinals.clone(), index);
    }
    let mut attach = |locator: &BlockLocator, position, image: &ImageSource| {
        let Some(raw) = &image.raw_source else {
            return Ok(());
        };
        let mut path = match locator {
            BlockLocator::UniqueXhtmlId(id) => ids
                .get(id)
                .and_then(Clone::clone)
                .ok_or(ReferenceIndexError::SourceMismatch)?,
            BlockLocator::NodePath {
                element_ordinals, ..
            } => element_ordinals.clone(),
        };
        path.extend_from_slice(&image.node_path);
        let index = *by_path
            .get(&path)
            .ok_or(ReferenceIndexError::SourceMismatch)?;
        let item = &mut output[index];
        let matches = match item.resolved.as_ref() {
            Some(Resolution::Internal(path, _)) => {
                image.resource_path.as_deref() == Some(path.as_str()) && !image.external
            }
            Some(Resolution::External(_)) => image.resource_path.is_none() && image.external,
            Some(Resolution::Invalid) => image.resource_path.is_none() && !image.external,
            _ => false,
        };
        if item.value != *raw || !matches {
            return Err(ReferenceIndexError::SourceMismatch);
        }
        item.source_occurrences.push(position);
        Ok(())
    };
    match images {
        ImageEvidence::Extracted(document) => {
            for block in document
                .blocks
                .iter()
                .filter(|block| block.element_name != "image-page")
            {
                for (offset, unit) in block.units.iter().enumerate() {
                    if let SourceUnitKind::Protected(ProtectedSourceUnit::Image(image)) = &unit.kind
                    {
                        let offset =
                            u32::try_from(offset).map_err(|_| ReferenceIndexError::InvalidInput)?;
                        attach(&block.locator, (*block.id.as_bytes(), offset), image)?;
                    }
                }
            }
        }
        ImageEvidence::Replayed(placements) => {
            for placement in placements {
                attach(
                    &placement.block_locator,
                    (*placement.block_id.as_bytes(), placement.unit_offset),
                    &placement.image,
                )?;
            }
        }
    }
    Ok(())
}

fn base_limit(error: BaseError) -> ReferenceIndexError {
    match error {
        BaseError::Limit(BaseLimit::Bytes) => ReferenceIndexError::BaseByteLimit,
        BaseError::Limit(BaseLimit::Nodes) => ReferenceIndexError::BaseNodeLimit,
        BaseError::Limit(BaseLimit::Work) => ReferenceIndexError::BaseWorkLimit,
        BaseError::Invalid(_) => ReferenceIndexError::UnsafeUri,
    }
}
fn resolved(
    value: &str,
    target: Result<crate::package::ResolvedHref, BaseError>,
) -> Result<Resolution, ReferenceIndexError> {
    match target {
        Ok(t) if t.external => Ok(Resolution::External(
            if value.contains(':') || value.starts_with("//") {
                scheme(value)
            } else {
                "external-base".into()
            },
        )),
        Ok(t) => Ok(t
            .path
            .map_or(Resolution::Invalid, |p| Resolution::Internal(p, t.fragment))),
        Err(BaseError::Invalid(_)) => Ok(Resolution::Invalid),
        Err(error) => Err(base_limit(error)),
    }
}
fn resolve_node(
    bases: &BaseContext,
    node: Node<'_, '_>,
    value: &str,
) -> Result<Resolution, ReferenceIndexError> {
    let kind = if node.tag_name().namespace() == Some(SVG) {
        BaseKind::Svg
    } else {
        BaseKind::Html
    };
    resolved(value, bases.resolve(node, value, kind)).map_err(|reason| located(node, reason))
}
fn resolve_css(bases: &BaseContext, value: &str) -> Result<Resolution, ReferenceIndexError> {
    resolved(value, bases.resolve_document_reference(value))
}
