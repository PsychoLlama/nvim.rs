//! `nvim_echo()`: printing a chunked, highlighted message.
//!
//! One function, because a message is an array of (text, highlight) chunks
//! that has to be rendered as a unit under the caller's `history`, `err`
//! and `verbose` options, with the message state saved and restored around
//! it.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported};
use crate::api::private::validate::err_expected;
use crate::api::private::validate::err_invalid_ptr;
use crate::api::private::validate::err_required_ptr;
use crate::api_error;
use crate::guard::Suppress;
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
    let mut error = ERROR_INIT;
    // SAFETY: the caller's keyset, live for the whole call.
    let opts = unsafe { EchoOpts::new(opts) };
    let mut id = Object::integer(-1);
    // SAFETY: the caller's chunk array, and `error` is this frame's own slot.
    let hl_msg: HlMessage = unsafe { parse_hl_msg(chunks, opts.err, &mut error) };
    if error.is_set() {
        // SAFETY: the message this frame just built and nothing else owns.
        unsafe { hl_msg_free(hl_msg) };
        return id.reported(error);
    }

    let mut kind: *mut c_char = opts.kind.data();
    if opts.verbose {
        // SAFETY: paired with the `verbose_leave` below.
        unsafe { verbose_enter() };
    } else if kind.is_null() {
        kind = if opts.err {
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
    let has_progress_keys = !opts.status.is_empty()
        || !opts.title.is_empty()
        || opts.percent != 0
        || opts.data.size != 0
        || !opts.source.is_empty();
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
        } else if is_progress && !status_named(opts.status) {
            let names = c"success|failed|running|cancel";
            // SAFETY: the keyset's string names its own NUL-terminated bytes.
            let got = crate::cstr::at_opt(opts.status.data());
            error = err_expected(c"status", names, got);
            true
        } else if is_progress && !(0..=100).contains(&opts.percent) {
            let range = c"out of range".as_ptr();
            error = err_invalid_ptr(c"percent".as_ptr(), range, 0, false);
            true
        } else if is_progress && opts.source.is_empty() {
            error = err_required_ptr(c"opts.source".as_ptr());
            true
        } else if opts.id.type_0 == kObjectTypeInteger && !msg_id_exists(opts.id.data.integer) {
            let id = opts.id.data.integer;
            error = api_error!(kErrorTypeValidation, "Invalid 'id': {id}");
            true
        } else {
            false
        }
    };

    if !rejected {
        let mut msg_data = MessageData {
            source: opts.source,
            percent: opts.percent,
            title: opts.title,
            status: opts.status,
            data: opts.data,
        };
        let save_nwr = need_wait_return.get();
        let save_lines_left = lines_left.get();
        let save_msg_didany = msg_didany.get();
        let truncate = opts._truncate;
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
                opts.id,
                hl_msg.clone(),
                kind,
                history,
                opts.err,
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
        if opts.verbose {
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
