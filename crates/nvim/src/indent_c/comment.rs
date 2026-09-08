//! Where a comment or a string starts, and how to step over one.
//!
//! Every recogniser in this family has to answer over *code*, so each one
//! opens by calling [`cin_skipcomment`], and the ones that walk a whole line
//! call [`skip_string`] too.  The `find_start_*` half is the other direction:
//! given the cursor, `findmatchlimit` backwards for the `/*` or the
//! `R"delim(` that encloses it, bounded by 'cinoptions' `*N`
//! (`b_ind_maxcomment`).  [`ind_find_start_comment_or_raw_string`] is the pair asked at once --
//! Comment Or Raw String -- and answers whichever starts later.
//!
//! Everything here is written over `&[u8]` and answers a byte *offset* into
//! the line it was given, so it is ordinary safe code with tests.  The C
//! names, for anyone reading upstream beside this:
//!
//! | C | here |
//! | --- | --- |
//! | `cin_skipcomment` | [`code_at`] |
//! | `skip_string` | [`string_end_at`] |
//! | `cin_skip_comment_and_string` | [`code_or_string_at`] |
//! | `cin_nocode` | [`only_comment_left`] |
//! | `cin_iscomment` | [`starts_comment`] |
//! | `cin_islinecomment` | [`starts_line_comment`] |

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

/// Where the comment enclosing the cursor starts, bounded by 'cinoptions'
/// `*N`.
///
/// # Safety
/// Reads the current buffer and window; the current line may be unlocked.
pub(crate) unsafe fn ind_find_start_comment() -> Option<Pos> {
    unsafe { find_start_comment(Buf::current().b_ind_maxcomment) }
}

/// Search backwards from the cursor for the `/*` that opens the comment it is
/// inside, giving up `ind_maxcomment` lines back.
///
/// A `/*` that is itself inside a string does not open a comment, so on
/// finding one the search restarts *below* that line -- which is what the
/// shrinking `cur_maxcomment` expresses.
///
/// # Safety
/// Reads the current buffer and window; the current line may be unlocked.
pub unsafe fn find_start_comment(ind_maxcomment: c_int) -> Option<Pos> {
    let mut cur_maxcomment = int64_t::from(ind_maxcomment);
    loop {
        // SAFETY: on the main thread, with a current window and buffer.
        let pos = unsafe {
            findmatchlimit(
                ::core::ptr::null_mut::<OpArg>(),
                c_int::from(b'*'),
                FM_BACKWARD,
                cur_maxcomment,
            )
        }?;
        // `findmatchlimit` found `pos` in the current buffer, so the cache
        // answers with the line it sits on.
        if !is_pos_in_string(Lines::current().line(pos.lnum), pos.col) {
            return Some(pos);
        }
        cur_maxcomment = int64_t::from(Win::current().w_cursor.lnum - pos.lnum - 1);
        if cur_maxcomment <= 0 {
            return None;
        }
    }
}

/// [`find_start_comment`] for a raw string literal's `R"delim(` instead.
///
/// # Safety
/// Reads the current buffer and window; the current line may be unlocked.
pub(crate) unsafe fn find_start_rawstring(ind_maxcomment: c_int) -> Option<Pos> {
    let mut cur_maxcomment = ind_maxcomment;
    loop {
        // SAFETY: on the main thread, with a current window and buffer.
        let pos = unsafe {
            findmatchlimit(
                ::core::ptr::null_mut::<OpArg>(),
                c_int::from(b'R'),
                FM_BACKWARD,
                int64_t::from(cur_maxcomment),
            )
        }?;
        // `findmatchlimit` found `pos` in the current buffer, so the cache
        // answers with the line it sits on.
        if !is_pos_in_string(Lines::current().line(pos.lnum), pos.col) {
            return Some(pos);
        }
        cur_maxcomment = (Win::current().w_cursor.lnum - pos.lnum - 1) as c_int;
        if cur_maxcomment <= 0 {
            return None;
        }
    }
}

