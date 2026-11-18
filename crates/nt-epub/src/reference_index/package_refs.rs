use super::markup::unqualified;
use super::{
    Budget, EpubInventory, OPF, RawReference, ReferenceIndexError, ReferenceKind,
    ReferenceLocation, Resolution, at, node_position, resolve,
};
use roxmltree::{Document, Node};
use std::collections::HashMap;

pub(super) fn scan(
    document: &Document<'_>,
    path: &str,
    inventory: &EpubInventory,
    budget: &mut Budget,
) -> Result<Vec<RawReference>, ReferenceIndexError> {
    let root = document.root_element();
    if !named(root, "package") {
        return Err(ReferenceIndexError::SourceMismatch);
    }
    let mut ids: HashMap<&str, Option<usize>> = HashMap::new();
    for (index, item) in inventory.resources.iter().enumerate() {
        match ids.entry(&item.id) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(Some(index));
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.insert(None);
            }
        }
    }
    let mut output = Vec::new();
    let mut frames = Vec::<(Node<'_, '_>, u32)>::new();
    let mut element_path = Vec::new();
    let mut item_index = 0_usize;
    let mut spine_index = 0_usize;
    for node in root.descendants().filter(Node::is_element) {
        if node != root {
            while frames
                .last()
                .is_some_and(|frame| Some(frame.0) != node.parent_element())
            {
                frames.pop();
                element_path.pop();
            }
            let frame = frames
                .last_mut()
                .ok_or(ReferenceIndexError::SourceMismatch)?;
            let ordinal = frame.1;
            frame.1 = frame
                .1
                .checked_add(1)
                .ok_or(ReferenceIndexError::XmlNodeLimit)?;
            element_path.push(ordinal);
        }
        if element_path.len() > budget.limits.max_xml_depth {
            return Err(ReferenceIndexError::XmlDepthLimit);
        }
        frames.push((node, 0));
        let parent = node.parent_element();
        if named(node, "item") && parent.is_some_and(|p| named(p, "manifest")) {
            let expected = inventory
                .resources
                .get(item_index)
                .ok_or(ReferenceIndexError::SourceMismatch)?;
            item_index += 1;
            let id = unqualified(node, "id").unwrap_or_default().trim();
            let href = unqualified(node, "href").unwrap_or_default().trim();
            let media = unqualified(node, "media-type")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase();
            if id != expected.id || media != expected.media_type {
                return Err(ReferenceIndexError::SourceMismatch);
            }
            if let Resolution::Internal(ref resolved, _) = resolve(path, href) {
                if expected.path.as_deref() != Some(resolved.as_str()) {
                    return Err(ReferenceIndexError::SourceMismatch);
                }
            } else if expected.path.is_some() {
                return Err(ReferenceIndexError::SourceMismatch);
            }
            push(
                &mut output,
                budget,
                node,
                &element_path,
                "href",
                ReferenceKind::Manifest,
                href,
                href,
                None,
                None,
            )?;
            if let Some(properties) = unqualified(node, "properties")
                && properties
                    .split_ascii_whitespace()
                    .any(|value| value == "cover-image")
            {
                push(
                    &mut output,
                    budget,
                    node,
                    &element_path,
                    "properties",
                    ReferenceKind::Cover,
                    properties,
                    href,
                    None,
                    None,
                )?;
            }
            for (attribute, kind) in [
                ("fallback", ReferenceKind::Fallback),
                ("media-overlay", ReferenceKind::MediaOverlay),
            ] {
                if let Some(value) = unqualified(node, attribute) {
                    let target = id_resolution(value.trim(), &ids, inventory);
                    push(
                        &mut output,
                        budget,
                        node,
                        &element_path,
                        attribute,
                        kind,
                        value,
                        value,
                        Some(target),
                        None,
                    )?;
                }
            }
        } else if named(node, "meta")
            && parent.is_some_and(|p| named(p, "metadata"))
            && unqualified(node, "name").is_some_and(|name| name.eq_ignore_ascii_case("cover"))
        {
            if let Some(value) = unqualified(node, "content") {
                let target = id_resolution(value.trim(), &ids, inventory);
                push(
                    &mut output,
                    budget,
                    node,
                    &element_path,
                    "content",
                    ReferenceKind::Cover,
                    value,
                    value,
                    Some(target),
                    None,
                )?;
            }
        } else if named(node, "itemref") && parent.is_some_and(|p| named(p, "spine")) {
            let expected = inventory
                .spine
                .get(spine_index)
                .ok_or(ReferenceIndexError::SourceMismatch)?;
            let value = unqualified(node, "idref").unwrap_or_default();
            if value.trim() != expected.idref {
                return Err(ReferenceIndexError::SourceMismatch);
            }
            let target = id_resolution(value.trim(), &ids, inventory);
            push(
                &mut output,
                budget,
                node,
                &element_path,
                "idref",
                ReferenceKind::Spine,
                value,
                value,
                Some(target),
                Some(vec![expected.reading_order]),
            )?;
            spine_index += 1;
        } else if (named(node, "link") && parent.is_some_and(|p| named(p, "metadata")))
            || (named(node, "reference") && parent.is_some_and(|p| named(p, "guide")))
        {
            if let Some(value) = unqualified(node, "href") {
                let cover = unqualified(node, "type").is_some_and(|value| {
                    value.split_ascii_whitespace().any(|part| part == "cover")
                });
                push(
                    &mut output,
                    budget,
                    node,
                    &element_path,
                    "href",
                    if cover {
                        ReferenceKind::Cover
                    } else {
                        ReferenceKind::PackageLink
                    },
                    value,
                    value,
                    None,
                    None,
                )?;
            }
        } else if named(node, "spine")
            && node.parent_element() == Some(root)
            && let Some(value) = unqualified(node, "toc")
        {
            let target = id_resolution(value.trim(), &ids, inventory);
            push(
                &mut output,
                budget,
                node,
                &element_path,
                "toc",
                ReferenceKind::PackageLink,
                value,
                value,
                Some(target),
                None,
            )?;
        }
    }
    if item_index != inventory.resources.len() || spine_index != inventory.spine.len() {
        return Err(ReferenceIndexError::SourceMismatch);
    }
    Ok(output)
}

