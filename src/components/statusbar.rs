use crate::state::AppState;
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn render_statusbar(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(35), // Left: Active stage & status
            Constraint::Min(25),    // Center: Current operation snippet
            Constraint::Length(40), // Right: Token stats & budget
        ])
        .split(area);

    // Left: Status indicator + Stage
    let (sim_text, sim_color) = if state.task.is_running {
        (" LIVE ", theme.success)
    } else if state.task.is_paused {
        (" PAUSED ", theme.warning)
    } else {
        (" IDLE ", theme.text_dim)
    };

    let left_spans = vec![
        Span::styled(
            sim_text,
            Style::default()
                .bg(sim_color)
                .fg(Color::Rgb(15, 23, 42))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!(
                "{} › {}",
                state.task.current_stage.label(),
                state.task.active_model.role.label()
            ),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
    ];
    f.render_widget(Paragraph::new(Line::from(left_spans)), chunks[0]);

    // Center: Current operation snippet
    let op_text = if state.task.current_operation.len() > 50 {
        format!("{}...", &state.task.current_operation[..47])
    } else {
        state.task.current_operation.clone()
    };
    let center_spans = vec![
        Span::styled("⚡ ", Style::default().fg(theme.primary)),
        Span::styled(op_text, Style::default().fg(theme.text_muted)),
    ];
    f.render_widget(Paragraph::new(Line::from(center_spans)), chunks[1]);

    // Right: Tokens, Cost, Budget
    let budget_str = if state.task.budget_unlimited {
        "Unlimited".to_string()
    } else if let Some(limit) = state.task.budget_limit_usd {
        format!("Cap: ${:.2}", limit)
    } else {
        "No Cap".to_string()
    };

    let right_spans = vec![
        Span::styled(
            format!("Tokens: {}k", state.task.session_tokens / 1000),
            Style::default().fg(theme.text_muted),
        ),
        Span::styled(" │ ", Style::default().fg(theme.panel_border)),
        Span::styled(
            format!("Cost: ${:.2}", state.task.session_cost_usd),
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(theme.panel_border)),
        Span::styled(budget_str, Style::default().fg(theme.primary_dim)),
    ];
    f.render_widget(
        Paragraph::new(Line::from(right_spans)).alignment(ratatui::layout::Alignment::Right),
        chunks[2],
    );
}
