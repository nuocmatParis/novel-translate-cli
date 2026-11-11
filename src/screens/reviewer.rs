use crate::state::AppState;
use crate::state::logs::LogLevel;
use crate::state::review::Severity;
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

pub struct ReviewerScreen {
    pub selected_issue_idx: usize,
    pub show_feedback_dialog: bool,
    pub feedback_input: String,
}

impl Default for ReviewerScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ReviewerScreen {
    pub fn new() -> Self {
        Self {
            selected_issue_idx: 0,
            show_feedback_dialog: false,
            feedback_input: String::new(),
        }
    }

    pub fn next(&mut self, state: &AppState) {
        if !state.review_issues.is_empty() {
            self.selected_issue_idx = (self.selected_issue_idx + 1) % state.review_issues.len();
        }
    }

    pub fn prev(&mut self, state: &AppState) {
        if !state.review_issues.is_empty() {
            if self.selected_issue_idx == 0 {
                self.selected_issue_idx = state.review_issues.len() - 1;
            } else {
                self.selected_issue_idx -= 1;
            }
        }
    }

    pub fn fix_selected(&mut self, state: &mut AppState) {
        if self.selected_issue_idx >= state.review_issues.len() {
            return;
        }
        let (suggested, block_id, issue_id) = {
            let issue = &mut state.review_issues[self.selected_issue_idx];
            issue.resolved = true;
            (
                issue.suggested_correction.clone(),
                issue.block_id.trim_start_matches('#').to_string(),
                issue.id.clone(),
            )
        };

        // Update matching editor block
        if let Some(blk) = state.editor_blocks.iter_mut().find(|b| b.id == block_id) {
            blk.vietnamese = suggested;
            blk.revision += 1;
            blk.review_status = "Approved (Fixed via Reviewer)".to_string();
            blk.history.push(format!(
                "Rev {}: Áp dụng đề xuất từ {}",
                blk.revision, issue_id
            ));
        }

        state.add_log(
            LogLevel::Info,
            "Issue Resolved",
            &format!(
                "Applied suggested fix for {} on block #{}",
                issue_id, block_id
            ),
        );
    }

    pub fn keep_with_exception(&mut self, state: &mut AppState) {
        if self.selected_issue_idx >= state.review_issues.len() {
            return;
        }
        let issue_id = {
            let issue = &mut state.review_issues[self.selected_issue_idx];
            issue.resolved = true;
            issue.user_exception = true;
            issue.id.clone()
        };
        state.add_log(
            LogLevel::Warn,
            "Exception Granted",
            &format!(
                "User kept current translation with exception for {}",
                issue_id
            ),
        );
    }

    pub fn review_again(&mut self, state: &mut AppState) {
        if self.selected_issue_idx >= state.review_issues.len() {
            return;
        }
        let (cycles, issue_id) = {
            let issue = &mut state.review_issues[self.selected_issue_idx];
            if issue.cycles_count < issue.max_cycles {
                issue.cycles_count += 1;
            }
            (issue.cycles_count, issue.id.clone())
        };
        state.add_log(
            LogLevel::Info,
            "Re-review Dispatched",
            &format!("Dispatching review cycle {}/5 for {}", cycles, issue_id),
        );
    }

    pub fn open_feedback(&mut self) {
        self.show_feedback_dialog = true;
        self.feedback_input.clear();
    }

    pub fn close_feedback(&mut self) {
        self.show_feedback_dialog = false;
        self.feedback_input.clear();
    }

    pub fn submit_feedback(&mut self, state: &mut AppState) {
        if self.selected_issue_idx >= state.review_issues.len() {
            self.close_feedback();
            return;
        }
        let fb = if self.feedback_input.is_empty() {
            "Người dùng yêu cầu giữ văn phong nghiêm túc của hoàng đế Vollachia".to_string()
        } else {
            self.feedback_input.clone()
        };

        let issue_id = {
            let issue = &mut state.review_issues[self.selected_issue_idx];
            issue.feedback_notes.push(format!("User Feedback: {}", fb));
            issue.id.clone()
        };

        state.add_log(
            LogLevel::Info,
            "User Feedback Added",
            &format!(
                "Human review instruction attached to {}: '{}'",
                issue_id, fb
            ),
        );
        self.close_feedback();
    }
}

