//! Sentences: the `(`/`)` motions and the `is`/`as` objects.
//!
//! A sentence ends at `.`, `!` or `?` followed by white space or end of line,
//! with any of `)]"'` allowed in between, and 'cpoptions' `J` decides whether
//! one space is enough or two are needed. [`findsent`] is that rule;
//! everything else here is about which side of the surrounding white space an
//! object claims.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::ascii::ascii_iswhite;
use crate::cursor::gchar_cursor;
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::main::{p_sel, redraw_cmdline};
use crate::mark::setpcmark;
use crate::memline::{decl, gchar_pos, inc, incl, ml_get};
use crate::normal::{VisualMode, set_visual_anchor, set_visual_mode, visual_active, visual_anchor};
use crate::option::cpo_has;
use crate::pos::{equalpos, lt};
use crate::search::{BACKWARD, FORWARD};
use crate::strings::vim_strchr;
use crate::types::{CpoFlag, Direction, FAIL, NUL, OK, oparg_T, pos_T};

/// One step of a position walk: [`incl`] going forward, [`decl`] going back.
type StepFn = unsafe fn(&mut pos_T) -> c_int;

/// Move to the start of the `count`th next sentence in `dir`, leaving the
/// cursor there. Answers OK when one was found.
///
/// The definition (`:h sentence`): a sentence ends at `.`, `!` or `?`
/// followed by end of line, or by a space or tab -- two of them when
/// 'cpoptions' has `J` -- with any number of `)`, `]`, `"` and `'` in
/// between. A paragraph or section boundary ends one as well.
///
/// # Safety
/// There must be a current buffer and window.
pub unsafe fn findsent(dir: Direction, mut count: c_int) -> c_int {
    let mut noskip = false; // do not skip blanks
    let mut pos = cur_win().w_cursor;
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
        'found: {
            // SAFETY, for every walk below: `pos` starts at the cursor and
            // only ever moves by `incl`/`decl`/`inc`, so it stays a position
            // of the current buffer the caller guarantees.
            if unsafe { gchar_pos(&raw mut pos) } == NUL {
                // On an empty line: skip up to a non-empty one.
                while unsafe { step(&mut pos) } != -1 {
                    if unsafe { gchar_pos(&raw mut pos) } != NUL {
                        break;
                    }
                }
                if dir as c_int == FORWARD as c_int {
                    break 'found;
                }
            } else if dir as c_int == FORWARD as c_int
                && pos.col == 0
                // SAFETY: `pos.lnum` is a line of the current buffer.
                && unsafe { starts_para(pos.lnum, NUL, false) }
            {
                // At the start of a paragraph or section, going forward:
                // the next line is the answer.
                if pos.lnum == cur_buf().b_ml.ml_line_count {
                    return FAIL;
                }
                pos.lnum += 1;
                break 'found;
            } else if dir as c_int == BACKWARD as c_int {
                // SAFETY: as above.
                unsafe { decl(&mut pos) };
            }

            // Back to the previous non-white, non-punctuation character.
            let mut found_dot = false;
            loop {
                // SAFETY: as above.
                let c = unsafe { gchar_pos(&raw mut pos) };
                // SAFETY: the literal is a NUL-terminated string.
                if !(ascii_iswhite(c) || unsafe { !vim_strchr(c".!?)]\"'".as_ptr(), c).is_null() })
                {
                    break;
                }
                let mut tpos = pos;
                // SAFETY: as above; `ml_get` is reached only once `decl` has
                // answered that `tpos` moved, so it is still in the buffer --
                // the `||` is the proof and is left whole.
                if unsafe {
                    decl(&mut tpos) == -1
                        || (*ml_get(tpos.lnum) as c_int == NUL && dir as c_int == FORWARD as c_int)
                } {
                    break;
                }
                if found_dot {
                    break;
                }
                // SAFETY: the literal is a NUL-terminated string.
                if unsafe { !vim_strchr(c".!?".as_ptr(), c).is_null() } {
                    found_dot = true;
                }
                // SAFETY: the literals are NUL-terminated, and `tpos` is a
                // position of the current buffer.
                if unsafe {
                    !vim_strchr(c")]\"'".as_ptr(), c).is_null()
                        && vim_strchr(c".!?)]\"'".as_ptr(), gchar_pos(&raw mut tpos)).is_null()
                } {
                    break;
                }
                // SAFETY: as above.
                unsafe { decl(&mut pos) };
            }

            // The line the search started on, so that a backward search
            // that crossed one can step back onto it.
            let startlnum = pos.lnum;
            let cpo_j = cpo_has(CpoFlag::ENDOFSENT);

            loop {
                // Find the end of the sentence.
                // SAFETY: as above.
                let mut c = unsafe { gchar_pos(&raw mut pos) };
                // SAFETY: `pos.lnum` is a line of the current buffer.
                if c == NUL || (pos.col == 0 && unsafe { starts_para(pos.lnum, NUL, false) }) {
                    if dir as c_int == BACKWARD as c_int && pos.lnum != startlnum {
                        pos.lnum += 1;
                    }
                    break;
                }
                if c == '.' as c_int || c == '!' as c_int || c == '?' as c_int {
                    let mut tpos = pos;
                    loop {
                        // SAFETY: as above.
                        c = unsafe { inc(&mut tpos) };
                        if c == -1 {
                            break;
                        }
                        // SAFETY: as above; the literal is NUL-terminated.
                        c = unsafe { gchar_pos(&raw mut tpos) };
                        if unsafe { vim_strchr(c")]\"'".as_ptr(), c).is_null() } {
                            break;
                        }
                    }
                    // SAFETY: as above.  The last two steps run only when the
                    // ones in front of them held, so the `&&` chain is left
                    // whole.
                    if c == -1
                        || (!cpo_j && (c == ' ' as c_int || c == '\t' as c_int))
                        || c == NUL
                        || unsafe {
                            cpo_j
                                && c == ' ' as c_int
                                && inc(&mut tpos) >= 0
                                && gchar_pos(&raw mut tpos) == ' ' as c_int
                        }
                    {
                        pos = tpos;
                        // SAFETY: as above.
                        if unsafe { gchar_pos(&raw mut pos) } == NUL {
                            unsafe { inc(&mut pos) }; // skip the NUL at end of line
                        }
                        break;
                    }
                }
                // SAFETY: as above.
                if unsafe { step(&mut pos) } == -1 {
                    if count != 0 {
                        return FAIL;
                    }
                    noskip = true;
                    break;
                }
            }
        }

        // Skip the white space in front of the sentence.
        while !noskip && {
            // SAFETY: as above.
            let c = unsafe { gchar_pos(&raw mut pos) };
            c == ' ' as c_int || c == '\t' as c_int
        } {
            // SAFETY: as above.
            if unsafe { incl(&mut pos) } == -1 {
                break;
            }
        }

        if equalpos(prev_pos, pos) {
            // Nothing moved: step one character and try again.
            // SAFETY: as above.
            if unsafe { step(&mut pos) } == -1 {
                if count != 0 {
                    return FAIL;
                }
                break;
            }
            count += 1;
        }
    }

    // SAFETY: on the main thread with a current window.
    unsafe { setpcmark() };
    cur_win().w_cursor = pos;
    OK
}

