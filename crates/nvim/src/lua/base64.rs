#![deny(unsafe_op_in_unsafe_fn)]

//! `vim.base64`: the Lua bindings over the codec in [`crate::base64`].
//!
//! # Boundary
//!
//! Both bindings are Lua C functions, so they keep the C-unwind ABI and
//! the raw `lua_State`; they share one shim that does all the stack work,
//! and the codec above it is safe and lives elsewhere.

use crate::base64::{decode, encode};
use crate::global_cell::SharedCell;
use crate::lua::ffi::{
    lua_createtable, lua_gettop, lua_pushlstring, lua_tolstring, lua_type, luaL_argerror,
    luaL_error, luaL_register,
};
use crate::types::{lua_State, luaL_Reg, size_t};
use core::ffi::{c_char, c_int};
use core::{ptr, slice};

pub const LUA_TSTRING: c_int = 4;

/// Body of both bindings: take the single string argument, run it through
/// `convert`, and push the result. `None` from `convert` is a Lua error.
///
/// # Safety
/// `lstate` must be a live Lua state, called as a Lua C function.
unsafe fn transcode(lstate: *mut lua_State, convert: fn(&[u8]) -> Option<Vec<u8>>) -> c_int {
    // SAFETY: the caller's live state; the argument checks come first, and
    // `luaL_argerror` does not return.
    let src = unsafe {
        if lua_gettop(lstate) < 1 {
            return luaL_error(lstate, c"Expected 1 argument".as_ptr());
        }
        if lua_type(lstate, 1) != LUA_TSTRING {
            luaL_argerror(lstate, 1, c"expected string".as_ptr());
        }
        let mut len: size_t = 0;
        let data = lua_tolstring(lstate, 1, &raw mut len);
        // The value stays on the stack, so the bytes outlive `convert`.
        slice::from_raw_parts(data.cast::<u8>(), len)
    };
    let Some(out) = convert(src) else {
        // SAFETY: as above.
        return unsafe { luaL_error(lstate, c"Invalid input".as_ptr()) };
    };
    // SAFETY: as above; Lua copies the bytes.
    unsafe { lua_pushlstring(lstate, out.as_ptr().cast::<c_char>(), out.len()) };
    1
}

/// `vim.base64.encode(str)`.
///
/// # Safety
/// Called by Lua with a live `lua_State`.
unsafe extern "C-unwind" fn nlua_base64_encode(lstate: *mut lua_State) -> c_int {
    // SAFETY: Lua calls this with a live state of its own.
    unsafe { transcode(lstate, |src| Some(encode(src).into_bytes())) }
}

/// `vim.base64.decode(str)`.
///
/// # Safety
/// Called by Lua with a live `lua_State`.
unsafe extern "C-unwind" fn nlua_base64_decode(lstate: *mut lua_State) -> c_int {
    // SAFETY: as above.
    unsafe { transcode(lstate, decode) }
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
    // SAFETY: the caller's live state; the table is what `luaL_register`
    // copies the (`'static`) registry into.
    unsafe {
        lua_createtable(lstate, 0, 0);
        luaL_register(lstate, ptr::null(), BASE64_FUNCTIONS.ptr().cast());
    }
    1
}
