use std::path::{Path, PathBuf};

use novel_translate::import_codec::{
    ScopeDecisionMatch, ScopeHandling, ScopeLocation, ScopeRecord, StoredBlockRange,
    StoredInlineKind, StoredMainFlow, StoredProtectedUnit, StoredUnitRun, VolumeRecord,
};
use novel_translate::import_workflow::{
    CaptureError, Classification, ImportFlowError, MappingDraft, MappingEditError,
    MappingViewVersion, Publication, ScopeChange, prepare_mapping_confirmation,
    preview_mapping_edit, read_saved_volume, read_volume_scope,
};
use nt_application::volume_import::{
    GenerationCommitError, ImportWriteError, VolumeId,
    chapter_map::edit::{ChapterBoundary, ChapterEdit},
    source_range::SourceBlockId,
    source_scope::DecisionKey,
};
use nt_storage::SqliteVolumeImportReader;
use nt_tui::{
    BookCatalog, BookChange, BookEdit, BookError, BookPosition, BookSource, BookSummary,
    BookVersion, BookView, BookWorkspace, ClassificationTarget, VolumeCommands,
};

use crate::{epub_dialog, import_commands};

pub(crate) struct ImportService;

impl VolumeCommands for ImportService {
    fn choose_epub(&self) -> Result<Option<PathBuf>, BookError> {
        epub_dialog::choose_epub().map_err(|_| BookError::ChooserUnavailable)
    }

    fn import_epub(&self, workspace: &BookWorkspace, epub: &Path) -> Result<BookChange, BookError> {
        let (_, saved) = import_commands::import(
            &workspace.path,
            Some((&workspace.series, &workspace.generation)),
            epub,
            import_commands::ImportDestination::New,
        )
        .map_err(|error| match error {
            import_commands::ImportCommandError::InvalidExtension => BookError::InvalidSource,
            import_commands::ImportCommandError::Import(error) => flow_error(error),
        })?;
        completed(saved)
    }

    fn catalog(&self, workspace: &BookWorkspace) -> Result<BookCatalog, BookError> {
        let mut reader = SqliteVolumeImportReader::open(&workspace.path).map_err(storage_error)?;
        let snapshot = reader.begin_snapshot().map_err(storage_error)?;
        if snapshot.series_id() != &workspace.series {
            return Err(BookError::Stale);
        }
        let generation = snapshot.generation_id().clone();
        let volumes = snapshot.volumes().map_err(storage_error)?;
        snapshot.finish().map_err(storage_error)?;
        Ok(BookCatalog {
            generation,
            volumes: volumes
                .into_iter()
                .map(|volume| {
                    let title = volume
                        .display_title
                        .as_deref()
                        .map(|text| clean(text, 100))
                        .filter(|text| !text.trim().is_empty())
                        .unwrap_or_else(|| format!("Tập {}", &volume.id.as_str()[..8]));
                    BookSummary {
                        id: volume.id,
                        title,
                    }
                })
                .collect(),
        })
    }

    fn view(
        &self,
        workspace: &BookWorkspace,
        volume: &VolumeId,
        position: BookPosition,
    ) -> Result<BookView, BookError> {
        let saved = read_saved_volume(
            &workspace.path,
            &workspace.series,
            volume,
            Some(&workspace.generation),
        )
        .map_err(flow_error)?;
        let mut review_count = 0;
        saved.for_each_review_issue(|_| review_count += 1);
        let mut view = BookView {
            version: None,
            mode: saved.version.state.mode(),
            position: BookPosition::default(),
            rows: 0,
            ranges: 0,
            caption: "Tập chưa nhập nguồn".into(),
            review_count,
            chapter: None,
            previous_chapter: None,
            next_chapter: None,
            source: None,
        };
        if let Some(record) = &saved.record {
            view.version = Some(BookVersion {
                epub_sha256: record.source.epub_sha256.0,
                parser_version: record.source.parser_version.clone(),
                mapping_version: record.source.mapping_version.0,
            });
            describe_tree(record, position, &mut view)?;
        }
        Ok(view)
    }

