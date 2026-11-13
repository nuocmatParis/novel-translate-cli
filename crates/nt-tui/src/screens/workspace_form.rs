use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use std::thread;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use nt_application::workspace::{WorkspaceError, WorkspaceMetadata};

use crate::WorkspaceCommands;
use crate::input::EventLayer;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkspaceOperation {
    Create,
    Open,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FormStatus {
    Idle,
    Editing(WorkspaceOperation),
    Pending {
        operation: WorkspaceOperation,
        request_id: u64,
    },
    Complete {
        operation: WorkspaceOperation,
        metadata: WorkspaceMetadata,
    },
    Failed {
        operation: WorkspaceOperation,
        error: WorkspaceError,
    },
}

pub(crate) struct WorkspaceForm {
    commands: Option<Arc<dyn WorkspaceCommands>>,
    current_workspace: Option<WorkspaceMetadata>,
    status: FormStatus,
    path: String,
    next_request_id: u64,
    result_sender: SyncSender<(u64, Result<WorkspaceMetadata, WorkspaceError>)>,
    result_receiver: Receiver<(u64, Result<WorkspaceMetadata, WorkspaceError>)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkspaceFormSnapshot {
    status: FormStatus,
    current_workspace: Option<WorkspaceMetadata>,
    path: String,
}

impl WorkspaceForm {
    pub(crate) fn new(commands: Option<Arc<dyn WorkspaceCommands>>) -> Self {
        let (result_sender, result_receiver) = mpsc::sync_channel(1);
        Self {
            commands,
            current_workspace: None,
            status: FormStatus::Idle,
            path: String::new(),
            next_request_id: 1,
            result_sender,
            result_receiver,
        }
    }

    pub(crate) fn poll_result(&mut self) -> bool {
        let mut changed = false;
        loop {
            match self.result_receiver.try_recv() {
                Ok((request_id, result)) => {
                    let operation = match self.status {
                        FormStatus::Pending {
                            operation,
                            request_id: active_id,
                        } if active_id == request_id => operation,
                        _ => continue,
                    };
                    self.status = match result {
                        Ok(metadata) => {
                            self.current_workspace = Some(metadata.clone());
                            FormStatus::Complete {
                                operation,
                                metadata,
                            }
                        }
                        Err(error) => FormStatus::Failed { operation, error },
                    };
                    changed = true;
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return changed,
            }
        }
    }

    pub(crate) fn snapshot(&self) -> WorkspaceFormSnapshot {
        WorkspaceFormSnapshot {
            status: self.status.clone(),
            current_workspace: self.current_workspace.clone(),
            path: self.path.clone(),
        }
    }

    fn begin(&mut self, operation: WorkspaceOperation) {
        self.path.clear();
        self.status = FormStatus::Editing(operation);
    }

    fn submit(&mut self, operation: WorkspaceOperation) {
        if self.path.is_empty() {
            self.status = FormStatus::Failed {
                operation,
                error: WorkspaceError::InvalidPath,
            };
            return;
        }

        let Some(commands) = self.commands.as_ref().map(Arc::clone) else {
            return;
        };
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
        let path = PathBuf::from(&self.path);
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
                let _ = sender.send((request_id, result));
            });
        if spawn.is_err() {
            self.status = FormStatus::Failed {
                operation,
                error: WorkspaceError::StorageFailure,
            };
        }
    }

    fn append_character(&mut self, character: char) {
        self.path.push(character);
    }

    fn remove_last_grapheme(&mut self) {
        use unicode_segmentation::UnicodeSegmentation;

        if let Some((byte_index, _)) = self.path.grapheme_indices(true).next_back() {
            self.path.truncate(byte_index);
        }
    }
}

impl EventLayer for WorkspaceForm {
    fn handle_event(&mut self, event: &Event) -> bool {
        let is_editing = matches!(self.status, FormStatus::Editing(_));
        let is_pending = matches!(self.status, FormStatus::Pending { .. });
        if is_editing || is_pending {
            let Event::Key(key) = event else {
                return true;
            };
            if is_pending {
                return true;
            }

            let FormStatus::Editing(operation) = self.status else {
                return true;
            };
            match (key.code, key.kind) {
                (KeyCode::Esc, KeyEventKind::Press) if no_modifiers(key.modifiers) => {
                    self.path.clear();
                    self.status = FormStatus::Idle;
                }
                (KeyCode::Enter, KeyEventKind::Press) if no_modifiers(key.modifiers) => {
                    self.submit(operation);
                }
                (KeyCode::Backspace, KeyEventKind::Press | KeyEventKind::Repeat)
                    if no_modifiers(key.modifiers) =>
                {
                    self.remove_last_grapheme();
                }
                (KeyCode::Char(character), KeyEventKind::Press | KeyEventKind::Repeat)
                    if no_modifiers(key.modifiers) =>
                {
                    self.append_character(character);
                }
                _ => {}
            }
            return true;
        }

        let Event::Key(key) = event else {
            return false;
        };
        if matches!(self.status, FormStatus::Failed { .. })
            && key.code == KeyCode::Esc
            && key.kind == KeyEventKind::Press
            && no_modifiers(key.modifiers)
        {
            self.path.clear();
            self.status = FormStatus::Idle;
            return true;
        }
        if key.kind != KeyEventKind::Press || !no_modifiers(key.modifiers) {
            return false;
        }
        match key.code {
            KeyCode::Char('c' | 'C') if self.commands.is_some() => {
                self.begin(WorkspaceOperation::Create);
                true
            }
            KeyCode::Char('o' | 'O') if self.commands.is_some() => {
                self.begin(WorkspaceOperation::Open);
                true
            }
            _ => false,
        }
    }
}

impl WorkspaceFormSnapshot {
    pub(crate) fn heading(&self) -> &'static str {
        match self.status {
            FormStatus::Idle | FormStatus::Editing(_) | FormStatus::Pending { .. } => {
                "Chưa mở workspace"
            }
            FormStatus::Complete { .. } => "Workspace đã sẵn sàng",
            FormStatus::Failed { operation, .. } => match operation {
                WorkspaceOperation::Create => "Không thể tạo workspace",
                WorkspaceOperation::Open => "Không thể mở workspace",
            },
        }
    }

    pub(crate) fn body(&self) -> String {
        match &self.status {
            FormStatus::Idle => format!(
                "{}\nC: Tạo workspace · O: Mở workspace",
                active_workspace_message(self.current_workspace.as_ref())
            ),
            FormStatus::Editing(operation) => format!(
                "{}\n{} workspace\nĐường dẫn: {}_\nNhập đường dẫn Unicode · Enter: xác nhận · Esc: hủy",
                active_workspace_message(self.current_workspace.as_ref()),
                operation.verb(),
                self.path
            ),
            FormStatus::Pending { operation, .. } => format!(
                "{}\n{} workspace\nĐường dẫn: {}\nĐang xử lý; bàn phím vẫn hoạt động.",
                active_workspace_message(self.current_workspace.as_ref()),
                operation.verb(),
                self.path
            ),
            FormStatus::Complete {
                operation,
                metadata,
            } => format!(
                "Đã {} workspace.\nSeries: {}\nSchema: {} · Generation: {}\nC: Tạo workspace · O: Mở workspace",
                operation.past_tense(),
                metadata.series.id.as_str(),
                metadata.schema_version,
                metadata.current_generation.as_str()
            ),
            FormStatus::Failed { error, .. } => format!(
                "{}\n{}\nĐường dẫn: {}",
                workspace_error_message(error),
                active_workspace_message(self.current_workspace.as_ref()),
                self.path
            ),
        }
    }

    pub(crate) fn help_summary(&self) -> String {
        match &self.status {
            FormStatus::Idle => {
                "Tổng quan: C tạo workspace, O mở workspace; nhập đường dẫn rồi nhấn Enter."
                    .to_owned()
            }
            FormStatus::Editing(operation) => format!(
                "Đang {} workspace. Nhập đường dẫn Unicode rồi Enter để tiếp tục; Esc hủy.",
                operation.verb().to_lowercase()
            ),
            FormStatus::Pending { operation, .. } => format!(
                "Đang {} workspace ở tác vụ nền. Chờ kết quả; Esc đóng trợ giúp để xem trạng thái.",
                operation.verb().to_lowercase()
            ),
            FormStatus::Complete { operation, .. } => format!(
                "Đã {} workspace. C tạo workspace khác, O mở workspace khác.",
                operation.past_tense()
            ),
            FormStatus::Failed { operation, error } => format!(
                "Không thể {} workspace. {} C hoặc O để thử thao tác khác; Esc đóng lỗi.",
                operation.verb().to_lowercase(),
                workspace_error_message(error)
            ),
        }
    }

    pub(crate) const fn footer_text(&self) -> &'static str {
        match self.status {
            FormStatus::Idle | FormStatus::Complete { .. } => {
                "C tạo · O mở · F1 trợ giúp · Tab thoát · Ctrl+C thoát"
            }
            FormStatus::Editing(_) => "Enter xác nhận · Esc hủy · F1 trợ giúp · Ctrl+C thoát",
            FormStatus::Pending { .. } => "Đang xử lý · F1 trợ giúp · Ctrl+C thoát",
            FormStatus::Failed { .. } => "Esc đóng lỗi · C tạo · O mở · F1 trợ giúp · Ctrl+C thoát",
        }
    }
}

