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

    /// Get the `magic_overruled` global value.
    fn nvim_get_magic_overruled() -> c_int;

    /// Get the `p_magic` global value.
    fn nvim_get_p_magic() -> c_int;
}

/// Direction constant for FORWARD.
const FORWARD: c_int = 1;

/// optmagic_T values from regexp_defs.h
#[allow(dead_code)]
const OPTION_MAGIC_NOT_SET: c_int = 0;
const OPTION_MAGIC_ON: c_int = 1;
const OPTION_MAGIC_OFF: c_int = 2;

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

/// Get the value of 'magic' taking "magic_overruled" into account.
///
/// This is the Rust equivalent of `magic_isset()` in option.c.
///
/// # Safety
/// Calls C accessor functions for global variables.
#[inline]
fn magic_isset_impl() -> bool {
    unsafe {
        match nvim_get_magic_overruled() {
            OPTION_MAGIC_ON => true,
            OPTION_MAGIC_OFF => false,
            _ => nvim_get_p_magic() != 0,
        }
    }
}

/// FFI wrapper for `magic_isset`.
///
/// Returns non-zero if magic is set.
#[no_mangle]
pub extern "C" fn rs_magic_isset() -> c_int {
    c_int::from(magic_isset_impl())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_constants() {
        // FORWARD should be 1 (matches search.c)
        assert_eq!(FORWARD, 1);
    }

    #[test]
    fn test_optmagic_constants() {
        // optmagic_T values from regexp_defs.h
        assert_eq!(OPTION_MAGIC_NOT_SET, 0);
        assert_eq!(OPTION_MAGIC_ON, 1);
        assert_eq!(OPTION_MAGIC_OFF, 2);
    }

    #[test]
    fn test_optmagic_distinct() {
        // Ensure all optmagic values are distinct
        let values = [OPTION_MAGIC_NOT_SET, OPTION_MAGIC_ON, OPTION_MAGIC_OFF];
        for i in 0..values.len() {
            for j in (i + 1)..values.len() {
                assert_ne!(
                    values[i], values[j],
                    "optmagic values at {i} and {j} should differ"
                );
            }
        }
    }

    #[test]
    fn test_optmagic_valid_for_match() {
        // Test that optmagic values work in match expressions
        let test_values = [OPTION_MAGIC_NOT_SET, OPTION_MAGIC_ON, OPTION_MAGIC_OFF];
        for val in test_values {
            let result = match val {
                OPTION_MAGIC_ON => true,
                OPTION_MAGIC_OFF => false,
                _ => false, // NOT_SET falls through
            };
            // OPTION_MAGIC_ON should return true, others false
            if val == OPTION_MAGIC_ON {
                assert!(result);
            }
        }
    }

    #[test]
    fn test_forward_backward_opposite() {
        // FORWARD is 1, BACKWARD is -1 (opposite sign)
        const BACKWARD: c_int = -1;
        let forward = FORWARD;
        let backward = BACKWARD;
        assert_eq!(forward, -backward);
        assert_eq!(forward + backward, 0);
    }

    #[test]
    fn test_optmagic_sequential() {
        // optmagic values should be sequential starting from 0
        assert_eq!(OPTION_MAGIC_NOT_SET, 0);
        assert_eq!(OPTION_MAGIC_ON, 1);
        assert_eq!(OPTION_MAGIC_OFF, 2);
        // Also verify they're in order
        let not_set = OPTION_MAGIC_NOT_SET;
        let on = OPTION_MAGIC_ON;
        let off = OPTION_MAGIC_OFF;
        assert!(not_set < on);
        assert!(on < off);
    }
}
