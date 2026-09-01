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

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    DECOR_ID_INVALID, DECOR_SIGN_HIGHLIGHT_INIT, decor_item, decor_sh_from_inline, kSHConceal,
    kSHConcealLines, kSHHlEol, kSHIsSign, kSHSpellOff, kSHSpellOn, kSHUIWatched,
};
use crate::api::extmark::virt_text_to_array;
use crate::api::private::helpers::{arena_array, arena_string, cstr_as_string};
use crate::decoration::{
    SIGN_WIDTH, kVLLeftcol, kVLScroll, kVPosWinCol, kVTHide, kVTIsLines, kVTLinesAbove,
    kVTRepeatLinebreak,
};
use crate::grid::{MAX_SCHAR_SIZE, schar_get};
use crate::highlight::dict::put;
use crate::highlight_group::syn_id2name;
use crate::sign::describe_sign_text;
use crate::types::{
    Arena, Array, DecorInline, DecorSignHighlight, DecorVirtText, Dict, Object, uint16_t, uint32_t,
};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// `kExtmark*`: which kinds of decoration a mark carries, as the bit mask the
/// marktree stores beside it.
const kExtmarkNone: uint16_t = 1;
const kExtmarkSign: uint16_t = 2;
const kExtmarkVirtText: uint16_t = 8;
const kExtmarkVirtLines: uint16_t = 16;
const kExtmarkHighlight: uint16_t = 32;

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
/// # Safety
/// `dict` must have room for every key below (its callers size it for the
/// whole `set_extmark` keyset); `decor` must be live, and `arena` valid or
/// null.
pub unsafe fn decor_to_dict_legacy(
    dict: &mut Dict,
    decor: DecorInline,
    hl_name: bool,
    arena: *mut Arena,
) {
    // SAFETY: the caller's decoration, dictionary and arena.
    let mut sh_hl: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
    let mut sh_sign: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
    let mut virt_text: *mut DecorVirtText = ptr::null_mut();
    let mut virt_lines: *mut DecorVirtText = ptr::null_mut();
    // A sentinel no real priority can take, so that "nothing here has a
    // priority" stays distinct from priority 0.
    let mut priority: i32 = -1;

    if decor.ext {
        let mut vt = unsafe { decor.data.ext }.vt;
        while !vt.is_null() {
            if unsafe { (*vt).flags } as c_int & kVTIsLines as c_int != 0 {
                virt_lines = vt;
            } else {
                virt_text = vt;
            }
            vt = unsafe { (*vt).next };
        }

        let mut idx: uint32_t = unsafe { decor.data.ext }.sh_idx;
        while idx != DECOR_ID_INVALID {
            let sh = decor_item(idx);
            if unsafe { (*sh).flags } as c_int & kSHIsSign as c_int != 0 {
                sh_sign = unsafe { *sh };
            } else {
                sh_hl = unsafe { *sh };
            }
            idx = unsafe { (*sh).next };
        }
    } else {
        sh_hl = decor_sh_from_inline(unsafe { decor.data.hl });
    }

    let flags = sh_hl.flags as c_int;
    if sh_hl.hl_id != 0 {
        unsafe { put(dict, c"hl_group", hl_group_name(sh_hl.hl_id, hl_name)) };
        let value = Object::boolean(flags & kSHHlEol as c_int != 0);
        // SAFETY: the caller's dictionary, sized for every key this
        // writes.
        unsafe { put(dict, c"hl_eol", value) };
        priority = i32::from(sh_hl.priority);
    }

    if flags & kSHConceal as c_int != 0 {
        let mut buf = [0 as c_char; MAX_SCHAR_SIZE as usize];
        unsafe { schar_get(buf.as_mut_ptr(), sh_hl.text[0]) };
        // SAFETY: the string the decoration owns, live for the copy.
        let value = unsafe { Object::string(arena_string(arena, cstr_as_string(buf.as_ptr()))) };
        // SAFETY: the caller's dictionary, sized for every key this
        // writes.
        unsafe { put(dict, c"conceal", value) };
    }

    if flags & kSHConcealLines as c_int != 0 {
        unsafe { put(dict, c"conceal_lines", Object::literal("")) };
    }

    if flags & kSHSpellOn as c_int != 0 {
        unsafe { put(dict, c"spell", Object::boolean(true)) };
    } else if flags & kSHSpellOff as c_int != 0 {
        unsafe { put(dict, c"spell", Object::boolean(false)) };
    }

    if flags & kSHUIWatched as c_int != 0 {
        unsafe { put(dict, c"ui_watched", Object::boolean(true)) };
    }

    if !sh_hl.url.is_null() {
        unsafe { put(dict, c"url", Object::string(cstr_as_string(sh_hl.url))) };
    }

    if !virt_text.is_null() {
        unsafe { put_virt_text(dict, &*virt_text, hl_name, arena) };
        priority = i32::from(unsafe { (*virt_text).priority });
    }

    if !virt_lines.is_null() {
        unsafe { put_virt_lines(dict, &*virt_lines, hl_name, arena) };
        priority = i32::from(unsafe { (*virt_lines).priority });
    }

    if sh_sign.flags as c_int & kSHIsSign as c_int != 0 {
        unsafe { put_sign(dict, &mut sh_sign, hl_name, arena) };
        priority = i32::from(sh_sign.priority);
    }

    if priority != -1 {
        unsafe { put(dict, c"priority", Object::integer(priority.into())) };
    }
}

