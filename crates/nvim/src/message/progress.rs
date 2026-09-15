//! Progress messages: `nvim_echo`'s `progress` kind.
//!
//! A progress message carries an id and a status, replaces its previous self
//! in the history rather than appending
//! ([`crate::message::msg_hist_add_multihl`]), and fires the
//! `Progress` autocommand ([`do_autocmd_progress`]).

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
use crate::cstr;
use crate::types::builders::{ArrayBuf, DictBuf};
use core::ffi::{CStr, c_char, c_int, c_long};
use core::ptr;

/// The highlight group a progress message's title takes from its status.
///
/// An unrecognised status -- and a null one -- leaves the title unhighlighted.
///
/// # Safety
/// `status` must be a null pointer or a valid C string.
unsafe fn status_hl_id(status: *const c_char) -> c_int {
    if status.is_null() {
        return 0;
    }
    // SAFETY: a valid C string, per this function's contract.
    let is = |name: &CStr| unsafe { strequal(status, name.as_ptr()) };
    let group: &CStr = if is(c"success") {
        c"OkMsg"
    } else if is(c"failed") {
        c"ErrorMsg"
    } else if is(c"running") {
        c"MoreMsg"
    } else if is(c"cancel") {
        c"WarningMsg"
    } else {
        return 0;
    };
    unsafe { syn_check_group(group.as_ptr(), group.count_bytes()) }
}

/// Prefix `hl_msg` with the "title: percent% " a progress message displays as.
///
/// Answers `hl_msg` itself when there is neither a title nor a percentage, so
/// the caller can tell whether it now owns a second message: the returned
/// chunks are copies, the argument's are not.
///
/// # Safety
/// `msg_data` must point at a valid message data block, and `hl_msg` must own
/// its chunks.
pub(crate) unsafe fn format_progress_message(
    hl_msg: HlMessage,
    msg_data: *mut MessageData,
) -> HlMessage {
    let mut updated = EMPTY_HL_MESSAGE;

    if !unsafe { (*msg_data).title.is_empty() } {
        let title = HlMessageChunk {
            // SAFETY: the caller's message data.
            text: unsafe { (*msg_data).title.clone() },
            hl_id: unsafe { status_hl_id((*msg_data).status.data()) },
        };
        unsafe { hl_msg_push(&mut updated, title) };
        let separator = HlMessageChunk {
            text: String_0::from_cstr(c": "),
            hl_id: 0,
        };
        unsafe { hl_msg_push(&mut updated, separator) };
    }

    if unsafe { (*msg_data).percent } > 0 {
        let mut percent_buf = [0 as c_char; 10];
        let out = percent_buf.as_mut_ptr();
        let cap = percent_buf.len();
        let percent = unsafe { (*msg_data).percent } as c_long;
        unsafe { vim_snprintf(out, cap, c"%3ld%% ".as_ptr(), percent) };
        let warning = c"WarningMsg";
        let chunk = HlMessageChunk {
            text: unsafe { cstr_to_string(percent_buf.as_ptr()) },
            hl_id: unsafe { syn_check_group(warning.as_ptr(), warning.count_bytes()) },
        };
        unsafe { hl_msg_push(&mut updated, chunk) };
    }

    if updated.size == 0 {
        return hl_msg;
    }
    for i in 0..hl_msg.size {
        let chunk = unsafe { (*hl_msg.items.add(i)).clone() };
        let copy = HlMessageChunk {
            text: chunk.text.clone(),
            hl_id: chunk.hl_id,
        };
        unsafe { hl_msg_push(&mut updated, copy) };
    }
    updated
}

/// Show `s` as a progress message from `id`, in state `status`.
///
/// Answers the string that was actually shown, which `trunc` may have moved
/// past the head of `s`.
///
/// # Safety
/// Every pointer argument must be null or a valid C string, and `s` must
/// remain valid until the message has been emitted.
pub unsafe fn msg_progress(
    mut s: *mut c_char,
    id: *mut c_char,
    status: *mut c_char,
    hl_id: c_int,
    hist: bool,
    trunc: bool,
) -> *mut c_char {
    let mut opts = KeyDict_echo_opts {
        kind: Some(String_0::from_cstr(c"progress")),
        id: Some(Object::string(unsafe { cstr_to_string(id) })),
        // Not `static_cstring(c"")`: upstream leaves this field zeroed,
        // so `title.data` is null rather than a pointer to "".
        title: Some(String_0::NULL),
        status: Some(unsafe { cstr_to_string(status) }),
        source: Some(String_0::from_cstr(c"nvim")),
        ..KeyDict_echo_opts::default()
    };

    // Under ext_messages the UI keeps the untruncated text, so history
    // gets the original either way; on a grid it gets what fits.
    if hist && (!trunc || ui_has(kUIMessages)) {
        // SAFETY: the progress message is NUL-terminated.
        msg_hist_add(unsafe { cstr::bytes_at(s) }, 0);
    }
    if trunc {
        s = unsafe { msg_may_trunc(false, s) };
    }

    let mut chunk = ArrayBuf::<2>::new();
    chunk.push(Object::string(unsafe { cstr_to_string(s) }));
    chunk.push(Object::integer(hl_id.into()));
    let mut chunks = ArrayBuf::<1>::new();
    chunks.push(chunk.object());

    // Nothing here can report: the chunks are strings and the options
    // are this frame's. The message is freed if one ever does.
    if let Err(mut e) = unsafe { nvim_echo(chunks.array(), false, &raw mut opts) } {
        e.clear();
    }
    ui_flush();
    s
}

/// Fire the `Progress` autocommand for a progress message.
///
/// # Safety
/// `msg` must be a valid message and `msg_data` null or a valid data block.
pub unsafe fn do_autocmd_progress(msg_id: Object, msg: HlMessage, msg_data: *mut MessageData) {
    if !has_event(AutoEvent::Progress) {
        return;
    }

    // The event carries copies: the message and its data block outlive the
    // autocommand and go on being the caller's.
    let mut messages = Array::with_capacity(msg.size);
    for i in 0..msg.size {
        // SAFETY: `i` is below `size`, so the chunk is inside `items`.
        messages.push(Object::string(unsafe { (*msg.items.add(i)).text.clone() }));
    }

    let mut data = DictBuf::<7>::new();
    data.insert(c"id", msg_id);
    data.insert(c"text", Object::array(messages));
    if !msg_data.is_null() {
        // SAFETY: the caller's data block, live for the call.
        unsafe {
            data.insert(c"percent", Object::integer((*msg_data).percent));
            data.insert(c"source", Object::string((*msg_data).source.clone()));
            data.insert(c"status", Object::string((*msg_data).status.clone()));
            data.insert(c"title", Object::string((*msg_data).title.clone()));
            data.insert(c"data", Object::dict((*msg_data).data.clone()));
        }
    }

    // The autocommand pattern is the message's source, so an autocommand
    // can match one producer's progress.
    // SAFETY: as above.
    let pattern = if !msg_data.is_null() && !unsafe { (*msg_data).source.is_empty() } {
        unsafe { (*msg_data).source.data() }
    } else {
        c"".as_ptr().cast_mut()
    };
    let mut event_data = data.object();
    // No file name, no buffer and no `:autocmd` argument block: the pattern
    // and the data are the whole of what this event carries.
    let payload = &raw mut event_data;
    let group = AUGROUP_ALL as c_int;
    let no_fname = ptr::null_mut();
    let no_buf = None;
    let fired = AutoEvent::Progress;
    unsafe { apply_autocmds_group(fired, pattern, no_fname, true, group, no_buf, None, payload) };
}
