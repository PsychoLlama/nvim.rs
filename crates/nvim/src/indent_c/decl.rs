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
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int};

/// Step `s` over `label:`, answering whether there was one.
///
/// `::` is C++ scope resolution rather than a label, and the walk is by
/// *character* rather than by byte, because an identifier may be multibyte.
///
/// # Safety
/// `*s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_islabel_skip(s: &mut *const c_char) -> bool {
    // SAFETY: the caller's promise, so reading `**s` is in bounds;
    // `vim_is_ident_char` reads the 'isident' table, set up long before any
    // buffer is indented.
    if !unsafe { vim_is_ident_char(c_int::from(**s as u8)) } {
        return false; // need at least one ID character
    }
    // SAFETY: `*s` walks the same NUL-terminated string: `utfc_ptr2len`
    // answers the length of the character it points at -- never past the NUL,
    // which is not an ID character -- and `cin_skipcomment` a pointer into
    // that string.  The `:` test in front of the `add(1)` is what says the
    // byte behind it is there too.
    while unsafe { vim_is_ident_char(c_int::from(**s as u8)) } {
        *s = unsafe { (*s).offset(utfc_ptr2len(*s) as isize) };
    }
    *s = unsafe { cin_skipcomment(*s) };
    if unsafe { **s } as u8 != b':' {
        return false;
    }
    *s = unsafe { (*s).add(1) };
    unsafe { **s as u8 != b':' }
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
    // SAFETY: on the main thread with a current buffer, so
    // `get_cursor_line_ptr` hands back a NUL-terminated line and
    // `cin_skipcomment` a pointer into it.  The chain is left whole:
    // `cin_islabel_skip` only steps over a line the two tests in front of it
    // did not claim.
    let is_label = unsafe {
        let mut s = cin_skipcomment(get_cursor_line_ptr());
        !cin_isdefault(s) && !cin_isscopedecl(s) && cin_islabel_skip(&mut s)
    };
    if !is_label {
        return false;
    }
    // SAFETY: the cursor is on a line of the current buffer.
    if unsafe { ind_find_start_comment_or_raw_string(None) }.is_some() {
        return false; // not a label in a comment or a raw string
    }

    let cursor_save = cur_win().w_cursor;
    while cur_win().w_cursor.lnum > 1 {
        cur_win().w_cursor.lnum -= 1;
        cur_win().w_cursor.col = 0;
        // SAFETY: the cursor is on a line of the current buffer.
        if let Some(trypos) = unsafe { ind_find_start_comment_or_raw_string(None) } {
            cur_win().w_cursor = trypos;
        }

        // SAFETY: the cursor is on a line of the current buffer, so
        // `get_cursor_line_ptr` hands back that line, NUL-terminated, and
        // `cin_skipcomment` answers a pointer into it.
        let line = unsafe {
            let line = get_cursor_line_ptr().cast_const();
            (!cin_ispreproc(line)).then(|| cin_skipcomment(line))
        };
        // Ignore #defines, #if, etc., and lines with nothing on them.
        // SAFETY: `line`, when there is one, points into the cursor's line.
        let Some(mut line) = line.filter(|&l| unsafe { *l } != 0) else {
            continue;
        };

        cur_win().w_cursor = cursor_save;
        // SAFETY: `line` is a NUL-terminated line of the current buffer, and
        // the chain is left whole so `cin_nocode` only sees where
        // `cin_islabel_skip` left `line` when it found a label.
        return unsafe {
            cin_isterminated(line, true, false) != 0
                || cin_isscopedecl(line)
                || cin_iscase(line, true)
                || (cin_islabel_skip(&mut line) && cin_nocode(line))
        };
    }
    cur_win().w_cursor = cursor_save;
    true // label at start of file???
}

