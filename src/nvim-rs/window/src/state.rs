//! Window state accessors.
//!
//! This module provides FFI exports for accessing and modifying
//! window state related to layout, cursor position, view settings,
//! and other fields needed for window management.

#![allow(clippy::missing_safety_doc)]

use std::ffi::c_int;

use crate::WinHandle;

// Type aliases matching C types
type LineNr = c_int;
type ColNr = c_int;

// =============================================================================
// External C accessor functions
// =============================================================================

extern "C" {
    // Window dimension accessors
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;
    fn nvim_win_field_width(wp: WinHandle) -> c_int;
    fn nvim_win_field_height(wp: WinHandle) -> c_int;
    fn nvim_win_field_set_width(wp: WinHandle, val: c_int);
    fn nvim_win_field_set_height(wp: WinHandle, val: c_int);

    // Window position accessors
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;
    fn nvim_win_get_wincol(wp: WinHandle) -> c_int;
    fn nvim_win_set_winrow(wp: WinHandle, val: c_int);
    fn nvim_win_set_wincol(wp: WinHandle, val: c_int);

    // Cursor position accessors
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> LineNr;
    fn nvim_win_get_cursor_col(wp: WinHandle) -> ColNr;
    fn nvim_win_get_cursor_coladd(wp: WinHandle) -> ColNr;
    fn nvim_win_set_cursor_lnum(wp: WinHandle, lnum: LineNr);
    fn nvim_win_set_cursor_col(wp: WinHandle, col: ColNr);
    fn nvim_win_get_curswant(wp: WinHandle) -> ColNr;
    fn nvim_win_set_curswant(wp: WinHandle, val: ColNr);
    fn nvim_win_get_set_curswant(wp: WinHandle) -> c_int;
    fn nvim_win_set_set_curswant(wp: WinHandle, val: c_int);

    // View position accessors
    fn nvim_win_get_topline(wp: WinHandle) -> LineNr;
    fn nvim_win_set_topline(wp: WinHandle, val: LineNr);
    fn nvim_win_get_botline(wp: WinHandle) -> LineNr;
    fn nvim_win_set_botline(wp: WinHandle, val: c_int);
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;
    fn nvim_win_set_topfill(wp: WinHandle, val: c_int);
    fn nvim_win_get_leftcol(wp: WinHandle) -> ColNr;
    fn nvim_win_set_leftcol(wp: WinHandle, val: c_int);
    fn nvim_win_get_skipcol(wp: WinHandle) -> ColNr;
    fn nvim_win_set_skipcol(wp: WinHandle, val: ColNr);

    // Separator accessors
    fn nvim_win_get_hsep_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_vsep_width(wp: WinHandle) -> c_int;
    fn nvim_win_set_hsep_height(wp: WinHandle, val: c_int);
    fn nvim_win_set_vsep_width(wp: WinHandle, val: c_int);
    fn nvim_win_get_status_height(wp: WinHandle) -> c_int;
    fn nvim_win_set_status_height(wp: WinHandle, val: c_int);
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;

    // Redraw state accessors
    fn nvim_win_get_redr_status(wp: WinHandle) -> c_int;
    fn nvim_win_set_redr_status(wp: WinHandle, val: c_int);
    fn nvim_win_get_redr_type(wp: WinHandle) -> c_int;
    fn nvim_win_set_redr_type(wp: WinHandle, val: c_int);
    fn nvim_win_get_lines_valid(wp: WinHandle) -> c_int;
    fn nvim_win_set_lines_valid(wp: WinHandle, val: c_int);
    fn nvim_win_set_pos_changed(wp: WinHandle, val: c_int);
    fn nvim_win_get_viewport_invalid(wp: WinHandle) -> c_int;
    fn nvim_win_set_viewport_invalid(wp: WinHandle, val: c_int);

    // Redraw range accessors
    fn nvim_win_get_redraw_top(wp: WinHandle) -> LineNr;
    fn nvim_win_set_redraw_top(wp: WinHandle, val: LineNr);
    fn nvim_win_get_redraw_bot(wp: WinHandle) -> LineNr;
    fn nvim_win_set_redraw_bot(wp: WinHandle, val: LineNr);

    // Current line info accessors
    fn nvim_win_get_cline_row(wp: WinHandle) -> c_int;
    fn nvim_win_set_cline_row(wp: WinHandle, val: c_int);
    fn nvim_win_get_cline_height(wp: WinHandle) -> c_int;
    fn nvim_win_set_cline_height(wp: WinHandle, val: c_int);
    fn nvim_win_get_cline_folded(wp: WinHandle) -> c_int;
    fn nvim_win_set_cline_folded(wp: WinHandle, val: c_int);
    fn nvim_win_get_cursorline(wp: WinHandle) -> LineNr;

    // Valid cursor/scroll accessors
    fn nvim_win_get_valid_cursor_lnum(wp: WinHandle) -> LineNr;
    fn nvim_win_get_valid_cursor_col(wp: WinHandle) -> ColNr;
    fn nvim_win_get_valid_cursor_coladd(wp: WinHandle) -> ColNr;
    fn nvim_win_set_valid_cursor(wp: WinHandle, lnum: LineNr, col: ColNr, coladd: ColNr);
    fn nvim_win_set_valid_cursor_col(wp: WinHandle, col: ColNr);
    fn nvim_win_set_valid_cursor_coladd(wp: WinHandle, coladd: ColNr);
    fn nvim_win_get_valid_leftcol(wp: WinHandle) -> ColNr;
    fn nvim_win_set_valid_leftcol(wp: WinHandle, val: ColNr);
    fn nvim_win_get_valid_skipcol(wp: WinHandle) -> ColNr;
    fn nvim_win_set_valid_skipcol(wp: WinHandle, val: ColNr);

    // Window display info
    fn nvim_win_get_wcol(wp: WinHandle) -> c_int;
    fn nvim_win_set_wcol(wp: WinHandle, val: c_int);
    fn nvim_win_get_wrow(wp: WinHandle) -> c_int;
    fn nvim_win_get_empty_rows(wp: WinHandle) -> c_int;
    fn nvim_win_set_empty_rows(wp: WinHandle, val: c_int);
    fn nvim_win_get_endrow(wp: WinHandle) -> c_int;
    fn nvim_win_get_endcol(wp: WinHandle) -> c_int;

    // Window options
    fn nvim_win_get_p_wrap(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_rl(wp: WinHandle) -> c_int;
    fn nvim_win_set_p_rl(wp: WinHandle, val: c_int);
    fn nvim_win_get_p_bri(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_cul(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_cuc(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_culopt_flags(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_nu(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_rnu(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_list(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_diff(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_fen(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_arab(wp: WinHandle) -> c_int;

    // Misc window state
    fn nvim_win_get_virtcol(wp: WinHandle) -> ColNr;
    fn nvim_win_get_wrap_flags(wp: WinHandle) -> c_int;
    fn nvim_win_get_arg_idx(wp: WinHandle) -> c_int;
    fn nvim_win_get_arg_idx_invalid(wp: WinHandle) -> c_int;
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
    nvim_win_get_w_width(wp)
}

/// Get window height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_w_height(wp)
}

/// Get window field width (raw w_width).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_field_width(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_field_width(wp)
}

/// Get window field height (raw w_height).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_field_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_field_height(wp)
}

/// Set window field width.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_field_width(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_field_set_width(wp, val);
    }
}

/// Set window field height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_field_height(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_field_set_height(wp, val);
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
    nvim_win_get_winrow(wp)
}

/// Get window column position in screen.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_wincol(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_wincol(wp)
}

/// Set window row position.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_winrow(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_winrow(wp, val);
    }
}

/// Set window column position.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_wincol(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_wincol(wp, val);
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
    nvim_win_get_cursor_col(wp)
}

