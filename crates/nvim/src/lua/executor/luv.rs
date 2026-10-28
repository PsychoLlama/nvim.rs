//! The luv bridge: running libuv callbacks on the right state.
//!
//! luv is given four function pointers (`cfpcall`, `cfcpcall`, and the
//! acquire/release pair) so that a callback raised from a libuv thread is
//! executed against *that* thread's `lua_State`, and one raised on the main
//! loop against the global one. [`nlua_fast_cfpcall`] is the main-loop half:
//! its body is [`fast_pcall`](super::fast_pcall), which is what raises
//! `in_fast_callback` -- the flag every api function marked `fast` is
//! allowed to run under.
//!
//! What stays raw here is luv's C surface: the hooks it calls with a bare
//! `lua_State`, and opening the library onto a state.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md (`lua/executor/luv.rs`).
#![allow(unsafe_code)]
// luv's C API: `luv_CFpcall` and its neighbours are that library's exported
// names.
#![allow(non_camel_case_types)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_int, c_void};
use core::ptr;
use std::io::Write;

use super::{LuvError, fast_pcall, nlua_init_state, report_luv_error};
use crate::event::r#loop::{main_loop_raw, schedule_deferred_job};
use crate::lua::ffi::{LUA_MULTRET, lua_close, luaopen_luv, luv_set_loop};
use crate::lua::stack::{At, Stack};
use crate::message::e_outofmem;
use crate::types::{lua_CFunction, lua_State};
use ::libc::pthread_exit;

unsafe extern "C" {
    fn luv_set_callback(L: *mut lua_State, pcall: luv_CFpcall);
    fn luv_set_thread(L: *mut lua_State, pcall: luv_CFpcall);
    fn luv_set_cthread(L: *mut lua_State, cpcall: luv_CFcpcall);
}

/// luv's protected-call hook: `(L, nargs, nresults, flags)`.
type luv_CFpcall =
    Option<unsafe extern "C-unwind" fn(*mut lua_State, c_int, c_int, c_int) -> c_int>;
/// luv's C-function protected-call hook.
type luv_CFcpcall =
    Option<unsafe extern "C-unwind" fn(*mut lua_State, lua_CFunction, *mut c_void, c_int) -> c_int>;

/// luv's "do not `preserve_exit` on out of memory" flag.
pub(crate) const LUVF_CALLBACK_NOEXIT: c_int = 0x1;

/// `lua_pcall`'s "out of memory" status, the one failure that is not a
/// recoverable Lua error.
pub(crate) const LUA_ERRMEM: c_int = 4;

/// Wire luv into the state with the `vim` table on top: the callback hooks
/// (a thread's, or the main loop's), then `vim.uv`, `vim.loop` and
/// `package.loaded.luv`, which are one table. A `-l` script gets the
/// library but no hooks.
///
/// # Safety
/// `lstate` must be a live state with the `vim` table on top, on the thread
/// it belongs to.
pub(crate) unsafe fn install_luv(lstate: *mut lua_State, is_thread: bool, is_standalone: bool) {
    // SAFETY: the caller's state; the hooks are functions of the shape luv
    // calls, and the main loop is a static.
    unsafe {
        if !is_standalone {
            if is_thread {
                luv_set_callback(lstate, Some(nlua_luv_thread_cb_cfpcall));
                luv_set_thread(lstate, Some(nlua_luv_thread_cfpcall));
                luv_set_cthread(lstate, Some(nlua_luv_thread_cfcpcall));
            } else {
                luv_set_loop(lstate, &raw mut (*main_loop_raw()).uv);
                luv_set_callback(lstate, Some(nlua_fast_cfpcall));
            }
        }
        luaopen_luv(lstate);
    }
    // SAFETY: the caller's state; this function's own frame.
    let mut stack = unsafe { Stack::from_raw(lstate) };
    stack.push_value(At::TOP);
    stack.set_field(At::from_top(3), c"uv");
    stack.push_value(At::TOP);
    stack.set_field(At::from_top(3), c"loop");

    stack.get_global(c"package");
    stack.get_field(At::TOP, c"loaded");
    stack.push_value(At::from_top(3));
    stack.set_field(At::from_top(2), c"luv");
    stack.pop(3);
}

