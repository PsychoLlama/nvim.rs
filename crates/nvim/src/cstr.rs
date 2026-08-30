//! The crate's C-string vocabulary.
//!
//! Fifteen modules each grew their own `cstr_at`/`cstr_opt`/`cstr_bytes`
//! one-liner around [`CStr`]; this is the one home. Three questions are
//! being asked, and the names say which:
//!
//! - **`at*`** — borrow what a pointer points at. `unsafe`, because only
//!   the caller knows the bytes are live and (for the terminated forms)
//!   terminated. Plain pointer-to-[`CStr`] is [`CStr::from_ptr`] and stays
//!   there; what lives here is the null case, the bytes case and the two
//!   bounded cases, which std does not spell.
//! - **`in_*`** — borrow the string a buffer *starts with*. Safe: the
//!   buffer bounds the search, and a buffer with no terminator answers the
//!   empty string rather than reading past the end.
//! - **`owned`** — copy bytes into a [`CString`].
//!
//! The borrowing forms hand out an unbounded lifetime, which is what every
//! caller needed: the string outlives the pointer variable it was read
//! from. That is the caller's obligation, stated once here.
//!
//! # The parameter convention
//!
//! A function that takes text takes one of three types, and which one is a
//! statement about the text rather than a matter of taste:
//!
//! - **`&CStr`** — a borrowed *NUL-terminated* string of arbitrary bytes.
//!   This is the default, because it is what the editor's text mostly is:
//!   option values, file names, function names, command lines. The length
//!   is implicit, so a caller that has one costs nothing and a caller that
//!   only has a pointer converts once, at the boundary, with [`at`]. Write
//!   a literal as `c"…"` — Rust has C string literals, so no macro is
//!   needed and none is provided.
//! - **`&[u8]`** — bytes with an *explicit* length, and therefore the type
//!   for text that may hold an interior NUL or is not terminated at all: a
//!   buffer line (`memline` stores a NUL as an `NL`, so a line's bytes can
//!   hold either), a slice of a command line, the `String_0` the API layer
//!   passes, anything measured by a separate `len` parameter. Prefer this
//!   whenever the caller already knows the length: `&CStr` would make it
//!   re-scan for a terminator that may not be there.
//! - **`&str`** — only where UTF-8 is *guaranteed by construction*: a
//!   literal, a name the parser built out of ASCII, a number formatted
//!   here. Vim's text is not UTF-8 — `'encoding'` aside, a file can hold
//!   any bytes — so `&str` on a path that carries user text is a bug
//!   waiting for a `from_utf8` to fail or a `to_string_lossy` to corrupt
//!   the bytes. Never reach for `to_string_lossy` to satisfy a signature;
//!   change the signature.
//!
//! The same three answer the return side. A function that answers borrowed
//! text answers `&CStr`/`&[u8]`; one that answers text it built answers
//! [`CString`]/`Vec<u8>`.
//!
//! # Replacing the libc string calls
//!
//! Every `str*`/`mem*` call has a slice spelling, and the slice spelling is
//! the one that cannot walk off the end:
//!
//! | C | slice |
//! | --- | --- |
//! | `strlen(p)` | [`bytes_at(p)`](bytes_at)`.len()` |
//! | `strcmp(a, b)` | `bytes_at(a) == bytes_at(b)`, or `.cmp(…)` |
//! | `strncmp(a, b, n)` | [`prefix_at(a, n)`](prefix_at)` == prefix_at(b, n)` |
//! | `strchr(s, c)` | `bytes_at(s).iter().position(…)` |
//! | `strstr(h, n)` | `bytes_at(h).windows(n.len()).position(…)` |
//!
//! Three differences bite, and each has cost this tree a bug:
//!
//! 1. **`strlen` is not `len()`.** `strlen` stops at the first NUL; a
//!    slice's `len()` counts every byte it was given, terminator included.
//!    A `&[u8]` built from a buffer is one byte longer than the `strlen` of
//!    the same buffer.
//! 2. **`strncmp`'s length may exceed either operand.** It stops at the
//!    first NUL *or* at `n`, whichever comes first, so `strncmp(p, "ab", 5)`
//!    asks whether `p` is exactly `"ab"`. [`prefix_at`] reproduces that by
//!    stopping at the NUL as well; a plain `p[..5]` would panic or compare
//!    bytes past the end.
//! 3. **`strchr(s, 0)` is an idiom**, and it answers a pointer to the
//!    terminator rather than null. `bytes_at(s).iter().position(…)` cannot
//!    find a NUL at all, because the terminator is not in the slice; such a
//!    site wants `bytes_at(s).len()`.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::cmp::Ordering;
use core::ffi::{CStr, c_char};
use core::slice;
use std::ffi::CString;

/// [`CStr::from_ptr`], answering `None` for a null pointer.
///
/// # Safety
/// A non-null `p` points at a NUL-terminated string that stays live and
/// unwritten for `'a`.
pub(crate) unsafe fn at_opt<'a>(p: *const c_char) -> Option<&'a CStr> {
    // SAFETY: caller's contract, minus the null case.
    (!p.is_null()).then(|| unsafe { CStr::from_ptr(p) })
}

/// The bytes of the string at `p`, without its terminator.
///
/// This is `strlen`'s scan: it walks to the NUL.
///
/// # Safety
/// [`at_opt`]'s contract, minus the null case.
pub(crate) unsafe fn bytes_at<'a>(p: *const c_char) -> &'a [u8] {
    // SAFETY: caller's contract.
    unsafe { CStr::from_ptr(p) }.to_bytes()
}

