//! Guessing the new line's indent from the old one: 'autoindent' and
//! 'smartindent'.
//!
//! Everything here runs *before* the line is opened and only ever computes a
//! column, but it does it by moving the cursor around -- `get_indent` reads
//! the cursor line and `findmatch` searches from the cursor -- so the caller
//! saves and restores `w_cursor` around the whole block.
//!
//! Besides the answer, this sets three of 'smartindent''s globals:
//! `did_si` ("indent the new line one level further"), `can_si_back` ("a `{`
//! typed on the new line may un-indent it") and, through the `no_si` half of
//! the answer, whether `did_si` should be cleared again once the indent has
//! been applied.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::memory::XString;
use core::ffi::{c_char, c_int};

use crate::change::*;
use crate::types::NUL;
use crate::winlayer::{Buf, Win};

/// The byte at `at` of `text`, or NUL at and past its end.
///
/// Every walk below tests the byte it is on before stepping, so the offset
/// stays inside the line; the one place it can run off is the end, which is
/// what upstream's terminating NUL answered and what this answers too.
fn byte(text: &[u8], at: usize) -> c_int {
    c_int::from(cstr::byte_at(text, at))
}

/// The indent of the line the cursor is on.
fn indent_here() -> c_int {
    // SAFETY: the cursor is on a valid line of the current buffer.
    get_indent()
}

/// Line `lnum` of the current buffer, copied.
///
/// A copy, not a borrow: every walk below hands control back to the editor --
/// `findmatch` searches, `get_indent` reads another line -- while it is still
/// looking at the text.
fn line_copy(lnum: LineNr) -> XString {
    XString::from_bytes(Buf::current().lines().line(lnum))
}

/// `findmatch` from the cursor, for `initc`.
fn find_match(initc: c_int) -> Option<Pos> {
    // SAFETY: the cursor is on a valid line of the current buffer.
    findmatch(None, initc)
}

/// [`get_leader_len`] when only the length is wanted.
fn leader_len_of(text: &XString) -> c_int {
    // SAFETY: an owned NUL-terminated line, and no flags are asked for.
    unsafe {
        get_leader_len(
            text.as_ptr().cast_mut(),
            ::core::ptr::null_mut(),
            false,
            true,
        )
    }
}

/// The offset of the first non-white byte of `text`.
fn skip_white_at(text: &[u8]) -> usize {
    text.iter()
        .position(|&b| !ascii_iswhite(c_int::from(b)))
        .unwrap_or(text.len())
}

/// Where the C comment that ends on this line began, as an indent.
///
/// Walks forward from the start of a `/*`-style leader looking for the `*/`
/// that closes it, and if `findmatch` can pair it up, answers the indent of
/// the line the comment *started* on. `None` leaves the indent alone.
///
/// ```text
///     /*
///      * A comment.
///      */
///     #define IN_THE_WAY
///     This should line up here;
/// ```
///
fn indent_of_comment_start(line: &XString) -> Option<c_int> {
    // Owned, because `findmatch` below can leave the walk on another line.
    let mut text = XString::from_bytes(line);
    let mut at = skip_white_at(&text);
    if byte(&text, at) == '/' as c_int && byte(&text, at + 1) == '*' as c_int {
        at += 1;
    }
    if byte(&text, at) != '*' as c_int {
        return None;
    }
    at += 1;
    while byte(&text, at) != 0 {
        if byte(&text, at) == '/' as c_int && byte(&text, at - 1) == '*' as c_int {
            // End of a C comment: line the indent up with the line
            // holding the start of it.
            Win::current().w_cursor.col = ColNr::try_from(at).unwrap_or(ColNr::MAX);
            if let Some(pos) = find_match(NUL) {
                Win::current().w_cursor.lnum = pos.lnum;
                return Some(indent_here());
            }
            // findmatch may have left us on another line; fetch it again.
            text = line_copy(Win::current().w_cursor.lnum);
            at = usize::try_from(Win::current().w_cursor.col).unwrap_or(0);
        }
        at += 1;
    }
    None
}

