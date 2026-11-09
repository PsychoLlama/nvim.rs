//! A decoration as an API dictionary.
//!
//! [`decor_to_dict_legacy`] is what `nvim_buf_get_extmark*(details = true)`
//! answers with: one flat dictionary carrying whichever of the virt-text,
//! virt-lines, sign and highlight parts the decoration has. It assumes at
//! most one of each kind, which is not always true — the name says "legacy"
//! for that reason.
//!
//! The dictionary is the caller's, already sized for the whole `set_extmark`
//! keyset, and is filled by appending: `put` panics rather than overflowing
//! it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use super::{
    DECOR_ID_INVALID, SH_CONCEAL, SH_CONCEAL_LINES, SH_HL_EOL, SH_IS_SIGN, SH_SPELL_OFF,
    SH_SPELL_ON, SHUI_WATCHED, decor_sh_from_inline, sh_chain, virt_chain, with_decor_item,
    with_decor_virt,
};
use crate::api::extmark::virt_text_to_array;
use crate::decoration::defs::{DecorInline, DecorSignHighlight, DecorVirtText};
use crate::decoration::{
    V_POS_WIN_COL, VL_LEFTCOL, VL_SCROLL, VT_HIDE, VT_IS_LINES, VT_LINES_ABOVE, VT_REPEAT_LINEBREAK,
};
use crate::grid::schar_bytes;
use crate::highlight::dict::put;
use crate::highlight_group::syn_id2name;
use crate::os::libc::uint16_t;
use crate::sign::sign_text_bytes;
use crate::types::DictBuilder;
use crate::types::{ApiString, Object};
use core::ffi::{CStr, c_int};

/// `kExtmark*`: which kinds of decoration a mark carries, as the bit mask the
/// marktree stores beside it.
const EXTMARK_NONE: uint16_t = 1;
const EXTMARK_SIGN: uint16_t = 2;
const EXTMARK_VIRT_TEXT: uint16_t = 8;
const EXTMARK_VIRT_LINES: uint16_t = 16;
const EXTMARK_HIGHLIGHT: uint16_t = 32;

/// `VirtTextPos` as the API spells it. Keep in sync with `VirtTextPos`.
const VIRT_TEXT_POS_STR: [&CStr; 6] = [
    c"eol",
    c"eol_right_align",
    c"inline",
    c"overlay",
    c"right_align",
    c"win_col",
];

/// `HlMode` as the API spells it; index 0 is "unset". Keep in sync with
/// `HlMode`.
const HL_MODE_STR: [&CStr; 4] = [c"", c"replace", c"combine", c"blend"];

