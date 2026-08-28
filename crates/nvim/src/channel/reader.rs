//! Delivering a channel's output to Vimscript.
//!
//! Bytes arrive on the event loop and are accumulated per reader; the
//! `on_stdout`/`on_stderr`/`on_exit` callbacks run later, from the channel's
//! own queue, so that a callback which writes back to its channel cannot
//! recurse into the read path.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::eval::callback_call;
use crate::eval::encode::encode_list_write;
use crate::eval::typval::{
    callback_free, kCallbackNone, tv_clear, tv_dict_add_list, tv_dict_find, tv_list_alloc,
    tv_list_append_string, tv_list_ref, tv_list_unref,
};
use crate::event::r#loop::one_arg_event;
use crate::event::multiqueue::multiqueue_put_event;
use crate::garray::{ga_clear, ga_concat_len, ga_init};
use crate::main::e_streamkey;
use crate::terminal::terminal_receive;
use crate::types::{
    CallbackReader, Channel, RStream, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VarLock,
    kListLenMayKnow, list_T, size_t, typval_T, typval_vval_union, varnumber_T,
};
use ::libc::strlen;

use super::{channel_decref, channel_incref, translated};

/// A `typval_T` of no type, which is what an argument slot starts as.
fn unknown_tv() -> typval_T {
    typval_T {
        v_type: VAR_UNKNOWN as _,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union { v_number: 0 },
    }
}

/// Starts buffering a reader's output under `type_0`, which names it in the
/// callback and in the `self` dict.
///
/// # Safety
/// `reader` is a live, zeroed or cleared reader; `type_0` is a `'static` C
/// string.
pub unsafe fn callback_reader_start(reader: *mut CallbackReader, type_0: *const c_char) {
    let elem_size = size_of::<*mut c_char>() as c_int;
    // SAFETY: the caller's reader.
    unsafe { ga_init(&raw mut (*reader).buffer, elem_size, 32) };
    unsafe { (*reader).type_0 = type_0 };
}

/// # Safety
/// `reader` is live and its callback and buffer are this call's to release.
pub unsafe fn callback_reader_free(reader: *mut CallbackReader) {
    // SAFETY: the caller's reader.
    unsafe { callback_free(&raw mut (*reader).cb) };
    unsafe { ga_clear(&raw mut (*reader).buffer) };
}

/// Whether a reader has anywhere to deliver to.
pub(super) fn callback_reader_set(reader: &CallbackReader) -> bool {
    reader.cb.type_0 != kCallbackNone || !reader.self_0.is_null()
}

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
        unsafe { ga_concat_len(&raw mut (*reader).buffer, buf, count) };
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
        unsafe {
            tv_dict_add_list(
                (*reader).self_0,
                (*reader).type_0,
                strlen((*reader).type_0),
                data,
            )
        };
    } else {
        unsafe { semsg_c!(translated(&e_streamkey), (*reader).type_0, (*chan).id,) };
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
    if unsafe { (*reader).buffer.ga_len } > 0 {
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
    let mut argv: [typval_T; 4] = [unknown_tv(); 4];
    argv[0].v_type = VAR_NUMBER as _;
    argv[2].v_type = VAR_STRING as _;
    let mut rettv = unknown_tv();

    // SAFETY: the caller's live channel and reader. The list built for a
    // reader is owned by `argv[1]` until it is unreferenced below.
    argv[0].vval.v_number = unsafe { (*chan).id } as varnumber_T;
    let cb = if reader.is_null() {
        argv[1].v_type = VAR_NUMBER as _;
        argv[1].vval.v_number = unsafe { (*chan).exit_status } as varnumber_T;
        argv[2].vval.v_string = c"exit".as_ptr() as *mut c_char;
        unsafe { &raw mut (*chan).on_exit }
    } else {
        argv[1].v_type = VAR_LIST as _;
        argv[1].vval.v_list = unsafe { reader_lines(reader) };
        unsafe { tv_list_ref(argv[1].vval.v_list) };
        unsafe { ga_clear(&raw mut (*reader).buffer) };
        argv[2].vval.v_string = unsafe { (*reader).type_0 } as *mut c_char;
        unsafe { &raw mut (*reader).cb }
    };

    unsafe { callback_call(cb, 3, argv.as_mut_ptr(), &raw mut rettv) };
    unsafe { tv_clear(&raw mut rettv) };
    if !reader.is_null() {
        unsafe { tv_list_unref(argv[1].vval.v_list) };
    }
}

/// Everything a reader has accumulated, as the list of lines its callback is
/// handed.
///
/// The list always starts with one empty string, so a chunk that does not end
/// in a newline leaves a partial last line the next chunk continues.
///
/// # Safety
/// `reader` is live.
unsafe fn reader_lines(reader: *mut CallbackReader) -> *mut list_T {
    let l = unsafe { tv_list_alloc(kListLenMayKnow as isize) };
    // SAFETY: the fresh list, and the caller's garray, which holds `ga_len`
    // readable bytes at `ga_data`.
    unsafe { tv_list_append_string(l, c"".as_ptr(), 0) };
    let len = unsafe { (*reader).buffer.ga_len } as size_t;
    if len > 0 {
        unsafe { encode_list_write(l.cast(), (*reader).buffer.ga_data.cast(), len) };
    }
    l
}
