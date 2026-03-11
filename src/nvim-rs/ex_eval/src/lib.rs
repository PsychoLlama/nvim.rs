//! Exception handling evaluation state for Neovim
//!
//! This module provides Rust implementations for checking exception handling
//! state during command execution.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::too_long_first_doc_paragraph)]

use std::os::raw::c_int;

// Direct access to C globals for exception state variables
extern "C" {
    static mut force_abort: bool;
    static mut did_emsg: c_int;
    static mut got_int: bool;
    static mut did_throw: bool;
    static mut trylevel: c_int;
    static mut emsg_silent: c_int;
    // cause_abort is file-local in C; accessed via nvim_get_cause_abort for now
    fn nvim_get_cause_abort() -> c_int;
}

/// FAIL constant from vim_defs.h
const FAIL: c_int = 0;

/// Returns true if a function with the "abort" flag should not be considered
/// ended on an error. Parsing commands is continued in order to find finally
/// clauses to be executed, and some errors in skipped commands are still reported.
#[export_name = "aborted_in_try"]
pub unsafe extern "C" fn aborted_in_try_impl() -> bool {
    force_abort
}

/// Returns true when immediately aborting on error, or when an interrupt
/// occurred or an exception was thrown but not caught.
///
/// Use for ":{range}call" to check whether an aborted function that does not
/// handle a range itself should be called again for the next line in the range.
#[export_name = "aborting"]
pub unsafe extern "C" fn aborting_impl() -> bool {
    (did_emsg != 0 && force_abort) || got_int || did_throw
}

/// Returns true if a command with a subcommand resulting in `retcode` should
/// abort the script processing.
#[export_name = "should_abort"]
pub unsafe extern "C" fn should_abort_impl(retcode: c_int) -> bool {
    (retcode == FAIL && trylevel != 0 && emsg_silent == 0) || aborting_impl()
}

/// Updates `force_abort` if `cause_abort` is set.
///
/// This is necessary to restore "force_abort" even before the throw point
/// for the error message has been reached.
#[export_name = "update_force_abort"]
pub unsafe extern "C" fn update_force_abort_impl() {
    if nvim_get_cause_abort() != 0 {
        force_abort = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fail_constant() {
        // FAIL should be 0 (matching vim_defs.h)
        assert_eq!(FAIL, 0);
    }

    #[test]
    fn test_fail_matches_c_false() {
        // FAIL == 0 should match C FALSE semantics
        assert_eq!(FAIL, 0);
        assert!(FAIL == 0);
    }

    #[test]
    fn test_fail_usable_in_conditions() {
        // FAIL should work correctly in boolean contexts
        let retcode = FAIL;
        let is_failure = retcode == FAIL;
        assert!(is_failure);

        let success = 1;
        let is_success = success != FAIL;
        assert!(is_success);
    }

    #[test]
    fn test_fail_distinct_from_success() {
        // FAIL (0) should be distinct from typical success values
        let ok: c_int = 1;
        assert_ne!(FAIL, ok);
    }
}
