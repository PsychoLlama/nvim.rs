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
