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

//! The receiving half of an RPC channel: the read callback, the drain loop,
//! and where each decoded message goes.
//!
//! Split out of [`super`] because the two halves of the channel barely touch:
//! everything here starts at bytes the transport handed over and ends at a
//! handler, a waiting call frame or a closed channel.
//!
//! Each message is decoded, and routed as far as routing runs no user code,
//! inside one borrow of the channel ([`next_message`]); what is left — a
//! handler run on the spot, a refusal sent back, a redraw applied — runs
//! after the borrow ends, so it may reach the channel table itself.

use core::ffi::{CStr, c_int, c_void};

use crate::api::private::dispatch_wrappers::{handle_nvim_get_mode, handle_nvim_ui_try_resize};
use crate::channel::{channel_decref, with_channel};
use crate::event::multiqueue::{put_job, put_job_in, put_job_on_either};
use crate::log::{LOGLVL_DBG, LOGLVL_ERR, LOGLVL_INF, logmsg};
use crate::memory::ARENA_EMPTY;
use crate::message_fmt::{msg_addr, msg_bytes};
use crate::msgpack_rpc::channel::ch_before_blocking_events;
use crate::os::input::input_blocking;
use crate::startup::ui_client_attached;
use crate::types::{
    ApiDispatchFn, Array, Channel, Error, GridLineEvent, MessageType, MsgpackRpcRequestHandler,
    Object, RpcState, kErrorTypeException, uint32_t, uint64_t,
};
use crate::ui::state::resize_events;
use crate::ui_client::ui_client_event_raw_line;

use super::envelope::serialize_response;
use super::known::*;
use super::{chan_close_on_err, close_message, trace};

// ---------------------------------------------------------------------------
// Receiving
// ---------------------------------------------------------------------------

/// The channel's read callback: decodes as much of `bytes` as it can and
/// reports how many were consumed. `from` is the stream, for the log.
pub(crate) fn receive_msgpack(id: uint64_t, from: *const c_void, bytes: &[u8], eof: bool) -> usize {
    logmsg!(
        LOGLVL_DBG,
        c"receive_msgpack",
        211,
        "ch {}: parsing {} bytes from msgpack Stream: {}",
        id,
        bytes.len(),
        msg_addr(from)
    );

    let mut consumed = 0;
    if !bytes.is_empty() {
        let (left, rejected) = parse_msgpack_reporting(id, bytes);
        // A rejected message leaves the position meaningless; consuming
        // nothing keeps the buffer for the close path to report on.
        if !rejected {
            consumed = bytes.len() - left;
        }
    }
    if eof {
        let msg = close_message(c"ch %lu was closed by the peer", &[id.into()]);
        chan_close_on_err(id, &msg, LOGLVL_INF);
    }
    consumed
}

/// Drains whole messages out of `bytes` and routes each one; answers how
/// many bytes are left over.
pub(super) fn parse_msgpack(id: uint64_t, bytes: &[u8]) -> usize {
    parse_msgpack_reporting(id, bytes).0
}

/// [`parse_msgpack`], and whether the decoder rejected the stream.
fn parse_msgpack_reporting(id: uint64_t, bytes: &[u8]) -> (usize, bool) {
    let mut input = bytes;
    loop {
        // Messages that route without running anything -- a response, a
        // queued request -- are drained inside the one borrow.
        let step = with_channel(id, |chan| {
            loop {
                match next_message(chan, &mut input) {
                    Step::Routed => {}
                    step => break step,
                }
            }
        });
        let Some(step) = step else {
            return (input.len(), false);
        };
        match step {
            Step::Wait => return (input.len(), false),
            Step::Rejected(why) => {
                chan_close_on_err(id, why.to_bytes(), LOGLVL_INF);
                return (input.len(), true);
            }
            Step::Close(why, level) => {
                chan_close_on_err(id, &why, level);
                return (input.len(), false);
            }
            Step::Routed => unreachable!("drained above"),
            Step::Run(request) => request_event(request),
            Step::Refuse(request, error) => send_error(&request, &error),
            Step::Redraw(redraw) => dispatch_redraw(redraw),
        }
    }
}

