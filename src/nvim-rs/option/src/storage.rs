//! Option value storage operations
//!
//! This module provides Rust implementations for OptVal storage operations
//! including copying, freeing, and comparing option values.

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::{OptInt, OptValType};

// =============================================================================
// C FFI Declarations
// =============================================================================

extern "C" {
    fn api_free_string(s: String_);
    fn copy_string(s: String_, arena: *mut std::ffi::c_void) -> String_;
    fn strnequal(s1: *const c_char, s2: *const c_char, n: usize) -> bool;
}

/// Nvim String type (matches api/private/defs.h)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct String_ {
    pub data: *mut c_char,
    pub size: usize,
}

impl Default for String_ {
    fn default() -> Self {
        Self {
            data: ptr::null_mut(),
            size: 0,
        }
    }
}

// =============================================================================
// OptVal Type
// =============================================================================

/// Union data for OptVal
#[repr(C)]
#[derive(Clone, Copy)]
pub union OptValData {
    pub boolean: c_int,
    pub number: OptInt,
    pub string: String_,
}

impl Default for OptValData {
    fn default() -> Self {
        Self { number: 0 }
    }
}

/// Option value (matches OptVal in option_defs.h)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct OptVal {
    pub type_: OptValType,
    pub data: OptValData,
}

impl Default for OptVal {
    fn default() -> Self {
        Self {
            type_: OptValType::Nil,
            data: OptValData::default(),
        }
    }
}

impl OptVal {
    /// Create a nil OptVal.
    #[must_use]
    pub const fn nil() -> Self {
        Self {
            type_: OptValType::Nil,
            data: OptValData { number: 0 },
        }
    }

    /// Create a boolean OptVal.
    #[must_use]
    pub const fn boolean(val: c_int) -> Self {
        Self {
            type_: OptValType::Boolean,
            data: OptValData { boolean: val },
        }
    }

    /// Create a number OptVal.
    #[must_use]
    pub const fn number(val: OptInt) -> Self {
        Self {
            type_: OptValType::Number,
            data: OptValData { number: val },
        }
    }

    /// Create a string OptVal.
    #[must_use]
    pub const fn string(s: String_) -> Self {
        Self {
            type_: OptValType::String,
            data: OptValData { string: s },
        }
    }

    /// Check if this is a nil value.
    #[must_use]
    pub const fn is_nil(&self) -> bool {
        matches!(self.type_, OptValType::Nil)
    }

    /// Check if this is a boolean value.
    #[must_use]
    pub const fn is_boolean(&self) -> bool {
        matches!(self.type_, OptValType::Boolean)
    }

    /// Check if this is a number value.
    #[must_use]
    pub const fn is_number(&self) -> bool {
        matches!(self.type_, OptValType::Number)
    }

    /// Check if this is a string value.
    #[must_use]
    pub const fn is_string(&self) -> bool {
        matches!(self.type_, OptValType::String)
    }
}

// =============================================================================
// Empty String Sentinel
// =============================================================================

extern "C" {
    /// Get the address of the empty_string_option array.
    /// We use an accessor because `empty_string_option` is a C `char[]` (array),
    /// and Rust's `extern static` would treat it as a pointer variable, reading
    /// the array contents instead of its address.
    fn nvim_get_empty_string_option() -> *mut c_char;
}

/// Get the empty string option sentinel pointer.
#[inline]
unsafe fn empty_string_option() -> *mut c_char {
    nvim_get_empty_string_option()
}

// =============================================================================
// OptVal Operations
// =============================================================================

/// Free an allocated OptVal.
///
/// # Safety
/// If the OptVal contains a string, the string data must be valid and
/// previously allocated (or be the empty_string_option sentinel).
#[no_mangle]
pub unsafe extern "C" fn rs_optval_free(o: OptVal) {
    match o.type_ {
        OptValType::Nil | OptValType::Boolean | OptValType::Number => {
            // Nothing to free
        }
        OptValType::String => {
            // Don't free empty string option sentinel
            if o.data.string.data != empty_string_option() {
                api_free_string(o.data.string);
            }
        }
    }
}

/// Copy an OptVal.
///
/// For string values, allocates a new string copy.
/// For other types, returns a bitwise copy.
///
/// # Safety
/// If the OptVal contains a string, the string data must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_optval_copy(o: OptVal) -> OptVal {
    match o.type_ {
        OptValType::Nil | OptValType::Boolean | OptValType::Number => o,
        OptValType::String => {
            let copied = copy_string(o.data.string, ptr::null_mut());
            OptVal::string(copied)
        }
    }
}

/// Check if two option values are equal.
///
/// # Safety
/// If either OptVal contains a string, the string data must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_optval_equal(o1: OptVal, o2: OptVal) -> c_int {
    if o1.type_ as c_int != o2.type_ as c_int {
        return 0;
    }

    let equal = match o1.type_ {
        OptValType::Nil => true,
        OptValType::Boolean => o1.data.boolean == o2.data.boolean,
        OptValType::Number => o1.data.number == o2.data.number,
        OptValType::String => {
            o1.data.string.size == o2.data.string.size
                && (o1.data.string.data == o2.data.string.data
                    || strnequal(
                        o1.data.string.data,
                        o2.data.string.data,
                        o1.data.string.size,
                    ))
        }
    };

    c_int::from(equal)
}