impl WorkspaceOperation {
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

fn active_workspace_message(workspace: Option<&WorkspaceMetadata>) -> String {
    let Some(workspace) = workspace else {
        return "Chưa có workspace nào đang mở.".to_owned();
    };
    format!(
        "Workspace đang mở · Series: {} · Schema: {} · Generation: {}",
        workspace.series.id.as_str(),
        workspace.schema_version,
        workspace.current_generation.as_str()
    )
}

fn no_modifiers(modifiers: KeyModifiers) -> bool {
    modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT
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
mod tests {
    use super::{FormStatus, WorkspaceForm, WorkspaceOperation};
    use crate::WorkspaceCommands;
    use crate::input::{EventLayer, UiAction, dispatch_event};
    use crate::state::AppState;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use nt_application::workspace::{WorkspaceError, WorkspaceMetadata};
    use std::path::{Path, PathBuf};
    use std::sync::mpsc::{self, Receiver, SyncSender};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    struct FailingCommands;

    impl WorkspaceCommands for FailingCommands {
        fn create_workspace(&self, _path: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
            Err(WorkspaceError::StorageFailure)
        }

        fn open_workspace(&self, _path: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
            Err(WorkspaceError::StorageFailure)
        }
    }

    struct BlockingCommands {
        started: SyncSender<(WorkspaceOperation, PathBuf)>,
        release: Arc<Mutex<Receiver<()>>>,
    }

    impl BlockingCommands {
        fn call(
            &self,
            operation: WorkspaceOperation,
            path: &Path,
        ) -> Result<WorkspaceMetadata, WorkspaceError> {
            self.started
                .send((operation, path.to_path_buf()))
                .expect("test receives the started operation");
            self.release
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .recv_timeout(Duration::from_secs(3))
                .expect("test releases the deliberately blocked operation");
            Err(WorkspaceError::StorageFailure)
        }
    }

    impl WorkspaceCommands for BlockingCommands {
        fn create_workspace(&self, path: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
            self.call(WorkspaceOperation::Create, path)
        }

        fn open_workspace(&self, path: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
            self.call(WorkspaceOperation::Open, path)
        }
    }

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new_with_kind(
            code,
            KeyModifiers::NONE,
            KeyEventKind::Press,
        ))
    }