/// Whether `s` is a structure or compound-literal initialisation:
/// `=`/`return` then `[&]`, an optional typecast, then any number of `{`.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_is_compound_init(s: *const c_char) -> bool {
    // Find the *last* `=` or `return` on the line: the initialiser is
    // whatever follows it.
    //
    // SAFETY: the caller's promise -- `p` walks the NUL-terminated `s`, and
    // both skips answer a pointer into it.  `add(6)` is inside because the
    // `starts_with(b"return")` in front of it matched six bytes, and `sub(1)`
    // runs only when `p` is past `s`; the `&&` chain is left whole so that it
    // keeps doing so.
    let r = unsafe {
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
        r
    };
    if r.is_null() {
        return false;
    }

    // SAFETY: `r` points into the same NUL-terminated string, so the walk
    // below stops at its NUL; each `add(1)` steps over a byte the test in
    // front of it has just read, and `cin_nocode` reads no further than the
    // NUL either.
    let mut p = r; // now just after the '=' or the "return"
    if unsafe { cin_nocode(p) } {
        return true;
    }
    if unsafe { *p } as u8 == b'&' {
        p = unsafe { cin_skipcomment(p.add(1)) };
    }
    if unsafe { *p } as u8 == b'(' {
        // Skip a typecast.
        let mut open_count = 1i32;
        while open_count != 0 {
            p = unsafe { cin_skip_comment_and_string(p.add(1)) };
            if unsafe { cin_nocode(p) } {
                return true;
            }
            open_count +=
                i32::from(unsafe { *p } as u8 == b'(') - i32::from(unsafe { *p } as u8 == b')');
        }
        p = unsafe { cin_skipcomment(p.add(1)) };
        if unsafe { cin_nocode(p) } {
            return true;
        }
    }
    while unsafe { *p } as u8 == b'{' {
        p = unsafe { cin_skipcomment(p.add(1)) };
    }
    unsafe { cin_nocode(p) }
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

    // SAFETY: on the main thread with a current buffer, so
    // `get_cursor_line_ptr` hands back a NUL-terminated line and
    // `cin_skipcomment` a pointer into it.
    let mut s = unsafe { cin_skipcomment(get_cursor_line_ptr()) };
    // SAFETY: each `add` steps over a word `cin_starts_with` has just matched
    // on that same line, so `s` never leaves it.
    if unsafe { cin_starts_with(s, b"typedef") } {
        s = unsafe { cin_skipcomment(s.add(7)) };
    }
    while let Some(word) = SKIP.iter().find(|word| unsafe { cin_starts_with(s, word) }) {
        s = unsafe { cin_skipcomment(s.add(word.len())) };
    }
    let is_enum = unsafe { cin_starts_with(s, b"enum") };
    is_enum || unsafe { cin_is_compound_init(s) }
}

