use super::*;
use nt_domain::chapter_map::NavigationSource;
use nt_domain::volume_import::CapturedNavigationLabel;

const NCX_NAMESPACE: &str = "http://www.daisy.org/z3986/2005/ncx/";

impl SourceExtraction {
    pub fn read_captured_navigation_labels(
        archive: &mut EpubZip<impl Read + Seek>,
        trees: &[MapNavigationTree],
        working_bytes: usize,
    ) -> Result<Vec<CapturedNavigationLabel>, CapturedSourceError> {
        let mut space = BlockSpace {
            remaining: working_bytes.min(MAX_IMPORT_WORKING_BYTES),
        };
        let mut output = Vec::new();
        for tree in trees.iter().filter(|tree| !tree.items.is_empty()) {
            let entry = archive
                .entry_by_path(&tree.resource_path)
                .map_err(|_| Error::InvalidSource)?
                .ok_or(Error::InvalidSource)?;
            if entry.declared_size() > (space.remaining / 7) as u64 {
                return Err(Error::ResourceLimit);
            }
            let bytes = archive
                .read_entry(entry.id())
                .map_err(|_| Error::InvalidSource)?;
            let labels = Self::visit_captured_navigation_labels(
                &bytes,
                CapturedSourceRequest {
                    canonical_path: &tree.resource_path,
                    epub_sha256: [0; 32],
                    parser_version: PARSER_VERSION,
                    is_navigation_document: true,
                    working_bytes: space.remaining,
                },
                tree.source,
            )?;
            for label in labels {
                let mut item = tree.items.as_slice();
                let mut selected = None;
                for &ordinal in &label.item_ordinals {
                    let current = item.get(ordinal as usize).ok_or(Error::EvidenceMismatch)?;
                    selected = Some(current);
                    item = &current.children;
                }
                let selected = selected.ok_or(Error::EvidenceMismatch)?;
                if label.text != selected.label || label.raw_href != selected.raw_href {
                    return Err(Error::EvidenceMismatch);
                }
                space.charge(
                    label.canonical_path.len()
                        + label.text.len()
                        + label.raw_href.as_ref().map_or(0, String::len)
                        + (label.element_ordinals.len() + label.item_ordinals.len()) * 4,
                )?;
                visit_vec(&mut space, &mut output, label)?;
            }
        }
        Ok(output)
    }

    /// Locate readable TOC labels in their original DOM. Item ordinals refer
    /// to the already parsed navigation tree, never to guessed title matches.
    pub fn visit_captured_navigation_labels(
        bytes: &[u8],
        request: CapturedSourceRequest<'_>,
        source: NavigationSource,
    ) -> Result<Vec<CapturedNavigationLabel>, CapturedSourceError> {
        bounded_xml(bytes, &request, |document, mut space| {
            validate_xml_depth(document.root(), request.canonical_path).map_err(source_error)?;
            TreeSpace::measure(document)?.reserve_tree(&mut space)?;
            let root = document.root_element();
            let list = match source {
                NavigationSource::Epub3Nav => {
                    let nav = root.descendants().find(|node| {
                        named(*node, XHTML_NAMESPACE, "nav")
                            && node
                                .attribute((EPUB_NAMESPACE, "type"))
                                .is_some_and(|value| {
                                    value.split_ascii_whitespace().any(|part| part == "toc")
                                })
                    });
                    nav.and_then(|node| {
                        node.children()
                            .find(|node| named(*node, XHTML_NAMESPACE, "ol"))
                    })
                }
                NavigationSource::Epub2Ncx => root
                    .descendants()
                    .find(|node| named(*node, NCX_NAMESPACE, "navMap")),
            };
            let mut labels = Vec::new();
            if let Some(list) = list {
                walk(list, root, source, &[], &request, &mut space, &mut labels)?;
            }
            Ok(labels)
        })
    }
}

