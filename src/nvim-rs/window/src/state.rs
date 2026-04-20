//! Window state accessors.
//!
//! This module provides FFI exports for accessing and modifying
//! window state related to layout, cursor position, view settings,
//! and other fields needed for window management.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::win_struct::{win_mut, win_ref};
use crate::WinHandle;

// Type aliases matching C types
type LineNr = c_int;
type ColNr = c_int;

// =============================================================================
// External C accessor functions (only those that cannot be replaced)
// =============================================================================

extern "C" {
    // nvim_win_argcount uses C macro WARGCOUNT -- cannot be replaced with direct field access
    fn nvim_win_argcount(wp: WinHandle) -> c_int;
}

// =============================================================================
// Window Dimensions FFI Exports
// =============================================================================

/// Get window width.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_width(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_width
}

/// Get window height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_height
}

/// Get window field width (raw w_width).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_field_width(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_width
}

/// Get window field height (raw w_height).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_field_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_height
}

/// Set window field width.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_field_width(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_width = val;
    }
}

/// Set window field height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_field_height(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_height = val;
    }
}

// =============================================================================
// Window Position FFI Exports
// =============================================================================

/// Get window row position in screen.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_winrow(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_winrow
}

/// Get window column position in screen.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_wincol(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_wincol
}

/// Set window row position.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_winrow(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_winrow = val;
    }
}

/// Set window column position.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_wincol(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_wincol = val;
    }
}

// =============================================================================
// Cursor Position FFI Exports
// =============================================================================

// NOTE: rs_win_get_cursor_lnum is defined in drawline crate

/// Get cursor column.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_cursor_col(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_cursor.col
}

/// Get cursor column add (for virtual columns).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_cursor_coladd(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_cursor.coladd
}

/// Set cursor line number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_cursor_lnum(wp: WinHandle, lnum: LineNr) {
    if !wp.is_null() {
        win_mut(wp).w_cursor.lnum = lnum;
    }
}

/// Set cursor column.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_cursor_col(wp: WinHandle, col: ColNr) {
    if !wp.is_null() {
        win_mut(wp).w_cursor.col = col;
    }
}

/// Get curswant (desired cursor column).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_curswant(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_curswant
}

/// Set curswant.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_curswant(wp: WinHandle, val: ColNr) {
    if !wp.is_null() {
        win_mut(wp).w_curswant = val;
    }
}

/// Get set_curswant flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_set_curswant(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_set_curswant
}

/// Set set_curswant flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_set_curswant(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_set_curswant = val;
    }
}

// =============================================================================
// View Position FFI Exports
// =============================================================================

// NOTE: rs_win_get_topline is defined in drawscreen crate

/// Set topline.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_topline(wp: WinHandle, val: LineNr) {
    if !wp.is_null() {
        win_mut(wp).w_topline = val;
    }
}

// NOTE: rs_win_get_botline is defined in drawscreen crate

/// Set botline.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_botline(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_botline = val;
    }
}

/// Get topfill (filler lines above topline).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_topfill(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_topfill
}

/// Set topfill.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_topfill(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_topfill = val;
    }
}

/// Get leftcol (first visible column).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_leftcol(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_leftcol
}

/// Set leftcol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_leftcol(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_leftcol = val;
    }
}

// NOTE: rs_win_get_skipcol is defined in drawline crate

/// Set skipcol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_skipcol(wp: WinHandle, val: ColNr) {
    if !wp.is_null() {
        win_mut(wp).w_skipcol = val;
    }
}

// =============================================================================
// Separator FFI Exports
// =============================================================================

/// Get horizontal separator height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_hsep_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_hsep_height
}

/// Get vertical separator width.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_vsep_width(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_vsep_width
}

/// Set horizontal separator height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_hsep_height(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_hsep_height = val;
    }
}

/// Set vertical separator width.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_vsep_width(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_vsep_width = val;
    }
}

/// Get status line height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_status_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_status_height
}

/// Set status line height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_status_height(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_status_height = val;
    }
}

/// Get winbar height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_winbar_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_winbar_height
}

// =============================================================================
// Redraw State FFI Exports
// =============================================================================

/// Get redraw status flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_redr_status(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    c_int::from(win_ref(wp).w_redr_status)
}

/// Set redraw status flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_redr_status(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_redr_status = val != 0;
    }
}

// NOTE: rs_win_get_redr_type is defined in drawscreen crate

// NOTE: rs_win_set_redr_type is defined in drawscreen crate

// NOTE: rs_win_get_lines_valid is defined in drawscreen crate

// NOTE: rs_win_set_lines_valid is defined in drawscreen crate

/// Set pos_changed flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_pos_changed(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_pos_changed = val != 0;
    }
}

/// Get viewport_invalid flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_viewport_invalid(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    c_int::from(win_ref(wp).w_viewport_invalid)
}

/// Set viewport_invalid flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_viewport_invalid(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_viewport_invalid = val != 0;
    }
}

