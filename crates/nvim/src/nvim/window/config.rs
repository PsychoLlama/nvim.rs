//! A window's configuration -- which buffer it shows, and where the UI is
//! told it sits.
//!
//! [`win_set_buf`] is the `nvim_win_set_buf()` half: switch to the window,
//! switch its buffer, and switch back, with the autocommands that implies.
//! [`ui_ext_win_position`] and [`ui_ext_win_viewport`] are the outbound half --
//! they tell an external UI where a floating window's grid is anchored and
//! which part of the buffer each window currently shows.
//! [`clear_float_config`] and [`merge_win_config`] normalise a `WinConfig`,
//! and the `check_split_disallowed` pair is the guard every layout change
//! asks first.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_set_error, cstr_as_string, find_window_by_handle, try_enter, try_leave,
};
use crate::src::nvim::buffer::do_buffer;
use crate::src::nvim::decoration::clear_virttext;
use crate::src::nvim::drawscreen::UPD_NOT_VALID;
use crate::src::nvim::eval::window::{restore_win_noblock, switch_win_noblock};
use crate::src::nvim::fold::getDeepestNesting;
use crate::src::nvim::grid::{grid_adjust, win_grid_alloc};
use crate::src::nvim::main::{
    Columns, RedrawingDisabled, Rows, default_grid, float_anchor_str, p_acd, p_ch,
};
use crate::src::nvim::r#move::textpos2screenpos;
use crate::src::nvim::os::libc::strncmp;
use crate::src::nvim::plines::win_text_height;
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::search::FORWARD;
use crate::src::nvim::types::ui::kUIMultigrid;
use crate::src::nvim::types::{
    Boolean, Error, Float, Integer, ScreenGrid, TryState, WinConfig, WinStyle, Window, buf_T,
    colnr_T, int64_t, kErrorTypeException, kErrorTypeNone, kFloatAnchorEast, kFloatAnchorSouth,
    kFloatRelativeLaststatus, kFloatRelativeTabline, kFloatRelativeWindow, linenr_T, pos_T, size_t,
    switchwin_T, win_T,
};
use crate::src::nvim::ui::{
    ui_call_win_external_pos, ui_call_win_float_pos, ui_call_win_hide, ui_call_win_pos,
    ui_call_win_viewport, ui_check_cursor_grid, ui_has,
};
use crate::src::nvim::ui_compositor::{
    ui_comp_layers_adjust, ui_comp_put_grid, ui_comp_remove_grid,
};
use crate::src::nvim::winfloat::WIN_CONFIG_INIT;
use crate::src::nvim::winlayer::{Buf, Win};

/// The zeroed `TryState` `try_enter()` fills in.
const TRY_STATE: TryState = TryState {
    current_exception: ptr::null_mut(),
    private_msg_list: ptr::null_mut(),
    msg_list: ptr::null(),
    got_int: 0,
    did_throw: false,
    need_rethrow: 0,
    did_emsg: 0,
};

pub unsafe extern "C" fn win_set_buf(win: *mut win_T, buf: *mut buf_T, err: *mut Error) {
    // SAFETY: the caller's promise -- a live window, a live buffer and a live
    // `Error` to report through.
    unsafe { set_buf(Win::new(win), Buf::new(buf), &mut *err) };
}

