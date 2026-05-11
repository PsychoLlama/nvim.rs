//! Small leaf functions from executor.c
//!
//! Phase D migration from executor.c.
//! Migrates: nlua_get_nil_ref, nlua_get_empty_dict_ref, nlua_in_fast_event,
//!           nlua_is_thread, nlua_is_table_from_lua, nlua_register_table_as_callable,
//!           nlua_init_defaults, nlua_init_argv, nlua_module_preloader.

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::state::LuaState;
use crate::stdlib::{LUA_GLOBALSINDEX, LUA_REGISTRYINDEX};
use crate::types::LUA_TFUNCTION;

// LUA_NOREF constant (-2)
const LUA_NOREF: c_int = -2;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Global Lua state
    fn get_global_lstate() -> *mut LuaState;

    // Lua C API
    fn lua_settop(lstate: *mut LuaState, idx: c_int);
    fn lua_type(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_pushboolean(lstate: *mut LuaState, b: c_int);
    fn lua_getfield(lstate: *mut LuaState, idx: c_int, k: *const c_char);
    fn lua_setfield(lstate: *mut LuaState, idx: c_int, k: *const c_char);
    fn lua_getmetatable(lstate: *mut LuaState, objindex: c_int) -> c_int;
    fn lua_rawseti(lstate: *mut LuaState, idx: c_int, n: c_int);
    fn lua_pushstring(lstate: *mut LuaState, s: *const c_char) -> *const c_char;
    fn lua_tointeger(lstate: *mut LuaState, idx: c_int) -> i64;
    fn lua_call(lstate: *mut LuaState, nargs: c_int, nresults: c_int);
    fn lua_createtable(lstate: *mut LuaState, narr: c_int, nrec: c_int);
    fn lua_tolstring(lstate: *mut LuaState, idx: c_int, len: *mut usize) -> *const c_char;
    fn lua_error(lstate: *mut LuaState) -> c_int;
    fn luaL_loadbuffer(
        lstate: *mut LuaState,
        buf: *const c_char,
        sz: usize,
        name: *const c_char,
    ) -> c_int;

    // Phase C function (exec.rs - stdlib wraps nlua_pcall)
    fn nlua_pcall(lstate: *mut LuaState, nargs: c_int, nresults: c_int) -> c_int;

    // Phase D C accessors (added to executor.c)
    fn nvim_get_in_fast_callback() -> c_int;
    fn nvim_lua_get_ref_state(lstate: *mut LuaState) -> *mut std::ffi::c_void;
    fn nvim_ref_state_get_nil_ref(rs: *mut std::ffi::c_void) -> c_int;
    fn nvim_ref_state_get_empty_dict_ref(rs: *mut std::ffi::c_void) -> c_int;
    fn nvim_typval_get_lua_table_ref(arg: *const std::ffi::c_void) -> c_int;
    fn nvim_lua_get_builtin_module(
        i: usize,
        name_out: *mut *const c_char,
        data_out: *mut *const u8,
        size_out: *mut usize,
    ) -> bool;
    fn nvim_lua_stderr_str(str_: *const c_char);

    // register_luafunc is in userfunc Rust crate, exported as C symbol
    fn register_luafunc(func: c_int) -> *mut c_char;

    // String helpers
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;
    fn strchrsub(str_: *mut c_char, c: c_char, x: c_char);
}

/// lua_pop: lua_settop(L, -(n)-1)
#[inline]
unsafe fn lua_pop(lstate: *mut LuaState, n: c_int) {
    lua_settop(lstate, -n - 1);
}

/// lua_newtable: lua_createtable(L, 0, 0)
#[inline]
unsafe fn lua_newtable(lstate: *mut LuaState) {
    lua_createtable(lstate, 0, 0);
}

/// lua_getglobal: lua_getfield(L, LUA_GLOBALSINDEX, name)
#[inline]
unsafe fn lua_getglobal(lstate: *mut LuaState, name: *const c_char) {
    lua_getfield(lstate, LUA_GLOBALSINDEX, name);
}

/// lua_setglobal: lua_setfield(L, LUA_GLOBALSINDEX, name)
#[inline]
unsafe fn lua_setglobal(lstate: *mut LuaState, name: *const c_char) {
    lua_setfield(lstate, LUA_GLOBALSINDEX, name);
}

/// lua_isfunction: lua_type(L, n) == LUA_TFUNCTION
#[inline]
unsafe fn lua_isfunction(lstate: *mut LuaState, n: c_int) -> bool {
    lua_type(lstate, n) == LUA_TFUNCTION
}

/// lua_upvalueindex: LUA_GLOBALSINDEX - i
#[inline]
#[must_use]
const fn lua_upvalueindex(i: c_int) -> c_int {
    LUA_GLOBALSINDEX - i
}

// =============================================================================
// Rust implementations (Phase D)
// =============================================================================

/// Get the nil ref from the thread-local ref state.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[unsafe(export_name = "nlua_get_nil_ref")]
pub unsafe extern "C" fn rs_nlua_get_nil_ref(lstate: *mut LuaState) -> c_int {
    let rs = nvim_lua_get_ref_state(lstate);
    nvim_ref_state_get_nil_ref(rs)
}

/// Get the empty dict ref from the thread-local ref state.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[unsafe(export_name = "nlua_get_empty_dict_ref")]
pub unsafe extern "C" fn rs_nlua_get_empty_dict_ref(lstate: *mut LuaState) -> c_int {
    let rs = nvim_lua_get_ref_state(lstate);
    nvim_ref_state_get_empty_dict_ref(rs)
}

/// Lua callback: push `in_fast_callback > 0` boolean.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[unsafe(export_name = "nlua_in_fast_event")]
pub unsafe extern "C" fn rs_nlua_in_fast_event(lstate: *mut LuaState) -> c_int {
    lua_pushboolean(lstate, c_int::from(nvim_get_in_fast_callback() > 0));
    1
}

