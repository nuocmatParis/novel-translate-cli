use crate::state::AppState;
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
};

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum ContextTab {
    #[default]
    Characters,
    Terminology,
    Addressing,
    Relationships,
    Timeline,
}

impl ContextTab {
    pub fn all() -> &'static [ContextTab] {
        &[
            ContextTab::Characters,
            ContextTab::Terminology,
            ContextTab::Addressing,
            ContextTab::Relationships,
            ContextTab::Timeline,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ContextTab::Characters => "1. Characters",
            ContextTab::Terminology => "2. Terminology",
            ContextTab::Addressing => "3. Addressing",
            ContextTab::Relationships => "4. Relationships",
            ContextTab::Timeline => "5. Timeline",
        }
    }
}

pub struct ContextScreen {
    pub active_tab: ContextTab,
    pub selected_item: usize,
    pub term_search: String,
    pub is_searching: bool,
}

impl Default for ContextScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextScreen {
    pub fn new() -> Self {
        Self {
            active_tab: ContextTab::Terminology,
            selected_item: 0,
            term_search: String::new(),
            is_searching: false,
        }
    }

    pub fn next_tab(&mut self) {
        let tabs = ContextTab::all();
        let idx = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        self.active_tab = tabs[(idx + 1) % tabs.len()];
        self.selected_item = 0;
    }

    pub fn prev_tab(&mut self) {
        let tabs = ContextTab::all();
        let idx = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        if idx == 0 {
            self.active_tab = tabs[tabs.len() - 1];
        } else {
            self.active_tab = tabs[idx - 1];
        }
        self.selected_item = 0;
    }

    pub fn next_item(&mut self, state: &AppState) {
        let len = self.current_items_len(state);
        if len > 0 {
            self.selected_item = (self.selected_item + 1) % len;
        }
    }

    pub fn prev_item(&mut self, state: &AppState) {
        let len = self.current_items_len(state);
        if len > 0 {
            if self.selected_item == 0 {
                self.selected_item = len - 1;
            } else {
                self.selected_item -= 1;
            }
        }
    }

    fn current_items_len(&self, state: &AppState) -> usize {
        match self.active_tab {
            ContextTab::Characters => state.characters.len(),
            ContextTab::Terminology => state.terminology.len(),
            ContextTab::Addressing => state.addressing.len(),
            ContextTab::Relationships => state.relationships.len(),
            ContextTab::Timeline => state.timeline.len(),
        }
    }
}

pub fn render_context(
    f: &mut Frame,
    area: Rect,
    screen: &ContextScreen,
    state: &AppState,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs Header
            Constraint::Min(10),   // Main Content: List + Detail
        ])
        .split(area);

    // 1. Tabs Header
    let tab_titles: Vec<Line> = ContextTab::all()
        .iter()
        .map(|t| Line::from(t.label()))
        .collect();

    let tab_idx = ContextTab::all()
        .iter()
        .position(|t| *t == screen.active_tab)
        .unwrap_or(0);

    let tabs_widget = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.panel_border))
                .title(Span::styled(
                    " ❖ CONTEXT MANAGEMENT SYSTEM (Tab / [1-5] to switch tabs) ",
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(theme.panel_bg)),
        )
        .select(tab_idx)
        .style(Style::default().fg(theme.text_dim))
        .highlight_style(
            Style::default()
                .fg(theme.primary)
                .bg(Color::Rgb(30, 41, 59))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );
    f.render_widget(tabs_widget, chunks[0]);

    // 2. Tab Contents
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    match screen.active_tab {
        ContextTab::Terminology => render_terminology_tab(f, &body_chunks, screen, state, theme),
        ContextTab::Addressing => render_addressing_tab(f, &body_chunks, screen, state, theme),
        ContextTab::Characters => render_characters_tab(f, &body_chunks, screen, state, theme),
        ContextTab::Relationships => {
            render_relationships_tab(f, &body_chunks, screen, state, theme)
        }
        ContextTab::Timeline => render_timeline_tab(f, &body_chunks, screen, state, theme),
    }
}

