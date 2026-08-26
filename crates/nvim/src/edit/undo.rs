//! Where one undoable insert ends and the next begins.
//!
//! Insert mode is one undo block from the first character typed to `<Esc>`
//! -- unless the cursor is *moved* in between, which starts a new one.
//! [`start_arrow`] is that: called by every motion key, it closes the current
//! block and remembers where the insert ended.  [`stop_arrow`] is the mirror,
//! called before any change, and is where the `u_save` for the block actually
//! happens (`ins_need_undo`).
//!
//! The pair is spelled with two flags rather than one state: `arrow_used`
//! means "the block is closed", `ins_need_undo` means "the next change still
//! owes a `u_save`", and `stop_arrow` answers `FAIL` when either is still
//! true afterwards, which is the caller's signal not to change the buffer at
//! all.
//!
//! [`stop_insert`] is the whole of leaving the mode: save the text for `.`,
//! auto-format if 'formatoptions' `a` asked for it, trim the auto-indent the
//! user never typed on, and set `'[`/`']`.  [`ins_apply_autocmds`] is here
//! because an autocommand may change the buffer, and that change needs an
//! undo block of its own.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::normal::visual_active;
use crate::option::cpo_has;
use crate::types::{CpoFlag, FAIL, FoFlag, NUL, OK};

/// Called when an arrow key is used in Insert mode: for undo and redo it
/// resembles hitting `<Esc>`.
///
/// `end_insert_pos` may be null, meaning the insert's end is not known
/// (another buffer is current now).
///
/// # Safety
/// `end_insert_pos`, if not null, must point to a valid position.
pub(crate) unsafe fn start_arrow(end_insert_pos: *mut pos_T) {
    // SAFETY: the caller's promise about `end_insert_pos` is exactly what
    // `start_arrow_common` passes on to `stop_insert`.
    unsafe { start_arrow_common(end_insert_pos, true) }
}

/// [`start_arrow`], but able to keep the undoable change open -- which is
/// `i_CTRL-G_U`, and is recorded so redo does the same.
///
/// # Safety
/// `end_insert_pos`, if not null, must point to a valid position.
pub(crate) unsafe fn start_arrow_with_change(end_insert_pos: *mut pos_T, end_change: bool) {
    // SAFETY: the caller's promise about `end_insert_pos` is exactly what
    // `start_arrow_common` passes on to `stop_insert`.
    unsafe { start_arrow_common(end_insert_pos, end_change) };
    if !end_change {
        append_to_redobuff_char(Ctrl_G);
        append_to_redobuff_char('U' as c_int);
    }
}

/// # Safety
/// `end_insert_pos`, if not null, must point to a valid position.
unsafe fn start_arrow_common(end_insert_pos: *mut pos_T, end_change: bool) {
    // SAFETY: the caller's promise about `end_insert_pos` is exactly what
    // `stop_insert` asks of it; `ESC_STR` is a static string.
    if !arrow_used.get() && end_change {
        // Something has been inserted: close the block.
        unsafe { append_to_redobuff(ESC_STR.as_ptr()) };
        unsafe { stop_insert(end_insert_pos, 0, 0) };
        arrow_used.set(true);
    }
    check_spell_redraw();
}

/// Highlight the word at the cursor if that was skipped while typing.
///
/// It may be skipped again, so the line number is cleared first.
pub(crate) fn check_spell_redraw() {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    if spell_redraw_lnum.get() != 0 {
        let lnum = spell_redraw_lnum.get();
        spell_redraw_lnum.set(0);
        unsafe { redraw_win_line(curwin.get(), lnum) };
    }
}

/// Called before any change in Insert mode: if an arrow key was used, start a
/// new insertion here.
///
/// `FAIL` when undo is impossible, in which case the caller must not insert.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
pub(crate) unsafe fn stop_arrow() -> c_int {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    if arrow_used.get() {
        Insstart.set(cur_win().w_cursor); // new insertion starts here
        if Insstart.get().col > Insstart_orig.get().col && !ins_need_undo.get() {
            // Don't update the original insert position when moved to the
            // right, except when nothing was inserted yet.
            update_Insstart_orig.set(false);
        }
        Insstart_textlen.set(unsafe { linetabsize_str(get_cursor_line_ptr()) } as colnr_T);

        if save_cursor_line() == OK {
            arrow_used.set(false);
            ins_need_undo.set(false);
        }
        ai_col.set(0);
        if State.get() & VREPLACE_FLAG != 0 {
            orig_line_count.set(cur_buf().b_ml.ml_line_count);
            vr_lines_changed.set(1);
        }
        unsafe { reset_redobuff() };
        unsafe { append_to_redobuff(c"1i".as_ptr()) }; // pretend we start an insertion
        new_insert_skip.set(2);
    } else if ins_need_undo.get() && save_cursor_line() == OK {
        ins_need_undo.set(false);
    }

    // Always open a fold at the cursor line when inserting something.
    unsafe { fold_open_cursor() };

    if arrow_used.get() || ins_need_undo.get() {
        FAIL
    } else {
        OK
    }
}

