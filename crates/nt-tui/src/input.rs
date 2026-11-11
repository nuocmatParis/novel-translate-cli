use crate::navigation::Screen;
use crate::state::{AppState, FocusRegion};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAction {
    None,
    RouteOpened(Screen),
    Shutdown,
}

/// A higher-priority interaction layer can consume an event before shell routing.
pub(crate) trait EventLayer {
    fn handle_event(&mut self, event: &Event) -> bool;
}

pub(crate) fn dispatch_event(
    state: &mut AppState,
    event: &Event,
    modal: Option<&mut dyn EventLayer>,
    focused_control: Option<&mut dyn EventLayer>,
) -> UiAction {
    if is_shutdown_request(event) {
        return UiAction::Shutdown;
    }

    if !state.terminal_size_supported() {
        return UiAction::None;
    }

    if state.help_open() {
        if let Event::Key(key) = event
            && key.code == KeyCode::Esc
            && key.kind == KeyEventKind::Press
            && key.modifiers == KeyModifiers::NONE
        {
            state.close_help();
        }
        return UiAction::None;
    }

    if let Some(modal) = modal {
        // An open modal owns the event even when it has no action for that key.
        modal.handle_event(event);
        return UiAction::None;
    }

    if let Event::Key(key) = event
        && key.code == KeyCode::F(1)
        && key.kind == KeyEventKind::Press
        && key.modifiers == KeyModifiers::NONE
    {
        state.open_help();
        return UiAction::None;
    }

    if let Some(control) = focused_control
        && control.handle_event(event)
    {
        return UiAction::None;
    }

    if let Event::Key(key) = event
        && matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat)
        && key.modifiers == KeyModifiers::NONE
    {
        match key.code {
            KeyCode::PageUp if state.can_scroll_up() => {
                state.scroll_page(false);
                return UiAction::None;
            }
            KeyCode::PageDown if state.can_scroll_down() => {
                state.scroll_page(true);
                return UiAction::None;
            }
            _ => {}
        }
    }

    let Event::Key(key) = event else {
        return UiAction::None;
    };

    if key.code == KeyCode::Tab || key.code == KeyCode::BackTab {
        if key.kind != KeyEventKind::Press
            || (key.modifiers != KeyModifiers::NONE && key.modifiers != KeyModifiers::SHIFT)
        {
            return UiAction::None;
        }
        let reverse = key.code == KeyCode::BackTab || key.modifiers == KeyModifiers::SHIFT;
        state.cycle_focus(reverse);
        return UiAction::None;
    }

    match state.focus_region() {
        FocusRegion::Navigation => match (key.code, key.kind, key.modifiers) {
            (KeyCode::Up, KeyEventKind::Press | KeyEventKind::Repeat, KeyModifiers::NONE) => {
                state.move_selection(false);
                UiAction::None
            }
            (KeyCode::Down, KeyEventKind::Press | KeyEventKind::Repeat, KeyModifiers::NONE) => {
                state.move_selection(true);
                UiAction::None
            }
            (KeyCode::Enter, KeyEventKind::Press, KeyModifiers::NONE) => {
                UiAction::RouteOpened(state.open_selected_route())
            }
            _ => UiAction::None,
        },
        FocusRegion::Exit => {
            if key.code == KeyCode::Enter
                && key.kind == KeyEventKind::Press
                && key.modifiers == KeyModifiers::NONE
            {
                UiAction::Shutdown
            } else {
                UiAction::None
            }
        }
    }
}

pub(crate) fn is_shutdown_request(event: &Event) -> bool {
    let Event::Key(key) = event else {
        return false;
    };
    key.kind == KeyEventKind::Press
        && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
        && key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::ALT)
}

