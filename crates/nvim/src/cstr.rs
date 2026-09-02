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
//! | `strcmp(a, b)` | [`eq(a, b)`](eq), or [`cmp`] |
//! | `strncmp(a, b, n)` | [`prefix_eq(a, b, n)`](prefix_eq), or [`prefix_cmp`] |
//! | `strchr(s, c)` | `bytes_at(s).iter().position(…)` |
//! | `strstr(h, n)` | `bytes_at(h).windows(n.len()).position(…)` |
//! | `memcmp(a, b, n)` | [`slice_at(a, n)`](slice_at)` == slice_at(b, n)` |
//! | `memcpy(d, s, n)` | `d.cast::<u8>().copy_from_nonoverlapping(…)` |
//! | `memmove(d, s, n)` | `d.cast::<u8>().copy_from(s.cast::<u8>(), n)` |
//! | `memset(d, b, n)` | `d.cast::<u8>().write_bytes(b, n)` |
//!
//! Four differences bite, and each has cost this tree a bug:
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
//! 3. **`memcmp` does not care about lengths, and `==` does.** Two slices
//!    of different lengths are simply unequal in Rust; `memcmp` compares
//!    `n` bytes of both and would read past the shorter one. Where the two
//!    lengths differ on purpose, compare explicit [`slice_at`]s.
//! 4. **`strchr(s, 0)` is an idiom**, and it answers a pointer to the
//!    terminator rather than null. `bytes_at(s).iter().position(…)` cannot
//!    find a NUL at all, because the terminator is not in the slice; such a
//!    site wants `bytes_at(s).len()`.
//! 5. **Measuring both operands is not comparing them.** `bytes_at(a) ==
//!    bytes_at(b)` and `prefix_at(a, n) == prefix_at(b, n)` read *every*
//!    byte of both strings before they look at the first pair, where the C
//!    call stops at the first difference. Where the caller is a filter
//!    rejecting candidates — completion's duplicate scan, a maphash bucket
//!    walk — that turns an O(match) test into O(length) and doubles the
//!    loop. Use [`eq`]/[`cmp`]/[`prefix_eq`]/[`prefix_cmp`]/[`starts_with`],
//!    which walk the two together. [`prefix_at`] is for the sites that want
//!    the *span* itself, not a comparison.

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

/// [`CStr::from_ptr`], under this module's lifetime convention.
///
/// # Safety
/// `p` points at a NUL-terminated string that stays live and unwritten for
/// `'a`.
pub(crate) unsafe fn at<'a>(p: *const c_char) -> &'a CStr {
    // SAFETY: caller's contract.
    unsafe { CStr::from_ptr(p) }
}

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

/// Whether two NUL-terminated strings are equal -- `strcmp(a, b) == 0`.
///
/// Where one side is a literal, [`eq_bytes`] says which operand is the
/// constant. **Not `bytes_at(p) == b"..."`**: that measures `p` to its
/// terminator before it looks at a single byte.
///
/// # Safety
/// [`bytes_at`]'s contract, for both.
pub(crate) unsafe fn eq(a: *const c_char, b: *const c_char) -> bool {
    // SAFETY: caller's contract.
    unsafe { cmp(a, b) }.is_eq()
}

/// Whether the string at `p` is exactly `want` -- `strcmp` against a
/// literal, without spelling the literal as a pointer.
///
/// **Walks `p` against `want` and stops at the first difference**, as
/// [`cmp`] does and for the reason written on [`prefix_cmp`]: the obvious
/// `bytes_at(p) == want` reads every byte of `p` first, so a test that a
/// one-byte mismatch settles costs the length of whatever `p` happens to
/// hold.
///
/// # Safety
/// [`bytes_at`]'s contract. `want` must hold no NUL, which is what makes
/// the walk stop at or before `p`'s terminator.
pub(crate) unsafe fn eq_bytes(p: *const c_char, want: &[u8]) -> bool {
    debug_assert!(!want.contains(&0), "a NUL in the literal outruns the walk");
    for (i, &byte) in want.iter().enumerate() {
        // SAFETY: caller's contract -- `p`'s terminator is readable, and a
        // NUL there cannot equal a byte of `want`, so the walk stops on it.
        if unsafe { *p.cast::<u8>().add(i) } != byte {
            return false;
        }
    }
    // SAFETY: `want.len()` bytes of `p` just matched, so `p` is at least
    // that long and the byte after them is readable.
    unsafe { *p.cast::<u8>().add(want.len()) == 0 }
}

/// How two NUL-terminated strings order -- `strcmp(a, b)`, as an
/// [`Ordering`] rather than a sign.
///
/// Walks the two together and stops at the first difference, as
/// [`prefix_cmp`] does and for the same reason: `bytes_at(a).cmp(bytes_at(b))`
/// measures both operands in full before it looks at a single byte.
///
/// # Safety
/// [`bytes_at`]'s contract, for both.
pub(crate) unsafe fn cmp(a: *const c_char, b: *const c_char) -> Ordering {
    let mut i = 0;
    loop {
        // SAFETY: caller's contract -- both are NUL-terminated, and the
        // terminator ends the walk.
        let (x, y) = unsafe { (*a.cast::<u8>().add(i), *b.cast::<u8>().add(i)) };
        let ord = x.cmp(&y);
        if ord != Ordering::Equal || x == 0 {
            return ord;
        }
        i += 1;
    }
}

