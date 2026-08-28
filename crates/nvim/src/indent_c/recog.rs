//! What kind of statement a line is.
//!
//! The `cin_is*` predicates `get_c_indent`'s backwards scan asks of each line
//! it walks past: is it a `case`/`default` label, a scope declaration
//! (`private:`, and whatever else 'cinscopedecls' names), a `break`, one of
//! the 'cinwords' keywords, an `if`/`else`/`do`, the `while` belonging to a
//! `do`.  [`cin_isterminated`] is the one the whole state machine turns on --
//! it answers the *character* a statement ended with (`;`, `,`, `{`, or 0 for
//! "did not end"), which is what tells a continuation line from a finished
//! one.
//!
//! Everything here walks a NUL-terminated line, because the walk interleaves
//! with [`cin_skipcomment`] and [`skip_string`]: where the walk is the whole
//! job the `unsafe` block is the whole walk, and where a line can be read as
//! a slice first it is.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int};

/// Whether `line` starts with a word from 'cinwords' -- `if`, `else`,
/// `while`, `do`, `for`, `switch` by default.
///
/// The word must be delimited on at least one side: an option item that is a
/// prefix of a longer identifier does not count unless the character before
/// it is already a non-word one.
///
/// # Safety
/// `line` must point at a NUL-terminated string.
pub unsafe fn cin_is_cinword(line: *const c_char) -> bool {
    // SAFETY: the caller's promise -- `line` is a NUL-terminated string.
    let all = unsafe { CStr::from_ptr(line) }.to_bytes();
    let start = all
        .iter()
        .position(|&b| !ascii_iswhite(c_int::from(b)))
        .unwrap_or(all.len());

    let mut cinw = cur_buf().b_p_cinw;
    // SAFETY: 'cinwords' is a NUL-terminated option string.
    let mut part = vec![0u8; unsafe { strlen(cinw) } + 1];
    let (part_len, comma) = (part.len(), c",".as_ptr().cast_mut());
    // SAFETY: `cinw` walks that same option string, which `copy_option_part`
    // advances while writing at most `part_len` bytes into `part`.
    while unsafe { *cinw } != 0 {
        let len =
            unsafe { copy_option_part(&raw mut cinw, part.as_mut_ptr().cast(), part_len, comma) };
        if !all[start..].starts_with(&part[..len]) {
            continue;
        }
        // Upstream reads `line[len - 1]`, which for an *empty* 'cinwords'
        // item on a line with no leading white space is the byte before
        // the buffer.  Refused: answering NUL gives the same verdict the
        // white-space case does, and no gate reaches it.
        let after = byte_at(all, start + len);
        let before = if start + len == 0 {
            0
        } else {
            byte_at(all, start + len - 1)
        };
        // SAFETY: `vim_iswordc` reads the current buffer's 'iskeyword' table.
        if unsafe { !vim_iswordc(c_int::from(after)) || !vim_iswordc(c_int::from(before)) } {
            return true;
        }
    }
    false
}

/// Whether `text` starts with `key:` -- the Javascript object-literal shape
/// 'cinoptions' `j1` indents against.
///
/// The key may be quoted (`'key':`, `"key":`), and `::` is C++ scope
/// resolution rather than a label.
///
/// # Safety
/// `text` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_has_js_key(text: *const c_char) -> bool {
    // SAFETY: the caller's promise, so `skipwhite` stops at the NUL at the
    // latest and its answer points into the same string.
    let s = unsafe { skipwhite(text) }.cast_const();
    // SAFETY: `s` points into that NUL-terminated string.
    let bytes = unsafe { CStr::from_ptr(s) }.to_bytes();

    let quote = match byte_at(bytes, 0) {
        q @ (b'\'' | b'"') => q,
        _ => 0,
    };
    let mut i = usize::from(quote != 0);
    // SAFETY: `vim_is_ident_char` reads the 'isident' table, which is set up
    // long before any buffer is indented.
    if !unsafe { vim_is_ident_char(c_int::from(byte_at(bytes, i))) } {
        return false; // need at least one ID character
    }
    // SAFETY: the same; `byte_at` answers the terminator past the end, which
    // is not an ID character, so `i` stops at `bytes.len()`.
    while unsafe { vim_is_ident_char(c_int::from(byte_at(bytes, i))) } {
        i += 1;
    }
    if byte_at(bytes, i) != 0 && byte_at(bytes, i) == quote {
        i += 1;
    }
    // SAFETY: `i` is at most `bytes.len()`, so `s.add(i)` is at worst the
    // string's NUL; `cin_skipcomment` answers a pointer into the same string
    // and the `:` test is kept in front of the `add(1)` behind it.
    let s = unsafe { cin_skipcomment(s.add(i)) };
    unsafe { *s as u8 == b':' && *s.add(1) as u8 != b':' }
}

