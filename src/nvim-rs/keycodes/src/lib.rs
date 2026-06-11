//! Keycode utilities for Neovim
//!
//! This crate provides Rust implementations of keycode conversion functions.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::os::raw::{c_int, c_uint};

/// Script ID type (same as int in C).
type ScidT = c_int;

/// Mirror of C's `sctx_T` struct (script context).
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(clippy::struct_field_names)]
struct SctxT {
    sc_sid: c_int,
    sc_seq: c_int,
    sc_lnum: i32,
    sc_chan: u64,
}

// Modifier mask constants (from keycodes.h)
const MOD_MASK_SHIFT: c_int = 0x02;
const MOD_MASK_CTRL: c_int = 0x04;
const MOD_MASK_ALT: c_int = 0x08;
const MOD_MASK_META: c_int = 0x10;
const MOD_MASK_2CLICK: c_int = 0x20;
const MOD_MASK_3CLICK: c_int = 0x40;
const MOD_MASK_4CLICK: c_int = 0x60;
const MOD_MASK_CMD: c_int = 0x80;
const MOD_MASK_MULTI_CLICK: c_int = MOD_MASK_2CLICK | MOD_MASK_3CLICK | MOD_MASK_4CLICK;

// TAB character (from ascii_defs.h)
const TAB: c_int = 0x09;

// Special key byte that marks a multi-byte key code (from keycodes.h)
const K_SPECIAL: u8 = 0x80;

// KS_* values for special key type identification
const KS_MODIFIER: u8 = 252;
const KS_EXTRA: c_int = 253;
const KS_SPECIAL: u8 = 254;
const KS_KEY: c_int = 242;

// Maximum length for key name strings (from keycodes.h)
const MAX_KEY_NAME_LEN: usize = 32;

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

// Special event keys (from keycodes.h)
#[allow(dead_code)]
const KE_EVENT: c_int = 102;
#[allow(dead_code)]
const KE_LUA: c_int = 103;
#[allow(dead_code)]
const KE_COMMAND: c_int = 104;

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

// ASCII characters
const BS: c_int = 0x08; // Backspace
const DEL: c_int = 0x7f; // Delete

// FSK_* flags for find_special_key
const FSK_KEYCODE: c_int = 0x01; // prefer key code, e.g. K_DEL in place of DEL
const FSK_KEEP_X_KEY: c_int = 0x02; // don't translate xHome to Home key
const FSK_IN_STRING: c_int = 0x04; // in string, double quote is escaped
const FSK_SIMPLIFY: c_int = 0x08; // simplify <C-H>, etc.

// CPO_* characters used in 'cpoptions' (from option_defs.h)
/// When 'B' is in 'cpoptions', backslash is NOT a special character
const CPO_BSLASH: c_int = b'B' as c_int;

// REPTERM_* flags for replace_termcodes
const REPTERM_FROM_PART: c_int = 1;
const REPTERM_DO_LT: c_int = 2;
#[allow(dead_code)]
const REPTERM_NO_SPECIAL: c_int = 4;
const REPTERM_NO_SIMPLIFY: c_int = 8;

// Special key extra codes for replace_termcodes
const KE_SNR: c_int = 82; // <SNR> script-local prefix

// Ctrl-V character (for escape handling)
const CTRL_V: u8 = 0x16;

// STR2NR constants for vim_str2nr
const STR2NR_BIN: c_int = 0x01;
const STR2NR_OCT: c_int = 0x02;
const STR2NR_HEX: c_int = 0x04;
const STR2NR_OOCT: c_int = 0x40;
const STR2NR_ALL: c_int = STR2NR_BIN | STR2NR_OCT | STR2NR_HEX | STR2NR_OOCT;

// KE_KDEL constant (keycodes.h:193; KE_KINS=79, KE_KDEL=80)
const KE_KDEL: c_int = 80;
const _: () = assert!(KE_KDEL == 80 && KE_KINS == 79, "keycodes.h KE_KDEL/KE_KINS");

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
#[allow(dead_code)]
const K_PAGEUP: c_int = termcap2key(b'k' as c_int, b'P' as c_int);
#[allow(dead_code)]
const K_PAGEDOWN: c_int = termcap2key(b'k' as c_int, b'N' as c_int);
#[allow(dead_code)]
const K_KPAGEUP: c_int = termcap2key(b'K' as c_int, b'3' as c_int);
#[allow(dead_code)]
const K_KPAGEDOWN: c_int = termcap2key(b'K' as c_int, b'5' as c_int);
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

// Special event/script keys
#[allow(dead_code)]
const K_EVENT: c_int = termcap2key(KS_EXTRA, KE_EVENT);
#[allow(dead_code)]
const K_COMMAND: c_int = termcap2key(KS_EXTRA, KE_COMMAND);
#[allow(dead_code)]
const K_LUA: c_int = termcap2key(KS_EXTRA, KE_LUA);

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
const K_BS: c_int = termcap2key(b'k' as c_int, b'b' as c_int);
const K_KDEL: c_int = termcap2key(KS_EXTRA, KE_KDEL);

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

// ASCII character constants (from ascii_defs.h)
#[allow(dead_code)]
const NL: c_int = 0x0A; // '\012'
#[allow(dead_code)]
const CAR: c_int = 0x0D; // '\015'
#[allow(dead_code)]
const ESC: c_int = 0x1B; // '\033'
#[allow(dead_code)]
const CSI: c_int = 0x9B; // 0x9b

// KE_ constants for special keys not yet defined above
#[allow(dead_code)]
const KE_KINS: c_int = 79;
#[allow(dead_code)]
const KE_PLUG: c_int = 83;
#[allow(dead_code)]
const KE_DROP: c_int = 95;
#[allow(dead_code)]
const KS_MOUSE: c_int = 251;

// Additional K_* constants from keycodes.h
// These are used in KEY_NAMES_TABLE; allow dead_code because static
// initializers don't count as "usage" for this lint.
#[allow(dead_code)]
const K_HELP: c_int = termcap2key(b'%' as c_int, b'1' as c_int);
#[allow(dead_code)]
const K_UNDO: c_int = termcap2key(b'&' as c_int, b'8' as c_int);
#[allow(dead_code)]
const K_FIND: c_int = termcap2key(b'@' as c_int, b'0' as c_int);
#[allow(dead_code)]
const K_KSELECT: c_int = termcap2key(b'*' as c_int, b'6' as c_int);
#[allow(dead_code)]
const K_KINS: c_int = termcap2key(KS_EXTRA, KE_KINS);
#[allow(dead_code)]
const K_KHOME: c_int = termcap2key(b'K' as c_int, b'1' as c_int);
#[allow(dead_code)]
const K_KEND: c_int = termcap2key(b'K' as c_int, b'4' as c_int);
#[allow(dead_code)]
const K_KUP: c_int = termcap2key(b'K' as c_int, b'u' as c_int);
#[allow(dead_code)]
const K_KDOWN: c_int = termcap2key(b'K' as c_int, b'd' as c_int);
#[allow(dead_code)]
const K_KLEFT: c_int = termcap2key(b'K' as c_int, b'l' as c_int);
#[allow(dead_code)]
const K_KRIGHT: c_int = termcap2key(b'K' as c_int, b'r' as c_int);
#[allow(dead_code)]
const K_KORIGIN: c_int = termcap2key(b'K' as c_int, b'2' as c_int);
#[allow(dead_code)]
const K_KPLUS: c_int = termcap2key(b'K' as c_int, b'6' as c_int);
#[allow(dead_code)]
const K_KMINUS: c_int = termcap2key(b'K' as c_int, b'7' as c_int);
#[allow(dead_code)]
const K_KDIVIDE: c_int = termcap2key(b'K' as c_int, b'8' as c_int);
#[allow(dead_code)]
const K_KMULTIPLY: c_int = termcap2key(b'K' as c_int, b'9' as c_int);
#[allow(dead_code)]
const K_KENTER: c_int = termcap2key(b'K' as c_int, b'A' as c_int);
#[allow(dead_code)]
const K_KPOINT: c_int = termcap2key(b'K' as c_int, b'B' as c_int);
#[allow(dead_code)]
const K_K0: c_int = termcap2key(b'K' as c_int, b'C' as c_int);
#[allow(dead_code)]
const K_K1: c_int = termcap2key(b'K' as c_int, b'D' as c_int);
#[allow(dead_code)]
const K_K2: c_int = termcap2key(b'K' as c_int, b'E' as c_int);
#[allow(dead_code)]
const K_K3: c_int = termcap2key(b'K' as c_int, b'F' as c_int);
#[allow(dead_code)]
const K_K4: c_int = termcap2key(b'K' as c_int, b'G' as c_int);
#[allow(dead_code)]
const K_K5: c_int = termcap2key(b'K' as c_int, b'H' as c_int);
#[allow(dead_code)]
const K_K6: c_int = termcap2key(b'K' as c_int, b'I' as c_int);
#[allow(dead_code)]
const K_K7: c_int = termcap2key(b'K' as c_int, b'J' as c_int);
#[allow(dead_code)]
const K_K8: c_int = termcap2key(b'K' as c_int, b'K' as c_int);
#[allow(dead_code)]
const K_K9: c_int = termcap2key(b'K' as c_int, b'L' as c_int);
#[allow(dead_code)]
const K_KCOMMA: c_int = termcap2key(b'K' as c_int, b'M' as c_int);
#[allow(dead_code)]
const K_KEQUAL: c_int = termcap2key(b'K' as c_int, b'N' as c_int);
#[allow(dead_code)]
const K_MOUSE: c_int = termcap2key(KS_MOUSE, KE_FILLER);
#[allow(dead_code)]
const K_IGNORE: c_int = termcap2key(KS_EXTRA, KE_IGNORE);
#[allow(dead_code)]
const K_SNR: c_int = termcap2key(KS_EXTRA, KE_SNR);
#[allow(dead_code)]
const K_PLUG: c_int = termcap2key(KS_EXTRA, KE_PLUG);
#[allow(dead_code)]
const K_DROP: c_int = termcap2key(KS_EXTRA, KE_DROP);

