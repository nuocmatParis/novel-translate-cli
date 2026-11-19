use crate::chapter_map::{
    AuxiliaryDocumentCoverage, ChapterMapInput, CoverageDomain, SourceDocument, VolumeTree,
    validate_coverage,
};
use crate::source_range::{BlockLocator, ProtectedSourceUnit, SourceBlock, SourceUnit};

use super::*;

const PATH: &str = "OPS/main.xhtml";
const AUX: &str = "OPS/notes.xhtml";

fn block_id(seed: u8) -> SourceBlockId {
    SourceBlockId::from_bytes([seed; 32])
}

fn range(path: &str, seed: u8, start: u32, end: u32) -> BlockUnitRange {
    BlockUnitRange {
        canonical_path: path.to_owned(),
        block_id: block_id(seed),
        start,
        end,
    }
}

fn position(seed: u8, offset: u32) -> ChapterBoundary {
    ChapterBoundary {
        canonical_path: PATH.to_owned(),
        block_id: block_id(seed),
        offset,
    }
}

fn document_block(seed: u8, path: &str, count: usize) -> SourceBlock {
    let mut units = vec![SourceUnit::grapheme("字".to_owned(), Vec::new()); count];
    if seed == 1 {
        units[1] = SourceUnit::protected(
            ProtectedSourceUnit::SceneBreak {
                element_path: vec![2],
            },
            Vec::new(),
        );
    }
    SourceBlock {
        id: block_id(seed),
        canonical_path: path.to_owned(),
        element_name: "p".to_owned(),
        locator: BlockLocator::NodePath {
            element_ordinals: vec![0, seed.into()],
            order: seed.into(),
        },
        parser_version: "test/1".to_owned(),
        content_hash: [seed; 32],
        units,
        inline_metadata: Vec::new(),
        attribute_text: Vec::new(),
        protected_resource_hashes: Vec::new(),
    }
}

fn fixture() -> (ChapterMapInput, ChapterMap) {
    let input = ChapterMapInput {
        epub_sha256: [42; 32],
        parser_version: "test/1".to_owned(),
        documents: vec![
            SourceDocument {
                canonical_path: PATH.to_owned(),
                domain: CoverageDomain::MainFlow,
                spine_positions: vec![0],
                linear_spine_positions: vec![0],
                blocks: vec![document_block(1, PATH, 5), document_block(2, PATH, 2)],
            },
            SourceDocument {
                canonical_path: AUX.to_owned(),
                domain: CoverageDomain::Auxiliary,
                spine_positions: vec![1],
                linear_spine_positions: Vec::new(),
                blocks: vec![document_block(3, AUX, 1)],
            },
        ],
        anchors: Vec::new(),
        navigation: Vec::new(),
        confirmed_boundaries: Vec::new(),
        diagnostics: Vec::new(),
    };
    let second = confirmed_evidence(&position(1, 3), &Some("Second".to_owned()));
    let mapping = ChapterMap {
        epub_sha256: input.epub_sha256,
        parser_version: input.parser_version.clone(),
        volume: VolumeTree {
            main_flow: vec![
                MainFlowContent::Chapter(Chapter {
                    chapter_id: 1,
                    label: Some("First".to_owned()),
                    status: MappingStatus::Provisional,
                    ranges: vec![range(PATH, 1, 0, 3)],
                    start_evidence: Vec::new(),
                }),
                MainFlowContent::Chapter(Chapter {
                    chapter_id: 2,
                    label: Some("Second".to_owned()),
                    status: MappingStatus::Confirmed,
                    ranges: vec![range(PATH, 1, 3, 5), range(PATH, 2, 0, 2)],
                    start_evidence: vec![second.clone()],
                }),
            ],
            auxiliary: vec![AuxiliaryDocumentCoverage {
                canonical_path: AUX.to_owned(),
                ranges: vec![range(AUX, 3, 0, 1)],
            }],
        },
        navigation: Vec::new(),
        boundary_evidence: vec![second],
        diagnostics: Vec::new(),
    };
    validate_coverage(&input, &mapping).unwrap();
    (input, mapping)
}

fn chapter(mapping: &ChapterMap, index: usize) -> &Chapter {
    let MainFlowContent::Chapter(chapter) = &mapping.volume.main_flow[index] else {
        panic!("expected a chapter");
    };
    chapter
}

#[test]
fn rename_changes_only_the_display_label_and_reports_the_chapter() {
    let (input, mut expected) = fixture();
    let edited = preview_chapter_edit(
        expected.clone(),
        &ChapterEdit::Rename {
            chapter: 1,
            label: "序章".to_owned(),
        },
    )
    .unwrap();

    chapter_mut(&mut expected.volume.main_flow[0], 1)
        .unwrap()
        .label = Some("序章".to_owned());
    assert_eq!(edited.mapping, expected);
    assert_eq!(edited.impact.chapters, vec![1]);
    assert_eq!(edited.impact.previous_boundary, None);
    assert_eq!(edited.impact.new_boundary, None);
    validate_coverage(&input, &edited.mapping).unwrap();
}

