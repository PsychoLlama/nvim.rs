//! Paragraphs and sections: the `{`/`}`/`[[`/`]]` motions and `ip`/`ap`.
//!
//! A paragraph boundary is an empty line, a form feed, or a line matching one
//! of the two-letter nroff macro lists in 'paragraphs'/'sections'.
//! [`starts_para`] is that test -- the rest of the tree asks it too -- and
//! [`findpar`] and [`current_par`] are the two shapes built on it.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::cstr;
use crate::cstr::byte_at;
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later, showmode};
use crate::mark::setpcmark;
use crate::mbyte::head_off;
use crate::memline::Lines;
use crate::normal::{
    VisualMode, set_visual_anchor, set_visual_mode, visual_active, visual_anchor, visual_mode,
};
use crate::option::vars::{p_para, p_sections};
use crate::search::{BACKWARD, FORWARD, linewhite};
use crate::types::ColNr;
use crate::types::{FAIL, LineNr, OK, OpArg};

/// `{` / `}` / `[[` / `]]`: move to the `count`th paragraph or section
/// boundary in `dir`, answering whether one was found.
///
/// `what` is NUL for a paragraph and `{` or `}` for a section; `both` also
/// stops at a `}` in column 0. `pincl` is set when the last character of the
/// buffer is reached and has to be included in the motion.
///
/// # Safety
/// `pincl` must be writable, and there must be a current window.
pub unsafe fn findpar(
    pincl: *mut bool,
    dir: c_int,
    mut count: c_int,
    what: c_int,
    both: bool,
) -> bool {
    let mut curr = Win::current().w_cursor.lnum;

    loop {
        let this = count;
        count -= 1;
        if this == 0 {
            break;
        }
        // Set once the separating lines have been skipped: a boundary
        // only counts after at least one non-empty line.
        let mut did_skip = false;
        let mut first = true;
        loop {
            // SAFETY: on the main thread with a current buffer; `ml_get`
            // checks the line number itself and hands back a NUL-terminated
            // line, so its first byte is there to read.
            if !Lines::current().line(curr).is_empty() {
                did_skip = true;
            }
            // Skip over a closed fold, which counts as one line.
            let mut fold_skipped = false;
            if first {
                let (folded, fold_first, fold_last) = Win::current().fold_span(curr);
                if folded {
                    curr = (if dir > 0 { fold_last } else { fold_first }) + dir as LineNr;
                    fold_skipped = true;
                }
            }
            if !first && did_skip && line_starts_para(curr, what, both) {
                break;
            }
            if fold_skipped {
                curr -= dir as LineNr;
            }
            curr += dir as LineNr;
            if curr < 1 || curr > Buf::current().b_ml.ml_line_count {
                if count != 0 {
                    return false;
                }
                curr -= dir as LineNr;
                break;
            }
            first = false;
        }
    }

    // SAFETY: on the main thread with a current window.
    setpcmark();
    if both && Lines::current().line(curr).first() == Some(&b'}') {
        curr += 1; // include the line holding the `}`
    }
    Win::current().w_cursor.lnum = curr;
    if curr == Buf::current().b_ml.ml_line_count && what != '}' as c_int && dir == FORWARD as c_int
    {
        // Put the cursor on the last character of the last line and make
        // the motion inclusive.
        let mut lines = Lines::current();
        let line = lines.line(curr);
        Win::current().w_cursor.col = line.len() as ColNr;
        if Win::current().w_cursor.col != 0 {
            let col = Win::current().w_cursor.col - 1;
            Win::current().w_cursor.col = col - head_off(line, col as usize) as ColNr;
            // SAFETY: the caller guarantees `pincl` is writable.
            unsafe { *pincl = true };
        }
    } else {
        Win::current().w_cursor.col = 0;
    }
    true
}

/// Whether `s` opens with an nroff macro named in `opt` -- a list of
/// two-character names run together, as 'paragraphs' and 'sections' are.
///
/// A space in either position matches a space in the line or the line having
/// ended, which is how a one-letter macro is spelled.
///
/// `opt` is read two bytes at a time; `s` is only ever asked for its first
/// two, because a macro name is what a line *starts* with. Past the end of
/// either is a NUL, which is what the "a space also matches the end" arm
/// tests for.
fn inmacro(opt: &[u8], s: &[u8]) -> bool {
    let (s0, s1) = (byte_at(s, 0), byte_at(s, 1));
    let mut at = 0usize;
    while byte_at(opt, at) != 0 {
        let (m0, m1) = (byte_at(opt, at), byte_at(opt, at + 1));
        if (m0 == s0 || (m0 == b' ' && (s0 == 0 || s0 == b' ')))
            && (m1 == s1 || ((m1 == 0 || m1 == b' ') && (s0 == 0 || s1 == 0 || s1 == b' ')))
        {
            break;
        }
        at += 1;
        if byte_at(opt, at) == 0 {
            break;
        }
        at += 1;
    }
    byte_at(opt, at) != 0
}

