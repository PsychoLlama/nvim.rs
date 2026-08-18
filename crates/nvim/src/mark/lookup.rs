use crate::ascii::ascii_isdigit;
use crate::buffer::{bt_prompt, buflist_getfile};
use crate::cursor::check_cursor;
use crate::edit::beginline;
use crate::global_cell::GlobalCell;
use crate::main::{curbuf, curwin, listcmd_busy, namedfm};
use crate::message::emsg;
use crate::pos::{MAXCOL, MAXLNUM, lt};
use crate::textobject::{findpar, findsent};
use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

use super::jumplist::*;
use super::*;
use crate::search::{BACKWARD, FORWARD};

pub unsafe fn mark_get(
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut fmp: *mut fmark_T,
    mut flag: MarkGet,
    mut name: c_int,
) -> *mut fmark_T {
    let mut fm: *mut fmark_T = ptr::null_mut();
    if name as c_uint >= 'A' as c_uint && name as c_uint <= 'Z' as c_uint || ascii_isdigit(name) {
        let mut xfm: *mut xfmark_T =
            mark_get_global(flag as c_uint != kMarkAllNoResolve as c_int as c_uint, name);
        fm = &raw mut (*xfm).fmark;
        if flag as c_uint == kMarkBufLocal as c_int as c_uint && (*xfm).fmark.fnum != (*buf).handle
        {
            return pos_to_mark(
                buf,
                ptr::null_mut(),
                pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
            );
        }
    } else if name > 0 && name < NMARK_LOCAL_MAX {
        fm = mark_get_local(buf, win, name);
    }
    if !fmp.is_null() && !fm.is_null() {
        *fmp = *fm;
        return fmp;
    }
    return fm;
}

/// Get a global mark {A-Z0-9}.
///
/// `name` — the name of the mark.
/// `resolve` — Whether to try resolving the mark fnum (i.e., load the buffer stored in
///                 the mark fname and update the xfmark_T (expensive)).
///
/// Returns mark
pub unsafe fn mark_get_global(mut resolve: bool, mut name: c_int) -> *mut xfmark_T {
    let mut mark: *mut xfmark_T = ptr::null_mut();
    if ascii_isdigit(name) {
        name = name - '0' as c_int + NMARKS;
    } else if name as c_uint >= 'A' as c_uint && name as c_uint <= 'Z' as c_uint {
        name -= 'A' as c_int;
    } else {
        // Deliberately a hard failure, not a `debug_assert!`: `name` is the
        // index into `namedfm` two lines down, and neither branch above has
        // clamped it, so falling through reads out of bounds. Both callers
        // (`mark_get` and `nvim_get_mark`) reject anything that is not a
        // digit or an uppercase letter before they get here.
        unreachable!("mark name is neither a digit nor an uppercase letter");
    }
    mark = (namedfm.ptr() as *mut xfmark_T).offset(name as isize);
    if resolve && (*mark).fmark.fnum == 0 {
        fname2fnum(mark);
    }
    return mark;
}

/// Get a local mark (lowercase and symbols).
///
/// Some marks are not actually marks, but positions that are never adjusted or motions presented as
/// marks. Search first for marks and fallback to finding motion type marks. If it's known
/// ahead of time that the mark is actually a motion use the mark_get_motion() directly.
///
/// @note  Lowercase, last_cursor '"', last insert '^', last change '.' are not statically
/// allocated, everything else is.
/// `name` — the name of the mark.
/// `win` — window to retrieve marks that belong to it (motions and context mark).
/// `buf` — buf to retrieve marks that belong to it.
///
/// Returns mark, NULL if not found.
pub unsafe fn mark_get_local(
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut name: c_int,
) -> *mut fmark_T {
    let mut mark: *mut fmark_T = ptr::null_mut();
    if name as c_uint >= 'a' as c_uint && name as c_uint <= 'z' as c_uint {
        mark = (&raw mut (*buf).b_namedm as *mut fmark_T).offset((name - 'a' as c_int) as isize);
    } else if name == '[' as c_int {
        mark = pos_to_mark(buf, ptr::null_mut(), (*buf).b_op_start);
    } else if name == ']' as c_int {
        mark = pos_to_mark(buf, ptr::null_mut(), (*buf).b_op_end);
    } else if name == '<' as c_int || name == '>' as c_int {
        mark = mark_get_visual(buf, name);
    } else if name == '\'' as c_int || name == '`' as c_int {
        mark = pos_to_mark(curbuf.get(), ptr::null_mut(), (*win).w_pcmark);
    } else if name == '"' as c_int {
        mark = &raw mut (*buf).b_last_cursor;
    } else if name == '^' as c_int {
        mark = &raw mut (*buf).b_last_insert;
    } else if name == '.' as c_int {
        mark = &raw mut (*buf).b_last_change;
    } else if name == ':' as c_int && bt_prompt(buf) {
        mark = &raw mut (*buf).b_prompt_start;
    } else {
        mark = mark_get_motion(buf, win, name);
    }
    if !mark.is_null() {
        (*mark).fnum = (*buf).handle as c_int;
    }
    return mark;
}

