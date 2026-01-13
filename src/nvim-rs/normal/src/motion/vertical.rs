//! Vertical motion commands.
//!
//! This module provides helper functions for vertical motions:
//! - nv_up/down (j/k, arrows)
//! - nv_scroll (Ctrl-E/Y)
//! - nv_halfpage (Ctrl-U/D)
//! - nv_page (Ctrl-F/B)

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Scroll Direction Constants
// =============================================================================

/// Scroll up direction.
pub const SCROLL_UP: c_int = -1;
/// Scroll down direction.
pub const SCROLL_DOWN: c_int = 1;

// =============================================================================
// Vertical Motion Helpers (Pure Rust)
// =============================================================================

/// Calculate target line for up motion.
fn calc_up_target(lnum: c_int, count: c_int) -> c_int {
    (lnum - count).max(1)
}

/// Calculate target line for down motion.
fn calc_down_target(lnum: c_int, count: c_int, line_count: c_int) -> c_int {
    (lnum + count).min(line_count)
}

/// Calculate half-page scroll amount.
fn calc_halfpage_amount(win_height: c_int, scroll: c_int) -> c_int {
    if scroll > 0 {
        scroll
    } else {
        (win_height - 1) / 2
    }
}

/// Calculate full-page scroll amount.
fn calc_page_amount(win_height: c_int) -> c_int {
    (win_height - 2).max(1)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Calculate up motion target.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_up_target(lnum: c_int, count: c_int) -> c_int {
    calc_up_target(lnum, count)
}

/// FFI: Calculate down motion target.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_down_target(lnum: c_int, count: c_int, line_count: c_int) -> c_int {
    calc_down_target(lnum, count, line_count)
}

/// FFI: Calculate halfpage scroll amount.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_halfpage_amount(win_height: c_int, scroll: c_int) -> c_int {
    calc_halfpage_amount(win_height, scroll)
}

/// FFI: Calculate full page scroll amount.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_page_amount(win_height: c_int) -> c_int {
    calc_page_amount(win_height)
}

/// FFI: Get SCROLL_UP constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_scroll_up() -> c_int {
    SCROLL_UP
}

/// FFI: Get SCROLL_DOWN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_scroll_down() -> c_int {
    SCROLL_DOWN
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_constants() {
        assert_eq!(SCROLL_UP, -1);
        assert_eq!(SCROLL_DOWN, 1);
    }

    #[test]
    fn test_calc_up_target() {
        assert_eq!(calc_up_target(10, 3), 7);
        assert_eq!(calc_up_target(3, 5), 1);
        assert_eq!(calc_up_target(1, 1), 1);
    }

    #[test]
    fn test_calc_down_target() {
        assert_eq!(calc_down_target(5, 3, 100), 8);
        assert_eq!(calc_down_target(98, 5, 100), 100);
        assert_eq!(calc_down_target(100, 1, 100), 100);
    }

    #[test]
    fn test_calc_halfpage_amount() {
        // With explicit scroll value
        assert_eq!(calc_halfpage_amount(20, 10), 10);
        // Without scroll (use half window)
        assert_eq!(calc_halfpage_amount(20, 0), 9);
        assert_eq!(calc_halfpage_amount(21, 0), 10);
    }

    #[test]
    fn test_calc_page_amount() {
        assert_eq!(calc_page_amount(20), 18);
        assert_eq!(calc_page_amount(3), 1);
        assert_eq!(calc_page_amount(2), 1);
    }
}
