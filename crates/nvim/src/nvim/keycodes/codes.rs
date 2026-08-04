//! The key codes themselves: the `K_*` families and the bytes they are built
//! from.
//!
//! A key that is not a character is a negative `c_int`, encoded as two bytes:
//! a `KS_*` "termcap name" byte in the low half and a second byte in the
//! high half, negated. Most keys use `KS_EXTRA` plus a `KE_*` code of their
//! own ([`extra`](super::extra)); the rest carry a two-character termcap
//! name ([`termcap`](super::termcap)). In a byte stream the same key is
//! `K_SPECIAL` followed by those two bytes.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use crate::src::nvim::types::key_extra;

use super::{extra, termcap};

/// C's `IS_SPECIAL`: whether `c` is a key code rather than a character.
/// Every key that is not a printable character travels as a *negative*
/// `c_int`, which is the whole of the test.
pub const fn is_special(c: c_int) -> bool {
    c < 0
}

/// The ASCII characters the `<>` notation has names for.
pub const NUL: c_int = 0x00;
pub const BS: c_int = 0x08;
pub const TAB: c_int = 0x09;
pub const NL: c_int = 0x0a;
pub const CAR: c_int = 0x0d;
pub const ESC: c_int = 0x1b;
pub const DEL: c_int = 0x7f;
pub const CSI: c_int = 0x9b;
/// CTRL-V, which quotes the next character.
pub const CTRL_V: c_int = 0x16;

/// The byte that introduces a special key in a byte stream. A literal 0x80
/// in text is escaped as `K_SPECIAL KS_SPECIAL KE_FILLER`.
pub const K_SPECIAL: c_int = 0x80;
/// Termcap-name bytes that are not termcap names.
pub const KS_SPECIAL: c_int = 254;
pub const KS_EXTRA: c_int = 253;
pub const KS_MODIFIER: c_int = 252;
pub const KS_KEY: c_int = 242;
/// The second byte of a key whose first byte already says everything.
pub const KE_FILLER: c_int = 'X' as c_int;

/// First bytes used by keys that are neither `KS_EXTRA` nor a termcap name.
const KS_ZERO: u8 = 255;
const KS_MOUSE: u8 = 251;
const KS_VER_SCROLLBAR: u8 = 249;
const KS_HOR_SCROLLBAR: u8 = 248;
const KS_SELECT: u8 = 245;

/// The three bytes a key code, a NUL or a literal `K_SPECIAL` is stored as in
/// a byte stream: upstream's `K_SPECIAL`, `K_SECOND(c)`, `K_THIRD(c)`.
///
/// Only these three need escaping; every other byte stands for itself. A NUL
/// cannot be stored literally because the streams are NUL-terminated, and a
/// literal 0x80 cannot because it is the escape byte.
pub fn key_escape(c: c_int) -> [u8; 3] {
    let (second, third) = if c == K_SPECIAL {
        (KS_SPECIAL as u8, KE_FILLER as u8)
    } else if c == 0 {
        (KS_ZERO, KE_FILLER as u8)
    } else {
        let name = super::tables::termcap_name(c);
        (name[0], name[1])
    };
    [K_SPECIAL as u8, second, third]
}

/// The key an escape's two trailing bytes stand for: upstream's `TO_SPECIAL`.
///
/// The inverse of [`key_escape`], except that a NUL comes back as the key
/// code `K_ZERO` rather than as 0 — the streams cannot carry a bare NUL, so
/// callers that want the byte back test for `K_ZERO` themselves.
pub fn key_unescape(second: u8, third: u8) -> c_int {
    if c_int::from(second) == KS_SPECIAL {
        K_SPECIAL
    } else if second == KS_ZERO {
        K_ZERO
    } else {
        super::tables::termcap_key([second, third])
    }
}

