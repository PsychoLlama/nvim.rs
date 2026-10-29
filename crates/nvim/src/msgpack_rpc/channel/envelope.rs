#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The msgpack-rpc envelope: `[type, id, method, args]` and its answers.
//!
//! Every message is packed into a fresh arena block and handed to
//! [`channel_write`] once per addressee. The packer's flush hook sends the
//! block early when a message outgrows it, so a large notification travels as
//! a series of writes rather than one big allocation.

use crate::api::private::dispatch_wrappers::handle_nvim_paste;
use crate::api::ui::remote_ui_flush_incomplete;
use crate::event::wstream::SharedBuffer;
use crate::memory::{block_vec, recycle_block};
use crate::msgpack_rpc::packer::{
    Addressees, mpack_array, mpack_integer, mpack_object, mpack_object_array, mpack_str, mpack_uint,
};
use crate::types::{
    ApiDispatchFn, Array, Error, Integer, MessageType, MsgpackRpcRequestHandler, Object,
    PackerBuffer, String_0, kErrorTypeNone, uint32_t, uint64_t,
};

use super::known::*;
use super::{channel_write, trace};

/// Packs a request (with an id) or a notification (without one) and sends it
/// to each of `chans`.
pub(crate) fn serialize_request(
    chans: Addressees,
    request_id: uint32_t,
    method: &[u8],
    mut args: Array,
) {
    let mut packer = packer_buffer_init(chans);
    let is_request = request_id != 0;
    mpack_array(&mut packer, if is_request { 4 } else { 3 });
    let kind = if is_request {
        kMessageTypeRequest
    } else {
        kMessageTypeNotification
    };
    packer.push_byte(kind.to_le_bytes()[0]);
    if is_request {
        mpack_uint(&mut packer, request_id);
    }
    mpack_str(method, &mut packer);
    mpack_object_array(&mut args, &mut packer);
    packer_buffer_finish(&mut packer);
}

/// Answers a request, or reports a failed notification.
///
/// A notification has no id to answer, so its error travels as an
/// `nvim_error_event` notification back the other way — except for
/// `nvim_paste`, whose failures are shown to this editor's user instead,
/// because the UI that sent the paste has nothing useful to do with them.
///
/// The error is *rendered*, not reported: this reads the failure the caller
/// already has rather than answering with one of its own.
pub fn serialize_response(
    channel_id: uint64_t,
    handler: MsgpackRpcRequestHandler,
    type_0: MessageType,
    response_id: uint32_t,
    err: &Error,
    arg: &mut Object,
) {
    let err_type = err.kind();
    let errored = err_type != kErrorTypeNone;

    if errored && type_0 == kMessageTypeNotification {
        report_failed_notification(channel_id, handler, err);
        return;
    }

    let mut packer = packer_buffer_init(Addressees::One(channel_id));
    mpack_array(&mut packer, 4);
    packer.push_byte(kMessageTypeResponse.to_le_bytes()[0]);
    mpack_uint(&mut packer, response_id);
    if errored {
        mpack_array(&mut packer, 2);
        mpack_integer(&mut packer, Integer::from(err_type));
        mpack_str(err.message_or_empty().to_bytes(), &mut packer);
        packer.push_byte(wire::NIL);
    } else {
        packer.push_byte(wire::NIL);
        mpack_object(arg, &mut packer);
    }
    packer_buffer_finish(&mut packer);
    trace::log_response(trace::SEND, channel_id, errored, response_id);
}

/// Reports a notification that failed, which has no response to fail in.
fn report_failed_notification(
    channel_id: uint64_t,
    handler: MsgpackRpcRequestHandler,
    err: &Error,
) {
    let is_paste = handler
        .fn_0
        .is_some_and(|f| core::ptr::fn_addr_eq(f, handle_nvim_paste as ApiDispatchFn));
    if is_paste {
        let msg = err.message_or_empty().to_string_lossy();
        crate::semsg!("paste: {msg}");
        return;
    }

    let args = Array::from(vec![
        Object::Integer(Integer::from(err.kind())),
        Object::string(String_0::from_cstr(err.message_or_empty())),
    ]);
    serialize_request(Addressees::One(channel_id), 0, b"nvim_error_event", args);
}

/// The msgpack encoding of nil, which is what the unused half of the
/// `[type, id, error, result]` envelope carries. Nested so it stays out of
/// the flat namespace the unit-test header generator collects constants into.
mod wire {
    pub(super) const NIL: u8 = 0xc0;
}

/// Opens a packer whose flush hook writes to every channel in `chans`.
///
/// Any UI on one of them is flushed first: its own event stream and this
/// message share the channel, and a half-written UI event may not be
/// interleaved with one.
fn packer_buffer_init(chans: Addressees) -> PackerBuffer {
    for &chan in chans.as_slice() {
        remote_ui_flush_incomplete(chan);
    }
    let mut packer = PackerBuffer::new(block_vec(), Some(channel_flush_callback));
    packer.channels = chans;
    packer
}

/// Sends whatever the packer has accumulated, once per channel: one buffer,
/// a reference per addressee, freed when the last write completes.
fn packer_buffer_finish(packer: &mut PackerBuffer) {
    let block = packer.take();
    if block.is_empty() {
        recycle_block(block);
        return;
    }
    let buf = SharedBuffer::from_vec(block);
    // Every addressee but the last takes a reference of its own; the last
    // takes this one.
    if let Some((&last, others)) = packer.channels.as_slice().split_last() {
        for &chan in others {
            channel_write(chan, buf.clone());
        }
        channel_write(last, buf);
    }
}

/// The packer ran out of room: send what there is and start a fresh block.
fn channel_flush_callback(packer: &mut PackerBuffer) {
    packer_buffer_finish(packer);
    let chans = core::mem::take(&mut packer.channels);
    *packer = packer_buffer_init(chans);
}
