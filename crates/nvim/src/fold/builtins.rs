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
use crate::main::{curbuf, curwin};
use crate::memline::ml_get;
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::os::cshim::{ngettext, snprintf};
use crate::search::linewhite;
use crate::strings::concat_str;
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
    // SAFETY: the caller's promise, plus a live current window and buffer.
    unsafe {
        let lnum = tv_get_lnum(argvars);
        if lnum >= 1 && lnum <= (*curbuf.get()).b_ml.ml_line_count {
            let mut first: linenr_T = 0;
            let mut last: linenr_T = 0;
            if has_folding_win(
                curwin.get(),
                lnum,
                &raw mut first,
                &raw mut last,
                false,
                ptr::null_mut(),
            ) {
                (*rettv).vval.v_number = (if end { last } else { first }) as varnumber_T;
                return;
            }
        }
        (*rettv).vval.v_number = -1 as varnumber_T;
    }
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
    // SAFETY: the caller's promise, plus a live current buffer.
    unsafe {
        let lnum = tv_get_lnum(argvars);
        if lnum >= 1 && lnum <= (*curbuf.get()).b_ml.ml_line_count {
            (*rettv).vval.v_number = fold_level(lnum) as varnumber_T;
        }
    }
}

/// "foldtext()" function
///
/// # Safety
/// `rettv` must be a live typval.
pub unsafe fn f_foldtext(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's promise, plus a live current buffer.
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ptr::null_mut();
        let foldstart = get_vim_var_nr(Vv::Foldstart) as linenr_T;
        let foldend = get_vim_var_nr(Vv::Foldend) as linenr_T;
        let dashes = get_vim_var_str(Vv::Folddashes);
        if !(foldstart > 0 && foldend <= (*curbuf.get()).b_ml.ml_line_count) {
            return;
        }
        // The first line of the fold that has anything on it.
        let mut lnum = foldstart;
        while lnum < foldend && linewhite(lnum) {
            lnum += 1;
        }
        let mut s = skipwhite(ml_get(lnum));
        // A comment opener is skipped, and an empty one takes the next line.
        if *s.offset(0) as c_int == '/' as c_int
            && (*s.offset(1) as c_int == '*' as c_int || *s.offset(1) as c_int == '/' as c_int)
        {
            s = skipwhite(s.offset(2));
            if *skipwhite(s) as c_int == NUL && (lnum + 1) < foldend {
                s = skipwhite(ml_get(lnum + 1));
                if *s as c_int == '*' as c_int {
                    s = skipwhite(s.offset(1));
                }
            }
        }
        let count = foldend - foldstart + 1;
        let txt = ngettext(
            c"+-%s%3d line: ".as_ptr(),
            c"+-%s%3d lines: ".as_ptr(),
            count as c_ulong,
        );
        let mut len = strlen(txt)
            .wrapping_add(strlen(dashes))
            .wrapping_add(20)
            .wrapping_add(strlen(s));
        let r = xmalloc(len) as *mut c_char;
        snprintf(r, len, txt, dashes, count);
        len = strlen(r);
        strcat(r, s);
        foldtext_cleanup(r.add(len));
        (*rettv).vval.v_string = r;
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
    // SAFETY: the caller's promise, plus a live current window.
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ptr::null_mut();
        if entered.get() {
            return;
        }
        entered.set(true);
        let lnum = tv_get_lnum(argvars).max(0);
        let info = fold_info(curwin.get(), lnum);
        if info.fi_lines > 0 {
            let mut vt: VirtText = VIRTTEXT_EMPTY;
            let mut text = get_foldtext(
                curwin.get(),
                lnum,
                lnum + info.fi_lines - 1,
                info,
                &raw mut buf as *mut c_char,
                &raw mut vt,
            );
            if text == &raw mut buf as *mut c_char {
                text = xstrdup(text);
            }
            if vt.size > 0 {
                debug_assert!(*text as c_int == '\0' as c_int, "*text == NUL");
                // A virtual-text 'foldtext' answers in chunks; flatten them.
                let mut i: size_t = 0;
                while i < vt.size {
                    let mut attr: c_int = 0;
                    let chunk = next_virt_text_chunk(vt, &raw mut i, &raw mut attr);
                    if chunk.is_null() {
                        break;
                    }
                    let joined = concat_str(text, chunk);
                    xfree(text as *mut c_void);
                    text = joined;
                }
            }
            clear_virttext(&raw mut vt);
            (*rettv).vval.v_string = text;
        }
        entered.set(false);
    }
}
