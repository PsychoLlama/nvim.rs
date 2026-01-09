//! Key event processing for insert mode.
//!
//! This module provides Rust infrastructure for handling keyboard events
//! in insert mode, including key classification, navigation keys, and
//! special key handling.

use std::ffi::c_int;

/// ASCII control characters (from `ascii_defs.h`).
pub mod ascii {
    /// Tab character.
    pub const TAB: i32 = 0x09;
    /// Newline character (LF).
    pub const NL: i32 = 0x0A;
    /// Carriage return.
    pub const CAR: i32 = 0x0D;
    /// Escape character.
    pub const ESC: i32 = 0x1B;

    // CTRL key codes
    /// Ctrl-@
    pub const CTRL_AT: i32 = 0;
    /// Ctrl-A (insert previously inserted text)
    pub const CTRL_A: i32 = 1;
    /// Ctrl-B
    pub const CTRL_B: i32 = 2;
    /// Ctrl-C (break)
    pub const CTRL_C: i32 = 3;
    /// Ctrl-D (decrease indent)
    pub const CTRL_D: i32 = 4;
    /// Ctrl-E (insert char from below cursor)
    pub const CTRL_E: i32 = 5;
    /// Ctrl-F
    pub const CTRL_F: i32 = 6;
    /// Ctrl-G (commands)
    pub const CTRL_G: i32 = 7;
    /// Ctrl-H (backspace)
    pub const CTRL_H: i32 = 8;
    /// Ctrl-I (tab)
    pub const CTRL_I: i32 = 9;
    /// Ctrl-J (newline)
    pub const CTRL_J: i32 = 10;
    /// Ctrl-K (digraph)
    pub const CTRL_K: i32 = 11;
    /// Ctrl-L
    pub const CTRL_L: i32 = 12;
    /// Ctrl-M (carriage return)
    pub const CTRL_M: i32 = 13;
    /// Ctrl-N (next match in completion)
    pub const CTRL_N: i32 = 14;
    /// Ctrl-O (execute one normal mode command)
    pub const CTRL_O: i32 = 15;
    /// Ctrl-P (previous match in completion)
    pub const CTRL_P: i32 = 16;
    /// Ctrl-Q (same as Ctrl-V on some systems)
    pub const CTRL_Q: i32 = 17;
    /// Ctrl-R (insert register contents)
    pub const CTRL_R: i32 = 18;
    /// Ctrl-S
    pub const CTRL_S: i32 = 19;
    /// Ctrl-T (increase indent)
    pub const CTRL_T: i32 = 20;
    /// Ctrl-U (delete to start of insert)
    pub const CTRL_U: i32 = 21;
    /// Ctrl-V (insert literal)
    pub const CTRL_V: i32 = 22;
    /// Ctrl-W (delete word)
    pub const CTRL_W: i32 = 23;
    /// Ctrl-X (completion mode prefix)
    pub const CTRL_X: i32 = 24;
    /// Ctrl-Y (insert char from above cursor)
    pub const CTRL_Y: i32 = 25;
    /// Ctrl-Z
    pub const CTRL_Z: i32 = 26;
    /// Ctrl-\ (backslash)
    pub const CTRL_BSL: i32 = 28;
    /// Ctrl-] (right square bracket)
    pub const CTRL_RSB: i32 = 29;
    /// Ctrl-^ (switch input mode)
    pub const CTRL_HAT: i32 = 30;
    /// Ctrl-_ (switch language)
    pub const CTRL_UNDERSCORE: i32 = 31;
}

/// Special key codes (derived from `keycodes.h`).
///
/// These are internal Neovim key codes for special keys.
pub mod special_keys {
    use std::ffi::c_int;

    // Key code generation helper (matches TERMCAP2KEY macro)
    // Uses `as` casts because `i32::from()` is not const stable yet
    #[allow(clippy::cast_lossless)]
    const fn termcap2key(a: u8, b: u8) -> c_int {
        // TERMCAP2KEY(a, b) = (-((a as i32) + ((b as i32) << 8)))
        -((a as i32) + ((b as i32) << 8))
    }

    // KS_EXTRA constant from keycodes.h
    const KS_EXTRA: u8 = 0xFE;

