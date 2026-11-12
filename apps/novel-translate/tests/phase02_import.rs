use serde_json::Value;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let root = std::env::temp_dir();
        for _ in 0..100 {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path = root.join(format!(
                "novel-translate-phase02-{}-{id}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("could not create isolated test directory: {error}"),
            }
        }
        panic!("could not allocate a unique test directory");
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

fn run_cli(args: impl IntoIterator<Item = OsString>, cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_novel-translate-cli"))
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("APPDATA", cwd)
        .env("LOCALAPPDATA", cwd)
        .env("USERPROFILE", cwd)
        .env("HOME", cwd)
        .env("XDG_CONFIG_HOME", cwd)
        .env("XDG_DATA_HOME", cwd)
        .env("XDG_STATE_HOME", cwd)
        .env("TMP", cwd)
        .env("TEMP", cwd)
        .env("TMPDIR", cwd)
        .output()
        .expect("CLI process starts")
}

fn strings(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

fn parse_one_json_line(output: &Output) -> Value {
    let stdout = std::str::from_utf8(&output.stdout).expect("CLI stdout is UTF-8");
    assert_eq!(stdout.matches('\n').count(), 1, "stdout: {stdout:?}");
    serde_json::from_str(stdout).expect("CLI stdout is exactly one JSON envelope")
}

fn assert_path_is_not_printed(output: &Output, path: &Path) {
    let path = path.to_string_lossy();
    assert!(!String::from_utf8_lossy(&output.stdout).contains(path.as_ref()));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(path.as_ref()));
}

#[test]
fn cli_creates_and_reopens_a_unicode_spaced_workspace_with_matching_metadata() {
    let temp = TempDir::new();
    let workspace = temp.path().join("series_日本語 edition 01");

    let created = run_cli(
        [
            OsString::from("--json"),
            OsString::from("workspace"),
            OsString::from("create"),
            workspace.as_os_str().to_owned(),
        ],
        temp.path(),
    );
    assert_eq!(created.status.code(), Some(0));
    assert!(created.stderr.is_empty());
    let created_json = parse_one_json_line(&created);
    assert_eq!(created_json["schema_version"], 1);
    assert_eq!(created_json["command"], "workspace create");
    assert_eq!(created_json["state"], "complete");
    assert_eq!(created_json["scope"]["kind"], "workspace");
    assert_eq!(created_json["result"]["schema_version"], 2);
    assert_eq!(created_json["result"]["current_generation"], "g0");
    assert_eq!(
        created_json["result"]["series_id"].as_str().unwrap().len(),
        32
    );
    assert_path_is_not_printed(&created, &workspace);
    assert!(workspace.join("series.sqlite3").is_file());

    let reopened = run_cli(
        [
            OsString::from("--json"),
            OsString::from("workspace"),
            OsString::from("open"),
            workspace.as_os_str().to_owned(),
        ],
        temp.path(),
    );
    assert_eq!(reopened.status.code(), Some(0));
    assert!(reopened.stderr.is_empty());
    let reopened_json = parse_one_json_line(&reopened);
    assert_eq!(reopened_json["command"], "workspace open");
    assert_eq!(reopened_json["state"], "complete");
    assert_eq!(
        reopened_json["result"], created_json["result"],
        "opening the workspace returns the same persisted metadata"
    );
    assert_path_is_not_printed(&reopened, &workspace);

    let text_open = run_cli(
        [
            OsString::from("workspace"),
            OsString::from("open"),
            workspace.as_os_str().to_owned(),
        ],
        temp.path(),
    );
    assert_eq!(text_open.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&text_open.stdout).contains("Đã mở workspace"));
    assert!(
        String::from_utf8_lossy(&text_open.stdout)
            .contains(created_json["result"]["series_id"].as_str().unwrap())
    );
    assert_path_is_not_printed(&text_open, &workspace);
}

