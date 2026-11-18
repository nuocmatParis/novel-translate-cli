use super::TempDir;
use novel_translate::import_codec::{
    Digest, ResourceFiles, ScopeHandling, SnapshotDraft, SnapshotInputs, StagedVolume,
};
use novel_translate::import_workflow::{
    Classification, ImportFlowError, ImportReadiness, ImportTarget, Publication, ScopeChange,
    capture_epub, prepare_import, prepare_scope_change, prepare_staged_import, read_saved_volume,
    read_volume_scope,
};
use nt_application::chapter_map::build_candidate_map;
use nt_application::source_scope::{ScopeInput, evaluate_scope, observe_map_text};
use nt_application::volume_import::{
    GenerationCommitError, GenerationId, ImportOperation, ImportWriteError, OperationId, SeriesId,
    VolumeId, VolumeImportRepository, VolumeImportWriteRequest,
    source_scope::{ContentKind, DecisionKey, DecisionMatch, Handling, ScopeMode},
};
use nt_application::workspace::create_workspace;
use nt_epub::{
    EpubZip, PARSER_VERSION, ReferenceIndexLimits, extract_source, index_references, inspect,
};
use nt_storage::{
    FileImportSourceStore, FileObjectStore, SqliteGenerationWriter, SqliteWorkspaceRepository,
};
use std::{fs, path::Path};

#[path = "epub.rs"]
mod epub;

fn fresh_volume(temp: &TempDir) -> (std::path::PathBuf, ImportTarget) {
    let workspace = temp.path().join("series");
    let series = create_workspace(&SqliteWorkspaceRepository, &workspace)
        .unwrap()
        .series
        .id;
    let volume = VolumeId::parse("8".repeat(32)).unwrap();
    let db = rusqlite::Connection::open(workspace.join("series.sqlite3")).unwrap();
    db.execute(
        "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms)
        VALUES (?1, ?2, 'Fixture', 0)",
        (series.as_str(), volume.as_str()),
    )
    .unwrap();
    let target = ImportTarget {
        series,
        volume,
        expected_parent: GenerationId::parse("g0").unwrap(),
        operation: OperationId::parse("9".repeat(32)).unwrap(),
        generation: GenerationId::parse("9".repeat(32)).unwrap(),
    };
    (workspace, target)
}

fn edit_book(bytes: &[u8], edit: impl FnOnce(&mut Vec<(String, Vec<u8>)>)) -> Vec<u8> {
    let mut archive = EpubZip::new(std::io::Cursor::new(bytes)).unwrap();
    let entries: Vec<_> = archive
        .entries()
        .iter()
        .map(|entry| (entry.name().to_owned(), entry.id()))
        .collect();
    let mut entries: Vec<_> = entries
        .into_iter()
        .map(|(name, id)| (name, archive.read_entry(id).unwrap()))
        .collect();
    edit(&mut entries);
    let borrowed: Vec<_> = entries
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect();
    epub::stored_zip(&borrowed)
}

fn replace_book_entry(bytes: &[u8], path: &str, replacement: &[u8]) -> Vec<u8> {
    edit_book(bytes, |entries| {
        let (_, data) = entries.iter_mut().find(|(name, _)| name == path).unwrap();
        *data = replacement.to_vec();
    })
}

