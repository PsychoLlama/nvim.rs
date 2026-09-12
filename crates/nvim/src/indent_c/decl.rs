//! Labels, declarations and the preprocessor.
//!
//! [`cin_islabel`] decides whether the current line is a jump label -- which
//! 'cinoptions' `L` moves to the left margin -- and has to look *backwards*
//! to do it, because `foo:` is only a label if the statement before it ended.
//! [`cin_isfuncdecl`] is the K&R-parameter test, [`cin_isinit`] the
//! `= {`/`enum` one, and [`cin_ispreproc_cont`] walks a `\`-continued
//! `#define` back to its first line so that the scan does not stop inside
//! one.
//!
//! | C | here |
//! | --- | --- |
//! | `cin_islabel_skip` | [`past_label`] |
//! | `cin_is_compound_init` | [`is_compound_init`] |
//! | `cin_ispreproc` | [`is_preproc`] |

#![forbid(unsafe_code)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

/// The offset past a `label:` at `line[at..]`, or `None` when there is not
/// one there.
///
/// `::` is C++ scope resolution rather than a label, and the walk is by
/// *character* rather than by byte, because an identifier may be multibyte.
pub(crate) fn past_label(line: &[u8], at: usize) -> Option<usize> {
    let mut i = at;
    if !vim_is_ident_char(c_int::from(byte_at(line, i))) {
        return None; // need at least one ID character
    }
    while vim_is_ident_char(c_int::from(byte_at(line, i))) {
        i += cluster_len(&line[i.min(line.len())..]);
    }
    i = code_at(line, i);
    if byte_at(line, i) != b':' {
        return None;
    }
    (byte_at(line, i + 1) != b':').then_some(i + 1)
}

/// Whether the cursor's line is a jump label (`foo:`).
///
/// A label only counts if the *previous* statement ended -- otherwise `foo:`
/// is a ternary's second half or a bit-field width -- so this walks back past
/// comments, raw strings and `#` directives until it finds a line it can
/// judge.  `default:` and a 'cinscopedecls' word are excluded: they indent
/// like switch labels, not like jump labels.
pub(crate) fn is_jump_label() -> bool {
    let is_label = {
        let mut lines = Lines::current();
        let line = lines.line(Win::current().w_cursor.lnum);
        let at = code_at(line, 0);
        // The chain is left whole: `past_label` only steps over a line the
        // two tests in front of it did not claim.
        !is_default_label(line, at) && !is_scope_decl(line, at) && past_label(line, at).is_some()
    };
    if !is_label {
        return false;
    }
    if ind_find_start_comment_or_raw_string(None).is_some() {
        return false; // not a label in a comment or a raw string
    }

    let cursor_save = Win::current().w_cursor;
    while Win::current().w_cursor.lnum > 1 {
        Win::current().w_cursor.lnum -= 1;
        Win::current().w_cursor.col = 0;
        if let Some(trypos) = ind_find_start_comment_or_raw_string(None) {
            Win::current().w_cursor = trypos;
        }

        // Ignore #defines, #if, etc., and lines with nothing on them.  The
        // borrow ends with the verdict: the cursor is restored below, and
        // nothing after this reads the text.
        let verdict = {
            let mut lines = Lines::current();
            let line = lines.line(Win::current().w_cursor.lnum);
            let at = code_at(line, 0);
            if is_preproc(line) || at >= line.len() {
                None
            } else {
                Some(
                    terminator(line, at, true, false) != 0
                        || is_scope_decl(line, at)
                        || is_case_label(line, at, true)
                        || past_label(line, at).is_some_and(|end| only_comment_left(line, end)),
                )
            }
        };
        let Some(verdict) = verdict else {
            continue;
        };

        Win::current().w_cursor = cursor_save;
        return verdict;
    }
    Win::current().w_cursor = cursor_save;
    true // label at start of file???
}

