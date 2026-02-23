//! User-defined and omni completion support.
//!
//! This module provides helper functions for user-defined function completion
//! (CTRL-X CTRL-U) and omni completion (CTRL-X CTRL-O).
//! The core function call operations remain in C.

#![allow(dead_code, unused_imports)]
use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
}

// CTRL-X mode constants
const CTRL_X_FUNCTION: c_int = 10;
const CTRL_X_OMNI: c_int = 11;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(CTRL_X_FUNCTION, 10);
        assert_eq!(CTRL_X_OMNI, 11);
    }
}
