//! Replay CP04 source semantics for persisted reads in compact text runs.
//! This is a separate consumer of the original token/locator helpers.

#[path = "source_proof/navigation_labels.rs"]
mod navigation_labels;
#[path = "source_proof/proof_runs.rs"]
mod proof_runs;

use nt_domain::volume_import::{
    CapturedBlock, CapturedBlockPlacement, CapturedScopeElement, CapturedSourceError,
    CapturedSourceRequest, CapturedUnitRun, MAX_IMPORT_WORKING_BYTES,
};
use std::io::{Read, Seek};
use std::mem::size_of;

use self::proof_runs::replay_block;
use super::{
    BaseContext, BlockIdentityRecord, BlockLocator, DEFAULT_XML_MAX_BYTES, DEFAULT_XML_MAX_NODES,
    Document, EPUB_NAMESPACE, ElementLocator, EpubInventory, EpubZip, HashMap, HashSet,
    MAX_XML_ELEMENT_DEPTH, MapNavigationTree, MetadataBuilder, NavigationEvidence, Node, NodeId,
    PARSER_VERSION, RawBlock, SVG_NAMESPACE, SourceAnchor, SourceBlockId, SourceDiagnostic,
    SourceExtraction, SourceExtractionError, SourceUnitKind, Token, XHTML_NAMESPACE,
    XmlFailureKind, align_text_segments, block_locator, collect_block_tokens,
    element_fragment_counts, element_fragments, element_id_counts, element_path,
    extract_navigation, extract_source_with_navigation, image_page_block, is_source_block,
    is_xhtml_element, is_xml_resource_limit, node_at_path, scan_cdata_content, svg_anchors,
    validate_xml_depth, with_xml_document,
};
use crate::effective_base::{MAX_REFERENCE_BYTES, first_html_base};
use crate::reference_index::native_space;
use crate::xml::{XmlFailure, with_decoded_xml_document};

type Error = CapturedSourceError;

// Allow for parser node and attribute arrays, their geometric growth and
// lexical evidence. The parsed tree gets a separate accounting pass before
// invoking the token and base helpers.
const XML_ARRAY_BYTES_PER_NODE: usize = 384;

impl SourceExtraction {
    pub fn extract_captured(
        archive: &mut EpubZip<impl Read + Seek>,
        inventory: &EpubInventory,
        epub_sha256: [u8; 32],
        navigation_working_bytes: usize,
    ) -> Result<Self, SourceExtractionError> {
        extract_source_with_navigation(
            archive,
            inventory,
            epub_sha256,
            PARSER_VERSION,
            |archive, inventory| {
                Self::read_captured_navigation(archive, inventory, navigation_working_bytes)
            },
        )
    }

    /// Read the original decoded XHTML once per pass, giving the verifier a
    /// borrowed view of each compact block. Hashes and locator projections
    /// come from the original XML, never from the saved block being checked.
    pub fn visit_captured_xhtml(
        bytes: &[u8],
        request: CapturedSourceRequest<'_>,
        mut visit: impl FnMut(&CapturedBlock) -> Result<(), CapturedSourceError>,
    ) -> Result<(Vec<SourceAnchor>, Vec<SourceDiagnostic>), CapturedSourceError> {
        bounded_xml(bytes, &request, |document, space| {
            visit_document(document, &request, space, &mut visit)
        })
    }

