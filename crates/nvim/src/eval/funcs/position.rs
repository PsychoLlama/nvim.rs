//! Positions in a buffer: the cursor, `line()`, `col()`, `virtcol()`,
//! `getpos()`/`setpos()` and the character-search state.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::wrappers::{
    arg_bool, arg_lnum, arg_number, arg_number_chk, arg_string, arg_string_chk, list_alloc_ret,
};
use crate::cursor::check_cursor;
use crate::eval::typval::{
    NumBuf, tv_check_for_dict_arg, tv_check_for_opt_number_arg, tv_check_for_string_or_list_arg,
    tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc_ret, tv_dict_find, tv_get_number,
    tv_list_append_number,
};
use crate::eval::window::{find_win_by_nr_or_id, win_and_tab_by_id};
use crate::eval::{buf_byteidx_to_charidx, buf_charidx_to_byteidx, list2fpos, var2fpos};
use crate::mark::setmark_pos;
use crate::mbyte::{mb_adjust_cursor, utf_ptr2char, utfc_ptr2len};
use crate::memline::{ml_find_line_or_offset, ml_get_buf, ml_get_buf_len};
use crate::message::e_invarg;
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::r#move::{WinValid, update_curswant};
use crate::option::vars::p_spk;
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
    ColNr, Direction, EvalFuncData, List, NUL, Pos, TypVal, VAR_LIST, VAR_NUMBER, VAR_STRING,
    VarNumber,
};
use crate::window::state::skip_update_topline;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// "End of line", the column sentinel. `MAXCOL` is spelled as an unsigned
/// constant but every column it is compared against is a `ColNr`.
const END_OF_LINE: ColNr = MAXCOL as ColNr;

/// The zeroed position both the getters and the setters start from.
const NOWHERE: Pos = Pos {
    lnum: 0,
    col: 0,
    coladd: 0,
};

/// `byte2line({byte})` — which line a byte offset falls in.
pub fn f_byte2line(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: `&args[0]` is a live typval and `curbuf` is the current
    // buffer; `boff` is a live local the callee reads and writes.
    let mut boff = arg_number(&args[0]) as c_int - 1;
    result.write_number(if boff < 0 {
        -1
    } else {
        unsafe { ml_find_line_or_offset(Buf::current(), 0, &raw mut boff, false) as VarNumber }
    });
}

/// `line2byte({lnum})` — the byte offset a line starts at, one-based, or -1
/// past the end. One past the last line is allowed: it is the buffer size.
pub fn f_line2byte(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: `&args[0]` is a live typval and `curbuf` is the current
    // buffer.
    let lnum = arg_lnum(&args[0]);
    let offset = if lnum < 1 || lnum > Buf::current().b_ml.ml_line_count + 1 {
        -1
    } else {
        unsafe { ml_find_line_or_offset(Buf::current(), lnum, ptr::null_mut(), false) as VarNumber }
    };
    result.write_number(offset);
    // The offset is zero-based inside memline and one-based here; -1
    // stays -1 because the bump only applies to a found offset.
    if offset >= 0 {
        result.write_number(offset + 1);
    }
}

/// `col({expr} [, {winid}])`.
pub fn f_col(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    get_col(args, result, false);
}

/// `charcol({expr} [, {winid}])` — as `col()` but counting characters.
pub fn f_charcol(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    get_col(args, result, true);
}

/// The window argument `col()`, `charcol()` and `virtcol()` share: the
/// current window unless a window id names another, in which case its
/// cursor is validated first. `None` means the id named no window, which
/// every caller treats as "no answer".
fn window_arg(args: &[TypVal], idx: usize) -> Option<Option<Win>> {
    if args.len() <= idx {
        return Some(Win::current_or_none());
    }
    let (wp, _) = win_and_tab_by_id(arg_number(&args[idx]) as c_int)?;
    check_cursor(wp);
    Some(Some(wp))
}

fn get_col(args: &[TypVal], result: &mut TypVal, charcol: bool) {
    // SAFETY throughout: `fnum` is a live local and
    // `var2fpos` hands back a pointer into the named window or buffer.
    if tv_check_for_string_or_list_arg(args, 0).is_err()
        || tv_check_for_opt_number_arg(args, 1).is_err()
    {
        return;
    }
    let Some(wp) = window_arg(args, 1) else {
        return;
    };
    let wp = wp.expect("a window for the column lookup");
    let bp = wp.buffer();
    let mut fnum = bp.handle as c_int;
    let fp = unsafe { var2fpos(&args[0], false, &raw mut fnum, charcol, wp) };
    let mut col: ColNr = 0;
    if let Some(mut fp) = fp
        && fnum == bp.handle
    {
        if fp.col == END_OF_LINE {
            // MAXCOL means "end of line"; past the last line there is
            // no line to measure, so it stays MAXCOL.
            col = if fp.lnum <= bp.b_ml.ml_line_count {
                (unsafe { ml_get_buf_len(bp, fp.lnum) }) + 1
            } else {
                END_OF_LINE
            };
        } else {
            col = fp.col + 1;
            col += unsafe { virtualedit_tail(wp, bp, &raw mut fp) };
        }
    }
    result.write_number(col as VarNumber);
}

