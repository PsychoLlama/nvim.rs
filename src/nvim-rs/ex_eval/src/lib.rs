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

use std::ffi::{c_char, c_int, c_void};

// Re-export types used across modules
pub type OptInt = i64;
pub type LinenrT = i32;

// Direct access to C globals for exception state variables
extern "C" {
    static mut force_abort: bool;
    static mut did_emsg: c_int;
    static mut got_int: bool;
    static mut did_throw: bool;
    static mut trylevel: c_int;
    static mut emsg_silent: c_int;
    static mut current_exception: *mut c_void;
    static mut need_rethrow: bool;
    static mut msg_list: *mut *mut c_void;
    static mut p_verbose: OptInt;
    static mut debug_break_level: c_int;
}

// C functions callable from Rust
extern "C" {
    fn handle_did_throw();
    fn modifier_len(p: *mut c_char) -> c_int;
    fn tv_free(tv: *mut c_void);
    fn xfree(ptr: *mut c_void);
    fn semsg(fmt: *const c_char, ...);
    fn verbose_enter();
    fn verbose_leave();
    fn report_pending(action: c_int, pending: c_int, value: *mut c_void);
}

// Error strings accessible from C
extern "C" {
    static e_str_not_inside_function: *const c_char;
}

// Rust-owned static replacing the C file-local `static bool cause_abort`
static mut CAUSE_ABORT: bool = false;

// RP_ constants for report_pending
const RP_MAKE: c_int = 0;

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
    if CAUSE_ABORT {
        force_abort = true;
    }
}

/// Get the Rust-owned cause_abort value (for C callers that still reference it).
#[no_mangle]
pub unsafe extern "C" fn rs_get_cause_abort() -> bool {
    CAUSE_ABORT
}

/// Set the Rust-owned cause_abort value (for C callers that still reference it).
#[no_mangle]
pub unsafe extern "C" fn rs_set_cause_abort(val: bool) {
    CAUSE_ABORT = val;
}

/// Representation of exception_state_T matching C layout.
#[repr(C)]
pub struct ExceptionStateT {
    pub estate_current_exception: *mut c_void,
    pub estate_did_throw: bool,
    pub estate_need_rethrow: bool,
    pub estate_trylevel: c_int,
    pub estate_did_emsg: c_int,
}

/// Save the current exception state in "estate".
#[export_name = "exception_state_save"]
pub unsafe extern "C" fn exception_state_save(estate: *mut ExceptionStateT) {
    (*estate).estate_current_exception = current_exception;
    (*estate).estate_did_throw = did_throw;
    (*estate).estate_need_rethrow = need_rethrow;
    (*estate).estate_trylevel = trylevel;
    (*estate).estate_did_emsg = did_emsg;
}

/// Restore the current exception state from "estate".
#[export_name = "exception_state_restore"]
pub unsafe extern "C" fn exception_state_restore(estate: *mut ExceptionStateT) {
    // Handle any outstanding exceptions before restoring the state
    if did_throw {
        handle_did_throw();
    }
    current_exception = (*estate).estate_current_exception;
    did_throw = (*estate).estate_did_throw;
    need_rethrow = (*estate).estate_need_rethrow;
    trylevel = (*estate).estate_trylevel;
    did_emsg = (*estate).estate_did_emsg;
}

/// Clear the current exception state.
#[export_name = "exception_state_clear"]
pub unsafe extern "C" fn exception_state_clear() {
    current_exception = std::ptr::null_mut();
    did_throw = false;
    need_rethrow = false;
    trylevel = 0;
    did_emsg = 0;
}

/// Representation of msglist_T matching C layout.
#[repr(C)]
pub struct MsglistT {
    pub next: *mut MsglistT,
    pub msg: *mut c_char,
    pub throw_msg: *mut c_char,
    pub sfile: *mut c_char,
    pub slnum: LinenrT,
    pub multiline: bool,
}

/// Free a "msg_list" and the messages it contains.
#[no_mangle]
pub unsafe extern "C" fn free_msglist(l: *mut MsglistT) {
    let mut messages = l;
    while !messages.is_null() {
        let next = (*messages).next;
        xfree((*messages).msg.cast::<c_void>());
        xfree((*messages).sfile.cast::<c_void>());
        xfree(messages.cast::<c_void>());
        messages = next;
    }
}

/// Free global "*msg_list" and the messages it contains, then set "*msg_list" to NULL.
#[export_name = "free_global_msglist"]
pub unsafe extern "C" fn free_global_msglist_impl() {
    free_msglist((*msg_list).cast::<MsglistT>());
    *msg_list = std::ptr::null_mut();
}

/// Discard a pending return value (calls tv_free).
#[no_mangle]
pub unsafe extern "C" fn discard_pending_return(p: *mut c_void) {
    tv_free(p);
}

/// Handle ":endfunction" when not after a ":function".
#[export_name = "ex_endfunction"]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn ex_endfunction_impl(eap: *mut c_void) {
    let _ = eap;
    semsg(e_str_not_inside_function, c":endfunction".as_ptr());
}

/// Returns true if the string "p" looks like a ":while" or ":for" command.
#[export_name = "has_loop_cmd"]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub unsafe extern "C" fn has_loop_cmd(mut p: *mut c_char) -> bool {
    // skip modifiers, white space and ':'
    loop {
        while *p == b' ' as i8 || *p == b'\t' as i8 || *p == b':' as i8 {
            p = p.add(1);
        }
        let len = modifier_len(p);
        if len == 0 {
            break;
        }
        p = p.add(len as usize);
    }
    (*p == b'w' as i8 && *p.add(1) == b'h' as i8)
        || (*p == b'f' as i8 && *p.add(1) == b'o' as i8 && *p.add(2) == b'r' as i8)
}

/// Report information about something pending in a finally clause if required by
/// the 'verbose' option or when debugging, when something is made pending.
#[export_name = "report_make_pending"]
pub unsafe extern "C" fn report_make_pending_impl(pending: c_int, value: *mut c_void) {
    if p_verbose >= 14 || debug_break_level > 0 {
        if debug_break_level <= 0 {
            verbose_enter();
        }
        report_pending(RP_MAKE, pending, value);
        if debug_break_level <= 0 {
            verbose_leave();
        }
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
