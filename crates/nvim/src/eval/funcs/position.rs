//! Positions in a buffer: the cursor, `line()`, `col()`, `virtcol()`,
//! `getpos()`/`setpos()` and the character-search state.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use crate::cursor::check_cursor;
use crate::eval::typval::{
    NumBuf, tv_check_for_dict_arg, tv_check_for_opt_number_arg, tv_check_for_string_or_list_arg,
    tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc_ret, tv_dict_find, tv_get_bool, tv_get_lnum,
    tv_get_number, tv_get_number_chk, tv_list_alloc_ret, tv_list_append_number,
};
use crate::eval::window::{find_win_by_nr_or_id, win_and_tab_by_id};
use crate::eval::{buf_byteidx_to_charidx, buf_charidx_to_byteidx, list2fpos, var2fpos};
use crate::main::{curbuf, curwin, e_invarg, e_invarg2, p_spk, skip_update_topline};
use crate::mark::setmark_pos;
use crate::mbyte::{mb_adjust_cursor, utf_ptr2char, utfc_ptr2len};
use crate::memline::{ml_find_line_or_offset, ml_get_buf, ml_get_buf_len};
use crate::message::emsg;
use crate::r#move::{WinValid, update_curswant};
use crate::os::cshim::gettext;
use crate::plines::{getvvcol, win_chartabsize};
use crate::pos::MAXCOL;
use crate::search::{
    BACKWARD, FORWARD, last_csearch, last_csearch_forward, last_csearch_until,
    set_csearch_direction, set_csearch_until, set_last_csearch,
};
use crate::semsg_c;
use crate::state::virtual_active;
use crate::types::{
    Direction, EvalFuncData, FAIL, NUL, OK, VAR_LIST, VAR_NUMBER, VAR_STRING, buf_T, colnr_T,
    list_T, pos_T, typval_T, varnumber_T, win_T,
};
use crate::winlayer::Win;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// "End of line", the column sentinel. `MAXCOL` is spelled as an unsigned
/// constant but every column it is compared against is a `colnr_T`.
const END_OF_LINE: colnr_T = MAXCOL as colnr_T;

/// The zeroed position both the getters and the setters start from.
const NOWHERE: pos_T = pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
};

/// `byte2line({byte})` — which line a byte offset falls in.
pub unsafe fn f_byte2line(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: `args.ptr(0)` is a live typval and `curbuf` is the current
    // buffer; `boff` is a live local the callee reads and writes.
    unsafe {
        let mut boff = tv_get_number(args.ptr(0)) as c_int - 1;
        rettv.vval.v_number = if boff < 0 {
            -1
        } else {
            ml_find_line_or_offset(curbuf.get(), 0, &raw mut boff, false) as varnumber_T
        };
    }
}

/// `line2byte({lnum})` — the byte offset a line starts at, one-based, or -1
/// past the end. One past the last line is allowed: it is the buffer size.
pub unsafe fn f_line2byte(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: `args.ptr(0)` is a live typval and `curbuf` is the current
    // buffer.
    unsafe {
        let lnum = tv_get_lnum(args.ptr(0));
        rettv.vval.v_number = if lnum < 1 || lnum > (*curbuf.get()).b_ml.ml_line_count + 1 {
            -1
        } else {
            ml_find_line_or_offset(curbuf.get(), lnum, ptr::null_mut(), false) as varnumber_T
        };
        // The offset is zero-based inside memline and one-based here; -1
        // stays -1 because the bump only applies to a found offset.
        if rettv.vval.v_number >= 0 {
            rettv.vval.v_number += 1;
        }
    }
}

/// `col({expr} [, {winid}])`.
pub unsafe fn f_col(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { get_col(args, rettv, false) };
}

/// `charcol({expr} [, {winid}])` — as `col()` but counting characters.
pub unsafe fn f_charcol(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { get_col(args, rettv, true) };
}

/// The window argument `col()`, `charcol()` and `virtcol()` share: the
/// current window unless a window id names another, in which case its
/// cursor is validated first. `None` means the id named no window, which
/// every caller treats as "no answer".
///
/// # Safety
/// `args.ptr(idx)` is a live typval.
unsafe fn window_arg(args: Args<'_>, idx: usize) -> Option<*mut win_T> {
    // SAFETY: the caller's obligation; `tp` is a live local.
    unsafe {
        if !args.has(idx) {
            return Some(curwin.get());
        }
        let (wp, _) = win_and_tab_by_id(tv_get_number(args.ptr(idx)) as c_int)?;
        check_cursor(wp.raw());
        Some(wp.raw())
    }
}

