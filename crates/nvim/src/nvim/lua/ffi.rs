//! Shared LuaJIT/luv/lpeg `extern "C"` declarations (phase 5b).
//!
//! One declaration per symbol, `use`d by every consumer, instead of
//! the per-module copies c2rust emitted. Everything here resolves
//! against the static LuaJIT/luv/lpeg libraries at link time.

#![deny(unsafe_op_in_unsafe_fn)]

// No forbid(unsafe_code): edition 2024 trips the unsafe_code lint on the
// extern block below, and declaring the foreign surface is this file's
// entire job.

use crate::src::nvim::types::*;

/// "Return every value the call produced", for `lua_pcall`/`lua_call`.
pub const LUA_MULTRET: ::core::ffi::c_int = -1;

/// The pseudo-index of the registry, where `luaL_ref` parks values.
pub const LUA_REGISTRYINDEX: ::core::ffi::c_int = -10000;
/// The pseudo-index of the running function's environment.
pub const LUA_ENVIRONINDEX: ::core::ffi::c_int = -10001;
/// The pseudo-index of the globals table.
pub const LUA_GLOBALSINDEX: ::core::ffi::c_int = -10002;

/// What `lua_type` answers for an index past the top of the stack.
pub const LUA_TNONE: ::core::ffi::c_int = -1;
pub const LUA_TNIL: ::core::ffi::c_int = 0;
pub const LUA_TBOOLEAN: ::core::ffi::c_int = 1;
pub const LUA_TLIGHTUSERDATA: ::core::ffi::c_int = 2;
pub const LUA_TNUMBER: ::core::ffi::c_int = 3;
pub const LUA_TSTRING: ::core::ffi::c_int = 4;
pub const LUA_TTABLE: ::core::ffi::c_int = 5;
pub const LUA_TFUNCTION: ::core::ffi::c_int = 6;
pub const LUA_TUSERDATA: ::core::ffi::c_int = 7;
pub const LUA_TTHREAD: ::core::ffi::c_int = 8;

/// The reference `luaL_ref` never hands out: "no value".
pub const LUA_NOREF: ::core::ffi::c_int = -2;
/// The reference `luaL_ref` hands out for `nil`.
pub const LUA_REFNIL: ::core::ffi::c_int = -1;

