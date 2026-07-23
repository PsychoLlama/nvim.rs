//! Shared LuaJIT/luv/lpeg `extern "C"` declarations (phase 5b).
//!
//! One declaration per symbol, `use`d by every consumer, instead of
//! the per-module copies c2rust emitted. Everything here resolves
//! against the static LuaJIT/luv/lpeg libraries at link time.

use crate::src::nvim::types::*;

extern "C-unwind" {
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
