//! What the terminal is currently wearing.
//!
//! The editor paints in highlight ids; the terminal understands escape
//! sequences. This module holds the table those ids index, and turns a
//! change of id into the smallest set of sequences that gets the terminal
//! from what it is wearing to what the next cell wants.
//!
//! Two things make that more than a lookup. Terminals disagree about how to
//! say the same thing — `sgr` sets six attributes at once where a poorer
//! terminfo has to say each in turn — and turning an attribute *off*
//! generally means resetting everything and rebuilding, which is why
//! [`update_attrs`] emits a full description rather than a diff.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight::{
    HL_ALTFONT, HL_BG_INDEXED, HL_BLINK, HL_BOLD, HL_CONCEALED, HL_DIM, HL_FG_INDEXED, HL_INVERSE,
    HL_ITALIC, HL_OVERLINE, HL_STANDOUT, HL_STRIKETHROUGH, HL_UNDERCURL, HL_UNDERDASHED,
    HL_UNDERDOTTED, HL_UNDERDOUBLE, HL_UNDERLINE, HL_UNDERLINE_MASK,
};
use crate::src::nvim::map::mh_put_cstr_t;
use crate::src::nvim::memory::xstrdup;
use crate::src::nvim::tui::output::{out, out_cstr, out_fmt, terminfo_out, terminfo_print_nums};
use crate::src::nvim::tui::paint::invalidate;
use crate::src::nvim::tui::terminfo::caps::{
    kTerm_enter_blink_mode, kTerm_enter_bold_mode, kTerm_enter_dim_mode, kTerm_enter_italics_mode,
    kTerm_enter_reverse_mode, kTerm_enter_secure_mode, kTerm_enter_standout_mode,
    kTerm_enter_strikethrough_mode, kTerm_enter_underline_mode, kTerm_exit_attribute_mode,
    kTerm_set_a_background, kTerm_set_a_foreground, kTerm_set_attributes, kTerm_set_rgb_background,
    kTerm_set_rgb_foreground, kTerm_set_underline_style,
};
use crate::src::nvim::tui::tui::DEFAULT_ATTRS;
use crate::src::nvim::types::{
    Array, HlAttrs, Integer, MHPutStatus, Set_cstr_t, TUIData, cstr_t, int32_t,
};
use core::ffi::{CStr, c_char, c_int};

/// The underline style parameter each `HL_UNDER*` bit asks the terminal for,
/// in the order the sequences are emitted.
const UNDERLINE_STYLES: [(c_int, c_int); 4] = [
    (HL_UNDERCURL, 3),
    (HL_UNDERDOUBLE, 2),
    (HL_UNDERDOTTED, 4),
    (HL_UNDERDASHED, 5),
];

/// The id base OSC 8 hyperlinks are numbered from, so that two runs of the
/// same link are recognised as one link by the terminal.
const URL_ID_BASE: u32 = 0xe1ea_0000;

/// What ends an OSC 8 sequence (a string terminator).
const OSC8_TERMINATOR: &[u8] = b"\x1b\\";

/// Every URL the editor has attached to a highlight, interned so an
/// attribute can name one by index. The copies are never freed: an
/// attribute may name one at any time until the TUI exits.
static URLS: GlobalCell<Set_cstr_t> = GlobalCell::new(Set_cstr_t::EMPTY);

/// `mh_put`'s "the key was already there" status.
const KEY_EXISTING: MHPutStatus = 0;

/// Intern `url` and return the index highlights name it by, or -1 for none.
///
/// # Safety
/// `url` must be null or NUL-terminated.
pub unsafe fn tui_add_url(_tui: &mut TUIData, url: *const c_char) -> int32_t {
    if url.is_null() {
        return -1;
    }
    // SAFETY: `url` is NUL-terminated, and the set is only ever touched from
    // the TUI's thread.
    unsafe {
        let mut status: MHPutStatus = KEY_EXISTING;
        let key = mh_put_cstr_t(URLS.ptr(), url as cstr_t, &raw mut status);
        // A new key borrows the caller's string; give the set its own copy.
        if status != KEY_EXISTING {
            *(*URLS.ptr()).keys.add(key as usize) = xstrdup(url) as cstr_t;
        }
        key as int32_t
    }
}

