//! Deciding whether a window may close, and closing all the others.
//!
//! [`close_windows`] closes every window showing a given buffer,
//! [`close_others`] is `:only`, and the predicates around them --
//! [`last_window`], [`one_window`], [`can_close_floats`],
//! [`can_close_in_cmdwin`] -- are the questions asked before any of it.
//! [`close_last_tabpage_window`] handles the case where the window being
//! closed is the last one on its tab page, and the
//! [`leave_window`]/[`enter_window`] pair keeps the prompt buffer's Insert
//! mode consistent across the move.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::types::AutoEvent;
use crate::types::CmdIdx;
use core::ffi::c_int;
use core::ptr;

use super::*;

use crate::autocmd::state::autocmd_busy;
use crate::buffer::{BufRef, buf_is_prompt, buf_valid, close_buffer, is_changed, reset_syntax};
use crate::drawscreen::UPD_NOT_VALID;
use crate::drawscreen::state::{clear_cmdline, mode_displayed};
use crate::ex_cmds2::{can_abandon, dialog_changed};
use crate::ex_docmd::cmdmod_has;
use crate::guard::Suppress;
use crate::keycodes::Ctrl_C;
use crate::message::{e_cmdwin, e_floatonly};
use crate::r#move::WinValid;
use crate::option::vars::{p_confirm, p_write};
use crate::state::MODE_INSERT;
use crate::state::mode::{State, restart_edit, stop_insert_mode};
use crate::types::{Buffer, CmdModFlags, ColNr, Error, FAIL, LineNr, NUL};
use crate::winlayer::graph::{
    cmdwin_old_curwin, cmdwin_result, cmdwin_type, cmdwin_win, leave_curbuf,
};
use crate::winlayer::{Win, WinId, first_buffer, first_window, tabs};

pub unsafe fn entering_window(win: Win) {
    enter_window(win);
}

/// Leaving a prompt window stops Insert mode, and remembers to restart it when
/// the window is entered again. Only matters for a prompt buffer, and never in
/// the autocommand window, which is only borrowed for the moment.
pub(crate) fn leave_window(win: Win) {
    if !is_prompt(win) || is_autocmd_window(Some(win)) {
        return;
    }
    win.buffer().b_prompt_insert = restart_edit.get();
    if restart_edit.get() != NUL && mode_displayed.get() {
        clear_cmdline.set(true); // unshow mode later
    }
    restart_edit.set(NUL);

    // When leaving (or closing) the window was done from a callback we need to
    // break out of the Insert mode loop and restart Insert mode on the way
    // back in.
    if State.get() & MODE_INSERT != 0 && !stop_insert_mode.get() {
        stop_insert_mode.set(true);
        if win.buffer().b_prompt_insert == NUL {
            win.buffer().b_prompt_insert = 'A' as c_int;
        }
    }
}

/// The other half of [`leave_window`]: restart Insert mode in a prompt window
/// if that is how it was left.
pub(crate) fn enter_window(win: Win) {
    if !is_prompt(win) || is_autocmd_window(Some(win)) {
        return;
    }
    // Switching to a prompt buffer that was in Insert mode must not stop Insert
    // mode: `leave_window` may have set it.
    if win.buffer().b_prompt_insert != NUL {
        stop_insert_mode.set(false);
    }
    if State.get() & MODE_INSERT == 0 {
        restart_edit.set(win.buffer().b_prompt_insert);
    }
}

/// Whether `win` shows a prompt buffer.
fn is_prompt(win: Win) -> bool {
    buf_is_prompt(win.buffer_or_none())
}

pub unsafe fn win_init_empty(window: Win) {
    init_empty(window);
}

/// Point `window` at the top of an empty buffer.
pub(crate) fn init_empty(window: Win) {
    let mut window = window;
    window.redraw_later(UPD_NOT_VALID);
    window.w_lines_valid = 0;
    window.w_cursor.lnum = 1;
    window.w_cursor.col = 0;
    window.w_curswant = window.w_cursor.col;
    window.w_cursor.coladd = 0 as ColNr;
    window.w_pcmark.lnum = 1; // pcmark not cleared but set to line 1
    window.w_pcmark.col = 0;
    window.w_prev_pcmark.lnum = 0 as LineNr;
    window.w_prev_pcmark.col = 0;
    window.w_topline = 1;
    window.w_topfill = 0;
    window.w_botline = 2;
    window.w_valid = WinValid::NONE;
    window.w_s = &raw mut window.buffer().b_s;
}

