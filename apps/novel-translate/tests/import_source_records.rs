#[path = "support/epub.rs"]
mod fixture;

#[path = "support/resource_records.rs"]
mod resource_records_cases;
#[path = "support/scope_records.rs"]
mod scope_records_cases;

use novel_translate::import_codec::{
    CodecError, Digest, StoredInventory, StoredMapping, StoredSourceIndex, StoredUnitRun,
};
use nt_epub::{EpubZip, PARSER_VERSION, extract_source, inspect};
use std::io::{Cursor, Write};

fn parsed(version: u8) -> (nt_epub::EpubInventory, nt_epub::SourceExtraction) {
    parsed_chapter(version, fixture::CHAPTER)
}

fn parsed_chapter(
    version: u8,
    chapter: &str,
) -> (nt_epub::EpubInventory, nt_epub::SourceExtraction) {
    let bytes = fixture::book(version, chapter);
    let mut sink = nt_storage::ImportDigestSink::new(bytes.len() as u64).unwrap();
    sink.write_all(&bytes).unwrap();
    let hash = *sink.finish().unwrap().hash.as_bytes();
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let inventory = inspect(&mut archive);
    let result = extract_source(&mut archive, &inventory, hash, PARSER_VERSION).unwrap();
    assert!(result.complete, "{:?}", result.diagnostics);
    (inventory, result)
}

fn extracted(version: u8) -> nt_epub::SourceExtraction {
    parsed(version).1
}

#[test]
fn source_runs_reject_mixed_duplicate_and_null_variant_fields() {
    for input in [
        r#"{"kind":"TextRun","text":"a","grapheme_lengths":"AQ==","segments":[[0,1,0,0,1]],"spans":[]}"#,
        r#"{"kind":"TextRun","kind":"TextRun","text":"a","grapheme_lengths":"AQ==","segments":[[0,1,0,0,1]]}"#,
        r#"{"kind":"Protected","unit":{"kind":"LineBreak","data":{"element_path":[1]}},"spans":[],"text":null}"#,
        r#"{"kind":"Protected","spans":[]}"#,
    ] {
        assert!(serde_json::from_str::<StoredUnitRun>(input).is_err());
    }
}

