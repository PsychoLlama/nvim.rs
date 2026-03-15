//! Keyboard input handling for `VTerm`
//!
//! This module provides keyboard input encoding for terminal emulation,
//! including special keys, function keys, keypad keys, and modifiers.

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::fn_params_excessive_bools)]

use std::ffi::c_int;

// =============================================================================
// Modifier Keys
// =============================================================================

/// Key modifiers
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum VTermModifier {
    #[default]
    None = 0x00,
    Shift = 0x01,
    Alt = 0x02,
    Ctrl = 0x04,
}

/// Combined modifier flags
pub const VTERM_MOD_NONE: u8 = 0x00;
pub const VTERM_MOD_SHIFT: u8 = 0x01;
pub const VTERM_MOD_ALT: u8 = 0x02;
pub const VTERM_MOD_CTRL: u8 = 0x04;
pub const VTERM_ALL_MODS_MASK: u8 = 0x07;

// =============================================================================
// Special Keys
// =============================================================================

/// Special terminal keys
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VTermKey {
    None = 0,

    Enter = 1,
    Tab = 2,
    Backspace = 3,
    Escape = 4,

    Up = 5,
    Down = 6,
    Left = 7,
    Right = 8,

    Ins = 9,
    Del = 10,
    Home = 11,
    End = 12,
    PageUp = 13,
    PageDown = 14,

    // Function keys start at 256
    Function0 = 256,
    // FunctionMax = 511 (Function0 + 255)

    // Keypad keys start at 512
    Kp0 = 512,
    Kp1 = 513,
    Kp2 = 514,
    Kp3 = 515,
    Kp4 = 516,
    Kp5 = 517,
    Kp6 = 518,
    Kp7 = 519,
    Kp8 = 520,
    Kp9 = 521,
    KpMult = 522,
    KpPlus = 523,
    KpComma = 524,
    KpMinus = 525,
    KpPeriod = 526,
    KpDivide = 527,
    KpEnter = 528,
    KpEqual = 529,

    Max = 530,
}

/// Function key constant base
pub const VTERM_KEY_FUNCTION_0: u16 = 256;
/// Maximum function key
pub const VTERM_KEY_FUNCTION_MAX: u16 = 511;
/// Keypad 0 key base
pub const VTERM_KEY_KP_0: u16 = 512;

/// Get function key value
#[inline]
pub const fn vterm_key_function(n: u8) -> u16 {
    VTERM_KEY_FUNCTION_0 + n as u16
}

impl VTermKey {
    /// Create a function key (F1-F12, etc.)
    ///
    /// Note: This returns Function0 + n, which may not correspond to a defined
    /// enum variant for n > 0. The value is still valid for encoding purposes.
    #[inline]
    pub const fn function(n: u8) -> u16 {
        VTERM_KEY_FUNCTION_0 + n as u16
    }

    /// Check if this is a function key
    #[inline]
    pub const fn is_function(&self) -> bool {
        let v = *self as u16;
        v >= VTERM_KEY_FUNCTION_0 && v <= VTERM_KEY_FUNCTION_MAX
    }

    /// Get function key number (if this is a function key)
    #[inline]
    pub const fn function_number(&self) -> Option<u8> {
        let v = *self as u16;
        if v >= VTERM_KEY_FUNCTION_0 && v <= VTERM_KEY_FUNCTION_MAX {
            Some((v - VTERM_KEY_FUNCTION_0) as u8)
        } else {
            None
        }
    }

    /// Check if this is a keypad key
    #[inline]
    pub const fn is_keypad(&self) -> bool {
        let v = *self as u16;
        v >= VTERM_KEY_KP_0 && v < Self::Max as u16
    }

    /// Get keypad key index (if this is a keypad key)
    #[inline]
    pub const fn keypad_index(&self) -> Option<u8> {
        let v = *self as u16;
        if v >= VTERM_KEY_KP_0 && v < Self::Max as u16 {
            Some((v - VTERM_KEY_KP_0) as u8)
        } else {
            None
        }
    }
}

// =============================================================================
// Keycode Types
// =============================================================================