/// # Safety
/// The arguments and `rettv` are live typvals.
unsafe fn get_col(args: Args<'_>, rettv: &mut typval_T, charcol: bool) {
    // SAFETY: the caller's obligation; `fnum` is a live local and
    // `var2fpos` hands back a pointer into the named window or buffer.
    unsafe {
        if tv_check_for_string_or_list_arg(args.ptr(0), 0) == FAIL
            || tv_check_for_opt_number_arg(args.ptr(0), 1) == FAIL
        {
            return;
        }
        let Some(wp) = window_arg(args, 1) else {
            return;
        };
        let bp = (*wp).w_buffer;
        let mut fnum = (*bp).handle as c_int;
        let fp = var2fpos(args.ptr(0), false, &raw mut fnum, charcol, wp);
        let mut col: colnr_T = 0;
        if let Some(mut fp) = fp
            && fnum == (*bp).handle
        {
            if fp.col == END_OF_LINE {
                // MAXCOL means "end of line"; past the last line there is
                // no line to measure, so it stays MAXCOL.
                col = if fp.lnum <= (*bp).b_ml.ml_line_count {
                    ml_get_buf_len(bp, fp.lnum) + 1
                } else {
                    END_OF_LINE
                };
            } else {
                col = fp.col + 1;
                col += virtualedit_tail(wp, bp, &raw mut fp);
            }
        }
        rettv.vval.v_number = col as varnumber_T;
    }
}

/// With 'virtualedit' on, a cursor sitting past the last character of the
/// line reports the column *after* it rather than on it — but only when it
/// is past the whole character, and only for the cursor itself.
///
/// Upstream tests `fp == &wp->w_cursor` for "the cursor itself", but
/// `var2fpos` — the only source of `fp` here — always answers a position of
/// its own, so the test never holds and the adjustment never applies. That
/// is preserved: `fp` is still an address so the comparison keeps its
/// (always false) answer. See F-P22-36.
///
/// # Safety
/// `wp`, `bp` and `fp` are live, and `fp` is a position in `bp`.
unsafe fn virtualedit_tail(wp: *mut win_T, bp: *mut buf_T, fp: *mut pos_T) -> colnr_T {
    // SAFETY: the caller's obligation; `p` points into the cursor's line
    // and is only walked forward by one character.
    unsafe {
        if !virtual_active(wp) || fp != &raw mut (*wp).w_cursor {
            return 0;
        }
        let p = ml_get_buf(bp, (*wp).w_cursor.lnum).offset((*wp).w_cursor.col as isize);
        if (*wp).w_cursor.coladd < win_chartabsize(wp, p, (*wp).w_virtcol - (*wp).w_cursor.coladd) {
            return 0;
        }
        // Only the last character of the line counts: the test is that the
        // byte after this character is the terminator.
        if *p == NUL as c_char {
            return 0;
        }
        let l = utfc_ptr2len(p);
        if *p.offset(l as isize) == NUL as c_char {
            l
        } else {
            0
        }
    }
}

/// `virtcol({expr} [, {list} [, {winid}]])`.
pub unsafe fn f_virtcol(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut vcol_start: colnr_T = 0;
    let mut vcol_end: colnr_T = 0;
    // SAFETY: the arguments and `rettv` are live typvals; `var2fpos` hands
    // back a pointer into the named window or buffer, which the clamp
    // below writes through — that is upstream's behaviour and is why a
    // position from a List argument is clamped in place.
    unsafe {
        // The window argument is only honoured when the `{list}` argument
        // was given too, because it is the third.
        let wp = if args.has(1) && args.has(2) {
            window_arg(args, 2)
        } else {
            Some(curwin.get())
        };
        if let Some(wp) = wp {
            let bp = (*wp).w_buffer;
            let mut fnum = (*bp).handle as c_int;
            let fp = var2fpos(args.ptr(0), false, &raw mut fnum, false, wp);
            if let Some(mut fp) = fp
                && fp.lnum <= (*bp).b_ml.ml_line_count
                && fnum == (*bp).handle
            {
                // Clamped before it is measured, as upstream clamps the
                // shared position it answered out of.
                if fp.col < 0 {
                    fp.col = 0;
                } else {
                    let len = ml_get_buf_len(bp, fp.lnum);
                    if fp.col > len {
                        fp.col = len;
                    }
                }
                getvvcol(
                    wp,
                    &raw mut fp,
                    &raw mut vcol_start,
                    ptr::null_mut(),
                    &raw mut vcol_end,
                );
                vcol_start += 1;
                vcol_end += 1;
            }
        }
        if args.has(1) && tv_get_bool(args.ptr(1)) != 0 {
            let l = tv_list_alloc_ret(rettv, 2);
            tv_list_append_number(l, vcol_start as varnumber_T);
            tv_list_append_number(l, vcol_end as varnumber_T);
        } else {
            rettv.vval.v_number = vcol_end as varnumber_T;
        }
    }
}

