//! The `RPC:` debug log, one line per message in each direction.
//!
//! Nothing is written unless a debug log level is in force, so the formatting
//! is left to `logmsg` rather than done at the call sites.

use crate::src::nvim::log::logmsg_c;
use core::ffi::{CStr, c_char};
use core::ptr;

use crate::src::nvim::types::{uint32_t, uint64_t};

/// Fixed-width columns, so consecutive lines align. The trailing spaces are
/// part of the text. Nested to stay out of the flat namespace the unit-test
/// header generator collects top-level constants into.
mod column {
    use core::ffi::{CStr, c_int};

    pub const LOGLVL_DBG: c_int = 1;

    pub const REQUEST: &CStr = c"[request]  ";
    pub const RESPONSE: &CStr = c"[response] ";
    pub const NOTIFY: &CStr = c"[notify]   ";
    pub const ERROR: &CStr = c"[error]    ";

    /// Sent by this editor.
    pub const SEND: &CStr = c"->";
    /// Received from the peer.
    pub const RECV: &CStr = c"<-";

    /// The prefix every trace line shares. Passed as `logmsg`'s context rather
    /// than baked into the format, which is how the log file groups them.
    pub const TAG: &CStr = c"RPC: ";
}

pub use column::{RECV, SEND};

/// Traces a request or a notification.
///
/// `req_id` is `None` for a notification, which carries no id and is logged in
/// the `[notify]` column instead.
pub unsafe fn log_call(
    dir: &CStr,
    channel_id: uint64_t,
    req_id: Option<uint32_t>,
    name: *const c_char,
) {
    if let Some(id) = req_id {
        logmsg_c!(
            column::LOGLVL_DBG,
            column::TAG.as_ptr(),
            ptr::null(),
            -1,
            false,
            c"%s %lu: %s id=%u: %s\n".as_ptr(),
            dir.as_ptr(),
            channel_id,
            column::REQUEST.as_ptr(),
            id,
            name,
        );
    } else {
        logmsg_c!(
            column::LOGLVL_DBG,
            column::TAG.as_ptr(),
            ptr::null(),
            -1,
            false,
            c"%s %lu: %s %s\n".as_ptr(),
            dir.as_ptr(),
            channel_id,
            column::NOTIFY.as_ptr(),
            name,
        );
    }
}

/// Traces a response. `errored` picks the `[error]` column over `[response]`.
pub unsafe fn log_response(dir: &CStr, channel_id: uint64_t, errored: bool, req_id: uint32_t) {
    let kind = if errored {
        column::ERROR
    } else {
        column::RESPONSE
    };
    logmsg_c!(
        column::LOGLVL_DBG,
        column::TAG.as_ptr(),
        ptr::null(),
        -1,
        false,
        c"%s %lu: %s id=%u\n".as_ptr(),
        dir.as_ptr(),
        channel_id,
        kind.as_ptr(),
        req_id,
    );
}