/// Get marks that are actually motions but return them as marks
///
/// Gets the following motions as marks: '{', '}', '(', ')'
/// `name` — name of the mark
/// `win` — window to retrieve the cursor to calculate the mark.
/// `buf` — buf to wrap motion marks with it's buffer number (fm->fnum).
///
/// @return[static] Mark.
pub unsafe fn mark_get_motion(
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut name: c_int,
) -> *mut fmark_T {
    let mut mark: *mut fmark_T = ptr::null_mut();
    let pos: pos_T = (*curwin.get()).w_cursor;
    let slcb: bool = listcmd_busy.get();
    listcmd_busy.set(true);
    if name == '{' as c_int || name == '}' as c_int {
        let mut oa: oparg_T = oparg_T {
            op_type: 0,
            regname: 0,
            motion_type: kMTCharWise,
            motion_force: 0,
            use_reg_one: false,
            inclusive: false,
            end_adjusted: false,
            start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            end: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cursor_start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            line_count: 0,
            empty: false,
            is_VIsual: false,
            start_vcol: 0,
            end_vcol: 0,
            prev_opcount: 0,
            prev_count0: 0,
            excl_tr_ws: false,
        };
        if findpar(
            &raw mut oa.inclusive,
            if name == '}' as c_int {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            },
            1,
            NUL,
            false,
        ) {
            mark = pos_to_mark(buf, ptr::null_mut(), (*win).w_cursor);
        }
    } else if name == '(' as c_int || name == ')' as c_int {
        if findsent(
            (if name == ')' as c_int {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            }) as Direction,
            1,
        ) != 0
        {
            mark = pos_to_mark(buf, ptr::null_mut(), (*win).w_cursor);
        }
    }
    (*curwin.get()).w_cursor = pos;
    listcmd_busy.set(slcb);
    return mark;
}

/// Get visual marks '<', '>'
///
/// This marks are different to normal marks:
/// 1. Never adjusted.
/// 2. Different behavior depending on editor state (visual mode).
/// 3. Not saved in shada.
/// 4. Re-ordered when defined in reverse.
/// `buf` — Buffer to get the mark from.
/// `name` — Mark name '<' or '>'.
///
/// @return[static]  Mark
pub unsafe fn mark_get_visual(mut buf: *mut buf_T, mut name: c_int) -> *mut fmark_T {
    let mut mark: *mut fmark_T = ptr::null_mut();
    if name == '<' as c_int || name == '>' as c_int {
        let mut startp: pos_T = (*buf).b_visual.vi_start;
        let mut endp: pos_T = (*buf).b_visual.vi_end;
        if ((name == '<' as c_int) as c_int == lt(startp, endp) as c_int || endp.lnum == 0)
            && startp.lnum != 0
        {
            mark = pos_to_mark(buf, ptr::null_mut(), startp);
        } else {
            mark = pos_to_mark(buf, ptr::null_mut(), endp);
        }
        if (*buf).b_visual.vi_mode == 'V' as c_int {
            if name == '<' as c_int {
                (*mark).mark.col = 0;
            } else {
                (*mark).mark.col = MAXCOL as c_int as colnr_T;
            }
            (*mark).mark.coladd = 0;
        }
    }
    return mark;
}

/// Search for the next named mark in the current file from a start position.
///
/// `startpos` — where to start.
/// `dir` — direction for search.
///
/// Returns next mark or NULL if no mark is found.
pub unsafe fn getnextmark(
    mut startpos: *mut pos_T,
    mut dir: c_int,
    mut begin_line: c_int,
) -> *mut fmark_T {
    let mut result: *mut fmark_T = ptr::null_mut();
    let mut pos: pos_T = *startpos;
    if dir == BACKWARD as c_int && begin_line != 0 {
        pos.col = 0;
    } else if dir == FORWARD as c_int && begin_line != 0 {
        pos.col = MAXCOL as c_int as colnr_T;
    }
    let mut i: c_int = 0;
    while i < NMARKS {
        if (*curbuf.get()).b_namedm[i as usize].mark.lnum > 0 {
            if dir == FORWARD as c_int {
                if (result.is_null()
                    || lt((*curbuf.get()).b_namedm[i as usize].mark, (*result).mark) as c_int != 0)
                    && lt(pos, (*curbuf.get()).b_namedm[i as usize].mark)
                {
                    result = (&raw mut (*curbuf.get()).b_namedm as *mut fmark_T).offset(i as isize);
                }
            } else if (result.is_null()
                || lt((*result).mark, (*curbuf.get()).b_namedm[i as usize].mark) as c_int != 0)
                && lt((*curbuf.get()).b_namedm[i as usize].mark, pos)
            {
                result = (&raw mut (*curbuf.get()).b_namedm as *mut fmark_T).offset(i as isize);
            }
        }
        i += 1;
    }
    return result;
}

