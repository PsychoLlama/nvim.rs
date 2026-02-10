//! Command execution infrastructure for Ex commands.
//!
//! This module provides types and utilities for the command execution
//! infrastructure, including the ExArg structure, security checks,
//! and command execution state management.

use std::ffi::{c_char, c_int};

use crate::ExArgHandle;

// =============================================================================
// EXFLAG constants for command flags
// =============================================================================

/// Flag for 'l': list output format
pub const EXFLAG_LIST: c_int = 0x01;
/// Flag for '#': number output format
pub const EXFLAG_NR: c_int = 0x02;
/// Flag for 'p': print output format
pub const EXFLAG_PRINT: c_int = 0x04;

// =============================================================================
// FFI declarations for C globals and helpers
// =============================================================================

extern "C" {
    fn nvim_get_sandbox() -> c_int;
    fn nvim_get_secure() -> c_int;
    fn nvim_set_secure(val: c_int);
    fn nvim_emsg(s: *const c_char);
    fn nvim_get_e_curdir() -> *const c_char;
    fn nvim_get_e_sandbox() -> *const c_char;

    fn nvim_eap_get_arg(eap: ExArgHandle) -> *mut c_char;
    fn nvim_eap_set_arg(eap: ExArgHandle, arg: *mut c_char);
    fn nvim_eap_get_flags(eap: ExArgHandle) -> c_int;
    fn nvim_eap_set_flags(eap: ExArgHandle, flags: c_int);
    fn skipwhite(p: *const c_char) -> *mut c_char;
}

// =============================================================================
// Security check utilities
// =============================================================================

/// Check if the sandbox is active.
///
/// Returns true if `sandbox != 0`, meaning operations that would
/// modify the system are disallowed.
#[inline]
pub fn in_sandbox() -> bool {
    unsafe { nvim_get_sandbox() != 0 }
}

/// FFI wrapper for sandbox check.
#[no_mangle]
pub extern "C" fn rs_in_sandbox() -> c_int {
    c_int::from(in_sandbox())
}

/// Check if secure mode is active.
///
/// Returns true if `secure != 0`, meaning operations that would
/// access files or run commands are restricted.
#[inline]
pub fn is_secure() -> bool {
    unsafe { nvim_get_secure() != 0 }
}

/// FFI wrapper for secure mode check.
#[no_mangle]
pub extern "C" fn rs_is_secure() -> c_int {
    c_int::from(is_secure())
}

/// Check if secure mode prevents an operation.
///
/// If secure mode is active, sets `secure = 2` and emits an error message.
/// Also checks sandbox mode.
///
/// Returns true if the operation is disallowed (error was emitted).
///
/// # Safety
///
/// Calls external C functions to access global state and emit errors.
#[no_mangle]
pub unsafe extern "C" fn rs_check_secure() -> c_int {
    // Check secure flag first
    if nvim_get_secure() != 0 {
        nvim_set_secure(2);
        nvim_emsg(nvim_get_e_curdir());
        return 1;
    }

    // Check sandbox mode
    if nvim_get_sandbox() != 0 {
        nvim_emsg(nvim_get_e_sandbox());
        return 1;
    }

    0
}

// =============================================================================
// Command execution state helpers
// =============================================================================

/// Check if an EXFLAG bit is set.
#[inline]
pub const fn has_exflag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Check if the list flag is set.
#[inline]
pub const fn has_list_flag(flags: c_int) -> bool {
    has_exflag(flags, EXFLAG_LIST)
}

/// Check if the number flag is set.
#[inline]
pub const fn has_nr_flag(flags: c_int) -> bool {
    has_exflag(flags, EXFLAG_NR)
}

/// Check if the print flag is set.
#[inline]
pub const fn has_print_flag(flags: c_int) -> bool {
    has_exflag(flags, EXFLAG_PRINT)
}

/// FFI wrapper for EXFLAG checking.
#[no_mangle]
pub extern "C" fn rs_has_exflag(flags: c_int, flag: c_int) -> c_int {
    c_int::from(has_exflag(flags, flag))
}

/// Check if the list output flag is set.
#[no_mangle]
pub extern "C" fn rs_has_list_flag(flags: c_int) -> c_int {
    c_int::from(has_list_flag(flags))
}

/// Check if the number output flag is set.
#[no_mangle]
pub extern "C" fn rs_has_nr_flag(flags: c_int) -> c_int {
    c_int::from(has_nr_flag(flags))
}

/// Check if the print output flag is set.
#[no_mangle]
pub extern "C" fn rs_has_print_flag(flags: c_int) -> c_int {
    c_int::from(has_print_flag(flags))
}

// =============================================================================
// Bang (!) handling
// =============================================================================

/// Check if force (!) was used with the command.
///
/// Many Ex commands support a `!` suffix to force the operation
/// (e.g., `:quit!` to quit without saving).
#[inline]
pub const fn is_forced(forceit: c_int) -> bool {
    forceit != 0
}

/// FFI wrapper for force check.
#[no_mangle]
pub extern "C" fn rs_is_forced(forceit: c_int) -> c_int {
    c_int::from(is_forced(forceit))
}

// =============================================================================
// Skip mode handling
// =============================================================================

/// Check if command should be skipped (only parsed, not executed).
///
/// This is used during conditional statement parsing (`:if`, `:while`, etc.)
/// when the condition is false and commands should not be executed.
#[inline]
pub const fn is_skip_mode(skip: c_int) -> bool {
    skip != 0
}

/// FFI wrapper for skip mode check.
#[no_mangle]
pub extern "C" fn rs_is_skip_mode(skip: c_int) -> c_int {
    c_int::from(is_skip_mode(skip))
}