/// Whether `s` is a `case` or `default` switch label.
///
/// `strict` is the C reading; without it a `"` after the `case` still counts,
/// which is what makes `case "x":` a label in Javascript.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_iscase(s: *const c_char, strict: bool) -> bool {
    // SAFETY: the caller's promise; `cin_skipcomment` answers a pointer into
    // the same string.
    let start = unsafe { cin_skipcomment(s) };
    // SAFETY: `start` points into that NUL-terminated string.
    if !unsafe { cin_starts_with(start, b"case") } {
        // SAFETY: the same.
        return unsafe { cin_isdefault(start) };
    }

    // SAFETY: `start` begins with "case", so four bytes in is still inside
    // the string, and from there the walk stops at the NUL.  Every `add`
    // steps over bytes the test in front of it has just seen, so `s` never
    // leaves the string; the `&&` chains are left whole so they keep doing
    // that.
    let mut s = unsafe { start.add(4) };
    while unsafe { *s } != 0 {
        s = unsafe { cin_skipcomment(s) };
        if unsafe { *s } == 0 {
            break;
        }
        if unsafe { *s } as u8 == b':' {
            if unsafe { *s.add(1) } as u8 == b':' {
                s = unsafe { s.add(1) }; // skip over "::" for C++
            } else {
                return true;
            }
        }
        if unsafe { *s } as u8 == b'\''
            && unsafe { *s.add(1) } != 0
            && unsafe { *s.add(2) } as u8 == b'\''
        {
            s = unsafe { s.add(2) }; // skip over ':'
        } else if unsafe { *s } as u8 == b'/'
            && (unsafe { *s.add(1) } as u8 == b'*' || unsafe { *s.add(1) } as u8 == b'/')
        {
            return false; // stop at comment
        } else if unsafe { *s } as u8 == b'"' {
            // A string ends the search under the C rules; under the
            // relaxed ones it *is* the label (`case "x":` in JS).
            return !strict;
        }
        s = unsafe { s.add(1) };
    }
    false
}

/// Whether `s` is a `default:` switch label.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_isdefault(s: *const c_char) -> bool {
    // SAFETY: the caller's promise -- `s` is a NUL-terminated string.
    if !unsafe { CStr::from_ptr(s) }
        .to_bytes()
        .starts_with(b"default")
    {
        return false;
    }
    // SAFETY: the string starts with "default", so seven bytes in is still
    // inside it, and `cin_skipcomment` answers a pointer into the same one.
    let after = unsafe { cin_skipcomment(s.add(7)) };
    // SAFETY: `after` points into that string, and the `:` test in front of
    // the `add(1)` is what says the byte behind it is there too.
    unsafe { *after as u8 == b':' && *after.add(1) as u8 != b':' }
}

/// Whether `p` is a scope declaration label named by 'cinscopedecls' --
/// `public`, `protected`, `private` by default.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_isscopedecl(p: *const c_char) -> bool {
    // SAFETY: the caller's promise; `cin_skipcomment` answers a pointer into
    // the same string.
    let s = unsafe { cin_skipcomment(p) };
    // SAFETY: `s` points into that NUL-terminated string.
    let bytes = unsafe { CStr::from_ptr(s) }.to_bytes();

    let mut cinsd = cur_buf().b_p_cinsd;
    // SAFETY: 'cinscopedecls' is a NUL-terminated option string.
    let mut part = vec![0u8; unsafe { strlen(cinsd) } + 1];
    let (part_len, comma) = (part.len(), c",".as_ptr().cast_mut());
    // SAFETY: `cinsd` walks that same option string, which `copy_option_part`
    // advances while writing at most `part_len` bytes into `part`.
    while unsafe { *cinsd } != 0 {
        let len =
            unsafe { copy_option_part(&raw mut cinsd, part.as_mut_ptr().cast(), part_len, comma) };
        if !bytes.starts_with(&part[..len]) {
            continue;
        }
        // SAFETY: `len` is the length of a prefix of `bytes`, so `s.add(len)`
        // is inside the string; `skip` points into it too, and the `:` test
        // in front of the `add(1)` says the byte behind it is there.
        let labelled = unsafe {
            let skip = cin_skipcomment(s.add(len));
            *skip as u8 == b':' && *skip.add(1) as u8 != b':'
        };
        if labelled {
            return true;
        }
    }
    false
}

