//! Lua stdlib helpers
//!
//! This module provides FFI helpers for Lua standard library functions
//! and error handling utilities used by Neovim.

use std::ffi::{c_char, c_int};

use crate::state::LuaState;

// =============================================================================
// C FFI declarations for stdlib functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Error handling from executor.c
    fn nlua_error(lstate: *mut LuaState, msg: *const c_char);
    fn nlua_pcall(lstate: *mut LuaState, nargs: c_int, nresults: c_int) -> c_int;

    // Error formatting from stdlib.c
    fn nlua_push_errstr(lstate: *mut LuaState, fmt: *const c_char, ...);

    // luaL_* functions
    fn luaL_error(lstate: *mut LuaState, fmt: *const c_char, ...) -> c_int;
    fn luaL_where(lstate: *mut LuaState, lvl: c_int);
    fn luaL_checkinteger(lstate: *mut LuaState, arg: c_int) -> i64;
    fn luaL_checknumber(lstate: *mut LuaState, arg: c_int) -> f64;
    // Note: luaL_checkstring is a macro that calls luaL_checklstring with NULL len
    fn luaL_checklstring(lstate: *mut LuaState, arg: c_int, len: *mut usize) -> *const c_char;
    fn luaL_checktype(lstate: *mut LuaState, arg: c_int, t: c_int);
    fn luaL_optinteger(lstate: *mut LuaState, arg: c_int, def: i64) -> i64;
    fn luaL_ref(lstate: *mut LuaState, t: c_int) -> c_int;
    fn luaL_unref(lstate: *mut LuaState, t: c_int, ref_: c_int);

    // lua_* stack manipulation
    fn lua_settop(lstate: *mut LuaState, idx: c_int);
    fn lua_gettop(lstate: *mut LuaState) -> c_int;
    fn lua_pushvalue(lstate: *mut LuaState, idx: c_int);
    fn lua_remove(lstate: *mut LuaState, idx: c_int);
    fn lua_insert(lstate: *mut LuaState, idx: c_int);
    fn lua_replace(lstate: *mut LuaState, idx: c_int);
    fn lua_checkstack(lstate: *mut LuaState, extra: c_int) -> c_int;

    // lua_* table operations
    fn lua_createtable(lstate: *mut LuaState, narr: c_int, nrec: c_int);
    fn lua_rawget(lstate: *mut LuaState, idx: c_int);
    fn lua_rawset(lstate: *mut LuaState, idx: c_int);
    fn lua_rawgeti(lstate: *mut LuaState, idx: c_int, n: c_int);
    fn lua_rawseti(lstate: *mut LuaState, idx: c_int, n: c_int);
    fn lua_getfield(lstate: *mut LuaState, idx: c_int, k: *const c_char);
    fn lua_setfield(lstate: *mut LuaState, idx: c_int, k: *const c_char);
    // Note: lua_getglobal and lua_setglobal are macros, implemented below

    // Lua string formatting
    fn lua_pushfstring(lstate: *mut LuaState, fmt: *const c_char, ...) -> *const c_char;
    fn lua_concat(lstate: *mut LuaState, n: c_int);
}

/// Lua registry index constant
pub const LUA_REGISTRYINDEX: c_int = -10000;
/// Lua environment index constant
pub const LUA_ENVIRONINDEX: c_int = -10001;
/// Lua globals index constant
pub const LUA_GLOBALSINDEX: c_int = -10002;

/// LUA_MULTRET constant (-1)
pub const LUA_MULTRET: c_int = -1;

// =============================================================================
// Rust FFI exports
// =============================================================================

/// Report a Lua error to Neovim's message system.
///
/// This converts a Lua error to a Vim error message.
/// The error at the top of the Lua stack is consumed.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `msg` must be a valid C string with a %.*s format specifier.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_error(lstate: *mut LuaState, msg: *const c_char) {
    nlua_error(lstate, msg);
}

/// Protected call with debug traceback.
///
/// Like lua_pcall, but uses debug.traceback as the error function.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - The Lua stack must have the function and arguments set up correctly.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pcall(
    lstate: *mut LuaState,
    nargs: c_int,
    nresults: c_int,
) -> c_int {
    nlua_pcall(lstate, nargs, nresults)
}

/// Push error string with location information.
///
/// Like luaL_error but allows cleanup - doesn't longjmp immediately.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `fmt` must be a valid printf-style format string.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_push_errstr(lstate: *mut LuaState, fmt: *const c_char) {
    nlua_push_errstr(lstate, fmt);
}

/// Get the current stack top index.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_gettop_stdlib(lstate: *mut LuaState) -> c_int {
    lua_gettop(lstate)
}

/// Set the stack top, removing or adding nil values.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_settop(lstate: *mut LuaState, idx: c_int) {
    lua_settop(lstate, idx);
}

/// Push a copy of a value onto the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `idx` must be a valid stack index.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_pushvalue(lstate: *mut LuaState, idx: c_int) {
    lua_pushvalue(lstate, idx);
}

/// Remove the value at the given index.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `idx` must be a valid stack index.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_remove(lstate: *mut LuaState, idx: c_int) {
    lua_remove(lstate, idx);
}

/// Insert the top value at the given index.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `idx` must be a valid stack index.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_insert(lstate: *mut LuaState, idx: c_int) {
    lua_insert(lstate, idx);
}

/// Check integer argument.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - Will throw Lua error if argument is not a number.
#[no_mangle]
pub unsafe extern "C" fn rs_luaL_checkinteger(lstate: *mut LuaState, arg: c_int) -> i64 {
    luaL_checkinteger(lstate, arg)
}

