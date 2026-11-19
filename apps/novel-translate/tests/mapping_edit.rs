#![cfg(windows)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use novel_translate::import_workflow::{
    Classification, ImportFlowError, ImportTarget, MappingEditError, PreparedImport, Publication,
    SavedVolume, ScopeChange, prepare_import, prepare_mapping_confirmation, prepare_scope_change,
    preview_mapping_edit, read_saved_volume, read_volume_scope,
};
use nt_application::volume_import::{
    GenerationCommitError, GenerationId, ImportWriteError, OperationId, VolumeId,
    chapter_map::{
        BlockUnitRange, Chapter, ChapterMap, MainFlowContent,
        edit::{ChapterBoundary, ChapterEdit, changed_chapters},
    },
    source_scope::{ContentKind, DecisionKey, DecisionMatch, ScopeMode},
};
use nt_application::workspace::create_workspace;
use nt_storage::SqliteWorkspaceRepository;

#[path = "support/epub.rs"]
mod epub;

struct TestWorkspace(PathBuf);

impl TestWorkspace {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let suffix = NEXT.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("nt-mapping-edit-{}-{suffix}", std::process::id()));
        fs::create_dir(&path).expect("isolated mapping test root");
        Self(path)
    }

    fn volume(&self) -> (PathBuf, ImportTarget) {
        let path = self.0.join("Series 日本語");
        let series = create_workspace(&SqliteWorkspaceRepository, &path)
            .unwrap()
            .series
            .id;
        let volume = VolumeId::parse("6".repeat(32)).unwrap();
        let connection = rusqlite::Connection::open(path.join("series.sqlite3")).unwrap();
        connection
            .execute(
                "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms)
             VALUES (?1, ?2, 'Mapping fixture', 0)",
                (series.as_str(), volume.as_str()),
            )
            .unwrap();
        let target = ImportTarget {
            series,
            volume,
            expected_parent: GenerationId::parse("g0").unwrap(),
            generation: GenerationId::parse("7".repeat(32)).unwrap(),
            operation: OperationId::parse("8".repeat(32)).unwrap(),
        };
        (path, target)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn generations(path: &Path) -> i64 {
    rusqlite::Connection::open(path.join("series.sqlite3"))
        .unwrap()
        .query_row("SELECT count(*) FROM generations", [], |row| row.get(0))
        .unwrap()
}

#[test]
fn preview_pins_a_real_import_and_leaves_current_and_history_untouched() {
    let fixture = TestWorkspace::new();
    let (path, target) = fixture.volume();
    let source = fixture.0.join("original.epub");
    fs::write(&source, epub::book(3, epub::CHAPTER)).unwrap();
    let series = target.series.clone();
    let volume = target.volume.clone();
    let generation = target.generation.clone();
    prepare_import(&path, &source, target)
        .unwrap()
        .submit()
        .unwrap();
    let before = read_saved_volume(&path, &series, &volume, None)
        .unwrap()
        .record
        .unwrap();
    let (map, _) = before.mapping.clone().into_map().unwrap();
    let old = map
        .volume
        .main_flow
        .iter()
        .find_map(|item| {
            if let MainFlowContent::Chapter(chapter) = item {
                Some(chapter)
            } else {
                None
            }
        })
        .expect("fixture chapter");
    let count = generations(&path);

    let command = ChapterEdit::Rename {
        chapter: old.chapter_id,
        label: "序章".to_owned(),
    };
    let draft =
        preview_mapping_edit(&path, &series, &volume, &generation, command.clone()).unwrap();
    assert_eq!(draft.workspace(), path);
    assert_eq!(draft.edit(), &command);
    assert_eq!(draft.version().series, series);
    assert_eq!(draft.version().volume, volume);
    assert_eq!(draft.version().generation, generation);
    assert_eq!(draft.version().epub_sha256, before.source.epub_sha256.0);
    assert_eq!(draft.version().parser_version, before.source.parser_version);
    assert_eq!(
        draft.version().mapping_version,
        before.source.mapping_version.0
    );
    let preview = draft.preview();
    assert_eq!(preview.before.len(), 1);
    assert_eq!(preview.after.len(), 1);
    let original = &preview.before[0];
    let updated = &preview.after[0];
    assert_eq!(updated.chapter_id, old.chapter_id);
    assert_eq!(updated.label.as_deref(), Some("序章"));
    assert_eq!(updated.start, original.start);
    assert_eq!(updated.end, original.end);
    assert_eq!(updated.source_range_count, old.ranges.len());
    assert_eq!(preview.impact.chapters, vec![old.chapter_id]);

    let current = read_saved_volume(&path, &series, &volume, None).unwrap();
    assert_eq!(current.generation, generation);
    assert!(current.record.unwrap() == before);
    assert_eq!(generations(&path), count);
}

#[test]
fn preview_rejects_an_empty_registered_volume_without_writing() {
    let fixture = TestWorkspace::new();
    let (path, target) = fixture.volume();
    assert!(matches!(
        preview_mapping_edit(
            &path,
            &target.series,
            &target.volume,
            &target.expected_parent,
            ChapterEdit::Merge {
                first: 1,
                second: 2
            }
        ),
        Err(MappingEditError::MissingImport)
    ));
    assert_eq!(generations(&path), 1);
}

#[test]
fn preview_stays_on_the_displayed_generation_when_current_has_advanced() {
    let fixture = TestWorkspace::new();
    let (path, target) = fixture.volume();
    let source = fixture.0.join("original.epub");
    fs::write(&source, epub::book(3, epub::CHAPTER)).unwrap();
    let series = target.series.clone();
    let volume = target.volume.clone();
    let displayed = target.generation.clone();
    prepare_import(&path, &source, target)
        .unwrap()
        .submit()
        .unwrap();
    let saved = read_saved_volume(&path, &series, &volume, Some(&displayed)).unwrap();
    let old = saved.record.unwrap();
    let (map, _) = old.mapping.into_map().unwrap();
    let chapter = map
        .volume
        .main_flow
        .iter()
        .find_map(|item| match item {
            MainFlowContent::Chapter(chapter) => Some(chapter.chapter_id),
            _ => None,
        })
        .expect("fixture chapter");

    let newer = GenerationId::parse("a".repeat(32)).unwrap();
    prepare_scope_change(
        &path,
        ImportTarget {
            series: series.clone(),
            volume: volume.clone(),
            expected_parent: displayed.clone(),
            generation: newer.clone(),
            operation: OperationId::parse("b".repeat(32)).unwrap(),
        },
        ScopeChange {
            mode: ScopeMode::Story,
            classifications: Vec::new(),
            save_as_series_default: false,
        },
    )
    .unwrap()
    .submit()
    .unwrap();

    let draft = preview_mapping_edit(
        &path,
        &series,
        &volume,
        &displayed,
        ChapterEdit::Rename {
            chapter,
            label: "Old review".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(draft.version().generation, displayed);
    assert_eq!(
        draft.version().mapping_version,
        old.source.mapping_version.0
    );
    let current = read_saved_volume(&path, &series, &volume, None).unwrap();
    assert_eq!(current.generation, newer);
    assert_eq!(current.record.unwrap().scope.mode(), ScopeMode::Story);
    assert_eq!(generations(&path), 3);
}

fn chapter_with_interior(mapping: &ChapterMap) -> (&Chapter, &BlockUnitRange) {
    mapping
        .volume
        .main_flow
        .iter()
        .find_map(|item| match item {
            MainFlowContent::Chapter(chapter) => chapter
                .ranges
                .iter()
                .find(|range| range.end - range.start >= 3)
                .map(|range| (chapter, range)),
            _ => None,
        })
        .expect("fixture has a chapter with room to split and move")
}

fn next_generation(seed: u8) -> (OperationId, GenerationId) {
    (
        OperationId::parse(format!("{:032x}", u16::from(seed) + 256)).unwrap(),
        GenerationId::parse(format!("{seed:032x}")).unwrap(),
    )
}

fn published(prepared: &PreparedImport) -> Box<SavedVolume> {
    let Publication::Published { saved, .. } = prepared.submit().unwrap() else {
        panic!("expected an acknowledged publication");
    };
    saved
}

#[test]
fn confirmation_saves_four_chapter_edits_and_reopens_exact_historical_maps() {
    let fixture = TestWorkspace::new();
    let (path, target) = fixture.volume();
    let source = fixture.0.join("original.epub");
    let bytes = epub::book(3, epub::CHAPTER);
    fs::write(&source, &bytes).unwrap();
    let series = target.series.clone();
    let volume = target.volume.clone();
    let original = published(&prepare_import(&path, &source, target).unwrap());
    let original_generation = original.generation.clone();
    let original = original.record.unwrap();
    let original_objects = original.resources.clone();
    let (map, _) = original.mapping.clone().into_map().unwrap();
    let (chapter, range) = chapter_with_interior(&map);
    let first = chapter.chapter_id;
    let start = ChapterBoundary {
        canonical_path: range.canonical_path.clone(),
        block_id: range.block_id,
        offset: range.start + 1,
    };

    // A mapping edit opens the immutable imported object after the selected
    // file has gone away.
    fs::remove_file(&source).unwrap();
    let split = preview_mapping_edit(
        &path,
        &series,
        &volume,
        &original_generation,
        ChapterEdit::Split {
            chapter: first,
            boundary: start.clone(),
            label: Some("後編".to_owned()),
        },
    )
    .unwrap();
    let second = split.preview().impact.chapters[1];
    let (operation, generation) = next_generation(2);
    let split_saved =
        published(&prepare_mapping_confirmation(&split, operation, generation).unwrap());
    let split_generation = split_saved.generation.clone();
    let split_record = split_saved.record.unwrap();
    assert_ne!(
        split_record.source.mapping_version,
        original.source.mapping_version
    );
    assert_eq!(
        split_record.source.original_object_sha256,
        original.source.original_object_sha256
    );

    let moved = preview_mapping_edit(
        &path,
        &series,
        &volume,
        &split_generation,
        ChapterEdit::MoveBoundary {
            first,
            second,
            boundary: ChapterBoundary {
                offset: start.offset + 1,
                ..start.clone()
            },
        },
    )
    .unwrap();
    assert_eq!(moved.preview().impact.previous_boundary, Some(start));
    let (operation, generation) = next_generation(3);
    let moved_saved =
        published(&prepare_mapping_confirmation(&moved, operation, generation).unwrap());
    let moved_record = moved_saved.record.unwrap();
    assert_ne!(
        moved_record.source.mapping_version,
        split_record.source.mapping_version
    );

    let merged = preview_mapping_edit(
        &path,
        &series,
        &volume,
        &moved_saved.generation,
        ChapterEdit::Merge { first, second },
    )
    .unwrap();
    let (operation, generation) = next_generation(4);
    let merged_saved =
        published(&prepare_mapping_confirmation(&merged, operation, generation).unwrap());
    assert_eq!(merged.preview().after.len(), 1);

    let renamed = preview_mapping_edit(
        &path,
        &series,
        &volume,
        &merged_saved.generation,
        ChapterEdit::Rename {
            chapter: first,
            label: "題名".to_owned(),
        },
    )
    .unwrap();
    let (operation, generation) = next_generation(5);
    let renamed_saved =
        published(&prepare_mapping_confirmation(&renamed, operation, generation).unwrap());
    let renamed_record = renamed_saved.record.unwrap();
    assert_eq!(renamed_record.resources, original_objects);
    assert_eq!(
        renamed_record.source.epub_sha256,
        original.source.epub_sha256
    );
    assert_eq!(renamed_record.source_index, original.source_index);

    let historical =
        read_saved_volume(&path, &series, &volume, Some(&original_generation)).unwrap();
    assert!(historical.record.unwrap() == original);
    let historical = read_saved_volume(&path, &series, &volume, Some(&split_generation)).unwrap();
    assert!(historical.record.unwrap() == split_record);
    let (before_move, _) = split_record.mapping.into_map().unwrap();
    let (after_move, _) = moved_record.mapping.into_map().unwrap();
    let changed = changed_chapters(&before_move, &after_move).unwrap();
    assert_eq!(
        changed
            .iter()
            .map(|chapter| chapter.chapter_id)
            .collect::<Vec<_>>(),
        [first, second]
    );
    let right = &changed[1];
    let previous = right.before.unwrap();
    let current = right.after.unwrap();
    assert_eq!(current.ranges[0].start, previous.ranges[0].start + 1);
    assert_eq!(current.ranges[0].block_id, previous.ranges[0].block_id);
    assert_eq!(
        current.start_evidence[0].unit_offset,
        Some(current.ranges[0].start)
    );

    fs::write(&source, bytes).unwrap();
    let (operation, generation) = next_generation(6);
    let reimport = prepare_import(
        &path,
        &source,
        ImportTarget {
            series,
            volume,
            expected_parent: renamed_saved.generation,
            operation,
            generation,
        },
    )
    .unwrap();
    let reread = published(&reimport).record.unwrap();
    assert_eq!(reread.mapping, renamed_record.mapping);
    assert_eq!(
        reread.source.mapping_version,
        renamed_record.source.mapping_version
    );
    assert_eq!(generations(&path), 7);
}

#[test]
fn concurrent_confirmations_keep_the_second_draft_and_publish_only_the_first() {
    let fixture = TestWorkspace::new();
    let (path, target) = fixture.volume();
    let source = fixture.0.join("original.epub");
    fs::write(&source, epub::book(3, epub::CHAPTER)).unwrap();
    let series = target.series.clone();
    let volume = target.volume.clone();
    let first = published(&prepare_import(&path, &source, target).unwrap());
    let (mapping, _) = first.record.unwrap().mapping.into_map().unwrap();
    let (chapter, _) = chapter_with_interior(&mapping);
    let make_draft = |label: &str| {
        preview_mapping_edit(
            &path,
            &series,
            &volume,
            &first.generation,
            ChapterEdit::Rename {
                chapter: chapter.chapter_id,
                label: label.to_owned(),
            },
        )
        .unwrap()
    };
    let accepted = make_draft("First draft");
    let pending = make_draft("Second draft");
    let (first_op, first_gen) = next_generation(2);
    let (second_op, second_gen) = next_generation(3);
    let prepared_a = prepare_mapping_confirmation(&accepted, first_op, first_gen.clone()).unwrap();
    let prepared_b = prepare_mapping_confirmation(&pending, second_op, second_gen.clone()).unwrap();
    published(&prepared_a);

    let Err(error) = prepared_b.submit() else {
        panic!("second commit should be stale")
    };
    assert!(matches!(
        error,
        ImportFlowError::Storage(ImportWriteError::Generation(
            GenerationCommitError::StaleGeneration
        ))
    ));
    let (retry_op, _) = next_generation(4);
    let retry = prepare_mapping_confirmation(&pending, retry_op, second_gen.clone());
    assert!(matches!(
        retry,
        Err(ImportFlowError::Storage(ImportWriteError::Generation(
            GenerationCommitError::StaleGeneration
        )))
    ));
    assert_eq!(
        pending.preview().after[0].label.as_deref(),
        Some("Second draft")
    );
    assert_eq!(
        read_saved_volume(&path, &series, &volume, None)
            .unwrap()
            .generation,
        first_gen
    );
    assert!(read_saved_volume(&path, &series, &volume, Some(&second_gen)).is_err());
    assert_eq!(generations(&path), 3);
}

#[test]
fn changed_mapping_marks_the_old_classification_for_comparison() {
    let fixture = TestWorkspace::new();
    let (path, target) = fixture.volume();
    let source = fixture.0.join("original.epub");
    fs::write(&source, epub::book(3, epub::CHAPTER)).unwrap();
    let series = target.series.clone();
    let volume = target.volume.clone();
    let imported = published(&prepare_import(&path, &source, target).unwrap());
    let (map, _) = imported.record.unwrap().mapping.into_map().unwrap();
    let (chapter, range) = chapter_with_interior(&map);

    let scope = read_volume_scope(&path, &series, &volume, Some(&imported.generation))
        .unwrap()
        .unwrap();
    let location = scope
        .content
        .iter()
        .find(|item| item.input.observation.excerpt.starts_with('A'))
        .unwrap()
        .input
        .observation
        .location
        .clone();
    let key = DecisionKey {
        context: scope.context,
        location: location.clone(),
    };
    let (operation, generation) = next_generation(2);
    let classified = published(
        &prepare_scope_change(
            &path,
            ImportTarget {
                series: series.clone(),
                volume: volume.clone(),
                expected_parent: imported.generation,
                operation,
                generation,
            },
            ScopeChange {
                mode: ScopeMode::Story,
                classifications: vec![Classification {
                    key: key.clone(),
                    kind: ContentKind::OtherStory,
                }],
                save_as_series_default: false,
            },
        )
        .unwrap(),
    );

    let draft = preview_mapping_edit(
        &path,
        &series,
        &volume,
        &classified.generation,
        ChapterEdit::Split {
            chapter: chapter.chapter_id,
            boundary: ChapterBoundary {
                canonical_path: range.canonical_path.clone(),
                block_id: range.block_id,
                offset: range.start + 1,
            },
            label: Some("Reviewed part".to_owned()),
        },
    )
    .unwrap();
    let (operation, generation) = next_generation(3);
    let saved = published(&prepare_mapping_confirmation(&draft, operation, generation).unwrap());
    let before = read_volume_scope(&path, &series, &volume, Some(&classified.generation))
        .unwrap()
        .unwrap();
    let after = read_volume_scope(&path, &series, &volume, Some(&saved.generation))
        .unwrap()
        .unwrap();

    assert_eq!(before.context.epub_sha256, after.context.epub_sha256);
    assert_ne!(
        before.context.mapping_version,
        after.context.mapping_version
    );
    assert_eq!(before.mode, after.mode);
    assert!(before.content.iter().any(|item| {
        item.input.observation.location == location
            && matches!(item.decision, DecisionMatch::Reused(_))
    }));
    assert!(after.content.iter().any(|item| {
        matches!(&item.decision, DecisionMatch::NeedsComparison(items)
            if items.iter().any(|decision| decision.key == key))
    }));
}
