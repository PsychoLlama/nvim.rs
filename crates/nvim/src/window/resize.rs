//! Applying a new size to a window, and the rows that are not text.
//!
//! [`set_inner_size`] is where a height or width finally lands on a `Window`: it
//! resizes the grid, re-wraps the text, and keeps the view stable by
//! remembering the cursor's [`save_fraction`] of the window and restoring it
//! with [`to_fraction`] ([`fix_scroll`] and [`fix_cursor`] are the
//! `'splitkeep'` half of the same problem).  The rest is the non-text
//! bookkeeping: [`command_height`] for `'cmdheight'`, [`last_status`] and
//! [`last_status_rec`] for `'laststatus'`, [`set_winbar`] for `'winbar'`,
//! [`tabline_rows`] for `'showtabline'`, and [`min_rows_of`], which says how
//! few rows the layout can survive in.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;
use core::ptr;

use super::arith::NextCurwin;
use super::*;
use crate::buffer::{buf_is_help, current_buf};
use crate::decoration::decor_conceal_line;
use crate::drawscreen::state::{cmdline_row, redraw_cmdline};
use crate::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, comp_col};
use crate::edit::{cursor_down_inner, cursor_up_inner};
use crate::grid::{default_gridview, grid_clear};
use crate::mark::setmark;
use crate::message::e_noroom;
use crate::message::state::{msg_row, msg_scrolled};
use crate::message::{msg_grid_validate, msg_grid_view};
use crate::r#move::{
    WinValid, changed_line_abv_curs_win, curs_columns, invalidate_botline_win, set_topline,
    validate_botline_win, win_col_off2,
};
use crate::option::get_scrolloff_value;
use crate::option::vars::{p_ch, p_ls, p_spk, p_stal, p_wbr};
use crate::options::kWinOptScroll;
use crate::plines::{plines_win, plines_win_col, plines_win_nofill};
use crate::startup::{exiting, full_screen};
use crate::state::{MODE_CMDLINE, MODE_NORMAL, MODE_TERMINAL, get_real_state};
use crate::statusline::stl_clear_click_defs;
use crate::terminal::terminal_check_size;
use crate::types::ui::{kUIMessages, kUIMultigrid, kUITabline};
use crate::types::{
    ColNr, FAIL, Integer, LineNr, NUL, OK, OptInt, ScriptId, StlClickDefinition, WindowHandle,
    size_t,
};
use crate::ui::state::{Columns, Rows};
use crate::ui::{ui_call_win_viewport_margins, ui_has};
use crate::window::state::{skip_update_topline, skip_win_fix_cursor};
use crate::winfloat::{win_border_height, win_border_width, win_float_anchor_laststatus};
use crate::winlayer::{FrameRef, TabPage, Win, tabs, windows};

// ---------------------------------------------------------------------------
// The neighbours only this file reaches

/// How many screen lines line `lnum` takes in `window`.
fn plines(window: Win, lnum: LineNr, limit_winheight: bool) -> c_int {
    // SAFETY: a line of the window's own buffer.
    unsafe { plines_win(window, lnum, limit_winheight) }
}

/// [`plines`] up to and including column `col` of the line.
fn plines_to_col(window: Win, lnum: LineNr, col: ::core::ffi::c_long) -> c_int {
    // SAFETY: a position in the window's own buffer.
    unsafe { plines_win_col(window, lnum, col) }
}

/// [`plines`] without the virtual lines a diff fills the window with.
fn plines_nofill(window: Win, lnum: LineNr, limit_winheight: bool) -> c_int {
    // SAFETY: a line of the window's own buffer.
    unsafe { plines_win_nofill(window, lnum, limit_winheight) }
}

/// Columns of `window` the text does not start in: `'number'`, signs and folds.
fn col_off(window: Win) -> c_int {
    // SAFETY: a live window.
    window.col_off()
}

/// The extra indent a wrapped line gets from `'cpoptions'`'s `n` flag.
fn col_off2(window: Win) -> c_int {
    // SAFETY: a live window.
    win_col_off2(window)
}

/// Forget everything cached below the window's last drawn line.
fn invalidate_botline(window: Win) {
    // SAFETY: a live window.
    invalidate_botline_win(window);
}

/// Recompute the window's last drawn line.
fn validate_botline(window: Win) {
    // SAFETY: a live window.
    validate_botline_win(window);
}

