//! Who is on the other end of a channel.
//!
//! `nvim_set_client_info` records a client's name, version, methods and
//! attributes against its channel, and `nvim_get_chan_info` renders that
//! back for one channel (`nvim_list_chans` for all of them).
//! `nvim_get_api_info` is what a client calls first: its own channel id
//! plus the packed api metadata.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::api::private::helpers::{array_add, dict_put};

pub unsafe extern "C" fn nvim_get_api_info(
    mut channel_id: uint64_t,
    mut arena: *mut Arena,
) -> Array {
    unsafe {
        let mut rv: Array = arena_array(arena, 2 as size_t);
        '_c2rust_label: {
            if channel_id <= 9223372036854775807 as uint64_t {
            } else {
                __assert_fail(
                    c"channel_id <= INT64_MAX".as_ptr(),
                    c"src/nvim/api/vim.rs".as_ptr(),
                    1658 as ::core::ffi::c_uint,
                    c"Array nvim_get_api_info(uint64_t, Arena *)".as_ptr(),
                );
            }
        };
        array_add(&mut rv, Object::integer(channel_id as int64_t));
        array_add(&mut rv, api_metadata());
        return rv;
    }
}

pub unsafe extern "C" fn nvim_set_client_info(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut version: Dict,
    mut type_0: String_0,
    mut methods: Dict,
    mut attributes: Dict,
    mut arena: *mut Arena,
    mut _err: *mut Error,
) {
    unsafe {
        let mut info: Dict = Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
        let mut info__items: [KeyValuePair; 5] = [KeyValuePair {
            key: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            value: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
        }; 5];
        info.capacity = 5 as size_t;
        info.items = &raw mut info__items as *mut KeyValuePair;
        dict_put(&mut info, c"name", Object::string(name));
        let mut has_major: bool = false;
        let mut i: size_t = 0 as size_t;
        while i < version.size {
            if strequal((*version.items.add(i)).key.data, c"major".as_ptr()) {
                has_major = true;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        if !has_major {
            let mut v: Dict = arena_dict(arena, version.size.wrapping_add(1 as size_t));
            if version.size != 0 {
                memcpy(
                    v.items as *mut ::core::ffi::c_void,
                    version.items as *const ::core::ffi::c_void,
                    version
                        .size
                        .wrapping_mul(::core::mem::size_of::<KeyValuePair>()),
                );
                v.size = version.size;
            }
            dict_put(&mut v, c"major", Object::integer(0 as Integer));
            version = v;
        }
        dict_put(&mut info, c"version", Object::dict(version));
        dict_put(&mut info, c"type", Object::string(type_0));
        dict_put(&mut info, c"methods", Object::dict(methods));
        dict_put(&mut info, c"attributes", Object::dict(attributes));
        rpc_set_client_info(
            channel_id,
            copy_dict(info, ::core::ptr::null_mut::<Arena>()),
        );
    }
}

pub unsafe extern "C" fn nvim__chan_set_detach(
    mut channel_id: uint64_t,
    mut detach: Boolean,
    mut err: *mut Error,
) {
    unsafe {
        let mut chan: *mut Channel = find_channel(channel_id);
        if chan.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                &raw const e_invchan as *const ::core::ffi::c_char,
            );
            return;
        }
        (*chan).detach = detach;
    }
}

pub unsafe extern "C" fn nvim_get_chan_info(
    mut channel_id: uint64_t,
    mut chan: Integer,
    mut arena: *mut Arena,
    mut _err: *mut Error,
) -> Dict {
    unsafe {
        if chan < 0 as Integer {
            return Dict {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            };
        }
        if chan == 0 as Integer && !is_internal_call(channel_id) {
            '_c2rust_label: {
                if channel_id <= 9223372036854775807 as uint64_t {
                } else {
                    __assert_fail(
                        c"channel_id <= INT64_MAX".as_ptr(),
                        c"src/nvim/api/vim.rs".as_ptr(),
                        1800 as ::core::ffi::c_uint,
                        c"Dict nvim_get_chan_info(uint64_t, Integer, Arena *, Error *)".as_ptr(),
                    );
                }
            };
            chan = channel_id as Integer;
        }
        return channel_info(chan as uint64_t, arena);
    }
}

pub unsafe extern "C" fn nvim_list_chans(mut arena: *mut Arena) -> Array {
    unsafe {
        return channel_all_info(arena);
    }
}

pub unsafe extern "C" fn nvim_list_uis(mut arena: *mut Arena) -> Array {
    unsafe {
        return ui_array(arena);
    }
}
