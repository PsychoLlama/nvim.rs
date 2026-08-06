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
                    b"channel_id <= INT64_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/api/vim.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1658 as ::core::ffi::c_uint,
                    b"Array nvim_get_api_info(uint64_t, Arena *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let c2rust_fresh12 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh12 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: channel_id as int64_t,
            },
        };
        let c2rust_fresh13 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh13 as isize) = api_metadata();
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
        let c2rust_fresh14 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh14 as isize) = key_value_pair {
            key: cstr_as_string(b"name\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed { string: name },
            },
        };
        let mut has_major: bool = false_0 != 0;
        let mut i: size_t = 0 as size_t;
        while i < version.size {
            if strequal(
                (*version.items.offset(i as isize)).key.data,
                b"major\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                has_major = true_0 != 0;
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
            let c2rust_fresh15 = v.size;
            v.size = v.size.wrapping_add(1);
            *v.items.offset(c2rust_fresh15 as isize) = key_value_pair {
                key: cstr_as_string(b"major\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: 0 as Integer,
                    },
                },
            };
            version = v;
        }
        let c2rust_fresh16 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh16 as isize) = key_value_pair {
            key: cstr_as_string(b"version\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: version },
            },
        };
        let c2rust_fresh17 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh17 as isize) = key_value_pair {
            key: cstr_as_string(b"type\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed { string: type_0 },
            },
        };
        let c2rust_fresh18 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh18 as isize) = key_value_pair {
            key: cstr_as_string(b"methods\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: methods },
            },
        };
        let c2rust_fresh19 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh19 as isize) = key_value_pair {
            key: cstr_as_string(b"attributes\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: attributes },
            },
        };
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
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
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
                        b"channel_id <= INT64_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/vim.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1800 as ::core::ffi::c_uint,
                        b"Dict nvim_get_chan_info(uint64_t, Integer, Arena *, Error *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
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
