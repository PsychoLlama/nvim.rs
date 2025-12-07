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

    /// Check if buffer has a terminal attached (`buf->terminal != NULL`).
    fn nvim_buf_get_terminal(buf: BufHandle) -> c_int;

    /// Get the first character of the `b_p_ff` (fileformat option) field.
    fn nvim_buf_get_fileformat(buf: BufHandle) -> c_char;

    /// Get the `b_p_bin` (binary mode) field from a buffer.
    fn nvim_buf_get_bin(buf: BufHandle) -> c_int;

    /// Get the last buffer in the buffer list (`lastbuf` global).
    fn nvim_get_lastbuf() -> BufHandle;

    /// Get the `b_prev` field from a buffer.
    fn nvim_buf_get_prev(buf: BufHandle) -> BufHandle;
}

/// Check if "buf" is a pointer to an existing buffer.
///
/// This is the Rust equivalent of `buf_valid()` in buffer.c.
/// Iterates backwards through the buffer list (lastbuf -> b_prev).
#[inline]
fn buf_valid_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }

    // Iterate backwards through the buffer list
    // SAFETY: nvim_get_lastbuf and nvim_buf_get_prev are safe accessors
    let mut bp = unsafe { nvim_get_lastbuf() };
    while !bp.is_null() {
        if bp == buf {
            return true;
        }
        bp = unsafe { nvim_buf_get_prev(bp) };
    }
    false
}

/// FFI wrapper for `buf_valid`.
///
/// Returns non-zero if the buffer is valid.
#[no_mangle]
pub extern "C" fn rs_buf_valid(buf: BufHandle) -> c_int {
    c_int::from(buf_valid_impl(buf))
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
        nvim_buf_get_buftype(buf) == b'n' as c_char && nvim_buf_get_buftype_2(buf) == b'f' as c_char
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

/// Check if buffer has a name that may not be a file name.
///
/// Returns true if buffer is "nofile", "acwrite", terminal, or "prompt".
/// This means the buffer name may not be a file name, at least not for writing.
#[inline]
fn bt_nofilename_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe {
        let bt0 = nvim_buf_get_buftype(buf);
        // "nofile": b_p_bt[0]=='n' && b_p_bt[2]=='f'
        // "acwrite": b_p_bt[0]=='a'
        // terminal: buf->terminal != NULL
        // "prompt": b_p_bt[0]=='p'
        (bt0 == b'n' as c_char && nvim_buf_get_buftype_2(buf) == b'f' as c_char)
            || bt0 == b'a' as c_char
            || nvim_buf_get_terminal(buf) != 0
            || bt0 == b'p' as c_char
    }
}

/// FFI wrapper for `bt_nofilename`.
#[no_mangle]
pub extern "C" fn rs_bt_nofilename(buf: BufHandle) -> c_int {
    c_int::from(bt_nofilename_impl(buf))
}

/// Check if buffer should not be written.
///
/// Returns true if buffer is "nowrite", "nofile", terminal, or "prompt".
#[inline]
fn bt_dontwrite_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe {
        let bt0 = nvim_buf_get_buftype(buf);
        // "nowrite" or "nofile": b_p_bt[0]=='n'
        // terminal: buf->terminal != NULL
        // "prompt": b_p_bt[0]=='p'
        bt0 == b'n' as c_char || nvim_buf_get_terminal(buf) != 0 || bt0 == b'p' as c_char
    }
}

/// FFI wrapper for `bt_dontwrite`.
#[no_mangle]
pub extern "C" fn rs_bt_dontwrite(buf: BufHandle) -> c_int {
    c_int::from(bt_dontwrite_impl(buf))
}

/// Check if buffer should not be read from a file.
///
/// Returns true if buffer is "nofile", "quickfix", terminal, or "prompt".
/// This means the buffer is not to be read from a file.
#[inline]
fn bt_nofileread_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe {
        let bt0 = nvim_buf_get_buftype(buf);
        // "nofile": b_p_bt[0]=='n' && b_p_bt[2]=='f'
        // terminal: b_p_bt[0]=='t'
        // quickfix: b_p_bt[0]=='q'
        // "prompt": b_p_bt[0]=='p'
        (bt0 == b'n' as c_char && nvim_buf_get_buftype_2(buf) == b'f' as c_char)
            || bt0 == b't' as c_char
            || bt0 == b'q' as c_char
            || bt0 == b'p' as c_char
    }
}

/// FFI wrapper for `bt_nofileread`.
#[no_mangle]
pub extern "C" fn rs_bt_nofileread(buf: BufHandle) -> c_int {
    c_int::from(bt_nofileread_impl(buf))
}

/// End-of-line type constants (matching C defines in `option_vars.h`).
pub const EOL_UNIX: c_int = 0; // NL
pub const EOL_DOS: c_int = 1; // CR NL
pub const EOL_MAC: c_int = 2; // CR

/// Get the current end-of-line type for a buffer.
///
/// Returns `EOL_DOS`, `EOL_UNIX`, or `EOL_MAC` based on the buffer's
/// 'fileformat' and 'binary' options.
#[inline]
fn get_fileformat_impl(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return EOL_UNIX;
    }
    // SAFETY: We check for null above.
    unsafe {
        // If binary mode or first char is 'u' (unix), return EOL_UNIX
        #[allow(clippy::cast_sign_loss)]
        let ff = nvim_buf_get_fileformat(buf) as u8;
        if nvim_buf_get_bin(buf) != 0 || ff == b'u' {
            return EOL_UNIX;
        }
        // If first char is 'm' (mac), return EOL_MAC
        if ff == b'm' {
            return EOL_MAC;
        }
        // Otherwise (dos), return EOL_DOS
        EOL_DOS
    }
}

/// FFI wrapper for `get_fileformat`.
#[no_mangle]
pub extern "C" fn rs_get_fileformat(buf: BufHandle) -> c_int {
    get_fileformat_impl(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buf_handle_null() {
        let handle = unsafe { BufHandle::from_ptr(std::ptr::null_mut()) };
        assert!(handle.is_null());
        assert!(!buf_valid_impl(handle));
        assert!(!bt_prompt_impl(handle));
        assert!(!bt_normal_impl(handle));
        assert!(!bt_quickfix_impl(handle));
        assert!(!bt_terminal_impl(handle));
        assert!(!bt_nofile_impl(handle));
        assert!(!bt_help_impl(handle));
        assert!(!bt_nofilename_impl(handle));
        assert!(!bt_dontwrite_impl(handle));
        assert!(!bt_nofileread_impl(handle));
        // Null buffer defaults to EOL_UNIX
        assert_eq!(get_fileformat_impl(handle), EOL_UNIX);
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
