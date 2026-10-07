//! Positions in a buffer: the cursor, `line()`, `col()`, `virtcol()`,
//! `getpos()`/`setpos()` and the character-search state.
#![forbid(unsafe_code)]

use super::wrappers::{arg_bool, arg_lnum, arg_number, arg_number_chk};
use crate::cursor::check_cursor;
use crate::eval::typval::{
    NumBuf, tv_check_for_dict_arg, tv_check_for_opt_number_arg, tv_check_for_string_or_list_arg,
    tv_dict_alloc_ret, tv_get_number, tv_list_alloc_ret,
};
use crate::eval::window::{find_win_by_nr_or_id, win_and_tab_by_id};
use crate::eval::{buf_byteidx_to_charidx, buf_charidx_to_byteidx, list2fpos, var2fpos};
use crate::mark::setmark_at;
use crate::mbyte::{char_at, cluster_len, mb_adjust_cursor};
use crate::memline::{ml_find_line_or_offset, ml_get_buf_len};
use crate::message::e_invarg;
use crate::message::emsg;
use crate::message_fmt::msg_cstr;
use crate::r#move::{WinValid, update_curswant};
use crate::option::vars::P_SPK;
use crate::os::cshim::gettext;
use crate::pos::MAXCOL;
use crate::search::{
    BACKWARD, FORWARD, last_csearch, last_csearch_forward, last_csearch_until,
    set_csearch_direction, set_csearch_until, set_last_csearch,
};
use crate::semsg;
use crate::types::{
    ColNr, Direction, EvalFuncData, List, Pos, TypVal, VAR_LIST, VAR_NUMBER, VAR_STRING, VarNumber,
};
use crate::window::state::skip_update_topline;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::c_int;

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
    let mut boff = arg_number(&args[0]) as c_int - 1;
    result.write_number(if boff < 0 {
        -1
    } else {
        ml_find_line_or_offset(Buf::current(), 0, Some(&mut boff), false) as VarNumber
    });
}

/// `line2byte({lnum})` — the byte offset a line starts at, one-based, or -1
/// past the end. One past the last line is allowed: it is the buffer size.
pub fn f_line2byte(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let lnum = arg_lnum(&args[0]);
    let offset = if lnum < 1 || lnum > Buf::current().b_ml.ml_line_count + 1 {
        -1
    } else {
        ml_find_line_or_offset(Buf::current(), lnum, None, false) as VarNumber
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
    let fp = var2fpos(&args[0], false, &mut fnum, charcol, wp);
    let mut col: ColNr = 0;
    if let Some(fp) = fp
        && fnum == bp.handle
    {
        if fp.col == END_OF_LINE {
            // MAXCOL means "end of line"; past the last line there is
            // no line to measure, so it stays MAXCOL.
            col = if fp.lnum <= bp.b_ml.ml_line_count {
                (ml_get_buf_len(bp, fp.lnum)) + 1
            } else {
                END_OF_LINE
            };
        } else {
            // Upstream adds one more here, with 'virtualedit' on, when the
            // position is the cursor itself (`fp == &wp->w_cursor`) and it
            // sits past the last character of the line. `var2fpos` answers
            // a position of its own rather than the cursor's address, so
            // that test never holds and the adjustment never applies; that
            // is preserved by not making it. See F-P22-36.
            col = fp.col + 1;
        }
    }
    result.write_number(col as VarNumber);
}

/// `virtcol({expr} [, {list} [, {winid}]])`.
pub fn f_virtcol(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut vcol_start: ColNr = 0;
    let mut vcol_end: ColNr = 0;
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
        let fp = var2fpos(&args[0], false, &mut fnum, false, wp);
        if let Some(mut fp) = fp
            && fp.lnum <= bp.b_ml.ml_line_count
            && fnum == bp.handle
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
            (vcol_start, vcol_end) = wp.virtual_vcol_span_at(fp);
            vcol_start += 1;
            vcol_end += 1;
        }
    }
    if args.len() > 1 && arg_bool(&args[1]) != 0 {
        let l = tv_list_alloc_ret(result, 2);
        l.push_number(vcol_start as VarNumber);
        l.push_number(vcol_end as VarNumber);
    } else {
        result.write_number(vcol_end as VarNumber);
    }
}

