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
// LineOff Structure for Scroll Calculations
// =============================================================================

/// Line offset structure for scroll calculations.
///
/// This matches the C `lineoff_T` structure and is used to track
/// position during scroll operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LineOff {
    /// Line number
    pub lnum: LinenrT,
    /// Filler lines
    pub fill: c_int,
    /// Height of added line
    pub height: c_int,
}

impl LineOff {
    /// Create a new line offset for a line number.
    #[inline]
    #[must_use]
    pub const fn new(lnum: LinenrT) -> Self {
        Self {
            lnum,
            fill: 0,
            height: 0,
        }
    }
}

// =============================================================================
// Additional C Accessor Functions for Scrolling
// =============================================================================

extern "C" {
    // Filler lines
    fn nvim_win_get_fill(wp: WinHandle, lnum: LinenrT) -> c_int;

    // Folding
    fn nvim_hasFolding(
        wp: WinHandle,
        lnum: LinenrT,
        first: *mut LinenrT,
        last: *mut LinenrT,
    ) -> c_int;

    // Buffer info
    fn nvim_win_buf_line_count(wp: WinHandle) -> LinenrT;

    // Physical lines
    fn nvim_plines_win_nofill(wp: WinHandle, lnum: LinenrT, winheight: c_int) -> c_int;

    // Scrolloff value
    fn rs_get_scrolloff_value(wp: WinHandle) -> c_int;

    // Lines concealed detection
    fn nvim_win_lines_concealed(wp: WinHandle) -> c_int;
}

/// Maximum column value (for scroll calculation end markers).
const MAXCOL: c_int = i32::MAX;

// =============================================================================
// Topline/Botline Navigation
// =============================================================================

/// Move one line up from the current position in `loff`.
///
/// This adds either a filler line or moves to the previous line.
/// The height of the added line is stored in `loff->height`.
/// Lines above line 1 have height `MAXCOL` (impossibly high).
///
/// # Safety
/// `wp` must be a valid window handle.
/// `loff` must be a valid pointer to a `LineOff` struct.
#[no_mangle]
pub unsafe extern "C" fn rs_topline_back_winheight(
    wp: WinHandle,
    loff: *mut LineOff,
    winheight: c_int,
) {
    if wp.is_null() || loff.is_null() {
        return;
    }
    let loff = &mut *loff;

    let fill = nvim_win_get_fill(wp, loff.lnum);
    if loff.fill < fill {
        // Add a filler line
        loff.fill += 1;
        loff.height = 1;
    } else {
        loff.lnum -= 1;
        loff.fill = 0;
        if loff.lnum < 1 {
            loff.height = MAXCOL;
        } else {
            let mut first_lnum = loff.lnum;
            if nvim_hasFolding(
                wp,
                loff.lnum,
                std::ptr::addr_of_mut!(first_lnum),
                std::ptr::null_mut(),
            ) != 0
            {
                // Add a closed fold unless concealed
                loff.lnum = first_lnum;
                loff.height = i32::from(nvim_decor_conceal_line(wp, loff.lnum - 1, 0) == 0);
            } else {
                loff.height = nvim_plines_win_nofill(wp, loff.lnum, winheight);
            }
        }
    }
}

/// Move one line up from the current position (with window height limit).
///
/// # Safety
/// `wp` must be a valid window handle.
/// `loff` must be a valid pointer to a `LineOff` struct.
#[no_mangle]
pub unsafe extern "C" fn rs_topline_back(wp: WinHandle, loff: *mut LineOff) {
    rs_topline_back_winheight(wp, loff, 1);
}

/// Move one line down from the current position in `loff`.
///
/// This adds either a filler line or moves to the next line.
/// The height of the added line is stored in `loff->height`.
/// Lines below the last line have height `MAXCOL` (impossibly high).
///
/// # Safety
/// `wp` must be a valid window handle.
/// `loff` must be a valid pointer to a `LineOff` struct.
#[no_mangle]
pub unsafe extern "C" fn rs_botline_forw(wp: WinHandle, loff: *mut LineOff) {
    if wp.is_null() || loff.is_null() {
        return;
    }
    let loff = &mut *loff;

    let fill = nvim_win_get_fill(wp, loff.lnum + 1);
    if loff.fill < fill {
        // Add a filler line
        loff.fill += 1;
        loff.height = 1;
    } else {
        loff.lnum += 1;
        loff.fill = 0;
        let line_count = nvim_win_buf_line_count(wp);
        if loff.lnum > line_count {
            loff.height = MAXCOL;
        } else {
            let mut last_lnum = loff.lnum;
            if nvim_hasFolding(
                wp,
                loff.lnum,
                std::ptr::null_mut(),
                std::ptr::addr_of_mut!(last_lnum),
            ) != 0
            {
                // Add a closed fold unless concealed
                loff.lnum = last_lnum;
                loff.height = i32::from(nvim_decor_conceal_line(wp, loff.lnum - 1, 0) == 0);
            } else {
                loff.height = nvim_plines_win_nofill(wp, loff.lnum, 1);
            }
        }
    }
}

// =============================================================================
// Topline/Botline State Management
// =============================================================================

/// Mark the botline as valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_botline_valid(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    let valid = nvim_win_get_valid(wp);
    nvim_win_set_valid(wp, valid | VALID_BOTLINE);
}

/// Mark botline as approximately valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_botline_approximate(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    let valid = nvim_win_get_valid(wp);
    nvim_win_set_valid(wp, valid | VALID_BOTLINE_AP);
}

/// Clear the topline validity flag.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_invalidate_topline(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    nvim_win_clear_valid_bits(wp, VALID_TOPLINE);
}

/// Clear botline and topline validity flags.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_invalidate_botline_topline(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    nvim_win_clear_valid_bits(wp, VALID_BOTLINE | VALID_BOTLINE_AP | VALID_TOPLINE);
}

// =============================================================================
// Scroll Position Queries
// =============================================================================

/// Get the number of filler lines above a line in the window.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_fill(wp: WinHandle, lnum: LinenrT) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_fill(wp, lnum)
}

/// Check if cursor is at or above topline.
///
/// Returns true if the cursor line is less than or equal to topline.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_at_or_above_topline(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    let topline = nvim_win_get_topline(wp);
    c_int::from(cursor_lnum <= topline)
}

/// Check if cursor is at or below botline.
///
/// Returns true if the cursor line is greater than or equal to botline - 1.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_at_or_below_botline(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    let botline = nvim_win_get_botline(wp);
    c_int::from(cursor_lnum >= botline - 1)
}

// =============================================================================
// Scroll Offset Checking
// =============================================================================

/// Check if there are not 'scrolloff' lines above the cursor.
///
/// Returns true when there are not 'scrolloff' lines above the cursor,
/// which means the view might need to scroll.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_check_top_offset(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let so = rs_get_scrolloff_value(wp);
    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    let topline = nvim_win_get_topline(wp);
    let topfill = nvim_win_get_topfill(wp);

    if cursor_lnum < topline + so || nvim_win_lines_concealed(wp) != 0 {
        let mut loff = LineOff {
            lnum: cursor_lnum,
            fill: 0,
            height: 0,
        };
        let mut n = topfill; // always have this context

        // Count the visible screen lines above the cursor line.
        while n < so {
            rs_topline_back(wp, std::ptr::addr_of_mut!(loff));
            // Stop when included a line above the window.
            if loff.lnum < topline || (loff.lnum == topline && loff.fill > 0) {
                break;
            }
            n += loff.height;
        }

        if n < so {
            return 1;
        }
    }

    0
}

// =============================================================================
// Skipcol Management
// =============================================================================

/// Redraw type constants (from drawscreen.h).
#[allow(dead_code)]
mod upd {
    use std::ffi::c_int;
    pub const VALID: c_int = 10;
    pub const SOME_VALID: c_int = 35;
    pub const NOT_VALID: c_int = 40;
}

extern "C" {
    // Skipcol setter
    fn nvim_win_set_skipcol(wp: WinHandle, val: ColnrT);

    // Redraw functions
    fn nvim_redraw_later(wp: WinHandle, type_: c_int);
}

/// Set skipcol to zero and redraw later if needed.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_reset_skipcol(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    let skipcol = nvim_win_get_skipcol(wp);
    if skipcol == 0 {
        return;
    }

    nvim_win_set_skipcol(wp, 0);

    // Should use the least expensive way that displays all that changed.
    // UPD_NOT_VALID is too expensive, UPD_REDRAW_TOP does not redraw
    // enough when the top line gets another screen line.
    nvim_redraw_later(wp, upd::SOME_VALID);
}

// =============================================================================
// Cursor Column Operations
// =============================================================================

extern "C" {
    // Column offsets (from plines crate)
    fn rs_win_col_off(wp: WinHandle) -> c_int;
    fn rs_win_col_off2(wp: WinHandle) -> c_int;

    // Window accessors for column operations
    fn nvim_win_get_wcol(wp: WinHandle) -> c_int;
    fn nvim_win_set_wcol(wp: WinHandle, val: c_int);
    fn nvim_win_get_view_width(wp: WinHandle) -> c_int;
}

