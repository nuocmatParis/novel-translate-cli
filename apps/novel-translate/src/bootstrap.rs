use crate::cli::{self, Decision};
use crate::diagnostics;
use nt_tui::{DiagnosticStore, TuiError};
use std::ffi::OsString;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ProcessOutput {
    pub(crate) exit_code: i32,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) fn run<I, F>(
    args: I,
    stdin_is_terminal: bool,
    stdout_is_terminal: bool,
    launch_tui: F,
) -> ProcessOutput
where
    I: IntoIterator<Item = OsString>,
    F: FnOnce(DiagnosticStore) -> Result<(), TuiError>,
{
    match cli::parse(args.into_iter().collect()) {
        Decision::Help(text) | Decision::Version(text) => ProcessOutput {
            exit_code: 0,
            stdout: text,
            stderr: String::new(),
        },
        Decision::Failure(failure) => {
            let (exit_code, stdout, stderr) = failure.render();
            ProcessOutput {
                exit_code,
                stdout,
                stderr,
            }
        }
        Decision::LaunchTui if !stdin_is_terminal || !stdout_is_terminal => ProcessOutput {
            exit_code: 3,
            stdout: String::new(),
            stderr: "Cần terminal tương tác cho cả stdin và stdout để mở TUI; dùng --help để xem trợ giúp.\n"
                .to_owned(),
        },
        Decision::LaunchTui => {
            let diagnostic_store = DiagnosticStore::default();
            let result = diagnostics::with_safe_panic_hook(diagnostic_store.clone(), || {
                diagnostics::with_safe_subscriber(diagnostic_store.clone(), || {
                    launch_tui(diagnostic_store.clone())
                })
            });
            match result {
                Ok(Ok(())) => ProcessOutput {
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                },
                Ok(Err(error)) => operational_failure(&diagnostic_store, error),
                Err(()) => operational_failure(
                    &diagnostic_store,
                    TuiError::new(nt_tui::TuiErrorKind::Panic),
                ),
            }
        }
    }
}

fn operational_failure(store: &DiagnosticStore, error: TuiError) -> ProcessOutput {
    ProcessOutput {
        exit_code: 3,
        stdout: String::new(),
        stderr: diagnostics::render_tui_error(store, error),
    }
}

#[cfg(test)]
mod tests {
    use super::{ProcessOutput, run};
    use nt_tui::{DiagnosticEvent, DiagnosticStore, TuiError, TuiErrorKind};
    use std::ffi::OsString;
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    const PROCESS_PROBE_ENV: &str = "NT_TUI_FAILURE_PROCESS_PROBE";
    const PROCESS_SECRET_SENTINEL: &str = "synthetic-tui-panic-secret-48cf";
    const PROCESS_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

    fn args(tokens: &[&str]) -> Vec<OsString> {
        std::iter::once("novel-translate-cli")
            .chain(tokens.iter().copied())
            .map(OsString::from)
            .collect()
    }

