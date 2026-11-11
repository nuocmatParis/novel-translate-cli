use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm_winapi::{Console, Handle, InputRecord, KeyEventRecord};
use std::collections::HashSet;
use std::io;
use std::time::{Duration, Instant};

const VK_BACK: u16 = 0x08;
const VK_TAB: u16 = 0x09;
const VK_RETURN: u16 = 0x0d;
const VK_ESCAPE: u16 = 0x1b;
const VK_SPACE: u16 = 0x20;
const VK_PRIOR: u16 = 0x21;
const VK_NEXT: u16 = 0x22;
const VK_END: u16 = 0x23;
const VK_HOME: u16 = 0x24;
const VK_LEFT: u16 = 0x25;
const VK_UP: u16 = 0x26;
const VK_RIGHT: u16 = 0x27;
const VK_DOWN: u16 = 0x28;
const VK_INSERT: u16 = 0x2d;
const VK_DELETE: u16 = 0x2e;
const VK_F1: u16 = 0x70;
const VK_F24: u16 = 0x87;

const SHIFT_PRESSED: u32 = 0x0010;
const ALT_PRESSED: u32 = 0x0003;
const CONTROL_PRESSED: u32 = 0x000c;
const ENHANCED_KEY: u32 = 0x0100;
const WAIT_OBJECT_0: u32 = 0;
const WAIT_TIMEOUT: u32 = 258;
const WAIT_FAILED: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct PhysicalKey {
    virtual_key: u16,
    scan_code: u16,
    enhanced: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeKeyEvent {
    key_down: bool,
    repeat_count: u16,
    virtual_key: u16,
    scan_code: u16,
    unicode: u32,
    modifiers: KeyModifiers,
    enhanced: bool,
}

impl From<KeyEventRecord> for NativeKeyEvent {
    fn from(record: KeyEventRecord) -> Self {
        let control_state = record.control_key_state;
        let mut modifiers = KeyModifiers::NONE;
        if control_state.has_state(SHIFT_PRESSED) {
            modifiers.insert(KeyModifiers::SHIFT);
        }
        if control_state.has_state(ALT_PRESSED) {
            modifiers.insert(KeyModifiers::ALT);
        }
        if control_state.has_state(CONTROL_PRESSED) {
            modifiers.insert(KeyModifiers::CONTROL);
        }

        let enhanced = control_state.has_state(ENHANCED_KEY);
        Self {
            key_down: record.key_down,
            repeat_count: record.repeat_count,
            virtual_key: record.virtual_key_code,
            scan_code: record.virtual_scan_code,
            unicode: u32::from(record.u_char),
            modifiers,
            enhanced,
        }
    }
}

impl NativeKeyEvent {
    fn key_code(self) -> Option<KeyCode> {
        let code = match self.virtual_key {
            VK_BACK => KeyCode::Backspace,
            VK_TAB if self.modifiers.contains(KeyModifiers::SHIFT) => KeyCode::BackTab,
            VK_TAB => KeyCode::Tab,
            VK_RETURN => KeyCode::Enter,
            VK_ESCAPE => KeyCode::Esc,
            VK_SPACE => KeyCode::Char(' '),
            VK_PRIOR => KeyCode::PageUp,
            VK_NEXT => KeyCode::PageDown,
            VK_END => KeyCode::End,
            VK_HOME => KeyCode::Home,
            VK_LEFT => KeyCode::Left,
            VK_UP => KeyCode::Up,
            VK_RIGHT => KeyCode::Right,
            VK_DOWN => KeyCode::Down,
            VK_INSERT => KeyCode::Insert,
            VK_DELETE => KeyCode::Delete,
            VK_F1..=VK_F24 => KeyCode::F((self.virtual_key - VK_F1 + 1) as u8),
            0x41..=0x5a
                if self.modifiers.contains(KeyModifiers::CONTROL)
                    && !self.modifiers.contains(KeyModifiers::ALT) =>
            {
                KeyCode::Char((self.virtual_key as u8 as char).to_ascii_lowercase())
            }
            _ => {
                let scalar = self.unicode;
                if scalar == 0 {
                    return None;
                }
                KeyCode::Char(char::from_u32(scalar)?)
            }
        };
        Some(code)
    }

    fn identity(self) -> PhysicalKey {
        PhysicalKey {
            virtual_key: self.virtual_key,
            scan_code: self.scan_code,
            enhanced: self.enhanced,
        }
    }
}

#[derive(Default)]
struct WindowsKeyState {
    physical_down: HashSet<PhysicalKey>,
    down_at_focus_gain: HashSet<u16>,
}

#[derive(Default)]
struct WindowsSurrogateBuffer {
    pending: Option<u16>,
}

impl WindowsSurrogateBuffer {
    fn convert(&mut self, native: NativeKeyEvent, event: Option<KeyEvent>) -> Option<KeyEvent> {
        if let Some(event) = event {
            self.pending = None;
            return Some(event);
        }

        if !(0xd800..=0xdfff).contains(&native.unicode) {
            return None;
        }

        let unit = native.unicode as u16;
        let Some(previous) = self.pending.take() else {
            self.pending = Some(unit);
            return None;
        };

        char::decode_utf16([previous, unit])
            .next()?
            .ok()
            .map(|character| KeyEvent::new(KeyCode::Char(character), native.modifiers))
    }
}

impl WindowsKeyState {
    fn process_key(
        &mut self,
        native: NativeKeyEvent,
        is_virtual_key_down: impl Fn(u16) -> bool,
    ) -> Option<KeyEvent> {
        let identity = native.identity();
        let kind = if native.key_down {
            let already_down = self.down_at_focus_gain.contains(&native.virtual_key)
                || self
                    .physical_down
                    .iter()
                    .any(|pressed| pressed.virtual_key == native.virtual_key);
            self.physical_down.insert(identity);

            if already_down || native.repeat_count > 1 {
                KeyEventKind::Repeat
            } else {
                KeyEventKind::Press
            }
        } else {
            self.physical_down.remove(&identity);
            if self.down_at_focus_gain.contains(&native.virtual_key)
                && !is_virtual_key_down(native.virtual_key)
            {
                self.down_at_focus_gain.remove(&native.virtual_key);
            }
            KeyEventKind::Release
        };

        native
            .key_code()
            .map(|code| KeyEvent::new_with_kind(code, native.modifiers, kind))
    }

    fn reconcile_focus(&mut self, is_virtual_key_down: impl Fn(u16) -> bool) {
        let virtual_keys: HashSet<_> = self
            .physical_down
            .iter()
            .map(|pressed| pressed.virtual_key)
            .chain(self.down_at_focus_gain.iter().copied())
            .collect();

        self.physical_down.clear();
        self.down_at_focus_gain.clear();
        self.down_at_focus_gain.extend(
            virtual_keys
                .into_iter()
                .filter(|virtual_key| is_virtual_key_down(*virtual_key)),
        );
    }
}

pub(crate) struct WindowsInputReader {
    console: Console,
    input_handle: Handle,
    key_state: WindowsKeyState,
    surrogate_buffer: WindowsSurrogateBuffer,
}

impl WindowsInputReader {
    pub(crate) fn new() -> io::Result<Self> {
        let input = Handle::current_in_handle()?;
        Ok(Self {
            console: Console::from(input.clone()),
            input_handle: input,
            key_state: WindowsKeyState::default(),
            surrogate_buffer: WindowsSurrogateBuffer::default(),
        })
    }

    pub(crate) fn read_event(&mut self) -> io::Result<Event> {
        loop {
            if let Some(event) = self.read_event_timeout(Duration::MAX)? {
                return Ok(event);
            }
        }
    }

    pub(crate) fn read_event_timeout(&mut self, timeout: Duration) -> io::Result<Option<Event>> {
        let started = Instant::now();
        loop {
            if !self.wait_for_input(timeout.saturating_sub(started.elapsed()))? {
                return Ok(None);
            }

            let event = match self.console.read_single_input_event()? {
                InputRecord::KeyEvent(record) => {
                    let native = NativeKeyEvent::from(record);
                    let event = self.key_state.process_key(native, is_virtual_key_down);
                    self.surrogate_buffer.convert(native, event).map(Event::Key)
                }
                InputRecord::WindowBufferSizeEvent(size) => Some(Event::Resize(
                    size.size.x.max(0) as u16,
                    size.size.y.max(0) as u16,
                )),
                InputRecord::FocusEvent(focus) if focus.set_focus => {
                    self.key_state.reconcile_focus(is_virtual_key_down);
                    Some(Event::FocusGained)
                }
                InputRecord::FocusEvent(_) => Some(Event::FocusLost),
                InputRecord::MouseEvent(_) | InputRecord::MenuEvent(_) => None,
            };
            if event.is_some() {
                return Ok(event);
            }
        }
    }

    fn wait_for_input(&self, timeout: Duration) -> io::Result<bool> {
        let mut timeout_ms = timeout.as_millis();
        if !timeout.subsec_nanos().is_multiple_of(1_000_000) {
            timeout_ms += 1;
        }
        // `u32::MAX` means INFINITE to WaitForSingleObject, so reserve it.
        let timeout_ms = timeout_ms.min(u128::from(u32::MAX - 1)) as u32;
        let handle = *self.input_handle as *mut std::ffi::c_void;
        let result = unsafe { wait_for_single_object(handle, timeout_ms) };
        match result {
            WAIT_OBJECT_0 => Ok(true),
            WAIT_TIMEOUT => Ok(false),
            WAIT_FAILED => Err(io::Error::last_os_error()),
            _ => Err(io::Error::other("unexpected console input wait result")),
        }
    }
}

#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "WaitForSingleObject"]
    fn wait_for_single_object(handle: *mut std::ffi::c_void, milliseconds: u32) -> u32;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn GetAsyncKeyState(virtual_key: i32) -> i16;
}

