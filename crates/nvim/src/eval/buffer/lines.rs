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
use crate::narrow::len_as_int;
use crate::types::{VAR_LIST, VAR_STRING};

/// Set or append lines in buffer `buf`, from `lines` — any type, converted to
/// a string, or a List of them.
///
/// `rettv` ends 0 when every line went in and 1 otherwise, which is what all
/// four builtins answer.
///
/// # Safety
/// `buf` must be a live buffer or NULL, and `lines`/`rettv` live typvals.
pub(crate) unsafe fn set_buffer_lines(
    buf: *mut buf_T,
    lnum_arg: linenr_T,
    append: bool,
    lines: *mut typval_T,
    rettv: *mut typval_T,
) {
    // SAFETY: the caller's obligation. `cob` is a live local, restored on
    // every path out; `line` is owned here and freed before each replacement
    // and once at the end.
    unsafe {
        let mut lnum: linenr_T = lnum_arg + linenr_T::from(append);
        let mut added: c_int = 0;
        let is_curbuf: bool = buf == curbuf.get();
        if buf.is_null() || !is_curbuf && (*buf).b_ml.ml_mfp.is_null() || lnum < 1 {
            (*rettv).vval.v_number = 1;
            return;
        }
        let mut cob = SavedBufferState::new();
        if !is_curbuf {
            cob.prepare(Buf::new(buf));
        }
        let append_lnum: linenr_T = if append {
            lnum - 1
        } else {
            Buf::current().line_count()
        };
        let mut l: *mut list_T = ptr::null_mut();
        let mut li: *mut listitem_T = ptr::null_mut();
        let mut line: *mut c_char = ptr::null_mut();
        '_cleanup: {
            if (*lines).v_type == VAR_LIST {
                l = (*lines).vval.v_list;
                if l.is_null() || (*l).lv_len == 0 {
                    break '_cleanup;
                }
                li = (*l).lv_first;
            } else {
                line = typval_tostring(lines, false);
            }
            loop {
                if (*lines).v_type == VAR_LIST {
                    if li.is_null() {
                        break;
                    }
                    xfree(line.cast());
                    line = typval_tostring(&raw mut (*li).li_tv, false);
                    li = (*li).li_next;
                }
                (*rettv).vval.v_number = 1;
                if line.is_null() || lnum > Buf::current().line_count() + 1 {
                    break;
                }
                if u_sync_once.get() == 2 {
                    u_sync_once.set(1);
                    u_sync(true);
                }
                if !append && lnum <= Buf::current().line_count() {
                    let old_len = len_as_int(strlen(ml_get(lnum)));
                    if u_savesub(lnum) == OK && ml_replace(lnum, line, true) == OK {
                        inserted_bytes(lnum, 0, old_len, len_as_int(strlen(line)));
                        if is_curbuf && lnum == Win::current().w_cursor.lnum {
                            check_cursor_col(Win::current());
                        }
                        (*rettv).vval.v_number = 0;
                    }
                } else if added > 0 || u_save(lnum - 1, lnum) == OK {
                    added += 1;
                    if ml_append(lnum - 1, line, 0, false) == OK {
                        (*rettv).vval.v_number = 0;
                    }
                }
                if l.is_null() {
                    break;
                }
                lnum += 1;
            }
            xfree(line.cast());
            if added > 0 {
                appended_lines_mark(append_lnum, added);
                // Only the current window of the current buffer follows the
                // insertion; the others keep looking at the line they were on.
                for mut wp in tab_windows() {
                    if wp.w_buffer == buf
                        && (wp.w_buffer != curbuf.get() || wp.is_current())
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
}

/// `setbufline()` and `appendbufline()`, which differ only in `append`.
///
/// # Safety
/// The arguments and `rettv` must be live typvals.
unsafe fn buf_set_append_line(args: Args<'_>, rettv: &mut typval_T, append: bool) {
    // SAFETY: the caller's obligation.
    unsafe {
        let did_emsg_before = did_emsg.get();
        let buf = tv_get_buf(args.ptr(0), 0);
        if buf.is_null() {
            rettv.vval.v_number = 1;
            return;
        }
        // The line number is resolved against the named buffer, and a bad one
        // reports; only then is anything written.
        let lnum = tv_get_lnum_buf(args.ptr(1), buf);
        if did_emsg.get() == did_emsg_before {
            set_buffer_lines(buf, lnum, append, args.ptr(2), rettv);
        }
    }
}

/// Lines `start..=end` of `buf`, as a List or as one String.
///
/// # Safety
/// `buf` must be a live buffer or NULL, and `rettv` a live typval.
unsafe fn get_buffer_lines(
    buf: *mut buf_T,
    mut start: linenr_T,
    mut end: linenr_T,
    retlist: bool,
    rettv: *mut typval_T,
) {
    // SAFETY: the caller's obligation; every line index is clamped to the
    // buffer before `ml_get_buf` sees it.
    unsafe {
        (*rettv).v_type = if retlist { VAR_LIST } else { VAR_STRING };
        (*rettv).vval.v_string = ptr::null_mut();
        if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() || start < 0 || end < start {
            if retlist {
                tv_list_alloc_ret(rettv, 0);
            }
            return;
        }
        let buf = Buf::new(buf);
        if !retlist {
            let len = |n| size_t::try_from(n).expect("a line length is not negative");
            let line = (start >= 1 && start <= buf.line_count())
                .then(|| xstrnsave(buf.line(start).raw(), len(buf.line_len(start))));
            (*rettv).vval.v_string = line.unwrap_or(ptr::null_mut());
            return;
        }
        start = start.max(1);
        end = end.min(buf.line_count());
        let list = tv_list_alloc_ret(rettv, (end - start + 1) as ptrdiff_t);
        for lnum in start..=end {
            tv_list_append_string(list, buf.line(lnum).raw(), buf.line_len(lnum) as ssize_t);
        }
    }
}

/// `getbufline()` when `retlist`, `getbufoneline()` otherwise.
///
/// # Safety
/// The arguments and `rettv` must be live typvals.
unsafe fn getbufline(args: Args<'_>, rettv: &mut typval_T, retlist: bool) {
    // SAFETY: the caller's obligation.
    unsafe {
        let did_emsg_before = did_emsg.get();
        let buf = tv_get_buf_from_arg(args.ptr(0));
        let lnum = tv_get_lnum_buf(args.ptr(1), buf);
        if did_emsg.get() > did_emsg_before {
            return;
        }
        let end = if args.has(2) {
            tv_get_lnum_buf(args.ptr(2), buf)
        } else {
            lnum
        };
        get_buffer_lines(buf, lnum, end, retlist, rettv);
    }
}

/// `append({lnum}, {string/list})`.
pub unsafe fn f_append(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; `curbuf` is set.
    unsafe {
        let did_emsg_before = did_emsg.get();
        let lnum = tv_get_lnum(args.ptr(0));
        if did_emsg.get() == did_emsg_before {
            set_buffer_lines(curbuf.get(), lnum, true, args.ptr(1), rettv);
        }
    }
}

/// `appendbufline({buf}, {lnum}, {string/list})`.
pub unsafe fn f_appendbufline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { buf_set_append_line(args, rettv, true) };
}

/// `setbufline({buf}, {lnum}, {string/list})`.
pub unsafe fn f_setbufline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { buf_set_append_line(args, rettv, false) };
}

/// `setline({lnum}, {string/list})`.
pub unsafe fn f_setline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; `curbuf` is set.
    unsafe {
        let did_emsg_before = did_emsg.get();
        let lnum = tv_get_lnum(args.ptr(0));
        if did_emsg.get() == did_emsg_before {
            set_buffer_lines(curbuf.get(), lnum, false, args.ptr(1), rettv);
        }
    }
}

