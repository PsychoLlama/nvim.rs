use crate::src::nvim::global_cell::GlobalCell;
pub type size_t = usize;
pub type uint8_t = u8;
pub type uint32_t = u32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermEncoding {
    pub init: Option<unsafe extern "C" fn(*mut VTermEncoding, *mut ::core::ffi::c_void) -> ()>,
    pub decode: Option<
        unsafe extern "C" fn(
            *mut VTermEncoding,
            *mut ::core::ffi::c_void,
            *mut uint32_t,
            *mut ::core::ffi::c_int,
            ::core::ffi::c_int,
            *const ::core::ffi::c_char,
            *mut size_t,
            size_t,
        ) -> (),
    >,
}
pub type VTermEncodingType = ::core::ffi::c_uint;
pub const ENC_SINGLE_94: VTermEncodingType = 1;
pub const ENC_UTF8: VTermEncodingType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed {
    pub type_0: VTermEncodingType,
    pub designation: ::core::ffi::c_char,
    pub enc: *mut VTermEncoding,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StaticTableEncoding {
    pub enc: VTermEncoding,
    pub chars: [uint32_t; 128],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UTF8DecoderData {
    pub bytes_remaining: ::core::ffi::c_int,
    pub bytes_total: ::core::ffi::c_int,
    pub this_cp: ::core::ffi::c_int,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UNICODE_INVALID: ::core::ffi::c_int = 0xfffd as ::core::ffi::c_int;
unsafe extern "C" fn init_utf8(mut _enc: *mut VTermEncoding, mut data_: *mut ::core::ffi::c_void) {
    let mut data: *mut UTF8DecoderData = data_ as *mut UTF8DecoderData;
    (*data).bytes_remaining = 0 as ::core::ffi::c_int;
    (*data).bytes_total = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn decode_utf8(
    mut _enc: *mut VTermEncoding,
    mut data_: *mut ::core::ffi::c_void,
    mut cp: *mut uint32_t,
    mut cpi: *mut ::core::ffi::c_int,
    mut cplen: ::core::ffi::c_int,
    mut bytes: *const ::core::ffi::c_char,
    mut pos: *mut size_t,
    mut bytelen: size_t,
) {
    let mut data: *mut UTF8DecoderData = data_ as *mut UTF8DecoderData;
    while *pos < bytelen && *cpi < cplen {
        let mut c: uint8_t = *bytes.offset(*pos as isize) as uint8_t;
        if (c as ::core::ffi::c_int) < 0x20 as ::core::ffi::c_int {
            return;
        } else {
            if c as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
                && (c as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                if (*data).bytes_remaining != 0 {
                    let c2rust_fresh3 = *cpi;
                    *cpi = *cpi + 1;
                    *cp.offset(c2rust_fresh3 as isize) = UNICODE_INVALID as uint32_t;
                }
                let c2rust_fresh4 = *cpi;
                *cpi = *cpi + 1;
                *cp.offset(c2rust_fresh4 as isize) = c as uint32_t;
                (*data).bytes_remaining = 0 as ::core::ffi::c_int;
            } else if c as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int {
                return;
            } else if c as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
                && (c as ::core::ffi::c_int) < 0xc0 as ::core::ffi::c_int
            {
                if (*data).bytes_remaining == 0 {
                    let c2rust_fresh5 = *cpi;
                    *cpi = *cpi + 1;
                    *cp.offset(c2rust_fresh5 as isize) = UNICODE_INVALID as uint32_t;
                } else {
                    (*data).this_cp <<= 6 as ::core::ffi::c_int;
                    (*data).this_cp |= c as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int;
                    (*data).bytes_remaining -= 1;
                    if (*data).bytes_remaining == 0 {
                        match (*data).bytes_total {
                            2 => {
                                if (*data).this_cp < 0x80 as ::core::ffi::c_int {
                                    (*data).this_cp = UNICODE_INVALID;
                                }
                            }
                            3 => {
                                if (*data).this_cp < 0x800 as ::core::ffi::c_int {
                                    (*data).this_cp = UNICODE_INVALID;
                                }
                            }
                            4 => {
                                if (*data).this_cp < 0x10000 as ::core::ffi::c_int {
                                    (*data).this_cp = UNICODE_INVALID;
                                }
                            }
                            5 => {
                                if (*data).this_cp < 0x200000 as ::core::ffi::c_int {
                                    (*data).this_cp = UNICODE_INVALID;
                                }
                            }
                            6 => {
                                if (*data).this_cp < 0x4000000 as ::core::ffi::c_int {
                                    (*data).this_cp = UNICODE_INVALID;
                                }
                            }
                            _ => {}
                        }
                        if (*data).this_cp >= 0xd800 as ::core::ffi::c_int
                            && (*data).this_cp <= 0xdfff as ::core::ffi::c_int
                            || (*data).this_cp == 0xfffe as ::core::ffi::c_int
                            || (*data).this_cp == 0xffff as ::core::ffi::c_int
                        {
                            (*data).this_cp = UNICODE_INVALID;
                        }
                        let c2rust_fresh6 = *cpi;
                        *cpi = *cpi + 1;
                        *cp.offset(c2rust_fresh6 as isize) = (*data).this_cp as uint32_t;
                    }
                }
            } else if c as ::core::ffi::c_int >= 0xc0 as ::core::ffi::c_int
                && (c as ::core::ffi::c_int) < 0xe0 as ::core::ffi::c_int
            {
                if (*data).bytes_remaining != 0 {
                    let c2rust_fresh7 = *cpi;
                    *cpi = *cpi + 1;
                    *cp.offset(c2rust_fresh7 as isize) = UNICODE_INVALID as uint32_t;
                }
                (*data).this_cp = c as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int;
                (*data).bytes_total = 2 as ::core::ffi::c_int;
                (*data).bytes_remaining = 1 as ::core::ffi::c_int;
            } else if c as ::core::ffi::c_int >= 0xe0 as ::core::ffi::c_int
                && (c as ::core::ffi::c_int) < 0xf0 as ::core::ffi::c_int
            {
                if (*data).bytes_remaining != 0 {
                    let c2rust_fresh8 = *cpi;
                    *cpi = *cpi + 1;
                    *cp.offset(c2rust_fresh8 as isize) = UNICODE_INVALID as uint32_t;
                }
                (*data).this_cp = c as ::core::ffi::c_int & 0xf as ::core::ffi::c_int;
                (*data).bytes_total = 3 as ::core::ffi::c_int;
                (*data).bytes_remaining = 2 as ::core::ffi::c_int;
            } else if c as ::core::ffi::c_int >= 0xf0 as ::core::ffi::c_int
                && (c as ::core::ffi::c_int) < 0xf8 as ::core::ffi::c_int
            {
                if (*data).bytes_remaining != 0 {
                    let c2rust_fresh9 = *cpi;
                    *cpi = *cpi + 1;
                    *cp.offset(c2rust_fresh9 as isize) = UNICODE_INVALID as uint32_t;
                }
                (*data).this_cp = c as ::core::ffi::c_int & 0x7 as ::core::ffi::c_int;
                (*data).bytes_total = 4 as ::core::ffi::c_int;
                (*data).bytes_remaining = 3 as ::core::ffi::c_int;
            } else if c as ::core::ffi::c_int >= 0xf8 as ::core::ffi::c_int
                && (c as ::core::ffi::c_int) < 0xfc as ::core::ffi::c_int
            {
                if (*data).bytes_remaining != 0 {
                    let c2rust_fresh10 = *cpi;
                    *cpi = *cpi + 1;
                    *cp.offset(c2rust_fresh10 as isize) = UNICODE_INVALID as uint32_t;
                }
                (*data).this_cp = c as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int;
                (*data).bytes_total = 5 as ::core::ffi::c_int;
                (*data).bytes_remaining = 4 as ::core::ffi::c_int;
            } else if c as ::core::ffi::c_int >= 0xfc as ::core::ffi::c_int
                && (c as ::core::ffi::c_int) < 0xfe as ::core::ffi::c_int
            {
                if (*data).bytes_remaining != 0 {
                    let c2rust_fresh11 = *cpi;
                    *cpi = *cpi + 1;
                    *cp.offset(c2rust_fresh11 as isize) = UNICODE_INVALID as uint32_t;
                }
                (*data).this_cp = c as ::core::ffi::c_int & 0x1 as ::core::ffi::c_int;
                (*data).bytes_total = 6 as ::core::ffi::c_int;
                (*data).bytes_remaining = 5 as ::core::ffi::c_int;
            } else {
                let c2rust_fresh12 = *cpi;
                *cpi = *cpi + 1;
                *cp.offset(c2rust_fresh12 as isize) = UNICODE_INVALID as uint32_t;
            }
            *pos = (*pos).wrapping_add(1);
        }
    }
}
static encoding_utf8: GlobalCell<VTermEncoding> = GlobalCell::new(VTermEncoding {
    init: Some(
        init_utf8 as unsafe extern "C" fn(*mut VTermEncoding, *mut ::core::ffi::c_void) -> (),
    ),
    decode: Some(
        decode_utf8
            as unsafe extern "C" fn(
                *mut VTermEncoding,
                *mut ::core::ffi::c_void,
                *mut uint32_t,
                *mut ::core::ffi::c_int,
                ::core::ffi::c_int,
                *const ::core::ffi::c_char,
                *mut size_t,
                size_t,
            ) -> (),
    ),
});
unsafe extern "C" fn decode_usascii(
    mut _enc: *mut VTermEncoding,
    mut _data: *mut ::core::ffi::c_void,
    mut cp: *mut uint32_t,
    mut cpi: *mut ::core::ffi::c_int,
    mut cplen: ::core::ffi::c_int,
    mut bytes: *const ::core::ffi::c_char,
    mut pos: *mut size_t,
    mut bytelen: size_t,
) {
    let mut is_gr: ::core::ffi::c_int =
        *bytes.offset(*pos as isize) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int;
    while *pos < bytelen && *cpi < cplen {
        let mut c: uint8_t =
            (*bytes.offset(*pos as isize) as ::core::ffi::c_int ^ is_gr) as uint8_t;
        if (c as ::core::ffi::c_int) < 0x20 as ::core::ffi::c_int
            || c as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int
            || c as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
        {
            return;
        }
        let c2rust_fresh0 = *cpi;
        *cpi = *cpi + 1;
        *cp.offset(c2rust_fresh0 as isize) = c as uint32_t;
        *pos = (*pos).wrapping_add(1);
    }
}
static encoding_usascii: GlobalCell<VTermEncoding> = GlobalCell::new(VTermEncoding {
    init: None,
    decode: Some(
        decode_usascii
            as unsafe extern "C" fn(
                *mut VTermEncoding,
                *mut ::core::ffi::c_void,
                *mut uint32_t,
                *mut ::core::ffi::c_int,
                ::core::ffi::c_int,
                *const ::core::ffi::c_char,
                *mut size_t,
                size_t,
            ) -> (),
    ),
});
unsafe extern "C" fn decode_table(
    mut enc: *mut VTermEncoding,
    mut _data: *mut ::core::ffi::c_void,
    mut cp: *mut uint32_t,
    mut cpi: *mut ::core::ffi::c_int,
    mut cplen: ::core::ffi::c_int,
    mut bytes: *const ::core::ffi::c_char,
    mut pos: *mut size_t,
    mut bytelen: size_t,
) {
    let mut table: *mut StaticTableEncoding = enc as *mut StaticTableEncoding;
    let mut is_gr: ::core::ffi::c_int =
        *bytes.offset(*pos as isize) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int;
    while *pos < bytelen && *cpi < cplen {
        let mut c: uint8_t =
            (*bytes.offset(*pos as isize) as ::core::ffi::c_int ^ is_gr) as uint8_t;
        if (c as ::core::ffi::c_int) < 0x20 as ::core::ffi::c_int
            || c as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int
            || c as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
        {
            return;
        }
        if (*table).chars[c as usize] != 0 {
            let c2rust_fresh1 = *cpi;
            *cpi = *cpi + 1;
            *cp.offset(c2rust_fresh1 as isize) = (*table).chars[c as usize];
        } else {
            let c2rust_fresh2 = *cpi;
            *cpi = *cpi + 1;
            *cp.offset(c2rust_fresh2 as isize) = c as uint32_t;
        }
        *pos = (*pos).wrapping_add(1);
    }
}
static encoding_DECdrawing: GlobalCell<StaticTableEncoding> =
    GlobalCell::new(StaticTableEncoding {
        enc: VTermEncoding {
            init: None,
            decode: Some(
                decode_table
                    as unsafe extern "C" fn(
                        *mut VTermEncoding,
                        *mut ::core::ffi::c_void,
                        *mut uint32_t,
                        *mut ::core::ffi::c_int,
                        ::core::ffi::c_int,
                        *const ::core::ffi::c_char,
                        *mut size_t,
                        size_t,
                    ) -> (),
            ),
        },
        chars: [
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0x25c6 as uint32_t,
            0x2592 as uint32_t,
            0x2409 as uint32_t,
            0x240c as uint32_t,
            0x240d as uint32_t,
            0x240a as uint32_t,
            0xb0 as uint32_t,
            0xb1 as uint32_t,
            0x2424 as uint32_t,
            0x240b as uint32_t,
            0x2518 as uint32_t,
            0x2510 as uint32_t,
            0x250c as uint32_t,
            0x2514 as uint32_t,
            0x253c as uint32_t,
            0x23ba as uint32_t,
            0x23bb as uint32_t,
            0x2500 as uint32_t,
            0x23bc as uint32_t,
            0x23bd as uint32_t,
            0x251c as uint32_t,
            0x2524 as uint32_t,
            0x2534 as uint32_t,
            0x252c as uint32_t,
            0x2502 as uint32_t,
            0x2a7d as uint32_t,
            0x2a7e as uint32_t,
            0x3c0 as uint32_t,
            0x2260 as uint32_t,
            0xa3 as uint32_t,
            0xb7 as uint32_t,
            0,
        ],
    });
static encodings: GlobalCell<[C2Rust_Unnamed; 4]> = GlobalCell::new(unsafe {
    [
        C2Rust_Unnamed {
            type_0: ENC_UTF8,
            designation: 'u' as ::core::ffi::c_char,
            enc: (encoding_utf8.as_raw() as *const _) as *mut VTermEncoding,
        },
        C2Rust_Unnamed {
            type_0: ENC_SINGLE_94,
            designation: '0' as ::core::ffi::c_char,
            enc: (encoding_DECdrawing.as_raw() as *const _) as *mut VTermEncoding,
        },
        C2Rust_Unnamed {
            type_0: ENC_SINGLE_94,
            designation: 'B' as ::core::ffi::c_char,
            enc: (encoding_usascii.as_raw() as *const _) as *mut VTermEncoding,
        },
        C2Rust_Unnamed {
            type_0: ENC_UTF8,
            designation: 0,
            enc: ::core::ptr::null_mut::<VTermEncoding>(),
        },
    ]
});
#[no_mangle]
pub unsafe extern "C" fn vterm_lookup_encoding(
    mut type_0: VTermEncodingType,
    mut designation: ::core::ffi::c_char,
) -> *mut VTermEncoding {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (*encodings.ptr())[i as usize].designation != 0 {
        if (*encodings.ptr())[i as usize].type_0 as ::core::ffi::c_uint
            == type_0 as ::core::ffi::c_uint
            && (*encodings.ptr())[i as usize].designation as ::core::ffi::c_int
                == designation as ::core::ffi::c_int
        {
            return (*encodings.ptr())[i as usize].enc;
        }
        i += 1;
    }
    return ::core::ptr::null_mut::<VTermEncoding>();
}
