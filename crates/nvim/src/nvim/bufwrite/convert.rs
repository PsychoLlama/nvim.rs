//! Turning buffer text into file bytes.
//!
//! The buffer is always UTF-8; the file is whatever `'fileencoding'` says. The
//! conversions Nvim knows itself — UCS-2, UCS-4, UTF-16 and Latin1, in either
//! endianness — go through `ucs2bytes` one character at a time; everything else
//! is handed to iconv. `buf_write_bytes` is the funnel both sides come out of:
//! it converts a chunk, writes it, and keeps the trailing partial character for
//! the next call.
//!
//! `make_bom` writes the byte-order mark, which is the same encoder applied to
//! U+FEFF.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ucs2bytes(
    mut c: ::core::ffi::c_uint,
    mut pp: *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut p: *mut uint8_t = *pp as *mut uint8_t;
        let mut error: bool = false_0 != 0;
        if flags & FIO_UCS4 as ::core::ffi::c_int != 0 {
            if flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
                let c2rust_fresh3 = p;
                p = p.offset(1);
                *c2rust_fresh3 = c as uint8_t;
                let c2rust_fresh4 = p;
                p = p.offset(1);
                *c2rust_fresh4 = (c >> 8 as ::core::ffi::c_int) as uint8_t;
                let c2rust_fresh5 = p;
                p = p.offset(1);
                *c2rust_fresh5 = (c >> 16 as ::core::ffi::c_int) as uint8_t;
                let c2rust_fresh6 = p;
                p = p.offset(1);
                *c2rust_fresh6 = (c >> 24 as ::core::ffi::c_int) as uint8_t;
            } else {
                let c2rust_fresh7 = p;
                p = p.offset(1);
                *c2rust_fresh7 = (c >> 24 as ::core::ffi::c_int) as uint8_t;
                let c2rust_fresh8 = p;
                p = p.offset(1);
                *c2rust_fresh8 = (c >> 16 as ::core::ffi::c_int) as uint8_t;
                let c2rust_fresh9 = p;
                p = p.offset(1);
                *c2rust_fresh9 = (c >> 8 as ::core::ffi::c_int) as uint8_t;
                let c2rust_fresh10 = p;
                p = p.offset(1);
                *c2rust_fresh10 = c as uint8_t;
            }
        } else if flags & (FIO_UCS2 as ::core::ffi::c_int | FIO_UTF16 as ::core::ffi::c_int) != 0 {
            if c >= 0x10000 as ::core::ffi::c_int as ::core::ffi::c_uint {
                if flags & FIO_UTF16 as ::core::ffi::c_int != 0 {
                    c = c.wrapping_sub(0x10000 as ::core::ffi::c_int as ::core::ffi::c_uint);
                    if c >= 0x100000 as ::core::ffi::c_int as ::core::ffi::c_uint {
                        error = true_0 != 0;
                    }
                    let mut cc: ::core::ffi::c_int = (c >> 10 as ::core::ffi::c_int
                        & 0x3ff as ::core::ffi::c_uint)
                        .wrapping_add(0xd800 as ::core::ffi::c_uint)
                        as ::core::ffi::c_int;
                    if flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
                        let c2rust_fresh11 = p;
                        p = p.offset(1);
                        *c2rust_fresh11 = cc as uint8_t;
                        let c2rust_fresh12 = p;
                        p = p.offset(1);
                        *c2rust_fresh12 = (cc >> 8 as ::core::ffi::c_int) as uint8_t;
                    } else {
                        let c2rust_fresh13 = p;
                        p = p.offset(1);
                        *c2rust_fresh13 = (cc >> 8 as ::core::ffi::c_int) as uint8_t;
                        let c2rust_fresh14 = p;
                        p = p.offset(1);
                        *c2rust_fresh14 = cc as uint8_t;
                    }
                    c = (c & 0x3ff as ::core::ffi::c_uint)
                        .wrapping_add(0xdc00 as ::core::ffi::c_uint);
                } else {
                    error = true_0 != 0;
                }
            }
            if flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
                let c2rust_fresh15 = p;
                p = p.offset(1);
                *c2rust_fresh15 = c as uint8_t;
                let c2rust_fresh16 = p;
                p = p.offset(1);
                *c2rust_fresh16 = (c >> 8 as ::core::ffi::c_int) as uint8_t;
            } else {
                let c2rust_fresh17 = p;
                p = p.offset(1);
                *c2rust_fresh17 = (c >> 8 as ::core::ffi::c_int) as uint8_t;
                let c2rust_fresh18 = p;
                p = p.offset(1);
                *c2rust_fresh18 = c as uint8_t;
            }
        } else if c >= 0x100 as ::core::ffi::c_uint {
            error = true_0 != 0;
            let c2rust_fresh19 = p;
            p = p.offset(1);
            *c2rust_fresh19 = 0xbf as uint8_t;
        } else {
            let c2rust_fresh20 = p;
            p = p.offset(1);
            *c2rust_fresh20 = c as uint8_t;
        }
        *pp = p as *mut ::core::ffi::c_char;
        return error;
    }
}