fn named(node: Node<'_, '_>, namespace: &str, name: &str) -> bool {
    node.is_element()
        && node.tag_name().namespace() == Some(namespace)
        && node.tag_name().name() == name
}

fn walk(
    list: Node<'_, '_>,
    root: Node<'_, '_>,
    source: NavigationSource,
    parent: &[u32],
    request: &CapturedSourceRequest<'_>,
    space: &mut BlockSpace,
    output: &mut Vec<CapturedNavigationLabel>,
) -> Result<(), Error> {
    if parent.len() >= 256 {
        return Err(Error::ResourceLimit);
    }
    let (namespace, item_name) = match source {
        NavigationSource::Epub3Nav => (XHTML_NAMESPACE, "li"),
        NavigationSource::Epub2Ncx => (NCX_NAMESPACE, "navPoint"),
    };
    for (index, item) in list
        .children()
        .filter(|node| named(*node, namespace, item_name))
        .enumerate()
    {
        let mut item_path = space.allocate(parent.len() + 1)?;
        item_path.extend_from_slice(parent);
        item_path.push(u32::try_from(index).map_err(|_| Error::ResourceLimit)?);
        let (label, href, nested) = match source {
            NavigationSource::Epub3Nav => {
                let label = item.children().find(|node| {
                    named(*node, XHTML_NAMESPACE, "a") || named(*node, XHTML_NAMESPACE, "span")
                });
                (
                    Some(label.unwrap_or(item)),
                    label
                        .filter(|node| named(*node, XHTML_NAMESPACE, "a"))
                        .and_then(|node| node.attribute("href")),
                    item.children()
                        .find(|node| named(*node, XHTML_NAMESPACE, "ol")),
                )
            }
            NavigationSource::Epub2Ncx => {
                let label = item
                    .children()
                    .find(|node| named(*node, NCX_NAMESPACE, "navLabel"))
                    .and_then(|label| {
                        label
                            .descendants()
                            .find(|node| named(*node, NCX_NAMESPACE, "text"))
                    });
                let href = item
                    .children()
                    .find(|node| named(*node, NCX_NAMESPACE, "content"))
                    .and_then(|node| node.attribute("src"));
                (label, href, Some(item))
            }
        };
        if let Some(label) = label {
            let mut pending_space = false;
            // Match the established navigation whitespace projection while
            // bounding allocations before copying any source text.
            let length = label
                .descendants()
                .filter(Node::is_text)
                .filter_map(|node| node.text())
                .try_fold(0usize, |sum, value| {
                    sum.checked_add(value.len()).ok_or(Error::ResourceLimit)
                })?;
            let mut text = space.string(length)?;
            for node in label
                .descendants()
                .filter(Node::is_text)
                .filter_map(|node| node.text())
            {
                for character in node.chars() {
                    if character.is_whitespace() {
                        pending_space = !text.is_empty();
                    } else {
                        if pending_space {
                            text.push(' ');
                            pending_space = false;
                        }
                        text.push(character);
                    }
                }
            }
            let path = element_path(root, label).ok_or(Error::InvalidSource)?;
            let mut element_ordinals = space.allocate(path.len())?;
            element_ordinals.extend(path);
            let canonical_path = space.string(request.canonical_path.len())?;
            let mut canonical_path = canonical_path;
            canonical_path.push_str(request.canonical_path);
            let raw_href = href
                .map(|href| {
                    let mut text = space.string(href.len())?;
                    text.push_str(href);
                    Ok(text)
                })
                .transpose()?;
            let mut identity = space.allocate(item_path.len())?;
            identity.extend_from_slice(&item_path);
            visit_vec(
                space,
                output,
                CapturedNavigationLabel {
                    canonical_path,
                    item_ordinals: identity,
                    element_ordinals,
                    text,
                    raw_href,
                },
            )?;
        }
        if let Some(nested) = nested {
            walk(nested, root, source, &item_path, request, space, output)?;
        }
    }
    Ok(())
}
