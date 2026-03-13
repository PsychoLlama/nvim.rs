//! Special key display utilities
//!
//! Provides constants and helpers for displaying special key sequences
//! in messages and mappings.

use std::ffi::{c_char, c_int};

// Key sequence constants
// These match the values in keycodes.h

/// K_SPECIAL - marks start of special key sequence
pub const K_SPECIAL: c_int = 0x80;

/// KS_MODIFIER - modifier byte marker
pub const KS_MODIFIER: c_int = 253;

/// KS_EXTRA - extra byte marker
pub const KS_EXTRA: c_int = 254;

/// Check if a byte is KS_MODIFIER.
#[no_mangle]
pub const extern "C" fn rs_is_ks_modifier(c: c_int) -> c_int {
    (c == KS_MODIFIER) as c_int
}

/// Check if a byte is KS_EXTRA.
#[no_mangle]
pub const extern "C" fn rs_is_ks_extra(c: c_int) -> c_int {
    (c == KS_EXTRA) as c_int
}

/// Get K_SPECIAL constant.
#[no_mangle]
pub const extern "C" fn rs_k_special() -> c_int {
    K_SPECIAL
}

/// Get KS_MODIFIER constant.
#[no_mangle]
pub const extern "C" fn rs_ks_modifier() -> c_int {
    KS_MODIFIER
}

/// Get KS_EXTRA constant.
#[no_mangle]
pub const extern "C" fn rs_ks_extra() -> c_int {
    KS_EXTRA
}

// Modifier key masks (from keycodes.h)

/// MOD_MASK_SHIFT
pub const MOD_MASK_SHIFT: c_int = 0x02;
/// MOD_MASK_CTRL
pub const MOD_MASK_CTRL: c_int = 0x04;
/// MOD_MASK_ALT (META)
pub const MOD_MASK_ALT: c_int = 0x08;
/// MOD_MASK_META (Command on macOS)
pub const MOD_MASK_META: c_int = 0x10;
/// MOD_MASK_CMD (Super on Linux)
pub const MOD_MASK_CMD: c_int = 0x40;

/// Get the shift modifier mask.
#[no_mangle]
pub const extern "C" fn rs_mod_mask_shift() -> c_int {
    MOD_MASK_SHIFT
}

/// Get the ctrl modifier mask.
#[no_mangle]
pub const extern "C" fn rs_mod_mask_ctrl() -> c_int {
    MOD_MASK_CTRL
}

/// Get the alt modifier mask.
#[no_mangle]
pub const extern "C" fn rs_mod_mask_alt() -> c_int {
    MOD_MASK_ALT
}

/// Get the meta modifier mask.
#[no_mangle]
pub const extern "C" fn rs_mod_mask_meta() -> c_int {
    MOD_MASK_META
}

/// Get the cmd/super modifier mask.
#[no_mangle]
pub const extern "C" fn rs_mod_mask_cmd() -> c_int {
    MOD_MASK_CMD
}

/// Check if modifiers include shift.
#[no_mangle]
pub const extern "C" fn rs_has_mod_shift(modifiers: c_int) -> c_int {
    ((modifiers & MOD_MASK_SHIFT) != 0) as c_int
}

/// Check if modifiers include ctrl.
#[no_mangle]
pub const extern "C" fn rs_has_mod_ctrl(modifiers: c_int) -> c_int {
    ((modifiers & MOD_MASK_CTRL) != 0) as c_int
}

/// Check if modifiers include alt.
#[no_mangle]
pub const extern "C" fn rs_has_mod_alt(modifiers: c_int) -> c_int {
    ((modifiers & MOD_MASK_ALT) != 0) as c_int
}

/// Check if modifiers include meta.
#[no_mangle]
pub const extern "C" fn rs_has_mod_meta(modifiers: c_int) -> c_int {
    ((modifiers & MOD_MASK_META) != 0) as c_int
}

/// Check if modifiers include cmd/super.
#[no_mangle]
pub const extern "C" fn rs_has_mod_cmd(modifiers: c_int) -> c_int {
    ((modifiers & MOD_MASK_CMD) != 0) as c_int
}

/// Check if any modifiers are set.
#[no_mangle]
pub const extern "C" fn rs_has_any_mod(modifiers: c_int) -> c_int {
    (modifiers != 0) as c_int
}

