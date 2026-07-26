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
//! [`unpacker`]: crate::src::nvim::msgpack_rpc::unpacker
//! [`packer`]: crate::src::nvim::msgpack_rpc::packer

use core::ffi::{CStr, c_char, c_int, c_void};
use core::{mem, ptr};

use crate::src::nvim::api::private::dispatch::{handle_nvim_get_mode, handle_nvim_ui_try_resize};
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_dict, api_free_object, api_set_error, arena_string, cstr_as_string,
};
use crate::src::nvim::api::ui::remote_ui_disconnect;
use crate::src::nvim::channel::{
    channel_close, channel_decref, channel_incref, channel_info_changed, find_channel,
};
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::event::r#loop::{one_arg_event, process_events_until};
use crate::src::nvim::event::multiqueue::{
    event_create_oneshot, multiqueue_new_child, multiqueue_put_event,
};
use crate::src::nvim::event::proc::exit_on_closed_chan;
use crate::src::nvim::event::rstream::rstream_start;
use crate::src::nvim::event::wstream::{wstream_release_wbuffer, wstream_write};
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{
    ch_before_blocking_events, channels, main_loop, resize_events, ui_client_attached,
    ui_client_channel_id, ui_client_error_exit,
};
use crate::src::nvim::memory::{
    ARENA_EMPTY, arena_finish, arena_mem_free, xcalloc, xfree, xmalloc,
};
use crate::src::nvim::msgpack_rpc::unpacker::{unpacker_advance, unpacker_init, unpacker_teardown};
use crate::src::nvim::os::input::input_blocking;
use crate::src::nvim::types::{
    Arena, ArenaMem, Array, Channel, ChannelCallFrame, ChannelPart, ChannelStreamType, ClientType,
    Dict, Error, ErrorType, Integer, MessageType, MsgpackRpcRequestHandler, Object, ObjectType,
    RStream, Stream, Unpacker, WBuffer, size_t, uint32_t, uint64_t,
};
use crate::src::nvim::ui_client::{ui_client_attach_to_restarted_server, ui_client_event_raw_line};

pub mod call_stack;
pub mod client;
pub mod envelope;
pub mod trace;

use call_stack::CallStack;
use client::classify_client;
use envelope::{serialize_request, serialize_response};

/// Values these belong to other modules; nested so they stay out of the flat
/// namespace the unit-test header generator collects constants into.
mod known {
    use super::{ChannelPart, ChannelStreamType, ClientType, ErrorType, MessageType, ObjectType};
    use core::ffi::c_int;

    pub const kErrorTypeNone: ErrorType = -1;
    pub const kErrorTypeException: ErrorType = 0;
    pub const kErrorTypeValidation: ErrorType = 1;

    pub const kObjectTypeNil: ObjectType = 0;
    pub const kObjectTypeInteger: ObjectType = 2;
    pub const kObjectTypeString: ObjectType = 4;
    pub const kObjectTypeArray: ObjectType = 5;

    pub const kMessageTypeRequest: MessageType = 0;
    pub const kMessageTypeResponse: MessageType = 1;
    pub const kMessageTypeNotification: MessageType = 2;
    pub const kMessageTypeRedrawEvent: MessageType = 3;

    pub const kChannelStreamProc: ChannelStreamType = 0;
    pub const kChannelStreamStdio: ChannelStreamType = 2;
    pub const kChannelStreamInternal: ChannelStreamType = 4;
    pub const kChannelPartRpc: ChannelPart = 3;

    pub const kClientTypeMsgpackRpc: ClientType = 5;

    pub const LOGLVL_DBG: c_int = 1;
    pub const LOGLVL_INF: c_int = 2;
    pub const LOGLVL_ERR: c_int = 4;

    /// libuv's "the peer hung up" error.
    pub const UV_EPIPE: c_int = -32;
    /// The arena block size the RPC packer writes into.
    pub const ARENA_BLOCK_SIZE: usize = 4096;
}

use known::*;

/// The all-nil `Object`, which is what an API call that produced nothing, or
/// failed, hands back.
const NIL: Object = Object {
    type_0: kObjectTypeNil,
    data: crate::src::nvim::types::object_data { boolean: false },
};

