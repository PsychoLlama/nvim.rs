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

// TAB character (from ascii_defs.h)
const TAB: c_int = 0x09;

// KS_EXTRA for building special key codes
const KS_EXTRA: c_int = 253;

// KE_* values for special keys (from keycodes.h enum key_extra)
const KE_S_UP: c_int = 4;
const KE_S_DOWN: c_int = 5;
const KE_S_F1: c_int = 6;
const KE_S_F2: c_int = 7;
const KE_S_F3: c_int = 8;
const KE_S_F4: c_int = 9;
const KE_S_F5: c_int = 10;
const KE_S_F6: c_int = 11;
const KE_S_F7: c_int = 12;
const KE_S_F8: c_int = 13;
const KE_S_F9: c_int = 14;
const KE_S_F10: c_int = 15;
const KE_S_F11: c_int = 16;
const KE_S_F12: c_int = 17;
const KE_S_F13: c_int = 18;
const KE_S_F14: c_int = 19;
const KE_S_F15: c_int = 20;
const KE_S_F16: c_int = 21;
const KE_S_F17: c_int = 22;
const KE_S_F18: c_int = 23;
const KE_S_F19: c_int = 24;
const KE_S_F20: c_int = 25;
const KE_S_F21: c_int = 26;
const KE_S_F22: c_int = 27;
const KE_S_F23: c_int = 28;
const KE_S_F24: c_int = 29;
const KE_S_F25: c_int = 30;
const KE_S_F26: c_int = 31;
const KE_S_F27: c_int = 32;
const KE_S_F28: c_int = 33;
const KE_S_F29: c_int = 34;
const KE_S_F30: c_int = 35;
const KE_S_F31: c_int = 36;
const KE_S_F32: c_int = 37;
const KE_S_F33: c_int = 38;
const KE_S_F34: c_int = 39;
const KE_S_F35: c_int = 40;
const KE_S_F36: c_int = 41;
const KE_S_F37: c_int = 42;
const KE_TAB: c_int = 54;
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
const KE_C_LEFT: c_int = 85;
const KE_C_RIGHT: c_int = 86;
const KE_C_HOME: c_int = 87;
const KE_C_END: c_int = 88;

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

// Tab and Shift-Tab key codes
const K_TAB: c_int = termcap2key(KS_EXTRA, KE_TAB);
const K_S_TAB: c_int = termcap2key(b'k' as c_int, b'B' as c_int);

// Shifted function keys (F5-F37)
const K_S_F5: c_int = termcap2key(KS_EXTRA, KE_S_F5);
const K_S_F6: c_int = termcap2key(KS_EXTRA, KE_S_F6);
const K_S_F7: c_int = termcap2key(KS_EXTRA, KE_S_F7);
const K_S_F8: c_int = termcap2key(KS_EXTRA, KE_S_F8);
const K_S_F9: c_int = termcap2key(KS_EXTRA, KE_S_F9);
const K_S_F10: c_int = termcap2key(KS_EXTRA, KE_S_F10);
const K_S_F11: c_int = termcap2key(KS_EXTRA, KE_S_F11);
const K_S_F12: c_int = termcap2key(KS_EXTRA, KE_S_F12);
const K_S_F13: c_int = termcap2key(KS_EXTRA, KE_S_F13);
const K_S_F14: c_int = termcap2key(KS_EXTRA, KE_S_F14);
const K_S_F15: c_int = termcap2key(KS_EXTRA, KE_S_F15);
const K_S_F16: c_int = termcap2key(KS_EXTRA, KE_S_F16);
const K_S_F17: c_int = termcap2key(KS_EXTRA, KE_S_F17);
const K_S_F18: c_int = termcap2key(KS_EXTRA, KE_S_F18);
const K_S_F19: c_int = termcap2key(KS_EXTRA, KE_S_F19);
const K_S_F20: c_int = termcap2key(KS_EXTRA, KE_S_F20);
const K_S_F21: c_int = termcap2key(KS_EXTRA, KE_S_F21);
const K_S_F22: c_int = termcap2key(KS_EXTRA, KE_S_F22);
const K_S_F23: c_int = termcap2key(KS_EXTRA, KE_S_F23);
const K_S_F24: c_int = termcap2key(KS_EXTRA, KE_S_F24);
const K_S_F25: c_int = termcap2key(KS_EXTRA, KE_S_F25);
const K_S_F26: c_int = termcap2key(KS_EXTRA, KE_S_F26);
const K_S_F27: c_int = termcap2key(KS_EXTRA, KE_S_F27);
const K_S_F28: c_int = termcap2key(KS_EXTRA, KE_S_F28);
const K_S_F29: c_int = termcap2key(KS_EXTRA, KE_S_F29);
const K_S_F30: c_int = termcap2key(KS_EXTRA, KE_S_F30);
const K_S_F31: c_int = termcap2key(KS_EXTRA, KE_S_F31);
const K_S_F32: c_int = termcap2key(KS_EXTRA, KE_S_F32);
const K_S_F33: c_int = termcap2key(KS_EXTRA, KE_S_F33);
const K_S_F34: c_int = termcap2key(KS_EXTRA, KE_S_F34);
const K_S_F35: c_int = termcap2key(KS_EXTRA, KE_S_F35);
const K_S_F36: c_int = termcap2key(KS_EXTRA, KE_S_F36);
const K_S_F37: c_int = termcap2key(KS_EXTRA, KE_S_F37);