/// The second byte of a `KS_EXTRA` key. Upstream's `enum key_extra`.
pub const KE_S_UP: key_extra = 4;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_F1: key_extra = 6;
pub const KE_S_F2: key_extra = 7;
pub const KE_S_F3: key_extra = 8;
pub const KE_S_F4: key_extra = 9;
pub const KE_S_F5: key_extra = 10;
pub const KE_S_F6: key_extra = 11;
pub const KE_S_F7: key_extra = 12;
pub const KE_S_F8: key_extra = 13;
pub const KE_S_F9: key_extra = 14;
pub const KE_S_F10: key_extra = 15;
pub const KE_S_F11: key_extra = 16;
pub const KE_S_F12: key_extra = 17;
pub const KE_S_F13: key_extra = 18;
pub const KE_S_F14: key_extra = 19;
pub const KE_S_F15: key_extra = 20;
pub const KE_S_F16: key_extra = 21;
pub const KE_S_F17: key_extra = 22;
pub const KE_S_F18: key_extra = 23;
pub const KE_S_F19: key_extra = 24;
pub const KE_S_F20: key_extra = 25;
pub const KE_S_F21: key_extra = 26;
pub const KE_S_F22: key_extra = 27;
pub const KE_S_F23: key_extra = 28;
pub const KE_S_F24: key_extra = 29;
pub const KE_S_F25: key_extra = 30;
pub const KE_S_F26: key_extra = 31;
pub const KE_S_F27: key_extra = 32;
pub const KE_S_F28: key_extra = 33;
pub const KE_S_F29: key_extra = 34;
pub const KE_S_F30: key_extra = 35;
pub const KE_S_F31: key_extra = 36;
pub const KE_S_F32: key_extra = 37;
pub const KE_S_F33: key_extra = 38;
pub const KE_S_F34: key_extra = 39;
pub const KE_S_F35: key_extra = 40;
pub const KE_S_F36: key_extra = 41;
pub const KE_S_F37: key_extra = 42;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_IGNORE: key_extra = 53;
pub const KE_TAB: key_extra = 54;
pub const KE_XF1: key_extra = 57;
pub const KE_XF2: key_extra = 58;
pub const KE_XF3: key_extra = 59;
pub const KE_XF4: key_extra = 60;
pub const KE_XEND: key_extra = 61;
pub const KE_ZEND: key_extra = 62;
pub const KE_XHOME: key_extra = 63;
pub const KE_ZHOME: key_extra = 64;
pub const KE_XUP: key_extra = 65;
pub const KE_XDOWN: key_extra = 66;
pub const KE_XLEFT: key_extra = 67;
pub const KE_XRIGHT: key_extra = 68;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_S_XF1: key_extra = 71;
pub const KE_S_XF2: key_extra = 72;
pub const KE_S_XF3: key_extra = 73;
pub const KE_S_XF4: key_extra = 74;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_KINS: key_extra = 79;
pub const KE_KDEL: key_extra = 80;
pub const KE_SNR: key_extra = 82;
pub const KE_PLUG: key_extra = 83;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_END: key_extra = 88;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_DROP: key_extra = 95;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_NOP: key_extra = 97;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_EVENT: key_extra = 102;
pub const KE_LUA: key_extra = 103;
pub const KE_COMMAND: key_extra = 104;
pub const KE_WILD: key_extra = 108;

