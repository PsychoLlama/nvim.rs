//! Delivering a channel's output to Vimscript.
//!
//! Bytes arrive on the event loop and are accumulated per reader; the
//! `on_stdout`/`on_stderr`/`on_exit` callbacks run later, from the channel's
//! own queue, so that a callback which writes back to its channel cannot
//! recurse into the read path.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::memory::xstrdup;
use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{c_char, c_void};
use core::{mem, ptr, slice};

use crate::eval::callback_call;
use crate::eval::encode::encode_list_write;
use crate::eval::typval::{
    ListRef, callback_free, tv_clear, tv_dict_add_list, tv_dict_find, tv_list_alloc,
};
use crate::event::r#loop::one_arg_event;
use crate::event::multiqueue::multiqueue_put_event;
use crate::terminal::terminal_receive;
use crate::types::{CallbackReader, Channel, RStream, VarNumber, kListLenMayKnow, size_t};

use super::{channel_decref, channel_incref};

/// Starts buffering a reader's output under `type_0`, which names it in the
/// callback and in the `self` dict.
///
/// # Safety
/// `reader` is a live reader; `type_0` is a `'static` C string.
pub unsafe fn callback_reader_start(reader: *mut CallbackReader, type_0: *const c_char) {
    // SAFETY: the caller's reader, borrowed once for the whole setup.
    let reader = unsafe { &mut *reader };
    reader.buffer.clear();
    reader.type_0 = type_0;
}

/// # Safety
/// `reader` is live and its callback and buffer are this call's to release.
pub unsafe fn callback_reader_free(reader: *mut CallbackReader) {
    // SAFETY: the caller's reader.
    unsafe { callback_free(&raw mut (*reader).cb) };
    // The reader itself may live in `xmalloc` memory whose release never runs
    // a destructor, so the buffer is handed back here rather than at drop.
    drop(unsafe { mem::take(&mut (*reader).buffer) });
}

/// Whether a reader has anywhere to deliver to.
pub(super) fn callback_reader_set(reader: &CallbackReader) -> bool {
    reader.cb.is_set() || !reader.self_0.is_null()
}

/// # Safety
///
/// As `RStream`'s read callback: `stream` must point at the stream this was
/// registered on, `buf` at `count` readable bytes of it, and `data` at the
/// payload it was registered with, live for the call.
pub unsafe fn on_channel_data(
    stream: *mut RStream,
    buf: *const c_char,
    count: size_t,
    data: *mut c_void,
    eof: bool,
) -> size_t {
    let chan: *mut Channel = data.cast();
    // SAFETY: `data` is the channel the stream was started with.
    unsafe { on_channel_output(stream, chan, buf, count, eof, &raw mut (*chan).on_data) }
}

/// # Safety
///
/// As `RStream`'s read callback: `stream` must point at the stream this was
/// registered on, `buf` at `count` readable bytes of it, and `data` at the
/// payload it was registered with, live for the call.
pub unsafe fn on_job_stderr(
    stream: *mut RStream,
    buf: *const c_char,
    count: size_t,
    data: *mut c_void,
    eof: bool,
) -> size_t {
    let chan: *mut Channel = data.cast();
    // SAFETY: `data` is the channel the stream was started with.
    unsafe { on_channel_output(stream, chan, buf, count, eof, &raw mut (*chan).on_stderr) }
}

/// Accepts everything the stream offers: a terminal gets it immediately, a
/// reader accumulates it for the callback that runs on the channel's queue.
///
/// # Safety
/// `chan` and `reader` are live; `buf` is `count` readable bytes.
unsafe fn on_channel_output(
    _stream: *mut RStream,
    chan: *mut Channel,
    buf: *const c_char,
    count: size_t,
    eof: bool,
    reader: *mut CallbackReader,
) -> size_t {
    // SAFETY: the caller's live channel, reader and buffer, throughout.
    let term = unsafe { (*chan).term };
    if !term.is_null() {
        unsafe { terminal_receive(term, buf, count) };
    }
    if eof {
        unsafe { (*reader).eof = true };
    }
    if callback_reader_set(unsafe { &*reader }) {
        // SAFETY: `buf` is `count` readable bytes that do not alias the
        // reader's own buffer.
        let chunk = unsafe { slice::from_raw_parts(buf.cast::<u8>(), count) };
        unsafe { (*reader).buffer.extend_from_slice(chunk) };
        unsafe { schedule_channel_event(chan) };
    }
    count
}

/// Asks for the channel's callbacks to run.
///
/// At most one such event is outstanding, and a request made while they are
/// already running is deferred to the end of that run instead — otherwise a
/// callback that writes to its own channel would recurse.
///
/// # Safety
/// `chan` is a live channel.
pub(super) unsafe fn schedule_channel_event(chan: *mut Channel) {
    // SAFETY: the caller's live channel, throughout.
    if unsafe { (*chan).callback_scheduled } {
        return;
    }
    if unsafe { !(*chan).callback_busy } {
        unsafe { queue_channel_event(chan) };
    }
    unsafe { (*chan).callback_scheduled = true };
}

/// Puts one callback-running event on the channel's own queue, with the
/// reference that event holds.
///
/// # Safety
/// `chan` is a live channel.
unsafe fn queue_channel_event(chan: *mut Channel) {
    // SAFETY: the caller's live channel; its queue outlives the event.
    let ev = one_arg_event(Some(on_channel_event), chan.cast());
    unsafe { multiqueue_put_event((*chan).events, ev) };
    unsafe { channel_incref(chan) };
}