pub(crate) unsafe extern "C" fn buf_write_convert_with_iconv(
    mut ip: *mut bw_info,
    mut bufp: *mut *mut ::core::ffi::c_char,
    mut lenp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut len: ::core::ffi::c_int = *lenp;
        let mut from: *const ::core::ffi::c_char = *bufp;
        let mut fromlen: size_t = len as size_t;
        let mut tolen: size_t = (*ip).bw_conv_buflen;
        let mut to: *mut ::core::ffi::c_char = (*ip).bw_conv_buf;
        if (*ip).bw_first != 0 {
            let mut save_len: size_t = tolen;
            iconv(
                (*ip).bw_iconv_fd,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                ::core::ptr::null_mut::<size_t>(),
                &raw mut to,
                &raw mut tolen,
            );
            if to.is_null() {
                to = (*ip).bw_conv_buf;
                tolen = save_len;
            }
            (*ip).bw_first = false_0;
        }
        if iconv(
            (*ip).bw_iconv_fd,
            &raw mut from as *mut ::core::ffi::c_void as *mut *mut ::core::ffi::c_char,
            &raw mut fromlen,
            &raw mut to,
            &raw mut tolen,
        ) == -1 as ::core::ffi::c_int as size_t
            && *__errno_location() != ICONV_EINVAL
        {
            (*ip).bw_conv_error = true_0;
            return -1 as ::core::ffi::c_int;
        }
        *bufp = (*ip).bw_conv_buf;
        *lenp = to.offset_from((*ip).bw_conv_buf) as ::core::ffi::c_int;
        return len - fromlen as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn buf_write_convert(
    mut ip: *mut bw_info,
    mut bufp: *mut *mut ::core::ffi::c_char,
    mut lenp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut flags: ::core::ffi::c_int = (*ip).bw_flags;
        let mut wlen: ::core::ffi::c_int = *lenp;
        if flags
            & (FIO_UCS4 as ::core::ffi::c_int
                | FIO_UTF16 as ::core::ffi::c_int
                | FIO_UCS2 as ::core::ffi::c_int
                | FIO_LATIN1 as ::core::ffi::c_int)
            != 0
        {
            let mut c: ::core::ffi::c_uint = 0;
            let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut p: *mut ::core::ffi::c_char = if flags & FIO_LATIN1 as ::core::ffi::c_int != 0 {
                *bufp
            } else {
                (*ip).bw_conv_buf
            };
            wlen = 0 as ::core::ffi::c_int;
            while wlen < *lenp {
                n = utf_ptr2len_len((*bufp).offset(wlen as isize), *lenp - wlen);
                if n > *lenp - wlen {
                    break;
                }
                c = if n > 1 as ::core::ffi::c_int {
                    utf_ptr2char((*bufp).offset(wlen as isize)) as ::core::ffi::c_uint
                } else {
                    *(*bufp).offset(wlen as isize) as uint8_t as ::core::ffi::c_uint
                };
                if flags & FIO_LATIN1 as ::core::ffi::c_int == 0 {
                    let mut need: size_t = (if flags & FIO_UCS4 as ::core::ffi::c_int != 0 {
                        4 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    }) as size_t;
                    if flags & FIO_UTF16 as ::core::ffi::c_int != 0
                        && c >= 0x10000 as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        need = 4 as size_t;
                    }
                    if (p.offset_from((*ip).bw_conv_buf) as size_t).wrapping_add(need)
                        > (*ip).bw_conv_buflen
                    {
                        return FAIL;
                    }
                }
                if ucs2bytes(c, &raw mut p, flags) as ::core::ffi::c_int != 0
                    && (*ip).bw_conv_error == 0
                {
                    (*ip).bw_conv_error = true_0;
                    (*ip).bw_conv_error_lnum = (*ip).bw_start_lnum;
                }
                if c == NL as ::core::ffi::c_uint {
                    (*ip).bw_start_lnum += 1;
                }
                wlen += n;
            }
            if flags & FIO_LATIN1 as ::core::ffi::c_int != 0 {
                *lenp = p.offset_from(*bufp) as ::core::ffi::c_int;
            } else {
                *bufp = (*ip).bw_conv_buf;
                *lenp = p.offset_from((*ip).bw_conv_buf) as ::core::ffi::c_int;
            }
        }
        if (*ip).bw_iconv_fd
            != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                -1 as ::core::ffi::c_int as usize,
            )
        {
            return buf_write_convert_with_iconv(ip, bufp, lenp);
        }
        return wlen;
    }
}

