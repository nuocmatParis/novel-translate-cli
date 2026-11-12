use std::path::Path;

use nt_application::workspace::{
    WorkspaceError, WorkspaceMetadata, create_workspace, open_workspace,
};
use nt_storage::SqliteWorkspaceRepository;
use nt_tui::WorkspaceCommands;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WorkspaceService;

impl WorkspaceCommands for WorkspaceService {
    fn create_workspace(&self, path: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
        create_workspace(&SqliteWorkspaceRepository, path)
    }

    fn open_workspace(&self, path: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
        open_workspace(&SqliteWorkspaceRepository, path)
    }
}