/// Combine two modifier masks.
#[no_mangle]
pub const extern "C" fn rs_combine_mod(mod1: c_int, mod2: c_int) -> c_int {
    mod1 | mod2
}

// Special key checks

/// Check if character is a control character (< 0x20).
#[no_mangle]
pub const extern "C" fn rs_is_ctrl_char(c: c_int) -> c_int {
    (c >= 0 && c < 0x20) as c_int
}

/// Check if character should be displayed in <> form.
///
/// Returns true for control characters and K_SPECIAL.
#[no_mangle]
pub const extern "C" fn rs_needs_special_form(c: c_int) -> c_int {
    (c >= 0 && c < 0x20 || c == K_SPECIAL) as c_int
}

// Note: rs_char_display_width is defined in tui/lib.rs

// C function declarations
extern "C" {
    /// Check if a key code is a special key
    fn nvim_is_special_key(key: c_int) -> c_int;

    // For str2special implementation (Phase 85)
    /// Unescape a K_SPECIAL-encoded multi-byte sequence
    fn mb_unescape(pp: *mut *const c_char) -> *const c_char;
    /// Get printable name for a special key (e.g. "<F1>", "<S-Up>")
    fn get_special_key_name(c: c_int, modifiers: c_int) -> *mut c_char;
    /// Decode a UTF-8 pointer to a codepoint
    fn utf_ptr2char(p: *const c_char) -> c_int;
    /// Get multibyte length from a first byte
    fn nvim_get_MB_BYTE2LEN(c: c_int) -> c_int;
    /// Allocate memory (xmalloc)
    fn xmalloc(size: usize) -> *mut c_char;
    /// Get length of NUL-terminated string (strlen)
    fn strlen(s: *const c_char) -> usize;
    /// Copy memory (memcpy)
    fn memcpy(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
}

/// Check if a key code represents a special key.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_special_key(key: c_int) -> c_int {
    nvim_is_special_key(key)
}

// ============================================================================
// Key String Conversion (Phase 85)
// ============================================================================

/// Whether a key code is "special" (i.e. negative, not a real codepoint).
#[inline]
const fn is_special_key(c: c_int) -> bool {
    c < 0
}

/// Encode `(a, b)` bytes after `K_SPECIAL` into an internal key code.
///
/// Matches the C macro `TO_SPECIAL(a, b)`.
#[inline]
#[allow(clippy::cast_possible_wrap)]
const fn to_special(a: u8, b: u8) -> c_int {
    (-1 - a as i32) * 256 - b as i32
}

/// Convert one key-code at `*sp` to its printable representation.
///
/// This is the Rust implementation of the C `str2special` function.
/// On return, `*sp` is advanced past the consumed bytes.
///
/// Returns a pointer to a static buffer (or a pointer inside the original
/// string for multi-byte unescaped chars). The buffer is 7 bytes and may be
/// overwritten by the next call.
///
/// # Safety
/// - `sp` must be a valid non-null pointer to a valid NUL-terminated C string
/// - The returned pointer is either into the original string or into a static
///   buffer that is valid until the next call to this function.
#[export_name = "str2special"]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    static_mut_refs
)]
pub unsafe extern "C" fn rs_str2special(
    sp: *mut *const c_char,
    replace_spaces: bool,
    replace_lt: bool,
) -> *const c_char {
    // Static return buffer — 7 bytes is enough for any single-char representation.
    // Safety: this matches the C `static char buf[7]` contract. Not reentrant.
    static mut BUF: [c_char; 7] = [0; 7];

    // Try to un-escape a multi-byte character first.
    let p = mb_unescape(sp);
    if !p.is_null() {
        return p;
    }

    let mut str = *sp;
    let mut c = c_int::from(*str as u8);
    let mut modifiers: c_int = 0;
    let mut special = false;

    if c == K_SPECIAL && *str.add(1) != 0 && *str.add(2) != 0 {
        if *str.add(1) as u8 == KS_MODIFIER as u8 {
            modifiers = c_int::from(*str.add(2) as u8);
            str = str.add(3);
            c = c_int::from(*str as u8);
        }
        if c == K_SPECIAL && *str.add(1) != 0 && *str.add(2) != 0 {
            c = to_special(*str.add(1) as u8, *str.add(2) as u8);
            str = str.add(2);
        }
        if is_special_key(c) || modifiers != 0 {
            special = true;
        }
    }

    if !is_special_key(c) && nvim_get_MB_BYTE2LEN(c) > 1 {
        *sp = str;
        // Try to un-escape a multi-byte char after modifiers.
        let p2 = mb_unescape(sp);
        if p2.is_null() {
            // Illegal byte — advance past it.
            *sp = str.add(1);
        } else {
            // Since 'special' is true the multi-byte character 'c' will be
            // processed by get_special_key_name().
            c = utf_ptr2char(p2);
        }
    } else {
        // Single-byte character, NUL, or illegal byte.
        *sp = str.add(usize::from(*str != 0));
    }

    // Return <> form for special keys, control chars, and optionally space/<.
    if special
        || c < c_int::from(b' ')
        || (replace_spaces && c == c_int::from(b' '))
        || (replace_lt && c == c_int::from(b'<'))
    {
        return get_special_key_name(c, modifiers);
    }

    BUF[0] = c as c_char;
    BUF[1] = 0;
    std::ptr::addr_of!(BUF).cast::<c_char>()
}