/// 'smartindent' looking *down* the file, for `o` and `<CR>`.
///
/// Answers the new indent and whether `did_si` was set by a `{` -- which the
/// caller has to undo after applying the indent, so that typing `{` on the
/// new line does not un-indent it a second time.
///
/// The caller must restore `w_cursor` afterwards.
fn smart_indent_forward(
    line: &XString,
    flags: c_int,
    lead_len: c_int,
    mut newindent: c_int,
) -> (c_int, bool) {
    // Owned: the walk moves to other lines and calls back into the editor.
    let mut text = XString::from_bytes(line);
    // Skip preprocessor directives, unless they are comments.
    if lead_len == 0 && byte(&text, 0) == '#' as c_int {
        while byte(&text, 0) == '#' as c_int && Win::current().w_cursor.lnum > 1 {
            Win::current().w_cursor.lnum -= 1;
            text = line_copy(Win::current().w_cursor.lnum);
        }
        newindent = indent_here();
    }
    // Re-measure: the `#` walk above may have landed on another line.
    let lead_len = if flags & OPENLINE_DO_COM != 0 {
        leader_len_of(&text)
    } else {
        0
    };

    if lead_len > 0 {
        if let Some(indent) = indent_of_comment_start(&text) {
            newindent = indent;
        }
        return (newindent, false);
    }

    // Not a comment line: look at what the line ends with.
    //
    // On an *empty* line upstream forms `ptr - 1` and reads it, which is
    // out of bounds -- the `#` walk above can land on one, so it is
    // reachable (O-B15-20), and what it answers is whatever the allocator
    // left in front of the copy. There is no last character there, so this
    // answers NUL, which is none of the three bytes tested below.
    let mut at = text.len().saturating_sub(1);
    while at > 0 && ascii_iswhite(byte(&text, at)) {
        at -= 1;
    }
    let last_char = byte(&text, at);

    // Step back over the `{` or `;` to whatever came before it.
    if last_char == '{' as c_int || last_char == ';' as c_int {
        at = at.saturating_sub(1);
        while at > 0 && ascii_iswhite(byte(&text, at)) {
            at -= 1;
        }
    }

    // A statement split over several lines lines up with the line the
    // condition started on:
    //     if (condition &&
    //             condition) {
    //         Should line up here!
    //     }
    if byte(&text, at) == ')' as c_int {
        Win::current().w_cursor.col = ColNr::try_from(at).unwrap_or(ColNr::MAX);
        if let Some(pos) = find_match('(' as c_int) {
            Win::current().w_cursor.lnum = pos.lnum;
            newindent = indent_here();
            text = line_copy(Win::current().w_cursor.lnum);
        }
    }

    let mut no_si = false;
    if last_char == '{' as c_int {
        // A trailing `{` indents, with no need to look for an `if`.
        did_si.set(true);
        no_si = true; // ... and typing `{` must not un-indent it again
    } else if last_char != ';' as c_int && last_char != '}' as c_int && starts_with_cinword(&text) {
        // One of 'cinwords', and the line before did not finish a
        // statement.
        did_si.set(true);
    }
    (newindent, no_si)
}

/// 'smartindent' looking *up* the file, for `O`.
///
/// The caller must restore `w_cursor` afterwards.
fn smart_indent_backward(line: &XString, lead_len: c_int, mut newindent: c_int) -> c_int {
    // Owned, as [`smart_indent_forward`].
    let mut text = XString::from_bytes(line);
    // Skip preprocessor directives, unless they are comments. A `\`
    // continuation carries the directive onto the next line.
    if lead_len == 0 && byte(&text, 0) == '#' as c_int {
        let mut was_backslashed = false;
        while (byte(&text, 0) == '#' as c_int || was_backslashed)
            && Win::current().w_cursor.lnum < Buf::current().b_ml.ml_line_count
        {
            was_backslashed = text.last() == Some(&b'\\');
            Win::current().w_cursor.lnum += 1;
            text = line_copy(Win::current().w_cursor.lnum);
        }
        newindent = if was_backslashed {
            0 // ran off the end of the file
        } else {
            indent_here()
        };
    }

    if byte(&text, skip_white_at(&text)) == '}' as c_int {
        did_si.set(true); // a line starting with `}` indents
    } else {
        can_si_back.set(true); // a `{` typed next can delete the indent
    }
    newindent
}

/// The whole 'smartindent' guess, for either direction.
///
/// Answers the new indent and the `no_si` flag (see [`smart_indent_forward`]).
/// The cursor is saved and restored around it.
///
/// # Safety
/// `saved_line` must be a NUL-terminated copy of the cursor line.
pub(crate) unsafe fn smart_indent(
    dir: c_int,
    flags: c_int,
    saved_line: *mut c_char,
    newindent: c_int,
) -> (c_int, bool) {
    let old_cursor = Win::current().w_cursor;
    // SAFETY: the caller's NUL-terminated copy of the cursor line.
    let line = XString::from_bytes(unsafe { cstr::bytes_at(saved_line) });
    let lead_len = if flags & OPENLINE_DO_COM != 0 {
        leader_len_of(&line)
    } else {
        0
    };
    let answer = if dir == FORWARD {
        smart_indent_forward(&line, flags, lead_len, newindent)
    } else {
        (smart_indent_backward(&line, lead_len, newindent), false)
    };
    Win::current().w_cursor = old_cursor;
    answer
}
