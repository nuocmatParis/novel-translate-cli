use crate::components::command_palette::{CommandPalette, PaletteAction};
use crate::components::footer::render_footer;
use crate::components::header::render_header;
use crate::components::help_modal::render_help_modal;
use crate::components::scenario_picker::render_scenario_picker;
use crate::components::sidebar::{ScreenId, render_sidebar};
use crate::components::statusbar::render_statusbar;
use crate::screens::*;
use crate::simulation::SimulationEngine;
use crate::state::{AppState, DemoScenario};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub struct App {
    pub current_screen: ScreenId,
    pub state: AppState,
    pub theme: Theme,
    pub sidebar_collapsed: bool,
    pub should_quit: bool,

    // Modals
    pub command_palette: CommandPalette,
    pub show_help: bool,
    pub show_scenario_picker: bool,
    pub scenario_picker_idx: usize,

    // Temporary Notification Banner
    pub notification: Option<(String, std::time::Instant)>,

    // Screen states
    pub dashboard_screen: DashboardScreen,
    pub library_screen: LibraryScreen,
    pub translation_screen: TranslationScreen,
    pub editor_screen: EditorScreen,
    pub reviewer_screen: ReviewerScreen,
    pub context_screen: ContextScreen,
    pub skills_screen: SkillsScreen,
    pub export_screen: ExportScreen,
    pub settings_screen: SettingsScreen,
    pub logs_screen: LogsScreen,

    // Simulation Engine
    pub simulation: SimulationEngine,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            current_screen: ScreenId::Dashboard,
            state: AppState::new(),
            theme: Theme::default(),
            sidebar_collapsed: false,
            should_quit: false,

            command_palette: CommandPalette::new(),
            show_help: false,
            show_scenario_picker: false,
            scenario_picker_idx: 0,

            notification: None,

            dashboard_screen: DashboardScreen::new(),
            library_screen: LibraryScreen::new(),
            translation_screen: TranslationScreen::new(),
            editor_screen: EditorScreen::new(),
            reviewer_screen: ReviewerScreen::new(),
            context_screen: ContextScreen::new(),
            skills_screen: SkillsScreen::new(),
            export_screen: ExportScreen::new(),
            settings_screen: SettingsScreen::new(),
            logs_screen: LogsScreen::new(),

            simulation: SimulationEngine::new(),
        }
    }

    pub fn set_notification(&mut self, message: &str) {
        self.notification = Some((message.to_string(), std::time::Instant::now()));
    }

    pub fn switch_screen(&mut self, screen: ScreenId) {
        self.current_screen = screen;
    }

    pub fn switch_scenario(&mut self, scenario: DemoScenario) {
        self.state.apply_scenario(scenario);
        self.set_notification(&format!("Activated Scenario: {}", scenario.title()));
    }

    pub fn update(&mut self) {
        // Clear expired notification after 4s
        if let Some((_, time)) = self.notification {
            if time.elapsed().as_secs() > 4 {
                self.notification = None;
            }
        }

        // Run simulation state machine update
        self.simulation.update(&mut self.state);
    }

    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        // Responsive boundary guard for small terminals
        if area.width < 60 || area.height < 15 {
            render_too_small(f, area, &self.theme);
            return;
        }

        // Auto collapse sidebar if terminal width is compact (< 95)
        let is_compact = area.width < 95;
        let effective_sidebar_collapsed = self.sidebar_collapsed || is_compact;

        // Global Layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Min(10),   // Content (Sidebar + Active Screen)
                Constraint::Length(1), // Status Bar
                Constraint::Length(1), // Footer Keybindings
            ])
            .split(area);

        // 1. Header
        render_header(f, main_chunks[0], &self.state, &self.theme);

        // 2. Middle Content: Sidebar + Screen
        let sidebar_width = if effective_sidebar_collapsed { 8 } else { 20 };
        let middle_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(sidebar_width), Constraint::Min(40)])
            .split(main_chunks[1]);

        render_sidebar(
            f,
            middle_chunks[0],
            self.current_screen,
            effective_sidebar_collapsed,
            &self.state,
            &self.theme,
        );

        let screen_area = middle_chunks[1];
        match self.current_screen {
            ScreenId::Dashboard => {
                render_dashboard(
                    f,
                    screen_area,
                    &self.dashboard_screen,
                    &self.state,
                    &self.theme,
                );
            }
            ScreenId::Library => {
                render_library(
                    f,
                    screen_area,
                    &self.library_screen,
                    &self.state,
                    &self.theme,
                );
            }
            ScreenId::Translation => {
                render_translation(
                    f,
                    screen_area,
                    &self.translation_screen,
                    &self.state,
                    &self.theme,
                );
            }
            ScreenId::Editor => {
                render_editor(
                    f,
                    screen_area,
                    &self.editor_screen,
                    &self.state,
                    &self.theme,
                );
            }
            ScreenId::Reviewer => {
                render_reviewer(
                    f,
                    screen_area,
                    &self.reviewer_screen,
                    &self.state,
                    &self.theme,
                );
            }
            ScreenId::Context => {
                render_context(
                    f,
                    screen_area,
                    &self.context_screen,
                    &self.state,
                    &self.theme,
                );
            }
            ScreenId::Skills => {
                render_skills(
                    f,
                    screen_area,
                    &self.skills_screen,
                    &self.state,
                    &self.theme,
                );
            }
            ScreenId::Export => {
                render_export(
                    f,
                    screen_area,
                    &self.export_screen,
                    &self.state,
                    &self.theme,
                );
            }
            ScreenId::Settings => {
                render_settings(
                    f,
                    screen_area,
                    &self.settings_screen,
                    &self.state,
                    &self.theme,
                );
            }
            ScreenId::Logs => {
                render_logs(f, screen_area, &self.logs_screen, &self.state, &self.theme);
            }
        }

        // Notification Banner overlay if active
        if let Some((msg, _)) = &self.notification {
            render_notification_toast(f, msg, &self.theme);
        }

        // 3. Status Bar
        render_statusbar(f, main_chunks[2], &self.state, &self.theme);

        // 4. Footer
        render_footer(f, main_chunks[3], &self.theme);

        // Overlays
        if self.command_palette.is_open {
            crate::components::command_palette::render_command_palette(
                f,
                &self.command_palette,
                &self.theme,
            );
        } else if self.show_help {
            render_help_modal(f, &self.theme);
        } else if self.show_scenario_picker {
            render_scenario_picker(f, &self.state, self.scenario_picker_idx, &self.theme);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // 1. If Command Palette is open
        if self.command_palette.is_open {
            match key.code {
                KeyCode::Esc => self.command_palette.close(),
                KeyCode::Up => self.command_palette.prev(),
                KeyCode::Down => self.command_palette.next(),
                KeyCode::Backspace => self.command_palette.handle_backspace(),
                KeyCode::Char(c) => self.command_palette.handle_char(c),
                KeyCode::Enter => {
                    let filtered = self.command_palette.filtered_commands();
                    if let Some(cmd) = filtered.get(self.command_palette.selected_index) {
                        let action = cmd.action.clone();
                        self.command_palette.close();
                        match action {
                            PaletteAction::SwitchScreen(s) => self.switch_screen(s),
                            PaletteAction::SwitchScenario(sc) => self.switch_scenario(sc),
                            PaletteAction::ToggleSimulation => {
                                self.state.task.is_running = !self.state.task.is_running;
                                self.set_notification(if self.state.task.is_running {
                                    "Simulation Started"
                                } else {
                                    "Simulation Paused"
                                });
                            }
                            PaletteAction::ShowHelp => self.show_help = true,
                        }
                    }
                }
                _ => {}
            }
            return;
        }

        // 2. If Help modal is open
        if self.show_help {
            if matches!(
                key.code,
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')
            ) {
                self.show_help = false;
            }
            return;
        }

        // 3. If Scenario Picker is open
        if self.show_scenario_picker {
            match key.code {
                KeyCode::Esc => self.show_scenario_picker = false,
                KeyCode::Up => {
                    if self.scenario_picker_idx == 0 {
                        self.scenario_picker_idx = DemoScenario::all().len() - 1;
                    } else {
                        self.scenario_picker_idx -= 1;
                    }
                }
                KeyCode::Down => {
                    self.scenario_picker_idx =
                        (self.scenario_picker_idx + 1) % DemoScenario::all().len();
                }
                KeyCode::Enter => {
                    let sc = DemoScenario::all()[self.scenario_picker_idx];
                    self.switch_scenario(sc);
                    self.show_scenario_picker = false;
                }
                _ => {}
            }
            return;
        }

        // 4. Editor text editing interception (User is typing in memory!)
        if self.current_screen == ScreenId::Editor && self.editor_screen.is_editing {
            match key.code {
                KeyCode::Esc => self.editor_screen.cancel_editing(),
                KeyCode::Enter => {
                    self.editor_screen.save_revision(&mut self.state);
                    self.set_notification("Saved revision to block in memory");
                }
                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.editor_screen.save_revision(&mut self.state);
                    self.set_notification("Saved revision to block in memory");
                }
                KeyCode::Backspace => self.editor_screen.handle_edit_backspace(),
                KeyCode::Char(c) => self.editor_screen.handle_edit_char(c),
                _ => {}
            }
            return;
        }

        // 5. Reviewer feedback dialog interception
        if self.current_screen == ScreenId::Reviewer && self.reviewer_screen.show_feedback_dialog {
            match key.code {
                KeyCode::Esc => self.reviewer_screen.close_feedback(),
                KeyCode::Enter => {
                    self.reviewer_screen.submit_feedback(&mut self.state);
                    self.set_notification("Feedback submitted to Review Model");
                }
                KeyCode::Backspace => {
                    self.reviewer_screen.feedback_input.pop();
                }
                KeyCode::Char(c) => {
                    self.reviewer_screen.feedback_input.push(c);
                }
                _ => {}
            }
            return;
        }

        // 6. Global Command Palette triggers: Ctrl+P or 'p' (when not editing)
        if (key.code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::CONTROL))
            || (key.code == KeyCode::Char('p') && self.current_screen != ScreenId::Editor)
        {
            self.command_palette.open();
            return;
        }

        // 7. Global Scenario keys (F1 - F7)
        match key.code {
            KeyCode::F(1) => {
                self.switch_scenario(DemoScenario::NormalTranslation);
                return;
            }
            KeyCode::F(2) => {
                self.switch_scenario(DemoScenario::ReviewIssuesDetected);
                return;
            }
            KeyCode::F(3) => {
                self.switch_scenario(DemoScenario::WaitingForTerminologyApproval);
                return;
            }
            KeyCode::F(4) => {
                self.switch_scenario(DemoScenario::ModelRequestFailed);
                return;
            }
            KeyCode::F(5) => {
                self.switch_scenario(DemoScenario::BudgetLimitReached);
                return;
            }
            KeyCode::F(6) => {
                self.switch_scenario(DemoScenario::TranslationCompleted);
                return;
            }
            KeyCode::F(7) => {
                self.switch_scenario(DemoScenario::ExportValidationFailed);
                return;
            }
            _ => {}
        }

        // 8. Global Screen switching [1-0]
        match key.code {
            KeyCode::Char('1') => {
                self.switch_screen(ScreenId::Dashboard);
                return;
            }
            KeyCode::Char('2') => {
                self.switch_screen(ScreenId::Library);
                return;
            }
            KeyCode::Char('3') => {
                self.switch_screen(ScreenId::Translation);
                return;
            }
            KeyCode::Char('4') => {
                self.switch_screen(ScreenId::Editor);
                return;
            }
            KeyCode::Char('5') => {
                self.switch_screen(ScreenId::Reviewer);
                return;
            }
            KeyCode::Char('6') => {
                self.switch_screen(ScreenId::Context);
                return;
            }
            KeyCode::Char('7') => {
                self.switch_screen(ScreenId::Skills);
                return;
            }
            KeyCode::Char('8') => {
                self.switch_screen(ScreenId::Export);
                return;
            }
            KeyCode::Char('9') => {
                self.switch_screen(ScreenId::Settings);
                return;
            }
            KeyCode::Char('0') => {
                self.switch_screen(ScreenId::Logs);
                return;
            }
            _ => {}
        }

        // 9. Global controls
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
                return;
            }
            KeyCode::Char('b') => {
                self.sidebar_collapsed = !self.sidebar_collapsed;
                return;
            }
            _ => {}
        }

        // 10. Screen-specific controls
        match self.current_screen {
            ScreenId::Dashboard => match key.code {
                KeyCode::Up => self.dashboard_screen.prev_action(),
                KeyCode::Down => self.dashboard_screen.next_action(),
                KeyCode::Enter => match self.dashboard_screen.selected_action {
                    0 => self.switch_screen(ScreenId::Translation),
                    1 => self.switch_screen(ScreenId::Library),
                    2 => self.switch_screen(ScreenId::Reviewer),
                    3 => self.set_notification("Mock Import: Open EPUB file dialog simulated"),
                    _ => {}
                },
                _ => {}
            },
            ScreenId::Library => match key.code {
                KeyCode::Tab => self.library_screen.toggle_focus(),
                KeyCode::Up => self.library_screen.prev(&self.state),
                KeyCode::Down => self.library_screen.next(&self.state),
                KeyCode::Char(' ') => self
                    .library_screen
                    .toggle_chapter_selection(&mut self.state),
                KeyCode::Char('t') => {
                    self.library_screen
                        .start_selected_translation(&mut self.state);
                    self.switch_screen(ScreenId::Translation);
                    self.set_notification("Started translation on selected chapter scope");
                }
                KeyCode::Char('a') => {
                    self.library_screen
                        .start_auto_translate_all(&mut self.state);
                    self.switch_screen(ScreenId::Translation);
                    self.set_notification("Started sequential auto-translation to end of EPUB");
                }
                _ => {}
            },
            ScreenId::Translation => match key.code {
                KeyCode::Char('s') => {
                    self.state.task.is_running = true;
                    self.state.task.is_paused = false;
                    self.set_notification("Started translation workflow simulation");
                }
                KeyCode::Char('p') => {
                    self.state.task.is_paused = true;
                    self.set_notification("Paused translation workflow simulation");
                }
                KeyCode::Char('r') => {
                    self.state.task.is_running = true;
                    self.state.task.is_paused = false;
                    self.set_notification("Resumed translation workflow simulation");
                }
                KeyCode::Char('v') => self.translation_screen.toggle_details(),
                KeyCode::Char('l') => self.switch_screen(ScreenId::Logs),
                KeyCode::Up => self.translation_screen.prev_step(),
                KeyCode::Down => self.translation_screen.next_step(),
                _ => {}
            },
            ScreenId::Editor => match key.code {
                KeyCode::Tab => self.editor_screen.toggle_pane(),
                KeyCode::Up | KeyCode::Char('p') => self.editor_screen.prev_block(&self.state),
                KeyCode::Down | KeyCode::Char('n') => self.editor_screen.next_block(&self.state),
                KeyCode::Char('e') => self.editor_screen.start_editing(&self.state),
                KeyCode::Char('m') => self.editor_screen.toggle_reading_mode(),
                KeyCode::Char('h') => self.editor_screen.toggle_history(),
                KeyCode::Char('r') => self.switch_screen(ScreenId::Reviewer),
                _ => {}
            },
            ScreenId::Reviewer => match key.code {
                KeyCode::Up => self.reviewer_screen.prev(&self.state),
                KeyCode::Down => self.reviewer_screen.next(&self.state),
                KeyCode::Char('f') => {
                    self.reviewer_screen.fix_selected(&mut self.state);
                    self.set_notification("Applied suggested correction to block translation");
                }
                KeyCode::Char('k') => {
                    self.reviewer_screen.keep_with_exception(&mut self.state);
                    self.set_notification("Kept current version with explicit user exception");
                }
                KeyCode::Char('r') => {
                    self.reviewer_screen.review_again(&mut self.state);
                    self.set_notification("Dispatched re-review cycle");
                }
                KeyCode::Char('a') => self.reviewer_screen.open_feedback(),
                KeyCode::Char('e') => self.switch_screen(ScreenId::Editor),
                _ => {}
            },
            ScreenId::Context => match key.code {
                KeyCode::Tab => self.context_screen.next_tab(),
                KeyCode::Left => self.context_screen.prev_tab(),
                KeyCode::Right => self.context_screen.next_tab(),
                KeyCode::Up => self.context_screen.prev_item(&self.state),
                KeyCode::Down => self.context_screen.next_item(&self.state),
                _ => {}
            },
            ScreenId::Skills => match key.code {
                KeyCode::Tab => self.skills_screen.toggle_pane(),
                KeyCode::Up => self.skills_screen.prev(&self.state),
                KeyCode::Down => self.skills_screen.next(&self.state),
                KeyCode::Char('n') => self.skills_screen.next_decision_item(&self.state),
                KeyCode::Char('c') => {
                    self.skills_screen.confirm_decision(&mut self.state);
                    self.set_notification("Confirmed terminology decision into mock rules");
                }
                KeyCode::Char('l') => {
                    self.skills_screen.defer_decision(&mut self.state);
                    self.set_notification("Deferred terminology decision");
                }
                KeyCode::Char('s') => {
                    self.set_notification("AI suggested alternate translations");
                }
                _ => {}
            },
            ScreenId::Export => match key.code {
                KeyCode::Up => self.export_screen.prev_action(),
                KeyCode::Down => self.export_screen.next_action(),
                KeyCode::Enter => match self.export_screen.selected_action {
                    0 => self.set_notification("Simulated: Opened ./workspace/rezero/images/edit"),
                    1 => self.export_screen.toggle_image_modal(),
                    2 => self.set_notification("Simulated: Launched EPUB viewer preview"),
                    3 => self.set_notification("Simulated Neovim Handoff: Staging folder ready"),
                    4 => {
                        self.export_screen
                            .run_validation_simulation(&mut self.state);
                        self.set_notification("Ran EPUB validation suite");
                    }
                    5 => {
                        self.export_screen.export_final_epub(&mut self.state);
                        self.set_notification("Export final EPUB action invoked");
                    }
                    _ => {}
                },
                KeyCode::Char('5') => {
                    self.export_screen
                        .run_validation_simulation(&mut self.state);
                    self.set_notification("Ran EPUB validation suite");
                }
                KeyCode::Char('6') => {
                    self.export_screen.export_final_epub(&mut self.state);
                    self.set_notification("Export final EPUB action invoked");
                }
                _ => {}
            },
            ScreenId::Settings => match key.code {
                KeyCode::Tab => self.settings_screen.next_tab(),
                KeyCode::Left => self.settings_screen.prev_tab(),
                KeyCode::Right => self.settings_screen.next_tab(),
                KeyCode::Char('b') => {
                    self.settings_screen.toggle_budget(&mut self.state);
                    self.set_notification("Toggled Budget mode (Limit vs Unlimited)");
                }
                _ => {}
            },
            ScreenId::Logs => match key.code {
                KeyCode::Up => self.logs_screen.prev(&self.state),
                KeyCode::Down => self.logs_screen.next(&self.state),
                KeyCode::Char('f') => self.logs_screen.toggle_filter(),
                KeyCode::Enter => self.logs_screen.toggle_details(),
                _ => {}
            },
        }
    }
}

fn render_too_small(f: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.error))
        .title(" Terminal Dimensions Too Small ")
        .style(Style::default().bg(theme.panel_bg));

    let text = vec![
        Line::from(vec![
            Span::styled(
                " ⚠ Terminal Size: ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{} cols × {} rows", area.width, area.height),
                Style::default().fg(theme.text),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Please expand or zoom out your terminal window (min: 60 cols × 15 rows).",
            Style::default().fg(theme.text_dim),
        )),
    ];

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn render_notification_toast(f: &mut Frame, msg: &str, theme: &Theme) {
    let area = f.area();
    let width = ((msg.len() + 6) as u16).clamp(32, 60);
    let height = 3;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = area.height.saturating_sub(height + 2);
    let toast_rect = Rect {
        x,
        y,
        width,
        height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(15, 23, 42)));

    let line = Line::from(vec![
        Span::styled(
            " ℹ ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            msg,
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
    ]);

    f.render_widget(ratatui::widgets::Clear, toast_rect);
    f.render_widget(Paragraph::new(line).block(block), toast_rect);
}