/// Compute `w_wcol` from `w_virtcol`.
///
/// This performs the column computation part of `validate_cursor_col`.
/// Note: `validate_virtcol` must be called first to ensure `w_virtcol` is valid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_compute_wcol(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    // If already valid, nothing to do
    let valid = nvim_win_get_valid(wp);
    if (valid & VALID_WCOL) != 0 {
        return;
    }

    let virtcol = nvim_win_get_virtcol(wp);
    let off = rs_win_col_off(wp);
    let mut col = virtcol + off;
    let view_width = nvim_win_get_view_width(wp);
    let width = view_width - off + rs_win_col_off2(wp);
    let p_wrap = nvim_win_get_p_wrap(wp);
    let leftcol = nvim_win_get_leftcol(wp);

    // long line wrapping, adjust
    if p_wrap != 0 && col >= view_width && width > 0 {
        // use same formula as what is used in curs_columns()
        col -= ((col - view_width) / width + 1) * width;
    }

    if col > leftcol {
        col -= leftcol;
    } else {
        col = 0;
    }

    nvim_win_set_wcol(wp, col);
    nvim_win_set_valid(wp, valid | VALID_WCOL);
}

/// Get the computed window column.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_get_wcol(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_wcol(wp)
}

/// Get the window column offset (number/sign column width).
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_get_col_off(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    rs_win_col_off(wp)
}

/// Get the additional column offset for wrapped lines.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_get_col_off2(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    rs_win_col_off2(wp)
}

/// Compute the wrapping width for a window.
///
/// Returns the effective width for line wrapping calculations.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_compute_wrap_width(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    let view_width = nvim_win_get_view_width(wp);
    let off = rs_win_col_off(wp);
    view_width - off + rs_win_col_off2(wp)
}

// =============================================================================
// Topfill Management
// =============================================================================

extern "C" {
    // Topline/topfill setters
    fn nvim_win_set_topline(wp: WinHandle, val: LinenrT);
    fn nvim_win_set_topfill(wp: WinHandle, val: c_int);

    // Float window management
    fn nvim_win_check_anchored_floats(wp: WinHandle);

    // Wrow accessors
    fn nvim_win_get_wrow(wp: WinHandle) -> c_int;
    fn nvim_win_set_wrow(wp: WinHandle, val: c_int);

    // Smoothscroll option accessor
    fn nvim_win_get_p_sms(wp: WinHandle) -> c_int;

    // Botline setter
    fn nvim_win_set_botline(wp: WinHandle, val: LinenrT);

    // Validation and cursor wrappers
    fn nvim_cursor_correct(wp: WinHandle);
    fn nvim_cursor_correct_sms(wp: WinHandle);
    fn nvim_validate_cursor_win(wp: WinHandle);
    fn nvim_validate_virtcol(wp: WinHandle);
    fn nvim_validate_cheight(wp: WinHandle);
    fn nvim_check_topfill(wp: WinHandle, down: c_int);
    fn nvim_invalidate_botline(wp: WinHandle);

    // Cursor movement (operates on curwin)
    fn nvim_scroll_cursor_up(n: i64, upd_topline: c_int) -> c_int;
    fn nvim_scroll_cursor_down(n: c_int, upd_topline: c_int) -> c_int;

    // Plines wrappers
    fn nvim_linetabsize_eol(wp: WinHandle, lnum: LinenrT) -> c_int;
    fn nvim_plines_win(wp: WinHandle, lnum: LinenrT, limit: c_int) -> c_int;
    fn nvim_win_may_fill(wp: WinHandle) -> c_int;

    // Column offsets (C versions)
    fn nvim_win_col_off(wp: WinHandle) -> c_int;
    fn nvim_win_col_off2(wp: WinHandle) -> c_int;

    // Fold/coladvance
    fn rs_foldAdjustCursor(wp: WinHandle);
    fn rs_coladvance(wp: WinHandle, wcol: ColnrT) -> c_int;

    // Cursor position setters
    fn nvim_win_set_cursor_lnum(wp: WinHandle, lnum: LinenrT);
    fn nvim_win_set_cursor_col(wp: WinHandle, col: ColnrT);
}

/// Ensure topfill doesn't use too many window lines.
///
/// If the filler lines plus the first line would exceed the window height,
/// adjust topfill or topline accordingly.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_check_topfill(wp: WinHandle, down: c_int) {
    if wp.is_null() {
        return;
    }

    let topfill = nvim_win_get_topfill(wp);
    if topfill > 0 {
        let topline = nvim_win_get_topline(wp);
        let n = nvim_plines_win_nofill(wp, topline, 1);
        let view_height = nvim_win_get_view_height(wp);

        if topfill + n > view_height {
            if down != 0 && topline > 1 {
                nvim_win_set_topline(wp, topline - 1);
                nvim_win_set_topfill(wp, 0);
            } else {
                let new_topfill = (view_height - n).max(0);
                nvim_win_set_topfill(wp, new_topfill);
            }
        }
    }

    nvim_win_check_anchored_floats(wp);
}

/// Get the topfill value for a window.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_get_topfill(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_topfill(wp)
}

/// Set the topfill value for a window.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_set_topfill(wp: WinHandle, val: c_int) {
    if wp.is_null() {
        return;
    }
    nvim_win_set_topfill(wp, val);
}

// =============================================================================
// Scroll Functions
// =============================================================================

/// Redraw type constants (from drawscreen.h).
mod upd_scroll {
    use std::ffi::c_int;
    pub const NOT_VALID: c_int = 40;
}

/// Flags cleared when cursor moves during scroll
const SCROLL_CURSOR_MOVED_BITS: c_int =
    VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW | VALID_VIRTCOL;

/// Scroll a window down by `line_count` logical lines. "CTRL-Y"
///
/// # Arguments
/// * `wp` - Window handle
/// * `line_count` - Number of lines to scroll
/// * `byfold` - If true, count a closed fold as one line
///
/// # Returns
/// True if cursor moved as a result.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_scrolldown(wp: WinHandle, line_count: LinenrT, byfold: c_int) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let byfold = byfold != 0;
    let mut done = 0; // total # of physical lines done
    let mut width1 = 0;
    let mut width2 = 0;
    let p_wrap = nvim_win_get_p_wrap(wp) != 0;
    let p_sms = nvim_win_get_p_sms(wp) != 0;
    let do_sms = p_wrap && p_sms;
    let view_width = nvim_win_get_view_width(wp);
    let view_height = nvim_win_get_view_height(wp);

    if do_sms {
        width1 = view_width - nvim_win_col_off(wp);
        width2 = width1 + nvim_win_col_off2(wp);
    }

    // Make sure w_topline is at the first of a sequence of folded lines.
    let mut topline = nvim_win_get_topline(wp);
    let mut first_lnum = topline;
    if nvim_hasFolding(
        wp,
        topline,
        std::ptr::addr_of_mut!(first_lnum),
        std::ptr::null_mut(),
    ) != 0
    {
        topline = first_lnum;
        nvim_win_set_topline(wp, topline);
    }

    nvim_validate_cursor_win(wp); // w_wrow needs to be valid

    let mut todo = line_count;
    while todo > 0 {
        let topfill = nvim_win_get_topfill(wp);
        topline = nvim_win_get_topline(wp);
        let skipcol = nvim_win_get_skipcol(wp);

        let fill = nvim_win_get_fill(wp, topline);
        let can_fill = topfill < view_height - 1 && topfill < fill;

        // break when at the very top
        if topline == 1 && !can_fill && (!do_sms || skipcol < width1) {
            break;
        }

        if do_sms && skipcol >= width1 {
            // scroll a screen line down
            if skipcol >= width1 + width2 {
                nvim_win_set_skipcol(wp, skipcol - width2);
            } else {
                nvim_win_set_skipcol(wp, skipcol - width1);
            }
            nvim_redraw_later(wp, upd_scroll::NOT_VALID);
            done += 1;
        } else if can_fill {
            nvim_win_set_topfill(wp, topfill + 1);
            done += 1;
        } else {
            // scroll a text line down
            topline -= 1;
            nvim_win_set_topline(wp, topline);
            nvim_win_set_skipcol(wp, 0);
            nvim_win_set_topfill(wp, 0);

            // A sequence of folded lines only counts for one logical line
            let mut first: LinenrT = 0;
            if nvim_hasFolding(
                wp,
                topline,
                std::ptr::addr_of_mut!(first),
                std::ptr::null_mut(),
            ) != 0
            {
                done += i32::from(nvim_decor_conceal_line(wp, first - 1, 0) == 0);
                if !byfold {
                    todo -= topline - first - 1;
                }
                let botline = nvim_win_get_botline(wp);
                nvim_win_set_botline(wp, botline - (topline - first));
                topline = first;
                nvim_win_set_topline(wp, topline);
            } else if nvim_decor_conceal_line(wp, topline - 1, 0) != 0 {
                todo += 1;
            } else if do_sms {
                let mut size = nvim_linetabsize_eol(wp, topline);
                if size > width1 {
                    nvim_win_set_skipcol(wp, width1);
                    size -= width1;
                    nvim_redraw_later(wp, upd_scroll::NOT_VALID);
                }
                while size > width2 {
                    nvim_win_set_skipcol(wp, nvim_win_get_skipcol(wp) + width2);
                    size -= width2;
                }
                done += 1;
            } else {
                done += nvim_plines_win_nofill(wp, topline, 1);
            }
        }

        // approximate w_botline
        let botline = nvim_win_get_botline(wp);
        nvim_win_set_botline(wp, botline - 1);
        nvim_invalidate_botline(wp);

        todo -= 1;
    }

    // Adjust for concealed lines above w_topline
    topline = nvim_win_get_topline(wp);
    while topline > 1 && nvim_decor_conceal_line(wp, topline - 2, 0) != 0 {
        topline -= 1;
        nvim_win_set_topline(wp, topline);
        let mut first: LinenrT = 0;
        if nvim_hasFolding(
            wp,
            topline,
            std::ptr::addr_of_mut!(first),
            std::ptr::null_mut(),
        ) != 0
        {
            topline = first;
            nvim_win_set_topline(wp, topline);
        }
    }

    // keep w_wrow updated
    let wrow = nvim_win_get_wrow(wp);
    nvim_win_set_wrow(wp, wrow + done);

    // keep w_cline_row updated
    let cline_row = nvim_win_get_cline_row(wp);
    nvim_win_set_cline_row(wp, cline_row + done);

    topline = nvim_win_get_topline(wp);
    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    if cursor_lnum == topline {
        nvim_win_set_cline_row(wp, 0);
    }

    nvim_check_topfill(wp, 1);

    // Compute the row number of the last row of the cursor line
    // and move the cursor onto the displayed part of the window.
    let mut wrow = nvim_win_get_wrow(wp);
    if p_wrap && view_width != 0 {
        nvim_validate_virtcol(wp);
        nvim_validate_cheight(wp);
        let cline_height = nvim_win_get_cline_height(wp);
        let virtcol = nvim_win_get_virtcol(wp);
        wrow += cline_height - 1 - virtcol / view_width;
    }

    let mut moved = false;
    let mut cursor_lnum = nvim_win_get_cursor_lnum(wp);
    while wrow >= view_height && cursor_lnum > 1 {
        let mut first: LinenrT = 0;
        if nvim_hasFolding(
            wp,
            cursor_lnum,
            std::ptr::addr_of_mut!(first),
            std::ptr::null_mut(),
        ) != 0
        {
            wrow -= i32::from(nvim_decor_conceal_line(wp, cursor_lnum - 1, 0) == 0);
            cursor_lnum = (first - 1).max(1);
        } else {
            wrow -= nvim_plines_win(wp, cursor_lnum, 1);
            cursor_lnum -= 1;
        }
        nvim_win_set_cursor_lnum(wp, cursor_lnum);
        nvim_win_clear_valid_bits(wp, SCROLL_CURSOR_MOVED_BITS);
        moved = true;
    }

    if moved {
        // Move cursor to first line of closed fold.
        rs_foldAdjustCursor(wp);
        let curswant = nvim_win_get_curswant(wp);
        rs_coladvance(wp, curswant);
    }

    topline = nvim_win_get_topline(wp);
    cursor_lnum = nvim_win_get_cursor_lnum(wp);
    if cursor_lnum < topline {
        nvim_win_set_cursor_lnum(wp, topline);
    }

    c_int::from(moved)
}

