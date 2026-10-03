//! Finding, opening and closing the quickfix window.
//!
//! [`ex_copen`] opens it — [`goto_cwindow`] when it is already there,
//! [`open_new_cwindow`] when it is not — and fills it through
//! `fill.rs`; [`ex_cwindow`] does that only when there is something to
//! show. [`qf_find_win`]/[`qf_find_buf`] are how the rest of the quickfix
//! code asks whether the window (or just its buffer) exists, and
//! [`qf_win_pos_update`] keeps its cursor on the current entry.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::buffer::find_buf;
use crate::cursor::check_cursor;
use crate::eval::vars::set_internal_string_var_to;
use crate::ex_cmds::{EcmdFlags, edit_buffer_number};
use crate::option::vars::P_QFTF;
use crate::option::{boolean_optval, callback_from_option};
use crate::types::{OptError, OptionSetFlags};
use crate::window::{
    WSP_BELOW, WSP_BOT, WSP_NEWLOC, WSP_QUICKFIX, WSP_VERT, close, goto_win, setheight_win,
    setwidth_win, split, tabline_rows, valid_win,
};
use crate::winlayer::TabPage;
use crate::winlayer::graph::switch_to;
use crate::winlayer::{Buf, Win, tab_windows, windows};
use core::ffi::{CStr, c_int};

/// An option value holding a string constant.
pub(crate) const fn string_optval(text: &'static CStr) -> OptVal {
    OptVal::static_string(text)
}

/// Call `wanted` on every window of every tab page, answering the first one
/// it accepts.
pub(crate) fn find_tab_win(mut wanted: impl FnMut(Win) -> bool) -> Option<Win> {
    tab_windows().find(|&wp| wanted(wp))
}

// ---------------------------------------------------------------------------
// Finding the window and the buffer.

/// Whether `win` is showing the stack `qi`.
///
/// A window showing the quickfix buffer has no `w_llist_ref`; one showing a
/// location list buffer names the stack it shows.
fn is_qf_win(win: Win, qi: Qi) -> bool {
    win.surviving_buffer().is_some()
        && win.shows_quickfix_buffer()
        && (qi.is_quickfix() && win.w_llist_ref.is_none()
            || qi.kind == QFLT_LOCATION && win.w_llist_ref == Some(qi.id()))
}

/// The window showing `qi` in the current tab page, if there is one.
pub(crate) fn qf_find_win(qi: Qi) -> Option<Win> {
    windows().find(|&win| is_qf_win(win, qi))
}

/// The buffer the stack is shown in, from any tab page, if there is one.
pub(crate) fn qf_find_buf(mut qi: Qi) -> Option<Buf> {
    if qi.bufnr != INVALID_QFBUFNR {
        if let Some(qfbuf) = find_buf(qi.bufnr) {
            return Some(qfbuf);
        }
        // The buffer is no longer present.
        qi.bufnr = INVALID_QFBUFNR;
    }
    tab_windows()
        .find(|&win| is_qf_win(win, qi))
        .map(Win::buffer)
}

// ---------------------------------------------------------------------------
// Opening the window.

/// Go to the window showing `qi`, answering whether there was one.
///
/// With `resize` it is also given the size the command asked for, unless
/// there is no room for it below.
fn goto_cwindow(qi: Qi, resize: bool, sz: c_int, vertsplit: bool) -> bool {
    let Some(win) = qf_find_win(qi) else {
        return false;
    };
    goto_win(win);
    if resize {
        if vertsplit {
            if sz != win.w_width {
                setwidth_win(sz, Win::current());
            }
        } else if sz != win.w_height
            && win.w_height + win.w_hsep_height + win.w_status_height + tabline_rows()
                < cmdline_row.get()
        {
            setheight_win(sz, Win::current());
        }
    }
    true
}

/// Set the options the buffer in a quickfix or location list window wants.
///
/// Must be called with the quickfix window current.
fn set_cwindow_options() {
    let local = OptionSetFlags::LOCAL;
    let off = boolean_optval(Some(false));
    set_option_value_give_err(kOptSwapfile, off, local);
    set_option_value_give_err(kOptBuftype, string_optval(c"quickfix"), local);
    set_option_value_give_err(kOptBufhidden, string_optval(c"hide"), local);
    // RESET_BINDING: no 'scrollbind'/'cursorbind', and never a diff.
    let mut win = Win::current();
    win.w_onebuf_opt.wo_scb = c_int::from(false);
    win.w_onebuf_opt.wo_crb = c_int::from(false);
    win.w_onebuf_opt.wo_diff = c_int::from(false);
    set_option_value_give_err(kOptFoldmethod, string_optval(c"manual"), local);
}