// =============================================================================
// OptVal Constructors (FFI)
// =============================================================================

/// Create a nil OptVal.
#[no_mangle]
pub extern "C" fn rs_optval_nil() -> OptVal {
    OptVal::nil()
}

/// Create a boolean OptVal.
#[no_mangle]
pub extern "C" fn rs_optval_boolean(val: c_int) -> OptVal {
    OptVal::boolean(val)
}

/// Create a number OptVal.
#[no_mangle]
pub extern "C" fn rs_optval_number(val: OptInt) -> OptVal {
    OptVal::number(val)
}

/// Create a string OptVal.
#[no_mangle]
pub extern "C" fn rs_optval_string(data: *mut c_char, size: usize) -> OptVal {
    OptVal::string(String_ { data, size })
}

// =============================================================================
// OptVal Accessors (FFI)
// =============================================================================

/// Get the type of an OptVal.
#[no_mangle]
pub extern "C" fn rs_optval_get_type(o: OptVal) -> c_int {
    o.type_ as c_int
}

/// Get the boolean value from an OptVal.
///
/// # Safety
/// Must only be called on boolean OptVals.
#[no_mangle]
pub unsafe extern "C" fn rs_optval_get_boolean(o: OptVal) -> c_int {
    o.data.boolean
}

/// Get the number value from an OptVal.
///
/// # Safety
/// Must only be called on number OptVals.
#[no_mangle]
pub unsafe extern "C" fn rs_optval_get_number(o: OptVal) -> OptInt {
    o.data.number
}

/// Get the string data from an OptVal.
///
/// # Safety
/// Must only be called on string OptVals.
#[no_mangle]
pub unsafe extern "C" fn rs_optval_get_string_data(o: OptVal) -> *mut c_char {
    o.data.string.data
}

/// Get the string size from an OptVal.
///
/// # Safety
/// Must only be called on string OptVals.
#[no_mangle]
pub unsafe extern "C" fn rs_optval_get_string_size(o: OptVal) -> usize {
    o.data.string.size
}

// =============================================================================
// OptVal Type Predicates (FFI)
// =============================================================================

/// Check if OptVal is nil.
#[no_mangle]
pub extern "C" fn rs_optval_is_nil(o: OptVal) -> c_int {
    c_int::from(o.is_nil())
}

/// Check if OptVal is boolean.
#[no_mangle]
pub extern "C" fn rs_optval_is_boolean(o: OptVal) -> c_int {
    c_int::from(o.is_boolean())
}

/// Check if OptVal is number.
#[no_mangle]
pub extern "C" fn rs_optval_is_number(o: OptVal) -> c_int {
    c_int::from(o.is_number())
}

/// Check if OptVal is string.
#[no_mangle]
pub extern "C" fn rs_optval_is_string(o: OptVal) -> c_int {
    c_int::from(o.is_string())
}

// =============================================================================
// Tests
// =============================================================================

// =============================================================================
// SctxT: Rust mirror of C's sctx_T
// =============================================================================

/// Rust mirror of C `sctx_T` (24 bytes).
///
/// C definition (eval/typval_defs.h):
///   typedef struct {
///     scid_T sc_sid;     // int, offset 0
///     int sc_seq;        // int, offset 4
///     linenr_T sc_lnum;  // int32_t, offset 8
///     // 4 bytes implicit padding for sc_chan alignment
///     uint64_t sc_chan;  // offset 16
///   } sctx_T;            // sizeof = 24
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SctxT {
    /// Script ID (scid_T = int).
    pub sc_sid: c_int,
    /// Sourcing sequence number.
    pub sc_seq: c_int,
    /// Line number in script (linenr_T = int32_t).
    pub sc_lnum: i32,
    // 4 bytes implicit padding here (repr(C) aligns sc_chan to 8 bytes)
    /// Channel ID (only used when sc_sid is SID_API_CLIENT).
    pub sc_chan: u64,
}

// Compile-time layout guard: must match C sctx_T in eval/typval_defs.h exactly.
// Any drift from the C definition will cause a build failure here.
const _: () = {
    assert!(core::mem::size_of::<SctxT>() == 24);
    assert!(core::mem::offset_of!(SctxT, sc_sid) == 0);
    assert!(core::mem::offset_of!(SctxT, sc_seq) == 4);
    assert!(core::mem::offset_of!(SctxT, sc_lnum) == 8);
    assert!(core::mem::offset_of!(SctxT, sc_chan) == 16);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optval_nil() {
        let o = OptVal::nil();
        assert!(o.is_nil());
        assert!(!o.is_boolean());
        assert!(!o.is_number());
        assert!(!o.is_string());
    }

    #[test]
    fn test_optval_boolean() {
        let o = OptVal::boolean(1);
        assert!(o.is_boolean());
        assert!(!o.is_nil());
        unsafe {
            assert_eq!(o.data.boolean, 1);
        }
    }

    #[test]
    fn test_optval_number() {
        let o = OptVal::number(42);
        assert!(o.is_number());
        unsafe {
            assert_eq!(o.data.number, 42);
        }
    }

    #[test]
    fn test_optval_string() {
        let s = String_ {
            data: ptr::null_mut(),
            size: 0,
        };
        let o = OptVal::string(s);
        assert!(o.is_string());
    }

    #[test]
    fn test_optval_default() {
        let o = OptVal::default();
        assert!(o.is_nil());
    }
}