/// The virt-text half of [`decor_to_dict_legacy`].
///
/// # Safety
/// `vt` must be a live virtual *text* item, not a virtual-lines one.
unsafe fn put_virt_text(dict: &mut Dict, vt: &DecorVirtText, hl_name: bool, arena: *mut Arena) {
    // SAFETY: the caller's virtual text and arena.
    if vt.hl_mode != 0 {
        let mode = HL_MODE_STR[vt.hl_mode as usize];
        // SAFETY: the string the decoration owns, live for the copy.
        let value = unsafe { Object::string(cstr_as_string(mode.as_ptr())) };
        // SAFETY: the caller's dictionary, sized for every key this
        // writes.
        unsafe { put(dict, c"hl_mode", value) };
    }

    let chunks = unsafe { virt_text_to_array(vt.data.text(), hl_name, arena) };
    unsafe { put(dict, c"virt_text", Object::array(chunks)) };
    let value = Object::boolean(vt.flags as c_int & kVTHide as c_int != 0);
    // SAFETY: the caller's dictionary, sized for every key this
    // writes.
    unsafe { put(dict, c"virt_text_hide", value) };
    let value = Object::boolean(vt.flags as c_int & kVTRepeatLinebreak as c_int != 0);
    // SAFETY: the caller's dictionary, sized for every key this
    // writes.
    unsafe { put(dict, c"virt_text_repeat_linebreak", value) };
    if vt.pos == kVPosWinCol {
        unsafe { put(dict, c"virt_text_win_col", Object::integer(vt.col.into())) };
    }
    let pos = VIRT_TEXT_POS_STR[vt.pos as usize];
    // SAFETY: the string the decoration owns, live for the copy.
    let value = unsafe { Object::string(cstr_as_string(pos.as_ptr())) };
    // SAFETY: the caller's dictionary, sized for every key this
    // writes.
    unsafe { put(dict, c"virt_text_pos", value) };
}

/// The virt-lines half of [`decor_to_dict_legacy`].
///
/// `virt_lines_leftcol` and `virt_lines_overflow` come from the *last* line's
/// flags: the flags are per line, the dictionary has one slot for them. That
/// is upstream's shape.
///
/// # Safety
/// `vt` must be a live virtual *lines* item.
unsafe fn put_virt_lines(dict: &mut Dict, vt: &DecorVirtText, hl_name: bool, arena: *mut Arena) {
    // SAFETY: the caller's virtual lines and arena.
    let lines = vt.data.lines();
    let mut all_chunks: Array = arena_array(arena, lines.size);
    let mut line_flags: c_int = 0;
    for i in 0..lines.size {
        let line = unsafe { *lines.items.add(i) };
        line_flags = line.flags;
        let chunks = unsafe { virt_text_to_array(line.line, hl_name, arena) };
        // `arena_array` was asked for exactly this many.
        assert!(all_chunks.size < all_chunks.capacity, "virt_lines overflow");
        unsafe { *all_chunks.items.add(all_chunks.size) = Object::array(chunks) };
        all_chunks.size += 1;
    }

    unsafe { put(dict, c"virt_lines", Object::array(all_chunks)) };
    let value = Object::boolean(vt.flags as c_int & kVTLinesAbove as c_int != 0);
    // SAFETY: the caller's dictionary, sized for every key this
    // writes.
    unsafe { put(dict, c"virt_lines_above", value) };
    let value = Object::boolean(line_flags & kVLLeftcol as c_int != 0);
    // SAFETY: the caller's dictionary, sized for every key this
    // writes.
    unsafe { put(dict, c"virt_lines_leftcol", value) };
    let overflow = if line_flags & kVLScroll as c_int != 0 {
        c"scroll"
    } else {
        c"trunc"
    };
    // SAFETY: the string the decoration owns, live for the copy.
    let value = unsafe { Object::string(cstr_as_string(overflow.as_ptr())) };
    // SAFETY: the caller's dictionary, sized for every key this
    // writes.
    unsafe { put(dict, c"virt_lines_overflow", value) };
}

