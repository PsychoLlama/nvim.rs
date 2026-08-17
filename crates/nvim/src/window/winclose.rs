//! `win_close()` -- closing one window.
//!
//! The re-entrant half of closing: free the window's buffer if nothing else
//! shows it, fire `WinClosed`, remove the frame and give its room to a
//! neighbour, pick the window to enter next, and cope with the fact that any
//! of those autocommands may have closed further windows or freed the buffer
//! in hand.  [`close_othertab`] is the same for a window that is not on the
//! current tab page, and cannot simply enter it to do the work.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

#[allow(unused_imports)]
use super::*;
use crate::autocmd::{
    EVENT_BUFENTER, EVENT_BUFLEAVE, EVENT_TABCLOSED, EVENT_TABCLOSEDPRE, EVENT_TABLEAVE,
    EVENT_WINCLOSED, EVENT_WINLEAVE, EVENT_WINNEWPRE, has_event,
};
use crate::buffer::{BufRef, bt_help, close_buffer};
use crate::diff::diffopt_closeoff;
use crate::drawscreen::UPD_NOT_VALID;
use crate::ex_eval::aborting;
use crate::global_cell::GlobalCell;
use crate::main::{
    curbuf, curtab, curwin, e_autocmd_close, e_floatonly, first_tabpage, firstwin, getout, lastwin,
    p_ea, p_ead, p_ru, redraw_cmdline, redraw_tabline,
};
use crate::message::internal_error;
use crate::normal::reset_VIsual_and_resel;
use crate::strings::vim_snprintf;
use crate::types::ui::kUIMultigrid;
use crate::types::{CMD_SIZE, CMD_close, Integer, frame_T, size_t};
use crate::ui::{ui_call_win_close, ui_has};
use crate::winfloat::win_float_find_altwin;
use crate::winlayer::tabs;

pub unsafe extern "C" fn win_close(win: *mut win_T, free_buf: bool, force: bool) -> c_int {
    // SAFETY: the caller's promise -- a live window.
    close(unsafe { Win::new(win) }, free_buf, force)
}

