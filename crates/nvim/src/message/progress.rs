//! Progress messages: `nvim_echo`'s `progress` kind.
//!
//! A progress message carries an id and a status, replaces its previous self
//! in the history rather than appending
//! ([`crate::message::msg_hist_add_multihl`]), and fires the
//! `Progress` autocommand ([`do_autocmd_progress`]).

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::builders::{ArrayBuf, DictBuf, static_cstring};
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

    if !unsafe { (*msg_data).title }.is_empty() {
        let title = HlMessageChunk {
            text: unsafe { copy_string((*msg_data).title, ptr::null_mut()) },
            hl_id: unsafe { status_hl_id((*msg_data).status.data()) },
        };
        unsafe { hl_msg_push(&mut updated, title) };
        let separator = HlMessageChunk {
            text: unsafe { cstr_to_string(c": ".as_ptr()) },
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
            text: unsafe { copy_string(chunk.text, ptr::null_mut()) },
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
        is_set__echo_opts_: 0,
        err: false,
        verbose: false,
        _truncate: false,
        kind: static_cstring(c"progress"),
        id: Object::string(unsafe { cstr_as_string(id) }),
        // Not `static_cstring(c"")`: upstream leaves this field zeroed,
        // so `title.data` is null rather than a pointer to "".
        title: String_0::from_raw_parts(ptr::null_mut(), 0),
        status: unsafe { cstr_as_string(status) },
        percent: 0,
        source: static_cstring(c"nvim"),
        data: Dict {
            size: 0,
            capacity: 0,
            items: ptr::null_mut(),
        },
    };

    // Under ext_messages the UI keeps the untruncated text, so history
    // gets the original either way; on a grid it gets what fits.
    if hist && (!trunc || ui_has(kUIMessages)) {
        unsafe { msg_hist_add(s, -1, 0) };
    }
    if trunc {
        s = unsafe { msg_may_trunc(false, s) };
    }

    let mut chunk = ArrayBuf::<2>::new();
    chunk.push(Object::string(unsafe { cstr_as_string(s) }));
    chunk.push(Object::integer(hl_id.into()));
    let mut chunks = ArrayBuf::<1>::new();
    chunks.push(chunk.object());

    // Nothing here can report: the chunks are strings and the options
    // are this frame's. The message is freed if one ever does.
    if let Err(mut e) = unsafe { nvim_echo(chunks.array(), false, &raw mut opts) } {
        e.clear();
    }
    unsafe { ui_flush() };
    s
}

/// Fire the `Progress` autocommand for a progress message.
///
/// # Safety
/// `msg` must be a valid message and `msg_data` null or a valid data block.
pub unsafe fn do_autocmd_progress(msg_id: Object, msg: HlMessage, msg_data: *mut MessageData) {
    if !has_event(EVENT_PROGRESS) {
        return;
    }

    // The chunk strings are borrowed, not copied: the autocommand runs
    // before this returns, and `messages` is freed at the end of it.
    let mut messages = EMPTY_ARRAY;
    for i in 0..msg.size {
        unsafe { array_push(&mut messages, Object::string((*msg.items.add(i)).text)) };
    }

    let mut data = DictBuf::<7>::new();
    data.insert(c"id", msg_id);
    data.insert(c"text", Object::array(messages));
    if !msg_data.is_null() {
        data.insert(c"percent", Object::integer(unsafe { (*msg_data).percent }));
        data.insert(c"source", Object::string(unsafe { (*msg_data).source }));
        data.insert(c"status", Object::string(unsafe { (*msg_data).status }));
        data.insert(c"title", Object::string(unsafe { (*msg_data).title }));
        data.insert(c"data", Object::dict(unsafe { (*msg_data).data }));
    }

    // The autocommand pattern is the message's source, so an autocommand
    // can match one producer's progress.
    let pattern = if !msg_data.is_null() && !unsafe { (*msg_data).source }.is_empty() {
        unsafe { (*msg_data).source }.data()
    } else {
        c"".as_ptr().cast_mut()
    };
    let mut event_data = data.object();
    // No file name, no buffer and no `:autocmd` argument block: the pattern
    // and the data are the whole of what this event carries.
    let payload = &raw mut event_data;
    let group = AUGROUP_ALL as c_int;
    let no_fname = ptr::null_mut();
    let no_buf = ptr::null_mut();
    let no_eap = ptr::null_mut();
    let fired = EVENT_PROGRESS;
    unsafe {
        apply_autocmds_group(
            fired, pattern, no_fname, true, group, no_buf, no_eap, payload,
        )
    };

    unsafe { xfree(messages.items.cast()) };
}
