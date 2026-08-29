//! Marks, jumps, changes and tags.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::tv_get_buf;
use super::wrappers::{
    arg_number, arg_string, arg_string_chk, check_arg, dict_alloc_ret, list_alloc_ret,
};
use crate::eval::typval::{
    NumBuf, tv_check_for_dict_arg, tv_check_for_string_arg, tv_dict_add_nr, tv_dict_add_str,
    tv_dict_alloc, tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict, tv_list_append_list,
    tv_list_append_number, tv_list_append_string,
};
use crate::eval::window::{find_tabwin, find_win_by_nr_or_id};
use crate::guard::Suppress;
use crate::main::{curbuf, curwin, vim_ignored};
use crate::mark::{cleanup_jumplist, get_buf_local_marks, get_global_marks};
use crate::os::cshim::gettext;
use crate::semsg_c;
use crate::tag::{TagFiles, get_tags, get_tagstack, set_tagstack};
use crate::types::{
    EvalFuncData, FAIL, NUL, OK, buf_T, dict_T, kListLenMayKnow, kListLenUnknown, list_T, pos_T,
    typval_T, varnumber_T,
};
use crate::winlayer::Win;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// `changenr()` — the sequence number of the change the undo tree is at.
pub unsafe fn f_changenr(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `curbuf` is live and `rettv` is the cleared return value.
    unsafe { (*rettv).vval.v_number = (*curbuf.get()).b_u_seq_cur as varnumber_T };
}

/// Add one `{lnum, col, coladd}` entry to `l`, skipping a cleared mark.
///
/// # Safety
/// `l` is a live list.
unsafe fn append_mark(l: *mut list_T, mark: pos_T) -> *mut dict_T {
    // SAFETY: the caller's obligation; the dict is handed to the list
    // immediately, so it is not leaked.
    let d = unsafe { tv_dict_alloc() };
    unsafe { tv_list_append_dict(l, d) };
    unsafe { tv_dict_add_nr(d, c"lnum".as_ptr(), 4, mark.lnum as varnumber_T) };
    unsafe { tv_dict_add_nr(d, c"col".as_ptr(), 3, mark.col as varnumber_T) };
    unsafe { tv_dict_add_nr(d, c"coladd".as_ptr(), 6, mark.coladd as varnumber_T) };
    d
}

/// `getchangelist([{buf}])` — `[changes, index]`.
pub unsafe fn f_getchangelist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY throughout: the arguments and `rettv` are live typvals; `curwin` and its
    // buffer's window-info vector are live for the whole call.
    let out = list_alloc_ret(rettv, 2);
    let buf: *const buf_T = if !args.has(0) {
        curbuf.get()
    } else {
        // The value is coerced to a Number purely so that a bad type
        // reports; the result is thrown away and the argument is
        // resolved as a buffer instead.
        vim_ignored.set(arg_number(args.get(0)) as c_int);
        let _no_emsg = Suppress::emsg();
        unsafe { tv_get_buf(args.ptr(0), 0) }
    };
    if buf.is_null() {
        return;
    }
    let l = unsafe { tv_list_alloc((*buf).b_changelistlen as isize) };
    unsafe { tv_list_append_list(out, l) };

    // The index is this window's if it is showing the buffer, and
    // otherwise the one remembered for this window in the buffer's
    // window-info list. A buffer this window has never shown reports
    // the end of the list.
    let index = if ptr::eq(buf, unsafe { (*curwin.get()).w_buffer }) {
        unsafe { (*curwin.get()).w_changelistidx }
    } else {
        (0..unsafe { (*buf).b_wininfo.size })
            .map(|i| unsafe { *(*buf).b_wininfo.items.add(i) })
            .find(|wip| unsafe { (**wip).wi_win } == curwin.get())
            .map_or(unsafe { (*buf).b_changelistlen }, |wip| unsafe {
                (*wip).wi_changelistidx
            })
    };
    unsafe { tv_list_append_number(out, index as varnumber_T) };

    for i in 0..unsafe { (*buf).b_changelistlen } {
        let mark = unsafe { (*buf).b_changelist[i as usize].mark };
        if mark.lnum != 0 {
            unsafe { append_mark(l, mark) };
        }
    }
}

