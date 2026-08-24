//! Installing all of the above onto a `lua_State`.
//!
//! [`nlua_state_add_stdlib`] is the one registration point: it pushes every
//! `vim.*` C function this module implements onto the `vim` table, and
//! `nlua_state_add_internal` the `vim._*` half that only the main state gets.
//! [`nlua_push_errstr`] is the shared error formatter.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::{
    REGEX_META, nlua_getvar, nlua_iconv, nlua_regex, nlua_setvar, nlua_str_byteindex,
    nlua_str_utf_end, nlua_str_utf_pos, nlua_str_utf_start, nlua_str_utfindex, nlua_stricmp,
    nlua_with,
};
use crate::api::private::helpers::handle_get_window;
use crate::cjson::lua_cjson::lua_cjson_new;
use crate::fold::fold_update;
use crate::lua::base64::luaopen_base64;
use crate::lua::ffi::{
    lua_concat, lua_getfield, lua_getglobal, lua_pop, lua_pushcfunction, lua_pushvalue,
    lua_pushvfstring, lua_setfield, luaL_checkinteger, luaL_error, luaL_newmetatable,
    luaL_register, luaL_where,
};
use crate::lua::spell::luaopen_spell;
use crate::lua::xdiff::nlua_xdl_diff;
use crate::mpack::lmpack::luaopen_mpack;
use crate::types::{handle_T, linenr_T, lua_State};

unsafe extern "C-unwind" {
    /// lpeg's own `luaopen_*`, linked in from the vendored library.
    fn luaopen_lpeg(lstate: *mut lua_State) -> c_int;
}

/// `vim._foldupdate(win, top, bot)`: recompute fold levels (which can mean
/// evaluating 'foldexpr') over a zero-based end-exclusive line range, without
/// any of `zx`'s other side effects.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
unsafe extern "C-unwind" fn nlua_foldupdate(lstate: *mut lua_State) -> c_int {
    unsafe {
        let window = luaL_checkinteger(lstate, 1) as handle_T;
        let win = handle_get_window(window);
        if win.is_null() {
            return luaL_error(lstate, c"invalid window".as_ptr());
        }
        let top = luaL_checkinteger(lstate, 2) as linenr_T + 1;
        if top < 1 {
            return luaL_error(lstate, c"invalid top".as_ptr());
        }
        let bot = luaL_checkinteger(lstate, 3) as linenr_T;
        if top > bot {
            return luaL_error(lstate, c"invalid bot".as_ptr());
        }

        fold_update(win, top, bot);
        0
    }
}

/// Store `f` under `name` in the table just below the top of the stack.
///
/// # Safety
/// `lstate` must be a live Lua state with that table at -1.
unsafe fn set_cfunction(
    lstate: *mut lua_State,
    name: &CStr,
    f: unsafe extern "C-unwind" fn(*mut lua_State) -> c_int,
) {
    unsafe {
        lua_pushcfunction(lstate, f);
        lua_setfield(lstate, -2, name.as_ptr());
    }
}

/// Access to internal functions. For use in `runtime/`.
///
/// # Safety
/// `lstate` must be a live Lua state with the `vim` table at -1.
unsafe fn nlua_state_add_internal(lstate: *mut lua_State) {
    unsafe {
        set_cfunction(lstate, c"_getvar", nlua_getvar);
        set_cfunction(lstate, c"_setvar", nlua_setvar);
        set_cfunction(lstate, c"_foldupdate", nlua_foldupdate);
        set_cfunction(lstate, c"_with_c", nlua_with);
    }
}

/// Register every `vim.*` function this module implements.
///
/// `is_thread` states are cut down to what is thread-safe: they get the
/// vendored libraries and nothing that reaches editor state.
///
/// # Safety
/// `lstate` must be a live Lua state with the `vim` table on top.
pub unsafe fn nlua_state_add_stdlib(lstate: *mut lua_State, is_thread: bool) {
    unsafe {
        if !is_thread {
            // TODO(bfredl): some of the basic string functions should already
            // be (or be easy to make) threadsafe.
            set_cfunction(lstate, c"stricmp", nlua_stricmp);
            set_cfunction(lstate, c"_str_utfindex", nlua_str_utfindex);
            set_cfunction(lstate, c"_str_byteindex", nlua_str_byteindex);
            set_cfunction(lstate, c"str_utf_pos", nlua_str_utf_pos);
            set_cfunction(lstate, c"str_utf_start", nlua_str_utf_start);
            set_cfunction(lstate, c"str_utf_end", nlua_str_utf_end);

            set_cfunction(lstate, c"regex", nlua_regex);
            luaL_newmetatable(lstate, c"nvim_regex".as_ptr());
            luaL_register(lstate, ptr::null(), REGEX_META.as_ptr());
            lua_pushvalue(lstate, -1); // [meta, meta]
            lua_setfield(lstate, -2, c"__index".as_ptr()); // [meta]
            lua_pop(lstate, 1); // don't use metatable now

            // vim.spell
            luaopen_spell(lstate);
            lua_setfield(lstate, -2, c"spell".as_ptr());

            // vim.iconv -- depends on p_ambw, p_emoji
            set_cfunction(lstate, c"iconv", nlua_iconv);

            // vim.base64
            luaopen_base64(lstate);
            lua_setfield(lstate, -2, c"base64".as_ptr());

            nlua_state_add_internal(lstate);
        }

        // Put the vendored library on top of the stack onto the `vim` table
        // under `name` *and* into `package.loaded`, so `require` hands back
        // this same table rather than initialising the library a second time.
        // `depth` is how far below the library's table the `vim` table has
        // sunk -- each library leaves its own table behind, so it grows.
        let share_module = |name: &CStr, depth: c_int| {
            lua_pushvalue(lstate, -1);
            lua_setfield(lstate, -depth - 1, name.as_ptr());

            lua_getglobal(lstate, c"package".as_ptr());
            lua_getfield(lstate, -1, c"loaded".as_ptr());
            lua_pushvalue(lstate, -3);
            lua_setfield(lstate, -2, name.as_ptr());
            lua_pop(lstate, depth + 1);
        };

        // vim.mpack -- shared, or luv is reinitialised by require'mpack'.
        luaopen_mpack(lstate);
        share_module(c"mpack", 2);

        // vim.lpeg
        luaopen_lpeg(lstate);
        share_module(c"lpeg", 3);

        // vim.text.diff
        // TODO(justinmk): set vim.text.diff here, or rename this to "_diff".
        set_cfunction(lstate, c"diff", nlua_xdl_diff);

        // vim.json
        lua_cjson_new(lstate);
        lua_setfield(lstate, -2, c"json".as_ptr());
    }
}

/// Like `luaL_error`, but leaves the message on the stack instead of throwing,
/// so the caller can clean up before its own `lua_error`.
///
/// # Safety
/// `lstate` must be a live Lua state, and `fmt`'s directives must match the
/// variadic arguments.
pub unsafe extern "C-unwind" fn nlua_push_errstr(
    lstate: *mut lua_State,
    fmt: *const c_char,
    mut args: ...
) {
    unsafe {
        luaL_where(lstate, 1);
        lua_pushvfstring(lstate, fmt, args.clone());
        lua_concat(lstate, 2);
    }
}