/// Move the cursor `n` lines down (or up, for [`cursor_up`]) without touching
/// the view, as `'splitkeep'` needs to measure it.
fn cursor_down(window: Win, n: c_int) {
    // SAFETY: a live window.
    cursor_down_inner(window, n, false);
}

fn cursor_up(window: Win, n: LineNr) {
    // SAFETY: a live window.
    cursor_up_inner(window, n, false);
}

/// The effective `'scrolloff'` for `window`.
fn scrolloff(window: Win) -> ::core::ffi::c_long {
    // SAFETY: a live window.
    get_scrolloff_value(window) as ::core::ffi::c_long
}

/// Free a window's status line or window bar click definitions.
pub(crate) fn free_click_defs(defs: *mut StlClickDefinition, size: size_t) {
    // SAFETY: the window's own array and its length.
    unsafe { stl_clear_click_defs(defs, size) };
    free(defs);
}

// ---------------------------------------------------------------------------
// The cursor's place in the window

pub fn set_fraction(window: Win) {
    save_fraction(window);
}

/// Remember where the cursor is as a fraction of the window's height, so a
/// resize can put it back in the same relative place.
pub(crate) fn save_fraction(window: Win) {
    let mut window = window;
    if window.w_view_height > 1 {
        window.w_fraction = arith::cursor_fraction(window.w_wrow, window.w_view_height);
    }
}

pub fn win_fix_scroll(resize: bool) {
    fix_scroll(resize);
}

/// Keep every window's scroll position across a layout change, the way
/// `'splitkeep'` asks for -- "screen" holds the text still on the screen,
/// "topline" holds the top line, and "cursor" (the default) does nothing here.
pub(crate) fn fix_scroll(resize: bool) {
    // SAFETY: `'splitkeep'` is a NUL-terminated option string.
    if unsafe { *p_spk.get() } as c_int == 'c' as c_int {
        return;
    }
    skip_update_topline.set(true);
    for mut wp in windows() {
        if !wp.w_floating && wp.w_height != wp.w_prev_height {
            wp.w_do_win_fix_cursor = true;
            // SAFETY: as above.
            let screen = unsafe { *p_spk.get() } as c_int == 's' as c_int;
            if screen
                && wp.w_winrow != wp.w_prev_winrow
                && wp.w_botline - 1 <= wp.buffer().line_count()
            {
                // Scroll `wp` so that the same text stays on the screen.
                let diff = wp.w_winrow - wp.w_prev_winrow + (wp.w_height - wp.w_prev_height);
                let cursor = wp.w_cursor;
                wp.w_cursor.lnum = wp.w_botline - 1;
                if diff > 0 {
                    cursor_down(wp, diff);
                } else {
                    cursor_up(wp, -diff);
                }
                wp.w_fraction = FRACTION_MULT;
                to_fraction(wp, wp.w_prev_height);
                wp.w_cursor = cursor;
                wp.w_valid.clear(WinValid::WCOL);
            } else if wp.is_current() {
                wp.w_valid.clear(WinValid::CROW);
            }
            invalidate_botline(wp);
            validate_botline(wp);
        }
        wp.w_prev_height = wp.w_height;
        wp.w_prev_winrow = wp.w_winrow;
    }
    skip_update_topline.set(false);
    let state = get_real_state();
    if state & (MODE_NORMAL | MODE_CMDLINE | MODE_TERMINAL) == 0 {
        fix_cursor(false);
    } else if resize {
        fix_cursor(true);
    }
}

/// Move the cursor into the visible part of the window when a resize left it
/// outside `'scrolloff'`.
pub(crate) fn fix_cursor(normal: bool) {
    let mut wp = Win::current();
    if skip_win_fix_cursor.get()
        || !wp.w_do_win_fix_cursor
        || wp.buffer().line_count() < wp.w_view_height as LineNr
    {
        return;
    }
    wp.w_do_win_fix_cursor = false;
    // Determine the first and last line of the window, `'scrolloff'` included.
    let so = scrolloff(wp).min((wp.w_view_height / 2) as ::core::ffi::c_long) as c_int;
    let lnum = wp.w_cursor.lnum;
    wp.w_cursor.lnum = wp.w_topline;
    cursor_down(wp, so);
    let top = wp.w_cursor.lnum;
    wp.w_cursor.lnum = wp.w_botline - 1;
    cursor_up(wp, so as LineNr);
    let bot = wp.w_cursor.lnum;
    wp.w_cursor.lnum = lnum;

    let mut nlnum = 0 as LineNr;
    if lnum > bot && wp.w_botline - wp.buffer().line_count() != 1 {
        nlnum = bot;
    } else if lnum < top && wp.w_topline != 1 {
        nlnum = if so == wp.w_view_height / 2 { bot } else { top };
    }
    if nlnum != 0 {
        if normal {
            // Save the position for the `''` mark.
            // SAFETY: sets a mark at the cursor of the current window.
            let _ = unsafe { setmark('\'' as c_int) };
            wp.w_cursor.lnum = nlnum;
        } else {
            wp.w_fraction = if nlnum == bot { FRACTION_MULT } else { 0 };
            to_fraction(wp, wp.w_prev_height);
            validate_botline(Win::current());
        }
    }
}

