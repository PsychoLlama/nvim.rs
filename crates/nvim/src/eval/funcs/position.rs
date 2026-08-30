//! Positions in a buffer: the cursor, `line()`, `col()`, `virtcol()`,
//! `getpos()`/`setpos()` and the character-search state.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use super::wrappers::{
    arg_bool, arg_lnum, arg_number, arg_number_chk, arg_string, arg_string_chk, check_arg,
    list_alloc_ret,
};
use crate::cursor::check_cursor;
use crate::eval::typval::{
    NumBuf, tv_check_for_dict_arg, tv_check_for_opt_number_arg, tv_check_for_string_or_list_arg,
    tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc_ret, tv_dict_find, tv_get_number,
    tv_list_append_number,
};
use crate::eval::window::{find_win_by_nr_or_id, win_and_tab_by_id};
use crate::eval::{buf_byteidx_to_charidx, buf_charidx_to_byteidx, list2fpos, var2fpos};
use crate::main::{curbuf, curwin, e_invarg, p_spk, skip_update_topline};
use crate::mark::setmark_pos;
use crate::mbyte::{mb_adjust_cursor, utf_ptr2char, utfc_ptr2len};
use crate::memline::{ml_find_line_or_offset, ml_get_buf, ml_get_buf_len};
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::r#move::{WinValid, update_curswant};
use crate::os::cshim::gettext;
use crate::plines::{getvvcol, win_chartabsize};
use crate::pos::MAXCOL;
use crate::search::{
    BACKWARD, FORWARD, last_csearch, last_csearch_forward, last_csearch_until,
    set_csearch_direction, set_csearch_until, set_last_csearch,
};
use crate::semsg;
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
    // SAFETY throughout: `args.ptr(0)` is a live typval and `curbuf` is the current
    // buffer; `boff` is a live local the callee reads and writes.
    let mut boff = arg_number(args.get(0)) as c_int - 1;
    rettv.vval.v_number = if boff < 0 {
        -1
    } else {
        unsafe { ml_find_line_or_offset(curbuf.get(), 0, &raw mut boff, false) as varnumber_T }
    };
}

/// `line2byte({lnum})` — the byte offset a line starts at, one-based, or -1
/// past the end. One past the last line is allowed: it is the buffer size.
pub unsafe fn f_line2byte(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY throughout: `args.ptr(0)` is a live typval and `curbuf` is the current
    // buffer.
    let lnum = arg_lnum(args.get(0));
    rettv.vval.v_number = if lnum < 1 || lnum > unsafe { (*curbuf.get()).b_ml.ml_line_count } + 1 {
        -1
    } else {
        unsafe { ml_find_line_or_offset(curbuf.get(), lnum, ptr::null_mut(), false) as varnumber_T }
    };
    // The offset is zero-based inside memline and one-based here; -1
    // stays -1 because the bump only applies to a found offset.
    if unsafe { rettv.vval.v_number } >= 0 {
        unsafe { rettv.vval.v_number += 1 };
    }
}

/// `col({expr} [, {winid}])`.
pub unsafe fn f_col(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    get_col(args, rettv, false);
}

/// `charcol({expr} [, {winid}])` — as `col()` but counting characters.
pub unsafe fn f_charcol(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    get_col(args, rettv, true);
}

/// The window argument `col()`, `charcol()` and `virtcol()` share: the
/// current window unless a window id names another, in which case its
/// cursor is validated first. `None` means the id named no window, which
/// every caller treats as "no answer".
fn window_arg(args: Args<'_>, idx: usize) -> Option<*mut win_T> {
    if !args.has(idx) {
        return Some(curwin.get());
    }
    let (wp, _) = win_and_tab_by_id(arg_number(args.get(idx)) as c_int)?;
    check_cursor(wp);
    Some(wp.raw())
}

