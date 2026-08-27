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

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::ex_docmd::{cmdmod_split, cmdmod_tab};
use crate::optionstr::empty_option;
use crate::types::{FAIL, OK};
use crate::window::{WSP_ABOVE, WSP_HELP, WSP_NEWLOC, WSP_TOP};
use crate::winlayer::{Win, tabs, windows, windows_in_tab};
use core::ffi::{c_int, c_uint};
use core::ptr;

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

/// A window that is not a quickfix window and uses this location list.
pub(crate) fn qf_find_win_with_loclist(ll: *const qf_info_T) -> Option<Win> {
    find_win(|wp| wp.w_llist == ll.cast_mut() && !is_qf_buffer(wp))
}

/// A window showing an ordinary file.
pub(crate) fn qf_find_win_with_normal_buf() -> Option<Win> {
    find_win(is_normal_buffer)
}

/// Give a window a location list, taking a reference to it.
pub(crate) fn win_set_loclist(mut wp: Win, mut qi: Qi) {
    debug_assert!(wp.w_llist.is_null(), "the window already holds a list");
    wp.w_llist = qi.raw();
    qi.qf_refcount += 1;
}

/// Find a help window, or split one off, and enter it.
///
/// # Safety
///
/// `qi` must be a live stack and `opened_window` writable.
pub(crate) unsafe fn jump_to_help_window(
    qi: *mut qf_info_T,
    newwin: bool,
    opened_window: *mut bool,
) -> c_int {
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let qi = unsafe { Qi::new(qi) };
    // SAFETY: forwarded from the caller.
    let wp = if cmdmod_tab() != 0 || newwin {
        None
    } else {
        qf_find_help_win()
    };
    // SAFETY: a live window's buffer is a live buffer.
    if let Some(wp) = wp.filter(|wp| unsafe { (*wp.w_buffer).b_nwindows } > 0) {
        // SAFETY: a live window, from the window list.
        unsafe { win_enter(wp.raw(), true) };
        restart_edit.set(0);
        return OK;
    }

    // Put the split at the very top when no position was asked for and
    // the current window is one of a narrow vertical split.
    let mut flags = WSP_HELP as c_int;
    if cmdmod_split() == 0 && cur_win().w_width != Columns.get() && cur_win().w_width < 80 {
        flags |= WSP_TOP as c_int;
    }
    // A new window asked for by the user gets its own copy of the
    // location list; otherwise it shares this one.
    let share_loclist = qi.qfl_type == QFLT_LOCATION && !newwin;
    if share_loclist {
        flags |= WSP_NEWLOC as c_int;
    }
    if win_split(0, flags) == FAIL {
        return FAIL;
    }
    unsafe { *opened_window = true };
    if (cur_win().w_height as OptInt) < p_hh.get() {
        win_setheight(p_hh.get() as c_int);
    }
    if share_loclist {
        win_set_loclist(cur_win(), qi);
    }
    // Do not want insert mode in a help file.
    restart_edit.set(0);
    OK
}

/// Go to a window showing the buffer, in any tab page.
pub(crate) fn qf_goto_tabwin_with_file(fnum: c_int) -> bool {
    for tp in tabs() {
        for wp in windows_in_tab(tp) {
            if wp.buffer().handle == fnum {
                // SAFETY: a live tab page and one of its live windows. The
                // walk stops here, so the lists it was reading may move.
                unsafe { goto_tabpage_win(tp.raw(), wp.raw()) };
                return true;
            }
        }
    }
    false
}

/// Split a window above the quickfix window to show a file in, when the
/// quickfix window is all there is.
///
/// # Safety
///
/// `ll_ref` must be null or a live location list stack.
unsafe fn qf_open_new_file_win(ll_ref: *mut qf_info_T) -> c_int {
    // SAFETY: forwarded from the caller.
    let mut flags = WSP_ABOVE as c_int;
    if !ll_ref.is_null() {
        flags |= WSP_NEWLOC as c_int;
    }
    if win_split(0, flags) == FAIL {
        // Not enough room for a window.
        return FAIL;
    }
    // Do not split again for the next entry.
    p_swb.set(empty_option());
    swb_flags.set(0);
    cur_win().w_onebuf_opt.wo_scb = false as c_int;
    cur_win().w_onebuf_opt.wo_crb = false as c_int;
    if !ll_ref.is_null() {
        // The new window shows the location list window's list.
        // SAFETY: the caller's promise -- a live stack, tested for null.
        win_set_loclist(cur_win(), unsafe { Qi::new(ll_ref) });
    }
    OK
}