/// Show `buf` in `win`: switch to the window, switch its buffer with the
/// autocommands that implies, and switch back.
fn set_buf(win: Win, buf: Buf, err: &mut Error) {
    let win_handle = win.handle;
    // SAFETY: a live window; the answer is its tab page.
    let tab = unsafe { win_find_tabpage(win.raw()) };
    RedrawingDisabled.set(RedrawingDisabled.get() + 1);

    let mut switchwin = switchwin_T {
        sw_curwin: ptr::null_mut::<win_T>(),
        sw_curtab: ptr::null_mut(),
        sw_same_win: false,
        sw_visual_active: false,
    };
    let mut tstate = TRY_STATE;
    let (sw, ts) = (&raw mut switchwin, &raw mut tstate);
    // SAFETY: `tstate` and `switchwin` are ours and live across the switch; the
    // window and tab page are live.
    let win_result = unsafe {
        try_enter(ts);
        switch_win_noblock(sw, win.raw(), tab, true)
    };
    if win_result != 0 {
        // Do not trigger 'autochdir' in the window we switched to.
        let save_acd = p_acd.get();
        if !switchwin.sw_same_win {
            p_acd.set(0);
        }
        let (goto, first, fwd) = (DOBUF_GOTO as c_int, DOBUF_FIRST as c_int, FORWARD as c_int);
        let nr = buf.handle as c_int;
        // SAFETY: a live buffer handle; this fires the Buf* autocommands.
        unsafe { do_buffer(goto, first, fwd, nr, 0) };
        if !switchwin.sw_same_win {
            p_acd.set(save_acd);
        }
    }
    // SAFETY: `tstate` is the state `try_enter` saved, and `err` is live.
    unsafe { try_leave(&raw mut tstate, err) };
    if win_result == FAIL && err.type_0 as c_int == kErrorTypeNone as c_int {
        let fmt = c"Failed to switch to window %d".as_ptr();
        // SAFETY: a live `Error` and a static format string.
        unsafe { api_set_error(err, kErrorTypeException, fmt, win_handle) };
    }
    cur_win().validate_cursor();
    // SAFETY: the state `switch_win_noblock` saved.
    unsafe { restore_win_noblock(&raw mut switchwin, true) };
    RedrawingDisabled.set(RedrawingDisabled.get() - 1);
}

pub unsafe extern "C" fn win_fdccol_count(wp: *mut win_T) -> c_int {
    // SAFETY: the caller's promise -- a live window.
    fdccol_count(unsafe { Win::new(wp) })
}

/// The columns `'foldcolumn'` asks for in `wp`, `auto[:N]` resolved against how
/// deeply its folds are nested.
fn fdccol_count(wp: Win) -> c_int {
    let fdc = wp.w_onebuf_opt.wo_fdc;
    // SAFETY: `'foldcolumn'` is a NUL-terminated option string, so the first
    // four bytes and -- once they read `auto` -- the two after them are inside
    // it.
    let byte = |n: isize| unsafe { *fdc.offset(n) } as c_int;
    // SAFETY: as above.
    if unsafe { strncmp(fdc, c"auto".as_ptr(), 4 as size_t) } != 0 {
        return byte(0) - '0' as c_int;
    }
    let fdccol = if byte(4) == ':' as c_int {
        byte(5) - '0' as c_int
    } else {
        1
    };
    // SAFETY: a live window.
    fdccol.min(unsafe { getDeepestNesting(wp.raw()) })
}

pub unsafe extern "C" fn merge_win_config(dst: *mut WinConfig, src: WinConfig) {
    // SAFETY: the caller's promise -- a live config to overwrite.
    unsafe { merge(&mut *dst, src) };
}

/// Overwrite `dst` with `src`, freeing the title and footer text `dst` owned
/// and `src` does not take over.
fn merge(dst: &mut WinConfig, src: WinConfig) {
    if dst.title_chunks.items != src.title_chunks.items {
        // SAFETY: the config's own virtual-text array.
        unsafe { clear_virttext(&raw mut dst.title_chunks) };
    }
    if dst.footer_chunks.items != src.footer_chunks.items {
        // SAFETY: as above.
        unsafe { clear_virttext(&raw mut dst.footer_chunks) };
    }
    *dst = src;
}

pub unsafe extern "C" fn clear_float_config(fconfig: *mut WinConfig, free_fields: bool) {
    // SAFETY: the caller's promise -- a live config.
    unsafe { clear_float(&mut *fconfig, free_fields) };
}

/// Put `fconfig` back to the defaults, keeping the two fields a window carries
/// across becoming an ordinary window.
fn clear_float(fconfig: &mut WinConfig, free_fields: bool) {
    let saved_style: WinStyle = fconfig.style;
    let saved_cmdline_offset = fconfig._cmdline_offset;
    if free_fields {
        merge(fconfig, WIN_CONFIG_INIT);
    } else {
        *fconfig = WIN_CONFIG_INIT;
    }
    fconfig.style = saved_style;
    fconfig._cmdline_offset = saved_cmdline_offset;
}

// ---------------------------------------------------------------------------
// Telling the UI where a window sits

pub unsafe extern "C" fn ui_ext_win_position(wp: *mut win_T, validate: bool) {
    // SAFETY: the caller's promise -- a live window.
    ext_win_position(unsafe { Win::new(wp) }, validate);
}

