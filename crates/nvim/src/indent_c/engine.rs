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

#![forbid(unsafe_code)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

/// The line being indented, and where it sits.
///
/// The text is a private copy: `ml_get` only guarantees the *most recent*
/// line it answered, and this code asks for hundreds of others before it is
/// done.
pub(crate) struct Line {
    /// The copy, from column 0.
    copy: Vec<u8>,
    /// Where the text being judged starts in it -- the leading white space
    /// skipped.
    start: usize,
    /// Where the cursor was when the question was asked.
    pub cur_curpos: Pos,
    /// Whether that line is a jump label, decided before anything moved.
    pub original_line_islabel: bool,
}

impl Line {
    /// The cursor's line, white space skipped -- the text being judged.
    pub(crate) fn theline(&self) -> &[u8] {
        &self.copy[self.start..]
    }

    /// The same copy from column 0, which the `#` test reads.
    pub(crate) fn whole(&self) -> &[u8] {
        &self.copy
    }

    /// Whether the line being indented starts with `c`.
    pub(crate) fn starts_with(&self, c: u8) -> bool {
        self.theline().first() == Some(&c)
    }
}

/// The indent for the cursor's line, or -1 to leave it alone (inside a raw
/// string).
pub fn get_c_indent() -> c_int {
    // Remember where the cursor was when we started.
    let cur_curpos = Win::current().w_cursor;

    // At line 1, zero indent is fine, right?
    if cur_curpos.lnum == 1 {
        return 0;
    }

    // A copy, because only the most recent line `ml_get` answered stays
    // valid and everything below asks for more.
    let mut copy = Lines::current().line(cur_curpos.lnum).to_vec();

    // In Insert mode with the cursor on a ')', truncate the line there:
    // new text should not line up with the matching '('.  The cursor can
    // be past the end of the line, for unknown reasons, so check.
    let col = usize::try_from(Win::current().w_cursor.col).unwrap_or(0);
    if State.get() & MODE_INSERT != 0 && copy.get(col) == Some(&b')') {
        copy.truncate(col);
    }
    let start = skip::white(&copy);

    // Move the cursor to the start of the line, and judge the line before
    // anything else moves: 'cinoptions' `L` reads the answer again at the
    // very end.
    Win::current().w_cursor.col = 0;
    let line = Line {
        copy,
        start,
        cur_curpos,
        original_line_islabel: is_jump_label(),
    };

    let amount = match c_indent(&line) {
        // Inside a raw string: leave the indent alone, and do not clamp.
        None => -1,
        Some(amount) => amount.max(0),
    };

    // Put the cursor back where it belongs.
    Win::current().w_cursor = cur_curpos;
    amount
}

/// The dispatch: which context encloses the line, and what that context says.
///
/// `None` means "inside a raw string, leave the indent alone".
fn c_indent(line: &Line) -> Option<c_int> {
    // A raw string wins over a comment only if it starts *earlier*; a raw
    // string inside a comment is just comment text.
    let mut comment_pos = ind_find_start_comment();
    let raw_string = find_start_rawstring(Buf::current().b_ind_maxcomment);
    if let Some(raw) = raw_string
        && comment_pos.is_none_or(|comment| lt(raw, comment))
    {
        return None;
    }

    // `#define` and friends go at the left when 'cinkeys' says so,
    // excluding `#pragma` when 'cinoptions' `P` asks.
    let theline = line.theline();
    let hash_at_left = line.starts_with(b'#')
        && (line.whole().first() == Some(&b'#')
            || in_cinkeys(c_int::from(b'#'), c_int::from(b' '), true))
        && {
            let directive = 1 + skip::white(&theline[1..]);
            Buf::current().b_ind_pragma == 0 || !theline[directive..].starts_with(b"pragma")
        };
    if hash_at_left {
        return Some(Buf::current().b_ind_hash_comment);
    }

    // A non-case label goes at the left margin too, unless the JS flag is
    // set or 'cinoptions' `L` is positive.
    if line.original_line_islabel
        && Buf::current().b_ind_js == 0
        && Buf::current().b_ind_jump_label < 0
    {
        return Some(0);
    }

    // Inside a `//` comment with another one above: line up with it.
    let aligned = starts_line_comment(theline, 0)
        .then(incomment::align_with_line_comment)
        .flatten();
    if let Some(amount) = aligned {
        return Some(amount);
    }

    // Inside a `/* */` comment, and not looking at its start: the
    // 'comments' option decides.
    if !starts_comment(theline, 0)
        && let Some(comment) = comment_pos.as_mut()
    {
        return Some(incomment::align_in_comment(line, comment));
    }

    // A `]` that has a match lines up with the line holding the `[`.
    let bracket = line
        .starts_with(b']')
        .then(|| find_match_char(b'[', Buf::current().b_ind_maxparen))
        .flatten();
    if let Some(trypos) = bracket {
        // SAFETY: `trypos` is a position in the current buffer.
        return Some(get_indent_lnum(trypos.lnum));
    }

    // Inside parentheses or braces?  Upstream spells the test as
    // `(paren && !java) || (brace = find_start_brace()) || paren`, so the
    // brace search runs in every case but "a paren, and not Java".
    let mut paren = find_match_paren(Buf::current().b_ind_maxparen);
    let mut brace = if paren.is_some() && Buf::current().b_ind_java == 0 {
        None
    } else {
        find_start_brace()
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

    let mut amount = match (paren, brace) {
        (Some(paren), _) => inparen::indent_in_parens(line, paren),
        (None, Some(brace)) => indent_in_block(line, brace),
        (None, None) => return Some(toplevel::indent_at_top_level(line)),
    };

    // Extra indent for a comment.
    if starts_comment(line.theline(), 0) {
        amount += Buf::current().b_ind_comment;
    }
    // Take back the extra left shift jump labels get.
    if Buf::current().b_ind_jump_label > 0 && line.original_line_islabel {
        amount -= Buf::current().b_ind_jump_label;
    }
    Some(amount)
}
