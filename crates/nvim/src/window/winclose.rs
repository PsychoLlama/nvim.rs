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

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::autocmd::defs::AutoEvent;
use crate::buffer::DOBUF_UNLOAD;
use crate::types::CmdIdx;
use core::ffi::c_int;
use std::ffi::CString;

use super::*;
use crate::autocmd::has_event;
use crate::buffer::{BufRef, buf_is_help, close_buffer};
use crate::diff::diffopt_closeoff;
use crate::drawscreen::UPD_NOT_VALID;
use crate::drawscreen::state::{REDRAW_CMDLINE, REDRAW_TABLINE};
use crate::ex_eval::aborting;
use crate::global_cell::GlobalCell;
use crate::guard::Lock;
use crate::message::E_FLOATONLY;
use crate::message::internal_error;
use crate::normal::reset_visual_and_resel;
use crate::option::vars::{p_ea, p_ead, p_ru};
use crate::startup::getout;
use crate::types::Integer;
use crate::ui::state::UI_MULTIGRID;
use crate::ui::{ui_call_win_close, ui_has};
use crate::winfloat::win_float_find_altwin;
use crate::winlayer::graph::{FIRST_TABPAGE, FIRSTWIN, LASTWIN};
use crate::winlayer::{FrameId, WinId, tabs};

/// A window that is still open after [`win_close`] was asked to close it.
///
/// Whatever stopped it -- the last window, a locked layout, a window already
/// being closed, an autocommand that closed or moved it first -- has said so
/// when it had something to say; most of those refusals are silent, as
/// upstream's are, so this is no "already reported" `Failed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the window was not closed")]
pub(crate) struct NotClosed;

pub(crate) fn win_close(win: Win, free_buf: bool, force: bool) -> Result<(), NotClosed> {
    close(win, free_buf, force)
}

/// Close window `win`, which must be on the current tab page, unloading its
/// buffer with `free_buf`.
///
/// Called by `:quit`, `:close`, `:xit`, `:wq` and `findtag()`.
pub(crate) fn close(win: Win, free_buf: bool, force: bool) -> Result<(), NotClosed> {
    let mut closing = Closing::new(win, free_buf, force);
    may_close(win, force)?;

    // When closing the last window in a tab page first go to another tab page
    // and then close the window and the tab page, so that `curwin` and `CURTAB`
    // are never invalid while memory is freed.
    if close_last_tabpage_window(win, free_buf, closing.prev_curtab) {
        return Err(NotClosed);
    }
    closing.drop_stale_snapshots(win);

    if win.is_current() {
        match leave_closing_window(win) {
            Leave::Failed => return Err(NotClosed),
            Leave::Ok { other } => closing.other_buffer = other,
        }
    }

    // Fire WinClosed just before starting to free window-related resources.
    fire_winclosed(win);
    // The autocommand may have freed the window already.
    if !valid_win_any_tab(win.id()) {
        return Ok(());
    }
    closing.release_buffer(win)?;

    // Now the window really is going to close. Disallow any autocommand from
    // splitting a window, to avoid trouble.
    let no_split = Lock::held(&SPLIT_DISALLOWED);
    let freed = closing.free_window(win);
    closing.share_out_room(&freed);
    closing.enter_next(&freed);
    drop(no_split);
    closing.restore_after(&freed);
    Ok(())
}

/// The refusals that come before anything is touched: the last window, a
/// locked layout, a window already being closed, the autocommand window.
/// Closing the last non-floating window closes the floats first.
fn may_close(win: Win, force: bool) -> Result<(), NotClosed> {
    if is_last_window(win) {
        err(WinError::LastWindow);
        return Err(NotClosed);
    }
    if !win.w_floating && layout_locked(CmdIdx::close) {
        return Err(NotClosed);
    }
    if win.w_locked || win.buffer_or_none().is_some_and(|buf| buf.b_locked > 0) {
        return Err(NotClosed); // window is already being closed
    }
    if is_autocmd_window(Some(win)) {
        err(WinError::AutocmdClose);
        return Err(NotClosed);
    }
    if last_win().w_floating && only_window(win, None) {
        close_the_floats(win, force)?;
    }
    Ok(())
}

