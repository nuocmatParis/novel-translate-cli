use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use nt_application::volume_import::{VolumeId, source_scope::ScopeMode};

use crate::import_commands::{
    BookCatalog, BookChange, BookError, BookPosition, BookSummary, BookView, BookWorkspace,
    VolumeCommands,
};
use crate::input::EventLayer;
use crate::navigation::Screen;

#[path = "import_draft.rs"]
mod draft;
use draft::{Draft, DraftAction};

#[cfg(test)]
#[path = "import_screen_tests.rs"]
mod tests;

pub(crate) const PICKER_LABEL: &str = "＋ Nhập tập truyện";

pub(crate) struct ImportScreen {
    commands: Option<Arc<dyn VolumeCommands>>,
    workspace: Option<(u64, BookWorkspace)>,
    catalog: Vec<BookSummary>,
    selected: Option<VolumeId>,
    view: Option<BookView>,
    position: BookPosition,
    previous_positions: Vec<(VolumeId, BookPosition)>,
    picker_focused: bool,
    screen: Screen,
    notice: String,
    read_error: Option<BookError>,
    drafts: Vec<Draft>,
    next_draft_id: u64,
    editing: bool,
    queued_read: Option<ReadRequest>,
    read_job: Option<Job<ReadResult>>,
    change_job: Option<Job<ChangeResult>>,
    writer_active: bool,
}

enum ReadRequest {
    Catalog,
    View(VolumeId, BookPosition),
}

enum ReadResult {
    Catalog(Result<BookCatalog, BookError>),
    View {
        volume: VolumeId,
        position: BookPosition,
        result: Result<Box<BookView>, BookError>,
    },
}

enum ChangeResult {
    Chosen(Result<Option<PathBuf>, BookError>),
    Imported(Result<BookChange, BookError>),
    Preview(u64, Result<String, BookError>),
    Saved(u64, VolumeId, Result<BookChange, BookError>),
}

struct Job<T> {
    epoch: u64,
    workspace: BookWorkspace,
    receiver: mpsc::Receiver<T>,
    thread: JoinHandle<()>,
}

impl<T: Send + 'static> Job<T> {
    fn start(
        epoch: u64,
        workspace: BookWorkspace,
        name: &str,
        run: impl FnOnce(&BookWorkspace) -> T + Send + 'static,
    ) -> Result<Self, BookError> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let origin = workspace.clone();
        let thread = thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || {
                let _ = sender.send(run(&origin));
            })
            .map_err(|_| BookError::NotSaved)?;
        Ok(Self {
            epoch,
            workspace,
            receiver,
            thread,
        })
    }

    fn result(&self) -> Option<Result<T, BookError>> {
        match self.receiver.try_recv() {
            Ok(value) => Some(Ok(value)),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(BookError::NotSaved)),
        }
    }
}

impl ImportScreen {
    pub(crate) fn new(commands: Option<Arc<dyn VolumeCommands>>) -> Self {
        Self {
            commands,
            workspace: None,
            catalog: Vec::new(),
            selected: None,
            view: None,
            position: BookPosition::default(),
            previous_positions: Vec::new(),
            picker_focused: true,
            screen: Screen::Overview,
            notice: String::new(),
            read_error: None,
            drafts: Vec::new(),
            next_draft_id: 1,
            editing: false,
            queued_read: None,
            read_job: None,
            change_job: None,
            writer_active: false,
        }
    }

    pub(crate) fn available(&self, screen: Screen) -> bool {
        self.workspace.is_some() && matches!(screen, Screen::Overview | Screen::VolumesAndChapters)
    }

