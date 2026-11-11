#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    S0Suggestion,
    S1Minor,
    S2Major,
    S3Critical,
}

impl Severity {
    pub fn code(&self) -> &'static str {
        match self {
            Severity::S0Suggestion => "S0",
            Severity::S1Minor => "S1",
            Severity::S2Major => "S2",
            Severity::S3Critical => "S3",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Severity::S0Suggestion => "S0 — Gợi ý (Suggestion)",
            Severity::S1Minor => "S1 — Nhỏ (Minor)",
            Severity::S2Major => "S2 — Lớn (Major)",
            Severity::S3Critical => "S3 — Nghiêm trọng (Critical)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Severity::S0Suggestion => {
                "Đúng ý và phù hợp quy ước; khác biệt chủ yếu là lựa chọn diễn đạt. Không chặn."
            }
            Severity::S1Minor => {
                "Sai sót rõ về câu chữ hoặc quy ước, chưa làm đổi thông tin cốt lõi."
            }
            Severity::S2Major => {
                "Sai/mất thông tin cần truyền đạt nhưng chưa thay đổi tiền đề hay sự kiện then chốt."
            }
            Severity::S3Critical => {
                "Làm thay đổi sự kiện, danh tính, quan hệ hoặc điều kiện then chốt của cốt truyện."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionScope {
    M1DirectFix,
    M2RetranslateBlock,
    M3RetranslateScene,
    M4RetranslateChapter,
}

impl ActionScope {
    pub fn code(&self) -> &'static str {
        match self {
            ActionScope::M1DirectFix => "M1",
            ActionScope::M2RetranslateBlock => "M2",
            ActionScope::M3RetranslateScene => "M3",
            ActionScope::M4RetranslateChapter => "M4",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ActionScope::M1DirectFix => "M1 — Sửa trực tiếp (Direct Fix)",
            ActionScope::M2RetranslateBlock => "M2 — Dịch lại đoạn (Block Retranslate)",
            ActionScope::M3RetranslateScene => "M3 — Dịch lại cảnh/nhóm đoạn (Scene)",
            ActionScope::M4RetranslateChapter => "M4 — Dịch lại chương (Chapter)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ActionScope::M1DirectFix => {
                "Biết chính xác phần cần thay; giữ nguyên cấu trúc xung quanh."
            }
            ActionScope::M2RetranslateBlock => {
                "Tạo lại ý/cách diễn đạt của 1 đoạn, ảnh hưởng khoanh vùng trong đoạn."
            }
            ActionScope::M3RetranslateScene => {
                "Nhiều đoạn cùng chịu một cách hiểu sai hoặc phụ thuộc lẫn nhau."
            }
            ActionScope::M4RetranslateChapter => {
                "Sai tiền đề lan rộng, sửa chọn lọc không đủ khôi phục mạch truyện."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceStatus {
    Confirmed,
    NeedsVerification,
    Rejected,
}

impl EvidenceStatus {
    pub fn label(&self) -> &'static str {
        match self {
            EvidenceStatus::Confirmed => "Đã xác nhận lỗi (Confirmed)",
            EvidenceStatus::NeedsVerification => "Cần xác minh (Needs Verification)",
            EvidenceStatus::Rejected => "Đã bác nhận định (Rejected)",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockReviewIssue {
    pub id: String,
    pub volume_num: usize,
    pub chapter_title: String,
    pub block_id: String,
    pub category: String,
    pub severity: Severity,
    pub action_scope: ActionScope,
    pub evidence_status: EvidenceStatus,
    pub japanese_source: String,
    pub vietnamese_translation: String,
    pub explanation: String,
    pub suggested_correction: String,
    pub cycles_count: usize,
    pub max_cycles: usize,
    pub resolved: bool,
    pub user_exception: bool,
    pub feedback_notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MockEditorBlock {
    pub id: String,
    pub block_num: usize,
    pub japanese: String,
    pub vietnamese: String,
    pub revision: usize,
    pub review_status: String,
    pub last_modified: String,
    pub history: Vec<String>,
}
