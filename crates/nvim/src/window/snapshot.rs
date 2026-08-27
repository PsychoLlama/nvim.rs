//! Saving and restoring a layout, revalidating it, and the odds and ends.
//!
//! The `*snapshot*` family saves the frame tree before a `:diffsplit` or a
//! help window opens and puts it back afterwards, matching the saved shape
//! against the current one ([`snapshot_matches`]) before trusting it.
//! [`check_lnums`] and [`reset_lnums`] revalidate every window's cursor and
//! topline against a buffer whose line count changed.  [`check_colorcolumn`]
//! parses `'colorcolumn'`, and [`win_ui_flush`] pushes the accumulated
//! position and viewport changes to the UI.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint};
use core::mem::size_of;
use core::{ptr, slice};

use super::*;
use crate::ascii::ascii_isdigit;
use crate::charset::getdigits_int;
use crate::drawscreen::UPD_NOT_VALID;
use crate::main::{curbuf, e_invarg};
use crate::memory::{xcalloc, xmalloc};
use crate::message::msg_ui_flush;
use crate::r#move::WinValid;
use crate::optionstr::empty_option;
use crate::popupmenu::pum_ui_flush;
use crate::pos::equalpos;
use crate::types::{Integer, NUL, OptInt, frame_T, handle_T, linenr_T, tabpage_T, win_T};
use crate::ui::ui_call_win_hide;
use crate::winlayer::{
    Buf, Frame, TabPage, Win, WinId, last_window, tab_windows, tabs, windows_in_tab,
};

// ---------------------------------------------------------------------------
// Revalidating cursors against a buffer that changed length

/// Clamp every window's cursor and topline into `curbuf`, remembering where
/// they were so [`reset_lnums`] can put them back.
///
/// `nested` means an outer call has already saved them, so only the
/// corrections are recorded.
fn check_lnums_both(do_curwin: bool, nested: bool) {
    let buf = curbuf.get();
    // SAFETY: `curbuf` is set from startup to exit.
    let line_count = unsafe { Buf::new(buf) }.line_count();
    for mut wp in tab_windows() {
        if (!do_curwin && wp.is_current()) || wp.w_buffer != buf {
            continue;
        }
        if !nested {
            wp.w_save_cursor.w_cursor_save = wp.w_cursor;
            wp.w_save_cursor.w_topline_save = wp.w_topline as c_int;
        }
        let mut need_adjust = wp.w_cursor.lnum > line_count;
        if need_adjust {
            wp.w_cursor.lnum = line_count;
        }
        if need_adjust || !nested {
            wp.w_save_cursor.w_cursor_corr = wp.w_cursor;
        }
        need_adjust = wp.w_topline > line_count;
        if need_adjust {
            wp.w_topline = line_count;
        }
        if need_adjust || !nested {
            wp.w_save_cursor.w_topline_corr = wp.w_topline as c_int;
        }
    }
}

pub fn check_lnums(do_curwin: bool) {
    check_lnums_both(do_curwin, false);
}

pub fn check_lnums_nested(do_curwin: bool) {
    check_lnums_both(do_curwin, true);
}

pub fn reset_lnums() {
    for mut wp in tab_windows() {
        if wp.w_buffer != curbuf.get() {
            continue;
        }
        // Restore the value if it was changed by `check_lnums` and has not been
        // changed since.
        if equalpos(wp.w_save_cursor.w_cursor_corr, wp.w_cursor)
            && wp.w_save_cursor.w_cursor_save.lnum != 0 as linenr_T
        {
            wp.w_cursor = wp.w_save_cursor.w_cursor_save;
        }
        if wp.w_save_cursor.w_topline_corr as linenr_T == wp.w_topline
            && wp.w_save_cursor.w_topline_save != 0
        {
            wp.w_topline = wp.w_save_cursor.w_topline_save as linenr_T;
        }
        if wp.w_save_cursor.w_topline_save as linenr_T > wp.buffer().line_count() {
            wp.w_valid.clear(WinValid::TOPLINE);
        }
    }
}

// ---------------------------------------------------------------------------
// The saved layout
//
// A snapshot is a frame tree of its own: the same shape, only the sizes and
// which leaf held `curwin`, and none of its frames is linked into the layout.

/// `tp->tp_snapshot[idx]`, borrowed as one slot.
fn snapshot_slot(tp: TabPage, idx: c_int) -> *mut *mut frame_T {
    let mut tp = tp;
    &raw mut tp.tp_snapshot[idx as usize]
}

