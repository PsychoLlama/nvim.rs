//! Command-line and register completion support.
//!
//! This module provides helper functions for command-line completion (CTRL-X CTRL-V)
//! and register completion (CTRL-X CTRL-R).
//! The core operations remain in C.

#![allow(dead_code, unused_imports)]
use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
}

// CTRL-X mode constants
const CTRL_X_CMDLINE: c_int = 8;
const CTRL_X_REGISTER: c_int = 17;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(CTRL_X_CMDLINE, 8);
        assert_eq!(CTRL_X_REGISTER, 17);
    }
}
