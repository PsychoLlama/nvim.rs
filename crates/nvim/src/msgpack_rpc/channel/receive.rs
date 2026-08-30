#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The receiving half of an RPC channel: the read callback, the drain loop,
//! and where each decoded message goes.
//!
//! Split out of [`super`] because the two halves of the channel barely touch:
//! everything here starts at bytes the transport handed over and ends at a
//! handler, a waiting call frame or a closed channel.

use core::ffi::{c_char, c_int, c_void};
use core::{mem, ptr};

use crate::api::private::dispatch_wrappers::{handle_nvim_get_mode, handle_nvim_ui_try_resize};
use crate::api::private::helpers::{api_free_object, api_set_error};
use crate::channel::{channel_decref, channel_incref};
use crate::event::r#loop::one_arg_event;
use crate::event::multiqueue::{event_create_oneshot, multiqueue_put_event};
use crate::log::{LOGLVL_DBG, LOGLVL_ERR, LOGLVL_INF, logmsg};
use crate::main::{ch_before_blocking_events, resize_events, ui_client_attached};
use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free, xfree, xmalloc};
use crate::msgpack_rpc::unpacker::unpacker_advance;
use crate::os::input::input_blocking;
use crate::types::{
    Arena, Array, Channel, Error, MessageType, MsgpackRpcRequestHandler, Object, RStream, Unpacker,
    kErrorTypeException, kObjectTypeArray, kObjectTypeNil, size_t, uint32_t, uint64_t,
};
use crate::ui_client::ui_client_event_raw_line;

use super::envelope::serialize_response;
use super::known::*;
use super::{Chan, NIL, RequestEvent, chan_close_on_err, trace};

// ---------------------------------------------------------------------------
// Receiving
// ---------------------------------------------------------------------------

/// The channel's read callback: decodes as much as the buffer holds and
/// reports how many bytes were consumed.
///
/// # Safety
/// libuv's contract: `rbuf` is `c` readable bytes and `data` is the channel
/// [`rpc_start`] registered.
pub(super) unsafe fn receive_msgpack(
    stream: *mut RStream,
    rbuf: *const c_char,
    c: size_t,
    data: *mut c_void,
    eof: bool,
) -> size_t {
    // SAFETY: the channel this callback was registered with.
    let chan = unsafe { Chan::new(data.cast::<Channel>()) };
    // SAFETY: as above; the reference is dropped at the end.
    unsafe { channel_incref(chan.as_ptr()) };
    let id = chan.id;
    // SAFETY: the verbs match the arguments.
    let fmt = c"ch %lu: parsing %zu bytes from msgpack Stream: %p";
    let from = stream.cast::<c_void>();
    unsafe { logmsg!(LOGLVL_DBG, c"receive_msgpack", 211, fmt, id, c, from) };

    let mut consumed = 0;
    if c > 0 {
        // SAFETY: the caller's buffer, which stays readable for this call.
        let p = unsafe { chan.unpacker() };
        p.read_ptr = rbuf;
        p.read_size = c;
        unsafe { parse_msgpack(chan) };
        // A rejected message leaves `read_size` meaningless; consuming
        // nothing keeps the buffer for the close path to report on.
        if p.state >= 0 {
            consumed = c - p.read_size;
        }
    }
    if eof {
        let mut buf = [0 as c_char; CLOSE_MSG_MAX];
        // SAFETY: the buffer is `CLOSE_MSG_MAX` writable bytes, the verbs
        // match, and the result is NUL-terminated.
        let (into, cap) = (buf.as_mut_ptr(), buf.len());
        let fmt = c"ch %lu was closed by the peer".as_ptr();
        unsafe { crate::os::cshim::snprintf(into, cap, fmt, id) };
        unsafe { chan_close_on_err(chan, buf.as_mut_ptr(), LOGLVL_INF) };
    }
    // SAFETY: the reference taken above.
    unsafe { channel_decref(chan.as_ptr()) };
    consumed
}

/// Drains whole messages out of the unpacker and routes each one.
///
/// # Safety
/// `chan` has been through [`rpc_start`] and its unpacker has been pointed at
/// the bytes to decode.
pub(super) unsafe fn parse_msgpack(chan: Chan) {
    // SAFETY: the caller's channel and its decoder.
    let p = unsafe { chan.unpacker() };
    while unsafe { unpacker_advance(p) } {
        match p.type_0 {
            kMessageTypeRedrawEvent => {
                unsafe { dispatch_redraw(p) };
                unsafe { arena_mem_free(arena_finish(&raw mut p.arena)) };
            }
            kMessageTypeResponse => {
                if !unsafe { complete_call(chan, p) } {
                    return;
                }
            }
            _ => {
                if !unsafe { dispatch_incoming(chan, p) } {
                    return;
                }
            }
        }
    }
    if p.state < 0 {
        let why = p.unpack_error.message_or_empty().as_ptr().cast_mut();
        // SAFETY: the channel is live and the message is the decoder's own.
        unsafe { chan_close_on_err(chan, why, LOGLVL_INF) };
        p.unpack_error.clear();
    }
}

