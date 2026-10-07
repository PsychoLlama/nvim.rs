//! Reading and writing buffer text: `getbufline()`, `setbufline()`,
//! `appendbufline()`, `deletebufline()` and their current-buffer forms.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::typval::list_len;
use crate::memline::{Lines, ml_append_text, ml_get_len, ml_replace_buf_text};
use crate::memory::{ThinCString, XString};
use crate::narrow::len_as_int;
use crate::types::{VAR_LIST, VAR_STRING};

/// Set or append lines in buffer `buffer`, from `lines` — any type, converted to
/// a string, or a List of them.
///
/// `result` ends 0 when every line went in and 1 otherwise, which is what all
/// four builtins answer.
pub(crate) fn set_buffer_lines(
    buffer: Option<Buf>,
    lnum_arg: LineNr,
    append: bool,
    lines: &TypVal,
    result: &mut TypVal,
) {
    let mut lnum: LineNr = lnum_arg + LineNr::from(append);
    let mut added: c_int = 0;
    let is_curbuf: bool = buffer == Buf::current_or_none();
    let unloaded = |b: Buf| !is_curbuf && b.b_ml.ml_mfp.is_null();
    let Some(target) = buffer.filter(|&b| !unloaded(b)) else {
        result.write_number(1);
        return;
    };
    if lnum < 1 {
        result.write_number(1);
        return;
    }
    let cob = (!is_curbuf).then(|| SavedBufferState::prepare(target));
    let append_lnum: LineNr = if append {
        lnum - 1
    } else {
        Buf::current().line_count()
    };
    // A List argument is walked through a handle of its own: the body below
    // runs autocommands, which may edit -- or drop -- the very list.
    let list = if lines.v_type() == VAR_LIST {
        lines.list_handle()
    } else {
        None
    };
    let mut at: usize = 0;
    let mut line: Option<XString> = None;
    '_cleanup: {
        if lines.v_type() == VAR_LIST {
            if list_len(list.as_deref()) == 0 {
                break '_cleanup;
            }
        } else {
            line = Some(typval_tostring(Some(lines), false));
        }
        loop {
            if let Some(list) = &list {
                // Re-read the items every time, as upstream does.
                let Some(item) = list.items().get(at) else {
                    break;
                };
                line = Some(typval_tostring(Some(&item.li_tv), false));
                at += 1;
            }
            result.write_number(1);
            let Some(text) = line.as_ref() else { break };
            if lnum > Buf::current().line_count() + 1 {
                break;
            }
            let text = text.as_cstr().to_bytes();
            if u_sync_once.get() == 2 {
                u_sync_once.set(1);
                u_sync(true);
            }
            if !append && lnum <= Buf::current().line_count() {
                let old_len = ml_get_len(lnum);
                if u_savesub(lnum).is_ok()
                    && ml_replace_buf_text(Buf::current(), lnum, text).is_ok()
                {
                    inserted_bytes(lnum, 0, old_len, len_as_int(text.len()));
                    if is_curbuf && lnum == Win::current().w_cursor.lnum {
                        check_cursor_col(Win::current());
                    }
                    result.write_number(0);
                }
            } else if added > 0 || u_save(lnum - 1, lnum).is_ok() {
                added += 1;
                if ml_append_text(lnum - 1, text).is_ok() {
                    result.write_number(0);
                }
            }
            if list.is_none() {
                break;
            }
            lnum += 1;
        }
        if added > 0 {
            appended_lines_mark(append_lnum, added);
            // Only the current window of the current buffer follows the
            // insertion; the others keep looking at the line they were on.
            for mut wp in tab_windows() {
                if Some(wp.w_buffer) == buffer
                    && (!wp.w_buffer.is_current() || wp.is_current())
                    && wp.w_cursor.lnum > append_lnum
                {
                    wp.w_cursor.lnum += added;
                }
            }
            check_cursor_col(Win::current());
            update_topline(Win::current());
        }
    }
    if let Some(cob) = cob {
        cob.restore();
    }
}

/// `setbufline()` and `appendbufline()`, which differ only in `append`.
fn buf_set_append_line(args: &[TypVal], result: &mut TypVal, append: bool) {
    let did_emsg_before = did_emsg.get();
    let Some(buf) = arg_buf(args, 0, 0) else {
        result.write_number(1);
        return;
    };
    // The line number is resolved against the named buffer, and a bad one
    // reports; only then is anything written.
    let lnum = arg_lnum_buf(args, 1, Some(buf));
    if did_emsg.get() == did_emsg_before {
        set_buffer_lines(Some(buf), lnum, append, &args[2], result);
    }
}

