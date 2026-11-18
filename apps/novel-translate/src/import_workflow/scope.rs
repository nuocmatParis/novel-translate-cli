use std::collections::{BTreeMap, HashMap, HashSet};

use nt_application::source_scope::{
    ContentInput, NoteInput, NoteReference, NoteTarget, RelatedKind, RelatedText, Relation,
    ScopeError, ScopeInput, ScopeWorking, observe_map_text_within,
};
use nt_application::volume_import::{
    CapturedBlockPlacement, CapturedNavigationLabel, CapturedScopeElement,
    chapter_map::{ChapterMap, ChapterMapInput, NavigationTarget, NavigationTree},
    source_range::{
        AttributeSourceText, BlockLocator, ElementLocator, InlineMetadataKind,
        InlineSourceMetadata, ProtectedSourceUnit, SourceBlock, SourceBlockId, SourceRange,
        SourceUnitKind,
    },
    source_scope::{
        ClassificationEvidence, ContentKind, ContentObservation, DecisionContext, EvidenceKind,
        LocatedRange, ScopeMode, SourceLocation, SourceLocationKind,
    },
};
use nt_epub::{
    ReferenceIndex, ReferenceKind, ReferenceTarget, ResourceReference, SourceAnchor,
    SourceDiagnostic, SourceExtraction,
};

#[derive(Default)]
pub(super) struct ObservedScope {
    pub(super) content: Vec<ContentInput>,
    pub(super) notes: Vec<NoteInput>,
    related: Vec<RelatedText>,
    note_references: Vec<NoteReference>,
}

impl ObservedScope {
    pub fn retained_working_bytes(&self) -> Result<usize, ScopeError> {
        let mut space = ScopeWorking::new(usize::MAX);
        space.observations(
            &self.content,
            &self.notes,
            &self.related,
            &self.note_references,
        )?;
        Ok(space.used())
    }

    pub fn with_context(self, context: DecisionContext, mode: ScopeMode) -> ScopeInput {
        ScopeInput {
            context,
            mode,
            content: self.content,
            notes: self.notes,
            related: self.related,
            note_references: self.note_references,
        }
    }
}

struct Owner {
    range: SourceRange,
    key: OwnerKey,
}

#[derive(Clone, Copy)]
enum OwnerKey {
    Content(usize),
    Note(usize),
}

impl OwnerKey {
    fn location(self, observed: &ObservedScope) -> &SourceLocation {
        match self {
            Self::Content(index) => &observed.content[index].observation.location,
            Self::Note(index) => &observed.notes[index].content.observation.location,
        }
    }
}

type Owners = HashMap<SourceBlockId, Vec<Owner>>;
type ElementKey<'a> = (&'a str, &'a [u32]);
type PlacementIndex<'a> = HashMap<(&'a str, &'a BlockLocator), &'a [u32]>;

pub(super) struct ScopeBlock {
    pub id: SourceBlockId,
    pub canonical_path: String,
    pub element_name: String,
    pub locator: BlockLocator,
    pub unit_count: u32,
    pub inline_metadata: Vec<InlineSourceMetadata>,
    pub attribute_text: Vec<AttributeSourceText>,
    pub images: Vec<(u32, Vec<u32>)>,
}