    fn preview(
        &self,
        workspace: &BookWorkspace,
        volume: &VolumeId,
        edit: &BookEdit,
    ) -> Result<String, BookError> {
        let BookEdit::Mapping { version, edit } = edit else {
            return Err(BookError::InvalidEdit);
        };
        let draft = mapping_draft(workspace, volume, version, edit)?;
        let format_chapters = |chapters: &[novel_translate::import_workflow::ChapterPreview]| {
            chapters
                .iter()
                .map(|chapter| {
                    let title = chapter
                        .label
                        .as_deref()
                        .map(|text| clean(text, 48))
                        .unwrap_or_else(|| format!("Chương {}", chapter.chapter_id));
                    format!(
                        "{} ({} vùng, {}:{} → {}:{})",
                        title,
                        chapter.source_range_count,
                        clean(&chapter.start.canonical_path, 48),
                        chapter.start.offset,
                        clean(&chapter.end.canonical_path, 48),
                        chapter.end.offset,
                    )
                })
                .collect::<Vec<_>>()
                .join(" · ")
        };
        let description = format!(
            "Trước: {}\nSau: {}\nEnter: Xác nhận lưu · Esc: Hủy",
            format_chapters(&draft.preview().before),
            format_chapters(&draft.preview().after),
        );
        Ok(description)
    }

    fn save(
        &self,
        workspace: &BookWorkspace,
        volume: &VolumeId,
        edit: &BookEdit,
    ) -> Result<BookChange, BookError> {
        let expected = Some((&workspace.series, &workspace.generation));
        let result = match edit {
            BookEdit::Mapping { version, edit } => save_mapping(workspace, volume, version, edit)?,
            BookEdit::Scope(mode) | BookEdit::Default(mode) => {
                import_commands::scope(
                    &workspace.path,
                    expected,
                    volume.clone(),
                    ScopeChange {
                        mode: *mode,
                        classifications: Vec::new(),
                        save_as_series_default: matches!(edit, BookEdit::Default(_)),
                    },
                )
                .map_err(flow_error)?
                .1
            }
            BookEdit::Classification { target, kind, mode } => {
                let scope = read_volume_scope(
                    &workspace.path,
                    &workspace.series,
                    volume,
                    Some(&workspace.generation),
                )
                .map_err(flow_error)?
                .ok_or(BookError::InvalidEdit)?;
                let location = match target {
                    ClassificationTarget::Content(index) => scope
                        .content
                        .get(*index)
                        .map(|item| &item.input.observation.location),
                    ClassificationTarget::Note(index) => scope
                        .notes
                        .get(*index)
                        .map(|item| &item.content.input.observation.location),
                }
                .ok_or(BookError::InvalidEdit)?
                .clone();
                import_commands::scope(
                    &workspace.path,
                    expected,
                    volume.clone(),
                    ScopeChange {
                        mode: *mode,
                        save_as_series_default: false,
                        classifications: vec![Classification {
                            key: DecisionKey {
                                context: scope.context,
                                location,
                            },
                            kind: *kind,
                        }],
                    },
                )
                .map_err(flow_error)?
                .1
            }
        };
        completed(result)
    }
}

fn save_mapping(
    workspace: &BookWorkspace,
    volume: &VolumeId,
    version: &BookVersion,
    edit: &ChapterEdit,
) -> Result<Publication, BookError> {
    let mut reader = SqliteVolumeImportReader::open(&workspace.path).map_err(storage_error)?;
    let snapshot = reader.begin_snapshot().map_err(storage_error)?;
    import_commands::check_view(&snapshot, Some((&workspace.series, &workspace.generation)))
        .map_err(flow_error)?;
    let target = import_commands::write_target(&snapshot, volume.clone()).map_err(flow_error)?;
    snapshot.finish().map_err(storage_error)?;
    let draft = mapping_draft(workspace, volume, version, edit)?;
    let prepared = prepare_mapping_confirmation(&draft, target.operation, target.generation)
        .map_err(flow_error)?;
    import_commands::submit(&prepared).map_err(flow_error)
}

fn mapping_draft(
    workspace: &BookWorkspace,
    volume: &VolumeId,
    version: &BookVersion,
    edit: &ChapterEdit,
) -> Result<MappingDraft, BookError> {
    let draft = preview_mapping_edit(
        &workspace.path,
        &workspace.series,
        volume,
        &workspace.generation,
        edit.clone(),
    )
    .map_err(mapping_error)?;
    let displayed = MappingViewVersion {
        series: workspace.series.clone(),
        volume: volume.clone(),
        generation: workspace.generation.clone(),
        epub_sha256: version.epub_sha256,
        parser_version: version.parser_version.clone(),
        mapping_version: version.mapping_version,
    };
    if draft.version() != &displayed {
        return Err(BookError::Stale);
    }
    Ok(draft)
}

