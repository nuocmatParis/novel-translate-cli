use crate::state::AppState;
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

pub fn render_header(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(32), // Logo & App Name
            Constraint::Min(20),    // Book & Volume info
            Constraint::Length(36), // Scenario & Model status badge
        ])
        .split(area);

    // Left: Logo & Brand
    let brand_line = Line::from(vec![
        Span::styled(
            " ◆ ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "novel-translate",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled("-cli ", Style::default().fg(theme.primary)),
        Span::styled(
            "v0.1.0-preview",
            Style::default()
                .fg(theme.text_dim)
                .add_modifier(Modifier::ITALIC),
        ),
    ]);
    f.render_widget(Paragraph::new(brand_line), chunks[0]);

    // Center: Book & Chapter Breadcrumb
    let vol_num = state
        .workspace
        .series
        .volumes
        .get(state.workspace.active_volume_index)
        .map(|v| v.number)
        .unwrap_or(26);
    let breadcrumb_line = Line::from(vec![
        Span::styled("Re:Zero", Style::default().fg(theme.text_muted)),
        Span::styled(" › ", Style::default().fg(theme.text_dim)),
        Span::styled(
            format!("Vol. {:02}", vol_num),
            Style::default().fg(theme.primary),
        ),
        Span::styled(" › ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "Ch. 04 Huyết chiến săn mồi",
            Style::default().fg(theme.text),
        ),
    ]);
    f.render_widget(Paragraph::new(breadcrumb_line), chunks[1]);

    // Right: Scenario Badge & Model Indicator
    let scenario_badge = Span::styled(
        format!(
            " [{}] {} ",
            state.active_scenario.key(),
            state.active_scenario.title()
        ),
        Style::default()
            .bg(Color::Rgb(30, 41, 59))
            .fg(theme.warning)
            .add_modifier(Modifier::BOLD),
    );
    let model_badge = Span::styled(
        format!(" {} ", state.task.active_model.model_name),
        Style::default()
            .bg(theme.panel_bg)
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD),
    );
    let right_line = Line::from(vec![scenario_badge, Span::raw(" "), model_badge]);
    f.render_widget(
        Paragraph::new(right_line).alignment(ratatui::layout::Alignment::Right),
        chunks[2],
    );

    // Subtle bottom border line
    let border_rect = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    f.render_widget(
        Block::default().style(Style::default().bg(theme.panel_border)),
        border_rect,
    );
}