fn get_col(args: Args<'_>, rettv: &mut typval_T, charcol: bool) {
    // SAFETY throughout: `fnum` is a live local and
    // `var2fpos` hands back a pointer into the named window or buffer.
    if check_arg(args, 0, tv_check_for_string_or_list_arg).is_err()
        || check_arg(args, 1, tv_check_for_opt_number_arg).is_err()
    {
        return;
    }
    let Some(wp) = window_arg(args, 1) else {
        return;
    };
    let bp = unsafe { (*wp).w_buffer };
    let mut fnum = unsafe { (*bp).handle } as c_int;
    let fp = unsafe { var2fpos(args.ptr(0), false, &raw mut fnum, charcol, wp) };
    let mut col: colnr_T = 0;
    if let Some(mut fp) = fp
        && fnum == unsafe { (*bp).handle }
    {
        if fp.col == END_OF_LINE {
            // MAXCOL means "end of line"; past the last line there is
            // no line to measure, so it stays MAXCOL.
            col = if fp.lnum <= unsafe { (*bp).b_ml.ml_line_count } {
                (unsafe { ml_get_buf_len(bp, fp.lnum) }) + 1
            } else {
                END_OF_LINE
            };
        } else {
            col = fp.col + 1;
            col += unsafe { virtualedit_tail(wp, bp, &raw mut fp) };
        }
    }
    rettv.vval.v_number = col as varnumber_T;
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
    // SAFETY: the caller's promise, taken once for the whole body.
    let mut win = unsafe { Win::new(wp) };
    // SAFETY throughout: the caller's obligation; `p` points into the cursor's line
    // and is only walked forward by one character.
    if !virtual_active(win) || fp != &raw mut win.w_cursor {
        return 0;
    }
    let p = unsafe { ml_get_buf(bp, win.w_cursor.lnum).offset(win.w_cursor.col as isize) };
    if win.w_cursor.coladd < unsafe { win_chartabsize(win, p, win.w_virtcol - win.w_cursor.coladd) }
    {
        return 0;
    }
    // Only the last character of the line counts: the test is that the
    // byte after this character is the terminator.
    if unsafe { *p } == NUL as c_char {
        return 0;
    }
    let l = unsafe { utfc_ptr2len(p) };
    if unsafe { *p.offset(l as isize) } == NUL as c_char {
        l
    } else {
        0
    }
}

/// `virtcol({expr} [, {list} [, {winid}]])`.
pub unsafe fn f_virtcol(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut vcol_start: colnr_T = 0;
    let mut vcol_end: colnr_T = 0;
    // SAFETY throughout: the arguments and `rettv` are live typvals; `var2fpos` hands
    // back a pointer into the named window or buffer, which the clamp
    // below writes through — that is upstream's behaviour and is why a
    // position from a List argument is clamped in place.
    // The window argument is only honoured when the `{list}` argument
    // was given too, because it is the third.
    let wp = if args.has(1) && args.has(2) {
        window_arg(args, 2)
    } else {
        Some(curwin.get())
    };
    if let Some(wp) = wp {
        let bp = unsafe { (*wp).w_buffer };
        let mut fnum = unsafe { (*bp).handle } as c_int;
        let fp = unsafe { var2fpos(args.ptr(0), false, &raw mut fnum, false, wp) };
        if let Some(mut fp) = fp
            && fp.lnum <= unsafe { (*bp).b_ml.ml_line_count }
            && fnum == unsafe { (*bp).handle }
        {
            // Clamped before it is measured, as upstream clamps the
            // shared position it answered out of.
            if fp.col < 0 {
                fp.col = 0;
            } else {
                let len = unsafe { ml_get_buf_len(bp, fp.lnum) };
                if fp.col > len {
                    fp.col = len;
                }
            }
            let (pos, start, end) = (&raw mut fp, &raw mut vcol_start, &raw mut vcol_end);
            // SAFETY: `wp` is the window resolved above and the three
            // out-parameters are locals.
            unsafe { getvvcol(Win::new(wp), pos, start, ptr::null_mut(), end) };
            vcol_start += 1;
            vcol_end += 1;
        }
    }
    if args.has(1) && arg_bool(args.get(1)) != 0 {
        let l = list_alloc_ret(rettv, 2);
        unsafe { tv_list_append_number(l, vcol_start as varnumber_T) };
        unsafe { tv_list_append_number(l, vcol_end as varnumber_T) };
    } else {
        rettv.vval.v_number = vcol_end as varnumber_T;
    }
}

