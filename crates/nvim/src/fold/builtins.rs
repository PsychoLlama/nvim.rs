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
use crate::eval::vars::{get_vim_var_nr, vim_var_string};
use crate::global_cell::GlobalCell;
use crate::memline::Lines;
use crate::memory::{ThinCString, XString};
use crate::os::cshim::ngettext;
use crate::search::linewhite;
use crate::snprintf;
use crate::winlayer::{Buf, Live};
use core::ffi::{c_char, c_int, c_ulong};

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
    rv.write_string(None);
    // The three `v:` variables the fold drawing has just set.
    let (start, end, dash) = (Vv::Foldstart, Vv::Foldend, Vv::Folddashes);
    let (foldstart, foldend, dashes) = (
        get_vim_var_nr(start),
        get_vim_var_nr(end),
        vim_var_string(dash).unwrap_or_else(ThinCString::empty),
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
    let one = c"+-%s%3d line: ";
    let many = c"+-%s%3d lines: ";
    let txt = ngettext(one, many, count as c_ulong);
    let dashes_len = dashes.as_bytes().len();
    // The prefix, then the title, then the cleanup, all in one buffer with
    // room for the widest count.
    let mut text = vec![0u8; txt.count_bytes() + dashes_len + 20 + title.len() + 1];
    // SAFETY: a static format string, the NUL-terminated `dashes`, and a
    // buffer big enough for both and the count.
    unsafe {
        snprintf!(
            text.as_mut_ptr().cast::<c_char>(),
            text.len(),
            txt.as_ptr(),
            dashes.as_ptr(),
            count
        )
    };
    let prefix_len = cstr::in_bytes(&text).count_bytes();
    text.truncate(prefix_len);
    text.extend_from_slice(title);
    text.push(0);
    // SAFETY: `text` is NUL-terminated and writable, and `prefix_len` is
    // inside it; the current window is live.
    unsafe { foldtext_cleanup(text.as_mut_ptr().cast::<c_char>().add(prefix_len)) };
    rv.write_string(Some(ThinCString::from_cstr(cstr::in_bytes(&text))));
}

/// "foldtextresult(lnum)" function
pub fn f_foldtextresult(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut buf: [c_char; FOLD_TEXT_LEN as usize] = [0; FOLD_TEXT_LEN as usize];
    // 'foldtext' can call `foldtextresult()` again; one level is enough.
    static entered: GlobalCell<bool> = GlobalCell::new(false);
    // SAFETY: the caller's promise -- a live typval.
    let mut rv = unsafe { Tv::new(result) };
    rv.write_string(None);
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
        let answer = unsafe { get_foldtext(win, lnum, last, info, out, &raw mut vt) };
        let mut text = if answer == &raw mut buf as *mut c_char {
            // The scratch buffer, which stays this frame's: copy it out.
            // SAFETY: `get_foldtext` left a NUL-terminated string in it.
            XString::from_bytes(unsafe { cstr::bytes_at(answer) })
        } else {
            // SAFETY: anything else is `get_foldtext`'s own allocation,
            // which it hands to its caller.
            unsafe { XString::from_raw(answer) }
        };
        if vt.size > 0 {
            debug_assert!(
                text.is_empty(),
                "a virtual-text 'foldtext' answers no bytes"
            );
            // A virtual-text 'foldtext' answers in chunks; flatten them.
            let mut i: size_t = 0;
            while i < vt.size {
                let mut attr: c_int = 0;
                let chunk = unsafe { next_virt_text_chunk(vt, &raw mut i, &raw mut attr) };
                if chunk.is_null() {
                    break;
                }
                // SAFETY: a chunk's NUL-terminated text.
                text.push_bytes(unsafe { cstr::bytes_at(chunk) });
            }
        }
        // SAFETY: `vt` is this frame's virtual text.
        unsafe { clear_virttext(&raw mut vt) };
        rv.write_string(Some(text.into()));
    }
    entered.set(false);
}

/// [`Live`]'s shape for the `TypVal` the Vimscript face answers in.
type Tv = Live<TypVal>;