#[test]
fn merge_keeps_the_first_id_and_removes_only_the_join() {
    let (input, mapping) = fixture();
    let other_coverage = mapping.volume.auxiliary.clone();
    let edited = preview_chapter_edit(
        mapping,
        &ChapterEdit::Merge {
            first: 1,
            second: 2,
        },
    )
    .unwrap();
    assert_eq!(edited.mapping.volume.main_flow.len(), 1);
    let chapter = chapter(&edited.mapping, 0);
    assert_eq!(chapter.chapter_id, 1);
    assert_eq!(chapter.label.as_deref(), Some("First"));
    assert_eq!(chapter.ranges, [range(PATH, 1, 0, 5), range(PATH, 2, 0, 2)]);
    assert_eq!(edited.impact.previous_boundary, Some(position(1, 3)));
    assert_eq!(edited.impact.new_boundary, None);
    assert!(edited.mapping.boundary_evidence.is_empty());
    assert_eq!(edited.mapping.volume.auxiliary, other_coverage);
    validate_coverage(&input, &edited.mapping).unwrap();
}

#[test]
fn split_keeps_the_protected_unit_whole_and_allocates_a_distinct_chapter() {
    let (input, mapping) = fixture();
    let edited = preview_chapter_edit(
        mapping.clone(),
        &ChapterEdit::Split {
            chapter: 1,
            boundary: position(1, 2),
            label: Some("続き".to_owned()),
        },
    )
    .unwrap();
    let first = chapter(&edited.mapping, 0);
    let next = chapter(&edited.mapping, 1);
    assert!(input.documents[0].blocks[0].units[1].is_protected());
    assert_eq!(first.ranges, [range(PATH, 1, 0, 2)]);
    assert_eq!(next.ranges, [range(PATH, 1, 2, 3)]);
    assert_eq!(next.chapter_id, 3);
    assert_eq!(next.status, MappingStatus::Confirmed);
    assert_eq!(next.start_evidence[0].unit_offset, Some(2));
    assert_eq!(chapter(&edited.mapping, 2), chapter(&mapping, 1));
    assert_eq!(edited.impact.chapters, vec![1, 3]);
    assert_eq!(edited.impact.new_boundary, Some(position(1, 2)));
    validate_coverage(&input, &edited.mapping).unwrap();
}

#[test]
fn move_reports_both_joins_and_preserves_full_coverage_across_blocks() {
    let (input, mapping) = fixture();
    let edited = preview_chapter_edit(
        mapping,
        &ChapterEdit::MoveBoundary {
            first: 1,
            second: 2,
            boundary: position(2, 1),
        },
    )
    .unwrap();
    let first = chapter(&edited.mapping, 0);
    let second = chapter(&edited.mapping, 1);
    assert_eq!(first.ranges, [range(PATH, 1, 0, 5), range(PATH, 2, 0, 1)]);
    assert_eq!(second.ranges, [range(PATH, 2, 1, 2)]);
    assert_eq!(second.label.as_deref(), Some("Second"));
    assert_eq!(edited.impact.previous_boundary, Some(position(1, 3)));
    assert_eq!(edited.impact.new_boundary, Some(position(2, 1)));
    assert_eq!(edited.mapping.boundary_evidence, second.start_evidence);
    validate_coverage(&input, &edited.mapping).unwrap();
}

#[test]
fn edge_wrong_source_and_unchanged_moves_return_typed_errors() {
    let (_, mapping) = fixture();
    let mut wrong_path = position(1, 1);
    wrong_path.canonical_path = AUX.to_owned();
    for boundary in [position(1, 0), position(1, 3), position(1, 9), wrong_path] {
        let edit = ChapterEdit::Split {
            chapter: 1,
            boundary: boundary.clone(),
            label: None,
        };
        assert_eq!(
            preview_chapter_edit(mapping.clone(), &edit),
            Err(ChapterEditError::InvalidBoundary(boundary))
        );
    }
    assert_eq!(
        preview_chapter_edit(
            mapping.clone(),
            &ChapterEdit::Merge {
                first: 2,
                second: 1
            }
        ),
        Err(ChapterEditError::ChaptersNotAdjacent {
            first: 2,
            second: 1
        })
    );
    let unchanged = ChapterEdit::MoveBoundary {
        first: 1,
        second: 2,
        boundary: position(1, 3),
    };
    assert_eq!(
        preview_chapter_edit(mapping, &unchanged),
        Err(ChapterEditError::Unchanged)
    );
}

#[test]
fn split_normalizes_the_end_of_one_block_to_the_start_of_the_next() {
    let (input, mapping) = fixture();
    let edited = preview_chapter_edit(
        mapping,
        &ChapterEdit::Split {
            chapter: 2,
            boundary: position(1, 5),
            label: None,
        },
    )
    .unwrap();
    assert_eq!(chapter(&edited.mapping, 1).ranges, [range(PATH, 1, 3, 5)]);
    assert_eq!(chapter(&edited.mapping, 2).ranges, [range(PATH, 2, 0, 2)]);
    assert_eq!(edited.impact.new_boundary, Some(position(2, 0)));
    validate_coverage(&input, &edited.mapping).unwrap();
}
