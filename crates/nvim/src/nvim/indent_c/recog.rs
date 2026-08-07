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
//! with [`cin_skipcomment`] and [`skip_string`]: the unit of work is a C
//! string, so one `unsafe` block per entry point is the honest shape.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use ::core::ffi::{CStr, c_char, c_int};

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
    unsafe {
        let all = CStr::from_ptr(line).to_bytes();
        let start = all
            .iter()
            .position(|&b| !ascii_iswhite(c_int::from(b)))
            .unwrap_or(all.len());

        let mut cinw = (*curbuf.get()).b_p_cinw;
        let mut part = vec![0u8; strlen(cinw) + 1];
        while *cinw != 0 {
            let len = copy_option_part(
                &raw mut cinw,
                part.as_mut_ptr().cast::<c_char>(),
                part.len(),
                c",".as_ptr().cast_mut(),
            );
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
            if !vim_iswordc(c_int::from(after)) || !vim_iswordc(c_int::from(before)) {
                return true;
            }
        }
        false
    }
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
    unsafe {
        let mut s = skipwhite(text).cast_const();
        let mut quote = 0u8;
        if *s as u8 == b'\'' || *s as u8 == b'"' {
            quote = *s as u8;
            s = s.add(1);
        }
        if !vim_isIDc(c_int::from(*s as u8)) {
            return false; // need at least one ID character
        }
        while vim_isIDc(c_int::from(*s as u8)) {
            s = s.add(1);
        }
        if *s != 0 && *s as u8 == quote {
            s = s.add(1);
        }
        s = cin_skipcomment(s);
        *s as u8 == b':' && *s.add(1) as u8 != b':'
    }
}

/// Whether `s` is a `case` or `default` switch label.
///
/// `strict` is the C reading; without it a `"` after the `case` still counts,
/// which is what makes `case "x":` a label in Javascript.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_iscase(s: *const c_char, strict: bool) -> bool {
    unsafe {
        let mut s = cin_skipcomment(s);
        if !cin_starts_with(s, b"case") {
            return cin_isdefault(s);
        }
        s = s.add(4);
        while *s != 0 {
            s = cin_skipcomment(s);
            if *s == 0 {
                break;
            }
            if *s as u8 == b':' {
                if *s.add(1) as u8 == b':' {
                    s = s.add(1); // skip over "::" for C++
                } else {
                    return true;
                }
            }
            if *s as u8 == b'\'' && *s.add(1) != 0 && *s.add(2) as u8 == b'\'' {
                s = s.add(2); // skip over ':'
            } else if *s as u8 == b'/' && (*s.add(1) as u8 == b'*' || *s.add(1) as u8 == b'/') {
                return false; // stop at comment
            } else if *s as u8 == b'"' {
                // A string ends the search under the C rules; under the
                // relaxed ones it *is* the label (`case "x":` in JS).
                return !strict;
            }
            s = s.add(1);
        }
        false
    }
}

/// Whether `s` is a `default:` switch label.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_isdefault(s: *const c_char) -> bool {
    unsafe {
        if !CStr::from_ptr(s).to_bytes().starts_with(b"default") {
            return false;
        }
        let s = cin_skipcomment(s.add(7));
        *s as u8 == b':' && *s.add(1) as u8 != b':'
    }
}

/// Whether `p` is a scope declaration label named by 'cinscopedecls' --
/// `public`, `protected`, `private` by default.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_isscopedecl(p: *const c_char) -> bool {
    unsafe {
        let s = cin_skipcomment(p);
        let bytes = CStr::from_ptr(s).to_bytes();

        let mut cinsd = (*curbuf.get()).b_p_cinsd;
        let mut part = vec![0u8; strlen(cinsd) + 1];
        while *cinsd != 0 {
            let len = copy_option_part(
                &raw mut cinsd,
                part.as_mut_ptr().cast::<c_char>(),
                part.len(),
                c",".as_ptr().cast_mut(),
            );
            if bytes.starts_with(&part[..len]) {
                let skip = cin_skipcomment(s.add(len));
                if *skip as u8 == b':' && *skip.add(1) as u8 != b':' {
                    return true;
                }
            }
        }
        false
    }
}

/// Whether `s` starts with `word` followed by a non-identifier character.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_starts_with(s: *const c_char, word: &[u8]) -> bool {
    unsafe {
        let bytes = CStr::from_ptr(s).to_bytes();
        bytes.starts_with(word) && !vim_isIDc(c_int::from(byte_at(bytes, word.len())))
    }
}

/// Whether `p` is an `if`.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_isif(p: *const c_char) -> bool {
    unsafe { cin_starts_with(p, b"if") }
}

