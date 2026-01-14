//! Terminal input handling
//!
//! This module provides Rust implementations for terminal input operations,
//! including key encoding, input forwarding, and special key handling.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::use_self)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_void;
use std::os::raw::c_int;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to Terminal struct.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalHandle(*mut c_void);

impl TerminalHandle {
    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

// =============================================================================
// Input Key Types
// =============================================================================

/// Special key codes for terminal input.
/// These correspond to VTermKey values.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalKey {
    /// No key (placeholder).
    None = 0,
    /// Enter/Return key.
    Enter = 1,
    /// Tab key.
    Tab = 2,
    /// Backspace key.
    Backspace = 3,
    /// Escape key.
    Escape = 4,
    /// Up arrow.
    Up = 5,
    /// Down arrow.
    Down = 6,
    /// Left arrow.
    Left = 7,
    /// Right arrow.
    Right = 8,
    /// Insert key.
    Insert = 9,
    /// Delete key.
    Delete = 10,
    /// Home key.
    Home = 11,
    /// End key.
    End = 12,
    /// Page Up key.
    PageUp = 13,
    /// Page Down key.
    PageDown = 14,
    /// Function key base (F1 = Function + 1).
    Function = 15,
    /// Keypad 0.
    KP0 = 16,
    /// Keypad 1.
    KP1 = 17,
    /// Keypad 2.
    KP2 = 18,
    /// Keypad 3.
    KP3 = 19,
    /// Keypad 4.
    KP4 = 20,
    /// Keypad 5.
    KP5 = 21,
    /// Keypad 6.
    KP6 = 22,
    /// Keypad 7.
    KP7 = 23,
    /// Keypad 8.
    KP8 = 24,
    /// Keypad 9.
    KP9 = 25,
}

impl TerminalKey {
    /// Check if this is an arrow key.
    pub const fn is_arrow(&self) -> bool {
        matches!(
            self,
            TerminalKey::Up | TerminalKey::Down | TerminalKey::Left | TerminalKey::Right
        )
    }

    /// Check if this is a navigation key.
    pub const fn is_navigation(&self) -> bool {
        matches!(
            self,
            TerminalKey::Home | TerminalKey::End | TerminalKey::PageUp | TerminalKey::PageDown
        )
    }

    /// Check if this is a keypad key.
    pub const fn is_keypad(&self) -> bool {
        matches!(
            self,
            TerminalKey::KP0
                | TerminalKey::KP1
                | TerminalKey::KP2
                | TerminalKey::KP3
                | TerminalKey::KP4
                | TerminalKey::KP5
                | TerminalKey::KP6
                | TerminalKey::KP7
                | TerminalKey::KP8
                | TerminalKey::KP9
        )
    }

    /// Check if this is a function key.
    pub const fn is_function(&self) -> bool {
        matches!(self, TerminalKey::Function)
    }

    /// Convert from raw value.
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(TerminalKey::None),
            1 => Some(TerminalKey::Enter),
            2 => Some(TerminalKey::Tab),
            3 => Some(TerminalKey::Backspace),
            4 => Some(TerminalKey::Escape),
            5 => Some(TerminalKey::Up),
            6 => Some(TerminalKey::Down),
            7 => Some(TerminalKey::Left),
            8 => Some(TerminalKey::Right),
            9 => Some(TerminalKey::Insert),
            10 => Some(TerminalKey::Delete),
            11 => Some(TerminalKey::Home),
            12 => Some(TerminalKey::End),
            13 => Some(TerminalKey::PageUp),
            14 => Some(TerminalKey::PageDown),
            15 => Some(TerminalKey::Function),
            16 => Some(TerminalKey::KP0),
            17 => Some(TerminalKey::KP1),
            18 => Some(TerminalKey::KP2),
            19 => Some(TerminalKey::KP3),
            20 => Some(TerminalKey::KP4),
            21 => Some(TerminalKey::KP5),
            22 => Some(TerminalKey::KP6),
            23 => Some(TerminalKey::KP7),
            24 => Some(TerminalKey::KP8),
            25 => Some(TerminalKey::KP9),
            _ => None,
        }
    }
}

