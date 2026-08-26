//! 'formatoptions' `a`: reformatting the paragraph as it is edited.
//!
//! [`auto_format`] runs after nearly every change in Insert mode, decides
//! whether the paragraph wants reflowing at all, and hands the work to
//! `format_lines`. The space it may add under the cursor so that a
//! part-typed word still ends a paragraph is `did_add_space`, and
//! [`check_auto_format`] is what takes it away again.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

use super::*;
use crate::change::{del_char, get_leader_len};
use crate::cursor::{
    check_cursor, check_cursor_col, coladvance, dec_cursor, gchar_cursor, get_cursor_line_len,
    get_cursor_line_ptr, inc_cursor,
};
use crate::global_cell::GlobalCell;
use crate::main::{State, curwin, saved_cursor};
use crate::memline::ml_replace;
use crate::pos::MAXCOL;
use crate::state::MODE_INSERT;
use crate::strings::xstrnsave;
use crate::types::{FAIL, NUL, size_t};
use crate::undo::u_save_cursor;

/// `auto_format` added an extra space under the cursor, and it has to come
/// back off.
static did_add_space: GlobalCell<bool> = GlobalCell::new(false);

/// Reformat from the current line to the end of the paragraph, keeping the
/// cursor where it is relative to the text. Called after nearly every insert
/// or delete when 'formatoptions' has `a`.
///
/// `trailblank` allows formatting with a trailing blank; `prev_line` allows
/// it to start one line earlier, so that after an `x` a word can move back up
/// if it now fits.
///
/// The caller must have saved the cursor line for undo; the lines after it
/// are saved here.
///
/// # Safety
/// There must be a current line, and it must be modifiable.
pub unsafe fn auto_format(trailblank: bool, prev_line: bool) {
    if !has_format_option(FoFlag::AUTO) {
        return;
    }

    let pos = cur_win().w_cursor;
    let old = get_cursor_line_ptr();

    // May remove an added space.
    unsafe { check_auto_format(false) };

    // Don't format in Insert mode when the cursor is on a trailing blank:
    // the user may be about to type ordinary text. Skip it too when `1`
    // is in 'formatoptions' and there is a single character before the
    // cursor -- otherwise the line is broken, and typing another
    // non-white character does not join it back together.
    let wasatend = pos.col == get_cursor_line_len();
    if unsafe { *old } as c_int != NUL && !trailblank && wasatend {
        dec_cursor();
        let mut cc = gchar_cursor();
        if !unsafe { whitechar(cc) }
            && cur_win().w_cursor.col > 0
            && has_format_option(FoFlag::ONE_LETTER)
        {
            dec_cursor();
        }
        cc = gchar_cursor();
        if unsafe { whitechar(cc) } {
            cur_win().w_cursor = pos;
            return;
        }
        cur_win().w_cursor = pos;
    }

    // Skip it as well when white space was just typed in the middle of a
    // line. Reformatting would join the paragraph and re-wrap it, and
    // `OPENLINE_DELSPACES` would eat that space at the break. Deferring
    // means the next non-white character lands next to the space, which
    // protects it, and the keystroke after that reformats properly.
    if unsafe { *old } as c_int != NUL
        && !trailblank
        && !wasatend
        && pos.col > 0
        && State.get() & MODE_INSERT != 0
    {
        let line = get_cursor_line_ptr();
        // Note the argument: `WHITECHAR` tests `ascii_iswhite` on what it
        // is given but the composing check at the *cursor*, which is one
        // byte further along than the byte named here.
        if unsafe { whitechar(*line.offset(pos.col as isize - 1) as c_int) } {
            cur_win().w_cursor = pos;
            return;
        }
    }

    // With `c` in 'formatoptions' and `t` missing, only comments format.
    if has_format_option(FoFlag::WRAP_COMS)
        && !has_format_option(FoFlag::WRAP)
        && unsafe { get_leader_len(old, ::core::ptr::null_mut::<*mut c_char>(), false, true) } == 0
    {
        return;
    }

    // May start one line earlier, but not at the start of a paragraph.
    if prev_line && !unsafe { paragraph_start(cur_win().w_cursor.lnum) } {
        cur_win().w_cursor.lnum -= 1;
        if u_save_cursor() == FAIL {
            return;
        }
    }

    // Format, then restore the cursor: `saved_cursor` is adjusted by the
    // formatting as the text moves under it.
    saved_cursor.set(pos);
    unsafe { format_lines(-1, false) };
    cur_win().w_cursor = saved_cursor.get();
    saved_cursor.set(saved_cursor.get().with_lnum(0));

    if cur_win().w_cursor.lnum > cur_buf().b_ml.ml_line_count {
        // "cannot happen"
        cur_win().w_cursor.lnum = cur_buf().b_ml.ml_line_count;
        unsafe { coladvance(curwin.get(), MAXCOL) };
    } else {
        unsafe { check_cursor_col(curwin.get()) };
    }

    // Insert mode: the cursor being past the end of the line when it was
    // not before means the line was broken. Because of the trailing-blank
    // rule above, `w` in 'formatoptions' then needs a space added to keep
    // the paragraph formatted.
    if !wasatend && has_format_option(FoFlag::WHITE_PAR) {
        let linep = get_cursor_line_ptr();
        let len = get_cursor_line_len();
        if cur_win().w_cursor.col == len {
            let plinep = unsafe { xstrnsave(linep, len as size_t + 2) };
            unsafe { *plinep.offset(len as isize) = ' ' as c_char };
            unsafe { *plinep.offset(len as isize + 1) = NUL as c_char };
            unsafe { ml_replace(cur_win().w_cursor.lnum, plinep, false) };
            // Remove the space later.
            did_add_space.set(true);
        } else {
            // May remove an added space.
            unsafe { check_auto_format(false) };
        }
    }

    unsafe { check_cursor(curwin.get()) };
}

/// Delete the space [`auto_format`] added to continue a paragraph, if it is
/// still there. It must be under the cursor, just after the insert position.
///
/// `end_insert` says Insert mode is ending, in which case the space counts as
/// trailing and goes whatever follows it.
///
/// # Safety
/// There must be a current line, and it must be modifiable.
pub unsafe fn check_auto_format(end_insert: bool) {
    if !did_add_space.get() {
        return;
    }
    let cc = gchar_cursor();
    if !unsafe { whitechar(cc) } {
        // Somehow the space was removed already.
        did_add_space.set(false);
        return;
    }
    let mut c = ' ' as c_int;
    if !end_insert {
        inc_cursor();
        c = gchar_cursor();
        dec_cursor();
    }
    if c != NUL {
        // The space is no longer at the end of the line: delete it.
        unsafe { del_char(false) };
        did_add_space.set(false);
    }
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
