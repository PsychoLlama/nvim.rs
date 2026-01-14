//! Special key display utilities
//!
//! Provides constants and helpers for displaying special key sequences
//! in messages and mappings.

use std::ffi::c_int;

// Key sequence constants
// These match the values in keycodes.h

/// K_SPECIAL - marks start of special key sequence
pub const K_SPECIAL: c_int = 0x80;

/// KS_MODIFIER - modifier byte marker
pub const KS_MODIFIER: c_int = 253;

/// KS_EXTRA - extra byte marker
pub const KS_EXTRA: c_int = 254;

// Note: rs_is_k_special is defined in regexp/regsub.rs

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
}

/// Check if a key code represents a special key.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_special_key(key: c_int) -> c_int {
    nvim_is_special_key(key)
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
