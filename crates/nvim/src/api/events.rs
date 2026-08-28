//! The two API calls a *client* makes of nvim rather than the other way
//! round: an async error report, and the terminal's answer to a query the UI
//! sent.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::api::private::helpers::{ERROR_INIT, Reported, api_typename};
use crate::api::private::validate::api_err_exp;
use crate::autocmd::do_termresponse_autocmd;
use crate::eval::vars::set_vim_var_string;
use crate::log::{LOGLVL_ERR, logmsg_c};
use crate::memory::strequal;
use crate::types::{Error, Integer, Object, String_0, Vv, kObjectTypeString, uint64_t};

/// Log the failure a client reported for a request it had sent us.
///
/// # Safety
/// `msg` must point at its own bytes.
pub unsafe fn nvim_error_event(channel_id: uint64_t, _type_0: Integer, msg: String_0) {
    // `msg` is the caller's, per this function's contract, and it is
    // NUL-terminated wherever it is not empty -- the RPC decoder terminates
    // every string it builds.
    let text = if msg.is_empty() {
        c"".as_ptr()
    } else {
        msg.data().cast_const()
    };
    logmsg_c!(
        LOGLVL_ERR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        c"nvim_error_event".as_ptr(),
        44 as ::core::ffi::c_int,
        true,
        c"async error on channel %ld: %s".as_ptr(),
        channel_id,
        text,
    );
}

/// Take delivery of a terminal event the UI forwarded. Only `termresponse` is
/// understood; anything else is ignored, so that a newer UI can send events
/// this build has never heard of.
///
/// # Safety
/// `event` and `value` must own their bytes.
pub unsafe fn nvim_ui_term_event(
    _channel_id: uint64_t,
    event: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    // SAFETY: `event` is the caller's and NUL-terminated, as the RPC decoder
    // leaves every string it builds.
    if !unsafe { strequal(c"termresponse".as_ptr(), event.data()) } {
        return Ok(());
    }
    if value.type_0 != kObjectTypeString {
        let name = c"termresponse".as_ptr();
        let (want, got) = (api_typename(kObjectTypeString), api_typename(value.type_0));
        // SAFETY: `err` is this frame's own; the type names are static.
        unsafe { api_err_exp(&raw mut err, name, want, got) };
        return Err(err);
    }
    // SAFETY: the tag says the payload is the string, and it is the caller's.
    let termresponse: String_0 = unsafe { value.data.string };
    let (text, len) = (termresponse.data(), termresponse.len().cast_signed());
    // SAFETY: `termresponse` is that string, live for `len` bytes.
    unsafe {
        set_vim_var_string(Vv::Termresponse, text, len);
        do_termresponse_autocmd(termresponse);
    }
    ().reported(err)
}