/// The saved frame tree in slot `idx` of `tp`, if there is one.
fn snapshot_of(tp: TabPage, idx: c_int) -> Option<Frame> {
    // SAFETY: a saved tree is live until `drop_snapshot` frees it.
    unsafe { Frame::from_raw(tp.tp_snapshot[idx as usize]) }
}

pub fn make_snapshot(idx: c_int) {
    take_snapshot(idx);
}

/// Save the current layout in slot `idx` of the current tab page.
pub(crate) fn take_snapshot(idx: c_int) {
    let tp = cur_tab();
    drop_snapshot(tp, idx);
    make_snapshot_rec(current_topframe(), snapshot_slot(tp, idx));
}

/// Copy `fr` and everything hanging off it into a freshly allocated tree at
/// `slot`.
fn make_snapshot_rec(fr: Frame, slot: *mut *mut frame_T) {
    // SAFETY: `xcalloc` aborts rather than answering null; `slot` is a field of
    // a live tab page or of a frame this walk has just allocated.
    let mut copy = unsafe {
        let frp = xcalloc(1, size_of::<frame_T>()).cast::<frame_T>();
        *slot = frp;
        Frame::new(frp)
    };
    copy.fr_layout = fr.fr_layout;
    copy.fr_width = fr.fr_width;
    copy.fr_height = fr.fr_height;
    if let Some(next) = fr.next() {
        make_snapshot_rec(next, &raw mut copy.fr_next);
    }
    if let Some(child) = fr.child() {
        make_snapshot_rec(child, &raw mut copy.fr_child);
    }
    if fr.fr_layout as c_int == FR_LEAF && fr.win().is_some_and(Win::is_current) {
        copy.fr_win = cur_win().raw();
    }
}

/// Free the saved tree in slot `idx` of `tp`, if there is one.
pub(crate) fn drop_snapshot(tp: TabPage, idx: c_int) {
    let mut tp = tp;
    if let Some(fr) = snapshot_of(tp, idx) {
        clear_snapshot_rec(fr);
    }
    tp.tp_snapshot[idx as usize] = ptr::null_mut::<frame_T>();
}

/// Free `fr` and everything hanging off it.
fn clear_snapshot_rec(fr: Frame) {
    if let Some(next) = fr.next() {
        clear_snapshot_rec(next);
    }
    if let Some(child) = fr.child() {
        clear_snapshot_rec(child);
    }
    free(fr.raw());
}

/// The window a saved tree remembers as the current one: the last leaf that
/// named one, searching `fr_next` before `fr_child`.
fn snapshot_curwin_rec(ft: Frame) -> Option<Win> {
    if let Some(next) = ft.next()
        && let Some(wp) = snapshot_curwin_rec(next)
    {
        return Some(wp);
    }
    if let Some(child) = ft.child()
        && let Some(wp) = snapshot_curwin_rec(child)
    {
        return Some(wp);
    }
    // SAFETY: a saved leaf's `fr_win` is the still-live `curwin` of the moment
    // the snapshot was taken, or null.
    unsafe { Win::from_raw(ft.fr_win) }
}

/// The window the snapshot in slot `idx` of the current tab page remembers as
/// the current one, if there is one.
pub(crate) fn snapshot_curwin(idx: c_int) -> Option<Win> {
    snapshot_of(cur_tab(), idx).and_then(snapshot_curwin_rec)
}

pub fn restore_snapshot(idx: c_int, close_curwin: c_int) {
    restore_layout(idx, close_curwin != 0);
}

/// Put the layout saved in slot `idx` back, if it still fits the screen.
pub(crate) fn restore_layout(idx: c_int, close_curwin: bool) {
    let tp = cur_tab();
    let top = current_topframe();
    if let Some(sn) = snapshot_of(tp, idx)
        && sn.fr_width == top.fr_width
        && sn.fr_height == top.fr_height
        && snapshot_matches(sn, top)
    {
        let wp = restore_snapshot_rec(sn, top);
        comp_positions();
        if let Some(wp) = wp.filter(|_| close_curwin) {
            goto_win(wp);
        }
        redraw_all(UPD_NOT_VALID);
    }
    drop_snapshot(tp, idx);
}

