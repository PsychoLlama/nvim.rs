//! Cursor movement and scrolling for Neovim
//!
//! This crate provides Rust implementations of cursor movement and scrolling
//! functions from `src/nvim/move.c`. It handles:
//! - Cursor validation and positioning
//! - Viewport and scroll management
//! - Topline/botline calculations
//! - Smooth scrolling support
//!
//! The crate uses the opaque handle pattern for window access.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(dead_code)] // Some extern declarations are pre-declared for future use
#![allow(clippy::if_not_else)] // Direct port from C logic - clearer to maintain original structure

use std::ffi::c_int;

use nvim_window::WinHandle;

// Re-export validity flags from viewport crate for convenience
pub use nvim_viewport::{
    VALID_BOTLINE, VALID_BOTLINE_AP, VALID_CHEIGHT, VALID_CROW, VALID_TOPLINE, VALID_VIRTCOL,
    VALID_WCOL, VALID_WROW,
};

// =============================================================================
// Line number and column types
// =============================================================================

/// Line number type (matches `linenr_T` in Neovim).
type LinenrT = i32;

/// Column number type (matches `colnr_T` in Neovim).
type ColnrT = i32;

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    // Window validity flags
    fn nvim_win_get_valid(wp: WinHandle) -> c_int;
    fn nvim_win_set_valid(wp: WinHandle, val: c_int);
    fn nvim_win_clear_valid_bits(wp: WinHandle, bits: c_int);

    // Cursor position
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> LinenrT;
    fn nvim_win_get_cursor_col(wp: WinHandle) -> ColnrT;
    fn nvim_win_get_cursor_coladd(wp: WinHandle) -> ColnrT;

    // Valid cursor tracking
    fn nvim_win_get_valid_cursor_lnum(wp: WinHandle) -> LinenrT;
    fn nvim_win_get_valid_cursor_col(wp: WinHandle) -> ColnrT;
    fn nvim_win_get_valid_cursor_coladd(wp: WinHandle) -> ColnrT;
    fn nvim_win_set_valid_cursor(wp: WinHandle, lnum: LinenrT, col: ColnrT, coladd: ColnrT);
    fn nvim_win_set_valid_cursor_col(wp: WinHandle, col: ColnrT);
    fn nvim_win_set_valid_cursor_coladd(wp: WinHandle, coladd: ColnrT);

    // Scroll positions
    fn nvim_win_get_leftcol(wp: WinHandle) -> ColnrT;
    fn nvim_win_get_skipcol(wp: WinHandle) -> ColnrT;
    fn nvim_win_get_valid_leftcol(wp: WinHandle) -> ColnrT;
    fn nvim_win_get_valid_skipcol(wp: WinHandle) -> ColnrT;
    fn nvim_win_set_valid_leftcol(wp: WinHandle, val: ColnrT);
    fn nvim_win_set_valid_skipcol(wp: WinHandle, val: ColnrT);

    // Viewport state
    fn nvim_win_get_viewport_invalid(wp: WinHandle) -> c_int;
    fn nvim_win_set_viewport_invalid(wp: WinHandle, val: c_int);

    // Concealment checking
    fn nvim_win_get_p_cole(wp: WinHandle) -> i64;
    fn nvim_win_is_curwin(wp: WinHandle) -> c_int;
    fn rs_conceal_cursor_line(wp: WinHandle) -> c_int;
    fn nvim_decor_conceal_line(wp: WinHandle, lnum: LinenrT, check_toggle: c_int) -> c_int;
    fn rs_changed_window_setting(wp: WinHandle);

    // Topline/botline
    fn nvim_win_get_topline(wp: WinHandle) -> LinenrT;
    fn nvim_win_get_botline(wp: WinHandle) -> LinenrT;
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;
    fn nvim_win_get_view_height(wp: WinHandle) -> c_int;

    // Cursor row/height info
    fn nvim_win_get_cline_row(wp: WinHandle) -> c_int;
    fn nvim_win_set_cline_row(wp: WinHandle, val: c_int);
    fn nvim_win_get_cline_height(wp: WinHandle) -> c_int;
    fn nvim_win_set_cline_height(wp: WinHandle, val: c_int);
    fn nvim_win_get_cline_folded(wp: WinHandle) -> c_int;
    fn nvim_win_set_cline_folded(wp: WinHandle, val: c_int);

    // Window options
    fn nvim_win_get_p_wrap(wp: WinHandle) -> c_int;

    // Curswant tracking
    fn nvim_win_get_curswant(wp: WinHandle) -> ColnrT;
    fn nvim_win_set_curswant(wp: WinHandle, val: ColnrT);
    fn nvim_win_get_set_curswant(wp: WinHandle) -> c_int;
    fn nvim_win_set_set_curswant(wp: WinHandle, val: c_int);
    fn nvim_win_get_virtcol(wp: WinHandle) -> ColnrT;

    // Current window accessor
    fn nvim_get_curwin() -> WinHandle;
}

// =============================================================================
// Validity Flags - Combination Constants
// =============================================================================