fn completed(result: Publication) -> Result<BookChange, BookError> {
    match result {
        Publication::Published { saved, .. } => Ok(BookChange {
            generation: saved.generation,
            volume: saved.version.volume_id,
        }),
        Publication::CommitOutcomeUnknown { operation, .. } => {
            Err(BookError::Unknown(operation.as_str().to_owned()))
        }
        Publication::SavedButUnreadable { .. } => Err(BookError::SavedButUnreadable),
    }
}

fn mapping_error(error: MappingEditError) -> BookError {
    match error {
        MappingEditError::Import(error) => flow_error(error),
        MappingEditError::Edit(_) | MappingEditError::MissingImport => BookError::InvalidEdit,
    }
}

fn storage_error(error: ImportWriteError) -> BookError {
    flow_error(error.into())
}

fn flow_error(error: ImportFlowError) -> BookError {
    match error {
        ImportFlowError::Storage(ImportWriteError::Generation(
            GenerationCommitError::StaleGeneration | GenerationCommitError::OperationConflict,
        ))
        | ImportFlowError::InvalidWorkspace => BookError::Stale,
        ImportFlowError::Capture(CaptureError::PermissionDenied) => BookError::SourcePermission,
        ImportFlowError::Archive(_)
        | ImportFlowError::Capture(_)
        | ImportFlowError::Blocked(_)
        | ImportFlowError::Extraction(_)
        | ImportFlowError::References(_) => BookError::InvalidSource,
        _ => BookError::NotSaved,
    }
}

fn describe_tree(
    record: &VolumeRecord,
    position: BookPosition,
    view: &mut BookView,
) -> Result<(), BookError> {
    let mapping = &record.mapping.volume;
    view.rows = mapping.main_flow.len() + mapping.auxiliary.len();
    if view.rows == 0 {
        view.caption = "EPUB chưa có vùng chữ".into();
        return Ok(());
    }
    let row = position.row.min(view.rows - 1);
    let ranges = if let Some(region) = mapping.main_flow.get(row) {
        view.caption = match region {
            StoredMainFlow::Chapter(chapter) => {
                view.chapter = Some(chapter.chapter_id);
                let neighbor = |index| match mapping.main_flow.get(index) {
                    Some(StoredMainFlow::Chapter(chapter)) => Some(chapter.chapter_id),
                    _ => None,
                };
                view.previous_chapter = row.checked_sub(1).and_then(neighbor);
                view.next_chapter = neighbor(row + 1);
                let title = chapter
                    .label
                    .as_deref()
                    .map(|text| clean(text, 90))
                    .unwrap_or_else(|| format!("Chương {}", row + 1));
                if matches!(
                    chapter.status,
                    novel_translate::import_codec::StoredMappingStatus::Provisional
                ) {
                    format!("{title} · Ranh giới tạm")
                } else {
                    title
                }
            }
            StoredMainFlow::OutOfChapter { reason, .. } => {
                format!("Ngoài chương · {}", clean(reason, 70))
            }
            StoredMainFlow::Unclassified { reason, .. } => {
                format!("Chưa rõ ranh giới · {}", clean(reason, 70))
            }
        };
        region.ranges()
    } else {
        let extra = &mapping.auxiliary[row - mapping.main_flow.len()];
        view.caption = format!("Phần phụ · {}", clean(&extra.canonical_path, 70));
        extra.ranges.as_slice()
    };
    view.ranges = ranges.len();
    view.position.row = row;
    if !ranges.is_empty() {
        let index = position.range.min(ranges.len() - 1);
        let range = &ranges[index];
        let offset = position
            .unit
            .clamp(range.start, range.end.saturating_sub(1));
        view.position.range = index;
        view.position.unit = offset;
        view.source = Some(source_excerpt(record, range, offset)?);
    }
    Ok(())
}

