//! Keycode utilities for Neovim
//!
//! This crate provides Rust implementations of keycode conversion functions.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::os::raw::{c_int, c_uint};

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

// Special key byte that marks a multi-byte key code (from keycodes.h)
const K_SPECIAL: u8 = 0x80;

// KS_* values for special key type identification
const KS_MODIFIER: u8 = 252;
const KS_EXTRA: c_int = 253;
const KS_SPECIAL: u8 = 254;

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
const KE_IGNORE: c_int = 53;

// Mouse button constants (from mouse.h)
const MOUSE_LEFT: c_int = 0x00;
const MOUSE_MIDDLE: c_int = 0x01;
const MOUSE_RIGHT: c_int = 0x02;
const MOUSE_RELEASE: c_int = 0x03;
const MOUSE_X1: c_int = 0x300;
const MOUSE_X2: c_int = 0x400;

// Special key codes for K_ZERO
const KS_ZERO: c_int = 255;
const KE_FILLER: c_int = b'X' as c_int;

// NUL character
const NUL: c_int = 0;

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

// K_ZERO (Ctrl-@)
const K_ZERO: c_int = termcap2key(KS_ZERO, KE_FILLER);

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

/// FFI result type for `extract_modifiers`
#[repr(C)]
pub struct ExtractModifiersResult {
    pub key: c_int,
    pub did_simplify: c_int,
}

/// Extract and apply modifiers to a key.
///
/// Converts a key with modifiers into a simplified form. For example:
/// - Shift+a becomes A with shift removed (unless Ctrl is also pressed)
/// - Ctrl+a becomes A (uppercase)
/// - When simplify=true, Ctrl+A becomes ^A (control character)
///
/// # Safety
/// `modp` must be a valid pointer to a modifiers value.
#[no_mangle]
pub unsafe extern "C" fn rs_extract_modifiers(
    key: c_int,
    modp: *mut c_int,
    simplify: bool,
) -> ExtractModifiersResult {
    let mut modifiers = *modp;
    let mut result_key = key;
    let mut did_simplify = false;

    // ASCII_ISALPHA check: 'A'-'Z' or 'a'-'z'
    let is_alpha = nvim_ascii::rs_ascii_isalpha(key) != 0;

    if (modifiers & MOD_MASK_SHIFT) != 0 && is_alpha {
        result_key = nvim_ascii::rs_ascii_toupper(key);
        // With <C-S-a> we keep the shift modifier.
        // With <S-a>, <A-S-a> and <S-A> we don't keep the shift modifier.
        if (modifiers & MOD_MASK_CTRL) == 0 {
            modifiers &= !MOD_MASK_SHIFT;
        }
    }

    // <C-H> and <C-h> mean the same thing, always use "H"
    if (modifiers & MOD_MASK_CTRL) != 0 && is_alpha {
        result_key = nvim_ascii::rs_ascii_toupper(result_key);
    }

    if simplify
        && (modifiers & MOD_MASK_CTRL) != 0
        && ((result_key >= c_int::from(b'?') && result_key <= c_int::from(b'_')) || is_alpha)
    {
        result_key = nvim_ascii::rs_ctrl_chr(result_key);
        modifiers &= !MOD_MASK_CTRL;
        if result_key == NUL {
            // <C-@> is <Nul>
            result_key = K_ZERO;
        }
        did_simplify = true;
    }

    *modp = modifiers;
    ExtractModifiersResult {
        key: result_key,
        did_simplify: c_int::from(did_simplify),
    }
}

/// Mouse table entry for mapping pseudo-codes to button info
struct MouseTableEntry {
    pseudo_code: c_int,
    button: c_int,
    is_click: bool,
    is_drag: bool,
}

