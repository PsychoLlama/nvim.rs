//! Vimscript typval_T operations for Neovim
//!
//! This crate provides Rust implementations of typval-related functions
//! from `src/nvim/eval/typval.c`. It uses an opaque handle pattern where
//! `typval_T*` pointers are treated as opaque handles, with field access
//! done through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::{c_char, c_int};

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
    /// Convert from C integer to VarType.
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

/// Opaque handle to a Vimscript typval (`typval_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypevalHandle(*const std::ffi::c_void);

impl TypevalHandle {
    /// Create a new typval handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `typval_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *const std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *const std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// C accessor functions for typval_T fields.
// These will be defined in eval/typval.c
extern "C" {
    /// Get the v_type field from a typval.
    fn nvim_tv_get_type(tv: TypevalHandle) -> c_int;

    /// Get the v_number field from a typval (only valid if v_type == VAR_NUMBER).
    fn nvim_tv_get_number(tv: TypevalHandle) -> i64;

    /// Get the v_bool field from a typval (only valid if v_type == VAR_BOOL).
    fn nvim_tv_get_bool(tv: TypevalHandle) -> c_int;

    /// Get the v_special field from a typval (only valid if v_type == VAR_SPECIAL).
    fn nvim_tv_get_special(tv: TypevalHandle) -> c_int;

    /// Get the v_float field from a typval (only valid if v_type == VAR_FLOAT).
    fn nvim_tv_get_float(tv: TypevalHandle) -> f64;

    /// Get the v_string field from a typval (only valid if v_type == VAR_STRING or VAR_FUNC).
    fn nvim_tv_get_string_ptr(tv: TypevalHandle) -> *const c_char;

    /// Check if v_list is NULL (only valid if v_type == VAR_LIST).
    fn nvim_tv_list_is_null(tv: TypevalHandle) -> c_int;

    /// Check if v_dict is NULL (only valid if v_type == VAR_DICT).
    fn nvim_tv_dict_is_null(tv: TypevalHandle) -> c_int;

    /// Check if v_blob is NULL (only valid if v_type == VAR_BLOB).
    fn nvim_tv_blob_is_null(tv: TypevalHandle) -> c_int;

    /// Check if v_partial is NULL (only valid if v_type == VAR_PARTIAL).
    fn nvim_tv_partial_is_null(tv: TypevalHandle) -> c_int;
}

// =============================================================================
// Type checking predicates
// =============================================================================

/// Get the VarType of a typval.
#[inline]
fn tv_type_impl(tv: TypevalHandle) -> VarType {
    if tv.is_null() {
        return VarType::Unknown;
    }
    // SAFETY: We check for null above.
    let type_int = unsafe { nvim_tv_get_type(tv) };
    VarType::from_c_int(type_int).unwrap_or(VarType::Unknown)
}

/// FFI wrapper: get the type of a typval.
#[no_mangle]
pub extern "C" fn rs_tv_type(tv: TypevalHandle) -> c_int {
    tv_type_impl(tv) as c_int
}

/// Check if typval is a number.
#[inline]
fn tv_is_number_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Number
}

/// FFI wrapper for tv_is_number.
#[no_mangle]
pub extern "C" fn rs_tv_is_number(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_number_impl(tv))
}

/// Check if typval is a string.
#[inline]
fn tv_is_string_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::String
}

/// FFI wrapper for tv_is_string.
#[no_mangle]
pub extern "C" fn rs_tv_is_string(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_string_impl(tv))
}

/// Check if typval is a float.
#[inline]
fn tv_is_float_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Float
}

/// FFI wrapper for tv_is_float.
#[no_mangle]
pub extern "C" fn rs_tv_is_float(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_float_impl(tv))
}

/// Check if typval is a bool.
#[inline]
fn tv_is_bool_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Bool
}

/// FFI wrapper for tv_is_bool.
#[no_mangle]
pub extern "C" fn rs_tv_is_bool(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_bool_impl(tv))
}

/// Check if typval is a list.
#[inline]
fn tv_is_list_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::List
}