/// Whether line `lnum` starts a section or a paragraph.
///
/// `para` is `{` or `}` to ask about sections only; `both` also stops at a
/// `}` in column 0.
pub fn starts_para(lnum: LineNr, para: c_int, both: bool) -> bool {
    let mut lines = Lines::current();
    let line = lines.line(lnum);
    let first = byte_at(line, 0);
    if c_int::from(first) == para || first == 0x0c || (both && first == b'}') {
        return true;
    }
    if first != b'.' {
        return false;
    }
    // SAFETY: 'sections' and 'paragraphs' are NUL-terminated option values.
    let (sections, paragraphs) = unsafe {
        (
            cstr::bytes_at(p_sections.get()),
            cstr::bytes_at(p_para.get()),
        )
    };
    let name = &line[1..];
    inmacro(sections, name) || (para == 0 && inmacro(paragraphs, name))
}

/// Grow an existing linewise Visual selection by `count` more paragraphs.
///
/// This is upstream's `extend:` label, reached both when the selection is
/// already more than one line and from the bottom of [`current_par`] when it
/// would otherwise get stuck -- `Vipipip` on a single white line.
///
/// Answers OK, or FAIL when the buffer ran out.
fn extend_paragraphs(mut start_lnum: LineNr, count: c_int, include: bool) -> c_int {
    let mut retval = OK;
    let dir = if start_lnum < visual_anchor().lnum {
        BACKWARD as c_int
    } else {
        FORWARD as c_int
    };
    // The line the walk cannot pass, in whichever direction it runs.
    let limit = |dir: c_int| {
        if dir == BACKWARD as c_int {
            1
        } else {
            Buf::current().b_ml.ml_line_count
        }
    };
    let mut i = count;
    loop {
        i -= 1;
        if i < 0 {
            break;
        }
        if start_lnum == limit(dir) {
            retval = FAIL;
            break;
        }
        // Two passes when white space is included: one over the
        // paragraph, one over the blank lines beside it. A pass that
        // finds the same kind of line as the one before it has run out
        // of paragraph.
        let mut prev_start_is_white = -1;
        for _ in 0..2 {
            start_lnum += dir as LineNr;
            let start_is_white = line_is_white(start_lnum) as c_int;
            if prev_start_is_white == start_is_white {
                start_lnum -= dir as LineNr;
                break;
            }
            while start_lnum != limit(dir) {
                if start_is_white != line_is_white(start_lnum + dir as LineNr) as c_int
                    || (start_is_white == 0
                        && line_starts_para(start_lnum + if dir > 0 { 1 } else { 0 }, 0, false))
                {
                    break;
                }
                start_lnum += dir as LineNr;
            }
            if !include || start_lnum == limit(dir) {
                break;
            }
            prev_start_is_white = start_is_white;
        }
    }
    Win::current().w_cursor.lnum = start_lnum;
    Win::current().w_cursor.col = 0;
    retval
}

/// `ip` / `ap`: the paragraph under the cursor, linewise, cursor left on its
/// last line. In Visual mode an existing multi-line selection is extended
/// instead.
///
/// `type_0` is `p`; `S` (section) is not implemented upstream and answers
/// FAIL.
///
/// # Safety
/// `op` must be a live operator argument, and there must be a current line.
pub unsafe fn current_par(op: *mut OpArg, count: c_int, include: bool, type_0: c_int) -> c_int {
    if type_0 == 'S' as c_int {
        return FAIL; // not implemented yet
    }
    let mut start_lnum = Win::current().w_cursor.lnum;

    // A Visual area of more than one line is extended, not replaced.
    if visual_active() && start_lnum != visual_anchor().lnum {
        return extend_paragraphs(start_lnum, count, include);
    }

    // Back to the start of the paragraph, or of the run of white lines.
    let white_in_front = line_is_white(start_lnum);
    while start_lnum > 1 {
        if white_in_front {
            if !line_is_white(start_lnum - 1) {
                break; // stop at the first white line
            }
        } else if line_is_white(start_lnum - 1) || line_starts_para(start_lnum, 0, false) {
            break; // stop at the paragraph's first line
        }
        start_lnum -= 1;
    }

    // Past the end of any white lines.
    let mut end_lnum = start_lnum;
    while end_lnum <= Buf::current().b_ml.ml_line_count && line_is_white(end_lnum) {
        end_lnum += 1;
    }
    end_lnum -= 1;

    let mut i = count;
    if !include && white_in_front {
        i -= 1;
    }
    // Whether the *next* run of white lines belongs to this object, which
    // for `ip` alternates: text, then blanks, then text.
    let mut do_white = false;
    loop {
        let this = i;
        i -= 1;
        if this == 0 {
            break;
        }
        if end_lnum == Buf::current().b_ml.ml_line_count {
            return FAIL;
        }
        if !include {
            do_white = line_is_white(end_lnum + 1);
        }
        if include || !do_white {
            end_lnum += 1;
            // On to the end of the paragraph.
            while end_lnum < Buf::current().b_ml.ml_line_count
                && !line_is_white(end_lnum + 1)
                && !line_starts_para(end_lnum + 1, 0, false)
            {
                end_lnum += 1;
            }
        }
        if i == 0 && white_in_front && include {
            break;
        }
        // On to the end of the white lines after the paragraph.
        if include || do_white {
            while end_lnum < Buf::current().b_ml.ml_line_count && line_is_white(end_lnum + 1) {
                end_lnum += 1;
            }
        }
    }

    // With no empty lines at the end, take some at the start instead --
    // unless that has been done already.
    if !white_in_front && !line_is_white(end_lnum) && include {
        while start_lnum > 1 && line_is_white(start_lnum - 1) {
            start_lnum -= 1;
        }
    }

    if visual_active() {
        // `Vipipip` on a single white line would otherwise get stuck
        // here, so hand it to the extending path instead.
        if visual_mode().is_line() && start_lnum == Win::current().w_cursor.lnum {
            return extend_paragraphs(start_lnum, count, include);
        }
        if visual_anchor().lnum != start_lnum {
            set_visual_anchor(visual_anchor().with_lnum(start_lnum).with_col(0));
        }
        set_visual_mode(VisualMode::LINE);
        redraw_curbuf_later(UPD_INVERTED); // update the inversion
        showmode();
    } else {
        // SAFETY: the caller guarantees `op` is a live operator argument.
        let op = unsafe { &mut *op };
        op.start.lnum = start_lnum;
        op.start.col = 0;
        op.motion_type = kMTLineWise;
    }
    Win::current().w_cursor.lnum = end_lnum;
    Win::current().w_cursor.col = 0;
    OK
}