/// A dispatched request, in flight between the decoder and whichever queue
/// gets to run it.
///
/// It owns the arena the arguments were decoded into: the handler reads them
/// and the arena is released once the response has been packed.
#[derive(Copy, Clone)]
#[repr(C)]
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
pub unsafe fn rpc_init() {
    ch_before_blocking_events.set(multiqueue_new_child((*main_loop.ptr()).events));
}

/// Turns `channel` into an RPC endpoint and starts reading from it.
///
/// The reference taken here is the channel's own: it is dropped by
/// [`rpc_close_event`] once the peer is gone.
pub unsafe fn rpc_start(channel: *mut Channel) {
    channel_incref(channel);
    (*channel).is_rpc = true;

    let rpc = &raw mut (*channel).rpc;
    (*rpc).closed = false;
    (*rpc).unpacker = xcalloc(1, mem::size_of::<Unpacker>()) as *mut Unpacker;
    unpacker_init((*rpc).unpacker);
    (*rpc).next_request_id = 1;
    (*rpc).info = Dict {
        size: 0,
        capacity: 0,
        items: ptr::null_mut(),
    };
    (*rpc).call_stack = CallStack::new();

    // An internal channel has no transport to read from: its peer hands
    // messages straight to `rpc_write_raw`.
    if (*channel).streamtype != kChannelStreamInternal {
        let out = channel_outstream(channel);
        let in_0 = channel_instream(channel);
        logmsg(
            LOGLVL_DBG,
            ptr::null(),
            c"rpc_start".as_ptr(),
            93,
            true,
            c"rpc ch %lu in-stream=%p out-stream=%p".as_ptr(),
            (*channel).id,
            in_0 as *mut c_void,
            out as *mut c_void,
        );
        rstream_start(out, Some(receive_msgpack), channel as *mut c_void);
    }
}

/// Marks the channel closed and schedules the teardown.
///
/// The state has to survive until the queued event runs — anything still
/// walking the channel gets a `closed` flag rather than freed memory.
pub unsafe fn rpc_close(channel: *mut Channel) {
    if (*channel).rpc.closed {
        return;
    }
    (*channel).rpc.closed = true;
    multiqueue_put_event(
        (*main_loop.ptr()).fast_events,
        one_arg_event(Some(rpc_close_event), channel as *mut c_void),
    );
}

/// Drops the channel's own reference, then decides whether the editor should
/// follow the peer out.
unsafe extern "C" fn rpc_close_event(argv: *mut *mut c_void) {
    let channel = *argv as *mut Channel;
    assert!(!channel.is_null());
    channel_decref(channel);
    remote_ui_disconnect((*channel).id, ptr::null_mut(), false);

    if ui_client_channel_id.get() != 0 && (*channel).id == ui_client_channel_id.get() {
        // A `--remote-ui` client whose server went away: try to reconnect
        // before giving up.
        ui_client_attach_to_restarted_server();
        if ui_client_channel_id.get() != (*channel).id {
            return;
        }
        // An embedded server exits through its process-exit callback instead,
        // which is what carries the exit status.
        if (*channel).streamtype == kChannelStreamProc && ui_client_error_exit.get() < 0 {
            return;
        }
        exit_on_closed_chan(0);
    } else if (*channel).streamtype == kChannelStreamStdio && !(*channel).detach {
        exit_on_closed_chan(0);
    }
}

/// Releases everything `rpc_start` allocated. Called from `channel_destroy`.
pub unsafe fn rpc_free(channel: *mut Channel) {
    unpacker_teardown((*channel).rpc.unpacker);
    xfree((*channel).rpc.unpacker as *mut c_void);
    (*channel).rpc.call_stack = CallStack::new();
    api_free_dict((*channel).rpc.info);
}

/// The channel `id` is talking msgpack-rpc over, if it still is.
unsafe fn find_rpc_channel(id: uint64_t) -> *mut Channel {
    let chan = find_channel(id);
    if chan.is_null() || !(*chan).is_rpc || (*chan).rpc.closed {
        return ptr::null_mut();
    }
    chan
}

