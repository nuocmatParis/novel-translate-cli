//! Interactive TUI Prototype for novel-translate-cli
//!
//! A modern developer terminal application for Japanese -> Vietnamese light novel translation with AI.

#![allow(dead_code)]

mod app;
mod components;
mod screens;
mod simulation;
mod state;
mod theme;

use app::App;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{Result, stdout};
use std::panic;
use std::time::Duration;

fn main() -> Result<()> {
    // Setup panic hook to restore terminal on crash
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run application loop
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error running novel-translate-cli TUI: {err}");
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    while !app.should_quit {
        // Render UI
        terminal.draw(|f| app.render(f))?;

        // Handle user input events with 33ms timeout (~30 FPS responsive tick)
        if event::poll(Duration::from_millis(33))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        }

        // Advance simulation and state updates
        app.update();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::sidebar::ScreenId;
    use crate::state::DemoScenario;

    #[test]
    fn test_app_initialization() {
        let app = App::new();
        assert_eq!(app.current_screen, ScreenId::Dashboard);
        assert_eq!(app.state.workspace.series.volumes.len(), 4);
        assert_eq!(app.state.unresolved_issues_count(), 3);
        assert_eq!(app.state.pending_decisions_count(), 2);
    }

    #[test]
    fn test_scenario_switching() {
        let mut app = App::new();
        app.switch_scenario(DemoScenario::TranslationCompleted);
        assert_eq!(app.state.task.volume_progress_pct, 100);
        assert_eq!(app.state.task.chapter_progress_pct, 100);
        assert!(app.state.export.can_export);

        app.switch_scenario(DemoScenario::ExportValidationFailed);
        assert!(!app.state.export.can_export);
        assert!(!app.state.export.errors.is_empty());
    }

    #[test]
    fn test_editor_revision_increment() {
        let mut app = App::new();
        let initial_rev = app.state.editor_blocks[0].revision;
        app.editor_screen.selected_block_idx = 0;
        app.editor_screen.start_editing(&app.state);
        app.editor_screen.edit_buffer = "Bản dịch đã được chỉnh sửa thủ công".to_string();
        app.editor_screen.save_revision(&mut app.state);

        assert_eq!(app.state.editor_blocks[0].revision, initial_rev + 1);
        assert_eq!(
            app.state.editor_blocks[0].vietnamese,
            "Bản dịch đã được chỉnh sửa thủ công"
        );
    }

    #[test]
    fn test_reviewer_fix_issue() {
        let mut app = App::new();
        assert_eq!(app.state.unresolved_issues_count(), 3);
        app.reviewer_screen.selected_issue_idx = 0;
        app.reviewer_screen.fix_selected(&mut app.state);
        assert_eq!(app.state.unresolved_issues_count(), 2);
    }

    #[test]
    fn test_command_palette_filter() {
        let mut palette = crate::components::command_palette::CommandPalette::new();
        palette.query = "Review".to_string();
        let filtered = palette.filtered_commands();
        assert!(!filtered.is_empty());
        assert!(filtered.iter().any(|c| c.label.contains("Review")));
    }
}