/// Convert `sp`, replacing key codes with printable representations.
///
/// Writes the result into `buf` (of size `len`), stopping if `len` is
/// exhausted. Always NUL-terminates `buf`.
///
/// # Safety
/// - `sp` must be a valid NUL-terminated C string
/// - `buf` must be a valid buffer of at least `len` bytes
#[export_name = "str2specialbuf"]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_str2specialbuf(sp: *const c_char, buf: *mut c_char, len: usize) {
    let mut p = sp;
    let mut out = buf;
    let mut remaining = len;

    while *p != 0 {
        let s = rs_str2special(std::ptr::addr_of_mut!(p), false, false);
        let s_len = strlen(s);
        if remaining <= s_len {
            break;
        }
        memcpy(out, s, s_len);
        out = out.add(s_len);
        remaining -= s_len;
    }
    *out = 0;
}

/// Convert `str`, replacing key codes with printables.
///
/// Returns a heap-allocated (xmalloc) NUL-terminated string that the caller
/// must free with `xfree`.
///
/// # Safety
/// - `str` must be a valid NUL-terminated C string
#[export_name = "str2special_save"]
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_str2special_save(
    str: *const c_char,
    replace_spaces: bool,
    replace_lt: bool,
) -> *mut c_char {
    // First pass: compute total output length.
    let mut p = str;
    let mut total: usize = 0;
    while *p != 0 {
        let s = rs_str2special(std::ptr::addr_of_mut!(p), replace_spaces, replace_lt);
        total += strlen(s);
    }

    // Allocate and fill.
    let buf = xmalloc(total + 1);
    let mut out = buf;
    p = str;
    while *p != 0 {
        let s = rs_str2special(std::ptr::addr_of_mut!(p), replace_spaces, replace_lt);
        let s_len = strlen(s);
        memcpy(out, s, s_len);
        out = out.add(s_len);
    }
    *out = 0;
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_k_special_constant() {
        assert_eq!(K_SPECIAL, 0x80);
        assert_eq!(rs_k_special(), 0x80);
    }

    #[test]
    fn test_modifier_masks() {
        assert_eq!(rs_mod_mask_shift(), 0x02);
        assert_eq!(rs_mod_mask_ctrl(), 0x04);
        assert_eq!(rs_mod_mask_alt(), 0x08);
    }

    #[test]
    fn test_has_mod() {
        let shift = MOD_MASK_SHIFT;
        let ctrl = MOD_MASK_CTRL;
        let both = shift | ctrl;

        assert_eq!(rs_has_mod_shift(shift), 1);
        assert_eq!(rs_has_mod_shift(ctrl), 0);
        assert_eq!(rs_has_mod_shift(both), 1);
        assert_eq!(rs_has_mod_ctrl(both), 1);
    }

    #[test]
    fn test_is_ctrl_char() {
        assert_eq!(rs_is_ctrl_char(0), 1); // NUL
        assert_eq!(rs_is_ctrl_char(0x1F), 1); // Last control char
        assert_eq!(rs_is_ctrl_char(0x20), 0); // Space (not control)
        assert_eq!(rs_is_ctrl_char(c_int::from(b'A')), 0);
    }

    #[test]
    fn test_combine_mod() {
        let shift = MOD_MASK_SHIFT;
        let ctrl = MOD_MASK_CTRL;
        assert_eq!(rs_combine_mod(shift, ctrl), shift | ctrl);
    }
}