/// `line({expr} [, {winid}])`.
pub unsafe fn f_line(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals.
    let fp = unsafe {
        let mut fnum: c_int = 0;
        if !args.has(1) {
            var2fpos(args.ptr(0), true, &raw mut fnum, false, curwin.get())
        } else {
            match win_and_tab_by_id(tv_get_number(args.ptr(1)) as c_int) {
                None => None,
                Some((wp, _)) => {
                    let wp = wp.raw();
                    // Resolving a position in another window moves its
                    // cursor, and 'splitkeep' decides whether that is allowed
                    // to scroll it. Diff-mode windows are always exempt
                    // because their scroll is bound to this one's.
                    if *p_spk.get() != b'c' as c_char
                        || ((*wp).w_onebuf_opt.wo_diff != 0
                            && (*curwin.get()).w_onebuf_opt.wo_diff != 0)
                    {
                        skip_update_topline.set(true);
                    }
                    check_cursor(wp);
                    let fp = var2fpos(args.ptr(0), true, &raw mut fnum, false, wp);
                    skip_update_topline.set(false);
                    fp
                }
            }
        }
    };
    rettv.vval.v_number = fp.map_or(0, |fp| fp.lnum as varnumber_T);
}

/// `getpos({expr})`.
pub unsafe fn f_getpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { getpos_both(args, rettv, false, false) };
}

/// `getcharpos({expr})` — as `getpos()` but with a character column.
pub unsafe fn f_getcharpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { getpos_both(args, rettv, false, true) };
}

/// `getcurpos([{winid}])` — the cursor, plus a fifth 'curswant' element.
pub unsafe fn f_getcurpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { getpos_both(args, rettv, true, false) };
}

/// `getcursorcharpos([{winid}])`.
pub unsafe fn f_getcursorcharpos(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { getpos_both(args, rettv, true, true) };
}

/// The four getters' shared body. `getcurpos` takes the cursor of the
/// window its argument names rather than resolving a position expression,
/// and appends 'curswant'.
///
/// # Safety
/// The arguments and `rettv` are live typvals.
unsafe fn getpos_both(args: Args<'_>, rettv: &mut typval_T, getcurpos: bool, charcol: bool) {
    // SAFETY: the caller's obligation.
    unsafe {
        let mut wp = curwin.get();
        let mut fnum: c_int = -1;
        let fp = if !getcurpos {
            var2fpos(args.ptr(0), true, &raw mut fnum, charcol, curwin.get())
        } else {
            let mut fp = if args.has(0) {
                // `wp` is overwritten even when the lookup fails: a
                // `getcurpos()` on a window that does not exist answers 0
                // for 'curswant' rather than the current window's.
                wp = find_win_by_nr_or_id(args.ptr(0)).map_or(ptr::null_mut(), Win::raw);
                (!wp.is_null()).then(|| (*wp).w_cursor)
            } else {
                Some((*curwin.get()).w_cursor)
            };
            if let Some(pos) = &mut fp
                && charcol
            {
                pos.col = buf_byteidx_to_charidx((*wp).w_buffer, pos.lnum, pos.col) as colnr_T;
            }
            fp
        };

        let l = tv_list_alloc_ret(rettv, 4 + isize::from(getcurpos));
        tv_list_append_number(l, if fnum != -1 { fnum as varnumber_T } else { 0 });
        let (lnum, col, coladd) = fp.map_or((0, 0, 0), |fp| {
            // MAXCOL is passed through rather than made one-based.
            let col = if fp.col == END_OF_LINE {
                END_OF_LINE
            } else {
                fp.col + 1
            };
            (
                fp.lnum as varnumber_T,
                col as varnumber_T,
                fp.coladd as varnumber_T,
            )
        });
        tv_list_append_number(l, lnum);
        tv_list_append_number(l, col);
        tv_list_append_number(l, coladd);
        if getcurpos {
            append_curswant(l, wp);
        }
    }
}

