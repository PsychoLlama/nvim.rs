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

use core::ffi::{c_int, c_uint};

/// A decimal digit.
pub(crate) fn ascii_isdigit(c: c_int) -> bool {
    (c_int::from(b'0')..=c_int::from(b'9')).contains(&c)
}

/// `ASCII_ISALPHA`: an unaccented Latin letter.
pub(crate) fn ascii_isalpha(c: c_int) -> bool {
    // Unsigned, so that a negative byte fails both ranges rather than
    // wrapping into one of them -- which is what the C macro's cast does.
    let c = c.cast_unsigned();
    (c_uint::from(b'A')..=c_uint::from(b'Z')).contains(&c)
        || (c_uint::from(b'a')..=c_uint::from(b'z')).contains(&c)
}

/// `ASCII_ISUPPER`: an unaccented upper-case Latin letter.
pub(crate) fn ascii_isupper(c: c_int) -> bool {
    (c_uint::from(b'A')..=c_uint::from(b'Z')).contains(&c.cast_unsigned())
}

/// A binary digit.
pub(crate) fn ascii_isbdigit(c: c_int) -> bool {
    c == c_int::from(b'0') || c == c_int::from(b'1')
}

/// A hexadecimal digit, in either case.
pub(crate) fn ascii_isxdigit(c: c_int) -> bool {
    ascii_isdigit(c)
        || (c_int::from(b'a')..=c_int::from(b'f')).contains(&c)
        || (c_int::from(b'A')..=c_int::from(b'F')).contains(&c)
}

/// A character that may appear in an identifier: a letter, a digit or `_`.
pub(crate) fn ascii_isident(c: c_int) -> bool {
    (c_int::from(b'A')..=c_int::from(b'Z')).contains(&c)
        || (c_int::from(b'a')..=c_int::from(b'z')).contains(&c)
        || ascii_isdigit(c)
        || c == c_int::from(b'_')
}

/// Horizontal whitespace: a space or a tab. Vim's notion of "white".
pub(crate) fn ascii_iswhite(c: c_int) -> bool {
    c == c_int::from(b' ') || c == c_int::from(b'\t')
}

/// Horizontal whitespace, or the end of the string.
pub(crate) fn ascii_iswhite_or_nul(c: c_int) -> bool {
    ascii_iswhite(c) || c == NUL
}

/// Horizontal whitespace, a newline, or the end of the string.
pub(crate) fn ascii_iswhite_nl_or_nul(c: c_int) -> bool {
    ascii_iswhite(c) || c == c_int::from(b'\n') || c == NUL
}

/// Whitespace as `isspace()` sees it: `\t\n\v\f\r` or a space.
pub(crate) fn ascii_isspace(c: c_int) -> bool {
    (9..=13).contains(&c) || c == c_int::from(b' ')
}

/// The string terminator, as the `int` a widened byte compares against.
/// `ascii_defs.h`'s `NUL`, one copy for the tree.
pub(crate) const NUL: ::core::ffi::c_int = 0;

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
        assert!(!ascii_isdigit(c_int::from(b'0') - 1));
        assert!(ascii_isdigit(c_int::from(b'0')));
        assert!(ascii_isdigit(c_int::from(b'9')));
        assert!(!ascii_isdigit(c_int::from(b'9') + 1));
        assert!(ascii_isbdigit(c_int::from(b'1')));
        assert!(!ascii_isbdigit(c_int::from(b'2')));
        assert!(ascii_isxdigit(c_int::from(b'f')));
        assert!(ascii_isxdigit(c_int::from(b'F')));
        assert!(!ascii_isxdigit(c_int::from(b'g')));
        assert!(ascii_isident(c_int::from(b'_')));
        assert!(!ascii_isident(c_int::from(b'-')));
        assert!(ascii_iswhite(c_int::from(b'\t')));
        assert!(!ascii_iswhite(c_int::from(b'\n')));
        assert!(ascii_iswhite_or_nul(NUL));
        assert!(!ascii_iswhite_or_nul(c_int::from(b'\n')));
        assert!(ascii_iswhite_nl_or_nul(c_int::from(b'\n')));
        assert!(ascii_isspace(c_int::from(b'\r')));
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
