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

/// KS_EXTRA - indicates an extra key code follows
const KS_EXTRA_BYTE: u8 = 253;

/// KE_FILLER - filler byte for special sequences
const KE_FILLER: u8 = b'X';

/// NUL character
const NUL: u8 = 0;

// =============================================================================
// C FFI Declarations
// =============================================================================

// old_char, old_mod_mask, old_KeyStuffed: moved from C statics to Rust (Phase 3).
// Exported as #[no_mangle] so C can still reference them via extern.
#[no_mangle]
pub static mut old_char: c_int = -1;
#[no_mangle]
pub static mut old_mod_mask: c_int = 0;
#[no_mangle]
pub static mut old_KeyStuffed: c_int = 0;

// old_mouse_* moved from C statics to Rust statics (Phase 4).
// These are only used by rs_vungetc and rs_restore_old_char_state (both in Rust).
static mut OLD_MOUSE_GRID: c_int = 0;
static mut OLD_MOUSE_ROW: c_int = 0;
static mut OLD_MOUSE_COL: c_int = 0;

extern "C" {
    // Global state: mod_mask and mouse state (direct access)
    static mut mod_mask: c_int;
    static mut mouse_grid: c_int;
    static mut mouse_row: c_int;
    static mut mouse_col: c_int;

    // KeyStuffed: true if current char from stuffbuf
    static mut KeyStuffed: c_int;

    // Stuff buffer check (for can_get_old_char logic)
    fn rs_stuff_empty() -> c_int;

    // For ins_char_typebuf (KeyNoremap is now non-static in C after Phase 3)
    static KeyNoremap: c_int;
    /// KeyTyped: true if user typed current char
    static mut KeyTyped: bool;
    /// cmd_silent: don't echo the command line
    static mut cmd_silent: bool;

    // For fix_input_buffer
    fn rs_using_script() -> c_int;

    // External Rust functions (exported via #[export_name])
    fn special_to_buf(key: c_int, modifiers: c_int, escape_ks: c_int, dst: *mut u8) -> c_uint;
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
    pub const fn save(&mut self, c: c_int, mm: c_int, grid: c_int, row: c_int, col: c_int) {
        self.char = c;
        self.mod_mask = mm;
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
    pub const K_KPLUS: c_int = termcap2key(b'K' as c_int, b'6' as c_int);
    pub const K_KMINUS: c_int = termcap2key(b'K' as c_int, b'7' as c_int);
    pub const K_KDIVIDE: c_int = termcap2key(b'K' as c_int, b'8' as c_int);
    pub const K_KMULTIPLY: c_int = termcap2key(b'K' as c_int, b'9' as c_int);
    pub const K_KENTER: c_int = termcap2key(b'K' as c_int, b'A' as c_int);
    pub const K_KPOINT: c_int = termcap2key(b'K' as c_int, b'B' as c_int);
    pub const K_KCOMMA: c_int = termcap2key(b'K' as c_int, b'M' as c_int);
    pub const K_KEQUAL: c_int = termcap2key(b'K' as c_int, b'N' as c_int);

    // Keypad number keys
    pub const K_K0: c_int = termcap2key(b'K' as c_int, b'C' as c_int);
    pub const K_K1: c_int = termcap2key(b'K' as c_int, b'D' as c_int);
    pub const K_K2: c_int = termcap2key(b'K' as c_int, b'E' as c_int);
    pub const K_K3: c_int = termcap2key(b'K' as c_int, b'F' as c_int);
    pub const K_K4: c_int = termcap2key(b'K' as c_int, b'G' as c_int);
    pub const K_K5: c_int = termcap2key(b'K' as c_int, b'H' as c_int);
    pub const K_K6: c_int = termcap2key(b'K' as c_int, b'I' as c_int);
    pub const K_K7: c_int = termcap2key(b'K' as c_int, b'J' as c_int);
    pub const K_K8: c_int = termcap2key(b'K' as c_int, b'K' as c_int);
    pub const K_K9: c_int = termcap2key(b'K' as c_int, b'L' as c_int);

    // Arrow keys
    pub const K_UP: c_int = termcap2key(b'k' as c_int, b'u' as c_int);
    pub const K_DOWN: c_int = termcap2key(b'k' as c_int, b'd' as c_int);
    pub const K_LEFT: c_int = termcap2key(b'k' as c_int, b'l' as c_int);
    pub const K_RIGHT: c_int = termcap2key(b'k' as c_int, b'r' as c_int);

    // Keypad arrow keys
    pub const K_KUP: c_int = termcap2key(b'K' as c_int, b'u' as c_int);
    pub const K_KDOWN: c_int = termcap2key(b'K' as c_int, b'd' as c_int);
    pub const K_KLEFT: c_int = termcap2key(b'K' as c_int, b'l' as c_int);
    pub const K_KRIGHT: c_int = termcap2key(b'K' as c_int, b'r' as c_int);

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
    pub const K_PASTE_START: c_int = termcap2key(b'P' as c_int, b'S' as c_int);
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
pub const fn translate_home_end_key(c: c_int, mm: c_int) -> (c_int, bool) {
    use keys::*;

    const MOD_MASK_SHIFT: c_int = 0x02;
    const MOD_MASK_CTRL: c_int = 0x04;

    match c {
        K_XHOME | K_ZHOME => {
            if mm == MOD_MASK_SHIFT {
                (K_S_HOME, true)
            } else if mm == MOD_MASK_CTRL {
                (K_C_HOME, true)
            } else {
                (K_HOME, false)
            }
        }
        K_XEND | K_ZEND => {
            if mm == MOD_MASK_SHIFT {
                (K_S_END, true)
            } else if mm == MOD_MASK_CTRL {
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
    c_int::from(old_char != -1 && (old_KeyStuffed != 0 || rs_stuff_empty() != 0))
}

/// Unget one character (can only be done once!).
///
/// If the character was stuffed, vgetc() will get it next time it is called.
/// Otherwise vgetc() will only get it when the stuff buffer is empty.
///
/// # Safety
/// Calls C accessor functions.
#[export_name = "vungetc"]
pub unsafe extern "C" fn rs_vungetc(c: c_int) {
    old_char = c;
    old_mod_mask = mod_mask;
    OLD_MOUSE_GRID = mouse_grid;
    OLD_MOUSE_ROW = mouse_row;
    OLD_MOUSE_COL = mouse_col;
    old_KeyStuffed = KeyStuffed;
}

/// Get the old character that was put back.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_get_old_char() -> c_int {
    old_char
}

/// Clear the old character, called after it has been consumed.
///
/// # Safety
/// Accesses `old_char` static directly.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_old_char() {
    old_char = -1;
}

/// Restore state from old_char (for vgetc when old_char is available).
///
/// Sets mod_mask, mouse_grid, mouse_row, mouse_col from old_* values
/// and clears old_char.
///
/// # Safety
/// Accesses old_* and mouse_* statics directly.
#[no_mangle]
pub unsafe extern "C" fn rs_restore_old_char_state() {
    mod_mask = old_mod_mask;
    mouse_grid = OLD_MOUSE_GRID;
    mouse_row = OLD_MOUSE_ROW;
    mouse_col = OLD_MOUSE_COL;
    old_char = -1;
}

// =============================================================================
// Modifier Constants
// =============================================================================

/// Ctrl modifier mask
const MOD_MASK_CTRL: c_int = 0x04;

/// K_ZERO: TERMCAP2KEY(KS_ZERO, KE_FILLER) = -(255 + (0x58 << 8))
const K_ZERO: c_int = -((KS_ZERO as c_int) + ((KE_FILLER as c_int) << 8));

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
/// Calls C accessor functions and special_to_buf, rs_ins_typebuf.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_char_typebuf(
    c: c_int,
    modifiers: c_int,
    on_key_ignore: c_int,
) -> c_int {
    let mut buf = [0u8; MB_MAXBYTES_TIMES_3_PLUS_4];
    let len = special_to_buf(c, modifiers, 1, buf.as_mut_ptr());
    // NUL-terminate the buffer
    buf[len as usize] = 0;

    let keynoremap = KeyNoremap;
    let keytyped = c_int::from(KeyTyped);
    let cmd_silent_val = c_int::from(cmd_silent);

    // ins_typebuf(buf, KeyNoremap, 0, !KeyTyped, cmd_silent)
    let nottyped = c_int::from(keytyped == 0); // !KeyTyped
    rs_ins_typebuf(buf.as_ptr(), keynoremap, 0, nottyped, cmd_silent_val);

    if keytyped != 0 && on_key_ignore != 0 {
        crate::orchestrator::on_key_ignore_len_add(len as usize);
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

/// Merge modifiers into a character code.
///
/// If CTRL is set and the character is in the '@'..0x7f range, convert to
/// a control character. CTRL-6 becomes CTRL-^.
///
/// # Safety
/// `modifiers` must be a valid pointer.
#[export_name = "merge_modifiers"]
pub unsafe extern "C" fn rs_merge_modifiers(c_arg: c_int, modifiers: *mut c_int) -> c_int {
    let mut c = c_arg;
    let mods = &mut *modifiers;

    if *mods & MOD_MASK_CTRL != 0 {
        if c >= c_int::from(b'@') && c <= 0x7f {
            c &= 0x1f;
            if c == 0 {
                c = K_ZERO;
            }
        } else if c == c_int::from(b'6') {
            // CTRL-6 is equivalent to CTRL-^
            c = 0x1e;
        }
        if c != c_arg {
            *mods &= !MOD_MASK_CTRL;
        }
    }
    c
}

/// Fix typed characters for use by vgetc().
///
/// When reading from a script, escapes NUL and K_SPECIAL bytes.
/// The buffer must have room to triple the number of bytes.
///
/// # Safety
/// `buf` must point to a buffer with at least `len * 3` bytes of capacity.
#[export_name = "fix_input_buffer"]
pub unsafe extern "C" fn rs_fix_input_buffer(buf: *mut u8, len: c_int) -> c_int {
    if rs_using_script() == 0 {
        // Not reading from script - don't escape K_SPECIAL
        *buf.add(len as usize) = 0; // NUL
        return len;
    }

    // Reading from script, need to process special bytes
    let mut p = buf;
    let mut new_len = len;

    // Two characters are special: NUL and K_SPECIAL.
    // Replace       NUL by K_SPECIAL KS_ZERO    KE_FILLER
    // Replace K_SPECIAL by K_SPECIAL KS_SPECIAL KE_FILLER
    let mut i = len;
    loop {
        i -= 1;
        if i < 0 {
            break;
        }

        if *p == 0 || (*p == K_SPECIAL && (i < 2 || *p.add(1) != KS_EXTRA_BYTE)) {
            let orig = *p;
            // memmove(p + 3, p + 1, i)
            std::ptr::copy(p.add(1), p.add(3), i as usize);
            // K_THIRD equivalent
            *p.add(2) = if orig == K_SPECIAL || orig == 0 {
                KE_FILLER
            } else {
                key2termcap1(-c_int::from(orig)) as u8
            };
            // K_SECOND equivalent
            *p.add(1) = if orig == K_SPECIAL {
                KS_SPECIAL
            } else if orig == 0 {
                KS_ZERO
            } else {
                key2termcap0(-c_int::from(orig)) as u8
            };
            *p = K_SPECIAL;
            p = p.add(2);
            new_len += 2;
        }

        p = p.add(1);
    }
    *p = 0; // NUL terminate
    new_len
}

// =============================================================================
// Phase 1: export_name wrappers -- replace C thin wrappers with bool params
// =============================================================================

/// `ins_char_typebuf(int c, int modifiers, bool on_key_ignore)`
/// Put character `c` back into the typeahead buffer.
///
/// # Safety
/// Calls C accessor functions.
#[must_use]
#[export_name = "ins_char_typebuf"]
pub unsafe extern "C" fn ins_char_typebuf_export(
    c: c_int,
    modifiers: c_int,
    on_key_ignore: bool,
) -> c_int {
    rs_ins_char_typebuf(c, modifiers, c_int::from(on_key_ignore))
}

// =============================================================================
// getchar_common, f_getchar, f_getcharstr, f_getcharmod -- Phase 3 migration
// =============================================================================

use nvim_eval::typval::TypvalT;

/// VAR_UNKNOWN = 0
const VAR_UNKNOWN: c_int = 0;
/// VAR_NUMBER = 1
const VAR_NUMBER: c_int = 1;
/// VAR_STRING = 2
const VAR_STRING: c_int = 2;
/// VAR_DICT = 5
const VAR_DICT: c_int = 5;
/// FAIL return from C functions
const FAIL: c_int = 0;

/// VV_MOUSE_WIN vim variable index (= 51, 0-based from VV_COUNT in eval_defs.h)
const VV_MOUSE_WIN: c_int = 51;
/// VV_MOUSE_WINID vim variable index
const VV_MOUSE_WINID: c_int = 52;
/// VV_MOUSE_LNUM vim variable index
const VV_MOUSE_LNUM: c_int = 53;
/// VV_MOUSE_COL vim variable index
const VV_MOUSE_COL: c_int = 54;

/// Opaque handle for window (win_T*).
type WinHandle = *mut std::ffi::c_void;

/// linenr_T (line number, C int32_t).
type LinenrT = c_int;

extern "C" {
    // globals needed by getchar_common
    static mut no_mapping: c_int;
    static mut allow_keys: c_int;
    static mut called_emsg: c_int;
    static msg_row: c_int;
    static msg_col: c_int;

    // error string globals
    static e_invarg2: [std::ffi::c_char; 0];
    static e_invargNval: [std::ffi::c_char; 0];

    // typval functions
    fn tv_check_for_opt_dict_arg(argvars: *const std::ffi::c_void, idx: c_int) -> c_int;
    fn tv_dict_get_bool(d: *mut std::ffi::c_void, key: *const std::ffi::c_char, def: c_int) -> i64;
    fn tv_dict_get_string(
        d: *const std::ffi::c_void,
        key: *const std::ffi::c_char,
        allocate: bool,
    ) -> *mut std::ffi::c_char;
    fn tv_dict_has_key(d: *const std::ffi::c_void, key: *const std::ffi::c_char) -> bool;
    fn tv_get_number_chk(tv: *const std::ffi::c_void, error: *mut bool) -> i64;

    // UI
    fn ui_busy_start();
    fn ui_busy_stop();
    fn ui_cursor_goto(row: c_int, col: c_int);

    // Input
    fn safe_vgetc() -> c_int;
    fn vpeekc_any() -> c_int;
    fn char_avail() -> bool;
    fn input_available() -> usize;
    fn state_handle_k_event();

    // set_vim_var_nr (implemented in Rust vars crate, exported as rs_set_vim_var_nr)
    #[link_name = "rs_set_vim_var_nr"]
    fn set_vim_var_nr(idx: c_int, val: i64);

    // Mouse window finding
    fn mouse_find_win_inner(grid: *mut c_int, row: *mut c_int, col: *mut c_int) -> WinHandle;
    fn mouse_comp_pos(win: WinHandle, row: *mut c_int, col: *mut c_int, lnum: *mut LinenrT)
        -> bool;

    // Window iteration
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_handle(wp: WinHandle) -> c_int;

    // String utilities
    fn utf_char2bytes(c: c_int, buf: *mut std::ffi::c_char) -> c_int;
    fn xmemdupz(data: *const std::ffi::c_void, len: usize) -> *mut std::ffi::c_void;

    // is_mouse_key
    fn is_mouse_key(c: c_int) -> bool;

    // semsg (for arg validation errors in getchar_common)
    fn semsg(fmt: *const std::ffi::c_char, ...) -> bool;

    // loop / main_loop access
    fn nvim_get_main_loop() -> *mut std::ffi::c_void;
    fn rs_loop_get_events(lp: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn rs_multiqueue_empty(mq: *mut std::ffi::c_void) -> c_int;

    // no_reduce_keys increment/decrement (exported from getchar crate)
    fn rs_inc_no_reduce_keys();
    fn rs_dec_no_reduce_keys();
}

/// Opaque handle for EvalFuncData union.
type EvalFuncData = *mut std::ffi::c_void;

/// "getchar()" and "getcharstr()" functions -- common implementation.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval_T pointers (pointing to an array
/// of at least 2 elements for argvars).
#[allow(
    clippy::cast_lossless,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::too_many_lines,
    unused_assignments
)]
unsafe fn getchar_common(argvars: *mut TypvalT, rettv: *mut TypvalT, allow_number: bool) {
    let mut n: i64 = 0;
    let called_emsg_start = called_emsg;
    let mut error = false;
    let mut simplify = true;
    let mut cursor_flag: u8 = b'\0';
    let mut allow_number = allow_number;

    // Validate optional dict argument
    if (*argvars).v_type != VAR_UNKNOWN && tv_check_for_opt_dict_arg(argvars.cast(), 1) == FAIL {
        return;
    }

    let argvars1 = argvars.add(1);
    if (*argvars).v_type != VAR_UNKNOWN && (*argvars1).v_type == VAR_DICT {
        let d = (*argvars1).vval.v_dict;

        if allow_number {
            allow_number = tv_dict_get_bool(d, c"number".as_ptr(), 1) != 0;
        } else if tv_dict_has_key(d.cast_const(), c"number".as_ptr()) {
            semsg(e_invarg2.as_ptr(), c"number".as_ptr());
        }

        simplify = tv_dict_get_bool(d, c"simplify".as_ptr(), 1) != 0;

        let cursor_str = tv_dict_get_string(d.cast_const(), c"cursor".as_ptr(), false);
        if !cursor_str.is_null() {
            let s = std::ffi::CStr::from_ptr(cursor_str);
            let sb = s.to_bytes();
            if sb == b"hide" || sb == b"keep" || sb == b"msg" {
                cursor_flag = sb[0];
            } else {
                semsg(e_invargNval.as_ptr(), c"cursor".as_ptr(), cursor_str);
            }
        }
    }

    if called_emsg != called_emsg_start {
        return;
    }

    if cursor_flag == b'h' {
        ui_busy_start();
    }

    no_mapping += 1;
    allow_keys += 1;
    if !simplify {
        rs_inc_no_reduce_keys();
    }

    loop {
        if cursor_flag == b'm' || (cursor_flag == b'\0' && msg_col > 0) {
            ui_cursor_goto(msg_row, msg_col);
        }

        if (*argvars).v_type == VAR_UNKNOWN
            || ((*argvars).v_type == VAR_NUMBER && (*argvars).vval.v_number == -1)
        {
            // getchar(): blocking wait.
            if !char_avail() {
                // Flush screen updates before blocking.
                crate::typebuf::ui_flush_for_getchar();
                let main_loop = nvim_get_main_loop();
                let events = rs_loop_get_events(main_loop);
                crate::typebuf::input_get_for_getchar(crate::typebuf::get_tb_change_cnt(), events);
                if input_available() == 0 && rs_multiqueue_empty(events) == 0 {
                    state_handle_k_event();
                    continue;
                }
            }
            n = safe_vgetc().into();
        } else if tv_get_number_chk(argvars.cast(), &raw mut error) == 1 {
            // getchar(1): only check if char avail
            n = vpeekc_any().into();
        } else if error || vpeekc_any() == 0 {
            // illegal argument or getchar(0) and no char avail: return zero
            n = 0;
        } else {
            // getchar(0) and char avail() != NUL: get a character.
            n = safe_vgetc().into();
        }

        if n == i64::from(keys::K_IGNORE)
            || n == i64::from(keys::K_MOUSEMOVE)
            || n == i64::from(keys::K_VER_SCROLLBAR)
            || n == i64::from(keys::K_HOR_SCROLLBAR)
        {
            continue;
        }
        break;
    }

    no_mapping -= 1;
    allow_keys -= 1;
    if !simplify {
        rs_dec_no_reduce_keys();
    }

    if cursor_flag == b'h' {
        ui_busy_stop();
    }

    set_vim_var_nr(VV_MOUSE_WIN, 0);
    set_vim_var_nr(VV_MOUSE_WINID, 0);
    set_vim_var_nr(VV_MOUSE_LNUM, 0);
    set_vim_var_nr(VV_MOUSE_COL, 0);

    let n_int = n as c_int;

    if n != 0 && (!allow_number || is_special(n_int) || mod_mask != 0) {
        let mut temp = [0u8; 10]; // modifier: 3, mbyte-char: 6, NUL: 1
        let mut i: usize = 0;

        // Turn a special key into three bytes, plus modifier.
        if mod_mask != 0 {
            temp[i] = K_SPECIAL;
            i += 1;
            temp[i] = KS_MODIFIER;
            i += 1;
            temp[i] = mod_mask as u8;
            i += 1;
        }
        if is_special(n_int) {
            temp[i] = K_SPECIAL;
            i += 1;
            temp[i] = k_second(n_int);
            i += 1;
            temp[i] = k_third(n_int);
            i += 1;
        } else {
            let written = utf_char2bytes(n_int, temp[i..].as_mut_ptr().cast());
            i += written as usize;
        }
        debug_assert!(i < 10);
        temp[i] = 0; // NUL

        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = xmemdupz(temp.as_ptr().cast(), i).cast();

        if is_mouse_key(n_int) {
            let mut row = mouse_row;
            let mut col = mouse_col;
            let mut grid = mouse_grid;
            let mut lnum: LinenrT = 0;

            if row >= 0 && col >= 0 {
                let mut winnr: i64 = 1;
                let win = mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
                if win.is_null() {
                    return;
                }
                mouse_comp_pos(win, &raw mut row, &raw mut col, &raw mut lnum);
                // Walk the window list to count `win`'s position.
                let mut wp = nvim_get_firstwin();
                while !wp.is_null() && wp != win {
                    wp = nvim_win_get_next(wp);
                    winnr += 1;
                }
                set_vim_var_nr(VV_MOUSE_WIN, winnr);
                set_vim_var_nr(VV_MOUSE_WINID, nvim_win_get_handle(wp).into());
                set_vim_var_nr(VV_MOUSE_LNUM, lnum.into());
                set_vim_var_nr(VV_MOUSE_COL, (col + 1).into());
            }
        }
    } else if !allow_number {
        (*rettv).v_type = VAR_STRING;
    } else {
        (*rettv).vval.v_number = n;
    }
}

/// "getchar()" VimL function -- Phase 3 Rust replacement.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval_T pointers.
#[unsafe(export_name = "f_getchar")]
pub unsafe extern "C" fn rs_f_getchar(
    argvars: *mut TypvalT,
    rettv: *mut TypvalT,
    _fptr: EvalFuncData,
) {
    getchar_common(argvars, rettv, true);
}

/// "getcharstr()" VimL function -- Phase 3 Rust replacement.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval_T pointers.
#[unsafe(export_name = "f_getcharstr")]
pub unsafe extern "C" fn rs_f_getcharstr(
    argvars: *mut TypvalT,
    rettv: *mut TypvalT,
    _fptr: EvalFuncData,
) {
    getchar_common(argvars, rettv, false);
}

/// "getcharmod()" VimL function -- Phase 3 Rust replacement.
///
/// # Safety
/// `rettv` must be a valid typval_T pointer.
#[unsafe(export_name = "f_getcharmod")]
pub unsafe extern "C" fn rs_f_getcharmod(
    _argvars: *mut TypvalT,
    rettv: *mut TypvalT,
    _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = mod_mask.into();
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