    #[test]
    fn help_and_non_tty_paths_never_launch_tui() {
        let calls = AtomicUsize::new(0);
        let help = run(args(&["--help"]), false, false, |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
        let non_tty = run(args(&[]), true, false, |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        assert_eq!(help.exit_code, 0);
        assert_eq!(non_tty.exit_code, 3);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn interactive_mode_invokes_exactly_one_tui_entrypoint() {
        let calls = AtomicUsize::new(0);
        let output = run(args(&[]), true, true, |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        assert_eq!(
            output,
            ProcessOutput {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            }
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn shell_failure_maps_to_operational_exit_without_exposing_details() {
        let output = run(args(&[]), true, true, |_| {
            Err(TuiError::new(TuiErrorKind::TerminalSetup))
        });
        assert_eq!(output.exit_code, 3);
        assert!(output.stderr.contains("Không thể khởi tạo"));
    }

    #[test]
    fn tui_failure_keeps_primary_and_cleanup_diagnostics_safe() {
        let output = run(args(&[]), true, true, |_| {
            Err(TuiError::with_cleanup(
                TuiErrorKind::Render,
                TuiErrorKind::TerminalCleanup,
            ))
        });

        assert_eq!(output.exit_code, 3);
        assert!(output.stdout.is_empty());
        assert!(
            output
                .stderr
                .starts_with("Không thể hiển thị giao diện terminal.")
        );
        assert!(
            output
                .stderr
                .contains("Không thể khôi phục terminal hoàn toàn.")
        );
    }

    #[test]
    fn panic_payload_is_hidden_and_only_safe_marker_reaches_sink() {
        let observed = Arc::new(Mutex::new(None::<DiagnosticStore>));
        let capture = observed.clone();
        let sentinel = "fake-panic-token-sentinel";
        let output = run(args(&[]), true, true, move |store| {
            *capture
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(store);
            panic!("panic payload contains {sentinel}");
        });

        let store = observed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
            .expect("interactive launcher receives the configured sink");
        let snapshot = store.snapshot();
        assert_eq!(output.exit_code, 3);
        assert!(output.stdout.is_empty());
        assert!(output.stderr.contains("Giao diện terminal dừng"));
        assert!(!output.stderr.contains(sentinel));
        assert!(
            snapshot
                .records
                .iter()
                .any(|record| record.event == DiagnosticEvent::Panic)
        );
        assert!(!format!("{snapshot:?}").contains(sentinel));
    }

    #[test]
    fn tui_failure_process_probe_child() {
        let Ok(mode) = std::env::var(PROCESS_PROBE_ENV) else {
            return;
        };

        let output = match mode.as_str() {
            "cleanup-error" => run(args(&[]), true, true, |_| {
                Err(TuiError::with_cleanup(
                    TuiErrorKind::Render,
                    TuiErrorKind::TerminalCleanup,
                ))
            }),
            "panic" => run(args(&[]), true, true, |_| {
                panic!("synthetic lifecycle panic contains {PROCESS_SECRET_SENTINEL}");
            }),
            other => panic!("unknown test-only subprocess probe mode: {other}"),
        };

        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        stdout
            .write_all(output.stdout.as_bytes())
            .expect("subprocess stdout is writable");
        stdout.flush().expect("subprocess stdout is flushed");

        let stderr = std::io::stderr();
        let mut stderr = stderr.lock();
        stderr
            .write_all(output.stderr.as_bytes())
            .expect("subprocess stderr is writable");
        stderr.flush().expect("subprocess stderr is flushed");
        std::process::exit(output.exit_code);
    }

    #[test]
    fn subprocess_lifecycle_failures_exit_nonzero_and_hide_panic_payload() {
        let cleanup = run_failure_probe("cleanup-error");
        assert_eq!(cleanup.status.code(), Some(3));
        assert!(
            !String::from_utf8_lossy(&cleanup.stdout)
                .contains("Không thể hiển thị giao diện terminal.")
        );
        let cleanup_stderr = String::from_utf8(cleanup.stderr).expect("diagnostic is UTF-8");
        assert!(cleanup_stderr.starts_with("Không thể hiển thị giao diện terminal."));
        assert!(cleanup_stderr.contains("Không thể khôi phục terminal hoàn toàn."));
        assert!(!cleanup_stderr.contains(PROCESS_SECRET_SENTINEL));

        let panic = run_failure_probe("panic");
        assert_eq!(panic.status.code(), Some(3));
        assert!(!String::from_utf8_lossy(&panic.stdout).contains("Giao diện terminal dừng"));
        assert!(!String::from_utf8_lossy(&panic.stdout).contains(PROCESS_SECRET_SENTINEL));
        let panic_stderr = String::from_utf8(panic.stderr).expect("diagnostic is UTF-8");
        assert!(panic_stderr.contains("Giao diện terminal dừng"));
        assert!(!panic_stderr.contains(PROCESS_SECRET_SENTINEL));
    }

    fn run_failure_probe(mode: &str) -> std::process::Output {
        let executable = std::env::current_exe().expect("test executable is available");
        let mut child = Command::new(executable)
            .args([
                "--exact",
                "bootstrap::tests::tui_failure_process_probe_child",
                "--nocapture",
            ])
            .env(PROCESS_PROBE_ENV, mode)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("fault probe subprocess starts");
        let deadline = Instant::now() + PROCESS_PROBE_TIMEOUT;

        let status = loop {
            if let Some(status) = child.try_wait().expect("subprocess status is readable") {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let status = child.wait().expect("timed-out subprocess is reaped");
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                child
                    .stdout
                    .take()
                    .expect("subprocess stdout is captured")
                    .read_to_end(&mut stdout)
                    .expect("subprocess stdout is readable");
                child
                    .stderr
                    .take()
                    .expect("subprocess stderr is captured")
                    .read_to_end(&mut stderr)
                    .expect("subprocess stderr is readable");
                panic!(
                    "fault probe subprocess timed out after {PROCESS_PROBE_TIMEOUT:?}; status={status:?}; stdout={:?}; stderr={:?}",
                    String::from_utf8_lossy(&stdout),
                    String::from_utf8_lossy(&stderr)
                );
            }
            thread::sleep(Duration::from_millis(10));
        };

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        child
            .stdout
            .take()
            .expect("subprocess stdout is captured")
            .read_to_end(&mut stdout)
            .expect("subprocess stdout is readable");
        child
            .stderr
            .take()
            .expect("subprocess stderr is captured")
            .read_to_end(&mut stderr)
            .expect("subprocess stderr is readable");
        std::process::Output {
            status,
            stdout,
            stderr,
        }
    }
}