/// Writes `decor` out as dictionary entries appended to `dict`.
///
/// Only one entry of each kind is described, which is what makes this the
/// "legacy" form: a mark may carry several virt texts or several highlights,
/// and the last of each wins here.
///
/// The `priority` reported is the *last* part's, in the fixed order
/// highlight, virt text, virt lines, sign — so a decoration with both a sign
/// and a highlight reports the sign's.
///
/// `put` panics rather than overflowing `dict`: its callers size it for the
/// whole `set_extmark` keyset.
pub fn decor_to_dict_legacy(dict: &mut DictBuilder, decor: DecorInline, hl_name: bool) {
    let mut sh_hl = DECOR_ID_INVALID;
    let mut sh_sign = DECOR_ID_INVALID;
    let mut virt_text = DECOR_ID_INVALID;
    let mut virt_lines = DECOR_ID_INVALID;
    // A sentinel no real priority can take, so that "nothing here has a
    // priority" stays distinct from priority 0.
    let mut priority: i32 = -1;

    match decor {
        DecorInline::Ext(ext) => {
            for vt in virt_chain(ext.vt_idx) {
                if with_decor_virt(vt, |link| link.flags & VT_IS_LINES != 0) {
                    virt_lines = vt;
                } else {
                    virt_text = vt;
                }
            }
            for idx in sh_chain(decor) {
                if with_decor_item(idx, |sh| sh.flags & SH_IS_SIGN != 0) {
                    sh_sign = idx;
                } else {
                    sh_hl = idx;
                }
            }
            if sh_hl != DECOR_ID_INVALID {
                with_decor_item(sh_hl, |sh| put_highlight(dict, sh, hl_name, &mut priority));
            }
        }
        DecorInline::Hl(hl) => {
            let sh = decor_sh_from_inline(hl);
            put_highlight(dict, &sh, hl_name, &mut priority);
        }
    }

    if virt_text != DECOR_ID_INVALID {
        with_decor_virt(virt_text, |vt| put_virt_text(dict, vt, hl_name));
        priority = i32::from(with_decor_virt(virt_text, |vt| vt.priority));
    }

    if virt_lines != DECOR_ID_INVALID {
        with_decor_virt(virt_lines, |vt| put_virt_lines(dict, vt, hl_name));
        priority = i32::from(with_decor_virt(virt_lines, |vt| vt.priority));
    }

    if sh_sign != DECOR_ID_INVALID {
        with_decor_item(sh_sign, |sh| put_sign(dict, sh, hl_name));
        priority = i32::from(with_decor_item(sh_sign, |sh| sh.priority));
    }

    if priority != -1 {
        put(dict, c"priority", Object::integer(priority.into()));
    }
}

/// The highlight half of [`decor_to_dict_legacy`], raising `priority` to the
/// highlight's when it names a group.
fn put_highlight(
    dict: &mut DictBuilder,
    sh_hl: &DecorSignHighlight,
    hl_name: bool,
    priority: &mut i32,
) {
    let flags = sh_hl.flags;
    if sh_hl.hl_id != 0 {
        put(dict, c"hl_group", hl_group_name(sh_hl.hl_id, hl_name));
        let value = Object::boolean(flags & SH_HL_EOL != 0);
        put(dict, c"hl_eol", value);
        *priority = i32::from(sh_hl.priority);
    }

    if flags & SH_CONCEAL != 0 {
        let text = schar_bytes(sh_hl.text[0]);
        let value = Object::string(ApiString::from_bytes(crate::cstr::until_nul(&text)));
        put(dict, c"conceal", value);
    }

    if flags & SH_CONCEAL_LINES != 0 {
        put(dict, c"conceal_lines", Object::literal(""));
    }

    if flags & SH_SPELL_ON != 0 {
        put(dict, c"spell", Object::boolean(true));
    } else if flags & SH_SPELL_OFF != 0 {
        put(dict, c"spell", Object::boolean(false));
    }

    if flags & SHUI_WATCHED != 0 {
        put(dict, c"ui_watched", Object::boolean(true));
    }

    if let Some(url) = &sh_hl.url {
        put(
            dict,
            c"url",
            Object::string(ApiString::from_bytes(url.to_bytes())),
        );
    }
}

/// The virt-text half of [`decor_to_dict_legacy`]: `vt` is a virtual *text*
/// link, not a virtual-lines one.
fn put_virt_text(dict: &mut DictBuilder, vt: &DecorVirtText, hl_name: bool) {
    if vt.hl_mode != 0 {
        let mode = HL_MODE_STR[vt.hl_mode as usize];
        let value = Object::string(ApiString::from_cstr(mode));
        put(dict, c"hl_mode", value);
    }

    let chunks = virt_text_to_array(vt.data.text(), hl_name);
    put(dict, c"virt_text", Object::array(chunks));
    let value = Object::boolean(vt.flags & VT_HIDE != 0);
    put(dict, c"virt_text_hide", value);
    let value = Object::boolean(vt.flags & VT_REPEAT_LINEBREAK != 0);
    put(dict, c"virt_text_repeat_linebreak", value);
    if vt.pos == V_POS_WIN_COL {
        put(dict, c"virt_text_win_col", Object::integer(vt.col.into()));
    }
    let pos = VIRT_TEXT_POS_STR[vt.pos as usize];
    let value = Object::string(ApiString::from_cstr(pos));
    put(dict, c"virt_text_pos", value);
}

