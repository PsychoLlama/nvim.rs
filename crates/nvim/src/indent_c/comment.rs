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
//! The scanners here are written over `&[u8]` and answer a byte *index*, so
//! they are ordinary safe code with tests; the pointer forms the rest of the
//! family calls are one-line wrappers over them.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::NUL;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int};

/// Where the comment enclosing the cursor starts, bounded by 'cinoptions'
/// `*N`.
///
/// # Safety
/// Reads the current buffer and window; the current line may be unlocked.
pub(crate) unsafe fn ind_find_start_comment() -> Option<pos_T> {
    unsafe { find_start_comment(cur_buf().b_ind_maxcomment) }
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
pub unsafe fn find_start_comment(ind_maxcomment: c_int) -> Option<pos_T> {
    let mut cur_maxcomment = int64_t::from(ind_maxcomment);
    loop {
        // SAFETY: on the main thread, with a current window and buffer.
        let pos = unsafe {
            findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                c_int::from(b'*'),
                FM_BACKWARD,
                cur_maxcomment,
            )
        }?;
        // SAFETY: `findmatchlimit` found `pos` in the current buffer, so
        // `ml_get` answers with the line it sits on.
        if !unsafe { is_pos_in_string(ml_get(pos.lnum), pos.col) } {
            return Some(pos);
        }
        cur_maxcomment = int64_t::from(cur_win().w_cursor.lnum - pos.lnum - 1);
        if cur_maxcomment <= 0 {
            return None;
        }
    }
}

/// [`find_start_comment`] for a raw string literal's `R"delim(` instead.
///
/// # Safety
/// Reads the current buffer and window; the current line may be unlocked.
pub(crate) unsafe fn find_start_rawstring(ind_maxcomment: c_int) -> Option<pos_T> {
    let mut cur_maxcomment = ind_maxcomment;
    loop {
        // SAFETY: on the main thread, with a current window and buffer.
        let pos = unsafe {
            findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                c_int::from(b'R'),
                FM_BACKWARD,
                int64_t::from(cur_maxcomment),
            )
        }?;
        // SAFETY: `findmatchlimit` found `pos` in the current buffer, so
        // `ml_get` answers with the line it sits on.
        if !unsafe { is_pos_in_string(ml_get(pos.lnum), pos.col) } {
            return Some(pos);
        }
        cur_maxcomment = (cur_win().w_cursor.lnum - pos.lnum - 1) as c_int;
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
    is_raw: Option<&mut linenr_T>,
) -> Option<pos_T> {
    // SAFETY: on the main thread, with a current window and buffer.
    let comment_pos = unsafe { find_start_comment(cur_buf().b_ind_maxcomment) };
    // SAFETY: the same.
    let rs_pos = unsafe { find_start_rawstring(cur_buf().b_ind_maxcomment) };

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

/// Step over the run of `"string"`s and `'c'` constants starting at `s[0]`,
/// answering the index upstream's pointer walk ends on.
///
/// Strings concatenate (`"date""time"`), which is why this is a loop, and the
/// walk deliberately ends one byte *past* the closing quote -- upstream's
/// `for (;; p++)` runs its increment on every `continue`.  Ending on the NUL
/// steps back one, so the answer is always inside `s`.
///
/// The answer is *signed* because upstream backs up off the NUL
/// unconditionally: over an empty tail that is the byte before `s`, which
/// `find_last_paren` genuinely reaches and depends on.
fn string_end(s: &[u8]) -> isize {
    let mut p = 0usize;
    loop {
        if byte_at(s, p) == b'\'' {
            // 'c', '\n' or '\000'.
            if byte_at(s, p + 1) == 0 {
                break; // ' at end of line
            }
            let mut i = 2;
            if byte_at(s, p + 1) == b'\\' && byte_at(s, p + 2) != 0 {
                i += 1;
                while byte_at(s, p + i - 1).is_ascii_digit() {
                    i += 1;
                }
            }
            // Check for the trailing '.
            if byte_at(s, p + i - 1) == 0 || byte_at(s, p + i) != b'\'' {
                break;
            }
            p += i;
        } else if byte_at(s, p) == b'"' {
            p += 1;
            while byte_at(s, p) != 0 {
                if byte_at(s, p) == b'\\' && byte_at(s, p + 1) != 0 {
                    p += 1;
                } else if byte_at(s, p) == b'"' {
                    break; // end of string
                }
                p += 1;
            }
            if byte_at(s, p) != b'"' {
                break;
            }
        } else if byte_at(s, p) == b'R' && byte_at(s, p + 1) == b'"' {
            // Raw string: R"[delim](...)[delim]"
            let delim = p + 2;
            let Some(delim_len) = s
                .get(delim..)
                .and_then(|t| t.iter().position(|&b| b == b'('))
            else {
                break;
            };
            p += 3;
            while byte_at(s, p) != 0 {
                if byte_at(s, p) == b')'
                    && s[p + 1..].starts_with(&s[delim..delim + delim_len])
                    && byte_at(s, p + delim_len + 1) == b'"'
                {
                    p += delim_len + 1;
                    break;
                }
                p += 1;
            }
            if byte_at(s, p) != b'"' {
                break;
            }
        } else {
            break; // no string found
        }
        p += 1;
    }
    // Back up off the NUL, as upstream does -- to -1 when `s` is empty.
    if byte_at(s, p) == 0 {
        p as isize - 1
    } else {
        p as isize
    }
}

/// [`string_end`] over a pointer: past the run of strings starting at `p`.
///
/// # Safety
/// `p` must point at a NUL-terminated string, and -- because an empty one
/// answers the byte *before* it -- must not be the start of its allocation
/// when it is empty.  `find_last_paren` is the only caller that reaches
/// that, and only from `p >= line + 1`.
pub(crate) unsafe fn skip_string(p: *const c_char) -> *const c_char {
    unsafe { p.offset(string_end(CStr::from_ptr(p).to_bytes())) }
}

/// Whether `line[col]` is inside a C string.
///
/// # Safety
/// `line` must point at a NUL-terminated string.
pub unsafe fn is_pos_in_string(line: *const c_char, col: colnr_T) -> bool {
    let s = unsafe { CStr::from_ptr(line).to_bytes() };
    let mut p = 0usize;
    while p < s.len() && (p as colnr_T) < col {
        // `p < s.len()` is upstream's `*p`, so the tail is non-empty and the
        // signed answer is non-negative.
        p = (p as isize + string_end(&s[p..]) + 1) as usize;
    }
    p as colnr_T > col
}

/// Step over white space and C comments -- and, with 'cinoptions' `#N`, over
/// Perl/shell `#` comments too.
///
/// The `#` form requires a space in front of it, so that `$#array` is not
/// read as a comment.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_skipcomment(s: *const c_char) -> *const c_char {
    let hash_comment = cur_buf().b_ind_hash_comment != 0;
    // SAFETY: the caller's promise -- `s` is NUL-terminated, and
    // `skip_comment` answers an index no further than that NUL.
    unsafe { s.add(skip_comment(CStr::from_ptr(s).to_bytes(), hash_comment)) }
}

