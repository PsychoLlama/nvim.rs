//! Building the `vim` table itself.
//!
//! `nlua_common_vim_init` is the shared half every state gets -- the api
//! functions, the `vim.*` C functions, the metatables -- and
//! `nlua_init_packages` runs `vim._init_packages` over the embedded Lua
//! modules.  `nlua_ui_attach`/`nlua_ui_detach` are `vim.ui_attach()`, whose
//! callbacks the compositor calls back into.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C-unwind" fn nlua_common_vim_init(
    mut lstate: *mut lua_State,
    mut is_thread: bool,
    mut is_standalone: bool,
) {
    unsafe {
        let mut ref_state: *mut nlua_ref_state_t = nlua_new_ref_state(lstate, is_thread);
        lua_setfield(
            lstate,
            LUA_REGISTRYINDEX,
            b"nlua.ref_state\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushboolean(lstate, is_thread as ::core::ffi::c_int);
        lua_setfield(
            lstate,
            LUA_REGISTRYINDEX,
            b"nvim.thread\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_is_thread as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"is_thread\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_newuserdata(lstate, 0 as size_t);
        lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        lua_pushcclosure(
            lstate,
            Some(
                nlua_nil_tostring
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_setmetatable(lstate, -2 as ::core::ffi::c_int);
        (*ref_state).nil_ref = nlua_ref(lstate, ref_state, -1 as ::core::ffi::c_int);
        lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
        lua_setfield(
            lstate,
            LUA_REGISTRYINDEX,
            b"mpack.NIL\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"NIL\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        lua_pushcclosure(
            lstate,
            Some(
                nlua_empty_dict_tostring
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        );
        (*ref_state).empty_dict_ref = nlua_ref(lstate, ref_state, -1 as ::core::ffi::c_int);
        lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
        lua_setfield(
            lstate,
            LUA_REGISTRYINDEX,
            b"mpack.empty_dict\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"_empty_dict_mt\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if !is_standalone {
            if is_thread {
                luv_set_callback(
                    lstate,
                    Some(
                        nlua_luv_thread_cb_cfpcall
                            as unsafe extern "C-unwind" fn(
                                *mut lua_State,
                                ::core::ffi::c_int,
                                ::core::ffi::c_int,
                                ::core::ffi::c_int,
                            )
                                -> ::core::ffi::c_int,
                    ),
                );
                luv_set_thread(
                    lstate,
                    Some(
                        nlua_luv_thread_cfpcall
                            as unsafe extern "C-unwind" fn(
                                *mut lua_State,
                                ::core::ffi::c_int,
                                ::core::ffi::c_int,
                                ::core::ffi::c_int,
                            )
                                -> ::core::ffi::c_int,
                    ),
                );
                luv_set_cthread(
                    lstate,
                    Some(
                        nlua_luv_thread_cfcpcall
                            as unsafe extern "C-unwind" fn(
                                *mut lua_State,
                                lua_CFunction,
                                *mut ::core::ffi::c_void,
                                ::core::ffi::c_int,
                            )
                                -> ::core::ffi::c_int,
                    ),
                );
            } else {
                luv_set_loop(lstate, &raw mut (*main_loop.ptr()).uv);
                luv_set_callback(
                    lstate,
                    Some(
                        nlua_fast_cfpcall
                            as unsafe extern "C-unwind" fn(
                                *mut lua_State,
                                ::core::ffi::c_int,
                                ::core::ffi::c_int,
                                ::core::ffi::c_int,
                            )
                                -> ::core::ffi::c_int,
                    ),
                );
            }
        }
        luaopen_luv(lstate);
        lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
        lua_setfield(
            lstate,
            -3 as ::core::ffi::c_int,
            b"uv\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
        lua_setfield(
            lstate,
            -3 as ::core::ffi::c_int,
            b"loop\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"package\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_getfield(
            lstate,
            -1 as ::core::ffi::c_int,
            b"loaded\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushvalue(lstate, -3 as ::core::ffi::c_int);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"luv\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_settop(lstate, -3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_module_preloader(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: size_t =
            lua_tointeger(lstate, LUA_GLOBALSINDEX - 1 as ::core::ffi::c_int) as size_t;
        let mut def: ModuleDef = (*builtin_modules.ptr())[i as usize];
        if luaL_loadbuffer(
            lstate,
            def.data as *const ::core::ffi::c_char,
            def.size.wrapping_sub(1 as size_t),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ) != 0
        {
            return lua_error(lstate);
        }
        lua_call(lstate, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_init_packages(
    mut lstate: *mut lua_State,
    mut is_standalone: bool,
) -> bool {
    unsafe {
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"package\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_getfield(
            lstate,
            -1 as ::core::ffi::c_int,
            b"preload\0".as_ptr() as *const ::core::ffi::c_char,
        );
        let mut i: size_t = 0 as size_t;
        while i < ::core::mem::size_of::<[ModuleDef; 21]>()
            .wrapping_div(::core::mem::size_of::<ModuleDef>())
            .wrapping_div(
                (::core::mem::size_of::<[ModuleDef; 21]>()
                    .wrapping_rem(::core::mem::size_of::<ModuleDef>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            let mut def: ModuleDef = (*builtin_modules.ptr())[i as usize];
            lua_pushinteger(lstate, i as lua_Integer);
            lua_pushcclosure(
                lstate,
                Some(
                    nlua_module_preloader
                        as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                ),
                1 as ::core::ffi::c_int,
            );
            lua_setfield(lstate, -2 as ::core::ffi::c_int, def.name);
            if nlua_disable_preload.get() as ::core::ffi::c_int != 0
                && !is_standalone
                && strequal(
                    def.name,
                    b"vim.inspect\0".as_ptr() as *const ::core::ffi::c_char,
                ) as ::core::ffi::c_int
                    != 0
            {
                break;
            }
            i = i.wrapping_add(1);
        }
        lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"require\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushstring(
            lstate,
            b"vim._init_packages\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if nlua_pcall(lstate, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
            fprintf(
                stderr,
                b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
                lua_tolstring(
                    lstate,
                    -1 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<size_t>(),
                ),
            );
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_ui_attach(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ns_id: uint32_t = luaL_checkinteger(lstate, 1 as ::core::ffi::c_int) as uint32_t;
        if !ns_initialized(ns_id) {
            return luaL_error(
                lstate,
                b"invalid ns_id\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if !(lua_type(lstate, 2 as ::core::ffi::c_int) == LUA_TTABLE) {
            return luaL_error(
                lstate,
                b"opts must be a table\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if !(lua_type(lstate, 3 as ::core::ffi::c_int) == LUA_TFUNCTION) {
            return luaL_error(
                lstate,
                b"callback must be a Lua function\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut ext_widgets: [bool; 5] = [false_0 != 0, false, false, false, false];
        let mut tbl_has_true_val: bool = false_0 != 0;
        lua_pushvalue(lstate, 2 as ::core::ffi::c_int);
        lua_pushnil(lstate);
        while lua_next(lstate, -2 as ::core::ffi::c_int) != 0 {
            let mut len: size_t = 0;
            let mut s: *const ::core::ffi::c_char =
                lua_tolstring(lstate, -2 as ::core::ffi::c_int, &raw mut len);
            let mut val: bool = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
            '_ok: {
                if strequal(s, b"set_cmdheight\0".as_ptr() as *const ::core::ffi::c_char) {
                    ui_refresh_cmdheight.set(val);
                } else {
                    let mut i: size_t = 0 as size_t;
                    while i < kUILinegrid as ::core::ffi::c_int as size_t {
                        if strequal(
                            s,
                            *(ui_ext_names.ptr() as *mut *const ::core::ffi::c_char)
                                .offset(i as isize),
                        ) {
                            if val {
                                tbl_has_true_val = true_0 != 0;
                            }
                            ext_widgets[i as usize] = val;
                            break '_ok;
                        } else {
                            i = i.wrapping_add(1);
                        }
                    }
                    return luaL_error(
                        lstate,
                        b"Unexpected key: %s\0".as_ptr() as *const ::core::ffi::c_char,
                        s,
                    );
                }
            }
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
        if !tbl_has_true_val {
            return luaL_error(
                lstate,
                b"opts table must contain at least one 'true' ext_widget\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
        let mut ui_event_cb: LuaRef = nlua_ref_global(lstate, 3 as ::core::ffi::c_int);
        ui_add_cb(ns_id, ui_event_cb, &raw mut ext_widgets as *mut bool);
        ui_refresh_cmdheight.set(true_0 != 0);
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_ui_detach(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ns_id: uint32_t = luaL_checkinteger(lstate, 1 as ::core::ffi::c_int) as uint32_t;
        if !ns_initialized(ns_id) {
            return luaL_error(
                lstate,
                b"invalid ns_id\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        ui_remove_cb(ns_id, false_0 != 0);
        return 0 as ::core::ffi::c_int;
    }
}
