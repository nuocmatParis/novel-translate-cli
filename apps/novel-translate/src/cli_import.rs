use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use novel_translate::import_codec::Digest;
use novel_translate::import_workflow::{
    CaptureError, ImportFlowError, Publication, ReviewIssue, ScopeChange,
};
use nt_application::volume_import::{
    GenerationCommitError, ImportWriteError, VolumeId, VolumeState, source_scope::ScopeMode,
};
use serde_json::{Value, json};

use super::{Decision, ErrorEnvelope, Issue, invalid_arguments_for, is_option_like_path};
use crate::import_commands::{self, ImportCommandError, ImportDestination};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct VolumeCommand {
    workspace: PathBuf,
    volume: VolumeId,
    action: VolumeAction,
    json: bool,
}

#[derive(Debug, Eq, PartialEq)]
enum VolumeAction {
    Import(PathBuf),
    Scope(ScopeMode),
}

impl VolumeAction {
    fn command(&self) -> &'static str {
        match self {
            Self::Import(_) => "import",
            Self::Scope(_) => "scope set",
        }
    }
}

pub(super) fn parse(tokens: &[&OsStr], json: bool) -> Option<Decision> {
    let command_position = if tokens.first() == Some(&OsStr::new("--workspace")) {
        2
    } else {
        0
    };
    let command = match tokens.get(command_position)?.to_str()? {
        "import" => "import",
        "scope" if tokens.get(command_position + 1) == Some(&OsStr::new("set")) => "scope set",
        "scope" => "scope",
        _ => return None,
    };
    let invalid = || Some(invalid_arguments_for(json, Some(command)));
    let mut workspace = None;
    let mut volume = None;
    let mut content = None;
    let mut positional = Vec::new();
    let mut arguments = tokens.iter().copied();
    while let Some(token) = arguments.next() {
        let option = match token.to_str() {
            Some("--workspace") => &mut workspace,
            Some("--volume") => &mut volume,
            Some("--content") => &mut content,
            _ => {
                positional.push(token);
                continue;
            }
        };
        let Some(value) = arguments.next() else {
            return invalid();
        };
        if option.is_some() || value.is_empty() || is_option_like_path(value) {
            return invalid();
        }
        *option = Some(value);
    }

    let (Some(workspace), Some(volume)) = (workspace, volume.and_then(OsStr::to_str)) else {
        return invalid();
    };
    let Ok(volume) = VolumeId::parse(volume) else {
        return invalid();
    };
    let action = match positional.as_slice() {
        [name, path] if *name == OsStr::new("import") && content.is_none() => {
            if path.is_empty() || is_option_like_path(path) {
                return invalid();
            }
            VolumeAction::Import(PathBuf::from(path))
        }
        [name, subcommand] if *name == OsStr::new("scope") && *subcommand == OsStr::new("set") => {
            let mode = match content.and_then(OsStr::to_str) {
                Some("all-text") => ScopeMode::AllText,
                Some("story") => ScopeMode::Story,
                _ => return invalid(),
            };
            VolumeAction::Scope(mode)
        }
        _ => return invalid(),
    };
    Some(Decision::Volume(VolumeCommand {
        workspace: workspace.into(),
        volume,
        action,
        json,
    }))
}