/// Do the few things that stop an insert.
///
/// `end_insert_pos` is where the insert ended, and is null when another
/// window or buffer has already been jumped to -- in which case neither the
/// trimming nor the `'[`/`']` marks happen.  `esc` says the caller is
/// `ins_esc`, and `nomove` is `i_CTRL-\_CTRL-O`, which must not move the
/// cursor.
///
/// # Safety
/// `end_insert_pos`, if not null, must point to a valid position.
pub(crate) unsafe fn stop_insert(end_insert_pos: *mut pos_T, esc: c_int, nomove: c_int) {
    stop_redo_ins();

    // Abandon the replace stack (this reinitialises it).
    // SAFETY: the replace stack is a live global that owns its `items`.
    unsafe { xfree((*replace_stack_ref()).items.cast()) };
    unsafe { *replace_stack_ref() = REPLACE_STACK_EMPTY };

    // Save the inserted text for a later `.`/CTRL-@/CTRL-A.  Not when
    // `restart_edit` was set and nothing was inserted, or `CTRL-O w`
    // followed by `<Left>` would clear `last_insert`.
    let inserted = unsafe { get_inserted() };
    let added = if inserted.data().is_null() {
        0
    } else {
        inserted.len() as c_int - new_insert_skip.get()
    };
    if did_restart_edit.get() == 0 || added > 0 {
        unsafe { last_insert_slot().replace(inserted) };
        last_insert_skip.set(if added < 0 { 0 } else { new_insert_skip.get() });
    } else {
        unsafe { xfree(inserted.data() as *mut ::core::ffi::c_void) };
    }

    if !arrow_used.get() && !end_insert_pos.is_null() {
        // Auto-format now.  It looks odd to do this when *stopping* an
        // insertion, but appending a line that ends in a space needs it.
        // Only when something was actually inserted, or undo breaks.
        let mut cc = 0;
        if !ins_need_undo.get() && unsafe { has_format_option(FoFlag::AUTO) } {
            let tpos = cur_win().w_cursor;

            // At the end of a line after a space, formatting would move
            // the cursor to the following word; move it onto the space
            // first so it does not.
            cc = 'x' as c_int;
            if cur_win().w_cursor.col > 0 && char_at_cursor() == NUL {
                unsafe { dec_cursor() };
                cc = char_at_cursor();
                if !ascii_iswhite(cc) {
                    cur_win().w_cursor = tpos;
                }
            }

            unsafe { auto_format(true, false) };

            if ascii_iswhite(cc) {
                if char_at_cursor() != NUL {
                    unsafe { inc_cursor() };
                }
                // Still on the same character: keep its `coladd` too.
                if char_at_cursor() == NUL
                    && cur_win().w_cursor.lnum == tpos.lnum
                    && cur_win().w_cursor.col == tpos.col
                {
                    cur_win().w_cursor.coladd = tpos.coladd;
                }
            }
        }

        // Remove a space that was inserted only for auto-formatting.
        unsafe { check_auto_format(true) };

        // After an auto-indent the user typed nothing on, take the white
        // space off the end of the line again and put the cursor back.
        // Only when ESC was used or the cursor moved to another line, and
        // only if the remembered position is still in the buffer -- the
        // text may have changed unexpectedly.
        if nomove == 0
            && did_ai.get()
            && (esc != 0
                || (!cpo_has(CpoFlag::INDENT)
                    && cur_win().w_cursor.lnum != unsafe { *end_insert_pos }.lnum))
            && unsafe { *end_insert_pos }.lnum <= cur_buf().b_ml.ml_line_count
        {
            let mut tpos = cur_win().w_cursor;
            let prev_col = unsafe { *end_insert_pos }.col;

            cur_win().w_cursor = unsafe { *end_insert_pos };
            unsafe { check_cursor_col(curwin.get()) }; // make sure it is not past the line
            loop {
                if char_at_cursor() == NUL && cur_win().w_cursor.col > 0 {
                    cur_win().w_cursor.col -= 1;
                }
                cc = char_at_cursor();
                if !ascii_iswhite(cc) || unsafe { del_char(true) } == FAIL {
                    break;
                }
            }

            if cur_win().w_cursor.lnum != tpos.lnum {
                cur_win().w_cursor = tpos;
            } else if cur_win().w_cursor.col < prev_col {
                // Reset `tpos`: the loop above may have invalidated it.
                tpos = cur_win().w_cursor;
                tpos.col += 1;
                if cc != NUL && unsafe { gchar_pos(&raw mut tpos) } == NUL {
                    cur_win().w_cursor.col += 1; // put the cursor back on the NUL
                }
            }

            // `<C-S-Right>` may have started Visual mode; adjust its
            // position for the characters just deleted.
            if visual_active() {
                unsafe { check_visual_pos() };
            }
        }
    }

    did_ai.set(false);
    did_si.set(false);
    can_si.set(false);
    can_si_back.set(false);

    // Set `'[` and `']` to the inserted text.  A null `end_insert_pos`
    // means a different buffer is current now.
    if !end_insert_pos.is_null() {
        cur_buf().b_op_start = Insstart.get();
        cur_buf().b_op_start_orig = Insstart_orig.get();
        cur_buf().b_op_end = unsafe { *end_insert_pos };
    }
}

/// Trigger `event` and take care of fixing undo.
///
/// Any change an autocommand makes belongs to an undo block of its own --
/// except for `InsertLeave`, whose change is still part of the insert.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
pub(crate) unsafe fn ins_apply_autocmds(event: event_T) -> c_int {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    let tick = unsafe { buf_get_changedtick(curbuf.get()) };
    let none = ::core::ptr::null_mut();
    let r = unsafe { apply_autocmds(event, none, none, false, curbuf.get()) } as c_int;

    if event != EVENT_INSERTLEAVE && tick != unsafe { buf_get_changedtick(curbuf.get()) } {
        unsafe { u_save(cur_win().w_cursor.lnum, cur_win().w_cursor.lnum + 1) };
    }
    r
}

/// The character under the cursor, `NUL` at the end of the line.
#[inline(always)]
fn char_at_cursor() -> c_int {
    // SAFETY: `curwin`/`curbuf` are live for the whole session.
    unsafe { gchar_cursor() }
}

/// Save the cursor's line for undo, answering `OK` when it was saved.
#[inline(always)]
fn save_cursor_line() -> c_int {
    // SAFETY: `curwin`/`curbuf` are live for the whole session.
    unsafe { u_save_cursor() }
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
