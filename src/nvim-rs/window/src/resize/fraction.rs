//! Fraction and scroll default calculations.
//!
//! This module provides Rust implementations of `set_fraction`,
//! `win_default_scroll`, and `scroll_to_fraction` from `src/nvim/window.c`.

use std::ffi::{c_int, c_long, c_longlong};
use std::ptr;

use crate::win_struct::{win_mut, win_ref};
use crate::WinHandle;

// =============================================================================
// Constants
// =============================================================================

/// Multiplier for computing cursor fraction in window.
/// Must match C `FRACTION_MULT` (window.c L7083).
pub const FRACTION_MULT: c_int = 16384;

/// Minimum number of screen lines.
/// Must match C `MIN_LINES` (window.h L27).
pub const MIN_LINES: c_int = 2;

/// Redraw type: some lines need validation.
/// Must match C `UPD_SOME_VALID` (screen.h).
const UPD_SOME_VALID: c_int = 35;

// =============================================================================
// Type aliases
// =============================================================================

type LinenrT = c_int;
type ColnrT = c_int;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {

    // Accessors used by scroll_to_fraction
    fn nvim_win_get_p_scb(wp: WinHandle) -> c_int;
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_win_buf_line_count(wp: WinHandle) -> LinenrT;

    // plines functions (C wrappers)
    fn nvim_plines_win_col(wp: WinHandle, lnum: LinenrT, column: c_long) -> c_int;
    fn nvim_plines_win(wp: WinHandle, lnum: LinenrT, limit: c_int) -> c_int;
    fn nvim_plines_win_nofill(wp: WinHandle, lnum: LinenrT, winheight: c_int) -> c_int;

    // Column offset helpers
    #[link_name = "win_col_off"]
    fn nvim_win_col_off(wp: WinHandle) -> c_int;
    #[link_name = "win_col_off2"]
    fn nvim_win_col_off2(wp: WinHandle) -> c_int;

    // Fold helpers
    fn nvim_hasFolding(
        wp: WinHandle,
        lnum: LinenrT,
        firstp: *mut LinenrT,
        lastp: *mut LinenrT,
    ) -> c_int;

    // Decoration
    fn nvim_decor_conceal_line(wp: WinHandle, row: c_int, check_cursor: c_int) -> c_int;

    // Higher-level functions already in Rust (called by C name)
    #[link_name = "set_topline"]
    fn nvim_set_topline(wp: WinHandle, lnum: LinenrT);
    #[link_name = "curs_columns"]
    fn nvim_curs_columns(wp: WinHandle, may_scroll: c_int);
    #[link_name = "redraw_later"]
    fn nvim_redraw_later(wp: WinHandle, redraw_type: c_int);
    #[link_name = "invalidate_botline"]
    fn nvim_invalidate_botline(wp: WinHandle);
}

// =============================================================================
// Implementations
// =============================================================================

/// Set wp->w_fraction for the current w_wrow and w_view_height.
/// Has no effect when the window is less than two lines.
///
/// Equivalent to C `set_fraction()` (window.c L7087).
fn set_fraction_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    unsafe {
        let view_height = win_ref(wp).w_view_height;
        if view_height > 1 {
            let wrow = win_ref(wp).w_wrow;
            let fraction = (wrow * FRACTION_MULT + FRACTION_MULT / 2) / view_height;
            win_mut(wp).w_fraction = fraction;
        }
    }
}

/// Return the default scroll amount for a window (half view height, min 1).
///
/// Equivalent to C `win_default_scroll()` (window.c L7397).
fn win_default_scroll_impl(wp: WinHandle) -> c_longlong {
    if wp.is_null() {
        return 1;
    }

    unsafe {
        let view_height = c_longlong::from(win_ref(wp).w_view_height);
        (view_height / 2).max(1)
    }
}

