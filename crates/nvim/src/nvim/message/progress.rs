//! Progress messages: `nvim_echo`'s `progress` kind.
//!
//! A progress message carries an id and a status, replaces its previous self
//! in the history rather than appending ([`crate::src::nvim::message::msg_hist_add_multihl`]),
//! and fires the `Progress` autocommand ([`do_autocmd_progress`]).

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn format_progress_message(
    mut hl_msg: HlMessage,
    mut msg_data: *mut MessageData,
) -> HlMessage {
    unsafe {
        let mut updated_msg: HlMessage = HlMessage {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<HlMessageChunk>(),
        };
        if (*msg_data).title.size != 0 as size_t {
            let mut hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if (*msg_data).status.data.is_null() {
                hl_id = 0 as ::core::ffi::c_int;
            } else if strequal(
                (*msg_data).status.data,
                b"success\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                hl_id = syn_check_group(
                    b"OkMsg\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                );
            } else if strequal(
                (*msg_data).status.data,
                b"failed\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                hl_id = syn_check_group(
                    b"ErrorMsg\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                );
            } else if strequal(
                (*msg_data).status.data,
                b"running\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                hl_id = syn_check_group(
                    b"MoreMsg\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                );
            } else if strequal(
                (*msg_data).status.data,
                b"cancel\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                hl_id = syn_check_group(
                    b"WarningMsg\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
                );
            }
            if updated_msg.size == updated_msg.capacity {
                updated_msg.capacity = if updated_msg.capacity != 0 {
                    updated_msg.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                updated_msg.items = xrealloc(
                    updated_msg.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(updated_msg.capacity),
                ) as *mut HlMessageChunk;
            } else {
            };
            let c2rust_fresh9 = updated_msg.size;
            updated_msg.size = updated_msg.size.wrapping_add(1);
            *updated_msg.items.offset(c2rust_fresh9 as isize) = HlMessageChunk {
                text: copy_string((*msg_data).title, ::core::ptr::null_mut::<Arena>()),
                hl_id: hl_id,
            };
            if updated_msg.size == updated_msg.capacity {
                updated_msg.capacity = if updated_msg.capacity != 0 {
                    updated_msg.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                updated_msg.items = xrealloc(
                    updated_msg.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(updated_msg.capacity),
                ) as *mut HlMessageChunk;
            } else {
            };
            let c2rust_fresh10 = updated_msg.size;
            updated_msg.size = updated_msg.size.wrapping_add(1);
            *updated_msg.items.offset(c2rust_fresh10 as isize) = HlMessageChunk {
                text: cstr_to_string(b": \0".as_ptr() as *const ::core::ffi::c_char),
                hl_id: 0 as ::core::ffi::c_int,
            };
        }
        if (*msg_data).percent > 0 as Integer {
            let mut percent_buf: [::core::ffi::c_char; 10] = [0; 10];
            vim_snprintf(
                &raw mut percent_buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>(),
                b"%3ld%% \0".as_ptr() as *const ::core::ffi::c_char,
                (*msg_data).percent as ::core::ffi::c_long,
            );
            let mut percent: String_0 =
                cstr_to_string(&raw mut percent_buf as *mut ::core::ffi::c_char);
            let mut hl_id_0: ::core::ffi::c_int = syn_check_group(
                b"WarningMsg\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
            );
            if updated_msg.size == updated_msg.capacity {
                updated_msg.capacity = if updated_msg.capacity != 0 {
                    updated_msg.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                updated_msg.items = xrealloc(
                    updated_msg.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(updated_msg.capacity),
                ) as *mut HlMessageChunk;
            } else {
            };
            let c2rust_fresh11 = updated_msg.size;
            updated_msg.size = updated_msg.size.wrapping_add(1);
            *updated_msg.items.offset(c2rust_fresh11 as isize) = HlMessageChunk {
                text: percent,
                hl_id: hl_id_0,
            };
        }
        if updated_msg.size != 0 as size_t {
            let mut i: uint32_t = 0 as uint32_t;
            while (i as size_t) < hl_msg.size {
                if updated_msg.size == updated_msg.capacity {
                    updated_msg.capacity = if updated_msg.capacity != 0 {
                        updated_msg.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    updated_msg.items = xrealloc(
                        updated_msg.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(updated_msg.capacity),
                    ) as *mut HlMessageChunk;
                } else {
                };
                let c2rust_fresh12 = updated_msg.size;
                updated_msg.size = updated_msg.size.wrapping_add(1);
                *updated_msg.items.offset(c2rust_fresh12 as isize) = HlMessageChunk {
                    text: copy_string(
                        (*hl_msg.items.offset(i as isize)).text,
                        ::core::ptr::null_mut::<Arena>(),
                    ),
                    hl_id: (*hl_msg.items.offset(i as isize)).hl_id,
                };
                i = i.wrapping_add(1);
            }
            return updated_msg;
        } else {
            return hl_msg;
        };
    }
}

pub unsafe extern "C" fn msg_progress(
    mut s: *mut ::core::ffi::c_char,
    mut id: *mut ::core::ffi::c_char,
    mut status: *mut ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
    mut trunc: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut opts: KeyDict_echo_opts = KeyDict_echo_opts {
            is_set__echo_opts_: 0,
            err: false,
            verbose: false,
            _truncate: false,
            kind: cstr_as_string(b"progress\0".as_ptr() as *const ::core::ffi::c_char),
            id: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_11 {
                    string: cstr_as_string(id),
                },
            },
            title: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            status: cstr_as_string(status),
            percent: 0,
            source: cstr_as_string(b"nvim\0".as_ptr() as *const ::core::ffi::c_char),
            data: Dict {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            },
        };
        if hist as ::core::ffi::c_int != 0
            && (!trunc || ui_has(kUIMessages) as ::core::ffi::c_int != 0)
        {
            msg_hist_add(s, -1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        }
        if trunc {
            s = msg_may_trunc(false_0 != 0, s);
        }
        let mut chunk: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut chunk__items: [Object; 2] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_11 { boolean: false },
        }; 2];
        chunk.capacity = 2 as size_t;
        chunk.items = &raw mut chunk__items as *mut Object;
        let mut chunks: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut chunks__items: [Object; 1] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_11 { boolean: false },
        }; 1];
        chunks.capacity = 1 as size_t;
        chunks.items = &raw mut chunks__items as *mut Object;
        let c2rust_fresh13 = chunk.size;
        chunk.size = chunk.size.wrapping_add(1);
        *chunk.items.offset(c2rust_fresh13 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_11 {
                string: cstr_as_string(s),
            },
        };
        let c2rust_fresh14 = chunk.size;
        chunk.size = chunk.size.wrapping_add(1);
        *chunk.items.offset(c2rust_fresh14 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed_11 {
                integer: hl_id as Integer,
            },
        };
        let c2rust_fresh15 = chunks.size;
        chunks.size = chunks.size.wrapping_add(1);
        *chunks.items.offset(c2rust_fresh15 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_11 { array: chunk },
        };
        nvim_echo(chunks, false_0 != 0, &raw mut opts, &raw mut err);
        ui_flush();
        return s;
    }
}

