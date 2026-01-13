//! Search motion commands.
//!
//! This module provides helper functions for search motions:
//! - nv_next (n/N)
//! - nv_search (/, ?)
//! - nv_csearch (f/F/t/T)
//! - nv_brackets ([, ])

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Search Direction Constants
// =============================================================================

/// Search forward direction.
pub const SEARCH_FORWARD: c_int = 1;
/// Search backward direction.
pub const SEARCH_BACKWARD: c_int = -1;

// =============================================================================
// Character Search Constants
// =============================================================================

/// Find character (f command).
pub const CSEARCH_FIND: c_int = 0;
/// Find character and stop before (t command).
pub const CSEARCH_TILL: c_int = 1;
/// Find character backward (F command).
pub const CSEARCH_FIND_BACK: c_int = 2;
/// Find character backward and stop after (T command).
pub const CSEARCH_TILL_BACK: c_int = 3;

// =============================================================================
// Bracket Motion Constants
// =============================================================================

/// Opening bracket for [ motions.
pub const BRACKET_OPEN: c_int = 0;
/// Closing bracket for ] motions.
pub const BRACKET_CLOSE: c_int = 1;

// =============================================================================
// Search Direction Helpers (Pure Rust)
// =============================================================================

/// Reverse search direction.
fn reverse_dir(dir: c_int) -> c_int {
    -dir
}

/// Check if direction is forward.
fn is_forward(dir: c_int) -> bool {
    dir > 0
}

/// Determine csearch type from command character.
fn get_csearch_type(cmdchar: c_int) -> c_int {
    if cmdchar == c_int::from(b'f') {
        CSEARCH_FIND
    } else if cmdchar == c_int::from(b't') {
        CSEARCH_TILL
    } else if cmdchar == c_int::from(b'F') {
        CSEARCH_FIND_BACK
    } else if cmdchar == c_int::from(b'T') {
        CSEARCH_TILL_BACK
    } else {
        CSEARCH_FIND
    }
}

/// Check if csearch is backward.
fn is_csearch_backward(csearch_type: c_int) -> bool {
    csearch_type >= CSEARCH_FIND_BACK
}

/// Check if csearch is till mode.
fn is_csearch_till(csearch_type: c_int) -> bool {
    csearch_type == CSEARCH_TILL || csearch_type == CSEARCH_TILL_BACK
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get SEARCH_FORWARD constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_search_forward() -> c_int {
    SEARCH_FORWARD
}

/// FFI: Get SEARCH_BACKWARD constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_search_backward() -> c_int {
    SEARCH_BACKWARD
}

/// FFI: Reverse search direction.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_reverse_dir(dir: c_int) -> c_int {
    reverse_dir(dir)
}

/// FFI: Check if direction is forward.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_is_forward(dir: c_int) -> c_int {
    c_int::from(is_forward(dir))
}

/// FFI: Get CSEARCH_FIND constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_csearch_find() -> c_int {
    CSEARCH_FIND
}

/// FFI: Get CSEARCH_TILL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_csearch_till() -> c_int {
    CSEARCH_TILL
}

/// FFI: Get CSEARCH_FIND_BACK constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_csearch_find_back() -> c_int {
    CSEARCH_FIND_BACK
}

/// FFI: Get CSEARCH_TILL_BACK constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_csearch_till_back() -> c_int {
    CSEARCH_TILL_BACK
}

/// FFI: Get csearch type from cmdchar.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_get_csearch_type(cmdchar: c_int) -> c_int {
    get_csearch_type(cmdchar)
}

/// FFI: Check if csearch is backward.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_is_csearch_backward(csearch_type: c_int) -> c_int {
    c_int::from(is_csearch_backward(csearch_type))
}

/// FFI: Check if csearch is till mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_is_csearch_till(csearch_type: c_int) -> c_int {
    c_int::from(is_csearch_till(csearch_type))
}

/// FFI: Get BRACKET_OPEN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_bracket_open() -> c_int {
    BRACKET_OPEN
}

/// FFI: Get BRACKET_CLOSE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_bracket_close() -> c_int {
    BRACKET_CLOSE
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_constants() {
        assert_eq!(SEARCH_FORWARD, 1);
        assert_eq!(SEARCH_BACKWARD, -1);
    }

    #[test]
    fn test_csearch_constants() {
        assert_eq!(CSEARCH_FIND, 0);
        assert_eq!(CSEARCH_TILL, 1);
        assert_eq!(CSEARCH_FIND_BACK, 2);
        assert_eq!(CSEARCH_TILL_BACK, 3);
    }

    #[test]
    fn test_bracket_constants() {
        assert_eq!(BRACKET_OPEN, 0);
        assert_eq!(BRACKET_CLOSE, 1);
    }

    #[test]
    fn test_reverse_dir() {
        assert_eq!(reverse_dir(SEARCH_FORWARD), SEARCH_BACKWARD);
        assert_eq!(reverse_dir(SEARCH_BACKWARD), SEARCH_FORWARD);
    }

    #[test]
    fn test_is_forward() {
        assert!(is_forward(SEARCH_FORWARD));
        assert!(!is_forward(SEARCH_BACKWARD));
        assert!(!is_forward(0));
    }

    #[test]
    fn test_get_csearch_type() {
        assert_eq!(get_csearch_type(c_int::from(b'f')), CSEARCH_FIND);
        assert_eq!(get_csearch_type(c_int::from(b't')), CSEARCH_TILL);
        assert_eq!(get_csearch_type(c_int::from(b'F')), CSEARCH_FIND_BACK);
        assert_eq!(get_csearch_type(c_int::from(b'T')), CSEARCH_TILL_BACK);
    }

    #[test]
    fn test_is_csearch_backward() {
        assert!(!is_csearch_backward(CSEARCH_FIND));
        assert!(!is_csearch_backward(CSEARCH_TILL));
        assert!(is_csearch_backward(CSEARCH_FIND_BACK));
        assert!(is_csearch_backward(CSEARCH_TILL_BACK));
    }

    #[test]
    fn test_is_csearch_till() {
        assert!(!is_csearch_till(CSEARCH_FIND));
        assert!(is_csearch_till(CSEARCH_TILL));
        assert!(!is_csearch_till(CSEARCH_FIND_BACK));
        assert!(is_csearch_till(CSEARCH_TILL_BACK));
    }
}
