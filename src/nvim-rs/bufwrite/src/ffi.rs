//! FFI type aliases and extern declarations for bufwrite.
//!
//! Opaque handle types for C structs that Rust accesses only via accessor functions.

use std::ffi::c_int;

/// Opaque handle to a C `buf_T` struct.
pub type BufHandle = *mut std::ffi::c_void;

/// Opaque handle to a C `exarg_T` struct.
pub type ExargHandle = *mut std::ffi::c_void;

/// Opaque handle to a C `FileInfo` struct.
pub type FileInfoHandle = *mut std::ffi::c_void;

/// Opaque handle to a C `struct bw_info`.
pub type BwInfoHandle = *mut std::ffi::c_void;

/// Opaque handle to a C `vim_acl_T`.
pub type AclHandle = *mut std::ffi::c_void;

// Return value constants matching C definitions
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const NOTDONE: c_int = 2;

extern "C" {
    fn nvim_bw_info_get_len(p: BwInfoHandle) -> c_int;
    fn nvim_bw_info_set_conv_error(p: BwInfoHandle, val: c_int);
}

/// Get bw_info len field.
///
/// # Safety
///
/// Handle must be valid.
pub unsafe fn nvim_bw_info_get_len_direct(p: BwInfoHandle) -> c_int {
    unsafe { nvim_bw_info_get_len(p) }
}

/// Set bw_info conv_error field.
///
/// # Safety
///
/// Handle must be valid.
pub unsafe fn nvim_bw_info_set_conv_error_direct(p: BwInfoHandle, val: c_int) {
    unsafe { nvim_bw_info_set_conv_error(p, val) }
}