#[cfg(test)]
mod tests {
    use super::{EventLayer, UiAction, dispatch_event};
    use crate::navigation::Screen;
    use crate::state::{AppState, FocusRegion};
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new_with_kind(
            code,
            modifiers,
            KeyEventKind::Press,
        ))
    }

    fn repeated_key(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new_with_kind(
            code,
            modifiers,
            KeyEventKind::Repeat,
        ))
    }

    #[test]
    fn arrows_select_and_enter_opens_the_route_once() {
        let mut state = AppState::default();
        assert_eq!(state.current_screen(), Screen::Overview);

        assert_eq!(
            dispatch_event(
                &mut state,
                &key(KeyCode::Down, KeyModifiers::NONE),
                None,
                None,
            ),
            UiAction::None
        );
        assert_eq!(state.selected_screen(), Screen::VolumesAndChapters);
        assert_eq!(state.current_screen(), Screen::Overview);

        assert_eq!(
            dispatch_event(
                &mut state,
                &repeated_key(KeyCode::Enter, KeyModifiers::NONE),
                None,
                None,
            ),
            UiAction::None
        );
        assert_eq!(state.current_screen(), Screen::Overview);

        assert_eq!(
            dispatch_event(
                &mut state,
                &key(KeyCode::Enter, KeyModifiers::NONE),
                None,
                None,
            ),
            UiAction::RouteOpened(Screen::VolumesAndChapters)
        );
        assert_eq!(state.current_screen(), Screen::VolumesAndChapters);
        assert_eq!(
            dispatch_event(
                &mut state,
                &repeated_key(KeyCode::Enter, KeyModifiers::NONE),
                None,
                None,
            ),
            UiAction::None
        );
    }

    #[test]
    fn every_route_opens_in_navigation_order() {
        let mut state = AppState::default();

        for route in Screen::ALL {
            while state.selected_screen() != route {
                dispatch_event(
                    &mut state,
                    &key(KeyCode::Down, KeyModifiers::NONE),
                    None,
                    None,
                );
            }
            assert_eq!(
                dispatch_event(
                    &mut state,
                    &key(KeyCode::Enter, KeyModifiers::NONE),
                    None,
                    None,
                ),
                UiAction::RouteOpened(route)
            );
            assert_eq!(state.current_screen(), route);
        }
    }

    #[test]
    fn navigation_selection_clamps_at_both_ends() {
        let mut state = AppState::default();
        dispatch_event(
            &mut state,
            &key(KeyCode::Up, KeyModifiers::NONE),
            None,
            None,
        );
        assert_eq!(state.selected_screen(), Screen::Overview);

        for _ in 0..Screen::ALL.len() {
            dispatch_event(
                &mut state,
                &key(KeyCode::Down, KeyModifiers::NONE),
                None,
                None,
            );
        }
        assert_eq!(state.selected_screen(), Screen::SettingsReportsAndHelp);
        dispatch_event(
            &mut state,
            &key(KeyCode::Down, KeyModifiers::NONE),
            None,
            None,
        );
        assert_eq!(state.selected_screen(), Screen::SettingsReportsAndHelp);
    }

    #[test]
    fn tab_cycles_the_available_focus_and_escape_at_root_keeps_shell_open() {
        let mut state = AppState::default();
        assert_eq!(state.focus_region(), FocusRegion::Navigation);

        dispatch_event(
            &mut state,
            &key(KeyCode::Tab, KeyModifiers::NONE),
            None,
            None,
        );
        assert_eq!(state.focus_region(), FocusRegion::Exit);
        dispatch_event(
            &mut state,
            &key(KeyCode::Tab, KeyModifiers::NONE),
            None,
            None,
        );
        assert_eq!(state.focus_region(), FocusRegion::Navigation);
        dispatch_event(
            &mut state,
            &key(KeyCode::BackTab, KeyModifiers::SHIFT),
            None,
            None,
        );
        assert_eq!(state.focus_region(), FocusRegion::Exit);

        assert_eq!(
            dispatch_event(
                &mut state,
                &key(KeyCode::Esc, KeyModifiers::NONE),
                None,
                None,
            ),
            UiAction::None
        );
        assert_eq!(state.focus_region(), FocusRegion::Exit);
    }

    #[test]
    fn focus_cycles_preserve_the_open_route_and_selected_position() {
        let mut state = AppState::default();
        dispatch_event(
            &mut state,
            &key(KeyCode::Down, KeyModifiers::NONE),
            None,
            None,
        );
        dispatch_event(
            &mut state,
            &key(KeyCode::Enter, KeyModifiers::NONE),
            None,
            None,
        );

        dispatch_event(
            &mut state,
            &key(KeyCode::Tab, KeyModifiers::NONE),
            None,
            None,
        );
        dispatch_event(
            &mut state,
            &key(KeyCode::Tab, KeyModifiers::NONE),
            None,
            None,
        );

        assert_eq!(state.current_screen(), Screen::VolumesAndChapters);
        assert_eq!(state.selected_screen(), Screen::VolumesAndChapters);
        assert_eq!(state.focus_region(), FocusRegion::Navigation);
    }

    #[test]
    fn exit_requires_enter_press_and_requests_the_normal_shutdown_path() {
        let mut state = AppState::default();
        dispatch_event(
            &mut state,
            &key(KeyCode::Tab, KeyModifiers::NONE),
            None,
            None,
        );
        assert_eq!(
            dispatch_event(
                &mut state,
                &repeated_key(KeyCode::Enter, KeyModifiers::NONE),
                None,
                None,
            ),
            UiAction::None
        );
        assert_eq!(
            dispatch_event(
                &mut state,
                &key(KeyCode::Enter, KeyModifiers::NONE),
                None,
                None,
            ),
            UiAction::Shutdown
        );
    }

    #[test]
    fn ctrl_alt_c_is_not_a_global_shutdown_request() {
        let mut state = AppState::default();

        assert_eq!(
            dispatch_event(
                &mut state,
                &key(
                    KeyCode::Char('c'),
                    KeyModifiers::CONTROL | KeyModifiers::ALT,
                ),
                None,
                None,
            ),
            UiAction::None
        );
        assert_eq!(state.current_screen(), Screen::Overview);
    }

    #[test]
    fn ctrl_c_is_still_a_global_shutdown_request() {
        let mut state = AppState::default();
        assert_eq!(
            dispatch_event(
                &mut state,
                &key(KeyCode::Char('c'), KeyModifiers::CONTROL),
                None,
                None,
            ),
            UiAction::Shutdown
        );
    }

    #[test]
    fn f1_opens_context_help_and_escape_restores_route_selection_and_focus() {
        let mut state = AppState::default();
        dispatch_event(
            &mut state,
            &key(KeyCode::Down, KeyModifiers::NONE),
            None,
            None,
        );
        dispatch_event(
            &mut state,
            &key(KeyCode::Enter, KeyModifiers::NONE),
            None,
            None,
        );
        dispatch_event(
            &mut state,
            &key(KeyCode::Tab, KeyModifiers::NONE),
            None,
            None,
        );
        let before = state;
        let mut control = ProbeLayer {
            consumes: true,
            ..ProbeLayer::default()
        };

        assert_eq!(
            dispatch_event(
                &mut state,
                &key(KeyCode::F(1), KeyModifiers::NONE),
                None,
                Some(&mut control)
            ),
            UiAction::None
        );
        assert_eq!(control.calls, 0);
        assert!(state.help_open());
        assert_eq!(state.current_screen(), before.current_screen());
        assert_eq!(state.selected_screen(), before.selected_screen());
        assert_eq!(state.focus_region(), before.focus_region());

        assert_eq!(
            dispatch_event(
                &mut state,
                &repeated_key(KeyCode::F(1), KeyModifiers::NONE),
                None,
                Some(&mut control),
            ),
            UiAction::None
        );
        assert!(state.help_open());
        dispatch_event(
            &mut state,
            &repeated_key(KeyCode::Esc, KeyModifiers::NONE),
            None,
            Some(&mut control),
        );
        assert!(state.help_open());

        assert_eq!(
            dispatch_event(
                &mut state,
                &key(KeyCode::Esc, KeyModifiers::NONE),
                None,
                Some(&mut control)
            ),
            UiAction::None
        );
        assert!(!state.help_open());
        assert_eq!(state.current_screen(), before.current_screen());
        assert_eq!(state.selected_screen(), before.selected_screen());
        assert_eq!(state.focus_region(), before.focus_region());
        assert_eq!(control.calls, 0);
    }

    #[test]
    fn help_owns_events_and_prevents_leak_to_focused_control() {
        let mut state = AppState::default();
        state.navigate_to(Screen::VolumesAndChapters);
        dispatch_event(
            &mut state,
            &key(KeyCode::F(1), KeyModifiers::NONE),
            None,
            None,
        );
        let mut control = ProbeLayer {
            consumes: true,
            ..ProbeLayer::default()
        };

        for event in [
            key(KeyCode::Enter, KeyModifiers::NONE),
            key(KeyCode::PageDown, KeyModifiers::NONE),
            key(KeyCode::Char('x'), KeyModifiers::NONE),
            Event::Resize(120, 30),
            repeated_key(KeyCode::Esc, KeyModifiers::NONE),
        ] {
            assert_eq!(
                dispatch_event(&mut state, &event, None, Some(&mut control)),
                UiAction::None
            );
            assert!(state.help_open());
            assert_eq!(state.current_screen(), Screen::VolumesAndChapters);
        }
        assert_eq!(control.calls, 0);

        dispatch_event(
            &mut state,
            &key(KeyCode::Esc, KeyModifiers::NONE),
            None,
            Some(&mut control),
        );
        assert!(!state.help_open());
        assert_eq!(state.current_screen(), Screen::VolumesAndChapters);
        assert_eq!(control.calls, 0);
    }

    #[test]
    fn page_up_and_page_down_scroll_without_changing_route_focus_or_selection() {
        let mut state = AppState::default();
        state.navigate_to(Screen::VolumesAndChapters);
        state.move_selection(true);
        state.cycle_focus(false);
        state.set_scroll_metrics(5, 12);
        let before = state;

        assert_eq!(
            dispatch_event(
                &mut state,
                &key(KeyCode::PageDown, KeyModifiers::NONE),
                None,
                None,
            ),
            UiAction::None
        );
        assert_eq!(state.scroll_offset(), 5);
        assert_eq!(state.current_screen(), before.current_screen());
        assert_eq!(state.selected_screen(), before.selected_screen());
        assert_eq!(state.focus_region(), before.focus_region());

        dispatch_event(
            &mut state,
            &repeated_key(KeyCode::PageDown, KeyModifiers::NONE),
            None,
            None,
        );
        assert_eq!(state.scroll_offset(), 10);
        dispatch_event(
            &mut state,
            &key(KeyCode::PageDown, KeyModifiers::NONE),
            None,
            None,
        );
        assert_eq!(state.scroll_offset(), 12);
        assert!(!state.can_scroll_down());

        dispatch_event(
            &mut state,
            &key(KeyCode::PageUp, KeyModifiers::NONE),
            None,
            None,
        );
        assert_eq!(state.scroll_offset(), 7);
        assert!(state.can_scroll_up());

        state.set_scroll_metrics(4, 3);
        assert_eq!(state.scroll_offset(), 3);
        state.set_scroll_metrics(4, 20);
        assert_eq!(state.scroll_offset(), 7);
        state.set_scroll_metrics(4, 0);
        assert_eq!(state.scroll_offset(), 0);
        assert!(!state.can_scroll_up());
        assert!(!state.can_scroll_down());
        dispatch_event(
            &mut state,
            &key(KeyCode::PageDown, KeyModifiers::NONE),
            None,
            None,
        );
        assert_eq!(state.scroll_offset(), 0);
        state.set_scroll_metrics(4, 20);
        assert_eq!(state.scroll_offset(), 7);

        state.set_scroll_metrics(4, 8);
        state.scroll_page(true);
        let route_offset = state.scroll_offset();
        state.navigate_to(Screen::Tasks);
        state.set_scroll_metrics(4, 0);
        assert_eq!(state.scroll_offset(), 0);
        state.navigate_to(Screen::VolumesAndChapters);
        state.set_scroll_metrics(3, 20);
        assert_eq!(state.scroll_offset(), route_offset);
    }

    #[test]
    fn undersized_terminal_suppresses_controls_but_keeps_global_shutdown() {
        for (width, height) in [(79, 24), (80, 23), (1, 1)] {
            let mut state = AppState::default();
            state.set_terminal_size(width, height);

            assert_eq!(
                dispatch_event(
                    &mut state,
                    &key(KeyCode::Down, KeyModifiers::NONE),
                    None,
                    None,
                ),
                UiAction::None
            );
            assert_eq!(state.selected_screen(), Screen::Overview);
            assert!(!state.help_open());

            assert_eq!(
                dispatch_event(
                    &mut state,
                    &key(KeyCode::F(1), KeyModifiers::NONE),
                    None,
                    None,
                ),
                UiAction::None
            );
            assert!(!state.help_open());
            assert_eq!(
                dispatch_event(
                    &mut state,
                    &key(KeyCode::Char('c'), KeyModifiers::CONTROL),
                    None,
                    None,
                ),
                UiAction::Shutdown
            );

            state.set_terminal_size(80, 24);
            assert_eq!(
                dispatch_event(
                    &mut state,
                    &key(KeyCode::Down, KeyModifiers::NONE),
                    None,
                    None,
                ),
                UiAction::None
            );
            assert_eq!(state.selected_screen(), Screen::VolumesAndChapters);
        }
    }

    #[derive(Default)]
    struct ProbeLayer {
        calls: usize,
        consumes: bool,
    }

    impl EventLayer for ProbeLayer {
        fn handle_event(&mut self, _event: &Event) -> bool {
            self.calls += 1;
            self.consumes
        }
    }

    #[test]
    fn modal_precedes_input_and_blocks_route_dispatch() {
        let mut state = AppState::default();
        dispatch_event(
            &mut state,
            &key(KeyCode::Down, KeyModifiers::NONE),
            None,
            None,
        );
        let mut modal = ProbeLayer::default();
        let mut input = ProbeLayer {
            consumes: true,
            ..ProbeLayer::default()
        };

        let action = dispatch_event(
            &mut state,
            &key(KeyCode::Enter, KeyModifiers::NONE),
            Some(&mut modal),
            Some(&mut input),
        );

        assert_eq!(action, UiAction::None);
        assert_eq!(modal.calls, 1);
        assert_eq!(input.calls, 0);
        assert_eq!(state.current_screen(), Screen::Overview);
    }

    #[test]
    fn modal_consumes_f1_without_opening_nested_shell_help() {
        let mut state = AppState::default();
        let mut modal = ProbeLayer {
            consumes: true,
            ..ProbeLayer::default()
        };

        assert_eq!(
            dispatch_event(
                &mut state,
                &key(KeyCode::F(1), KeyModifiers::NONE),
                Some(&mut modal),
                None,
            ),
            UiAction::None
        );
        assert_eq!(modal.calls, 1);
        assert!(!state.help_open());
    }

    #[test]
    fn focused_input_can_consume_enter_without_opening_the_route_behind_it() {
        let mut state = AppState::default();
        dispatch_event(
            &mut state,
            &key(KeyCode::Down, KeyModifiers::NONE),
            None,
            None,
        );
        let mut input = ProbeLayer {
            consumes: true,
            ..ProbeLayer::default()
        };

        for event in [
            key(KeyCode::Char(' '), KeyModifiers::NONE),
            key(KeyCode::Enter, KeyModifiers::NONE),
            key(KeyCode::Char('x'), KeyModifiers::NONE),
        ] {
            let action = dispatch_event(&mut state, &event, None, Some(&mut input));

            assert_eq!(action, UiAction::None);
            assert_eq!(state.current_screen(), Screen::Overview);
        }
        assert_eq!(input.calls, 3);
    }
}