/// What the decoder made of the next message, once it has been routed as
/// far as can be done with the channel borrowed.
enum Step {
    /// No whole message yet.
    Wait,
    /// The stream is not msgpack-rpc, and why.
    Rejected(std::ffi::CString),
    /// The message breaks the protocol: close the channel with this message.
    Close(Vec<u8>, c_int),
    /// Answered a call, or queued a request: nothing left to do.
    Routed,
    /// A fast request, to run right here.
    Run(Request),
    /// A request whose method did not resolve, and why.
    Refuse(Request, Error),
    /// A `redraw` event for this UI client.
    Redraw(Redraw),
}

/// One `redraw` event, decoded.
enum Redraw {
    /// A `grid_line`, decoded straight into the shared line buffers.
    Line(GridLineEvent),
    /// Any other event, with its arguments.
    Event(Option<fn(Array)>, Object),
}

/// A request or notification on its way to its handler. It holds a
/// reference to its channel, which [`request_event`] drops.
struct Request {
    channel: uint64_t,
    type_0: MessageType,
    handler: MsgpackRpcRequestHandler,
    args: Array,
    request_id: uint32_t,
}

/// Decode the next message out of `input` and route it.
fn next_message(chan: &mut Channel, input: &mut &[u8]) -> Step {
    let id = chan.id;
    let RpcState {
        unpacker,
        call_stack,
        client_type,
        ..
    } = &mut chan.rpc;
    let Some(p) = unpacker.as_deref_mut() else {
        return Step::Wait;
    };
    if !p.advance(input) {
        if p.state < 0 {
            let why = p.unpack_error.message_or_empty().to_owned();
            p.unpack_error.clear();
            return Step::Rejected(why);
        }
        return Step::Wait;
    }
    match p.type_0 {
        kMessageTypeRedrawEvent => Step::Redraw(if p.has_grid_line_event {
            p.has_grid_line_event = false;
            Redraw::Line(p.grid_line_event)
        } else {
            Redraw::Event(p.ui_handler.fn_0, p.result.take())
        }),
        kMessageTypeResponse => {
            // Hand the response to the call that is waiting for it.
            let frame = if *client_type == kClientTypeMsgpackRpc {
                call_stack.find(p.request_id)
            } else {
                call_stack.top_matching(p.request_id)
            };
            let Some(frame) = frame else {
                let msg = close_message(
                    c"ch %lu (type=%u) returned a response with an unknown request id %u. Ensure the client is properly synchronized",
                    &[id.into(), client_type.cast_unsigned().into(), p.request_id.into()],
                );
                return Step::Close(msg, LOGLVL_ERR);
            };
            frame.returned = true;
            frame.errored = !p.error.is_nil();
            frame.result = if frame.errored {
                p.error.take()
            } else {
                p.result.take()
            };
            trace::log_response(trace::RECV, id, frame.errored, p.request_id);
            Step::Routed
        }
        _ => {
            let req_id = (p.type_0 != kMessageTypeNotification).then_some(p.request_id);
            trace::log_call(trace::RECV, id, req_id, p.handler.name);
            let Some(args) = p.result.take().into_array() else {
                let why = b"msgpack-rpc request args must be an array".to_vec();
                return Step::Close(why, LOGLVL_ERR);
            };
            let request = Request {
                channel: id,
                type_0: p.type_0,
                handler: p.handler,
                args,
                request_id: p.request_id,
            };
            // The decoder could not resolve a handler, so `unpack_error` says
            // why.
            if request.handler.fn_0.is_none() {
                let why = core::mem::take(&mut p.unpack_error);
                return Step::Refuse(request, why);
            }
            let method_name_len = p.method_name_len;
            schedule(chan, request, method_name_len)
        }
    }
}

