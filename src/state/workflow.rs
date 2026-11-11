#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Pending,
    Running,
    Completed,
    Failed,
    WaitingForUser,
}

impl StepState {
    pub fn label(&self) -> &'static str {
        match self {
            StepState::Pending => "Pending",
            StepState::Running => "Running",
            StepState::Completed => "Completed",
            StepState::Failed => "Failed",
            StepState::WaitingForUser => "Waiting for User",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowStage {
    PreparingContext,
    TranslatingBatch,
    ValidatingStructure,
    ReviewingBatch,
    FixingIssues,
    FinalChapterReview,
}

impl WorkflowStage {
    pub fn all() -> &'static [WorkflowStage] {
        &[
            WorkflowStage::PreparingContext,
            WorkflowStage::TranslatingBatch,
            WorkflowStage::ValidatingStructure,
            WorkflowStage::ReviewingBatch,
            WorkflowStage::FixingIssues,
            WorkflowStage::FinalChapterReview,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            WorkflowStage::PreparingContext => "1. Preparing Context",
            WorkflowStage::TranslatingBatch => "2. Translating Batch",
            WorkflowStage::ValidatingStructure => "3. Validating Structure",
            WorkflowStage::ReviewingBatch => "4. Reviewing Batch",
            WorkflowStage::FixingIssues => "5. Fixing Issues",
            WorkflowStage::FinalChapterReview => "6. Final Chapter Review",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            WorkflowStage::PreparingContext => {
                "Loading series skills, character addressing and terminology"
            }
            WorkflowStage::TranslatingBatch => {
                "Generating Japanese -> Vietnamese translation batch with context"
            }
            WorkflowStage::ValidatingStructure => {
                "Verifying XML/XHTML tags, block mapping and paragraph boundaries"
            }
            WorkflowStage::ReviewingBatch => {
                "Rubric-based evaluation: terminology, continuity, accuracy S0-S3"
            }
            WorkflowStage::FixingIssues => {
                "Applying scoped corrections M1-M4 (max 5 cycles before intervention)"
            }
            WorkflowStage::FinalChapterReview => {
                "End-of-chapter narrative flow and global consistency check"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelRole {
    Translation,
    Review,
    Terminology,
}

impl ModelRole {
    pub fn label(&self) -> &'static str {
        match self {
            ModelRole::Translation => "Translation",
            ModelRole::Review => "Review",
            ModelRole::Terminology => "Terminology",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockModelConfig {
    pub provider: String,
    pub model_name: String,
    pub role: ModelRole,
    pub temperature: f32,
    pub max_tokens: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationMode {
    SelectedChapters,
    AutoTranslateEpub,
}

impl TranslationMode {
    pub fn label(&self) -> &'static str {
        match self {
            TranslationMode::SelectedChapters => "Selected Chapters",
            TranslationMode::AutoTranslateEpub => "Auto Translate to End of EPUB",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockTranslationTask {
    pub is_running: bool,
    pub is_paused: bool,
    pub mode: TranslationMode,
    pub current_stage: WorkflowStage,
    pub stage_states: [StepState; 6],
    pub current_batch: usize,
    pub total_batches: usize,
    pub volume_progress_pct: u16,
    pub chapter_progress_pct: u16,
    pub active_model: MockModelConfig,
    pub current_operation: String,
    pub session_tokens: usize,
    pub session_cost_usd: f32,
    pub budget_limit_usd: Option<f32>,
    pub budget_unlimited: bool,
}
