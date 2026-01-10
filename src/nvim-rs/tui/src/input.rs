//! TUI input handling
//!
//! This module contains types and constants for terminal input handling,
//! including keyboard encoding protocols and input state management.

use std::ffi::c_int;

// ============================================================================
// Key Encoding Protocol
// ============================================================================

/// Key encoding protocol used by the terminal
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyEncoding {
    /// Legacy key encoding (default terminal behavior)
    #[default]
    Legacy = 0,
    /// Kitty keyboard protocol
    Kitty = 1,
    /// Xterm modifyOtherKeys protocol
    Xterm = 2,
}

impl KeyEncoding {
    /// Convert from C int
    #[must_use]
    pub fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Legacy,
            1 => Self::Kitty,
            2 => Self::Xterm,
            _ => Self::Legacy,
        }
    }
}

// ============================================================================
// Input Handle Types
// ============================================================================

/// Opaque handle to TermInput struct in C
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermInputHandle(*mut std::ffi::c_void);

impl TermInputHandle {
    /// Create a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to termkey struct from libtermkey
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermKeyHandle(*mut std::ffi::c_void);

impl TermKeyHandle {
    /// Create a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// ============================================================================
// Paste Bracket Constants
// ============================================================================

/// Bracketed paste start sequence: ESC [ 2 0 0 ~
pub const BRACKETED_PASTE_START: &[u8] = b"\x1b[200~";

/// Bracketed paste end sequence: ESC [ 2 0 1 ~
pub const BRACKETED_PASTE_END: &[u8] = b"\x1b[201~";

/// Length of bracketed paste start sequence
pub const BRACKETED_PASTE_START_LEN: usize = 6;

/// Length of bracketed paste end sequence
pub const BRACKETED_PASTE_END_LEN: usize = 6;

// ============================================================================
// Kitty Key Constants (matching input.c)
// ============================================================================

/// Kitty key: Escape
pub const KITTY_KEY_ESCAPE: u32 = 27;
/// Kitty key: Enter/Return
pub const KITTY_KEY_ENTER: u32 = 13;
/// Kitty key: Tab
pub const KITTY_KEY_TAB: u32 = 9;
/// Kitty key: Backspace
pub const KITTY_KEY_BACKSPACE: u32 = 127;
/// Kitty key: Insert
pub const KITTY_KEY_INSERT: u32 = 57348;
/// Kitty key: Delete
pub const KITTY_KEY_DELETE: u32 = 57349;
/// Kitty key: Left arrow
pub const KITTY_KEY_LEFT: u32 = 57350;
/// Kitty key: Right arrow
pub const KITTY_KEY_RIGHT: u32 = 57351;
/// Kitty key: Up arrow
pub const KITTY_KEY_UP: u32 = 57352;
/// Kitty key: Down arrow
pub const KITTY_KEY_DOWN: u32 = 57353;
/// Kitty key: Page Up
pub const KITTY_KEY_PAGE_UP: u32 = 57354;
/// Kitty key: Page Down
pub const KITTY_KEY_PAGE_DOWN: u32 = 57355;
/// Kitty key: Home
pub const KITTY_KEY_HOME: u32 = 57356;
/// Kitty key: End
pub const KITTY_KEY_END: u32 = 57357;
/// Kitty key: Caps Lock
pub const KITTY_KEY_CAPS_LOCK: u32 = 57358;
/// Kitty key: Scroll Lock
pub const KITTY_KEY_SCROLL_LOCK: u32 = 57359;
/// Kitty key: Num Lock
pub const KITTY_KEY_NUM_LOCK: u32 = 57360;
/// Kitty key: Print Screen
pub const KITTY_KEY_PRINT_SCREEN: u32 = 57361;
/// Kitty key: Pause
pub const KITTY_KEY_PAUSE: u32 = 57362;
/// Kitty key: Menu
pub const KITTY_KEY_MENU: u32 = 57363;

// Kitty function keys (F1-F35)
/// Kitty key: F1
pub const KITTY_KEY_F1: u32 = 57364;
/// Kitty key: F2
pub const KITTY_KEY_F2: u32 = 57365;
/// Kitty key: F3
pub const KITTY_KEY_F3: u32 = 57366;
/// Kitty key: F4
pub const KITTY_KEY_F4: u32 = 57367;
/// Kitty key: F5
pub const KITTY_KEY_F5: u32 = 57368;
/// Kitty key: F6
pub const KITTY_KEY_F6: u32 = 57369;
/// Kitty key: F7
pub const KITTY_KEY_F7: u32 = 57370;
/// Kitty key: F8
pub const KITTY_KEY_F8: u32 = 57371;
/// Kitty key: F9
pub const KITTY_KEY_F9: u32 = 57372;
/// Kitty key: F10
pub const KITTY_KEY_F10: u32 = 57373;
/// Kitty key: F11
pub const KITTY_KEY_F11: u32 = 57374;
/// Kitty key: F12
pub const KITTY_KEY_F12: u32 = 57375;

// Kitty modifier keys
/// Kitty key: Left Shift
pub const KITTY_KEY_LEFT_SHIFT: u32 = 57441;
/// Kitty key: Left Control
pub const KITTY_KEY_LEFT_CONTROL: u32 = 57442;
/// Kitty key: Left Alt
pub const KITTY_KEY_LEFT_ALT: u32 = 57443;
/// Kitty key: Left Super
pub const KITTY_KEY_LEFT_SUPER: u32 = 57444;
/// Kitty key: Left Hyper
pub const KITTY_KEY_LEFT_HYPER: u32 = 57445;
/// Kitty key: Left Meta
pub const KITTY_KEY_LEFT_META: u32 = 57446;
/// Kitty key: Right Shift
pub const KITTY_KEY_RIGHT_SHIFT: u32 = 57447;
/// Kitty key: Right Control
pub const KITTY_KEY_RIGHT_CONTROL: u32 = 57448;
/// Kitty key: Right Alt
pub const KITTY_KEY_RIGHT_ALT: u32 = 57449;
/// Kitty key: Right Super
pub const KITTY_KEY_RIGHT_SUPER: u32 = 57450;
/// Kitty key: Right Hyper
pub const KITTY_KEY_RIGHT_HYPER: u32 = 57451;
/// Kitty key: Right Meta
pub const KITTY_KEY_RIGHT_META: u32 = 57452;

// Kitty numpad keys
/// Kitty key: Keypad 0
pub const KITTY_KEY_KP_0: u32 = 57399;
/// Kitty key: Keypad 1
pub const KITTY_KEY_KP_1: u32 = 57400;
/// Kitty key: Keypad 2
pub const KITTY_KEY_KP_2: u32 = 57401;
/// Kitty key: Keypad 3
pub const KITTY_KEY_KP_3: u32 = 57402;
/// Kitty key: Keypad 4
pub const KITTY_KEY_KP_4: u32 = 57403;
/// Kitty key: Keypad 5
pub const KITTY_KEY_KP_5: u32 = 57404;
/// Kitty key: Keypad 6
pub const KITTY_KEY_KP_6: u32 = 57405;
/// Kitty key: Keypad 7
pub const KITTY_KEY_KP_7: u32 = 57406;
/// Kitty key: Keypad 8
pub const KITTY_KEY_KP_8: u32 = 57407;
/// Kitty key: Keypad 9
pub const KITTY_KEY_KP_9: u32 = 57408;
/// Kitty key: Keypad Decimal
pub const KITTY_KEY_KP_DECIMAL: u32 = 57409;
/// Kitty key: Keypad Divide
pub const KITTY_KEY_KP_DIVIDE: u32 = 57410;
/// Kitty key: Keypad Multiply
pub const KITTY_KEY_KP_MULTIPLY: u32 = 57411;
/// Kitty key: Keypad Subtract
pub const KITTY_KEY_KP_SUBTRACT: u32 = 57412;
/// Kitty key: Keypad Add
pub const KITTY_KEY_KP_ADD: u32 = 57413;
/// Kitty key: Keypad Enter
pub const KITTY_KEY_KP_ENTER: u32 = 57414;
/// Kitty key: Keypad Equal
pub const KITTY_KEY_KP_EQUAL: u32 = 57415;

// ============================================================================
// Input Helper Functions
// ============================================================================

/// Check if a key code is a Kitty special key (not a regular character)
///
/// Special keys have codes >= 57344
#[no_mangle]
pub extern "C" fn rs_is_kitty_special_key(keycode: u32) -> c_int {
    c_int::from(keycode >= 57344)
}

/// Check if a key code is a Kitty function key (F1-F35)
#[no_mangle]
pub extern "C" fn rs_is_kitty_function_key(keycode: u32) -> c_int {
    c_int::from((57364..=57398).contains(&keycode))
}

/// Check if a key code is a Kitty modifier key (Shift, Ctrl, Alt, etc.)
#[no_mangle]
pub extern "C" fn rs_is_kitty_modifier_key(keycode: u32) -> c_int {
    c_int::from((KITTY_KEY_LEFT_SHIFT..=KITTY_KEY_RIGHT_META).contains(&keycode))
}

/// Check if a key code is a Kitty keypad key
#[no_mangle]
pub extern "C" fn rs_is_kitty_keypad_key(keycode: u32) -> c_int {
    c_int::from((KITTY_KEY_KP_0..=KITTY_KEY_KP_EQUAL).contains(&keycode))
}

/// Check if a key code is a Kitty navigation key (arrows, home, end, etc.)
#[no_mangle]
pub extern "C" fn rs_is_kitty_navigation_key(keycode: u32) -> c_int {
    c_int::from((KITTY_KEY_INSERT..=KITTY_KEY_MENU).contains(&keycode))
}

/// Get the function key number (1-12+) from a Kitty key code
///
/// Returns 0 if not a function key.
#[no_mangle]
pub extern "C" fn rs_kitty_function_key_number(keycode: u32) -> c_int {
    if (KITTY_KEY_F1..=KITTY_KEY_F12).contains(&keycode) {
        (keycode - KITTY_KEY_F1 + 1) as c_int
    } else if (57376..=57398).contains(&keycode) {
        // F13-F35
        (keycode - 57376 + 13) as c_int
    } else {
        0
    }
}

/// Get the keypad digit (0-9) from a Kitty key code
///
/// Returns -1 if not a keypad digit.
#[no_mangle]
pub extern "C" fn rs_kitty_keypad_digit(keycode: u32) -> c_int {
    if (KITTY_KEY_KP_0..=KITTY_KEY_KP_9).contains(&keycode) {
        (keycode - KITTY_KEY_KP_0) as c_int
    } else {
        -1
    }
}

/// Check if a sequence starts with the bracketed paste start sequence
///
/// # Safety
///
/// `data` must point to at least `len` valid bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_is_bracketed_paste_start(data: *const u8, len: usize) -> c_int {
    if data.is_null() || len < BRACKETED_PASTE_START_LEN {
        return 0;
    }

