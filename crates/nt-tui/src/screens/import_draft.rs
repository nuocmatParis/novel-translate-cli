use crossterm::event::{KeyCode, KeyEvent};
use nt_application::volume_import::{
    VolumeId,
    chapter_map::edit::{ChapterBoundary, ChapterEdit},
    source_scope::{ContentKind, ScopeMode},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::import_commands::{
    BookEdit, BookError, BookVersion, BookView, BookWorkspace, ClassificationTarget,
};

use super::scope_label;

pub(super) struct Draft {
    pub id: u64,
    pub workspace: BookWorkspace,
    pub volume: VolumeId,
    form: Form,
    preview_input: Option<Form>,
    preview_error: Option<BookError>,
}

enum Form {
    Rename {
        chapter: u32,
        version: BookVersion,
        text: String,
    },
    Boundary {
        edit: BoundaryEdit,
        version: BookVersion,
        boundary: ChapterBoundary,
        limit: u32,
        text: String,
    },
    Classify {
        target: ClassificationTarget,
        mode: ScopeMode,
        selection: usize,
    },
    Confirm {
        edit: BookEdit,
        description: String,
        can_save: bool,
    },
}

enum BoundaryEdit {
    Split(u32),
    Move(u32, u32),
}

pub(super) enum DraftAction {
    Preview(BookEdit),
    Save(BookEdit),
    None,
}

const CLASSIFICATIONS: &[(ContentKind, &str)] = &[
    (ContentKind::Opening, "Mở đầu"),
    (ContentKind::Chapter, "Chương truyện"),
    (ContentKind::SideStory, "Ngoại truyện"),
    (ContentKind::Ending, "Kết truyện"),
    (ContentKind::Afterword, "Lời bạt"),
    (ContentKind::OtherStory, "Phần truyện khác"),
    (ContentKind::Copyright, "Bản quyền"),
    (ContentKind::AuthorIntroduction, "Giới thiệu tác giả"),
    (ContentKind::Advertisement, "Quảng cáo"),
    (ContentKind::PromotionalExcerpt, "Trích đoạn quảng bá"),
    (ContentKind::OtherAuxiliary, "Phần phụ khác"),
];

impl Draft {
    pub(super) fn new(
        id: u64,
        workspace: BookWorkspace,
        volume: VolumeId,
        view: &BookView,
        key: char,
    ) -> Option<Self> {
        let form = match key {
            'p' => {
                let mode = match view.mode {
                    ScopeMode::AllText => ScopeMode::Story,
                    ScopeMode::Story => ScopeMode::AllText,
                };
                Form::Confirm {
                    edit: BookEdit::Scope(mode),
                    description: format!(
                        "Đổi phạm vi tập sang {}?\nEnter: Xác nhận lưu · Esc: Trở về",
                        scope_label(mode)
                    ),
                    can_save: true,
                }
            }
            'b' => Form::Confirm {
                edit: BookEdit::Default(view.mode),
                description: format!(
                    "Đặt {} cho các tập nhập sau?\nEnter: Lưu mặc định bộ · Esc: Trở về",
                    scope_label(view.mode)
                ),
                can_save: true,
            },
            'k' => Form::Classify {
                target: view.source.as_ref()?.classification?,
                mode: view.mode,
                selection: 0,
            },
            'n' => Form::Rename {
                chapter: view.chapter?,
                version: view.version.clone()?,
                text: String::new(),
            },
            'm' => Form::Confirm {
                edit: BookEdit::Mapping {
                    version: view.version.clone()?,
                    edit: ChapterEdit::Merge {
                        first: view.chapter?,
                        second: view.next_chapter?,
                    },
                },
                description: "Gộp chương đang chọn với chương liền sau. Enter: Xem trước".into(),
                can_save: false,
            },
            's' | 'd' => {
                let source = view.source.as_ref()?;
                let chapter = view.chapter?;
                let edit = if key == 's' {
                    BoundaryEdit::Split(chapter)
                } else if let Some(previous) = view.previous_chapter {
                    BoundaryEdit::Move(previous, chapter)
                } else {
                    BoundaryEdit::Move(chapter, view.next_chapter?)
                };
                Form::Boundary {
                    edit,
                    version: view.version.clone()?,
                    boundary: source.start.clone(),
                    limit: source.end,
                    text: view.position.unit.to_string(),
                }
            }
            _ => return None,
        };
        Some(Self {
            id,
            workspace,
            volume,
            form,
            preview_input: None,
            preview_error: None,
        })
    }

    pub(super) fn is_for(&self, workspace: &BookWorkspace, volume: &VolumeId) -> bool {
        self.workspace.series == workspace.series
            && self.workspace.path == workspace.path
            && &self.volume == volume
    }

    pub(super) fn body(&self) -> String {
        let prompt = match &self.form {
            Form::Rename { text, .. } => {
                format!("Tên chương mới: {text}_\nEnter: Xem trước · Esc: Trở về · F2: Bỏ nháp")
            }
            Form::Boundary {
                edit,
                boundary,
                limit,
                text,
                ..
            } => {
                let action = match edit {
                    BoundaryEdit::Split(_) => "Tách chương",
                    BoundaryEdit::Move(_, _) => "Dời ranh giới",
                };
                format!(
                    "{action} tại đơn vị {}–{limit}: {text}_\nEnter: Xem trước · Esc: Trở về · F2: Bỏ nháp",
                    boundary.offset
                )
            }
            Form::Classify { selection, .. } => format!(
                "Loại vùng: {}\n↑/↓: Chọn loại · Enter: Xem lựa chọn · Esc: Trở về",
                CLASSIFICATIONS[*selection].1,
            ),
            Form::Confirm { description, .. } => description.clone(),
        };
        match &self.preview_error {
            Some(error) => format!("{}\n{prompt}", error.message()),
            None => prompt,
        }
    }

    pub(super) fn handle(&mut self, key: KeyEvent) -> DraftAction {
        match &mut self.form {
            Form::Rename {
                chapter,
                version,
                text,
            } => {
                if key.code == KeyCode::Enter && !text.trim().is_empty() {
                    let edit = BookEdit::Mapping {
                        version: version.clone(),
                        edit: ChapterEdit::Rename {
                            chapter: *chapter,
                            label: text.trim().to_owned(),
                        },
                    };
                    return self.preview(edit);
                }
                edit_text(text, key.code, false);
            }
            Form::Boundary {
                edit,
                version,
                boundary,
                limit,
                text,
            } => {
                if key.code == KeyCode::Enter {
                    if let Ok(offset) = text.parse::<u32>()
                        && (boundary.offset..=*limit).contains(&offset)
                    {
                        let mut boundary = boundary.clone();
                        boundary.offset = offset;
                        let edit = match *edit {
                            BoundaryEdit::Split(chapter) => ChapterEdit::Split {
                                chapter,
                                boundary,
                                label: None,
                            },
                            BoundaryEdit::Move(first, second) => ChapterEdit::MoveBoundary {
                                first,
                                second,
                                boundary,
                            },
                        };
                        let request = BookEdit::Mapping {
                            version: version.clone(),
                            edit,
                        };
                        return self.preview(request);
                    }
                } else {
                    edit_text(text, key.code, true);
                }
            }
            Form::Classify {
                target,
                mode,
                selection,
            } => match key.code {
                KeyCode::Up => *selection = selection.saturating_sub(1),
                KeyCode::Down => *selection = (*selection + 1).min(CLASSIFICATIONS.len() - 1),
                KeyCode::Enter => {
                    let (kind, name) = CLASSIFICATIONS[*selection];
                    self.form = Form::Confirm {
                        edit: BookEdit::Classification {
                            target: *target,
                            kind,
                            mode: *mode,
                        },
                        description: format!("Xác nhận vùng là {name}?\nEnter: Lưu · Esc: Trở về"),
                        can_save: true,
                    };
                }
                _ => {}
            },
            Form::Confirm { edit, can_save, .. } if key.code == KeyCode::Enter => {
                let request = edit.clone();
                return if *can_save {
                    DraftAction::Save(request)
                } else {
                    self.preview(request)
                };
            }
            Form::Confirm { .. } => {}
        }
        DraftAction::None
    }

    fn preview(&mut self, edit: BookEdit) -> DraftAction {
        self.preview_error = None;
        let confirmation = Form::Confirm {
            edit: edit.clone(),
            description: "Đang dựng bản xem trước…".into(),
            can_save: false,
        };
        let input = std::mem::replace(&mut self.form, confirmation);
        self.preview_input = match input {
            Form::Rename { .. } | Form::Boundary { .. } => Some(input),
            _ => None,
        };
        DraftAction::Preview(edit)
    }

    pub(super) fn set_preview(&mut self, result: Result<String, BookError>) {
        let result = match (result, self.preview_input.take()) {
            (Err(error), Some(input)) => {
                self.form = input;
                self.preview_error = Some(error);
                return;
            }
            (result, _) => result,
        };
        let Form::Confirm {
            description,
            can_save,
            ..
        } = &mut self.form
        else {
            return;
        };
        match result {
            Ok(preview) => {
                *description = preview;
                *can_save = true;
            }
            Err(error) => {
                *description = format!("{}\nF2: Bỏ bản nháp · Esc: Trở về", error.message());
                *can_save = false;
            }
        }
    }
}

#[cfg(test)]
#[path = "import_draft_tests.rs"]
mod tests;

fn edit_text(text: &mut String, code: KeyCode, digits: bool) {
    match code {
        KeyCode::Backspace => {
            if let Some((offset, _)) = text.grapheme_indices(true).next_back() {
                text.truncate(offset);
            }
        }
        KeyCode::Char(character)
            if !character.is_control()
                && (!digits || character.is_ascii_digit())
                && text.len() + character.len_utf8() <= if digits { 10 } else { 512 } =>
        {
            text.push(character);
        }
        _ => {}
    }
}
