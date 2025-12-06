//! Keycode utilities for Neovim
//!
//! This crate provides Rust implementations of keycode conversion functions.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::os::raw::c_int;

// Modifier mask constants (from keycodes.h)
const MOD_MASK_SHIFT: c_int = 0x02;
const MOD_MASK_CTRL: c_int = 0x04;
const MOD_MASK_ALT: c_int = 0x08;
const MOD_MASK_META: c_int = 0x10;
const MOD_MASK_2CLICK: c_int = 0x20;
const MOD_MASK_3CLICK: c_int = 0x40;
const MOD_MASK_4CLICK: c_int = 0x60;
const MOD_MASK_CMD: c_int = 0x80;

// KS_EXTRA for building special key codes
const KS_EXTRA: c_int = 253;

// KE_* values for special keys (from keycodes.h enum key_extra)
const KE_S_F1: c_int = 6;
const KE_S_F2: c_int = 7;
const KE_S_F3: c_int = 8;
const KE_S_F4: c_int = 9;
const KE_XF1: c_int = 57;
const KE_XF2: c_int = 58;
const KE_XF3: c_int = 59;
const KE_XF4: c_int = 60;
const KE_XEND: c_int = 61;
const KE_ZEND: c_int = 62;
const KE_XHOME: c_int = 63;
const KE_ZHOME: c_int = 64;
const KE_XUP: c_int = 65;
const KE_XDOWN: c_int = 66;
const KE_XLEFT: c_int = 67;
const KE_XRIGHT: c_int = 68;
const KE_S_XF1: c_int = 71;
const KE_S_XF2: c_int = 72;
const KE_S_XF3: c_int = 73;
const KE_S_XF4: c_int = 74;

// Mouse key KE_* constants (from keycodes.h)
const KE_LEFTMOUSE: c_int = 44;
const KE_LEFTDRAG: c_int = 45;
const KE_LEFTRELEASE: c_int = 46;
const KE_MIDDLEMOUSE: c_int = 47;
const KE_MIDDLEDRAG: c_int = 48;
const KE_MIDDLERELEASE: c_int = 49;
const KE_RIGHTMOUSE: c_int = 50;
const KE_RIGHTDRAG: c_int = 51;
const KE_RIGHTRELEASE: c_int = 52;
const KE_LEFTMOUSE_NM: c_int = 69;
const KE_LEFTRELEASE_NM: c_int = 70;
const KE_MOUSEDOWN: c_int = 75;
const KE_MOUSEUP: c_int = 76;
const KE_MOUSELEFT: c_int = 77;
const KE_MOUSERIGHT: c_int = 78;
const KE_X1MOUSE: c_int = 89;
const KE_X1DRAG: c_int = 90;
const KE_X1RELEASE: c_int = 91;
const KE_X2MOUSE: c_int = 92;
const KE_X2DRAG: c_int = 93;
const KE_X2RELEASE: c_int = 94;
const KE_MOUSEMOVE: c_int = 100;

/// Convert termcap codes to internal key representation
/// TERMCAP2KEY(a, b) = -((a) + ((int)(b) << 8))
const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -((a) + (b << 8))
}

// Standard key codes
const K_UP: c_int = termcap2key(b'k' as c_int, b'u' as c_int);
const K_DOWN: c_int = termcap2key(b'k' as c_int, b'd' as c_int);
const K_LEFT: c_int = termcap2key(b'k' as c_int, b'l' as c_int);
const K_RIGHT: c_int = termcap2key(b'k' as c_int, b'r' as c_int);
const K_HOME: c_int = termcap2key(b'k' as c_int, b'h' as c_int);
const K_END: c_int = termcap2key(b'@' as c_int, b'7' as c_int);
const K_F1: c_int = termcap2key(b'k' as c_int, b'1' as c_int);
const K_F2: c_int = termcap2key(b'k' as c_int, b'2' as c_int);
const K_F3: c_int = termcap2key(b'k' as c_int, b'3' as c_int);
const K_F4: c_int = termcap2key(b'k' as c_int, b'4' as c_int);
const K_S_F1: c_int = termcap2key(KS_EXTRA, KE_S_F1);
const K_S_F2: c_int = termcap2key(KS_EXTRA, KE_S_F2);
const K_S_F3: c_int = termcap2key(KS_EXTRA, KE_S_F3);
const K_S_F4: c_int = termcap2key(KS_EXTRA, KE_S_F4);