/// Get cursor column add (for virtual columns).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_cursor_coladd(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_cursor_coladd(wp)
}

/// Set cursor line number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_cursor_lnum(wp: WinHandle, lnum: LineNr) {
    if !wp.is_null() {
        nvim_win_set_cursor_lnum(wp, lnum);
    }
}

/// Set cursor column.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_cursor_col(wp: WinHandle, col: ColNr) {
    if !wp.is_null() {
        nvim_win_set_cursor_col(wp, col);
    }
}

/// Get curswant (desired cursor column).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_curswant(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_curswant(wp)
}

/// Set curswant.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_curswant(wp: WinHandle, val: ColNr) {
    if !wp.is_null() {
        nvim_win_set_curswant(wp, val);
    }
}

/// Get set_curswant flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_set_curswant(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_set_curswant(wp)
}

/// Set set_curswant flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_set_curswant(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_set_curswant(wp, val);
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
        nvim_win_set_topline(wp, val);
    }
}

// NOTE: rs_win_get_botline is defined in drawscreen crate

/// Set botline.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_botline(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_botline(wp, val);
    }
}

/// Get topfill (filler lines above topline).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_topfill(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_topfill(wp)
}

/// Set topfill.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_topfill(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_topfill(wp, val);
    }
}

/// Get leftcol (first visible column).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_leftcol(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_leftcol(wp)
}

