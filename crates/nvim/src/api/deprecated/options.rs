//! The pre-`nvim_get_option_value` option accessors.
//!
//! `get_option_from` and `set_option_to` are the shared implementations --
//! they resolve a name against the global, buffer or window scope and convert
//! between `OptVal` and the api's Object -- and the seven entry points differ
//! only in which scope they fix and whether they read or write.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported, buffer_by_handle, window_by_handle};
use crate::option::NIL_OPTVAL;
use crate::types::OptionSetFlags;
use core::ffi::{CStr, c_char, c_void};

pub unsafe fn nvim_get_option_info(name: String_0, arena: *mut Arena) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let (buf, win) = (curbuf.get(), curwin.get());
    // SAFETY: `name` is the caller's, the two globals name the current
    // buffer and window, and `arena`/`err` are the caller's and this
    // frame's slot.
    unsafe { get_vimoption(name, OptionSetFlags::GLOBAL, buf, win, arena, err) }.reported(error)
}

pub unsafe fn nvim_set_option(
    channel_id: uint64_t,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    // SAFETY: the global scope names no object, so `NULL` is what it takes.
    unsafe { set_option_to(channel_id, NULL, kOptScopeGlobal, name, value, err) };
    ().reported(error)
}

pub unsafe fn nvim_get_option(name: String_0) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    // SAFETY: as `nvim_set_option`.
    unsafe { get_option_from(NULL, kOptScopeGlobal, name, err) }.reported(error)
}

pub unsafe fn nvim_buf_get_option(buffer: Buffer, name: String_0) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let Some(buf) = buffer_by_handle(buffer, &mut error) else {
        return NIL.reported(error);
    };
    let from = buf.raw().cast::<c_void>();
    // SAFETY: `from` is that live buffer, which is what `kOptScopeBuf` says
    // it is; `error` is this frame's slot.
    unsafe { get_option_from(from, kOptScopeBuf, name, &raw mut error) }.reported(error)
}

pub unsafe fn nvim_buf_set_option(
    channel_id: uint64_t,
    buffer: Buffer,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let Some(buf) = buffer_by_handle(buffer, &mut error) else {
        return ().reported(error);
    };
    let to = buf.raw().cast::<c_void>();
    let err = &raw mut error;
    // SAFETY: as `nvim_buf_get_option`.
    unsafe { set_option_to(channel_id, to, kOptScopeBuf, name, value, err) };
    ().reported(error)
}

pub unsafe fn nvim_win_get_option(window: Window, name: String_0) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let Some(win) = window_by_handle(window, &mut error) else {
        return NIL.reported(error);
    };
    let from = win.raw().cast::<c_void>();
    // SAFETY: `from` is that live window, which is what `kOptScopeWin` says
    // it is; `error` is this frame's slot.
    unsafe { get_option_from(from, kOptScopeWin, name, &raw mut error) }.reported(error)
}

pub unsafe fn nvim_win_set_option(
    channel_id: uint64_t,
    window: Window,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let Some(win) = window_by_handle(window, &mut error) else {
        return ().reported(error);
    };
    let to = win.raw().cast::<c_void>();
    let err = &raw mut error;
    // SAFETY: as `nvim_win_get_option`.
    unsafe { set_option_to(channel_id, to, kOptScopeWin, name, value, err) };
    ().reported(error)
}

/// The option `name` names, as its C string and its index, or `None` after
/// reporting through `err` why it is not the name of one.
///
/// # Safety
/// `err` must be the caller's error slot.
unsafe fn resolve_option(name: String_0, err: *mut Error) -> Option<(*const c_char, OptIndex)> {
    if name.is_empty() {
        let empty = c"<empty>".as_ptr();
        // SAFETY: the caller's promise about `err`; both strings are static.
        unsafe { api_err_invalid(err, c"option name".as_ptr(), empty, 0, true) };
        return None;
    }
    let opt_name = name.data();
    // SAFETY: an API string is NUL-terminated.
    let opt_idx: OptIndex = find_option(unsafe { CStr::from_ptr(opt_name) });
    if opt_idx == kOptInvalid as OptIndex {
        // SAFETY: the caller's promise about `err`; `opt_name` is a C string.
        unsafe { api_err_invalid(err, c"option name".as_ptr(), opt_name, 0, true) };
        return None;
    }
    Some((opt_name, opt_idx))
}

/// The value of `name` in `scope`, read out of `from`.
///
/// # Safety
/// `from` must be null, a live buffer or a live window, as `scope` says, and
/// `err` must be the caller's error slot.
unsafe fn get_option_from(
    from: *mut c_void,
    scope: OptScope,
    name: String_0,
    err: *mut Error,
) -> Object {
    // SAFETY: the caller's promise about `err`.
    let Some((opt_name, opt_idx)) = (unsafe { resolve_option(name, err) }) else {
        return NIL;
    };
    let mut value: OptVal = NIL_OPTVAL;
    if option_has_scope(opt_idx, scope) {
        let flags = if scope == kOptScopeGlobal {
            OptionSetFlags::GLOBAL
        } else {
            OptionSetFlags::LOCAL
        };
        // SAFETY: the caller's promise about `from` and `err`.
        value = unsafe { get_option_value_for(opt_idx, flags, scope, from, err) };
        // SAFETY: as above.
        if unsafe { (*err).kind() } != kErrorTypeNone {
            return NIL;
        }
    }
    // An option the scope does not have reads as the unset value, which is
    // the same answer as a name that is not an option's at all.
    if value.type_0 as ::core::ffi::c_int == kOptValTypeNil as ::core::ffi::c_int {
        // SAFETY: as above; `opt_name` is a C string.
        unsafe { api_err_invalid(err, c"option name".as_ptr(), opt_name, 0, true) };
        return NIL;
    }
    optval_as_object(value)
}

/// Set `name` in `scope` to `value`, on `to`.
///
/// # Safety
/// As [`get_option_from`].
unsafe fn set_option_to(
    channel_id: uint64_t,
    to: *mut c_void,
    scope: OptScope,
    name: String_0,
    value: Object,
    err: *mut Error,
) {
    // SAFETY: the caller's promise about `err`.
    let Some((opt_name, opt_idx)) = (unsafe { resolve_option(name, err) }) else {
        return;
    };
    let Some(optval) = object_as_optval(value) else {
        let want = c"valid option type".as_ptr();
        let got = api_typename(value.type_0);
        // SAFETY: as above; the names are static and `api_typename`'s own.
        unsafe { api_err_exp(err, c"value".as_ptr(), want, got) };
        return;
    };
    // A window-local option with no global half is set locally without the
    // "and globally" that `LOCAL` would otherwise imply.
    let opt_flags: OptionSetFlags =
        if scope == kOptScopeWin && !option_has_scope(opt_idx, kOptScopeGlobal) {
            OptionSetFlags::NONE
        } else if scope == kOptScopeGlobal {
            OptionSetFlags::GLOBAL
        } else {
            OptionSetFlags::LOCAL
        };
    let _sctx = api_set_sctx(channel_id);
    // SAFETY: the caller's promise about `to` and `err`.
    unsafe { set_option_value_for(opt_name, opt_idx, optval, opt_flags, scope, to, err) };
}
