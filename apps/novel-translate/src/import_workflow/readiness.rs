use crate::import_codec::{
    InventorySeverity, ScopeDecisionMatch, ScopeHandling, ScopeRecord, ScopeTarget, SourceIssue,
    StoredMainFlow, StoredMapIssue, VolumeRecord,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportReadiness {
    Ready,
    NeedsReview { issues: Vec<ReviewIssue> },
}

/// The index identifies the evidence in this saved record's indicated
/// diagnostic, main flow, scope record or note list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewIssue {
    Source(usize),
    Navigation(usize),
    ResourceReference(usize),
    Mapping(usize),
    UnclassifiedBoundary(usize),
    Classification(usize),
    Relationship(usize),
    DecisionComparison(usize),
    NoteClassification(usize),
    NoteDecisionComparison(usize),
}

impl ImportReadiness {
    pub(super) fn from_record(record: &VolumeRecord) -> Self {
        use ReviewIssue::*;

        let mut issues = Vec::new();
        for (index, issue) in record.diagnostics.source.iter().enumerate() {
            match issue {
                SourceIssue::InventoryPartial {} => {}
                SourceIssue::Inventory { value } if value.severity == InventorySeverity::Info => {}
                _ => issues.push(Source(index)),
            }
        }
        issues.extend((0..record.diagnostics.navigation.len()).map(Navigation));
        issues
            .extend((0..record.diagnostics.unresolved_resource_uses.len()).map(ResourceReference));
        for (index, issue) in record.diagnostics.mapping.iter().enumerate() {
            if !matches!(issue, StoredMapIssue::InputDiagnostic { .. }) {
                issues.push(Mapping(index));
            }
        }
        for (index, region) in record.mapping.volume.main_flow.iter().enumerate() {
            if matches!(region, StoredMainFlow::Unclassified { .. }) {
                issues.push(UnclassifiedBoundary(index));
            }
        }
        for (index, item) in record.scope.records.iter().enumerate() {
            match item {
                ScopeRecord::Content(content) => {
                    if content.handling == ScopeHandling::NeedsClassification {
                        issues.push(Classification(index));
                    }
                    if matches!(content.decision, ScopeDecisionMatch::NeedsComparison { .. }) {
                        issues.push(DecisionComparison(index));
                    }
                }
                ScopeRecord::Related(text) => {
                    if text.handling == ScopeHandling::NeedsClassification {
                        issues.push(Classification(index));
                    }
                    if text.handling == ScopeHandling::NeedsRelationship
                        || text
                            .input
                            .relations
                            .iter()
                            .any(|relation| !matches!(relation, ScopeTarget::Resolved { .. }))
                    {
                        issues.push(Relationship(index));
                    }
                }
                ScopeRecord::NoteReference(link)
                    if !matches!(link.target, ScopeTarget::Resolved { .. }) =>
                {
                    issues.push(Relationship(index));
                }
                ScopeRecord::NoteReference(_) => {}
            }
        }
        for (index, note) in record.scope.notes.iter().enumerate() {
            if note.content.handling == ScopeHandling::NeedsClassification {
                issues.push(NoteClassification(index));
            }
            if matches!(
                note.content.decision,
                ScopeDecisionMatch::NeedsComparison { .. }
            ) {
                issues.push(NoteDecisionComparison(index));
            }
        }
        if issues.is_empty() {
            Self::Ready
        } else {
            Self::NeedsReview { issues }
        }
    }
}