/// Close window `win`, which must be on the current tab page, unloading its
/// buffer with `free_buf`. `FAIL` when the window was not closed.
///
/// Called by `:quit`, `:close`, `:xit`, `:wq` and `findtag()`.
pub(crate) fn close(win: Win, free_buf: bool, force: bool) -> c_int {
    let mut win = win;
    let prev_curtab = curtab.get();
    let win_frame = if win.w_floating {
        ptr::null_mut::<frame_T>()
    } else {
        win.frame().fr_parent
    };
    let had_diffmode = win.w_onebuf_opt.wo_diff != 0;

    if is_last_window(win) {
        err(e_cannot_close_last_window.as_ptr());
        return FAIL;
    }
    if !win.w_floating && layout_locked(CMD_close) {
        return FAIL;
    }
    if win.w_locked || win.buffer_or_none().is_some_and(|buf| buf.b_locked > 0) {
        return FAIL; // window is already being closed
    }
    if is_autocmd_window(Some(win)) {
        err(&raw const e_autocmd_close as *const c_char);
        return FAIL;
    }
    if last_win().w_floating && only_window(win, None) {
        if let Some(rc) = close_the_floats(win, force) {
            return rc;
        }
    }

    // When closing the last window in a tab page first go to another tab page
    // and then close the window and the tab page, so that `curwin` and `curtab`
    // are never invalid while memory is freed.
    if close_last_tabpage_window(win, free_buf, prev_curtab) {
        return FAIL;
    }

    // When closing the help window, try restoring a snapshot afterwards.
    // Otherwise clear the snapshot, which is now invalid.
    let help_window = is_help(win.buffer_or_none());
    if !help_window {
        drop_snapshot(cur_tab(), SNAP_HELP_IDX);
    }
    let quickfix_window = is_quickfix(win.buffer_or_none());
    if !quickfix_window {
        drop_snapshot(cur_tab(), SNAP_QUICKFIX_IDX);
    }

    let mut other_buffer = false;
    if win.is_current() {
        match leave_closing_window(win) {
            Leave::Failed => return FAIL,
            Leave::Ok { other } => other_buffer = other,
        }
    }

    // Fire WinClosed just before starting to free window-related resources.
    fire_winclosed(win);
    // The autocommand may have freed the window already.
    if !valid_win_any_tab(win.raw()) {
        return OK;
    }

    let bufref = BufRef::of_raw(win.w_buffer);
    let action = if free_buf { DOBUF_UNLOAD as c_int } else { 0 };
    let did_decrement = close_win_buffer(win, action, true);

    if valid_win(win.raw()).is_some()
        && win.buffer_or_none().is_none()
        && !win.w_floating
        && is_last_window(win)
    {
        // Autocommands have closed all windows, quit now. Restore
        // `curwin->w_buffer`, or writing the ShaDa file may fail.
        if cur_win().buffer_or_none().is_none() {
            cur_win().w_buffer = curbuf.get();
        }
        quit_now();
    }
    // Autocommands may have moved to another tab page.
    if curtab.get() != prev_curtab && valid_win_any_tab(win.raw()) && win.buffer_or_none().is_none()
    {
        // The window has to be closed anyway, since the buffer is gone.
        if let Some(prev) = valid_tab(prev_curtab) {
            close_othertab(win, false, prev, force);
        }
        return FAIL;
    }

    // Autocommands may have closed the window already, or closed the only other
    // window, or moved to another tab page.
    if valid_win(win.raw()).is_none() {
        return FAIL;
    }
    if only_window(win, None) && (first_tab().next().is_none() || last_win().w_floating) {
        if first_tab().next().is_some() {
            err_raw(&raw const e_floatonly as *const c_char);
        }
        unclose_win_buffer(win, bufref, did_decrement);
        return FAIL;
    }
    if close_last_tabpage_window(win, free_buf, prev_curtab) {
        return FAIL;
    }

    // Now the window really is going to close. Disallow any autocommand from
    // splitting a window, to avoid trouble.
    split_disallowed.set(split_disallowed.get() + 1);
    let was_floating = win.w_floating;
    if ui_has(kUIMultigrid) {
        ui_call_win_close(win.w_grid_alloc.handle as Integer);
    }
    if win.w_floating {
        drop_grid(win);
        debug_assert!(tabs().next().is_some(), "first_tabpage != NULL");
        if win.w_config.external {
            for mut tp in tabs() {
                if !tp.is_current() && tp.tp_curwin == win.raw() {
                    // An autocommand can still abort the closing of this
                    // window, but carrying the change out anyway is no
                    // catastrophe.
                    tp.tp_curwin = tp.tp_firstwin;
                }
            }
        }
    }

    // About to free the window: remember its final buffer for
    // `terminal_check_size`, which may have changed since the last `BufRef`
    // (`close_buffer` autocommands, say).
    let bufref = BufRef::of_raw(win.w_buffer);
    let had_cmdline_ruler = p_ru.get() != 0 && win.is_current() && win.w_status_height == 0;
    let was_current = win.is_current();

    // Free the memory the window used, and take the window that received its
    // screen space.
    let (wp, dir) = free_mem(win, None);
    let mut wp = wp.expect("the window that took the room");
    if help_window || quickfix_window {
        // Closing the help window moves the cursor back to the window that was
        // current when the snapshot was taken.
        let idx = snapshot_index(help_window);
        if let Some(prev) = snapshot_curwin(idx).and_then(|w| valid_win(w.raw())) {
            wp = prev;
        }
    }

    // Make sure `curwin` is not invalid: it causes severe trouble when printing
    // an error message, and `win_equal()` needs `curbuf` to be valid too.
    let close_curwin = was_current;
    if was_current {
        curwin.set(wp.raw());
        if wp.w_onebuf_opt.wo_pvw != 0 || is_quickfix(wp.buffer_or_none()) {
            wp = away_from_preview(wp);
        }
        curbuf.set(cur_win().w_buffer);
        // The cursor position may be invalid if the buffer changed after the
        // window was last used.
        revalidate_cursor(cur_win());
    }

    if !was_floating {
        // If the last window has a status line now and we do not want one,
        // remove it. Do this before `equal()`, which may change a height.
        update_last_status(false);
        // SAFETY: `'eadirection'` is a NUL-terminated option string.
        let ead = unsafe { *p_ead.get() } as c_int;
        if !cur_win().w_floating && p_ea.get() != 0 && (ead == 'b' as c_int || ead == dir) {
            // If the frame of the closed window contains the new current
            // window, resize only that frame; otherwise resize all windows.
            let same = cur_win().frame().fr_parent == win_frame;
            equal(Some(cur_win()), same, dir);
        } else {
            comp_positions();
            fix_scroll(false);
        }
    } else if had_cmdline_ruler && wp.w_status_height > 0 {
        redraw_cmdline.set(true); // clear the cmdline 'ruler'
    }
    if let Some(buf) = bufref.get() {
        resize_terminal(buf);
    }

    if close_curwin {
        let flags = WEE_CURWIN_INVALID as c_int
            | WEE_TRIGGER_ENTER_AUTOCMDS as c_int
            | WEE_TRIGGER_LEAVE_AUTOCMDS as c_int;
        enter_ext(wp, flags);
        if other_buffer {
            // careful: after this `wp` and `win` may be invalid!
            fire(EVENT_BUFENTER, cur_buf());
        }
    }

    if firstwin.get() == lastwin.get()
        && cur_win().w_locked
        && cur_buf().b_locked_split != 0
        && first_tab().next().is_some()
    {
        // The new `curwin` is the last window of the current tab page and is
        // already being closed. Trigger TabLeave now: once its buffer is gone
        // it is no longer safe to do so.
        fire(EVENT_TABLEAVE, cur_buf());
    }
    split_disallowed.set(split_disallowed.get() - 1);

    // After closing the help or quickfix window, try restoring the layout from
    // before it was opened.
    if help_window || quickfix_window {
        restore_layout(snapshot_index(help_window), close_curwin);
    }

    // If the window had 'diff' set and only one window with 'diff' is left in
    // the tab page, and "closeoff" is in 'diffopt', run ":diffoff!".
    if diffopt_closeoff() && had_diffmode && curtab.get() == prev_curtab {
        let diffcount = windows().filter(|w| w.w_onebuf_opt.wo_diff != 0).count();
        if diffcount == 1 {
            run_cmd(c"diffoff!".as_ptr());
        }
    }

    cur_win().w_pos_changed = true;
    if !was_floating {
        redraw_all(UPD_NOT_VALID);
    }
    OK
}

