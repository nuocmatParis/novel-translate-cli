use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const CLI_TIMEOUT: Duration = Duration::from_secs(10);
const TIMEOUT_PROBE: &str = "NT_CLI_CONTRACT_TIMEOUT_PROBE";
const TIMEOUT_MARKER: &str = "NT_CLI_CONTRACT_TIMEOUT_MARKER";
const SECRET_ENV: &str = "NT_CLI_CONTRACT_SECRET_SENTINEL";
const SECRET_SENTINEL: &str = "synthetic-cli-secret-sentinel-9f52";
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let root = std::env::temp_dir();
        for _ in 0..100 {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path = root.join(format!(
                "novel-translate-cli-contract-{}-{id}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("could not create temporary CLI directory: {error}"),
            }
        }
        panic!("could not allocate a unique temporary CLI directory");
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

enum ChildOutcome {
    Exited(Output),
    TimedOut {
        status: ExitStatus,
        kill_succeeded: bool,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
}

fn run_cli(args: &[&str]) -> Output {
    let cwd = TempDir::new();
    run_cli_in(args, cwd.path())
}

fn run_cli_in(args: &[&str], cwd: &Path) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_novel-translate-cli"));
    command
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env(SECRET_ENV, SECRET_SENTINEL)
        .env("APPDATA", cwd)
        .env("LOCALAPPDATA", cwd)
        .env("USERPROFILE", cwd)
        .env("HOME", cwd)
        .env("XDG_CONFIG_HOME", cwd)
        .env("XDG_DATA_HOME", cwd)
        .env("XDG_STATE_HOME", cwd)
        .env("TMP", cwd)
        .env("TEMP", cwd)
        .env("TMPDIR", cwd);

    match run_child(&mut command, CLI_TIMEOUT) {
        ChildOutcome::Exited(output) => {
            assert_process_output_hides_secret(&output);
            output
        }
        ChildOutcome::TimedOut { .. } => {
            panic!("CLI process exceeded the bounded {CLI_TIMEOUT:?} timeout")
        }
    }
}

fn run_child(command: &mut Command, timeout: Duration) -> ChildOutcome {
    let mut child = command.spawn().expect("child process should start");
    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let stdout_reader = thread::spawn(move || read_pipe(stdout));
    let stderr_reader = thread::spawn(move || read_pipe(stderr));
    let started_at = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = join_pipe_reader(stdout_reader);
                let stderr = join_pipe_reader(stderr_reader);
                return ChildOutcome::Exited(Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) if started_at.elapsed() >= timeout => {
                return kill_reap_and_capture(&mut child, stdout_reader, stderr_reader);
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                panic!("failed while waiting for child process: {error}");
            }
        }
    }
}

fn kill_reap_and_capture(
    child: &mut Child,
    stdout_reader: thread::JoinHandle<Vec<u8>>,
    stderr_reader: thread::JoinHandle<Vec<u8>>,
) -> ChildOutcome {
    let kill_succeeded = child.kill().is_ok();
    let status = child.wait().expect("timed-out child must be reaped");
    let stdout = join_pipe_reader(stdout_reader);
    let stderr = join_pipe_reader(stderr_reader);
    ChildOutcome::TimedOut {
        status,
        kill_succeeded,
        stdout,
        stderr,
    }
}

fn read_pipe(mut pipe: impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    pipe.read_to_end(&mut bytes)
        .expect("child output pipe should be readable");
    bytes
}

fn join_pipe_reader(reader: thread::JoinHandle<Vec<u8>>) -> Vec<u8> {
    reader.join().expect("child output reader should finish")
}

fn assert_process_output_hides_secret(output: &Output) {
    assert!(
        !output
            .stdout
            .windows(SECRET_SENTINEL.len())
            .any(|part| { part == SECRET_SENTINEL.as_bytes() })
    );
    assert!(
        !output
            .stderr
            .windows(SECRET_SENTINEL.len())
            .any(|part| { part == SECRET_SENTINEL.as_bytes() })
    );
}

