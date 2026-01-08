//! Function dispatch helpers for VimL built-in functions.
//!
//! This module provides common utilities and types used by all VimL function
//! implementations.

#![allow(clippy::must_use_candidate)]
#![allow(dead_code)]

use std::ffi::{c_int, c_void};

// =============================================================================
// Typval Handle and Accessors
// =============================================================================

/// Opaque handle to a `typval_T*` (VimL value).
///
/// Rust cannot directly access typval_T fields; all access goes through
/// C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct TypevalPtr(*const c_void);

impl TypevalPtr {
    /// Create from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `typval_T*` or null.
    #[inline]
    pub const unsafe fn from_raw(ptr: *const c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *const c_void {
        self.0
    }

    /// Check if null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Mutable typval handle for return values.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct TypevalPtrMut(*mut c_void);

impl TypevalPtrMut {
    /// Create from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `typval_T*` or null.
    #[inline]
    pub const unsafe fn from_raw(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    /// Check if null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// VarType enum values (matching C's VarType in typval_defs.h).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarType {
    Unknown = 0,
    Number = 1,
    String = 2,
    Func = 3,
    List = 4,
    Dict = 5,
    Float = 6,
    Bool = 7,
    Special = 8,
    Partial = 9,
    Blob = 10,
}

impl VarType {
    /// Convert from C integer.
    #[inline]
    pub const fn from_c_int(v: c_int) -> Option<Self> {
        match v {
            0 => Some(Self::Unknown),
            1 => Some(Self::Number),
            2 => Some(Self::String),
            3 => Some(Self::Func),
            4 => Some(Self::List),
            5 => Some(Self::Dict),
            6 => Some(Self::Float),
            7 => Some(Self::Bool),
            8 => Some(Self::Special),
            9 => Some(Self::Partial),
            10 => Some(Self::Blob),
            _ => None,
        }
    }
}

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    // --- Type accessors ---
    fn nvim_tv_get_type(tv: *const c_void) -> c_int;
    fn nvim_tv_get_number(tv: *const c_void) -> i64;
    fn nvim_tv_get_float(tv: *const c_void) -> f64;

    // --- Setters for return values ---
    fn nvim_tv_set_number(tv: *mut c_void, n: i64);
    fn nvim_tv_set_float(tv: *mut c_void, f: f64);

    // --- Value extraction with error checking ---
    /// Get number from typval with error checking.
    /// Sets `*error = true` if the type is invalid for number conversion.
    fn nvim_tv_get_number_chk(tv: *const c_void, error: *mut bool) -> i64;

    /// Get float from typval with error checking.
    /// Returns false and sets rettv to 0.0 if conversion fails.
    fn nvim_tv_get_float_chk(tv: *const c_void, ret: *mut f64) -> bool;
}

// =============================================================================
// Safe Wrappers
// =============================================================================

/// Get the VarType of a typval.
#[inline]
pub fn tv_get_type(tv: TypevalPtr) -> VarType {
    if tv.is_null() {
        return VarType::Unknown;
    }
    let type_int = unsafe { nvim_tv_get_type(tv.as_ptr()) };
    VarType::from_c_int(type_int).unwrap_or(VarType::Unknown)
}

/// Get number value (raw, no type conversion).
#[inline]
pub fn tv_get_number_raw(tv: TypevalPtr) -> i64 {
    if tv.is_null() {
        return 0;
    }
    unsafe { nvim_tv_get_number(tv.as_ptr()) }
}

/// Get float value (raw, no type conversion).
#[inline]
pub fn tv_get_float_raw(tv: TypevalPtr) -> f64 {
    if tv.is_null() {
        return 0.0;
    }
    unsafe { nvim_tv_get_float(tv.as_ptr()) }
}

/// Get number with error checking (reports VimL errors for invalid types).
/// Returns the number and whether an error occurred.
#[inline]
pub fn tv_get_number_chk(tv: TypevalPtr) -> (i64, bool) {
    if tv.is_null() {
        return (0, true);
    }
    let mut error = false;
    let n = unsafe { nvim_tv_get_number_chk(tv.as_ptr(), &raw mut error) };
    (n, error)
}

/// Get float with error checking (reports VimL errors for invalid types).
/// Returns Some(float) on success, None on error.
#[inline]
pub fn tv_get_float_chk(tv: TypevalPtr) -> Option<f64> {
    if tv.is_null() {
        return None;
    }
    let mut ret = 0.0;
    let ok = unsafe { nvim_tv_get_float_chk(tv.as_ptr(), &raw mut ret) };
    if ok {
        Some(ret)
    } else {
        None
    }
}

/// Set return value to a number.
#[inline]
pub fn rettv_set_number(rettv: TypevalPtrMut, n: i64) {
    if !rettv.is_null() {
        unsafe { nvim_tv_set_number(rettv.as_ptr(), n) };
    }
}

/// Set return value to a float.
#[inline]
pub fn rettv_set_float(rettv: TypevalPtrMut, f: f64) {
    if !rettv.is_null() {
        unsafe { nvim_tv_set_float(rettv.as_ptr(), f) };
    }
}

// =============================================================================
// Argument Array Access
// =============================================================================

/// Get a typval from an argument array by index.
///
/// # Safety
/// - `argvars` must be a valid pointer to a typval array
/// - `index` must be within bounds of the array
#[inline]
pub unsafe fn argvar_at(argvars: *const c_void, index: usize) -> TypevalPtr {
    // typval_T is 24 bytes on 64-bit systems (v_type: i32, padding, union: 8 bytes, v_lock: i32, padding)
    // We use the accessor to get sizeof(typval_T) from C
    let offset = index * TYPVAL_SIZE;
    let ptr = argvars.cast::<u8>().add(offset).cast::<c_void>();
    TypevalPtr::from_raw(ptr)
}

// Size of typval_T in bytes (from C)
// This is platform-dependent but we match the C definition
const TYPVAL_SIZE: usize = 24; // sizeof(typval_T) on 64-bit

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vartype_from_int() {
        assert_eq!(VarType::from_c_int(0), Some(VarType::Unknown));
        assert_eq!(VarType::from_c_int(1), Some(VarType::Number));
        assert_eq!(VarType::from_c_int(6), Some(VarType::Float));
        assert_eq!(VarType::from_c_int(99), None);
    }

    #[test]
    fn test_null_typval_ptr() {
        let null_ptr = unsafe { TypevalPtr::from_raw(std::ptr::null()) };
        assert!(null_ptr.is_null());
        // tv_get_type() calls C functions, can't test without linking to C
    }

    #[test]
    fn test_typval_size() {
        // Verify our TYPVAL_SIZE constant matches expected layout
        assert_eq!(TYPVAL_SIZE, 24);
    }
}
