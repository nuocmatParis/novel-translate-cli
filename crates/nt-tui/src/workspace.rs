use std::path::{Path, PathBuf};

use nt_application::workspace::{WorkspaceError, WorkspaceMetadata};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FolderPickerError {
    Busy,
    Unavailable,
    PermissionDenied,
}

/// Application-owned workspace operations consumed by the Overview form.
pub trait WorkspaceCommands: Send + Sync + 'static {
    fn create_workspace(&self, path: &Path) -> Result<WorkspaceMetadata, WorkspaceError>;

    fn open_workspace(&self, path: &Path) -> Result<WorkspaceMetadata, WorkspaceError>;

    /// Returns only the selected folder. Confirmation is a separate command.
    fn choose_workspace_folder(&self) -> Result<Option<PathBuf>, FolderPickerError> {
        Err(FolderPickerError::Unavailable)
    }
}