pub(crate) unsafe extern "C" fn buf_write_bytes(mut ip: *mut bw_info) -> ::core::ffi::c_int {
    unsafe {
        let mut buf: *mut ::core::ffi::c_char = (*ip).bw_buf;
        let mut len: ::core::ffi::c_int = (*ip).bw_len;
        let mut flags: ::core::ffi::c_int = (*ip).bw_flags;
        let mut converted: ::core::ffi::c_int = len;
        let mut remaining: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if flags & FIO_NOCONVERT as ::core::ffi::c_int == 0 {
            converted = buf_write_convert(ip, &raw mut buf, &raw mut len);
            if converted < 0 as ::core::ffi::c_int {
                return FAIL;
            }
            remaining = (*ip).bw_len - converted;
        }
        (*ip).bw_len = remaining;
        if (*ip).bw_fd >= 0 as ::core::ffi::c_int {
            let mut wlen: ::core::ffi::c_int =
                write_eintr((*ip).bw_fd, buf as *mut ::core::ffi::c_void, len as size_t)
                    as ::core::ffi::c_int;
            if wlen < len {
                return FAIL;
            }
        }
        if remaining > 0 as ::core::ffi::c_int {
            memmove(
                (*ip).bw_buf as *mut ::core::ffi::c_void,
                (*ip).bw_buf.offset(converted as isize) as *const ::core::ffi::c_void,
                remaining as size_t,
            );
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn make_bom(
    mut buf_in: *mut ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut buf: *mut uint8_t = buf_in as *mut uint8_t;
        let mut flags: ::core::ffi::c_int = get_fio_flags(name);
        if flags == FIO_LATIN1 as ::core::ffi::c_int || flags == 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if flags == FIO_UTF8 as ::core::ffi::c_int {
            *buf.offset(0 as ::core::ffi::c_int as isize) = 0xef as uint8_t;
            *buf.offset(1 as ::core::ffi::c_int as isize) = 0xbb as uint8_t;
            *buf.offset(2 as ::core::ffi::c_int as isize) = 0xbf as uint8_t;
            return 3 as ::core::ffi::c_int;
        }
        let mut p: *mut ::core::ffi::c_char = buf as *mut ::core::ffi::c_char;
        ucs2bytes(0xfeff as ::core::ffi::c_uint, &raw mut p, flags);
        return (p as *mut uint8_t).offset_from(buf) as ::core::ffi::c_int;
    }
}
