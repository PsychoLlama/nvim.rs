//! The four key tables, and every lookup over them.
//!
//! Upstream keeps the key names in `src/nvim/keycodes.lua` and turns them
//! into a C array plus a perfect-hash dispatch at build time (`v0.12.4`'s
//! `src/gen/gen_keycodes.lua`, whose output is ~430 lines of generated
//! `switch`). Here the array is sorted by lower-cased name and binary
//! searched instead, which answers the same lookups without a generator:
//! [`KEY_NAMES`] carries a `const` assertion that it is still in order, so
//! adding a name is a matter of putting the row in the right place.
//!
//! Everything in this module is data plus total functions over it. The
//! raw-pointer entry points the rest of the editor calls stay in the parent,
//! which is what lets this half carry `forbid(unsafe_code)`.

#![forbid(unsafe_code)]

use crate::keycodes::ModMask;
use crate::types::CAR;
use crate::types::ESC;
use crate::types::NL;
use crate::types::TAB;
use core::ffi::c_int;
use core::iter;

use super::*;
use crate::keycodes::{
    KE_C_END, KE_C_HOME, KE_C_LEFT, KE_C_RIGHT, KE_DROP, KE_IGNORE, KE_LEFTDRAG, KE_LEFTMOUSE,
    KE_LEFTRELEASE, KE_MIDDLEDRAG, KE_MIDDLEMOUSE, KE_MIDDLERELEASE, KE_MOUSEMOVE, KE_PLUG,
    KE_RIGHTDRAG, KE_RIGHTMOUSE, KE_RIGHTRELEASE, KE_S_DOWN, KE_S_F1, KE_S_F2, KE_S_F3, KE_S_F4,
    KE_S_F5, KE_S_F6, KE_S_F7, KE_S_F8, KE_S_F9, KE_S_F10, KE_S_F11, KE_S_F12, KE_S_F13, KE_S_F14,
    KE_S_F15, KE_S_F16, KE_S_F17, KE_S_F18, KE_S_F19, KE_S_F20, KE_S_F21, KE_S_F22, KE_S_F23,
    KE_S_F24, KE_S_F25, KE_S_F26, KE_S_F27, KE_S_F28, KE_S_F29, KE_S_F30, KE_S_F31, KE_S_F32,
    KE_S_F33, KE_S_F34, KE_S_F35, KE_S_F36, KE_S_F37, KE_S_UP, KE_S_XF1, KE_S_XF2, KE_S_XF3,
    KE_S_XF4, KE_SNR, KE_TAB, KE_X1DRAG, KE_X1MOUSE, KE_X1RELEASE, KE_X2DRAG, KE_X2MOUSE,
    KE_X2RELEASE, KE_XF1, KE_XF2, KE_XF3, KE_XF4,
};

/// A modifier prefix letter and the bits it stands for.
struct ModPrefix {
    /// The bits of the modifier state this row is about.
    mask: ModMask,
    /// The value those bits must have for the row to apply. Only the
    /// multi-click rows differ from `mask`: they share one two-bit field.
    flag: ModMask,
    /// The letter that spells the modifier in `<X-Key>`.
    name: u8,
}

const fn mod_prefix(mask: ModMask, flag: ModMask, name: u8) -> ModPrefix {
    ModPrefix { mask, flag, name }
}

/// The modifiers that are spelled out when a key is printed, in the order
/// they are printed: `<M-C-S-Up>`, never `<S-C-M-Up>`.
static PRINTED_MOD_MASKS: [ModPrefix; 8] = [
    mod_prefix(ModMask::ALT, ModMask::ALT, b'M'),
    mod_prefix(ModMask::META, ModMask::META, b'T'),
    mod_prefix(ModMask::CTRL, ModMask::CTRL, b'C'),
    mod_prefix(ModMask::SHIFT, ModMask::SHIFT, b'S'),
    mod_prefix(ModMask::MULTI_CLICK, ModMask::TWO_CLICK, b'2'),
    mod_prefix(ModMask::MULTI_CLICK, ModMask::THREE_CLICK, b'3'),
    mod_prefix(ModMask::MULTI_CLICK, ModMask::FOUR_CLICK, b'4'),
    mod_prefix(ModMask::CMD, ModMask::CMD, b'D'),
];