/// Which snapshot slot a help or quickfix window restores from.
fn snapshot_index(help_window: bool) -> c_int {
    if help_window {
        SNAP_HELP_IDX
    } else {
        SNAP_QUICKFIX_IDX
    }
}

/// Closing `win` would leave only floating windows: close those first.
///
/// `None` when the caller may go on, `Some(FAIL)` when it may not.
fn close_the_floats(win: Win, force: bool) -> Option<c_int> {
    if is_autocmd_window(Some(last_win())) {
        err(c"E814: Cannot close window, only autocmd window would remain".as_ptr());
        return Some(FAIL);
    }
    if !force && !can_close_floats(None) {
        err_raw(&raw const e_floatonly as *const c_char);
        return Some(FAIL);
    }
    // Close the last window until there are no floating windows left. The
    // `force` flag is not actually used when closing a floating window.
    while last_win().w_floating {
        if close(last_win(), !hides(last_win().buffer()), true) == FAIL {
            // Give up rather than loop forever.
            return Some(FAIL);
        }
    }
    if !valid_win_any_tab(win.raw()) {
        return Some(FAIL); // already closed by autocommands
    }
    // Autocommands may have closed all other tab pages; check again.
    if is_last_window(win) {
        err(e_cannot_close_last_window.as_ptr());
        return Some(FAIL);
    }
    None
}

