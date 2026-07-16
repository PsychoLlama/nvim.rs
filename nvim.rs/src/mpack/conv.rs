extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
}
pub type mpack_sint32_t = ::core::ffi::c_int;
pub type mpack_uint32_t = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_value_s {
    pub lo: mpack_uint32_t,
    pub hi: mpack_uint32_t,
}
pub type mpack_value_t = mpack_value_s;
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
    pub data: C2Rust_Unnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed {
    pub value: mpack_value_t,
    pub chunk_ptr: *const ::core::ffi::c_char,
    pub ext_type: ::core::ffi::c_int,
}
pub type mpack_token_t = mpack_token_s;
pub type mpack_sintmax_t = ::core::ffi::c_longlong;
pub type mpack_uintmax_t = ::core::ffi::c_ulonglong;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_0 {
    pub i: mpack_uint32_t,
    pub c: [::core::ffi::c_char; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_1 {
    pub d: ::core::ffi::c_double,
    pub m: mpack_value_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_2 {
    pub f: ::core::ffi::c_float,
    pub m: mpack_uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_3 {
    pub d: ::core::ffi::c_double,
    pub m: mpack_value_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
    pub f: ::core::ffi::c_float,
    pub m: mpack_uint32_t,
}
#[no_mangle]
pub unsafe extern "C" fn mpack_pack_nil() -> mpack_token_t {
    let mut rv: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    rv.type_0 = MPACK_TOKEN_NIL;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_pack_boolean(mut v: ::core::ffi::c_uint) -> mpack_token_t {
    let mut rv: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    rv.type_0 = MPACK_TOKEN_BOOLEAN;
    rv.data.value.lo = (if v != 0 {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as mpack_uint32_t;
    rv.data.value.hi = 0 as mpack_uint32_t;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_pack_uint(mut v: mpack_uintmax_t) -> mpack_token_t {
    let mut rv: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    rv.data.value.lo = (v & 0xffffffff as mpack_uintmax_t) as mpack_uint32_t;
    rv.data.value.hi = (v >> 31 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as mpack_uint32_t;
    rv.type_0 = MPACK_TOKEN_UINT;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_pack_sint(mut v: mpack_sintmax_t) -> mpack_token_t {
    if v < 0 as mpack_sintmax_t {
        let mut rv: mpack_token_t = mpack_token_t {
            type_0: 0 as mpack_token_type_t,
            length: 0,
            data: C2Rust_Unnamed {
                value: mpack_value_t { lo: 0, hi: 0 },
            },
        };
        let mut tc: mpack_uintmax_t = ((v + 1 as mpack_sintmax_t) as mpack_uintmax_t)
            .wrapping_neg()
            .wrapping_add(1 as mpack_uintmax_t);
        tc = (!tc).wrapping_add(1 as mpack_uintmax_t);
        rv = mpack_pack_uint(tc);
        rv.type_0 = MPACK_TOKEN_SINT;
        return rv;
    }
    return mpack_pack_uint(v as mpack_uintmax_t);
}
#[no_mangle]
pub unsafe extern "C" fn mpack_pack_float_compat(mut v: ::core::ffi::c_double) -> mpack_token_t {
    let mut rv: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    if mpack_fits_single(v) != 0 {
        rv.length = 4 as mpack_uint32_t;
        rv.data.value = mpack_pack_ieee754(v, 23 as ::core::ffi::c_uint, 8 as ::core::ffi::c_uint);
    } else {
        rv.length = 8 as mpack_uint32_t;
        rv.data.value = mpack_pack_ieee754(v, 52 as ::core::ffi::c_uint, 11 as ::core::ffi::c_uint);
    }
    rv.type_0 = MPACK_TOKEN_FLOAT;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_pack_float_fast(mut v: ::core::ffi::c_double) -> mpack_token_t {
    let mut rv: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    if mpack_fits_single(v) != 0 {
        let mut conv: C2Rust_Unnamed_2 = C2Rust_Unnamed_2 { f: 0. };
        conv.f = v as ::core::ffi::c_float;
        rv.length = 4 as mpack_uint32_t;
        rv.data.value.lo = conv.m;
        rv.data.value.hi = 0 as mpack_uint32_t;
    } else {
        let mut conv_0: C2Rust_Unnamed_1 = C2Rust_Unnamed_1 { d: 0. };
        conv_0.d = v;
        rv.length = 8 as mpack_uint32_t;
        rv.data.value = conv_0.m;
        if mpack_is_be() != 0 {
            let mut lo: mpack_uint32_t = rv.data.value.lo;
            rv.data.value.lo = rv.data.value.hi;
            rv.data.value.hi = lo;
        }
    }
    rv.type_0 = MPACK_TOKEN_FLOAT;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_pack_number(mut v: ::core::ffi::c_double) -> mpack_token_t {
    let mut tok: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    let mut vabs: ::core::ffi::c_double = 0.;
    vabs = if v < 0 as ::core::ffi::c_int as ::core::ffi::c_double {
        -v
    } else {
        v
    };
    '_c2rust_label: {
        if v <= 9007199254740991.0f64 && v >= -9007199254740991.0f64 {
        } else {
            __assert_fail(
                b"v <= 9007199254740991. && v >= -9007199254740991.\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/mpack/conv.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                113 as ::core::ffi::c_uint,
                b"mpack_token_t mpack_pack_number(double)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    tok.data.value.hi = (vabs
        / (((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
            as ::core::ffi::c_double
            * ((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
                as ::core::ffi::c_double
            * ((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int % 2 as ::core::ffi::c_int)
                as ::core::ffi::c_double)) as mpack_uint32_t;
    tok.data.value.lo = mpack_fmod_pow2_32(vabs) as mpack_uint32_t;
    if v < 0 as ::core::ffi::c_int as ::core::ffi::c_double {
        tok.type_0 = MPACK_TOKEN_SINT;
        tok.data.value.hi = !tok.data.value.hi;
        tok.data.value.lo = (!tok.data.value.lo).wrapping_add(1 as mpack_uint32_t);
        if tok.data.value.lo == 0 {
            tok.data.value.hi = tok.data.value.hi.wrapping_add(1);
        }
        if tok.data.value.lo == 0 as mpack_uint32_t && tok.data.value.hi == 0 as mpack_uint32_t {
            tok.length = 1 as mpack_uint32_t;
        } else if tok.data.value.lo < 0x80000000 as ::core::ffi::c_uint {
            tok.length = 8 as mpack_uint32_t;
        } else if tok.data.value.lo < 0xffff8000 as ::core::ffi::c_uint {
            tok.length = 4 as mpack_uint32_t;
        } else if tok.data.value.lo < 0xffffff80 as ::core::ffi::c_uint {
            tok.length = 2 as mpack_uint32_t;
        } else {
            tok.length = 1 as mpack_uint32_t;
        }
    } else {
        tok.type_0 = MPACK_TOKEN_UINT;
        if tok.data.value.hi != 0 {
            tok.length = 8 as mpack_uint32_t;
        } else if tok.data.value.lo > 0xffff as mpack_uint32_t {
            tok.length = 4 as mpack_uint32_t;
        } else if tok.data.value.lo > 0xff as mpack_uint32_t {
            tok.length = 2 as mpack_uint32_t;
        } else {
            tok.length = 1 as mpack_uint32_t;
        }
    }
    if mpack_unpack_number(tok) != v {
        return mpack_pack_float_fast(v);
    }
    return tok;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_pack_chunk(
    mut p: *const ::core::ffi::c_char,
    mut l: mpack_uint32_t,
) -> mpack_token_t {
    let mut rv: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    rv.type_0 = MPACK_TOKEN_CHUNK;
    rv.data.chunk_ptr = p;
    rv.length = l;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_pack_str(mut l: mpack_uint32_t) -> mpack_token_t {
    let mut rv: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    rv.type_0 = MPACK_TOKEN_STR;
    rv.length = l;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_pack_bin(mut l: mpack_uint32_t) -> mpack_token_t {
    let mut rv: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    rv.type_0 = MPACK_TOKEN_BIN;
    rv.length = l;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_pack_ext(
    mut t: ::core::ffi::c_int,
    mut l: mpack_uint32_t,
) -> mpack_token_t {
    let mut rv: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    rv.type_0 = MPACK_TOKEN_EXT;
    rv.length = l;
    rv.data.ext_type = t;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_pack_array(mut l: mpack_uint32_t) -> mpack_token_t {
    let mut rv: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    rv.type_0 = MPACK_TOKEN_ARRAY;
    rv.length = l;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_pack_map(mut l: mpack_uint32_t) -> mpack_token_t {
    let mut rv: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    rv.type_0 = MPACK_TOKEN_MAP;
    rv.length = l;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_unpack_boolean(mut t: mpack_token_t) -> bool {
    return t.data.value.lo != 0 || t.data.value.hi != 0;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_unpack_uint(mut t: mpack_token_t) -> mpack_uintmax_t {
    return ((t.data.value.hi as mpack_uintmax_t) << 31 as ::core::ffi::c_int)
        << 1 as ::core::ffi::c_int
        | t.data.value.lo as mpack_uintmax_t;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_unpack_sint(mut t: mpack_token_t) -> mpack_sintmax_t {
    let mut hi: mpack_uint32_t = t.data.value.hi;
    let mut lo: mpack_uint32_t = t.data.value.lo;
    let mut rv: mpack_uintmax_t = lo as mpack_uintmax_t;
    '_c2rust_label: {
        if t.length as usize <= ::core::mem::size_of::<mpack_sintmax_t>() {
        } else {
            __assert_fail(
                b"t.length <= sizeof(mpack_sintmax_t)\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/mpack/conv.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                210 as ::core::ffi::c_uint,
                b"mpack_sintmax_t mpack_unpack_sint(mpack_token_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if t.length == 8 as mpack_uint32_t {
        rv |= ((hi as mpack_uintmax_t) << 31 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
    }
    rv = (!rv
        & ((1 as ::core::ffi::c_int as mpack_uintmax_t)
            << t.length
                .wrapping_mul(8 as mpack_uint32_t)
                .wrapping_sub(1 as mpack_uint32_t))
        .wrapping_sub(1 as mpack_uintmax_t))
    .wrapping_add(1 as mpack_uintmax_t);
    return -(rv.wrapping_sub(1 as mpack_uintmax_t) as mpack_sintmax_t) - 1 as mpack_sintmax_t;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_unpack_float_compat(mut t: mpack_token_t) -> ::core::ffi::c_double {
    let mut sign: mpack_uint32_t = 0;
    let mut exponent: mpack_sint32_t = 0;
    let mut bias: mpack_sint32_t = 0;
    let mut mantbits: ::core::ffi::c_uint = 0;
    let mut expbits: ::core::ffi::c_uint = 0;
    let mut mant: ::core::ffi::c_double = 0.;
    if t.data.value.lo == 0 as mpack_uint32_t && t.data.value.hi == 0 as mpack_uint32_t {
        return 0 as ::core::ffi::c_int as ::core::ffi::c_double;
    }
    if t.length == 4 as mpack_uint32_t {
        mantbits = 23 as ::core::ffi::c_uint;
        expbits = 8 as ::core::ffi::c_uint;
    } else {
        mantbits = 52 as ::core::ffi::c_uint;
        expbits = 11 as ::core::ffi::c_uint;
    }
    bias = (((1 as ::core::ffi::c_int) << expbits.wrapping_sub(1 as ::core::ffi::c_uint))
        - 1 as ::core::ffi::c_int) as mpack_sint32_t;
    if mantbits == 52 as ::core::ffi::c_uint {
        sign = t.data.value.hi >> 31 as ::core::ffi::c_int;
        exponent = (t.data.value.hi >> 20 as ::core::ffi::c_int
            & (((1 as ::core::ffi::c_int) << 11 as ::core::ffi::c_int) - 1 as ::core::ffi::c_int)
                as mpack_uint32_t) as mpack_sint32_t;
        mant = (t.data.value.hi
            & (((1 as ::core::ffi::c_int) << 20 as ::core::ffi::c_int) - 1 as ::core::ffi::c_int)
                as mpack_uint32_t) as ::core::ffi::c_double
            * (((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
                as ::core::ffi::c_double
                * ((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
                    as ::core::ffi::c_double
                * ((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int % 2 as ::core::ffi::c_int)
                    as ::core::ffi::c_double);
        mant += t.data.value.lo as ::core::ffi::c_double;
    } else {
        sign = t.data.value.lo >> 31 as ::core::ffi::c_int;
        exponent = (t.data.value.lo >> 23 as ::core::ffi::c_int
            & (((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) - 1 as ::core::ffi::c_int)
                as mpack_uint32_t) as mpack_sint32_t;
        mant = (t.data.value.lo
            & (((1 as ::core::ffi::c_int) << 23 as ::core::ffi::c_int) - 1 as ::core::ffi::c_int)
                as mpack_uint32_t) as ::core::ffi::c_double;
    }
    mant /= ((1 as ::core::ffi::c_int) << mantbits.wrapping_div(2 as ::core::ffi::c_uint))
        as ::core::ffi::c_double
        * ((1 as ::core::ffi::c_int) << mantbits.wrapping_div(2 as ::core::ffi::c_uint))
            as ::core::ffi::c_double
        * ((1 as ::core::ffi::c_int) << mantbits.wrapping_rem(2 as ::core::ffi::c_uint))
            as ::core::ffi::c_double;
    if exponent != 0 {
        mant += 1.0f64;
    } else {
        exponent = 1 as ::core::ffi::c_int as mpack_sint32_t;
    }
    exponent -= bias;
    while exponent > 0 as ::core::ffi::c_int {
        mant *= 2.0f64;
        exponent -= 1;
    }
    while exponent < 0 as ::core::ffi::c_int {
        mant /= 2.0f64;
        exponent += 1;
    }
    return mant
        * (if sign != 0 {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) as ::core::ffi::c_double;
}
#[no_mangle]
pub unsafe extern "C" fn mpack_unpack_float_fast(mut t: mpack_token_t) -> ::core::ffi::c_double {
    if t.length == 4 as mpack_uint32_t {
        let mut conv: C2Rust_Unnamed_4 = C2Rust_Unnamed_4 { f: 0. };
        conv.m = t.data.value.lo;
        return conv.f as ::core::ffi::c_double;
    } else {
        let mut conv_0: C2Rust_Unnamed_3 = C2Rust_Unnamed_3 { d: 0. };
        conv_0.m = t.data.value;
        if mpack_is_be() != 0 {
            let mut lo: mpack_uint32_t = conv_0.m.lo;
            conv_0.m.lo = conv_0.m.hi;
            conv_0.m.hi = lo;
        }
        return conv_0.d;
    };
}
#[no_mangle]
pub unsafe extern "C" fn mpack_unpack_number(mut t: mpack_token_t) -> ::core::ffi::c_double {
    let mut rv: ::core::ffi::c_double = 0.;
    let mut hi: mpack_uint32_t = 0;
    let mut lo: mpack_uint32_t = 0;
    if t.type_0 as ::core::ffi::c_uint
        == MPACK_TOKEN_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return mpack_unpack_float_fast(t);
    }
    '_c2rust_label: {
        if t.type_0 as ::core::ffi::c_uint
            == MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint
            || t.type_0 as ::core::ffi::c_uint
                == MPACK_TOKEN_SINT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"t.type == MPACK_TOKEN_UINT || t.type == MPACK_TOKEN_SINT\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/mpack/conv.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                292 as ::core::ffi::c_uint,
                b"double mpack_unpack_number(mpack_token_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    hi = t.data.value.hi;
    lo = t.data.value.lo;
    if t.type_0 as ::core::ffi::c_uint
        == MPACK_TOKEN_SINT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if hi == 0 {
            '_c2rust_label_0: {
                if t.length <= 4 as mpack_uint32_t {
                } else {
                    __assert_fail(
                        b"t.length <= 4\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/mpack/conv.c\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        300 as ::core::ffi::c_uint,
                        b"double mpack_unpack_number(mpack_token_t)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            lo = !lo
                & ((1 as ::core::ffi::c_int as mpack_uint32_t)
                    << t.length
                        .wrapping_mul(8 as mpack_uint32_t)
                        .wrapping_sub(1 as mpack_uint32_t))
                .wrapping_sub(1 as mpack_uint32_t);
        } else {
            hi = !hi;
            lo = !lo;
        }
        lo = lo.wrapping_add(1);
        if lo == 0 {
            hi = hi.wrapping_add(1);
        }
    }
    rv = lo as ::core::ffi::c_double
        + ((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
            as ::core::ffi::c_double
            * ((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
                as ::core::ffi::c_double
            * ((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int % 2 as ::core::ffi::c_int)
                as ::core::ffi::c_double
            * hi as ::core::ffi::c_double;
    return if t.type_0 as ::core::ffi::c_uint
        == MPACK_TOKEN_SINT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        -rv
    } else {
        rv
    };
}
unsafe extern "C" fn mpack_fits_single(mut v: ::core::ffi::c_double) -> ::core::ffi::c_int {
    return (v as ::core::ffi::c_float as ::core::ffi::c_double == v) as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_pack_ieee754(
    mut v: ::core::ffi::c_double,
    mut mantbits: ::core::ffi::c_uint,
    mut expbits: ::core::ffi::c_uint,
) -> mpack_value_t {
    let mut rv: mpack_value_t = mpack_value_s {
        lo: 0 as mpack_uint32_t,
        hi: 0 as mpack_uint32_t,
    };
    let mut exponent: mpack_sint32_t = 0;
    let mut bias: mpack_sint32_t = ((1 as mpack_sint32_t)
        << expbits.wrapping_sub(1 as ::core::ffi::c_uint))
        - 1 as mpack_sint32_t;
    let mut sign: mpack_uint32_t = 0;
    let mut mant: ::core::ffi::c_double = 0.;
    if v == 0 as ::core::ffi::c_int as ::core::ffi::c_double {
        rv.lo = 0 as mpack_uint32_t;
        rv.hi = 0 as mpack_uint32_t;
    } else {
        if v < 0 as ::core::ffi::c_int as ::core::ffi::c_double {
            sign = 1 as mpack_uint32_t;
            mant = -v;
        } else {
            sign = 0 as mpack_uint32_t;
            mant = v;
        }
        exponent = 0 as ::core::ffi::c_int as mpack_sint32_t;
        while mant >= 2.0f64 {
            mant /= 2.0f64;
            exponent += 1;
        }
        while mant < 1.0f64 && exponent > -(bias as ::core::ffi::c_int - 1 as ::core::ffi::c_int) {
            mant *= 2.0f64;
            exponent -= 1;
        }
        if mant < 1.0f64 {
            exponent = -bias;
        } else {
            mant = mant - 1.0f64;
        }
        exponent += bias;
        mant *= ((1 as ::core::ffi::c_int) << mantbits.wrapping_div(2 as ::core::ffi::c_uint))
            as ::core::ffi::c_double
            * ((1 as ::core::ffi::c_int) << mantbits.wrapping_div(2 as ::core::ffi::c_uint))
                as ::core::ffi::c_double
            * ((1 as ::core::ffi::c_int) << mantbits.wrapping_rem(2 as ::core::ffi::c_uint))
                as ::core::ffi::c_double;
        if mantbits == 52 as ::core::ffi::c_uint {
            rv.hi = (mant
                / (((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
                    as ::core::ffi::c_double
                    * ((1 as ::core::ffi::c_int)
                        << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
                        as ::core::ffi::c_double
                    * ((1 as ::core::ffi::c_int)
                        << 32 as ::core::ffi::c_int % 2 as ::core::ffi::c_int)
                        as ::core::ffi::c_double)) as mpack_uint32_t;
            rv.lo = (mant
                - rv.hi as ::core::ffi::c_double
                    * (((1 as ::core::ffi::c_int)
                        << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
                        as ::core::ffi::c_double
                        * ((1 as ::core::ffi::c_int)
                            << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
                            as ::core::ffi::c_double
                        * ((1 as ::core::ffi::c_int)
                            << 32 as ::core::ffi::c_int % 2 as ::core::ffi::c_int)
                            as ::core::ffi::c_double)) as mpack_uint32_t;
            rv.hi |= (exponent as mpack_uint32_t) << 20 as ::core::ffi::c_int
                | sign << 31 as ::core::ffi::c_int;
        } else if mantbits == 23 as ::core::ffi::c_uint {
            rv.hi = 0 as mpack_uint32_t;
            rv.lo = mant as mpack_uint32_t;
            rv.lo |= (exponent as mpack_uint32_t) << 23 as ::core::ffi::c_int
                | sign << 31 as ::core::ffi::c_int;
        }
    }
    return rv;
}
unsafe extern "C" fn mpack_is_be() -> ::core::ffi::c_int {
    let mut test: C2Rust_Unnamed_0 = C2Rust_Unnamed_0 { i: 0 };
    test.i = 1 as mpack_uint32_t;
    return (test.c[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn mpack_fmod_pow2_32(mut a: ::core::ffi::c_double) -> ::core::ffi::c_double {
    return a
        - (a / (((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
            as ::core::ffi::c_double
            * ((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
                as ::core::ffi::c_double
            * ((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int % 2 as ::core::ffi::c_int)
                as ::core::ffi::c_double)) as mpack_uint32_t as ::core::ffi::c_double
            * (((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
                as ::core::ffi::c_double
                * ((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int / 2 as ::core::ffi::c_int)
                    as ::core::ffi::c_double
                * ((1 as ::core::ffi::c_int) << 32 as ::core::ffi::c_int % 2 as ::core::ffi::c_int)
                    as ::core::ffi::c_double);
}