fn is_virtual_key_down(virtual_key: u16) -> bool {
    // The high bit reports the key's current down state. The low bit is deliberately ignored.
    unsafe { GetAsyncKeyState(virtual_key as i32) & i16::MIN != 0 }
}

#[cfg(test)]
mod tests {
    use super::{NativeKeyEvent, VK_DOWN, VK_NEXT, VK_PRIOR, VK_RETURN, VK_TAB, WindowsKeyState};
    use crate::input::{UiAction, dispatch_event};
    use crate::navigation::Screen;
    use crate::state::{AppState, FocusRegion};
    use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

    fn native_key(
        virtual_key: u16,
        scan_code: u16,
        enhanced: bool,
        key_down: bool,
        repeat_count: u16,
        modifiers: KeyModifiers,
    ) -> NativeKeyEvent {
        NativeKeyEvent {
            key_down,
            repeat_count,
            virtual_key,
            scan_code,
            unicode: 0,
            modifiers,
            enhanced,
        }
    }

    fn kind(state: &mut WindowsKeyState, key: NativeKeyEvent) -> KeyEventKind {
        state
            .process_key(key, |_| false)
            .expect("key should map to a TUI event")
            .kind
    }

    #[test]
    fn keypad_enter_release_does_not_clear_main_enter() {
        let mut state = WindowsKeyState::default();
        let main_enter = native_key(VK_RETURN, 0x1c, false, true, 1, KeyModifiers::NONE);
        let keypad_enter = native_key(VK_RETURN, 0x1c, true, true, 1, KeyModifiers::NONE);
        let keypad_release = native_key(VK_RETURN, 0x1c, true, false, 1, KeyModifiers::NONE);
        let main_repeat = native_key(VK_RETURN, 0x1c, false, true, 2, KeyModifiers::NONE);
        let main_release = native_key(VK_RETURN, 0x1c, false, false, 1, KeyModifiers::NONE);

        assert_eq!(kind(&mut state, main_enter), KeyEventKind::Press);
        let keypad_event = state
            .process_key(keypad_enter, |_| false)
            .expect("keypad Enter maps");
        assert_eq!(keypad_event.kind, KeyEventKind::Repeat);
        assert!(keypad_event.state.is_empty());
        assert_eq!(kind(&mut state, keypad_release), KeyEventKind::Release);
        assert_eq!(kind(&mut state, main_repeat), KeyEventKind::Repeat);
        assert_eq!(kind(&mut state, main_release), KeyEventKind::Release);
        assert_eq!(kind(&mut state, main_enter), KeyEventKind::Press);
    }

