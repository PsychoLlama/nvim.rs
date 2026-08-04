//! `vim.base64`: the Lua bindings over the codec in [`crate::src::nvim::base64`].
//!
//! # Boundary
//!
//! Both bindings are Lua C functions, so they keep the C-unwind ABI and
//! the raw `lua_State`; they share one shim that does all the stack work,
//! and the codec above it is safe and lives elsewhere.

use crate::src::nvim::base64::{decode, encode};
use crate::src::nvim::global_cell::SharedCell;
use crate::src::nvim::lua::ffi::{
    lua_createtable, lua_gettop, lua_pushlstring, lua_tolstring, lua_type, luaL_argerror,
    luaL_error, luaL_register,
};
use crate::src::nvim::types::{lua_State, luaL_Reg, size_t};
use core::ffi::{c_char, c_int};
use core::{ptr, slice};

pub const LUA_TSTRING: c_int = 4;

/// Body of both bindings: take the single string argument, run it through
/// `convert`, and push the result. `None` from `convert` is a Lua error.
///
/// # Safety
/// `lstate` must be a live Lua state, called as a Lua C function.
unsafe fn transcode(lstate: *mut lua_State, convert: fn(&[u8]) -> Option<Vec<u8>>) -> c_int {
    if lua_gettop(lstate) < 1 {
        return luaL_error(lstate, c"Expected 1 argument".as_ptr());
    }
    if lua_type(lstate, 1) != LUA_TSTRING {
        // Does not return.
        luaL_argerror(lstate, 1, c"expected string".as_ptr());
    }
    let mut len: size_t = 0;
    let src = lua_tolstring(lstate, 1, &raw mut len);
    let Some(out) = convert(slice::from_raw_parts(src as *const u8, len)) else {
        return luaL_error(lstate, c"Invalid input".as_ptr());
    };
    lua_pushlstring(lstate, out.as_ptr() as *const c_char, out.len());
    1
}

unsafe extern "C-unwind" fn nlua_base64_encode(lstate: *mut lua_State) -> c_int {
    transcode(lstate, |src| Some(encode(src).into_bytes()))
}

unsafe extern "C-unwind" fn nlua_base64_decode(lstate: *mut lua_State) -> c_int {
    transcode(lstate, decode)
}

/// What `luaL_register` copies into the table, terminated by a null name.
/// The raw name pointers are what keep it out of a plain `static`.
static BASE64_FUNCTIONS: SharedCell<[luaL_Reg; 3]> = SharedCell::new([
    luaL_Reg {
        name: c"encode".as_ptr(),
        func: Some(nlua_base64_encode),
    },
    luaL_Reg {
        name: c"decode".as_ptr(),
        func: Some(nlua_base64_decode),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
]);

/// Build the `vim.base64` table and leave it on the stack.
///
/// # Safety
/// `lstate` must be a live Lua state with room for one more value.
pub unsafe fn luaopen_base64(lstate: *mut lua_State) -> c_int {
    lua_createtable(lstate, 0, 0);
    luaL_register(lstate, ptr::null(), BASE64_FUNCTIONS.ptr().cast());
    1
}
