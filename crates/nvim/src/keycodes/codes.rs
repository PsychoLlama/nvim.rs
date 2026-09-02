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

use crate::types::key_extra;

use super::{extra, termcap};

/// C's `IS_SPECIAL`: whether `c` is a key code rather than a character.
/// Every key that is not a printable character travels as a *negative*
/// `c_int`, which is the whole of the test.
pub const fn is_special(c: c_int) -> bool {
    c < 0
}

/// The ASCII characters the `<>` notation has names for.
pub const CSI: c_int = 0x9b;

/// The C0 control characters, named after the key that types them
/// (upstream's `ascii_defs.h`). Several have a second name above: `Ctrl_H`
/// is `BS`, `Ctrl_I` is `TAB`, `Ctrl_J` is `NL`, `Ctrl_M` is `CAR`, and
/// CTRL-`[` is `ESC` and has no `Ctrl_` spelling at all. `Ctrl_V` is the
/// one that quotes the next character. The four non-letters are named after
/// the character rather than the key: `@`, `\`, `]` and `^`.
pub const Ctrl_AT: c_int = 0;
pub const Ctrl_A: c_int = 1;
pub const Ctrl_B: c_int = 2;
pub const Ctrl_C: c_int = 3;
pub const Ctrl_D: c_int = 4;
pub const Ctrl_E: c_int = 5;
pub const Ctrl_F: c_int = 6;
pub const Ctrl_G: c_int = 7;
pub const Ctrl_H: c_int = 8;
pub const Ctrl_I: c_int = 9;
pub const Ctrl_J: c_int = 10;
pub const Ctrl_K: c_int = 11;
pub const Ctrl_L: c_int = 12;
pub const Ctrl_M: c_int = 13;
pub const Ctrl_N: c_int = 14;
pub const Ctrl_O: c_int = 15;
pub const Ctrl_P: c_int = 16;
pub const Ctrl_Q: c_int = 17;
pub const Ctrl_R: c_int = 18;
pub const Ctrl_S: c_int = 19;
pub const Ctrl_T: c_int = 20;
pub const Ctrl_U: c_int = 21;
pub const Ctrl_V: c_int = 22;
pub const Ctrl_W: c_int = 23;
pub const Ctrl_X: c_int = 24;
pub const Ctrl_Y: c_int = 25;
pub const Ctrl_Z: c_int = 26;
pub const Ctrl_BSL: c_int = 28;
pub const Ctrl_RSB: c_int = 29;
pub const Ctrl_HAT: c_int = 30;
pub const Ctrl__: c_int = 31;

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
        Key::Zero.code()
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