#[cfg(windows)]
#[test]
fn captured_epub2_and_epub3_seal_the_full_snapshot_and_exact_writer_references() {
    use novel_translate::import_codec::{
        ResourceFiles, SnapshotDraft, SnapshotInputs, VolumeRecord,
    };
    use novel_translate::import_workflow::capture_epub;
    use nt_application::chapter_map::build_candidate_map;
    use nt_application::source_scope::{ScopeInput, evaluate_scope, observe_map_text};
    use nt_application::volume_import::{VolumeId, source_scope::ScopeMode};
    use nt_application::workspace::create_workspace;
    use nt_epub::{ReferenceIndexLimits, index_references};
    use nt_storage::{FileImportSourceStore, FileObjectStore, SqliteWorkspaceRepository};
    use std::fs;
    use std::path::PathBuf;

    struct Temp(PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch_root = std::env::temp_dir().join(format!("nt-cp05-seal-{}", std::process::id()));
    fs::create_dir(&scratch_root).unwrap();
    let scratch = Temp(scratch_root);
    let workspace = scratch.0.join("series");
    let project = create_workspace(&SqliteWorkspaceRepository, &workspace).unwrap();
    let volume = VolumeId::parse("5".repeat(32)).unwrap();
    let objects = FileObjectStore::new(&workspace).unwrap();
    let source_images = FileImportSourceStore::new(&workspace).unwrap();
    let chapter = fixture::CHAPTER.replace(
        r#"<a epub:type="noteref" href="notes.xhtml#note">1</a>"#,
        "",
    );
    for version in [2, 3] {
        let path = scratch.0.join(format!("source-{version}.epub"));
        fs::write(&path, fixture::book(version, &chapter)).unwrap();
        let original = capture_epub(&objects, &path).unwrap();
        let mut archive = EpubZip::new(objects.open_import_object(original).unwrap()).unwrap();
        let inventory = inspect(&mut archive);
        let source = extract_source(
            &mut archive,
            &inventory,
            *original.hash.as_bytes(),
            PARSER_VERSION,
        )
        .unwrap();
        let references = index_references(
            &mut archive,
            &inventory,
            &source,
            ReferenceIndexLimits::default(),
        )
        .unwrap();
        let map_input = source.chapter_map_input(vec![]);
        let mapping = build_candidate_map(&map_input).unwrap();
        let files = ResourceFiles {
            objects: &objects,
            source_images: &source_images,
            volume: &volume,
            epub: Digest(*original.hash.as_bytes()),
        };
        let draft = SnapshotDraft::capture(
            SnapshotInputs {
                original,
                series: &project.series.id,
                inventory: &inventory,
                source: &source,
                mapping: &mapping,
                confirmed_boundaries: &map_input.confirmed_boundaries,
                references: &references,
                mapping_rule_version: "map/test",
                classification_rule_version: "scope/test",
            },
            &files,
            &mut archive,
            0,
        )
        .unwrap();
        let scope = evaluate_scope(ScopeInput {
            context: draft.decision_context().clone(),
            mode: ScopeMode::AllText,
            content: observe_map_text(&map_input, &mapping).unwrap(),
            notes: Vec::new(),
            related: Vec::new(),
            note_references: Vec::new(),
        })
        .unwrap();
        let staged = draft.stage(&scope, &objects).unwrap();
        assert_eq!(staged.state.epub_sha256, *original.hash.as_bytes());
        assert_eq!(staged.state.mapping_version, scope.context.mapping_version);
        let mut expected = vec![original, staged.state.payload.object];
        expected.extend(staged.resources.iter().map(|row| row.object));
        expected.sort_unstable_by_key(|object| object.hash);
        expected.dedup();
        assert_eq!(expected, staged.objects);

        let mut sealed = objects
            .open_import_object(staged.state.payload.object)
            .unwrap();
        let reopened = VolumeRecord::read_unverified(&mut sealed).unwrap();
        reopened.validate_structure().unwrap();
        assert!(reopened == staged.record);
        assert_eq!(reopened.mapping.clone().into_map().unwrap().0, mapping);
        assert!(reopened.scope.records.len() >= 3);
        let proof = reopened
            .source_index
            .replay_capture(
                &mut archive,
                &inventory,
                &reopened.resources,
                files.epub,
                PARSER_VERSION,
                0,
            )
            .unwrap();
        assert_eq!(proof.navigation, source.navigation);
        assert_eq!(proof.navigation_diagnostics, source.navigation_diagnostics);
        assert!(proof.document_diagnostics.is_empty());
        let rebuilt = nt_epub::ReferenceIndex::from_replayed(
            &mut archive,
            &inventory,
            &proof.images,
            &proof.document_diagnostics,
            nt_epub::ReferenceIndexLimits::default(),
        )
        .unwrap();
        assert_eq!(rebuilt, references);
        let verified = reopened
            .verify_capture(&files, &staged.resources, 0)
            .unwrap();
        assert_eq!(verified.inventory, inventory);
        assert_eq!(verified.references, references);

        let mut altered = reopened.clone();
        let image = altered
            .resources
            .iter_mut()
            .find(|resource| {
                resource
                    .uses
                    .iter()
                    .any(|usage| !usage.source_occurrences.is_empty())
            })
            .unwrap();
        let usage = image
            .uses
            .iter_mut()
            .find(|usage| !usage.source_occurrences.is_empty())
            .unwrap();
        usage.location.column += 1;
        altered.validate_structure().unwrap();
        assert!(matches!(
            altered.verify_capture(&files, &staged.resources, 0),
            Err(
                novel_translate::import_codec::CaptureVerificationError::Resources(
                    novel_translate::import_codec::ResourceFileError::EvidenceMismatch,
                )
            ),
        ));

        let mut forged = serde_json::to_value(&reopened.source_index).unwrap();
        let text = forged["documents"][0]["blocks"][0]["unit_runs"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|run| run["kind"] == "TextRun")
            .unwrap();
        let segment = text["segments"][0].as_array_mut().unwrap();
        for field in [3, 4] {
            segment[field] = serde_json::Value::from(segment[field].as_u64().unwrap() + 1);
        }
        let forged: StoredSourceIndex = serde_json::from_value(forged).unwrap();
        forged
            .validate_structure(files.epub, PARSER_VERSION)
            .unwrap();
        assert!(
            forged
                .replay_capture(
                    &mut archive,
                    &inventory,
                    &reopened.resources,
                    files.epub,
                    PARSER_VERSION,
                    0,
                )
                .is_err(),
            "valid-looking offsets must bind to the actual original node"
        );

        let mut forged = reopened.source_index.clone();
        forged.documents[0].spine_positions = vec![1234];
        forged.documents[0].linear_spine_positions = vec![1234];
        forged
            .validate_structure(files.epub, PARSER_VERSION)
            .unwrap();
        assert!(
            forged
                .replay_capture(
                    &mut archive,
                    &inventory,
                    &reopened.resources,
                    files.epub,
                    PARSER_VERSION,
                    0,
                )
                .is_err(),
            "spine positions must match the original archive"
        );
        files
            .verify(
                &mut archive,
                &inventory,
                &rebuilt,
                &reopened.resources,
                &staged.resources,
                0,
            )
            .unwrap();
    }
}

#[test]
fn real_map_roundtrip_keeps_tree_all_evidence_navigation_and_diagnostics() {
    use nt_application::{chapter_map::build_candidate_map, volume_import::chapter_map as map};
    for version in [2, 3] {
        let extracted = extracted(version);
        let input = extracted.chapter_map_input(vec![]);
        let map = build_candidate_map(&input).unwrap();
        map::validate_coverage(&input, &map).unwrap();
        let source = StoredSourceIndex::capture(&extracted, 0).unwrap();
        let stored = StoredMapping::capture(&map, &input.confirmed_boundaries, 0).unwrap();
        stored
            .validate_coverage(&source, Digest(extracted.epub_sha256), PARSER_VERSION)
            .unwrap();
        let json = serde_json::to_vec(&stored).unwrap();
        let reopened: StoredMapping = serde_json::from_slice(&json).unwrap();
        let (again, confirmed) = reopened.into_map().unwrap();
        assert_eq!(again, map);
        assert_eq!(confirmed, input.confirmed_boundaries);
        assert_eq!(again.navigation[0].items[0].label, "第一章");
        assert!(!again.volume.auxiliary.is_empty());
        assert!(!again.boundary_evidence.is_empty());
    }
}

#[test]
fn compact_mapping_coverage_agrees_with_the_published_domain_validator() {
    use nt_application::{chapter_map::build_candidate_map, volume_import::chapter_map as map};
    let extracted = extracted(3);
    let input = extracted.chapter_map_input(vec![]);
    let valid = build_candidate_map(&input).unwrap();
    let source = StoredSourceIndex::capture(&extracted, 0).unwrap();
    for case in 0..7 {
        let mut candidate = valid.clone();
        let map::MainFlowContent::Chapter(first) = &mut candidate.volume.main_flow[0] else {
            panic!("chapter fixture")
        };
        match case {
            0 => {}
            1 => {
                first.ranges.remove(0);
            }
            2 => {
                let duplicate = first.ranges[0].clone();
                first.ranges.insert(0, duplicate);
            }
            3 => {
                first.ranges[0].end = u32::MAX;
            }
            4 => {
                first.ranges[0].start = first.ranges[0].end;
            }
            5 => {
                first.ranges.reverse();
            }
            _ => {
                first.ranges[0].canonical_path = "OPS/other.xhtml".into();
            }
        }
        let expected = map::validate_coverage(&input, &candidate).is_ok();
        assert_eq!(expected, case == 0);
        let stored = StoredMapping::capture(&candidate, &[], 0).unwrap();
        assert_eq!(
            stored
                .validate_coverage(&source, Digest(extracted.epub_sha256), PARSER_VERSION)
                .is_ok(),
            expected
        );
    }
}

#[test]
fn inventory_roundtrip_keeps_metadata_resources_spine_and_both_navigation_forms() {
    for version in [2, 3] {
        let (inventory, _) = parsed(version);
        let stored = StoredInventory::capture(&inventory, 0).unwrap();
        let json = serde_json::to_vec(&stored).unwrap();
        let decoded: StoredInventory = serde_json::from_slice(&json).unwrap();
        assert_eq!(decoded.into_inventory().unwrap(), inventory);
    }
}

#[test]
fn flattened_navigation_keeps_nested_sibling_order_and_rejects_reopened_subtrees() {
    use nt_epub::{NavigationItem as N, NavigationSource, NavigationTree};
    let item = |label: &str, children: Vec<N>| N {
        label: label.into(),
        target: None,
        children,
    };
    let inventory = nt_epub::EpubInventory {
        navigation: vec![NavigationTree {
            source: NavigationSource::Epub3Nav,
            resource_path: "OPS/nav.xhtml".into(),
            items: vec![
                item(
                    "A",
                    vec![item("A1", vec![]), item("A2", vec![item("A2a", vec![])])],
                ),
                item("B", vec![]),
            ],
        }],
        ..Default::default()
    };
    let stored = StoredInventory::capture(&inventory, 0).unwrap();
    assert_eq!(
        stored.navigation[0]
            .items
            .iter()
            .map(|n| n.label.as_str())
            .collect::<Vec<_>>(),
        ["A", "A1", "A2", "A2a", "B"]
    );
    assert_eq!(stored.clone().into_inventory().unwrap(), inventory);
    for (parent, sibling) in [(Some(4), 0), (Some(1), 0), (None, 2)] {
        let mut invalid = stored.clone();
        invalid.navigation[0].items[4].parent_index = parent;
        invalid.navigation[0].items[4].sibling_index = sibling;
        assert_eq!(invalid.into_inventory(), Err(CodecError::InvalidRecord));
    }
    let mut invalid = stored;
    invalid.primary_navigation = Some(1);
    assert_eq!(invalid.into_inventory(), Err(CodecError::InvalidRecord));
}

#[test]
fn actual_epub2_and_epub3_source_records_roundtrip_every_unit_and_metadata() {
    for version in [2, 3] {
        let extracted = extracted(version);
        let stored = StoredSourceIndex::capture(&extracted, 0).unwrap();
        let json = serde_json::to_vec(&stored).unwrap();
        let reopened: StoredSourceIndex = serde_json::from_slice(&json).unwrap();
        assert_eq!(stored, reopened);
        reopened
            .validate_structure(Digest(extracted.epub_sha256), PARSER_VERSION)
            .unwrap();
        assert_eq!(reopened.documents.len(), extracted.documents.len());
        assert!(!reopened.anchors.is_empty());
        let mut saw_ruby = false;
        let mut saw_image = false;
        let mut saw_break = false;
        for (document, original) in reopened.documents.iter().zip(&extracted.documents) {
            assert_eq!(document.canonical_path, original.canonical_path);
            assert_eq!(document.blocks.len(), original.blocks.len());
            for (block, original) in document.blocks.iter().zip(&original.blocks) {
                assert!(block.matches_extracted(original).unwrap());
                assert_eq!(block.unit_count().unwrap() as usize, original.unit_count());
                if block.element_name == "p" && block.canonical_path == "OPS/Text/ch.xhtml" {
                    assert!(block.inline_metadata.len() >= 2);
                    assert_eq!(
                        block
                            .attribute_text
                            .iter()
                            .find(|a| a.attribute == "alt")
                            .unwrap()
                            .value,
                        "説明"
                    );
                    let StoredUnitRun::Text(run) = &block.unit_runs[0] else {
                        panic!("text first")
                    };
                    let units = run
                        .validate()
                        .unwrap()
                        .units()
                        .map(|u| u.text.to_owned())
                        .collect::<Vec<_>>();
                    assert_eq!(units, ["A", "e\u{301}", "👩🏽‍💻"]);
                    assert_eq!(
                        run.validate()
                            .unwrap()
                            .units()
                            .nth(1)
                            .unwrap()
                            .spans()
                            .count(),
                        2
                    );
                }
                for run in &block.unit_runs {
                    if let StoredUnitRun::Protected(unit) = run {
                        let value = serde_json::to_string(unit).unwrap();
                        saw_ruby |= value.contains("\"kind\":\"Ruby\"") && value.contains("かん");
                        saw_image |= value.contains("\"kind\":\"Image\"")
                            && value.contains("OPS/Images/a.png");
                        saw_break |= value.contains("\"kind\":\"LineBreak\"");
                    }
                }
            }
        }
        assert!(saw_ruby && saw_image && saw_break);
    }
}

#[test]
fn source_structure_rejects_wrong_identity_and_invalid_ownership() {
    let extracted = extracted(3);
    let valid = StoredSourceIndex::capture(&extracted, 0).unwrap();
    for case in 0..5 {
        let mut invalid = valid.clone();
        match case {
            0 => invalid.documents[0].blocks[0].id.0[0] ^= 1,
            1 => invalid.documents[0].blocks[0].canonical_path = "../escape.xhtml".into(),
            2 => invalid.anchors[0].unit_offset = u32::MAX,
            3 => invalid.documents[0].blocks[0].parser_version = "old".into(),
            _ => {
                let duplicate = invalid.documents[0].blocks[0].clone();
                invalid.documents[0].blocks.push(duplicate);
            }
        }
        assert_eq!(
            invalid.validate_structure(Digest(extracted.epub_sha256), PARSER_VERSION),
            Err(CodecError::InvalidRecord)
        );
    }
    assert_eq!(
        StoredSourceIndex::capture(&extracted, 256 * 1024 * 1024 + 1),
        Err(CodecError::ResourceLimit)
    );
}

#[test]
fn digests_are_closed_lowercase_strings_and_do_not_accept_arrays_or_overflow() {
    let hash = Digest([0xab; 32]);
    assert_eq!(
        serde_json::to_string(&hash).unwrap(),
        format!("\"{}\"", "ab".repeat(32))
    );
    for text in [
        format!("\"{}\"", "AB".repeat(32)),
        format!("\"{}\"", "a".repeat(63)),
        "[]".into(),
        "null".into(),
        format!("\"{}g\"", "a".repeat(63)),
    ] {
        assert!(serde_json::from_str::<Digest>(&text).is_err());
    }
}
