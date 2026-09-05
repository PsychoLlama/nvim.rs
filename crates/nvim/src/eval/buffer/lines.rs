//! Reading and writing buffer text: `getbufline()`, `setbufline()`,
//! `appendbufline()`, `deletebufline()` and their current-buffer forms.

#![deny(unsafe_op_in_unsafe_fn)]
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
    buffer: *mut Buffer,
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
    let is_curbuf: bool = buffer == curbuf.get();
    // SAFETY: the caller's obligation -- live typvals, and a live buffer or
    // NULL, which the test below tells apart.
    let mut ret = unsafe { Tv::new(result) };
    if buffer.is_null() || !is_curbuf && unsafe { (*buffer).b_ml.ml_mfp }.is_null() || lnum < 1 {
        ret.vval.v_number = 1;
        return;
    }
    let mut cob = SavedBufferState::new();
    if !is_curbuf {
        unsafe { cob.prepare(Buf::new(buffer)) };
    }
    let append_lnum: LineNr = if append {
        lnum - 1
    } else {
        cur_buf().line_count()
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
            if line.is_null() || lnum > cur_buf().line_count() + 1 {
                break;
            }
            if u_sync_once.get() == 2 {
                u_sync_once.set(1);
                u_sync(true);
            }
            if !append && lnum <= cur_buf().line_count() {
                let old_len = len_as_int(unsafe { cstr::bytes_at(ml_get(lnum)) }.len());
                if u_savesub(lnum).is_ok() && unsafe { ml_replace(lnum, line, true) }.is_ok() {
                    let new_len = len_as_int(unsafe { cstr::bytes_at(line) }.len());
                    unsafe { inserted_bytes(lnum, 0, old_len, new_len) };
                    if is_curbuf && lnum == cur_win().w_cursor.lnum {
                        check_cursor_col(cur_win());
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
                if wp.w_buffer == buffer
                    && (wp.w_buffer != curbuf.get() || wp.is_current())
                    && wp.w_cursor.lnum > append_lnum
                {
                    wp.w_cursor.lnum += added;
                }
            }
            check_cursor_col(cur_win());
            update_topline(cur_win());
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
    let buf = arg_buf(args, 0, 0);
    if buf.is_null() {
        result.vval.v_number = 1;
        return;
    }
    // The line number is resolved against the named buffer, and a bad one
    // reports; only then is anything written.
    let lnum = unsafe { arg_lnum_buf(args, 1, buf) };
    if did_emsg.get() == did_emsg_before {
        unsafe { set_buffer_lines(buf, lnum, append, args.ptr(2), result) };
    }
}

/// Lines `start..=end` of `buf`, as a List or as one String.
///
/// # Safety
/// `buf` must be a live buffer or NULL, and `result` a live typval.
unsafe fn get_buffer_lines(
    buffer: *mut Buffer,
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
    if buffer.is_null() || unsafe { (*buffer).b_ml.ml_mfp }.is_null() || start < 0 || end < start {
        if retlist {
            unsafe { tv_list_alloc_ret(result, 0) };
        }
        return;
    }
    let buf = unsafe { Buf::new(buffer) };
    if !retlist {
        let len = |n| size_t::try_from(n).expect("a line length is not negative");
        let line = (start >= 1 && start <= buf.line_count())
            .then(|| unsafe { xstrnsave(buf.line(start).raw(), len(buf.line_len(start))) });
        ret.vval.v_string = line.unwrap_or(ptr::null_mut());
        return;
    }
    start = start.max(1);
    end = end.min(buf.line_count());
    let list = unsafe { tv_list_alloc_ret(result, (end - start + 1) as ptrdiff_t) };
    for lnum in start..=end {
        let (text, len) = unsafe { (buf.line(lnum).raw(), buf.line_len(lnum) as ssize_t) };
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
pub unsafe fn f_append(argvars: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(argvars, result);
    // SAFETY: the arguments and `result` are live typvals; `curbuf` is set.
    let did_emsg_before = did_emsg.get();
    let lnum = arg_lnum(args, 0);
    if did_emsg.get() == did_emsg_before {
        unsafe { set_buffer_lines(curbuf.get(), lnum, true, args.ptr(1), result) };
    }
}

/// `appendbufline({buf}, {lnum}, {string/list})`.
pub unsafe fn f_appendbufline(argvars: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(argvars, result);
    // SAFETY: the arguments and `result` are live typvals.
    unsafe { buf_set_append_line(args, result, true) };
}

/// `setbufline({buf}, {lnum}, {string/list})`.
pub unsafe fn f_setbufline(argvars: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(argvars, result);
    // SAFETY: the arguments and `result` are live typvals.
    unsafe { buf_set_append_line(args, result, false) };
}

/// `setline({lnum}, {string/list})`.
pub unsafe fn f_setline(argvars: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(argvars, result);
    // SAFETY: the arguments and `result` are live typvals; `curbuf` is set.
    let did_emsg_before = did_emsg.get();
    let lnum = arg_lnum(args, 0);
    if did_emsg.get() == did_emsg_before {
        unsafe { set_buffer_lines(curbuf.get(), lnum, false, args.ptr(1), result) };
    }
}

/// `getline({lnum} [, {end}])` — one String, or a List for a range.
pub unsafe fn f_getline(argvars: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(argvars, result);
    // SAFETY: the arguments and `result` are live typvals; `curbuf` is set.
    let lnum = arg_lnum(args, 0);
    // One argument answers a string, a range answers a list.
    let (end, retlist) = if args.has(1) {
        (arg_lnum(args, 1), true)
    } else {
        (lnum, false)
    };
    unsafe { get_buffer_lines(curbuf.get(), lnum, end, retlist, result) };
}

/// `getbufline({buf}, {lnum} [, {end}])`.
pub unsafe fn f_getbufline(argvars: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(argvars, result);
    // SAFETY: the arguments and `result` are live typvals.
    unsafe { getbufline(args, result, true) };
}

/// `getbufoneline({buf}, {lnum})`.
pub unsafe fn f_getbufoneline(argvars: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(argvars, result);
    // SAFETY: the arguments and `result` are live typvals.
    unsafe { getbufline(args, result, false) };
}

/// `deletebufline({buf}, {first} [, {last}])` — 0 when the lines went.
pub unsafe fn f_deletebufline(argvars: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(argvars, result);
    result.vval.v_number = 1;
    // SAFETY: the arguments and `result` are live typvals; `cob` is a live
    // local, restored on every path out of the change.
    let did_emsg_before = did_emsg.get();
    let buf = arg_buf(args, 0, 0);
    if buf.is_null() {
        return;
    }
    let first = unsafe { arg_lnum_buf(args, 1, buf) };
    if did_emsg.get() > did_emsg_before {
        return;
    }
    let mut last = if args.has(2) {
        unsafe { arg_lnum_buf(args, 2, buf) }
    } else {
        first
    };
    // SAFETY: `tv_get_buf` answers a live buffer or NULL, and the null was
    // returned above.
    let (mfp, count) = unsafe { ((*buf).b_ml.ml_mfp, (*buf).b_ml.ml_line_count) };
    if mfp.is_null() || first < 1 || first > count || last < first {
        return;
    }
    let is_curbuf = buf == curbuf.get();
    let mut cob = SavedBufferState::new();
    if !is_curbuf {
        unsafe { cob.prepare(Buf::new(buf)) };
    }
    last = last.min(cur_buf().line_count());
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
        check_cursor_col(cur_win());
        unsafe { deleted_lines_mark(first, count) };
        result.vval.v_number = 0;
    }
    if !is_curbuf {
        unsafe { cob.restore() };
    }
}
