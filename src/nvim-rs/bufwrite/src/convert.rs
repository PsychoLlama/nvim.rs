//! Encoding conversion pipeline for buffer writing.
//!
//! Mirrors the C `buf_write_convert_with_iconv`, `buf_write_convert`,
//! and `buf_write_bytes` functions.

#![allow(clippy::cast_lossless)]
#![allow(clippy::too_many_lines)]

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::ffi::{BwInfoHandle, FAIL, OK};

// NL character
const NL: u8 = b'\n';

extern "C" {
    // bw_info field accessors
    fn nvim_bw_info_get_fd(ip: BwInfoHandle) -> c_int;
    fn nvim_bw_info_get_buf(ip: BwInfoHandle) -> *mut c_char;
    fn nvim_bw_info_get_len(ip: BwInfoHandle) -> c_int;
    fn nvim_bw_info_get_flags(ip: BwInfoHandle) -> c_int;
    fn nvim_bw_info_get_restlen(ip: BwInfoHandle) -> c_int;
    fn nvim_bw_info_set_restlen(ip: BwInfoHandle, val: c_int);
    fn nvim_bw_info_get_rest_ptr(ip: BwInfoHandle) -> *mut u8;
    fn nvim_bw_info_get_conv_buf(ip: BwInfoHandle) -> *mut c_char;
    fn nvim_bw_info_get_conv_error(ip: BwInfoHandle) -> c_int;
    fn nvim_bw_info_set_conv_error(ip: BwInfoHandle, val: c_int);
    fn nvim_bw_info_set_conv_error_lnum(ip: BwInfoHandle, val: i32);
    fn nvim_bw_info_get_start_lnum(ip: BwInfoHandle) -> i32;
    fn nvim_bw_info_set_start_lnum(ip: BwInfoHandle, val: i32);
    fn nvim_bw_info_has_iconv(ip: BwInfoHandle) -> c_int;

    // iconv wrappers
    fn nvim_bw_iconv_convert(ip: BwInfoHandle, bufp: *mut *mut c_char, lenp: *mut c_int) -> c_int;

    // mbyte
    #[link_name = "utf_char2bytes"]
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    #[link_name = "utf_ptr2char"]
    fn utf_ptr2char(p: *const c_char) -> c_int;
    #[link_name = "utf_ptr2len_len"]
    fn utf_ptr2len_len(p: *const c_char, len: c_int) -> c_int;

    // I/O
    fn nvim_bw_write_eintr(fd: c_int, buf: *const c_char, len: usize) -> c_int;
}

/// Convert buffer contents using iconv (via C wrapper).
///
/// # Safety
///
/// `ip`, `bufp`, `lenp` must all be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_write_convert_with_iconv(
    ip: BwInfoHandle,
    bufp: *mut *mut c_char,
    lenp: *mut c_int,
) -> c_int {
    // Delegate to C wrapper that handles the iconv complexity directly
    unsafe { nvim_bw_iconv_convert(ip, bufp, lenp) }
}

/// Convert UTF-8 buffer to Latin-1 → UTF-8 encoding (FIO_UTF8 path).
///
/// # Safety
///
/// All pointers must be valid.
unsafe fn convert_utf8_path(ip: BwInfoHandle, bufp: *mut *mut c_char, lenp: *mut c_int) {
    let conv_buf = unsafe { nvim_bw_info_get_conv_buf(ip) };
    let mut p = conv_buf;
    let buf = unsafe { *bufp };
    let len = unsafe { *lenp };
    for wlen in 0..len {
        let byte = unsafe { *buf.add(wlen as usize) } as u8;
        let n = unsafe { utf_char2bytes(c_int::from(byte), p) };
        p = unsafe { p.add(n as usize) };
    }
    unsafe { *bufp = conv_buf };
    unsafe { *lenp = p.offset_from(conv_buf) as c_int };
}

