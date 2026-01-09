//! Lua to Typval conversion helpers
//!
//! This module provides FFI helpers for converting Lua values to Neovim API values.
//! These functions wrap the C `nlua_pop_*` functions from converter.c.

use std::ffi::c_int;

use crate::state::LuaState;
use crate::to_lua::{Array, Dict, Float, Integer, NvimString, Object};

// =============================================================================
// Error type (matches Neovim's Error struct)
// =============================================================================

/// Error type enum
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    None = -1,
    Exception = 0,
    Validation = 1,
}

/// Error struct matching Neovim's Error
#[repr(C)]
pub struct Error {
    pub r#type: ErrorType,
    pub msg: *mut std::ffi::c_char,
}

impl Error {
    /// Check if an error is set
    #[must_use]
    pub fn is_set(&self) -> bool {
        self.r#type != ErrorType::None
    }
}

/// Arena type (opaque handle)
#[repr(C)]
pub struct Arena {
    _private: [u8; 0],
}

/// Boolean type
pub type Boolean = bool;

// =============================================================================
// C FFI declarations for nlua_pop_* functions
// =============================================================================

extern "C" {
    fn nlua_pop_Integer(lstate: *mut LuaState, arena: *mut Arena, err: *mut Error) -> Integer;
    fn nlua_pop_Float(lstate: *mut LuaState, arena: *mut Arena, err: *mut Error) -> Float;
    fn nlua_pop_Boolean(lstate: *mut LuaState, arena: *mut Arena, err: *mut Error) -> Boolean;
    fn nlua_pop_Boolean_strict(lstate: *mut LuaState, err: *mut Error) -> Boolean;
    fn nlua_pop_String(lstate: *mut LuaState, arena: *mut Arena, err: *mut Error) -> NvimString;
    fn nlua_pop_Array(lstate: *mut LuaState, arena: *mut Arena, err: *mut Error) -> Array;
    fn nlua_pop_Dict(
        lstate: *mut LuaState,
        r#ref: bool,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn nlua_pop_Object(
        lstate: *mut LuaState,
        r#ref: bool,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nlua_pop_typval(lstate: *mut LuaState, ret_tv: *mut std::ffi::c_void) -> bool;

    // Lua C API functions for reading values
    fn lua_tonumber(lstate: *mut LuaState, idx: c_int) -> f64;
    fn lua_toboolean(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_tolstring(lstate: *mut LuaState, idx: c_int, len: *mut usize)
        -> *const std::ffi::c_char;
    fn lua_type(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_settop(lstate: *mut LuaState, idx: c_int);
    fn lua_gettop(lstate: *mut LuaState) -> c_int;
}

// lua_pop is a macro: #define lua_pop(L,n) lua_settop(L, -(n)-1)
#[inline]
unsafe fn lua_pop(lstate: *mut LuaState, n: c_int) {
    lua_settop(lstate, -n - 1);
}

// =============================================================================
// Rust FFI exports
// =============================================================================

/// Pop an Integer from the Lua stack.
///
/// Validates that the value is a Lua number and converts to Integer.
/// Always pops one value from the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `err` must be a valid Error pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_integer(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> Integer {
    nlua_pop_Integer(lstate, arena, err)
}

/// Pop a Float from the Lua stack.
///
/// Accepts Lua numbers or typed float tables.
/// Always pops one value from the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `err` must be a valid Error pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_float(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> Float {
    nlua_pop_Float(lstate, arena, err)
}

/// Pop a Boolean from the Lua stack.
///
/// Uses Lua semantics for booleans (any value can be coerced).
/// Always pops one value from the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `err` must be a valid Error pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_boolean(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> Boolean {
    nlua_pop_Boolean(lstate, arena, err)
}

/// Pop a Boolean from the Lua stack with strict validation.
///
/// Follows API conventions - only accepts boolean, number, or nil.
/// Always pops one value from the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `err` must be a valid Error pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_boolean_strict(
    lstate: *mut LuaState,
    err: *mut Error,
) -> Boolean {
    nlua_pop_Boolean_strict(lstate, err)
}

/// Pop a String from the Lua stack.
///
/// Validates that the value is a Lua string.
/// Always pops one value from the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `err` must be a valid Error pointer.
/// - `arena` can be NULL for heap allocation or a valid Arena pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_string(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> NvimString {
    nlua_pop_String(lstate, arena, err)
}

