#![cfg(windows)]

#[path = "support/epub.rs"]
mod epub;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::Barrier;
use std::sync::atomic::{AtomicUsize, Ordering};

use novel_translate::import_workflow::{
    ImportFlowError, ImportTarget, Publication, prepare_registered_import, read_saved_volume,
};
use nt_application::volume_import::{
    GenerationCommitError, GenerationId, ImportWriteError, VolumeId, source_scope::ScopeMode,
};
use nt_application::workspace::{WorkspaceRepository, create_workspace};
use nt_storage::{SqliteVolumeImportReader, SqliteWorkspaceRepository};
use rusqlite::Connection;
use serde_json::Value;

const V1: &str = "11111111111111111111111111111111";
const V2: &str = "22222222222222222222222222222222";
const BAD: &str = "33333333333333333333333333333333";

struct Fixture {
    root: PathBuf,
    workspace: PathBuf,
    epub: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "nt-p02-t12-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let workspace = root.join("Workspace 日本語");
        create_workspace(&SqliteWorkspaceRepository, &workspace).unwrap();
        let epub = root.join("巻 01.epub");
        fs::write(&epub, epub::book(3, epub::CHAPTER)).unwrap();
        Self {
            root,
            workspace,
            epub,
        }
    }

    fn cli(&self, args: &[&OsStr]) -> (i32, Value, Output) {
        let output = Command::new(env!("CARGO_BIN_EXE_novel-translate-cli"))
            .args(args)
            .current_dir(&self.root)
            .stdin(Stdio::null())
            .env("HOME", &self.root)
            .env("LOCALAPPDATA", &self.root)
            .env("APPDATA", &self.root)
            .env("TEMP", &self.root)
            .env("TMP", &self.root)
            .output()
            .unwrap();
        let code = output.status.code().unwrap();
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        assert_eq!(stdout.lines().count(), 1, "{stdout}\n{stderr}");
        for stream in [stdout, stderr] {
            assert!(!stream.contains(&*self.root.to_string_lossy()));
            assert!(!stream.contains("巻 01"));
        }
        let value: Value = serde_json::from_str(stdout).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 8);
        assert_eq!(value["schema_version"], 1);
        assert!(value["task_id"].is_null());
        (code, value, output)
    }

    fn import(&self, path: &Path, volume: &str) -> (i32, Value, Output) {
        self.cli(&[
            OsStr::new("--workspace"),
            self.workspace.as_os_str(),
            OsStr::new("import"),
            path.as_os_str(),
            OsStr::new("--volume"),
            OsStr::new(volume),
            OsStr::new("--json"),
            OsStr::new("--non-interactive"),
        ])
    }

    fn version(&self) -> (String, i64) {
        let db = Connection::open(self.workspace.join("series.sqlite3")).unwrap();
        db.query_row(
            "SELECT generation_id, (SELECT count(*) FROM volumes) FROM current_generation",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn cli_registers_two_ids_and_scope_survives_reopen_without_changing_the_other_volume() {
    let fx = Fixture::new();
    for volume in [V1, V2] {
        let (exit, json, _) = fx.import(&fx.epub, volume);
        assert!(
            matches!(exit, 0 | 4) && json["result"]["published"] == true,
            "{json}"
        );
        assert_eq!(json["result"]["content_mode"], "all-text");
        assert_eq!(json["result"]["volume_id"], volume);
        assert!(
            json["result"]["epub_sha256"]
                .as_str()
                .is_some_and(|hash| hash.len() == 64)
        );
    }
    assert_eq!(fx.version().1, 2);
    let (code, result, _) = fx.cli(&[
        OsStr::new("scope"),
        OsStr::new("set"),
        OsStr::new("--content"),
        OsStr::new("story"),
        OsStr::new("--volume"),
        OsStr::new(V1),
        OsStr::new("--workspace"),
        fx.workspace.as_os_str(),
        OsStr::new("--json"),
    ]);
    assert_eq!(code, 4, "{result}");
    assert_eq!(result["state"], "awaiting_approval");
    assert_eq!(result["result"]["published"], true);
    assert_eq!(result["result"]["content_mode"], "story");
    assert!(
        !result["result"]["review_pending"]
            .as_object()
            .unwrap()
            .is_empty()
    );

    let meta = SqliteWorkspaceRepository.open(&fx.workspace).unwrap();
    let mut reader = SqliteVolumeImportReader::open(&fx.workspace).unwrap();
    let snapshot = reader.begin_snapshot().unwrap();
    let listed = snapshot.volumes().unwrap();
    assert_eq!(listed.len(), 2);
    assert!(
        listed
            .iter()
            .all(|volume| volume.display_title.as_deref() == Some("Codec test"))
    );
    for (raw, mode) in [(V1, ScopeMode::Story), (V2, ScopeMode::AllText)] {
        let id = VolumeId::parse(raw).unwrap();
        let opened = read_saved_volume(&fx.workspace, &meta.series.id, &id, None).unwrap();
        assert_eq!(opened.version.state.mode(), mode);
        assert!(!opened.record.unwrap().mapping.volume.main_flow.is_empty());
    }
    snapshot.finish().unwrap();
    let g0 = reader
        .begin_generation_snapshot(&GenerationId::parse("g0").unwrap())
        .unwrap();
    assert!(g0.volumes().unwrap().is_empty());
    for raw in [V1, V2] {
        let id = VolumeId::parse(raw).unwrap();
        assert!(matches!(
            g0.volume(&id),
            Err(ImportWriteError::InvalidVolume)
        ));
        assert!(matches!(
            g0.open_payload(&id),
            Err(ImportWriteError::InvalidVolume)
        ));
    }
}

#[test]
fn invalid_arguments_and_sources_never_create_a_volume_or_publish_a_generation() {
    let fx = Fixture::new();
    let invalid_file = fx.root.join("private malformed.epub");
    fs::write(&invalid_file, b"not a zip").unwrap();
    let before = fx.version();
    let (invalid_id, invalid_json, _) = fx.import(&fx.epub, "secret_bad_id");
    assert_eq!(invalid_id, 2);
    assert_eq!(invalid_json["command"], "import");
    assert_eq!(invalid_json["state"], "failed");
    assert!(invalid_json["result"].is_null());
    let (malformed, malformed_json, output) = fx.import(&invalid_file, BAD);
    assert_eq!(malformed, 3);
    assert_eq!(malformed_json["issues"][0]["code"], "epub.invalid");
    assert!(!String::from_utf8_lossy(&output.stderr).contains("private malformed"));
    let (wrong_extension, wrong_json, _) = fx.import(&fx.root.join("secret.zip"), BAD);
    assert_eq!(wrong_extension, 3);
    assert_eq!(wrong_json["issues"][0]["code"], "epub.extension");
    let (scope, scope_json, _) = fx.cli(&[
        OsStr::new("scope"),
        OsStr::new("set"),
        OsStr::new("--content"),
        OsStr::new("story"),
        OsStr::new("--volume"),
        OsStr::new(BAD),
        OsStr::new("--workspace"),
        fx.workspace.as_os_str(),
        OsStr::new("--json"),
    ]);
    assert_eq!(scope, 3);
    assert_eq!(scope_json["issues"][0]["code"], "volume.invalid_target");
    assert_eq!(fx.version(), before);
}

#[test]
fn one_registration_wins_on_a_shared_parent_and_replay_returns_the_same_generation() {
    let fx = Fixture::new();
    let make = |id| {
        let mut reader = SqliteVolumeImportReader::open(&fx.workspace).unwrap();
        let view = reader.begin_snapshot().unwrap();
        let (operation, generation) = view.new_write_ids().unwrap();
        let target = ImportTarget {
            series: view.series_id().clone(),
            expected_parent: view.generation_id().clone(),
            volume: VolumeId::parse(id).unwrap(),
            operation,
            generation,
        };
        view.finish().unwrap();
        prepare_registered_import(&fx.workspace, &fx.epub, target)
            .ok()
            .unwrap()
    };
    let left = make(V1);
    let right = make(V2);
    assert_eq!(fx.version(), ("g0".into(), 0));
    let barrier = Barrier::new(2);
    let (first, second) = std::thread::scope(|scope| {
        let thread = scope.spawn(|| {
            barrier.wait();
            left.submit()
        });
        barrier.wait();
        let second = right.submit();
        (thread.join().unwrap(), second)
    });
    let (saved, error, request) = if first.is_ok() {
        (first, second, left)
    } else {
        (second, first, right)
    };
    assert!(
        matches!(
            &error,
            Err(ImportFlowError::Storage(ImportWriteError::Generation(
                GenerationCommitError::StaleGeneration
            )))
        ),
        "unexpected registration loser: {:?}",
        error.as_ref().err()
    );
    let Publication::Published {
        saved,
        replayed: false,
    } = saved.unwrap()
    else {
        panic!("expected one successful first registration");
    };
    let Publication::Published {
        saved: replayed,
        replayed: true,
    } = request.submit().unwrap()
    else {
        panic!("expected first registration to replay");
    };
    assert_eq!(saved.generation, replayed.generation);
    assert_eq!(fx.version(), (saved.generation.as_str().to_owned(), 1));
}

#[test]
fn schema_three_with_an_import_preserves_legacy_history_and_backs_up_the_source() {
    let fx = Fixture::new();
    assert!(matches!(fx.import(&fx.epub, V1).0, 0 | 4));
    let before = fx.version();
    let database = Connection::open(fx.workspace.join("series.sqlite3")).unwrap();
    database
        .execute_batch("DROP TABLE cp06_volume_registrations; PRAGMA user_version=3;")
        .unwrap();
    drop(database);
    let opened = SqliteWorkspaceRepository.open(&fx.workspace).unwrap();
    assert_eq!(opened.schema_version, 4);
    assert_eq!(fx.version(), before);
    let id = VolumeId::parse(V1).unwrap();
    assert!(
        read_saved_volume(&fx.workspace, &opened.series.id, &id, None)
            .unwrap()
            .record
            .is_some()
    );
    let backup = fs::read_dir(&fx.workspace)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".nt-migration-backup-")
        })
        .unwrap();
    let backed_up = Connection::open(backup.join("series.sqlite3")).unwrap();
    let version: u32 = backed_up
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 3);
    let stored_path: String = backed_up.query_row(
        "SELECT source_relative_path FROM cp05_resource_refs WHERE source_relative_path IS NOT NULL",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(
        fs::read(backup.join(&stored_path)).unwrap(),
        fs::read(fx.workspace.join(&stored_path)).unwrap()
    );
    assert!(
        fs::read_to_string(backup.join("manifest.sha256"))
            .unwrap()
            .contains(&stored_path)
    );
}