// =============================================================================
// Address/range validation
// =============================================================================

/// Check if command has any address (range) specified.
#[inline]
pub const fn has_range(addr_count: c_int) -> bool {
    addr_count > 0
}

/// Check if command has a single address.
#[inline]
pub const fn has_single_addr(addr_count: c_int) -> bool {
    addr_count == 1
}

/// Check if command has a line range (two addresses).
#[inline]
pub const fn has_line_range(addr_count: c_int) -> bool {
    addr_count >= 2
}

/// FFI wrapper for range check.
#[no_mangle]
pub extern "C" fn rs_has_range(addr_count: c_int) -> c_int {
    c_int::from(has_range(addr_count))
}

/// FFI wrapper for single address check.
#[no_mangle]
pub extern "C" fn rs_has_single_addr(addr_count: c_int) -> c_int {
    c_int::from(has_single_addr(addr_count))
}

/// FFI wrapper for line range check.
#[no_mangle]
pub extern "C" fn rs_has_line_range(addr_count: c_int) -> c_int {
    c_int::from(has_line_range(addr_count))
}

/// Check if the line range is valid (line1 <= line2).
#[inline]
pub const fn valid_line_range(line1: i64, line2: i64) -> bool {
    line1 <= line2
}

/// FFI wrapper for line range validation.
#[no_mangle]
pub extern "C" fn rs_valid_line_range(line1: i64, line2: i64) -> c_int {
    c_int::from(valid_line_range(line1, line2))
}

// =============================================================================
// get_flags - Parse l/p/# flags from command arguments
// =============================================================================

/// Parse `l`, `p`, `#` flags from the current argument position.
///
/// Sets corresponding EXFLAG bits and advances `eap->arg` past the flags.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[no_mangle]
pub unsafe extern "C" fn rs_get_flags(eap: ExArgHandle) {
    if eap.is_null() {
        return;
    }

    loop {
        let arg = nvim_eap_get_arg(eap);
        let c = *arg as u8;

        let flag = match c {
            b'l' => EXFLAG_LIST,
            b'p' => EXFLAG_PRINT,
            b'#' => EXFLAG_NR,
            _ => break,
        };

        let flags = nvim_eap_get_flags(eap);
        nvim_eap_set_flags(eap, flags | flag);
        nvim_eap_set_arg(eap, skipwhite(arg.add(1) as *const c_char));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exflags() {
        assert!(has_list_flag(EXFLAG_LIST));
        assert!(!has_list_flag(EXFLAG_NR));
        assert!(!has_list_flag(0));

        assert!(has_nr_flag(EXFLAG_NR));
        assert!(!has_nr_flag(EXFLAG_LIST));

        assert!(has_print_flag(EXFLAG_PRINT));
        assert!(!has_print_flag(EXFLAG_LIST));

        // Combined flags
        let combined = EXFLAG_LIST | EXFLAG_NR;
        assert!(has_list_flag(combined));
        assert!(has_nr_flag(combined));
        assert!(!has_print_flag(combined));
    }

    #[test]
    fn test_force_check() {
        assert!(is_forced(1));
        assert!(!is_forced(0));
    }

    #[test]
    fn test_skip_mode() {
        assert!(is_skip_mode(1));
        assert!(!is_skip_mode(0));
    }

    #[test]
    fn test_range_checks() {
        assert!(!has_range(0));
        assert!(has_range(1));
        assert!(has_range(2));

        assert!(!has_single_addr(0));
        assert!(has_single_addr(1));
        assert!(!has_single_addr(2));

        assert!(!has_line_range(0));
        assert!(!has_line_range(1));
        assert!(has_line_range(2));
        assert!(has_line_range(3));
    }

    #[test]
    fn test_valid_line_range() {
        assert!(valid_line_range(1, 5));
        assert!(valid_line_range(1, 1));
        assert!(!valid_line_range(5, 1));
        assert!(valid_line_range(0, 0));
    }

    #[test]
    fn test_exflag_constants() {
        // Verify flag values don't overlap
        assert_eq!(EXFLAG_LIST & EXFLAG_NR, 0);
        assert_eq!(EXFLAG_LIST & EXFLAG_PRINT, 0);
        assert_eq!(EXFLAG_NR & EXFLAG_PRINT, 0);

        // Verify expected values
        assert_eq!(EXFLAG_LIST, 0x01);
        assert_eq!(EXFLAG_NR, 0x02);
        assert_eq!(EXFLAG_PRINT, 0x04);
    }

    #[test]
    fn test_ffi_wrappers() {
        assert_eq!(rs_has_exflag(EXFLAG_LIST, EXFLAG_LIST), 1);
        assert_eq!(rs_has_exflag(EXFLAG_LIST, EXFLAG_NR), 0);

        assert_eq!(rs_has_list_flag(EXFLAG_LIST), 1);
        assert_eq!(rs_has_list_flag(0), 0);

        assert_eq!(rs_is_forced(1), 1);
        assert_eq!(rs_is_forced(0), 0);

        assert_eq!(rs_is_skip_mode(1), 1);
        assert_eq!(rs_is_skip_mode(0), 0);

        assert_eq!(rs_has_range(1), 1);
        assert_eq!(rs_has_range(0), 0);

        assert_eq!(rs_has_single_addr(1), 1);
        assert_eq!(rs_has_single_addr(2), 0);

        assert_eq!(rs_has_line_range(2), 1);
        assert_eq!(rs_has_line_range(1), 0);

        assert_eq!(rs_valid_line_range(1, 5), 1);
        assert_eq!(rs_valid_line_range(5, 1), 0);
    }
}