/// What [`close`] knows of the window from before it was freed, for the
/// steps that run after.
struct Closing {
    free_buf: bool,
    force: bool,
    prev_curtab: TabPage,
    /// The frame the closing window hung off, held as an identity because
    /// `win` and its whole subtree are freed before it is compared.
    win_frame: Option<FrameId>,
    had_diffmode: bool,
    help_window: bool,
    quickfix_window: bool,
    /// Leaving the window entered another buffer: `BufEnter` is owed.
    other_buffer: bool,
}

/// What freeing the window left: the window that took its room, and what
/// the window was.
struct Freed {
    /// The window that took the room, to enter next.
    next: Win,
    /// The direction the room went (`b'v'`/`b'h'`), for 'eadirection'.
    dir: u8,
    was_floating: bool,
    /// The closed window was `curwin`: the next one must be entered.
    close_curwin: bool,
    had_cmdline_ruler: bool,
    /// The window's final buffer, for `terminal_check_size`.
    bufref: BufRef,
}

impl Closing {
    fn new(win: Win, free_buf: bool, force: bool) -> Closing {
        Closing {
            free_buf,
            force,
            prev_curtab: TabPage::current(),
            win_frame: if win.w_floating {
                None
            } else {
                win.frame().fr_parent
            },
            had_diffmode: win.w_onebuf_opt.wo_diff != 0,
            help_window: false,
            quickfix_window: false,
            other_buffer: false,
        }
    }

    /// When closing the help window, try restoring a snapshot afterwards.
    /// Otherwise clear the snapshot, which is now invalid. The same for the
    /// quickfix window.
    fn drop_stale_snapshots(&mut self, win: Win) {
        self.help_window = buf_is_help(win.buffer_or_none());
        if !self.help_window {
            drop_snapshot(TabPage::current(), SNAP_HELP_IDX);
        }
        self.quickfix_window = buf_is_quickfix(win.buffer_or_none());
        if !self.quickfix_window {
            drop_snapshot(TabPage::current(), SNAP_QUICKFIX_IDX);
        }
    }

    /// Let go of the window's buffer, then re-check everything its
    /// autocommands may have changed: all windows closed (quit), another
    /// tab page entered, the window or its only sibling closed.
    fn release_buffer(&self, win: Win) -> Result<(), NotClosed> {
        let bufref = BufRef::of_opt(win.buffer_or_none());
        let action = if self.free_buf { DOBUF_UNLOAD } else { 0 };
        let did_decrement = close_win_buffer(win, action, true);

        if valid_win(win.id()).is_some()
            && win.buffer_or_none().is_none()
            && !win.w_floating
            && is_last_window(win)
        {
            // Autocommands have closed all windows, quit now. Restore
            // `curwin->w_buffer`, or writing the ShaDa file may fail.
            if Win::current().buffer_or_none().is_none() {
                Win::current().w_buffer = Buf::current_or_none().unwrap_or(Buf::NULL);
            }
            quit_now();
        }
        // Autocommands may have moved to another tab page.
        if !self.prev_curtab.is_current()
            && valid_win_any_tab(win.id())
            && win.buffer_or_none().is_none()
        {
            // The window has to be closed anyway, since the buffer is gone.
            if let Some(prev) = valid_tab(self.prev_curtab.id()) {
                let _ = close_othertab(win, false, prev, self.force);
            }
            return Err(NotClosed);
        }

        // Autocommands may have closed the window already, or closed the only
        // other window, or moved to another tab page.
        if valid_win(win.id()).is_none() {
            return Err(NotClosed);
        }
        if only_window(win, None) && (first_tab().next().is_none() || last_win().w_floating) {
            if first_tab().next().is_some() {
                err_raw(E_FLOATONLY);
            }
            unclose_win_buffer(win, bufref, did_decrement);
            return Err(NotClosed);
        }
        if close_last_tabpage_window(win, self.free_buf, self.prev_curtab) {
            return Err(NotClosed);
        }
        Ok(())
    }

