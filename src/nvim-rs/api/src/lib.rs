//! API utilities for Neovim
//!
//! This crate provides C-compatible implementations of API utility functions.

#![allow(clippy::items_after_test_module)]
#![allow(dead_code)]

pub mod buffer;
pub mod window;

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
#[unsafe(export_name = "api_typename")]
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
    #[allow(dead_code)]
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_char);
    fn xmemdupz(data: *const c_char, len: usize) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strnlen(s: *const c_char, maxlen: usize) -> usize;
    fn arena_memdupz(arena: *mut Arena, data: *const c_char, len: usize) -> *mut c_char;
    fn api_set_error(err: *mut Error, err_type: c_int, format: *const c_char, ...);
}

/// Opaque Arena type from C
#[repr(C)]
pub struct Arena {
    _private: [u8; 0],
}

/// Error struct matching C definition
#[repr(C)]
pub struct Error {
    pub err_type: c_int,
    pub msg: *mut c_char,
}

/// LuaRef type (same as Integer in C)
pub type LuaRef = i64;

/// Object union data
#[repr(C)]
#[derive(Clone, Copy)]
pub union ObjectData {
    pub boolean: bool,
    pub integer: i64,
    pub floating: f64,
    pub string: NvimString,
    pub array: Array,
    pub dict: Dict,
    pub luaref: LuaRef,
}

/// Array struct matching C definition (kvec)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Array {
    pub size: usize,
    pub capacity: usize,
    pub items: *mut Object,
}

/// KeyValuePair struct matching C definition
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeyValuePair {
    pub key: NvimString,
    pub value: Object,
}

/// Dict struct matching C definition (kvec of KeyValuePair)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Dict {
    pub size: usize,
    pub capacity: usize,
    pub items: *mut KeyValuePair,
}