/// Tell the UI where `wp` is: its position on the screen for an ordinary
/// window, and where its own grid is anchored for a float.
fn ext_win_position(wp: Win, validate: bool) {
    let mut wp = wp;
    wp.w_pos_changed = false;
    if !wp.w_floating {
        if ui_has(kUIMultigrid) {
            wp.w_grid_alloc.comp_col = wp.w_wincol;
            wp.w_grid_alloc.comp_row = wp.w_winrow;
        }
        // Tell the UI where the window is.
        {
            ui_call_win_pos(
                wp.w_grid_alloc.handle as Integer,
                wp.handle as Window,
                wp.w_winrow as Integer,
                wp.w_wincol as Integer,
                wp.w_width as Integer,
                wp.w_height as Integer,
            );
        }
        return;
    }
    let c: WinConfig = wp.w_config;
    if c.external {
        ui_call_win_external_pos(wp.w_grid_alloc.handle as Integer, wp.handle as Window);
        return;
    }

    let mut grid = default_grid.ptr();
    let mut row = c.row as Float;
    let mut col = c.col as Float;
    if c.relative as c_uint == kFloatRelativeWindow as c_uint {
        if let Some(parent) = parent_window(c.window) {
            anchor_to_window(parent, &c, validate, &mut grid, &mut row, &mut col);
        }
    } else if c.relative as c_uint == kFloatRelativeLaststatus as c_uint {
        row += (Rows.get() - p_ch.get() as c_int - last_stl_rows(false)) as Float;
    } else if c.relative as c_uint == kFloatRelativeTabline as c_uint {
        row += tabline_rows() as Float;
    }

    // A changed 'zindex' means the float has to move within the compositor's
    // stack of layers.
    let resort =
        wp.w_grid_alloc.comp_index != 0 as size_t && wp.w_grid_alloc.zindex != wp.w_config.zindex;
    let raise = resort && wp.w_grid_alloc.zindex < wp.w_config.zindex;
    wp.w_grid_alloc.zindex = wp.w_config.zindex;
    if resort {
        ui_comp_layers_adjust(wp.w_grid_alloc.comp_index, raise);
    }

    let valid = wp.w_redr_type == 0 || ui_has(kUIMultigrid);
    if !valid && !validate {
        wp.w_pos_changed = true;
        return;
    }

    let east = c.anchor as c_int & kFloatAnchorEast as c_int != 0;
    let south = c.anchor as c_int & kFloatAnchorSouth as c_int != 0;
    let mut comp_row = row as c_int - if south { wp.w_height_outer } else { 0 };
    let mut comp_col = col as c_int - if east { wp.w_width_outer } else { 0 };
    // Don't cover the command line unless the float sits above the messages.
    let above_ch = if wp.w_config.zindex < kZIndexMessages as c_int {
        p_ch.get() as c_int
    } else {
        0
    };
    // SAFETY: `grid` is the default grid or the parent window's, both live.
    unsafe {
        comp_row += (*grid).comp_row;
        comp_col += (*grid).comp_col;
    }
    comp_row = comp_row
        .min(Rows.get() - wp.w_height_outer - above_ch)
        .max(0);
    if !c.fixed || east {
        comp_col = comp_col.min(Columns.get() - wp.w_width_outer).max(0);
    }
    wp.w_winrow = comp_row;
    wp.w_wincol = comp_col;

    if c.hide {
        if ui_has(kUIMultigrid) {
            ui_call_win_hide(wp.w_grid_alloc.handle as Integer);
        }
        // SAFETY: the window's own grid.
        unsafe { ui_comp_remove_grid(&raw mut wp.w_grid_alloc) };
        return;
    }
    let (own, h, w) = (
        &raw mut wp.w_grid_alloc,
        wp.w_height_outer,
        wp.w_width_outer,
    );
    // SAFETY: the window's own grid.
    unsafe { ui_comp_put_grid(own, comp_row, comp_col, h, w, valid, false) };
    if ui_has(kUIMultigrid) {
        // SAFETY: `float_anchor_str` is an array of static strings indexed by
        // the anchor, and `grid` is a live grid.
        let (anchor, anchor_grid) = unsafe {
            let names = (&raw const float_anchor_str).cast::<*const c_char>();
            (
                cstr_as_string(*names.offset(c.anchor as isize)),
                (*grid).handle,
            )
        };
        ui_call_win_float_pos(
            wp.w_grid_alloc.handle as Integer,
            wp.handle as Window,
            anchor,
            anchor_grid as Integer,
            row,
            col,
            c.mouse as Boolean,
            wp.w_grid_alloc.zindex as Integer,
            wp.w_grid_alloc.comp_index as c_int as Integer,
            wp.w_winrow as Integer,
            wp.w_wincol as Integer,
        );
    }
    ui_check_cursor_grid(wp.w_grid_alloc.handle);
    wp.w_grid_alloc.mouse_enabled = wp.w_config.mouse;
    if !valid {
        wp.w_grid_alloc.valid = false;
        wp.redraw_later(UPD_NOT_VALID);
    }
}

