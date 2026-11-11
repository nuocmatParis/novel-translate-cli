use crate::state::AppState;
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
};

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum SettingsTab {
    #[default]
    Providers,
    Models,
    Budget,
    Appearance,
    Keybindings,
}

impl SettingsTab {
    pub fn all() -> &'static [SettingsTab] {
        &[
            SettingsTab::Providers,
            SettingsTab::Models,
            SettingsTab::Budget,
            SettingsTab::Appearance,
            SettingsTab::Keybindings,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SettingsTab::Providers => "1. AI Providers",
            SettingsTab::Models => "2. Models by Role",
            SettingsTab::Budget => "3. Budget & Limits",
            SettingsTab::Appearance => "4. Appearance",
            SettingsTab::Keybindings => "5. Keybindings",
        }
    }
}

pub struct SettingsScreen {
    pub active_tab: SettingsTab,
    pub selected_item: usize,
}

impl Default for SettingsScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self {
            active_tab: SettingsTab::Models,
            selected_item: 0,
        }
    }

    pub fn next_tab(&mut self) {
        let tabs = SettingsTab::all();
        let idx = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        self.active_tab = tabs[(idx + 1) % tabs.len()];
    }

    pub fn prev_tab(&mut self) {
        let tabs = SettingsTab::all();
        let idx = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        if idx == 0 {
            self.active_tab = tabs[tabs.len() - 1];
        } else {
            self.active_tab = tabs[idx - 1];
        }
    }

    pub fn toggle_budget(&mut self, state: &mut AppState) {
        state.task.budget_unlimited = !state.task.budget_unlimited;
        if state.task.budget_unlimited {
            state.task.budget_limit_usd = None;
        } else {
            state.task.budget_limit_usd = Some(10.00);
        }
    }
}

pub fn render_settings(
    f: &mut Frame,
    area: Rect,
    screen: &SettingsScreen,
    state: &AppState,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs Header
            Constraint::Min(10),   // Main Content
        ])
        .split(area);

    let tab_titles: Vec<Line> = SettingsTab::all()
        .iter()
        .map(|t| Line::from(t.label()))
        .collect();

    let tab_idx = SettingsTab::all()
        .iter()
        .position(|t| *t == screen.active_tab)
        .unwrap_or(0);

    let tabs_widget = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.panel_border))
                .title(Span::styled(
                    " ⚙ APPLICATION SETTINGS (Tab / [1-5] to switch tabs) ",
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

    match screen.active_tab {
        SettingsTab::Providers => render_providers_tab(f, chunks[1], theme),
        SettingsTab::Models => render_models_tab(f, chunks[1], state, theme),
        SettingsTab::Budget => render_budget_tab(f, chunks[1], state, theme),
        SettingsTab::Appearance => render_appearance_tab(f, chunks[1], theme),
        SettingsTab::Keybindings => render_keybindings_tab(f, chunks[1], theme),
    }
}

fn render_providers_tab(f: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" 🌐 AI Provider Configurations ")
        .style(Style::default().bg(theme.panel_bg));

    let items = vec![
        ListItem::new(
            " [x] Anthropic  — API Key: sk-ant-api03-****  — Status: Connected (Claude 3.5 Sonnet ready)",
        ),
        ListItem::new(
            " [x] OpenAI     — API Key: sk-proj-ab49****    — Status: Connected (GPT-4o & GPT-4o-mini ready)",
        ),
        ListItem::new(
            " [x] Google     — API Key: AIzaSyA7q****       — Status: Connected (Gemini 1.5 Pro ready)",
        ),
        ListItem::new(" [ ] DeepSeek   — API Key: (Not configured)    — Status: Disabled"),
    ];

    f.render_widget(List::new(items).block(block), area);
}