    // KE_* extra key codes
    const KE_S_UP: u8 = 4;
    const KE_S_DOWN: u8 = 5;
    const KE_TAB: u8 = 16;

    /// Up arrow.
    pub const K_UP: c_int = termcap2key(b'k', b'u');
    /// Down arrow.
    pub const K_DOWN: c_int = termcap2key(b'k', b'd');
    /// Left arrow.
    pub const K_LEFT: c_int = termcap2key(b'k', b'l');
    /// Right arrow.
    pub const K_RIGHT: c_int = termcap2key(b'k', b'r');

    /// Shift-Up.
    pub const K_S_UP: c_int = termcap2key(KS_EXTRA, KE_S_UP);
    /// Shift-Down.
    pub const K_S_DOWN: c_int = termcap2key(KS_EXTRA, KE_S_DOWN);
    /// Shift-Left.
    pub const K_S_LEFT: c_int = termcap2key(b'#', b'4');
    /// Shift-Right.
    pub const K_S_RIGHT: c_int = termcap2key(b'%', b'i');

    /// Tab key.
    pub const K_TAB: c_int = termcap2key(KS_EXTRA, KE_TAB);
    /// Shift-Tab.
    pub const K_S_TAB: c_int = termcap2key(b'k', b'B');

    /// Backspace.
    pub const K_BS: c_int = termcap2key(b'k', b'b');
    /// Insert key.
    pub const K_INS: c_int = termcap2key(b'k', b'I');
    /// Delete key.
    pub const K_DEL: c_int = termcap2key(b'k', b'D');
    /// Home key.
    pub const K_HOME: c_int = termcap2key(b'k', b'h');
    /// End key.
    pub const K_END: c_int = termcap2key(b'@', b'7');
    /// Page Up.
    pub const K_PAGEUP: c_int = termcap2key(b'k', b'P');
    /// Page Down.
    pub const K_PAGEDOWN: c_int = termcap2key(b'k', b'N');

    /// Keypad Enter.
    pub const K_KENTER: c_int = termcap2key(b'K', b'A');
    /// Keypad Home.
    pub const K_KHOME: c_int = termcap2key(b'K', b'1');
    /// Keypad End.
    pub const K_KEND: c_int = termcap2key(b'K', b'4');
    /// Keypad Delete.
    pub const K_KDEL: c_int = termcap2key(b'K', b'9');
    /// Keypad Page Up.
    pub const K_KPAGEUP: c_int = termcap2key(b'K', b'3');
    /// Keypad Page Down.
    pub const K_KPAGEDOWN: c_int = termcap2key(b'K', b'5');

    /// Shift-Home.
    pub const K_S_HOME: c_int = termcap2key(b'#', b'2');
    /// Shift-End.
    pub const K_S_END: c_int = termcap2key(b'*', b'7');

    /// Ctrl-Home.
    pub const K_C_HOME: c_int = termcap2key(b'k', b'6');
    /// Ctrl-End.
    pub const K_C_END: c_int = termcap2key(b'k', b'7');
    /// Ctrl-Left.
    pub const K_C_LEFT: c_int = termcap2key(b'k', b'8');
    /// Ctrl-Right.
    pub const K_C_RIGHT: c_int = termcap2key(b'k', b'9');

    /// Zero key (special).
    pub const K_ZERO: c_int = termcap2key(b'K', b'0');
}

/// Backspace mode constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum BackspaceMode {
    /// Delete a single character.
    Char = 0,
    /// Delete a word.
    Word = 1,
    /// Delete to start of line.
    Line = 2,
}

impl BackspaceMode {
    /// Create from integer value.
    #[must_use]
    pub const fn from_int(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::Char),
            1 => Some(Self::Word),
            2 => Some(Self::Line),
            _ => None,
        }
    }
}

/// Key action classification for insert mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Insert a normal character.
    InsertChar,
    /// Delete character(s).
    Delete,
    /// Navigate cursor.
    Navigate,
    /// Completion-related key.
    Completion,
    /// Mode transition (exit insert, CTRL-O, etc.).
    ModeChange,
    /// Special operation (register insert, digraph, etc.).
    Special,
    /// Ignore this key.
    Ignore,
}

