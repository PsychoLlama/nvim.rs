//! Search-related utilities for Neovim
//!
//! This crate provides Rust implementations of search-related functions
//! from `src/nvim/search.c`. It uses an accessor pattern where
//! static variables are accessed through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::{c_char, c_int};

// C accessor functions for search state.
// These are defined in search.c and provide safe access to static variables.
extern "C" {
    /// Get the `lastcdir` static variable (FORWARD=1, BACKWARD=-1).
    fn nvim_get_lastcdir() -> c_int;

    /// Get the `last_t_cmd` static variable.
    fn nvim_get_last_t_cmd() -> c_int;

    /// Get the `lastc_bytes` static variable.
    fn nvim_get_lastc_bytes() -> *const c_char;

    /// Get the `last_idx` static variable.
    fn nvim_get_last_idx() -> c_int;

    /// Get the `had_eol` static variable from regexp.c.
    fn nvim_get_regexp_had_eol() -> c_int;
}

/// Direction constant for FORWARD.
const FORWARD: c_int = 1;

/// Check if last character search direction was forward.
///
/// This is the Rust equivalent of `last_csearch_forward()` in search.c.
#[inline]
fn last_csearch_forward_impl() -> bool {
    // SAFETY: nvim_get_lastcdir is a simple global accessor
    unsafe { nvim_get_lastcdir() == FORWARD }
}

/// FFI wrapper for `last_csearch_forward`.
///
/// Returns non-zero if the last search direction was forward.
#[no_mangle]
pub extern "C" fn rs_last_csearch_forward() -> c_int {
    c_int::from(last_csearch_forward_impl())
}

/// Check if last character search was a 't' command (until).
///
/// This is the Rust equivalent of `last_csearch_until()` in search.c.
#[inline]
fn last_csearch_until_impl() -> c_int {
    // SAFETY: nvim_get_last_t_cmd is a simple global accessor
    unsafe { nvim_get_last_t_cmd() }
}

/// FFI wrapper for `last_csearch_until`.
///
/// Returns non-zero if the last search was a 't' command.
#[no_mangle]
pub extern "C" fn rs_last_csearch_until() -> c_int {
    last_csearch_until_impl()
}

/// Get the last character search bytes.
///
/// Returns a pointer to the static `lastc_bytes` array.
///
/// # Safety
///
/// Calls external C function to get pointer to static variable.
#[no_mangle]
pub unsafe extern "C" fn rs_last_csearch() -> *const c_char {
    nvim_get_lastc_bytes()
}

/// Check if search pattern was the last used one.
///
/// Returns true if `last_idx == 0`, meaning the search pattern (not substitute)
/// was last used.
#[inline]
fn search_was_last_used_impl() -> bool {
    // SAFETY: nvim_get_last_idx is a simple global accessor
    unsafe { nvim_get_last_idx() == 0 }
}

/// FFI wrapper for `search_was_last_used`.
#[no_mangle]
pub extern "C" fn rs_search_was_last_used() -> c_int {
    c_int::from(search_was_last_used_impl())
}

/// Check if during the previous call to `vim_regcomp` the EOL item "$" was found.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_regcomp_had_eol() -> c_int {
    nvim_get_regexp_had_eol()
}

#[cfg(test)]
mod tests {
    // Note: Tests would need to mock the C accessor functions
}