/// Comment Or Raw String: whichever of the two encloses the cursor.
///
/// If both answer, the later one wins -- the earlier one contains it, so the
/// cursor is really inside the later.  `is_raw` is set to the line number
/// when the answer is a raw string, which is how `get_c_indent` knows not to
/// treat that line as an unterminated statement.
///
/// # Safety
/// Reads the current buffer and window; the current line may be unlocked.
pub(crate) unsafe fn ind_find_start_comment_or_raw_string(
    is_raw: Option<&mut LineNr>,
) -> Option<Pos> {
    // SAFETY: on the main thread, with a current window and buffer.
    let comment_pos = unsafe { find_start_comment(Buf::current().b_ind_maxcomment) };
    // SAFETY: the same.
    let rs_pos = unsafe { find_start_rawstring(Buf::current().b_ind_maxcomment) };

    let raw_wins = match (comment_pos, rs_pos) {
        (None, _) => true,
        (Some(comment), Some(raw)) => lt(raw, comment),
        (Some(_), None) => false,
    };
    if raw_wins {
        if let Some(is_raw) = is_raw
            && let Some(raw) = rs_pos
        {
            *is_raw = raw.lnum;
        }
        return rs_pos;
    }
    comment_pos
}

/// Step over the run of `"string"`s and `'c'` constants starting at
/// `line[at]`, answering the offset upstream's pointer walk ends on.
///
/// Strings concatenate (`"date""time"`), which is why this is a loop, and the
/// walk deliberately ends one byte *past* the closing quote -- upstream's
/// `for (;; p++)` runs its increment on every `continue`.  Ending on the
/// terminator steps back one, so the answer is always a byte of the line --
/// which is why [`find_last_paren`] can go round again on it.
///
/// `at` at the end of the line therefore answers the byte *before* it.  That
/// is upstream's own arithmetic, and `find_last_paren` is the caller that
/// reaches it (a line ending in a comment); nothing calls this on an empty
/// line, where the answer would have to be negative.
pub(crate) fn string_end_at(line: &[u8], at: usize) -> usize {
    let mut p = at;
    loop {
        if byte_at(line, p) == b'\'' {
            // 'c', '\n' or '\000'.
            if byte_at(line, p + 1) == 0 {
                break; // ' at end of line
            }
            let mut i = 2;
            if byte_at(line, p + 1) == b'\\' && byte_at(line, p + 2) != 0 {
                i += 1;
                while byte_at(line, p + i - 1).is_ascii_digit() {
                    i += 1;
                }
            }
            // Check for the trailing '.
            if byte_at(line, p + i - 1) == 0 || byte_at(line, p + i) != b'\'' {
                break;
            }
            p += i;
        } else if byte_at(line, p) == b'"' {
            p += 1;
            while byte_at(line, p) != 0 {
                if byte_at(line, p) == b'\\' && byte_at(line, p + 1) != 0 {
                    p += 1;
                } else if byte_at(line, p) == b'"' {
                    break; // end of string
                }
                p += 1;
            }
            if byte_at(line, p) != b'"' {
                break;
            }
        } else if byte_at(line, p) == b'R' && byte_at(line, p + 1) == b'"' {
            // Raw string: R"[delim](...)[delim]"
            let delim = p + 2;
            let Some(delim_len) = line
                .get(delim..)
                .and_then(|t| t.iter().position(|&b| b == b'('))
            else {
                break;
            };
            p += 3;
            while byte_at(line, p) != 0 {
                if byte_at(line, p) == b')'
                    && line[p + 1..].starts_with(&line[delim..delim + delim_len])
                    && byte_at(line, p + delim_len + 1) == b'"'
                {
                    p += delim_len + 1;
                    break;
                }
                p += 1;
            }
            if byte_at(line, p) != b'"' {
                break;
            }
        } else {
            break; // no string found
        }
        p += 1;
    }
    // Back up off the terminator, as upstream does.
    if byte_at(line, p) == 0 {
        p.saturating_sub(1)
    } else {
        p
    }
}

