//! The arrow keys, and the two that start a selection.
//!
//! Every one of these is a Normal-mode motion plus the same Insert-mode
//! bookkeeping around it: open a fold if 'foldopen' has `hor`
//! ([`may_open_fold_hor`]), take the `$` off the screen, close the undo block
//! with `start_arrow` -- passing the position the *insert* ended at, not the
//! one the cursor is going to -- and beep if the motion could not happen.
//!
//! Two things vary.  The four that can extend an undoable change
//! (`ins_left`, `ins_right` and their shifted forms) go through
//! `start_arrow_with_change`, because `i_CTRL-G_U` asks them not to break
//! the block and that has to be recorded for redo.  And the shifted forms
//! reach [`ins_start_select`] first, which turns the key into a Select-mode
//! selection when 'keymodel' contains `startsel`.
//!
//! [`ins_updown`] and [`ins_page`] are each one function with a direction:
//! upstream's four are the same body twice apiece.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win, first_tab};
use core::ffi::{c_char, c_int};

use super::*;
use crate::keycodes::{K_C_END, K_C_HOME};
use crate::types::{NUL, OK};

/// If 'keymodel' contains `startsel`, turn `c` into the start of a
/// Select-mode selection and stuff it back for Normal mode to handle.
///
/// Only the shifted keys do this, plus the un-shifted `<Home>`/`<End>`/page
/// keys when Shift is in `mod_mask`.  Answers whether a CTRL-O and the key
/// were stuffed.
pub(crate) fn ins_start_select(c: c_int) -> bool {
    if !km_startsel.get() {
        return false;
    }
    let starts = match c {
        K_KHOME | K_KEND | K_PAGEUP | K_KPAGEUP | K_PAGEDOWN | K_KPAGEDOWN => {
            mod_mask.get() & MOD_MASK_SHIFT != 0
        }
        K_S_LEFT | K_S_RIGHT | K_S_UP | K_S_DOWN | K_S_END | K_S_HOME => true,
        _ => false,
    };
    if !starts {
        return false;
    }

    start_selection();
    stuff_readbuf_char(Ctrl_O);
    if mod_mask.get() != 0 {
        // The modifiers have to be stuffed back too, as the three-byte
        // K_SPECIAL sequence that carries them.
        let buf: [c_char; 4] = [
            K_SPECIAL as c_char,
            KS_MODIFIER as c_char,
            mod_mask.get() as uint8_t as c_char,
            NUL as c_char,
        ];
        // SAFETY: `buf` is a live four-byte array and 3 of it is read.
        unsafe { stuff_readbuf_len(buf.as_ptr(), 3) };
    }
    stuff_readbuf_char(c);
    true
}

/// Open the fold under the cursor if 'foldopen' contains `hor` and the key
/// was typed rather than mapped.
fn may_open_fold_hor() {
    if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_uint != 0 && KeyTyped.get() {
        // SAFETY: `curwin` is live for the whole session.
        unsafe { fold_open_cursor() };
    }
}

/// Is the undoable change to be ended by this motion?
///
/// `i_CTRL-G_U` sets `dont_sync_undo` for exactly one motion, which is what
/// lets an insert survive an arrow key as a single undo block.
fn ends_change() -> bool {
    dont_sync_undo.get() == KeepUndo::No
}

/// `<Left>` in Insert mode.
pub(crate) fn ins_left() {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    let end_change = ends_change();
    may_open_fold_hor();
    hide_dollar();

    let mut tpos = cur_win().w_cursor;
    if unsafe { oneleft() } == OK {
        start_arrow_changing(&mut tpos, end_change);
        if !end_change {
            append_to_redobuff_char(K_LEFT);
        }
        // Only the characters 'revins' itself put there are legal to go
        // back over.
        if revins_scol.get() != -1 && cur_win().w_cursor.col >= revins_scol.get() {
            revins_legal.set(revins_legal.get() + 1);
        }
        revins_chars.set(revins_chars.get() + 1);
    } else if !unsafe { vim_strchr(p_ww.get(), '[' as c_int) }.is_null()
        && cur_win().w_cursor.lnum > 1
    {
        // 'whichwrap' allows the motion to leave the line.
        start_arrow_at(&mut tpos);
        cur_win().w_cursor.lnum -= 1;
        coladvance_to(MAXCOL as c_int);
        cur_win().w_set_curswant = true;
    } else {
        beep_cursor();
    }
    dont_sync_undo.set(KeepUndo::No);
}