/// Set leftcol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_leftcol(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_leftcol(wp, val);
    }
}

// NOTE: rs_win_get_skipcol is defined in drawline crate

/// Set skipcol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_skipcol(wp: WinHandle, val: ColNr) {
    if !wp.is_null() {
        nvim_win_set_skipcol(wp, val);
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
    nvim_win_get_hsep_height(wp)
}

/// Get vertical separator width.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_vsep_width(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_vsep_width(wp)
}

/// Set horizontal separator height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_hsep_height(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_hsep_height(wp, val);
    }
}

/// Set vertical separator width.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_vsep_width(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_vsep_width(wp, val);
    }
}

/// Get status line height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_status_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_status_height(wp)
}

/// Set status line height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_status_height(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_status_height(wp, val);
    }
}

/// Get winbar height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_winbar_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_winbar_height(wp)
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
    nvim_win_get_redr_status(wp)
}

/// Set redraw status flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_redr_status(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_redr_status(wp, val);
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
        nvim_win_set_pos_changed(wp, val);
    }
}

/// Get viewport_invalid flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_viewport_invalid(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_viewport_invalid(wp)
}

/// Set viewport_invalid flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_viewport_invalid(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_viewport_invalid(wp, val);
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
    nvim_win_get_cline_row(wp)
}

/// Set cline_row.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_cline_row(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_cline_row(wp, val);
    }
}

/// Get cline_height (screen height of cursor line).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_cline_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 1;
    }
    nvim_win_get_cline_height(wp)
}

/// Set cline_height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_cline_height(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_cline_height(wp, val);
    }
}

/// Get cline_folded flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_cline_folded(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_cline_folded(wp)
}

/// Set cline_folded flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_cline_folded(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_cline_folded(wp, val);
    }
}

/// Get cursorline (line number with 'cursorline').
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_cursorline(wp: WinHandle) -> LineNr {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_cursorline(wp)
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
    nvim_win_get_valid_cursor_lnum(wp)
}

/// Get valid cursor column.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_valid_cursor_col(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_valid_cursor_col(wp)
}

/// Get valid cursor coladd.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_valid_cursor_coladd(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_valid_cursor_coladd(wp)
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
        nvim_win_set_valid_cursor(wp, lnum, col, coladd);
    }
}

/// Set valid cursor column.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_valid_cursor_col(wp: WinHandle, col: ColNr) {
    if !wp.is_null() {
        nvim_win_set_valid_cursor_col(wp, col);
    }
}

/// Set valid cursor coladd.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_valid_cursor_coladd(wp: WinHandle, coladd: ColNr) {
    if !wp.is_null() {
        nvim_win_set_valid_cursor_coladd(wp, coladd);
    }
}

