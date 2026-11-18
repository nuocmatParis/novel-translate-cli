use super::super::CodecError;
use super::super::inventory_records::names;
use super::super::records::Budget;
use super::evidence::{
    ClassificationEvidence, ConfirmedDecision, ContentKind, DecisionMatch, Handling,
};
use super::location::{Location, RangeLookup};
use nt_application::source_scope as app;
use nt_application::volume_import::source_scope as domain;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub location: Location,
    pub excerpt: String,
    pub possible_kinds: Vec<ContentKind>,
    pub evidence: Vec<ClassificationEvidence>,
}
impl Observation {
    fn capture(v: &domain::ContentObservation, budget: &mut Budget) -> Result<Self, CodecError> {
        let mut possible_kinds = budget.vector(v.possible_kinds().len())?;
        for &kind in v.possible_kinds() {
            possible_kinds.push(kind.into());
        }
        Ok(Self {
            location: Location::capture(&v.location, budget)?,
            excerpt: budget.text(&v.excerpt)?,
            possible_kinds,
            evidence: ClassificationEvidence::all(&v.evidence, budget)?,
        })
    }
    fn restore(
        self,
        context: &domain::DecisionContext,
        lookup: &mut impl RangeLookup,
    ) -> Result<domain::ContentObservation, CodecError> {
        domain::ContentObservation::new(
            self.location.restore(context, lookup)?,
            self.excerpt,
            self.possible_kinds.into_iter().map(Into::into).collect(),
            self.evidence
                .into_iter()
                .map(|e| e.restore(context, lookup))
                .collect::<Result<_, _>>()?,
        )
        .map_err(|_| CodecError::InvalidRecord)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentInput {
    pub observation: Observation,
    pub history: Vec<ConfirmedDecision>,
}
impl ContentInput {
    fn capture(value: &app::ContentInput, budget: &mut Budget) -> Result<Self, CodecError> {
        Ok(Self {
            observation: Observation::capture(&value.observation, budget)?,
            history: ConfirmedDecision::all(&value.history, budget)?,
        })
    }
    fn restore(
        self,
        context: &domain::DecisionContext,
        lookup: &mut impl RangeLookup,
    ) -> Result<app::ContentInput, CodecError> {
        Ok(app::ContentInput {
            observation: self.observation.restore(context, lookup)?,
            history: ConfirmedDecision::restore_all(self.history, lookup)?,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatedContent {
    pub input: ContentInput,
    pub decision: DecisionMatch,
    pub handling: Handling,
}
impl EvaluatedContent {
    pub(super) fn capture(
        value: &app::EvaluatedContent,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        Ok(Self {
            input: ContentInput::capture(&value.input, budget)?,
            decision: DecisionMatch::capture(&value.decision, budget)?,
            handling: value.handling.into(),
        })
    }
    pub(super) fn restore(
        self,
        context: &domain::DecisionContext,
        lookup: &mut impl RangeLookup,
    ) -> Result<app::EvaluatedContent, CodecError> {
        Ok(app::EvaluatedContent {
            input: self.input.restore(context, lookup)?,
            decision: self.decision.restore(lookup)?,
            handling: self.handling.into(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum Target {
    Resolved {
        location: Location,
    },
    Missing {
        description: String,
    },
    Ambiguous {
        description: String,
        candidates: Vec<Location>,
    },
}
impl Target {
    fn locations(
        values: &[domain::SourceLocation],
        budget: &mut Budget,
    ) -> Result<Vec<Location>, CodecError> {
        let mut out = budget.vector(values.len())?;
        for v in values {
            out.push(Location::capture(v, budget)?);
        }
        Ok(out)
    }
    fn relation(value: &app::Relation, budget: &mut Budget) -> Result<Self, CodecError> {
        Ok(match value {
            app::Relation::Resolved(location) => Self::Resolved {
                location: Location::capture(location, budget)?,
            },
            app::Relation::Missing { description } => Self::Missing {
                description: budget.text(description)?,
            },
            app::Relation::Ambiguous {
                description,
                candidates,
            } => Self::Ambiguous {
                description: budget.text(description)?,
                candidates: Self::locations(candidates, budget)?,
            },
        })
    }
    fn note(value: &app::NoteTarget, budget: &mut Budget) -> Result<Self, CodecError> {
        Ok(match value {
            app::NoteTarget::Resolved(location) => Self::Resolved {
                location: Location::capture(location, budget)?,
            },
            app::NoteTarget::Missing { description } => Self::Missing {
                description: budget.text(description)?,
            },
            app::NoteTarget::Ambiguous {
                description,
                candidates,
            } => Self::Ambiguous {
                description: budget.text(description)?,
                candidates: Self::locations(candidates, budget)?,
            },
        })
    }
    fn restore_relation(
        self,
        context: &domain::DecisionContext,
        lookup: &mut impl RangeLookup,
    ) -> Result<app::Relation, CodecError> {
        Ok(match self {
            Self::Resolved { location } => {
                app::Relation::Resolved(location.restore(context, lookup)?)
            }
            Self::Missing { description } => app::Relation::Missing { description },
            Self::Ambiguous {
                description,
                candidates,
            } => app::Relation::Ambiguous {
                description,
                candidates: candidates
                    .into_iter()
                    .map(|c| c.restore(context, lookup))
                    .collect::<Result<_, _>>()?,
            },
        })
    }
    fn restore_note(
        self,
        context: &domain::DecisionContext,
        lookup: &mut impl RangeLookup,
    ) -> Result<app::NoteTarget, CodecError> {
        Ok(match self {
            Self::Resolved { location } => {
                app::NoteTarget::Resolved(location.restore(context, lookup)?)
            }
            Self::Missing { description } => app::NoteTarget::Missing { description },
            Self::Ambiguous {
                description,
                candidates,
            } => app::NoteTarget::Ambiguous {
                description,
                candidates: candidates
                    .into_iter()
                    .map(|c| c.restore(context, lookup))
                    .collect::<Result<_, _>>()?,
            },
        })
    }
}
names!(
    RelatedKind,
    app::RelatedKind,
    Heading,
    NavigationLabel,
    Caption,
    AltText,
    Ruby,
    OtherLabel
);
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelatedText {
    pub location: Location,
    pub kind: RelatedKind,
    pub text: String,
    pub relations: Vec<Target>,
    pub evidence: Vec<ClassificationEvidence>,
}
impl RelatedText {
    fn capture(value: &app::RelatedText, budget: &mut Budget) -> Result<Self, CodecError> {
        let mut relations = budget.vector(value.relations.len())?;
        for r in &value.relations {
            relations.push(Target::relation(r, budget)?);
        }
        Ok(Self {
            location: Location::capture(&value.location, budget)?,
            kind: value.kind.into(),
            text: budget.text(&value.text)?,
            relations,
            evidence: ClassificationEvidence::all(&value.evidence, budget)?,
        })
    }
    fn restore(
        self,
        context: &domain::DecisionContext,
        lookup: &mut impl RangeLookup,
    ) -> Result<app::RelatedText, CodecError> {
        Ok(app::RelatedText {
            location: self.location.restore(context, lookup)?,
            kind: self.kind.into(),
            text: self.text,
            relations: self
                .relations
                .into_iter()
                .map(|r| r.restore_relation(context, lookup))
                .collect::<Result<_, _>>()?,
            evidence: self
                .evidence
                .into_iter()
                .map(|e| e.restore(context, lookup))
                .collect::<Result<_, _>>()?,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatedRelated {
    pub input: RelatedText,
    pub handling: Handling,
}
impl EvaluatedRelated {
    pub(super) fn capture(
        value: &app::EvaluatedRelated,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        Ok(Self {
            input: RelatedText::capture(&value.input, budget)?,
            handling: value.handling.into(),
        })
    }
    pub(super) fn restore(
        self,
        context: &domain::DecisionContext,
        lookup: &mut impl RangeLookup,
    ) -> Result<app::EvaluatedRelated, CodecError> {
        Ok(app::EvaluatedRelated {
            input: self.input.restore(context, lookup)?,
            handling: self.handling.into(),
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoteReference {
    pub location: Location,
    pub from: Location,
    pub raw_href: Option<String>,
    pub target: Target,
    pub evidence: Vec<ClassificationEvidence>,
}
impl NoteReference {
    pub(super) fn capture(
        value: &app::NoteReference,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        Ok(Self {
            location: Location::capture(&value.location, budget)?,
            from: Location::capture(&value.from, budget)?,
            raw_href: budget.optional_text(&value.raw_href)?,
            target: Target::note(&value.target, budget)?,
            evidence: ClassificationEvidence::all(&value.evidence, budget)?,
        })
    }
    pub(super) fn all(
        values: &[app::NoteReference],
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
    ) -> Result<app::NoteReference, CodecError> {
        Ok(app::NoteReference {
            location: self.location.restore(context, lookup)?,
            from: self.from.restore(context, lookup)?,
            raw_href: self.raw_href,
            target: self.target.restore_note(context, lookup)?,
            evidence: self
                .evidence
                .into_iter()
                .map(|e| e.restore(context, lookup))
                .collect::<Result<_, _>>()?,
        })
    }
}

// Retained allocation accounting.
crate::import_codec::memory::heap_struct!(Observation {
    location,
    excerpt,
    possible_kinds,
    evidence
});
crate::import_codec::memory::heap_struct!(ContentInput {
    observation,
    history
});
crate::import_codec::memory::heap_struct!(EvaluatedContent {
    input,
    decision,
    handling
});
crate::import_codec::memory::heap_enum!(Target {Resolved {location}, Missing {description}, Ambiguous {description, candidates}});
crate::import_codec::memory::heap_struct!(RelatedText {
    location,
    kind,
    text,
    relations,
    evidence
});
crate::import_codec::memory::heap_struct!(EvaluatedRelated { input, handling });
crate::import_codec::memory::heap_struct!(NoteReference {
    location,
    from,
    raw_href,
    target,
    evidence
});
