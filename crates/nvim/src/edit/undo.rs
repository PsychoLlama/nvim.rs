//! Where one undoable insert ends and the next begins.
//!
//! Insert mode is one undo block from the first character typed to `<Esc>`
//! -- unless the cursor is *moved* in between, which starts a new one.
//! [`start_arrow`] is that: called by every motion key, it closes the current
//! block and remembers where the insert ended.  [`stop_arrow`] is the mirror,
//! called before any change, and is where the `u_save` for the block actually
//! happens (`INS_NEED_UNDO`).
//!
//! The pair is spelled with two flags rather than one state: `ARROW_USED`
//! means "the block is closed", `INS_NEED_UNDO` means "the next change still
//! owes a `u_save`", and `stop_arrow` answers `FAIL` when either is still
//! true afterwards, which is the caller's signal not to change the buffer at
//! all.
//!
//! [`stop_insert`] is the whole of leaving the mode: save the text for `.`,
//! auto-format if 'formatoptions' `a` asked for it, trim the auto-indent the
//! user never typed on, and set `'[`/`']`.  [`ins_apply_autocmds`] is here
//! because an autocommand may change the buffer, and that change needs an
//! undo block of its own.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::types::Key;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::ascii::NUL;
use crate::normal::visual_active;
use crate::option::cpo_has;
use crate::option::defs::{CpoFlag, FoFlag};
use crate::types::Failed;

/// Where an insert ended, as [`stop_insert`] reads it.
///
/// Upstream passes a `pos_T *`, and the callers point it either at a copy of
/// the position or at the window's own cursor -- which `stop_insert` then
/// sees move as it moves the cursor itself. The two are two arms here.
#[derive(Clone, Copy, Debug)]
pub(crate) enum InsertEnd {
    /// This position.
    At(Pos),
    /// `win`'s cursor, wherever it is each time it is read.
    CursorOf(Win),
}

impl InsertEnd {
    /// The current window's cursor, read live.
    pub(crate) fn cursor() -> Self {
        Self::CursorOf(Win::current())
    }

    fn pos(self) -> Pos {
        match self {
            Self::At(pos) => pos,
            Self::CursorOf(win) => win.w_cursor,
        }
    }
}

/// Called when an arrow key is used in Insert mode: for undo and redo it
/// resembles hitting `<Esc>`.
///
/// `end_insert_pos` is `None` when the insert's end is not known (another
/// buffer is current now).
pub(crate) fn start_arrow(end_insert_pos: Option<InsertEnd>) {
    start_arrow_common(end_insert_pos, true);
}

/// [`start_arrow`], but able to keep the undoable change open -- which is
/// `i_CTRL-G_U`, and is recorded so redo does the same.
pub(crate) fn start_arrow_with_change(end_insert_pos: Option<InsertEnd>, end_change: bool) {
    start_arrow_common(end_insert_pos, end_change);
    if !end_change {
        append_to_redobuff_char(Key::ctrl(b'G'));
        append_to_redobuff_char(Key::ascii(b'U'));
    }
}

fn start_arrow_common(end_insert_pos: Option<InsertEnd>, end_change: bool) {
    if !ARROW_USED.get() && end_change {
        // Something has been inserted: close the block.
        append_to_redobuff(ESC_STR.to_bytes());
        stop_insert(end_insert_pos, false, false);
        ARROW_USED.set(true);
    }
    check_spell_redraw();
}

/// Highlight the word at the cursor if that was skipped while typing.
///
/// It may be skipped again, so the line number is cleared first.
pub(crate) fn check_spell_redraw() {
    if SPELL_REDRAW_LNUM.get() != LineNr::ZERO {
        let lnum = SPELL_REDRAW_LNUM.get();
        SPELL_REDRAW_LNUM.set(LineNr::ZERO);
        redraw_win_line(Win::current(), lnum);
    }
}

/// Called before any change in Insert mode: if an arrow key was used, start a
/// new insertion here.
///
/// `Err` when undo is impossible, in which case the caller must not insert.
pub(crate) fn stop_arrow() -> Result<(), Failed> {
    if ARROW_USED.get() {
        INSSTART.set(Win::current().w_cursor); // new insertion starts here
        if INSSTART.get().col > INSSTART_ORIG.get().col && !INS_NEED_UNDO.get() {
            // Don't update the original insert position when moved to the
            // right, except when nothing was inserted yet.
            UPDATE_INSSTART_ORIG.set(false);
        }
        let mut lines = Lines::current();
        INSSTART_TEXTLEN.set(ColNr::new(linetabsize_str(
            lines.line(Win::current().w_cursor.lnum),
        )));

        if save_cursor_line().is_ok() {
            ARROW_USED.set(false);
            INS_NEED_UNDO.set(false);
        }
        AI_COL.set(ColNr::ZERO);
        if STATE.get() & VREPLACE_FLAG != 0 {
            ORIG_LINE_COUNT.set(Buf::current().b_ml.ml_line_count);
            VR_LINES_CHANGED.set(1);
        }
        reset_redobuff();
        append_to_redobuff(b"1i"); // pretend we start an insertion
        NEW_INSERT_SKIP.set(2);
    } else if INS_NEED_UNDO.get() && save_cursor_line().is_ok() {
        INS_NEED_UNDO.set(false);
    }

    // Always open a fold at the cursor line when inserting something.
    fold_open_cursor();

    if ARROW_USED.get() || INS_NEED_UNDO.get() {
        Err(Failed)
    } else {
        Ok(())
    }
}