/// Get valid leftcol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_valid_leftcol(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_valid_leftcol(wp)
}

/// Set valid leftcol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_valid_leftcol(wp: WinHandle, val: ColNr) {
    if !wp.is_null() {
        nvim_win_set_valid_leftcol(wp, val);
    }
}

/// Get valid skipcol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_valid_skipcol(wp: WinHandle) -> ColNr {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_valid_skipcol(wp)
}

/// Set valid skipcol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_valid_skipcol(wp: WinHandle, val: ColNr) {
    if !wp.is_null() {
        nvim_win_set_valid_skipcol(wp, val);
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
    nvim_win_get_wcol(wp)
}

/// Set wcol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_wcol(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_wcol(wp, val);
    }
}

/// Get wrow (cursor row in window).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_wrow(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_wrow(wp)
}

/// Get empty_rows (filler rows at end of window).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_empty_rows(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_empty_rows(wp)
}

/// Set empty_rows.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_empty_rows(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_empty_rows(wp, val);
    }
}

/// Get endrow (last row of window content).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_endrow(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_endrow(wp)
}

/// Get endcol (last column of window content).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_endcol(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_endcol(wp)
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
    nvim_win_get_p_wrap(wp)
}

/// Get 'rightleft' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_rl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_p_rl(wp)
}

/// Set 'rightleft' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_set_p_rl(wp: WinHandle, val: c_int) {
    if !wp.is_null() {
        nvim_win_set_p_rl(wp, val);
    }
}

/// Get 'breakindent' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_bri(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_p_bri(wp)
}

/// Get 'cursorline' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_cul(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_p_cul(wp)
}

/// Get 'cursorcolumn' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_cuc(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_p_cuc(wp)
}

/// Get cursorlineopt flags.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_culopt_flags(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_p_culopt_flags(wp)
}

/// Get 'number' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_nu(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_p_nu(wp)
}

/// Get 'relativenumber' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_rnu(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_p_rnu(wp)
}

/// Get 'list' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_list(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_p_list(wp)
}

/// Get 'diff' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_diff(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_p_diff(wp)
}

/// Get 'foldenable' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_fen(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_p_fen(wp)
}

/// Get 'arabic' option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_p_arab(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_p_arab(wp)
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
    nvim_win_get_virtcol(wp)
}

/// Get wrap flags.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_wrap_flags(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_wrap_flags(wp)
}

/// Get argument index.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_arg_idx(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_arg_idx(wp)
}

/// Get argument index invalid flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_arg_idx_invalid(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_arg_idx_invalid(wp)
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
    WinCursor {
        lnum: nvim_win_get_cursor_lnum(wp),
        col: nvim_win_get_cursor_col(wp),
        coladd: nvim_win_get_cursor_coladd(wp),
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
    WinView {
        topline: nvim_win_get_topline(wp),
        botline: nvim_win_get_botline(wp),
        topfill: nvim_win_get_topfill(wp),
        leftcol: nvim_win_get_leftcol(wp),
        skipcol: nvim_win_get_skipcol(wp),
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
    WinDimensions {
        width: nvim_win_get_w_width(wp),
        height: nvim_win_get_w_height(wp),
        winrow: nvim_win_get_winrow(wp),
        wincol: nvim_win_get_wincol(wp),
        hsep_height: nvim_win_get_hsep_height(wp),
        vsep_width: nvim_win_get_vsep_width(wp),
        status_height: nvim_win_get_status_height(wp),
        winbar_height: nvim_win_get_winbar_height(wp),
    }
}

/// Get total window height including separators.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_total_height(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_w_height(wp)
        + nvim_win_get_hsep_height(wp)
        + nvim_win_get_status_height(wp)
        + nvim_win_get_winbar_height(wp)
}

/// Get total window width including separator.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_total_width(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_w_width(wp) + nvim_win_get_vsep_width(wp)
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
