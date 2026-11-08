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
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::c_int;

use crate::keycodes::KeyExtra;
use crate::types::Key;

use super::{extra, termcap};

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
pub const K_SPECIAL: u8 = 0x80;
/// Termcap-name bytes that are not termcap names: the second byte of an
/// escape that says what the third one means.
pub const KS_SPECIAL: u8 = 254;
pub const KS_EXTRA: u8 = 253;
pub const KS_MODIFIER: u8 = 252;
pub const KS_KEY: u8 = 242;
/// The second byte of a key whose first byte already says everything.
pub const KE_FILLER: u8 = b'X';

/// First bytes used by keys that are neither `KS_EXTRA` nor a termcap name.
/// `KS_ZERO` is the escaped NUL.
pub(crate) const KS_ZERO: u8 = 255;
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
pub fn key_escape(key: Key) -> [u8; 3] {
    let (second, third) = if key == Key::byte(K_SPECIAL) {
        (KS_SPECIAL, KE_FILLER)
    } else if key.is_nul() {
        (KS_ZERO, KE_FILLER)
    } else {
        let name = super::tables::termcap_name(key);
        (name[0], name[1])
    };
    [K_SPECIAL, second, third]
}

/// The key an escape's two trailing bytes stand for: upstream's `TO_SPECIAL`.
///
/// The inverse of [`key_escape`], except that a NUL comes back as the key
/// code `K_ZERO` rather than as 0 — the streams cannot carry a bare NUL, so
/// callers that want the byte back test for `K_ZERO` themselves.
pub fn key_unescape(second: u8, third: u8) -> Key {
    if second == KS_SPECIAL {
        Key::byte(K_SPECIAL)
    } else if second == KS_ZERO {
        Key::from_special(SpecialKey::Zero)
    } else {
        super::tables::termcap_key([second, third])
    }
}