/// What `leave_closing_window` found.
enum Leave {
    /// An autocommand invalidated the window, or aborted the script.
    Failed,
    /// The `BufLeave`/`WinLeave` events are done; `other` says whether the
    /// window taking over shows a different buffer.
    Ok { other: bool },
}

/// Leave `win`, which is the current window and about to close: fire
/// `BufLeave` and `WinLeave`, guarding the window across both.
fn leave_closing_window(win: Win) -> Leave {
    let mut win = win;
    leave_window(cur_win());

    // Guess which window is going to be the new current one. This may change
    // because of the autocommands (sigh).
    let wp = if win.w_floating {
        // SAFETY: a live window; a null tab page means the current one.
        unsafe { Win::new(win_float_find_altwin(win.raw(), ptr::null())) }
    } else {
        frame2window(alt_frame(win, None))
    };

    // Be careful: if autocommands delete the window, or leave it the last one,
    // return now.
    let mut other_buffer = false;
    if wp.w_buffer != curbuf.get() {
        reset_VIsual_and_resel(); // stop Visual mode
        other_buffer = true;
        if valid_win(win.raw()).is_none() {
            return Leave::Failed;
        }
        win.w_locked = true;
        fire(EVENT_BUFLEAVE, cur_buf());
        if valid_win(win.raw()).is_none() {
            return Leave::Failed;
        }
        win.w_locked = false;
        if is_last_window(win) {
            return Leave::Failed;
        }
    }
    win.w_locked = true;
    fire(EVENT_WINLEAVE, cur_buf());
    if valid_win(win.raw()).is_none() {
        return Leave::Failed;
    }
    win.w_locked = false;
    if is_last_window(win) {
        return Leave::Failed;
    }
    // autocmds may abort script processing
    if aborting() {
        return Leave::Failed;
    }
    Leave::Ok {
        other: other_buffer,
    }
}

/// The cursor would land on the preview or quickfix window: walk on round the
/// window list looking for one it may sit in instead.
fn away_from_preview(wp: Win) -> Win {
    let mut wp = wp;
    loop {
        wp = wp.next().unwrap_or_else(first_win);
        if wp.is_current() {
            return wp;
        }
        let hidden = wp.w_floating && (wp.w_config.hide || !wp.w_config.focusable);
        if wp.w_onebuf_opt.wo_pvw == 0 && !is_quickfix(wp.buffer_or_none()) && !hidden {
            curwin.set(wp.raw());
            return wp;
        }
    }
}

pub(crate) fn trigger_winnewpre() {
    window_lock();
    fire_named(EVENT_WINNEWPRE, ptr::null_mut(), None);
    window_unlock();
}

/// `WinClosed`, named after the window's own handle. Never re-entered.
fn fire_winclosed(mut win: Win) {
    static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);
    if RECURSIVE.get() || !event_wanted(EVENT_WINCLOSED) {
        return;
    }
    RECURSIVE.set(true);
    let mut winid = [0 as c_char; NUMBUFLEN as usize];
    number_into(&mut winid, c"%d".as_ptr(), win.handle);
    fire_named(EVENT_WINCLOSED, winid.as_mut_ptr(), win.buffer_or_none());
    RECURSIVE.set(false);
}

pub unsafe extern "C" fn trigger_tabclosedpre(tp: *mut tabpage_T) {
    tabclosedpre(tp);
}

