//! Rebuild classification observations from authenticated compact source data.
use std::collections::HashMap;

use nt_application::source_scope::{
    ScopeEvaluation, ScopeTextSource, ScopeWorking, observe_source_text_within,
};
use nt_application::volume_import::{
    MAX_IMPORT_WORKING_BYTES,
    chapter_map::MainFlowContent,
    source_range::{
        AttributeSourceText, ElementLocator, InlineMetadataKind, InlineSourceMetadata,
        SourceBlockId, UnitRange,
    },
};
use nt_epub::SourceAnchor;

use super::{
    ImportFlowError,
    scope::{ObservedScope, ScopeBlock, ScopeEvidence, assemble_scope},
};
use crate::import_codec::{
    CaptureEvidence, CodecError, StoredInlineKind, StoredProtectedUnit, StoredSourceBlock,
    StoredUnitRun, VolumeRecord,
};

pub(super) fn rebuild_scope(
    record: &VolumeRecord,
    evidence: &CaptureEvidence,
) -> Result<(ObservedScope, usize), ImportFlowError> {
    let reserved = record.scope_rebuild_bytes(evidence.working_bytes)?;
    let mut space = ScopeWorking::new(
        MAX_IMPORT_WORKING_BYTES
            .checked_sub(reserved)
            .ok_or(CodecError::ResourceLimit)?,
    );
    let mut blocks = Vec::new();
    let mut positions = HashMap::new();
    let mut spine_positions = Vec::new();
    let mut source_blocks = HashMap::new();
    space
        .slots::<Vec<usize>>(record.source_index.documents.len())
        .map_err(ImportFlowError::Scope)?;
    for (document_index, document) in record.source_index.documents.iter().enumerate() {
        space
            .slots::<usize>(document.spine_positions.len())
            .map_err(ImportFlowError::Scope)?;
        let positions_for_document = document
            .spine_positions
            .iter()
            .map(|position| usize::try_from(*position).map_err(|_| CodecError::ResourceLimit))
            .collect::<Result<Vec<_>, _>>()?;
        spine_positions.push(positions_for_document);
        for block in &document.blocks {
            let view = block_view(block)?;
            positions.insert(view.id, document_index);
            source_blocks.insert(view.id, block);
            blocks.push(view);
        }
    }
    let views: HashMap<_, _> = blocks.iter().map(|block| (block.id, block)).collect();
    let (mapping, _) = record.mapping.clone().into_map()?;
    let mut boundaries = HashMap::<_, Vec<_>>::new();
    for boundary in &mapping.boundary_evidence {
        if let Some(id) = boundary.block_id {
            boundaries.entry(id).or_default().push(boundary);
        }
    }
    let mut observations = Vec::new();
    for ranges in mapping
        .volume
        .main_flow
        .iter()
        .map(|flow| match flow {
            MainFlowContent::Chapter(chapter) => &chapter.ranges,
            MainFlowContent::OutOfChapter { ranges, .. }
            | MainFlowContent::Unclassified { ranges, .. } => ranges,
        })
        .chain(mapping.volume.auxiliary.iter().map(|item| &item.ranges))
    {
        for range in ranges {
            let block = views
                .get(&range.block_id)
                .ok_or(CodecError::InvalidRecord)?;
            let text = excerpt(
                source_blocks[&range.block_id],
                range.start,
                range.end,
                &mut space,
            )?;
            let spine = &spine_positions[positions[&block.id]];
            if let Some(item) = observe_source_text_within(
                ScopeTextSource {
                    id: block.id,
                    canonical_path: &block.canonical_path,
                    element_name: &block.element_name,
                    unit_count: block.unit_count as usize,
                    spine_positions: spine,
                    inline_metadata: &block.inline_metadata,
                },
                range,
                boundaries.get(&block.id).map_or(&[], Vec::as_slice),
                text,
                &mut space,
            )
            .map_err(ImportFlowError::Scope)?
            {
                observations.push(item);
            }
        }
    }
    let anchors = record
        .source_index
        .anchors
        .iter()
        .map(|anchor| {
            Ok(SourceAnchor {
                canonical_path: anchor.canonical_path.clone(),
                fragment: anchor.fragment.clone(),
                locator: locator(&anchor.locator)?,
                block_id: SourceBlockId::from_bytes(anchor.block_id.0),
                unit_offset: anchor.unit_offset,
                is_unique: anchor.is_unique,
            })
        })
        .collect::<Result<Vec<_>, CodecError>>()?;
    let observed = assemble_scope(
        &blocks,
        observations,
        ScopeEvidence {
            placements: &evidence.placements,
            anchors: &anchors,
            diagnostics: &evidence.diagnostics,
            references: &evidence.references,
            navigation_labels: &evidence.navigation_labels,
            navigation: &mapping.navigation,
        },
        &mut space,
    )
    .map_err(ImportFlowError::Scope)?;
    let retained = observed
        .retained_working_bytes()
        .map_err(ImportFlowError::Scope)?;
    Ok((observed, retained))
}

