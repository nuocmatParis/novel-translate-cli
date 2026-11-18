//! Working allowance for materializing classification evidence and relationships.

use std::mem::size_of;

use nt_domain::source_range::{BlockLocator, ElementLocator};
use nt_domain::source_scope::{ContentKind, SourceLocationKind};
use nt_domain::volume_import::MAX_IMPORT_WORKING_BYTES;

use super::{
    ClassificationEvidence, ConfirmedDecision, ContentInput, EvaluatedContent, EvaluatedNote,
    EvaluatedRelated, EvidenceKind, NoteInput, NoteReference, NoteTarget, RelatedText, Relation,
    ScopeError, ScopeEvaluation, ScopeInput, SourceLocation,
};

/// A caller supplies the spare bytes after its other live import data.
/// Reservations cover Rust heap contents; the OS and allocator metadata are separate.
pub struct ScopeWorking {
    limit: usize,
    used: usize,
}

impl ScopeWorking {
    pub fn new(remaining_bytes: usize) -> Self {
        Self {
            limit: remaining_bytes.min(MAX_IMPORT_WORKING_BYTES),
            used: 0,
        }
    }

    pub fn used(&self) -> usize {
        self.used
    }

    pub fn remaining(&self) -> usize {
        self.limit - self.used
    }

    pub fn charge(&mut self, bytes: usize) -> Result<(), ScopeError> {
        self.used = self
            .used
            .checked_add(bytes)
            .filter(|size| *size <= self.limit)
            .ok_or(ScopeError::ResourceLimit)?;
        Ok(())
    }

    /// Allow for minimum capacities, spare slots and overlapping old and new
    /// allocations when a projection's Vec or hash table grows.
    pub fn slots<T>(&mut self, count: usize) -> Result<(), ScopeError> {
        self.charge(count.saturating_mul(8).saturating_mul(size_of::<T>()))
    }

    pub fn array<T>(&mut self, count: usize) -> Result<(), ScopeError> {
        self.charge(count.saturating_mul(size_of::<T>()))
    }

    pub(super) fn vector<T>(&mut self, count: usize) -> Result<Vec<T>, ScopeError> {
        self.array::<T>(count)?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(count)
            .map_err(|_| ScopeError::ResourceLimit)?;
        Ok(output)
    }

    pub(super) fn next_slot<T>(&mut self, output: &mut Vec<T>) -> Result<(), ScopeError> {
        if output.len() == output.capacity() {
            let capacity = output.capacity().saturating_mul(2).max(4);
            self.array::<T>(capacity)?;
            output
                .try_reserve_exact(capacity - output.len())
                .map_err(|_| ScopeError::ResourceLimit)?;
        }
        Ok(())
    }

    pub fn append(&mut self, output: &mut String, text: &str) -> Result<(), ScopeError> {
        let length = output
            .len()
            .checked_add(text.len())
            .ok_or(ScopeError::ResourceLimit)?;
        if length > output.capacity() {
            let capacity = output.capacity().saturating_mul(2).max(length).max(8);
            self.charge(capacity)?;
            output
                .try_reserve_exact(capacity - output.len())
                .map_err(|_| ScopeError::ResourceLimit)?;
        }
        output.push_str(text);
        Ok(())
    }

    pub fn block_locator(&mut self, value: &BlockLocator) -> Result<(), ScopeError> {
        match value {
            BlockLocator::UniqueXhtmlId(id) => self.charge(id.len()),
            BlockLocator::NodePath {
                element_ordinals, ..
            } => self.charge(element_ordinals.len().saturating_mul(size_of::<u32>())),
        }
    }

    pub fn element_locator(&mut self, value: &ElementLocator) -> Result<(), ScopeError> {
        self.block_locator(&value.block)?;
        self.charge(
            value
                .element_ordinals
                .len()
                .saturating_mul(size_of::<u32>()),
        )
    }

