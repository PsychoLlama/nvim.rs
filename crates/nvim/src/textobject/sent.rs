//! Sentences: the `(`/`)` motions and the `is`/`as` objects.
//!
//! A sentence ends at `.`, `!` or `?` followed by white space or end of line,
//! with any of `)]"'` allowed in between, and 'cpoptions' `J` decides whether
//! one space is enough or two are needed. [`findsent`] is that rule;
//! everything else here is about which side of the surrounding white space an
//! object claims.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::mbyte::Codepoint;
use crate::normal::defs::{MotionType, OpArg};
use crate::option::vars::P_SEL;
use crate::strings::has_char;
use crate::types::ColNr;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::ascii::ascii_iswhite;
use crate::cursor::gchar_cursor;
use crate::drawscreen::state::REDRAW_CMDLINE;
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::mark::setpcmark;
use crate::memline::{Lines, Moved, decl, gchar_pos, inc, incl};
use crate::normal::{VisualMode, set_visual_anchor, set_visual_mode, visual_active, visual_anchor};
use crate::option::cpo_has;
use crate::option::defs::CpoFlag;
use crate::pos::{equalpos, lt};
use crate::search::{BACKWARD, FORWARD};
use crate::types::{Direction, Pos};

/// One step of a position walk: [`incl`] going forward, [`decl`] going back.
type StepFn = fn(&mut Pos) -> Moved;

/// Move to the start of the `count`th next sentence in `dir`, leaving the
/// cursor there. Answers `Ok` when one was found.
///
/// The definition (`:h sentence`): a sentence ends at `.`, `!` or `?`
/// followed by end of line, or by a space or tab -- two of them when
/// 'cpoptions' has `J` -- with any number of `)`, `]`, `"` and `'` in
/// between. A paragraph or section boundary ends one as well.
pub(crate) fn findsent(dir: Direction, mut count: c_int) -> Result<(), NoObject> {
    let mut noskip = false; // do not skip blanks
    let mut pos = Win::current().w_cursor;
    let step: StepFn = if dir as c_int == FORWARD as c_int {
        incl
    } else {
        decl
    };

    loop {
        let this = count;
        count -= 1;
        if this == 0 {
            break;
        }
        let prev_pos = pos;
        let (end, buffer_ended) = next_sentence(pos, dir, step, count)?;
        pos = end;
        if buffer_ended {
            noskip = true;
        }

        // Skip the white space in front of the sentence.
        while !noskip && {
            let c = gchar_pos(&pos);
            c == b' ' || c == b'\t'
        } {
            if incl(&mut pos).is_stuck() {
                break;
            }
        }

        if equalpos(prev_pos, pos) {
            // Nothing moved: step one character and try again.
            if step(&mut pos).is_stuck() {
                if count != 0 {
                    return Err(NoObject);
                }
                break;
            }
            count += 1;
        }
    }

    setpcmark();
    Win::current().w_cursor = pos;
    Ok(())
}

