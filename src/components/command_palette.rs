use crate::components::sidebar::ScreenId;
use crate::state::DemoScenario;
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

#[derive(Debug, Clone)]
pub enum PaletteAction {
    SwitchScreen(ScreenId),
    SwitchScenario(DemoScenario),
    ToggleSimulation,
    ShowHelp,
}

#[derive(Debug, Clone)]
pub struct CommandItem {
    pub label: String,
    pub category: String,
    pub action: PaletteAction,
}

pub struct CommandPalette {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
    commands: Vec<CommandItem>,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        let commands = vec![
            CommandItem {
                label: "Go to: Dashboard (Workspace Overview)".to_string(),
                category: "Navigation".to_string(),
                action: PaletteAction::SwitchScreen(ScreenId::Dashboard),
            },
            CommandItem {
                label: "Go to: Library (Volumes & Chapters)".to_string(),
                category: "Navigation".to_string(),
                action: PaletteAction::SwitchScreen(ScreenId::Library),
            },
            CommandItem {
                label: "Go to: Translation (Workflow Runner)".to_string(),
                category: "Navigation".to_string(),
                action: PaletteAction::SwitchScreen(ScreenId::Translation),
            },
            CommandItem {
                label: "Go to: Translation Editor (Split View)".to_string(),
                category: "Navigation".to_string(),
                action: PaletteAction::SwitchScreen(ScreenId::Editor),
            },
            CommandItem {
                label: "Go to: Reviewer (S0-S3 Issues & Rubric)".to_string(),
                category: "Navigation".to_string(),
                action: PaletteAction::SwitchScreen(ScreenId::Reviewer),
            },
            CommandItem {
                label: "Go to: Context (Characters, Terms, Addressing)".to_string(),
                category: "Navigation".to_string(),
                action: PaletteAction::SwitchScreen(ScreenId::Context),
            },
            CommandItem {
                label: "Go to: Skills (Guidelines & Decisions)".to_string(),
                category: "Navigation".to_string(),
                action: PaletteAction::SwitchScreen(ScreenId::Skills),
            },
            CommandItem {
                label: "Go to: Export & Stage (EPUB Validation)".to_string(),
                category: "Navigation".to_string(),
                action: PaletteAction::SwitchScreen(ScreenId::Export),
            },
            CommandItem {
                label: "Go to: Settings (Models, Budget, Providers)".to_string(),
                category: "Navigation".to_string(),
                action: PaletteAction::SwitchScreen(ScreenId::Settings),
            },
            CommandItem {
                label: "Go to: Logs & Activity Events".to_string(),
                category: "Navigation".to_string(),
                action: PaletteAction::SwitchScreen(ScreenId::Logs),
            },
            CommandItem {
                label: "Workflow: Start / Pause Simulation".to_string(),
                category: "Action".to_string(),
                action: PaletteAction::ToggleSimulation,
            },
            CommandItem {
                label: "Scenario [F1]: Normal Translation".to_string(),
                category: "Demo".to_string(),
                action: PaletteAction::SwitchScenario(DemoScenario::NormalTranslation),
            },
            CommandItem {
                label: "Scenario [F2]: Review Issues Detected".to_string(),
                category: "Demo".to_string(),
                action: PaletteAction::SwitchScenario(DemoScenario::ReviewIssuesDetected),
            },
            CommandItem {
                label: "Scenario [F3]: Waiting for Terminology Approval".to_string(),
                category: "Demo".to_string(),
                action: PaletteAction::SwitchScenario(DemoScenario::WaitingForTerminologyApproval),
            },
            CommandItem {
                label: "Scenario [F4]: Model Request Failed (503)".to_string(),
                category: "Demo".to_string(),
                action: PaletteAction::SwitchScenario(DemoScenario::ModelRequestFailed),
            },
            CommandItem {
                label: "Scenario [F5]: Budget Limit Reached ($5.00)".to_string(),
                category: "Demo".to_string(),
                action: PaletteAction::SwitchScenario(DemoScenario::BudgetLimitReached),
            },
            CommandItem {
                label: "Scenario [F6]: Translation Completed (Ready for Export)".to_string(),
                category: "Demo".to_string(),
                action: PaletteAction::SwitchScenario(DemoScenario::TranslationCompleted),
            },
            CommandItem {
                label: "Scenario [F7]: Export Validation Failed (EPUBCheck error)".to_string(),
                category: "Demo".to_string(),
                action: PaletteAction::SwitchScenario(DemoScenario::ExportValidationFailed),
            },
            CommandItem {
                label: "Help: Show Keyboard Shortcuts Modal".to_string(),
                category: "Help".to_string(),
                action: PaletteAction::ShowHelp,
            },
        ];