/// Scroll a window up by `line_count` logical lines. "CTRL-E"
///
/// # Arguments
/// * `wp` - Window handle
/// * `line_count` - Number of lines to scroll
/// * `byfold` - If true, count a closed fold as one line
///
/// # Returns
/// True if topline or botline changed.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_scrollup(wp: WinHandle, line_count: LinenrT, byfold: c_int) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let byfold = byfold != 0;
    let orig_topline = nvim_win_get_topline(wp);
    let orig_botline = nvim_win_get_botline(wp);

    let p_wrap = nvim_win_get_p_wrap(wp) != 0;
    let p_sms = nvim_win_get_p_sms(wp) != 0;
    let do_sms = p_wrap && p_sms;
    let view_width = nvim_win_get_view_width(wp);

    if do_sms || (byfold && nvim_win_lines_concealed(wp) != 0) || nvim_win_may_fill(wp) != 0 {
        let width1 = view_width - nvim_win_col_off(wp);
        let width2 = width1 + nvim_win_col_off2(wp);
        let prev_skipcol = nvim_win_get_skipcol(wp);

        let mut size = if do_sms {
            nvim_linetabsize_eol(wp, nvim_win_get_topline(wp))
        } else {
            0
        };

        // diff mode: first consume "topfill"
        // 'smoothscroll': increase "w_skipcol" until it goes over the end of
        // the line, then advance to the next line.
        // folding: count each sequence of folded lines as one logical line.
        let mut todo = line_count;
        while todo > 0 {
            let topline = nvim_win_get_topline(wp);
            todo += nvim_decor_conceal_line(wp, topline - 1, 0);

            let topfill = nvim_win_get_topfill(wp);
            if topfill > 0 {
                nvim_win_set_topfill(wp, topfill - 1);
            } else {
                let mut lnum = topline;
                if byfold {
                    // for a closed fold: go to the last line in the fold
                    nvim_hasFolding(wp, lnum, std::ptr::null_mut(), std::ptr::addr_of_mut!(lnum));
                }

                if lnum == topline && do_sms {
                    // 'smoothscroll': increase "w_skipcol" until it goes over
                    // the end of the line, then advance to the next line.
                    let skipcol = nvim_win_get_skipcol(wp);
                    let add = if skipcol > 0 { width2 } else { width1 };
                    nvim_win_set_skipcol(wp, skipcol + add);

                    let new_skipcol = nvim_win_get_skipcol(wp);
                    if new_skipcol >= size {
                        let line_count = nvim_win_buf_line_count(wp);
                        if lnum == line_count {
                            // at the last screen line, can't scroll further
                            nvim_win_set_skipcol(wp, new_skipcol - add);
                            break;
                        }
                        lnum += 1;
                    }
                } else {
                    let line_count = nvim_win_buf_line_count(wp);
                    if lnum >= line_count {
                        break;
                    }
                    lnum += 1;
                }

                if lnum > topline {
                    // approximate w_botline
                    let botline = nvim_win_get_botline(wp);
                    nvim_win_set_botline(wp, botline + (lnum - topline));
                    nvim_win_set_topline(wp, lnum);
                    let fill = nvim_win_get_fill(wp, lnum);
                    nvim_win_set_topfill(wp, fill);
                    nvim_win_set_skipcol(wp, 0);
                    if todo > 1 && do_sms {
                        size = nvim_linetabsize_eol(wp, nvim_win_get_topline(wp));
                    }
                }
            }

            todo -= 1;
        }

        let skipcol = nvim_win_get_skipcol(wp);
        if prev_skipcol > 0 || skipcol > 0 {
            // need to redraw more, because wl_size of the (new) topline may
            // now be invalid
            nvim_redraw_later(wp, upd_scroll::NOT_VALID);
        }
    } else {
        let topline = nvim_win_get_topline(wp);
        let botline = nvim_win_get_botline(wp);
        nvim_win_set_topline(wp, topline + line_count);
        nvim_win_set_botline(wp, botline + line_count); // approximate w_botline
    }

    // Clamp topline and botline to buffer bounds
    let line_count = nvim_win_buf_line_count(wp);
    let topline = nvim_win_get_topline(wp);
    if topline > line_count {
        nvim_win_set_topline(wp, line_count);
    }
    let botline = nvim_win_get_botline(wp);
    if botline > line_count + 1 {
        nvim_win_set_botline(wp, line_count + 1);
    }

    nvim_check_topfill(wp, 0);

    // Make sure w_topline is at the first of a sequence of folded lines.
    let topline = nvim_win_get_topline(wp);
    let mut first: LinenrT = 0;
    if nvim_hasFolding(
        wp,
        topline,
        std::ptr::addr_of_mut!(first),
        std::ptr::null_mut(),
    ) != 0
    {
        nvim_win_set_topline(wp, first);
    }

    nvim_win_clear_valid_bits(wp, VALID_WROW | VALID_CROW | VALID_BOTLINE);

    let topline = nvim_win_get_topline(wp);
    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    if cursor_lnum < topline {
        nvim_win_set_cursor_lnum(wp, topline);
        nvim_win_clear_valid_bits(wp, SCROLL_CURSOR_MOVED_BITS);
        let curswant = nvim_win_get_curswant(wp);
        rs_coladvance(wp, curswant);
    }

    let new_topline = nvim_win_get_topline(wp);
    let new_botline = nvim_win_get_botline(wp);
    let moved = orig_topline != new_topline || orig_botline != new_botline;

    c_int::from(moved)
}

/// Redraw type constant for VALID redraw.
mod upd_valid {
    use std::ffi::c_int;
    pub const VALID: c_int = 10;
}

/// Scroll `count` lines up or down, and redraw.
///
/// This is the main entry point for CTRL-E and CTRL-Y scrolling.
///
/// # Arguments
/// * `up` - If non-zero, scroll up (CTRL-E); otherwise scroll down (CTRL-Y)
/// * `count` - Number of lines to scroll
///
/// # Safety
/// Accesses curwin global and modifies window state.
#[no_mangle]
pub unsafe extern "C" fn rs_scroll_redraw(up: c_int, count: LinenrT) {
    let wp = nvim_get_curwin();
    if wp.is_null() {
        return;
    }

    let up = up != 0;
    let prev_topline = nvim_win_get_topline(wp);
    let prev_skipcol = nvim_win_get_skipcol(wp);
    let prev_topfill = nvim_win_get_topfill(wp);
    let prev_lnum = nvim_win_get_cursor_lnum(wp);

    let moved = if up {
        rs_scrollup(wp, count, 1) != 0
    } else {
        rs_scrolldown(wp, count, 1) != 0
    };

    if rs_get_scrolloff_value(wp) > 0 {
        // Adjust the cursor position for 'scrolloff'. Mark w_topline as
        // valid, otherwise the screen jumps back at the end of the file.
        nvim_cursor_correct(wp);
        rs_check_cursor_moved(wp);
        let valid = nvim_win_get_valid(wp);
        nvim_win_set_valid(wp, valid | VALID_TOPLINE);

        // If moved back to where we were, at least move the cursor, otherwise
        // we get stuck at one position. Don't move the cursor up if the
        // first line of the buffer is already on the screen
        loop {
            let topline = nvim_win_get_topline(wp);
            let skipcol = nvim_win_get_skipcol(wp);
            let topfill = nvim_win_get_topfill(wp);

            if topline != prev_topline || skipcol != prev_skipcol || topfill != prev_topfill {
                break;
            }

            let cursor_lnum = nvim_win_get_cursor_lnum(wp);
            if up {
                if cursor_lnum > prev_lnum || nvim_scroll_cursor_down(1, 0) == 0 {
                    break;
                }
            } else if cursor_lnum < prev_lnum
                || prev_topline == 1
                || nvim_scroll_cursor_up(1, 0) == 0
            {
                break;
            }

            // Mark w_topline as valid, otherwise the screen jumps back at the
            // end of the file.
            rs_check_cursor_moved(wp);
            let valid = nvim_win_get_valid(wp);
            nvim_win_set_valid(wp, valid | VALID_TOPLINE);
        }
    }

    if moved {
        nvim_win_set_viewport_invalid(wp, 1);
    }

    nvim_cursor_correct_sms(wp);

    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    if cursor_lnum != prev_lnum {
        let curswant = nvim_win_get_curswant(wp);
        rs_coladvance(wp, curswant);
    }

    nvim_redraw_later(wp, upd_valid::VALID);
}

