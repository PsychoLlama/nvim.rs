//! The Vimscript fold builtins.
//!
//! `foldclosed()`, `foldclosedend()` and `foldlevel()` all read the tree
//! *without* the display cache — they pass `cache = false` to
//! [`has_folding_win`] — which is what makes them a usable oracle for the
//! fold tree in a headless editor.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::charset::skipwhite;
use crate::decoration::{clear_virttext, next_virt_text_chunk};
use crate::eval::typval::tv_get_lnum;
use crate::eval::vars::{get_vim_var_nr, get_vim_var_str};
use crate::global_cell::GlobalCell;
use crate::main::curwin;
use crate::memline::ml_get;
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::os::cshim::{ngettext, snprintf};
use crate::search::linewhite;
use crate::strings::concat_str;
use crate::winlayer::{Buf, Live};
use ::libc::{strcat, strlen};
use core::ffi::{c_char, c_int, c_ulong, c_void};
use core::ptr;

use super::text::*;
use super::*;
use crate::types::{VAR_STRING, Vv};

/// "foldclosed()" and "foldclosedend()" functions
///
/// # Safety
/// `argvars` and `rettv` must be live typvals.
pub(super) unsafe fn foldclosed_both(argvars: *mut typval_T, rettv: *mut typval_T, end: bool) {
    // SAFETY: the caller's promise -- live typvals.
    let (mut rv, lnum) = unsafe { (Tv::new(rettv), tv_get_lnum(argvars)) };
    if lnum >= 1 && lnum <= cur_buf().b_ml.ml_line_count {
        let mut first: linenr_T = 0;
        let mut last: linenr_T = 0;
        let (fp, lp) = (&raw mut first, &raw mut last);
        // SAFETY: `curwin` is live and both out-parameters are this frame's.
        let closed = unsafe { has_folding_win(curwin.get(), lnum, fp, lp, false, ptr::null_mut()) };
        if closed {
            rv.vval.v_number = (if end { last } else { first }) as varnumber_T;
            return;
        }
    }
    rv.vval.v_number = -1 as varnumber_T;
}

/// "foldclosed()" function
///
/// # Safety
/// `argvars` and `rettv` must be live typvals.
pub unsafe fn f_foldclosed(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's promise.
    unsafe { foldclosed_both(argvars, rettv, false) };
}

/// "foldclosedend()" function
///
/// # Safety
/// `argvars` and `rettv` must be live typvals.
pub unsafe fn f_foldclosedend(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's promise.
    unsafe { foldclosed_both(argvars, rettv, true) };
}

/// "foldlevel()" function
///
/// # Safety
/// `argvars` and `rettv` must be live typvals.
pub unsafe fn f_foldlevel(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's promise -- live typvals.
    let (mut rv, lnum) = unsafe { (Tv::new(rettv), tv_get_lnum(argvars)) };
    if lnum >= 1 && lnum <= cur_buf().b_ml.ml_line_count {
        // SAFETY: `lnum` is inside the current buffer.
        rv.vval.v_number = unsafe { fold_level(lnum) } as varnumber_T;
    }
}

