use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use nt_application::volume_import::{GenerationId, SeriesId, source_range::SourceBlockId};

use super::*;
use crate::import_commands::{BookPosition, BookSource};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn fixture(action: char) -> Draft {
    let view = BookView {
        version: Some(BookVersion {
            epub_sha256: [1; 32],
            parser_version: "fixture-parser".to_owned(),
            mapping_version: [2; 32],
        }),
        mode: ScopeMode::Story,
        position: BookPosition::default(),
        rows: 1,
        ranges: 1,
        caption: "Fixture chapter".to_owned(),
        review_count: 0,
        chapter: Some(7),
        previous_chapter: None,
        next_chapter: None,
        source: Some(BookSource {
            start: ChapterBoundary {
                canonical_path: "chapter.xhtml".to_owned(),
                block_id: SourceBlockId::from_bytes([3; 32]),
                offset: 0,
            },
            end: 10,
            text: "abcdefghij".to_owned(),
            classification: None,
        }),
    };
    Draft::new(
        1,
        BookWorkspace {
            path: PathBuf::from("fixture-workspace"),
            series: SeriesId::parse("11111111111111111111111111111111").unwrap(),
            generation: GenerationId::parse("g0").unwrap(),
        },
        VolumeId::parse("22222222222222222222222222222222").unwrap(),
        &view,
        action,
    )
    .unwrap()
}

#[test]
fn rejected_split_returns_to_its_position_input_and_previews_the_corrected_offset() {
    let mut draft = fixture('s');
    let expected_workspace = draft.workspace.clone();
    let expected_volume = draft.volume.clone();
    let DraftAction::Preview(BookEdit::Mapping {
        version: initial_version,
        edit: ChapterEdit::Split { boundary, .. },
    }) = draft.handle(key(KeyCode::Enter))
    else {
        panic!("expected a preview request");
    };
    assert_eq!(boundary.offset, 0);
    draft.set_preview(Err(BookError::InvalidEdit));
    assert!(draft.body().contains("Tách chương tại đơn vị 0–10: 0_"));
    assert!(draft.body().contains("không khớp"));
    assert!(matches!(
        draft.handle(key(KeyCode::Backspace)),
        DraftAction::None
    ));
    assert!(matches!(
        draft.handle(key(KeyCode::Char('5'))),
        DraftAction::None
    ));
    let DraftAction::Preview(BookEdit::Mapping {
        version,
        edit: ChapterEdit::Split { boundary, .. },
    }) = draft.handle(key(KeyCode::Enter))
    else {
        panic!("editing should ask to preview again");
    };
    assert_eq!(boundary.offset, 5);
    assert_eq!(version, initial_version);
    assert_eq!(draft.workspace, expected_workspace);
    assert_eq!(draft.volume, expected_volume);
    draft.set_preview(Ok("Xem trước tại 5".to_owned()));
    assert!(draft.body().contains("Xem trước tại 5"));
    assert!(matches!(
        draft.handle(key(KeyCode::Enter)),
        DraftAction::Save(BookEdit::Mapping { edit: ChapterEdit::Split { boundary, .. }, .. })
            if boundary.offset == 5
    ));
}

#[test]
fn rejected_rename_keeps_its_unicode_input_for_editing() {
    let mut draft = fixture('n');
    for character in "章X".chars() {
        draft.handle(key(KeyCode::Char(character)));
    }
    assert!(matches!(
        draft.handle(key(KeyCode::Enter)),
        DraftAction::Preview(_)
    ));
    draft.set_preview(Err(BookError::InvalidEdit));
    assert!(draft.body().contains("章X_"));
    draft.handle(key(KeyCode::Backspace));
    draft.handle(key(KeyCode::Char('二')));
    assert!(matches!(
        draft.handle(key(KeyCode::Enter)),
        DraftAction::Preview(BookEdit::Mapping { edit: ChapterEdit::Rename { label, .. }, .. })
            if label == "章二"
    ));
}
