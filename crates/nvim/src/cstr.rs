//! The crate's C-string vocabulary, over borrowed strings and slices.
//!
//! Every helper here is safe. A pointer becomes a [`CStr`] at the one place
//! it is read -- [`CStr::from_ptr`], in the perimeter module that holds the
//! foreign call -- and from there on the text is a borrow with a length.
//! What lives here is what std does not spell: the string a buffer *starts
//! with* ([`in_bytes`]), the tail of a string ([`suffix`]), the reader that
//! answers NUL past the end ([`byte_at`]), and `strncmp`'s comparison over
//! two slices ([`ncmp_eq`]).
//!
//! # The parameter convention
//!
//! A function that takes text takes one of three types, and which one is a
//! statement about the text rather than a matter of taste:
//!
//! - **`&CStr`** — a borrowed *NUL-terminated* string of arbitrary bytes.
//!   This is the default, because it is what the editor's text mostly is:
//!   option values, file names, function names, command lines. The length
//!   is implicit, so a caller that has one costs nothing. Write a literal
//!   as `c"…"` — Rust has C string literals, so no macro is needed and none
//!   is provided.
//! - **`&[u8]`** — bytes with an *explicit* length, and therefore the type
//!   for text that may hold an interior NUL or is not terminated at all: a
//!   buffer line (`memline` stores a NUL as an `NL`, so a line's bytes can
//!   hold either), a slice of a command line, the `ApiString` the API layer
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
//! | `strlen(p)` | `s.to_bytes().len()` |
//! | `strcmp(a, b) == 0` | slices compared |
//! | `strncmp(a, b, n) == 0` | [`ncmp_eq`] |
//! | `strchr(s, c)` | [`find_byte`] |
//! | `strstr(h, n)` | `h.windows(n.len()).position(…)` |
//! | `memcmp(a, b, n)` | `a[..n] == b[..n]` |
//!
//! Four differences bite, and each has cost this tree a bug:
//!
//! 1. **`strlen` is not `len()`.** `strlen` stops at the first NUL; a
//!    slice's `len()` counts every byte it was given, terminator included.
//!    A `&[u8]` built from a buffer is one byte longer than the `strlen` of
//!    the same buffer ([`until_nul`] is the `strlen` of a buffer).
//! 2. **`strncmp`'s length may exceed either operand.** It stops at the
//!    first NUL *or* at `n`, whichever comes first, so `strncmp(p, "ab", 5)`
//!    asks whether `p` is exactly `"ab"`. [`ncmp_eq`] reproduces that by
//!    reading NUL past each end; a plain `p[..5]` would panic.
//! 3. **`memcmp` does not care about lengths, and `==` does.** Two slices
//!    of different lengths are simply unequal in Rust; `memcmp` compares
//!    `n` bytes of both and would read past the shorter one.
//! 4. **`strchr(s, 0)` is an idiom**, and it answers a pointer to the
//!    terminator rather than null. A search of the bytes cannot find a NUL
//!    at all, because the terminator is not in the slice; such a site wants
//!    the length.
//! 5. **Measuring both operands is not comparing them.** Building two
//!    `CStr`s and comparing them reads *every* byte of both strings before
//!    it looks at the first pair, where the C call stops at the first
//!    difference. Where the caller is a filter rejecting candidates —
//!    completion's duplicate scan, a maphash bucket walk — that turns an
//!    O(match) test into O(length) and doubles the loop. Walk the two
//!    together where that matters (`Iterator::eq` over the bytes stops at
//!    the first difference).

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::CStr;
use std::ffi::CString;

/// The string `s` from byte `at` on.
///
/// A terminated string stays terminated however much of its front is
/// dropped, so this is the one shape of "a pointer into a C string" that
/// needs no promise from the caller. It is what a caller that used to write
/// `p + n` and pass that on wants. It costs a scan of the tail, which
/// `CStr` makes to confirm the terminator is the only NUL.
///
/// # Panics
/// When `at` is past the terminator.
pub(crate) fn suffix(s: &CStr, at: usize) -> &CStr {
    let bytes = s.to_bytes_with_nul();
    assert!(at < bytes.len(), "suffix past the end of the string");
    CStr::from_bytes_with_nul(&bytes[at..]).expect("the tail of a C string is one")
}