/// Whether `line[at..]` is a structure or compound-literal initialisation:
/// `=`/`return` then `[&]`, an optional typecast, then any number of `{`.
pub(crate) fn is_compound_init(line: &[u8], at: usize) -> bool {
    // Find the *last* `=` or `return` on the line: the initialiser is
    // whatever follows it.
    let mut i = at;
    let mut found = None;
    while byte_at(line, i) != 0 {
        if byte_at(line, i) == b'=' {
            i = code_at(line, i + 1);
            found = Some(i);
        } else if line[i.min(line.len())..].starts_with(b"return")
            && !vim_is_ident_char(c_int::from(byte_at(line, i + 6)))
            && (i == at || !vim_is_ident_char(c_int::from(byte_at(line, i - 1))))
        {
            i = code_at(line, i + 6);
            found = Some(i);
        } else {
            i = code_or_string_at(line, i + 1);
        }
    }
    let Some(mut i) = found else {
        return false;
    };

    // `i` is now just after the '=' or the "return".
    if only_comment_left(line, i) {
        return true;
    }
    if byte_at(line, i) == b'&' {
        i = code_at(line, i + 1);
    }
    if byte_at(line, i) == b'(' {
        // Skip a typecast.
        let mut open_count = 1i32;
        while open_count != 0 {
            i = code_or_string_at(line, i + 1);
            if only_comment_left(line, i) {
                return true;
            }
            open_count += i32::from(byte_at(line, i) == b'(') - i32::from(byte_at(line, i) == b')');
        }
        i = code_at(line, i + 1);
        if only_comment_left(line, i) {
            return true;
        }
    }
    while byte_at(line, i) == b'{' {
        i = code_at(line, i + 1);
    }
    only_comment_left(line, i)
}

/// Whether the cursor's line is an enumeration or a structure
/// initialisation: `[typedef] [static|public|protected|private] enum`, or
/// anything [`is_compound_init`] accepts.
pub(crate) fn is_enum_or_init() -> bool {
    /// Storage-class and access words that may precede the `enum`.
    const SKIP: [&[u8]; 4] = [b"static", b"public", b"protected", b"private"];

    let mut lines = Lines::current();
    let line = lines.line(Win::current().w_cursor.lnum);
    let mut at = code_at(line, 0);
    if starts_with_word(line, at, b"typedef") {
        at = code_at(line, at + 7);
    }
    while let Some(word) = SKIP.iter().find(|word| starts_with_word(line, at, word)) {
        at = code_at(line, at + word.len());
    }
    starts_with_word(line, at, b"enum") || is_compound_init(line, at)
}

/// Whether `line` is a preprocessor directive: anything starting with `#`.
pub(crate) fn is_preproc(line: &[u8]) -> bool {
    byte_at(line, skip::white(line)) == b'#'
}

