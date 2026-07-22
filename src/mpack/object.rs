use crate::src::mpack::mpack_core::{mpack_read, mpack_tokbuf_init, mpack_write};
use crate::src::nvim::os::libc::{__assert_fail, memcpy, memset};
pub use crate::src::nvim::types::{
    mpack_data_t, mpack_node_s, mpack_node_t, mpack_one_parser_t, mpack_parser_t, mpack_sintmax_t,
    mpack_tokbuf_s, mpack_tokbuf_t, mpack_token_s, mpack_token_s_data as C2Rust_Unnamed_0,
    mpack_token_t, mpack_token_type_t, mpack_uint32_t, mpack_uintmax_t, mpack_value_s,
    mpack_value_t, mpack_walk_cb, size_t,
};
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
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const MPACK_MAX_OBJECT_DEPTH: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub unsafe extern "C" fn mpack_parser_init(
    mut parser: *mut mpack_parser_t,
    mut capacity: mpack_uint32_t,
) {
    mpack_tokbuf_init(&raw mut (*parser).tokbuf);
    (*parser).data.p = NULL;
    (*parser).capacity = if capacity != 0 {
        capacity
    } else {
        MPACK_MAX_OBJECT_DEPTH as mpack_uint32_t
    };
    (*parser).size = 0 as mpack_uint32_t;
    (*parser).exiting = 0 as ::core::ffi::c_int;
    memset(
        &raw mut (*parser).items as *mut mpack_node_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<mpack_node_t>()
            .wrapping_mul((*parser).capacity.wrapping_add(1 as mpack_uint32_t) as size_t),
    );
    (*parser).items[0 as ::core::ffi::c_int as usize].pos = -1 as ::core::ffi::c_int as size_t;
    (*parser).status = 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn mpack_parse_tok(
    mut parser: *mut mpack_parser_t,
    mut tok: mpack_token_t,
    mut enter_cb: mpack_walk_cb,
    mut exit_cb: mpack_walk_cb,
) -> ::core::ffi::c_int {
    if (*parser).status == MPACK_EXCEPTION as ::core::ffi::c_int {
        return MPACK_EXCEPTION as ::core::ffi::c_int;
    }
    let mut n: *mut mpack_node_t = ::core::ptr::null_mut::<mpack_node_t>();
    if (*parser).exiting != 0 {
        (*parser).exiting = 0 as ::core::ffi::c_int;
        loop {
            n = mpack_parser_pop(parser);
            if n.is_null() {
                break;
            }
            exit_cb.expect("non-null function pointer")(parser, n);
            if (*parser).status == MPACK_EXCEPTION as ::core::ffi::c_int {
                return MPACK_EXCEPTION as ::core::ffi::c_int;
            }
            if (*parser).size == 0 {
                return MPACK_OK as ::core::ffi::c_int;
            }
        }
        return MPACK_EOF as ::core::ffi::c_int;
    } else {
        if mpack_parser_full(parser) != 0 {
            return MPACK_NOMEM as ::core::ffi::c_int;
        }
        n = mpack_parser_push(parser);
        (*n).tok = tok;
        enter_cb.expect("non-null function pointer")(parser, n);
        if (*parser).status == MPACK_EXCEPTION as ::core::ffi::c_int {
            return MPACK_EXCEPTION as ::core::ffi::c_int;
        }
        (*parser).exiting = 1 as ::core::ffi::c_int;
        return MPACK_EOF as ::core::ffi::c_int;
    };
}
pub unsafe extern "C" fn mpack_unparse_tok(
    mut parser: *mut mpack_parser_t,
    mut tok: *mut mpack_token_t,
    mut enter_cb: mpack_walk_cb,
    mut exit_cb: mpack_walk_cb,
) -> ::core::ffi::c_int {
    if (*parser).status == MPACK_EXCEPTION as ::core::ffi::c_int {
        return MPACK_EXCEPTION as ::core::ffi::c_int;
    }
    let mut n: *mut mpack_node_t = ::core::ptr::null_mut::<mpack_node_t>();
    if (*parser).exiting != 0 {
        (*parser).exiting = 0 as ::core::ffi::c_int;
        loop {
            n = mpack_parser_pop(parser);
            if n.is_null() {
                break;
            }
            exit_cb.expect("non-null function pointer")(parser, n);
            if (*parser).status == MPACK_EXCEPTION as ::core::ffi::c_int {
                return MPACK_EXCEPTION as ::core::ffi::c_int;
            }
            if (*parser).size == 0 {
                return MPACK_OK as ::core::ffi::c_int;
            }
        }
        return MPACK_EOF as ::core::ffi::c_int;
    } else {
        if mpack_parser_full(parser) != 0 {
            return MPACK_NOMEM as ::core::ffi::c_int;
        }
        n = mpack_parser_push(parser);
        enter_cb.expect("non-null function pointer")(parser, n);
        *tok = (*n).tok;
        if (*parser).status == MPACK_EXCEPTION as ::core::ffi::c_int {
            return MPACK_EXCEPTION as ::core::ffi::c_int;
        }
        (*parser).exiting = 1 as ::core::ffi::c_int;
        return MPACK_EOF as ::core::ffi::c_int;
    };
}
pub unsafe extern "C" fn mpack_parse(
    mut parser: *mut mpack_parser_t,
    mut buf: *mut *const ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut enter_cb: mpack_walk_cb,
    mut exit_cb: mpack_walk_cb,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = MPACK_EOF as ::core::ffi::c_int;
    if (*parser).status == MPACK_EXCEPTION as ::core::ffi::c_int {
        return MPACK_EXCEPTION as ::core::ffi::c_int;
    }
    while *buflen != 0 && status != 0 {
        let mut tok: mpack_token_t = mpack_token_t {
            type_0: 0 as mpack_token_type_t,
            length: 0,
            data: C2Rust_Unnamed_0 {
                value: mpack_value_t { lo: 0, hi: 0 },
            },
        };
        let mut tb: *mut mpack_tokbuf_t = &raw mut (*parser).tokbuf;
        let mut buf_save: *const ::core::ffi::c_char = *buf;
        let mut buflen_save: size_t = *buflen;
        status = mpack_read(tb, buf, buflen, &raw mut tok);
        if status == MPACK_EOF as ::core::ffi::c_int {
            continue;
        }
        if status != MPACK_ERROR as ::core::ffi::c_int {
            loop {
                status = mpack_parse_tok(parser, tok, enter_cb, exit_cb);
                if (*parser).status == MPACK_EXCEPTION as ::core::ffi::c_int {
                    return MPACK_EXCEPTION as ::core::ffi::c_int;
                }
                if (*parser).exiting == 0 {
                    break;
                }
            }
            if status != MPACK_NOMEM as ::core::ffi::c_int {
                continue;
            }
        }
        *buf = buf_save;
        *buflen = buflen_save;
        break;
    }
    return status;
}
pub unsafe extern "C" fn mpack_unparse(
    mut parser: *mut mpack_parser_t,
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut enter_cb: mpack_walk_cb,
    mut exit_cb: mpack_walk_cb,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = MPACK_EOF as ::core::ffi::c_int;
    if (*parser).status == MPACK_EXCEPTION as ::core::ffi::c_int {
        return MPACK_EXCEPTION as ::core::ffi::c_int;
    }
    while *buflen != 0 && status != 0 {
        let mut write_status: ::core::ffi::c_int = 0;
        let mut tok: mpack_token_t = mpack_token_t {
            type_0: 0 as mpack_token_type_t,
            length: 0,
            data: C2Rust_Unnamed_0 {
                value: mpack_value_t { lo: 0, hi: 0 },
            },
        };
        let mut tb: *mut mpack_tokbuf_t = &raw mut (*parser).tokbuf;
        if (*tb).plen == 0 {
            (*parser).status = mpack_unparse_tok(parser, &raw mut tok, enter_cb, exit_cb);
        }
        if (*parser).status == MPACK_EXCEPTION as ::core::ffi::c_int {
            return MPACK_EXCEPTION as ::core::ffi::c_int;
        }
        status = (*parser).status;
        if status == MPACK_NOMEM as ::core::ffi::c_int {
            break;
        }
        if (*parser).exiting != 0 {
            write_status = mpack_write(tb, buf, buflen, &raw mut tok);
            status = if write_status != 0 {
                write_status
            } else {
                status
            };
        }
    }
    return status;
}
pub unsafe extern "C" fn mpack_parser_copy(mut d: *mut mpack_parser_t, mut s: *mut mpack_parser_t) {
    let mut dst: *mut mpack_one_parser_t = d as *mut mpack_one_parser_t;
    let mut src: *mut mpack_one_parser_t = s as *mut mpack_one_parser_t;
    let mut i: mpack_uint32_t = 0;
    let mut dst_capacity: mpack_uint32_t = (*dst).capacity;
    '_c2rust_label: {
        if (*src).capacity <= dst_capacity {
        } else {
            __assert_fail(
                b"src->capacity <= dst_capacity\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/object.rs\0".as_ptr() as *const ::core::ffi::c_char,
                135 as ::core::ffi::c_uint,
                b"void mpack_parser_copy(mpack_parser_t *, mpack_parser_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    memcpy(
        dst as *mut ::core::ffi::c_void,
        src as *const ::core::ffi::c_void,
        ::core::mem::size_of::<mpack_one_parser_t>()
            .wrapping_sub(::core::mem::size_of::<mpack_node_t>()),
    );
    (*dst).capacity = dst_capacity;
    i = 0 as mpack_uint32_t;
    while i <= (*src).capacity {
        *(&raw mut (*dst).items as *mut mpack_node_t).offset(i as isize) =
            *(&raw mut (*src).items as *mut mpack_node_t).offset(i as isize);
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn mpack_parser_full(mut parser: *mut mpack_parser_t) -> ::core::ffi::c_int {
    return ((*parser).size == (*parser).capacity) as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_parser_push(mut p: *mut mpack_parser_t) -> *mut mpack_node_t {
    let mut parser: *mut mpack_one_parser_t = p as *mut mpack_one_parser_t;
    let mut top: *mut mpack_node_t = ::core::ptr::null_mut::<mpack_node_t>();
    '_c2rust_label: {
        if (*parser).size < (*parser).capacity {
        } else {
            __assert_fail(
                b"parser->size < parser->capacity\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/object.rs\0".as_ptr() as *const ::core::ffi::c_char,
                155 as ::core::ffi::c_uint,
                b"mpack_node_t *mpack_parser_push(mpack_parser_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    top = (&raw mut (*parser).items as *mut mpack_node_t)
        .offset((*parser).size as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    (*top).data[0 as ::core::ffi::c_int as usize].p = NULL;
    (*top).data[1 as ::core::ffi::c_int as usize].p = NULL;
    (*top).pos = 0 as size_t;
    (*top).key_visited = 0 as ::core::ffi::c_int;
    (*parser).size = (*parser).size.wrapping_add(1);
    return top;
}
unsafe extern "C" fn mpack_parser_pop(mut p: *mut mpack_parser_t) -> *mut mpack_node_t {
    let mut parser: *mut mpack_one_parser_t = p as *mut mpack_one_parser_t;
    let mut top: *mut mpack_node_t = ::core::ptr::null_mut::<mpack_node_t>();
    let mut parent: *mut mpack_node_t = ::core::ptr::null_mut::<mpack_node_t>();
    '_c2rust_label: {
        if (*parser).size != 0 {
        } else {
            __assert_fail(
                b"parser->size\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/object.rs\0".as_ptr() as *const ::core::ffi::c_char,
                170 as ::core::ffi::c_uint,
                b"mpack_node_t *mpack_parser_pop(mpack_parser_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    top = (&raw mut (*parser).items as *mut mpack_node_t).offset((*parser).size as isize);
    if (*top).tok.type_0 as ::core::ffi::c_uint
        > MPACK_TOKEN_CHUNK as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*top).pos < (*top).tok.length as size_t
    {
        return ::core::ptr::null_mut::<mpack_node_t>();
    }
    parent = if (*top.offset(-(1 as ::core::ffi::c_int as isize))).pos
        == -1 as ::core::ffi::c_int as size_t
    {
        ::core::ptr::null_mut::<mpack_node_t>()
    } else {
        top.offset(-(1 as ::core::ffi::c_int as isize))
    };
    if !parent.is_null() {
        if (*top).tok.type_0 as ::core::ffi::c_uint
            == MPACK_TOKEN_CHUNK as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*parent).pos = (*parent).pos.wrapping_add((*top).tok.length as size_t);
        } else if (*parent).tok.type_0 as ::core::ffi::c_uint
            == MPACK_TOKEN_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*parent).key_visited != 0 {
                (*parent).pos = (*parent).pos.wrapping_add(1);
            }
            (*parent).key_visited = ((*parent).key_visited == 0) as ::core::ffi::c_int;
        } else {
            (*parent).pos = (*parent).pos.wrapping_add(1);
        }
    }
    (*parser).size = (*parser).size.wrapping_sub(1);
    return top;
}
