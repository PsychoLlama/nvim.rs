//! `nvim_echo()`: printing a chunked, highlighted message.
//!
//! One function, because a message is an array of (text, highlight) chunks
//! that has to be rendered as a unit under the caller's `history`, `err`
//! and `verbose` options, with the message state saved and restored around
//! it.

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
use crate::api::private::helpers::Reported;
use crate::api::private::validate::{err_expected, err_out_of_range, err_required};
use crate::api_error;
use crate::guard::Suppress;
use crate::message::EMPTY_HL_MESSAGE;
use crate::message_fmt::c_str;
use crate::winlayer::Live;
use core::ffi::{CStr, c_char};

/// The decoded `nvim_echo` keyset, whose caller has promised it outlives the
/// value.
type EchoOpts = Live<KeyDict_echo_opts>;

/// The `status` key's four spellings, which only a `kind='progress'` message
/// may carry.
const PROGRESS_STATUS: [&CStr; 4] = [c"success", c"failed", c"running", c"cancel"];

/// Print `chunks` as one message, `history` to record it in `:messages`.
///
/// # Safety
/// `chunks` must name its own items and `opts` must be the caller's decoded
/// keyset, whose strings are NUL-terminated.
pub unsafe fn nvim_echo(
    chunks: Array,
    history: Boolean,
    opts: *mut KeyDict_echo_opts,
) -> Result<Object, Error> {
    let mut error = Error::none();
    // SAFETY: the caller's keyset, live for the whole call.
    let opts = unsafe { EchoOpts::new(opts) };
    // The keys, with every one the caller left out reading as its default:
    // no text, no percentage, no dictionary.
    let no_string = String_0::NULL;
    let (err, verbose, truncate) = (
        opts.err.unwrap_or(false),
        opts.verbose.unwrap_or(false),
        opts._truncate.unwrap_or(false),
    );
    let (title, status, source) = (
        opts.title.unwrap_or(no_string),
        opts.status.unwrap_or(no_string),
        opts.source.unwrap_or(no_string),
    );
    let percent = opts.percent.unwrap_or(0);
    let data = opts.data.unwrap_or(ApiDict::EMPTY);
    let given_id = opts.id.unwrap_or(Object::Nil);
    let mut id = Object::integer(-1);
    let mut hl_msg = EMPTY_HL_MESSAGE;
    // SAFETY: the caller's chunk array, and `hl_msg` is this frame's own.
    if let Err(e) = unsafe { parse_hl_msg(&mut hl_msg, chunks, err) } {
        // SAFETY: the message this frame just built and nothing else owns.
        unsafe { hl_msg_free(hl_msg) };
        return Err(e);
    }

    let mut kind: *mut c_char = opts.kind.unwrap_or(no_string).data();
    if verbose {
        // SAFETY: paired with the `verbose_leave` below.
        unsafe { verbose_enter() };
    } else if kind.is_null() {
        kind = if err {
            c"echoerr".as_ptr().cast_mut()
        } else if history {
            c"echomsg".as_ptr().cast_mut()
        } else {
            c"echo".as_ptr().cast_mut()
        };
    }
    // SAFETY: `kind` is a literal above, or the keyset's NUL-terminated
    // string.
    let is_progress = unsafe { strequal(kind, c"progress".as_ptr()) };
    let mut needs_clear = !history;

    // The progress keys belong to `kind='progress'` and to nothing else, and
    // each of them has its own range.
    let has_progress_keys = !status.is_empty()
        || !title.is_empty()
        || percent != 0
        || data.size != 0
        || !source.is_empty();
    let echo_id = given_id.as_integer();
    // SAFETY: the keyset's strings are NUL-terminated, and `error` is this
    // frame's own slot.
    let rejected = unsafe {
        if !is_progress && has_progress_keys {
            let kind = c_str(kind);
            error = api_error!(
                kErrorTypeValidation,
                "Conflict: title/source/status/percent/data not allowed with kind='{kind}'"
            );
            true
        } else if is_progress && !status_named(status) {
            let names = c"success|failed|running|cancel";
            // SAFETY: the keyset's string names its own NUL-terminated bytes.
            let got = crate::cstr::at_opt(status.data());
            error = err_expected(c"status", names, got);
            true
        } else if is_progress && !(0..=100).contains(&percent) {
            error = err_out_of_range(c"percent");
            true
        } else if is_progress && source.is_empty() {
            error = err_required(c"opts.source");
            true
        } else if let Some(id) = echo_id.filter(|&id| !msg_id_exists(id)) {
            error = api_error!(kErrorTypeValidation, "Invalid 'id': {id}");
            true
        } else {
            false
        }
    };

    if !rejected {
        let mut msg_data = MessageData {
            source,
            percent,
            title,
            status,
            data,
        };
        let save_nwr = need_wait_return.get();
        let save_lines_left = lines_left.get();
        let save_msg_didany = msg_didany.get();
        let no_prompt = truncate.then(Suppress::wait_return);
        if truncate {
            lines_left.set(0 as ::core::ffi::c_int);
            msg_didany.set(true);
            msg_no_more.set(true);
        }
        // SAFETY: `msg_data` and `needs_clear` are this frame's own, and the
        // message is the one built above.
        id = unsafe {
            msg_multihl(
                given_id,
                hl_msg.clone(),
                kind,
                history,
                err,
                &raw mut msg_data,
                &raw mut needs_clear,
            )
        };
        if truncate {
            msg_no_more.set(false);
            msg_didany.set(save_msg_didany);
            lines_left.set(save_lines_left);
            drop(no_prompt);
            need_wait_return.set(save_nwr);
        }
        if verbose {
            // SAFETY: paired with the `verbose_enter` above.
            unsafe {
                verbose_leave();
                verbose_stop();
            }
        }
        if is_progress {
            // SAFETY: `msg_data` is this frame's own, live for the call.
            unsafe { do_autocmd_progress(id, hl_msg.clone(), &raw mut msg_data) };
        }
        if !needs_clear {
            return id.reported(error);
        }
    }
    // SAFETY: the message this frame built and nothing else owns.
    unsafe { hl_msg_free(hl_msg) };
    id.reported(error)
}

/// Whether `status` is one of the four a progress message may carry. The
/// empty string is not one of them.
///
/// # Safety
/// `status` must be NUL-terminated.
unsafe fn status_named(status: String_0) -> bool {
    PROGRESS_STATUS
        .iter()
        // SAFETY: the caller's promise.
        .any(|name| unsafe { strequal(status.data(), name.as_ptr()) })
}