// Extended function keys F38-F63 (from keycodes.h)
#[allow(dead_code)]
const K_F38: c_int = termcap2key(b'F' as c_int, b'S' as c_int);
#[allow(dead_code)]
const K_F39: c_int = termcap2key(b'F' as c_int, b'T' as c_int);
#[allow(dead_code)]
const K_F40: c_int = termcap2key(b'F' as c_int, b'U' as c_int);
#[allow(dead_code)]
const K_F41: c_int = termcap2key(b'F' as c_int, b'V' as c_int);
#[allow(dead_code)]
const K_F42: c_int = termcap2key(b'F' as c_int, b'W' as c_int);
#[allow(dead_code)]
const K_F43: c_int = termcap2key(b'F' as c_int, b'X' as c_int);
#[allow(dead_code)]
const K_F44: c_int = termcap2key(b'F' as c_int, b'Y' as c_int);
#[allow(dead_code)]
const K_F45: c_int = termcap2key(b'F' as c_int, b'Z' as c_int);
#[allow(dead_code)]
const K_F46: c_int = termcap2key(b'F' as c_int, b'a' as c_int);
#[allow(dead_code)]
const K_F47: c_int = termcap2key(b'F' as c_int, b'b' as c_int);
#[allow(dead_code)]
const K_F48: c_int = termcap2key(b'F' as c_int, b'c' as c_int);
#[allow(dead_code)]
const K_F49: c_int = termcap2key(b'F' as c_int, b'd' as c_int);
#[allow(dead_code)]
const K_F50: c_int = termcap2key(b'F' as c_int, b'e' as c_int);
#[allow(dead_code)]
const K_F51: c_int = termcap2key(b'F' as c_int, b'f' as c_int);
#[allow(dead_code)]
const K_F52: c_int = termcap2key(b'F' as c_int, b'g' as c_int);
#[allow(dead_code)]
const K_F53: c_int = termcap2key(b'F' as c_int, b'h' as c_int);
#[allow(dead_code)]
const K_F54: c_int = termcap2key(b'F' as c_int, b'i' as c_int);
#[allow(dead_code)]
const K_F55: c_int = termcap2key(b'F' as c_int, b'j' as c_int);
#[allow(dead_code)]
const K_F56: c_int = termcap2key(b'F' as c_int, b'k' as c_int);
#[allow(dead_code)]
const K_F57: c_int = termcap2key(b'F' as c_int, b'l' as c_int);
#[allow(dead_code)]
const K_F58: c_int = termcap2key(b'F' as c_int, b'm' as c_int);
#[allow(dead_code)]
const K_F59: c_int = termcap2key(b'F' as c_int, b'n' as c_int);
#[allow(dead_code)]
const K_F60: c_int = termcap2key(b'F' as c_int, b'o' as c_int);
#[allow(dead_code)]
const K_F61: c_int = termcap2key(b'F' as c_int, b'p' as c_int);
#[allow(dead_code)]
const K_F62: c_int = termcap2key(b'F' as c_int, b'q' as c_int);
#[allow(dead_code)]
const K_F63: c_int = termcap2key(b'F' as c_int, b'r' as c_int);

// =============================================================================
// Key names table (Rust-native replacement for C key_names_table)
// =============================================================================
//
// This table is a port of the generated `key_names_table` in
// `build/src/nvim/auto/keycode_names.generated.h`, which is produced by
// `src/gen/gen_keycodes.lua` from `src/nvim/keycodes.lua`.
//
// The ordering is determined by `gen_keycodes.lua`'s `hashy` sorting algorithm
// (sorted by lowercase name, then by hash bucket). The order MUST match the
// generated header exactly since `key_names_table_hash()` returns indices into
// this array.
//
// When new special keys are added to `keycodes.lua`, this table must also be
// updated to match the newly generated header.

struct KeyNameEntry {
    key: c_int,
    is_alt: bool,
    name: &'static str,
}

