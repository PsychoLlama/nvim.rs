//! Character input functions
//!
//! This module provides Rust implementations of character input handling,
//! including functions for reading characters from the typeahead buffer
//! and translating special key sequences.

// Allow integer casts that are safe given the constraints of the input handling
// (character values and buffer indices fit in i32)
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use std::ffi::c_int;

// =============================================================================
// Constants
// =============================================================================

/// K_SPECIAL byte that marks a special key sequence
const K_SPECIAL: u8 = 0x80;

/// KS_SPECIAL - indicates an escaped K_SPECIAL
const KS_SPECIAL: u8 = 254;

/// KS_ZERO - indicates an escaped NUL
const KS_ZERO: u8 = 255;

/// KS_MODIFIER - indicates a modifier key
#[allow(dead_code)]
const KS_MODIFIER: u8 = 252;

/// KE_FILLER - filler byte for special sequences
const KE_FILLER: u8 = b'X';

/// NUL character
const NUL: u8 = 0;

// =============================================================================
// C FFI Declarations
// =============================================================================

extern "C" {
    // Global state accessors for old_char (vungetc/can_get_old_char)
    fn nvim_get_old_char() -> c_int;
    fn nvim_set_old_char(val: c_int);
    fn nvim_get_old_mod_mask() -> c_int;
    fn nvim_set_old_mod_mask(val: c_int);
    fn nvim_get_old_mouse_grid() -> c_int;
    fn nvim_set_old_mouse_grid(val: c_int);
    fn nvim_get_old_mouse_row() -> c_int;
    fn nvim_set_old_mouse_row(val: c_int);
    fn nvim_get_old_mouse_col() -> c_int;
    fn nvim_set_old_mouse_col(val: c_int);
    #[allow(dead_code)]
    fn nvim_get_old_keystuffed() -> c_int;
    fn nvim_set_old_keystuffed(val: c_int);

    // Global state accessors for mod_mask and mouse state
    fn nvim_get_mod_mask() -> c_int;
    fn nvim_set_mod_mask(val: c_int);
    fn nvim_get_mouse_grid() -> c_int;
    fn nvim_set_mouse_grid(val: c_int);
    fn nvim_get_mouse_row() -> c_int;
    fn nvim_set_mouse_row(val: c_int);
    fn nvim_get_mouse_col() -> c_int;
    fn nvim_set_mouse_col(val: c_int);

    // Keystuffed state
    fn nvim_get_keystuffed() -> c_int;

    // Stuff buffer check (used for can_get_old_char logic, but wrapped in C)
    #[allow(dead_code)]
    fn rs_stuff_empty() -> c_int;

    // Can get old char wrapper
    fn nvim_can_get_old_char() -> c_int;

    // For ins_char_typebuf
    fn nvim_get_keynoremap() -> c_int;
    fn nvim_get_keytyped() -> c_int;
    fn nvim_get_cmd_silent() -> c_int;
    fn nvim_add_on_key_ignore_len(val: usize);

    // External Rust functions
    fn rs_special_to_buf(key: c_int, modifiers: c_int, escape_ks: c_int, dst: *mut u8) -> c_uint;
    fn rs_ins_typebuf(
        str: *const u8,
        noremap: c_int,
        offset: c_int,
        nottyped: c_int,
        silent: c_int,
    ) -> c_int;
}

use std::ffi::c_uint;

// =============================================================================
// Special Key Utilities
// =============================================================================

/// Convert termcap codes to internal key representation.
///
/// This is equivalent to C's `TERMCAP2KEY(a, b)` macro.
#[inline]
#[must_use]
pub const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -((a) + (b << 8))
}

/// Get the first byte of a special key code.
///
/// This is equivalent to C's `KEY2TERMCAP0(x)` macro.
#[inline]
#[must_use]
pub const fn key2termcap0(x: c_int) -> c_int {
    (-x) & 0xff
}

/// Get the second byte of a special key code.
///
/// This is equivalent to C's `KEY2TERMCAP1(x)` macro.
#[inline]
#[must_use]
pub const fn key2termcap1(x: c_int) -> c_int {
    ((-x) >> 8) & 0xff
}

