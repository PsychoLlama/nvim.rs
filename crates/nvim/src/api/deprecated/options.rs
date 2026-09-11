//! The pre-`nvim_get_option_value` option accessors.
//!
//! `get_option_from` and `set_option_to` are the shared implementations --
//! they resolve a name against the global, buffer or window scope and convert
//! between `OptVal` and the api's Object -- and the seven entry points differ
//! only in which scope they fix and whether they read or write.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::helpers::{find_buffer_by_handle, find_window_by_handle};
use crate::api::private::validate::{err_bad_value, err_expected};
use crate::cstr;
use crate::types::OptionSetFlags;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_void};

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `arena` must point at a live arena, which the memory this
/// answers with is taken from and must outlive.
pub unsafe fn nvim_get_option_info(name: String_0, arena: *mut Arena) -> Result<ApiDict, Error> {
    let (buf, win) = (Buf::current(), Win::current());
    // SAFETY: `name` is the caller's, the two globals name the current
    // buffer and window, and `arena` is the caller's.
    unsafe { get_vimoption(name, OptionSetFlags::GLOBAL, buf, win, arena) }
}

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `value` must be a well-formed API object the caller owns
/// for the call.
pub unsafe fn nvim_set_option(
    channel_id: uint64_t,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    // SAFETY: the global scope names no object, so `NULL` is what it takes.
    unsafe { set_option_to(channel_id, NULL, kOptScopeGlobal, name, value) }
}

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`.
pub unsafe fn nvim_get_option(name: String_0) -> Result<Object, Error> {
    // SAFETY: as `nvim_set_option`.
    unsafe { get_option_from(NULL, kOptScopeGlobal, name) }
}

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`.
pub unsafe fn nvim_buf_get_option(buffer: BufferHandle, name: String_0) -> Result<Object, Error> {
    let Some(buf) = find_buffer_by_handle(buffer)? else {
        return Ok(Object::Nil);
    };
    let from = buf.raw().cast::<c_void>();
    // SAFETY: `from` is that live buffer, which is what `kOptScopeBuf` says
    // it is.
    unsafe { get_option_from(from, kOptScopeBuf, name) }
}

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `value` must be a well-formed API object the caller owns
/// for the call.
pub unsafe fn nvim_buf_set_option(
    channel_id: uint64_t,
    buffer: BufferHandle,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let Some(buf) = find_buffer_by_handle(buffer)? else {
        return Ok(());
    };
    let to = buf.raw().cast::<c_void>();
    // SAFETY: as `nvim_buf_get_option`.
    unsafe { set_option_to(channel_id, to, kOptScopeBuf, name, value) }
}

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`.
pub unsafe fn nvim_win_get_option(window: WindowHandle, name: String_0) -> Result<Object, Error> {
    let Some(win) = find_window_by_handle(window)? else {
        return Ok(Object::Nil);
    };
    let from = win.raw().cast::<c_void>();
    // SAFETY: `from` is that live window, which is what `kOptScopeWin` says
    // it is.
    unsafe { get_option_from(from, kOptScopeWin, name) }
}

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `value` must be a well-formed API object the caller owns
/// for the call.
pub unsafe fn nvim_win_set_option(
    channel_id: uint64_t,
    window: WindowHandle,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let Some(win) = find_window_by_handle(window)? else {
        return Ok(());
    };
    let to = win.raw().cast::<c_void>();
    // SAFETY: as `nvim_win_get_option`.
    unsafe { set_option_to(channel_id, to, kOptScopeWin, name, value) }
}

/// The option `name` names, as its C string and its index, or `None` after
/// answering why it is not the name of one.
///
/// # Safety
/// `name` must name its own bytes.
unsafe fn resolve_option(name: String_0) -> Result<(*const c_char, OptIndex), Error> {
    if name.is_empty() {
        let empty = c"<empty>".as_ptr();
        // SAFETY: the names and values are NUL-terminated strings.
        return Err(err_bad_value(c"option name", unsafe { cstr::at(empty) }));
    }
    let opt_name = name.data();
    // SAFETY: an API string is NUL-terminated.
    let opt_idx: OptIndex = find_option(unsafe { CStr::from_ptr(opt_name) });
    if opt_idx == kOptInvalid as OptIndex {
        // SAFETY: the names and values are NUL-terminated strings.
        return Err(err_bad_value(c"option name", unsafe { cstr::at(opt_name) }));
    }
    Ok((opt_name, opt_idx))
}

/// The value of `name` in `scope`, read out of `from`.
///
/// # Safety
/// `from` must be null, a live buffer or a live window, as `scope` says.
unsafe fn get_option_from(
    from: *mut c_void,
    scope: OptScope,
    name: String_0,
) -> Result<Object, Error> {
    // SAFETY: `name` names its own bytes.
    let (opt_name, opt_idx) = unsafe { resolve_option(name) }?;
    let mut value: OptVal = OptVal::Nil;
    if option_has_scope(opt_idx, scope) {
        let flags = if scope == kOptScopeGlobal {
            OptionSetFlags::GLOBAL
        } else {
            OptionSetFlags::LOCAL
        };
        // SAFETY: the caller's promise about `from`.
        value = unsafe { get_option_value_for(opt_idx, flags, scope, from) }?;
    }
    // An option the scope does not have reads as the unset value, which is
    // the same answer as a name that is not an option's at all.
    if value.is_nil() {
        // SAFETY: the names and values are NUL-terminated strings.
        return Err(err_bad_value(c"option name", unsafe { cstr::at(opt_name) }));
    }
    Ok(optval_as_object(value))
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
) -> Result<(), Error> {
    // SAFETY: `name` names its own bytes.
    let (opt_name, opt_idx) = unsafe { resolve_option(name) }?;
    let Some(optval) = object_as_optval(value) else {
        let want = c"valid option type";
        let got = api_typename(value.kind());
        return Err(err_expected(c"value", want, Some(got)));
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
    // SAFETY: the caller's promise about `to`.
    unsafe { set_option_value_for(opt_name, opt_idx, optval, opt_flags, scope, to) }
}
