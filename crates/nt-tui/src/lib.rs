//! Terminal lifecycle and the Phase 01 presentation shell live here.

mod diagnostics;
mod error;
mod input;
mod navigation;
mod rendering;
mod screens;
mod state;
mod terminal;
#[cfg(windows)]
mod windows_input;

pub use diagnostics::{
    DIAGNOSTIC_CAPACITY, DiagnosticComponent, DiagnosticEvent, DiagnosticFinalError,
    DiagnosticRecord, DiagnosticSeverity, DiagnosticSnapshot, DiagnosticStore,
};
pub use error::{TuiError, TuiErrorKind};
pub use terminal::run_shell;