/// Navigation direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavDirection {
    /// Move left.
    Left,
    /// Move right.
    Right,
    /// Move up.
    Up,
    /// Move down.
    Down,
    /// Move to start of line.
    Home,
    /// Move to end of line.
    End,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
}

/// Check if a key code is an arrow key.
#[inline]
#[must_use]
pub const fn is_arrow_key(key: c_int) -> bool {
    matches!(
        key,
        special_keys::K_UP
            | special_keys::K_DOWN
            | special_keys::K_LEFT
            | special_keys::K_RIGHT
            | special_keys::K_S_UP
            | special_keys::K_S_DOWN
            | special_keys::K_S_LEFT
            | special_keys::K_S_RIGHT
            | special_keys::K_C_LEFT
            | special_keys::K_C_RIGHT
    )
}

/// Check if a key code is a navigation key (arrows, home, end, page up/down).
#[inline]
#[must_use]
pub const fn is_navigation_key(key: c_int) -> bool {
    is_arrow_key(key)
        || matches!(
            key,
            special_keys::K_HOME
                | special_keys::K_END
                | special_keys::K_KHOME
                | special_keys::K_KEND
                | special_keys::K_S_HOME
                | special_keys::K_S_END
                | special_keys::K_C_HOME
                | special_keys::K_C_END
                | special_keys::K_PAGEUP
                | special_keys::K_PAGEDOWN
                | special_keys::K_KPAGEUP
                | special_keys::K_KPAGEDOWN
        )
}

/// Check if a key code is a delete key (BS, DEL).
#[inline]
#[must_use]
pub const fn is_delete_key(key: c_int) -> bool {
    matches!(
        key,
        special_keys::K_BS | special_keys::K_DEL | special_keys::K_KDEL | ascii::CTRL_H
    )
}

/// Check if a key code is a backspace variant.
#[inline]
#[must_use]
pub const fn is_backspace_key(key: c_int) -> bool {
    matches!(key, special_keys::K_BS | ascii::CTRL_H)
}

/// Check if a key is an enter/newline key.
///
/// Note: `CTRL_M` == `CAR` (13) and `CTRL_J` == `NL` (10)
#[inline]
#[must_use]
pub const fn is_enter_key(key: c_int) -> bool {
    matches!(key, ascii::CAR | ascii::NL | special_keys::K_KENTER)
}

/// Check if a key is a tab key.
///
/// Note: `CTRL_I` == `TAB` (9)
#[inline]
#[must_use]
pub const fn is_tab_key(key: c_int) -> bool {
    matches!(key, ascii::TAB | special_keys::K_TAB | special_keys::K_S_TAB)
}

/// Check if a key is a control character (Ctrl-A through Ctrl-Z).
#[inline]
#[must_use]
pub const fn is_ctrl_key(key: c_int) -> bool {
    key >= 0 && key < 32
}

/// Check if a key is an escape key.
#[inline]
#[must_use]
pub const fn is_escape_key(key: c_int) -> bool {
    key == ascii::ESC || key == ascii::CTRL_C
}

/// Check if a key is printable ASCII.
#[inline]
#[must_use]
pub const fn is_printable_ascii(key: c_int) -> bool {
    key >= 0x20 && key < 0x7f
}

/// Check if this is a special (negative) key code.
#[inline]
#[must_use]
pub const fn is_special_key(key: c_int) -> bool {
    key < 0
}

/// Get the navigation direction for a key, if it's a navigation key.
#[must_use]
pub const fn get_nav_direction(key: c_int) -> Option<NavDirection> {
    match key {
        special_keys::K_LEFT
        | special_keys::K_S_LEFT
        | special_keys::K_C_LEFT => Some(NavDirection::Left),
        special_keys::K_RIGHT
        | special_keys::K_S_RIGHT
        | special_keys::K_C_RIGHT => Some(NavDirection::Right),
        special_keys::K_UP | special_keys::K_S_UP => Some(NavDirection::Up),
        special_keys::K_DOWN | special_keys::K_S_DOWN => Some(NavDirection::Down),
        special_keys::K_HOME
        | special_keys::K_KHOME
        | special_keys::K_S_HOME
        | special_keys::K_C_HOME => Some(NavDirection::Home),
        special_keys::K_END
        | special_keys::K_KEND
        | special_keys::K_S_END
        | special_keys::K_C_END => Some(NavDirection::End),
        special_keys::K_PAGEUP | special_keys::K_KPAGEUP => Some(NavDirection::PageUp),
        special_keys::K_PAGEDOWN | special_keys::K_KPAGEDOWN => Some(NavDirection::PageDown),
        _ => None,
    }
}