/// Whether the saved tree `sn` still has the shape of the live tree `fr`, and
/// every window it remembers is still there.
fn snapshot_matches(sn: Frame, fr: Frame) -> bool {
    if sn.fr_layout != fr.fr_layout
        || sn.next().is_none() != fr.next().is_none()
        || sn.child().is_none() != fr.child().is_none()
    {
        return false;
    }
    if let (Some(sn_next), Some(fr_next)) = (sn.next(), fr.next())
        && !snapshot_matches(sn_next, fr_next)
    {
        return false;
    }
    if let (Some(sn_child), Some(fr_child)) = (sn.child(), fr.child())
        && !snapshot_matches(sn_child, fr_child)
    {
        return false;
    }
    // SAFETY: `win_valid` only compares the saved pointer against the list.
    !(!sn.fr_win.is_null() && !win_valid(sn.fr_win))
}

/// Give the live tree `fr` the sizes saved in `sn`, and answer the window `sn`
/// remembered as current.
fn restore_snapshot_rec(sn: Frame, fr: Frame) -> Option<Win> {
    let mut fr = fr;
    let mut wp = None;
    fr.fr_height = sn.fr_height;
    fr.fr_width = sn.fr_width;
    if fr.fr_layout as c_int == FR_LEAF {
        new_height(fr, fr.fr_height, false, false, false);
        new_width(fr, fr.fr_width, false, false);
        // SAFETY: as in [`snapshot_curwin_rec`].
        wp = unsafe { Win::from_raw(sn.fr_win) };
    }
    if let (Some(sn_next), Some(fr_next)) = (sn.next(), fr.next()) {
        wp = restore_snapshot_rec(sn_next, fr_next).or(wp);
    }
    if let (Some(sn_child), Some(fr_child)) = (sn.child(), fr.child()) {
        wp = restore_snapshot_rec(sn_child, fr_child).or(wp);
    }
    wp
}

// ---------------------------------------------------------------------------
// 'colorcolumn'

pub unsafe fn check_colorcolumn(cc: *mut c_char, wp: *mut win_T) -> *const c_char {
    // SAFETY: the caller's promise -- a live window or null, and a
    // NUL-terminated string or null.
    let win = unsafe { Win::from_raw(wp) };
    if win.is_some_and(|w| w.w_buffer.is_null()) {
        return ptr::null(); // buffer was closed
    }
    let mut s = match () {
        _ if !cc.is_null() => cc,
        _ => match win {
            Some(w) => w.w_onebuf_opt.wo_cc,
            None => empty_option(),
        },
    };
    let tw = win.map_or(0 as OptInt, |w| w.buffer().b_p_tw);

    let mut count = 0 as c_uint;
    let mut color_cols = [0 as c_int; 256];
    // Upstream's `color_cols` holds 256 entries and stops one short of them.
    while peek(s) != NUL && count < 255 as c_uint {
        let mut skip = false;
        let mut col;
        if peek(s) == '-' as c_int || peek(s) == '+' as c_int {
            // -N and +N: relative to 'textwidth'.
            col = if peek(s) == '-' as c_int { -1 } else { 1 };
            s = step(s);
            if !ascii_isdigit(peek(s)) {
                return &raw const e_invarg as *const c_char;
            }
            col *= digits(&mut s);
            if tw == 0 {
                skip = true; // 'textwidth' not set, skip this item
            } else {
                debug_assert!(
                    (col >= 0 && tw <= (c_int::MAX - col) as OptInt)
                        || (col < 0 && tw >= (c_int::MIN - col) as OptInt),
                    "col + tw fits in an int"
                );
                col += tw as c_int;
                skip = col < 0;
            }
        } else if ascii_isdigit(peek(s)) {
            col = digits(&mut s);
        } else {
            return &raw const e_invarg as *const c_char;
        }
        if !skip {
            color_cols[count as usize] = col - 1; // 1-based to 0-based
            count += 1;
        }
        if peek(s) == NUL {
            break;
        }
        if peek(s) != ',' as c_int {
            return &raw const e_invarg as *const c_char;
        }
        s = step(s);
        if peek(s) == NUL {
            return &raw const e_invarg as *const c_char; // illegal trailing comma
        }
    }

    let Some(mut win) = win else {
        return ptr::null(); // only parse the value, do not store it
    };
    free(win.w_p_cc_cols);
    if count == 0 {
        win.w_p_cc_cols = ptr::null_mut::<c_int>();
        return ptr::null();
    }
    let cols = &mut color_cols[..count as usize];
    arith::sort_columns(cols);
    // SAFETY: `xmalloc` aborts rather than answering null, so this is an array
    // of `count + 1` `int`s that nothing else has a reference to.
    let out = unsafe {
        let ptr = xmalloc(size_of::<c_int>() * (count as usize + 1)).cast::<c_int>();
        slice::from_raw_parts_mut(ptr, count as usize + 1)
    };
    win.w_p_cc_cols = out.as_mut_ptr();
    let mut j = 0;
    for col in cols.iter().copied() {
        // Skip duplicates.
        if j == 0 || out[j - 1] != col {
            out[j] = col;
            j += 1;
        }
    }
    out[j] = -1; // end marker
    ptr::null()
}

