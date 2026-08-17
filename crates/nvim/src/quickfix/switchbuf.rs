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

#[allow(unused_imports)]
use super::*;
use crate::window::{WSP_ABOVE, WSP_HELP, WSP_NEWLOC, WSP_TOP};
use core::ffi::{c_int, c_uint};
use core::ptr;

/// The first window of the current tab page that `wanted` accepts, or null.
///
/// # Safety
///
/// `wanted` must not close or reorder windows.
unsafe fn find_win(mut wanted: impl FnMut(*mut win_T) -> bool) -> *mut win_T {
    // SAFETY: the window list is walked front to back and not modified.
    unsafe {
        let mut wp = firstwin.get();
        while !wp.is_null() {
            if wanted(wp) {
                return wp;
            }
            wp = (*wp).w_next;
        }
        ptr::null_mut()
    }
}

/// A window showing a help file, that the user can reach.
pub(crate) unsafe fn qf_find_help_win() -> *mut win_T {
    // SAFETY: `find_win` only reads.
    unsafe {
        find_win(|wp| bt_help((*wp).w_buffer) && !(*wp).w_config.hide && (*wp).w_config.focusable)
    }
}

/// A window that is not a quickfix window and uses this location list.
pub(crate) unsafe fn qf_find_win_with_loclist(ll: *const qf_info_T) -> *mut win_T {
    // SAFETY: `find_win` only reads.
    unsafe { find_win(|wp| (*wp).w_llist == ll.cast_mut() && !bt_quickfix((*wp).w_buffer)) }
}

/// A window showing an ordinary file.
pub(crate) unsafe fn qf_find_win_with_normal_buf() -> *mut win_T {
    // SAFETY: `find_win` only reads.
    unsafe { find_win(|wp| bt_normal((*wp).w_buffer)) }
}

/// Give a window a location list, taking a reference to it.
///
/// # Safety
///
/// `wp` and `qi` must be live, and `wp` must not already hold a list.
pub(crate) unsafe fn win_set_loclist(wp: *mut win_T, qi: *mut qf_info_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        (*wp).w_llist = qi;
        (*qi).qf_refcount += 1;
    }
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
    // SAFETY: forwarded from the caller.
    unsafe {
        let wp = if (*cmdmod.ptr()).cmod_tab != 0 || newwin {
            ptr::null_mut()
        } else {
            qf_find_help_win()
        };
        if !wp.is_null() && (*(*wp).w_buffer).b_nwindows > 0 {
            win_enter(wp, true);
            restart_edit.set(0);
            return OK;
        }

        // Put the split at the very top when no position was asked for and
        // the current window is one of a narrow vertical split.
        let mut flags = WSP_HELP as c_int;
        if (*cmdmod.ptr()).cmod_split == 0
            && (*curwin.get()).w_width != Columns.get()
            && (*curwin.get()).w_width < 80
        {
            flags |= WSP_TOP as c_int;
        }
        // A new window asked for by the user gets its own copy of the
        // location list; otherwise it shares this one.
        let share_loclist = (*qi).qfl_type == QFLT_LOCATION && !newwin;
        if share_loclist {
            flags |= WSP_NEWLOC as c_int;
        }
        if win_split(0, flags) == FAIL {
            return FAIL;
        }
        *opened_window = true;
        if ((*curwin.get()).w_height as OptInt) < p_hh.get() {
            win_setheight(p_hh.get() as c_int);
        }
        if share_loclist {
            win_set_loclist(curwin.get(), qi);
        }
        // Do not want insert mode in a help file.
        restart_edit.set(0);
        OK
    }
}

/// Go to a window showing the buffer, in any tab page.
pub(crate) unsafe fn qf_goto_tabwin_with_file(fnum: c_int) -> bool {
    // SAFETY: the tab and window lists are walked without being modified;
    // `goto_tabpage_win` is the last thing done.
    unsafe {
        let mut tp = first_tabpage.get();
        while !tp.is_null() {
            let mut wp = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if (*(*wp).w_buffer).handle == fnum {
                    goto_tabpage_win(tp, wp);
                    return true;
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next;
        }
        false
    }
}

/// Split a window above the quickfix window to show a file in, when the
/// quickfix window is all there is.
///
/// # Safety
///
/// `ll_ref` must be null or a live location list stack.
unsafe fn qf_open_new_file_win(ll_ref: *mut qf_info_T) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut flags = WSP_ABOVE as c_int;
        if !ll_ref.is_null() {
            flags |= WSP_NEWLOC as c_int;
        }
        if win_split(0, flags) == FAIL {
            // Not enough room for a window.
            return FAIL;
        }
        // Do not split again for the next entry.
        p_swb.set(empty_string_option.ptr().cast());
        swb_flags.set(0);
        (*curwin.get()).w_onebuf_opt.wo_scb = false as c_int;
        (*curwin.get()).w_onebuf_opt.wo_crb = false as c_int;
        if !ll_ref.is_null() {
            // The new window shows the location list window's list.
            win_set_loclist(curwin.get(), ll_ref);
        }
        OK
    }
}

