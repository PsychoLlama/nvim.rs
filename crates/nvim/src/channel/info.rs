//! What the editor tells the user about its channels.
//!
//! `nvim_get_chan_info()` and `nvim_list_chans()` read the dicts built here,
//! and the same dict is what the `ChanOpen`/`ChanInfo` autocommands publish in
//! `v:event`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::CStr;
use std::ffi::CString;

use crate::autocmd::{fire_autocmds, has_event};
use crate::channel::channels;
use crate::eval::encode::encode_tv2json;
use crate::eval::{eval_source_name_line, get_v_event, restore_v_event};
use crate::event::multiqueue::put_id_event_on_main_loop;
use crate::event::proc::proc_is_stopped;
use crate::log::{LOGLVL_INF, logmsg};
use crate::message_fmt::msg_cstr;
use crate::registry::SlotTable;
use crate::terminal::terminal_buf;
use crate::types::{
    ApiDict, Array, AutoEvent, ChannelStream, ChannelStreamType, Integer, Object, SaveVEvent,
    String_0, TypVal, VAR_DICT, uint64_t,
};
use crate::winlayer::Buf;

use super::{ChannelRef, channel_decref, with_channel};

/// A string object holding a copy of a `'static` C literal.
fn literal_obj(text: &'static CStr) -> Object {
    Object::string(String_0::from_cstr(text))
}

/// Channel `id`'s info dict, as the `TypVal` the Vimscript layer wants.
fn info_tv(id: uint64_t) -> TypVal {
    let tv = TypVal::from(Object::dict(channel_info(id)));
    debug_assert!(tv.v_type() == VAR_DICT);
    tv
}

// ---------------------------------------------------------------------------
// Announcements
// ---------------------------------------------------------------------------

/// Announces newly opened channel `id`: `ChanOpen`, plus a log line naming
/// whatever opened it -- `ext_source`, or the script executing now.
pub fn channel_create_event(id: uint64_t, ext_source: Option<&CStr>) {
    // The script location is copied out of `IObuff` rather than pointed at:
    // building the info dict below reenters the evaluator, which formats its
    // own messages through the same scratch buffer.
    let script_source: Option<CString> = ext_source.is_none().then(eval_source_name_line);
    let source = ext_source.or(script_source.as_deref()).unwrap_or(c"");

    debug_assert!(i64::try_from(id).is_ok());
    let tv = info_tv(id);
    let info = encode_tv2json(&tv);
    let (source, info) = (msg_cstr(source), msg_cstr(info.as_cstr()));
    logmsg!(
        LOGLVL_INF,
        c"channel_create_event",
        258,
        "new channel {id} ({source}) : {info}"
    );
    channel_info_changed(id, true);
}

/// Queues the `ChanOpen`/`ChanInfo` autocommand, if anything is listening.
///
/// It is queued rather than fired here because the caller is often inside the
/// channel's own setup, where running arbitrary Vimscript would be reentrant.
pub fn channel_info_changed(id: uint64_t, new_chan: bool) {
    let event = if new_chan {
        AutoEvent::ChanOpen
    } else {
        AutoEvent::ChanInfo
    };
    if !has_event(event) {
        return;
    }
    // The event carries this reference, and `set_info_event` drops it.
    let Some(held) = ChannelRef::new(id) else {
        return;
    };
    let row = u64::try_from(event.index()).expect("an event row fits a u64");
    put_id_event_on_main_loop(set_info_event, held.into_raw(), row);
}

/// The main queue reached the event [`channel_info_changed`] put there.
fn set_info_event(id: uint64_t, row: u64) {
    let event = usize::try_from(row)
        .ok()
        .and_then(AutoEvent::at_row)
        .expect("`channel_info_changed` queued a real event");

    let mut save_v_event = SaveVEvent::default();
    let dict = get_v_event(&mut save_v_event);
    let retval = info_tv(id);
    if let Some(info) = retval.dict_shared() {
        let _ = dict.edit().add_dict(b"info", Some(info.clone()));
    }
    dict.edit().set_keys_readonly();
    fire_autocmds(event, true, Buf::current_or_none());
    restore_v_event(dict, &mut save_v_event);
    channel_decref(id);
}

// ---------------------------------------------------------------------------
// The info dict
// ---------------------------------------------------------------------------

/// Whether `id` names a job whose process is still running.
pub fn channel_job_running(id: uint64_t) -> bool {
    with_channel(id, |channel| match &channel.stream {
        ChannelStream::Job(job) => !proc_is_stopped(job.proc()),
        _ => false,
    })
    .unwrap_or(false)
}

/// The kind of stream the channel `id` names, or `None` when there is no such
/// channel.
pub(crate) fn channel_stream_type(id: uint64_t) -> Option<ChannelStreamType> {
    with_channel(id, |channel| channel.streamtype())
}

/// What `nvim_get_chan_info()` reports. An unknown id answers with an empty
/// dict rather than an error.
pub fn channel_info(id: uint64_t) -> ApiDict {
    with_channel(id, channel_info_of).unwrap_or(ApiDict::EMPTY)
}

/// [`channel_info`] of a channel borrowed from the table. Nothing here runs
/// user code.
fn channel_info_of(channel: &mut crate::types::Channel) -> ApiDict {
    // id, stream, mode, and up to six more from the branches below.
    let mut info = ApiDict::with_capacity(9);
    let mut push = |key: &CStr, value: Object| info.insert(key, value);

    push(c"id", Object::Integer(channel.id.cast_signed()));

    let stream_desc = match &mut channel.stream {
        ChannelStream::Job(job) => {
            if job.pty().is_some() {
                // A closed master has no name, which reads as a null string.
                let name = job.tty_name();
                let name = name.as_deref().map_or(String_0::NULL, String_0::from_cstr);
                push(c"pty", Object::string(name));
            }
            push(c"argv", Object::array(argv_array(&job.argv())));
            c"job"
        }
        ChannelStream::Stdio(_) => c"stdio",
        ChannelStream::Stderr(_) => c"stderr",
        ChannelStream::Internal(_) => {
            push(c"internal", Object::Boolean(true));
            // An internal channel reports itself as a socket, because that
            // is what it stands in for.
            c"socket"
        }
        ChannelStream::Socket(_) => c"socket",
    };
    push(c"stream", literal_obj(stream_desc));

    let mode_desc = if channel.is_rpc {
        push(c"client", Object::dict(channel.rpc.info.clone()));
        c"rpc"
    } else if let Some(term) = channel.term {
        let handle = Integer::from(terminal_buf(term));
        // `buf` is the documented key; `buffer` is kept for older plugins.
        push(c"buf", Object::Buffer(handle));
        push(c"buffer", Object::Buffer(handle));
        push(
            c"exitcode",
            Object::Integer(Integer::from(channel.exit_status)),
        );
        c"terminal"
    } else {
        c"bytes"
    };
    push(c"mode", literal_obj(mode_desc));
    info
}

/// The child's command line, as an array of strings copied out of it.
fn argv_array(args: &[&CStr]) -> Array {
    args.iter()
        .map(|arg| Object::string(String_0::from_cstr(arg)))
        .collect()
}

/// Every channel's info, ordered by id.
pub fn channel_all_info() -> Array {
    // The registry iterates in registration order; the API contract is
    // ascending id.
    let mut ids = channels.with(SlotTable::snapshot_keys);
    ids.sort_unstable();
    ids.into_iter()
        .map(|id| Object::dict(channel_info(id)))
        .collect()
}