// =============================================================================
// Additional C Accessor Functions for Cursor Positioning
// =============================================================================

extern "C" {
    // Empty rows
    fn nvim_win_get_empty_rows(wp: WinHandle) -> c_int;
    fn nvim_win_set_empty_rows(wp: WinHandle, val: c_int);

    // Filler rows and botfill
    fn nvim_win_get_filler_rows(wp: WinHandle) -> c_int;
    fn nvim_win_set_filler_rows(wp: WinHandle, val: c_int);
    fn nvim_win_get_botfill(wp: WinHandle) -> c_int;
    fn nvim_win_set_botfill(wp: WinHandle, val: c_int);

    // Mouse dragging state
    fn nvim_get_mouse_dragging() -> c_int;

    // Botline validation
    fn nvim_validate_botline(wp: WinHandle);

    // plines_win_full wrapper
    fn nvim_plines_win_full(
        wp: WinHandle,
        lnum: LinenrT,
        nextp: *mut LinenrT,
        foldedp: *mut c_int,
        cache: c_int,
        limit_winheight: c_int,
    ) -> c_int;
}

// =============================================================================
// Skipcol Calculation (extern from plines crate)
// =============================================================================

extern "C" {
    /// Calculate the skipcol value for a given number of physical lines offset.
    fn rs_skipcol_from_plines(wp: WinHandle, plines_off: c_int) -> ColnrT;
}

// =============================================================================
// Cursor Positioning Functions
// =============================================================================

/// Recompute topline to put the cursor halfway across the window.
///
/// This handles the `zz` command (and related `z.`, `z<CR>` commands).
///
/// # Arguments
/// * `wp` - Window handle
/// * `atend` - If non-zero, also put cursor halfway to end of file
/// * `prefer_above` - If non-zero, prefer adding lines above when centering
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_scroll_cursor_halfway(
    wp: WinHandle,
    atend: c_int,
    prefer_above: c_int,
) {
    if wp.is_null() {
        return;
    }

    let atend = atend != 0;
    let prefer_above = prefer_above != 0;
    let old_topline = nvim_win_get_topline(wp);

    // Get cursor line, handling folding
    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    let mut loff = LineOff::new(cursor_lnum);
    let mut boff = LineOff::new(cursor_lnum);

    // Adjust for folding
    let mut first_lnum = loff.lnum;
    let mut last_lnum = boff.lnum;
    if nvim_hasFolding(
        wp,
        loff.lnum,
        std::ptr::addr_of_mut!(first_lnum),
        std::ptr::addr_of_mut!(last_lnum),
    ) != 0
    {
        loff.lnum = first_lnum;
        boff.lnum = last_lnum;
    }

    let mut used = nvim_plines_win_nofill(wp, loff.lnum, 1);
    loff.fill = 0;
    boff.fill = 0;
    let mut topline = loff.lnum;
    let mut skipcol: ColnrT = 0;

    let p_wrap = nvim_win_get_p_wrap(wp) != 0;
    let p_sms = nvim_win_get_p_sms(wp) != 0;
    let do_sms = p_wrap && p_sms;
    let view_height = nvim_win_get_view_height(wp);
    let line_count = nvim_win_buf_line_count(wp);

    let mut want_height = 0;
    if do_sms {
        // 'smoothscroll' and 'wrap' are set
        if atend {
            want_height = (view_height - used) / 2;
            used = 0;
        } else {
            want_height = view_height;
        }
    }

    let mut topfill = 0;
    while topline > 1 {
        // If using smoothscroll, we can precisely scroll to the
        // exact point where the cursor is halfway down the screen.
        if do_sms {
            rs_topline_back_winheight(wp, std::ptr::addr_of_mut!(loff), 0);
            if loff.height == MAXCOL {
                break;
            }
            used += loff.height;
            if !atend && boff.lnum < line_count {
                rs_botline_forw(wp, std::ptr::addr_of_mut!(boff));
                used += boff.height;
            }
            if used > want_height {
                if used - loff.height < want_height {
                    topline = loff.lnum;
                    topfill = loff.fill;
                    skipcol = rs_skipcol_from_plines(wp, used - want_height);
                }
                break;
            }
            topline = loff.lnum;
            topfill = loff.fill;
            continue;
        }

        // If not using smoothscroll, we have to iteratively find how many
        // lines to scroll down to roughly fit the cursor.
        // This may not be right in the middle if the lines'
        // physical height > 1 (e.g. 'wrap' is on).

        // Depending on "prefer_above" we add a line above or below first.
        // Loop twice to avoid duplicating code.
        let mut done = false;
        let mut above = 0;
        let mut below = 0;

        for round in 1..=2 {
            let should_add_below = if prefer_above {
                round == 2 && below < above
            } else {
                round == 1 && below <= above
            };

            if should_add_below {
                // add a line below the cursor
                if boff.lnum < line_count {
                    rs_botline_forw(wp, std::ptr::addr_of_mut!(boff));
                    used += boff.height;
                    if used > view_height {
                        done = true;
                        break;
                    }
                    below += boff.height;
                } else {
                    below += 1; // count a "~" line
                    if atend {
                        used += 1;
                    }
                }
            }

            let should_add_above = if prefer_above {
                round == 1 && below >= above
            } else {
                round == 1 && below > above
            };

            if should_add_above {
                // add a line above the cursor
                rs_topline_back(wp, std::ptr::addr_of_mut!(loff));
                if loff.height == MAXCOL {
                    used = MAXCOL;
                } else {
                    used += loff.height;
                }
                if used > view_height {
                    done = true;
                    break;
                }
                above += loff.height;
                topline = loff.lnum;
                topfill = loff.fill;
            }
        }

        if done {
            break;
        }
    }

    // Set topline, handling folding
    let mut new_topline = topline;
    if nvim_hasFolding(
        wp,
        topline,
        std::ptr::addr_of_mut!(new_topline),
        std::ptr::null_mut(),
    ) == 0
    {
        // Not in a fold
        let current_topline = nvim_win_get_topline(wp);
        let current_skipcol = nvim_win_get_skipcol(wp);
        if current_topline != topline || skipcol != 0 || current_skipcol != 0 {
            nvim_win_set_topline(wp, topline);
            if skipcol != 0 {
                nvim_win_set_skipcol(wp, skipcol);
                nvim_redraw_later(wp, upd_scroll::NOT_VALID);
            } else if do_sms {
                rs_reset_skipcol(wp);
            }
        }
    } else {
        // In a fold, use the fold start
        nvim_win_set_topline(wp, new_topline);
    }

    nvim_win_set_topfill(wp, topfill);

    if old_topline > nvim_win_get_topline(wp) + view_height {
        nvim_win_set_botfill(wp, 0);
    }

    nvim_check_topfill(wp, 0);

    // Clear and set validity flags
    nvim_win_clear_valid_bits(
        wp,
        VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP,
    );
    let valid = nvim_win_get_valid(wp);
    nvim_win_set_valid(wp, valid | VALID_TOPLINE);
}

