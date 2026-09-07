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
use crate::narrow::len_as_int;
use crate::types::{VAR_LIST, VAR_STRING};
use core::mem::offset_of;

/// Set or append lines in buffer `buffer`, from `lines` — any type, converted to
/// a string, or a List of them.
///
/// `result` ends 0 when every line went in and 1 otherwise, which is what all
/// four builtins answer.
///
/// # Safety
/// `buffer` must be a live buffer or NULL, and `lines`/`result` live typvals.
pub(crate) unsafe fn set_buffer_lines(
    buffer: Option<Buf>,
    lnum_arg: LineNr,
    append: bool,
    lines: *mut TypVal,
    result: *mut TypVal,
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
        ret.vval.v_number = 1;
        return;
    }
    let mut cob = SavedBufferState::new();
    if let (false, Some(buffer)) = (is_curbuf, buffer) {
        unsafe { cob.prepare(buffer) };
    }
    let append_lnum: LineNr = if append {
        lnum - 1
    } else {
        Buf::current().line_count()
    };
    let mut l: *mut List = ptr::null_mut();
    let mut li: *mut ListItem = ptr::null_mut();
    let mut line: *mut c_char = ptr::null_mut();
    let src = unsafe { Tv::new(lines) };
    '_cleanup: {
        if src.v_type == VAR_LIST {
            l = src.list_or_null();
            if l.is_null() || unsafe { (*l).lv_len } == 0 {
                break '_cleanup;
            }
            li = unsafe { (*l).lv_first };
        } else {
            line = unsafe { typval_tostring(lines, false) };
        }
        loop {
            // Re-read, as upstream does: the type tag is the argument's own
            // and the walk below can run user code.
            if src.v_type == VAR_LIST {
                if li.is_null() {
                    break;
                }
                let item = unsafe { Li::new(li) };
                let itv = item.field_ptr(offset_of!(ListItem, li_tv));
                unsafe { xfree(line.cast()) };
                line = unsafe { typval_tostring(itv, false) };
                li = item.li_next;
            }
            ret.vval.v_number = 1;
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
                    ret.vval.v_number = 0;
                }
            } else if added > 0 || u_save(lnum - 1, lnum).is_ok() {
                added += 1;
                if unsafe { ml_append(lnum - 1, line, 0, false) }.is_ok() {
                    ret.vval.v_number = 0;
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
        unsafe { cob.restore() };
    }
}

/// `setbufline()` and `appendbufline()`, which differ only in `append`.
///
/// # Safety
/// The arguments and `result` must be live typvals.
unsafe fn buf_set_append_line(args: Args<'_>, result: &mut TypVal, append: bool) {
    // SAFETY: the caller's obligation.
    let did_emsg_before = did_emsg.get();
    let Some(buf) = arg_buf(args, 0, 0) else {
        result.vval.v_number = 1;
        return;
    };
    // The line number is resolved against the named buffer, and a bad one
    // reports; only then is anything written.
    let lnum = unsafe { arg_lnum_buf(args, 1, Some(buf)) };
    if did_emsg.get() == did_emsg_before {
        unsafe { set_buffer_lines(Some(buf), lnum, append, args.ptr(2), result) };
    }
}

/// Lines `start..=end` of `buffer`, as a List or as one String.
///
/// # Safety
/// `buffer` must be a live buffer or NULL, and `result` a live typval.
unsafe fn get_buffer_lines(
    buffer: Option<Buf>,
    mut start: LineNr,
    mut end: LineNr,
    retlist: bool,
    result: *mut TypVal,
) {
    // SAFETY: the caller's obligation; every line index is clamped to the
    // buffer before `ml_get_buf` sees it.
    let mut ret = unsafe { Tv::new(result) };
    ret.v_type = if retlist { VAR_LIST } else { VAR_STRING };
    ret.vval.v_string = ptr::null_mut();
    if buffer.is_none_or(|b| b.b_ml.ml_mfp.is_null()) || start < 0 || end < start {
        if retlist {
            unsafe { tv_list_alloc_ret(result, 0) };
        }
        return;
    }
    let buffer = buffer.expect("the early return covers an absent buffer");
    if !retlist {
        let len = |n| size_t::try_from(n).expect("a line length is not negative");
        let line = (start >= 1 && start <= buffer.line_count())
            .then(|| unsafe { xstrnsave(buffer.line(start).raw(), len(buffer.line_len(start))) });
        ret.vval.v_string = line.unwrap_or(ptr::null_mut());
        return;
    }
    start = start.max(1);
    end = end.min(buffer.line_count());
    let list = unsafe { tv_list_alloc_ret(result, (end - start + 1) as ptrdiff_t) };
    for lnum in start..=end {
        let (text, len) = unsafe { (buffer.line(lnum).raw(), buffer.line_len(lnum) as ssize_t) };
        unsafe { tv_list_append_string(list, text, len) };
    }
}

