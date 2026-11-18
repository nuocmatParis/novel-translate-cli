/// Pure scope, classification, source-location and history contracts.
use crate::chapter_map::{BlockUnitRange, BoundaryEvidenceKind, NavigationSource};
use crate::source_range::{ElementLocator, SourceBlock, SourceRange};
use crate::workspace::{SeriesId, VolumeId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeMode {
    AllText,
    Story,
}

/// An in-memory rule for creation; the registration transaction belongs to T12.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeriesScope {
    pub series_id: SeriesId,
    default_mode: ScopeMode,
}

impl SeriesScope {
    pub fn new(series_id: SeriesId) -> Self {
        Self {
            series_id,
            default_mode: ScopeMode::AllText,
        }
    }

    /// Reconstruct a default already read by the storage owner.
    pub fn from_saved(series_id: SeriesId, default_mode: ScopeMode) -> Self {
        Self {
            series_id,
            default_mode,
        }
    }

    pub fn default_mode(&self) -> ScopeMode {
        self.default_mode
    }

    pub fn set_default(&mut self, mode: ScopeMode) {
        self.default_mode = mode;
    }

    /// Captures the current default even if the new volume is not yet imported.
    pub fn capture_for_new_volume(&self, volume_id: VolumeId) -> VolumeScope {
        VolumeScope {
            series_id: self.series_id.clone(),
            volume_id,
            mode: self.default_mode,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeScope {
    pub series_id: SeriesId,
    pub volume_id: VolumeId,
    mode: ScopeMode,
}

impl VolumeScope {
    /// Reconstruct the effective value saved for this volume; do not re-inherit.
    pub fn from_saved(series_id: SeriesId, volume_id: VolumeId, mode: ScopeMode) -> Self {
        Self {
            series_id,
            volume_id,
            mode,
        }
    }

    pub fn mode(&self) -> ScopeMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ScopeMode) {
        self.mode = mode;
    }
}

/// One validated source range together with the path supplied by its source block.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LocatedRange {
    canonical_path: String,
    range: SourceRange,
}

impl LocatedRange {
    pub fn new(block: &SourceBlock, start: usize, end: usize) -> Result<Self, InvalidLocation> {
        if block.canonical_path.is_empty() {
            return Err(InvalidLocation);
        }
        let range = SourceRange::new(block.id, start, end, block.unit_count())
            .map_err(|_| InvalidLocation)?;
        Ok(Self {
            canonical_path: block.canonical_path.clone(),
            range,
        })
    }

    /// Refuse a map range that belongs to a different block or canonical path.
    pub fn from_map(block: &SourceBlock, mapped: &BlockUnitRange) -> Result<Self, InvalidLocation> {
        if block.id != mapped.block_id || block.canonical_path != mapped.canonical_path {
            return Err(InvalidLocation);
        }
        Self::new(block, mapped.start as usize, mapped.end as usize)
    }

    pub fn canonical_path(&self) -> &str {
        &self.canonical_path
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }
}

/// Source identity uses published block ranges or element/attribute locators.
/// Paths for element locations must be the canonical paths from the extractor;
/// callers must resolve ambiguous anchors before constructing a resolved target.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SourceLocation(SourceLocationKind);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SourceLocationKind {
    TextRanges(Vec<LocatedRange>),
    Element {
        canonical_path: String,
        locator: ElementLocator,
    },
    Attribute {
        canonical_path: String,
        locator: ElementLocator,
        attribute: String,
    },
}

impl SourceLocation {
    /// Canonicalize range order and coalesce adjacent pieces of the same block.
    /// Reading order remains the chapter map's responsibility.
    pub fn text(mut ranges: Vec<LocatedRange>) -> Result<Self, InvalidLocation> {
        if ranges.is_empty() {
            return Err(InvalidLocation);
        }
        ranges.sort_unstable_by(|left, right| {
            (
                left.range.block_id(),
                &left.canonical_path,
                left.range.start(),
                left.range.end(),
            )
                .cmp(&(
                    right.range.block_id(),
                    &right.canonical_path,
                    right.range.start(),
                    right.range.end(),
                ))
        });
        let mut canonical: Vec<LocatedRange> = Vec::with_capacity(ranges.len());
        for current in ranges {
            if let Some(previous) = canonical
                .last_mut()
                .filter(|previous| previous.range.block_id() == current.range.block_id())
            {
                if previous.canonical_path != current.canonical_path
                    || previous.range.end() > current.range.start()
                {
                    return Err(InvalidLocation);
                }
                if previous.range.end() == current.range.start() {
                    previous.range = SourceRange::new(
                        previous.range.block_id(),
                        previous.range.start() as usize,
                        current.range.end() as usize,
                        current.range.end() as usize,
                    )
                    .map_err(|_| InvalidLocation)?;
                    continue;
                }
            }
            canonical.push(current);
        }
        Ok(Self(SourceLocationKind::TextRanges(canonical)))
    }

    pub fn kind(&self) -> &SourceLocationKind {
        &self.0
    }

    pub fn element(
        canonical_path: impl Into<String>,
        locator: ElementLocator,
    ) -> Result<Self, InvalidLocation> {
        let canonical_path = canonical_path.into();
        if canonical_path.is_empty() {
            return Err(InvalidLocation);
        }
        Ok(Self(SourceLocationKind::Element {
            canonical_path,
            locator,
        }))
    }

