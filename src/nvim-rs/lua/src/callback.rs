//! Callback infrastructure for Lua
//!
//! This module provides FFI helpers for managing Lua references
//! and callback invocation.

use std::ffi::{c_char, c_int, c_void};

use crate::state::LuaState;
use nvim_api::{Arena, Array, Error, LuaRef, Object};

// =============================================================================
// nlua_ref_state_t (opaque handle)
// =============================================================================

/// Opaque handle to nlua_ref_state_t
#[repr(C)]
pub struct NluaRefState {
    _private: [u8; 0],
}

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Reference management
    fn nlua_ref(lstate: *mut LuaState, ref_state: *mut NluaRefState, index: c_int) -> LuaRef;
    fn nlua_ref_global(lstate: *mut LuaState, index: c_int) -> LuaRef;
    fn nlua_unref(lstate: *mut LuaState, ref_state: *mut NluaRefState, ref_: LuaRef);
    fn nlua_unref_global(lstate: *mut LuaState, ref_: LuaRef);
    fn api_free_luaref(ref_: LuaRef);
    fn api_new_luaref(original_ref: LuaRef) -> LuaRef;

    // Reference push
    fn nlua_pushref(lstate: *mut LuaState, ref_: LuaRef);

    // Special references
    fn nlua_get_nil_ref(lstate: *mut LuaState) -> LuaRef;
    fn nlua_get_empty_dict_ref(lstate: *mut LuaState) -> LuaRef;

    // Reference checking
    fn nlua_ref_is_function(ref_: LuaRef) -> bool;

    // Callback invocation (already in lib.rs but we expose them here too)
    fn nlua_call_ref(
        ref_: LuaRef,
        name: *const c_char,
        args: Array,
        mode: c_int,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;

    fn nlua_call_ref_ctx(
        fast: bool,
        ref_: LuaRef,
        name: *const c_char,
        args: Array,
        mode: c_int,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;

    // Typval callback
    fn typval_exec_lua_callable(
        lua_cb: LuaRef,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: *mut c_void,
    ) -> c_int;
}

// =============================================================================
// Rust FFI exports - Reference Management
// =============================================================================

/// Create a reference to a Lua value.
///
/// Creates a reference to the value at the given stack index in the given
/// reference state. The value is not popped from the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `ref_state` must be a valid reference state pointer.
/// - `index` must be a valid stack index.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_ref(
    lstate: *mut LuaState,
    ref_state: *mut NluaRefState,
    index: c_int,
) -> LuaRef {
    nlua_ref(lstate, ref_state, index)
}

/// Create a reference to a Lua value in the global reference state.
///
/// Creates a reference to the value at the given stack index.
/// The value is not popped from the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `index` must be a valid stack index.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_ref_global(lstate: *mut LuaState, index: c_int) -> LuaRef {
    nlua_ref_global(lstate, index)
}

/// Release a Lua reference.
///
/// Releases the reference in the given reference state.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `ref_state` must be a valid reference state pointer.
/// - `ref_` must be a valid reference or LUA_NOREF.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_unref(
    lstate: *mut LuaState,
    ref_state: *mut NluaRefState,
    ref_: LuaRef,
) {
    nlua_unref(lstate, ref_state, ref_);
}

/// Release a Lua reference from the global reference state.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `ref_` must be a valid reference or LUA_NOREF.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_unref_global(lstate: *mut LuaState, ref_: LuaRef) {
    nlua_unref_global(lstate, ref_);
}

/// Free a Lua reference without a Lua state.
///
/// Uses the global Lua state to free the reference.
///
/// # Safety
///
/// `ref_` must be a valid reference or LUA_NOREF.
#[no_mangle]
pub unsafe extern "C" fn rs_api_free_luaref(ref_: LuaRef) {
    api_free_luaref(ref_);
}

/// Create a new reference to the same Lua value.
///
/// Creates a new reference that points to the same Lua value as the original.
/// This is used when an Object containing a LuaRef is copied.
///
/// # Safety
///
/// `original_ref` must be a valid reference.
#[no_mangle]
pub unsafe extern "C" fn rs_api_new_luaref(original_ref: LuaRef) -> LuaRef {
    api_new_luaref(original_ref)
}

