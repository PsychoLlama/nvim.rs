//! `:ball` -- one window per buffer.
//!
//! [`ex_buffer_all`] opens a window for every listed buffer (or closes the
//! extra ones for `:unhide`), splitting until the count or `'winheight'` says
//! to stop, reusing windows that already show the right buffer, and loading
//! each buffer as its window is entered.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::types::CmdIdx;
use core::ffi::c_int;

use super::*;
use crate::autocmd::is_aucmd_win;
use crate::buffer::state::{SWAP_EXISTS_ACTION, SWAP_EXISTS_DID_QUIT};
use crate::ex_cmds2::autowrite;
use crate::ex_docmd::state::CMDMOD;
use crate::ex_eval::defs::Cleanup;
use crate::ex_eval::{aborting, enter_cleanup, leave_cleanup};
use crate::getchar::state::GOT_INT;
use crate::getchar::vgetc;
use crate::guard::Suppress;
use crate::mark::setpcmark;
use crate::normal::reset_visual_and_resel;
use crate::option::vars::{JOP_FLAGS, P_EA, p_ch, p_ea, p_tpm};
use crate::options::kOptJopFlagClean;
use crate::os::input::os_breakcheck;
use crate::types::{ExArg, Failed, LineNr, OptInt};
use crate::ui::state::{COLUMNS, ROWS};
use crate::undo::buf_is_changed;
use crate::window::{
    NotClosed, WSP_BELOW, WSP_ROOM, WSP_VERT, global_stl_height, goto_tab as goto_tab_page,
    lastwin_nofloating, tabline_height, tabpage_index, win_close, win_enter, win_locked,
    win_move_after, win_split, win_valid,
};
use crate::winlayer::{Buf, TabPage, Win, buffers, first_tab, first_window, last_window, windows};

// ---------------------------------------------------------------------------
// The neighbours, wrapped

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
    win_enter(win, false);
}

fn close_win(win: Win, free_buf: bool) -> Result<(), NotClosed> {
    win_close(win, free_buf, false)
}

fn move_win_after(win: Win, after: Win) {
    win_move_after(win, after);
}

fn split_below_room() -> Result<(), Failed> {
    win_split(0, WSP_ROOM | WSP_BELOW)
}

fn is_locked(win: Win) -> bool {
    win_locked(win) != 0
}

/// Whether `win` is still in the window list.
///
/// By identity: the handle was taken while the window was live, so a window
/// autocommands have since freed answers `false` even where the allocator
/// has put another one at the same address.
fn is_valid(win: Win) -> bool {
    win_valid(win.id())
}

fn is_aucmd(win: Win) -> bool {
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
    buf_is_changed(buffer)
}

fn buf_hidden(buffer: Buf) -> bool {
    buf_hide(buffer)
}

fn auto_write(buffer: Buf) -> Result<(), Failed> {
    autowrite(buffer, false)
}

fn get_key() {
    // SAFETY: reads one key through the main input machinery.
    vgetc().code();
}

/// Make `buffer` the current buffer, as `:buffer` does.
fn goto_buf(buffer: Buf) {
    let update_jumplist = JOP_FLAGS.get() & kOptJopFlagClean == 0;
    set_curbuf(buffer, DOBUF_GOTO, update_jumplist);
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
        exception: None,
    };
    enter_cleanup(&mut cs);
    f();
    leave_cleanup(&mut cs);
}

// ---------------------------------------------------------------------------
// :ball