/// Applies a `redraw` batch from the server this UI client is attached to.
///
/// `grid_line` is decoded straight into an event by the unpacker (it is by far
/// the most frequent one), so it is offered first and separately.
///
/// # Safety
/// `p` points at a live unpacker holding a decoded redraw event.
unsafe fn dispatch_redraw(p: &mut Unpacker) {
    if !ui_client_attached.get() {
        return;
    }
    if p.has_grid_line_event {
        // SAFETY: the event the unpacker just filled in.
        unsafe { ui_client_event_raw_line(&raw mut p.grid_line_event) };
        p.has_grid_line_event = false;
    } else if p.result.type_0 == kObjectTypeArray
        && let Some(handler) = p.ui_handler.fn_0
    {
        // SAFETY: `result` is an array, so the union read matches, and the
        // handler was resolved from the event name by the unpacker.
        unsafe { handler(p.result.data.array) };
    }
}

/// Hands a response to the call that is waiting for it.
///
/// Returns false when the response could not be placed, in which case the
/// channel has been closed and decoding must stop.
///
/// # Safety
/// `chan` is live and `p` is its decoder, holding a decoded response.
unsafe fn complete_call(chan: Chan, p: &mut Unpacker) -> bool {
    let stack = &chan.rpc.call_stack;
    let frame = if chan.rpc.client_type == kClientTypeMsgpackRpc {
        stack.find(p.request_id)
    } else {
        stack.top_matching(p.request_id)
    };
    let Some(frame) = frame else {
        let mut buf = [0 as c_char; CLOSE_MSG_MAX];
        // SAFETY: the buffer is `CLOSE_MSG_MAX` writable bytes and the verbs
        // match the arguments.
        let (into, cap) = (buf.as_mut_ptr(), buf.len());
        let fmt = c"ch %lu (type=%u) returned a response with an unknown request id %u. Ensure the client is properly synchronized"
            .as_ptr();
        let kind = chan.rpc.client_type.cast_unsigned();
        let want = p.request_id;
        unsafe { crate::os::cshim::snprintf(into, cap, fmt, chan.id, kind, want) };
        unsafe { chan_close_on_err(chan, buf.as_mut_ptr(), LOGLVL_ERR) };
        return false;
    };

    // SAFETY: the frame is a `rpc_send_call` stack frame that is still on the
    // call stack, so it outlives this write.
    unsafe { (*frame).returned = true };
    unsafe { (*frame).errored = p.error.type_0 != kObjectTypeNil };
    unsafe { (*frame).result = if (*frame).errored { p.error } else { p.result } };
    unsafe { (*frame).result_mem = arena_finish(&raw mut p.arena) };
    trace::log_response(
        trace::RECV,
        chan.id,
        unsafe { (*frame).errored },
        p.request_id,
    );
    true
}

/// Routes a request or notification to its handler.
///
/// Returns false when the message was malformed and the channel was closed.
///
/// # Safety
/// `chan` is live and `p` is its decoder, holding a decoded request.
unsafe fn dispatch_incoming(chan: Chan, p: &mut Unpacker) -> bool {
    let req_id = (p.type_0 != kMessageTypeNotification).then_some(p.request_id);
    // SAFETY: the handler name is either null or a static string.
    unsafe { trace::log_call(trace::RECV, chan.id, req_id, p.handler.name) };

    if p.result.type_0 != kObjectTypeArray {
        // SAFETY: a static string, and the channel is live.
        let why = c"msgpack-rpc request args must be an array"
            .as_ptr()
            .cast_mut();
        unsafe { chan_close_on_err(chan, why, LOGLVL_ERR) };
        return false;
    }
    // SAFETY: `result` is an array, so the union read matches.
    let args = unsafe { p.result.data.array };
    // SAFETY: the caller's channel and decoder.
    unsafe { handle_request(chan, p, args) };
    true
}