// =============================================================================
// Redraw Range FFI Exports
// =============================================================================

// NOTE: rs_win_get_redraw_top is defined in drawscreen crate

// NOTE: rs_win_set_redraw_top is defined in drawscreen crate

// NOTE: rs_win_get_redraw_bot is defined in drawscreen crate

// NOTE: rs_win_set_redraw_bot is defined in drawscreen crate

// =============================================================================
// Current Line Info FFI Exports
// =============================================================================

/// Get cline_row (screen row of cursor line).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_cline_row(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_cline_row
}

/// Set cline_row.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_cline_row(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_cline_row = val;
    }
}

/// Get cline_height (screen height of cursor line).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_cline_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 1;
    }
    win_ref(wp).w_cline_height
}

/// Set cline_height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_cline_height(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_cline_height = val;
    }
}

/// Get cline_folded flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_cline_folded(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    c_int::from(win_ref(wp).w_cline_folded)
}

/// Set cline_folded flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_cline_folded(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_cline_folded = val != 0;
    }
}

/// Get cursorline (line number with 'cursorline').
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_cursorline(wp: WinHandle) -> LineNr {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_cursorline
}

// =============================================================================
// Valid Cursor/Scroll FFI Exports
// =============================================================================

/// Get valid cursor line number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_valid_cursor_lnum(wp: WinHandle) -> LineNr {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_valid_cursor.lnum
}

/// Get valid cursor column.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_valid_cursor_col(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_valid_cursor.col
}

/// Get valid cursor coladd.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_valid_cursor_coladd(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_valid_cursor.coladd
}

/// Set valid cursor position.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_valid_cursor(
    wp: WinHandle,
    lnum: LineNr,
    col: ColNr,
    coladd: ColNr,
) {
    if !wp.is_null() {
        let ws = win_mut(wp);
        ws.w_valid_cursor.lnum = lnum;
        ws.w_valid_cursor.col = col;
        ws.w_valid_cursor.coladd = coladd;
    }
}

/// Set valid cursor column.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_valid_cursor_col(wp: WinHandle, col: ColNr) {
    if !wp.is_null() {
        win_mut(wp).w_valid_cursor.col = col;
    }
}

/// Set valid cursor coladd.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_valid_cursor_coladd(wp: WinHandle, coladd: ColNr) {
    if !wp.is_null() {
        win_mut(wp).w_valid_cursor.coladd = coladd;
    }
}

/// Get valid leftcol.
#[unsafe(no_mangle)]
pub const unsafe extern "C" fn rs_win_get_valid_leftcol(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_valid_leftcol
}

/// Set valid leftcol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_valid_leftcol(wp: WinHandle, val: ColNr) {
    if !wp.is_null() {
        win_mut(wp).w_valid_leftcol = val;
    }
}

/// Get valid skipcol.
#[unsafe(no_mangle)]
pub const unsafe extern "C" fn rs_win_get_valid_skipcol(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_valid_skipcol
}

/// Set valid skipcol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_valid_skipcol(wp: WinHandle, val: ColNr) {
    if !wp.is_null() {
        win_mut(wp).w_valid_skipcol = val;
    }
}

// =============================================================================
// Window Display Info FFI Exports
// =============================================================================

/// Get wcol (cursor column in window).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_wcol(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_wcol
}

/// Set wcol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_wcol(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_wcol = val;
    }
}

/// Get wrow (cursor row in window).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_wrow(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_wrow
}

/// Get empty_rows (filler rows at end of window).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_empty_rows(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_empty_rows
}

/// Set empty_rows.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_empty_rows(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).w_empty_rows = val;
    }
}

/// Get endrow (last row of window content = w_winrow + w_height).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_endrow(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    let ws = win_ref(wp);
    ws.w_winrow + ws.w_height
}

/// Get endcol (last column of window content = w_wincol + w_width).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_endcol(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    let ws = win_ref(wp);
    ws.w_wincol + ws.w_width
}

// =============================================================================
// Window Options FFI Exports
// =============================================================================

/// Get 'wrap' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_wrap(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_p_wrap()
}

/// Get 'rightleft' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_rl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_p_rl()
}

/// Set 'rightleft' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_p_rl(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        win_mut(wp).set_w_p_rl(val);
    }
}

/// Get 'breakindent' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_bri(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_p_bri()
}

/// Get 'cursorline' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_cul(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_p_cul()
}

/// Get 'cursorcolumn' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_cuc(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_p_cuc()
}

/// Get cursorlineopt flags.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_culopt_flags(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    c_int::from(win_ref(wp).w_p_culopt_flags)
}

/// Get 'number' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_nu(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_p_nu()
}

/// Get 'relativenumber' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_rnu(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_p_rnu()
}

/// Get 'list' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_list(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_p_list()
}

/// Get 'diff' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_diff(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_p_diff()
}

