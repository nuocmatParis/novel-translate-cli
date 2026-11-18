use super::super::CodecError;
use super::super::inventory_records::names;
use super::super::mapping_records::{EvidenceKind as BoundaryKind, NavigationSource};
use super::super::records::{Budget, Digest};
use super::location::{Location, RangeLookup};
use nt_application::volume_import::{SeriesId, VolumeId, source_scope as domain};
use serde::{Deserialize, Serialize};

names!(
    ContentKind,
    domain::ContentKind,
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
    OtherAuxiliary
);
names!(
    Handling,
    domain::Handling,
    TranslateAndReview,
    PreserveSource,
    NeedsClassification,
    NeedsRelationship
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub series_id: String,
    pub volume_id: String,
    pub epub_sha256: Digest,
    pub mapping_version: Digest,
}
impl Context {
    fn capture(c: &domain::DecisionContext, budget: &mut Budget) -> Result<Self, CodecError> {
        Ok(Self {
            series_id: budget.text(c.series_id.as_str())?,
            volume_id: budget.text(c.volume_id.as_str())?,
            epub_sha256: Digest(c.epub_sha256),
            mapping_version: Digest(c.mapping_version),
        })
    }
    fn restore(self) -> Result<domain::DecisionContext, CodecError> {
        Ok(domain::DecisionContext {
            series_id: SeriesId::parse(self.series_id).map_err(|_| CodecError::InvalidRecord)?,
            volume_id: VolumeId::parse(self.volume_id).map_err(|_| CodecError::InvalidRecord)?,
            epub_sha256: self.epub_sha256.0,
            mapping_version: self.mapping_version.0,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum EvidenceKind {
    SpineOrder {
        position: u64,
    },
    Heading {
        text: String,
    },
    MappingBoundary {
        kind: BoundaryKind,
        source: Option<NavigationSource>,
        unit_offset: Option<u32>,
        label: Option<String>,
        raw_href: Option<String>,
        reason: String,
        depth: Option<u64>,
    },
    Link {
        href: Option<String>,
        note_reference: bool,
    },
    Location {},
    Other {
        description: String,
    },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassificationEvidence {
    pub location: Location,
    pub kind: EvidenceKind,
}
impl ClassificationEvidence {
    pub(super) fn capture(
        e: &domain::ClassificationEvidence,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        use domain::EvidenceKind as K;
        let kind = match &e.kind {
            K::SpineOrder { position } => EvidenceKind::SpineOrder {
                position: u64::try_from(*position).map_err(|_| CodecError::ResourceLimit)?,
            },
            K::Heading { text } => EvidenceKind::Heading {
                text: budget.text(text)?,
            },
            K::MappingBoundary {
                kind,
                source,
                unit_offset,
                label,
                raw_href,
                reason,
                depth,
            } => EvidenceKind::MappingBoundary {
                kind: (*kind).into(),
                source: source.map(Into::into),
                unit_offset: *unit_offset,
                label: budget.optional_text(label)?,
                raw_href: budget.optional_text(raw_href)?,
                reason: budget.text(reason)?,
                depth: depth
                    .map(|n| u64::try_from(n).map_err(|_| CodecError::ResourceLimit))
                    .transpose()?,
            },
            K::Link {
                href,
                note_reference,
            } => EvidenceKind::Link {
                href: budget.optional_text(href)?,
                note_reference: *note_reference,
            },
            K::Location => EvidenceKind::Location {},
            K::Other { description } => EvidenceKind::Other {
                description: budget.text(description)?,
            },
        };
        Ok(Self {
            location: Location::capture(&e.location, budget)?,
            kind,
        })
    }
    pub(super) fn all(
        values: &[domain::ClassificationEvidence],
        budget: &mut Budget,
    ) -> Result<Vec<Self>, CodecError> {
        let mut out = budget.vector(values.len())?;
        for v in values {
            out.push(Self::capture(v, budget)?);
        }
        Ok(out)
    }
    pub(super) fn restore(
        self,
        context: &domain::DecisionContext,
        lookup: &mut impl RangeLookup,
    ) -> Result<domain::ClassificationEvidence, CodecError> {
        use domain::EvidenceKind as K;
        let kind = match self.kind {
            EvidenceKind::SpineOrder { position } => K::SpineOrder {
                position: usize::try_from(position).map_err(|_| CodecError::ResourceLimit)?,
            },
            EvidenceKind::Heading { text } => K::Heading { text },
            EvidenceKind::MappingBoundary {
                kind,
                source,
                unit_offset,
                label,
                raw_href,
                reason,
                depth,
            } => K::MappingBoundary {
                kind: kind.into(),
                source: source.map(Into::into),
                unit_offset,
                label,
                raw_href,
                reason,
                depth: depth
                    .map(|n| usize::try_from(n).map_err(|_| CodecError::ResourceLimit))
                    .transpose()?,
            },
            EvidenceKind::Link {
                href,
                note_reference,
            } => K::Link {
                href,
                note_reference,
            },
            EvidenceKind::Location {} => K::Location,
            EvidenceKind::Other { description } => K::Other { description },
        };
        Ok(domain::ClassificationEvidence {
            location: self.location.restore(context, lookup)?,
            kind,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionKey {
    pub context: Context,
    pub location: Location,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmedDecision {
    pub key: DecisionKey,
    pub kind: ContentKind,
    pub evidence: Vec<ClassificationEvidence>,
}
impl ConfirmedDecision {
    fn capture(value: &domain::ConfirmedDecision, budget: &mut Budget) -> Result<Self, CodecError> {
        Ok(Self {
            key: DecisionKey {
                context: Context::capture(&value.key.context, budget)?,
                location: Location::capture(&value.key.location, budget)?,
            },
            kind: value.kind.into(),
            evidence: ClassificationEvidence::all(&value.evidence, budget)?,
        })
    }
    fn restore(
        self,
        lookup: &mut impl RangeLookup,
    ) -> Result<domain::ConfirmedDecision, CodecError> {
        let context = self.key.context.restore()?;
        let location = self.key.location.restore(&context, lookup)?;
        let evidence = self
            .evidence
            .into_iter()
            .map(|e| e.restore(&context, lookup))
            .collect::<Result<_, _>>()?;
        Ok(domain::ConfirmedDecision {
            key: domain::DecisionKey { context, location },
            kind: self.kind.into(),
            evidence,
        })
    }
    pub(super) fn all(
        values: &[domain::ConfirmedDecision],
        budget: &mut Budget,
    ) -> Result<Vec<Self>, CodecError> {
        let mut out = budget.vector(values.len())?;
        for v in values {
            out.push(Self::capture(v, budget)?);
        }
        Ok(out)
    }
    pub(super) fn restore_all(
        values: Vec<Self>,
        lookup: &mut impl RangeLookup,
    ) -> Result<Vec<domain::ConfirmedDecision>, CodecError> {
        values.into_iter().map(|v| v.restore(lookup)).collect()
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum DecisionMatch {
    Reused { decision: Box<ConfirmedDecision> },
    NeedsComparison { candidates: Vec<ConfirmedDecision> },
    Unconfirmed {},
}
impl DecisionMatch {
    pub(super) fn capture(
        value: &domain::DecisionMatch,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        Ok(match value {
            domain::DecisionMatch::Reused(d) => {
                budget.charge(std::mem::size_of::<ConfirmedDecision>(), 1)?;
                Self::Reused {
                    decision: Box::new(ConfirmedDecision::capture(d, budget)?),
                }
            }
            domain::DecisionMatch::NeedsComparison(candidates) => Self::NeedsComparison {
                candidates: ConfirmedDecision::all(candidates, budget)?,
            },
            domain::DecisionMatch::Unconfirmed => Self::Unconfirmed {},
        })
    }
    pub(super) fn restore(
        self,
        lookup: &mut impl RangeLookup,
    ) -> Result<domain::DecisionMatch, CodecError> {
        Ok(match self {
            Self::Reused { decision } => {
                domain::DecisionMatch::Reused(Box::new(decision.restore(lookup)?))
            }
            Self::NeedsComparison { candidates } => domain::DecisionMatch::NeedsComparison(
                ConfirmedDecision::restore_all(candidates, lookup)?,
            ),
            Self::Unconfirmed {} => domain::DecisionMatch::Unconfirmed,
        })
    }
}

// Retained allocation accounting.
crate::import_codec::memory::heap_struct!(Context {
    series_id,
    volume_id,
    epub_sha256,
    mapping_version
});
crate::import_codec::memory::heap_enum!(EvidenceKind {SpineOrder {position}, Heading {text}, MappingBoundary {kind, source, unit_offset, label, raw_href, reason, depth}, Link {href, note_reference}, Location {}, Other {description}});
crate::import_codec::memory::heap_struct!(ClassificationEvidence { location, kind });
crate::import_codec::memory::heap_struct!(DecisionKey { context, location });
crate::import_codec::memory::heap_struct!(ConfirmedDecision {
    key,
    kind,
    evidence
});
crate::import_codec::memory::heap_enum!(DecisionMatch {Reused {decision}, NeedsComparison {candidates}, Unconfirmed {}});