/// `getcurpos()`'s fifth element. Reading it means recomputing 'curswant',
/// which is a side effect the caller must not see — so the three fields
/// that recomputation touches are put back, and the cached virtual column
/// invalidated so the next reader recomputes it properly.
///
/// # Safety
/// `l` is a live list and `wp` is a window pointer or null.
unsafe fn append_curswant(l: *mut list_T, wp: *mut win_T) {
    // SAFETY: the caller's obligation.
    unsafe {
        let cur = curwin.get();
        let saved_set_curswant = (*cur).w_set_curswant;
        let saved_curswant = (*cur).w_curswant;
        let saved_virtcol = (*cur).w_virtcol;
        if wp == cur {
            update_curswant();
        }
        tv_list_append_number(
            l,
            if wp.is_null() {
                0
            } else if (*wp).w_curswant == END_OF_LINE {
                MAXCOL as varnumber_T
            } else {
                (*wp).w_curswant as varnumber_T + 1
            },
        );
        // Only restored when 'curswant' was due to be recomputed anyway:
        // if it was already valid, `update_curswant` did not change it.
        if wp == cur && saved_set_curswant {
            (*cur).w_set_curswant = saved_set_curswant;
            (*cur).w_curswant = saved_curswant;
            (*cur).w_virtcol = saved_virtcol;
            (*cur).w_valid.clear(WinValid::VIRTCOL);
        }
    }
}

/// `cursor({lnum}, {col} [, {off}])` or `cursor({list})`.
pub unsafe fn f_cursor(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { set_cursorpos(args, rettv, false) };
}

/// `setcursorcharpos({lnum}, {col} [, {off}])` or with a List.
pub unsafe fn f_setcursorcharpos(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { set_cursorpos(args, rettv, true) };
}

/// # Safety
/// The arguments and `rettv` are live typvals.
unsafe fn set_cursorpos(args: Args<'_>, rettv: &mut typval_T, charcol: bool) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's obligation; `pos` and `curswant` are live
    // locals the List parser fills.
    unsafe {
        rettv.vval.v_number = -1;
        let mut set_curswant = true;
        let (lnum, mut col, coladd) = if args.ty(0) == VAR_LIST {
            let mut pos = NOWHERE;
            let mut curswant: colnr_T = -1;
            if list2fpos(
                args.ptr(0),
                &raw mut pos,
                ptr::null_mut(),
                &raw mut curswant,
                charcol,
            ) == FAIL
            {
                emsg(gettext(e_invarg.as_ptr()));
                return;
            }
            if curswant >= 0 {
                (*curwin.get()).w_curswant = curswant - 1;
                set_curswant = false;
            }
            (pos.lnum, pos.col, pos.coladd)
        } else if matches!(args.ty(0), VAR_NUMBER | VAR_STRING)
            && matches!(args.ty(1), VAR_NUMBER | VAR_STRING)
        {
            let mut lnum = tv_get_lnum(args.ptr(0));
            if lnum < 0 {
                // Kept on the variadic message call: the argument is
                // arbitrary user bytes. Note that this reports and then
                // carries on to the range check below.
                semsg_c!(gettext(e_invarg2.as_ptr()), numbuf.string(args.ptr(0)),);
            } else if lnum == 0 {
                lnum = (*curwin.get()).w_cursor.lnum;
            }
            let mut col = tv_get_number_chk(args.ptr(1), ptr::null_mut()) as colnr_T;
            if charcol {
                col = buf_charidx_to_byteidx(curbuf.get(), lnum, col) + 1;
            }
            let coladd = if args.has(2) {
                tv_get_number_chk(args.ptr(2), ptr::null_mut()) as colnr_T
            } else {
                0
            };
            (lnum, col, coladd)
        } else {
            emsg(gettext(e_invarg.as_ptr()));
            return;
        };

        if lnum < 0 || col < 0 || coladd < 0 {
            return;
        }
        if lnum > 0 {
            (*curwin.get()).w_cursor.lnum = lnum;
        }
        // The column is one-based on the way in, except for MAXCOL which
        // means "end of line" and is passed through.
        if col != END_OF_LINE {
            col = (col - 1).max(0);
        }
        (*curwin.get()).w_cursor.col = col;
        (*curwin.get()).w_cursor.coladd = coladd;
        check_cursor(curwin.get());
        mb_adjust_cursor();
        (*curwin.get()).w_set_curswant = set_curswant;
        rettv.vval.v_number = 0;
    }
}

