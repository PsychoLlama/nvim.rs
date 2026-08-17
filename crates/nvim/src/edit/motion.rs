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

use ::core::ffi::{c_char, c_int};

use super::*;
use crate::keycodes::{K_C_END, K_C_HOME};

/// If 'keymodel' contains `startsel`, turn `c` into the start of a
/// Select-mode selection and stuff it back for Normal mode to handle.
///
/// Only the shifted keys do this, plus the un-shifted `<Home>`/`<End>`/page
/// keys when Shift is in `mod_mask`.  Answers whether a CTRL-O and the key
/// were stuffed.
///
/// # Safety
/// Must run on the main thread.
pub(crate) unsafe fn ins_start_select(c: c_int) -> bool {
    unsafe {
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
        stuffcharReadbuff(Ctrl_O);
        if mod_mask.get() != 0 {
            // The modifiers have to be stuffed back too, as the three-byte
            // K_SPECIAL sequence that carries them.
            let buf: [c_char; 4] = [
                K_SPECIAL as c_char,
                KS_MODIFIER as c_char,
                mod_mask.get() as uint8_t as c_char,
                NUL as c_char,
            ];
            stuffReadbuffLen(buf.as_ptr(), 3);
        }
        stuffcharReadbuff(c);
        true
    }
}

/// Open the fold under the cursor if 'foldopen' contains `hor` and the key
/// was typed rather than mapped.
///
/// # Safety
/// Must run with a live `curwin`.
unsafe fn may_open_fold_hor() {
    unsafe {
        if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_uint != 0 && KeyTyped.get() {
            foldOpenCursor();
        }
    }
}

/// Is the undoable change to be ended by this motion?
///
/// `i_CTRL-G_U` sets `dont_sync_undo` for exactly one motion, which is what
/// lets an insert survive an arrow key as a single undo block.
fn ends_change() -> bool {
    dont_sync_undo.get() == kFalse
}

/// `<Left>` in Insert mode.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn ins_left() {
    unsafe {
        let end_change = ends_change();
        may_open_fold_hor();
        undisplay_dollar();

        let mut tpos = (*curwin.get()).w_cursor;
        if oneleft() == OK {
            start_arrow_with_change(&raw mut tpos, end_change);
            if !end_change {
                AppendCharToRedobuff(K_LEFT);
            }
            // Only the characters 'revins' itself put there are legal to go
            // back over.
            if revins_scol.get() != -1 && (*curwin.get()).w_cursor.col >= revins_scol.get() {
                (*revins_legal.ptr()) += 1;
            }
            (*revins_chars.ptr()) += 1;
        } else if !vim_strchr(p_ww.get(), '[' as c_int).is_null()
            && (*curwin.get()).w_cursor.lnum > 1
        {
            // 'whichwrap' allows the motion to leave the line.
            start_arrow(&raw mut tpos);
            (*curwin.get()).w_cursor.lnum -= 1;
            coladvance(curwin.get(), MAXCOL as c_int);
            (*curwin.get()).w_set_curswant = true_0;
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_uint);
        }
        dont_sync_undo.set(kFalse);
    }
}

/// `<Home>`, and `<C-Home>` -- which goes to the first line first.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn ins_home(c: c_int) {
    unsafe {
        may_open_fold_hor();
        undisplay_dollar();

        let mut tpos = (*curwin.get()).w_cursor;
        if c == K_C_HOME {
            (*curwin.get()).w_cursor.lnum = 1;
        }
        (*curwin.get()).w_cursor.col = 0;
        (*curwin.get()).w_cursor.coladd = 0;
        (*curwin.get()).w_curswant = 0;
        start_arrow(&raw mut tpos);
    }
}

/// `<End>`, and `<C-End>` -- which goes to the last line first.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
pub(crate) unsafe fn ins_end(c: c_int) {
    unsafe {
        may_open_fold_hor();
        undisplay_dollar();

        let mut tpos = (*curwin.get()).w_cursor;
        if c == K_C_END {
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
        }
        coladvance(curwin.get(), MAXCOL as c_int);
        (*curwin.get()).w_curswant = MAXCOL as colnr_T;
        start_arrow(&raw mut tpos);
    }
}