/// Static table mapping mouse pseudo-codes to button, click, and drag info
static MOUSE_TABLE: &[MouseTableEntry] = &[
    MouseTableEntry {
        pseudo_code: KE_LEFTMOUSE,
        button: MOUSE_LEFT,
        is_click: true,
        is_drag: false,
    },
    MouseTableEntry {
        pseudo_code: KE_LEFTDRAG,
        button: MOUSE_LEFT,
        is_click: false,
        is_drag: true,
    },
    MouseTableEntry {
        pseudo_code: KE_LEFTRELEASE,
        button: MOUSE_LEFT,
        is_click: false,
        is_drag: false,
    },
    MouseTableEntry {
        pseudo_code: KE_MIDDLEMOUSE,
        button: MOUSE_MIDDLE,
        is_click: true,
        is_drag: false,
    },
    MouseTableEntry {
        pseudo_code: KE_MIDDLEDRAG,
        button: MOUSE_MIDDLE,
        is_click: false,
        is_drag: true,
    },
    MouseTableEntry {
        pseudo_code: KE_MIDDLERELEASE,
        button: MOUSE_MIDDLE,
        is_click: false,
        is_drag: false,
    },
    MouseTableEntry {
        pseudo_code: KE_RIGHTMOUSE,
        button: MOUSE_RIGHT,
        is_click: true,
        is_drag: false,
    },
    MouseTableEntry {
        pseudo_code: KE_RIGHTDRAG,
        button: MOUSE_RIGHT,
        is_click: false,
        is_drag: true,
    },
    MouseTableEntry {
        pseudo_code: KE_RIGHTRELEASE,
        button: MOUSE_RIGHT,
        is_click: false,
        is_drag: false,
    },
    MouseTableEntry {
        pseudo_code: KE_X1MOUSE,
        button: MOUSE_X1,
        is_click: true,
        is_drag: false,
    },
    MouseTableEntry {
        pseudo_code: KE_X1DRAG,
        button: MOUSE_X1,
        is_click: false,
        is_drag: true,
    },
    MouseTableEntry {
        pseudo_code: KE_X1RELEASE,
        button: MOUSE_X1,
        is_click: false,
        is_drag: false,
    },
    MouseTableEntry {
        pseudo_code: KE_X2MOUSE,
        button: MOUSE_X2,
        is_click: true,
        is_drag: false,
    },
    MouseTableEntry {
        pseudo_code: KE_X2DRAG,
        button: MOUSE_X2,
        is_click: false,
        is_drag: true,
    },
    MouseTableEntry {
        pseudo_code: KE_X2RELEASE,
        button: MOUSE_X2,
        is_click: false,
        is_drag: false,
    },
    // DRAG without CLICK
    MouseTableEntry {
        pseudo_code: KE_MOUSEMOVE,
        button: MOUSE_RELEASE,
        is_click: false,
        is_drag: true,
    },
    // RELEASE without CLICK
    MouseTableEntry {
        pseudo_code: KE_IGNORE,
        button: MOUSE_RELEASE,
        is_click: false,
        is_drag: false,
    },
];

/// Look up the given mouse code to return the relevant information.
///
/// Returns which button is down or was released.
///
/// # Safety
/// `is_click` and `is_drag` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_get_mouse_button(
    code: c_int,
    is_click: *mut bool,
    is_drag: *mut bool,
) -> c_int {
    for entry in MOUSE_TABLE {
        if code == entry.pseudo_code {
            *is_click = entry.is_click;
            *is_drag = entry.is_drag;
            return entry.button;
        }
    }
    0 // Shouldn't get here
}

/// Remove escaping from `K_SPECIAL` characters.
///
/// This is the reverse of `vim_strsave_escape_ks`. It converts escaped
/// `K_SPECIAL` sequences (`K_SPECIAL` `KS_SPECIAL` `KE_FILLER`) back to plain
/// `K_SPECIAL` bytes. Works in-place.
///
/// # Safety
/// `p` must be a valid pointer to a NUL-terminated C string that can be
/// modified in place.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_unescape_ks(p: *mut std::ffi::c_char) {
    if p.is_null() {
        return;
    }

    let mut s = p.cast::<u8>();
    let mut d = p.cast::<u8>();

    while *s != 0 {
        if *s == K_SPECIAL && *s.add(1) == KS_SPECIAL && *s.add(2) == b'X' {
            // Found escaped K_SPECIAL sequence, replace with single K_SPECIAL
            *d = K_SPECIAL;
            d = d.add(1);
            s = s.add(3);
        } else {
            // Copy byte as-is
            *d = *s;
            d = d.add(1);
            s = s.add(1);
        }
    }
    *d = 0; // NUL terminate
}

