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

use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

use super::*;
use crate::grid::default_grid_ref;
use crate::option::cpo_has;
use crate::statusline::hl_attr;
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
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    // The strings walked below are NUL-terminated lines of that buffer, and
    // every step stops at the NUL.
    if char_avail() {
        return;
    }

    // CursorMovedI, if the cursor moved.  Not while the popup menu is up:
    // the command might delete it.
    if ready
        && has_event(AutoEvent::CursorMovedI)
        && (last_cursormoved_win.get() != curwin.get()
            || !equalpos(last_cursormoved.get(), cur_win().w_cursor))
        && !pum_visible()
    {
        // Update the screen first so syntax highlighting is right after a
        // change (inserting a `(`, say).  The autocommand may ask for
        // another redraw, which happens again below.
        if unsafe { syntax_present(curwin.get()) } && must_redraw.get() != 0 {
            let _ = unsafe { update_screen() };
        }
        // An autocommand may call getcurpos(), so curswant has to be
        // correct first.
        unsafe { update_curswant() };
        unsafe { ins_apply_autocmds(AutoEvent::CursorMovedI) };
        last_cursormoved_win.set(curwin.get());
        last_cursormoved.set(cur_win().w_cursor);
    }

    // TextChangedI when changedtick_i differs, and TextChangedP when
    // changedtick_pum does.  They keep separate ticks because closing the
    // popup menu still has to fire TextChangedI, for compatibility.
    //
    // The autocommand may change the buffer *and* the window, so `curbuf`
    // is saved around it; and if it changed the text, the insert's undo
    // block has to be closed the way `ins_apply_autocmds` does it.
    let fire_text_changed = |event: AutoEvent, tick: *mut varnumber_T| {
        let mut aco = aco_save_T::default();
        let before = unsafe { buf_get_changedtick(Buf::new(curbuf.get())) };

        // Save and restore curwin/curbuf, in case the autocommand changes
        // them.
        unsafe { aucmd_prepbuf(&raw mut aco, curbuf.get()) };
        let none = ::core::ptr::null_mut();
        unsafe { apply_autocmds(event, none, none, false, curbuf.get()) };
        unsafe { aucmd_restbuf(&raw mut aco) };

        unsafe { *tick = buf_get_changedtick(Buf::new(curbuf.get())) };
        if before != unsafe { *tick } {
            // See `ins_apply_autocmds`: the autocommand's change belongs
            // to a block of its own.
            let _ = u_save(cur_win().w_cursor.lnum, cur_win().w_cursor.lnum + 1);
        }
    };

    let mut buf = cur_buf();
    if ready && has_event(AutoEvent::TextChangedI) && !pum_visible() {
        let tick = &mut buf.b_last_changedtick_i;
        if *tick != unsafe { buf_get_changedtick(Buf::new(curbuf.get())) } {
            fire_text_changed(AutoEvent::TextChangedI, tick);
        }
    }
    if ready && has_event(AutoEvent::TextChangedP) && pum_visible() {
        let tick = &mut buf.b_last_changedtick_pum;
        if *tick != unsafe { buf_get_changedtick(Buf::new(curbuf.get())) } {
            fire_text_changed(AutoEvent::TextChangedP, tick);
        }
    }

    if ready {
        unsafe { may_trigger_win_scrolled_resized() };
    }

    // BufModified, if b_changed_invalid is set.
    if ready
        && has_event(AutoEvent::BufModifiedSet)
        && cur_buf().b_changed_invalid
        && !pum_visible()
    {
        let none = ::core::ptr::null_mut();
        unsafe { apply_autocmds(AutoEvent::BufModifiedSet, none, none, false, curbuf.get()) };
        cur_buf().b_changed_invalid = false;
    }

    // SafeState, if nothing is pending.
    unsafe { may_trigger_safestate(ready && !ins_compl_active() && !pum_visible()) };

    unsafe { pum_check_clear() };
    unsafe { show_cursor_info_later(false) };
    if must_redraw.get() != 0 {
        let _ = unsafe { update_screen() };
    } else {
        unsafe { redraw_statuslines() };
        if clear_cmdline.get() || redraw_cmdline.get() || redraw_mode.get() {
            unsafe { showmode() }; // clear cmdline and show mode
        }
    }
    unsafe { setcursor() };
    emsg_on_display.set(false); // may remove error message now
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
    let mut win = cur_win();
    if !win.w_grid_alloc.is_allocated() && !default_grid_ref().is_allocated() {
        return;
    }

    // SAFETY: `curwin` is live, which is all these editor-wide routines ask
    // for, and the grid line is opened and flushed around every write.
    update_topline(win); // just in case w_topline isn't valid
    validate_cursor(win);
    let attr = if highlight { hl_attr(HLF_8) } else { 0 };

    pc_row.set(win.w_wrow);
    pc_status.set(PutChar::Unset);
    unsafe { grid_line_start(win.w_grid, pc_row.get()) };
    if win.w_onebuf_opt.wo_rl != 0 {
        pc_col.set(win.w_view_width - 1 - win.w_wcol);
        if unsafe { grid_line_getchar(pc_col.get(), ::core::ptr::null_mut()) } == NUL as schar_T {
            grid_line_put_schar(pc_col.get() - 1, ' ' as schar_T, attr);
            win.w_wcol -= 1;
            pc_status.set(PutChar::Right);
        }
    } else {
        pc_col.set(win.w_wcol);
        if unsafe { grid_line_getchar(pc_col.get() + 1, ::core::ptr::null_mut()) } == NUL as schar_T
        {
            // pc_col is the left half of a double-width character.
            pc_status.set(PutChar::Left);
        }
    }

    // Save the character, so it can be put back.
    if pc_status.get() == PutChar::Unset {
        let mut attr = pc_attr.get();
        pc_schar.set(unsafe { grid_line_getchar(pc_col.get(), &raw mut attr) });
        pc_attr.set(attr);
        pc_status.set(PutChar::Set);
    }

    let mut buf: [c_char; MB_MAXCHAR + 1] = [0; MB_MAXCHAR + 1];
    let p = buf.as_mut_ptr();
    unsafe { grid_line_puts(pc_col.get(), p, utf_char2bytes(c, p), attr) };
    unsafe { grid_line_flush() };
}