// =============================================================================
// Input Event Types
// =============================================================================

/// Type of terminal input event.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEventType {
    /// Character input (Unicode codepoint).
    Char = 0,
    /// Special key input.
    Key = 1,
    /// Mouse event.
    Mouse = 2,
    /// Paste start.
    PasteStart = 3,
    /// Paste end.
    PasteEnd = 4,
    /// Focus gained.
    FocusGained = 5,
    /// Focus lost.
    FocusLost = 6,
}

/// Terminal input event.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    /// Event type.
    pub event_type: InputEventType,
    /// Character code (for Char events).
    pub char_code: u32,
    /// Key code (for Key events).
    pub key_code: c_int,
    /// Modifier flags.
    pub modifiers: c_int,
}

impl InputEvent {
    /// Create a character input event.
    pub const fn char_event(code: u32, modifiers: c_int) -> Self {
        Self {
            event_type: InputEventType::Char,
            char_code: code,
            key_code: 0,
            modifiers,
        }
    }

    /// Create a key input event.
    pub const fn key_event(key: c_int, modifiers: c_int) -> Self {
        Self {
            event_type: InputEventType::Key,
            char_code: 0,
            key_code: key,
            modifiers,
        }
    }

    /// Create a paste start event.
    pub const fn paste_start() -> Self {
        Self {
            event_type: InputEventType::PasteStart,
            char_code: 0,
            key_code: 0,
            modifiers: 0,
        }
    }

    /// Create a paste end event.
    pub const fn paste_end() -> Self {
        Self {
            event_type: InputEventType::PasteEnd,
            char_code: 0,
            key_code: 0,
            modifiers: 0,
        }
    }

    /// Create a focus gained event.
    pub const fn focus_gained() -> Self {
        Self {
            event_type: InputEventType::FocusGained,
            char_code: 0,
            key_code: 0,
            modifiers: 0,
        }
    }

    /// Create a focus lost event.
    pub const fn focus_lost() -> Self {
        Self {
            event_type: InputEventType::FocusLost,
            char_code: 0,
            key_code: 0,
            modifiers: 0,
        }
    }
}

// =============================================================================
// Input Validation
// =============================================================================

/// Result of input validation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputValidation {
    /// Input is valid.
    Valid = 0,
    /// Terminal is null.
    NullTerminal = 1,
    /// Terminal is closed.
    Closed = 2,
    /// Invalid key code.
    InvalidKey = 3,
    /// Invalid character.
    InvalidChar = 4,
}

/// Check if a character is valid for terminal input.
pub const fn is_valid_input_char(c: u32) -> bool {
    // Valid Unicode range excluding surrogates and invalid codepoints
    // Max valid Unicode codepoint is U+10FFFF
    // Surrogates are U+D800 to U+DFFF
    c <= 0x0010_FFFF && !(c >= 0xD800 && c <= 0xDFFF)
}

/// Check if a key code is valid for terminal input.
pub const fn is_valid_key_code(key: c_int) -> bool {
    // Valid VTermKey range (0-36 based on VTERM_KEY_MAX)
    key >= 0 && key <= 36
}

// =============================================================================
// Input Modifiers
// =============================================================================

/// Input modifier flags.
pub const INPUT_MOD_SHIFT: c_int = 1;
pub const INPUT_MOD_ALT: c_int = 2;
pub const INPUT_MOD_CTRL: c_int = 4;

/// Helper for building modifier combinations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputModifiers {
    flags: c_int,
}

impl InputModifiers {
    /// No modifiers.
    pub const fn none() -> Self {
        Self { flags: 0 }
    }

    /// Create from raw flags.
    pub const fn from_raw(flags: c_int) -> Self {
        Self { flags }
    }

    /// Get raw flags.
    pub const fn as_raw(self) -> c_int {
        self.flags
    }