/// `getline({lnum} [, {end}])` — one String, or a List for a range.
pub unsafe fn f_getline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; `curbuf` is set.
    unsafe {
        let lnum = tv_get_lnum(args.ptr(0));
        // One argument answers a string, a range answers a list.
        let (end, retlist) = if args.has(1) {
            (tv_get_lnum(args.ptr(1)), true)
        } else {
            (lnum, false)
        };
        get_buffer_lines(curbuf.get(), lnum, end, retlist, rettv);
    }
}

/// `getbufline({buf}, {lnum} [, {end}])`.
pub unsafe fn f_getbufline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { getbufline(args, rettv, true) };
}

/// `getbufoneline({buf}, {lnum})`.
pub unsafe fn f_getbufoneline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { getbufline(args, rettv, false) };
}

/// `deletebufline({buf}, {first} [, {last}])` — 0 when the lines went.
pub unsafe fn f_deletebufline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = 1;
    // SAFETY: the arguments and `rettv` are live typvals; `cob` is a live
    // local, restored on every path out of the change.
    unsafe {
        let did_emsg_before = did_emsg.get();
        let buf = tv_get_buf(args.ptr(0), 0);
        if buf.is_null() {
            return;
        }
        let first = tv_get_lnum_buf(args.ptr(1), buf);
        if did_emsg.get() > did_emsg_before {
            return;
        }
        let mut last = if args.has(2) {
            tv_get_lnum_buf(args.ptr(2), buf)
        } else {
            first
        };
        if (*buf).b_ml.ml_mfp.is_null()
            || first < 1
            || first > (*buf).b_ml.ml_line_count
            || last < first
        {
            return;
        }
        let is_curbuf = buf == curbuf.get();
        let mut cob = SavedBufferState::new();
        if !is_curbuf {
            cob.prepare(Buf::new(buf));
        }
        last = last.min(Buf::current().line_count());
        let count = last - first + 1;
        if u_sync_once.get() == 2 {
            u_sync_once.set(1);
            u_sync(true);
        }
        if u_save(first - 1, last + 1) != FAIL {
            // Every delete takes the same line number: the lines below move
            // up.
            for _ in first..=last {
                ml_delete_flags(first, ML_DEL_MESSAGE);
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
            rettv.vval.v_number = 0;
        }
        if !is_curbuf {
            cob.restore();
        }
    }
}
