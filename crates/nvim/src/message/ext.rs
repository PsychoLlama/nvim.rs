//! The `ext_messages` emitters.
//!
//! With `ext_messages` set the message text is not drawn at all: it is
//! accumulated into highlight-coloured chunks ([`msg_ext_emit_chunk`]) and
//! handed to the UI as a `msg_show` event ([`msg_ext_ui_flush`]), which then
//! decides where to put it.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use core::ffi::{c_char, c_int};
use core::ptr;

/// Start a new message of kind `msg_kind`, flushing whatever preceded it.
///
/// The kind is **copied**: it is read again when the message is flushed or
/// enters the history, which can be long after the caller's string is gone.
///
/// # Safety
/// `msg_kind` must be null or a valid C string.
pub unsafe fn msg_ext_set_kind(msg_kind: *const c_char) {
    // Flush before setting the kind, so the previous message is emitted
    // under the kind it was written with.
    msg_ext_ui_flush();
    // SAFETY: the caller's promise.
    msg_ext_kind.set(unsafe { cstr_to_string(msg_kind) });
    // An appended message continues the previous one's column run.
    if !msg_ext_append.get() {
        redir_col.set(0);
    }
}

/// Mark the next message as continuing the last one rather than replacing it.
pub fn msg_ext_set_append(append: bool) {
    msg_ext_ui_flush();
    msg_ext_append.set(append);
}

/// Record what caused the next message, for a UI that wants to group by it.
pub fn msg_ext_set_trigger(trigger: &'static CStr) {
    msg_ext_ui_flush();
    msg_ext_trigger.set(Some(trigger));
}

/// Close off the run of text accumulated under one highlight.
///
/// Each chunk is `[attr, text, hl_id]`, which is what the `msg_show` UI event
/// carries.
pub(crate) fn msg_ext_emit_chunk() {
    if msg_ext_chunks.with(Option::is_none) {
        // The first chunk of the session starts the column over, as the
        // null-pointer check upstream's `msg_ext_init_chunks` guards did.
        msg_ext_init_chunks();
    }
    if msg_ext_last_attr.get() == -1 {
        return;
    }
    // The accumulated text moves out, leaving the buffer empty.
    let accumulated = msg_ext_last_chunk.take();
    let mut chunk = Array::with_capacity(3);

    chunk.push(Object::integer(msg_ext_last_attr.get().into()));
    msg_ext_last_attr.set(-1);
    // SAFETY: `accumulated` is the chunk's own bytes.
    let text = unsafe { cbuf_to_string(accumulated.as_ptr().cast::<c_char>(), accumulated.len()) };
    chunk.push(Object::string(text));
    chunk.push(Object::integer(msg_ext_last_hl_id.get().into()));
    msg_ext_chunks.with_mut(|chunks| {
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
    msg_col.set(0);
    msg_ext_chunks
        .with_mut(|chunks| chunks.replace(Array::EMPTY))
        .unwrap_or(Array::EMPTY)
}

/// Emit everything accumulated so far as one `msg_show` event.
///
/// Without `ext_messages` this only clears the pending kind: the text went to
/// the grid as it was written.
pub fn msg_ext_ui_flush() {
    if !ui_has(kUIMessages) {
        msg_ext_kind.set(String_0::NULL);
        return;
    }
    if msg_ext_skip_flush.get() {
        return;
    }

    msg_ext_emit_chunk();
    if msg_ext_chunks.with(|chunks| chunks.as_ref().is_none_or(|chunks| chunks.is_empty())) {
        return;
    }

    let mut chunks = msg_ext_init_chunks();
    let to_ui_history = msg_ext_history.get();
    // The UI is sent a copy only when this message is also kept here; when
    // the UI takes the history the array moves out whole.
    let shown = if to_ui_history {
        core::mem::take(&mut chunks)
    } else {
        chunks.clone()
    };
    ui_call_msg_show(
        // SAFETY: both are null or NUL-terminated protocol names.
        msg_ext_kind.with(String_0::clone),
        shown,
        msg_ext_overwrite.get(),
        to_ui_history,
        msg_ext_append.get(),
        msg_ext_id.with(Object::clone),
        msg_ext_trigger
            .get()
            .map_or_else(String_0::default, |trigger| {
                String_0::from_bytes(trigger.to_bytes())
            }),
    );

    if !to_ui_history {
        // Not going to the UI's history, so keep it in ours -- as a
        // temporary entry, which the next message displaces.  The chunk
        // arrays are unwrapped rather than copied: the strings move.
        let mut msg = EMPTY_HL_MESSAGE;
        for entry in chunks {
            let chunk = entry
                .into_array()
                .expect("a chunk this module emitted is an array");
            // `msg_ext_emit_chunk` pushed [attr, text, hl_id] in that order.
            let hl_id = chunk[2]
                .as_integer()
                .expect("a chunk's third element is its highlight id");
            let mut chunk = chunk.into_vec();
            let moved = HlMessageChunk {
                text: chunk[1]
                    .take()
                    .into_string()
                    .expect("a chunk's second element is its text"),
                hl_id: c_int::try_from(hl_id).expect("this module only emits c_int ids"),
            };
            // SAFETY: `msg` started empty and is only pushed to here.
            unsafe { hl_msg_push(&mut msg, moved) };
        }
        // SAFETY: `msg` is this frame's, and the history takes it over.
        unsafe { msg_hist_add_multihl(msg, true, ptr::null_mut()) };
    }

    msg_ext_overwrite.set(false);
    msg_ext_history.set(false);
    msg_ext_append.set(false);
    msg_ext_kind.set(String_0::NULL);
    // Only claim the next id if nothing else took it in the meantime. An id
    // the caller supplied is a `String`, not an `Integer`, and never matches.
    if msg_ext_id.with(|id| id.as_integer()) == Some(msg_id_next.get()) {
        msg_id_next.set(msg_id_next.get() + 1);
    }
    msg_ext_id.set(Object::integer(msg_id_next.get()));
}

/// Emit the pending showmode/showcmd/ruler text as its own event.
pub fn msg_ext_flush_showmode() {
    // One trailing empty event after the mode text goes away, so the UI
    // knows to clear what it drew.
    static clear: GlobalCell<bool> = GlobalCell::new(false);
    let pending = msg_ext_last_attr.get() != -1;
    if ui_has(kUIMessages) && (pending || clear.get()) {
        clear.set(pending);
        msg_ext_emit_chunk();
        ui_call_msg_showmode(msg_ext_init_chunks());
    }
}
