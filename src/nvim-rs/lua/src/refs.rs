//! Lua reference management
//!
//! This module implements `nlua_ref`, `nlua_unref`, `nlua_pushref`,
//! `nlua_ref_global`, `nlua_unref_global`, `api_free_luaref`,
//! `api_new_luaref`, and `nlua_ref_is_function`.
//!
//! Phase A migration from executor.c.

use std::ffi::c_int;

use crate::state::LuaState;
use crate::stdlib::LUA_REGISTRYINDEX;
use crate::types::LUA_TFUNCTION;

// LuaRef type (matches C: typedef int LuaRef)
type LuaRef = c_int;

/// LUA_NOREF constant (-2)
const LUA_NOREF: LuaRef = -2;

// =============================================================================
// Opaque handle for nlua_ref_state_t
// =============================================================================

/// Opaque handle for `nlua_ref_state_t` in C.
///
/// Layout is NOT replicated here; all field access goes through C accessors.
#[repr(C)]
pub struct NluaRefState {
    _private: [u8; 0],
}

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Global Lua state
    fn get_global_lstate() -> *mut LuaState;

    // ref_state accessors (added in Phase A in executor.c)
    fn nvim_get_nlua_global_refs() -> *mut NluaRefState;
    fn nvim_nlua_ref_state_inc(rs: *mut NluaRefState);
    fn nvim_nlua_ref_state_dec(rs: *mut NluaRefState);
    fn nvim_nlua_ref_track(rs: *mut NluaRefState, ref_: c_int);
    fn nvim_nlua_ref_untrack(rs: *mut NluaRefState, ref_: c_int);

    // Lua C API
    fn lua_pushvalue(lstate: *mut LuaState, idx: c_int);
    fn lua_rawgeti(lstate: *mut LuaState, idx: c_int, n: c_int);
    fn lua_type(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_settop(lstate: *mut LuaState, idx: c_int);
    fn luaL_ref(lstate: *mut LuaState, t: c_int) -> c_int;
    fn luaL_unref(lstate: *mut LuaState, t: c_int, ref_: c_int);
}

/// lua_pop macro equivalent
#[inline]
unsafe fn lua_pop(lstate: *mut LuaState, n: c_int) {
    lua_settop(lstate, -n - 1);
}

// =============================================================================
// Rust implementations (Phase A)
// =============================================================================

/// Add a value to the Lua registry and return a reference to it.
///
/// The current implementation does not support calls from threads.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `ref_state` must be a valid pointer to `nlua_ref_state_t`.
/// - `index` must be a valid stack index.
#[unsafe(export_name = "nlua_ref")]
pub unsafe extern "C" fn rs_nlua_ref(
    lstate: *mut LuaState,
    ref_state: *mut NluaRefState,
    index: c_int,
) -> LuaRef {
    lua_pushvalue(lstate, index);
    let ref_ = luaL_ref(lstate, LUA_REGISTRYINDEX);
    if ref_ > 0 {
        nvim_nlua_ref_state_inc(ref_state);
        nvim_nlua_ref_track(ref_state, ref_);
    }
    ref_
}

/// Add a value to the Lua registry using the global ref state.
///
/// TODO(lewis6991): Currently cannot be run in __gc metamethods as they are
/// invoked in lua_close() which can be invoked after the ref_markers map is
/// destroyed in nlua_common_free_all_mem.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `index` must be a valid stack index.
#[unsafe(export_name = "nlua_ref_global")]
pub unsafe extern "C" fn rs_nlua_ref_global(lstate: *mut LuaState, index: c_int) -> LuaRef {
    rs_nlua_ref(lstate, nvim_get_nlua_global_refs(), index)
}

/// Remove a value from the Lua registry.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `ref_state` must be a valid pointer to `nlua_ref_state_t`.
#[unsafe(export_name = "nlua_unref")]
pub unsafe extern "C" fn rs_nlua_unref(
    lstate: *mut LuaState,
    ref_state: *mut NluaRefState,
    ref_: LuaRef,
) {
    if ref_ > 0 {
        nvim_nlua_ref_state_dec(ref_state);
        // NB: don't remove entry from map to track double-unref
        nvim_nlua_ref_untrack(ref_state, ref_);
        luaL_unref(lstate, LUA_REGISTRYINDEX, ref_);
    }
}

/// Remove a value from the Lua registry using the global ref state.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
#[unsafe(export_name = "nlua_unref_global")]
pub unsafe extern "C" fn rs_nlua_unref_global(lstate: *mut LuaState, ref_: LuaRef) {
    rs_nlua_unref(lstate, nvim_get_nlua_global_refs(), ref_);
}

/// Free a Lua API reference.
///
/// # Safety
///
/// `ref_` must be a valid `LuaRef` or `LUA_NOREF`.
#[unsafe(export_name = "api_free_luaref")]
pub unsafe extern "C" fn rs_api_free_luaref(ref_: LuaRef) {
    rs_nlua_unref_global(get_global_lstate(), ref_);
}

/// Push a value referenced in the Lua registry onto the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `ref_` must be a valid `LuaRef`.
#[unsafe(export_name = "nlua_pushref")]
pub unsafe extern "C" fn rs_nlua_pushref(lstate: *mut LuaState, ref_: LuaRef) {
    lua_rawgeti(lstate, LUA_REGISTRYINDEX, ref_);
}

/// Gets a new reference to an object stored at `original_ref`.
///
/// Does not copy the value; creates a new ref to the same Lua object.
/// Leaves the stack unchanged.
///
/// # Safety
///
/// `original_ref` must be a valid `LuaRef` or `LUA_NOREF`.
#[must_use]
#[unsafe(export_name = "api_new_luaref")]
pub unsafe extern "C" fn rs_api_new_luaref(original_ref: LuaRef) -> LuaRef {
    if original_ref == LUA_NOREF {
        return LUA_NOREF;
    }
    let lstate = get_global_lstate();
    rs_nlua_pushref(lstate, original_ref);
    let new_ref = rs_nlua_ref_global(lstate, -1);
    lua_pop(lstate, 1);
    new_ref
}

/// Check whether a `LuaRef` points to a Lua function.
///
/// # Safety
///
/// `ref_` must be a valid `LuaRef`.
#[must_use]
#[unsafe(export_name = "nlua_ref_is_function")]
pub unsafe extern "C" fn rs_nlua_ref_is_function(ref_: LuaRef) -> bool {
    let lstate = get_global_lstate();
    rs_nlua_pushref(lstate, ref_);
    // TODO(tjdevries): This should probably check for callable tables as well.
    let is_function = lua_type(lstate, -1) == LUA_TFUNCTION;
    lua_pop(lstate, 1);
    is_function
}