/// Recompute topline to put the cursor at the top of the window.
///
/// This handles the `zt` command. Scroll at least `min_scroll` lines.
/// If `always` is true, always set topline (for `zt`).
///
/// # Arguments
/// * `wp` - Window handle
/// * `min_scroll` - Minimum lines to scroll
/// * `always` - If non-zero, always set topline
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_scroll_cursor_top(wp: WinHandle, min_scroll: c_int, always: c_int) {
    if wp.is_null() {
        return;
    }

    let always = always != 0;
    let old_topline = nvim_win_get_topline(wp);
    let old_skipcol = nvim_win_get_skipcol(wp);
    let old_topfill = nvim_win_get_topfill(wp);
    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    let line_count = nvim_win_buf_line_count(wp);
    let view_height = nvim_win_get_view_height(wp);

    let mut off = rs_get_scrolloff_value(wp);
    let mouse_dragging = nvim_get_mouse_dragging();
    if mouse_dragging > 0 {
        off = mouse_dragging - 1;
    }

    // Decrease topline until:
    // - it has become 1
    // - (part of) the cursor line is moved off the screen or
    // - moved at least 'scrolljump' lines and
    // - at least 'scrolloff' lines above and below the cursor
    nvim_validate_cheight(wp);
    let mut scrolled = 0;
    let cline_height = nvim_win_get_cline_height(wp);
    let mut used = cline_height; // includes filler lines above

    if cursor_lnum < old_topline {
        scrolled = used;
    }

    // Get cursor line boundaries (accounting for folding)
    let mut top: LinenrT;
    let mut bot: LinenrT;
    let mut first_lnum = cursor_lnum;
    let mut last_lnum = cursor_lnum;
    if nvim_hasFolding(
        wp,
        cursor_lnum,
        std::ptr::addr_of_mut!(first_lnum),
        std::ptr::addr_of_mut!(last_lnum),
    ) != 0
    {
        top = first_lnum - 1;
        bot = last_lnum + 1;
    } else {
        top = cursor_lnum - 1;
        bot = cursor_lnum + 1;
    }

    let mut new_topline = top + 1;

    // "used" already contains the number of filler lines above, don't add it again.
    // Hide filler lines above cursor line by adding them to "extra".
    let mut extra = nvim_win_get_fill(wp, cursor_lnum);

    // Check if the lines from "top" to "bot" fit in the window. If they do,
    // set new_topline and advance "top" and "bot" to include more lines.
    while top > 0 {
        let i = nvim_plines_win_nofill(wp, top, 1);

        // Adjust top for folding
        let mut fold_start = top;
        nvim_hasFolding(
            wp,
            top,
            std::ptr::addr_of_mut!(fold_start),
            std::ptr::null_mut(),
        );
        top = fold_start;

        if top < old_topline {
            scrolled += i;
        }

        // If scrolling is needed, scroll at least 'sj' lines.
        if (new_topline >= old_topline || scrolled > min_scroll) && extra >= off {
            break;
        }

        used += i;
        if extra + i <= off && bot < line_count {
            let mut next_bot: LinenrT = 0;
            used += nvim_plines_win_full(
                wp,
                bot,
                std::ptr::addr_of_mut!(next_bot),
                std::ptr::null_mut(),
                1,
                1,
            );
            if next_bot > 0 {
                bot = next_bot;
            }
        }
        if used > view_height {
            break;
        }

        extra += i;
        new_topline = top;
        top -= 1;
        bot += 1;
    }

    // If we don't have enough space, put cursor in the middle.
    // This makes sure we get the same position when using "k" and "j"
    // in a small window.
    if used > view_height {
        rs_scroll_cursor_halfway(wp, 0, 0);
    } else {
        // If "always" is false, only adjust topline to a lower value, higher
        // value may happen with wrapping lines.
        let current_topline = nvim_win_get_topline(wp);
        if new_topline < current_topline || always {
            nvim_win_set_topline(wp, new_topline);
        }

        // Ensure topline <= cursor line
        let topline = nvim_win_get_topline(wp);
        if topline > cursor_lnum {
            nvim_win_set_topline(wp, cursor_lnum);
        }

        let topline = nvim_win_get_topline(wp);
        let mut new_topfill = nvim_win_get_fill(wp, topline);
        nvim_win_set_topfill(wp, new_topfill);

        if new_topfill > 0 && extra > off {
            new_topfill -= extra - off;
            new_topfill = new_topfill.max(0);
            nvim_win_set_topfill(wp, new_topfill);
        }

        nvim_check_topfill(wp, 0);

        let new_topline = nvim_win_get_topline(wp);
        if new_topline != old_topline {
            rs_reset_skipcol(wp);
        } else if new_topline == cursor_lnum {
            nvim_validate_virtcol(wp);
            let skipcol = nvim_win_get_skipcol(wp);
            let virtcol = nvim_win_get_virtcol(wp);
            if skipcol >= virtcol {
                // TODO(vim): if the line doesn't fit may optimize w_skipcol
                // instead of making it zero
                rs_reset_skipcol(wp);
            }
        }

        let new_topline = nvim_win_get_topline(wp);
        let new_skipcol = nvim_win_get_skipcol(wp);
        let new_topfill = nvim_win_get_topfill(wp);
        if new_topline != old_topline || new_skipcol != old_skipcol || new_topfill != old_topfill {
            nvim_win_clear_valid_bits(
                wp,
                VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP,
            );
        }
        let valid = nvim_win_get_valid(wp);
        nvim_win_set_valid(wp, valid | VALID_TOPLINE);
        nvim_win_set_viewport_invalid(wp, 1);
    }
}

/// Set `w_empty_rows` and `w_filler_rows` for window `wp`, having used up `used`
/// screen lines for text lines.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_set_empty_rows(wp: WinHandle, used: c_int) {
    if wp.is_null() {
        return;
    }

    nvim_win_set_filler_rows(wp, 0);

    if used == 0 {
        nvim_win_set_empty_rows(wp, 0); // single line that doesn't fit
    } else {
        let view_height = nvim_win_get_view_height(wp);
        let mut empty_rows = view_height - used;
        let botline = nvim_win_get_botline(wp);
        let line_count = nvim_win_buf_line_count(wp);

        if botline <= line_count {
            let filler = nvim_win_get_fill(wp, botline);
            if empty_rows > filler {
                empty_rows -= filler;
                nvim_win_set_filler_rows(wp, filler);
            } else {
                nvim_win_set_filler_rows(wp, empty_rows);
                empty_rows = 0;
            }
        }
        nvim_win_set_empty_rows(wp, empty_rows);
    }
}

