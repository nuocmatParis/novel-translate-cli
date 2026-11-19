use std::path::{Path, PathBuf};

use nt_application::volume_import::{
    GenerationId, SeriesId, VolumeId,
    chapter_map::edit::{ChapterBoundary, ChapterEdit},
    source_scope::{ContentKind, ScopeMode},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BookWorkspace {
    pub path: PathBuf,
    pub series: SeriesId,
    pub generation: GenerationId,
}

#[derive(Clone, Debug)]
pub struct BookCatalog {
    pub generation: GenerationId,
    pub volumes: Vec<BookSummary>,
}

#[derive(Clone, Debug)]
pub struct BookSummary {
    pub id: VolumeId,
    pub title: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BookPosition {
    pub row: usize,
    pub range: usize,
    pub unit: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BookVersion {
    pub epub_sha256: [u8; 32],
    pub parser_version: String,
    pub mapping_version: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct BookView {
    pub version: Option<BookVersion>,
    pub mode: ScopeMode,
    pub position: BookPosition,
    pub rows: usize,
    pub ranges: usize,
    pub caption: String,
    pub review_count: usize,
    pub chapter: Option<u32>,
    pub previous_chapter: Option<u32>,
    pub next_chapter: Option<u32>,
    pub source: Option<BookSource>,
}

#[derive(Clone, Debug)]
pub struct BookSource {
    pub start: ChapterBoundary,
    pub end: u32,
    pub text: String,
    pub classification: Option<ClassificationTarget>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClassificationTarget {
    Content(usize),
    Note(usize),
}

#[derive(Clone, Debug)]
pub enum BookEdit {
    Scope(ScopeMode),
    Default(ScopeMode),
    Classification {
        target: ClassificationTarget,
        kind: ContentKind,
        mode: ScopeMode,
    },
    Mapping {
        version: BookVersion,
        edit: ChapterEdit,
    },
}

#[derive(Clone, Debug)]
pub struct BookChange {
    pub generation: GenerationId,
    pub volume: VolumeId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BookError {
    ChooserUnavailable,
    InvalidSource,
    SourcePermission,
    Stale,
    InvalidEdit,
    NotSaved,
    Unknown(String),
    SavedButUnreadable,
}

impl BookError {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::ChooserUnavailable => {
                "Không mở được hộp chọn EPUB. Nhấn Enter để thử lại hoặc Esc để trở về.".to_owned()
            }
            Self::InvalidSource => {
                "Không đọc được EPUB. Kiểm tra tệp .epub và thử chọn lại.".to_owned()
            }
            Self::SourcePermission => {
                "Không đủ quyền đọc EPUB. Kiểm tra quyền tệp rồi chọn lại.".to_owned()
            }
            Self::Stale => {
                "Workspace vừa thay đổi. Nhấn R để đọc lại. Bản nháp vẫn giữ.".to_owned()
            }
            Self::InvalidEdit => "Thao tác không khớp ranh giới hoặc dữ liệu đang xem.".to_owned(),
            Self::NotSaved => {
                "Không hoàn tất được thao tác. Kiểm tra workspace rồi thử lại.".to_owned()
            }
            Self::Unknown(operation) => format!(
                "Chưa rõ kết quả lưu. Đọc lại workspace và đối chiếu mã thao tác {operation}."
            ),
            Self::SavedButUnreadable => {
                "Đã lưu nhưng chưa đọc lại được. Mở lại workspace để kiểm tra.".to_owned()
            }
        }
    }
}

/// Both front ends use the same app-owned import and write operations.
pub trait VolumeCommands: Send + Sync + 'static {
    fn choose_epub(&self) -> Result<Option<PathBuf>, BookError>;
    fn import_epub(&self, workspace: &BookWorkspace, epub: &Path) -> Result<BookChange, BookError>;
    fn catalog(&self, workspace: &BookWorkspace) -> Result<BookCatalog, BookError>;
    fn view(
        &self,
        workspace: &BookWorkspace,
        volume: &VolumeId,
        position: BookPosition,
    ) -> Result<BookView, BookError>;
    fn preview(
        &self,
        workspace: &BookWorkspace,
        volume: &VolumeId,
        edit: &BookEdit,
    ) -> Result<String, BookError>;
    fn save(
        &self,
        workspace: &BookWorkspace,
        volume: &VolumeId,
        edit: &BookEdit,
    ) -> Result<BookChange, BookError>;
}
