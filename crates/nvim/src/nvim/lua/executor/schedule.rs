//! `vim.schedule()` and `vim.wait()`.
//!
//! `nlua_schedule` defers a Lua reference onto the main loop's event queue,
//! which is how a fast callback reaches anything that is not fast.
//! `nlua_wait` is the other direction: it pumps the loop until a condition
//! callback returns true, a timeout expires or the user interrupts, and it
//! is the one place a Lua function drives the event loop re-entrantly.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn nlua_schedule_event(mut argv: *mut *mut ::core::ffi::c_void) {
    unsafe {
        let mut cb: LuaRef = (*argv.offset(0 as ::core::ffi::c_int as isize)).expose_provenance()
            as ptrdiff_t as LuaRef;
        let mut ns_id: uint32_t = (*argv.offset(1 as ::core::ffi::c_int as isize))
            .expose_provenance() as ptrdiff_t as uint32_t;
        let lstate: *mut lua_State = global_lstate.get();
        nlua_pushref(lstate, cb);
        nlua_unref_global(lstate, cb);
        let mut save_expr_map_lock: ::core::ffi::c_int = expr_map_lock.get();
        let mut save_textlock: ::core::ffi::c_int = textlock.get();
        expr_map_lock.set(if ns_id > 0 as uint32_t {
            0 as ::core::ffi::c_int
        } else {
            expr_map_lock.get()
        });
        textlock.set(if ns_id > 0 as uint32_t {
            0 as ::core::ffi::c_int
        } else {
            textlock.get()
        });
        if nlua_pcall(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
            nlua_error(
                lstate,
                gettext(b"vim.schedule callback: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
            ui_remove_cb(ns_id, true_0 != 0);
        }
        expr_map_lock.set(save_expr_map_lock);
        textlock.set(save_textlock);
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_schedule(lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        if lua_type(lstate, 1 as ::core::ffi::c_int) != LUA_TFUNCTION {
            lua_pushlstring(
                lstate,
                b"vim.schedule: expected function\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>()
                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                    .wrapping_sub(1 as size_t),
            );
            return lua_error(lstate);
        }
        lua_pushnil(lstate);
        if (*main_loop.ptr()).closing {
            lua_pushlstring(
                lstate,
                b"main loop is closing\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 21]>()
                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                    .wrapping_sub(1 as size_t),
            );
            return 2 as ::core::ffi::c_int;
        }
        let mut cb: LuaRef = nlua_ref_global(lstate, 1 as ::core::ffi::c_int);
        multiqueue_put_event(
            (*main_loop.ptr()).events,
            Event {
                handler: Some(
                    nlua_schedule_event
                        as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                        cb as ptrdiff_t as usize,
                    ),
                    ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                        ui_event_ns_id.get() as ptrdiff_t as usize,
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
        lua_pushnil(lstate);
        return 2 as ::core::ffi::c_int;
    }
}

unsafe extern "C" fn dummy_timer_due_cb(
    mut tw: *mut TimeWatcher,
    mut _data: *mut ::core::ffi::c_void,
) {
    unsafe {
        if (*main_loop.ptr()).closing {
            time_watcher_stop(tw);
            time_watcher_close(
                tw,
                Some(
                    dummy_timer_close_cb
                        as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
                ),
            );
        }
    }
}

unsafe extern "C" fn dummy_timer_close_cb(
    mut tw: *mut TimeWatcher,
    mut _data: *mut ::core::ffi::c_void,
) {
    unsafe {
        xfree(tw as *mut ::core::ffi::c_void);
    }
}

unsafe extern "C-unwind" fn nlua_wait_condition(
    mut lstate: *mut lua_State,
    mut status: *mut ::core::ffi::c_int,
    mut callback_result: *mut bool,
    mut nresults: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut top: ::core::ffi::c_int = lua_gettop(lstate);
        lua_pushvalue(lstate, 2 as ::core::ffi::c_int);
        *status = nlua_pcall(lstate, 0 as ::core::ffi::c_int, LUA_MULTRET);
        if *status != 0 {
            return true_0 != 0;
        }
        *nresults = lua_gettop(lstate) - top;
        if *nresults == 0 as ::core::ffi::c_int {
            *callback_result = false_0 != 0;
            return false_0 != 0;
        }
        *callback_result = lua_toboolean(lstate, top + 1 as ::core::ffi::c_int) != 0;
        if !*callback_result {
            lua_settop(lstate, top);
            return false_0 != 0;
        }
        lua_remove(lstate, top + 1 as ::core::ffi::c_int);
        *nresults -= 1;
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_wait(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        if in_fast_callback.get() != 0 {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"vim.wait\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut timeout_number: ::core::ffi::c_double =
            luaL_checknumber(lstate, 1 as ::core::ffi::c_int);
        if timeout_number < 0 as ::core::ffi::c_int as ::core::ffi::c_double {
            return luaL_error(
                lstate,
                b"timeout must be >= 0\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut timeout: int64_t = if timeout_number.is_nan() as i32 != 0
            || timeout_number > INT64_MAX as ::core::ffi::c_double
        {
            INT64_MAX as int64_t
        } else {
            timeout_number as int64_t
        };
        let mut lua_top: ::core::ffi::c_int = lua_gettop(lstate);
        let mut is_function: bool = false_0 != 0;
        if lua_top >= 2 as ::core::ffi::c_int
            && !(lua_type(lstate, 2 as ::core::ffi::c_int) == LUA_TNIL)
        {
            is_function = lua_type(lstate, 2 as ::core::ffi::c_int) == LUA_TFUNCTION;
            if !is_function
                && luaL_getmetafield(
                    lstate,
                    2 as ::core::ffi::c_int,
                    b"__call\0".as_ptr() as *const ::core::ffi::c_char,
                ) != 0 as ::core::ffi::c_int
            {
                is_function = lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TFUNCTION;
                lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            }
            if !is_function {
                lua_pushlstring(
                    lstate,
                    b"vim.wait: callback must be callable\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 36]>()
                        .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                        .wrapping_sub(1 as size_t),
                );
                return lua_error(lstate);
            }
        }
        let mut interval: intptr_t = 200 as ::core::ffi::c_int as intptr_t;
        if lua_top >= 3 as ::core::ffi::c_int
            && !(lua_type(lstate, 3 as ::core::ffi::c_int) == LUA_TNIL)
        {
            interval = luaL_checkinteger(lstate, 3 as ::core::ffi::c_int) as intptr_t;
            if interval < 0 as intptr_t {
                return luaL_error(
                    lstate,
                    b"interval must be >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
        let mut fast_only: bool = false_0 != 0;
        if lua_top >= 4 as ::core::ffi::c_int {
            fast_only = lua_toboolean(lstate, 4 as ::core::ffi::c_int) != 0;
        }
        let mut loop_events: *mut MultiQueue = if fast_only as ::core::ffi::c_int != 0 {
            (*main_loop.ptr()).fast_events
        } else {
            (*main_loop.ptr()).events
        };
        let mut tw: *mut TimeWatcher =
            xmalloc(::core::mem::size_of::<TimeWatcher>()) as *mut TimeWatcher;
        time_watcher_init(main_loop.ptr(), tw, NULL);
        (*tw).events = ::core::ptr::null_mut::<MultiQueue>();
        time_watcher_start(
            tw,
            Some(
                dummy_timer_due_cb
                    as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
            ),
            interval as uint64_t,
            interval as uint64_t,
        );
        let mut pcall_status: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut callback_result: bool = false_0 != 0;
        let mut nresults: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        ui_flush();
        process_events_until(main_loop.ptr(), loop_events, timeout, || {
            got_int.get()
                || is_function
                    && nlua_wait_condition(
                        lstate,
                        &raw mut pcall_status,
                        &raw mut callback_result,
                        &raw mut nresults,
                    )
        });
        time_watcher_stop(tw);
        time_watcher_close(
            tw,
            Some(
                dummy_timer_close_cb
                    as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
            ),
        );
        if pcall_status != 0 {
            return lua_error(lstate);
        } else if callback_result {
            lua_pushboolean(lstate, 1 as ::core::ffi::c_int);
            if nresults == 0 as ::core::ffi::c_int {
                lua_pushnil(lstate);
                nresults = 1 as ::core::ffi::c_int;
            } else {
                lua_insert(lstate, -1 as ::core::ffi::c_int - nresults);
            }
            return nresults + 1 as ::core::ffi::c_int;
        } else if got_int.get() {
            got_int.set(false_0 != 0);
            vgetc();
            lua_pushboolean(lstate, 0 as ::core::ffi::c_int);
            lua_pushinteger(lstate, -2 as lua_Integer);
            return 2 as ::core::ffi::c_int;
        } else {
            lua_pushboolean(lstate, 0 as ::core::ffi::c_int);
            lua_pushinteger(lstate, -1 as lua_Integer);
            return 2 as ::core::ffi::c_int;
        };
    }
}

pub unsafe extern "C-unwind" fn nlua_in_fast_event(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        lua_pushboolean(
            lstate,
            (in_fast_callback.get() > 0 as ::core::ffi::c_int) as ::core::ffi::c_int,
        );
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn viml_func_is_fast(
    mut name: *const ::core::ffi::c_char,
) -> bool {
    unsafe {
        let fdef: *const EvalFuncDef = find_internal_func(name);
        if !fdef.is_null() {
            return (*fdef).fast;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C-unwind" fn nlua_is_deferred_safe() -> bool {
    return in_fast_callback.get() == 0 as ::core::ffi::c_int;
}
