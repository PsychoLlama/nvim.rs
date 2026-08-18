//! `vim.schedule()` and `vim.wait()`.
//!
//! [`nlua_schedule`] defers a Lua reference onto the main loop's event queue,
//! which is how a fast callback reaches anything that is not fast.
//! [`nlua_wait`] is the other direction: it pumps the loop until a condition
//! callback returns true, a timeout expires or the user interrupts, and it
//! is the one place a Lua function drives the event loop re-entrantly.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_int, c_void};
use core::ptr;

use super::{in_fast_callback, nlua_error, nlua_pcall, nlua_pushref, nlua_ref_global};
use crate::eval::funcs::find_internal_func;
use crate::event::r#loop::process_events_until;
use crate::event::multiqueue::multiqueue_put_event;
use crate::event::time::{
    time_watcher_close, time_watcher_init, time_watcher_start, time_watcher_stop,
};
use crate::getchar::vgetc;
use crate::lua::executor::nlua_unref_global;
use crate::lua::ffi::{
    LUA_MULTRET, LUA_TFUNCTION, LUA_TNIL, lua_error, lua_gettop, lua_insert, lua_pop,
    lua_pushboolean, lua_pushinteger, lua_pushlstring, lua_pushnil, lua_pushvalue, lua_remove,
    lua_settop, lua_toboolean, lua_type, luaL_checkinteger, luaL_checknumber, luaL_error,
    luaL_getmetafield,
};
use crate::main::{
    e_fast_api_disabled, expr_map_lock, got_int, main_loop, textlock, ui_event_ns_id,
};
use crate::memory::{xfree, xmalloc};
use crate::os::cshim::gettext;
use crate::types::{
    EvalFuncDef, Event, LuaRef, MultiQueue, TimeWatcher, int64_t, intptr_t, lua_Integer, lua_State,
    ptrdiff_t, uint32_t, uint64_t,
};
use crate::ui::{ui_flush, ui_remove_cb};

/// How often `vim.wait` wakes to re-test its condition, in milliseconds.
const DEFAULT_INTERVAL: intptr_t = 200;

/// `vim.wait`'s answer for "the user interrupted".
const WAIT_INTERRUPTED: lua_Integer = -2;
/// `vim.wait`'s answer for "the timeout expired".
const WAIT_TIMED_OUT: lua_Integer = -1;

/// Run the deferred callback, on the main loop.
///
/// A callback scheduled from a UI event handler (`ns_id > 0`) is let out of
/// the text and expression locks, because it is not running inside whatever
/// held them; if it fails, that handler is detached.
///
/// # Safety
/// `argv` must be the two-element array [`nlua_schedule`] queued.
unsafe extern "C" fn nlua_schedule_event(argv: *mut *mut c_void) {
    unsafe {
        let cb = (*argv.add(0)).expose_provenance() as ptrdiff_t as LuaRef;
        let ns_id = (*argv.add(1)).expose_provenance() as ptrdiff_t as uint32_t;
        let lstate = super::get_global_lstate();
        nlua_pushref(lstate, cb);
        nlua_unref_global(lstate, cb);

        let save_expr_map_lock = expr_map_lock.get();
        let save_textlock = textlock.get();
        if ns_id > 0 {
            expr_map_lock.set(0);
            textlock.set(0);
        }
        if nlua_pcall(lstate, 0, 0) != 0 {
            nlua_error(lstate, gettext(c"vim.schedule callback: %.*s".as_ptr()));
            ui_remove_cb(ns_id, true);
        }
        expr_map_lock.set(save_expr_map_lock);
        textlock.set(save_textlock);
    }
}

/// `vim.schedule(fn)`.
///
/// Answers `nil, nil` on success and `nil, "main loop is closing"` when there
/// is no queue left to defer onto — the second value is the reason, so a
/// caller can tell the two apart.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub(crate) unsafe extern "C-unwind" fn nlua_schedule(lstate: *mut lua_State) -> c_int {
    unsafe {
        if lua_type(lstate, 1) != LUA_TFUNCTION {
            push_str(lstate, c"vim.schedule: expected function");
            return lua_error(lstate);
        }
        lua_pushnil(lstate);
        if (*main_loop.ptr()).closing {
            push_str(lstate, c"main loop is closing");
            return 2;
        }

        let cb: LuaRef = nlua_ref_global(lstate, 1);
        multiqueue_put_event(
            (*main_loop.ptr()).events,
            Event::new(
                Some(nlua_schedule_event),
                [
                    ptr::with_exposed_provenance_mut::<c_void>(cb as ptrdiff_t as usize),
                    ptr::with_exposed_provenance_mut::<c_void>(
                        ui_event_ns_id.get() as ptrdiff_t as usize
                    ),
                ],
            ),
        );
        lua_pushnil(lstate);
        2
    }
}

/// Push a literal, without its terminator.
///
/// # Safety
/// `lstate` must be a live Lua state with room for one more value.
unsafe fn push_str(lstate: *mut lua_State, s: &core::ffi::CStr) {
    unsafe { lua_pushlstring(lstate, s.as_ptr(), s.count_bytes()) };
}

/// The `vim.wait` heartbeat: it exists so that `process_events_until` wakes
/// up at all, and only ever does anything when the loop is shutting down.
///
/// # Safety
/// Called by the event loop with the watcher it was started on.
unsafe extern "C" fn dummy_timer_due_cb(tw: *mut TimeWatcher, _data: *mut c_void) {
    unsafe {
        if (*main_loop.ptr()).closing {
            time_watcher_stop(tw);
            time_watcher_close(tw, Some(dummy_timer_close_cb));
        }
    }
}

