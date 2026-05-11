//! Lua to Neovim API value conversion helpers
//!
//! This module provides real Rust implementations of nlua_pop_* functions,
//! replacing C bodies from converter.c. Phase 2: scalar poppers.

#![allow(non_snake_case)]

use std::ffi::{c_char, c_int};

use crate::state::LuaState;
use crate::types::{LUA_TBOOLEAN, LUA_TNIL, LUA_TNUMBER};
use nvim_api::{Array, Dict, NvimString, Object};

/// Float type (f64)
pub type Float = f64;
/// Integer type (i64)
pub type Integer = i64;
/// Boolean type
pub type Boolean = bool;

/// LuaRef is typedef int in C
type LuaRef = c_int;

// API_INTEGER_MIN corresponds to INT64_MIN; MAX not needed (literal used instead)
const API_INTEGER_MIN: Integer = i64::MIN;

// kErrorTypeValidation = 1
const K_ERROR_VALIDATION: c_int = 1;
// kErrorTypeException = 0
const K_ERROR_EXCEPTION: c_int = 0;

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
    pub msg: *mut c_char,
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

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Lua C API functions for reading values
    fn lua_tonumber(lstate: *mut LuaState, idx: c_int) -> f64;
    fn lua_toboolean(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_tolstring(lstate: *mut LuaState, idx: c_int, len: *mut usize) -> *const c_char;
    fn lua_type(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_settop(lstate: *mut LuaState, idx: c_int);
    fn lua_gettop(lstate: *mut LuaState) -> c_int;

    // API error function (variadic; only used with literal format strings below)
    fn api_set_error(err: *mut Error, err_type: c_int, fmt: *const c_char, ...);

    // arena_memdupz for pop_String (Phase 3; unused until then)
    #[allow(dead_code)]
    fn arena_memdupz(arena: *mut Arena, data: *const c_char, len: usize) -> *mut c_char;

    // Still-C pop functions (Phases 3, 6, 7 will replace these)
    fn nlua_pop_String(lstate: *mut LuaState, arena: *mut Arena, err: *mut Error) -> NvimString;
    fn nlua_pop_Float(lstate: *mut LuaState, arena: *mut Arena, err: *mut Error) -> Float;
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
}

// lua_pop is a macro: #define lua_pop(L,n) lua_settop(L, -(n)-1)
#[inline]
unsafe fn lua_pop(lstate: *mut LuaState, n: c_int) {
    lua_settop(lstate, -n - 1);
}

// =============================================================================
// Phase 2: Trivial scalar poppers (real implementations)
// =============================================================================

/// Convert a Lua number to Integer.
///
/// Validates the top-of-stack value is a number and fits in Integer range.
/// Always pops one value from the stack.
#[unsafe(export_name = "nlua_pop_Integer")]
pub unsafe extern "C" fn rs_nlua_pop_Integer(
    lstate: *mut LuaState,
    _arena: *mut Arena,
    err: *mut Error,
) -> Integer {
    if lua_type(lstate, -1) != LUA_TNUMBER {
        lua_pop(lstate, 1);
        api_set_error(err, K_ERROR_VALIDATION, c"Expected Lua number".as_ptr());
        return 0;
    }
    let n = lua_tonumber(lstate, -1);
    lua_pop(lstate, 1);
    // i64::MAX as f64 rounds up, so compare with next-lower exact power of two
    #[allow(clippy::cast_precision_loss)]
    let in_range = n >= (API_INTEGER_MIN as f64) && n < 9_223_372_036_854_775_808.0_f64;
    let is_integral = n.fract() == 0.0;
    if !in_range || !is_integral {
        api_set_error(err, K_ERROR_EXCEPTION, c"Number is not integral".as_ptr());
        return 0;
    }
    #[allow(clippy::cast_possible_truncation)]
    {
        n as Integer
    }
}

/// Convert a Lua value to Boolean using Lua semantics.
///
/// Any value can be coerced to boolean (nil and false are falsy).
/// Always pops one value from the stack.
#[unsafe(export_name = "nlua_pop_Boolean")]
pub unsafe extern "C" fn rs_nlua_pop_Boolean(
    lstate: *mut LuaState,
    _arena: *mut Arena,
    _err: *mut Error,
) -> Boolean {
    let ret = lua_toboolean(lstate, -1) != 0;
    lua_pop(lstate, 1);
    ret
}

/// Convert a Lua value to Boolean with strict API validation.
///
/// Follows API conventions: only boolean, number, or nil are accepted.
/// Always pops one value from the stack.
#[unsafe(export_name = "nlua_pop_Boolean_strict")]
pub unsafe extern "C" fn rs_nlua_pop_Boolean_strict(
    lstate: *mut LuaState,
    err: *mut Error,
) -> Boolean {
    let ret = match lua_type(lstate, -1) {
        t if t == LUA_TBOOLEAN => lua_toboolean(lstate, -1) != 0,
        t if t == LUA_TNUMBER => lua_tonumber(lstate, -1) != 0.0,
        t if t == LUA_TNIL => false,
        _ => {
            api_set_error(err, K_ERROR_VALIDATION, c"not a boolean".as_ptr());
            false
        }
    };
    lua_pop(lstate, 1);
    ret
}

/// Pop a LuaRef from the Lua stack.
///
/// Creates a global reference to the top-of-stack value and pops it.
#[unsafe(export_name = "nlua_pop_LuaRef")]
pub unsafe extern "C" fn rs_nlua_pop_LuaRef(
    lstate: *mut LuaState,
    _arena: *mut Arena,
    _err: *mut Error,
) -> LuaRef {
    let rv = crate::refs::rs_nlua_ref_global(lstate, -1);
    lua_pop(lstate, 1);
    rv
}

/// Pop a handle (Buffer/Window/Tabpage integer) from the Lua stack.
///
/// Validates the top-of-stack value is a Lua number.
/// Always pops one value from the stack.
#[unsafe(export_name = "nlua_pop_handle")]
pub unsafe extern "C" fn rs_nlua_pop_handle(
    lstate: *mut LuaState,
    _arena: *mut Arena,
    err: *mut Error,
) -> c_int {
    let ret = if lua_type(lstate, -1) == LUA_TNUMBER {
        #[allow(clippy::cast_possible_truncation)]
        {
            lua_tonumber(lstate, -1) as c_int
        }
    } else {
        api_set_error(err, K_ERROR_VALIDATION, c"Expected Lua number".as_ptr());
        -1
    };
    lua_pop(lstate, 1);
    ret
}

// =============================================================================
// Still-forwarding stubs for Phases 3, 6, 7 (pop_String, pop_Float,
// pop_Array, pop_Dict, pop_Object, pop_typval). These will be replaced
// with real implementations in later phases.
// =============================================================================

/// Pop a String from the Lua stack. (Phase 3 will replace with real impl)
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_string(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> NvimString {
    nlua_pop_String(lstate, arena, err)
}

/// Pop a Float from the Lua stack. (Phase 6 will replace with real impl)
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_float(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> Float {
    nlua_pop_Float(lstate, arena, err)
}

/// Pop an Array from the Lua stack. (Phase 6 will replace with real impl)
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_array(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> Array {
    nlua_pop_Array(lstate, arena, err)
}

/// Pop a Dict from the Lua stack. (Phase 6 will replace with real impl)
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_dict(
    lstate: *mut LuaState,
    ref_: bool,
    arena: *mut Arena,
    err: *mut Error,
) -> Dict {
    nlua_pop_Dict(lstate, ref_, arena, err)
}

/// Pop an Object from the Lua stack. (Phase 7 will replace with real impl)
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_object(
    lstate: *mut LuaState,
    ref_: bool,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    nlua_pop_Object(lstate, ref_, arena, err)
}

/// Pop a typval from the Lua stack. (Phase 10 will replace with real impl)
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_typval(
    lstate: *mut LuaState,
    ret_tv: *mut std::ffi::c_void,
) -> bool {
    nlua_pop_typval(lstate, ret_tv)
}

// =============================================================================
// Direct Lua stack reading operations (kept for Rust consumers)
// =============================================================================

/// Get a number from the Lua stack without popping.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_tonumber(lstate: *mut LuaState, idx: c_int) -> f64 {
    lua_tonumber(lstate, idx)
}

/// Get a boolean from the Lua stack without popping.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_toboolean(lstate: *mut LuaState, idx: c_int) -> bool {
    lua_toboolean(lstate, idx) != 0
}

/// Get a string from the Lua stack without popping.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_tolstring(
    lstate: *mut LuaState,
    idx: c_int,
    len: *mut usize,
) -> *const c_char {
    lua_tolstring(lstate, idx, len)
}

/// Get the type of a Lua value on the stack.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_type(lstate: *mut LuaState, idx: c_int) -> c_int {
    lua_type(lstate, idx)
}

/// Pop values from the Lua stack.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_pop(lstate: *mut LuaState, n: c_int) {
    lua_pop(lstate, n);
}

/// Get the top of the Lua stack (number of elements).
#[no_mangle]
pub unsafe extern "C" fn rs_lua_gettop(lstate: *mut LuaState) -> c_int {
    lua_gettop(lstate)
}

// =============================================================================
// Safe Rust wrappers
// =============================================================================

/// Get the type of a Lua value on the stack.
#[inline]
#[must_use]
pub unsafe fn get_type(lstate: *mut LuaState, idx: c_int) -> crate::types::LuaType {
    crate::types::LuaType::from_raw(lua_type(lstate, idx)).unwrap_or(crate::types::LuaType::None)
}

/// Get the stack height.
#[inline]
#[must_use]
pub unsafe fn stack_height(lstate: *mut LuaState) -> c_int {
    lua_gettop(lstate)
}

/// Pop n values from the stack.
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
        assert_eq!(std::mem::size_of::<Arena>(), 0);
    }

    #[test]
    fn test_integer_bounds() {
        assert_eq!(API_INTEGER_MIN, i64::MIN);
    }
}