/// Get the second byte when translating a special key code.
///
/// This is equivalent to C's `K_SECOND(c)` macro.
#[inline]
#[must_use]
pub const fn k_second(c: c_int) -> u8 {
    if c == K_SPECIAL as c_int {
        KS_SPECIAL
    } else if c == NUL as c_int {
        KS_ZERO
    } else {
        key2termcap0(c) as u8
    }
}

/// Get the third byte when translating a special key code.
///
/// This is equivalent to C's `K_THIRD(c)` macro.
#[inline]
#[must_use]
pub const fn k_third(c: c_int) -> u8 {
    if c == K_SPECIAL as c_int || c == NUL as c_int {
        KE_FILLER
    } else {
        key2termcap1(c) as u8
    }
}

/// Convert a two-byte sequence after K_SPECIAL to a special key code.
///
/// This is equivalent to C's `TO_SPECIAL(a, b)` macro.
#[inline]
#[must_use]
pub const fn to_special(a: c_int, b: c_int) -> c_int {
    if a == KS_SPECIAL as c_int {
        K_SPECIAL as c_int
    } else if a == KS_ZERO as c_int {
        NUL as c_int
    } else {
        termcap2key(a, b)
    }
}

/// Check if a character is a special key (negative value).
///
/// This is equivalent to C's `IS_SPECIAL(c)` macro.
#[inline]
#[must_use]
pub const fn is_special(c: c_int) -> bool {
    c < 0
}

// =============================================================================
// Character State Management
// =============================================================================

/// State for a character that was put back with `vungetc()`.
#[derive(Debug, Default, Clone)]
pub struct OldCharState {
    /// The character that was put back
    pub char: c_int,
    /// The modifier mask when the character was put back
    pub mod_mask: c_int,
    /// Mouse grid position
    pub mouse_grid: c_int,
    /// Mouse row position
    pub mouse_row: c_int,
    /// Mouse column position
    pub mouse_col: c_int,
}

impl OldCharState {
    /// Create a new state with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            char: -1,
            mod_mask: 0,
            mouse_grid: 0,
            mouse_row: 0,
            mouse_col: 0,
        }
    }

    /// Check if a character can be retrieved from the old char state.
    #[must_use]
    pub const fn can_get(&self) -> bool {
        self.char != -1
    }

    /// Clear the old char state.
    pub const fn clear(&mut self) {
        self.char = -1;
    }

    /// Save a character to be retrieved later.
    pub const fn save(&mut self, c: c_int, mod_mask: c_int, grid: c_int, row: c_int, col: c_int) {
        self.char = c;
        self.mod_mask = mod_mask;
        self.mouse_grid = grid;
        self.mouse_row = row;
        self.mouse_col = col;
    }
}

// =============================================================================
// Keypad Key Translation
// =============================================================================

/// Standard termcap key definitions.
/// These are equivalent to the K_* macros in C.
pub mod keys {
    use super::termcap2key;
    use std::ffi::c_int;

    // Keypad keys
    pub const K_KPLUS: c_int = termcap2key(b'K' as c_int, b'1' as c_int);
    pub const K_KMINUS: c_int = termcap2key(b'K' as c_int, b'2' as c_int);
    pub const K_KDIVIDE: c_int = termcap2key(b'K' as c_int, b'3' as c_int);
    pub const K_KMULTIPLY: c_int = termcap2key(b'K' as c_int, b'4' as c_int);
    pub const K_KENTER: c_int = termcap2key(b'K' as c_int, b'5' as c_int);
    pub const K_KPOINT: c_int = termcap2key(b'K' as c_int, b'6' as c_int);
    pub const K_KCOMMA: c_int = termcap2key(b'K' as c_int, b'9' as c_int);
    pub const K_KEQUAL: c_int = termcap2key(b'K' as c_int, b'=' as c_int);

