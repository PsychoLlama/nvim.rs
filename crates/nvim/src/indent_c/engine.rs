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
use crate::types::NUL;
use crate::winlayer::{Buf, Win};
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
    // Remember where the cursor was when we started.
    let cur_curpos = cur_win().w_cursor;

    // At line 1, zero indent is fine, right?
    if cur_curpos.lnum == 1 {
        return 0;
    }

    // A copy, because only the most recent line `ml_get` answered stays
    // valid and everything below asks for more.
    // SAFETY: on the main thread with a current buffer; `xstrdup` copies the
    // NUL-terminated line `ml_get` answers with.
    let linecopy = unsafe { xstrdup(ml_get(cur_curpos.lnum)) };

    // In Insert mode with the cursor on a ')', truncate the line there:
    // new text should not line up with the matching '('.  The cursor can
    // be past the end of the line, for unknown reasons, so check.
    let col = cur_win().w_cursor.col;
    // SAFETY: `linecopy` is a NUL-terminated copy of the line, and the
    // `strlen` test -- which the `&&` chain keeps in front -- is what says
    // `col` indexes inside it.
    unsafe {
        if State.get() & MODE_INSERT != 0
            && (col as size_t) < strlen(linecopy)
            && *linecopy.offset(col as isize) as u8 == b')'
        {
            *linecopy.offset(col as isize) = NUL as c_char;
        }
    }

    // SAFETY: `linecopy` is NUL-terminated, so `skipwhite` stops inside it.
    let theline = unsafe { skipwhite(linecopy) }.cast_const();

    // Move the cursor to the start of the line, and judge the line before
    // anything else moves: 'cinoptions' `L` reads the answer again at the
    // very end.
    cur_win().w_cursor.col = 0;
    let line = Line {
        theline,
        linecopy: linecopy.cast_const(),
        cur_curpos,
        // SAFETY: the cursor is on a line of the current buffer.
        original_line_islabel: unsafe { cin_islabel() },
    };

    // SAFETY: `line` outlives the call, and the cursor is restored below.
    let amount = match unsafe { c_indent(&line) } {
        // Inside a raw string: leave the indent alone, and do not clamp.
        None => -1,
        Some(amount) => amount.max(0),
    };

    // Put the cursor back where it belongs.
    cur_win().w_cursor = cur_curpos;
    // SAFETY: `linecopy` came from `xstrdup` and nothing else owns it.
    unsafe { xfree(linecopy.cast::<::core::ffi::c_void>()) };
    amount
}

/// The dispatch: which context encloses the line, and what that context says.
///
/// `None` means "inside a raw string, leave the indent alone".
///
/// # Safety
/// Moves the cursor and unlocks the current line; the caller restores it.
unsafe fn c_indent(line: &Line) -> Option<c_int> {
    // A raw string wins over a comment only if it starts *earlier*; a raw
    // string inside a comment is just comment text.
    // SAFETY: on the main thread, with a current window and buffer.
    let mut comment_pos = unsafe { ind_find_start_comment() };
    // SAFETY: the same.
    let raw_string = unsafe { find_start_rawstring(cur_buf().b_ind_maxcomment) };
    if let Some(raw) = raw_string
        && comment_pos.is_none_or(|comment| lt(raw, comment))
    {
        return None;
    }

    // `#define` and friends go at the left when 'cinkeys' says so,
    // excluding `#pragma` when 'cinoptions' `P` asks.
    // SAFETY: `line`'s two pointers are NUL-terminated copies of the cursor's
    // line, alive for the whole call.  `theline.add(1)` is inside the copy
    // because `starts_with` has just seen a `#` at `theline[0]`, which the
    // `&&` chain keeps in front of it.
    let hash_at_left = unsafe {
        line.starts_with(b'#')
            && (*line.linecopy as u8 == b'#'
                || in_cinkeys(c_int::from(b'#'), c_int::from(b' '), true))
            && {
                let directive = skipwhite(line.theline.add(1));
                cur_buf().b_ind_pragma == 0
                    || !CStr::from_ptr(directive).to_bytes().starts_with(b"pragma")
            }
    };
    if hash_at_left {
        return Some(cur_buf().b_ind_hash_comment);
    }

    // A non-case label goes at the left margin too, unless the JS flag is
    // set or 'cinoptions' `L` is positive.
    if line.original_line_islabel && cur_buf().b_ind_js == 0 && cur_buf().b_ind_jump_label < 0 {
        return Some(0);
    }

    // Inside a `//` comment with another one above: line up with it.
    // SAFETY: `line.theline` is NUL-terminated; the alignment search only
    // runs when it is a `//` comment, as upstream has it.
    let aligned = unsafe {
        cin_islinecomment(line.theline)
            .then(|| incomment::align_with_line_comment())
            .flatten()
    };
    if let Some(amount) = aligned {
        return Some(amount);
    }

    // Inside a `/* */` comment, and not looking at its start: the
    // 'comments' option decides.
    // SAFETY: `line.theline` is NUL-terminated.
    if !unsafe { cin_iscomment(line.theline) }
        && let Some(comment) = comment_pos.as_mut()
    {
        // SAFETY: `line` outlives the call and `comment` is a position this
        // scan found in the current buffer.
        return Some(unsafe { incomment::align_in_comment(line, comment) });
    }

    // A `]` that has a match lines up with the line holding the `[`.
    // SAFETY: `line.theline` is NUL-terminated, so `skipwhite` stops inside
    // it; the match search runs on the current buffer, and only for a `]`.
    let bracket = unsafe {
        (*skipwhite(line.theline) as u8 == b']')
            .then(|| find_match_char(b'[', cur_buf().b_ind_maxparen))
            .flatten()
    };
    if let Some(trypos) = bracket {
        // SAFETY: `trypos` is a position in the current buffer.
        return Some(unsafe { get_indent_lnum(trypos.lnum) });
    }

    // Inside parentheses or braces?  Upstream spells the test as
    // `(paren && !java) || (brace = find_start_brace()) || paren`, so the
    // brace search runs in every case but "a paren, and not Java".
    // SAFETY: both search the current buffer from the cursor.
    let mut paren = unsafe { find_match_paren(cur_buf().b_ind_maxparen) };
    let mut brace = if paren.is_some() && cur_buf().b_ind_java == 0 {
        None
    } else {
        // SAFETY: the same.
        unsafe { find_start_brace() }
    };
    if let (Some(p), Some(b)) = (paren, brace) {
        // Both unmatched: take the one closer to the cursor.
        let paren_is_further_up = if p.lnum != b.lnum {
            p.lnum < b.lnum
        } else {
            p.col < b.col
        };
        if paren_is_further_up {
            paren = None;
        } else {
            brace = None;
        }
    }

    // SAFETY: `line` outlives the call, and `paren`/`brace` are positions
    // this function found in the current buffer.
    let mut amount = unsafe {
        match (paren, brace) {
            (Some(paren), _) => inparen::indent_in_parens(line, paren),
            (None, Some(brace)) => indent_in_block(line, brace),
            (None, None) => return Some(toplevel::indent_at_top_level(line)),
        }
    };

    // Extra indent for a comment.
    // SAFETY: `line.theline` is NUL-terminated.
    if unsafe { cin_iscomment(line.theline) } {
        amount += cur_buf().b_ind_comment;
    }
    // Take back the extra left shift jump labels get.
    if cur_buf().b_ind_jump_label > 0 && line.original_line_islabel {
        amount -= cur_buf().b_ind_jump_label;
    }
    Some(amount)
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
