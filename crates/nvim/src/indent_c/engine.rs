//! `get_c_indent` -- the C indent itself.
//!
//! One question, asked of one line: what column should it start at?  The
//! answer comes from the lines *above* it, and which of them matters depends
//! entirely on what encloses the cursor.  That is this file: the dispatch.
//!
//! | enclosing context | who answers | the 'cinoptions' letters |
//! | --- | --- | --- |
//! | a raw string | here -- the indent is left alone | -- |
//! | a `#` directive | here | `#`, `P` |
//! | a jump label | here | `L` |
//! | a comment | [`incomment`](super::incomment) | `c`, `C`, `/`, and 'comments' |
//! | unclosed `(` or `[` | [`inparen`](super::inparen) | `(`, `u`, `U`, `w`, `W`, `m`, `M`, `k`, `)` |
//! | an unclosed `{` | [`inblock`](super::inblock) + [`lookfor`](super::lookfor) | most of the rest |
//! | nothing | [`toplevel`](super::toplevel) | `f`, `t`, `p`, `+`, `i` |
//!
//! Every one of them may move the cursor and unlock the current line; the
//! cursor is put back here, once, on the way out.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use core::ffi::{CStr, c_char, c_int};

/// The line being indented, and where it sits.
///
/// `theline` points into `linecopy`, a private copy of the line: `ml_get`
/// only guarantees the *most recent* line it answered, and this code asks for
/// hundreds of others before it is done.
pub(crate) struct Line {
    /// The cursor's line, white space skipped -- the text being judged.
    pub theline: *const c_char,
    /// The same copy from column 0, which the `#` test reads.
    pub linecopy: *const c_char,
    /// Where the cursor was when the question was asked.
    pub cur_curpos: pos_T,
    /// Whether that line is a jump label, decided before anything moved.
    pub original_line_islabel: bool,
}

impl Line {
    /// Whether the line being indented starts with `c`.
    ///
    /// # Safety
    /// `theline` must still be valid.
    pub(crate) unsafe fn starts_with(&self, c: u8) -> bool {
        unsafe { *self.theline as u8 == c }
    }
}

/// The indent for the cursor's line, or -1 to leave it alone (inside a raw
/// string).
///
/// # Safety
/// Reads and restores the cursor; unlocks the current line freely.
pub unsafe fn get_c_indent() -> c_int {
    unsafe {
        // Remember where the cursor was when we started.
        let cur_curpos = (*curwin.get()).w_cursor;

        // At line 1, zero indent is fine, right?
        if cur_curpos.lnum == 1 {
            return 0;
        }

        // A copy, because only the most recent line `ml_get` answered stays
        // valid and everything below asks for more.
        let linecopy = xstrdup(ml_get(cur_curpos.lnum));

        // In Insert mode with the cursor on a ')', truncate the line there:
        // new text should not line up with the matching '('.  The cursor can
        // be past the end of the line, for unknown reasons, so check.
        let col = (*curwin.get()).w_cursor.col;
        if State.get() & MODE_INSERT != 0
            && (col as size_t) < strlen(linecopy)
            && *linecopy.offset(col as isize) as u8 == b')'
        {
            *linecopy.offset(col as isize) = NUL as c_char;
        }

        let theline = skipwhite(linecopy).cast_const();

        // Move the cursor to the start of the line, and judge the line before
        // anything else moves: 'cinoptions' `L` reads the answer again at the
        // very end.
        (*curwin.get()).w_cursor.col = 0;
        let line = Line {
            theline,
            linecopy: linecopy.cast_const(),
            cur_curpos,
            original_line_islabel: cin_islabel(),
        };

        let amount = match c_indent(&line) {
            // Inside a raw string: leave the indent alone, and do not clamp.
            None => -1,
            Some(amount) => amount.max(0),
        };

        // Put the cursor back where it belongs.
        (*curwin.get()).w_cursor = cur_curpos;
        xfree(linecopy.cast::<::core::ffi::c_void>());
        amount
    }
}