/// `A` is an alternative spelling of `M`, accepted where a key is written but
/// never printed — which is why it is not one of [`PRINTED_MOD_MASKS`].
static ALT_MOD_MASK: ModPrefix = mod_prefix(ModMask::ALT, ModMask::ALT, b'A');

/// The modifier bit a `<X-Key>` prefix letter names — `S` for shift, `C` for
/// ctrl — or 0 when it names no modifier. ASCII case is folded.
pub fn name_to_mod_mask(c: c_int) -> ModMask {
    let c = to_upper_ascii(c);
    PRINTED_MOD_MASKS
        .iter()
        .chain(iter::once(&ALT_MOD_MASK))
        .find(|m| c == c_int::from(m.name))
        .map_or(ModMask::NONE, |m| m.flag)
}

/// The modifier prefixes `modifiers` calls for, in printing order.
pub(crate) fn printed_modifiers(modifiers: ModMask) -> impl Iterator<Item = u8> {
    PRINTED_MOD_MASKS
        .iter()
        .filter(move |m| modifiers.masked(m.mask) == m.flag)
        .map(|m| m.name)
}

/// A terminal code that already carries a modifier, and the code for the same
/// key without it. Mouse codes are deliberately not in here.
struct ModifierKey {
    /// The modifier bit the shifted code carries.
    modifier: ModMask,
    /// Termcap name of the key *with* the modifier.
    with: [u8; 2],
    /// Termcap name of the key *without* it.
    without: [u8; 2],
}

const fn mod_key(modifier: ModMask, with: [u8; 2], without: [u8; 2]) -> ModifierKey {
    ModifierKey {
        modifier,
        with,
        without,
    }
}

/// The modifier bits and the `KS_EXTRA` marker, narrowed to the byte type
/// [`MODIFIER_KEYS`] stores. Upstream's table is a flat `uint8_t[]`.
const SHIFT: ModMask = ModMask::SHIFT;
const CTRL: ModMask = ModMask::CTRL;
const EXTRA: u8 = KS_EXTRA as u8;