/// Fails every outstanding call with `msg`, then closes the channel.
///
/// The frames are filled from the unpacker's arena, which the reply would
/// otherwise have used; `arena_finish` hands each frame its own block, so the
/// message outlives the arena.
unsafe fn chan_close_on_err(channel: *mut Channel, msg: *mut c_char, loglevel: c_int) {
    let arena = &raw mut (*(*channel).rpc.unpacker).arena;
    for frame in (*channel).rpc.call_stack.frames() {
        if !(*frame).returned {
            (*frame).returned = true;
            (*frame).errored = true;
            (*frame).result = Object {
                type_0: kObjectTypeString,
                data: crate::src::nvim::types::object_data {
                    string: arena_string(arena, cstr_as_string(msg)),
                },
            };
            (*frame).result_mem = arena_finish(arena);
        }
    }
    channel_close((*channel).id, kChannelPartRpc, ptr::null_mut());
    logmsg(
        loglevel,
        ptr::null(),
        c"chan_close_on_err".as_ptr(),
        545,
        true,
        c"RPC: %s".as_ptr(),
        msg,
    );
}

// ---------------------------------------------------------------------------
// Sending
// ---------------------------------------------------------------------------

/// Sends a notification. `id` of 0 broadcasts to every RPC channel.
///
/// Returns false only when a named channel does not exist; a broadcast to
/// nobody still succeeds.
pub unsafe fn rpc_send_event(id: uint64_t, name: *const c_char, args: Array) -> bool {
    let mut channel = ptr::null_mut::<Channel>();
    if id != 0 {
        channel = find_rpc_channel(id);
        if channel.is_null() {
            return false;
        }
    }
    trace::log_call(
        trace::SEND,
        if channel.is_null() { 0 } else { (*channel).id },
        None,
        name,
    );
    if channel.is_null() {
        broadcast_event(name, args);
    } else {
        serialize_request(&mut channel, 1, 0, name, args);
    }
    true
}

/// Sends a request and runs the event loop until the answer arrives.
///
/// The frame lives on this stack frame; the decoder fills it in when the
/// matching response lands. `result_mem` receives the arena the result was
/// decoded into, which the caller must free.
pub unsafe fn rpc_send_call(
    id: uint64_t,
    method_name: *const c_char,
    args: Array,
    result_mem: *mut ArenaMem,
    err: *mut Error,
) -> Object {
    let mut channel = find_rpc_channel(id);
    if channel.is_null() {
        api_set_error(
            err,
            kErrorTypeException,
            c"Invalid channel: %lu".as_ptr(),
            id,
        );
        return NIL;
    }
    channel_incref(channel);

    let rpc = &raw mut (*channel).rpc;
    let request_id = (*rpc).next_request_id;
    (*rpc).next_request_id = request_id.wrapping_add(1);
    serialize_request(&mut channel, 1, request_id, method_name, args);
    trace::log_call(trace::SEND, (*channel).id, Some(request_id), method_name);

    let mut frame = ChannelCallFrame {
        request_id,
        returned: false,
        errored: false,
        result: NIL,
        result_mem: ptr::null_mut(),
    };
    let frame_ptr = &raw mut frame;
    (*rpc).call_stack.push(request_id, frame_ptr);
    process_events_until(main_loop.ptr(), (*channel).events, -1, || {
        (*frame_ptr).returned || (*rpc).closed
    });
    (*rpc).call_stack.pop();

    if !frame.returned {
        api_set_error(
            err,
            kErrorTypeException,
            c"Invalid channel: %lu".as_ptr(),
            id,
        );
        channel_decref(channel);
        return NIL;
    }
    if frame.errored {
        report_call_error(err, &frame.result);
        arena_mem_free(frame.result_mem);
        frame.result_mem = ptr::null_mut();
    }
    channel_decref(channel);
    *result_mem = frame.result_mem;
    if frame.errored { NIL } else { frame.result }
}

