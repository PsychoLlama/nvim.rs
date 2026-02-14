//! Format operator implementations for text formatting.
//!
//! This module provides Rust wrappers for the format operators (gq, gw)
//! and formatexpr evaluation.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::{c_int, c_void};

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to operator arguments (oparg_T*).
pub type OapHandle = *mut c_void;

/// Opaque handle to a window (win_T*).
pub type WinHandle = *mut c_void;

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    // oparg_T accessors
    fn nvim_textfmt_oap_get_cursor_start_lnum(oap: OapHandle) -> c_int;
    fn nvim_textfmt_oap_get_cursor_start_col(oap: OapHandle) -> c_int;
    fn nvim_textfmt_oap_get_start_lnum(oap: OapHandle) -> c_int;
    fn nvim_textfmt_oap_get_start_col(oap: OapHandle) -> c_int;
    fn nvim_textfmt_oap_get_end_lnum(oap: OapHandle) -> c_int;
    fn nvim_textfmt_oap_get_line_count(oap: OapHandle) -> c_int;
    fn nvim_textfmt_oap_get_is_VIsual(oap: OapHandle) -> bool;
    fn nvim_textfmt_oap_get_end_adjusted(oap: OapHandle) -> bool;

    // Cursor and window operations
    fn nvim_textfmt_set_curwin_cursor(lnum: c_int, col: c_int);
    fn nvim_textfmt_get_curwin_cursor_lnum() -> c_int;

    // Undo operations
    fn nvim_textfmt_u_save(top: c_int, bot: c_int) -> c_int;

    // Redraw operations
    fn nvim_textfmt_redraw_curbuf_later(typ: c_int);

    // Command modifier flags
    fn nvim_textfmt_get_cmdmod_lockmarks() -> bool;

    // Buffer mark operations
    fn nvim_textfmt_set_curbuf_op_start(lnum: c_int, col: c_int);
    fn nvim_textfmt_set_curbuf_op_end(lnum: c_int, col: c_int);

    // Saved cursor global
    fn nvim_textfmt_set_saved_cursor(lnum: c_int, col: c_int);
    fn nvim_textfmt_get_saved_cursor_lnum() -> c_int;
    fn nvim_textfmt_clear_saved_cursor();

    // Core formatting
    fn nvim_textfmt_format_lines(line_count: c_int, avoid_fex: bool);

    // Cursor positioning
    fn nvim_textfmt_beginline(flags: c_int);
    fn nvim_textfmt_check_cursor(win: WinHandle);
    fn nvim_textfmt_get_curwin() -> WinHandle;

    // Buffer state
    fn nvim_textfmt_get_ml_line_count() -> c_int;

    // Messages
    fn nvim_textfmt_msgmore(n: c_int);

    // Window iteration for visual mode adjustment
    fn nvim_textfmt_adjust_visual_windows(old_line_count: c_int);

}

// =============================================================================
// Constants
// =============================================================================

/// BL_WHITE | BL_FIX flags for beginline
const BL_WHITE_FIX: c_int = 0x04 | 0x08;

/// Update type: UPD_INVERTED (from drawscreen.h)
const UPD_INVERTED: c_int = 20;

/// FAIL return value
const FAIL: c_int = 0;

// =============================================================================
// Format Operator Implementations
// =============================================================================