    fn type_text(form: &mut WorkspaceForm, text: &str) {
        for character in text.chars() {
            assert!(form.handle_event(&key(KeyCode::Char(character))));
        }
    }

    #[test]
    fn edit_captures_navigation_and_backspace_removes_a_whole_grapheme() {
        let commands: Arc<dyn WorkspaceCommands> = Arc::new(FailingCommands);
        let mut form = WorkspaceForm::new(Some(commands));
        let mut state = AppState::default();

        assert_eq!(
            dispatch_event(&mut state, &key(KeyCode::Char('c')), None, Some(&mut form),),
            UiAction::None
        );
        type_text(&mut form, "A e\u{301}");
        assert!(form.handle_event(&key(KeyCode::Backspace)));
        assert_eq!(form.path, "A ");

        let selected = state.selected_screen();
        assert_eq!(
            dispatch_event(&mut state, &key(KeyCode::Down), None, Some(&mut form)),
            UiAction::None
        );
        assert_eq!(state.selected_screen(), selected);
        assert!(form.handle_event(&key(KeyCode::Esc)));
        assert_eq!(form.status, FormStatus::Idle);
        assert!(form.path.is_empty());
    }

    #[test]
    fn empty_submission_fails_without_calling_the_workspace_service() {
        let mut form = WorkspaceForm::new(Some(Arc::new(FailingCommands)));
        assert!(form.handle_event(&key(KeyCode::Char('o'))));
        assert!(form.handle_event(&key(KeyCode::Enter)));
        assert_eq!(
            form.status,
            FormStatus::Failed {
                operation: WorkspaceOperation::Open,
                error: WorkspaceError::InvalidPath,
            }
        );
    }