/// # Safety
/// Called by the event loop with the watcher being closed.
unsafe extern "C" fn dummy_timer_close_cb(tw: *mut TimeWatcher, _data: *mut c_void) {
    unsafe { xfree(tw.cast::<c_void>()) };
}

/// Call `vim.wait`'s condition once.
///
/// `true` means stop waiting — either the call failed (`*status`) or it
/// answered truthily, in which case the answer itself is removed and the
/// `*nresults` values behind it are the caller's return values. A falsy
/// answer leaves the stack as it was found.
///
/// # Safety
/// `lstate` must be a live Lua state with the condition at slot 2, and every
/// out-parameter writable.
unsafe fn nlua_wait_condition(
    lstate: *mut lua_State,
    status: *mut c_int,
    callback_result: *mut bool,
    nresults: *mut c_int,
) -> bool {
    unsafe {
        let top = lua_gettop(lstate);
        lua_pushvalue(lstate, 2);
        *status = nlua_pcall(lstate, 0, LUA_MULTRET);
        if *status != 0 {
            return true;
        }
        *nresults = lua_gettop(lstate) - top;
        if *nresults == 0 {
            *callback_result = false;
            return false;
        }
        *callback_result = lua_toboolean(lstate, top + 1) != 0;
        if !*callback_result {
            lua_settop(lstate, top);
            return false;
        }
        lua_remove(lstate, top + 1);
        *nresults -= 1;
        true
    }
}

/// `vim.wait(timeout[, condition[, interval[, fast_only]]])`.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub(crate) unsafe extern "C-unwind" fn nlua_wait(lstate: *mut lua_State) -> c_int {
    unsafe {
        if in_fast_callback.get() != 0 {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const _,
                c"vim.wait".as_ptr(),
            );
        }

        let timeout_number = luaL_checknumber(lstate, 1);
        if timeout_number < 0.0 {
            return luaL_error(lstate, c"timeout must be >= 0".as_ptr());
        }
        let timeout: int64_t = if timeout_number.is_nan() || timeout_number > int64_t::MAX as f64 {
            int64_t::MAX
        } else {
            timeout_number as int64_t
        };

        let lua_top = lua_gettop(lstate);

        // The condition may be a function or anything with a `__call`.
        let mut is_function = false;
        if lua_top >= 2 && lua_type(lstate, 2) != LUA_TNIL {
            is_function = lua_type(lstate, 2) == LUA_TFUNCTION;
            if !is_function && luaL_getmetafield(lstate, 2, c"__call".as_ptr()) != 0 {
                is_function = lua_type(lstate, -1) == LUA_TFUNCTION;
                lua_pop(lstate, 1);
            }
            if !is_function {
                push_str(lstate, c"vim.wait: callback must be callable");
                return lua_error(lstate);
            }
        }

        let mut interval = DEFAULT_INTERVAL;
        if lua_top >= 3 && lua_type(lstate, 3) != LUA_TNIL {
            interval = luaL_checkinteger(lstate, 3) as intptr_t;
            if interval < 0 {
                return luaL_error(lstate, c"interval must be >= 0".as_ptr());
            }
        }

        let fast_only = lua_top >= 4 && lua_toboolean(lstate, 4) != 0;
        let loop_events: *mut MultiQueue = if fast_only {
            (*main_loop.ptr()).fast_events
        } else {
            (*main_loop.ptr()).events
        };

        // The watcher is what makes the loop wake up on `interval`; it does
        // nothing else, and it outlives this call if the loop is closing.
        let tw = xmalloc(size_of::<TimeWatcher>()).cast::<TimeWatcher>();
        time_watcher_init(main_loop.ptr(), tw, ptr::null_mut());
        (*tw).events = ptr::null_mut::<MultiQueue>();
        time_watcher_start(
            tw,
            Some(dummy_timer_due_cb),
            interval as uint64_t,
            interval as uint64_t,
        );

        let mut pcall_status: c_int = 0;
        let mut callback_result = false;
        let mut nresults: c_int = 0;

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
        time_watcher_close(tw, Some(dummy_timer_close_cb));

        if pcall_status != 0 {
            lua_error(lstate)
        } else if callback_result {
            lua_pushboolean(lstate, 1);
            if nresults == 0 {
                lua_pushnil(lstate);
                nresults = 1;
            } else {
                lua_insert(lstate, -1 - nresults);
            }
            nresults + 1
        } else if got_int.get() {
            // Swallow the interrupt: `vim.wait` reports it rather than
            // letting it escape into whatever called it.
            got_int.set(false);
            vgetc();
            lua_pushboolean(lstate, 0);
            lua_pushinteger(lstate, WAIT_INTERRUPTED);
            2
        } else {
            lua_pushboolean(lstate, 0);
            lua_pushinteger(lstate, WAIT_TIMED_OUT);
            2
        }
    }
}

/// `vim.in_fast_event()`.
///
/// # Safety
/// `lstate` must be a live Lua state with room for one more value.
pub unsafe extern "C-unwind" fn nlua_in_fast_event(lstate: *mut lua_State) -> c_int {
    unsafe {
        lua_pushboolean(lstate, (in_fast_callback.get() > 0) as c_int);
        1
    }
}

/// Whether the named Vimscript builtin is marked `fast`, i.e. callable from
/// a fast callback.
///
/// # Safety
/// `name` must be a NUL-terminated function name.
pub(crate) unsafe fn viml_func_is_fast(name: *const core::ffi::c_char) -> bool {
    unsafe {
        let fdef: *const EvalFuncDef = find_internal_func(name);
        !fdef.is_null() && (*fdef).fast
    }
}

/// Whether anything that is not `fast` may run right now.
pub unsafe fn nlua_is_deferred_safe() -> bool {
    in_fast_callback.get() == 0
}