    /// Free the window and pick the one to enter next, made `curwin` at
    /// once when the closed one was. `win` is gone afterwards.
    fn free_window(&self, win: Win) -> Freed {
        let was_floating = win.w_floating;
        if ui_has(UI_MULTIGRID) {
            ui_call_win_close(Integer::from(win.w_grid_alloc.handle));
        }
        if win.w_floating {
            drop_grid(win);
            debug_assert!(tabs().next().is_some(), "first_tabpage != NULL");
            if win.w_config.external {
                for mut tp in tabs() {
                    if !tp.is_current() && tp.tp_curwin == Some(win.id()) {
                        // An autocommand can still abort the closing of this
                        // window, but carrying the change out anyway is no
                        // catastrophe.
                        tp.tp_curwin = tp.tp_firstwin;
                    }
                }
            }
        }

        // About to free the window: remember its final buffer for
        // `terminal_check_size`, which may have changed since the last
        // `BufRef` (`close_buffer` autocommands, say).
        let bufref = BufRef::of_opt(win.buffer_or_none());
        let had_cmdline_ruler = p_ru() && win.is_current() && win.w_status_height == 0;
        let was_current = win.is_current();

        // Free the memory the window used, and take the window that received
        // its screen space.
        let (wp, dir) = free_mem(win, None);
        let mut wp = wp.expect("the window that took the room");
        if self.help_window || self.quickfix_window {
            // Closing the help window moves the cursor back to the window that
            // was current when the snapshot was taken.
            let idx = snapshot_index(self.help_window);
            if let Some(prev) = snapshot_curwin(idx).and_then(|w| valid_win(w.id())) {
                wp = prev;
            }
        }

        // Make sure `curwin` is not invalid: it causes severe trouble when
        // printing an error message, and `win_equal()` needs `CURBUF` to be
        // valid too.
        if was_current {
            wp.make_current();
            if wp.w_onebuf_opt.wo_pvw != 0 || buf_is_quickfix(wp.buffer_or_none()) {
                wp = away_from_preview(wp);
            }
            Win::current().buffer().make_current();
            // The cursor position may be invalid if the buffer changed after
            // the window was last used.
            revalidate_cursor(Win::current());
        }
        Freed {
            next: wp,
            dir,
            was_floating,
            close_curwin: was_current,
            had_cmdline_ruler,
            bufref,
        }
    }

    /// Give the closed window's room out: the status line, 'equalalways',
    /// the cmdline ruler a float covered, a terminal's size.
    fn share_out_room(&self, freed: &Freed) {
        if !freed.was_floating {
            // If the last window has a status line now and we do not want one,
            // remove it. Do this before `equal()`, which may change a height.
            update_last_status(false);
            let ead = p_ead(cstr::first);
            if !Win::current().w_floating && p_ea() && (ead == b'b' || ead == freed.dir) {
                // If the frame of the closed window contains the new current
                // window, resize only that frame; otherwise resize all
                // windows.
                let same = Win::current().frame().fr_parent == self.win_frame;
                equal(Some(Win::current()), same, freed.dir);
            } else {
                comp_positions();
                fix_scroll(false);
            }
        } else if freed.had_cmdline_ruler && freed.next.w_status_height > 0 {
            REDRAW_CMDLINE.set(true); // clear the cmdline 'ruler'
        }
        if let Some(buf) = freed.bufref.get() {
            resize_terminal(buf);
        }
    }

    /// Enter the next window when the closed one was current, with its
    /// autocommands, and fire `TabLeave` while that is still safe.
    fn enter_next(&self, freed: &Freed) {
        if freed.close_curwin {
            let flags =
                WEE_CURWIN_INVALID | WEE_TRIGGER_ENTER_AUTOCMDS | WEE_TRIGGER_LEAVE_AUTOCMDS;
            enter_ext(freed.next, flags);
            if self.other_buffer {
                // careful: after this `wp` and `win` may be invalid!
                fire(AutoEvent::BufEnter, Buf::current());
            }
        }

        if FIRSTWIN.get() == LASTWIN.get()
            && Win::current().w_locked
            && Buf::current().b_locked_split != 0
            && first_tab().next().is_some()
        {
            // The new `curwin` is the last window of the current tab page and
            // is already being closed. Trigger TabLeave now: once its buffer
            // is gone it is no longer safe to do so.
            fire(AutoEvent::TabLeave, Buf::current());
        }
    }

