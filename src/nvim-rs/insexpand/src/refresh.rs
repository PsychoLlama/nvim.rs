//! Completion refresh and continuation support.
//!
//! This module provides helper functions for refreshing and continuing
//! completion searches. The core refresh logic remains in C, but Rust
//! provides utilities for state checking and leader comparison.

use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_was_interrupted() -> c_int;
    fn nvim_get_compl_opt_refresh_always() -> c_int;
    fn nvim_get_compl_matches() -> c_int;
    fn nvim_get_compl_length() -> c_int;
    fn nvim_get_compl_leader_data() -> *const c_char;
    fn nvim_get_compl_leader_size() -> usize;
    fn nvim_get_compl_orig_text_data() -> *const c_char;
    fn nvim_get_compl_orig_text_size() -> usize;

    // UTF-8 functions
    fn rs_utfc_ptr2len(ptr: *const c_char) -> c_int;
}

// CTRL-X mode constants
const CTRL_X_FUNCTION: c_int = 12;
const CTRL_X_OMNI: c_int = 13;

/// Check if completion needs to restart.
///
/// Returns true if completion was interrupted or refresh_always is set.
#[no_mangle]
pub unsafe extern "C" fn rs_refresh_needs_restart() -> c_int {
    let was_interrupted = nvim_get_compl_was_interrupted() != 0;
    let refresh_always = rs_refresh_always_active() != 0;
    c_int::from(was_interrupted || refresh_always)
}

/// Check if "refresh:always" is active.
///
/// Only applies to function and omni completion modes.
#[no_mangle]
pub unsafe extern "C" fn rs_refresh_always_active() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    let is_func_or_omni = mode == CTRL_X_FUNCTION || mode == CTRL_X_OMNI;
    c_int::from(is_func_or_omni && nvim_get_compl_opt_refresh_always() != 0)
}

/// Check if we should continue searching for more matches.
///
/// Returns true if completion is active and there's more to search.
#[no_mangle]
pub unsafe extern "C" fn rs_refresh_should_continue() -> c_int {
    if nvim_get_compl_started() == 0 {
        return 0;
    }
    // Continue if we were interrupted
    c_int::from(nvim_get_compl_was_interrupted() != 0)
}

/// Check if matches need filtering.
///
/// Returns true if the leader has changed and matches need to be re-filtered.
#[no_mangle]
pub unsafe extern "C" fn rs_refresh_needs_filter() -> c_int {
    if nvim_get_compl_started() == 0 {
        return 0;
    }
    // Need filter if leader differs from original text
    let leader_data = nvim_get_compl_leader_data();
    let leader_size = nvim_get_compl_leader_size();
    let orig_size = nvim_get_compl_orig_text_size();

    if leader_data.is_null() {
        return 0; // No leader, no filter needed
    }

    c_int::from(leader_size != orig_size)
}

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
        let char_len = rs_utfc_ptr2len(ptr.add(pos));
        if char_len <= 0 {
            break;
        }
        pos += char_len as usize;
        count += 1;
    }

    count
}

/// Check if leader has grown since completion started.
///
/// Returns true if more characters have been typed.
#[no_mangle]
pub unsafe extern "C" fn rs_refresh_leader_grew() -> c_int {
    let leader_size = nvim_get_compl_leader_size();
    let orig_size = nvim_get_compl_orig_text_size();
    c_int::from(leader_size > orig_size)
}

/// Check if leader has shrunk since completion started.
///
/// Returns true if characters have been deleted.
#[no_mangle]
pub unsafe extern "C" fn rs_refresh_leader_shrunk() -> c_int {
    let leader_size = nvim_get_compl_leader_size();
    let orig_size = nvim_get_compl_orig_text_size();
    c_int::from(leader_size < orig_size)
}

/// Check if there are any matches.
#[no_mangle]
pub unsafe extern "C" fn rs_refresh_has_matches() -> c_int {
    c_int::from(nvim_get_compl_matches() > 0)
}

