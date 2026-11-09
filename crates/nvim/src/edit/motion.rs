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

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::keycodes::ModMask;
use crate::keycodes::SpecialKey;
use crate::mbyte::c_cluster_len;
use crate::types::Key;
use crate::winlayer::{Buf, Win, first_tab};
use core::ffi::c_int;

use super::*;
use crate::ascii::NUL;

/// If 'keymodel' contains `startsel`, turn `c` into the start of a
/// Select-mode selection and stuff it back for Normal mode to handle.
///
/// Only the shifted keys do this, plus the un-shifted `<Home>`/`<End>`/page
/// keys when Shift is in `mod_mask`.  Answers whether a CTRL-O and the key
/// were stuffed.
pub(crate) fn ins_start_select(c: Key) -> bool {
    if !km_startsel.get() {
        return false;
    }
    let starts = match c.special() {
        Some(
            SpecialKey::Khome
            | SpecialKey::Kend
            | SpecialKey::Pageup
            | SpecialKey::Kpageup
            | SpecialKey::Pagedown
            | SpecialKey::Kpagedown,
        ) => MOD_MASK.get().has(ModMask::SHIFT),
        Some(
            SpecialKey::SLeft
            | SpecialKey::SRight
            | SpecialKey::SUp
            | SpecialKey::SDown
            | SpecialKey::SEnd
            | SpecialKey::SHome,
        ) => true,
        _ => false,
    };
    if !starts {
        return false;
    }

    start_selection();
    stuff_readbuf_char(Key::ctrl(b'O'));
    if !MOD_MASK.get().is_empty() {
        // The modifiers have to be stuffed back too, as the three-byte
        // K_SPECIAL sequence that carries them.
        let buf: [u8; 3] = [
            K_SPECIAL.to_le_bytes()[0],
            KS_MODIFIER.to_le_bytes()[0],
            MOD_MASK.get().bits().to_le_bytes()[0],
        ];
        stuff_readbuf_len(&buf);
    }
    stuff_readbuf_char(c);
    true
}

