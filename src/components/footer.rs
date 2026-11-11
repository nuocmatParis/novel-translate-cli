use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn render_footer(f: &mut Frame, area: Rect, theme: &Theme) {
    let key_style = Style::default()
        .bg(theme.panel_border)
        .fg(theme.primary)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(theme.text_dim);
    let sep = Span::styled("  ", desc_style);

    let spans = vec![
        Span::raw(" "),
        Span::styled(" [1-0] ", key_style),
        Span::styled(" Screens", desc_style),
        sep.clone(),
        Span::styled(" [Ctrl+P/p] ", key_style),
        Span::styled(" Command Palette", desc_style),
        sep.clone(),
        Span::styled(" [Tab] ", key_style),
        Span::styled(" Switch Pane", desc_style),
        sep.clone(),
        Span::styled(" [b] ", key_style),
        Span::styled(" Toggle Sidebar", desc_style),
        sep.clone(),
        Span::styled(" [F1-F7] ", key_style),
        Span::styled(" Demo Scenarios", desc_style),
        sep.clone(),
        Span::styled(" [?] ", key_style),
        Span::styled(" Help", desc_style),
        sep,
        Span::styled(" [q] ", key_style),
        Span::styled(" Quit", desc_style),
    ];

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
