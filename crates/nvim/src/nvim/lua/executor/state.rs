//! Creating, initialising and freeing a `lua_State`.
//!
//! `nlua_init_state` is the constructor -- for the main state and for a
//! thread's -- and `nlua_state_init` the part that only the main one gets:
//! the runtime files, `vim._init_packages`, the `vim.g`-style accessors and
//! the default mappings.  `nlua_init` is what `main()` calls.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C-unwind" fn nlua_init_argv(
    L: *mut lua_State,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut lua_arg0: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        if lua_arg0 > 0 as ::core::ffi::c_int {
            lua_pushstring(
                L,
                *argv.offset((lua_arg0 - 1 as ::core::ffi::c_int) as isize),
            );
            lua_rawseti(L, -2 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            while i + lua_arg0 < argc {
                lua_pushstring(L, *argv.offset((i + lua_arg0) as isize));
                lua_rawseti(L, -2 as ::core::ffi::c_int, i + 1 as ::core::ffi::c_int);
                i += 1;
            }
        }
        lua_setfield(
            L,
            LUA_GLOBALSINDEX,
            b"arg\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return i;
    }
}

unsafe extern "C-unwind" fn nlua_state_init(lstate: *mut lua_State) -> bool {
    unsafe {
        lua_pushcclosure(
            lstate,
            Some(nlua_print as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"print\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"debug\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(nlua_debug as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"debug\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        nlua_add_api_functions(lstate);
        nlua_init_types(lstate);
        lua_pushcclosure(
            lstate,
            Some(
                nlua_schedule as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"schedule\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_in_fast_event
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"in_fast_event\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(nlua_call as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"call\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_rpcrequest
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"rpcrequest\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_rpcnotify as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"rpcnotify\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(nlua_wait as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"wait\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_ui_attach as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"ui_attach\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_ui_detach as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"ui_detach\0".as_ptr() as *const ::core::ffi::c_char,
        );
        nlua_common_vim_init(lstate, false_0 != 0, false_0 != 0);
        if !(*time_fd.ptr()).is_null() {
            lua_getfield(
                lstate,
                LUA_GLOBALSINDEX,
                b"require\0".as_ptr() as *const ::core::ffi::c_char,
            );
            require_ref.set(nlua_ref_global(lstate, -1 as ::core::ffi::c_int));
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            lua_pushcclosure(
                lstate,
                Some(
                    nlua_require
                        as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                ),
                0 as ::core::ffi::c_int,
            );
            lua_setfield(
                lstate,
                LUA_GLOBALSINDEX,
                b"require\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        nlua_treesitter_init(lstate);
        nlua_state_add_stdlib(lstate, false_0 != 0);
        lua_setfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"vim\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if !nlua_init_packages(lstate, false_0 != 0) {
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C-unwind" fn nlua_init(
    mut argv: *mut *mut ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut lua_arg0: ::core::ffi::c_int,
) {
    unsafe {
        let mut lstate: *mut lua_State = luaL_newstate();
        if lstate.is_null() {
            fprintf(
                stderr,
                gettext(b"E970: Failed to initialize Lua interpreter\n\0".as_ptr()
                    as *const ::core::ffi::c_char),
            );
            os_exit(1 as ::core::ffi::c_int);
        }
        luaL_openlibs(lstate);
        if !nlua_state_init(lstate) {
            fprintf(
                stderr,
                gettext(
                    b"E970: Failed to initialize builtin Lua modules\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
            );
            os_exit(1 as ::core::ffi::c_int);
        }
        luv_set_thread_cb(
            Some(nlua_thread_acquire_vm as unsafe extern "C-unwind" fn() -> *mut lua_State),
            Some(nlua_common_free_all_mem as unsafe extern "C-unwind" fn(*mut lua_State) -> ()),
        );
        global_lstate.set(lstate);
        active_lstate.set(lstate);
        main_thread.set(uv_thread_self());
        nlua_init_argv(lstate, argv, argc, lua_arg0);
    }
}

pub unsafe extern "C-unwind" fn nlua_run_script(
    mut argv: *mut *mut ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut lua_arg0: ::core::ffi::c_int,
) -> ! {
    unsafe {
        in_script.set(true_0 != 0);
        global_lstate.set(nlua_init_state(false_0 != 0));
        luv_set_thread_cb(
            Some(nlua_thread_acquire_vm as unsafe extern "C-unwind" fn() -> *mut lua_State),
            Some(nlua_common_free_all_mem as unsafe extern "C-unwind" fn(*mut lua_State) -> ()),
        );
        nlua_init_argv(global_lstate.get(), argv, argc, lua_arg0);
        let mut lua_ok: bool =
            nlua_exec_file(*argv.offset((lua_arg0 - 1 as ::core::ffi::c_int) as isize));
        exit(if lua_ok as ::core::ffi::c_int != 0 {
            0 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        });
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_init_state(mut thread: bool) -> *mut lua_State {
    unsafe {
        let self_0: uv_thread_t = uv_thread_self();
        if !in_script.get() && uv_thread_equal(main_thread.ptr(), &raw const self_0) != 0 {
            runtime_search_path_validate();
        }
        let mut lstate: *mut lua_State = luaL_newstate();
        luaL_openlibs(lstate);
        if !in_script.get() {
            lua_pushcclosure(
                lstate,
                Some(
                    nlua_print as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                ),
                0 as ::core::ffi::c_int,
            );
            lua_setfield(
                lstate,
                LUA_GLOBALSINDEX,
                b"print\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        lua_pushinteger(lstate, 0 as lua_Integer);
        lua_setfield(
            lstate,
            LUA_REGISTRYINDEX,
            b"nlua.refcount\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        nlua_common_vim_init(lstate, thread, in_script.get());
        nlua_state_add_stdlib(lstate, true_0 != 0);
        if !in_script.get() {
            lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            lua_pushcclosure(
                lstate,
                Some(
                    nlua_thr_api_nvim__get_runtime
                        as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                ),
                0 as ::core::ffi::c_int,
            );
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                b"nvim__get_runtime\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                b"api\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        lua_setfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"vim\0".as_ptr() as *const ::core::ffi::c_char,
        );
        nlua_init_packages(lstate, in_script.get());
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
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"vim\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"vim\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return lstate;
    }
}

unsafe extern "C-unwind" fn nlua_common_free_all_mem(mut lstate: *mut lua_State) {
    unsafe {
        let mut ref_state: *mut nlua_ref_state_t = nlua_get_ref_state(lstate);
        nlua_unref(lstate, ref_state, (*ref_state).nil_ref);
        nlua_unref(lstate, ref_state, (*ref_state).empty_dict_ref);
        lua_close(lstate);
    }
}

pub unsafe extern "C-unwind" fn nlua_init_defaults() {
    unsafe {
        let L: *mut lua_State = global_lstate.get();
        '_c2rust_label: {
            if !L.is_null() {
            } else {
                __assert_fail(
                    b"L\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2417 as ::core::ffi::c_uint,
                    b"void nlua_init_defaults(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        lua_getfield(
            L,
            LUA_GLOBALSINDEX,
            b"require\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushstring(
            L,
            b"vim._core.defaults\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if nlua_pcall(L, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
            fprintf(
                stderr,
                b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
                lua_tolstring(
                    L,
                    -1 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<size_t>(),
                ),
            );
        }
    }
}
