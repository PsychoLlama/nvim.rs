//! Exception handling evaluation state for Neovim
//!
//! This module provides Rust implementations for checking exception handling
//! state during command execution.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]

use std::os::raw::c_int;

// C accessor for the global force_abort variable
extern "C" {
    fn nvim_get_force_abort() -> c_int;
}

/// Check if a function with the "abort" flag should not be considered
/// ended on an error.
///
/// Returns the value of the global force_abort variable.
#[no_mangle]
pub unsafe extern "C" fn rs_aborted_in_try() -> c_int {
    nvim_get_force_abort()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Basic test to verify compilation
        assert!(true);
    }
}