    let slice = std::slice::from_raw_parts(data, BRACKETED_PASTE_START_LEN);
    c_int::from(slice == BRACKETED_PASTE_START)
}

/// Check if a sequence starts with the bracketed paste end sequence
///
/// # Safety
///
/// `data` must point to at least `len` valid bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_is_bracketed_paste_end(data: *const u8, len: usize) -> c_int {
    if data.is_null() || len < BRACKETED_PASTE_END_LEN {
        return 0;
    }

    let slice = std::slice::from_raw_parts(data, BRACKETED_PASTE_END_LEN);
    c_int::from(slice == BRACKETED_PASTE_END)
}

/// Parse a CSI sequence parameter as an integer
///
/// Returns the parsed value, or default_val if parsing fails.
/// Updates `consumed` with the number of bytes consumed.
///
/// # Safety
///
/// `data` must point to at least `len` valid bytes.
/// `consumed` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_csi_param(
    data: *const u8,
    len: usize,
    default_val: c_int,
    consumed: *mut usize,
) -> c_int {
    if data.is_null() || len == 0 {
        if !consumed.is_null() {
            *consumed = 0;
        }
        return default_val;
    }

    let slice = std::slice::from_raw_parts(data, len);
    let mut value: c_int = 0;
    let mut i = 0usize;
    let mut found_digit = false;

    while i < slice.len() {
        let c = slice[i];
        if c.is_ascii_digit() {
            value = value * 10 + (c - b'0') as c_int;
            found_digit = true;
            i += 1;
        } else {
            break;
        }
    }

    if !consumed.is_null() {
        *consumed = i;
    }

    if found_digit {
        value
    } else {
        default_val
    }
}