#[test]
fn option_like_create_and_open_paths_fail_before_filesystem_io() {
    let temp = TempDir::new();
    let mut before: Vec<_> = fs::read_dir(temp.path())
        .expect("read empty CLI working directory")
        .map(|entry| entry.expect("read directory entry").file_name())
        .collect();
    before.sort();

    for operation in ["create", "open"] {
        for token in ["--unknown", "--workspace", "--help", "--version", "-x"] {
            let output = run_cli(
                [
                    OsString::from("--json"),
                    OsString::from("workspace"),
                    OsString::from(operation),
                    OsString::from(token),
                ],
                temp.path(),
            );
            assert_eq!(
                output.status.code(),
                Some(2),
                "operation={operation}, token={token}, stdout={}, stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let envelope = parse_one_json_line(&output);
            assert_eq!(envelope["command"], format!("workspace {operation}"));
            assert_eq!(envelope["issues"][0]["code"], "cli.arguments_invalid");
            assert_eq!(envelope["next_action"], Value::Null);
            assert!(
                !temp.path().join(token).exists(),
                "invalid path token created a filesystem entry: {token}"
            );
        }
    }

    let mut after: Vec<_> = fs::read_dir(temp.path())
        .expect("read CLI working directory after rejected arguments")
        .map(|entry| entry.expect("read directory entry").file_name())
        .collect();
    after.sort();
    assert_eq!(
        after, before,
        "argument parsing must have no filesystem side effect"
    );
}

#[test]
fn cli_argument_and_storage_errors_are_safe_and_do_not_echo_paths() {
    let temp = TempDir::new();
    let missing = temp.path().join("missing_日本語 workspace");

    let argument_error = run_cli(strings(&["--json", "workspace", "create"]), temp.path());
    assert_eq!(argument_error.status.code(), Some(2));
    let argument_json = parse_one_json_line(&argument_error);
    assert_eq!(argument_json["command"], "workspace create");
    assert_eq!(argument_json["state"], "failed");

    let open_error = run_cli(
        [
            OsString::from("--json"),
            OsString::from("workspace"),
            OsString::from("open"),
            missing.as_os_str().to_owned(),
        ],
        temp.path(),
    );
    assert_eq!(open_error.status.code(), Some(3));
    let open_json = parse_one_json_line(&open_error);
    assert_eq!(open_json["command"], "workspace open");
    assert_eq!(open_json["state"], "failed");
    assert_eq!(open_json["issues"][0]["code"], "workspace.not_found");
    assert_path_is_not_printed(&open_error, &missing);

    let created = run_cli(
        [
            OsString::from("workspace"),
            OsString::from("create"),
            missing.as_os_str().to_owned(),
        ],
        temp.path(),
    );
    assert_eq!(created.status.code(), Some(0));
    let duplicate = run_cli(
        [
            OsString::from("--json"),
            OsString::from("workspace"),
            OsString::from("create"),
            missing.as_os_str().to_owned(),
        ],
        temp.path(),
    );
    assert_eq!(duplicate.status.code(), Some(3));
    let duplicate_json = parse_one_json_line(&duplicate);
    assert_eq!(
        duplicate_json["issues"][0]["code"],
        "workspace.already_exists"
    );
    assert_path_is_not_printed(&duplicate, &missing);
}

#[test]
fn cli_rejects_a_corrupt_workspace_without_affecting_another_openable_workspace() {
    let temp = TempDir::new();
    let valid = temp.path().join("series_日本語 edition 01");
    let created = run_cli(
        [
            OsString::from("--json"),
            OsString::from("workspace"),
            OsString::from("create"),
            valid.as_os_str().to_owned(),
        ],
        temp.path(),
    );
    assert_eq!(created.status.code(), Some(0));
    let created_json = parse_one_json_line(&created);

    let corrupt = temp.path().join("corrupt 日本語 workspace");
    fs::create_dir(&corrupt).expect("create corrupt-workspace directory");
    fs::write(corrupt.join("series.sqlite3"), b"not a SQLite database")
        .expect("write corrupt database fixture");
    let rejected = run_cli(
        [
            OsString::from("--json"),
            OsString::from("workspace"),
            OsString::from("open"),
            corrupt.as_os_str().to_owned(),
        ],
        temp.path(),
    );
    assert_eq!(rejected.status.code(), Some(3));
    let rejected_json = parse_one_json_line(&rejected);
    assert_eq!(rejected_json["command"], "workspace open");
    assert_eq!(rejected_json["state"], "failed");
    assert_eq!(rejected_json["issues"][0]["code"], "workspace.corrupt");
    assert_eq!(
        rejected_json["issues"][0]["message"],
        "Dữ liệu workspace không qua kiểm tra toàn vẹn. Khôi phục từ bản sao lưu đã kiểm chứng; giữ nguyên thư mục hiện tại để kiểm tra."
    );
    assert_eq!(
        rejected_json["next_action"],
        "restore_verified_workspace_backup"
    );
    assert_path_is_not_printed(&rejected, &corrupt);

    let reopened = run_cli(
        [
            OsString::from("--json"),
            OsString::from("workspace"),
            OsString::from("open"),
            valid.as_os_str().to_owned(),
        ],
        temp.path(),
    );
    assert_eq!(reopened.status.code(), Some(0));
    assert_eq!(
        parse_one_json_line(&reopened)["result"],
        created_json["result"]
    );
}

#[cfg(windows)]
#[test]
fn cli_reports_busy_for_a_shared_file_lock_and_reopens_after_release() {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_SHARE_READ: u32 = 0x0000_0001;

    let temp = TempDir::new();
    let workspace = temp.path().join("locked 日本語 series");
    let created = run_cli(
        [
            OsString::from("workspace"),
            OsString::from("create"),
            workspace.as_os_str().to_owned(),
        ],
        temp.path(),
    );
    assert_eq!(created.status.code(), Some(0));
    let held = OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(FILE_SHARE_READ)
        .open(workspace.join("series.sqlite3"))
        .expect("hold an open that permits reads but blocks writer access");

    let busy = run_cli(
        [
            OsString::from("--json"),
            OsString::from("workspace"),
            OsString::from("open"),
            workspace.as_os_str().to_owned(),
        ],
        temp.path(),
    );
    assert_eq!(busy.status.code(), Some(3));
    let busy_json = parse_one_json_line(&busy);
    assert_eq!(busy_json["issues"][0]["code"], "workspace.busy");
    assert_eq!(busy_json["next_action"], "close_other_process_and_retry");
    assert!(
        busy_json["issues"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Đóng tiến trình đó rồi thử lại")
    );
    drop(held);

    let reopened = run_cli(
        [
            OsString::from("--json"),
            OsString::from("workspace"),
            OsString::from("open"),
            workspace.as_os_str().to_owned(),
        ],
        temp.path(),
    );
    assert_eq!(reopened.status.code(), Some(0));
    assert_eq!(parse_one_json_line(&reopened)["state"], "complete");
}
