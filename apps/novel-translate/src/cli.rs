use crate::diagnostics::{self, SafeCode};
use clap::{ArgAction, Parser};
use nt_application::workspace::WorkspaceError;
use nt_application::workspace::WorkspaceMetadata;
use serde::Serialize;
use serde_json::{Value, json};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "novel-translate-cli",
    bin_name = "novel-translate-cli",
    version = env!("CARGO_PKG_VERSION"),
    about = "CLI và TUI nền tảng cho Novel Translate",
    after_help = "Commands:\n  workspace create <path>\n  workspace open <path>"
)]
struct Arguments {
    /// Yêu cầu kết quả máy đọc cho lệnh.
    #[arg(long, action = ArgAction::Count, global = true)]
    json: u8,

    /// Không mở TUI và không chờ nhập từ terminal.
    #[arg(long = "non-interactive", action = ArgAction::Count, global = true)]
    non_interactive: u8,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Decision {
    Help(String),
    Version(String),
    LaunchTui,
    Workspace(WorkspaceCommand),
    Failure(CliFailure),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CliFailure {
    pub(crate) json: bool,
    pub(crate) command: Option<&'static str>,
    pub(crate) kind: FailureKind,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorkspaceCommand {
    pub(crate) operation: WorkspaceOperation,
    pub(crate) path: PathBuf,
    pub(crate) json: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkspaceOperation {
    Create,
    Open,
}

impl WorkspaceOperation {
    pub(crate) const fn canonical(self) -> &'static str {
        match self {
            Self::Create => "workspace create",
            Self::Open => "workspace open",
        }
    }

    pub(crate) const fn verb(self) -> &'static str {
        match self {
            Self::Create => "tạo",
            Self::Open => "mở",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FailureKind {
    CommandRequired,
    ArgumentsInvalid,
    HelpVersionConflict,
}

#[derive(Serialize)]
struct ErrorEnvelope {
    schema_version: u8,
    command: Option<String>,
    state: &'static str,
    task_id: Option<String>,
    scope: Option<Value>,
    result: Option<Value>,
    issues: Vec<Issue>,
    next_action: Option<&'static str>,
}

#[derive(Serialize)]
struct Issue {
    code: &'static str,
    severity: &'static str,
    message: &'static str,
}

pub(crate) fn parse(args: Vec<OsString>) -> Decision {
    let raw_tokens: Vec<&OsStr> = args.iter().skip(1).map(OsString::as_os_str).collect();
    let string_tokens: Vec<Option<&str>> = raw_tokens.iter().map(|value| value.to_str()).collect();
    let json_requested = string_tokens.contains(&Some("--json"));

    let json_count = string_tokens
        .iter()
        .filter(|token| **token == Some("--json"))
        .count();
    let non_interactive_count = string_tokens
        .iter()
        .filter(|token| **token == Some("--non-interactive"))
        .count();
    let has_help = string_tokens
        .iter()
        .any(|token| matches!(token, Some("--help" | "-h")));
    let has_version = string_tokens
        .iter()
        .any(|token| matches!(token, Some("--version" | "-V")));
    let malformed_mode_flag = string_tokens.iter().any(|token| {
        token.is_some_and(|token| {
            token.starts_with("--json=") || token.starts_with("--non-interactive=")
        })
    });

    // These global switches are intentionally strict. In particular, don't
    // let Clap's raw diagnostic echo an arbitrary argument or secret value.
    if has_help && has_version {
        return Decision::Failure(CliFailure {
            json: false,
            command: None,
            kind: FailureKind::HelpVersionConflict,
        });
    }
    if malformed_mode_flag || json_count > 1 || non_interactive_count > 1 {
        return invalid_arguments(json_requested);
    }
    let tokens: Vec<&OsStr> = raw_tokens
        .iter()
        .copied()
        .filter(|token| !matches!(token.to_str(), Some("--json" | "--non-interactive")))
        .collect();
    if tokens
        .first()
        .is_some_and(|token| *token == OsStr::new("workspace"))
    {
        return parse_workspace_command(&tokens, json_requested);
    }
    if tokens.iter().any(|token| token.to_str().is_none())
        || tokens
            .iter()
            .any(|token| !matches!(token.to_str(), Some("--help" | "-h" | "--version" | "-V")))
    {
        return invalid_arguments(json_requested);
    }

    match Arguments::try_parse_from(args) {
        Ok(parsed) => {
            if parsed.json > 0 || parsed.non_interactive > 0 {
                Decision::Failure(CliFailure {
                    json: parsed.json > 0,
                    command: None,
                    kind: FailureKind::CommandRequired,
                })
            } else {
                Decision::LaunchTui
            }
        }
        Err(error) => match error.kind() {
            clap::error::ErrorKind::DisplayHelp => Decision::Help(error.to_string()),
            clap::error::ErrorKind::DisplayVersion => Decision::Version(error.to_string()),
            _ => invalid_arguments(json_requested),
        },
    }
}

fn parse_workspace_command(tokens: &[&OsStr], json: bool) -> Decision {
    if tokens.len() < 2 {
        return invalid_arguments_for(json, Some("workspace"));
    }
    let operation = match tokens[1].to_str() {
        Some("create") => WorkspaceOperation::Create,
        Some("open") => WorkspaceOperation::Open,
        _ => return invalid_arguments_for(json, Some("workspace")),
    };
    if tokens.len() != 3 || tokens[2].is_empty() || is_option_like_path(tokens[2]) {
        return invalid_arguments_for(json, Some(operation.canonical()));
    }
    Decision::Workspace(WorkspaceCommand {
        operation,
        path: PathBuf::from(tokens[2]),
        json,
    })
}

fn is_option_like_path(path: &OsStr) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        path.encode_wide().next() == Some(u16::from(b'-'))
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        path.as_bytes().starts_with(b"-")
    }
    #[cfg(not(any(unix, windows)))]
    {
        path.to_str().is_some_and(|path| path.starts_with('-'))
    }
}

impl CliFailure {
    pub(crate) fn render(&self) -> (i32, String, String) {
        if self.kind == FailureKind::HelpVersionConflict {
            return (
                2,
                String::new(),
                diagnostics::catalog(SafeCode::HelpVersionConflict)
                    .text
                    .to_owned(),
            );
        }
        if self.json {
            let entry = match self.kind {
                FailureKind::CommandRequired => diagnostics::catalog(SafeCode::CommandRequired),
                FailureKind::ArgumentsInvalid => diagnostics::catalog(SafeCode::ArgumentsInvalid),
                FailureKind::HelpVersionConflict => {
                    diagnostics::catalog(SafeCode::HelpVersionConflict)
                }
            };
            let envelope = ErrorEnvelope {
                schema_version: 1,
                command: self.command.map(str::to_owned),
                state: "failed",
                task_id: None,
                scope: None,
                result: None,
                issues: vec![Issue {
                    code: entry.code,
                    severity: entry.severity,
                    message: entry.message,
                }],
                next_action: entry.next_action,
            };
            let mut stdout =
                serde_json::to_string(&envelope).expect("fixed CLI error envelope is serializable");
            stdout.push('\n');
            (2, stdout, entry.json_stderr.to_owned())
        } else {
            let code = match self.kind {
                FailureKind::CommandRequired => SafeCode::CommandRequired,
                FailureKind::ArgumentsInvalid => SafeCode::ArgumentsInvalid,
                FailureKind::HelpVersionConflict => SafeCode::HelpVersionConflict,
            };
            (2, String::new(), diagnostics::catalog(code).text.to_owned())
        }
    }
}

fn invalid_arguments(json: bool) -> Decision {
    invalid_arguments_for(json, None)
}

fn invalid_arguments_for(json: bool, command: Option<&'static str>) -> Decision {
    Decision::Failure(CliFailure {
        json,
        command,
        kind: FailureKind::ArgumentsInvalid,
    })
}

pub(crate) fn render_workspace_result(
    command: &WorkspaceCommand,
    result: Result<WorkspaceMetadata, WorkspaceError>,
) -> (i32, String, String) {
    let command_name = command.operation.canonical();
    match result {
        Ok(metadata) => {
            if command.json {
                let envelope = ErrorEnvelope {
                    schema_version: 1,
                    command: Some(command_name.to_owned()),
                    state: "complete",
                    task_id: None,
                    scope: Some(json!({
                        "kind": "workspace",
                        "volume_ids": [],
                        "chapter_ids": []
                    })),
                    result: Some(json!({
                        "series_id": metadata.series.id.as_str(),
                        "schema_version": metadata.schema_version,
                        "current_generation": metadata.current_generation.as_str()
                    })),
                    issues: Vec::new(),
                    next_action: None,
                };
                let mut stdout =
                    serde_json::to_string(&envelope).expect("workspace result is serializable");
                stdout.push('\n');
                (0, stdout, String::new())
            } else {
                (
                    0,
                    format!(
                        "Đã {} workspace.\nSeries: {}\nSchema: {}\nGeneration: {}\n",
                        command.operation.verb(),
                        metadata.series.id.as_str(),
                        metadata.schema_version,
                        metadata.current_generation.as_str()
                    ),
                    String::new(),
                )
            }
        }
        Err(error) => {
            let (code, message, next_action) = workspace_issue(&error);
            if command.json {
                let envelope = ErrorEnvelope {
                    schema_version: 1,
                    command: Some(command_name.to_owned()),
                    state: "failed",
                    task_id: None,
                    scope: Some(json!({
                        "kind": "workspace",
                        "volume_ids": [],
                        "chapter_ids": []
                    })),
                    result: None,
                    issues: vec![Issue {
                        code,
                        severity: "error",
                        message,
                    }],
                    next_action,
                };
                let mut stdout =
                    serde_json::to_string(&envelope).expect("workspace error is serializable");
                stdout.push('\n');
                (
                    3,
                    stdout,
                    format!(
                        "Không thể {} workspace. {message}\n",
                        command.operation.verb()
                    ),
                )
            } else {
                (
                    3,
                    String::new(),
                    format!(
                        "Không thể {} workspace. {message}\n",
                        command.operation.verb()
                    ),
                )
            }
        }
    }
}

fn workspace_issue(error: &WorkspaceError) -> (&'static str, &'static str, Option<&'static str>) {
    match error {
        WorkspaceError::InvalidPath => (
            "workspace.invalid_path",
            "Đường dẫn workspace không hợp lệ.",
            Some("provide_arguments"),
        ),
        WorkspaceError::UnsupportedFilesystem => (
            "workspace.unsupported_filesystem",
            "Workspace phải nằm trên ổ cục bộ được hỗ trợ.",
            Some("choose_local_workspace"),
        ),
        WorkspaceError::AlreadyExists => (
            "workspace.already_exists",
            "Workspace đã tồn tại. Hãy mở workspace hiện có thay vì tạo lại.",
            Some("open_workspace"),
        ),
        WorkspaceError::DirectoryNotEmpty => (
            "workspace.directory_not_empty",
            "Thư mục đích không trống. Hãy chọn thư mục mới hoặc thư mục rỗng.",
            Some("choose_empty_directory"),
        ),
        WorkspaceError::NotFound => (
            "workspace.not_found",
            "Không tìm thấy workspace. Kiểm tra đường dẫn hoặc tạo workspace mới.",
            Some("check_path_or_create_workspace"),
        ),
        WorkspaceError::PermissionDenied => (
            "workspace.permission_denied",
            "Không đủ quyền truy cập workspace. Kiểm tra quyền tệp/thư mục rồi thử lại.",
            Some("check_permissions"),
        ),
        WorkspaceError::Busy => (
            "workspace.busy",
            "Workspace đang được tiến trình khác sử dụng. Đóng tiến trình đó rồi thử lại.",
            Some("close_other_process_and_retry"),
        ),
        WorkspaceError::LegacyGoWorkspace => (
            "workspace.legacy_go_workspace",
            "Đây là workspace Go cũ. Tạo workspace Rust mới rồi nhập lại EPUB nguồn.",
            Some("create_rust_workspace"),
        ),
        WorkspaceError::UnsupportedFormat => (
            "workspace.unsupported_format",
            "Định dạng workspace không được hỗ trợ. Dữ liệu hiện có được giữ nguyên.",
            Some("open_with_compatible_version"),
        ),
        WorkspaceError::NewerSchema { .. } => (
            "workspace.newer_schema",
            "Workspace cần phiên bản ứng dụng mới hơn. Mở bằng phiên bản hỗ trợ schema này.",
            Some("use_newer_application"),
        ),
        WorkspaceError::CorruptWorkspace => (
            "workspace.corrupt",
            "Dữ liệu workspace không qua kiểm tra toàn vẹn. Khôi phục từ bản sao lưu đã kiểm chứng; giữ nguyên thư mục hiện tại để kiểm tra.",
            Some("restore_verified_workspace_backup"),
        ),
        WorkspaceError::StorageFailure => (
            "workspace.storage_failed",
            "Không thể đọc hoặc ghi workspace an toàn. Kiểm tra ổ đĩa và thử mở lại; dữ liệu cũ vẫn được giữ.",
            Some("check_storage_and_retry"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{Decision, FailureKind, is_option_like_path, parse, workspace_issue};
    use std::ffi::OsString;

    fn parse_tokens(tokens: &[&str]) -> Decision {
        parse(
            std::iter::once("novel-translate-cli")
                .chain(tokens.iter().copied())
                .map(OsString::from)
                .collect(),
        )
    }

    #[test]
    fn no_arguments_selects_tui() {
        assert_eq!(parse_tokens(&[]), Decision::LaunchTui);
    }

    #[test]
    fn help_and_version_are_text_actions() {
        assert!(matches!(parse_tokens(&["--help"]), Decision::Help(_)));
        assert!(matches!(parse_tokens(&["--version"]), Decision::Version(_)));
        assert!(matches!(
            parse_tokens(&["--json", "--help"]),
            Decision::Help(_)
        ));
    }

    #[test]
    fn conflicting_or_malformed_flags_fail_without_echoing_values() {
        assert!(matches!(
            parse_tokens(&["--help", "--version"]),
            Decision::Failure(_)
        ));
        assert!(matches!(
            parse_tokens(&["--json=true"]),
            Decision::Failure(_)
        ));
        assert!(matches!(
            parse_tokens(&["--json", "--json"]),
            Decision::Failure(_)
        ));
    }

    #[test]
    fn machine_mode_without_command_uses_precommand_error() {
        assert_eq!(
            parse_tokens(&["--json"]),
            Decision::Failure(super::CliFailure {
                json: true,
                command: None,
                kind: FailureKind::CommandRequired,
            })
        );
    }

    #[test]
    fn option_like_workspace_paths_are_rejected_before_the_command_is_dispatched() {
        for operation in ["create", "open"] {
            for path in ["--unknown", "--workspace", "--help", "--version", "-x"] {
                let Decision::Failure(failure) =
                    parse_tokens(&["--json", "workspace", operation, path])
                else {
                    panic!("option-like path was dispatched: {operation} {path}");
                };
                assert!(failure.json, "operation={operation}, path={path}");
                assert_eq!(failure.kind, FailureKind::ArgumentsInvalid);
                assert_eq!(
                    failure.command,
                    Some(match operation {
                        "create" => "workspace create",
                        "open" => "workspace open",
                        _ => unreachable!(),
                    })
                );
            }
        }
        assert!(!is_option_like_path(std::ffi::OsStr::new("./--valid-name")));
        assert!(!is_option_like_path(std::ffi::OsStr::new(
            "series_日本語 01"
        )));
    }

    #[test]
    fn workspace_errors_include_safe_recovery_actions() {
        use nt_application::workspace::WorkspaceError;

        assert_eq!(
            workspace_issue(&WorkspaceError::Busy),
            (
                "workspace.busy",
                "Workspace đang được tiến trình khác sử dụng. Đóng tiến trình đó rồi thử lại.",
                Some("close_other_process_and_retry")
            )
        );
        assert_eq!(
            workspace_issue(&WorkspaceError::PermissionDenied).2,
            Some("check_permissions")
        );
        assert_eq!(
            workspace_issue(&WorkspaceError::CorruptWorkspace).2,
            Some("restore_verified_workspace_backup")
        );
    }
}