/// Object struct matching C definition
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Object {
    pub obj_type: c_int,
    pub data: ObjectData,
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
#[unsafe(export_name = "cstr_as_string")]
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
#[unsafe(export_name = "cstr_to_string")]
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
#[unsafe(export_name = "cbuf_to_string")]
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
#[unsafe(export_name = "cstrn_to_string")]
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
#[unsafe(export_name = "cstrn_as_string")]
pub unsafe extern "C" fn rs_cstrn_as_string(s: *mut c_char, maxsize: usize) -> NvimString {
    let len = strnlen(s, maxsize);
    NvimString { data: s, size: len }
}

/// Allocates a String consisting of a single char.
/// Does not support multibyte characters.
/// The resulting string is NUL-terminated.
///
/// # Safety
/// Calls xmemdupz which must be available at link time.
///
/// # Arguments
/// * `c` - The char to convert
///
/// # Returns
/// A newly allocated String (empty if c was NUL)
#[unsafe(export_name = "cchar_to_string")]
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
#[unsafe(export_name = "api_free_string")]
pub unsafe extern "C" fn rs_api_free_string(value: NvimString) {
    xfree(value.data);
}

// FFI declaration for api_free_luaref (in Lua executor)
extern "C" {
    fn api_free_luaref(r: LuaRef);
}

/// ObjectType constants for free functions
const K_OBJECT_TYPE_STRING: c_int = 4;
const K_OBJECT_TYPE_ARRAY: c_int = 5;
const K_OBJECT_TYPE_DICT: c_int = 6;
const K_OBJECT_TYPE_LUAREF: c_int = 7;

/// Free an Object and all its nested data.
///
/// # Safety
/// The object's data must have been allocated with xmalloc/xmemdupz.
/// For strings, arrays, and dicts, frees recursively.
///
/// # Arguments
/// * `value` - The Object to free
#[unsafe(export_name = "api_free_object")]
pub unsafe extern "C" fn rs_api_free_object(value: Object) {
    match value.obj_type {
        // These types have no heap data
        K_OBJECT_TYPE_NIL
        | K_OBJECT_TYPE_BOOLEAN
        | K_OBJECT_TYPE_INTEGER
        | 3 /* Float */
        | 8 /* Buffer */
        | 9 /* Window */
        | 10 /* Tabpage */ => {}

        K_OBJECT_TYPE_STRING => {
            rs_api_free_string(value.data.string);
        }

        K_OBJECT_TYPE_ARRAY => {
            rs_api_free_array(value.data.array);
        }

        K_OBJECT_TYPE_DICT => {
            rs_api_free_dict(value.data.dict);
        }

        K_OBJECT_TYPE_LUAREF => {
            api_free_luaref(value.data.luaref);
        }

        _ => {}
    }
}

/// Free an Array and all its items recursively.
///
/// # Safety
/// The array's items must have been allocated with xmalloc.
///
/// # Arguments
/// * `value` - The Array to free
#[unsafe(export_name = "api_free_array")]
pub unsafe extern "C" fn rs_api_free_array(value: Array) {
    for i in 0..value.size {
        rs_api_free_object(*value.items.add(i));
    }
    xfree(value.items as *mut c_char);
}

/// Free a Dict and all its key-value pairs recursively.
///
/// # Safety
/// The dict's items must have been allocated with xmalloc.
///
/// # Arguments
/// * `value` - The Dict to free
#[unsafe(export_name = "api_free_dict")]
pub unsafe extern "C" fn rs_api_free_dict(value: Dict) {
    for i in 0..value.size {
        let item = &*value.items.add(i);
        rs_api_free_string(item.key);
        rs_api_free_object(item.value);
    }
    xfree(value.items as *mut c_char);
}

/// ObjectType constants
const K_OBJECT_TYPE_NIL: c_int = 0;
const K_OBJECT_TYPE_BOOLEAN: c_int = 1;
const K_OBJECT_TYPE_INTEGER: c_int = 2;

/// ErrorType constants
const K_ERROR_TYPE_VALIDATION: c_int = 1;

/// Force object to bool.
/// If it fails, returns false and sets err.
///
/// # Safety
/// `what` must be a valid null-terminated C string.
/// `err` must be a valid pointer to an Error struct.
///
/// # Arguments
/// * `obj` - The object to coerce to a boolean
/// * `what` - The name of the object, used for error message
/// * `nil_value` - What to return if the type is nil
/// * `err` - Set if there was an error in converting to a bool
///
/// # Returns
/// The boolean value of the object
#[unsafe(export_name = "api_object_to_bool")]
pub unsafe extern "C" fn rs_api_object_to_bool(
    obj: Object,
    what: *const c_char,
    nil_value: bool,
    err: *mut Error,
) -> bool {
    match obj.obj_type {
        K_OBJECT_TYPE_BOOLEAN => obj.data.boolean,
        K_OBJECT_TYPE_INTEGER => obj.data.integer != 0, // C semantics: non-zero int is true
        K_OBJECT_TYPE_NIL => nil_value,                 // caller decides what NIL means
        _ => {
            // Set error: "%s is not a boolean"
            static FMT: &[u8] = b"%s is not a boolean\0";
            api_set_error(
                err,
                K_ERROR_TYPE_VALIDATION,
                FMT.as_ptr() as *const c_char,
                what,
            );
            false
        }
    }
}

/// Copy a String, allocating new memory.
/// If arena is non-NULL, uses arena allocation; otherwise uses xmalloc.
///
/// # Safety
/// `arena` must be NULL or a valid Arena pointer.
/// `str` must have valid data pointer if size > 0.
///
/// # Arguments
/// * `str` - The String to copy
/// * `arena` - Arena for allocation (can be NULL for global allocation)
///
/// # Returns
/// A newly allocated copy of the string
#[unsafe(export_name = "copy_string")]
pub unsafe extern "C" fn rs_copy_string(str: NvimString, arena: *mut Arena) -> NvimString {
    if str.data.is_null() {
        return NvimString::default();
    }
    NvimString {
        data: arena_memdupz(arena, str.data, str.size),
        size: str.size,
    }
}

/// ErrorType constants for comparison
const K_ERROR_TYPE_NONE: c_int = -1;

/// Check if an error is set.
///
/// # Safety
/// `err` must be NULL or a valid pointer to an Error struct.
///
/// # Arguments
/// * `err` - The Error to check
///
/// # Returns
/// true if the error type is not kErrorTypeNone
#[no_mangle]
pub unsafe extern "C" fn rs_error_set(err: *const Error) -> bool {
    if err.is_null() {
        return false;
    }
    (*err).err_type != K_ERROR_TYPE_NONE
}

/// Clear an error, freeing its message.
///
/// # Safety
/// `err` must be a valid pointer to an Error struct.
/// If err->msg is non-null, it must have been allocated with xmalloc.
#[unsafe(export_name = "api_clear_error")]
pub unsafe extern "C" fn rs_api_clear_error(err: *mut Error) {
    if err.is_null() {
        return;
    }
    let e = &mut *err;
    if e.err_type == K_ERROR_TYPE_NONE {
        return;
    }
    if !e.msg.is_null() {
        xfree(e.msg);
        e.msg = ptr::null_mut();
    }
    e.err_type = K_ERROR_TYPE_NONE;
}

// FFI for xstrndup and strchr
extern "C" {
    fn xstrndup(str: *const c_char, len: usize) -> *mut c_char;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
}

/// Creates "Invalid …" message and sets it on `err`.
///
/// # Safety
/// All string parameters must be valid null-terminated C strings.
/// `err` must be a valid pointer to an Error struct.
///
/// # Arguments
/// * `err` - Error struct to set
/// * `name` - Name of the parameter/field (quoted if no spaces, unquoted otherwise)
/// * `val_s` - String value (NULL for numeric error, empty for no value)
/// * `val_n` - Numeric value (only used if val_s is NULL)
/// * `quote_val` - Whether to quote string values
#[unsafe(export_name = "api_err_invalid")]
pub unsafe extern "C" fn rs_api_err_invalid(
    err: *mut Error,
    name: *const c_char,
    val_s: *const c_char,
    val_n: i64,
    quote_val: bool,
) {
    // Treat `name` without whitespace as a parameter (surround in quotes).
    // Treat `name` with whitespace as a description (no quotes).
    let has_space = !strchr(name, b' ' as c_int).is_null();

    // No value case: val_s is non-null but empty
    if !val_s.is_null() && *val_s == 0 {
        if has_space {
            static FMT: &[u8] = b"Invalid %s\0";
            api_set_error(
                err,
                K_ERROR_TYPE_VALIDATION,
                FMT.as_ptr() as *const c_char,
                name,
            );
        } else {
            static FMT: &[u8] = b"Invalid '%s'\0";
            api_set_error(
                err,
                K_ERROR_TYPE_VALIDATION,
                FMT.as_ptr() as *const c_char,
                name,
            );
        }
        return;
    }

    // Number value case: val_s is NULL
    if val_s.is_null() {
        if has_space {
            static FMT: &[u8] = b"Invalid %s: %lld\0";
            api_set_error(
                err,
                K_ERROR_TYPE_VALIDATION,
                FMT.as_ptr() as *const c_char,
                name,
                val_n,
            );
        } else {
            static FMT: &[u8] = b"Invalid '%s': %lld\0";
            api_set_error(
                err,
                K_ERROR_TYPE_VALIDATION,
                FMT.as_ptr() as *const c_char,
                name,
                val_n,
            );
        }
        return;
    }

    // String value case
    if has_space {
        if quote_val {
            static FMT: &[u8] = b"Invalid %s: '%s'\0";
            api_set_error(
                err,
                K_ERROR_TYPE_VALIDATION,
                FMT.as_ptr() as *const c_char,
                name,
                val_s,
            );
        } else {
            static FMT: &[u8] = b"Invalid %s: %s\0";
            api_set_error(
                err,
                K_ERROR_TYPE_VALIDATION,
                FMT.as_ptr() as *const c_char,
                name,
                val_s,
            );
        }
    } else if quote_val {
        static FMT: &[u8] = b"Invalid '%s': '%s'\0";
        api_set_error(
            err,
            K_ERROR_TYPE_VALIDATION,
            FMT.as_ptr() as *const c_char,
            name,
            val_s,
        );
    } else {
        static FMT: &[u8] = b"Invalid '%s': %s\0";
        api_set_error(
            err,
            K_ERROR_TYPE_VALIDATION,
            FMT.as_ptr() as *const c_char,
            name,
            val_s,
        );
    }
}

/// Creates "Invalid …: expected …" message and sets it on `err`.
///
/// # Safety
/// All string parameters must be valid null-terminated C strings (actual may be NULL).
/// `err` must be a valid pointer to an Error struct.
///
/// # Arguments
/// * `err` - Error struct to set
/// * `name` - Name of the parameter/field
/// * `expected` - Expected type/value description
/// * `actual` - Actual type/value description (may be NULL)
#[unsafe(export_name = "api_err_exp")]
pub unsafe extern "C" fn rs_api_err_exp(
    err: *mut Error,
    name: *const c_char,
    expected: *const c_char,
    actual: *const c_char,
) {
    // Treat `name` without whitespace as a parameter (surround in quotes).
    // Treat `name` with whitespace as a description (no quotes).
    let has_space = !strchr(name, b' ' as c_int).is_null();

    if actual.is_null() {
        if has_space {
            static FMT: &[u8] = b"Invalid %s: expected %s\0";
            api_set_error(
                err,
                K_ERROR_TYPE_VALIDATION,
                FMT.as_ptr() as *const c_char,
                name,
                expected,
            );
        } else {
            static FMT: &[u8] = b"Invalid '%s': expected %s\0";
            api_set_error(
                err,
                K_ERROR_TYPE_VALIDATION,
                FMT.as_ptr() as *const c_char,
                name,
                expected,
            );
        }
        return;
    }

    if has_space {
        static FMT: &[u8] = b"Invalid %s: expected %s, got %s\0";
        api_set_error(
            err,
            K_ERROR_TYPE_VALIDATION,
            FMT.as_ptr() as *const c_char,
            name,
            expected,
            actual,
        );
    } else {
        static FMT: &[u8] = b"Invalid '%s': expected %s, got %s\0";
        api_set_error(
            err,
            K_ERROR_TYPE_VALIDATION,
            FMT.as_ptr() as *const c_char,
            name,
            expected,
            actual,
        );
    }
}

// FFI declarations for highlight functions
extern "C" {
    fn syn_check_group(name: *const c_char, len: usize) -> c_int;
    fn highlight_num_groups() -> c_int;
}

/// Convert an Object to a highlight group ID.
///
/// # Safety
/// `what` must be a valid null-terminated C string.
/// `err` must be a valid pointer to an Error struct.
///
/// # Arguments
/// * `obj` - The Object to convert (String or Integer)
/// * `what` - Description for error messages
/// * `err` - Error struct to set on failure
///
/// # Returns
/// The highlight group ID, or 0 on failure
#[unsafe(export_name = "object_to_hl_id")]
pub unsafe extern "C" fn rs_object_to_hl_id(
    obj: Object,
    what: *const c_char,
    err: *mut Error,
) -> c_int {
    match obj.obj_type {
        K_OBJECT_TYPE_STRING => {
            let str = obj.data.string;
            if str.size == 0 {
                return 0;
            }
            syn_check_group(str.data, str.size)
        }
        K_OBJECT_TYPE_INTEGER => {
            let id = obj.data.integer as c_int;
            let num_groups = highlight_num_groups();
            if 1 <= id && id <= num_groups {
                id
            } else {
                0
            }
        }
        _ => {
            static FMT: &[u8] = b"Invalid hl_group: %s\0";
            api_set_error(
                err,
                K_ERROR_TYPE_VALIDATION,
                FMT.as_ptr() as *const c_char,
                what,
            );
            0
        }
    }
}

/// Copies a String to an allocated, NUL-terminated C string.
///
/// # Safety
/// `str` must have valid data pointer if size > 0.
///
/// # Arguments
/// * `str` - The String to copy
///
/// # Returns
/// A newly allocated NUL-terminated C string
#[unsafe(export_name = "string_to_cstr")]
pub unsafe extern "C" fn rs_string_to_cstr(str: NvimString) -> *mut c_char {
    xstrndup(str.data, str.size)
}

/// GArray struct matching C definition
#[repr(C)]
pub struct GArray {
    pub ga_data: *mut c_char,
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
}

/// Return the owned memory of a ga as a String.
/// Reinitializes the ga to a valid empty state.
///
/// # Safety
/// `ga` must be a valid pointer to a garray_T.
///
/// # Arguments
/// * `ga` - The garray to take ownership of
///
/// # Returns
/// A String containing the ga's data
#[unsafe(export_name = "ga_take_string")]
pub unsafe extern "C" fn rs_ga_take_string(ga: *mut GArray) -> NvimString {
    if ga.is_null() {
        return NvimString::default();
    }
    let g = &mut *ga;
    let str = NvimString {
        data: g.ga_data,
        size: g.ga_len as usize,
    };
    g.ga_data = ptr::null_mut();
    g.ga_len = 0;
    g.ga_maxlen = 0;
    str
}

/// Free luarefs from an Object (only frees luarefs, not other data).
///
/// This function recursively frees luarefs from nested arrays and dicts,
/// but does not free the actual string/array/dict memory.
///
/// # Safety
/// The object must have valid pointers for array and dict items.
///
/// # Arguments
/// * `value` - The Object to free luarefs from
#[unsafe(export_name = "api_luarefs_free_object")]
pub unsafe extern "C" fn rs_api_luarefs_free_object(value: Object) {
    match value.obj_type {
        K_OBJECT_TYPE_LUAREF => {
            api_free_luaref(value.data.luaref);
        }
        K_OBJECT_TYPE_ARRAY => {
            rs_api_luarefs_free_array(value.data.array);
        }
        K_OBJECT_TYPE_DICT => {
            rs_api_luarefs_free_dict(value.data.dict);
        }
        _ => {}
    }
}

/// Free luarefs from an Array (only frees luarefs, not other data).
///
/// # Safety
/// The array's items must be valid pointers.
///
/// # Arguments
/// * `value` - The Array to free luarefs from
#[unsafe(export_name = "api_luarefs_free_array")]
pub unsafe extern "C" fn rs_api_luarefs_free_array(value: Array) {
    for i in 0..value.size {
        rs_api_luarefs_free_object(*value.items.add(i));
    }
}

/// Free luarefs from a Dict (only frees luarefs, not other data).
///
/// # Safety
/// The dict's items must be valid pointers.
///
/// # Arguments
/// * `value` - The Dict to free luarefs from
#[unsafe(export_name = "api_luarefs_free_dict")]
pub unsafe extern "C" fn rs_api_luarefs_free_dict(value: Dict) {
    for i in 0..value.size {
        let item = &*value.items.add(i);
        rs_api_luarefs_free_object(item.value);
    }
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

    #[test]
    fn test_object_type_enum_values() {
        // Verify ObjectType enum values match C definitions
        assert_eq!(ObjectType::Nil as i32, 0);
        assert_eq!(ObjectType::Boolean as i32, 1);
        assert_eq!(ObjectType::Integer as i32, 2);
        assert_eq!(ObjectType::Float as i32, 3);
        assert_eq!(ObjectType::String as i32, 4);
        assert_eq!(ObjectType::Array as i32, 5);
        assert_eq!(ObjectType::Dict as i32, 6);
        assert_eq!(ObjectType::LuaRef as i32, 7);
        assert_eq!(ObjectType::Buffer as i32, 8);
        assert_eq!(ObjectType::Window as i32, 9);
        assert_eq!(ObjectType::Tabpage as i32, 10);
    }

    #[test]
    fn test_error_type_enum_values() {
        // Verify ErrorType enum values match C definitions
        assert_eq!(ErrorType::None as i32, -1);
        assert_eq!(ErrorType::Exception as i32, 0);
        assert_eq!(ErrorType::Validation as i32, 1);
    }

    #[test]
    fn test_internal_call_mask() {
        // Verify INTERNAL_CALL_MASK has bit 63 set
        assert_eq!(INTERNAL_CALL_MASK, 1u64 << 63);
    }

    #[test]
    fn test_nvim_string_size() {
        // NvimString: ptr (8) + size_t (8) = 16 bytes on 64-bit
        assert_eq!(std::mem::size_of::<NvimString>(), 16);
    }

    #[test]
    fn test_array_size() {
        // Array: 3 size_t fields = 24 bytes on 64-bit
        assert_eq!(std::mem::size_of::<Array>(), 24);
    }

    #[test]
    fn test_dict_size() {
        // Dict: 3 size_t fields = 24 bytes on 64-bit
        assert_eq!(std::mem::size_of::<Dict>(), 24);
    }

    #[test]
    fn test_object_constants() {
        // Verify K_OBJECT_TYPE constants match ObjectType enum
        assert_eq!(K_OBJECT_TYPE_NIL, ObjectType::Nil as c_int);
        assert_eq!(K_OBJECT_TYPE_BOOLEAN, ObjectType::Boolean as c_int);
        assert_eq!(K_OBJECT_TYPE_INTEGER, ObjectType::Integer as c_int);
        assert_eq!(K_OBJECT_TYPE_STRING, ObjectType::String as c_int);
        assert_eq!(K_OBJECT_TYPE_ARRAY, ObjectType::Array as c_int);
        assert_eq!(K_OBJECT_TYPE_DICT, ObjectType::Dict as c_int);
        assert_eq!(K_OBJECT_TYPE_LUAREF, ObjectType::LuaRef as c_int);
    }

    #[test]
    fn test_error_type_constants() {
        // Verify K_ERROR_TYPE constants match ErrorType enum
        assert_eq!(K_ERROR_TYPE_NONE, ErrorType::None as c_int);
        assert_eq!(K_ERROR_TYPE_VALIDATION, ErrorType::Validation as c_int);
    }
}

// =============================================================================
// Object constructors - equivalent to C macros like BOOLEAN_OBJ, INTEGER_OBJ
// =============================================================================

impl Object {
    /// Create a nil object
    #[inline]
    pub const fn nil() -> Self {
        Self {
            obj_type: ObjectType::Nil as c_int,
            data: ObjectData { integer: 0 },
        }
    }

    /// Create a boolean object
    #[inline]
    pub const fn boolean(b: bool) -> Self {
        Self {
            obj_type: ObjectType::Boolean as c_int,
            data: ObjectData { boolean: b },
        }
    }

    /// Create an integer object
    #[inline]
    pub const fn integer(i: i64) -> Self {
        Self {
            obj_type: ObjectType::Integer as c_int,
            data: ObjectData { integer: i },
        }
    }

    /// Create a float object
    #[inline]
    pub const fn float(f: f64) -> Self {
        Self {
            obj_type: ObjectType::Float as c_int,
            data: ObjectData { floating: f },
        }
    }

    /// Create a string object (takes ownership of the NvimString)
    #[inline]
    pub const fn string(s: NvimString) -> Self {
        Self {
            obj_type: ObjectType::String as c_int,
            data: ObjectData { string: s },
        }
    }

    /// Create an array object
    #[inline]
    pub const fn array(a: Array) -> Self {
        Self {
            obj_type: ObjectType::Array as c_int,
            data: ObjectData { array: a },
        }
    }

    /// Create a dict object
    #[inline]
    pub const fn dict(d: Dict) -> Self {
        Self {
            obj_type: ObjectType::Dict as c_int,
            data: ObjectData { dict: d },
        }
    }
}

/// C-callable object constructors
#[no_mangle]
pub extern "C" fn rs_nil_obj() -> Object {
    Object::nil()
}

#[no_mangle]
pub extern "C" fn rs_boolean_obj(b: bool) -> Object {
    Object::boolean(b)
}

#[no_mangle]
pub extern "C" fn rs_integer_obj(i: i64) -> Object {
    Object::integer(i)
}

#[no_mangle]
pub extern "C" fn rs_float_obj(f: f64) -> Object {
    Object::float(f)
}

#[no_mangle]
pub extern "C" fn rs_string_obj(s: NvimString) -> Object {
    Object::string(s)
}

#[no_mangle]
pub extern "C" fn rs_array_obj(a: Array) -> Object {
    Object::array(a)
}

#[no_mangle]
pub extern "C" fn rs_dict_obj(d: Dict) -> Object {
    Object::dict(d)
}

// =============================================================================
// Arena allocation wrappers
// =============================================================================

/// Allocate an Array with a given capacity using arena allocation.
/// If arena is NULL, uses xmalloc.
///
/// # Safety
/// Arena must be NULL or a valid arena pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_arena_array(arena: *mut Arena, max_size: usize) -> Array {
    let items = if arena.is_null() {
        xmalloc(max_size * std::mem::size_of::<Object>()) as *mut Object
    } else {
        arena_alloc(arena, max_size * std::mem::size_of::<Object>()) as *mut Object
    };
    Array {
        size: 0,
        capacity: max_size,
        items,
    }
}