// X-terminal variant key codes
const K_XUP: c_int = termcap2key(KS_EXTRA, KE_XUP);
const K_XDOWN: c_int = termcap2key(KS_EXTRA, KE_XDOWN);
const K_XLEFT: c_int = termcap2key(KS_EXTRA, KE_XLEFT);
const K_XRIGHT: c_int = termcap2key(KS_EXTRA, KE_XRIGHT);
const K_XHOME: c_int = termcap2key(KS_EXTRA, KE_XHOME);
const K_ZHOME: c_int = termcap2key(KS_EXTRA, KE_ZHOME);
const K_XEND: c_int = termcap2key(KS_EXTRA, KE_XEND);
const K_ZEND: c_int = termcap2key(KS_EXTRA, KE_ZEND);
const K_XF1: c_int = termcap2key(KS_EXTRA, KE_XF1);
const K_XF2: c_int = termcap2key(KS_EXTRA, KE_XF2);
const K_XF3: c_int = termcap2key(KS_EXTRA, KE_XF3);
const K_XF4: c_int = termcap2key(KS_EXTRA, KE_XF4);
const K_S_XF1: c_int = termcap2key(KS_EXTRA, KE_S_XF1);
const K_S_XF2: c_int = termcap2key(KS_EXTRA, KE_S_XF2);
const K_S_XF3: c_int = termcap2key(KS_EXTRA, KE_S_XF3);
const K_S_XF4: c_int = termcap2key(KS_EXTRA, KE_S_XF4);

// Mouse key codes
const K_LEFTMOUSE: c_int = termcap2key(KS_EXTRA, KE_LEFTMOUSE);
const K_LEFTMOUSE_NM: c_int = termcap2key(KS_EXTRA, KE_LEFTMOUSE_NM);
const K_LEFTDRAG: c_int = termcap2key(KS_EXTRA, KE_LEFTDRAG);
const K_LEFTRELEASE: c_int = termcap2key(KS_EXTRA, KE_LEFTRELEASE);
const K_LEFTRELEASE_NM: c_int = termcap2key(KS_EXTRA, KE_LEFTRELEASE_NM);
const K_MOUSEMOVE: c_int = termcap2key(KS_EXTRA, KE_MOUSEMOVE);
const K_MIDDLEMOUSE: c_int = termcap2key(KS_EXTRA, KE_MIDDLEMOUSE);
const K_MIDDLEDRAG: c_int = termcap2key(KS_EXTRA, KE_MIDDLEDRAG);
const K_MIDDLERELEASE: c_int = termcap2key(KS_EXTRA, KE_MIDDLERELEASE);
const K_RIGHTMOUSE: c_int = termcap2key(KS_EXTRA, KE_RIGHTMOUSE);
const K_RIGHTDRAG: c_int = termcap2key(KS_EXTRA, KE_RIGHTDRAG);
const K_RIGHTRELEASE: c_int = termcap2key(KS_EXTRA, KE_RIGHTRELEASE);
const K_MOUSEDOWN: c_int = termcap2key(KS_EXTRA, KE_MOUSEDOWN);
const K_MOUSEUP: c_int = termcap2key(KS_EXTRA, KE_MOUSEUP);
const K_MOUSELEFT: c_int = termcap2key(KS_EXTRA, KE_MOUSELEFT);
const K_MOUSERIGHT: c_int = termcap2key(KS_EXTRA, KE_MOUSERIGHT);
const K_X1MOUSE: c_int = termcap2key(KS_EXTRA, KE_X1MOUSE);
const K_X1DRAG: c_int = termcap2key(KS_EXTRA, KE_X1DRAG);
const K_X1RELEASE: c_int = termcap2key(KS_EXTRA, KE_X1RELEASE);
const K_X2MOUSE: c_int = termcap2key(KS_EXTRA, KE_X2MOUSE);
const K_X2DRAG: c_int = termcap2key(KS_EXTRA, KE_X2DRAG);
const K_X2RELEASE: c_int = termcap2key(KS_EXTRA, KE_X2RELEASE);

/// Modifier mask table entry
struct ModMaskEntry {
    mod_flag: c_int,
    name: u8,
}

