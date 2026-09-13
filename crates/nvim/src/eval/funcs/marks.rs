//! Marks, jumps, changes and tags.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::tv_get_buf;
use super::wrappers::{arg_number, arg_string, arg_string_chk, dict_alloc_ret, list_alloc_ret};
use crate::eval::typval::{
    NumBuf, tv_check_for_dict_arg, tv_check_for_string_arg, tv_dict_add_nr, tv_dict_add_str,
    tv_dict_alloc, tv_list_alloc, tv_list_alloc_ret,
};
use crate::eval::window::{find_tabwin, find_win_by_nr_or_id};
use crate::guard::Suppress;
use crate::mark::{cleanup_jumplist, get_buf_local_marks, get_global_marks};
use crate::message_fmt::c_str;
use crate::semsg;
use crate::startup::vim_ignored;
use crate::tag::{TagFiles, get_tags, get_tagstack, set_tagstack};
use crate::types::{
    Dict, EvalFuncData, List, NUL, Pos, TypVal, VarNumber, kListLenMayKnow, kListLenUnknown,
};
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// `changenr()` — the sequence number of the change the undo tree is at.
pub fn f_changenr(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `curbuf` is live and `result` is the cleared return value.
    result.write_number(Buf::current().b_u_seq_cur as VarNumber);
}

/// Add one `{lnum, col, coladd}` entry to `l`, skipping a cleared mark.
///
/// # Safety
/// `l` is a live list.
unsafe fn append_mark(l: *mut List, mark: Pos) -> *mut Dict {
    // SAFETY: the caller's obligation; the dict is handed to the list
    // immediately, so it is not leaked.
    let d_held = tv_dict_alloc();
    let d = d_held.as_ptr();
    unsafe { (*l).push_dict(Some(d_held)) };
    let _ = unsafe { tv_dict_add_nr(d, c"lnum".as_ptr(), 4, mark.lnum as VarNumber) };
    let _ = unsafe { tv_dict_add_nr(d, c"col".as_ptr(), 3, mark.col as VarNumber) };
    let _ = unsafe { tv_dict_add_nr(d, c"coladd".as_ptr(), 6, mark.coladd as VarNumber) };
    d
}

/// `getchangelist([{buf}])` — `[changes, index]`.
pub fn f_getchangelist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the arguments and `result` are live typvals; `curwin` and its
    // buffer's window-info vector are live for the whole call.
    let out = list_alloc_ret(result, 2);
    let buf = if args.is_empty() {
        Buf::current_or_none()
    } else {
        // The value is coerced to a Number purely so that a bad type
        // reports; the result is thrown away and the argument is
        // resolved as a buffer instead.
        vim_ignored.set(arg_number(&args[0]) as c_int);
        let _no_emsg = Suppress::emsg();
        tv_get_buf(&args[0], 0)
    };
    let Some(buf) = buf else {
        return;
    };
    let entries = tv_list_alloc(buf.b_changelistlen as isize);
    let l = entries.as_ptr();
    unsafe { (*out).push_list(Some(entries)) };

    // The index is this window's if it is showing the buffer, and
    // otherwise the one remembered for this window in the buffer's
    // window-info list. A buffer this window has never shown reports
    // the end of the list.
    let index = if buf == Win::current().buffer() {
        Win::current().w_changelistidx
    } else {
        (0..buf.b_wininfo.size)
            .map(|i| unsafe { *buf.b_wininfo.items.add(i) })
            .find(|wip| unsafe { (**wip).wi_win } == Win::current_raw())
            .map_or(buf.b_changelistlen, |wip| unsafe {
                (*wip).wi_changelistidx
            })
    };
    unsafe { (*out).push_number(index as VarNumber) };

    for i in 0..buf.b_changelistlen {
        let mark = buf.b_changelist[i as usize].mark;
        if mark.lnum != 0 {
            unsafe { append_mark(l, mark) };
        }
    }
}

