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

#![forbid(unsafe_code)]

use core::ffi::{CStr, c_int};

use super::*;
use crate::charset::getdigits_int_at;
use crate::drawscreen::UPD_NOT_VALID;
use crate::message::e_invarg;
use crate::message::msg_ui_flush;
use crate::r#move::WinValid;
use crate::optionstr::OptStringRef;
use crate::popupmenu::pum_ui_flush;
use crate::pos::equalpos;
use crate::types::{Handle, Integer, LineNr, OptError, OptInt};
use crate::ui::ui_call_win_hide;
use crate::winlayer::{
    Buf, FrameId, FrameRef, TabPage, Win, WinId, last_window, tab_windows, tabs, windows_in_tab,
};

// ---------------------------------------------------------------------------
// Revalidating cursors against a buffer that changed length

/// Clamp every window's cursor and topline into `curbuf`, remembering where
/// they were so [`reset_lnums`] can put them back.
///
/// `nested` means an outer call has already saved them, so only the
/// corrections are recorded.
fn check_lnums_both(do_curwin: bool, nested: bool) {
    let buf = Buf::current();
    let line_count = buf.line_count();
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
        if !wp.w_buffer.is_current() {
            continue;
        }
        // Restore the value if it was changed by `check_lnums` and has not been
        // changed since.
        if equalpos(wp.w_save_cursor.w_cursor_corr, wp.w_cursor)
            && wp.w_save_cursor.w_cursor_save.lnum != 0 as LineNr
        {
            wp.w_cursor = wp.w_save_cursor.w_cursor_save;
        }
        if wp.w_save_cursor.w_topline_corr as LineNr == wp.w_topline
            && wp.w_save_cursor.w_topline_save != 0
        {
            wp.w_topline = wp.w_save_cursor.w_topline_save as LineNr;
        }
        if wp.w_save_cursor.w_topline_save as LineNr > wp.buffer().line_count() {
            wp.w_valid.clear(WinValid::TOPLINE);
        }
    }
}

// ---------------------------------------------------------------------------
// The saved layout
//
// A snapshot is a frame tree of its own: the same shape, only the sizes and
// which leaf held `curwin`, and none of its frames is linked into the layout.

/// The saved frame tree in slot `idx` of `tabpage`, if there is one.
fn snapshot_of(tabpage: TabPage, idx: c_int) -> Option<FrameRef> {
    tabpage.tp_snapshot[idx as usize].and_then(FrameId::get)
}

pub fn make_snapshot(idx: c_int) {
    take_snapshot(idx);
}

/// Save the current layout in slot `idx` of the current tab page.
pub(crate) fn take_snapshot(idx: c_int) {
    let mut tp = TabPage::current();
    drop_snapshot(tp, idx);
    tp.tp_snapshot[idx as usize] = Some(make_snapshot_rec(current_topframe()));
}

/// Copy `fr` and everything hanging off it into a freshly allocated tree,
/// and answer that tree's root.
///
/// The copies are registered frames like any other, so a snapshot's links
/// are the same identities the live tree's are -- which is what lets
/// [`snapshot_matches`] walk one that a `:diffsplit` later freed.
fn make_snapshot_rec(fr: FrameRef) -> FrameId {
    let mut copy = new_frame();
    copy.fr_layout = fr.fr_layout;
    copy.fr_width = fr.fr_width;
    copy.fr_height = fr.fr_height;
    copy.fr_next = fr.next().map(make_snapshot_rec);
    copy.fr_child = fr.child().map(make_snapshot_rec);
    if fr.fr_layout as c_int == FR_LEAF && fr.win().is_some_and(Win::is_current) {
        copy.fr_win = Some(Win::current().id());
    }
    copy.id()
}

/// Free the saved tree in slot `idx` of `tabpage`, if there is one.
pub(crate) fn drop_snapshot(tabpage: TabPage, idx: c_int) {
    let mut tabpage = tabpage;
    if let Some(fr) = snapshot_of(tabpage, idx) {
        clear_snapshot_rec(fr);
    }
    tabpage.tp_snapshot[idx as usize] = None;
}

/// Free `fr` and everything hanging off it.
fn clear_snapshot_rec(fr: FrameRef) {
    if let Some(next) = fr.next() {
        clear_snapshot_rec(next);
    }
    if let Some(child) = fr.child() {
        clear_snapshot_rec(child);
    }
    free_frame(fr);
}

/// The window a saved leaf remembers, if it is **still on this tab page's
/// window list**.
///
/// The C's `win_valid(sn->fr_win)`, and deliberately not `FrameRef::win`: a
/// snapshot's leaf names a window that may since have been closed *or moved
/// to another tab page*, and only the list walk refuses the second. The live
/// tree's leaves cannot be either, which is why they use the plain accessor.
fn snapshot_win(sn: FrameRef) -> Option<Win> {
    sn.fr_win.filter(|&win| win_valid(win)).and_then(WinId::get)
}

/// The window a saved tree remembers as the current one: the last leaf that
/// named one, searching `fr_next` before `fr_child`.
fn snapshot_curwin_rec(ft: FrameRef) -> Option<Win> {
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
    snapshot_win(ft)
}

/// The window the snapshot in slot `idx` of the current tab page remembers as
/// the current one, if there is one.
pub(crate) fn snapshot_curwin(idx: c_int) -> Option<Win> {
    snapshot_of(TabPage::current(), idx).and_then(snapshot_curwin_rec)
}

pub fn restore_snapshot(idx: c_int, close_curwin: c_int) {
    restore_layout(idx, close_curwin != 0);
}

