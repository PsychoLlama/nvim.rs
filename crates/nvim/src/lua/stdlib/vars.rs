//! `vim.g` / `vim.b` / `vim.w` / `vim.t` / `vim.v`: the scope accessors.
//!
//! [`nlua_get_var_scope`] resolves the (scope, handle) pair the accessor was
//! built with to the dictionary it names, and [`nlua_setvar`]/[`nlua_getvar`]
//! are the `__newindex`/`__index` behind it — including the `vim.v` rules,
//! where a variable may be read-only or fixed-typed.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::nlua_push_errstr;
use crate::api::private::helpers::{
    dict_check_writable, find_buffer_by_handle, find_tab_by_handle, find_window_by_handle,
};
use crate::eval::typval::{
    TV_INITIAL_VALUE, dict_find, dict_is_watched, dict_watcher_notify, tv_clear, tv_copy,
    tv_dict_item_alloc_len, tv_dict_item_remove,
};
use crate::eval::vars::{before_set_vvar, get_globvar_dict, get_vimvar_dict};
use crate::ex_eval::aborting;
use crate::lua::converter::{nlua_pop_typval, nlua_push_typval};
use crate::lua::ffi::{
    LUA_TNIL, lua_error, lua_gettop, lua_pushvalue, lua_type, luaL_checkinteger, luaL_checklstring,
    luaL_error,
};
use crate::narrow::number_as_int;
use crate::runtime::script_autoload;
use crate::types::{
    BufferHandle, Dict, DictItem, Handle, String_0, TabpageHandle, WindowHandle, lua_State, size_t,
};

/// The dictionary the `(scope, handle)` pair at stack slots 1 and 2 names.
///
/// Never returns for a scope letter it does not know, or for a handle that
/// names nothing: both leave through `luaL_error`/`lua_error`, which longjmp
/// past every frame between here and the `lua_pcall` that entered Lua.
///
/// # Safety
/// `lstate` must be a live Lua state with a scope string at 1 and a handle at 2.
unsafe fn nlua_get_var_scope(lstate: *mut lua_State) -> *mut Dict {
    unsafe {
        let scope = CStr::from_ptr(luaL_checklstring(lstate, 1, ptr::null_mut()));
        let handle: Handle = number_as_int(luaL_checkinteger(lstate, 2) as i64);
        // A handle that names nothing answers a null dictionary, which is
        // what the caller reports on; why it did not resolve is not read.
        match scope.to_bytes() {
            b"g" => get_globvar_dict(),
            b"v" => get_vimvar_dict(),
            b"b" => find_buffer_by_handle(handle as BufferHandle)
                .unwrap_or_default()
                .map_or(ptr::null_mut(), |buf| buf.b_vars),
            b"w" => find_window_by_handle(handle as WindowHandle)
                .unwrap_or_default()
                .map_or(ptr::null_mut(), |win| win.w_vars),
            b"t" => find_tab_by_handle(handle as TabpageHandle)
                .unwrap_or_default()
                .map_or(ptr::null_mut(), |tabpage| tabpage.tp_vars),
            _ => {
                luaL_error(lstate, c"invalid scope".as_ptr());
                ptr::null_mut()
            }
        }
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
        let mut key_len: size_t = 0;
        let data = luaL_checklstring(lstate, 3, &raw mut key_len);
        // Copied out of the Lua state: the key outlives the value below,
        // which runs Lua and may collect the string it came from.
        let key = String_0::from_raw_bytes(data, key_len);

        let del = lua_gettop(lstate) < 4 || lua_type(lstate, 4) == LUA_TNIL;

        let mut di: *mut DictItem = match dict_check_writable(dict, &key, del) {
            Ok(di) => di,
            Err(e) => {
                nlua_push_errstr(lstate, c"%s".as_ptr(), e.message_or_empty().as_ptr());
                lua_error(lstate);
                return 0;
            }
        };

        let watched = dict_is_watched((dict).as_ref());

        if del {
            if di.is_null() {
                // Doesn't exist, nothing to do.
                return 0;
            }
            if watched {
                dict_watcher_notify(dict, key.as_cstr(), None, Some(&(*di).di_tv));
            }
            tv_dict_item_remove(dict, di);
            return 0;
        }

        // Convert the Lua value into a temporary before anything is disturbed.
        lua_pushvalue(lstate, 4);
        let Some(mut tv) = nlua_pop_typval(lstate) else {
            return luaL_error(lstate, c"Couldn't convert lua value".as_ptr());
        };

        let mut oldtv = TV_INITIAL_VALUE;
        if di.is_null() {
            di = tv_dict_item_alloc_len(key.data(), key.len());
            let _ = (*dict).add_item(di);
        } else {
            let mut type_error = false;
            if dict == get_vimvar_dict()
                && !before_set_vvar(key.data(), di, &mut tv, true, watched, &raw mut type_error)
            {
                tv_clear(&mut tv);
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
                tv_copy(&(*di).di_tv, &mut oldtv);
            }
            tv_clear(&mut (*di).di_tv);
        }

        tv_copy(&tv, &mut (*di).di_tv);

        if watched {
            dict_watcher_notify(dict, key.as_cstr(), Some(&tv), Some(&oldtv));
            tv_clear(&mut oldtv);
        }
        tv_clear(&mut tv);
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
        let mut di = dict_find(dict.as_ref(), cstr::slice_at(name, len));
        if di.is_none() && dict == get_globvar_dict() {
            if !script_autoload(name, len, false) || aborting() {
                return 0; // nil
            }
            // The autoload ran arbitrary Vimscript, so the lookup starts
            // again rather than reusing the borrow it invalidated.
            di = dict_find(dict.as_ref(), cstr::slice_at(name, len));
        }
        let Some(di) = di else {
            return 0; // nil
        };
        nlua_push_typval(lstate, &di.di_tv, 0);
        1
    }
}
