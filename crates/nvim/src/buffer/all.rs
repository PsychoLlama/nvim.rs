//! `:ball` -- one window per buffer.
//!
//! [`ex_buffer_all`] opens a window for every listed buffer (or closes the
//! extra ones for `:unhide`), splitting until the count or `'winheight'` says
//! to stop, reusing windows that already show the right buffer, and loading
//! each buffer as its window is entered.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::types::CmdIdx;
use core::ffi::c_int;
use core::ptr;

use super::*;
use crate::autocmd::is_aucmd_win;
use crate::buffer::state::{swap_exists_action, swap_exists_did_quit};
use crate::ex_cmds2::autowrite;
use crate::ex_docmd::state::cmdmod;
use crate::ex_eval::{aborting, enter_cleanup, leave_cleanup};
use crate::getchar::state::got_int;
use crate::getchar::vgetc;
use crate::guard::Suppress;
use crate::mark::setpcmark;
use crate::normal::reset_visual_and_resel;
use crate::option::vars::{jop_flags, p_ch, p_ea, p_tpm};
use crate::options::kOptJopFlagClean;
use crate::os::input::os_breakcheck;
use crate::types::{Cleanup, ExArg, Exception, FAIL, Failed, LineNr, OptInt};
use crate::ui::state::{Columns, Rows};
use crate::undo::buf_is_changed;
use crate::window::{
    WSP_BELOW, WSP_ROOM, WSP_VERT, global_stl_height, goto_tab as goto_tab_page,
    lastwin_nofloating, tabline_height, tabpage_index, win_close, win_enter, win_locked,
    win_move_after, win_split,
};
use crate::winlayer::{
    Buf, TabPage, Win, buffers, first_tab, first_window, last_window, window_at, windows,
};

// ---------------------------------------------------------------------------
// The neighbours, wrapped
//
// window.rs, the mark stack and the exception machinery are all still
// transpiled `unsafe fn`s over raw pointers; each is reached
// through one wrapper here rather than through an `unsafe` at every call
// site.

/// Where `close_superfluous_windows` starts, and restarts: floats live at
/// the end of the list and are closed first.
fn walk_head() -> Option<Win> {
    match last_win().w_floating {
        true => last_window(),
        false => first_window(),
    }
}

fn first_win() -> Win {
    first_window().expect("the editor always has a window")
}

fn last_win() -> Win {
    last_window().expect("the editor always has a window")
}

fn current_win() -> Win {
    Win::current()
}

fn current_tab() -> TabPage {
    TabPage::current()
}

fn set_pcmark() {
    // SAFETY: pushes the current position on the jump list.
    setpcmark();
}

/// Make `tabpage` the current tab page, with autocommands.
fn goto_tab(tabpage: TabPage) {
    goto_tab_page(tabpage, true, true);
}

/// The last non-floating window of the current tab page.
fn last_nofloat() -> Win {
    // SAFETY: null asks for the current tab page, which always has a window.
    lastwin_nofloating(None)
}

fn enter_win(win: Win) {
    // SAFETY: a live window.
    unsafe { win_enter(win, false) };
}

fn close_win(win: Win, free_buf: bool) -> c_int {
    // SAFETY: a live window.
    unsafe { win_close(win, free_buf, false) }
}

fn move_win_after(win: Win, after: Win) {
    // SAFETY: two live windows.
    unsafe { win_move_after(win, after) };
}

fn split_below_room() -> Result<(), Failed> {
    win_split(0, WSP_ROOM as c_int | WSP_BELOW as c_int)
}

fn is_locked(win: Win) -> bool {
    // SAFETY: a live window.
    unsafe { win_locked(win) != 0 }
}

/// Whether `win` is still in the window list -- asked about a pointer
/// autocommands may already have freed, which is why it does not take a
/// [`Win`].
fn is_valid(win: Win) -> bool {
    // `window_at` walks the window list comparing addresses, which is the
    // only safe question about a window that may already be freed.
    window_at(win.raw()).is_some()
}

fn is_aucmd(win: Win) -> bool {
    // SAFETY: a live window.
    is_aucmd_win(win)
}

fn tab_index() -> c_int {
    tabpage_index(None)
}

fn tabline_rows() -> c_int {
    tabline_height()
}

fn global_stl_rows() -> c_int {
    global_stl_height()
}

fn buf_changed(buffer: Buf) -> bool {
    // SAFETY: a live buffer.
    buf_is_changed(buffer)
}

fn buf_hidden(buffer: Buf) -> bool {
    // SAFETY: a live buffer.
    unsafe { buf_hide(buffer) }
}

fn auto_write(buffer: Buf) -> Result<(), Failed> {
    // SAFETY: a live buffer; `false` is upstream's `forceit`.
    unsafe { autowrite(buffer, false) }
}

fn get_key() {
    // SAFETY: reads one key through the main input machinery.
    vgetc();
}

/// Make `buffer` the current buffer, as `:buffer` does.
fn goto_buf(buffer: Buf) {
    let update_jumplist = jop_flags.get() & kOptJopFlagClean as c_int as u32 == 0;
    // SAFETY: a live buffer.
    unsafe { set_curbuf(buffer, DOBUF_GOTO as c_int, update_jumplist) };
}

/// The swap-file dialogue's aftermath, when the user did not choose Quit.
fn handled_swap_exists() {
    // SAFETY: null means "no buffer to restore".
    handle_swap_exists(None);
}