// 187 entries matching keycode_names.generated.h order exactly
static KEY_NAMES_TABLE: &[KeyNameEntry] = &[
    KeyNameEntry {
        key: K_K0,
        is_alt: false,
        name: "k0",
    },
    KeyNameEntry {
        key: K_F1,
        is_alt: false,
        name: "F1",
    },
    KeyNameEntry {
        key: K_K1,
        is_alt: false,
        name: "k1",
    },
    KeyNameEntry {
        key: K_F2,
        is_alt: false,
        name: "F2",
    },
    KeyNameEntry {
        key: K_K2,
        is_alt: false,
        name: "k2",
    },
    KeyNameEntry {
        key: K_F3,
        is_alt: false,
        name: "F3",
    },
    KeyNameEntry {
        key: K_K3,
        is_alt: false,
        name: "k3",
    },
    KeyNameEntry {
        key: K_F4,
        is_alt: false,
        name: "F4",
    },
    KeyNameEntry {
        key: K_K4,
        is_alt: false,
        name: "k4",
    },
    KeyNameEntry {
        key: K_F5,
        is_alt: false,
        name: "F5",
    },
    KeyNameEntry {
        key: K_K5,
        is_alt: false,
        name: "k5",
    },
    KeyNameEntry {
        key: K_F6,
        is_alt: false,
        name: "F6",
    },
    KeyNameEntry {
        key: K_K6,
        is_alt: false,
        name: "k6",
    },
    KeyNameEntry {
        key: K_F7,
        is_alt: false,
        name: "F7",
    },
    KeyNameEntry {
        key: K_K7,
        is_alt: false,
        name: "k7",
    },
    KeyNameEntry {
        key: K_F8,
        is_alt: false,
        name: "F8",
    },
    KeyNameEntry {
        key: K_K8,
        is_alt: false,
        name: "k8",
    },
    KeyNameEntry {
        key: K_F9,
        is_alt: false,
        name: "F9",
    },
    KeyNameEntry {
        key: K_K9,
        is_alt: false,
        name: "k9",
    },
    KeyNameEntry {
        key: NL,
        is_alt: true,
        name: "LF",
    },
    KeyNameEntry {
        key: NL,
        is_alt: false,
        name: "NL",
    },
    KeyNameEntry {
        key: K_UP,
        is_alt: false,
        name: "Up",
    },
    KeyNameEntry {
        key: CAR,
        is_alt: false,
        name: "CR",
    },
    KeyNameEntry {
        key: K_BS,
        is_alt: false,
        name: "BS",
    },
    KeyNameEntry {
        key: '<' as c_int,
        is_alt: false,
        name: "lt",
    },
    KeyNameEntry {
        key: K_F10,
        is_alt: false,
        name: "F10",
    },
    KeyNameEntry {
        key: K_F20,
        is_alt: false,
        name: "F20",
    },
    KeyNameEntry {
        key: K_F30,
        is_alt: false,
        name: "F30",
    },
    KeyNameEntry {
        key: K_F40,
        is_alt: false,
        name: "F40",
    },
    KeyNameEntry {
        key: K_F50,
        is_alt: false,
        name: "F50",
    },
    KeyNameEntry {
        key: K_F60,
        is_alt: false,
        name: "F60",
    },
    KeyNameEntry {
        key: K_KINS,
        is_alt: true,
        name: "KP0",
    },
    KeyNameEntry {
        key: K_F11,
        is_alt: false,
        name: "F11",
    },
    KeyNameEntry {
        key: K_F21,
        is_alt: false,
        name: "F21",
    },
    KeyNameEntry {
        key: K_F31,
        is_alt: false,
        name: "F31",
    },
    KeyNameEntry {
        key: K_F41,
        is_alt: false,
        name: "F41",
    },
    KeyNameEntry {
        key: K_F51,
        is_alt: false,
        name: "F51",
    },
    KeyNameEntry {
        key: K_F61,
        is_alt: false,
        name: "F61",
    },
    KeyNameEntry {
        key: K_KEND,
        is_alt: true,
        name: "KP1",
    },
    KeyNameEntry {
        key: K_XF1,
        is_alt: false,
        name: "xF1",
    },
    KeyNameEntry {
        key: K_F12,
        is_alt: false,
        name: "F12",
    },
    KeyNameEntry {
        key: K_F22,
        is_alt: false,
        name: "F22",
    },
    KeyNameEntry {
        key: K_F32,
        is_alt: false,
        name: "F32",
    },
    KeyNameEntry {
        key: K_F42,
        is_alt: false,
        name: "F42",
    },
    KeyNameEntry {
        key: K_F52,
        is_alt: false,
        name: "F52",
    },
    KeyNameEntry {
        key: K_F62,
        is_alt: false,
        name: "F62",
    },
    KeyNameEntry {
        key: K_KDOWN,
        is_alt: true,
        name: "KP2",
    },
    KeyNameEntry {
        key: K_XF2,
        is_alt: false,
        name: "xF2",
    },
    KeyNameEntry {
        key: K_F13,
        is_alt: false,
        name: "F13",
    },
    KeyNameEntry {
        key: K_F23,
        is_alt: false,
        name: "F23",
    },
    KeyNameEntry {
        key: K_F33,
        is_alt: false,
        name: "F33",
    },
    KeyNameEntry {
        key: K_F43,
        is_alt: false,
        name: "F43",
    },
    KeyNameEntry {
        key: K_F53,
        is_alt: false,
        name: "F53",
    },
    KeyNameEntry {
        key: K_F63,
        is_alt: false,
        name: "F63",
    },
    KeyNameEntry {
        key: K_KPAGEDOWN,
        is_alt: true,
        name: "KP3",
    },
    KeyNameEntry {
        key: K_XF3,
        is_alt: false,
        name: "xF3",
    },
    KeyNameEntry {
        key: K_F14,
        is_alt: false,
        name: "F14",
    },
    KeyNameEntry {
        key: K_F24,
        is_alt: false,
        name: "F24",
    },
    KeyNameEntry {
        key: K_F34,
        is_alt: false,
        name: "F34",
    },
    KeyNameEntry {
        key: K_F44,
        is_alt: false,
        name: "F44",
    },
    KeyNameEntry {
        key: K_F54,
        is_alt: false,
        name: "F54",
    },
    KeyNameEntry {
        key: K_KLEFT,
        is_alt: true,
        name: "KP4",
    },
    KeyNameEntry {
        key: K_XF4,
        is_alt: false,
        name: "xF4",
    },
    KeyNameEntry {
        key: K_F15,
        is_alt: false,
        name: "F15",
    },
    KeyNameEntry {
        key: K_F25,
        is_alt: false,
        name: "F25",
    },
    KeyNameEntry {
        key: K_F35,
        is_alt: false,
        name: "F35",
    },
    KeyNameEntry {
        key: K_F45,
        is_alt: false,
        name: "F45",
    },
    KeyNameEntry {
        key: K_F55,
        is_alt: false,
        name: "F55",
    },
    KeyNameEntry {
        key: K_KORIGIN,
        is_alt: true,
        name: "KP5",
    },
    KeyNameEntry {
        key: K_F16,
        is_alt: false,
        name: "F16",
    },
    KeyNameEntry {
        key: K_F26,
        is_alt: false,
        name: "F26",
    },
    KeyNameEntry {
        key: K_F36,
        is_alt: false,
        name: "F36",
    },
    KeyNameEntry {
        key: K_F46,
        is_alt: false,
        name: "F46",
    },
    KeyNameEntry {
        key: K_F56,
        is_alt: false,
        name: "F56",
    },
    KeyNameEntry {
        key: K_KRIGHT,
        is_alt: true,
        name: "KP6",
    },
    KeyNameEntry {
        key: K_F17,
        is_alt: false,
        name: "F17",
    },
    KeyNameEntry {
        key: K_F27,
        is_alt: false,
        name: "F27",
    },
    KeyNameEntry {
        key: K_F37,
        is_alt: false,
        name: "F37",
    },
    KeyNameEntry {
        key: K_F47,
        is_alt: false,
        name: "F47",
    },
    KeyNameEntry {
        key: K_F57,
        is_alt: false,
        name: "F57",
    },
    KeyNameEntry {
        key: K_KHOME,
        is_alt: true,
        name: "KP7",
    },
    KeyNameEntry {
        key: K_F18,
        is_alt: false,
        name: "F18",
    },
    KeyNameEntry {
        key: K_F28,
        is_alt: false,
        name: "F28",
    },
    KeyNameEntry {
        key: K_F38,
        is_alt: false,
        name: "F38",
    },
    KeyNameEntry {
        key: K_F48,
        is_alt: false,
        name: "F48",
    },
    KeyNameEntry {
        key: K_F58,
        is_alt: false,
        name: "F58",
    },
    KeyNameEntry {
        key: K_KUP,
        is_alt: true,
        name: "KP8",
    },
    KeyNameEntry {
        key: K_F19,
        is_alt: false,
        name: "F19",
    },
    KeyNameEntry {
        key: K_F29,
        is_alt: false,
        name: "F29",
    },
    KeyNameEntry {
        key: K_F39,
        is_alt: false,
        name: "F39",
    },
    KeyNameEntry {
        key: K_F49,
        is_alt: false,
        name: "F49",
    },
    KeyNameEntry {
        key: K_F59,
        is_alt: false,
        name: "F59",
    },
    KeyNameEntry {
        key: K_KPAGEUP,
        is_alt: true,
        name: "KP9",
    },
    KeyNameEntry {
        key: TAB,
        is_alt: false,
        name: "Tab",
    },
    KeyNameEntry {
        key: K_TAB,
        is_alt: false,
        name: "Tab",
    },
    KeyNameEntry {
        key: ESC,
        is_alt: false,
        name: "Esc",
    },
    KeyNameEntry {
        key: K_COMMAND,
        is_alt: false,
        name: "Cmd",
    },
    KeyNameEntry {
        key: K_END,
        is_alt: false,
        name: "End",
    },
    KeyNameEntry {
        key: CSI,
        is_alt: false,
        name: "CSI",
    },
    KeyNameEntry {
        key: K_DEL,
        is_alt: false,
        name: "Del",
    },
    KeyNameEntry {
        key: K_ZERO,
        is_alt: false,
        name: "Nul",
    },
    KeyNameEntry {
        key: K_KUP,
        is_alt: false,
        name: "kUp",
    },
    KeyNameEntry {
        key: K_XUP,
        is_alt: false,
        name: "xUp",
    },
    KeyNameEntry {
        key: '|' as c_int,
        is_alt: false,
        name: "Bar",
    },
    KeyNameEntry {
        key: K_SNR,
        is_alt: false,
        name: "SNR",
    },
    KeyNameEntry {
        key: K_INS,
        is_alt: true,
        name: "Ins",
    },
    KeyNameEntry {
        key: K_DOWN,
        is_alt: false,
        name: "Down",
    },
    KeyNameEntry {
        key: K_DROP,
        is_alt: false,
        name: "Drop",
    },
    KeyNameEntry {
        key: K_FIND,
        is_alt: false,
        name: "Find",
    },
    KeyNameEntry {
        key: K_HELP,
        is_alt: false,
        name: "Help",
    },
    KeyNameEntry {
        key: K_HOME,
        is_alt: false,
        name: "Home",
    },
    KeyNameEntry {
        key: K_KDEL,
        is_alt: false,
        name: "kDel",
    },
    KeyNameEntry {
        key: K_KEND,
        is_alt: false,
        name: "kEnd",
    },
    KeyNameEntry {
        key: K_LEFT,
        is_alt: false,
        name: "Left",
    },
    KeyNameEntry {
        key: K_PLUG,
        is_alt: false,
        name: "Plug",
    },
    KeyNameEntry {
        key: K_UNDO,
        is_alt: false,
        name: "Undo",
    },
    KeyNameEntry {
        key: K_XEND,
        is_alt: false,
        name: "xEnd",
    },
    KeyNameEntry {
        key: K_ZEND,
        is_alt: false,
        name: "zEnd",
    },
    KeyNameEntry {
        key: K_KDOWN,
        is_alt: false,
        name: "kDown",
    },
    KeyNameEntry {
        key: K_XDOWN,
        is_alt: false,
        name: "xDown",
    },
    KeyNameEntry {
        key: K_KHOME,
        is_alt: false,
        name: "kHome",
    },
    KeyNameEntry {
        key: K_XHOME,
        is_alt: false,
        name: "xHome",
    },
    KeyNameEntry {
        key: K_ZHOME,
        is_alt: false,
        name: "zHome",
    },
    KeyNameEntry {
        key: K_RIGHT,
        is_alt: false,
        name: "Right",
    },
    KeyNameEntry {
        key: K_KLEFT,
        is_alt: false,
        name: "kLeft",
    },
    KeyNameEntry {
        key: K_XLEFT,
        is_alt: false,
        name: "xLeft",
    },
    KeyNameEntry {
        key: CAR,
        is_alt: true,
        name: "Enter",
    },
    KeyNameEntry {
        key: K_MOUSE,
        is_alt: false,
        name: "Mouse",
    },
    KeyNameEntry {
        key: K_KDIVIDE,
        is_alt: true,
        name: "KPDiv",
    },
    KeyNameEntry {
        key: K_KPLUS,
        is_alt: false,
        name: "kPlus",
    },
    KeyNameEntry {
        key: ' ' as c_int,
        is_alt: false,
        name: "Space",
    },
    KeyNameEntry {
        key: ESC,
        is_alt: true,
        name: "Escape",
    },
    KeyNameEntry {
        key: K_X1DRAG,
        is_alt: false,
        name: "X1Drag",
    },
    KeyNameEntry {
        key: K_X2DRAG,
        is_alt: false,
        name: "X2Drag",
    },
    KeyNameEntry {
        key: K_PAGEUP,
        is_alt: false,
        name: "PageUp",
    },
    KeyNameEntry {
        key: K_KMINUS,
        is_alt: false,
        name: "kMinus",
    },
    KeyNameEntry {
        key: K_KRIGHT,
        is_alt: false,
        name: "kRight",
    },
    KeyNameEntry {
        key: K_XRIGHT,
        is_alt: false,
        name: "xRight",
    },
    KeyNameEntry {
        key: '\\' as c_int,
        is_alt: false,
        name: "Bslash",
    },
    KeyNameEntry {
        key: K_DEL,
        is_alt: true,
        name: "Delete",
    },
    KeyNameEntry {
        key: K_KSELECT,
        is_alt: false,
        name: "Select",
    },
    KeyNameEntry {
        key: K_KMULTIPLY,
        is_alt: true,
        name: "KPMult",
    },
    KeyNameEntry {
        key: K_IGNORE,
        is_alt: false,
        name: "Ignore",
    },
    KeyNameEntry {
        key: K_KENTER,
        is_alt: false,
        name: "kEnter",
    },
    KeyNameEntry {
        key: K_KCOMMA,
        is_alt: false,
        name: "kComma",
    },
    KeyNameEntry {
        key: K_KPOINT,
        is_alt: false,
        name: "kPoint",
    },
    KeyNameEntry {
        key: K_KPLUS,
        is_alt: true,
        name: "KPPlus",
    },
    KeyNameEntry {
        key: K_KEQUAL,
        is_alt: false,
        name: "kEqual",
    },
    KeyNameEntry {
        key: K_INS,
        is_alt: false,
        name: "Insert",
    },
    KeyNameEntry {
        key: CAR,
        is_alt: true,
        name: "Return",
    },
    KeyNameEntry {
        key: K_KPAGEUP,
        is_alt: false,
        name: "kPageUp",
    },
    KeyNameEntry {
        key: K_KCOMMA,
        is_alt: true,
        name: "KPComma",
    },
    KeyNameEntry {
        key: K_KENTER,
        is_alt: true,
        name: "KPEnter",
    },
    KeyNameEntry {
        key: K_KDIVIDE,
        is_alt: false,
        name: "kDivide",
    },
    KeyNameEntry {
        key: K_KMINUS,
        is_alt: true,
        name: "KPMinus",
    },
    KeyNameEntry {
        key: K_X1MOUSE,
        is_alt: false,
        name: "X1Mouse",
    },
    KeyNameEntry {
        key: K_X2MOUSE,
        is_alt: false,
        name: "X2Mouse",
    },
    KeyNameEntry {
        key: K_KINS,
        is_alt: false,
        name: "kInsert",
    },
    KeyNameEntry {
        key: K_KORIGIN,
        is_alt: false,
        name: "kOrigin",
    },
    KeyNameEntry {
        key: K_MOUSEUP,
        is_alt: true,
        name: "MouseUp",
    },
    KeyNameEntry {
        key: NL,
        is_alt: true,
        name: "NewLine",
    },
    KeyNameEntry {
        key: K_KEQUAL,
        is_alt: true,
        name: "KPEquals",
    },
    KeyNameEntry {
        key: K_LEFTDRAG,
        is_alt: false,
        name: "LeftDrag",
    },
    KeyNameEntry {
        key: K_PAGEDOWN,
        is_alt: false,
        name: "PageDown",
    },
    KeyNameEntry {
        key: NL,
        is_alt: true,
        name: "LineFeed",
    },
    KeyNameEntry {
        key: K_KDEL,
        is_alt: true,
        name: "KPPeriod",
    },
    KeyNameEntry {
        key: K_BS,
        is_alt: true,
        name: "BackSpace",
    },
    KeyNameEntry {
        key: K_KMULTIPLY,
        is_alt: false,
        name: "kMultiply",
    },
    KeyNameEntry {
        key: K_KPAGEDOWN,
        is_alt: false,
        name: "kPageDown",
    },
    KeyNameEntry {
        key: K_LEFTMOUSE,
        is_alt: false,
        name: "LeftMouse",
    },
    KeyNameEntry {
        key: K_MOUSEDOWN,
        is_alt: true,
        name: "MouseDown",
    },
    KeyNameEntry {
        key: K_MOUSEMOVE,
        is_alt: false,
        name: "MouseMove",
    },
    KeyNameEntry {
        key: K_RIGHTDRAG,
        is_alt: false,
        name: "RightDrag",
    },
    KeyNameEntry {
        key: K_X1RELEASE,
        is_alt: false,
        name: "X1Release",
    },
    KeyNameEntry {
        key: K_X2RELEASE,
        is_alt: false,
        name: "X2Release",
    },
    KeyNameEntry {
        key: K_MIDDLEDRAG,
        is_alt: false,
        name: "MiddleDrag",
    },
    KeyNameEntry {
        key: K_RIGHTMOUSE,
        is_alt: false,
        name: "RightMouse",
    },
    KeyNameEntry {
        key: K_MIDDLEMOUSE,
        is_alt: false,
        name: "MiddleMouse",
    },
    KeyNameEntry {
        key: K_LEFTMOUSE_NM,
        is_alt: false,
        name: "LeftMouseNM",
    },
    KeyNameEntry {
        key: K_LEFTRELEASE,
        is_alt: false,
        name: "LeftRelease",
    },
    KeyNameEntry {
        key: K_RIGHTRELEASE,
        is_alt: false,
        name: "RightRelease",
    },
    KeyNameEntry {
        key: K_LEFTRELEASE_NM,
        is_alt: false,
        name: "LeftReleaseNM",
    },
    KeyNameEntry {
        key: K_MIDDLERELEASE,
        is_alt: false,
        name: "MiddleRelease",
    },
    KeyNameEntry {
        key: K_MOUSEDOWN,
        is_alt: false,
        name: "ScrollWheelUp",
    },
    KeyNameEntry {
        key: K_MOUSEUP,
        is_alt: false,
        name: "ScrollWheelDown",
    },
    KeyNameEntry {
        key: K_MOUSERIGHT,
        is_alt: false,
        name: "ScrollWheelLeft",
    },
    KeyNameEntry {
        key: K_MOUSELEFT,
        is_alt: false,
        name: "ScrollWheelRight",
    },
];