/// Internal keycode type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeycodeType {
    /// No keycode
    None,
    /// Literal character
    Literal,
    /// Tab key (special handling for Shift-Tab)
    Tab,
    /// Enter key (special handling for newline mode)
    Enter,
    /// SS3 escape sequence
    Ss3,
    /// CSI escape sequence
    Csi,
    /// CSI cursor key (uses SS3 in application cursor mode)
    CsiCursor,
    /// CSI with numeric parameter
    CsiNum,
    /// Keypad key (uses SS3 in application keypad mode)
    Keypad,
}

/// Keycode entry
#[derive(Clone, Copy, Debug)]
pub struct Keycode {
    /// Type of keycode
    pub keycode_type: KeycodeType,
    /// Literal character or CSI final byte
    pub literal: u8,
    /// CSI numeric parameter (for `CsiNum`) or SS3 char (for `Keypad`)
    pub csinum: u16,
}

impl Keycode {
    const fn none() -> Self {
        Self {
            keycode_type: KeycodeType::None,
            literal: 0,
            csinum: 0,
        }
    }

    const fn literal(c: u8) -> Self {
        Self {
            keycode_type: KeycodeType::Literal,
            literal: c,
            csinum: 0,
        }
    }

    const fn tab() -> Self {
        Self {
            keycode_type: KeycodeType::Tab,
            literal: b'\t',
            csinum: 0,
        }
    }

    const fn enter() -> Self {
        Self {
            keycode_type: KeycodeType::Enter,
            literal: b'\r',
            csinum: 0,
        }
    }

    const fn ss3(c: u8) -> Self {
        Self {
            keycode_type: KeycodeType::Ss3,
            literal: c,
            csinum: 0,
        }
    }

    const fn csi_cursor(c: u8) -> Self {
        Self {
            keycode_type: KeycodeType::CsiCursor,
            literal: c,
            csinum: 0,
        }
    }

    const fn csi_num(c: u8, num: u16) -> Self {
        Self {
            keycode_type: KeycodeType::CsiNum,
            literal: c,
            csinum: num,
        }
    }

    const fn keypad(literal: u8, ss3_char: u8) -> Self {
        Self {
            keycode_type: KeycodeType::Keypad,
            literal,
            csinum: ss3_char as u16,
        }
    }

    const fn keypad_csiu(codepoint: u16, ss3_char: u8) -> Self {
        Self {
            keycode_type: KeycodeType::Keypad,
            literal: ss3_char,
            csinum: codepoint,
        }
    }
}

// =============================================================================
// Keycode Tables
// =============================================================================

/// Basic keycodes (NONE through PAGEDOWN)
const KEYCODES: [Keycode; 15] = [
    Keycode::none(),           // NONE
    Keycode::enter(),          // ENTER
    Keycode::tab(),            // TAB
    Keycode::literal(0x7f),    // BACKSPACE == ASCII DEL
    Keycode::literal(0x1b),    // ESCAPE
    Keycode::csi_cursor(b'A'), // UP
    Keycode::csi_cursor(b'B'), // DOWN
    Keycode::csi_cursor(b'D'), // LEFT
    Keycode::csi_cursor(b'C'), // RIGHT
    Keycode::csi_num(b'~', 2), // INS
    Keycode::csi_num(b'~', 3), // DEL
    Keycode::csi_cursor(b'H'), // HOME
    Keycode::csi_cursor(b'F'), // END
    Keycode::csi_num(b'~', 5), // PAGEUP
    Keycode::csi_num(b'~', 6), // PAGEDOWN
];

/// Function key keycodes (F1-F12)
const KEYCODES_FN: [Keycode; 13] = [
    Keycode::none(),            // F0 - shouldn't happen
    Keycode::ss3(b'P'),         // F1
    Keycode::ss3(b'Q'),         // F2
    Keycode::ss3(b'R'),         // F3
    Keycode::ss3(b'S'),         // F4
    Keycode::csi_num(b'~', 15), // F5
    Keycode::csi_num(b'~', 17), // F6
    Keycode::csi_num(b'~', 18), // F7
    Keycode::csi_num(b'~', 19), // F8
    Keycode::csi_num(b'~', 20), // F9
    Keycode::csi_num(b'~', 21), // F10
    Keycode::csi_num(b'~', 23), // F11
    Keycode::csi_num(b'~', 24), // F12
];