/// Whether two NUL-terminated strings agree over their first `n` bytes --
/// `strncmp(a, b, n) == 0`.
///
/// # Safety
/// [`prefix_at`]'s contract, for both.
pub(crate) unsafe fn prefix_eq(a: *const c_char, b: *const c_char, n: usize) -> bool {
    // SAFETY: caller's contract.
    unsafe { prefix_cmp(a, b, n) }.is_eq()
}

/// How two NUL-terminated strings order over their first `n` bytes --
/// `strncmp(a, b, n)`, as an [`Ordering`](Ordering) rather than a
/// sign. `as c_int` recovers the -1/0/1 a `qsort` comparator wants.
///
/// **Walks the two together and stops at the first difference**, the way
/// `strncmp` does, rather than measuring each operand and comparing the two
/// spans. Measuring first is what the obvious `prefix_at(a, n) ==
/// prefix_at(b, n)` does, and it reads every byte of both operands even when
/// the first pair already differs -- which is the whole cost when the caller
/// is a filter rejecting thousands of candidates. `ins_compl_add`'s
/// duplicate scan and `mapping/table.rs`'s bucket walk are both that shape,
/// and both roughly doubled on the measured-first form (p24-9).
///
/// # Safety
/// [`prefix_at`]'s contract, for both.
pub(crate) unsafe fn prefix_cmp(a: *const c_char, b: *const c_char, n: usize) -> Ordering {
    for i in 0..n {
        // SAFETY: caller's contract -- every byte up to and including each
        // terminator is readable, and the loop stops at the first one.
        let (x, y) = unsafe { (*a.cast::<u8>().add(i), *b.cast::<u8>().add(i)) };
        let ord = x.cmp(&y);
        // A terminator on both sides ends the comparison: C says the bytes
        // after a NUL are not compared.
        if ord != Ordering::Equal || x == 0 {
            return ord;
        }
    }
    Ordering::Equal
}

/// Whether the string at `p` starts with `prefix` -- `strncmp(p, prefix,
/// prefix.len()) == 0`, without the length that has to be kept in step with
/// the literal.
///
/// Short-circuits like [`prefix_cmp`], and for the same reason.
///
/// # Safety
/// [`prefix_at`]'s contract.
pub(crate) unsafe fn starts_with(p: *const c_char, prefix: &[u8]) -> bool {
    for (i, &want) in prefix.iter().enumerate() {
        // SAFETY: caller's contract -- `p`'s terminator is readable and ends
        // the walk, since `prefix` reaching a NUL means `want` matched it.
        let got = unsafe { *p.cast::<u8>().add(i) };
        if got != want {
            return false;
        }
        if got == 0 {
            break;
        }
    }
    true
}

/// Exactly `n` bytes at `p`, terminator or not.
///
/// The `mem*` calls' operand: a length the caller already knows, with no
/// scan and no NUL rule. A null `p` with `n == 0` answers the empty slice,
/// which [`slice::from_raw_parts`] itself refuses.
///
/// # Safety
/// `p` has `n` readable bytes, live and unwritten for `'a`.
pub(crate) unsafe fn slice_at<'a>(p: *const c_char, n: usize) -> &'a [u8] {
    if n == 0 {
        return &[];
    }
    // SAFETY: caller's contract.
    unsafe { slice::from_raw_parts(p.cast::<u8>(), n) }
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

    #[test]
    fn eq_bytes_answers_strcmp_against_a_literal() {
        // SAFETY: both are literals, live for the test.
        let is = |s: &CStr, want: &[u8]| unsafe { eq_bytes(s.as_ptr(), want) };
        assert!(is(c"abc", b"abc"));
        assert!(!is(c"abc", b"abd"), "a differing byte");
        assert!(!is(c"abc", b"ab"), "the string is longer");
        assert!(!is(c"abc", b"abcd"), "the literal is longer");
        assert!(is(c"", b""), "both empty");
        assert!(!is(c"abc", b""), "only the literal is empty");
    }

    /// A mismatch at byte 0 must not read byte 1 -- the whole point of the
    /// helper. The string here is not terminated, so measuring it first is
    /// undefined and only the short-circuit keeps the read in bounds.
    #[test]
    fn eq_bytes_stops_at_the_first_difference() {
        let unterminated = *b"xyz";
        // SAFETY: `b"a"` differs from `x` at byte 0, so the walk reads
        // exactly one byte of an array that has three.
        assert!(!unsafe { eq_bytes(unterminated.as_ptr().cast(), b"a") });
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
