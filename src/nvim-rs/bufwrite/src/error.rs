//! Error handling for buffer write operations.
//!
//! Mirrors the C `Error_T` struct and provides `set_err_*` / `emit_err` functions.

use std::ffi::{c_char, c_int};
use std::ptr;

/// Mirrors the C `Error_T` struct layout.
///
/// Used to accumulate an error during `buf_write()` and emit it at the end.
#[repr(C)]
pub struct BwError {
    /// Optional error number string (e.g. "E502"), or null.
    pub num: *const c_char,
    /// Error message string, or null if no error.
    pub msg: *mut c_char,
    /// Errno value for `os_strerror()`, or 0.
    pub arg: c_int,
    /// Whether `msg` was dynamically allocated and needs `xfree()`.
    pub alloc: bool,
}

impl Default for BwError {
    fn default() -> Self {
        Self {
            num: ptr::null(),
            msg: ptr::null_mut(),
            arg: 0,
            alloc: false,
        }
    }
}

impl BwError {
    /// Create an error with a number prefix and message.
    #[must_use]
    pub const fn with_num(num: *const c_char, msg: *const c_char) -> Self {
        Self {
            num,
            msg: msg as *mut c_char,
            arg: 0,
            alloc: false,
        }
    }

    /// Create an error with just a message (or null to clear).
    #[must_use]
    pub const fn with_msg(msg: *const c_char) -> Self {
        Self {
            num: ptr::null(),
            msg: msg as *mut c_char,
            arg: 0,
            alloc: false,
        }
    }

    /// Create an error with a message and errno value.
    #[must_use]
    pub const fn with_msg_arg(msg: *const c_char, arg: c_int) -> Self {
        Self {
            num: ptr::null(),
            msg: msg as *mut c_char,
            arg,
            alloc: false,
        }
    }
}

extern "C" {
    fn nvim_bw_emsg(msg: *const c_char);
    fn nvim_bw_semsg_2(fmt: *const c_char, a: *const c_char, b: *const c_char);
    fn nvim_bw_semsg_3(
        fmt: *const c_char,
        a: *const c_char,
        b: *const c_char,
        c: *const c_char,
    );
    fn nvim_bw_semsg_4(
        fmt: *const c_char,
        a: *const c_char,
        b: *const c_char,
        c: *const c_char,
        d: *const c_char,
    );
    fn nvim_bw_os_strerror(errnum: c_int) -> *const c_char;
    fn nvim_bw_get_IObuff() -> *const c_char;
    fn nvim_bw_xfree(ptr: *mut c_char);
}

/// Emit the buffered error message, matching the C `emit_err` logic exactly.
///
/// # Safety
///
/// All pointers in `e` must be valid or null as appropriate.
unsafe fn emit_err_inner(e: &BwError) {
    if !e.num.is_null() {
        if e.arg != 0 {
            // semsg("%s: %s%s: %s", e->num, IObuff, e->msg, os_strerror(e->arg))
            let fmt = c"%s: %s%s: %s".as_ptr();
            let strerr = unsafe { nvim_bw_os_strerror(e.arg) };
            let iobuff = unsafe { nvim_bw_get_IObuff() };
            unsafe { nvim_bw_semsg_4(fmt, e.num, iobuff, e.msg, strerr) };
        } else {
            // semsg("%s: %s%s", e->num, IObuff, e->msg)
            let fmt = c"%s: %s%s".as_ptr();
            let iobuff = unsafe { nvim_bw_get_IObuff() };
            unsafe { nvim_bw_semsg_3(fmt, e.num, iobuff, e.msg) };
        }
    } else if e.arg != 0 {
        // semsg(e->msg, os_strerror(e->arg))
        let strerr = unsafe { nvim_bw_os_strerror(e.arg) };
        unsafe { nvim_bw_semsg_2(e.msg, strerr, ptr::null()) };
    } else {
        // emsg(e->msg)
        unsafe { nvim_bw_emsg(e.msg) };
    }
    if e.alloc {
        unsafe { nvim_bw_xfree(e.msg) };
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create an error with number and message.
///
/// Replaces C `set_err_num()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_set_err_num(num: *const c_char, msg: *const c_char) -> BwError {
    BwError::with_num(num, msg)
}

/// Create an error with just a message.
///
/// Replaces C `set_err()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_set_err(msg: *const c_char) -> BwError {
    BwError::with_msg(msg)
}

/// Create an error with message and errno.
///
/// Replaces C `set_err_arg()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_set_err_arg(msg: *const c_char, arg: c_int) -> BwError {
    BwError::with_msg_arg(msg, arg)
}

/// Emit a buffered error message.
///
/// Replaces C `emit_err()`.
///
/// # Safety
///
/// `e` must point to a valid `BwError`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_emit_err(e: *const BwError) {
    if e.is_null() {
        return;
    }
    let e = unsafe { &*e };
    unsafe { emit_err_inner(e) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bw_error_default() {
        let e = BwError::default();
        assert!(e.num.is_null());
        assert!(e.msg.is_null());
        assert_eq!(e.arg, 0);
        assert!(!e.alloc);
    }

    #[test]
    fn test_bw_error_with_num() {
        let num = c"E502".as_ptr();
        let msg = c"is a directory".as_ptr();
        let e = BwError::with_num(num, msg);
        assert_eq!(e.num, num);
        assert_eq!(e.msg, msg as *mut c_char);
        assert_eq!(e.arg, 0);
        assert!(!e.alloc);
    }

    #[test]
    fn test_bw_error_with_msg() {
        let msg = c"some error".as_ptr();
        let e = BwError::with_msg(msg);
        assert!(e.num.is_null());
        assert_eq!(e.msg, msg as *mut c_char);
        assert_eq!(e.arg, 0);
        assert!(!e.alloc);
    }

    #[test]
    fn test_bw_error_with_msg_arg() {
        let msg = c"write failed: %s".as_ptr();
        let e = BwError::with_msg_arg(msg, 5);
        assert!(e.num.is_null());
        assert_eq!(e.msg, msg as *mut c_char);
        assert_eq!(e.arg, 5);
        assert!(!e.alloc);
    }

    #[test]
    fn test_bw_error_clear() {
        let e = BwError::with_msg(ptr::null());
        assert!(e.msg.is_null());
    }
}
