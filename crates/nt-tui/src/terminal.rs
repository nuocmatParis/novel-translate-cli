use crate::WorkspaceCommands;
use crate::input::{UiAction, dispatch_event, is_shutdown_request};
use crate::screens::{WorkspaceForm, WorkspaceFormSnapshot};
use crate::state::AppState;
#[cfg(windows)]
use crate::windows_input::WindowsInputReader;
use crate::{TuiError, TuiErrorKind};
#[cfg(not(windows))]
use crossterm::event;
use crossterm::event::Event;
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::Print,
};
use ratatui::buffer::{Buffer, CellWidth};
use ratatui::layout::Rect;
use std::io::{self, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::time::Duration;

const INPUT_POLL_INTERVAL: Duration = Duration::from_millis(100);

trait TerminalDriver {
    fn terminal_area(&self) -> io::Result<Rect>;
    fn enable_raw_mode(&mut self) -> io::Result<()>;
    fn enter_alternate_screen(&mut self) -> io::Result<()>;
    fn hide_cursor(&mut self) -> io::Result<()>;
    fn show_cursor(&mut self) -> io::Result<()>;
    fn leave_alternate_screen(&mut self) -> io::Result<()>;
    fn disable_raw_mode(&mut self) -> io::Result<()>;
    fn draw_shell(&mut self, area: Rect, state: &AppState) -> io::Result<()>;
    fn draw_shell_with_workspace(
        &mut self,
        area: Rect,
        state: &AppState,
        _workspace: &WorkspaceFormSnapshot,
    ) -> io::Result<()> {
        self.draw_shell(area, state)
    }
    fn read_event(&mut self) -> io::Result<Event>;
    fn poll_event(&mut self, _timeout: Duration) -> io::Result<Option<Event>> {
        self.read_event().map(Some)
    }
}

/// Runs the small lifecycle shell until the user requests shutdown.
/// The main shell presentation is supplied by the following TUI tasks.
pub fn run_shell() -> Result<(), TuiError> {
    let driver = CrosstermTerminal::new().map_err(|_| TuiError::new(TuiErrorKind::EventRead))?;
    run_shell_with_driver(driver)
}

pub fn run_shell_with_workspace(commands: Arc<dyn WorkspaceCommands>) -> Result<(), TuiError> {
    let driver = CrosstermTerminal::new().map_err(|_| TuiError::new(TuiErrorKind::EventRead))?;
    run_shell_with_driver_and_workspace(driver, Some(commands))
}

fn run_shell_with_driver<D: TerminalDriver>(driver: D) -> Result<(), TuiError> {
    run_shell_with_driver_and_workspace(driver, None)
}

fn run_shell_with_driver_and_workspace<D: TerminalDriver>(
    driver: D,
    commands: Option<Arc<dyn WorkspaceCommands>>,
) -> Result<(), TuiError> {
    run_with_driver(driver, |driver| {
        let mut state = AppState::default();
        let mut workspace_form = WorkspaceForm::new(commands);
        let mut rendered_area = None;
        let mut redraw_needed = true;
        loop {
            let workspace_changed = workspace_form.poll_result();
            let area = driver.terminal_area().map_err(|_| TuiErrorKind::Render)?;
            if redraw_needed || rendered_area != Some(area) || workspace_changed {
                refresh_scroll_layout(&mut state, area);
                driver
                    .draw_shell_with_workspace(area, &state, &workspace_form.snapshot())
                    .map_err(|_| TuiErrorKind::Render)?;
                rendered_area = Some(area);
                redraw_needed = false;
            }
            let controls_hidden_in_last_frame = !state.terminal_size_supported();
            let event = driver
                .poll_event(INPUT_POLL_INTERVAL)
                .map_err(|_| TuiErrorKind::EventRead)?;
            let Some(event) = event else {
                let area = driver.terminal_area().map_err(|_| TuiErrorKind::Render)?;
                if rendered_area != Some(area) {
                    redraw_needed = true;
                }
                continue;
            };
            if is_shutdown_request(&event) {
                return Ok(());
            }
            let area = driver.terminal_area().map_err(|_| TuiErrorKind::Render)?;
            refresh_scroll_layout(&mut state, area);
            if controls_hidden_in_last_frame && state.terminal_size_supported() {
                redraw_needed = true;
                continue;
            }
            let screen_before_dispatch = state.current_screen();
            let scroll_offset_before_dispatch = state.scroll_offset();
            let focused_control = (state.current_screen() == crate::navigation::Screen::Overview)
                .then_some(&mut workspace_form as &mut dyn crate::input::EventLayer);
            match dispatch_event(&mut state, &event, None, focused_control) {
                UiAction::Shutdown => return Ok(()),
                UiAction::None | UiAction::RouteOpened(_) => {}
            }
            refresh_scroll_layout(&mut state, area);
            if state.current_screen() != screen_before_dispatch {
                let offset = crate::rendering::scroll_offset_for_anchor(area, &state);
                state.set_scroll_offset(offset);
            } else if state.scroll_offset() != scroll_offset_before_dispatch {
                crate::rendering::capture_scroll_anchor(&mut state, area);
            }
            redraw_needed = true;
        }
    })
}

fn refresh_scroll_layout(state: &mut AppState, area: Rect) {
    let (previous_width, previous_height) = state.terminal_size();
    let resized = (previous_width, previous_height) != (area.width, area.height);
    state.set_terminal_size(area.width, area.height);
    let (page_lines, scroll_limit) = crate::rendering::scroll_metrics(area, state);
    state.set_scroll_metrics(page_lines, scroll_limit);
    if resized && state.terminal_size_supported() {
        let offset = crate::rendering::scroll_offset_for_anchor(area, state);
        state.set_scroll_offset(offset);
    }
}

fn run_with_driver<D, F>(driver: D, operation: F) -> Result<(), TuiError>
where
    D: TerminalDriver,
    F: FnOnce(&mut D) -> Result<(), TuiErrorKind>,
{
    let mut session = TerminalSession::new(driver);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        session.setup()?;
        operation(&mut session.driver)
    }));
    let cleanup_error = session.cleanup();

    match outcome {
        Ok(Ok(())) => match cleanup_error {
            Some(kind) => Err(TuiError::new(kind)),
            None => Ok(()),
        },
        Ok(Err(primary)) => Err(error_with_cleanup(primary, cleanup_error)),
        Err(payload) => {
            // The app's panic hook records only a safe marker; never format the payload.
            discard_panic_payload(payload);
            Err(error_with_cleanup(TuiErrorKind::Panic, cleanup_error))
        }
    }
}