/// Get 'foldenable' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_fen(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_p_fen()
}

/// Get 'arabic' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_arab(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_p_arab()
}

// =============================================================================
// Misc Window State FFI Exports
// =============================================================================

/// Get virtual column under cursor.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_virtcol(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_virtcol
}

/// Get wrap flags.
#[unsafe(no_mangle)]
pub const unsafe extern "C" fn rs_win_get_wrap_flags(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_p_wrap_flags()
}

/// Get argument index.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_arg_idx(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_arg_idx
}

/// Get argument index invalid flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_arg_idx_invalid(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    win_ref(wp).w_arg_idx_invalid
}

/// Get argument count.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_argcount(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_argcount(wp)
}

// =============================================================================
// Composite Structures for FFI
// =============================================================================

/// Window cursor position.
#[repr(C)]
pub struct WinCursor {
    /// Line number (1-based)
    pub lnum: LineNr,
    /// Column (0-based byte offset)
    pub col: ColNr,
    /// Virtual column add (for wide chars)
    pub coladd: ColNr,
}

/// Get cursor position as a struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_cursor(wp: WinHandle) -> WinCursor {
    if wp.is_null() {
        return WinCursor {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
    }
    let ws = win_ref(wp);
    WinCursor {
        lnum: ws.w_cursor.lnum,
        col: ws.w_cursor.col,
        coladd: ws.w_cursor.coladd,
    }
}

/// Window view position.
#[repr(C)]
pub struct WinView {
    /// First visible line
    pub topline: LineNr,
    /// Last visible line + 1
    pub botline: LineNr,
    /// Filler lines above topline
    pub topfill: c_int,
    /// First visible column
    pub leftcol: ColNr,
    /// Columns skipped for wide lines
    pub skipcol: ColNr,
}

/// Get view position as a struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_view(wp: WinHandle) -> WinView {
    if wp.is_null() {
        return WinView {
            topline: 1,
            botline: 1,
            topfill: 0,
            leftcol: 0,
            skipcol: 0,
        };
    }
    let ws = win_ref(wp);
    WinView {
        topline: ws.w_topline,
        botline: ws.w_botline,
        topfill: ws.w_topfill,
        leftcol: ws.w_leftcol,
        skipcol: ws.w_skipcol,
    }
}

/// Window dimensions.
#[repr(C)]
pub struct WinDimensions {
    /// Window width
    pub width: c_int,
    /// Window height
    pub height: c_int,
    /// Window row in screen
    pub winrow: c_int,
    /// Window column in screen
    pub wincol: c_int,
    /// Horizontal separator height
    pub hsep_height: c_int,
    /// Vertical separator width
    pub vsep_width: c_int,
    /// Status line height
    pub status_height: c_int,
    /// Winbar height
    pub winbar_height: c_int,
}

/// Get window dimensions as a struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_dimensions(wp: WinHandle) -> WinDimensions {
    if wp.is_null() {
        return WinDimensions {
            width: 0,
            height: 0,
            winrow: 0,
            wincol: 0,
            hsep_height: 0,
            vsep_width: 0,
            status_height: 0,
            winbar_height: 0,
        };
    }
    let ws = win_ref(wp);
    WinDimensions {
        width: ws.w_width,
        height: ws.w_height,
        winrow: ws.w_winrow,
        wincol: ws.w_wincol,
        hsep_height: ws.w_hsep_height,
        vsep_width: ws.w_vsep_width,
        status_height: ws.w_status_height,
        winbar_height: ws.w_winbar_height,
    }
}

/// Get total window height including separators.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_total_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    let ws = win_ref(wp);
    ws.w_height + ws.w_hsep_height + ws.w_status_height + ws.w_winbar_height
}

/// Get total window width including separator.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_total_width(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    let ws = win_ref(wp);
    ws.w_width + ws.w_vsep_width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_safety() {
        unsafe {
            // All functions should handle null gracefully
            assert_eq!(rs_win_get_width(WinHandle::null()), 0);
            assert_eq!(rs_win_get_height(WinHandle::null()), 0);
            assert_eq!(rs_win_get_winrow(WinHandle::null()), 0);
            assert_eq!(rs_win_get_wincol(WinHandle::null()), 0);

            // Setters should not crash
            rs_win_set_topline(WinHandle::null(), 10);
            rs_win_set_winrow(WinHandle::null(), 5);
        }
    }

    #[test]
    fn test_composite_structs() {
        unsafe {
            // Test WinDimensions with null handle
            let dims = rs_win_get_dimensions(WinHandle::null());
            assert_eq!(dims.width, 0);
            assert_eq!(dims.height, 0);
            assert_eq!(dims.winrow, 0);
            assert_eq!(dims.wincol, 0);

            // Total sizes return 0 for null
            assert_eq!(rs_win_total_height(WinHandle::null()), 0);
            assert_eq!(rs_win_total_width(WinHandle::null()), 0);
        }
    }
}