// Shifted arrow and navigation keys
const K_S_UP: c_int = termcap2key(KS_EXTRA, KE_S_UP);
const K_S_DOWN: c_int = termcap2key(KS_EXTRA, KE_S_DOWN);

// Ctrl+arrow and navigation keys
const K_C_LEFT: c_int = termcap2key(KS_EXTRA, KE_C_LEFT);
const K_C_RIGHT: c_int = termcap2key(KS_EXTRA, KE_C_RIGHT);
const K_C_HOME: c_int = termcap2key(KS_EXTRA, KE_C_HOME);
const K_C_END: c_int = termcap2key(KS_EXTRA, KE_C_END);

// Standard function keys (F5-F10)
const K_F5: c_int = termcap2key(b'k' as c_int, b'5' as c_int);
const K_F6: c_int = termcap2key(b'k' as c_int, b'6' as c_int);
const K_F7: c_int = termcap2key(b'k' as c_int, b'7' as c_int);
const K_F8: c_int = termcap2key(b'k' as c_int, b'8' as c_int);
const K_F9: c_int = termcap2key(b'k' as c_int, b'9' as c_int);
const K_F10: c_int = termcap2key(b'k' as c_int, b';' as c_int);

// Standard function keys (F11-F63)
const K_F11: c_int = termcap2key(b'F' as c_int, b'1' as c_int);
const K_F12: c_int = termcap2key(b'F' as c_int, b'2' as c_int);
const K_F13: c_int = termcap2key(b'F' as c_int, b'3' as c_int);
const K_F14: c_int = termcap2key(b'F' as c_int, b'4' as c_int);
const K_F15: c_int = termcap2key(b'F' as c_int, b'5' as c_int);
const K_F16: c_int = termcap2key(b'F' as c_int, b'6' as c_int);
const K_F17: c_int = termcap2key(b'F' as c_int, b'7' as c_int);
const K_F18: c_int = termcap2key(b'F' as c_int, b'8' as c_int);
const K_F19: c_int = termcap2key(b'F' as c_int, b'9' as c_int);
const K_F20: c_int = termcap2key(b'F' as c_int, b'A' as c_int);
const K_F21: c_int = termcap2key(b'F' as c_int, b'B' as c_int);
const K_F22: c_int = termcap2key(b'F' as c_int, b'C' as c_int);
const K_F23: c_int = termcap2key(b'F' as c_int, b'D' as c_int);
const K_F24: c_int = termcap2key(b'F' as c_int, b'E' as c_int);
const K_F25: c_int = termcap2key(b'F' as c_int, b'F' as c_int);
const K_F26: c_int = termcap2key(b'F' as c_int, b'G' as c_int);
const K_F27: c_int = termcap2key(b'F' as c_int, b'H' as c_int);
const K_F28: c_int = termcap2key(b'F' as c_int, b'I' as c_int);
const K_F29: c_int = termcap2key(b'F' as c_int, b'J' as c_int);
const K_F30: c_int = termcap2key(b'F' as c_int, b'K' as c_int);
const K_F31: c_int = termcap2key(b'F' as c_int, b'L' as c_int);
const K_F32: c_int = termcap2key(b'F' as c_int, b'M' as c_int);
const K_F33: c_int = termcap2key(b'F' as c_int, b'N' as c_int);
const K_F34: c_int = termcap2key(b'F' as c_int, b'O' as c_int);
const K_F35: c_int = termcap2key(b'F' as c_int, b'P' as c_int);
const K_F36: c_int = termcap2key(b'F' as c_int, b'Q' as c_int);
const K_F37: c_int = termcap2key(b'F' as c_int, b'R' as c_int);