    pub(crate) fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }

    pub(crate) fn editing(&self) -> bool {
        self.editing
    }

    pub(crate) fn focus_picker(&mut self) {
        self.picker_focused = true;
        self.editing = false;
    }

    pub(crate) fn open_workspace(&mut self, epoch: u64, workspace: BookWorkspace) {
        if self.commands.is_none()
            || self
                .workspace
                .as_ref()
                .is_some_and(|(active, _)| *active == epoch)
        {
            return;
        }
        self.workspace = Some((epoch, workspace));
        self.catalog.clear();
        self.read_error = None;
        self.selected = None;
        self.view = None;
        self.position = BookPosition::default();
        self.previous_positions.clear();
        self.focus_picker();
        self.notice = "Đang đọc danh sách tập…".into();
        self.queued_read = Some(ReadRequest::Catalog);
    }

    /// Returns whether to redraw and whether a new import needs keyboard focus.
    pub(crate) fn poll(&mut self) -> (bool, bool) {
        let mut redraw = self.poll_read();
        let mut imported = false;
        if let Some(result) = self.change_job.as_ref().and_then(Job::result) {
            let job = self.change_job.take().expect("a result has a job");
            self.writer_active = false;
            if self
                .workspace
                .as_ref()
                .is_some_and(|(epoch, _)| *epoch == job.epoch)
            {
                redraw = true;
                match result {
                    Ok(ChangeResult::Chosen(Ok(Some(path)))) => {
                        if self
                            .workspace
                            .as_ref()
                            .is_some_and(|(_, active)| active == &job.workspace)
                        {
                            self.start_import(path);
                        } else {
                            self.notice = BookError::Stale.message();
                        }
                    }
                    Ok(ChangeResult::Chosen(Ok(None))) => self.notice = "Đã hủy chọn EPUB.".into(),
                    Ok(ChangeResult::Imported(Ok(change))) => {
                        self.select_volume(change.volume.clone());
                        self.picker_focused = false;
                        self.editing = false;
                        self.finish_write(change);
                        imported = true;
                    }
                    Ok(ChangeResult::Saved(draft_id, volume, Ok(change))) => {
                        self.drafts.retain(|draft| draft.id != draft_id);
                        if self.selected.as_ref() == Some(&volume) {
                            self.editing = false;
                        }
                        self.finish_write(change);
                    }
                    Ok(ChangeResult::Preview(draft_id, preview)) => {
                        if let Some(draft) =
                            self.drafts.iter_mut().find(|draft| draft.id == draft_id)
                        {
                            draft.set_preview(preview);
                        }
                        self.notice.clear();
                    }
                    Ok(
                        ChangeResult::Chosen(Err(error))
                        | ChangeResult::Imported(Err(error))
                        | ChangeResult::Saved(_, _, Err(error)),
                    )
                    | Err(error) => {
                        self.notice = error.message();
                    }
                }
            }
        }
        self.start_queued_read();
        (redraw, imported)
    }

    fn poll_read(&mut self) -> bool {
        let Some(result) = self.read_job.as_ref().and_then(Job::result) else {
            return false;
        };
        let job = self.read_job.take().expect("a result has a job");
        if self
            .workspace
            .as_ref()
            .is_none_or(|(epoch, active)| *epoch != job.epoch || active != &job.workspace)
        {
            return false;
        }
        match result {
            Ok(ReadResult::Catalog(Ok(catalog))) => {
                if let Some((_, workspace)) = &mut self.workspace {
                    workspace.generation = catalog.generation;
                }
                self.catalog = catalog.volumes;
                if !self
                    .catalog
                    .iter()
                    .any(|book| Some(&book.id) == self.selected.as_ref())
                {
                    self.selected = self.catalog.first().map(|book| book.id.clone());
                    self.position = BookPosition::default();
                }
                self.queue_view();
                if self.notice == "Đang đọc danh sách tập…" {
                    self.notice.clear();
                }
            }
            Ok(ReadResult::View {
                volume,
                position,
                result,
            }) => {
                if self.selected.as_ref() == Some(&volume) && self.position == position {
                    match result {
                        Ok(view) => {
                            self.position = view.position;
                            self.view = Some(*view);
                        }
                        Err(error) => self.read_error = Some(error),
                    }
                }
            }
            Ok(ReadResult::Catalog(Err(error))) | Err(error) => {
                if self.notice == "Đang đọc danh sách tập…" {
                    self.notice.clear();
                }
                self.read_error = Some(error);
            }
        }
        true
    }

    fn queue_view(&mut self) {
        self.view = None;
        if let Some(volume) = &self.selected {
            self.queued_read = Some(ReadRequest::View(volume.clone(), self.position));
        }
    }

    fn start_queued_read(&mut self) {
        if self.read_job.is_some() {
            return;
        }
        let (Some(request), Some((epoch, workspace)), Some(commands)) = (
            self.queued_read.take(),
            self.workspace.clone(),
            self.commands.as_ref().map(Arc::clone),
        ) else {
            return;
        };
        self.read_error = None;
        match Job::start(
            epoch,
            workspace,
            "nt-volume-view",
            move |workspace| match request {
                ReadRequest::Catalog => ReadResult::Catalog(commands.catalog(workspace)),
                ReadRequest::View(volume, position) => {
                    let result = commands.view(workspace, &volume, position).map(Box::new);
                    ReadResult::View {
                        volume,
                        position,
                        result,
                    }
                }
            },
        ) {
            Ok(job) => self.read_job = Some(job),
            Err(error) => self.read_error = Some(error),
        }
    }

    fn start_change(
        &mut self,
        writer: bool,
        message: &str,
        operation: impl FnOnce(&dyn VolumeCommands, &BookWorkspace) -> ChangeResult + Send + 'static,
    ) {
        if self.change_job.is_some() {
            return;
        }
        let (Some((epoch, workspace)), Some(commands)) = (
            self.workspace.clone(),
            self.commands.as_ref().map(Arc::clone),
        ) else {
            return;
        };
        match Job::start(epoch, workspace, "nt-volume-command", move |workspace| {
            operation(commands.as_ref(), workspace)
        }) {
            Ok(job) => {
                self.writer_active = writer;
                self.change_job = Some(job);
                self.notice = message.to_owned();
            }
            Err(error) => self.notice = error.message(),
        }
    }

    pub(crate) fn open_picker(&mut self) {
        self.focus_picker();
        self.start_change(false, "Đang mở hộp chọn EPUB…", |commands, _| {
            ChangeResult::Chosen(commands.choose_epub())
        });
    }

    fn start_import(&mut self, epub: PathBuf) {
        self.start_change(true, "Đang nhập EPUB…", move |commands, workspace| {
            ChangeResult::Imported(commands.import_epub(workspace, &epub))
        });
    }

    fn finish_write(&mut self, change: BookChange) {
        if let Some((_, workspace)) = &mut self.workspace {
            workspace.generation = change.generation;
        }
        self.view = None;
        self.notice = "Đã lưu tập.".into();
        self.queued_read = Some(ReadRequest::Catalog);
    }

    fn shift_volume(&mut self, forward: bool) {
        let Some(index) = self
            .catalog
            .iter()
            .position(|item| Some(&item.id) == self.selected.as_ref())
        else {
            return;
        };
        if let Some(index) = shift_index(index, self.catalog.len(), forward) {
            self.select_volume(self.catalog[index].id.clone());
            self.editing = false;
            self.picker_focused = false;
            self.queue_view();
        }
    }

    fn select_volume(&mut self, volume: VolumeId) {
        if Some(&volume) == self.selected.as_ref() {
            return;
        }
        if let Some(previous) = self.selected.replace(volume.clone()) {
            if let Some((_, position)) = self
                .previous_positions
                .iter_mut()
                .find(|(id, _)| id == &previous)
            {
                *position = self.position;
            } else {
                self.previous_positions.push((previous, self.position));
            }
        }
        self.position = self
            .previous_positions
            .iter()
            .find(|(id, _)| id == &volume)
            .map_or_else(BookPosition::default, |(_, position)| *position);
    }

    fn shift_position(&mut self, selection: KeyCode) {
        let Some(view) = &self.view else { return };
        let before = self.position;
        match selection {
            KeyCode::Down | KeyCode::Up => {
                if let Some(row) =
                    shift_index(self.position.row, view.rows, selection == KeyCode::Down)
                {
                    self.position = BookPosition {
                        row,
                        ..BookPosition::default()
                    };
                }
            }
            KeyCode::PageDown | KeyCode::PageUp => {
                if let Some(range) = shift_index(
                    self.position.range,
                    view.ranges,
                    selection == KeyCode::PageDown,
                ) {
                    self.position = BookPosition {
                        row: self.position.row,
                        range,
                        unit: 0,
                    };
                }
            }
            KeyCode::Char(']') => {
                if let Some(source) = &view.source {
                    self.position.unit = self.position.unit.saturating_add(96).min(source.end - 1);
                }
            }
            KeyCode::Char('[') => self.position.unit = self.position.unit.saturating_sub(96),
            _ => {}
        }
        if before != self.position {
            self.queue_view();
        }
    }

    fn active_draft(&self) -> Option<usize> {
        let (_, workspace) = self.workspace.as_ref()?;
        let id = self.selected.as_ref()?;
        self.drafts
            .iter()
            .position(|draft| draft.is_for(workspace, id))
    }

    fn start_draft(&mut self, key: char) {
        if self.change_job.is_some() && self.writer_active {
            return;
        }
        let (Some(view), Some((_, workspace)), Some(volume)) = (
            self.view.as_ref(),
            self.workspace.as_ref(),
            self.selected.as_ref(),
        ) else {
            return;
        };
        if let Some(index) = self.active_draft() {
            self.editing = true;
            self.notice = if &self.drafts[index].workspace == workspace {
                "Đang mở bản nháp. F2: Bỏ bản nháp.".into()
            } else {
                BookError::Stale.message()
            };
            return;
        }
        let Some(draft) = Draft::new(
            self.next_draft_id,
            workspace.clone(),
            volume.clone(),
            view,
            key,
        ) else {
            self.notice = BookError::InvalidEdit.message();
            return;
        };
        self.notice.clear();
        self.next_draft_id = self.next_draft_id.wrapping_add(1);
        self.drafts.push(draft);
        self.editing = true;
    }

    fn edit_event(&mut self, key: KeyEvent) -> bool {
        let Some(index) = self.active_draft() else {
            return false;
        };
        match key.code {
            KeyCode::Esc => {
                self.editing = false;
                self.notice.clear();
                true
            }
            KeyCode::F(2) if !self.writer_active => {
                self.drafts.remove(index);
                self.editing = false;
                self.notice = "Đã bỏ bản nháp.".into();
                true
            }
            _ => {
                let draft = &mut self.drafts[index];
                if self.change_job.is_some() {
                    return true;
                }
                if self
                    .workspace
                    .as_ref()
                    .is_some_and(|(_, workspace)| &draft.workspace != workspace)
                {
                    self.notice = BookError::Stale.message();
                    return true;
                }
                let volume = draft.volume.clone();
                let draft_id = draft.id;
                match draft.handle(key) {
                    DraftAction::Preview(edit) => self.start_change(
                        false,
                        "Đang dựng bản xem trước…",
                        move |commands, origin| {
                            ChangeResult::Preview(
                                draft_id,
                                commands.preview(origin, &volume, &edit),
                            )
                        },
                    ),
                    DraftAction::Save(edit) => {
                        self.start_change(true, "Đang lưu…", move |commands, origin| {
                            let result = commands.save(origin, &volume, &edit);
                            ChangeResult::Saved(draft_id, volume, result)
                        })
                    }
                    DraftAction::None => {}
                }
                true
            }
        }
    }

    pub(crate) fn body(&self, focused: bool) -> String {
        self.body_with_workspace_actions(focused, "")
    }

    pub(crate) fn body_with_workspace_actions(&self, focused: bool, actions: &str) -> String {
        let marker = if focused && self.picker_focused {
            "› "
        } else {
            "  "
        };
        let mut body = format!("{marker}[{PICKER_LABEL}]");
        if !actions.is_empty() {
            body.push('\n');
            body.push_str(actions);
        }
        if !self.notice.is_empty() {
            body.push_str(&format!("\n{}", self.notice));
        }
        if let Some(error) = &self.read_error {
            body.push_str(&format!("\n{} R: Đọc lại.", error.message()));
        }
        if self.screen == Screen::Overview {
            if self.catalog.is_empty() {
                body.push_str("\nWorkspace chưa có tập.");
            } else {
                body.push_str(&format!(
                    "\n{} tập · Mở Tập và chương để xem.",
                    self.catalog.len()
                ));
            }
            body.push_str("\nC: Tạo workspace · O: Mở workspace");
            return body;
        }
        let Some(index) = self
            .catalog
            .iter()
            .position(|book| Some(&book.id) == self.selected.as_ref())
        else {
            body.push_str("\nWorkspace chưa có tập.");
            return body;
        };
        let selected_marker = if focused && !self.picker_focused {
            "›"
        } else {
            " "
        };
        body.push_str(&format!(
            "\n{selected_marker} Tập {}/{} · {}",
            index + 1,
            self.catalog.len(),
            self.catalog[index].title
        ));
        if self.editing
            && let Some(index) = self.active_draft()
        {
            body.push_str(&format!("\n{}", self.drafts[index].body()));
            return body;
        }
        let Some(view) = &self.view else {
            body.push_str("\nĐang đọc cây tập…");
            return body;
        };
        body.push_str(&format!(
            "\n{} · {} cần kiểm",
            scope_label(view.mode),
            view.review_count
        ));
        if view.rows == 0 {
            body.push_str(&format!("\n{}", view.caption));
        } else {
            body.push_str(&format!(
                "\n  {}/{} · {}",
                view.position.row + 1,
                view.rows,
                view.caption
            ));
        }
        if let Some(source) = &view.source {
            body.push_str(&format!(
                "\n    Vùng {}/{} · mục {}–{}\n{}",
                view.position.range + 1,
                view.ranges,
                view.position.unit,
                source.end,
                source.text
            ));
            if source.classification.is_some() {
                body.push_str("\nCần phân loại ở vùng này. K: Chọn loại.");
            }
        }
        body.push_str("\n←/→ Tập · ↑/↓ Chương · PgUp/PgDn Vùng · [ ] Đọc tiếp");
        body.push_str("\nP Phạm vi · B Mặc định bộ · N Đổi tên · M Gộp · S Tách · D Dời ranh giới");
        body
    }
}