/// Move `posp` back to the first character of the run of white space it ends
/// in, or leave it alone when it is not preceded by any.
///
/// # Safety
/// `posp` must name a valid position in the current buffer.
pub(crate) unsafe fn find_first_blank(posp: *mut pos_T) {
    // SAFETY: the caller guarantees `posp` names a position of the current
    // buffer, and `decl`/`incl` leave it as one.
    while unsafe { decl(&mut *posp) } != -1 {
        // SAFETY: as above.
        let c = unsafe { gchar_pos(posp) };
        if !ascii_iswhite(c) {
            // SAFETY: as above.
            unsafe { incl(&mut *posp) };
            break;
        }
    }
}

/// Skip `count`/2 sentences and `count`/2 runs of the white space between
/// them, starting on whichever of the two `at_start_sent` says.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
unsafe fn findsent_forward(mut count: c_int, mut at_start_sent: bool) {
    loop {
        let this = count;
        count -= 1;
        if this == 0 {
            break;
        }
        // SAFETY, throughout: the caller guarantees a current window whose
        // cursor is on a line of the current buffer, and each of these leaves
        // it on one for the next.
        unsafe { findsent(FORWARD, 1) };
        if at_start_sent {
            unsafe { find_first_blank(cur_win().cursor().raw()) };
        }
        if count == 0 || at_start_sent {
            unsafe { decl(&mut cur_win().cursor()) };
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
///
/// # Safety
/// There must be a current line and Visual mode must be active.
unsafe fn extend_sentences(mut count: c_int, include: bool, start_pos: pos_T, mut pos: pos_T) {
    if lt(start_pos, visual_anchor()) {
        // The cursor is at the start of the Visual area. Work out where
        // that is: in the white space before a sentence, inside one or
        // just after it, or exactly at the start of one.
        let mut at_start_sent = true;
        // SAFETY, throughout: `pos` and the cursor are positions of the
        // current buffer, which the caller guarantees, and every step here
        // leaves them as ones.
        unsafe { decl(&mut pos) };
        while lt(pos, cur_win().w_cursor) {
            if !ascii_iswhite(unsafe { gchar_pos(&raw mut pos) }) {
                at_start_sent = false;
                break;
            }
            unsafe { incl(&mut pos) };
        }
        if !at_start_sent {
            unsafe { findsent(BACKWARD, 1) };
            if equalpos(cur_win().w_cursor, start_pos) {
                at_start_sent = true; // exactly at the start of a sentence
            } else {
                // Inside a sentence: go to its end, the next one's start.
                unsafe { findsent(FORWARD, 1) };
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
                unsafe { find_first_blank(cur_win().cursor().raw()) };
            }
            let c = unsafe { gchar_cursor() };
            if !at_start_sent || (!include && !ascii_iswhite(c)) {
                unsafe { findsent(BACKWARD, 1) };
            }
            at_start_sent = !at_start_sent;
        }
    } else {
        // The cursor is at the end of the Visual area: just before a
        // sentence, in or just before the white space in front of one, or
        // inside one.
        // SAFETY: as above.
        unsafe { incl(&mut pos) };
        let mut at_start_sent = true;
        if !equalpos(pos, cur_win().w_cursor) {
            // Not just before a sentence.
            at_start_sent = false;
            while lt(pos, cur_win().w_cursor) {
                if !ascii_iswhite(unsafe { gchar_pos(&raw mut pos) }) {
                    at_start_sent = true;
                    break;
                }
                unsafe { incl(&mut pos) };
            }
            if at_start_sent {
                unsafe { findsent(BACKWARD, 1) }; // inside the sentence
            } else {
                cur_win().w_cursor = start_pos; // in the white space
            }
        }
        if include {
            count *= 2; // `as` gets twice as much as `is`
        }
        unsafe { findsent_forward(count, at_start_sent) };
        // SAFETY: 'selection' is a NUL-terminated option string.
        if unsafe { *p_sel.get() } as c_int == 'e' as c_int {
            cur_win().w_cursor.col += 1;
        }
    }
}

/// `is` / `as`: the sentence(s) under the cursor, cursor left at the end. In
/// Visual mode an existing selection is extended by one or more sentences
/// instead.
///
/// # Safety
/// `oap` must be a live operator argument, and there must be a current line.
pub unsafe fn current_sent(oap: *mut oparg_T, count: c_int, include: bool) -> c_int {
    let mut start_pos = cur_win().w_cursor;
    let mut pos = start_pos;
    // SAFETY, throughout: the caller guarantees a current window whose cursor
    // is on a line of the current buffer; `pos` and `start_pos` are copies of
    // it moved only by `incl`/`decl`, so they stay positions of the buffer.
    unsafe { findsent(FORWARD, 1) }; // the start of the next sentence

    // A Visual area bigger than one character is extended, not replaced.
    if visual_active() && !equalpos(start_pos, visual_anchor()) {
        unsafe { extend_sentences(count, include, start_pos, pos) };
        return OK;
    }

    // The cursor started on a blank: is it just before the start of the
    // next sentence?
    while ascii_iswhite(unsafe { gchar_pos(&raw mut pos) }) {
        unsafe { incl(&mut pos) };
    }
    let start_blank = equalpos(pos, cur_win().w_cursor);
    if start_blank {
        unsafe { find_first_blank(&raw mut start_pos) }; // back to the first blank
    } else {
        unsafe { findsent(BACKWARD, 1) };
        start_pos = cur_win().w_cursor;
    }

    let ncount = if include {
        count * 2
    } else if start_blank {
        count - 1
    } else {
        count
    };
    if ncount > 0 {
        unsafe { findsent_forward(ncount, true) };
    } else {
        unsafe { decl(&mut cur_win().cursor()) };
    }

    if include {
        // With the blank in front of the sentence included, leave the
        // blanks at the end out: go back to the first of them. When there
        // are none, take the leading blanks instead.
        if start_blank {
            unsafe { find_first_blank(cur_win().cursor().raw()) };
            if ascii_iswhite(unsafe { gchar_pos(cur_win().cursor().raw()) }) {
                unsafe { decl(&mut cur_win().cursor()) };
            }
        } else if !ascii_iswhite(unsafe { gchar_cursor() }) {
            unsafe { find_first_blank(&raw mut start_pos) };
        }
    }

    if visual_active() {
        // Don't get stuck with `is` on a single space before a sentence.
        if equalpos(start_pos, cur_win().w_cursor) {
            unsafe { extend_sentences(count, include, start_pos, pos) };
            return OK;
        }
        // SAFETY: 'selection' is a NUL-terminated option string.
        if unsafe { *p_sel.get() } as c_int == 'e' as c_int {
            cur_win().w_cursor.col += 1;
        }
        set_visual_anchor(start_pos);
        set_visual_mode(VisualMode::CHAR);
        redraw_cmdline.set(true); // show the mode later
        // SAFETY: on the main thread with a current buffer.
        unsafe { redraw_curbuf_later(UPD_INVERTED) }; // update the inversion
    } else {
        // Include the newline after the sentence, if there is one.
        // SAFETY: the cursor is on a line of the current buffer.
        let inclusive = unsafe { incl(&mut cur_win().cursor()) } == -1;
        // SAFETY: the caller guarantees `oap` is a live operator argument.
        let oap = unsafe { &mut *oap };
        oap.inclusive = inclusive;
        oap.start = start_pos;
        oap.motion_type = kMTCharWise;
    }
    OK
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
