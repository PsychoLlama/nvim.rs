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
#![allow(unsafe_code)]

use crate::cstr;
use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

use super::*;
use crate::api::private::helpers::{cstr_as_string, find_window_by_handle, try_enter, try_leave};
use crate::buffer::do_buffer;
use crate::decoration::clear_virttext;
use crate::drawscreen::UPD_NOT_VALID;
use crate::eval::window::{restore_win_noblock, switch_win_noblock};
use crate::fold::deepest_fold_nesting;
use crate::grid::{default_grid_ref, grid_adjust, win_grid_alloc};
use crate::guard::Suppress;
use crate::r#move::textpos2screenpos;
use crate::option::vars::{p_acd, p_ch};
use crate::os::cshim::gettext_ptr;
use crate::plines::win_text_height;
use crate::pos::MAXCOL;
use crate::search::FORWARD;
use crate::types::ui::kUIMultigrid;
use crate::types::{
    Boolean, ColNr, Error, FAIL, Float, Integer, LineNr, OK, Pos, ScreenGrid, SwitchWin, TryState,
    WinConfig, WinStyle, WindowHandle, int64_t, kErrorTypeException, kFloatAnchorEast,
    kFloatAnchorSouth, kFloatRelativeLaststatus, kFloatRelativeTabline, kFloatRelativeWindow,
    size_t,
};
use crate::ui::state::{Columns, Rows};
use crate::ui::{
    ui_call_win_external_pos, ui_call_win_float_pos, ui_call_win_hide, ui_call_win_pos,
    ui_call_win_viewport, ui_check_cursor_grid, ui_has,
};
use crate::ui_compositor::{ui_comp_layers_adjust, ui_comp_put_grid, ui_comp_remove_grid};
use crate::window::state::float_anchor_str;
use crate::winfloat::WIN_CONFIG_INIT;
use crate::winlayer::{Buf, Win};

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
use crate::api_error;

pub fn win_set_buf(win: Win, buffer: Buf) -> Result<(), Error> {
    set_buf(win, buffer)
}

/// Show `buffer` in `win`: switch to the window, switch its buffer with the
/// autocommands that implies, and switch back.
fn set_buf(win: Win, buffer: Buf) -> Result<(), Error> {
    let tab = win_find_tabpage(win.id());
    let _redraw_off = Suppress::redraw();

    let mut switchwin = SwitchWin {
        sw_curwin: None,
        sw_curtab: None,
        sw_same_win: false,
        sw_visual_active: false,
    };
    let mut tstate = TRY_STATE;
    let (sw, ts) = (&raw mut switchwin, &raw mut tstate);
    // SAFETY: `tstate` and `switchwin` are ours and live across the switch; the
    // window and tab page are live.
    let win_result = unsafe {
        try_enter(ts);
        switch_win_noblock(sw, win, tab, true)
    };
    if win_result.is_ok() {
        // Do not trigger 'autochdir' in the window we switched to.
        let save_acd = p_acd.get();
        if !switchwin.sw_same_win {
            p_acd.set(0);
        }
        let (goto, first, fwd) = (DOBUF_GOTO as c_int, DOBUF_FIRST as c_int, FORWARD as c_int);
        let nr = buffer.handle();
        let _ = do_buffer(goto, first, fwd, nr, 0);
        if !switchwin.sw_same_win {
            p_acd.set(save_acd);
        }
    }
    // SAFETY: `tstate` is the state `try_enter` saved.
    let mut caught = unsafe { try_leave(&raw mut tstate) };
    if win_result.is_err() && caught.is_ok() {
        let handle = win.id().handle();
        caught = Err(api_error!(
            kErrorTypeException,
            "Failed to switch to window {handle}"
        ));
    }
    Win::current().validate_cursor();
    // SAFETY: the state `switch_win_noblock` saved.
    unsafe { restore_win_noblock(&raw mut switchwin, true) };
    caught
}

pub fn win_fdccol_count(window: Win) -> c_int {
    fdccol_count(window)
}

