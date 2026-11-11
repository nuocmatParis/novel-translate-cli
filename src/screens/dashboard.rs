use crate::components::sidebar::ScreenId;
use crate::state::AppState;
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

#[derive(Default)]
pub struct DashboardScreen {
    pub selected_action: usize,
}

impl DashboardScreen {
    pub fn new() -> Self {
        Self { selected_action: 0 }
    }

    pub fn next_action(&mut self) {
        self.selected_action = (self.selected_action + 1) % 4;
    }

    pub fn prev_action(&mut self) {
        if self.selected_action == 0 {
            self.selected_action = 3;
        } else {
            self.selected_action -= 1;
        }
    }
}

pub fn render_dashboard(
    f: &mut Frame,
    area: Rect,
    screen_state: &DashboardScreen,
    state: &AppState,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Overview Header & metadata
            Constraint::Length(4), // Progress Gauge
            Constraint::Length(8), // 3 Metric Cards (Issues, Decisions, Task)
            Constraint::Min(6),    // Quick Actions & Recent Activity
        ])
        .split(area);

    // 1. Overview Header
    let vol = &state.workspace.series.volumes[state.workspace.active_volume_index];
    let header_lines = vec![
        Line::from(vec![
            Span::styled(
                " SERIES: ",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &state.workspace.series.title,
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" ({})", state.workspace.series.original_title),
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(" WORKSPACE: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                &state.workspace.name,
                Style::default().fg(theme.primary_dim),
            ),
            Span::styled(" │ PATH: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                &state.workspace.root_path,
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(" ACTIVE VOLUME: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                &vol.title,
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ IMPORTED VOLUMES: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{} volumes", state.workspace.series.volumes.len()),
                Style::default().fg(theme.text),
            ),
        ]),
    ];

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(Span::styled(
            " 📖 WORKSPACE OVERVIEW ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));
    f.render_widget(Paragraph::new(header_lines).block(header_block), chunks[0]);

    // 2. Volume Progress Gauge
    let gauge_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(Span::styled(
            format!(
                " 📊 TRANSLATION PROGRESS — Volume {:02} (4/6 Chapters Completed) ",
                vol.number
            ),
            Style::default().fg(theme.primary),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let gauge = Gauge::default()
        .block(gauge_block)
        .gauge_style(
            Style::default()
                .fg(theme.primary)
                .bg(Color::Rgb(30, 41, 59))
                .add_modifier(Modifier::BOLD),
        )
        .percent(state.task.volume_progress_pct)
        .label(format!("{}% Complete", state.task.volume_progress_pct));
    f.render_widget(gauge, chunks[1]);

    // 3. Three Metric Cards
    let card_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(chunks[2]);

    // Card 1: Review Issues
    let issues_count = state.unresolved_issues_count();
    let issue_color = if issues_count > 0 {
        theme.warning
    } else {
        theme.success
    };
    let issues_text = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", issues_count),
                Style::default()
                    .fg(issue_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Pending Review Issues", Style::default().fg(theme.text)),
        ]),
        Line::from(vec![Span::styled(
            " • S2 Major: 1   • S1 Minor: 1",
            Style::default().fg(theme.text_muted),
        )]),
        Line::from(vec![Span::styled(
            " • S3 Critical: 1 (5 cycles hit!)",
            Style::default().fg(theme.error),
        )]),
        Line::from(vec![Span::styled(
            " → Press [3] in actions to inspect",
            Style::default().fg(theme.text_dim),
        )]),
    ];
    let card1 = Paragraph::new(issues_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.panel_border))
            .title(Span::styled(
                " ⚖ REVIEW ISSUES ",
                Style::default()
                    .fg(issue_color)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(theme.panel_bg)),
    );
    f.render_widget(card1, card_chunks[0]);

    // Card 2: Terminology Decisions
    let dec_count = state.pending_decisions_count();
    let dec_color = if dec_count > 0 {
        theme.accent
    } else {
        theme.success
    };
    let dec_text = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", dec_count),
                Style::default().fg(dec_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Pending Term Decisions", Style::default().fg(theme.text)),
        ]),
        Line::from(vec![Span::styled(
            " • 「大罪司教」: Ty Giáo vs Giám Mục",
            Style::default().fg(theme.text_muted),
        )]),
        Line::from(vec![Span::styled(
            " • 「星名」: Star Names convention",
            Style::default().fg(theme.text_muted),
        )]),
        Line::from(vec![Span::styled(
            " → Open Skills [Screen 7] to decide",
            Style::default().fg(theme.text_dim),
        )]),
    ];
    let card2 = Paragraph::new(dec_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.panel_border))
            .title(Span::styled(
                " ⌘ TERMINOLOGY DECISIONS ",
                Style::default().fg(dec_color).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(theme.panel_bg)),
    );
    f.render_widget(card2, card_chunks[1]);

    // Card 3: Current Task & Engine
    let runner_color = if state.task.is_running {
        theme.success
    } else {
        theme.primary
    };
    let task_text = vec![
        Line::from(vec![
            Span::styled(" Active: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                "Translating Ch. 04 Batch 3/5",
                Style::default()
                    .fg(runner_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Model: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                &state.task.active_model.model_name,
                Style::default().fg(theme.primary),
            ),
            Span::styled(
                format!(" ({})", state.task.active_model.provider),
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Cost: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("${:.2} / $5.00 limit", state.task.session_cost_usd),
                Style::default().fg(theme.warning),
            ),
        ]),
        Line::from(vec![Span::styled(
            " → Live Simulation available",
            Style::default().fg(theme.text_dim),
        )]),
    ];
    let card3 = Paragraph::new(task_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.panel_border))
            .title(Span::styled(
                " ⚡ WORKFLOW ENGINE ",
                Style::default()
                    .fg(runner_color)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(theme.panel_bg)),
    );
    f.render_widget(card3, card_chunks[2]);

    // 4. Quick Actions
    let action_items = [
        (
            "Continue Translation",
            "Resume active translation workflow on Chapter 04",
            ScreenId::Translation,
        ),
        (
            "Open Volume / Chapters",
            "View chapter progress and select translation scopes",
            ScreenId::Library,
        ),
        (
            "Review Issues",
            "Inspect S0-S3 flagged issues and apply rubric corrections",
            ScreenId::Reviewer,
        ),
        (
            "Import EPUB",
            "Import new Japanese EPUB into current workspace",
            ScreenId::Dashboard,
        ),
    ];

    let items: Vec<ListItem> = action_items
        .iter()
        .enumerate()
        .map(|(idx, (name, desc, _))| {
            let is_sel = idx == screen_state.selected_action;
            let bg = if is_sel {
                theme.selection_bg
            } else {
                Color::Reset
            };
            let prefix = if is_sel { " ▶ " } else { "   " };
            let line = Line::from(vec![
                Span::styled(
                    prefix,
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("[{}] ", idx + 1),
                    Style::default().fg(theme.primary),
                ),
                Span::styled(
                    *name,
                    if is_sel {
                        Style::default()
                            .fg(theme.selection_fg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                Span::styled(
                    format!(" — {}", desc),
                    Style::default().fg(theme.text_muted),
                ),
            ]);
            ListItem::new(line).style(Style::default().bg(bg))
        })
        .collect();

    let actions_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(Span::styled(
            " ⚡ QUICK ACTIONS (↑/↓ to select, Enter to execute) ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    f.render_widget(List::new(items).block(actions_block), chunks[3]);
}