/// The window a `relative='win'` float is anchored to, if it is still there.
fn parent_window(handle: Window) -> Option<Win> {
    let mut dummy = Error {
        type_0: kErrorTypeNone,
        msg: ptr::null_mut::<c_char>(),
    };
    // SAFETY: a live `Error` of ours; the answer is a live window or null.
    unsafe {
        let win = find_window_by_handle(handle, &raw mut dummy);
        api_clear_error(&raw mut dummy);
        Win::from_raw(win)
    }
}

/// Move `row`/`col` from the parent window's grid onto the screen, resolving
/// `bufpos` to a screen position when it is set.
fn anchor_to_window(
    parent: Win,
    c: &WinConfig,
    validate: bool,
    grid: &mut *mut ScreenGrid,
    row: &mut Float,
    col: &mut Float,
) {
    // SAFETY: only compares the pointer against the window list.
    if parent.w_pos_changed
        && !parent.w_grid_alloc.chars.is_null()
        && unsafe { win_valid(parent.raw()) }
    {
        ext_win_position(parent, validate);
    }
    let mut parent = parent;
    let (mut row_off, mut col_off) = (0, 0);
    let (own, r, c1) = (&raw mut parent.w_grid, &raw mut row_off, &raw mut col_off);
    // SAFETY: a live window and its own grid.
    unsafe { win_grid_alloc(parent.raw()) };
    // SAFETY: as above, plus two out-parameters of ours.
    *grid = unsafe { grid_adjust(own, r, c1) };
    *row += row_off as Float;
    *col += col_off as Float;
    if c.bufpos.lnum < 0 as linenr_T {
        return;
    }
    // The line after the one `bufpos` names, clamped to the buffer. Widened:
    // `bufpos={INT_MAX, ...}` reaches here, and the C's `lnum + 1` wraps.
    let lnum = (c.bufpos.lnum as i64 + 1).min(parent.buffer().line_count() as i64);
    let mut pos = pos_T {
        lnum: lnum as linenr_T,
        col: c.bufpos.col,
        coladd: 0 as colnr_T,
    };
    let (mut trow, mut tcol, mut tcolc, mut tcole) = (0, 0, 0, 0);
    let (win, at) = (parent.raw(), &raw mut pos);
    let (r, c1, c2, c3) = (&raw mut trow, &raw mut tcol, &raw mut tcolc, &raw mut tcole);
    // SAFETY: a live window and a position in its buffer, plus four
    // out-parameters of ours.
    unsafe { textpos2screenpos(win, at, r, c1, c2, c3, true) };
    *row += (trow - 1) as Float;
    *col += (tcol - 1) as Float;
}

pub unsafe extern "C" fn ui_ext_win_viewport(wp: *mut win_T) {
    // SAFETY: the caller's promise -- a live window.
    ext_win_viewport(unsafe { Win::new(wp) });
}

