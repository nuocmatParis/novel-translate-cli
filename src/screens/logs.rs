use crate::state::AppState;
use crate::state::logs::{LogLevel, MockLogEntry};
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

pub struct LogsScreen {
    pub selected_idx: usize,
    pub filter_level: Option<LogLevel>,
    pub search_query: String,
    pub is_searching: bool,
    pub show_details_modal: bool,
}

impl Default for LogsScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl LogsScreen {
    pub fn new() -> Self {
        Self {
            selected_idx: 0,
            filter_level: None,
            search_query: String::new(),
            is_searching: false,
            show_details_modal: false,
        }
    }

    pub fn filtered_logs<'a>(&self, state: &'a AppState) -> Vec<&'a MockLogEntry> {
        let q = self.search_query.trim().to_lowercase();
        state
            .logs
            .iter()
            .rev()
            .filter(|e| {
                if let Some(lvl) = self.filter_level {
                    if e.level != lvl {
                        return false;
                    }
                }
                if !q.is_empty() {
                    let text = format!("{} {} {} {}", e.event, e.message, e.model, e.status)
                        .to_lowercase();
                    if !text.contains(&q) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    pub fn next(&mut self, state: &AppState) {
        let len = self.filtered_logs(state).len();
        if len > 0 {
            self.selected_idx = (self.selected_idx + 1) % len;
        }
    }

    pub fn prev(&mut self, state: &AppState) {
        let len = self.filtered_logs(state).len();
        if len > 0 {
            if self.selected_idx == 0 {
                self.selected_idx = len - 1;
            } else {
                self.selected_idx -= 1;
            }
        }
    }

    pub fn toggle_filter(&mut self) {
        self.filter_level = match self.filter_level {
            None => Some(LogLevel::Info),
            Some(LogLevel::Info) => Some(LogLevel::Warn),
            Some(LogLevel::Warn) => Some(LogLevel::Error),
            Some(LogLevel::Error) => None,
            _ => None,
        };
        self.selected_idx = 0;
    }

    pub fn toggle_details(&mut self) {
        self.show_details_modal = !self.show_details_modal;
    }
}

pub fn render_logs(
    f: &mut Frame,
    area: Rect,
    screen: &LogsScreen,
    state: &AppState,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Filter & Search Header
            Constraint::Min(10),   // Logs Table
        ])
        .split(area);

    // 1. Filter Header
    let filter_text = match screen.filter_level {
        None => "ALL LEVELS",
        Some(LogLevel::Info) => "INFO ONLY",
        Some(LogLevel::Warn) => "WARN ONLY",
        Some(LogLevel::Error) => "ERROR ONLY",
        Some(LogLevel::Debug) => "DEBUG ONLY",
    };

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(Span::styled(
            " 📋 ACTIVITY STREAM & SYSTEM LOGS ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let header_line = Line::from(vec![
        Span::styled(" Filter: ", Style::default().fg(theme.text_dim)),
        Span::styled(
            format!(" [f] {} ", filter_text),
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ Search: ", Style::default().fg(theme.text_dim)),
        Span::styled(
            if screen.search_query.is_empty() {
                "None (Press [/] to search)"
            } else {
                &screen.search_query
            },
            Style::default().fg(theme.primary),
        ),
        Span::styled(" │ Actions: ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "[Enter] View Details  [↑/↓] Scroll",
            Style::default().fg(theme.text),
        ),
    ]);
    f.render_widget(Paragraph::new(header_line).block(header_block), chunks[0]);

    // 2. Logs Table / List
    let logs_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border_focused))
        .title(" Timestamp │ Level │ Event │ Volume/Ch │ Model │ Message ")
        .style(Style::default().bg(theme.panel_bg));

    let filtered = screen.filtered_logs(state);

    if filtered.is_empty() {
        let empty_msg = Paragraph::new("  No logs matching the current filter.")
            .style(Style::default().fg(theme.text_dim))
            .block(logs_block);
        f.render_widget(empty_msg, chunks[1]);
        return;
    }

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_sel = idx == screen.selected_idx;
            let bg = if is_sel {
                theme.selection_bg
            } else {
                Color::Reset
            };
            let prefix = if is_sel { "▶ " } else { "  " };

            let lvl_color = match entry.level {
                LogLevel::Info => theme.info,
                LogLevel::Warn => theme.warning,
                LogLevel::Error => theme.error,
                LogLevel::Debug => theme.text_dim,
            };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.primary)),
                Span::styled(
                    format!("{:8} ", entry.timestamp),
                    Style::default().fg(theme.text_dim),
                ),
                Span::styled(
                    format!("[{:^5}] ", entry.level.label()),
                    Style::default().fg(lvl_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:22} ", entry.event),
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} › {} ", entry.volume, entry.chapter),
                    Style::default().fg(theme.primary_dim),
                ),
                Span::styled(
                    format!("({:18}) ", entry.model),
                    Style::default().fg(theme.text_dim),
                ),
                Span::styled(&entry.message, Style::default().fg(theme.text_muted)),
            ]);

            ListItem::new(line).style(Style::default().bg(bg))
        })
        .collect();

    f.render_widget(List::new(items).block(logs_block), chunks[1]);

    // Details Modal
    if screen.show_details_modal {
        if let Some(entry) = filtered.get(screen.selected_idx) {
            render_log_details_modal(f, entry, theme);
        }
    }
}

fn render_log_details_modal(f: &mut Frame, entry: &MockLogEntry, theme: &Theme) {
    let area = f.area();
    let width = 72.min(area.width.saturating_sub(4));
    let height = 14.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let modal_rect = Rect {
        x,
        y,
        width,
        height,
    };

    f.render_widget(Clear, modal_rect);

    let modal_block = Block::default()
        .title(Span::styled(
            " 📋 LOG EVENT INSPECTOR (Enter/Esc to close) ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(Color::Rgb(15, 23, 42)));

    let inner = modal_block.inner(modal_rect);
    f.render_widget(modal_block, modal_rect);

    let lines = vec![
        Line::from(vec![
            Span::styled(" Event ID: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("#{}", entry.id),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ Timestamp: ", Style::default().fg(theme.text_muted)),
            Span::styled(&entry.timestamp, Style::default().fg(theme.primary)),
            Span::styled(" │ Level: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                entry.level.label(),
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Event: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                &entry.event,
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Context: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{} › {}", entry.volume, entry.chapter),
                Style::default().fg(theme.primary_dim),
            ),
            Span::styled(" │ Model: ", Style::default().fg(theme.text_muted)),
            Span::styled(&entry.model, Style::default().fg(theme.secondary)),
            Span::styled(" │ Status: ", Style::default().fg(theme.text_muted)),
            Span::styled(&entry.status, Style::default().fg(theme.success)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " Detailed Message Payload:",
            Style::default().fg(theme.text_dim),
        )),
        Line::from(Span::styled(
            &entry.message,
            Style::default().fg(theme.text),
        )),
    ];

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}