pub fn win_new_height(window: Win, height: c_int) {
    new_win_height(window, height);
}

/// Give window `window` height `height`.
pub(crate) fn new_win_height(window: Win, height: c_int) {
    let mut window = window;
    // Don't want a negative height: happens when splitting a tiny window, and
    // is equalized away soon after.
    let height = height.max(0);
    if window.w_height == height {
        return;
    }
    window.w_height = height;
    window.w_pos_changed = true;
    set_inner_size(window, true);
}

pub fn scroll_to_fraction(window: Win, prev_height: c_int) {
    to_fraction(window, prev_height);
}

/// Put the cursor back at the [`save_fraction`] of the window it was at before
/// the resize, scrolling the view to suit.
pub(crate) fn to_fraction(window: Win, prev_height: c_int) {
    let mut window = window;
    let height = window.w_view_height;
    // Don't change `w_topline` when the window has no height, when
    // `'scrollbind'` is set on a window that is not current, or when the whole
    // buffer fits and its first line is visible.
    if height > 0
        && (window.w_onebuf_opt.wo_scb == 0 || window.is_current())
        && ((height as LineNr) < window.buffer().line_count() || window.w_topline > 1)
    {
        // Find a `w_topline` that shows the cursor at the same relative
        // position in the window as before (more or less).
        let mut lnum = window.w_cursor.lnum.max(1); // can be 0 during startup
        window.w_wrow = arith::fraction_row(window.w_fraction, height);
        let mut line_size =
            plines_to_col(window, lnum, window.w_cursor.col as ::core::ffi::c_long) - 1;
        let mut sline = window.w_wrow - line_size;

        if sline >= 0 {
            // Make sure the whole cursor line is visible, if possible.
            let rows = plines(window, lnum, false);
            if sline > window.w_view_height - rows {
                sline = window.w_view_height - rows;
                window.w_wrow -= rows - line_size;
            }
        }
        if sline < 0 {
            // The cursor line would go off the top of the screen: make it the
            // first line in the window, and use `w_skipcol` when it does not
            // fit whole.
            window.w_wrow = line_size;
            if window.w_wrow >= window.w_view_height && window.w_view_width - col_off(window) > 0 {
                window.w_skipcol += window.w_view_width - col_off(window);
                window.w_wrow -= 1;
                while window.w_wrow >= window.w_view_height {
                    window.w_skipcol += window.w_view_width - col_off(window) + col_off2(window);
                    window.w_wrow -= 1;
                }
            }
        } else if sline > 0 {
            while sline > 0 && lnum > 1 {
                if let Some(first) = window.fold_first(lnum) {
                    lnum = first;
                }
                if lnum == 1 {
                    // The first line in the buffer is folded.
                    line_size = !decor_conceal_line(window, 0, false) as c_int;
                    sline -= 1;
                    break;
                }
                lnum -= 1;
                line_size = if lnum == window.w_topline {
                    plines_nofill(window, lnum, true) + window.w_topfill
                } else {
                    plines(window, lnum, true)
                };
                sline -= line_size;
            }
            if sline < 0 {
                // The line we want at the top would go off the top of the
                // screen: use the next one instead.
                lnum = window.fold_last(lnum) + 1;
                window.w_wrow -= line_size + sline;
            } else if sline > 0 {
                // The first line of the file was reached: use that as topline.
                lnum = 1;
                window.w_wrow -= sline;
            }
        }
        // SAFETY: a live window and a line of its buffer.
        set_topline(window, lnum);
    }

    if window.is_current() {
        // SAFETY: a live window; validates `w_wrow`.
        curs_columns(window, 0);
    }
    if prev_height > 0 {
        window.w_prev_fraction_row = window.w_wrow;
    }
    window.redraw_later(UPD_SOME_VALID);
    invalidate_botline(window);
}