/// Turns a peer's error payload into an `Error`.
///
/// The wire form is either a bare string or the `[type, message]` pair the
/// protocol prescribes; anything else, including a type outside the two the
/// API defines, is reported as "unknown error" rather than trusted.
unsafe fn report_call_error(err: *mut Error, result: &Object) {
    if result.type_0 == kObjectTypeString {
        api_set_error(
            err,
            kErrorTypeException,
            c"%s".as_ptr(),
            result.data.string.data,
        );
        return;
    }
    if result.type_0 == kObjectTypeArray {
        let array = result.data.array;
        if array.size == 2 {
            let kind = &*array.items;
            let message = &*array.items.add(1);
            if kind.type_0 == kObjectTypeInteger
                && (kind.data.integer == kErrorTypeException as Integer
                    || kind.data.integer == kErrorTypeValidation as Integer)
                && message.type_0 == kObjectTypeString
            {
                api_set_error(
                    err,
                    kind.data.integer as ErrorType,
                    c"%s".as_ptr(),
                    message.data.string.data,
                );
                return;
            }
        }
    }
    api_set_error(
        err,
        kErrorTypeException,
        c"%s".as_ptr(),
        c"unknown error".as_ptr(),
    );
}

/// Hands an already-encoded message to a channel. Takes ownership of `buffer`.
pub unsafe fn rpc_write_raw(id: uint64_t, buffer: *mut WBuffer) -> bool {
    let channel = find_rpc_channel(id);
    if channel.is_null() {
        wstream_release_wbuffer(buffer);
        return false;
    }
    channel_write(channel, buffer)
}

/// Sends `name` to every RPC channel at once.
///
/// One pass over the packer serves all of them: [`packer_buffer_finish`] hands
/// the same [`WBuffer`] to each, which is why the buffer is created with their
/// count as its reference count.
unsafe fn broadcast_event(name: *const c_char, args: Array) {
    let map = channels.ptr();
    let mut chans: Vec<*mut Channel> = Vec::new();
    for i in 0..(*map).set.h.n_keys {
        let channel = *(*map).values.add(i as usize) as *mut Channel;
        if (*channel).is_rpc {
            chans.push(channel);
        }
    }
    if !chans.is_empty() {
        serialize_request(chans.as_mut_ptr(), chans.len(), 0, name, args);
    }
}

/// Queues an encoded message on `channel`'s transport. Takes ownership of
/// `buffer`.
///
/// An internal channel has no transport: the bytes go straight back into the
/// decoder, either through the channel's queue or — for a channel with no
/// queue at all — on the spot.
unsafe fn channel_write(channel: *mut Channel, buffer: *mut WBuffer) -> bool {
    if (*channel).rpc.closed {
        wstream_release_wbuffer(buffer);
        return false;
    }
    let mut err = 0;
    if (*channel).streamtype == kChannelStreamInternal {
        channel_incref(channel);
        let mut argv = [channel as *mut c_void, buffer as *mut c_void];
        if (*channel).events.is_null() {
            internal_read_event(argv.as_mut_ptr());
        } else {
            let mut event = one_arg_event(Some(internal_read_event), argv[0]);
            event.argv[1] = argv[1];
            multiqueue_put_event((*channel).events, event);
        }
    } else {
        err = wstream_write(channel_instream(channel), buffer);
    }

    if err != 0 {
        let mut buf = [0 as c_char; 256];
        crate::src::nvim::os::libc::snprintf(
            buf.as_mut_ptr(),
            buf.len(),
            c"ch %lu: stream write failed: %s. RPC canceled; closing channel".as_ptr(),
            (*channel).id,
            uv_strerror(err),
        );
        let level = if err == UV_EPIPE {
            LOGLVL_INF
        } else {
            LOGLVL_ERR
        };
        chan_close_on_err(channel, buf.as_mut_ptr(), level);
    }
    err == 0
}

/// Feeds an internal channel's own message back through its decoder.
unsafe extern "C" fn internal_read_event(argv: *mut *mut c_void) {
    let channel = *argv as *mut Channel;
    let buffer = *argv.add(1) as *mut WBuffer;
    let p = (*channel).rpc.unpacker;
    (*p).read_ptr = (*buffer).data;
    (*p).read_size = (*buffer).size;
    parse_msgpack(channel);

    // The peer is this process, so a message it could not finish decoding is
    // a bug here rather than a protocol violation.
    if (*p).read_size != 0 && !(*channel).rpc.closed {
        chan_close_on_err(
            channel,
            c"internal channel: internal error".as_ptr() as *mut c_char,
            LOGLVL_ERR,
        );
    }
    channel_decref(channel);
    wstream_release_wbuffer(buffer);
}