/// `TabClosedPre` for `tp`, fired from inside that tab page and never
/// re-entered. Comes back to the tab page it started in, or to the first.
fn tabclosedpre(tp: *mut tabpage_T) {
    static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);
    let ptp = curtab.get();
    // Return quickly when there is no TabClosedPre autocommand to run, or one
    // is already running.
    if !event_wanted(EVENT_TABCLOSEDPRE) || RECURSIVE.get() {
        return;
    }
    if let Some(tp) = valid_tab(tp) {
        goto_tab(tp, false, false);
    }
    RECURSIVE.set(true);
    window_lock();
    fire_named(EVENT_TABCLOSEDPRE, ptr::null_mut(), None);
    window_unlock();
    RECURSIVE.set(false);
    // The tab page may have been modified or deleted by the autocommands: try
    // to recover it, and fall back to the first tab page.
    // SAFETY: `first_tabpage` is set from startup to exit.
    let back = valid_tab(ptp).unwrap_or_else(|| unsafe { TabPage::new(first_tabpage.get()) });
    goto_tab(back, false, false);
}

pub unsafe extern "C" fn win_close_othertab(
    win: *mut win_T,
    free_buf: c_int,
    tp: *mut tabpage_T,
    force: bool,
) -> bool {
    // SAFETY: the caller's promise -- a live window and a live tab page.
    let (win, tp) = unsafe { (Win::new(win), TabPage::new(tp)) };
    close_othertab(win, free_buf != 0, tp, force)
}

/// Close window `win` in tab page `tp`, which is not the current one.
///
/// This may be the last window of that tab page and so close the tab page,
/// which makes `tp` invalid. The caller must check whether the buffer is
/// hidden and whether the tabline needs updating.
///
/// `false` when the window was not closed as a direct result of this call
/// (through autocommands, say).
pub(crate) fn close_othertab(win: Win, free_buf: bool, tp: TabPage, force: bool) -> bool {
    let mut win = win;
    let mut tp = tp;
    debug_assert!(!tp.is_current(), "tp != curtab");
    let mut did_decrement = false;
    let mut bufref = BufRef::of_raw(ptr::null_mut());

    // Commands that may call this already check the lock, but check again just
    // in case.
    if layout_locked(CMD_SIZE) {
        return false;
    }
    // Get here with `win->w_buffer == NULL` when `close()` detects that the tab
    // page changed.
    if win.w_locked || win.buffer_or_none().is_some_and(|buf| buf.b_locked > 0) {
        return false; // window is already being closed
    }
    if is_autocmd_window(Some(win)) {
        err(&raw const e_autocmd_close as *const c_char);
        return false;
    }

    'leave_open: {
        // Would closing this window leave only floating windows?
        if tab_last_win(tp).w_floating && only_window(win, Some(tp)) {
            if !force && !can_close_floats(Some(tp)) {
                err_raw(&raw const e_floatonly as *const c_char);
                break 'leave_open;
            }
            // Close the last window until there are no floating windows left.
            // The `force` flag is not actually used for a floating window.
            while tab_last_win(tp).w_floating {
                let last = tab_last_win(tp);
                if !close_othertab(last, !hides(last.buffer()), tp, true) {
                    // Give up rather than loop forever.
                    break 'leave_open;
                }
            }
            if !valid_win_any_tab(win.raw()) {
                return false; // already closed by autocommands
            }
        }

        // Fire WinClosed just before freeing window-related resources. With no
        // buffer it is not safe to trigger autocommands, and `close()` will
        // already have fired WinClosed.
        if win.buffer_or_none().is_some() {
            fire_winclosed(win);
            // The autocommand may have freed the window already.
            if !valid_win_any_tab(win.raw()) {
                return false;
            }
        }
        if tp.tp_firstwin == tp.tp_lastwin && !tp.tp_did_tabclosedpre {
            tabclosedpre(tp.raw());
            // The autocommand may have freed the window already.
            if !valid_win_any_tab(win.raw()) {
                return false;
            }
        }

        bufref = BufRef::of_raw(win.w_buffer);
        if let Some(mut buf) = win.buffer_or_none() {
            // Close the link to the buffer.
            let action = if free_buf { DOBUF_UNLOAD as c_int } else { 0 };
            let (w, b) = (win.raw(), buf.raw());
            // SAFETY: a live window and its own live buffer.
            did_decrement = unsafe { close_buffer(w, b, action, false, true) };
        }

        // Careful: autocommands may have closed the tab page, or made it the
        // current one.
        if valid_tab(tp.raw()).is_none() || tp.is_current() {
            break 'leave_open;
        }
        // Autocommands may have closed the window already, or
        // `nvim_win_set_config` moved it to a different tab page.
        if !valid_win_in_tab(tp, win.raw()) {
            break 'leave_open;
        }
        // Autocommands may again leave only floats; check again, but this time
        // without bothering to close them.
        if tab_last_win(tp).w_floating && only_window(win, Some(tp)) {
            err_raw(&raw const e_floatonly as *const c_char);
            break 'leave_open;
        }

        // When closing the last window of a tab page, remove the tab page.
        let mut free_tp_idx = 0;
        if tp.tp_firstwin == tp.tp_lastwin {
            free_tp_idx = tab_index(tp);
            let h = tabline_rows();
            if tp.raw() == first_tabpage.get() {
                first_tabpage.set(tp.tp_next);
            } else {
                let Some(mut ptp) = tabs().find(|ptp| ptp.tp_next == tp.raw()) else {
                    // SAFETY: a static message naming the caller.
                    unsafe { internal_error(c"win_close_othertab()".as_ptr()) };
                    return false;
                };
                ptp.tp_next = tp.tp_next;
            }
            redraw_tabline.set(true);
            if h != tabline_rows() {
                new_screen_rows();
            }
        }

        // About to free the window: remember its final buffer for
        // `terminal_check_size` and TabClosed, which may have changed since the
        // last `BufRef` (`close_buffer` autocommands, say).
        bufref = BufRef::of_raw(win.w_buffer);
        free_mem(win, Some(tp));
        if let Some(buf) = bufref.get() {
            resize_terminal(buf);
        }
        if free_tp_idx > 0 {
            free_tab(tp);
            if event_wanted(EVENT_TABCLOSED) {
                let mut prev_idx = [0 as c_char; NUMBUFLEN as usize];
                number_into(&mut prev_idx, c"%i".as_ptr(), free_tp_idx);
                let buf = bufref.get().unwrap_or_else(cur_buf);
                fire_named(EVENT_TABCLOSED, prev_idx.as_mut_ptr(), Some(buf));
            }
        }
        return true;
    }

    if let Some(win) = valid_win_any_tab(win.raw()).then_some(win) {
        unclose_win_buffer(win, bufref, did_decrement);
    }
    false
}