    // Keypad number keys
    pub const K_K0: c_int = termcap2key(b'K' as c_int, b'A' as c_int);
    pub const K_K1: c_int = termcap2key(b'K' as c_int, b'B' as c_int);
    pub const K_K2: c_int = termcap2key(b'K' as c_int, b'C' as c_int);
    pub const K_K3: c_int = termcap2key(b'K' as c_int, b'D' as c_int);
    pub const K_K4: c_int = termcap2key(b'K' as c_int, b'E' as c_int);
    pub const K_K5: c_int = termcap2key(b'K' as c_int, b'F' as c_int);
    pub const K_K6: c_int = termcap2key(b'K' as c_int, b'G' as c_int);
    pub const K_K7: c_int = termcap2key(b'K' as c_int, b'H' as c_int);
    pub const K_K8: c_int = termcap2key(b'K' as c_int, b'I' as c_int);
    pub const K_K9: c_int = termcap2key(b'K' as c_int, b'J' as c_int);

    // Arrow keys
    pub const K_UP: c_int = termcap2key(b'k' as c_int, b'u' as c_int);
    pub const K_DOWN: c_int = termcap2key(b'k' as c_int, b'd' as c_int);
    pub const K_LEFT: c_int = termcap2key(b'k' as c_int, b'l' as c_int);
    pub const K_RIGHT: c_int = termcap2key(b'k' as c_int, b'r' as c_int);

    // Keypad arrow keys
    pub const K_KUP: c_int = termcap2key(b'K' as c_int, b'7' as c_int);
    pub const K_KDOWN: c_int = termcap2key(b'K' as c_int, b'8' as c_int);
    pub const K_KLEFT: c_int = termcap2key(b'#' as c_int, b'4' as c_int);
    pub const K_KRIGHT: c_int = termcap2key(b'%' as c_int, b'i' as c_int);

    // X terminal keys
    pub const K_XUP: c_int = termcap2key(253, 65);
    pub const K_XDOWN: c_int = termcap2key(253, 66);
    pub const K_XLEFT: c_int = termcap2key(253, 67);
    pub const K_XRIGHT: c_int = termcap2key(253, 68);

    // Home/End keys
    pub const K_HOME: c_int = termcap2key(b'k' as c_int, b'h' as c_int);
    pub const K_END: c_int = termcap2key(b'@' as c_int, b'7' as c_int);
    pub const K_XHOME: c_int = termcap2key(253, 63);
    pub const K_ZHOME: c_int = termcap2key(253, 64);
    pub const K_XEND: c_int = termcap2key(253, 61);
    pub const K_ZEND: c_int = termcap2key(253, 62);

    // Shifted/control variants
    pub const K_S_HOME: c_int = termcap2key(b'#' as c_int, b'2' as c_int);
    pub const K_S_END: c_int = termcap2key(b'*' as c_int, b'7' as c_int);
    pub const K_C_HOME: c_int = termcap2key(253, 87);
    pub const K_C_END: c_int = termcap2key(253, 88);

    // Special keys
    pub const K_IGNORE: c_int = termcap2key(253, 53);
    pub const K_VER_SCROLLBAR: c_int = termcap2key(249, b'X' as c_int);
    pub const K_HOR_SCROLLBAR: c_int = termcap2key(248, b'X' as c_int);
    pub const K_MOUSEMOVE: c_int = termcap2key(253, 100);
    pub const K_COMMAND: c_int = termcap2key(253, 104);
    pub const K_LUA: c_int = termcap2key(253, 103);
    pub const K_PASTE_START: c_int = termcap2key(253, 111);
}