/// Keypad keycodes (normal mode)
const KEYCODES_KP: [Keycode; 18] = [
    Keycode::keypad(b'0', b'p'),  // KP_0
    Keycode::keypad(b'1', b'q'),  // KP_1
    Keycode::keypad(b'2', b'r'),  // KP_2
    Keycode::keypad(b'3', b's'),  // KP_3
    Keycode::keypad(b'4', b't'),  // KP_4
    Keycode::keypad(b'5', b'u'),  // KP_5
    Keycode::keypad(b'6', b'v'),  // KP_6
    Keycode::keypad(b'7', b'w'),  // KP_7
    Keycode::keypad(b'8', b'x'),  // KP_8
    Keycode::keypad(b'9', b'y'),  // KP_9
    Keycode::keypad(b'*', b'j'),  // KP_MULT
    Keycode::keypad(b'+', b'k'),  // KP_PLUS
    Keycode::keypad(b',', b'l'),  // KP_COMMA
    Keycode::keypad(b'-', b'm'),  // KP_MINUS
    Keycode::keypad(b'.', b'n'),  // KP_PERIOD
    Keycode::keypad(b'/', b'o'),  // KP_DIVIDE
    Keycode::keypad(b'\n', b'M'), // KP_ENTER
    Keycode::keypad(b'=', b'X'),  // KP_EQUAL
];

/// Keypad keycodes (CSI u mode - for disambiguate flag)
const KEYCODES_KP_CSIU: [Keycode; 18] = [
    Keycode::keypad_csiu(57399, b'p'), // KP_0
    Keycode::keypad_csiu(57400, b'q'), // KP_1
    Keycode::keypad_csiu(57401, b'r'), // KP_2
    Keycode::keypad_csiu(57402, b's'), // KP_3
    Keycode::keypad_csiu(57403, b't'), // KP_4
    Keycode::keypad_csiu(57404, b'u'), // KP_5
    Keycode::keypad_csiu(57405, b'v'), // KP_6
    Keycode::keypad_csiu(57406, b'w'), // KP_7
    Keycode::keypad_csiu(57407, b'x'), // KP_8
    Keycode::keypad_csiu(57408, b'y'), // KP_9
    Keycode::keypad_csiu(57411, b'j'), // KP_MULT
    Keycode::keypad_csiu(57413, b'k'), // KP_PLUS
    Keycode::keypad_csiu(57416, b'l'), // KP_COMMA
    Keycode::keypad_csiu(57412, b'm'), // KP_MINUS
    Keycode::keypad_csiu(57409, b'n'), // KP_PERIOD
    Keycode::keypad_csiu(57410, b'o'), // KP_DIVIDE
    Keycode::keypad_csiu(57414, b'M'), // KP_ENTER
    Keycode::keypad_csiu(57415, b'X'), // KP_EQUAL
];

// =============================================================================
// Keycode Lookup
// =============================================================================

/// Look up the keycode for a key
///
/// Returns `None` if the key is invalid.
/// If `disambiguate` is true, uses CSI u codes for keypad keys.
pub fn lookup_keycode(key: VTermKey, disambiguate: bool) -> Option<Keycode> {
    lookup_keycode_by_value(key as u16, disambiguate)
}

/// Look up the keycode for a key value
///
/// Returns `None` if the key is invalid.
/// If `disambiguate` is true, uses CSI u codes for keypad keys.
pub fn lookup_keycode_by_value(key_val: u16, disambiguate: bool) -> Option<Keycode> {
    if key_val < VTERM_KEY_FUNCTION_0 {
        // Basic keys
        let idx = key_val as usize;
        if idx < KEYCODES.len() {
            return Some(KEYCODES[idx]);
        }
    } else if key_val <= VTERM_KEY_FUNCTION_MAX {
        // Function keys
        let idx = (key_val - VTERM_KEY_FUNCTION_0) as usize;
        if idx < KEYCODES_FN.len() {
            return Some(KEYCODES_FN[idx]);
        }
    } else if key_val >= VTERM_KEY_KP_0 {
        // Keypad keys
        let idx = (key_val - VTERM_KEY_KP_0) as usize;
        if idx < KEYCODES_KP.len() {
            return if disambiguate {
                Some(KEYCODES_KP_CSIU[idx])
            } else {
                Some(KEYCODES_KP[idx])
            };
        }
    }

    None
}

// =============================================================================
// Keyboard Output Generation
// =============================================================================