#[test]
fn selected_file_navigation_failure_is_optional_only_outside_the_spine() {
    for version in [2, 3] {
        let temp = TempDir::new();
        let (workspace, target) = fresh_volume(&temp);
        let nav_path = if version == 3 {
            "OPS/nav.xhtml"
        } else {
            "OPS/toc.ncx"
        };
        let bytes = replace_book_entry(&epub::book(version, epub::CHAPTER), nav_path, b"<broken");
        let source = temp.path().join("navigation.epub");
        fs::write(&source, &bytes).unwrap();
        let prepared = prepare_import(&workspace, &source, target).unwrap();
        let Publication::Published { saved, .. } = prepared.submit().unwrap() else {
            panic!("optional navigation must publish with evidence");
        };
        assert!(
            matches!(saved.readiness(), Some(ImportReadiness::NeedsReview { issues }) if !issues.is_empty())
        );
        assert!(!saved.record.unwrap().diagnostics.navigation.is_empty());
    }

    for navigation in [b"<broken".as_slice(),
        br#"<!DOCTYPE html [<!ENTITY bad SYSTEM "https://invalid.invalid/entity">]><html xmlns="http://www.w3.org/1999/xhtml"><body>&bad;</body></html>"#,
    ] {
        let temp = TempDir::new();
        let (workspace, target) = fresh_volume(&temp);
        let bytes = epub::book(3, epub::CHAPTER);
        let mut archive = EpubZip::new(std::io::Cursor::new(&bytes)).unwrap();
        let package = archive.entry_by_path("OPS/book.opf").unwrap().unwrap().id();
        let package = String::from_utf8(archive.read_entry(package).unwrap()).unwrap();
        let package = package.replace("</spine>", r#"<itemref idref="nav"/></spine>"#);
        let bytes = replace_book_entry(&bytes, "OPS/book.opf", package.as_bytes());
        let bytes = replace_book_entry(&bytes, "OPS/nav.xhtml", navigation);
        let source = temp.path().join("required-navigation.epub");
        fs::write(&source, bytes).unwrap();
        assert!(prepare_import(&workspace, &source, ImportTarget {
            series: target.series.clone(), volume: target.volume.clone(), ..target
        }).is_err());
        assert_eq!(read_saved_volume(&workspace, &target.series, &target.volume, None).unwrap().generation,
            GenerationId::parse("g0").unwrap());
    }
}

#[test]
fn selected_file_missing_required_image_keeps_the_previous_generation() {
    let temp = TempDir::new();
    let (workspace, target) = fresh_volume(&temp);
    let series = target.series.clone();
    let volume = target.volume.clone();
    let chapter = epub::CHAPTER.replace("../Images/a.png", "../Images/missing.png");
    let source = temp.path().join("missing-image.epub");
    fs::write(&source, epub::book(3, &chapter)).unwrap();
    let result = prepare_import(&workspace, &source, target);
    assert!(matches!(result, Err(ImportFlowError::Blocked(_))));
    assert_eq!(
        read_saved_volume(&workspace, &series, &volume, None)
            .unwrap()
            .generation,
        GenerationId::parse("g0").unwrap()
    );
}

#[test]
fn saved_svg_illustration_uses_its_plain_href_and_verified_image_bytes() {
    use nt_application::volume_import::ObjectHash;

    let temp = TempDir::new();
    let (workspace, target) = fresh_volume(&temp);
    let series = target.series.clone();
    let volume = target.volume.clone();
    let generation = target.generation.clone();
    let source = temp.path().join("illustration.epub");
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><head><title>Story</title></head>
        <body><h1 id="title">Illustration</h1><p>Story
        <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
        <image xlink:href="../Images/legacy.svg" href="../Images/a.png" width="1" height="1"/>
        </svg></p></body></html>"#;
    let book = edit_book(&epub::book(3, chapter), |entries| {
        let (_, package) = entries
            .iter_mut()
            .find(|(path, _)| path == "OPS/book.opf")
            .unwrap();
        let xml = std::str::from_utf8(package).unwrap()
            .replace("id=\"ch\"", "id=\"ch\" properties=\"svg\"")
            .replace("</manifest>", r#"<item id="legacy" href="Images/legacy.svg" media-type="image/svg+xml"/></manifest>"#);
        *package = xml.into_bytes();
        entries.push(("OPS/Images/legacy.svg".into(),
            br#"<svg xmlns="http://www.w3.org/2000/svg" width="2" height="2"><rect width="2" height="2"/></svg>"#.to_vec()));
    });
    fs::write(&source, book).unwrap();
    assert!(matches!(
        prepare_import(&workspace, &source, target)
            .unwrap()
            .submit()
            .unwrap(),
        Publication::Published { .. }
    ));
    let saved = read_saved_volume(&workspace, &series, &volume, Some(&generation)).unwrap();
    let record = saved.record.unwrap();
    let image = record
        .resources
        .iter()
        .find(|r| r.canonical_path == "OPS/Images/a.png")
        .unwrap();
    let expected =
        ObjectHash::parse("6af484b8a73288a07fec730818a6de8bebce88aa915b52f66a8769bfb865f434")
            .unwrap();
    assert_eq!(image.object_sha256.0, *expected.as_bytes());
    assert!(
        image
            .uses
            .iter()
            .any(|usage| usage.source_path == "OPS/Text/ch.xhtml"
                && usage.raw_reference == "../Images/a.png"
                && !usage.source_occurrences.is_empty())
    );
    let fallback = record
        .resources
        .iter()
        .find(|r| r.canonical_path == "OPS/Images/legacy.svg")
        .unwrap();
    assert!(
        fallback
            .uses
            .iter()
            .all(|usage| usage.source_path != "OPS/Text/ch.xhtml")
    );
}

#[test]
fn only_unqualified_roles_classify_notes_in_saved_story_scope() {
    let temp = TempDir::new();
    let (workspace, initial) = fresh_volume(&temp);
    let series = initial.series.clone();
    let volume = initial.volume.clone();
    let imported = initial.generation.clone();
    let source = temp.path().join("roles.epub");
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"
        xmlns:epub="http://www.idpf.org/2007/ops" xmlns:f="urn:example:annotations">
        <head><title>Chapter</title></head><body epub:type="chapter"><h1 id="title">Chapter</h1>
        <p f:role="doc-footnote">A narrative sentence.</p>
        <p f:role="doc-footnote" role="doc-chapter">Foreign attribute first.</p>
        <p role="doc-chapter" f:role="doc-footnote">Plain attribute first.</p>
        <p role="doc-footnote">An actual note.</p>
        <p f:role="doc-chapter" role="doc-footnote">Another actual note.</p>
        </body></html>"#;
    fs::write(&source, epub::book(3, chapter)).unwrap();
    assert!(matches!(
        prepare_import(&workspace, &source, initial)
            .unwrap()
            .submit()
            .unwrap(),
        Publication::Published { .. }
    ));
    let edit = ImportTarget {
        series: series.clone(),
        volume: volume.clone(),
        expected_parent: imported.clone(),
        operation: OperationId::parse("a".repeat(32)).unwrap(),
        generation: GenerationId::parse("a".repeat(32)).unwrap(),
    };
    let change = ScopeChange {
        mode: ScopeMode::Story,
        classifications: Vec::new(),
        save_as_series_default: false,
    };
    assert!(matches!(
        prepare_scope_change(&workspace, edit, change)
            .unwrap()
            .submit()
            .unwrap(),
        Publication::Published { .. }
    ));

    for (generation, mode, note_handling) in [
        (None, ScopeMode::Story, Handling::PreserveSource),
        (
            Some(&imported),
            ScopeMode::AllText,
            Handling::TranslateAndReview,
        ),
    ] {
        let scope = read_volume_scope(&workspace, &series, &volume, generation)
            .unwrap()
            .unwrap();
        assert_eq!(scope.mode, mode);
        for text in [
            "A narrative sentence.",
            "Foreign attribute first.",
            "Plain attribute first.",
        ] {
            assert_eq!(
                scope
                    .content
                    .iter()
                    .find(|item| item.input.observation.excerpt == text)
                    .map(|item| item.handling),
                Some(Handling::TranslateAndReview),
                "{text} in {mode:?}"
            );
        }
        for text in ["An actual note.", "Another actual note."] {
            assert_eq!(
                scope
                    .notes
                    .iter()
                    .find(|note| note.content.input.observation.excerpt == text)
                    .map(|note| note.content.handling),
                Some(note_handling),
                "{text} in {mode:?}"
            );
        }
    }
}

#[test]
fn classification_reopens_reuses_exact_context_and_compares_changed_source() {
    let temp = TempDir::new();
    let (workspace, initial) = fresh_volume(&temp);
    let series = initial.series.clone();
    let volume = initial.volume.clone();
    let imported = initial.generation.clone();
    let source = temp.path().join("scope.epub");
    fs::write(&source, epub::book(3, epub::CHAPTER)).unwrap();
    assert!(matches!(
        prepare_import(&workspace, &source, initial)
            .unwrap()
            .submit()
            .unwrap(),
        Publication::Published { .. }
    ));
    let target = |parent: &GenerationId, number: u128| ImportTarget {
        series: series.clone(),
        volume: volume.clone(),
        expected_parent: parent.clone(),
        operation: OperationId::parse(format!("{number:032x}")).unwrap(),
        generation: GenerationId::parse(format!("{number:032x}")).unwrap(),
    };
    let scope = read_volume_scope(&workspace, &series, &volume, None)
        .unwrap()
        .unwrap();
    let paragraph = scope
        .content
        .iter()
        .find(|item| item.input.observation.excerpt.starts_with('A'))
        .unwrap();
    let location = paragraph.input.observation.location.clone();
    let key = DecisionKey {
        context: scope.context.clone(),
        location: location.clone(),
    };
    let mut wrong = key.clone();
    wrong.context.epub_sha256[0] ^= 1;
    assert!(matches!(
        prepare_scope_change(
            &workspace,
            target(&imported, 10),
            ScopeChange {
                mode: ScopeMode::Story,
                classifications: vec![Classification {
                    key: wrong,
                    kind: ContentKind::OtherStory
                }],
                save_as_series_default: false,
            }
        ),
        Err(ImportFlowError::InvalidSnapshot)
    ));
    let request = prepare_scope_change(
        &workspace,
        target(&imported, 10),
        ScopeChange {
            mode: ScopeMode::Story,
            classifications: vec![Classification {
                key,
                kind: ContentKind::OtherStory,
            }],
            save_as_series_default: false,
        },
    )
    .unwrap();
    let Publication::Published {
        saved,
        replayed: false,
    } = request.submit().unwrap()
    else {
        panic!("scope edit must publish and reopen");
    };
    let classified = saved.generation.clone();
    assert_eq!(saved.version.state.mode(), ScopeMode::Story);
    assert_eq!(
        saved.record.as_ref().unwrap().source.epub_sha256.0,
        scope.context.epub_sha256
    );
    assert_eq!(
        saved.record.as_ref().unwrap().source.mapping_version.0,
        scope.context.mapping_version
    );
    let restored = read_volume_scope(&workspace, &series, &volume, None)
        .unwrap()
        .unwrap();
    let paragraph = restored
        .content
        .iter()
        .find(|item| item.input.observation.location == location)
        .unwrap();
    assert!(matches!(paragraph.decision, DecisionMatch::Reused(_)));
    assert_eq!(paragraph.handling, Handling::TranslateAndReview);
    assert_eq!(restored.notes.len(), 1);
    assert_eq!(
        restored.notes[0].content.handling,
        Handling::TranslateAndReview
    );
    assert!(matches!(
        prepare_scope_change(
            &workspace,
            target(&imported, 11),
            ScopeChange {
                mode: ScopeMode::AllText,
                classifications: Vec::new(),
                save_as_series_default: false,
            }
        ),
        Err(ImportFlowError::Storage(ImportWriteError::Generation(
            GenerationCommitError::StaleGeneration
        )))
    ));

    let Publication::Published { saved, .. } =
        prepare_import(&workspace, &source, target(&classified, 12))
            .unwrap()
            .submit()
            .unwrap()
    else {
        panic!("identical reimport must reopen");
    };
    let identical = saved.generation.clone();
    let same = read_volume_scope(&workspace, &series, &volume, None)
        .unwrap()
        .unwrap();
    assert_eq!(same.context, restored.context);
    assert!(
        same.content
            .iter()
            .any(|item| item.input.observation.location == location
                && matches!(item.decision, DecisionMatch::Reused(_)))
    );

    fs::write(&source, epub::book(3, &epub::CHAPTER.replace("/>Z", "/>Q"))).unwrap();
    assert!(matches!(
        prepare_import(&workspace, &source, target(&identical, 13))
            .unwrap()
            .submit()
            .unwrap(),
        Publication::Published { .. }
    ));
    let changed = read_volume_scope(&workspace, &series, &volume, None)
        .unwrap()
        .unwrap();
    assert_ne!(changed.context.epub_sha256, same.context.epub_sha256);
    let paragraph = changed
        .content
        .iter()
        .find(|item| item.input.observation.excerpt.starts_with('A'))
        .unwrap();
    assert!(
        matches!(&paragraph.decision, DecisionMatch::NeedsComparison(items) if items.len() == 1)
    );
    assert_eq!(paragraph.handling, Handling::NeedsClassification);
    let historical = read_volume_scope(&workspace, &series, &volume, Some(&classified))
        .unwrap()
        .unwrap();
    assert_eq!(historical, restored);
    assert!(
        matches!(request.reconcile().unwrap(), Publication::Published { saved, replayed: true }
        if saved.generation == classified)
    );
}

#[test]
fn volume_mode_and_explicit_series_default_remain_separate_after_reopen() {
    let temp = TempDir::new();
    let (workspace, target) = fresh_volume(&temp);
    let series = target.series.clone();
    let first = target.volume.clone();
    let second = VolumeId::parse("7".repeat(32)).unwrap();
    rusqlite::Connection::open(workspace.join("series.sqlite3"))
        .unwrap()
        .execute(
            "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms)
         VALUES (?1, ?2, 'Second', 0)",
            (series.as_str(), second.as_str()),
        )
        .unwrap();
    let Publication::Published { saved, .. } = prepare_scope_change(
        &workspace,
        target,
        ScopeChange {
            mode: ScopeMode::Story,
            classifications: Vec::new(),
            save_as_series_default: false,
        },
    )
    .unwrap()
    .submit()
    .unwrap() else {
        panic!("unimported scope change must reopen");
    };
    assert_eq!(saved.version.state.mode(), ScopeMode::Story);
    assert_eq!(
        read_saved_volume(&workspace, &series, &second, None)
            .unwrap()
            .version
            .state
            .mode(),
        ScopeMode::AllText
    );
    let default = || {
        let mut reader = nt_storage::SqliteVolumeImportReader::open(&workspace).unwrap();
        let snapshot = reader.begin_snapshot().unwrap();
        let mode = snapshot.series_default().unwrap();
        snapshot.finish().unwrap();
        mode
    };
    assert_eq!(default(), ScopeMode::AllText);
    let Publication::Published { saved, .. } = prepare_scope_change(
        &workspace,
        ImportTarget {
            series: series.clone(),
            volume: first.clone(),
            expected_parent: saved.generation,
            operation: OperationId::parse("a".repeat(32)).unwrap(),
            generation: GenerationId::parse("a".repeat(32)).unwrap(),
        },
        ScopeChange {
            mode: ScopeMode::Story,
            classifications: Vec::new(),
            save_as_series_default: true,
        },
    )
    .unwrap()
    .submit()
    .unwrap() else {
        panic!("explicit default save must reopen");
    };
    assert_eq!(default(), ScopeMode::Story);
    assert_eq!(
        read_saved_volume(&workspace, &series, &second, None)
            .unwrap()
            .version
            .state
            .mode(),
        ScopeMode::AllText
    );
    let inherited =
        nt_application::volume_import::source_scope::SeriesScope::from_saved(series, default())
            .capture_for_new_volume(VolumeId::parse("6".repeat(32)).unwrap());
    assert_eq!(inherited.mode(), ScopeMode::Story);
    assert_eq!(saved.version.state.mode(), ScopeMode::Story);
}

#[test]
fn source_proof_rejects_forged_or_omitted_scope_data_before_publication() {
    let temp = TempDir::new();
    let (workspace, target) = fresh_volume(&temp);
    let series = target.series.clone();
    let volume = target.volume.clone();
    let source = temp.path().join("scope-tamper.epub");
    fs::write(&source, epub::book(3, epub::CHAPTER)).unwrap();
    let Publication::Published { saved, .. } = prepare_import(&workspace, &source, target)
        .unwrap()
        .submit()
        .unwrap()
    else {
        panic!("valid source must publish");
    };
    let parent = saved.generation.clone();
    let record = saved.record.unwrap();
    let objects = FileObjectStore::new(&workspace).unwrap();
    for change in 0..3 {
        let mut forged = record.clone();
        match change {
            0 => {
                let novel_translate::import_codec::ScopeRecord::Content(item) = forged
                    .scope
                    .records
                    .iter_mut()
                    .find(|item| {
                        matches!(item, novel_translate::import_codec::ScopeRecord::Content(_))
                    })
                    .unwrap()
                else {
                    unreachable!()
                };
                item.input.observation.excerpt.push_str(" forged text");
            }
            1 => {
                forged.scope.records.retain(|item| !matches!(item,
                    novel_translate::import_codec::ScopeRecord::Related(related) if related.input.text == "説明"));
            }
            2 => {
                let novel_translate::import_codec::ScopeRecord::Related(item) = forged.scope.records.iter_mut()
                    .find(|item| matches!(item, novel_translate::import_codec::ScopeRecord::Related(related) if related.input.text == "第一章")).unwrap() else { unreachable!() };
                item.input.text = "unrelated TOC label".to_owned();
            }
            _ => unreachable!(),
        }
        let staged = forged.stage(&objects, 0).unwrap();
        let result = prepare_staged_import(
            &workspace,
            ImportTarget {
                series: series.clone(),
                volume: volume.clone(),
                expected_parent: parent.clone(),
                operation: OperationId::parse(format!("{:032x}", 30 + change)).unwrap(),
                generation: GenerationId::parse(format!("{:032x}", 30 + change)).unwrap(),
            },
            staged,
        );
        assert!(
            matches!(result, Err(ImportFlowError::InvalidSnapshot)),
            "change {change} bypassed source proof"
        );
        assert_eq!(
            read_saved_volume(&workspace, &series, &volume, None)
                .unwrap()
                .generation,
            parent
        );
    }
}

fn stage(
    workspace: &Path,
    series: &SeriesId,
    volume: &VolumeId,
    source_path: &Path,
) -> StagedVolume {
    let objects = FileObjectStore::new(workspace).unwrap();
    let images = FileImportSourceStore::new(workspace).unwrap();
    let original = capture_epub(&objects, source_path).unwrap();
    let digest = Digest(*original.hash.as_bytes());
    let mut archive = EpubZip::new(objects.open_import_object(original).unwrap()).unwrap();
    let inventory = inspect(&mut archive);
    let source = extract_source(&mut archive, &inventory, digest.0, PARSER_VERSION).unwrap();
    let references = index_references(
        &mut archive,
        &inventory,
        &source,
        ReferenceIndexLimits::default(),
    )
    .unwrap();
    let input = source.chapter_map_input(Vec::new());
    let mapping = build_candidate_map(&input).unwrap();
    let files = ResourceFiles {
        objects: &objects,
        source_images: &images,
        volume,
        epub: digest,
    };
    let draft = SnapshotDraft::capture(
        SnapshotInputs {
            original,
            series,
            inventory: &inventory,
            source: &source,
            mapping: &mapping,
            confirmed_boundaries: &[],
            references: &references,
            mapping_rule_version: "test-map/1",
            classification_rule_version: "test-scope/1",
        },
        &files,
        &mut archive,
        0,
    )
    .unwrap();
    let scope = evaluate_scope(ScopeInput {
        context: draft.decision_context().clone(),
        mode: ScopeMode::AllText,
        content: observe_map_text(&input, &mapping).unwrap(),
        notes: Vec::new(),
        related: Vec::new(),
        note_references: Vec::new(),
    })
    .unwrap();
    draft.stage(&scope, &objects).unwrap()
}

#[test]
fn staged_import_submits_reopens_and_replays_the_exact_historical_generation() {
    let temp = TempDir::new();
    let workspace = temp.path().join("series");
    let metadata = create_workspace(&SqliteWorkspaceRepository, &workspace).unwrap();
    let series = metadata.series.id;
    let volume = VolumeId::parse("1".repeat(32)).unwrap();
    let source = temp.path().join("example.epub");
    // This ledger fixture deliberately has only ordinary text and one image.
    // The selected-file cases below cover authored note, alt and TOC semantics.
    let chapter = epub::CHAPTER.replace(" alt=\"説明\"", "").replace(
        r#"<a epub:type="noteref" href="notes.xhtml#note">1</a>"#,
        "",
    );
    let bytes = epub::book(3, &chapter);
    let bytes = replace_book_entry(
        &bytes,
        "OPS/Text/notes.xhtml",
        br#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p>Plain appendix</p></body></html>"#,
    );
    let bytes = replace_book_entry(&bytes, "OPS/nav.xhtml", br#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol/></nav></body></html>"#);
    fs::write(&source, &bytes).unwrap();
    let connection = rusqlite::Connection::open(workspace.join("series.sqlite3")).unwrap();
    connection
        .execute(
            "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms)
         VALUES (?1, ?2, 'Fixture', 0)",
            (series.as_str(), volume.as_str()),
        )
        .unwrap();
    drop(connection);

    let initial = GenerationId::parse("g0").unwrap();
    let generation = |number: u128| GenerationId::parse(format!("{number:032x}")).unwrap();
    let operation = |number: u128| OperationId::parse(format!("{number:032x}")).unwrap();
    let target = |op, result| ImportTarget {
        series: series.clone(),
        volume: volume.clone(),
        expected_parent: initial.clone(),
        operation: operation(op),
        generation: generation(result),
    };
    let staged = stage(&workspace, &series, &volume, &source);
    assert_eq!(
        staged.record.source.original_object_byte_len,
        bytes.len() as u64
    );
    let mapping = staged.record.source.mapping_version;
    let expected_hash = staged.record.source.epub_sha256;
    let prepared = prepare_staged_import(&workspace, target(1, 1), staged).unwrap();
    let conflict = prepare_staged_import(
        &workspace,
        target(1, 2),
        stage(&workspace, &series, &volume, &source),
    )
    .unwrap();
    let stale = prepare_staged_import(
        &workspace,
        target(3, 3),
        stage(&workspace, &series, &volume, &source),
    )
    .unwrap();
    assert!(matches!(
        prepared.reconcile().unwrap(),
        Publication::CommitOutcomeUnknown { operation: id, fingerprint }
            if id == operation(1) && fingerprint == prepared.fingerprint(),
    ));
    fs::write(&source, b"replaced after capture").unwrap();

    let Publication::Published {
        saved,
        replayed: false,
    } = prepared.submit().unwrap()
    else {
        panic!("expected the newly published generation");
    };
    assert_eq!(saved.generation, generation(1));
    let record = saved.record.unwrap();
    assert_eq!(record.source.epub_sha256, expected_hash);
    assert_eq!(record.source.mapping_version, mapping);
    assert!(
        record
            .resources
            .iter()
            .any(|resource| resource.source_relative_path.is_some())
    );
    assert!(matches!(
        conflict.submit(),
        Err(ImportFlowError::Storage(ImportWriteError::Generation(
            GenerationCommitError::OperationConflict
        ))),
    ));
    assert!(matches!(
        stale.submit(),
        Err(ImportFlowError::Storage(ImportWriteError::Generation(
            GenerationCommitError::StaleGeneration
        ))),
    ));

    SqliteGenerationWriter::open(&workspace)
        .unwrap()
        .commit_volume_import(
            VolumeImportWriteRequest::new(
                series.clone(),
                operation(4),
                generation(1),
                generation(4),
                ImportOperation::SetSeriesDefault {
                    mode: ScopeMode::Story,
                },
                Vec::new(),
            )
            .unwrap(),
        )
        .unwrap();
    for result in [prepared.submit().unwrap(), prepared.reconcile().unwrap()] {
        let Publication::Published {
            saved,
            replayed: true,
        } = result
        else {
            panic!("expected historical replay after the parent advanced");
        };
        assert_eq!(saved.generation, generation(1));
        assert_eq!(saved.record.unwrap().source.mapping_version, mapping);
    }
    let current = read_saved_volume(&workspace, &series, &volume, None).unwrap();
    assert_eq!(current.generation, generation(4));
    assert_eq!(current.version.state.mode(), ScopeMode::AllText);
    assert_eq!(current.record.unwrap().source.epub_sha256, expected_hash);
}

#[test]
fn selected_file_reopens_one_note_with_two_ranges_and_two_references() {
    use nt_application::source_scope::NoteTarget;
    use nt_application::volume_import::source_scope::SourceLocationKind;

    let temp = TempDir::new();
    let (workspace, target) = fresh_volume(&temp);
    let source = temp.path().join("note.epub");
    let chapter = epub::CHAPTER.replace(
        "</body>",
        r#"<p>More text <a epub:type="noteref" href="notes.xhtml#note">2</a></p></body>"#,
    );
    let notes = br#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>Notes</title></head><body><aside id="note" epub:type="footnote"><p>First part</p><p>Second part</p></aside></body></html>"#;
    let bytes = replace_book_entry(&epub::book(3, &chapter), "OPS/Text/notes.xhtml", notes);
    fs::write(&source, &bytes).unwrap();
    let (series, volume) = (target.series.clone(), target.volume.clone());
    let prepared = prepare_import(&workspace, &source, target).unwrap();
    let Publication::Published {
        saved,
        replayed: false,
    } = prepared.submit().unwrap()
    else {
        panic!("expected one published generation");
    };
    let scope = read_volume_scope(&workspace, &series, &volume, Some(&saved.generation))
        .unwrap()
        .unwrap();
    assert_eq!(scope.notes.len(), 1);
    let note = &scope.notes[0];
    let location = &note.content.input.observation.location;
    let SourceLocationKind::TextRanges(ranges) = location.kind() else {
        panic!("a text note retains its source ranges");
    };
    assert_eq!(ranges.len(), 2);
    assert_ne!(ranges[0].range().block_id(), ranges[1].range().block_id());
    assert!(
        ranges
            .iter()
            .all(|range| range.canonical_path() == "OPS/Text/notes.xhtml")
    );
    assert_eq!(
        note.content.input.observation.excerpt,
        "First part\nSecond part"
    );
    assert_eq!(note.content.handling, Handling::TranslateAndReview);
    assert_eq!(scope.note_references.len(), 2);
    assert_eq!(note.references, scope.note_references);
    assert_ne!(note.references[0].location, note.references[1].location);
    for reference in &note.references {
        assert_eq!(&reference.target, &NoteTarget::Resolved(location.clone()));
    }
    let current = read_volume_scope(&workspace, &series, &volume, None)
        .unwrap()
        .unwrap();
    assert_eq!(scope, current);
    assert_eq!(fs::read(source).unwrap(), bytes);
}

#[test]
fn selected_file_import_reopens_shared_notes_and_related_text_in_epub2_and_epub3() {
    for version in [2, 3] {
        let temp = TempDir::new();
        let workspace = temp.path().join("book series");
        let series = create_workspace(&SqliteWorkspaceRepository, &workspace)
            .unwrap()
            .series
            .id;
        let volume = VolumeId::parse("2".repeat(32)).unwrap();
        let connection = rusqlite::Connection::open(workspace.join("series.sqlite3")).unwrap();
        connection
            .execute(
                "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms)
            VALUES (?1, ?2, 'Fixture', 0)",
                (series.as_str(), volume.as_str()),
            )
            .unwrap();
        drop(connection);
        let parent = GenerationId::parse("3".repeat(32)).unwrap();
        let generation = GenerationId::parse("4".repeat(32)).unwrap();
        SqliteGenerationWriter::open(&workspace)
            .unwrap()
            .commit_volume_import(
                VolumeImportWriteRequest::new(
                    series.clone(),
                    OperationId::parse("3".repeat(32)).unwrap(),
                    GenerationId::parse("g0").unwrap(),
                    parent.clone(),
                    ImportOperation::SetVolumeScope {
                        volume_id: volume.clone(),
                        state: nt_application::volume_import::VolumeState::Unimported {
                            mode: ScopeMode::Story,
                        },
                        resources: Vec::new(),
                        default_delta: None,
                    },
                    Vec::new(),
                )
                .unwrap(),
            )
            .unwrap();

        let source = temp.path().join("巻 1.epub");
        let chapter = epub::CHAPTER
            .replace("<body>", r#"<body epub:type="chapter">"#)
            .replace("</body>", r#"<section epub:type="copyright-page"><p>Credit<a href="notes.xhtml#note">2</a></p></section></body>"#);
        let original = epub::book(version, &chapter);
        fs::write(&source, &original).unwrap();
        let prepared = prepare_import(
            &workspace,
            &source,
            ImportTarget {
                series: series.clone(),
                volume: volume.clone(),
                expected_parent: parent.clone(),
                operation: OperationId::parse("4".repeat(32)).unwrap(),
                generation: generation.clone(),
            },
        )
        .unwrap();
        assert_eq!(
            read_saved_volume(&workspace, &series, &volume, None)
                .unwrap()
                .generation,
            parent
        );
        let published = prepared.submit().unwrap();
        let Publication::Published {
            saved,
            replayed: false,
        } = published
        else {
            panic!("expected the selected file to publish and reopen");
        };
        let record = saved.record.unwrap();
        assert_eq!(saved.generation, generation);
        assert_eq!(
            record.source.original_object_byte_len,
            original.len() as u64
        );
        assert_eq!(record.scope.mode(), ScopeMode::Story);
        assert_eq!(record.scope.notes.len(), 1);
        let note = &record.scope.notes[0];
        assert_eq!(note.references.len(), 2);
        assert_ne!(note.references[0].location, note.references[1].location);
        assert_eq!(note.content.handling, ScopeHandling::TranslateAndReview);
        assert!(record.scope.records.iter().any(|record| matches!(
            record,
            novel_translate::import_codec::ScopeRecord::Related(related)
                if related.input.text == "説明" && related.handling == ScopeHandling::TranslateAndReview
        )));
        assert!(record.scope.records.iter().any(|record| matches!(
            record,
            novel_translate::import_codec::ScopeRecord::Content(content)
                if content.input.observation.excerpt.starts_with("Credit")
                    && content.handling == ScopeHandling::PreserveSource
        )));
        let scope = read_volume_scope(&workspace, &series, &volume, None)
            .unwrap()
            .unwrap();
        let label = scope
            .related
            .iter()
            .find(|item| {
                item.input.kind == nt_application::source_scope::RelatedKind::NavigationLabel
            })
            .unwrap();
        assert_eq!(label.input.text, "第一章");
        let nt_application::volume_import::source_scope::SourceLocationKind::Element {
            canonical_path,
            locator,
        } = label.input.location.kind()
        else {
            panic!("TOC label must have its actual XML element");
        };
        let expected = if version == 3 {
            ("OPS/nav.xhtml", vec![1, 0, 1, 0, 0])
        } else {
            ("OPS/toc.ncx", vec![2, 0, 0, 0])
        };
        assert_eq!(canonical_path, expected.0);
        assert_eq!(locator.element_ordinals, expected.1);
        let heading = scope
            .content
            .iter()
            .find(|item| item.input.observation.excerpt == "第一章")
            .unwrap();
        assert_eq!(
            label.input.relations,
            vec![nt_application::source_scope::Relation::Resolved(
                heading.input.observation.location.clone()
            )]
        );
        assert_eq!(label.handling, Handling::TranslateAndReview);
        assert_eq!(fs::read(&source).unwrap(), original);
    }
}