        Self {
            is_open: false,
            query: String::new(),
            selected_index: 0,
            commands,
        }
    }

    pub fn filtered_commands(&self) -> Vec<&CommandItem> {
        let q = self.query.trim().to_lowercase();
        if q.is_empty() {
            self.commands.iter().collect()
        } else {
            self.commands
                .iter()
                .filter(|c| {
                    c.label.to_lowercase().contains(&q) || c.category.to_lowercase().contains(&q)
                })
                .collect()
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.query.clear();
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
    }

    pub fn next(&mut self) {
        let len = self.filtered_commands().len();
        if len > 0 {
            self.selected_index = (self.selected_index + 1) % len;
        }
    }

    pub fn prev(&mut self) {
        let len = self.filtered_commands().len();
        if len > 0 {
            if self.selected_index == 0 {
                self.selected_index = len - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn handle_char(&mut self, c: char) {
        self.query.push(c);
        self.selected_index = 0;
    }

    pub fn handle_backspace(&mut self) {
        self.query.pop();
        self.selected_index = 0;
    }
}

pub fn render_command_palette(f: &mut Frame, palette: &CommandPalette, theme: &Theme) {
    if !palette.is_open {
        return;
    }

    let area = f.area();
    // Centered overlay modal
    let width = (area.width * 3 / 5).clamp(55, 90);
    let height = 16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = area.height.saturating_sub(height) / 3;
    let modal_rect = Rect {
        x,
        y,
        width,
        height,
    };

    // Clear background behind modal
    f.render_widget(Clear, modal_rect);

    let block = Block::default()
        .title(Span::styled(
            " ⌘ COMMAND PALETTE (Type to filter, Esc to exit) ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(Color::Rgb(15, 23, 42)));

    let inner = block.inner(modal_rect);
    f.render_widget(block, modal_rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(4)])
        .split(inner);

    // Search bar input
    let search_box = Paragraph::new(Line::from(vec![
        Span::styled(
            " › ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            &palette.query,
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled("█", Style::default().fg(theme.primary)),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme.panel_border)),
    );
    f.render_widget(search_box, chunks[0]);

    // Filtered results list
    let filtered = palette.filtered_commands();
    if filtered.is_empty() {
        let no_match = Paragraph::new(Span::styled(
            "  No matching commands found.",
            Style::default()
                .fg(theme.text_dim)
                .add_modifier(Modifier::ITALIC),
        ));
        f.render_widget(no_match, chunks[1]);
        return;
    }

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let is_sel = idx == palette.selected_index;
            let bg = if is_sel {
                theme.selection_bg
            } else {
                Color::Reset
            };
            let cat_color = match item.category.as_str() {
                "Navigation" => theme.primary,
                "Demo" => theme.warning,
                "Action" => theme.success,
                _ => theme.secondary,
            };

            let line = Line::from(vec![
                Span::styled(
                    format!(" [{:^10}] ", item.category),
                    Style::default().fg(cat_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &item.label,
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

    let list = List::new(items);
    f.render_widget(list, chunks[1]);
}