/// Put the layout saved in slot `idx` back, if it still fits the screen.
pub(crate) fn restore_layout(idx: c_int, close_curwin: bool) {
    let tp = TabPage::current();
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
fn snapshot_matches(sn: FrameRef, fr: FrameRef) -> bool {
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
    sn.fr_win.is_none() || snapshot_win(sn).is_some()
}

/// Give the live tree `fr` the sizes saved in `sn`, and answer the window `sn`
/// remembered as current.
fn restore_snapshot_rec(sn: FrameRef, fr: FrameRef) -> Option<Win> {
    let mut fr = fr;
    let mut wp = None;
    fr.fr_height = sn.fr_height;
    fr.fr_width = sn.fr_width;
    if fr.fr_layout as c_int == FR_LEAF {
        new_height(fr, fr.fr_height, false, false, false);
        new_width(fr, fr.fr_width, false, false);
        wp = snapshot_win(sn);
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

/// [`check_colorcolumn`] over the window's own 'colorcolumn'.
pub(crate) fn check_window_colorcolumn(window: Win) -> Result<(), OptError> {
    check_colorcolumn(None, Some(window))
}

/// Parse 'colorcolumn' -- `cc`, or the window's own value when there is
/// none -- and, given a window, store the columns it names.
///
/// The value is parsed whole before the window is written.
pub fn check_colorcolumn(cc: Option<&CStr>, window: Option<Win>) -> Result<(), OptError> {
    if window.is_some_and(|w| w.w_buffer.is_null()) {
        return Ok(()); // buffer was closed
    }
    let text: &[u8] = match (cc, &window) {
        (Some(cc), _) => cc.to_bytes(),
        (None, Some(w)) => w.w_onebuf_opt.wo_cc.bytes(),
        (None, None) => b"",
    };
    let tw = window.map_or(0 as OptInt, |w| w.buffer().b_p_tw);
    let cols = parse_colorcolumn(text, tw)?;

    let Some(mut win) = window else {
        return Ok(()); // only parse the value, do not store it
    };
    win.w_p_cc_cols = cols;
    Ok(())
}

/// The columns a 'colorcolumn' value names, 0-based, sorted, without
/// duplicates and ended by `-1` -- or empty when it names none. `tw` is the
/// 'textwidth' the `-N`/`+N` items are relative to.
fn parse_colorcolumn(text: &[u8], tw: OptInt) -> Result<Vec<c_int>, OptError> {
    let at = |i: usize| text.get(i).copied().unwrap_or(0);
    let mut i = 0;
    let mut cols: Vec<c_int> = Vec::new();
    // Upstream's `color_cols` holds 256 entries and stops one short of them.
    while at(i) != 0 && cols.len() < 255 {
        let mut skip = false;
        let mut col;
        if at(i) == b'-' || at(i) == b'+' {
            // -N and +N: relative to 'textwidth'.
            col = if at(i) == b'-' { -1 } else { 1 };
            i += 1;
            if !at(i).is_ascii_digit() {
                return Err((e_invarg).into());
            }
            let (n, past) = getdigits_int_at(text, i, true, 0);
            (col, i) = (col * n, past);
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
        } else if at(i).is_ascii_digit() {
            (col, i) = getdigits_int_at(text, i, true, 0);
        } else {
            return Err((e_invarg).into());
        }
        if !skip {
            cols.push(col - 1); // 1-based to 0-based
        }
        if at(i) == 0 {
            break;
        }
        if at(i) != b',' {
            return Err((e_invarg).into());
        }
        i += 1;
        if at(i) == 0 {
            return Err((e_invarg).into()); // illegal trailing comma
        }
    }

    if cols.is_empty() {
        return Ok(cols);
    }
    arith::sort_columns(&mut cols);
    cols.dedup();
    cols.push(-1); // end marker
    Ok(cols)
}

// ---------------------------------------------------------------------------
// Odds and ends

pub fn get_last_winid() -> c_int {
    last_win_id.get()
}

pub fn win_locked(window: Win) -> c_int {
    window.w_locked as c_int
}

/// The tab page and window number of the window with handle `id`, both
/// 1-based; `(0, 0)` when there is no such window or it is one the `winnr`
/// numbering skips.
pub fn win_get_tabwin(id: Handle) -> (c_int, c_int) {
    tab_and_win_number(id).unwrap_or((0, 0))
}

/// The tab page and window number of the window with handle `id`, both 1-based
/// -- `None` when there is no such window, and `Some((0, 0))` when it is one
/// the `winnr` numbering skips.
fn tab_and_win_number(id: Handle) -> Option<(c_int, c_int)> {
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

pub fn win_ui_flush(validate: bool) {
    for tp in tabs() {
        for mut wp in windows_in_tab(tp) {
            let moved = wp.w_pos_changed || wp.w_grid_alloc.pending_comp_index_update;
            if moved && wp.w_grid_alloc.is_allocated() {
                if tp.is_current() {
                    ui_ext_win_position(wp, validate);
                } else {
                    // A window of another tab page is not on the screen.
                    ui_call_win_hide(wp.w_grid_alloc.handle as Integer);
                    wp.w_pos_changed = false;
                }
                wp.w_grid_alloc.pending_comp_index_update = false;
            }
            if tp.is_current() {
                ui_ext_win_viewport(wp);
            }
        }
    }
    pum_ui_flush();
    msg_ui_flush();
}

/// The last non-floating window of `tabpage`, or of the current tab page.
pub(crate) fn lastwin_nofloating(tabpage: Option<TabPage>) -> Win {
    debug_assert!(
        tabpage.is_none_or(|tp| !tp.is_current()),
        "tp != curtab || !tp"
    );
    let mut res = match tabpage {
        Some(tp) => tp.tp_lastwin.and_then(WinId::get),
        None => last_window(),
    }
    .expect("a window list has a tail");
    while res.w_floating {
        res = res.prev().expect("a float is never the first window");
    }
    res
}