/// Move to the given file mark, changing the buffer and cursor position.
///
/// Validate the mark, switch to the buffer, and move the cursor.
/// `fm` — Mark, can be NULL will raise E78: Unknown mark
/// `flags` — MarkMove flags to configure the movement to the mark.
///
/// Returns markMovekRes flags representing the outcome
pub unsafe fn mark_move_to(mut fm: *mut fmark_T, mut flags: MarkMove) -> MarkMoveRes {
    let mut prev_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    static fm_copy: GlobalCell<fmark_T> = GlobalCell::new(fmark_T {
        mark: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        fnum: 0,
        timestamp: 0 as Timestamp,
        view: fmarkv_T {
            topline_offset: MAXLNUM as c_int,
            skipcol: 0,
        },
        additional_data: ptr::null_mut(),
    });
    let mut res: MarkMoveRes = kMarkMoveSuccess;
    let mut errormsg: *const c_char = ptr::null();
    '_end: {
        if !mark_check(fm, &raw mut errormsg) {
            if !errormsg.is_null() {
                emsg(errormsg);
            }
            res = kMarkMoveFailed;
        } else {
            if (*fm).fnum != (*curbuf.get()).handle {
                fm_copy.set(*fm);
                fm = fm_copy.ptr();
                res = (res as c_uint
                    | switch_to_mark_buf(
                        fm,
                        flags as c_uint & kMarkJumpList as c_int as c_uint == 0,
                    ) as c_uint) as MarkMoveRes;
                if res as c_uint & kMarkMoveFailed as c_int as c_uint != 0 {
                    break '_end;
                } else if !mark_check_line_bounds(curbuf.get(), fm, &raw mut errormsg) {
                    if !errormsg.is_null() {
                        emsg(errormsg);
                    }
                    res = (res as c_uint | kMarkMoveFailed as c_int as c_uint) as MarkMoveRes;
                    break '_end;
                }
            } else if flags as c_uint & kMarkContext as c_int as c_uint != 0 {
                setpcmark();
            }
            prev_pos = (*curwin.get()).w_cursor;
            pos = (*fm).mark;
            (*curwin.get()).w_cursor = (*fm).mark;
            if flags as c_uint & kMarkBeginLine as c_int as c_uint != 0 {
                beginline(BL_WHITE as c_int | BL_FIX as c_int);
            }
            res = (if prev_pos.lnum != pos.lnum {
                res as c_uint
                    | kMarkChangedLine as c_int as c_uint
                    | kMarkChangedCursor as c_int as c_uint
            } else {
                res as c_uint
            }) as MarkMoveRes;
            res = (if prev_pos.col != pos.col {
                res as c_uint
                    | kMarkChangedCol as c_int as c_uint
                    | kMarkChangedCursor as c_int as c_uint
            } else {
                res as c_uint
            }) as MarkMoveRes;
            if flags as c_uint & kMarkSetView as c_int as c_uint != 0 {
                mark_view_restore(fm);
            }
            if res as c_uint & kMarkSwitchedBuf as c_int as c_uint != 0
                || res as c_uint & kMarkChangedCursor as c_int as c_uint != 0
            {
                check_cursor(curwin.get());
            }
        }
    }
    return res;
}

/// Attempt to switch to the buffer of the given global mark
///
/// `fm`
/// `pcmark_on_switch` — leave a context mark when switching buffer.
/// Returns whether the buffer was switched or not.
pub(super) unsafe fn switch_to_mark_buf(
    mut fm: *mut fmark_T,
    mut pcmark_on_switch: bool,
) -> MarkMoveRes {
    if (*fm).fnum != (*curbuf.get()).handle {
        let mut getfile_flag: c_int = if pcmark_on_switch {
            GETF_SETMARK as c_int
        } else {
            0
        };
        let mut res: bool =
            buflist_getfile((*fm).fnum, (*fm).mark.lnum, getfile_flag, false_0) == OK;
        return (if res as c_int == true_0 {
            kMarkSwitchedBuf as c_int
        } else {
            kMarkMoveFailed as c_int
        }) as MarkMoveRes;
    }
    return 0 as MarkMoveRes;
}

#[inline]
pub(super) fn mark_global_index(name: c_char) -> c_int {
    return if name as c_uint >= 'A' as c_uint && name as c_uint <= 'Z' as c_uint {
        name as c_int - 'A' as c_int
    } else if ascii_isdigit(name as c_int) {
        NMARKS + (name as c_int - '0' as c_int)
    } else {
        -1
    };
}
