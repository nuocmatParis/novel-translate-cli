use crate::state::AppState;
use crate::state::logs::LogLevel;
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub struct SkillsScreen {
    pub selected_skill: usize,
    pub selected_decision: usize,
    pub selected_suggestion: usize,
    pub active_pane: SkillsPane,
    pub custom_input: String,
    pub is_entering_custom: bool,
}

#[derive(PartialEq, Eq)]
pub enum SkillsPane {
    SkillDocs,
    PendingDecisions,
}

impl Default for SkillsScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillsScreen {
    pub fn new() -> Self {
        Self {
            selected_skill: 1, // Default to Re:Zero series skills
            selected_decision: 0,
            selected_suggestion: 0,
            active_pane: SkillsPane::PendingDecisions,
            custom_input: String::new(),
            is_entering_custom: false,
        }
    }

    pub fn toggle_pane(&mut self) {
        if !self.is_entering_custom {
            self.active_pane = match self.active_pane {
                SkillsPane::SkillDocs => SkillsPane::PendingDecisions,
                SkillsPane::PendingDecisions => SkillsPane::SkillDocs,
            };
        }
    }

    pub fn next(&mut self, state: &AppState) {
        if self.is_entering_custom {
            return;
        }
        match self.active_pane {
            SkillsPane::SkillDocs => {
                if !state.skills.is_empty() {
                    self.selected_skill = (self.selected_skill + 1) % state.skills.len();
                }
            }
            SkillsPane::PendingDecisions => {
                if let Some(dec) = state.decisions.get(self.selected_decision) {
                    if !dec.suggestions.is_empty() {
                        self.selected_suggestion =
                            (self.selected_suggestion + 1) % dec.suggestions.len();
                    }
                }
            }
        }
    }

    pub fn prev(&mut self, state: &AppState) {
        if self.is_entering_custom {
            return;
        }
        match self.active_pane {
            SkillsPane::SkillDocs => {
                if !state.skills.is_empty() {
                    if self.selected_skill == 0 {
                        self.selected_skill = state.skills.len() - 1;
                    } else {
                        self.selected_skill -= 1;
                    }
                }
            }
            SkillsPane::PendingDecisions => {
                if let Some(dec) = state.decisions.get(self.selected_decision) {
                    if !dec.suggestions.is_empty() {
                        if self.selected_suggestion == 0 {
                            self.selected_suggestion = dec.suggestions.len() - 1;
                        } else {
                            self.selected_suggestion -= 1;
                        }
                    }
                }
            }
        }
    }

    pub fn next_decision_item(&mut self, state: &AppState) {
        if !state.decisions.is_empty() {
            self.selected_decision = (self.selected_decision + 1) % state.decisions.len();
            self.selected_suggestion = 0;
        }
    }

    pub fn confirm_decision(&mut self, state: &mut AppState) {
        if self.selected_decision >= state.decisions.len() {
            return;
        }
        let (term, chosen) = {
            let dec = &mut state.decisions[self.selected_decision];
            dec.is_confirmed = true;
            dec.selected_index = self.selected_suggestion;
            let chosen = dec
                .suggestions
                .get(self.selected_suggestion)
                .cloned()
                .unwrap_or_default();
            (dec.original_term.clone(), chosen)
        };
        state.add_log(
            LogLevel::Info,
            "Terminology Confirmed",
            &format!("User confirmed convention: {} -> '{}'", term, chosen),
        );
    }

    pub fn defer_decision(&mut self, state: &mut AppState) {
        if self.selected_decision >= state.decisions.len() {
            return;
        }
        let term = {
            let dec = &mut state.decisions[self.selected_decision];
            dec.is_deferred = true;
            dec.original_term.clone()
        };
        state.add_log(
            LogLevel::Warn,
            "Decision Deferred",
            &format!("Deferred decision for term: {}", term),
        );
    }
}