/// Open a new quickfix or location list window, load the quickfix buffer
/// and set the window's options. Answers false when there was no room.
fn open_new_cwindow(mut qi: Qi, height: c_int) -> bool {
    let oldwin = Win::current();
    let prevtab = TabPage::current_raw();
    // Looked up before the split, and read after it: upstream does the same,
    // so an autocommand that wipes the quickfix buffer during `win_split`
    // leaves this naming a buffer that is gone either way.
    let qf_buf = qf_find_buf(qi).map(|buf| buf.handle);
    // The current window becomes the previous window afterwards.
    let win = Win::current().id();

    if split(height, split_flags(qi)).is_err() {
        return false; // not enough room for the window
    }
    // RESET_BINDING.
    let mut new = Win::current();
    new.w_onebuf_opt.wo_scb = c_int::from(false);
    new.w_onebuf_opt.wo_crb = c_int::from(false);

    if qi.kind == QFLT_LOCATION {
        // The location list window references the stack it shows.
        new.w_llist_ref = Some(qi.id());
        qi.refcount.retain();
    }

    // Don't store info when the split above left us in another window.
    let oldwin = (oldwin == Win::current()).then(|| oldwin.id());
    let hide = EcmdFlags::HIDE | EcmdFlags::NOWINENTER;
    match qf_buf {
        // Use the existing quickfix buffer.
        Some(bufnr) => {
            if edit_buffer_number(bufnr, 1, hide | EcmdFlags::OLDBUF, oldwin).is_err() {
                return false;
            }
        }
        // Create a new quickfix buffer and remember its number.
        None => {
            if edit_buffer_number(0, 1, hide, oldwin).is_err() {
                return false;
            }
            qi.bufnr = Buf::current().handle;
        }
    }

    // Set the options for the quickfix buffer/window even if the buffer
    // was already present: an autocommand may have :bdeleted it since.
    if !Win::current().shows_quickfix_buffer() {
        set_cwindow_options();
    }

    // Only set the height when still in the same tab page and there is
    // no window to the side.
    if TabPage::current_raw() == prevtab && Win::current().w_width == Columns.get() {
        setheight_win(height, Win::current());
    }
    Win::current().w_onebuf_opt.wo_wfh = c_int::from(true); // 'winfixheight'
    if let Some(win) = valid_win(win) {
        prevwin.set(Some(win.id()));
    }
    true
}

/// Which half of the current window the new one takes.
///
/// The default is below the current window or at the bottom, except when
/// `:belowright` or `:aboveleft` was used. A quickfix window — but not a
/// location list one — also snapshots the layout, so closing it can restore
/// what it covered.
fn split_flags(qi: Qi) -> c_int {
    let mut flags = if cmdmod.with(|m| m.cmod_split) != 0 {
        0
    } else if qi.is_quickfix() {
        WSP_BOT.cast_signed()
    } else {
        WSP_BELOW.cast_signed()
    };
    flags |= WSP_NEWLOC.cast_signed();
    if qi.is_quickfix() {
        flags |= WSP_QUICKFIX.cast_signed();
    }
    flags
}

/// Set `w:quickfix_title` from the title of `qi`'s current list, if it has
/// one.
///
/// Must be called with the quickfix window current.
fn set_list_title(qi: Qi) {
    // A copy: setting a variable can run a watcher.
    let title = qi.current_list().title.clone();
    if let Some(title) = title {
        set_internal_string_var_to(c"w:quickfix_title", title.as_cstr());
    }
}

/// Set `w:quickfix_title` in every window showing the stack, in every tab
/// page.
pub(crate) fn qf_update_win_titlevar(qi: Qi) {
    let save_curwin = Win::current();
    // `set_list_title` only writes a window variable, so the window list is
    // stable across the walk.
    for win in tab_windows() {
        if is_qf_win(win, qi) {
            win.make_current();
            set_list_title(qi);
        }
    }
    save_curwin.make_current();
}