/// One sentence of [`findsent`]: from `pos`, past the end of the sentence
/// it is in (or, on an empty line or a paragraph start, to the next
/// non-empty line). Answers where that is -- the white space before the
/// next sentence -- and whether the buffer ended before the sentence did,
/// so that no white space is to be skipped; `Err` when it ended with
/// `count` sentences still to go.
fn next_sentence(
    mut pos: Pos,
    dir: Direction,
    step: StepFn,
    count: c_int,
) -> Result<(Pos, bool), NoObject> {
    // For every walk below: `pos` starts at the cursor and
    // only ever moves by `incl`/`decl`/`inc`, so it stays a position
    // of the current buffer the caller guarantees.
    if gchar_pos(&pos) == Codepoint::NUL {
        // On an empty line: skip up to a non-empty one.
        while !step(&mut pos).is_stuck() {
            if gchar_pos(&pos) != Codepoint::NUL {
                break;
            }
        }
        if dir as c_int == FORWARD as c_int {
            return Ok((pos, false));
        }
    } else if dir as c_int == FORWARD as c_int
        && pos.col == ColNr::ZERO
        && starts_para(pos.lnum, 0, false)
    {
        // At the start of a paragraph or section, going forward:
        // the next line is the answer.
        if pos.lnum == Buf::current().b_ml.ml_line_count {
            return Err(NoObject);
        }
        pos.lnum += 1;
        return Ok((pos, false));
    } else if dir as c_int == BACKWARD as c_int {
        decl(&mut pos);
    }

    // Back to the previous non-white, non-punctuation character.
    let mut found_dot = false;
    loop {
        let c = gchar_pos(&pos);
        if !(ascii_iswhite(c) || has_char(c".!?)]\"'", c.get())) {
            break;
        }
        let mut tpos = pos;
        let moved = decl(&mut tpos);
        // The line is read only once `decl` has answered that `tpos`
        // moved, so it is still in the buffer.
        if moved.is_stuck() || (dir == FORWARD && Lines::current().line(tpos.lnum).is_empty()) {
            break;
        }
        if found_dot {
            break;
        }
        if has_char(c".!?", c.get()) {
            found_dot = true;
        }
        if has_char(c")]\"'", c.get())
            // `tpos` is a position of the current buffer.
            && !has_char(c".!?)]\"'", gchar_pos(&tpos).get())
        {
            break;
        }
        decl(&mut pos);
    }

    // The line the search started on, so that a backward search
    // that crossed one can step back onto it.
    let startlnum = pos.lnum;
    let cpo_j = cpo_has(CpoFlag::ENDOFSENT);

    loop {
        // Find the end of the sentence.
        let mut c = gchar_pos(&pos);
        // `pos.lnum` is a line of the current buffer.
        if c == Codepoint::NUL || (pos.col == ColNr::ZERO && starts_para(pos.lnum, 0, false)) {
            if dir as c_int == BACKWARD as c_int && pos.lnum != startlnum {
                pos.lnum += 1;
            }
            break;
        }
        if c == b'.' || c.get() == '!' as c_int || c.get() == '?' as c_int {
            let mut tpos = pos;
            // Whether the end of the buffer came before anything
            // but closing punctuation.
            let mut at_end = false;
            loop {
                if inc(&mut tpos).is_stuck() {
                    at_end = true;
                    break;
                }
                c = gchar_pos(&tpos);
                if !has_char(c")]\"'", c.get()) {
                    break;
                }
            }
            if at_end
                || (!cpo_j && (c == b' ' || c == b'\t'))
                || c == Codepoint::NUL
                || (cpo_j && c == b' ' && !inc(&mut tpos).is_stuck() && gchar_pos(&tpos) == b' ')
            {
                pos = tpos;
                if gchar_pos(&pos) == Codepoint::NUL {
                    inc(&mut pos); // skip the NUL at end of line
                }
                break;
            }
        }
        if step(&mut pos).is_stuck() {
            if count != 0 {
                return Err(NoObject);
            }
            return Ok((pos, true));
        }
    }
    Ok((pos, false))
}

/// Move `posp` back to the first character of the run of white space it ends
/// in, or leave it alone when it is not preceded by any. `pos` names a
/// position in the current buffer, and `decl`/`incl` leave it as one.
fn find_first_blank(pos: &mut Pos) {
    while !decl(pos).is_stuck() {
        if !ascii_iswhite(gchar_pos(pos)) {
            incl(pos);
            break;
        }
    }
}

/// [`find_first_blank`] of the cursor.
fn cursor_to_first_blank() {
    let mut cursor = Win::current().w_cursor;
    find_first_blank(&mut cursor);
    Win::current().w_cursor = cursor;
}

/// Skip `count`/2 sentences and `count`/2 runs of the white space between
/// them, starting on whichever of the two `at_start_sent` says.
fn findsent_forward(mut count: c_int, mut at_start_sent: bool) {
    loop {
        let this = count;
        count -= 1;
        if this == 0 {
            break;
        }
        let _ = findsent(FORWARD, 1);
        if at_start_sent {
            cursor_to_first_blank();
        }
        if count == 0 || at_start_sent {
            decl(&mut Win::current().cursor());
        }
        at_start_sent = !at_start_sent;
    }
}

