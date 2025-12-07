//! Buffer handling utilities for Neovim
//!
//! This crate provides Rust implementations of buffer-related functions
//! from `src/nvim/buffer.c`. It uses an opaque handle pattern where
//! `buf_T*` pointers are treated as opaque handles, with field access
//! done through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::cast_possible_wrap)] // Byte literals in ASCII range are safe

use std::ffi::{c_char, c_int};

/// Opaque handle to a Neovim buffer (`buf_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufHandle(*mut std::ffi::c_void);

impl BufHandle {
    /// Create a new buffer handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `buf_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// C accessor functions for buffer fields.
// These are defined in buffer.c and provide safe access to buf_T fields.
extern "C" {
    /// Get the `b_p_bt` (buftype option) field - returns first char.
    fn nvim_buf_get_buftype(buf: BufHandle) -> c_char;

    /// Get the `b_p_bt[2]` character (for checking "nofile" vs "nowrite").
    fn nvim_buf_get_buftype_2(buf: BufHandle) -> c_char;

    /// Get the `b_help` field from a buffer.
    fn nvim_buf_get_help(buf: BufHandle) -> c_int;
}

/// Check if buffer is a prompt buffer ('buftype' starts with 'p').
#[inline]
fn bt_prompt_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above, and the accessor handles the pointer safely.
    unsafe { nvim_buf_get_buftype(buf) == b'p' as c_char }
}

/// FFI wrapper for `bt_prompt`.
#[no_mangle]
pub extern "C" fn rs_bt_prompt(buf: BufHandle) -> c_int {
    c_int::from(bt_prompt_impl(buf))
}

/// Check if buffer is a normal buffer ('buftype' is empty/NUL).
#[inline]
fn bt_normal_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_buf_get_buftype(buf) == 0 }
}

/// FFI wrapper for `bt_normal`.
#[no_mangle]
pub extern "C" fn rs_bt_normal(buf: BufHandle) -> c_int {
    c_int::from(bt_normal_impl(buf))
}

/// Check if buffer is the quickfix buffer ('buftype' starts with 'q').
#[inline]
fn bt_quickfix_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_buf_get_buftype(buf) == b'q' as c_char }
}

/// FFI wrapper for `bt_quickfix`.
#[no_mangle]
pub extern "C" fn rs_bt_quickfix(buf: BufHandle) -> c_int {
    c_int::from(bt_quickfix_impl(buf))
}

/// Check if buffer is a terminal buffer ('buftype' starts with 't').
#[inline]
fn bt_terminal_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_buf_get_buftype(buf) == b't' as c_char }
}

/// FFI wrapper for `bt_terminal`.
#[no_mangle]
pub extern "C" fn rs_bt_terminal(buf: BufHandle) -> c_int {
    c_int::from(bt_terminal_impl(buf))
}

/// Check if buffer has 'buftype' set to "nofile".
///
/// This checks that `b_p_bt[0]` == 'n' AND `b_p_bt[2]` == 'f'.
#[inline]
fn bt_nofile_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe {
        nvim_buf_get_buftype(buf) == b'n' as c_char
            && nvim_buf_get_buftype_2(buf) == b'f' as c_char
    }
}

/// FFI wrapper for `bt_nofile`.
#[no_mangle]
pub extern "C" fn rs_bt_nofile(buf: BufHandle) -> c_int {
    c_int::from(bt_nofile_impl(buf))
}

/// Check if buffer is a help buffer.
#[inline]
fn bt_help_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_buf_get_help(buf) != 0 }
}

/// FFI wrapper for `bt_help`.
#[no_mangle]
pub extern "C" fn rs_bt_help(buf: BufHandle) -> c_int {
    c_int::from(bt_help_impl(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buf_handle_null() {
        let handle = unsafe { BufHandle::from_ptr(std::ptr::null_mut()) };
        assert!(handle.is_null());
        assert!(!bt_prompt_impl(handle));
        assert!(!bt_normal_impl(handle));
        assert!(!bt_quickfix_impl(handle));
        assert!(!bt_terminal_impl(handle));
        assert!(!bt_nofile_impl(handle));
        assert!(!bt_help_impl(handle));
    }

    #[test]
    fn test_buf_handle_non_null() {
        // Create a fake non-null pointer for testing
        let fake_ptr = 0x1000 as *mut std::ffi::c_void;
        let handle = unsafe { BufHandle::from_ptr(fake_ptr) };
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), fake_ptr);
    }
}
