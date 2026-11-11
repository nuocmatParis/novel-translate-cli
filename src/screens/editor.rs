use crate::state::AppState;
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

#[derive(PartialEq, Eq)]
pub enum EditorPane {
    BlocksList,
    JapaneseSource,
    VietnameseTranslation,
}

pub struct EditorScreen {
    pub active_pane: EditorPane,
    pub selected_block_idx: usize,
    pub is_editing: bool,
    pub edit_buffer: String,
    pub reading_mode: bool,
    pub show_history: bool,
    pub reading_scroll: u16,
}

impl Default for EditorScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorScreen {
    pub fn new() -> Self {
        Self {
            active_pane: EditorPane::VietnameseTranslation,
            selected_block_idx: 1, // Default to Block #B042 (with Issue)
            is_editing: false,
            edit_buffer: String::new(),
            reading_mode: false,
            show_history: false,
            reading_scroll: 0,
        }
    }

    pub fn next_block(&mut self, state: &AppState) {
        if !self.is_editing && !state.editor_blocks.is_empty() {
            self.selected_block_idx = (self.selected_block_idx + 1) % state.editor_blocks.len();
        }
    }

    pub fn prev_block(&mut self, state: &AppState) {
        if !self.is_editing && !state.editor_blocks.is_empty() {
            if self.selected_block_idx == 0 {
                self.selected_block_idx = state.editor_blocks.len() - 1;
            } else {
                self.selected_block_idx -= 1;
            }
        }
    }

    pub fn toggle_pane(&mut self) {
        if !self.is_editing {
            self.active_pane = match self.active_pane {
                EditorPane::BlocksList => EditorPane::JapaneseSource,
                EditorPane::JapaneseSource => EditorPane::VietnameseTranslation,
                EditorPane::VietnameseTranslation => EditorPane::BlocksList,
            };
        }
    }

    pub fn toggle_reading_mode(&mut self) {
        if !self.is_editing {
            self.reading_mode = !self.reading_mode;
        }
    }

    pub fn toggle_history(&mut self) {
        if !self.is_editing {
            self.show_history = !self.show_history;
        }
    }

    pub fn start_editing(&mut self, state: &AppState) {
        if let Some(blk) = state.editor_blocks.get(self.selected_block_idx) {
            self.edit_buffer = blk.vietnamese.clone();
            self.is_editing = true;
            self.active_pane = EditorPane::VietnameseTranslation;
        }
    }

    pub fn cancel_editing(&mut self) {
        self.is_editing = false;
        self.edit_buffer.clear();
    }

    pub fn save_revision(&mut self, state: &mut AppState) {
        if self.is_editing {
            if let Some(blk) = state.editor_blocks.get_mut(self.selected_block_idx) {
                blk.vietnamese = self.edit_buffer.clone();
                blk.revision += 1;
                blk.last_modified = "10:18:45".to_string();
                blk.history.push(format!(
                    "Rev {}: Chỉnh sửa thủ công trong Editor",
                    blk.revision
                ));
            }
            self.is_editing = false;
            self.edit_buffer.clear();
            state.add_log(
                crate::state::logs::LogLevel::Info,
                "Translation Revised",
                "Updated Vietnamese block translation in-memory and incremented revision",
            );
        }
    }

    pub fn handle_edit_char(&mut self, c: char) {
        if self.is_editing {
            self.edit_buffer.push(c);
        }
    }

    pub fn handle_edit_backspace(&mut self) {
        if self.is_editing {
            self.edit_buffer.pop();
        }
    }
}