/// With 'virtualedit' on, a cursor sitting past the last character of the
/// line reports the column *after* it rather than on it — but only when it
/// is past the whole character, and only for the cursor itself.
///
/// Upstream tests `fp == &wp->w_cursor` for "the cursor itself", but
/// `var2fpos` — the only source of `pos` here — always answers a position of
/// its own, so the test never holds and the adjustment never applies. That
/// is preserved: `pos` is still an address so the comparison keeps its
/// (always false) answer. See F-P22-36.
///
/// # Safety
/// `window`, `bp` and `pos` are live, and `pos` is a position in `bp`.
unsafe fn virtualedit_tail(mut win: Win, buffer: Buf, pos: *mut Pos) -> ColNr {
    // SAFETY: the caller's promise, taken once for the whole body.
    // SAFETY throughout: the caller's obligation; `p` points into the cursor's line
    // and is only walked forward by one character.
    if !virtual_active(win) || pos != &raw mut win.w_cursor {
        return 0;
    }
    let p = unsafe { ml_get_buf(buffer, win.w_cursor.lnum).offset(win.w_cursor.col as isize) };
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
pub fn f_virtcol(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut vcol_start: ColNr = 0;
    let mut vcol_end: ColNr = 0;
    // SAFETY throughout: the arguments and `result` are live typvals; `var2fpos` hands
    // back a pointer into the named window or buffer, which the clamp
    // below writes through — that is upstream's behaviour and is why a
    // position from a List argument is clamped in place.
    // The window argument is only honoured when the `{list}` argument
    // was given too, because it is the third.
    let wp = if args.len() > 1 && args.len() > 2 {
        window_arg(args, 2)
    } else {
        Some(Win::current_or_none())
    };
    if let Some(Some(wp)) = wp {
        let bp = wp.buffer();
        let mut fnum = bp.handle as c_int;
        let fp = unsafe { var2fpos(&args[0], false, &raw mut fnum, false, wp) };
        if let Some(mut fp) = fp
            && fp.lnum <= bp.b_ml.ml_line_count
            && fnum == bp.handle
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
            unsafe { getvvcol(wp, pos, start, ptr::null_mut(), end) };
            vcol_start += 1;
            vcol_end += 1;
        }
    }
    if args.len() > 1 && arg_bool(&args[1]) != 0 {
        let l = list_alloc_ret(result, 2);
        unsafe { tv_list_append_number(l, vcol_start as VarNumber) };
        unsafe { tv_list_append_number(l, vcol_end as VarNumber) };
    } else {
        result.write_number(vcol_end as VarNumber);
    }
}

/// `line({expr} [, {winid}])`.
pub fn f_line(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut fnum: c_int = 0;
    let out = &raw mut fnum;
    let fp = if args.len() <= 1 {
        // SAFETY: argument 0 is a live typval and `curwin` a live window.
        unsafe { var2fpos(&args[0], true, out, false, Win::current()) }
    } else {
        match win_and_tab_by_id(arg_number(&args[1]) as c_int) {
            None => None,
            Some((wp, _)) => {
                // Resolving a position in another window moves its cursor,
                // and 'splitkeep' decides whether that is allowed to scroll
                // it. Diff-mode windows are always exempt because their
                // scroll is bound to this one's.
                let both_diff =
                    wp.w_onebuf_opt.wo_diff != 0 && Win::current().w_onebuf_opt.wo_diff != 0;
                // SAFETY: `p_spk` is the option's own C string value.
                if unsafe { *p_spk.get() } != b'c' as c_char || both_diff {
                    skip_update_topline.set(true);
                }
                check_cursor(wp);
                let fp = unsafe { var2fpos(&args[0], true, out, false, wp) };
                skip_update_topline.set(false);
                fp
            }
        }
    };
    result.write_number(fp.map_or(0, |fp| fp.lnum as VarNumber));
}

/// `getpos({expr})`.
pub fn f_getpos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    getpos_both(args, result, false, false);
}

/// `getcharpos({expr})` — as `getpos()` but with a character column.
pub fn f_getcharpos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    getpos_both(args, result, false, true);
}

/// `getcurpos([{winid}])` — the cursor, plus a fifth 'curswant' element.
pub fn f_getcurpos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    getpos_both(args, result, true, false);
}

