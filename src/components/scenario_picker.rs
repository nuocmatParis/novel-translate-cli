use crate::state::{AppState, DemoScenario};
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
};

pub fn render_scenario_picker(f: &mut Frame, state: &AppState, selected_idx: usize, theme: &Theme) {
    let area = f.area();
    let width = 74.min(area.width.saturating_sub(4));
    let height = 18.min(area.height.saturating_sub(2));
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
            " 🧪 DEMO SCENARIO SELECTOR (Enter to activate, Esc to close) ",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.warning))
        .style(Style::default().bg(Color::Rgb(15, 23, 42)));

    let inner = block.inner(modal_rect);
    f.render_widget(block, modal_rect);

    let items: Vec<ListItem> = DemoScenario::all()
        .iter()
        .enumerate()
        .map(|(idx, scenario)| {
            let is_sel = idx == selected_idx;
            let is_active = *scenario == state.active_scenario;

            let mark = if is_active {
                " ● [ACTIVE]"
            } else {
                "          "
            };
            let mark_style = if is_active {
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_dim)
            };

            let title_style = if is_sel {
                Style::default()
                    .fg(theme.selection_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let desc_style = Style::default().fg(theme.text_dim);

            let bg = if is_sel {
                theme.selection_bg
            } else {
                Color::Reset
            };

            let line1 = Line::from(vec![
                Span::styled(
                    format!(" [{}] ", scenario.key()),
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(scenario.title(), title_style),
                Span::styled(mark, mark_style),
            ]);
            let line2 = Line::from(vec![
                Span::raw("        "),
                Span::styled(scenario.description(), desc_style),
            ]);

            ListItem::new(vec![line1, line2]).style(Style::default().bg(bg))
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner);
}