/// A key that is not a character: everything the `<>` notation has a name
/// for and no code point stands for.
///
/// The discriminants are the key codes themselves, written the way
/// upstream writes them -- a `KS_EXTRA` byte plus a [`key_extra`] code, or a
/// two-character termcap name -- so the enum *is* the definition and not a
/// second copy of it. They are all negative, which is what [`is_special`]
/// tests and what lets a key and a character share one `c_int`: `vgetc` and
/// the typeahead carry either, and [`Key::try_from`] is the seam between
/// them. Raw bytes stay bytes; only the *named* codes are in here.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum Key {
    /// The "extra" keys: everything with no termcap name of its own, which
    /// is most of what a modern terminal sends.
    Command = extra(KE_COMMAND),
    Cmdwin = extra(KE_CMDWIN),
    CEnd = extra(KE_C_END),
    CHome = extra(KE_C_HOME),
    CLeft = extra(KE_C_LEFT),
    CRight = extra(KE_C_RIGHT),
    Event = extra(KE_EVENT),
    Ignore = extra(KE_IGNORE),
    Kdel = extra(KE_KDEL),
    Kins = extra(KE_KINS),
    Leftdrag = extra(KE_LEFTDRAG),
    Leftmouse = extra(KE_LEFTMOUSE),
    LeftmouseNm = extra(KE_LEFTMOUSE_NM),
    Leftrelease = extra(KE_LEFTRELEASE),
    LeftreleaseNm = extra(KE_LEFTRELEASE_NM),
    Lua = extra(KE_LUA),
    Middledrag = extra(KE_MIDDLEDRAG),
    Middlemouse = extra(KE_MIDDLEMOUSE),
    Middlerelease = extra(KE_MIDDLERELEASE),
    Mousedown = extra(KE_MOUSEDOWN),
    Mouseleft = extra(KE_MOUSELEFT),
    Mousemove = extra(KE_MOUSEMOVE),
    Mouseright = extra(KE_MOUSERIGHT),
    Mouseup = extra(KE_MOUSEUP),
    Nop = extra(KE_NOP),
    Rightdrag = extra(KE_RIGHTDRAG),
    Rightmouse = extra(KE_RIGHTMOUSE),
    Rightrelease = extra(KE_RIGHTRELEASE),
    Plug = extra(KE_PLUG),
    Snr = extra(KE_SNR),
    SDown = extra(KE_S_DOWN),
    SF1 = extra(KE_S_F1),
    SF2 = extra(KE_S_F2),
    SF3 = extra(KE_S_F3),
    SF4 = extra(KE_S_F4),
    SF5 = extra(KE_S_F5),
    SF6 = extra(KE_S_F6),
    SF7 = extra(KE_S_F7),
    SF8 = extra(KE_S_F8),
    SF9 = extra(KE_S_F9),
    SF10 = extra(KE_S_F10),
    SF11 = extra(KE_S_F11),
    SF12 = extra(KE_S_F12),
    SUp = extra(KE_S_UP),
    SXf1 = extra(KE_S_XF1),
    SXf2 = extra(KE_S_XF2),
    SXf3 = extra(KE_S_XF3),
    SXf4 = extra(KE_S_XF4),
    X1drag = extra(KE_X1DRAG),
    X1mouse = extra(KE_X1MOUSE),
    X1release = extra(KE_X1RELEASE),
    X2drag = extra(KE_X2DRAG),
    X2mouse = extra(KE_X2MOUSE),
    X2release = extra(KE_X2RELEASE),
    Xdown = extra(KE_XDOWN),
    Xend = extra(KE_XEND),
    Xf1 = extra(KE_XF1),
    Xf2 = extra(KE_XF2),
    Xf3 = extra(KE_XF3),
    Xf4 = extra(KE_XF4),
    Xhome = extra(KE_XHOME),
    Xleft = extra(KE_XLEFT),
    Xright = extra(KE_XRIGHT),
    Xup = extra(KE_XUP),
    Wild = extra(KE_WILD),
    Zend = extra(KE_ZEND),
    Zhome = extra(KE_ZHOME),

    /// The keys named by a two-character termcap entry.
    Bs = termcap(b'k', b'b'),
    Del = termcap(b'k', b'D'),
    Down = termcap(b'k', b'd'),
    End = termcap(b'@', b'7'),
    F1 = termcap(b'k', b'1'),
    F2 = termcap(b'k', b'2'),
    F3 = termcap(b'k', b'3'),
    F4 = termcap(b'k', b'4'),
    F5 = termcap(b'k', b'5'),
    F6 = termcap(b'k', b'6'),
    F7 = termcap(b'k', b'7'),
    F8 = termcap(b'k', b'8'),
    F9 = termcap(b'k', b'9'),
    F10 = termcap(b'k', b';'),
    F11 = termcap(b'F', b'1'),
    F12 = termcap(b'F', b'2'),
    F13 = termcap(b'F', b'3'),
    F14 = termcap(b'F', b'4'),
    F15 = termcap(b'F', b'5'),
    F16 = termcap(b'F', b'6'),
    F17 = termcap(b'F', b'7'),
    F18 = termcap(b'F', b'8'),
    F19 = termcap(b'F', b'9'),
    F20 = termcap(b'F', b'A'),
    F21 = termcap(b'F', b'B'),
    F22 = termcap(b'F', b'C'),
    F23 = termcap(b'F', b'D'),
    F24 = termcap(b'F', b'E'),
    F25 = termcap(b'F', b'F'),
    F26 = termcap(b'F', b'G'),
    F27 = termcap(b'F', b'H'),
    F28 = termcap(b'F', b'I'),
    F29 = termcap(b'F', b'J'),
    F30 = termcap(b'F', b'K'),
    F31 = termcap(b'F', b'L'),
    F32 = termcap(b'F', b'M'),
    F33 = termcap(b'F', b'N'),
    F34 = termcap(b'F', b'O'),
    F35 = termcap(b'F', b'P'),
    F36 = termcap(b'F', b'Q'),
    F37 = termcap(b'F', b'R'),
    F38 = termcap(b'F', b'S'),
    F39 = termcap(b'F', b'T'),
    F40 = termcap(b'F', b'U'),
    F41 = termcap(b'F', b'V'),
    F42 = termcap(b'F', b'W'),
    F43 = termcap(b'F', b'X'),
    F44 = termcap(b'F', b'Y'),
    F45 = termcap(b'F', b'Z'),
    F46 = termcap(b'F', b'a'),
    F47 = termcap(b'F', b'b'),
    F48 = termcap(b'F', b'c'),
    F49 = termcap(b'F', b'd'),
    F50 = termcap(b'F', b'e'),
    F51 = termcap(b'F', b'f'),
    F52 = termcap(b'F', b'g'),
    F53 = termcap(b'F', b'h'),
    F54 = termcap(b'F', b'i'),
    F55 = termcap(b'F', b'j'),
    F56 = termcap(b'F', b'k'),
    F57 = termcap(b'F', b'l'),
    F58 = termcap(b'F', b'm'),
    F59 = termcap(b'F', b'n'),
    F60 = termcap(b'F', b'o'),
    F61 = termcap(b'F', b'p'),
    F62 = termcap(b'F', b'q'),
    F63 = termcap(b'F', b'r'),
    Find = termcap(b'@', b'0'),
    Help = termcap(b'%', b'1'),
    Home = termcap(b'k', b'h'),
    HorScrollbar = termcap(KS_HOR_SCROLLBAR, KE_FILLER as u8),
    Ins = termcap(b'k', b'I'),
    K0 = termcap(b'K', b'C'),
    K1 = termcap(b'K', b'D'),
    K2 = termcap(b'K', b'E'),
    K3 = termcap(b'K', b'F'),
    K4 = termcap(b'K', b'G'),
    K5 = termcap(b'K', b'H'),
    K6 = termcap(b'K', b'I'),
    K7 = termcap(b'K', b'J'),
    K8 = termcap(b'K', b'K'),
    K9 = termcap(b'K', b'L'),
    Kcomma = termcap(b'K', b'M'),
    Kdivide = termcap(b'K', b'8'),
    Kdown = termcap(b'K', b'd'),
    Kend = termcap(b'K', b'4'),
    Kenter = termcap(b'K', b'A'),
    Kequal = termcap(b'K', b'N'),
    Khome = termcap(b'K', b'1'),
    Kleft = termcap(b'K', b'l'),
    Kminus = termcap(b'K', b'7'),
    Kmultiply = termcap(b'K', b'9'),
    Korigin = termcap(b'K', b'2'),
    Kpagedown = termcap(b'K', b'5'),
    Kpageup = termcap(b'K', b'3'),
    Kplus = termcap(b'K', b'6'),
    Kpoint = termcap(b'K', b'B'),
    Kright = termcap(b'K', b'r'),
    Kselect = termcap(b'*', b'6'),
    Kup = termcap(b'K', b'u'),
    Left = termcap(b'k', b'l'),
    Mouse = termcap(KS_MOUSE, KE_FILLER as u8),
    Pagedown = termcap(b'k', b'N'),
    Pageup = termcap(b'k', b'P'),
    PasteEnd = termcap(b'P', b'E'),
    PasteStart = termcap(b'P', b'S'),
    Right = termcap(b'k', b'r'),
    Select = termcap(KS_SELECT, KE_FILLER as u8),
    SEnd = termcap(b'*', b'7'),
    SHome = termcap(b'#', b'2'),
    SLeft = termcap(b'#', b'4'),
    SRight = termcap(b'%', b'i'),
    STab = termcap(b'k', b'B'),
    Undo = termcap(b'&', b'8'),
    Up = termcap(b'k', b'u'),
    VerScrollbar = termcap(KS_VER_SCROLLBAR, KE_FILLER as u8),
    Zero = termcap(KS_ZERO, KE_FILLER as u8),
}