/// `line({expr} [, {winid}])`.
pub unsafe fn f_line(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut fnum: c_int = 0;
    let out = &raw mut fnum;
    let fp = if !args.has(1) {
        // SAFETY: argument 0 is a live typval and `curwin` a live window.
        unsafe { var2fpos(args.ptr(0), true, out, false, curwin.get()) }
    } else {
        match win_and_tab_by_id(arg_number(args.get(1)) as c_int) {
            None => None,
            Some((wp, _)) => {
                let wp = wp.raw();
                // Resolving a position in another window moves its cursor,
                // and 'splitkeep' decides whether that is allowed to scroll
                // it. Diff-mode windows are always exempt because their
                // scroll is bound to this one's.
                // SAFETY: `wp` is the window the id resolved to, and
                // `curwin` is live.
                let both_diff = unsafe { (*wp).w_onebuf_opt.wo_diff } != 0
                    && unsafe { (*curwin.get()).w_onebuf_opt.wo_diff } != 0;
                if unsafe { *p_spk.get() } != b'c' as c_char || both_diff {
                    skip_update_topline.set(true);
                }
                // SAFETY: `wp` is the window the id resolved to.
                check_cursor(unsafe { Win::new(wp) });
                let fp = unsafe { var2fpos(args.ptr(0), true, out, false, wp) };
                skip_update_topline.set(false);
                fp
            }
        }
    };
    rettv.vval.v_number = fp.map_or(0, |fp| fp.lnum as varnumber_T);
}

/// `getpos({expr})`.
pub unsafe fn f_getpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    getpos_both(args, rettv, false, false);
}

/// `getcharpos({expr})` — as `getpos()` but with a character column.
pub unsafe fn f_getcharpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    getpos_both(args, rettv, false, true);
}

/// `getcurpos([{winid}])` — the cursor, plus a fifth 'curswant' element.
pub unsafe fn f_getcurpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    getpos_both(args, rettv, true, false);
}

/// `getcursorcharpos([{winid}])`.
pub unsafe fn f_getcursorcharpos(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    getpos_both(args, rettv, true, true);
}