// ---------------------------------------------------------------------------
// Receiving
// ---------------------------------------------------------------------------

/// The channel's read callback: decodes as much as the buffer holds and
/// reports how many bytes were consumed.
unsafe extern "C" fn receive_msgpack(
    stream: *mut RStream,
    rbuf: *const c_char,
    c: size_t,
    data: *mut c_void,
    eof: bool,
) -> size_t {
    let channel = data as *mut Channel;
    channel_incref(channel);
    logmsg(
        LOGLVL_DBG,
        ptr::null(),
        c"receive_msgpack".as_ptr(),
        211,
        true,
        c"ch %lu: parsing %zu bytes from msgpack Stream: %p".as_ptr(),
        (*channel).id,
        c,
        stream as *mut c_void,
    );

    let mut consumed = 0;
    if c > 0 {
        let p = (*channel).rpc.unpacker;
        (*p).read_ptr = rbuf;
        (*p).read_size = c;
        parse_msgpack(channel);
        // A rejected message leaves `read_size` meaningless; consuming
        // nothing keeps the buffer for the close path to report on.
        if (*p).state >= 0 {
            consumed = c - (*p).read_size;
        }
    }
    if eof {
        let mut buf = [0 as c_char; 256];
        crate::src::nvim::os::libc::snprintf(
            buf.as_mut_ptr(),
            buf.len(),
            c"ch %lu was closed by the peer".as_ptr(),
            (*channel).id,
        );
        chan_close_on_err(channel, buf.as_mut_ptr(), LOGLVL_INF);
    }
    channel_decref(channel);
    consumed
}

/// Drains whole messages out of the unpacker and routes each one.
unsafe fn parse_msgpack(channel: *mut Channel) {
    let p = (*channel).rpc.unpacker;
    while unpacker_advance(p) {
        match (*p).type_0 {
            kMessageTypeRedrawEvent => {
                dispatch_redraw(p);
                arena_mem_free(arena_finish(&raw mut (*p).arena));
            }
            kMessageTypeResponse => {
                if !complete_call(channel, p) {
                    return;
                }
            }
            _ => {
                if !dispatch_incoming(channel, p) {
                    return;
                }
            }
        }
    }
    if (*p).state < 0 {
        chan_close_on_err(channel, (*p).unpack_error.msg, LOGLVL_INF);
        api_clear_error(&raw mut (*p).unpack_error);
    }
}

/// Applies a `redraw` batch from the server this UI client is attached to.
///
/// `grid_line` is decoded straight into an event by the unpacker (it is by far
/// the most frequent one), so it is offered first and separately.
unsafe fn dispatch_redraw(p: *mut Unpacker) {
    if !ui_client_attached.get() {
        return;
    }
    if (*p).has_grid_line_event {
        ui_client_event_raw_line(&raw mut (*p).grid_line_event);
        (*p).has_grid_line_event = false;
    } else if (*p).ui_handler.fn_0.is_some() && (*p).result.type_0 == kObjectTypeArray {
        (*p).ui_handler.fn_0.expect("checked above")((*p).result.data.array);
    }
}

/// Hands a response to the call that is waiting for it.
///
/// Returns false when the response could not be placed, in which case the
/// channel has been closed and decoding must stop.
unsafe fn complete_call(channel: *mut Channel, p: *mut Unpacker) -> bool {
    let stack = &(*channel).rpc.call_stack;
    let frame = if (*channel).rpc.client_type == kClientTypeMsgpackRpc {
        stack.find((*p).request_id)
    } else {
        stack.top_matching((*p).request_id)
    };
    let Some(frame) = frame else {
        let mut buf = [0 as c_char; 256];
        crate::src::nvim::os::libc::snprintf(
            buf.as_mut_ptr(),
            buf.len(),
            c"ch %lu (type=%u) returned a response with an unknown request id %u. Ensure the client is properly synchronized"
                .as_ptr(),
            (*channel).id,
            (*channel).rpc.client_type as u32,
            (*p).request_id,
        );
        chan_close_on_err(channel, buf.as_mut_ptr(), LOGLVL_ERR);
        return false;
    };

    (*frame).returned = true;
    (*frame).errored = (*p).error.type_0 != kObjectTypeNil;
    (*frame).result = if (*frame).errored {
        (*p).error
    } else {
        (*p).result
    };
    (*frame).result_mem = arena_finish(&raw mut (*p).arena);
    trace::log_response(
        trace::RECV,
        (*channel).id,
        (*frame).errored,
        (*p).request_id,
    );
    true
}