/// Case-insensitive ASCII comparison (equivalent to `vim_strnicmp_asc` semantics).
/// Returns true if `a` and `b` are equal (case-insensitively) for the first `len` bytes.
fn ascii_strnicmp(a: &[u8], b: &[u8], len: usize) -> bool {
    if a.len() < len || b.len() < len {
        return false;
    }
    for i in 0..len {
        let ca = a[i].to_ascii_uppercase();
        let cb = b[i].to_ascii_uppercase();
        if ca != cb {
            return false;
        }
    }
    true
}

/// Lookup a special key code by name using the hash function ported from
/// `get_special_key_code_hash` in `keycode_names.generated.h`.
///
/// Returns the index into `KEY_NAMES_TABLE`, or `None` if not found.
#[allow(clippy::too_many_lines)]
fn key_names_table_hash(name: &[u8]) -> Option<usize> {
    let len = name.len();
    let (low, high): (usize, usize) = match len {
        2 => match name[1] {
            b'0' => (0, 1),
            b'1' => (1, 3),
            b'2' => (3, 5),
            b'3' => (5, 7),
            b'4' => (7, 9),
            b'5' => (9, 11),
            b'6' => (11, 13),
            b'7' => (13, 15),
            b'8' => (15, 17),
            b'9' => (17, 19),
            b'F' | b'f' => (19, 20),
            b'L' | b'l' => (20, 21),
            b'P' | b'p' => (21, 22),
            b'R' | b'r' => (22, 23),
            b'S' | b's' => (23, 24),
            b'T' | b't' => (24, 25),
            _ => return None,
        },
        3 => match name[2] {
            b'0' => (25, 32),
            b'1' => (32, 40),
            b'2' => (40, 48),
            b'3' => (48, 56),
            b'4' => (56, 63),
            b'5' => (63, 69),
            b'6' => (69, 75),
            b'7' => (75, 81),
            b'8' => (81, 87),
            b'9' => (87, 93),
            b'B' | b'b' => (93, 95),
            b'C' | b'c' => (95, 96),
            b'D' | b'd' => (96, 98),
            b'I' | b'i' => (98, 99),
            b'L' | b'l' => (99, 101),
            b'P' | b'p' => (101, 103),
            b'R' | b'r' => (103, 105),
            b'S' | b's' => (105, 106),
            _ => return None,
        },
        4 => match name[0] {
            b'D' | b'd' => (106, 108),
            b'F' | b'f' => (108, 109),
            b'H' | b'h' => (109, 111),
            b'K' | b'k' => (111, 113),
            b'L' | b'l' => (113, 114),
            b'P' | b'p' => (114, 115),
            b'U' | b'u' => (115, 116),
            b'X' | b'x' => (116, 117),
            b'Z' | b'z' => (117, 118),
            _ => return None,
        },
        5 => match name[1] {
            b'D' | b'd' => (118, 120),
            b'H' | b'h' => (120, 123),
            b'I' | b'i' => (123, 124),
            b'L' | b'l' => (124, 126),
            b'N' | b'n' => (126, 127),
            b'O' | b'o' => (127, 128),
            b'P' | b'p' => (128, 131),
            _ => return None,
        },
        6 => match name[2] {
            b'C' | b'c' => (131, 132),
            b'D' | b'd' => (132, 134),
            b'G' | b'g' => (134, 135),
            b'I' | b'i' => (135, 138),
            b'L' | b'l' => (138, 141),
            b'M' | b'm' => (141, 142),
            b'N' | b'n' => (142, 144),
            b'O' | b'o' => (144, 146),
            b'P' | b'p' => (146, 147),
            b'Q' | b'q' => (147, 148),
            b'S' | b's' => (148, 149),
            b'T' | b't' => (149, 150),
            _ => return None,
        },
        7 => match name[2] {
            b'A' | b'a' => (150, 151),
            b'C' | b'c' => (151, 152),
            b'E' | b'e' => (152, 153),
            b'I' | b'i' => (153, 154),
            b'M' | b'm' => (154, 157),
            b'N' | b'n' => (157, 158),
            b'R' | b'r' => (158, 159),
            b'U' | b'u' => (159, 160),
            b'W' | b'w' => (160, 161),
            _ => return None,
        },
        8 => match name[2] {
            b'E' | b'e' => (161, 162),
            b'F' | b'f' => (162, 163),
            b'G' | b'g' => (163, 164),
            b'N' | b'n' => (164, 165),
            b'P' | b'p' => (165, 166),
            _ => return None,
        },
        9 => match name[0] {
            b'B' | b'b' => (166, 167),
            b'K' | b'k' => (167, 169),
            b'L' | b'l' => (169, 170),
            b'M' | b'm' => (170, 172),
            b'R' | b'r' => (172, 173),
            b'X' | b'x' => (173, 175),
            _ => return None,
        },
        10 => match name[0] {
            b'M' | b'm' => (175, 176),
            b'R' | b'r' => (176, 177),
            _ => return None,
        },
        11 => match name[4] {
            b'L' | b'l' => (177, 178),
            b'M' | b'm' => (178, 179),
            b'R' | b'r' => (179, 180),
            _ => return None,
        },
        12 => (180, 181),
        13 => match name[0] {
            b'L' | b'l' => (181, 182),
            b'M' | b'm' => (182, 183),
            b'S' | b's' => (183, 184),
            _ => return None,
        },
        15 => match name[11] {
            b'D' | b'd' => (184, 185),
            b'L' | b'l' => (185, 186),
            _ => return None,
        },
        16 => (186, 187),
        _ => return None,
    };
    KEY_NAMES_TABLE[low..high]
        .iter()
        .enumerate()
        .find(|(_, entry)| ascii_strnicmp(name, entry.name.as_bytes(), len))
        .map(|(i, _)| low + i)
}