/// Reset the error/interrupt/exception state around closing a window, so
/// that `aborting()` answers false while it happens.
fn with_clean_error_state(f: impl FnOnce()) {
    let mut cs = Cleanup {
        pending: 0,
        exception: ptr::null_mut::<Exception>(),
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
pub unsafe fn ex_buffer_all(args: *mut ExArg) {
    // SAFETY: the caller's promise -- the command being executed.
    let args = unsafe { &*args };
    let mut split_ret = Ok(());
    let mut open_wins = 0;
    let had_tab = cmdmod.with(|m| m.cmod_tab);

    // The maximum number of windows to open: as many as possible, or as many
    // as the count asked for.
    let count: LineNr = if args.addr_count == 0 {
        9999 as LineNr
    } else {
        args.line2
    };

    // Whether to load inactive buffers too.
    let all = args.cmdidx != CmdIdx::unhide && args.cmdidx != CmdIdx::sunhide;

    // Stop Visual mode: the cursor and "VIsual" may very well be invalid
    // after switching to another buffer.
    reset_visual_and_resel();
    set_pcmark();

    close_superfluous_windows(had_tab, &mut open_wins);

    // Go through the buffer list. When a buffer doesn't have a window yet,
    // open one; otherwise move the window to the right position. Watch out
    // for autocommands that delete buffers or windows.
    //
    // Don't execute Win/Buf Enter/Leave autocommands here.
    let no_enter = Suppress::win_enter_autocmds();
    // `lastwin` may be the autocommand window.
    enter_win(last_nofloat());
    let no_leave = Suppress::win_leave_autocmds();

    for b in buffers() {
        if (open_wins as LineNr) >= count {
            break;
        }
        if !open_window_for(b, all, had_tab, &mut split_ret, &mut open_wins) {
            break;
        }
    }

    // The release order is load-bearing: the window entered below fires
    // `WinEnter`/`BufEnter` but still no `WinLeave`/`BufLeave`.
    drop(no_enter);
    // Back to the first window.
    enter_win(first_win());
    drop(no_leave);

    close_extra_windows(count, &mut open_wins);
}

/// The first stage: close the windows showing a buffer twice, and the ones
/// that are not full width.
fn close_superfluous_windows(had_tab: c_int, open_wins: &mut c_int) {
    if had_tab > 0 {
        goto_tab(first_tab().expect("there is always a first tab page"));
    }
    loop {
        let mut tpnext = current_tab().next();
        // Try to close floating windows first.
        let mut wp = walk_head();
        while let Some(w) = wp {
            let mut wpnext = if w.w_floating {
                // A float always has a predecessor, as upstream's unguarded
                // `wp->w_prev->w_floating` assumes.
                let prev = w.prev().expect("a float is never the first window");
                match prev.w_floating {
                    true => Some(prev),
                    false => first_window(),
                }
            } else {
                w.next().filter(|next| !next.w_floating)
            };
            if should_close(w, had_tab) {
                if close_win(w, false) == FAIL {
                    break;
                }
                // Just in case an autocommand does something strange with
                // windows: start all over.
                wpnext = walk_head();
                tpnext = first_tab();
                *open_wins = 0;
            } else {
                *open_wins += 1;
            }
            wp = wpnext;
        }

        // Without the ":tab" modifier only do the current tab page.
        let (false, Some(tp)) = (had_tab == 0, tpnext) else {
            break;
        };
        goto_tab(tp);
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
    buffer: Buf,
    all: bool,
    had_tab: c_int,
    split_ret: &mut Result<(), Failed>,
    open_wins: &mut c_int,
) -> bool {
    // Check whether this buffer needs a window.
    if !all && buffer.b_ml.ml_mfp.is_null() || buffer.b_p_bl == 0 {
        return true;
    }

    let wp = if had_tab != 0 {
        // With the ":tab" modifier don't move the window.
        (buffer.b_nwindows > 0).then(last_win)
    } else {
        // Check whether this buffer already has a window.
        let wp = windows().find(|win| !win.w_floating && win.w_buffer == buffer.raw());
        // If the buffer already has a window, move it.
        if let Some(win) = wp {
            move_win_after(win, current_win());
        }
        wp
    };

    if wp.is_none() && split_ret.is_ok() {
        let bufref = BufRef::of(buffer);
        // Split the window and put the buffer in it.
        let p_ea_save = p_ea.get();
        // Use space from all windows.
        p_ea.set(1);
        *split_ret = split_below_room();
        *open_wins += 1;
        p_ea.set(p_ea_save);
        if split_ret.is_err() {
            return true;
        }

        // Open the buffer in this window.
        swap_exists_action.set(SEA_DIALOG);
        goto_buf(buffer);
        if !bufref.valid() {
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
fn close_extra_windows(count: LineNr, open_wins: &mut c_int) {
    let mut wp = last_window();
    while *open_wins as LineNr > count {
        let Some(win) = wp else { break };
        let r = (buf_hidden(win.buffer())
            || !buf_changed(win.buffer())
            || auto_write(win.buffer()).is_ok())
            && !is_aucmd(win);
        if !is_valid(win) {
            // A BufWrite autocommand made the window invalid; start over.
            wp = last_window();
        } else if r {
            let free_buf = !buf_hidden(win.buffer());
            close_win(win, free_buf);
            *open_wins -= 1;
            wp = last_window();
        } else {
            wp = win.prev();
            if wp.is_none() {
                break;
            }
        }
    }
}
