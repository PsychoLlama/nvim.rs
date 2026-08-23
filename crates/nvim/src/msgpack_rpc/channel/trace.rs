#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The `RPC:` debug log, one line per message in each direction.
//!
//! Nothing is written unless a debug log level is in force, so the formatting
//! is left to `logmsg` rather than done at the call sites.

use crate::log::logmsg_c;
use core::ffi::{CStr, c_char};
use core::ptr;

use crate::types::{uint32_t, uint64_t};

/// Fixed-width columns, so consecutive lines align. The trailing spaces are
/// part of the text. Nested to stay out of the flat namespace the unit-test
/// header generator collects top-level constants into.
mod column {
    use core::ffi::{CStr, c_int};

    pub(super) const LOGLVL_DBG: c_int = 1;

    pub(super) const REQUEST: &CStr = c"[request]  ";
    pub(super) const RESPONSE: &CStr = c"[response] ";
    pub(super) const NOTIFY: &CStr = c"[notify]   ";
    pub(super) const ERROR: &CStr = c"[error]    ";

    /// Sent by this editor.
    pub const SEND: &CStr = c"->";
    /// Received from the peer.
    pub const RECV: &CStr = c"<-";

    /// The prefix every trace line shares. Passed as `logmsg`'s context rather
    /// than baked into the format, which is how the log file groups them.
    pub(super) const TAG: &CStr = c"RPC: ";

    /// The three line shapes. A traced line is always
    /// `<direction> <channel>: <column>` and then whatever the kind adds.
    pub(super) const FMT_ID_NAME: &CStr = c"%s %lu: %s id=%u: %s\n";
    pub(super) const FMT_NAME: &CStr = c"%s %lu: %s %s\n";
    pub(super) const FMT_ID: &CStr = c"%s %lu: %s id=%u\n";
}

pub use column::{RECV, SEND};

/// Traces a request or a notification.
///
/// `req_id` is `None` for a notification, which carries no id and is logged in
/// the `[notify]` column instead.
///
/// # Safety
/// `name` is a NUL-terminated string, or null — the `%s` verb prints `(null)`
/// for one, which is what an unresolved handler leaves behind.
pub unsafe fn log_call(
    dir: &CStr,
    channel_id: uint64_t,
    req_id: Option<uint32_t>,
    name: *const c_char,
) {
    let (lvl, tag, nofile) = (column::LOGLVL_DBG, column::TAG.as_ptr(), ptr::null());
    let d = dir.as_ptr();
    // SAFETY: the caller's `name`, and format verbs that match the arguments
    // beside them. `logmsg_begin` refuses the line if no log file is open, so
    // nothing here is evaluated against a closed handle.
    unsafe {
        match req_id {
            Some(id) => {
                let k = column::REQUEST.as_ptr();
                let fmt = column::FMT_ID_NAME.as_ptr();
                logmsg_c!(lvl, tag, nofile, -1, false, fmt, d, channel_id, k, id, name)
            }
            None => {
                let k = column::NOTIFY.as_ptr();
                let fmt = column::FMT_NAME.as_ptr();
                logmsg_c!(lvl, tag, nofile, -1, false, fmt, d, channel_id, k, name)
            }
        };
    }
}

/// Traces a response. `errored` picks the `[error]` column over `[response]`.
pub fn log_response(dir: &CStr, channel_id: uint64_t, errored: bool, req_id: uint32_t) {
    let (lvl, tag, nofile) = (column::LOGLVL_DBG, column::TAG.as_ptr(), ptr::null());
    let d = dir.as_ptr();
    let k = if errored {
        column::ERROR.as_ptr()
    } else {
        column::RESPONSE.as_ptr()
    };
    let fmt = column::FMT_ID.as_ptr();
    // SAFETY: every argument is a `&CStr`'s pointer or an integer, and the
    // verbs match them; there is no caller-supplied pointer to get wrong.
    unsafe {
        logmsg_c!(lvl, tag, nofile, -1, false, fmt, d, channel_id, k, req_id);
    }
}