/// The second byte of a `KS_EXTRA` key. Upstream's `enum key_extra`.
pub const KE_S_UP: KeyExtra = 4;
pub const KE_S_DOWN: KeyExtra = 5;
pub const KE_S_F1: KeyExtra = 6;
pub const KE_S_F2: KeyExtra = 7;
pub const KE_S_F3: KeyExtra = 8;
pub const KE_S_F4: KeyExtra = 9;
pub const KE_S_F5: KeyExtra = 10;
pub const KE_S_F6: KeyExtra = 11;
pub const KE_S_F7: KeyExtra = 12;
pub const KE_S_F8: KeyExtra = 13;
pub const KE_S_F9: KeyExtra = 14;
pub const KE_S_F10: KeyExtra = 15;
pub const KE_S_F11: KeyExtra = 16;
pub const KE_S_F12: KeyExtra = 17;
pub const KE_S_F13: KeyExtra = 18;
pub const KE_S_F14: KeyExtra = 19;
pub const KE_S_F15: KeyExtra = 20;
pub const KE_S_F16: KeyExtra = 21;
pub const KE_S_F17: KeyExtra = 22;
pub const KE_S_F18: KeyExtra = 23;
pub const KE_S_F19: KeyExtra = 24;
pub const KE_S_F20: KeyExtra = 25;
pub const KE_S_F21: KeyExtra = 26;
pub const KE_S_F22: KeyExtra = 27;
pub const KE_S_F23: KeyExtra = 28;
pub const KE_S_F24: KeyExtra = 29;
pub const KE_S_F25: KeyExtra = 30;
pub const KE_S_F26: KeyExtra = 31;
pub const KE_S_F27: KeyExtra = 32;
pub const KE_S_F28: KeyExtra = 33;
pub const KE_S_F29: KeyExtra = 34;
pub const KE_S_F30: KeyExtra = 35;
pub const KE_S_F31: KeyExtra = 36;
pub const KE_S_F32: KeyExtra = 37;
pub const KE_S_F33: KeyExtra = 38;
pub const KE_S_F34: KeyExtra = 39;
pub const KE_S_F35: KeyExtra = 40;
pub const KE_S_F36: KeyExtra = 41;
pub const KE_S_F37: KeyExtra = 42;
pub const KE_LEFTMOUSE: KeyExtra = 44;
pub const KE_LEFTDRAG: KeyExtra = 45;
pub const KE_LEFTRELEASE: KeyExtra = 46;
pub const KE_MIDDLEMOUSE: KeyExtra = 47;
pub const KE_MIDDLEDRAG: KeyExtra = 48;
pub const KE_MIDDLERELEASE: KeyExtra = 49;
pub const KE_RIGHTMOUSE: KeyExtra = 50;
pub const KE_RIGHTDRAG: KeyExtra = 51;
pub const KE_RIGHTRELEASE: KeyExtra = 52;
pub const KE_IGNORE: KeyExtra = 53;
pub const KE_TAB: KeyExtra = 54;
pub const KE_XF1: KeyExtra = 57;
pub const KE_XF2: KeyExtra = 58;
pub const KE_XF3: KeyExtra = 59;
pub const KE_XF4: KeyExtra = 60;
pub const KE_XEND: KeyExtra = 61;
pub const KE_ZEND: KeyExtra = 62;
pub const KE_XHOME: KeyExtra = 63;
pub const KE_ZHOME: KeyExtra = 64;
pub const KE_XUP: KeyExtra = 65;
pub const KE_XDOWN: KeyExtra = 66;
pub const KE_XLEFT: KeyExtra = 67;
pub const KE_XRIGHT: KeyExtra = 68;
pub const KE_LEFTMOUSE_NM: KeyExtra = 69;
pub const KE_LEFTRELEASE_NM: KeyExtra = 70;
pub const KE_S_XF1: KeyExtra = 71;
pub const KE_S_XF2: KeyExtra = 72;
pub const KE_S_XF3: KeyExtra = 73;
pub const KE_S_XF4: KeyExtra = 74;
pub const KE_MOUSEDOWN: KeyExtra = 75;
pub const KE_MOUSEUP: KeyExtra = 76;
pub const KE_MOUSELEFT: KeyExtra = 77;
pub const KE_MOUSERIGHT: KeyExtra = 78;
pub const KE_KINS: KeyExtra = 79;
pub const KE_KDEL: KeyExtra = 80;
pub const KE_SNR: KeyExtra = 82;
pub const KE_PLUG: KeyExtra = 83;
pub const KE_C_LEFT: KeyExtra = 85;
pub const KE_C_RIGHT: KeyExtra = 86;
pub const KE_C_HOME: KeyExtra = 87;
pub const KE_C_END: KeyExtra = 88;
pub const KE_X1MOUSE: KeyExtra = 89;
pub const KE_X1DRAG: KeyExtra = 90;
pub const KE_X1RELEASE: KeyExtra = 91;
pub const KE_X2MOUSE: KeyExtra = 92;
pub const KE_X2DRAG: KeyExtra = 93;
pub const KE_X2RELEASE: KeyExtra = 94;
pub const KE_DROP: KeyExtra = 95;
pub const KE_CMDWIN: KeyExtra = 84;
pub const KE_NOP: KeyExtra = 97;
pub const KE_MOUSEMOVE: KeyExtra = 100;
pub const KE_EVENT: KeyExtra = 102;
pub const KE_LUA: KeyExtra = 103;
pub const KE_COMMAND: KeyExtra = 104;
pub const KE_WILD: KeyExtra = 108;

/// A key that is not a character: everything the `<>` notation has a name
/// for and no code point stands for.
///
/// The discriminants are the key codes themselves, written the way
/// upstream writes them -- a `KS_EXTRA` byte plus a [`KeyExtra`] code, or a
/// two-character termcap name -- so the enum *is* the definition and not a
/// second copy of it. They are all negative, which is what
/// [`Key::is_special`] tests and what lets a key and a character share one `c_int`: `vgetc` and
/// the typeahead carry either, and [`SpecialKey::try_from`] is the seam between
/// them. Raw bytes stay bytes; only the *named* codes are in here.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum SpecialKey {
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
    /// `<Tab>` as a special key: what `<S-Tab>` is without its shift.
    Tab = extra(KE_TAB),
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
    HorScrollbar = termcap(KS_HOR_SCROLLBAR, KE_FILLER),
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
    Mouse = termcap(KS_MOUSE, KE_FILLER),
    Pagedown = termcap(b'k', b'N'),
    Pageup = termcap(b'k', b'P'),
    PasteEnd = termcap(b'P', b'E'),
    PasteStart = termcap(b'P', b'S'),
    Right = termcap(b'k', b'r'),
    Select = termcap(KS_SELECT, KE_FILLER),
    SEnd = termcap(b'*', b'7'),
    SHome = termcap(b'#', b'2'),
    SLeft = termcap(b'#', b'4'),
    SRight = termcap(b'%', b'i'),
    STab = termcap(b'k', b'B'),
    Undo = termcap(b'&', b'8'),
    Up = termcap(b'k', b'u'),
    VerScrollbar = termcap(KS_VER_SCROLLBAR, KE_FILLER),
    Zero = termcap(KS_ZERO, KE_FILLER),
}