/// The four getters' shared body. `getcurpos` takes the cursor of the
/// window its argument names rather than resolving a position expression,
/// and appends 'curswant'.
fn getpos_both(args: Args<'_>, rettv: &mut typval_T, getcurpos: bool, charcol: bool) {
    // SAFETY throughout: `curwin` names a live window, and every pointer
    // read below comes back from the position parser.
    let mut wp = curwin.get();
    let mut fnum: c_int = -1;
    let fp = if !getcurpos {
        unsafe { var2fpos(args.ptr(0), true, &raw mut fnum, charcol, curwin.get()) }
    } else {
        let mut fp = if args.has(0) {
            // `wp` is overwritten even when the lookup fails: a
            // `getcurpos()` on a window that does not exist answers 0
            // for 'curswant' rather than the current window's.
            wp = unsafe { find_win_by_nr_or_id(args.ptr(0)) }.map_or(ptr::null_mut(), Win::raw);
            (!wp.is_null()).then(|| unsafe { (*wp).w_cursor })
        } else {
            Some(unsafe { (*curwin.get()).w_cursor })
        };
        if let Some(pos) = &mut fp
            && charcol
        {
            pos.col =
                unsafe { buf_byteidx_to_charidx((*wp).w_buffer, pos.lnum, pos.col) } as colnr_T;
        }
        fp
    };

    let l = list_alloc_ret(rettv, 4 + isize::from(getcurpos));
    unsafe { tv_list_append_number(l, if fnum != -1 { fnum as varnumber_T } else { 0 }) };
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
    unsafe { tv_list_append_number(l, lnum) };
    unsafe { tv_list_append_number(l, col) };
    unsafe { tv_list_append_number(l, coladd) };
    if getcurpos {
        unsafe { append_curswant(l, wp) };
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
    // SAFETY throughout: the caller's obligation.
    let cur = curwin.get();
    let saved_set_curswant = unsafe { (*cur).w_set_curswant };
    let saved_curswant = unsafe { (*cur).w_curswant };
    let saved_virtcol = unsafe { (*cur).w_virtcol };
    if wp == cur {
        unsafe { update_curswant() };
    }
    // SAFETY throughout: `wp` is null or the window resolved above, and `l` the list
    // being filled in.
    let curswant = if wp.is_null() {
        0
    } else if unsafe { (*wp).w_curswant } == END_OF_LINE {
        MAXCOL as varnumber_T
    } else {
        (unsafe { (*wp).w_curswant }) as varnumber_T + 1
    };
    unsafe { tv_list_append_number(l, curswant) };
    // Only restored when 'curswant' was due to be recomputed anyway:
    // if it was already valid, `update_curswant` did not change it.
    if wp == cur && saved_set_curswant {
        unsafe { (*cur).w_set_curswant = saved_set_curswant };
        unsafe { (*cur).w_curswant = saved_curswant };
        unsafe { (*cur).w_virtcol = saved_virtcol };
        unsafe { (*cur).w_valid.clear(WinValid::VIRTCOL) };
    }
}

/// `cursor({lnum}, {col} [, {off}])` or `cursor({list})`.
pub unsafe fn f_cursor(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    set_cursorpos(args, rettv, false);
}

/// `setcursorcharpos({lnum}, {col} [, {off}])` or with a List.
pub unsafe fn f_setcursorcharpos(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    set_cursorpos(args, rettv, true);
}

fn set_cursorpos(args: Args<'_>, rettv: &mut typval_T, charcol: bool) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: `pos` and `curswant` are live
    // locals the List parser fills.
    rettv.vval.v_number = -1;
    let mut set_curswant = true;
    let (lnum, mut col, coladd) = if args.ty(0) == VAR_LIST {
        let mut pos = NOWHERE;
        let mut curswant: colnr_T = -1;
        let (out, want) = (&raw mut pos, &raw mut curswant);
        // SAFETY: argument 0 is a live typval and both are locals.
        let read = unsafe { list2fpos(args.ptr(0), out, ptr::null_mut(), want, charcol) };
        if read == FAIL {
            emsg(gettext(e_invarg));
            return;
        }
        if curswant >= 0 {
            unsafe { (*curwin.get()).w_curswant = curswant - 1 };
            set_curswant = false;
        }
        (pos.lnum, pos.col, pos.coladd)
    } else if matches!(args.ty(0), VAR_NUMBER | VAR_STRING)
        && matches!(args.ty(1), VAR_NUMBER | VAR_STRING)
    {
        let mut lnum = arg_lnum(args.get(0));
        if lnum < 0 {
            // Kept on the variadic message call: the argument is
            // arbitrary user bytes. Note that this reports and then
            // carries on to the range check below.
            let what = arg_string(&mut numbuf, args.get(0));
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let what = unsafe { c_str(what) };
            semsg!("E475: Invalid argument: {what}");
        } else if lnum == 0 {
            lnum = unsafe { (*curwin.get()).w_cursor.lnum };
        }
        let mut col = arg_number_chk(args.get(1), None) as colnr_T;
        if charcol {
            col = unsafe { buf_charidx_to_byteidx(curbuf.get(), lnum, col) } + 1;
        }
        let coladd = if args.has(2) {
            arg_number_chk(args.get(2), None) as colnr_T
        } else {
            0
        };
        (lnum, col, coladd)
    } else {
        emsg(gettext(e_invarg));
        return;
    };

    if lnum < 0 || col < 0 || coladd < 0 {
        return;
    }
    if lnum > 0 {
        unsafe { (*curwin.get()).w_cursor.lnum = lnum };
    }
    // The column is one-based on the way in, except for MAXCOL which
    // means "end of line" and is passed through.
    if col != END_OF_LINE {
        col = (col - 1).max(0);
    }
    unsafe { (*curwin.get()).w_cursor.col = col };
    unsafe { (*curwin.get()).w_cursor.coladd = coladd };
    check_cursor(unsafe { Win::current() });
    unsafe { mb_adjust_cursor() };
    unsafe { (*curwin.get()).w_set_curswant = set_curswant };
    rettv.vval.v_number = 0;
}

/// `setpos({expr}, {list})`.
pub unsafe fn f_setpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    set_position(args, rettv, false);
}

