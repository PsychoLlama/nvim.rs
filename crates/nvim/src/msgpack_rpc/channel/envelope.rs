#![deny(unsafe_op_in_unsafe_fn)]
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
use crate::api::ui::remote_ui_flush_pending_data;
use crate::event::wstream::wstream_new_buffer;
use core::ffi::{c_char, c_void};
use core::{ptr, slice};

use crate::api::private::helpers::cstr_as_string;
use crate::memory::{alloc_block, free_block};
use crate::msgpack_rpc::packer::{
    mpack_array, mpack_integer, mpack_object, mpack_object_array, mpack_str, mpack_uint,
};
use crate::types::{
    Arena, Array, Channel, Error, Integer, MessageType, MsgpackRpcRequestHandler, Object,
    PackerBuffer, kErrorTypeNone, kObjectTypeInteger, kObjectTypeString, uint32_t, uint64_t,
};

use super::known::*;
use super::{Chan, channel_write, trace};

/// Packs a request (with an id) or a notification (without one) and sends it
/// to each of `chans`.
///
/// # Safety
/// Every channel in `chans` is live, `method` is a NUL-terminated string, and
/// `args` describes `args.size` live objects.
pub unsafe fn serialize_request(
    chans: &mut [*mut Channel],
    request_id: uint32_t,
    method: *const c_char,
    args: Array,
) {
    // SAFETY: the caller's channels, method name and argument array.
    let mut packer = unsafe { packer_buffer_init(chans) };
    let is_request = request_id != 0;
    mpack_array(&mut packer.ptr, if is_request { 4 } else { 3 });
    let kind = if is_request {
        kMessageTypeRequest
    } else {
        kMessageTypeNotification
    };
    unsafe { put_byte(&mut packer, kind.to_le_bytes()[0].cast_signed()) };
    if is_request {
        mpack_uint(&mut packer.ptr, request_id);
    }
    unsafe { mpack_str(cstr_as_string(method), &mut packer) };
    unsafe { mpack_object_array(args, &mut packer) };
    unsafe { packer_buffer_finish(&mut packer) };
}

/// Answers a request, or reports a failed notification.
///
/// A notification has no id to answer, so its error travels as an
/// `nvim_error_event` notification back the other way — except for
/// `nvim_paste`, whose failures are shown to this editor's user instead,
/// because the UI that sent the paste has nothing useful to do with them.
///
/// # Safety
/// `channel` is live, and `err`/`arg` point at a writable `Error` and a live
/// `Object`.
pub unsafe fn serialize_response(
    channel: *mut Channel,
    handler: MsgpackRpcRequestHandler,
    type_0: MessageType,
    response_id: uint32_t,
    err: &mut Error,
    arg: *mut Object,
) {
    let err_type = err.kind();
    let errored = err_type != kErrorTypeNone;

    if errored && type_0 == kMessageTypeNotification {
        // SAFETY: the caller's error slot and channel.
        unsafe { report_failed_notification(channel, handler, err) };
        return;
    }

    let mut chan = channel;
    // SAFETY: the caller's channel, error message and result object; the
    // packer writes into a block it owns.
    let mut packer = unsafe { packer_buffer_init(slice::from_mut(&mut chan)) };
    mpack_array(&mut packer.ptr, 4);
    let kind = kMessageTypeResponse.to_le_bytes()[0].cast_signed();
    unsafe { put_byte(&mut packer, kind) };
    mpack_uint(&mut packer.ptr, response_id);
    if errored {
        mpack_array(&mut packer.ptr, 2);
        mpack_integer(&mut packer.ptr, Integer::from(err_type));
        // SAFETY: the caller's error slot, whose message is a live string.
        let why = unsafe { cstr_as_string(err.message_or_empty().as_ptr()) };
        // SAFETY: `packer` is this frame's own.
        unsafe { mpack_str(why, &mut packer) };
        unsafe { put_byte(&mut packer, wire::NIL) };
    } else {
        unsafe { put_byte(&mut packer, wire::NIL) };
        unsafe { mpack_object(arg, &mut packer) };
    }
    unsafe { packer_buffer_finish(&mut packer) };
    trace::log_response(trace::SEND, unsafe { (*channel).id }, errored, response_id);
}

/// Reports a notification that failed, which has no response to fail in.
///
/// # Safety
/// [`serialize_response`]'s contract.
unsafe fn report_failed_notification(
    channel: *mut Channel,
    handler: MsgpackRpcRequestHandler,
    err: &mut Error,
) {
    let is_paste = handler.fn_0.is_some_and(|f| {
        ptr::fn_addr_eq(
            f,
            handle_nvim_paste as unsafe fn(uint64_t, Array, *mut Arena, &mut Error) -> Object,
        )
    });
    if is_paste {
        let msg = err.message_or_empty().to_string_lossy();
        crate::semsg!("paste: {msg}");
        err.clear();
        return;
    }

    // SAFETY: the caller's error slot. `items` lives until the request has
    // been packed, which `serialize_request` does before returning.
    let mut items = [
        Object {
            type_0: kObjectTypeInteger,
            data: crate::types::object_data {
                integer: Integer::from(err.kind()),
            },
        },
        Object {
            type_0: kObjectTypeString,
            data: crate::types::object_data {
                string: unsafe { cstr_as_string(err.message_or_empty().as_ptr()) },
            },
        },
    ];
    let args = Array {
        size: 2,
        capacity: 2,
        items: items.as_mut_ptr(),
    };
    let mut chan = channel;
    let to = slice::from_mut(&mut chan);
    unsafe { serialize_request(to, 0, c"nvim_error_event".as_ptr(), args) };
}