/// Check number argument.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - Will throw Lua error if argument is not a number.
#[no_mangle]
pub unsafe extern "C" fn rs_luaL_checknumber(lstate: *mut LuaState, arg: c_int) -> f64 {
    luaL_checknumber(lstate, arg)
}

/// Check string argument.
///
/// This is the Rust implementation of the luaL_checkstring macro,
/// which calls luaL_checklstring with a NULL length pointer.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - Will throw Lua error if argument is not a string.
#[no_mangle]
pub unsafe extern "C" fn rs_luaL_checkstring(lstate: *mut LuaState, arg: c_int) -> *const c_char {
    // luaL_checkstring is a macro: #define luaL_checkstring(L,n) (luaL_checklstring(L, (n), NULL))
    luaL_checklstring(lstate, arg, std::ptr::null_mut())
}

/// Check string argument with length.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `len` must be a valid pointer or NULL.
/// - Will throw Lua error if argument is not a string.
#[no_mangle]
pub unsafe extern "C" fn rs_luaL_checklstring(
    lstate: *mut LuaState,
    arg: c_int,
    len: *mut usize,
) -> *const c_char {
    luaL_checklstring(lstate, arg, len)
}

/// Check argument type.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - Will throw Lua error if argument doesn't match type.
#[no_mangle]
pub unsafe extern "C" fn rs_luaL_checktype(lstate: *mut LuaState, arg: c_int, t: c_int) {
    luaL_checktype(lstate, arg, t);
}

/// Optional integer argument with default.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_luaL_optinteger(lstate: *mut LuaState, arg: c_int, def: i64) -> i64 {
    luaL_optinteger(lstate, arg, def)
}

/// Create a reference to the value at the top of stack.
///
/// Pops the value and returns a reference that can be used with luaL_unref.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_luaL_ref(lstate: *mut LuaState, t: c_int) -> c_int {
    luaL_ref(lstate, t)
}

/// Release a previously created reference.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_luaL_unref(lstate: *mut LuaState, t: c_int, ref_: c_int) {
    luaL_unref(lstate, t, ref_);
}

/// Create a new empty table.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_createtable(lstate: *mut LuaState, narr: c_int, nrec: c_int) {
    lua_createtable(lstate, narr, nrec);
}

/// Get a table field by name.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `k` must be a valid C string.
/// - `idx` must be a valid stack index.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_getfield(lstate: *mut LuaState, idx: c_int, k: *const c_char) {
    lua_getfield(lstate, idx, k);
}

/// Set a table field by name.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `k` must be a valid C string.
/// - `idx` must be a valid stack index.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_setfield(lstate: *mut LuaState, idx: c_int, k: *const c_char) {
    lua_setfield(lstate, idx, k);
}

/// Get a global variable.
///
/// This is the Rust implementation of the lua_getglobal macro,
/// which calls lua_getfield with LUA_GLOBALSINDEX.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `name` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_getglobal(lstate: *mut LuaState, name: *const c_char) {
    // lua_getglobal is a macro: #define lua_getglobal(L,s) lua_getfield(L, LUA_GLOBALSINDEX, (s))
    lua_getfield(lstate, LUA_GLOBALSINDEX, name);
}

/// Set a global variable.
///
/// This is the Rust implementation of the lua_setglobal macro,
/// which calls lua_setfield with LUA_GLOBALSINDEX.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `name` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_setglobal(lstate: *mut LuaState, name: *const c_char) {
    // lua_setglobal is a macro: #define lua_setglobal(L,s) lua_setfield(L, LUA_GLOBALSINDEX, (s))
    lua_setfield(lstate, LUA_GLOBALSINDEX, name);
}

/// Ensure enough space on the Lua stack.
///
/// Returns 1 if successful, 0 if it would overflow.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_checkstack(lstate: *mut LuaState, extra: c_int) -> c_int {
    lua_checkstack(lstate, extra)
}

/// Concatenate n values at the top of the stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_concat(lstate: *mut LuaState, n: c_int) {
    lua_concat(lstate, n);
}

/// Push location info (file:line:) onto the stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_luaL_where(lstate: *mut LuaState, lvl: c_int) {
    luaL_where(lstate, lvl);
}

// =============================================================================
// Safe Rust wrappers
// =============================================================================

/// Protected call with traceback error handler.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - The Lua stack must be set up correctly.
#[inline]
pub unsafe fn pcall(lstate: *mut LuaState, nargs: c_int, nresults: c_int) -> c_int {
    nlua_pcall(lstate, nargs, nresults)
}

/// Get the current stack size.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[inline]
#[must_use]
pub unsafe fn gettop(lstate: *mut LuaState) -> c_int {
    lua_gettop(lstate)
}

/// Set the stack top.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[inline]
pub unsafe fn settop(lstate: *mut LuaState, idx: c_int) {
    lua_settop(lstate, idx);
}

/// Create a new table with preallocated space.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[inline]
pub unsafe fn newtable(lstate: *mut LuaState) {
    lua_createtable(lstate, 0, 0);
}

/// Pop n values from the stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[inline]
pub unsafe fn pop_n(lstate: *mut LuaState, n: c_int) {
    lua_settop(lstate, -n - 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(LUA_REGISTRYINDEX, -10000);
        assert_eq!(LUA_ENVIRONINDEX, -10001);
        assert_eq!(LUA_GLOBALSINDEX, -10002);
        assert_eq!(LUA_MULTRET, -1);
    }
}