/// Check if completion should be kept going.
///
/// Returns true if completion is active and has matches or is still searching.
#[no_mangle]
pub unsafe extern "C" fn rs_refresh_keep_going() -> c_int {
    if nvim_get_compl_started() == 0 {
        return 0;
    }
    let has_matches = nvim_get_compl_matches() > 0;
    let was_interrupted = nvim_get_compl_was_interrupted() != 0;
    c_int::from(has_matches || was_interrupted)
}

/// Get the minimum prefix length for completion.
///
/// Returns the value of compl_length.
#[no_mangle]
pub unsafe extern "C" fn rs_refresh_get_min_len() -> c_int {
    nvim_get_compl_length()
}

// =============================================================================
// Phase 8: Restart and Cleanup Helpers
// =============================================================================

// Additional C accessor functions
extern "C" {
    fn nvim_get_compl_cont_status() -> c_int;
    fn nvim_get_compl_cont_mode() -> c_int;
    fn nvim_get_compl_autocomplete() -> c_int;
    fn nvim_compl_first_match_is_null() -> c_int;
}

/// Check if completion restart is needed based on current state.
///
/// Returns true if either interrupted or refresh_always is active.
#[no_mangle]
pub unsafe extern "C" fn rs_restart_is_needed() -> c_int {
    let was_interrupted = nvim_get_compl_was_interrupted() != 0;
    let refresh_always = rs_refresh_always_active() != 0;
    c_int::from(was_interrupted || refresh_always)
}

/// Check if completion state should be fully reset.
///
/// Returns true when restarting in a mode that requires full reset.
#[no_mangle]
pub unsafe extern "C" fn rs_restart_needs_full_reset() -> c_int {
    // Full reset needed if continuation status is cleared
    c_int::from(nvim_get_compl_cont_status() == 0)
}

/// Get the continuation mode for restart.
#[no_mangle]
pub unsafe extern "C" fn rs_restart_get_cont_mode() -> c_int {
    nvim_get_compl_cont_mode()
}

/// Check if there's a first match that needs cleanup.
#[no_mangle]
pub unsafe extern "C" fn rs_cleanup_has_first_match() -> c_int {
    // Return 1 if first match exists (is NOT null)
    c_int::from(nvim_compl_first_match_is_null() == 0)
}

/// Check if completion was in autocomplete mode (needs special cleanup).
#[no_mangle]
pub unsafe extern "C" fn rs_cleanup_was_autocomplete() -> c_int {
    nvim_get_compl_autocomplete()
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

/// Check if the leader prefix changed (not just got longer/shorter at end).
///
/// Returns true if the beginning of leader doesn't match the beginning of original.
#[no_mangle]
pub unsafe extern "C" fn rs_restart_leader_prefix_changed() -> c_int {
    let leader_data = nvim_get_compl_leader_data();
    let orig_data = nvim_get_compl_orig_text_data();

    if leader_data.is_null() {
        return 0; // No leader, no change
    }
    if orig_data.is_null() {
        return 1; // Has leader but no original
    }

    let leader_size = nvim_get_compl_leader_size();
    let orig_size = nvim_get_compl_orig_text_size();

    // Compare up to the shorter length
    let compare_len = leader_size.min(orig_size);
    for i in 0..compare_len {
        if *leader_data.add(i) != *orig_data.add(i) {
            return 1; // Prefix changed
        }
    }

    0 // Prefix same
}

/// Check if completion should continue after restart.
///
/// Returns true if we have matches and completion was started.
#[no_mangle]
pub unsafe extern "C" fn rs_restart_should_continue() -> c_int {
    if nvim_get_compl_started() == 0 {
        return 0;
    }
    c_int::from(nvim_get_compl_matches() > 0)
}

/// Check if cleanup should update the screen.
///
/// Returns true when changes were made that affect display.
#[no_mangle]
pub unsafe extern "C" fn rs_cleanup_needs_redraw() -> c_int {
    // Need redraw if there are matches or first match exists
    let has_matches = nvim_get_compl_matches() > 0;
    let has_first = nvim_compl_first_match_is_null() == 0;
    c_int::from(has_matches || has_first)
}

/// Check if continuation status allows continuing.
#[no_mangle]
pub unsafe extern "C" fn rs_restart_cont_status_allows() -> c_int {
    c_int::from(nvim_get_compl_cont_status() != 0)
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