/// Record highlight `id`'s definition.
///
/// The editor sends the RGB and cterm descriptions of one highlight as two
/// structures; the TUI keeps a single merged entry and picks a half at paint
/// time depending on `'termguicolors'`.
///
pub fn tui_hl_attr_define(
    tui: &mut TUIData,
    id: Integer,
    mut attrs: HlAttrs,
    cterm_attrs: HlAttrs,
    _info: Array,
) {
    attrs.cterm_ae_attr = cterm_attrs.cterm_ae_attr;
    attrs.cterm_fg_color = cterm_attrs.cterm_fg_color;
    attrs.cterm_bg_color = cterm_attrs.cterm_bg_color;
    let id = id as usize;
    reserve_attr(tui, id);
    tui.attrs[id] = attrs;
}

/// Make `id` a valid index into the attribute table, growing it if needed.
///
/// Ids the editor skipped over on the way to this one are the default
/// highlight: the table is indexed by id, so they have to be something, and
/// painting under an undefined id is a bug the editor is not to be crashed
/// for.
pub(crate) fn reserve_attr(tui: &mut TUIData, id: usize) {
    if tui.attrs.len() <= id {
        tui.attrs.resize(id + 1, DEFAULT_ATTRS);
    }
}

/// Highlight `id`'s definition.
pub(crate) fn attr(tui: &TUIData, id: c_int) -> HlAttrs {
    assert!(
        (id as usize) < tui.attrs.len(),
        "attribute {id} not defined"
    );
    tui.attrs[id as usize]
}

/// Would painting under `id1` look any different from painting under `id2`?
///
/// Ids that the terminal cannot tell apart need no sequences between them.
/// Which halves of the definition count depends on `'termguicolors'`, except
/// for the underline colour, which cterm terminals also honour when there is
/// an underline to colour.
pub(crate) fn attrs_differ(tui: &TUIData, id1: c_int, id2: c_int, rgb: bool) -> bool {
    if id1 == id2 {
        return false;
    }
    // A negative id is "unknown", which never matches anything.
    if id1 < 0 || id2 < 0 {
        return true;
    }
    let (a1, a2) = (attr(tui, id1), attr(tui, id2));
    if a1.url != a2.url {
        return true;
    }
    if rgb {
        a1.rgb_fg_color != a2.rgb_fg_color
            || a1.rgb_bg_color != a2.rgb_bg_color
            || a1.rgb_ae_attr != a2.rgb_ae_attr
            || a1.rgb_sp_color != a2.rgb_sp_color
    } else {
        a1.cterm_fg_color != a2.cterm_fg_color
            || a1.cterm_bg_color != a2.cterm_bg_color
            || a1.cterm_ae_attr != a2.cterm_ae_attr
            || (a1.cterm_ae_attr & HL_UNDERLINE_MASK != 0 && a1.rgb_sp_color != a2.rgb_sp_color)
    }
}

/// The attribute bits `attr_id` asks for, from the half of its definition
/// this terminal is being painted in.
fn ae_attr(tui: &TUIData, attrs: HlAttrs) -> c_int {
    if tui.rgb {
        attrs.rgb_ae_attr
    } else {
        attrs.cterm_ae_attr
    }
}