    /// The layout from before a help or quickfix window, ":diffoff!" for a
    /// lone 'diff' window, and the redraw.
    fn restore_after(&self, freed: &Freed) {
        // After closing the help or quickfix window, try restoring the layout
        // from before it was opened.
        if self.help_window || self.quickfix_window {
            restore_layout(snapshot_index(self.help_window), freed.close_curwin);
        }

        // If the window had 'diff' set and only one window with 'diff' is left
        // in the tab page, and "closeoff" is in 'diffopt', run ":diffoff!".
        if diffopt_closeoff() && self.had_diffmode && self.prev_curtab.is_current() {
            let diffcount = windows().filter(|w| w.w_onebuf_opt.wo_diff != 0).count();
            if diffcount == 1 {
                run_cmd(c"diffoff!");
            }
        }

        Win::current().w_pos_changed = true;
        if !freed.was_floating {
            redraw_all(UPD_NOT_VALID);
        }
    }
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
/// `Err` when the window may not be closed after all.
fn close_the_floats(win: Win, force: bool) -> Result<(), NotClosed> {
    if is_autocmd_window(Some(last_win())) {
        err(WinError::OnlyAutocmdWindow);
        return Err(NotClosed);
    }
    if !force && !can_close_floats(None) {
        err_raw(E_FLOATONLY);
        return Err(NotClosed);
    }
    // Close the last window until there are no floating windows left. The
    // `force` flag is not actually used when closing a floating window.
    while last_win().w_floating {
        if close(last_win(), !hides(last_win().buffer()), true).is_err() {
            // Give up rather than loop forever.
            return Err(NotClosed);
        }
    }
    if !valid_win_any_tab(win.id()) {
        return Err(NotClosed); // already closed by autocommands
    }
    // Autocommands may have closed all other tab pages; check again.
    if is_last_window(win) {
        err(WinError::LastWindow);
        return Err(NotClosed);
    }
    Ok(())
}

/// What `leave_closing_window` found.
#[derive(Debug)]
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
    leave_window(Win::current());

    // Guess which window is going to be the new current one. This may change
    // because of the autocommands (sigh).
    let wp = if win.w_floating {
        // `None` means the current tab page.
        win_float_find_altwin(win, None)
            .expect("a float being closed always has a window to fall back to")
    } else {
        alt_frame(win, None).first_window()
    };