/// Key codes for the "extra" keys: everything with no termcap name of its
/// own, which is most of what a modern terminal sends.
pub const K_COMMAND: c_int = extra(KE_COMMAND);
pub const K_CMDWIN: c_int = extra(KE_CMDWIN);
pub const K_C_END: c_int = extra(KE_C_END);
pub const K_C_HOME: c_int = extra(KE_C_HOME);
pub const K_C_LEFT: c_int = extra(KE_C_LEFT);
pub const K_C_RIGHT: c_int = extra(KE_C_RIGHT);
pub const K_EVENT: c_int = extra(KE_EVENT);
pub const K_IGNORE: c_int = extra(KE_IGNORE);
pub const K_KDEL: c_int = extra(KE_KDEL);
pub const K_KINS: c_int = extra(KE_KINS);
pub const K_LEFTDRAG: c_int = extra(KE_LEFTDRAG);
pub const K_LEFTMOUSE: c_int = extra(KE_LEFTMOUSE);
pub const K_LEFTMOUSE_NM: c_int = extra(KE_LEFTMOUSE_NM);
pub const K_LEFTRELEASE: c_int = extra(KE_LEFTRELEASE);
pub const K_LEFTRELEASE_NM: c_int = extra(KE_LEFTRELEASE_NM);
pub const K_LUA: c_int = extra(KE_LUA);
pub const K_MIDDLEDRAG: c_int = extra(KE_MIDDLEDRAG);
pub const K_MIDDLEMOUSE: c_int = extra(KE_MIDDLEMOUSE);
pub const K_MIDDLERELEASE: c_int = extra(KE_MIDDLERELEASE);
pub const K_MOUSEDOWN: c_int = extra(KE_MOUSEDOWN);
pub const K_MOUSELEFT: c_int = extra(KE_MOUSELEFT);
pub const K_MOUSEMOVE: c_int = extra(KE_MOUSEMOVE);
pub const K_MOUSERIGHT: c_int = extra(KE_MOUSERIGHT);
pub const K_MOUSEUP: c_int = extra(KE_MOUSEUP);
pub const K_NOP: c_int = extra(KE_NOP);
pub const K_RIGHTDRAG: c_int = extra(KE_RIGHTDRAG);
pub const K_RIGHTMOUSE: c_int = extra(KE_RIGHTMOUSE);
pub const K_RIGHTRELEASE: c_int = extra(KE_RIGHTRELEASE);
pub const K_PLUG: c_int = extra(KE_PLUG);
pub const K_SNR: c_int = extra(KE_SNR);
pub const K_S_DOWN: c_int = extra(KE_S_DOWN);
pub const K_S_F1: c_int = extra(KE_S_F1);
pub const K_S_F2: c_int = extra(KE_S_F2);
pub const K_S_F3: c_int = extra(KE_S_F3);
pub const K_S_F4: c_int = extra(KE_S_F4);
pub const K_S_F5: c_int = extra(KE_S_F5);
pub const K_S_F6: c_int = extra(KE_S_F6);
pub const K_S_F7: c_int = extra(KE_S_F7);
pub const K_S_F8: c_int = extra(KE_S_F8);
pub const K_S_F9: c_int = extra(KE_S_F9);
pub const K_S_F10: c_int = extra(KE_S_F10);
pub const K_S_F11: c_int = extra(KE_S_F11);
pub const K_S_F12: c_int = extra(KE_S_F12);
pub const K_S_UP: c_int = extra(KE_S_UP);
pub const K_S_XF1: c_int = extra(KE_S_XF1);
pub const K_S_XF2: c_int = extra(KE_S_XF2);
pub const K_S_XF3: c_int = extra(KE_S_XF3);
pub const K_S_XF4: c_int = extra(KE_S_XF4);
pub const K_X1DRAG: c_int = extra(KE_X1DRAG);
pub const K_X1MOUSE: c_int = extra(KE_X1MOUSE);
pub const K_X1RELEASE: c_int = extra(KE_X1RELEASE);
pub const K_X2DRAG: c_int = extra(KE_X2DRAG);
pub const K_X2MOUSE: c_int = extra(KE_X2MOUSE);
pub const K_X2RELEASE: c_int = extra(KE_X2RELEASE);
pub const K_XDOWN: c_int = extra(KE_XDOWN);
pub const K_XEND: c_int = extra(KE_XEND);
pub const K_XF1: c_int = extra(KE_XF1);
pub const K_XF2: c_int = extra(KE_XF2);
pub const K_XF3: c_int = extra(KE_XF3);
pub const K_XF4: c_int = extra(KE_XF4);
pub const K_XHOME: c_int = extra(KE_XHOME);
pub const K_XLEFT: c_int = extra(KE_XLEFT);
pub const K_XRIGHT: c_int = extra(KE_XRIGHT);
pub const K_XUP: c_int = extra(KE_XUP);
pub const K_WILD: c_int = extra(KE_WILD);
pub const K_ZEND: c_int = extra(KE_ZEND);
pub const K_ZHOME: c_int = extra(KE_ZHOME);