pub unsafe extern "C" fn do_autocmd_progress(
    mut msg_id: Object,
    mut msg_0: HlMessage,
    mut msg_data: *mut MessageData,
) {
    unsafe {
        if !has_event(EVENT_PROGRESS) {
            return;
        }
        let mut data: Dict = Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
        let mut data__items: [KeyValuePair; 7] = [KeyValuePair {
            key: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            value: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_11 { boolean: false },
            },
        }; 7];
        data.capacity = 7 as size_t;
        data.items = &raw mut data__items as *mut KeyValuePair;
        let mut messages: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut i: size_t = 0 as size_t;
        while i < msg_0.size {
            if messages.size == messages.capacity {
                messages.capacity = if messages.capacity != 0 {
                    messages.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                messages.items = xrealloc(
                    messages.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<Object>().wrapping_mul(messages.capacity),
                ) as *mut Object;
            } else {
            };
            let c2rust_fresh16 = messages.size;
            messages.size = messages.size.wrapping_add(1);
            *messages.items.offset(c2rust_fresh16 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_11 {
                    string: (*msg_0.items.offset(i as isize)).text,
                },
            };
            i = i.wrapping_add(1);
        }
        let c2rust_fresh17 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh17 as isize) = key_value_pair {
            key: cstr_as_string(b"id\0".as_ptr() as *const ::core::ffi::c_char),
            value: msg_id,
        };
        let c2rust_fresh18 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh18 as isize) = key_value_pair {
            key: cstr_as_string(b"text\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_11 { array: messages },
            },
        };
        if !msg_data.is_null() {
            let c2rust_fresh19 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh19 as isize) = key_value_pair {
                key: cstr_as_string(b"percent\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_11 {
                        integer: (*msg_data).percent,
                    },
                },
            };
            let c2rust_fresh20 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh20 as isize) = key_value_pair {
                key: cstr_as_string(b"source\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_11 {
                        string: (*msg_data).source,
                    },
                },
            };
            let c2rust_fresh21 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh21 as isize) = key_value_pair {
                key: cstr_as_string(b"status\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_11 {
                        string: (*msg_data).status,
                    },
                },
            };
            let c2rust_fresh22 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh22 as isize) = key_value_pair {
                key: cstr_as_string(b"title\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_11 {
                        string: (*msg_data).title,
                    },
                },
            };
            let c2rust_fresh23 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh23 as isize) = key_value_pair {
                key: cstr_as_string(b"data\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeDict,
                    data: C2Rust_Unnamed_11 {
                        dict: (*msg_data).data,
                    },
                },
            };
        }
        let mut c2rust_lvalue: Object = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed_11 { dict: data },
        };
        apply_autocmds_group(
            EVENT_PROGRESS,
            (if !msg_data.is_null() && (*msg_data).source.size > 0 as size_t {
                (*msg_data).source.data as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            true_0 != 0,
            AUGROUP_ALL as ::core::ffi::c_int,
            ::core::ptr::null_mut::<buf_T>(),
            ::core::ptr::null_mut::<exarg_T>(),
            &raw mut c2rust_lvalue,
        );
        xfree(messages.items as *mut ::core::ffi::c_void);
        messages.capacity = 0 as size_t;
        messages.size = messages.capacity;
        messages.items = ::core::ptr::null_mut::<Object>();
    }
}