/// Grow an existing Visual selection by `count` more sentences, in whichever
/// direction the cursor sits relative to its start.
///
/// This is upstream's `extend:` label, which is jumped to from two places:
/// once when the selection is already bigger than one character, and once
/// from the bottom of [`current_sent`] when the object it computed turned out
/// to be empty -- `is` on a single space before a sentence, which would
/// otherwise never move.
fn extend_sentences(mut count: c_int, include: bool, start_pos: Pos, mut pos: Pos) {
    if lt(start_pos, visual_anchor()) {
        // The cursor is at the start of the Visual area. Work out where
        // that is: in the white space before a sentence, inside one or
        // just after it, or exactly at the start of one.
        let mut at_start_sent = true;
        decl(&mut pos);
        while lt(pos, Win::current().w_cursor) {
            if !ascii_iswhite(gchar_pos(&pos)) {
                at_start_sent = false;
                break;
            }
            incl(&mut pos);
        }
        if !at_start_sent {
            let _ = findsent(BACKWARD, 1);
            if equalpos(Win::current().w_cursor, start_pos) {
                at_start_sent = true; // exactly at the start of a sentence
            } else {
                // Inside a sentence: go to its end, the next one's start.
                let _ = findsent(FORWARD, 1);
            }
        }
        if include {
            count *= 2; // `as` gets twice as much as `is`
        }
        loop {
            let this = count;
            count -= 1;
            if this == 0 {
                break;
            }
            if at_start_sent {
                cursor_to_first_blank();
            }
            let c = gchar_cursor();
            if !at_start_sent || (!include && !ascii_iswhite(c)) {
                let _ = findsent(BACKWARD, 1);
            }
            at_start_sent = !at_start_sent;
        }
    } else {
        // The cursor is at the end of the Visual area: just before a
        // sentence, in or just before the white space in front of one, or
        // inside one.
        incl(&mut pos);
        let mut at_start_sent = true;
        if !equalpos(pos, Win::current().w_cursor) {
            // Not just before a sentence.
            at_start_sent = false;
            while lt(pos, Win::current().w_cursor) {
                if !ascii_iswhite(gchar_pos(&pos)) {
                    at_start_sent = true;
                    break;
                }
                incl(&mut pos);
            }
            if at_start_sent {
                let _ = findsent(BACKWARD, 1); // inside the sentence
            } else {
                Win::current().w_cursor = start_pos; // in the white space
            }
        }
        if include {
            count *= 2; // `as` gets twice as much as `is`
        }
        findsent_forward(count, at_start_sent);
        if P_SEL.first_byte() == b'e' {
            Win::current().w_cursor.col += 1;
        }
    }
}

/// `is` / `as`: the sentence(s) under the cursor, cursor left at the end. In
/// Visual mode an existing selection is extended by one or more sentences
/// instead.
pub(crate) fn current_sent(op: &mut OpArg, count: c_int, include: bool) -> Result<(), NoObject> {
    let mut start_pos = Win::current().w_cursor;
    let mut pos = start_pos;
    let _ = findsent(FORWARD, 1); // the start of the next sentence

    // A Visual area bigger than one character is extended, not replaced.
    if visual_active() && !equalpos(start_pos, visual_anchor()) {
        extend_sentences(count, include, start_pos, pos);
        return Ok(());
    }

    // The cursor started on a blank: is it just before the start of the
    // next sentence?
    while ascii_iswhite(gchar_pos(&pos)) {
        incl(&mut pos);
    }
    let start_blank = equalpos(pos, Win::current().w_cursor);
    if start_blank {
        find_first_blank(&mut start_pos); // back to the first blank
    } else {
        let _ = findsent(BACKWARD, 1);
        start_pos = Win::current().w_cursor;
    }

    let ncount = if include {
        count * 2
    } else if start_blank {
        count - 1
    } else {
        count
    };
    if ncount > 0 {
        findsent_forward(ncount, true);
    } else {
        decl(&mut Win::current().cursor());
    }

    if include {
        // With the blank in front of the sentence included, leave the
        // blanks at the end out: go back to the first of them. When there
        // are none, take the leading blanks instead.
        if start_blank {
            cursor_to_first_blank();
            if ascii_iswhite(gchar_pos(&Win::current().cursor())) {
                decl(&mut Win::current().cursor());
            }
        } else if !ascii_iswhite(gchar_cursor()) {
            find_first_blank(&mut start_pos);
        }
    }

    if visual_active() {
        // Don't get stuck with `is` on a single space before a sentence.
        if equalpos(start_pos, Win::current().w_cursor) {
            extend_sentences(count, include, start_pos, pos);
            return Ok(());
        }
        if P_SEL.first_byte() == b'e' {
            Win::current().w_cursor.col += 1;
        }
        set_visual_anchor(start_pos);
        set_visual_mode(VisualMode::CHAR);
        REDRAW_CMDLINE.set(true); // show the mode later
        redraw_curbuf_later(UPD_INVERTED); // update the inversion
    } else {
        // Include the newline after the sentence, if there is one.
        let inclusive = incl(&mut Win::current().cursor()).is_stuck();
        op.inclusive = inclusive;
        op.start = start_pos;
        op.motion_type = MotionType::CharWise;
    }
    Ok(())
}