/// Init the current window. Called when a new file is being edited.
pub fn curwin_init() {
    init_empty(Win::current());
}

pub unsafe fn close_windows(buffer: Buf, keep_curwin: bool) {
    close_all(buffer, keep_curwin);
}

/// Close every window showing `buffer`, on this tab page and every other, unless
/// there is only one non-floating window left.
fn close_all(buffer: Buf, keep_curwin: bool) {
    let _redraw_off = Suppress::redraw();
    'theend: {
        // Start from `lastwin` to close floating windows showing the buffer
        // first. When the autocommand window is involved `win_close()` may need
        // to print an error message.
        let mut cur = Some(last_win());
        while let Some(wp) = cur {
            if !is_autocmd_window(Some(last_win())) && only_window(wp, None) {
                break;
            }
            if wp.w_buffer == buffer.raw() && (!keep_curwin || !wp.is_current()) && !locked(wp) {
                if layout_locked(CmdIdx::SIZE) {
                    break 'theend; // Only give one error message.
                }
                if close(wp, false, false) == FAIL {
                    // Give up rather than loop forever.
                    break;
                }
                // Start all over: autocommands may change the window layout.
                cur = Some(last_win());
            } else {
                cur = wp.prev();
            }
        }

        // Also check windows in other tab pages.
        let mut tab = tabs().next();
        while let Some(tp) = tab {
            let mut nexttp = tp.next();
            if !tp.is_current() {
                // Start from `tp_lastwin` to close floating windows first.
                let mut cur = tp.tp_lastwin.and_then(WinId::get);
                while let Some(wp) = cur {
                    if wp.w_buffer == buffer.raw() && !locked(wp) {
                        if layout_locked(CmdIdx::SIZE) {
                            break 'theend; // Only give one error message.
                        }
                        if !close_othertab(wp, false, tp, false) {
                            // Give up rather than loop forever.
                            break;
                        }
                        // Start all over: the tab page may be gone and
                        // autocommands may change the window layout.
                        nexttp = tabs().next();
                        break;
                    }
                    cur = wp.prev();
                }
            }
            tab = nexttp;
        }
    }
}

/// Whether `window` or the buffer it shows is pinned against closing.
fn locked(window: Win) -> bool {
    window.w_locked || window.buffer().b_locked > 0
}

pub unsafe fn last_window(win: Win) -> bool {
    is_last_window(win)
}

/// Whether `win` is the last non-floating window that exists at all.
pub(crate) fn is_last_window(win: Win) -> bool {
    only_window(win, None) && first_tab().next().is_none()
}

pub unsafe fn one_window(win: Win, tabpage: *mut Tabpage) -> bool {
    // SAFETY: the caller's promise -- a live window and a live tab page or
    // null.
    unsafe { only_window(win, TabPage::from_raw(tabpage)) }
}

/// Whether `win` is the only non-floating window of `tabpage`, or of the current
/// tab page when `tabpage` is `None`.
///
/// This is what to ask in place of `ONE_WINDOW`, with `firstwin` or the
/// affected window as the argument depending on the situation.
pub(crate) fn only_window(win: Win, tabpage: Option<TabPage>) -> bool {
    let first = match tabpage {
        Some(tp) => tp.tp_firstwin.and_then(WinId::get),
        None => first_window(),
    }
    .expect("a window list has a head");
    debug_assert!(
        tabpage.is_none_or(|tp| !tp.is_current()) && !first.w_floating,
        "(!tp || tp != curtab) && !first->w_floating"
    );
    first == win && win.next().is_none_or(|next| next.w_floating)
}