/// Allocate a Dict with a given capacity using arena allocation.
/// If arena is NULL, uses xmalloc.
///
/// # Safety
/// Arena must be NULL or a valid arena pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_arena_dict(arena: *mut Arena, max_size: usize) -> Dict {
    let items = if arena.is_null() {
        xmalloc(max_size * std::mem::size_of::<KeyValuePair>()) as *mut KeyValuePair
    } else {
        arena_alloc(arena, max_size * std::mem::size_of::<KeyValuePair>()) as *mut KeyValuePair
    };
    Dict {
        size: 0,
        capacity: max_size,
        items,
    }
}

extern "C" {
    fn arena_alloc(arena: *mut Arena, size: usize) -> *mut c_char;
}

// =============================================================================
// Dict/Array manipulation helpers
// =============================================================================

impl Array {
    /// Push an object to the array.
    ///
    /// # Safety
    /// The array must have been allocated with sufficient capacity.
    #[inline]
    pub unsafe fn push(&mut self, obj: Object) {
        debug_assert!(self.size < self.capacity);
        *self.items.add(self.size) = obj;
        self.size += 1;
    }

    /// Create an empty array (no allocation)
    #[inline]
    pub const fn empty() -> Self {
        Self {
            size: 0,
            capacity: 0,
            items: ptr::null_mut(),
        }
    }
}

