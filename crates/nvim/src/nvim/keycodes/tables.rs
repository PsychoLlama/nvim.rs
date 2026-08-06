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

use core::ffi::c_int;
use core::iter;

use super::*;
use crate::src::nvim::keycodes::{
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
struct ModMask {
    /// The bits of the modifier state this row is about.
    mask: c_int,
    /// The value those bits must have for the row to apply. Only the
    /// multi-click rows differ from `mask`: they share one two-bit field.
    flag: c_int,
    /// The letter that spells the modifier in `<X-Key>`.
    name: u8,
}

const fn mod_mask(mask: c_int, flag: c_int, name: u8) -> ModMask {
    ModMask { mask, flag, name }
}

/// The modifiers that are spelled out when a key is printed, in the order
/// they are printed: `<M-C-S-Up>`, never `<S-C-M-Up>`.
static PRINTED_MOD_MASKS: [ModMask; 8] = [
    mod_mask(MOD_MASK_ALT, MOD_MASK_ALT, b'M'),
    mod_mask(MOD_MASK_META, MOD_MASK_META, b'T'),
    mod_mask(MOD_MASK_CTRL, MOD_MASK_CTRL, b'C'),
    mod_mask(MOD_MASK_SHIFT, MOD_MASK_SHIFT, b'S'),
    mod_mask(MOD_MASK_MULTI_CLICK, MOD_MASK_2CLICK, b'2'),
    mod_mask(MOD_MASK_MULTI_CLICK, MOD_MASK_3CLICK, b'3'),
    mod_mask(MOD_MASK_MULTI_CLICK, MOD_MASK_4CLICK, b'4'),
    mod_mask(MOD_MASK_CMD, MOD_MASK_CMD, b'D'),
];

/// `A` is an alternative spelling of `M`, accepted where a key is written but
/// never printed — which is why it is not one of [`PRINTED_MOD_MASKS`].
static ALT_MOD_MASK: ModMask = mod_mask(MOD_MASK_ALT, MOD_MASK_ALT, b'A');

/// The modifier bit a `<X-Key>` prefix letter names — `S` for shift, `C` for
/// ctrl — or 0 when it names no modifier. ASCII case is folded.
pub fn name_to_mod_mask(c: c_int) -> c_int {
    let c = to_upper_ascii(c);
    PRINTED_MOD_MASKS
        .iter()
        .chain(iter::once(&ALT_MOD_MASK))
        .find(|m| c == c_int::from(m.name))
        .map_or(0, |m| m.flag)
}

/// The modifier prefixes `modifiers` calls for, in printing order.
pub(crate) fn printed_modifiers(modifiers: c_int) -> impl Iterator<Item = u8> {
    PRINTED_MOD_MASKS
        .iter()
        .filter(move |m| modifiers & m.mask == m.flag)
        .map(|m| m.name)
}

/// A terminal code that already carries a modifier, and the code for the same
/// key without it. Mouse codes are deliberately not in here.
struct ModifierKey {
    /// The modifier bit the shifted code carries.
    modifier: u8,
    /// Termcap name of the key *with* the modifier.
    with: [u8; 2],
    /// Termcap name of the key *without* it.
    without: [u8; 2],
}

const fn mod_key(modifier: u8, with: [u8; 2], without: [u8; 2]) -> ModifierKey {
    ModifierKey {
        modifier,
        with,
        without,
    }
}

/// The modifier bits and the `KS_EXTRA` marker, narrowed to the byte type
/// [`MODIFIER_KEYS`] stores. Upstream's table is a flat `uint8_t[]`.
const SHIFT: u8 = MOD_MASK_SHIFT as u8;
const CTRL: u8 = MOD_MASK_CTRL as u8;
const EXTRA: u8 = KS_EXTRA as u8;

/// Shifted and ctrl'ed terminal codes, and the unmodified code each stands
/// for. Terminals send `<S-Up>` as a code of its own rather than as `Up` plus
/// a modifier byte, and this is the table that relates the two.
static MODIFIER_KEYS: [ModifierKey; 75] = [
    mod_key(SHIFT, [b'&', b'9'], [b'@', b'1']),
    mod_key(SHIFT, [b'&', b'0'], [b'@', b'2']),
    mod_key(SHIFT, [b'*', b'1'], [b'@', b'4']),
    mod_key(SHIFT, [b'*', b'2'], [b'@', b'5']),
    mod_key(SHIFT, [b'*', b'3'], [b'@', b'6']),
    mod_key(SHIFT, [b'*', b'4'], [b'k', b'D']),
    mod_key(SHIFT, [b'*', b'5'], [b'k', b'L']),
    mod_key(SHIFT, [b'*', b'7'], [b'@', b'7']),
    mod_key(CTRL, [EXTRA, KE_C_END as u8], [b'@', b'7']),
    mod_key(SHIFT, [b'*', b'9'], [b'@', b'9']),
    mod_key(SHIFT, [b'*', b'0'], [b'@', b'0']),
    mod_key(SHIFT, [b'#', b'1'], [b'%', b'1']),
    mod_key(SHIFT, [b'#', b'2'], [b'k', b'h']),
    mod_key(CTRL, [EXTRA, KE_C_HOME as u8], [b'k', b'h']),
    mod_key(SHIFT, [b'#', b'3'], [b'k', b'I']),
    mod_key(SHIFT, [b'#', b'4'], [b'k', b'l']),
    mod_key(CTRL, [EXTRA, KE_C_LEFT as u8], [b'k', b'l']),
    mod_key(SHIFT, [b'%', b'a'], [b'%', b'3']),
    mod_key(SHIFT, [b'%', b'b'], [b'%', b'4']),
    mod_key(SHIFT, [b'%', b'c'], [b'%', b'5']),
    mod_key(SHIFT, [b'%', b'd'], [b'%', b'7']),
    mod_key(SHIFT, [b'%', b'e'], [b'%', b'8']),
    mod_key(SHIFT, [b'%', b'f'], [b'%', b'9']),
    mod_key(SHIFT, [b'%', b'g'], [b'%', b'0']),
    mod_key(SHIFT, [b'%', b'h'], [b'&', b'3']),
    mod_key(SHIFT, [b'%', b'i'], [b'k', b'r']),
    mod_key(CTRL, [EXTRA, KE_C_RIGHT as u8], [b'k', b'r']),
    mod_key(SHIFT, [b'%', b'j'], [b'&', b'5']),
    mod_key(SHIFT, [b'!', b'1'], [b'&', b'6']),
    mod_key(SHIFT, [b'!', b'2'], [b'&', b'7']),
    mod_key(SHIFT, [b'!', b'3'], [b'&', b'8']),
    mod_key(SHIFT, [EXTRA, KE_S_UP as u8], [b'k', b'u']),
    mod_key(SHIFT, [EXTRA, KE_S_DOWN as u8], [b'k', b'd']),
    mod_key(SHIFT, [EXTRA, KE_S_XF1 as u8], [EXTRA, KE_XF1 as u8]),
    mod_key(SHIFT, [EXTRA, KE_S_XF2 as u8], [EXTRA, KE_XF2 as u8]),
    mod_key(SHIFT, [EXTRA, KE_S_XF3 as u8], [EXTRA, KE_XF3 as u8]),
    mod_key(SHIFT, [EXTRA, KE_S_XF4 as u8], [EXTRA, KE_XF4 as u8]),
    mod_key(SHIFT, [EXTRA, KE_S_F1 as u8], [b'k', b'1']),
    mod_key(SHIFT, [EXTRA, KE_S_F2 as u8], [b'k', b'2']),
    mod_key(SHIFT, [EXTRA, KE_S_F3 as u8], [b'k', b'3']),
    mod_key(SHIFT, [EXTRA, KE_S_F4 as u8], [b'k', b'4']),
    mod_key(SHIFT, [EXTRA, KE_S_F5 as u8], [b'k', b'5']),
    mod_key(SHIFT, [EXTRA, KE_S_F6 as u8], [b'k', b'6']),
    mod_key(SHIFT, [EXTRA, KE_S_F7 as u8], [b'k', b'7']),
    mod_key(SHIFT, [EXTRA, KE_S_F8 as u8], [b'k', b'8']),
    mod_key(SHIFT, [EXTRA, KE_S_F9 as u8], [b'k', b'9']),
    mod_key(SHIFT, [EXTRA, KE_S_F10 as u8], [b'k', b';']),
    mod_key(SHIFT, [EXTRA, KE_S_F11 as u8], [b'F', b'1']),
    mod_key(SHIFT, [EXTRA, KE_S_F12 as u8], [b'F', b'2']),
    mod_key(SHIFT, [EXTRA, KE_S_F13 as u8], [b'F', b'3']),
    mod_key(SHIFT, [EXTRA, KE_S_F14 as u8], [b'F', b'4']),
    mod_key(SHIFT, [EXTRA, KE_S_F15 as u8], [b'F', b'5']),
    mod_key(SHIFT, [EXTRA, KE_S_F16 as u8], [b'F', b'6']),
    mod_key(SHIFT, [EXTRA, KE_S_F17 as u8], [b'F', b'7']),
    mod_key(SHIFT, [EXTRA, KE_S_F18 as u8], [b'F', b'8']),
    mod_key(SHIFT, [EXTRA, KE_S_F19 as u8], [b'F', b'9']),
    mod_key(SHIFT, [EXTRA, KE_S_F20 as u8], [b'F', b'A']),
    mod_key(SHIFT, [EXTRA, KE_S_F21 as u8], [b'F', b'B']),
    mod_key(SHIFT, [EXTRA, KE_S_F22 as u8], [b'F', b'C']),
    mod_key(SHIFT, [EXTRA, KE_S_F23 as u8], [b'F', b'D']),
    mod_key(SHIFT, [EXTRA, KE_S_F24 as u8], [b'F', b'E']),
    mod_key(SHIFT, [EXTRA, KE_S_F25 as u8], [b'F', b'F']),
    mod_key(SHIFT, [EXTRA, KE_S_F26 as u8], [b'F', b'G']),
    mod_key(SHIFT, [EXTRA, KE_S_F27 as u8], [b'F', b'H']),
    mod_key(SHIFT, [EXTRA, KE_S_F28 as u8], [b'F', b'I']),
    mod_key(SHIFT, [EXTRA, KE_S_F29 as u8], [b'F', b'J']),
    mod_key(SHIFT, [EXTRA, KE_S_F30 as u8], [b'F', b'K']),
    mod_key(SHIFT, [EXTRA, KE_S_F31 as u8], [b'F', b'L']),
    mod_key(SHIFT, [EXTRA, KE_S_F32 as u8], [b'F', b'M']),
    mod_key(SHIFT, [EXTRA, KE_S_F33 as u8], [b'F', b'N']),
    mod_key(SHIFT, [EXTRA, KE_S_F34 as u8], [b'F', b'O']),
    mod_key(SHIFT, [EXTRA, KE_S_F35 as u8], [b'F', b'P']),
    mod_key(SHIFT, [EXTRA, KE_S_F36 as u8], [b'F', b'Q']),
    mod_key(SHIFT, [EXTRA, KE_S_F37 as u8], [b'F', b'R']),
    mod_key(SHIFT, [b'k', b'B'], [EXTRA, KE_TAB as u8]),
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
pub(crate) fn simplify(key: c_int, modifiers: c_int) -> Option<(c_int, c_int)> {
    let name = termcap_name(key);
    let row = MODIFIER_KEYS
        .iter()
        .find(|m| name == m.without && modifiers & c_int::from(m.modifier) != 0)?;
    Some((
        termcap_key(row.with),
        modifiers & !c_int::from(row.modifier),
    ))
}

/// The unmodified key a shifted terminal code stands for, and the modifier it
/// carries — the reverse of [`simplify`], used when a code is printed.
pub(crate) fn unshift(key: c_int) -> Option<(c_int, c_int)> {
    let name = termcap_name(key);
    let row = MODIFIER_KEYS.iter().find(|m| name == m.with)?;
    Some((termcap_key(row.without), c_int::from(row.modifier)))
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
    key_name(K_BS, true, "BackSpace"),
    key_name('|' as c_int, false, "Bar"),
    key_name(K_BS, false, "BS"),
    key_name('\\' as c_int, false, "Bslash"),
    key_name(K_COMMAND, false, "Cmd"),
    key_name(CAR, false, "CR"),
    key_name(CSI, false, "CSI"),
    key_name(K_DEL, false, "Del"),
    key_name(K_DEL, true, "Delete"),
    key_name(K_DOWN, false, "Down"),
    key_name(extra(KE_DROP), false, "Drop"),
    key_name(K_END, false, "End"),
    key_name(CAR, true, "Enter"),
    key_name(ESC, false, "Esc"),
    key_name(ESC, true, "Escape"),
    key_name(K_F1, false, "F1"),
    key_name(K_F10, false, "F10"),
    key_name(K_F11, false, "F11"),
    key_name(K_F12, false, "F12"),
    key_name(K_F13, false, "F13"),
    key_name(K_F14, false, "F14"),
    key_name(K_F15, false, "F15"),
    key_name(K_F16, false, "F16"),
    key_name(K_F17, false, "F17"),
    key_name(K_F18, false, "F18"),
    key_name(K_F19, false, "F19"),
    key_name(K_F2, false, "F2"),
    key_name(K_F20, false, "F20"),
    key_name(K_F21, false, "F21"),
    key_name(K_F22, false, "F22"),
    key_name(K_F23, false, "F23"),
    key_name(K_F24, false, "F24"),
    key_name(K_F25, false, "F25"),
    key_name(K_F26, false, "F26"),
    key_name(K_F27, false, "F27"),
    key_name(K_F28, false, "F28"),
    key_name(K_F29, false, "F29"),
    key_name(K_F3, false, "F3"),
    key_name(K_F30, false, "F30"),
    key_name(K_F31, false, "F31"),
    key_name(K_F32, false, "F32"),
    key_name(K_F33, false, "F33"),
    key_name(K_F34, false, "F34"),
    key_name(K_F35, false, "F35"),
    key_name(K_F36, false, "F36"),
    key_name(K_F37, false, "F37"),
    key_name(K_F38, false, "F38"),
    key_name(K_F39, false, "F39"),
    key_name(K_F4, false, "F4"),
    key_name(K_F40, false, "F40"),
    key_name(K_F41, false, "F41"),
    key_name(K_F42, false, "F42"),
    key_name(K_F43, false, "F43"),
    key_name(K_F44, false, "F44"),
    key_name(K_F45, false, "F45"),
    key_name(K_F46, false, "F46"),
    key_name(K_F47, false, "F47"),
    key_name(K_F48, false, "F48"),
    key_name(K_F49, false, "F49"),
    key_name(K_F5, false, "F5"),
    key_name(K_F50, false, "F50"),
    key_name(K_F51, false, "F51"),
    key_name(K_F52, false, "F52"),
    key_name(K_F53, false, "F53"),
    key_name(K_F54, false, "F54"),
    key_name(K_F55, false, "F55"),
    key_name(K_F56, false, "F56"),
    key_name(K_F57, false, "F57"),
    key_name(K_F58, false, "F58"),
    key_name(K_F59, false, "F59"),
    key_name(K_F6, false, "F6"),
    key_name(K_F60, false, "F60"),
    key_name(K_F61, false, "F61"),
    key_name(K_F62, false, "F62"),
    key_name(K_F63, false, "F63"),
    key_name(K_F7, false, "F7"),
    key_name(K_F8, false, "F8"),
    key_name(K_F9, false, "F9"),
    key_name(K_FIND, false, "Find"),
    key_name(K_HELP, false, "Help"),
    key_name(K_HOME, false, "Home"),
    key_name(K_IGNORE, false, "Ignore"),
    key_name(K_INS, true, "Ins"),
    key_name(K_INS, false, "Insert"),
    key_name(K_K0, false, "k0"),
    key_name(K_K1, false, "k1"),
    key_name(K_K2, false, "k2"),
    key_name(K_K3, false, "k3"),
    key_name(K_K4, false, "k4"),
    key_name(K_K5, false, "k5"),
    key_name(K_K6, false, "k6"),
    key_name(K_K7, false, "k7"),
    key_name(K_K8, false, "k8"),
    key_name(K_K9, false, "k9"),
    key_name(K_KCOMMA, false, "kComma"),
    key_name(K_KDEL, false, "kDel"),
    key_name(K_KDIVIDE, false, "kDivide"),
    key_name(K_KDOWN, false, "kDown"),
    key_name(K_KEND, false, "kEnd"),
    key_name(K_KENTER, false, "kEnter"),
    key_name(K_KEQUAL, false, "kEqual"),
    key_name(K_KHOME, false, "kHome"),
    key_name(K_KINS, false, "kInsert"),
    key_name(K_KLEFT, false, "kLeft"),
    key_name(K_KMINUS, false, "kMinus"),
    key_name(K_KMULTIPLY, false, "kMultiply"),
    key_name(K_KORIGIN, false, "kOrigin"),
    key_name(K_KINS, true, "KP0"),
    key_name(K_KEND, true, "KP1"),
    key_name(K_KDOWN, true, "KP2"),
    key_name(K_KPAGEDOWN, true, "KP3"),
    key_name(K_KLEFT, true, "KP4"),
    key_name(K_KORIGIN, true, "KP5"),
    key_name(K_KRIGHT, true, "KP6"),
    key_name(K_KHOME, true, "KP7"),
    key_name(K_KUP, true, "KP8"),
    key_name(K_KPAGEUP, true, "KP9"),
    key_name(K_KPAGEDOWN, false, "kPageDown"),
    key_name(K_KPAGEUP, false, "kPageUp"),
    key_name(K_KCOMMA, true, "KPComma"),
    key_name(K_KDIVIDE, true, "KPDiv"),
    key_name(K_KENTER, true, "KPEnter"),
    key_name(K_KEQUAL, true, "KPEquals"),
    key_name(K_KPLUS, false, "kPlus"),
    key_name(K_KMINUS, true, "KPMinus"),
    key_name(K_KMULTIPLY, true, "KPMult"),
    key_name(K_KPOINT, false, "kPoint"),
    key_name(K_KDEL, true, "KPPeriod"),
    key_name(K_KPLUS, true, "KPPlus"),
    key_name(K_KRIGHT, false, "kRight"),
    key_name(K_KUP, false, "kUp"),
    key_name(K_LEFT, false, "Left"),
    key_name(K_LEFTDRAG, false, "LeftDrag"),
    key_name(K_LEFTMOUSE, false, "LeftMouse"),
    key_name(K_LEFTMOUSE_NM, false, "LeftMouseNM"),
    key_name(K_LEFTRELEASE, false, "LeftRelease"),
    key_name(K_LEFTRELEASE_NM, false, "LeftReleaseNM"),
    key_name(NL, true, "LF"),
    key_name(NL, true, "LineFeed"),
    key_name('<' as c_int, false, "lt"),
    key_name(K_MIDDLEDRAG, false, "MiddleDrag"),
    key_name(K_MIDDLEMOUSE, false, "MiddleMouse"),
    key_name(K_MIDDLERELEASE, false, "MiddleRelease"),
    key_name(K_MOUSE, false, "Mouse"),
    key_name(K_MOUSEDOWN, true, "MouseDown"),
    key_name(K_MOUSEMOVE, false, "MouseMove"),
    key_name(K_MOUSEUP, true, "MouseUp"),
    key_name(NL, true, "NewLine"),
    key_name(NL, false, "NL"),
    key_name(K_ZERO, false, "Nul"),
    key_name(K_PAGEDOWN, false, "PageDown"),
    key_name(K_PAGEUP, false, "PageUp"),
    key_name(extra(KE_PLUG), false, "Plug"),
    key_name(CAR, true, "Return"),
    key_name(K_RIGHT, false, "Right"),
    key_name(K_RIGHTDRAG, false, "RightDrag"),
    key_name(K_RIGHTMOUSE, false, "RightMouse"),
    key_name(K_RIGHTRELEASE, false, "RightRelease"),
    key_name(K_MOUSEUP, false, "ScrollWheelDown"),
    key_name(K_MOUSERIGHT, false, "ScrollWheelLeft"),
    key_name(K_MOUSELEFT, false, "ScrollWheelRight"),
    key_name(K_MOUSEDOWN, false, "ScrollWheelUp"),
    key_name(K_KSELECT, false, "Select"),
    key_name(extra(KE_SNR), false, "SNR"),
    key_name(' ' as c_int, false, "Space"),
    key_name(TAB, false, "Tab"),
    key_name(extra(KE_TAB), false, "Tab"),
    key_name(K_UNDO, false, "Undo"),
    key_name(K_UP, false, "Up"),
    key_name(K_X1DRAG, false, "X1Drag"),
    key_name(K_X1MOUSE, false, "X1Mouse"),
    key_name(K_X1RELEASE, false, "X1Release"),
    key_name(K_X2DRAG, false, "X2Drag"),
    key_name(K_X2MOUSE, false, "X2Mouse"),
    key_name(K_X2RELEASE, false, "X2Release"),
    key_name(K_XDOWN, false, "xDown"),
    key_name(K_XEND, false, "xEnd"),
    key_name(K_XF1, false, "xF1"),
    key_name(extra(KE_XF2), false, "xF2"),
    key_name(extra(KE_XF3), false, "xF3"),
    key_name(extra(KE_XF4), false, "xF4"),
    key_name(K_XHOME, false, "xHome"),
    key_name(K_XLEFT, false, "xLeft"),
    key_name(K_XRIGHT, false, "xRight"),
    key_name(K_XUP, false, "xUp"),
    key_name(K_ZEND, false, "zEnd"),
    key_name(K_ZHOME, false, "zHome"),
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