/// Open a window for every listed buffer, closing the superfluous ones.
pub fn ex_buffer_all(excmd: &mut ExArg) {
    // SAFETY: the caller's promise -- the command being executed.
    let excmd = &*excmd;
    let mut split_ret = Ok(());
    let mut open_wins = 0;
    let had_tab = CMDMOD.with(|m| m.cmod_tab);

    // The maximum number of windows to open: as many as possible, or as many
    // as the count asked for.
    let count: i32 = if excmd.addr_count == 0 {
        9999
    } else {
        excmd.number2
    };

    // Whether to load inactive buffers too.
    let all = excmd.cmdidx != CmdIdx::unhide && excmd.cmdidx != CmdIdx::sunhide;

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
        if LineNr::new(open_wins) >= LineNr::new(count) {
            break;
        }
        if !b.open_window_for(all, had_tab, &mut split_ret, &mut open_wins) {
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
                if close_win(w, false).is_err() {
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
    let too_small = if CMDMOD.with(|m| m.cmod_split) & WSP_VERT != 0 {
        OptInt::from(win.w_height + win.w_hsep_height + win.w_status_height)
            < OptInt::from(ROWS.get())
                - p_ch()
                - OptInt::from(tabline_rows())
                - OptInt::from(global_stl_rows())
    } else {
        win.w_width != COLUMNS.get()
    };
    let buf = win.buffer();
    (buf.b_nwindows > 1 || win.w_floating || too_small || had_tab > 0 && win != first_win())
        && first_win() != last_win()
        && !(is_locked(win) || buf.b_locked > 0)
        && !is_aucmd(win)
}

impl Buf {
    /// One buffer of the second stage. Answers false when the walk must stop.
    fn open_window_for(
        self,
        all: bool,
        had_tab: c_int,
        split_ret: &mut Result<(), Failed>,
        open_wins: &mut c_int,
    ) -> bool {
        // Check whether this buffer needs a window.
        if !all && self.b_ml.ml_mfp.is_none() || self.b_p_bl == 0 {
            return true;
        }

        let wp = if had_tab != 0 {
            // With the ":tab" modifier don't move the window.
            (self.b_nwindows > 0).then(last_win)
        } else {
            // Check whether this buffer already has a window.
            let wp = windows().find(|win| !win.w_floating && win.w_buffer == self);
            // If the buffer already has a window, move it.
            if let Some(win) = wp {
                move_win_after(win, current_win());
            }
            wp
        };

        if wp.is_none() && split_ret.is_ok() {
            let bufref = BufRef::of(self);
            // Split the window and put the buffer in it.
            let p_ea_save = p_ea();
            // Use space from all windows.
            P_EA.set(true);
            *split_ret = split_below_room();
            *open_wins += 1;
            P_EA.set(p_ea_save);
            if split_ret.is_err() {
                return true;
            }

            // Open the buffer in this window.
            SWAP_EXISTS_ACTION.set(SEA_DIALOG);
            goto_buf(self);
            if !bufref.valid() {
                // Autocommands deleted the buffer.
                SWAP_EXISTS_ACTION.set(SEA_NONE);
                return false;
            }
            if SWAP_EXISTS_ACTION.get() == SEA_QUIT {
                // The user selected Quit at the ATTENTION prompt; close this
                // window.
                with_clean_error_state(|| {
                    let _ = close_win(current_win(), true);
                    *open_wins -= 1;
                    SWAP_EXISTS_ACTION.set(SEA_NONE);
                    SWAP_EXISTS_DID_QUIT.set(true);
                });
            } else {
                handled_swap_exists();
            }
        }

        os_breakcheck();
        if GOT_INT.get() {
            // Only break the file loading, not the rest.
            get_key();
            return false;
        }
        // Autocommands deleted the buffer or aborted script processing.
        if aborting() {
            return false;
        }
        // When ":tab" was used open a new tab for a new window repeatedly.
        if had_tab > 0 && OptInt::from(tab_index()) <= p_tpm() {
            CMDMOD.with_mut(|m| m.cmod_tab = 9999);
        }
        true
    }
}

/// The last stage: close the windows over the count asked for.
fn close_extra_windows(count: i32, open_wins: &mut c_int) {
    let mut wp = last_window();
    while LineNr::new(*open_wins) > LineNr::new(count) {
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
            let _ = close_win(win, free_buf);
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