/// Enter a window to show a file in, jumping from a *location list* window.
///
/// The caller may already have found one; otherwise it is the window showing
/// the file, or failing that the nearest previous window holding an ordinary
/// buffer.
///
/// # Safety
///
/// `use_win` must be null or a live window, `ll_ref` null or a live stack.
unsafe fn qf_goto_win_with_ll_file(use_win: *mut win_T, qf_fnum: c_int, ll_ref: *mut qf_info_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut win = use_win;
        if win.is_null() {
            win = find_win(|wp| (*(*wp).w_buffer).handle == qf_fnum);
        }
        if win.is_null() {
            // Walk backwards from here, wrapping at the top, for a window
            // holding an ordinary buffer.
            win = curwin.get();
            while !bt_normal((*win).w_buffer) {
                win = if (*win).w_prev.is_null() {
                    lastwin.get()
                } else {
                    (*win).w_prev
                };
                if win == curwin.get() {
                    break;
                }
            }
        }
        win_goto(win);
        // A window that has no location list of its own adopts the one the
        // location list window was showing.
        if (*win).w_llist.is_null() && !ll_ref.is_null() {
            win_set_loclist(win, ll_ref);
        }
    }
}

/// Enter a window to show a file in, jumping from a *quickfix* window.
///
/// Walks backwards from the current window, wrapping at the top, until it
/// finds the file or comes back round to the quickfix window; in that case
/// it settles for the previously used window (`'switchbuf'` `uselast`), the
/// best ordinary window seen on the way, or whichever window neighbours the
/// quickfix window.
///
/// # Safety
///
/// The window list must be live.
unsafe fn qf_goto_win_with_qfl_file(qf_fnum: c_int) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut win = curwin.get();
        let mut altwin: *mut win_T = ptr::null_mut();
        while (*(*win).w_buffer).handle != qf_fnum {
            win = if (*win).w_prev.is_null() {
                lastwin.get()
            } else {
                (*win).w_prev
            };
            if is_qf_window(win) {
                win = if swb_flags.get() & kOptSwbFlagUselast as c_uint != 0
                    && win_valid(prevwin.get())
                    && (*prevwin.get()).w_onebuf_opt.wo_wfb == 0
                {
                    prevwin.get()
                } else if !altwin.is_null() {
                    altwin
                } else if !(*curwin.get()).w_prev.is_null() {
                    (*curwin.get()).w_prev
                } else {
                    (*curwin.get()).w_next
                };
                break;
            }
            if altwin.is_null()
                && (*win).w_onebuf_opt.wo_pvw == 0
                && (*win).w_onebuf_opt.wo_wfb == 0
                && bt_normal((*win).w_buffer)
            {
                altwin = win;
            }
        }
        win_goto(win);
    }
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
    unsafe {
        // A new window must not share the location list the current window
        // is showing, or two windows would refer to the same one.
        let ll_ref = if newwin {
            ptr::null_mut()
        } else {
            (*curwin.get()).w_llist_ref
        };
        let mut usable_wp = ptr::null_mut();
        if !ll_ref.is_null() {
            usable_wp = qf_find_win_with_loclist(ll_ref);
        }
        // Upstream throws the window away and keeps only the answer to
        // "is there one", so a window showing an ordinary buffer does not
        // become the one jumped to; `qf_goto_win_*` looks again.
        let mut usable_win = !usable_wp.is_null() || !qf_find_win_with_normal_buf().is_null();
        if !usable_win && swb_flags.get() & kOptSwbFlagUsetab as c_uint != 0 {
            usable_win = qf_goto_tabwin_with_file(qf_fnum);
        }

        let only_the_quickfix_window = firstwin.get() == lastwin.get() && bt_quickfix(curbuf.get());
        if only_the_quickfix_window || !usable_win || newwin {
            if qf_open_new_file_win(ll_ref) != OK {
                return FAIL;
            }
            // Close it again if the jump fails.
            *opened_window = true;
        } else if !(*curwin.get()).w_llist_ref.is_null() {
            qf_goto_win_with_ll_file(usable_wp, qf_fnum, ll_ref);
        } else {
            qf_goto_win_with_qfl_file(qf_fnum);
        }
        OK
    }
}