fn render_terminology_tab(
    f: &mut Frame,
    chunks: &[Rect],
    screen: &ContextScreen,
    state: &AppState,
    theme: &Theme,
) {
    // Left: Searchable terms list
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border_focused))
        .title(" 📚 Terminology Glossary (Search / Select) ")
        .style(Style::default().bg(theme.panel_bg));

    let items: Vec<ListItem> = state
        .terminology
        .iter()
        .enumerate()
        .map(|(idx, term)| {
            let is_sel = idx == screen.selected_item;
            let bg = if is_sel {
                theme.selection_bg
            } else {
                Color::Reset
            };
            let prefix = if is_sel { "▶ " } else { "  " };

            let line1 = Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.primary)),
                Span::styled(
                    &term.source_term,
                    if is_sel {
                        Style::default()
                            .fg(theme.selection_fg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
            ]);
            let line2 = Line::from(vec![
                Span::raw("    → "),
                Span::styled(
                    &term.vietnamese_translation,
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);

            ListItem::new(vec![line1, line2]).style(Style::default().bg(bg))
        })
        .collect();

    f.render_widget(List::new(items).block(list_block), chunks[0]);

    // Right: Detail Panel
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" 🔍 Terminology Rule Details ")
        .style(Style::default().bg(theme.panel_bg));

    if let Some(term) = state.terminology.get(screen.selected_item) {
        let lines = vec![
            Line::from(vec![
                Span::styled(" Source Term: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    &term.source_term,
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    " Vietnamese Standard: ",
                    Style::default().fg(theme.text_muted),
                ),
                Span::styled(
                    &term.vietnamese_translation,
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Scope: ", Style::default().fg(theme.text_muted)),
                Span::styled(&term.scope, Style::default().fg(theme.warning)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Defined In: ", Style::default().fg(theme.text_muted)),
                Span::styled(&term.source_file, Style::default().fg(theme.primary_dim)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Usage Notes: ", Style::default().fg(theme.text_muted)),
                Span::styled(&term.notes, Style::default().fg(theme.text)),
            ]),
        ];
        f.render_widget(
            Paragraph::new(lines)
                .block(detail_block)
                .wrap(Wrap { trim: true }),
            chunks[1],
        );
    }
}

fn render_addressing_tab(
    f: &mut Frame,
    chunks: &[Rect],
    screen: &ContextScreen,
    state: &AppState,
    theme: &Theme,
) {
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border_focused))
        .title(" 🗣 Directed Addressing Pairs (Speaker → Listener) ")
        .style(Style::default().bg(theme.panel_bg));

    let items: Vec<ListItem> = state
        .addressing
        .iter()
        .enumerate()
        .map(|(idx, addr)| {
            let is_sel = idx == screen.selected_item;
            let bg = if is_sel {
                theme.selection_bg
            } else {
                Color::Reset
            };
            let prefix = if is_sel { "▶ " } else { "  " };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.primary)),
                Span::styled(
                    &addr.speaker,
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ➔ ", Style::default().fg(theme.text_dim)),
                Span::styled(&addr.listener, Style::default().fg(theme.text)),
                Span::styled(
                    format!("  [{}]", addr.scope),
                    Style::default().fg(theme.text_dim),
                ),
            ]);

            ListItem::new(line).style(Style::default().bg(bg))
        })
        .collect();

    f.render_widget(List::new(items).block(list_block), chunks[0]);

    // Right: Addressing Details
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" 🔍 Addressing Protocol & Context Scope ")
        .style(Style::default().bg(theme.panel_bg));

    if let Some(addr) = state.addressing.get(screen.selected_item) {
        let lines = vec![
            Line::from(vec![
                Span::styled(" Speaker: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    &addr.speaker,
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  ➔  Listener: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    &addr.listener,
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    " Self Reference (Xưng): ",
                    Style::default().fg(theme.text_muted),
                ),
                Span::styled(
                    &addr.self_reference,
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    " Listener Reference (Hô): ",
                    Style::default().fg(theme.text_muted),
                ),
                Span::styled(
                    &addr.listener_reference,
                    Style::default()
                        .fg(theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Applicable Scope: ", Style::default().fg(theme.text_muted)),
                Span::styled(&addr.scope, Style::default().fg(theme.secondary)),
            ]),
            Line::from(vec![
                Span::styled(" Context Nuance: ", Style::default().fg(theme.text_muted)),
                Span::styled(&addr.context, Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Rule Notes: ", Style::default().fg(theme.text_muted)),
                Span::styled(&addr.notes, Style::default().fg(theme.text)),
            ]),
        ];
        f.render_widget(
            Paragraph::new(lines)
                .block(detail_block)
                .wrap(Wrap { trim: true }),
            chunks[1],
        );
    }
}

fn render_characters_tab(
    f: &mut Frame,
    chunks: &[Rect],
    screen: &ContextScreen,
    state: &AppState,
    theme: &Theme,
) {
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border_focused))
        .title(" 👤 Characters Registry ")
        .style(Style::default().bg(theme.panel_bg));

    let items: Vec<ListItem> = state
        .characters
        .iter()
        .enumerate()
        .map(|(idx, ch)| {
            let is_sel = idx == screen.selected_item;
            let bg = if is_sel {
                theme.selection_bg
            } else {
                Color::Reset
            };
            let prefix = if is_sel { "▶ " } else { "  " };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.primary)),
                Span::styled(
                    &ch.name,
                    if is_sel {
                        Style::default()
                            .fg(theme.selection_fg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    },
                ),
                Span::styled(
                    format!(" ({})", ch.original_name),
                    Style::default().fg(theme.text_dim),
                ),
            ]);
            ListItem::new(line).style(Style::default().bg(bg))
        })
        .collect();

    f.render_widget(List::new(items).block(list_block), chunks[0]);

    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" 🔍 Character Profile ")
        .style(Style::default().bg(theme.panel_bg));

    if let Some(ch) = state.characters.get(screen.selected_item) {
        let lines = vec![
            Line::from(vec![
                Span::styled(" Name: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    &ch.name,
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" [{}]", ch.original_name),
                    Style::default().fg(theme.text_dim),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Alias / Titles: ", Style::default().fg(theme.text_muted)),
                Span::styled(&ch.alias, Style::default().fg(theme.warning)),
            ]),
            Line::from(vec![
                Span::styled(" Role: ", Style::default().fg(theme.text_muted)),
                Span::styled(&ch.role, Style::default().fg(theme.secondary)),
                Span::styled(" │ Gender: ", Style::default().fg(theme.text_muted)),
                Span::styled(&ch.gender, Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Profile / Lore: ", Style::default().fg(theme.text_muted)),
                Span::styled(&ch.description, Style::default().fg(theme.text)),
            ]),
        ];
        f.render_widget(
            Paragraph::new(lines)
                .block(detail_block)
                .wrap(Wrap { trim: true }),
            chunks[1],
        );
    }
}