/// "foldtext()" function
///
/// # Safety
/// `rettv` must be a live typval.
pub unsafe fn f_foldtext(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's promise -- a live typval.
    let mut rv = unsafe { Tv::new(rettv) };
    rv.v_type = VAR_STRING;
    rv.vval.v_string = ptr::null_mut();
    // SAFETY: reading three `v:` variables the fold drawing has just set.
    let (start, end, dash) = (Vv::Foldstart, Vv::Foldend, Vv::Folddashes);
    let (foldstart, foldend, dashes) = unsafe {
        (
            get_vim_var_nr(start),
            get_vim_var_nr(end),
            get_vim_var_str(dash),
        )
    };
    let (foldstart, foldend) = (foldstart as linenr_T, foldend as linenr_T);
    if !(foldstart > 0 && foldend <= cur_buf().b_ml.ml_line_count) {
        return;
    }
    // The first line of the fold that has anything on it.
    let mut lnum = foldstart;
    // SAFETY: `lnum` is inside the buffer, `foldend` having been checked.
    while lnum < foldend && unsafe { linewhite(lnum) } {
        lnum += 1;
    }
    // Both are NUL-terminated: a buffer line, and 'folddashes'.
    // SAFETY: `p` is inside one of them, at or before its terminator.
    let at = |p: *const c_char| unsafe { *p } as c_int;
    // SAFETY: as `at`.
    let skip_ws = |p: *mut c_char| unsafe { skipwhite(p) };
    // SAFETY: `lnum` is inside the buffer.
    let line = |n: linenr_T| ml_get(n);

    let mut s = skip_ws(line(lnum));
    // A comment opener is skipped, and an empty one takes the next line.
    if at(s) == '/' as c_int
        && (at(s.wrapping_offset(1)) == '*' as c_int || at(s.wrapping_offset(1)) == '/' as c_int)
    {
        s = skip_ws(s.wrapping_offset(2));
        if at(skip_ws(s)) == NUL && (lnum + 1) < foldend {
            s = skip_ws(line(lnum + 1));
            if at(s) == '*' as c_int {
                s = skip_ws(s.wrapping_offset(1));
            }
        }
    }
    let count = foldend - foldstart + 1;
    // SAFETY: three static format strings, and the NUL-terminated strings
    // `dashes` and `s`; `r` is an allocation big enough for all of them.
    let one = c"+-%s%3d line: ".as_ptr();
    let many = c"+-%s%3d lines: ".as_ptr();
    unsafe {
        let txt = ngettext(one, many, count as c_ulong);
        let mut len = strlen(txt) + strlen(dashes) + 20 + strlen(s);
        let r = xmalloc(len) as *mut c_char;
        snprintf(r, len, txt, dashes, count);
        len = strlen(r);
        strcat(r, s);
        foldtext_cleanup(r.add(len));
        rv.vval.v_string = r;
    }
}

/// "foldtextresult(lnum)" function
///
/// # Safety
/// `argvars` and `rettv` must be live typvals.
pub unsafe fn f_foldtextresult(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut buf: [c_char; FOLD_TEXT_LEN as usize] = [0; FOLD_TEXT_LEN as usize];
    // 'foldtext' can call `foldtextresult()` again; one level is enough.
    static entered: GlobalCell<bool> = GlobalCell::new(false);
    // SAFETY: the caller's promise -- a live typval.
    let mut rv = unsafe { Tv::new(rettv) };
    rv.v_type = VAR_STRING;
    rv.vval.v_string = ptr::null_mut();
    if entered.get() {
        return;
    }
    entered.set(true);
    // SAFETY: the caller's promise, plus a live current window.
    let lnum = unsafe { tv_get_lnum(argvars) }.max(0);
    // SAFETY: `curwin` is a live window.
    let info = unsafe { fold_info(curwin.get(), lnum) };
    if info.fi_lines > 0 {
        let mut vt: VirtText = VIRTTEXT_EMPTY;
        let (last, out) = (lnum + info.fi_lines - 1, buf.as_mut_ptr());
        // SAFETY: `buf` holds `FOLD_TEXT_LEN` bytes and `vt` is this frame's.
        let mut text = unsafe { get_foldtext(curwin.get(), lnum, last, info, out, &raw mut vt) };
        if text == &raw mut buf as *mut c_char {
            text = unsafe { xstrdup(text) };
        }
        if vt.size > 0 {
            debug_assert!(unsafe { *text } as c_int == '\0' as c_int, "*text == NUL");
            // A virtual-text 'foldtext' answers in chunks; flatten them.
            let mut i: size_t = 0;
            while i < vt.size {
                let mut attr: c_int = 0;
                let chunk = unsafe { next_virt_text_chunk(vt, &raw mut i, &raw mut attr) };
                if chunk.is_null() {
                    break;
                }
                let joined = unsafe { concat_str(text, chunk) };
                unsafe { xfree(text as *mut c_void) };
                text = joined;
            }
        }
        // SAFETY: `vt` is this frame's virtual text.
        unsafe { clear_virttext(&raw mut vt) };
        rv.vval.v_string = text;
    }
    entered.set(false);
}

/// [`Live`]'s shape for the `typval_T` the Vimscript face answers in.
type Tv = Live<typval_T>;

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