    /// Replay the one atomic source block of a linear image page. The caller
    /// also checks the archived image object and its resource dependencies.
    pub fn visit_captured_image_page(
        svg_bytes: Option<&[u8]>,
        request: CapturedSourceRequest<'_>,
        spine_position: usize,
        mut visit: impl FnMut(&CapturedBlock) -> Result<(), CapturedSourceError>,
    ) -> Result<Vec<SourceAnchor>, CapturedSourceError> {
        if request.parser_version != PARSER_VERSION {
            return Err(Error::UnsupportedParser);
        }
        let path = request.canonical_path;
        let page = image_page_block(
            path,
            request.epub_sha256,
            request.parser_version,
            spine_position,
        )
        .map_err(source_error)?;
        let anchors = svg_bytes
            .map(|bytes| {
                bounded_xml(bytes, &request, |document, mut space| {
                    validate_xml_depth(document.root(), path).map_err(source_error)?;
                    let root = document.root_element();
                    if root.tag_name().name() != "svg"
                        || root.tag_name().namespace() != Some(SVG_NAMESPACE)
                    {
                        return Err(Error::InvalidRoot);
                    }
                    let tree = TreeSpace::measure(document)?;
                    tree.reserve_tree(&mut space)?;
                    tree.reserve_tokens(path, &HashMap::new(), &mut space)?;
                    let counts = element_fragment_counts(root);
                    Ok(svg_anchors(root, path, &page, &counts))
                })
            })
            .transpose()?
            .unwrap_or_default();
        let mut units = page.units.into_iter();
        let (Some(unit), None) = (units.next(), units.next()) else {
            return Err(Error::InvalidSource);
        };
        let SourceUnitKind::Protected(protected) = unit.kind else {
            return Err(Error::InvalidSource);
        };
        visit(&CapturedBlock {
            id: page.id,
            element_name: page.element_name,
            locator: page.locator,
            placement: CapturedBlockPlacement::default(),
            content_hash: page.content_hash,
            unit_count: 1,
            runs: vec![CapturedUnitRun::Protected {
                unit: protected,
                spans: unit.spans,
            }],
            inline_metadata: page.inline_metadata,
            attribute_text: page.attribute_text,
        })?;
        Ok(anchors)
    }

    /// Read the same navigation evidence the mapping producer used.
    pub fn read_captured_navigation(
        archive: &mut EpubZip<impl Read + Seek>,
        inventory: &EpubInventory,
        working_bytes: usize,
    ) -> Result<NavigationEvidence, SourceExtractionError> {
        native_space::scoped(working_bytes, || {
            native_space::retain_navigation_merge(inventory)
                .map_err(|_| SourceExtractionError::WorkingMemoryLimit)?;
            extract_navigation(archive, inventory)
        })
        .map_err(|_| SourceExtractionError::WorkingMemoryLimit)?
    }
}