/// [`cin_skipcomment`] over a slice: the index of the first byte of code.
fn skip_comment(s: &[u8], hash_comment: bool) -> usize {
    let mut p = 0usize;
    while byte_at(s, p) != 0 {
        let prev = p;
        while ascii_iswhite(c_int::from(byte_at(s, p))) {
            p += 1;
        }
        // A Perl/shell `#` comment runs to end of line.
        if hash_comment && p != prev && byte_at(s, p) == b'#' {
            return s.len();
        }
        if byte_at(s, p) != b'/' {
            break;
        }
        p += 1;
        if byte_at(s, p) == b'/' {
            // A `//` comment runs to end of line.
            return s.len();
        }
        if byte_at(s, p) != b'*' {
            break;
        }
        p += 1;
        while byte_at(s, p) != 0 {
            if byte_at(s, p) == b'*' && byte_at(s, p + 1) == b'/' {
                p += 2;
                break;
            }
            p += 1;
        }
    }
    p
}

/// Whether there is no code at `s`: white space and comments are not code.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_nocode(s: *const c_char) -> bool {
    unsafe { *cin_skipcomment(s) as c_int == NUL }
}

/// The nearest `//` comment above the cursor, skipping blank lines.
///
/// # Safety
/// Reads the current buffer and window.
pub(crate) unsafe fn find_line_comment() -> Option<pos_T> {
    let mut pos = cur_win().w_cursor;
    loop {
        pos.lnum -= 1;
        if pos.lnum <= 0 {
            return None;
        }
        // SAFETY: on the main thread with a current buffer; `ml_get` hands
        // back a NUL-terminated line, which is all the rest ask for.
        let (is_comment, col, at_end) = unsafe {
            let line = ml_get(pos.lnum);
            let p = skipwhite(line);
            (
                cin_islinecomment(p),
                p.offset_from(line) as colnr_T,
                *p as c_int == NUL,
            )
        };
        if is_comment {
            return Some(pos.with_col(col));
        }
        if !at_end {
            return None;
        }
    }
}

/// Step over comments *and* strings together, in either order.
///
/// They interleave: `"string0" /*comment*/ "string1"` is one run, and neither
/// skipper alone gets past it.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_skip_comment_and_string(s: *const c_char) -> *const c_char {
    let mut p = s;
    loop {
        let r = p;
        // SAFETY: `s` is NUL-terminated and neither skipper walks past its
        // NUL, so every `p` the loop sees is NUL-terminated too.
        p = unsafe {
            let p = cin_skipcomment(p);
            if *p != 0 { skip_string(p) } else { p }
        };
        if p == r {
            return p;
        }
    }
}

/// The start of a C or C++ comment.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_iscomment(p: *const c_char) -> bool {
    unsafe { *p == b'/' as c_char && (*p.add(1) == b'*' as c_char || *p.add(1) == b'/' as c_char) }
}

/// The start of a `//` comment.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_islinecomment(p: *const c_char) -> bool {
    unsafe { *p == b'/' as c_char && *p.add(1) == b'/' as c_char }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skip(s: &str) -> isize {
        string_end(s.as_bytes())
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