/// Table mapping modifier names to modifier flags
static MOD_MASK_TABLE: &[ModMaskEntry] = &[
    ModMaskEntry {
        mod_flag: MOD_MASK_ALT,
        name: b'M',
    },
    ModMaskEntry {
        mod_flag: MOD_MASK_META,
        name: b'T',
    },
    ModMaskEntry {
        mod_flag: MOD_MASK_CTRL,
        name: b'C',
    },
    ModMaskEntry {
        mod_flag: MOD_MASK_SHIFT,
        name: b'S',
    },
    ModMaskEntry {
        mod_flag: MOD_MASK_2CLICK,
        name: b'2',
    },
    ModMaskEntry {
        mod_flag: MOD_MASK_3CLICK,
        name: b'3',
    },
    ModMaskEntry {
        mod_flag: MOD_MASK_4CLICK,
        name: b'4',
    },
    ModMaskEntry {
        mod_flag: MOD_MASK_CMD,
        name: b'D',
    },
    // 'A' must be last - it's an alias for ALT
    ModMaskEntry {
        mod_flag: MOD_MASK_ALT,
        name: b'A',
    },
];

/// Convert ASCII character to uppercase (ASCII only)
const fn toupper_asc(c: c_int) -> c_int {
    if c >= b'a' as c_int && c <= b'z' as c_int {
        c - 32
    } else {
        c
    }
}

/// Return the modifier mask bit corresponding to modifier name.
///
/// E.g. 'S' for shift, 'C' for ctrl, 'M' for alt/meta.
/// Returns 0 if the character doesn't correspond to a known modifier.
#[no_mangle]
pub extern "C" fn rs_name_to_mod_mask(c: c_int) -> c_int {
    let c = toupper_asc(c);
    for entry in MOD_MASK_TABLE {
        if c == c_int::from(entry.name) {
            return entry.mod_flag;
        }
    }
    0
}

/// Change <xKey> to <Key>
///
/// Maps X-terminal specific key codes (like `K_XUP`, `K_XF1`) to their
/// standard equivalents (`K_UP`, `K_F1`).
#[no_mangle]
pub extern "C" fn rs_handle_x_keys(key: c_int) -> c_int {
    match key {
        K_XUP => K_UP,
        K_XDOWN => K_DOWN,
        K_XLEFT => K_LEFT,
        K_XRIGHT => K_RIGHT,
        K_XHOME | K_ZHOME => K_HOME,
        K_XEND | K_ZEND => K_END,
        K_XF1 => K_F1,
        K_XF2 => K_F2,
        K_XF3 => K_F3,
        K_XF4 => K_F4,
        K_S_XF1 => K_S_F1,
        K_S_XF2 => K_S_F2,
        K_S_XF3 => K_S_F3,
        K_S_XF4 => K_S_F4,
        _ => key,
    }
}

