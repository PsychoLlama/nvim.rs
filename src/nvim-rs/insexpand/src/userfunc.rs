//! User-defined and omni completion support.
//!
//! This module provides helper functions for user-defined function completion
//! (CTRL-X CTRL-U) and omni completion (CTRL-X CTRL-O).
//! The core function call operations remain in C.

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

/// Check if we're in user-defined function completion mode.
#[no_mangle]
pub unsafe extern "C" fn rs_userfunc_is_function_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_FUNCTION)
}

/// Check if we're in omni completion mode.
#[no_mangle]
pub unsafe extern "C" fn rs_userfunc_is_omni_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_OMNI)
}

/// Check if we're in any user function mode (function or omni).
#[no_mangle]
pub unsafe extern "C" fn rs_userfunc_is_user_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_FUNCTION || mode == CTRL_X_OMNI)
}

/// Check if completion was interrupted during function call.
#[no_mangle]
pub unsafe extern "C" fn rs_userfunc_was_interrupted() -> c_int {
    nvim_get_compl_interrupted()
}

/// Get the current completion direction for function completion.
#[no_mangle]
pub unsafe extern "C" fn rs_userfunc_get_direction() -> c_int {
    nvim_get_compl_direction()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(CTRL_X_FUNCTION, 10);
        assert_eq!(CTRL_X_OMNI, 11);
    }
}