impl SpecialKey {
    /// Every key, in *code* order, which is why the list reads as though it
    /// were shuffled: the codes run from `Up`'s termcap name down to the
    /// highest `KE_*` byte, and nothing else about the order means anything.
    /// It is the table [`SpecialKey::try_from`] binary searches, and the `const`
    /// block below fails the build if it ever stops being sorted.
    pub const ALL: [SpecialKey; 185] = [
        SpecialKey::Up,
        SpecialKey::Kup,
        SpecialKey::Right,
        SpecialKey::Kright,
        SpecialKey::F63,
        SpecialKey::F62,
        SpecialKey::F61,
        SpecialKey::F60,
        SpecialKey::F59,
        SpecialKey::F58,
        SpecialKey::Wild,
        SpecialKey::Left,
        SpecialKey::Kleft,
        SpecialKey::F57,
        SpecialKey::F56,
        SpecialKey::F55,
        SpecialKey::F54,
        SpecialKey::SRight,
        SpecialKey::Command,
        SpecialKey::Home,
        SpecialKey::F53,
        SpecialKey::Lua,
        SpecialKey::F52,
        SpecialKey::Event,
        SpecialKey::F51,
        SpecialKey::F50,
        SpecialKey::Mousemove,
        SpecialKey::Down,
        SpecialKey::Kdown,
        SpecialKey::F49,
        SpecialKey::F48,
        SpecialKey::Bs,
        SpecialKey::F47,
        SpecialKey::Nop,
        SpecialKey::F46,
        SpecialKey::X2release,
        SpecialKey::X2drag,
        SpecialKey::X2mouse,
        SpecialKey::X1release,
        SpecialKey::X1drag,
        SpecialKey::F45,
        SpecialKey::X1mouse,
        SpecialKey::F44,
        SpecialKey::Zero,
        SpecialKey::CEnd,
        SpecialKey::Mouse,
        SpecialKey::VerScrollbar,
        SpecialKey::HorScrollbar,
        SpecialKey::Select,
        SpecialKey::F43,
        SpecialKey::CHome,
        SpecialKey::F42,
        SpecialKey::CRight,
        SpecialKey::F41,
        SpecialKey::CLeft,
        SpecialKey::F40,
        SpecialKey::Cmdwin,
        SpecialKey::F39,
        SpecialKey::Plug,
        SpecialKey::PasteStart,
        SpecialKey::F38,
        SpecialKey::Snr,
        SpecialKey::F37,
        SpecialKey::F36,
        SpecialKey::Kdel,
        SpecialKey::Pageup,
        SpecialKey::F35,
        SpecialKey::Kins,
        SpecialKey::F34,
        SpecialKey::Mouseright,
        SpecialKey::Pagedown,
        SpecialKey::Kequal,
        SpecialKey::F33,
        SpecialKey::Mouseleft,
        SpecialKey::Kcomma,
        SpecialKey::F32,
        SpecialKey::Mouseup,
        SpecialKey::K9,
        SpecialKey::F31,
        SpecialKey::Mousedown,
        SpecialKey::K8,
        SpecialKey::F30,
        SpecialKey::SXf4,
        SpecialKey::K7,
        SpecialKey::F29,
        SpecialKey::SXf3,
        SpecialKey::Ins,
        SpecialKey::K6,
        SpecialKey::F28,
        SpecialKey::SXf2,
        SpecialKey::K5,
        SpecialKey::F27,
        SpecialKey::SXf1,
        SpecialKey::K4,
        SpecialKey::F26,
        SpecialKey::LeftreleaseNm,
        SpecialKey::K3,
        SpecialKey::F25,
        SpecialKey::LeftmouseNm,
        SpecialKey::PasteEnd,
        SpecialKey::K2,
        SpecialKey::F24,
        SpecialKey::Xright,
        SpecialKey::Del,
        SpecialKey::K1,
        SpecialKey::F23,
        SpecialKey::Xleft,
        SpecialKey::K0,
        SpecialKey::F22,
        SpecialKey::Xdown,
        SpecialKey::STab,
        SpecialKey::Kpoint,
        SpecialKey::F21,
        SpecialKey::Xup,
        SpecialKey::Kenter,
        SpecialKey::F20,
        SpecialKey::Zhome,
        SpecialKey::Xhome,
        SpecialKey::Zend,
        SpecialKey::Xend,
        SpecialKey::Xf4,
        SpecialKey::Xf3,
        SpecialKey::F10,
        SpecialKey::Xf2,
        SpecialKey::Xf1,
        SpecialKey::F9,
        SpecialKey::Kmultiply,
        SpecialKey::F19,
        SpecialKey::F8,
        SpecialKey::Kdivide,
        SpecialKey::F18,
        SpecialKey::Undo,
        SpecialKey::F7,
        SpecialKey::Kminus,
        SpecialKey::F17,
        SpecialKey::End,
        SpecialKey::SEnd,
        SpecialKey::Tab,
        SpecialKey::F6,
        SpecialKey::Kplus,
        SpecialKey::F16,
        SpecialKey::Kselect,
        SpecialKey::Ignore,
        SpecialKey::F5,
        SpecialKey::Kpagedown,
        SpecialKey::F15,
        SpecialKey::Rightrelease,
        SpecialKey::F4,
        SpecialKey::Kend,
        SpecialKey::F14,
        SpecialKey::SLeft,
        SpecialKey::Rightdrag,
        SpecialKey::F3,
        SpecialKey::Kpageup,
        SpecialKey::F13,
        SpecialKey::Rightmouse,
        SpecialKey::F2,
        SpecialKey::Korigin,
        SpecialKey::F12,
        SpecialKey::SHome,
        SpecialKey::Middlerelease,
        SpecialKey::F1,
        SpecialKey::Khome,
        SpecialKey::F11,
        SpecialKey::Help,
        SpecialKey::Middledrag,
        SpecialKey::Find,
        SpecialKey::Middlemouse,
        SpecialKey::Leftrelease,
        SpecialKey::Leftdrag,
        SpecialKey::Leftmouse,
        SpecialKey::SF12,
        SpecialKey::SF11,
        SpecialKey::SF10,
        SpecialKey::SF9,
        SpecialKey::SF8,
        SpecialKey::SF7,
        SpecialKey::SF6,
        SpecialKey::SF5,
        SpecialKey::SF4,
        SpecialKey::SF3,
        SpecialKey::SF2,
        SpecialKey::SF1,
        SpecialKey::SDown,
        SpecialKey::SUp,
    ];

