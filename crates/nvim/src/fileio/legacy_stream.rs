//! The `FILE *` forms of the byte helpers in [`stream`](super::stream), for
//! the readers that still hold a C stream.
//!
//! `spellfile/` reads and writes its `.spl`/`.sug` files through these, and
//! `diff/`, `search/`, `insexpand/` and `spellsuggest/` read lines with
//! [`vim_fgets`]. Each byte helper is a shim over its `Read`/`Write` form
//! with [`CStream`] as the reader; `vim_fgets` stays on `fgets` itself,
//! which reads a whole line per call where a `Read` over `getc` would take
//! a call per byte. Every one of them goes when its last caller holds a
//! `Read` or `Write` instead.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use std::io::{Read, Write};

use ::libc::{__errno_location, feof, ferror, fgets};

use super::stream::{read_be, read_counted, write_be, write_time};
use crate::memory::xmemdupz;
use crate::os::cshim::{getc, putc};
use crate::types::{FAIL, FILE, size_t, time_t, uintmax_t};
use core::ffi::{c_char, c_int};
use core::ptr;

/// `EOF` as `getc`/`putc` answer it.
const EOF: c_int = -1;

/// The scratch line [`vim_fgets`] throws an over-long line's rest into.
const TBUF_LEN: c_int = 200;

/// A C stream as a `Read` and a `Write`, a byte at a time through `getc` and
/// `putc` -- exactly the calls the helpers made before they were shims.
struct CStream(*mut FILE);

impl Read for CStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for (n, slot) in buf.iter_mut().enumerate() {
            // SAFETY: the caller's live stream (see each shim's contract).
            let c = unsafe { getc(self.0) };
            if c == EOF {
                return Ok(n);
            }
            *slot = u8::try_from(c).expect("getc answers a byte or EOF");
        }
        Ok(buf.len())
    }
}

impl Write for CStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for (n, &byte) in buf.iter().enumerate() {
            // SAFETY: the caller's live stream (see each shim's contract).
            if unsafe { putc(c_int::from(byte), self.0) } == EOF {
                return if n == 0 {
                    Err(std::io::ErrorKind::WriteZero.into())
                } else {
                    Ok(n)
                };
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Like `fgets()`, but a line longer than the buffer is truncated and the rest
/// of it thrown away.
///
/// @return  true for EOF or error
///
/// # Safety
///
/// `buf` must point at `size` bytes the caller owns, readable and writable,
/// unaliased for the call. `stream` must point at a live `FILE`, unaliased
/// for the call.
pub unsafe fn vim_fgets(buf: *mut c_char, size: c_int, stream: *mut FILE) -> bool {
    debug_assert!(size > 0);
    // The last-but-one byte tells us whether the line fitted: `fgets`
    // leaves it alone if the line was shorter than the buffer.
    let last = (size - 2) as isize;
    unsafe { *buf.offset(last) = 0 };

    let mut retval;
    loop {
        unsafe { *__errno_location() = 0 };
        retval = unsafe { fgets(buf, size, stream) };
        if !(retval.is_null()
            && unsafe { *__errno_location() } == ::libc::EINTR
            && unsafe { ferror(stream) } != 0)
        {
            break;
        }
    }

    let filled = |c: c_char| c != 0 && c != b'\n'.cast_signed();
    if filled(unsafe { *buf.offset(last) }) {
        // SAFETY: `buf` holds `size` bytes and `last + 1` is inside them.
        unsafe { buf.offset(last + 1).write(0) }; // truncate the line

        // Now throw away the rest of the line.
        let mut tbuf = [0u8; TBUF_LEN as usize];
        let tlast = tbuf.len() - 2;
        loop {
            tbuf[tlast] = 0;
            unsafe { *__errno_location() = 0 };
            let into = tbuf.as_mut_ptr().cast::<c_char>();
            retval = unsafe { fgets(into, TBUF_LEN, stream) };
            if retval.is_null()
                && (unsafe { feof(stream) } != 0 || unsafe { *__errno_location() } != ::libc::EINTR)
            {
                break;
            }
            if !filled(tbuf[tlast].cast_signed()) {
                break;
            }
        }
    }
    retval.is_null()
}

/// Read 2 bytes from `fd` and turn them into an int, MSB first.
///
/// @return  -1 when encountering EOF.
///
/// # Safety
///
/// `fd` must point at a live `FILE`, unaliased for the call.
pub unsafe fn get2c(fd: *mut FILE) -> c_int {
    read_be(&mut CStream(fd), 2).map_or(-1, |n| c_int::try_from(n).expect("two bytes"))
}

/// Read 3 bytes from `fd` and turn them into an int, MSB first.
///
/// @return  -1 when encountering EOF.
///
/// # Safety
///
/// `fd` must point at a live `FILE`, unaliased for the call.
pub unsafe fn get3c(fd: *mut FILE) -> c_int {
    read_be(&mut CStream(fd), 3).map_or(-1, |n| c_int::try_from(n).expect("three bytes"))
}

/// Read 4 bytes from `fd` and turn them into an int, MSB first.
///
/// The result wraps around rather than saturating when the top bit is set,
/// which is what upstream's unsigned accumulator gives.
///
/// @return  -1 when encountering EOF.
///
/// # Safety
///
/// `fd` must point at a live `FILE`, unaliased for the call.
pub unsafe fn get4c(fd: *mut FILE) -> c_int {
    read_be(&mut CStream(fd), 4).map_or(-1, |n| u32::try_from(n).expect("four bytes").cast_signed())
}

/// Read 8 bytes from `fd` and turn them into a `time_t`, MSB first.
///
/// @return  -1 when encountering EOF.
///
/// # Safety
///
/// `fd` must point at a live `FILE`, unaliased for the call.
pub unsafe fn get8ctime(fd: *mut FILE) -> time_t {
    read_be(&mut CStream(fd), 8).map_or(-1, u64::cast_signed)
}

/// Read a string of length `cnt` from `fd` into allocated memory.
///
/// @return  the string, or NULL when unable to read that many bytes.
///
/// # Safety
///
/// `fd` must point at a live `FILE`, unaliased for the call.
pub unsafe fn read_string(fd: *mut FILE, cnt: size_t) -> *mut c_char {
    match read_counted(&mut CStream(fd), cnt) {
        // SAFETY: `cnt` readable bytes, copied with a NUL after them.
        Some(bytes) => unsafe { xmemdupz(bytes.as_ptr().cast(), cnt) }.cast(),
        None => ptr::null_mut(),
    }
}

/// Write `number` to `fd` in `len` bytes, most significant byte first.
///
/// @return  false in case of an error.
///
/// # Safety
///
/// `fd` must point at a live `FILE`, unaliased for the call.
pub unsafe fn put_bytes(fd: *mut FILE, number: uintmax_t, len: size_t) -> bool {
    write_be(&mut CStream(fd), number, len)
}

/// Write a `time_t` to `fd` in 8 bytes.
///
/// @return  FAIL when the write failed.
///
/// # Safety
///
/// `fd` must point at a live `FILE`, unaliased for the call.
pub unsafe fn put_time(fd: *mut FILE, time_: time_t) -> c_int {
    // Upstream compares `fwrite`'s item count against 1 while asking it for
    // 8 one-byte items, so this always answers FAIL. Both callers ignore the
    // answer. Preserved.
    let _ = write_time(&mut CStream(fd), time_);
    FAIL
}
