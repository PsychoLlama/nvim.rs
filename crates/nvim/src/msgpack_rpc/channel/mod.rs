#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! msgpack-rpc over a channel.
//!
//! Three message kinds travel each way. Requests carry an id and are answered
//! by a response with the same id; notifications carry none and are not
//! answered. Incoming bytes are decoded by [`unpacker`], dispatched here, and
//! answered through [`packer`].
//!
//! Everything is single-threaded and re-entrant: dispatching a request can run
//! arbitrary Lua, which can send another request and block on the reply, which
//! runs the event loop again. That is why the channel is ref-counted around
//! every callback and why the outstanding calls form a stack.
//!
//! [`unpacker`]: crate::msgpack_rpc::unpacker
//! [`packer`]: crate::msgpack_rpc::packer

use crate::os::uv_error::UV_EPIPE;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ops::{Deref, DerefMut};
use core::{ptr, slice};

use crate::api::private::helpers::{api_free_dict, arena_string, cstr_as_string};
use crate::api::ui::remote_ui_disconnect;
use crate::channel::{
    channel_close, channel_decref, channel_incref, channel_info_changed, channel_instream,
    channel_outstream, find_channel,
};
use crate::event::libuv::uv_strerror;
use crate::event::r#loop::{one_arg_event, process_events_until};
use crate::event::multiqueue::{multiqueue_new_child, multiqueue_put_event};
use crate::event::proc::exit_on_closed_chan;
use crate::event::rstream::rstream_start;
use crate::event::wstream::{wstream_release_wbuffer, wstream_write};
use crate::log::{LOGLVL_DBG, LOGLVL_ERR, LOGLVL_INF, logmsg};
use crate::main::{
    ch_before_blocking_events, channels, main_loop, ui_client_channel_id, ui_client_error_exit,
};
use crate::memory::{arena_finish, arena_mem_free, xcalloc, xfree};
use crate::msgpack_rpc::unpacker::{unpacker_init, unpacker_teardown};
use crate::registry::SlotTable;
use crate::types::{
    Arena, ArenaMem, Array, Channel, ChannelCallFrame, ChannelPart, ChannelStreamType, ClientType,
    Dict, Error, Integer, MessageType, MsgpackRpcRequestHandler, Object, Unpacker, WBuffer,
    kErrorTypeException, kErrorTypeValidation, kObjectTypeArray, kObjectTypeInteger,
    kObjectTypeNil, kObjectTypeString, uint32_t, uint64_t,
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
use receive::{parse_msgpack, receive_msgpack};

/// Values these belong to other modules; nested so they stay out of the flat
/// namespace the unit-test header generator collects constants into.
mod known {
    use super::{ChannelPart, ChannelStreamType, ClientType, MessageType};

    pub(super) const kMessageTypeRequest: MessageType = 0;
    pub(super) const kMessageTypeResponse: MessageType = 1;
    pub(super) const kMessageTypeNotification: MessageType = 2;
    pub(super) const kMessageTypeRedrawEvent: MessageType = 3;

    pub(super) const kChannelStreamProc: ChannelStreamType = 0;
    pub(super) const kChannelStreamStdio: ChannelStreamType = 2;
    pub(super) const kChannelStreamInternal: ChannelStreamType = 4;
    pub(super) const kChannelPartRpc: ChannelPart = 3;

    pub(super) const kClientTypeMsgpackRpc: ClientType = 5;

    /// libuv's "the peer hung up" error.
    /// The arena block size the RPC packer writes into.
    pub(super) const ARENA_BLOCK_SIZE: usize = 4096;

    /// The widest message any of the close paths formats, matching the
    /// `char buf[256]` upstream declares at each of them.
    pub(super) const CLOSE_MSG_MAX: usize = 256;
}

use crate::api::private::validate::err_msg_ptr;
use known::*;

/// The all-nil `Object`, which is what an API call that produced nothing, or
/// failed, hands back.
const NIL: Object = Object {
    type_0: kObjectTypeNil,
    data: crate::types::object_data { boolean: false },
};
use crate::api_error;

/// A channel this module is working with, plus the promise that the pointer
/// behind it stays live for as long as the handle does.
///
/// The RPC layer is re-entrant — dispatching a request can run Lua that sends
/// another request and blocks on the reply — so a channel is always reached
/// through a raw pointer that outlives any borrow of it. Wrapping the pointer
/// pays the `unsafe` once, at construction, and leaves `chan.rpc.closed` and
/// `chan.id` ordinary Rust everywhere below.
#[derive(Copy, Clone)]
struct Chan(*mut Channel);

impl Chan {
    /// # Safety
    /// `channel` is non-null and points at a live `Channel` for the whole life
    /// of the handle and of everything derived from it.
    unsafe fn new(channel: *mut Channel) -> Self {
        debug_assert!(!channel.is_null());
        Chan(channel)
    }

    /// The pointer back, for the C-shaped callees that still want one.
    fn as_ptr(self) -> *mut Channel {
        self.0
    }

    /// The channel's decoder.
    ///
    /// # Safety
    /// The channel has been through [`rpc_start`], so its unpacker exists.
    unsafe fn unpacker<'a>(self) -> &'a mut Unpacker {
        let p: *mut Unpacker = self.rpc.unpacker;
        // SAFETY: `rpc_start` allocates it and `rpc_free` is the only thing
        // that releases it, after which no caller here holds the channel.
        unsafe { &mut *p }
    }
}

impl Deref for Chan {
    type Target = Channel;

    fn deref(&self) -> &Channel {
        // SAFETY: the promise made at construction.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Chan {
    fn deref_mut(&mut self) -> &mut Channel {
        // SAFETY: the promise made at construction.
        unsafe { &mut *self.0 }
    }
}

/// A dispatched request, in flight between the decoder and whichever queue
/// gets to run it.
///
/// It owns the arena the arguments were decoded into: the handler reads them
/// and the arena is released once the response has been packed.
#[derive(Clone)]
pub struct RequestEvent {
    pub type_0: MessageType,
    pub channel: *mut Channel,
    pub handler: MsgpackRpcRequestHandler,
    pub args: Array,
    pub request_id: uint32_t,
    pub used_mem: Arena,
}

// ---------------------------------------------------------------------------
// Opening and closing
// ---------------------------------------------------------------------------

/// Creates the queue that `nvim_get_mode` replies are answered from.
///
/// # Safety
/// The main loop is initialised.
pub unsafe fn rpc_init() {
    // SAFETY: the caller's guarantee that the loop exists.
    let queue = unsafe { multiqueue_new_child((*main_loop.ptr()).events) };
    ch_before_blocking_events.set(queue);
}

/// Turns `channel` into an RPC endpoint and starts reading from it.
///
/// The reference taken here is the channel's own: it is dropped by
/// [`rpc_close_event`] once the peer is gone.
///
/// # Safety
/// `channel` points at a live `Channel` that is not already an RPC endpoint.
pub unsafe fn rpc_start(channel: *mut Channel) {
    // SAFETY: the caller's channel.
    let mut chan = unsafe {
        channel_incref(channel);
        Chan::new(channel)
    };
    chan.is_rpc = true;

    // SAFETY: `xcalloc` hands back `size_of::<Unpacker>()` zeroed bytes, which
    // is what `unpacker_init` expects to be handed.
    let unpacker = unsafe { xcalloc(1, size_of::<Unpacker>()) }.cast::<Unpacker>();
    // SAFETY: as above.
    unsafe { unpacker_init(unpacker) };

    let rpc = &mut chan.rpc;
    rpc.closed = false;
    rpc.unpacker = unpacker;
    rpc.next_request_id = 1;
    rpc.info = Dict {
        size: 0,
        capacity: 0,
        items: ptr::null_mut(),
    };
    rpc.call_stack = CallStack::new();

    // An internal channel has no transport to read from: its peer hands
    // messages straight to `rpc_write_raw`.
    if chan.streamtype == kChannelStreamInternal {
        return;
    }
    let id = chan.id;
    // SAFETY: the channel is live and now an RPC endpoint, so it has both
    // streams; `receive_msgpack` is handed the same pointer back.
    let out = unsafe { channel_outstream(channel) };
    let in_0 = unsafe { channel_instream(channel) };
    unsafe {
        logmsg!(
            LOGLVL_DBG,
            c"rpc_start",
            93,
            c"rpc ch %lu in-stream=%p out-stream=%p",
            id,
            in_0.cast::<c_void>(),
            out.cast::<c_void>(),
        )
    };
    unsafe { rstream_start(out, Some(receive_msgpack), channel.cast::<c_void>()) };
}

/// Marks the channel closed and schedules the teardown.
///
/// The state has to survive until the queued event runs — anything still
/// walking the channel gets a `closed` flag rather than freed memory.
///
/// # Safety
/// `channel` points at a live RPC `Channel`.
pub unsafe fn rpc_close(channel: *mut Channel) {
    // SAFETY: the caller's channel.
    let mut chan = unsafe { Chan::new(channel) };
    if chan.rpc.closed {
        return;
    }
    chan.rpc.closed = true;
    // SAFETY: the loop exists whenever a channel does, and the event carries
    // the channel to `rpc_close_event`, which is what keeps it alive.
    let queue = unsafe { (*main_loop.ptr()).fast_events };
    let event = one_arg_event(Some(rpc_close_event), channel.cast::<c_void>());
    unsafe { multiqueue_put_event(queue, event) };
}

/// Drops the channel's own reference, then decides whether the editor should
/// follow the peer out.
///
/// # Safety
/// `argv[0]` is the live channel [`rpc_close`] queued this event for.
unsafe extern "C" fn rpc_close_event(argv: *mut *mut c_void) {
    // SAFETY: `rpc_close` built this argument vector with one live channel.
    let chan = unsafe { Chan::new((*argv).cast::<Channel>()) };
    // SAFETY: as above.
    unsafe { channel_decref(chan.as_ptr()) };
    // Nothing reads the reason a closing channel's UI could not be found.
    let mut ignored = Error::none();
    remote_ui_disconnect(chan.id, &mut ignored, false);

    if ui_client_channel_id.get() != 0 && chan.id == ui_client_channel_id.get() {
        // A `--remote-ui` client whose server went away: try to reconnect
        // before giving up.
        // SAFETY: reattaching runs entirely inside the ui client.
        unsafe { ui_client_attach_to_restarted_server() };
        if ui_client_channel_id.get() != chan.id {
            return;
        }
        // An embedded server exits through its process-exit callback instead,
        // which is what carries the exit status.
        if chan.streamtype == kChannelStreamProc && ui_client_error_exit.get() < 0 {
            return;
        }
        exit_on_closed_chan(0);
    } else if chan.streamtype == kChannelStreamStdio && !chan.detach {
        exit_on_closed_chan(0);
    }
}

/// Releases everything `rpc_start` allocated. Called from `channel_destroy`.
///
/// # Safety
/// `channel` points at a `Channel` that has been through [`rpc_start`] and is
/// about to be destroyed.
pub unsafe fn rpc_free(channel: *mut Channel) {
    // SAFETY: the caller's channel, whose unpacker and info dict this owns.
    let mut chan = unsafe { Chan::new(channel) };
    // SAFETY: as above.
    unsafe { unpacker_teardown(chan.rpc.unpacker) };
    unsafe { xfree(chan.rpc.unpacker.cast::<c_void>()) };
    chan.rpc.call_stack = CallStack::new();
    // SAFETY: the dict was built by `rpc_set_client_info` and is owned here.
    unsafe { api_free_dict(chan.rpc.info) };
}

/// The channel `id` is talking msgpack-rpc over, if it still is.
///
/// # Safety
/// The channel table is initialised.
unsafe fn find_rpc_channel(id: uint64_t) -> Option<Chan> {
    // `find_channel` answers null rather than a dangling pointer.
    let chan = find_channel(id);
    if chan.is_null() {
        return None;
    }
    // SAFETY: `find_channel` answered a live channel.
    let chan = unsafe { Chan::new(chan) };
    (chan.is_rpc && !chan.rpc.closed).then_some(chan)
}

/// Fails every outstanding call with `msg`, then closes the channel.
///
/// The frames are filled from the unpacker's arena, which the reply would
/// otherwise have used; `arena_finish` hands each frame its own block, so the
/// message outlives the arena.
///
/// # Safety
/// `chan` has been through [`rpc_start`] and `msg` is a NUL-terminated string.
unsafe fn chan_close_on_err(chan: Chan, msg: *mut c_char, loglevel: c_int) {
    // SAFETY: the channel's own unpacker, and the caller's message.
    let arena = &raw mut unsafe { chan.unpacker() }.arena;
    for frame in chan.rpc.call_stack.frames() {
        if !unsafe { (*frame).returned } {
            unsafe { (*frame).returned = true };
            unsafe { (*frame).errored = true };
            let string = unsafe { arena_string(arena, cstr_as_string(msg)) };
            let result = Object {
                type_0: kObjectTypeString,
                data: crate::types::object_data { string },
            };
            unsafe { (*frame).result = result };
            let mem = unsafe { arena_finish(arena) };
            unsafe { (*frame).result_mem = mem };
        }
    }
    unsafe { channel_close(chan.id, kChannelPartRpc, ptr::null_mut()) };
    unsafe { logmsg!(loglevel, c"chan_close_on_err", 545, c"RPC: %s", msg) };
}

// ---------------------------------------------------------------------------
// Sending
// ---------------------------------------------------------------------------

/// Sends a notification. `id` of 0 broadcasts to every RPC channel.
///
/// Returns false only when a named channel does not exist; a broadcast to
/// nobody still succeeds.
///
/// # Safety
/// `name` is a NUL-terminated string and `args` describes `args.size` live
/// objects.
pub unsafe fn rpc_send_event(id: uint64_t, name: *const c_char, args: Array) -> bool {
    // SAFETY: the channel table is live whenever the editor is.
    let channel = if id == 0 {
        None
    } else {
        match unsafe { find_rpc_channel(id) } {
            Some(chan) => Some(chan),
            None => return false,
        }
    };
    // SAFETY: the caller's name, and a channel this just resolved.
    unsafe { trace::log_call(trace::SEND, channel.map_or(0, |chan| chan.id), None, name) };
    match channel {
        Some(chan) => unsafe {
            serialize_request(slice::from_mut(&mut chan.as_ptr()), 0, name, args)
        },
        None => unsafe { broadcast_event(name, args) },
    }
    true
}

/// Sends a request and runs the event loop until the answer arrives.
///
/// The frame lives on this stack frame; the decoder fills it in when the
/// matching response lands. `result_mem` receives the arena the result was
/// decoded into, which the caller must free.
///
/// # Safety
/// `method_name` is a NUL-terminated string, `args` describes `args.size` live
/// objects, and `result_mem`/`err` point at writable slots.
pub unsafe fn rpc_send_call(
    id: uint64_t,
    method_name: *const c_char,
    args: Array,
    result_mem: *mut ArenaMem,
    err: &mut Error,
) -> Object {
    // SAFETY: the channel table is live whenever the editor is.
    let Some(mut chan) = (unsafe { find_rpc_channel(id) }) else {
        *err = api_error!(kErrorTypeException, "Invalid channel: {id}");
        return NIL;
    };
    // SAFETY: the channel is live; this reference is dropped below.
    unsafe { channel_incref(chan.as_ptr()) };

    let request_id = chan.rpc.next_request_id;
    chan.rpc.next_request_id = request_id.wrapping_add(1);
    // SAFETY: the caller's method name and arguments.
    let mut only = chan.as_ptr();
    let to = slice::from_mut(&mut only);
    unsafe { serialize_request(to, request_id, method_name, args) };
    unsafe { trace::log_call(trace::SEND, chan.id, Some(request_id), method_name) };

    let mut frame = ChannelCallFrame {
        request_id,
        returned: false,
        errored: false,
        result: NIL,
        result_mem: ptr::null_mut(),
    };
    let frame_ptr = &raw mut frame;
    chan.rpc.call_stack.push(request_id, frame_ptr);
    // The condition reads a stack frame this call owns and a channel it holds
    // a reference to, so the closure carries its own `unsafe` and the loop
    // below takes it as an ordinary one.
    let answered = || unsafe { (*frame_ptr).returned || chan.rpc.closed };
    // SAFETY: the main loop and the channel's queue are both live.
    unsafe { process_events_until(main_loop.ptr(), chan.events, -1, answered) };
    chan.rpc.call_stack.pop();

    if !frame.returned {
        *err = api_error!(kErrorTypeException, "Invalid channel: {id}");
        unsafe { channel_decref(chan.as_ptr()) };
        return NIL;
    }
    if frame.errored {
        // SAFETY: the result the decoder placed in the frame, and its arena.
        unsafe { report_call_error(err, &frame.result) };
        unsafe { arena_mem_free(frame.result_mem) };
        frame.result_mem = ptr::null_mut();
    }
    // SAFETY: the reference taken above, and the caller's out-parameter.
    unsafe { channel_decref(chan.as_ptr()) };
    unsafe { *result_mem = frame.result_mem };
    if frame.errored { NIL } else { frame.result }
}

/// Turns a peer's error payload into an `Error`.
///
/// The wire form is either a bare string or the `[type, message]` pair the
/// protocol prescribes; anything else, including a type outside the two the
/// API defines, is reported as "unknown error" rather than trusted.
///
/// # Safety
/// `err` points at a writable `Error` and `result` is a live decoded object.
unsafe fn report_call_error(err: &mut Error, result: &Object) {
    // SAFETY: the caller's error slot and decoded result. Every union read
    // below is guarded by the `type_0` it belongs to.
    if result.type_0 == kObjectTypeString {
        let text = unsafe { result.data.string }.data();
        // SAFETY: the message is a NUL-terminated string.
        unsafe { *err = err_msg_ptr(kErrorTypeException, text) };
        return;
    }
    if result.type_0 == kObjectTypeArray {
        let array = unsafe { result.data.array };
        if array.size == 2 {
            let kind = unsafe { &*array.items };
            let message = unsafe { &*array.items.add(1) };
            if kind.type_0 == kObjectTypeInteger
                && (unsafe { kind.data.integer } == Integer::from(kErrorTypeException)
                    || unsafe { kind.data.integer } == Integer::from(kErrorTypeValidation))
                && message.type_0 == kObjectTypeString
            {
                let kind = crate::narrow::number_as_int(unsafe { kind.data.integer });
                // SAFETY: the message is a NUL-terminated string.
                unsafe { *err = err_msg_ptr(kind, message.data.string.data()) };
                return;
            }
        }
    }
    *err = Error::exception(c"unknown error");
}

/// Hands an already-encoded message to a channel. Takes ownership of `buffer`.
///
/// # Safety
/// `buffer` is a live write buffer this call takes over.
pub unsafe fn rpc_write_raw(id: uint64_t, buffer: *mut WBuffer) -> bool {
    // SAFETY: the channel table is live whenever the editor is.
    match unsafe { find_rpc_channel(id) } {
        // SAFETY: the caller's buffer, handed on to the channel.
        Some(chan) => unsafe { channel_write(chan, buffer) },
        // SAFETY: as above; nobody took it, so it is released here.
        None => {
            unsafe { wstream_release_wbuffer(buffer) };
            false
        }
    }
}

/// Sends `name` to every RPC channel at once.
///
/// One pass over the packer serves all of them: [`packer_buffer_finish`] hands
/// the same [`WBuffer`] to each, which is why the buffer is created with their
/// count as its reference count.
///
/// [`packer_buffer_finish`]: envelope
///
/// # Safety
/// [`rpc_send_event`]'s contract.
unsafe fn broadcast_event(name: *const c_char, args: Array) {
    // SAFETY: the channel table's values are live channels.
    let mut chans: Vec<*mut Channel> = channels
        .with(SlotTable::snapshot_values)
        .into_iter()
        .filter(|&channel| unsafe { (*channel).is_rpc })
        .collect();
    if !chans.is_empty() {
        // SAFETY: the caller's name and arguments, and the channels just
        // collected, which nothing between here and the send can free.
        unsafe { serialize_request(&mut chans, 0, name, args) };
    }
}

/// Queues an encoded message on `chan`'s transport. Takes ownership of
/// `buffer`.
///
/// An internal channel has no transport: the bytes go straight back into the
/// decoder, either through the channel's queue or — for a channel with no
/// queue at all — on the spot.
///
/// # Safety
/// `buffer` is a live write buffer this call takes over.
unsafe fn channel_write(chan: Chan, buffer: *mut WBuffer) -> bool {
    if chan.rpc.closed {
        // SAFETY: nobody took the buffer.
        unsafe { wstream_release_wbuffer(buffer) };
        return false;
    }
    let mut err = 0;
    if chan.streamtype == kChannelStreamInternal {
        // SAFETY: the reference is dropped by `internal_read_event`, which
        // also releases the buffer.
        unsafe { channel_incref(chan.as_ptr()) };
        let mut argv = [chan.as_ptr().cast::<c_void>(), buffer.cast::<c_void>()];
        if chan.events.is_null() {
            unsafe { internal_read_event(argv.as_mut_ptr()) };
        } else {
            let mut event = one_arg_event(Some(internal_read_event), argv[0]);
            event.argv[1] = argv[1];
            unsafe { multiqueue_put_event(chan.events, event) };
        }
    } else {
        // SAFETY: an RPC channel that is not internal has an in-stream.
        err = unsafe { wstream_write(channel_instream(chan.as_ptr()), buffer) };
    }

    if err != 0 {
        let mut buf = [0 as c_char; CLOSE_MSG_MAX];
        // SAFETY: the buffer is `CLOSE_MSG_MAX` writable bytes and the verbs
        // match the arguments; `uv_strerror` answers a static string.
        let (into, cap) = (buf.as_mut_ptr(), buf.len());
        let fmt = c"ch %lu: stream write failed: %s. RPC canceled; closing channel".as_ptr();
        let why = unsafe { uv_strerror(err) };
        unsafe { crate::os::cshim::snprintf(into, cap, fmt, chan.id, why) };
        let level = if err == UV_EPIPE {
            LOGLVL_INF
        } else {
            LOGLVL_ERR
        };
        // SAFETY: `buf` is NUL-terminated and outlives the call.
        unsafe { chan_close_on_err(chan, buf.as_mut_ptr(), level) };
    }
    err == 0
}

/// Feeds an internal channel's own message back through its decoder.
///
/// # Safety
/// `argv[0]` is the live channel [`channel_write`] took a reference to and
/// `argv[1]` the write buffer it handed over; both are released here.
unsafe extern "C" fn internal_read_event(argv: *mut *mut c_void) {
    // SAFETY: `channel_write` built this argument vector: a live channel that
    // it took a reference to, and a write buffer it handed over.
    let (chan, buffer) = unsafe {
        (
            Chan::new((*argv).cast::<Channel>()),
            (*argv.add(1)).cast::<WBuffer>(),
        )
    };
    // SAFETY: the channel is an RPC endpoint and the buffer is live.
    let read_size = unsafe {
        let p = chan.unpacker();
        p.read_ptr = (*buffer).data;
        p.read_size = (*buffer).size;
        parse_msgpack(chan);
        p.read_size
    };

    // The peer is this process, so a message it could not finish decoding is
    // a bug here rather than a protocol violation.
    if read_size != 0 && !chan.rpc.closed {
        // SAFETY: a static string, and the channel is still live.
        let why = c"internal channel: internal error".as_ptr().cast_mut();
        unsafe { chan_close_on_err(chan, why, LOGLVL_ERR) };
    }
    // SAFETY: the reference and the buffer `channel_write` handed over.
    unsafe { channel_decref(chan.as_ptr()) };
    unsafe { wstream_release_wbuffer(buffer) };
}

// ---------------------------------------------------------------------------
// Client info
// ---------------------------------------------------------------------------

/// Records what `nvim_set_client_info` was told, and classifies the peer.
///
/// The classification decides how responses are matched (see
/// [`CallStack::find`]), so a peer that claims `msgpack-rpc` is taken at its
/// word.
///
/// # Safety
/// `info` is a live dict this call takes ownership of.
pub unsafe fn rpc_set_client_info(id: uint64_t, info: Dict) {
    // SAFETY: the channel table is live whenever the editor is.
    let mut chan =
        unsafe { find_rpc_channel(id) }.expect("client info for a channel that is not rpc");
    // SAFETY: the dict being replaced was owned by the channel.
    unsafe { api_free_dict(chan.rpc.info) };
    chan.rpc.info = info;
    // SAFETY: the dict was just stored on this channel, so its strings are
    // live for the classification.
    let name = unsafe { crate::cstr::at_opt(get_client_info(chan.as_ptr(), c"type".as_ptr())) };
    chan.rpc.client_type = classify_client(name);
    // SAFETY: the channel is live.
    unsafe { channel_info_changed(chan.as_ptr(), false) };
}

/// The string value `key` has in this channel's client info, or null.
///
/// # Safety
/// `chan` points at a live `Channel` and `key` is a NUL-terminated string.
pub unsafe fn get_client_info(chan: *mut Channel, key: *const c_char) -> *const c_char {
    // SAFETY: the caller's channel and key, and an info dict whose entries are
    // live for as long as it is.
    if !unsafe { (*chan).is_rpc } {
        return ptr::null();
    }
    let info = unsafe { (*chan).rpc.info };
    let key = unsafe { CStr::from_ptr(key) };
    for i in 0..info.size {
        let item = unsafe { &*info.items.add(i) };
        if item.value.type_0 == kObjectTypeString
            && unsafe { CStr::from_ptr(item.key.data()) } == key
        {
            return unsafe { item.value.data.string }.data();
        }
    }
    ptr::null()
}
