mod bootstrap;
mod cli;
mod diagnostics;

use nt_tui::{DiagnosticStore, TuiError};
use std::io::{self, IsTerminal, Write};

fn main() {
    let result = bootstrap::run(
        std::env::args_os(),
        io::stdin().is_terminal(),
        io::stdout().is_terminal(),
        launch_tui,
    );

    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    if !result.stdout.is_empty() {
        let _ = stdout.write_all(result.stdout.as_bytes());
        let _ = stdout.flush();
    }
    if !result.stderr.is_empty() {
        let _ = stderr.write_all(result.stderr.as_bytes());
        let _ = stderr.flush();
    }
    std::process::exit(result.exit_code);
}

// The app owns panic/logging configuration; nt-tui owns the terminal lifecycle.
fn launch_tui(_diagnostics: DiagnosticStore) -> Result<(), TuiError> {
    nt_tui::run_shell()
}