/// Decides where a request runs: here, on the channel's queue, or on the queue
/// that is drained just before the editor blocks for input.
///
/// # Safety
/// [`dispatch_incoming`]'s contract, and `args` is the decoded argument array.
unsafe fn handle_request(chan: Chan, p: &mut Unpacker, args: Array) {
    debug_assert!(p.type_0 == kMessageTypeRequest || p.type_0 == kMessageTypeNotification);

    // The decoder could not resolve a handler, so `unpack_error` says why.
    let Some(handler_fn) = p.handler.fn_0 else {
        let why = p.unpack_error.message_or_empty().as_ptr().cast_mut();
        let (handler, kind, id) = (p.handler, p.type_0, p.request_id);
        // SAFETY: the channel is live and the message is the decoder's own.
        unsafe { send_error(chan, handler, kind, id, why) };
        p.unpack_error.clear();
        unsafe { arena_mem_free(arena_finish(&raw mut p.arena)) };
        return;
    };

    // SAFETY: `xmalloc` hands back `size_of::<RequestEvent>()` writable bytes,
    // and the reference taken here is dropped by `request_event`.
    let evdata = unsafe {
        let evdata = xmalloc(size_of::<RequestEvent>()).cast::<RequestEvent>();
        evdata.write(RequestEvent {
            type_0: p.type_0,
            channel: chan.as_ptr(),
            handler: p.handler,
            args,
            request_id: p.request_id,
            // The event takes the arena the request was unpacked into and
            // frees it when it has run.
            used_mem: mem::replace(&mut p.arena, ARENA_EMPTY),
        });
        channel_incref(chan.as_ptr());
        evdata
    };

    // The event owns `evdata` from here on.
    let event = one_arg_event(Some(request_event), evdata.cast::<c_void>());
    if p.handler.fast {
        // A "fast" handler may run from the read callback — except
        // `nvim_get_mode`, whose answer is only meaningful once the editor is
        // about to wait for input.
        let is_get_mode = ptr::fn_addr_eq(
            handler_fn,
            handle_nvim_get_mode as unsafe fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        );
        // SAFETY: either queue is live, and running the event here consumes
        // `evdata` exactly once.
        if is_get_mode && !input_blocking() {
            unsafe { multiqueue_put_event(ch_before_blocking_events.get(), event) };
        } else {
            let mut argv = [evdata.cast::<c_void>()];
            unsafe { request_event(argv.as_mut_ptr()) };
        }
        return;
    }

    // A resize has to be seen by whichever of the two queues is drained
    // first, and run only once; a one-shot event does exactly that.
    let is_resize = ptr::fn_addr_eq(
        handler_fn,
        handle_nvim_ui_try_resize as unsafe fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
    );
    if is_resize {
        // SAFETY: both queues are live, and the one-shot runs the event once
        // however many queues reach it first.
        let ev = event_create_oneshot(event, 2);
        unsafe { multiqueue_put_event(chan.events, ev.clone()) };
        unsafe { multiqueue_put_event(resize_events.get(), ev) };
        return;
    }
    // A method name is at most `protocol::METHOD_NAME_MAX` bytes, so the
    // `%.*s` precision always fits; the clamp is for a shape the header
    // parser cannot produce.
    let name_len = c_int::try_from(p.method_name_len).unwrap_or(c_int::MAX);
    let name = p.handler.name;
    // SAFETY: the channel's queue is live, and the handler name is a static
    // string the verbs match.
    unsafe { multiqueue_put_event(chan.events, event) };
    let fmt = c"RPC: scheduled %.*s";
    unsafe { logmsg!(LOGLVL_DBG, c"handle_request", 347, fmt, name_len, name) };
}

/// Runs one dispatched request and answers it.
///
/// # Safety
/// `argv[0]` is the `RequestEvent` [`handle_request`] allocated, which this
/// call takes ownership of along with the channel reference it holds.
unsafe extern "C" fn request_event(argv: *mut *mut c_void) {
    // SAFETY: `handle_request` built this argument vector around one
    // `xmalloc`ed `RequestEvent`, whose channel it holds a reference to.
    let (e, chan) = unsafe {
        let e = (*argv).cast::<RequestEvent>();
        (e, Chan::new((*e).channel))
    };
    // SAFETY: as above.
    let (handler, type_0, request_id, args) =
        unsafe { ((*e).handler, (*e).type_0, (*e).request_id, (*e).args) };
    let mut error = Error::none();

    // A channel closed while the request sat on a queue is simply dropped —
    // there is nowhere left to send the answer.
    if !chan.rpc.closed {
        // SAFETY: the handler was resolved from the method name, and the
        // arena and error slot are this call's own.
        let mem = unsafe { &raw mut (*e).used_mem };
        let result = unsafe {
            handler.fn_0.expect("dispatched with a handler")(chan.id, args, mem, &raw mut error)
        };
        // A notification is only answered when it failed, and then with
        // `nvim_error_event` rather than a response.
        if type_0 == kMessageTypeRequest || error.is_set() {
            let mut answer = result;
            // SAFETY: the channel is live and both slots are stack locals.
            let (to, err, out) = (chan.as_ptr(), &raw mut error, &raw mut answer);
            unsafe { serialize_response(to, handler, type_0, request_id, err, out) };
        }
        if handler.ret_alloc {
            // SAFETY: the handler said it allocated the result.
            unsafe { api_free_object(result) };
        }
    }
    // SAFETY: the arena, the reference and the allocation this event owned.
    unsafe { arena_mem_free(arena_finish(&raw mut (*e).used_mem)) };
    unsafe { channel_decref(chan.as_ptr()) };
    unsafe { xfree(e.cast::<c_void>()) };
    error.clear();
}

/// Answers a request that never reached a handler.
///
/// # Safety
/// `chan` is live and `err` is a NUL-terminated string.
unsafe fn send_error(
    chan: Chan,
    handler: MsgpackRpcRequestHandler,
    type_0: MessageType,
    id: uint32_t,
    err: *mut c_char,
) {
    let mut e = Error::none();
    let mut answer = NIL;
    // SAFETY: the caller's message, and two stack locals.
    unsafe { api_set_error(&raw mut e, kErrorTypeException, c"%s".as_ptr(), err) };
    let (to, err, out) = (chan.as_ptr(), &raw mut e, &raw mut answer);
    unsafe { serialize_response(to, handler, type_0, id, err, out) };
    e.clear();
}
