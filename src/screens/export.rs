use crate::state::AppState;
use crate::state::logs::LogLevel;
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

pub struct ExportScreen {
    pub selected_action: usize,
    pub show_image_modal: bool,
    pub validation_feedback: Option<String>,
}

impl Default for ExportScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportScreen {
    pub fn new() -> Self {
        Self {
            selected_action: 0,
            show_image_modal: false,
            validation_feedback: None,
        }
    }

    pub fn next_action(&mut self) {
        self.selected_action = (self.selected_action + 1) % 6;
    }

    pub fn prev_action(&mut self) {
        if self.selected_action == 0 {
            self.selected_action = 5;
        } else {
            self.selected_action -= 1;
        }
    }

    pub fn toggle_image_modal(&mut self) {
        self.show_image_modal = !self.show_image_modal;
    }

    pub fn run_validation_simulation(&mut self, state: &mut AppState) {
        let has_issues = state.unresolved_issues_count() > 0;
        if has_issues {
            state.export.checklist[1].passed = false;
            state.export.checklist[1].details = format!(
                "Cảnh báo: Còn {} vấn đề Review chưa duyệt",
                state.unresolved_issues_count()
            );
            state.export.can_export = false;
            self.validation_feedback =
                Some("Validation Failed: Unresolved review issues present!".to_string());
        } else {
            for item in &mut state.export.checklist {
                item.passed = true;
            }
            state.export.can_export = true;
            state.export.errors.clear();
            self.validation_feedback = Some("Validation Succeeded: All checks passed!".to_string());
        }
        state.add_log(
            LogLevel::Info,
            "Validation Completed",
            "Executed full validation suite: XHTML, Manifest, and EPUBCheck",
        );
    }

    pub fn export_final_epub(&mut self, state: &mut AppState) {
        if state.export.can_export {
            state.add_log(
                LogLevel::Info,
                "EPUB Exported",
                &format!(
                    "Exported final production EPUB to: {}",
                    state.export.output_path
                ),
            );
            self.validation_feedback =
                Some(format!("SUCCESS: Generated {}", state.export.output_path));
        } else {
            self.validation_feedback = Some(
                "BLOCKED: Cannot export final EPUB until validation errors are cleared."
                    .to_string(),
            );
        }
    }
}