pub fn win_set_inner_size(window: Win, valid_cursor: bool) {
    set_inner_size(window, valid_cursor);
}

/// Give the window's *text area* the size its frame now implies, and tell the
/// UI, the terminal and the view about it.
pub(crate) fn set_inner_size(window: Win, valid_cursor: bool) {
    let mut window = window;
    let mut width = window.w_width_request;
    if width == 0 {
        width = window.w_width;
    }
    let prev_height = window.w_view_height;
    let mut height = window.w_height_request;
    if height == 0 {
        height = (window.w_height - window.w_winbar_height).max(0);
    }
    // SAFETY: `'splitkeep'` is a NUL-terminated option string.
    let keeps_cursor = unsafe { *p_spk.get() } as c_int == 'c' as c_int;

    if height != prev_height {
        if height > 0 && valid_cursor {
            if window.is_current() && (keeps_cursor || window.w_floating) {
                Win::current().validate_cursor();
            }
            if window.w_view_height != prev_height {
                // Recursive call: the cursor validation resized this window.
                return;
            }
            if window.w_wrow != window.w_prev_fraction_row {
                save_fraction(window);
            }
        }
        window.w_view_height = height;
        comp_scroll(window);
        if valid_cursor && !exiting.get() && (keeps_cursor || window.w_floating) {
            window.w_skipcol = 0 as ColNr;
            to_fraction(window, prev_height);
        }
        window.redraw_later(UPD_SOME_VALID);
    }
    if width != window.w_view_width {
        window.w_view_width = width;
        window.w_lines_valid = 0;
        if valid_cursor {
            // SAFETY: a live window.
            changed_line_abv_curs_win(window);
            invalidate_botline(window);
            if window.is_current() && (keeps_cursor || window.w_floating) {
                // SAFETY: a live window.
                curs_columns(window, 1);
            }
        }
        window.redraw_later(UPD_NOT_VALID);
    }
    if !window.buffer().terminal.is_null() {
        // SAFETY: the buffer's own terminal.
        unsafe { terminal_check_size(window.buffer().terminal) };
    }

    // SAFETY: a live window; both read its border configuration.
    let (border_height, border_width) = (win_border_height(window), win_border_width(window));
    let float_stl = if window.w_floating && window.w_status_height != 0 {
        STATUS_HEIGHT as c_int
    } else {
        0
    };
    window.w_height_outer =
        window.w_view_height + border_height + window.w_winbar_height + float_stl;
    window.w_width_outer = window.w_view_width + border_width;
    window.w_winrow_off = window.w_border_adj[0] + window.w_winbar_height;
    window.w_wincol_off = window.w_border_adj[3];
    if ui_has(kUIMultigrid) {
        {
            ui_call_win_viewport_margins(
                window.w_grid_alloc.handle as Integer,
                window.handle as WindowHandle,
                window.w_winrow_off as Integer,
                window.w_border_adj[2] as Integer,
                window.w_wincol_off as Integer,
                window.w_border_adj[1] as Integer,
            );
        }
    }
    window.w_redr_status = true;
}

pub fn win_new_width(window: Win, width: c_int) {
    new_win_width(window, width);
}

/// Give window `window` width `width`.
pub(crate) fn new_win_width(window: Win, width: c_int) {
    let mut window = window;
    window.w_width = width.max(0);
    window.w_pos_changed = true;
    set_inner_size(window, true);
}

pub fn win_default_scroll(window: Win) -> OptInt {
    default_scroll(window)
}

/// The `'scroll'` a window gets when the option is not set by hand: half its
/// height, and never less than one.
pub(crate) fn default_scroll(window: Win) -> OptInt {
    (window.w_view_height / 2).max(1) as OptInt
}

pub fn win_comp_scroll(window: Win) {
    comp_scroll(window);
}