impl Dict {
    /// Put a key-value pair into the dict.
    /// Key must be a static C string (not copied).
    ///
    /// # Safety
    /// - The dict must have been allocated with sufficient capacity.
    /// - The key must be a valid static C string that outlives the dict.
    #[inline]
    pub unsafe fn put_static(&mut self, key: *const c_char, value: Object) {
        debug_assert!(self.size < self.capacity);
        let pair = &mut *self.items.add(self.size);
        pair.key = NvimString {
            data: key as *mut c_char,
            size: strlen(key),
        };
        pair.value = value;
        self.size += 1;
    }

    /// Create an empty dict (no allocation)
    #[inline]
    pub const fn empty() -> Self {
        Self {
            size: 0,
            capacity: 0,
            items: ptr::null_mut(),
        }
    }
}

/// Push an object to an array.
///
/// # Safety
/// The array must have been allocated with sufficient capacity.
#[no_mangle]
pub unsafe extern "C" fn rs_array_push(arr: *mut Array, obj: Object) {
    (*arr).push(obj);
}

/// Put a key-value pair into a dict.
/// Key must be a static C string (not copied).
///
/// # Safety
/// - The dict must have been allocated with sufficient capacity.
/// - The key must be a valid static C string.
#[no_mangle]
pub unsafe extern "C" fn rs_dict_put_static(dict: *mut Dict, key: *const c_char, value: Object) {
    (*dict).put_static(key, value);
}

/// Create a String from a static C string literal.
/// Does not copy the data.
///
/// # Safety
/// The string must be a valid static C string.
#[no_mangle]
pub unsafe extern "C" fn rs_static_cstr(s: *const c_char) -> NvimString {
    NvimString {
        data: s as *mut c_char,
        size: strlen(s),
    }
}