/// Lines `start..=end` of `buffer`, as a List or as one String.
fn get_buffer_lines(
    buffer: Option<Buf>,
    mut start: LineNr,
    mut end: LineNr,
    retlist: bool,
    result: &mut TypVal,
) {
    result.write_empty(if retlist { VAR_LIST } else { VAR_STRING });
    result.write_string(None);
    let Some(buffer) = buffer.filter(|b| !b.b_ml.ml_mfp.is_null()) else {
        if retlist {
            tv_list_alloc_ret(result, 0);
        }
        return;
    };
    if start < 0 || end < start {
        if retlist {
            tv_list_alloc_ret(result, 0);
        }
        return;
    }
    let mut text = Lines::in_buffer(buffer);
    if !retlist {
        let line = (start >= 1 && start <= buffer.line_count())
            .then(|| ThinCString::from_bytes(text.line(start)));
        result.write_string(line);
        return;
    }
    start = start.max(1);
    end = end.min(buffer.line_count());
    let list = tv_list_alloc_ret(result, (end - start + 1) as ptrdiff_t);
    for lnum in start..=end {
        list.push_bytes(Some(text.line(lnum)));
    }
}

/// `getbufline()` when `retlist`, `getbufoneline()` otherwise.
fn getbufline(args: &[TypVal], result: &mut TypVal, retlist: bool) {
    let did_emsg_before = did_emsg.get();
    let buf = arg_buf_chk(args, 0);
    let lnum = arg_lnum_buf(args, 1, buf);
    if did_emsg.get() > did_emsg_before {
        return;
    }
    let end = if args.len() > 2 {
        arg_lnum_buf(args, 2, buf)
    } else {
        lnum
    };
    get_buffer_lines(buf, lnum, end, retlist, result);
}

/// `append({lnum}, {string/list})`.
pub fn f_append(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let did_emsg_before = did_emsg.get();
    let lnum = arg_lnum(args, 0);
    if did_emsg.get() == did_emsg_before {
        set_buffer_lines(Buf::current_or_none(), lnum, true, &args[1], result);
    }
}

/// `appendbufline({buf}, {lnum}, {string/list})`.
pub fn f_appendbufline(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    buf_set_append_line(args, result, true);
}

/// `setbufline({buf}, {lnum}, {string/list})`.
pub fn f_setbufline(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    buf_set_append_line(args, result, false);
}

/// `setline({lnum}, {string/list})`.
pub fn f_setline(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let did_emsg_before = did_emsg.get();
    let lnum = arg_lnum(args, 0);
    if did_emsg.get() == did_emsg_before {
        set_buffer_lines(Buf::current_or_none(), lnum, false, &args[1], result);
    }
}

/// `getline({lnum} [, {end}])` — one String, or a List for a range.
pub fn f_getline(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let lnum = arg_lnum(args, 0);
    // One argument answers a string, a range answers a list.
    let (end, retlist) = if args.len() > 1 {
        (arg_lnum(args, 1), true)
    } else {
        (lnum, false)
    };
    get_buffer_lines(Buf::current_or_none(), lnum, end, retlist, result);
}

/// `getbufline({buf}, {lnum} [, {end}])`.
pub fn f_getbufline(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    getbufline(args, result, true);
}

/// `getbufoneline({buf}, {lnum})`.
pub fn f_getbufoneline(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    getbufline(args, result, false);
}

/// `deletebufline({buf}, {first} [, {last}])` — 0 when the lines went.
pub fn f_deletebufline(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(1);
    let did_emsg_before = did_emsg.get();
    let Some(buf) = arg_buf(args, 0, 0) else {
        return;
    };
    let first = arg_lnum_buf(args, 1, Some(buf));
    if did_emsg.get() > did_emsg_before {
        return;
    }
    let mut last = if args.len() > 2 {
        arg_lnum_buf(args, 2, Some(buf))
    } else {
        first
    };
    let (mfp, count) = (buf.b_ml.ml_mfp, buf.b_ml.ml_line_count);
    if mfp.is_null() || first < 1 || first > count || last < first {
        return;
    }
    let is_curbuf = Some(buf) == Buf::current_or_none();
    let cob = (!is_curbuf).then(|| SavedBufferState::prepare(buf));
    last = last.min(Buf::current().line_count());
    let count = last - first + 1;
    if u_sync_once.get() == 2 {
        u_sync_once.set(1);
        u_sync(true);
    }
    if u_save(first - 1, last + 1).is_ok() {
        // Every delete takes the same line number: the lines below move
        // up.
        for _ in first..=last {
            let _ = ml_delete_flags(first, ML_DEL_MESSAGE);
        }
        // Pull every cursor that was inside or after the deleted range
        // back onto a line that still exists.
        for mut wp in tab_windows().filter(|wp| wp.w_buffer == buf) {
            if wp.w_cursor.lnum > last {
                wp.w_cursor.lnum -= count;
            } else if wp.w_cursor.lnum > first {
                wp.w_cursor.lnum = first;
            }
            let line_count = wp.buffer().line_count();
            if wp.w_cursor.lnum > line_count {
                wp.w_cursor.lnum = line_count;
            }
        }
        check_cursor_col(Win::current());
        deleted_lines_mark(first, count);
        result.write_number(0);
    }
    if let Some(cob) = cob {
        cob.restore();
    }
}
