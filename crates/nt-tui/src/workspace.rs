use std::path::Path;

use nt_application::workspace::{WorkspaceError, WorkspaceMetadata};

/// Application-owned workspace operations consumed by the Overview form.
pub trait WorkspaceCommands: Send + Sync + 'static {
    fn create_workspace(&self, path: &Path) -> Result<WorkspaceMetadata, WorkspaceError>;

    fn open_workspace(&self, path: &Path) -> Result<WorkspaceMetadata, WorkspaceError>;
}