/// The columns `'foldcolumn'` asks for in `window`, `auto[:N]` resolved against how
/// deeply its folds are nested.
fn fdccol_count(window: Win) -> c_int {
    let fdc = window.w_onebuf_opt.wo_fdc;
    // SAFETY: `'foldcolumn'` is a NUL-terminated option string, so the first
    // four bytes and -- once they read `auto` -- the two after them are inside
    // it.
    let byte = |n: isize| unsafe { *fdc.offset(n) } as c_int;
    // SAFETY: as above.
    if !unsafe { cstr::starts_with(fdc, b"auto") } {
        return byte(0) - '0' as c_int;
    }
    let fdccol = if byte(4) == ':' as c_int {
        byte(5) - '0' as c_int
    } else {
        1
    };
    // SAFETY: a live window.
    fdccol.min(deepest_fold_nesting(window))
}

/// # Safety
///
/// `dst` must point at a live `WinConfig`, unaliased for the call.
pub unsafe fn merge_win_config(dst: *mut WinConfig, src: WinConfig) {
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

/// # Safety
///
/// `fconfig` must point at a live `WinConfig`, unaliased for the call.
pub unsafe fn clear_float_config(fconfig: *mut WinConfig, free_fields: bool) {
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

pub fn ui_ext_win_position(window: Win, validate: bool) {
    ext_win_position(window, validate);
}

/// Tell the UI where `wp` is: its position on the screen for an ordinary
/// window, and where its own grid is anchored for a float.
fn ext_win_position(window: Win, validate: bool) {
    let mut wp = window;
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
                wp.handle as WindowHandle,
                wp.w_winrow as Integer,
                wp.w_wincol as Integer,
                wp.w_width as Integer,
                wp.w_height as Integer,
            );
        }
        return;
    }
    let c = wp.w_config.clone();
    if c.external {
        ui_call_win_external_pos(wp.w_grid_alloc.handle as Integer, wp.handle as WindowHandle);
        return;
    }

    let mut grid = default_grid_ref().raw();
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
            wp.handle as WindowHandle,
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
fn parent_window(handle: WindowHandle) -> Option<Win> {
    // A parent that has gone away is not an error here, only a `None`.
    find_window_by_handle(handle).unwrap_or_default()
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
    if parent.w_pos_changed && parent.w_grid_alloc.is_allocated() && win_valid(parent.id()) {
        ext_win_position(parent, validate);
    }
    let (mut row_off, mut col_off) = (0, 0);
    // SAFETY: a live window and its own grid.
    unsafe { win_grid_alloc(parent) };
    let own = parent.w_grid;
    // SAFETY: as above; `win_grid_alloc` has just run for this view.
    *grid = unsafe { grid_adjust(own, &mut row_off, &mut col_off) }.raw();
    *row += row_off as Float;
    *col += col_off as Float;
    if c.bufpos.lnum < 0 as LineNr {
        return;
    }
    // The line after the one `bufpos` names, clamped to the buffer. Widened:
    // `bufpos={INT_MAX, ...}` reaches here, and the C's `lnum + 1` wraps.
    let lnum = (c.bufpos.lnum as i64 + 1).min(parent.buffer().line_count() as i64);
    let mut pos = Pos {
        lnum: lnum as LineNr,
        col: c.bufpos.col,
        coladd: 0 as ColNr,
    };
    let (mut trow, mut tcol, mut tcolc, mut tcole) = (0, 0, 0, 0);
    let at = &raw mut pos;
    let (r, c1, c2, c3) = (&raw mut trow, &raw mut tcol, &raw mut tcolc, &raw mut tcole);
    // SAFETY: a live window and a position in its buffer, plus four
    // out-parameters of ours.
    unsafe { textpos2screenpos(parent, at, r, c1, c2, c3, true) };
    *row += (trow - 1) as Float;
    *col += (tcol - 1) as Float;
}

pub fn ui_ext_win_viewport(window: Win) {
    ext_win_viewport(window);
}

