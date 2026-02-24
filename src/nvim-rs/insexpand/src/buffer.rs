//! Buffer name completion and buffer scanning support.
//!
//! This module provides helper functions for buffer name completion
//! and buffer scanning during keyword completion.
//! The core buffer operations remain in C.

#![allow(dead_code, unused_imports)]
use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_cont_status() -> c_int;

    // Compound accessor for buffer name completion
    fn nvim_get_next_bufname_token_impl();
}

// CTRL-X mode for buffer names
const CTRL_X_BUFNAMES: c_int = 18;

// Continuation status flags
const CONT_LOCAL: c_int = 32;

// =============================================================================
// Phase 2 (pass 4): get_next_bufname_token
// =============================================================================

/// Get the next buffer name completion matches.
///
/// Iterates over all loaded buffers looking for buffer names that start with
/// the current completion leader (compl_orig_text), and adds matching buffer
/// tail names to the completion list.
///
/// # Safety
/// Requires valid buffer list state; called from insert mode completion only.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_bufname_token() {
    nvim_get_next_bufname_token_impl();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_bufnames_constant() {
        assert_eq!(CTRL_X_BUFNAMES, 18);
    }

    #[test]
    fn test_cont_local_constant() {
        assert_eq!(CONT_LOCAL, 32);
    }
}
