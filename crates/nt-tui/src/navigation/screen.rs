#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Screen {
    Overview,
    VolumesAndChapters,
    Tasks,
    JapaneseVietnameseEditor,
    Reviewer,
    SkillsAndContext,
    ImagesAndExport,
    SettingsReportsAndHelp,
}

impl Screen {
    pub(crate) const ALL: [Self; 8] = [
        Self::Overview,
        Self::VolumesAndChapters,
        Self::Tasks,
        Self::JapaneseVietnameseEditor,
        Self::Reviewer,
        Self::SkillsAndContext,
        Self::ImagesAndExport,
        Self::SettingsReportsAndHelp,
    ];

    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Overview => "Tổng quan",
            Self::VolumesAndChapters => "Tập và chương",
            Self::Tasks => "Tác vụ",
            Self::JapaneseVietnameseEditor => "Editor Nhật–Việt",
            Self::Reviewer => "Reviewer",
            Self::SkillsAndContext => "Skills và ngữ cảnh",
            Self::ImagesAndExport => "Ảnh và xuất",
            Self::SettingsReportsAndHelp => "Cấu hình, báo cáo và trợ giúp",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Overview => 0,
            Self::VolumesAndChapters => 1,
            Self::Tasks => 2,
            Self::JapaneseVietnameseEditor => 3,
            Self::Reviewer => 4,
            Self::SkillsAndContext => 5,
            Self::ImagesAndExport => 6,
            Self::SettingsReportsAndHelp => 7,
        }
    }
}