/// Add a character to a buffer, escaping `K_SPECIAL` bytes.
///
/// Converts the character to UTF-8 bytes and writes them to the buffer.
/// Any `K_SPECIAL` byte in the UTF-8 encoding is escaped as
/// `K_SPECIAL` `KS_SPECIAL` `KE_FILLER`.
///
/// # Safety
/// - `s` must be a valid pointer to a buffer with at least `MB_MAXBYTES * 3 + 1` bytes
///   available (worst case: 6 UTF-8 bytes each escaped to 3 bytes).
///
/// # Returns
/// Pointer to after the added bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_add_char2buf(
    c: c_int,
    s: *mut std::ffi::c_char,
) -> *mut std::ffi::c_char {
    if s.is_null() {
        return s;
    }

    // Convert character to UTF-8 bytes (MB_MAXBYTES = 6 for UTF-8)
    let mut temp = [0u8; 7];
    let len = nvim_mbyte::rs_utf_char2bytes(c, temp.as_mut_ptr().cast::<std::ffi::c_char>());

    let mut d = s.cast::<u8>();
    let len_usize = usize::try_from(len).unwrap_or(0);
    for &byte in temp.iter().take(len_usize) {
        if byte == K_SPECIAL {
            // Escape K_SPECIAL as 3-byte sequence
            *d = K_SPECIAL;
            *d.add(1) = KS_SPECIAL;
            *d.add(2) = b'X'; // KE_FILLER
            d = d.add(3);
        } else {
            *d = byte;
            d = d.add(1);
        }
    }

    d.cast()
}

/// Copy a string to allocated memory, escaping `K_SPECIAL` bytes.
///
/// This function allocates a new string and copies the input, escaping any
/// `K_SPECIAL` bytes so the result can be put in the typeahead buffer.
/// Existing special key sequences (`K_SPECIAL` followed by 2 bytes) are
/// copied unchanged.
///
/// # Safety
/// - `p` must be a valid pointer to a NUL-terminated C string.
///
/// # Returns
/// Newly allocated string with `K_SPECIAL` bytes escaped. Caller must free.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_strsave_escape_ks(
    p: *mut std::ffi::c_char,
) -> *mut std::ffi::c_char {
    if p.is_null() {
        return std::ptr::null_mut();
    }

    // Calculate buffer size: up to 4x the original (worst case for illegal UTF-8)
    let len = libc::strlen(p);
    let buf_size = len * 4 + 1;
    let res = nvim_memory::xmalloc(buf_size).cast::<std::ffi::c_char>();

    let mut s = p.cast::<u8>();
    let mut d = res;

    while *s != 0 {
        if *s == K_SPECIAL && *s.add(1) != 0 && *s.add(2) != 0 {
            // Copy special key unmodified (3-byte sequence)
            let d_u8 = d.cast::<u8>();
            *d_u8 = *s;
            *d_u8.add(1) = *s.add(1);
            *d_u8.add(2) = *s.add(2);
            d = d.add(3);
            s = s.add(3);
        } else {
            // Get character and advance source pointer
            let c = nvim_mbyte::rs_utf_ptr2char(s.cast());
            let char_len = nvim_mbyte::rs_utf_ptr2len(s.cast());

            // Add character to destination, escaping K_SPECIAL
            d = rs_add_char2buf(c, d);

            // Advance source by character length
            let advance = usize::try_from(char_len).unwrap_or(1);
            s = s.add(advance);
        }
    }

    // NUL terminate
    *d = 0;

    res
}

/// Check if a key code is a special key (negative value).
#[inline]
const fn is_special(key: c_int) -> bool {
    key < 0
}

