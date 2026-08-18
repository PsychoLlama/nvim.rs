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
use crate::pos::MAXCOL;
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
    unsafe {
        let start_col = (*startpos).col;
        // The delimiter runs from just after the quote to the '('.
        let mut p = linep.offset(start_col as isize + 1);
        while *p as c_int != NUL && *p as c_int != '(' as c_int {
            p = p.offset(1);
        }
        let delim_len = (p.offset_from(linep) - start_col as isize - 1) as size_t;
        let delim = xmemdupz(
            linep.offset(start_col as isize + 1) as *const c_void,
            delim_len,
        ) as *mut c_char;

        let mut found = false;
        let mut lnum = (*startpos).lnum;
        while lnum <= (*endpos).lnum && !found {
            let line = ml_get(lnum);
            let from = if lnum == (*startpos).lnum {
                start_col + 1
            } else {
                0
            };
            let mut p = line.offset(from as isize);
            while *p as c_int != NUL {
                if lnum == (*endpos).lnum && p.offset_from(line) as colnr_T >= (*endpos).col {
                    break;
                }
                if *p as c_int == ')' as c_int
                    && strncmp(delim, p.offset(1), delim_len) == 0
                    && *p.offset(delim_len as isize + 1) as c_int == '"' as c_int
                {
                    found = true;
                    break;
                }
                p = p.offset(1);
            }
            lnum += 1;
        }
        xfree(delim as *mut c_void);
        found
    }
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
    unsafe {
        let mut p = line; // scan from the start
        if (*curbuf.get()).b_p_lisp != 0 {
            // Skip Lispish one-line comments.
            if vim_strchr(p, ';' as c_int).is_null() {
                return MAXCOL; // there are no comments
            }
            let mut in_str = false; // inside of a string
            loop {
                p = strpbrk(p, c"\";".as_ptr());
                if p.is_null() {
                    return MAXCOL;
                }
                if *p as c_int == '"' as c_int {
                    if in_str {
                        if *p.offset(-1) as c_int != '\\' as c_int {
                            in_str = false; // skip an escaped quote
                        }
                    } else if p == line
                        || (p.offset_from(line) >= 2
                            // skip the #\" form
                            && *p.offset(-1) as c_int != '\\' as c_int
                            && *p.offset(-2) as c_int != '#' as c_int)
                    {
                        in_str = true;
                    }
                } else if !in_str
                    && (p.offset_from(line) < 2
                        || (*p.offset(-1) as c_int != '\\' as c_int
                            && *p.offset(-2) as c_int != '#' as c_int))
                    && !is_pos_in_string(line, p.offset_from(line) as colnr_T)
                {
                    break; // found!
                }
                p = p.offset(1);
            }
        } else {
            loop {
                p = vim_strchr(p, '/' as c_int);
                if p.is_null() {
                    return MAXCOL;
                }
                // Accept a double '/', unless it is preceded by '*' and
                // followed by '*', because "*//*" ends one comment and
                // starts the next. Only accept the position when it is
                // not inside a string.
                if *p.offset(1) as c_int == '/' as c_int
                    && (p == line
                        || *p.offset(-1) as c_int != '*' as c_int
                        || *p.offset(2) as c_int != '*' as c_int)
                    && !is_pos_in_string(line, p.offset_from(line) as colnr_T)
                {
                    break;
                }
                p = p.offset(1);
            }
        }
        p.offset_from(line) as c_int
    }
}

/// Whether line `lnum` is empty or holds nothing but white space.
///
/// # Safety
/// `lnum` must be a line of the current buffer.
pub unsafe fn linewhite(lnum: linenr_T) -> bool {
    unsafe { *skipwhite(ml_get(lnum)) as c_int == NUL }
}
