use crate::state::AppState;
use crate::state::workflow::TranslationMode;
use crate::state::workspace::VolumeStatus;
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

#[derive(Default, PartialEq, Eq)]
pub enum LibraryFocus {
    #[default]
    Volumes,
    Chapters,
}

pub struct LibraryScreen {
    pub focus: LibraryFocus,
    pub selected_volume: usize,
    pub selected_chapter: usize,
}

impl Default for LibraryScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryScreen {
    pub fn new() -> Self {
        Self {
            focus: LibraryFocus::Volumes,
            selected_volume: 2,  // Default to Volume 26 (Arc 7)
            selected_chapter: 4, // Default to Chapter 04
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            LibraryFocus::Volumes => LibraryFocus::Chapters,
            LibraryFocus::Chapters => LibraryFocus::Volumes,
        };
    }

    pub fn next(&mut self, state: &AppState) {
        match self.focus {
            LibraryFocus::Volumes => {
                let total = state.workspace.series.volumes.len();
                if total > 0 {
                    self.selected_volume = (self.selected_volume + 1) % total;
                }
            }
            LibraryFocus::Chapters => {
                if let Some(vol) = state.workspace.series.volumes.get(self.selected_volume) {
                    if !vol.chapters.is_empty() {
                        self.selected_chapter = (self.selected_chapter + 1) % vol.chapters.len();
                    }
                }
            }
        }
    }

    pub fn prev(&mut self, state: &AppState) {
        match self.focus {
            LibraryFocus::Volumes => {
                let total = state.workspace.series.volumes.len();
                if total > 0 {
                    if self.selected_volume == 0 {
                        self.selected_volume = total - 1;
                    } else {
                        self.selected_volume -= 1;
                    }
                }
            }
            LibraryFocus::Chapters => {
                if let Some(vol) = state.workspace.series.volumes.get(self.selected_volume) {
                    if !vol.chapters.is_empty() {
                        if self.selected_chapter == 0 {
                            self.selected_chapter = vol.chapters.len() - 1;
                        } else {
                            self.selected_chapter -= 1;
                        }
                    }
                }
            }
        }
    }

    pub fn toggle_chapter_selection(&mut self, state: &mut AppState) {
        if let Some(vol) = state.workspace.series.volumes.get_mut(self.selected_volume) {
            if let Some(ch) = vol.chapters.get_mut(self.selected_chapter) {
                ch.selected = !ch.selected;
            }
        }
    }

    pub fn start_selected_translation(&mut self, state: &mut AppState) {
        state.task.mode = TranslationMode::SelectedChapters;
        state.task.is_running = true;
        state.task.is_paused = false;
        state.task.current_operation =
            "Initiating translation on user-selected chapter scope".to_string();
    }

    pub fn start_auto_translate_all(&mut self, state: &mut AppState) {
        state.task.mode = TranslationMode::AutoTranslateEpub;
        state.task.is_running = true;
        state.task.is_paused = false;
        state.task.current_operation =
            "Auto-translating sequentially to end of EPUB volume".to_string();
    }
}