/// The sign half of [`decor_to_dict_legacy`].
///
/// # Safety
/// `sh` must be a live sign item; `arena` valid or null.
unsafe fn put_sign(dict: &mut Dict, sh: &mut DecorSignHighlight, hl_name: bool, arena: *mut Arena) {
    // SAFETY: the caller's sign and arena.
    if sh.text[0] != 0 {
        let mut buf = [0 as c_char; SIGN_WIDTH as usize * MAX_SCHAR_SIZE as usize];
        unsafe { describe_sign_text(buf.as_mut_ptr(), sh.text.as_mut_ptr()) };
        // SAFETY: the string the decoration owns, live for the copy.
        let value = unsafe { Object::string(arena_string(arena, cstr_as_string(buf.as_ptr()))) };
        // SAFETY: the caller's dictionary, sized for every key this
        // writes.
        unsafe { put(dict, c"sign_text", value) };
    }

    if !sh.sign_name.is_null() {
        // SAFETY: the string the decoration owns, live for the copy.
        let value = unsafe { Object::string(cstr_as_string(sh.sign_name)) };
        // SAFETY: the caller's dictionary, sized for every key this
        // writes.
        unsafe { put(dict, c"sign_name", value) };
    }

    for (key, id) in [
        (c"sign_hl_group", sh.hl_id),
        (c"number_hl_group", sh.number_hl_id),
        (c"line_hl_group", sh.line_hl_id),
        (c"cursorline_hl_group", sh.cursorline_hl_id),
    ] {
        if id != 0 {
            unsafe { put(dict, key, hl_group_name(id, hl_name)) };
        }
    }
}

/// Which kinds of decoration `decor` carries, as the `kExtmark*` mask the
/// marktree stores beside the mark.
///
/// # Safety
/// `decor` must be live.
pub unsafe fn decor_type_flags(decor: DecorInline) -> uint16_t {
    // SAFETY: the caller's decoration.
    if !decor.ext {
        return if unsafe { decor.data.hl }.flags as c_int & kSHIsSign as c_int != 0 {
            kExtmarkSign
        } else {
            kExtmarkHighlight
        };
    }

    let mut type_flags = kExtmarkNone;
    let mut vt = unsafe { decor.data.ext }.vt;
    while !vt.is_null() {
        type_flags |= if unsafe { (*vt).flags } as c_int & kVTIsLines as c_int != 0 {
            kExtmarkVirtLines
        } else {
            kExtmarkVirtText
        };
        vt = unsafe { (*vt).next };
    }
    let mut idx: uint32_t = unsafe { decor.data.ext }.sh_idx;
    while idx != DECOR_ID_INVALID {
        let sh = decor_item(idx);
        type_flags |= if unsafe { (*sh).flags } as c_int & kSHIsSign as c_int != 0 {
            kExtmarkSign
        } else {
            kExtmarkHighlight
        };
        idx = unsafe { (*sh).next };
    }
    type_flags
}

/// A highlight group as the API reports it: its name when the caller asked
/// for names, otherwise its id.
///
/// # Safety
/// Reaches the group table; main thread only.
pub unsafe fn hl_group_name(hl_id: c_int, hl_name: bool) -> Object {
    if hl_name {
        // SAFETY: `syn_id2name` answers a static or table-owned string.
        Object::string(unsafe { cstr_as_string(syn_id2name(hl_id)) })
    } else {
        Object::integer(hl_id.into())
    }
}
