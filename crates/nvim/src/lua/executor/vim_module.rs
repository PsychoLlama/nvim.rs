//! Building the `vim` table itself.
//!
//! [`nlua_common_vim_init`] is the shared half every state gets -- the
//! reference table, the `vim.NIL` and `vim.empty_dict()` sentinels, luv --
//! and [`nlua_init_packages`] runs `vim._init_packages` over the embedded Lua
//! modules.  [`nlua_ui_attach`]/[`nlua_ui_detach`] are `vim.ui_attach()`,
//! whose callbacks the compositor calls back into.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};

use super::{
    BUILTIN_MODULES, luv_set_callback, luv_set_cthread, luv_set_thread, nlua_empty_dict_tostring,
    nlua_fast_cfpcall, nlua_is_thread, nlua_luv_thread_cb_cfpcall, nlua_luv_thread_cfcpcall,
    nlua_luv_thread_cfpcall, nlua_new_ref_state, nlua_nil_tostring, nlua_pcall, nlua_ref,
    nlua_ref_global,
};
use crate::api::extmark::ns_initialized;
use crate::lua::ffi::{
    LUA_REGISTRYINDEX, LUA_TFUNCTION, LUA_TTABLE, lua_call, lua_error, lua_getfield, lua_getglobal,
    lua_newtable, lua_newuserdata, lua_next, lua_pop, lua_pushboolean, lua_pushcclosure,
    lua_pushcfunction, lua_pushinteger, lua_pushnil, lua_pushstring, lua_pushvalue, lua_setfield,
    lua_setmetatable, lua_toboolean, lua_tointeger, lua_tolstring, lua_tostring, lua_type,
    lua_upvalueindex, luaL_checkinteger, luaL_error, luaL_loadbuffer, luaopen_luv, luv_set_loop,
};
use crate::main::{main_loop, nlua_disable_preload, ui_ext_names, ui_refresh_cmdheight};
use crate::memory::strequal;
use crate::os::cshim::stderr;
use crate::types::ui::kUILinegrid;
use crate::types::{LuaRef, lua_Integer, lua_State, size_t, uint32_t};
use crate::ui::{ui_add_cb, ui_remove_cb};
use ::libc::fprintf;

/// The registry key this state's reference bookkeeping is parked under.
const REF_STATE_KEY: &CStr = c"nlua.ref_state";
/// The name the module whose preload the `--luamod-dev` flag stops at.
const FIRST_DEV_MODULE: &CStr = c"vim.inspect";

/// The half of the `vim` table that a thread state gets as well as the main
/// one: the reference table, the two identity sentinels, and luv.
///
/// Expects the `vim` table on top; leaves it there.
///
/// # Safety
/// `lstate` must be a live Lua state with the `vim` table on top.
pub(crate) unsafe fn nlua_common_vim_init(
    lstate: *mut lua_State,
    is_thread: bool,
    is_standalone: bool,
) {
    unsafe {
        let ref_state = nlua_new_ref_state(lstate, is_thread);
        lua_setfield(lstate, LUA_REGISTRYINDEX, REF_STATE_KEY.as_ptr());

        lua_pushboolean(lstate, is_thread as c_int);
        lua_setfield(lstate, LUA_REGISTRYINDEX, c"nvim.thread".as_ptr());
        lua_pushcfunction(lstate, nlua_is_thread);
        lua_setfield(lstate, -2, c"is_thread".as_ptr());

        // vim.NIL: a zero-length userdatum, so it is unique and compares by
        // identity, with a metatable that renders it.
        lua_newuserdata(lstate, 0);
        lua_newtable(lstate);
        lua_pushcfunction(lstate, nlua_nil_tostring);
        lua_setfield(lstate, -2, c"__tostring".as_ptr());
        lua_setmetatable(lstate, -2);
        (*ref_state).nil_ref = nlua_ref(lstate, ref_state, -1);
        lua_pushvalue(lstate, -1);
        lua_setfield(lstate, LUA_REGISTRYINDEX, c"mpack.NIL".as_ptr());
        lua_setfield(lstate, -2, c"NIL".as_ptr());

        // vim.empty_dict(): the *metatable* is the marker, so an empty table
        // carrying it converts as a dictionary rather than a list.
        lua_newtable(lstate);
        lua_pushcfunction(lstate, nlua_empty_dict_tostring);
        lua_setfield(lstate, -2, c"__tostring".as_ptr());
        (*ref_state).empty_dict_ref = nlua_ref(lstate, ref_state, -1);
        lua_pushvalue(lstate, -1);
        lua_setfield(lstate, LUA_REGISTRYINDEX, c"mpack.empty_dict".as_ptr());
        lua_setfield(lstate, -2, c"_empty_dict_mt".as_ptr());

        if !is_standalone {
            if is_thread {
                luv_set_callback(lstate, Some(nlua_luv_thread_cb_cfpcall));
                luv_set_thread(lstate, Some(nlua_luv_thread_cfpcall));
                luv_set_cthread(lstate, Some(nlua_luv_thread_cfcpcall));
            } else {
                luv_set_loop(lstate, &raw mut (*main_loop.ptr()).uv);
                luv_set_callback(lstate, Some(nlua_fast_cfpcall));
            }
        }

        // vim.uv, vim.loop and package.loaded.luv are the same table.
        luaopen_luv(lstate);
        lua_pushvalue(lstate, -1);
        lua_setfield(lstate, -3, c"uv".as_ptr());
        lua_pushvalue(lstate, -1);
        lua_setfield(lstate, -3, c"loop".as_ptr());

        lua_getglobal(lstate, c"package".as_ptr());
        lua_getfield(lstate, -1, c"loaded".as_ptr());
        lua_pushvalue(lstate, -3);
        lua_setfield(lstate, -2, c"luv".as_ptr());
        lua_pop(lstate, 3);
    }
}

