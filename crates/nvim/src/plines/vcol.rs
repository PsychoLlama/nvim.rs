#![deny(unsafe_op_in_unsafe_fn)]

//! Virtual columns of a position.
//!
//! The `getvcol` family: where a `pos_T` lands on screen, once tabs, inline
//! virtual text, double-width characters and 'virtualedit' have had their
//! say. All of it walks the line with the parent module's charsize
//! functions.

use super::*;
use crate::pos::MAXCOL;
use crate::types::NUL;

/// Virtual column of `pos`, in up to three flavours:
///
/// * `start` — the first column the character occupies,
/// * `cursor` — where the cursor sits on it (the last column of a tab, in
///   Normal mode without 'list'),
/// * `end` — the last column it occupies.
///
/// Called very often; keep it fast. A `pos.col` past the end of the line is
/// clamped to the line length on the way out.
///
/// # Safety
/// `wp` and `pos` must be live; the out-parameters may each be null.
pub unsafe fn getvcol(
    wp: *mut win_T,
    pos: *mut pos_T,
    start: *mut colnr_T,
    cursor: *mut colnr_T,
    end: *mut colnr_T,
) {
    unsafe {
        let line = ml_get_buf((*wp).w_buffer, (*pos).lnum);
        let end_col = (*pos).col;

        let mut csarg = CharsizeArg::default();
        let cstype = init_charsize_arg(&mut csarg, wp, (*pos).lnum, line);
        csarg.max_head_vcol = -1;

        let mut on_nul = false;
        let mut vcol: colnr_T = 0;
        let mut char_size;
        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);

        if cstype == CharsizeKind::Fast {
            let use_tabstop = csarg.use_tabstop;
            loop {
                if *ci.ptr == NUL as c_char {
                    // The cursor on a NUL is treated like a one-cell char.
                    char_size = CharSize { width: 1, head: 0 };
                    break;
                }
                char_size = charsize_fast_impl(wp, ci.ptr, use_tabstop, vcol, ci.chr.value);
                let next = utfc_next(ci);
                if next.ptr.offset_from(line) > end_col as isize {
                    break;
                }
                ci = next;
                vcol += char_size.width;
            }
        } else {
            loop {
                char_size = charsize_regular(&mut csarg, ci.ptr, vcol, ci.chr.value);
                // Don't go past the end of the line.
                if *ci.ptr == NUL as c_char {
                    // A NUL at the end of the line takes one column, unless
                    // there is virtual text.
                    char_size.width = 1 + csarg.cur_text_width_left + csarg.cur_text_width_right;
                    on_nul = true;
                    break;
                }
                let next = utfc_next(ci);
                if next.ptr.offset_from(line) > end_col as isize {
                    break;
                }
                ci = next;
                vcol += char_size.width;
            }
        }

        if *ci.ptr == NUL as c_char
            && end_col < MAXCOL
            && end_col as isize > ci.ptr.offset_from(line)
        {
            (*pos).col = ci.ptr.offset_from(line) as colnr_T;
        }

        let head = char_size.head;
        let incr = char_size.width;

        if !start.is_null() {
            *start = vcol + head;
        }
        if !end.is_null() {
            *end = vcol + incr - 1;
        }
        if !cursor.is_null() {
            let cursor_at_tab_end = ci.chr.value == TAB
                && State.get() & MODE_NORMAL != 0
                && (*wp).w_onebuf_opt.wo_list == 0
                && !virtual_active(wp)
                && !(VIsual_active.get()
                    && (*p_sel.get() == b'e' as c_char || ltoreq(*pos, VIsual.get())));
            if cursor_at_tab_end {
                *cursor = vcol + incr - 1;
            } else {
                vcol += virt_text_cursor_off(&csarg, on_nul);
                *cursor = vcol + head;
            }
        }
    }
}

/// Virtual cursor column in the current window, pretending 'list' is off.
///
/// # Safety
/// `posp` must be live.
pub unsafe fn getvcol_nolist(posp: *mut pos_T) -> colnr_T {
    unsafe {
        let win = curwin.get();
        let list_save = (*win).w_onebuf_opt.wo_list;
        let mut vcol: colnr_T = 0;
        let null = ::core::ptr::null_mut::<colnr_T>();

        (*win).w_onebuf_opt.wo_list = 0;
        if (*posp).coladd != 0 {
            getvvcol(win, posp, null, &raw mut vcol, null);
        } else {
            getvcol(win, posp, null, &raw mut vcol, null);
        }
        (*win).w_onebuf_opt.wo_list = list_save;
        vcol
    }
}

/// [`getvcol`] in virtual-edit mode, where the cursor can sit past the end of
/// a line or inside a tab.
///
/// # Safety
/// As [`getvcol`].
pub unsafe fn getvvcol(
    wp: *mut win_T,
    pos: *mut pos_T,
    start: *mut colnr_T,
    cursor: *mut colnr_T,
    end: *mut colnr_T,
) {
    unsafe {
        if !virtual_active(wp) {
            getvcol(wp, pos, start, cursor, end);
            return;
        }

        // In virtual mode only one value is wanted.
        let null = ::core::ptr::null_mut::<colnr_T>();
        let mut col: colnr_T = 0;
        getvcol(wp, pos, &raw mut col, null, null);

        let mut coladd = (*pos).coladd;
        let mut endadd: colnr_T = 0;

        // The cursor cannot sit on part of a wide character.
        let ptr = ml_get_buf((*wp).w_buffer, (*pos).lnum);
        if (*pos).col < ml_get_buf_len((*wp).w_buffer, (*pos).lnum) {
            let c = utf_ptr2char(ptr.offset((*pos).col as isize));
            if c != TAB && vim_isprintc(c) {
                endadd = ptr2cells(ptr.offset((*pos).col as isize)) - 1;
                if coladd > endadd {
                    // Past the end of the line.
                    endadd = 0;
                } else {
                    coladd = 0;
                }
            }
        }
        col += coladd;

        if !start.is_null() {
            *start = col;
        }
        if !cursor.is_null() {
            *cursor = col;
        }
        if !end.is_null() {
            *end = col + endadd;
        }
    }
}

/// Leftmost and rightmost virtual column of `pos1` and `pos2`, for Visual
/// block mode.
///
/// # Safety
/// All pointers must be live; `left` and `right` are always written.
pub unsafe fn getvcols(
    wp: *mut win_T,
    pos1: *mut pos_T,
    pos2: *mut pos_T,
    left: *mut colnr_T,
    right: *mut colnr_T,
) {
    unsafe {
        let (first, second) = if lt(*pos1, *pos2) {
            (pos1, pos2)
        } else {
            (pos2, pos1)
        };

        let null = ::core::ptr::null_mut::<colnr_T>();
        let mut from1: colnr_T = 0;
        let mut from2: colnr_T = 0;
        let mut to1: colnr_T = 0;
        let mut to2: colnr_T = 0;
        getvvcol(wp, first, &raw mut from1, null, &raw mut to1);
        getvvcol(wp, second, &raw mut from2, null, &raw mut to2);

        *left = from1.min(from2);
        // With 'selection' exclusive the block stops one column short of the
        // second position -- but only when that still leaves the first one's
        // last column inside the block.
        let before_second = from2 - 1;
        *right = if to2 > to1 {
            if *p_sel.get() == b'e' as c_char && before_second >= to1 {
                before_second
            } else {
                to2
            }
        } else {
            to1
        };
    }
}
