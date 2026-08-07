//! Sentences: the `(`/`)` motions and the `is`/`as` objects.
//!
//! A sentence ends at `.`, `!` or `?` followed by white space or end of line,
//! with any of `)]"'` allowed in between, and 'cpoptions' `J` decides whether
//! one space is enough or two are needed. [`findsent`] is that rule;
//! everything else here is about which side of the surrounding white space an
//! object claims.

#![deny(unsafe_op_in_unsafe_fn)]

use ::core::ffi::c_int;

use super::*;
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::cursor::gchar_cursor;
use crate::src::nvim::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::src::nvim::main::{
    VIsual, VIsual_active, VIsual_mode, curbuf, curwin, p_cpo, p_sel, redraw_cmdline,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::memline::{decl, gchar_pos, inc, incl, ml_get};
use crate::src::nvim::pos::{equalpos, lt};
use crate::src::nvim::search::{BACKWARD, FORWARD};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{Direction, oparg_T, pos_T};

/// One step of a position walk: [`incl`] going forward, [`decl`] going back.
type StepFn = unsafe fn(*mut pos_T) -> c_int;

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
pub unsafe extern "C" fn findsent(dir: Direction, mut count: c_int) -> c_int {
    unsafe {
        let mut noskip = false; // do not skip blanks
        let mut pos = (*curwin.get()).w_cursor;
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
                if gchar_pos(&raw mut pos) == NUL {
                    // On an empty line: skip up to a non-empty one.
                    while step(&raw mut pos) != -1 {
                        if gchar_pos(&raw mut pos) != NUL {
                            break;
                        }
                    }
                    if dir as c_int == FORWARD as c_int {
                        break 'found;
                    }
                } else if dir as c_int == FORWARD as c_int
                    && pos.col == 0
                    && startPS(pos.lnum, NUL, false)
                {
                    // At the start of a paragraph or section, going forward:
                    // the next line is the answer.
                    if pos.lnum == (*curbuf.get()).b_ml.ml_line_count {
                        return FAIL;
                    }
                    pos.lnum += 1;
                    break 'found;
                } else if dir as c_int == BACKWARD as c_int {
                    decl(&raw mut pos);
                }

                // Back to the previous non-white, non-punctuation character.
                let mut found_dot = false;
                loop {
                    let c = gchar_pos(&raw mut pos);
                    if !(ascii_iswhite(c) || !vim_strchr(c".!?)]\"'".as_ptr(), c).is_null()) {
                        break;
                    }
                    let mut tpos = pos;
                    if decl(&raw mut tpos) == -1
                        || (*ml_get(tpos.lnum) as c_int == NUL && dir as c_int == FORWARD as c_int)
                    {
                        break;
                    }
                    if found_dot {
                        break;
                    }
                    if !vim_strchr(c".!?".as_ptr(), c).is_null() {
                        found_dot = true;
                    }
                    if !vim_strchr(c")]\"'".as_ptr(), c).is_null()
                        && vim_strchr(c".!?)]\"'".as_ptr(), gchar_pos(&raw mut tpos)).is_null()
                    {
                        break;
                    }
                    decl(&raw mut pos);
                }

                // The line the search started on, so that a backward search
                // that crossed one can step back onto it.
                let startlnum = pos.lnum;
                let cpo_j = !vim_strchr(p_cpo.get(), CPO_ENDOFSENT).is_null();

                loop {
                    // Find the end of the sentence.
                    let mut c = gchar_pos(&raw mut pos);
                    if c == NUL || (pos.col == 0 && startPS(pos.lnum, NUL, false)) {
                        if dir as c_int == BACKWARD as c_int && pos.lnum != startlnum {
                            pos.lnum += 1;
                        }
                        break;
                    }
                    if c == '.' as c_int || c == '!' as c_int || c == '?' as c_int {
                        let mut tpos = pos;
                        loop {
                            c = inc(&raw mut tpos);
                            if c == -1 {
                                break;
                            }
                            c = gchar_pos(&raw mut tpos);
                            if vim_strchr(c")]\"'".as_ptr(), c).is_null() {
                                break;
                            }
                        }
                        if c == -1
                            || (!cpo_j && (c == ' ' as c_int || c == '\t' as c_int))
                            || c == NUL
                            || (cpo_j
                                && c == ' ' as c_int
                                && inc(&raw mut tpos) >= 0
                                && gchar_pos(&raw mut tpos) == ' ' as c_int)
                        {
                            pos = tpos;
                            if gchar_pos(&raw mut pos) == NUL {
                                inc(&raw mut pos); // skip the NUL at end of line
                            }
                            break;
                        }
                    }
                    if step(&raw mut pos) == -1 {
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
                let c = gchar_pos(&raw mut pos);
                c == ' ' as c_int || c == '\t' as c_int
            } {
                if incl(&raw mut pos) == -1 {
                    break;
                }
            }

            if equalpos(prev_pos, pos) {
                // Nothing moved: step one character and try again.
                if step(&raw mut pos) == -1 {
                    if count != 0 {
                        return FAIL;
                    }
                    break;
                }
                count += 1;
            }
        }

        setpcmark();
        (*curwin.get()).w_cursor = pos;
        OK
    }
}

/// Move `posp` back to the first character of the run of white space it ends
/// in, or leave it alone when it is not preceded by any.
///
/// # Safety
/// `posp` must name a valid position in the current buffer.
pub(crate) unsafe fn find_first_blank(posp: *mut pos_T) {
    unsafe {
        while decl(posp) != -1 {
            let c = gchar_pos(posp);
            if !ascii_iswhite(c) {
                incl(posp);
                break;
            }
        }
    }
}

/// Skip `count`/2 sentences and `count`/2 runs of the white space between
/// them, starting on whichever of the two `at_start_sent` says.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
unsafe fn findsent_forward(mut count: c_int, mut at_start_sent: bool) {
    unsafe {
        loop {
            let this = count;
            count -= 1;
            if this == 0 {
                break;
            }
            findsent(FORWARD, 1);
            if at_start_sent {
                find_first_blank(&raw mut (*curwin.get()).w_cursor);
            }
            if count == 0 || at_start_sent {
                decl(&raw mut (*curwin.get()).w_cursor);
            }
            at_start_sent = !at_start_sent;
        }
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
    unsafe {
        if lt(start_pos, VIsual.get()) {
            // The cursor is at the start of the Visual area. Work out where
            // that is: in the white space before a sentence, inside one or
            // just after it, or exactly at the start of one.
            let mut at_start_sent = true;
            decl(&raw mut pos);
            while lt(pos, (*curwin.get()).w_cursor) {
                if !ascii_iswhite(gchar_pos(&raw mut pos)) {
                    at_start_sent = false;
                    break;
                }
                incl(&raw mut pos);
            }
            if !at_start_sent {
                findsent(BACKWARD, 1);
                if equalpos((*curwin.get()).w_cursor, start_pos) {
                    at_start_sent = true; // exactly at the start of a sentence
                } else {
                    // Inside a sentence: go to its end, the next one's start.
                    findsent(FORWARD, 1);
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
                    find_first_blank(&raw mut (*curwin.get()).w_cursor);
                }
                let c = gchar_cursor();
                if !at_start_sent || (!include && !ascii_iswhite(c)) {
                    findsent(BACKWARD, 1);
                }
                at_start_sent = !at_start_sent;
            }
        } else {
            // The cursor is at the end of the Visual area: just before a
            // sentence, in or just before the white space in front of one, or
            // inside one.
            incl(&raw mut pos);
            let mut at_start_sent = true;
            if !equalpos(pos, (*curwin.get()).w_cursor) {
                // Not just before a sentence.
                at_start_sent = false;
                while lt(pos, (*curwin.get()).w_cursor) {
                    if !ascii_iswhite(gchar_pos(&raw mut pos)) {
                        at_start_sent = true;
                        break;
                    }
                    incl(&raw mut pos);
                }
                if at_start_sent {
                    findsent(BACKWARD, 1); // inside the sentence
                } else {
                    (*curwin.get()).w_cursor = start_pos; // in the white space
                }
            }
            if include {
                count *= 2; // `as` gets twice as much as `is`
            }
            findsent_forward(count, at_start_sent);
            if *p_sel.get() as c_int == 'e' as c_int {
                (*curwin.get()).w_cursor.col += 1;
            }
        }
    }
}

/// `is` / `as`: the sentence(s) under the cursor, cursor left at the end. In
/// Visual mode an existing selection is extended by one or more sentences
/// instead.
///
/// # Safety
/// `oap` must be a live operator argument, and there must be a current line.
pub unsafe extern "C" fn current_sent(oap: *mut oparg_T, count: c_int, include: bool) -> c_int {
    unsafe {
        let mut start_pos = (*curwin.get()).w_cursor;
        let mut pos = start_pos;
        findsent(FORWARD, 1); // the start of the next sentence

        // A Visual area bigger than one character is extended, not replaced.
        if VIsual_active.get() && !equalpos(start_pos, VIsual.get()) {
            extend_sentences(count, include, start_pos, pos);
            return OK;
        }

        // The cursor started on a blank: is it just before the start of the
        // next sentence?
        while ascii_iswhite(gchar_pos(&raw mut pos)) {
            incl(&raw mut pos);
        }
        let start_blank = equalpos(pos, (*curwin.get()).w_cursor);
        if start_blank {
            find_first_blank(&raw mut start_pos); // back to the first blank
        } else {
            findsent(BACKWARD, 1);
            start_pos = (*curwin.get()).w_cursor;
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
            decl(&raw mut (*curwin.get()).w_cursor);
        }

        if include {
            // With the blank in front of the sentence included, leave the
            // blanks at the end out: go back to the first of them. When there
            // are none, take the leading blanks instead.
            if start_blank {
                find_first_blank(&raw mut (*curwin.get()).w_cursor);
                if ascii_iswhite(gchar_pos(&raw mut (*curwin.get()).w_cursor)) {
                    decl(&raw mut (*curwin.get()).w_cursor);
                }
            } else if !ascii_iswhite(gchar_cursor()) {
                find_first_blank(&raw mut start_pos);
            }
        }

        if VIsual_active.get() {
            // Don't get stuck with `is` on a single space before a sentence.
            if equalpos(start_pos, (*curwin.get()).w_cursor) {
                extend_sentences(count, include, start_pos, pos);
                return OK;
            }
            if *p_sel.get() as c_int == 'e' as c_int {
                (*curwin.get()).w_cursor.col += 1;
            }
            VIsual.set(start_pos);
            VIsual_mode.set('v' as c_int);
            redraw_cmdline.set(true); // show the mode later
            redraw_curbuf_later(UPD_INVERTED); // update the inversion
        } else {
            // Include the newline after the sentence, if there is one.
            (*oap).inclusive = incl(&raw mut (*curwin.get()).w_cursor) == -1;
            (*oap).start = start_pos;
            (*oap).motion_type = kMTCharWise;
        }
        OK
    }
}
