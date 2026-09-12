//! The Vimscript fold builtins.
//!
//! `foldclosed()`, `foldclosedend()` and `foldlevel()` all read the tree
//! *without* the display cache — they pass `cache = false` to
//! [`has_folding_win`] — which is what makes them a usable oracle for the
//! fold tree in a headless editor.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::charset::skip;
use crate::cstr;
use crate::cstr::byte_at;
use crate::decoration::{clear_virttext, next_virt_text_chunk};
use crate::eval::typval::tv_get_lnum;
use crate::eval::vars::{get_vim_var_nr, get_vim_var_str};
use crate::global_cell::GlobalCell;
use crate::memline::Lines;
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::os::cshim::{ngettext, snprintf};
use crate::search::linewhite;
use crate::strings::concat_str;
use crate::winlayer::{Buf, Live};
use ::libc::strcat;
use core::ffi::{c_char, c_int, c_ulong, c_void};
use core::ptr;

use super::text::*;
use super::*;
use crate::types::Vv;

/// "foldclosed()" and "foldclosedend()" functions
pub(super) fn foldclosed_both(args: &[TypVal], result: &mut TypVal, end: bool) {
    // SAFETY: the caller's promise -- live typvals.
    let (mut rv, lnum) = unsafe { (Tv::new(result), tv_get_lnum(&args[0])) };
    if lnum >= 1 && lnum <= Buf::current().b_ml.ml_line_count {
        let mut first: LineNr = 0;
        let mut last: LineNr = 0;
        let win = Win::current();
        let closed = has_folding_win(win, lnum, Some(&mut first), Some(&mut last), false, None);
        if closed {
            rv.write_number((if end { last } else { first }) as VarNumber);
            return;
        }
    }
    rv.write_number(-1 as VarNumber);
}

/// "foldclosed()" function
pub fn f_foldclosed(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    foldclosed_both(args, result, false);
}

/// "foldclosedend()" function
pub fn f_foldclosedend(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    foldclosed_both(args, result, true);
}

/// "foldlevel()" function
pub fn f_foldlevel(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's promise -- live typvals.
    let (mut rv, lnum) = unsafe { (Tv::new(result), tv_get_lnum(&args[0])) };
    if lnum >= 1 && lnum <= Buf::current().b_ml.ml_line_count {
        rv.write_number(fold_level(lnum) as VarNumber);
    }
}

/// "foldtext()" function
pub fn f_foldtext(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's promise -- a live typval.
    let mut rv = unsafe { Tv::new(result) };
    rv.write_string(ptr::null_mut());
    // SAFETY: reading three `v:` variables the fold drawing has just set.
    let (start, end, dash) = (Vv::Foldstart, Vv::Foldend, Vv::Folddashes);
    let (foldstart, foldend, dashes) = (
        get_vim_var_nr(start),
        get_vim_var_nr(end),
        get_vim_var_str(dash),
    );
    let (foldstart, foldend) = (foldstart as LineNr, foldend as LineNr);
    if !(foldstart > 0 && foldend <= Buf::current().b_ml.ml_line_count) {
        return;
    }
    // The first line of the fold that has anything on it.
    let mut lnum = foldstart;
    while lnum < foldend && linewhite(lnum) {
        lnum += 1;
    }
    // Which line the fold's title is taken from, and where in it it starts.
    let mut lines = Lines::current();
    let mut which = lnum;
    let mut at = {
        let text = lines.line(lnum);
        let mut at = skip::white(text);
        // A comment opener is skipped, and an empty one takes the next line.
        if byte_at(text, at) == b'/'
            && (byte_at(text, at + 1) == b'*' || byte_at(text, at + 1) == b'/')
        {
            at = (at + 2).min(text.len());
            at += skip::white(&text[at..]);
            if byte_at(text, at) == 0 && (lnum + 1) < foldend {
                which = lnum + 1;
            }
        }
        at
    };
    if which != lnum {
        let text = lines.line(which);
        at = skip::white(text);
        if byte_at(text, at) == b'*' {
            at += 1;
            at += skip::white(&text[at..]);
        }
    }
    let title = &lines.line(which)[at..];

    let count = foldend - foldstart + 1;
    // SAFETY: three static format strings, and the NUL-terminated strings
    // `dashes` and `s` -- `s` is the tail of a buffer line, so the line's own
    // terminator ends it; `r` is an allocation big enough for all of them.
    let s = title.as_ptr().cast::<c_char>();
    let one = c"+-%s%3d line: ";
    let many = c"+-%s%3d lines: ";
    let txt = ngettext(one, many, count as c_ulong);
    let mut len = txt.count_bytes() + unsafe { cstr::bytes_at(dashes) }.len() + 20 + title.len();
    let r = unsafe { xmalloc(len) } as *mut c_char;
    unsafe { snprintf(r, len, txt.as_ptr(), dashes, count) };
    len = unsafe { cstr::bytes_at(r) }.len();
    unsafe { strcat(r, s) };
    unsafe { foldtext_cleanup(r.add(len)) };
    rv.write_string(r);
}

/// "foldtextresult(lnum)" function
pub fn f_foldtextresult(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut buf: [c_char; FOLD_TEXT_LEN as usize] = [0; FOLD_TEXT_LEN as usize];
    // 'foldtext' can call `foldtextresult()` again; one level is enough.
    static entered: GlobalCell<bool> = GlobalCell::new(false);
    // SAFETY: the caller's promise -- a live typval.
    let mut rv = unsafe { Tv::new(result) };
    rv.write_string(ptr::null_mut());
    if entered.get() {
        return;
    }
    entered.set(true);
    let lnum = tv_get_lnum(&args[0]).max(0);
    let win = Win::current();
    let info = fold_info(win, lnum);
    if info.fi_lines > 0 {
        let mut vt: VirtText = VIRTTEXT_EMPTY;
        let (last, out) = (lnum + info.fi_lines - 1, buf.as_mut_ptr());
        // SAFETY: `buf` holds `FOLD_TEXT_LEN` bytes and `vt` is this frame's.
        let mut text = unsafe { get_foldtext(win, lnum, last, info, out, &raw mut vt) };
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
        rv.write_string(text);
    }
    entered.set(false);
}

/// [`Live`]'s shape for the `TypVal` the Vimscript face answers in.
type Tv = Live<TypVal>;