/// The virt-lines half of [`decor_to_dict_legacy`]: `vt` is a virtual
/// *lines* link.
///
/// `virt_lines_leftcol` and `virt_lines_overflow` come from the *last* line's
/// flags: the flags are per line, the dictionary has one slot for them. That
/// is upstream's shape.
fn put_virt_lines(dict: &mut DictBuilder, vt: &DecorVirtText, hl_name: bool) {
    let lines = vt.data.lines();
    let mut all_chunks = Vec::with_capacity(lines.len());
    let mut line_flags: c_int = 0;
    for line in lines {
        line_flags = line.flags;
        let chunks = virt_text_to_array(&line.line, hl_name);
        all_chunks.push(Object::array(chunks));
    }

    put(dict, c"virt_lines", Object::array(all_chunks));
    let value = Object::boolean(vt.flags & VT_LINES_ABOVE != 0);
    put(dict, c"virt_lines_above", value);
    let value = Object::boolean(line_flags & VL_LEFTCOL != 0);
    put(dict, c"virt_lines_leftcol", value);
    let overflow = if line_flags & VL_SCROLL != 0 {
        c"scroll"
    } else {
        c"trunc"
    };
    let value = Object::string(ApiString::from_cstr(overflow));
    put(dict, c"virt_lines_overflow", value);
}

/// The sign half of [`decor_to_dict_legacy`].
fn put_sign(dict: &mut DictBuilder, sh: &DecorSignHighlight, hl_name: bool) {
    if sh.text[0] != 0 {
        let value = Object::string(ApiString::from_bytes(&sign_text_bytes(&sh.text)));
        put(dict, c"sign_text", value);
    }

    if let Some(name) = sh.sign_name_cstr() {
        put(
            dict,
            c"sign_name",
            Object::string(ApiString::from_bytes(name.to_bytes())),
        );
    }

    for (key, id) in [
        (c"sign_hl_group", sh.hl_id),
        (c"number_hl_group", sh.number_hl_id),
        (c"line_hl_group", sh.line_hl_id),
        (c"cursorline_hl_group", sh.cursorline_hl_id),
    ] {
        if id != 0 {
            put(dict, key, hl_group_name(id, hl_name));
        }
    }
}

/// Which kinds of decoration `decor` carries, as the `kExtmark*` mask the
/// marktree stores beside the mark.
pub fn decor_type_flags(decor: DecorInline) -> uint16_t {
    let ext = match decor {
        DecorInline::Hl(hl) => {
            return if hl.flags & SH_IS_SIGN != 0 {
                EXTMARK_SIGN
            } else {
                EXTMARK_HIGHLIGHT
            };
        }
        DecorInline::Ext(ext) => ext,
    };

    let mut type_flags = EXTMARK_NONE;
    for vt in virt_chain(ext.vt_idx) {
        type_flags |= if with_decor_virt(vt, |link| link.flags & VT_IS_LINES != 0) {
            EXTMARK_VIRT_LINES
        } else {
            EXTMARK_VIRT_TEXT
        };
    }
    for idx in sh_chain(decor) {
        type_flags |= if with_decor_item(idx, |sh| sh.flags & SH_IS_SIGN != 0) {
            EXTMARK_SIGN
        } else {
            EXTMARK_HIGHLIGHT
        };
    }
    type_flags
}

/// A highlight group as the API reports it: its name when the caller asked
/// for names, otherwise its id.
pub fn hl_group_name(hl_id: c_int, hl_name: bool) -> Object {
    if hl_name {
        Object::string(ApiString::from_cstr(syn_id2name(hl_id)))
    } else {
        Object::integer(hl_id.into())
    }
}
