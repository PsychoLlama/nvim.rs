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

//! msgpack-rpc over a channel.
//!
//! Three message kinds travel each way. Requests carry an id and are answered
//! by a response with the same id; notifications carry none and are not
//! answered. Incoming bytes are decoded by [`unpacker`], dispatched here, and
//! answered through [`packer`].
//!
//! Everything is single-threaded and re-entrant: dispatching a request can run
//! arbitrary Lua, which can send another request and block on the reply, which
//! runs the event loop again. So a channel is reached by id and borrowed from
//! the table one step at a time ([`with_channel`]), never across a handler; a
//! queued request holds a reference to its channel, and the outstanding calls
//! form a stack the channel owns.
//!
//! [`unpacker`]: crate::msgpack_rpc::unpacker
//! [`packer`]: crate::msgpack_rpc::packer

use crate::event::multiqueue::{ChildQueue, put_id_event_on_fast_events, put_job};
use crate::message_fmt::{msg_addr, msg_bytes};
use crate::os::uv_error::UV_EPIPE;
use core::ffi::{CStr, c_int};

use crate::api::ui::remote_ui_disconnect;
use crate::channel::channels;
use crate::channel::{
    channel_close, channel_decref, channel_free_later, channel_incref, channel_info_changed,
    with_channel,
};
use crate::event::channel::{
    Reads, start_reads, stream_addresses, wait_on_channel_events, write_shared,
};
use crate::event::proc::exit_on_closed_chan;
use crate::event::wstream::SharedBuffer;
use crate::log::{LOGLVL_DBG, LOGLVL_ERR, LOGLVL_INF, logmsg, logmsg_line};
use crate::msgpack_rpc::packer::Addressees;
use crate::msgpack_rpc::unpacker::Unpacker;
use crate::startup::{ui_client_channel_id, ui_client_error_exit};
use crate::types::{
    ApiDict, Array, Channel, ChannelCallFrame, ChannelPart, ClientType, Error, Integer,
    MessageType, Object, String_0, kErrorTypeException, kErrorTypeValidation, uint64_t,
};
use crate::ui_client::ui_client_attach_to_restarted_server;

pub mod call_stack;
pub mod client;
pub mod envelope;
pub mod receive;
pub mod trace;

use call_stack::CallStack;
use client::classify_client;
use envelope::serialize_request;
use receive::parse_msgpack;

/// Values these belong to other modules; nested so they stay out of the flat
/// namespace the unit-test header generator collects constants into.
mod known {
    use super::{ChannelPart, ClientType, MessageType};

    pub(super) const kMessageTypeRequest: MessageType = 0;
    pub(super) const kMessageTypeResponse: MessageType = 1;
    pub(super) const kMessageTypeNotification: MessageType = 2;
    pub(super) const kMessageTypeRedrawEvent: MessageType = 3;

    pub(super) const kChannelPartRpc: ChannelPart = 3;

    pub(super) const kClientTypeMsgpackRpc: ClientType = 5;

    /// The widest message any of the close paths formats, matching the
    /// `char buf[256]` upstream declares at each of them.
    pub(super) const CLOSE_MSG_MAX: usize = 256;
}

use crate::types::channel::{kChannelStreamInternal, kChannelStreamProc, kChannelStreamStdio};
use known::*;

use crate::api_error;
use crate::global_cell::GlobalCell;

pub(crate) static ch_before_blocking_events: GlobalCell<Option<ChildQueue>> = GlobalCell::new(None);

// ---------------------------------------------------------------------------
// Opening and closing
// ---------------------------------------------------------------------------

/// Creates the queue that `nvim_get_mode` replies are answered from.
pub fn rpc_init() {
    ch_before_blocking_events.set(Some(ChildQueue::of_main_loop()));
}

