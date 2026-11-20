use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use super::{
    FolderPickerError, FormControl, FormStatus, Reply, WorkspaceCommands, WorkspaceError,
    WorkspaceForm, WorkspaceMetadata, WorkspaceOperation,
};
use crate::input::{EventLayer, UiAction, dispatch_event};
use crate::state::AppState;

struct Commands {
    picker: Mutex<mpsc::Receiver<Result<Option<PathBuf>, FolderPickerError>>>,
    started: mpsc::Sender<(WorkspaceOperation, PathBuf)>,
    proceed: Mutex<mpsc::Receiver<WorkspaceError>>,
}

impl Commands {
    fn apply(
        &self,
        operation: WorkspaceOperation,
        path: &Path,
    ) -> Result<WorkspaceMetadata, WorkspaceError> {
        self.started.send((operation, path.to_path_buf())).unwrap();
        Err(self
            .proceed
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(3))
            .unwrap())
    }
}

impl WorkspaceCommands for Commands {
    fn choose_workspace_folder(&self) -> Result<Option<PathBuf>, FolderPickerError> {
        self.picker
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(3))
            .unwrap()
    }

    fn create_workspace(&self, path: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
        self.apply(WorkspaceOperation::Create, path)
    }

    fn open_workspace(&self, path: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
        self.apply(WorkspaceOperation::Open, path)
    }
}

struct Harness {
    form: WorkspaceForm,
    choose: mpsc::Sender<Result<Option<PathBuf>, FolderPickerError>>,
    calls: mpsc::Receiver<(WorkspaceOperation, PathBuf)>,
    storage: mpsc::Sender<WorkspaceError>,
}

impl Harness {
    fn new() -> Self {
        let (choose, choice) = mpsc::channel();
        let (started, calls) = mpsc::channel();
        let (storage, proceed) = mpsc::channel();
        let form = WorkspaceForm::new(Some(Arc::new(Commands {
            picker: Mutex::new(choice),
            started,
            proceed: Mutex::new(proceed),
        })));
        Self {
            form,
            choose,
            calls,
            storage,
        }
    }

    fn key(&mut self, code: KeyCode) {
        assert!(self.form.handle_event(&key(code)));
    }

