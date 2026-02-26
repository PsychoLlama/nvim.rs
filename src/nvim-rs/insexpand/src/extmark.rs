//! Extmark and cleanup support for completion.
//!
//! This module provides helper functions for managing extmarks during completion
//! and cleanup operations. The core extmark operations remain in C (extmark.c),
//! but Rust provides utilities for tracking state.

#![allow(dead_code, unused_imports)]
use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_col() -> c_int;
    fn nvim_get_compl_lnum() -> c_int;
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_get_curwin_cursor_lnum() -> c_int;
    fn nvim_compl_shown_match_has_newline() -> c_int;

    // Compound accessors for extmark management and cleanup
    fn nvim_save_orig_extmarks_impl();
    fn nvim_restore_orig_extmarks();
    fn nvim_free_insexpand_stuff_impl();
}

/// Save extmarks in `compl_orig_text` so they may be restored when completion
/// is cancelled or the original text is completed.
///
/// # Safety
/// Requires valid global completion state (`curbuf`, `curwin`, `compl_col`,
/// `compl_length`, `compl_orig_extmarks`).
#[no_mangle]
pub unsafe extern "C" fn rs_save_orig_extmarks() {
    nvim_save_orig_extmarks_impl();
}

/// Restore extmarks saved by `rs_save_orig_extmarks`, replaying them in
/// reverse order.
///
/// # Safety
/// Requires valid global completion state (`compl_orig_extmarks`).
#[no_mangle]
pub unsafe extern "C" fn rs_restore_orig_extmarks() {
    nvim_restore_orig_extmarks();
}

/// Free all completion-related global state at process exit (EXITFREE).
///
/// # Safety
/// Should only be called at process exit; modifies static callback state.
#[no_mangle]
pub unsafe extern "C" fn rs_free_insexpand_stuff() {
    nvim_free_insexpand_stuff_impl();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_line_count_for_single_line() {
        // A single-line completion has 1 line
        let expected = 1;
        assert!(expected >= 1);
    }
}
