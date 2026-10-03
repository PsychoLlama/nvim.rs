//! Choosing the window a jump lands in.
//!
//! [`qf_jump_to_usable_window`] is what `'switchbuf'` is about: from the
//! quickfix window, find a window that can show the file — one already
//! showing it, one showing any normal buffer, one in another tab page with
//! `usetab`, the previously used one with `uselast` — and split a new one
//! above the quickfix window when there is none.
//!
//! [`jump_to_help_window`] is the same question for `:helpgrep` entries,
//! which want a help window.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::ex_docmd::{cmdmod_split, cmdmod_tab};
use crate::types::{Failed, QfId};
use crate::window::{WSP_ABOVE, WSP_HELP, WSP_NEWLOC, WSP_TOP};
use crate::winlayer::{Win, last_window, tabs, windows, windows_in_tab};
use core::ffi::{c_int, c_uint};

/// The first window of the current tab page that `wanted` accepts.
///
/// `wanted` only reads: [`windows`] walks the list front to back, so a
/// predicate that closed or reordered windows would walk off it.
fn find_win(mut wanted: impl FnMut(Win) -> bool) -> Option<Win> {
    windows().find(|&wp| wanted(wp))
}

/// A window showing a help file, that the user can reach.
pub(crate) fn qf_find_help_win() -> Option<Win> {
    find_win(|wp| is_help_buffer(wp) && !wp.w_config.hide && wp.w_config.focusable)
}

/// A window showing an ordinary file.
fn qf_find_win_with_normal_buf() -> Option<Win> {
    find_win(is_normal_buffer)
}

/// Find a help window, or split one off, and enter it.
pub(crate) fn jump_to_help_window(
    qi: Qi,
    newwin: bool,
    opened_window: &mut bool,
) -> Result<(), Failed> {
    let wp = if cmdmod_tab() != 0 || newwin {
        None
    } else {
        qf_find_help_win()
    };
    if let Some(wp) = wp.filter(|wp| wp.w_buffer.b_nwindows > 0) {
        win_enter(wp, true);
        restart_edit.set(0);
        return Ok(());
    }

    // Put the split at the very top when no position was asked for and
    // the current window is one of a narrow vertical split.
    let mut flags = WSP_HELP.cast_signed();
    if cmdmod_split() == 0 && Win::current().w_width != Columns.get() && Win::current().w_width < 80
    {
        flags |= WSP_TOP.cast_signed();
    }
    // A new window asked for by the user gets its own copy of the
    // location list; otherwise it shares this one.
    let share_loclist = qi.kind == QFLT_LOCATION && !newwin;
    if share_loclist {
        flags |= WSP_NEWLOC.cast_signed();
    }
    win_split(0, flags)?;
    *opened_window = true;
    if OptInt::from(Win::current().w_height) < p_hh() {
        win_setheight(c_int::try_from(p_hh()).unwrap_or(c_int::MAX));
    }
    if share_loclist {
        Win::current().set_location_list(qi);
    }
    // Do not want insert mode in a help file.
    restart_edit.set(0);
    Ok(())
}

/// Go to a window showing the buffer, in any tab page.
fn qf_goto_tabwin_with_file(fnum: c_int) -> bool {
    for tp in tabs() {
        for wp in windows_in_tab(tp) {
            if wp.buffer().handle == fnum {
                goto_tabpage_win(tp, wp);
                return true;
            }
        }
    }
    false
}

/// Split a window above the quickfix window to show a file in, when the
/// quickfix window is all there is. `ll_ref` is the location list the
/// location list window shows, which the new window takes.
fn qf_open_new_file_win(ll_ref: Option<QfId>) -> Result<(), Failed> {
    let mut flags = WSP_ABOVE.cast_signed();
    if ll_ref.is_some() {
        flags |= WSP_NEWLOC.cast_signed();
    }
    if win_split(0, flags).is_err() {
        // Not enough room for a window.
        return Err(Failed);
    }
    // Do not split again for the next entry.
    P_SWB.clear();
    swb_flags.set(0);
    Win::current().w_onebuf_opt.wo_scb = c_int::from(false);
    Win::current().w_onebuf_opt.wo_crb = c_int::from(false);
    if let Some(ll_ref) = ll_ref {
        // The new window shows the location list window's list.
        Win::current().set_location_list(ll_ref.stack());
    }
    Ok(())
}