/// Whether `s` is a preprocessor directive: anything starting with `#`.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_ispreproc(s: *const c_char) -> bool {
    // SAFETY: the caller's promise, so `skipwhite` stops at the NUL at the
    // latest and its answer is inside the string.
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
    let mut line = *pp;
    let mut lnum = *lnump;
    let mut retval = false;
    let mut candidate_amount = *amount;

    // SAFETY: the caller's promise -- `*pp` is a NUL-terminated line.
    if unsafe { cin_ends_in_backslash(line) } {
        // SAFETY: `lnum` is the line `*pp` came from, so it is a line of the
        // current buffer.
        candidate_amount = unsafe { get_indent_lnum(lnum) };
    }

    loop {
        // SAFETY: `line` is a NUL-terminated line of the current buffer.
        if unsafe { cin_ispreproc(line) } {
            retval = true;
            *lnump = lnum;
            break;
        }
        if lnum == 1 {
            break;
        }
        lnum -= 1;
        // SAFETY: `lnum` is at least 1 and no larger than the line it started
        // on, so it is a line of the buffer; `ml_get` hands back a
        // NUL-terminated one.
        line = ml_get(lnum);
        // SAFETY: the line `ml_get` just answered with.
        if !unsafe { cin_ends_in_backslash(line) } {
            break;
        }
    }

    if lnum != *lnump {
        // SAFETY: `*lnump` is a line of the current buffer.
        *pp = ml_get(*lnump);
    }
    if retval {
        *amount = candidate_amount;
    }
    retval
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
    let mut lnum = first_lnum;
    let save_lnum = cur_win().w_cursor.lnum;
    let mut retval = false;
    let mut just_started = true;

    let mut s = match &sp {
        Some(p) => **p,
        // SAFETY: on the main thread with a current buffer; `ml_get` reports
        // a line number of its own that is out of range, and hands back a
        // NUL-terminated line.
        None => ml_get(lnum),
    };

    // Position on the rightmost unmatched paren so that matching it
    // takes us to the line the declaration starts on.
    cur_win().w_cursor.lnum = lnum;
    // SAFETY: `s` is a NUL-terminated line; both searches run on the current
    // buffer from the cursor, and `find_match_paren` runs only when
    // `find_last_paren` found one, as upstream has it.
    let opening = unsafe {
        find_last_paren(s, b'(', b')')
            .then(|| find_match_paren(cur_buf().b_ind_maxparen))
            .flatten()
    };
    if let Some(trypos) = opening {
        lnum = trypos.lnum;
        if lnum < min_lnum {
            cur_win().w_cursor.lnum = save_lnum;
            return false;
        }
        // SAFETY: `lnum` is the line the match was found on.
        s = ml_get(lnum);
    }
    cur_win().w_cursor.lnum = save_lnum;

    // SAFETY: `s` is a NUL-terminated line.
    if unsafe { cin_ispreproc(s) } {
        return false; // ignore a line starting with #
    }

    // SAFETY: `s` walks that same line and stops at its NUL;
    // `cin_skipcomment` answers a pointer into it, and `add(1)`/`add(2)` step
    // over bytes the tests in front of them have just read.
    while unsafe { *s } != 0
        && unsafe { *s } as u8 != b'('
        && unsafe { *s } as u8 != b';'
        && unsafe { *s } as u8 != b'\''
        && unsafe { *s } as u8 != b'"'
    {
        if unsafe { cin_iscomment(s) } {
            s = unsafe { cin_skipcomment(s) };
        } else if unsafe { *s } as u8 == b':' {
            if unsafe { *s.add(1) } as u8 != b':' {
                // A constructor's initialiser list is not a declaration:
                //     A::A(int a, int b)
                //         : a(0)  // <-- not a function decl
                //         , b(0)
                return false;
            }
            s = unsafe { s.add(2) };
        } else {
            s = unsafe { s.add(1) };
        }
    }
    // SAFETY: `s` is inside that line.
    if unsafe { *s } as u8 != b'(' {
        return false; // ';', ' or " before any () or no '('
    }

    'done: {
        loop {
            // SAFETY: `s` is inside a NUL-terminated line of the current
            // buffer, so reading its byte is in bounds.
            let c = unsafe { *s } as u8;
            if c == 0 || c == b';' || c == b'\'' || c == b'"' {
                break;
            }
            // SAFETY: `s` is inside that line, so `add(1)` is at worst its
            // NUL, which `cin_nocode` only reads.
            if c == b')' && unsafe { cin_nocode(s.add(1)) } {
                // ')' at the end: a match, unless the line before the
                // one we started on ends in a backslash --
                //     #if defined(x) && \
                //         defined(y)
                lnum = first_lnum - 1;
                // SAFETY: on the main thread with a current buffer; `ml_get`
                // reports a line number of its own that is out of range, and
                // hands back a NUL-terminated line.
                retval = !unsafe { cin_ends_in_backslash(ml_get(lnum)) };
                break 'done;
            }
            // SAFETY: the same, and the chain is left whole so that the
            // tests stay in the order upstream asks them in.
            let continues =
                unsafe { (c == b',' && cin_nocode(s.add(1))) || *s.add(1) == 0 || cin_nocode(s) };
            if continues {
                let comma = c == b',';

                // A ',' at the end continues into the next line; so does
                // the end of the line, for this style:
                //     func(arg1
                //           , arg2)
                while lnum < cur_buf().b_ml.ml_line_count {
                    lnum += 1;
                    // SAFETY: `lnum` is a line of the current buffer.
                    s = ml_get(lnum);
                    // SAFETY: `s` is the NUL-terminated line it answered.
                    if !unsafe { cin_ispreproc(s) } {
                        break;
                    }
                }
                if lnum >= cur_buf().b_ml.ml_line_count {
                    break;
                }
                // Require a comma at the end of this line, or a comma or
                // ')' at the start of the next.
                // SAFETY: `s` is a NUL-terminated line, so `skipwhite` stops
                // inside it.
                s = unsafe { skipwhite(s) };
                // SAFETY: `s` is inside that line.
                let next = unsafe { *s } as u8;
                if !just_started && !comma && next != b',' && next != b')' {
                    break;
                }
                just_started = false;
                continue;
            }
            // SAFETY: `s` is inside a NUL-terminated line, and
            // `cin_skipcomment` answers a pointer into it.
            if unsafe { cin_iscomment(s) } {
                // SAFETY: the same.
                s = unsafe { cin_skipcomment(s) };
            } else {
                // SAFETY: the byte at `s` is not the NUL -- the top of the
                // loop broke out on that -- so `add(1)` stays inside.
                s = unsafe { s.add(1) };
                just_started = false;
            }
        }
    }

    if lnum != first_lnum
        && let Some(p) = sp
    {
        // SAFETY: `first_lnum` is the line the caller named; `ml_get` reports
        // a line number of its own that is out of range.
        *p = ml_get(first_lnum);
    }
    retval
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
