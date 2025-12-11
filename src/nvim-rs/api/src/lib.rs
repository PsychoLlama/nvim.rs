//! API utilities for Neovim
//!
//! This crate provides C-compatible implementations of API utility functions.

use std::ffi::{c_char, c_int};
use std::ptr;

/// Mask for all internal calls
const INTERNAL_CALL_MASK: u64 = 1u64 << 63;

/// ObjectType enum values matching C definitions
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Nil = 0,
    Boolean = 1,
    Integer = 2,
    Float = 3,
    String = 4,
    Array = 5,
    Dict = 6,
    LuaRef = 7,
    Buffer = 8,
    Window = 9,
    Tabpage = 10,
}

/// ErrorType enum values matching C definitions
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    None = -1,
    Exception = 0,
    Validation = 1,
}

/// Check whether a channel_id refers to an internal call.
///
/// Internal calls include Vimscript code and Lua code, identified by
/// having the high bit set in the channel_id.
///
/// # Arguments
/// * `channel_id` - The channel ID to check
///
/// # Returns
/// 1 if the channel_id refers to an internal channel, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_is_internal_call(channel_id: u64) -> c_int {
    c_int::from((channel_id & INTERNAL_CALL_MASK) != 0)
}

/// Static string literals for type names
static NIL_STR: &[u8] = b"nil\0";
static BOOLEAN_STR: &[u8] = b"Boolean\0";
static INTEGER_STR: &[u8] = b"Integer\0";
static FLOAT_STR: &[u8] = b"Float\0";
static STRING_STR: &[u8] = b"String\0";
static ARRAY_STR: &[u8] = b"Array\0";
static DICT_STR: &[u8] = b"Dict\0";
static FUNCTION_STR: &[u8] = b"Function\0";
static BUFFER_STR: &[u8] = b"Buffer\0";
static WINDOW_STR: &[u8] = b"Window\0";
static TABPAGE_STR: &[u8] = b"Tabpage\0";
static UNKNOWN_STR: &[u8] = b"Unknown\0";

/// Get the name of an ObjectType as a C string.
///
/// # Arguments
/// * `t` - The ObjectType value
///
/// # Returns
/// A pointer to a static null-terminated string with the type name
#[no_mangle]
pub extern "C" fn rs_api_typename(t: c_int) -> *const c_char {
    let bytes = match t {
        0 => NIL_STR,      // kObjectTypeNil
        1 => BOOLEAN_STR,  // kObjectTypeBoolean
        2 => INTEGER_STR,  // kObjectTypeInteger
        3 => FLOAT_STR,    // kObjectTypeFloat
        4 => STRING_STR,   // kObjectTypeString
        5 => ARRAY_STR,    // kObjectTypeArray
        6 => DICT_STR,     // kObjectTypeDict
        7 => FUNCTION_STR, // kObjectTypeLuaRef
        8 => BUFFER_STR,   // kObjectTypeBuffer
        9 => WINDOW_STR,   // kObjectTypeWindow
        10 => TABPAGE_STR, // kObjectTypeTabpage
        _ => UNKNOWN_STR,
    };
    bytes.as_ptr() as *const c_char
}

// FFI declarations for C functions we need
extern "C" {
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_char);
    fn xmemdupz(data: *const c_char, len: usize) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strnlen(s: *const c_char, maxlen: usize) -> usize;
}

/// String struct matching C definition
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NvimString {
    pub data: *mut c_char,
    pub size: usize,
}

impl Default for NvimString {
    fn default() -> Self {
        Self {
            data: ptr::null_mut(),
            size: 0,
        }
    }
}

/// Creates a String using the given C string without copying.
///
/// # Safety
/// The input string must be valid for reads and null-terminated.
///
/// # Arguments
/// * `str` - The C string to use (not copied)
///
/// # Returns
/// A String struct pointing to the input data, or empty if str was NULL
#[no_mangle]
pub unsafe extern "C" fn rs_cstr_as_string(s: *const c_char) -> NvimString {
    if s.is_null() {
        return NvimString::default();
    }
    let len = strlen(s);
    NvimString {
        data: s as *mut c_char,
        size: len,
    }
}

/// Copies a C string into a String (binary safe string, characters + length).
/// The resulting string is also NUL-terminated.
///
/// # Safety
/// The input string must be valid for reads and null-terminated.
///
/// # Arguments
/// * `str` - The C string to copy
///
/// # Returns
/// A newly allocated String, or empty if str was NULL
#[no_mangle]
pub unsafe extern "C" fn rs_cstr_to_string(s: *const c_char) -> NvimString {
    if s.is_null() {
        return NvimString::default();
    }
    let len = strlen(s);
    NvimString {
        data: xmemdupz(s, len),
        size: len,
    }
}

/// Copies buffer to an allocated String.
/// The resulting string is also NUL-terminated.
///
/// # Safety
/// The buffer must be valid for reads of `size` bytes.
///
/// # Arguments
/// * `buf` - The buffer to copy
/// * `size` - The length of the buffer
///
/// # Returns
/// A newly allocated String
#[no_mangle]
pub unsafe extern "C" fn rs_cbuf_to_string(buf: *const c_char, size: usize) -> NvimString {
    NvimString {
        data: xmemdupz(buf, size),
        size,
    }
}

