//! The pre-`nvim_get_option_value` option accessors.
//!
//! `get_option_from` and `set_option_to` are the shared implementations --
//! they resolve a name against the global, buffer or window scope and convert
//! between `OptVal` and the api's Object -- and the seven entry points differ
//! only in which scope they fix and whether they read or write.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported};
use crate::option::NIL_OPTVAL;
use crate::types::OptionSetFlags;
use core::ffi::CStr;

pub unsafe fn nvim_get_option_info(name: String_0, arena: *mut Arena) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        return get_vimoption(
            name,
            OptionSetFlags::GLOBAL,
            curbuf.get(),
            curwin.get(),
            arena,
            err,
        )
        .reported(error);
    }
}

pub unsafe fn nvim_set_option(
    channel_id: uint64_t,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        set_option_to(channel_id, NULL, kOptScopeGlobal, name, value, err);
    }
    ().reported(error)
}

pub unsafe fn nvim_get_option(name: String_0) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        return get_option_from(NULL, kOptScopeGlobal, name, err).reported(error);
    }
}

pub unsafe fn nvim_buf_get_option(buffer: Buffer, name: String_0) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
        if buf.is_null() {
            return NIL.reported(error);
        }
        return get_option_from(buf as *mut ::core::ffi::c_void, kOptScopeBuf, name, err)
            .reported(error);
    }
}

pub unsafe fn nvim_buf_set_option(
    channel_id: uint64_t,
    buffer: Buffer,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
        if buf.is_null() {
            return ().reported(error);
        }
        set_option_to(
            channel_id,
            buf as *mut ::core::ffi::c_void,
            kOptScopeBuf,
            name,
            value,
            err,
        );
    }
    ().reported(error)
}

pub unsafe fn nvim_win_get_option(window: Window, name: String_0) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut win: *mut win_T = find_window_by_handle(window, err);
        if win.is_null() {
            return NIL.reported(error);
        }
        return get_option_from(win as *mut ::core::ffi::c_void, kOptScopeWin, name, err)
            .reported(error);
    }
}

pub unsafe fn nvim_win_set_option(
    channel_id: uint64_t,
    window: Window,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut win: *mut win_T = find_window_by_handle(window, err);
        if win.is_null() {
            return ().reported(error);
        }
        set_option_to(
            channel_id,
            win as *mut ::core::ffi::c_void,
            kOptScopeWin,
            name,
            value,
            err,
        );
    }
    ().reported(error)
}

unsafe fn get_option_from(
    mut from: *mut ::core::ffi::c_void,
    mut scope: OptScope,
    mut name: String_0,
    mut err: *mut Error,
) -> Object {
    unsafe {
        if !(name.len() > 0 as size_t) {
            api_err_invalid(
                err,
                c"option name".as_ptr(),
                c"<empty>".as_ptr(),
                0 as int64_t,
                true,
            );
            return NIL;
        }
        let opt_name = name.data();
        let mut opt_idx: OptIndex = find_option(CStr::from_ptr(opt_name));
        if !(opt_idx as ::core::ffi::c_int != kOptInvalid as ::core::ffi::c_int) {
            api_err_invalid(err, c"option name".as_ptr(), opt_name, 0 as int64_t, true);
            return NIL;
        }
        let mut value: OptVal = NIL_OPTVAL;
        if option_has_scope(opt_idx, scope) {
            value = get_option_value_for(
                opt_idx,
                if scope as ::core::ffi::c_uint
                    == kOptScopeGlobal as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    OptionSetFlags::GLOBAL
                } else {
                    OptionSetFlags::LOCAL
                },
                scope,
                from,
                err,
            );
            if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                return NIL;
            }
        }
        if !(value.type_0 as ::core::ffi::c_int != kOptValTypeNil as ::core::ffi::c_int) {
            api_err_invalid(err, c"option name".as_ptr(), opt_name, 0 as int64_t, true);
            return NIL;
        }
        return optval_as_object(value);
    }
}

unsafe fn set_option_to(
    mut channel_id: uint64_t,
    mut to: *mut ::core::ffi::c_void,
    mut scope: OptScope,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    unsafe {
        if !(name.len() > 0 as size_t) {
            api_err_invalid(
                err,
                c"option name".as_ptr(),
                c"<empty>".as_ptr(),
                0 as int64_t,
                true,
            );
            return;
        }
        let opt_name = name.data();
        let mut opt_idx: OptIndex = find_option(CStr::from_ptr(opt_name));
        if !(opt_idx as ::core::ffi::c_int != kOptInvalid as ::core::ffi::c_int) {
            api_err_invalid(err, c"option name".as_ptr(), opt_name, 0 as int64_t, true);
            return;
        }
        let Some(optval) = object_as_optval(value) else {
            api_err_exp(
                err,
                c"value".as_ptr(),
                c"valid option type".as_ptr(),
                api_typename(value.type_0),
            );
            return;
        };
        let opt_flags: OptionSetFlags = if scope as ::core::ffi::c_uint
            == kOptScopeWin as ::core::ffi::c_int as ::core::ffi::c_uint
            && !option_has_scope(opt_idx, kOptScopeGlobal)
        {
            OptionSetFlags::NONE
        } else if scope as ::core::ffi::c_uint
            == kOptScopeGlobal as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            OptionSetFlags::GLOBAL
        } else {
            OptionSetFlags::LOCAL
        };
        let save_current_sctx: sctx_T = api_set_sctx(channel_id);
        set_option_value_for(opt_name, opt_idx, optval, opt_flags, scope, to, err);
        current_sctx.set(save_current_sctx);
    }
}