/// Recompute topline to put the cursor at the bottom of the window.
///
/// This handles the `zb` command. When scrolling, scroll at least `min_scroll` lines.
/// If `set_topbot` is non-zero, set topline and botline first (for `zb`).
///
/// # Arguments
/// * `wp` - Window handle
/// * `min_scroll` - Minimum lines to scroll
/// * `set_topbot` - If non-zero, set topline and botline first
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_scroll_cursor_bot(wp: WinHandle, min_scroll: c_int, set_topbot: c_int) {
    if wp.is_null() {
        return;
    }

    let set_topbot = set_topbot != 0;
    let old_topline = nvim_win_get_topline(wp);
    let old_skipcol = nvim_win_get_skipcol(wp);
    let old_topfill = nvim_win_get_topfill(wp);
    let old_botline = nvim_win_get_botline(wp);
    let old_valid = nvim_win_get_valid(wp);
    let old_empty_rows = nvim_win_get_empty_rows(wp);
    let cln = nvim_win_get_cursor_lnum(wp); // Cursor Line Number
    let line_count = nvim_win_buf_line_count(wp);
    let view_height = nvim_win_get_view_height(wp);

    let p_wrap = nvim_win_get_p_wrap(wp) != 0;
    let p_sms = nvim_win_get_p_sms(wp) != 0;
    let do_sms = p_wrap && p_sms;

    if set_topbot {
        let mut used = 0;
        nvim_win_set_botline(wp, cln + 1);
        let mut loff = LineOff {
            lnum: cln + 1,
            fill: 0,
            height: 0,
        };

        loop {
            rs_topline_back_winheight(wp, std::ptr::addr_of_mut!(loff), 0);
            if loff.height == MAXCOL {
                break;
            }
            if used + loff.height > view_height {
                if do_sms {
                    // 'smoothscroll' and 'wrap' are set. The above line is
                    // too long to show in its entirety, so we show just a part
                    // of it.
                    if used < view_height {
                        let plines_offset = used + loff.height - view_height;
                        used = view_height;
                        nvim_win_set_topfill(wp, loff.fill);
                        nvim_win_set_topline(wp, loff.lnum);
                        nvim_win_set_skipcol(wp, rs_skipcol_from_plines(wp, plines_offset));
                    }
                }
                break;
            }
            nvim_win_set_topfill(wp, loff.fill);
            nvim_win_set_topline(wp, loff.lnum);
            used += loff.height;
        }

        rs_set_empty_rows(wp, used);
        let valid = nvim_win_get_valid(wp);
        nvim_win_set_valid(wp, valid | VALID_BOTLINE | VALID_BOTLINE_AP);

        let new_topline = nvim_win_get_topline(wp);
        let new_topfill = nvim_win_get_topfill(wp);
        let new_skipcol = nvim_win_get_skipcol(wp);
        if new_topline != old_topline
            || new_topfill != old_topfill
            || new_skipcol != old_skipcol
            || new_skipcol != 0
        {
            nvim_win_clear_valid_bits(wp, VALID_WROW | VALID_CROW);
            if new_skipcol != old_skipcol {
                nvim_redraw_later(wp, upd_scroll::NOT_VALID);
            } else {
                rs_reset_skipcol(wp);
            }
        }
    } else {
        nvim_validate_botline(wp);
    }

    // The lines of the cursor line itself are always used.
    let mut used = nvim_plines_win_nofill(wp, cln, 1);

    let mut scrolled = 0;
    // If the cursor is on or below botline, we will at least scroll by the
    // height of the cursor line, which is "used". Correct for empty lines,
    // which are really part of botline.
    let botline = nvim_win_get_botline(wp);
    let empty_rows = nvim_win_get_empty_rows(wp);
    if cln >= botline {
        scrolled = used;
        if cln == botline {
            scrolled -= empty_rows;
        }
        if do_sms {
            // 'smoothscroll' and 'wrap' are set.
            // Calculate how many screen lines the current top line of window
            // occupies. If it is occupying more than the entire window, we
            // need to scroll the additional clipped lines to scroll past the
            // top line before we can move on to the other lines.
            let topline = nvim_win_get_topline(wp);
            let top_plines = nvim_plines_win_nofill(wp, topline, 0);
            let view_width = nvim_win_get_view_width(wp);
            let width1 = view_width - nvim_win_col_off(wp);

            if width1 > 0 {
                let width2 = width1 + nvim_win_col_off2(wp);
                let mut skip_lines = 0;
                let skipcol = nvim_win_get_skipcol(wp);

                // A similar formula is used in curs_columns().
                if skipcol > width1 {
                    skip_lines += (skipcol - width1) / width2 + 1;
                } else if skipcol > 0 {
                    skip_lines = 1;
                }

                let adjusted_top_plines = top_plines - skip_lines;
                if adjusted_top_plines > view_height {
                    scrolled += adjusted_top_plines - view_height;
                }
            }
        }
    }

    // Get cursor line boundaries for folding
    let mut loff_lnum = cln;
    let mut boff_lnum = cln;
    if nvim_hasFolding(
        wp,
        cln,
        std::ptr::addr_of_mut!(loff_lnum),
        std::ptr::addr_of_mut!(boff_lnum),
    ) == 0
    {
        loff_lnum = cln;
        boff_lnum = cln;
    }

    let mut loff = LineOff {
        lnum: loff_lnum,
        fill: 0,
        height: 0,
    };
    let mut boff = LineOff {
        lnum: boff_lnum,
        fill: 0,
        height: 0,
    };

    let botline = nvim_win_get_botline(wp);
    let filler_rows = nvim_win_get_filler_rows(wp);
    let fill_below_window = nvim_win_get_fill(wp, botline) - filler_rows;

    let mut extra = 0;
    let so = rs_get_scrolloff_value(wp);
    let mouse_dragging = nvim_get_mouse_dragging();

    // Stop counting lines to scroll when
    // - hitting start of the file
    // - scrolled nothing or at least 'sj' lines
    // - at least 'so' lines below the cursor
    // - lines between botline and cursor have been counted
    while loff.lnum > 1 {
        let so_effective = if mouse_dragging > 0 {
            mouse_dragging - 1
        } else {
            so
        };

        // Stop when scrolled nothing or at least "min_scroll", found "extra"
        // context for 'scrolloff' and counted all lines below the window.
        if ((scrolled <= 0 || scrolled >= min_scroll) && extra >= so_effective
            || boff.lnum + 1 > line_count)
            && loff.lnum <= botline
            && (loff.lnum < botline || loff.fill >= fill_below_window)
        {
            break;
        }

        // Add one line above
        rs_topline_back(wp, std::ptr::addr_of_mut!(loff));
        if loff.height == MAXCOL {
            used = MAXCOL;
        } else {
            used += loff.height;
        }
        if used > view_height {
            break;
        }

        let botline = nvim_win_get_botline(wp);
        let empty_rows = nvim_win_get_empty_rows(wp);
        if loff.lnum >= botline && (loff.lnum > botline || loff.fill <= fill_below_window) {
            // Count screen lines that are below the window.
            scrolled += loff.height;
            if loff.lnum == botline && loff.fill == 0 {
                scrolled -= empty_rows;
            }
        }

        if boff.lnum < line_count {
            // Add one line below
            rs_botline_forw(wp, std::ptr::addr_of_mut!(boff));
            used += boff.height;
            if used > view_height {
                break;
            }

            let so_effective = if mouse_dragging > 0 {
                mouse_dragging - 1
            } else {
                so
            };

            if extra < so_effective || scrolled < min_scroll {
                extra += boff.height;
                let botline = nvim_win_get_botline(wp);
                let filler_rows = nvim_win_get_filler_rows(wp);
                if boff.lnum >= botline || (boff.lnum + 1 == botline && boff.fill > filler_rows) {
                    // Count screen lines that are below the window.
                    scrolled += boff.height;
                    let empty_rows = nvim_win_get_empty_rows(wp);
                    if boff.lnum == botline && boff.fill == 0 {
                        scrolled -= empty_rows;
                    }
                }
            }
        }
    }

    // Determine how many lines to scroll
    let line_count_scroll: LinenrT;

    // w_empty_rows is larger, no need to scroll
    if scrolled <= 0 {
        line_count_scroll = 0;
    // more than a screenfull, don't scroll but redraw
    } else if used > view_height {
        line_count_scroll = used;
    // scroll minimal number of lines
    } else {
        let mut count = 0;
        let topfill = nvim_win_get_topfill(wp);
        let topline = nvim_win_get_topline(wp);
        let botline = nvim_win_get_botline(wp);

        boff.fill = topfill;
        boff.lnum = topline - 1;

        let mut i = 0;
        while i < scrolled && boff.lnum < botline {
            rs_botline_forw(wp, std::ptr::addr_of_mut!(boff));
            i += boff.height;
            count += 1;
        }

        if i < scrolled {
            // below w_botline, don't scroll
            line_count_scroll = 9999;
        } else {
            line_count_scroll = count;
        }
    }

    // Scroll up if the cursor is off the bottom of the screen a bit.
    // Otherwise put it at 1/2 of the screen.
    if line_count_scroll >= view_height && line_count_scroll > min_scroll {
        rs_scroll_cursor_halfway(wp, 0, 1);
    } else if line_count_scroll > 0 {
        if do_sms {
            rs_scrollup(wp, scrolled, 1); // TODO(vim):
        } else {
            rs_scrollup(wp, line_count_scroll, 1);
        }
    }

    // If topline didn't change we need to restore w_botline and w_empty_rows
    // (we changed them).
    // If topline did change, update_screen() will set botline.
    let new_topline = nvim_win_get_topline(wp);
    let new_skipcol = nvim_win_get_skipcol(wp);
    if new_topline == old_topline && new_skipcol == old_skipcol && set_topbot {
        nvim_win_set_botline(wp, old_botline);
        nvim_win_set_empty_rows(wp, old_empty_rows);
        nvim_win_set_valid(wp, old_valid);
    }
    let valid = nvim_win_get_valid(wp);
    nvim_win_set_valid(wp, valid | VALID_TOPLINE);
    nvim_win_set_viewport_invalid(wp, 1);

    // Make sure cursor is still visible after adjusting skipcol for "zb".
    if set_topbot {
        nvim_cursor_correct_sms(wp);
    }
}

// =============================================================================
// Set Valid Virtcol Function
// =============================================================================

extern "C" {
    // Virtcol setter
    fn nvim_win_set_virtcol(wp: WinHandle, val: ColnrT);

    // Redraw for cursorcolumn wrapper
    fn nvim_redraw_for_cursorcolumn(wp: WinHandle);
}

/// Set wp->w_virtcol to a value ("vcol") that is already valid.
/// Handles redrawing if wp->w_virtcol was previously invalid.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_set_valid_virtcol(wp: WinHandle, vcol: ColnrT) {
    if wp.is_null() {
        return;
    }

    nvim_win_set_virtcol(wp, vcol);
    nvim_redraw_for_cursorcolumn(wp);

    let valid = nvim_win_get_valid(wp);
    nvim_win_set_valid(wp, valid | VALID_VIRTCOL);
}

// =============================================================================
// Set Topline Function
// =============================================================================

extern "C" {
    // Topline was set flag
    fn nvim_win_set_topline_was_set(wp: WinHandle, val: c_int);
}

/// Set wp->w_topline to a certain number.
///
/// Handles folding, updates botline approximation, and triggers redraw.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_set_topline(wp: WinHandle, lnum: LinenrT) {
    if wp.is_null() {
        return;
    }

    let prev_topline = nvim_win_get_topline(wp);

    // Go to first of folded lines
    let mut folded_lnum = lnum;
    nvim_hasFolding(
        wp,
        lnum,
        std::ptr::addr_of_mut!(folded_lnum),
        std::ptr::null_mut(),
    );

    // Approximate the value of w_botline
    let botline = nvim_win_get_botline(wp);
    let topline = nvim_win_get_topline(wp);
    nvim_win_set_botline(wp, botline + (folded_lnum - topline));
    nvim_win_set_topline(wp, folded_lnum);
    nvim_win_set_topline_was_set(wp, 1);

    if folded_lnum != prev_topline {
        // Keep the filler lines when the topline didn't change.
        nvim_win_set_topfill(wp, 0);
    }

    nvim_win_clear_valid_bits(wp, VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_TOPLINE);
    // Don't set VALID_TOPLINE here, 'scrolloff' needs to be checked.
    nvim_redraw_later(wp, upd::VALID);
}

// =============================================================================
// Scroll Clamping Functions
// =============================================================================

extern "C" {
    // Buffer line count accessor for curbuf
    fn nvim_curbuf_line_count() -> LinenrT;
}

/// Scroll the screen one line down, but don't do it if it would move the
/// cursor off the screen.
///
/// # Safety
/// Accesses curwin and curbuf globals.
#[no_mangle]
pub unsafe extern "C" fn rs_scrolldown_clamp() {
    let wp = nvim_get_curwin();
    if wp.is_null() {
        return;
    }

    let topline = nvim_win_get_topline(wp);
    let topfill = nvim_win_get_topfill(wp);
    let fill = nvim_win_get_fill(wp, topline);
    let can_fill = topfill < fill;

    if topline <= 1 && !can_fill {
        return;
    }

    nvim_validate_cursor_win(wp); // w_wrow needs to be valid

    // Compute the row number of the last row of the cursor line
    // and make sure it doesn't go off the screen. Make sure the cursor
    // doesn't go past 'scrolloff' lines from the screen end.
    let mut end_row = nvim_win_get_wrow(wp);
    if can_fill {
        end_row += 1;
    } else {
        end_row += nvim_plines_win_nofill(wp, topline - 1, 1);
    }

    let p_wrap = nvim_win_get_p_wrap(wp) != 0;
    let view_width = nvim_win_get_view_width(wp);
    if p_wrap && view_width != 0 {
        nvim_validate_cheight(wp);
        nvim_validate_virtcol(wp);
        let cline_height = nvim_win_get_cline_height(wp);
        let virtcol = nvim_win_get_virtcol(wp);
        end_row += cline_height - 1 - virtcol / view_width;
    }

    let view_height = nvim_win_get_view_height(wp);
    let so = rs_get_scrolloff_value(wp);
    if end_row < view_height - so {
        if can_fill {
            nvim_win_set_topfill(wp, topfill + 1);
            nvim_check_topfill(wp, 1);
        } else {
            let mut new_topline = topline - 1;
            nvim_win_set_topline(wp, new_topline);
            nvim_win_set_topfill(wp, 0);

            // Handle folding - go to first line of fold
            let mut first_lnum = new_topline;
            if nvim_hasFolding(
                wp,
                new_topline,
                std::ptr::addr_of_mut!(first_lnum),
                std::ptr::null_mut(),
            ) != 0
            {
                new_topline = first_lnum;
                nvim_win_set_topline(wp, new_topline);
            }
        }

        // approximate w_botline
        let botline = nvim_win_get_botline(wp);
        nvim_win_set_botline(wp, botline - 1);

        nvim_win_clear_valid_bits(wp, VALID_WROW | VALID_CROW | VALID_BOTLINE);
    }
}

