//! [`vim_fgets`]: `fgets` over a C stream, for the readers that still hold
//! one.
//!
//! `spellfile/` (the `:mkspell` inputs and `:spellgood`'s word file),
//! `diff/`, `search/`, `insexpand/` and `spellsuggest/` read lines through
//! it. [`Fgets::line_capped`](super::Fgets::line_capped) is the same reader
//! over any `BufRead`; this goes when its last caller holds one.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use ::libc::{__errno_location, feof, ferror, fgets};

use crate::types::FILE;
use core::ffi::{c_char, c_int};

/// The scratch line [`vim_fgets`] throws an over-long line's rest into.
const TBUF_LEN: c_int = 200;

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