/// Result of encoding a key press
#[derive(Clone, Debug)]
pub enum KeyOutput {
    /// Output literal bytes
    Literal(Vec<u8>),
    /// Output ESC + character (for Alt modifier)
    EscChar(u8),
    /// Output SS3 sequence: ESC O <char>
    Ss3(u8),
    /// Output CSI sequence: ESC [ <char>
    Csi(u8),
    /// Output CSI sequence with modifier: ESC [ 1 ; <mod> <char>
    CsiMod(u8, u8),
    /// Output CSI sequence with number: ESC [ <num> <char>
    CsiNum(u16, u8),
    /// Output CSI sequence with number and modifier: ESC [ <num> ; <mod> <char>
    CsiNumMod(u16, u8, u8),
    /// Output CSI u sequence: ESC [ <code> ; <mod> u
    CsiU(u32, u8),
    /// No output
    None,
}

/// Encode a Unicode character for keyboard input
///
/// # Arguments
/// * `codepoint` - The Unicode codepoint to encode
/// * `mods` - Modifier keys (shift, alt, ctrl)
/// * `disambiguate` - Whether to use CSI u encoding
///
/// # Returns
/// The encoded key output
pub fn encode_unichar(codepoint: u32, mods: u8, disambiguate: bool) -> KeyOutput {
    let mut c = codepoint;
    let mut mod_val = mods;

    // Determine if we should pass through without encoding
    let passthru = if c == u32::from(b' ') {
        // Space is passed through only when there are no modifiers (including shift)
        mod_val == VTERM_MOD_NONE
    } else {
        // Otherwise pass through when there are no modifiers (ignoring shift)
        (mod_val & !VTERM_MOD_SHIFT) == 0
    };

    if passthru {
        // Encode as UTF-8
        let mut buf = [0u8; 4];
        let len = encode_utf8(c, &mut buf);
        return KeyOutput::Literal(buf[..len].to_vec());
    }

    if disambiguate {
        // Always use unshifted codepoint
        if c >= u32::from(b'A') && c <= u32::from(b'Z') {
            c += u32::from(b'a' - b'A');
            mod_val |= VTERM_MOD_SHIFT;
        }
        return KeyOutput::CsiU(c, mod_val + 1);
    }

    // Handle Ctrl modifier
    if (mod_val & VTERM_MOD_CTRL) != 0 {
        c = ctrl_transform(c);
    }

    // Handle Alt modifier
    if (mod_val & VTERM_MOD_ALT) != 0 {
        KeyOutput::EscChar(c as u8)
    } else {
        KeyOutput::Literal(vec![c as u8])
    }
}

/// Encode a special key for keyboard input given a raw key value
///
/// This is like `encode_key` but accepts a raw u16 key value instead of a `VTermKey` enum.
/// Returns `KeyOutput::None` if the key value is invalid.
pub fn encode_key_raw(
    key_val: u16,
    mods: u8,
    disambiguate: bool,
    cursor_mode: bool,
    keypad_mode: bool,
    newline_mode: bool,
) -> KeyOutput {
    // SAFETY: VTermKey is repr(u16) and we handle unknown keys via lookup_keycode_by_value
    // We use Function0 as a stand-in for the raw value since encode_key only uses the
    // numeric value via lookup_keycode anyway.
    //
    // For VTERM_KEY_NONE (0), return None immediately.
    if key_val == VTermKey::None as u16 {
        return KeyOutput::None;
    }

    let Some(keycode) = lookup_keycode_by_value(key_val, disambiguate) else {
        return KeyOutput::None;
    };

    // Reconstruct a VTermKey for special handling in encode_key.
    // We need to know the key identity for Tab/Enter/Backspace disambiguation.
    // Use a helper that works directly with the keycode.
    encode_keycode(
        keycode,
        key_val,
        mods,
        disambiguate,
        cursor_mode,
        keypad_mode,
        newline_mode,
    )
}

