//! Marks, jumps, changes and tags.
#![forbid(unsafe_code)]

use super::tv_get_buf;
use super::wrappers::{arg_number, dict_alloc_ret};
use crate::buffer::WinInfos;
use crate::eval::typval::{
    NumBuf, tv_check_for_dict_arg, tv_check_for_string_arg, tv_dict_alloc, tv_list_alloc,
    tv_list_alloc_ret,
};
use crate::eval::window::{find_tabwin, find_win_by_nr_or_id};
use crate::guard::Suppress;
use crate::mark::{cleanup_jumplist, get_buf_local_marks, get_global_marks};
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::startup::vim_ignored;
use crate::tag::{TagFiles, get_tags, get_tagstack, set_tagstack};
use crate::types::{
    DictRef, EvalFuncData, Pos, TypVal, VarNumber, kListLenMayKnow, kListLenUnknown,
};
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{c_char, c_int};

/// `changenr()` — the sequence number of the change the undo tree is at.
pub fn f_changenr(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(Buf::current().b_u_seq_cur as VarNumber);
}

/// One `{lnum, col, coladd}` entry, for the caller to add to and push.
fn mark_dict(mark: Pos) -> DictRef {
    let d = tv_dict_alloc();
    let _ = d.edit().add_number(b"lnum", mark.lnum as VarNumber);
    let _ = d.edit().add_number(b"col", mark.col as VarNumber);
    let _ = d.edit().add_number(b"coladd", mark.coladd as VarNumber);
    d
}

/// `getchangelist([{buf}])` — `[changes, index]`.
pub fn f_getchangelist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let out = tv_list_alloc_ret(result, 2);
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
    let Some(mut buf) = buf else {
        return;
    };
    let entries = tv_list_alloc(buf.b_changelistlen as isize);
    out.push_list(Some(entries.clone()));

    // The index is this window's if it is showing the buffer, and
    // otherwise the one remembered for this window in the buffer's
    // window-info list. A buffer this window has never shown reports
    // the end of the list.
    let index = if buf == Win::current().buffer() {
        Win::current().w_changelistidx
    } else {
        let here = Win::current_or_none().map(Win::id);
        let changelistlen = buf.b_changelistlen;
        WinInfos::of(&mut buf)
            .entries_mut()
            .iter()
            .find(|entry| entry.wi_win == here)
            .map_or(changelistlen, |entry| entry.wi_changelistidx)
    };
    out.push_number(index as VarNumber);

    for i in 0..buf.b_changelistlen {
        let mark = buf.b_changelist[i as usize].mark;
        if mark.lnum != 0 {
            entries.edit().push_dict(Some(mark_dict(mark)));
        }
    }
}

/// `getjumplist([{winnr} [, {tabnr}]])` — `[jumps, index]`.
pub fn f_getjumplist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // The jump list is compacted before it is read so no entry is stale.
    let out = tv_list_alloc_ret(result, kListLenMayKnow as isize);
    let Some(wp) = find_tabwin(args.first(), args.get(1)) else {
        return;
    };
    cleanup_jumplist(wp, true);
    let entries = tv_list_alloc(wp.w_jumplistlen as isize);
    out.push_list(Some(entries.clone()));
    out.push_number(wp.w_jumplistidx as VarNumber);
    for i in 0..wp.w_jumplistlen {
        let entry = &wp.w_jumplist[i as usize];
        if entry.fmark.mark.lnum == 0 {
            continue;
        }
        let d = mark_dict(entry.fmark.mark);
        let _ = d.edit().add_number(b"bufnr", entry.fmark.fnum as VarNumber);
        // A jump into a file that is no longer loaded keeps its name.
        if let Some(fname) = entry.file_name() {
            let _ = d.edit().add_str(b"filename", Some(fname));
        }
        entries.edit().push_dict(Some(d));
    }
}

/// `getmarklist([{buf}])` — the global marks, or one buffer's local ones.
pub fn f_getmarklist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let out = tv_list_alloc_ret(result, kListLenMayKnow as isize);
    if args.is_empty() {
        get_global_marks(out);
        return;
    }
    if let Some(buf) = tv_get_buf(&args[0], 0) {
        get_buf_local_marks(buf, out);
    }
}

/// `gettagstack([{winnr}])`.
pub fn f_gettagstack(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // The dict is allocated before the window is resolved, so a bad window
    // still yields an empty dict rather than nothing.
    dict_alloc_ret(result);
    let found = if args.is_empty() {
        Win::current_or_none()
    } else {
        find_win_by_nr_or_id(&args[0])
    };
    let Some(wp) = found else {
        return;
    };
    get_tagstack(wp, result.dict_mut().expect("the dict just stored"));
}

/// `settagstack({winnr}, {dict} [, {action}])`.
pub fn f_settagstack(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(-1);
    let found = find_win_by_nr_or_id(&args[0]);
    let Some(wp) = found.filter(|_| tv_check_for_dict_arg(args, 1).is_ok()) else {
        return;
    };
    let Some(d) = args[1].dict_ref() else {
        return;
    };
    // "r" replaces, "a" appends, "t" truncates; anything else, including
    // a longer string starting with one of them, is E962.
    let mut action = b'r' as c_char;
    if args.len() > 2 {
        if tv_check_for_string_arg(args, 2).is_err() {
            return;
        }
        let Some(actstr) = numbuf.bytes_chk(&args[2]) else {
            return;
        };
        match actstr {
            &[letter @ (b'r' | b'a' | b't')] => action = letter as c_char,
            _ => {
                let actstr = msg_bytes(actstr);
                semsg!("E962: Invalid action: '{actstr}'");
                return;
            }
        }
    }
    if set_tagstack(wp, d, c_int::from(action)).is_ok() {
        result.write_number(0);
    }
}

/// `tagfiles()` — the tags files that would be searched, in order.
pub fn f_tagfiles(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let out = tv_list_alloc_ret(result, kListLenUnknown as isize);
    let mut files = TagFiles::new();
    while let Some(name) = files.next() {
        out.push_str(Some(name.as_cstr()));
    }
}

/// `taglist({expr} [, {filename}])`.
pub fn f_taglist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    // Both strings are the arguments' own, which 'tagfunc' cannot free.
    let pattern = numbuf.string(&args[0]);
    // An empty pattern answers 0 — a Number, not an empty List.
    result.write_number(0);
    if pattern.is_empty() {
        return;
    }
    let fname = args.get(1).map(|fname| numbuf2.string(fname));
    let list = tv_list_alloc_ret(result, kListLenUnknown as isize);
    let _ = get_tags(list, pattern, fname);
}