/// Turns channel `id` into an RPC endpoint and starts reading from it.
///
/// The reference taken here is the channel's own: it is dropped by
/// [`rpc_close_event`] once the peer is gone.
pub fn rpc_start(id: uint64_t) {
    channel_incref(id);
    let internal = with_channel(id, |chan| {
        chan.is_rpc = true;
        let rpc = &mut chan.rpc;
        rpc.closed = false;
        rpc.unpacker = Some(Box::new(Unpacker::new()));
        rpc.next_request_id = 1;
        rpc.info = ApiDict::EMPTY;
        rpc.call_stack = CallStack::new();
        chan.streamtype() == kChannelStreamInternal
    })
    .expect("rpc_start on a live channel");

    // An internal channel has no transport to read from: its peer hands
    // messages straight to `rpc_write_raw`.
    if internal {
        return;
    }
    let (in_0, out) = stream_addresses(id);
    logmsg!(
        LOGLVL_DBG,
        c"rpc_start",
        93,
        "rpc ch {} in-stream={} out-stream={}",
        id,
        msg_addr(in_0),
        msg_addr(out)
    );
    start_reads(id, Reads::Rpc);
}

/// Marks channel `id` closed and schedules the teardown.
///
/// The state has to survive until the queued event runs — anything still
/// walking the channel gets a `closed` flag rather than freed memory.
pub fn rpc_close(id: uint64_t) {
    let newly = with_channel(id, |chan| !core::mem::replace(&mut chan.rpc.closed, true));
    if newly == Some(true) {
        // The channel's own reference, held since `rpc_start`, keeps it in
        // the table until the event drops it.
        put_id_event_on_fast_events(rpc_close_event, id, 0);
    }
}

/// Drops the channel's own reference, then decides whether the editor should
/// follow the peer out.
fn rpc_close_event(id: uint64_t, _: uint64_t) {
    let (streamtype, detach) = with_channel(id, |chan| (chan.streamtype(), chan.detach))
        .expect("a closing channel holds its own reference");
    channel_decref(id);
    // Nothing reads the reason a closing channel's UI could not be found.
    drop(remote_ui_disconnect(id, false));

    if ui_client_channel_id.get() != 0 && id == ui_client_channel_id.get() {
        // A `--remote-ui` client whose server went away: try to reconnect
        // before giving up.
        ui_client_attach_to_restarted_server();
        if ui_client_channel_id.get() != id {
            return;
        }
        // An embedded server exits through its process-exit callback instead,
        // which is what carries the exit status.
        if streamtype == kChannelStreamProc && ui_client_error_exit.get() < 0 {
            return;
        }
        exit_on_closed_chan(0);
    } else if streamtype == kChannelStreamStdio && !detach {
        exit_on_closed_chan(0);
    }
}

/// Releases everything `rpc_start` allocated. Called from `channel_destroy`,
/// with the channel out of the table.
pub fn rpc_free(channel: &mut Channel) {
    channel.rpc.unpacker = None;
    channel.rpc.call_stack = CallStack::new();
    // The dict was built by `rpc_set_client_info` and is owned here.
    drop(core::mem::take(&mut channel.rpc.info));
}

/// Whether channel `id` is talking msgpack-rpc, and still is.
fn is_live_rpc(chan: &Channel) -> bool {
    chan.is_rpc && !chan.rpc.closed
}

/// Fails every outstanding call with `msg`, then closes the channel.
fn chan_close_on_err(id: uint64_t, msg: &[u8], loglevel: c_int) {
    with_channel(id, |chan| {
        for frame in chan.rpc.call_stack.frames_mut() {
            if !frame.returned {
                frame.returned = true;
                frame.errored = true;
                frame.result = Object::string(String_0::from_bytes(msg));
            }
        }
    });
    let _ = channel_close(id, kChannelPartRpc);
    let msg = msg_bytes(msg);
    // Upstream's `LOG(loglevel, …)`: a level known only at run time, so the
    // line is filtered by the minimum level rather than compiled out.
    logmsg_line(
        loglevel,
        None,
        Some(c"chan_close_on_err"),
        545,
        true,
        || format!("RPC: {msg}"),
    );
}

