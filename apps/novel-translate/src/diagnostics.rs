use nt_tui::{
    DiagnosticComponent, DiagnosticEvent, DiagnosticFinalError, DiagnosticSeverity,
    DiagnosticStore, TuiError, TuiErrorKind,
};
use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;
use tracing::{Event, Subscriber, field::Field, field::Visit};
use tracing_subscriber::{
    layer::{Context, Layer},
    prelude::*,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SafeCode {
    CommandRequired,
    ArgumentsInvalid,
    HelpVersionConflict,
    TuiFailure(TuiErrorKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CatalogEntry {
    pub(crate) code: &'static str,
    pub(crate) severity: &'static str,
    pub(crate) message: &'static str,
    pub(crate) next_action: Option<&'static str>,
    pub(crate) text: &'static str,
    pub(crate) json_stderr: &'static str,
}

pub(crate) const fn catalog(code: SafeCode) -> CatalogEntry {
    match code {
        SafeCode::CommandRequired => CatalogEntry {
            code: "cli.command_required",
            severity: "error",
            message: "Cần cung cấp một lệnh.",
            next_action: None,
            text: "Cần cung cấp một lệnh khi dùng chế độ không tương tác. Dùng --help để xem trợ giúp.\n",
            json_stderr: "Thiếu lệnh CLI.\n",
        },
        SafeCode::ArgumentsInvalid => CatalogEntry {
            code: "cli.arguments_invalid",
            severity: "error",
            message: "Tham số không hợp lệ.",
            next_action: None,
            text: "Tham số CLI không hợp lệ. Dùng --help để xem trợ giúp.\n",
            json_stderr: "Tham số CLI không hợp lệ.\n",
        },
        SafeCode::HelpVersionConflict => CatalogEntry {
            code: "cli.help_version_conflict",
            severity: "error",
            message: "Không thể dùng đồng thời --help và --version.",
            next_action: None,
            text: "Không thể dùng đồng thời --help và --version.\n",
            json_stderr: "Không thể dùng đồng thời --help và --version.\n",
        },
        SafeCode::TuiFailure(TuiErrorKind::TerminalSetup) => CatalogEntry {
            code: "tui.terminal_setup_failed",
            severity: "error",
            message: "Không thể khởi tạo giao diện terminal.",
            next_action: None,
            text: "Không thể khởi tạo giao diện terminal.\n",
            json_stderr: "Không thể khởi tạo giao diện terminal.\n",
        },
        SafeCode::TuiFailure(TuiErrorKind::EventRead) => CatalogEntry {
            code: "tui.event_read_failed",
            severity: "error",
            message: "Không thể đọc sự kiện từ terminal.",
            next_action: None,
            text: "Không thể đọc sự kiện từ terminal.\n",
            json_stderr: "Không thể đọc sự kiện từ terminal.\n",
        },
        SafeCode::TuiFailure(TuiErrorKind::Render) => CatalogEntry {
            code: "tui.render_failed",
            severity: "error",
            message: "Không thể hiển thị giao diện terminal.",
            next_action: None,
            text: "Không thể hiển thị giao diện terminal.\n",
            json_stderr: "Không thể hiển thị giao diện terminal.\n",
        },
        SafeCode::TuiFailure(TuiErrorKind::TerminalCleanup) => CatalogEntry {
            code: "tui.terminal_cleanup_failed",
            severity: "error",
            message: "Không thể khôi phục terminal hoàn toàn.",
            next_action: None,
            text: "Không thể khôi phục terminal hoàn toàn.\n",
            json_stderr: "Không thể khôi phục terminal hoàn toàn.\n",
        },
        SafeCode::TuiFailure(TuiErrorKind::Panic) => CatalogEntry {
            code: "tui.panic",
            severity: "error",
            message: "Giao diện terminal dừng do lỗi nội bộ.",
            next_action: None,
            text: "Giao diện terminal dừng do lỗi nội bộ.\n",
            json_stderr: "Giao diện terminal dừng do lỗi nội bộ.\n",
        },
    }
}

pub(crate) fn record_panic(store: &DiagnosticStore, _payload: &(dyn Any + Send)) {
    store.record(
        DiagnosticSeverity::Error,
        DiagnosticComponent::Tui,
        DiagnosticEvent::Panic,
    );
}

static PANIC_HOOK_LOCK: Mutex<()> = Mutex::new(());

fn discard_panic_payload(payload: Box<dyn Any + Send>) {
    // A panic payload destructor is arbitrary code. Keep the safe hook installed
    // while dropping it and contain a second panic; its payload is intentionally
    // forgotten to avoid running another potentially panicking destructor.
    if let Err(nested_payload) = catch_unwind(AssertUnwindSafe(|| drop(payload))) {
        std::mem::forget(nested_payload);
    }
}

pub(crate) fn with_safe_panic_hook<R>(
    store: DiagnosticStore,
    operation: impl FnOnce() -> R,
) -> Result<R, ()> {
    let _guard = PANIC_HOOK_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        record_panic(&store, panic_info.payload());
    }));
    let outcome = catch_unwind(AssertUnwindSafe(operation));
    let outcome = match outcome {
        Ok(value) => Ok(value),
        Err(payload) => {
            discard_panic_payload(payload);
            Err(())
        }
    };
    std::panic::set_hook(previous_hook);
    outcome
}

