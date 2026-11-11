use crate::state::AppState;
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenId {
    Dashboard = 0,
    Library = 1,
    Translation = 2,
    Editor = 3,
    Reviewer = 4,
    Context = 5,
    Skills = 6,
    Export = 7,
    Settings = 8,
    Logs = 9,
}

impl ScreenId {
    pub fn all() -> &'static [ScreenId] {
        &[
            ScreenId::Dashboard,
            ScreenId::Library,
            ScreenId::Translation,
            ScreenId::Editor,
            ScreenId::Reviewer,
            ScreenId::Context,
            ScreenId::Skills,
            ScreenId::Export,
            ScreenId::Settings,
            ScreenId::Logs,
        ]
    }

    pub fn number(&self) -> usize {
        *self as usize + 1
    }

    pub fn key_char(&self) -> char {
        match self {
            ScreenId::Dashboard => '1',
            ScreenId::Library => '2',
            ScreenId::Translation => '3',
            ScreenId::Editor => '4',
            ScreenId::Reviewer => '5',
            ScreenId::Context => '6',
            ScreenId::Skills => '7',
            ScreenId::Export => '8',
            ScreenId::Settings => '9',
            ScreenId::Logs => '0',
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ScreenId::Dashboard => "Dashboard",
            ScreenId::Library => "Library / Vols",
            ScreenId::Translation => "Translation",
            ScreenId::Editor => "Editor",
            ScreenId::Reviewer => "Reviewer",
            ScreenId::Context => "Context",
            ScreenId::Skills => "Skills",
            ScreenId::Export => "Export & Stage",
            ScreenId::Settings => "Settings",
            ScreenId::Logs => "Logs & Events",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ScreenId::Dashboard => "⊞",
            ScreenId::Library => "📚",
            ScreenId::Translation => "⚡",
            ScreenId::Editor => "✎",
            ScreenId::Reviewer => "⚖",
            ScreenId::Context => "❖",
            ScreenId::Skills => "⌘",
            ScreenId::Export => "⇪",
            ScreenId::Settings => "⚙",
            ScreenId::Logs => "☷",
        }
    }
}

pub fn render_sidebar(
    f: &mut Frame,
    area: Rect,
    current_screen: ScreenId,
    collapsed: bool,
    state: &AppState,
    theme: &Theme,
) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme.panel_border))
        .style(Style::default().bg(theme.sidebar_bg));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = ScreenId::all()
        .iter()
        .map(|screen| {
            let is_selected = *screen == current_screen;
            let key = screen.key_char();

            // Calculate badges
            let badge = match screen {
                ScreenId::Reviewer => {
                    let count = state.unresolved_issues_count();
                    if count > 0 {
                        Some((count.to_string(), theme.warning))
                    } else {
                        None
                    }
                }
                ScreenId::Skills => {
                    let count = state.pending_decisions_count();
                    if count > 0 {
                        Some((count.to_string(), theme.accent))
                    } else {
                        None
                    }
                }
                ScreenId::Translation => {
                    if state.task.is_running {
                        Some(("●".to_string(), theme.success))
                    } else if state.task.is_paused {
                        Some(("⏸".to_string(), theme.warning))
                    } else {
                        None
                    }
                }
                _ => None,
            };

            let mut spans = Vec::new();
            if collapsed {
                // Compact icon + number
                let style = if is_selected {
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text_dim)
                };
                spans.push(Span::styled(format!(" [{}]", key), style));
                if let Some((b_text, b_color)) = badge {
                    spans.push(Span::styled(
                        format!(" {}", b_text),
                        Style::default().fg(b_color),
                    ));
                }
            } else {
                // Full label
                let key_style = if is_selected {
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text_dim)
                };
                spans.push(Span::styled(format!(" [{}] ", key), key_style));

                let label_style = if is_selected {
                    Style::default()
                        .fg(theme.selection_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text_muted)
                };
                spans.push(Span::styled(screen.label(), label_style));

                if let Some((b_text, b_color)) = badge {
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        format!("[{}]", b_text),
                        Style::default().fg(b_color).add_modifier(Modifier::BOLD),
                    ));
                }
            }

            let item_style = if is_selected {
                Style::default().bg(theme.selection_bg)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(spans)).style(item_style)
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner_area);
}