fn render_relationships_tab(
    f: &mut Frame,
    chunks: &[Rect],
    screen: &ContextScreen,
    state: &AppState,
    theme: &Theme,
) {
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" 👥 Character Relationships ")
        .style(Style::default().bg(theme.panel_bg));

    let items: Vec<ListItem> = state
        .relationships
        .iter()
        .map(|r| {
            let line = Line::from(vec![
                Span::styled(
                    format!(" {} ↔ {}: ", r.from_char, r.to_char),
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&r.relation_type, Style::default().fg(theme.warning)),
            ]);
            ListItem::new(line)
        })
        .collect();

    f.render_widget(List::new(items).block(list_block), chunks[0]);

    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" 🔍 Relationship Dynamics ")
        .style(Style::default().bg(theme.panel_bg));

    if let Some(r) = state.relationships.get(screen.selected_item) {
        let lines = vec![
            Line::from(vec![
                Span::styled(" Dynamic: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    format!("{} and {}", r.from_char, r.to_char),
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Nature: ", Style::default().fg(theme.text_muted)),
                Span::styled(&r.relation_type, Style::default().fg(theme.warning)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Notes: ", Style::default().fg(theme.text_muted)),
                Span::styled(&r.notes, Style::default().fg(theme.text)),
            ]),
        ];
        f.render_widget(
            Paragraph::new(lines)
                .block(detail_block)
                .wrap(Wrap { trim: true }),
            chunks[1],
        );
    }
}

fn render_timeline_tab(
    f: &mut Frame,
    chunks: &[Rect],
    screen: &ContextScreen,
    state: &AppState,
    theme: &Theme,
) {
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" ⏳ Arc 7 Narrative Timeline ")
        .style(Style::default().bg(theme.panel_bg));

    let items: Vec<ListItem> = state
        .timeline
        .iter()
        .map(|t| {
            let line = Line::from(vec![
                Span::styled(
                    format!(" [{}] ", t.chapter),
                    Style::default().fg(theme.primary),
                ),
                Span::styled(
                    &t.point,
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    f.render_widget(List::new(items).block(list_block), chunks[0]);

    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" 🔍 Plot Checkpoint Summary ")
        .style(Style::default().bg(theme.panel_bg));

    if let Some(t) = state.timeline.get(screen.selected_item) {
        let lines = vec![
            Line::from(vec![
                Span::styled(" Chapter: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    &t.chapter,
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Checkpoint: ", Style::default().fg(theme.text_muted)),
                Span::styled(&t.point, Style::default().fg(theme.warning)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Summary: ", Style::default().fg(theme.text_muted)),
                Span::styled(&t.summary, Style::default().fg(theme.text)),
            ]),
        ];
        f.render_widget(
            Paragraph::new(lines)
                .block(detail_block)
                .wrap(Wrap { trim: true }),
            chunks[1],
        );
    }
}