/// Shifted and ctrl'ed terminal codes, and the unmodified code each stands
/// for. Terminals send `<S-Up>` as a code of its own rather than as `Up` plus
/// a modifier byte, and this is the table that relates the two.
static MODIFIER_KEYS: [ModifierKey; 75] = [
    mod_key(SHIFT, *b"&9", *b"@1"),
    mod_key(SHIFT, *b"&0", *b"@2"),
    mod_key(SHIFT, *b"*1", *b"@4"),
    mod_key(SHIFT, *b"*2", *b"@5"),
    mod_key(SHIFT, *b"*3", *b"@6"),
    mod_key(SHIFT, *b"*4", *b"kD"),
    mod_key(SHIFT, *b"*5", *b"kL"),
    mod_key(SHIFT, *b"*7", *b"@7"),
    mod_key(CTRL, [EXTRA, KE_C_END as u8], *b"@7"),
    mod_key(SHIFT, *b"*9", *b"@9"),
    mod_key(SHIFT, *b"*0", *b"@0"),
    mod_key(SHIFT, *b"#1", *b"%1"),
    mod_key(SHIFT, *b"#2", *b"kh"),
    mod_key(CTRL, [EXTRA, KE_C_HOME as u8], *b"kh"),
    mod_key(SHIFT, *b"#3", *b"kI"),
    mod_key(SHIFT, *b"#4", *b"kl"),
    mod_key(CTRL, [EXTRA, KE_C_LEFT as u8], *b"kl"),
    mod_key(SHIFT, *b"%a", *b"%3"),
    mod_key(SHIFT, *b"%b", *b"%4"),
    mod_key(SHIFT, *b"%c", *b"%5"),
    mod_key(SHIFT, *b"%d", *b"%7"),
    mod_key(SHIFT, *b"%e", *b"%8"),
    mod_key(SHIFT, *b"%f", *b"%9"),
    mod_key(SHIFT, *b"%g", *b"%0"),
    mod_key(SHIFT, *b"%h", *b"&3"),
    mod_key(SHIFT, *b"%i", *b"kr"),
    mod_key(CTRL, [EXTRA, KE_C_RIGHT as u8], *b"kr"),
    mod_key(SHIFT, *b"%j", *b"&5"),
    mod_key(SHIFT, *b"!1", *b"&6"),
    mod_key(SHIFT, *b"!2", *b"&7"),
    mod_key(SHIFT, *b"!3", *b"&8"),
    mod_key(SHIFT, [EXTRA, KE_S_UP as u8], *b"ku"),
    mod_key(SHIFT, [EXTRA, KE_S_DOWN as u8], *b"kd"),
    mod_key(SHIFT, [EXTRA, KE_S_XF1 as u8], [EXTRA, KE_XF1 as u8]),
    mod_key(SHIFT, [EXTRA, KE_S_XF2 as u8], [EXTRA, KE_XF2 as u8]),
    mod_key(SHIFT, [EXTRA, KE_S_XF3 as u8], [EXTRA, KE_XF3 as u8]),
    mod_key(SHIFT, [EXTRA, KE_S_XF4 as u8], [EXTRA, KE_XF4 as u8]),
    mod_key(SHIFT, [EXTRA, KE_S_F1 as u8], *b"k1"),
    mod_key(SHIFT, [EXTRA, KE_S_F2 as u8], *b"k2"),
    mod_key(SHIFT, [EXTRA, KE_S_F3 as u8], *b"k3"),
    mod_key(SHIFT, [EXTRA, KE_S_F4 as u8], *b"k4"),
    mod_key(SHIFT, [EXTRA, KE_S_F5 as u8], *b"k5"),
    mod_key(SHIFT, [EXTRA, KE_S_F6 as u8], *b"k6"),
    mod_key(SHIFT, [EXTRA, KE_S_F7 as u8], *b"k7"),
    mod_key(SHIFT, [EXTRA, KE_S_F8 as u8], *b"k8"),
    mod_key(SHIFT, [EXTRA, KE_S_F9 as u8], *b"k9"),
    mod_key(SHIFT, [EXTRA, KE_S_F10 as u8], *b"k;"),
    mod_key(SHIFT, [EXTRA, KE_S_F11 as u8], *b"F1"),
    mod_key(SHIFT, [EXTRA, KE_S_F12 as u8], *b"F2"),
    mod_key(SHIFT, [EXTRA, KE_S_F13 as u8], *b"F3"),
    mod_key(SHIFT, [EXTRA, KE_S_F14 as u8], *b"F4"),
    mod_key(SHIFT, [EXTRA, KE_S_F15 as u8], *b"F5"),
    mod_key(SHIFT, [EXTRA, KE_S_F16 as u8], *b"F6"),
    mod_key(SHIFT, [EXTRA, KE_S_F17 as u8], *b"F7"),
    mod_key(SHIFT, [EXTRA, KE_S_F18 as u8], *b"F8"),
    mod_key(SHIFT, [EXTRA, KE_S_F19 as u8], *b"F9"),
    mod_key(SHIFT, [EXTRA, KE_S_F20 as u8], *b"FA"),
    mod_key(SHIFT, [EXTRA, KE_S_F21 as u8], *b"FB"),
    mod_key(SHIFT, [EXTRA, KE_S_F22 as u8], *b"FC"),
    mod_key(SHIFT, [EXTRA, KE_S_F23 as u8], *b"FD"),
    mod_key(SHIFT, [EXTRA, KE_S_F24 as u8], *b"FE"),
    mod_key(SHIFT, [EXTRA, KE_S_F25 as u8], *b"FF"),
    mod_key(SHIFT, [EXTRA, KE_S_F26 as u8], *b"FG"),
    mod_key(SHIFT, [EXTRA, KE_S_F27 as u8], *b"FH"),
    mod_key(SHIFT, [EXTRA, KE_S_F28 as u8], *b"FI"),
    mod_key(SHIFT, [EXTRA, KE_S_F29 as u8], *b"FJ"),
    mod_key(SHIFT, [EXTRA, KE_S_F30 as u8], *b"FK"),
    mod_key(SHIFT, [EXTRA, KE_S_F31 as u8], *b"FL"),
    mod_key(SHIFT, [EXTRA, KE_S_F32 as u8], *b"FM"),
    mod_key(SHIFT, [EXTRA, KE_S_F33 as u8], *b"FN"),
    mod_key(SHIFT, [EXTRA, KE_S_F34 as u8], *b"FO"),
    mod_key(SHIFT, [EXTRA, KE_S_F35 as u8], *b"FP"),
    mod_key(SHIFT, [EXTRA, KE_S_F36 as u8], *b"FQ"),
    mod_key(SHIFT, [EXTRA, KE_S_F37 as u8], *b"FR"),
    mod_key(SHIFT, *b"kB", [EXTRA, KE_TAB as u8]),
];