fn bounded_xml<T>(
    bytes: &[u8],
    request: &CapturedSourceRequest<'_>,
    visit: impl FnOnce(&Document<'_>, BlockSpace) -> Result<T, Error>,
) -> Result<T, Error> {
    if request.parser_version != PARSER_VERSION {
        return Err(Error::UnsupportedParser);
    }
    let mut space = BlockSpace {
        remaining: request.working_bytes.min(MAX_IMPORT_WORKING_BYTES),
    };
    space.charge(bytes.len())?;
    let xml_bytes = DEFAULT_XML_MAX_BYTES.min(space.remaining / 6);
    if xml_bytes == 0 || bytes.len() > xml_bytes {
        return Err(Error::ResourceLimit);
    }
    let xml_nodes = node_limit(space.remaining - 4 * xml_bytes);
    if xml_nodes == 0 {
        return Err(Error::ResourceLimit);
    }
    native_space::scoped_replay(space.remaining, || {
        with_xml_document(bytes, xml_bytes, xml_nodes, |document| {
            let remaining = native_space::remaining()?;
            visit(document, BlockSpace { remaining })
        })
        .map_err(captured_xml_failure)?
    })?
}

fn captured_xml_failure(failure: XmlFailure) -> Error {
    match failure.kind {
        kind if is_xml_resource_limit(kind) => Error::ResourceLimit,
        XmlFailureKind::UnsupportedEncoding => Error::UnsupportedEncoding,
        XmlFailureKind::ExternalEntity | XmlFailureKind::UnknownEntity => Error::UnsafeXml,
        _ => Error::InvalidXml {
            line: failure.line,
            column: failure.column,
        },
    }
}

fn node_limit(bytes: usize) -> u32 {
    (bytes / XML_ARRAY_BYTES_PER_NODE).min(DEFAULT_XML_MAX_NODES as usize) as u32
}

fn visit_document(
    document: &Document<'_>,
    request: &CapturedSourceRequest<'_>,
    mut space: BlockSpace,
    visit: &mut impl FnMut(&CapturedBlock) -> Result<(), Error>,
) -> Result<(Vec<SourceAnchor>, Vec<SourceDiagnostic>), Error> {
    let canonical_path = request.canonical_path;
    validate_xml_depth(document.root(), canonical_path).map_err(source_error)?;
    let root = document.root_element();
    if root.tag_name().name() != "html" || root.tag_name().namespace() != Some(XHTML_NAMESPACE) {
        return Err(Error::InvalidRoot);
    }

    let inventory = TreeSpace::measure(document)?;
    inventory.reserve_tree(&mut space)?;
    let text_segments = replay_lexical_pieces(document, &mut space)?;
    inventory.reserve_tokens(canonical_path, &text_segments, &mut space)?;
    let base = space.with_native(|| {
        BaseContext::from_document(canonical_path, document).map_err(|_| Error::ResourceLimit)
    })?;
    let id_counts = element_id_counts(root);
    let fragment_counts = element_fragment_counts(root);
    let body = root
        .descendants()
        .find(|node| is_xhtml_element(*node, "body"))
        .unwrap_or(root);
    let body_path = element_path(root, body).unwrap_or_default();
    let excluded = if request.is_navigation_document {
        navigation_list_path(root)
    } else {
        None
    };
    let mut collector = Collector {
        request,
        base: &base,
        id_counts: &id_counts,
        fragment_counts: &fragment_counts,
        text_segments: &text_segments,
        excluded,
        anchors: Vec::new(),
        first_readable: None,
        space,
        visit,
    };
    collector.walk(body, &body_path, &[])?;
    if let Some(first) = collector.first_readable {
        for fragment in element_fragments(root) {
            if fragment_counts.get(&fragment) == Some(&1) {
                collector.anchor(
                    first,
                    fragment,
                    ElementLocator {
                        block: BlockLocator::NodePath {
                            element_ordinals: Vec::new(),
                            order: 0,
                        },
                        element_ordinals: Vec::new(),
                    },
                    0,
                )?;
            }
        }
    }
    let mut diagnostics = Vec::new();
    for (id, &count) in &id_counts {
        if count > 1 {
            let path = collector.space.copy_text(canonical_path)?;
            let id = collector.space.copy_text(id)?;
            visit_vec(
                &mut collector.space,
                &mut diagnostics,
                SourceDiagnostic::DuplicateXhtmlId {
                    canonical_path: path,
                    id,
                },
            )?;
        }
    }
    // Raw HashMap order is deliberately irrelevant to duplicate-ID evidence.
    diagnostics.sort_by(|left, right| duplicate_id(left).cmp(duplicate_id(right)));
    Ok((collector.anchors, diagnostics))
}

fn duplicate_id(diagnostic: &SourceDiagnostic) -> &str {
    match diagnostic {
        SourceDiagnostic::DuplicateXhtmlId { id, .. } => id,
        _ => "",
    }
}

fn navigation_list_path(root: Node<'_, '_>) -> Option<Vec<u32>> {
    let toc = root.descendants().find(|node| {
        is_xhtml_element(*node, "nav")
            && node
                .attribute((EPUB_NAMESPACE, "type"))
                .is_some_and(|value| value.split_ascii_whitespace().any(|token| token == "toc"))
    })?;
    let list = toc
        .descendants()
        .find(|node| is_xhtml_element(*node, "ol"))?;
    element_path(root, list)
}

struct Collector<'a, V> {
    request: &'a CapturedSourceRequest<'a>,
    base: &'a BaseContext,
    id_counts: &'a HashMap<String, usize>,
    fragment_counts: &'a HashMap<String, usize>,
    text_segments: &'a HashMap<NodeId, Vec<String>>,
    excluded: Option<Vec<u32>>,
    anchors: Vec<SourceAnchor>,
    first_readable: Option<SourceBlockId>,
    space: BlockSpace,
    visit: &'a mut V,
}