fn directory_snapshot(path: &Path) -> BTreeMap<std::ffi::OsString, Option<Vec<u8>>> {
    fs::read_dir(path)
        .expect("temporary working directory is readable")
        .map(|entry| {
            let entry = entry.expect("directory entry is readable");
            let kind = entry.file_type().expect("file type is readable");
            let content = if kind.is_file() {
                Some(fs::read(entry.path()).expect("file content is readable"))
            } else {
                None
            };
            (entry.file_name(), content)
        })
        .collect()
}

#[test]
fn timeout_probe_child() {
    let Ok(marker) = std::env::var(TIMEOUT_MARKER) else {
        return;
    };
    fs::write(marker, b"started").expect("timeout child writes its start marker");
    writeln!(std::io::stdout(), "timeout child reached wait")
        .expect("timeout child writes a captured marker");
    while std::env::var_os(TIMEOUT_PROBE).is_some() {
        thread::park_timeout(Duration::from_secs(60));
    }
}

#[test]
fn timeout_guard_kills_and_reaps_a_hung_child() {
    let cwd = TempDir::new();
    let marker = cwd.path().join("child-started.txt");
    let executable = std::env::current_exe().expect("integration test executable exists");
    let mut command = Command::new(executable);
    command
        .args(["--exact", "timeout_probe_child", "--nocapture"])
        .current_dir(cwd.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env(TIMEOUT_PROBE, "1")
        .env(TIMEOUT_MARKER, &marker);

    let ChildOutcome::TimedOut {
        status,
        kill_succeeded,
        stdout,
        stderr,
    } = run_child(&mut command, Duration::from_secs(5))
    else {
        panic!("hanging child unexpectedly exited before the timeout");
    };

    assert!(marker.is_file(), "child reached the deliberate wait state");
    assert!(kill_succeeded, "timeout path terminated the child");
    assert!(
        !status.success(),
        "killed child returned a failing process status"
    );
    assert!(String::from_utf8_lossy(&stdout).contains("timeout child reached wait"));
    assert!(stderr.is_empty());
}

#[test]
fn help_and_version_are_successful_text_before_terminal_setup() {
    let help = run_cli(&["--help"]);
    assert!(help.status.success());
    let help_stdout = String::from_utf8(help.stdout).expect("help is UTF-8");
    assert!(help_stdout.contains("Usage: novel-translate-cli"));
    assert!(help_stdout.contains("--non-interactive"));
    assert!(help_stdout.contains("workspace create <path>"));
    assert!(help.stderr.is_empty());

    let version = run_cli(&["--version"]);
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).expect("version is UTF-8"),
        format!("novel-translate-cli {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(version.stderr.is_empty());
}

#[test]
fn redirected_no_argument_invocation_fails_finitely_without_tui() {
    let output = run_cli(&[]);
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("diagnostic is UTF-8");
    assert!(stderr.contains("stdin và stdout"));
    assert!(!stderr.contains("raw mode"));
}

#[test]
fn non_interactive_without_command_is_a_finite_argument_error() {
    let output = run_cli(&["--non-interactive"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .expect("diagnostic is UTF-8")
            .contains("Cần cung cấp một lệnh")
    );
}

#[test]
fn json_command_required_matches_the_precommand_fixture_and_has_one_object() {
    let output = run_cli(&["--json"]);
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8(output.stdout).expect("JSON output is UTF-8");
    assert_eq!(stdout.matches('\n').count(), 1);
    let actual: Value = serde_json::from_str(&stdout).expect("stdout is one JSON value");
    let expected: Value = serde_json::from_str(include_str!("fixtures/cli/pre-command-error.json"))
        .expect("fixture is valid JSON");
    assert_eq!(actual, expected);
    assert!(
        String::from_utf8(output.stderr)
            .expect("diagnostic is UTF-8")
            .contains("Thiếu lệnh")
    );
}

#[test]
fn unknown_command_or_flag_never_echoes_the_raw_argument() {
    for argument in ["unknown-command", "--unknown-option=sentinel-private-value"] {
        let output = run_cli(&[argument]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).expect("diagnostic is UTF-8");
        assert!(stderr.contains("Tham số CLI không hợp lệ"));
        assert!(!stderr.contains(argument));
    }
}

#[test]
fn json_argument_errors_remain_machine_readable_and_suppress_ansi() {
    let invalid_precommand_cases = [
        vec!["--json", "unknown-command"],
        vec!["unknown-command", "--json"],
        vec!["--json", "--workspace"],
        vec!["--json", "--workspace", "sentinel-private-value"],
        vec!["--json", "--json"],
        vec!["--json", "--json=true"],
    ];

    for args in invalid_precommand_cases {
        let output = run_cli(&args);
        assert_eq!(output.status.code(), Some(2), "args: {args:?}");
        let stdout = String::from_utf8(output.stdout).expect("JSON output is UTF-8");
        assert_eq!(stdout.matches('\n').count(), 1, "args: {args:?}");
        let value: Value = serde_json::from_str(&stdout).expect("stdout is one JSON value");
        assert_eq!(value["schema_version"], 1, "args: {args:?}");
        assert_eq!(value["command"], Value::Null, "args: {args:?}");
        assert_eq!(value["state"], "failed", "args: {args:?}");
        assert_eq!(value["task_id"], Value::Null, "args: {args:?}");
        assert_eq!(value["scope"], Value::Null, "args: {args:?}");
        assert_eq!(value["result"], Value::Null, "args: {args:?}");
        assert_eq!(
            value["issues"].as_array().unwrap().len(),
            1,
            "args: {args:?}"
        );
        assert_eq!(value["next_action"], Value::Null, "args: {args:?}");
        assert!(!stdout.contains('\u{1b}'));
        assert!(!stdout.contains("unknown-command"));
        assert!(!stdout.contains("sentinel-private-value"));
    }
}

#[test]
fn malformed_mode_flag_and_help_version_conflict_fail_with_exit_two() {
    for args in [vec!["--json=true"], vec!["--help", "--version"]] {
        let output = run_cli(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(
            !String::from_utf8(output.stderr)
                .expect("diagnostic is UTF-8")
                .contains("--json=true")
        );
    }

    let conflict_with_json = run_cli(&["--json", "--help", "--version"]);
    assert_eq!(conflict_with_json.status.code(), Some(2));
    assert!(conflict_with_json.stdout.is_empty());
    assert!(
        String::from_utf8(conflict_with_json.stderr)
            .expect("diagnostic is UTF-8")
            .contains("Không thể dùng đồng thời")
    );
}

#[test]
fn help_or_version_with_an_unknown_argument_is_not_a_success_shortcut() {
    for args in [
        vec!["--help", "unexpected"],
        vec!["--version", "unexpected"],
    ] {
        let output = run_cli(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .expect("diagnostic is UTF-8")
                .contains("Tham số CLI không hợp lệ")
        );
    }

    let json_error = run_cli(&["--json", "--help", "unexpected"]);
    assert_eq!(json_error.status.code(), Some(2));
    let stdout = String::from_utf8(json_error.stdout).expect("JSON output is UTF-8");
    assert_eq!(
        serde_json::from_str::<Value>(&stdout).unwrap()["state"],
        "failed"
    );
}

#[test]
fn workspace_argument_error_leaves_temporary_directory_untouched() {
    let cwd = TempDir::new();
    fs::write(cwd.path().join("existing-sentinel.txt"), b"keep this file")
        .expect("sentinel file is created in the isolated directory");
    let before = directory_snapshot(cwd.path());

    let output = run_cli_in(&["--json", "workspace", "create"], cwd.path());

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8(output.stdout).expect("JSON output is UTF-8");
    assert_eq!(stdout.matches('\n').count(), 1);
    let parsed: Value = serde_json::from_str(&stdout).expect("whole stdout is one JSON object");
    assert_eq!(parsed["command"], "workspace create");
    assert_eq!(parsed["state"], "failed");
    assert!(!stdout.contains("existing-sentinel"));
    assert_eq!(directory_snapshot(cwd.path()), before);
}