/// Check if a byte is a CSI parameter byte (0-9, :, ;)
#[no_mangle]
pub extern "C" fn rs_is_csi_param_byte(byte: u8) -> c_int {
    c_int::from(byte.is_ascii_digit() || byte == b':' || byte == b';')
}

/// Check if a byte is a CSI intermediate byte (0x20-0x2F)
#[no_mangle]
pub extern "C" fn rs_is_csi_intermediate_byte(byte: u8) -> c_int {
    c_int::from((0x20..=0x2F).contains(&byte))
}

/// Check if a byte is a CSI final byte (0x40-0x7E)
#[no_mangle]
pub extern "C" fn rs_is_csi_final_byte(byte: u8) -> c_int {
    c_int::from((0x40..=0x7E).contains(&byte))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_encoding_from_c_int() {
        assert_eq!(KeyEncoding::from_c_int(0), KeyEncoding::Legacy);
        assert_eq!(KeyEncoding::from_c_int(1), KeyEncoding::Kitty);
        assert_eq!(KeyEncoding::from_c_int(2), KeyEncoding::Xterm);
        assert_eq!(KeyEncoding::from_c_int(99), KeyEncoding::Legacy);
    }

    #[test]
    fn test_bracketed_paste_sequences() {
        assert_eq!(BRACKETED_PASTE_START, b"\x1b[200~");
        assert_eq!(BRACKETED_PASTE_END, b"\x1b[201~");
        assert_eq!(BRACKETED_PASTE_START_LEN, BRACKETED_PASTE_START.len());
        assert_eq!(BRACKETED_PASTE_END_LEN, BRACKETED_PASTE_END.len());
    }

    #[test]
    fn test_key_encoding_default() {
        assert_eq!(KeyEncoding::default(), KeyEncoding::Legacy);
    }

    #[test]
    fn test_kitty_key_constants() {
        assert_eq!(KITTY_KEY_ESCAPE, 27);
        assert_eq!(KITTY_KEY_ENTER, 13);
        assert_eq!(KITTY_KEY_TAB, 9);
        assert_eq!(KITTY_KEY_BACKSPACE, 127);
        // Verify function keys are sequential
        assert_eq!(KITTY_KEY_F2, KITTY_KEY_F1 + 1);
        assert_eq!(KITTY_KEY_F12, KITTY_KEY_F1 + 11);
    }

    #[test]
    fn test_kitty_keypad_constants() {
        // Verify keypad keys are sequential
        assert_eq!(KITTY_KEY_KP_1, KITTY_KEY_KP_0 + 1);
        assert_eq!(KITTY_KEY_KP_9, KITTY_KEY_KP_0 + 9);
    }
}