/// Decides where a request runs: here, on the channel's queue, or on the
/// queue that is drained just before the editor blocks for input. The
/// request takes a reference to the channel, whichever it is.
fn schedule(chan: &mut Channel, request: Request, method_name_len: usize) -> Step {
    debug_assert!(
        request.type_0 == kMessageTypeRequest || request.type_0 == kMessageTypeNotification
    );
    let handler = request.handler;
    let handler_fn = handler.fn_0.expect("a resolved handler");
    chan.refcount.retain();
    if handler.fast {
        // A "fast" handler may run from the read callback — except
        // `nvim_get_mode`, whose answer is only meaningful once the editor is
        // about to wait for input.
        let is_get_mode = core::ptr::fn_addr_eq(handler_fn, handle_nvim_get_mode as ApiDispatchFn);
        if is_get_mode && !input_blocking() {
            put_job_in(&ch_before_blocking_events, job(request));
            return Step::Routed;
        }
        return Step::Run(request);
    }

    // A resize has to be seen by whichever of the two queues is drained
    // first, and run only once.
    let is_resize = core::ptr::fn_addr_eq(handler_fn, handle_nvim_ui_try_resize as ApiDispatchFn);
    if is_resize {
        put_job_on_either(&chan.events, &resize_events, job(request));
        return Step::Routed;
    }
    put_job(&chan.events, job(request));
    // The handler's name is the method's, of which the wire carried
    // `method_name_len` bytes.
    let name = handler.name.map_or(&b""[..], CStr::to_bytes);
    let name = msg_bytes(&name[..method_name_len.min(name.len())]);
    logmsg!(LOGLVL_DBG, c"handle_request", 347, "RPC: scheduled {name}");
    Step::Routed
}

/// The queued form of a request.
fn job(request: Request) -> impl FnOnce() + 'static {
    move || request_event(request)
}

/// Applies a `redraw` event from the server this UI client is attached to.
fn dispatch_redraw(redraw: Redraw) {
    if !ui_client_attached.get() {
        return;
    }
    match redraw {
        Redraw::Line(event) => ui_client_event_raw_line(&event),
        Redraw::Event(Some(handler), args) => {
            if let Some(args) = args.into_array() {
                handler(args);
            }
        }
        Redraw::Event(None, _) => {}
    }
}

/// Runs one dispatched request and answers it, then drops the reference to
/// the channel the request held.
fn request_event(request: Request) {
    let Request {
        channel: id,
        type_0,
        handler,
        args,
        request_id,
    } = request;
    let mut error = Error::none();

    // A channel closed while the request sat on a queue is simply dropped —
    // there is nowhere left to send the answer.
    if with_channel(id, |chan| !chan.rpc.closed) == Some(true) {
        // Scratch the handler may allocate in. Nothing in the API does any
        // more -- every answer owns its storage -- so it stays empty.
        let mut arena = ARENA_EMPTY;
        // A refused call answers nil, as every wrapper's own refusal paths
        // did, and the failure travels in `error` for the response below.
        let dispatch = handler.fn_0.expect("dispatched with a handler");
        let mut answer = match dispatch(id, args, &mut arena) {
            Ok(rv) => rv,
            Err(e) => {
                error = e;
                Object::Nil
            }
        };
        debug_assert!(
            arena.cur_blk.is_null(),
            "a handler allocated into its arena"
        );
        // A notification is only answered when it failed, and then with
        // `nvim_error_event` rather than a response.
        if type_0 == kMessageTypeRequest || error.is_set() {
            serialize_response(id, handler, type_0, request_id, &error, &mut answer);
        }
    }
    channel_decref(id);
}

/// Answers a request that never reached a handler.
fn send_error(request: &Request, why: &Error) {
    let mut answer = Object::Nil;
    let e = Error::from_message(kErrorTypeException, why.message_or_empty());
    serialize_response(
        request.channel,
        request.handler,
        request.type_0,
        request.request_id,
        &e,
        &mut answer,
    );
}
