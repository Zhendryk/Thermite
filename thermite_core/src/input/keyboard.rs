use crate::platform::event::{Event, EventCategory};
use bitflags::bitflags;
use winit::event::{KeyboardInput, ModifiersState, ScanCode, VirtualKeyCode};

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct KeyCode {
    physical: ScanCode,
    mapped: Option<VirtualKeyCode>,
}

impl From<KeyboardInput> for KeyCode {
    fn from(keyboard_input: KeyboardInput) -> Self {
        Self {
            physical: keyboard_input.scancode,
            mapped: keyboard_input.virtual_keycode,
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct KeyboardModifiers: u8 {
        const NONE  = 0b0000_0000;
        const SHIFT = 0b0000_0001;
        const CTRL  = 0b0000_0010;
        const ALT   = 0b0000_0100;
        const LOGO  = 0b0000_1000;
    }
}

impl From<ModifiersState> for KeyboardModifiers {
    fn from(modifiers_state: ModifiersState) -> Self {
        let mut keyboard_modifiers = KeyboardModifiers::empty();
        if modifiers_state.shift() {
            keyboard_modifiers |= Self::SHIFT;
        }
        if modifiers_state.ctrl() {
            keyboard_modifiers |= Self::CTRL;
        }
        if modifiers_state.alt() {
            keyboard_modifiers |= Self::ALT;
        }
        if modifiers_state.logo() {
            keyboard_modifiers |= Self::LOGO;
        }
        keyboard_modifiers
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum KeyboardEvent {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    ModifiersChanged(KeyboardModifiers),
}

impl Event for KeyboardEvent {
    fn category(&self) -> EventCategory {
        EventCategory::Keyboard
    }

    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