/// A close message, cut where upstream's `char buf[256]` cut it.
fn close_message(fmt: &CStr, args: &[crate::strings::FmtArg<'_>]) -> Vec<u8> {
    crate::strings::formatted(CLOSE_MSG_MAX, fmt.to_bytes(), args)
}

// ---------------------------------------------------------------------------
// Sending
// ---------------------------------------------------------------------------

/// Sends a notification. `id` of 0 broadcasts to every RPC channel.
///
/// Returns false only when a named channel does not exist; a broadcast to
/// nobody still succeeds.
pub fn rpc_send_event(id: uint64_t, name: &CStr, args: Array) -> bool {
    if id != 0 && with_channel(id, |chan| is_live_rpc(chan)) != Some(true) {
        return false;
    }
    trace::log_call(trace::SEND, id, None, Some(name));
    if id == 0 {
        broadcast_event(name, args);
    } else {
        serialize_request(Addressees::One(id), 0, name.to_bytes(), args);
    }
    true
}

/// [`rpc_send_call`] of a method named as a string, the answer handed to
/// `read`.
pub(crate) fn rpc_call_with<T>(
    id: uint64_t,
    method_name: &CStr,
    args: Array,
    read: impl FnOnce(Object) -> T,
) -> Result<T, Error> {
    rpc_send_call(id, method_name, args).map(read)
}

/// Sends a request and runs the event loop until the answer arrives.
///
/// The call's frame lives on the channel's call stack; the decoder fills it
/// in when the matching response lands.
fn rpc_send_call(id: uint64_t, method_name: &CStr, args: Array) -> Result<Object, Error> {
    let invalid = || api_error!(kErrorTypeException, "Invalid channel: {id}");
    // The reference taken here keeps the channel, and so the frame, until
    // the answer has been read back.
    let request_id = with_channel(id, |chan| {
        is_live_rpc(chan).then(|| {
            chan.refcount.retain();
            let request_id = chan.rpc.next_request_id;
            chan.rpc.next_request_id = request_id.wrapping_add(1);
            request_id
        })
    })
    .flatten()
    .ok_or_else(invalid)?;

    serialize_request(
        Addressees::One(id),
        request_id,
        method_name.to_bytes(),
        args,
    );
    trace::log_call(trace::SEND, id, Some(request_id), Some(method_name));

    let depth = with_channel(id, |chan| {
        chan.rpc.call_stack.push(ChannelCallFrame {
            request_id,
            returned: false,
            errored: false,
            result: Object::Nil,
        })
    })
    .expect("the call holds a reference");
    wait_on_channel_events(id, || {
        with_channel(id, |chan| {
            chan.rpc.closed || chan.rpc.call_stack.frame(depth).returned
        })
        .unwrap_or(true)
    });
    // The frame back, and the reference taken above given up, in one step.
    let (frame, last) = with_channel(id, |chan| {
        let frame = chan.rpc.call_stack.pop();
        (frame, chan.refcount.release() == 0)
    })
    .expect("the call holds a reference");
    if last {
        channel_free_later(id);
    }
    let frame = frame.expect("the call's own frame");

    if !frame.returned {
        return Err(invalid());
    }
    if frame.errored {
        return Err(call_error(&frame.result));
    }
    Ok(frame.result)
}

/// Turns a peer's error payload into an `Error`.
///
/// The wire form is either a bare string or the `[type, message]` pair the
/// protocol prescribes; anything else, including a type outside the two the
/// API defines, is reported as "unknown error" rather than trusted.
fn call_error(result: &Object) -> Error {
    if let Some(text) = result.as_string() {
        return Error::from_message(kErrorTypeException, text.as_cstr());
    }
    if let Some(array) = result.as_array()
        && array.len() == 2
    {
        let (kind, message) = (&array[0], &array[1]);
        if let (Some(kind), Some(message)) = (kind.as_integer(), message.as_string())
            && (kind == Integer::from(kErrorTypeException)
                || kind == Integer::from(kErrorTypeValidation))
        {
            let kind = crate::narrow::number_as_int(kind);
            return Error::from_message(kind, message.as_cstr());
        }
    }
    Error::exception(c"unknown error")
}

/// Hands an already-encoded message to channel `id`; whether it took it.
pub(crate) fn rpc_write_raw(id: uint64_t, buffer: SharedBuffer) -> bool {
    if with_channel(id, |chan| is_live_rpc(chan)) != Some(true) {
        return false;
    }
    channel_write(id, buffer)
}

/// Sends `name` to every RPC channel at once.
///
/// One pass over the packer serves all of them: the packer's flush hands the
/// same buffer to each.
fn broadcast_event(name: &CStr, args: Array) {
    let chans: Vec<uint64_t> = channels.with(|table| {
        let rpc = table.values().filter(|channel| channel.is_rpc);
        rpc.map(|channel| channel.id).collect()
    });
    if !chans.is_empty() {
        serialize_request(Addressees::Many(chans), 0, name.to_bytes(), args);
    }
}

/// What [`channel_write`] did with a buffer.
enum Written {
    /// The channel is closed; the buffer was dropped.
    Refused,
    /// Queued for the channel's own decoder.
    Internal,
    /// Handed to the transport, with libuv's answer.
    Stream(c_int),
}

/// Queues an encoded message on channel `id`'s transport.
///
/// An internal channel has no transport: the bytes go straight back into its
/// decoder, through the channel's queue.
fn channel_write(id: uint64_t, buffer: SharedBuffer) -> bool {
    let written = with_channel(id, |chan| {
        if chan.rpc.closed {
            return Written::Refused;
        }
        if chan.streamtype() == kChannelStreamInternal {
            // The reference is dropped by `internal_read`.
            chan.refcount.retain();
            put_job(&chan.events, Box::new(move || internal_read(id, &buffer)));
            return Written::Internal;
        }
        Written::Stream(write_shared(chan, buffer))
    });
    match written {
        None | Some(Written::Refused) => false,
        Some(Written::Internal) => true,
        Some(Written::Stream(0)) => true,
        Some(Written::Stream(err)) => {
            let why = crate::os::fs::os_strerror(err);
            let msg = close_message(
                c"ch %lu: stream write failed: %s. RPC canceled; closing channel",
                &[id.into(), why.into()],
            );
            let level = if err == UV_EPIPE {
                LOGLVL_INF
            } else {
                LOGLVL_ERR
            };
            chan_close_on_err(id, &msg, level);
            false
        }
    }
}

/// Feeds an internal channel's own message back through its decoder, and
/// drops the reference [`channel_write`] took.
fn internal_read(id: uint64_t, buffer: &SharedBuffer) {
    let left = parse_msgpack(id, buffer.bytes());
    // The peer is this process, so a message it could not finish decoding is
    // a bug here rather than a protocol violation.
    let closed = with_channel(id, |chan| chan.rpc.closed).unwrap_or(true);
    if left != 0 && !closed {
        chan_close_on_err(id, b"internal channel: internal error", LOGLVL_ERR);
    }
    channel_decref(id);
}

// ---------------------------------------------------------------------------
// Client info
// ---------------------------------------------------------------------------

/// The string value `key` has in an info dict.
fn info_string<'a>(info: &'a ApiDict, key: &CStr) -> Option<&'a CStr> {
    info.iter().find_map(|item| {
        let value = item.value.as_string()?;
        (item.key.as_c_str() == key).then(|| value.as_cstr())
    })
}

/// Records what `nvim_set_client_info` was told, and classifies the peer.
///
/// The classification decides how responses are matched (see
/// [`CallStack::find`]), so a peer that claims `msgpack-rpc` is taken at its
/// word.
pub fn rpc_set_client_info(id: uint64_t, info: ApiDict) {
    with_channel(id, |chan| {
        assert!(
            is_live_rpc(chan),
            "client info for a channel that is not rpc"
        );
        // The dict being replaced was owned by the channel.
        chan.rpc.info = info;
        chan.rpc.client_type = classify_client(info_string(&chan.rpc.info, c"type"));
    })
    .expect("client info for a channel that is not rpc");
    channel_info_changed(id, false);
}

/// A copy of the string value `key` has in channel `id`'s client info, when
/// it is an RPC channel that has one.
pub(crate) fn client_info(id: uint64_t, key: &CStr) -> Option<std::ffi::CString> {
    with_channel(id, |channel| {
        if !channel.is_rpc {
            return None;
        }
        info_string(&channel.rpc.info, key).map(CStr::to_owned)
    })
    .flatten()
}