/// The main loop's `luv_set_callback`: [`fast_pcall`] on the global state.
///
/// # Safety
/// luv's contract: `lstate` is the live main state with a function and
/// `nargs` arguments on top.
unsafe extern "C-unwind" fn nlua_fast_cfpcall(
    lstate: *mut lua_State,
    nargs: c_int,
    nresult: c_int,
    flags: c_int,
) -> c_int {
    // SAFETY: the caller's state, and this function's own frame.
    fast_pcall(
        &mut unsafe { Stack::from_raw(lstate) },
        nargs,
        nresult,
        flags,
    )
}

/// A libuv *thread*'s `luv_set_callback`.
///
/// # Safety
/// As [`nlua_fast_cfpcall`], on that thread's own state.
unsafe extern "C-unwind" fn nlua_luv_thread_cb_cfpcall(
    lstate: *mut lua_State,
    nargs: c_int,
    nresult: c_int,
    flags: c_int,
) -> c_int {
    // SAFETY: the caller's promise.
    unsafe { nlua_luv_thread_common_cfpcall(lstate, nargs, nresult, flags, true) }
}

/// A libuv thread's `luv_set_thread`.
///
/// # Safety
/// As [`nlua_luv_thread_cb_cfpcall`].
unsafe extern "C-unwind" fn nlua_luv_thread_cfpcall(
    lstate: *mut lua_State,
    nargs: c_int,
    nresult: c_int,
    flags: c_int,
) -> c_int {
    // SAFETY: the caller's promise.
    unsafe { nlua_luv_thread_common_cfpcall(lstate, nargs, nresult, flags, false) }
}

/// A libuv thread's `luv_set_cthread`: the same, for a C function and one
/// light-userdata argument.
///
/// # Safety
/// `lstate` must be a live Lua state and `func` a callable C function.
unsafe extern "C-unwind" fn nlua_luv_thread_cfcpcall(
    lstate: *mut lua_State,
    func: lua_CFunction,
    ud: *mut c_void,
    flags: c_int,
) -> c_int {
    // SAFETY: the caller's state, and this function's own frame.
    let mut stack = unsafe { Stack::from_raw(lstate) };
    stack.push_function(func.expect("luv hands over a function"));
    stack.push_light_userdata(ud);
    // SAFETY: the caller's promise.
    unsafe { nlua_luv_thread_cfpcall(lstate, 1, 0, flags) }
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
unsafe fn nlua_luv_thread_common_cfpcall(
    lstate: *mut lua_State,
    nargs: c_int,
    mut nresult: c_int,
    flags: c_int,
    is_callback: bool,
) -> c_int {
    // SAFETY: the caller's state, and this function's own frame.
    let mut stack = unsafe { Stack::from_raw(lstate) };
    let top = stack.top().index();
    if let Err(failed) = stack.pcall(nargs, nresult, None) {
        let status = failed.0;
        if status == LUA_ERRMEM && flags & LUVF_CALLBACK_NOEXIT == 0 {
            let _ = std::io::stderr().write_all(&[e_outofmem.to_bytes(), b"\n"].concat());
            // SAFETY: the thread's own state, closed once, and the thread
            // left without touching it again.
            unsafe {
                lua_close(lstate);
                pthread_exit(ptr::null_mut());
            }
        }
        // `lua_tostring`'s copy: up to the first NUL, or nothing at all.
        let error = stack.to_bytes(At::TOP).map(|text| {
            let end = text.iter().position(|&b| b == 0).unwrap_or(text.len());
            text[..end].to_vec()
        });
        let kind = if is_callback {
            LuvError::ThreadCallback
        } else {
            LuvError::Thread
        };
        schedule_deferred_job(Box::new(move || report_luv_error(kind, error.as_deref())));
        stack.pop(1);
        return -status;
    }
    if nresult == LUA_MULTRET {
        nresult = stack.top().index() - top + nargs + 1;
    }
    nresult
}

/// luv's `acquire_vm`: a thread that needs a state gets a fresh one.
///
/// # Safety
/// Called by luv on a thread with no state of its own.
pub(crate) unsafe extern "C-unwind" fn nlua_thread_acquire_vm() -> *mut lua_State {
    nlua_init_state(true)
}