/// Open the fold under the cursor if 'foldopen' contains `hor` and the key
/// was typed rather than mapped.
fn may_open_fold_hor() {
    if FDO_FLAGS.get() & kOptFdoFlagHor as ::core::ffi::c_uint != 0 && KEY_TYPED.get() {
        fold_open_cursor();
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
    let end_change = ends_change();
    may_open_fold_hor();
    hide_dollar();

    let tpos = Win::current().w_cursor;
    if oneleft().is_ok() {
        start_arrow_changing(InsertEnd::At(tpos), end_change);
        if !end_change {
            append_to_redobuff_char(Key::from(SpecialKey::Left));
        }
        // Only the characters 'revins' itself put there are legal to go
        // back over.
        if revins_scol.get() != -1 && Win::current().w_cursor.col >= ColNr::new(revins_scol.get()) {
            revins_legal.set(revins_legal.get() + 1);
        }
        revins_chars.set(revins_chars.get() + 1);
    } else if p_ww(|value| value.to_bytes().contains(&b'['))
        && Win::current().w_cursor.lnum > LineNr::ONE
    {
        // 'whichwrap' allows the motion to leave the line.
        start_arrow_at(InsertEnd::At(tpos));
        Win::current().w_cursor.lnum -= 1;
        coladvance_to(MAXCOL);
        Win::current().w_set_curswant = true;
    } else {
        beep_cursor();
    }
    dont_sync_undo.set(KeepUndo::No);
}

/// `<Home>`, and `<C-Home>` -- which goes to the first line first.
pub(crate) fn ins_home(c: Key) {
    may_open_fold_hor();
    hide_dollar();

    let tpos = Win::current().w_cursor;
    if c == SpecialKey::CHome {
        Win::current().w_cursor.lnum = LineNr::ONE;
    }
    Win::current().w_cursor.col = ColNr::ZERO;
    Win::current().w_cursor.coladd = 0;
    Win::current().w_curswant = ColNr::ZERO;
    start_arrow_at(InsertEnd::At(tpos));
}

/// `<End>`, and `<C-End>` -- which goes to the last line first.
pub(crate) fn ins_end(c: Key) {
    may_open_fold_hor();
    hide_dollar();

    let tpos = Win::current().w_cursor;
    if c == SpecialKey::CEnd {
        Win::current().w_cursor.lnum = Buf::current().b_ml.ml_line_count;
    }
    coladvance_to(MAXCOL);
    Win::current().w_curswant = ColNr::MAX;
    start_arrow_at(InsertEnd::At(tpos));
}

/// `<S-Left>`: one word back.
pub(crate) fn ins_s_left() {
    let end_change = ends_change();
    may_open_fold_hor();
    hide_dollar();

    if Win::current().w_cursor.lnum > LineNr::ONE || Win::current().w_cursor.col > ColNr::ZERO {
        start_arrow_changing(InsertEnd::cursor(), end_change);
        if !end_change {
            append_to_redobuff_char(Key::from(SpecialKey::SLeft));
        }
        let _ = bck_word(1, false, false);
        Win::current().w_set_curswant = true;
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

    if gchar_cursor() != NUL || virtual_active(Win::current()) {
        start_arrow_changing(InsertEnd::cursor(), end_change);
        if !end_change {
            append_to_redobuff_char(Key::from(SpecialKey::Right));
        }
        Win::current().w_set_curswant = true;
        if virtual_active(Win::current()) {
            let _ = oneright();
        } else {
            let cursor = Win::current().w_cursor;
            let step = {
                let mut lines = Lines::current();
                let line = lines.line(cursor.lnum);
                c_cluster_len(
                    &line[usize::try_from(cursor.col)
                        .unwrap_or(usize::MAX)
                        .min(line.len())..],
                )
            };
            Win::current().w_cursor.col += ColNr::try_from(step).unwrap_or(ColNr::MAX).get();
        }

        revins_legal.set(revins_legal.get() + 1);
        if revins_chars.get() != 0 {
            revins_chars.set(revins_chars.get() - 1);
        }
    } else if p_ww(|value| value.to_bytes().contains(&b']'))
        && Win::current().w_cursor.lnum < Buf::current().b_ml.ml_line_count
    {
        // 'whichwrap' allows the motion to leave the line.
        start_arrow_at(InsertEnd::cursor());
        Win::current().w_set_curswant = true;
        Win::current().w_cursor.lnum += 1;
        Win::current().w_cursor.col = ColNr::ZERO;
    } else {
        beep_cursor();
    }
    dont_sync_undo.set(KeepUndo::No);
}

/// `<S-Right>`: one word forward.
pub(crate) fn ins_s_right() {
    // The strings walked below are NUL-terminated lines of that buffer, and
    // every step stops at the NUL.
    let end_change = ends_change();
    may_open_fold_hor();
    hide_dollar();

    if Win::current().w_cursor.lnum < Buf::current().b_ml.ml_line_count || gchar_cursor() != NUL {
        start_arrow_changing(InsertEnd::cursor(), end_change);
        if !end_change {
            append_to_redobuff_char(Key::from(SpecialKey::SRight));
        }
        let _ = fwd_word(1, false, false);
        Win::current().w_set_curswant = true;
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
    let old_topline = Win::current().w_topline;
    let old_topfill = Win::current().w_topfill;
    hide_dollar();

    let tpos = Win::current().w_cursor;
    let moved = if up {
        cursor_up(1, true)
    } else {
        cursor_down(1, true)
    };
    if moved.is_ok() {
        if startcol {
            // `getvcol_nolist` only reads: a copy keeps the global out
            // of the call's reach.
            coladvance_to(getvcol_nolist(&mut Insstart.get()).get());
        }
        if old_topline != Win::current().w_topline || old_topfill != Win::current().w_topfill {
            redraw_later(Win::current(), UPD_VALID);
        }
        start_arrow_at(InsertEnd::At(tpos));
        can_cindent.set(true);
    } else {
        beep_cursor();
    }
}

/// `<PageUp>` and `<PageDown>` in Insert mode -- or, with CTRL, the previous
/// and next tab page.
pub(crate) fn ins_page(back: bool) {
    hide_dollar();

    if MOD_MASK.get().has(ModMask::CTRL) {
        // <C-PageUp>/<C-PageDown>: another tab page, if there is one.
        if first_tab().is_some_and(|tp| tp.next().is_some()) {
            start_arrow_at(InsertEnd::cursor());
            goto_tabpage(if back { -1 } else { 0 });
        }
        return;
    }

    let tpos = Win::current().w_cursor;
    let dir = if back { BACKWARD } else { FORWARD };
    if pagescroll(dir, 1, false) {
        start_arrow_at(InsertEnd::At(tpos));
        can_cindent.set(true);
    } else {
        beep_cursor();
    }
}

/// Beep, or flash, for a motion that could not go anywhere.
#[inline(always)]
fn beep_cursor() {
    vim_beep(kOptBoFlagCursor as ::core::ffi::c_uint)
}

/// Take the `$` 'cpoptions' puts at the end of a change off the screen.
#[inline(always)]
fn hide_dollar() {
    undisplay_dollar()
}

/// End the undoable insert before an arrow key moves the cursor away from
/// `pos`, the position the insert ended at.
#[inline(always)]
fn start_arrow_at(pos: InsertEnd) {
    start_arrow(Some(pos));
}

/// [`start_arrow_at`], with `i_CTRL-G_U`'s answer for whether the change
/// ends here too.
#[inline(always)]
fn start_arrow_changing(pos: InsertEnd, end_change: bool) {
    start_arrow_with_change(Some(pos), end_change);
}

/// Move the cursor to virtual column `vcol` of its line.
#[inline(always)]
fn coladvance_to(vcol: c_int) {
    coladvance(Win::current(), ColNr::new(vcol));
}
