//! Terminal lifecycle and the Phase 01 presentation shell live here.

mod diagnostics;
mod error;
mod import_commands;
mod input;
mod navigation;
mod rendering;
mod screens;
mod state;
mod terminal;
#[cfg(windows)]
mod windows_input;
mod workspace;

pub use diagnostics::{
    DIAGNOSTIC_CAPACITY, DiagnosticComponent, DiagnosticEvent, DiagnosticFinalError,
    DiagnosticRecord, DiagnosticSeverity, DiagnosticSnapshot, DiagnosticStore,
};
pub use error::{TuiError, TuiErrorKind};
pub use import_commands::{
    BookCatalog, BookChange, BookEdit, BookError, BookPosition, BookSource, BookSummary,
    BookVersion, BookView, BookWorkspace, ClassificationTarget, VolumeCommands,
};
pub use terminal::run_shell;
pub use terminal::{run_shell_with_volumes, run_shell_with_workspace};
pub use workspace::{FolderPickerError, WorkspaceCommands};