/// The first `n` bytes of the string at `p`, stopping at its terminator --
/// which is exactly the span `strncmp(p, _, n)` compares.
///
/// Answers fewer than `n` bytes for a shorter string, so comparing two of
/// them reproduces `strncmp`'s answer including the case where one operand
/// runs out first.
///
/// # Safety
/// `p` points at a NUL-terminated string, live and unwritten for `'a`. It
/// need not have `n` bytes: the scan stops at the terminator.
pub(crate) unsafe fn prefix_at<'a>(p: *const c_char, n: usize) -> &'a [u8] {
    let mut len = 0;
    // SAFETY: caller's contract -- every byte up to and including the
    // terminator is readable, and the loop stops at it.
    while len < n && unsafe { *p.add(len) } != 0 {
        len += 1;
    }
    // SAFETY: `len` bytes were just read one at a time.
    unsafe { slice::from_raw_parts(p.cast::<u8>(), len) }
}

/// Whether two NUL-terminated strings agree over their first `n` bytes --
/// `strncmp(a, b, n) == 0`.
///
/// One call rather than two [`prefix_at`]s so that the comparison still fits
/// on one line: a wrapped expression inside an `unsafe` block costs the
/// ratchet three to seven unchecked lines for nothing.
///
/// # Safety
/// [`prefix_at`]'s contract, for both.
pub(crate) unsafe fn prefix_eq(a: *const c_char, b: *const c_char, n: usize) -> bool {
    // SAFETY: caller's contract.
    unsafe { prefix_at(a, n) == prefix_at(b, n) }
}

/// How two NUL-terminated strings order over their first `n` bytes --
/// `strncmp(a, b, n)`, as an [`Ordering`](Ordering) rather than a
/// sign. `as c_int` recovers the -1/0/1 a `qsort` comparator wants.
///
/// # Safety
/// [`prefix_at`]'s contract, for both.
pub(crate) unsafe fn prefix_cmp(a: *const c_char, b: *const c_char, n: usize) -> Ordering {
    // SAFETY: caller's contract.
    unsafe { prefix_at(a, n).cmp(prefix_at(b, n)) }
}

/// Whether the string at `p` starts with `prefix` -- `strncmp(p, prefix,
/// prefix.len()) == 0`, without the length that has to be kept in step with
/// the literal.
///
/// # Safety
/// [`prefix_at`]'s contract.
pub(crate) unsafe fn starts_with(p: *const c_char, prefix: &[u8]) -> bool {
    // SAFETY: caller's contract.
    unsafe { prefix_at(p, prefix.len()) == prefix }
}

/// The string `buf` starts with.
///
/// A buffer holding no terminator answers `c""`: every caller is reading a
/// buffer some writer was supposed to terminate, and the empty string is
/// the answer that keeps a formatting bug from becoming a panic.
pub(crate) fn in_bytes(buf: &[u8]) -> &CStr {
    CStr::from_bytes_until_nul(buf).unwrap_or(c"")
}

/// [`in_bytes`] for a buffer a C callee wrote, typed `c_char`.
pub(crate) fn in_chars(buf: &[c_char]) -> &CStr {
    in_bytes(as_bytes(buf))
}

/// A `c_char` buffer's bytes.
///
/// `c_char` is `i8` on this tree's targets and `u8` on others; the two are
/// the same bytes either way, and the editor's text is bytes. Reading a
/// buffer *as* bytes is the first move of almost every conversion away from
/// a `str*` call, so it is one function rather than a `cast` at each site.
pub(crate) fn as_bytes(buf: &[c_char]) -> &[u8] {
    // SAFETY: `c_char` and `u8` have the same size and alignment, and every
    // bit pattern is valid for both.
    unsafe { slice::from_raw_parts(buf.as_ptr().cast::<u8>(), buf.len()) }
}

/// `bytes` as an owned C string.
///
/// # Panics
/// If `bytes` holds an interior NUL. Callers pass text a parser has already
/// split on NUL; use [`in_bytes`] where the terminator is inside the bytes.
pub(crate) fn owned(bytes: &[u8]) -> CString {
    CString::new(bytes).expect("the bytes hold an interior NUL")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_stops_at_the_terminator_and_at_n() {
        let s = c"abcdef";
        // SAFETY: a literal, live for the test.
        let (short, exact, long) = unsafe {
            (
                prefix_at(s.as_ptr(), 3),
                prefix_at(s.as_ptr(), 6),
                prefix_at(s.as_ptr(), 20),
            )
        };
        assert_eq!(short, b"abc");
        assert_eq!(exact, b"abcdef");
        assert_eq!(long, b"abcdef", "the terminator bounds it, not `n`");
    }

    /// `strncmp(a, b, n)`'s three answers, as slice comparisons.
    #[test]
    fn prefix_reproduces_strncmp() {
        let cases: &[(&CStr, &CStr, usize, Ordering)] = &[
            (c"ab", c"abc", 5, Ordering::Less),
            (c"ab", c"ab", 5, Ordering::Equal),
            (c"abc", c"abd", 2, Ordering::Equal),
            (c"abc", c"abd", 3, Ordering::Less),
            (c"", c"a", 1, Ordering::Less),
            (c"a", c"", 4, Ordering::Greater),
        ];
        for &(a, b, n, want) in cases {
            // SAFETY: both are literals, live for the test.
            let got = unsafe { prefix_at(a.as_ptr(), n).cmp(prefix_at(b.as_ptr(), n)) };
            assert_eq!(got, want, "{a:?} vs {b:?} over {n}");
        }
    }
}
