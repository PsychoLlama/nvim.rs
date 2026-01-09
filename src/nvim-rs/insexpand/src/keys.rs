//! Completion key handling support.
//!
//! This module provides helper functions for handling keys during completion.
//! The actual key processing remains in C, but Rust provides utilities.

use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_compl_started() -> c_int;
}

// Key constants (from keycode definitions)
const CTRL_N: c_int = 14; // ^N
const CTRL_P: c_int = 16; // ^P
const CTRL_Y: c_int = 25; // ^Y
const CTRL_E: c_int = 5; // ^E
const BS: c_int = 8; // Backspace
const TAB: c_int = 9; // Tab
const CR: c_int = 13; // Carriage return / Enter
const ESC: c_int = 27; // Escape

/// Check if a key should navigate forward in completion.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_is_forward_key(key: c_int) -> c_int {
    c_int::from(key == CTRL_N || key == TAB)
}

/// Check if a key should navigate backward in completion.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_is_backward_key(key: c_int) -> c_int {
    c_int::from(key == CTRL_P)
}

/// Check if a key should accept the current completion.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_is_accept_key(key: c_int) -> c_int {
    c_int::from(key == CTRL_Y || key == CR)
}

/// Check if a key should cancel completion.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_is_cancel_key(key: c_int) -> c_int {
    c_int::from(key == CTRL_E || key == ESC)
}

/// Check if a key is backspace.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_is_bs_key(key: c_int) -> c_int {
    c_int::from(key == BS)
}

/// Check if a key should trigger completion restart.
///
/// Returns true for printable characters that should restart completion.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_is_printable(key: c_int) -> c_int {
    // Basic ASCII printable range (space to ~)
    c_int::from((32..=126).contains(&key))
}

/// Check if completion is active and should handle the key.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_should_handle() -> c_int {
    nvim_get_compl_started()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_constants() {
        assert_eq!(CTRL_N, 14);
        assert_eq!(CTRL_P, 16);
        assert_eq!(CTRL_Y, 25);
        assert_eq!(CTRL_E, 5);
        assert_eq!(BS, 8);
        assert_eq!(TAB, 9);
        assert_eq!(CR, 13);
        assert_eq!(ESC, 27);
    }

    #[test]
    fn test_forward_key() {
        unsafe {
            assert_eq!(rs_keys_is_forward_key(CTRL_N), 1);
            assert_eq!(rs_keys_is_forward_key(TAB), 1);
            assert_eq!(rs_keys_is_forward_key(CTRL_P), 0);
        }
    }

    #[test]
    fn test_backward_key() {
        unsafe {
            assert_eq!(rs_keys_is_backward_key(CTRL_P), 1);
            assert_eq!(rs_keys_is_backward_key(CTRL_N), 0);
        }
    }

    #[test]
    fn test_accept_key() {
        unsafe {
            assert_eq!(rs_keys_is_accept_key(CTRL_Y), 1);
            assert_eq!(rs_keys_is_accept_key(CR), 1);
            assert_eq!(rs_keys_is_accept_key(ESC), 0);
        }
    }

    #[test]
    fn test_cancel_key() {
        unsafe {
            assert_eq!(rs_keys_is_cancel_key(CTRL_E), 1);
            assert_eq!(rs_keys_is_cancel_key(ESC), 1);
            assert_eq!(rs_keys_is_cancel_key(CTRL_Y), 0);
        }
    }

    #[test]
    fn test_printable() {
        unsafe {
            assert_eq!(rs_keys_is_printable(65), 1); // 'A'
            assert_eq!(rs_keys_is_printable(32), 1); // space
            assert_eq!(rs_keys_is_printable(10), 0); // newline
        }
    }
}
