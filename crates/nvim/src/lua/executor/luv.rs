//! The luv bridge: running libuv callbacks on the right state.
//!
//! luv is given four function pointers (`cfpcall`, `cfcpcall`, and the
//! acquire/release pair) so that a callback raised from a libuv thread is
//! executed against *that* thread's `lua_State`, and one raised on the main
//! loop against the global one.  [`nlua_fast_cfpcall`] is the main-loop half,
//! and it is what sets `in_fast_callback` -- the flag every api function
//! marked `fast` is allowed to run under.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::{
    LUVF_CALLBACK_NOEXIT, in_fast_callback, kThread, kThreadCallback, nlua_get_error,
    nlua_init_state, nlua_luv_error_event, nlua_pcall,
};
use crate::api::private::helpers::{api_clear_error, api_free_array};
use crate::event::r#loop::loop_schedule_deferred;
use crate::event::multiqueue::multiqueue_put_event;
use crate::ex_getln::ERROR_INIT;
use crate::lua::converter::{kNluaPushSpecial, nlua_pop_Array, nlua_push_Array};
use crate::lua::ffi::{
    LUA_MULTRET, LUA_REGISTRYINDEX, LUA_TBOOLEAN, LUA_TTABLE, lua_close, lua_concat, lua_error,
    lua_getfield, lua_gettop, lua_pcall, lua_pop, lua_pushcclosure, lua_pushlightuserdata,
    lua_pushstring, lua_toboolean, lua_tostring, lua_type, luaL_checktype, luaL_error, luaL_where,
};
use crate::main::{e_outofmem, main_loop, preserve_exit};
use crate::memory::xstrdup;
use crate::os::libc::{fprintf, pthread_exit, stderr};
use crate::runtime::runtime_get_named_thread;
use crate::types::{
    Arena, Array, Event, intptr_t, kErrorTypeNone, lua_CFunction, lua_State, size_t,
};

/// `lua_pcall`'s "out of memory" status, the one failure that is not a
/// recoverable Lua error.
const LUA_ERRMEM: c_int = 4;

/// The main loop's `luv_set_callback`: run a libuv callback against the
/// global state, with `in_fast_callback` raised for its duration.
///
/// A failure is *reported* rather than propagated — there is no caller to
/// return an error to — so the message is copied and scheduled onto the
/// event queue.
///
/// # Safety
/// `lstate` must be a live Lua state with a function and `nargs` arguments
/// on top.
pub(crate) unsafe extern "C-unwind" fn nlua_fast_cfpcall(
    lstate: *mut lua_State,
    nargs: c_int,
    mut nresult: c_int,
    flags: c_int,
) -> c_int {
    unsafe {
        (*in_fast_callback.ptr()) += 1;
        let top = lua_gettop(lstate);
        let status = nlua_pcall(lstate, nargs, nresult);
        let retval = if status != 0 {
            if status == LUA_ERRMEM && flags & LUVF_CALLBACK_NOEXIT == 0 {
                preserve_exit(&raw const e_outofmem as *const _);
            }
            let mut len: size_t = 0;
            let error = nlua_get_error(lstate, &raw mut len);
            multiqueue_put_event(
                (*main_loop.ptr()).events,
                Event::new(Some(nlua_luv_error_event), [dup_error(error)]),
            );
            lua_pop(lstate, 1);
            -status
        } else {
            if nresult == LUA_MULTRET {
                nresult = lua_gettop(lstate) - top + nargs + 1;
            }
            nresult
        };
        (*in_fast_callback.ptr()) -= 1;
        retval
    }
}

/// A copy of the error text for the event queue to own and free, or null.
///
/// # Safety
/// `error` must be null or a NUL-terminated string.
unsafe fn dup_error(error: *const c_char) -> *mut c_void {
    unsafe {
        if error.is_null() {
            ptr::null_mut()
        } else {
            xstrdup(error).cast::<c_void>()
        }
    }
}

/// A libuv *thread*'s `luv_set_callback`.
///
/// # Safety
/// As [`nlua_fast_cfpcall`], on that thread's own state.
pub(crate) unsafe extern "C-unwind" fn nlua_luv_thread_cb_cfpcall(
    lstate: *mut lua_State,
    nargs: c_int,
    nresult: c_int,
    flags: c_int,
) -> c_int {
    unsafe { nlua_luv_thread_common_cfpcall(lstate, nargs, nresult, flags, true) }
}

/// A libuv thread's `luv_set_thread`.
///
/// # Safety
/// As [`nlua_luv_thread_cb_cfpcall`].
pub(crate) unsafe extern "C-unwind" fn nlua_luv_thread_cfpcall(
    lstate: *mut lua_State,
    nargs: c_int,
    nresult: c_int,
    flags: c_int,
) -> c_int {
    unsafe { nlua_luv_thread_common_cfpcall(lstate, nargs, nresult, flags, false) }
}