    /// Charge the owned heap data of one SourceLocation copy.
    pub fn location(&mut self, value: &SourceLocation) -> Result<(), ScopeError> {
        match value.kind() {
            SourceLocationKind::TextRanges(ranges) => {
                self.charge(
                    ranges
                        .len()
                        .saturating_mul(size_of::<nt_domain::source_scope::LocatedRange>()),
                )?;
                for range in ranges {
                    self.charge(range.canonical_path().len())?;
                }
            }
            SourceLocationKind::Element {
                canonical_path,
                locator,
            } => {
                self.charge(canonical_path.len())?;
                self.element_locator(locator)?;
            }
            SourceLocationKind::Attribute {
                canonical_path,
                locator,
                attribute,
            } => {
                self.charge(canonical_path.len().saturating_add(attribute.len()))?;
                self.element_locator(locator)?;
            }
        }
        Ok(())
    }

    pub fn evidence(&mut self, values: &[ClassificationEvidence]) -> Result<(), ScopeError> {
        self.array::<ClassificationEvidence>(values.len())?;
        for value in values {
            self.location(&value.location)?;
            let optional = |value: &Option<String>| value.as_ref().map_or(0, String::len);
            match &value.kind {
                EvidenceKind::Heading { text } => self.charge(text.len())?,
                EvidenceKind::MappingBoundary {
                    label,
                    raw_href,
                    reason,
                    ..
                } => {
                    self.charge(
                        optional(label)
                            .saturating_add(optional(raw_href))
                            .saturating_add(reason.len()),
                    )?;
                }
                EvidenceKind::Link { href, .. } => self.charge(optional(href))?,
                EvidenceKind::Other { description } => self.charge(description.len())?,
                EvidenceKind::Location | EvidenceKind::SpineOrder { .. } => {}
            }
        }
        Ok(())
    }

    pub fn decisions(&mut self, values: &[ConfirmedDecision]) -> Result<(), ScopeError> {
        self.array::<ConfirmedDecision>(values.len())?;
        for value in values {
            self.location(&value.key.location)?;
            self.charge(value.key.context.series_id.as_str().len())?;
            self.charge(value.key.context.volume_id.as_str().len())?;
            self.evidence(&value.evidence)?;
        }
        Ok(())
    }

    fn reference(&mut self, value: &NoteReference) -> Result<(), ScopeError> {
        self.location(&value.location)?;
        self.location(&value.from)?;
        self.charge(value.raw_href.as_ref().map_or(0, String::len))?;
        self.evidence(&value.evidence)?;
        match &value.target {
            NoteTarget::Resolved(location) => self.location(location)?,
            NoteTarget::Missing { description } => self.charge(description.len())?,
            NoteTarget::Ambiguous {
                description,
                candidates,
            } => {
                self.charge(description.len())?;
                self.array::<SourceLocation>(candidates.len())?;
                for location in candidates {
                    self.location(location)?;
                }
            }
        }
        Ok(())
    }

    /// Account for an input's live owned data before evaluating it.
    pub fn input(&mut self, value: &ScopeInput) -> Result<(), ScopeError> {
        self.observations(
            &value.content,
            &value.notes,
            &value.related,
            &value.note_references,
        )?;
        self.charge(value.context.series_id.as_str().len())?;
        self.charge(value.context.volume_id.as_str().len())
    }

    /// Carry only the retained scope buffers into the next import stage.
    pub fn observations(
        &mut self,
        content: &Vec<ContentInput>,
        notes: &Vec<NoteInput>,
        related: &Vec<RelatedText>,
        references: &Vec<NoteReference>,
    ) -> Result<(), ScopeError> {
        self.array::<ContentInput>(content.capacity())?;
        self.array::<NoteInput>(notes.capacity())?;
        self.array::<RelatedText>(related.capacity())?;
        self.array::<NoteReference>(references.capacity())?;
        for item in content.iter().chain(notes.iter().map(|note| &note.content)) {
            self.content(item)?;
        }
        for item in related {
            self.related(item)?;
        }
        for reference in references {
            self.reference(reference)?;
            self.array::<ClassificationEvidence>(
                reference.evidence.capacity() - reference.evidence.len(),
            )?;
        }
        Ok(())
    }