/// The dispatch: which context encloses the line, and what that context says.
///
/// `None` means "inside a raw string, leave the indent alone".
///
/// # Safety
/// Moves the cursor and unlocks the current line; the caller restores it.
unsafe fn c_indent(line: &Line) -> Option<c_int> {
    unsafe {
        // A raw string wins over a comment only if it starts *earlier*; a raw
        // string inside a comment is just comment text.  `findmatchlimit`
        // answers out of one static, so the comment position is copied by
        // value before the second search overwrites it.
        let mut comment_pos = ind_find_start_comment().as_ref().copied();
        let raw_string = find_start_rawstring((*curbuf.get()).b_ind_maxcomment);
        if !raw_string.is_null() && comment_pos.is_none_or(|comment| lt(*raw_string, comment)) {
            return None;
        }

        // `#define` and friends go at the left when 'cinkeys' says so,
        // excluding `#pragma` when 'cinoptions' `P` asks.
        if line.starts_with(b'#')
            && (*line.linecopy as u8 == b'#'
                || in_cinkeys(c_int::from(b'#'), c_int::from(b' '), true))
        {
            let directive = skipwhite(line.theline.add(1));
            if (*curbuf.get()).b_ind_pragma == 0
                || !CStr::from_ptr(directive).to_bytes().starts_with(b"pragma")
            {
                return Some((*curbuf.get()).b_ind_hash_comment);
            }
        }

        // A non-case label goes at the left margin too, unless the JS flag is
        // set or 'cinoptions' `L` is positive.
        if line.original_line_islabel
            && (*curbuf.get()).b_ind_js == 0
            && (*curbuf.get()).b_ind_jump_label < 0
        {
            return Some(0);
        }

        // Inside a `//` comment with another one above: line up with it.
        if cin_islinecomment(line.theline)
            && let Some(amount) = incomment::align_with_line_comment()
        {
            return Some(amount);
        }

        // Inside a `/* */` comment, and not looking at its start: the
        // 'comments' option decides.
        if !cin_iscomment(line.theline)
            && let Some(comment) = comment_pos.as_mut()
        {
            return Some(incomment::align_in_comment(line, comment));
        }

        // A `]` that has a match lines up with the line holding the `[`.
        if *skipwhite(line.theline) as u8 == b']' {
            let trypos = find_match_char(b'[', (*curbuf.get()).b_ind_maxparen);
            if !trypos.is_null() {
                return Some(get_indent_lnum((*trypos).lnum));
            }
        }

        // Inside parentheses or braces?  Upstream spells the test as
        // `(paren && !java) || (brace = find_start_brace()) || paren`, so the
        // brace search runs in every case but "a paren, and not Java".
        let mut paren = find_match_paren((*curbuf.get()).b_ind_maxparen);
        let mut brace = if !paren.is_null() && (*curbuf.get()).b_ind_java == 0 {
            ::core::ptr::null_mut::<pos_T>()
        } else {
            find_start_brace()
        };
        if paren.is_null() && brace.is_null() {
            return Some(toplevel::indent_at_top_level(line));
        }
        if !paren.is_null() && !brace.is_null() {
            // Both unmatched: take the one closer to the cursor.
            let paren_is_further_up = if (*paren).lnum != (*brace).lnum {
                (*paren).lnum < (*brace).lnum
            } else {
                (*paren).col < (*brace).col
            };
            if paren_is_further_up {
                paren = ::core::ptr::null_mut::<pos_T>();
            } else {
                brace = ::core::ptr::null_mut::<pos_T>();
            }
        }

        let mut amount = if paren.is_null() {
            inblock::indent_in_block(line, *brace)
        } else {
            inparen::indent_in_parens(line, *paren)
        };

        // Extra indent for a comment.
        if cin_iscomment(line.theline) {
            amount += (*curbuf.get()).b_ind_comment;
        }
        // Take back the extra left shift jump labels get.
        if (*curbuf.get()).b_ind_jump_label > 0 && line.original_line_islabel {
            amount -= (*curbuf.get()).b_ind_jump_label;
        }
        Some(amount)
    }
}