/// Recompute `'scroll'` after a resize, marking it as set by the layout rather
/// than by the user.
pub(crate) fn comp_scroll(window: Win) {
    let mut window = window;
    let old = window.w_onebuf_opt.wo_scr;
    window.w_onebuf_opt.wo_scr = default_scroll(window);
    if window.w_onebuf_opt.wo_scr != old {
        let ctx = &mut window.w_onebuf_opt.wo_script_ctx[kWinOptScroll as usize];
        ctx.sc_sid = SID_WINLAYOUT as ScriptId;
        ctx.sc_lnum = 0 as LineNr;
    }
}

// ---------------------------------------------------------------------------
// The rows that are not text

pub unsafe fn command_height() {
    let mut old_p_ch = TabPage::current().tp_ch_used as c_int;
    // Find the last frame that spans the whole width and is not pinned by
    // 'winfixheight', which is the one the command line trades rows with.
    let mut frp = Some(lastwin_nofloating(None).frame());
    while let Some(fr) = frp.filter(|fr| fr.fr_width != Columns.get()) {
        frp = fr.parent();
        if frp.is_none() {
            frp = Some(fr);
            break;
        }
    }
    while let Some(fr) = frp.filter(|fr| {
        fr.prev().is_some()
            && fr.fr_layout as c_int == FR_LEAF
            && fr.win().is_some_and(|w| w.w_onebuf_opt.wo_wfh != 0)
    }) {
        frp = fr.prev();
    }

    while p_ch.get() > old_p_ch as OptInt && command_frame_height.get() {
        let Some(fr) = frp else {
            err(e_noroom.as_ptr());
            p_ch.set(old_p_ch as OptInt);
            break;
        };
        let spare = fr.fr_height - minheight(fr, NextCurwin::Unset);
        let h = ((p_ch.get() - old_p_ch as OptInt) as c_int).min(spare);
        add_height(fr, -h);
        old_p_ch += h;
        frp = fr.prev();
    }
    if p_ch.get() < old_p_ch as OptInt
        && command_frame_height.get()
        && let Some(fr) = frp
    {
        add_height(fr, (old_p_ch as OptInt - p_ch.get()) as c_int);
    }

    comp_positions();
    cmdline_row.set(Rows.get() - p_ch.get() as c_int);
    redraw_cmdline.set(true);
    if msg_scrolled.get() == 0 && full_screen.get() {
        let mut grid = default_gridview();
        if !ui_has(kUIMessages) {
            // SAFETY: makes sure the message grid exists before it is cleared.
            unsafe { msg_grid_validate() };
            grid = msg_grid_view();
        }
        // SAFETY: a live grid, and a row range inside the screen.
        unsafe { grid_clear(grid, cmdline_row.get(), Rows.get(), 0, Columns.get(), 0) };
        msg_row.set(cmdline_row.get());
    }
    TabPage::current().tp_ch_used = p_ch.get();
    min_set_ch.set(p_ch.get());
}

/// Add `n` rows to frame `frp` and to every frame above it, from
/// `frame_add_height()`. Negative `n` takes them away.
fn add_height(frp: FrameRef, n: c_int) {
    new_height(frp, frp.fr_height + n, false, false, false);
    let mut up = frp.parent();
    while let Some(mut fr) = up {
        fr.fr_height += n;
        up = fr.parent();
    }
}

pub fn last_status(morewin: bool) {
    update_last_status(morewin);
}

/// Add or remove the last window's status line, whichever `'laststatus'` and
/// the number of windows now call for.
pub(crate) fn update_last_status(morewin: bool) {
    // If the window has a status line and it is not needed, or the other way
    // round, add or remove one.
    last_status_rec(
        current_topframe(),
        last_stl_rows(morewin) > 0,
        global_stl_rows() > 0,
    );
    win_float_anchor_laststatus();
}

pub fn win_remove_status_line(window: Win, add_hsep: bool) {
    remove_status_line(window, add_hsep);
}

/// Take `window`'s status line away, giving its row either to a horizontal
/// separator or back to the window's text.
pub(crate) fn remove_status_line(window: Win, add_hsep: bool) {
    let mut window = window;
    window.w_status_height = 0;
    if add_hsep {
        window.w_hsep_height = 1;
    } else {
        let text = if window.w_floating {
            window.w_view_height
        } else {
            window.w_height
        };
        new_win_height(window, text + STATUS_HEIGHT as c_int);
    }
    // SAFETY: recomputes the column the message area starts in.
    unsafe { comp_col() };
    free_click_defs(window.w_status_click_defs, window.w_status_click_defs_size);
    window.w_status_click_defs_size = 0 as size_t;
    window.w_status_click_defs = ptr::null_mut::<StlClickDefinition>();
}