/// FFI wrapper for tv_is_list.
#[no_mangle]
pub extern "C" fn rs_tv_is_list(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_list_impl(tv))
}

/// Check if typval is a dict.
#[inline]
fn tv_is_dict_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Dict
}

/// FFI wrapper for tv_is_dict.
#[no_mangle]
pub extern "C" fn rs_tv_is_dict(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_dict_impl(tv))
}

/// Check if typval is a blob.
#[inline]
fn tv_is_blob_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Blob
}

/// FFI wrapper for tv_is_blob.
#[no_mangle]
pub extern "C" fn rs_tv_is_blob(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_blob_impl(tv))
}

/// Check if typval is a function reference (VAR_FUNC or VAR_PARTIAL).
#[inline]
fn tv_is_func_impl(tv: TypevalHandle) -> bool {
    let t = tv_type_impl(tv);
    t == VarType::Func || t == VarType::Partial
}

/// FFI wrapper for tv_is_func.
#[no_mangle]
pub extern "C" fn rs_tv_is_func(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_func_impl(tv))
}

/// Check if typval is special (null).
#[inline]
fn tv_is_special_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Special
}

/// FFI wrapper for tv_is_special.
#[no_mangle]
pub extern "C" fn rs_tv_is_special(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_special_impl(tv))
}

/// Check if typval has a numeric type (number or float).
#[inline]
fn tv_is_numeric_impl(tv: TypevalHandle) -> bool {
    let t = tv_type_impl(tv);
    t == VarType::Number || t == VarType::Float
}

/// FFI wrapper for tv_is_numeric.
#[no_mangle]
pub extern "C" fn rs_tv_is_numeric(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_numeric_impl(tv))
}

// =============================================================================
// Value extraction (simple - no error reporting)
// =============================================================================

/// Get the number value from a typval (returns 0 if not a number).
#[inline]
fn tv_get_number_simple_impl(tv: TypevalHandle) -> i64 {
    if tv.is_null() {
        return 0;
    }
    let t = tv_type_impl(tv);
    match t {
        VarType::Number => unsafe { nvim_tv_get_number(tv) },
        VarType::Bool => {
            // v_bool: 0 = false, 1 = true
            let b = unsafe { nvim_tv_get_bool(tv) };
            i64::from(b)
        }
        _ => 0,
    }
}

/// FFI wrapper: get number value from typval (0 if not a number).
#[no_mangle]
pub extern "C" fn rs_tv_get_number_simple(tv: TypevalHandle) -> i64 {
    tv_get_number_simple_impl(tv)
}

/// Get the float value from a typval (returns 0.0 if not a float or number).
#[inline]
fn tv_get_float_simple_impl(tv: TypevalHandle) -> f64 {
    if tv.is_null() {
        return 0.0;
    }
    let t = tv_type_impl(tv);
    match t {
        VarType::Float => unsafe { nvim_tv_get_float(tv) },
        VarType::Number => {
            let n = unsafe { nvim_tv_get_number(tv) };
            n as f64
        }
        _ => 0.0,
    }
}

/// FFI wrapper: get float value from typval (0.0 if not a float).
#[no_mangle]
pub extern "C" fn rs_tv_get_float_simple(tv: TypevalHandle) -> f64 {
    tv_get_float_simple_impl(tv)
}

/// Get the bool value from a typval (returns false if not a bool/number).
#[inline]
fn tv_get_bool_simple_impl(tv: TypevalHandle) -> bool {
    if tv.is_null() {
        return false;
    }
    let t = tv_type_impl(tv);
    match t {
        VarType::Bool => {
            let b = unsafe { nvim_tv_get_bool(tv) };
            b != 0
        }
        VarType::Number => {
            let n = unsafe { nvim_tv_get_number(tv) };
            n != 0
        }
        _ => false,
    }
}

/// FFI wrapper: get bool value from typval (false if not a bool).
#[no_mangle]
pub extern "C" fn rs_tv_get_bool_simple(tv: TypevalHandle) -> c_int {
    c_int::from(tv_get_bool_simple_impl(tv))
}

