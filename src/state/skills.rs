#[derive(Debug, Clone)]
pub struct MockSkillDoc {
    pub filename: String,
    pub title: String,
    pub category: String, // Shared / Series / Extracted
    pub scope: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct MockPendingDecision {
    pub id: String,
    pub original_term: String,
    pub suggestions: Vec<String>,
    pub selected_index: usize,
    pub custom_input: String,
    pub reasoning: String,
    pub scope: String,
    pub is_confirmed: bool,
    pub is_deferred: bool,
}