/// Whether `s` starts with `word` followed by a non-identifier character.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_starts_with(s: *const c_char, word: &[u8]) -> bool {
    // SAFETY: the caller's promise -- `s` is a NUL-terminated string.
    let bytes = unsafe { CStr::from_ptr(s) }.to_bytes();
    let after = byte_at(bytes, word.len());
    // SAFETY: `vim_is_ident_char` reads the 'isident' table, which is set up
    // long before any buffer is indented.
    bytes.starts_with(word) && !unsafe { vim_is_ident_char(c_int::from(after)) }
}

/// `s` with a leading `}` -- and any comment behind it -- stepped over: the
/// `} else` / `} while (cond);` shape three of the predicates here accept.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
unsafe fn cin_skip_close_brace(s: *const c_char) -> *const c_char {
    // SAFETY: the caller's promise -- `s` is NUL-terminated, so reading its
    // first byte is in bounds.
    let brace = unsafe { *s } as u8 == b'}';
    if brace {
        // SAFETY: that byte is a `}`, so the one behind it is inside the
        // string too, and `cin_skipcomment` answers a pointer into it.
        unsafe { cin_skipcomment(s.add(1)) }
    } else {
        s
    }
}

/// Whether `p` is an `if`.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_isif(p: *const c_char) -> bool {
    // SAFETY: the caller's promise, passed straight on.
    unsafe { cin_starts_with(p, b"if") }
}

/// Whether `p` is an `else`, accepting `} else`.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_iselse(p: *const c_char) -> bool {
    // SAFETY: the caller's promise, passed straight on; both callees answer
    // a pointer into the same string.
    unsafe { cin_starts_with(cin_skip_close_brace(p), b"else") }
}

/// Whether `p` is a `do`.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_isdo(p: *const c_char) -> bool {
    // SAFETY: the caller's promise, passed straight on.
    unsafe { cin_starts_with(p, b"do") }
}

/// Whether `p` is a `break`.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_isbreak(p: *const c_char) -> bool {
    // SAFETY: the caller's promise, passed straight on.
    unsafe { cin_starts_with(p, b"break") }
}

/// Whether `p` on line `lnum` is the `while` closing a `do`.
///
/// Only `while (condition);` counts -- nothing but white space between the
/// `)` and the `;` -- because that is the shape that ends a statement rather
/// than opening one.  The condition may span lines, which is why the answer
/// needs the cursor and `findmatchlimit` rather than the text alone.
///
/// # Safety
/// `p` must point at a NUL-terminated string; moves the cursor and restores
/// it, and may unlock the current line.
pub(crate) unsafe fn cin_iswhileofdo(p: *const c_char, lnum: linenr_T) -> bool {
    // SAFETY: the caller's promise; each callee answers a pointer into the
    // same string.  "} while (cond);" counts as a while.
    let is_while = unsafe { cin_starts_with(cin_skip_close_brace(cin_skipcomment(p)), b"while") };
    if !is_while {
        return false;
    }

    let cursor_save = cur_win().w_cursor;
    cur_win().w_cursor.lnum = lnum;
    // SAFETY: on the main thread with a current buffer, and the cursor is on
    // a line of it; `get_cursor_line_ptr` hands back that line, NUL-terminated.
    let line = unsafe { CStr::from_ptr(get_cursor_line_ptr()) }.to_bytes();
    // Step over any '}' until the 'w' of the "while".
    let w = line.iter().position(|&b| b == b'w').unwrap_or(line.len());
    cur_win().w_cursor.col = w as colnr_T;

    let maxparen = int64_t::from(cur_buf().b_ind_maxparen);
    // SAFETY: the cursor is on a line of the current buffer; `ml_get_pos`
    // hands back a NUL-terminated line at the position `findmatchlimit`
    // found in it, so `add(1)` is at worst that line's NUL.
    let retval = unsafe {
        findmatchlimit(::core::ptr::null_mut::<oparg_T>(), 0, 0, maxparen)
            .is_some_and(|pos| *cin_skipcomment(ml_get_pos(&raw const pos).add(1)) as u8 == b';')
    };
    cur_win().w_cursor = cursor_save;
    retval
}