/// A libuv thread's `luv_set_cthread`: the same, for a C function and one
/// light-userdata argument.
///
/// # Safety
/// `lstate` must be a live Lua state and `func` a callable C function.
pub(crate) unsafe extern "C-unwind" fn nlua_luv_thread_cfcpcall(
    lstate: *mut lua_State,
    func: lua_CFunction,
    ud: *mut c_void,
    flags: c_int,
) -> c_int {
    unsafe {
        lua_pushcclosure(lstate, func, 0);
        lua_pushlightuserdata(lstate, ud);
        nlua_luv_thread_cfpcall(lstate, 1, 0, flags)
    }
}

/// The shared body of the two thread `cfpcall`s.
///
/// Off the main loop there is no `debug.traceback` to install and nothing to
/// `preserve_exit` into, so an out-of-memory failure closes the state and
/// leaves the thread outright; any other failure is deferred onto the main
/// loop for reporting.
///
/// # Safety
/// As [`nlua_luv_thread_cb_cfpcall`].
unsafe extern "C-unwind" fn nlua_luv_thread_common_cfpcall(
    lstate: *mut lua_State,
    nargs: c_int,
    mut nresult: c_int,
    flags: c_int,
    is_callback: bool,
) -> c_int {
    unsafe {
        let top = lua_gettop(lstate);
        let status = lua_pcall(lstate, nargs, nresult, 0);
        if status != 0 {
            if status == LUA_ERRMEM && flags & LUVF_CALLBACK_NOEXIT == 0 {
                fprintf(stderr, c"%s\n".as_ptr(), &raw const e_outofmem as *const _);
                lua_close(lstate);
                pthread_exit(ptr::null_mut());
            }
            let error = lua_tostring(lstate, -1);
            let kind = if is_callback {
                kThreadCallback
            } else {
                kThread
            };
            loop_schedule_deferred(
                main_loop.ptr(),
                Event::new(
                    Some(nlua_luv_error_event),
                    [
                        dup_error(error),
                        ptr::with_exposed_provenance_mut::<c_void>(kind as intptr_t as usize),
                    ],
                ),
            );
            lua_pop(lstate, 1);
            return -status;
        }
        if nresult == LUA_MULTRET {
            nresult = lua_gettop(lstate) - top + nargs + 1;
        }
        nresult
    }
}

/// `nvim__get_runtime()` as a thread state sees it: the one api function a
/// luv thread is allowed, because it reads only the runtime path.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub(crate) unsafe extern "C-unwind" fn nlua_thr_api_nvim__get_runtime(
    lstate: *mut lua_State,
) -> c_int {
    unsafe {
        if lua_gettop(lstate) != 3 {
            return luaL_error(lstate, c"Expected 3 arguments".as_ptr());
        }

        luaL_checktype(lstate, -1, LUA_TTABLE);
        lua_getfield(lstate, -1, c"is_lua".as_ptr());
        if lua_type(lstate, -1) != LUA_TBOOLEAN {
            return luaL_error(lstate, c"is_lua is not a boolean".as_ptr());
        }
        let is_lua = lua_toboolean(lstate, -1) != 0;
        lua_pop(lstate, 2);

        luaL_checktype(lstate, -1, LUA_TBOOLEAN);
        let all = lua_toboolean(lstate, -1) != 0;
        lua_pop(lstate, 1);

        let mut err = ERROR_INIT;
        let pat: Array = nlua_pop_Array(lstate, ptr::null_mut::<Arena>(), &raw mut err);
        if err.type_0 != kErrorTypeNone {
            luaL_where(lstate, 1);
            lua_pushstring(lstate, err.msg);
            api_clear_error(&raw mut err);
            lua_concat(lstate, 2);
            return lua_error(lstate);
        }

        let ret = runtime_get_named_thread(is_lua, pat, all);
        nlua_push_Array(lstate, ret, kNluaPushSpecial as c_int);
        api_free_array(ret);
        api_free_array(pat);
        1
    }
}

/// luv's `acquire_vm`: a thread that needs a state gets a fresh one.
///
/// # Safety
/// Called by luv on a thread with no state of its own.
pub(crate) unsafe extern "C-unwind" fn nlua_thread_acquire_vm() -> *mut lua_State {
    unsafe { nlua_init_state(true) }
}

/// `vim.is_thread()`.
///
/// # Safety
/// `lstate` must be a live Lua state with room for one more value.
pub(crate) unsafe extern "C-unwind" fn nlua_is_thread(lstate: *mut lua_State) -> c_int {
    unsafe {
        lua_getfield(lstate, LUA_REGISTRYINDEX, c"nvim.thread".as_ptr());
        1
    }
}