/// [`linewhite`] for a line of the current buffer.
fn line_is_white(lnum: LineNr) -> bool {
    linewhite(lnum)
}

/// [`starts_para`] for a line of the current buffer.
fn line_starts_para(lnum: LineNr, para: c_int, both: bool) -> bool {
    starts_para(lnum, para, both)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `'sections'`/`'paragraphs'` are two-letter macro names run together,
    /// and `inmacro` is asked about the text *after* a line's leading `.`.
    fn matches(opt: &[u8], after_dot: &[u8]) -> bool {
        inmacro(opt, after_dot)
    }

    #[test]
    fn a_two_letter_name_matches_the_line_it_names() {
        let opt = b"SHNHH HUnhshsh";
        assert!(matches(opt, b"SH"));
        assert!(matches(opt, b"NH"));
        assert!(matches(opt, b"nh"));
        assert!(!matches(opt, b"XY"));
    }

    #[test]
    fn only_the_first_two_bytes_of_the_line_are_looked_at() {
        // The rest of the line is arguments to the macro -- and it is not
        // examined at all, so a *longer* word beginning with the item's two
        // letters matches it too. Upstream's behaviour, pinned.
        assert!(matches(b"SH", b"SH Introduction"));
        assert!(matches(b"SH", b"SHX"));
    }

    #[test]
    fn a_space_in_the_item_is_a_one_letter_name() {
        // "P " names the one-letter macro `.P`, which the line spells with
        // nothing after it or with a space.
        let opt = b"P LI";
        assert!(matches(opt, b"P"));
        assert!(matches(opt, b"P foo"));
        assert!(matches(opt, b"LI"));
        assert!(!matches(opt, b"PX"));
    }

    #[test]
    fn a_space_in_the_line_matches_a_space_in_the_item() {
        // The first byte's space arm: an item beginning with a space
        // matches a line that has one there, or has ended.
        assert!(matches(b" P", b" P"));
        assert!(!matches(b" P", b"xP"));
    }

    #[test]
    fn an_empty_option_matches_nothing() {
        assert!(!matches(b"", b"SH"));
        assert!(!matches(b"", b""));
    }

    #[test]
    fn an_odd_length_option_stops_on_its_last_byte() {
        // "SHP" is one full item and a stray letter; the walk steps two at
        // a time, sees the NUL where the item's second byte would be, and
        // stops -- so the stray letter is an item whose second byte is the
        // end of the option.
        assert!(matches(b"SHP", b"SH"));
        assert!(matches(b"SHP", b"P"));
        assert!(!matches(b"SHP", b"PX"));
    }

    #[test]
    fn a_line_that_is_only_the_dot_reads_the_terminator() {
        // `starts_para` passes the bytes after the `.`, which for a line
        // of just "." is empty. Nothing is read past the end.
        assert!(!matches(b"SHNH", b""));
        // An item whose *first* byte is a space gets the "or the line
        // having ended" arm, but its second byte then has to be a space or
        // the end as well -- " P" does not match a bare ".".
        assert!(!matches(b" P", b""));
        assert!(matches(b"  ", b""));
    }
}