impl<V: FnMut(&CapturedBlock) -> Result<(), Error>> Collector<'_, V> {
    fn walk(
        &mut self,
        node: Node<'_, '_>,
        path: &[u32],
        inherited: &[MetadataBuilder],
    ) -> Result<(), Error> {
        if self.excluded.as_deref() == Some(path) {
            return Ok(());
        }
        if is_source_block(node) {
            return self.split_block(node, path, inherited);
        }
        for (ordinal, child) in node.children().filter(Node::is_element).enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(ordinal as u32);
            self.walk(child, &child_path, inherited)?;
        }
        Ok(())
    }

    fn split_block(
        &mut self,
        node: Node<'_, '_>,
        path: &[u32],
        inherited: &[MetadataBuilder],
    ) -> Result<(), Error> {
        let provisional = block_locator(node, path, 0, self.id_counts);
        let RawBlock { tokens, metadata } = self.space.with_native(|| {
            collect_block_tokens(
                node,
                &provisional,
                self.fragment_counts,
                self.request.canonical_path,
                inherited,
                self.text_segments,
                self.base,
            )
            .map_err(source_error)
        })?;
        let mut pending = Vec::new();
        let mut segment_order = 0;
        for token in tokens {
            let Token::NestedBlock {
                relative_path,
                metadata: active,
            } = token
            else {
                pending.push(token);
                continue;
            };
            if self.emit(
                node,
                path,
                segment_order,
                std::mem::take(&mut pending),
                &metadata,
            )? {
                segment_order += 1;
            }
            let child = node_at_path(node, &relative_path).ok_or(Error::InvalidSource)?;
            let mut child_path = path.to_vec();
            child_path.extend(relative_path);
            let inherited: Vec<_> = active
                .iter()
                .map(|&index| {
                    let mut builder = metadata[index].clone();
                    builder.rebase_to_segment = false;
                    builder
                })
                .collect();
            self.walk(child, &child_path, &inherited)?;
        }
        self.emit(node, path, segment_order, pending, &metadata)?;
        Ok(())
    }

    fn emit(
        &mut self,
        node: Node<'_, '_>,
        path: &[u32],
        order: u32,
        tokens: Vec<Token>,
        metadata: &[MetadataBuilder],
    ) -> Result<bool, Error> {
        if !tokens.iter().any(|token| {
            matches!(
                token,
                Token::Text { .. } | Token::Atomic { .. } | Token::Anchor { .. }
            )
        }) {
            return Ok(false);
        }
        let locator = if order == 0 {
            block_locator(node, path, order, self.id_counts)
        } else {
            BlockLocator::NodePath {
                element_ordinals: path.to_vec(),
                order,
            }
        };
        let identity = BlockIdentityRecord {
            epub_sha256: self.request.epub_sha256,
            parser_version: self.request.parser_version.to_owned(),
            canonical_path: self.request.canonical_path.to_owned(),
            locator,
        };
        let mut space = self.space;
        let placement = block_placement(node, path, &mut space)?;
        let (block, anchors) = replay_block(
            tokens,
            metadata,
            identity,
            node.tag_name().name(),
            placement,
            space,
        )?;
        (self.visit)(&block)?;
        if block.unit_count > 0 && self.first_readable.is_none() {
            self.first_readable = Some(block.id);
        }
        for (fragment, locator, offset) in anchors {
            self.anchor(block.id, fragment, locator, offset)?;
        }
        Ok(true)
    }

    fn anchor(
        &mut self,
        block_id: SourceBlockId,
        fragment: String,
        locator: ElementLocator,
        unit_offset: u32,
    ) -> Result<(), Error> {
        let path = self.space.copy_text(self.request.canonical_path)?;
        let is_unique = self.fragment_counts.get(&fragment) == Some(&1);
        visit_vec(
            &mut self.space,
            &mut self.anchors,
            SourceAnchor {
                canonical_path: path,
                fragment,
                locator,
                block_id,
                unit_offset,
                is_unique,
            },
        )
    }
}

