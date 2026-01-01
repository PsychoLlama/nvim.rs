//! Lua executor FFI for Neovim
//!
//! This module provides Rust FFI for Lua callback invocation and executor state checking.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int};

// Re-export API types for convenience
pub use nvim_api::{Arena, Array, Error, LuaRef, NvimString, Object};

// =============================================================================
// LuaRetMode enum (matches C definition in executor.h)
// =============================================================================

/// Lua return value handling mode
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuaRetMode {
    /// Any object, but doesn't preserve nested luarefs
    Object = 0,
    /// NIL preserved as such, other values return their booleanness.
    /// Should also be used when return value is ignored, as it is allocation-free.
    NilBool = 1,
    /// Return value becomes a single `LuaRef`, regardless of type (except NIL)
    Luaref = 2,
    /// Like Object but return multiple return values as an Array
    Multi = 3,
}

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Existing accessor for in_fast_callback
    fn nvim_get_in_fast_callback() -> c_int;

    // Accessor for nlua_global_refs->ref_count
    fn nvim_get_nlua_global_ref_count() -> c_int;

    // Lua callback invocation (USE_RUST_LUA must be enabled in build)
    fn nvim_nlua_call_ref(
        ref_: LuaRef,
        name: *const c_char,
        args: Array,
        mode: c_int,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;

    fn nvim_nlua_call_ref_ctx(
        fast: c_int,
        ref_: LuaRef,
        name: *const c_char,
        args: Array,
        mode: c_int,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
}

// =============================================================================
// Safe Rust wrappers
// =============================================================================

/// Call a Lua callback reference.
///
/// # Arguments
/// * `ref_` - The `LuaRef` to call (not consumed)
/// * `name` - If non-NULL, function name for error messages
/// * `args` - Arguments to pass to the callback
/// * `mode` - How to handle the return value
/// * `arena` - Arena for return value allocation (can be NULL)
/// * `err` - Error output (can be NULL, errors are echoed)
///
/// # Safety
/// - `name` must be NULL or a valid C string
/// - `arena` must be NULL or a valid Arena pointer
/// - `err` must be NULL or a valid Error pointer
/// - `args` items must be valid Objects
#[inline]
pub unsafe fn lua_call_ref(
    ref_: LuaRef,
    name: *const c_char,
    args: Array,
    mode: LuaRetMode,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    nvim_nlua_call_ref(ref_, name, args, mode as c_int, arena, err)
}

/// Call a Lua callback in fast or normal context.
///
/// Use `fast=true` for decoration providers and UI callbacks where
/// the API is not safe to call directly.
///
/// # Safety
/// Same as `lua_call_ref`.
#[inline]
pub unsafe fn lua_call_ref_ctx(
    fast: bool,
    ref_: LuaRef,
    name: *const c_char,
    args: Array,
    mode: LuaRetMode,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    nvim_nlua_call_ref_ctx(
        c_int::from(fast),
        ref_,
        name,
        args,
        mode as c_int,
        arena,
        err,
    )
}

// =============================================================================
// Existing function (unchanged)
// =============================================================================

/// Check if the current execution context is safe for calling deferred API methods.
///
/// Luv callbacks are unsafe as they are called inside the uv loop.
/// Returns true if `in_fast_callback` == 0.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_is_deferred_safe() -> c_int {
    c_int::from(nvim_get_in_fast_callback() == 0)
}

/// Get the global Lua reference count.
///
/// Returns `nlua_global_refs->ref_count`.
///
/// # Safety
///
/// Calls external C function to access global state.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_get_global_ref_count() -> c_int {
    nvim_get_nlua_global_ref_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_ret_mode_values() {
        // Verify enum values match C definitions
        assert_eq!(LuaRetMode::Object as i32, 0);
        assert_eq!(LuaRetMode::NilBool as i32, 1);
        assert_eq!(LuaRetMode::Luaref as i32, 2);
        assert_eq!(LuaRetMode::Multi as i32, 3);
    }

    #[test]
    fn test_lua_ret_mode_sequential() {
        // Values should be sequential from 0 to 3
        let modes = [
            LuaRetMode::Object,
            LuaRetMode::NilBool,
            LuaRetMode::Luaref,
            LuaRetMode::Multi,
        ];
        for (i, mode) in modes.iter().enumerate() {
            assert_eq!(*mode as i32, i32::try_from(i).unwrap());
        }
    }

    #[test]
    fn test_lua_ret_mode_distinct() {
        // All modes should have distinct values
        let modes = [
            LuaRetMode::Object,
            LuaRetMode::NilBool,
            LuaRetMode::Luaref,
            LuaRetMode::Multi,
        ];
        for (i, &mode_a) in modes.iter().enumerate() {
            for (j, &mode_b) in modes.iter().enumerate() {
                if i != j {
                    assert_ne!(mode_a as i32, mode_b as i32);
                }
            }
        }
    }

    #[test]
    fn test_lua_ret_mode_size() {
        // Enum should be i32-sized due to #[repr(i32)]
        assert_eq!(
            std::mem::size_of::<LuaRetMode>(),
            std::mem::size_of::<i32>()
        );
    }
}