fn source_excerpt(
    record: &VolumeRecord,
    range: &StoredBlockRange,
    start: u32,
) -> Result<BookSource, BookError> {
    let block = record
        .source_index
        .documents
        .iter()
        .find(|document| document.canonical_path == range.canonical_path)
        .and_then(|document| {
            document
                .blocks
                .iter()
                .find(|block| block.id == range.block_id)
        })
        .ok_or(BookError::NotSaved)?;
    let end = range.end.min(start.saturating_add(96));
    let mut text = String::new();
    let mut offset: u32 = 0;
    for run in &block.unit_runs {
        if offset >= end {
            break;
        }
        match run {
            StoredUnitRun::Text(run) => {
                let valid = run.validate().map_err(|_| BookError::NotSaved)?;
                if start < offset + valid.unit_count() {
                    for unit in valid
                        .units()
                        .skip(start.saturating_sub(offset) as usize)
                        .take((end - offset.max(start)) as usize)
                    {
                        append_clean(&mut text, unit.text, 48);
                    }
                }
                offset += valid.unit_count();
            }
            StoredUnitRun::Protected(run) => {
                if offset >= start {
                    match &run.unit {
                        StoredProtectedUnit::Ruby {
                            base, annotation, ..
                        } => {
                            text.push_str(&format!(
                                "{}〔{}〕",
                                clean(base, 40),
                                clean(annotation, 40)
                            ));
                        }
                        StoredProtectedUnit::Image {
                            resource_path, alt, ..
                        } => {
                            text.push_str(&format!(
                                "[Ảnh {}{}]",
                                clean(resource_path.as_deref().unwrap_or(""), 85),
                                alt.as_deref()
                                    .map(|value| format!(" · {}", clean(value, 30)))
                                    .unwrap_or_default()
                            ));
                        }
                        StoredProtectedUnit::MarkerPair { .. } => text.push_str("[Mốc]"),
                        StoredProtectedUnit::LineBreak { .. } => text.push(' '),
                        StoredProtectedUnit::SceneBreak { .. } => text.push_str(" · · · "),
                    }
                }
                offset += 1;
            }
        }
    }
    for inline in block
        .inline_metadata
        .iter()
        .filter(|inline| inline.ranges.iter().any(|r| r.start < end && r.end > start))
        .filter_map(|inline| match &inline.kind {
            StoredInlineKind::NoteReference { href: Some(target) } => Some(target),
            _ => None,
        })
        .take(4)
    {
        text.push_str(&format!(" [Chú thích: {}]", clean(inline, 85)));
    }
    let classification = classification_at(record, range, start, end);
    Ok(BookSource {
        start: ChapterBoundary {
            canonical_path: range.canonical_path.clone(),
            block_id: SourceBlockId::from_bytes(range.block_id.0),
            offset: range.start,
        },
        end: range.end,
        text,
        classification,
    })
}

fn classification_at(
    record: &VolumeRecord,
    range: &StoredBlockRange,
    start: u32,
    end: u32,
) -> Option<ClassificationTarget> {
    let intersects = |location: &ScopeLocation| match location {
        ScopeLocation::TextRanges { ranges } => ranges.iter().any(|current| {
            current.canonical_path == range.canonical_path
                && current.block_id == range.block_id
                && current.start < end
                && current.end > start
        }),
        _ => false,
    };
    record
        .scope
        .records
        .iter()
        .filter_map(|record| match record {
            ScopeRecord::Content(content) => Some(content),
            _ => None,
        })
        .enumerate()
        .find(|(_, content)| {
            intersects(&content.input.observation.location)
                && (content.handling == ScopeHandling::NeedsClassification
                    || matches!(content.decision, ScopeDecisionMatch::NeedsComparison { .. }))
        })
        .map(|(index, _)| ClassificationTarget::Content(index))
        .or_else(|| {
            record
                .scope
                .notes
                .iter()
                .position(|note| {
                    intersects(&note.content.input.observation.location)
                        && (note.content.handling == ScopeHandling::NeedsClassification
                            || matches!(
                                note.content.decision,
                                ScopeDecisionMatch::NeedsComparison { .. }
                            ))
                })
                .map(ClassificationTarget::Note)
        })
}

fn clean(text: &str, characters: usize) -> String {
    let mut result = String::new();
    append_clean(&mut result, text, characters);
    result
}

fn append_clean(result: &mut String, text: &str, characters: usize) {
    result.extend(
        text.chars()
            .filter(|character| !character.is_control())
            .take(characters),
    );
}