fn render_models_tab(f: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" 🤖 Model Assignment by Specialized Role ")
        .style(Style::default().bg(theme.panel_bg));

    let lines = vec![
        Line::from(vec![
            Span::styled(
                " 1. Translation Engine Model: ",
                Style::default().fg(theme.text_muted),
            ),
            Span::styled(
                "claude-3-5-sonnet-20241022",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" (Anthropic)", Style::default().fg(theme.text_dim)),
        ]),
        Line::from(vec![
            Span::styled("    Role: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                "Performs Japanese-to-Vietnamese translation batch with glossary prompts",
                Style::default().fg(theme.text),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " 2. Review & Quality Rubric Model: ",
                Style::default().fg(theme.text_muted),
            ),
            Span::styled(
                "gpt-4o",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" (OpenAI)", Style::default().fg(theme.text_dim)),
        ]),
        Line::from(vec![
            Span::styled("    Role: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                "Independent reviewer assessing translation against S0-S3 rubric & addressing rules",
                Style::default().fg(theme.text),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " 3. Terminology & Context Search Model: ",
                Style::default().fg(theme.text_muted),
            ),
            Span::styled(
                "gpt-4o-mini",
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" (OpenAI)", Style::default().fg(theme.text_dim)),
        ]),
        Line::from(vec![
            Span::styled("    Role: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                "Fast lexical analysis and entity extraction across volume skills",
                Style::default().fg(theme.text),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " * Mock configuration: models can be switched seamlessly in simulation.",
            Style::default().fg(theme.text_dim),
        )),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_budget_tab(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" 💰 Cost Management & Budget Caps (Press [b] to toggle Unlimited/Limit) ")
        .style(Style::default().bg(theme.panel_bg));

    let radio_limit = if !state.task.budget_unlimited {
        "(•)"
    } else {
        "( )"
    };
    let radio_unlimited = if state.task.budget_unlimited {
        "(•)"
    } else {
        "( )"
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", radio_limit),
                Style::default().fg(theme.primary),
            ),
            Span::styled(
                "Set Budget Limit Mode: ",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "$10.00 USD Maximum",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "     Warning threshold: ",
                Style::default().fg(theme.text_dim),
            ),
            Span::styled(
                "80% ($8.00) — Halts automatic batch dispatch upon limit",
                Style::default().fg(theme.text_muted),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!(" {} ", radio_unlimited),
                Style::default().fg(theme.primary),
            ),
            Span::styled(
                "Unlimited Cost Mode",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::styled(
            "     Runs continuously without pausing for expenditure caps",
            Style::default().fg(theme.text_muted),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Current Session Spend: ",
                Style::default().fg(theme.text_muted),
            ),
            Span::styled(
                format!("${:.3} USD", state.task.session_cost_usd),
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ Session Tokens: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{} tokens", state.task.session_tokens),
                Style::default().fg(theme.primary),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_appearance_tab(f: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" 🎨 Theme & Typography Tokens ")
        .style(Style::default().bg(theme.panel_bg));

    let items = vec![
        ListItem::new(
            " (•) Dark Navy (Default) — Deep slate-900 with sky-cyan accents & rich contrast",
        ),
        ListItem::new(" ( ) Slate Gray          — Neutral slate-800 with emerald-green accents"),
        ListItem::new(" ( ) Cyber Neon          — Deep black with vibrant magenta & neon cyan"),
        ListItem::new(" ( ) Monokai Classic     — Dark brown-gray with vibrant yellow and orange"),
    ];

    f.render_widget(List::new(items).block(block), area);
}

fn render_keybindings_tab(f: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.panel_border))
        .title(" ⌨ Global Keyboard Shortcuts Map ")
        .style(Style::default().bg(theme.panel_bg));

    let lines = vec![
        Line::from("  [1 - 0]        Jump directly to Screen 1..10"),
        Line::from("  [Tab / S-Tab]  Switch focus / panels"),
        Line::from("  [Ctrl+P / p]   Open Global Command Palette"),
        Line::from("  [F1 - F7]      Switch between 7 Demo Scenarios"),
        Line::from("  [b]            Toggle collapsible sidebar"),
        Line::from("  [s / p / r]    Start / Pause / Resume live translation simulation"),
        Line::from("  [?]            Open full help modal"),
        Line::from("  [q]            Quit application"),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);
}