/// Classify a key for insert mode processing.
#[must_use]
pub const fn classify_key(key: c_int) -> KeyAction {
    // Escape exits insert mode
    if is_escape_key(key) {
        return KeyAction::ModeChange;
    }

    // Delete keys
    if is_delete_key(key) {
        return KeyAction::Delete;
    }

    // Navigation keys
    if is_navigation_key(key) {
        return KeyAction::Navigate;
    }

    // Completion keys
    if key == ascii::CTRL_N || key == ascii::CTRL_P || key == ascii::CTRL_X {
        return KeyAction::Completion;
    }

    // Mode change keys
    if key == ascii::CTRL_O || key == ascii::CTRL_C {
        return KeyAction::ModeChange;
    }

    // Special operation keys
    if key == ascii::CTRL_R   // register
        || key == ascii::CTRL_K   // digraph
        || key == ascii::CTRL_V   // literal
        || key == ascii::CTRL_A   // previous insert
        || key == ascii::CTRL_E   // char below
        || key == ascii::CTRL_Y   // char above
        || key == ascii::CTRL_G   // CTRL-G commands
        || key == ascii::CTRL_HAT // input mode switch
        || key == ascii::CTRL_UNDERSCORE  // language switch
    {
        return KeyAction::Special;
    }

    // Enter and Tab are normal insertion (with special handling)
    if is_enter_key(key) || is_tab_key(key) {
        return KeyAction::InsertChar;
    }

    // Indent change
    if key == ascii::CTRL_D || key == ascii::CTRL_T {
        return KeyAction::Special;
    }

    // Word/line delete
    if key == ascii::CTRL_W || key == ascii::CTRL_U {
        return KeyAction::Delete;
    }

    // Printable characters
    if is_printable_ascii(key) || key >= 0x100 {
        return KeyAction::InsertChar;
    }

    // Control characters we don't handle specifically
    if is_ctrl_key(key) {
        return KeyAction::Ignore;
    }

    // Special key codes (negative values)
    if is_special_key(key) {
        // Many special keys need specific handling
        return KeyAction::Special;
    }

    // Default: treat as character to insert
    KeyAction::InsertChar
}

// FFI exports

// FFI exports - prefixed with `edit_` to avoid conflicts with cmdline/keys.rs

/// FFI: Check if key is an arrow key (edit mode).
#[no_mangle]
pub extern "C" fn rs_edit_is_arrow_key(key: c_int) -> c_int {
    c_int::from(is_arrow_key(key))
}

/// FFI: Check if key is a navigation key (edit mode).
#[no_mangle]
pub extern "C" fn rs_edit_is_navigation_key(key: c_int) -> c_int {
    c_int::from(is_navigation_key(key))
}

/// FFI: Check if key is a delete key (edit mode).
#[no_mangle]
pub extern "C" fn rs_edit_is_delete_key(key: c_int) -> c_int {
    c_int::from(is_delete_key(key))
}

/// FFI: Check if key is a backspace key (edit mode).
#[no_mangle]
pub extern "C" fn rs_edit_is_backspace_key(key: c_int) -> c_int {
    c_int::from(is_backspace_key(key))
}

/// FFI: Check if key is an enter key (edit mode).
#[no_mangle]
pub extern "C" fn rs_edit_is_enter_key(key: c_int) -> c_int {
    c_int::from(is_enter_key(key))
}

/// FFI: Check if key is a tab key (edit mode).
#[no_mangle]
pub extern "C" fn rs_edit_is_tab_key(key: c_int) -> c_int {
    c_int::from(is_tab_key(key))
}

/// FFI: Check if key is a control key (edit mode).
#[no_mangle]
pub extern "C" fn rs_edit_is_ctrl_key(key: c_int) -> c_int {
    c_int::from(is_ctrl_key(key))
}

/// FFI: Check if key is an escape key (edit mode).
#[no_mangle]
pub extern "C" fn rs_edit_is_escape_key(key: c_int) -> c_int {
    c_int::from(is_escape_key(key))
}