pub(crate) fn run(command: VolumeCommand) -> (i32, String, String) {
    let result = match &command.action {
        VolumeAction::Import(epub) => import_commands::import(
            &command.workspace,
            None,
            epub,
            ImportDestination::Selected(command.volume.clone()),
        ),
        VolumeAction::Scope(mode) => import_commands::scope(
            &command.workspace,
            None,
            command.volume.clone(),
            ScopeChange {
                mode: *mode,
                classifications: Vec::new(),
                save_as_series_default: false,
            },
        )
        .map_err(Into::into),
    };
    let mut envelope = ErrorEnvelope {
        schema_version: 1,
        command: Some(command.action.command().to_owned()),
        state: "failed",
        task_id: None,
        scope: Some(json!({
            "kind": "volume", "volume_ids": [command.volume.as_str()], "chapter_ids": []
        })),
        result: None,
        issues: Vec::new(),
        next_action: None,
    };
    let (exit, stdout, stderr) = match result {
        Ok((series, Publication::Published { saved, .. })) => {
            let (epub, mapping) = match &saved.version.state {
                VolumeState::Imported(state) => (
                    Some(Digest(state.epub_sha256)),
                    Some(Digest(state.mapping_version)),
                ),
                VolumeState::Unimported { .. } => (None, None),
            };
            let content_mode = match saved.version.state.mode() {
                ScopeMode::AllText => "all-text",
                ScopeMode::Story => "story",
            };
            let mut counts = BTreeMap::new();
            let mut locations = Vec::new();
            saved.for_each_review_issue(|issue| {
                let (kind, position) = review_position(issue);
                *counts.entry(kind).or_insert(0_u64) += 1;
                if locations.len() < 32 {
                    locations.push(json!({ "kind": kind, "position": position }));
                }
            });
            envelope.result = Some(json!({
                "series_id": series.as_str(), "volume_id": command.volume.as_str(),
                "generation_id": saved.generation.as_str(), "epub_sha256": epub,
                "mapping_version": mapping, "content_mode": content_mode, "published": true,
                "review_pending": counts, "review_positions": locations
            }));
            let summary = format!(
                "Đã lưu tập {} ở mốc {} với phạm vi {}.\n",
                command.volume.as_str(),
                saved.generation.as_str(),
                content_mode
            );
            if counts.is_empty() {
                envelope.state = "complete";
                (0, summary, String::new())
            } else {
                envelope.state = "awaiting_approval";
                envelope.next_action = Some("review_in_tui");
                let message = "Tập đã được lưu. Mở Tập và chương để xem nội dung cần duyệt.";
                envelope.issues.push(Issue {
                    code: "volume.review_pending",
                    severity: "warning",
                    message,
                });
                (4, summary, format!("{message}\n"))
            }
        }
        Ok((_, Publication::CommitOutcomeUnknown { operation, .. })) => {
            envelope.result =
                Some(json!({ "operation_id": operation.as_str(), "published": Value::Null }));
            apply_issue(
                &mut envelope,
                IssueDetails::new(
                    "volume.commit_unknown",
                    "Chưa xác định lượt lưu. Mở lại workspace và kiểm tra mã thao tác.",
                    "reconcile_operation",
                ),
            )
        }
        Ok((_, Publication::SavedButUnreadable { generation, .. })) => {
            envelope.result =
                Some(json!({ "generation_id": generation.as_str(), "published": true }));
            apply_issue(
                &mut envelope,
                IssueDetails::new(
                    "volume.saved_unreadable",
                    "Đã lưu mốc mới nhưng không đọc lại được. Mở lại workspace để kiểm tra.",
                    "reopen_workspace",
                ),
            )
        }
        Err(error) => apply_issue(&mut envelope, diagnose(&error)),
    };
    if command.json {
        let line = serde_json::to_string(&envelope).expect("fixed volume envelope is serializable");
        (exit, format!("{line}\n"), stderr)
    } else {
        (exit, stdout, stderr)
    }
}

struct IssueDetails {
    code: &'static str,
    message: &'static str,
    next: &'static str,
    conflict: bool,
}

impl IssueDetails {
    fn new(code: &'static str, message: &'static str, next: &'static str) -> Self {
        Self {
            code,
            message,
            next,
            conflict: false,
        }
    }
}

fn apply_issue(envelope: &mut ErrorEnvelope, details: IssueDetails) -> (i32, String, String) {
    envelope.next_action = Some(details.next);
    if details.conflict {
        envelope.state = "conflict";
    }
    envelope.issues.push(Issue {
        code: details.code,
        severity: "error",
        message: details.message,
    });
    (
        if details.conflict { 4 } else { 3 },
        String::new(),
        format!("{}\n", details.message),
    )
}

