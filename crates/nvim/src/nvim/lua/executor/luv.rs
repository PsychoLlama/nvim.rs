//! The luv bridge: running libuv callbacks on the right state.
//!
//! luv is given four function pointers (`cfpcall`, `cfcpcall`, and the
//! acquire/release pair) so that a callback raised from a libuv thread is
//! executed against *that* thread's `lua_State`, and one raised on the main
//! loop against the global one.  `nlua_fast_cfpcall` is the main-loop half,
//! and it is what sets `in_fast_callback` -- the flag every api function
//! marked `fast` is allowed to run under.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C-unwind" fn nlua_fast_cfpcall(
    mut lstate: *mut lua_State,
    mut nargs: ::core::ffi::c_int,
    mut nresult: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut retval: ::core::ffi::c_int = 0;
        (*in_fast_callback.ptr()) += 1;
        let mut top: ::core::ffi::c_int = lua_gettop(lstate);
        let mut status: ::core::ffi::c_int = nlua_pcall(lstate, nargs, nresult);
        if status != 0 {
            if status == LUA_ERRMEM && flags & LUVF_CALLBACK_NOEXIT == 0 {
                preserve_exit(&raw const e_outofmem as *const ::core::ffi::c_char);
            }
            let mut len: size_t = 0;
            let mut error: *const ::core::ffi::c_char = nlua_get_error(lstate, &raw mut len);
            multiqueue_put_event(
                (*main_loop.ptr()).events,
                Event {
                    handler: Some(
                        nlua_luv_error_event
                            as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                    ),
                    argv: [
                        (if !error.is_null() {
                            xstrdup(error)
                        } else {
                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                        }) as *mut ::core::ffi::c_void,
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ],
                },
            );
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            retval = -status;
        } else {
            if nresult == LUA_MULTRET {
                nresult = lua_gettop(lstate) - top + nargs + 1 as ::core::ffi::c_int;
            }
            retval = nresult;
        }
        (*in_fast_callback.ptr()) -= 1;
        return retval;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_luv_thread_cb_cfpcall(
    mut lstate: *mut lua_State,
    mut nargs: ::core::ffi::c_int,
    mut nresult: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return nlua_luv_thread_common_cfpcall(lstate, nargs, nresult, flags, true_0 != 0);
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_luv_thread_cfpcall(
    mut lstate: *mut lua_State,
    mut nargs: ::core::ffi::c_int,
    mut nresult: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return nlua_luv_thread_common_cfpcall(lstate, nargs, nresult, flags, false_0 != 0);
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_luv_thread_cfcpcall(
    mut lstate: *mut lua_State,
    mut func: lua_CFunction,
    mut ud: *mut ::core::ffi::c_void,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        lua_pushcclosure(lstate, func, 0 as ::core::ffi::c_int);
        lua_pushlightuserdata(lstate, ud);
        let mut retval: ::core::ffi::c_int = nlua_luv_thread_cfpcall(
            lstate,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            flags,
        );
        return retval;
    }
}

unsafe extern "C-unwind" fn nlua_luv_thread_common_cfpcall(
    mut lstate: *mut lua_State,
    mut nargs: ::core::ffi::c_int,
    mut nresult: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut is_callback: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut retval: ::core::ffi::c_int = 0;
        let mut top: ::core::ffi::c_int = lua_gettop(lstate);
        let mut status: ::core::ffi::c_int =
            lua_pcall(lstate, nargs, nresult, 0 as ::core::ffi::c_int);
        if status != 0 {
            if status == LUA_ERRMEM && flags & LUVF_CALLBACK_NOEXIT == 0 {
                fprintf(
                    stderr,
                    b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw const e_outofmem as *const ::core::ffi::c_char,
                );
                lua_close(lstate);
                pthread_exit(::core::ptr::null_mut::<::core::ffi::c_void>());
            }
            let mut error: *const ::core::ffi::c_char = lua_tolstring(
                lstate,
                -1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<size_t>(),
            );
            loop_schedule_deferred(
                main_loop.ptr(),
                Event {
                    handler: Some(
                        nlua_luv_error_event
                            as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                    ),
                    argv: [
                        (if !error.is_null() {
                            xstrdup(error)
                        } else {
                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                        }) as *mut ::core::ffi::c_void,
                        ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                            (if is_callback as ::core::ffi::c_int != 0 {
                                kThreadCallback as ::core::ffi::c_int
                            } else {
                                kThread as ::core::ffi::c_int
                            }) as intptr_t as usize,
                        ),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ],
                },
            );
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            retval = -status;
        } else {
            if nresult == LUA_MULTRET {
                nresult = lua_gettop(lstate) - top + nargs + 1 as ::core::ffi::c_int;
            }
            retval = nresult;
        }
        return retval;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_thr_api_nvim__get_runtime(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
            return luaL_error(
                lstate,
                b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        luaL_checktype(lstate, -1 as ::core::ffi::c_int, LUA_TTABLE);
        lua_getfield(
            lstate,
            -1 as ::core::ffi::c_int,
            b"is_lua\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if !(lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TBOOLEAN) {
            return luaL_error(
                lstate,
                b"is_lua is not a boolean\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut is_lua: bool = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
        lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        luaL_checktype(lstate, -1 as ::core::ffi::c_int, LUA_TBOOLEAN);
        let mut all: bool = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let pat: Array = nlua_pop_Array(lstate, ::core::ptr::null_mut::<Arena>(), &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            luaL_where(lstate, 1 as ::core::ffi::c_int);
            lua_pushstring(lstate, err.msg);
            api_clear_error(&raw mut err);
            lua_concat(lstate, 2 as ::core::ffi::c_int);
            return lua_error(lstate);
        }
        let mut ret: Array = runtime_get_named_thread(is_lua, pat, all);
        nlua_push_Array(lstate, ret, kNluaPushSpecial as ::core::ffi::c_int);
        api_free_array(ret);
        api_free_array(pat);
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_thread_acquire_vm() -> *mut lua_State {
    unsafe {
        return nlua_init_state(true_0 != 0);
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_is_thread(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        lua_getfield(
            lstate,
            LUA_REGISTRYINDEX,
            b"nvim.thread\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 1 as ::core::ffi::c_int;
    }
}
