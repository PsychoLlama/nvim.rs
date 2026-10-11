//! The pointer fronts: `vim_snprintf` and friends as the rest of the tree
//! still calls them, into a raw buffer, with a raw format and raw `%s`
//! pointers, and the `vim_snprintf!` macros that build their argument
//! arrays.
//!
//! This is the one file of `strings` that reads through a pointer. The
//! formatter itself (`emit`) takes slices; what it cannot check -- that the
//! destination really has `str_m` bytes, that the format and every pointer a
//! `%s` reads are NUL-terminated -- is each front's `# Safety` contract,
//! and [`Values`] is how that promise reaches the one read that relies on it.
//! The fronts go when their callers hand the formatter owned buffers and
//! owned text (`format_into` over a slice is already there).
//!
//! The macros are the type gate the C variadic never had. Each argument
//! goes through `FmtArg::from`, and an owner has no conversion, so it is a
//! compile error at the call site rather than two words of garbage in the
//! argument area (`spell_load_lang` once handed `vim_snprintf` an
//! `XString`):
//!
//! ```compile_fail,E0277
//! use core::ffi::c_char;
//! use neovim::{memory::XString, vim_snprintf};
//! fn spell_file(buf: *mut c_char, room: usize, lang: *const c_char, enc: &XString) {
//!     unsafe { vim_snprintf!(buf, room, c"spell/%s.%s.spl".as_ptr(), lang, enc) };
//! }
//! ```
//!
//! The array is built where the macro is written, on the stack: a call
//! allocates nothing for its arguments.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int};
use core::slice;

use super::FmtArg;
use super::printf::{Source, format, length_as_int};
use crate::cstr;
use crate::memory::xmemscan;
use crate::types::String_0;

/// The value arguments of one call, and whether the caller vouched for the
/// raw pointers among them.
///
/// Built trusting only by the fronts below, whose callers promise that every
/// [`FmtArg::Ptr`] a `%s` reads is a NUL-terminated string -- or, under a
/// precision, has that many readable bytes or a NUL first -- alive for the
/// call. Everywhere else a pointer `%s` is `None`, which prints as `[NULL]`.
pub(in crate::strings) struct Values<'a> {
    args: &'a [FmtArg<'a>],
    trusted: bool,
}

impl<'a> Values<'a> {
    /// Arguments whose raw pointers are not to be read.
    pub(in crate::strings) fn new(args: &'a [FmtArg<'a>]) -> Self {
        Values {
            args,
            trusted: false,
        }
    }

    /// The `idx`-th argument (zero-based), if the caller passed one.
    pub(in crate::strings) fn get(&self, idx: usize) -> Option<FmtArg<'a>> {
        self.args.get(idx).copied()
    }

    /// `arg`'s text for `%s`/`%S`: `None` for a null pointer or one this
    /// call may not read. `max` bounds how far a pointer is read; without
    /// one it is read to its NUL.
    pub(in crate::strings) fn text(&self, arg: FmtArg<'a>, max: Option<usize>) -> Option<&'a [u8]> {
        match arg {
            FmtArg::Str(text) => Some(text),
            FmtArg::Ptr(at) if self.trusted && !at.is_null() => {
                let at = at.cast::<c_char>();
                Some(match max {
                    // SAFETY: a trusting `Values` is built only by a front
                    // whose caller promised a NUL-terminated string here.
                    None => unsafe { cstr::bytes_at(at) },
                    Some(cap) => {
                        // SAFETY: as above, and under a precision the
                        // promise is `cap` readable bytes or a NUL first,
                        // which is what the scan reads.
                        let end = unsafe { xmemscan(at.cast(), 0, cap) };
                        let len = end.addr() - at.addr();
                        // SAFETY: the `len` bytes the scan just read.
                        unsafe { slice::from_raw_parts(at.cast::<u8>(), len) }
                    }
                })
            }
            _ => None,
        }
    }
}

/// The `str_m` bytes at `str` as a slice; empty when there are none, so a
/// measuring call may pass a null `str`.
///
/// # Safety
///
/// Unless `str_m` is zero, `str` must point at `str_m` writable bytes the
/// caller owns, unaliased for `'o`.
unsafe fn destination<'o>(str: *mut c_char, str_m: usize) -> &'o mut [u8] {
    if str_m == 0 {
        return &mut [];
    }
    // SAFETY: the caller's promise.
    unsafe { slice::from_raw_parts_mut(str.cast::<u8>(), str_m) }
}

/// The format's bytes; a null format is the empty one.
///
/// # Safety
///
/// `fmt` must be null or point at a NUL-terminated string that outlives `'f`.
unsafe fn format_text<'f>(fmt: *const c_char) -> &'f [u8] {
    if fmt.is_null() {
        return b"";
    }
    // SAFETY: the caller's promise.
    unsafe { CStr::from_ptr(fmt) }.to_bytes()
}

