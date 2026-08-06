//! Turning a Lua error into an editor error.
//!
//! `nlua_get_error` formats whatever is on the stack -- a string, or a table
//! with a `__tostring` -- and `nlua_error` prints it through the multiline
//! message path.  `nlua_pcall` is the protected call every entry point in
//! this module funnels through, and `nlua_luv_error_event` is the same for
//! an error raised on the event loop, where there is no caller to return
//! to.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C-unwind" fn nlua_get_error(
    mut lstate: *mut lua_State,
    mut len: *mut size_t,
) -> *const ::core::ffi::c_char {
    unsafe {
        if luaL_getmetafield(
            lstate,
            -1 as ::core::ffi::c_int,
            b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0
        {
            if lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TFUNCTION
                && luaL_callmeta(
                    lstate,
                    -2 as ::core::ffi::c_int,
                    b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
                ) != 0
            {
                lua_replace(lstate, -3 as ::core::ffi::c_int);
            }
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
        return lua_tolstring(lstate, -1 as ::core::ffi::c_int, len);
    }
}

pub unsafe extern "C-unwind" fn nlua_error(
    lstate: *mut lua_State,
    msg: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut len: size_t = 0;
        let mut str: *const ::core::ffi::c_char = nlua_get_error(lstate, &raw mut len);
        if in_script.get() {
            fprintf(stderr, msg, len as ::core::ffi::c_int, str);
            fprintf(stderr, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            semsg_multiline(
                b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
                msg,
                len as ::core::ffi::c_int,
                str,
            );
        }
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C-unwind" fn nlua_pcall(
    mut lstate: *mut lua_State,
    mut nargs: ::core::ffi::c_int,
    mut nresults: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"debug\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_getfield(
            lstate,
            -1 as ::core::ffi::c_int,
            b"traceback\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_remove(lstate, -2 as ::core::ffi::c_int);
        lua_insert(lstate, -2 as ::core::ffi::c_int - nargs);
        let mut pre_top: ::core::ffi::c_int = lua_gettop(lstate);
        let mut status: ::core::ffi::c_int =
            lua_pcall(lstate, nargs, nresults, -2 as ::core::ffi::c_int - nargs);
        if status != 0 {
            lua_remove(lstate, -2 as ::core::ffi::c_int);
        } else {
            if nresults == LUA_MULTRET {
                nresults = lua_gettop(lstate) - (pre_top - nargs - 1 as ::core::ffi::c_int);
            }
            lua_remove(lstate, -1 as ::core::ffi::c_int - nresults);
        }
        return status;
    }
}

pub(crate) unsafe extern "C" fn nlua_luv_error_event(mut argv: *mut *mut ::core::ffi::c_void) {
    unsafe {
        let mut error: *mut ::core::ffi::c_char =
            *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
        let mut type_0: luv_err_t = (*argv.offset(1 as ::core::ffi::c_int as isize))
            .expose_provenance() as intptr_t as luv_err_t;
        match type_0 as ::core::ffi::c_uint {
            0 => {
                semsg_multiline(
                    b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
                    b"Lua callback:\n%s\0".as_ptr() as *const ::core::ffi::c_char,
                    error,
                );
            }
            1 => {
                semsg_multiline(
                    b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
                    b"Luv thread:\n%s\0".as_ptr() as *const ::core::ffi::c_char,
                    error,
                );
            }
            2 => {
                semsg_multiline(
                    b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
                    b"Luv callback, thread:\n%s\0".as_ptr() as *const ::core::ffi::c_char,
                    error,
                );
            }
            _ => {}
        }
        xfree(error as *mut ::core::ffi::c_void);
    }
}