pub fn render_skills(
    f: &mut Frame,
    area: Rect,
    screen: &SkillsScreen,
    state: &AppState,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    // Left: Skill Documents & Markdown Preview
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(8)])
        .split(chunks[0]);

    // Skill doc list
    let doc_list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default().fg(if screen.active_pane == SkillsPane::SkillDocs {
                theme.panel_border_focused
            } else {
                theme.panel_border
            }),
        )
        .title(" ⌘ SKILL GUIDELINES & MARKDOWN REPOSITORY ")
        .style(Style::default().bg(theme.panel_bg));

    let doc_items: Vec<ListItem> = state
        .skills
        .iter()
        .enumerate()
        .map(|(idx, doc)| {
            let is_sel = idx == screen.selected_skill;
            let bg = if is_sel {
                theme.selection_bg
            } else {
                Color::Reset
            };
            let prefix = if is_sel { "▶ " } else { "  " };

            let line1 = Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.primary)),
                Span::styled(
                    &doc.filename,
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
                Span::raw("    "),
                Span::styled(&doc.category, Style::default().fg(theme.primary_dim)),
                Span::styled(
                    format!(" [{}]", doc.scope),
                    Style::default().fg(theme.text_dim),
                ),
            ]);

            ListItem::new(vec![line1, line2]).style(Style::default().bg(bg))
        })
        .collect();

    f.render_widget(List::new(doc_items).block(doc_list_block), left_chunks[0]);

    // Skill doc markdown preview
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" 📄 Markdown Read-Only Preview ")
        .style(Style::default().bg(theme.panel_bg));

    let preview_text = state
        .skills
        .get(screen.selected_skill)
        .map(|d| d.content.as_str())
        .unwrap_or("No skill selected");

    f.render_widget(
        Paragraph::new(preview_text)
            .block(preview_block)
            .wrap(Wrap { trim: true }),
        left_chunks[1],
    );

    // Right: Pending Decisions Approval Flow
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Decision Term Header & Reason
            Constraint::Min(8),    // Suggestions list & custom input
            Constraint::Length(4), // Actions: Confirm / Suggest Again / Later
        ])
        .split(chunks[1]);

    let dec_block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default().fg(if screen.active_pane == SkillsPane::PendingDecisions {
                theme.panel_border_focused
            } else {
                theme.panel_border
            }),
        )
        .title(Span::styled(
            " ⚖ PENDING TERMINOLOGY DECISIONS (Tab to switch pane) ",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel_bg));

    let current_dec = state.decisions.get(screen.selected_decision);

    if let Some(dec) = current_dec {
        // Section 1: Term details
        let term_lines = vec![
            Line::from(vec![
                Span::styled(" Candidate Term: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    &dec.original_term,
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" │ Scope: ", Style::default().fg(theme.text_muted)),
                Span::styled(&dec.scope, Style::default().fg(theme.secondary)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                " Linguistic Reasoning / Evidence: ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                &dec.reasoning,
                Style::default().fg(theme.text),
            )]),
        ];
        f.render_widget(
            Paragraph::new(term_lines)
                .block(dec_block)
                .wrap(Wrap { trim: true }),
            right_chunks[0],
        );

        // Section 2: Suggestions list
        let sug_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.panel_border))
            .title(" 💡 Candidate Suggestions (↑/↓ to select, separate [c] to confirm) ")
            .style(Style::default().bg(theme.panel_bg));

        let sug_items: Vec<ListItem> = dec
            .suggestions
            .iter()
            .enumerate()
            .map(|(s_idx, sug)| {
                let is_sel = s_idx == screen.selected_suggestion;
                let bg = if is_sel {
                    theme.selection_bg
                } else {
                    Color::Reset
                };
                let radio = if is_sel { "(•)" } else { "( )" };

                let line = Line::from(vec![
                    Span::styled(
                        format!(" {} ", radio),
                        Style::default().fg(if is_sel {
                            theme.primary
                        } else {
                            theme.text_dim
                        }),
                    ),
                    Span::styled(
                        sug,
                        if is_sel {
                            Style::default()
                                .fg(theme.selection_fg)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.text)
                        },
                    ),
                ]);
                ListItem::new(line).style(Style::default().bg(bg))
            })
            .collect();

        f.render_widget(List::new(sug_items).block(sug_block), right_chunks[1]);

        // Section 3: Action Buttons (Confirm / Suggest Again / Later)
        let act_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.panel_border))
            .style(Style::default().bg(theme.panel_bg));

        let status_badge = if dec.is_confirmed {
            Span::styled(
                " [CONFIRMED ✓] ",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            )
        } else if dec.is_deferred {
            Span::styled(" [DEFERRED ⏸] ", Style::default().fg(theme.text_dim))
        } else {
            Span::styled(
                " [PENDING APPROVAL] ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            )
        };

        let act_lines = vec![
            Line::from(vec![
                Span::styled(" Status: ", Style::default().fg(theme.text_dim)),
                status_badge,
                Span::styled(
                    " │ Switch Decision: [n] Next Item",
                    Style::default().fg(theme.text_dim),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    " [c] ",
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Confirm Selection  ", Style::default().fg(theme.text)),
                Span::styled(
                    " [s] ",
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Suggest Again (AI)  ", Style::default().fg(theme.text)),
                Span::styled(
                    " [l] ",
                    Style::default()
                        .fg(theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Later (Defer)", Style::default().fg(theme.text)),
            ]),
        ];
        f.render_widget(Paragraph::new(act_lines).block(act_block), right_chunks[2]);
    }
}