/// The two termcap-name bytes of a key code, read the way C's `KEY2TERMCAP*`
/// macros read them: the low two bytes of the code's negation. Nothing checks
/// that `key` is negative — see [`simplify`].
pub(crate) fn termcap_name(key: c_int) -> [u8; 2] {
    let bits = -key as u32;
    [(bits & 0xff) as u8, ((bits >> 8) & 0xff) as u8]
}

/// The key code a two-byte termcap name stands for (C's `TERMCAP2KEY`).
pub(crate) const fn termcap_key(name: [u8; 2]) -> c_int {
    -((name[0] as c_int) + ((name[1] as c_int) << 8))
}

/// The single code that stands for `key` with one of `modifiers` held down,
/// and the modifiers left over — `None` when no terminal code covers it.
///
/// The comparison is on the low two bytes of `-key` rather than on the code
/// itself, exactly as the C is. A large *positive* key congruent to one of the
/// table's codes modulo 65536 therefore matches: `<S-U+8A95>` comes out as
/// `<S-Up>`. Faithful to upstream, and left that way deliberately.
pub(crate) fn simplify(key: c_int, modifiers: ModMask) -> Option<(c_int, ModMask)> {
    let name = termcap_name(key);
    let row = MODIFIER_KEYS
        .iter()
        .find(|m| name == m.without && modifiers.has(m.modifier))?;
    Some((termcap_key(row.with), modifiers.without(row.modifier)))
}

/// The unmodified key a shifted terminal code stands for, and the modifier it
/// carries — the reverse of [`simplify`], used when a code is printed.
pub(crate) fn unshift(key: c_int) -> Option<(c_int, ModMask)> {
    let name = termcap_name(key);
    let row = MODIFIER_KEYS.iter().find(|m| name == m.with)?;
    Some((termcap_key(row.without), row.modifier))
}

/// One name of one key code.
struct KeyName {
    /// The code this name spells.
    key: c_int,
    /// Set when the name is an alternative spelling — `Return` for `CR`, say.
    /// Exactly one row per code has it clear, and that is the name the code is
    /// printed as.
    is_alt: bool,
    /// The name itself. ASCII, matched case-insensitively.
    name: &'static str,
}

const fn key_name(key: c_int, is_alt: bool, name: &'static str) -> KeyName {
    KeyName { key, is_alt, name }
}

/// Every name the `<>` notation knows, **sorted by lower-cased name** — the
/// order [`code_for_name`] binary searches and the `_SORTED` block below
/// asserts at compile time.
///
/// Where two rows share a name (`TAB` and `K_TAB` are both `Tab`) the first
/// wins the lookup, so their relative order is load-bearing too.
static KEY_NAMES: [KeyName; 187] = KEY_NAME_ROWS;

