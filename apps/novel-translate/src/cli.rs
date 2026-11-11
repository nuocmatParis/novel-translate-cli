use crate::diagnostics::{self, SafeCode};
use clap::{ArgAction, Parser};
use serde::Serialize;
use serde_json::Value;
use std::ffi::OsString;

#[derive(Debug, Parser)]
#[command(
    name = "novel-translate-cli",
    bin_name = "novel-translate-cli",
    version = env!("CARGO_PKG_VERSION"),
    about = "CLI và TUI nền tảng cho Novel Translate"
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
    Failure(CliFailure),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CliFailure {
    pub(crate) json: bool,
    pub(crate) kind: FailureKind,
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
    let tokens: Vec<String> = args
        .iter()
        .skip(1)
        .filter_map(|value| value.to_str().map(str::to_owned))
        .collect();
    let json_requested = tokens.iter().any(|token| token == "--json");

    let json_count = tokens
        .iter()
        .filter(|token| token.as_str() == "--json")
        .count();
    let non_interactive_count = tokens
        .iter()
        .filter(|token| token.as_str() == "--non-interactive")
        .count();
    let has_help = tokens
        .iter()
        .any(|token| matches!(token.as_str(), "--help" | "-h"));
    let has_version = tokens
        .iter()
        .any(|token| matches!(token.as_str(), "--version" | "-V"));
    let malformed_mode_flag = tokens
        .iter()
        .any(|token| token.starts_with("--json=") || token.starts_with("--non-interactive="));

    // These global switches are intentionally strict. In particular, don't
    // let Clap's raw diagnostic echo an arbitrary argument or secret value.
    if has_help && has_version {
        return Decision::Failure(CliFailure {
            json: false,
            kind: FailureKind::HelpVersionConflict,
        });
    }
    if malformed_mode_flag || json_count > 1 || non_interactive_count > 1 {
        return invalid_arguments(json_requested);
    }
    if tokens.len() != args.len().saturating_sub(1)
        || tokens.iter().any(|token| {
            !matches!(
                token.as_str(),
                "--help" | "-h" | "--version" | "-V" | "--json" | "--non-interactive"
            )
        })
    {
        return invalid_arguments(json_requested);
    }

    match Arguments::try_parse_from(args) {
        Ok(parsed) => {
            if parsed.json > 0 || parsed.non_interactive > 0 {
                Decision::Failure(CliFailure {
                    json: parsed.json > 0,
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
                command: None,
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
    Decision::Failure(CliFailure {
        json,
        kind: FailureKind::ArgumentsInvalid,
    })
}

#[cfg(test)]
mod tests {
    use super::{Decision, FailureKind, parse};
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
                kind: FailureKind::CommandRequired,
            })
        );
    }
}
