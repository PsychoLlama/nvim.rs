//! Typval to Lua conversion helpers
//!
//! This module provides FFI helpers for converting Neovim API values to Lua values.
//! These functions wrap the C `nlua_push_*` functions from converter.c.

use std::ffi::c_int;

use crate::state::LuaState;
use crate::types::push_flags;

// Re-export push flags for convenience
pub use crate::types::push_flags::{NLUA_PUSH_FREE_REFS, NLUA_PUSH_SPECIAL};

// =============================================================================
// API type aliases for documentation
// =============================================================================

/// Integer type (i64)
pub type Integer = i64;
/// Float type (f64)
pub type Float = f64;
/// Boolean type
pub type Boolean = bool;
/// Handle type for Buffer, Window, Tabpage
pub type Handle = c_int;

/// String type matching Neovim's String struct
#[repr(C)]
pub struct NvimString {
    pub data: *mut std::ffi::c_char,
    pub size: usize,
}

/// Array type matching Neovim's Array struct
#[repr(C)]
pub struct Array {
    pub items: *mut Object,
    pub size: usize,
    pub capacity: usize,
}

/// KeyValuePair for Dict entries
#[repr(C)]
pub struct KeyValuePair {
    pub key: NvimString,
    pub value: Object,
}

/// Dict type matching Neovim's Dict struct
#[repr(C)]
pub struct Dict {
    pub items: *mut KeyValuePair,
    pub size: usize,
    pub capacity: usize,
}

/// ObjectType enum matching Neovim's ObjectType
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

/// Object data union (simplified for FFI)
#[repr(C)]
pub union ObjectData {
    pub boolean: bool,
    pub integer: i64,
    pub floating: f64,
    pub string: std::mem::ManuallyDrop<NvimString>,
    pub array: std::mem::ManuallyDrop<Array>,
    pub dict: std::mem::ManuallyDrop<Dict>,
    pub luaref: c_int,
}

/// Object type matching Neovim's Object struct
#[repr(C)]
pub struct Object {
    pub r#type: ObjectType,
    pub data: ObjectData,
}

// =============================================================================
// C FFI declarations for nlua_push_* functions
// =============================================================================

extern "C" {
    fn nlua_push_Integer(lstate: *mut LuaState, n: Integer, flags: c_int);
    fn nlua_push_Float(lstate: *mut LuaState, f: Float, flags: c_int);
    fn nlua_push_Boolean(lstate: *mut LuaState, b: Boolean, flags: c_int);
    fn nlua_push_String(lstate: *mut LuaState, s: NvimString, flags: c_int);
    fn nlua_push_Array(lstate: *mut LuaState, array: Array, flags: c_int);
    fn nlua_push_Dict(lstate: *mut LuaState, dict: Dict, flags: c_int);
    fn nlua_push_Object(lstate: *mut LuaState, obj: *mut Object, flags: c_int);
    fn nlua_push_handle(lstate: *mut LuaState, item: Handle, flags: c_int);

    // Lua C API functions (for direct use)
    fn lua_pushnil(lstate: *mut LuaState);
    fn lua_pushnumber(lstate: *mut LuaState, n: f64);
    fn lua_pushboolean(lstate: *mut LuaState, b: c_int);
    fn lua_pushlstring(lstate: *mut LuaState, s: *const std::ffi::c_char, len: usize);
}

// =============================================================================
// Rust FFI exports
// =============================================================================

/// Push an Integer onto the Lua stack.
///
/// Converts to Lua number type.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_push_integer(lstate: *mut LuaState, n: Integer, flags: c_int) {
    nlua_push_Integer(lstate, n, flags);
}

/// Push a Float onto the Lua stack.
///
/// If `flags & NLUA_PUSH_SPECIAL`, creates a typed table with the float value.
/// Otherwise, pushes as a Lua number.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_push_float(lstate: *mut LuaState, f: Float, flags: c_int) {
    nlua_push_Float(lstate, f, flags);
}

/// Push a Boolean onto the Lua stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_push_boolean(lstate: *mut LuaState, b: Boolean, flags: c_int) {
    nlua_push_Boolean(lstate, b, flags);
}

/// Push a String onto the Lua stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
/// `s` must have valid data pointer (or be empty with size 0).
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_push_string(lstate: *mut LuaState, s: NvimString, flags: c_int) {
    nlua_push_String(lstate, s, flags);
}

/// Push an Array onto the Lua stack.
///
/// Creates a Lua table with sequential integer keys (1-based).
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
/// `array` must have valid items pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_push_array(lstate: *mut LuaState, array: Array, flags: c_int) {
    nlua_push_Array(lstate, array, flags);
}

/// Push a Dict onto the Lua stack.
///
/// Creates a Lua table with string keys.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
/// `dict` must have valid items pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_push_dict(lstate: *mut LuaState, dict: Dict, flags: c_int) {
    nlua_push_Dict(lstate, dict, flags);
}

/// Push an Object onto the Lua stack.
///
/// Dispatches based on object type to the appropriate push function.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
/// `obj` must be a valid Object pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_push_object(
    lstate: *mut LuaState,
    obj: *mut Object,
    flags: c_int,
) {
    nlua_push_Object(lstate, obj, flags);
}

/// Push a handle (Buffer, Window, Tabpage) onto the Lua stack.
///
/// Converts to Lua number type.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_push_handle(lstate: *mut LuaState, item: Handle, flags: c_int) {
    nlua_push_handle(lstate, item, flags);
}

// =============================================================================
// Direct Lua stack operations
// =============================================================================

/// Push nil onto the Lua stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_pushnil(lstate: *mut LuaState) {
    lua_pushnil(lstate);
}

/// Push a number onto the Lua stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_pushnumber(lstate: *mut LuaState, n: f64) {
    lua_pushnumber(lstate, n);
}

/// Push a boolean onto the Lua stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_pushboolean(lstate: *mut LuaState, b: bool) {
    lua_pushboolean(lstate, c_int::from(b));
}

/// Push a string onto the Lua stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
/// `s` must be a valid pointer with at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_pushlstring(
    lstate: *mut LuaState,
    s: *const std::ffi::c_char,
    len: usize,
) {
    lua_pushlstring(lstate, s, len);
}

// =============================================================================
// Safe Rust wrappers
// =============================================================================

/// Push an integer onto the Lua stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[inline]
pub unsafe fn push_integer(lstate: *mut LuaState, n: Integer, special: bool) {
    let flags = if special {
        push_flags::NLUA_PUSH_SPECIAL
    } else {
        0
    };
    nlua_push_Integer(lstate, n, flags);
}

/// Push a float onto the Lua stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[inline]
pub unsafe fn push_float(lstate: *mut LuaState, f: Float, special: bool) {
    let flags = if special {
        push_flags::NLUA_PUSH_SPECIAL
    } else {
        0
    };
    nlua_push_Float(lstate, f, flags);
}

/// Push a boolean onto the Lua stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[inline]
pub unsafe fn push_boolean(lstate: *mut LuaState, b: bool) {
    nlua_push_Boolean(lstate, b, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_type_values() {
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
    fn test_push_flags() {
        assert_eq!(NLUA_PUSH_SPECIAL, 0x01);
        assert_eq!(NLUA_PUSH_FREE_REFS, 0x02);
    }

    #[test]
    fn test_struct_sizes() {
        // Verify struct layouts are reasonable
        assert!(std::mem::size_of::<NvimString>() > 0);
        assert!(std::mem::size_of::<Array>() > 0);
        assert!(std::mem::size_of::<Dict>() > 0);
        assert!(std::mem::size_of::<Object>() > 0);
    }
}