unsafe extern "C-unwind" {
    pub fn luaL_argerror(
        L: *mut lua_State,
        numarg: ::core::ffi::c_int,
        extramsg: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn luaL_buffinit(L: *mut lua_State, B: *mut luaL_Buffer);
    pub fn luaL_callmeta(
        L: *mut lua_State,
        obj: ::core::ffi::c_int,
        e: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn luaL_checkinteger(L: *mut lua_State, numArg: ::core::ffi::c_int) -> lua_Integer;
    pub fn luaL_checklstring(
        L: *mut lua_State,
        numArg: ::core::ffi::c_int,
        l: *mut size_t,
    ) -> *const ::core::ffi::c_char;
    pub fn luaL_checknumber(L: *mut lua_State, numArg: ::core::ffi::c_int) -> lua_Number;
    pub fn luaL_checkstack(
        L: *mut lua_State,
        sz: ::core::ffi::c_int,
        msg: *const ::core::ffi::c_char,
    );
    pub fn luaL_checktype(L: *mut lua_State, narg: ::core::ffi::c_int, t: ::core::ffi::c_int);
    pub fn luaL_checkudata(
        L: *mut lua_State,
        ud: ::core::ffi::c_int,
        tname: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_void;
    pub fn luaL_error(
        L: *mut lua_State,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn luaL_getmetafield(
        L: *mut lua_State,
        obj: ::core::ffi::c_int,
        e: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn luaL_loadbuffer(
        L: *mut lua_State,
        buff: *const ::core::ffi::c_char,
        sz: size_t,
        name: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn luaL_newmetatable(
        L: *mut lua_State,
        tname: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn luaL_newstate() -> *mut lua_State;
    pub fn luaL_openlibs(L: *mut lua_State);
    pub fn luaL_prepbuffer(B: *mut luaL_Buffer) -> *mut ::core::ffi::c_char;
    pub fn luaL_pushresult(B: *mut luaL_Buffer);
    pub fn luaL_ref(L: *mut lua_State, t: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn luaL_register(
        L: *mut lua_State,
        libname: *const ::core::ffi::c_char,
        l: *const luaL_Reg,
    );
    pub fn luaL_unref(L: *mut lua_State, t: ::core::ffi::c_int, ref_0: ::core::ffi::c_int);
    pub fn luaL_where(L: *mut lua_State, lvl: ::core::ffi::c_int);
    pub fn lua_call(L: *mut lua_State, nargs: ::core::ffi::c_int, nresults: ::core::ffi::c_int);
    pub fn lua_checkstack(L: *mut lua_State, sz: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn lua_close(L: *mut lua_State);
    pub fn lua_concat(L: *mut lua_State, n: ::core::ffi::c_int);
    pub fn lua_createtable(L: *mut lua_State, narr: ::core::ffi::c_int, nrec: ::core::ffi::c_int);
    pub fn lua_error(L: *mut lua_State) -> ::core::ffi::c_int;
    pub fn lua_getfenv(L: *mut lua_State, idx: ::core::ffi::c_int);
    pub fn lua_getfield(L: *mut lua_State, idx: ::core::ffi::c_int, k: *const ::core::ffi::c_char);
    pub fn lua_getmetatable(L: *mut lua_State, objindex: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn lua_gettable(L: *mut lua_State, idx: ::core::ffi::c_int);
    pub fn lua_gettop(L: *mut lua_State) -> ::core::ffi::c_int;
    pub fn lua_insert(L: *mut lua_State, idx: ::core::ffi::c_int);
    pub fn lua_iscfunction(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn lua_isnumber(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn lua_isstring(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn lua_isuserdata(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn lua_newuserdata(L: *mut lua_State, sz: size_t) -> *mut ::core::ffi::c_void;
    pub fn lua_next(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn lua_objlen(L: *mut lua_State, idx: ::core::ffi::c_int) -> size_t;
    pub fn lua_pcall(
        L: *mut lua_State,
        nargs: ::core::ffi::c_int,
        nresults: ::core::ffi::c_int,
        errfunc: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn lua_pushboolean(L: *mut lua_State, b: ::core::ffi::c_int);
    pub fn lua_pushcclosure(L: *mut lua_State, fn_0: lua_CFunction, n: ::core::ffi::c_int);
    pub fn lua_pushfstring(
        L: *mut lua_State,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> *const ::core::ffi::c_char;
    pub fn lua_pushinteger(L: *mut lua_State, n: lua_Integer);
    pub fn lua_pushlightuserdata(L: *mut lua_State, p: *mut ::core::ffi::c_void);
    pub fn lua_pushlstring(L: *mut lua_State, s: *const ::core::ffi::c_char, l: size_t);
    pub fn lua_pushnil(L: *mut lua_State);
    pub fn lua_pushnumber(L: *mut lua_State, n: lua_Number);
    pub fn lua_pushstring(L: *mut lua_State, s: *const ::core::ffi::c_char);
    pub fn lua_pushvalue(L: *mut lua_State, idx: ::core::ffi::c_int);
    pub fn lua_pushvfstring(
        L: *mut lua_State,
        fmt: *const ::core::ffi::c_char,
        argp: ::core::ffi::VaList,
    ) -> *const ::core::ffi::c_char;
    pub fn lua_rawequal(
        L: *mut lua_State,
        idx1: ::core::ffi::c_int,
        idx2: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn lua_rawget(L: *mut lua_State, idx: ::core::ffi::c_int);
    pub fn lua_rawgeti(L: *mut lua_State, idx: ::core::ffi::c_int, n: ::core::ffi::c_int);
    pub fn lua_rawset(L: *mut lua_State, idx: ::core::ffi::c_int);
    pub fn lua_rawseti(L: *mut lua_State, idx: ::core::ffi::c_int, n: ::core::ffi::c_int);
    pub fn lua_remove(L: *mut lua_State, idx: ::core::ffi::c_int);
    pub fn lua_replace(L: *mut lua_State, idx: ::core::ffi::c_int);
    pub fn lua_setfenv(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn lua_setfield(L: *mut lua_State, idx: ::core::ffi::c_int, k: *const ::core::ffi::c_char);
    pub fn lua_setmetatable(L: *mut lua_State, objindex: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn lua_settable(L: *mut lua_State, idx: ::core::ffi::c_int);
    pub fn lua_settop(L: *mut lua_State, idx: ::core::ffi::c_int);
    pub fn lua_toboolean(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn lua_tocfunction(L: *mut lua_State, idx: ::core::ffi::c_int) -> lua_CFunction;
    pub fn lua_tointeger(L: *mut lua_State, idx: ::core::ffi::c_int) -> lua_Integer;
    pub fn lua_tolstring(
        L: *mut lua_State,
        idx: ::core::ffi::c_int,
        len: *mut size_t,
    ) -> *const ::core::ffi::c_char;
    pub fn lua_tonumber(L: *mut lua_State, idx: ::core::ffi::c_int) -> lua_Number;
    pub fn lua_topointer(L: *mut lua_State, idx: ::core::ffi::c_int) -> *const ::core::ffi::c_void;
    pub fn lua_touserdata(L: *mut lua_State, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_void;
    pub fn lua_type(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn lua_typename(L: *mut lua_State, tp: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    pub fn luaopen_luv(L: *mut lua_State) -> ::core::ffi::c_int;
    pub fn luv_set_loop(L: *mut lua_State, loop_0: *mut uv_loop_t);
}

// -- The header's macros ----------------------------------------------------
//
// `lua.h` and `lauxlib.h` spell about a third of the C API as macros over the
// functions above, and c2rust expanded every use of them. They are written
// out here once instead: each is the macro's definition verbatim, so a body
// that reads like the C it was ported from means the same thing.

/// Drop the top `n` values.
///
/// # Safety
/// `L` must be a live Lua state with at least `n` values on its stack.
pub unsafe fn lua_pop(L: *mut lua_State, n: ::core::ffi::c_int) {
    unsafe { lua_settop(L, -n - 1) };
}

/// Push a fresh empty table.
///
/// # Safety
/// `L` must be a live Lua state with room for one more value.
pub unsafe fn lua_newtable(L: *mut lua_State) {
    unsafe { lua_createtable(L, 0, 0) };
}

/// Push `f` as a Lua function taking no upvalues.
///
/// # Safety
/// `L` must be a live Lua state with room for one more value.
pub unsafe fn lua_pushcfunction(
    L: *mut lua_State,
    f: unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
) {
    unsafe { lua_pushcclosure(L, Some(f), 0) };
}

/// Push the global named `k`.
///
/// # Safety
/// `L` must be a live Lua state and `k` a NUL-terminated name.
pub unsafe fn lua_getglobal(L: *mut lua_State, k: *const ::core::ffi::c_char) {
    unsafe { lua_getfield(L, LUA_GLOBALSINDEX, k) };
}

/// Store the value on top of the stack in the global named `k`.
///
/// # Safety
/// `L` must be a live Lua state with a value on top, `k` a NUL-terminated name.
pub unsafe fn lua_setglobal(L: *mut lua_State, k: *const ::core::ffi::c_char) {
    unsafe { lua_setfield(L, LUA_GLOBALSINDEX, k) };
}

/// The value at `idx` as a NUL-terminated string, converting a number in
/// place; null when it is neither.
///
/// # Safety
/// `L` must be a live Lua state and `idx` a valid index.
pub unsafe fn lua_tostring(
    L: *mut lua_State,
    idx: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    unsafe { lua_tolstring(L, idx, ::core::ptr::null_mut()) }
}

/// Is the value at `idx` nil?
///
/// # Safety
/// `L` must be a live Lua state and `idx` a valid index.
pub unsafe fn lua_isnil(L: *mut lua_State, idx: ::core::ffi::c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TNIL }
}

/// Is `idx` past the top of the stack, or nil?
///
/// # Safety
/// `L` must be a live Lua state.
pub unsafe fn lua_isnoneornil(L: *mut lua_State, idx: ::core::ffi::c_int) -> bool {
    unsafe { lua_type(L, idx) <= 0 }
}

/// Is the value at `idx` a table?
///
/// # Safety
/// `L` must be a live Lua state and `idx` a valid index.
pub unsafe fn lua_istable(L: *mut lua_State, idx: ::core::ffi::c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TTABLE }
}

/// Is the value at `idx` a function — Lua or C?
///
/// # Safety
/// `L` must be a live Lua state and `idx` a valid index.
pub unsafe fn lua_isfunction(L: *mut lua_State, idx: ::core::ffi::c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TFUNCTION }
}

/// Check that argument `narg` is a string and answer it, or throw.
///
/// # Safety
/// `L` must be a live Lua state; this longjmps when the argument is wrong.
pub unsafe fn luaL_checkstring(
    L: *mut lua_State,
    narg: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    unsafe { luaL_checklstring(L, narg, ::core::ptr::null_mut()) }
}

/// Throw `luaL_argerror` unless `cond` holds.
///
/// # Safety
/// `L` must be a live Lua state; this longjmps when `cond` is false.
pub unsafe fn luaL_argcheck(
    L: *mut lua_State,
    cond: bool,
    narg: ::core::ffi::c_int,
    extramsg: *const ::core::ffi::c_char,
) {
    if !cond {
        unsafe { luaL_argerror(L, narg, extramsg) };
    }
}

/// Where upvalue `i` of the running C function sits.
pub const fn lua_upvalueindex(i: ::core::ffi::c_int) -> ::core::ffi::c_int {
    LUA_GLOBALSINDEX - i
}
