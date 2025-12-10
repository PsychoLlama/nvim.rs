//! Lua executor state checking for Neovim
//!
//! This module provides Rust implementations for checking Lua executor state.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]

use std::os::raw::c_int;

// C accessor for the static in_fast_callback variable
extern "C" {
    fn nvim_get_in_fast_callback() -> c_int;
}

/// Check if the current execution context is safe for calling deferred API methods.
///
/// Luv callbacks are unsafe as they are called inside the uv loop.
/// Returns true if in_fast_callback == 0.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_is_deferred_safe() -> c_int {
    c_int::from(nvim_get_in_fast_callback() == 0)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Basic test to verify compilation
        assert!(true);
    }
}
