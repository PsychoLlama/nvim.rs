//! Lua error helpers and tostring stubs
//!
//! Phase B migration from executor.c.
//! Migrates: nlua_get_error, nlua_error, nlua_nil_tostring, nlua_empty_dict_tostring.

use std::ffi::{c_char, c_int};

use crate::state::LuaState;
use crate::types::LUA_TFUNCTION;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Lua C API
    fn luaL_getmetafield(lstate: *mut LuaState, obj: c_int, e: *const c_char) -> c_int;
    fn luaL_callmeta(lstate: *mut LuaState, obj: c_int, e: *const c_char) -> c_int;
    fn lua_type(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_replace(lstate: *mut LuaState, idx: c_int);
    fn lua_settop(lstate: *mut LuaState, idx: c_int);
    fn lua_tolstring(lstate: *mut LuaState, idx: c_int, len: *mut usize) -> *const c_char;
    fn lua_pushstring(lstate: *mut LuaState, s: *const c_char) -> *const c_char;

    // Neovim error reporting shims (added in Phase B in executor.c)
    fn nvim_lua_in_script() -> bool;
    fn nvim_lua_semsg_multiline(fmt: *const c_char, len: c_int, str_: *const c_char);
    fn nvim_lua_fprintf_stderr(fmt: *const c_char, len: c_int, str_: *const c_char);
}

/// lua_isfunction macro: lua_type(L, n) == LUA_TFUNCTION
#[inline]
unsafe fn lua_isfunction(lstate: *mut LuaState, idx: c_int) -> bool {
    lua_type(lstate, idx) == LUA_TFUNCTION
}

/// lua_pop macro equivalent: lua_settop(L, -(n)-1)
#[inline]
unsafe fn lua_pop(lstate: *mut LuaState, n: c_int) {
    lua_settop(lstate, -n - 1);
}

// =============================================================================
// Rust implementations (Phase B)
// =============================================================================

/// Gets the Lua error at the top of the stack as a string.
///
/// Possibly modifies the error in-place via `__tostring` metamethod but does
/// not change the stack height.
///
/// The returned string points to memory on the Lua stack. Use or duplicate
/// it before calling any Lua API that could invalidate stack data.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `len` must be NULL or a valid pointer to a `usize`.
#[unsafe(export_name = "nlua_get_error")]
pub unsafe extern "C" fn rs_nlua_get_error(
    lstate: *mut LuaState,
    len: *mut usize,
) -> *const c_char {
    if luaL_getmetafield(lstate, -1, c"__tostring".as_ptr()) != 0 {
        if lua_isfunction(lstate, -1) && luaL_callmeta(lstate, -2, c"__tostring".as_ptr()) != 0 {
            // Call __tostring, convert the result and replace the error with it.
            lua_replace(lstate, -3);
        }
        // Pop __tostring.
        lua_pop(lstate, 1);
    }

    lua_tolstring(lstate, -1, len)
}

/// Converts a Lua error into a Vim error message.
///
/// The error at the top of the stack is consumed (popped).
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `msg` must be a valid C string containing exactly one `%.*s` format specifier.
#[unsafe(export_name = "nlua_error")]
pub unsafe extern "C" fn rs_nlua_error(lstate: *mut LuaState, msg: *const c_char) {
    let mut len: usize = 0;
    let str_ = rs_nlua_get_error(lstate, &raw mut len);
    // Cast mirrors the C: (int)len — error strings fit comfortably in i32.
    #[allow(clippy::cast_possible_truncation)]
    let len_int = len as c_int;

    if nvim_lua_in_script() {
        nvim_lua_fprintf_stderr(msg, len_int, str_);
    } else {
        nvim_lua_semsg_multiline(msg, len_int, str_);
    }

    lua_pop(lstate, 1);
}

/// `__tostring` metamethod for `vim.NIL`.
///
/// Pushes the string `"vim.NIL"` onto the Lua stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[unsafe(export_name = "nlua_nil_tostring")]
pub unsafe extern "C" fn rs_nlua_nil_tostring(lstate: *mut LuaState) -> c_int {
    lua_pushstring(lstate, c"vim.NIL".as_ptr());
    1
}

/// `__tostring` metamethod for `vim.empty_dict()`.
///
/// Pushes the string `"vim.empty_dict()"` onto the Lua stack.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[unsafe(export_name = "nlua_empty_dict_tostring")]
pub unsafe extern "C" fn rs_nlua_empty_dict_tostring(lstate: *mut LuaState) -> c_int {
    lua_pushstring(lstate, c"vim.empty_dict()".as_ptr());
    1
}
