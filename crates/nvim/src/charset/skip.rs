//! The `skip*` family over `&[u8]`, answering an offset instead of a pointer.
//!
//! Upstream's skippers take a NUL-terminated string and hand back a pointer
//! into it; the caller then either walks on from there or subtracts to get a
//! length ([`getwhitecols`](super::getwhitecols) is that subtraction, written
//! out). A slice already carries its own end, so the honest answer is the
//! offset — the count of leading bytes the skip stepped over, which is
//! equally the index the caller wants to resume at and the length of what was
//! skipped. `s[skip::white(s)..]` is the pointer form; `skip::white(s)` on its
//! own is `getwhitecols`.
//!
//! The offset is always in `0..=s.len()`, so indexing with it cannot panic,
//! and every function here answers `s.len()` for a slice made entirely of the
//! bytes it skips — the position of the NUL, in the pointer forms' terms.
//! A NUL inside the slice is an ordinary byte and stops none of them; the
//! pointer forms stop there only because that is where their string ends.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{is_bdigit, is_digit, is_white, is_xdigit};
use crate::keycodes::Ctrl_V;

/// How many leading bytes of `s` are spaces or tabs.
///
/// The slice form of [`skipwhite`](super::skipwhite), and of
/// [`getwhitecols`](super::getwhitecols), which are the same function.
pub fn white(s: &[u8]) -> usize {
    count(s, is_white)
}

/// How many bytes of `s` come before the first space or tab.
///
/// The slice form of [`skiptowhite`](super::skiptowhite).
pub fn to_white(s: &[u8]) -> usize {
    count(s, |byte| !is_white(byte))
}

/// [`to_white`], but a backslash or CTRL-V hides the byte after it.
///
/// The slice form of [`skiptowhite_esc`](super::skiptowhite_esc). The escape
/// only hides a byte that is actually there: a trailing backslash is the last
/// byte skipped, never a step past the end.
pub fn to_white_esc(s: &[u8]) -> usize {
    let mut at = 0;
    while let Some(&byte) = s.get(at) {
        if is_white(byte) {
            break;
        }
        let escapes = (byte == b'\\' || i32::from(byte) == Ctrl_V) && at + 1 < s.len();
        at += 1 + usize::from(escapes);
    }
    at
}

/// How many leading bytes of `s` are decimal digits.
///
/// The slice form of [`skipdigits`](super::skipdigits).
pub fn digits(s: &[u8]) -> usize {
    count(s, is_digit)
}

/// How many leading bytes of `s` are hexadecimal digits.
///
/// The slice form of [`skiphex`](super::skiphex).
pub fn hex(s: &[u8]) -> usize {
    count(s, is_xdigit)
}

/// How many leading bytes of `s` are binary digits.
///
/// The slice form of [`skipbin`](super::skipbin).
pub fn bin(s: &[u8]) -> usize {
    count(s, is_bdigit)
}

/// How many bytes of `s` come before the first decimal digit.
///
/// The slice form of [`skiptodigit`](super::skiptodigit).
pub fn to_digit(s: &[u8]) -> usize {
    count(s, |byte| !is_digit(byte))
}

/// How many bytes of `s` come before the first hexadecimal digit.
///
/// The slice form of [`skiptohex`](super::skiptohex). `pub(crate)`, like its
/// binary twin: no ledger names either, and the crate's boundary is already
/// wider than it should be.
pub(crate) fn to_hex(s: &[u8]) -> usize {
    count(s, |byte| !is_xdigit(byte))
}

/// How many bytes of `s` come before the first binary digit.
///
/// The slice form of [`skiptobin`](super::skiptobin).
pub(crate) fn to_bin(s: &[u8]) -> usize {
    count(s, |byte| !is_bdigit(byte))
}

/// The leading run of `s` that `keep` accepts.
fn count(s: &[u8], keep: impl Fn(u8) -> bool) -> usize {
    s.iter().position(|&byte| !keep(byte)).unwrap_or(s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_offset_is_always_a_valid_index() {
        for s in [
            b"".as_slice(),
            b" \t \tx",
            b"   ",
            b"0x1F ",
            b"abc",
            b"\0 ",
            b"101",
        ] {
            for offset in [
                white(s),
                to_white(s),
                to_white_esc(s),
                digits(s),
                hex(s),
                bin(s),
                to_digit(s),
                to_hex(s),
                to_bin(s),
            ] {
                assert!(offset <= s.len(), "{s:?} answered {offset}");
                let _ = &s[offset..];
            }
        }
    }

    #[test]
    fn a_slice_of_nothing_but_the_skipped_class_is_skipped_whole() {
        assert_eq!(white(b" \t \t"), 4);
        assert_eq!(digits(b"90210"), 5);
        assert_eq!(hex(b"dEadBeef01"), 10);
        assert_eq!(bin(b"1001"), 4);
        assert_eq!(to_white(b"nowhitespace"), 12);
    }

    #[test]
    fn the_first_byte_of_the_other_class_stops_the_skip() {
        assert_eq!(white(b"  x  "), 2);
        assert_eq!(to_white(b"ab cd"), 2);
        assert_eq!(digits(b"12ab"), 2);
        assert_eq!(hex(b"12abg"), 4);
        assert_eq!(bin(b"1012"), 3);
        assert_eq!(to_digit(b"abc1"), 3);
        assert_eq!(to_hex(b"zzz1F"), 3);
        assert_eq!(to_bin(b"xy01"), 2);
    }

    /// A NUL is a byte like any other here: the pointer forms stop at one
    /// only because it is the end of their string.
    #[test]
    fn an_embedded_nul_stops_nothing() {
        assert_eq!(to_white(b"a\0b c"), 3);
        assert_eq!(white(b"  \0 "), 2);
    }

    #[test]
    fn an_escape_hides_the_byte_after_it_but_never_one_past_the_end() {
        assert_eq!(to_white_esc(b"a\\ b c"), 4);
        assert_eq!(to_white_esc(b"a\\"), 2);
        assert_eq!(to_white_esc(b"\\ "), 2);
        assert_eq!(to_white_esc(b" a"), 0);
    }
}
