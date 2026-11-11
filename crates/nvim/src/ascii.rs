#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! Character classification for the ASCII range.
//!
//! These are the predicates the editor uses instead of `<ctype.h>`: they are
//! locale-independent and defined for every `int`, including the negative
//! values a `char` widens to on a signed-char platform and the `K_SPECIAL`
//! terminal codes above 0x80. The C had them as `static inline`s in a header,
//! so the transpiler left a copy in every module that called one.
//!
//! Each takes anything that widens to an `int` without loss, so a byte of
//! text and a code point ask the same function: `ascii_isdigit(byte)` and
//! `ascii_isdigit(c)` are one body, where the C needed a cast or a copy.

use core::ffi::c_int;

/// The byte `c` is, when it is one: an ASCII class is a question about a
/// byte, and nothing outside `0..=0xff` answers yes to any of them.
#[inline(always)]
fn as_byte(c: c_int) -> Option<u8> {
    u8::try_from(c).ok()
}

/// A decimal digit.
pub(crate) fn ascii_isdigit(c: impl Into<c_int>) -> bool {
    as_byte(c.into()).is_some_and(|b| b.is_ascii_digit())
}

/// `ASCII_ISALPHA`: an unaccented Latin letter.
pub(crate) fn ascii_isalpha(c: impl Into<c_int>) -> bool {
    // A negative byte fails, rather than wrapping into a letter -- which is
    // what the C macro's unsigned cast does.
    as_byte(c.into()).is_some_and(|b| b.is_ascii_alphabetic())
}

/// `ASCII_ISUPPER`: an unaccented upper-case Latin letter.
pub(crate) fn ascii_isupper(c: impl Into<c_int>) -> bool {
    as_byte(c.into()).is_some_and(|b| b.is_ascii_uppercase())
}

/// A binary digit.
pub(crate) fn ascii_isbdigit(c: impl Into<c_int>) -> bool {
    as_byte(c.into()).is_some_and(|b| b == b'0' || b == b'1')
}

/// An octal digit.
pub(crate) fn ascii_isodigit(c: impl Into<c_int>) -> bool {
    as_byte(c.into()).is_some_and(|b| (b'0'..=b'7').contains(&b))
}

/// A hexadecimal digit, in either case.
pub(crate) fn ascii_isxdigit(c: impl Into<c_int>) -> bool {
    as_byte(c.into()).is_some_and(|b| b.is_ascii_hexdigit())
}

/// A character that may appear in an identifier: a letter, a digit or `_`.
pub(crate) fn ascii_isident(c: impl Into<c_int>) -> bool {
    as_byte(c.into()).is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Horizontal whitespace: a space or a tab. Vim's notion of "white".
pub(crate) fn ascii_iswhite(c: impl Into<c_int>) -> bool {
    as_byte(c.into()).is_some_and(|b| b == b' ' || b == b'\t')
}

/// Horizontal whitespace, or the end of the string.
pub(crate) fn ascii_iswhite_or_nul(c: impl Into<c_int>) -> bool {
    as_byte(c.into()).is_some_and(|b| matches!(b, b' ' | b'\t' | 0))
}

/// Horizontal whitespace, a newline, or the end of the string.
pub(crate) fn ascii_iswhite_nl_or_nul(c: impl Into<c_int>) -> bool {
    as_byte(c.into()).is_some_and(|b| matches!(b, b' ' | b'\t' | b'\n' | 0))
}

/// Whitespace as `isspace()` sees it: `\t\n\v\f\r` or a space.
pub(crate) fn ascii_isspace(c: impl Into<c_int>) -> bool {
    as_byte(c.into()).is_some_and(|b| (9..=13).contains(&b) || b == b' ')
}

/// The path separator as a string, for the code that appends one.
pub(crate) const PATHSEPSTR: &::core::ffi::CStr = c"/";

/// The path separator.
pub(crate) const PATHSEP: u8 = b'/';

/// A horizontal tab. The control bytes below are bytes: a key code compares
/// against [`crate::types::Key`]'s copies, and a reader holding a `c_int`
/// widens the byte where it compares.
pub(crate) const TAB: u8 = 9;

/// A line feed.
pub(crate) const NL: u8 = 10;

/// A carriage return.
pub(crate) const CAR: u8 = 13;

/// The escape byte.
pub(crate) const ESC: u8 = 27;

/// A backspace.
pub(crate) const BS: u8 = 8;

/// A form feed.
pub(crate) const FF: u8 = 0x0c;

/// The delete byte, which some terminals send for backspace.
pub(crate) const DEL: u8 = 0x7f;

// The control bytes by the letter that types them, upstream's `Ctrl_*`, for
// readers of bytes (text, option values, `:menu` and mapping strings, the
// typeahead's raw input). A key compares against `Key::CTRL_*`.
/// CTRL-A.
pub(crate) const CTRL_A: u8 = 0x01;
/// CTRL-C.
pub(crate) const CTRL_C: u8 = 0x03;
/// CTRL-G.
pub(crate) const CTRL_G: u8 = 0x07;
/// CTRL-H.
pub(crate) const CTRL_H: u8 = 0x08;
/// CTRL-N.
pub(crate) const CTRL_N: u8 = 0x0e;
/// CTRL-O.
pub(crate) const CTRL_O: u8 = 0x0f;
/// CTRL-V, which quotes the next character.
pub(crate) const CTRL_V: u8 = 0x16;
/// CTRL-X.
pub(crate) const CTRL_X: u8 = 0x18;
/// CTRL-\.
pub(crate) const CTRL_BSL: u8 = 0x1c;
/// CTRL-].
pub(crate) const CTRL_RSB: u8 = 0x1d;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_the_boundaries_of_each_range() {
        assert!(!ascii_isdigit(b'0' - 1));
        assert!(ascii_isdigit(b'0'));
        assert!(ascii_isdigit(b'9'));
        assert!(!ascii_isdigit(b'9' + 1));
        assert!(ascii_isbdigit(b'1'));
        assert!(!ascii_isbdigit(b'2'));
        assert!(ascii_isxdigit(b'f'));
        assert!(ascii_isxdigit(b'F'));
        assert!(!ascii_isxdigit(b'g'));
        assert!(ascii_isident(b'_'));
        assert!(!ascii_isident(b'-'));
        assert!(ascii_iswhite(b'\t'));
        assert!(!ascii_iswhite(b'\n'));
        assert!(ascii_iswhite_or_nul(b'\0'));
        assert!(!ascii_iswhite_or_nul(b'\n'));
        assert!(ascii_iswhite_nl_or_nul(b'\n'));
        assert!(ascii_isspace(b'\r'));
        assert!(!ascii_isspace(8));
    }

    /// A `char` that widened to a negative `int`, and a terminal code past
    /// 0x7f, must fall out of every class rather than wrapping into one.
    #[test]
    fn rejects_values_outside_the_ascii_range() {
        for c in [-1, -128, 0x80, 0x100, c_int::MIN, c_int::MAX] {
            assert!(!ascii_isdigit(c));
            assert!(!ascii_isxdigit(c));
            assert!(!ascii_isident(c));
            assert!(!ascii_iswhite(c));
            assert!(!ascii_isspace(c));
        }
    }
}
