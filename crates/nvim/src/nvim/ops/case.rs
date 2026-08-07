//! `g~`, `gu`, `gU` and `g?` -- rewriting the characters in place.
//!
//! `op_tilde` walks the region and hands each position to `swapchar`, which
//! is the whole of the per-character decision: which of the four operators
//! is running, whether the character has a case at all, and -- for rot13 --
//! that only ASCII letters move.  A case change can change a character's
//! *byte length* (there are pairs whose upper and lower cases differ in
//! UTF-8 width), which is why the walk works through `pbyte` rather than
//! writing into the line.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn op_tilde(mut oap: *mut oparg_T) {
    unsafe {
        let mut bd: block_def = block_def {
            startspaces: 0,
            endspaces: 0,
            textlen: 0,
            textstart: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            textcol: 0,
            start_vcol: 0,
            end_vcol: 0,
            is_short: 0,
            is_MAX: 0,
            is_oneChar: 0,
            pre_whitesp: 0,
            pre_whitesp_c: 0,
            end_char_vcols: 0,
            start_char_vcols: 0,
        };
        let mut did_change: bool = false_0 != 0;
        if u_save(
            (*oap).start.lnum - 1 as linenr_T,
            (*oap).end.lnum + 1 as linenr_T,
        ) == FAIL
        {
            return;
        }
        let mut pos: pos_T = (*oap).start;
        if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            while pos.lnum <= (*oap).end.lnum {
                block_prep(oap, &raw mut bd, pos.lnum, false_0 != 0);
                pos.col = bd.textcol;
                let mut one_change: bool = swapchars((*oap).op_type, &raw mut pos, bd.textlen) != 0;
                did_change =
                    did_change as ::core::ffi::c_int | one_change as ::core::ffi::c_int != 0;
                pos.lnum += 1;
            }
            if did_change {
                changed_lines(
                    curbuf.get(),
                    (*oap).start.lnum,
                    0 as colnr_T,
                    (*oap).end.lnum + 1 as linenr_T,
                    0 as linenr_T,
                    true_0 != 0,
                );
            }
        } else {
            if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                (*oap).start.col = 0 as ::core::ffi::c_int as colnr_T;
                pos.col = 0 as ::core::ffi::c_int as colnr_T;
                (*oap).end.col = ml_get_len((*oap).end.lnum);
                if (*oap).end.col != 0 {
                    (*oap).end.col -= 1;
                }
            } else if !(*oap).inclusive {
                dec(&raw mut (*oap).end);
            }
            if pos.lnum == (*oap).end.lnum {
                did_change = swapchars(
                    (*oap).op_type,
                    &raw mut pos,
                    (*oap).end.col as ::core::ffi::c_int - pos.col as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int,
                ) != 0;
            } else {
                loop {
                    did_change = did_change as ::core::ffi::c_int
                        | swapchars(
                            (*oap).op_type,
                            &raw mut pos,
                            if pos.lnum == (*oap).end.lnum {
                                (*oap).end.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                            } else {
                                ml_get_pos_len(&raw mut pos)
                            },
                        )
                        != 0;
                    if ltoreq((*oap).end, pos) as ::core::ffi::c_int != 0
                        || inc(&raw mut pos) == -1 as ::core::ffi::c_int
                    {
                        break;
                    }
                }
            }
            if did_change {
                changed_lines(
                    curbuf.get(),
                    (*oap).start.lnum,
                    (*oap).start.col,
                    (*oap).end.lnum + 1 as linenr_T,
                    0 as linenr_T,
                    true_0 != 0,
                );
            }
        }
        if !did_change && (*oap).is_VIsual as ::core::ffi::c_int != 0 {
            redraw_curbuf_later(UPD_INVERTED);
        }
        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curbuf.get()).b_op_start = (*oap).start;
            (*curbuf.get()).b_op_end = (*oap).end;
        }
        if (*oap).line_count as OptInt > p_report.get() {
            smsg(
                0 as ::core::ffi::c_int,
                ngettext(
                    b"%ld line changed\0".as_ptr() as *const ::core::ffi::c_char,
                    b"%ld lines changed\0".as_ptr() as *const ::core::ffi::c_char,
                    (*oap).line_count as ::core::ffi::c_ulong,
                ),
                (*oap).line_count as int64_t,
            );
        }
    }
}

unsafe extern "C" fn swapchars(
    mut op_type: ::core::ffi::c_int,
    mut pos: *mut pos_T,
    mut length: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut did_change: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut todo: ::core::ffi::c_int = length;
        while todo > 0 as ::core::ffi::c_int {
            let len: ::core::ffi::c_int = utfc_ptr2len(ml_get_pos(pos));
            if len > 0 as ::core::ffi::c_int {
                todo -= len - 1 as ::core::ffi::c_int;
            }
            did_change |= swapchar(op_type, pos) as ::core::ffi::c_int;
            if inc(pos) == -1 as ::core::ffi::c_int {
                break;
            }
            todo -= 1;
        }
        return did_change;
    }
}

pub unsafe extern "C" fn swapchar(mut op_type: ::core::ffi::c_int, mut pos: *mut pos_T) -> bool {
    unsafe {
        let c: ::core::ffi::c_int = gchar_pos(pos);
        if c >= 0x80 as ::core::ffi::c_int && op_type == OP_ROT13 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        let mut nc: ::core::ffi::c_int = c;
        if mb_islower(c) {
            if op_type == OP_ROT13 as ::core::ffi::c_int {
                nc = (c - 'a' as ::core::ffi::c_int + 13 as ::core::ffi::c_int)
                    % 26 as ::core::ffi::c_int
                    + 'a' as ::core::ffi::c_int;
            } else if op_type != OP_LOWER as ::core::ffi::c_int {
                nc = mb_toupper(c);
            }
        } else if mb_isupper(c) {
            if op_type == OP_ROT13 as ::core::ffi::c_int {
                nc = (c - 'A' as ::core::ffi::c_int + 13 as ::core::ffi::c_int)
                    % 26 as ::core::ffi::c_int
                    + 'A' as ::core::ffi::c_int;
            } else if op_type != OP_UPPER as ::core::ffi::c_int {
                nc = mb_tolower(c);
            }
        }
        if nc != c {
            if c >= 0x80 as ::core::ffi::c_int || nc >= 0x80 as ::core::ffi::c_int {
                let mut sp: pos_T = (*curwin.get()).w_cursor;
                (*curwin.get()).w_cursor = *pos;
                del_bytes(
                    utf_ptr2len(get_cursor_pos_ptr()),
                    false_0 != 0,
                    false_0 != 0,
                );
                ins_char(nc);
                (*curwin.get()).w_cursor = sp;
            } else {
                pbyte(*pos, nc);
            }
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}