pub fn render_editor(
    f: &mut Frame,
    area: Rect,
    screen: &EditorScreen,
    state: &AppState,
    theme: &Theme,
) {
    if screen.reading_mode {
        render_reading_mode(f, area, screen, state, theme);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Block Metadata Banner
            Constraint::Min(10),   // Split View or 3-pane layout
            Constraint::Length(3), // Editor Controls Helper
        ])
        .split(area);

    let current_block = state.editor_blocks.get(screen.selected_block_idx);

    // 1. Metadata Banner
    let meta_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(Span::styled(
            " ✎ TRANSLATION EDITOR — Vol. 26 / Ch. 04 Huyết chiến săn mồi ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    if let Some(blk) = current_block {
        let status_style = if blk.review_status.contains("Pending") {
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD)
        };

        let meta_line1 = Line::from(vec![
            Span::styled(" Block ID: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("#{}", blk.id),
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    " (Block {} of {})",
                    blk.block_num,
                    state.editor_blocks.len()
                ),
                Style::default().fg(theme.text_dim),
            ),
            Span::styled(" │ Revision: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("rev {}", blk.revision),
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ Review Status: ", Style::default().fg(theme.text_muted)),
            Span::styled(&blk.review_status, status_style),
            Span::styled(" │ Last Modified: ", Style::default().fg(theme.text_muted)),
            Span::styled(&blk.last_modified, Style::default().fg(theme.text_dim)),
        ]);
        f.render_widget(Paragraph::new(meta_line1).block(meta_block), chunks[0]);
    }

    // 2. Main Editing Layout: Left List + Split Center/Right
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22),     // Blocks selector list
            Constraint::Percentage(39), // Japanese source
            Constraint::Percentage(39), // Vietnamese translation
        ])
        .split(chunks[1]);

    // Blocks list
    let list_border = if screen.active_pane == EditorPane::BlocksList {
        theme.panel_border_focused
    } else {
        theme.panel_border
    };
    let blk_list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(list_border))
        .title(" Blocks ")
        .style(Style::default().bg(theme.panel_bg));

    let blk_items: Vec<ListItem> = state
        .editor_blocks
        .iter()
        .enumerate()
        .map(|(idx, blk)| {
            let is_sel = idx == screen.selected_block_idx;
            let bg = if is_sel {
                theme.selection_bg
            } else {
                Color::Reset
            };
            let prefix = if is_sel { "▶ " } else { "  " };
            let mark = if blk.review_status.contains("Pending") {
                " ⚠"
            } else {
                " ✓"
            };
            let mark_color = if blk.review_status.contains("Pending") {
                theme.warning
            } else {
                theme.success
            };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.primary)),
                Span::styled(
                    format!("#{} ", blk.id),
                    Style::default().fg(if is_sel {
                        theme.selection_fg
                    } else {
                        theme.text
                    }),
                ),
                Span::styled(
                    format!("r{}", blk.revision),
                    Style::default().fg(theme.text_dim),
                ),
                Span::styled(mark, Style::default().fg(mark_color)),
            ]);
            ListItem::new(line).style(Style::default().bg(bg))
        })
        .collect();

    f.render_widget(List::new(blk_items).block(blk_list_block), main_chunks[0]);

    // Japanese Source Pane
    let jp_border = if screen.active_pane == EditorPane::JapaneseSource {
        theme.panel_border_focused
    } else {
        theme.panel_border
    };
    let jp_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(jp_border))
        .title(Span::styled(
            " 🇯🇵 JAPANESE ORIGINAL SOURCE ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let jp_text = current_block
        .map(|b| b.japanese.as_str())
        .unwrap_or("No block selected");
    f.render_widget(
        Paragraph::new(jp_text)
            .block(jp_block)
            .wrap(Wrap { trim: true }),
        main_chunks[1],
    );

    // Vietnamese Translation Pane
    let vi_border = if screen.active_pane == EditorPane::VietnameseTranslation {
        theme.panel_border_focused
    } else {
        theme.panel_border
    };
    let vi_title = if screen.is_editing {
        " 🇻🇳 VIETNAMESE TRANSLATION [EDITING - Enter/Ctrl+S to save, Esc to cancel] "
    } else {
        " 🇻🇳 VIETNAMESE TRANSLATION (Press [e] to edit text) "
    };
    let vi_title_color = if screen.is_editing {
        theme.warning
    } else {
        theme.success
    };

    let vi_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if screen.is_editing {
            theme.warning
        } else {
            vi_border
        }))
        .title(Span::styled(
            vi_title,
            Style::default()
                .fg(vi_title_color)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let vi_content = if screen.is_editing {
        format!("{}█", screen.edit_buffer)
    } else {
        current_block
            .map(|b| b.vietnamese.clone())
            .unwrap_or_else(|| "No translation".to_string())
    };

    f.render_widget(
        Paragraph::new(vi_content)
            .block(vi_block)
            .wrap(Wrap { trim: true }),
        main_chunks[2],
    );

    // 3. Editor Controls Helper
    let ctrl_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .style(Style::default().bg(theme.panel_bg));

    let ctrl_line = Line::from(vec![
        Span::styled(
            " [p] ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Prev Block  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            " [n] ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Next Block  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            " [Tab] ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Switch Pane  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            " [e] ",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Edit Translation  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            " [m] ",
            Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Reading Mode  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            " [h] ",
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Revision History  ", Style::default().fg(theme.text_dim)),
        Span::styled(
            " [r] ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Open in Reviewer", Style::default().fg(theme.text_dim)),
    ]);
    f.render_widget(Paragraph::new(ctrl_line).block(ctrl_block), chunks[2]);

    // History Modal overlay
    if screen.show_history {
        render_history_modal(f, current_block, theme);
    }
}

fn render_reading_mode(
    f: &mut Frame,
    area: Rect,
    screen: &EditorScreen,
    state: &AppState,
    theme: &Theme,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            " 📖 CONTINUOUS READING MODE — Vietnamese Translation (Press [m] to return to Split View) ",
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let mut lines = Vec::new();
    lines.push(Line::from(vec![Span::styled(
        "Chương 04: Huyết chiến săn mồi",
        Style::default()
            .fg(theme.warning)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )]));
    lines.push(Line::from(""));

    for blk in &state.editor_blocks {
        lines.push(Line::from(vec![
            Span::styled(
                format!("[#{}] ", blk.id),
                Style::default().fg(theme.primary_dim),
            ),
            Span::styled(&blk.vietnamese, Style::default().fg(theme.text)),
        ]));
        lines.push(Line::from(""));
    }

    let p = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((screen.reading_scroll, 0));
    f.render_widget(p, area);
}

fn render_history_modal(
    f: &mut Frame,
    block: Option<&crate::state::review::MockEditorBlock>,
    theme: &Theme,
) {
    let area = f.area();
    let width = 64.min(area.width.saturating_sub(4));
    let height = 12.min(area.height.saturating_sub(2));
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
            " 📜 REVISION HISTORY (Press [h] or [Esc] to close) ",
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.secondary))
        .style(Style::default().bg(Color::Rgb(15, 23, 42)));

    let inner = modal_block.inner(modal_rect);
    f.render_widget(modal_block, modal_rect);

    if let Some(blk) = block {
        let items: Vec<ListItem> = blk
            .history
            .iter()
            .map(|h| {
                ListItem::new(Line::from(vec![
                    Span::styled(" • ", Style::default().fg(theme.primary)),
                    Span::styled(h, Style::default().fg(theme.text)),
                ]))
            })
            .collect();
        f.render_widget(List::new(items), inner);
    }
}