/// # Safety
///
/// `args` must point at a writable `*mut c_void` slot the caller owns for the
/// call.
unsafe extern "C" fn on_channel_event(args: *mut *mut c_void) {
    // SAFETY: the event carries the channel `queue_channel_event` queued it
    // for, and the reference it took.
    let chan = unsafe { *args }.cast::<Channel>();
    unsafe { (*chan).callback_busy = true };
    unsafe { (*chan).callback_scheduled = false };

    // Latched before the reader callbacks run: one of them may start
    // another job on this channel and reset it.
    let exit_status = unsafe { (*chan).exit_status };
    unsafe { channel_reader_callbacks(chan, &raw mut (*chan).on_data) };
    unsafe { channel_reader_callbacks(chan, &raw mut (*chan).on_stderr) };
    if exit_status > -1 {
        unsafe { channel_callback_call(chan, ptr::null_mut()) };
        unsafe { (*chan).exit_status = -1 };
    }

    unsafe { (*chan).callback_busy = false };
    if unsafe { (*chan).callback_scheduled } {
        unsafe { queue_channel_event(chan) };
    }
    unsafe { channel_decref(chan) };
}

/// Delivers whatever a reader has accumulated.
///
/// A buffered reader delivers once, at EOF — into the `self` dict if it has
/// one, otherwise through the callback. An unbuffered reader delivers as it
/// goes, and again with an empty list at EOF.
///
/// # Safety
/// `chan` and `reader` are live and `reader` belongs to `chan`.
pub unsafe fn channel_reader_callbacks(chan: *mut Channel, reader: *mut CallbackReader) {
    // SAFETY: the caller's live channel and reader, in both arms.
    if unsafe { (*reader).buffered } {
        unsafe { deliver_buffered(chan, reader) };
    } else {
        unsafe { deliver_streaming(chan, reader) };
    }
}

/// A buffered reader's one delivery, at EOF.
///
/// # Safety
/// As [`channel_reader_callbacks`].
unsafe fn deliver_buffered(chan: *mut Channel, reader: *mut CallbackReader) {
    // SAFETY: the caller's live channel and reader.
    if !unsafe { (*reader).eof } {
        return;
    }
    if unsafe { (*reader).self_0 }.is_null() {
        unsafe { channel_callback_call(chan, reader) };
    } else if unsafe { tv_dict_find((*reader).self_0, (*reader).type_0, -1) }.is_null() {
        let data = unsafe { reader_lines(reader) };
        let n_len = unsafe { cstr::bytes_at((*reader).type_0) }.len();
        let _ = unsafe { tv_dict_add_list((*reader).self_0, (*reader).type_0, n_len, Some(data)) };
    } else {
        // SAFETY: the reader's own stream name and the channel's id.
        let (kind, id) = unsafe { (c_str((*reader).type_0), (*chan).id) };
        semsg!("E5210: dict key '{kind}' already set for buffered stream in channel {id}");
    }
    unsafe { (*reader).eof = false };
}

/// An unbuffered reader's delivery: whatever has arrived, plus an empty
/// delivery to mark EOF.
///
/// # Safety
/// As [`channel_reader_callbacks`].
unsafe fn deliver_streaming(chan: *mut Channel, reader: *mut CallbackReader) {
    // SAFETY: the caller's live channel and reader.
    let is_eof = unsafe { (*reader).eof };
    if !unsafe { (*reader).buffer.is_empty() } {
        unsafe { channel_callback_call(chan, reader) };
    }
    if is_eof {
        unsafe { channel_callback_call(chan, reader) };
        unsafe { (*reader).eof = false };
    }
}

/// Calls `on_stdout`/`on_stderr` for `reader`, or `on_exit` when it is null.
///
/// # Safety
/// `chan` is live; `reader` is null or one of its readers.
unsafe fn channel_callback_call(chan: *mut Channel, reader: *mut CallbackReader) {
    let mut argv = [TV_INITIAL_VALUE; 3];
    let mut rettv = TV_INITIAL_VALUE;

    // SAFETY: the caller's live channel and reader. Every slot is this
    // frame's own value, released when the array goes out of scope --
    // which is why the stream name is duplicated rather than borrowed.
    argv[0].write_number(unsafe { (*chan).id }.cast_signed());
    let cb = if reader.is_null() {
        argv[1].write_number(VarNumber::from(unsafe { (*chan).exit_status }));
        argv[2].write_string(unsafe { xstrdup(c"exit".as_ptr()) });
        unsafe { &raw mut (*chan).on_exit }
    } else {
        argv[1].write_list(Some(unsafe { reader_lines(reader) }));
        unsafe { (*reader).buffer.clear() };
        argv[2].write_string(unsafe { xstrdup((*reader).type_0) });
        unsafe { &raw mut (*reader).cb }
    };

    unsafe { callback_call(cb, &argv, &mut rettv) };
    tv_clear(&mut rettv);
}

/// Everything a reader has accumulated, as the list of lines its callback is
/// handed.
///
/// The list always starts with one empty string, so a chunk that does not end
/// in a newline leaves a partial last line the next chunk continues.
///
/// # Safety
/// `reader` is live.
pub unsafe fn reader_lines(reader: *mut CallbackReader) -> ListRef {
    let l = tv_list_alloc(kListLenMayKnow as isize);
    let into = l.as_ptr();
    // SAFETY: the fresh list, and the caller's garray, which holds `ga_len`
    // readable bytes at `ga_data`.
    unsafe { (*into).push_string(c"".as_ptr(), 0) };
    let buffer = unsafe { &(*reader).buffer };
    if !buffer.is_empty() {
        unsafe { encode_list_write(into.cast(), buffer.as_ptr().cast(), buffer.len()) };
    }
    l
}
