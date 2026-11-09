//! The `ext_messages` emitters.
//!
//! With `ext_messages` set the message text is not drawn at all: it is
//! accumulated into highlight-coloured chunks ([`msg_ext_emit_chunk`]) and
//! handed to the UI as a `msg_show` event ([`msg_ext_ui_flush`]), which then
//! decides where to put it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use core::ffi::c_int;

/// Start a new message of kind `msg_kind`, flushing whatever preceded it.
///
/// The kind is **copied**: it is read again when the message is flushed or
/// enters the history, which can be long after the caller's string is gone.
pub fn msg_ext_set_kind(msg_kind: &CStr) {
    msg_ext_set_kind_opt(Some(msg_kind));
}

/// [`msg_ext_set_kind`] for the two callers that may have no kind at all.
///
/// "No kind" is [`ApiString::NULL`], which is a *different* state from the
/// empty kind: `emsg_core` asks `MSG_EXT_KIND.is_null()` to decide whether to
/// stamp a message as a warning, so the absence has to survive the round
/// trip.
pub(crate) fn msg_ext_set_kind_opt(msg_kind: Option<&CStr>) {
    // Flush before setting the kind, so the previous message is emitted
    // under the kind it was written with.
    msg_ext_ui_flush();
    MSG_EXT_KIND.set(msg_kind.map_or(ApiString::NULL, ApiString::from_cstr));
    // An appended message continues the previous one's column run.
    if !MSG_EXT_APPEND.get() {
        REDIR_COL.set(0);
    }
}

/// Mark the next message as continuing the last one rather than replacing it.
pub fn msg_ext_set_append(append: bool) {
    msg_ext_ui_flush();
    MSG_EXT_APPEND.set(append);
}

/// Record what caused the next message, for a UI that wants to group by it.
pub fn msg_ext_set_trigger(trigger: &'static CStr) {
    msg_ext_ui_flush();
    MSG_EXT_TRIGGER.set(Some(trigger));
}

/// Close off the run of text accumulated under one highlight.
///
/// Each chunk is `[attr, text, hl_id]`, which is what the `msg_show` UI event
/// carries.
pub(crate) fn msg_ext_emit_chunk() {
    if MSG_EXT_CHUNKS.with(Option::is_none) {
        // The first chunk of the session starts the column over, as the
        // null-pointer check upstream's `msg_ext_init_chunks` guards did.
        msg_ext_init_chunks();
    }
    if MSG_EXT_LAST_ATTR.get() == -1 {
        return;
    }
    // The accumulated text moves out, leaving the buffer empty.
    let accumulated = MSG_EXT_LAST_CHUNK.take();
    let mut chunk = Vec::with_capacity(3);

    chunk.push(Object::integer(MSG_EXT_LAST_ATTR.get().into()));
    MSG_EXT_LAST_ATTR.set(-1);
    let text = ApiString::from_bytes(&accumulated);
    chunk.push(Object::string(text));
    chunk.push(Object::integer(MSG_EXT_LAST_HL_ID.get().into()));
    MSG_EXT_CHUNKS.update(|chunks| {
        chunks
            .as_mut()
            .expect("the check above installed an array")
            .push(Object::array(chunk));
    });
}

/// Start a fresh chunk array, handing the old one to the caller.
///
/// `None` is upstream's null pointer: the array does not exist until the
/// first chunk is emitted, and resetting the column is what that first
/// emission did.
pub(crate) fn msg_ext_init_chunks() -> Array {
    MSG_COL.set(0);
    MSG_EXT_CHUNKS
        .replace(Some(Vec::new()))
        .map(Array::from)
        .unwrap_or_default()
}

/// Emit everything accumulated so far as one `msg_show` event.
///
/// Without `ext_messages` this only clears the pending kind: the text went to
/// the grid as it was written.
pub fn msg_ext_ui_flush() {
    if !ui_has(UI_MESSAGES) {
        MSG_EXT_KIND.set(ApiString::NULL);
        return;
    }
    if MSG_EXT_SKIP_FLUSH.get() {
        return;
    }

    msg_ext_emit_chunk();
    if MSG_EXT_CHUNKS.with(|chunks| chunks.as_ref().is_none_or(|chunks| chunks.is_empty())) {
        return;
    }

    let mut chunks = msg_ext_init_chunks();
    let to_ui_history = MSG_EXT_HISTORY.get();
    // The UI is sent a copy only when this message is also kept here; when
    // the UI takes the history the array moves out whole.
    let shown = if to_ui_history {
        core::mem::take(&mut chunks)
    } else {
        chunks.clone()
    };
    ui_call_msg_show(
        // SAFETY: both are null or NUL-terminated protocol names.
        MSG_EXT_KIND.with(ApiString::clone),
        shown,
        MSG_EXT_OVERWRITE.get(),
        to_ui_history,
        MSG_EXT_APPEND.get(),
        MSG_EXT_ID.with(Object::clone),
        MSG_EXT_TRIGGER
            .get()
            .map_or_else(ApiString::default, |trigger| {
                ApiString::from_bytes(trigger.to_bytes())
            }),
    );

    if !to_ui_history {
        // Not going to the UI's history, so keep it in ours -- as a
        // temporary entry, which the next message displaces.  The chunk
        // arrays are unwrapped rather than copied: the strings move.
        let moved = chunks.into_iter().map(|entry| {
            let chunk = entry
                .into_array()
                .expect("a chunk this module emitted is an array");
            // `msg_ext_emit_chunk` pushed [attr, text, hl_id] in that order.
            let hl_id = chunk[2]
                .as_integer()
                .expect("a chunk's third element is its highlight id");
            let mut chunk = chunk.into_vec();
            HlMessageChunk {
                text: chunk[1]
                    .take()
                    .into_string()
                    .expect("a chunk's second element is its text"),
                hl_id: c_int::try_from(hl_id).expect("this module only emits c_int ids"),
            }
        });
        msg_hist_add_chunks(moved, true);
    }

    MSG_EXT_OVERWRITE.set(false);
    MSG_EXT_HISTORY.set(false);
    MSG_EXT_APPEND.set(false);
    MSG_EXT_KIND.set(ApiString::NULL);
    // Only claim the next id if nothing else took it in the meantime. An id
    // the caller supplied is a `String`, not an `Integer`, and never matches.
    if MSG_EXT_ID.with(|id| id.as_integer()) == Some(MSG_ID_NEXT.get()) {
        MSG_ID_NEXT.set(MSG_ID_NEXT.get() + 1);
    }
    MSG_EXT_ID.set(Object::integer(MSG_ID_NEXT.get()));
}

/// Emit the pending showmode/showcmd/ruler text as its own event.
pub fn msg_ext_flush_showmode() {
    // One trailing empty event after the mode text goes away, so the UI
    // knows to clear what it drew.
    let pending = MSG_EXT_LAST_ATTR.get() != -1;
    if ui_has(UI_MESSAGES) && (pending || SHOWMODE_CLEAR_PENDING.get()) {
        SHOWMODE_CLEAR_PENDING.set(pending);
        msg_ext_emit_chunk();
        ui_call_msg_showmode(msg_ext_init_chunks());
    }
}
