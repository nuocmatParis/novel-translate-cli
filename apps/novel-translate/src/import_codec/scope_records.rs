//! Actual T08 scope records, including ordered history and unresolved relations.
//! Restoring text ranges requires membership/count from the source of each key;
//! an old comparison range is never checked against an invented current count.
mod content;
mod evidence;
mod location;

use super::CodecError;
use super::inventory_records::names;
use super::records::Budget;
pub(crate) use content::Target as RelationTarget;
pub use content::{EvaluatedContent, NoteReference, RelatedText};
pub use evidence::{ClassificationEvidence, ContentKind, Context, DecisionMatch, Handling};
pub use location::{Location, RangeLookup};
use nt_application::source_scope as app;
use nt_application::volume_import::source_scope as domain;
use serde::{Deserialize, Serialize};

names!(Mode, domain::ScopeMode, AllText, Story);
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum ScopeRecord {
    Content(EvaluatedContent),
    Related(content::EvaluatedRelated),
    NoteReference(NoteReference),
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Note {
    pub content: EvaluatedContent,
    pub references: Vec<NoteReference>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredScope {
    pub effective_mode: Mode,
    pub records: Vec<ScopeRecord>,
    pub notes: Vec<Note>,
}
impl StoredScope {
    pub fn mode(&self) -> domain::ScopeMode {
        self.effective_mode.into()
    }

    pub fn capture(
        value: &app::ScopeEvaluation,
        reserved_working_bytes: usize,
    ) -> Result<Self, CodecError> {
        Self::capture_with_budget(value, &mut Budget::new(reserved_working_bytes)?)
    }
    pub(super) fn capture_with_budget(
        value: &app::ScopeEvaluation,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        let count = value
            .content
            .len()
            .checked_add(value.related.len())
            .and_then(|n| n.checked_add(value.note_references.len()))
            .ok_or(CodecError::ResourceLimit)?;
        let mut records = budget.vector(count)?;
        for c in &value.content {
            records.push(ScopeRecord::Content(EvaluatedContent::capture(c, budget)?));
        }
        for r in &value.related {
            records.push(ScopeRecord::Related(content::EvaluatedRelated::capture(
                r, budget,
            )?));
        }
        for r in &value.note_references {
            records.push(ScopeRecord::NoteReference(NoteReference::capture(
                r, budget,
            )?));
        }
        let mut notes = budget.vector(value.notes.len())?;
        for n in &value.notes {
            notes.push(Note {
                content: EvaluatedContent::capture(&n.content, budget)?,
                references: NoteReference::all(&n.references, budget)?,
            });
        }
        Ok(Self {
            effective_mode: value.mode.into(),
            records,
            notes,
        })
    }
    /// The context comes from the same typed DB volume version and source
    /// header. The lookup must bind each current/historical key to its source.
    pub fn into_evaluation(
        self,
        context: domain::DecisionContext,
        lookup: &mut impl RangeLookup,
    ) -> Result<app::ScopeEvaluation, CodecError> {
        let reserved = super::memory::working(&self, 0)?
            .checked_mul(8)
            .ok_or(CodecError::ResourceLimit)?;
        let remaining = Budget::new(reserved)?.remaining_bytes();
        let result = self.restore(context, lookup)?;
        Self::verify_evaluation(&result, remaining)?;
        Ok(result)
    }

    pub(crate) fn restore(
        self,
        context: domain::DecisionContext,
        lookup: &mut impl RangeLookup,
    ) -> Result<app::ScopeEvaluation, CodecError> {
        let mut content = Vec::new();
        let mut related = Vec::new();
        let mut note_references = Vec::new();
        let mut group = 0u8;
        for record in self.records {
            match record {
                ScopeRecord::Content(c) if group == 0 => content.push(c.restore(&context, lookup)?),
                ScopeRecord::Related(r) if group <= 1 => {
                    group = 1;
                    related.push(r.restore(&context, lookup)?);
                }
                ScopeRecord::NoteReference(r) => {
                    group = 2;
                    note_references.push(r.restore(&context, lookup)?);
                }
                _ => return Err(CodecError::InvalidRecord),
            }
        }
        let mut notes = Vec::with_capacity(self.notes.len());
        for note in self.notes {
            notes.push(app::EvaluatedNote {
                content: note.content.restore(&context, lookup)?,
                references: note
                    .references
                    .into_iter()
                    .map(|r| r.restore(&context, lookup))
                    .collect::<Result<_, _>>()?,
            });
        }
        Ok(app::ScopeEvaluation {
            context,
            mode: self.effective_mode.into(),
            content,
            notes,
            related,
            note_references,
        })
    }

    pub(crate) fn verify_evaluation(
        result: &app::ScopeEvaluation,
        remaining: usize,
    ) -> Result<(), CodecError> {
        // Recompute the deterministic evaluation from all retained inputs. A
        // forged saved handling/reuse or dropped incoming reference fails.
        let mut working = app::ScopeWorking::new(remaining);
        // The caller reserves the held result. One complete result bounds the
        // subset copied into the check input, including vector capacities.
        working.result(result).map_err(scope_verification_error)?;
        let input = app::ScopeInput {
            context: result.context.clone(),
            mode: result.mode,
            content: result.content.iter().map(|c| c.input.clone()).collect(),
            notes: result
                .notes
                .iter()
                .map(|n| app::NoteInput {
                    content: n.content.input.clone(),
                })
                .collect(),
            related: result.related.iter().map(|r| r.input.clone()).collect(),
            note_references: result.note_references.clone(),
        };
        let checked =
            app::evaluate_scope_within(input, &mut working).map_err(scope_verification_error)?;
        if &checked != result {
            return Err(CodecError::InvalidRecord);
        }
        Ok(())
    }
}

fn scope_verification_error(error: app::ScopeError) -> CodecError {
    match error {
        app::ScopeError::ResourceLimit => CodecError::ResourceLimit,
        _ => CodecError::InvalidRecord,
    }
}

// Retained allocation accounting.
crate::import_codec::memory::heap_enum!(ScopeRecord {Content(value), Related(value), NoteReference(value)});
crate::import_codec::memory::heap_struct!(Note {
    content,
    references
});
crate::import_codec::memory::heap_struct!(StoredScope {
    effective_mode,
    records,
    notes
});