// Other navigation keys
const K_INS: c_int = termcap2key(b'k' as c_int, b'I' as c_int);
const K_DEL: c_int = termcap2key(b'k' as c_int, b'D' as c_int);
const K_DELLINE: c_int = termcap2key(b'k' as c_int, b'L' as c_int);

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

// ============================================================================
// Modifier keys table and simplify_key
// ============================================================================

/// Entry in the modifier keys table.
/// Maps a key with modifier to a key without modifier (or vice versa).
struct ModifierKeyEntry {
    mod_mask: c_int,        // Modifier mask to apply
    key_with_mod: c_int,    // Key code with modifier
    key_without_mod: c_int, // Key code without modifier
}

/// Extract termcap component 0 from a special key.
/// `KEY2TERMCAP0(x) = ((-(x)) & 0xff)`
const fn key2termcap0(x: c_int) -> c_int {
    (-x) & 0xff
}

/// Extract termcap component 1 from a special key.
/// `KEY2TERMCAP1(x) = (((unsigned)(-(x)) >> 8) & 0xff)`
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
const fn key2termcap1(x: c_int) -> c_int {
    (((-x) as u32 >> 8) & 0xff) as c_int
}

/// Table of modifier key mappings.
/// Maps keys with modifiers to their unmodified forms.
#[allow(clippy::too_many_lines)]
static MODIFIER_KEYS_TABLE: &[ModifierKeyEntry] = &[
    // Shift + special termcap keys -> unmodified forms
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'&' as c_int, b'9' as c_int),
        key_without_mod: termcap2key(b'@' as c_int, b'1' as c_int),
    }, // begin
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'&' as c_int, b'0' as c_int),
        key_without_mod: termcap2key(b'@' as c_int, b'2' as c_int),
    }, // cancel
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'*' as c_int, b'1' as c_int),
        key_without_mod: termcap2key(b'@' as c_int, b'4' as c_int),
    }, // command
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'*' as c_int, b'2' as c_int),
        key_without_mod: termcap2key(b'@' as c_int, b'5' as c_int),
    }, // copy
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'*' as c_int, b'3' as c_int),
        key_without_mod: termcap2key(b'@' as c_int, b'6' as c_int),
    }, // create
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'*' as c_int, b'4' as c_int),
        key_without_mod: K_DEL,
    }, // delete char
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'*' as c_int, b'5' as c_int),
        key_without_mod: K_DELLINE,
    }, // delete line
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'*' as c_int, b'7' as c_int),
        key_without_mod: K_END,
    }, // end
    ModifierKeyEntry {
        mod_mask: MOD_MASK_CTRL,
        key_with_mod: K_C_END,
        key_without_mod: K_END,
    }, // ctrl-end
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'*' as c_int, b'9' as c_int),
        key_without_mod: termcap2key(b'@' as c_int, b'9' as c_int),
    }, // exit
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'*' as c_int, b'0' as c_int),
        key_without_mod: termcap2key(b'@' as c_int, b'0' as c_int),
    }, // find
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'#' as c_int, b'1' as c_int),
        key_without_mod: termcap2key(b'%' as c_int, b'1' as c_int),
    }, // help
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'#' as c_int, b'2' as c_int),
        key_without_mod: K_HOME,
    }, // home
    ModifierKeyEntry {
        mod_mask: MOD_MASK_CTRL,
        key_with_mod: K_C_HOME,
        key_without_mod: K_HOME,
    }, // ctrl-home
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'#' as c_int, b'3' as c_int),
        key_without_mod: K_INS,
    }, // insert
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'#' as c_int, b'4' as c_int),
        key_without_mod: K_LEFT,
    }, // left arrow
    ModifierKeyEntry {
        mod_mask: MOD_MASK_CTRL,
        key_with_mod: K_C_LEFT,
        key_without_mod: K_LEFT,
    }, // ctrl-left
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'%' as c_int, b'a' as c_int),
        key_without_mod: termcap2key(b'%' as c_int, b'3' as c_int),
    }, // message
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'%' as c_int, b'b' as c_int),
        key_without_mod: termcap2key(b'%' as c_int, b'4' as c_int),
    }, // move
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'%' as c_int, b'c' as c_int),
        key_without_mod: termcap2key(b'%' as c_int, b'5' as c_int),
    }, // next
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'%' as c_int, b'd' as c_int),
        key_without_mod: termcap2key(b'%' as c_int, b'7' as c_int),
    }, // options
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'%' as c_int, b'e' as c_int),
        key_without_mod: termcap2key(b'%' as c_int, b'8' as c_int),
    }, // previous
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'%' as c_int, b'f' as c_int),
        key_without_mod: termcap2key(b'%' as c_int, b'9' as c_int),
    }, // print
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'%' as c_int, b'g' as c_int),
        key_without_mod: termcap2key(b'%' as c_int, b'0' as c_int),
    }, // redo
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'%' as c_int, b'h' as c_int),
        key_without_mod: termcap2key(b'&' as c_int, b'3' as c_int),
    }, // replace
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'%' as c_int, b'i' as c_int),
        key_without_mod: K_RIGHT,
    }, // right arrow
    ModifierKeyEntry {
        mod_mask: MOD_MASK_CTRL,
        key_with_mod: K_C_RIGHT,
        key_without_mod: K_RIGHT,
    }, // ctrl-right
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'%' as c_int, b'j' as c_int),
        key_without_mod: termcap2key(b'&' as c_int, b'5' as c_int),
    }, // resume
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'!' as c_int, b'1' as c_int),
        key_without_mod: termcap2key(b'&' as c_int, b'6' as c_int),
    }, // save
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'!' as c_int, b'2' as c_int),
        key_without_mod: termcap2key(b'&' as c_int, b'7' as c_int),
    }, // suspend
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: termcap2key(b'!' as c_int, b'3' as c_int),
        key_without_mod: termcap2key(b'&' as c_int, b'8' as c_int),
    }, // undo
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_UP,
        key_without_mod: K_UP,
    }, // up arrow
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_DOWN,
        key_without_mod: K_DOWN,
    }, // down arrow
    // vt100 F1-F4
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_XF1,
        key_without_mod: K_XF1,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_XF2,
        key_without_mod: K_XF2,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_XF3,
        key_without_mod: K_XF3,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_XF4,
        key_without_mod: K_XF4,
    },
    // Function keys F1-F10
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F1,
        key_without_mod: K_F1,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F2,
        key_without_mod: K_F2,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F3,
        key_without_mod: K_F3,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F4,
        key_without_mod: K_F4,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F5,
        key_without_mod: K_F5,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F6,
        key_without_mod: K_F6,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F7,
        key_without_mod: K_F7,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F8,
        key_without_mod: K_F8,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F9,
        key_without_mod: K_F9,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F10,
        key_without_mod: K_F10,
    },
    // Function keys F11-F20
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F11,
        key_without_mod: K_F11,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F12,
        key_without_mod: K_F12,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F13,
        key_without_mod: K_F13,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F14,
        key_without_mod: K_F14,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F15,
        key_without_mod: K_F15,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F16,
        key_without_mod: K_F16,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F17,
        key_without_mod: K_F17,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F18,
        key_without_mod: K_F18,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F19,
        key_without_mod: K_F19,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F20,
        key_without_mod: K_F20,
    },
    // Function keys F21-F30
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F21,
        key_without_mod: K_F21,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F22,
        key_without_mod: K_F22,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F23,
        key_without_mod: K_F23,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F24,
        key_without_mod: K_F24,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F25,
        key_without_mod: K_F25,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F26,
        key_without_mod: K_F26,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F27,
        key_without_mod: K_F27,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F28,
        key_without_mod: K_F28,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F29,
        key_without_mod: K_F29,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F30,
        key_without_mod: K_F30,
    },
    // Function keys F31-F37
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F31,
        key_without_mod: K_F31,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F32,
        key_without_mod: K_F32,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F33,
        key_without_mod: K_F33,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F34,
        key_without_mod: K_F34,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F35,
        key_without_mod: K_F35,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F36,
        key_without_mod: K_F36,
    },
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_F37,
        key_without_mod: K_F37,
    },
    // TAB pseudo code
    ModifierKeyEntry {
        mod_mask: MOD_MASK_SHIFT,
        key_with_mod: K_S_TAB,
        key_without_mod: K_TAB,
    },
];