/// `setcharpos({expr}, {list})` — as `setpos()` with a character column.
pub unsafe fn f_setcharpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    set_position(args, rettv, true);
}

fn set_position(args: Args<'_>, rettv: &mut typval_T, charpos: bool) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: `pos`, `fnum` and `curswant` are
    // live locals the List parser fills, and `name` is NUL-terminated.
    rettv.vval.v_number = -1;
    let name = arg_string_chk(&mut numbuf, args.get(0));
    if name.is_null() {
        return;
    }
    let mut pos = NOWHERE;
    let mut fnum: c_int = 0;
    let mut curswant: colnr_T = -1;
    let (out, buf, want) = (&raw mut pos, &raw mut fnum, &raw mut curswant);
    // SAFETY: argument 1 is a live typval and the three are locals.
    if unsafe { list2fpos(args.ptr(1), out, buf, want, charpos) } != OK {
        return;
    }
    if pos.col != END_OF_LINE {
        pos.col = (pos.col - 1).max(0);
    }
    match unsafe { CStr::from_ptr(name) }.to_bytes() {
        b"." => {
            unsafe { (*curwin.get()).w_cursor = pos };
            if curswant >= 0 {
                unsafe { (*curwin.get()).w_curswant = curswant - 1 };
                unsafe { (*curwin.get()).w_set_curswant = false };
            }
            check_cursor(unsafe { Win::current() });
            rettv.vval.v_number = 0;
        }
        // A mark name is exactly one byte after the quote.
        [b'\'', c] => {
            if unsafe { setmark_pos(*c as c_int, &raw mut pos, fnum, ptr::null_mut()) } == OK {
                rettv.vval.v_number = 0;
            }
        }
        _ => {
            emsg(gettext(e_invarg));
        }
    }
}

/// `getcharsearch()` — the state `;` and `,` repeat.
pub unsafe fn f_getcharsearch(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY throughout: `rettv` is the dispatcher's cleared return value; the three
    // readers answer from the process-wide character-search state.
    let csearch = last_csearch();
    unsafe { tv_dict_alloc_ret(rettv) };
    let dict = unsafe { (*rettv).vval.v_dict };
    let _ = unsafe { tv_dict_add_str(dict, c"char".as_ptr(), 4, csearch.as_ptr()) };
    let forward = last_csearch_forward() as varnumber_T;
    let _ = unsafe { tv_dict_add_nr(dict, c"forward".as_ptr(), 7, forward) };
    let until = last_csearch_until() as varnumber_T;
    let _ = unsafe { tv_dict_add_nr(dict, c"until".as_ptr(), 5, until) };
}

/// `setcharsearch({dict})` — each key is optional and missing keys leave
/// that part of the state alone.
pub unsafe fn f_setcharsearch(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, _rettv) = frame!(argvars, _rettv);
    // SAFETY throughout: `args.ptr(0)` is a live typval; after the check the union
    // holds a Dict pointer, which may still be null.
    if check_arg(args, 0, tv_check_for_dict_arg).is_err() {
        return;
    }
    let d = unsafe { args.get(0).vval.v_dict };
    if d.is_null() {
        return;
    }
    let csearch = unsafe { numbuf.dict_string(d, c"char".as_ptr()) };
    if !csearch.is_null() {
        unsafe { set_last_csearch(utf_ptr2char(csearch), csearch, utfc_ptr2len(csearch)) };
    }
    let di = unsafe { tv_dict_find(d, c"forward".as_ptr(), 7) };
    if !di.is_null() {
        let forward = unsafe { tv_get_number(&raw mut (*di).di_tv) } != 0;
        set_csearch_direction(if forward { FORWARD } else { BACKWARD } as Direction);
    }
    let di = unsafe { tv_dict_find(d, c"until".as_ptr(), 5) };
    if !di.is_null() {
        set_csearch_until((unsafe { tv_get_number(&raw mut (*di).di_tv) } != 0) as c_int);
    }
}