/// Modifier mask table entry (for `name_to_mod_mask`)
struct ModMaskEntry {
    mod_flag: c_int,
    name: u8,
}

/// Table mapping modifier names to modifier flags (for `name_to_mod_mask`)
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

/// Full modifier mask table entry (for `get_special_key_name`)
/// Contains both `mod_mask` (for grouping) and `mod_flag` (specific value)
struct ModMaskTableEntry {
    mod_mask: c_int,
    mod_flag: c_int,
    name: u8,
}

/// Table for translating modifiers to string representation.
/// The 'A' entry is the sentinel - iteration stops before it.
static MOD_MASK_TABLE_FULL: &[ModMaskTableEntry] = &[
    ModMaskTableEntry {
        mod_mask: MOD_MASK_ALT,
        mod_flag: MOD_MASK_ALT,
        name: b'M',
    },
    ModMaskTableEntry {
        mod_mask: MOD_MASK_META,
        mod_flag: MOD_MASK_META,
        name: b'T',
    },
    ModMaskTableEntry {
        mod_mask: MOD_MASK_CTRL,
        mod_flag: MOD_MASK_CTRL,
        name: b'C',
    },
    ModMaskTableEntry {
        mod_mask: MOD_MASK_SHIFT,
        mod_flag: MOD_MASK_SHIFT,
        name: b'S',
    },
    ModMaskTableEntry {
        mod_mask: MOD_MASK_MULTI_CLICK,
        mod_flag: MOD_MASK_2CLICK,
        name: b'2',
    },
    ModMaskTableEntry {
        mod_mask: MOD_MASK_MULTI_CLICK,
        mod_flag: MOD_MASK_3CLICK,
        name: b'3',
    },
    ModMaskTableEntry {
        mod_mask: MOD_MASK_MULTI_CLICK,
        mod_flag: MOD_MASK_4CLICK,
        name: b'4',
    },
    ModMaskTableEntry {
        mod_mask: MOD_MASK_CMD,
        mod_flag: MOD_MASK_CMD,
        name: b'D',
    },
    // 'A' is the sentinel - iteration stops before this entry
    ModMaskTableEntry {
        mod_mask: MOD_MASK_ALT,
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

// =============================================================================
// C accessor functions for key_names_table
// =============================================================================

extern "C" {
    /// Translate character to printable string representation.
    /// Returns a pointer to a static buffer.
    fn transchar(c: c_int) -> *const std::ffi::c_char;

    /// Get length of UTF-8 character with max bytes check.
    fn utfc_ptr2len_len(p: *const std::ffi::c_char, maxlen: c_int) -> c_int;

    /// Get length of UTF-8 character.
    fn utfc_ptr2len(p: *const std::ffi::c_char) -> c_int;

    /// Emit error message.
    fn emsg(s: *const std::ffi::c_char) -> c_int;

    /// Translate a message string.
    fn gettext(msgid: *const std::ffi::c_char) -> *const std::ffi::c_char;

    /// Get `e_invarg` error string.
    static e_invarg: *const std::ffi::c_char;

    /// "Using <SID> not in a script context" error string.
    static e_usingsid: std::ffi::c_char;

    /// Current script context (`sctx_T` in C).
    static mut current_sctx: SctxT;

    /// Get value of a variable by name (e.g. `g:mapleader`).
    fn rs_get_var_value(name: *const std::ffi::c_char) -> *const std::ffi::c_char;

    /// Search for character `c` in string `s`. Returns pointer to first
    /// occurrence, or NULL if not found.
    fn vim_strchr(s: *const std::ffi::c_char, c: c_int) -> *mut std::ffi::c_char;
}

/// Try to find key "c" in the special key table.
///
/// # Returns
/// The index when found, -1 when not found.
#[must_use]
#[export_name = "find_special_key_in_table"]
pub extern "C" fn rs_find_special_key_in_table(c: c_int) -> c_int {
    for (i, entry) in KEY_NAMES_TABLE.iter().enumerate() {
        if c == entry.key && !entry.is_alt {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            return i as c_int;
        }
    }
    -1
}

/// Find the special key with the given name.
///
/// # Arguments
/// * `name` - Name of the special key. Does not have to end with NUL, it is
///   assumed to end before the first non-idchar. If name starts with "t_" the
///   next two characters are interpreted as a termcap name.
///
/// # Returns
/// Key code or 0 if not found.
///
/// # Safety
/// `name` must be a valid pointer to a C string with at least 4 readable bytes
/// if the name starts with "t_".
#[must_use]
#[export_name = "get_special_key_code"]
pub unsafe extern "C" fn rs_get_special_key_code(name: *const std::ffi::c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    let name_u8 = name.cast::<u8>();

    // Check for termcap name: t_xx
    if *name_u8 == b't' && *name_u8.add(1) == b'_' && *name_u8.add(2) != 0 && *name_u8.add(3) != 0 {
        return termcap2key(c_int::from(*name_u8.add(2)), c_int::from(*name_u8.add(3)));
    }

    // Find the end of the identifier
    let mut name_end = name_u8;
    while nvim_ascii::rs_ascii_isident(c_int::from(*name_end)) != 0 {
        name_end = name_end.add(1);
    }

    // name_end is always >= name_u8, so the offset is always non-negative
    #[allow(clippy::cast_sign_loss)]
    let len = name_end.offset_from(name_u8) as usize;
    let name_slice = std::slice::from_raw_parts(name_u8, len);
    key_names_table_hash(name_slice).map_or(0, |idx| KEY_NAMES_TABLE[idx].key)
}

/// Static buffer for `get_special_key_name` result.
/// This matches the C behavior of returning a pointer to a static buffer.
static mut SPECIAL_KEY_NAME_BUF: [u8; MAX_KEY_NAME_LEN + 1] = [0; MAX_KEY_NAME_LEN + 1];

/// Get a string representation of a key with modifiers.
///
/// Returns a string like `<C-S-Up>` for a key code with modifiers.
///
/// # Safety
/// This function uses a static mutable buffer. The returned pointer is valid
/// until the next call to this function.
#[export_name = "get_special_key_name"]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_get_special_key_name(
    mut c: c_int,
    mut modifiers: c_int,
) -> *mut std::ffi::c_char {
    let string = std::ptr::addr_of_mut!(SPECIAL_KEY_NAME_BUF).cast::<u8>();
    *string = b'<';
    let mut idx: usize = 1;

    // Key that stands for a normal character.
    if is_special(c) && key2termcap0(c) == KS_KEY {
        c = key2termcap1(c);
    }

    // Translate shifted special keys into unshifted keys and set modifier.
    // Same for CTRL and ALT modifiers.
    if is_special(c) {
        for entry in MODIFIER_KEYS_TABLE {
            let with_mod_0 = key2termcap0(entry.key_with_mod);
            let with_mod_1 = key2termcap1(entry.key_with_mod);
            if key2termcap0(c) == with_mod_0 && key2termcap1(c) == with_mod_1 {
                modifiers |= entry.mod_mask;
                c = entry.key_without_mod;
                break;
            }
        }
    }

    // Try to find the key in the special key table
    let mut table_idx = rs_find_special_key_in_table(c);

    // When not a known special key, and not a printable character, try to
    // extract modifiers.
    if c > 0 && nvim_mbyte::rs_utf_char2len(c) == 1 {
        if table_idx < 0
            && (!nvim_charset::rs_vim_isprintc(c) || (c & 0x7f) == c_int::from(b' '))
            && (c & 0x80) != 0
        {
            c &= 0x7f;
            modifiers |= MOD_MASK_ALT;
            // Try again, to find the un-alted key in the special key table
            table_idx = rs_find_special_key_in_table(c);
        }
        if table_idx < 0 && !nvim_charset::rs_vim_isprintc(c) && c < c_int::from(b' ') {
            c += c_int::from(b'@');
            modifiers |= MOD_MASK_CTRL;
        }
    }

    // Translate the modifier into a string (stop before 'A' sentinel)
    for entry in &MOD_MASK_TABLE_FULL[..MOD_MASK_TABLE_FULL.len() - 1] {
        if (modifiers & entry.mod_mask) == entry.mod_flag {
            *string.add(idx) = entry.name;
            idx += 1;
            *string.add(idx) = b'-';
            idx += 1;
        }
    }

    if table_idx < 0 {
        // Unknown special key, may output t_xx
        if is_special(c) {
            *string.add(idx) = b't';
            idx += 1;
            *string.add(idx) = b'_';
            idx += 1;
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            {
                *string.add(idx) = key2termcap0(c) as u8;
                idx += 1;
                *string.add(idx) = key2termcap1(c) as u8;
                idx += 1;
            }
        } else {
            // Not a special key, only modifiers, output directly.
            let len = nvim_mbyte::rs_utf_char2len(c);
            if len == 1 && nvim_charset::rs_vim_isprintc(c) {
                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                {
                    *string.add(idx) = c as u8;
                }
                idx += 1;
            } else if len > 1 {
                let written =
                    nvim_mbyte::rs_utf_char2bytes(c, string.add(idx).cast::<std::ffi::c_char>());
                #[allow(clippy::cast_sign_loss)]
                {
                    idx += written as usize;
                }
            } else {
                // Use transchar for unprintable characters
                let s = transchar(c);
                if !s.is_null() {
                    let mut p = s.cast::<u8>();
                    while *p != 0 {
                        *string.add(idx) = *p;
                        idx += 1;
                        p = p.add(1);
                    }
                }
            }
        }
    } else {
        // Use name of special key
        #[allow(clippy::cast_sign_loss)]
        let entry = &KEY_NAMES_TABLE[table_idx as usize];
        let entry_name = entry.name.as_bytes();
        let name_size = entry_name.len();
        if name_size + idx + 2 <= MAX_KEY_NAME_LEN {
            std::ptr::copy_nonoverlapping(entry_name.as_ptr(), string.add(idx), name_size);
            idx += name_size;
        }
    }

    *string.add(idx) = b'>';
    idx += 1;
    *string.add(idx) = 0; // NUL terminate

    string.cast::<std::ffi::c_char>()
}

/// Return the modifier mask bit corresponding to modifier name.
///
/// E.g. 'S' for shift, 'C' for ctrl, 'M' for alt/meta.
/// Returns 0 if the character doesn't correspond to a known modifier.
#[must_use]
#[export_name = "name_to_mod_mask"]
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
#[must_use]
#[export_name = "handle_x_keys"]
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
#[must_use]
#[export_name = "is_mouse_key"]
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
#[export_name = "simplify_key"]
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
#[export_name = "get_mouse_button"]
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
#[export_name = "vim_unescape_ks"]
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
#[export_name = "add_char2buf"]
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
#[export_name = "vim_strsave_escape_ks"]
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

/// Case-insensitive comparison of prefix.
/// Returns true if `s` starts with `prefix` (case-insensitive).
unsafe fn strnicmp_prefix(s: *const u8, prefix: &[u8]) -> bool {
    for (i, &p) in prefix.iter().enumerate() {
        let c = *s.add(i);
        if !c.eq_ignore_ascii_case(&p) {
            return false;
        }
    }
    true
}

/// Try translating a <> name to a key code.
///
/// # Arguments
/// * `srcp` - Pointer to pointer to the source string. Advanced past the <> name on success.
/// * `src_len` - Length of the source string.
/// * `modp` - Output pointer for modifier flags.
/// * `flags` - FSK_* flags.
/// * `did_simplify` - Output pointer, set to true if `FSK_SIMPLIFY` found `<C-H>`, etc.
///
/// # Returns
/// Key code or 0 if no match.
///
/// # Safety
/// - `srcp` must be a valid pointer to a valid C string pointer.
/// - `modp` must be a valid pointer.
/// - `did_simplify` may be null.
#[export_name = "find_special_key"]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_find_special_key(
    srcp: *mut *const std::ffi::c_char,
    src_len: usize,
    modp: *mut c_int,
    flags: c_int,
    did_simplify: *mut bool,
) -> c_int {
    if srcp.is_null() || modp.is_null() || (*srcp).is_null() {
        return 0;
    }

    if src_len == 0 {
        return 0;
    }

    let start = (*srcp).cast::<u8>();
    let end = start.add(src_len - 1);
    let in_string = (flags & FSK_IN_STRING) != 0;

    // Check for '<' at start
    if *start != b'<' {
        return 0;
    }

    let mut src = start;
    if *start.add(1) == b'*' {
        // <*xxx>: do not simplify
        src = src.add(1);
    }

    // Find end of modifier list
    let mut bp = src.add(1);
    let mut last_dash = src;

    while bp <= end && (*bp == b'-' || nvim_ascii::rs_ascii_isident(c_int::from(*bp)) != 0) {
        if *bp == b'-' {
            last_dash = bp;
            if bp.add(1) <= end {
                let l = utfc_ptr2len_len(bp.add(1).cast(), ptr_diff(end, bp) + 1);
                // Anything accepted, like <C-?>.
                // <C-"> or <M-"> are not special in strings as " is
                // the string delimiter. With a backslash it works: <M-\">
                #[allow(clippy::cast_sign_loss)]
                if ptr_diff(end, bp) > l
                    && !(in_string && *bp.add(1) == b'"')
                    && *bp.add(1 + l as usize) == b'>'
                {
                    #[allow(clippy::cast_sign_loss)]
                    {
                        bp = bp.add(l as usize);
                    }
                } else if ptr_diff(end, bp) > 2
                    && in_string
                    && *bp.add(1) == b'\\'
                    && *bp.add(2) == b'"'
                    && *bp.add(3) == b'>'
                {
                    bp = bp.add(2);
                }
            }
        }
        if ptr_diff(end, bp) > 3 && *bp == b't' && *bp.add(1) == b'_' {
            bp = bp.add(3); // skip t_xx, xx may be '-' or '>'
        } else if ptr_diff(end, bp) > 4 && strnicmp_prefix(bp, b"char-") {
            let mut l: c_int = 0;
            nvim_charset::rs_vim_str2nr(
                bp.add(5).cast(),
                std::ptr::null_mut(),
                &raw mut l,
                STR2NR_ALL,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                true,
                std::ptr::null_mut(),
            );
            if l == 0 {
                emsg(e_invarg);
                return 0;
            }
            #[allow(clippy::cast_sign_loss)]
            {
                bp = bp.add(l as usize + 5);
            }
            break;
        }
        bp = bp.add(1);
    }

    if bp <= end && *bp == b'>' {
        // Found matching '>'
        let end_of_name = bp.add(1);

        // Which modifiers are given?
        let mut modifiers: c_int = 0;
        bp = src.add(1);
        while bp < last_dash {
            if *bp != b'-' {
                let bit = rs_name_to_mod_mask(c_int::from(*bp));
                if bit == 0 {
                    break; // Illegal modifier name
                }
                modifiers |= bit;
            }
            bp = bp.add(1);
        }

        // Legal modifier name
        if bp >= last_dash {
            let key: c_int;

            if strnicmp_prefix(last_dash.add(1), b"char-")
                && nvim_ascii::rs_ascii_isdigit(c_int::from(*last_dash.add(6))) != 0
            {
                // <Char-123> or <Char-033> or <Char-0x33>
                let mut l: c_int = 0;
                let mut n: u64 = 0;
                nvim_charset::rs_vim_str2nr(
                    last_dash.add(6).cast(),
                    std::ptr::null_mut(),
                    &raw mut l,
                    STR2NR_ALL,
                    std::ptr::null_mut(),
                    &raw mut n,
                    0,
                    true,
                    std::ptr::null_mut(),
                );
                if l == 0 {
                    emsg(e_invarg);
                    return 0;
                }
                #[allow(clippy::cast_possible_truncation)]
                {
                    key = n as c_int;
                }
            } else {
                // Modifier with single letter, or special key name.
                let (off, l) =
                    if in_string && *last_dash.add(1) == b'\\' && *last_dash.add(2) == b'"' {
                        // Special case for a double-quoted string
                        (2_usize, 2_i32)
                    } else {
                        (1_usize, utfc_ptr2len(last_dash.add(1).cast()))
                    };

                #[allow(clippy::cast_sign_loss)]
                if modifiers != 0 && *last_dash.add(l as usize + 1) == b'>' {
                    key = nvim_mbyte::rs_utf_ptr2char(last_dash.add(off).cast());
                } else {
                    let mut k = rs_get_special_key_code(last_dash.add(off).cast());
                    if (flags & FSK_KEEP_X_KEY) == 0 {
                        k = rs_handle_x_keys(k);
                    }
                    key = k;
                }
            }

            // get_special_key_code() may return NUL for invalid special key name.
            if key != NUL {
                // Only use a modifier when there is no special key code that
                // includes the modifier.
                let key = rs_simplify_key(key, &raw mut modifiers);

                let key = if (flags & FSK_KEYCODE) == 0 {
                    // don't want keycode, use single byte code
                    if key == K_BS {
                        BS
                    } else if key == K_DEL || key == K_KDEL {
                        DEL
                    } else {
                        key
                    }
                } else {
                    key
                };

                // Normal Key with modifier:
                // Try to make a single byte code (except for Alt/Meta modifiers).
                let key = if is_special(key) {
                    key
                } else {
                    let simplify = (flags & FSK_SIMPLIFY) != 0;
                    let result = rs_extract_modifiers(key, &raw mut modifiers, simplify);
                    if !did_simplify.is_null() && result.did_simplify != 0 {
                        *did_simplify = true;
                    }
                    result.key
                };

                *modp = modifiers;
                *srcp = end_of_name.cast();
                return key;
            }
        }
    }

    0
}

