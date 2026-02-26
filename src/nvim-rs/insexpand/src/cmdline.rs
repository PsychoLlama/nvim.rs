//! Command-line and register completion support.
//!
//! This module provides helper functions for command-line completion (CTRL-X CTRL-V)
//! and register completion (CTRL-X CTRL-R).
//! The core operations remain in C.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;

    // Compound accessor for cmdline pattern setup
    fn nvim_get_cmdline_compl_info_impl(line: *mut c_char, curs_col: c_int) -> c_int;
}

// CTRL-X mode constants
const CTRL_X_CMDLINE: c_int = 11;
const CTRL_X_REGISTER: c_int = 19;

/// Get the pattern, column and length for command-line completion.
///
/// Sets `compl_col`, `compl_length`, and `compl_pattern` globals.
///
/// # Safety
/// `line` must be a valid C string. Requires valid global completion state.
#[no_mangle]
pub unsafe extern "C" fn rs_get_cmdline_compl_info(line: *mut c_char, curs_col: c_int) -> c_int {
    nvim_get_cmdline_compl_info_impl(line, curs_col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(CTRL_X_CMDLINE, 11);
        assert_eq!(CTRL_X_REGISTER, 19);
    }
}