fn diagnose(error: &ImportCommandError) -> IssueDetails {
    use ImportCommandError::*;
    use ImportFlowError as Flow;
    use IssueDetails as Issue;
    match error {
        InvalidExtension => Issue::new(
            "epub.extension",
            "Chọn một tệp có đuôi .epub.",
            "choose_epub",
        ),
        Import(Flow::Capture(CaptureError::PermissionDenied)) => Issue::new(
            "epub.permission_denied",
            "Không đủ quyền đọc EPUB. Kiểm tra quyền rồi thử lại.",
            "check_permissions",
        ),
        Import(Flow::Capture(CaptureError::SourceChanged)) => Issue::new(
            "epub.source_changed",
            "EPUB đang thay đổi. Đóng ứng dụng đang sửa tệp rồi thử lại.",
            "retry_import",
        ),
        Import(Flow::Capture(_)) => Issue::new(
            "epub.unreadable",
            "Không đọc được tệp EPUB đã chọn.",
            "check_epub",
        ),
        Import(Flow::Archive(_) | Flow::Extraction(_) | Flow::References(_)) => Issue::new(
            "epub.invalid",
            "EPUB không hợp lệ hoặc có cấu trúc chưa hỗ trợ.",
            "check_epub",
        ),
        Import(Flow::Blocked(_)) => Issue::new(
            "epub.source_incomplete",
            "EPUB thiếu hoặc có phần nguồn bắt buộc chưa được hỗ trợ.",
            "check_epub",
        ),
        Import(Flow::Storage(ImportWriteError::InvalidVolume) | Flow::InvalidWorkspace) => {
            Issue::new(
                "volume.invalid_target",
                "Tập đích không hợp lệ trong workspace đã chọn.",
                "check_workspace_and_volume",
            )
        }
        Import(Flow::Storage(ImportWriteError::Generation(
            GenerationCommitError::OperationConflict | GenerationCommitError::StaleGeneration,
        ))) => Issue {
            conflict: true,
            ..Issue::new(
                "volume.conflict",
                "Workspace vừa thay đổi hoặc mã thao tác đã được dùng. Mở lại và đối chiếu trước khi thử lại.",
                "reopen_workspace",
            )
        },
        Import(Flow::Storage(ImportWriteError::Generation(GenerationCommitError::Busy))) => {
            Issue::new(
                "workspace.busy",
                "Workspace đang bận. Kiểm tra thao tác đang chạy rồi thử lại.",
                "retry",
            )
        }
        Import(Flow::Storage(ImportWriteError::ResourceLimit) | Flow::Source(_)) => Issue::new(
            "volume.source_limits",
            "Nguồn vượt giới hạn xử lý hoặc không hợp lệ.",
            "check_epub",
        ),
        Import(_) => Issue::new(
            "volume.operation_failed",
            "Không thể hoàn tất thao tác. Kiểm tra workspace và nguồn rồi thử lại.",
            "check_workspace",
        ),
    }
}

fn review_position(issue: ReviewIssue) -> (&'static str, usize) {
    match issue {
        ReviewIssue::Source(position) => ("source", position),
        ReviewIssue::Navigation(position) => ("navigation", position),
        ReviewIssue::ResourceReference(position) => ("resource_reference", position),
        ReviewIssue::Mapping(position) => ("mapping", position),
        ReviewIssue::UnclassifiedBoundary(position) => ("unclassified_boundary", position),
        ReviewIssue::Classification(position) => ("classification", position),
        ReviewIssue::Relationship(position) => ("relationship", position),
        ReviewIssue::DecisionComparison(position) => ("decision_comparison", position),
        ReviewIssue::NoteClassification(position) => ("note_classification", position),
        ReviewIssue::NoteDecisionComparison(position) => ("note_decision_comparison", position),
    }
}
