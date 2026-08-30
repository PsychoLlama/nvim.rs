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

use crate::api::private::helpers::{Reported, api_typename};
use crate::api::private::validate::err_expected;
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
    let fmt = c"async error on channel %ld: %s".as_ptr();
    let (here, no_context) = (c"nvim_error_event".as_ptr(), ::core::ptr::null());
    // SAFETY: the log macro's own operations; every argument outlives the
    // call and matches its verb.
    unsafe {
        logmsg_c!(
            LOGLVL_ERR, no_context, here, 44, true, fmt, channel_id, text
        )
    };
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
    let mut err = Error::none();
    // SAFETY: `event` is the caller's and NUL-terminated, as the RPC decoder
    // leaves every string it builds.
    if !unsafe { strequal(c"termresponse".as_ptr(), event.data()) } {
        return Ok(());
    }
    if value.type_0 != kObjectTypeString {
        let (want, got) = (api_typename(kObjectTypeString), api_typename(value.type_0));
        err = err_expected(c"termresponse", want, Some(got));
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
