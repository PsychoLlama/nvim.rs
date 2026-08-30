//! Who is on the other end of a channel.
//!
//! `nvim_set_client_info` records a client's name, version, methods and
//! attributes against its channel, and `nvim_get_chan_info` renders that
//! back for one channel (`nvim_list_chans` for all of them).
//! `nvim_get_api_info` is what a client calls first: its own channel id
//! plus the packed api metadata.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{NIL, Reported, array_add, dict_put};
use crate::cstr;

pub unsafe fn nvim_get_api_info(channel_id: uint64_t, arena: *mut Arena) -> Array {
    let mut rv: Array = arena_array(arena, 2 as size_t);
    debug_assert!(
        channel_id <= 9223372036854775807 as uint64_t,
        "channel_id <= INT64_MAX"
    );
    unsafe { array_add(&mut rv, Object::integer(channel_id as int64_t)) };
    unsafe { array_add(&mut rv, api_metadata()) };
    rv
}

pub unsafe fn nvim_set_client_info(
    channel_id: uint64_t,
    name: String_0,
    mut version: Dict,
    type_0: String_0,
    methods: Dict,
    attributes: Dict,
    arena: *mut Arena,
) {
    let mut info: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut info__items: [KeyValuePair; 5] = [KeyValuePair {
        key: String_0::NULL,
        value: NIL,
    }; 5];
    info.capacity = 5 as size_t;
    info.items = &raw mut info__items as *mut KeyValuePair;
    unsafe { dict_put(&mut info, c"name", Object::string(name)) };
    let mut has_major: bool = false;
    let mut i: size_t = 0 as size_t;
    while i < version.size {
        if unsafe { strequal((*version.items.add(i)).key.data(), c"major".as_ptr()) } {
            has_major = true;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if !has_major {
        let mut v: Dict = arena_dict(arena, version.size.wrapping_add(1 as size_t));
        if version.size != 0 {
            let dst = v.items.cast::<::core::ffi::c_void>();
            let src = version.items.cast::<::core::ffi::c_void>();
            let bytes = version
                .size
                .wrapping_mul(::core::mem::size_of::<KeyValuePair>());
            // SAFETY: `v` is the arena block just sized for one more pair
            // than `version` holds, and `version` is the caller's.
            unsafe { dst.cast::<u8>().copy_from_nonoverlapping(src.cast(), bytes) };
            v.size = version.size;
        }
        unsafe { dict_put(&mut v, c"major", Object::integer(0 as Integer)) };
        version = v;
    }
    unsafe { dict_put(&mut info, c"version", Object::dict(version)) };
    unsafe { dict_put(&mut info, c"type", Object::string(type_0)) };
    unsafe { dict_put(&mut info, c"methods", Object::dict(methods)) };
    unsafe { dict_put(&mut info, c"attributes", Object::dict(attributes)) };
    let no_arena = ::core::ptr::null_mut::<Arena>();
    // SAFETY: `info` is this frame's own, and the copy the channel keeps is
    // owned rather than borrowed from the arena.
    unsafe { rpc_set_client_info(channel_id, copy_dict(info, no_arena)) };
}

pub unsafe fn nvim__chan_set_detach(channel_id: uint64_t, detach: Boolean) -> Result<(), Error> {
    let mut error = Error::none();
    let mut chan: *mut Channel = find_channel(channel_id);
    if chan.is_null() {
        let msg = e_invchan.as_ptr();
        // SAFETY: the message the caller handed over, live for this call.
        error = Error::from_message(kErrorTypeValidation, unsafe { cstr::at(msg) });
        return ().reported(error);
    }
    unsafe { (*chan).detach = detach };
    ().reported(error)
}

pub unsafe fn nvim_get_chan_info(
    channel_id: uint64_t,
    mut chan: Integer,
    arena: *mut Arena,
) -> Dict {
    if chan < 0 as Integer {
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    if chan == 0 as Integer && !is_internal_call(channel_id) {
        debug_assert!(
            channel_id <= 9223372036854775807 as uint64_t,
            "channel_id <= INT64_MAX"
        );
        chan = channel_id as Integer;
    }
    unsafe { channel_info(chan as uint64_t, arena) }
}

pub unsafe fn nvim_list_chans(arena: *mut Arena) -> Array {
    unsafe { channel_all_info(arena) }
}

pub unsafe fn nvim_list_uis(arena: *mut Arena) -> Array {
    unsafe { ui_array(arena) }
}