/// Dress the terminal for `attr_id`, if it is not already dressed for it.
pub(crate) fn update_attrs(tui: &mut TUIData, attr_id: c_int) {
    if !attrs_differ(tui, attr_id, tui.print_attr_id, tui.rgb) {
        tui.print_attr_id = attr_id;
        return;
    }
    tui.print_attr_id = attr_id;

    let attrs = attr(tui, attr_id);
    let attr = ae_attr(tui, attrs);
    let has = |bit: c_int| attr & bit != 0;
    let (bold, italic, reverse) = (has(HL_BOLD), has(HL_ITALIC), has(HL_INVERSE));
    let (standout, strikethrough) = (has(HL_STANDOUT), has(HL_STRIKETHROUGH));
    let (altfont, dim, blink) = (has(HL_ALTFONT), has(HL_DIM), has(HL_BLINK));
    let (conceal, overline) = (has(HL_CONCEALED), has(HL_OVERLINE));

    // Terminals that can style an underline get the exact style asked for;
    // the rest get a plain underline for any of them.
    let styled_underline = !tui.ti.defs[kTerm_set_underline_style as usize].is_null();
    let underline_bits = attr & HL_UNDERLINE_MASK;
    let underline = if styled_underline {
        underline_bits == HL_UNDERLINE
    } else {
        underline_bits != 0
    };
    let style_of = |bit: c_int| styled_underline && underline_bits == bit;
    let any_underline = underline || UNDERLINE_STYLES.iter().any(|&(bit, _)| style_of(bit));

    if !tui.ti.defs[kTerm_set_attributes as usize].is_null() {
        // One sequence describes the six attributes `sgr` covers, so
        // whatever it does not mention is turned off for free.
        if bold || dim || blink || reverse || underline || standout {
            terminfo_print_nums(
                tui,
                kTerm_set_attributes,
                &[
                    standout as c_int,
                    underline as c_int,
                    reverse as c_int,
                    blink as c_int,
                    dim as c_int,
                    bold as c_int,
                ],
            );
        } else if !tui.default_attr {
            terminfo_out(tui, kTerm_exit_attribute_mode);
        }
    } else {
        // Without `sgr` there is no way to say "only these": reset, then
        // build the set back up one capability at a time.
        if !tui.default_attr {
            terminfo_out(tui, kTerm_exit_attribute_mode);
        }
        for (on, cap) in [
            (bold, kTerm_enter_bold_mode),
            (underline, kTerm_enter_underline_mode),
            (standout, kTerm_enter_standout_mode),
            (reverse, kTerm_enter_reverse_mode),
            (dim, kTerm_enter_dim_mode),
            (blink, kTerm_enter_blink_mode),
        ] {
            if on {
                terminfo_out(tui, cap);
            }
        }
    }

    // Attributes `sgr` never covered, whichever branch ran.
    if italic {
        terminfo_out(tui, kTerm_enter_italics_mode);
    }
    if altfont {
        let cap = tui.terminfo_ext.enter_altfont_mode;
        out_cstr(tui, cap);
    }
    if strikethrough {
        terminfo_out(tui, kTerm_enter_strikethrough_mode);
    }
    if conceal {
        terminfo_out(tui, kTerm_enter_secure_mode);
    }
    if overline {
        out(tui, b"\x1b[53m");
    }
    for (bit, style) in UNDERLINE_STYLES {
        if style_of(bit) {
            terminfo_print_nums(tui, kTerm_set_underline_style, &[style]);
        }
    }
    if any_underline && tui.can_set_underline_color {
        let color = attrs.rgb_sp_color;
        if color != -1 {
            out_fmt(
                tui,
                format_args!(
                    "\x1b[58:2::{}:{}:{}m",
                    color >> 16 & 0xff,
                    color >> 8 & 0xff,
                    color & 0xff
                ),
            );
        }
    }

    let fg = set_color(tui, ColorRole::Foreground, attrs, attr);
    let bg = set_color(tui, ColorRole::Background, attrs, attr);
    set_url(tui, attrs.url);

    // What the next cell can assume it inherits: whether the terminal is
    // back at its defaults, and whether a clear will paint the right
    // background (`bce` terminals clear in the current background, so a
    // coloured background is only safe to clear through if they say so).
    let plain = !bold && !dim && !blink && !conceal && !overline && !italic && !any_underline;
    tui.default_attr = fg == -1 && bg == -1 && plain && !reverse && !standout && !strikethrough;
    tui.can_clear_attr = !reverse
        && !standout
        && !dim
        && !blink
        && !conceal
        && !overline
        && !any_underline
        && !strikethrough
        && (tui.bce || bg == -1);
}

/// Which half of a colour pair is being set.
#[derive(Clone, Copy, PartialEq)]
enum ColorRole {
    Foreground,
    Background,
}