/// Flags cleared when cursor line changes
const VALID_LINE_CHANGE: c_int =
    VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CHEIGHT | VALID_CROW | VALID_TOPLINE;

/// Flags cleared when skipcol changes
const VALID_SKIPCOL_CHANGE: c_int = VALID_WROW
    | VALID_WCOL
    | VALID_VIRTCOL
    | VALID_CHEIGHT
    | VALID_CROW
    | VALID_BOTLINE
    | VALID_BOTLINE_AP;

/// Flags cleared when column changes
const VALID_COL_CHANGE: c_int = VALID_WROW | VALID_WCOL | VALID_VIRTCOL;

// =============================================================================
// Cursor Movement Detection
// =============================================================================

/// Check if the cursor has moved. Set the `w_valid` flag accordingly.
///
/// This is a key function called by many other validation functions.
/// It detects cursor movement and invalidates the appropriate flags.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_check_cursor_moved(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    let valid_cursor_lnum = nvim_win_get_valid_cursor_lnum(wp);

    if cursor_lnum != valid_cursor_lnum {
        // Line changed - major invalidation
        nvim_win_clear_valid_bits(wp, VALID_LINE_CHANGE);

        // Check for concealed line visibility toggle
        let is_curwin = nvim_win_is_curwin(wp) != 0;
        let p_cole = nvim_win_get_p_cole(wp);
        let conceal_cursor = rs_conceal_cursor_line(wp) != 0;

        if is_curwin && valid_cursor_lnum > 0 && p_cole >= 2 && !conceal_cursor {
            // Check if either old or new line has concealment
            let new_concealed = nvim_decor_conceal_line(wp, cursor_lnum - 1, 1) != 0;
            let old_concealed = nvim_decor_conceal_line(wp, valid_cursor_lnum - 1, 1) != 0;
            if new_concealed || old_concealed {
                rs_changed_window_setting(wp);
            }
        }

        // Update tracking state
        let cursor_col = nvim_win_get_cursor_col(wp);
        let cursor_coladd = nvim_win_get_cursor_coladd(wp);
        let leftcol = nvim_win_get_leftcol(wp);
        let skipcol = nvim_win_get_skipcol(wp);

        nvim_win_set_valid_cursor(wp, cursor_lnum, cursor_col, cursor_coladd);
        nvim_win_set_valid_leftcol(wp, leftcol);
        nvim_win_set_valid_skipcol(wp, skipcol);
        nvim_win_set_viewport_invalid(wp, 1);
    } else {
        let skipcol = nvim_win_get_skipcol(wp);
        let valid_skipcol = nvim_win_get_valid_skipcol(wp);

        if skipcol != valid_skipcol {
            // Skipcol changed
            nvim_win_clear_valid_bits(wp, VALID_SKIPCOL_CHANGE);

            let cursor_col = nvim_win_get_cursor_col(wp);
            let cursor_coladd = nvim_win_get_cursor_coladd(wp);
            let leftcol = nvim_win_get_leftcol(wp);

            nvim_win_set_valid_cursor(wp, cursor_lnum, cursor_col, cursor_coladd);
            nvim_win_set_valid_leftcol(wp, leftcol);
            nvim_win_set_valid_skipcol(wp, skipcol);
        } else {
            let cursor_col = nvim_win_get_cursor_col(wp);
            let cursor_coladd = nvim_win_get_cursor_coladd(wp);
            let leftcol = nvim_win_get_leftcol(wp);
            let valid_cursor_col = nvim_win_get_valid_cursor_col(wp);
            let valid_cursor_coladd = nvim_win_get_valid_cursor_coladd(wp);
            let valid_leftcol = nvim_win_get_valid_leftcol(wp);

            if cursor_col != valid_cursor_col
                || leftcol != valid_leftcol
                || cursor_coladd != valid_cursor_coladd
            {
                // Column changed
                nvim_win_clear_valid_bits(wp, VALID_COL_CHANGE);
                nvim_win_set_valid_cursor_col(wp, cursor_col);
                nvim_win_set_valid_leftcol(wp, leftcol);
                nvim_win_set_valid_cursor_coladd(wp, cursor_coladd);
                nvim_win_set_viewport_invalid(wp, 1);
            }
        }
    }
}

// =============================================================================
// Cursor Validity Checks
// =============================================================================

/// Return true if `wp->w_wrow` and `wp->w_wcol` are valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_valid(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    rs_check_cursor_moved(wp);
    let valid = nvim_win_get_valid(wp);
    c_int::from((valid & (VALID_WROW | VALID_WCOL)) == (VALID_WROW | VALID_WCOL))
}

// =============================================================================
// Curswant Management
// =============================================================================

/// Update `w_curswant` unconditionally.
///
/// Sets curswant to the current virtual column.
///
/// # Safety
/// Accesses curwin global.
#[no_mangle]
pub unsafe extern "C" fn rs_update_curswant_force() {
    let wp = nvim_get_curwin();
    if wp.is_null() {
        return;
    }

    // Note: validate_virtcol must be called before this.
    // The caller is responsible for ensuring virtcol is valid.
    let virtcol = nvim_win_get_virtcol(wp);
    nvim_win_set_curswant(wp, virtcol);
    nvim_win_set_set_curswant(wp, 0);
}

