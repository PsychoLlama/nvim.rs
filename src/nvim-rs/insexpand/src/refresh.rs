//! Completion refresh and continuation support.
//!
//! This module provides helper functions for refreshing and continuing
//! completion searches. The core refresh logic remains in C, but Rust
//! provides utilities for state checking and leader comparison.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    fn nvim_get_compl_leader_data() -> *const c_char;
    fn nvim_get_compl_leader_size() -> usize;
    fn nvim_get_compl_orig_text_data() -> *const c_char;
    fn nvim_get_compl_orig_text_size() -> usize;

    // UTF-8 functions
    fn utfc_ptr2len(ptr: *const c_char) -> c_int;
}

// CTRL-X mode constants
const CTRL_X_FUNCTION: c_int = 12;
const CTRL_X_OMNI: c_int = 13;

/// Get the new leader length (characters typed since restart).
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_refresh_leader_char_count() -> c_int {
    let leader_data = nvim_get_compl_leader_data();
    if leader_data.is_null() {
        let orig_data = nvim_get_compl_orig_text_data();
        if orig_data.is_null() {
            return 0;
        }
        return rs_refresh_count_chars(orig_data, nvim_get_compl_orig_text_size());
    }
    rs_refresh_count_chars(leader_data, nvim_get_compl_leader_size())
}

/// Count the number of UTF-8 characters in a string.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_refresh_count_chars(ptr: *const c_char, len: usize) -> c_int {
    if ptr.is_null() || len == 0 {
        return 0;
    }

    let mut count = 0;
    let mut pos = 0usize;
    let end = len;

    while pos < end {
        let char_len = utfc_ptr2len(ptr.add(pos));
        if char_len <= 0 {
            break;
        }
        pos += char_len as usize;
        count += 1;
    }

    count
}

// =============================================================================
// Phase 8: Restart and Cleanup Helpers
// =============================================================================

// Additional C accessor functions
extern "C" {
    fn nvim_compl_first_match_is_null() -> c_int;
}

/// Calculate how much the leader has changed.
///
/// Returns the difference in size between current leader and original text.
/// Positive means leader grew, negative means it shrunk.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_restart_leader_diff() -> c_int {
    let leader_size = nvim_get_compl_leader_size();
    let orig_size = nvim_get_compl_orig_text_size();
    (leader_size as c_int) - (orig_size as c_int)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_constants() {
        assert_eq!(CTRL_X_FUNCTION, 12);
        assert_eq!(CTRL_X_OMNI, 13);
    }
}