/// Emit `role`'s colour and return it, or -1 when the terminal is to use its
/// own default.
///
/// A highlight that names no colour of its own inherits the default colours
/// the editor last sent. `HL_*_INDEXED` marks a colour that is a palette
/// index even under `'termguicolors'`, which is why the RGB path can still
/// fall through to the indexed one.
fn set_color(tui: &mut TUIData, role: ColorRole, attrs: HlAttrs, attr: c_int) -> c_int {
    let foreground = role == ColorRole::Foreground;
    let indexed_bit = if foreground {
        HL_FG_INDEXED
    } else {
        HL_BG_INDEXED
    };
    if tui.rgb && attr & indexed_bit as c_int == 0 {
        let (want, fallback) = if foreground {
            (attrs.rgb_fg_color, tui.clear_attrs.rgb_fg_color)
        } else {
            (attrs.rgb_bg_color, tui.clear_attrs.rgb_bg_color)
        };
        let color = if want != -1 { want } else { fallback };
        if color != -1 {
            let cap = if foreground {
                kTerm_set_rgb_foreground
            } else {
                kTerm_set_rgb_background
            };
            terminfo_print_nums(
                tui,
                cap,
                &[color >> 16 & 0xff, color >> 8 & 0xff, color & 0xff],
            );
        }
        color
    } else {
        // cterm colours are stored one higher than they are sent, so that
        // zero can mean "no colour of my own".
        let (want, fallback) = if foreground {
            (attrs.cterm_fg_color, tui.clear_attrs.cterm_fg_color)
        } else {
            (attrs.cterm_bg_color, tui.clear_attrs.cterm_bg_color)
        };
        let color = c_int::from(if want != 0 { want } else { fallback }) - 1;
        if color != -1 {
            let cap = if foreground {
                kTerm_set_a_foreground
            } else {
                kTerm_set_a_background
            };
            terminfo_print_nums(tui, cap, &[color]);
        }
        color
    }
}

/// Open, close or leave alone the OSC 8 hyperlink the cells are being
/// painted inside.
fn set_url(tui: &mut TUIData, url: int32_t) {
    if tui.url == url {
        return;
    }
    if url >= 0 {
        // Assembled whole and staged in one go: a URL can be longer than the
        // staging buffer, and staging it in pieces would let a flush fall
        // between them — which wraps what it writes in synchronised-output
        // or cursor sequences, inside the OSC.
        let id = URL_ID_BASE.wrapping_add(url as u32);
        // SAFETY: the interned key is a NUL-terminated copy this module owns
        // and never frees.
        let target = unsafe { CStr::from_ptr(*(*URLS.ptr()).keys.add(url as usize)) };
        let mut seq = format!("\x1b]8;id={id};").into_bytes();
        let target = target.to_bytes();
        seq.reserve(target.len() + OSC8_TERMINATOR.len());
        seq.extend_from_slice(target);
        seq.extend_from_slice(OSC8_TERMINATOR);
        out(tui, &seq);
    } else {
        out(tui, b"\x1b]8;;\x1b\\");
    }
    tui.url = url;
}

/// Record the colours the editor wants unset colours to fall back to.
///
/// Every cell on screen may have been painted against the old defaults, so
/// the whole grid is repainted.
///
pub fn tui_default_colors_set(
    tui: &mut TUIData,
    rgb_fg: Integer,
    rgb_bg: Integer,
    rgb_sp: Integer,
    cterm_fg: Integer,
    cterm_bg: Integer,
) {
    tui.clear_attrs.rgb_fg_color = rgb_fg as int32_t;
    tui.clear_attrs.rgb_bg_color = rgb_bg as int32_t;
    tui.clear_attrs.rgb_sp_color = rgb_sp as int32_t;
    tui.clear_attrs.cterm_fg_color = cterm_fg as i16;
    tui.clear_attrs.cterm_bg_color = cterm_bg as i16;
    tui.print_attr_id = -1;
    // Until this arrives, nothing can be said about what the terminal would
    // paint an erased cell with, so clearing does not go through attributes.
    tui.set_default_colors = true;
    invalidate(tui, 0, tui.grid.height, 0, tui.grid.width);
}