    pub fn attribute(
        canonical_path: impl Into<String>,
        locator: ElementLocator,
        attribute: impl Into<String>,
    ) -> Result<Self, InvalidLocation> {
        let canonical_path = canonical_path.into();
        let attribute = attribute.into();
        if canonical_path.is_empty() || attribute.is_empty() {
            return Err(InvalidLocation);
        }
        Ok(Self(SourceLocationKind::Attribute {
            canonical_path,
            locator,
            attribute,
        }))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidLocation;

impl LocatedRange {
    /// Restore a range whose block membership, canonical path and unit count
    /// have been checked against the owning immutable source snapshot.
    /// SourceRange retains its own checked half-open offset invariant.
    pub fn from_saved(
        canonical_path: impl Into<String>,
        range: SourceRange,
    ) -> Result<Self, InvalidLocation> {
        let canonical_path = canonical_path.into();
        if canonical_path.is_empty() {
            return Err(InvalidLocation);
        }
        Ok(Self {
            canonical_path,
            range,
        })
    }
}

/// Classification is separate from what the selected scope does with the text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentKind {
    Opening,
    Chapter,
    SideStory,
    Ending,
    Afterword,
    OtherStory,
    Copyright,
    AuthorIntroduction,
    Advertisement,
    PromotionalExcerpt,
    OtherAuxiliary,
}

impl ContentKind {
    fn story_handling(self) -> Handling {
        match self {
            Self::Opening
            | Self::Chapter
            | Self::SideStory
            | Self::Ending
            | Self::Afterword
            | Self::OtherStory => Handling::TranslateAndReview,
            Self::Copyright
            | Self::AuthorIntroduction
            | Self::Advertisement
            | Self::PromotionalExcerpt
            | Self::OtherAuxiliary => Handling::PreserveSource,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Handling {
    TranslateAndReview,
    PreserveSource,
    NeedsClassification,
    /// A related item cannot yet be associated with its actual content.
    NeedsRelationship,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceKind {
    SpineOrder {
        position: usize,
    },
    Heading {
        text: String,
    },
    MappingBoundary {
        kind: BoundaryEvidenceKind,
        source: Option<NavigationSource>,
        unit_offset: Option<u32>,
        label: Option<String>,
        raw_href: Option<String>,
        reason: String,
        depth: Option<usize>,
    },
    Link {
        href: Option<String>,
        note_reference: bool,
    },
    Location,
    Other {
        description: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassificationEvidence {
    pub location: SourceLocation,
    pub kind: EvidenceKind,
}

/// Only readable text belongs in this observation; bitmap text uses the image flow.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentObservation {
    pub location: SourceLocation,
    pub excerpt: String,
    possible_kinds: Vec<ContentKind>,
    pub evidence: Vec<ClassificationEvidence>,
}

impl ContentObservation {
    pub fn new(
        location: SourceLocation,
        excerpt: String,
        possible_kinds: Vec<ContentKind>,
        evidence: Vec<ClassificationEvidence>,
    ) -> Result<Self, EmptyClassifications> {
        if possible_kinds.is_empty() {
            return Err(EmptyClassifications);
        }
        Ok(Self {
            location,
            excerpt,
            possible_kinds,
            evidence,
        })
    }

    pub fn possible_kinds(&self) -> &[ContentKind] {
        &self.possible_kinds
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmptyClassifications;

/// Mapping version is an opaque 32-byte digest supplied by the owning mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionContext {
    pub series_id: SeriesId,
    pub volume_id: VolumeId,
    pub epub_sha256: [u8; 32],
    pub mapping_version: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionKey {
    pub context: DecisionContext,
    pub location: SourceLocation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfirmedDecision {
    pub key: DecisionKey,
    pub kind: ContentKind,
    pub evidence: Vec<ClassificationEvidence>,
}

/// History is ordered oldest to newest. An exact decision alone can be reused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionMatch {
    Reused(Box<ConfirmedDecision>),
    NeedsComparison(Vec<ConfirmedDecision>),
    Unconfirmed,
}

fn match_decision(key: &DecisionKey, history: &[ConfirmedDecision]) -> DecisionMatch {
    if let Some(exact) = history.iter().rev().find(|decision| decision.key == *key) {
        DecisionMatch::Reused(Box::new(exact.clone()))
    } else if history.is_empty() {
        DecisionMatch::Unconfirmed
    } else {
        DecisionMatch::NeedsComparison(history.to_vec())
    }
}

fn handling_for(
    mode: ScopeMode,
    observation: &ContentObservation,
    decision: &DecisionMatch,
) -> Handling {
    if mode == ScopeMode::AllText {
        return Handling::TranslateAndReview;
    }
    if let DecisionMatch::Reused(decision) = decision {
        return decision.kind.story_handling();
    }
    let first = observation.possible_kinds[0].story_handling();
    if observation
        .possible_kinds
        .iter()
        .all(|kind| kind.story_handling() == first)
    {
        first
    } else {
        Handling::NeedsClassification
    }
}

/// Apply a confirmed kind only after independently checking its complete key.
pub fn evaluate_observation(
    mode: ScopeMode,
    context: &DecisionContext,
    observation: &ContentObservation,
    history: &[ConfirmedDecision],
) -> (DecisionMatch, Handling) {
    let key = DecisionKey {
        context: context.clone(),
        location: observation.location.clone(),
    };
    let decision = match_decision(&key, history);
    let handling = handling_for(mode, observation, &decision);
    (decision, handling)
}
