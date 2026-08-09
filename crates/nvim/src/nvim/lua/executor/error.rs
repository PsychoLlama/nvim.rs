//! Turning a Lua error into an editor error.
//!
//! [`nlua_get_error`] formats whatever is on the stack -- a string, or a
//! table with a `__tostring` -- and [`nlua_error`] prints it through the
//! multiline message path.  [`nlua_pcall`] is the protected call every entry
//! point in this module funnels through, and [`nlua_luv_error_event`] is the
//! same for an error raised on the event loop, where there is no caller to
//! return to.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_multiline_c;
use core::ffi::{CStr, c_char, c_int, c_void};

use super::{in_script, kCallback, kThread, kThreadCallback, luv_err_t};
use crate::src::nvim::lua::ffi::{
    LUA_MULTRET, LUA_TFUNCTION, lua_getfield, lua_getglobal, lua_gettop, lua_insert, lua_pcall,
    lua_pop, lua_remove, lua_replace, lua_tolstring, lua_type, luaL_callmeta, luaL_getmetafield,
};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::os::libc::{fprintf, stderr};
use crate::src::nvim::types::{intptr_t, lua_State, size_t};

/// The message kind every Lua error is reported under.
const LUA_ERROR_KIND: &CStr = c"lua_error";
/// The metamethod a Lua error object may render itself through.
const TOSTRING: &CStr = c"__tostring";

/// The error on top of the stack as text, leaving it there.
///
/// # Safety
/// `lstate` must be a live Lua state with the error value on top.
pub(crate) unsafe extern "C-unwind" fn nlua_get_error(
    lstate: *mut lua_State,
    len: *mut size_t,
) -> *const c_char {
    unsafe {
        if luaL_getmetafield(lstate, -1, TOSTRING.as_ptr()) != 0 {
            if lua_type(lstate, -1) == LUA_TFUNCTION
                && luaL_callmeta(lstate, -2, TOSTRING.as_ptr()) != 0
            {
                lua_replace(lstate, -3);
            }
            lua_pop(lstate, 1);
        }
        lua_tolstring(lstate, -1, len)
    }
}

/// Report the error on top of the stack, and pop it.  `msg` is a format with
/// one `%.*s` for the error's own text.
///
/// A `-l` script has no editor to show a message in, so its errors go to
/// stderr.
///
/// # Safety
/// As [`nlua_get_error`].
pub unsafe extern "C-unwind" fn nlua_error(lstate: *mut lua_State, msg: *const c_char) {
    unsafe {
        let mut len: size_t = 0;
        let str = nlua_get_error(lstate, &raw mut len);
        if in_script.get() {
            fprintf(stderr, msg, len as c_int, str);
            fprintf(stderr, c"\n".as_ptr());
        } else {
            semsg_multiline_c!(LUA_ERROR_KIND.as_ptr(), msg, len as c_int, str);
        }
        lua_pop(lstate, 1);
    }
}

/// `lua_pcall` with `debug.traceback` installed as the message handler, so a
/// failure carries the Lua stack that produced it.
///
/// The handler is inserted below the function and its arguments, and removed
/// again whichever way the call went — so the stack effect is exactly
/// `lua_pcall`'s.
///
/// # Safety
/// `lstate` must be a live Lua state with a function and `nargs` arguments
/// on top.
pub unsafe extern "C-unwind" fn nlua_pcall(
    lstate: *mut lua_State,
    nargs: c_int,
    mut nresults: c_int,
) -> c_int {
    unsafe {
        lua_getglobal(lstate, c"debug".as_ptr());
        lua_getfield(lstate, -1, c"traceback".as_ptr());
        lua_remove(lstate, -2);
        lua_insert(lstate, -2 - nargs);
        let pre_top = lua_gettop(lstate);
        let status = lua_pcall(lstate, nargs, nresults, -2 - nargs);
        if status != 0 {
            lua_remove(lstate, -2);
        } else {
            if nresults == LUA_MULTRET {
                nresults = lua_gettop(lstate) - (pre_top - nargs - 1);
            }
            lua_remove(lstate, -1 - nresults);
        }
        status
    }
}

/// Report an error raised where no caller can see it: a luv callback, a luv
/// thread, or a callback on one.
///
/// Scheduled onto the main loop by whoever raised it; `argv[0]` is the
/// message, which this frees, and `argv[1]` the [`luv_err_t`].
///
/// # Safety
/// `argv` must be the two-element array the scheduler was handed.
pub(crate) unsafe extern "C" fn nlua_luv_error_event(argv: *mut *mut c_void) {
    unsafe {
        let error = (*argv.add(0)).cast::<c_char>();
        let type_0 = (*argv.add(1)).expose_provenance() as intptr_t as luv_err_t;
        // An unknown kind reports nothing, exactly as upstream's `switch`
        // with no `default` does — but the message is still freed.
        let fmt = match type_0 {
            kCallback => Some(c"Lua callback:\n%s"),
            kThread => Some(c"Luv thread:\n%s"),
            kThreadCallback => Some(c"Luv callback, thread:\n%s"),
            _ => None,
        };
        if let Some(fmt) = fmt {
            semsg_multiline_c!(LUA_ERROR_KIND.as_ptr(), fmt.as_ptr(), error);
        }
        xfree(error.cast::<c_void>());
    }
}