/// Whether the floating windows of `tabpage` -- `None` for the current tab page --
/// can all be closed. Do not ask while the autocommand window is in use.
pub(crate) fn can_close_floats(tabpage: Option<TabPage>) -> bool {
    debug_assert!(
        tabpage.is_none_or(|tp| !tp.is_current())
            && (tabpage.is_some() || !is_autocmd_window(Some(last_win()))),
        "tp != curtab && (tp || !is_aucmd_win(lastwin))"
    );
    let mut wp = match tabpage {
        Some(tp) => tp.tp_lastwin.and_then(WinId::get),
        None => crate::winlayer::last_window(),
    }
    .expect("a window list has a tail");
    while wp.w_floating {
        let buf = wp.buffer();
        let need_hide = is_changed(buf) && buf.b_nwindows <= 1;
        if need_hide && !hides(buf) {
            return false;
        }
        wp = wp.prev().expect("a float is never the first window");
    }
    true
}

pub unsafe fn can_close_in_cmdwin(win: Win, err: &mut Error) -> bool {
    cmdwin_allows(win, &mut *err)
}

/// Whether, the cmdline window considered, `win` is safe to close. When it is
/// not and `win` *is* the cmdline window, that window is closed; otherwise
/// `err` is set.
fn cmdwin_allows(win: Win, err: &mut Error) -> bool {
    if cmdwin_type.get() != 0 {
        if cmdwin_win.get() == Some(win.id()) {
            cmdwin_result.set(Ctrl_C);
            return false;
        } else if cmdwin_old_curwin.get() == Some(win.id()) {
            set_err(err, e_cmdwin.as_ptr());
            return false;
        }
    }
    true
}

/// Close the possibly last window of a tab page, `prev_curtab` being the tab
/// page that will be closed with it.
///
/// `false` when there are other windows and nothing was done.
pub(crate) fn close_last_tabpage_window(win: Win, free_buf: bool, prev_curtab: TabPage) -> bool {
    let mut free_buf = free_buf;
    if firstwin.get() != lastwin.get() {
        return false;
    }
    let old_curbuf = Buf::current_raw();
    if win
        .buffer_or_none()
        .is_some_and(|buf| !buf.terminal.is_null())
    {
        free_buf = false; // Don't free terminal buffers
    }

    // Closing the last window in a tab page: first go to another tab page and
    // then close the window and the tab page. That avoids `curwin` and `curtab`
    // being invalid while memory is freed, since they may be used in UI events.
    // Don't trigger *Enter autocommands yet -- they would use the wrong values,
    // so that happens below. Do trigger *Leave autocommands unless the window
    // has no buffer, in which case they have already been triggered.
    let has_buffer = win.buffer_or_none().is_some();
    goto_tab(alt_tab_page(), false, has_buffer);

    // Safety check: autocommands may have switched back to the old tab page or
    // closed the window while jumping to the other one.
    if let Some(prev) =
        valid_tab(prev_curtab.id()).filter(|_| TabPage::current_raw() != prev_curtab.raw())
        && prev.tp_firstwin == Some(win.id())
    {
        close_othertab(win, free_buf, prev, false);
    }
    enter_window(Win::current());

    // `goto_tab` above did not trigger *Enter autocommands: do that now.
    fire(AutoEvent::WinEnter, Buf::current());
    fire(AutoEvent::TabEnter, Buf::current());
    if old_curbuf != Buf::current_raw() {
        fire(AutoEvent::BufEnter, Buf::current());
    }
    true
}

/// Close the buffer of `win`, unloading it when `action` is `DOBUF_UNLOAD`
/// (zero does nothing). `abort_if_last` is passed to `close_buffer()`.
///
/// Answers whether `close_buffer()` decremented `b_nwindows`.
pub(crate) fn close_win_buffer(win: Win, action: c_int, abort_if_last: bool) -> bool {
    let mut win = win;
    let Some(mut buf) = win.buffer_or_none() else {
        return false;
    };
    // Free an independent synblock before the buffer is freed.
    reset_syntax(win);
    // When a quickfix or location list window is closed and its buffer is shown
    // in only one window, unlist the buffer.
    if buf_is_quickfix(Some(buf)) && buf.b_nwindows == 1 {
        buf.b_p_bl = 0;
    }
    // Close the link to the buffer.
    let bufref = BufRef::of(Buf::current());
    win.w_locked = true;
    let retval = close_buffer(Some(win), buf, action, abort_if_last, true);
    if valid_win_any_tab(win.id()) {
        win.w_locked = false;
    }
    // Make sure `curbuf` is valid: it can become invalid if 'bufhidden' is
    // "wipe".
    if !bufref.valid() {
        match first_buffer() {
            Some(buf) => buf.make_current(),
            None => leave_curbuf(),
        }
    }
    retval
}