    #[test]
    fn page_up_and_page_down_records_keep_their_scroll_keycodes() {
        let mut state = WindowsKeyState::default();
        let page_up = state
            .process_key(
                native_key(VK_PRIOR, 0x49, true, true, 1, KeyModifiers::NONE),
                |_| false,
            )
            .expect("PageUp record maps");
        let page_down = state
            .process_key(
                native_key(VK_NEXT, 0x51, true, true, 1, KeyModifiers::NONE),
                |_| false,
            )
            .expect("PageDown record maps");

        assert_eq!(page_up.code, KeyCode::PageUp);
        assert_eq!(page_down.code, KeyCode::PageDown);
    }

    #[test]
    fn focus_gain_preserves_held_keys_and_clears_keys_released_while_away() {
        let mut state = WindowsKeyState::default();
        let enter = native_key(VK_RETURN, 0x1c, false, true, 1, KeyModifiers::NONE);
        let enter_repeat = native_key(VK_RETURN, 0x1c, false, true, 2, KeyModifiers::NONE);

        assert_eq!(kind(&mut state, enter), KeyEventKind::Press);
        state.reconcile_focus(|virtual_key| virtual_key == VK_RETURN);
        assert_eq!(kind(&mut state, enter_repeat), KeyEventKind::Repeat);

        state.reconcile_focus(|_| false);
        assert_eq!(kind(&mut state, enter), KeyEventKind::Press);
    }