/// Enter a window to show a file in, jumping from a *location list* window.
///
/// The caller may already have found one; otherwise it is the window showing
/// the file, or failing that the nearest previous window holding an ordinary
/// buffer.
fn qf_goto_win_with_ll_file(use_win: Option<Win>, qf_fnum: c_int, ll_ref: Option<QfId>) {
    let win = use_win
        .or_else(|| find_win(|wp| wp.buffer().handle == qf_fnum))
        .unwrap_or_else(|| {
            // Walk backwards from here, wrapping at the top, for a window
            // holding an ordinary buffer.
            let mut win = Win::current();
            while !is_normal_buffer(win) {
                win = prev_window(win);
                if win == Win::current() {
                    break;
                }
            }
            win
        });
    win_goto(win);
    // A window that has no location list of its own adopts the one the
    // location list window was showing.
    if win.w_llist.is_none()
        && let Some(ll_ref) = ll_ref
    {
        win.set_location_list(ll_ref.stack());
    }
}

/// The window before `window` in the current tab page's list, wrapping round to
/// the last: the step of the two backwards walks below.
fn prev_window(window: Win) -> Win {
    window
        .prev()
        .or_else(last_window)
        .expect("the window list is never empty")
}

/// Enter a window to show a file in, jumping from a *quickfix* window.
///
/// Walks backwards from the current window, wrapping at the top, until it
/// finds the file or comes back round to the quickfix window; in that case
/// it settles for the previously used window (`'switchbuf'` `uselast`), the
/// best ordinary window seen on the way, or whichever window neighbours the
/// quickfix window.
fn qf_goto_win_with_qfl_file(qf_fnum: c_int) {
    let mut win = Win::current();
    let mut altwin: Option<Win> = None;
    while win.buffer().handle != qf_fnum {
        win = prev_window(win);
        if win.is_quickfix_window() {
            let last = crate::winlayer::prev_window()
                .filter(|p| win_valid(p.id()) && p.w_onebuf_opt.wo_wfb == 0)
                .filter(|_| swb_flags.get() & kOptSwbFlagUselast as c_uint != 0);
            win = if let Some(last) = last {
                last
            } else if let Some(altwin) = altwin {
                altwin
            } else {
                // The quickfix window is not the only one here -- the
                // caller splits one off when it is -- so it has a
                // neighbour on one side or the other.
                Win::current()
                    .prev()
                    .or_else(|| Win::current().next())
                    .expect("the quickfix window has a neighbour")
            };
            break;
        }
        if altwin.is_none()
            && win.w_onebuf_opt.wo_pvw == 0
            && win.w_onebuf_opt.wo_wfb == 0
            && is_normal_buffer(win)
        {
            altwin = Some(win);
        }
    }
    win_goto(win);
}

/// Enter a window that can show the file an entry names, splitting one off
/// when there is none — or always, with `newwin`.
///
/// `opened_window` is set when a window was split, so that the caller can
/// close it again if the jump then fails.
pub(crate) fn qf_jump_to_usable_window(
    qf_fnum: c_int,
    newwin: bool,
    opened_window: &mut bool,
) -> Result<(), Failed> {
    // A new window must not share the location list the current window
    // is showing, or two windows would refer to the same one.
    let ll_ref = if newwin {
        None
    } else {
        Win::current().w_llist_ref
    };
    let usable_wp = ll_ref.and_then(qf_find_win_with_loclist);
    // Upstream throws the window away and keeps only the answer to
    // "is there one", so a window showing an ordinary buffer does not
    // become the one jumped to; `qf_goto_win_*` looks again.
    let mut usable_win = usable_wp.is_some() || qf_find_win_with_normal_buf().is_some();
    if !usable_win && swb_flags.get() & kOptSwbFlagUsetab as c_uint != 0 {
        usable_win = qf_goto_tabwin_with_file(qf_fnum);
    }

    let only_the_quickfix_window =
        firstwin.get() == lastwin.get() && buf_is_quickfix(current_buf());
    if only_the_quickfix_window || !usable_win || newwin {
        qf_open_new_file_win(ll_ref)?;
        // Close it again if the jump fails.
        *opened_window = true;
    } else if Win::current().w_llist_ref.is_some() {
        qf_goto_win_with_ll_file(usable_wp, qf_fnum, ll_ref);
    } else {
        qf_goto_win_with_qfl_file(qf_fnum);
    }
    Ok(())
}
