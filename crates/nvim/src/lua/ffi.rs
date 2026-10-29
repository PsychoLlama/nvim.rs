//! Shared LuaJIT/luv/lpeg `extern "C"` declarations.
//!
//! One declaration per symbol, `use`d by every consumer, instead of
//! the per-module copies c2rust emitted. Everything here resolves
//! against the static LuaJIT/luv/lpeg libraries at link time.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md.
#![allow(unsafe_code)]
// LuaJIT's own symbol names: `L` for the state and the `luaL_*` functions.
// Renaming them would hide which C function each one is. The header's
// macros (`lua_pop`, `lua_upvalueindex`, ...) are `Stack` methods.
#![allow(non_snake_case)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// No forbid(unsafe_code): edition 2024 trips the unsafe_code lint on the
// extern block below, and declaring the foreign surface is this file's
// entire job.

use crate::types::*;

/// "Return every value the call produced", for `lua_pcall`/`lua_call`.
pub const LUA_MULTRET: ::core::ffi::c_int = -1;

/// The pseudo-index of the registry, where `luaL_ref` parks values.
pub const LUA_REGISTRYINDEX: ::core::ffi::c_int = -10000;
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
    pub fn luaL_checktype(L: *mut lua_State, narg: ::core::ffi::c_int, t: ::core::ffi::c_int);
    pub fn luaL_testudata(
        L: *mut lua_State,
        ud: ::core::ffi::c_int,
        tname: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_void;
    pub fn luaL_checkudata(
        L: *mut lua_State,
        ud: ::core::ffi::c_int,
        tname: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_void;
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
    pub fn luaL_ref(L: *mut lua_State, t: ::core::ffi::c_int) -> ::core::ffi::c_int;
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
    pub fn lua_pushinteger(L: *mut lua_State, n: lua_Integer);
    pub fn lua_pushlightuserdata(L: *mut lua_State, p: *mut ::core::ffi::c_void);
    pub fn lua_pushlstring(L: *mut lua_State, s: *const ::core::ffi::c_char, l: size_t);
    pub fn lua_pushnil(L: *mut lua_State);
    pub fn lua_pushnumber(L: *mut lua_State, n: lua_Number);
    pub fn lua_pushvalue(L: *mut lua_State, idx: ::core::ffi::c_int);
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
    pub fn lua_getstack(
        L: *mut lua_State,
        level: ::core::ffi::c_int,
        ar: *mut lua_Debug,
    ) -> ::core::ffi::c_int;
    pub fn lua_getinfo(
        L: *mut lua_State,
        what: *const ::core::ffi::c_char,
        ar: *mut lua_Debug,
    ) -> ::core::ffi::c_int;
}

/// What `lua_getinfo` fills in: `lua.h`'s `lua_Debug`, layout and all.
#[repr(C)]
pub struct lua_Debug {
    pub event: ::core::ffi::c_int,
    pub name: *const ::core::ffi::c_char,
    pub namewhat: *const ::core::ffi::c_char,
    pub what: *const ::core::ffi::c_char,
    pub source: *const ::core::ffi::c_char,
    pub currentline: ::core::ffi::c_int,
    pub nups: ::core::ffi::c_int,
    pub linedefined: ::core::ffi::c_int,
    pub lastlinedefined: ::core::ffi::c_int,
    /// `LUA_IDSIZE` bytes of the chunk name, for messages; nothing here
    /// reads it.
    pub short_src: [u8; 60],
    pub i_ci: ::core::ffi::c_int,
}

impl Default for lua_Debug {
    /// An all-zero record, which `lua_getstack`/`lua_getinfo` fill.
    fn default() -> Self {
        lua_Debug {
            event: 0,
            name: ::core::ptr::null(),
            namewhat: ::core::ptr::null(),
            what: ::core::ptr::null(),
            source: ::core::ptr::null(),
            currentline: 0,
            nups: 0,
            linedefined: 0,
            lastlinedefined: 0,
            short_src: [0; 60],
            i_ci: 0,
        }
    }
}