/// `getcursorcharpos([{winid}])`.
pub fn f_getcursorcharpos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    getpos_both(args, result, true, true);
}

/// The four getters' shared body. `getcurpos` takes the cursor of the
/// window its argument names rather than resolving a position expression,
/// and appends 'curswant'.
fn getpos_both(args: &[TypVal], result: &mut TypVal, getcurpos: bool, charcol: bool) {
    // SAFETY throughout: every pointer read below comes back from the
    // position parser.
    let mut wp = Win::current_or_none();
    let mut fnum: c_int = -1;
    let fp = if !getcurpos {
        unsafe { var2fpos(&args[0], true, &raw mut fnum, charcol, Win::current()) }
    } else {
        let mut fp = if !args.is_empty() {
            // `wp` is overwritten even when the lookup fails: a
            // `getcurpos()` on a window that does not exist answers 0
            // for 'curswant' rather than the current window's.
            wp = find_win_by_nr_or_id(&args[0]);
            wp.map(|wp| wp.w_cursor)
        } else {
            Some(Win::current().w_cursor)
        };
        if let Some(pos) = &mut fp
            && charcol
        {
            let buffer = wp.and_then(Win::buffer_or_none);
            pos.col = buf_byteidx_to_charidx(buffer, pos.lnum, pos.col) as ColNr;
        }
        fp
    };

    let l = list_alloc_ret(result, 4 + isize::from(getcurpos));
    unsafe { tv_list_append_number(l, if fnum != -1 { fnum as VarNumber } else { 0 }) };
    let (lnum, col, coladd) = fp.map_or((0, 0, 0), |fp| {
        // MAXCOL is passed through rather than made one-based.
        let col = if fp.col == END_OF_LINE {
            END_OF_LINE
        } else {
            fp.col + 1
        };
        (
            fp.lnum as VarNumber,
            col as VarNumber,
            fp.coladd as VarNumber,
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
/// `l` is a live list and `window` is a window pointer or null.
unsafe fn append_curswant(l: *mut List, window: Option<Win>) {
    // SAFETY throughout: the caller's obligation.
    let mut cur = Win::current();
    let saved_set_curswant = cur.w_set_curswant;
    let saved_curswant = cur.w_curswant;
    let saved_virtcol = cur.w_virtcol;
    if window == Some(cur) {
        update_curswant();
    }
    // SAFETY throughout: `window` is null or the window resolved above, and `l` the list
    // being filled in.
    let curswant = match window.map(|w| w.w_curswant) {
        None => 0,
        Some(END_OF_LINE) => MAXCOL as VarNumber,
        Some(want) => want as VarNumber + 1,
    };
    unsafe { tv_list_append_number(l, curswant) };
    // Only restored when 'curswant' was due to be recomputed anyway:
    // if it was already valid, `update_curswant` did not change it.
    if window == Some(cur) && saved_set_curswant {
        cur.w_set_curswant = saved_set_curswant;
        cur.w_curswant = saved_curswant;
        cur.w_virtcol = saved_virtcol;
        cur.w_valid.clear(WinValid::VIRTCOL);
    }
}

/// `cursor({lnum}, {col} [, {off}])` or `cursor({list})`.
pub fn f_cursor(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    set_cursorpos(args, result, false);
}

/// `setcursorcharpos({lnum}, {col} [, {off}])` or with a List.
pub fn f_setcursorcharpos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    set_cursorpos(args, result, true);
}

fn set_cursorpos(args: &[TypVal], result: &mut TypVal, charcol: bool) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: `pos` and `curswant` are live
    // locals the List parser fills.
    result.write_number(-1);
    let mut set_curswant = true;
    let (lnum, mut col, coladd) = if args.first().is_some_and(|arg| arg.v_type() == VAR_LIST) {
        let mut pos = NOWHERE;
        let mut curswant: ColNr = -1;
        let (out, want) = (&raw mut pos, &raw mut curswant);
        // SAFETY: argument 0 is a live typval and both are locals.
        let read = unsafe { list2fpos(&args[0], out, ptr::null_mut(), want, charcol) };
        if read.is_err() {
            emsg(gettext(e_invarg));
            return;
        }
        if curswant >= 0 {
            Win::current().w_curswant = curswant - 1;
            set_curswant = false;
        }
        (pos.lnum, pos.col, pos.coladd)
    } else if matches!(args[0].v_type(), VAR_NUMBER | VAR_STRING)
        && args
            .get(1)
            .is_some_and(|arg| matches!(arg.v_type(), VAR_NUMBER | VAR_STRING))
    {
        let mut lnum = arg_lnum(&args[0]);
        if lnum < 0 {
            // Kept on the variadic message call: the argument is
            // arbitrary user bytes. Note that this reports and then
            // carries on to the range check below.
            let what = arg_string(&mut numbuf, &args[0]);
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let what = unsafe { c_str(what) };
            semsg!("E475: Invalid argument: {what}");
        } else if lnum == 0 {
            lnum = Win::current().w_cursor.lnum;
        }
        let mut col = arg_number_chk(&args[1], None) as ColNr;
        if charcol {
            col = buf_charidx_to_byteidx(Buf::current_or_none(), lnum, col) + 1;
        }
        let coladd = if args.len() > 2 {
            arg_number_chk(&args[2], None) as ColNr
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
        Win::current().w_cursor.lnum = lnum;
    }
    // The column is one-based on the way in, except for MAXCOL which
    // means "end of line" and is passed through.
    if col != END_OF_LINE {
        col = (col - 1).max(0);
    }
    Win::current().w_cursor.col = col;
    Win::current().w_cursor.coladd = coladd;
    check_cursor(Win::current());
    unsafe { mb_adjust_cursor() };
    Win::current().w_set_curswant = set_curswant;
    result.write_number(0);
}

/// `setpos({expr}, {list})`.
pub fn f_setpos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    set_position(args, result, false);
}

/// `setcharpos({expr}, {list})` — as `setpos()` with a character column.
pub fn f_setcharpos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    set_position(args, result, true);
}

fn set_position(args: &[TypVal], result: &mut TypVal, charpos: bool) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: `pos`, `fnum` and `curswant` are
    // live locals the List parser fills, and `name` is NUL-terminated.
    result.write_number(-1);
    let name = arg_string_chk(&mut numbuf, &args[0]);
    if name.is_null() {
        return;
    }
    let mut pos = NOWHERE;
    let mut fnum: c_int = 0;
    let mut curswant: ColNr = -1;
    let (out, buf, want) = (&raw mut pos, &raw mut fnum, &raw mut curswant);
    // SAFETY: argument 1 is a live typval and the three are locals.
    if unsafe { list2fpos(&args[1], out, buf, want, charpos) }.is_err() {
        return;
    }
    if pos.col != END_OF_LINE {
        pos.col = (pos.col - 1).max(0);
    }
    match unsafe { CStr::from_ptr(name) }.to_bytes() {
        b"." => {
            Win::current().w_cursor = pos;
            if curswant >= 0 {
                Win::current().w_curswant = curswant - 1;
                Win::current().w_set_curswant = false;
            }
            check_cursor(Win::current());
            result.write_number(0);
        }
        // A mark name is exactly one byte after the quote.
        [b'\'', c] => {
            if unsafe { setmark_pos(*c as c_int, &raw mut pos, fnum, ptr::null_mut()) }.is_ok() {
                result.write_number(0);
            }
        }
        _ => {
            emsg(gettext(e_invarg));
        }
    }
}