/// Scroll the screen one line up, but don't do it if it would move the cursor
/// off the screen.
///
/// # Safety
/// Accesses curwin and curbuf globals.
#[no_mangle]
pub unsafe extern "C" fn rs_scrollup_clamp() {
    let wp = nvim_get_curwin();
    if wp.is_null() {
        return;
    }

    let topline = nvim_win_get_topline(wp);
    let topfill = nvim_win_get_topfill(wp);
    let line_count = nvim_curbuf_line_count();

    if topline == line_count && topfill == 0 {
        return;
    }

    nvim_validate_cursor_win(wp); // w_wrow needs to be valid

    // Compute the row number of the first row of the cursor line
    // and make sure it doesn't go off the screen. Make sure the cursor
    // doesn't go before 'scrolloff' lines from the screen start.
    let wrow = nvim_win_get_wrow(wp);
    let plines = nvim_plines_win_nofill(wp, topline, 1);
    let mut start_row = wrow - plines - topfill;

    let p_wrap = nvim_win_get_p_wrap(wp) != 0;
    let view_width = nvim_win_get_view_width(wp);
    if p_wrap && view_width != 0 {
        nvim_validate_virtcol(wp);
        let virtcol = nvim_win_get_virtcol(wp);
        start_row -= virtcol / view_width;
    }

    let so = rs_get_scrolloff_value(wp);
    if start_row >= so {
        if topfill > 0 {
            nvim_win_set_topfill(wp, topfill - 1);
        } else {
            // Handle folding - go to last line of fold
            let mut last_lnum = topline;
            nvim_hasFolding(
                wp,
                topline,
                std::ptr::null_mut(),
                std::ptr::addr_of_mut!(last_lnum),
            );
            nvim_win_set_topline(wp, last_lnum + 1);
        }

        // approximate w_botline
        let botline = nvim_win_get_botline(wp);
        nvim_win_set_botline(wp, botline + 1);

        nvim_win_clear_valid_bits(wp, VALID_WROW | VALID_CROW | VALID_BOTLINE);
    }
}

// =============================================================================
// Cursor Correction for Smooth Scroll
// =============================================================================

extern "C" {
    // Smooth scroll marker overlap
    fn rs_sms_marker_overlap(wp: WinHandle, extra2: c_int) -> c_int;

    // Cursor position setters for coladd
    fn nvim_win_set_cursor_coladd(wp: WinHandle, coladd: ColnrT);
}

/// Make sure the cursor is in the visible part of the topline after scrolling
/// the screen with 'smoothscroll'.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_correct_sms(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    let p_sms = nvim_win_get_p_sms(wp) != 0;
    let p_wrap = nvim_win_get_p_wrap(wp) != 0;
    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    let topline = nvim_win_get_topline(wp);

    if !p_sms || !p_wrap || cursor_lnum != topline {
        return;
    }

    let so = rs_get_scrolloff_value(wp);
    let view_width = nvim_win_get_view_width(wp);
    let view_height = nvim_win_get_view_height(wp);
    let width1 = view_width - nvim_win_col_off(wp);
    let width2 = width1 + nvim_win_col_off2(wp);
    let mut so_cols = if so == 0 {
        0
    } else {
        width1 + (so - 1) * width2
    };
    let space_cols = (view_height - 1) * width2;
    let size = if so == 0 {
        0
    } else {
        nvim_linetabsize_eol(wp, topline)
    };

    let skipcol = nvim_win_get_skipcol(wp);
    if topline == 1 && skipcol == 0 {
        so_cols = 0; // Ignore 'scrolloff' at top of buffer.
    } else if so_cols > space_cols / 2 {
        so_cols = space_cols / 2; // Not enough room: put cursor in the middle.
    }

    // Not enough screen lines in topline: ignore 'scrolloff'.
    while so_cols > size && so_cols - width2 >= width1 && width1 > 0 {
        so_cols -= width2;
    }
    if so_cols >= width1 && so_cols > size {
        so_cols -= width1;
    }

    let overlap = if skipcol == 0 {
        0
    } else {
        rs_sms_marker_overlap(wp, view_width - width2)
    };

    // If we have non-zero scrolloff, ignore marker overlap.
    let top = skipcol + if so_cols != 0 { so_cols } else { overlap };
    let bot = skipcol + width1 + (view_height - 1) * width2 - so_cols;

    nvim_validate_virtcol(wp);
    let virtcol = nvim_win_get_virtcol(wp);
    let mut col = virtcol;

    if col < top {
        if col < width1 {
            col += width1;
        }
        while width2 > 0 && col < top {
            col += width2;
        }
    } else {
        while width2 > 0 && col >= bot {
            col -= width2;
        }
    }

    if col != virtcol {
        nvim_win_set_curswant(wp, col);
        let rc = rs_coladvance(wp, col);

        // validate_virtcol() marked various things as valid, but after
        // moving the cursor they need to be recomputed
        nvim_win_clear_valid_bits(
            wp,
            VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW | VALID_VIRTCOL,
        );

        let line_count = nvim_win_buf_line_count(wp);
        let skipcol = nvim_win_get_skipcol(wp);
        let cursor_lnum = nvim_win_get_cursor_lnum(wp);
        if rc != 0 && skipcol > 0 && cursor_lnum < line_count {
            nvim_validate_virtcol(wp);
            let virtcol = nvim_win_get_virtcol(wp);
            if virtcol < skipcol + overlap {
                // Cursor still not visible: move it to the next line instead.
                nvim_win_set_cursor_lnum(wp, cursor_lnum + 1);
                nvim_win_set_cursor_col(wp, 0);
                nvim_win_set_cursor_coladd(wp, 0);
                nvim_win_set_curswant(wp, 0);
                nvim_win_clear_valid_bits(wp, VALID_VIRTCOL);
            }
        }
    }
}

// =============================================================================
// Adjust Skipcol
// =============================================================================

/// Called after changing the cursor column: make sure that curwin->w_skipcol is
/// valid for 'smoothscroll'.
///
/// # Safety
/// Accesses curwin global.
#[no_mangle]
pub unsafe extern "C" fn rs_adjust_skipcol() {
    let wp = nvim_get_curwin();
    if wp.is_null() {
        return;
    }

    let p_wrap = nvim_win_get_p_wrap(wp) != 0;
    let p_sms = nvim_win_get_p_sms(wp) != 0;
    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    let topline = nvim_win_get_topline(wp);

    if !p_wrap || !p_sms || cursor_lnum != topline {
        return;
    }

    let view_width = nvim_win_get_view_width(wp);
    let width1 = view_width - nvim_win_col_off(wp);
    if width1 <= 0 {
        return; // no text will be displayed
    }

    let width2 = width1 + nvim_win_col_off2(wp);
    let so = rs_get_scrolloff_value(wp);
    let scrolloff_cols: ColnrT = if so == 0 {
        0
    } else {
        width1 + (so - 1) * width2
    };
    let mut scrolled = false;

    nvim_validate_cheight(wp);
    let cline_height = nvim_win_get_cline_height(wp);
    let view_height = nvim_win_get_view_height(wp);

    if cline_height == view_height
        // w_cline_height may be capped at w_view_height, check there aren't
        // actually more lines.
        && nvim_plines_win(wp, cursor_lnum, 0) <= view_height
    {
        // the line just fits in the window, don't scroll
        rs_reset_skipcol(wp);
        return;
    }

    nvim_validate_virtcol(wp);
    let overlap = rs_sms_marker_overlap(wp, view_width - width2);
    let mut skipcol = nvim_win_get_skipcol(wp);
    let virtcol = nvim_win_get_virtcol(wp);

    while skipcol > 0 && virtcol < skipcol + overlap + scrolloff_cols {
        // scroll a screen line down
        if skipcol >= width1 + width2 {
            skipcol -= width2;
        } else {
            skipcol -= width1;
        }
        nvim_win_set_skipcol(wp, skipcol);
        scrolled = true;
    }

    if scrolled {
        nvim_validate_virtcol(wp);
        nvim_redraw_later(wp, upd::NOT_VALID);
        return; // don't scroll in the other direction now
    }

    let mut row = 0;
    let virtcol = nvim_win_get_virtcol(wp);
    let mut col = virtcol + scrolloff_cols;

    // Avoid adjusting for 'scrolloff' beyond the text line height.
    if scrolloff_cols > 0 {
        let mut size = nvim_linetabsize_eol(wp, topline);
        size = width1 + width2 * ((size - width1 + width2 - 1) / width2);
        while col > size {
            col -= width2;
        }
    }

    let skipcol = nvim_win_get_skipcol(wp);
    col -= skipcol;

    if col >= width1 {
        col -= width1;
        row += 1;
    }
    if col > width2 {
        row += col / width2;
    }

    if row >= view_height {
        let mut skipcol = nvim_win_get_skipcol(wp);
        if skipcol == 0 {
            skipcol += width1;
            nvim_win_set_skipcol(wp, skipcol);
            row -= 1;
        }
        if row >= view_height {
            skipcol = nvim_win_get_skipcol(wp);
            skipcol += (row - view_height) * width2;
            nvim_win_set_skipcol(wp, skipcol);
        }
        nvim_redraw_later(wp, upd::NOT_VALID);
    }
}

