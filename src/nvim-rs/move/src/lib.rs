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