/// `getjumplist([{winnr} [, {tabnr}]])` — `[jumps, index]`.
pub fn f_getjumplist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the arguments and `result` are live typvals, and the jump
    // list is compacted before it is read so no entry is stale.
    let out = list_alloc_ret(result, kListLenMayKnow as isize);
    let Some(wp) = find_tabwin(args.first(), args.get(1)) else {
        return;
    };
    cleanup_jumplist(wp, true);
    let entries = tv_list_alloc(wp.w_jumplistlen as isize);
    let l = entries.as_ptr();
    unsafe { (*out).push_list(Some(entries)) };
    unsafe { (*out).push_number(wp.w_jumplistidx as VarNumber) };
    for i in 0..wp.w_jumplistlen {
        let entry = &wp.w_jumplist[i as usize];
        if entry.fmark.mark.lnum == 0 {
            continue;
        }
        let d = unsafe { append_mark(l, entry.fmark.mark) };
        let _ = unsafe { tv_dict_add_nr(d, c"bufnr".as_ptr(), 5, entry.fmark.fnum as VarNumber) };
        // A jump into a file that is no longer loaded keeps its name.
        if !entry.fname.is_null() {
            let _ = unsafe { tv_dict_add_str(d, c"filename".as_ptr(), 8, entry.fname) };
        }
    }
}

/// `getmarklist([{buf}])` — the global marks, or one buffer's local ones.
pub fn f_getmarklist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the arguments and `result` are live typvals.
    let out = list_alloc_ret(result, kListLenMayKnow as isize);
    if args.is_empty() {
        unsafe { get_global_marks(out) };
        return;
    }
    let buf = tv_get_buf(&args[0], 0);
    if buf.is_none() {
        return;
    }
    unsafe { get_buf_local_marks(buf.expect("a live handle"), out) };
}

/// `gettagstack([{winnr}])`.
pub fn f_gettagstack(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the arguments and `result` are live typvals. The dict is
    // allocated before the window is resolved, so a bad window still
    // yields an empty dict rather than nothing.
    dict_alloc_ret(result);
    let found = if args.is_empty() {
        unsafe { Win::from_raw(Win::current_raw()) }
    } else {
        find_win_by_nr_or_id(&args[0])
    };
    let Some(wp) = found else {
        return;
    };
    unsafe { get_tagstack(wp, result.dict_or_null()) };
}

/// `settagstack({winnr}, {dict} [, {action}])`.
pub fn f_settagstack(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(-1);
    let found = find_win_by_nr_or_id(&args[0]);
    let Some(wp) = found.filter(|_| tv_check_for_dict_arg(args, 1).is_ok()) else {
        return;
    };
    let d = args[1].dict_or_null();
    if d.is_null() {
        return;
    }
    // "r" replaces, "a" appends, "t" truncates; anything else, including
    // a longer string starting with one of them, is E962.
    let mut action = b'r' as c_char;
    if args.len() > 2 {
        if tv_check_for_string_arg(args, 2).is_err() {
            return;
        }
        let actstr = arg_string_chk(&mut numbuf, &args[2]);
        if actstr.is_null() {
            return;
        }
        match unsafe { CStr::from_ptr(actstr) }.to_bytes() {
            b"r" | b"a" | b"t" => action = unsafe { *actstr },
            _ => {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let actstr = unsafe { c_str(actstr) };
                semsg!("E962: Invalid action: '{actstr}'");
                return;
            }
        }
    }
    if unsafe { set_tagstack(wp, d, action as c_int) }.is_ok() {
        result.write_number(0);
    }
}

/// `tagfiles()` — the tags files that would be searched, in order.
pub fn f_tagfiles(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `result` is the cleared return value; each name the walk
    // answers is NUL-terminated and lives until the next round.
    let out = tv_list_alloc_ret(result, kListLenUnknown as isize);
    let mut files = TagFiles::new();
    while let Some(name) = files.next() {
        unsafe { (*out).push_string(name.as_ptr(), -1) };
    }
}

/// `taglist({expr} [, {filename}])`.
pub fn f_taglist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    // SAFETY throughout: the arguments and `result` are live typvals; both strings are
    // NUL-terminated and outlive the search.
    let pattern = arg_string(&mut numbuf, &args[0]);
    // An empty pattern answers 0 — a Number, not an empty List.
    result.write_number(0);
    if unsafe { *pattern } == NUL as c_char {
        return;
    }
    let fname = if args.len() > 1 {
        arg_string(&mut numbuf2, &args[1])
    } else {
        ptr::null()
    };
    let list = list_alloc_ret(result, kListLenUnknown as isize);
    let (pat, file) = (pattern as *mut c_char, fname as *mut c_char);
    let _ = unsafe { get_tags(list, pat, file) };
}
