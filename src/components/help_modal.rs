use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn render_help_modal(f: &mut Frame, theme: &Theme) {
    let area = f.area();
    let width = 76.min(area.width.saturating_sub(4));
    let height = 24.min(area.height.saturating_sub(2));
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
            " ? KEYBOARD SHORTCUTS & NAVIGATION GUIDE ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(Color::Rgb(15, 23, 42)));

    let inner = block.inner(modal_rect);
    f.render_widget(block, modal_rect);

    let key_st = Style::default()
        .fg(theme.primary)
        .add_modifier(Modifier::BOLD);
    let desc_st = Style::default().fg(theme.text);
    let cat_st = Style::default()
        .fg(theme.warning)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

    let lines = vec![
        Line::from(Span::styled("Navigation & Screens", cat_st)),
        Line::from(vec![
            Span::styled("  1 - 9, 0     ", key_st),
            Span::styled(
                "Jump to Screen 1..10 (Dashboard, Library, Translation...)",
                desc_st,
            ),
        ]),
        Line::from(vec![
            Span::styled("  b            ", key_st),
            Span::styled("Toggle Sidebar collapse / expand", desc_st),
        ]),
        Line::from(vec![
            Span::styled("  Tab / S-Tab  ", key_st),
            Span::styled("Switch focus between panels / tabs", desc_st),
        ]),
        Line::from(vec![
            Span::styled("  ↑ / ↓, k / j ", key_st),
            Span::styled("Navigate lists and items", desc_st),
        ]),
        Line::from(vec![
            Span::styled("  Enter        ", key_st),
            Span::styled("Select / Confirm / Open details", desc_st),
        ]),
        Line::from(vec![
            Span::styled("  Esc          ", key_st),
            Span::styled("Close modal / cancel selection", desc_st),
        ]),
        Line::from(""),
        Line::from(Span::styled("Global Command & Demo Scenarios", cat_st)),
        Line::from(vec![
            Span::styled("  Ctrl+P / p   ", key_st),
            Span::styled("Open Global Command Palette (search actions)", desc_st),
        ]),
        Line::from(vec![
            Span::styled("  F1 - F7      ", key_st),
            Span::styled(
                "Switch Demo Scenarios (Normal, Issues, Failed, Budget...)",
                desc_st,
            ),
        ]),
        Line::from(vec![
            Span::styled("  s / p / r    ", key_st),
            Span::styled(
                "Start / Pause / Resume live translation simulation",
                desc_st,
            ),
        ]),
        Line::from(vec![
            Span::styled("  ?            ", key_st),
            Span::styled("Toggle this Help modal", desc_st),
        ]),
        Line::from(vec![
            Span::styled("  q            ", key_st),
            Span::styled("Quit application", desc_st),
        ]),
        Line::from(""),
        Line::from(Span::styled("Screen-Specific Controls", cat_st)),
        Line::from(vec![
            Span::styled("  Editor:      ", key_st),
            Span::styled(
                "[e] Edit translation in-memory, [Ctrl+S] Save revision, [m] Reading mode",
                desc_st,
            ),
        ]),
        Line::from(vec![
            Span::styled("  Reviewer:    ", key_st),
            Span::styled(
                "[f] Fix issue, [k] Keep exception, [a] Add user feedback",
                desc_st,
            ),
        ]),
        Line::from(vec![
            Span::styled("  Skills:      ", key_st),
            Span::styled(
                "[c] Confirm decision, [s] Suggest again, [l] Later",
                desc_st,
            ),
        ]),
        Line::from(vec![
            Span::styled("  Export:      ", key_st),
            Span::styled(
                "[5] Run validation simulation, [6] Export Final EPUB",
                desc_st,
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press [Esc] or [?] to close",
            Style::default().fg(theme.text_dim),
        )),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}
