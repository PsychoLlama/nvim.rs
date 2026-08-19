//! Progress messages: `nvim_echo`'s `progress` kind.
//!
//! A progress message carries an id and a status, replaces its previous self
//! in the history rather than appending
//! ([`crate::message::msg_hist_add_multihl`]), and fires the
//! `Progress` autocommand ([`do_autocmd_progress`]).

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::api_clear_error;
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
    let group: &CStr = unsafe {
        if status.is_null() {
            return 0;
        } else if strequal(status, c"success".as_ptr()) {
            c"OkMsg"
        } else if strequal(status, c"failed".as_ptr()) {
            c"ErrorMsg"
        } else if strequal(status, c"running".as_ptr()) {
            c"MoreMsg"
        } else if strequal(status, c"cancel".as_ptr()) {
            c"WarningMsg"
        } else {
            return 0;
        }
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
    unsafe {
        let mut updated = EMPTY_HL_MESSAGE;

        if (*msg_data).title.size != 0 {
            hl_msg_push(
                &mut updated,
                HlMessageChunk {
                    text: copy_string((*msg_data).title, ptr::null_mut()),
                    hl_id: status_hl_id((*msg_data).status.data),
                },
            );
            hl_msg_push(
                &mut updated,
                HlMessageChunk {
                    text: cstr_to_string(c": ".as_ptr()),
                    hl_id: 0,
                },
            );
        }

        if (*msg_data).percent > 0 {
            let mut percent_buf = [0 as c_char; 10];
            vim_snprintf(
                percent_buf.as_mut_ptr(),
                percent_buf.len(),
                c"%3ld%% ".as_ptr(),
                (*msg_data).percent as c_long,
            );
            hl_msg_push(
                &mut updated,
                HlMessageChunk {
                    text: cstr_to_string(percent_buf.as_ptr()),
                    hl_id: syn_check_group(c"WarningMsg".as_ptr(), c"WarningMsg".count_bytes()),
                },
            );
        }

        if updated.size == 0 {
            return hl_msg;
        }
        for i in 0..hl_msg.size {
            let chunk = *hl_msg.items.add(i);
            hl_msg_push(
                &mut updated,
                HlMessageChunk {
                    text: copy_string(chunk.text, ptr::null_mut()),
                    hl_id: chunk.hl_id,
                },
            );
        }
        updated
    }
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
    unsafe {
        let mut opts = KeyDict_echo_opts {
            is_set__echo_opts_: 0,
            err: false,
            verbose: false,
            _truncate: false,
            kind: static_cstring(c"progress"),
            id: Object::string(cstr_as_string(id)),
            // Not `static_cstring(c"")`: upstream leaves this field zeroed,
            // so `title.data` is null rather than a pointer to "".
            title: String_0 {
                data: ptr::null_mut(),
                size: 0,
            },
            status: cstr_as_string(status),
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
            msg_hist_add(s, -1, 0);
        }
        if trunc {
            s = msg_may_trunc(false, s);
        }

        let mut chunk = ArrayBuf::<2>::new();
        chunk.push(Object::string(cstr_as_string(s)));
        chunk.push(Object::integer(hl_id.into()));
        let mut chunks = ArrayBuf::<1>::new();
        chunks.push(chunk.object());

        // Nothing here can report: the chunks are strings and the options
        // are this frame's. The message is freed if one ever does.
        if let Err(mut e) = nvim_echo(chunks.array(), false, &raw mut opts) {
            api_clear_error(&raw mut e);
        }
        ui_flush();
        s
    }
}

/// Fire the `Progress` autocommand for a progress message.
///
/// # Safety
/// `msg` must be a valid message and `msg_data` null or a valid data block.
pub unsafe fn do_autocmd_progress(msg_id: Object, msg: HlMessage, msg_data: *mut MessageData) {
    unsafe {
        if !has_event(EVENT_PROGRESS) {
            return;
        }

        // The chunk strings are borrowed, not copied: the autocommand runs
        // before this returns, and `messages` is freed at the end of it.
        let mut messages = EMPTY_ARRAY;
        for i in 0..msg.size {
            array_push(&mut messages, Object::string((*msg.items.add(i)).text));
        }

        let mut data = DictBuf::<7>::new();
        data.insert(c"id", msg_id);
        data.insert(c"text", Object::array(messages));
        if !msg_data.is_null() {
            data.insert(c"percent", Object::integer((*msg_data).percent));
            data.insert(c"source", Object::string((*msg_data).source));
            data.insert(c"status", Object::string((*msg_data).status));
            data.insert(c"title", Object::string((*msg_data).title));
            data.insert(c"data", Object::dict((*msg_data).data));
        }

        // The autocommand pattern is the message's source, so an autocommand
        // can match one producer's progress.
        let pattern = if !msg_data.is_null() && (*msg_data).source.size > 0 {
            (*msg_data).source.data
        } else {
            c"".as_ptr().cast_mut()
        };
        let mut event_data = data.object();
        apply_autocmds_group(
            EVENT_PROGRESS,
            pattern,
            ptr::null_mut(),
            true,
            AUGROUP_ALL as c_int,
            ptr::null_mut(),
            ptr::null_mut(),
            &raw mut event_data,
        );

        xfree(messages.items.cast());
    }
}