/// Enter a window to show a file in, jumping from a *location list* window.
///
/// The caller may already have found one; otherwise it is the window showing
/// the file, or failing that the nearest previous window holding an ordinary
/// buffer.
///
/// # Safety
///
/// `ll_ref` must be null or a live location list stack.
unsafe fn qf_goto_win_with_ll_file(use_win: Option<Win>, qf_fnum: c_int, ll_ref: *mut qf_info_T) {
    let win = use_win
        .or_else(|| find_win(|wp| wp.buffer().handle == qf_fnum))
        .unwrap_or_else(|| {
            // Walk backwards from here, wrapping at the top, for a window
            // holding an ordinary buffer.
            let mut win = cur_win();
            while !is_normal_buffer(win) {
                win = prev_window(win);
                if win == cur_win() {
                    break;
                }
            }
            win
        });
    // SAFETY: a live window, from the window list.
    unsafe { win_goto(win.raw()) };
    // A window that has no location list of its own adopts the one the
    // location list window was showing.
    if win.w_llist.is_null() && !ll_ref.is_null() {
        // SAFETY: the caller's promise -- a live stack, tested for null.
        win_set_loclist(win, unsafe { Qi::new(ll_ref) });
    }
}

/// The window before `wp` in the current tab page's list, wrapping round to
/// the last: the step of the two backwards walks below.
fn prev_window(wp: Win) -> Win {
    let prev = if wp.w_prev.is_null() {
        lastwin.get()
    } else {
        wp.w_prev
    };
    // SAFETY: `w_prev`/`lastwin` are links of the live window list, and the
    // list is never empty.
    unsafe { Win::new(prev) }
}

/// Enter a window to show a file in, jumping from a *quickfix* window.
///
/// Walks backwards from the current window, wrapping at the top, until it
/// finds the file or comes back round to the quickfix window; in that case
/// it settles for the previously used window (`'switchbuf'` `uselast`), the
/// best ordinary window seen on the way, or whichever window neighbours the
/// quickfix window.
///
fn qf_goto_win_with_qfl_file(qf_fnum: c_int) {
    let mut win = cur_win();
    let mut altwin: Option<Win> = None;
    while win.buffer().handle != qf_fnum {
        win = prev_window(win);
        if is_qf_window(win) {
            win = if swb_flags.get() & kOptSwbFlagUselast as c_uint != 0
                && win_valid(prevwin.get())
                // SAFETY: `win_valid` just established it.
                && unsafe { (*prevwin.get()).w_onebuf_opt.wo_wfb } == 0
            {
                // SAFETY: as the test above.
                unsafe { Win::new(prevwin.get()) }
            } else if let Some(altwin) = altwin {
                altwin
            } else {
                // The quickfix window is not the only one here -- the
                // caller splits one off when it is -- so it has a
                // neighbour on one side or the other.
                let neighbour = if cur_win().w_prev.is_null() {
                    cur_win().w_next
                } else {
                    cur_win().w_prev
                };
                // SAFETY: a link of the live window list, non-null by the
                // reasoning above.
                unsafe { Win::new(neighbour) }
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
    // SAFETY: a live window, from the window list.
    unsafe { win_goto(win.raw()) };
}

/// Enter a window that can show the file an entry names, splitting one off
/// when there is none — or always, with `newwin`.
///
/// # Safety
///
/// `opened_window` must be writable; it is set when a window was split, so
/// that the caller can close it again if the jump then fails.
pub(crate) unsafe fn qf_jump_to_usable_window(
    qf_fnum: c_int,
    newwin: bool,
    opened_window: *mut bool,
) -> c_int {
    // SAFETY: forwarded from the caller.
    // A new window must not share the location list the current window
    // is showing, or two windows would refer to the same one.
    let ll_ref = if newwin {
        ptr::null_mut()
    } else {
        cur_win().w_llist_ref
    };
    let usable_wp = (!ll_ref.is_null())
        .then(|| qf_find_win_with_loclist(ll_ref))
        .flatten();
    // Upstream throws the window away and keeps only the answer to
    // "is there one", so a window showing an ordinary buffer does not
    // become the one jumped to; `qf_goto_win_*` looks again.
    let mut usable_win = usable_wp.is_some() || qf_find_win_with_normal_buf().is_some();
    if !usable_win && swb_flags.get() & kOptSwbFlagUsetab as c_uint != 0 {
        usable_win = qf_goto_tabwin_with_file(qf_fnum);
    }

    let only_the_quickfix_window =
        firstwin.get() == lastwin.get() && unsafe { bt_quickfix(curbuf.get()) };
    if only_the_quickfix_window || !usable_win || newwin {
        if unsafe { qf_open_new_file_win(ll_ref) } != OK {
            return FAIL;
        }
        // Close it again if the jump fails.
        unsafe { *opened_window = true };
    } else if !cur_win().w_llist_ref.is_null() {
        unsafe { qf_goto_win_with_ll_file(usable_wp, qf_fnum, ll_ref) };
    } else {
        qf_goto_win_with_qfl_file(qf_fnum);
    }
    OK
}