/// Do the few things that stop an insert.
///
/// `end_insert_pos` is where the insert ended, and is `None` when another
/// window or buffer has already been jumped to -- in which case neither the
/// trimming nor the `'[`/`']` marks happen.  `esc` says the caller is
/// `ins_esc`, and `nomove` is `i_CTRL-\_CTRL-O`, which must not move the
/// cursor.
pub(crate) fn stop_insert(end_insert_pos: Option<InsertEnd>, esc: bool, nomove: bool) {
    stop_redo_ins();

    // Abandon the replace stack (this reinitialises it).
    replace_stack_clear();

    // Save the inserted text for a later `.`/CTRL-@/CTRL-A.  Not when
    // `RESTART_EDIT` was set and nothing was inserted, or `CTRL-O w`
    // followed by `<Left>` would clear `LAST_INSERT`.
    let inserted = get_inserted();
    let added = if inserted.data().is_null() {
        0
    } else {
        c_int::try_from(inserted.len()).expect("inserted text is never longer than an int")
            - NEW_INSERT_SKIP.get()
    };
    if DID_RESTART_EDIT.get().is_nul() || added > 0 {
        LAST_INSERT.set(inserted);
        LAST_INSERT_SKIP.set(if added < 0 { 0 } else { NEW_INSERT_SKIP.get() });
    }

    if !ARROW_USED.get()
        && let Some(end_insert_pos) = end_insert_pos
    {
        // Auto-format now.  It looks odd to do this when *stopping* an
        // insertion, but appending a line that ends in a space needs it.
        // Only when something was actually inserted, or undo breaks.
        let mut cc;
        if !INS_NEED_UNDO.get() && has_format_option(FoFlag::AUTO) {
            let tpos = Win::current().w_cursor;

            // At the end of a line after a space, formatting would move
            // the cursor to the following word; move it onto the space
            // first so it does not.
            cc = c_int::from(b'x');
            if Win::current().w_cursor.col > ColNr::ZERO && char_at_cursor() == NUL {
                dec_cursor();
                cc = char_at_cursor();
                if !ascii_iswhite(cc) {
                    Win::current().w_cursor = tpos;
                }
            }

            auto_format(true, false);

            if ascii_iswhite(cc) {
                if char_at_cursor() != NUL {
                    inc_cursor();
                }
                // Still on the same character: keep its `coladd` too.
                if char_at_cursor() == NUL
                    && Win::current().w_cursor.lnum == tpos.lnum
                    && Win::current().w_cursor.col == tpos.col
                {
                    Win::current().w_cursor.coladd = tpos.coladd;
                }
            }
        }

        // Remove a space that was inserted only for auto-formatting.
        check_auto_format(true);

        // After an auto-indent the user typed nothing on, take the white
        // space off the end of the line again and put the cursor back.
        // Only when ESC was used or the cursor moved to another line, and
        // only if the remembered position is still in the buffer -- the
        // text may have changed unexpectedly.
        if !nomove
            && DID_AI.get()
            && (esc
                || (!cpo_has(CpoFlag::INDENT)
                    && Win::current().w_cursor.lnum != end_insert_pos.pos().lnum))
            && end_insert_pos.pos().lnum <= Buf::current().b_ml.ml_line_count
        {
            let mut tpos = Win::current().w_cursor;
            let prev_col = end_insert_pos.pos().col;

            Win::current().w_cursor = end_insert_pos.pos();
            check_cursor_col(Win::current()); // make sure it is not past the line
            loop {
                if char_at_cursor() == NUL && Win::current().w_cursor.col > ColNr::ZERO {
                    Win::current().w_cursor.col -= 1;
                }
                cc = char_at_cursor();
                if !ascii_iswhite(cc) || del_char(true).is_err() {
                    break;
                }
            }

            if Win::current().w_cursor.lnum != tpos.lnum {
                Win::current().w_cursor = tpos;
            } else if Win::current().w_cursor.col < prev_col {
                // Reset `tpos`: the loop above may have invalidated it.
                tpos = Win::current().w_cursor;
                tpos.col += 1;
                if cc != NUL && gchar_pos(&tpos) == NUL {
                    Win::current().w_cursor.col += 1; // put the cursor back on the NUL
                }
            }

            // `<C-S-Right>` may have started Visual mode; adjust its
            // position for the characters just deleted.
            if visual_active() {
                check_visual_pos();
            }
        }
    }

    DID_AI.set(false);
    DID_SI.set(false);
    CAN_SI.set(false);
    CAN_SI_BACK.set(false);

    // Set `'[` and `']` to the inserted text.  A null `end_insert_pos`
    // means a different buffer is current now.
    if let Some(end_insert_pos) = end_insert_pos {
        Buf::current().b_op_start = INSSTART.get();
        Buf::current().b_op_start_orig = INSSTART_ORIG.get();
        Buf::current().b_op_end = end_insert_pos.pos();
    }
}

/// Trigger `event` and take care of fixing undo.
///
/// Any change an autocommand makes belongs to an undo block of its own --
/// except for `InsertLeave`, whose change is still part of the insert.
///
/// Answers whether any autocommand ran.
pub(crate) fn ins_apply_autocmds(event: AutoEvent) -> bool {
    let tick = buf_get_changedtick(Buf::current());
    let r = apply_autocmds(event, None, None, false, Buf::current_or_none());

    if event != AutoEvent::InsertLeave && tick != buf_get_changedtick(Buf::current()) {
        let _ = u_save(
            Win::current().w_cursor.lnum,
            Win::current().w_cursor.lnum + 1,
        );
    }
    r
}

/// The character under the cursor, `NUL` at the end of the line.
#[inline(always)]
fn char_at_cursor() -> c_int {
    gchar_cursor()
}

/// Save the cursor's line for undo, answering `Ok` when it was saved.
#[inline(always)]
fn save_cursor_line() -> Result<(), Failed> {
    u_save_cursor()
}
