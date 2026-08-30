//! `nvim_echo()`: printing a chunked, highlighted message.
//!
//! One function, because a message is an array of (text, highlight) chunks
//! that has to be rendered as a unit under the caller's `history`, `err`
//! and `verbose` options, with the message state saved and restored around
//! it.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported};
use crate::guard::Suppress;
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
    let err = &raw mut error;
    // SAFETY: the caller's keyset, live for the whole call.
    let opts = unsafe { EchoOpts::new(opts) };
    let mut id = Object::integer(-1);
    // SAFETY: the caller's chunk array, and `err` is this frame's own slot.
    let hl_msg: HlMessage = unsafe { parse_hl_msg(chunks, opts.err, err) };
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
    // SAFETY: the keyset's strings are NUL-terminated, and `err` is this
    // frame's own slot.
    let rejected = unsafe {
        if !is_progress && has_progress_keys {
            let fmt = c"Conflict: title/source/status/percent/data not allowed with kind='%s'";
            api_set_error(err, kErrorTypeValidation, fmt.as_ptr(), kind);
            true
        } else if is_progress && !status_named(opts.status) {
            let names = c"success|failed|running|cancel".as_ptr();
            api_err_exp(err, c"status".as_ptr(), names, opts.status.data());
            true
        } else if is_progress && !(0..=100).contains(&opts.percent) {
            let range = c"out of range".as_ptr();
            api_err_invalid(err, c"percent".as_ptr(), range, 0, false);
            true
        } else if is_progress && opts.source.is_empty() {
            api_err_required(err, c"opts.source".as_ptr());
            true
        } else if opts.id.type_0 == kObjectTypeInteger && !msg_id_exists(opts.id.data.integer) {
            let fmt = c"Invalid 'id': %ld".as_ptr();
            api_set_error(err, kErrorTypeValidation, fmt, opts.id.data.integer);
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