    // Be careful: if autocommands delete the window, or leave it the last one,
    // return now.
    let mut other_buffer = false;
    if !wp.w_buffer.is_current() {
        reset_visual_and_resel(); // stop Visual mode
        other_buffer = true;
        if valid_win(win.id()).is_none() {
            return Leave::Failed;
        }
        win.w_locked = true;
        fire(AutoEvent::BufLeave, Buf::current());
        if valid_win(win.id()).is_none() {
            return Leave::Failed;
        }
        win.w_locked = false;
        if is_last_window(win) {
            return Leave::Failed;
        }
    }
    win.w_locked = true;
    fire(AutoEvent::WinLeave, Buf::current());
    if valid_win(win.id()).is_none() {
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
fn away_from_preview(window: Win) -> Win {
    let mut window = window;
    loop {
        window = window.next().unwrap_or_else(first_win);
        if window.is_current() {
            return window;
        }
        let hidden = window.w_floating && (window.w_config.hide || !window.w_config.focusable);
        if window.w_onebuf_opt.wo_pvw == 0 && !buf_is_quickfix(window.buffer_or_none()) && !hidden {
            window.make_current();
            return window;
        }
    }
}

pub(crate) fn trigger_winnewpre() {
    window_lock();
    fire_named(AutoEvent::WinNewPre, None, None);
    window_unlock();
}

/// `WinClosed`, named after the window's own handle. Never re-entered.
fn fire_winclosed(win: Win) {
    static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);
    if RECURSIVE.get() || !event_wanted(AutoEvent::WinClosed) {
        return;
    }
    RECURSIVE.set(true);
    let winid = number_name(win.handle.get());
    fire_named(AutoEvent::WinClosed, Some(&winid), win.buffer_or_none());
    RECURSIVE.set(false);
}

pub fn trigger_tabclosedpre(tabpage: TabPage) {
    tabclosedpre(tabpage);
}

/// `TabClosedPre` for `tabpage`, fired from inside that tab page and never
/// re-entered. Comes back to the tab page it started in, or to the first.
fn tabclosedpre(tabpage: TabPage) {
    static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);
    let ptp = TabPage::current();
    // Return quickly when there is no TabClosedPre autocommand to run, or one
    // is already running.
    if !event_wanted(AutoEvent::TabClosedPre) || RECURSIVE.get() {
        return;
    }
    if let Some(tp) = valid_tab(tabpage.id()) {
        goto_tab(tp, false, false);
    }
    RECURSIVE.set(true);
    window_lock();
    fire_named(AutoEvent::TabClosedPre, None, None);
    window_unlock();
    RECURSIVE.set(false);
    // The tab page may have been modified or deleted by the autocommands: try
    // to recover it, and fall back to the first tab page.
    let back = valid_tab(ptp.id()).unwrap_or_else(first_tab);
    goto_tab(back, false, false);
}

pub(crate) fn win_close_othertab(
    win: Win,
    free_buf: bool,
    tabpage: TabPage,
    force: bool,
) -> Result<(), NotClosed> {
    close_othertab(win, free_buf, tabpage, force)
}