/// Tell the UI which part of its buffer `window` shows, and how far the text
/// scrolled since the last time it was told.
fn ext_win_viewport(window: Win) {
    let mut window = window;
    if !((window.is_current() || ui_has(kUIMultigrid))
        && window.w_viewport_invalid
        && window.w_redr_type == 0)
    {
        return;
    }
    let line_count = window.buffer().line_count();
    let cur_topline = window.w_topline.min(line_count);
    let cur_botline = window.w_botline.min(line_count);
    let mut delta = 0 as int64_t;
    let mut last_topline = window.w_viewport_last_topline;
    let mut last_botline = window.w_viewport_last_botline;
    let mut last_topfill = window.w_viewport_last_topfill as c_int;
    let mut last_skipcol = window.w_viewport_last_skipcol as int64_t;
    // Lines were removed below the last known top line.
    if last_topline > line_count {
        delta -= (last_topline - line_count) as int64_t;
        last_topline = line_count;
        last_topfill = 0;
        last_skipcol = MAXCOL as c_int as int64_t;
    }
    last_botline = last_botline.min(line_count);

    if cur_topline < last_topline
        || (cur_topline == last_topline && (window.w_skipcol as int64_t) < last_skipcol)
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
            window,
            cur_topline,
            window.w_skipcol as int64_t,
            &mut lnume,
            &mut vcole,
        );
    } else if cur_topline > last_topline
        || (cur_topline == last_topline && window.w_skipcol as int64_t > last_skipcol)
    {
        // Scrolled down.
        let mut vcole = window.w_skipcol as int64_t;
        let mut lnume = cur_topline;
        if last_botline > 0 && cur_topline > last_botline {
            delta += (cur_topline - last_botline) as int64_t;
            lnume = last_botline;
            vcole = 0;
        }
        delta += text_height(window, last_topline, last_skipcol, &mut lnume, &mut vcole);
    }
    delta += last_topfill as int64_t;
    delta -= window.w_topfill as int64_t;

    // `w_botline` is one past the last line, except when the last line is not
    // fully visible.
    let mut ev_botline = window.w_botline;
    if ev_botline == line_count + 1 && window.w_empty_rows == 0 {
        ev_botline = line_count;
    }
    {
        ui_call_win_viewport(
            window.w_grid_alloc.handle as Integer,
            window.handle as WindowHandle,
            (window.w_topline - 1) as Integer,
            ev_botline as Integer,
            (window.w_cursor.lnum - 1) as Integer,
            window.w_cursor.col as Integer,
            line_count as Integer,
            delta as Integer,
        );
    }
    window.w_viewport_invalid = false;
    window.w_viewport_last_topline = window.w_topline;
    window.w_viewport_last_botline = window.w_botline;
    window.w_viewport_last_topfill = window.w_topfill as LineNr;
    window.w_viewport_last_skipcol = window.w_skipcol as LineNr;
}

/// The screen lines between two buffer positions, `win_text_height()` with its
/// two in-out parameters borrowed rather than pointed at.
fn text_height(
    window: Win,
    start_lnum: LineNr,
    start_vcol: int64_t,
    end_lnum: &mut LineNr,
    end_vcol: &mut int64_t,
) -> int64_t {
    let (none, all) = (ptr::null_mut::<int64_t>(), INT64_MAX as int64_t);
    // SAFETY: two lines of the window's own buffer, and two out-parameters of
    // the caller's.
    unsafe {
        win_text_height(
            window, start_lnum, start_vcol, end_lnum, end_vcol, none, all,
        )
    }
}

// ---------------------------------------------------------------------------
// May the layout change at all?

pub fn check_split_disallowed(window: Win) -> c_int {
    match check_split_disallowed_err(window) {
        Ok(()) => OK,
        Err(e) => {
            // SAFETY: the message the check just wrote, owned by `e`.
            unsafe { emsg(gettext_ptr(e.message_or_empty().as_ptr())) };
            FAIL
        }
    }
}

pub fn check_split_disallowed_err(window: Win) -> Result<(), Error> {
    if split_disallowed.get() > 0 {
        return Err(Error::exception(
            c"E242: Can't split a window while closing another",
        ));
    }
    if window.buffer().b_locked_split != 0 {
        return Err(Error::exception(e_cannot_split_window_when_closing_buffer));
    }
    Ok(())
}