/// Tell the UI which part of its buffer `wp` shows, and how far the text
/// scrolled since the last time it was told.
fn ext_win_viewport(wp: Win) {
    let mut wp = wp;
    if !((wp.is_current() || ui_has(kUIMultigrid)) && wp.w_viewport_invalid && wp.w_redr_type == 0)
    {
        return;
    }
    let line_count = wp.buffer().line_count();
    let cur_topline = wp.w_topline.min(line_count);
    let cur_botline = wp.w_botline.min(line_count);
    let mut delta = 0 as int64_t;
    let mut last_topline = wp.w_viewport_last_topline;
    let mut last_botline = wp.w_viewport_last_botline;
    let mut last_topfill = wp.w_viewport_last_topfill as c_int;
    let mut last_skipcol = wp.w_viewport_last_skipcol as int64_t;
    // Lines were removed below the last known top line.
    if last_topline > line_count {
        delta -= (last_topline - line_count) as int64_t;
        last_topline = line_count;
        last_topfill = 0;
        last_skipcol = MAXCOL as c_int as int64_t;
    }
    last_botline = last_botline.min(line_count);

    if cur_topline < last_topline
        || (cur_topline == last_topline && (wp.w_skipcol as int64_t) < last_skipcol)
    {
        // Scrolled up: measure the text between the two positions.
        let mut vcole = last_skipcol;
        let mut lnume = last_topline;
        if last_topline > 0 && cur_botline < last_topline {
            delta -= (last_topline - cur_botline) as int64_t;
            lnume = cur_botline;
            vcole = 0;
        }
        delta -= text_height(
            wp,
            cur_topline,
            wp.w_skipcol as int64_t,
            &mut lnume,
            &mut vcole,
        );
    } else if cur_topline > last_topline
        || (cur_topline == last_topline && wp.w_skipcol as int64_t > last_skipcol)
    {
        // Scrolled down.
        let mut vcole = wp.w_skipcol as int64_t;
        let mut lnume = cur_topline;
        if last_botline > 0 && cur_topline > last_botline {
            delta += (cur_topline - last_botline) as int64_t;
            lnume = last_botline;
            vcole = 0;
        }
        delta += text_height(wp, last_topline, last_skipcol, &mut lnume, &mut vcole);
    }
    delta += last_topfill as int64_t;
    delta -= wp.w_topfill as int64_t;

    // `w_botline` is one past the last line, except when the last line is not
    // fully visible.
    let mut ev_botline = wp.w_botline;
    if ev_botline == line_count + 1 && wp.w_empty_rows == 0 {
        ev_botline = line_count;
    }
    {
        ui_call_win_viewport(
            wp.w_grid_alloc.handle as Integer,
            wp.handle as Window,
            (wp.w_topline - 1) as Integer,
            ev_botline as Integer,
            (wp.w_cursor.lnum - 1) as Integer,
            wp.w_cursor.col as Integer,
            line_count as Integer,
            delta as Integer,
        );
    }
    wp.w_viewport_invalid = false;
    wp.w_viewport_last_topline = wp.w_topline;
    wp.w_viewport_last_botline = wp.w_botline;
    wp.w_viewport_last_topfill = wp.w_topfill as linenr_T;
    wp.w_viewport_last_skipcol = wp.w_skipcol as linenr_T;
}

/// The screen lines between two buffer positions, `win_text_height()` with its
/// two in-out parameters borrowed rather than pointed at.
fn text_height(
    wp: Win,
    start_lnum: linenr_T,
    start_vcol: int64_t,
    end_lnum: &mut linenr_T,
    end_vcol: &mut int64_t,
) -> int64_t {
    let (win, none, all) = (wp.raw(), ptr::null_mut::<int64_t>(), INT64_MAX as int64_t);
    // SAFETY: a live window, two lines of its buffer, and two out-parameters of
    // the caller's.
    unsafe { win_text_height(win, start_lnum, start_vcol, end_lnum, end_vcol, none, all) }
}

// ---------------------------------------------------------------------------
// May the layout change at all?

pub unsafe extern "C" fn check_split_disallowed(wp: *const win_T) -> c_int {
    let mut err = Error {
        type_0: kErrorTypeNone,
        msg: ptr::null_mut::<c_char>(),
    };
    // SAFETY: the caller's promise -- a live window; `err` is ours.
    let ok = unsafe { check_split_disallowed_err(wp, &raw mut err) };
    if err.type_0 as c_int != kErrorTypeNone as c_int {
        // SAFETY: the message `api_set_error` just wrote.
        unsafe { emsg(gettext(err.msg)) };
        // SAFETY: as above.
        unsafe { api_clear_error(&raw mut err) };
    }
    if ok { OK } else { FAIL }
}

pub unsafe extern "C" fn check_split_disallowed_err(wp: *const win_T, err: *mut Error) -> bool {
    if split_disallowed.get() > 0 {
        let msg = c"E242: Can't split a window while closing another".as_ptr();
        // SAFETY: a live `Error` and a static message.
        unsafe { api_set_error(err, kErrorTypeException, msg) };
        return false;
    }
    // SAFETY: the caller's promise -- a live window, whose buffer is live.
    if unsafe { Win::new(wp as *mut win_T) }
        .buffer()
        .b_locked_split
        != 0
    {
        let msg = e_cannot_split_window_when_closing_buffer.as_ptr();
        // SAFETY: as above.
        unsafe { api_set_error(err, kErrorTypeException, c"%s".as_ptr(), msg) };
        return false;
    }
    true
}