    #[test]
    fn utf16_surrogate_pair_preserves_a_non_bmp_character() {
        use super::WindowsSurrogateBuffer;

        let mut buffer = WindowsSurrogateBuffer::default();
        let mut high = native_key(0x41, 0x1e, false, true, 1, KeyModifiers::NONE);
        high.unicode = 0xd83d;
        let low = NativeKeyEvent {
            unicode: 0xde00,
            ..high
        };

        assert!(buffer.convert(high, None).is_none());
        let event = buffer
            .convert(low, None)
            .expect("surrogate pair maps to one key event");
        assert_eq!(event.code, KeyCode::Char('😀'));
        assert_eq!(event.kind, KeyEventKind::Press);
    }

    #[test]
    fn altgr_c_preserves_unicode_and_does_not_shutdown_or_open_a_route() {
        let mut key_state = WindowsKeyState::default();
        let mut app = AppState::default();
        let mut altgr_c = native_key(
            0x43,
            0x2e,
            false,
            true,
            1,
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        );
        altgr_c.unicode = 'ć' as u32;

        let event = key_state
            .process_key(altgr_c, |_| false)
            .expect("AltGr+C should retain its printable Unicode key");
        assert_eq!(event.code, KeyCode::Char('ć'));
        assert_eq!(event.modifiers, KeyModifiers::CONTROL | KeyModifiers::ALT);
        assert_eq!(event.kind, KeyEventKind::Press);
        assert_eq!(
            dispatch_event(&mut app, &Event::Key(event), None, None),
            UiAction::None
        );
        assert_eq!(app.current_screen(), Screen::Overview);
        assert_eq!(app.selected_screen(), Screen::Overview);
    }