/// Update `w_curswant` if `w_set_curswant` is set.
///
/// # Safety
/// Accesses curwin global.
#[no_mangle]
pub unsafe extern "C" fn rs_update_curswant() {
    let wp = nvim_get_curwin();
    if wp.is_null() {
        return;
    }

    if nvim_win_get_set_curswant(wp) != 0 {
        rs_update_curswant_force();
    }
}

// =============================================================================
// Validation State Queries
// =============================================================================

/// Check if the window row (`w_wrow`) is valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_wrow_valid(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    c_int::from((nvim_win_get_valid(wp) & VALID_WROW) != 0)
}

/// Check if the window column (`w_wcol`) is valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_wcol_valid(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    c_int::from((nvim_win_get_valid(wp) & VALID_WCOL) != 0)
}

/// Check if the virtual column (`w_virtcol`) is valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_virtcol_valid(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    c_int::from((nvim_win_get_valid(wp) & VALID_VIRTCOL) != 0)
}

/// Check if the cursor height (`w_cline_height`) is valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cheight_valid(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    c_int::from((nvim_win_get_valid(wp) & VALID_CHEIGHT) != 0)
}

/// Check if the cursor row (`w_cline_row`) is valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_crow_valid(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    c_int::from((nvim_win_get_valid(wp) & VALID_CROW) != 0)
}

/// Check if the topline is valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_topline_valid(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    c_int::from((nvim_win_get_valid(wp) & VALID_TOPLINE) != 0)
}

/// Check if the botline is valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_botline_valid(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    c_int::from((nvim_win_get_valid(wp) & VALID_BOTLINE) != 0)
}

// =============================================================================
// Validation Flag Manipulation
// =============================================================================

/// Mark the topline as valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_topline_valid(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    let valid = nvim_win_get_valid(wp);
    nvim_win_set_valid(wp, valid | VALID_TOPLINE);
}

/// Mark the virtual column as valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_virtcol_valid(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    let valid = nvim_win_get_valid(wp);
    nvim_win_set_valid(wp, valid | VALID_VIRTCOL);
}

/// Mark the cursor row as valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_crow_valid(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    let valid = nvim_win_get_valid(wp);
    nvim_win_set_valid(wp, valid | VALID_CROW);
}

/// Mark the cursor height as valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_cheight_valid(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    let valid = nvim_win_get_valid(wp);
    nvim_win_set_valid(wp, valid | VALID_CHEIGHT);
}

/// Mark both cursor row and height as valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_crow_cheight_valid(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    let valid = nvim_win_get_valid(wp);
    nvim_win_set_valid(wp, valid | VALID_CROW | VALID_CHEIGHT);
}

/// Mark window cursor position (row and column) as valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_wcol_wrow_valid(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    let valid = nvim_win_get_valid(wp);
    nvim_win_set_valid(wp, valid | VALID_WCOL | VALID_WROW);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validity_constants() {
        // Ensure our combined constants are correct
        assert_eq!(VALID_WROW, 0x01);
        assert_eq!(VALID_WCOL, 0x02);
        assert_eq!(VALID_VIRTCOL, 0x04);
        assert_eq!(VALID_CHEIGHT, 0x08);
        assert_eq!(VALID_CROW, 0x10);
        assert_eq!(VALID_BOTLINE, 0x20);
        assert_eq!(VALID_BOTLINE_AP, 0x40);
        assert_eq!(VALID_TOPLINE, 0x80);
    }

    #[test]
    fn test_valid_line_change_flags() {
        // VALID_LINE_CHANGE should include all position-related flags
        assert_ne!(VALID_LINE_CHANGE & VALID_WROW, 0);
        assert_ne!(VALID_LINE_CHANGE & VALID_WCOL, 0);
        assert_ne!(VALID_LINE_CHANGE & VALID_VIRTCOL, 0);
        assert_ne!(VALID_LINE_CHANGE & VALID_CHEIGHT, 0);
        assert_ne!(VALID_LINE_CHANGE & VALID_CROW, 0);
        assert_ne!(VALID_LINE_CHANGE & VALID_TOPLINE, 0);
        // But not botline
        assert_eq!(VALID_LINE_CHANGE & VALID_BOTLINE, 0);
    }

    #[test]
    fn test_valid_col_change_flags() {
        // VALID_COL_CHANGE is a subset of VALID_LINE_CHANGE
        assert_ne!(VALID_COL_CHANGE & VALID_WROW, 0);
        assert_ne!(VALID_COL_CHANGE & VALID_WCOL, 0);
        assert_ne!(VALID_COL_CHANGE & VALID_VIRTCOL, 0);
        // But not row-related flags
        assert_eq!(VALID_COL_CHANGE & VALID_CROW, 0);
        assert_eq!(VALID_COL_CHANGE & VALID_CHEIGHT, 0);
    }
}