fn discard_panic_payload(payload: Box<dyn std::any::Any + Send>) {
    // A payload destructor is arbitrary code too. Contain a second panic while
    // the app's safe hook is still installed; its nested payload is intentionally
    // forgotten because running another destructor could unwind again.
    if let Err(nested_payload) = catch_unwind(AssertUnwindSafe(|| drop(payload))) {
        std::mem::forget(nested_payload);
    }
}

fn error_with_cleanup(primary: TuiErrorKind, cleanup: Option<TuiErrorKind>) -> TuiError {
    match cleanup {
        Some(cleanup) => TuiError::with_cleanup(primary, cleanup),
        None => TuiError::new(primary),
    }
}

struct TerminalSession<D> {
    driver: D,
    raw_mode_may_be_enabled: bool,
    alternate_screen_may_be_active: bool,
    cursor_may_be_hidden: bool,
    cleanup_result: Option<Option<TuiErrorKind>>,
}

impl<D: TerminalDriver> TerminalSession<D> {
    fn new(driver: D) -> Self {
        Self {
            driver,
            raw_mode_may_be_enabled: false,
            alternate_screen_may_be_active: false,
            cursor_may_be_hidden: false,
            cleanup_result: None,
        }
    }

    fn setup(&mut self) -> Result<(), TuiErrorKind> {
        // Record a resource before issuing its command: a failed command may
        // still have performed part of its terminal side effect.
        self.raw_mode_may_be_enabled = true;
        self.driver
            .enable_raw_mode()
            .map_err(|_| TuiErrorKind::TerminalSetup)?;

        self.alternate_screen_may_be_active = true;
        self.driver
            .enter_alternate_screen()
            .map_err(|_| TuiErrorKind::TerminalSetup)?;

        self.cursor_may_be_hidden = true;
        self.driver
            .hide_cursor()
            .map_err(|_| TuiErrorKind::TerminalSetup)?;
        Ok(())
    }

    fn cleanup(&mut self) -> Option<TuiErrorKind> {
        if let Some(result) = self.cleanup_result {
            return result;
        }
        let mut failed = false;

        if self.cursor_may_be_hidden {
            self.cursor_may_be_hidden = false;
            failed |= attempt_cleanup(|| self.driver.show_cursor());
        }
        if self.alternate_screen_may_be_active {
            self.alternate_screen_may_be_active = false;
            failed |= attempt_cleanup(|| self.driver.leave_alternate_screen());
        }
        if self.raw_mode_may_be_enabled {
            self.raw_mode_may_be_enabled = false;
            failed |= attempt_cleanup(|| self.driver.disable_raw_mode());
        }

        let result = failed.then_some(TuiErrorKind::TerminalCleanup);
        self.cleanup_result = Some(result);
        result
    }
}

fn attempt_cleanup(action: impl FnOnce() -> io::Result<()>) -> bool {
    // Keep the entire Result, including destruction of any custom io::Error
    // payload, inside the unwind boundary.
    match catch_unwind(AssertUnwindSafe(|| action().is_err())) {
        Ok(failed) => failed,
        Err(payload) => {
            discard_panic_payload(payload);
            true
        }
    }
}

struct CrosstermTerminal {
    #[cfg(windows)]
    windows_input: WindowsInputReader,
}

impl CrosstermTerminal {
    fn new() -> io::Result<Self> {
        #[cfg(windows)]
        {
            Ok(Self {
                windows_input: WindowsInputReader::new()?,
            })
        }
        #[cfg(not(windows))]
        {
            Ok(Self {})
        }
    }
}

impl TerminalDriver for CrosstermTerminal {
    fn terminal_area(&self) -> io::Result<Rect> {
        let (width, height) = terminal::size()?;
        Ok(Rect::new(0, 0, width, height))
    }

    fn enable_raw_mode(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()
    }

    fn enter_alternate_screen(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, Hide)
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, Show)
    }

    fn leave_alternate_screen(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, LeaveAlternateScreen)
    }

    fn disable_raw_mode(&mut self) -> io::Result<()> {
        terminal::disable_raw_mode()
    }

    fn draw_shell(&mut self, area: Rect, state: &AppState) -> io::Result<()> {
        let mut stdout = io::stdout();
        draw_shell_to(&mut stdout, area, state)
    }

    fn draw_shell_with_workspace(
        &mut self,
        area: Rect,
        state: &AppState,
        workspace: &WorkspaceFormSnapshot,
    ) -> io::Result<()> {
        let mut stdout = io::stdout();
        draw_shell_to_with_workspace(&mut stdout, area, state, workspace)
    }

    fn read_event(&mut self) -> io::Result<Event> {
        #[cfg(windows)]
        {
            self.windows_input.read_event()
        }
        #[cfg(not(windows))]
        {
            event::read()
        }
    }

    fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<Event>> {
        #[cfg(windows)]
        {
            self.windows_input.read_event_timeout(timeout)
        }
        #[cfg(not(windows))]
        {
            if event::poll(timeout)? {
                event::read().map(Some)
            } else {
                Ok(None)
            }
        }
    }
}

fn draw_shell_to<W: Write>(writer: &mut W, area: Rect, state: &AppState) -> io::Result<()> {
    let buffer = render_shell_buffer(area, state);
    draw_buffer(writer, area, &buffer)
}

