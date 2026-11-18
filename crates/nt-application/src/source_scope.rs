//! Pure orchestration for content scope. Persistence and EPUB resolution are separate.

use std::collections::{HashMap, VecDeque};

use nt_domain::chapter_map::{
    BlockUnitRange, BoundaryEvidence, ChapterMap, ChapterMapError, ChapterMapInput,
    MainFlowContent, SourceDocument, validate_coverage,
};
use nt_domain::source_range::{
    InlineMetadataKind, ProtectedSourceUnit, SourceBlock, SourceUnitKind,
};
use nt_domain::source_scope::{
    ClassificationEvidence, ConfirmedDecision, ContentKind, ContentObservation, DecisionContext,
    DecisionMatch, EvidenceKind, Handling, LocatedRange, ScopeMode, SourceLocation,
    evaluate_observation,
};

mod working;
pub use working::ScopeWorking;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentInput {
    pub observation: ContentObservation,
    /// Comparison candidates are explicitly supplied by the history owner, oldest first.
    /// They are never attached to this location merely by matching a title.
    pub history: Vec<ConfirmedDecision>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoteInput {
    /// There is exactly one note input per resolved source identity.
    pub content: ContentInput,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelatedKind {
    Heading,
    NavigationLabel,
    Caption,
    AltText,
    Ruby,
    OtherLabel,
}

/// An unresolved relation is retained as evidence and is never guessed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Relation {
    Resolved(SourceLocation),
    Missing {
        description: String,
    },
    Ambiguous {
        description: String,
        candidates: Vec<SourceLocation>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelatedText {
    pub location: SourceLocation,
    pub kind: RelatedKind,
    pub text: String,
    pub relations: Vec<Relation>,
    pub evidence: Vec<ClassificationEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NoteTarget {
    Resolved(SourceLocation),
    Missing {
        description: String,
    },
    Ambiguous {
        description: String,
        candidates: Vec<SourceLocation>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoteReference {
    /// Exact source element of this reference, distinct even within one content range.
    pub location: SourceLocation,
    /// Content containing the reference; its scope determines whether the note is needed.
    pub from: SourceLocation,
    pub raw_href: Option<String>,
    pub target: NoteTarget,
    pub evidence: Vec<ClassificationEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeInput {
    pub context: DecisionContext,
    pub mode: ScopeMode,
    pub content: Vec<ContentInput>,
    pub notes: Vec<NoteInput>,
    pub related: Vec<RelatedText>,
    pub note_references: Vec<NoteReference>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluatedContent {
    pub input: ContentInput,
    pub decision: DecisionMatch,
    pub handling: Handling,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluatedNote {
    pub content: EvaluatedContent,
    /// All distinct, resolved incoming reference positions in source input order.
    pub references: Vec<NoteReference>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluatedRelated {
    pub input: RelatedText,
    pub handling: Handling,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeEvaluation {
    pub context: DecisionContext,
    pub mode: ScopeMode,
    pub content: Vec<EvaluatedContent>,
    pub notes: Vec<EvaluatedNote>,
    pub related: Vec<EvaluatedRelated>,
    /// Includes missing and ambiguous targets, without assigning them to a note.
    pub note_references: Vec<NoteReference>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeError {
    DuplicateSourceLocation,
    ConflictingReference,
    UnknownReferenceOwner,
    UnknownResolvedNote,
    UnknownRelatedTarget,
    InvalidMap(ChapterMapError),
    InvalidSourceLocation,
    ResourceLimit,
}

fn evaluate_content(
    context: &DecisionContext,
    mode: ScopeMode,
    input: ContentInput,
) -> EvaluatedContent {
    let (decision, handling) =
        evaluate_observation(mode, context, &input.observation, &input.history);
    EvaluatedContent {
        input,
        decision,
        handling,
    }
}

// This ordering is the union of possible requirements: a definite need to
// translate wins; otherwise unresolved relationships and classifications remain visible.
fn combine(left: Handling, right: Handling) -> Handling {
    use Handling::{NeedsClassification, NeedsRelationship, PreserveSource, TranslateAndReview};
    match (left, right) {
        (TranslateAndReview, _) | (_, TranslateAndReview) => TranslateAndReview,
        (NeedsRelationship, _) | (_, NeedsRelationship) => NeedsRelationship,
        (NeedsClassification, _) | (_, NeedsClassification) => NeedsClassification,
        (PreserveSource, PreserveSource) => PreserveSource,
    }
}

/// Evaluate all readable items and their explicit relationships without mutating history.
/// Each unique note is emitted once, with every exact incoming reference retained.
pub fn evaluate_scope(input: ScopeInput) -> Result<ScopeEvaluation, ScopeError> {
    let mut space = ScopeWorking::new(usize::MAX);
    space.input(&input)?;
    evaluate_scope_within(input, &mut space)
}

/// Evaluate within the caller's remainder, after reserving the input it owns.
pub fn evaluate_scope_within(
    input: ScopeInput,
    space: &mut ScopeWorking,
) -> Result<ScopeEvaluation, ScopeError> {
    space.evaluation(&input)?;
    let ScopeInput {
        context,
        mode,
        content,
        notes,
        related,
        note_references,
    } = input;
    let mut content: Vec<_> = content
        .into_iter()
        .map(|item| evaluate_content(&context, mode, item))
        .collect();
    let mut notes: Vec<_> = notes
        .into_iter()
        .map(|item| EvaluatedNote {
            content: evaluate_content(&context, mode, item.content),
            references: Vec::new(),
        })
        .collect();

    let content_len = content.len();
    let note_len = notes.len();
    let related_start = content_len + note_len;
    let node_count = related_start + related.len();
    let mut indices = HashMap::new();
    for (index, location) in content
        .iter()
        .map(|item| &item.input.observation.location)
        .chain(
            notes
                .iter()
                .map(|item| &item.content.input.observation.location),
        )
        .chain(related.iter().map(|item| &item.location))
        .enumerate()
    {
        if indices.insert(location.clone(), index).is_some() {
            return Err(ScopeError::DuplicateSourceLocation);
        }
    }

    // Edges point from an owning item to the item that inherits its treatment.
    let mut outgoing = vec![Vec::<usize>::new(); node_count];
    let mut base = Vec::with_capacity(node_count);
    base.extend(content.iter().map(|item| item.handling));
    base.extend(notes.iter().map(|item| item.content.handling));
    base.extend(related.iter().map(|item| {
        if mode == ScopeMode::AllText {
            Handling::TranslateAndReview
        } else if item.relations.is_empty()
            || item
                .relations
                .iter()
                .any(|relation| !matches!(relation, Relation::Resolved(_)))
        {
            Handling::NeedsRelationship
        } else {
            Handling::PreserveSource
        }
    }));

    for (offset, item) in related.iter().enumerate() {
        let dependent = related_start + offset;
        for relation in &item.relations {
            if let Relation::Resolved(location) = relation {
                let owner = *indices
                    .get(location)
                    .ok_or(ScopeError::UnknownRelatedTarget)?;
                outgoing[owner].push(dependent);
            }
        }
    }

    let mut unique_references = Vec::<NoteReference>::new();
    let mut reference_positions = HashMap::<SourceLocation, usize>::new();
    for reference in note_references {
        let owner = *indices
            .get(&reference.from)
            .ok_or(ScopeError::UnknownReferenceOwner)?;
        if let Some(&index) = reference_positions.get(&reference.location) {
            if unique_references[index] != reference {
                return Err(ScopeError::ConflictingReference);
            }
            continue;
        }
        if let NoteTarget::Resolved(location) = &reference.target {
            let target = *indices
                .get(location)
                .ok_or(ScopeError::UnknownResolvedNote)?;
            if !(content_len..related_start).contains(&target) {
                return Err(ScopeError::UnknownResolvedNote);
            }
            outgoing[owner].push(target);
            notes[target - content_len]
                .references
                .push(reference.clone());
        }
        reference_positions.insert(reference.location.clone(), unique_references.len());
        unique_references.push(reference);
    }

    // A cycle of labels alone has no real content to inherit from. Mark it
    // unresolved; a cycle connected to content or a note is grounded.
    let mut grounded = vec![false; node_count];
    let mut ground_queue = VecDeque::new();
    for index in 0..node_count {
        if index < related_start || base[index] == Handling::NeedsRelationship {
            grounded[index] = true;
            ground_queue.push_back(index);
        }
    }
    while let Some(index) = ground_queue.pop_front() {
        for &dependent in &outgoing[index] {
            if !grounded[dependent] {
                grounded[dependent] = true;
                ground_queue.push_back(dependent);
            }
        }
    }
    if mode == ScopeMode::Story {
        for index in related_start..node_count {
            if !grounded[index] {
                base[index] = Handling::NeedsRelationship;
            }
        }
    }

    let mut handling = base;
    let mut pending: VecDeque<_> = (0..node_count).collect();
    while let Some(index) = pending.pop_front() {
        for &dependent in &outgoing[index] {
            let next = combine(handling[dependent], handling[index]);
            if next != handling[dependent] {
                handling[dependent] = next;
                pending.push_back(dependent);
            }
        }
    }

    for (item, value) in content.iter_mut().zip(&handling[..content_len]) {
        item.handling = *value;
    }
    for (item, value) in notes.iter_mut().zip(&handling[content_len..related_start]) {
        item.content.handling = *value;
    }
    let related = related
        .into_iter()
        .zip(&handling[related_start..])
        .map(|(input, handling)| EvaluatedRelated {
            input,
            handling: *handling,
        })
        .collect();

    Ok(ScopeEvaluation {
        context,
        mode,
        content,
        notes,
        related,
        note_references: unique_references,
    })
}

/// Convert readable ranges of an already validated CP04 map into observations.
/// No filename, heading or map label alone is treated as a confirmed kind.
/// Attribute text, note target resolution and non-map navigation labels require
/// the import composition; they are supplied separately to evaluate_scope.
pub fn observe_map_text(
    input: &ChapterMapInput,
    map: &ChapterMap,
) -> Result<Vec<ContentInput>, ScopeError> {
    observe_map_text_within(input, map, &mut ScopeWorking::new(usize::MAX))
}

pub fn observe_map_text_within(
    input: &ChapterMapInput,
    map: &ChapterMap,
    space: &mut ScopeWorking,
) -> Result<Vec<ContentInput>, ScopeError> {
    for document in &input.documents {
        space.slots::<(
            nt_domain::source_range::SourceBlockId,
            &SourceDocument,
            &SourceBlock,
        )>(document.blocks.len())?;
    }
    space.slots::<(
        nt_domain::source_range::SourceBlockId,
        Vec<&BoundaryEvidence>,
    )>(map.boundary_evidence.len())?;
    validate_coverage(input, map).map_err(ScopeError::InvalidMap)?;
    let mut blocks = HashMap::<_, (&SourceDocument, &SourceBlock)>::new();
    for document in &input.documents {
        for block in &document.blocks {
            blocks.insert(block.id, (document, block));
        }
    }
    let mut boundaries = HashMap::<_, Vec<&BoundaryEvidence>>::new();
    for boundary in &map.boundary_evidence {
        if let Some(block_id) = boundary.block_id {
            boundaries.entry(block_id).or_default().push(boundary);
        }
    }
    let mut observations = Vec::new();
    for ranges in map
        .volume
        .main_flow
        .iter()
        .map(|item| match item {
            MainFlowContent::Chapter(chapter) => &chapter.ranges,
            MainFlowContent::OutOfChapter { ranges, .. }
            | MainFlowContent::Unclassified { ranges, .. } => ranges,
        })
        .chain(map.volume.auxiliary.iter().map(|item| &item.ranges))
    {
        for range in ranges {
            let (document, block) = blocks
                .get(&range.block_id)
                .copied()
                .ok_or(ScopeError::InvalidSourceLocation)?;
            let evidence = boundaries.get(&block.id).map_or(&[][..], Vec::as_slice);
            if let Some(item) = observe_range(document, block, range, evidence, space)? {
                observations.push(item);
            }
        }
    }
    Ok(observations)
}

fn observe_range(
    document: &SourceDocument,
    block: &SourceBlock,
    range: &BlockUnitRange,
    boundaries: &[&BoundaryEvidence],
    space: &mut ScopeWorking,
) -> Result<Option<ContentInput>, ScopeError> {
    let mut excerpt = String::new();
    for unit in &block.units[range.start as usize..range.end as usize] {
        match &unit.kind {
            SourceUnitKind::Grapheme(text) => space.append(&mut excerpt, text)?,
            SourceUnitKind::Protected(ProtectedSourceUnit::Ruby(ruby)) => {
                space.append(&mut excerpt, &ruby.base)?;
                space.append(&mut excerpt, &ruby.annotation)?;
            }
            SourceUnitKind::Protected(_) => {}
        }
    }
    observe_source_text_within(
        ScopeTextSource {
            id: block.id,
            canonical_path: &block.canonical_path,
            element_name: &block.element_name,
            unit_count: block.unit_count(),
            spine_positions: &document.spine_positions,
            inline_metadata: &block.inline_metadata,
        },
        range,
        boundaries,
        excerpt,
        space,
    )
}

/// Metadata from a source block that the caller has verified against its
/// original. This view does not allocate one object per grapheme.
pub struct ScopeTextSource<'a> {
    pub id: nt_domain::source_range::SourceBlockId,
    pub canonical_path: &'a str,
    pub element_name: &'a str,
    pub unit_count: usize,
    pub spine_positions: &'a [usize],
    pub inline_metadata: &'a [nt_domain::source_range::InlineSourceMetadata],
}

/// Build identical classification evidence for eager and compact source readers.
pub fn observe_source_text(
    block: ScopeTextSource<'_>,
    range: &BlockUnitRange,
    boundaries: &[&BoundaryEvidence],
    excerpt: String,
) -> Result<Option<ContentInput>, ScopeError> {
    let mut space = ScopeWorking::new(usize::MAX);
    space.charge(excerpt.capacity())?;
    observe_source_text_within(block, range, boundaries, excerpt, &mut space)
}

/// Materialize all evidence for this range after charging its live owned data.
pub fn observe_source_text_within(
    block: ScopeTextSource<'_>,
    range: &BlockUnitRange,
    boundaries: &[&BoundaryEvidence],
    excerpt: String,
    space: &mut ScopeWorking,
) -> Result<Option<ContentInput>, ScopeError> {
    if block.id != range.block_id || block.canonical_path != range.canonical_path {
        return Err(ScopeError::InvalidSourceLocation);
    }
    let source_range = nt_domain::source_range::SourceRange::new(
        block.id,
        range.start as usize,
        range.end as usize,
        block.unit_count,
    )
    .map_err(|_| ScopeError::InvalidSourceLocation)?;
    if excerpt.is_empty() {
        return Ok(None);
    }
    let evidence_count = block.spine_positions.len().saturating_add(1);
    let location_copy =
        std::mem::size_of::<LocatedRange>().saturating_add(block.canonical_path.len());
    space.slots::<ContentInput>(1)?;
    space.array::<ContentKind>(2)?;
    space.array::<LocatedRange>(2)?;
    space.charge(block.canonical_path.len())?;
    space.charge(evidence_count.saturating_mul(location_copy))?;
    let mut evidence = space.vector::<ClassificationEvidence>(evidence_count)?;
    let located = LocatedRange::from_saved(block.canonical_path.to_owned(), source_range)
        .map_err(|_| ScopeError::InvalidSourceLocation)?;
    let location =
        SourceLocation::text(vec![located]).map_err(|_| ScopeError::InvalidSourceLocation)?;

    evidence.push(ClassificationEvidence {
        location: location.clone(),
        kind: EvidenceKind::Location,
    });
    for &position in block.spine_positions {
        evidence.push(ClassificationEvidence {
            location: location.clone(),
            kind: EvidenceKind::SpineOrder { position },
        });
    }
    if matches!(block.element_name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
        space.next_slot(&mut evidence)?;
        space.charge(location_copy.saturating_add(excerpt.len()))?;
        evidence.push(ClassificationEvidence {
            location: location.clone(),
            kind: EvidenceKind::Heading {
                text: excerpt.clone(),
            },
        });
    }
    for boundary in boundaries {
        if boundary.canonical_path == block.canonical_path
            && boundary.block_id == Some(block.id)
            && boundary
                .unit_offset
                .is_none_or(|offset| range.start <= offset && offset < range.end)
        {
            space.next_slot(&mut evidence)?;
            space.charge(
                location_copy
                    .saturating_add(boundary.label.as_ref().map_or(0, String::len))
                    .saturating_add(boundary.raw_href.as_ref().map_or(0, String::len))
                    .saturating_add(boundary.reason.len()),
            )?;
            evidence.push(ClassificationEvidence {
                location: location.clone(),
                kind: EvidenceKind::MappingBoundary {
                    kind: boundary.kind,
                    source: boundary.source,
                    unit_offset: boundary.unit_offset,
                    label: boundary.label.clone(),
                    raw_href: boundary.raw_href.clone(),
                    reason: boundary.reason.clone(),
                    depth: boundary.depth,
                },
            });
        }
    }
    for metadata in block.inline_metadata {
        let (href, note_reference) = match &metadata.kind {
            InlineMetadataKind::Link { href } => (href, false),
            InlineMetadataKind::NoteReference { href } => (href, true),
            InlineMetadataKind::Formatting { .. } => continue,
        };
        let belongs = metadata.ranges.iter().any(|part| {
            if part.start == part.end {
                range.start <= part.start
                    && (part.start < range.end
                        || (part.start == range.end && range.end as usize == block.unit_count))
            } else {
                part.start < range.end && range.start < part.end
            }
        });
        if belongs {
            space.next_slot(&mut evidence)?;
            space.charge(
                block
                    .canonical_path
                    .len()
                    .saturating_add(href.as_ref().map_or(0, String::len)),
            )?;
            space.element_locator(&metadata.locator)?;
            let link_location =
                SourceLocation::element(block.canonical_path.to_owned(), metadata.locator.clone())
                    .map_err(|_| ScopeError::InvalidSourceLocation)?;
            evidence.push(ClassificationEvidence {
                location: link_location,
                kind: EvidenceKind::Link {
                    href: href.clone(),
                    note_reference,
                },
            });
        }
    }
    let observation = ContentObservation::new(
        location,
        excerpt,
        vec![ContentKind::OtherStory, ContentKind::OtherAuxiliary],
        evidence,
    )
    .map_err(|_| ScopeError::InvalidSourceLocation)?;
    Ok(Some(ContentInput {
        observation,
        history: Vec::new(),
    }))
}

#[test]
fn preserves_each_spine_position_with_a_checked_allowance() {
    let id = nt_domain::source_range::SourceBlockId::from_bytes([1; 32]);
    let path = "OPS/chapter.xhtml";
    let source = || ScopeTextSource {
        id,
        canonical_path: path,
        element_name: "p",
        unit_count: 3,
        spine_positions: &[0, 2, 4],
        inline_metadata: &[],
    };
    let range = BlockUnitRange {
        block_id: id,
        canonical_path: path.into(),
        start: 0,
        end: 3,
    };
    let mut space = ScopeWorking::new(8192);
    let built = observe_source_text_within(source(), &range, &[], "abc".into(), &mut space)
        .unwrap()
        .unwrap();
    assert_eq!(built.observation.excerpt, "abc");
    assert_eq!(
        built
            .observation
            .evidence
            .iter()
            .filter_map(|evidence| match evidence.kind {
                EvidenceKind::SpineOrder { position } => Some(position),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [0, 2, 4],
    );
    assert_eq!(
        observe_source_text_within(
            source(),
            &range,
            &[],
            "abc".into(),
            &mut ScopeWorking::new(space.used() - 1),
        ),
        Err(ScopeError::ResourceLimit),
    );
}