fn block_placement(
    node: Node<'_, '_>,
    path: &[u32],
    space: &mut BlockSpace,
) -> Result<CapturedBlockPlacement, Error> {
    let mut element_ordinals = space.allocate(path.len())?;
    element_ordinals.extend_from_slice(path);
    let mut context = Vec::new();
    for (distance, node) in node.ancestors().take_while(Node::is_element).enumerate() {
        let epub_type = node.attribute((EPUB_NAMESPACE, "type"));
        let role = crate::effective_base::unqualified_attribute(node, "role");
        let figure_or_table = is_xhtml_element(node, "figure") || is_xhtml_element(node, "table");
        if epub_type.is_none() && role.is_none() && !figure_or_table {
            continue;
        }
        let depth = path
            .len()
            .checked_sub(distance)
            .ok_or(Error::InvalidSource)?;
        let mut ordinals = space.allocate(depth)?;
        ordinals.extend_from_slice(&path[..depth]);
        let element = CapturedScopeElement {
            element_ordinals: ordinals,
            epub_type: epub_type.map(|value| space.copy_text(value)).transpose()?,
            role: role.map(|value| space.copy_text(value)).transpose()?,
            figure_or_table,
        };
        visit_vec(space, &mut context, element)?;
    }
    Ok(CapturedBlockPlacement {
        element_ordinals,
        context,
    })
}

fn source_error(error: SourceExtractionError) -> Error {
    match error {
        SourceExtractionError::XmlDepthLimit { .. }
        | SourceExtractionError::BaseResolutionLimit { .. } => Error::ResourceLimit,
        _ => Error::InvalidSource,
    }
}

/// Reuse the lexical CDATA splitter, with a separate node budget for the
/// virtual XML document. Normal text has no virtual tree.
fn replay_lexical_pieces(
    original: &Document<'_>,
    space: &mut BlockSpace,
) -> Result<HashMap<NodeId, Vec<String>>, Error> {
    let input = original.input_text();
    let possible = input.matches("<![CDATA[").count();
    if possible == 0 {
        return Ok(HashMap::new());
    }
    let mut boundaries = space.allocate(possible.checked_mul(2).ok_or(Error::ResourceLimit)?)?;
    scan_cdata_content(input.as_bytes(), 0, input.len(), true, &mut boundaries);
    boundaries.sort_unstable();
    boundaries.dedup();
    if boundaries.is_empty() {
        return Ok(HashMap::new());
    }

    const SEPARATOR: &str = "<!---->";
    let extra = array_bytes::<[u8; SEPARATOR.len()]>(boundaries.len())?;
    let virtual_length = input.len().checked_add(extra).ok_or(Error::ResourceLimit)?;
    let mut augmented = space.string(virtual_length)?;
    // The index currently has one slot per injected separator.
    space.charge(array_bytes::<[usize; 4]>(boundaries.len())?)?;
    let mut separators = HashSet::new();
    separators
        .try_reserve(boundaries.len())
        .map_err(|_| Error::ResourceLimit)?;
    let mut offset = 0;
    for boundary in boundaries {
        augmented.push_str(&input[offset..boundary]);
        separators.insert(augmented.len());
        augmented.push_str(SEPARATOR);
        offset = boundary;
    }
    augmented.push_str(&input[offset..]);
    let max_nodes = node_limit(space.remaining / 2);
    if max_nodes == 0 {
        return Err(Error::ResourceLimit);
    }
    // Both XML documents and the existing lexical projection can be live.
    let (segments, used) = space.with_native(|| {
        with_decoded_xml_document(
            &augmented,
            DEFAULT_XML_MAX_BYTES.saturating_add(extra),
            max_nodes,
            |parsed| {
                let allowance = native_space::remaining()?;
                let mut extra_space = BlockSpace {
                    remaining: allowance,
                };
                let virtual_inventory = TreeSpace::measure(parsed)?;
                virtual_inventory.reserve_tree(&mut extra_space)?;
                extra_space.charge(array_bytes::<[usize; 8]>(virtual_inventory.nodes)?)?;
                extra_space.charge(virtual_inventory.text_bytes)?;
                let mut segments = HashMap::new();
                align_text_segments(original.root(), parsed.root(), &separators, &mut segments)
                    .map_err(|_| Error::InvalidSource)?;
                Ok((segments, allowance - extra_space.remaining))
            },
        )
        .map_err(captured_xml_failure)?
    })?;
    space.charge(used)?;
    Ok(segments)
}