    /// Check if shift is pressed.
    pub const fn has_shift(self) -> bool {
        (self.flags & INPUT_MOD_SHIFT) != 0
    }

    /// Check if alt is pressed.
    pub const fn has_alt(self) -> bool {
        (self.flags & INPUT_MOD_ALT) != 0
    }

    /// Check if ctrl is pressed.
    pub const fn has_ctrl(self) -> bool {
        (self.flags & INPUT_MOD_CTRL) != 0
    }

    /// Add shift modifier.
    pub const fn with_shift(self) -> Self {
        Self {
            flags: self.flags | INPUT_MOD_SHIFT,
        }
    }

    /// Add alt modifier.
    pub const fn with_alt(self) -> Self {
        Self {
            flags: self.flags | INPUT_MOD_ALT,
        }
    }

    /// Add ctrl modifier.
    pub const fn with_ctrl(self) -> Self {
        Self {
            flags: self.flags | INPUT_MOD_CTRL,
        }
    }

    /// Count active modifiers.
    pub const fn count(self) -> c_int {
        let mut count = 0;
        if self.has_shift() {
            count += 1;
        }
        if self.has_alt() {
            count += 1;
        }
        if self.has_ctrl() {
            count += 1;
        }
        count
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if character is valid for terminal input.
#[no_mangle]
pub extern "C" fn rs_terminal_is_valid_input_char(c: u32) -> c_int {
    c_int::from(is_valid_input_char(c))
}

/// FFI export: Check if key code is valid for terminal input.
#[no_mangle]
pub extern "C" fn rs_terminal_is_valid_key_code(key: c_int) -> c_int {
    c_int::from(is_valid_key_code(key))
}

/// FFI export: Create character input event.
#[no_mangle]
pub extern "C" fn rs_terminal_char_event(code: u32, modifiers: c_int) -> InputEvent {
    InputEvent::char_event(code, modifiers)
}

/// FFI export: Create key input event.
#[no_mangle]
pub extern "C" fn rs_terminal_key_event(key: c_int, modifiers: c_int) -> InputEvent {
    InputEvent::key_event(key, modifiers)
}

/// FFI export: Count active modifiers.
#[no_mangle]
pub extern "C" fn rs_terminal_modifier_count(flags: c_int) -> c_int {
    InputModifiers::from_raw(flags).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_key_classification() {
        assert!(TerminalKey::Up.is_arrow());
        assert!(TerminalKey::Down.is_arrow());
        assert!(TerminalKey::Left.is_arrow());
        assert!(TerminalKey::Right.is_arrow());
        assert!(!TerminalKey::Enter.is_arrow());

        assert!(TerminalKey::Home.is_navigation());
        assert!(TerminalKey::End.is_navigation());
        assert!(TerminalKey::PageUp.is_navigation());
        assert!(TerminalKey::PageDown.is_navigation());
        assert!(!TerminalKey::Up.is_navigation());

        assert!(TerminalKey::KP0.is_keypad());
        assert!(TerminalKey::KP9.is_keypad());
        assert!(!TerminalKey::Enter.is_keypad());

        assert!(TerminalKey::Function.is_function());
        assert!(!TerminalKey::Enter.is_function());
    }

    #[test]
    fn test_terminal_key_from_raw() {
        assert_eq!(TerminalKey::from_raw(0), Some(TerminalKey::None));
        assert_eq!(TerminalKey::from_raw(1), Some(TerminalKey::Enter));
        assert_eq!(TerminalKey::from_raw(5), Some(TerminalKey::Up));
        assert_eq!(TerminalKey::from_raw(25), Some(TerminalKey::KP9));
        assert_eq!(TerminalKey::from_raw(100), None);
        assert_eq!(TerminalKey::from_raw(-1), None);
    }

    #[test]
    fn test_input_event_creation() {
        let char_event = InputEvent::char_event(0x41, INPUT_MOD_SHIFT);
        assert_eq!(char_event.event_type, InputEventType::Char);
        assert_eq!(char_event.char_code, 0x41);
        assert_eq!(char_event.modifiers, INPUT_MOD_SHIFT);

        let key_event = InputEvent::key_event(5, INPUT_MOD_CTRL);
        assert_eq!(key_event.event_type, InputEventType::Key);
        assert_eq!(key_event.key_code, 5);
        assert_eq!(key_event.modifiers, INPUT_MOD_CTRL);

        let paste_start = InputEvent::paste_start();
        assert_eq!(paste_start.event_type, InputEventType::PasteStart);

        let focus = InputEvent::focus_gained();
        assert_eq!(focus.event_type, InputEventType::FocusGained);
    }

    #[test]
    fn test_input_validation() {
        // Valid ASCII
        assert!(is_valid_input_char(0x41)); // 'A'
        assert!(is_valid_input_char(0x20)); // space

        // Valid Unicode
        assert!(is_valid_input_char(0x1F600)); // emoji
        assert!(is_valid_input_char(0x0010_FFFF)); // max valid

        // Invalid - surrogates
        assert!(!is_valid_input_char(0xD800));
        assert!(!is_valid_input_char(0xDFFF));

        // Invalid - beyond max
        assert!(!is_valid_input_char(0x0011_0000));
    }

    #[test]
    fn test_key_code_validation() {
        assert!(is_valid_key_code(0));
        assert!(is_valid_key_code(1)); // Enter
        assert!(is_valid_key_code(25)); // KP9
        assert!(is_valid_key_code(36)); // Max

        assert!(!is_valid_key_code(-1));
        assert!(!is_valid_key_code(37));
        assert!(!is_valid_key_code(100));
    }

    #[test]
    fn test_input_modifiers() {
        let none = InputModifiers::none();
        assert!(!none.has_shift());
        assert!(!none.has_alt());
        assert!(!none.has_ctrl());
        assert_eq!(none.count(), 0);

        let shift = none.with_shift();
        assert!(shift.has_shift());
        assert!(!shift.has_alt());
        assert_eq!(shift.count(), 1);

        let shift_ctrl = shift.with_ctrl();
        assert!(shift_ctrl.has_shift());
        assert!(shift_ctrl.has_ctrl());
        assert_eq!(shift_ctrl.count(), 2);

        let all = InputModifiers::none().with_shift().with_alt().with_ctrl();
        assert_eq!(all.count(), 3);
    }

    #[test]
    fn test_input_modifiers_from_raw() {
        let mods = InputModifiers::from_raw(INPUT_MOD_SHIFT | INPUT_MOD_CTRL);
        assert!(mods.has_shift());
        assert!(mods.has_ctrl());
        assert!(!mods.has_alt());
        assert_eq!(mods.as_raw(), 5);
    }

    #[test]
    fn test_input_validation_enum_values() {
        assert_eq!(InputValidation::Valid as c_int, 0);
        assert_eq!(InputValidation::NullTerminal as c_int, 1);
        assert_eq!(InputValidation::Closed as c_int, 2);
        assert_eq!(InputValidation::InvalidKey as c_int, 3);
        assert_eq!(InputValidation::InvalidChar as c_int, 4);
    }

    #[test]
    fn test_input_event_type_values() {
        assert_eq!(InputEventType::Char as c_int, 0);
        assert_eq!(InputEventType::Key as c_int, 1);
        assert_eq!(InputEventType::Mouse as c_int, 2);
        assert_eq!(InputEventType::PasteStart as c_int, 3);
        assert_eq!(InputEventType::PasteEnd as c_int, 4);
        assert_eq!(InputEventType::FocusGained as c_int, 5);
        assert_eq!(InputEventType::FocusLost as c_int, 6);
    }

    #[test]
    fn test_struct_sizes() {
        use std::mem::size_of;
        // InputModifiers: 1 * 4 = 4 bytes
        assert_eq!(size_of::<InputModifiers>(), 4);
        // InputEvent: should be reasonable size
        assert!(size_of::<InputEvent>() <= 24);
    }
}
