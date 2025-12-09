//! Autocommand state checking for Neovim
//!
//! This module provides Rust implementations for checking autocommand state.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]

use std::os::raw::c_int;

// C accessor for the static autocmd_blocked variable
extern "C" {
    fn nvim_get_autocmd_blocked() -> c_int;
}

/// Check if autocommands are blocked.
///
/// Returns true if autocmd_blocked != 0.
#[no_mangle]
pub unsafe extern "C" fn rs_is_autocmd_blocked() -> c_int {
    c_int::from(nvim_get_autocmd_blocked() != 0)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Basic test to verify compilation
        assert!(true);
    }
}