/// Encode a special key for keyboard input
pub fn encode_key(
    key: VTermKey,
    mods: u8,
    disambiguate: bool,
    cursor_mode: bool,
    keypad_mode: bool,
    newline_mode: bool,
) -> KeyOutput {
    if key == VTermKey::None {
        return KeyOutput::None;
    }

    let Some(keycode) = lookup_keycode(key, disambiguate) else {
        return KeyOutput::None;
    };

    match keycode.keycode_type {
        KeycodeType::None => KeyOutput::None,

        KeycodeType::Tab => {
            if disambiguate {
                encode_literal_key(keycode.literal, key, mods, true)
            } else if mods == VTERM_MOD_SHIFT {
                // Shift-Tab is CSI Z
                KeyOutput::Csi(b'Z')
            } else if (mods & VTERM_MOD_SHIFT) != 0 {
                // Shift + other mods
                KeyOutput::CsiMod(b'Z', mods + 1)
            } else {
                encode_literal_key(keycode.literal, key, mods, false)
            }
        }

        KeycodeType::Enter => {
            if newline_mode {
                // Enter is CRLF in newline mode
                KeyOutput::Literal(vec![b'\r', b'\n'])
            } else {
                encode_literal_key(keycode.literal, key, mods, disambiguate)
            }
        }

        KeycodeType::Literal => encode_literal_key(keycode.literal, key, mods, disambiguate),

        KeycodeType::Ss3 => {
            if mods == 0 {
                KeyOutput::Ss3(keycode.literal)
            } else {
                // With modifiers, use CSI format
                KeyOutput::CsiMod(keycode.literal, mods + 1)
            }
        }

        KeycodeType::Csi => {
            if mods == 0 {
                KeyOutput::Csi(keycode.literal)
            } else {
                KeyOutput::CsiMod(keycode.literal, mods + 1)
            }
        }

        KeycodeType::CsiCursor => {
            if cursor_mode {
                // Use SS3 in application cursor mode
                if mods == 0 {
                    KeyOutput::Ss3(keycode.literal)
                } else {
                    KeyOutput::CsiMod(keycode.literal, mods + 1)
                }
            } else if mods == 0 {
                KeyOutput::Csi(keycode.literal)
            } else {
                KeyOutput::CsiMod(keycode.literal, mods + 1)
            }
        }

        KeycodeType::CsiNum => {
            if mods == 0 {
                KeyOutput::CsiNum(keycode.csinum, keycode.literal)
            } else {
                KeyOutput::CsiNumMod(keycode.csinum, keycode.literal, mods + 1)
            }
        }

        KeycodeType::Keypad => {
            if keypad_mode {
                // Application keypad mode - use SS3
                let ss3_char = keycode.csinum as u8;
                if mods == 0 {
                    KeyOutput::Ss3(ss3_char)
                } else {
                    KeyOutput::CsiMod(ss3_char, mods + 1)
                }
            } else {
                // Normal mode - send literal
                encode_literal_key(keycode.literal, key, mods, disambiguate)
            }
        }
    }
}

/// Internal implementation: encode a keycode given raw key value
///
/// This avoids needing to convert the raw u16 key value to a `VTermKey` enum.
fn encode_keycode(
    keycode: Keycode,
    key_val: u16,
    mods: u8,
    disambiguate: bool,
    cursor_mode: bool,
    keypad_mode: bool,
    newline_mode: bool,
) -> KeyOutput {
    match keycode.keycode_type {
        KeycodeType::None => KeyOutput::None,

        KeycodeType::Tab => {
            if disambiguate {
                encode_literal_key_raw(keycode.literal, key_val, mods, true)
            } else if mods == VTERM_MOD_SHIFT {
                KeyOutput::Csi(b'Z')
            } else if (mods & VTERM_MOD_SHIFT) != 0 {
                KeyOutput::CsiMod(b'Z', mods + 1)
            } else {
                encode_literal_key_raw(keycode.literal, key_val, mods, false)
            }
        }

        KeycodeType::Enter => {
            if newline_mode {
                KeyOutput::Literal(vec![b'\r', b'\n'])
            } else {
                encode_literal_key_raw(keycode.literal, key_val, mods, disambiguate)
            }
        }

        KeycodeType::Literal => {
            encode_literal_key_raw(keycode.literal, key_val, mods, disambiguate)
        }

        KeycodeType::Ss3 => {
            if mods == 0 {
                KeyOutput::Ss3(keycode.literal)
            } else {
                KeyOutput::CsiMod(keycode.literal, mods + 1)
            }
        }

        KeycodeType::Csi => {
            if mods == 0 {
                KeyOutput::Csi(keycode.literal)
            } else {
                KeyOutput::CsiMod(keycode.literal, mods + 1)
            }
        }

        KeycodeType::CsiCursor => {
            if cursor_mode {
                if mods == 0 {
                    KeyOutput::Ss3(keycode.literal)
                } else {
                    KeyOutput::CsiMod(keycode.literal, mods + 1)
                }
            } else if mods == 0 {
                KeyOutput::Csi(keycode.literal)
            } else {
                KeyOutput::CsiMod(keycode.literal, mods + 1)
            }
        }

        KeycodeType::CsiNum => {
            if mods == 0 {
                KeyOutput::CsiNum(keycode.csinum, keycode.literal)
            } else {
                KeyOutput::CsiNumMod(keycode.csinum, keycode.literal, mods + 1)
            }
        }

        KeycodeType::Keypad => {
            if keypad_mode {
                let ss3_char = keycode.csinum as u8;
                if mods == 0 {
                    KeyOutput::Ss3(ss3_char)
                } else {
                    KeyOutput::CsiMod(ss3_char, mods + 1)
                }
            } else {
                encode_literal_key_raw(keycode.literal, key_val, mods, disambiguate)
            }
        }
    }
}