pub fn render_export(
    f: &mut Frame,
    area: Rect,
    screen: &ExportScreen,
    state: &AppState,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Validation Checklist Panel
            Constraint::Length(7), // Image Resources Panel
            Constraint::Min(7),    // Staging Editor Handoff & Actions
        ])
        .split(area);

    // 1. Validation Checklist Panel
    let check_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if state.export.can_export {
            theme.success
        } else {
            theme.error
        }))
        .title(Span::styled(
            " 🔍 PRE-EXPORT VALIDATION CHECKLIST ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let check_items: Vec<ListItem> = state
        .export
        .checklist
        .iter()
        .map(|item| {
            let (icon, color) = if item.passed {
                ("✓ PASS", theme.success)
            } else {
                ("✗ FAIL", theme.error)
            };
            let line = Line::from(vec![
                Span::styled(
                    format!(" [{}] ", icon),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &item.name,
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" — {}", item.details),
                    Style::default().fg(theme.text_muted),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    f.render_widget(List::new(check_items).block(check_block), chunks[0]);

    // 2. Image Resources Panel
    let img_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" 🖼 IMAGE RESOURCES & CLEANING INVENTORY ")
        .style(Style::default().bg(theme.panel_bg));

    let img = &state.export.images;
    let inv_color = if img.invalid > 0 {
        theme.error
    } else {
        theme.success
    };

    let img_lines = vec![
        Line::from(vec![
            Span::styled(" Total Images: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{}", img.total),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ Original: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{}", img.original),
                Style::default().fg(theme.primary),
            ),
            Span::styled(
                " │ Cleaned/Replaced: ",
                Style::default().fg(theme.text_muted),
            ),
            Span::styled(
                format!("{}", img.replaced),
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ Invalid: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{}", img.invalid),
                Style::default().fg(inv_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " │ Unused Edit Files: ",
                Style::default().fg(theme.text_muted),
            ),
            Span::styled(
                format!("{}", img.unused_edits),
                Style::default().fg(theme.warning),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Staging Area: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                &state.export.staging_path,
                Style::default().fg(theme.primary_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Edit Folder:  ", Style::default().fg(theme.text_dim)),
            Span::styled(
                &state.export.edit_folder_path,
                Style::default().fg(theme.primary_dim),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(img_lines).block(img_block), chunks[1]);

    // 3. Staging Handoff & Action Controls
    let action_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[2]);

    // Action list
    let actions_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(Span::styled(
            " ⚡ ACTIONS (↑/↓ to choose, Enter to execute) ",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let action_labels = [
        (
            "1. Open Edit Folder",
            "Open ./images/edit/ folder to place cleaned illustrations",
        ),
        (
            "2. View Image Manifest",
            "Inspect table of 18 images and replacement mappings",
        ),
        (
            "3. Preview EPUB in Viewer",
            "Open temporary EPUB bundle in external reader",
        ),
        (
            "4. Open Staging in Neovim",
            "Handoff terminal to Neovim for direct XML/OPF edits",
        ),
        (
            "5. Run Full Validation",
            "Re-run structure, resource, and EPUBCheck validators",
        ),
        (
            "6. Export Final EPUB",
            "Compile final clean Vietnamese EPUB to ./dist/",
        ),
    ];

    let items: Vec<ListItem> = action_labels
        .iter()
        .enumerate()
        .map(|(idx, (name, desc))| {
            let is_sel = idx == screen.selected_action;
            let bg = if is_sel {
                theme.selection_bg
            } else {
                Color::Reset
            };
            let prefix = if is_sel { " ▶ " } else { "   " };

            let name_color = if idx == 5 {
                if state.export.can_export {
                    theme.success
                } else {
                    theme.text_dim
                }
            } else {
                if is_sel {
                    theme.selection_fg
                } else {
                    theme.text
                }
            };

            let line = Line::from(vec![
                Span::styled(
                    prefix,
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    *name,
                    Style::default().fg(name_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" — {}", desc),
                    Style::default().fg(theme.text_muted),
                ),
            ]);
            ListItem::new(line).style(Style::default().bg(bg))
        })
        .collect();

    f.render_widget(List::new(items).block(actions_block), action_chunks[0]);

    // Right: Status & Feedback Banner
    let fb_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" 📋 Execution Status & Feedback ")
        .style(Style::default().bg(theme.panel_bg));

    let mut fb_lines = Vec::new();
    if let Some(msg) = &screen.validation_feedback {
        fb_lines.push(Line::from(vec![
            Span::styled(
                " Response: ",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(msg, Style::default().fg(theme.text)),
        ]));
        fb_lines.push(Line::from(""));
    }

    if !state.export.errors.is_empty() {
        fb_lines.push(Line::from(Span::styled(
            " Blocking Validation Errors:",
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        )));
        for err in &state.export.errors {
            fb_lines.push(Line::from(vec![
                Span::styled("  ✗ ", Style::default().fg(theme.error)),
                Span::styled(err, Style::default().fg(theme.text)),
            ]));
        }
    } else {
        fb_lines.push(Line::from(vec![
            Span::styled(" Status: ", Style::default().fg(theme.text_dim)),
            if state.export.can_export {
                Span::styled(
                    "Ready to Export",
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled("Validation Pending", Style::default().fg(theme.warning))
            },
        ]));
        fb_lines.push(Line::from(vec![
            Span::styled(" Output: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                &state.export.output_path,
                Style::default().fg(theme.primary_dim),
            ),
        ]));
    }

    f.render_widget(
        Paragraph::new(fb_lines)
            .block(fb_block)
            .wrap(Wrap { trim: true }),
        action_chunks[1],
    );

    // Image list modal
    if screen.show_image_modal {
        render_image_modal(f, theme);
    }
}

fn render_image_modal(f: &mut Frame, theme: &Theme) {
    let area = f.area();
    let width = 74.min(area.width.saturating_sub(4));
    let height = 16.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let modal_rect = Rect {
        x,
        y,
        width,
        height,
    };

    f.render_widget(Clear, modal_rect);

    let block = Block::default()
        .title(Span::styled(
            " 🖼 IMAGE MANIFEST & REPLACEMENTS (Esc to close) ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(Color::Rgb(15, 23, 42)));

    let inner = block.inner(modal_rect);
    f.render_widget(block, modal_rect);

    let image_items = vec![
        ListItem::new(" • cover.jpg — [Original 100%] — Bìa chính (Shinichirou Otsuka)"),
        ListItem::new(" • ill_001.jpg — [Replaced ✓] — Nhân vật giới thiệu (Đã xóa text Nhật)"),
        ListItem::new(" • ill_002.jpg — [Replaced ✓] — Bản đồ đế quốc Vollachia"),
        ListItem::new(" • ill_003.jpg — [Original 100%] — Subaru và Rem trong rừng"),
        ListItem::new(" • ill_004.jpg — [Replaced ✓] — Vincent rút kiếm (Đã chèn thoại Việt)"),
        ListItem::new(" • ill_005.jpg — [Replaced ✓] — Huyết chiến săn mồi tộc Shudraq"),
    ];

    f.render_widget(List::new(image_items), inner);
}
