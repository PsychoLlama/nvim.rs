//! Digraph list operations.
//!
//! This module provides Rust implementations for building digraph lists,
//! used by `digraph_getlist()` Vimscript function.

use std::ffi::{c_char, c_void};

use libc::c_int;

use crate::DigrT;

// C accessor functions
extern "C" {
    /// Get pointer to user digraphs array data.
    fn nvim_get_user_digraphs_data() -> *const c_void;

    /// Get length of user digraphs array.
    fn nvim_get_user_digraphs_len() -> c_int;

    /// Get pointer to default digraphs array.
    fn nvim_get_digraphdefault() -> *const c_void;

    /// Get exact digraph match.
    fn rs_getexactdigraph(char1: c_int, char2: c_int, meta_char: c_int) -> c_int;

    /// Convert character to UTF-8.
    fn rs_utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
}

/// Callback type for iterating over digraphs.
///
/// Called for each digraph with:
/// - `char1`, `char2`: The digraph input characters
/// - `result`: The digraph result character
/// - `ctx`: User context pointer
///
/// Return `true` to continue iteration, `false` to stop.
pub type DigraphIterCallback =
    unsafe extern "C" fn(char1: u8, char2: u8, result: c_int, ctx: *mut c_void) -> c_int;

/// Iterate over all digraphs (user and default).
///
/// Calls the callback for each digraph. User digraphs are iterated first,
/// then default digraphs.
///
/// # Arguments
/// * `list_all` - If true, include default digraphs; if false, only user digraphs
/// * `callback` - Function called for each digraph
/// * `ctx` - User context passed to callback
///
/// # Returns
/// Number of digraphs iterated.
///
/// # Safety
/// Callback must be a valid function pointer, ctx can be null.
#[no_mangle]
pub unsafe extern "C" fn rs_digraph_iterate(
    list_all: c_int,
    callback: DigraphIterCallback,
    ctx: *mut c_void,
) -> c_int {
    let mut count = 0;

    // Iterate default digraphs first (if requested)
    if list_all != 0 {
        let default_data = unsafe { nvim_get_digraphdefault() };
        if !default_data.is_null() {
            let default_digraphs = default_data.cast::<DigrT>();
            let mut i = 0;
            loop {
                let dp = unsafe { &*default_digraphs.add(i) };
                // Default array is null-terminated
                if dp.char1 == 0 {
                    break;
                }

                // Get actual result (may be overridden by user digraph)
                let result =
                    unsafe { rs_getexactdigraph(c_int::from(dp.char1), c_int::from(dp.char2), 0) };

                // Skip if result is 0 or same as char2 (no digraph)
                if result != 0 && result != c_int::from(dp.char2) {
                    let should_continue = unsafe { callback(dp.char1, dp.char2, result, ctx) };
                    if should_continue == 0 {
                        return count;
                    }
                    count += 1;
                }

                i += 1;
            }
        }
    }

    // Iterate user digraphs
    let user_data = unsafe { nvim_get_user_digraphs_data() };
    let user_len = unsafe { nvim_get_user_digraphs_len() };

    if !user_data.is_null() && user_len > 0 {
        let user_digraphs = user_data.cast::<DigrT>();
        #[allow(clippy::cast_sign_loss)]
        let len = user_len as usize;

        for i in 0..len {
            let dp = unsafe { &*user_digraphs.add(i) };
            let should_continue = unsafe { callback(dp.char1, dp.char2, dp.result, ctx) };
            if should_continue == 0 {
                return count;
            }
            count += 1;
        }
    }

    count
}

/// Format a digraph as a two-character string.
///
/// Writes the two digraph characters to `buf` followed by NUL.
///
/// # Arguments
/// * `char1` - First character
/// * `char2` - Second character
/// * `buf` - Output buffer (at least 3 bytes)
///
/// # Safety
/// `buf` must point to at least 3 writable bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_digraph_format_pair(char1: u8, char2: u8, buf: *mut c_char) {
    if buf.is_null() {
        return;
    }
    unsafe {
        #[allow(clippy::cast_possible_wrap)]
        {
            *buf = char1 as c_char;
            *buf.add(1) = char2 as c_char;
        }
        *buf.add(2) = 0;
    }
}

/// Format a digraph result as UTF-8.
///
/// Writes the UTF-8 representation of the result character to `buf`.
///
/// # Arguments
/// * `result` - The digraph result character
/// * `buf` - Output buffer (at least 7 bytes for UTF-8 + NUL)
///
/// # Returns
/// Number of bytes written (not including NUL).
///
/// # Safety
/// `buf` must point to at least 7 writable bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_digraph_format_result(result: c_int, buf: *mut c_char) -> c_int {
    if buf.is_null() {
        return 0;
    }

    let len = unsafe { rs_utf_char2bytes(result, buf) };

    // NUL terminate
    #[allow(clippy::cast_sign_loss)]
    if len >= 0 {
        unsafe { *buf.add(len as usize) = 0 };
    }

    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_format_pair() {
        let mut buf = [0i8; 4];
        unsafe { rs_digraph_format_pair(b'a', b':', buf.as_mut_ptr()) };
        assert_eq!(buf[0], b'a' as i8);
        assert_eq!(buf[1], b':' as i8);
        assert_eq!(buf[2], 0);
    }

    #[test]
    fn test_format_pair_null_safe() {
        // Should not crash with null
        unsafe { rs_digraph_format_pair(b'a', b'b', std::ptr::null_mut()) };
    }
}