/// Check if there is a special key code for "key" with specified modifiers.
///
/// If a key can be represented as a simpler key + modifier removed from the
/// modifier mask, this function returns the simplified key and updates the
/// modifier mask.
///
/// # Arguments
/// * `key` - Initial key code
/// * `modifiers` - Pointer to modifier mask (updated in place)
///
/// # Returns
/// Simplified key code
///
/// # Safety
/// `modifiers` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_simplify_key(key: c_int, modifiers: *mut c_int) -> c_int {
    if modifiers.is_null() {
        return key;
    }

    // Quick check: if neither SHIFT nor CTRL is set, nothing to simplify
    if (*modifiers & (MOD_MASK_SHIFT | MOD_MASK_CTRL)) == 0 {
        return key;
    }

    // TAB is a special case
    if key == TAB && (*modifiers & MOD_MASK_SHIFT) != 0 {
        *modifiers &= !MOD_MASK_SHIFT;
        return K_S_TAB;
    }

    // Extract termcap components from the key
    let key0 = key2termcap0(key);
    let key1 = key2termcap1(key);

    // Search the modifier keys table
    for entry in MODIFIER_KEYS_TABLE {
        let entry_key0 = key2termcap0(entry.key_without_mod);
        let entry_key1 = key2termcap1(entry.key_without_mod);

        if key0 == entry_key0 && key1 == entry_key1 && (*modifiers & entry.mod_mask) != 0 {
            *modifiers &= !entry.mod_mask;
            return entry.key_with_mod;
        }
    }

    key
}