/// Helper to compute pointer difference as `c_int`
#[inline]
unsafe fn ptr_diff(end: *const u8, bp: *const u8) -> c_int {
    #[allow(clippy::cast_possible_truncation)]
    {
        end.offset_from(bp) as c_int
    }
}

/// Try translating a <> name ("keycode").
///
/// This is the main entry point for parsing key notation like `<C-S-Up>`.
///
/// # Arguments
/// * `srcp` - Source from which <> are translated. Is advanced to after the <> name on match.
/// * `src_len` - Length of the source string.
/// * `dst` - Buffer to write the result to. Must be at least 19 bytes.
/// * `flags` - FSK_* flags.
/// * `escape_ks` - Whether to escape `K_SPECIAL` bytes.
/// * `did_simplify` - Output pointer, set to true if simplification occurred.
///
/// # Returns
/// Number of characters added to dst, zero for no match.
///
/// # Safety
/// - `srcp` must be a valid pointer to a valid C string pointer.
/// - `dst` must be a valid pointer with at least 19 bytes of space.
/// - `did_simplify` may be null.
#[export_name = "trans_special"]
pub unsafe extern "C" fn rs_trans_special(
    srcp: *mut *const std::ffi::c_char,
    src_len: usize,
    dst: *mut std::ffi::c_char,
    flags: c_int,
    escape_ks: bool,
    did_simplify: *mut bool,
) -> c_uint {
    if srcp.is_null() || dst.is_null() {
        return 0;
    }

    let mut modifiers: c_int = 0;
    let key = rs_find_special_key(srcp, src_len, &raw mut modifiers, flags, did_simplify);
    if key == 0 {
        return 0;
    }

    rs_special_to_buf(key, modifiers, escape_ks, dst)
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
#[export_name = "special_to_buf"]
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

/// Check if a slice starts with the given pattern (case-insensitive).
fn starts_with_ignore_case(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() {
        return false;
    }
    haystack[..needle.len()]
        .iter()
        .zip(needle.iter())
        .all(|(h, n)| h.eq_ignore_ascii_case(n))
}

/// Replace terminal codes in a string.
///
/// This is the main function for translating keycode notation like `<C-S-Up>`,
/// `<SID>`, `<Leader>`, etc. into their internal representation.
///
/// # Arguments
/// * `from` - Source string containing keycodes to translate.
/// * `from_len` - Length of the source string.
/// * `buf` - Buffer to write the result to. Must be at least `from_len * 6 + 1` bytes.
/// * `sid_arg` - Script ID to use for `<SID>`, or 0 to use `current_sctx`.
/// * `flags` - `REPTERM_*` flags.
/// * `did_simplify` - Output pointer, set to true if simplification occurred.
/// * `do_backslash` - Whether backslash is a special escape character.
/// * `do_special` - Whether to process `<>` notation.
///
/// # Returns
/// Pointer to the result buffer (same as `buf`), or NULL on overflow with fixed buffer.
///
/// # Safety
/// - `from` must be a valid pointer to at least `from_len` bytes.
/// - `buf` must be a valid pointer to at least `from_len * 6 + 1` bytes.
/// - `did_simplify` may be null.
#[allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
unsafe fn rs_replace_termcodes(
    from: *const std::ffi::c_char,
    from_len: usize,
    buf: *mut std::ffi::c_char,
    sid_arg: ScidT,
    flags: c_int,
    did_simplify: *mut bool,
    do_backslash: bool,
    do_special: bool,
) -> *mut std::ffi::c_char {
    if from.is_null() || buf.is_null() {
        return std::ptr::null_mut();
    }

    // Handle empty input - return empty string
    if from_len == 0 {
        *buf = 0;
        return buf;
    }

    let from_u8 = from.cast::<u8>();
    let buf_u8 = buf.cast::<u8>();
    let end = from_u8.add(from_len - 1);

    let mut src = from_u8;
    let mut dlen: usize = 0;

    // Copy each byte from *from to result[dlen]
    while src <= end {
        // Check for special <> keycodes, like "<C-S-LeftMouse>"
        if do_special
            && ((flags & REPTERM_DO_LT) != 0
                || (end.offset_from(src) >= 3
                    && !starts_with_ignore_case(
                        std::slice::from_raw_parts(src, 4.min((end.offset_from(src) + 1) as usize)),
                        b"<lt>",
                    )))
        {
            // Change <SID>Func to K_SNR <script-nr> _Func.  This name is used
            // for script-local user functions.
            // (room: 5 * 6 = 30 bytes; needed: 3 + <nr> + 1 <= 14)
            let remaining = end.offset_from(src) as usize + 1;
            if remaining >= 5
                && starts_with_ignore_case(std::slice::from_raw_parts(src, 5), b"<SID>")
            {
                let current_sid = current_sctx.sc_sid;
                if sid_arg < 0 || (sid_arg == 0 && current_sid <= 0) {
                    emsg(gettext(std::ptr::addr_of!(e_usingsid)));
                } else {
                    let sid = if sid_arg != 0 { sid_arg } else { current_sid };
                    src = src.add(5);
                    *buf_u8.add(dlen) = K_SPECIAL;
                    dlen += 1;
                    *buf_u8.add(dlen) = KS_EXTRA as u8;
                    dlen += 1;
                    *buf_u8.add(dlen) = KE_SNR as u8;
                    dlen += 1;
                    // Write the script ID as a decimal number
                    let sid_str = format!("{sid}");
                    for byte in sid_str.as_bytes() {
                        *buf_u8.add(dlen) = *byte;
                        dlen += 1;
                    }
                    *buf_u8.add(dlen) = b'_';
                    dlen += 1;
                    continue;
                }
            }

            // Try translating standard <> keycodes
            let mut src_ptr = src.cast::<std::ffi::c_char>();
            let src_remaining = (end.offset_from(src) + 1) as usize;
            let trans_flags = FSK_KEYCODE
                | (if (flags & REPTERM_NO_SIMPLIFY) != 0 {
                    0
                } else {
                    FSK_SIMPLIFY
                });
            let slen = rs_trans_special(
                &raw mut src_ptr,
                src_remaining,
                buf.add(dlen),
                trans_flags,
                true,
                did_simplify,
            );
            if slen > 0 {
                dlen += slen as usize;
                src = src_ptr.cast::<u8>();
                continue;
            }
        }

        if do_special {
            // Replace <Leader> by the value of "mapleader".
            // Replace <LocalLeader> by the value of "maplocalleader".
            // If "mapleader" or "maplocalleader" isn't set use a backslash.
            let remaining = end.offset_from(src) as usize + 1;
            let (leader_len, leader_value) = if remaining >= 8
                && starts_with_ignore_case(std::slice::from_raw_parts(src, 8), b"<Leader>")
            {
                (8, rs_get_var_value(c"g:mapleader".as_ptr()).cast_mut())
            } else if remaining >= 13
                && starts_with_ignore_case(std::slice::from_raw_parts(src, 13), b"<LocalLeader>")
            {
                (
                    13,
                    rs_get_var_value(c"g:maplocalleader".as_ptr()).cast_mut(),
                )
            } else {
                (0, std::ptr::null_mut())
            };

            if leader_len != 0 {
                // Allow up to 8 * 6 characters for "mapleader".
                let leader_str = if leader_value.is_null() || *leader_value.cast::<u8>() == 0 || {
                    let mut len = 0;
                    let mut p = leader_value.cast::<u8>();
                    while *p != 0 {
                        len += 1;
                        p = p.add(1);
                    }
                    len > 8 * 6
                } {
                    b"\\" as *const u8
                } else {
                    leader_value.cast::<u8>()
                };

                let mut s = leader_str;
                while *s != 0 {
                    *buf_u8.add(dlen) = *s;
                    dlen += 1;
                    s = s.add(1);
                }
                src = src.add(leader_len);
                continue;
            }
        }

        // Remove CTRL-V and ignore the next character.
        // For "from" side the CTRL-V at the end is included, for the "to"
        // part it is removed.
        // If 'cpoptions' does not contain 'B', also accept a backslash.
        let key = *src;
        if key == CTRL_V || (do_backslash && key == b'\\') {
            src = src.add(1); // skip CTRL-V or backslash
            if src > end {
                if (flags & REPTERM_FROM_PART) != 0 {
                    *buf_u8.add(dlen) = key;
                    dlen += 1;
                }
                break;
            }
        }

        // skip multibyte char correctly
        let char_len = utfc_ptr2len_len(
            src.cast::<std::ffi::c_char>(),
            (end.offset_from(src) + 1) as c_int,
        );
        for _ in 0..char_len {
            // If the character is K_SPECIAL, replace it with K_SPECIAL
            // KS_SPECIAL KE_FILLER.
            if *src == K_SPECIAL {
                *buf_u8.add(dlen) = K_SPECIAL;
                dlen += 1;
                *buf_u8.add(dlen) = KS_SPECIAL;
                dlen += 1;
                *buf_u8.add(dlen) = KE_FILLER as u8;
            } else {
                *buf_u8.add(dlen) = *src;
            }
            dlen += 1;
            src = src.add(1);
        }
    }
    *buf_u8.add(dlen) = 0; // NUL terminate

    buf
}

/// Replace termcodes in `from`, writing the result to `*bufp`.
///
/// This is the exported replacement for the C `replace_termcodes` wrapper.
/// It handles allocation, `CPO_BSLASH` detection, and `REPTERM_NO_SPECIAL` flag.
///
/// If `*bufp` is NULL, a buffer is allocated via `xmalloc` and `*bufp` is set
/// to the (realloc'd) result. If `*bufp` is non-NULL, it is used as-is and
/// is assumed to be 128 bytes (enough for transcoding LHS of a mapping).
///
/// On overflow (fixed buffer too small), `*bufp` is set to NULL and NULL is
/// returned.
///
/// # Safety
/// - `from` must be a valid pointer to at least `from_len` bytes.
/// - `bufp` must be a valid non-null pointer to a `*mut c_char`.
/// - `cpo_val` must be a valid null-terminated C string.
/// - `did_simplify` may be null.
#[unsafe(export_name = "replace_termcodes")]
#[allow(clippy::too_many_arguments, clippy::cast_sign_loss)]
pub unsafe extern "C" fn exported_replace_termcodes(
    from: *const std::ffi::c_char,
    from_len: usize,
    bufp: *mut *mut std::ffi::c_char,
    sid_arg: ScidT,
    flags: c_int,
    did_simplify: *mut bool,
    cpo_val: *const std::ffi::c_char,
) -> *mut std::ffi::c_char {
    // Determine flags from cpo_val and flags
    let do_backslash = vim_strchr(cpo_val, CPO_BSLASH).is_null();
    let do_special = (flags & REPTERM_NO_SPECIAL) == 0;

    let allocated = (*bufp).is_null();

    // Allocate space for the translation. Worst case: 6 bytes per char + NUL.
    let buf_len = if allocated { from_len * 6 + 1 } else { 128 };
    let result: *mut std::ffi::c_char = if allocated {
        nvim_memory::xmalloc(buf_len).cast()
    } else {
        *bufp
    };

    let ret = rs_replace_termcodes(
        from,
        from_len,
        result,
        sid_arg,
        flags,
        did_simplify,
        do_backslash,
        do_special,
    );

    if ret.is_null() {
        // Overflow with fixed buffer
        if allocated {
            nvim_memory::xfree(result.cast());
        }
        *bufp = std::ptr::null_mut();
        return std::ptr::null_mut();
    }

    if allocated {
        let new_len = libc::strlen(result) + 1;
        *bufp = nvim_memory::xrealloc(result.cast(), new_len).cast();
    }

    *bufp
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

    #[test]
    fn test_mod_mask_constants() {
        // Verify modifier mask constants match C definitions
        assert_eq!(MOD_MASK_SHIFT, 0x02);
        assert_eq!(MOD_MASK_CTRL, 0x04);
        assert_eq!(MOD_MASK_ALT, 0x08);
        assert_eq!(MOD_MASK_META, 0x10);
        assert_eq!(MOD_MASK_2CLICK, 0x20);
        assert_eq!(MOD_MASK_3CLICK, 0x40);
        assert_eq!(MOD_MASK_4CLICK, 0x60);
        assert_eq!(MOD_MASK_CMD, 0x80);
    }

    #[test]
    fn test_special_key_constants() {
        // Verify special key constants match C definitions
        assert_eq!(K_SPECIAL, 0x80);
        assert_eq!(KS_MODIFIER, 252);
        assert_eq!(KS_EXTRA, 253);
        assert_eq!(KS_SPECIAL, 254);
    }
}