pub fn render_reviewer(
    f: &mut Frame,
    area: Rect,
    screen: &ReviewerScreen,
    state: &AppState,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(area);

    // Left: Issue List
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border_focused))
        .title(Span::styled(
            " ⚖ REVIEW ISSUES (↑/↓ to select) ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let items: Vec<ListItem> = state
        .review_issues
        .iter()
        .enumerate()
        .map(|(idx, issue)| {
            let is_sel = idx == screen.selected_issue_idx;
            let bg = if is_sel {
                theme.selection_bg
            } else {
                Color::Reset
            };
            let prefix = if is_sel { "▶ " } else { "  " };

            let sev_color = match issue.severity {
                Severity::S0Suggestion => theme.info,
                Severity::S1Minor => theme.text_muted,
                Severity::S2Major => theme.warning,
                Severity::S3Critical => theme.error,
            };

            let status_mark = if issue.resolved {
                if issue.user_exception {
                    " [EXCEPTION]"
                } else {
                    " [RESOLVED ✓]"
                }
            } else if issue.cycles_count >= issue.max_cycles {
                " [USER INTERVENTION!]"
            } else {
                " [PENDING]"
            };

            let mark_color = if issue.resolved {
                theme.success
            } else if issue.cycles_count >= issue.max_cycles {
                theme.error
            } else {
                theme.warning
            };

            let line1 = Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.primary)),
                Span::styled(
                    format!("{} ", issue.id),
                    if is_sel {
                        Style::default()
                            .fg(theme.selection_fg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                Span::styled(
                    format!("[{}] ", issue.severity.code()),
                    Style::default().fg(sev_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("[{}] ", issue.action_scope.code()),
                    Style::default()
                        .fg(theme.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("Block {}", issue.block_id),
                    Style::default().fg(theme.text_dim),
                ),
            ]);
            let line2 = Line::from(vec![
                Span::raw("    "),
                Span::styled(&issue.category, Style::default().fg(theme.text_muted)),
                Span::styled(
                    status_mark,
                    Style::default().fg(mark_color).add_modifier(Modifier::BOLD),
                ),
            ]);

            ListItem::new(vec![line1, line2]).style(Style::default().bg(bg))
        })
        .collect();

    f.render_widget(List::new(items).block(list_block), chunks[0]);

    // Right: Detailed Inspection Panel
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Core Classification: S0-S3 vs M1-M4 & Evidence
            Constraint::Length(8), // Japanese Source vs Vietnamese Translation
            Constraint::Min(8),    // Explanation, Suggested Fix & Feedback Notes
            Constraint::Length(3), // Action bar
        ])
        .split(chunks[1]);

    let selected_issue = state.review_issues.get(screen.selected_issue_idx);

    if let Some(issue) = selected_issue {
        // Section 1: Classification
        let sev_color = match issue.severity {
            Severity::S0Suggestion => theme.info,
            Severity::S1Minor => theme.text_muted,
            Severity::S2Major => theme.warning,
            Severity::S3Critical => theme.error,
        };

        let is_limit_exceeded = issue.cycles_count >= issue.max_cycles && !issue.resolved;

        let banner_title = if is_limit_exceeded {
            " ⚠ 5-ROUND CYCLE LIMIT EXCEEDED — USER INTERVENTION REQUIRED "
        } else {
            " 🔍 ISSUE CLASSIFICATION & RUBRIC MAPPING "
        };
        let banner_color = if is_limit_exceeded {
            theme.error
        } else {
            theme.primary
        };

        let class_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(banner_color))
            .title(Span::styled(
                banner_title,
                Style::default()
                    .fg(banner_color)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(theme.panel_bg));

        let class_lines = vec![
            Line::from(vec![
                Span::styled(" Issue ID: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    &issue.id,
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" │ Scope: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    format!(
                        "Vol. {} › {} › {}",
                        issue.volume_num, issue.chapter_title, issue.block_id
                    ),
                    Style::default().fg(theme.primary),
                ),
                Span::styled(" │ Cycles: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    format!("{}/{} rounds", issue.cycles_count, issue.max_cycles),
                    Style::default()
                        .fg(if is_limit_exceeded {
                            theme.error
                        } else {
                            theme.warning
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    " Severity (Ảnh hưởng): ",
                    Style::default().fg(theme.text_muted),
                ),
                Span::styled(
                    issue.severity.label(),
                    Style::default().fg(sev_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" — ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    issue.severity.description(),
                    Style::default().fg(theme.text_dim),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    " Action Scope (Xử lý): ",
                    Style::default().fg(theme.text_muted),
                ),
                Span::styled(
                    issue.action_scope.label(),
                    Style::default()
                        .fg(theme.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" — ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    issue.action_scope.description(),
                    Style::default().fg(theme.text_dim),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Evidence Status: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    issue.evidence_status.label(),
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];
        f.render_widget(
            Paragraph::new(class_lines).block(class_block),
            right_chunks[0],
        );

        // Section 2: Japanese Source & Vietnamese Translation
        let text_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.panel_border))
            .title(" 📄 TEXT COMPARISON ")
            .style(Style::default().bg(theme.panel_bg));

        let comp_lines = vec![
            Line::from(vec![
                Span::styled(
                    " JP Source: ",
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&issue.japanese_source, Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    " Current VI: ",
                    Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &issue.vietnamese_translation,
                    Style::default().fg(theme.text),
                ),
            ]),
        ];
        f.render_widget(
            Paragraph::new(comp_lines)
                .block(text_block)
                .wrap(Wrap { trim: true }),
            right_chunks[1],
        );

        // Section 3: Explanation & Suggested Correction
        let expl_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.panel_border))
            .title(" 💡 RUBRIC EXPLANATION & SUGGESTED CORRECTION ")
            .style(Style::default().bg(theme.panel_bg));

        let mut expl_lines = vec![
            Line::from(vec![
                Span::styled(
                    " Evaluation: ",
                    Style::default()
                        .fg(theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&issue.explanation, Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    " Suggested Fix: ",
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &issue.suggested_correction,
                    Style::default().fg(theme.success),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                " Cycle History & Feedback Notes:",
                Style::default().fg(theme.text_dim),
            )),
        ];
        for note in &issue.feedback_notes {
            expl_lines.push(Line::from(vec![
                Span::styled("  • ", Style::default().fg(theme.primary)),
                Span::styled(note, Style::default().fg(theme.text_muted)),
            ]));
        }

        f.render_widget(
            Paragraph::new(expl_lines)
                .block(expl_block)
                .wrap(Wrap { trim: true }),
            right_chunks[2],
        );

        // Section 4: Action Buttons
        let act_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.panel_border))
            .style(Style::default().bg(theme.panel_bg));

        let act_line = Line::from(vec![
            Span::styled(
                " [f] ",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Fix Issue  ", Style::default().fg(theme.text)),
            Span::styled(
                " [r] ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Review Again  ", Style::default().fg(theme.text)),
            Span::styled(
                " [k] ",
                Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Keep with Exception  ", Style::default().fg(theme.text)),
            Span::styled(
                " [a] ",
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Add Feedback  ", Style::default().fg(theme.text)),
            Span::styled(
                " [e] ",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Open in Editor", Style::default().fg(theme.text)),
        ]);
        f.render_widget(Paragraph::new(act_line).block(act_block), right_chunks[3]);
    }

    // Feedback Dialog Modal
    if screen.show_feedback_dialog {
        render_feedback_modal(f, &screen.feedback_input, theme);
    }
}

fn render_feedback_modal(f: &mut Frame, input: &str, theme: &Theme) {
    let area = f.area();
    let width = 68.min(area.width.saturating_sub(4));
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
            " 💬 ADD HUMAN FEEDBACK TO REVIEW MODEL (Enter to submit, Esc to cancel) ",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.warning))
        .style(Style::default().bg(Color::Rgb(15, 23, 42)));

    let inner = modal_block.inner(modal_rect);
    f.render_widget(modal_block, modal_rect);

    let content = vec![
        Line::from(Span::styled(
            "Provide custom prompt guidance for the Review & Translation engine:",
            Style::default().fg(theme.text_dim),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " > ",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if input.is_empty() {
                    "Keep Vincent's cold emperor persona; do not draw sword."
                } else {
                    input
                },
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(theme.primary)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to inject guidance into Review Model context cycle",
            Style::default().fg(theme.success),
        )),
    ];
    f.render_widget(Paragraph::new(content), inner);
}
