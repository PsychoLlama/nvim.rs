extern "C" {
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
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
pub const MPACK_MAX_TOKEN_LEN: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn mpack_tokbuf_init(mut tokbuf: *mut mpack_tokbuf_t) {
    (*tokbuf).ppos = 0 as size_t;
    (*tokbuf).plen = 0 as size_t;
    (*tokbuf).passthrough = 0 as mpack_uint32_t;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_read(
    mut tokbuf: *mut mpack_tokbuf_t,
    mut buf: *mut *const ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut tok: *mut mpack_token_t,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = 0;
    let mut initial_ppos: size_t = 0;
    let mut ptrlen: size_t = 0;
    let mut advanced: size_t = 0;
    let mut ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ptr_save: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    '_c2rust_label: {
        if !(*buf).is_null() {
        } else {
            __assert_fail(
                b"*buf\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/mpack_core.rs\0".as_ptr() as *const ::core::ffi::c_char,
                50 as ::core::ffi::c_uint,
                b"int mpack_read(mpack_tokbuf_t *, const char **, size_t *, mpack_token_t *)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if *buflen == 0 as size_t {
        return MPACK_EOF as ::core::ffi::c_int;
    }
    if (*tokbuf).passthrough != 0 {
        (*tok).type_0 = MPACK_TOKEN_CHUNK;
        (*tok).data.chunk_ptr = *buf;
        (*tok).length = if (*buflen as mpack_uint32_t) < (*tokbuf).passthrough {
            *buflen as mpack_uint32_t
        } else {
            (*tokbuf).passthrough
        };
        (*tokbuf).passthrough = (*tokbuf).passthrough.wrapping_sub((*tok).length);
        *buf = (*buf).offset((*tok).length as isize);
        *buflen = (*buflen).wrapping_sub((*tok).length as size_t);
    } else {
        initial_ppos = (*tokbuf).ppos;
        if (*tokbuf).plen != 0 {
            if mpack_rpending(buf, buflen, tokbuf) == 0 {
                return MPACK_EOF as ::core::ffi::c_int;
            }
            ptr = &raw mut (*tokbuf).pending as *mut ::core::ffi::c_char;
            ptrlen = (*tokbuf).ppos;
        } else {
            ptr = *buf;
            ptrlen = *buflen;
        }
        ptr_save = ptr;
        status = mpack_rtoken(&raw mut ptr, &raw mut ptrlen, tok);
        if status != 0 {
            if status != MPACK_EOF as ::core::ffi::c_int {
                return MPACK_ERROR as ::core::ffi::c_int;
            }
            '_c2rust_label_0: {
                if (*tokbuf).plen == 0 {
                } else {
                    __assert_fail(
                        b"!tokbuf->plen\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/mpack/mpack_core.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        85 as ::core::ffi::c_uint,
                        b"int mpack_read(mpack_tokbuf_t *, const char **, size_t *, mpack_token_t *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            (*tokbuf).plen = (*tok).length.wrapping_add(1 as mpack_uint32_t) as size_t;
            '_c2rust_label_1: {
                if (*tokbuf).plen <= ::core::mem::size_of::<[::core::ffi::c_char; 9]>() {
                } else {
                    __assert_fail(
                        b"tokbuf->plen <= sizeof(tokbuf->pending)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/mpack/mpack_core.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        90 as ::core::ffi::c_uint,
                        b"int mpack_read(mpack_tokbuf_t *, const char **, size_t *, mpack_token_t *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            (*tokbuf).ppos = 0 as size_t;
            status = mpack_rpending(buf, buflen, tokbuf);
            '_c2rust_label_2: {
                if status == 0 {
                } else {
                    __assert_fail(
                        b"!status\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/mpack/mpack_core.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        93 as ::core::ffi::c_uint,
                        b"int mpack_read(mpack_tokbuf_t *, const char **, size_t *, mpack_token_t *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            return MPACK_EOF as ::core::ffi::c_int;
        }
        advanced = (ptr.offset_from(ptr_save) as size_t).wrapping_sub(initial_ppos);
        (*tokbuf).ppos = 0 as size_t;
        (*tokbuf).plen = (*tokbuf).ppos;
        *buflen = (*buflen).wrapping_sub(advanced);
        *buf = (*buf).offset(advanced as isize);
        if (*tok).type_0 as ::core::ffi::c_uint
            > MPACK_TOKEN_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*tokbuf).passthrough = (*tok).length;
        }
    }
    return MPACK_OK as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_write(
    mut tokbuf: *mut mpack_tokbuf_t,
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut t: *const mpack_token_t,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = 0;
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ptrlen: size_t = 0;
    let mut tok: mpack_token_t = if (*tokbuf).plen != 0 {
        (*tokbuf).pending_tok
    } else {
        *t
    };
    '_c2rust_label: {
        if !(*buf).is_null() && *buflen != 0 {
        } else {
            __assert_fail(
                b"*buf && *buflen\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/mpack_core.rs\0".as_ptr() as *const ::core::ffi::c_char,
                117 as ::core::ffi::c_uint,
                b"int mpack_write(mpack_tokbuf_t *, char **, size_t *, const mpack_token_t *)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if tok.type_0 as ::core::ffi::c_uint
        == MPACK_TOKEN_CHUNK as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut written: size_t = 0;
        let mut pending: size_t = 0;
        let mut count: size_t = 0;
        if (*tokbuf).plen == 0 {
            (*tokbuf).ppos = 0 as size_t;
        }
        written = (*tokbuf).ppos;
        pending = (tok.length as size_t).wrapping_sub(written);
        count = if pending < *buflen { pending } else { *buflen };
        memcpy(
            *buf as *mut ::core::ffi::c_void,
            tok.data.chunk_ptr.offset(written as isize) as *const ::core::ffi::c_void,
            count,
        );
        *buf = (*buf).offset(count as isize);
        *buflen = (*buflen).wrapping_sub(count);
        (*tokbuf).ppos = (*tokbuf).ppos.wrapping_add(count);
        (*tokbuf).plen = (if count == pending {
            0 as mpack_uint32_t
        } else {
            tok.length
        }) as size_t;
        if count == pending {
            return MPACK_OK as ::core::ffi::c_int;
        } else {
            (*tokbuf).pending_tok = tok;
            return MPACK_EOF as ::core::ffi::c_int;
        }
    }
    if (*tokbuf).plen != 0 {
        return mpack_wpending(buf, buflen, tokbuf);
    }
    if *buflen < MPACK_MAX_TOKEN_LEN as size_t {
        ptr = &raw mut (*tokbuf).pending as *mut ::core::ffi::c_char;
        ptrlen = ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t;
    } else {
        ptr = *buf;
        ptrlen = *buflen;
    }
    status = mpack_wtoken(&raw mut tok, &raw mut ptr, &raw mut ptrlen);
    if status != 0 {
        return status;
    }
    if *buflen < MPACK_MAX_TOKEN_LEN as size_t {
        let mut toklen: size_t =
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(ptrlen);
        let mut write_cnt: size_t = if toklen < *buflen { toklen } else { *buflen };
        memcpy(
            *buf as *mut ::core::ffi::c_void,
            &raw mut (*tokbuf).pending as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            write_cnt,
        );
        *buf = (*buf).offset(write_cnt as isize);
        *buflen = (*buflen).wrapping_sub(write_cnt);
        if write_cnt < toklen {
            '_c2rust_label_0: {
                if *buflen == 0 {
                } else {
                    __assert_fail(
                        b"!*buflen\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/mpack/mpack_core.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        157 as ::core::ffi::c_uint,
                        b"int mpack_write(mpack_tokbuf_t *, char **, size_t *, const mpack_token_t *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            (*tokbuf).plen = toklen;
            (*tokbuf).ppos = write_cnt;
            (*tokbuf).pending_tok = tok;
            return MPACK_EOF as ::core::ffi::c_int;
        }
    } else {
        *buflen = (*buflen).wrapping_sub(ptr.offset_from(*buf) as size_t);
        *buf = ptr;
    }
    return MPACK_OK as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_rtoken(
    mut buf: *mut *const ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut tok: *mut mpack_token_t,
) -> ::core::ffi::c_int {
    if *buflen == 0 as size_t {
        return MPACK_EOF as ::core::ffi::c_int;
    }
    *buflen = (*buflen).wrapping_sub(1);
    let c2rust_fresh0 = *buf;
    *buf = (*buf).offset(1);
    let mut t: ::core::ffi::c_uchar = *c2rust_fresh0 as ::core::ffi::c_uchar;
    if (t as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        return mpack_value(MPACK_TOKEN_UINT, 1 as mpack_uint32_t, mpack_byte(t), tok);
    } else if (t as ::core::ffi::c_int) < 0x90 as ::core::ffi::c_int {
        return mpack_blob(
            MPACK_TOKEN_MAP,
            (t as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as mpack_uint32_t,
            0 as ::core::ffi::c_int,
            tok,
        );
    } else if (t as ::core::ffi::c_int) < 0xa0 as ::core::ffi::c_int {
        return mpack_blob(
            MPACK_TOKEN_ARRAY,
            (t as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as mpack_uint32_t,
            0 as ::core::ffi::c_int,
            tok,
        );
    } else if (t as ::core::ffi::c_int) < 0xc0 as ::core::ffi::c_int {
        return mpack_blob(
            MPACK_TOKEN_STR,
            (t as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as mpack_uint32_t,
            0 as ::core::ffi::c_int,
            tok,
        );
    } else if (t as ::core::ffi::c_int) < 0xe0 as ::core::ffi::c_int {
        match t as ::core::ffi::c_int {
            192 => {
                return mpack_value(
                    MPACK_TOKEN_NIL,
                    0 as mpack_uint32_t,
                    mpack_byte(0 as ::core::ffi::c_uchar),
                    tok,
                );
            }
            194 => {
                return mpack_value(
                    MPACK_TOKEN_BOOLEAN,
                    1 as mpack_uint32_t,
                    mpack_byte(0 as ::core::ffi::c_uchar),
                    tok,
                );
            }
            195 => {
                return mpack_value(
                    MPACK_TOKEN_BOOLEAN,
                    1 as mpack_uint32_t,
                    mpack_byte(1 as ::core::ffi::c_uchar),
                    tok,
                );
            }
            196 | 197 | 198 => {
                return mpack_rblob(
                    MPACK_TOKEN_BIN,
                    ((1 as ::core::ffi::c_int)
                        << t as ::core::ffi::c_int - 0xc4 as ::core::ffi::c_int)
                        as mpack_uint32_t,
                    buf,
                    buflen,
                    tok,
                );
            }
            199 | 200 | 201 => {
                return mpack_rblob(
                    MPACK_TOKEN_EXT,
                    ((1 as ::core::ffi::c_int)
                        << t as ::core::ffi::c_int - 0xc7 as ::core::ffi::c_int)
                        as mpack_uint32_t,
                    buf,
                    buflen,
                    tok,
                );
            }
            202 | 203 => {
                return mpack_rvalue(
                    MPACK_TOKEN_FLOAT,
                    ((1 as ::core::ffi::c_int)
                        << t as ::core::ffi::c_int - 0xc8 as ::core::ffi::c_int)
                        as mpack_uint32_t,
                    buf,
                    buflen,
                    tok,
                );
            }
            204 | 205 | 206 | 207 => {
                return mpack_rvalue(
                    MPACK_TOKEN_UINT,
                    ((1 as ::core::ffi::c_int)
                        << t as ::core::ffi::c_int - 0xcc as ::core::ffi::c_int)
                        as mpack_uint32_t,
                    buf,
                    buflen,
                    tok,
                );
            }
            208 | 209 | 210 | 211 => {
                return mpack_rvalue(
                    MPACK_TOKEN_SINT,
                    ((1 as ::core::ffi::c_int)
                        << t as ::core::ffi::c_int - 0xd0 as ::core::ffi::c_int)
                        as mpack_uint32_t,
                    buf,
                    buflen,
                    tok,
                );
            }
            212 | 213 | 214 | 215 | 216 => {
                if *buflen == 0 as size_t {
                    (*tok).length = 1 as mpack_uint32_t;
                    return MPACK_EOF as ::core::ffi::c_int;
                }
                (*tok).length = ((1 as ::core::ffi::c_int)
                    << t as ::core::ffi::c_int - 0xd4 as ::core::ffi::c_int)
                    as mpack_uint32_t;
                (*tok).type_0 = MPACK_TOKEN_EXT;
                *buflen = (*buflen).wrapping_sub(1);
                let c2rust_fresh1 = *buf;
                *buf = (*buf).offset(1);
                (*tok).data.ext_type = *c2rust_fresh1 as ::core::ffi::c_uchar as ::core::ffi::c_int;
                return MPACK_OK as ::core::ffi::c_int;
            }
            217 | 218 | 219 => {
                return mpack_rblob(
                    MPACK_TOKEN_STR,
                    ((1 as ::core::ffi::c_int)
                        << t as ::core::ffi::c_int - 0xd9 as ::core::ffi::c_int)
                        as mpack_uint32_t,
                    buf,
                    buflen,
                    tok,
                );
            }
            220 | 221 => {
                return mpack_rblob(
                    MPACK_TOKEN_ARRAY,
                    ((1 as ::core::ffi::c_int)
                        << t as ::core::ffi::c_int - 0xdb as ::core::ffi::c_int)
                        as mpack_uint32_t,
                    buf,
                    buflen,
                    tok,
                );
            }
            222 | 223 => {
                return mpack_rblob(
                    MPACK_TOKEN_MAP,
                    ((1 as ::core::ffi::c_int)
                        << t as ::core::ffi::c_int - 0xdd as ::core::ffi::c_int)
                        as mpack_uint32_t,
                    buf,
                    buflen,
                    tok,
                );
            }
            _ => return MPACK_ERROR as ::core::ffi::c_int,
        }
    } else {
        return mpack_value(MPACK_TOKEN_SINT, 1 as mpack_uint32_t, mpack_byte(t), tok);
    };
}
unsafe extern "C" fn mpack_rpending(
    mut buf: *mut *const ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut state: *mut mpack_tokbuf_t,
) -> ::core::ffi::c_int {
    let mut count: size_t = 0;
    '_c2rust_label: {
        if (*state).ppos < (*state).plen {
        } else {
            __assert_fail(
                b"state->ppos < state->plen\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/mpack_core.rs\0".as_ptr() as *const ::core::ffi::c_char,
                255 as ::core::ffi::c_uint,
                b"int mpack_rpending(const char **, size_t *, mpack_tokbuf_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    count = if (*state).plen.wrapping_sub((*state).ppos) < *buflen {
        (*state).plen.wrapping_sub((*state).ppos)
    } else {
        *buflen
    };
    memcpy(
        (&raw mut (*state).pending as *mut ::core::ffi::c_char).offset((*state).ppos as isize)
            as *mut ::core::ffi::c_void,
        *buf as *const ::core::ffi::c_void,
        count,
    );
    (*state).ppos = (*state).ppos.wrapping_add(count);
    if (*state).ppos < (*state).plen {
        *buf = (*buf).offset(*buflen as isize);
        *buflen = 0 as size_t;
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_rvalue(
    mut type_0: mpack_token_type_t,
    mut remaining: mpack_uint32_t,
    mut buf: *mut *const ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut tok: *mut mpack_token_t,
) -> ::core::ffi::c_int {
    if *buflen < remaining as size_t {
        (*tok).length = remaining;
        return MPACK_EOF as ::core::ffi::c_int;
    }
    mpack_value(
        type_0,
        remaining,
        mpack_byte(0 as ::core::ffi::c_uchar),
        tok,
    );
    while remaining != 0 {
        *buflen = (*buflen).wrapping_sub(1);
        let c2rust_fresh3 = *buf;
        *buf = (*buf).offset(1);
        let mut byte: mpack_uint32_t = *c2rust_fresh3 as ::core::ffi::c_uchar as mpack_uint32_t;
        let mut byte_idx: mpack_uint32_t = 0;
        let mut byte_shift: mpack_uint32_t = 0;
        remaining = remaining.wrapping_sub(1);
        byte_idx = remaining;
        byte_shift = byte_idx
            .wrapping_rem(4 as mpack_uint32_t)
            .wrapping_mul(8 as mpack_uint32_t);
        (*tok).data.value.lo |= byte << byte_shift;
        if remaining == 4 as mpack_uint32_t {
            (*tok).data.value.hi = (*tok).data.value.lo;
            (*tok).data.value.lo = 0 as mpack_uint32_t;
        }
    }
    if type_0 as ::core::ffi::c_uint
        == MPACK_TOKEN_SINT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut hi: mpack_uint32_t = (*tok).data.value.hi;
        let mut lo: mpack_uint32_t = (*tok).data.value.lo;
        let mut msb: mpack_uint32_t = ((*tok).length == 8 as mpack_uint32_t
            && hi >> 31 as ::core::ffi::c_int != 0
            || (*tok).length == 4 as mpack_uint32_t && lo >> 31 as ::core::ffi::c_int != 0
            || (*tok).length == 2 as mpack_uint32_t && lo >> 15 as ::core::ffi::c_int != 0
            || (*tok).length == 1 as mpack_uint32_t && lo >> 7 as ::core::ffi::c_int != 0)
            as ::core::ffi::c_int as mpack_uint32_t;
        if msb == 0 {
            (*tok).type_0 = MPACK_TOKEN_UINT;
        }
    }
    return MPACK_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_rblob(
    mut type_0: mpack_token_type_t,
    mut tlen: mpack_uint32_t,
    mut buf: *mut *const ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut tok: *mut mpack_token_t,
) -> ::core::ffi::c_int {
    let mut l: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed_0 {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    let mut required: mpack_uint32_t = tlen.wrapping_add(
        (if type_0 as ::core::ffi::c_uint
            == MPACK_TOKEN_EXT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as mpack_uint32_t,
    );
    if *buflen < required as size_t {
        (*tok).length = required;
        return MPACK_EOF as ::core::ffi::c_int;
    }
    l.data.value.lo = 0 as mpack_uint32_t;
    mpack_rvalue(MPACK_TOKEN_UINT, tlen, buf, buflen, &raw mut l);
    (*tok).type_0 = type_0;
    (*tok).length = l.data.value.lo;
    if type_0 as ::core::ffi::c_uint == MPACK_TOKEN_EXT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        *buflen = (*buflen).wrapping_sub(1);
        let c2rust_fresh2 = *buf;
        *buf = (*buf).offset(1);
        (*tok).data.ext_type = *c2rust_fresh2 as ::core::ffi::c_uchar as ::core::ffi::c_int;
    }
    return MPACK_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_wtoken(
    mut tok: *const mpack_token_t,
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
) -> ::core::ffi::c_int {
    match (*tok).type_0 as ::core::ffi::c_uint {
        1 => return mpack_w1(buf, buflen, 0xc0 as mpack_uint32_t),
        2 => {
            return mpack_w1(
                buf,
                buflen,
                (if (*tok).data.value.lo != 0 {
                    0xc3 as ::core::ffi::c_int
                } else {
                    0xc2 as ::core::ffi::c_int
                }) as mpack_uint32_t,
            );
        }
        3 => return mpack_wpint(buf, buflen, (*tok).data.value),
        4 => return mpack_wnint(buf, buflen, (*tok).data.value),
        5 => return mpack_wfloat(buf, buflen, tok),
        9 => return mpack_wbin(buf, buflen, (*tok).length),
        10 => return mpack_wstr(buf, buflen, (*tok).length),
        11 => return mpack_wext(buf, buflen, (*tok).data.ext_type, (*tok).length),
        7 => return mpack_warray(buf, buflen, (*tok).length),
        8 => return mpack_wmap(buf, buflen, (*tok).length),
        _ => return MPACK_ERROR as ::core::ffi::c_int,
    };
}
unsafe extern "C" fn mpack_wpending(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut state: *mut mpack_tokbuf_t,
) -> ::core::ffi::c_int {
    let mut count: size_t = 0;
    '_c2rust_label: {
        if (*state).ppos < (*state).plen {
        } else {
            __assert_fail(
                b"state->ppos < state->plen\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/mpack_core.rs\0".as_ptr() as *const ::core::ffi::c_char,
                361 as ::core::ffi::c_uint,
                b"int mpack_wpending(char **, size_t *, mpack_tokbuf_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    count = if (*state).plen.wrapping_sub((*state).ppos) < *buflen {
        (*state).plen.wrapping_sub((*state).ppos)
    } else {
        *buflen
    };
    memcpy(
        *buf as *mut ::core::ffi::c_void,
        (&raw mut (*state).pending as *mut ::core::ffi::c_char).offset((*state).ppos as isize)
            as *const ::core::ffi::c_void,
        count,
    );
    (*state).ppos = (*state).ppos.wrapping_add(count);
    *buf = (*buf).offset(count as isize);
    *buflen = (*buflen).wrapping_sub(count);
    if (*state).ppos == (*state).plen {
        (*state).plen = 0 as size_t;
        return MPACK_OK as ::core::ffi::c_int;
    }
    return MPACK_EOF as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_wpint(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut val: mpack_value_t,
) -> ::core::ffi::c_int {
    let mut hi: mpack_uint32_t = val.hi;
    let mut lo: mpack_uint32_t = val.lo;
    if hi != 0 {
        return (mpack_w1(buf, buflen, 0xcf as mpack_uint32_t) != 0
            || mpack_w4(buf, buflen, hi) != 0
            || mpack_w4(buf, buflen, lo) != 0) as ::core::ffi::c_int;
    } else if lo > 0xffff as mpack_uint32_t {
        return (mpack_w1(buf, buflen, 0xce as mpack_uint32_t) != 0
            || mpack_w4(buf, buflen, lo) != 0) as ::core::ffi::c_int;
    } else if lo > 0xff as mpack_uint32_t {
        return (mpack_w1(buf, buflen, 0xcd as mpack_uint32_t) != 0
            || mpack_w2(buf, buflen, lo) != 0) as ::core::ffi::c_int;
    } else if lo > 0x7f as mpack_uint32_t {
        return (mpack_w1(buf, buflen, 0xcc as mpack_uint32_t) != 0
            || mpack_w1(buf, buflen, lo) != 0) as ::core::ffi::c_int;
    } else {
        return mpack_w1(buf, buflen, lo);
    };
}
unsafe extern "C" fn mpack_wnint(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut val: mpack_value_t,
) -> ::core::ffi::c_int {
    let mut hi: mpack_uint32_t = val.hi;
    let mut lo: mpack_uint32_t = val.lo;
    if lo < 0x80000000 as ::core::ffi::c_uint {
        return (mpack_w1(buf, buflen, 0xd3 as mpack_uint32_t) != 0
            || mpack_w4(buf, buflen, hi) != 0
            || mpack_w4(buf, buflen, lo) != 0) as ::core::ffi::c_int;
    } else if lo < 0xffff8000 as ::core::ffi::c_uint {
        return (mpack_w1(buf, buflen, 0xd2 as mpack_uint32_t) != 0
            || mpack_w4(buf, buflen, lo) != 0) as ::core::ffi::c_int;
    } else if lo < 0xffffff80 as ::core::ffi::c_uint {
        return (mpack_w1(buf, buflen, 0xd1 as mpack_uint32_t) != 0
            || mpack_w2(buf, buflen, lo) != 0) as ::core::ffi::c_int;
    } else if lo < 0xffffffe0 as ::core::ffi::c_uint {
        return (mpack_w1(buf, buflen, 0xd0 as mpack_uint32_t) != 0
            || mpack_w1(buf, buflen, lo) != 0) as ::core::ffi::c_int;
    } else {
        return mpack_w1(buf, buflen, (0x100 as mpack_uint32_t).wrapping_add(lo));
    };
}
unsafe extern "C" fn mpack_wfloat(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut tok: *const mpack_token_t,
) -> ::core::ffi::c_int {
    if (*tok).length == 4 as mpack_uint32_t {
        return (mpack_w1(buf, buflen, 0xca as mpack_uint32_t) != 0
            || mpack_w4(buf, buflen, (*tok).data.value.lo) != 0)
            as ::core::ffi::c_int;
    } else if (*tok).length == 8 as mpack_uint32_t {
        return (mpack_w1(buf, buflen, 0xcb as mpack_uint32_t) != 0
            || mpack_w4(buf, buflen, (*tok).data.value.hi) != 0
            || mpack_w4(buf, buflen, (*tok).data.value.lo) != 0)
            as ::core::ffi::c_int;
    } else {
        return MPACK_ERROR as ::core::ffi::c_int;
    };
}
unsafe extern "C" fn mpack_wstr(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut len: mpack_uint32_t,
) -> ::core::ffi::c_int {
    if len < 0x20 as mpack_uint32_t {
        return mpack_w1(buf, buflen, 0xa0 as mpack_uint32_t | len);
    } else if len < 0x100 as mpack_uint32_t {
        return (mpack_w1(buf, buflen, 0xd9 as mpack_uint32_t) != 0
            || mpack_w1(buf, buflen, len) != 0) as ::core::ffi::c_int;
    } else if len < 0x10000 as ::core::ffi::c_int as mpack_uint32_t {
        return (mpack_w1(buf, buflen, 0xda as mpack_uint32_t) != 0
            || mpack_w2(buf, buflen, len) != 0) as ::core::ffi::c_int;
    } else {
        return (mpack_w1(buf, buflen, 0xdb as mpack_uint32_t) != 0
            || mpack_w4(buf, buflen, len) != 0) as ::core::ffi::c_int;
    };
}
unsafe extern "C" fn mpack_wbin(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut len: mpack_uint32_t,
) -> ::core::ffi::c_int {
    if len < 0x100 as mpack_uint32_t {
        return (mpack_w1(buf, buflen, 0xc4 as mpack_uint32_t) != 0
            || mpack_w1(buf, buflen, len) != 0) as ::core::ffi::c_int;
    } else if len < 0x10000 as ::core::ffi::c_int as mpack_uint32_t {
        return (mpack_w1(buf, buflen, 0xc5 as mpack_uint32_t) != 0
            || mpack_w2(buf, buflen, len) != 0) as ::core::ffi::c_int;
    } else {
        return (mpack_w1(buf, buflen, 0xc6 as mpack_uint32_t) != 0
            || mpack_w4(buf, buflen, len) != 0) as ::core::ffi::c_int;
    };
}
unsafe extern "C" fn mpack_wext(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut type_0: ::core::ffi::c_int,
    mut len: mpack_uint32_t,
) -> ::core::ffi::c_int {
    let mut t: mpack_uint32_t = 0;
    '_c2rust_label: {
        if type_0 >= 0 as ::core::ffi::c_int && type_0 < 0x80 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"type >= 0 && type < 0x80\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/mpack_core.rs\0".as_ptr() as *const ::core::ffi::c_char,
                478 as ::core::ffi::c_uint,
                b"int mpack_wext(char **, size_t *, int, mpack_uint32_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    t = type_0 as mpack_uint32_t;
    match len {
        1 => {
            mpack_w1(buf, buflen, 0xd4 as mpack_uint32_t);
            return mpack_w1(buf, buflen, t);
        }
        2 => {
            mpack_w1(buf, buflen, 0xd5 as mpack_uint32_t);
            return mpack_w1(buf, buflen, t);
        }
        4 => {
            mpack_w1(buf, buflen, 0xd6 as mpack_uint32_t);
            return mpack_w1(buf, buflen, t);
        }
        8 => {
            mpack_w1(buf, buflen, 0xd7 as mpack_uint32_t);
            return mpack_w1(buf, buflen, t);
        }
        16 => {
            mpack_w1(buf, buflen, 0xd8 as mpack_uint32_t);
            return mpack_w1(buf, buflen, t);
        }
        _ => {
            if len < 0x100 as mpack_uint32_t {
                return (mpack_w1(buf, buflen, 0xc7 as mpack_uint32_t) != 0
                    || mpack_w1(buf, buflen, len) != 0
                    || mpack_w1(buf, buflen, t) != 0) as ::core::ffi::c_int;
            } else if len < 0x10000 as ::core::ffi::c_int as mpack_uint32_t {
                return (mpack_w1(buf, buflen, 0xc8 as mpack_uint32_t) != 0
                    || mpack_w2(buf, buflen, len) != 0
                    || mpack_w1(buf, buflen, t) != 0) as ::core::ffi::c_int;
            } else {
                return (mpack_w1(buf, buflen, 0xc9 as mpack_uint32_t) != 0
                    || mpack_w4(buf, buflen, len) != 0
                    || mpack_w1(buf, buflen, t) != 0) as ::core::ffi::c_int;
            }
        }
    };
}
unsafe extern "C" fn mpack_warray(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut len: mpack_uint32_t,
) -> ::core::ffi::c_int {
    if len < 0x10 as mpack_uint32_t {
        return mpack_w1(buf, buflen, 0x90 as mpack_uint32_t | len);
    } else if len < 0x10000 as ::core::ffi::c_int as mpack_uint32_t {
        return (mpack_w1(buf, buflen, 0xdc as mpack_uint32_t) != 0
            || mpack_w2(buf, buflen, len) != 0) as ::core::ffi::c_int;
    } else {
        return (mpack_w1(buf, buflen, 0xdd as mpack_uint32_t) != 0
            || mpack_w4(buf, buflen, len) != 0) as ::core::ffi::c_int;
    };
}
unsafe extern "C" fn mpack_wmap(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buflen: *mut size_t,
    mut len: mpack_uint32_t,
) -> ::core::ffi::c_int {
    if len < 0x10 as mpack_uint32_t {
        return mpack_w1(buf, buflen, 0x80 as mpack_uint32_t | len);
    } else if len < 0x10000 as ::core::ffi::c_int as mpack_uint32_t {
        return (mpack_w1(buf, buflen, 0xde as mpack_uint32_t) != 0
            || mpack_w2(buf, buflen, len) != 0) as ::core::ffi::c_int;
    } else {
        return (mpack_w1(buf, buflen, 0xdf as mpack_uint32_t) != 0
            || mpack_w4(buf, buflen, len) != 0) as ::core::ffi::c_int;
    };
}
unsafe extern "C" fn mpack_w1(
    mut b: *mut *mut ::core::ffi::c_char,
    mut bl: *mut size_t,
    mut v: mpack_uint32_t,
) -> ::core::ffi::c_int {
    *bl = (*bl).wrapping_sub(1);
    let c2rust_fresh8 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh8 = (v & 0xff as mpack_uint32_t) as ::core::ffi::c_char;
    return MPACK_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_w2(
    mut b: *mut *mut ::core::ffi::c_char,
    mut bl: *mut size_t,
    mut v: mpack_uint32_t,
) -> ::core::ffi::c_int {
    *bl = (*bl).wrapping_sub(2 as size_t);
    let c2rust_fresh9 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh9 = (v >> 8 as ::core::ffi::c_int & 0xff as mpack_uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh10 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh10 = (v & 0xff as mpack_uint32_t) as ::core::ffi::c_char;
    return MPACK_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_w4(
    mut b: *mut *mut ::core::ffi::c_char,
    mut bl: *mut size_t,
    mut v: mpack_uint32_t,
) -> ::core::ffi::c_int {
    *bl = (*bl).wrapping_sub(4 as size_t);
    let c2rust_fresh4 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh4 =
        (v >> 24 as ::core::ffi::c_int & 0xff as mpack_uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh5 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh5 =
        (v >> 16 as ::core::ffi::c_int & 0xff as mpack_uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh6 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh6 = (v >> 8 as ::core::ffi::c_int & 0xff as mpack_uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh7 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh7 = (v & 0xff as mpack_uint32_t) as ::core::ffi::c_char;
    return MPACK_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_value(
    mut type_0: mpack_token_type_t,
    mut length: mpack_uint32_t,
    mut value: mpack_value_t,
    mut tok: *mut mpack_token_t,
) -> ::core::ffi::c_int {
    (*tok).type_0 = type_0;
    (*tok).length = length;
    (*tok).data.value = value;
    return MPACK_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_blob(
    mut type_0: mpack_token_type_t,
    mut length: mpack_uint32_t,
    mut ext_type: ::core::ffi::c_int,
    mut tok: *mut mpack_token_t,
) -> ::core::ffi::c_int {
    (*tok).type_0 = type_0;
    (*tok).length = length;
    (*tok).data.ext_type = ext_type;
    return MPACK_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_byte(mut byte: ::core::ffi::c_uchar) -> mpack_value_t {
    let mut rv: mpack_value_t = mpack_value_t { lo: 0, hi: 0 };
    rv.lo = byte as mpack_uint32_t;
    rv.hi = 0 as mpack_uint32_t;
    return rv;
}
