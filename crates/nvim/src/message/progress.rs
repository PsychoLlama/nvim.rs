//! Progress messages: `nvim_echo`'s `progress` kind.
//!
//! A progress message carries an id and a status, replaces its previous self
//! in the history rather than appending
//! ([`crate::message::msg_hist_add_multihl`]), and fires the
//! `Progress` autocommand ([`do_autocmd_progress`]).

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::autocmd::fire_autocmds_for;
use crate::types::builders::{ArrayBuf, DictBuf};
use core::ffi::{CStr, c_int};

/// The highlight group a progress message's title takes from its status.
///
/// An unrecognised status -- and a null one -- leaves the title unhighlighted.
fn status_hl_id(status: &String_0) -> c_int {
    let group: &CStr = match status.as_bytes() {
        _ if status.is_null() => return 0,
        b"success" => c"OkMsg",
        b"failed" => c"ErrorMsg",
        b"running" => c"MoreMsg",
        b"cancel" => c"WarningMsg",
        _ => return 0,
    };
    syn_check_group(group.to_bytes())
}

/// `hl_msg` prefixed with the "title: percent% " a progress message displays
/// as, or `None` when there is neither a title nor a percentage.
pub(crate) fn format_progress_message(
    hl_msg: &[HlMessageChunk],
    msg_data: &MessageData,
) -> Option<HlMessage> {
    let mut updated = HlMessage::new();

    if !msg_data.title.is_empty() {
        updated.push(HlMessageChunk {
            text: msg_data.title.clone(),
            hl_id: status_hl_id(&msg_data.status),
        });
        updated.push(HlMessageChunk {
            text: String_0::from_cstr(c": "),
            hl_id: 0,
        });
    }

    if msg_data.percent > 0 {
        // Upstream formats into a ten-byte buffer, so a percentage past
        // seven digits is cut there.
        let text = crate::formatted!(10, b"%3ld%% ", msg_data.percent);
        updated.push(HlMessageChunk {
            text: String_0::from_bytes(&text),
            hl_id: syn_check_group(b"WarningMsg"),
        });
    }

    if updated.is_empty() {
        return None;
    }
    updated.extend_from_slice(hl_msg);
    Some(updated)
}

/// Show `text` as a progress message from `id`, in state `status`.
///
/// Answers the text that was actually shown, which `trunc` may have cut at
/// the head.
pub fn msg_progress<'t>(
    text: &'t CStr,
    id: &CStr,
    status: &CStr,
    hl_id: c_int,
    hist: bool,
    trunc: bool,
) -> Cow<'t, [u8]> {
    let mut opts = KeyDict_echo_opts {
        kind: Some(String_0::from_cstr(c"progress")),
        id: Some(Object::string(String_0::from_cstr(id))),
        // Not `static_cstring(c"")`: upstream leaves this field zeroed,
        // so `title.data` is null rather than a pointer to "".
        title: Some(String_0::NULL),
        status: Some(String_0::from_cstr(status)),
        source: Some(String_0::from_cstr(c"nvim")),
        ..KeyDict_echo_opts::default()
    };

    // Under ext_messages the UI keeps the untruncated text, so history
    // gets the original either way; on a grid it gets what fits.
    if hist && (!trunc || ui_has(kUIMessages)) {
        msg_hist_add(text.to_bytes(), 0);
    }
    let shown = if trunc {
        fit_head(text.to_bytes(), false)
    } else {
        Cow::Borrowed(text.to_bytes())
    };

    let mut chunk = ArrayBuf::<2>::new();
    chunk.push(Object::string(String_0::from_bytes(&shown)));
    chunk.push(Object::integer(hl_id.into()));
    let mut chunks = ArrayBuf::<1>::new();
    chunks.push(chunk.object());

    // Nothing here can report: the chunks are strings and the options
    // are this frame's. The message is freed if one ever does.
    if let Err(mut e) = nvim_echo(chunks.array(), false, &mut opts) {
        e.clear();
    }
    ui_flush();
    shown
}

/// Fire the `Progress` autocommand for a progress message.
pub fn do_autocmd_progress(msg_id: Object, msg: &[HlMessageChunk], msg_data: Option<&MessageData>) {
    if !has_event(AutoEvent::Progress) {
        return;
    }

    // The event carries copies: the message and its data block outlive the
    // autocommand and go on being the caller's.
    let mut messages = Array::with_capacity(msg.len());
    for chunk in msg {
        messages.push(Object::string(chunk.text.clone()));
    }

    let mut data = DictBuf::<7>::new();
    data.insert(c"id", msg_id);
    data.insert(c"text", Object::array(messages));
    if let Some(msg_data) = msg_data {
        data.insert(c"percent", Object::integer(msg_data.percent));
        data.insert(c"source", Object::string(msg_data.source.clone()));
        data.insert(c"status", Object::string(msg_data.status.clone()));
        data.insert(c"title", Object::string(msg_data.title.clone()));
        data.insert(c"data", Object::dict(msg_data.data.clone()));
    }

    // The autocommand pattern is the message's source, so an autocommand
    // can match one producer's progress.
    let pattern = match msg_data {
        Some(msg_data) if !msg_data.source.is_empty() => msg_data.source.as_cstr(),
        _ => c"",
    };
    let mut event_data = data.object();
    // No file name, no buffer and no `:autocmd` argument block: the pattern
    // and the data are the whole of what this event carries.
    let data = Some(&mut event_data);
    fire_autocmds_for(AutoEvent::Progress, Some(pattern), None, true, None, data);
}