/// `<S-Left>`: one word back.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn ins_s_left() {
    unsafe {
        let end_change = ends_change();
        may_open_fold_hor();
        undisplay_dollar();

        if (*curwin.get()).w_cursor.lnum > 1 || (*curwin.get()).w_cursor.col > 0 {
            start_arrow_with_change(&raw mut (*curwin.get()).w_cursor, end_change);
            if !end_change {
                AppendCharToRedobuff(K_S_LEFT);
            }
            bck_word(1, false, false);
            (*curwin.get()).w_set_curswant = true_0;
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_uint);
        }
        dont_sync_undo.set(kFalse);
    }
}

/// `<Right>` in Insert mode.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
pub(crate) unsafe fn ins_right() {
    unsafe {
        let end_change = ends_change();
        may_open_fold_hor();
        undisplay_dollar();

        if gchar_cursor() != NUL || virtual_active(curwin.get()) {
            start_arrow_with_change(&raw mut (*curwin.get()).w_cursor, end_change);
            if !end_change {
                AppendCharToRedobuff(K_RIGHT);
            }
            (*curwin.get()).w_set_curswant = true_0;
            if virtual_active(curwin.get()) {
                oneright();
            } else {
                (*curwin.get()).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
            }

            (*revins_legal.ptr()) += 1;
            if revins_chars.get() != 0 {
                (*revins_chars.ptr()) -= 1;
            }
        } else if !vim_strchr(p_ww.get(), ']' as c_int).is_null()
            && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
        {
            // 'whichwrap' allows the motion to leave the line.
            start_arrow(&raw mut (*curwin.get()).w_cursor);
            (*curwin.get()).w_set_curswant = true_0;
            (*curwin.get()).w_cursor.lnum += 1;
            (*curwin.get()).w_cursor.col = 0;
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_uint);
        }
        dont_sync_undo.set(kFalse);
    }
}

/// `<S-Right>`: one word forward.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
pub(crate) unsafe fn ins_s_right() {
    unsafe {
        let end_change = ends_change();
        may_open_fold_hor();
        undisplay_dollar();

        if (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
            || gchar_cursor() != NUL
        {
            start_arrow_with_change(&raw mut (*curwin.get()).w_cursor, end_change);
            if !end_change {
                AppendCharToRedobuff(K_S_RIGHT);
            }
            fwd_word(1, false, false);
            (*curwin.get()).w_set_curswant = true_0;
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_uint);
        }
        dont_sync_undo.set(kFalse);
    }
}

/// `<Up>` and `<Down>` in Insert mode.
///
/// With `startcol`, the cursor goes to the column the *insert* started at
/// rather than the one it is in now -- that is `<C-Up>`/`<C-Down>`.  The
/// `w_topline`/`w_topfill` check is because the motion may have scrolled the
/// window even when the cursor stayed in view.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn ins_updown(up: bool, startcol: bool) {
    unsafe {
        let old_topline = (*curwin.get()).w_topline;
        let old_topfill = (*curwin.get()).w_topfill;
        undisplay_dollar();

        let mut tpos = (*curwin.get()).w_cursor;
        let moved = if up {
            cursor_up(1, true)
        } else {
            cursor_down(1, true)
        };
        if moved == OK {
            if startcol {
                coladvance(curwin.get(), getvcol_nolist(Insstart.ptr()));
            }
            if old_topline != (*curwin.get()).w_topline || old_topfill != (*curwin.get()).w_topfill
            {
                redraw_later(curwin.get(), UPD_VALID);
            }
            start_arrow(&raw mut tpos);
            can_cindent.set(true);
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_uint);
        }
    }
}

/// `<PageUp>` and `<PageDown>` in Insert mode -- or, with CTRL, the previous
/// and next tab page.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn ins_page(back: bool) {
    unsafe {
        undisplay_dollar();

        if mod_mask.get() & MOD_MASK_CTRL != 0 {
            // <C-PageUp>/<C-PageDown>: another tab page, if there is one.
            if !(*first_tabpage.get()).tp_next.is_null() {
                start_arrow(&raw mut (*curwin.get()).w_cursor);
                goto_tabpage(if back { -1 } else { 0 });
            }
            return;
        }

        let mut tpos = (*curwin.get()).w_cursor;
        let dir = if back { BACKWARD } else { FORWARD };
        if pagescroll(dir, 1, false) == OK {
            start_arrow(&raw mut tpos);
            can_cindent.set(true);
        } else {
            vim_beep(kOptBoFlagCursor as ::core::ffi::c_uint);
        }
    }
}