impl ScopeBlock {
    fn from_eager(block: &SourceBlock, space: &mut ScopeWorking) -> Result<Self, ScopeError> {
        space.slots::<Self>(1)?;
        space.charge(
            block
                .canonical_path
                .len()
                .saturating_add(block.element_name.len()),
        )?;
        space.block_locator(&block.locator)?;
        space.slots::<InlineSourceMetadata>(block.inline_metadata.len())?;
        for item in &block.inline_metadata {
            space.element_locator(&item.locator)?;
            space.charge(item.ranges.len().saturating_mul(std::mem::size_of::<
                nt_application::volume_import::source_range::UnitRange,
            >()))?;
            match &item.kind {
                InlineMetadataKind::Formatting { element } => space.charge(element.len())?,
                InlineMetadataKind::Link { href } | InlineMetadataKind::NoteReference { href } => {
                    space.charge(href.as_ref().map_or(0, String::len))?;
                }
            }
        }
        space.slots::<AttributeSourceText>(block.attribute_text.len())?;
        for attribute in &block.attribute_text {
            space.element_locator(&attribute.locator)?;
            space.charge(
                attribute
                    .attribute
                    .len()
                    .saturating_add(attribute.value.len())
                    .saturating_add(attribute.resource_path.as_ref().map_or(0, String::len)),
            )?;
        }
        for unit in &block.units {
            if let SourceUnitKind::Protected(ProtectedSourceUnit::Image(image)) = &unit.kind {
                space.slots::<(u32, Vec<u32>)>(1)?;
                space.charge(
                    image
                        .node_path
                        .len()
                        .saturating_mul(std::mem::size_of::<u32>()),
                )?;
            }
        }
        Ok(Self {
            id: block.id,
            canonical_path: block.canonical_path.clone(),
            element_name: block.element_name.clone(),
            locator: block.locator.clone(),
            unit_count: u32::try_from(block.unit_count())
                .map_err(|_| ScopeError::InvalidSourceLocation)?,
            inline_metadata: block.inline_metadata.clone(),
            attribute_text: block.attribute_text.clone(),
            images: block
                .units
                .iter()
                .enumerate()
                .filter_map(|(offset, unit)| {
                    if let SourceUnitKind::Protected(ProtectedSourceUnit::Image(image)) = &unit.kind
                    {
                        Some((offset as u32, image.node_path.clone()))
                    } else {
                        None
                    }
                })
                .collect(),
        })
    }
}

pub(super) fn scope_input(
    source: &SourceExtraction,
    input: &ChapterMapInput,
    map: &ChapterMap,
    placements: &HashMap<SourceBlockId, CapturedBlockPlacement>,
    references: &ReferenceIndex,
    navigation_labels: &[CapturedNavigationLabel],
    space: &mut ScopeWorking,
) -> Result<ObservedScope, ScopeError> {
    let blocks = input
        .documents
        .iter()
        .flat_map(|doc| &doc.blocks)
        .map(|block| ScopeBlock::from_eager(block, space))
        .collect::<Result<Vec<_>, _>>()?;
    assemble_scope(
        &blocks,
        observe_map_text_within(input, map, space)?,
        ScopeEvidence {
            placements,
            anchors: &source.anchors,
            diagnostics: &source.diagnostics,
            references,
            navigation_labels,
            navigation: &source.navigation,
        },
        space,
    )
}

pub(super) struct ScopeEvidence<'a> {
    pub placements: &'a HashMap<SourceBlockId, CapturedBlockPlacement>,
    pub anchors: &'a [SourceAnchor],
    pub diagnostics: &'a [SourceDiagnostic],
    pub references: &'a ReferenceIndex,
    pub navigation_labels: &'a [CapturedNavigationLabel],
    pub navigation: &'a [NavigationTree],
}

