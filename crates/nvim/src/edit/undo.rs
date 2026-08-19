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

use core::ffi::c_int;

use super::*;
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
    unsafe { start_arrow_common(end_insert_pos, true) }
}

/// [`start_arrow`], but able to keep the undoable change open -- which is
/// `i_CTRL-G_U`, and is recorded so redo does the same.
///
/// # Safety
/// `end_insert_pos`, if not null, must point to a valid position.
pub(crate) unsafe fn start_arrow_with_change(end_insert_pos: *mut pos_T, end_change: bool) {
    unsafe {
        start_arrow_common(end_insert_pos, end_change);
        if !end_change {
            AppendCharToRedobuff(Ctrl_G);
            AppendCharToRedobuff('U' as c_int);
        }
    }
}

/// # Safety
/// `end_insert_pos`, if not null, must point to a valid position.
unsafe fn start_arrow_common(end_insert_pos: *mut pos_T, end_change: bool) {
    unsafe {
        if !arrow_used.get() && end_change {
            // Something has been inserted: close the block.
            AppendToRedobuff(ESC_STR.as_ptr());
            stop_insert(end_insert_pos, 0, 0);
            arrow_used.set(true);
        }
        check_spell_redraw();
    }
}

/// Highlight the word at the cursor if that was skipped while typing.
///
/// It may be skipped again, so the line number is cleared first.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn check_spell_redraw() {
    unsafe {
        if spell_redraw_lnum.get() != 0 {
            let lnum = spell_redraw_lnum.get();
            spell_redraw_lnum.set(0);
            redrawWinline(curwin.get(), lnum);
        }
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
    unsafe {
        if arrow_used.get() {
            Insstart.set((*curwin.get()).w_cursor); // new insertion starts here
            if (*Insstart.ptr()).col > (*Insstart_orig.ptr()).col && !ins_need_undo.get() {
                // Don't update the original insert position when moved to the
                // right, except when nothing was inserted yet.
                update_Insstart_orig.set(false);
            }
            Insstart_textlen.set(linetabsize_str(get_cursor_line_ptr()) as colnr_T);

            if u_save_cursor() == OK {
                arrow_used.set(false);
                ins_need_undo.set(false);
            }
            ai_col.set(0);
            if State.get() & VREPLACE_FLAG != 0 {
                orig_line_count.set((*curbuf.get()).b_ml.ml_line_count);
                vr_lines_changed.set(1);
            }
            ResetRedobuff();
            AppendToRedobuff(c"1i".as_ptr()); // pretend we start an insertion
            new_insert_skip.set(2);
        } else if ins_need_undo.get() && u_save_cursor() == OK {
            ins_need_undo.set(false);
        }

        // Always open a fold at the cursor line when inserting something.
        foldOpenCursor();

        if arrow_used.get() || ins_need_undo.get() {
            FAIL
        } else {
            OK
        }
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
    unsafe {
        stop_redo_ins();

        // Abandon the replace stack (this reinitialises it).
        xfree((*replace_stack.ptr()).items as *mut ::core::ffi::c_void);
        *replace_stack.ptr() = REPLACE_STACK_EMPTY;

        // Save the inserted text for a later `.`/CTRL-@/CTRL-A.  Not when
        // `restart_edit` was set and nothing was inserted, or `CTRL-O w`
        // followed by `<Left>` would clear `last_insert`.
        let inserted = get_inserted();
        let added = if inserted.data.is_null() {
            0
        } else {
            inserted.size as c_int - new_insert_skip.get()
        };
        if did_restart_edit.get() == 0 || added > 0 {
            xfree((*last_insert.ptr()).data as *mut ::core::ffi::c_void);
            last_insert.set(inserted); // structure copy
            last_insert_skip.set(if added < 0 { 0 } else { new_insert_skip.get() });
        } else {
            xfree(inserted.data as *mut ::core::ffi::c_void);
        }

        if !arrow_used.get() && !end_insert_pos.is_null() {
            // Auto-format now.  It looks odd to do this when *stopping* an
            // insertion, but appending a line that ends in a space needs it.
            // Only when something was actually inserted, or undo breaks.
            let mut cc = 0;
            if !ins_need_undo.get() && has_format_option(FoFlag::AUTO) {
                let tpos = (*curwin.get()).w_cursor;

                // At the end of a line after a space, formatting would move
                // the cursor to the following word; move it onto the space
                // first so it does not.
                cc = 'x' as c_int;
                if (*curwin.get()).w_cursor.col > 0 && gchar_cursor() == NUL {
                    dec_cursor();
                    cc = gchar_cursor();
                    if !ascii_iswhite(cc) {
                        (*curwin.get()).w_cursor = tpos;
                    }
                }

                auto_format(true, false);

                if ascii_iswhite(cc) {
                    if gchar_cursor() != NUL {
                        inc_cursor();
                    }
                    // Still on the same character: keep its `coladd` too.
                    if gchar_cursor() == NUL
                        && (*curwin.get()).w_cursor.lnum == tpos.lnum
                        && (*curwin.get()).w_cursor.col == tpos.col
                    {
                        (*curwin.get()).w_cursor.coladd = tpos.coladd;
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
            if nomove == 0
                && did_ai.get()
                && (esc != 0
                    || (!cpo_has(CpoFlag::INDENT)
                        && (*curwin.get()).w_cursor.lnum != (*end_insert_pos).lnum))
                && (*end_insert_pos).lnum <= (*curbuf.get()).b_ml.ml_line_count
            {
                let mut tpos = (*curwin.get()).w_cursor;
                let prev_col = (*end_insert_pos).col;

                (*curwin.get()).w_cursor = *end_insert_pos;
                check_cursor_col(curwin.get()); // make sure it is not past the line
                loop {
                    if gchar_cursor() == NUL && (*curwin.get()).w_cursor.col > 0 {
                        (*curwin.get()).w_cursor.col -= 1;
                    }
                    cc = gchar_cursor();
                    if !ascii_iswhite(cc) || del_char(true) == FAIL {
                        break;
                    }
                }

                if (*curwin.get()).w_cursor.lnum != tpos.lnum {
                    (*curwin.get()).w_cursor = tpos;
                } else if (*curwin.get()).w_cursor.col < prev_col {
                    // Reset `tpos`: the loop above may have invalidated it.
                    tpos = (*curwin.get()).w_cursor;
                    tpos.col += 1;
                    if cc != NUL && gchar_pos(&raw mut tpos) == NUL {
                        (*curwin.get()).w_cursor.col += 1; // put the cursor back on the NUL
                    }
                }

                // `<C-S-Right>` may have started Visual mode; adjust its
                // position for the characters just deleted.
                if VIsual_active.get() {
                    check_visual_pos();
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
            (*curbuf.get()).b_op_start = Insstart.get();
            (*curbuf.get()).b_op_start_orig = Insstart_orig.get();
            (*curbuf.get()).b_op_end = *end_insert_pos;
        }
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
    unsafe {
        let tick = buf_get_changedtick(curbuf.get());
        let r = apply_autocmds(
            event,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            false,
            curbuf.get(),
        ) as c_int;

        if event != EVENT_INSERTLEAVE && tick != buf_get_changedtick(curbuf.get()) {
            u_save(
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.lnum + 1,
            );
        }
        r
    }
}
