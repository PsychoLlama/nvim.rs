pub use crate::src::nvim::types::{
    mpack_data_t, mpack_rpc_header_s, mpack_rpc_header_t, mpack_rpc_message_s, mpack_rpc_message_t,
    mpack_rpc_one_session_t, mpack_rpc_session_t, mpack_rpc_slot_s, mpack_sintmax_t,
    mpack_tokbuf_s, mpack_tokbuf_t, mpack_token_s, mpack_token_s_data as C2Rust_Unnamed_0,
    mpack_token_t, mpack_token_type_t, mpack_uint32_t, mpack_uintmax_t, mpack_value_s,
    mpack_value_t, size_t,
};
extern "C" {
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn mpack_tokbuf_init(tb: *mut mpack_tokbuf_t);
    fn mpack_read(
        tb: *mut mpack_tokbuf_t,
        b: *mut *const ::core::ffi::c_char,
        bl: *mut size_t,
        tok: *mut mpack_token_t,
    ) -> ::core::ffi::c_int;
    fn mpack_write(
        tb: *mut mpack_tokbuf_t,
        b: *mut *mut ::core::ffi::c_char,
        bl: *mut size_t,
        tok: *const mpack_token_t,
    ) -> ::core::ffi::c_int;
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MPACK_ERROR: C2Rust_Unnamed = 2;
pub const MPACK_EOF: C2Rust_Unnamed = 1;
pub const MPACK_OK: C2Rust_Unnamed = 0;
pub const MPACK_TOKEN_EXT: mpack_token_type_t = 11;
pub const MPACK_TOKEN_STR: mpack_token_type_t = 10;
pub const MPACK_TOKEN_BIN: mpack_token_type_t = 9;
pub const MPACK_TOKEN_MAP: mpack_token_type_t = 8;
pub const MPACK_TOKEN_ARRAY: mpack_token_type_t = 7;
pub const MPACK_TOKEN_CHUNK: mpack_token_type_t = 6;
pub const MPACK_TOKEN_FLOAT: mpack_token_type_t = 5;
pub const MPACK_TOKEN_SINT: mpack_token_type_t = 4;
pub const MPACK_TOKEN_UINT: mpack_token_type_t = 3;
pub const MPACK_TOKEN_BOOLEAN: mpack_token_type_t = 2;
pub const MPACK_TOKEN_NIL: mpack_token_type_t = 1;
pub type C2Rust_Unnamed_1 = ::core::ffi::c_int;
pub const MPACK_NOMEM: C2Rust_Unnamed_1 = 3;
pub const MPACK_EXCEPTION: C2Rust_Unnamed_1 = -1;
pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub const MPACK_RPC_ERROR: C2Rust_Unnamed_2 = 7;
pub const MPACK_RPC_NOTIFICATION: C2Rust_Unnamed_2 = 6;
pub const MPACK_RPC_RESPONSE: C2Rust_Unnamed_2 = 5;
pub const MPACK_RPC_REQUEST: C2Rust_Unnamed_2 = 4;
pub type C2Rust_Unnamed_3 = ::core::ffi::c_uint;
pub const MPACK_RPC_ERESPID: C2Rust_Unnamed_3 = 11;
pub const MPACK_RPC_EMSGID: C2Rust_Unnamed_3 = 10;
pub const MPACK_RPC_ETYPE: C2Rust_Unnamed_3 = 9;
pub const MPACK_RPC_EARRAYL: C2Rust_Unnamed_3 = 8;
pub const MPACK_RPC_EARRAY: C2Rust_Unnamed_3 = 7;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const MPACK_RPC_MAX_REQUESTS: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn mpack_rpc_session_init(
    mut session: *mut mpack_rpc_session_t,
    mut capacity: mpack_uint32_t,
) {
    (*session).capacity = if capacity != 0 {
        capacity
    } else {
        MPACK_RPC_MAX_REQUESTS as mpack_uint32_t
    };
    (*session).request_id = 0 as mpack_uint32_t;
    mpack_tokbuf_init(&raw mut (*session).reader);
    mpack_tokbuf_init(&raw mut (*session).writer);
    mpack_rpc_reset_hdr(&raw mut (*session).receive);
    mpack_rpc_reset_hdr(&raw mut (*session).send);
    memset(
        &raw mut (*session).slots as *mut mpack_rpc_slot_s as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<mpack_rpc_slot_s>().wrapping_mul((*session).capacity as size_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn mpack_rpc_receive_tok(
    mut session: *mut mpack_rpc_session_t,
    mut tok: mpack_token_t,
    mut msg: *mut mpack_rpc_message_t,
) -> ::core::ffi::c_int {
    let mut type_0: ::core::ffi::c_int = 0;
    if (*session).receive.index == 0 as ::core::ffi::c_int {
        if tok.type_0 as ::core::ffi::c_uint
            != MPACK_TOKEN_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return MPACK_RPC_EARRAY as ::core::ffi::c_int;
        }
        if tok.length < 3 as mpack_uint32_t || tok.length > 4 as mpack_uint32_t {
            return MPACK_RPC_EARRAYL as ::core::ffi::c_int;
        }
        (*session).receive.toks[0 as ::core::ffi::c_int as usize] = tok;
        (*session).receive.index += 1;
        return MPACK_EOF as ::core::ffi::c_int;
    }
    if (*session).receive.index == 1 as ::core::ffi::c_int {
        if tok.type_0 as ::core::ffi::c_uint
            != MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint
            || tok.length > 1 as mpack_uint32_t
            || tok.data.value.lo > 2 as mpack_uint32_t
        {
            return MPACK_RPC_ETYPE as ::core::ffi::c_int;
        }
        if tok.data.value.lo < 2 as mpack_uint32_t
            && (*session).receive.toks[0 as ::core::ffi::c_int as usize].length
                != 4 as mpack_uint32_t
        {
            return MPACK_RPC_EARRAYL as ::core::ffi::c_int;
        }
        if tok.data.value.lo == 2 as mpack_uint32_t
            && (*session).receive.toks[0 as ::core::ffi::c_int as usize].length
                != 3 as mpack_uint32_t
        {
            return MPACK_RPC_EARRAYL as ::core::ffi::c_int;
        }
        (*session).receive.toks[1 as ::core::ffi::c_int as usize] = tok;
        (*session).receive.index += 1;
        if tok.data.value.lo < 2 as mpack_uint32_t {
            return MPACK_EOF as ::core::ffi::c_int;
        }
        type_0 = MPACK_RPC_NOTIFICATION as ::core::ffi::c_int;
    } else {
        '_c2rust_label: {
            if (*session).receive.index == 2 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"session->receive.index == 2\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/mpack/rpc.rs\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    73 as ::core::ffi::c_uint,
                    b"int mpack_rpc_receive_tok(mpack_rpc_session_t *, mpack_token_t, mpack_rpc_message_t *)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if tok.type_0 as ::core::ffi::c_uint
            != MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint
            || tok.length > 4 as mpack_uint32_t
        {
            return MPACK_RPC_EMSGID as ::core::ffi::c_int;
        }
        (*msg).id = tok.data.value.lo;
        (*msg).data.p = NULL;
        type_0 = (*session).receive.toks[1 as ::core::ffi::c_int as usize]
            .data
            .value
            .lo as ::core::ffi::c_int
            + MPACK_RPC_REQUEST as ::core::ffi::c_int;
        if type_0 == MPACK_RPC_RESPONSE as ::core::ffi::c_int && mpack_rpc_pop(session, msg) == 0 {
            return MPACK_RPC_ERESPID as ::core::ffi::c_int;
        }
    }
    mpack_rpc_reset_hdr(&raw mut (*session).receive);
    return type_0;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_rpc_request_tok(
    mut session: *mut mpack_rpc_session_t,
    mut tok: *mut mpack_token_t,
    mut data: mpack_data_t,
) -> ::core::ffi::c_int {
    if (*session).send.index == 0 as ::core::ffi::c_int {
        let mut status: ::core::ffi::c_int = 0;
        let mut msg: mpack_rpc_message_t = mpack_rpc_message_t {
            id: 0,
            data: mpack_data_t {
                p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
        };
        loop {
            msg.id = (*session).request_id;
            msg.data = data;
            (*session).send = mpack_rpc_request_hdr();
            (*session).send.toks[2 as ::core::ffi::c_int as usize].type_0 = MPACK_TOKEN_UINT;
            (*session).send.toks[2 as ::core::ffi::c_int as usize]
                .data
                .value
                .lo = msg.id;
            (*session).send.toks[2 as ::core::ffi::c_int as usize]
                .data
                .value
                .hi = 0 as mpack_uint32_t;
            *tok = (*session).send.toks[0 as ::core::ffi::c_int as usize];
            status = mpack_rpc_put(session, msg);
            if status == -1 as ::core::ffi::c_int {
                return MPACK_NOMEM as ::core::ffi::c_int;
            }
            (*session).request_id = ((*session).request_id as ::core::ffi::c_uint)
                .wrapping_add(1 as ::core::ffi::c_uint)
                .wrapping_rem(0xffffffff as ::core::ffi::c_uint)
                as mpack_uint32_t;
            if status != 0 {
                break;
            }
        }
        (*session).send.index += 1;
        return MPACK_EOF as ::core::ffi::c_int;
    }
    if (*session).send.index == 1 as ::core::ffi::c_int {
        *tok = (*session).send.toks[1 as ::core::ffi::c_int as usize];
        (*session).send.index += 1;
        return MPACK_EOF as ::core::ffi::c_int;
    }
    '_c2rust_label: {
        if (*session).send.index == 2 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"session->send.index == 2\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/rpc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                120 as ::core::ffi::c_uint,
                b"int mpack_rpc_request_tok(mpack_rpc_session_t *, mpack_token_t *, mpack_data_t)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    *tok = (*session).send.toks[2 as ::core::ffi::c_int as usize];
    mpack_rpc_reset_hdr(&raw mut (*session).send);
    return MPACK_OK as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_rpc_reply_tok(
    mut session: *mut mpack_rpc_session_t,
    mut tok: *mut mpack_token_t,
    mut id: mpack_uint32_t,
) -> ::core::ffi::c_int {
    if (*session).send.index == 0 as ::core::ffi::c_int {
        (*session).send = mpack_rpc_reply_hdr();
        (*session).send.toks[2 as ::core::ffi::c_int as usize].type_0 = MPACK_TOKEN_UINT;
        (*session).send.toks[2 as ::core::ffi::c_int as usize]
            .data
            .value
            .lo = id;
        (*session).send.toks[2 as ::core::ffi::c_int as usize]
            .data
            .value
            .hi = 0 as mpack_uint32_t;
        *tok = (*session).send.toks[0 as ::core::ffi::c_int as usize];
        (*session).send.index += 1;
        return MPACK_EOF as ::core::ffi::c_int;
    }
    if (*session).send.index == 1 as ::core::ffi::c_int {
        *tok = (*session).send.toks[1 as ::core::ffi::c_int as usize];
        (*session).send.index += 1;
        return MPACK_EOF as ::core::ffi::c_int;
    }
    '_c2rust_label: {
        if (*session).send.index == 2 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"session->send.index == 2\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/rpc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                145 as ::core::ffi::c_uint,
                b"int mpack_rpc_reply_tok(mpack_rpc_session_t *, mpack_token_t *, mpack_uint32_t)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    *tok = (*session).send.toks[2 as ::core::ffi::c_int as usize];
    mpack_rpc_reset_hdr(&raw mut (*session).send);
    return MPACK_OK as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_rpc_notify_tok(
    mut session: *mut mpack_rpc_session_t,
    mut tok: *mut mpack_token_t,
) -> ::core::ffi::c_int {
    if (*session).send.index == 0 as ::core::ffi::c_int {
        (*session).send = mpack_rpc_notify_hdr();
        *tok = (*session).send.toks[0 as ::core::ffi::c_int as usize];
        (*session).send.index += 1;
        return MPACK_EOF as ::core::ffi::c_int;
    }
    '_c2rust_label: {
        if (*session).send.index == 1 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"session->send.index == 1\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/rpc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                161 as ::core::ffi::c_uint,
                b"int mpack_rpc_notify_tok(mpack_rpc_session_t *, mpack_token_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    *tok = (*session).send.toks[1 as ::core::ffi::c_int as usize];
    mpack_rpc_reset_hdr(&raw mut (*session).send);
    return MPACK_OK as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_rpc_receive(
    mut session: *mut mpack_rpc_session_t,
    mut buf: *mut *const ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut msg: *mut mpack_rpc_message_t,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = 0;
    loop {
        let mut tok: mpack_token_t = mpack_token_t {
            type_0: 0 as mpack_token_type_t,
            length: 0,
            data: C2Rust_Unnamed_0 {
                value: mpack_value_t { lo: 0, hi: 0 },
            },
        };
        status = mpack_read(&raw mut (*session).reader, buf, buflen, &raw mut tok);
        if status != 0 {
            break;
        }
        status = mpack_rpc_receive_tok(session, tok, msg);
        if status >= MPACK_RPC_REQUEST as ::core::ffi::c_int {
            break;
        }
        if *buflen == 0 {
            break;
        }
    }
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_rpc_request(
    mut session: *mut mpack_rpc_session_t,
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut data: mpack_data_t,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = MPACK_EOF as ::core::ffi::c_int;
    while status != 0 && *buflen != 0 {
        let mut write_status: ::core::ffi::c_int = 0;
        let mut tok: mpack_token_t = mpack_token_t {
            type_0: 0 as mpack_token_type_t,
            length: 0,
            data: C2Rust_Unnamed_0 {
                value: mpack_value_t { lo: 0, hi: 0 },
            },
        };
        if (*session).writer.plen == 0 {
            status = mpack_rpc_request_tok(session, &raw mut tok, data);
        }
        if status == MPACK_NOMEM as ::core::ffi::c_int {
            break;
        }
        write_status = mpack_write(&raw mut (*session).writer, buf, buflen, &raw mut tok);
        status = if write_status != 0 {
            write_status
        } else {
            status
        };
    }
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_rpc_reply(
    mut session: *mut mpack_rpc_session_t,
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut id: mpack_uint32_t,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = MPACK_EOF as ::core::ffi::c_int;
    while status != 0 && *buflen != 0 {
        let mut write_status: ::core::ffi::c_int = 0;
        let mut tok: mpack_token_t = mpack_token_t {
            type_0: 0 as mpack_token_type_t,
            length: 0,
            data: C2Rust_Unnamed_0 {
                value: mpack_value_t { lo: 0, hi: 0 },
            },
        };
        if (*session).writer.plen == 0 {
            status = mpack_rpc_reply_tok(session, &raw mut tok, id);
        }
        write_status = mpack_write(&raw mut (*session).writer, buf, buflen, &raw mut tok);
        status = if write_status != 0 {
            write_status
        } else {
            status
        };
    }
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_rpc_notify(
    mut session: *mut mpack_rpc_session_t,
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = MPACK_EOF as ::core::ffi::c_int;
    while status != 0 && *buflen != 0 {
        let mut write_status: ::core::ffi::c_int = 0;
        let mut tok: mpack_token_t = mpack_token_t {
            type_0: 0 as mpack_token_type_t,
            length: 0,
            data: C2Rust_Unnamed_0 {
                value: mpack_value_t { lo: 0, hi: 0 },
            },
        };
        if (*session).writer.plen == 0 {
            status = mpack_rpc_notify_tok(session, &raw mut tok);
        }
        write_status = mpack_write(&raw mut (*session).writer, buf, buflen, &raw mut tok);
        status = if write_status != 0 {
            write_status
        } else {
            status
        };
    }
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_rpc_session_copy(
    mut dst: *mut mpack_rpc_session_t,
    mut src: *mut mpack_rpc_session_t,
) {
    let mut i: mpack_uint32_t = 0;
    let mut dst_capacity: mpack_uint32_t = (*dst).capacity;
    '_c2rust_label: {
        if (*src).capacity <= dst_capacity {
        } else {
            __assert_fail(
                b"src->capacity <= dst_capacity\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/rpc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                243 as ::core::ffi::c_uint,
                b"void mpack_rpc_session_copy(mpack_rpc_session_t *, mpack_rpc_session_t *)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    memcpy(
        dst as *mut ::core::ffi::c_void,
        src as *const ::core::ffi::c_void,
        ::core::mem::size_of::<mpack_rpc_one_session_t>()
            .wrapping_sub(::core::mem::size_of::<mpack_rpc_slot_s>()),
    );
    (*dst).capacity = dst_capacity;
    memset(
        &raw mut (*dst).slots as *mut mpack_rpc_slot_s as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<mpack_rpc_slot_s>().wrapping_mul((*dst).capacity as size_t),
    );
    i = 0 as mpack_uint32_t;
    while i < (*src).capacity {
        if (*src).slots[i as usize].used != 0 {
            mpack_rpc_put(dst, (*src).slots[i as usize].msg);
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn mpack_rpc_request_hdr() -> mpack_rpc_header_t {
    let mut hdr: mpack_rpc_header_t = mpack_rpc_header_t {
        toks: [mpack_token_t {
            type_0: 0 as mpack_token_type_t,
            length: 0,
            data: C2Rust_Unnamed_0 {
                value: mpack_value_t { lo: 0, hi: 0 },
            },
        }; 3],
        index: 0,
    };
    hdr.index = 0 as ::core::ffi::c_int;
    hdr.toks[0 as ::core::ffi::c_int as usize].type_0 = MPACK_TOKEN_ARRAY;
    hdr.toks[0 as ::core::ffi::c_int as usize].length = 4 as mpack_uint32_t;
    hdr.toks[1 as ::core::ffi::c_int as usize].type_0 = MPACK_TOKEN_UINT;
    hdr.toks[1 as ::core::ffi::c_int as usize].data.value.lo = 0 as mpack_uint32_t;
    hdr.toks[1 as ::core::ffi::c_int as usize].data.value.hi = 0 as mpack_uint32_t;
    return hdr;
}
unsafe extern "C" fn mpack_rpc_reply_hdr() -> mpack_rpc_header_t {
    let mut hdr: mpack_rpc_header_t = mpack_rpc_request_hdr();
    hdr.toks[1 as ::core::ffi::c_int as usize].data.value.lo = 1 as mpack_uint32_t;
    hdr.toks[1 as ::core::ffi::c_int as usize].data.value.hi = 0 as mpack_uint32_t;
    return hdr;
}
unsafe extern "C" fn mpack_rpc_notify_hdr() -> mpack_rpc_header_t {
    let mut hdr: mpack_rpc_header_t = mpack_rpc_request_hdr();
    hdr.toks[0 as ::core::ffi::c_int as usize].length = 3 as mpack_uint32_t;
    hdr.toks[1 as ::core::ffi::c_int as usize].data.value.lo = 2 as mpack_uint32_t;
    hdr.toks[1 as ::core::ffi::c_int as usize].data.value.hi = 0 as mpack_uint32_t;
    return hdr;
}
unsafe extern "C" fn mpack_rpc_put(
    mut session: *mut mpack_rpc_session_t,
    mut msg: mpack_rpc_message_t,
) -> ::core::ffi::c_int {
    let mut slot: *mut mpack_rpc_slot_s = ::core::ptr::null_mut::<mpack_rpc_slot_s>();
    let mut i: mpack_uint32_t = 0;
    let mut hash: mpack_uint32_t = msg.id.wrapping_rem((*session).capacity);
    i = 0 as mpack_uint32_t;
    while i < (*session).capacity {
        if (*session).slots[hash as usize].used == 0
            || (*session).slots[hash as usize].msg.id == msg.id
        {
            slot = (&raw mut (*session).slots as *mut mpack_rpc_slot_s).offset(hash as isize);
            break;
        } else {
            hash = if hash > 0 as mpack_uint32_t {
                hash.wrapping_sub(1 as mpack_uint32_t)
            } else {
                (*session).capacity.wrapping_sub(1 as mpack_uint32_t)
            };
            i = i.wrapping_add(1);
        }
    }
    if slot.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    if (*slot).msg.id == msg.id && (*slot).used != 0 {
        return 0 as ::core::ffi::c_int;
    }
    (*slot).msg = msg;
    (*slot).used = 1 as ::core::ffi::c_int;
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_rpc_pop(
    mut session: *mut mpack_rpc_session_t,
    mut msg: *mut mpack_rpc_message_t,
) -> ::core::ffi::c_int {
    let mut slot: *mut mpack_rpc_slot_s = ::core::ptr::null_mut::<mpack_rpc_slot_s>();
    let mut i: mpack_uint32_t = 0;
    let mut hash: mpack_uint32_t = (*msg).id.wrapping_rem((*session).capacity);
    i = 0 as mpack_uint32_t;
    while i < (*session).capacity {
        if (*session).slots[hash as usize].used != 0
            && (*session).slots[hash as usize].msg.id == (*msg).id
        {
            slot = (&raw mut (*session).slots as *mut mpack_rpc_slot_s).offset(hash as isize);
            break;
        } else {
            hash = if hash > 0 as mpack_uint32_t {
                hash.wrapping_sub(1 as mpack_uint32_t)
            } else {
                (*session).capacity.wrapping_sub(1 as mpack_uint32_t)
            };
            i = i.wrapping_add(1);
        }
    }
    if slot.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    *msg = (*slot).msg;
    (*slot).used = 0 as ::core::ffi::c_int;
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_rpc_reset_hdr(mut hdr: *mut mpack_rpc_header_t) {
    (*hdr).index = 0 as ::core::ffi::c_int;
}