/// Encode a key and modifiers into a byte sequence.
///
/// Writes the encoded bytes to `dst` and returns the number of bytes written.
/// This is how characters in a string are encoded for the typeahead buffer.
///
/// # Safety
///
/// - `dst` must be a valid pointer to a buffer with at least 6 bytes of space.
///
/// # Returns
///
/// Number of bytes written to `dst`.
#[no_mangle]
pub unsafe extern "C" fn rs_special_to_buf(
    key: c_int,
    modifiers: c_int,
    escape_ks: bool,
    dst: *mut std::ffi::c_char,
) -> c_uint {
    if dst.is_null() {
        return 0;
    }

    let d = dst.cast::<u8>();
    let mut dlen: c_uint = 0;

    // Put the appropriate modifier in a string
    if modifiers != 0 {
        *d.add(dlen as usize) = K_SPECIAL;
        *d.add(dlen as usize + 1) = KS_MODIFIER;
        // modifiers is a bitmask that fits in a byte
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        {
            *d.add(dlen as usize + 2) = modifiers as u8;
        }
        dlen += 3;
    }

    if is_special(key) {
        *d.add(dlen as usize) = K_SPECIAL;
        // key2termcap returns values already masked to 0xff
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        {
            *d.add(dlen as usize + 1) = key2termcap0(key) as u8;
            *d.add(dlen as usize + 2) = key2termcap1(key) as u8;
        }
        dlen += 3;
    } else if escape_ks {
        let after = rs_add_char2buf(key, dst.add(dlen as usize));
        let after_offset = after.offset_from(dst);
        // after_offset is always positive (pointer advances forward)
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        {
            dlen = after_offset as c_uint;
        }
    } else {
        let written = nvim_mbyte::rs_utf_char2bytes(key, dst.add(dlen as usize));
        // written is always 1-6 bytes (valid UTF-8 character)
        #[allow(clippy::cast_sign_loss)]
        {
            dlen += written as c_uint;
        }
    }

    dlen
}