/// The msgpack encoding of nil, which is what the unused half of the
/// `[type, id, error, result]` envelope carries. Nested so it stays out of
/// the flat namespace the unit-test header generator collects constants into.
mod wire {
    use core::ffi::c_char;

    pub(super) const NIL: c_char = 0xc0u8.cast_signed();
}

/// Writes one raw byte through the packer's cursor.
///
/// Both call sites spend it on a value the packer has no encoder for: the
/// message type, which is a bare fixint, and the nil half of the envelope.
///
/// # Safety
/// The packer's cursor has room for one more byte, which
/// [`mpack_check_buffer`](crate::msgpack_rpc::packer::mpack_check_buffer)
/// guarantees.
unsafe fn put_byte(packer: &mut PackerBuffer, byte: c_char) {
    // SAFETY: the caller's guarantee of room.
    unsafe { *packer.ptr = byte };
    packer.ptr = unsafe { packer.ptr.add(1) };
}

/// Opens a packer buffer whose flush hook writes to every channel in `chans`.
///
/// Any UI on one of them is flushed first: its own event stream and this
/// message share the channel, and a half-written UI event may not be
/// interleaved with one.
///
/// # Safety
/// Every channel in `chans` is live, and `chans` itself outlives the packer:
/// the flush hook reads it back out of `anydata`.
unsafe fn packer_buffer_init(chans: &mut [*mut Channel]) -> PackerBuffer {
    for &chan in chans.iter() {
        // SAFETY: the caller's channels.
        let ui = unsafe { (*chan).rpc.ui };
        if !ui.is_null() && unsafe { (*ui).incomplete_event } {
            unsafe { remote_ui_flush_pending_data(ui) };
        }
    }
    // SAFETY: the block allocator takes no arguments and hands back a fresh
    // `ARENA_BLOCK_SIZE` block.
    let startptr = unsafe { alloc_block() }.cast::<c_char>();
    PackerBuffer {
        startptr,
        ptr: startptr,
        endptr: startptr.wrapping_add(ARENA_BLOCK_SIZE),
        anydata: chans.as_mut_ptr().cast::<c_void>(),
        anyint: i64::try_from(chans.len()).expect("addressee count fits an i64"),
        packer_flush: Some(channel_flush_callback),
    }
}

/// The channels a packer was opened for, as [`packer_buffer_init`] stashed
/// them.
///
/// # Safety
/// `packer` was opened by [`packer_buffer_init`] and the slice it was given is
/// still alive.
unsafe fn packer_channels<'a>(packer: &PackerBuffer) -> &'a mut [*mut Channel] {
    // SAFETY: the caller's guarantee that the slice outlives the packer.
    let base = packer.anydata.cast::<*mut Channel>();
    let n = usize::try_from(packer.anyint).expect("`packer_buffer_init` stored a count here");
    unsafe { slice::from_raw_parts_mut(base, n) }
}

/// Sends whatever the packer has accumulated, once per channel.
///
/// The buffer is created with the channel count as its reference count, so the
/// block is freed when the last write completes.
///
/// # Safety
/// [`packer_channels`]'s contract.
unsafe fn packer_buffer_finish(packer: &mut PackerBuffer) {
    let len = packer.ptr.addr() - packer.startptr.addr();
    if len == 0 {
        // SAFETY: the block `packer_buffer_init` allocated, never written to.
        unsafe { free_block(packer.startptr.cast::<c_void>()) };
        return;
    }
    // SAFETY: `startptr..ptr` is the block this packer filled, and the write
    // buffer takes one reference per addressee.
    let chans = unsafe { packer_channels(packer) };
    let buf = wstream_new_buffer(packer.startptr, len, chans.len(), Some(free_block));
    for &chan in chans.iter() {
        unsafe { channel_write(Chan::new(chan), buf) };
    }
}

/// The packer ran out of room: send what there is and start a fresh block.
///
/// # Safety
/// [`packer_channels`]'s contract, and `packer` points at a live buffer.
unsafe fn channel_flush_callback(packer: *mut PackerBuffer) {
    // SAFETY: the caller's packer, which libmpack's writer owns for the
    // duration of this call.
    let packer = unsafe { &mut *packer };
    unsafe { packer_buffer_finish(packer) };
    *packer = unsafe { packer_buffer_init(packer_channels(packer)) };
}