/// `setpos({expr}, {list})`.
pub unsafe fn f_setpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { set_position(args, rettv, false) };
}

/// `setcharpos({expr}, {list})` — as `setpos()` with a character column.
pub unsafe fn f_setcharpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { set_position(args, rettv, true) };
}

/// # Safety
/// The arguments and `rettv` are live typvals.
unsafe fn set_position(args: Args<'_>, rettv: &mut typval_T, charpos: bool) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's obligation; `pos`, `fnum` and `curswant` are
    // live locals the List parser fills, and `name` is NUL-terminated.
    unsafe {
        rettv.vval.v_number = -1;
        let name = numbuf.string_chk(args.ptr(0));
        if name.is_null() {
            return;
        }
        let mut pos = NOWHERE;
        let mut fnum: c_int = 0;
        let mut curswant: colnr_T = -1;
        if list2fpos(
            args.ptr(1),
            &raw mut pos,
            &raw mut fnum,
            &raw mut curswant,
            charpos,
        ) != OK
        {
            return;
        }
        if pos.col != END_OF_LINE {
            pos.col = (pos.col - 1).max(0);
        }
        match CStr::from_ptr(name).to_bytes() {
            b"." => {
                (*curwin.get()).w_cursor = pos;
                if curswant >= 0 {
                    (*curwin.get()).w_curswant = curswant - 1;
                    (*curwin.get()).w_set_curswant = false;
                }
                check_cursor(curwin.get());
                rettv.vval.v_number = 0;
            }
            // A mark name is exactly one byte after the quote.
            [b'\'', c] => {
                if setmark_pos(*c as c_int, &raw mut pos, fnum, ptr::null_mut()) == OK {
                    rettv.vval.v_number = 0;
                }
            }
            _ => {
                emsg(gettext(e_invarg.as_ptr()));
            }
        }
    }
}

/// `getcharsearch()` — the state `;` and `,` repeat.
pub unsafe fn f_getcharsearch(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the dispatcher's cleared return value; the three
    // readers answer from the process-wide character-search state.
    let csearch = last_csearch();
    unsafe {
        tv_dict_alloc_ret(rettv);
        let dict = (*rettv).vval.v_dict;
        tv_dict_add_str(dict, c"char".as_ptr(), 4, csearch.as_ptr());
        tv_dict_add_nr(
            dict,
            c"forward".as_ptr(),
            7,
            last_csearch_forward() as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            c"until".as_ptr(),
            5,
            last_csearch_until() as varnumber_T,
        );
    }
}

/// `setcharsearch({dict})` — each key is optional and missing keys leave
/// that part of the state alone.
pub unsafe fn f_setcharsearch(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, _rettv) = frame!(argvars, _rettv);
    // SAFETY: `args.ptr(0)` is a live typval; after the check the union
    // holds a Dict pointer, which may still be null.
    unsafe {
        if tv_check_for_dict_arg(args.ptr(0), 0) == FAIL {
            return;
        }
        let d = args.get(0).vval.v_dict;
        if d.is_null() {
            return;
        }
        let csearch = numbuf.dict_string(d, c"char".as_ptr());
        if !csearch.is_null() {
            set_last_csearch(utf_ptr2char(csearch), csearch, utfc_ptr2len(csearch));
        }
        let di = tv_dict_find(d, c"forward".as_ptr(), 7);
        if !di.is_null() {
            let forward = tv_get_number(&raw mut (*di).di_tv) != 0;
            set_csearch_direction(if forward { FORWARD } else { BACKWARD } as Direction);
        }
        let di = tv_dict_find(d, c"until".as_ptr(), 5);
        if !di.is_null() {
            set_csearch_until((tv_get_number(&raw mut (*di).di_tv) != 0) as c_int);
        }
    }
}
