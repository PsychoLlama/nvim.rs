use super::*;
use crate::types::{VAR_LIST, VAR_STRING, VAR_UNKNOWN};

/// Set line or list of lines in buffer "buf" to "lines".
/// Any type is allowed and converted to a string.
pub(crate) unsafe fn set_buffer_lines(
    buf: *mut buf_T,
    lnum_arg: linenr_T,
    append: bool,
    lines: *mut typval_T,
    rettv: *mut typval_T,
) {
    let mut lnum: linenr_T = lnum_arg + (if append { 1 } else { 0 });
    let mut added: c_int = 0;
    let is_curbuf: bool = buf == curbuf.get();
    if buf.is_null() || !is_curbuf && (*buf).b_ml.ml_mfp.is_null() || lnum < 1 {
        (*rettv).vval.v_number = 1;
        return;
    }
    let mut cob: SavedBufferState = mem::zeroed();
    if !is_curbuf {
        change_other_buffer_prepare(&raw mut cob, buf);
    }
    let append_lnum: linenr_T = if append {
        lnum - 1
    } else {
        (*curbuf.get()).b_ml.ml_line_count
    };
    let mut l: *mut list_T = ptr::null_mut();
    let mut li: *mut listitem_T = ptr::null_mut();
    let mut line: *mut c_char = ptr::null_mut();
    '_cleanup: {
        if (*lines).v_type == VAR_LIST {
            l = (*lines).vval.v_list;
            if l.is_null() || (*l).lv_len == 0 {
                break '_cleanup;
            } else {
                li = (*l).lv_first;
            }
        } else {
            line = typval_tostring(lines, false);
        }
        loop {
            if (*lines).v_type == VAR_LIST {
                if li.is_null() {
                    break;
                }
                xfree(line as *mut c_void);
                line = typval_tostring(&raw mut (*li).li_tv, false);
                li = (*li).li_next;
            }
            (*rettv).vval.v_number = 1;
            if line.is_null() || lnum > (*curbuf.get()).b_ml.ml_line_count + 1 {
                break;
            }
            if u_sync_once.get() == 2 {
                u_sync_once.set(1);
                u_sync(true);
            }
            if !append && lnum <= (*curbuf.get()).b_ml.ml_line_count {
                let mut old_len: c_int = strlen(ml_get(lnum)) as c_int;
                if u_savesub(lnum) == OK && ml_replace(lnum, line, true) == OK {
                    inserted_bytes(lnum, 0, old_len, strlen(line) as c_int);
                    if is_curbuf && lnum == (*curwin.get()).w_cursor.lnum {
                        check_cursor_col(curwin.get());
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
        xfree(line as *mut c_void);
        if added > 0 {
            appended_lines_mark(append_lnum, added);
            // Only the current window of the current buffer follows the
            // insertion; the others keep looking at the line they were on.
            for wp in tab_windows().map(Win::raw) {
                if (*wp).w_buffer == buf
                    && ((*wp).w_buffer != curbuf.get() || wp == curwin.get())
                    && (*wp).w_cursor.lnum > append_lnum
                {
                    (*wp).w_cursor.lnum += added as linenr_T;
                }
            }
            check_cursor_col(curwin.get());
            update_topline(curwin.get());
        }
    }
    if !is_curbuf {
        change_other_buffer_restore(&raw mut cob);
    }
}
/// Set or append lines to a buffer.
unsafe fn buf_set_append_line(argvars: *mut typval_T, rettv: *mut typval_T, append: bool) {
    let did_emsg_before: c_int = did_emsg.get();
    let buf: *mut buf_T = tv_get_buf(argvars.offset(0), false_0);
    if buf.is_null() {
        (*rettv).vval.v_number = 1;
    } else {
        let lnum: linenr_T = tv_get_lnum_buf(argvars.offset(1), buf);
        if did_emsg.get() == did_emsg_before {
            set_buffer_lines(buf, lnum, append, argvars.offset(2), rettv);
        }
    };
}
/// Get line or list of lines from buffer "buf" into "rettv".
///
/// `retlist` — if true, then the lines are returned as a Vim List.
///
/// Returns range (from start to end) of lines in rettv from the specified
///          buffer.
unsafe fn get_buffer_lines(
    buf: *mut buf_T,
    mut start: linenr_T,
    mut end: linenr_T,
    retlist: bool,
    rettv: *mut typval_T,
) {
    (*rettv).v_type = if retlist { VAR_LIST } else { VAR_STRING };
    (*rettv).vval.v_string = ptr::null_mut();
    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() || start < 0 || end < start {
        if retlist {
            tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
        }
        return;
    }
    if retlist {
        if start < 1 {
            start = 1;
        }
        if end > (*buf).b_ml.ml_line_count {
            end = (*buf).b_ml.ml_line_count;
        }
        tv_list_alloc_ret(rettv, (end - start + 1) as ptrdiff_t);
        while start <= end {
            tv_list_append_string(
                (*rettv).vval.v_list,
                ml_get_buf(buf, start),
                ml_get_buf_len(buf, start) as ssize_t,
            );
            start += 1;
        }
    } else {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = if start >= 1 && start <= (*buf).b_ml.ml_line_count {
            xstrnsave(ml_get_buf(buf, start), ml_get_buf_len(buf, start) as size_t)
        } else {
            ptr::null_mut()
        };
    };
}
/// `retlist` — true: "getbufline()" function
///                 false: "getbufoneline()" function
unsafe fn getbufline(argvars: *mut typval_T, rettv: *mut typval_T, retlist: bool) {
    let did_emsg_before: c_int = did_emsg.get();
    let buf: *mut buf_T = tv_get_buf_from_arg(argvars.offset(0));
    let lnum: linenr_T = tv_get_lnum_buf(argvars.offset(1), buf);
    if did_emsg.get() > did_emsg_before {
        return;
    }
    let end: linenr_T = if (*argvars.offset(2)).v_type == VAR_UNKNOWN {
        lnum
    } else {
        tv_get_lnum_buf(argvars.offset(2), buf)
    };
    get_buffer_lines(buf, lnum, end, retlist, rettv);
}
/// "append(lnum, string/list)" function
pub unsafe fn f_append(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let did_emsg_before: c_int = did_emsg.get();
    let lnum: linenr_T = tv_get_lnum(argvars.offset(0));
    if did_emsg.get() == did_emsg_before {
        set_buffer_lines(curbuf.get(), lnum, true, argvars.offset(1), rettv);
    }
}
/// "appendbufline(buf, lnum, string/list)" function
pub unsafe fn f_appendbufline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    buf_set_append_line(argvars, rettv, true);
}
/// "setbufline()" function
pub unsafe fn f_setbufline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    buf_set_append_line(argvars, rettv, false);
}
/// "setline()" function
pub unsafe fn f_setline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let did_emsg_before: c_int = did_emsg.get();
    let lnum: linenr_T = tv_get_lnum(argvars.offset(0));
    if did_emsg.get() == did_emsg_before {
        set_buffer_lines(curbuf.get(), lnum, false, argvars.offset(1), rettv);
    }
}
/// "getline(lnum, [end])" function
pub unsafe fn f_getline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let lnum: linenr_T = tv_get_lnum(argvars);
    // One argument answers a string, a range answers a list.
    let (end, retlist) = if (*argvars.offset(1)).v_type == VAR_UNKNOWN {
        (lnum, false)
    } else {
        (tv_get_lnum(argvars.offset(1)), true)
    };
    get_buffer_lines(curbuf.get(), lnum, end, retlist, rettv);
}
/// "getbufline()" function
pub unsafe fn f_getbufline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    getbufline(argvars, rettv, true);
}
/// "getbufoneline()" function
pub unsafe fn f_getbufoneline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    getbufline(argvars, rettv, false);
}
/// "deletebufline()" function
pub unsafe fn f_deletebufline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let did_emsg_before: c_int = did_emsg.get();
    (*rettv).vval.v_number = 1;
    let buf: *mut buf_T = tv_get_buf(argvars.offset(0), false_0);
    if buf.is_null() {
        return;
    }
    let first: linenr_T = tv_get_lnum_buf(argvars.offset(1), buf);
    if did_emsg.get() > did_emsg_before {
        return;
    }
    let mut last: linenr_T = if (*argvars.offset(2)).v_type != VAR_UNKNOWN {
        tv_get_lnum_buf(argvars.offset(2), buf)
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
    let is_curbuf: bool = buf == curbuf.get();
    let mut cob: SavedBufferState = mem::zeroed();
    if !is_curbuf {
        change_other_buffer_prepare(&raw mut cob, buf);
    }
    if last > (*curbuf.get()).b_ml.ml_line_count {
        last = (*curbuf.get()).b_ml.ml_line_count;
    }
    let count: c_int = last as c_int - first as c_int + 1;
    if u_sync_once.get() == 2 {
        u_sync_once.set(1);
        u_sync(true);
    }
    if u_save(first - 1, last + 1) != FAIL {
        // Every delete takes the same line number: the lines below move up.
        for _ in first..=last {
            ml_delete_flags(first, ML_DEL_MESSAGE as c_int);
        }
        // Pull every cursor that was inside or after the deleted range back
        // onto a line that still exists.
        for wp in tab_windows().map(Win::raw) {
            if (*wp).w_buffer == buf {
                if (*wp).w_cursor.lnum > last {
                    (*wp).w_cursor.lnum -= count as linenr_T;
                } else if (*wp).w_cursor.lnum > first {
                    (*wp).w_cursor.lnum = first;
                }
                if (*wp).w_cursor.lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
                    (*wp).w_cursor.lnum = (*(*wp).w_buffer).b_ml.ml_line_count;
                }
            }
        }
        check_cursor_col(curwin.get());
        deleted_lines_mark(first, count);
        (*rettv).vval.v_number = 0;
    }
    if !is_curbuf {
        change_other_buffer_restore(&raw mut cob);
    }
}
