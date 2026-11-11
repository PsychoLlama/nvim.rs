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
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::keycodes::ModMask;
use crate::types::Key;
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
#[derive(Debug)]
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
pub fn name_to_mod_mask(c: u8) -> ModMask {
    let c = c.to_ascii_uppercase();
    PRINTED_MOD_MASKS
        .iter()
        .chain(iter::once(&ALT_MOD_MASK))
        .find(|m| c == m.name)
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
#[derive(Debug)]
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
const EXTRA: u8 = KS_EXTRA;

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
    mod_key(CTRL, [EXTRA, KE_C_END], *b"@7"),
    mod_key(SHIFT, *b"*9", *b"@9"),
    mod_key(SHIFT, *b"*0", *b"@0"),
    mod_key(SHIFT, *b"#1", *b"%1"),
    mod_key(SHIFT, *b"#2", *b"kh"),
    mod_key(CTRL, [EXTRA, KE_C_HOME], *b"kh"),
    mod_key(SHIFT, *b"#3", *b"kI"),
    mod_key(SHIFT, *b"#4", *b"kl"),
    mod_key(CTRL, [EXTRA, KE_C_LEFT], *b"kl"),
    mod_key(SHIFT, *b"%a", *b"%3"),
    mod_key(SHIFT, *b"%b", *b"%4"),
    mod_key(SHIFT, *b"%c", *b"%5"),
    mod_key(SHIFT, *b"%d", *b"%7"),
    mod_key(SHIFT, *b"%e", *b"%8"),
    mod_key(SHIFT, *b"%f", *b"%9"),
    mod_key(SHIFT, *b"%g", *b"%0"),
    mod_key(SHIFT, *b"%h", *b"&3"),
    mod_key(SHIFT, *b"%i", *b"kr"),
    mod_key(CTRL, [EXTRA, KE_C_RIGHT], *b"kr"),
    mod_key(SHIFT, *b"%j", *b"&5"),
    mod_key(SHIFT, *b"!1", *b"&6"),
    mod_key(SHIFT, *b"!2", *b"&7"),
    mod_key(SHIFT, *b"!3", *b"&8"),
    mod_key(SHIFT, [EXTRA, KE_S_UP], *b"ku"),
    mod_key(SHIFT, [EXTRA, KE_S_DOWN], *b"kd"),
    mod_key(SHIFT, [EXTRA, KE_S_XF1], [EXTRA, KE_XF1]),
    mod_key(SHIFT, [EXTRA, KE_S_XF2], [EXTRA, KE_XF2]),
    mod_key(SHIFT, [EXTRA, KE_S_XF3], [EXTRA, KE_XF3]),
    mod_key(SHIFT, [EXTRA, KE_S_XF4], [EXTRA, KE_XF4]),
    mod_key(SHIFT, [EXTRA, KE_S_F1], *b"k1"),
    mod_key(SHIFT, [EXTRA, KE_S_F2], *b"k2"),
    mod_key(SHIFT, [EXTRA, KE_S_F3], *b"k3"),
    mod_key(SHIFT, [EXTRA, KE_S_F4], *b"k4"),
    mod_key(SHIFT, [EXTRA, KE_S_F5], *b"k5"),
    mod_key(SHIFT, [EXTRA, KE_S_F6], *b"k6"),
    mod_key(SHIFT, [EXTRA, KE_S_F7], *b"k7"),
    mod_key(SHIFT, [EXTRA, KE_S_F8], *b"k8"),
    mod_key(SHIFT, [EXTRA, KE_S_F9], *b"k9"),
    mod_key(SHIFT, [EXTRA, KE_S_F10], *b"k;"),
    mod_key(SHIFT, [EXTRA, KE_S_F11], *b"F1"),
    mod_key(SHIFT, [EXTRA, KE_S_F12], *b"F2"),
    mod_key(SHIFT, [EXTRA, KE_S_F13], *b"F3"),
    mod_key(SHIFT, [EXTRA, KE_S_F14], *b"F4"),
    mod_key(SHIFT, [EXTRA, KE_S_F15], *b"F5"),
    mod_key(SHIFT, [EXTRA, KE_S_F16], *b"F6"),
    mod_key(SHIFT, [EXTRA, KE_S_F17], *b"F7"),
    mod_key(SHIFT, [EXTRA, KE_S_F18], *b"F8"),
    mod_key(SHIFT, [EXTRA, KE_S_F19], *b"F9"),
    mod_key(SHIFT, [EXTRA, KE_S_F20], *b"FA"),
    mod_key(SHIFT, [EXTRA, KE_S_F21], *b"FB"),
    mod_key(SHIFT, [EXTRA, KE_S_F22], *b"FC"),
    mod_key(SHIFT, [EXTRA, KE_S_F23], *b"FD"),
    mod_key(SHIFT, [EXTRA, KE_S_F24], *b"FE"),
    mod_key(SHIFT, [EXTRA, KE_S_F25], *b"FF"),
    mod_key(SHIFT, [EXTRA, KE_S_F26], *b"FG"),
    mod_key(SHIFT, [EXTRA, KE_S_F27], *b"FH"),
    mod_key(SHIFT, [EXTRA, KE_S_F28], *b"FI"),
    mod_key(SHIFT, [EXTRA, KE_S_F29], *b"FJ"),
    mod_key(SHIFT, [EXTRA, KE_S_F30], *b"FK"),
    mod_key(SHIFT, [EXTRA, KE_S_F31], *b"FL"),
    mod_key(SHIFT, [EXTRA, KE_S_F32], *b"FM"),
    mod_key(SHIFT, [EXTRA, KE_S_F33], *b"FN"),
    mod_key(SHIFT, [EXTRA, KE_S_F34], *b"FO"),
    mod_key(SHIFT, [EXTRA, KE_S_F35], *b"FP"),
    mod_key(SHIFT, [EXTRA, KE_S_F36], *b"FQ"),
    mod_key(SHIFT, [EXTRA, KE_S_F37], *b"FR"),
    mod_key(SHIFT, *b"kB", [EXTRA, KE_TAB]),
];

