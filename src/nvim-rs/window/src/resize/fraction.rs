//! Fraction and scroll default calculations.
//!
//! This module provides Rust implementations of `set_fraction` and
//! `win_default_scroll` from `src/nvim/window.c`.

use std::ffi::{c_int, c_longlong};

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

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_win_get_view_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_wrow(wp: WinHandle) -> c_int;
    fn nvim_win_set_fraction(wp: WinHandle, val: c_int);
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
        let view_height = nvim_win_get_view_height(wp);
        if view_height > 1 {
            let wrow = nvim_win_get_wrow(wp);
            let fraction = (wrow * FRACTION_MULT + FRACTION_MULT / 2) / view_height;
            nvim_win_set_fraction(wp, fraction);
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
        let view_height = c_longlong::from(nvim_win_get_view_height(wp));
        (view_height / 2).max(1)
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
    fn test_null_set_fraction() {
        // Should not panic
        set_fraction_impl(WinHandle::null());
    }

    #[test]
    fn test_null_win_default_scroll() {
        assert_eq!(win_default_scroll_impl(WinHandle::null()), 1);
    }
}
