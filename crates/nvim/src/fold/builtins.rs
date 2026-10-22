//! The Vimscript fold builtins.
//!
//! `foldclosed()`, `foldclosedend()` and `foldlevel()` all read the tree
//! *without* the display cache — they pass `cache = false` to
//! [`has_folding_win`] — which is what makes them a usable oracle for the
//! fold tree in a headless editor.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::charset::skip;
use crate::cstr;
use crate::cstr::byte_at;
use crate::decoration::virt_text_into_string;
use crate::eval::typval::tv_get_lnum;
use crate::eval::vars::{get_vim_var_nr, vim_var_string};
use crate::global_cell::GlobalCell;
use crate::memline::Lines;
use crate::memory::{ThinCString, XString};
use crate::os::cshim::ngettext;
use crate::search::linewhite;
use crate::strings::{FmtArg, formatted};
use crate::winlayer::Buf;
use core::ffi::c_ulong;

use super::text::*;
use super::*;
use crate::types::Vv;

/// "foldclosed()" and "foldclosedend()" functions
pub(super) fn foldclosed_both(args: &[TypVal], result: &mut TypVal, end: bool) {
    let (rv, lnum) = (result, tv_get_lnum(&args[0]));
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
    let (rv, lnum) = (result, tv_get_lnum(&args[0]));
    if lnum >= 1 && lnum <= Buf::current().b_ml.ml_line_count {
        rv.write_number(fold_level(lnum) as VarNumber);
    }
}

/// "foldtext()" function
pub fn f_foldtext(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let rv = result;
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
    // The prefix, then the title, then the cleanup.
    let args = [FmtArg::from(dashes.as_bytes()), FmtArg::from(count)];
    let mut text = formatted(usize::MAX, txt.to_bytes(), &args);
    let prefix_len = text.len();
    text.extend_from_slice(title);
    foldtext_cleanup(&mut text, prefix_len);
    // Read as the C string upstream built: up to the first NUL.
    let end = text.iter().position(|&b| b == 0).unwrap_or(text.len());
    rv.write_string(Some(ThinCString::from_bytes(&text[..end])));
}

/// "foldtextresult(lnum)" function
pub fn f_foldtextresult(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut buf = [0u8; FOLD_TEXT_LEN as usize];
    // 'foldtext' can call `foldtextresult()` again; one level is enough.
    static entered: GlobalCell<bool> = GlobalCell::new(false);
    let rv = result;
    rv.write_string(None);
    if entered.get() {
        return;
    }
    entered.set(true);
    let lnum = tv_get_lnum(&args[0]).max(0);
    let win = Win::current();
    let info = fold_info(win, lnum);
    if info.fi_lines > 0 {
        let last = lnum + info.fi_lines - 1;
        let text = match get_foldtext(win, lnum, last, info, &mut buf) {
            // The scratch buffer, which stays this frame's: copy it out.
            FoldText::Scratch => XString::from_bytes(cstr::in_bytes(&buf).to_bytes()),
            FoldText::Owned(text) => text,
            // A virtual-text 'foldtext' answers in chunks; flatten them.
            FoldText::Chunks(vt) => virt_text_into_string(vt),
        };
        rv.write_string(Some(text.into()));
    }
    entered.set(false);
}