/// The rows of [`KEY_NAMES`], separately, and as a `const` so that the order
/// can be asserted below: a `static` cannot be read from a `const` context.
const KEY_NAME_ROWS: [KeyName; 187] = [
    key_name(Key::Bs.code(), true, "BackSpace"),
    key_name('|' as c_int, false, "Bar"),
    key_name(Key::Bs.code(), false, "BS"),
    key_name('\\' as c_int, false, "Bslash"),
    key_name(Key::Command.code(), false, "Cmd"),
    key_name(CAR, false, "CR"),
    key_name(CSI, false, "CSI"),
    key_name(Key::Del.code(), false, "Del"),
    key_name(Key::Del.code(), true, "Delete"),
    key_name(Key::Down.code(), false, "Down"),
    key_name(extra(KE_DROP), false, "Drop"),
    key_name(Key::End.code(), false, "End"),
    key_name(CAR, true, "Enter"),
    key_name(ESC, false, "Esc"),
    key_name(ESC, true, "Escape"),
    key_name(Key::F1.code(), false, "F1"),
    key_name(Key::F10.code(), false, "F10"),
    key_name(Key::F11.code(), false, "F11"),
    key_name(Key::F12.code(), false, "F12"),
    key_name(Key::F13.code(), false, "F13"),
    key_name(Key::F14.code(), false, "F14"),
    key_name(Key::F15.code(), false, "F15"),
    key_name(Key::F16.code(), false, "F16"),
    key_name(Key::F17.code(), false, "F17"),
    key_name(Key::F18.code(), false, "F18"),
    key_name(Key::F19.code(), false, "F19"),
    key_name(Key::F2.code(), false, "F2"),
    key_name(Key::F20.code(), false, "F20"),
    key_name(Key::F21.code(), false, "F21"),
    key_name(Key::F22.code(), false, "F22"),
    key_name(Key::F23.code(), false, "F23"),
    key_name(Key::F24.code(), false, "F24"),
    key_name(Key::F25.code(), false, "F25"),
    key_name(Key::F26.code(), false, "F26"),
    key_name(Key::F27.code(), false, "F27"),
    key_name(Key::F28.code(), false, "F28"),
    key_name(Key::F29.code(), false, "F29"),
    key_name(Key::F3.code(), false, "F3"),
    key_name(Key::F30.code(), false, "F30"),
    key_name(Key::F31.code(), false, "F31"),
    key_name(Key::F32.code(), false, "F32"),
    key_name(Key::F33.code(), false, "F33"),
    key_name(Key::F34.code(), false, "F34"),
    key_name(Key::F35.code(), false, "F35"),
    key_name(Key::F36.code(), false, "F36"),
    key_name(Key::F37.code(), false, "F37"),
    key_name(Key::F38.code(), false, "F38"),
    key_name(Key::F39.code(), false, "F39"),
    key_name(Key::F4.code(), false, "F4"),
    key_name(Key::F40.code(), false, "F40"),
    key_name(Key::F41.code(), false, "F41"),
    key_name(Key::F42.code(), false, "F42"),
    key_name(Key::F43.code(), false, "F43"),
    key_name(Key::F44.code(), false, "F44"),
    key_name(Key::F45.code(), false, "F45"),
    key_name(Key::F46.code(), false, "F46"),
    key_name(Key::F47.code(), false, "F47"),
    key_name(Key::F48.code(), false, "F48"),
    key_name(Key::F49.code(), false, "F49"),
    key_name(Key::F5.code(), false, "F5"),
    key_name(Key::F50.code(), false, "F50"),
    key_name(Key::F51.code(), false, "F51"),
    key_name(Key::F52.code(), false, "F52"),
    key_name(Key::F53.code(), false, "F53"),
    key_name(Key::F54.code(), false, "F54"),
    key_name(Key::F55.code(), false, "F55"),
    key_name(Key::F56.code(), false, "F56"),
    key_name(Key::F57.code(), false, "F57"),
    key_name(Key::F58.code(), false, "F58"),
    key_name(Key::F59.code(), false, "F59"),
    key_name(Key::F6.code(), false, "F6"),
    key_name(Key::F60.code(), false, "F60"),
    key_name(Key::F61.code(), false, "F61"),
    key_name(Key::F62.code(), false, "F62"),
    key_name(Key::F63.code(), false, "F63"),
    key_name(Key::F7.code(), false, "F7"),
    key_name(Key::F8.code(), false, "F8"),
    key_name(Key::F9.code(), false, "F9"),
    key_name(Key::Find.code(), false, "Find"),
    key_name(Key::Help.code(), false, "Help"),
    key_name(Key::Home.code(), false, "Home"),
    key_name(Key::Ignore.code(), false, "Ignore"),
    key_name(Key::Ins.code(), true, "Ins"),
    key_name(Key::Ins.code(), false, "Insert"),
    key_name(Key::K0.code(), false, "k0"),
    key_name(Key::K1.code(), false, "k1"),
    key_name(Key::K2.code(), false, "k2"),
    key_name(Key::K3.code(), false, "k3"),
    key_name(Key::K4.code(), false, "k4"),
    key_name(Key::K5.code(), false, "k5"),
    key_name(Key::K6.code(), false, "k6"),
    key_name(Key::K7.code(), false, "k7"),
    key_name(Key::K8.code(), false, "k8"),
    key_name(Key::K9.code(), false, "k9"),
    key_name(Key::Kcomma.code(), false, "kComma"),
    key_name(Key::Kdel.code(), false, "kDel"),
    key_name(Key::Kdivide.code(), false, "kDivide"),
    key_name(Key::Kdown.code(), false, "kDown"),
    key_name(Key::Kend.code(), false, "kEnd"),
    key_name(Key::Kenter.code(), false, "kEnter"),
    key_name(Key::Kequal.code(), false, "kEqual"),
    key_name(Key::Khome.code(), false, "kHome"),
    key_name(Key::Kins.code(), false, "kInsert"),
    key_name(Key::Kleft.code(), false, "kLeft"),
    key_name(Key::Kminus.code(), false, "kMinus"),
    key_name(Key::Kmultiply.code(), false, "kMultiply"),
    key_name(Key::Korigin.code(), false, "kOrigin"),
    key_name(Key::Kins.code(), true, "KP0"),
    key_name(Key::Kend.code(), true, "KP1"),
    key_name(Key::Kdown.code(), true, "KP2"),
    key_name(Key::Kpagedown.code(), true, "KP3"),
    key_name(Key::Kleft.code(), true, "KP4"),
    key_name(Key::Korigin.code(), true, "KP5"),
    key_name(Key::Kright.code(), true, "KP6"),
    key_name(Key::Khome.code(), true, "KP7"),
    key_name(Key::Kup.code(), true, "KP8"),
    key_name(Key::Kpageup.code(), true, "KP9"),
    key_name(Key::Kpagedown.code(), false, "kPageDown"),
    key_name(Key::Kpageup.code(), false, "kPageUp"),
    key_name(Key::Kcomma.code(), true, "KPComma"),
    key_name(Key::Kdivide.code(), true, "KPDiv"),
    key_name(Key::Kenter.code(), true, "KPEnter"),
    key_name(Key::Kequal.code(), true, "KPEquals"),
    key_name(Key::Kplus.code(), false, "kPlus"),
    key_name(Key::Kminus.code(), true, "KPMinus"),
    key_name(Key::Kmultiply.code(), true, "KPMult"),
    key_name(Key::Kpoint.code(), false, "kPoint"),
    key_name(Key::Kdel.code(), true, "KPPeriod"),
    key_name(Key::Kplus.code(), true, "KPPlus"),
    key_name(Key::Kright.code(), false, "kRight"),
    key_name(Key::Kup.code(), false, "kUp"),
    key_name(Key::Left.code(), false, "Left"),
    key_name(Key::Leftdrag.code(), false, "LeftDrag"),
    key_name(Key::Leftmouse.code(), false, "LeftMouse"),
    key_name(Key::LeftmouseNm.code(), false, "LeftMouseNM"),
    key_name(Key::Leftrelease.code(), false, "LeftRelease"),
    key_name(Key::LeftreleaseNm.code(), false, "LeftReleaseNM"),
    key_name(NL, true, "LF"),
    key_name(NL, true, "LineFeed"),
    key_name('<' as c_int, false, "lt"),
    key_name(Key::Middledrag.code(), false, "MiddleDrag"),
    key_name(Key::Middlemouse.code(), false, "MiddleMouse"),
    key_name(Key::Middlerelease.code(), false, "MiddleRelease"),
    key_name(Key::Mouse.code(), false, "Mouse"),
    key_name(Key::Mousedown.code(), true, "MouseDown"),
    key_name(Key::Mousemove.code(), false, "MouseMove"),
    key_name(Key::Mouseup.code(), true, "MouseUp"),
    key_name(NL, true, "NewLine"),
    key_name(NL, false, "NL"),
    key_name(Key::Zero.code(), false, "Nul"),
    key_name(Key::Pagedown.code(), false, "PageDown"),
    key_name(Key::Pageup.code(), false, "PageUp"),
    key_name(extra(KE_PLUG), false, "Plug"),
    key_name(CAR, true, "Return"),
    key_name(Key::Right.code(), false, "Right"),
    key_name(Key::Rightdrag.code(), false, "RightDrag"),
    key_name(Key::Rightmouse.code(), false, "RightMouse"),
    key_name(Key::Rightrelease.code(), false, "RightRelease"),
    key_name(Key::Mouseup.code(), false, "ScrollWheelDown"),
    key_name(Key::Mouseright.code(), false, "ScrollWheelLeft"),
    key_name(Key::Mouseleft.code(), false, "ScrollWheelRight"),
    key_name(Key::Mousedown.code(), false, "ScrollWheelUp"),
    key_name(Key::Kselect.code(), false, "Select"),
    key_name(extra(KE_SNR), false, "SNR"),
    key_name(' ' as c_int, false, "Space"),
    key_name(TAB, false, "Tab"),
    key_name(extra(KE_TAB), false, "Tab"),
    key_name(Key::Undo.code(), false, "Undo"),
    key_name(Key::Up.code(), false, "Up"),
    key_name(Key::X1drag.code(), false, "X1Drag"),
    key_name(Key::X1mouse.code(), false, "X1Mouse"),
    key_name(Key::X1release.code(), false, "X1Release"),
    key_name(Key::X2drag.code(), false, "X2Drag"),
    key_name(Key::X2mouse.code(), false, "X2Mouse"),
    key_name(Key::X2release.code(), false, "X2Release"),
    key_name(Key::Xdown.code(), false, "xDown"),
    key_name(Key::Xend.code(), false, "xEnd"),
    key_name(Key::Xf1.code(), false, "xF1"),
    key_name(extra(KE_XF2), false, "xF2"),
    key_name(extra(KE_XF3), false, "xF3"),
    key_name(extra(KE_XF4), false, "xF4"),
    key_name(Key::Xhome.code(), false, "xHome"),
    key_name(Key::Xleft.code(), false, "xLeft"),
    key_name(Key::Xright.code(), false, "xRight"),
    key_name(Key::Xup.code(), false, "xUp"),
    key_name(Key::Zend.code(), false, "zEnd"),
    key_name(Key::Zhome.code(), false, "zHome"),
];