    #[test]
    fn failed_workspace_errors_show_recovery_and_visible_dismissal_actions() {
        let mut form = WorkspaceForm::new(Some(Arc::new(FailingCommands)));
        form.status = FormStatus::Failed {
            operation: WorkspaceOperation::Open,
            error: WorkspaceError::CorruptWorkspace,
        };
        form.path = "series_日本語 01".to_owned();
        let snapshot = form.snapshot();
        let body = snapshot.body();
        assert!(body.contains("Khôi phục từ bản sao lưu đã kiểm chứng"));
        assert!(body.contains("Giữ nguyên thư mục hiện tại"));
        assert!(snapshot.footer_text().contains("Esc đóng lỗi"));
        assert!(snapshot.help_summary().contains("Esc đóng lỗi"));

        form.status = FormStatus::Failed {
            operation: WorkspaceOperation::Open,
            error: WorkspaceError::PermissionDenied,
        };
        let permission = form.snapshot();
        assert!(
            permission
                .body()
                .contains("Kiểm tra quyền truy cập tệp/thư mục")
        );
        assert!(permission.footer_text().contains("Esc đóng lỗi"));

        form.status = FormStatus::Failed {
            operation: WorkspaceOperation::Open,
            error: WorkspaceError::Busy,
        };
        let busy = form.snapshot();
        assert!(busy.body().contains("Đóng tiến trình khác rồi thử lại"));
    }

    #[test]
    fn pending_workspace_help_describes_the_active_operation() {
        let mut form = WorkspaceForm::new(Some(Arc::new(FailingCommands)));
        form.status = FormStatus::Pending {
            operation: WorkspaceOperation::Open,
            request_id: 42,
        };
        let snapshot = form.snapshot();
        let summary = snapshot.help_summary();
        assert!(summary.contains("Đang mở workspace ở tác vụ nền"));
        assert!(summary.contains("Chờ kết quả"));
        assert!(!summary.contains("chưa khả dụng"));
        assert!(snapshot.footer_text().contains("Đang xử lý"));
    }