fn draw_shell_to_with_workspace<W: Write>(
    writer: &mut W,
    area: Rect,
    state: &AppState,
    workspace: &WorkspaceFormSnapshot,
) -> io::Result<()> {
    let buffer = render_shell_buffer_with_workspace(area, state, workspace);
    draw_buffer(writer, area, &buffer)
}

fn draw_buffer<W: Write>(writer: &mut W, area: Rect, buffer: &Buffer) -> io::Result<()> {
    queue!(writer, Clear(ClearType::All))?;
    for row in 0..area.height {
        let line = buffer_row_text(buffer, area, row);
        queue!(writer, MoveTo(area.x, area.y + row), Print(line))?;
    }
    // Leave the cursor at a known point; terminal ownership remains with this module.
    queue!(writer, MoveTo(area.x, area.y))?;
    writer.flush()
}

fn buffer_row_text(buffer: &Buffer, area: Rect, row: u16) -> String {
    let mut line = String::new();
    let mut column = 0;
    while column < area.width {
        let symbol = buffer[(column, row)].symbol();
        line.push_str(symbol);
        // Ratatui stores a wide grapheme in its leading cell and marks the
        // following cell as occupied. The terminal advances across both cells
        // when the grapheme is printed, so do not emit the continuation cell.
        column = column.saturating_add(symbol.cell_width().max(1));
    }
    line
}

fn render_shell_buffer(area: Rect, state: &AppState) -> Buffer {
    let mut buffer = Buffer::empty(area);
    crate::rendering::render(&mut buffer, area, state);
    buffer
}

fn render_shell_buffer_with_workspace(
    area: Rect,
    state: &AppState,
    workspace: &WorkspaceFormSnapshot,
) -> Buffer {
    let mut buffer = Buffer::empty(area);
    crate::rendering::render_with_workspace(&mut buffer, area, state, workspace);
    buffer
}

