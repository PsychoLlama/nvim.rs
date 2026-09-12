//! Reading and writing buffer text: `getbufline()`, `setbufline()`,
//! `appendbufline()`, `deletebufline()` and their current-buffer forms.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use crate::eval::typval::{tv_list_items, tv_list_len};
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
    // SAFETY: the caller's obligation. `cob` is a live local, restored on
    // every path out; `line` is owned here and freed before each replacement
    // and once at the end.
    let mut lnum: LineNr = lnum_arg + LineNr::from(append);
    let mut added: c_int = 0;
    let is_curbuf: bool = buffer == Buf::current_or_none();
    // SAFETY: the caller's obligation -- live typvals, and a live buffer or
    // NULL, which the test below tells apart.
    let mut ret = unsafe { Tv::new(result) };
    let unloaded = |b: Buf| !is_curbuf && b.b_ml.ml_mfp.is_null();
    if buffer.is_none_or(unloaded) || lnum < 1 {
        ret.write_number(1);
        return;
    }
    let mut cob = SavedBufferState::new();
    if let (false, Some(buffer)) = (is_curbuf, buffer) {
        cob.prepare(buffer);
    }
    let append_lnum: LineNr = if append {
        lnum - 1
    } else {
        Buf::current().line_count()
    };
    let mut l: *mut List = ptr::null_mut();
    let mut at: usize = 0;
    let mut line: *mut c_char = ptr::null_mut();
    let src = lines;
    '_cleanup: {
        if src.v_type() == VAR_LIST {
            l = src.list_or_null();
            if unsafe { tv_list_len(l) } == 0 {
                break '_cleanup;
            }
        } else {
            line = unsafe { typval_tostring(Some(lines), false) };
        }
        loop {
            // Re-read, as upstream does: the type tag is the argument's own
            // and the walk below can run user code.
            if src.v_type() == VAR_LIST {
                // Re-read the items too: the body below runs autocommands,
                // which may edit the very list being appended.
                let Some(item) = (unsafe { tv_list_items(l) }).get(at) else {
                    break;
                };
                unsafe { xfree(line.cast()) };
                line = unsafe { typval_tostring(Some(&item.li_tv), false) };
                at += 1;
            }
            ret.write_number(1);
            if line.is_null() || lnum > Buf::current().line_count() + 1 {
                break;
            }
            if u_sync_once.get() == 2 {
                u_sync_once.set(1);
                u_sync(true);
            }
            if !append && lnum <= Buf::current().line_count() {
                let old_len = len_as_int(unsafe { cstr::bytes_at(ml_get(lnum)) }.len());
                if u_savesub(lnum).is_ok() && unsafe { ml_replace(lnum, line, true) }.is_ok() {
                    let new_len = len_as_int(unsafe { cstr::bytes_at(line) }.len());
                    unsafe { inserted_bytes(lnum, 0, old_len, new_len) };
                    if is_curbuf && lnum == Win::current().w_cursor.lnum {
                        check_cursor_col(Win::current());
                    }
                    ret.write_number(0);
                }
            } else if added > 0 || u_save(lnum - 1, lnum).is_ok() {
                added += 1;
                if unsafe { ml_append(lnum - 1, line, 0, false) }.is_ok() {
                    ret.write_number(0);
                }
            }
            if l.is_null() {
                break;
            }
            lnum += 1;
        }
        unsafe { xfree(line.cast()) };
        if added > 0 {
            unsafe { appended_lines_mark(append_lnum, added) };
            // Only the current window of the current buffer follows the
            // insertion; the others keep looking at the line they were on.
            for mut wp in tab_windows() {
                if wp.w_buffer == buffer.map_or(ptr::null_mut(), Buf::raw)
                    && (wp.w_buffer != Buf::current_raw() || wp.is_current())
                    && wp.w_cursor.lnum > append_lnum
                {
                    wp.w_cursor.lnum += added;
                }
            }
            check_cursor_col(Win::current());
            update_topline(Win::current());
        }
    }
    if !is_curbuf {
        cob.restore();
    }
}

/// `setbufline()` and `appendbufline()`, which differ only in `append`.
fn buf_set_append_line(args: &[TypVal], result: &mut TypVal, append: bool) {
    // SAFETY: the caller's obligation.
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
    // SAFETY: the caller's obligation; every line index is clamped to the
    // buffer before `ml_get_buf` sees it.
    let mut ret = unsafe { Tv::new(result) };
    ret.write_empty(if retlist { VAR_LIST } else { VAR_STRING });
    ret.write_string(ptr::null_mut());
    if buffer.is_none_or(|b| b.b_ml.ml_mfp.is_null()) || start < 0 || end < start {
        if retlist {
            tv_list_alloc_ret(result, 0);
        }
        return;
    }
    let buffer = buffer.expect("the early return covers an absent buffer");
    if !retlist {
        let len = |n| size_t::try_from(n).expect("a line length is not negative");
        let line = (start >= 1 && start <= buffer.line_count())
            .then(|| unsafe { xstrnsave(buffer.line(start).raw(), len(buffer.line_len(start))) });
        ret.write_string(line.unwrap_or(ptr::null_mut()));
        return;
    }
    start = start.max(1);
    end = end.min(buffer.line_count());
    let list = tv_list_alloc_ret(result, (end - start + 1) as ptrdiff_t);
    for lnum in start..=end {
        let (text, len) = (buffer.line(lnum).raw(), buffer.line_len(lnum) as ssize_t);
        unsafe { tv_list_append_string(list, text, len) };
    }
}

/// `getbufline()` when `retlist`, `getbufoneline()` otherwise.
fn getbufline(args: &[TypVal], result: &mut TypVal, retlist: bool) {
    // SAFETY: the caller's obligation.
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
    // SAFETY: the arguments and `result` are live typvals; `curbuf` is set.
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
    // SAFETY: the arguments and `result` are live typvals; `curbuf` is set.
    let did_emsg_before = did_emsg.get();
    let lnum = arg_lnum(args, 0);
    if did_emsg.get() == did_emsg_before {
        set_buffer_lines(Buf::current_or_none(), lnum, false, &args[1], result);
    }
}

/// `getline({lnum} [, {end}])` — one String, or a List for a range.
pub fn f_getline(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the arguments and `result` are live typvals; `curbuf` is set.
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
    // SAFETY: the arguments and `result` are live typvals; `cob` is a live
    // local, restored on every path out of the change.
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
    let mut cob = SavedBufferState::new();
    if !is_curbuf {
        cob.prepare(buf);
    }
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
            let _ = unsafe { ml_delete_flags(first, ML_DEL_MESSAGE) };
        }
        // Pull every cursor that was inside or after the deleted range
        // back onto a line that still exists.
        for mut wp in tab_windows().filter(|wp| wp.w_buffer == buf.raw()) {
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
        unsafe { deleted_lines_mark(first, count) };
        result.write_number(0);
    }
    if !is_curbuf {
        cob.restore();
    }
}