/// Encode a literal key by raw key value (for use without `VTermKey` enum)
fn encode_literal_key_raw(literal: u8, key_val: u16, mods: u8, disambiguate: bool) -> KeyOutput {
    let mut use_csiu = disambiguate;

    // For Tab, Enter, Backspace without modifiers, don't use CSI u
    if use_csiu {
        let is_special = key_val == VTermKey::Tab as u16
            || key_val == VTermKey::Enter as u16
            || key_val == VTermKey::Backspace as u16;
        if is_special {
            use_csiu = mods != VTERM_MOD_NONE;
        }
    }

    if use_csiu {
        KeyOutput::CsiU(u32::from(literal), mods + 1)
    } else if (mods & VTERM_MOD_ALT) != 0 {
        KeyOutput::EscChar(literal)
    } else {
        KeyOutput::Literal(vec![literal])
    }
}

/// Encode a literal key with optional CSI u encoding
fn encode_literal_key(literal: u8, key: VTermKey, mods: u8, disambiguate: bool) -> KeyOutput {
    let mut use_csiu = disambiguate;

    // For certain keys without modifiers, don't use CSI u
    if use_csiu {
        match key {
            VTermKey::Tab | VTermKey::Enter | VTermKey::Backspace => {
                use_csiu = mods != VTERM_MOD_NONE;
            }
            _ => {}
        }
    }

    if use_csiu {
        KeyOutput::CsiU(u32::from(literal), mods + 1)
    } else if (mods & VTERM_MOD_ALT) != 0 {
        KeyOutput::EscChar(literal)
    } else {
        KeyOutput::Literal(vec![literal])
    }
}

/// Transform a character with Ctrl modifier
fn ctrl_transform(c: u32) -> u32 {
    match c {
        // Ctrl+2 is NUL to match Ctrl+@ (which is Shift+2 on US keyboards)
        // Ctrl+Space is also NUL for some reason
        0x32 | 0x20 => 0x00,
        // Ctrl+3 through Ctrl+7 are sequential starting from 0x1b
        0x33..=0x37 => 0x1b + c - 0x33,
        // Ctrl+8 is DEL
        0x38 => 0x7f,
        // Ctrl+/ is equivalent to Ctrl+_
        0x2f => 0x1f,
        // Standard control character mapping
        0x40..=0x7f => c & 0x1f,
        _ => c,
    }
}

/// Encode a codepoint as UTF-8
fn encode_utf8(c: u32, buf: &mut [u8; 4]) -> usize {
    if c < 0x80 {
        buf[0] = c as u8;
        1
    } else if c < 0x800 {
        buf[0] = (0xC0 | (c >> 6)) as u8;
        buf[1] = (0x80 | (c & 0x3F)) as u8;
        2
    } else if c < 0x1_0000 {
        buf[0] = (0xE0 | (c >> 12)) as u8;
        buf[1] = (0x80 | ((c >> 6) & 0x3F)) as u8;
        buf[2] = (0x80 | (c & 0x3F)) as u8;
        3
    } else {
        buf[0] = (0xF0 | (c >> 18)) as u8;
        buf[1] = (0x80 | ((c >> 12) & 0x3F)) as u8;
        buf[2] = (0x80 | ((c >> 6) & 0x3F)) as u8;
        buf[3] = (0x80 | (c & 0x3F)) as u8;
        4
    }
}