/// Undo the previous [`edit_putchar`].
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn edit_unputchar() {
    let mut win = cur_win();
    // SAFETY: `curwin` is live, and the line it is on is a line of its own
    // buffer.
    match pc_status.get() {
        PutChar::Unset => {}
        // Half of a double-width character was overwritten and cannot be
        // restored a cell at a time; redraw the whole line instead.
        PutChar::Right => {
            win.w_wcol += 1;
            unsafe { redraw_win_line(win.raw(), win.w_cursor.lnum) };
        }
        PutChar::Left => unsafe { redraw_win_line(win.raw(), win.w_cursor.lnum) },
        PutChar::Set => {
            unsafe { grid_line_start(win.w_grid, pc_row.get()) };
            grid_line_put_schar(pc_col.get(), pc_schar.get(), pc_attr.get());
            unsafe { grid_line_flush() };
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
    let col = col_arg.max(0);

    if !unsafe { redrawing() } {
        return;
    }

    let mut win = cur_win();
    let save_col = win.w_cursor.col;
    win.w_cursor.col = col;

    // On the last byte of a multi-byte character, move to the first byte.
    // SAFETY: the caller promises the cursor line holds at least `col` bytes,
    // so `p + col` is a byte of that line.
    let p = get_cursor_line_ptr();
    win.w_cursor.col -= unsafe { utf_head_off(p, p.offset(col as isize)) };
    curs_columns(win, 0); // recompute w_wrow and w_wcol
    if win.w_wcol < win.w_view_width {
        unsafe { edit_putchar('$' as c_int, false) };
        dollar_vcol.set(win.w_virtcol);
    }
    win.w_cursor.col = save_col;
}

/// Take the `$` away again.  Call before moving the cursor off the normal
/// insert position.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn undisplay_dollar() {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    if dollar_vcol.get() < 0 {
        return;
    }
    dollar_vcol.set(-1);
    unsafe { redraw_win_line(curwin.get(), cur_win().w_cursor.lnum) };
}

/// The value `w_virtcol` would have with 'list' off -- unless 'cpoptions'
/// contains `L`, which says the option should be honoured after all.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn get_nolist_virtcol() -> colnr_T {
    let mut win = cur_win();
    if win.w_buffer.is_null()
        || win.buffer().b_ml.ml_mfp.is_null()
        || win.w_cursor.lnum > win.buffer().b_ml.ml_line_count
    {
        return 0;
    }
    if win.w_onebuf_opt.wo_list != 0 && !cpo_has(CpoFlag::LISTWM) {
        // SAFETY: a live window's own cursor, in a buffer it has lines for.
        return unsafe { getvcol_nolist(&mut win.w_cursor) };
    }
    // SAFETY: `curwin` is live for the whole session.
    validate_virtcol(win);
    win.w_virtcol
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