/// Return true if "c" is a mouse key.
#[no_mangle]
pub extern "C" fn rs_is_mouse_key(c: c_int) -> bool {
    matches!(
        c,
        K_LEFTMOUSE
            | K_LEFTMOUSE_NM
            | K_LEFTDRAG
            | K_LEFTRELEASE
            | K_LEFTRELEASE_NM
            | K_MOUSEMOVE
            | K_MIDDLEMOUSE
            | K_MIDDLEDRAG
            | K_MIDDLERELEASE
            | K_RIGHTMOUSE
            | K_RIGHTDRAG
            | K_RIGHTRELEASE
            | K_MOUSEDOWN
            | K_MOUSEUP
            | K_MOUSELEFT
            | K_MOUSERIGHT
            | K_X1MOUSE
            | K_X1DRAG
            | K_X1RELEASE
            | K_X2MOUSE
            | K_X2DRAG
            | K_X2RELEASE
    )
}

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    #[test]
    fn test_name_to_mod_mask() {
        // Test uppercase
        assert_eq!(rs_name_to_mod_mask(b'S' as c_int), MOD_MASK_SHIFT);
        assert_eq!(rs_name_to_mod_mask(b'C' as c_int), MOD_MASK_CTRL);
        assert_eq!(rs_name_to_mod_mask(b'M' as c_int), MOD_MASK_ALT);
        assert_eq!(rs_name_to_mod_mask(b'A' as c_int), MOD_MASK_ALT);
        assert_eq!(rs_name_to_mod_mask(b'T' as c_int), MOD_MASK_META);
        assert_eq!(rs_name_to_mod_mask(b'D' as c_int), MOD_MASK_CMD);
        assert_eq!(rs_name_to_mod_mask(b'2' as c_int), MOD_MASK_2CLICK);
        assert_eq!(rs_name_to_mod_mask(b'3' as c_int), MOD_MASK_3CLICK);
        assert_eq!(rs_name_to_mod_mask(b'4' as c_int), MOD_MASK_4CLICK);

        // Test lowercase (should be converted to uppercase)
        assert_eq!(rs_name_to_mod_mask(b's' as c_int), MOD_MASK_SHIFT);
        assert_eq!(rs_name_to_mod_mask(b'c' as c_int), MOD_MASK_CTRL);
        assert_eq!(rs_name_to_mod_mask(b'm' as c_int), MOD_MASK_ALT);

        // Test unknown characters
        assert_eq!(rs_name_to_mod_mask(b'X' as c_int), 0);
        assert_eq!(rs_name_to_mod_mask(b'0' as c_int), 0);
    }

    #[test]
    fn test_handle_x_keys() {
        // X-keys should be converted to standard keys
        assert_eq!(rs_handle_x_keys(K_XUP), K_UP);
        assert_eq!(rs_handle_x_keys(K_XDOWN), K_DOWN);
        assert_eq!(rs_handle_x_keys(K_XLEFT), K_LEFT);
        assert_eq!(rs_handle_x_keys(K_XRIGHT), K_RIGHT);
        assert_eq!(rs_handle_x_keys(K_XHOME), K_HOME);
        assert_eq!(rs_handle_x_keys(K_ZHOME), K_HOME);
        assert_eq!(rs_handle_x_keys(K_XEND), K_END);
        assert_eq!(rs_handle_x_keys(K_ZEND), K_END);
        assert_eq!(rs_handle_x_keys(K_XF1), K_F1);
        assert_eq!(rs_handle_x_keys(K_XF2), K_F2);
        assert_eq!(rs_handle_x_keys(K_XF3), K_F3);
        assert_eq!(rs_handle_x_keys(K_XF4), K_F4);
        assert_eq!(rs_handle_x_keys(K_S_XF1), K_S_F1);
        assert_eq!(rs_handle_x_keys(K_S_XF2), K_S_F2);
        assert_eq!(rs_handle_x_keys(K_S_XF3), K_S_F3);
        assert_eq!(rs_handle_x_keys(K_S_XF4), K_S_F4);

        // Non-X keys should pass through unchanged
        assert_eq!(rs_handle_x_keys(K_UP), K_UP);
        assert_eq!(rs_handle_x_keys(K_F1), K_F1);
        assert_eq!(rs_handle_x_keys(0), 0);
        assert_eq!(rs_handle_x_keys(42), 42);
    }

    #[test]
    fn test_is_mouse_key() {
        // All mouse keys should return true
        assert!(rs_is_mouse_key(K_LEFTMOUSE));
        assert!(rs_is_mouse_key(K_LEFTMOUSE_NM));
        assert!(rs_is_mouse_key(K_LEFTDRAG));
        assert!(rs_is_mouse_key(K_LEFTRELEASE));
        assert!(rs_is_mouse_key(K_LEFTRELEASE_NM));
        assert!(rs_is_mouse_key(K_MOUSEMOVE));
        assert!(rs_is_mouse_key(K_MIDDLEMOUSE));
        assert!(rs_is_mouse_key(K_MIDDLEDRAG));
        assert!(rs_is_mouse_key(K_MIDDLERELEASE));
        assert!(rs_is_mouse_key(K_RIGHTMOUSE));
        assert!(rs_is_mouse_key(K_RIGHTDRAG));
        assert!(rs_is_mouse_key(K_RIGHTRELEASE));
        assert!(rs_is_mouse_key(K_MOUSEDOWN));
        assert!(rs_is_mouse_key(K_MOUSEUP));
        assert!(rs_is_mouse_key(K_MOUSELEFT));
        assert!(rs_is_mouse_key(K_MOUSERIGHT));
        assert!(rs_is_mouse_key(K_X1MOUSE));
        assert!(rs_is_mouse_key(K_X1DRAG));
        assert!(rs_is_mouse_key(K_X1RELEASE));
        assert!(rs_is_mouse_key(K_X2MOUSE));
        assert!(rs_is_mouse_key(K_X2DRAG));
        assert!(rs_is_mouse_key(K_X2RELEASE));

        // Non-mouse keys should return false
        assert!(!rs_is_mouse_key(K_UP));
        assert!(!rs_is_mouse_key(K_DOWN));
        assert!(!rs_is_mouse_key(K_LEFT));
        assert!(!rs_is_mouse_key(K_RIGHT));
        assert!(!rs_is_mouse_key(K_F1));
        assert!(!rs_is_mouse_key(0));
        assert!(!rs_is_mouse_key(42));
        assert!(!rs_is_mouse_key(b'a' as c_int));
    }
}