#[cfg(test)]
fn buffer_text(buffer: &Buffer, area: Rect) -> String {
    (0..area.height)
        .map(|row| {
            (0..area.width)
                .map(|column| buffer[(column, row)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::{
        TerminalDriver, TerminalSession, buffer_row_text, buffer_text, render_shell_buffer,
        render_shell_buffer_with_workspace, run_shell_with_driver, run_with_driver,
    };
    use crate::TuiErrorKind;
    use crate::WorkspaceCommands;
    use crate::input::is_shutdown_request;
    use crate::screens::WorkspaceForm;
    use crate::state::AppState;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use nt_application::workspace::{WorkspaceError, WorkspaceMetadata};
    use ratatui::buffer::CellWidth;
    use ratatui::layout::Rect;
    use ratatui::widgets::{Paragraph, Widget};
    use std::cell::RefCell;
    use std::collections::{HashSet, VecDeque};
    use std::io;
    use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
    use std::path::Path;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    enum Operation {
        EnableRaw,
        EnterAlternate,
        HideCursor,
        ShowCursor,
        LeaveAlternate,
        DisableRaw,
        Draw,
        Read,
    }

    struct FakeTerminal {
        calls: Rc<RefCell<Vec<Operation>>>,
        output: Rc<RefCell<Vec<u8>>>,
        frames: Rc<RefCell<Vec<String>>>,
        area: Rect,
        failures: HashSet<Operation>,
        panics: HashSet<Operation>,
        error_payload_panics: HashSet<Operation>,
        events: VecDeque<Option<Event>>,
        area_changes_after_reads: VecDeque<Option<Rect>>,
    }

    impl FakeTerminal {
        fn new() -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
                output: Rc::new(RefCell::new(Vec::new())),
                frames: Rc::new(RefCell::new(Vec::new())),
                area: Rect::new(0, 0, 80, 24),
                failures: HashSet::new(),
                panics: HashSet::new(),
                error_payload_panics: HashSet::new(),
                events: VecDeque::from([Some(key(KeyCode::Char('c'), KeyModifiers::CONTROL))]),
                area_changes_after_reads: VecDeque::new(),
            }
        }

        fn failing(mut self, operation: Operation) -> Self {
            self.failures.insert(operation);
            self
        }

        fn panicking(mut self, operation: Operation) -> Self {
            self.panics.insert(operation);
            self
        }

        fn error_payload_panics_when_dropped(mut self, operation: Operation) -> Self {
            self.error_payload_panics.insert(operation);
            self
        }

        fn with_events(mut self, events: impl IntoIterator<Item = Event>) -> Self {
            self.events = events.into_iter().map(Some).collect();
            self
        }

        fn with_reads(mut self, reads: impl IntoIterator<Item = Option<Event>>) -> Self {
            self.events = reads.into_iter().collect();
            self
        }

        fn with_size(mut self, width: u16, height: u16) -> Self {
            self.area = Rect::new(0, 0, width, height);
            self
        }

        fn with_area_changes_after_reads(
            mut self,
            areas: impl IntoIterator<Item = Option<Rect>>,
        ) -> Self {
            self.area_changes_after_reads = areas.into_iter().collect();
            self
        }

        fn perform(&mut self, operation: Operation) -> io::Result<()> {
            self.calls.borrow_mut().push(operation);
            if self.panics.contains(&operation) {
                panic!("synthetic terminal panic");
            }
            if self.error_payload_panics.contains(&operation) {
                return Err(io::Error::other(PanickingIoError));
            }
            if self.failures.contains(&operation) {
                Err(io::Error::other("synthetic terminal fault"))
            } else {
                Ok(())
            }
        }

        fn next_read(&mut self) -> Option<Event> {
            let event = self
                .events
                .pop_front()
                .unwrap_or_else(|| Some(key(KeyCode::Char('c'), KeyModifiers::CONTROL)));
            if let Some(area) = self.area_changes_after_reads.pop_front().flatten() {
                self.area = area;
            }
            match event {
                Some(Event::Resize(width, height)) => {
                    self.area = Rect::new(0, 0, width, height);
                    Some(Event::Resize(width, height))
                }
                event => event,
            }
        }
    }

    impl TerminalDriver for FakeTerminal {
        fn terminal_area(&self) -> io::Result<Rect> {
            Ok(self.area)
        }

        fn enable_raw_mode(&mut self) -> io::Result<()> {
            self.perform(Operation::EnableRaw)
        }
        fn enter_alternate_screen(&mut self) -> io::Result<()> {
            self.perform(Operation::EnterAlternate)
        }
        fn hide_cursor(&mut self) -> io::Result<()> {
            self.perform(Operation::HideCursor)
        }
        fn show_cursor(&mut self) -> io::Result<()> {
            self.perform(Operation::ShowCursor)
        }
        fn leave_alternate_screen(&mut self) -> io::Result<()> {
            self.perform(Operation::LeaveAlternate)
        }
        fn disable_raw_mode(&mut self) -> io::Result<()> {
            self.perform(Operation::DisableRaw)
        }
        fn draw_shell(&mut self, area: Rect, state: &AppState) -> io::Result<()> {
            self.perform(Operation::Draw)?;
            let buffer = render_shell_buffer(area, state);
            let frame = buffer_text(&buffer, area);
            self.frames.borrow_mut().push(frame.clone());
            self.output.borrow_mut().extend_from_slice(frame.as_bytes());
            Ok(())
        }
        fn read_event(&mut self) -> io::Result<Event> {
            loop {
                self.perform(Operation::Read)?;
                if let Some(event) = self.next_read() {
                    return Ok(event);
                }
            }
        }

        fn poll_event(&mut self, _timeout: Duration) -> io::Result<Option<Event>> {
            self.perform(Operation::Read)?;
            Ok(self.next_read())
        }
    }

    #[derive(Debug)]
    struct PanickingIoError;

    impl std::fmt::Display for PanickingIoError {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("synthetic I/O error")
        }
    }

    impl std::error::Error for PanickingIoError {}

    impl Drop for PanickingIoError {
        fn drop(&mut self) {
            std::panic::panic_any("synthetic I/O error payload destructor panic");
        }
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new_with_kind(
            code,
            modifiers,
            KeyEventKind::Press,
        ))
    }

    fn repeated_key(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new_with_kind(
            code,
            modifiers,
            KeyEventKind::Repeat,
        ))
    }

    #[test]
    fn successful_shutdown_restores_terminal_in_reverse_order() {
        let fake =
            FakeTerminal::new().with_events([key(KeyCode::Char('c'), KeyModifiers::CONTROL)]);
        let calls = fake.calls.clone();
        assert!(run_shell_with_driver(fake).is_ok());
        assert_eq!(
            *calls.borrow(),
            vec![
                Operation::EnableRaw,
                Operation::EnterAlternate,
                Operation::HideCursor,
                Operation::Draw,
                Operation::Read,
                Operation::ShowCursor,
                Operation::LeaveAlternate,
                Operation::DisableRaw,
            ]
        );
    }

    #[test]
    fn timeout_redraws_viewport_only_resize_and_preserves_help() {
        let fake = FakeTerminal::new()
            .with_reads([
                Some(key(KeyCode::F(1), KeyModifiers::NONE)),
                None,
                None,
                Some(key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            ])
            .with_area_changes_after_reads([
                None,
                Some(Rect::new(0, 0, 79, 22)),
                Some(Rect::new(0, 0, 80, 24)),
                None,
            ]);
        let frames = fake.frames.clone();
        let calls = fake.calls.clone();

        assert!(run_shell_with_driver(fake).is_ok());

        let frames = frames.borrow();
        assert_eq!(frames.len(), 4);
        assert!(frames[1].contains("Trợ giúp · Tổng quan"));
        assert!(frames[2].contains("79×22 quá nhỏ"));
        assert!(!frames[2].contains("Enter: Thoát"));
        assert!(frames[3].contains("Trợ giúp · Tổng quan"));
        assert_eq!(frames[3], frames[1]);
        assert_eq!(
            calls
                .borrow()
                .iter()
                .filter(|call| **call == Operation::Read)
                .count(),
            4
        );
    }

    #[test]
    fn unchanged_timeouts_do_not_redraw_the_shell() {
        let fake = FakeTerminal::new().with_reads([
            None,
            None,
            None,
            Some(key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        ]);
        let frames = fake.frames.clone();

        assert!(run_shell_with_driver(fake).is_ok());

        assert_eq!(frames.borrow().len(), 1);
    }

    #[test]
    fn default_shell_output_does_not_include_terminal_cleanup_commands() {
        let fake = FakeTerminal::new();
        let calls = fake.calls.clone();
        let output = fake.output.clone();
        let result = run_with_driver(fake, |driver| {
            let area = driver.terminal_area().map_err(|_| TuiErrorKind::Render)?;
            driver
                .draw_shell(area, &AppState::default())
                .map_err(|_| TuiErrorKind::Render)?;
            let event = driver.read_event().map_err(|_| TuiErrorKind::EventRead)?;
            assert!(is_shutdown_request(&event));
            Ok(())
        });

        assert!(result.is_ok());
        assert_eq!(
            *calls.borrow(),
            vec![
                Operation::EnableRaw,
                Operation::EnterAlternate,
                Operation::HideCursor,
                Operation::Draw,
                Operation::Read,
                Operation::ShowCursor,
                Operation::LeaveAlternate,
                Operation::DisableRaw,
            ]
        );
        let rendered = String::from_utf8(output.borrow().clone()).expect("renderer emits UTF-8");
        assert!(rendered.contains("Chưa mở workspace"));
        assert!(!rendered.contains("\u{1b}[?25h"));
    }

    struct RenderWorkspaceCommands;

    impl WorkspaceCommands for RenderWorkspaceCommands {
        fn create_workspace(&self, _path: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
            Err(WorkspaceError::StorageFailure)
        }

        fn open_workspace(&self, _path: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
            Err(WorkspaceError::NotFound)
        }
    }

    #[test]
    fn workspace_form_is_visible_at_the_minimum_supported_terminal_size() {
        let area = Rect::new(0, 0, 80, 24);
        let workspace = WorkspaceForm::new(Some(Arc::new(RenderWorkspaceCommands))).snapshot();
        let buffer = render_shell_buffer_with_workspace(area, &AppState::default(), &workspace);
        let rendered = buffer_text(&buffer, area);

        assert!(rendered.contains("Chưa mở workspace"));
        assert!(rendered.contains("C: Tạo workspace"));
        assert!(rendered.contains("O: Mở workspace"));
    }
    #[test]
    fn buffer_renderer_never_emits_a_cursor_restore_command() {
        let area = Rect::new(0, 0, 80, 24);
        let buffer = render_shell_buffer(area, &AppState::default());
        let rendered = buffer_text(&buffer, area);

        assert!(rendered.contains("Chưa mở workspace"));
        assert!(!rendered.contains("\u{1b}[?25h"));
    }

    #[test]
    fn production_row_writer_counts_wide_glyphs_and_keeps_source_spaces() {
        let area = Rect::new(0, 0, 80, 1);
        let source = "日本語  a\u{0301} Tiếng Việt";
        let mut buffer = ratatui::buffer::Buffer::empty(area);
        Paragraph::new(source).render(area, &mut buffer);

        let row = buffer_row_text(&buffer, area, 0);

        assert!(row.starts_with(source));
        assert_eq!(row.cell_width(), area.width);
    }

    #[test]
    fn tab_focuses_exit_and_enter_uses_the_lifecycle_cleanup() {
        let fake = FakeTerminal::new().with_events([
            key(KeyCode::Tab, KeyModifiers::NONE),
            repeated_key(KeyCode::Enter, KeyModifiers::NONE),
            key(KeyCode::Enter, KeyModifiers::NONE),
        ]);
        let calls = fake.calls.clone();

        let result = run_shell_with_driver(fake);

        assert!(result.is_ok());
        assert_eq!(
            *calls.borrow(),
            vec![
                Operation::EnableRaw,
                Operation::EnterAlternate,
                Operation::HideCursor,
                Operation::Draw,
                Operation::Read,
                Operation::Draw,
                Operation::Read,
                Operation::Draw,
                Operation::Read,
                Operation::ShowCursor,
                Operation::LeaveAlternate,
                Operation::DisableRaw,
            ]
        );
    }

    #[test]
    fn contextual_help_owns_the_event_loop_and_escape_restores_the_same_view() {
        let fake = FakeTerminal::new().with_events([
            key(KeyCode::Down, KeyModifiers::NONE),
            key(KeyCode::Enter, KeyModifiers::NONE),
            key(KeyCode::Tab, KeyModifiers::NONE),
            key(KeyCode::F(1), KeyModifiers::NONE),
            key(KeyCode::Enter, KeyModifiers::NONE),
            key(KeyCode::Esc, KeyModifiers::NONE),
            key(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ]);
        let frames = fake.frames.clone();

        assert!(run_shell_with_driver(fake).is_ok());

        let frames = frames.borrow();
        assert_eq!(frames.len(), 7);
        let help_frame = frames[4].split_whitespace().collect::<Vec<_>>().join(" ");
        let repeated_help_frame = frames[5].split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(help_frame.contains("Trợ giúp · Tập và chương"));
        assert!(help_frame.contains("Sau khi đóng trợ giúp bằng Esc, focus Thoát sẽ tiếp tục:"));
        assert!(help_frame.contains("Enter: Thoát · Tab: Quay về Điều hướng"));
        assert!(help_frame.contains("Esc: Đóng trợ giúp · Ctrl+C: Thoát"));
        assert!(repeated_help_frame.contains("Trợ giúp · Tập và chương"));
        assert!(repeated_help_frame.contains("Chưa có nguồn"));
        assert!(!frames[6].contains("Trợ giúp · Tập và chương"));
        assert!(frames[6].contains("Chưa có nguồn"));
        assert!(frames[6].contains("F1: Trợ giúp"));
        assert!(frames[6].contains("Enter: Thoát"));
    }

    #[test]
    fn integrated_demo_visits_every_route_resizes_help_and_restores_terminal() {
        let mut events = Vec::new();
        for _ in 0..7 {
            events.push(key(KeyCode::Down, KeyModifiers::NONE));
            events.push(key(KeyCode::Enter, KeyModifiers::NONE));
        }
        events.extend([
            key(KeyCode::Tab, KeyModifiers::NONE),
            key(KeyCode::F(1), KeyModifiers::NONE),
            Event::Resize(120, 24),
            Event::Resize(79, 24),
            Event::Resize(80, 24),
            key(KeyCode::Esc, KeyModifiers::NONE),
            key(KeyCode::Enter, KeyModifiers::NONE),
        ]);

        let fake = FakeTerminal::new().with_events(events);
        let frames = fake.frames.clone();
        let calls = fake.calls.clone();

        assert!(run_shell_with_driver(fake).is_ok());

        let frames = frames.borrow();
        assert_eq!(frames.len(), 21);
        for (route_index, route) in [
            "Tổng quan",
            "Tập và chương",
            "Tác vụ",
            "Editor Nhật–Việt",
            "Reviewer",
            "Skills và ngữ cảnh",
            "Ảnh và xuất",
            "Cấu hình, báo cáo và trợ giúp",
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                frames[route_index * 2].contains(route),
                "route {route_index} was not rendered: {route}"
            );
        }

        assert!(frames[16].contains("Trợ giúp · Cấu hình, báo cáo và trợ giúp"));
        assert!(frames[16].contains("Enter: Thoát"));
        assert!(frames[17].contains("Trợ giúp · Cấu hình, báo cáo và trợ giúp"));
        assert!(frames[18].contains("79×24 quá nhỏ"));
        assert!(!frames[18].contains("Enter: Thoát"));
        assert!(frames[19].contains("Trợ giúp · Cấu hình, báo cáo và trợ giúp"));
        assert!(frames[20].contains("Cấu hình, báo cáo và trợ giúp"));
        assert!(!frames[20].contains("Trợ giúp · Cấu hình, báo cáo và trợ giúp"));
        assert!(frames[20].contains("Enter: Thoát"));

        let calls = calls.borrow();
        assert_eq!(
            &calls[calls.len() - 3..],
            &[
                Operation::ShowCursor,
                Operation::LeaveAlternate,
                Operation::DisableRaw,
            ]
        );
        for cleanup in [
            Operation::ShowCursor,
            Operation::LeaveAlternate,
            Operation::DisableRaw,
        ] {
            assert_eq!(calls.iter().filter(|call| **call == cleanup).count(), 1);
        }
    }

    #[test]
    fn resize_hides_controls_when_too_small_and_restores_help_and_focus() {
        let fake = FakeTerminal::new().with_size(80, 24).with_events([
            key(KeyCode::Down, KeyModifiers::NONE),
            key(KeyCode::Enter, KeyModifiers::NONE),
            key(KeyCode::Tab, KeyModifiers::NONE),
            key(KeyCode::F(1), KeyModifiers::NONE),
            Event::Resize(79, 24),
            key(KeyCode::Enter, KeyModifiers::NONE),
            Event::Resize(120, 24),
            key(KeyCode::Esc, KeyModifiers::NONE),
            key(KeyCode::Enter, KeyModifiers::NONE),
        ]);
        let frames = fake.frames.clone();

        assert!(run_shell_with_driver(fake).is_ok());

        let frames = frames.borrow();
        assert_eq!(frames.len(), 9);
        for frame in [&frames[5], &frames[6]] {
            assert!(frame.contains("quá nhỏ"));
            assert!(!frame.contains("Màn hình · Điều hướng"));
            assert!(!frame.contains("Enter: Thoát"));
        }
        assert!(frames[7].contains("Trợ giúp · Tập và chương"));
        assert!(frames[7].contains("focus Thoát sẽ tiếp tục"));
        assert!(frames[8].contains("Tập và chương"));
        assert!(frames[8].contains("Enter: Thoát"));
    }

    #[test]
    fn viewport_only_shrink_before_enter_blocks_exit_and_draws_warning() {
        let fake = FakeTerminal::new()
            .with_events([
                key(KeyCode::Tab, KeyModifiers::NONE),
                key(KeyCode::Enter, KeyModifiers::NONE),
                key(KeyCode::Char('c'), KeyModifiers::CONTROL),
            ])
            .with_area_changes_after_reads([None, Some(Rect::new(0, 0, 79, 10)), None]);
        let frames = fake.frames.clone();
        let calls = fake.calls.clone();

        assert!(run_shell_with_driver(fake).is_ok());

        let frames = frames.borrow();
        assert_eq!(frames.len(), 3);
        assert!(frames[1].contains("Enter: Thoát"));
        assert!(frames[2].contains("79×10 quá nhỏ"));
        assert!(!frames[2].contains("Enter: Thoát"));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .filter(|call| **call == Operation::Read)
                .count(),
            3
        );
    }

    #[test]
    fn viewport_only_grow_redraws_controls_before_accepting_enter() {
        let fake = FakeTerminal::new()
            .with_events([
                key(KeyCode::Tab, KeyModifiers::NONE),
                key(KeyCode::F(2), KeyModifiers::NONE),
                key(KeyCode::Enter, KeyModifiers::NONE),
                key(KeyCode::Enter, KeyModifiers::NONE),
            ])
            .with_area_changes_after_reads([
                None,
                Some(Rect::new(0, 0, 79, 10)),
                Some(Rect::new(0, 0, 80, 24)),
                None,
            ]);
        let frames = fake.frames.clone();
        let calls = fake.calls.clone();

        assert!(run_shell_with_driver(fake).is_ok());

        let frames = frames.borrow();
        assert_eq!(frames.len(), 4);
        assert!(frames[0].contains("Enter: Mở"));
        assert!(frames[1].contains("Enter: Thoát"));
        assert!(frames[2].contains("79×10 quá nhỏ"));
        assert!(!frames[2].contains("Enter: Thoát"));
        assert!(frames[3].contains("Enter: Thoát"));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .filter(|call| **call == Operation::Read)
                .count(),
            4
        );
    }

    #[test]
    fn setup_failures_attempt_cleanup_for_the_current_partial_step_and_previous_steps() {
        let cases = [
            (
                Operation::EnableRaw,
                vec![Operation::EnableRaw, Operation::DisableRaw],
            ),
            (
                Operation::EnterAlternate,
                vec![
                    Operation::EnableRaw,
                    Operation::EnterAlternate,
                    Operation::LeaveAlternate,
                    Operation::DisableRaw,
                ],
            ),
            (
                Operation::HideCursor,
                vec![
                    Operation::EnableRaw,
                    Operation::EnterAlternate,
                    Operation::HideCursor,
                    Operation::ShowCursor,
                    Operation::LeaveAlternate,
                    Operation::DisableRaw,
                ],
            ),
        ];

        for (failed_step, expected_calls) in cases {
            let fake = FakeTerminal::new().failing(failed_step);
            let calls = fake.calls.clone();
            let mut session = TerminalSession::new(fake);
            assert_eq!(session.setup(), Err(TuiErrorKind::TerminalSetup));
            assert_eq!(session.cleanup(), None);
            assert_eq!(*calls.borrow(), expected_calls);
        }
    }

    #[test]
    fn draw_and_read_failures_restore_all_acquired_resources() {
        for (failed_step, expected_kind, expected_prefix) in [
            (Operation::Draw, TuiErrorKind::Render, vec![Operation::Draw]),
            (
                Operation::Read,
                TuiErrorKind::EventRead,
                vec![Operation::Draw, Operation::Read],
            ),
        ] {
            let fake = FakeTerminal::new().failing(failed_step);
            let calls = fake.calls.clone();
            let result = run_with_driver(fake, |driver| {
                let area = driver.terminal_area().map_err(|_| TuiErrorKind::Render)?;
                driver
                    .draw_shell(area, &AppState::default())
                    .map_err(|_| TuiErrorKind::Render)?;
                driver.read_event().map_err(|_| TuiErrorKind::EventRead)?;
                Ok(())
            });
            let error = result.expect_err("fault must return an operational error");
            assert_eq!(error.kind(), expected_kind);
            assert_eq!(error.cleanup_kind(), None);

            let expected = [
                vec![
                    Operation::EnableRaw,
                    Operation::EnterAlternate,
                    Operation::HideCursor,
                ],
                expected_prefix,
                vec![
                    Operation::ShowCursor,
                    Operation::LeaveAlternate,
                    Operation::DisableRaw,
                ],
            ]
            .concat();
            assert_eq!(*calls.borrow(), expected);
        }
    }

    #[test]
    fn cleanup_continues_after_each_cleanup_step_fails_and_preserves_primary() {
        for failed_cleanup in [
            Operation::ShowCursor,
            Operation::LeaveAlternate,
            Operation::DisableRaw,
        ] {
            let fake = FakeTerminal::new().failing(failed_cleanup);
            let calls = fake.calls.clone();
            let result = run_with_driver(fake, |driver| {
                let area = driver.terminal_area().map_err(|_| TuiErrorKind::Render)?;
                driver
                    .draw_shell(area, &AppState::default())
                    .map_err(|_| TuiErrorKind::Render)?;
                Err(TuiErrorKind::Render)
            });
            let error = result.expect_err("cleanup failure must not report success");
            assert_eq!(error.kind(), TuiErrorKind::Render);
            assert_eq!(error.cleanup_kind(), Some(TuiErrorKind::TerminalCleanup));
            assert_eq!(
                *calls.borrow(),
                vec![
                    Operation::EnableRaw,
                    Operation::EnterAlternate,
                    Operation::HideCursor,
                    Operation::Draw,
                    Operation::ShowCursor,
                    Operation::LeaveAlternate,
                    Operation::DisableRaw,
                ]
            );
        }
    }

    #[test]
    fn cleanup_panics_continue_and_keep_the_original_failure_primary() {
        for panicking_cleanup in [
            Operation::ShowCursor,
            Operation::LeaveAlternate,
            Operation::DisableRaw,
        ] {
            let fake = FakeTerminal::new()
                .failing(Operation::Read)
                .panicking(panicking_cleanup);
            let calls = fake.calls.clone();
            let result = with_suppressed_panic_hook(|| {
                run_with_driver(fake, |driver| {
                    driver.read_event().map_err(|_| TuiErrorKind::EventRead)?;
                    Ok(())
                })
            });
            let error = result.expect_err("cleanup panic must not erase the primary error");

            assert_eq!(error.kind(), TuiErrorKind::EventRead);
            assert_eq!(error.cleanup_kind(), Some(TuiErrorKind::TerminalCleanup));
            assert_eq!(
                *calls.borrow(),
                vec![
                    Operation::EnableRaw,
                    Operation::EnterAlternate,
                    Operation::HideCursor,
                    Operation::Read,
                    Operation::ShowCursor,
                    Operation::LeaveAlternate,
                    Operation::DisableRaw,
                ]
            );
        }
    }

    #[test]
    fn cleanup_panic_without_an_earlier_failure_still_returns_failure() {
        for panicking_cleanup in [
            Operation::ShowCursor,
            Operation::LeaveAlternate,
            Operation::DisableRaw,
        ] {
            let fake = FakeTerminal::new().panicking(panicking_cleanup);
            let calls = fake.calls.clone();
            let result = with_suppressed_panic_hook(|| run_with_driver(fake, |_driver| Ok(())));
            let error = result.expect_err("cleanup panic cannot report success");

            assert_eq!(error.kind(), TuiErrorKind::TerminalCleanup);
            assert_eq!(error.cleanup_kind(), None);
            assert_eq!(
                *calls.borrow(),
                vec![
                    Operation::EnableRaw,
                    Operation::EnterAlternate,
                    Operation::HideCursor,
                    Operation::ShowCursor,
                    Operation::LeaveAlternate,
                    Operation::DisableRaw,
                ]
            );
        }
    }

    #[test]
    fn cleanup_error_payload_panics_are_contained_for_each_step() {
        for panicking_cleanup in [
            Operation::ShowCursor,
            Operation::LeaveAlternate,
            Operation::DisableRaw,
        ] {
            let fake = FakeTerminal::new()
                .failing(Operation::Read)
                .error_payload_panics_when_dropped(panicking_cleanup);
            let calls = fake.calls.clone();
            let result = with_suppressed_panic_hook(|| {
                run_with_driver(fake, |driver| {
                    driver.read_event().map_err(|_| TuiErrorKind::EventRead)?;
                    Ok(())
                })
            });
            let error = result.expect_err("I/O error payload panic cannot erase the failure");

            assert_eq!(error.kind(), TuiErrorKind::EventRead);
            assert_eq!(error.cleanup_kind(), Some(TuiErrorKind::TerminalCleanup));
            assert_eq!(
                *calls.borrow(),
                vec![
                    Operation::EnableRaw,
                    Operation::EnterAlternate,
                    Operation::HideCursor,
                    Operation::Read,
                    Operation::ShowCursor,
                    Operation::LeaveAlternate,
                    Operation::DisableRaw,
                ]
            );

            let fake = FakeTerminal::new().error_payload_panics_when_dropped(panicking_cleanup);
            let calls = fake.calls.clone();
            let result = with_suppressed_panic_hook(|| run_with_driver(fake, |_driver| Ok(())));
            let error = result.expect_err("cleanup-only I/O payload panic cannot report success");

            assert_eq!(error.kind(), TuiErrorKind::TerminalCleanup);
            assert_eq!(error.cleanup_kind(), None);
            assert_eq!(
                *calls.borrow(),
                vec![
                    Operation::EnableRaw,
                    Operation::EnterAlternate,
                    Operation::HideCursor,
                    Operation::ShowCursor,
                    Operation::LeaveAlternate,
                    Operation::DisableRaw,
                ]
            );
        }
    }

    struct PanickingDrop;

    impl Drop for PanickingDrop {
        fn drop(&mut self) {
            panic!("synthetic terminal payload destructor panic");
        }
    }

    #[test]
    fn panic_payload_destructor_is_contained_after_terminal_cleanup() {
        let fake = FakeTerminal::new();
        let calls = fake.calls.clone();
        let result = with_suppressed_panic_hook(|| {
            run_with_driver(fake, |_driver| {
                std::panic::panic_any(PanickingDrop);
            })
        });
        let error = result.expect_err("panic payload destructor must not escape");

        assert_eq!(error.kind(), TuiErrorKind::Panic);
        assert_eq!(error.cleanup_kind(), None);
        assert_eq!(
            *calls.borrow(),
            vec![
                Operation::EnableRaw,
                Operation::EnterAlternate,
                Operation::HideCursor,
                Operation::ShowCursor,
                Operation::LeaveAlternate,
                Operation::DisableRaw,
            ]
        );
    }

    #[test]
    fn cleanup_failure_without_an_earlier_error_still_returns_failure() {
        let fake = FakeTerminal::new().failing(Operation::ShowCursor);
        let calls = fake.calls.clone();
        let output = fake.output.clone();
        let result = run_with_driver(fake, |driver| {
            let area = driver.terminal_area().map_err(|_| TuiErrorKind::Render)?;
            driver
                .draw_shell(area, &AppState::default())
                .map_err(|_| TuiErrorKind::Render)?;
            Ok(())
        });
        let error = result.expect_err("cleanup failure cannot return success");

        assert_eq!(error.kind(), TuiErrorKind::TerminalCleanup);
        assert_eq!(error.cleanup_kind(), None);
        assert_eq!(
            *calls.borrow(),
            vec![
                Operation::EnableRaw,
                Operation::EnterAlternate,
                Operation::HideCursor,
                Operation::Draw,
                Operation::ShowCursor,
                Operation::LeaveAlternate,
                Operation::DisableRaw,
            ]
        );
        let rendered = String::from_utf8(output.borrow().clone()).expect("renderer emits UTF-8");
        assert!(!rendered.contains("Failed to show the cursor"));
        assert!(!rendered.contains("synthetic terminal fault"));
    }

    #[test]
    fn cleanup_is_idempotent_and_does_not_repeat_terminal_commands() {
        let mut session = TerminalSession::new(FakeTerminal::new().failing(Operation::ShowCursor));
        assert_eq!(session.setup(), Ok(()));
        let cleanup_result = session.cleanup();
        assert_eq!(cleanup_result, Some(TuiErrorKind::TerminalCleanup));
        let first_call_count = session.driver.calls.borrow().len();
        assert_eq!(session.cleanup(), cleanup_result);
        assert_eq!(session.driver.calls.borrow().len(), first_call_count);
    }

    #[test]
    fn panic_unwind_runs_cleanup_and_returns_only_the_panic_kind() {
        const SECRET_SENTINEL: &str = "P01_T12_FAKE_PANIC_SECRET_SENTINEL";
        let fake = FakeTerminal::new();
        let calls = fake.calls.clone();
        let output = fake.output.clone();
        let result = with_suppressed_panic_hook(|| {
            run_with_driver(fake, |driver| {
                let area = driver.terminal_area().map_err(|_| TuiErrorKind::Render)?;
                driver
                    .draw_shell(area, &AppState::default())
                    .map_err(|_| TuiErrorKind::Render)?;
                panic!("synthetic panic payload contains {SECRET_SENTINEL}");
            })
        });
        let error = result.expect_err("panic unwind must return failure");
        assert_eq!(error.kind(), TuiErrorKind::Panic);
        assert_eq!(error.cleanup_kind(), None);
        let safe_error = format!("{error:?} {error}");
        assert!(!safe_error.contains(SECRET_SENTINEL));
        let rendered = String::from_utf8(output.borrow().clone()).expect("renderer emits UTF-8");
        assert!(!rendered.contains(SECRET_SENTINEL));
        assert_eq!(
            *calls.borrow(),
            vec![
                Operation::EnableRaw,
                Operation::EnterAlternate,
                Operation::HideCursor,
                Operation::Draw,
                Operation::ShowCursor,
                Operation::LeaveAlternate,
                Operation::DisableRaw,
            ]
        );
    }

    #[test]
    fn shutdown_requests_are_only_press_events() {
        assert!(is_shutdown_request(&key(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL
        )));
        assert!(!is_shutdown_request(&key(
            KeyCode::Char('q'),
            KeyModifiers::NONE
        )));
        assert!(!is_shutdown_request(&key(KeyCode::Esc, KeyModifiers::NONE)));
        assert!(!is_shutdown_request(&key(
            KeyCode::Char('x'),
            KeyModifiers::NONE
        )));

        let release = Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
            KeyEventKind::Release,
        ));
        assert!(!is_shutdown_request(&release));
    }

    fn with_suppressed_panic_hook<R>(operation: impl FnOnce() -> R) -> R {
        static HOOK_LOCK: Mutex<()> = Mutex::new(());
        let _guard = HOOK_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let outcome = catch_unwind(AssertUnwindSafe(operation));
        std::panic::set_hook(previous_hook);
        match outcome {
            Ok(value) => value,
            Err(payload) => resume_unwind(payload),
        }
    }
}