impl Key {
    /// Every key, in *code* order, which is why the list reads as though it
    /// were shuffled: the codes run from `Up`'s termcap name down to the
    /// highest `KE_*` byte, and nothing else about the order means anything.
    /// It is the table [`Key::try_from`] binary searches, and the `const`
    /// block below fails the build if it ever stops being sorted.
    const ALL: [Key; 184] = [
        Key::Up,
        Key::Kup,
        Key::Right,
        Key::Kright,
        Key::F63,
        Key::F62,
        Key::F61,
        Key::F60,
        Key::F59,
        Key::F58,
        Key::Wild,
        Key::Left,
        Key::Kleft,
        Key::F57,
        Key::F56,
        Key::F55,
        Key::F54,
        Key::SRight,
        Key::Command,
        Key::Home,
        Key::F53,
        Key::Lua,
        Key::F52,
        Key::Event,
        Key::F51,
        Key::F50,
        Key::Mousemove,
        Key::Down,
        Key::Kdown,
        Key::F49,
        Key::F48,
        Key::Bs,
        Key::F47,
        Key::Nop,
        Key::F46,
        Key::X2release,
        Key::X2drag,
        Key::X2mouse,
        Key::X1release,
        Key::X1drag,
        Key::F45,
        Key::X1mouse,
        Key::F44,
        Key::Zero,
        Key::CEnd,
        Key::Mouse,
        Key::VerScrollbar,
        Key::HorScrollbar,
        Key::Select,
        Key::F43,
        Key::CHome,
        Key::F42,
        Key::CRight,
        Key::F41,
        Key::CLeft,
        Key::F40,
        Key::Cmdwin,
        Key::F39,
        Key::Plug,
        Key::PasteStart,
        Key::F38,
        Key::Snr,
        Key::F37,
        Key::F36,
        Key::Kdel,
        Key::Pageup,
        Key::F35,
        Key::Kins,
        Key::F34,
        Key::Mouseright,
        Key::Pagedown,
        Key::Kequal,
        Key::F33,
        Key::Mouseleft,
        Key::Kcomma,
        Key::F32,
        Key::Mouseup,
        Key::K9,
        Key::F31,
        Key::Mousedown,
        Key::K8,
        Key::F30,
        Key::SXf4,
        Key::K7,
        Key::F29,
        Key::SXf3,
        Key::Ins,
        Key::K6,
        Key::F28,
        Key::SXf2,
        Key::K5,
        Key::F27,
        Key::SXf1,
        Key::K4,
        Key::F26,
        Key::LeftreleaseNm,
        Key::K3,
        Key::F25,
        Key::LeftmouseNm,
        Key::PasteEnd,
        Key::K2,
        Key::F24,
        Key::Xright,
        Key::Del,
        Key::K1,
        Key::F23,
        Key::Xleft,
        Key::K0,
        Key::F22,
        Key::Xdown,
        Key::STab,
        Key::Kpoint,
        Key::F21,
        Key::Xup,
        Key::Kenter,
        Key::F20,
        Key::Zhome,
        Key::Xhome,
        Key::Zend,
        Key::Xend,
        Key::Xf4,
        Key::Xf3,
        Key::F10,
        Key::Xf2,
        Key::Xf1,
        Key::F9,
        Key::Kmultiply,
        Key::F19,
        Key::F8,
        Key::Kdivide,
        Key::F18,
        Key::Undo,
        Key::F7,
        Key::Kminus,
        Key::F17,
        Key::End,
        Key::SEnd,
        Key::F6,
        Key::Kplus,
        Key::F16,
        Key::Kselect,
        Key::Ignore,
        Key::F5,
        Key::Kpagedown,
        Key::F15,
        Key::Rightrelease,
        Key::F4,
        Key::Kend,
        Key::F14,
        Key::SLeft,
        Key::Rightdrag,
        Key::F3,
        Key::Kpageup,
        Key::F13,
        Key::Rightmouse,
        Key::F2,
        Key::Korigin,
        Key::F12,
        Key::SHome,
        Key::Middlerelease,
        Key::F1,
        Key::Khome,
        Key::F11,
        Key::Help,
        Key::Middledrag,
        Key::Find,
        Key::Middlemouse,
        Key::Leftrelease,
        Key::Leftdrag,
        Key::Leftmouse,
        Key::SF12,
        Key::SF11,
        Key::SF10,
        Key::SF9,
        Key::SF8,
        Key::SF7,
        Key::SF6,
        Key::SF5,
        Key::SF4,
        Key::SF3,
        Key::SF2,
        Key::SF1,
        Key::SDown,
        Key::SUp,
    ];

