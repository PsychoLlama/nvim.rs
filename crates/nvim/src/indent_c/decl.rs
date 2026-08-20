//! Labels, declarations and the preprocessor.
//!
//! [`cin_islabel`] decides whether the current line is a jump label -- which
//! 'cinoptions' `L` moves to the left margin -- and has to look *backwards*
//! to do it, because `foo:` is only a label if the statement before it ended.
//! [`cin_isfuncdecl`] is the K&R-parameter test, [`cin_isinit`] the
//! `= {`/`enum` one, and [`cin_ispreproc_cont`] walks a `\`-continued
//! `#define` back to its first line so that the scan does not stop inside
//! one.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use core::ffi::{CStr, c_char, c_int};

/// Step `s` over `label:`, answering whether there was one.
///
/// `::` is C++ scope resolution rather than a label, and the walk is by
/// *character* rather than by byte, because an identifier may be multibyte.
///
/// # Safety
/// `*s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_islabel_skip(s: &mut *const c_char) -> bool {
    unsafe {
        if !vim_is_ident_char(c_int::from(**s as u8)) {
            return false; // need at least one ID character
        }
        while vim_is_ident_char(c_int::from(**s as u8)) {
            *s = (*s).offset(utfc_ptr2len(*s) as isize);
        }
        *s = cin_skipcomment(*s);
        if **s as u8 != b':' {
            return false;
        }
        *s = (*s).add(1);
        **s as u8 != b':'
    }
}

/// Whether the cursor's line is a jump label (`foo:`).
///
/// A label only counts if the *previous* statement ended -- otherwise `foo:`
/// is a ternary's second half or a bit-field width -- so this walks back past
/// comments, raw strings and `#` directives until it finds a line it can
/// judge.  `default:` and a 'cinscopedecls' word are excluded: they indent
/// like switch labels, not like jump labels.
///
/// # Safety
/// Reads and restores the cursor; may unlock the current line.
pub(crate) unsafe fn cin_islabel() -> bool {
    unsafe {
        let mut s = cin_skipcomment(get_cursor_line_ptr());
        if cin_isdefault(s) || cin_isscopedecl(s) || !cin_islabel_skip(&mut s) {
            return false;
        }
        if !ind_find_start_comment_or_raw_string(None).is_null() {
            return false; // not a label in a comment or a raw string
        }

        let cursor_save = (*curwin.get()).w_cursor;
        while (*curwin.get()).w_cursor.lnum > 1 {
            (*curwin.get()).w_cursor.lnum -= 1;
            (*curwin.get()).w_cursor.col = 0;
            let trypos = ind_find_start_comment_or_raw_string(None);
            if !trypos.is_null() {
                (*curwin.get()).w_cursor = *trypos;
            }

            let mut line = get_cursor_line_ptr().cast_const();
            if cin_ispreproc(line) {
                continue; // ignore #defines, #if, etc.
            }
            line = cin_skipcomment(line);
            if *line == 0 {
                continue;
            }

            (*curwin.get()).w_cursor = cursor_save;
            return cin_isterminated(line, true, false) != 0
                || cin_isscopedecl(line)
                || cin_iscase(line, true)
                || (cin_islabel_skip(&mut line) && cin_nocode(line));
        }
        (*curwin.get()).w_cursor = cursor_save;
        true // label at start of file???
    }
}

