//! The text a bracket may be hiding in.
//!
//! [`check_linecomment`] answers where a `//` (or Lisp `;`) comment
//! starts on a line, and [`find_rawstring_end`] whether an `R"delim(`
//! really opens a raw string. Both exist so that
//! [`findmatchlimit`](super::findmatchlimit) can tell a bracket that
//! counts from one that does not; `check_linecomment` is also what the
//! formatting and C-indent code ask.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::charset::skip;
use crate::cstr::byte_at;
use crate::memline::Lines;
use crate::pos::MAXCOL;
use crate::winlayer::Buf;
use core::ffi::c_int;

/// Whether a raw string starting at `linep[startpos.col - 1]` ends
/// between `startpos` and `endpos`.
///
/// The scan reads every line between the two positions out of `lines`,
/// including the one `startpos` is on -- so a caller holding a slice from
/// that cache must have dropped it.
pub(crate) fn find_rawstring_end(lines: &mut Lines, startpos: &Pos, endpos: &Pos) -> bool {
    let (start_lnum, start_col) = (startpos.lnum, startpos.col as usize);
    let (end_lnum, end_col) = (endpos.lnum, endpos.col as usize);
    // The delimiter runs from just after the quote to the '(' -- or, when
    // the line has none, to its end. A copy, because the scan below reads
    // other lines out of the same cache.
    let delim = {
        let linep = lines.line(start_lnum);
        let tail = &linep[(start_col + 1).min(linep.len())..];
        tail[..tail.iter().position(|&b| b == b'(').unwrap_or(tail.len())].to_vec()
    };

    for lnum in start_lnum..=end_lnum {
        let line = lines.line(lnum);
        let mut at = if lnum == start_lnum {
            (start_col + 1).min(line.len())
        } else {
            0
        };
        let stop = if lnum == end_lnum {
            end_col.min(line.len())
        } else {
            line.len()
        };
        while at < stop {
            if line[at] == b')'
                && line[at + 1..].starts_with(&delim)
                && byte_at(line, at + 1 + delim.len()) == b'"'
            {
                return true;
            }
            at += 1;
        }
    }
    false
}

// ---------------------------------------------------------------------
// Line-level helpers, shared with the indent and formatting code.
// ---------------------------------------------------------------------

/// The column a `//` comment starts at on `line`, or `MAXCOL`.
///
/// With `'lisp'` set the comment character is `;` instead, and neither a
/// `#\;` nor a `;` inside a string counts.
pub fn check_linecomment(line: &[u8]) -> c_int {
    // `at` scans from the start; a byte read past the end is the line's own
    // terminator, which stops every test below.
    let mut at = 0usize;
    if Buf::current().b_p_lisp != 0 {
        // Skip Lispish one-line comments.
        if !line.contains(&b';') {
            return MAXCOL; // there are no comments
        }
        let mut in_str = false; // inside of a string
        loop {
            let Some(off) = line[at..].iter().position(|&b| b == b'"' || b == b';') else {
                return MAXCOL;
            };
            at += off;
            if line[at] == b'"' {
                if in_str {
                    if at == 0 || line[at - 1] != b'\\' {
                        in_str = false; // skip an escaped quote
                    }
                } else if at == 0
                    // skip the #\" form
                    || (at >= 2 && line[at - 1] != b'\\' && line[at - 2] != b'#')
                {
                    in_str = true;
                }
            } else if !in_str
                && (at < 2 || (line[at - 1] != b'\\' && line[at - 2] != b'#'))
                && !is_pos_in_string(line, at as ColNr)
            {
                break; // found!
            }
            at += 1;
        }
    } else {
        loop {
            let Some(off) = line[at..].iter().position(|&b| b == b'/') else {
                return MAXCOL;
            };
            at += off;
            // Accept a double '/', unless it is preceded by '*' and
            // followed by '*', because "*//*" ends one comment and
            // starts the next. Only accept the position when it is
            // not inside a string.
            if byte_at(line, at + 1) == b'/'
                && (at == 0 || line[at - 1] != b'*' || byte_at(line, at + 2) != b'*')
                && !is_pos_in_string(line, at as ColNr)
            {
                break;
            }
            at += 1;
        }
    }
    at as c_int
}

/// Whether line `lnum` is empty or holds nothing but white space.
pub fn linewhite(lnum: LineNr) -> bool {
    let mut lines = Lines::current();
    let line = lines.line(lnum);
    skip::white(line) == line.len()
}
