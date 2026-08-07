//! Converting text between encodings.
//!
//! `convert_setup` builds a `vimconv_T` describing how to get from one encoding to
//! another, and `string_convert_ext` runs it over a string.  Most pairs go through
//! iconv, which is a real foreign boundary: `my_iconv_open` opens a descriptor
//! (and remembers whether the host's iconv works at all), `iconv_string` pumps it.
//! The Latin-1 and Latin-9 pairs are handled directly, without iconv.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub type WorkingStatus = ::core::ffi::c_uint;

pub const kBroken: WorkingStatus = 2;

pub const kWorking: WorkingStatus = 1;

pub const kUnknown: WorkingStatus = 0;

pub unsafe extern "C" fn my_iconv_open(
    mut to: *mut ::core::ffi::c_char,
    mut from: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_void {
    unsafe {
        let mut tobuf: [::core::ffi::c_char; 400] = [0; 400];
        static iconv_working: GlobalCell<WorkingStatus> = GlobalCell::new(kUnknown);
        if iconv_working.get() as ::core::ffi::c_uint
            == kBroken as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                -1 as ::core::ffi::c_int as usize,
            );
        }
        let mut fd: iconv_t = iconv_open(enc_skip(to), enc_skip(from));
        if fd
            != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                -1 as ::core::ffi::c_int as usize,
            )
            && iconv_working.get() as ::core::ffi::c_uint
                == kUnknown as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut p: *mut ::core::ffi::c_char = &raw mut tobuf as *mut ::core::ffi::c_char;
            let mut tolen: size_t = ICONV_TESTLEN as size_t;
            iconv(
                fd,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                ::core::ptr::null_mut::<size_t>(),
                &raw mut p,
                &raw mut tolen,
            );
            if p.is_null() {
                iconv_working.set(kBroken);
                iconv_close(fd);
                fd = ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                    -1 as ::core::ffi::c_int as usize,
                );
            } else {
                iconv_working.set(kWorking);
            }
        }
        return fd;
    }
}

pub const ICONV_TESTLEN: ::core::ffi::c_int = 400 as ::core::ffi::c_int;

unsafe extern "C" fn iconv_string(
    vcp: *const vimconv_T,
    mut str: *const ::core::ffi::c_char,
    mut slen: size_t,
    mut unconvlenp: *mut size_t,
    mut resultlenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut to: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut len: size_t = 0 as size_t;
        let mut done: size_t = 0 as size_t;
        let mut result: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut from: *const ::core::ffi::c_char = str;
        let mut fromlen: size_t = slen;
        loop {
            if len == 0 as size_t || *__errno_location() == ICONV_E2BIG {
                len = len
                    .wrapping_add(fromlen.wrapping_mul(2 as size_t))
                    .wrapping_add(40 as size_t);
                let mut p: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
                if done > 0 as size_t {
                    memmove(
                        p as *mut ::core::ffi::c_void,
                        result as *const ::core::ffi::c_void,
                        done,
                    );
                }
                xfree(result as *mut ::core::ffi::c_void);
                result = p;
            }
            to = result.offset(done as isize);
            let mut tolen: size_t = len.wrapping_sub(done).wrapping_sub(2 as size_t);
            if iconv(
                (*vcp).vc_fd,
                &raw mut from as *mut ::core::ffi::c_void as *mut *mut ::core::ffi::c_char,
                &raw mut fromlen,
                &raw mut to,
                &raw mut tolen,
            ) != SIZE_MAX as size_t
            {
                *to = NUL as ::core::ffi::c_char;
                break;
            } else if !(*vcp).vc_fail
                && !unconvlenp.is_null()
                && (*__errno_location() == ICONV_EINVAL || *__errno_location() == EINVAL)
            {
                *to = NUL as ::core::ffi::c_char;
                *unconvlenp = fromlen;
                break;
            } else {
                if !(*vcp).vc_fail
                    && (*__errno_location() == ICONV_EILSEQ
                        || *__errno_location() == EILSEQ
                        || *__errno_location() == ICONV_EINVAL
                        || *__errno_location() == EINVAL)
                {
                    let c2rust_fresh10 = to;
                    to = to.offset(1);
                    *c2rust_fresh10 = '?' as ::core::ffi::c_char;
                    if utf_ptr2cells(from) > 1 as ::core::ffi::c_int {
                        let c2rust_fresh11 = to;
                        to = to.offset(1);
                        *c2rust_fresh11 = '?' as ::core::ffi::c_char;
                    }
                    let mut l: ::core::ffi::c_int =
                        utfc_ptr2len_len(from, fromlen as ::core::ffi::c_int);
                    from = from.offset(l as isize);
                    fromlen = fromlen.wrapping_sub(l as size_t);
                } else if *__errno_location() != ICONV_E2BIG {
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        &raw mut result as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    let _ = *ptr_;
                    break;
                }
                done = to.offset_from(result) as size_t;
            }
        }
        if !resultlenp.is_null() && !result.is_null() {
            *resultlenp = to.offset_from(result) as size_t;
        }
        return result;
    }
}