fn id_resolution(
    value: &str,
    ids: &HashMap<&str, Option<usize>>,
    inventory: &EpubInventory,
) -> Resolution {
    match ids.get(value) {
        Some(Some(index)) => inventory.resources[*index]
            .path
            .as_ref()
            .map_or(Resolution::MissingIdentifier, |path| {
                Resolution::Internal(path.clone(), None)
            }),
        Some(None) => Resolution::Ambiguous,
        None => Resolution::MissingIdentifier,
    }
}
fn named(node: Node<'_, '_>, name: &str) -> bool {
    node.is_element() && node.tag_name().namespace() == Some(OPF) && node.tag_name().name() == name
}
#[allow(clippy::too_many_arguments)]
fn push(
    output: &mut Vec<RawReference>,
    budget: &mut Budget,
    node: Node<'_, '_>,
    path: &[u32],
    attribute: &str,
    kind: ReferenceKind,
    raw: &str,
    value: &str,
    resolved: Option<Resolution>,
    spine_override: Option<Vec<usize>>,
) -> Result<(), ReferenceIndexError> {
    let (line, column) = node_position(node);
    budget
        .reference(raw.len())
        .map_err(|error| at("", Some(line), Some(column), error))?;
    if value.len() > budget.limits.max_reference_bytes {
        return Err(at(
            "",
            Some(line),
            Some(column),
            ReferenceIndexError::ReferenceByteLimit,
        ));
    }
    output.push(RawReference {
        location: ReferenceLocation {
            element_ordinals: path.to_vec(),
            attribute_namespace: None,
            attribute_local_name: Some(attribute.to_owned()),
            candidate_ordinal: 0,
            line,
            column,
            decoded_byte_range: None,
        },
        kind,
        value: value.to_owned(),
        raw: raw.to_owned(),
        resolved,
        spine_override,
        inventory_occurrence: None,
        source_occurrences: Vec::new(),
    });
    Ok(())
}