/// After failing to close a window `close_win_buffer` was already called on,
/// give it a buffer again.
///
/// `bufref` names `win->w_buffer` from before that call, and `did_decrement`
/// says whether it decremented `b_nwindows`.
pub(crate) fn unclose_win_buffer(win: Win, bufref: BufRef, did_decrement: bool) {
    let mut win = win;
    let Some(mut buf) = win.buffer_or_none() else {
        // The buffer was removed from the window: it has to be given one.
        let mut first = first_buffer().expect("a window means a buffer list");
        win.w_buffer = first.raw();
        first.b_nwindows += 1;
        if win.is_current() {
            first.make_current();
        }
        init_empty(win);
        return;
    };
    if did_decrement && buf.raw() == bufref.raw() && bufref.valid() {
        // `close_buffer()` decremented the window count but the window is being
        // kept; as it still shows the buffer, put the count back.
        buf.b_nwindows += 1;
    }
}

pub unsafe fn close_others(message: c_int, forceit: c_int) {
    close_all_others(message != 0, forceit != 0);
}

/// Try to close every window but the current one, hiding their buffers if
/// 'hidden' is set or `forceit` and the buffer was changed. `:only`, `:bdel`.
fn close_all_others(message: bool, forceit: bool) {
    let old_curwin = Win::current();
    let announce = message && !autocmd_busy.get();
    if old_curwin.w_floating {
        if announce {
            err_raw(e_floatonly.as_ptr());
        }
        return;
    }
    if only_window(first_win(), None) && !last_win().w_floating {
        if announce {
            only_one_message();
        }
        return;
    }

    // Be very careful here: autocommands may change the window layout.
    let mut next = first_window().map(Win::id);
    while let Some(mut wp) = next.and_then(valid_win) {
        let mut nextwp = wp.next().map(Win::id);
        'skip: {
            // autocommands messed this one up
            if !old_curwin.is_current() && valid_win(old_curwin.id()).is_some() {
                old_curwin.make_current();
                old_curwin.buffer().make_current();
            }
            if wp.is_current() {
                break 'skip; // don't close the current window
            }
            // autocommands messed this one up
            if !buf_is_valid(wp.buffer()) && valid_win(wp.id()).is_some() {
                wp.w_buffer = ptr::null_mut::<Buffer>();
                close(wp, false, false);
                break 'skip;
            }
            // Check whether it is allowed to abandon this window.
            let r = may_abandon(wp.buffer(), forceit);
            if valid_win(wp.id()).is_none() {
                nextwp = first_window().map(Win::id); // messed up
                break 'skip;
            }
            if !r {
                let confirm = p_confirm.get() != 0 || cmdmod_has(CmdModFlags::CONFIRM);
                if message && confirm && p_write.get() != 0 {
                    ask_about_changes(wp.buffer());
                    if valid_win(wp.id()).is_none() {
                        nextwp = first_window().map(Win::id); // messed up
                        break 'skip;
                    }
                }
                if is_changed(wp.buffer()) {
                    break 'skip;
                }
            }
            let free_buf = !hides(wp.buffer()) && !is_changed(wp.buffer());
            close(wp, free_buf, false);
        }
        next = nextwp;
    }

    if message && firstwin.get() != lastwin.get() {
        err(c"E445: Other window contains changes".as_ptr());
    }
}

/// Whether `buffer` is still on the buffer list.
fn buf_is_valid(buffer: Buf) -> bool {
    buf_valid(buffer.id())
}

/// Whether `buffer` may be abandoned, saying why it may not.
fn may_abandon(buffer: Buf, forceit: bool) -> bool {
    // SAFETY: a live buffer.
    unsafe { can_abandon(buffer, forceit) }
}

/// Put up the "Save changes?" dialogue for `buffer`, and act on the answer.
fn ask_about_changes(buffer: Buf) {
    // SAFETY: a live buffer.
    unsafe { dialog_changed(buffer, false) };
}