/// The nearest frame at or above `fr` that has a row to spare, from
/// `find_horizontally_resizable_frame()`. `None` when the layout is full.
fn resizable_frame(fr: FrameRef) -> Option<FrameRef> {
    let mut fp = fr;
    let top = current_topframe();
    while fp.fr_height <= minheight(fp, NextCurwin::Unset) {
        if fp == top {
            return None;
        }
        let parent = fp.parent().expect("not the top frame");
        fp = match fp.prev() {
            Some(prev) if parent.fr_layout as c_int == FR_COL => prev,
            _ => parent,
        };
    }
    Some(fp)
}

/// Make room for the status line `fr`'s window has just been given.
fn resize_frame_for_status(fr: FrameRef) -> bool {
    let wp = fr.win().expect("a leaf frame holds a window");
    let Some(fp) = resizable_frame(fr) else {
        err(e_noroom.as_ptr());
        return false;
    };
    if fp != fr {
        new_height(fp, fp.fr_height - 1, false, false, false);
        frame_fix_height(wp);
        comp_positions();
    } else {
        new_win_height(wp, wp.w_height - 1);
    }
    true
}

/// Make room for the window bar `fr`'s window has just been given, which --
/// unlike a status line -- cannot come out of the window's own text.
fn resize_frame_for_winbar(fr: FrameRef) -> bool {
    let wp = fr.win().expect("a leaf frame holds a window");
    let Some(fp) = resizable_frame(fr).filter(|fp| *fp != fr) else {
        err(e_noroom.as_ptr());
        return false;
    };
    new_height(fp, fp.fr_height - 1, false, false, false);
    new_win_height(wp, wp.w_height + 1);
    frame_fix_height(wp);
    comp_positions();
    true
}

/// Add or remove the status lines `'laststatus'` asks for, over every window in
/// frame `fr`.
fn last_status_rec(fr: FrameRef, statusline: bool, is_stl_global: bool) {
    let Some(mut wp) = fr.win() else {
        for fp in fr.children() {
            last_status_rec(fp, statusline, is_stl_global);
        }
        return;
    };
    if is_bottom_window(wp) {
        if wp.w_status_height != 0 && (!statusline || is_stl_global) {
            remove_status_line(wp, false);
        } else if wp.w_status_height == 0 && !is_stl_global && statusline {
            // Add a status line, taking a row from wherever there is one.
            wp.w_status_height = STATUS_HEIGHT as c_int;
            if !resize_frame_for_status(fr) {
                return;
            }
            // SAFETY: recomputes the column the message area starts in.
            unsafe { comp_col() };
        }
        if (wp.w_height - wp.w_prev_height).abs() == 1 {
            wp.w_prev_height = wp.w_height;
        }
    } else if wp.w_status_height != 0 && is_stl_global {
        remove_status_line(wp, true);
    } else if wp.w_status_height == 0 && !is_stl_global {
        wp.w_status_height = STATUS_HEIGHT as c_int;
        wp.w_hsep_height = 0;
        // SAFETY: recomputes the column the message area starts in.
        unsafe { comp_col() };
    }
}

pub fn set_winbar_win(window: Win, make_room: bool, valid_cursor: bool) -> c_int {
    winbar_win(window, make_room, valid_cursor)
}

/// Give `window` the window bar `'winbar'` asks for, or take it away.
fn winbar_win(window: Win, make_room: bool, valid_cursor: bool) -> c_int {
    let mut window = window;
    // SAFETY: both are NUL-terminated option strings.
    let (global, local) = unsafe { (*p_wbr.get() as c_int, *window.w_onebuf_opt.wo_wbr as c_int) };
    let winbar_height = if window.w_floating {
        (local != NUL) as c_int
    } else {
        (global != NUL || local != NUL) as c_int
    };
    if window.w_winbar_height != winbar_height {
        if winbar_height == 1 && window.w_view_height <= 1 {
            if window.w_floating {
                err(e_noroom.as_ptr());
                return NOTDONE;
            } else if !make_room || !resize_frame_for_winbar(window.frame()) {
                return FAIL;
            }
        }
        window.w_winbar_height = winbar_height;
        set_inner_size(window, valid_cursor);
        if winbar_height == 0 {
            free_click_defs(window.w_winbar_click_defs, window.w_winbar_click_defs_size);
            window.w_winbar_click_defs_size = 0 as size_t;
            window.w_winbar_click_defs = ptr::null_mut::<StlClickDefinition>();
        }
    }
    OK
}