/// Translate keypad key to its ASCII equivalent.
///
/// Returns the translated key, or the original key if no translation is needed.
#[must_use]
#[allow(clippy::wildcard_imports)]
pub const fn translate_keypad_key(c: c_int) -> c_int {
    use keys::*;

    match c {
        K_KPLUS => b'+' as c_int,
        K_KMINUS => b'-' as c_int,
        K_KDIVIDE => b'/' as c_int,
        K_KMULTIPLY => b'*' as c_int,
        K_KENTER => b'\r' as c_int, // CAR
        K_KPOINT => b'.' as c_int,
        K_KCOMMA => b',' as c_int,
        K_KEQUAL => b'=' as c_int,
        K_K0 => b'0' as c_int,
        K_K1 => b'1' as c_int,
        K_K2 => b'2' as c_int,
        K_K3 => b'3' as c_int,
        K_K4 => b'4' as c_int,
        K_K5 => b'5' as c_int,
        K_K6 => b'6' as c_int,
        K_K7 => b'7' as c_int,
        K_K8 => b'8' as c_int,
        K_K9 => b'9' as c_int,
        K_KUP | K_XUP => K_UP,
        K_KDOWN | K_XDOWN => K_DOWN,
        K_KLEFT | K_XLEFT => K_LEFT,
        K_KRIGHT | K_XRIGHT => K_RIGHT,
        _ => c,
    }
}

/// Translate home/end keys based on modifier mask.
///
/// Returns the translated key and whether the modifier was consumed.
#[must_use]
#[allow(clippy::wildcard_imports)]
pub const fn translate_home_end_key(c: c_int, mod_mask: c_int) -> (c_int, bool) {
    use keys::*;

    const MOD_MASK_SHIFT: c_int = 0x02;
    const MOD_MASK_CTRL: c_int = 0x04;

    match c {
        K_XHOME | K_ZHOME => {
            if mod_mask == MOD_MASK_SHIFT {
                (K_S_HOME, true)
            } else if mod_mask == MOD_MASK_CTRL {
                (K_C_HOME, true)
            } else {
                (K_HOME, false)
            }
        }
        K_XEND | K_ZEND => {
            if mod_mask == MOD_MASK_SHIFT {
                (K_S_END, true)
            } else if mod_mask == MOD_MASK_CTRL {
                (K_C_END, true)
            } else {
                (K_END, false)
            }
        }
        _ => (c, false),
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if a character can be retrieved from the old char buffer.
///
/// Returns true if `old_char != -1` and either `old_KeyStuffed` is set or
/// the stuff buffer is empty.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_can_get_old_char() -> c_int {
    // Use the C wrapper which checks the full condition
    nvim_can_get_old_char()
}

/// Unget one character (can only be done once!).
///
/// If the character was stuffed, vgetc() will get it next time it is called.
/// Otherwise vgetc() will only get it when the stuff buffer is empty.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_vungetc(c: c_int) {
    nvim_set_old_char(c);
    nvim_set_old_mod_mask(nvim_get_mod_mask());
    nvim_set_old_mouse_grid(nvim_get_mouse_grid());
    nvim_set_old_mouse_row(nvim_get_mouse_row());
    nvim_set_old_mouse_col(nvim_get_mouse_col());
    nvim_set_old_keystuffed(nvim_get_keystuffed());
}

/// Get the old character that was put back.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_get_old_char() -> c_int {
    nvim_get_old_char()
}

/// Clear the old character, called after it has been consumed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_old_char() {
    nvim_set_old_char(-1);
}

/// Restore state from old_char (for vgetc when old_char is available).
///
/// Sets mod_mask, mouse_grid, mouse_row, mouse_col from old_* values
/// and clears old_char.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_restore_old_char_state() {
    nvim_set_mod_mask(nvim_get_old_mod_mask());
    nvim_set_mouse_grid(nvim_get_old_mouse_grid());
    nvim_set_mouse_row(nvim_get_old_mouse_row());
    nvim_set_mouse_col(nvim_get_old_mouse_col());
    nvim_set_old_char(-1);
}

/// Maximum bytes for a special key sequence with modifiers
/// MB_MAXBYTES * 3 + 4 = 6 * 3 + 4 = 22
const MB_MAXBYTES_TIMES_3_PLUS_4: usize = 22;