    #[test]
    fn genuine_ctrl_c_still_requests_shutdown() {
        let mut key_state = WindowsKeyState::default();
        let mut app = AppState::default();
        let mut ctrl_c = native_key(0x43, 0x2e, false, true, 1, KeyModifiers::CONTROL);
        ctrl_c.unicode = 0x03;

        let event = key_state
            .process_key(ctrl_c, |_| false)
            .expect("Ctrl+C should map to the shell shutdown event");
        assert_eq!(event.code, KeyCode::Char('c'));
        assert_eq!(
            dispatch_event(&mut app, &Event::Key(event), None, None),
            UiAction::Shutdown
        );
    }

    #[test]
    fn unmapped_key_release_still_clears_physical_key_state() {
        let mut state = WindowsKeyState::default();
        let mut letter = native_key(0x41, 0x1e, false, true, 1, KeyModifiers::NONE);
        letter.unicode = 'a' as u32;
        assert_eq!(kind(&mut state, letter), KeyEventKind::Press);

        letter.key_down = false;
        letter.unicode = 0;
        assert!(state.process_key(letter, |_| false).is_none());

        letter.key_down = true;
        letter.unicode = 'a' as u32;
        assert_eq!(kind(&mut state, letter), KeyEventKind::Press);
    }

    #[test]
    fn repeat_count_and_tab_shift_keep_activation_single_and_focus_stable() {
        let mut state = WindowsKeyState::default();
        let mut app = AppState::default();
        let tab = native_key(VK_TAB, 0x0f, false, true, 1, KeyModifiers::NONE);
        let repeated_tab = native_key(VK_TAB, 0x0f, false, true, 2, KeyModifiers::NONE);
        let first = state.process_key(tab, |_| false).expect("Tab maps");
        assert_eq!(first.kind, KeyEventKind::Press);
        assert_eq!(
            dispatch_event(&mut app, &Event::Key(first), None, None),
            UiAction::None
        );
        assert_eq!(app.focus_region(), FocusRegion::Exit);

        let repeat = state
            .process_key(repeated_tab, |_| false)
            .expect("repeated Tab maps");
        assert_eq!(repeat.kind, KeyEventKind::Repeat);
        assert_eq!(
            dispatch_event(&mut app, &Event::Key(repeat), None, None),
            UiAction::None
        );
        assert_eq!(app.focus_region(), FocusRegion::Exit);

        let tab_release = native_key(VK_TAB, 0x0f, false, false, 1, KeyModifiers::NONE);
        let release = state
            .process_key(tab_release, |_| false)
            .expect("Tab release maps");
        assert_eq!(release.kind, KeyEventKind::Release);

        let shifted_tab = native_key(VK_TAB, 0x0f, false, true, 1, KeyModifiers::SHIFT);
        let back_tab = state
            .process_key(shifted_tab, |_| false)
            .expect("Shift+Tab maps");
        assert_eq!(back_tab.code, KeyCode::BackTab);
        assert_eq!(back_tab.kind, KeyEventKind::Press);
        assert_eq!(
            dispatch_event(&mut app, &Event::Key(back_tab), None, None),
            UiAction::None
        );
        assert_eq!(app.focus_region(), FocusRegion::Navigation);

        let repeated_down = native_key(VK_DOWN, 0x50, false, true, 2, KeyModifiers::NONE);
        let event = state
            .process_key(repeated_down, |_| false)
            .expect("Down maps");
        assert_eq!(event.code, KeyCode::Down);
        assert_eq!(event.kind, KeyEventKind::Repeat);
        assert_eq!(
            dispatch_event(&mut app, &Event::Key(event), None, None),
            UiAction::None
        );
        assert_eq!(app.current_screen(), Screen::Overview);

        let down = native_key(VK_DOWN, 0x50, false, true, 1, KeyModifiers::NONE);
        let event = state.process_key(down, |_| false).expect("Down maps");
        assert_eq!(event.kind, KeyEventKind::Repeat);
        assert_eq!(
            dispatch_event(&mut app, &Event::Key(event), None, None),
            UiAction::None
        );
        assert_eq!(app.selected_screen(), Screen::Tasks);
    }
}
