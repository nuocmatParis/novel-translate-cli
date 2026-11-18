use super::super::CodecError;
use super::super::records::{Budget, Digest, ElementLocator, safe_path};
use nt_application::volume_import::{source_range as source, source_scope as domain};
use serde::{Deserialize, Serialize};

/// Backed by the immutable source belonging to the exact decision key. In
/// particular, returning range.end as a guessed unit count is not permitted.
pub trait RangeLookup {
    fn unit_count(
        &mut self,
        context: &domain::DecisionContext,
        path: &str,
        block: Digest,
    ) -> Result<u32, CodecError>;
}
impl<F> RangeLookup for F
where
    F: FnMut(&domain::DecisionContext, &str, Digest) -> Result<u32, CodecError>,
{
    fn unit_count(
        &mut self,
        context: &domain::DecisionContext,
        path: &str,
        block: Digest,
    ) -> Result<u32, CodecError> {
        self(context, path, block)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocatedRange {
    pub canonical_path: String,
    pub block_id: Digest,
    pub start: u32,
    pub end: u32,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum Location {
    TextRanges {
        ranges: Vec<LocatedRange>,
    },
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
impl Location {
    pub(super) fn capture(
        value: &domain::SourceLocation,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        Ok(match value.kind() {
            domain::SourceLocationKind::TextRanges(values) => {
                let mut ranges = budget.vector(values.len())?;
                for v in values {
                    let r = v.range();
                    ranges.push(LocatedRange {
                        canonical_path: budget.text(v.canonical_path())?,
                        block_id: Digest(*r.block_id().as_bytes()),
                        start: r.start(),
                        end: r.end(),
                    });
                }
                Self::TextRanges { ranges }
            }
            domain::SourceLocationKind::Element {
                canonical_path,
                locator,
            } => Self::Element {
                canonical_path: budget.text(canonical_path)?,
                locator: ElementLocator::capture(locator, budget)?,
            },
            domain::SourceLocationKind::Attribute {
                canonical_path,
                locator,
                attribute,
            } => Self::Attribute {
                canonical_path: budget.text(canonical_path)?,
                locator: ElementLocator::capture(locator, budget)?,
                attribute: budget.text(attribute)?,
            },
        })
    }
    pub(super) fn restore(
        self,
        context: &domain::DecisionContext,
        lookup: &mut impl RangeLookup,
    ) -> Result<domain::SourceLocation, CodecError> {
        fn element(locator: ElementLocator) -> Result<source::ElementLocator, CodecError> {
            if locator.element_ordinals.len() > 256 {
                return Err(CodecError::InvalidRecord);
            }
            Ok(source::ElementLocator {
                block: locator.block.source_locator()?,
                element_ordinals: locator.element_ordinals,
            })
        }
        match self {
            Self::TextRanges { ranges } => {
                let mut result = Vec::with_capacity(ranges.len());
                let mut previous: Option<(Digest, String, u32)> = None;
                for r in ranges {
                    if !safe_path(&r.canonical_path)
                        || previous.as_ref().is_some_and(|(id, path, end)| {
                            *id > r.block_id
                                || (*id == r.block_id
                                    && (path != &r.canonical_path || *end >= r.start))
                        })
                    {
                        return Err(CodecError::InvalidRecord);
                    }
                    let count = lookup.unit_count(context, &r.canonical_path, r.block_id)?;
                    let range = source::SourceRange::new(
                        source::SourceBlockId::from_bytes(r.block_id.0),
                        r.start as usize,
                        r.end as usize,
                        count as usize,
                    )
                    .map_err(|_| CodecError::InvalidRecord)?;
                    previous = Some((r.block_id, r.canonical_path.clone(), r.end));
                    result.push(
                        domain::LocatedRange::from_saved(r.canonical_path, range)
                            .map_err(|_| CodecError::InvalidRecord)?,
                    );
                }
                domain::SourceLocation::text(result).map_err(|_| CodecError::InvalidRecord)
            }
            Self::Element {
                canonical_path,
                locator,
            } => {
                if !safe_path(&canonical_path) {
                    return Err(CodecError::InvalidRecord);
                }
                domain::SourceLocation::element(canonical_path, element(locator)?)
                    .map_err(|_| CodecError::InvalidRecord)
            }
            Self::Attribute {
                canonical_path,
                locator,
                attribute,
            } => {
                if !safe_path(&canonical_path) {
                    return Err(CodecError::InvalidRecord);
                }
                domain::SourceLocation::attribute(canonical_path, element(locator)?, attribute)
                    .map_err(|_| CodecError::InvalidRecord)
            }
        }
    }
}

// Retained allocation accounting.
crate::import_codec::memory::heap_struct!(LocatedRange {
    canonical_path,
    block_id,
    start,
    end
});
crate::import_codec::memory::heap_enum!(Location {TextRanges {ranges}, Element {canonical_path, locator}, Attribute {canonical_path, locator, attribute}});