/// The string `buf` starts with.
///
/// A buffer holding no terminator answers `c""`: every caller is reading a
/// buffer some writer was supposed to terminate, and the empty string is
/// the answer that keeps a formatting bug from becoming a panic.
pub(crate) fn in_bytes(buf: &[u8]) -> &CStr {
    CStr::from_bytes_until_nul(buf).unwrap_or(c"")
}

/// The first byte of a C string, or NUL when it has none.
///
/// The spelling of `*p` for a value that is a string in its own right: the
/// option variables, whose first byte is what a dozen `'selection'`-shaped
/// tests ask about, answer `&CStr` and have no pointer to dereference.
pub(crate) fn first(s: &CStr) -> u8 {
    s.to_bytes().first().copied().unwrap_or(0)
}

/// The byte at `i`, answering `NUL` past the end.
///
/// The one adaptation almost every pointer walk needs when it becomes an
/// index walk: the C read a NUL-terminated string, so `p[i]` at or past the
/// length was the terminator and the loop stopped there. A slice has no
/// terminator, so `s[i]` would panic and `s.get(i)` would need a match at
/// every step. This is the reader that behaves like the pointer did —
/// the bytes, then `NUL` forever.
///
/// It is *not* for deciding a walk's bounds. A loop that can say
/// `i < s.len()` should say it; this is for the sites that genuinely look
/// one past the end, and for translating a `while (*p != NUL)` whose bound
/// is the terminator itself.
pub(crate) fn byte_at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

/// Whether `a` and `b` agree for `n` bytes -- `strncmp(a, b, n) == 0` over
/// two NUL-terminated strings held as slices.
///
/// Not `starts_with`: a comparison that reaches the NUL in *both* stops there
/// and succeeds, which is how an empty line matches an as-yet-unset comment
/// leader.  That case is reachable -- a blank line inside a comment, before
/// any `s:`/`m:` item has been seen -- and `starts_with` answers differently.
pub(crate) fn ncmp_eq(a: &[u8], b: &[u8], n: usize) -> bool {
    for i in 0..n {
        let (x, y) = (byte_at(a, i), byte_at(b, i));
        if x != y {
            return false;
        }
        if x == 0 {
            return true;
        }
    }
    true
}

/// Whether `a` and `b` agree for `n` bytes ignoring ASCII case:
/// `strncasecmp(a, b, n) == 0` over two C strings held as slices, each read
/// as NUL past its end. A comparison that reaches a NUL in both stops there
/// and succeeds.
pub(crate) fn ncase_eq_ascii(a: &[u8], b: &[u8], n: usize) -> bool {
    for i in 0..n {
        let (x, y) = (
            byte_at(a, i).to_ascii_lowercase(),
            byte_at(b, i).to_ascii_lowercase(),
        );
        if x != y {
            return false;
        }
        if x == 0 {
            return true;
        }
    }
    true
}

/// `bytes` up to their first NUL, or all of them: what a C reader handed
/// them as a string sees.
pub(crate) fn until_nul(bytes: &[u8]) -> &[u8] {
    // The C library's vectorised `memchr`: `CStr::from_bytes_until_nul`'s
    // search costs several times as much per byte (a register put spent
    // four times `strlen`'s instructions in it).
    crate::memory::memchr_in(bytes, 0).map_or(bytes, |at| &bytes[..at])
}