/// Key codes named by a two-character termcap entry.
pub const K_BS: c_int = termcap(b'k', b'b');
pub const K_DEL: c_int = termcap(b'k', b'D');
pub const K_DOWN: c_int = termcap(b'k', b'd');
pub const K_END: c_int = termcap(b'@', b'7');
pub const K_F1: c_int = termcap(b'k', b'1');
pub const K_F2: c_int = termcap(b'k', b'2');
pub const K_F3: c_int = termcap(b'k', b'3');
pub const K_F4: c_int = termcap(b'k', b'4');
pub const K_F5: c_int = termcap(b'k', b'5');
pub const K_F6: c_int = termcap(b'k', b'6');
pub const K_F7: c_int = termcap(b'k', b'7');
pub const K_F8: c_int = termcap(b'k', b'8');
pub const K_F9: c_int = termcap(b'k', b'9');
pub const K_F10: c_int = termcap(b'k', b';');
pub const K_F11: c_int = termcap(b'F', b'1');
pub const K_F12: c_int = termcap(b'F', b'2');
pub const K_F13: c_int = termcap(b'F', b'3');
pub const K_F14: c_int = termcap(b'F', b'4');
pub const K_F15: c_int = termcap(b'F', b'5');
pub const K_F16: c_int = termcap(b'F', b'6');
pub const K_F17: c_int = termcap(b'F', b'7');
pub const K_F18: c_int = termcap(b'F', b'8');
pub const K_F19: c_int = termcap(b'F', b'9');
pub const K_F20: c_int = termcap(b'F', b'A');
pub const K_F21: c_int = termcap(b'F', b'B');
pub const K_F22: c_int = termcap(b'F', b'C');
pub const K_F23: c_int = termcap(b'F', b'D');
pub const K_F24: c_int = termcap(b'F', b'E');
pub const K_F25: c_int = termcap(b'F', b'F');
pub const K_F26: c_int = termcap(b'F', b'G');
pub const K_F27: c_int = termcap(b'F', b'H');
pub const K_F28: c_int = termcap(b'F', b'I');
pub const K_F29: c_int = termcap(b'F', b'J');
pub const K_F30: c_int = termcap(b'F', b'K');
pub const K_F31: c_int = termcap(b'F', b'L');
pub const K_F32: c_int = termcap(b'F', b'M');
pub const K_F33: c_int = termcap(b'F', b'N');
pub const K_F34: c_int = termcap(b'F', b'O');
pub const K_F35: c_int = termcap(b'F', b'P');
pub const K_F36: c_int = termcap(b'F', b'Q');
pub const K_F37: c_int = termcap(b'F', b'R');
pub const K_F38: c_int = termcap(b'F', b'S');
pub const K_F39: c_int = termcap(b'F', b'T');
pub const K_F40: c_int = termcap(b'F', b'U');
pub const K_F41: c_int = termcap(b'F', b'V');
pub const K_F42: c_int = termcap(b'F', b'W');
pub const K_F43: c_int = termcap(b'F', b'X');
pub const K_F44: c_int = termcap(b'F', b'Y');
pub const K_F45: c_int = termcap(b'F', b'Z');
pub const K_F46: c_int = termcap(b'F', b'a');
pub const K_F47: c_int = termcap(b'F', b'b');
pub const K_F48: c_int = termcap(b'F', b'c');
pub const K_F49: c_int = termcap(b'F', b'd');
pub const K_F50: c_int = termcap(b'F', b'e');
pub const K_F51: c_int = termcap(b'F', b'f');
pub const K_F52: c_int = termcap(b'F', b'g');
pub const K_F53: c_int = termcap(b'F', b'h');
pub const K_F54: c_int = termcap(b'F', b'i');
pub const K_F55: c_int = termcap(b'F', b'j');
pub const K_F56: c_int = termcap(b'F', b'k');
pub const K_F57: c_int = termcap(b'F', b'l');
pub const K_F58: c_int = termcap(b'F', b'm');
pub const K_F59: c_int = termcap(b'F', b'n');
pub const K_F60: c_int = termcap(b'F', b'o');
pub const K_F61: c_int = termcap(b'F', b'p');
pub const K_F62: c_int = termcap(b'F', b'q');
pub const K_F63: c_int = termcap(b'F', b'r');
pub const K_FIND: c_int = termcap(b'@', b'0');
pub const K_HELP: c_int = termcap(b'%', b'1');
pub const K_HOME: c_int = termcap(b'k', b'h');
pub const K_HOR_SCROLLBAR: c_int = termcap(KS_HOR_SCROLLBAR, KE_FILLER as u8);
pub const K_INS: c_int = termcap(b'k', b'I');
pub const K_K0: c_int = termcap(b'K', b'C');
pub const K_K1: c_int = termcap(b'K', b'D');
pub const K_K2: c_int = termcap(b'K', b'E');
pub const K_K3: c_int = termcap(b'K', b'F');
pub const K_K4: c_int = termcap(b'K', b'G');
pub const K_K5: c_int = termcap(b'K', b'H');
pub const K_K6: c_int = termcap(b'K', b'I');
pub const K_K7: c_int = termcap(b'K', b'J');
pub const K_K8: c_int = termcap(b'K', b'K');
pub const K_K9: c_int = termcap(b'K', b'L');
pub const K_KCOMMA: c_int = termcap(b'K', b'M');
pub const K_KDIVIDE: c_int = termcap(b'K', b'8');
pub const K_KDOWN: c_int = termcap(b'K', b'd');
pub const K_KEND: c_int = termcap(b'K', b'4');
pub const K_KENTER: c_int = termcap(b'K', b'A');
pub const K_KEQUAL: c_int = termcap(b'K', b'N');
pub const K_KHOME: c_int = termcap(b'K', b'1');
pub const K_KLEFT: c_int = termcap(b'K', b'l');
pub const K_KMINUS: c_int = termcap(b'K', b'7');
pub const K_KMULTIPLY: c_int = termcap(b'K', b'9');
pub const K_KORIGIN: c_int = termcap(b'K', b'2');
pub const K_KPAGEDOWN: c_int = termcap(b'K', b'5');
pub const K_KPAGEUP: c_int = termcap(b'K', b'3');
pub const K_KPLUS: c_int = termcap(b'K', b'6');
pub const K_KPOINT: c_int = termcap(b'K', b'B');
pub const K_KRIGHT: c_int = termcap(b'K', b'r');
pub const K_KSELECT: c_int = termcap(b'*', b'6');
pub const K_KUP: c_int = termcap(b'K', b'u');
pub const K_LEFT: c_int = termcap(b'k', b'l');
pub const K_MOUSE: c_int = termcap(KS_MOUSE, KE_FILLER as u8);
pub const K_PAGEDOWN: c_int = termcap(b'k', b'N');
pub const K_PAGEUP: c_int = termcap(b'k', b'P');
pub const K_PASTE_END: c_int = termcap(b'P', b'E');
pub const K_PASTE_START: c_int = termcap(b'P', b'S');
pub const K_RIGHT: c_int = termcap(b'k', b'r');
pub const K_SELECT: c_int = termcap(KS_SELECT, KE_FILLER as u8);
pub const K_S_END: c_int = termcap(b'*', b'7');
pub const K_S_HOME: c_int = termcap(b'#', b'2');
pub const K_S_LEFT: c_int = termcap(b'#', b'4');
pub const K_S_RIGHT: c_int = termcap(b'%', b'i');
pub const K_S_TAB: c_int = termcap(b'k', b'B');
pub const K_UNDO: c_int = termcap(b'&', b'8');
pub const K_UP: c_int = termcap(b'k', b'u');
pub const K_VER_SCROLLBAR: c_int = termcap(KS_VER_SCROLLBAR, KE_FILLER as u8);
pub const K_ZERO: c_int = termcap(KS_ZERO, KE_FILLER as u8);