    #[test]
    fn overview_render_shows_recovery_help_and_dismissal_at_eighty_by_twenty_four() {
        use crate::navigation::Screen;
        use crate::rendering::render_with_workspace;
        use crate::state::AppState;
        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;

        fn rendered(state: &AppState, form: &WorkspaceForm) -> String {
            let area = Rect::new(0, 0, 80, 24);
            let mut buffer = Buffer::empty(area);
            render_with_workspace(&mut buffer, area, state, &form.snapshot());
            (0..area.height)
                .map(|row| {
                    (0..area.width)
                        .map(|column| buffer[(column, row)].symbol())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n")
        }

        let mut form = WorkspaceForm::new(Some(Arc::new(FailingCommands)));
        form.status = FormStatus::Failed {
            operation: WorkspaceOperation::Open,
            error: WorkspaceError::CorruptWorkspace,
        };
        let state = AppState::default();
        assert_eq!(state.current_screen(), Screen::Overview);
        let frame = rendered(&state, &form);
        let visible_words = frame.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(visible_words.contains("Khôi phục"), "{visible_words}");
        assert!(visible_words.contains("bản sao lưu"), "{visible_words}");
        assert!(visible_words.contains("đã kiểm"), "{visible_words}");
        assert!(visible_words.contains("chứng."), "{visible_words}");
        assert!(frame.contains("Esc đóng lỗi"));

        form.status = FormStatus::Pending {
            operation: WorkspaceOperation::Open,
            request_id: 7,
        };
        let mut state = AppState::default();
        state.open_help();
        let help = rendered(&state, &form);
        assert!(help.contains("Đang mở workspace ở tác vụ nền"));
        assert!(help.contains("Chờ kết quả"));
        assert!(!help.contains("chưa khả dụng"));
    }

    #[test]
    fn blocking_io_runs_off_the_event_loop_and_result_returns_to_its_request() {
        let (started_sender, started_receiver) = mpsc::sync_channel(1);
        let (release_sender, release_receiver) = mpsc::channel();
        let commands = BlockingCommands {
            started: started_sender,
            release: Arc::new(Mutex::new(release_receiver)),
        };
        let mut form = WorkspaceForm::new(Some(Arc::new(commands)));
        let expected_path = PathBuf::from("C:\\Manga\\異世界 01");

        assert!(form.handle_event(&key(KeyCode::Char('c'))));
        type_text(
            &mut form,
            expected_path.to_str().expect("test path is Unicode"),
        );
        assert!(form.handle_event(&key(KeyCode::Enter)));
        assert!(matches!(form.status, FormStatus::Pending { .. }));
        assert_eq!(
            started_receiver
                .recv_timeout(Duration::from_secs(1))
                .expect("storage call begins on a worker"),
            (WorkspaceOperation::Create, expected_path.clone())
        );

        let event_started = Instant::now();
        assert!(form.handle_event(&key(KeyCode::Right)));
        assert!(event_started.elapsed() < Duration::from_millis(100));
        assert!(matches!(form.status, FormStatus::Pending { .. }));
        assert_eq!(form.path, expected_path.to_string_lossy());

        release_sender
            .send(())
            .expect("worker is waiting for release");
        let deadline = Instant::now() + Duration::from_secs(2);
        while !form.poll_result() {
            assert!(Instant::now() < deadline, "worker result returns in time");
            thread::sleep(Duration::from_millis(5));
        }
        assert!(matches!(
            form.status,
            FormStatus::Failed {
                operation: WorkspaceOperation::Create,
                error: WorkspaceError::StorageFailure,
            }
        ));
        assert!(
            form.snapshot()
                .body()
                .contains("Không thể lưu hoặc đọc workspace an toàn.")
        );
        assert!(form.handle_event(&key(KeyCode::Esc)));
        assert_eq!(form.status, FormStatus::Idle);
    }

    #[test]
    fn stale_result_does_not_replace_the_active_request() {
        let mut form = WorkspaceForm::new(Some(Arc::new(FailingCommands)));
        form.status = FormStatus::Pending {
            operation: WorkspaceOperation::Open,
            request_id: 9,
        };
        form.result_sender
            .send((8, Err(WorkspaceError::NotFound)))
            .expect("stale test result enters the channel");
        assert!(!form.poll_result());
        assert!(matches!(
            form.status,
            FormStatus::Pending { request_id: 9, .. }
        ));

        form.result_sender
            .send((9, Err(WorkspaceError::StorageFailure)))
            .expect("current test result enters the channel");
        assert!(form.poll_result());
        assert!(matches!(
            form.status,
            FormStatus::Failed {
                operation: WorkspaceOperation::Open,
                error: WorkspaceError::StorageFailure,
            }
        ));
    }

    #[test]
    fn failed_open_keeps_the_requested_path_visible_for_recovery() {
        let mut form = WorkspaceForm::new(Some(Arc::new(FailingCommands)));
        form.path = "corrupt workspace fixture".to_owned();
        form.status = FormStatus::Pending {
            operation: WorkspaceOperation::Open,
            request_id: 12,
        };
        form.result_sender
            .send((12, Err(WorkspaceError::CorruptWorkspace)))
            .expect("corrupt-open result enters the result channel");

        assert!(form.poll_result());
        assert_eq!(form.path, "corrupt workspace fixture");
        assert!(
            form.snapshot()
                .body()
                .contains("Đường dẫn: corrupt workspace fixture")
        );
    }
}
