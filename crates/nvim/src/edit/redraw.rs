//! Drawing while inserting: postponing it, and the two things drawn
//! directly.
//!
//! [`ins_redraw`] is the postponement -- Insert mode does not redraw after
//! each character but just before the next key is *waited for*, which is
//! what makes a long CTRL-R or a mapping fast and is also where the
//! `TextChangedI`/`CursorMovedI` autocommands and the completion popup's
//! update live.  Everything it does is conditional on `char_avail()` being
//! false: if the user has already typed ahead, none of it happens.
//!
//! [`edit_putchar`]/[`edit_unputchar`] bypass all of that: they write one
//! character straight onto the grid and remember what was under it, which is
//! how CTRL-V and CTRL-K show a `^` or a `?` at the cursor while they wait
//! for the rest of the sequence.  [`display_dollar`]/[`undisplay_dollar`] are
//! the same trick for the `$` that 'cpoptions' `$` puts at the end of a
//! changed region.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::*;
use crate::grid::default_grid_ref;
use crate::option::cpo_has;
use crate::types::{CpoFlag, MB_MAXCHAR, NUL};

/// Redraw for Insert mode.
///
/// Postponed until the next character is asked for, so that `$` in
/// 'cpoptions' works, and skipped entirely while characters are already
/// available -- which is what makes a long CTRL-R or a mapping fast.
///
/// `ready` means "not busy with something": with it false only the drawing
/// happens, and none of the autocommands.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
pub(crate) unsafe fn ins_redraw(ready: bool) {
    unsafe {
        if char_avail() {
            return;
        }

        // CursorMovedI, if the cursor moved.  Not while the popup menu is up:
        // the command might delete it.
        if ready
            && has_event(EVENT_CURSORMOVEDI)
            && (last_cursormoved_win.get() != curwin.get()
                || !equalpos(last_cursormoved.get(), (*curwin.get()).w_cursor))
            && !pum_visible()
        {
            // Update the screen first so syntax highlighting is right after a
            // change (inserting a `(`, say).  The autocommand may ask for
            // another redraw, which happens again below.
            if syntax_present(curwin.get()) && must_redraw.get() != 0 {
                update_screen();
            }
            // An autocommand may call getcurpos(), so curswant has to be
            // correct first.
            update_curswant();
            ins_apply_autocmds(EVENT_CURSORMOVEDI);
            last_cursormoved_win.set(curwin.get());
            last_cursormoved.set((*curwin.get()).w_cursor);
        }

        // TextChangedI when changedtick_i differs, and TextChangedP when
        // changedtick_pum does.  They keep separate ticks because closing the
        // popup menu still has to fire TextChangedI, for compatibility.
        //
        // The autocommand may change the buffer *and* the window, so `curbuf`
        // is saved around it; and if it changed the text, the insert's undo
        // block has to be closed the way `ins_apply_autocmds` does it.
        let fire_text_changed = |event: event_T, tick: *mut varnumber_T| {
            let mut aco = aco_save_T::default();
            let before = buf_get_changedtick(curbuf.get());

            // Save and restore curwin/curbuf, in case the autocommand changes
            // them.
            aucmd_prepbuf(&raw mut aco, curbuf.get());
            apply_autocmds(
                event,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                false,
                curbuf.get(),
            );
            aucmd_restbuf(&raw mut aco);

            *tick = buf_get_changedtick(curbuf.get());
            if before != *tick {
                // See `ins_apply_autocmds`: the autocommand's change belongs
                // to a block of its own.
                u_save(
                    (*curwin.get()).w_cursor.lnum,
                    (*curwin.get()).w_cursor.lnum + 1,
                );
            }
        };

        if ready && has_event(EVENT_TEXTCHANGEDI) && !pum_visible() {
            let tick = &raw mut (*curbuf.get()).b_last_changedtick_i;
            if *tick != buf_get_changedtick(curbuf.get()) {
                fire_text_changed(EVENT_TEXTCHANGEDI, tick);
            }
        }
        if ready && has_event(EVENT_TEXTCHANGEDP) && pum_visible() {
            let tick = &raw mut (*curbuf.get()).b_last_changedtick_pum;
            if *tick != buf_get_changedtick(curbuf.get()) {
                fire_text_changed(EVENT_TEXTCHANGEDP, tick);
            }
        }

        if ready {
            may_trigger_win_scrolled_resized();
        }

        // BufModified, if b_changed_invalid is set.
        if ready
            && has_event(EVENT_BUFMODIFIEDSET)
            && (*curbuf.get()).b_changed_invalid
            && !pum_visible()
        {
            apply_autocmds(
                EVENT_BUFMODIFIEDSET,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                false,
                curbuf.get(),
            );
            (*curbuf.get()).b_changed_invalid = false;
        }

        // SafeState, if nothing is pending.
        may_trigger_safestate(ready && !ins_compl_active() && !pum_visible());

        pum_check_clear();
        show_cursor_info_later(false);
        if must_redraw.get() != 0 {
            update_screen();
        } else {
            redraw_statuslines();
            if clear_cmdline.get() || redraw_cmdline.get() || redraw_mode.get() {
                showmode(); // clear cmdline and show mode
            }
        }
        setcursor();
        emsg_on_display.set(false); // may remove error message now
    }
}