/// Put character "c" back into the typeahead buffer.
///
/// Can be used for a character obtained by vgetc() that needs to be put back.
/// Uses cmd_silent, KeyTyped and KeyNoremap to restore the flags belonging to
/// the char.
///
/// # Arguments
/// * `c` - Character to insert
/// * `modifiers` - Key modifiers
/// * `on_key_ignore` - Don't store these bytes for vim.on_key()
///
/// # Returns
/// The length of what was inserted
///
/// # Safety
/// Calls C accessor functions and rs_special_to_buf, rs_ins_typebuf.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_char_typebuf(
    c: c_int,
    modifiers: c_int,
    on_key_ignore: c_int,
) -> c_int {
    let mut buf = [0u8; MB_MAXBYTES_TIMES_3_PLUS_4];
    let len = rs_special_to_buf(c, modifiers, 1, buf.as_mut_ptr());
    // NUL-terminate the buffer
    buf[len as usize] = 0;

    let keynoremap = nvim_get_keynoremap();
    let keytyped = nvim_get_keytyped();
    let cmd_silent = nvim_get_cmd_silent();

    // ins_typebuf(buf, KeyNoremap, 0, !KeyTyped, cmd_silent)
    let nottyped = c_int::from(keytyped == 0); // !KeyTyped
    rs_ins_typebuf(buf.as_ptr(), keynoremap, 0, nottyped, cmd_silent);

    if keytyped != 0 && on_key_ignore != 0 {
        nvim_add_on_key_ignore_len(len as usize);
    }

    len as c_int
}

/// Translate a special key sequence to its key code.
///
/// Given the two bytes after K_SPECIAL, returns the complete key code.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_to_special(a: c_int, b: c_int) -> c_int {
    to_special(a, b)
}

/// Get the second byte for a special key code.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_lossless)]
pub extern "C" fn rs_k_second(c: c_int) -> c_int {
    k_second(c) as c_int
}

/// Get the third byte for a special key code.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_lossless)]
pub extern "C" fn rs_k_third(c: c_int) -> c_int {
    k_third(c) as c_int
}

/// Check if a character is a special key.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_is_special(c: c_int) -> c_int {
    c_int::from(is_special(c))
}

/// Translate a keypad key to its ASCII equivalent.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_translate_keypad_key(c: c_int) -> c_int {
    translate_keypad_key(c)
}

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    #[test]
    fn test_termcap2key() {
        // Test that termcap2key produces negative values for special keys
        let k = termcap2key(b'k' as c_int, b'u' as c_int);
        assert!(k < 0);
    }

    #[test]
    fn test_special_key_roundtrip() {
        // Test that we can encode and decode special keys
        let c = -12345;
        let a = key2termcap0(c);
        let b = key2termcap1(c);
        let decoded = termcap2key(a, b);
        assert_eq!(c, decoded);
    }

    #[test]
    fn test_is_special() {
        assert!(is_special(-1));
        assert!(is_special(-100));
        assert!(!is_special(0));
        assert!(!is_special(65)); // 'A'
    }

    #[test]
    fn test_to_special_ks_special() {
        // TO_SPECIAL(KS_SPECIAL, _) should return K_SPECIAL
        let result = to_special(KS_SPECIAL as c_int, 0);
        assert_eq!(result, K_SPECIAL as c_int);
    }

    #[test]
    fn test_to_special_ks_zero() {
        // TO_SPECIAL(KS_ZERO, _) should return NUL
        let result = to_special(KS_ZERO as c_int, 0);
        assert_eq!(result, NUL as c_int);
    }

    #[test]
    fn test_translate_keypad() {
        assert_eq!(translate_keypad_key(keys::K_KPLUS), b'+' as c_int);
        assert_eq!(translate_keypad_key(keys::K_K0), b'0' as c_int);
        assert_eq!(translate_keypad_key(keys::K_K9), b'9' as c_int);
        assert_eq!(translate_keypad_key(keys::K_KUP), keys::K_UP);
    }

    #[test]
    fn test_old_char_state() {
        let mut state = OldCharState::new();
        assert!(!state.can_get());

        state.save(65, 0, 0, 0, 0);
        assert!(state.can_get());
        assert_eq!(state.char, 65);

        state.clear();
        assert!(!state.can_get());
    }
}
