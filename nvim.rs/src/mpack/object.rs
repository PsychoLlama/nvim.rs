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
pub type size_t = usize;
pub type mpack_uint32_t = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_value_s {
    pub lo: mpack_uint32_t,
    pub hi: mpack_uint32_t,
}
pub type mpack_value_t = mpack_value_s;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MPACK_ERROR: C2Rust_Unnamed = 2;
pub const MPACK_EOF: C2Rust_Unnamed = 1;
pub const MPACK_OK: C2Rust_Unnamed = 0;
pub type mpack_token_type_t = ::core::ffi::c_uint;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_token_s {
    pub type_0: mpack_token_type_t,
    pub length: mpack_uint32_t,
    pub data: C2Rust_Unnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_0 {
    pub value: mpack_value_t,
    pub chunk_ptr: *const ::core::ffi::c_char,
    pub ext_type: ::core::ffi::c_int,
}
pub type mpack_token_t = mpack_token_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_tokbuf_s {
    pub pending: [::core::ffi::c_char; 9],
    pub pending_tok: mpack_token_t,
    pub ppos: size_t,
    pub plen: size_t,
    pub passthrough: mpack_uint32_t,
}
pub type mpack_tokbuf_t = mpack_tokbuf_s;
pub type mpack_sintmax_t = ::core::ffi::c_longlong;
pub type mpack_uintmax_t = ::core::ffi::c_ulonglong;
pub type C2Rust_Unnamed_1 = ::core::ffi::c_int;
pub const MPACK_NOMEM: C2Rust_Unnamed_1 = 3;
pub const MPACK_EXCEPTION: C2Rust_Unnamed_1 = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub union mpack_data_t {
    pub p: *mut ::core::ffi::c_void,
    pub u: mpack_uintmax_t,
    pub i: mpack_sintmax_t,
    pub d: ::core::ffi::c_double,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_node_s {
    pub tok: mpack_token_t,
    pub pos: size_t,
    pub key_visited: ::core::ffi::c_int,
    pub data: [mpack_data_t; 2],
}
pub type mpack_node_t = mpack_node_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_one_parser_t {
    pub data: mpack_data_t,
    pub size: mpack_uint32_t,
    pub capacity: mpack_uint32_t,
    pub status: ::core::ffi::c_int,
    pub exiting: ::core::ffi::c_int,
    pub tokbuf: mpack_tokbuf_t,
    pub items: [mpack_node_t; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_parser_t {
    pub data: mpack_data_t,
    pub size: mpack_uint32_t,
    pub capacity: mpack_uint32_t,
    pub status: ::core::ffi::c_int,
    pub exiting: ::core::ffi::c_int,
    pub tokbuf: mpack_tokbuf_t,
    pub items: [mpack_node_t; 33],
}
pub type mpack_walk_cb = Option<
    unsafe extern "C" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
>;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const MPACK_MAX_OBJECT_DEPTH: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
#[no_mangle]
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
    (*parser).items[0 as ::core::ffi::c_int as usize].pos = -1 as ::core::ffi::c_int
        as size_t;
    (*parser).status = 0 as ::core::ffi::c_int;
}
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
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
            (*parser).status = mpack_unparse_tok(
                parser,
                &raw mut tok,
                enter_cb,
                exit_cb,
            );
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
            status = if write_status != 0 { write_status } else { status };
        }
    }
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_parser_copy(
    mut d: *mut mpack_parser_t,
    mut s: *mut mpack_parser_t,
) {
    let mut dst: *mut mpack_one_parser_t = d as *mut mpack_one_parser_t;
    let mut src: *mut mpack_one_parser_t = s as *mut mpack_one_parser_t;
    let mut i: mpack_uint32_t = 0;
    let mut dst_capacity: mpack_uint32_t = (*dst).capacity;
    '_c2rust_label: {
        if (*src).capacity <= dst_capacity {} else {
            __assert_fail(
                b"src->capacity <= dst_capacity\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/mpack/object.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
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
        *(&raw mut (*dst).items as *mut mpack_node_t).offset(i as isize) = *(&raw mut (*src)
            .items as *mut mpack_node_t)
            .offset(i as isize);
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn mpack_parser_full(
    mut parser: *mut mpack_parser_t,
) -> ::core::ffi::c_int {
    return ((*parser).size == (*parser).capacity) as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_parser_push(mut p: *mut mpack_parser_t) -> *mut mpack_node_t {
    let mut parser: *mut mpack_one_parser_t = p as *mut mpack_one_parser_t;
    let mut top: *mut mpack_node_t = ::core::ptr::null_mut::<mpack_node_t>();
    '_c2rust_label: {
        if (*parser).size < (*parser).capacity {} else {
            __assert_fail(
                b"parser->size < parser->capacity\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/mpack/object.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
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
        if (*parser).size != 0 {} else {
            __assert_fail(
                b"parser->size\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/mpack/object.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                170 as ::core::ffi::c_uint,
                b"mpack_node_t *mpack_parser_pop(mpack_parser_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    top = (&raw mut (*parser).items as *mut mpack_node_t)
        .offset((*parser).size as isize);
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