pub(super) fn verify_observations(
    actual: ObservedScope,
    saved: &ScopeEvaluation,
) -> Result<(), ImportFlowError> {
    let expected = actual.with_context(saved.context.clone(), saved.mode);
    if expected.content.len() != saved.content.len()
        || expected.notes.len() != saved.notes.len()
        || expected.related.len() != saved.related.len()
        || expected.note_references != saved.note_references
        || !expected
            .content
            .iter()
            .zip(&saved.content)
            .all(|(a, b)| a.observation == b.input.observation)
        || !expected
            .notes
            .iter()
            .zip(&saved.notes)
            .all(|(a, b)| a.content.observation == b.content.input.observation)
        || !expected
            .related
            .iter()
            .zip(&saved.related)
            .all(|(a, b)| a == &b.input)
    {
        return Err(ImportFlowError::InvalidSnapshot);
    }
    Ok(())
}

fn locator(value: &crate::import_codec::ElementLocator) -> Result<ElementLocator, CodecError> {
    Ok(ElementLocator {
        block: value.block.source_locator()?,
        element_ordinals: value.element_ordinals.clone(),
    })
}

fn block_view(block: &StoredSourceBlock) -> Result<ScopeBlock, CodecError> {
    let inline_metadata = block
        .inline_metadata
        .iter()
        .map(|item| {
            Ok(InlineSourceMetadata {
                locator: locator(&item.locator)?,
                kind: match &item.kind {
                    StoredInlineKind::Formatting { element } => InlineMetadataKind::Formatting {
                        element: element.clone(),
                    },
                    StoredInlineKind::Link { href } => {
                        InlineMetadataKind::Link { href: href.clone() }
                    }
                    StoredInlineKind::NoteReference { href } => {
                        InlineMetadataKind::NoteReference { href: href.clone() }
                    }
                },
                ranges: item
                    .ranges
                    .iter()
                    .map(|range| UnitRange {
                        start: range.start,
                        end: range.end,
                    })
                    .collect(),
            })
        })
        .collect::<Result<_, CodecError>>()?;
    let attribute_text = block
        .attribute_text
        .iter()
        .map(|item| {
            Ok(AttributeSourceText {
                locator: locator(&item.locator)?,
                attribute: item.attribute.clone(),
                value: item.value.clone(),
                resource_path: item.resource_path.clone(),
            })
        })
        .collect::<Result<_, CodecError>>()?;
    let mut images = Vec::new();
    let mut offset = 0u32;
    for run in &block.unit_runs {
        let count = match run {
            StoredUnitRun::Text(run) => run.validate()?.unit_count(),
            StoredUnitRun::Protected(run) => {
                if let StoredProtectedUnit::Image { node_path, .. } = &run.unit {
                    images.push((offset, node_path.clone()));
                }
                1
            }
        };
        offset = offset.checked_add(count).ok_or(CodecError::ResourceLimit)?;
    }
    Ok(ScopeBlock {
        id: SourceBlockId::from_bytes(block.id.0),
        canonical_path: block.canonical_path.clone(),
        element_name: block.element_name.clone(),
        locator: block.locator.source_locator()?,
        unit_count: offset,
        inline_metadata,
        attribute_text,
        images,
    })
}

fn excerpt(
    block: &StoredSourceBlock,
    start: u32,
    end: u32,
    space: &mut ScopeWorking,
) -> Result<String, ImportFlowError> {
    let mut text = String::new();
    let mut offset = 0u32;
    for run in &block.unit_runs {
        match run {
            StoredUnitRun::Text(run) => {
                let checked = run.validate()?;
                let count = checked.unit_count();
                if start < offset + count && offset < end {
                    for (local, unit) in checked.units().enumerate() {
                        let position = offset + local as u32;
                        if start <= position && position < end {
                            space
                                .append(&mut text, unit.text)
                                .map_err(ImportFlowError::Scope)?;
                        }
                    }
                }
                offset = offset.checked_add(count).ok_or(CodecError::ResourceLimit)?;
            }
            StoredUnitRun::Protected(run) => {
                if start <= offset
                    && offset < end
                    && let StoredProtectedUnit::Ruby {
                        base, annotation, ..
                    } = &run.unit
                {
                    space
                        .append(&mut text, base)
                        .map_err(ImportFlowError::Scope)?;
                    space
                        .append(&mut text, annotation)
                        .map_err(ImportFlowError::Scope)?;
                }
                offset = offset.checked_add(1).ok_or(CodecError::ResourceLimit)?;
            }
        }
    }
    if end > offset || start >= end {
        return Err(CodecError::InvalidRecord.into());
    }
    Ok(text)
}