/// Whether an `if`, `for` or `while` sits just before `*poffset` in `line`,
/// and if so where -- 'cinoptions' `U`'s "is this paren a control clause's"
/// test.
///
/// # Safety
/// `line` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_is_if_for_while_before_offset(
    line: *const c_char,
    poffset: &mut c_int,
) -> bool {
    // SAFETY: the caller's promise -- `line` is a NUL-terminated string.
    let bytes = unsafe { CStr::from_ptr(line) }.to_bytes();
    let mut offset = *poffset;
    if offset < 2 {
        return false;
    }
    offset -= 1;
    while offset > 2 && ascii_iswhite(c_int::from(byte_at(bytes, offset as usize))) {
        offset -= 1;
    }

    // Each keyword is tested at the offset its *last* character would sit at,
    // walking further left as the words get longer.
    let starts_at = |off: c_int, word: &[u8]| {
        bytes
            .get(off as usize..)
            .is_some_and(|tail| tail.starts_with(word))
    };
    offset -= 1;
    if !starts_at(offset, b"if") {
        if offset < 1 {
            return false;
        }
        offset -= 1;
        if !starts_at(offset, b"for") {
            if offset < 2 {
                return false;
            }
            offset -= 2;
            if !starts_at(offset, b"while") {
                return false;
            }
        }
    }

    // It is only the keyword if nothing identifier-ish precedes it.
    let before = byte_at(bytes, (offset - 1) as usize);
    // SAFETY: `vim_is_ident_char` reads the 'isident' table, which is set up
    // long before any buffer is indented.
    if offset != 0 && unsafe { vim_is_ident_char(c_int::from(before)) } {
        return false;
    }
    *poffset = offset;
    true
}

/// Whether the cursor's line is the end of a `do ... while (...);`, and if so
/// leave the cursor on the line holding the `while`.
///
/// ```text
/// do
///    nothing;
/// while (foo
///          && bar);  <-- here
/// ```
///
/// # Safety
/// Reads and moves the cursor; may unlock the current line.
pub(crate) unsafe fn cin_iswhileofdo_end(terminated: u8) -> bool {
    if terminated != b';' {
        return false; // there must be a ';' at the end
    }
    // SAFETY: on the main thread with a current buffer; `get_cursor_line_ptr`
    // hands back the cursor's line, NUL-terminated.
    let mut line = get_cursor_line_ptr().cast_const();
    let mut p = line;
    // SAFETY: `p` walks that line and never passes its NUL.
    while unsafe { *p } != 0 {
        // SAFETY: `cin_skipcomment` answers a pointer into the same line.
        p = unsafe { cin_skipcomment(p) };
        // SAFETY: `p` is inside the line, so `add(1)` is at worst its NUL,
        // and so is `s.add(1)` behind the `;`.  The `&&` chain is left whole
        // so that each test stays in front of the step that needs it.
        let closed = unsafe {
            *p as u8 == b')' && {
                let s = skipwhite(p.add(1));
                *s as u8 == b';' && cin_nocode(s.add(1))
            }
        };
        if closed {
            // Found ");" at end of the line; now check there is a "while"
            // before the matching '('.
            // SAFETY: `p` points into `line`, so the distance is in range.
            let i = unsafe { p.offset_from(line) };
            cur_win().w_cursor.col = i as colnr_T;
            // SAFETY: searches the current buffer from the cursor, and puts
            // the cursor back where it found it.
            if let Some(trypos) = unsafe { find_match_paren(cur_buf().b_ind_maxparen) } {
                // SAFETY: `trypos` is a position in the current buffer, so
                // `ml_get` hands back its line, NUL-terminated; the two
                // skips answer pointers into that same line.
                let opener = unsafe { cin_skip_close_brace(cin_skipcomment(ml_get(trypos.lnum))) };
                // SAFETY: `opener` points into that line.
                if unsafe { cin_starts_with(opener, b"while") } {
                    cur_win().w_cursor.lnum = trypos.lnum;
                    return true;
                }
            }
            // The search may have unlocked "line"; get it again.  It left
            // the cursor where it found it, so the line is the same one and
            // `i` is still an offset into it.
            // SAFETY: the cursor is on a line of the current buffer.
            line = get_cursor_line_ptr();
            // SAFETY: `i` is an offset inside that line.
            p = unsafe { line.offset(i) };
        }
        // SAFETY: `p` is inside the line.
        if unsafe { *p } != 0 {
            // SAFETY: the byte at `p` is not the NUL, so `add(1)` stays in.
            p = unsafe { p.add(1) };
        }
    }
    false
}