pub(super) fn assemble_scope(
    source_blocks: &[ScopeBlock],
    observations: Vec<ContentInput>,
    evidence: ScopeEvidence<'_>,
    space: &mut ScopeWorking,
) -> Result<ObservedScope, ScopeError> {
    let ScopeEvidence {
        placements,
        anchors: source_anchors,
        diagnostics: source_diagnostics,
        references,
        navigation_labels,
        navigation,
    } = evidence;
    space.slots::<(SourceBlockId, &ScopeBlock)>(source_blocks.len())?;
    space.slots::<((&str, &BlockLocator), &[u32])>(source_blocks.len())?;
    let blocks: HashMap<_, _> = source_blocks
        .iter()
        .map(|block| (block.id, block))
        .collect();
    let mut structural_paths = PlacementIndex::new();
    for block in blocks.values() {
        if let Some(placement) = placements.get(&block.id) {
            structural_paths.insert(
                (&block.canonical_path, &block.locator),
                &placement.element_ordinals,
            );
        }
    }
    let mut observed = ObservedScope::default();
    let mut owners = Owners::new();
    let mut note_groups = BTreeMap::<ElementKey<'_>, Vec<ContentInput>>::new();
    let mut captions = Vec::new();
    let mut groups = BTreeMap::<ElementKey<'_>, Vec<SourceLocation>>::new();
    for mut item in observations {
        let range = only_range(&item.observation.location)?;
        let block_id = range.range().block_id();
        let block = blocks[&block_id];
        let placement = placements
            .get(&block_id)
            .ok_or(ScopeError::InvalidSourceLocation)?;
        if let Some(note) = placement.context.iter().find(|element| is_note(element)) {
            space.slots::<(ElementKey<'_>, Vec<ContentInput>)>(1)?;
            space.slots::<ContentInput>(1)?;
            note_groups
                .entry((&block.canonical_path, &note.element_ordinals))
                .or_default()
                .push(item);
            continue;
        }
        let group = enclosing_group(placement).map(|element| {
            (
                block.canonical_path.as_str(),
                element.element_ordinals.as_slice(),
            )
        });
        if matches!(block.element_name.as_str(), "caption" | "figcaption") {
            space.slots::<(Option<ElementKey<'_>>, ContentObservation)>(1)?;
            captions.push((group, item.observation));
            continue;
        }
        apply_authored_classification(
            &mut item.observation,
            &block.canonical_path,
            placement,
            space,
        )?;
        register_owner(
            &mut owners,
            &item.observation.location,
            OwnerKey::Content(observed.content.len()),
            space,
        )?;
        if let Some(group) = group {
            space.slots::<(ElementKey<'_>, Vec<SourceLocation>)>(1)?;
            space.slots::<SourceLocation>(1)?;
            space.location(&item.observation.location)?;
            let values = groups.entry(group).or_default();
            values.push(item.observation.location.clone());
        }
        space.slots::<ContentInput>(1)?;
        observed.content.push(item);
    }

    space.slots::<ElementKey<'_>>(note_groups.len())?;
    space.slots::<NoteInput>(note_groups.len())?;
    space.slots::<(ElementKey<'_>, &SourceLocation)>(note_groups.len())?;
    let mut note_keys = Vec::new();
    for (key, items) in note_groups {
        let note = merge_note(items, space)?;
        register_owner(
            &mut owners,
            &note.content.observation.location,
            OwnerKey::Note(observed.notes.len()),
            space,
        )?;
        note_keys.push(key);
        observed.notes.push(note);
    }
    let notes: BTreeMap<_, _> = note_keys
        .into_iter()
        .zip(
            observed
                .notes
                .iter()
                .map(|note| &note.content.observation.location),
        )
        .collect();
    for (group, caption) in captions {
        let relations = related_to(
            group.and_then(|key| groups.get(&key)),
            "Caption owner is unresolved",
            space,
        )?;
        space.slots::<RelatedText>(1)?;
        observed.related.push(RelatedText {
            location: caption.location,
            kind: RelatedKind::Caption,
            text: caption.excerpt,
            relations,
            evidence: caption.evidence,
        });
    }

    space.slots::<(ElementKey<'_>, &ResourceReference)>(references.references.len())?;
    space.slots::<((&str, &str), Vec<&SourceAnchor>)>(source_anchors.len())?;
    space.slots::<(&str, &str)>(source_diagnostics.len())?;
    let links: BTreeMap<_, _> = references
        .references
        .iter()
        .filter(|reference| {
            reference.kind == ReferenceKind::XhtmlLink
                && reference.location.attribute_local_name.as_deref() == Some("href")
                && reference.location.attribute_namespace.is_none()
        })
        .map(|reference| {
            (
                (
                    reference.source_path.as_str(),
                    reference.location.element_ordinals.as_slice(),
                ),
                reference,
            )
        })
        .collect();
    let anchors = anchor_index(source_anchors);
    let duplicates: HashSet<_> = source_diagnostics
        .iter()
        .filter_map(|diagnostic| match diagnostic {
            SourceDiagnostic::DuplicateXhtmlId { canonical_path, id } => {
                Some((canonical_path.as_str(), id.as_str()))
            }
            _ => None,
        })
        .collect();
    let mut seen_links = HashSet::new();
    for block in source_blocks {
        for attribute in &block.attribute_text {
            if attribute.value.trim().is_empty() {
                continue;
            }
            space.slots::<RelatedText>(1)?;
            space.slots::<ClassificationEvidence>(1)?;
            space.charge(
                block
                    .canonical_path
                    .len()
                    .saturating_add(attribute.attribute.len())
                    .saturating_add(attribute.value.len()),
            )?;
            space.element_locator(&attribute.locator)?;
            let location = SourceLocation::attribute(
                &block.canonical_path,
                attribute.locator.clone(),
                &attribute.attribute,
            )
            .map_err(|_| ScopeError::InvalidSourceLocation)?;
            let owner = image_owner(block, &attribute.locator, &owners);
            let related_group = placements
                .get(&block.id)
                .and_then(enclosing_group)
                .and_then(|element| {
                    groups.get(&(
                        block.canonical_path.as_str(),
                        element.element_ordinals.as_slice(),
                    ))
                });
            let relations = if let Some(key) = owner {
                let location = key.location(&observed);
                space.slots::<Relation>(1)?;
                space.location(location)?;
                vec![Relation::Resolved(location.clone())]
            } else {
                related_to(related_group, "Image text owner is unresolved", space)?
            };
            space.location(&location)?;
            observed.related.push(RelatedText {
                location: location.clone(),
                kind: RelatedKind::AltText,
                text: attribute.value.clone(),
                relations,
                evidence: vec![ClassificationEvidence {
                    location,
                    kind: EvidenceKind::Location,
                }],
            });
        }

        for metadata in &block.inline_metadata {
            let (href, explicit) = match &metadata.kind {
                InlineMetadataKind::NoteReference { href } => (href, true),
                InlineMetadataKind::Link { href } => (href, false),
                InlineMetadataKind::Formatting { .. } => continue,
            };
            let path = absolute_path(
                &block.canonical_path,
                &metadata.locator,
                &structural_paths,
                space,
            )?
            .ok_or(ScopeError::InvalidSourceLocation)?;
            let Some(from) = metadata
                .ranges
                .iter()
                .find_map(|range| range_owner(&owners, block, range.start))
            else {
                continue;
            };
            let reference = links
                .get(&(block.canonical_path.as_str(), path.as_slice()))
                .copied();
            let Some(target) = resolve_note(
                reference,
                explicit,
                &anchors,
                &duplicates,
                &notes,
                &structural_paths,
                space,
            )?
            else {
                continue;
            };
            space.charge(block.canonical_path.len())?;
            let location = absolute_location(&block.canonical_path, path)?;
            space.slots::<SourceLocation>(1)?;
            space.location(&location)?;
            if !seen_links.insert(location.clone()) {
                continue;
            }
            let from = from.location(&observed);
            space.slots::<NoteReference>(1)?;
            space.slots::<ClassificationEvidence>(1)?;
            space.location(&location)?;
            space.location(from)?;
            space.charge(href.as_ref().map_or(0, String::len).saturating_mul(2))?;
            let from = from.clone();
            observed.note_references.push(NoteReference {
                location: location.clone(),
                from,
                raw_href: href.clone(),
                target,
                evidence: vec![ClassificationEvidence {
                    location,
                    kind: EvidenceKind::Link {
                        href: href.clone(),
                        note_reference: explicit,
                    },
                }],
            });
        }
    }
    for label in navigation_labels
        .iter()
        .filter(|label| !label.text.is_empty())
    {
        let item = navigation
            .iter()
            .find(|tree| tree.resource_path == label.canonical_path)
            .and_then(|tree| {
                let mut items = tree.items.as_slice();
                let mut result = None;
                for &ordinal in &label.item_ordinals {
                    let item = items.get(ordinal as usize)?;
                    result = Some(item);
                    items = &item.children;
                }
                result
            })
            .ok_or(ScopeError::InvalidSourceLocation)?;
        space.slots::<RelatedText>(1)?;
        space.slots::<Relation>(1)?;
        space.slots::<ClassificationEvidence>(1)?;
        space.charge(
            label
                .canonical_path
                .len()
                .saturating_add(label.text.len())
                .saturating_add(label.raw_href.as_ref().map_or(0, String::len))
                .saturating_add(
                    label
                        .element_ordinals
                        .len()
                        .saturating_mul(std::mem::size_of::<u32>()),
                ),
        )?;
        let location = absolute_location(&label.canonical_path, label.element_ordinals.clone())?;
        space.location(&location)?;
        let relation = if let Some(NavigationTarget::Resolved {
            canonical_path,
            fragment,
        }) = &item.target
        {
            if let Some(fragment) = fragment {
                if let Some(found) = anchors.get(&(canonical_path.as_str(), fragment.as_str()))
                    && let [anchor] = found.as_slice()
                    && anchor.is_unique
                    && !duplicates.contains(&(canonical_path.as_str(), fragment.as_str()))
                    && let Some(block) = blocks.get(&anchor.block_id)
                    && let Some(owner) = range_owner(&owners, block, anchor.unit_offset)
                {
                    let target = owner.location(&observed);
                    space.location(target)?;
                    Relation::Resolved(target.clone())
                } else {
                    missing_relation("Navigation fragment has no unique readable owner", space)?
                }
            } else if let Some(owner) = source_blocks
                .iter()
                .filter(|block| &block.canonical_path == canonical_path)
                .find_map(|block| owners.get(&block.id).and_then(|values| values.first()))
            {
                let target = owner.key.location(&observed);
                space.location(target)?;
                Relation::Resolved(target.clone())
            } else {
                missing_relation("Navigation document has no readable owner", space)?
            }
        } else {
            missing_relation("Navigation target is unresolved", space)?
        };
        observed.related.push(RelatedText {
            location: location.clone(),
            kind: RelatedKind::NavigationLabel,
            text: label.text.clone(),
            relations: vec![relation],
            evidence: vec![ClassificationEvidence {
                location,
                kind: EvidenceKind::Link {
                    href: label.raw_href.clone(),
                    note_reference: false,
                },
            }],
        });
    }
    Ok(observed)
}

fn only_range(location: &SourceLocation) -> Result<&LocatedRange, ScopeError> {
    if let SourceLocationKind::TextRanges(ranges) = location.kind()
        && let [range] = ranges.as_slice()
    {
        return Ok(range);
    }
    Err(ScopeError::InvalidSourceLocation)
}

fn register_owner(
    owners: &mut Owners,
    location: &SourceLocation,
    key: OwnerKey,
    space: &mut ScopeWorking,
) -> Result<(), ScopeError> {
    let SourceLocationKind::TextRanges(ranges) = location.kind() else {
        return Err(ScopeError::InvalidSourceLocation);
    };
    space.slots::<(SourceBlockId, Vec<Owner>)>(ranges.len())?;
    space.slots::<Owner>(ranges.len())?;
    for range in ranges {
        let range = range.range();
        owners
            .entry(range.block_id())
            .or_default()
            .push(Owner { range, key });
    }
    Ok(())
}

fn range_owner(owners: &Owners, block: &ScopeBlock, offset: u32) -> Option<OwnerKey> {
    owners
        .get(&block.id)?
        .iter()
        .find(|owner| {
            let range = owner.range;
            range.start() <= offset
                && (offset < range.end() || offset == range.end() && offset == block.unit_count)
        })
        .map(|owner| owner.key)
}

fn image_owner(block: &ScopeBlock, locator: &ElementLocator, owners: &Owners) -> Option<OwnerKey> {
    let (offset, _) = block
        .images
        .iter()
        .find(|(_, path)| *path == locator.element_ordinals)?;
    range_owner(owners, block, *offset)
}

fn enclosing_group(placement: &CapturedBlockPlacement) -> Option<&CapturedScopeElement> {
    placement
        .context
        .iter()
        .find(|element| element.figure_or_table)
}

fn missing_relation(description: &str, space: &mut ScopeWorking) -> Result<Relation, ScopeError> {
    space.charge(description.len())?;
    Ok(Relation::Missing {
        description: description.to_owned(),
    })
}

fn related_to(
    owners: Option<&Vec<SourceLocation>>,
    description: &str,
    space: &mut ScopeWorking,
) -> Result<Vec<Relation>, ScopeError> {
    space.slots::<Relation>(1)?;
    let relation = match owners.map(Vec::as_slice).unwrap_or_default() {
        [] => missing_relation(description, space)?,
        [owner] => {
            space.location(owner)?;
            Relation::Resolved(owner.clone())
        }
        candidates => {
            space.charge(description.len())?;
            space.slots::<SourceLocation>(candidates.len())?;
            for owner in candidates {
                space.location(owner)?;
            }
            Relation::Ambiguous {
                description: description.to_owned(),
                candidates: candidates.to_vec(),
            }
        }
    };
    Ok(vec![relation])
}

fn is_note(element: &CapturedScopeElement) -> bool {
    let contains = |value: &Option<String>, names: &[&str]| {
        value
            .as_deref()
            .unwrap_or_default()
            .split_ascii_whitespace()
            .any(|name| names.contains(&name))
    };
    contains(
        &element.epub_type,
        &["note", "footnote", "endnote", "rearnote"],
    ) || contains(&element.role, &["doc-footnote", "doc-endnote"])
}

fn apply_authored_classification(
    observation: &mut ContentObservation,
    path: &str,
    placement: &CapturedBlockPlacement,
    space: &mut ScopeWorking,
) -> Result<(), ScopeError> {
    for element in &placement.context {
        let Some(epub_type) = &element.epub_type else {
            continue;
        };
        let mut kinds = Vec::new();
        for token in epub_type.split_ascii_whitespace() {
            let kind = match token {
                "prologue" => ContentKind::Opening,
                "chapter" => ContentKind::Chapter,
                "epilogue" => ContentKind::Ending,
                "afterword" => ContentKind::Afterword,
                "copyright-page" | "colophon" => ContentKind::Copyright,
                "advertisement" => ContentKind::Advertisement,
                "titlepage" | "toc" | "landmarks" | "page-list" => ContentKind::OtherAuxiliary,
                _ => continue,
            };
            if !kinds.contains(&kind) {
                space.slots::<ContentKind>(1)?;
                kinds.push(kind);
            }
        }
        if kinds.is_empty() {
            continue;
        }
        space.charge(
            path.len()
                .saturating_add(
                    element
                        .element_ordinals
                        .len()
                        .saturating_mul(std::mem::size_of::<u32>()),
                )
                .saturating_add(epub_type.len())
                .saturating_add("epub:type=".len()),
        )?;
        space.location(&observation.location)?;
        space.slots::<ClassificationEvidence>(observation.evidence.len().saturating_add(1))?;
        let location = absolute_location(path, element.element_ordinals.clone())?;
        let mut evidence = std::mem::take(&mut observation.evidence);
        evidence.push(ClassificationEvidence {
            location,
            kind: EvidenceKind::Other {
                description: format!("epub:type={epub_type}"),
            },
        });
        *observation = ContentObservation::new(
            observation.location.clone(),
            std::mem::take(&mut observation.excerpt),
            kinds,
            evidence,
        )
        .map_err(|_| ScopeError::InvalidSourceLocation)?;
        break;
    }
    Ok(())
}

fn merge_note(items: Vec<ContentInput>, space: &mut ScopeWorking) -> Result<NoteInput, ScopeError> {
    // One canonical range list, plus the temporary vector used while making it.
    space.slots::<LocatedRange>(items.len().saturating_mul(2))?;
    space.slots::<ContentKind>(1)?;
    let mut ranges = Vec::new();
    let mut excerpt = String::new();
    let mut evidence = Vec::new();
    for item in items {
        space.location(&item.observation.location)?;
        ranges.push(only_range(&item.observation.location)?.clone());
        if !excerpt.is_empty() {
            space.append(&mut excerpt, "\n")?;
        }
        space.append(&mut excerpt, &item.observation.excerpt)?;
        space.slots::<ClassificationEvidence>(item.observation.evidence.len())?;
        evidence.extend(item.observation.evidence);
    }
    let location = SourceLocation::text(ranges).map_err(|_| ScopeError::InvalidSourceLocation)?;
    let observation = ContentObservation::new(
        location,
        excerpt,
        vec![ContentKind::OtherAuxiliary],
        evidence,
    )
    .map_err(|_| ScopeError::InvalidSourceLocation)?;
    Ok(NoteInput {
        content: ContentInput {
            observation,
            history: Vec::new(),
        },
    })
}

fn anchor_index(source: &[SourceAnchor]) -> BTreeMap<(&str, &str), Vec<&SourceAnchor>> {
    let mut anchors = BTreeMap::<_, Vec<_>>::new();
    for anchor in source {
        anchors
            .entry((anchor.canonical_path.as_str(), anchor.fragment.as_str()))
            .or_default()
            .push(anchor);
    }
    anchors
}

fn resolve_note(
    link: Option<&ResourceReference>,
    explicit: bool,
    anchors: &BTreeMap<(&str, &str), Vec<&SourceAnchor>>,
    duplicates: &HashSet<(&str, &str)>,
    notes: &BTreeMap<ElementKey<'_>, &SourceLocation>,
    placements: &PlacementIndex<'_>,
    space: &mut ScopeWorking,
) -> Result<Option<NoteTarget>, ScopeError> {
    if let Some(ResourceReference {
        target:
            ReferenceTarget::Internal {
                path,
                fragment: Some(fragment),
            },
        ..
    }) = link
    {
        if duplicates.contains(&(path, fragment)) && explicit {
            let description = "The note fragment has more than one source element";
            space.charge(description.len())?;
            return Ok(Some(NoteTarget::Ambiguous {
                description: description.to_owned(),
                candidates: Vec::new(),
            }));
        }
        if let Some(anchors) = anchors.get(&(path, fragment))
            && let [anchor] = anchors.as_slice()
            && anchor.is_unique
            && let Some(element) = absolute_path(path, &anchor.locator, placements, space)?
        {
            for depth in (0..=element.len()).rev() {
                if let Some(&location) = notes.get(&(path, &element[..depth])) {
                    space.location(location)?;
                    return Ok(Some(NoteTarget::Resolved(location.clone())));
                }
            }
        }
    }
    if explicit {
        let description = "The link could not be tied to a readable note";
        space.charge(description.len())?;
        return Ok(Some(NoteTarget::Missing {
            description: description.to_owned(),
        }));
    }
    Ok(None)
}

fn absolute_path(
    path: &str,
    locator: &ElementLocator,
    placements: &PlacementIndex<'_>,
    space: &mut ScopeWorking,
) -> Result<Option<Vec<u32>>, ScopeError> {
    let root = match &locator.block {
        BlockLocator::NodePath {
            element_ordinals, ..
        } => element_ordinals.as_slice(),
        locator => {
            let Some(&ordinals) = placements.get(&(path, locator)) else {
                return Ok(None);
            };
            ordinals
        }
    };
    space.slots::<u32>(root.len().saturating_add(locator.element_ordinals.len()))?;
    let mut element = root.to_vec();
    element.extend(&locator.element_ordinals);
    Ok(Some(element))
}

fn absolute_location(path: &str, ordinals: Vec<u32>) -> Result<SourceLocation, ScopeError> {
    SourceLocation::element(
        path,
        ElementLocator {
            block: BlockLocator::NodePath {
                element_ordinals: Vec::new(),
                order: 0,
            },
            element_ordinals: ordinals,
        },
    )
    .map_err(|_| ScopeError::InvalidSourceLocation)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(n: u8) -> ScopeBlock {
        ScopeBlock {
            id: SourceBlockId::from_bytes([n; 32]),
            canonical_path: "OPS/chapter.xhtml".into(),
            element_name: "p".into(),
            locator: BlockLocator::NodePath {
                element_ordinals: vec![u32::from(n)],
                order: 0,
            },
            unit_count: 3,
            inline_metadata: Vec::new(),
            attribute_text: Vec::new(),
            images: Vec::new(),
        }
    }

    fn located(block: &ScopeBlock) -> LocatedRange {
        LocatedRange::from_saved(
            block.canonical_path.clone(),
            SourceRange::new(block.id, 0, 3, 3).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn distinct_ranges_look_up_the_same_note_location() {
        let first = block(1);
        let second = block(2);
        let location = SourceLocation::text(vec![located(&first), located(&second)]).unwrap();
        let observed = ObservedScope {
            notes: vec![NoteInput {
                content: ContentInput {
                    observation: ContentObservation::new(
                        location,
                        "one\ntwo".into(),
                        vec![ContentKind::OtherAuxiliary],
                        Vec::new(),
                    )
                    .unwrap(),
                    history: Vec::new(),
                },
            }],
            ..ObservedScope::default()
        };
        let location = &observed.notes[0].content.observation.location;
        let mut working = ScopeWorking::new(4096);
        let mut owners = Owners::new();
        register_owner(&mut owners, location, OwnerKey::Note(0), &mut working).unwrap();
        for block in [&first, &second] {
            assert_eq!(owners[&block.id].len(), 1);
            let found = range_owner(&owners, block, 2).unwrap().location(&observed);
            assert!(std::ptr::eq(found, location));
        }

        let mut denied = Owners::new();
        let mut limited = ScopeWorking::new(working.used() - 1);
        assert_eq!(
            register_owner(&mut denied, location, OwnerKey::Note(0), &mut limited),
            Err(ScopeError::ResourceLimit),
        );
        assert!(denied.is_empty());
    }

    #[test]
    fn ambiguous_caption_keeps_both_owners_with_a_shared_allowance() {
        let locations = vec![
            SourceLocation::text(vec![located(&block(1))]).unwrap(),
            SourceLocation::text(vec![located(&block(2))]).unwrap(),
        ];
        let mut space = ScopeWorking::new(4096);
        let relations = related_to(Some(&locations), "figure", &mut space).unwrap();
        assert_eq!(
            relations,
            [Relation::Ambiguous {
                description: "figure".into(),
                candidates: locations.clone(),
            }],
        );
        assert_eq!(
            related_to(
                Some(&locations),
                "figure",
                &mut ScopeWorking::new(space.used() - 1)
            ),
            Err(ScopeError::ResourceLimit),
        );
    }
}
