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

use crate::types::NUL;
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

/// `ASCII_ISLOWER`: an unaccented lower-case Latin letter.
pub(crate) fn ascii_islower(c: c_int) -> bool {
    // Unsigned, as `ascii_isalpha`, so a negative byte fails rather than
    // wrapping into the range.
    (c_uint::from(b'a')..=c_uint::from(b'z')).contains(&c.cast_unsigned())
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
