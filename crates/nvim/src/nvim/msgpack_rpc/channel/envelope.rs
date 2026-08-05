//! The msgpack-rpc envelope: `[type, id, method, args]` and its answers.
//!
//! Every message is packed into a fresh arena block and handed to
//! [`channel_write`] once per addressee. The packer's flush hook sends the
//! block early when a message outgrows it, so a large notification travels as
//! a series of writes rather than one big allocation.

use core::ffi::{CStr, c_char, c_void};
use core::ptr;

use crate::src::nvim::api::private::dispatch_wrappers::handle_nvim_paste;
use crate::src::nvim::api::private::helpers::{api_clear_error, cstr_as_string};
use crate::src::nvim::api::ui::remote_ui_flush_pending_data;
use crate::src::nvim::event::wstream::wstream_new_buffer;
use crate::src::nvim::memory::{alloc_block, free_block};
use crate::src::nvim::msgpack_rpc::packer::{
    mpack_array, mpack_integer, mpack_object, mpack_object_array, mpack_str, mpack_uint,
};
use crate::src::nvim::types::{
    Arena, Array, Channel, Error, Integer, MessageType, MsgpackRpcRequestHandler, Object,
    PackerBuffer, kErrorTypeNone, kObjectTypeInteger, kObjectTypeString, size_t, uint32_t,
    uint64_t,
};

use super::known::*;
use super::{channel_write, trace};

/// Packs a request (with an id) or a notification (without one) and sends it
/// to each of `nchans` channels.
pub unsafe fn serialize_request(
    chans: *mut *mut Channel,
    nchans: usize,
    request_id: uint32_t,
    method: *const c_char,
    args: Array,
) {
    let mut packer = packer_buffer_init(chans, nchans);
    let is_request = request_id != 0;
    mpack_array(&mut packer.ptr, if is_request { 4 } else { 3 });
    put_byte(
        &mut packer,
        if is_request {
            kMessageTypeRequest
        } else {
            kMessageTypeNotification
        } as c_char,
    );
    if is_request {
        mpack_uint(&mut packer.ptr, request_id);
    }
    mpack_str(cstr_as_string(method), &mut packer);
    mpack_object_array(args, &mut packer);
    packer_buffer_finish(&mut packer);
}

/// Answers a request, or reports a failed notification.
///
/// A notification has no id to answer, so its error travels as an
/// `nvim_error_event` notification back the other way — except for
/// `nvim_paste`, whose failures are shown to this editor's user instead,
/// because the UI that sent the paste has nothing useful to do with them.
pub unsafe fn serialize_response(
    channel: *mut Channel,
    handler: MsgpackRpcRequestHandler,
    type_0: MessageType,
    response_id: uint32_t,
    err: *mut Error,
    arg: *mut Object,
) {
    if (*err).type_0 != kErrorTypeNone && type_0 == kMessageTypeNotification {
        let is_paste = handler.fn_0.is_some_and(|f| {
            ptr::fn_addr_eq(
                f,
                handle_nvim_paste
                    as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
            )
        });
        if is_paste {
            let msg = CStr::from_ptr((*err).msg).to_string_lossy();
            crate::semsg!("paste: {msg}");
            api_clear_error(err);
        } else {
            let mut items = [
                Object {
                    type_0: kObjectTypeInteger,
                    data: crate::src::nvim::types::object_data {
                        integer: (*err).type_0 as Integer,
                    },
                },
                Object {
                    type_0: kObjectTypeString,
                    data: crate::src::nvim::types::object_data {
                        string: cstr_as_string((*err).msg),
                    },
                },
            ];
            let args = Array {
                size: 2,
                capacity: 2,
                items: items.as_mut_ptr(),
            };
            let mut chan = channel;
            serialize_request(&mut chan, 1, 0, c"nvim_error_event".as_ptr(), args);
        }
        return;
    }

    let mut chan = channel;
    let mut packer = packer_buffer_init(&mut chan, 1);
    mpack_array(&mut packer.ptr, 4);
    put_byte(&mut packer, kMessageTypeResponse as c_char);
    mpack_uint(&mut packer.ptr, response_id);
    let errored = (*err).type_0 != kErrorTypeNone;
    if errored {
        mpack_array(&mut packer.ptr, 2);
        mpack_integer(&mut packer.ptr, (*err).type_0 as Integer);
        mpack_str(cstr_as_string((*err).msg), &mut packer);
        put_byte(&mut packer, wire::NIL);
    } else {
        put_byte(&mut packer, wire::NIL);
        mpack_object(arg, &mut packer);
    }
    packer_buffer_finish(&mut packer);
    trace::log_response(trace::SEND, (*channel).id, errored, response_id);
}

/// The msgpack encoding of nil, which is what the unused half of the
/// `[type, id, error, result]` envelope carries. Nested so it stays out of
/// the flat namespace the unit-test header generator collects constants into.
mod wire {
    use core::ffi::c_char;

    pub const NIL: c_char = 0xc0u8 as c_char;
}

/// Writes one raw byte through the packer's cursor.
///
/// Both call sites spend it on a value the packer has no encoder for: the
/// message type, which is a bare fixint, and the nil half of the envelope.
fn put_byte(packer: &mut PackerBuffer, byte: c_char) {
    unsafe {
        *packer.ptr = byte;
        packer.ptr = packer.ptr.add(1);
    }
}

/// Opens a packer buffer whose flush hook writes to `nchans` channels.
///
/// Any UI on one of them is flushed first: its own event stream and this
/// message share the channel, and a half-written UI event may not be
/// interleaved with one.
unsafe fn packer_buffer_init(chans: *mut *mut Channel, nchans: usize) -> PackerBuffer {
    for i in 0..nchans {
        let chan = *chans.add(i);
        let ui = (*chan).rpc.ui;
        if !ui.is_null() && (*ui).incomplete_event {
            remote_ui_flush_pending_data(ui);
        }
    }
    let startptr = alloc_block() as *mut c_char;
    PackerBuffer {
        startptr,
        ptr: startptr,
        endptr: startptr.add(ARENA_BLOCK_SIZE),
        anydata: chans as *mut c_void,
        anyint: nchans as i64,
        packer_flush: Some(channel_flush_callback),
    }
}

/// Sends whatever the packer has accumulated, once per channel.
///
/// The buffer is created with the channel count as its reference count, so the
/// block is freed when the last write completes.
unsafe fn packer_buffer_finish(packer: &mut PackerBuffer) {
    let len = packer.ptr.addr() - packer.startptr.addr();
    if len == 0 {
        free_block(packer.startptr as *mut c_void);
        return;
    }
    let buf = wstream_new_buffer(
        packer.startptr,
        len,
        packer.anyint as size_t,
        Some(free_block),
    );
    let chans = packer.anydata as *mut *mut Channel;
    for i in 0..packer.anyint as usize {
        channel_write(*chans.add(i), buf);
    }
}

/// The packer ran out of room: send what there is and start a fresh block.
unsafe extern "C" fn channel_flush_callback(packer: *mut PackerBuffer) {
    packer_buffer_finish(&mut *packer);
    *packer = packer_buffer_init(
        (*packer).anydata as *mut *mut Channel,
        (*packer).anyint as usize,
    );
}
