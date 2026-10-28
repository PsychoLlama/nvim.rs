//! Delivering a channel's output to Vimscript.
//!
//! Bytes arrive on the event loop and are accumulated per reader; the
//! `on_stdout`/`on_stderr`/`on_exit` callbacks run later, from the channel's
//! own queue, so that a callback which writes back to its channel cannot
//! recurse into the read path.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::CStr;
use core::mem;

use crate::eval::callback_call;
use crate::eval::encode::encode_list_write;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::{
    ListRef, callback_copy, callback_free, dict_find, tv_clear, tv_list_alloc,
};
use crate::event::multiqueue::put_id_event;
use crate::memory::ThinCString;
use crate::semsg;
use crate::terminal::terminal_receive;
use crate::types::{Callback, CallbackReader, Channel, VarNumber, kListLenMayKnow, uint64_t};

use super::{ChannelRef, channel_decref, with_channel};

/// Starts buffering a reader's output under `stream_name`, which names it in
/// the callback and in the `self` dict.
pub fn callback_reader_start(reader: &mut CallbackReader, stream_name: &'static CStr) {
    reader.buffer.clear();
    reader.stream_name = stream_name;
}

/// Release `reader`'s callback, dictionary and buffer.
pub fn callback_reader_free(reader: &mut CallbackReader) {
    callback_free(&mut reader.cb);
    drop(reader.self_dict.take());
    drop(mem::take(&mut reader.buffer));
}

/// The reader `stderr` picks out of `channel`.
fn reader_of(channel: &mut Channel, stderr: bool) -> &mut CallbackReader {
    if stderr {
        &mut channel.on_stderr
    } else {
        &mut channel.on_data
    }
}

/// Accepts everything a read stream offers channel `id`: a terminal gets it
/// immediately, a reader accumulates it for the callback that runs on the
/// channel's queue. `stderr` picks the job's stderr reader over its
/// stdout/data one. Answers how much was consumed, which is all of it.
pub(crate) fn on_channel_output(id: uint64_t, stderr: bool, bytes: &[u8], eof: bool) -> usize {
    let Some(term) = with_channel(id, |channel| {
        let term = channel.term;
        let reader = reader_of(channel, stderr);
        if eof {
            reader.eof = true;
        }
        let deliver = reader.is_set();
        if deliver {
            reader.buffer.extend_from_slice(bytes);
        }
        (term, deliver)
    }) else {
        return bytes.len();
    };
    let (term, deliver) = term;
    // The terminal sees the bytes before the callbacks are scheduled, as
    // upstream's order has it; it re-enters the channel (its replies to the
    // child), so nothing of the channel is borrowed meanwhile.
    if let Some(term) = term {
        terminal_receive(term, bytes);
    }
    if deliver {
        schedule_channel_event(id);
    }
    bytes.len()
}

/// Asks for channel `id`'s callbacks to run.
///
/// At most one such event is outstanding, and a request made while they are
/// already running is deferred to the end of that run instead — otherwise a
/// callback that writes to its own channel would recurse.
pub(super) fn schedule_channel_event(id: uint64_t) {
    let queue_now = with_channel(id, |channel| {
        if channel.callback_scheduled {
            return false;
        }
        channel.callback_scheduled = true;
        !channel.callback_busy
    });
    if queue_now == Some(true) {
        queue_channel_event(id);
    }
}

/// Puts one callback-running event on channel `id`'s own queue, with the
/// reference that event holds.
fn queue_channel_event(id: uint64_t) {
    let Some(held) = ChannelRef::new(id) else {
        return;
    };
    let id = held.into_raw();
    with_channel(id, |channel| {
        put_id_event(&channel.events, on_channel_event, id, 0)
    });
}

/// The channel's queue reached the event [`queue_channel_event`] put there:
/// run its callbacks, then drop the event's reference.
fn on_channel_event(id: uint64_t, _: u64) {
    // Latched before the reader callbacks run: one of them may start
    // another job on this channel and reset it.
    let Some(exit_status) = with_channel(id, |channel| {
        channel.callback_busy = true;
        channel.callback_scheduled = false;
        channel.exit_status
    }) else {
        return;
    };
    channel_reader_callbacks(id, false);
    channel_reader_callbacks(id, true);
    if exit_status > -1 {
        channel_callback_call(id, None);
        with_channel(id, |channel| channel.exit_status = -1);
    }

    let requeue = with_channel(id, |channel| {
        channel.callback_busy = false;
        channel.callback_scheduled
    });
    if requeue == Some(true) {
        queue_channel_event(id);
    }
    channel_decref(id);
}

