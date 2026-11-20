use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use nt_application::workspace::{WorkspaceError, WorkspaceMetadata};

use crate::input::EventLayer;
use crate::{FolderPickerError, WorkspaceCommands};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkspaceOperation {
    Create,
    Open,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FormControl {
    Choose,
    Confirm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OverviewAction {
    Import,
    Create,
    Open,
}

impl From<WorkspaceOperation> for OverviewAction {
    fn from(operation: WorkspaceOperation) -> Self {
        match operation {
            WorkspaceOperation::Create => Self::Create,
            WorkspaceOperation::Open => Self::Open,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FormStatus {
    Idle,
    Editing(WorkspaceOperation),
    Choosing {
        operation: WorkspaceOperation,
        request_id: u64,
    },
    Pending {
        operation: WorkspaceOperation,
        request_id: u64,
    },
    Complete,
    Failed {
        operation: WorkspaceOperation,
        error: WorkspaceError,
    },
}

enum Reply {
    Folder {
        request_id: u64,
        result: Result<Option<PathBuf>, FolderPickerError>,
    },
    Workspace {
        request_id: u64,
        path: PathBuf,
        result: Result<WorkspaceMetadata, WorkspaceError>,
    },
}

pub(crate) struct WorkspaceForm {
    commands: Option<Arc<dyn WorkspaceCommands>>,
    opened: Option<(u64, PathBuf, WorkspaceMetadata)>,
    status: FormStatus,
    path: Option<PathBuf>,
    chooser_error: Option<FolderPickerError>,
    panel_focus: bool,
    entry: OverviewAction,
    import_available: bool,
    control: FormControl,
    next_request_id: u64,
    result_sender: SyncSender<Reply>,
    result_receiver: Receiver<Reply>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkspaceFormSnapshot {
    status: FormStatus,
    current_workspace: Option<WorkspaceMetadata>,
    path: Option<PathBuf>,
    chooser_error: Option<FolderPickerError>,
    panel_focus: bool,
    entry: OverviewAction,
    control: FormControl,
}

impl WorkspaceForm {
    pub(crate) fn new(commands: Option<Arc<dyn WorkspaceCommands>>) -> Self {
        let (result_sender, result_receiver) = mpsc::sync_channel(1);
        Self {
            commands,
            opened: None,
            status: FormStatus::Idle,
            path: None,
            chooser_error: None,
            panel_focus: false,
            entry: OverviewAction::Create,
            import_available: false,
            control: FormControl::Choose,
            next_request_id: 1,
            result_sender,
            result_receiver,
        }
    }

    pub(crate) fn available(&self) -> bool {
        self.commands.is_some()
    }

    pub(crate) fn set_panel_focus(&mut self, focused: bool, import_available: bool) {
        if (focused && !self.panel_focus && !self.is_active_form())
            || self.import_available != import_available
        {
            self.entry = if import_available {
                OverviewAction::Import
            } else {
                OverviewAction::Create
            };
        }
        self.panel_focus = focused;
        self.import_available = import_available;
    }

    pub(crate) fn focus_import(&mut self) {
        self.entry = OverviewAction::Import;
    }

    pub(crate) fn import_focused(&self) -> bool {
        self.panel_focus && self.entry == OverviewAction::Import
    }

    pub(crate) fn opened(&self) -> Option<(u64, &Path, &WorkspaceMetadata)> {
        self.opened
            .as_ref()
            .map(|(epoch, path, metadata)| (*epoch, path.as_path(), metadata))
    }

    pub(crate) fn route_left(&mut self) {
        if matches!(
            self.status,
            FormStatus::Choosing { .. } | FormStatus::Editing(_) | FormStatus::Failed { .. }
        ) {
            self.dismiss();
        }
    }

    pub(crate) fn is_active_form(&self) -> bool {
        matches!(
            self.status,
            FormStatus::Editing(_)
                | FormStatus::Choosing { .. }
                | FormStatus::Pending { .. }
                | FormStatus::Failed { .. }
        )
    }

    pub(crate) fn poll_result(&mut self) -> bool {
        let mut changed = false;
        while let Ok(reply) = self.result_receiver.try_recv() {
            changed |= self.apply_reply(reply);
        }
        changed
    }

    fn apply_reply(&mut self, reply: Reply) -> bool {
        match (reply, &self.status) {
            (
                Reply::Folder { request_id, result },
                FormStatus::Choosing {
                    operation,
                    request_id: active_id,
                },
            ) if request_id == *active_id => {
                let operation = *operation;
                match result {
                    Ok(Some(path)) => {
                        self.path = Some(path);
                        self.control = FormControl::Confirm;
                    }
                    Ok(None) => {}
                    Err(error) => self.chooser_error = Some(error),
                }
                self.status = FormStatus::Editing(operation);
                true
            }
            (
                Reply::Workspace {
                    request_id,
                    path,
                    result,
                },
                FormStatus::Pending {
                    operation,
                    request_id: active_id,
                },
            ) if request_id == *active_id => {
                let operation = *operation;
                self.status = match result {
                    Ok(metadata) => {
                        self.opened = Some((request_id, path, metadata));
                        if self.import_available {
                            self.focus_import();
                        }
                        FormStatus::Complete
                    }
                    Err(error) => FormStatus::Failed { operation, error },
                };
                true
            }
            _ => false,
        }
    }

    pub(crate) fn snapshot(&self) -> WorkspaceFormSnapshot {
        WorkspaceFormSnapshot {
            status: self.status.clone(),
            current_workspace: self
                .opened
                .as_ref()
                .map(|(_, _, metadata)| metadata.clone()),
            path: self.path.clone(),
            chooser_error: self.chooser_error,
            panel_focus: self.panel_focus,
            entry: self.entry,
            control: self.control,
        }
    }

    fn request_id(&mut self) -> u64 {
        let request = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
        request
    }

    fn begin(&mut self, operation: WorkspaceOperation) {
        self.entry = operation.into();
        self.path = None;
        self.chooser_error = None;
        self.control = FormControl::Choose;
        self.status = FormStatus::Editing(operation);
    }

    fn dismiss(&mut self) {
        self.path = None;
        self.chooser_error = None;
        self.status = FormStatus::Idle;
    }

    fn choose(&mut self, operation: WorkspaceOperation) {
        let Some(commands) = self.commands.as_ref().map(Arc::clone) else {
            return;
        };
        let request_id = self.request_id();
        let sender = self.result_sender.clone();
        self.chooser_error = None;
        self.control = FormControl::Choose;
        self.status = FormStatus::Choosing {
            operation,
            request_id,
        };
        let spawn = thread::Builder::new()
            .name("nt-workspace-folder-picker".to_owned())
            .spawn(move || {
                let result = commands.choose_workspace_folder();
                let _ = sender.send(Reply::Folder { request_id, result });
            });
        if spawn.is_err() {
            self.status = FormStatus::Editing(operation);
            self.chooser_error = Some(FolderPickerError::Unavailable);
        }
    }

    fn submit(&mut self, operation: WorkspaceOperation) {
        let (Some(path), Some(commands)) =
            (self.path.clone(), self.commands.as_ref().map(Arc::clone))
        else {
            return;
        };
        let request_id = self.request_id();
        let sender = self.result_sender.clone();
        self.status = FormStatus::Pending {
            operation,
            request_id,
        };
        let spawn = thread::Builder::new()
            .name("nt-workspace-form".to_owned())
            .spawn(move || {
                let result = match operation {
                    WorkspaceOperation::Create => commands.create_workspace(&path),
                    WorkspaceOperation::Open => commands.open_workspace(&path),
                };
                let _ = sender.send(Reply::Workspace {
                    request_id,
                    path,
                    result,
                });
            });
        if spawn.is_err() {
            self.status = FormStatus::Failed {
                operation,
                error: WorkspaceError::StorageFailure,
            };
        }
    }

    pub(crate) fn click(&mut self, page_row: u16, column: u16) -> bool {
        if !self.available() {
            return false;
        }
        let op = match self.status {
            FormStatus::Editing(operation) | FormStatus::Failed { operation, .. } => operation,
            FormStatus::Idle | FormStatus::Complete => {
                match page_row {
                    0 if in_button(column, "Tạo workspace") => {
                        self.begin(WorkspaceOperation::Create)
                    }
                    1 if in_button(column, "Mở workspace") => {
                        self.begin(WorkspaceOperation::Open)
                    }
                    _ => return false,
                }
                return true;
            }
            FormStatus::Choosing { .. } | FormStatus::Pending { .. } => return false,
        };
        match page_row {
            0 if in_button(column, op.browse()) => self.choose(op),
            1 if self.path.is_some() && in_button(column, op.verb()) => self.submit(op),
            _ => return false,
        }
        true
    }

    fn form_key(&mut self, key: KeyEvent, operation: WorkspaceOperation) {
        match key.code {
            KeyCode::Esc => self.dismiss(),
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Up | KeyCode::Down
                if self.path.is_some() =>
            {
                self.control = match self.control {
                    FormControl::Choose => FormControl::Confirm,
                    FormControl::Confirm => FormControl::Choose,
                };
            }
            KeyCode::Enter if self.control == FormControl::Choose => self.choose(operation),
            KeyCode::Enter => self.submit(operation),
            _ => {}
        }
    }
}

impl EventLayer for WorkspaceForm {
    fn handle_event(&mut self, event: &Event) -> bool {
        let active = self.is_active_form();
        let Event::Key(key) = *event else {
            return active;
        };
        if key.kind != KeyEventKind::Press
            || !(key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT)
        {
            return active;
        }
        if matches!(key.code, KeyCode::PageUp | KeyCode::PageDown) {
            return false;
        }
        if matches!(self.status, FormStatus::Pending { .. }) {
            return true;
        }

        let shortcut = match key.code {
            KeyCode::Char('c' | 'C') => Some(WorkspaceOperation::Create),
            KeyCode::Char('o' | 'O') => Some(WorkspaceOperation::Open),
            _ => None,
        };
        if self.available()
            && let Some(operation) = shortcut
        {
            self.begin(operation);
            return true;
        }
        match self.status {
            FormStatus::Choosing { operation, .. } => {
                if key.code == KeyCode::Esc {
                    self.status = FormStatus::Editing(operation);
                }
                true
            }
            FormStatus::Editing(operation) | FormStatus::Failed { operation, .. } => {
                self.form_key(key, operation);
                true
            }
            _ if self.panel_focus && self.available() => {
                let control = match (key.code, key.modifiers) {
                    (KeyCode::Tab, KeyModifiers::SHIFT) => KeyCode::BackTab,
                    (code, _) => code,
                };
                match (control, self.entry) {
                    (KeyCode::Enter, OverviewAction::Create) => {
                        self.begin(WorkspaceOperation::Create);
                    }
                    (KeyCode::Enter, OverviewAction::Open) => {
                        self.begin(WorkspaceOperation::Open);
                    }
                    (KeyCode::Tab | KeyCode::Right | KeyCode::Down, OverviewAction::Import) => {
                        self.entry = OverviewAction::Create;
                    }
                    (KeyCode::Tab | KeyCode::Right | KeyCode::Down, OverviewAction::Create) => {
                        self.entry = OverviewAction::Open;
                    }
                    (KeyCode::BackTab | KeyCode::Left | KeyCode::Up, OverviewAction::Open) => {
                        self.entry = OverviewAction::Create;
                    }
                    (KeyCode::BackTab | KeyCode::Left | KeyCode::Up, OverviewAction::Create)
                        if self.import_available =>
                    {
                        self.focus_import();
                    }
                    _ => return false,
                }
                true
            }
            _ => false,
        }
    }
}

impl WorkspaceFormSnapshot {
    pub(crate) fn heading(&self) -> &'static str {
        match self.status {
            FormStatus::Editing(operation) | FormStatus::Choosing { operation, .. } => {
                operation.title()
            }
            FormStatus::Pending { operation, .. } => operation.title(),
            FormStatus::Failed { operation, .. } => match operation {
                WorkspaceOperation::Create => "Không thể tạo workspace",
                WorkspaceOperation::Open => "Không thể mở workspace",
            },
            _ if self.current_workspace.is_some() => "Workspace đã sẵn sàng",
            _ => "Chưa mở workspace",
        }
    }

    pub(crate) fn body(&self) -> String {
        let active = active_workspace_message(self.current_workspace.as_ref());
        match &self.status {
            FormStatus::Idle | FormStatus::Complete => {
                format!("{}\n{active}", self.entry_buttons())
            }
            FormStatus::Choosing { operation, .. } => {
                format!(
                    "{}\nĐang mở hộp chọn thư mục…\n{}\n{active}",
                    button(operation.browse(), true),
                    self.selection()
                )
            }
            FormStatus::Editing(operation) | FormStatus::Failed { operation, .. } => {
                let mut body = button(operation.browse(), self.control == FormControl::Choose);
                if self.path.is_some() {
                    body.push_str(&format!(
                        "\n{}",
                        button(operation.verb(), self.control == FormControl::Confirm)
                    ));
                }
                if let FormStatus::Failed { error, .. } = &self.status {
                    body.push_str(&format!("\n{}", workspace_error_message(error)));
                }
                if let Some(error) = self.chooser_error {
                    body.push_str(&format!("\n{}", picker_error_message(error)));
                }
                body.push_str(&format!("\n{}\n{active}", self.selection()));
                body
            }
            FormStatus::Pending { operation, .. } => {
                format!(
                    "Đang {} workspace…\n{}\n{active}",
                    operation.past_tense(),
                    self.selection()
                )
            }
        }
    }

    fn entry_buttons(&self) -> String {
        format!(
            "{}\n{}",
            button(
                "Tạo workspace",
                self.panel_focus && self.entry == OverviewAction::Create,
            ),
            button(
                "Mở workspace",
                self.panel_focus && self.entry == OverviewAction::Open,
            ),
        )
    }

    fn selection(&self) -> String {
        let Some(path) = &self.path else {
            return "Chưa chọn thư mục.".to_owned();
        };
        let visible: String = path
            .to_string_lossy()
            .chars()
            .filter(|c| !c.is_control())
            .collect();
        format!("Nơi đã chọn: {visible}")
    }

    pub(crate) fn help_summary(&self) -> String {
        match self.status {
            FormStatus::Idle | FormStatus::Complete => {
                "Tổng quan: Tab đến Tạo hoặc Mở, nhấn Enter để vào form.".to_owned()
            }
            FormStatus::Choosing { .. } => {
                "Đang chọn thư mục. Hủy trong hộp chọn để quay lại form.".to_owned()
            }
            FormStatus::Pending { operation, .. } => format!(
                "Đang {} workspace ở tác vụ nền. Chờ kết quả.",
                operation.past_tense()
            ),
            FormStatus::Editing(_) => {
                "Chọn thư mục rồi dùng Tạo hoặc Mở để xác nhận. Tab đổi nút, Esc đóng form."
                    .to_owned()
            }
            FormStatus::Failed { .. } => {
                "Chọn lại thư mục hoặc thử lại. Tab đổi nút, Esc đóng lỗi.".to_owned()
            }
        }
    }

    pub(crate) const fn footer_text(&self) -> &'static str {
        match self.status {
            FormStatus::Idle | FormStatus::Complete => {
                "Tab chọn nút · Enter mở form · C tạo · O mở · F1 trợ giúp"
            }
            FormStatus::Choosing { .. } => "Đang chọn thư mục · Esc hủy · F1 trợ giúp",
            FormStatus::Editing(_) => "Tab đổi nút · Enter chọn · Esc đóng form · F1 trợ giúp",
            FormStatus::Failed { .. } => "Tab đổi nút · Enter thử lại · Esc đóng lỗi · F1 trợ giúp",
            FormStatus::Pending { .. } => "Đang xử lý · F1 trợ giúp · Ctrl+C thoát",
        }
    }
}

impl WorkspaceOperation {
    const fn title(self) -> &'static str {
        match self {
            Self::Create => "Tạo workspace",
            Self::Open => "Mở workspace",
        }
    }

    const fn browse(self) -> &'static str {
        match self {
            Self::Create => "Chọn nơi lưu…",
            Self::Open => "Chọn workspace…",
        }
    }

    const fn verb(self) -> &'static str {
        match self {
            Self::Create => "Tạo",
            Self::Open => "Mở",
        }
    }

    const fn past_tense(self) -> &'static str {
        match self {
            Self::Create => "tạo",
            Self::Open => "mở",
        }
    }
}