#[cfg(test)]
#[allow(clippy::cast_lossless, clippy::borrow_as_ptr)]
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

    #[test]
    fn test_simplify_key_no_modifiers() {
        // No modifiers - key unchanged
        let mut modifiers: c_int = 0;
        unsafe {
            assert_eq!(rs_simplify_key(K_UP, &mut modifiers), K_UP);
            assert_eq!(modifiers, 0);
            assert_eq!(rs_simplify_key(K_F1, &mut modifiers), K_F1);
            assert_eq!(modifiers, 0);
        }
    }

    #[test]
    fn test_simplify_key_shift_tab() {
        // Shift+TAB is a special case
        let mut modifiers: c_int = MOD_MASK_SHIFT;
        unsafe {
            assert_eq!(rs_simplify_key(TAB, &mut modifiers), K_S_TAB);
            assert_eq!(modifiers, 0); // Shift is removed
        }
    }

    #[test]
    fn test_simplify_key_shift_up() {
        // Shift+Up -> S-Up
        let mut modifiers: c_int = MOD_MASK_SHIFT;
        unsafe {
            assert_eq!(rs_simplify_key(K_UP, &mut modifiers), K_S_UP);
            assert_eq!(modifiers, 0); // Shift is removed
        }
    }

    #[test]
    fn test_simplify_key_shift_f1() {
        // Shift+F1 -> S-F1
        let mut modifiers: c_int = MOD_MASK_SHIFT;
        unsafe {
            assert_eq!(rs_simplify_key(K_F1, &mut modifiers), K_S_F1);
            assert_eq!(modifiers, 0); // Shift is removed
        }
    }

    #[test]
    fn test_simplify_key_ctrl_left() {
        // Ctrl+Left -> C-Left
        let mut modifiers: c_int = MOD_MASK_CTRL;
        unsafe {
            assert_eq!(rs_simplify_key(K_LEFT, &mut modifiers), K_C_LEFT);
            assert_eq!(modifiers, 0); // Ctrl is removed
        }
    }

    #[test]
    fn test_simplify_key_ctrl_home() {
        // Ctrl+Home -> C-Home
        let mut modifiers: c_int = MOD_MASK_CTRL;
        unsafe {
            assert_eq!(rs_simplify_key(K_HOME, &mut modifiers), K_C_HOME);
            assert_eq!(modifiers, 0); // Ctrl is removed
        }
    }

    #[test]
    fn test_simplify_key_preserves_other_modifiers() {
        // Shift+Alt+Up: Shift is simplified, Alt preserved
        let mut modifiers: c_int = MOD_MASK_SHIFT | MOD_MASK_ALT;
        unsafe {
            assert_eq!(rs_simplify_key(K_UP, &mut modifiers), K_S_UP);
            assert_eq!(modifiers, MOD_MASK_ALT); // Only Shift is removed
        }
    }

    #[test]
    fn test_simplify_key_null_modifiers() {
        // Null pointer - key unchanged
        unsafe {
            assert_eq!(rs_simplify_key(K_UP, std::ptr::null_mut()), K_UP);
        }
    }
}