/// Pop an Array from the Lua stack.
///
/// Validates that the value is an array-like Lua table.
/// Always pops one value from the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `err` must be a valid Error pointer.
/// - `arena` can be NULL for heap allocation or a valid Arena pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_array(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> Array {
    nlua_pop_Array(lstate, arena, err)
}

/// Pop a Dict from the Lua stack.
///
/// Validates that the value is a dict-like Lua table.
/// Always pops one value from the stack.
///
/// # Arguments
/// * `ref_` - If true, preserve LuaRefs in the dict; if false, convert them.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `err` must be a valid Error pointer.
/// - `arena` can be NULL for heap allocation or a valid Arena pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_dict(
    lstate: *mut LuaState,
    ref_: bool,
    arena: *mut Arena,
    err: *mut Error,
) -> Dict {
    nlua_pop_Dict(lstate, ref_, arena, err)
}

/// Pop an Object from the Lua stack.
///
/// Converts any Lua value to the appropriate Object type.
/// Always pops one value from the stack.
///
/// # Arguments
/// * `ref_` - If true, preserve LuaRefs; if false, convert functions to nil.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `err` must be a valid Error pointer.
/// - `arena` can be NULL for heap allocation or a valid Arena pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_object(
    lstate: *mut LuaState,
    ref_: bool,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    nlua_pop_Object(lstate, ref_, arena, err)
}

/// Pop a typval from the Lua stack.
///
/// Converts a Lua value to a Vimscript typval_T.
/// Always pops one value from the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `ret_tv` must be a valid typval_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_typval(
    lstate: *mut LuaState,
    ret_tv: *mut std::ffi::c_void,
) -> bool {
    nlua_pop_typval(lstate, ret_tv)
}

// =============================================================================
// Direct Lua stack reading operations
// =============================================================================

/// Get a number from the Lua stack without popping.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
/// `idx` must be a valid stack index.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_tonumber(lstate: *mut LuaState, idx: c_int) -> f64 {
    lua_tonumber(lstate, idx)
}

/// Get a boolean from the Lua stack without popping.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
/// `idx` must be a valid stack index.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_toboolean(lstate: *mut LuaState, idx: c_int) -> bool {
    lua_toboolean(lstate, idx) != 0
}

/// Get a string from the Lua stack without popping.
///
/// Returns the string pointer and sets `len` to the string length.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `idx` must be a valid stack index.
/// - `len` can be NULL or a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_tolstring(
    lstate: *mut LuaState,
    idx: c_int,
    len: *mut usize,
) -> *const std::ffi::c_char {
    lua_tolstring(lstate, idx, len)
}

/// Get the type of a Lua value on the stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
/// `idx` must be a valid stack index.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_type(lstate: *mut LuaState, idx: c_int) -> c_int {
    lua_type(lstate, idx)
}

/// Pop values from the Lua stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
/// `n` must not exceed the stack size.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_pop(lstate: *mut LuaState, n: c_int) {
    lua_pop(lstate, n);
}

/// Get the top of the Lua stack (number of elements).
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_gettop(lstate: *mut LuaState) -> c_int {
    lua_gettop(lstate)
}

// =============================================================================
// Safe Rust wrappers
// =============================================================================

/// Get the type of a Lua value on the stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[inline]
#[must_use]
pub unsafe fn get_type(lstate: *mut LuaState, idx: c_int) -> crate::types::LuaType {
    crate::types::LuaType::from_raw(lua_type(lstate, idx)).unwrap_or(crate::types::LuaType::None)
}

/// Get the stack height.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[inline]
#[must_use]
pub unsafe fn stack_height(lstate: *mut LuaState) -> c_int {
    lua_gettop(lstate)
}

/// Pop n values from the stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[inline]
pub unsafe fn pop(lstate: *mut LuaState, n: c_int) {
    lua_pop(lstate, n);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_values() {
        assert_eq!(ErrorType::None as i32, -1);
        assert_eq!(ErrorType::Exception as i32, 0);
        assert_eq!(ErrorType::Validation as i32, 1);
    }

    #[test]
    fn test_error_is_set() {
        let error_none = Error {
            r#type: ErrorType::None,
            msg: std::ptr::null_mut(),
        };
        assert!(!error_none.is_set());

        let error_set = Error {
            r#type: ErrorType::Exception,
            msg: std::ptr::null_mut(),
        };
        assert!(error_set.is_set());
    }

    #[test]
    fn test_arena_is_opaque() {
        // Arena should be zero-sized (opaque handle)
        assert_eq!(std::mem::size_of::<Arena>(), 0);
    }
}