/// `getcharsearch()` — the state `;` and `,` repeat.
pub fn f_getcharsearch(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: `result` is the dispatcher's cleared return value; the three
    // readers answer from the process-wide character-search state.
    let csearch = last_csearch();
    tv_dict_alloc_ret(result);
    let dict = result.dict_or_null();
    let _ = unsafe { tv_dict_add_str(dict, c"char".as_ptr(), 4, csearch.as_ptr()) };
    let forward = last_csearch_forward() as VarNumber;
    let _ = unsafe { tv_dict_add_nr(dict, c"forward".as_ptr(), 7, forward) };
    let until = last_csearch_until() as VarNumber;
    let _ = unsafe { tv_dict_add_nr(dict, c"until".as_ptr(), 5, until) };
}

/// `setcharsearch({dict})` — each key is optional and missing keys leave
/// that part of the state alone.
pub fn f_setcharsearch(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let _rettv = _result;
    // SAFETY throughout: `&args[0]` is a live typval; after the check the union
    // holds a Dict pointer, which may still be null.
    if tv_check_for_dict_arg(args, 0).is_err() {
        return;
    }
    let d = args[0].dict_or_null();
    if d.is_null() {
        return;
    }
    let csearch = unsafe { numbuf.dict_string(d, c"char".as_ptr()) };
    if !csearch.is_null() {
        unsafe { set_last_csearch(utf_ptr2char(csearch), csearch, utfc_ptr2len(csearch)) };
    }
    let di = unsafe { tv_dict_find(d, c"forward".as_ptr(), 7) };
    if !di.is_null() {
        let forward = unsafe { tv_get_number(&(*di).di_tv) } != 0;
        set_csearch_direction(if forward { FORWARD } else { BACKWARD } as Direction);
    }
    let di = unsafe { tv_dict_find(d, c"until".as_ptr(), 5) };
    if !di.is_null() {
        set_csearch_until((unsafe { tv_get_number(&(*di).di_tv) } != 0) as c_int);
    }
}