pub unsafe extern "C" fn f_iconv(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut vimconv: vimconv_T = vimconv_T {
            vc_type: 0,
            vc_factor: 0,
            vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            vc_fail: false,
        };
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let str: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
        let from: *mut ::core::ffi::c_char = enc_canonize(enc_skip(tv_get_string_buf(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf1 as *mut ::core::ffi::c_char,
        )
            as *mut ::core::ffi::c_char));
        let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
        let to: *mut ::core::ffi::c_char = enc_canonize(enc_skip(tv_get_string_buf(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut buf2 as *mut ::core::ffi::c_char,
        )
            as *mut ::core::ffi::c_char));
        vimconv.vc_type = CONV_NONE;
        convert_setup(&raw mut vimconv, from, to);
        if vimconv.vc_type == CONV_NONE {
            (*rettv).vval.v_string = xstrdup(str);
        } else {
            (*rettv).vval.v_string = string_convert(
                &raw mut vimconv,
                str as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<size_t>(),
            );
        }
        convert_setup(
            &raw mut vimconv,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
        xfree(from as *mut ::core::ffi::c_void);
        xfree(to as *mut ::core::ffi::c_void);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn convert_setup(
    mut vcp: *mut vimconv_T,
    mut from: *mut ::core::ffi::c_char,
    mut to: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        return convert_setup_ext(vcp, from, true_0 != 0, to, true_0 != 0);
    }
}

pub unsafe extern "C" fn convert_setup_ext(
    mut vcp: *mut vimconv_T,
    mut from: *mut ::core::ffi::c_char,
    mut from_unicode_is_utf8: bool,
    mut to: *mut ::core::ffi::c_char,
    mut to_unicode_is_utf8: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut from_is_utf8: ::core::ffi::c_int = 0;
        let mut to_is_utf8: ::core::ffi::c_int = 0;
        if (*vcp).vc_type == CONV_ICONV
            && (*vcp).vc_fd
                != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                    -1 as ::core::ffi::c_int as usize,
                )
        {
            iconv_close((*vcp).vc_fd);
        }
        *vcp = vimconv_T {
            vc_type: CONV_NONE,
            vc_factor: 1 as ::core::ffi::c_int,
            vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            vc_fail: false_0 != 0,
        };
        if from.is_null()
            || *from as ::core::ffi::c_int == NUL
            || to.is_null()
            || *to as ::core::ffi::c_int == NUL
            || strcmp(from, to) == 0 as ::core::ffi::c_int
        {
            return OK;
        }
        let mut from_prop: ::core::ffi::c_int = enc_canon_props(from);
        let mut to_prop: ::core::ffi::c_int = enc_canon_props(to);
        if from_unicode_is_utf8 {
            from_is_utf8 = from_prop & ENC_UNICODE;
        } else {
            from_is_utf8 = (from_prop == ENC_UNICODE) as ::core::ffi::c_int;
        }
        if to_unicode_is_utf8 {
            to_is_utf8 = to_prop & ENC_UNICODE;
        } else {
            to_is_utf8 = (to_prop == ENC_UNICODE) as ::core::ffi::c_int;
        }
        if from_prop & ENC_LATIN1 != 0 && to_is_utf8 != 0 {
            (*vcp).vc_type = CONV_TO_UTF8;
            (*vcp).vc_factor = 2 as ::core::ffi::c_int;
        } else if from_prop & ENC_LATIN9 != 0 && to_is_utf8 != 0 {
            (*vcp).vc_type = CONV_9_TO_UTF8;
            (*vcp).vc_factor = 3 as ::core::ffi::c_int;
        } else if from_is_utf8 != 0 && to_prop & ENC_LATIN1 != 0 {
            (*vcp).vc_type = CONV_TO_LATIN1;
        } else if from_is_utf8 != 0 && to_prop & ENC_LATIN9 != 0 {
            (*vcp).vc_type = CONV_TO_LATIN9;
        } else {
            (*vcp).vc_fd = my_iconv_open(
                (if to_is_utf8 != 0 {
                    b"utf-8\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    to as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char,
                (if from_is_utf8 != 0 {
                    b"utf-8\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    from as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char,
            );
            if (*vcp).vc_fd
                != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                    -1 as ::core::ffi::c_int as usize,
                )
            {
                (*vcp).vc_type = CONV_ICONV;
                (*vcp).vc_factor = 4 as ::core::ffi::c_int;
            }
        }
        if (*vcp).vc_type == CONV_NONE {
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn string_convert(
    vcp: *const vimconv_T,
    mut ptr: *mut ::core::ffi::c_char,
    mut lenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    unsafe {
        return string_convert_ext(vcp, ptr, lenp, ::core::ptr::null_mut::<size_t>());
    }
}

pub unsafe extern "C" fn string_convert_ext(
    vcp: *const vimconv_T,
    mut ptr: *mut ::core::ffi::c_char,
    mut lenp: *mut size_t,
    mut unconvlenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut retval: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
        let mut d: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
        let mut c: ::core::ffi::c_int = 0;
        let mut len: size_t = 0;
        if lenp.is_null() {
            len = strlen(ptr);
        } else {
            len = *lenp;
        }
        if len == 0 as size_t {
            return xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
        }
        match (*vcp).vc_type {
            1 => {
                retval = xmalloc(len.wrapping_mul(2 as size_t).wrapping_add(1 as size_t))
                    as *mut uint8_t;
                d = retval;
                let mut i: size_t = 0 as size_t;
                while i < len {
                    c = *ptr.offset(i as isize) as uint8_t as ::core::ffi::c_int;
                    if c < 0x80 as ::core::ffi::c_int {
                        let c2rust_fresh2 = d;
                        d = d.offset(1);
                        *c2rust_fresh2 = c as uint8_t;
                    } else {
                        let c2rust_fresh3 = d;
                        d = d.offset(1);
                        *c2rust_fresh3 = (0xc0 as ::core::ffi::c_int
                            + (c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as uint8_t
                                as ::core::ffi::c_int)
                            as uint8_t;
                        let c2rust_fresh4 = d;
                        d = d.offset(1);
                        *c2rust_fresh4 = (0x80 as ::core::ffi::c_int
                            + (c & 0x3f as ::core::ffi::c_int))
                            as uint8_t;
                    }
                    i = i.wrapping_add(1);
                }
                *d = NUL as uint8_t;
                if !lenp.is_null() {
                    *lenp = d.offset_from(retval) as size_t;
                }
            }
            2 => {
                retval = xmalloc(len.wrapping_mul(3 as size_t).wrapping_add(1 as size_t))
                    as *mut uint8_t;
                d = retval;
                let mut i_0: size_t = 0 as size_t;
                while i_0 < len {
                    c = *ptr.offset(i_0 as isize) as uint8_t as ::core::ffi::c_int;
                    match c {
                        164 => {
                            c = 0x20ac as ::core::ffi::c_int;
                        }
                        166 => {
                            c = 0x160 as ::core::ffi::c_int;
                        }
                        168 => {
                            c = 0x161 as ::core::ffi::c_int;
                        }
                        180 => {
                            c = 0x17d as ::core::ffi::c_int;
                        }
                        184 => {
                            c = 0x17e as ::core::ffi::c_int;
                        }
                        188 => {
                            c = 0x152 as ::core::ffi::c_int;
                        }
                        189 => {
                            c = 0x153 as ::core::ffi::c_int;
                        }
                        190 => {
                            c = 0x178 as ::core::ffi::c_int;
                        }
                        _ => {}
                    }
                    d = d.offset(utf_char2bytes(c, d as *mut ::core::ffi::c_char) as isize);
                    i_0 = i_0.wrapping_add(1);
                }
                *d = NUL as uint8_t;
                if !lenp.is_null() {
                    *lenp = d.offset_from(retval) as size_t;
                }
            }
            3 | 4 => {
                retval = xmalloc(len.wrapping_add(1 as size_t)) as *mut uint8_t;
                d = retval;
                let mut i_1: size_t = 0 as size_t;
                while i_1 < len {
                    let mut l: ::core::ffi::c_int = utf_ptr2len_len(
                        ptr.offset(i_1 as isize),
                        len.wrapping_sub(i_1) as ::core::ffi::c_int,
                    );
                    if l == 0 as ::core::ffi::c_int {
                        let c2rust_fresh5 = d;
                        d = d.offset(1);
                        *c2rust_fresh5 = NUL as uint8_t;
                    } else if l == 1 as ::core::ffi::c_int {
                        let mut l_w: uint8_t = (*utf8len_tab_zero.ptr())
                            [*ptr.offset(i_1 as isize) as uint8_t as usize];
                        if l_w as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                            xfree(retval as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<::core::ffi::c_char>();
                        }
                        if !unconvlenp.is_null() && l_w as size_t > len.wrapping_sub(i_1) {
                            *unconvlenp = len.wrapping_sub(i_1);
                            break;
                        } else {
                            let c2rust_fresh6 = d;
                            d = d.offset(1);
                            *c2rust_fresh6 = *ptr.offset(i_1 as isize) as uint8_t;
                        }
                    } else {
                        c = utf_ptr2char(ptr.offset(i_1 as isize));
                        if (*vcp).vc_type == CONV_TO_LATIN9 {
                            match c {
                                8364 => {
                                    c = 0xa4 as ::core::ffi::c_int;
                                }
                                352 => {
                                    c = 0xa6 as ::core::ffi::c_int;
                                }
                                353 => {
                                    c = 0xa8 as ::core::ffi::c_int;
                                }
                                381 => {
                                    c = 0xb4 as ::core::ffi::c_int;
                                }
                                382 => {
                                    c = 0xb8 as ::core::ffi::c_int;
                                }
                                338 => {
                                    c = 0xbc as ::core::ffi::c_int;
                                }
                                339 => {
                                    c = 0xbd as ::core::ffi::c_int;
                                }
                                376 => {
                                    c = 0xbe as ::core::ffi::c_int;
                                }
                                164 | 166 | 168 | 180 | 184 | 188 | 189 | 190 => {
                                    c = 0x100 as ::core::ffi::c_int;
                                }
                                _ => {}
                            }
                        }
                        if !utf_iscomposing_legacy(c) {
                            if c < 0x100 as ::core::ffi::c_int {
                                let c2rust_fresh7 = d;
                                d = d.offset(1);
                                *c2rust_fresh7 = c as uint8_t;
                            } else if (*vcp).vc_fail {
                                xfree(retval as *mut ::core::ffi::c_void);
                                return ::core::ptr::null_mut::<::core::ffi::c_char>();
                            } else {
                                let c2rust_fresh8 = d;
                                d = d.offset(1);
                                *c2rust_fresh8 = 0xbf as uint8_t;
                                if utf_char2cells(c) > 1 as ::core::ffi::c_int {
                                    let c2rust_fresh9 = d;
                                    d = d.offset(1);
                                    *c2rust_fresh9 = '?' as uint8_t;
                                }
                            }
                        }
                        i_1 = i_1.wrapping_add((l as size_t).wrapping_sub(1 as size_t));
                    }
                    i_1 = i_1.wrapping_add(1);
                }
                *d = NUL as uint8_t;
                if !lenp.is_null() {
                    *lenp = d.offset_from(retval) as size_t;
                }
            }
            5 => {
                retval = iconv_string(vcp, ptr, len, unconvlenp, lenp) as *mut uint8_t;
            }
            _ => {}
        }
        return retval as *mut ::core::ffi::c_char;
    }
}

pub const E2BIG: ::core::ffi::c_int = 7 as ::core::ffi::c_int;

pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;

pub const ICONV_E2BIG: ::core::ffi::c_int = E2BIG;

pub const ICONV_EINVAL: ::core::ffi::c_int = EINVAL;

pub const ICONV_EILSEQ: ::core::ffi::c_int = EILSEQ;

pub const EILSEQ: ::core::ffi::c_int = 84 as ::core::ffi::c_int;