/// The last window of `tp`, floats included. `tp` is never the current tab
/// page here, so `tp_lastwin` is the live answer.
fn tab_last_win(tp: TabPage) -> Win {
    // SAFETY: the tail of a live window list is a live window.
    unsafe { Win::new(tp.tp_lastwin) }
}

/// Whether `buf` is a help buffer.
fn is_help(buf: Option<Buf>) -> bool {
    let raw = buf.map_or(ptr::null(), |b| b.raw() as *const buf_T);
    // SAFETY: a live buffer, or the null the callers pass for "no buffer".
    unsafe { bt_help(raw) }
}

/// Whether any autocommand is listening for `event`.
fn event_wanted(event: event_T) -> bool {
    // SAFETY: reads the autocommand tables.
    unsafe { has_event(event) }
}

/// `vim_snprintf(buf, sizeof(buf), fmt, n)` for the one-number event names.
fn number_into(buf: &mut [c_char; NUMBUFLEN as usize], fmt: *const c_char, n: c_int) {
    let (dst, len) = (buf.as_mut_ptr(), buf.len() as size_t);
    // SAFETY: a buffer of its own length, and a format taking one `int`.
    unsafe { vim_snprintf(dst, len, fmt, n) };
}

/// Exit the editor: every window is gone.
fn quit_now() -> ! {
    // SAFETY: never returns; tears the editor down.
    unsafe { getout(0) }
}