/// `true` when `a` sorts before `b` with ASCII case folded — the order
/// [`KEY_NAMES`] is in, and the comparison [`code_for_name`] searches with.
const fn before_ignore_case(a: &[u8], b: &[u8]) -> bool {
    let mut i = 0;
    while i < a.len() && i < b.len() {
        let (x, y) = (a[i].to_ascii_lowercase(), b[i].to_ascii_lowercase());
        if x != y {
            return x < y;
        }
        i += 1;
    }
    a.len() < b.len()
}

/// [`KEY_NAMES`] is binary searched, so its order is a correctness condition
/// rather than a tidiness one. Check it where a mistake costs nothing.
const _SORTED: () = {
    let mut i = 1;
    while i < KEY_NAME_ROWS.len() {
        assert!(
            !before_ignore_case(
                KEY_NAME_ROWS[i].name.as_bytes(),
                KEY_NAME_ROWS[i - 1].name.as_bytes()
            ),
            "KEY_NAMES must stay sorted by lower-cased name"
        );
        i += 1;
    }
};

/// The code `name` spells, or 0 when the table has no such name. ASCII case is
/// folded, and `name` must be the whole name: `Esc` and `Escape` are separate
/// rows, and `Esc` does not match `Escape`.
pub(crate) fn code_for_name(name: &[u8]) -> c_int {
    let at = KEY_NAMES.partition_point(|e| before_ignore_case(e.name.as_bytes(), name));
    match KEY_NAMES.get(at) {
        Some(e) if e.name.as_bytes().eq_ignore_ascii_case(name) => e.key,
        _ => 0,
    }
}