    fn response(&mut self) {
        let until = Instant::now() + Duration::from_secs(3);
        while !self.form.poll_result() {
            assert!(
                Instant::now() < until,
                "worker did not deliver the expected response"
            );
            thread::yield_now();
        }
    }
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

#[test]
fn tab_and_enter_select_unicode_root_then_confirm_without_appending_a_folder() {
    let mut h = Harness::new();
    let mut app = AppState::default();
    app.set_workspace_controls(true);
    assert_eq!(
        dispatch_event(&mut app, &key(KeyCode::Tab), None, Some(&mut h.form)),
        UiAction::None
    );
    h.form.set_panel_focus(true, false);
    h.key(KeyCode::Enter);
    assert_eq!(
        h.form.status,
        FormStatus::Editing(WorkspaceOperation::Create)
    );

    // Typing is consumed by the form and does not become the workspace path.
    h.key(KeyCode::Char('z'));
    h.key(KeyCode::Enter);
    let root = PathBuf::from("C:/Manga/異世界 root 01");
    h.choose.send(Ok(Some(root.clone()))).unwrap();
    h.response();
    assert_eq!(h.form.path, Some(root.clone()));
    assert!(h.form.snapshot().body().contains(&*root.to_string_lossy()));
    assert!(h.calls.try_recv().is_err());
    h.key(KeyCode::Enter);
    assert_eq!(
        h.calls.recv_timeout(Duration::from_secs(1)).unwrap(),
        (WorkspaceOperation::Create, root)
    );
    h.storage.send(WorkspaceError::DirectoryNotEmpty).unwrap();
    h.response();
    assert!(
        h.form
            .snapshot()
            .body()
            .contains("Thư mục đích không trống")
    );
}

#[test]
fn cancel_and_chooser_error_keep_previous_selection_and_never_open_it() {
    let mut h = Harness::new();
    h.form.set_panel_focus(true, false);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert_eq!(h.form.status, FormStatus::Editing(WorkspaceOperation::Open));
    h.key(KeyCode::Enter);
    let root = PathBuf::from("C:/Workspace/巻 02");
    h.choose.send(Ok(Some(root.clone()))).unwrap();
    h.response();
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    h.choose.send(Ok(None)).unwrap();
    h.response();
    assert_eq!(h.form.path, Some(root.clone()));
    assert_eq!(h.form.control, FormControl::Choose);

    h.key(KeyCode::Enter);
    h.choose
        .send(Err(FolderPickerError::PermissionDenied))
        .unwrap();
    h.response();
    assert_eq!(h.form.path, Some(root));
    assert!(
        h.form
            .snapshot()
            .body()
            .contains("Thiếu quyền chọn thư mục")
    );
    assert!(h.calls.try_recv().is_err());
}

#[test]
fn leaving_or_replacing_a_delayed_form_discards_its_result() {
    let mut h = Harness::new();
    h.key(KeyCode::Char('o'));
    h.form.status = FormStatus::Choosing {
        operation: WorkspaceOperation::Open,
        request_id: 18,
    };
    h.form.route_left();
    assert!(!h.form.apply_reply(Reply::Folder {
        request_id: 18,
        result: Ok(Some(PathBuf::from("C:/Old"))),
    }));
    assert_eq!(h.form.status, FormStatus::Idle);
    h.key(KeyCode::Char('c'));
    h.form.status = FormStatus::Choosing {
        operation: WorkspaceOperation::Create,
        request_id: 19,
    };
    h.key(KeyCode::Char('o'));
    assert!(!h.form.apply_reply(Reply::Folder {
        request_id: 19,
        result: Ok(Some(PathBuf::from("C:/Replaced"))),
    }));
    assert_eq!(h.form.status, FormStatus::Editing(WorkspaceOperation::Open));
    assert!(h.form.path.is_none());
    assert!(h.calls.try_recv().is_err());
}

#[test]
fn workspace_io_keeps_the_loop_responsive_and_reports_only_the_exact_request() {
    let mut h = Harness::new();
    h.key(KeyCode::Char('o'));
    h.key(KeyCode::Enter);
    let root = PathBuf::from("C:/Workspace/古い 01");
    h.choose.send(Ok(Some(root.clone()))).unwrap();
    h.response();
    h.key(KeyCode::Enter);
    let FormStatus::Pending { request_id, .. } = h.form.status else {
        panic!("expected worker");
    };
    assert_eq!(
        h.calls.recv_timeout(Duration::from_secs(1)).unwrap(),
        (WorkspaceOperation::Open, root.clone())
    );
    let begin = Instant::now();
    h.key(KeyCode::Right);
    assert!(begin.elapsed() < Duration::from_millis(100));
    assert!(!h.form.apply_reply(Reply::Workspace {
        request_id: request_id - 1,
        path: PathBuf::from("stale folder"),
        result: Err(WorkspaceError::NotFound)
    }));
    assert!(h.form.snapshot().help_summary().contains("Chờ kết quả"));
    h.storage.send(WorkspaceError::CorruptWorkspace).unwrap();
    h.response();
    assert_eq!(h.form.path, Some(root));
    assert!(
        h.form
            .snapshot()
            .body()
            .contains("Khôi phục từ bản sao lưu")
    );
    assert!(h.form.snapshot().footer_text().contains("Esc đóng lỗi"));
}

#[test]
fn mouse_clicks_only_the_visible_choose_and_confirm_controls() {
    let mut h = Harness::new();
    assert!(h.form.click(1, 4));
    assert!(!h.form.click(1, 4));
    assert!(h.form.click(0, 4));
    assert!(!h.form.click(1, 4));
    let root = PathBuf::from("C:/Workspace/本 03");
    h.choose.send(Ok(Some(root.clone()))).unwrap();
    h.response();
    assert!(!h.form.click(1, 40));
    assert!(h.form.click(1, 4));
    assert_eq!(
        h.calls.recv_timeout(Duration::from_secs(1)).unwrap(),
        (WorkspaceOperation::Open, root)
    );
    h.storage
        .send(WorkspaceError::NewerSchema {
            found: 10,
            supported: 4,
        })
        .unwrap();
    h.response();
    assert!(
        h.form
            .snapshot()
            .body()
            .contains("phiên bản ứng dụng mới hơn")
    );
}

#[test]
fn failed_workspace_errors_show_recovery_and_visible_dismissal_actions() {
    let mut form = Harness::new().form;
    form.status = FormStatus::Failed {
        operation: WorkspaceOperation::Open,
        error: WorkspaceError::CorruptWorkspace,
    };
    form.path = Some(PathBuf::from("series_日本語 01"));
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
    let mut form = Harness::new().form;
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

    let mut form = Harness::new().form;
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
fn stale_workspace_reply_does_not_consume_the_current_confirmation() {
    let mut form = Harness::new().form;
    let path = PathBuf::from("workspace hiện tại");
    form.path = Some(path.clone());
    form.status = FormStatus::Pending {
        operation: WorkspaceOperation::Open,
        request_id: 9,
    };
    form.result_sender
        .send(Reply::Workspace {
            request_id: 8,
            path: PathBuf::from("lượt trước"),
            result: Err(WorkspaceError::NotFound),
        })
        .unwrap();
    assert!(!form.poll_result());
    assert_eq!(
        form.status,
        FormStatus::Pending {
            operation: WorkspaceOperation::Open,
            request_id: 9,
        }
    );

    form.result_sender
        .send(Reply::Workspace {
            request_id: 9,
            path: path.clone(),
            result: Err(WorkspaceError::CorruptWorkspace),
        })
        .unwrap();
    assert!(form.poll_result());
    assert_eq!(form.path.as_deref(), Some(path.as_path()));
    assert!(
        form.snapshot()
            .body()
            .contains("Nơi đã chọn: workspace hiện tại")
    );
    assert!(
        form.snapshot()
            .body()
            .contains("Khôi phục từ bản sao lưu đã kiểm chứng")
    );
}

#[test]
fn overview_keeps_import_as_the_first_control_when_a_workspace_is_open() {
    let mut h = Harness::new();
    h.form.set_panel_focus(true, true);
    assert!(h.form.import_focused());
    assert!(!h.form.handle_event(&key(KeyCode::Enter)));

    h.key(KeyCode::Tab);
    assert!(!h.form.import_focused());
    assert!(
        h.form
            .snapshot()
            .entry_buttons()
            .contains("› [Tạo workspace]")
    );
    h.key(KeyCode::Tab);
    assert!(
        h.form
            .snapshot()
            .entry_buttons()
            .contains("› [Mở workspace]")
    );
    h.key(KeyCode::BackTab);
    h.key(KeyCode::BackTab);
    assert!(h.form.import_focused());
}