/// Put character `c` directly onto the screen at the cursor, remembering what
/// was there so [`edit_unputchar`] can put it back.
///
/// Used while handling CTRL-V, CTRL-K and friends, which have to show
/// something at the cursor while they wait for the rest of the sequence.
/// Nothing is stored in a buffer, so the next real redraw removes it.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn edit_putchar(c: c_int, highlight: bool) {
    unsafe {
        let win = curwin.get();
        if !(*win).w_grid_alloc.is_allocated() && !default_grid_ref().is_allocated() {
            return;
        }

        update_topline(win); // just in case w_topline isn't valid
        validate_cursor(win);
        let attr = if highlight {
            *(*hl_attr_active.ptr()).offset(HLF_8 as isize)
        } else {
            0
        };

        pc_row.set((*win).w_wrow);
        pc_status.set(PutChar::Unset);
        grid_line_start((*win).w_grid, pc_row.get());
        if (*win).w_onebuf_opt.wo_rl != 0 {
            pc_col.set((*win).w_view_width - 1 - (*win).w_wcol);
            if grid_line_getchar(pc_col.get(), ::core::ptr::null_mut()) == NUL as schar_T {
                grid_line_put_schar(pc_col.get() - 1, ' ' as schar_T, attr);
                (*win).w_wcol -= 1;
                pc_status.set(PutChar::Right);
            }
        } else {
            pc_col.set((*win).w_wcol);
            if grid_line_getchar(pc_col.get() + 1, ::core::ptr::null_mut()) == NUL as schar_T {
                // pc_col is the left half of a double-width character.
                pc_status.set(PutChar::Left);
            }
        }

        // Save the character, so it can be put back.
        if pc_status.get() == PutChar::Unset {
            pc_schar.set(grid_line_getchar(pc_col.get(), pc_attr.ptr()));
            pc_status.set(PutChar::Set);
        }

        let mut buf: [c_char; MB_MAXCHAR + 1] = [0; MB_MAXCHAR + 1];
        let p = buf.as_mut_ptr();
        grid_line_puts(pc_col.get(), p, utf_char2bytes(c, p), attr);
        grid_line_flush();
    }
}

/// Undo the previous [`edit_putchar`].
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn edit_unputchar() {
    unsafe {
        let win = curwin.get();
        match pc_status.get() {
            PutChar::Unset => {}
            // Half of a double-width character was overwritten and cannot be
            // restored a cell at a time; redraw the whole line instead.
            PutChar::Right => {
                (*win).w_wcol += 1;
                redraw_win_line(win, (*win).w_cursor.lnum);
            }
            PutChar::Left => redraw_win_line(win, (*win).w_cursor.lnum),
            PutChar::Set => {
                grid_line_start((*win).w_grid, pc_row.get());
                grid_line_put_schar(pc_col.get(), pc_schar.get(), pc_attr.get());
                grid_line_flush();
            }
        }
    }
}

/// Called when `$` is in 'cpoptions': show a `$` at the end of the changed
/// text.  Only works while the cursor is in the line that changes.
///
/// # Safety
/// Must run with a live `curwin` whose cursor line is at least `col_arg`
/// bytes long.
pub(crate) unsafe fn display_dollar(col_arg: colnr_T) {
    unsafe {
        let col = col_arg.max(0);

        if !redrawing() {
            return;
        }

        let win = curwin.get();
        let save_col = (*win).w_cursor.col;
        (*win).w_cursor.col = col;

        // On the last byte of a multi-byte character, move to the first byte.
        let p = get_cursor_line_ptr();
        (*win).w_cursor.col -= utf_head_off(p, p.offset(col as isize));
        curs_columns(win, 0); // recompute w_wrow and w_wcol
        if (*win).w_wcol < (*win).w_view_width {
            edit_putchar('$' as c_int, false);
            dollar_vcol.set((*win).w_virtcol);
        }
        (*win).w_cursor.col = save_col;
    }
}

/// Take the `$` away again.  Call before moving the cursor off the normal
/// insert position.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn undisplay_dollar() {
    unsafe {
        if dollar_vcol.get() < 0 {
            return;
        }
        dollar_vcol.set(-1);
        redraw_win_line(curwin.get(), (*curwin.get()).w_cursor.lnum);
    }
}

/// The value `w_virtcol` would have with 'list' off -- unless 'cpoptions'
/// contains `L`, which says the option should be honoured after all.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn get_nolist_virtcol() -> colnr_T {
    unsafe {
        let win = curwin.get();
        if (*win).w_buffer.is_null()
            || (*(*win).w_buffer).b_ml.ml_mfp.is_null()
            || (*win).w_cursor.lnum > (*(*win).w_buffer).b_ml.ml_line_count
        {
            return 0;
        }
        if (*win).w_onebuf_opt.wo_list != 0 && !cpo_has(CpoFlag::LISTWM) {
            return getvcol_nolist(&raw mut (*win).w_cursor);
        }
        validate_virtcol(win);
        (*win).w_virtcol
    }
}