/// The name `key` is printed as: the one row for `key` that is not an
/// alternative spelling.
pub(crate) fn name_of_code(key: c_int) -> Option<&'static str> {
    KEY_NAMES
        .iter()
        .find(|e| e.key == key && !e.is_alt)
        .map(|e| e.name)
}

/// Whether the table has a name for `key`.
pub fn has_key_name(key: c_int) -> bool {
    name_of_code(key).is_some()
}

/// What a mouse pseudo-code means.
pub(crate) struct MouseEvent {
    /// Which button, as a `MOUSE_*` code.
    pub button: c_int,
    /// A button going down.
    pub is_click: bool,
    /// The mouse moving with a button held.
    pub is_drag: bool,
}

/// A mouse pseudo-code and what it means.
struct MouseCode {
    code: c_int,
    event: MouseEvent,
}

const fn mouse_code(code: key_extra, button: c_int, is_click: bool, is_drag: bool) -> MouseCode {
    MouseCode {
        code: code as c_int,
        event: MouseEvent {
            button,
            is_click,
            is_drag,
        },
    }
}

/// Every mouse pseudo-code the terminal layer produces.
static MOUSE_CODES: [MouseCode; 17] = [
    mouse_code(KE_LEFTMOUSE, MOUSE_LEFT, true, false),
    mouse_code(KE_LEFTDRAG, MOUSE_LEFT, false, true),
    mouse_code(KE_LEFTRELEASE, MOUSE_LEFT, false, false),
    mouse_code(KE_MIDDLEMOUSE, MOUSE_MIDDLE, true, false),
    mouse_code(KE_MIDDLEDRAG, MOUSE_MIDDLE, false, true),
    mouse_code(KE_MIDDLERELEASE, MOUSE_MIDDLE, false, false),
    mouse_code(KE_RIGHTMOUSE, MOUSE_RIGHT, true, false),
    mouse_code(KE_RIGHTDRAG, MOUSE_RIGHT, false, true),
    mouse_code(KE_RIGHTRELEASE, MOUSE_RIGHT, false, false),
    mouse_code(KE_X1MOUSE, MOUSE_X1, true, false),
    mouse_code(KE_X1DRAG, MOUSE_X1, false, true),
    mouse_code(KE_X1RELEASE, MOUSE_X1, false, false),
    mouse_code(KE_X2MOUSE, MOUSE_X2, true, false),
    mouse_code(KE_X2DRAG, MOUSE_X2, false, true),
    mouse_code(KE_X2RELEASE, MOUSE_X2, false, false),
    // A drag with no click before it.
    mouse_code(KE_MOUSEMOVE, MOUSE_RELEASE, false, true),
    // A release with no click before it.
    mouse_code(KE_IGNORE, MOUSE_RELEASE, false, false),
];

/// What `code` means as a mouse event, or `None` when it is not one.
pub(crate) fn mouse_event(code: c_int) -> Option<&'static MouseEvent> {
    MOUSE_CODES
        .iter()
        .find(|m| m.code == code)
        .map(|m| &m.event)
}