// =============================================================================
// Cursor Correction
// =============================================================================

/// Correct cursor position to be within scrolloff bounds.
///
/// Corrects the cursor position so that it is in a part of the screen at least
/// 'so' lines from the top and bottom, if possible.
/// If not possible, put it at the same position as `scroll_cursor_halfway()`.
/// When called topline must be valid!
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_cursor_correct(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    // How many lines we would like to have above/below the cursor depends on
    // whether the first/last line of the file is on screen.
    let mut above_wanted = rs_get_scrolloff_value(wp);
    let mut below_wanted = rs_get_scrolloff_value(wp);

    let mouse_dragging = nvim_get_mouse_dragging();
    if mouse_dragging > 0 {
        above_wanted = mouse_dragging - 1;
        below_wanted = mouse_dragging - 1;
    }

    let topline = nvim_win_get_topline(wp);
    let view_height = nvim_win_get_view_height(wp);

    if topline == 1 {
        above_wanted = 0;
        let max_off = view_height / 2;
        below_wanted = below_wanted.min(max_off);
    }

    nvim_validate_botline(wp);
    let botline = nvim_win_get_botline(wp);
    let line_count = nvim_win_buf_line_count(wp);

    if botline == line_count + 1 && mouse_dragging == 0 {
        below_wanted = 0;
        let max_off = (view_height - 1) / 2;
        above_wanted = above_wanted.min(max_off);
    }

    // If there are sufficient file-lines above and below the cursor, we can
    // return now.
    let cln = nvim_win_get_cursor_lnum(wp); // Cursor Line Number
    if cln >= topline + above_wanted
        && cln < botline - below_wanted
        && nvim_win_lines_concealed(wp) == 0
    {
        return;
    }

    let p_sms = nvim_win_get_p_sms(wp) != 0;
    let p_wrap = nvim_win_get_p_wrap(wp) != 0;

    if p_sms && !p_wrap {
        // 'smoothscroll' is active
        let cline_height = nvim_win_get_cline_height(wp);
        if cline_height == view_height {
            // The cursor line just fits in the window, don't scroll.
            rs_reset_skipcol(wp);
            return;
        }
        // TODO(vim): If the cursor line doesn't fit in the window then only
        // adjust w_skipcol.
    }

    // Narrow down the area where the cursor can be put by taking lines from
    // the top and the bottom until:
    // - the desired context lines are found
    // - the lines from the top is past the lines from the bottom
    let mut top_lnum = topline;
    let mut bot_lnum = botline - 1;

    // count filler lines as context
    let topfill = nvim_win_get_topfill(wp);
    let filler_rows = nvim_win_get_filler_rows(wp);
    let mut above = topfill; // screen lines above topline
    let mut below = filler_rows; // screen lines below botline

    while (above < above_wanted || below < below_wanted) && top_lnum < bot_lnum {
        if below < below_wanted && (below <= above || above >= above_wanted) {
            below += nvim_plines_win_full(
                wp,
                bot_lnum,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                1,
                1,
            );
            let mut fold_start: LinenrT = 0;
            nvim_hasFolding(
                wp,
                bot_lnum,
                std::ptr::addr_of_mut!(fold_start),
                std::ptr::null_mut(),
            );
            bot_lnum = fold_start;
            bot_lnum -= 1;
        }
        if above < above_wanted && (above < below || below >= below_wanted) {
            above += nvim_plines_win_nofill(wp, top_lnum, 1);
            let mut fold_end: LinenrT = 0;
            nvim_hasFolding(
                wp,
                top_lnum,
                std::ptr::null_mut(),
                std::ptr::addr_of_mut!(fold_end),
            );
            top_lnum = fold_end;

            // Count filler lines below this line as context.
            if top_lnum < bot_lnum {
                above += nvim_win_get_fill(wp, top_lnum + 1);
            }
            top_lnum += 1;
        }
    }

    if top_lnum == bot_lnum || bot_lnum == 0 {
        nvim_win_set_cursor_lnum(wp, top_lnum);
    } else if top_lnum > bot_lnum {
        nvim_win_set_cursor_lnum(wp, bot_lnum);
    } else {
        if cln < top_lnum && topline > 1 {
            nvim_win_set_cursor_lnum(wp, top_lnum);
            nvim_win_clear_valid_bits(wp, VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW);
        }
        if cln > bot_lnum && botline <= line_count {
            nvim_win_set_cursor_lnum(wp, bot_lnum);
            nvim_win_clear_valid_bits(wp, VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW);
        }
    }

    rs_check_cursor_moved(wp);
    let valid = nvim_win_get_valid(wp);
    nvim_win_set_valid(wp, valid | VALID_TOPLINE);
    nvim_win_set_viewport_invalid(wp, 1);
}

// =============================================================================
// Direction Constants
// =============================================================================

/// Direction: Forward (matches C `FORWARD = 1`).
const DIRECTION_FORWARD: c_int = 1;

/// Direction: Backward (matches C `BACKWARD = -1`).
const DIRECTION_BACKWARD: c_int = -1;

// =============================================================================
// Page Scroll Overlap Calculation
// =============================================================================

/// Calculate overlap for page-up/page-down scrolling.
///
/// Decides how much overlap to use for page-up or page-down scrolling.
/// This is symmetric, so that doing both keeps the same lines displayed.
/// Three lines are examined to determine optimal overlap.
///
/// # Arguments
/// * `dir` - Direction: `DIRECTION_FORWARD` (1) or `DIRECTION_BACKWARD` (-1)
///
/// # Returns
/// The number of lines to use for the scroll amount (includes overlap adjustment).
///
/// # Safety
/// Accesses curwin and curbuf globals.
#[no_mangle]
pub unsafe extern "C" fn rs_get_scroll_overlap(dir: c_int) -> c_int {
    let wp = nvim_get_curwin();
    if wp.is_null() {
        return 0;
    }

    let view_height = nvim_win_get_view_height(wp);
    let min_height = view_height - 2;

    nvim_validate_botline(wp);

    let topline = nvim_win_get_topline(wp);
    let botline = nvim_win_get_botline(wp);
    let line_count = nvim_curbuf_line_count();

    // Check if we're at the buffer boundaries
    if (dir == DIRECTION_BACKWARD && topline == 1)
        || (dir == DIRECTION_FORWARD && botline > line_count)
    {
        return min_height + 2; // no overlap, still handle 'smoothscroll'
    }

    // Initialize lineoff for the edge line
    let initial_lnum = if dir == DIRECTION_FORWARD {
        botline
    } else {
        topline - 1
    };

    let fill_base = nvim_win_get_fill(wp, initial_lnum + c_int::from(dir == DIRECTION_BACKWARD));
    let fill_subtract = if dir == DIRECTION_FORWARD {
        nvim_win_get_filler_rows(wp)
    } else {
        nvim_win_get_topfill(wp)
    };

    let mut loff = LineOff {
        lnum: initial_lnum,
        fill: fill_base - fill_subtract,
        height: 0,
    };

    loff.height = if loff.fill > 0 {
        1
    } else {
        nvim_plines_win_nofill(wp, loff.lnum, 1)
    };

    let h1 = loff.height;
    if h1 > min_height {
        return min_height + 2; // no overlap
    }

    // Move to next line
    if dir == DIRECTION_FORWARD {
        rs_topline_back(wp, std::ptr::addr_of_mut!(loff));
    } else {
        rs_botline_forw(wp, std::ptr::addr_of_mut!(loff));
    }

    let h2 = loff.height;
    if h2 == MAXCOL || h2 + h1 > min_height {
        return min_height + 2; // no overlap
    }

    // Move to next line
    if dir == DIRECTION_FORWARD {
        rs_topline_back(wp, std::ptr::addr_of_mut!(loff));
    } else {
        rs_botline_forw(wp, std::ptr::addr_of_mut!(loff));
    }

    let h3 = loff.height;
    if h3 == MAXCOL || h3 + h2 > min_height {
        return min_height + 2; // no overlap
    }

    // Move to next line
    if dir == DIRECTION_FORWARD {
        rs_topline_back(wp, std::ptr::addr_of_mut!(loff));
    } else {
        rs_botline_forw(wp, std::ptr::addr_of_mut!(loff));
    }

    let h4 = loff.height;
    if h4 == MAXCOL || h4 + h3 + h2 > min_height || h3 + h2 + h1 > min_height {
        min_height + 1 // 1 line overlap
    } else {
        min_height // 2 lines overlap
    }
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

    #[test]
    fn test_lineoff_new() {
        let loff = LineOff::new(10);
        assert_eq!(loff.lnum, 10);
        assert_eq!(loff.fill, 0);
        assert_eq!(loff.height, 0);
    }

    #[test]
    fn test_lineoff_default() {
        let loff = LineOff::default();
        assert_eq!(loff.lnum, 0);
        assert_eq!(loff.fill, 0);
        assert_eq!(loff.height, 0);
    }

    #[test]
    fn test_maxcol() {
        assert_eq!(MAXCOL, i32::MAX);
    }
}