    fn content(&mut self, value: &ContentInput) -> Result<(), ScopeError> {
        let item = &value.observation;
        self.location(&item.location)?;
        self.charge(item.excerpt.capacity())?;
        // The small private kind vector uses the same fixed set of kinds for
        // all source records. Account for its builder's spare slots.
        self.slots::<ContentKind>(item.possible_kinds().len())?;
        self.evidence(&item.evidence)?;
        self.array::<ClassificationEvidence>(item.evidence.capacity() - item.evidence.len())?;
        self.decisions(&value.history)?;
        self.array::<ConfirmedDecision>(value.history.capacity() - value.history.len())
    }

    fn related(&mut self, item: &RelatedText) -> Result<(), ScopeError> {
        self.location(&item.location)?;
        self.charge(item.text.capacity())?;
        self.evidence(&item.evidence)?;
        self.array::<ClassificationEvidence>(item.evidence.capacity() - item.evidence.len())?;
        self.array::<Relation>(item.relations.capacity())?;
        for relation in &item.relations {
            match relation {
                Relation::Resolved(location) => self.location(location)?,
                Relation::Missing { description } => self.charge(description.len())?,
                Relation::Ambiguous {
                    description,
                    candidates,
                } => {
                    self.charge(description.len())?;
                    self.array::<SourceLocation>(candidates.capacity())?;
                    for location in candidates {
                        self.location(location)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// First reserve the evaluated source data, then any overlapping conversion.
    pub fn result(&mut self, value: &ScopeEvaluation) -> Result<(), ScopeError> {
        self.array::<EvaluatedContent>(value.content.capacity())?;
        self.array::<EvaluatedNote>(value.notes.capacity())?;
        self.array::<EvaluatedRelated>(value.related.capacity())?;
        self.array::<NoteReference>(value.note_references.capacity())?;
        for item in value
            .content
            .iter()
            .chain(value.notes.iter().map(|note| &note.content))
        {
            self.content(&item.input)?;
            use nt_domain::source_scope::DecisionMatch;
            match &item.decision {
                DecisionMatch::Reused(decision) => {
                    self.decisions(std::slice::from_ref(decision.as_ref()))?
                }
                DecisionMatch::NeedsComparison(decisions) => self.decisions(decisions)?,
                DecisionMatch::Unconfirmed => {}
            }
        }
        for item in &value.related {
            self.related(&item.input)?;
        }
        for note in &value.notes {
            self.array::<NoteReference>(note.references.capacity())?;
            for reference in &note.references {
                self.reference(reference)?;
                self.array::<ClassificationEvidence>(
                    reference.evidence.capacity() - reference.evidence.len(),
                )?;
            }
        }
        for reference in &value.note_references {
            self.reference(reference)?;
            self.array::<ClassificationEvidence>(
                reference.evidence.capacity() - reference.evidence.len(),
            )?;
        }
        Ok(())
    }

    pub(super) fn evaluation(&mut self, input: &ScopeInput) -> Result<(), ScopeError> {
        let nodes = input
            .content
            .len()
            .saturating_add(input.notes.len())
            .saturating_add(input.related.len());
        self.array::<EvaluatedContent>(input.content.len())?;
        self.array::<EvaluatedNote>(input.notes.len())?;
        self.array::<EvaluatedRelated>(input.related.len())?;
        self.slots::<(SourceLocation, usize)>(nodes)?;
        self.array::<Vec<usize>>(nodes)?;
        // The first queue drains; the second starts with one entry per node
        // and can add at most one per increase in its four-level handling state.
        self.array::<usize>(nodes.saturating_mul(10))?;
        self.array::<super::Handling>(nodes)?;
        self.array::<bool>(nodes)?;
        for item in input
            .content
            .iter()
            .chain(input.notes.iter().map(|note| &note.content))
        {
            self.location(&item.observation.location)?;
            self.decisions(&item.history)?;
        }
        for item in &input.related {
            self.location(&item.location)?;
            self.slots::<usize>(item.relations.len())?;
        }
        let references = &input.note_references;
        self.slots::<NoteReference>(references.len())?;
        self.slots::<NoteReference>(references.len())?;
        self.slots::<(SourceLocation, usize)>(references.len())?;
        self.slots::<usize>(references.len())?;
        for reference in references {
            self.location(&reference.location)?;
            if matches!(reference.target, NoteTarget::Resolved(_)) {
                self.reference(reference)?;
            }
        }
        Ok(())
    }
}