/// Implementation of the format operator 'gq'.
///
/// # Arguments
/// * `oap` - Operator arguments
/// * `keep_cursor` - Keep cursor on same text char
///
/// # Safety
/// Accesses global state via C functions.
unsafe fn op_format_impl(oap: OapHandle, keep_cursor: bool) {
    let old_line_count = nvim_textfmt_get_ml_line_count();

    // Place the cursor where the "gq" or "gw" command was given, so that "u"
    // can put it back there.
    let cursor_start_lnum = nvim_textfmt_oap_get_cursor_start_lnum(oap);
    let cursor_start_col = nvim_textfmt_oap_get_cursor_start_col(oap);
    nvim_textfmt_set_curwin_cursor(cursor_start_lnum, cursor_start_col);

    let start_lnum = nvim_textfmt_oap_get_start_lnum(oap);
    let end_lnum = nvim_textfmt_oap_get_end_lnum(oap);

    if nvim_textfmt_u_save(start_lnum - 1, end_lnum + 1) == FAIL {
        return;
    }

    let start_col = nvim_textfmt_oap_get_start_col(oap);
    nvim_textfmt_set_curwin_cursor(start_lnum, start_col);

    if nvim_textfmt_oap_get_is_VIsual(oap) {
        // When there is no change: need to remove the Visual selection
        nvim_textfmt_redraw_curbuf_later(UPD_INVERTED);
    }

    if !nvim_textfmt_get_cmdmod_lockmarks() {
        // Set '[ mark at the start of the formatted area
        nvim_textfmt_set_curbuf_op_start(start_lnum, start_col);
    }

    // For "gw" remember the cursor position and put it back below (adjusted
    // for joined and split lines).
    if keep_cursor {
        nvim_textfmt_set_saved_cursor(cursor_start_lnum, cursor_start_col);
    }

    let line_count = nvim_textfmt_oap_get_line_count(oap);
    nvim_textfmt_format_lines(line_count, keep_cursor);

    // Leave the cursor at the first non-blank of the last formatted line.
    // If the cursor was moved one line back (e.g. with "Q}") go to the next
    // line, so "." will do the next lines.
    let ml_line_count = nvim_textfmt_get_ml_line_count();
    if nvim_textfmt_oap_get_end_adjusted(oap)
        && nvim_textfmt_get_curwin_cursor_lnum() < ml_line_count
    {
        let new_lnum = nvim_textfmt_get_curwin_cursor_lnum() + 1;
        nvim_textfmt_set_curwin_cursor(new_lnum, 0);
    }

    nvim_textfmt_beginline(BL_WHITE_FIX);

    let new_line_count = nvim_textfmt_get_ml_line_count();
    let line_diff = new_line_count - old_line_count;
    nvim_textfmt_msgmore(line_diff);

    if !nvim_textfmt_get_cmdmod_lockmarks() {
        // Put '] mark on the end of the formatted area
        let cursor_lnum = nvim_textfmt_get_curwin_cursor_lnum();
        nvim_textfmt_set_curbuf_op_end(cursor_lnum, 0);
    }

    if keep_cursor {
        let saved_lnum = nvim_textfmt_get_saved_cursor_lnum();
        nvim_textfmt_set_curwin_cursor(saved_lnum, 0);
        nvim_textfmt_clear_saved_cursor();

        // Formatting may have made the cursor position invalid
        nvim_textfmt_check_cursor(nvim_textfmt_get_curwin());
    }

    if nvim_textfmt_oap_get_is_VIsual(oap) {
        nvim_textfmt_adjust_visual_windows(line_diff);
    }
}

/// Implementation of the format operator 'gq' for when using 'formatexpr'.
///
/// # Arguments
/// * `oap` - Operator arguments
///
/// # Safety
/// Accesses global state via C functions.
unsafe fn op_formatexpr_impl(oap: OapHandle) {
    if nvim_textfmt_oap_get_is_VIsual(oap) {
        // When there is no change: need to remove the Visual selection
        nvim_textfmt_redraw_curbuf_later(UPD_INVERTED);
    }

    let start_lnum = nvim_textfmt_oap_get_start_lnum(oap);
    let line_count = nvim_textfmt_oap_get_line_count(oap);

    if crate::fex::fex_format_impl(start_lnum, line_count as std::ffi::c_long, 0) != 0 {
        // As documented: when 'formatexpr' returns non-zero fall back to
        // internal formatting.
        op_format_impl(oap, false);
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Format operator 'gq' implementation.
///
/// # Safety
/// Accesses global state via C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_op_format(oap: OapHandle, keep_cursor: c_int) {
    op_format_impl(oap, keep_cursor != 0);
}

/// Format operator 'gq' for 'formatexpr'.
///
/// # Safety
/// Accesses global state via C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_op_formatexpr(oap: OapHandle) {
    op_formatexpr_impl(oap);
}

#[cfg(test)]
mod tests {
    // Integration testing is done via the full Neovim build.
}