/// The `package.preload` entry for one embedded module: load its bytecode and
/// run it. Its upvalue is the module's index in [`BUILTIN_MODULES`].
///
/// # Safety
/// `lstate` must be a live Lua state and the upvalue a valid index.
pub(crate) unsafe extern "C-unwind" fn nlua_module_preloader(lstate: *mut lua_State) -> c_int {
    unsafe {
        let i = lua_tointeger(lstate, lua_upvalueindex(1)) as size_t;
        let chunk = BUILTIN_MODULES[i].chunk();
        if luaL_loadbuffer(
            lstate,
            chunk.as_ptr().cast::<c_char>(),
            chunk.len(),
            ::core::ptr::null(),
        ) != 0
        {
            return lua_error(lstate);
        }
        lua_call(lstate, 0, 1);
        1
    }
}

/// Register every embedded module in `package.preload`, then `require`
/// `vim._init_packages`, which is what wires up the rest of the runtime.
///
/// `nlua_disable_preload` (`--luamod-dev`) stops the registration at
/// `vim.inspect`, so everything after it is loaded from the runtime
/// directory instead.
///
/// # Safety
/// `lstate` must be a live Lua state.
pub(crate) unsafe fn nlua_init_packages(lstate: *mut lua_State, is_standalone: bool) -> bool {
    unsafe {
        lua_getglobal(lstate, c"package".as_ptr());
        lua_getfield(lstate, -1, c"preload".as_ptr());
        for (i, def) in BUILTIN_MODULES.iter().enumerate() {
            lua_pushinteger(lstate, i as lua_Integer);
            lua_pushcclosure(lstate, Some(nlua_module_preloader), 1);
            lua_setfield(lstate, -2, def.name.as_ptr());
            if nlua_disable_preload.get()
                && !is_standalone
                && strequal(def.name.as_ptr(), FIRST_DEV_MODULE.as_ptr())
            {
                break;
            }
        }
        lua_pop(lstate, 2);

        lua_getglobal(lstate, c"require".as_ptr());
        lua_pushstring(lstate, c"vim._init_packages".as_ptr());
        if nlua_pcall(lstate, 1, 0) != 0 {
            fprintf(stderr, c"%s\n".as_ptr(), lua_tostring(lstate, -1));
            return false;
        }
        true
    }
}

/// `vim.ui_attach(ns_id, opts, callback)`.
///
/// `opts` names the UI extensions the callback wants; at least one must be
/// true, or the attach is refused.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub(crate) unsafe extern "C-unwind" fn nlua_ui_attach(lstate: *mut lua_State) -> c_int {
    unsafe {
        let ns_id = luaL_checkinteger(lstate, 1) as uint32_t;
        if !ns_initialized(ns_id) {
            return luaL_error(lstate, c"invalid ns_id".as_ptr());
        }
        if lua_type(lstate, 2) != LUA_TTABLE {
            return luaL_error(lstate, c"opts must be a table".as_ptr());
        }
        if lua_type(lstate, 3) != LUA_TFUNCTION {
            return luaL_error(lstate, c"callback must be a Lua function".as_ptr());
        }

        let mut ext_widgets = [false; 5];
        let mut tbl_has_true_val = false;
        lua_pushvalue(lstate, 2);
        lua_pushnil(lstate);
        while lua_next(lstate, -2) != 0 {
            let mut len: size_t = 0;
            let s = lua_tolstring(lstate, -2, &raw mut len);
            let val = lua_toboolean(lstate, -1) != 0;
            if strequal(s, c"set_cmdheight".as_ptr()) {
                ui_refresh_cmdheight.set(val);
            } else {
                let found = (0..kUILinegrid as size_t).find(|&i| strequal(s, ui_ext_names[i]));
                match found {
                    Some(i) => {
                        if val {
                            tbl_has_true_val = true;
                        }
                        ext_widgets[i] = val;
                    }
                    None => return luaL_error(lstate, c"Unexpected key: %s".as_ptr(), s),
                }
            }
            lua_pop(lstate, 1);
        }
        if !tbl_has_true_val {
            return luaL_error(
                lstate,
                c"opts table must contain at least one 'true' ext_widget".as_ptr(),
            );
        }

        let ui_event_cb: LuaRef = nlua_ref_global(lstate, 3);
        ui_add_cb(ns_id, ui_event_cb, ext_widgets.as_mut_ptr());
        ui_refresh_cmdheight.set(true);
        0
    }
}

/// `vim.ui_detach(ns_id)`.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub(crate) unsafe extern "C-unwind" fn nlua_ui_detach(lstate: *mut lua_State) -> c_int {
    unsafe {
        let ns_id = luaL_checkinteger(lstate, 1) as uint32_t;
        if !ns_initialized(ns_id) {
            return luaL_error(lstate, c"invalid ns_id".as_ptr());
        }
        ui_remove_cb(ns_id, false);
        0
    }
}