pub fn set_winbar(make_room: bool) {
    for wp in windows() {
        if winbar_win(wp, make_room, true) == FAIL {
            break;
        }
    }
}

pub fn tabline_height() -> c_int {
    tabline_rows()
}

/// The rows `'showtabline'` takes off the top of the screen.
pub(crate) fn tabline_rows() -> c_int {
    // Tabline is always 0 when the UI draws it itself.
    if ui_has(kUITabline) {
        return 0;
    }
    debug_assert!(tabs().next().is_some(), "first_tabpage");
    match p_stal.get() {
        // Only draw the tab line for a second tab page.
        1 => tabs().nth(1).map_or(0, |_| 1),
        0 => 0,
        _ => 1,
    }
}

pub fn global_winbar_height() -> c_int {
    global_winbar_rows()
}

/// The rows a global `'winbar'` takes off every window.
pub(crate) fn global_winbar_rows() -> c_int {
    // SAFETY: `'winbar'` is a NUL-terminated option string.
    (unsafe { *p_wbr.get() } as c_int != NUL) as c_int
}

pub fn global_stl_height() -> c_int {
    global_stl_rows()
}

/// The rows a global status line (`'laststatus'` = 3) takes.
pub(crate) fn global_stl_rows() -> c_int {
    if p_ls.get() == 3 as OptInt {
        STATUS_HEIGHT as c_int
    } else {
        0
    }
}

pub fn last_stl_height(morewin: bool) -> c_int {
    last_stl_rows(morewin)
}

/// Whether the last window gets a status line, given `'laststatus'` and
/// whether a window is about to be added.
pub(crate) fn last_stl_rows(morewin: bool) -> c_int {
    let alone = is_only_window(first_window(), None);
    if p_ls.get() > 1 as OptInt || (p_ls.get() == 1 as OptInt && (morewin || !alone)) {
        STATUS_HEIGHT as c_int
    } else {
        0
    }
}

/// The first window of the current tab page.
fn first_window() -> Win {
    windows().next().expect("a tab page has a window")
}

pub fn min_rows(tabpage: TabPage) -> c_int {
    min_rows_of(tabpage)
}

/// The fewest rows tab page `tabpage` can be drawn in.
pub(crate) fn min_rows_of(tabpage: TabPage) -> c_int {
    if windows().next().is_none() {
        return MIN_LINES as c_int;
    }
    let mut total = minheight(tabpage.topframe(), NextCurwin::Unset);
    total += tabline_rows() + global_stl_rows();
    if cmdheight_of(tabpage) > 0 as OptInt {
        total += 1; // Include the last statusline.
    }
    total
}

/// The `'cmdheight'` in force on tab page `tabpage`, which for the current one is
/// the global option rather than the saved copy.
fn cmdheight_of(tabpage: TabPage) -> OptInt {
    if tabpage.is_current() {
        p_ch.get()
    } else {
        tabpage.tp_ch_used
    }
}

pub fn min_rows_for_all_tabpages() -> c_int {
    min_rows_all_tabpages()
}

/// The fewest rows every tab page can be drawn in at once.
pub(crate) fn min_rows_all_tabpages() -> c_int {
    if windows().next().is_none() {
        return MIN_LINES as c_int;
    }
    let mut total = 0;
    for tp in tabs() {
        let mut n = minheight(tp.topframe(), NextCurwin::Unset);
        if cmdheight_of(tp) > 0 as OptInt {
            n += 1;
        }
        total = total.max(n);
    }
    total + tabline_rows() + global_stl_rows()
}

pub fn only_one_window() -> bool {
    // If there is another tab page there always is another window.
    if tabs().nth(1).is_some() {
        return false;
    }
    let count = windows()
        .filter(|wp| {
            if wp.w_buffer.is_null() || is_autocmd_window(Some(*wp)) {
                return false;
            }
            let (help, cur_help) = (buf_is_help(wp.buffer_or_none()), buf_is_help(current_buf()));
            let skip = (help && !cur_help) || wp.w_floating || wp.w_onebuf_opt.wo_pvw != 0;
            !skip || wp.is_current()
        })
        .count();
    count <= 1
}