/// `:copen`/`:lopen`: open a window showing the list.
pub fn ex_copen(excmd: &mut ExArg) {
    let Some(qi) = stack_for_cmd(excmd.cmdidx, true) else {
        return;
    };
    let busy = QuickfixBusy::hold();

    let (addr_count, line2) = (excmd.addr_count, excmd.line2);
    let height = if addr_count != 0 {
        line2 as c_int
    } else {
        QF_WINHEIGHT.cast_signed()
    };
    reset_visual_and_resel(); // stop Visual mode

    // Find an existing quickfix window, or open a new one.
    let vertical = cmdmod.with(|m| m.cmod_split) & WSP_VERT.cast_signed() != 0;
    let found =
        cmdmod.with(|m| m.cmod_tab) == 0 && goto_cwindow(qi, addr_count != 0, height, vertical);
    if !found && !open_new_cwindow(qi, height) {
        drop(busy);
        return;
    }

    set_list_title(qi);
    // Save the current index here: updating the buffer may free the list.
    let lnum = qi.current_list().index;

    qf_fill_buffer(
        qi.current_slot(),
        Buf::current(),
        None,
        Win::current().handle,
    );

    drop(busy);

    let mut win = Win::current();
    win.w_cursor.lnum = LineNr::from(lnum);
    win.w_cursor.col = 0;
    check_cursor(win);
    win.update_topline(); // scroll to show the line
}

/// `:cwindow`/`:lwindow`: open the window if there is something to show,
/// close it if there is not.
pub fn ex_cwindow(excmd: &mut ExArg) {
    let Some(qi) = stack_for_cmd(excmd.cmdidx, true) else {
        return;
    };
    let win = qf_find_win(qi);
    let nothing = {
        let qfl = qi.current_list();
        qi.is_empty() || qfl.no_valid || qfl.is_empty()
    };
    if nothing {
        if win.is_some() {
            ex_cclose(excmd);
        }
    } else if win.is_none() {
        ex_copen(excmd);
    }
}

/// `:cclose`/`:lclose`: close the window showing the list.
pub fn ex_cclose(excmd: &mut ExArg) {
    let Some(qi) = stack_for_cmd(excmd.cmdidx, false) else {
        return;
    };
    if let Some(win) = qf_find_win(qi) {
        close(win, false, false);
    }
}

// ---------------------------------------------------------------------------
// The cursor in the window.

/// The window is made current for the cursor move only; nothing in between
/// can leave it current.
fn win_goto_line(mut win: Win, lnum: LineNr) {
    let saved = switch_to(win);
    win.w_cursor.lnum = lnum;
    win.w_cursor.col = 0;
    win.w_cursor.coladd = 0;
    win.w_curswant = 0;
    win.update_topline(); // scroll to show the line
    win.redraw_later(UPD_VALID);
    win.w_redr_status = true; // update ruler
    saved.restore();
}

/// `:cbottom`/`:lbottom`: put the cursor on the last line of the window.
pub fn ex_cbottom(excmd: &mut ExArg) {
    let Some(qi) = stack_for_cmd(excmd.cmdidx, true) else {
        return;
    };
    if let Some(win) = qf_find_win(qi) {
        let last = win.buffer().b_ml.ml_line_count;
        if win.w_cursor.lnum != last {
            win_goto_line(win, last);
        }
    }
}

impl Win {
    /// The line of this quickfix window holding the current entry, which is
    /// what the display code highlights. The window must be showing a
    /// quickfix buffer.
    pub(crate) fn quickfix_current_line(self) -> LineNr {
        let qi = match self.w_llist_ref {
            // In a location list window, the referenced list is the one.
            Some(shown) if self.is_location_list_window() => shown.stack(),
            _ => Qi::global(),
        };
        LineNr::from(qi.current_list().index)
    }
}

/// Put the cursor of the quickfix window on the current entry, answering
/// whether there is such a window.
pub(crate) fn qf_win_pos_update(qi: Qi, old_qf_index: c_int) -> bool {
    let qf_index = qi.current_list().index;
    let Some(mut win) = qf_find_win(qi) else {
        return false;
    };
    if LineNr::from(qf_index) <= win.buffer().b_ml.ml_line_count && old_qf_index != qf_index {
        // Both the old and the new line need redrawing.
        win.w_redraw_top = LineNr::from(old_qf_index.min(qf_index));
        win.w_redraw_bot = LineNr::from(old_qf_index.max(qf_index));
        win_goto_line(win, LineNr::from(qf_index));
    }
    true
}

/// Process the `'quickfixtextfunc'` option value.
pub fn did_set_quickfixtextfunc(_args: &mut OptSet) -> Result<(), OptError> {
    // A copy, and a fresh callback swapped in afterwards: the value can be
    // an expression, and evaluating it can run user code that sets the
    // option again.
    let value = P_QFTF.get();
    let Ok(cb) = callback_from_option(value.as_cstr()) else {
        return Err((e_invarg).into());
    };
    let mut old = qftf_cb.with_mut(|slot| core::mem::replace(slot, cb.unwrap_or(Callback::None)));
    old.clear();
    Ok(())
}