/// `getjumplist([{winnr} [, {tabnr}]])` — `[jumps, index]`.
pub unsafe fn f_getjumplist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY throughout: the arguments and `rettv` are live typvals, and the jump
    // list is compacted before it is read so no entry is stale.
    let out = list_alloc_ret(rettv, kListLenMayKnow as isize);
    let wp = unsafe { find_tabwin(args.ptr(0), args.ptr(1)) }.map_or(ptr::null_mut(), Win::raw);
    if wp.is_null() {
        return;
    }
    unsafe { cleanup_jumplist(wp, true) };
    let l = unsafe { tv_list_alloc((*wp).w_jumplistlen as isize) };
    unsafe { tv_list_append_list(out, l) };
    unsafe { tv_list_append_number(out, (*wp).w_jumplistidx as varnumber_T) };
    for i in 0..unsafe { (*wp).w_jumplistlen } {
        let entry = unsafe { &(*wp).w_jumplist[i as usize] };
        if entry.fmark.mark.lnum == 0 {
            continue;
        }
        let d = unsafe { append_mark(l, entry.fmark.mark) };
        unsafe { tv_dict_add_nr(d, c"bufnr".as_ptr(), 5, entry.fmark.fnum as varnumber_T) };
        // A jump into a file that is no longer loaded keeps its name.
        if !entry.fname.is_null() {
            unsafe { tv_dict_add_str(d, c"filename".as_ptr(), 8, entry.fname) };
        }
    }
}

/// `getmarklist([{buf}])` — the global marks, or one buffer's local ones.
pub unsafe fn f_getmarklist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY throughout: the arguments and `rettv` are live typvals.
    let out = list_alloc_ret(rettv, kListLenMayKnow as isize);
    if !args.has(0) {
        unsafe { get_global_marks(out) };
        return;
    }
    let buf = unsafe { tv_get_buf(args.ptr(0), 0) };
    if buf.is_null() {
        return;
    }
    unsafe { get_buf_local_marks(buf, out) };
}

/// `gettagstack([{winnr}])`.
pub unsafe fn f_gettagstack(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY throughout: the arguments and `rettv` are live typvals. The dict is
    // allocated before the window is resolved, so a bad window still
    // yields an empty dict rather than nothing.
    dict_alloc_ret(rettv);
    let found = if !args.has(0) {
        unsafe { Win::from_raw(curwin.get()) }
    } else {
        unsafe { find_win_by_nr_or_id(args.ptr(0)) }
    };
    let Some(wp) = found else {
        return;
    };
    unsafe { get_tagstack(wp, rettv.vval.v_dict) };
}

/// `settagstack({winnr}, {dict} [, {action}])`.
pub unsafe fn f_settagstack(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1;
    // SAFETY: the arguments are live typvals; after the check argument 1's
    // union holds a Dict pointer, which may still be null.
    let found = unsafe { find_win_by_nr_or_id(args.ptr(0)) };
    let Some(wp) = found.filter(|_| check_arg(args, 1, tv_check_for_dict_arg) != FAIL) else {
        return;
    };
    let d = unsafe { args.get(1).vval.v_dict };
    if d.is_null() {
        return;
    }
    // "r" replaces, "a" appends, "t" truncates; anything else, including
    // a longer string starting with one of them, is E962.
    let mut action = b'r' as c_char;
    if args.has(2) {
        if check_arg(args, 2, tv_check_for_string_arg) == FAIL {
            return;
        }
        let actstr = arg_string_chk(&mut numbuf, args.get(2));
        if actstr.is_null() {
            return;
        }
        match unsafe { CStr::from_ptr(actstr) }.to_bytes() {
            b"r" | b"a" | b"t" => action = unsafe { *actstr },
            _ => {
                unsafe { semsg_c!(gettext(c"E962: Invalid action: '%s'"), actstr) };
                return;
            }
        }
    }
    if unsafe { set_tagstack(wp, d, action as c_int) } == OK {
        rettv.vval.v_number = 0;
    }
}

/// `tagfiles()` — the tags files that would be searched, in order.
pub unsafe fn f_tagfiles(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value; each name the walk
    // answers is NUL-terminated and lives until the next round.
    let out = unsafe { tv_list_alloc_ret(rettv, kListLenUnknown as isize) };
    let mut files = TagFiles::new();
    while let Some(name) = files.next() {
        unsafe { tv_list_append_string(out, name.as_ptr(), -1) };
    }
}

/// `taglist({expr} [, {filename}])`.
pub unsafe fn f_taglist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY throughout: the arguments and `rettv` are live typvals; both strings are
    // NUL-terminated and outlive the search.
    let pattern = arg_string(&mut numbuf, args.get(0));
    // An empty pattern answers 0 — a Number, not an empty List.
    rettv.vval.v_number = 0;
    if unsafe { *pattern } == NUL as c_char {
        return;
    }
    let fname = if args.has(1) {
        arg_string(&mut numbuf2, args.get(1))
    } else {
        ptr::null()
    };
    let list = list_alloc_ret(rettv, kListLenUnknown as isize);
    let (pat, file) = (pattern as *mut c_char, fname as *mut c_char);
    unsafe { get_tags(list, pat, file) };
}
