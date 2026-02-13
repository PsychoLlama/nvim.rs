//! Opaque handle types for FFI between Rust profiling code and C.

use std::os::raw::c_void;

/// Opaque handle to a `scriptitem_T` (C-side script item).
pub type ScriptItemHandle = *mut c_void;

/// Opaque handle to a `ufunc_T` (C-side user function).
pub type UFuncHandle = *mut c_void;

/// Opaque handle to a `funccall_T` (C-side function call frame).
pub type FuncCallHandle = *mut c_void;

/// Opaque handle to an `exarg_T` (C-side ex command argument).
pub type ExargHandle = *mut c_void;

/// Opaque handle to an `expand_T` (C-side command expansion context).
pub type ExpandHandle = *mut c_void;

/// Opaque handle to a `FILE *` (C stdio file pointer).
pub type FileHandle = *mut c_void;