pub fn render_library(
    f: &mut Frame,
    area: Rect,
    screen: &LibraryScreen,
    state: &AppState,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left Pane: Volumes List
    let vol_focused = screen.focus == LibraryFocus::Volumes;
    let vol_border = if vol_focused {
        theme.panel_border_focused
    } else {
        theme.panel_border
    };
    let vol_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(vol_border))
        .title(Span::styled(
            " 📚 VOLUMES (↑/↓ to choose, Tab to switch) ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let vol_items: Vec<ListItem> = state
        .workspace
        .series
        .volumes
        .iter()
        .enumerate()
        .map(|(idx, vol)| {
            let is_sel = idx == screen.selected_volume;
            let bg = if is_sel {
                theme.selection_bg
            } else {
                Color::Reset
            };
            let (status_color, status_text) = match vol.status {
                VolumeStatus::Completed => (theme.success, "✓ Completed"),
                VolumeStatus::InProgress => (theme.warning, "▶ In Progress"),
                VolumeStatus::NotStarted => (theme.text_dim, "○ Not Started"),
            };

            let prefix = if is_sel { " ▶ " } else { "   " };
            let line1 = Line::from(vec![
                Span::styled(
                    prefix,
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &vol.title,
                    if is_sel {
                        Style::default()
                            .fg(theme.selection_fg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
            ]);
            let line2 = Line::from(vec![
                Span::raw("     "),
                Span::styled(status_text, Style::default().fg(status_color)),
                Span::styled(
                    format!("  ({}% translated)", vol.progress_percent),
                    Style::default().fg(theme.text_dim),
                ),
            ]);

            ListItem::new(vec![line1, line2]).style(Style::default().bg(bg))
        })
        .collect();

    f.render_widget(List::new(vol_items).block(vol_block), chunks[0]);

    // Right Pane: Chapters List & Scope Actions
    let ch_focused = screen.focus == LibraryFocus::Chapters;
    let ch_border = if ch_focused {
        theme.panel_border_focused
    } else {
        theme.panel_border
    };

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(6)])
        .split(chunks[1]);

    let active_vol = state.workspace.series.volumes.get(screen.selected_volume);
    let ch_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ch_border))
        .title(Span::styled(
            format!(
                " 📑 CHAPTERS — {} ([Space] Toggle, [t] Translate Selected) ",
                active_vol.map(|v| v.title.as_str()).unwrap_or("")
            ),
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    if let Some(vol) = active_vol {
        if vol.chapters.is_empty() {
            let empty_msg = Paragraph::new("  No chapters imported for this volume yet.")
                .style(Style::default().fg(theme.text_dim))
                .block(ch_block);
            f.render_widget(empty_msg, right_chunks[0]);
        } else {
            let ch_items: Vec<ListItem> = vol
                .chapters
                .iter()
                .enumerate()
                .map(|(idx, ch)| {
                    let is_sel = idx == screen.selected_chapter;
                    let bg = if is_sel {
                        theme.selection_bg
                    } else {
                        Color::Reset
                    };
                    let checkbox = if ch.selected { "[x]" } else { "[ ]" };
                    let check_color = if ch.selected {
                        theme.primary
                    } else {
                        theme.text_dim
                    };

                    let (status_color, status_text) = match ch.status {
                        crate::state::workspace::ChapterStatus::Completed => {
                            (theme.success, "✓ Completed")
                        }
                        crate::state::workspace::ChapterStatus::InReview => {
                            (theme.warning, "⚖ In Review")
                        }
                        crate::state::workspace::ChapterStatus::Translating => {
                            (theme.primary, "⚡ Translating")
                        }
                        crate::state::workspace::ChapterStatus::Queued => (theme.info, "⏱ Queued"),
                        crate::state::workspace::ChapterStatus::NotStarted => {
                            (theme.text_dim, "○ Not Started")
                        }
                    };

                    let line = Line::from(vec![
                        Span::styled(
                            format!(" {} ", checkbox),
                            Style::default()
                                .fg(check_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            &ch.title,
                            if is_sel {
                                Style::default()
                                    .fg(theme.selection_fg)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(theme.text)
                            },
                        ),
                        Span::styled(
                            format!(" ({}) ", ch.original_title),
                            Style::default().fg(theme.text_dim),
                        ),
                        Span::styled(
                            format!(" [{}]", status_text),
                            Style::default().fg(status_color),
                        ),
                        Span::styled(
                            format!("  {}/{} blk", ch.translated_blocks, ch.total_blocks),
                            Style::default().fg(theme.text_muted),
                        ),
                    ]);

                    ListItem::new(line).style(Style::default().bg(bg))
                })
                .collect();

            f.render_widget(List::new(ch_items).block(ch_block), right_chunks[0]);
        }
    }

    // Bottom Action Controls Panel
    let action_box = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(Span::styled(
            " ⚡ TRANSLATION SCOPE ACTIONS ",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let action_text = vec![
        Line::from(vec![
            Span::styled(
                " [t] ",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Start Selected Chapters (2 chapters marked)",
                Style::default().fg(theme.text),
            ),
            Span::styled(
                "  — Translate only checked chapters",
                Style::default().fg(theme.text_muted),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                " [a] ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Auto Translate to End of EPUB",
                Style::default().fg(theme.text),
            ),
            Span::styled(
                "  — Sequentially translate remainder of volume without manual pausing",
                Style::default().fg(theme.text_muted),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Active Mode: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                state.task.mode.label(),
                Style::default()
                    .fg(theme.primary_dim)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    f.render_widget(
        Paragraph::new(action_text).block(action_box),
        right_chunks[1],
    );
}
