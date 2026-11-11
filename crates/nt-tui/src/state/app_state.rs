use crate::navigation::Screen;

pub(crate) const MIN_TERMINAL_COLUMNS: u16 = 80;
pub(crate) const MIN_TERMINAL_ROWS: u16 = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FocusRegion {
    Navigation,
    Exit,
}

impl FocusRegion {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Navigation => "Điều hướng",
            Self::Exit => "Thoát",
        }
    }

    pub(crate) const fn next(self, _reverse: bool) -> Self {
        match self {
            Self::Navigation => Self::Exit,
            Self::Exit => Self::Navigation,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AppState {
    current_screen: Screen,
    selected_screen: Screen,
    focus_region: FocusRegion,
    help_open: bool,
    terminal_width: u16,
    terminal_height: u16,
    scroll_offsets: [u16; Screen::ALL.len()],
    scroll_anchor_bytes: [usize; Screen::ALL.len()],
    scroll_limit: u16,
    scroll_page_lines: u16,
}

impl AppState {
    pub(crate) const fn new() -> Self {
        Self {
            current_screen: Screen::Overview,
            selected_screen: Screen::Overview,
            focus_region: FocusRegion::Navigation,
            help_open: false,
            terminal_width: MIN_TERMINAL_COLUMNS,
            terminal_height: MIN_TERMINAL_ROWS,
            scroll_offsets: [0; Screen::ALL.len()],
            scroll_anchor_bytes: [0; Screen::ALL.len()],
            scroll_limit: 0,
            scroll_page_lines: 1,
        }
    }

    pub(crate) const fn current_screen(self) -> Screen {
        self.current_screen
    }

    pub(crate) const fn selected_screen(self) -> Screen {
        self.selected_screen
    }

    pub(crate) const fn focus_region(self) -> FocusRegion {
        self.focus_region
    }

    pub(crate) const fn help_open(self) -> bool {
        self.help_open
    }

    pub(crate) const fn terminal_size_supported(self) -> bool {
        self.terminal_width >= MIN_TERMINAL_COLUMNS && self.terminal_height >= MIN_TERMINAL_ROWS
    }

    pub(crate) fn set_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_width = width;
        self.terminal_height = height;
    }

    pub(crate) const fn terminal_size(self) -> (u16, u16) {
        (self.terminal_width, self.terminal_height)
    }

    pub(crate) const fn scroll_offset(self) -> u16 {
        let offset = self.scroll_offsets[self.current_screen.index()];
        if offset < self.scroll_limit {
            offset
        } else {
            self.scroll_limit
        }
    }

    pub(crate) const fn can_scroll_up(self) -> bool {
        self.scroll_offset() > 0
    }

    pub(crate) const fn can_scroll_down(self) -> bool {
        self.scroll_offset() < self.scroll_limit
    }

    pub(crate) fn set_scroll_metrics(&mut self, page_lines: u16, limit: u16) {
        self.scroll_page_lines = page_lines.max(1);
        self.scroll_limit = limit;
    }

    pub(crate) const fn scroll_anchor_byte(self) -> usize {
        self.scroll_anchor_bytes[self.current_screen.index()]
    }

    pub(crate) fn set_scroll_anchor_byte(&mut self, byte: usize) {
        self.scroll_anchor_bytes[self.current_screen.index()] = byte;
    }

    pub(crate) fn set_scroll_offset(&mut self, offset: u16) {
        self.scroll_offsets[self.current_screen.index()] = offset;
    }

    pub(crate) fn scroll_page(&mut self, down: bool) {
        let offset = if down {
            self.scroll_offset()
                .saturating_add(self.scroll_page_lines)
                .min(self.scroll_limit)
        } else {
            self.scroll_offset().saturating_sub(self.scroll_page_lines)
        };
        self.scroll_offsets[self.current_screen.index()] = offset;
    }

    pub(crate) fn open_help(&mut self) {
        self.help_open = true;
    }

    pub(crate) fn close_help(&mut self) {
        self.help_open = false;
    }

    pub(crate) fn navigate_to(&mut self, screen: Screen) {
        self.current_screen = screen;
        self.selected_screen = screen;
    }

    pub(crate) fn move_selection(&mut self, down: bool) {
        let index = self.selected_screen.index();
        let next_index = if down {
            (index + 1).min(Screen::ALL.len() - 1)
        } else {
            index.saturating_sub(1)
        };
        self.selected_screen = Screen::ALL[next_index];
    }

    pub(crate) fn open_selected_route(&mut self) -> Screen {
        let selected = self.selected_screen;
        self.navigate_to(selected);
        self.current_screen
    }

    pub(crate) fn cycle_focus(&mut self, reverse: bool) {
        self.focus_region = self.focus_region.next(reverse);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