/// `<Home>`, and `<C-Home>` -- which goes to the first line first.
pub(crate) fn ins_home(c: c_int) {
    may_open_fold_hor();
    hide_dollar();

    let mut tpos = cur_win().w_cursor;
    if c == K_C_HOME {
        cur_win().w_cursor.lnum = 1;
    }
    cur_win().w_cursor.col = 0;
    cur_win().w_cursor.coladd = 0;
    cur_win().w_curswant = 0;
    start_arrow_at(&mut tpos);
}

/// `<End>`, and `<C-End>` -- which goes to the last line first.
pub(crate) fn ins_end(c: c_int) {
    may_open_fold_hor();
    hide_dollar();

    let mut tpos = cur_win().w_cursor;
    if c == K_C_END {
        cur_win().w_cursor.lnum = cur_buf().b_ml.ml_line_count;
    }
    coladvance_to(MAXCOL as c_int);
    cur_win().w_curswant = MAXCOL as colnr_T;
    start_arrow_at(&mut tpos);
}

/// `<S-Left>`: one word back.
pub(crate) fn ins_s_left() {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    let end_change = ends_change();
    may_open_fold_hor();
    hide_dollar();

    if cur_win().w_cursor.lnum > 1 || cur_win().w_cursor.col > 0 {
        start_arrow_changing(&mut cur_win().w_cursor, end_change);
        if !end_change {
            append_to_redobuff_char(K_S_LEFT);
        }
        unsafe { bck_word(1, false, false) };
        cur_win().w_set_curswant = true;
    } else {
        beep_cursor();
    }
    dont_sync_undo.set(KeepUndo::No);
}

/// `<Right>` in Insert mode.
pub(crate) fn ins_right() {
    let end_change = ends_change();
    may_open_fold_hor();
    hide_dollar();

    if gchar_cursor() != NUL || virtual_active(cur_win()) {
        start_arrow_changing(&mut cur_win().w_cursor, end_change);
        if !end_change {
            append_to_redobuff_char(K_RIGHT);
        }
        cur_win().w_set_curswant = true;
        if virtual_active(cur_win()) {
            unsafe { oneright() };
        } else {
            // SAFETY: the cursor is on a character of its line, so the
            // character there has a length.
            cur_win().w_cursor.col += unsafe { utfc_ptr2len(get_cursor_pos_ptr()) };
        }

        revins_legal.set(revins_legal.get() + 1);
        if revins_chars.get() != 0 {
            revins_chars.set(revins_chars.get() - 1);
        }
    } else if !unsafe { vim_strchr(p_ww.get(), ']' as c_int) }.is_null()
        && cur_win().w_cursor.lnum < cur_buf().b_ml.ml_line_count
    {
        // 'whichwrap' allows the motion to leave the line.
        start_arrow_at(&mut cur_win().w_cursor);
        cur_win().w_set_curswant = true;
        cur_win().w_cursor.lnum += 1;
        cur_win().w_cursor.col = 0;
    } else {
        beep_cursor();
    }
    dont_sync_undo.set(KeepUndo::No);
}

/// `<S-Right>`: one word forward.
pub(crate) fn ins_s_right() {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    // The strings walked below are NUL-terminated lines of that buffer, and
    // every step stops at the NUL.
    let end_change = ends_change();
    may_open_fold_hor();
    hide_dollar();

    if cur_win().w_cursor.lnum < cur_buf().b_ml.ml_line_count || gchar_cursor() != NUL {
        start_arrow_changing(&mut cur_win().w_cursor, end_change);
        if !end_change {
            append_to_redobuff_char(K_S_RIGHT);
        }
        unsafe { fwd_word(1, false, false) };
        cur_win().w_set_curswant = true;
    } else {
        beep_cursor();
    }
    dont_sync_undo.set(KeepUndo::No);
}