/// Whether line `*lnum` is a preprocessor directive *or a `\`-continuation
/// of one*, walking `*lnum` back to the line that started it.
///
/// `*amount` is only written when the answer is yes, and then it is the
/// indent of the *continued* line rather than of the directive -- so a scan
/// that skips over a `#define` keeps the amount it would have used.
///
/// Upstream also hands the caller its line pointer back, refetched for
/// whichever line `*lnum` ended on; here the caller reads that line itself,
/// which is the same read and one it can see.
pub(crate) fn preproc_start(lnum: &mut LineNr, amount: &mut c_int) -> bool {
    let mut at_lnum = *lnum;
    let mut retval = false;
    let mut candidate_amount = *amount;
    let mut lines = Lines::current();

    if ends_in_backslash(lines.line(at_lnum)) {
        candidate_amount = get_indent_lnum(at_lnum);
    }

    loop {
        if is_preproc(lines.line(at_lnum)) {
            retval = true;
            *lnum = at_lnum;
            break;
        }
        if at_lnum == 1 {
            break;
        }
        at_lnum -= 1;
        if !ends_in_backslash(lines.line(at_lnum)) {
            break;
        }
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
/// matching `(` may be.
///
/// Upstream also takes the caller's line pointer, both as the first line's
/// text -- always `ml_get(first_lnum)` at every call site -- and to hand it
/// back refetched.  Here the line is read from the number and the caller
/// reads it again itself, which is the same pair of reads and one it can see.
pub(crate) fn is_func_decl(first_lnum: LineNr, min_lnum: LineNr) -> bool {
    /// Why the walk stopped on the line it was looking at.
    enum Stopped {
        /// Something that cannot be a declaration: not one.
        NotOne,
        /// A `)` at the end of the line: a match, unless the line above the
        /// one we started on continues into it.
        CloseParen,
        /// The line runs on into the next one; `comma` is whether it ended
        /// with one.
        Continues { comma: bool },
    }

    let mut lnum = first_lnum;
    let save_lnum = Win::current().w_cursor.lnum;
    let mut just_started = true;

    // Position on the rightmost unmatched paren so that matching it
    // takes us to the line the declaration starts on.  The borrow ends
    // with the statement: the match search reads other lines.
    Win::current().w_cursor.lnum = lnum;
    let has_paren = find_last_paren(Lines::current().line(lnum), b'(', b')');
    let opening = if has_paren {
        find_match_paren(Buf::current().b_ind_maxparen)
    } else {
        None
    };
    if let Some(trypos) = opening {
        lnum = trypos.lnum;
        if lnum < min_lnum {
            Win::current().w_cursor.lnum = save_lnum;
            return false;
        }
    }
    Win::current().w_cursor.lnum = save_lnum;

    let mut lines = Lines::current();
    let mut at = 0usize;
    {
        let line = lines.line(lnum);
        if is_preproc(line) {
            return false; // ignore a line starting with #
        }
        while !matches!(byte_at(line, at), 0 | b'(' | b';' | b'\'' | b'"') {
            if starts_comment(line, at) {
                at = code_at(line, at);
            } else if byte_at(line, at) == b':' {
                if byte_at(line, at + 1) != b':' {
                    // A constructor's initialiser list is not a declaration:
                    //     A::A(int a, int b)
                    //         : a(0)  // <-- not a function decl
                    //         , b(0)
                    return false;
                }
                at += 2;
            } else {
                at += 1;
            }
        }
        if byte_at(line, at) != b'(' {
            return false; // ';', ' or " before any () or no '('
        }
    }

    let mut retval = false;
    loop {
        // One line's worth of the walk.  The borrow is the line's, and ends
        // where the walk has to look at another one.
        let stopped = {
            let line = lines.line(lnum);
            loop {
                let c = byte_at(line, at);
                if matches!(c, 0 | b';' | b'\'' | b'"') {
                    break Stopped::NotOne;
                }
                if c == b')' && only_comment_left(line, at + 1) {
                    break Stopped::CloseParen;
                }
                // A ',' at the end continues into the next line; so does the
                // end of the line, for this style:
                //     func(arg1
                //           , arg2)
                if (c == b',' && only_comment_left(line, at + 1))
                    || byte_at(line, at + 1) == 0
                    || only_comment_left(line, at)
                {
                    break Stopped::Continues { comma: c == b',' };
                }
                if starts_comment(line, at) {
                    at = code_at(line, at);
                } else {
                    at += 1;
                    just_started = false;
                }
            }
        };

        match stopped {
            Stopped::NotOne => break,
            Stopped::CloseParen => {
                //     #if defined(x) && \
                //         defined(y)
                // is not a declaration, however it ends.  Line 0 is line 1,
                // as it is for `ml_get`.
                lnum = first_lnum - 1;
                retval = !ends_in_backslash(lines.line(lnum));
                break;
            }
            Stopped::Continues { comma } => {
                while lnum < Buf::current().b_ml.ml_line_count {
                    lnum += 1;
                    if !is_preproc(lines.line(lnum)) {
                        break;
                    }
                }
                if lnum >= Buf::current().b_ml.ml_line_count {
                    break;
                }
                // Require a comma at the end of this line, or a comma or
                // ')' at the start of the next.
                let line = lines.line(lnum);
                at = skip::white(line);
                let next = byte_at(line, at);
                if !just_started && !comma && next != b',' && next != b')' {
                    break;
                }
                just_started = false;
            }
        }
    }
    retval
}