/// Scroll window `wp` so the cursor stays at the same relative position
/// after a resize. Called after window height changes.
///
/// Equivalent to C `scroll_to_fraction()` (window_shim.c).
fn scroll_to_fraction_impl(wp: WinHandle, prev_height: c_int) {
    if wp.is_null() {
        return;
    }

    unsafe {
        let height = win_ref(wp).w_view_height;
        let p_scb = nvim_win_get_p_scb(wp) != 0;
        let is_curwin = nvim_get_curwin() == wp;
        let line_count = nvim_win_buf_line_count(wp);
        let topline = win_ref(wp).w_topline;

        // Don't change w_topline in any of these cases:
        // - window height is 0
        // - 'scrollbind' is set and this isn't the current window
        // - window height is sufficient to display the whole buffer and first
        //   line is visible.
        if height > 0 && (!p_scb || is_curwin) && (height < line_count || topline > 1) {
            // Find a value for w_topline that shows the cursor at the same
            // relative position in the window as before (more or less).
            let cursor_lnum = win_ref(wp).w_cursor.lnum;
            let cursor_col = win_ref(wp).w_cursor.col;
            // can happen when starting up
            let mut lnum: LinenrT = cursor_lnum.max(1);

            let fraction = win_ref(wp).w_fraction;
            let wrow = (fraction * height - 1) / FRACTION_MULT;
            win_mut(wp).w_wrow = wrow;

            let line_size = nvim_plines_win_col(wp, lnum, c_long::from(cursor_col)) - 1;
            let mut sline = wrow - line_size;

            if sline >= 0 {
                // Make sure the whole cursor line is visible, if possible.
                let rows = nvim_plines_win(wp, lnum, 0);
                let view_height = win_ref(wp).w_view_height;

                if sline > view_height - rows {
                    sline = view_height - rows;
                    win_mut(wp).w_wrow = wrow - (rows - line_size);
                }
            }

            let view_height = win_ref(wp).w_view_height;

            match sline.cmp(&0) {
                std::cmp::Ordering::Less => {
                    // Cursor line would go off top of screen if w_wrow was this high.
                    // Make cursor line the first line in the window.  If not enough
                    // room use w_skipcol.
                    win_mut(wp).w_wrow = line_size;
                    let new_wrow = win_ref(wp).w_wrow;
                    let view_width = win_ref(wp).w_view_width;
                    let col_off = nvim_win_col_off(wp);

                    if new_wrow >= view_height && (view_width - col_off) > 0 {
                        let skipcol = win_ref(wp).w_skipcol;
                        win_mut(wp).w_skipcol = skipcol + view_width - col_off;
                        win_mut(wp).w_wrow = new_wrow - 1;

                        loop {
                            let cur_wrow = win_ref(wp).w_wrow;
                            if cur_wrow < view_height {
                                break;
                            }
                            let col_off2 = nvim_win_col_off2(wp);
                            let skipcol = win_ref(wp).w_skipcol;
                            win_mut(wp).w_skipcol = skipcol + view_width - col_off + col_off2;
                            win_mut(wp).w_wrow = cur_wrow - 1;
                        }
                    }
                }
                std::cmp::Ordering::Greater => {
                    let mut line_size_mut = line_size;
                    while sline > 0 && lnum > 1 {
                        nvim_hasFolding(wp, lnum, ptr::addr_of_mut!(lnum), ptr::null_mut());
                        if lnum == 1 {
                            // first line in buffer is folded
                            line_size_mut =
                                i32::from(nvim_decor_conceal_line(wp, lnum - 1, 0) == 0);
                            sline -= 1;
                            break;
                        }
                        lnum -= 1;
                        let cur_topline = win_ref(wp).w_topline;
                        if lnum == cur_topline {
                            line_size_mut =
                                nvim_plines_win_nofill(wp, lnum, 1) + win_ref(wp).w_topfill;
                        } else {
                            line_size_mut = nvim_plines_win(wp, lnum, 1);
                        }
                        sline -= line_size_mut;
                    }

                    match sline.cmp(&0) {
                        std::cmp::Ordering::Less => {
                            // Line we want at top would go off top of screen.  Use next
                            // line instead.
                            nvim_hasFolding(wp, lnum, ptr::null_mut(), ptr::addr_of_mut!(lnum));
                            lnum += 1;
                            let cur_wrow = win_ref(wp).w_wrow;
                            win_mut(wp).w_wrow = cur_wrow - (line_size_mut + sline);
                        }
                        std::cmp::Ordering::Greater => {
                            // First line of file reached, use that as topline.
                            lnum = 1;
                            let cur_wrow = win_ref(wp).w_wrow;
                            win_mut(wp).w_wrow = cur_wrow - sline;
                        }
                        std::cmp::Ordering::Equal => {}
                    }
                }
                std::cmp::Ordering::Equal => {
                    // sline == 0, wrow already correct, no topline change needed
                }
            }

            nvim_set_topline(wp, lnum);
        }

        if is_curwin {
            nvim_curs_columns(wp, 0); // validate w_wrow
        }
        if prev_height > 0 {
            let final_wrow = win_ref(wp).w_wrow;
            win_mut(wp).w_prev_fraction_row = final_wrow;
        }

        nvim_redraw_later(wp, UPD_SOME_VALID);
        nvim_invalidate_botline(wp);
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Set w_fraction for the current w_wrow and w_view_height.
#[unsafe(no_mangle)]
pub extern "C" fn rs_set_fraction(wp: WinHandle) {
    set_fraction_impl(wp);
}

/// FFI: Return the default scroll amount for a window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_default_scroll(wp: WinHandle) -> c_longlong {
    win_default_scroll_impl(wp)
}

/// FFI: Scroll window to maintain cursor's relative fraction position after resize.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scroll_to_fraction(wp: WinHandle, prev_height: c_int) {
    scroll_to_fraction_impl(wp, prev_height);
}

/// C export: `scroll_to_fraction` — eliminates the C thin wrapper.
#[unsafe(export_name = "scroll_to_fraction")]
pub extern "C" fn scroll_to_fraction(wp: WinHandle, prev_height: c_int) {
    scroll_to_fraction_impl(wp, prev_height);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fraction_mult_value() {
        assert_eq!(FRACTION_MULT, 16384);
    }

    #[test]
    fn test_min_lines_value() {
        assert_eq!(MIN_LINES, 2);
    }

    #[test]
    fn test_upd_some_valid_value() {
        assert_eq!(UPD_SOME_VALID, 35);
    }

    #[test]
    fn test_null_set_fraction() {
        // Should not panic
        set_fraction_impl(WinHandle::null());
    }

    #[test]
    fn test_null_win_default_scroll() {
        assert_eq!(win_default_scroll_impl(WinHandle::null()), 1);
    }

    #[test]
    fn test_null_scroll_to_fraction() {
        // Should not panic
        scroll_to_fraction_impl(WinHandle::null(), 0);
    }
}