/// Get the string pointer from a typval (returns NULL if not a string).
#[inline]
fn tv_get_string_ptr_impl(tv: TypevalHandle) -> *const c_char {
    if tv.is_null() {
        return std::ptr::null();
    }
    let t = tv_type_impl(tv);
    if t == VarType::String {
        unsafe { nvim_tv_get_string_ptr(tv) }
    } else {
        std::ptr::null()
    }
}

/// FFI wrapper: get string pointer from typval (NULL if not a string).
#[no_mangle]
pub extern "C" fn rs_tv_get_string_ptr(tv: TypevalHandle) -> *const c_char {
    tv_get_string_ptr_impl(tv)
}

// =============================================================================
// Emptiness/truthiness checks
// =============================================================================

/// Check if a typval is "empty" (falsy in Vimscript terms).
///
/// - Numbers: 0 is empty
/// - Strings: empty string or NULL is empty
/// - Lists: NULL or empty list is empty
/// - Dicts: NULL or empty dict is empty
/// - Blobs: NULL or empty blob is empty
/// - Bools: false is empty
/// - Special: always empty (v:null)
/// - Floats: 0.0 is empty
/// - Funcs/Partials: never empty (always truthy if set)
#[inline]
fn tv_is_empty_impl(tv: TypevalHandle) -> bool {
    if tv.is_null() {
        return true;
    }
    let t = tv_type_impl(tv);
    match t {
        VarType::Unknown => true,
        VarType::Number => unsafe { nvim_tv_get_number(tv) == 0 },
        VarType::String => {
            let s = unsafe { nvim_tv_get_string_ptr(tv) };
            if s.is_null() {
                true
            } else {
                // Check if first byte is NUL (empty string)
                unsafe { *s == 0 }
            }
        }
        VarType::Float => unsafe { nvim_tv_get_float(tv) == 0.0 },
        VarType::Bool => unsafe { nvim_tv_get_bool(tv) == 0 },
        VarType::Special => true, // v:null is always empty
        VarType::List => unsafe { nvim_tv_list_is_null(tv) != 0 },
        VarType::Dict => unsafe { nvim_tv_dict_is_null(tv) != 0 },
        VarType::Blob => unsafe { nvim_tv_blob_is_null(tv) != 0 },
        VarType::Func => {
            // Function reference is non-empty if string is set
            let s = unsafe { nvim_tv_get_string_ptr(tv) };
            s.is_null() || unsafe { *s == 0 }
        }
        VarType::Partial => unsafe { nvim_tv_partial_is_null(tv) != 0 },
    }
}

/// FFI wrapper: check if typval is empty/falsy.
#[no_mangle]
pub extern "C" fn rs_tv_is_empty(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_empty_impl(tv))
}

/// Check if a typval is "truthy" (non-empty in Vimscript terms).
#[inline]
fn tv_is_truthy_impl(tv: TypevalHandle) -> bool {
    !tv_is_empty_impl(tv)
}

/// FFI wrapper: check if typval is truthy.
#[no_mangle]
pub extern "C" fn rs_tv_is_truthy(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_truthy_impl(tv))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vartype_from_c_int() {
        assert_eq!(VarType::from_c_int(0), Some(VarType::Unknown));
        assert_eq!(VarType::from_c_int(1), Some(VarType::Number));
        assert_eq!(VarType::from_c_int(2), Some(VarType::String));
        assert_eq!(VarType::from_c_int(10), Some(VarType::Blob));
        assert_eq!(VarType::from_c_int(99), None);
    }

    #[test]
    fn test_typval_handle_null() {
        let handle = unsafe { TypevalHandle::from_ptr(std::ptr::null()) };
        assert!(handle.is_null());
        assert_eq!(tv_type_impl(handle), VarType::Unknown);
        assert!(!tv_is_number_impl(handle));
        assert!(!tv_is_string_impl(handle));
        assert!(!tv_is_float_impl(handle));
        assert!(tv_is_empty_impl(handle));
    }
}