/// Routes a request or notification to its handler.
///
/// Returns false when the message was malformed and the channel was closed.
unsafe fn dispatch_incoming(channel: *mut Channel, p: *mut Unpacker) -> bool {
    let req_id = if (*p).type_0 == kMessageTypeNotification {
        None
    } else {
        Some((*p).request_id)
    };
    trace::log_call(trace::RECV, (*channel).id, req_id, (*p).handler.name);

    if (*p).result.type_0 != kObjectTypeArray {
        chan_close_on_err(
            channel,
            c"msgpack-rpc request args must be an array".as_ptr() as *mut c_char,
            LOGLVL_ERR,
        );
        return false;
    }
    let args = (*p).result.data.array;
    handle_request(channel, p, args);
    true
}

/// Decides where a request runs: here, on the channel's queue, or on the queue
/// that is drained just before the editor blocks for input.
unsafe fn handle_request(channel: *mut Channel, p: *mut Unpacker, args: Array) {
    assert!((*p).type_0 == kMessageTypeRequest || (*p).type_0 == kMessageTypeNotification);

    // The decoder could not resolve a handler, so `unpack_error` says why.
    if (*p).handler.fn_0.is_none() {
        send_error(
            channel,
            (*p).handler,
            (*p).type_0,
            (*p).request_id,
            (*p).unpack_error.msg,
        );
        api_clear_error(&raw mut (*p).unpack_error);
        arena_mem_free(arena_finish(&raw mut (*p).arena));
        return;
    }

    let evdata = xmalloc(mem::size_of::<RequestEvent>()) as *mut RequestEvent;
    (*evdata).type_0 = (*p).type_0;
    (*evdata).channel = channel;
    (*evdata).handler = (*p).handler;
    (*evdata).args = args;
    (*evdata).used_mem = (*p).arena;
    (*p).arena = ARENA_EMPTY;
    (*evdata).request_id = (*p).request_id;
    channel_incref(channel);

    let event = one_arg_event(Some(request_event), evdata as *mut c_void);
    if (*p).handler.fast {
        // A "fast" handler may run from the read callback — except
        // `nvim_get_mode`, whose answer is only meaningful once the editor is
        // about to wait for input.
        let is_get_mode = (*p).handler.fn_0.is_some_and(|f| {
            ptr::fn_addr_eq(
                f,
                handle_nvim_get_mode
                    as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
            )
        });
        if is_get_mode && !input_blocking() {
            multiqueue_put_event(ch_before_blocking_events.get(), event);
        } else {
            let mut argv = [evdata as *mut c_void];
            request_event(argv.as_mut_ptr());
        }
        return;
    }

    // A resize has to be seen by whichever of the two queues is drained
    // first, and run only once; a one-shot event does exactly that.
    let is_resize = (*p).handler.fn_0.is_some_and(|f| {
        ptr::fn_addr_eq(
            f,
            handle_nvim_ui_try_resize
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        )
    });
    if is_resize {
        let ev = event_create_oneshot(event, 2);
        multiqueue_put_event((*channel).events, ev);
        multiqueue_put_event(resize_events.get(), ev);
    } else {
        multiqueue_put_event((*channel).events, event);
        logmsg(
            LOGLVL_DBG,
            ptr::null(),
            c"handle_request".as_ptr(),
            347,
            true,
            c"RPC: scheduled %.*s".as_ptr(),
            (*p).method_name_len as c_int,
            (*p).handler.name,
        );
    }
}

