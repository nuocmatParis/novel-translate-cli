use crate::state::AppState;
use crate::state::workflow::{StepState, WorkflowStage};
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

pub struct TranslationScreen {
    pub selected_step: usize,
    pub show_details: bool,
}

impl Default for TranslationScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationScreen {
    pub fn new() -> Self {
        Self {
            selected_step: 1,
            show_details: false,
        }
    }

    pub fn next_step(&mut self) {
        self.selected_step = (self.selected_step + 1) % 6;
    }

    pub fn prev_step(&mut self) {
        if self.selected_step == 0 {
            self.selected_step = 5;
        } else {
            self.selected_step -= 1;
        }
    }

    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }
}

pub fn render_translation(
    f: &mut Frame,
    area: Rect,
    screen: &TranslationScreen,
    state: &AppState,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Overall Progress & Mode
            Constraint::Length(9), // 6 Workflow Steps Pipeline
            Constraint::Length(6), // Active Model & Budget Card
            Constraint::Min(6),    // Activity Log & Controls
        ])
        .split(area);

    // 1. Overall Progress Header
    let progress_box = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(Span::styled(
            format!(
                " ⚡ WORKFLOW RUNNER — Mode: {} (Series: Re:Zero, Vol. 26, Ch. 04) ",
                state.task.mode.label()
            ),
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let prog_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(progress_box.inner(chunks[0]));

    f.render_widget(progress_box, chunks[0]);

    // Left gauge: Volume Progress
    let vol_gauge = Gauge::default()
        .block(Block::default().title("Volume 26 Overall Progress"))
        .gauge_style(
            Style::default()
                .fg(theme.primary)
                .bg(Color::Rgb(30, 41, 59)),
        )
        .percent(state.task.volume_progress_pct)
        .label(format!("{}%", state.task.volume_progress_pct));
    f.render_widget(vol_gauge, prog_chunks[0]);

    // Right gauge: Current Chapter Progress
    let ch_gauge = Gauge::default()
        .block(Block::default().title(format!(
            "Chapter 04 Progress (Batch {}/{})",
            state.task.current_batch, state.task.total_batches
        )))
        .gauge_style(
            Style::default()
                .fg(theme.warning)
                .bg(Color::Rgb(30, 41, 59)),
        )
        .percent(state.task.chapter_progress_pct)
        .label(format!("{}%", state.task.chapter_progress_pct));
    f.render_widget(ch_gauge, prog_chunks[1]);

    // 2. 6-Stage Workflow Pipeline
    let pipeline_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(Span::styled(
            " 🔄 PIPELINE STAGES (↑/↓ to inspect stage, [v] Details) ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let stages = WorkflowStage::all();
    let step_items: Vec<ListItem> = stages
        .iter()
        .enumerate()
        .map(|(idx, stage)| {
            let step_state = state.task.stage_states[idx];
            let is_sel = idx == screen.selected_step;
            let bg = if is_sel {
                theme.selection_bg
            } else {
                Color::Reset
            };

            let (icon, state_color, state_label) = match step_state {
                StepState::Completed => ("✓", theme.success, "Completed"),
                StepState::Running => ("▶", theme.primary, "RUNNING..."),
                StepState::Pending => ("○", theme.text_dim, "Pending"),
                StepState::Failed => ("✗", theme.error, "FAILED"),
                StepState::WaitingForUser => ("⏸", theme.warning, "Waiting for User"),
            };

            let prefix = if is_sel { "▶ " } else { "  " };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.primary)),
                Span::styled(
                    format!(" [{}] ", icon),
                    Style::default()
                        .fg(state_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    stage.label(),
                    if is_sel {
                        Style::default()
                            .fg(theme.selection_fg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                Span::styled(
                    format!("  [{}]", state_label),
                    Style::default()
                        .fg(state_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" — {}", stage.description()),
                    Style::default().fg(theme.text_dim),
                ),
            ]);

            ListItem::new(line).style(Style::default().bg(bg))
        })
        .collect();

    f.render_widget(List::new(step_items).block(pipeline_block), chunks[1]);

    // 3. Active Model Information & Budget
    let model_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(Span::styled(
            " 🤖 ACTIVE MODEL ENGINE & RESOURCE UTILIZATION ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let budget_info = if state.task.budget_unlimited {
        "Unlimited Cost Mode".to_string()
    } else if let Some(limit) = state.task.budget_limit_usd {
        format!("Cost Cap: ${:.2} (Alert at 80%)", limit)
    } else {
        "No limit configured".to_string()
    };

    let model_text = vec![
        Line::from(vec![
            Span::styled(" Provider: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                &state.task.active_model.provider,
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ Model: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                &state.task.active_model.model_name,
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ Role: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                state.task.active_model.role.label(),
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Operation: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                &state.task.current_operation,
                Style::default().fg(theme.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Session Tokens: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{} tokens", state.task.session_tokens),
                Style::default().fg(theme.text),
            ),
            Span::styled(" │ Est. Cost: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("${:.3}", state.task.session_cost_usd),
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ Budget Mode: ", Style::default().fg(theme.text_muted)),
            Span::styled(budget_info, Style::default().fg(theme.primary_dim)),
        ]),
    ];
    f.render_widget(Paragraph::new(model_text).block(model_block), chunks[2]);

    // 4. Activity Log & Control Buttons
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[3]);

    // Recent Activity Log
    let log_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(Span::styled(
            " 📋 LIVE ACTIVITY STREAM ",
            Style::default().fg(theme.primary),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let recent_logs: Vec<ListItem> = state
        .logs
        .iter()
        .rev()
        .take(5)
        .map(|entry| {
            let lvl_color = match entry.level {
                crate::state::logs::LogLevel::Info => theme.info,
                crate::state::logs::LogLevel::Warn => theme.warning,
                crate::state::logs::LogLevel::Error => theme.error,
                crate::state::logs::LogLevel::Debug => theme.text_dim,
            };
            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", entry.timestamp),
                    Style::default().fg(theme.text_dim),
                ),
                Span::styled(
                    format!("[{}] ", entry.level.label()),
                    Style::default().fg(lvl_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} — ", entry.event),
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
                Span::styled(&entry.message, Style::default().fg(theme.text_muted)),
            ]);
            ListItem::new(line)
        })
        .collect();

    f.render_widget(List::new(recent_logs).block(log_block), bottom_chunks[0]);

    // Live Controls Panel
    let ctrl_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(Span::styled(
            " 🎮 CONTROLS ",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let ctrl_text = vec![
        Line::from(vec![
            Span::styled(
                " [s] ",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Start Simulation", Style::default().fg(theme.text)),
        ]),
        Line::from(vec![
            Span::styled(
                " [p] ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Pause Simulation", Style::default().fg(theme.text)),
        ]),
        Line::from(vec![
            Span::styled(
                " [r] ",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Resume Simulation", Style::default().fg(theme.text)),
        ]),
        Line::from(vec![
            Span::styled(
                " [v] ",
                Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Toggle Stage Details", Style::default().fg(theme.text)),
        ]),
        Line::from(vec![
            Span::styled(
                " [l] ",
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("View Full Logs (Sc. 10)", Style::default().fg(theme.text)),
        ]),
    ];
    f.render_widget(
        Paragraph::new(ctrl_text).block(ctrl_block),
        bottom_chunks[1],
    );
}