    /// The key code itself, for the many places that still carry one as a
    /// `c_int` beside the characters.
    pub const fn code(self) -> c_int {
        self as c_int
    }
}

/// [`SpecialKey::ALL`] has to stay sorted for the binary search below, and the
/// assertion is here rather than in a test because a `const` can check it.
const _: () = {
    let mut i = 1;
    while i < SpecialKey::ALL.len() {
        assert!(
            SpecialKey::ALL[i - 1].code() < SpecialKey::ALL[i].code(),
            "SpecialKey::ALL is out of order"
        );
        i += 1;
    }
};

/// The number is not one of the named keys -- most often because it is a
/// character, which shares the space.
///
/// It carries the number so that a dispatch over both halves can name a
/// character in the same `match`: `Err(NotAKey(Ctrl_B))` is the arm for a
/// literal CTRL-B beside `Ok(SpecialKey::Home)` for the key.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NotAKey(pub c_int);

impl TryFrom<c_int> for SpecialKey {
    type Error = NotAKey;

    fn try_from(code: c_int) -> Result<SpecialKey, NotAKey> {
        match SpecialKey::ALL.binary_search_by_key(&code, |key| key.code()) {
            Ok(i) => Ok(SpecialKey::ALL[i]),
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

impl ModMask {
    /// The modifiers as the one byte they travel as after `KS_MODIFIER`.
    /// Every flag lives in the low byte, so nothing is lost.
    #[inline]
    pub const fn byte(self) -> u8 {
        self.bits().to_le_bytes()[0]
    }

    /// The modifiers a `KS_MODIFIER` byte carries.
    #[inline]
    pub const fn from_byte(byte: u8) -> Self {
        Self::from_bits(c_int::from_le_bytes([byte, 0, 0, 0]))
    }
}

/// Which mouse button an event is about.
pub const MOUSE_LEFT: c_int = 0x00;
pub const MOUSE_MIDDLE: c_int = 0x01;
pub const MOUSE_RIGHT: c_int = 0x02;
pub const MOUSE_X1: c_int = 0x300;
pub const MOUSE_X2: c_int = 0x400;
/// Not a button: a release or a move with no button down.
pub const MOUSE_RELEASE: c_int = 0x03;

/// Flags for [`special_key_at`](super::special_key_at) and
/// [`trans_special_into`](super::trans_special_into).
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
pub const MAX_KEY_NAME_LEN: usize = 32;
