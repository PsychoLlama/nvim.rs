//! `vim.g` / `vim.b` / `vim.w` / `vim.t` / `vim.v`: the scope accessors.
//!
//! `nlua_get_var_scope` resolves the (scope, handle) pair the accessor was
//! built with to the dictionary it names, and `nlua_setvar`/`nlua_getvar`
//! are the `__newindex`/`__index` behind it -- including the `vim.v` rules,
//! where a variable may be read-only or fixed-typed.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C-unwind" fn nlua_get_var_scope(mut lstate: *mut lua_State) -> *mut dict_T {
    unsafe {
        let mut scope: *const ::core::ffi::c_char = luaL_checklstring(
            lstate,
            1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        );
        let mut handle: handle_T = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int) as handle_T;
        let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        if strequal(scope, b"g\0".as_ptr() as *const ::core::ffi::c_char) {
            dict = get_globvar_dict();
        } else if strequal(scope, b"v\0".as_ptr() as *const ::core::ffi::c_char) {
            dict = get_vimvar_dict();
        } else if strequal(scope, b"b\0".as_ptr() as *const ::core::ffi::c_char) {
            let mut buf: *mut buf_T = find_buffer_by_handle(handle as Buffer, &raw mut err);
            if !buf.is_null() {
                dict = (*buf).b_vars;
            }
        } else if strequal(scope, b"w\0".as_ptr() as *const ::core::ffi::c_char) {
            let mut win: *mut win_T = find_window_by_handle(handle as Window, &raw mut err);
            if !win.is_null() {
                dict = (*win).w_vars;
            }
        } else if strequal(scope, b"t\0".as_ptr() as *const ::core::ffi::c_char) {
            let mut tabpage: *mut tabpage_T = find_tab_by_handle(handle as Tabpage, &raw mut err);
            if !tabpage.is_null() {
                dict = (*tabpage).tp_vars;
            }
        } else {
            luaL_error(
                lstate,
                b"invalid scope\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return ::core::ptr::null_mut::<dict_T>();
        }
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            nlua_push_errstr(
                lstate,
                b"scoped variable: %s\0".as_ptr() as *const ::core::ffi::c_char,
                err.msg,
            );
            api_clear_error(&raw mut err);
            lua_error(lstate);
            return ::core::ptr::null_mut::<dict_T>();
        }
        return dict;
    }
}

pub unsafe extern "C-unwind" fn nlua_setvar(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut dict: *mut dict_T = nlua_get_var_scope(lstate);
        let mut key: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        };
        key.data = luaL_checklstring(lstate, 3 as ::core::ffi::c_int, &raw mut key.size)
            as *mut ::core::ffi::c_char;
        let mut del: bool = lua_gettop(lstate) < 4 as ::core::ffi::c_int
            || lua_type(lstate, 4 as ::core::ffi::c_int) == LUA_TNIL;
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut di: *mut dictitem_T = dict_check_writable(dict, key, del, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            nlua_push_errstr(
                lstate,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                err.msg,
            );
            api_clear_error(&raw mut err);
            lua_error(lstate);
            return 0 as ::core::ffi::c_int;
        }
        let mut watched: bool = tv_dict_is_watched(dict);
        if del {
            if di.is_null() {
                return 0 as ::core::ffi::c_int;
            }
            if watched {
                tv_dict_watcher_notify(
                    dict,
                    key.data,
                    ::core::ptr::null_mut::<typval_T>(),
                    &raw mut (*di).di_tv,
                );
            }
            tv_dict_item_remove(dict, di);
        } else {
            let mut tv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            lua_pushvalue(lstate, 4 as ::core::ffi::c_int);
            if !nlua_pop_typval(lstate, &raw mut tv) {
                return luaL_error(
                    lstate,
                    b"Couldn't convert lua value\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            let mut oldtv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if di.is_null() {
                di = tv_dict_item_alloc_len(key.data, key.size);
                tv_dict_add(dict, di);
            } else {
                let mut type_error: bool = false_0 != 0;
                if dict == get_vimvar_dict()
                    && !before_set_vvar(
                        key.data,
                        di,
                        &raw mut tv,
                        true_0 != 0,
                        watched,
                        &raw mut type_error,
                    )
                {
                    tv_clear(&raw mut tv);
                    if type_error {
                        return luaL_error(
                            lstate,
                            b"Setting v:%s to value with wrong type\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            key.data,
                        );
                    }
                    return 0 as ::core::ffi::c_int;
                }
                if watched {
                    tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
                }
                tv_clear(&raw mut (*di).di_tv);
            }
            tv_copy(&raw mut tv, &raw mut (*di).di_tv);
            if watched {
                tv_dict_watcher_notify(dict, key.data, &raw mut tv, &raw mut oldtv);
                tv_clear(&raw mut oldtv);
            }
            tv_clear(&raw mut tv);
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C-unwind" fn nlua_getvar(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut dict: *mut dict_T = nlua_get_var_scope(lstate);
        let mut len: size_t = 0;
        let mut name: *const ::core::ffi::c_char =
            luaL_checklstring(lstate, 3 as ::core::ffi::c_int, &raw mut len);
        let mut di: *mut dictitem_T = tv_dict_find(dict, name, len as ptrdiff_t);
        if di.is_null() && dict == get_globvar_dict() {
            if !script_autoload(name, len, false_0 != 0) || aborting() as ::core::ffi::c_int != 0 {
                return 0 as ::core::ffi::c_int;
            }
            di = tv_dict_find(dict, name, len as ptrdiff_t);
        }
        if di.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        nlua_push_typval(lstate, &raw mut (*di).di_tv, 0 as ::core::ffi::c_int);
        return 1 as ::core::ffi::c_int;
    }
}
