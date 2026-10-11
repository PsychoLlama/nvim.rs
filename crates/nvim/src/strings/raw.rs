//! The pointer fronts: `vim_snprintf` and friends as the rest of the tree
//! still calls them, into a raw buffer, with a raw format and raw `%s`
//! pointers, and the `vim_snprintf!` macros that build their argument
//! arrays; and the four string helpers whose callers (in the buffer, regexp,
//! spell, edit and Lua modules) still hold raw strings -- `vim_strchr`,
//! `xstrnsave`, `striequal`, `sort_strings`, each a shim over a slice form or
//! libc. They go as their callers convert.
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
use core::{ptr, slice};

use super::printf::{Source, format, length_as_int};
use super::{FmtArg, find_char};
use crate::cstr;
use crate::memory::xmallocz;
use crate::os::cshim::strchr;
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
                        let len = unsafe { strnlen(at, cap) };
                        // SAFETY: the `len` bytes the scan just read.
                        unsafe { slice::from_raw_parts(at.cast::<u8>(), len) }
                    }
                })
            }
            _ => None,
        }
    }
}

/// `strnlen`: the bytes before the terminator, reading at most `maxlen` and
/// never one past a NUL -- so a string shorter than `maxlen` is read only up
/// to its end, which a slice of `maxlen` bytes would not promise.
///
/// # Safety
///
/// `s` must point at `maxlen` readable bytes, or at a NUL-terminated string.
unsafe fn strnlen(s: *const c_char, maxlen: usize) -> usize {
    let mut n = 0;
    // SAFETY: the caller's bytes, each read only while no NUL came before.
    while n < maxlen && unsafe { *s.add(n) } != 0 {
        n += 1;
    }
    n
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
    let len = unsafe { vim_snprintf(ptr::null_mut(), 0, fmt, args) };
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

/// Find character `c` (a codepoint, not a byte) in `string`; [`find_char`]
/// over a C string.
///
/// The ASCII half keeps libc's `strchr`, which [`find_char`] cannot match:
/// one early-exiting SIMD pass against a `strlen` over the *whole* string
/// plus a scalar scan. Measured at +0.4 % of `inbench` when this body was
/// routed through the slice form, with sixty-odd cursor callers still on
/// the command-line path.
///
/// # Safety
///
/// `string` must point at a NUL-terminated string.
pub unsafe fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char {
    if c <= 0 {
        return ptr::null_mut();
    }
    if c < 0x80 {
        // SAFETY: the caller's NUL-terminated string.
        return unsafe { strchr(string, c) };
    }
    // SAFETY: the caller's NUL-terminated string, and an offset `find_char`
    // answered from within it.
    match find_char(unsafe { cstr::bytes_at(string) }, c) {
        Some(at) => unsafe { string.add(at) }.cast_mut(),
        None => ptr::null_mut(),
    }
}

/// Copy at most `len` bytes of `string` into a fresh NUL-terminated block of
/// `len + 1` bytes, zero-filling the remainder (`strncpy` semantics).
///
/// # Safety
///
/// `string` must point at `len` readable bytes, or at a NUL-terminated
/// string.
pub unsafe fn xstrnsave(string: *const c_char, len: usize) -> *mut c_char {
    // SAFETY: the caller's bytes, read up to `len` or the NUL; `xmallocz`
    // answers `len + 1` bytes, the last a NUL, and the copy and the fill
    // write the `len` before it.
    unsafe {
        let n = strnlen(string, len);
        let ret = xmallocz(len).cast::<c_char>();
        ptr::copy_nonoverlapping(string, ret, n);
        ptr::write_bytes(ret.add(n), 0, len - n);
        ret
    }
}

/// Case-insensitive `strcmp` equality where NULL only equals NULL.
/// `strcasecmp` is locale-aware, so the libc call stays.
///
/// # Safety
///
/// `a` and `b` must each be null or point at a NUL-terminated string.
pub unsafe fn striequal(a: *const c_char, b: *const c_char) -> bool {
    (a.is_null() && b.is_null())
        || (!a.is_null() && !b.is_null() && unsafe { ::libc::strcasecmp(a, b) } == 0)
}

/// Sort `count` strings at `files` bytewise, as `strcmp` orders them.
///
/// # Safety
///
/// `files` must point at `count` writable slots, each holding a NUL-terminated
/// string.
pub unsafe fn sort_strings(files: *mut *mut c_char, count: c_int) {
    let Ok(count @ 1..) = usize::try_from(count) else {
        return;
    };
    // SAFETY: the caller's `count` slots.
    let files = unsafe { slice::from_raw_parts_mut(files, count) };
    // SAFETY: each slot holds a NUL-terminated string.
    files.sort_unstable_by(|&a, &b| unsafe { CStr::from_ptr(a).cmp(CStr::from_ptr(b)) });
}

#[cfg(test)]
mod tests {
    use super::{sort_strings, vim_snprintf_add, vim_snprintf_safelen, vim_strchr, xstrnsave};
    use crate::memory::xfree;
    use crate::strings::{FmtArg, format_into};
    use core::ffi::{CStr, c_char, c_int, c_long};

    /// `vim_snprintf!` into an allocation of exactly `size` bytes, so a
    /// write past it is a Miri error; answers the return and the bytes up
    /// to the NUL.
    fn into_exact(size: usize, f: impl Fn(*mut c_char, usize) -> c_int) -> (c_int, Vec<u8>) {
        let mut buf = vec![0xa5u8; size].into_boxed_slice();
        let ret = f(buf.as_mut_ptr().cast(), size);
        let text = buf.iter().take_while(|&&b| b != 0).copied().collect();
        (ret, text)
    }

    #[test]
    fn the_pointer_front_formats_and_truncates_at_every_size() {
        let name = c"na\u{ef}ve";
        let want = "[na\u{ef}ve|   42|ff  |2.72|x|%|0x10|-1.0e21]";
        for size in 0..=want.len() + 1 {
            let (ret, text) = into_exact(size, |b, n| unsafe {
                crate::vim_snprintf!(
                    b,
                    n,
                    c"[%s|%5d|%-4x|%.2f|%c|%%|%p|%g]".as_ptr(),
                    name.as_ptr(),
                    42 as c_int,
                    255 as c_int,
                    2.71875,
                    c_int::from(b'x'),
                    core::ptr::without_provenance::<u8>(16),
                    -1e21,
                )
            });
            assert_eq!(usize::try_from(ret), Ok(want.len()));
            let keep = want.len().min(size.saturating_sub(1));
            assert_eq!(text, &want.as_bytes()[..keep], "size {size}");
        }
    }

    #[test]
    fn a_precision_bounds_how_far_a_pointer_is_read() {
        // Three bytes and no terminator: `%.3s` and `%.2s` may read them,
        // and must not read a fourth.
        let bytes = Box::new(*b"abc");
        let at = bytes.as_ptr().cast::<c_char>();
        for (fmt, want) in [(c"<%.3s>", "<abc>"), (c"<%.2s>", "<ab>"), (c"<%.0s>", "<>")] {
            let (_, text) = into_exact(16, |b, n| unsafe {
                crate::vim_snprintf!(b, n, fmt.as_ptr(), at)
            });
            assert_eq!(text, want.as_bytes());
        }
        // A precision past a shorter terminated string stops at its NUL.
        let short = c"hi";
        let (_, text) = into_exact(16, |b, n| unsafe {
            crate::vim_snprintf!(
                b,
                n,
                c"<%.40s|%.*S>".as_ptr(),
                short.as_ptr(),
                9 as c_int,
                short.as_ptr()
            )
        });
        assert_eq!(text, b"<hi|hi>");
    }

    #[test]
    fn add_appends_and_safelen_answers_what_fits() {
        let mut buf = *b"ab\0\0\0\0\0\0";
        let ret = unsafe {
            vim_snprintf_add(
                buf.as_mut_ptr().cast(),
                buf.len(),
                c"%s%ld".as_ptr(),
                &[FmtArg::from(c"cd".as_ptr()), FmtArg::from(7 as c_long)],
            )
        };
        assert_eq!(ret, 3);
        assert_eq!(&buf, b"abcd7\0\0\0");
        let mut small = [0xa5u8; 4];
        let fitted = unsafe {
            vim_snprintf_safelen(
                small.as_mut_ptr().cast(),
                small.len(),
                c"%s".as_ptr(),
                &[FmtArg::from(c"wxyz".as_ptr())],
            )
        };
        assert_eq!((fitted, &small), (3, b"wxy\0"));
    }

    #[test]
    fn the_safe_form_never_reads_a_raw_pointer() {
        let mut out = [0u8; 32];
        let n = format_into(
            &mut out,
            b"%s %s %d",
            &[
                FmtArg::from(c"x".as_ptr()),
                FmtArg::Str(b"bytes"),
                FmtArg::from(-3 as c_int),
            ],
        );
        assert_eq!(&out[..n], b"[NULL] bytes -3");
        assert_eq!(out[n], 0);
    }

    #[test]
    fn the_string_shims_read_no_further_than_they_may() {
        unsafe {
            let exact = Box::new(*b"abcd");
            let copy = xstrnsave(exact.as_ptr().cast(), 3);
            assert_eq!(CStr::from_ptr(copy).to_bytes(), b"abc");
            xfree(copy.cast());
            let copy = xstrnsave(c"ab".as_ptr(), 2);
            assert_eq!(CStr::from_ptr(copy).to_bytes(), b"ab");
            xfree(copy.cast());

            let text = c"a\u{ab}b";
            let at = vim_strchr(text.as_ptr(), 0xab);
            assert_eq!(at.addr() - text.as_ptr().addr(), 1);
            assert!(vim_strchr(text.as_ptr(), i32::from(b'z')).is_null());
            assert_eq!(
                vim_strchr(text.as_ptr(), i32::from(b'b')).addr() - text.as_ptr().addr(),
                3
            );

            let (b, a, c) = (c"b", c"a", c"c");
            let mut names = [
                b.as_ptr().cast_mut(),
                a.as_ptr().cast_mut(),
                c.as_ptr().cast_mut(),
            ];
            sort_strings(names.as_mut_ptr(), 3);
            let sorted: Vec<&[u8]> = names
                .iter()
                .map(|&p| CStr::from_ptr(p).to_bytes())
                .collect();
            assert_eq!(sorted, [b"a", b"b", b"c"]);
        }
    }
}