/// The two termcap-name bytes of a key code, read the way C's `KEY2TERMCAP*`
/// macros read them: the low two bytes of the code's negation. Nothing checks
/// that `key` is negative — see [`simplify`].
pub(crate) fn termcap_name(key: Key) -> [u8; 2] {
    let bits = key.raw().wrapping_neg().to_le_bytes();
    [bits[0], bits[1]]
}

/// The key code a two-byte termcap name stands for (C's `TERMCAP2KEY`).
pub(crate) const fn termcap_key(name: [u8; 2]) -> Key {
    Key::from_raw(termcap(name[0], name[1]))
}

/// The single code that stands for `key` with one of `modifiers` held down,
/// and the modifiers left over — `None` when no terminal code covers it.
///
/// The comparison is on the low two bytes of `-key` rather than on the code
/// itself, exactly as the C is. A large *positive* key congruent to one of the
/// table's codes modulo 65536 therefore matches: `<S-U+8A95>` comes out as
/// `<S-Up>`. Faithful to upstream, and left that way deliberately.
pub(crate) fn simplify(key: Key, modifiers: ModMask) -> Option<(Key, ModMask)> {
    let name = termcap_name(key);
    let row = MODIFIER_KEYS
        .iter()
        .find(|m| name == m.without && modifiers.has(m.modifier))?;
    Some((termcap_key(row.with), modifiers.without(row.modifier)))
}

/// The unmodified key a shifted terminal code stands for, and the modifier it
/// carries — the reverse of [`simplify`], used when a code is printed.
pub(crate) fn unshift(key: Key) -> Option<(Key, ModMask)> {
    let name = termcap_name(key);
    let row = MODIFIER_KEYS.iter().find(|m| name == m.with)?;
    Some((termcap_key(row.without), row.modifier))
}

/// One name of one key code.
#[derive(Debug)]
struct KeyName {
    /// The key this name spells.
    key: Key,
    /// Set when the name is an alternative spelling — `Return` for `CR`, say.
    /// Exactly one row per code has it clear, and that is the name the code is
    /// printed as.
    is_alt: bool,
    /// The name itself. ASCII, matched case-insensitively.
    name: &'static str,
}

const fn key_name(key: Key, is_alt: bool, name: &'static str) -> KeyName {
    KeyName { key, is_alt, name }
}

