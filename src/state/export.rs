#[derive(Debug, Clone)]
pub struct ValidationItem {
    pub name: String,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct MockImageResourceInfo {
    pub total: usize,
    pub original: usize,
    pub replaced: usize,
    pub invalid: usize,
    pub unused_edits: usize,
}

#[derive(Debug, Clone)]
pub struct MockExportStatus {
    pub is_validating: bool,
    pub checklist: Vec<ValidationItem>,
    pub images: MockImageResourceInfo,
    pub staging_path: String,
    pub edit_folder_path: String,
    pub output_path: String,
    pub last_validation_time: String,
    pub can_export: bool,
    pub errors: Vec<String>,
}

impl MockExportStatus {
    pub fn all_passed(&self) -> bool {
        self.checklist.iter().all(|item| item.passed) && self.images.invalid == 0
    }
}
