//! The `ext_messages` emitters.
//!
//! With `ext_messages` set the message text is not drawn at all: it is
//! accumulated into highlight-coloured chunks ([`msg_ext_emit_chunk`]) and
//! handed to the UI as a `msg_show` event ([`msg_ext_ui_flush`]), which then
//! decides where to put it.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use core::ffi::{c_char, c_int};
use core::ptr;

/// Start a new message of kind `msg_kind`, flushing whatever preceded it.
///
/// # Safety
/// `msg_kind` must be null or a C string that outlives the message -- the
/// kind is stored by pointer, not copied.
pub unsafe fn msg_ext_set_kind(msg_kind: *const c_char) {
    // Flush before setting the kind, so the previous message is emitted
    // under the kind it was written with.
    unsafe { msg_ext_ui_flush() };
    msg_ext_kind.set(msg_kind);
    // An appended message continues the previous one's column run.
    if !msg_ext_append.get() {
        redir_col.set(0);
    }
}

/// Mark the next message as continuing the last one rather than replacing it.
///
/// # Safety
/// Only that the emitter statics are in a consistent state.
pub unsafe fn msg_ext_set_append(append: bool) {
    unsafe { msg_ext_ui_flush() };
    msg_ext_append.set(append);
}

/// Record what caused the next message, for a UI that wants to group by it.
///
/// # Safety
/// As [`msg_ext_set_kind`]: `trigger` is stored by pointer.
pub unsafe fn msg_ext_set_trigger(trigger: *const c_char) {
    unsafe { msg_ext_ui_flush() };
    msg_ext_trigger.set(trigger);
}

/// Close off the run of text accumulated under one highlight.
///
/// Each chunk is `[attr, text, hl_id]`, which is what the `msg_show` UI event
/// carries.
///
/// # Safety
/// Only that the emitter statics are in a consistent state.
pub(crate) unsafe fn msg_ext_emit_chunk() {
    if msg_ext_chunks.get().is_null() {
        // SAFETY: the caller's obligation, as documented above.
        unsafe { msg_ext_init_chunks() };
    }
    if msg_ext_last_attr.get() == -1 {
        return;
    }
    // The accumulated text moves out, leaving the buffer empty.
    let accumulated = msg_ext_last_chunk.take();
    let mut chunk = EMPTY_ARRAY;

    // SAFETY: `accumulated` is the chunk's own bytes, and `msg_ext_chunks` is
    // non-null by the test above.
    unsafe { array_push(&mut chunk, Object::integer(msg_ext_last_attr.get().into())) };
    msg_ext_last_attr.set(-1);
    let text = unsafe { cbuf_to_string(accumulated.as_ptr().cast::<c_char>(), accumulated.len()) };
    unsafe { array_push(&mut chunk, Object::string(text)) };
    unsafe { array_push(&mut chunk, Object::integer(msg_ext_last_hl_id.get().into())) };
    unsafe { array_push(&mut *msg_ext_chunks.get(), Object::array(chunk)) };
}

/// Start a fresh chunk array, handing the old one to the caller to dispose of.
///
/// # Safety
/// Only that the emitter statics are in a consistent state.
pub(crate) unsafe fn msg_ext_init_chunks() -> *mut Array {
    let tofree = msg_ext_chunks.get();
    msg_ext_chunks.set(unsafe { xcalloc(1, ::core::mem::size_of::<Array>()) }.cast());
    msg_col.set(0);
    tofree
}

/// Emit everything accumulated so far as one `msg_show` event.
///
/// Without `ext_messages` this only clears the pending kind: the text went to
/// the grid as it was written.
///
/// # Safety
/// Only that the emitter statics are in a consistent state.
pub unsafe fn msg_ext_ui_flush() {
    if !ui_has(kUIMessages) {
        msg_ext_kind.set(ptr::null());
        return;
    }
    if msg_ext_skip_flush.get() {
        return;
    }

    unsafe { msg_ext_emit_chunk() };
    if unsafe { (*msg_ext_chunks.get()).size } == 0 {
        return;
    }

    let tofree = unsafe { msg_ext_init_chunks() };
    ui_call_msg_show(
        unsafe { cstr_as_string(msg_ext_kind.get()) },
        unsafe { *tofree },
        msg_ext_overwrite.get(),
        msg_ext_history.get(),
        msg_ext_append.get(),
        msg_ext_id.get(),
        unsafe { cstr_as_string(msg_ext_trigger.get()) },
    );

    if msg_ext_history.get() {
        // The UI owns the history copy; ours is redundant.
        unsafe { api_free_array(*tofree) };
    } else {
        // Not going to the UI's history, so keep it in ours -- as a
        // temporary entry, which the next message displaces.  The chunk
        // arrays are unwrapped rather than copied: the strings move.
        let mut msg = EMPTY_HL_MESSAGE;
        for i in 0..unsafe { (*tofree).size } {
            let chunk = unsafe { *(*tofree).items.add(i) }
                .as_array()
                .expect("a chunk this module emitted is an array")
                .items;
            // `msg_ext_emit_chunk` pushed [attr, text, hl_id] in that order.
            let moved = HlMessageChunk {
                text: unsafe { *chunk.add(1) }
                    .as_string()
                    .expect("a chunk's second element is its text"),
                hl_id: unsafe { *chunk.add(2) }
                    .as_integer()
                    .expect("a chunk's third element is its highlight id")
                    as c_int,
            };
            unsafe { hl_msg_push(&mut msg, moved) };
            unsafe { xfree(chunk.cast()) };
        }
        unsafe { xfree((*tofree).items.cast()) };
        unsafe { msg_hist_add_multihl(msg, true, ptr::null_mut()) };
    }
    unsafe { xfree(tofree.cast()) };

    msg_ext_overwrite.set(false);
    msg_ext_history.set(false);
    msg_ext_append.set(false);
    msg_ext_kind.set(ptr::null());
    // Only claim the next id if nothing else took it in the meantime. An id
    // the caller supplied is a `String`, not an `Integer`, and never matches.
    if msg_ext_id.with(|id| id.as_integer()) == Some(msg_id_next.get()) {
        msg_id_next.set(msg_id_next.get() + 1);
    }
    msg_ext_id.set(Object::integer(msg_id_next.get()));
}

/// Emit the pending showmode/showcmd/ruler text as its own event.
///
/// # Safety
/// Only that the emitter statics are in a consistent state.
pub unsafe fn msg_ext_flush_showmode() {
    // One trailing empty event after the mode text goes away, so the UI
    // knows to clear what it drew.
    static clear: GlobalCell<bool> = GlobalCell::new(false);
    let pending = msg_ext_last_attr.get() != -1;
    if ui_has(kUIMessages) && (pending || clear.get()) {
        clear.set(pending);
        unsafe { msg_ext_emit_chunk() };
        let tofree = unsafe { msg_ext_init_chunks() };
        ui_call_msg_showmode(unsafe { *tofree });
        unsafe { api_free_array(*tofree) };
        unsafe { xfree(tofree.cast()) };
    }
}