    /// The key code itself, for the many places that still carry one as a
    /// `c_int` beside the characters.
    pub const fn code(self) -> c_int {
        self as c_int
    }
}

/// [`Key::ALL`] has to stay sorted for the binary search below, and the
/// assertion is here rather than in a test because a `const` can check it.
const _: () = {
    let mut i = 1;
    while i < Key::ALL.len() {
        assert!(
            Key::ALL[i - 1].code() < Key::ALL[i].code(),
            "Key::ALL is out of order"
        );
        i += 1;
    }
};

/// The number is not one of the named keys -- most often because it is a
/// character, which shares the space.
///
/// It carries the number so that a dispatch over both halves can name a
/// character in the same `match`: `Err(NotAKey(Ctrl_B))` is the arm for a
/// literal CTRL-B beside `Ok(Key::Home)` for the key.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NotAKey(pub c_int);

impl TryFrom<c_int> for Key {
    type Error = NotAKey;

    fn try_from(code: c_int) -> Result<Key, NotAKey> {
        match Key::ALL.binary_search_by_key(&code, |key| key.code()) {
            Ok(i) => Ok(Key::ALL[i]),
            Err(_) => Err(NotAKey(code)),
        }
    }
}

crate::flag_set! {
    /// Modifier bits, as they travel in a `K_SPECIAL KS_MODIFIER <bits>`
    /// sequence and in `mod_mask`.
    ///
    /// Two of the eight bits are not a flag at all: together they are a
    /// *count*, and [`Self::MULTI_CLICK`] is the sub-field they occupy.
    /// `mouse_click_count` is the only thing that should read them.
    pub struct ModMask;

    const SHIFT = 0x02;
    const CTRL = 0x04;
    const ALT = 0x08;
    const META = 0x10;
    /// Click count 1 in the two-bit sub-field.
    const TWO_CLICK = 0x20;
    /// Click count 2 -- *not* [`Self::TWO_CLICK`] with another bit set.
    const THREE_CLICK = 0x40;
    /// Click count 3, which is both bits at once and so overlaps the two
    /// above.
    const FOUR_CLICK = 0x60;
    const CMD = 0x80;

    /// The two bits the click counts live in.
    const MULTI_CLICK = Self::TWO_CLICK.bits() | Self::THREE_CLICK.bits();
}

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