struct TreeSpace {
    nodes: usize,
    max_depth: usize,
    long_locator: usize,
    text_bytes: usize,
    input_bytes: usize,
    attribute_count: usize,
    attribute_bytes: usize,
    base_declarations: usize,
    image_count: usize,
}

impl TreeSpace {
    fn measure(document: &Document<'_>) -> Result<Self, Error> {
        let mut tree = Self {
            nodes: 0,
            max_depth: 0,
            long_locator: 0,
            text_bytes: 0,
            input_bytes: document.input_text().len(),
            attribute_count: 0,
            attribute_bytes: 0,
            base_declarations: usize::from(first_html_base(document).is_some()),
            image_count: 0,
        };
        for node in document.descendants() {
            tree.nodes += 1;
            let depth = node.ancestors().take(MAX_XML_ELEMENT_DEPTH + 3).count();
            tree.max_depth = tree.max_depth.max(depth);
            if node.is_text() {
                tree.text_bytes = tree
                    .text_bytes
                    .checked_add(node.text().map_or(0, str::len))
                    .ok_or(Error::ResourceLimit)?;
            }
            if node.is_element() {
                tree.image_count += usize::from(matches!(node.tag_name().name(), "img" | "image"));
                tree.base_declarations += usize::from(
                    node.attribute(("http://www.w3.org/XML/1998/namespace", "base"))
                        .is_some(),
                );
                for attribute in node.attributes() {
                    tree.attribute_count += 1;
                    tree.attribute_bytes = tree
                        .attribute_bytes
                        .checked_add(attribute.name().len() + attribute.value().len())
                        .ok_or(Error::ResourceLimit)?;
                    if attribute.name() == "id" {
                        tree.long_locator = tree.long_locator.max(attribute.value().len());
                    }
                }
            }
        }
        Ok(tree)
    }

    fn reserve_tree(&self, space: &mut BlockSpace) -> Result<(), Error> {
        space.charge_product(self.nodes, XML_ARRAY_BYTES_PER_NODE)?;
        space.charge_product(self.attribute_count, 160)?;
        space.charge_product(self.input_bytes, 2)?;
        space.charge_product(self.text_bytes, 2)?;
        space.charge_product(self.attribute_bytes, 2)
    }

    fn reserve_tokens(
        &self,
        canonical_path: &str,
        text_segments: &HashMap<NodeId, Vec<String>>,
        space: &mut BlockSpace,
    ) -> Result<(), Error> {
        let extra = text_segments.values().try_fold(0usize, |count, segments| {
            count
                .checked_add(segments.len())
                .ok_or(Error::ResourceLimit)
        })?;
        let locators = self.long_locator.max(array_bytes::<u32>(self.max_depth)?);
        let record = 4 * size_of::<Token>() + 6 * locators + self.max_depth * size_of::<usize>();
        space.charge_product(
            self.nodes.checked_add(extra).ok_or(Error::ResourceLimit)?,
            record,
        )?;
        space.charge_product(self.text_bytes, 2)?;
        space.charge_product(self.attribute_bytes, 6)?;
        // Root base allocation, derived anchors and the copies made around
        // resolve_href. That helper separately accounts its component buffers.
        space.charge(canonical_path.len().saturating_add(128))?;
        space.charge_product(self.base_declarations, 6 * MAX_REFERENCE_BYTES)?;
        space.charge_product(self.image_count, 2 * MAX_REFERENCE_BYTES)?;
        space.charge_product(canonical_path.len(), self.attribute_count)
    }
}

#[derive(Clone, Copy)]
struct BlockSpace {
    remaining: usize,
}

impl BlockSpace {
    /// Include this visitor's reserved space while calling bounded native
    /// helpers. Keep any new native retained charges out of the local remainder.
    fn with_native<T>(&mut self, body: impl FnOnce() -> Result<T, Error>) -> Result<T, Error> {
        let local = native_space::remaining()?
            .checked_sub(self.remaining)
            .ok_or(Error::ResourceLimit)?;
        let _projection = native_space::hold(local).map_err(|_| Error::ResourceLimit)?;
        let value = body()?;
        self.remaining = native_space::remaining()?;
        Ok(value)
    }