/// Delivers whatever one of channel `id`'s readers has accumulated.
///
/// A buffered reader delivers once, at EOF — into the `self` dict if it has
/// one, otherwise through the callback. An unbuffered reader delivers as it
/// goes, and again with an empty list at EOF.
pub fn channel_reader_callbacks(id: uint64_t, stderr: bool) {
    let buffered = with_channel(id, |channel| reader_of(channel, stderr).buffered);
    match buffered {
        Some(true) => deliver_buffered(id, stderr),
        Some(false) => deliver_streaming(id, stderr),
        None => {}
    }
}

/// A buffered reader's one delivery, at EOF.
fn deliver_buffered(id: uint64_t, stderr: bool) {
    enum Into {
        Callback,
        Dict,
        Taken(&'static CStr),
    }
    let into = with_channel(id, |channel| {
        let reader = reader_of(channel, stderr);
        if !reader.eof {
            return None;
        }
        Some(match &reader.self_dict {
            None => Into::Callback,
            Some(dict) => {
                let name = reader.stream_name;
                if dict_find(Some(dict), name.to_bytes()).is_none() {
                    let lines = reader_lines(reader);
                    let _ = dict.edit().add_list(name.to_bytes(), Some(lines));
                    Into::Dict
                } else {
                    Into::Taken(name)
                }
            }
        })
    })
    .flatten();
    match into {
        None => return,
        Some(Into::Callback) => channel_callback_call(id, Some(stderr)),
        Some(Into::Dict) => {}
        Some(Into::Taken(name)) => {
            let kind = name.to_string_lossy();
            semsg!("E5210: dict key '{kind}' already set for buffered stream in channel {id}");
        }
    }
    with_channel(id, |channel| reader_of(channel, stderr).eof = false);
}

/// An unbuffered reader's delivery: whatever has arrived, plus an empty
/// delivery to mark EOF.
fn deliver_streaming(id: uint64_t, stderr: bool) {
    let Some((is_eof, has_data)) = with_channel(id, |channel| {
        let reader = reader_of(channel, stderr);
        (reader.eof, !reader.buffer.is_empty())
    }) else {
        return;
    };
    if has_data {
        channel_callback_call(id, Some(stderr));
    }
    if is_eof {
        channel_callback_call(id, Some(stderr));
        with_channel(id, |channel| reader_of(channel, stderr).eof = false);
    }
}

/// Calls `on_stdout`/`on_stderr` for the reader `stderr` picks, or
/// `on_exit` for `None`.
///
/// The callback is copied out and called with nothing of the channel
/// borrowed: it runs Vimscript, which can close the channel or start
/// another job on it.
fn channel_callback_call(id: uint64_t, reader: Option<bool>) {
    let mut argv = [TV_INITIAL_VALUE, TV_INITIAL_VALUE, TV_INITIAL_VALUE];
    let Some(mut cb) = with_channel(id, |channel| {
        let mut cb = Callback::None;
        match reader {
            None => {
                argv[1].write_number(VarNumber::from(channel.exit_status));
                argv[2].write_string(Some(ThinCString::from_cstr(c"exit")));
                callback_copy(&mut cb, &channel.on_exit);
            }
            Some(stderr) => {
                let reader = reader_of(channel, stderr);
                argv[1].write_list(Some(reader_lines(reader)));
                reader.buffer.clear();
                argv[2].write_string(Some(ThinCString::from_cstr(reader.stream_name)));
                callback_copy(&mut cb, &reader.cb);
            }
        }
        cb
    }) else {
        return;
    };
    argv[0].write_number(id.cast_signed());

    let mut rettv = TV_INITIAL_VALUE;
    callback_call(&cb, &argv, &mut rettv);
    tv_clear(&mut rettv);
    callback_free(&mut cb);
}

/// Everything a reader has accumulated, as the list of lines its callback
/// is handed.
///
/// The list always starts with one empty string, so a chunk that does not
/// end in a newline leaves a partial last line the next chunk continues.
pub fn reader_lines(reader: &CallbackReader) -> ListRef {
    let mut l = tv_list_alloc(kListLenMayKnow as isize);
    l.push_bytes(Some(b""));
    encode_list_write(&mut l, &reader.buffer);
    l
}
