//! Finding, opening and closing the quickfix window.
//!
//! [`ex_copen`] opens it — [`goto_cwindow`] when it is already there,
//! [`open_new_cwindow`] when it is not — and fills it through
//! `fill.rs`; [`ex_cwindow`] does that only when there is something to
//! show. [`qf_find_win`]/[`qf_find_buf`] are how the rest of the quickfix
//! code asks whether the window (or just its buffer) exists, and
//! [`qf_win_pos_update`] keeps its cursor on the current entry.
//!
//! The window half of all this is `winlayer`'s: [`Win`] carries the window
//! and the walks carry the lists. The quickfix half is still transpiled, so
//! the stack and the list get a wrapper apiece here.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::buffer::{buf_valid, find_buf};
use crate::cursor::check_cursor;
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::newlnum;
use crate::option::boolean_optval;
use crate::types::{Failed, OptionSetFlags};
use crate::window::{
    WSP_BELOW, WSP_BOT, WSP_NEWLOC, WSP_QUICKFIX, WSP_VERT, close, goto_win, setheight_win,
    setwidth_win, split, tabline_rows, valid_win,
};
use crate::winlayer::TabPage;
use crate::winlayer::graph::switch_to;
use crate::winlayer::{Buf, Win, tab_windows, windows};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

// ---------------------------------------------------------------------------
// The stack and the list, which the rest of the quickfix code still hands
// around as raw pointers.

impl Qi {
    /// `IS_QF_STACK()`: the quickfix stack, rather than a location list.
    fn is_quickfix_stack(self) -> bool {
        self.qfl_type == QFLT_QUICKFIX
    }

    /// `qf_get_curlist()`: the list the stack is currently on.
    fn curlist(self) -> Qfl {
        // SAFETY: a live stack, which always has a current list.
        unsafe { Qfl::new(qf_get_curlist(self.raw())) }
    }

    /// `qf_stack_empty()`.
    fn is_empty(self) -> bool {
        // SAFETY: a live stack.
        unsafe { qf_stack_empty(self.raw()) }
    }
}

impl Qfl {
    /// `qf_list_empty()`.
    fn is_empty(self) -> bool {
        // SAFETY: a live list.
        unsafe { qf_list_empty(self.raw()) }
    }
}

// ---------------------------------------------------------------------------
// The neighbours that are still transpiled, one wrapper each.

/// An option value holding a string constant.
pub(crate) const fn string_optval(text: &'static CStr) -> OptVal {
    OptVal::static_string(text)
}

/// `buf_valid()`: whether the window still shows a buffer on the buffer list.
///
/// By identity — the question is asked about a buffer an autocommand may
/// already have freed, and a number that names nothing answers `false` where
/// an address might have been reused.
fn buf_is_valid(window: Win) -> bool {
    window
        .buffer_or_none()
        .is_some_and(|buf| buf_valid(buf.id()))
}

/// `do_ecmd()` as the quickfix window calls it: load `fnum`, or a new buffer
/// when it is zero, without entering the window.
fn load_buffer(fnum: c_int, flags: EcmdFlags, oldwin: Option<Win>) -> Result<(), Failed> {
    let no_name = ptr::null_mut();
    let one = newlnum::ONE as LineNr;
    let oldwin = oldwin.map(Win::id);
    // SAFETY: a buffer number the caller has just looked up.
    unsafe { do_ecmd(fnum, no_name, no_name, None, one, flags, oldwin) }
}

fn set_title_var(title: *mut c_char) {
    // SAFETY: a NUL-terminated title, into a window variable.
    unsafe { set_internal_string_var(c"w:quickfix_title".as_ptr(), title) };
}

fn busy_end() {
    decr_quickfix_busy();
}

/// `qf_fill_buffer()`: rewrite `buffer` from `qfl`.
fn fill_buffer(qfl: Qfl, buffer: Buf, win: Win) {
    // SAFETY: a live list, buffer and window handle.
    unsafe { qf_fill_buffer(qfl.raw(), buffer, ptr::null_mut(), win.handle) };
}

fn is_location_list_window(window: Win) -> bool {
    is_ll_window(window)
}

fn clamp_cursor(window: Win) {
    // SAFETY: a live window.
    check_cursor(window);
}