/// `<Up>` and `<Down>` in Insert mode.
///
/// With `startcol`, the cursor goes to the column the *insert* started at
/// rather than the one it is in now -- that is `<C-Up>`/`<C-Down>`.  The
/// `w_topline`/`w_topfill` check is because the motion may have scrolled the
/// window even when the cursor stayed in view.
pub(crate) fn ins_updown(up: bool, startcol: bool) {
    let old_topline = cur_win().w_topline;
    let old_topfill = cur_win().w_topfill;
    hide_dollar();

    let mut tpos = cur_win().w_cursor;
    // SAFETY: `curwin` is live, which is all a cursor motion asks for.
    let moved = if up {
        unsafe { cursor_up(1, true) }
    } else {
        unsafe { cursor_down(1, true) }
    };
    if moved == OK {
        if startcol {
            // `getvcol_nolist` only reads: a copy keeps the global out
            // of the call's reach.
            // SAFETY: `Insstart` is a live position in the current buffer.
            coladvance_to(unsafe { getvcol_nolist(&mut Insstart.get()) });
        }
        if old_topline != cur_win().w_topline || old_topfill != cur_win().w_topfill {
            unsafe { redraw_later(curwin.get(), UPD_VALID) };
        }
        start_arrow_at(&mut tpos);
        can_cindent.set(true);
    } else {
        beep_cursor();
    }
}

/// `<PageUp>` and `<PageDown>` in Insert mode -- or, with CTRL, the previous
/// and next tab page.
pub(crate) fn ins_page(back: bool) {
    hide_dollar();

    if mod_mask.get() & MOD_MASK_CTRL != 0 {
        // <C-PageUp>/<C-PageDown>: another tab page, if there is one.
        if first_tab().is_some_and(|tp| tp.next().is_some()) {
            start_arrow_at(&mut cur_win().w_cursor);
            goto_tabpage(if back { -1 } else { 0 });
        }
        return;
    }

    let mut tpos = cur_win().w_cursor;
    let dir = if back { BACKWARD } else { FORWARD };
    if unsafe { pagescroll(dir, 1, false) } == OK {
        start_arrow_at(&mut tpos);
        can_cindent.set(true);
    } else {
        beep_cursor();
    }
}

/// Beep, or flash, for a motion that could not go anywhere.
#[inline(always)]
fn beep_cursor() {
    // SAFETY: the bell only reads options.
    unsafe { vim_beep(kOptBoFlagCursor as ::core::ffi::c_uint) }
}

/// Take the `$` 'cpoptions' puts at the end of a change off the screen.
#[inline(always)]
fn hide_dollar() {
    // SAFETY: `curwin` is live for the whole session.
    unsafe { undisplay_dollar() }
}

/// End the undoable insert before an arrow key moves the cursor away from
/// `pos`, the position the insert ended at.
#[inline(always)]
fn start_arrow_at(pos: &mut pos_T) {
    // SAFETY: `pos` is a live position, and `curbuf` is live.
    unsafe { start_arrow(pos) }
}

/// [`start_arrow_at`], with `i_CTRL-G_U`'s answer for whether the change
/// ends here too.
#[inline(always)]
fn start_arrow_changing(pos: &mut pos_T, end_change: bool) {
    // SAFETY: `pos` is a live position, and `curbuf` is live.
    unsafe { start_arrow_with_change(pos, end_change) }
}

/// Move the cursor to virtual column `vcol` of its line.
#[inline(always)]
fn coladvance_to(vcol: c_int) {
    // SAFETY: `curwin` is live for the whole session.
    coladvance(unsafe { Win::current() }, vcol);
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