/// Whether `line[col]` is inside a C string.
pub fn is_pos_in_string(line: &[u8], col: ColNr) -> bool {
    let mut p = 0usize;
    while p < line.len() && (p as ColNr) < col {
        // `p < line.len()` is upstream's `*p`, so the walk below cannot be
        // the "back up off the terminator" case and `p` strictly grows.
        p = string_end_at(line, p) + 1;
    }
    p as ColNr > col
}

/// Whether 'cinoptions' `#N` is on, i.e. a `#` starts a comment.
///
/// `false` where there is no buffer, which is the library test harness and
/// nothing else -- and is also the option's default.
fn hash_comments() -> bool {
    Buf::current_or_none().is_some_and(|buffer| buffer.b_ind_hash_comment != 0)
}

/// The offset of the first byte of code at or after `at`: white space and C
/// comments -- and, with 'cinoptions' `#N`, Perl/shell `#` comments -- are
/// stepped over.
pub(crate) fn code_at(line: &[u8], at: usize) -> usize {
    code_from(line, at, hash_comments())
}

/// [`code_at`] with the `#` rule spelled out rather than read off the buffer.
///
/// The `#` form requires a space in front of it, so that `$#array` is not
/// read as a comment.
fn code_from(line: &[u8], at: usize, hash_comment: bool) -> usize {
    let mut p = at.min(line.len());
    while byte_at(line, p) != 0 {
        let prev = p;
        while ascii_iswhite(c_int::from(byte_at(line, p))) {
            p += 1;
        }
        // A Perl/shell `#` comment runs to end of line.
        if hash_comment && p != prev && byte_at(line, p) == b'#' {
            return line.len();
        }
        if byte_at(line, p) != b'/' {
            break;
        }
        p += 1;
        if byte_at(line, p) == b'/' {
            // A `//` comment runs to end of line.
            return line.len();
        }
        if byte_at(line, p) != b'*' {
            break;
        }
        p += 1;
        while byte_at(line, p) != 0 {
            if byte_at(line, p) == b'*' && byte_at(line, p + 1) == b'/' {
                p += 2;
                break;
            }
            p += 1;
        }
    }
    p
}

/// Whether there is no code left at `at`: white space and comments are not
/// code.
pub(crate) fn only_comment_left(line: &[u8], at: usize) -> bool {
    code_at(line, at) >= line.len()
}

/// The nearest `//` comment above the cursor, skipping blank lines.
pub(crate) fn find_line_comment() -> Option<Pos> {
    let mut pos = Win::current().w_cursor;
    let mut lines = Lines::current();
    loop {
        pos.lnum -= 1;
        if pos.lnum <= 0 {
            return None;
        }
        let line = lines.line(pos.lnum);
        let col = skip::white(line);
        if starts_line_comment(line, col) {
            return Some(pos.with_col(col as ColNr));
        }
        if col < line.len() {
            return None; // code before it: not a comment line
        }
    }
}

/// Step over comments *and* strings together, in either order.
///
/// They interleave: `"string0" /*comment*/ "string1"` is one run, and neither
/// skipper alone gets past it.
pub(crate) fn code_or_string_at(line: &[u8], at: usize) -> usize {
    let mut p = at;
    loop {
        let before = p;
        p = code_at(line, p);
        if p < line.len() {
            p = string_end_at(line, p);
        }
        if p == before {
            return p;
        }
    }
}

/// The start of a C or C++ comment.
pub(crate) fn starts_comment(line: &[u8], at: usize) -> bool {
    byte_at(line, at) == b'/' && matches!(byte_at(line, at + 1), b'*' | b'/')
}