/// Write `fmt` formatted with `args` to `str`.
///
/// Returns the number of bytes, excluding the NUL, that *would* have been
/// written had `str_m` been large enough -- which is why it is not safe to
/// use as a buffer offset. See `vim_snprintf_safelen`.
///
/// # Safety
///
/// `fmt` must be null or point at a NUL-terminated format, and every
/// [`FmtArg::Ptr`] it reads with `%s`/`%S` must point at a NUL-terminated
/// string (under a `%.Ns` precision: at `N` readable bytes or a NUL first).
/// Unless `str_m` is zero, `str` must point at `str_m` writable bytes the
/// caller owns, unaliased for the call; it is left NUL-terminated.
pub unsafe fn vim_snprintf(
    str: *mut c_char,
    str_m: usize,
    fmt: *const c_char,
    args: &[FmtArg<'_>],
) -> c_int {
    // SAFETY: the caller's promises, one apiece.
    let (out, fmt) = unsafe { (destination(str, str_m), format_text(fmt)) };
    let values = Values {
        args,
        trusted: true,
    };
    length_as_int(format(out, fmt, Source::Values(values)))
}

/// Append `fmt` formatted with `args` to the string already in `str`.
///
/// # Safety
///
/// As [`vim_snprintf`], and `str` must hold a NUL-terminated string within
/// its `str_m` bytes; the result is appended to it.
pub unsafe fn vim_snprintf_add(
    str: *mut c_char,
    str_m: usize,
    fmt: *const c_char,
    args: &[FmtArg<'_>],
) -> c_int {
    // SAFETY: the caller's terminated string.
    let len = unsafe { cstr::bytes_at(str) }.len();
    let space = str_m.saturating_sub(len);
    // SAFETY: the rest of the caller's buffer, past its string.
    unsafe { vim_snprintf(str.add(len), space, fmt, args) }
}

/// Like [`vim_snprintf`] but with a return value that can safely increment a
/// buffer length: never greater than `str_m - 1`.
///
/// # Safety
///
/// As [`vim_snprintf`].
pub unsafe fn vim_snprintf_safelen(
    str: *mut c_char,
    str_m: usize,
    fmt: *const c_char,
    args: &[FmtArg<'_>],
) -> usize {
    if str_m == 0 {
        return 0;
    }
    // SAFETY: the caller's promises.
    let str_l = unsafe { vim_snprintf(str, str_m, fmt, args) };
    let Ok(str_l) = usize::try_from(str_l) else {
        // SAFETY: `str_m` is not zero, so the first byte is the caller's.
        unsafe { *str = 0 };
        return 0;
    };
    str_l.min(str_m - 1)
}

/// `fmt` formatted with `args` into a fresh api [`String_0`], which owns the
/// bytes.
///
/// Upstream calls this `arena_printf`, and formats through libc's
/// `vsnprintf`; it is the editor's own formatter here, which prints the
/// conversions its callers use (`%d %ld %s %.*s`) byte for byte as libc does.
///
/// # Safety
///
/// As [`vim_snprintf`] for `fmt` and `args`.
pub unsafe fn printf_string(fmt: *const c_char, args: &[FmtArg<'_>]) -> String_0 {
    // SAFETY: the caller's promises; a measuring pass writes nothing.
    let len = unsafe { vim_snprintf(core::ptr::null_mut(), 0, fmt, args) };
    let Ok(len) = usize::try_from(len) else {
        return String_0::NULL;
    };
    let mut out = vec![0u8; len + 1];
    // SAFETY: as above, into the `len + 1` bytes just made.
    unsafe { vim_snprintf(out.as_mut_ptr().cast(), len + 1, fmt, args) };
    String_0::from_bytes(&out[..len])
}

/// `vim_snprintf(buf, len, fmt, args...)`, the arguments gathered into an
/// array on the stack through `FmtArg::from`.
#[macro_export]
macro_rules! vim_snprintf {
    ($buf:expr, $len:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::strings::vim_snprintf(
            $buf, $len, $fmt, &[$($crate::strings::FmtArg::from($arg)),*]
        )
    };
}

/// `vim_snprintf_add(buf, len, fmt, args...)`; see [`vim_snprintf!`].
#[macro_export]
macro_rules! vim_snprintf_add {
    ($buf:expr, $len:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::strings::vim_snprintf_add(
            $buf, $len, $fmt, &[$($crate::strings::FmtArg::from($arg)),*]
        )
    };
}

/// `vim_snprintf_safelen(buf, len, fmt, args...)`; see [`vim_snprintf!`].
#[macro_export]
macro_rules! vim_snprintf_safelen {
    ($buf:expr, $len:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::strings::vim_snprintf_safelen(
            $buf, $len, $fmt, &[$($crate::strings::FmtArg::from($arg)),*]
        )
    };
}

/// `printf_string(fmt, args...)`; see [`vim_snprintf!`].
#[macro_export]
macro_rules! printf_string {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::strings::printf_string($fmt, &[$($crate::strings::FmtArg::from($arg)),*])
    };
}