pub(crate) fn render_tui_error(store: &DiagnosticStore, error: TuiError) -> String {
    let primary_event = event_for_kind(error.kind());
    let cleanup = error
        .cleanup_kind()
        .map(|kind| (DiagnosticComponent::Terminal, event_for_kind(kind)));
    store.record_final_error(DiagnosticFinalError {
        primary_component: DiagnosticComponent::Terminal,
        primary_event,
        cleanup,
    });

    let mut output = catalog(SafeCode::TuiFailure(error.kind())).text.to_owned();
    if let Some((_, cleanup_event)) = cleanup
        && cleanup_event != primary_event
    {
        let cleanup_kind = kind_for_event(cleanup_event);
        output.push_str(catalog(SafeCode::TuiFailure(cleanup_kind)).text);
    }
    output
}

fn event_for_kind(kind: TuiErrorKind) -> DiagnosticEvent {
    match kind {
        TuiErrorKind::TerminalSetup => DiagnosticEvent::TerminalSetupFailed,
        TuiErrorKind::EventRead => DiagnosticEvent::EventReadFailed,
        TuiErrorKind::Render => DiagnosticEvent::RenderFailed,
        TuiErrorKind::TerminalCleanup => DiagnosticEvent::TerminalCleanupFailed,
        TuiErrorKind::Panic => DiagnosticEvent::Panic,
    }
}

fn kind_for_event(event: DiagnosticEvent) -> TuiErrorKind {
    match event {
        DiagnosticEvent::TerminalSetupFailed => TuiErrorKind::TerminalSetup,
        DiagnosticEvent::EventReadFailed => TuiErrorKind::EventRead,
        DiagnosticEvent::RenderFailed => TuiErrorKind::Render,
        DiagnosticEvent::TerminalCleanupFailed => TuiErrorKind::TerminalCleanup,
        DiagnosticEvent::Panic => TuiErrorKind::Panic,
        DiagnosticEvent::StartupFailed
        | DiagnosticEvent::ArgumentsInvalid
        | DiagnosticEvent::TerminalUnavailable => TuiErrorKind::TerminalSetup,
    }
}

pub(crate) fn with_safe_subscriber<R>(store: DiagnosticStore, operation: impl FnOnce() -> R) -> R {
    let subscriber = tracing_subscriber::registry().with(SafeDiagnosticsLayer { store });
    tracing::subscriber::with_default(subscriber, operation)
}

#[derive(Clone)]
struct SafeDiagnosticsLayer {
    store: DiagnosticStore,
}

impl<S> Layer<S> for SafeDiagnosticsLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let mut visitor = SafeFieldVisitor::default();
        event.record(&mut visitor);
        if visitor.rejected {
            return;
        }
        let (Some(component), Some(event_code)) = (visitor.component, visitor.event) else {
            return;
        };

        let severity = match *event.metadata().level() {
            tracing::Level::ERROR => DiagnosticSeverity::Error,
            tracing::Level::WARN => DiagnosticSeverity::Warn,
            tracing::Level::INFO => DiagnosticSeverity::Info,
            tracing::Level::DEBUG => DiagnosticSeverity::Debug,
            tracing::Level::TRACE => DiagnosticSeverity::Trace,
        };
        self.store.record(severity, component, event_code);
    }
}

#[derive(Default)]
struct SafeFieldVisitor {
    component: Option<DiagnosticComponent>,
    event: Option<DiagnosticEvent>,
    rejected: bool,
}

impl SafeFieldVisitor {
    fn accept_component(&mut self, value: &str) {
        let Some(parsed) = parse_component(value) else {
            self.rejected = true;
            return;
        };
        if self.component.is_some_and(|existing| existing != parsed) {
            self.rejected = true;
            return;
        }
        self.component = Some(parsed);
    }

    fn accept_event(&mut self, value: &str) {
        let Some(parsed) = parse_event(value) else {
            self.rejected = true;
            return;
        };
        if self.event.is_some_and(|existing| existing != parsed) {
            self.rejected = true;
            return;
        }
        self.event = Some(parsed);
    }
}

impl Visit for SafeFieldVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "component" => self.accept_component(value),
            "event" => self.accept_event(value),
            _ => {}
        }
    }

    fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}
}

