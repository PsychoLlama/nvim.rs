//! `gn` and `gN`: select the next match of the last search pattern.
//!
//! Used while an operator is pending and in Visual mode, so that `dgn`
//! deletes the next match and `gn` on its own extends the selection over
//! it. The work is two [`searchit`](super::searchit) calls in a row --
//! backwards then forwards -- so that a match under the cursor counts.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::normal::{
    VisualMode, set_visual_active, set_visual_anchor, set_visual_mode, visual_active,
    visual_anchor, with_visual_anchor,
};
use crate::regexp::RE_SEARCH;
use crate::search::{SEARCH_END, SEARCH_KEEP};
use crate::types::Failed;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;
use core::ptr;

/// Where the two searches [`current_search`] runs left the match.
struct Match {
    /// The start of the match; where `pos` ended up.
    start: pos_T,
    /// The end of the match.
    end: pos_T,
}

/// Find the match under or after the cursor, twice.
///
/// The trick is to first search backwards and then forwards again, so that
/// a match at the cursor position itself is captured; with `forward`
/// false it is the other way around. The first search may fail, in which
/// case the second starts from the far end of the buffer.
///
/// Answers `None` when the pattern is not there at all.
///
/// # Safety
/// The current window and buffer must be valid.
unsafe fn search_around(
    mut pos: pos_T,
    count: c_int,
    forward: bool,
    skip_first_backward: bool,
    zero_width: bool,
) -> Option<Match> {
    let old_p_ws = p_ws.get();
    let mut end_pos = pos_T::default();
    for i in 0..2 {
        if forward && i == 0 && skip_first_backward {
            continue;
        }
        let forwards = if forward { i != 0 } else { i == 0 };

        // Looking backwards for a pattern that is not zero-width, the
        // end of the match is what has to be recorded.
        let flags = if !forwards && !zero_width {
            SEARCH_END
        } else {
            0
        };
        end_pos = pos;

        // Wrapping should not occur in the first round.
        if i == 0 {
            p_ws.set(0);
        }
        let result = unsafe {
            searchit(
                Some(Win::current()),
                Buf::current(),
                &raw mut pos,
                &raw mut end_pos,
                if forwards { FORWARD } else { BACKWARD },
                last_used_pattern().pat,
                last_used_pattern().patlen,
                if i != 0 { count } else { 1 },
                SEARCH_KEEP | flags,
                RE_SEARCH,
                ptr::null_mut(),
            )
        };
        p_ws.set(old_p_ws);

        if result != 0 {
            continue;
        }
        if i == 1 {
            return None; // not found, abort
        }
        // The first search may fail; start again from the far end of
        // the buffer, because the cursor might be on the match. Not
        // done in Visual mode, so that extending the selection works.
        if forward {
            clearpos(&mut pos);
        } else {
            // Searching backwards, so start at the last line and col.
            let last = unsafe { (*cur_win().w_buffer).b_ml.ml_line_count };
            pos.lnum = last;
            pos.col = ml_get_len(last);
        }
    }
    Some(Match {
        start: pos,
        end: end_pos,
    })
}

/// Select the next match of the last search pattern, as `gn` and `gN`.
///
/// Used while an operator is pending and in Visual mode: the match becomes
/// the Visual area.
///
/// # Safety
/// The current window and buffer must be valid.
pub unsafe fn current_search(count: c_int, forward: bool) -> Result<(), Failed> {
    let save_visual = visual_anchor();

    // Correct the cursor when 'selection' is exclusive.
    if visual_active()
        && unsafe { *p_sel.get() } as c_int == 'e' as c_int
        && lt(visual_anchor(), cur_win().w_cursor)
    {
        dec_cursor();
    }

    // When searching forward and the cursor is at the start of the
    // Visual area, skip the first backward search, or it would not
    // move.
    let skip_first_backward = forward && visual_active() && lt(cur_win().w_cursor, visual_anchor());

    let orig_pos = cur_win().w_cursor; // where the cursor started
    let mut pos = orig_pos; // position after the pattern
    if visual_active() {
        // Searching further will extend the match.
        if forward {
            unsafe { incl(&mut pos) };
        } else {
            unsafe { decl(&mut pos) };
        }
    }

    // Is the pattern zero-width? This time, don't care about the
    // direction.
    let pat = last_used_pattern();
    let cursor = cur_win().cursor().raw();
    // SAFETY: the last search pattern and the live window's cursor.
    let zero_width = unsafe { is_zero_width(pat.pat, pat.patlen, true, cursor, FORWARD) };
    if zero_width == -1 {
        return Err(Failed); // pattern not found
    }

    let found = unsafe { search_around(pos, count, forward, skip_first_backward, zero_width != 0) };
    let Some(found) = found else {
        cur_win().w_cursor = orig_pos;
        if visual_active() {
            set_visual_anchor(save_visual);
        }
        return Err(Failed);
    };

    if !visual_active() {
        set_visual_anchor(found.start);
    }

    // Put the cursor after the match.
    cur_win().w_cursor = found.end;
    if lt(visual_anchor(), found.end) && forward {
        if skip_first_backward {
            // Put the cursor on the start of the match.
            cur_win().w_cursor = found.start;
        } else {
            // Put the cursor on the last character of the match.
            dec_cursor();
        }
    } else if visual_active() && lt(cur_win().w_cursor, visual_anchor()) && forward {
        cur_win().w_cursor = found.start;
    }
    set_visual_active(true);
    set_visual_mode(VisualMode::CHAR);

    if unsafe { *p_sel.get() } as c_int == 'e' as c_int {
        // Correction for exclusive selection depends on the direction.
        if forward && ltoreq(visual_anchor(), cur_win().w_cursor) {
            inc_cursor();
        } else if !forward && ltoreq(cur_win().w_cursor, visual_anchor()) {
            with_visual_anchor(|anchor| unsafe { inc(anchor) });
        }
    }

    if fdo_flags.get() & kOptFdoFlagSearch != 0 && KeyTyped.get() {
        unsafe { fold_open_cursor() };
    }

    may_start_select('c' as c_int);
    setmouse();
    redraw_curbuf_later(UPD_INVERTED);
    unsafe { showmode() };
    Ok(())
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