/// Push a referenced Lua value onto the stack.
///
/// Pushes the Lua value referenced by `ref_` onto the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `ref_` must be a valid reference.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pushref(lstate: *mut LuaState, ref_: LuaRef) {
    nlua_pushref(lstate, ref_);
}

// =============================================================================
// Rust FFI exports - Special References
// =============================================================================

/// Get the nil reference.
///
/// Returns a reference to the special nil value used for vim.NIL.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_get_nil_ref(lstate: *mut LuaState) -> LuaRef {
    nlua_get_nil_ref(lstate)
}

/// Get the empty dict reference.
///
/// Returns a reference to the special empty dict value used for vim.empty_dict().
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_get_empty_dict_ref(lstate: *mut LuaState) -> LuaRef {
    nlua_get_empty_dict_ref(lstate)
}

// =============================================================================
// Rust FFI exports - Reference Checking
// =============================================================================

/// Check if a reference is a function.
///
/// Returns true if the referenced value is a Lua function.
///
/// # Safety
///
/// `ref_` must be a valid reference.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_ref_is_function(ref_: LuaRef) -> bool {
    nlua_ref_is_function(ref_)
}

// =============================================================================
// Rust FFI exports - Callback Invocation
// =============================================================================

/// Call a Lua callback reference.
///
/// Calls the function referenced by `ref_` with the given arguments.
///
/// # Arguments
/// * `ref_` - The LuaRef to call (not consumed)
/// * `name` - If non-NULL, function name for error messages
/// * `args` - Arguments to pass to the callback
/// * `mode` - How to handle the return value (LuaRetMode)
/// * `arena` - Arena for return value allocation (can be NULL)
/// * `err` - Error output (can be NULL, errors are echoed)
///
/// # Safety
///
/// - `ref_` must be a valid function reference.
/// - `name` must be NULL or a valid C string.
/// - `args` items must be valid Objects.
/// - `arena` must be NULL or a valid Arena pointer.
/// - `err` must be NULL or a valid Error pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_call_ref(
    ref_: LuaRef,
    name: *const c_char,
    args: Array,
    mode: c_int,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    nlua_call_ref(ref_, name, args, mode, arena, err)
}

/// Call a Lua callback in fast or normal context.
///
/// Use `fast=true` for decoration providers and UI callbacks where
/// the API is not safe to call directly.
///
/// # Safety
///
/// Same as `rs_nlua_call_ref`.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_call_ref_ctx(
    fast: bool,
    ref_: LuaRef,
    name: *const c_char,
    args: Array,
    mode: c_int,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    nlua_call_ref_ctx(fast, ref_, name, args, mode, arena, err)
}

/// Execute a Lua callable from typval context.
///
/// Calls a Lua function with typval arguments and returns the result
/// in a typval.
///
/// # Safety
///
/// - `lua_cb` must be a valid function reference.
/// - `argvars` must point to `argcount` valid typval_T values.
/// - `rettv` must be a valid typval_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_typval_exec_lua_callable(
    lua_cb: LuaRef,
    argcount: c_int,
    argvars: *mut c_void,
    rettv: *mut c_void,
) -> c_int {
    typval_exec_lua_callable(lua_cb, argcount, argvars, rettv)
}

// =============================================================================
// Safe Rust wrappers
// =============================================================================

/// Create a global reference to a Lua value.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `index` must be a valid stack index.
#[inline]
pub unsafe fn ref_global(lstate: *mut LuaState, index: c_int) -> LuaRef {
    nlua_ref_global(lstate, index)
}

/// Release a global reference.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `ref_` must be a valid reference or LUA_NOREF.
#[inline]
pub unsafe fn unref_global(lstate: *mut LuaState, ref_: LuaRef) {
    nlua_unref_global(lstate, ref_);
}

/// Push a referenced value onto the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `ref_` must be a valid reference.
#[inline]
pub unsafe fn pushref(lstate: *mut LuaState, ref_: LuaRef) {
    nlua_pushref(lstate, ref_);
}

/// Check if a reference is a function.
///
/// # Safety
///
/// `ref_` must be a valid reference.
#[inline]
#[must_use]
pub unsafe fn is_function(ref_: LuaRef) -> bool {
    nlua_ref_is_function(ref_)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nlua_ref_state_is_opaque() {
        assert_eq!(std::mem::size_of::<NluaRefState>(), 0);
    }
}