#[cfg(test)]
#[allow(
    clippy::cast_lossless,
    clippy::borrow_as_ptr,
    clippy::ptr_as_ptr,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]
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

    #[test]
    fn test_get_mouse_button_left() {
        let mut is_click = false;
        let mut is_drag = false;
        unsafe {
            // Left click
            assert_eq!(
                rs_get_mouse_button(KE_LEFTMOUSE, &mut is_click, &mut is_drag),
                MOUSE_LEFT
            );
            assert!(is_click);
            assert!(!is_drag);

            // Left drag
            assert_eq!(
                rs_get_mouse_button(KE_LEFTDRAG, &mut is_click, &mut is_drag),
                MOUSE_LEFT
            );
            assert!(!is_click);
            assert!(is_drag);

            // Left release
            assert_eq!(
                rs_get_mouse_button(KE_LEFTRELEASE, &mut is_click, &mut is_drag),
                MOUSE_LEFT
            );
            assert!(!is_click);
            assert!(!is_drag);
        }
    }

    #[test]
    fn test_get_mouse_button_middle() {
        let mut is_click = false;
        let mut is_drag = false;
        unsafe {
            assert_eq!(
                rs_get_mouse_button(KE_MIDDLEMOUSE, &mut is_click, &mut is_drag),
                MOUSE_MIDDLE
            );
            assert!(is_click);
            assert!(!is_drag);
        }
    }

    #[test]
    fn test_get_mouse_button_right() {
        let mut is_click = false;
        let mut is_drag = false;
        unsafe {
            assert_eq!(
                rs_get_mouse_button(KE_RIGHTMOUSE, &mut is_click, &mut is_drag),
                MOUSE_RIGHT
            );
            assert!(is_click);
            assert!(!is_drag);
        }
    }

    #[test]
    fn test_get_mouse_button_x1_x2() {
        let mut is_click = false;
        let mut is_drag = false;
        unsafe {
            assert_eq!(
                rs_get_mouse_button(KE_X1MOUSE, &mut is_click, &mut is_drag),
                MOUSE_X1
            );
            assert!(is_click);

            assert_eq!(
                rs_get_mouse_button(KE_X2MOUSE, &mut is_click, &mut is_drag),
                MOUSE_X2
            );
            assert!(is_click);
        }
    }

    #[test]
    fn test_get_mouse_button_special() {
        let mut is_click = false;
        let mut is_drag = false;
        unsafe {
            // MOUSEMOVE - drag without click
            assert_eq!(
                rs_get_mouse_button(KE_MOUSEMOVE, &mut is_click, &mut is_drag),
                MOUSE_RELEASE
            );
            assert!(!is_click);
            assert!(is_drag);

            // IGNORE - release without click
            assert_eq!(
                rs_get_mouse_button(KE_IGNORE, &mut is_click, &mut is_drag),
                MOUSE_RELEASE
            );
            assert!(!is_click);
            assert!(!is_drag);
        }
    }

    #[test]
    fn test_extract_modifiers_shift_alpha() {
        // Shift+a becomes A, shift removed
        let mut modifiers = MOD_MASK_SHIFT;
        unsafe {
            let result = rs_extract_modifiers(b'a' as c_int, &mut modifiers, false);
            assert_eq!(result.key, b'A' as c_int);
            assert_eq!(modifiers, 0); // Shift removed
            assert_eq!(result.did_simplify, 0);
        }
    }

    #[test]
    fn test_extract_modifiers_ctrl_shift_alpha() {
        // Ctrl+Shift+a keeps shift (because ctrl is also pressed)
        let mut modifiers = MOD_MASK_CTRL | MOD_MASK_SHIFT;
        unsafe {
            let result = rs_extract_modifiers(b'a' as c_int, &mut modifiers, false);
            assert_eq!(result.key, b'A' as c_int);
            assert_eq!(modifiers, MOD_MASK_CTRL | MOD_MASK_SHIFT); // Shift kept
        }
    }

    #[test]
    fn test_extract_modifiers_ctrl_alpha_simplify() {
        // Ctrl+a with simplify becomes ^A (control character 1)
        let mut modifiers = MOD_MASK_CTRL;
        unsafe {
            let result = rs_extract_modifiers(b'a' as c_int, &mut modifiers, true);
            assert_eq!(result.key, 1); // Ctrl-A = 1
            assert_eq!(modifiers, 0); // Ctrl removed
            assert_eq!(result.did_simplify, 1);
        }
    }

    #[test]
    fn test_extract_modifiers_ctrl_at_simplify() {
        // Ctrl+@ with simplify becomes K_ZERO
        let mut modifiers = MOD_MASK_CTRL;
        unsafe {
            let result = rs_extract_modifiers(b'@' as c_int, &mut modifiers, true);
            assert_eq!(result.key, K_ZERO); // Ctrl-@ = K_ZERO
            assert_eq!(modifiers, 0);
            assert_eq!(result.did_simplify, 1);
        }
    }

    #[test]
    fn test_extract_modifiers_no_simplify() {
        // Ctrl+a without simplify keeps Ctrl
        let mut modifiers = MOD_MASK_CTRL;
        unsafe {
            let result = rs_extract_modifiers(b'a' as c_int, &mut modifiers, false);
            assert_eq!(result.key, b'A' as c_int); // Uppercase
            assert_eq!(modifiers, MOD_MASK_CTRL); // Ctrl preserved
            assert_eq!(result.did_simplify, 0);
        }
    }

    #[test]
    fn test_vim_unescape_ks_empty() {
        // Empty string
        let mut buf = [0u8; 1];
        unsafe {
            rs_vim_unescape_ks(buf.as_mut_ptr() as *mut std::ffi::c_char);
            assert_eq!(buf[0], 0);
        }
    }

    #[test]
    fn test_vim_unescape_ks_no_escape() {
        // String without any K_SPECIAL escapes
        let mut buf = *b"hello\0";
        unsafe {
            rs_vim_unescape_ks(buf.as_mut_ptr() as *mut std::ffi::c_char);
            assert_eq!(&buf[..6], b"hello\0");
        }
    }

    #[test]
    fn test_vim_unescape_ks_single_escape() {
        // Single escaped K_SPECIAL sequence
        let mut buf = [K_SPECIAL, KS_SPECIAL, b'X', 0, 0, 0];
        unsafe {
            rs_vim_unescape_ks(buf.as_mut_ptr() as *mut std::ffi::c_char);
            assert_eq!(buf[0], K_SPECIAL);
            assert_eq!(buf[1], 0); // NUL after the unescaped byte
        }
    }

    #[test]
    fn test_vim_unescape_ks_multiple_escapes() {
        // Two escaped K_SPECIAL sequences: "aXXXbXXXc\0" -> "a\x80b\x80c\0"
        let mut buf = [
            b'a', K_SPECIAL, KS_SPECIAL, b'X', b'b', K_SPECIAL, KS_SPECIAL, b'X', b'c', 0,
        ];
        unsafe {
            rs_vim_unescape_ks(buf.as_mut_ptr() as *mut std::ffi::c_char);
            assert_eq!(buf[0], b'a');
            assert_eq!(buf[1], K_SPECIAL);
            assert_eq!(buf[2], b'b');
            assert_eq!(buf[3], K_SPECIAL);
            assert_eq!(buf[4], b'c');
            assert_eq!(buf[5], 0);
        }
    }

    #[test]
    fn test_vim_unescape_ks_null_ptr() {
        // Null pointer should not crash
        unsafe {
            rs_vim_unescape_ks(std::ptr::null_mut());
        }
    }

    #[test]
    fn test_add_char2buf_ascii() {
        // ASCII character 'A' should just be copied
        let mut buf = [0i8; 20];
        unsafe {
            let end = rs_add_char2buf(b'A' as c_int, buf.as_mut_ptr());
            assert_eq!(end.offset_from(buf.as_ptr()), 1);
            assert_eq!(buf[0] as u8, b'A');
        }
    }

    #[test]
    fn test_add_char2buf_multibyte() {
        // Euro sign € (U+20AC) encodes as 3 UTF-8 bytes: E2 82 AC
        let mut buf = [0i8; 20];
        unsafe {
            let end = rs_add_char2buf(0x20AC, buf.as_mut_ptr());
            assert_eq!(end.offset_from(buf.as_ptr()), 3);
            assert_eq!(buf[0] as u8, 0xE2);
            assert_eq!(buf[1] as u8, 0x82);
            assert_eq!(buf[2] as u8, 0xAC);
        }
    }

    #[test]
    fn test_add_char2buf_k_special_escape() {
        // Character that encodes to K_SPECIAL (0x80) should be escaped
        // U+0080 encodes as C2 80 in UTF-8
        let mut buf = [0i8; 20];
        unsafe {
            let end = rs_add_char2buf(0x80, buf.as_mut_ptr());
            // First byte C2 is normal, second byte 0x80 gets escaped to 3 bytes
            // Total: 1 + 3 = 4 bytes
            assert_eq!(end.offset_from(buf.as_ptr()), 4);
            assert_eq!(buf[0] as u8, 0xC2);
            assert_eq!(buf[1] as u8, K_SPECIAL);
            assert_eq!(buf[2] as u8, KS_SPECIAL);
            assert_eq!(buf[3] as u8, b'X');
        }
    }

    #[test]
    fn test_add_char2buf_null_ptr() {
        // Null pointer should be returned as-is
        unsafe {
            let result = rs_add_char2buf(b'A' as c_int, std::ptr::null_mut());
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_special_to_buf_null_ptr() {
        // Null pointer should return 0
        unsafe {
            let result = rs_special_to_buf(b'A' as c_int, 0, false, std::ptr::null_mut());
            assert_eq!(result, 0);
        }
    }

    #[test]
    fn test_special_to_buf_regular_char() {
        // Regular ASCII character without modifiers
        let mut buf = [0i8; 10];
        unsafe {
            let result = rs_special_to_buf(b'A' as c_int, 0, false, buf.as_mut_ptr());
            assert_eq!(result, 1);
            assert_eq!(buf[0], b'A' as i8);
        }
    }

    #[test]
    fn test_special_to_buf_with_modifiers() {
        // Character with modifiers
        let mut buf = [0i8; 10];
        unsafe {
            let result = rs_special_to_buf(b'A' as c_int, MOD_MASK_CTRL, false, buf.as_mut_ptr());
            // Should write 3 bytes for modifier + 1 byte for character
            assert_eq!(result, 4);
            assert_eq!(buf[0] as u8, K_SPECIAL);
            assert_eq!(buf[1] as u8, KS_MODIFIER);
            assert_eq!(buf[2] as u8, MOD_MASK_CTRL as u8);
            assert_eq!(buf[3], b'A' as i8);
        }
    }

    #[test]
    fn test_special_to_buf_special_key() {
        // Special key (negative key code)
        let mut buf = [0i8; 10];
        let key = termcap2key(b'k' as c_int, b'u' as c_int); // K_UP
        unsafe {
            let result = rs_special_to_buf(key, 0, false, buf.as_mut_ptr());
            // Should write 3 bytes for special key
            assert_eq!(result, 3);
            assert_eq!(buf[0] as u8, K_SPECIAL);
            // Verify the termcap bytes
            assert_eq!(buf[1] as u8, key2termcap0(key) as u8);
            assert_eq!(buf[2] as u8, key2termcap1(key) as u8);
        }
    }

    // Note: vim_strsave_escape_ks allocates memory with xmalloc which
    // isn't available in pure Rust tests. Tested through C integration.
}