/// Whether `s` is a structure or compound-literal initialisation:
/// `=`/`return` then `[&]`, an optional typecast, then any number of `{`.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_is_compound_init(s: *const c_char) -> bool {
    unsafe {
        // Find the *last* `=` or `return` on the line: the initialiser is
        // whatever follows it.
        let mut p = s;
        let mut r = ::core::ptr::null::<c_char>();
        while *p != 0 {
            if *p as u8 == b'=' {
                p = cin_skipcomment(p.add(1));
                r = p;
            } else if CStr::from_ptr(p).to_bytes().starts_with(b"return")
                && !vim_is_ident_char(c_int::from(*p.add(6) as u8))
                && (p == s || !vim_is_ident_char(c_int::from(*p.sub(1) as u8)))
            {
                p = cin_skipcomment(p.add(6));
                r = p;
            } else {
                p = cin_skip_comment_and_string(p.add(1));
            }
        }
        if r.is_null() {
            return false;
        }

        let mut p = r; // now just after the '=' or the "return"
        if cin_nocode(p) {
            return true;
        }
        if *p as u8 == b'&' {
            p = cin_skipcomment(p.add(1));
        }
        if *p as u8 == b'(' {
            // Skip a typecast.
            let mut open_count = 1i32;
            while open_count != 0 {
                p = cin_skip_comment_and_string(p.add(1));
                if cin_nocode(p) {
                    return true;
                }
                open_count += i32::from(*p as u8 == b'(') - i32::from(*p as u8 == b')');
            }
            p = cin_skipcomment(p.add(1));
            if cin_nocode(p) {
                return true;
            }
        }
        while *p as u8 == b'{' {
            p = cin_skipcomment(p.add(1));
        }
        cin_nocode(p)
    }
}

/// Whether the cursor's line is an enumeration or a structure
/// initialisation: `[typedef] [static|public|protected|private] enum`, or
/// anything [`cin_is_compound_init`] accepts.
///
/// # Safety
/// Reads the cursor; may unlock the current line.
pub(crate) unsafe fn cin_isinit() -> bool {
    /// Storage-class and access words that may precede the `enum`.
    const SKIP: [&[u8]; 4] = [b"static", b"public", b"protected", b"private"];

    unsafe {
        let mut s = cin_skipcomment(get_cursor_line_ptr());
        if cin_starts_with(s, b"typedef") {
            s = cin_skipcomment(s.add(7));
        }
        while let Some(word) = SKIP.iter().find(|word| cin_starts_with(s, word)) {
            s = cin_skipcomment(s.add(word.len()));
        }
        cin_starts_with(s, b"enum") || cin_is_compound_init(s)
    }
}

/// Whether `s` is a preprocessor directive: anything starting with `#`.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_ispreproc(s: *const c_char) -> bool {
    unsafe { *skipwhite(s) as u8 == b'#' }
}

/// Whether line `*lnump` is a preprocessor directive *or a `\`-continuation
/// of one*, walking `*lnump`/`*pp` back to the line that started it.
///
/// `*amount` is only written when the answer is yes, and then it is the
/// indent of the *continued* line rather than of the directive -- so a scan
/// that skips over a `#define` keeps the amount it would have used.
///
/// # Safety
/// `*pp` must point at a NUL-terminated line; may unlock the current line.
pub(crate) unsafe fn cin_ispreproc_cont(
    pp: &mut *const c_char,
    lnump: &mut linenr_T,
    amount: &mut c_int,
) -> bool {
    unsafe {
        let mut line = *pp;
        let mut lnum = *lnump;
        let mut retval = false;
        let mut candidate_amount = *amount;

        if *line != 0 && *line.add(strlen(line) - 1) as u8 == b'\\' {
            candidate_amount = get_indent_lnum(lnum);
        }

        loop {
            if cin_ispreproc(line) {
                retval = true;
                *lnump = lnum;
                break;
            }
            if lnum == 1 {
                break;
            }
            lnum -= 1;
            line = ml_get(lnum);
            if *line == 0 || *line.add(strlen(line) - 1) as u8 != b'\\' {
                break;
            }
        }

        if lnum != *lnump {
            *pp = ml_get(*lnump);
        }
        if retval {
            *amount = candidate_amount;
        }
        retval
    }
}