/// Run `f` over `bytes` as a C string, up to the first NUL among them.
///
/// For a callee that still reads a NUL-terminated name, handed a slice of a
/// line that has no terminator where the name ends. A short name is copied
/// into a buffer on the stack, so the common case allocates nothing -- the
/// in-place terminator this replaces cost two stores.
pub(crate) fn with_terminated<R>(bytes: &[u8], f: impl FnOnce(&CStr) -> R) -> R {
    const ROOM: usize = 64;
    if bytes.len() < ROOM {
        let mut buf = [0u8; ROOM];
        buf[..bytes.len()].copy_from_slice(bytes);
        // One scan: an interior NUL ends the string early, as it would have
        // in the C.
        let terminated = &buf[..=bytes.len()];
        let string = CStr::from_bytes_with_nul(terminated)
            .or_else(|_| CStr::from_bytes_until_nul(terminated))
            .expect("the buffer ends in a NUL");
        f(string)
    } else {
        let bytes = &bytes[..bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())];
        f(&owned(bytes))
    }
}

/// `bytes` as an owned C string.
///
/// # Panics
/// If `bytes` holds an interior NUL. Callers pass text a parser has already
/// split on NUL; use [`in_bytes`] where the terminator is inside the bytes.
pub(crate) fn owned(bytes: &[u8]) -> CString {
    CString::new(bytes).expect("the bytes hold an interior NUL")
}

/// Eight copies of `byte`.
const fn splat(byte: u8) -> u64 {
    0x0101_0101_0101_0101 * byte as u64
}

/// The high bit of every byte of `word` that is zero.
///
/// A byte above a zero one can come out set as well (the subtraction
/// borrows through it), but never one below it: the lowest bit set is the
/// first zero byte, which is all [`find_either`] asks.
const fn zero_bytes(word: u64) -> u64 {
    // Integer arithmetic, spelled as a call: the ratchet's pointer-arithmetic
    // count reads `.wrapping_sub(` as a pointer step.
    u64::wrapping_sub(word, splat(1)) & !word & splat(0x80)
}

/// Where the first byte of `hay` that is `a` or `b` is.
///
/// Eight bytes at a time: a reader that runs this over every byte of a
/// file (a tags file, a sourced script) and a byte-at-a-time walk costs several times what C's
/// `memchr` did there.
pub(crate) fn find_either(hay: &[u8], a: u8, b: u8) -> Option<usize> {
    let (pa, pb) = (splat(a), splat(b));
    let (words, rest) = hay.as_chunks::<8>();
    let mut at = 0;
    for word in words {
        let word = u64::from_le_bytes(*word);
        let found = zero_bytes(word ^ pa) | zero_bytes(word ^ pb);
        if found != 0 {
            // Little-endian: the lowest set bit is the first byte.
            return Some(at + (found.trailing_zeros() / 8) as usize);
        }
        at += 8;
    }
    rest.iter()
        .position(|&byte| byte == a || byte == b)
        .map(|found| at + found)
}

/// Where the first `needle` in `hay` is; [`find_either`] for one byte.
pub(crate) fn find_byte(hay: &[u8], needle: u8) -> Option<usize> {
    find_either(hay, needle, needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The word-at-a-time search answers what a byte walk does, wherever
    /// the bytes fall against the word boundaries — including a byte above
    /// 0x80 and a match right after one.
    #[test]
    fn find_either_is_a_byte_walk() {
        let alphabet = [b'a', b'\n', 0, 0x80, 0xff, 0x01, b'\t'];
        let mut seed: u32 = 36;
        for len in 0..40 {
            for _ in 0..50 {
                let hay: Vec<u8> = (0..len)
                    .map(|_| {
                        seed = u32::wrapping_add(seed.wrapping_mul(1_103_515_245), 12_345);
                        alphabet[(seed >> 16) as usize % alphabet.len()]
                    })
                    .collect();
                for (a, b) in [(b'\n', 0), (b'\t', b'\t'), (0x80, 0xff)] {
                    let walk = hay.iter().position(|&byte| byte == a || byte == b);
                    assert_eq!(find_either(&hay, a, b), walk, "{hay:?} {a} {b}");
                }
            }
        }
    }
}