impl EventLayer for ImportScreen {
    fn handle_event(&mut self, event: &Event) -> bool {
        let Event::Key(key) = event else { return false };
        if key.kind != KeyEventKind::Press
            || !(key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT)
            || matches!(key.code, KeyCode::Tab | KeyCode::BackTab)
        {
            return false;
        }
        if self.screen == Screen::VolumesAndChapters && self.editing {
            return self.edit_event(*key);
        }
        if key.code == KeyCode::Enter && self.picker_focused
            || matches!(key.code, KeyCode::Char('i' | 'I'))
        {
            self.open_picker();
            return true;
        }
        if key.code == KeyCode::Esc {
            self.notice.clear();
            self.picker_focused = true;
            return true;
        }
        if matches!(key.code, KeyCode::Char('r' | 'R')) {
            self.queued_read = Some(ReadRequest::Catalog);
            return true;
        }
        if self.screen != Screen::VolumesAndChapters {
            return false;
        }
        match key.code {
            KeyCode::Home => self.focus_picker(),
            KeyCode::Left | KeyCode::Right => self.shift_volume(key.code == KeyCode::Right),
            KeyCode::Down if self.picker_focused => self.picker_focused = false,
            KeyCode::Up if !self.picker_focused && self.position.row == 0 => {
                self.picker_focused = true
            }
            KeyCode::Down
            | KeyCode::Up
            | KeyCode::PageDown
            | KeyCode::PageUp
            | KeyCode::Char('[' | ']') => {
                self.shift_position(key.code);
            }
            KeyCode::Char('e' | 'E') => self.editing = self.active_draft().is_some(),
            KeyCode::Char(action) if "pbnmsdk".contains(action.to_ascii_lowercase()) => {
                self.start_draft(action.to_ascii_lowercase())
            }
            _ => return false,
        }
        true
    }
}

impl Drop for ImportScreen {
    fn drop(&mut self) {
        // A dialog and read jobs never write. An accepted write drains its
        // transaction before main exits, even if the terminal has closed.
        if self.writer_active
            && let Some(job) = self.change_job.take()
        {
            let _ = job.thread.join();
        }
    }
}

fn shift_index(index: usize, length: usize, forward: bool) -> Option<usize> {
    if forward {
        index.checked_add(1).filter(|candidate| *candidate < length)
    } else {
        index.checked_sub(1)
    }
}

fn scope_label(mode: ScopeMode) -> &'static str {
    match mode {
        ScopeMode::Story => "Chỉ truyện",
        ScopeMode::AllText => "Toàn bộ phần chữ",
    }
}