/// Whether the line at `first_lnum` looks like a function declaration: an
/// open paren somewhere, a close paren at the end of the line, and no
/// semicolon in between.
///
/// A line ending in `,` continues into the next one, which is why this can
/// read further down the buffer.  `min_lnum` bounds how far *back* the
/// matching `(` may be, and `sp`, when given, both supplies the first line
/// and is restored to it before returning.
///
/// # Safety
/// `*sp` must point at a NUL-terminated line; reads and restores the cursor
/// line number, and may unlock the current line.
pub(crate) unsafe fn cin_isfuncdecl(
    sp: Option<&mut *const c_char>,
    first_lnum: linenr_T,
    min_lnum: linenr_T,
) -> bool {
    unsafe {
        let mut lnum = first_lnum;
        let save_lnum = (*curwin.get()).w_cursor.lnum;
        let mut retval = false;
        let mut just_started = true;

        let mut s = match &sp {
            Some(p) => **p,
            None => ml_get(lnum),
        };

        // Position on the rightmost unmatched paren so that matching it
        // takes us to the line the declaration starts on.
        (*curwin.get()).w_cursor.lnum = lnum;
        if find_last_paren(s, b'(', b')') {
            let trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
            if !trypos.is_null() {
                lnum = (*trypos).lnum;
                if lnum < min_lnum {
                    (*curwin.get()).w_cursor.lnum = save_lnum;
                    return false;
                }
                s = ml_get(lnum);
            }
        }
        (*curwin.get()).w_cursor.lnum = save_lnum;

        if cin_ispreproc(s) {
            return false; // ignore a line starting with #
        }

        while *s != 0
            && *s as u8 != b'('
            && *s as u8 != b';'
            && *s as u8 != b'\''
            && *s as u8 != b'"'
        {
            if cin_iscomment(s) {
                s = cin_skipcomment(s);
            } else if *s as u8 == b':' {
                if *s.add(1) as u8 != b':' {
                    // A constructor's initialiser list is not a declaration:
                    //     A::A(int a, int b)
                    //         : a(0)  // <-- not a function decl
                    //         , b(0)
                    return false;
                }
                s = s.add(2);
            } else {
                s = s.add(1);
            }
        }
        if *s as u8 != b'(' {
            return false; // ';', ' or " before any () or no '('
        }

        'done: {
            while *s != 0 && *s as u8 != b';' && *s as u8 != b'\'' && *s as u8 != b'"' {
                if *s as u8 == b')' && cin_nocode(s.add(1)) {
                    // ')' at the end: a match, unless the line before the
                    // one we started on ends in a backslash --
                    //     #if defined(x) && \
                    //         defined(y)
                    lnum = first_lnum - 1;
                    s = ml_get(lnum);
                    retval = *s == 0 || *s.add(strlen(s) - 1) as u8 != b'\\';
                    break 'done;
                }
                if (*s as u8 == b',' && cin_nocode(s.add(1))) || *s.add(1) == 0 || cin_nocode(s) {
                    let comma = *s as u8 == b',';

                    // A ',' at the end continues into the next line; so does
                    // the end of the line, for this style:
                    //     func(arg1
                    //           , arg2)
                    while lnum < (*curbuf.get()).b_ml.ml_line_count {
                        lnum += 1;
                        s = ml_get(lnum);
                        if !cin_ispreproc(s) {
                            break;
                        }
                    }
                    if lnum >= (*curbuf.get()).b_ml.ml_line_count {
                        break;
                    }
                    // Require a comma at the end of this line, or a comma or
                    // ')' at the start of the next.
                    s = skipwhite(s);
                    if !just_started && !comma && *s as u8 != b',' && *s as u8 != b')' {
                        break;
                    }
                    just_started = false;
                } else if cin_iscomment(s) {
                    s = cin_skipcomment(s);
                } else {
                    s = s.add(1);
                    just_started = false;
                }
            }
        }

        if lnum != first_lnum
            && let Some(p) = sp
        {
            *p = ml_get(first_lnum);
        }
        retval
    }
}
