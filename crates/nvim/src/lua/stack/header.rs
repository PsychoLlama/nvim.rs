//! The header's macros, for raw call sites not yet on [`Stack`](super::Stack).
//!
//! `lua.h` and `lauxlib.h` spell part of the C API as macros, which c2rust
//! expanded at every use. These are those macros written out once; each is
//! the macro's definition verbatim.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/stack/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_char, c_int};
use core::ptr;

use super::CFunction;
use crate::lua::ffi::{
    LUA_GLOBALSINDEX, LUA_TNIL, LUA_TTABLE, lua_createtable, lua_getfield, lua_pushcclosure,
    lua_setfield, lua_settop, lua_tolstring, lua_type, luaL_argerror, luaL_checklstring,
};
use crate::types::lua_State;

/// Drop the top `n` values.
///
/// # Safety
/// `L` must be a live Lua state with at least `n` values on its stack.
#[allow(non_snake_case)]
pub unsafe fn lua_pop(L: *mut lua_State, n: c_int) {
    unsafe { lua_settop(L, -n - 1) };
}

/// Push a fresh empty table.
///
/// # Safety
/// `L` must be a live Lua state with room for one more value.
#[allow(non_snake_case)]
pub unsafe fn lua_newtable(L: *mut lua_State) {
    unsafe { lua_createtable(L, 0, 0) };
}

/// Push `f` as a Lua function taking no upvalues.
///
/// # Safety
/// `L` must be a live Lua state with room for one more value.
#[allow(non_snake_case)]
pub unsafe fn lua_pushcfunction(L: *mut lua_State, f: CFunction) {
    unsafe { lua_pushcclosure(L, Some(f), 0) };
}

/// Push the global named `k`.
///
/// # Safety
/// `L` must be a live Lua state and `k` a NUL-terminated name.
#[allow(non_snake_case)]
pub unsafe fn lua_getglobal(L: *mut lua_State, k: *const c_char) {
    unsafe { lua_getfield(L, LUA_GLOBALSINDEX, k) };
}

/// Store the value on top of the stack in the global named `k`.
///
/// # Safety
/// `L` must be a live Lua state with a value on top, `k` a NUL-terminated name.
#[allow(non_snake_case)]
pub unsafe fn lua_setglobal(L: *mut lua_State, k: *const c_char) {
    unsafe { lua_setfield(L, LUA_GLOBALSINDEX, k) };
}

/// The value at `idx` as a NUL-terminated string, converting a number in
/// place; null when it is neither.
///
/// # Safety
/// `L` must be a live Lua state and `idx` a valid index.
#[allow(non_snake_case)]
pub unsafe fn lua_tostring(L: *mut lua_State, idx: c_int) -> *const c_char {
    unsafe { lua_tolstring(L, idx, ptr::null_mut()) }
}

/// Is the value at `idx` nil?
///
/// # Safety
/// `L` must be a live Lua state and `idx` a valid index.
#[allow(non_snake_case)]
pub unsafe fn lua_isnil(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TNIL }
}

/// Is `idx` past the top of the stack, or nil?
///
/// # Safety
/// `L` must be a live Lua state.
#[allow(non_snake_case)]
pub unsafe fn lua_isnoneornil(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) <= 0 }
}

/// Is the value at `idx` a table?
///
/// # Safety
/// `L` must be a live Lua state and `idx` a valid index.
#[allow(non_snake_case)]
pub unsafe fn lua_istable(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TTABLE }
}

/// Check that argument `narg` is a string and answer it, or throw.
///
/// # Safety
/// `L` must be a live Lua state; this longjmps when the argument is wrong.
#[allow(non_snake_case)]
pub unsafe fn luaL_checkstring(L: *mut lua_State, narg: c_int) -> *const c_char {
    unsafe { luaL_checklstring(L, narg, ptr::null_mut()) }
}

/// Throw `luaL_argerror` unless `cond` holds.
///
/// # Safety
/// `L` must be a live Lua state; this longjmps when `cond` is false.
#[allow(non_snake_case)]
pub unsafe fn luaL_argcheck(L: *mut lua_State, cond: bool, narg: c_int, extramsg: *const c_char) {
    if !cond {
        unsafe { luaL_argerror(L, narg, extramsg) };
    }
}

/// Where upvalue `i` of the running C function sits.
pub const fn lua_upvalueindex(i: c_int) -> c_int {
    LUA_GLOBALSINDEX - i
}
