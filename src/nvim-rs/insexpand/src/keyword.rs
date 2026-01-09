//! Default (keyword) completion support.
//!
//! This module provides helper functions for default keyword completion
//! (CTRL-N / CTRL-P without CTRL-X prefix).
//! The core keyword scanning operations remain in C.

use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
}

// CTRL-X mode constants
const CTRL_X_NORMAL: c_int = 0;

// Direction constants
const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;

/// Check if we're in normal (keyword) completion mode.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_is_normal_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_NORMAL)
}

/// Check if completion is searching forward.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_is_forward() -> c_int {
    let dir = nvim_get_compl_direction();
    c_int::from(dir == FORWARD)
}

/// Check if completion is searching backward.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_is_backward() -> c_int {
    let dir = nvim_get_compl_direction();
    c_int::from(dir == BACKWARD)
}

/// Check if completion was interrupted during keyword search.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_was_interrupted() -> c_int {
    nvim_get_compl_interrupted()
}

/// Get the current completion direction for keyword search.
#[no_mangle]
pub unsafe extern "C" fn rs_keyword_get_direction() -> c_int {
    nvim_get_compl_direction()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
    }

    #[test]
    fn test_ctrl_x_mode_constant() {
        assert_eq!(CTRL_X_NORMAL, 0);
    }
}