/// Creates a String using a buffer without copying.
///
/// # Arguments
/// * `buf` - The buffer to use (not copied)
/// * `size` - The length of the buffer
///
/// # Returns
/// A String struct pointing to the input data
#[no_mangle]
pub extern "C" fn rs_cbuf_as_string(buf: *mut c_char, size: usize) -> NvimString {
    NvimString { data: buf, size }
}

/// Copies a C string up to maxsize into a newly allocated String.
///
/// # Safety
/// The input string must be valid for reads up to maxsize bytes.
///
/// # Arguments
/// * `str` - The C string to copy
/// * `maxsize` - Maximum number of bytes to copy
///
/// # Returns
/// A newly allocated String
#[no_mangle]
pub unsafe extern "C" fn rs_cstrn_to_string(s: *const c_char, maxsize: usize) -> NvimString {
    let len = strnlen(s, maxsize);
    rs_cbuf_to_string(s, len)
}

/// Creates a String using a C string up to maxsize without copying.
///
/// # Safety
/// The input string must be valid for reads up to maxsize bytes.
///
/// # Arguments
/// * `str` - The C string to use (not copied)
/// * `maxsize` - Maximum number of bytes to consider
///
/// # Returns
/// A String struct pointing to the input data
#[no_mangle]
pub unsafe extern "C" fn rs_cstrn_as_string(s: *mut c_char, maxsize: usize) -> NvimString {
    let len = strnlen(s, maxsize);
    NvimString { data: s, size: len }
}

/// Allocates a String consisting of a single char.
/// Does not support multibyte characters.
/// The resulting string is NUL-terminated.
///
/// # Arguments
/// * `c` - The char to convert
///
/// # Returns
/// A newly allocated String (empty if c was NUL)
#[no_mangle]
pub unsafe extern "C" fn rs_cchar_to_string(c: c_char) -> NvimString {
    let buf = [c, 0];
    NvimString {
        data: xmemdupz(buf.as_ptr(), 1),
        size: if c != 0 { 1 } else { 0 },
    }
}

/// Free a String's data.
///
/// # Safety
/// The string's data must have been allocated with xmalloc/xmemdupz.
#[no_mangle]
pub unsafe extern "C" fn rs_api_free_string(value: NvimString) {
    xfree(value.data);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_internal_call() {
        // External calls (bit 63 not set)
        assert_eq!(rs_is_internal_call(0), 0);
        assert_eq!(rs_is_internal_call(1), 0);
        assert_eq!(rs_is_internal_call(12345), 0);
        assert_eq!(rs_is_internal_call((1u64 << 62) - 1), 0);

        // Internal calls (bit 63 set)
        // VIML_INTERNAL_CALL = INTERNAL_CALL_MASK
        assert_ne!(rs_is_internal_call(INTERNAL_CALL_MASK), 0);
        // LUA_INTERNAL_CALL = VIML_INTERNAL_CALL + 1
        assert_ne!(rs_is_internal_call(INTERNAL_CALL_MASK + 1), 0);
        // Any value with bit 63 set
        assert_ne!(rs_is_internal_call(u64::MAX), 0);
        assert_ne!(rs_is_internal_call(INTERNAL_CALL_MASK | 0x12345), 0);
    }

    #[test]
    fn test_api_typename() {
        unsafe {
            // Check each type returns correct string
            let nil = std::ffi::CStr::from_ptr(rs_api_typename(0));
            assert_eq!(nil.to_str().unwrap(), "nil");

            let boolean = std::ffi::CStr::from_ptr(rs_api_typename(1));
            assert_eq!(boolean.to_str().unwrap(), "Boolean");

            let integer = std::ffi::CStr::from_ptr(rs_api_typename(2));
            assert_eq!(integer.to_str().unwrap(), "Integer");

            let float = std::ffi::CStr::from_ptr(rs_api_typename(3));
            assert_eq!(float.to_str().unwrap(), "Float");

            let string = std::ffi::CStr::from_ptr(rs_api_typename(4));
            assert_eq!(string.to_str().unwrap(), "String");

            let array = std::ffi::CStr::from_ptr(rs_api_typename(5));
            assert_eq!(array.to_str().unwrap(), "Array");

            let dict = std::ffi::CStr::from_ptr(rs_api_typename(6));
            assert_eq!(dict.to_str().unwrap(), "Dict");

            let luaref = std::ffi::CStr::from_ptr(rs_api_typename(7));
            assert_eq!(luaref.to_str().unwrap(), "Function");

            let buffer = std::ffi::CStr::from_ptr(rs_api_typename(8));
            assert_eq!(buffer.to_str().unwrap(), "Buffer");

            let window = std::ffi::CStr::from_ptr(rs_api_typename(9));
            assert_eq!(window.to_str().unwrap(), "Window");

            let tabpage = std::ffi::CStr::from_ptr(rs_api_typename(10));
            assert_eq!(tabpage.to_str().unwrap(), "Tabpage");

            // Unknown type
            let unknown = std::ffi::CStr::from_ptr(rs_api_typename(99));
            assert_eq!(unknown.to_str().unwrap(), "Unknown");
        }
    }
}