/// [`key_name`] for a named special key.
const fn special(key: SpecialKey, is_alt: bool, name: &'static str) -> KeyName {
    key_name(Key::from_special(key), is_alt, name)
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
    special(SpecialKey::Bs, true, "BackSpace"),
    key_name(Key::ascii(b'|'), false, "Bar"),
    special(SpecialKey::Bs, false, "BS"),
    key_name(Key::ascii(b'\\'), false, "Bslash"),
    special(SpecialKey::Command, false, "Cmd"),
    key_name(Key::CAR, false, "CR"),
    key_name(Key::from_raw(CSI), false, "CSI"),
    special(SpecialKey::Del, false, "Del"),
    special(SpecialKey::Del, true, "Delete"),
    special(SpecialKey::Down, false, "Down"),
    key_name(Key::from_raw(extra(KE_DROP)), false, "Drop"),
    special(SpecialKey::End, false, "End"),
    key_name(Key::CAR, true, "Enter"),
    key_name(Key::ESC, false, "Esc"),
    key_name(Key::ESC, true, "Escape"),
    special(SpecialKey::F1, false, "F1"),
    special(SpecialKey::F10, false, "F10"),
    special(SpecialKey::F11, false, "F11"),
    special(SpecialKey::F12, false, "F12"),
    special(SpecialKey::F13, false, "F13"),
    special(SpecialKey::F14, false, "F14"),
    special(SpecialKey::F15, false, "F15"),
    special(SpecialKey::F16, false, "F16"),
    special(SpecialKey::F17, false, "F17"),
    special(SpecialKey::F18, false, "F18"),
    special(SpecialKey::F19, false, "F19"),
    special(SpecialKey::F2, false, "F2"),
    special(SpecialKey::F20, false, "F20"),
    special(SpecialKey::F21, false, "F21"),
    special(SpecialKey::F22, false, "F22"),
    special(SpecialKey::F23, false, "F23"),
    special(SpecialKey::F24, false, "F24"),
    special(SpecialKey::F25, false, "F25"),
    special(SpecialKey::F26, false, "F26"),
    special(SpecialKey::F27, false, "F27"),
    special(SpecialKey::F28, false, "F28"),
    special(SpecialKey::F29, false, "F29"),
    special(SpecialKey::F3, false, "F3"),
    special(SpecialKey::F30, false, "F30"),
    special(SpecialKey::F31, false, "F31"),
    special(SpecialKey::F32, false, "F32"),
    special(SpecialKey::F33, false, "F33"),
    special(SpecialKey::F34, false, "F34"),
    special(SpecialKey::F35, false, "F35"),
    special(SpecialKey::F36, false, "F36"),
    special(SpecialKey::F37, false, "F37"),
    special(SpecialKey::F38, false, "F38"),
    special(SpecialKey::F39, false, "F39"),
    special(SpecialKey::F4, false, "F4"),
    special(SpecialKey::F40, false, "F40"),
    special(SpecialKey::F41, false, "F41"),
    special(SpecialKey::F42, false, "F42"),
    special(SpecialKey::F43, false, "F43"),
    special(SpecialKey::F44, false, "F44"),
    special(SpecialKey::F45, false, "F45"),
    special(SpecialKey::F46, false, "F46"),
    special(SpecialKey::F47, false, "F47"),
    special(SpecialKey::F48, false, "F48"),
    special(SpecialKey::F49, false, "F49"),
    special(SpecialKey::F5, false, "F5"),
    special(SpecialKey::F50, false, "F50"),
    special(SpecialKey::F51, false, "F51"),
    special(SpecialKey::F52, false, "F52"),
    special(SpecialKey::F53, false, "F53"),
    special(SpecialKey::F54, false, "F54"),
    special(SpecialKey::F55, false, "F55"),
    special(SpecialKey::F56, false, "F56"),
    special(SpecialKey::F57, false, "F57"),
    special(SpecialKey::F58, false, "F58"),
    special(SpecialKey::F59, false, "F59"),
    special(SpecialKey::F6, false, "F6"),
    special(SpecialKey::F60, false, "F60"),
    special(SpecialKey::F61, false, "F61"),
    special(SpecialKey::F62, false, "F62"),
    special(SpecialKey::F63, false, "F63"),
    special(SpecialKey::F7, false, "F7"),
    special(SpecialKey::F8, false, "F8"),
    special(SpecialKey::F9, false, "F9"),
    special(SpecialKey::Find, false, "Find"),
    special(SpecialKey::Help, false, "Help"),
    special(SpecialKey::Home, false, "Home"),
    special(SpecialKey::Ignore, false, "Ignore"),
    special(SpecialKey::Ins, true, "Ins"),
    special(SpecialKey::Ins, false, "Insert"),
    special(SpecialKey::K0, false, "k0"),
    special(SpecialKey::K1, false, "k1"),
    special(SpecialKey::K2, false, "k2"),
    special(SpecialKey::K3, false, "k3"),
    special(SpecialKey::K4, false, "k4"),
    special(SpecialKey::K5, false, "k5"),
    special(SpecialKey::K6, false, "k6"),
    special(SpecialKey::K7, false, "k7"),
    special(SpecialKey::K8, false, "k8"),
    special(SpecialKey::K9, false, "k9"),
    special(SpecialKey::Kcomma, false, "kComma"),
    special(SpecialKey::Kdel, false, "kDel"),
    special(SpecialKey::Kdivide, false, "kDivide"),
    special(SpecialKey::Kdown, false, "kDown"),
    special(SpecialKey::Kend, false, "kEnd"),
    special(SpecialKey::Kenter, false, "kEnter"),
    special(SpecialKey::Kequal, false, "kEqual"),
    special(SpecialKey::Khome, false, "kHome"),
    special(SpecialKey::Kins, false, "kInsert"),
    special(SpecialKey::Kleft, false, "kLeft"),
    special(SpecialKey::Kminus, false, "kMinus"),
    special(SpecialKey::Kmultiply, false, "kMultiply"),
    special(SpecialKey::Korigin, false, "kOrigin"),
    special(SpecialKey::Kins, true, "KP0"),
    special(SpecialKey::Kend, true, "KP1"),
    special(SpecialKey::Kdown, true, "KP2"),
    special(SpecialKey::Kpagedown, true, "KP3"),
    special(SpecialKey::Kleft, true, "KP4"),
    special(SpecialKey::Korigin, true, "KP5"),
    special(SpecialKey::Kright, true, "KP6"),
    special(SpecialKey::Khome, true, "KP7"),
    special(SpecialKey::Kup, true, "KP8"),
    special(SpecialKey::Kpageup, true, "KP9"),
    special(SpecialKey::Kpagedown, false, "kPageDown"),
    special(SpecialKey::Kpageup, false, "kPageUp"),
    special(SpecialKey::Kcomma, true, "KPComma"),
    special(SpecialKey::Kdivide, true, "KPDiv"),
    special(SpecialKey::Kenter, true, "KPEnter"),
    special(SpecialKey::Kequal, true, "KPEquals"),
    special(SpecialKey::Kplus, false, "kPlus"),
    special(SpecialKey::Kminus, true, "KPMinus"),
    special(SpecialKey::Kmultiply, true, "KPMult"),
    special(SpecialKey::Kpoint, false, "kPoint"),
    special(SpecialKey::Kdel, true, "KPPeriod"),
    special(SpecialKey::Kplus, true, "KPPlus"),
    special(SpecialKey::Kright, false, "kRight"),
    special(SpecialKey::Kup, false, "kUp"),
    special(SpecialKey::Left, false, "Left"),
    special(SpecialKey::Leftdrag, false, "LeftDrag"),
    special(SpecialKey::Leftmouse, false, "LeftMouse"),
    special(SpecialKey::LeftmouseNm, false, "LeftMouseNM"),
    special(SpecialKey::Leftrelease, false, "LeftRelease"),
    special(SpecialKey::LeftreleaseNm, false, "LeftReleaseNM"),
    key_name(Key::NL, true, "LF"),
    key_name(Key::NL, true, "LineFeed"),
    key_name(Key::ascii(b'<'), false, "lt"),
    special(SpecialKey::Middledrag, false, "MiddleDrag"),
    special(SpecialKey::Middlemouse, false, "MiddleMouse"),
    special(SpecialKey::Middlerelease, false, "MiddleRelease"),
    special(SpecialKey::Mouse, false, "Mouse"),
    special(SpecialKey::Mousedown, true, "MouseDown"),
    special(SpecialKey::Mousemove, false, "MouseMove"),
    special(SpecialKey::Mouseup, true, "MouseUp"),
    key_name(Key::NL, true, "NewLine"),
    key_name(Key::NL, false, "NL"),
    special(SpecialKey::Zero, false, "Nul"),
    special(SpecialKey::Pagedown, false, "PageDown"),
    special(SpecialKey::Pageup, false, "PageUp"),
    key_name(Key::from_raw(extra(KE_PLUG)), false, "Plug"),
    key_name(Key::CAR, true, "Return"),
    special(SpecialKey::Right, false, "Right"),
    special(SpecialKey::Rightdrag, false, "RightDrag"),
    special(SpecialKey::Rightmouse, false, "RightMouse"),
    special(SpecialKey::Rightrelease, false, "RightRelease"),
    special(SpecialKey::Mouseup, false, "ScrollWheelDown"),
    special(SpecialKey::Mouseright, false, "ScrollWheelLeft"),
    special(SpecialKey::Mouseleft, false, "ScrollWheelRight"),
    special(SpecialKey::Mousedown, false, "ScrollWheelUp"),
    special(SpecialKey::Kselect, false, "Select"),
    key_name(Key::from_raw(extra(KE_SNR)), false, "SNR"),
    key_name(Key::ascii(b' '), false, "Space"),
    key_name(Key::TAB, false, "Tab"),
    key_name(Key::from_raw(extra(KE_TAB)), false, "Tab"),
    special(SpecialKey::Undo, false, "Undo"),
    special(SpecialKey::Up, false, "Up"),
    special(SpecialKey::X1drag, false, "X1Drag"),
    special(SpecialKey::X1mouse, false, "X1Mouse"),
    special(SpecialKey::X1release, false, "X1Release"),
    special(SpecialKey::X2drag, false, "X2Drag"),
    special(SpecialKey::X2mouse, false, "X2Mouse"),
    special(SpecialKey::X2release, false, "X2Release"),
    special(SpecialKey::Xdown, false, "xDown"),
    special(SpecialKey::Xend, false, "xEnd"),
    special(SpecialKey::Xf1, false, "xF1"),
    key_name(Key::from_raw(extra(KE_XF2)), false, "xF2"),
    key_name(Key::from_raw(extra(KE_XF3)), false, "xF3"),
    key_name(Key::from_raw(extra(KE_XF4)), false, "xF4"),
    special(SpecialKey::Xhome, false, "xHome"),
    special(SpecialKey::Xleft, false, "xLeft"),
    special(SpecialKey::Xright, false, "xRight"),
    special(SpecialKey::Xup, false, "xUp"),
    special(SpecialKey::Zend, false, "zEnd"),
    special(SpecialKey::Zhome, false, "zHome"),
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
pub(crate) fn code_for_name(name: &[u8]) -> Key {
    let at = KEY_NAMES.partition_point(|e| before_ignore_case(e.name.as_bytes(), name));
    match KEY_NAMES.get(at) {
        Some(e) if e.name.as_bytes().eq_ignore_ascii_case(name) => e.key,
        _ => Key::NUL,
    }
}

/// The name `key` is printed as: the one row for `key` that is not an
/// alternative spelling.
pub(crate) fn name_of_code(key: Key) -> Option<&'static str> {
    KEY_NAMES
        .iter()
        .find(|e| e.key == key && !e.is_alt)
        .map(|e| e.name)
}

/// Whether the table has a name for `key`.
pub fn has_key_name(key: Key) -> bool {
    name_of_code(key).is_some()
}

/// What a mouse pseudo-code means.
#[derive(Debug)]
pub(crate) struct MouseEvent {
    /// Which button, as a `MOUSE_*` code.
    pub button: c_int,
    /// A button going down.
    pub is_click: bool,
    /// The mouse moving with a button held.
    pub is_drag: bool,
}

/// A mouse pseudo-code and what it means.
#[derive(Debug)]
struct MouseCode {
    code: KeyExtra,
    event: MouseEvent,
}

const fn mouse_code(code: KeyExtra, button: c_int, is_click: bool, is_drag: bool) -> MouseCode {
    MouseCode {
        code,
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

/// What the `KE_*` byte `code` means as a mouse event, or `None` when it is
/// not one.
pub(crate) fn mouse_event(code: KeyExtra) -> Option<&'static MouseEvent> {
    MOUSE_CODES
        .iter()
        .find(|m| m.code == code)
        .map(|m| &m.event)
}