// =============================================================================
// Paste Bracket Sequences
// =============================================================================

/// Bracketed paste start sequence
pub const PASTE_START: &[u8] = b"\x1b[200~";

/// Bracketed paste end sequence
pub const PASTE_END: &[u8] = b"\x1b[201~";

// =============================================================================
// FFI Functions
// =============================================================================

/// Check if a key value is a function key
#[no_mangle]
pub extern "C" fn rs_vterm_key_is_function(key: c_int) -> c_int {
    let key_val = key as u16;
    c_int::from((VTERM_KEY_FUNCTION_0..=VTERM_KEY_FUNCTION_MAX).contains(&key_val))
}

/// Check if a key value is a keypad key
#[no_mangle]
pub extern "C" fn rs_vterm_key_is_keypad(key: c_int) -> c_int {
    let key_val = key as u16;
    c_int::from(key_val >= VTERM_KEY_KP_0 && key_val < VTermKey::Max as u16)
}

/// Get a function key value
#[no_mangle]
pub extern "C" fn rs_vterm_key_function(n: c_int) -> c_int {
    c_int::from(VTERM_KEY_FUNCTION_0 + n as u16)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_constants() {
        assert_eq!(VTERM_MOD_NONE, 0);
        assert_eq!(VTERM_MOD_SHIFT, 1);
        assert_eq!(VTERM_MOD_ALT, 2);
        assert_eq!(VTERM_MOD_CTRL, 4);
    }

    #[test]
    fn test_vterm_key_function() {
        let f1 = VTermKey::function(1);
        assert_eq!(f1, 257);
        assert!((VTERM_KEY_FUNCTION_0..=VTERM_KEY_FUNCTION_MAX).contains(&f1));

        let f12 = VTermKey::function(12);
        assert_eq!(f12, 268);
        assert!((VTERM_KEY_FUNCTION_0..=VTERM_KEY_FUNCTION_MAX).contains(&f12));

        // Test the enum method
        assert!(VTermKey::Function0.is_function());
        assert_eq!(VTermKey::Function0.function_number(), Some(0));
    }

    #[test]
    fn test_vterm_key_keypad() {
        let kp0 = VTermKey::Kp0;
        assert!(kp0.is_keypad());
        assert_eq!(kp0.keypad_index(), Some(0));

        let kp_enter = VTermKey::KpEnter;
        assert!(kp_enter.is_keypad());
    }

    #[test]
    fn test_lookup_keycode_basic() {
        let enter = lookup_keycode(VTermKey::Enter, false);
        assert!(enter.is_some());
        let enter = enter.unwrap();
        assert_eq!(enter.keycode_type, KeycodeType::Enter);
        assert_eq!(enter.literal, b'\r');
    }

    #[test]
    fn test_lookup_keycode_function() {
        let f1 = lookup_keycode_by_value(VTermKey::function(1), false);
        assert!(f1.is_some());
        let f1 = f1.unwrap();
        assert_eq!(f1.keycode_type, KeycodeType::Ss3);
        assert_eq!(f1.literal, b'P');

        let f5 = lookup_keycode_by_value(VTermKey::function(5), false);
        assert!(f5.is_some());
        let f5 = f5.unwrap();
        assert_eq!(f5.keycode_type, KeycodeType::CsiNum);
        assert_eq!(f5.csinum, 15);
    }

    #[test]
    fn test_lookup_keycode_cursor() {
        let up = lookup_keycode(VTermKey::Up, false);
        assert!(up.is_some());
        let up = up.unwrap();
        assert_eq!(up.keycode_type, KeycodeType::CsiCursor);
        assert_eq!(up.literal, b'A');
    }

    #[test]
    fn test_encode_unichar_passthrough() {
        let result = encode_unichar(u32::from(b'a'), VTERM_MOD_NONE, false);
        assert!(matches!(result, KeyOutput::Literal(ref v) if v == b"a"));
    }

    #[test]
    fn test_encode_unichar_with_shift() {
        // Shift alone should pass through
        let result = encode_unichar(u32::from(b'A'), VTERM_MOD_SHIFT, false);
        assert!(matches!(result, KeyOutput::Literal(ref v) if v == b"A"));
    }

    #[test]
    fn test_encode_unichar_with_alt() {
        let result = encode_unichar(u32::from(b'a'), VTERM_MOD_ALT, false);
        assert!(matches!(result, KeyOutput::EscChar(b'a')));
    }

    #[test]
    fn test_encode_unichar_disambiguate() {
        let result = encode_unichar(u32::from(b'a'), VTERM_MOD_CTRL, true);
        assert!(matches!(result, KeyOutput::CsiU(97, 5))); // 'a' = 97, mod + 1 = 5
    }

    #[test]
    fn test_ctrl_transform() {
        assert_eq!(ctrl_transform(0x20), 0x00); // Ctrl+Space = NUL
        assert_eq!(ctrl_transform(0x32), 0x00); // Ctrl+2 = NUL
        assert_eq!(ctrl_transform(0x33), 0x1b); // Ctrl+3 = ESC
        assert_eq!(ctrl_transform(0x38), 0x7f); // Ctrl+8 = DEL
        assert_eq!(ctrl_transform(0x2f), 0x1f); // Ctrl+/ = 0x1f
        assert_eq!(ctrl_transform(0x63), 0x03); // Ctrl+c = 0x03
        assert_eq!(ctrl_transform(0x43), 0x03); // Ctrl+C = 0x03
    }

    #[test]
    fn test_encode_utf8() {
        let mut buf = [0u8; 4];

        // ASCII
        assert_eq!(encode_utf8(0x41, &mut buf), 1);
        assert_eq!(buf[0], 0x41);

        // 2-byte
        assert_eq!(encode_utf8(0xE9, &mut buf), 2);
        assert_eq!(buf[..2], [0xC3, 0xA9]); // é

        // 3-byte
        assert_eq!(encode_utf8(0x4E2D, &mut buf), 3);
        assert_eq!(buf[..3], [0xE4, 0xB8, 0xAD]); // 中

        // 4-byte
        assert_eq!(encode_utf8(0x1F600, &mut buf), 4);
        assert_eq!(buf, [0xF0, 0x9F, 0x98, 0x80]); // 😀
    }

    #[test]
    fn test_encode_key_enter() {
        // Normal mode (linefeed)
        let result = encode_key(VTermKey::Enter, 0, false, false, false, false);
        assert!(matches!(result, KeyOutput::Literal(ref v) if v == b"\r"));

        // Newline mode
        let result = encode_key(VTermKey::Enter, 0, false, false, false, true);
        assert!(matches!(result, KeyOutput::Literal(ref v) if v == b"\r\n"));
    }

    #[test]
    fn test_encode_key_tab() {
        // Plain tab
        let result = encode_key(VTermKey::Tab, 0, false, false, false, false);
        assert!(matches!(result, KeyOutput::Literal(ref v) if v == b"\t"));

        // Shift-Tab
        let result = encode_key(VTermKey::Tab, VTERM_MOD_SHIFT, false, false, false, false);
        assert!(matches!(result, KeyOutput::Csi(b'Z')));
    }

    #[test]
    fn test_encode_key_cursor() {
        // Normal cursor mode
        let result = encode_key(VTermKey::Up, 0, false, false, false, false);
        assert!(matches!(result, KeyOutput::Csi(b'A')));

        // Application cursor mode
        let result = encode_key(VTermKey::Up, 0, false, true, false, false);
        assert!(matches!(result, KeyOutput::Ss3(b'A')));

        // With modifier
        let result = encode_key(VTermKey::Up, VTERM_MOD_SHIFT, false, false, false, false);
        assert!(matches!(result, KeyOutput::CsiMod(b'A', 2)));
    }

    #[test]
    fn test_ffi_functions() {
        assert_eq!(rs_vterm_key_is_function(256), 1); // F0
        assert_eq!(rs_vterm_key_is_function(257), 1); // F1
        assert_eq!(rs_vterm_key_is_function(100), 0); // Not a function key
        assert_eq!(rs_vterm_key_is_keypad(512), 1); // KP_0
        assert_eq!(rs_vterm_key_is_keypad(100), 0); // Not a keypad key
        assert_eq!(rs_vterm_key_function(1), 257); // F1
    }
}
