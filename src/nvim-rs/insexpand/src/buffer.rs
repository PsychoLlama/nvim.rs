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
}

// CTRL-X mode for buffer names
const CTRL_X_BUFNAMES: c_int = 18;

// Continuation status flags
const CONT_LOCAL: c_int = 32;

// =============================================================================
// Phase 3: Buffer Scanning Support Functions
// =============================================================================

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