/// Close window `win` in tab page `tabpage`, which is not the current one.
///
/// This may be the last window of that tab page and so close the tab page,
/// which makes `tabpage` invalid. The caller must check whether the buffer is
/// hidden and whether the tabline needs updating.
///
/// [`NotClosed`] when the window was not closed as a direct result of this
/// call (through autocommands, say).
pub(crate) fn close_othertab(
    win: Win,
    free_buf: bool,
    tabpage: TabPage,
    force: bool,
) -> Result<(), NotClosed> {
    debug_assert!(!tabpage.is_current(), "tp != curtab");
    let mut did_decrement = false;
    let mut bufref = BufRef::of_opt(None);

    // Commands that may call this already check the lock, but check again just
    // in case.
    if layout_locked(CmdIdx::SIZE) {
        return Err(NotClosed);
    }
    // Get here with `win->w_buffer == NULL` when `close()` detects that the tab
    // page changed.
    if win.w_locked || win.buffer_or_none().is_some_and(|buf| buf.b_locked > 0) {
        return Err(NotClosed); // window is already being closed
    }
    if is_autocmd_window(Some(win)) {
        err(WinError::AutocmdClose);
        return Err(NotClosed);
    }

    'leave_open: {
        // Would closing this window leave only floating windows?
        if tab_last_win(tabpage).w_floating && only_window(win, Some(tabpage)) {
            if !force && !can_close_floats(Some(tabpage)) {
                err_raw(E_FLOATONLY);
                break 'leave_open;
            }
            // Close the last window until there are no floating windows left.
            // The `force` flag is not actually used for a floating window.
            while tab_last_win(tabpage).w_floating {
                let last = tab_last_win(tabpage);
                if close_othertab(last, !hides(last.buffer()), tabpage, true).is_err() {
                    // Give up rather than loop forever.
                    break 'leave_open;
                }
            }
            if !valid_win_any_tab(win.id()) {
                return Err(NotClosed); // already closed by autocommands
            }
        }

        // Fire WinClosed just before freeing window-related resources. With no
        // buffer it is not safe to trigger autocommands, and `close()` will
        // already have fired WinClosed.
        if win.buffer_or_none().is_some() {
            fire_winclosed(win);
            // The autocommand may have freed the window already.
            if !valid_win_any_tab(win.id()) {
                return Err(NotClosed);
            }
        }
        if tabpage.tp_firstwin == tabpage.tp_lastwin && !tabpage.tp_did_tabclosedpre {
            tabclosedpre(tabpage);
            // The autocommand may have freed the window already.
            if !valid_win_any_tab(win.id()) {
                return Err(NotClosed);
            }
        }

        bufref = BufRef::of_opt(win.buffer_or_none());
        if let Some(buf) = win.buffer_or_none() {
            // Close the link to the buffer.
            let action = if free_buf { DOBUF_UNLOAD } else { 0 };
            did_decrement = close_buffer(Some(win), buf, action, false, true);
        }

        // Careful: autocommands may have closed the tab page, or made it the
        // current one.
        if valid_tab(tabpage.id()).is_none() || tabpage.is_current() {
            break 'leave_open;
        }
        // Autocommands may have closed the window already, or
        // `nvim_win_set_config` moved it to a different tab page.
        if !valid_win_in_tab(tabpage, win.id()) {
            break 'leave_open;
        }
        // Autocommands may again leave only floats; check again, but this time
        // without bothering to close them.
        if tab_last_win(tabpage).w_floating && only_window(win, Some(tabpage)) {
            err_raw(E_FLOATONLY);
            break 'leave_open;
        }

        // When closing the last window of a tab page, remove the tab page.
        let mut free_tp_idx = 0;
        if tabpage.tp_firstwin == tabpage.tp_lastwin {
            free_tp_idx = tab_index(tabpage);
            let h = tabline_rows();
            if FIRST_TABPAGE.get() == Some(tabpage.id()) {
                FIRST_TABPAGE.set(tabpage.tp_next);
            } else {
                let Some(mut ptp) = tabs().find(|ptp| ptp.tp_next == Some(tabpage.id())) else {
                    internal_error(c"win_close_othertab()");
                    return Err(NotClosed);
                };
                ptp.tp_next = tabpage.tp_next;
            }
            REDRAW_TABLINE.set(true);
            if h != tabline_rows() {
                new_screen_rows();
            }
        }

        // Take the window off the UI before freeing it: a multigrid UI
        // must hear that its grid closed, and the compositor must not keep
        // a float's grid it would otherwise draw from after the free.
        if ui_has(UI_MULTIGRID) {
            ui_call_win_close(Integer::from(win.w_grid_alloc.handle));
        }
        if win.w_floating {
            drop_grid(win);
        }

        // About to free the window: remember its final buffer for
        // `terminal_check_size` and TabClosed, which may have changed since the
        // last `BufRef` (`close_buffer` autocommands, say).
        bufref = BufRef::of_opt(win.buffer_or_none());
        free_mem(win, Some(tabpage));
        if let Some(buf) = bufref.get() {
            resize_terminal(buf);
        }
        if free_tp_idx > 0 {
            free_tab(tabpage);
            if event_wanted(AutoEvent::TabClosed) {
                let prev_idx = number_name(free_tp_idx);
                let buf = bufref.get().unwrap_or_else(Buf::current);
                fire_named(AutoEvent::TabClosed, Some(&prev_idx), Some(buf));
            }
        }
        return Ok(());
    }

    if let Some(win) = valid_win_any_tab(win.id()).then_some(win) {
        unclose_win_buffer(win, bufref, did_decrement);
    }
    Err(NotClosed)
}

/// The last window of `tabpage`, floats included. `tabpage` is never the current tab
/// page here, so `tp_lastwin` is the live answer.
fn tab_last_win(tabpage: TabPage) -> Win {
    tabpage
        .tp_lastwin
        .and_then(WinId::get)
        .expect("a window list has a tail")
}

/// Whether any autocommand is listening for `event`.
fn event_wanted(event: AutoEvent) -> bool {
    has_event(event)
}

/// `n` in decimal, for the one-number event names.
fn number_name(n: c_int) -> CString {
    CString::new(n.to_string()).expect("digits")
}

/// Exit the editor: every window is gone.
fn quit_now() -> ! {
    getout(0)
}
