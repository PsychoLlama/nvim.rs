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

use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::autocmd::{EVENT_BUFENTER, EVENT_TABENTER, EVENT_WINENTER};
use crate::buffer::{BufRef, bt_prompt, buf_valid, close_buffer, is_changed, reset_syntax};
use crate::drawscreen::UPD_NOT_VALID;
use crate::ex_cmds2::{can_abandon, dialog_changed};
use crate::keycodes::Ctrl_C;
use crate::main::{
    RedrawingDisabled, State, autocmd_busy, clear_cmdline, cmdmod, cmdwin_old_curwin,
    cmdwin_result, cmdwin_type, cmdwin_win, curbuf, curtab, curwin, e_cmdwin, e_floatonly,
    firstbuf, firstwin, lastwin, mode_displayed, p_confirm, p_write, restart_edit,
    stop_insert_mode,
};
use crate::state::MODE_INSERT;
use crate::types::{CMD_SIZE, CMOD_CONFIRM, Error, buf_T, colnr_T, linenr_T};
use crate::winlayer::tabs;

pub unsafe fn entering_window(win: *mut win_T) {
    // SAFETY: the caller's promise -- a live window.
    enter_window(unsafe { Win::new(win) });
}

/// Leaving a prompt window stops Insert mode, and remembers to restart it when
/// the window is entered again. Only matters for a prompt buffer, and never in
/// the autocommand window, which is only borrowed for the moment.
pub(crate) fn leave_window(win: Win) {
    let mut win = win;
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
fn is_prompt(mut win: Win) -> bool {
    // SAFETY: a live window's buffer.
    unsafe { bt_prompt(win.buffer().raw()) }
}

pub unsafe fn win_init_empty(wp: *mut win_T) {
    // SAFETY: the caller's promise -- a live window.
    init_empty(unsafe { Win::new(wp) });
}

/// Point `wp` at the top of an empty buffer.
pub(crate) fn init_empty(wp: Win) {
    let mut wp = wp;
    wp.redraw_later(UPD_NOT_VALID);
    wp.w_lines_valid = 0;
    wp.w_cursor.lnum = 1;
    wp.w_cursor.col = 0;
    wp.w_curswant = wp.w_cursor.col;
    wp.w_cursor.coladd = 0 as colnr_T;
    wp.w_pcmark.lnum = 1; // pcmark not cleared but set to line 1
    wp.w_pcmark.col = 0;
    wp.w_prev_pcmark.lnum = 0 as linenr_T;
    wp.w_prev_pcmark.col = 0;
    wp.w_topline = 1;
    wp.w_topfill = 0;
    wp.w_botline = 2;
    wp.w_valid = 0;
    wp.w_s = &raw mut wp.buffer().b_s;
}

/// Init the current window. Called when a new file is being edited.
pub fn curwin_init() {
    init_empty(cur_win());
}

pub unsafe fn close_windows(buf: *mut buf_T, keep_curwin: bool) {
    // SAFETY: the caller's promise -- a live buffer.
    close_all(unsafe { Buf::new(buf) }, keep_curwin);
}

/// Close every window showing `buf`, on this tab page and every other, unless
/// there is only one non-floating window left.
fn close_all(buf: Buf, keep_curwin: bool) {
    RedrawingDisabled.set(RedrawingDisabled.get() + 1);
    'theend: {
        // Start from `lastwin` to close floating windows showing the buffer
        // first. When the autocommand window is involved `win_close()` may need
        // to print an error message.
        let mut cur = Some(last_win());
        while let Some(wp) = cur {
            if !is_autocmd_window(Some(last_win())) && only_window(wp, None) {
                break;
            }
            if wp.w_buffer == buf.raw() && (!keep_curwin || !wp.is_current()) && !locked(wp) {
                if layout_locked(CMD_SIZE) {
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
                // SAFETY: a live tab page's last window, or null when it has
                // none.
                let mut cur = unsafe { Win::from_raw(tp.tp_lastwin) };
                while let Some(wp) = cur {
                    if wp.w_buffer == buf.raw() && !locked(wp) {
                        if layout_locked(CMD_SIZE) {
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
    RedrawingDisabled.set(RedrawingDisabled.get() - 1);
}

/// Whether `wp` or the buffer it shows is pinned against closing.
fn locked(mut wp: Win) -> bool {
    wp.w_locked || wp.buffer().b_locked > 0
}

pub unsafe fn last_window(win: *mut win_T) -> bool {
    // SAFETY: the caller's promise -- a live window.
    is_last_window(unsafe { Win::new(win) })
}

/// Whether `win` is the last non-floating window that exists at all.
pub(crate) fn is_last_window(win: Win) -> bool {
    only_window(win, None) && first_tab().next().is_none()
}

pub unsafe fn one_window(win: *mut win_T, tp: *mut tabpage_T) -> bool {
    // SAFETY: the caller's promise -- a live window and a live tab page or
    // null.
    unsafe { only_window(Win::new(win), TabPage::from_raw(tp)) }
}

/// Whether `win` is the only non-floating window of `tp`, or of the current
/// tab page when `tp` is `None`.
///
/// This is what to ask in place of `ONE_WINDOW`, with `firstwin` or the
/// affected window as the argument depending on the situation.
pub(crate) fn only_window(win: Win, tp: Option<TabPage>) -> bool {
    let first = tp.map_or_else(|| firstwin.get(), |tp| tp.tp_firstwin);
    // SAFETY: the head of a live window list is a live window.
    let first = unsafe { Win::new(first) };
    debug_assert!(
        tp.is_none_or(|tp| !tp.is_current()) && !first.w_floating,
        "(!tp || tp != curtab) && !first->w_floating"
    );
    first == win && win.next().is_none_or(|next| next.w_floating)
}

/// Whether the floating windows of `tp` -- `None` for the current tab page --
/// can all be closed. Do not ask while the autocommand window is in use.
pub(crate) fn can_close_floats(tp: Option<TabPage>) -> bool {
    debug_assert!(
        tp.is_none_or(|tp| !tp.is_current())
            && (tp.is_some() || !is_autocmd_window(Some(last_win()))),
        "tp != curtab && (tp || !is_aucmd_win(lastwin))"
    );
    // SAFETY: the tail of a live window list is a live window.
    let mut wp = unsafe { Win::new(tp.map_or_else(|| lastwin.get(), |tp| tp.tp_lastwin)) };
    while wp.w_floating {
        let buf = wp.buffer();
        let need_hide = is_changed(buf) && buf.b_nwindows <= 1;
        if need_hide && !hides(buf) {
            return false;
        }
        // SAFETY: a floating window is never the first, so `w_prev` is live.
        wp = unsafe { Win::new(wp.w_prev) };
    }
    true
}

pub unsafe fn can_close_in_cmdwin(win: *mut win_T, err: *mut Error) -> bool {
    // SAFETY: the caller's promise -- a live window and a writable error slot.
    unsafe { cmdwin_allows(Win::new(win), &mut *err) }
}

/// Whether, the cmdline window considered, `win` is safe to close. When it is
/// not and `win` *is* the cmdline window, that window is closed; otherwise
/// `err` is set.
fn cmdwin_allows(win: Win, err: &mut Error) -> bool {
    if cmdwin_type.get() != 0 {
        if win.raw() == cmdwin_win.get() {
            cmdwin_result.set(Ctrl_C);
            return false;
        } else if win.raw() == cmdwin_old_curwin.get() {
            set_err(err, &raw const e_cmdwin as *const c_char);
            return false;
        }
    }
    true
}

/// Close the possibly last window of a tab page, `prev_curtab` being the tab
/// page that will be closed with it.
///
/// `false` when there are other windows and nothing was done.
pub(crate) fn close_last_tabpage_window(
    win: Win,
    free_buf: bool,
    prev_curtab: *mut tabpage_T,
) -> bool {
    let mut free_buf = free_buf;
    if firstwin.get() != lastwin.get() {
        return false;
    }
    let old_curbuf = curbuf.get();
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
    if let Some(prev) = valid_tab(prev_curtab).filter(|_| curtab.get() != prev_curtab) {
        if prev.tp_firstwin == win.raw() {
            close_othertab(win, free_buf, prev, false);
        }
    }
    enter_window(cur_win());

    // `goto_tab` above did not trigger *Enter autocommands: do that now.
    fire(EVENT_WINENTER, cur_buf());
    fire(EVENT_TABENTER, cur_buf());
    if old_curbuf != curbuf.get() {
        fire(EVENT_BUFENTER, cur_buf());
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
    if is_quickfix(Some(buf)) && buf.b_nwindows == 1 {
        buf.b_p_bl = false_0;
    }
    // Close the link to the buffer.
    let bufref = BufRef::of(cur_buf());
    win.w_locked = true;
    let (w, b) = (win.raw(), buf.raw());
    // SAFETY: a live window and its own live buffer.
    let retval = unsafe { close_buffer(w, b, action, abort_if_last, true) };
    if valid_win_any_tab(win.raw()) {
        win.w_locked = false;
    }
    // Make sure `curbuf` is valid: it can become invalid if 'bufhidden' is
    // "wipe".
    if !bufref.valid() {
        curbuf.set(firstbuf.get());
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
        // SAFETY: `firstbuf` is set whenever a window exists.
        let mut first = unsafe { Buf::new(firstbuf.get()) };
        win.w_buffer = first.raw();
        first.b_nwindows += 1;
        if win.is_current() {
            curbuf.set(cur_win().w_buffer);
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

pub unsafe extern "C" fn close_others(message: c_int, forceit: c_int) {
    close_all_others(message != 0, forceit != 0);
}

/// Try to close every window but the current one, hiding their buffers if
/// 'hidden' is set or `forceit` and the buffer was changed. `:only`, `:bdel`.
fn close_all_others(message: bool, forceit: bool) {
    let old_curwin = cur_win();
    let announce = message && !autocmd_busy.get();
    if old_curwin.w_floating {
        if announce {
            err_raw(&raw const e_floatonly as *const c_char);
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
    let mut next = firstwin.get();
    while let Some(mut wp) = valid_win(next) {
        let mut nextwp = wp.w_next;
        'skip: {
            // autocommands messed this one up
            if !old_curwin.is_current() && valid_win(old_curwin.raw()).is_some() {
                curwin.set(old_curwin.raw());
                curbuf.set(cur_win().w_buffer);
            }
            if wp.is_current() {
                break 'skip; // don't close the current window
            }
            // autocommands messed this one up
            if !buf_is_valid(wp.w_buffer) && valid_win(wp.raw()).is_some() {
                wp.w_buffer = ptr::null_mut::<buf_T>();
                close(wp, false, false);
                break 'skip;
            }
            // Check whether it is allowed to abandon this window.
            let r = may_abandon(wp.buffer(), forceit);
            if valid_win(wp.raw()).is_none() {
                nextwp = firstwin.get(); // autocommands messed `wp` up
                break 'skip;
            }
            if !r {
                let confirm = p_confirm.get() != 0
                    || cmdmod.with(|m| m.cmod_flags) & CMOD_CONFIRM as c_int != 0;
                if message && confirm && p_write.get() != 0 {
                    ask_about_changes(wp.buffer());
                    if valid_win(wp.raw()).is_none() {
                        nextwp = firstwin.get(); // autocommands messed `wp` up
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

/// Whether `buf` is still on the buffer list.
fn buf_is_valid(buf: *mut buf_T) -> bool {
    // SAFETY: only compared against the buffer list, never read.
    unsafe { buf_valid(buf) }
}

/// Whether `buf` may be abandoned, saying why it may not.
fn may_abandon(mut buf: Buf, forceit: bool) -> bool {
    // SAFETY: a live buffer.
    unsafe { can_abandon(buf.raw(), forceit) }
}

/// Put up the "Save changes?" dialogue for `buf`, and act on the answer.
fn ask_about_changes(mut buf: Buf) {
    // SAFETY: a live buffer.
    unsafe { dialog_changed(buf.raw(), false) };
}
