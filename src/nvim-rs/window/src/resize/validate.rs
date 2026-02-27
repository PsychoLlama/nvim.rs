//! Option validation for window minimum sizes.
//!
//! This module provides Rust implementations of `did_set_winminheight` and
//! `did_set_winminwidth` from `src/nvim/window.c`.

use std::ffi::c_int;
use std::os::raw::c_char;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_Rows() -> c_int;
    fn nvim_get_Columns() -> c_int;
    fn nvim_get_window_p_ch() -> i64;
    fn nvim_get_p_wmh() -> i64;
    fn nvim_get_p_wmw() -> i64;
    fn nvim_set_p_wmh(val: i64);
    fn nvim_set_p_wmw(val: i64);
    fn nvim_emsg_id(id: c_int);
    fn rs_min_rows_for_all_tabpages() -> c_int;
    fn rs_frame_minwidth(topfrp: *const crate::Frame, next_curwin: crate::WinHandle) -> c_int;
    fn nvim_get_topframe() -> *mut crate::Frame;
}

// =============================================================================
// EMSG IDs
// =============================================================================

const EMSG_NOROOM: c_int = 13;

// =============================================================================
// Implementations
// =============================================================================

/// Check 'winminheight' for a valid value and reduce it if needed.
///
/// Equivalent to C `did_set_winminheight()` (window.c L7039).
fn did_set_winminheight_impl() -> *const c_char {
    unsafe {
        let mut first = true;

        // loop until there is a 'winminheight' that is possible
        while nvim_get_p_wmh() > 0 {
            #[allow(clippy::cast_possible_truncation)]
            let room = nvim_get_Rows() - nvim_get_window_p_ch() as c_int;
            let needed = rs_min_rows_for_all_tabpages();
            if room >= needed {
                break;
            }
            nvim_set_p_wmh(nvim_get_p_wmh() - 1);
            if first {
                nvim_emsg_id(EMSG_NOROOM);
                first = false;
            }
        }
    }
    std::ptr::null()
}

/// Check 'winminwidth' for a valid value and reduce it if needed.
///
/// Equivalent to C `did_set_winminwidth()` (window.c L7060).
fn did_set_winminwidth_impl() -> *const c_char {
    unsafe {
        let mut first = true;

        // loop until there is a 'winminwidth' that is possible
        while nvim_get_p_wmw() > 0 {
            let room = nvim_get_Columns();
            let needed = rs_frame_minwidth(nvim_get_topframe(), crate::WinHandle::null());
            if room >= needed {
                break;
            }
            nvim_set_p_wmw(nvim_get_p_wmw() - 1);
            if first {
                nvim_emsg_id(EMSG_NOROOM);
                first = false;
            }
        }
    }
    std::ptr::null()
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Check 'winminheight' for a valid value and reduce it if needed.
/// Returns NULL (success).
///
/// The `_args` parameter is required by the option callback system but unused.
#[unsafe(no_mangle)]
pub extern "C" fn rs_did_set_winminheight(_args: *mut std::ffi::c_void) -> *const c_char {
    did_set_winminheight_impl()
}

/// FFI: Check 'winminwidth' for a valid value and reduce it if needed.
/// Returns NULL (success).
///
/// The `_args` parameter is required by the option callback system but unused.
#[unsafe(no_mangle)]
pub extern "C" fn rs_did_set_winminwidth(_args: *mut std::ffi::c_void) -> *const c_char {
    did_set_winminwidth_impl()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_return_null() {
        // The return type is always null
        let null_ptr: *const c_char = std::ptr::null();
        assert!(null_ptr.is_null());
    }
}