/// `getbufline()` when `retlist`, `getbufoneline()` otherwise.
///
/// # Safety
/// The arguments and `result` must be live typvals.
unsafe fn getbufline(args: Args<'_>, result: &mut TypVal, retlist: bool) {
    // SAFETY: the caller's obligation.
    let did_emsg_before = did_emsg.get();
    let buf = arg_buf_chk(args, 0);
    let lnum = unsafe { arg_lnum_buf(args, 1, buf) };
    if did_emsg.get() > did_emsg_before {
        return;
    }
    let end = if args.has(2) {
        unsafe { arg_lnum_buf(args, 2, buf) }
    } else {
        lnum
    };
    unsafe { get_buffer_lines(buf, lnum, end, retlist, result) };
}

/// `append({lnum}, {string/list})`.
pub unsafe fn f_append(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments and `result` are live typvals; `curbuf` is set.
    let did_emsg_before = did_emsg.get();
    let lnum = arg_lnum(args, 0);
    if did_emsg.get() == did_emsg_before {
        unsafe { set_buffer_lines(Buf::current_or_none(), lnum, true, args.ptr(1), result) };
    }
}

/// `appendbufline({buf}, {lnum}, {string/list})`.
pub unsafe fn f_appendbufline(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments and `result` are live typvals.
    unsafe { buf_set_append_line(args, result, true) };
}

/// `setbufline({buf}, {lnum}, {string/list})`.
pub unsafe fn f_setbufline(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments and `result` are live typvals.
    unsafe { buf_set_append_line(args, result, false) };
}

/// `setline({lnum}, {string/list})`.
pub unsafe fn f_setline(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments and `result` are live typvals; `curbuf` is set.
    let did_emsg_before = did_emsg.get();
    let lnum = arg_lnum(args, 0);
    if did_emsg.get() == did_emsg_before {
        unsafe { set_buffer_lines(Buf::current_or_none(), lnum, false, args.ptr(1), result) };
    }
}

/// `getline({lnum} [, {end}])` — one String, or a List for a range.
pub unsafe fn f_getline(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments and `result` are live typvals; `curbuf` is set.
    let lnum = arg_lnum(args, 0);
    // One argument answers a string, a range answers a list.
    let (end, retlist) = if args.has(1) {
        (arg_lnum(args, 1), true)
    } else {
        (lnum, false)
    };
    unsafe { get_buffer_lines(Buf::current_or_none(), lnum, end, retlist, result) };
}

/// `getbufline({buf}, {lnum} [, {end}])`.
pub unsafe fn f_getbufline(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments and `result` are live typvals.
    unsafe { getbufline(args, result, true) };
}

/// `getbufoneline({buf}, {lnum})`.
pub unsafe fn f_getbufoneline(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments and `result` are live typvals.
    unsafe { getbufline(args, result, false) };
}

/// `deletebufline({buf}, {first} [, {last}])` — 0 when the lines went.
pub unsafe fn f_deletebufline(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    result.vval.v_number = 1;
    // SAFETY: the arguments and `result` are live typvals; `cob` is a live
    // local, restored on every path out of the change.
    let did_emsg_before = did_emsg.get();
    let Some(buf) = arg_buf(args, 0, 0) else {
        return;
    };
    let first = unsafe { arg_lnum_buf(args, 1, Some(buf)) };
    if did_emsg.get() > did_emsg_before {
        return;
    }
    let mut last = if args.has(2) {
        unsafe { arg_lnum_buf(args, 2, Some(buf)) }
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
        unsafe { cob.prepare(buf) };
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
        result.vval.v_number = 0;
    }
    if !is_curbuf {
        unsafe { cob.restore() };
    }
}