/// Modifier bits, as they travel in a `K_SPECIAL KS_MODIFIER <bits>` sequence
/// and in `mod_mask`.
pub const MOD_MASK_SHIFT: c_int = 0x02;
pub const MOD_MASK_CTRL: c_int = 0x04;
pub const MOD_MASK_ALT: c_int = 0x08;
pub const MOD_MASK_META: c_int = 0x10;
pub const MOD_MASK_2CLICK: c_int = 0x20;
pub const MOD_MASK_3CLICK: c_int = 0x40;
pub const MOD_MASK_4CLICK: c_int = 0x60;
pub const MOD_MASK_CMD: c_int = 0x80;
/// The two bits the click-count values share.
pub const MOD_MASK_MULTI_CLICK: c_int = MOD_MASK_2CLICK | MOD_MASK_3CLICK | MOD_MASK_4CLICK;

/// Which mouse button an event is about.
pub const MOUSE_LEFT: c_int = 0x00;
pub const MOUSE_MIDDLE: c_int = 0x01;
pub const MOUSE_RIGHT: c_int = 0x02;
pub const MOUSE_X1: c_int = 0x300;
pub const MOUSE_X2: c_int = 0x400;
/// Not a button: a release or a move with no button down.
pub const MOUSE_RELEASE: c_int = 0x03;

