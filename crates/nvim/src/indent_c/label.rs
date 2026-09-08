//! Indent amounts read off a line rather than computed.
//!
//! [`skip_label`] and [`get_indent_nolabel`] answer "how indented is this
//! line, not counting a jump label in front of it"; [`after_label`] is the
//! offset of the text past one.  [`first_id_amount`] is 'cinoptions' `+`'s
//! continuation base -- the column of the first identifier after a type --
//! and [`equal_amount`] the column after a trailing `=`, which is what a
//! `\`-continued assignment lines up with.
//!
//! | C | here |
//! | --- | --- |
//! | `cin_first_id_amount` | [`first_id_amount`] |
//! | `cin_get_equal_amount` | [`equal_amount`] |

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::winlayer::Win;
use core::ffi::c_int;

/// The first non-white non-comment character after a `:` label in `line`.
///
/// ```text
///    case 234:    a = b;
///                 ^
/// ```
///
/// `::` is C++ scope resolution, and a `:` that opens *another* `case` is not
/// the end of this label.  `None` when there is nothing after one.
pub(crate) fn after_label(line: &[u8]) -> Option<usize> {
    let mut at = 0usize;
    while byte_at(line, at) != 0 {
        if byte_at(line, at) == b':' {
            if byte_at(line, at + 1) == b':' {
                at += 1; // skip over "::" for C++
            } else if !is_case_label(line, at + 1, false) {
                break;
            }
        } else if byte_at(line, at) == b'\''
            && byte_at(line, at + 1) != 0
            && byte_at(line, at + 2) == b'\''
        {
            at += 2; // skip over 'x'
        }
        at += 1;
    }
    if byte_at(line, at) == 0 {
        return None;
    }
    let at = code_at(line, at + 1);
    (byte_at(line, at) != 0).then_some(at)
}

/// The screen column the code *after* a label on line `lnum` starts at, or 0
/// when there is nothing after it.
///
/// # Safety
/// `lnum` must be a valid line; may unlock the current line.
pub(crate) unsafe fn get_indent_nolabel(lnum: LineNr) -> c_int {
    let Some(at) = after_label(Lines::current().line(lnum)) else {
        return 0;
    };
    // SAFETY: `at` is an offset inside line `lnum` of the current buffer.
    unsafe { line_vcol(lnum, at as ColNr) }
}

/// The indent of line `lnum` ignoring any case or jump label, with the offset
/// of the text the amount belongs to.
///
/// ```text
///   label:     if (asdf && asdfasdf)
///              ^
/// ```
///
/// # Safety
/// Moves the cursor and restores it; may unlock the current line.
pub(crate) unsafe fn skip_label(lnum: LineNr) -> (c_int, usize) {
    let cursor_save = Win::current().w_cursor;
    Win::current().w_cursor.lnum = lnum;
    // The chain re-enters at `is_jump_label`, so the borrow the two tests in
    // front of it take is dropped before it runs.
    let labelled = {
        let mut lines = Lines::current();
        let line = lines.line(lnum);
        // SAFETY: `is_scope_decl` reads the buffer's 'cinscopedecls'.
        is_case_label(line, 0, false) || unsafe { is_scope_decl(line, 0) }
        // SAFETY: the cursor sits on `lnum`, a line of the current buffer.
    } || unsafe { is_jump_label() };

    let answer = if labelled {
        // SAFETY: `lnum` is a line of the current buffer.
        let amount = unsafe { get_indent_nolabel(lnum) };
        // Upstream falls back to the start of the line when there is nothing
        // after the label, "just in case".
        (
            amount,
            after_label(Lines::current().line(lnum)).unwrap_or(0),
        )
    } else {
        // SAFETY: reads the cursor's line, which is `lnum`.
        (get_indent(), 0)
    };

    Win::current().w_cursor = cursor_save;
    answer
}

/// The screen column of the first variable name after a type in a
/// declaration -- 'cinoptions' `+`'s base for a continued declaration.
///
/// ```text
///  int     a,                  the column of "a"
///  static struct foo    b,     the column of "b"
///  enum bla    c,              the column of "c"
/// ```
///
/// Zero when the line does not look like a declaration.
///
/// # Safety
/// Reads the cursor; may unlock the current line.
pub(crate) unsafe fn first_id_amount() -> c_int {
    let lnum = Win::current().w_cursor.lnum;
    let mut lines = Lines::current();
    let line = lines.line(lnum);
    let mut at = skip::white(line);

    // Step over the storage class and the type's first word, so that the
    // identifier the answer is about is what `at` ends on.
    let mut len = skip::to_white(&line[at..]);
    if len == 6 && line[at..].starts_with(b"static") {
        at += 6;
        at += skip::white(&line[at..]);
        len = skip::to_white(&line[at..]);
    }
    let word = &line[at..];
    if len == 6 && word.starts_with(b"struct") {
        at += 6;
        at += skip::white(&line[at..]);
    } else if len == 4 && word.starts_with(b"enum") {
        at += 4;
        at += skip::white(&line[at..]);
    } else if (len == 8 && word.starts_with(b"unsigned"))
        || (len == 6 && word.starts_with(b"signed"))
    {
        // `unsigned`/`signed` only prefixes a type; take the type with it.
        let after = at + len + skip::white(&line[at + len..]);
        let rest = &line[after..];
        let takes_type = [&b"int"[..], b"long", b"short", b"char"]
            .into_iter()
            .any(|kw| rest.starts_with(kw) && ascii_iswhite(c_int::from(byte_at(rest, kw.len()))));
        if takes_type {
            at = after;
        }
    }

    let mut len = 0usize;
    while vim_is_ident_char(c_int::from(byte_at(line, at + len))) {
        len += 1;
    }
    if len == 0
        || !ascii_iswhite(c_int::from(byte_at(line, at + len)))
        || only_comment_left(line, at)
    {
        return 0;
    }

    let at = at + len + skip::white(&line[at + len..]);
    // SAFETY: `at` is an offset inside the cursor's line.
    unsafe { line_vcol(lnum, at as ColNr) }
}

/// The screen column of the first non-blank after an `=` on line `lnum`.
///
/// ```text
///       char *foo = "here";
///                    ^
/// ```
///
/// Zero when there is no useful `=`, and **-1** when the line *above* `lnum`
/// ends in a backslash -- the assignment started further up, so this line's
/// `=` is not the one to line up with.
///
/// # Safety
/// `lnum` must be a valid line; may unlock the current line.
pub(crate) unsafe fn equal_amount(lnum: LineNr) -> c_int {
    if lnum > 1 && ends_in_backslash(Lines::current().line(lnum - 1)) {
        return -1;
    }

    let mut lines = Lines::current();
    let line = lines.line(lnum);
    let mut at = 0usize;
    while byte_at(line, at) != 0 && !b"=;{}\"'".contains(&byte_at(line, at)) {
        if starts_comment(line, at) {
            at = code_at(line, at);
        } else {
            at += 1;
        }
    }
    if byte_at(line, at) != b'=' {
        return 0;
    }

    let mut at = at + 1;
    at += skip::white(&line[at.min(line.len())..]);
    if only_comment_left(line, at) {
        return 0;
    }
    if byte_at(line, at) == b'"' {
        at += 1; // nice alignment for continued strings
    }
    // SAFETY: `at` is an offset inside line `lnum`.
    unsafe { line_vcol(lnum, at as ColNr) }
}
