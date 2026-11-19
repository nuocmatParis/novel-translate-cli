use std::path::{Path, PathBuf};

use nt_application::volume_import::{
    GenerationId, ObjectHash, ObjectReference, OperationId, SeriesId, VolumeId,
    chapter_map::{
        BoundaryEvidenceKind, ChapterMap, ChapterMapInput, ConfirmedBoundary, MainFlowContent,
        MappingStatus,
        edit::{
            ChapterBoundary, ChapterEdit, ChapterEditError, ChapterEditImpact, preview_chapter_edit,
        },
        validate_coverage,
    },
};

use crate::import_codec::{
    Budget, CodecError, StoredMainFlow, StoredMapping, VolumeRecord, memory::working,
};

use super::prepare::prepare_edited_mapping;
use super::{ImportFlowError, ImportTarget, PreparedImport, SavedVolume, read_saved_volume};

pub(super) const MANUAL_MAPPING_RULE: &str = "nt-chapter-map/1/manual/1";

/// The exact source displayed while the user edits a volume mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MappingViewVersion {
    pub series: SeriesId,
    pub volume: VolumeId,
    pub generation: GenerationId,
    pub epub_sha256: [u8; 32],
    pub parser_version: String,
    pub mapping_version: [u8; 32],
}

#[derive(Debug)]
pub struct MappingDraft {
    workspace: PathBuf,
    version: MappingViewVersion,
    edit: ChapterEdit,
    preview: MappingPreview,
}

#[derive(Debug, Eq, PartialEq)]
pub struct MappingPreview {
    pub before: Vec<ChapterPreview>,
    pub after: Vec<ChapterPreview>,
    pub impact: ChapterEditImpact,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ChapterPreview {
    pub chapter_id: u32,
    pub label: Option<String>,
    pub status: MappingStatus,
    pub start: ChapterBoundary,
    pub end: ChapterBoundary,
    pub source_range_count: usize,
}

impl MappingDraft {
    pub fn workspace(&self) -> &Path {
        &self.workspace
    }

    pub fn version(&self) -> &MappingViewVersion {
        &self.version
    }

    pub fn edit(&self) -> &ChapterEdit {
        &self.edit
    }

    pub fn preview(&self) -> &MappingPreview {
        &self.preview
    }

