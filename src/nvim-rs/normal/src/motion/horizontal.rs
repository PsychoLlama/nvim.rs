//! Horizontal motion commands.
//!
//! This module provides helper functions for horizontal motions:
//! - nv_left/right (h/l, arrows)
//! - nv_home/end (Home/End keys)
//! - nv_pipe (| command)
//! - nv_dollar ($ command)
//! - nv_beginline (0, ^ commands)

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Beginline Constants
// =============================================================================

/// Move cursor to first white character (^ command).
pub const BL_WHITE: c_int = 1;
/// Stay in same column if possible.
pub const BL_SOL: c_int = 2;
/// Fix for 'startofline' option.
pub const BL_FIX: c_int = 4;

// =============================================================================
// Horizontal Motion Helpers (Pure Rust)
// =============================================================================

/// Check if motion should go to first non-blank.
fn should_go_to_first_nonblank(arg: c_int) -> bool {
    (arg & BL_WHITE) != 0
}

/// Check if motion should respect 'startofline'.
fn should_respect_sol(arg: c_int) -> bool {
    (arg & BL_FIX) != 0
}

/// Calculate target column for left motion.
fn calc_left_target(col: c_int, count: c_int) -> c_int {
    (col - count).max(0)
}

/// Calculate target column for right motion.
fn calc_right_target(col: c_int, count: c_int, line_len: c_int) -> c_int {
    (col + count).min(line_len.saturating_sub(1).max(0))
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Check if should go to first non-blank.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_should_first_nonblank(arg: c_int) -> c_int {
    c_int::from(should_go_to_first_nonblank(arg))
}

/// FFI: Check if should respect startofline.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_should_respect_sol(arg: c_int) -> c_int {
    c_int::from(should_respect_sol(arg))
}

/// FFI: Get BL_WHITE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_bl_white() -> c_int {
    BL_WHITE
}

/// FFI: Get BL_SOL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_bl_sol() -> c_int {
    BL_SOL
}

/// FFI: Get BL_FIX constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_bl_fix() -> c_int {
    BL_FIX
}

/// FFI: Calculate left motion target column.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_left_target(col: c_int, count: c_int) -> c_int {
    calc_left_target(col, count)
}

/// FFI: Calculate right motion target column.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_right_target(col: c_int, count: c_int, line_len: c_int) -> c_int {
    calc_right_target(col, count, line_len)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bl_constants() {
        assert_eq!(BL_WHITE, 1);
        assert_eq!(BL_SOL, 2);
        assert_eq!(BL_FIX, 4);
    }

    #[test]
    fn test_should_go_to_first_nonblank() {
        assert!(should_go_to_first_nonblank(BL_WHITE));
        assert!(should_go_to_first_nonblank(BL_WHITE | BL_FIX));
        assert!(!should_go_to_first_nonblank(BL_SOL));
        assert!(!should_go_to_first_nonblank(0));
    }

    #[test]
    fn test_should_respect_sol() {
        assert!(should_respect_sol(BL_FIX));
        assert!(should_respect_sol(BL_WHITE | BL_FIX));
        assert!(!should_respect_sol(BL_WHITE));
        assert!(!should_respect_sol(0));
    }

    #[test]
    fn test_left_target() {
        assert_eq!(calc_left_target(5, 3), 2);
        assert_eq!(calc_left_target(2, 5), 0);
        assert_eq!(calc_left_target(0, 1), 0);
    }

    #[test]
    fn test_right_target() {
        assert_eq!(calc_right_target(0, 3, 10), 3);
        assert_eq!(calc_right_target(5, 10, 8), 7);
        assert_eq!(calc_right_target(7, 1, 8), 7);
    }
}