/// FFI: Check if key is printable ASCII (edit mode).
#[no_mangle]
pub extern "C" fn rs_edit_is_printable_ascii(key: c_int) -> c_int {
    c_int::from(is_printable_ascii(key))
}

/// FFI: Check if key is a special (negative) key code (edit mode).
#[no_mangle]
pub extern "C" fn rs_edit_is_special_key(key: c_int) -> c_int {
    c_int::from(is_special_key(key))
}

/// FFI: Get navigation direction for a key (returns -1 if not a nav key) (edit mode).
#[no_mangle]
pub const extern "C" fn rs_edit_get_nav_direction(key: c_int) -> c_int {
    match get_nav_direction(key) {
        Some(NavDirection::Left) => 0,
        Some(NavDirection::Right) => 1,
        Some(NavDirection::Up) => 2,
        Some(NavDirection::Down) => 3,
        Some(NavDirection::Home) => 4,
        Some(NavDirection::End) => 5,
        Some(NavDirection::PageUp) => 6,
        Some(NavDirection::PageDown) => 7,
        None => -1,
    }
}

/// FFI: Classify a key for insert mode (returns `KeyAction` as int).
#[no_mangle]
pub const extern "C" fn rs_edit_classify_key(key: c_int) -> c_int {
    match classify_key(key) {
        KeyAction::InsertChar => 0,
        KeyAction::Delete => 1,
        KeyAction::Navigate => 2,
        KeyAction::Completion => 3,
        KeyAction::ModeChange => 4,
        KeyAction::Special => 5,
        KeyAction::Ignore => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_constants() {
        assert_eq!(ascii::TAB, 9);
        assert_eq!(ascii::NL, 10);
        assert_eq!(ascii::CAR, 13);
        assert_eq!(ascii::ESC, 27);
        assert_eq!(ascii::CTRL_A, 1);
        assert_eq!(ascii::CTRL_Z, 26);
    }

    #[test]
    fn test_is_arrow_key() {
        assert!(is_arrow_key(special_keys::K_UP));
        assert!(is_arrow_key(special_keys::K_DOWN));
        assert!(is_arrow_key(special_keys::K_LEFT));
        assert!(is_arrow_key(special_keys::K_RIGHT));
        assert!(!is_arrow_key(special_keys::K_HOME));
        assert!(!is_arrow_key(i32::from(b'a')));
    }

    #[test]
    fn test_is_navigation_key() {
        assert!(is_navigation_key(special_keys::K_UP));
        assert!(is_navigation_key(special_keys::K_HOME));
        assert!(is_navigation_key(special_keys::K_END));
        assert!(is_navigation_key(special_keys::K_PAGEUP));
        assert!(!is_navigation_key(special_keys::K_BS));
    }

    #[test]
    fn test_is_delete_key() {
        assert!(is_delete_key(special_keys::K_BS));
        assert!(is_delete_key(special_keys::K_DEL));
        assert!(is_delete_key(ascii::CTRL_H));
        assert!(!is_delete_key(special_keys::K_UP));
    }

    #[test]
    fn test_is_enter_key() {
        assert!(is_enter_key(ascii::CAR));
        assert!(is_enter_key(ascii::NL));
        assert!(is_enter_key(special_keys::K_KENTER));
        assert!(!is_enter_key(ascii::TAB));
    }

    #[test]
    fn test_is_tab_key() {
        assert!(is_tab_key(ascii::TAB));
        assert!(is_tab_key(special_keys::K_TAB));
        assert!(is_tab_key(special_keys::K_S_TAB));
        assert!(!is_tab_key(ascii::CAR));
    }

    #[test]
    fn test_is_ctrl_key() {
        assert!(is_ctrl_key(ascii::CTRL_A));
        assert!(is_ctrl_key(ascii::CTRL_Z));
        assert!(is_ctrl_key(0));
        assert!(!is_ctrl_key(32));
        assert!(!is_ctrl_key(i32::from(b'a')));
    }

    #[test]
    fn test_is_escape_key() {
        assert!(is_escape_key(ascii::ESC));
        assert!(is_escape_key(ascii::CTRL_C));
        assert!(!is_escape_key(ascii::CTRL_A));
    }

    #[test]
    fn test_is_printable_ascii() {
        assert!(is_printable_ascii(i32::from(b' ')));
        assert!(is_printable_ascii(i32::from(b'a')));
        assert!(is_printable_ascii(i32::from(b'~')));
        assert!(!is_printable_ascii(0));
        assert!(!is_printable_ascii(31));
        assert!(!is_printable_ascii(127));
    }

    #[test]
    fn test_is_special_key() {
        assert!(is_special_key(special_keys::K_UP));
        assert!(is_special_key(special_keys::K_BS));
        assert!(!is_special_key(i32::from(b'a')));
        assert!(!is_special_key(0));
    }

    #[test]
    fn test_get_nav_direction() {
        assert_eq!(get_nav_direction(special_keys::K_LEFT), Some(NavDirection::Left));
        assert_eq!(get_nav_direction(special_keys::K_RIGHT), Some(NavDirection::Right));
        assert_eq!(get_nav_direction(special_keys::K_UP), Some(NavDirection::Up));
        assert_eq!(get_nav_direction(special_keys::K_DOWN), Some(NavDirection::Down));
        assert_eq!(get_nav_direction(special_keys::K_HOME), Some(NavDirection::Home));
        assert_eq!(get_nav_direction(special_keys::K_END), Some(NavDirection::End));
        assert_eq!(get_nav_direction(special_keys::K_PAGEUP), Some(NavDirection::PageUp));
        assert_eq!(get_nav_direction(special_keys::K_PAGEDOWN), Some(NavDirection::PageDown));
        assert_eq!(get_nav_direction(i32::from(b'a')), None);
    }

    #[test]
    fn test_classify_key() {
        // Escape -> mode change
        assert_eq!(classify_key(ascii::ESC), KeyAction::ModeChange);

        // Delete keys
        assert_eq!(classify_key(special_keys::K_BS), KeyAction::Delete);
        assert_eq!(classify_key(special_keys::K_DEL), KeyAction::Delete);
        assert_eq!(classify_key(ascii::CTRL_W), KeyAction::Delete);
        assert_eq!(classify_key(ascii::CTRL_U), KeyAction::Delete);

        // Navigation
        assert_eq!(classify_key(special_keys::K_UP), KeyAction::Navigate);
        assert_eq!(classify_key(special_keys::K_HOME), KeyAction::Navigate);

        // Completion
        assert_eq!(classify_key(ascii::CTRL_N), KeyAction::Completion);
        assert_eq!(classify_key(ascii::CTRL_P), KeyAction::Completion);
        assert_eq!(classify_key(ascii::CTRL_X), KeyAction::Completion);

        // Special operations
        assert_eq!(classify_key(ascii::CTRL_R), KeyAction::Special);
        assert_eq!(classify_key(ascii::CTRL_K), KeyAction::Special);
        assert_eq!(classify_key(ascii::CTRL_V), KeyAction::Special);

        // Normal characters
        assert_eq!(classify_key(i32::from(b'a')), KeyAction::InsertChar);
        assert_eq!(classify_key(i32::from(b' ')), KeyAction::InsertChar);
        assert_eq!(classify_key(ascii::TAB), KeyAction::InsertChar);
        assert_eq!(classify_key(ascii::CAR), KeyAction::InsertChar);
    }

    #[test]
    fn test_backspace_mode() {
        assert_eq!(BackspaceMode::from_int(0), Some(BackspaceMode::Char));
        assert_eq!(BackspaceMode::from_int(1), Some(BackspaceMode::Word));
        assert_eq!(BackspaceMode::from_int(2), Some(BackspaceMode::Line));
        assert_eq!(BackspaceMode::from_int(3), None);
    }

    #[test]
    fn test_special_key_values_are_negative() {
        // All special keys should be negative (use runtime check to avoid const optimization)
        let keys = [
            special_keys::K_UP,
            special_keys::K_DOWN,
            special_keys::K_LEFT,
            special_keys::K_RIGHT,
            special_keys::K_BS,
            special_keys::K_DEL,
            special_keys::K_HOME,
            special_keys::K_END,
        ];
        for key in keys {
            assert!(key < 0, "Expected negative key code, got {key}");
        }
    }
}
