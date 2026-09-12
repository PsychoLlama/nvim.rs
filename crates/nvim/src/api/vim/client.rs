//! Who is on the other end of a channel.
//!
//! `nvim_set_client_info` records a client's name, version, methods and
//! attributes against its channel, and `nvim_get_chan_info` renders that
//! back for one channel (`nvim_list_chans` for all of them).
//! `nvim_get_api_info` is what a client calls first: its own channel id
//! plus the packed api metadata.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::cstr;

/// # Safety
/// The answer's storage is the api's own: the caller frees whatever this hands
/// back.
pub unsafe fn nvim_get_api_info(channel_id: uint64_t) -> Array {
    let mut rv: Array = Array::with_capacity(2 as size_t);
    debug_assert!(
        channel_id <= 9223372036854775807 as uint64_t,
        "channel_id <= INT64_MAX"
    );
    rv.push(Object::integer(channel_id.cast_signed()));
    rv.push(api_metadata());
    rv
}

/// # Safety
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `mut version` must be a well-formed API dictionary, its
/// `size` entries initialized. `type_0` must be a well-formed API string:
/// `size` readable bytes with a NUL at `data[size]`. `methods` must be a well-
/// formed API dictionary, its `size` entries initialized. `attributes` must be
/// a well-formed API dictionary, its `size` entries initialized.
pub unsafe fn nvim_set_client_info(
    channel_id: uint64_t,
    name: String_0,
    mut version: ApiDict,
    type_0: String_0,
    methods: ApiDict,
    attributes: ApiDict,
) {
    let mut info: ApiDict = ApiDict::with_capacity(5);
    info.insert(c"name", Object::string(name));
    // A client that did not say which major version it speaks is version 0.
    let has_major = version.iter().any(|pair| pair.key.bytes() == b"major");
    if !has_major {
        version.insert(c"major", Object::integer(0 as Integer));
    }
    info.insert(c"version", Object::dict(version));
    info.insert(c"type", Object::string(type_0));
    info.insert(c"methods", Object::dict(methods));
    info.insert(c"attributes", Object::dict(attributes));
    rpc_set_client_info(channel_id, info.clone());
}

// `nvim__chan_set_detach` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__chan_set_detach(channel_id: uint64_t, detach: Boolean) -> Result<(), Error> {
    let mut error = Error::none();
    let chan: *mut Channel = find_channel(channel_id);
    if chan.is_null() {
        let msg = e_invchan.as_ptr();
        // SAFETY: the message the caller handed over, live for this call.
        error = Error::from_message(kErrorTypeValidation, unsafe { cstr::at(msg) });
        return ().reported(error);
    }
    unsafe { (*chan).detach = detach };
    ().reported(error)
}

/// # Safety
/// The answer's storage is the api's own: the caller frees whatever this hands
/// back.
pub unsafe fn nvim_get_chan_info(channel_id: uint64_t, mut chan: Integer) -> ApiDict {
    if chan < 0 as Integer {
        return ApiDict::EMPTY;
    }
    if chan == 0 as Integer && !is_internal_call(channel_id) {
        debug_assert!(
            channel_id <= 9223372036854775807 as uint64_t,
            "channel_id <= INT64_MAX"
        );
        chan = channel_id.cast_signed();
    }
    channel_info(chan.cast_unsigned())
}

/// # Safety
/// The answer's storage is the api's own: the caller frees whatever this hands
/// back.
pub unsafe fn nvim_list_chans() -> Array {
    channel_all_info()
}

/// # Safety
/// The answer's storage is the api's own: the caller frees whatever this hands
/// back.
pub unsafe fn nvim_list_uis() -> Array {
    ui_array()
}
