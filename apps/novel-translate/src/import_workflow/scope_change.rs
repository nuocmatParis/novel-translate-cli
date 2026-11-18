use std::collections::HashSet;
use std::path::Path;

use nt_application::source_scope::{
    ContentInput, NoteInput, RelatedText, ScopeEvaluation, ScopeInput, ScopeWorking,
    evaluate_scope_within,
};
use nt_application::volume_import::{
    CapturedSourceError, GenerationCommitError, ImportOperation, ImportWriteError,
    MAX_IMPORT_WORKING_BYTES, VolumeImportWriteRequest, VolumeState,
    source_scope::{ConfirmedDecision, ContentKind, DecisionKey, ScopeMode},
};
use nt_storage::FileObjectStore;

use super::persisted::read_scope;
use super::publication::prepare_staged_scope;
use super::{ImportFlowError, ImportTarget, PreparedImport, read_saved_volume};
use crate::import_codec::StoredScope;

pub struct Classification {
    pub key: DecisionKey,
    pub kind: ContentKind,
}

pub struct ScopeChange {
    pub mode: ScopeMode,
    pub classifications: Vec<Classification>,
    pub save_as_series_default: bool,
}

/// Prepare an explicit scope edit against its displayed generation and source.
pub fn prepare_scope_change(
    workspace: &Path,
    target: ImportTarget,
    change: ScopeChange,
) -> Result<PreparedImport, ImportFlowError> {
    let before = read_saved_volume(workspace, &target.series, &target.volume, None)?;
    if before.generation != target.expected_parent {
        return Err(ImportWriteError::from(GenerationCommitError::StaleGeneration).into());
    }
    let default_delta = change.save_as_series_default.then_some(change.mode);
    let Some(mut record) = before.record else {
        if !change.classifications.is_empty() {
            return Err(ImportFlowError::InvalidSnapshot);
        }
        let request = VolumeImportWriteRequest::new(
            target.series,
            target.operation,
            target.expected_parent,
            target.generation,
            ImportOperation::SetVolumeScope {
                volume_id: target.volume.clone(),
                state: VolumeState::Unimported { mode: change.mode },
                resources: Vec::new(),
                default_delta,
            },
            Vec::new(),
        )?;
        return Ok(PreparedImport::new(workspace, target.volume, request));
    };
    let scope = read_scope(
        workspace,
        &target.series,
        &target.volume,
        &before.generation,
        &mut record,
    )?;
    let base = record.retained_working_bytes()?;
    let mut space = ScopeWorking::new(
        MAX_IMPORT_WORKING_BYTES
            .checked_sub(base)
            .ok_or(CapturedSourceError::ResourceLimit)?,
    );
    space.result(&scope).map_err(ImportFlowError::Scope)?;
    space
        .slots::<ContentInput>(scope.content.len())
        .map_err(ImportFlowError::Scope)?;
    space
        .slots::<NoteInput>(scope.notes.len())
        .map_err(ImportFlowError::Scope)?;
    space
        .slots::<RelatedText>(scope.related.len())
        .map_err(ImportFlowError::Scope)?;
    let mut input = scope_inputs(scope);
    input.mode = change.mode;
    let mut seen = HashSet::new();
    for classification in change.classifications {
        space
            .slots::<nt_application::volume_import::source_scope::SourceLocation>(1)
            .map_err(ImportFlowError::Scope)?;
        space
            .location(&classification.key.location)
            .map_err(ImportFlowError::Scope)?;
        if classification.key.context != input.context
            || !seen.insert(classification.key.location.clone())
        {
            return Err(ImportFlowError::InvalidSnapshot);
        }
        let item = input
            .content
            .iter_mut()
            .chain(input.notes.iter_mut().map(|note| &mut note.content))
            .find(|item| item.observation.location == classification.key.location)
            .ok_or(ImportFlowError::InvalidSnapshot)?;
        space
            .slots::<ConfirmedDecision>(item.history.len().saturating_add(1))
            .map_err(ImportFlowError::Scope)?;
        space
            .evidence(&item.observation.evidence)
            .map_err(ImportFlowError::Scope)?;
        item.history.push(ConfirmedDecision {
            key: classification.key,
            kind: classification.kind,
            evidence: item.observation.evidence.clone(),
        });
    }
    let evaluated = evaluate_scope_within(input, &mut space).map_err(ImportFlowError::Scope)?;
    record.scope = StoredScope::capture(
        &evaluated,
        base.checked_add(space.used())
            .ok_or(CapturedSourceError::ResourceLimit)?,
    )?;
    drop(evaluated);
    let objects = FileObjectStore::new(workspace).map_err(ImportFlowError::Files)?;
    let staged = record.stage(&objects, 0).map_err(ImportFlowError::Stage)?;
    prepare_staged_scope(workspace, target, staged, default_delta)
}

fn scope_inputs(value: ScopeEvaluation) -> ScopeInput {
    ScopeInput {
        context: value.context,
        mode: value.mode,
        content: value.content.into_iter().map(|item| item.input).collect(),
        notes: value
            .notes
            .into_iter()
            .map(|item| NoteInput {
                content: item.content.input,
            })
            .collect(),
        related: value.related.into_iter().map(|item| item.input).collect(),
        note_references: value.note_references,
    }
}