    fn charge(&mut self, bytes: usize) -> Result<(), Error> {
        self.remaining = self
            .remaining
            .checked_sub(bytes)
            .ok_or(Error::ResourceLimit)?;
        Ok(())
    }

    fn charge_product(&mut self, count: usize, bytes: usize) -> Result<(), Error> {
        self.charge(count.checked_mul(bytes).ok_or(Error::ResourceLimit)?)
    }

    fn allocate<T>(&mut self, count: usize) -> Result<Vec<T>, Error> {
        self.charge(array_bytes::<T>(count)?)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| Error::ResourceLimit)?;
        Ok(values)
    }

    fn string(&mut self, bytes: usize) -> Result<String, Error> {
        self.charge(bytes)?;
        let mut value = String::new();
        value
            .try_reserve_exact(bytes)
            .map_err(|_| Error::ResourceLimit)?;
        Ok(value)
    }

    fn copy_text(&mut self, text: &str) -> Result<String, Error> {
        let mut value = self.string(text.len())?;
        value.push_str(text);
        Ok(value)
    }
}

fn array_bytes<T>(count: usize) -> Result<usize, Error> {
    count
        .checked_mul(size_of::<T>())
        .ok_or(Error::ResourceLimit)
}

fn visit_vec<T>(space: &mut BlockSpace, values: &mut Vec<T>, value: T) -> Result<(), Error> {
    if values.len() == values.capacity() {
        let extra = values.capacity().max(1);
        space.charge(array_bytes::<T>(extra)?)?;
        values
            .try_reserve_exact(extra)
            .map_err(|_| Error::ResourceLimit)?;
    }
    values.push(value);
    Ok(())
}

#[test]
fn compact_projection_and_uri_resolution_share_one_allowance() {
    const ALLOWANCE: usize = 12 * 1024;
    const PATH: &str = "OPS/ch.xhtml";
    const HREF: &str = "art.png";
    let remaining = native_space::scoped_replay(ALLOWANCE, || {
        let mut space = BlockSpace {
            remaining: ALLOWANCE,
        };
        space.charge(4 * 1024)?;
        let found = space.with_native(|| {
            crate::package::resolve_href(PATH, HREF).map_err(|_| Error::ResourceLimit)
        })?;
        assert_eq!(found.path.as_deref(), Some("OPS/art.png"));
        Ok::<_, Error>(space.remaining)
    })
    .unwrap()
    .unwrap();
    assert_eq!(remaining, 8 * 1024 - 8 * (PATH.len() + HREF.len()));

    let limited = native_space::scoped_replay(ALLOWANCE, || {
        let mut space = BlockSpace {
            remaining: ALLOWANCE,
        };
        space.charge(8 * 1024)?;
        space.with_native(|| {
            crate::package::resolve_href(PATH, HREF).map_err(|_| Error::ResourceLimit)
        })
    });
    assert!(matches!(limited, Err(Error::ResourceLimit)));
}

#[test]
fn compact_cdata_replay_shares_the_live_parser_allowance() {
    let xml = b"<p xmlns='urn:book'>chapter <![CDATA[one]]></p>";
    let mut request = CapturedSourceRequest {
        canonical_path: "chapter.xhtml",
        epub_sha256: [0; 32],
        parser_version: PARSER_VERSION,
        is_navigation_document: false,
        working_bytes: 30 * 1024,
    };
    let mut reached = false;
    let result = bounded_xml(xml, &request, |document, mut space| {
        reached = true;
        replay_lexical_pieces(document, &mut space)
    });
    assert!(reached);
    assert_eq!(result, Err(Error::ResourceLimit));
    request.working_bytes = 96 * 1024;
    let pieces = bounded_xml(xml, &request, |document, mut space| {
        replay_lexical_pieces(document, &mut space)
    })
    .unwrap();
    assert!(
        pieces
            .values()
            .any(|segments| segments.concat() == "chapter one")
    );
}

#[cfg(test)]
#[path = "source_proof/tests.rs"]
mod tests;