fn parse_component(value: &str) -> Option<DiagnosticComponent> {
    match value {
        "application" => Some(DiagnosticComponent::Application),
        "cli" => Some(DiagnosticComponent::Cli),
        "terminal" => Some(DiagnosticComponent::Terminal),
        "tui" => Some(DiagnosticComponent::Tui),
        _ => None,
    }
}

fn parse_event(value: &str) -> Option<DiagnosticEvent> {
    match value {
        "startup_failed" => Some(DiagnosticEvent::StartupFailed),
        "arguments_invalid" => Some(DiagnosticEvent::ArgumentsInvalid),
        "terminal_unavailable" => Some(DiagnosticEvent::TerminalUnavailable),
        "terminal_setup_failed" => Some(DiagnosticEvent::TerminalSetupFailed),
        "event_read_failed" => Some(DiagnosticEvent::EventReadFailed),
        "render_failed" => Some(DiagnosticEvent::RenderFailed),
        "terminal_cleanup_failed" => Some(DiagnosticEvent::TerminalCleanupFailed),
        "panic" => Some(DiagnosticEvent::Panic),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{SafeCode, catalog, record_panic, render_tui_error, with_safe_subscriber};
    use crate::cli::{Decision, FailureKind};
    use nt_tui::{
        DiagnosticComponent, DiagnosticEvent, DiagnosticSeverity, DiagnosticStore, TuiError,
        TuiErrorKind,
    };
    use serde_json::Value;
    use std::error::Error;
    use std::fmt;
    use std::process::Command;
    use std::sync::{Arc, Mutex};

    const TOKEN_SENTINEL: &str = "fake-token-sentinel-123";
    const URL_SENTINEL: &str =
        "https://user:fake-url-secret@example.invalid/path?token=fake-query-secret";
    const HEADER_SENTINEL: &str = "Authorization: Bearer fake-header-secret";

    #[derive(Debug)]
    struct NestedSecretError {
        message: String,
        source: Option<Box<NestedSecretError>>,
    }

    impl fmt::Display for NestedSecretError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(&self.message)
        }
    }

    impl Error for NestedSecretError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            self.source
                .as_deref()
                .map(|source| source as &(dyn Error + 'static))
        }
    }

    fn nested_secret_error() -> NestedSecretError {
        NestedSecretError {
            message: format!("outer {URL_SENTINEL}"),
            source: Some(Box::new(NestedSecretError {
                message: format!("inner {TOKEN_SENTINEL} {HEADER_SENTINEL}"),
                source: None,
            })),
        }
    }

    #[test]
    fn tracing_records_only_allowlisted_codes_and_never_formats_other_fields() {
        let store = DiagnosticStore::default();
        let source = nested_secret_error();
        with_safe_subscriber(store.clone(), || {
            tracing::error!(
                component = "tui",
                event = "render_failed",
                message = %TOKEN_SENTINEL,
                url = %URL_SENTINEL,
                authorization = %HEADER_SENTINEL,
                error = ?source,
            );
            tracing::warn!(component = "untrusted", event = "render_failed", message = %TOKEN_SENTINEL);
            tracing::warn!(component = "tui", event = "user_supplied_event", message = %TOKEN_SENTINEL);
        });

        let snapshot = store.snapshot();
        assert_eq!(snapshot.records.len(), 1);
        assert_eq!(snapshot.records[0].component, DiagnosticComponent::Tui);
        assert_eq!(snapshot.records[0].event, DiagnosticEvent::RenderFailed);
        assert_eq!(snapshot.records[0].severity, DiagnosticSeverity::Error);
        let sink = format!("{snapshot:?}");
        for secret in [TOKEN_SENTINEL, URL_SENTINEL, HEADER_SENTINEL] {
            assert!(!sink.contains(secret));
        }
    }

    #[test]
    fn nested_error_debug_display_and_chain_are_not_exposed() {
        let source = nested_secret_error();
        let error = TuiError::from_source(TuiErrorKind::Render, &source);
        let exposed = format!("{error:?} {error} {:?}", Error::source(&error));
        for secret in [TOKEN_SENTINEL, URL_SENTINEL, HEADER_SENTINEL] {
            assert!(!exposed.contains(secret));
        }
        assert!(Error::source(&error).is_none());
    }

    #[test]
    fn panic_payload_is_replaced_by_safe_event_code() {
        let store = DiagnosticStore::default();
        let payload: Box<dyn std::any::Any + Send> = Box::new(format!(
            "panic {TOKEN_SENTINEL} {URL_SENTINEL} {HEADER_SENTINEL}"
        ));
        record_panic(&store, payload.as_ref());

        let sink = format!("{:?}", store.snapshot());
        assert!(sink.contains("Panic"));
        for secret in [TOKEN_SENTINEL, URL_SENTINEL, HEADER_SENTINEL] {
            assert!(!sink.contains(secret));
        }
    }

    const PANIC_DROP_CHILD_ENV: &str = "NOVEL_TRANSLATE_PANIC_DROP_CHILD";
    const PANIC_DROP_SENTINEL: &str = "P01_T06_SYNTHETIC_DESTRUCTOR_SENTINEL";
    const RESTORED_HOOK_MARKER: &str = "P01_T06_RESTORED_HOOK_MARKER";

    struct PanickingDrop;

    impl Drop for PanickingDrop {
        fn drop(&mut self) {
            panic!("{PANIC_DROP_SENTINEL}");
        }
    }

    #[test]
    fn panicking_payload_destructor_is_contained_and_previous_hook_is_restored() {
        if std::env::var_os(PANIC_DROP_CHILD_ENV).is_some() {
            let previous_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(|_| eprintln!("{RESTORED_HOOK_MARKER}")));

            let observed = Arc::new(Mutex::new(None::<DiagnosticStore>));
            let capture = observed.clone();
            let output =
                crate::bootstrap::run(["novel-translate-cli".into()], true, true, move |store| {
                    *capture
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(store);
                    std::panic::panic_any(PanickingDrop);
                });

            assert_eq!(output.exit_code, 3);
            assert!(output.stdout.is_empty());
            assert!(output.stderr.contains("Giao diện terminal dừng"));
            assert!(!output.stderr.contains(PANIC_DROP_SENTINEL));
            let store = observed
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone()
                .expect("launcher receives diagnostic store");
            let snapshot = store.snapshot();
            assert!(
                snapshot
                    .records
                    .iter()
                    .any(|record| record.event == DiagnosticEvent::Panic)
            );
            assert!(!format!("{snapshot:?}").contains(PANIC_DROP_SENTINEL));

            let _ = std::panic::catch_unwind(|| panic!("post-boundary panic"));
            std::panic::set_hook(previous_hook);
            return;
        }

        let output = Command::new(std::env::current_exe().expect("test executable path"))
            .args([
                "--exact",
                "diagnostics::tests::panicking_payload_destructor_is_contained_and_previous_hook_is_restored",
                "--nocapture",
            ])
            .env(PANIC_DROP_CHILD_ENV, "1")
            .output()
            .expect("spawn isolated panic-hook probe");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "child failed; stdout={stdout}; stderr={stderr}"
        );
        assert!(stderr.contains(RESTORED_HOOK_MARKER));
        assert!(!stdout.contains(PANIC_DROP_SENTINEL));
        assert!(!stderr.contains(PANIC_DROP_SENTINEL));
    }

    #[test]
    fn cleanup_failure_stays_secondary_to_primary_and_both_codes_are_safe() {
        let store = DiagnosticStore::default();
        let error = TuiError::with_cleanup(TuiErrorKind::Render, TuiErrorKind::TerminalCleanup);
        let output = render_tui_error(&store, error);
        let snapshot = store.snapshot();

        assert!(output.starts_with(catalog(SafeCode::TuiFailure(TuiErrorKind::Render)).text));
        assert!(output.contains("Không thể khôi phục terminal"));
        assert_eq!(
            snapshot.final_error.unwrap().primary_event,
            DiagnosticEvent::RenderFailed
        );
        assert_eq!(
            snapshot.final_error.unwrap().cleanup,
            Some((
                DiagnosticComponent::Terminal,
                DiagnosticEvent::TerminalCleanupFailed
            ))
        );
    }

    #[test]
    fn json_stderr_and_error_catalog_never_reflect_untrusted_argument_text() {
        let args = [
            "novel-translate-cli",
            "--json",
            TOKEN_SENTINEL,
            URL_SENTINEL,
            HEADER_SENTINEL,
        ]
        .into_iter()
        .map(std::ffi::OsString::from)
        .collect();
        let Decision::Failure(failure) = crate::cli::parse(args) else {
            panic!("secret-bearing unknown arguments must fail before command dispatch");
        };
        assert_eq!(failure.kind, FailureKind::ArgumentsInvalid);
        let (exit_code, stdout, stderr) = failure.render();
        let value: Value = serde_json::from_str(stdout.trim()).unwrap();

        assert_eq!(exit_code, 2);
        assert_eq!(value["issues"][0]["code"], "cli.arguments_invalid");
        assert_eq!(value["next_action"], Value::Null);
        for secret in [TOKEN_SENTINEL, URL_SENTINEL, HEADER_SENTINEL] {
            assert!(!stdout.contains(secret));
            assert!(!stderr.contains(secret));
        }
        assert_eq!(catalog(SafeCode::ArgumentsInvalid).next_action, None);
    }
}