/// The byte `s` points at, as the C reads it.
fn peek(s: *const c_char) -> c_int {
    // SAFETY: `s` walks a NUL-terminated option string and stops at its NUL.
    (unsafe { *s }) as c_int
}

/// `s` advanced one byte, which the caller has checked is not the NUL.
fn step(s: *mut c_char) -> *mut c_char {
    // SAFETY: as [`peek`]; one past a non-NUL byte is still inside the string.
    unsafe { s.offset(1) }
}

/// The number `s` starts with, advancing it past the digits.
fn digits(s: &mut *mut c_char) -> c_int {
    // SAFETY: as [`peek`]; `getdigits_int` leaves `s` on the first non-digit.
    unsafe { getdigits_int(s, true, 0) }
}

// ---------------------------------------------------------------------------
// Odds and ends

pub fn get_last_winid() -> c_int {
    last_win_id.get()
}

pub unsafe fn win_locked(wp: *mut win_T) -> c_int {
    // SAFETY: the caller's promise -- a live window.
    unsafe { Win::new(wp) }.w_locked as c_int
}

pub unsafe fn win_get_tabwin(id: handle_T, tabnr: *mut c_int, winnr: *mut c_int) {
    let found = tab_and_win_number(id);
    // SAFETY: the caller's promise -- two writable `int`s.
    unsafe {
        let (tnum, wnum) = found.unwrap_or((0, 0));
        *tabnr = tnum;
        *winnr = wnum;
    }
}

/// The tab page and window number of the window with handle `id`, both 1-based
/// -- `None` when there is no such window, and `Some((0, 0))` when it is one
/// the `winnr` numbering skips.
fn tab_and_win_number(id: handle_T) -> Option<(c_int, c_int)> {
    for (tnum, tp) in (1..).zip(tabs()) {
        let mut wnum = 1;
        for wp in windows_in_tab(tp) {
            let numbered = wp.has_winnr(tp);
            if wp.handle == id {
                return Some(if numbered { (tnum, wnum) } else { (0, 0) });
            }
            wnum += numbered as c_int;
        }
    }
    None
}

pub unsafe fn win_ui_flush(validate: bool) {
    for tp in tabs() {
        for mut wp in windows_in_tab(tp) {
            let moved = wp.w_pos_changed || wp.w_grid_alloc.pending_comp_index_update;
            if moved && wp.w_grid_alloc.is_allocated() {
                if tp.is_current() {
                    // SAFETY: a live window.
                    unsafe { ui_ext_win_position(wp.raw(), validate) };
                } else {
                    // A window of another tab page is not on the screen.
                    ui_call_win_hide(wp.w_grid_alloc.handle as Integer);
                    wp.w_pos_changed = false;
                }
                wp.w_grid_alloc.pending_comp_index_update = false;
            }
            if tp.is_current() {
                // SAFETY: a live window.
                unsafe { ui_ext_win_viewport(wp.raw()) };
            }
        }
    }
    // SAFETY: flush the popup menu and message grids the same way.
    unsafe {
        pum_ui_flush();
        msg_ui_flush();
    }
}

pub unsafe fn lastwin_nofloating(tp: *mut tabpage_T) -> *mut win_T {
    // SAFETY: the caller's promise -- a live tab page or null.
    last_nonfloating(unsafe { TabPage::from_raw(tp) }).raw()
}

/// The last non-floating window of `tp`, or of the current tab page.
pub(crate) fn last_nonfloating(tp: Option<TabPage>) -> Win {
    debug_assert!(tp.is_none_or(|tp| !tp.is_current()), "tp != curtab || !tp");
    let mut res = match tp {
        Some(tp) => tp.tp_lastwin.and_then(WinId::get),
        None => last_window(),
    }
    .expect("a window list has a tail");
    while res.w_floating {
        res = res.prev().expect("a float is never the first window");
    }
    res
}
