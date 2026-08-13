//! `:ball` -- one window per buffer.
//!
//! [`ex_buffer_all`] opens a window for every listed buffer (or closes the
//! extra ones for `:unhide`), splitting until the count or `'winheight'` says
//! to stop, reusing windows that already show the right buffer, and loading
//! each buffer as its window is entered.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;
use core::ptr;

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::autocmd::is_aucmd_win;
use crate::src::nvim::ex_cmds2::autowrite;
use crate::src::nvim::ex_eval::{aborting, enter_cleanup, leave_cleanup};
use crate::src::nvim::getchar::vgetc;
use crate::src::nvim::main::{
    Columns, Rows, autocmd_no_enter, autocmd_no_leave, cmdmod, first_tabpage, firstbuf, firstwin,
    got_int, jop_flags, lastwin, p_ch, p_ea, p_tpm, swap_exists_action, swap_exists_did_quit,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::normal::reset_VIsual_and_resel;
use crate::src::nvim::options::kOptJopFlagClean;
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::types::{
    CMD_sunhide, CMD_unhide, OptInt, bufref_T, cleanup_T, exarg_T, except_T, linenr_T, tabpage_T,
    win_T,
};
use crate::src::nvim::undo::bufIsChanged;
use crate::src::nvim::window::{
    WSP_BELOW, WSP_ROOM, WSP_VERT, global_stl_height, goto_tabpage_tp, lastwin_nofloating,
    tabline_height, tabpage_index, win_close, win_enter, win_locked, win_move_after, win_split,
    win_valid,
};
use crate::src::nvim::winlayer::{Buf, TabPage, Win};

// ---------------------------------------------------------------------------
// The neighbours, wrapped
//
// window.rs, the mark stack and the exception machinery are all still
// transpiled `unsafe extern "C"` functions over raw pointers; each is reached
// through one wrapper here rather than through an `unsafe` at every call
// site.

fn win_of(win: *mut win_T) -> Option<Win> {
    // SAFETY: every pointer reached below comes from the window list, so it
    // is null or a live window.
    (!win.is_null()).then(|| unsafe { Win::new(win) })
}

fn first_win() -> Win {
    // SAFETY: `firstwin` is set from startup to exit.
    unsafe { Win::new(firstwin.get()) }
}

fn last_win() -> Win {
    // SAFETY: `lastwin` is set from startup to exit.
    unsafe { Win::new(lastwin.get()) }
}

fn current_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

fn current_tab() -> TabPage {
    // SAFETY: `curtab` is set from startup to exit.
    unsafe { TabPage::current() }
}

fn set_pcmark() {
    // SAFETY: pushes the current position on the jump list.
    unsafe { setpcmark() };
}

/// Make `tp` the current tab page, with autocommands.
fn goto_tab(tp: *mut tabpage_T) {
    // SAFETY: a live tab page.
    unsafe { goto_tabpage_tp(tp, true, true) };
}

/// The last non-floating window of the current tab page.
fn last_nofloat() -> *mut win_T {
    // SAFETY: null asks for the current tab page.
    unsafe { lastwin_nofloating(ptr::null_mut()) }
}

fn enter_win(win: *mut win_T) {
    // SAFETY: a live window.
    unsafe { win_enter(win, false) };
}

fn close_win(mut win: Win, free_buf: bool) -> c_int {
    // SAFETY: a live window.
    unsafe { win_close(win.raw(), free_buf, false) }
}

fn move_win_after(mut win: Win, mut after: Win) {
    // SAFETY: two live windows.
    unsafe { win_move_after(win.raw(), after.raw()) };
}

fn split_below_room() -> c_int {
    // SAFETY: splits the current window; the flags are upstream's.
    unsafe { win_split(0, WSP_ROOM as c_int | WSP_BELOW as c_int) }
}

fn is_locked(mut win: Win) -> bool {
    // SAFETY: a live window.
    unsafe { win_locked(win.raw()) != 0 }
}

/// Whether `win` is still in the window list -- asked about a pointer
/// autocommands may already have freed, which is why it does not take a
/// [`Win`].
fn is_valid(win: *mut win_T) -> bool {
    // SAFETY: `win_valid` walks the window list and does not dereference its
    // argument.
    unsafe { win_valid(win) }
}

fn is_aucmd(mut win: Win) -> bool {
    // SAFETY: a live window.
    unsafe { is_aucmd_win(win.raw()) }
}

fn tab_index() -> c_int {
    // SAFETY: null asks for the current tab page's index.
    unsafe { tabpage_index(ptr::null_mut()) }
}

fn tabline_rows() -> c_int {
    // SAFETY: reads the 'showtabline' globals.
    unsafe { tabline_height() }
}

fn global_stl_rows() -> c_int {
    // SAFETY: reads the 'laststatus' globals.
    unsafe { global_stl_height() }
}

fn buf_changed(mut buf: Buf) -> bool {
    // SAFETY: a live buffer.
    unsafe { bufIsChanged(buf.raw()) }
}

fn buf_hidden(mut buf: Buf) -> bool {
    // SAFETY: a live buffer.
    unsafe { buf_hide(buf.raw()) }
}

fn auto_write(mut buf: Buf) -> c_int {
    // SAFETY: a live buffer; `false` is upstream's `forceit`.
    unsafe { autowrite(buf.raw(), false) }
}

fn get_key() {
    // SAFETY: reads one key through the main input machinery.
    unsafe { vgetc() };
}

/// Make `buf` the current buffer, as `:buffer` does.
fn goto_buf(mut buf: Buf) {
    let update_jumplist = jop_flags.get() & kOptJopFlagClean as c_int as u32 == 0;
    // SAFETY: a live buffer.
    unsafe { set_curbuf(buf.raw(), DOBUF_GOTO as c_int, update_jumplist) };
}

/// A `bufref_T` for `buf`, the record that survives an autocommand.
fn bufref_of(mut buf: Buf) -> bufref_T {
    let mut bufref = bufref_T::default();
    // SAFETY: a local to fill in, and a live buffer.
    unsafe { set_bufref(&raw mut bufref, buf.raw()) };
    bufref
}

fn still_valid(bufref: &mut bufref_T) -> bool {
    // SAFETY: a `bufref_T` this function set.
    unsafe { bufref_valid(bufref) }
}

/// The swap-file dialogue's aftermath, when the user did not choose Quit.
fn handled_swap_exists() {
    // SAFETY: null means "no buffer to restore".
    unsafe { handle_swap_exists(ptr::null_mut()) };
}

/// Reset the error/interrupt/exception state around closing a window, so
/// that `aborting()` answers false while it happens.
fn with_clean_error_state(f: impl FnOnce()) {
    let mut cs = cleanup_T {
        pending: 0,
        exception: ptr::null_mut::<except_T>(),
    };
    // SAFETY: a local the matching `leave_cleanup` below hands back.
    unsafe { enter_cleanup(&raw mut cs) };
    f();
    // SAFETY: the state `enter_cleanup` saved.
    unsafe { leave_cleanup(&raw mut cs) };
}

// ---------------------------------------------------------------------------
// :ball

/// Open a window for every listed buffer, closing the superfluous ones.
pub unsafe fn ex_buffer_all(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- the command being executed.
    let eap = unsafe { &*eap };
    let mut split_ret = OK;
    let mut open_wins = 0;
    let had_tab = cmdmod.with(|m| m.cmod_tab);

    // The maximum number of windows to open: as many as possible, or as many
    // as the count asked for.
    let count: linenr_T = if eap.addr_count == 0 {
        9999 as linenr_T
    } else {
        eap.line2
    };

    // Whether to load inactive buffers too.
    let all = eap.cmdidx != CMD_unhide && eap.cmdidx != CMD_sunhide;

    // Stop Visual mode: the cursor and "VIsual" may very well be invalid
    // after switching to another buffer.
    reset_VIsual_and_resel();
    set_pcmark();

    close_superfluous_windows(had_tab, &mut open_wins);

    // Go through the buffer list. When a buffer doesn't have a window yet,
    // open one; otherwise move the window to the right position. Watch out
    // for autocommands that delete buffers or windows.
    //
    // Don't execute Win/Buf Enter/Leave autocommands here.
    autocmd_no_enter.set(autocmd_no_enter.get() + 1);
    // `lastwin` may be the autocommand window.
    enter_win(last_nofloat());
    autocmd_no_leave.set(autocmd_no_leave.get() + 1);

    let mut buf = Some(first_buf());
    while let Some(b) = buf {
        if (open_wins as linenr_T) >= count {
            break;
        }
        if !open_window_for(b, all, had_tab, &mut split_ret, &mut open_wins) {
            break;
        }
        buf = b.next();
    }

    autocmd_no_enter.set(autocmd_no_enter.get() - 1);
    // Back to the first window.
    enter_win(firstwin.get());
    autocmd_no_leave.set(autocmd_no_leave.get() - 1);

    close_extra_windows(count, &mut open_wins);
}

fn first_buf() -> Buf {
    // SAFETY: `firstbuf` is set from startup to exit.
    unsafe { Buf::new(firstbuf.get()) }
}

/// The first stage: close the windows showing a buffer twice, and the ones
/// that are not full width.
fn close_superfluous_windows(had_tab: c_int, open_wins: &mut c_int) {
    if had_tab > 0 {
        goto_tab(first_tabpage.get());
    }
    loop {
        let mut tpnext = current_tab().tp_next;
        // Try to close floating windows first.
        let mut wp = if last_win().w_floating {
            lastwin.get()
        } else {
            firstwin.get()
        };
        while let Some(w) = win_of(wp) {
            let mut wpnext = if w.w_floating {
                // SAFETY: a float always has a predecessor, as upstream's
                // unguarded `wp->w_prev->w_floating` assumes.
                if unsafe { Win::new(w.w_prev) }.w_floating {
                    w.w_prev
                } else {
                    firstwin.get()
                }
            } else if w.w_next.is_null() || win_of(w.w_next).is_some_and(|n| n.w_floating) {
                ptr::null_mut()
            } else {
                w.w_next
            };
            if should_close(w, had_tab) {
                if close_win(w, false) == FAIL {
                    break;
                }
                // Just in case an autocommand does something strange with
                // windows: start all over.
                wpnext = if last_win().w_floating {
                    lastwin.get()
                } else {
                    firstwin.get()
                };
                tpnext = first_tabpage.get();
                *open_wins = 0;
            } else {
                *open_wins += 1;
            }
            wp = wpnext;
        }

        // Without the ":tab" modifier only do the current tab page.
        if had_tab == 0 || tpnext.is_null() {
            break;
        }
        goto_tab(tpnext);
    }
}

/// Whether `win` is one of the superfluous windows the first stage closes.
fn should_close(win: Win, had_tab: c_int) -> bool {
    let too_small = if cmdmod.with(|m| m.cmod_split) & WSP_VERT as c_int != 0 {
        ((win.w_height + win.w_hsep_height + win.w_status_height) as OptInt)
            < Rows.get() as OptInt
                - p_ch.get()
                - tabline_rows() as OptInt
                - global_stl_rows() as OptInt
    } else {
        win.w_width != Columns.get()
    };
    let buf = win.buffer();
    (buf.b_nwindows > 1 || win.w_floating || too_small || had_tab > 0 && win != first_win())
        && first_win() != last_win()
        && !(is_locked(win) || buf.b_locked > 0)
        && !is_aucmd(win)
}

/// One buffer of the second stage. Answers false when the walk must stop.
fn open_window_for(
    buf: Buf,
    all: bool,
    had_tab: c_int,
    split_ret: &mut c_int,
    open_wins: &mut c_int,
) -> bool {
    // Check whether this buffer needs a window.
    if !all && buf.b_ml.ml_mfp.is_null() || buf.b_p_bl == 0 {
        return true;
    }

    let wp = if had_tab != 0 {
        // With the ":tab" modifier don't move the window.
        (buf.b_nwindows > 0).then(last_win)
    } else {
        // Check whether this buffer already has a window.
        let mut wp = None;
        let mut w = win_of(firstwin.get());
        while let Some(win) = w {
            if !win.w_floating && win.w_buffer == buf.raw() {
                wp = Some(win);
                break;
            }
            w = win_of(win.w_next);
        }
        // If the buffer already has a window, move it.
        if let Some(win) = wp {
            move_win_after(win, current_win());
        }
        wp
    };

    if wp.is_none() && *split_ret == OK {
        let mut bufref = bufref_of(buf);
        // Split the window and put the buffer in it.
        let p_ea_save = p_ea.get();
        // Use space from all windows.
        p_ea.set(true_0);
        *split_ret = split_below_room();
        *open_wins += 1;
        p_ea.set(p_ea_save);
        if *split_ret == FAIL {
            return true;
        }

        // Open the buffer in this window.
        swap_exists_action.set(SEA_DIALOG);
        goto_buf(buf);
        if !still_valid(&mut bufref) {
            // Autocommands deleted the buffer.
            swap_exists_action.set(SEA_NONE);
            return false;
        }
        if swap_exists_action.get() == SEA_QUIT {
            // The user selected Quit at the ATTENTION prompt; close this
            // window.
            with_clean_error_state(|| {
                close_win(current_win(), true);
                *open_wins -= 1;
                swap_exists_action.set(SEA_NONE);
                swap_exists_did_quit.set(true);
            });
        } else {
            handled_swap_exists();
        }
    }

    os_breakcheck();
    if got_int.get() {
        // Only break the file loading, not the rest.
        get_key();
        return false;
    }
    // Autocommands deleted the buffer or aborted script processing.
    if aborting() {
        return false;
    }
    // When ":tab" was used open a new tab for a new window repeatedly.
    if had_tab > 0 && tab_index() as OptInt <= p_tpm.get() {
        cmdmod.with_mut(|m| m.cmod_tab = 9999);
    }
    true
}

/// The last stage: close the windows over the count asked for.
fn close_extra_windows(count: linenr_T, open_wins: &mut c_int) {
    let mut wp = lastwin.get();
    while *open_wins as linenr_T > count {
        let Some(win) = win_of(wp) else { break };
        let r = (buf_hidden(win.buffer())
            || !buf_changed(win.buffer())
            || auto_write(win.buffer()) == OK)
            && !is_aucmd(win);
        if !is_valid(wp) {
            // A BufWrite autocommand made the window invalid; start over.
            wp = lastwin.get();
        } else if r {
            let free_buf = !buf_hidden(win.buffer());
            close_win(win, free_buf);
            *open_wins -= 1;
            wp = lastwin.get();
        } else {
            wp = win.w_prev;
            if wp.is_null() {
                break;
            }
        }
    }
}