    pub(super) fn verified_original(
        &self,
        record: Option<&VolumeRecord>,
    ) -> Result<ObjectReference, ImportFlowError> {
        let source = &record.ok_or(ImportFlowError::InvalidSnapshot)?.source;
        if source.epub_sha256.0 != self.version.epub_sha256
            || source.parser_version != self.version.parser_version
            || source.mapping_version.0 != self.version.mapping_version
        {
            return Err(ImportFlowError::InvalidSnapshot);
        }
        Ok(ObjectReference {
            hash: ObjectHash::from_bytes(source.original_object_sha256.0),
            byte_len: source.original_object_byte_len,
        })
    }
}

#[derive(Debug)]
pub enum MappingEditError {
    Import(ImportFlowError),
    Edit(ChapterEditError),
    MissingImport,
}

impl From<ImportFlowError> for MappingEditError {
    fn from(error: ImportFlowError) -> Self {
        Self::Import(error)
    }
}

impl From<CodecError> for MappingEditError {
    fn from(error: CodecError) -> Self {
        Self::Import(error.into())
    }
}

/// Read the verified source from the user's displayed generation and propose
/// one edit. The saved generation and its source objects stay unchanged.
pub fn preview_mapping_edit(
    workspace: &Path,
    series: &SeriesId,
    volume: &VolumeId,
    displayed_generation: &GenerationId,
    edit: ChapterEdit,
) -> Result<MappingDraft, MappingEditError> {
    let SavedVolume {
        generation,
        record,
        version,
    } = read_saved_volume(workspace, series, volume, Some(displayed_generation))?;
    drop(version);
    let record = record.ok_or(MappingEditError::MissingImport)?;

    // The saved-map reservation covers existing strings. The new title
    // also appears in the summary produced while the proposed map is live.
    let mut budget = Budget::new(record.scope_rebuild_bytes(0)?)?;
    reserve_edit_label(&mut budget, &edit, 1)?;
    let VolumeRecord {
        source,
        mapping,
        inventory,
        source_index,
        scope,
        resources,
        diagnostics,
        ..
    } = record;
    drop((inventory, source_index, scope, resources, diagnostics));
    let (mapping, _) = mapping.into_map()?;
    let selected = match &edit {
        ChapterEdit::Rename { chapter, .. } | ChapterEdit::Split { chapter, .. } => {
            [*chapter, *chapter]
        }
        ChapterEdit::Merge { first, second } | ChapterEdit::MoveBoundary { first, second, .. } => {
            [*first, *second]
        }
    };
    let before = preview_chapters(&mapping, &selected);
    let preview = preview_chapter_edit(mapping, &edit).map_err(MappingEditError::Edit)?;
    let after = preview_chapters(&preview.mapping, &preview.impact.chapters);
    Ok(MappingDraft {
        workspace: workspace.to_path_buf(),
        version: MappingViewVersion {
            series: series.clone(),
            volume: volume.clone(),
            generation,
            epub_sha256: source.epub_sha256.0,
            parser_version: source.parser_version,
            mapping_version: source.mapping_version.0,
        },
        edit,
        preview: MappingPreview {
            before,
            after,
            impact: preview.impact,
        },
    })
}

/// Recheck the displayed generation and stage its explicit edit. The
/// transaction handle checks that same generation again when submitted.
pub fn prepare_mapping_confirmation(
    draft: &MappingDraft,
    operation: OperationId,
    generation: GenerationId,
) -> Result<PreparedImport, ImportFlowError> {
    let expected = draft.version();
    prepare_edited_mapping(
        draft,
        ImportTarget {
            series: expected.series.clone(),
            volume: expected.volume.clone(),
            expected_parent: expected.generation.clone(),
            operation,
            generation,
        },
    )
}

pub(super) fn edit_mapping_reservation(
    record: &VolumeRecord,
    draft: Option<&MappingDraft>,
    prior: usize,
) -> Result<usize, CodecError> {
    let stored = working(&record.mapping, 0)?;
    // Use the same restore allowance as CP05 scope rebuilding. Keep an
    // additional copy live for edited vectors while staging starts.
    let mut budget = Budget::new(prior)?;
    budget.charge(stored.checked_mul(4).ok_or(CodecError::ResourceLimit)?, 0)?;
    if let Some(draft) = draft {
        reserve_confirmation_labels(
            &mut budget,
            &draft.edit,
            &draft.preview,
            active_label_copies(record, &draft.edit),
        )?;
        for chapter in draft.preview.before.iter().chain(&draft.preview.after) {
            budget.charge(std::mem::size_of::<ChapterPreview>(), 1)?;
            budget.charge(chapter.start.canonical_path.capacity(), 0)?;
            budget.charge(chapter.end.canonical_path.capacity(), 0)?;
        }
    }
    Ok(budget.used_bytes())
}

fn reserve_edit_label(
    budget: &mut Budget,
    edit: &ChapterEdit,
    additional_copies: usize,
) -> Result<(), CodecError> {
    let (label, map_copies) = match edit {
        ChapterEdit::Rename { label, .. } => (label, 1_usize),
        ChapterEdit::Split {
            label: Some(label), ..
        } => (label, 3),
        _ => return Ok(()),
    };
    // An edited chapter owns its title. Split also retains it in the
    // start evidence and in the map's boundary evidence.
    let copies = map_copies
        .checked_add(additional_copies)
        .ok_or(CodecError::ResourceLimit)?;
    let bytes = label
        .len()
        .checked_mul(copies)
        .and_then(|bytes| bytes.checked_add(label.capacity()))
        .ok_or(CodecError::ResourceLimit)?;
    budget.charge(bytes, 0)
}

fn reserve_confirmation_labels(
    budget: &mut Budget,
    edit: &ChapterEdit,
    preview: &MappingPreview,
    active_copies: usize,
) -> Result<(), CodecError> {
    for label in preview
        .before
        .iter()
        .chain(&preview.after)
        .filter_map(|chapter| chapter.label.as_ref())
    {
        budget.charge(label.capacity(), 0)?;
    }
    // The comparison summary is dropped before retaining confirmed
    // boundaries. Reserve the larger of those two temporary projections.
    reserve_edit_label(budget, edit, active_copies.max(1))
}

fn active_label_copies(record: &VolumeRecord, edit: &ChapterEdit) -> usize {
    match edit {
        ChapterEdit::Rename { chapter, .. } => record
            .mapping
            .volume
            .main_flow
            .iter()
            .find_map(|flow| match flow {
                StoredMainFlow::Chapter(value) if value.chapter_id == *chapter => {
                    Some(value.start_evidence.len())
                }
                _ => None,
            })
            .unwrap_or(0),
        ChapterEdit::Split { .. } => 1,
        _ => 0,
    }
}

pub(super) fn restore_manual_mapping(
    stored: StoredMapping,
    draft: Option<&MappingDraft>,
    input: &ChapterMapInput,
) -> Result<(ChapterMap, Vec<ConfirmedBoundary>), ImportFlowError> {
    let (original, old_boundaries) = stored.into_map()?;
    let (mapping, confirmed) = if let Some(draft) = draft {
        let edited = preview_chapter_edit(original, draft.edit())
            .map_err(|_| ImportFlowError::InvalidSnapshot)?;
        if edited.impact != draft.preview.impact
            || preview_chapters(&edited.mapping, &edited.impact.chapters) != draft.preview.after
        {
            return Err(ImportFlowError::InvalidSnapshot);
        }
        let confirmed = active_boundaries(&edited.mapping)?;
        (edited.mapping, confirmed)
    } else {
        (original, old_boundaries)
    };
    validate_coverage(input, &mapping).map_err(ImportFlowError::Mapping)?;
    Ok((mapping, confirmed))
}

fn active_boundaries(mapping: &ChapterMap) -> Result<Vec<ConfirmedBoundary>, ImportFlowError> {
    let mut confirmed = Vec::new();
    for item in &mapping.volume.main_flow {
        let MainFlowContent::Chapter(chapter) = item else {
            continue;
        };
        for evidence in chapter
            .start_evidence
            .iter()
            .filter(|evidence| evidence.kind == BoundaryEvidenceKind::Confirmed)
        {
            confirmed.push(ConfirmedBoundary {
                epub_sha256: mapping.epub_sha256,
                parser_version: mapping.parser_version.clone(),
                canonical_path: evidence.canonical_path.clone(),
                block_id: evidence.block_id.ok_or(ImportFlowError::InvalidSnapshot)?,
                unit_offset: evidence
                    .unit_offset
                    .ok_or(ImportFlowError::InvalidSnapshot)?,
                label: chapter.label.clone(),
            });
        }
    }
    Ok(confirmed)
}

fn preview_chapters(mapping: &ChapterMap, selected: &[u32]) -> Vec<ChapterPreview> {
    mapping
        .volume
        .main_flow
        .iter()
        .filter_map(|item| {
            let MainFlowContent::Chapter(chapter) = item else {
                return None;
            };
            if !selected.contains(&chapter.chapter_id) {
                return None;
            }
            let first = chapter.ranges.first()?;
            let last = chapter.ranges.last()?;
            Some(ChapterPreview {
                chapter_id: chapter.chapter_id,
                label: chapter.label.clone(),
                status: chapter.status,
                start: ChapterBoundary {
                    canonical_path: first.canonical_path.clone(),
                    block_id: first.block_id,
                    offset: first.start,
                },
                end: ChapterBoundary {
                    canonical_path: last.canonical_path.clone(),
                    block_id: last.block_id,
                    offset: last.end,
                },
                source_range_count: chapter.ranges.len(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use nt_application::volume_import::{MAX_IMPORT_WORKING_BYTES, source_range::SourceBlockId};

    use super::*;

    fn split_edit(label: String) -> ChapterEdit {
        ChapterEdit::Split {
            chapter: 1,
            boundary: ChapterBoundary {
                canonical_path: "OPS/chapter.xhtml".to_owned(),
                block_id: SourceBlockId::from_bytes([1; 32]),
                offset: 1,
            },
            label: Some(label),
        }
    }

    fn check_label_limit(required: usize, reserve: impl Fn(&mut Budget) -> Result<(), CodecError>) {
        let mut exact = Budget::new(MAX_IMPORT_WORKING_BYTES - required).unwrap();
        reserve(&mut exact).unwrap();
        assert_eq!(exact.remaining_bytes(), 0);

        let mut short = Budget::new(MAX_IMPORT_WORKING_BYTES - required + 1).unwrap();
        assert_eq!(reserve(&mut short), Err(CodecError::ResourceLimit));
    }

    #[test]
    fn split_preview_reserves_the_edit_and_four_title_copies() {
        let mut label = String::with_capacity(24);
        label.push_str("序章");
        let required = label.capacity() + 4 * label.len();
        let edit = split_edit(label);

        check_label_limit(required, |budget| reserve_edit_label(budget, &edit, 1));
    }

    #[test]
    fn split_confirmation_also_reserves_the_held_summary_title() {
        let mut label = String::with_capacity(24);
        label.push_str("後編");
        let held_label = label.clone();
        let required = label.capacity() + 4 * label.len() + held_label.capacity();
        let edit = split_edit(label);
        let ChapterEdit::Split { boundary, .. } = &edit else {
            unreachable!("fixture must be a split");
        };
        let preview = MappingPreview {
            before: Vec::new(),
            after: vec![ChapterPreview {
                chapter_id: 2,
                label: Some(held_label),
                status: MappingStatus::Confirmed,
                start: boundary.clone(),
                end: ChapterBoundary {
                    offset: 2,
                    ..boundary.clone()
                },
                source_range_count: 1,
            }],
            impact: ChapterEditImpact {
                chapters: vec![1, 2],
                previous_boundary: None,
                new_boundary: Some(boundary.clone()),
            },
        };
        check_label_limit(required, |budget| {
            reserve_confirmation_labels(budget, &edit, &preview, 1)
        });
    }
}
