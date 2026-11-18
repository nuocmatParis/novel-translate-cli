use std::collections::HashMap;

use nt_application::source_scope::{ScopeEvaluation, ScopeWorking};
use nt_application::volume_import::{
    chapter_map::ChapterMapInput,
    source_range::{BlockLocator, SourceBlockId},
    source_scope::{ConfirmedDecision, SourceLocation, SourceLocationKind},
};

use super::{ImportFlowError, scope::ObservedScope};
use crate::import_codec::{CodecError, StoredSourceIndex};

type Origin = Vec<(String, BlockLocator)>;

/// History candidates follow a source element. Reuse still checks every
/// context component and the exact ranges in the domain decision evaluator.
#[derive(Default)]
pub(super) struct PreviousClassifications {
    candidates: HashMap<Origin, Vec<ConfirmedDecision>>,
}

impl PreviousClassifications {
    pub fn capture(index: &StoredSourceIndex, scope: ScopeEvaluation) -> Result<Self, CodecError> {
        let mut classified = scope
            .content
            .into_iter()
            .map(|item| item.input)
            .chain(scope.notes.into_iter().map(|note| note.content.input))
            .filter(|item| !item.history.is_empty())
            .peekable();
        if classified.peek().is_none() {
            return Ok(Self::default());
        }
        let blocks = index
            .documents
            .iter()
            .flat_map(|document| &document.blocks)
            .map(|block| {
                Ok((
                    SourceBlockId::from_bytes(block.id.0),
                    block.locator.source_locator()?,
                ))
            })
            .collect::<Result<HashMap<_, _>, CodecError>>()?;
        let mut result = Self::default();
        for item in classified {
            let origin = source_origin(&item.observation.location, &blocks)?;
            let history = result.candidates.entry(origin).or_default();
            for decision in item.history {
                if !history.contains(&decision) {
                    history.push(decision);
                }
            }
        }
        Ok(result)
    }

    pub fn apply(
        self,
        scope: &mut ObservedScope,
        input: &ChapterMapInput,
        space: &mut ScopeWorking,
    ) -> Result<(), ImportFlowError> {
        if self.candidates.is_empty() {
            return Ok(());
        }
        for block in input.documents.iter().flat_map(|document| &document.blocks) {
            space
                .slots::<(SourceBlockId, BlockLocator)>(1)
                .map_err(ImportFlowError::Scope)?;
            space
                .block_locator(&block.locator)
                .map_err(ImportFlowError::Scope)?;
        }
        let blocks: HashMap<_, _> = input
            .documents
            .iter()
            .flat_map(|document| &document.blocks)
            .map(|block| (block.id, block.locator.clone()))
            .collect();
        for item in scope
            .content
            .iter_mut()
            .chain(scope.notes.iter_mut().map(|note| &mut note.content))
        {
            let location = &item.observation.location;
            let SourceLocationKind::TextRanges(ranges) = location.kind() else {
                return Err(CodecError::InvalidRecord.into());
            };
            space
                .slots::<(String, BlockLocator)>(ranges.len())
                .map_err(ImportFlowError::Scope)?;
            for range in ranges {
                let locator = blocks
                    .get(&range.range().block_id())
                    .ok_or(CodecError::InvalidRecord)?;
                space
                    .charge(range.canonical_path().len())
                    .map_err(ImportFlowError::Scope)?;
                space
                    .block_locator(locator)
                    .map_err(ImportFlowError::Scope)?;
            }
            let origin = source_origin(location, &blocks)?;
            if let Some(candidates) = self.candidates.get(&origin) {
                space
                    .decisions(candidates)
                    .map_err(ImportFlowError::Scope)?;
                item.history = candidates.clone();
            }
        }
        Ok(())
    }
}

fn source_origin(
    location: &SourceLocation,
    blocks: &HashMap<SourceBlockId, BlockLocator>,
) -> Result<Origin, CodecError> {
    let SourceLocationKind::TextRanges(ranges) = location.kind() else {
        return Err(CodecError::InvalidRecord);
    };
    let mut origin = Vec::with_capacity(ranges.len());
    for range in ranges {
        let locator = blocks
            .get(&range.range().block_id())
            .ok_or(CodecError::InvalidRecord)?;
        let item = (range.canonical_path().to_owned(), locator.clone());
        if origin.last() != Some(&item) {
            origin.push(item);
        }
    }
    Ok(origin)
}
