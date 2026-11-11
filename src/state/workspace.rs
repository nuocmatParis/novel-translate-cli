#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeStatus {
    Completed,
    InProgress,
    NotStarted,
}

impl VolumeStatus {
    pub fn label(&self) -> &'static str {
        match self {
            VolumeStatus::Completed => "Completed",
            VolumeStatus::InProgress => "In Progress",
            VolumeStatus::NotStarted => "Not Started",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChapterStatus {
    Completed,
    InReview,
    Translating,
    Queued,
    NotStarted,
}

impl ChapterStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ChapterStatus::Completed => "Completed",
            ChapterStatus::InReview => "In Review",
            ChapterStatus::Translating => "Translating",
            ChapterStatus::Queued => "Queued",
            ChapterStatus::NotStarted => "Not Started",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockChapter {
    pub id: String,
    pub index: usize,
    pub title: String,
    pub original_title: String,
    pub status: ChapterStatus,
    pub total_blocks: usize,
    pub translated_blocks: usize,
    pub reviewed_blocks: usize,
    pub selected: bool,
}

#[derive(Debug, Clone)]
pub struct MockVolume {
    pub id: String,
    pub number: usize,
    pub title: String,
    pub status: VolumeStatus,
    pub progress_percent: u16,
    pub chapters: Vec<MockChapter>,
}

#[derive(Debug, Clone)]
pub struct MockSeries {
    pub title: String,
    pub original_title: String,
    pub author: String,
    pub illustrator: String,
    pub description: String,
    pub volumes: Vec<MockVolume>,
}

#[derive(Debug, Clone)]
pub struct MockWorkspace {
    pub name: String,
    pub root_path: String,
    pub series: MockSeries,
    pub active_volume_index: usize,
    pub active_chapter_index: usize,
}