/// The start of a `//` comment.
pub(crate) fn starts_line_comment(line: &[u8], at: usize) -> bool {
    byte_at(line, at) == b'/' && byte_at(line, at + 1) == b'/'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skip(s: &str) -> isize {
        // The offset form never answers a negative, so the empty case --
        // upstream's "back up onto the byte before the argument" -- is read
        // off the saturating answer instead.
        if s.is_empty() {
            return -1;
        }
        string_end_at(s.as_bytes(), 0) as isize
    }

    fn skip_comment(s: &[u8], hash_comment: bool) -> usize {
        code_from(s, 0, hash_comment)
    }

    #[test]
    fn skip_string_stops_past_a_closing_quote() {
        // Upstream's loop increment runs on every continue, so the walk ends
        // one byte past the string -- unless that byte is the terminator, in
        // which case it steps back onto the quote.
        assert_eq!(skip("\"abc\" x"), 5);
        assert_eq!(skip("\"abc\""), 4);
        assert_eq!(skip("plain"), 0);
        // The empty tail: upstream steps off the NUL onto the byte *before*
        // the argument, which `find_last_paren` reaches after a trailing
        // comment and which makes it revisit the line's last byte.
        assert_eq!(skip(""), -1);
        // From the middle of a line, the answer is an offset into the whole
        // of it: the walk is the same one, only its origin moves.
        assert_eq!(string_end_at(b"x = \"abc\" y", 4), 9);
        // At the end of a line, upstream steps back onto the last byte --
        // which is what makes `find_last_paren` look at it a second time.
        assert_eq!(string_end_at(b"code", 4), 3);
    }

    #[test]
    fn skip_string_concatenates() {
        assert_eq!(skip("\"date\"\"time\"!"), 12);
    }

    #[test]
    fn skip_string_takes_escapes_and_char_constants() {
        assert_eq!(skip("\"a\\\"b\" "), 6);
        assert_eq!(skip("'c' "), 3);
        assert_eq!(skip("'\\n' "), 4);
        // An *octal* escape is not recognised: upstream's digit scan opens
        // at `i = 3` and then adds one per digit, so it looks for the
        // closing quote one byte past where it is (O-B15-18).  Reproduced.
        assert_eq!(skip("'\\0' "), 0);
        assert_eq!(skip("'\\000' "), 0);
        // No digit count works: the closing quote of `'\\<k digits>'` is at
        // `2 + k` and the scan always looks at `3 + k`.  A non-digit escape
        // is fine, which is what localises the defect to the digit loop.
        assert_eq!(skip("'\\0000' "), 0);
        assert_eq!(skip("'\\\\' "), 4);
        // An unterminated char constant is not one.
        assert_eq!(skip("'abc"), 0);
        assert_eq!(skip("'"), 0);
    }

    #[test]
    fn skip_string_takes_raw_strings() {
        assert_eq!(skip("R\"d(x)d\" "), 8);
        assert_eq!(skip("R\"(x)\" "), 6);
        // No `(` after the delimiter: not a raw string at all.
        assert_eq!(skip("R\"nope"), 0);
        // The closing delimiter never arrives.
        assert_eq!(skip("R\"d(xxx"), 6);
    }

    #[test]
    fn skip_comment_takes_white_space_and_both_comment_forms() {
        assert_eq!(skip_comment(b"  code", false), 2);
        assert_eq!(skip_comment(b"/* c */x", false), 7);
        assert_eq!(skip_comment(b"/* a */ /* b */x", false), 15);
        assert_eq!(skip_comment(b"// c", false), 4);
        // An unterminated /* eats the rest of the line.
        assert_eq!(skip_comment(b"/* c", false), 4);
    }

    #[test]
    fn skip_comment_needs_a_space_before_a_hash() {
        assert_eq!(skip_comment(b" # c", true), 4);
        assert_eq!(skip_comment(b" # c", false), 1);
        // `$#array` has no space in front of the `#`, so it is code.
        assert_eq!(skip_comment(b"$#array", true), 0);
    }
}
