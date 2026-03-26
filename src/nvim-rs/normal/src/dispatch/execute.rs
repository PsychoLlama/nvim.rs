//! Normal mode command dispatch execution.
//!
//! This module provides helper functions for normal mode command execution,
//! including state checks and operator argument handling.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use super::types::{CmdArgHandle, OpArgHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // Global state
    fn nvim_get_VIsual_active() -> c_int;
    fn nvim_get_cmdwin_type() -> c_int;
}

// =============================================================================
// Operator Argument Helpers
// =============================================================================

/// Check if an operator is pending.
#[inline]
fn op_pending_impl(oap: OpArgHandle) -> bool {
    if oap.is_null() {
        return false;
    }
    unsafe { (*oap.as_ptr()).op_type != 0 }
}

/// Clear pending operator.
#[inline]
fn clear_op_impl(oap: OpArgHandle) {
    if !oap.is_null() {
        unsafe { (*oap.as_ptr()).op_type = 0 };
    }
}

/// Get operator type.
#[inline]
fn get_op_type_impl(oap: OpArgHandle) -> c_int {
    if oap.is_null() {
        return 0;
    }
    unsafe { (*oap.as_ptr()).op_type }
}

/// Set operator type.
#[inline]
fn set_op_type_impl(oap: OpArgHandle, val: c_int) {
    if !oap.is_null() {
        unsafe { (*oap.as_ptr()).op_type = val };
    }
}

/// Get motion type.
#[inline]
fn get_motion_type_impl(oap: OpArgHandle) -> c_int {
    if oap.is_null() {
        return -1;
    }
    unsafe { (*oap.as_ptr()).motion_type }
}

/// Set motion type.
#[inline]
fn set_motion_type_impl(oap: OpArgHandle, val: c_int) {
    if !oap.is_null() {
        unsafe { (*oap.as_ptr()).motion_type = val };
    }
}

/// Get inclusive flag.
#[inline]
fn get_inclusive_impl(oap: OpArgHandle) -> bool {
    if oap.is_null() {
        return false;
    }
    unsafe { (*oap.as_ptr()).inclusive }
}

/// Set inclusive flag.
#[inline]
fn set_inclusive_impl(oap: OpArgHandle, val: bool) {
    if !oap.is_null() {
        unsafe { (*oap.as_ptr()).inclusive = val };
    }
}

/// Get register name.
#[inline]
fn get_regname_impl(oap: OpArgHandle) -> c_int {
    if oap.is_null() {
        return 0;
    }
    unsafe { (*oap.as_ptr()).regname }
}

/// Set register name.
#[inline]
fn set_regname_impl(oap: OpArgHandle, val: c_int) {
    if !oap.is_null() {
        unsafe { (*oap.as_ptr()).regname = val };
    }
}

/// Get motion force character.
#[inline]
fn get_motion_force_impl(oap: OpArgHandle) -> c_int {
    if oap.is_null() {
        return 0;
    }
    unsafe { (*oap.as_ptr()).motion_force }
}

/// Set motion force character.
#[inline]
unsafe fn set_motion_force_impl(oap: OpArgHandle, val: c_int) {
    if !oap.is_null() {
        (*oap.as_ptr()).motion_force = val;
    }
}

// =============================================================================
// Command Argument Helpers
// =============================================================================

/// Get operator args from cmdarg.
#[inline]
fn cap_get_oap_impl(cap: CmdArgHandle) -> OpArgHandle {
    if cap.is_null() {
        return OpArgHandle::null();
    }
    unsafe {
        let oap_ptr = (*cap.as_ptr()).oap;
        if oap_ptr.is_null() {
            OpArgHandle::null()
        } else {
            OpArgHandle::from_ptr(oap_ptr)
        }
    }
}

/// Get command character from cmdarg.
#[inline]
fn cap_get_cmdchar_impl(cap: CmdArgHandle) -> c_int {
    if cap.is_null() {
        return 0;
    }
    unsafe { (*cap.as_ptr()).cmdchar }
}

/// Get second character from cmdarg.
#[inline]
fn cap_get_nchar_impl(cap: CmdArgHandle) -> c_int {
    if cap.is_null() {
        return 0;
    }
    unsafe { (*cap.as_ptr()).nchar }
}

/// Get count0 from cmdarg.
#[inline]
fn cap_get_count0_impl(cap: CmdArgHandle) -> c_int {
    if cap.is_null() {
        return 0;
    }
    unsafe { (*cap.as_ptr()).count0 }
}

/// Get count1 from cmdarg.
#[inline]
fn cap_get_count1_impl(cap: CmdArgHandle) -> c_int {
    if cap.is_null() {
        return 1;
    }
    unsafe { (*cap.as_ptr()).count1 }
}

