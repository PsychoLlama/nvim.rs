//! The pre-`nvim_get_option_value` option accessors.
//!
//! `get_option_from` and `set_option_to` are the shared implementations --
//! they resolve a name against the global, buffer or window scope and convert
//! between `OptVal` and the api's Object -- and the seven entry points differ
//! only in which scope they fix and whether they read or write.

#![forbid(unsafe_code)]
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
use crate::option::OptionTarget;
use crate::types::OptionSetFlags;
use crate::winlayer::{Buf, Win};

pub fn nvim_get_option_info(name: String_0) -> Result<ApiDict, Error> {
    let (buf, win) = (Buf::current(), Win::current());
    get_vimoption(name, OptionSetFlags::GLOBAL, buf, win)
}

pub fn nvim_set_option(channel_id: uint64_t, name: String_0, value: Object) -> Result<(), Error> {
    set_option_to(channel_id, None, name, value)
}

pub fn nvim_get_option(name: String_0) -> Result<Object, Error> {
    get_option_from(None, name)
}

pub fn nvim_buf_get_option(buffer: BufferHandle, name: String_0) -> Result<Object, Error> {
    let Some(buf) = find_buffer_by_handle(buffer)? else {
        return Ok(Object::Nil);
    };
    get_option_from(Some(OptionTarget::Buf(buf)), name)
}

pub fn nvim_buf_set_option(
    channel_id: uint64_t,
    buffer: BufferHandle,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let Some(buf) = find_buffer_by_handle(buffer)? else {
        return Ok(());
    };
    set_option_to(channel_id, Some(OptionTarget::Buf(buf)), name, value)
}

pub fn nvim_win_get_option(window: WindowHandle, name: String_0) -> Result<Object, Error> {
    let Some(win) = find_window_by_handle(window)? else {
        return Ok(Object::Nil);
    };
    get_option_from(Some(OptionTarget::Win(win)), name)
}

pub fn nvim_win_set_option(
    channel_id: uint64_t,
    window: WindowHandle,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let Some(win) = find_window_by_handle(window)? else {
        return Ok(());
    };
    set_option_to(channel_id, Some(OptionTarget::Win(win)), name, value)
}

/// The option `name` names, or the error saying why it is not the name of
/// one.
///
/// `name` is borrowed: the caller goes on to report it. Taking it by value
/// freed it here while the caller still held a pointer to its bytes -- an
/// error naming a scope the option lacks read freed memory.
fn resolve_option(name: &String_0) -> Result<OptIndex, Error> {
    if name.is_empty() {
        return Err(err_bad_value(c"option name", c"<empty>"));
    }
    let opt_idx: OptIndex = find_option(name.as_cstr());
    if opt_idx == kOptInvalid as OptIndex {
        return Err(err_bad_value(c"option name", name.as_cstr()));
    }
    Ok(opt_idx)
}

/// The value of `name` as `from` sees it; `None` is the global scope.
fn get_option_from(from: Option<OptionTarget>, name: String_0) -> Result<Object, Error> {
    let opt_idx = resolve_option(&name)?;
    let scope = OptionTarget::scope_of(from);
    let mut value: OptVal = OptVal::Nil;
    if option_has_scope(opt_idx, scope) {
        let flags = if scope == kOptScopeGlobal {
            OptionSetFlags::GLOBAL
        } else {
            OptionSetFlags::LOCAL
        };
        value = get_option_value_for(opt_idx, flags, from)?;
    }
    // An option the scope does not have reads as the unset value, which is
    // the same answer as a name that is not an option's at all.
    if value.is_nil() {
        return Err(err_bad_value(c"option name", name.as_cstr()));
    }
    Ok(optval_as_object(value))
}

/// Set `name` to `value` on `to`; `None` is the global scope.
fn set_option_to(
    channel_id: uint64_t,
    to: Option<OptionTarget>,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let opt_idx = resolve_option(&name)?;
    let scope = OptionTarget::scope_of(to);
    let got = api_typename(value.kind());
    let Some(optval) = object_as_optval(value) else {
        let want = c"valid option type";
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
    set_option_value_for(name.as_bytes(), opt_idx, optval, opt_flags, to)
}