/// Flags for [`find_special_key`](super::find_special_key) and
/// [`trans_special`](super::trans_special).
///
/// * `FSK_KEYCODE` — answer with a key code rather than a single byte, so
///   `<BS>` stays `K_BS` instead of collapsing to 0x08.
/// * `FSK_KEEP_X_KEY` — leave `<xUp>` as `K_XUP` instead of folding it to
///   `K_UP`.
/// * `FSK_IN_STRING` — the source is a double-quoted string, where a bare `"`
///   ends the string and `\"` is the way to name it.
/// * `FSK_SIMPLIFY` — fold `<C-H>` to 0x08 and friends.
pub const FSK_KEYCODE: c_int = 1;
pub const FSK_KEEP_X_KEY: c_int = 2;
pub const FSK_IN_STRING: c_int = 4;
pub const FSK_SIMPLIFY: c_int = 8;

/// Flags for [`replace_termcodes`](super::replace_termcodes).
///
/// * `REPTERM_FROM_PART` — this is the lhs of a mapping, so a trailing
///   CTRL-V is kept rather than dropped.
/// * `REPTERM_DO_LT` — translate `<lt>` as well; without it a literal
///   `<lt>` is passed through.
/// * `REPTERM_NO_SPECIAL` — do not accept `<key>` notation at all.
/// * `REPTERM_NO_SIMPLIFY` — keep `<C-H>` as a key code.
pub const REPTERM_FROM_PART: c_int = 1;
pub const REPTERM_DO_LT: c_int = 2;
pub const REPTERM_NO_SPECIAL: c_int = 4;
pub const REPTERM_NO_SIMPLIFY: c_int = 8;

/// The longest `<...>` a key can print as, `<` and `>` included. Adding a
/// longer key name, or another modifier letter, means raising this.
pub const MAX_KEY_NAME_LEN: c_int = 32;
