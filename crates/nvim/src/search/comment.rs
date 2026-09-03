//! The text a bracket may be hiding in.
//!
//! [`check_linecomment`] answers where a `//` (or Lisp `;`) comment
//! starts on a line, and [`find_rawstring_end`] whether an `R"delim(`
//! really opens a raw string. Both exist so that
//! [`findmatchlimit`](super::findmatchlimit) can tell a bracket that
//! counts from one that does not; `check_linecomment` is also what the
//! formatting and C-indent code ask.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::charset::skip;
use crate::cstr;
use crate::memline::Lines;
use crate::pos::MAXCOL;
use crate::types::NUL;
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int, c_void};

/// Whether a raw string starting at `linep[startpos.col - 1]` ends
/// between `startpos` and `endpos`.
///
/// # Safety
/// `linep` must be the line `startpos` is on; `startpos` and `endpos`
/// must be positions in the current buffer.
pub(crate) unsafe fn find_rawstring_end(
    linep: *mut c_char,
    startpos: *mut pos_T,
    endpos: *mut pos_T,
) -> bool {
    let start_col = unsafe { (*startpos).col };
    // The delimiter runs from just after the quote to the '('.
    let mut p = unsafe { linep.offset(start_col as isize + 1) };
    while unsafe { *p } as c_int != NUL && unsafe { *p } as c_int != '(' as c_int {
        p = unsafe { p.offset(1) };
    }
    let delim_len = (unsafe { p.offset_from(linep) } - start_col as isize - 1) as size_t;
    // SAFETY: `delim_len` bytes starting one past `start_col` are inside the
    // line the caller found the `(` on.
    let delim = unsafe {
        let from = linep.offset(start_col as isize + 1) as *const c_void;
        xmemdupz(from, delim_len)
    } as *mut c_char;

    let mut found = false;
    let mut lnum = unsafe { (*startpos).lnum };
    while lnum <= unsafe { (*endpos).lnum } && !found {
        let line = ml_get(lnum);
        let from = if lnum == unsafe { (*startpos).lnum } {
            start_col + 1
        } else {
            0
        };
        let mut p = unsafe { line.offset(from as isize) };
        while unsafe { *p } as c_int != NUL {
            if lnum == unsafe { (*endpos).lnum }
                && unsafe { p.offset_from(line) } as colnr_T >= unsafe { (*endpos).col }
            {
                break;
            }
            if unsafe { *p } as c_int == ')' as c_int
                && unsafe { cstr::prefix_eq(delim, p.offset(1), delim_len) }
                && unsafe { *p.offset(delim_len as isize + 1) } as c_int == '"' as c_int
            {
                found = true;
                break;
            }
            p = unsafe { p.offset(1) };
        }
        lnum += 1;
    }
    unsafe { xfree(delim as *mut c_void) };
    found
}

// ---------------------------------------------------------------------
// Line-level helpers, shared with the indent and formatting code.
// ---------------------------------------------------------------------

/// The column a `//` comment starts at on `line`, or `MAXCOL`.
///
/// With `'lisp'` set the comment character is `;` instead, and neither a
/// `#\;` nor a `;` inside a string counts.
///
/// # Safety
/// `line` must be NUL-terminated.
pub unsafe fn check_linecomment(line: *const c_char) -> c_int {
    let mut p = line; // scan from the start
    if cur_buf().b_p_lisp != 0 {
        // Skip Lispish one-line comments.
        if unsafe { vim_strchr(p, ';' as c_int) }.is_null() {
            return MAXCOL; // there are no comments
        }
        let mut in_str = false; // inside of a string
        loop {
            p = unsafe { strpbrk(p, c"\";".as_ptr()) };
            if p.is_null() {
                return MAXCOL;
            }
            if unsafe { *p } as c_int == '"' as c_int {
                if in_str {
                    if unsafe { *p.offset(-1) } as c_int != '\\' as c_int {
                        in_str = false; // skip an escaped quote
                    }
                } else if p == line
                    || (unsafe { p.offset_from(line) } >= 2
                        // skip the #\" form
                        && unsafe { *p.offset(-1) } as c_int != '\\' as c_int
                        && unsafe { *p.offset(-2) } as c_int != '#' as c_int)
                {
                    in_str = true;
                }
            } else if !in_str
                && (unsafe { p.offset_from(line) } < 2
                    || (unsafe { *p.offset(-1) } as c_int != '\\' as c_int
                        && unsafe { *p.offset(-2) } as c_int != '#' as c_int))
                && !unsafe { is_pos_in_string(line, p.offset_from(line) as colnr_T) }
            {
                break; // found!
            }
            p = unsafe { p.offset(1) };
        }
    } else {
        loop {
            p = unsafe { vim_strchr(p, '/' as c_int) };
            if p.is_null() {
                return MAXCOL;
            }
            // Accept a double '/', unless it is preceded by '*' and
            // followed by '*', because "*//*" ends one comment and
            // starts the next. Only accept the position when it is
            // not inside a string.
            if unsafe { *p.offset(1) } as c_int == '/' as c_int
                && (p == line
                    || unsafe { *p.offset(-1) } as c_int != '*' as c_int
                    || unsafe { *p.offset(2) } as c_int != '*' as c_int)
                && !unsafe { is_pos_in_string(line, p.offset_from(line) as colnr_T) }
            {
                break;
            }
            p = unsafe { p.offset(1) };
        }
    }
    unsafe { p.offset_from(line) as c_int }
}

/// Whether line `lnum` is empty or holds nothing but white space.
///
/// # Safety
/// `lnum` must be a line of the current buffer.
pub unsafe fn linewhite(lnum: linenr_T) -> bool {
    // SAFETY: the slice is read and dropped before anything else runs, so
    // nothing can swap the line out from under it.
    let mut lines = unsafe { Lines::current() };
    let line = lines.line(lnum);
    skip::white(line) == line.len()
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