/// Whether `s` ends in a backslash: the line is continued on the next one.
///
/// Unlike [`cin_ends_in`] this is the *last byte* of the line, comments and
/// all -- a backslash only continues a line when nothing follows it.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_ends_in_backslash(s: *const c_char) -> bool {
    // SAFETY: the caller's promise; `strlen` is at least 1 past the test.
    unsafe { *s != 0 && *s.add(strlen(s) - 1) as u8 == b'\\' }
}

/// Whether `s` ends with `find`, allowing white space and comments after it.
/// Strings and comments in between are skipped.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_ends_in(s: *const c_char, find: &[u8]) -> bool {
    let mut p = s;
    // SAFETY: the caller's promise -- `p` walks the NUL-terminated `s`, and
    // `cin_skipcomment`/`skipwhite` answer pointers into it.  `add(find.len())`
    // is inside because the `starts_with` in front of it matched that many
    // bytes, and `add(1)` runs only on a byte that is not the NUL.
    while unsafe { *p } != 0 {
        p = unsafe { cin_skipcomment(p) };
        if unsafe { CStr::from_ptr(p) }.to_bytes().starts_with(find)
            && unsafe { cin_nocode(skipwhite(p.add(find.len()))) }
        {
            return true;
        }
        if unsafe { *p } != 0 {
            p = unsafe { p.add(1) };
        }
    }
    false
}

/// The character a statement on `s` ended with -- `;`, `}`, `,` or `{` -- or
/// 0 when it did not end.
///
/// This is the fact `get_c_indent`'s whole backwards scan turns on: a
/// terminated line above is something to line up with, an unterminated one is
/// a statement still being written.  A `,` only counts with `incl_comma`, and
/// an opening `{` only with `incl_open`; a `{` or `}` at the *start* is the
/// fallback answer when nothing else terminates.
///
/// `} else` is deliberately not terminated -- the `else` continues the
/// statement -- which is what `is_else` suppresses until its block closes.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_isterminated(s: *const c_char, incl_open: bool, incl_comma: bool) -> u8 {
    // SAFETY: the caller's promise; `cin_skipcomment` answers a pointer into
    // the same string.
    let mut s = unsafe { cin_skipcomment(s) };
    let mut n_open = 0u32;

    // SAFETY: `s` points into that NUL-terminated string, and `cin_iselse`
    // only reads it.  The `}` test is kept in front of `cin_iselse`, as
    // upstream has it, so it is asked no more often than it was.
    let first = unsafe { *s } as u8;
    let found_start = if first == b'{' || (first == b'}' && !unsafe { cin_iselse(s) }) {
        first
    } else {
        0
    };
    // SAFETY: the same.
    let is_else = found_start == 0 && unsafe { cin_iselse(s) };

    // SAFETY: `s` walks the same NUL-terminated string; `cin_skipcomment` and
    // `skip_string` answer pointers into it, `add(1)` is at worst its NUL --
    // which `cin_nocode` only reads -- and the final `add(1)` runs only on a
    // byte that is not the NUL.
    while unsafe { *s } != 0 {
        // Skip over comments, "" strings and 'c'haracters.
        s = unsafe { skip_string(cin_skipcomment(s)) };
        if unsafe { *s } as u8 == b'}' && n_open > 0 {
            n_open -= 1;
        }
        if (!is_else || n_open == 0)
            && (unsafe { *s } as u8 == b';'
                || unsafe { *s } as u8 == b'}'
                || (incl_comma && unsafe { *s } as u8 == b','))
            && unsafe { cin_nocode(s.add(1)) }
        {
            return unsafe { *s } as u8;
        } else if unsafe { *s } as u8 == b'{' {
            if incl_open && unsafe { cin_nocode(s.add(1)) } {
                return unsafe { *s } as u8;
            }
            n_open += 1;
        }
        if unsafe { *s } != 0 {
            s = unsafe { s.add(1) };
        }
    }
    found_start
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