/// Runs one dispatched request and answers it.
unsafe extern "C" fn request_event(argv: *mut *mut c_void) {
    let e = *argv as *mut RequestEvent;
    let channel = (*e).channel;
    let handler = (*e).handler;
    let mut error = Error {
        type_0: kErrorTypeNone,
        msg: ptr::null_mut(),
    };

    // A channel closed while the request sat on a queue is simply dropped —
    // there is nowhere left to send the answer.
    if !(*channel).rpc.closed {
        let result = handler.fn_0.expect("dispatched with a handler")(
            (*channel).id,
            (*e).args,
            &raw mut (*e).used_mem,
            &raw mut error,
        );
        // A notification is only answered when it failed, and then with
        // `nvim_error_event` rather than a response.
        if (*e).type_0 == kMessageTypeRequest || error.type_0 != kErrorTypeNone {
            let mut answer = result;
            serialize_response(
                channel,
                (*e).handler,
                (*e).type_0,
                (*e).request_id,
                &raw mut error,
                &raw mut answer,
            );
        }
        if handler.ret_alloc {
            api_free_object(result);
        }
    }
    arena_mem_free(arena_finish(&raw mut (*e).used_mem));
    channel_decref(channel);
    xfree(e as *mut c_void);
    api_clear_error(&raw mut error);
}

/// Answers a request that never reached a handler.
unsafe fn send_error(
    chan: *mut Channel,
    handler: MsgpackRpcRequestHandler,
    type_0: MessageType,
    id: uint32_t,
    err: *mut c_char,
) {
    let mut e = Error {
        type_0: kErrorTypeNone,
        msg: ptr::null_mut(),
    };
    api_set_error(&raw mut e, kErrorTypeException, c"%s".as_ptr(), err);
    let mut answer = NIL;
    serialize_response(chan, handler, type_0, id, &raw mut e, &raw mut answer);
    api_clear_error(&raw mut e);
}

// ---------------------------------------------------------------------------
// Transports
// ---------------------------------------------------------------------------

/// The stream this channel writes to.
///
/// Only defined for the three transports that have one; the internal and
/// stderr channels never reach here, because [`rpc_start`] and
/// [`channel_write`] both branch on the type first.
unsafe fn channel_instream(chan: *mut Channel) -> *mut Stream {
    match (*chan).streamtype {
        0 => &raw mut (*chan).stream.proc.in_0,
        1 => &raw mut (*chan).stream.socket.s,
        2 => &raw mut (*chan).stream.stdio.out,
        other => unreachable!("channel stream type {other} has no write stream"),
    }
}

/// The stream this channel reads from. See [`channel_instream`].
unsafe fn channel_outstream(chan: *mut Channel) -> *mut RStream {
    match (*chan).streamtype {
        0 => &raw mut (*chan).stream.proc.out,
        1 => &raw mut (*chan).stream.socket,
        2 => &raw mut (*chan).stream.stdio.in_0,
        other => unreachable!("channel stream type {other} has no read stream"),
    }
}

// ---------------------------------------------------------------------------
// Client info
// ---------------------------------------------------------------------------

/// Records what `nvim_set_client_info` was told, and classifies the peer.
///
/// The classification decides how responses are matched (see
/// [`CallStack::find`]), so a peer that claims `msgpack-rpc` is taken at its
/// word.
pub unsafe fn rpc_set_client_info(id: uint64_t, info: Dict) {
    let chan = find_rpc_channel(id);
    assert!(!chan.is_null(), "client info for a channel that is not rpc");
    api_free_dict((*chan).rpc.info);
    (*chan).rpc.info = info;
    let type_0 = get_client_info(chan, c"type".as_ptr());
    let name = if type_0.is_null() {
        None
    } else {
        Some(CStr::from_ptr(type_0))
    };
    (*chan).rpc.client_type = classify_client(name);
    channel_info_changed(chan, false);
}

/// The string value `key` has in this channel's client info, or null.
pub unsafe fn get_client_info(chan: *mut Channel, key: *const c_char) -> *const c_char {
    if !(*chan).is_rpc {
        return ptr::null();
    }
    let info = (*chan).rpc.info;
    let key = CStr::from_ptr(key);
    for i in 0..info.size {
        let item = &*info.items.add(i);
        if item.value.type_0 == kObjectTypeString && CStr::from_ptr(item.key.data) == key {
            return item.value.data.string.data;
        }
    }
    ptr::null()
}