/// Convert UTF-8 buffer to UCS-2/UCS-4/UTF-16/Latin-1 encoding.
///
/// # Safety
///
/// All pointers must be valid.
unsafe fn convert_ucs_path(
    ip: BwInfoHandle,
    flags: c_int,
    bufp: *mut *mut c_char,
    lenp: *mut c_int,
) -> c_int {
    let conv_buf = unsafe { nvim_bw_info_get_conv_buf(ip) };
    let rest_ptr = unsafe { nvim_bw_info_get_rest_ptr(ip) };
    let buf = unsafe { *bufp };
    let len = unsafe { *lenp };

    // translate in-place (can only get shorter) or to buffer
    let mut p = if flags & crate::FIO_LATIN1 as c_int != 0 {
        buf
    } else {
        conv_buf
    };

    let mut wlen: c_int = 0;
    while wlen < len {
        let n;
        let c: u32;

        if wlen == 0 && unsafe { nvim_bw_info_get_restlen(ip) } != 0 {
            let restlen = unsafe { nvim_bw_info_get_restlen(ip) };
            // Use remainder of previous call. Append the start of buf[] to get a full sequence.
            let l = std::cmp::min(len, crate::CONV_RESTLEN as c_int - restlen);
            unsafe {
                ptr::copy(
                    buf.cast::<u8>().cast_const(),
                    rest_ptr.add(restlen as usize),
                    l as usize,
                );
            }
            let seq_len =
                unsafe { utf_ptr2len_len(rest_ptr.cast::<c_char>().cast_const(), restlen + l) };
            if seq_len > restlen + len {
                // Incomplete byte sequence at the end.
                if restlen + len > crate::CONV_RESTLEN as c_int {
                    return FAIL;
                }
                unsafe { nvim_bw_info_set_restlen(ip, restlen + len) };
                break;
            }
            c = if seq_len > 1 {
                (unsafe { utf_ptr2char(rest_ptr.cast::<c_char>().cast_const()) }) as u32
            } else {
                u32::from(unsafe { *rest_ptr })
            };
            if seq_len >= restlen {
                n = seq_len - restlen;
                unsafe { nvim_bw_info_set_restlen(ip, 0) };
            } else {
                unsafe {
                    let new_restlen = restlen - seq_len;
                    ptr::copy(
                        rest_ptr.add(seq_len as usize),
                        rest_ptr,
                        new_restlen as usize,
                    );
                    nvim_bw_info_set_restlen(ip, new_restlen);
                }
                n = 0;
            }
        } else {
            let seq_len = unsafe { utf_ptr2len_len(buf.add(wlen as usize), len - wlen) };
            if seq_len > len - wlen {
                // Incomplete byte sequence at the end.
                let remaining = len - wlen;
                if remaining > crate::CONV_RESTLEN as c_int {
                    return FAIL;
                }
                unsafe {
                    ptr::copy(
                        buf.add(wlen as usize).cast::<u8>().cast_const(),
                        rest_ptr,
                        remaining as usize,
                    );
                    nvim_bw_info_set_restlen(ip, remaining);
                }
                break;
            }
            c = if seq_len > 1 {
                (unsafe { utf_ptr2char(buf.add(wlen as usize)) }) as u32
            } else {
                u32::from(unsafe { *(buf.add(wlen as usize)) } as u8)
            };
            n = seq_len;
        }

        // Convert the character using ucs2bytes logic
        let mut p_u8: *mut u8 = p.cast();
        let error = unsafe { crate::encoding::rs_ucs2bytes(c, &raw mut p_u8, flags) };
        p = p_u8.cast();
        if error != 0 && unsafe { nvim_bw_info_get_conv_error(ip) } == 0 {
            unsafe {
                nvim_bw_info_set_conv_error(ip, 1);
                nvim_bw_info_set_conv_error_lnum(ip, nvim_bw_info_get_start_lnum(ip));
            }
        }
        if c == u32::from(NL) {
            unsafe {
                nvim_bw_info_set_start_lnum(ip, nvim_bw_info_get_start_lnum(ip) + 1);
            }
        }
        wlen += n;
    }

    if flags & crate::FIO_LATIN1 as c_int != 0 {
        unsafe { *lenp = p.offset_from(buf) as c_int };
    } else {
        unsafe {
            *bufp = conv_buf;
            *lenp = p.offset_from(conv_buf) as c_int;
        }
    }

    OK
}

/// Convert buffer data based on encoding flags.
///
/// Replaces C `buf_write_convert`.
///
/// # Safety
///
/// `ip`, `bufp`, `lenp` must all be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_write_convert(
    ip: BwInfoHandle,
    bufp: *mut *mut c_char,
    lenp: *mut c_int,
) -> c_int {
    let flags = unsafe { nvim_bw_info_get_flags(ip) };

    if flags & crate::FIO_UTF8 as c_int != 0 {
        // Convert latin1 in the buffer to UTF-8 in the file.
        unsafe { convert_utf8_path(ip, bufp, lenp) };
    } else if flags
        & (crate::FIO_UCS4 | crate::FIO_UTF16 | crate::FIO_UCS2 | crate::FIO_LATIN1) as c_int
        != 0
        && unsafe { convert_ucs_path(ip, flags, bufp, lenp) } == FAIL
    {
        return FAIL;
    }

    if unsafe { nvim_bw_info_has_iconv(ip) } != 0
        && unsafe { nvim_bw_iconv_convert(ip, bufp, lenp) } == FAIL
    {
        return FAIL;
    }

    OK
}

/// Write buffer data, handling encoding conversion.
///
/// Replaces C `buf_write_bytes`.
///
/// # Safety
///
/// `ip` must be a valid `bw_info` handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_write_bytes(ip: BwInfoHandle) -> c_int {
    let mut buf = unsafe { nvim_bw_info_get_buf(ip) };
    let mut len = unsafe { nvim_bw_info_get_len(ip) };
    let flags = unsafe { nvim_bw_info_get_flags(ip) };

    // Skip conversion when writing the BOM.
    if flags & crate::FIO_NOCONVERT as c_int == 0
        && unsafe { rs_buf_write_convert(ip, &raw mut buf, &raw mut len) } == FAIL
    {
        return FAIL;
    }

    let fd = unsafe { nvim_bw_info_get_fd(ip) };
    if fd < 0 {
        // Only checking conversion, which is OK if we get here.
        return OK;
    }
    let wlen = unsafe { nvim_bw_write_eintr(fd, buf, len as usize) };
    if wlen < len {
        FAIL
    } else {
        OK
    }
}
