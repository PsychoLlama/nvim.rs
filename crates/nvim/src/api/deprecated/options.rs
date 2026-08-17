//! The pre-`nvim_get_option_value` option accessors.
//!
//! `get_option_from` and `set_option_to` are the shared implementations --
//! they resolve a name against the global, buffer or window scope and convert
//! between `OptVal` and the api's Object -- and the seven entry points differ
//! only in which scope they fix and whether they read or write.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_get_option_info(
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    unsafe {
        return get_vimoption(
            name,
            OPT_GLOBAL as ::core::ffi::c_int,
            curbuf.get(),
            curwin.get(),
            arena,
            err,
        );
    }
}

pub unsafe extern "C" fn nvim_set_option(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    unsafe {
        set_option_to(channel_id, NULL, kOptScopeGlobal, name, value, err);
    }
}

pub unsafe extern "C" fn nvim_get_option(mut name: String_0, mut err: *mut Error) -> Object {
    unsafe {
        return get_option_from(NULL, kOptScopeGlobal, name, err);
    }
}

pub unsafe extern "C" fn nvim_buf_get_option(
    mut buffer: Buffer,
    mut name: String_0,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
        if buf.is_null() {
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        return get_option_from(buf as *mut ::core::ffi::c_void, kOptScopeBuf, name, err);
    }
}

pub unsafe extern "C" fn nvim_buf_set_option(
    mut channel_id: uint64_t,
    mut buffer: Buffer,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    unsafe {
        let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
        if buf.is_null() {
            return;
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
}

pub unsafe extern "C" fn nvim_win_get_option(
    mut window: Window,
    mut name: String_0,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let mut win: *mut win_T = find_window_by_handle(window, err);
        if win.is_null() {
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        return get_option_from(win as *mut ::core::ffi::c_void, kOptScopeWin, name, err);
    }
}

pub unsafe extern "C" fn nvim_win_set_option(
    mut channel_id: uint64_t,
    mut window: Window,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    unsafe {
        let mut win: *mut win_T = find_window_by_handle(window, err);
        if win.is_null() {
            return;
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
}

unsafe extern "C" fn get_option_from(
    mut from: *mut ::core::ffi::c_void,
    mut scope: OptScope,
    mut name: String_0,
    mut err: *mut Error,
) -> Object {
    unsafe {
        if !(name.size > 0 as size_t) {
            api_err_invalid(
                err,
                c"option name".as_ptr(),
                c"<empty>".as_ptr(),
                0 as int64_t,
                true,
            );
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        let mut opt_idx: OptIndex = find_option(name.data);
        if !(opt_idx as ::core::ffi::c_int != kOptInvalid as ::core::ffi::c_int) {
            api_err_invalid(err, c"option name".as_ptr(), name.data, 0 as int64_t, true);
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        let mut value: OptVal = OptVal {
            type_0: kOptValTypeNil,
            data: OptValData { boolean: kFalse },
        };
        if option_has_scope(opt_idx, scope) {
            value = get_option_value_for(
                opt_idx,
                if scope as ::core::ffi::c_uint
                    == kOptScopeGlobal as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    OPT_GLOBAL as ::core::ffi::c_int
                } else {
                    OPT_LOCAL as ::core::ffi::c_int
                },
                scope,
                from,
                err,
            );
            if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                return object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                };
            }
        }
        if !(value.type_0 as ::core::ffi::c_int != kOptValTypeNil as ::core::ffi::c_int) {
            api_err_invalid(err, c"option name".as_ptr(), name.data, 0 as int64_t, true);
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        return optval_as_object(value);
    }
}

unsafe extern "C" fn set_option_to(
    mut channel_id: uint64_t,
    mut to: *mut ::core::ffi::c_void,
    mut scope: OptScope,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    unsafe {
        if !(name.size > 0 as size_t) {
            api_err_invalid(
                err,
                c"option name".as_ptr(),
                c"<empty>".as_ptr(),
                0 as int64_t,
                true,
            );
            return;
        }
        let mut opt_idx: OptIndex = find_option(name.data);
        if !(opt_idx as ::core::ffi::c_int != kOptInvalid as ::core::ffi::c_int) {
            api_err_invalid(err, c"option name".as_ptr(), name.data, 0 as int64_t, true);
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
        let opt_flags: ::core::ffi::c_int = if scope as ::core::ffi::c_uint
            == kOptScopeWin as ::core::ffi::c_int as ::core::ffi::c_uint
            && !option_has_scope(opt_idx, kOptScopeGlobal)
        {
            0 as ::core::ffi::c_int
        } else if scope as ::core::ffi::c_uint
            == kOptScopeGlobal as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            OPT_GLOBAL as ::core::ffi::c_int
        } else {
            OPT_LOCAL as ::core::ffi::c_int
        };
        let save_current_sctx: sctx_T = api_set_sctx(channel_id);
        set_option_value_for(name.data, opt_idx, optval, opt_flags, scope, to, err);
        current_sctx.set(save_current_sctx);
    }
}
