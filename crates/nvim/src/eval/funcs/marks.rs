//! Marks, jumps, changes and tags.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::tv_get_buf;
use crate::eval::typval::{
    tv_check_for_dict_arg, tv_check_for_string_arg, tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc,
    tv_dict_alloc_ret, tv_get_number, tv_get_string, tv_get_string_chk, tv_list_alloc,
    tv_list_alloc_ret, tv_list_append_dict, tv_list_append_list, tv_list_append_number,
    tv_list_append_string,
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
    typval_T, varnumber_T, win_T,
};
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
    unsafe {
        let d = tv_dict_alloc();
        tv_list_append_dict(l, d);
        tv_dict_add_nr(d, c"lnum".as_ptr(), 4, mark.lnum as varnumber_T);
        tv_dict_add_nr(d, c"col".as_ptr(), 3, mark.col as varnumber_T);
        tv_dict_add_nr(d, c"coladd".as_ptr(), 6, mark.coladd as varnumber_T);
        d
    }
}

/// `getchangelist([{buf}])` — `[changes, index]`.
pub unsafe fn f_getchangelist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; `curwin` and its
    // buffer's window-info vector are live for the whole call.
    unsafe {
        let out = tv_list_alloc_ret(rettv, 2);
        let buf: *const buf_T = if !args.has(0) {
            curbuf.get()
        } else {
            // The value is coerced to a Number purely so that a bad type
            // reports; the result is thrown away and the argument is
            // resolved as a buffer instead.
            vim_ignored.set(tv_get_number(args.ptr(0)) as c_int);
            let _no_emsg = Suppress::emsg();
            tv_get_buf(args.ptr(0), 0)
        };
        if buf.is_null() {
            return;
        }
        let l = tv_list_alloc((*buf).b_changelistlen as isize);
        tv_list_append_list(out, l);

        // The index is this window's if it is showing the buffer, and
        // otherwise the one remembered for this window in the buffer's
        // window-info list. A buffer this window has never shown reports
        // the end of the list.
        let index = if buf == (*curwin.get()).w_buffer as *const buf_T {
            (*curwin.get()).w_changelistidx
        } else {
            (0..(*buf).b_wininfo.size)
                .map(|i| *(*buf).b_wininfo.items.add(i))
                .find(|wip| (**wip).wi_win == curwin.get())
                .map_or((*buf).b_changelistlen, |wip| (*wip).wi_changelistidx)
        };
        tv_list_append_number(out, index as varnumber_T);

        for i in 0..(*buf).b_changelistlen {
            let mark = (*buf).b_changelist[i as usize].mark;
            if mark.lnum != 0 {
                append_mark(l, mark);
            }
        }
    }
}

/// `getjumplist([{winnr} [, {tabnr}]])` — `[jumps, index]`.
pub unsafe fn f_getjumplist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals, and the jump
    // list is compacted before it is read so no entry is stale.
    unsafe {
        let out = tv_list_alloc_ret(rettv, kListLenMayKnow as isize);
        let wp: *mut win_T = find_tabwin(args.ptr(0), args.ptr(1));
        if wp.is_null() {
            return;
        }
        cleanup_jumplist(wp, true);
        let l = tv_list_alloc((*wp).w_jumplistlen as isize);
        tv_list_append_list(out, l);
        tv_list_append_number(out, (*wp).w_jumplistidx as varnumber_T);
        for i in 0..(*wp).w_jumplistlen {
            let entry = &(*wp).w_jumplist[i as usize];
            if entry.fmark.mark.lnum == 0 {
                continue;
            }
            let d = append_mark(l, entry.fmark.mark);
            tv_dict_add_nr(d, c"bufnr".as_ptr(), 5, entry.fmark.fnum as varnumber_T);
            // A jump into a file that is no longer loaded keeps its name.
            if !entry.fname.is_null() {
                tv_dict_add_str(d, c"filename".as_ptr(), 8, entry.fname);
            }
        }
    }
}

/// `getmarklist([{buf}])` — the global marks, or one buffer's local ones.
pub unsafe fn f_getmarklist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe {
        let out = tv_list_alloc_ret(rettv, kListLenMayKnow as isize);
        if !args.has(0) {
            get_global_marks(out);
            return;
        }
        let buf = tv_get_buf(args.ptr(0), 0);
        if buf.is_null() {
            return;
        }
        get_buf_local_marks(buf, out);
    }
}

/// `gettagstack([{winnr}])`.
pub unsafe fn f_gettagstack(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals. The dict is
    // allocated before the window is resolved, so a bad window still
    // yields an empty dict rather than nothing.
    unsafe {
        tv_dict_alloc_ret(rettv);
        let wp = if !args.has(0) {
            curwin.get()
        } else {
            find_win_by_nr_or_id(args.ptr(0))
        };
        if wp.is_null() {
            return;
        }
        get_tagstack(wp, rettv.vval.v_dict);
    }
}

/// `settagstack({winnr}, {dict} [, {action}])`.
pub unsafe fn f_settagstack(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1;
    // SAFETY: the arguments are live typvals; after the check argument 1's
    // union holds a Dict pointer, which may still be null.
    unsafe {
        let wp = find_win_by_nr_or_id(args.ptr(0));
        if wp.is_null() || tv_check_for_dict_arg(args.ptr(0), 1) == FAIL {
            return;
        }
        let d = args.get(1).vval.v_dict;
        if d.is_null() {
            return;
        }
        // "r" replaces, "a" appends, "t" truncates; anything else, including
        // a longer string starting with one of them, is E962.
        let mut action = b'r' as c_char;
        if args.has(2) {
            if tv_check_for_string_arg(args.ptr(0), 2) == FAIL {
                return;
            }
            let actstr = tv_get_string_chk(args.ptr(2));
            if actstr.is_null() {
                return;
            }
            match CStr::from_ptr(actstr).to_bytes() {
                b"r" | b"a" | b"t" => action = *actstr,
                _ => {
                    semsg_c!(gettext(c"E962: Invalid action: '%s'".as_ptr()), actstr);
                    return;
                }
            }
        }
        if set_tagstack(wp, d, action as c_int) == OK {
            rettv.vval.v_number = 0;
        }
    }
}

/// `tagfiles()` — the tags files that would be searched, in order.
pub unsafe fn f_tagfiles(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value; each name the walk
    // answers is NUL-terminated and lives until the next round.
    unsafe {
        let out = tv_list_alloc_ret(rettv, kListLenUnknown as isize);
        let mut files = TagFiles::new();
        while let Some(name) = files.next() {
            tv_list_append_string(out, name.as_ptr(), -1);
        }
    }
}

/// `taglist({expr} [, {filename}])`.
pub unsafe fn f_taglist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; both strings are
    // NUL-terminated and outlive the search.
    unsafe {
        let pattern = tv_get_string(args.ptr(0));
        // An empty pattern answers 0 — a Number, not an empty List.
        rettv.vval.v_number = 0;
        if *pattern == NUL as c_char {
            return;
        }
        let fname = if args.has(1) {
            tv_get_string(args.ptr(1))
        } else {
            ptr::null()
        };
        get_tags(
            tv_list_alloc_ret(rettv, kListLenUnknown as isize),
            pattern as *mut c_char,
            fname as *mut c_char,
        );
    }
}