/// Lua callback: push the `nvim.thread` registry field.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[unsafe(export_name = "nlua_is_thread")]
pub unsafe extern "C" fn rs_nlua_is_thread(lstate: *mut LuaState) -> c_int {
    lua_getfield(lstate, LUA_REGISTRYINDEX, c"nvim.thread".as_ptr());
    1
}

/// Return true if `arg` is a dict or list that originated from a Lua table.
///
/// # Safety
///
/// `arg` must be a valid `typval_T *`.
#[must_use]
#[unsafe(export_name = "nlua_is_table_from_lua")]
pub unsafe extern "C" fn rs_nlua_is_table_from_lua(arg: *const std::ffi::c_void) -> bool {
    nvim_typval_get_lua_table_ref(arg) != LUA_NOREF
}

/// Register a Lua table that has a `__call` metamethod as a callable.
///
/// Returns a heap-allocated name string on success, NULL otherwise.
///
/// # Safety
///
/// `arg` must be a valid `typval_T *`.
#[must_use]
#[unsafe(export_name = "nlua_register_table_as_callable")]
pub unsafe extern "C" fn rs_nlua_register_table_as_callable(
    arg: *const std::ffi::c_void,
) -> *mut c_char {
    let table_ref = nvim_typval_get_lua_table_ref(arg);
    if table_ref == LUA_NOREF {
        return ptr::null_mut();
    }

    let lstate = get_global_lstate();

    crate::refs::rs_nlua_pushref(lstate, table_ref); // [table]
    if lua_getmetatable(lstate, -1) == 0 {
        lua_pop(lstate, 1);
        return ptr::null_mut();
    } // [table, mt]

    lua_getfield(lstate, -1, c"__call".as_ptr()); // [table, mt, mt.__call]
    if !lua_isfunction(lstate, -1) {
        lua_pop(lstate, 3);
        return ptr::null_mut();
    }
    lua_pop(lstate, 2); // [table]

    let func = crate::refs::rs_nlua_ref_global(lstate, -1);

    let name = register_luafunc(func);

    lua_pop(lstate, 1); // []

    name
}

/// Execute the `vim._defaults` module to set up default mappings and autocommands.
///
/// # Safety
///
/// This function accesses the global Lua state and must be called from the main thread.
#[unsafe(export_name = "nlua_init_defaults")]
pub unsafe extern "C" fn rs_nlua_init_defaults() {
    let lstate = get_global_lstate();
    assert!(!lstate.is_null());

    lua_getglobal(lstate, c"require".as_ptr());
    lua_pushstring(lstate, c"vim._defaults".as_ptr());
    if nlua_pcall(lstate, 1, 0) != 0 {
        let s = lua_tolstring(lstate, -1, ptr::null_mut());
        nvim_lua_stderr_str(s);
    }
}

/// Build `_G.arg` table from argv and set it as the global.
///
/// Returns the number of script arguments pushed.
///
/// # Safety
///
/// `lstate` must be valid. `argv` must be a valid C string array of length >= `argc`.
#[unsafe(export_name = "nlua_init_argv")]
pub unsafe extern "C" fn rs_nlua_init_argv(
    lstate: *mut LuaState,
    argv: *const *const c_char,
    arg_count: c_int,
    lua_arg0: c_int,
) -> c_int {
    let mut i: c_int = 0;
    lua_newtable(lstate); // _G.arg

    if lua_arg0 > 0 {
        lua_pushstring(lstate, *argv.offset((lua_arg0 - 1) as isize));
        lua_rawseti(lstate, -2, 0); // _G.arg[0] = "foo.lua"

        while i + lua_arg0 < arg_count {
            lua_pushstring(lstate, *argv.offset((i + lua_arg0) as isize));
            lua_rawseti(lstate, -2, i + 1);
            i += 1;
        }
    }

    lua_setglobal(lstate, c"arg".as_ptr());
    i
}

/// Lua callback: load and execute a builtin Lua module by upvalue index.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[unsafe(export_name = "nlua_module_preloader")]
pub unsafe extern "C" fn rs_nlua_module_preloader(lstate: *mut LuaState) -> c_int {
    let i = lua_tointeger(lstate, lua_upvalueindex(1));
    let idx = usize::try_from(i).unwrap_or(usize::MAX);

    let mut name_ptr: *const c_char = ptr::null();
    let mut data_ptr: *const u8 = ptr::null();
    let mut size: usize = 0;

    if !nvim_lua_get_builtin_module(idx, &raw mut name_ptr, &raw mut data_ptr, &raw mut size) {
        lua_pushstring(lstate, c"invalid builtin module index".as_ptr());
        return lua_error(lstate);
    }

    // Build chunk name: "@" + name.replace('.', '/') + ".lua"
    let mut namebuf = [0u8; 256];
    namebuf[0] = b'@';
    let off = xstrlcpy(
        namebuf.as_mut_ptr().add(1).cast::<c_char>(),
        name_ptr,
        namebuf.len() - 2,
    );
    strchrsub(
        namebuf.as_mut_ptr().add(1).cast::<c_char>(),
        b'.' as c_char,
        b'/' as c_char,
    );
    xstrlcpy(
        namebuf.as_mut_ptr().add(1 + off).cast::<c_char>(),
        c".lua".as_ptr(),
        namebuf.len() - 2 - off,
    );

    if luaL_loadbuffer(
        lstate,
        data_ptr.cast::<c_char>(),
        size - 1,
        namebuf.as_ptr().cast::<c_char>(),
    ) != 0
    {
        return lua_error(lstate);
    }

    lua_call(lstate, 0, 1); // propagates error to caller
    1
}