/// `line({expr} [, {winid}])`.
pub fn f_line(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut fnum: c_int = 0;
    let fp = if args.len() <= 1 {
        var2fpos(&args[0], true, &mut fnum, false, Win::current())
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
                if P_SPK.first_byte() != b'c' || both_diff {
                    skip_update_topline.set(true);
                }
                check_cursor(wp);
                let fp = var2fpos(&args[0], true, &mut fnum, false, wp);
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
    let mut wp = Win::current_or_none();
    let mut fnum: c_int = -1;
    let fp = if !getcurpos {
        var2fpos(&args[0], true, &mut fnum, charcol, Win::current())
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

    let l = tv_list_alloc_ret(result, 4 + isize::from(getcurpos));
    l.push_number(if fnum != -1 { fnum as VarNumber } else { 0 });
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
    l.push_number(lnum);
    l.push_number(col);
    l.push_number(coladd);
    if getcurpos {
        append_curswant(l, wp);
    }
}

/// `getcurpos()`'s fifth element. Reading it means recomputing 'curswant',
/// which is a side effect the caller must not see — so the three fields
/// that recomputation touches are put back, and the cached virtual column
/// invalidated so the next reader recomputes it properly.
fn append_curswant(l: &mut List, window: Option<Win>) {
    let mut cur = Win::current();
    let saved_set_curswant = cur.w_set_curswant;
    let saved_curswant = cur.w_curswant;
    let saved_virtcol = cur.w_virtcol;
    if window == Some(cur) {
        update_curswant();
    }
    let curswant = match window.map(|w| w.w_curswant) {
        None => 0,
        Some(END_OF_LINE) => MAXCOL as VarNumber,
        Some(want) => want as VarNumber + 1,
    };
    l.push_number(curswant);
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
    result.write_number(-1);
    let mut set_curswant = true;
    let (lnum, mut col, coladd) = if args.first().is_some_and(|arg| arg.v_type() == VAR_LIST) {
        let mut pos = NOWHERE;
        let mut curswant: ColNr = -1;
        let read = list2fpos(&args[0], &mut pos, None, Some(&mut curswant), charcol);
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
            // Note that this reports and then carries on to the range
            // check below.
            let what = msg_cstr(numbuf.string(&args[0]));
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
    mb_adjust_cursor();
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
    result.write_number(-1);
    let Some(name) = numbuf.bytes_chk(&args[0]) else {
        return;
    };
    let mut pos = NOWHERE;
    let mut fnum: c_int = 0;
    let mut curswant: ColNr = -1;
    let want = Some(&mut curswant);
    if list2fpos(&args[1], &mut pos, Some(&mut fnum), want, charpos).is_err() {
        return;
    }
    if pos.col != END_OF_LINE {
        pos.col = (pos.col - 1).max(0);
    }
    match name {
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
            if setmark_at(c_int::from(*c), pos, fnum).is_ok() {
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
    // The store is NUL-terminated within its cell's worth of bytes.
    let csearch = last_csearch().map(|byte| byte as u8);
    let text = csearch.split(|&byte| byte == 0).next().unwrap_or_default();
    tv_dict_alloc_ret(result);
    let dict = result.dict_mut().expect("the dict just stored");
    let _ = dict.add_str_len(b"char", Some(text));
    let _ = dict.add_number(b"forward", last_csearch_forward() as VarNumber);
    let _ = dict.add_number(b"until", last_csearch_until() as VarNumber);
}

/// `setcharsearch({dict})` — each key is optional and missing keys leave
/// that part of the state alone.
pub fn f_setcharsearch(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if tv_check_for_dict_arg(args, 0).is_err() {
        return;
    }
    let Some(d) = args[0].dict_ref() else {
        return;
    };
    if let Some(csearch) = numbuf.dict_string(Some(d), b"char") {
        let text = csearch.to_bytes();
        set_last_csearch(char_at(text), &text[..cluster_len(text)]);
    }
    if let Some(di) = d.find(b"forward") {
        let forward = tv_get_number(&di.di_tv) != 0;
        set_csearch_direction(if forward { FORWARD } else { BACKWARD } as Direction);
    }
    if let Some(di) = d.find(b"until") {
        set_csearch_until((tv_get_number(&di.di_tv) != 0) as c_int);
    }
}
