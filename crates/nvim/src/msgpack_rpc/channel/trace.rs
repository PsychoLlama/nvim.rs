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

use crate::log::logmsg_tagged;
use core::ffi::{CStr, c_char};

use crate::message_fmt::{c_str, msg_cstr};
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
}

pub use column::{RECV, SEND};

/// Traces a request or a notification.
///
/// `req_id` is `None` for a notification, which carries no id and is logged in
/// the `[notify]` column instead.
///
/// # Safety
/// `name` is a NUL-terminated string, or null — an unresolved handler leaves
/// null behind, and `c_str` prints it as `[NULL]`. glibc's `%s` wrote
/// `(null)` there; the text of one debug line moves, nothing else.
pub unsafe fn log_call(
    dir: &CStr,
    channel_id: uint64_t,
    req_id: Option<uint32_t>,
    name: *const c_char,
) {
    let dir = msg_cstr(dir);
    // SAFETY: the caller's promise -- `name` is NUL-terminated or null.
    let name = unsafe { c_str(name) };
    // A traced line is always `<direction> <channel>: <column>` and then
    // whatever the kind adds. The closure runs only when a debug log is
    // open, so an editor that is not tracing pays nothing to build it.
    match req_id {
        Some(id) => {
            let kind = msg_cstr(column::REQUEST);
            logmsg_tagged!(
                column::LOGLVL_DBG,
                column::TAG,
                false,
                "{dir} {channel_id}: {kind} id={id}: {name}\n"
            )
        }
        None => {
            let kind = msg_cstr(column::NOTIFY);
            logmsg_tagged!(
                column::LOGLVL_DBG,
                column::TAG,
                false,
                "{dir} {channel_id}: {kind} {name}\n"
            )
        }
    };
}

/// Traces a response. `errored` picks the `[error]` column over `[response]`.
pub fn log_response(dir: &CStr, channel_id: uint64_t, errored: bool, req_id: uint32_t) {
    let dir = msg_cstr(dir);
    let kind = msg_cstr(if errored {
        column::ERROR
    } else {
        column::RESPONSE
    });
    logmsg_tagged!(
        column::LOGLVL_DBG,
        column::TAG,
        false,
        "{dir} {channel_id}: {kind} id={req_id}\n"
    );
}