/// Whether `p` is an `else`, accepting `} else`.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_iselse(p: *const c_char) -> bool {
    unsafe {
        let p = if *p as u8 == b'}' {
            cin_skipcomment(p.add(1))
        } else {
            p
        };
        cin_starts_with(p, b"else")
    }
}

/// Whether `p` is a `do`.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_isdo(p: *const c_char) -> bool {
    unsafe { cin_starts_with(p, b"do") }
}

/// Whether `p` is a `break`.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_isbreak(p: *const c_char) -> bool {
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
    unsafe {
        let mut p = cin_skipcomment(p);
        if *p as u8 == b'}' {
            p = cin_skipcomment(p.add(1)); // accept "} while (cond);"
        }
        if !cin_starts_with(p, b"while") {
            return false;
        }
        let cursor_save = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor.lnum = lnum;
        (*curwin.get()).w_cursor.col = 0;
        let mut p = get_cursor_line_ptr().cast_const();
        // Step over any '}' until the 'w' of the "while".
        while *p != 0 && *p as u8 != b'w' {
            p = p.add(1);
            (*curwin.get()).w_cursor.col += 1;
        }
        let trypos = findmatchlimit(
            ::core::ptr::null_mut::<oparg_T>(),
            0,
            0,
            int64_t::from((*curbuf.get()).b_ind_maxparen),
        );
        let retval = !trypos.is_null() && *cin_skipcomment(ml_get_pos(trypos).add(1)) as u8 == b';';
        (*curwin.get()).w_cursor = cursor_save;
        retval
    }
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
    unsafe {
        let bytes = CStr::from_ptr(line).to_bytes();
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
        if offset != 0 && vim_isIDc(c_int::from(byte_at(bytes, (offset - 1) as usize))) {
            return false;
        }
        *poffset = offset;
        true
    }
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
    unsafe {
        if terminated != b';' {
            return false; // there must be a ';' at the end
        }
        let mut line = get_cursor_line_ptr().cast_const();
        let mut p = line;
        while *p != 0 {
            p = cin_skipcomment(p);
            if *p as u8 == b')' {
                let s = skipwhite(p.add(1));
                if *s as u8 == b';' && cin_nocode(s.add(1)) {
                    // Found ");" at end of the line; now check there is a
                    // "while" before the matching '('.
                    let i = p.offset_from(line);
                    (*curwin.get()).w_cursor.col = i as colnr_T;
                    let trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                    if !trypos.is_null() {
                        let mut s = cin_skipcomment(ml_get((*trypos).lnum));
                        if *s as u8 == b'}' {
                            s = cin_skipcomment(s.add(1)); // accept "} while (cond);"
                        }
                        if cin_starts_with(s, b"while") {
                            (*curwin.get()).w_cursor.lnum = (*trypos).lnum;
                            return true;
                        }
                    }
                    // The search may have unlocked "line"; get it again.
                    line = get_cursor_line_ptr();
                    p = line.offset(i);
                }
            }
            if *p != 0 {
                p = p.add(1);
            }
        }
        false
    }
}

/// Whether `s` ends with `find`, allowing white space and comments after it.
/// Strings and comments in between are skipped.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_ends_in(s: *const c_char, find: &[u8]) -> bool {
    unsafe {
        let mut p = s;
        while *p != 0 {
            p = cin_skipcomment(p);
            if CStr::from_ptr(p).to_bytes().starts_with(find)
                && cin_nocode(skipwhite(p.add(find.len())))
            {
                return true;
            }
            if *p != 0 {
                p = p.add(1);
            }
        }
        false
    }
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
    unsafe {
        let mut s = cin_skipcomment(s);
        let mut n_open = 0u32;

        let found_start = if *s as u8 == b'{' || (*s as u8 == b'}' && !cin_iselse(s)) {
            *s as u8
        } else {
            0
        };
        let is_else = found_start == 0 && cin_iselse(s);

        while *s != 0 {
            // Skip over comments, "" strings and 'c'haracters.
            s = skip_string(cin_skipcomment(s));
            if *s as u8 == b'}' && n_open > 0 {
                n_open -= 1;
            }
            if (!is_else || n_open == 0)
                && (*s as u8 == b';' || *s as u8 == b'}' || (incl_comma && *s as u8 == b','))
                && cin_nocode(s.add(1))
            {
                return *s as u8;
            } else if *s as u8 == b'{' {
                if incl_open && cin_nocode(s.add(1)) {
                    return *s as u8;
                }
                n_open += 1;
            }
            if *s != 0 {
                s = s.add(1);
            }
        }
        found_start
    }
}