/// The stack `excmd`'s command names, or none when there is not one.
fn stack_of(excmd: &mut ExArg, print_emsg: bool) -> Option<Qi> {
    // SAFETY: forwarded from the caller.
    let qi = unsafe { qf_cmd_get_stack(excmd, print_emsg) };
    (!qi.is_null()).then_some(unsafe { Qi::new(qi) })
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
/// location list buffer points at the list it shows.
fn is_qf_win(win: Win, qi: Qi) -> bool {
    buf_is_valid(win)
        && is_qf_buffer(win)
        && (qi.is_quickfix_stack() && win.w_llist_ref.is_null()
            || qi.qfl_type == QFLT_LOCATION && ptr::eq(win.w_llist_ref, qi.raw()))
}

/// The window showing `qi` in the current tab page, if there is one.
pub(crate) fn qf_find_win(qi: Qi) -> Option<Win> {
    windows().find(|&win| is_qf_win(win, qi))
}

/// The buffer the stack is shown in, from any tab page, if there is one.
pub(crate) fn qf_find_buf(mut qi: Qi) -> Option<Buf> {
    if qi.qf_bufnr != INVALID_QFBUFNR {
        if let Some(qfbuf) = find_buf(qi.qf_bufnr) {
            return Some(qfbuf);
        }
        // The buffer is no longer present.
        qi.qf_bufnr = INVALID_QFBUFNR;
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
    win.w_onebuf_opt.wo_scb = false as c_int;
    win.w_onebuf_opt.wo_crb = false as c_int;
    win.w_onebuf_opt.wo_diff = false as c_int;
    set_option_value_give_err(kOptFoldmethod, string_optval(c"manual"), local);
}

/// Open a new quickfix or location list window, load the quickfix buffer
/// and set the window's options. Answers false when there was no room.
fn open_new_cwindow(mut qi: Qi, height: c_int) -> bool {
    let oldwin = Win::current();
    let prevtab = TabPage::current_raw();
    // Looked up before the split, and read after it: upstream does the same,
    // so an autocommand that wipes the quickfix buffer during `win_split`
    // leaves this reading a freed buffer either way.
    let qf_buf = qf_find_buf(qi);
    // The current window becomes the previous window afterwards.
    let win = Win::current().id();

    if split(height, split_flags(qi)).is_err() {
        return false; // not enough room for the window
    }
    // RESET_BINDING.
    let mut new = Win::current();
    new.w_onebuf_opt.wo_scb = false as c_int;
    new.w_onebuf_opt.wo_crb = false as c_int;

    if qi.qfl_type == QFLT_LOCATION {
        // The location list window references the stack it shows.
        new.w_llist_ref = qi.raw();
        qi.qf_refcount.retain();
    }

    // Don't store info when the split above left us in another window.
    let oldwin = (oldwin == Win::current()).then_some(oldwin);
    let hide = EcmdFlags::HIDE | EcmdFlags::NOWINENTER;
    match qf_buf {
        // Use the existing quickfix buffer.
        Some(buf) => {
            let flags = hide | EcmdFlags::OLDBUF;
            if load_buffer(buf.handle, flags, oldwin).is_err() {
                return false;
            }
        }
        // Create a new quickfix buffer and remember its number.
        None => {
            if load_buffer(0, hide, oldwin).is_err() {
                return false;
            }
            qi.qf_bufnr = Buf::current().handle;
        }
    }

    // Set the options for the quickfix buffer/window even if the buffer
    // was already present: an autocommand may have :bdeleted it since.
    if !is_qf_buffer(Win::current()) {
        set_cwindow_options();
    }

    // Only set the height when still in the same tab page and there is
    // no window to the side.
    if TabPage::current_raw() == prevtab && Win::current().w_width == Columns.get() {
        setheight_win(height, Win::current());
    }
    Win::current().w_onebuf_opt.wo_wfh = true as c_int; // 'winfixheight'
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
    } else if qi.is_quickfix_stack() {
        WSP_BOT as c_int
    } else {
        WSP_BELOW as c_int
    };
    flags |= WSP_NEWLOC as c_int;
    if qi.is_quickfix_stack() {
        flags |= WSP_QUICKFIX as c_int;
    }
    flags
}

/// Set `w:quickfix_title` from the list's title, if it has one.
///
/// Must be called with the quickfix window current.
fn set_list_title(qfl: Qfl) {
    if !qfl.qf_title.is_null() {
        set_title_var(qfl.qf_title);
    }
}

/// Set `w:quickfix_title` in every window showing the stack, in every tab
/// page.
pub(crate) fn qf_update_win_titlevar(qi: Qi) {
    let qfl = qi.curlist();
    let save_curwin = Win::current();
    // `set_list_title` only writes a window variable, so the window list is
    // stable across the walk.
    for win in tab_windows() {
        if is_qf_win(win, qi) {
            win.make_current();
            set_list_title(qfl);
        }
    }
    save_curwin.make_current();
}

/// `:copen`/`:lopen`: open a window showing the list.
pub fn ex_copen(excmd: &mut ExArg) {
    // SAFETY: the caller's promise -- a live command.
    let Some(qi) = stack_of(excmd, true) else {
        return;
    };
    incr_quickfix_busy();

    // SAFETY: the caller's promise -- a live command.
    let (addr_count, line2) = (excmd.addr_count, excmd.line2);
    let height = if addr_count != 0 {
        line2 as c_int
    } else {
        QF_WINHEIGHT as c_int
    };
    reset_visual_and_resel(); // stop Visual mode

    // Find an existing quickfix window, or open a new one.
    let vertical = cmdmod.with(|m| m.cmod_split) & WSP_VERT as c_int != 0;
    let found =
        cmdmod.with(|m| m.cmod_tab) == 0 && goto_cwindow(qi, addr_count != 0, height, vertical);
    if !found && !open_new_cwindow(qi, height) {
        busy_end();
        return;
    }

    let qfl = qi.curlist();
    set_list_title(qfl);
    // Save the current index here: updating the buffer may free the list.
    let lnum = qfl.qf_index;

    fill_buffer(qfl, Buf::current(), Win::current());

    busy_end();

    let mut win = Win::current();
    win.w_cursor.lnum = lnum as LineNr;
    win.w_cursor.col = 0;
    clamp_cursor(win);
    win.update_topline(); // scroll to show the line
}

/// `:cwindow`/`:lwindow`: open the window if there is something to show,
/// close it if there is not.
pub fn ex_cwindow(excmd: &mut ExArg) {
    // SAFETY: the caller's promise -- a live command.
    let Some(qi) = stack_of(excmd, true) else {
        return;
    };
    let qfl = qi.curlist();
    let win = qf_find_win(qi);
    if qi.is_empty() || qfl.qf_nonevalid || qfl.is_empty() {
        if win.is_some() {
            // SAFETY: the caller's promise -- a live command.
            ex_cclose(excmd);
        }
    } else if win.is_none() {
        // SAFETY: the caller's promise -- a live command.
        ex_copen(excmd);
    }
}

/// `:cclose`/`:lclose`: close the window showing the list.
pub fn ex_cclose(excmd: &mut ExArg) {
    // SAFETY: the caller's promise -- a live command.
    let Some(qi) = stack_of(excmd, false) else {
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
    // SAFETY: the caller's promise -- a live command.
    let Some(qi) = stack_of(excmd, true) else {
        return;
    };
    if let Some(win) = qf_find_win(qi) {
        let last = win.buffer().b_ml.ml_line_count;
        if win.w_cursor.lnum != last {
            win_goto_line(win, last);
        }
    }
}

/// The line of the quickfix window holding the current entry, which is what
/// the display code highlights.
///
/// `window` must be showing a quickfix buffer.
pub fn qf_current_entry(window: Win) -> LineNr {
    let mut qi = QfStack::Global.qi();
    if is_location_list_window(window) {
        // In the location list window, the referenced list is the one.
        qi = QfStack::Local(window.w_llist_ref.cast()).qi();
    }
    qi.curlist().qf_index as LineNr
}

/// Put the cursor of the quickfix window on the current entry, answering
/// whether there is such a window.
pub(crate) fn qf_win_pos_update(qi: Qi, old_qf_index: c_int) -> bool {
    let qf_index = qi.curlist().qf_index;
    let Some(mut win) = qf_find_win(qi) else {
        return false;
    };
    if qf_index as LineNr <= win.buffer().b_ml.ml_line_count && old_qf_index != qf_index {
        // Both the old and the new line need redrawing.
        win.w_redraw_top = old_qf_index.min(qf_index) as LineNr;
        win.w_redraw_bot = old_qf_index.max(qf_index) as LineNr;
        win_goto_line(win, qf_index as LineNr);
    }
    true
}

/// Process the `'quickfixtextfunc'` option value.
pub fn did_set_quickfixtextfunc(_args: &mut OptSet) -> Option<&CStr> {
    let (value, cb) = (p_qftf(), global_qftf());
    // SAFETY: the option's own value and its callback slot.
    if unsafe { option_set_callback_func(value, cb) }.is_err() {
        return Some(e_invarg);
    }
    None
}
