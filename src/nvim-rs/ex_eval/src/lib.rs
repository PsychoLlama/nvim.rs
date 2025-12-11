//! Exception handling evaluation state for Neovim
//!
//! This module provides Rust implementations for checking exception handling
//! state during command execution.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]

use std::os::raw::c_int;

// C accessors for global exception state variables
extern "C" {
    fn nvim_get_force_abort() -> c_int;
    fn nvim_get_did_emsg() -> c_int;
    fn nvim_get_got_int() -> c_int;
    fn nvim_get_did_throw() -> c_int;
    fn nvim_get_trylevel() -> c_int;
    fn nvim_get_emsg_silent() -> c_int;
}

/// FAIL constant from vim_defs.h
const FAIL: c_int = 0;

/// Check if a function with the "abort" flag should not be considered
/// ended on an error.
///
/// Returns the value of the global `force_abort` variable.
#[no_mangle]
pub unsafe extern "C" fn rs_aborted_in_try() -> c_int {
    nvim_get_force_abort()
}

/// Check if execution should be aborted.
///
/// Returns true if:
/// - An error message was shown AND `force_abort` is set, OR
/// - An interrupt occurred (`got_int`), OR
/// - An exception was thrown but not caught (`did_throw`)
///
/// This is the Rust equivalent of `aborting()` in `ex_eval.c`.
#[no_mangle]
pub unsafe extern "C" fn rs_aborting() -> c_int {
    let did_emsg = nvim_get_did_emsg() != 0;
    let force_abort = nvim_get_force_abort() != 0;
    let got_int = nvim_get_got_int() != 0;
    let did_throw = nvim_get_did_throw() != 0;

    c_int::from((did_emsg && force_abort) || got_int || did_throw)
}

/// Check if a command with a subcommand resulting in `retcode` should
/// abort the script processing.
///
/// Returns true if:
/// - `retcode` == FAIL AND `trylevel` != 0 AND `emsg_silent` == 0, OR
/// - `aborting()` returns true
///
/// This is the Rust equivalent of `should_abort()` in `ex_eval.c`.
#[no_mangle]
pub unsafe extern "C" fn rs_should_abort(retcode: c_int) -> c_int {
    let trylevel = nvim_get_trylevel();
    let emsg_silent = nvim_get_emsg_silent();

    let fail_condition = retcode == FAIL && trylevel != 0 && emsg_silent == 0;
    let aborting = rs_aborting() != 0;

    c_int::from(fail_condition || aborting)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Basic test to verify compilation
        assert!(true);
    }
}