fn button(label: &str, focused: bool) -> String {
    let marker = if focused { "› " } else { "  " };
    format!("{marker}[{label}]")
}

fn in_button(column: u16, label: &str) -> bool {
    usize::from(column) < label.chars().count() + 4
}

fn active_workspace_message(workspace: Option<&WorkspaceMetadata>) -> String {
    let Some(workspace) = workspace else {
        return "Chưa có workspace nào đang mở.".to_owned();
    };
    format!(
        "Workspace đang mở · Series {} · Schema {}",
        workspace.series.id.as_str(),
        workspace.schema_version,
    )
}

fn picker_error_message(error: FolderPickerError) -> &'static str {
    match error {
        FolderPickerError::Busy => "Hãy đóng hộp chọn đang mở rồi thử lại.",
        FolderPickerError::Unavailable => {
            "Không mở được hộp chọn thư mục. Enter để thử lại hoặc Esc để trở về."
        }
        FolderPickerError::PermissionDenied => {
            "Thiếu quyền chọn thư mục. Kiểm tra quyền rồi thử lại."
        }
    }
}

fn workspace_error_message(error: &WorkspaceError) -> &'static str {
    match error {
        WorkspaceError::InvalidPath => "Đường dẫn workspace không hợp lệ.",
        WorkspaceError::UnsupportedFilesystem => "Workspace phải nằm trên ổ cục bộ được hỗ trợ.",
        WorkspaceError::AlreadyExists => "Workspace đã tồn tại tại vị trí này.",
        WorkspaceError::DirectoryNotEmpty => "Thư mục đích không trống.",
        WorkspaceError::NotFound => "Không tìm thấy workspace.",
        WorkspaceError::PermissionDenied => {
            "Thiếu quyền. Kiểm tra quyền truy cập tệp/thư mục rồi thử lại."
        }
        WorkspaceError::Busy => "Workspace đang bận. Đóng tiến trình khác rồi thử lại.",
        WorkspaceError::LegacyGoWorkspace => "Đây là workspace Go cũ; hãy tạo workspace Rust mới.",
        WorkspaceError::UnsupportedFormat => "Định dạng workspace không được hỗ trợ.",
        WorkspaceError::NewerSchema { .. } => "Workspace cần phiên bản ứng dụng mới hơn.",
        WorkspaceError::CorruptWorkspace => {
            "Workspace lỗi. Giữ nguyên thư mục hiện tại; Khôi phục từ bản sao lưu đã kiểm chứng."
        }
        WorkspaceError::StorageFailure => {
            "Không thể lưu hoặc đọc workspace an toàn. Kiểm tra ổ đĩa và thử mở lại; dữ liệu cũ vẫn được giữ."
        }
    }
}

#[cfg(test)]
#[path = "workspace_folder_tests.rs"]
mod tests;