/// Get arg from cmdarg.
#[inline]
fn cap_get_arg_impl(cap: CmdArgHandle) -> c_int {
    if cap.is_null() {
        return 0;
    }
    unsafe { (*cap.as_ptr()).arg }
}

/// Get retval from cmdarg.
#[inline]
fn cap_get_retval_impl(cap: CmdArgHandle) -> c_int {
    if cap.is_null() {
        return 0;
    }
    unsafe { (*cap.as_ptr()).retval }
}

// =============================================================================
// Global State Helpers
// =============================================================================

/// Check if visual mode is active.
#[inline]
fn is_visual_active_impl() -> bool {
    unsafe { nvim_get_VIsual_active() != 0 }
}

/// Check if in command-line window.
#[inline]
fn in_cmdline_win_impl() -> bool {
    unsafe { nvim_get_cmdwin_type() != 0 }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Check if operator pending.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_op_pending(oap: OpArgHandle) -> c_int {
    c_int::from(op_pending_impl(oap))
}

/// FFI: Clear pending operator.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_clear_op(oap: OpArgHandle) {
    clear_op_impl(oap);
}

/// FFI: Get operator type.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_get_op_type(oap: OpArgHandle) -> c_int {
    get_op_type_impl(oap)
}

/// FFI: Set operator type.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_set_op_type(oap: OpArgHandle, val: c_int) {
    set_op_type_impl(oap, val);
}

/// FFI: Get motion type.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_get_motion_type(oap: OpArgHandle) -> c_int {
    get_motion_type_impl(oap)
}

/// FFI: Set motion type.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_set_motion_type(oap: OpArgHandle, val: c_int) {
    set_motion_type_impl(oap, val);
}

/// FFI: Get inclusive flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_get_inclusive(oap: OpArgHandle) -> c_int {
    c_int::from(get_inclusive_impl(oap))
}

/// FFI: Set inclusive flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_set_inclusive(oap: OpArgHandle, val: c_int) {
    set_inclusive_impl(oap, val != 0);
}

/// FFI: Get register name.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_get_regname(oap: OpArgHandle) -> c_int {
    get_regname_impl(oap)
}

/// FFI: Set register name.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_set_regname(oap: OpArgHandle, val: c_int) {
    set_regname_impl(oap, val);
}

/// FFI: Get motion force.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_get_motion_force(oap: OpArgHandle) -> c_int {
    get_motion_force_impl(oap)
}

/// FFI: Set motion force.
///
/// # Safety
/// `oap` must be a valid `oparg_T` pointer or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_exec_set_motion_force(oap: OpArgHandle, val: c_int) {
    set_motion_force_impl(oap, val);
}

/// FFI: Get operator args from cmdarg.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cap_get_oap(cap: CmdArgHandle) -> OpArgHandle {
    cap_get_oap_impl(cap)
}

/// FFI: Get command character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cap_get_cmdchar(cap: CmdArgHandle) -> c_int {
    cap_get_cmdchar_impl(cap)
}

/// FFI: Get second character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cap_get_nchar(cap: CmdArgHandle) -> c_int {
    cap_get_nchar_impl(cap)
}

/// FFI: Get count0.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cap_get_count0(cap: CmdArgHandle) -> c_int {
    cap_get_count0_impl(cap)
}

/// FFI: Get count1.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cap_get_count1(cap: CmdArgHandle) -> c_int {
    cap_get_count1_impl(cap)
}

/// FFI: Get arg.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cap_get_arg(cap: CmdArgHandle) -> c_int {
    cap_get_arg_impl(cap)
}

/// FFI: Get retval.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cap_get_retval(cap: CmdArgHandle) -> c_int {
    cap_get_retval_impl(cap)
}

/// FFI: Check if visual mode active.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_visual_active() -> c_int {
    c_int::from(is_visual_active_impl())
}

/// FFI: Check if in command-line window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exec_in_cmdline_win() -> c_int {
    c_int::from(in_cmdline_win_impl())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_oparg() {
        let oap = OpArgHandle::null();
        assert!(!op_pending_impl(oap));
        assert_eq!(get_op_type_impl(oap), 0);
        assert_eq!(get_motion_type_impl(oap), -1);
        assert!(!get_inclusive_impl(oap));
    }

    #[test]
    fn test_null_cmdarg() {
        let cap = CmdArgHandle::null();
        assert_eq!(cap_get_cmdchar_impl(cap), 0);
        assert_eq!(cap_get_count0_impl(cap), 0);
        assert_eq!(cap_get_count1_impl(cap), 1);
        assert!(cap_get_oap_impl(cap).is_null());
    }
}
