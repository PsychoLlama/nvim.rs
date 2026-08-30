//! `vim.g` / `vim.b` / `vim.w` / `vim.t` / `vim.v`: the scope accessors.
//!
//! [`nlua_get_var_scope`] resolves the (scope, handle) pair the accessor was
//! built with to the dictionary it names, and [`nlua_setvar`]/[`nlua_getvar`]
//! are the `__newindex`/`__index` behind it — including the `vim.v` rules,
//! where a variable may be read-only or fixed-typed.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::{ERROR_INIT, nlua_push_errstr};
use crate::api::private::helpers::{
    dict_check_writable, find_buffer_by_handle, find_tab_by_handle, find_window_by_handle,
};
use crate::eval::typval::{
    TV_INITIAL_VALUE, tv_clear, tv_copy, tv_dict_add, tv_dict_find, tv_dict_is_watched,
    tv_dict_item_alloc_len, tv_dict_item_remove, tv_dict_watcher_notify,
};
use crate::eval::vars::{before_set_vvar, get_globvar_dict, get_vimvar_dict};
use crate::ex_eval::aborting;
use crate::lua::converter::{nlua_pop_typval, nlua_push_typval};
use crate::lua::ffi::{
    LUA_TNIL, lua_error, lua_gettop, lua_pushvalue, lua_type, luaL_checkinteger, luaL_checklstring,
    luaL_error,
};
use crate::runtime::script_autoload;
use crate::types::{
    Buffer, String_0, Tabpage, Window, dict_T, dictitem_T, handle_T, lua_State, ptrdiff_t, size_t,
};

/// The dictionary the `(scope, handle)` pair at stack slots 1 and 2 names.
///
/// Never returns for a scope letter it does not know, or for a handle that
/// names nothing: both leave through `luaL_error`/`lua_error`, which longjmp
/// past every frame between here and the `lua_pcall` that entered Lua.
///
/// # Safety
/// `lstate` must be a live Lua state with a scope string at 1 and a handle at 2.
unsafe fn nlua_get_var_scope(lstate: *mut lua_State) -> *mut dict_T {
    unsafe {
        let scope = CStr::from_ptr(luaL_checklstring(lstate, 1, ptr::null_mut()));
        let handle = luaL_checkinteger(lstate, 2) as handle_T;
        let mut err = ERROR_INIT;
        let dict = match scope.to_bytes() {
            b"g" => get_globvar_dict(),
            b"v" => get_vimvar_dict(),
            b"b" => {
                let buf = find_buffer_by_handle(handle as Buffer, &mut err);
                if buf.is_null() {
                    ptr::null_mut()
                } else {
                    (*buf).b_vars
                }
            }
            b"w" => {
                let win = find_window_by_handle(handle as Window, &mut err);
                if win.is_null() {
                    ptr::null_mut()
                } else {
                    (*win).w_vars
                }
            }
            b"t" => {
                let tabpage = find_tab_by_handle(handle as Tabpage, &mut err);
                if tabpage.is_null() {
                    ptr::null_mut()
                } else {
                    (*tabpage).tp_vars
                }
            }
            _ => {
                luaL_error(lstate, c"invalid scope".as_ptr());
                return ptr::null_mut();
            }
        };
        if err.is_set() {
            let why = err.message_or_empty().as_ptr();
            nlua_push_errstr(lstate, c"scoped variable: %s".as_ptr(), why);
            err.clear();
            lua_error(lstate);
            return ptr::null_mut();
        }
        dict
    }
}

/// `__newindex` on a scope table: set slot 3's key to slot 4's value, or
/// delete it when slot 4 is absent or nil.
///
/// # Safety
/// `lstate` must be a live Lua state holding this accessor's arguments.
pub unsafe extern "C-unwind" fn nlua_setvar(lstate: *mut lua_State) -> c_int {
    unsafe {
        // Non-local return if the scope names nothing.
        let dict = nlua_get_var_scope(lstate);
        let mut key = String_0::NULL;
        let data = luaL_checklstring(lstate, 3, key.len_mut()).cast_mut();
        key.set_data(data);

        let del = lua_gettop(lstate) < 4 || lua_type(lstate, 4) == LUA_TNIL;

        let mut err = ERROR_INIT;
        let mut di: *mut dictitem_T = dict_check_writable(dict, key, del, &mut err);
        if err.is_set() {
            nlua_push_errstr(lstate, c"%s".as_ptr(), err.message_or_empty().as_ptr());
            err.clear();
            lua_error(lstate);
            return 0;
        }

        let watched = tv_dict_is_watched(dict);

        if del {
            if di.is_null() {
                // Doesn't exist, nothing to do.
                return 0;
            }
            if watched {
                tv_dict_watcher_notify(dict, key.data(), ptr::null_mut(), &raw mut (*di).di_tv);
            }
            tv_dict_item_remove(dict, di);
            return 0;
        }

        // Convert the Lua value into a temporary before anything is disturbed.
        let mut tv = TV_INITIAL_VALUE;
        lua_pushvalue(lstate, 4);
        if !nlua_pop_typval(lstate, &raw mut tv) {
            return luaL_error(lstate, c"Couldn't convert lua value".as_ptr());
        }

        let mut oldtv = TV_INITIAL_VALUE;
        if di.is_null() {
            di = tv_dict_item_alloc_len(key.data(), key.len());
            let _ = tv_dict_add(dict, di);
        } else {
            let mut type_error = false;
            if dict == get_vimvar_dict()
                && !before_set_vvar(
                    key.data(),
                    di,
                    &raw mut tv,
                    true,
                    watched,
                    &raw mut type_error,
                )
            {
                tv_clear(&raw mut tv);
                if type_error {
                    return luaL_error(
                        lstate,
                        c"Setting v:%s to value with wrong type".as_ptr(),
                        key.data(),
                    );
                }
                return 0;
            }
            if watched {
                tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
            }
            tv_clear(&raw mut (*di).di_tv);
        }

        tv_copy(&raw mut tv, &raw mut (*di).di_tv);

        if watched {
            tv_dict_watcher_notify(dict, key.data(), &raw mut tv, &raw mut oldtv);
            tv_clear(&raw mut oldtv);
        }
        tv_clear(&raw mut tv);
        0
    }
}

/// `__index` on a scope table: push the value of slot 3's key, or nothing.
///
/// A miss in `g:` tries the autoload directory once before giving up.
///
/// # Safety
/// `lstate` must be a live Lua state holding this accessor's arguments.
pub unsafe extern "C-unwind" fn nlua_getvar(lstate: *mut lua_State) -> c_int {
    unsafe {
        // Non-local return if the scope names nothing.
        let dict = nlua_get_var_scope(lstate);
        let mut len: size_t = 0;
        let name: *const c_char = luaL_checklstring(lstate, 3, &raw mut len);
        let mut di = tv_dict_find(dict, name, len as ptrdiff_t);
        if di.is_null() && dict == get_globvar_dict() {
            if !script_autoload(name, len, false) || aborting() {
                return 0; // nil
            }
            di = tv_dict_find(dict, name, len as ptrdiff_t);
        }
        if di.is_null() {
            return 0; // nil
        }
        nlua_push_typval(lstate, &raw mut (*di).di_tv, 0);
        1
    }
}
