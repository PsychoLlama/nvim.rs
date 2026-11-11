//! [`Codepoint`]: one character of decoded text.
//!
//! What [`char_at`](super::char_at) answers and what
//! [`encode_char`](super::encode_char) writes back: the number a UTF-8
//! sequence carries. The codec is the original six-byte UTF-8, and a reader
//! of text gets back whatever the bytes say -- a surrogate, a value past
//! `U+10FFFF`, the lead byte's own value for a sequence it could not decode
//! -- and the editor round-trips all of them unchanged. So this is not a
//! `char`: a `char` refuses half of what the text holds. [`to_char`] is the
//! crossing for the readers that need a real scalar value.
//!
//! It is the integer upstream carried, `#[repr(transparent)]` over the same
//! `int`, so a walk costs what it did; what changes is that a codepoint no
//! longer passes for a key, a byte, a count or a column. A byte compares
//! against it directly (`c == b'\\'`), which is the comparison upstream wrote
//! as `c == '\\'`; arithmetic reads [`get`].
//!
//! The crossings are spelled: text to a key is `Key::from_code`, a key to
//! text `Key::code`; a byte widens with `Codepoint::from(byte)`.
//!
//! [`to_char`]: Codepoint::to_char
//! [`get`]: Codepoint::get

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::cmp::Ordering;
use core::fmt;

/// One character of decoded text: the value a UTF-8 sequence carries.
///
/// Any `i32` the codec can produce, which is more than a `char` holds; the
/// default is [`Codepoint::NUL`].
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Codepoint(i32);

const _: () = assert!(
    size_of::<Codepoint>() == 4,
    "a codepoint is the int upstream carried"
);

impl Codepoint {
    /// The NUL character: what a read past the end of the text answers.
    pub const NUL: Codepoint = Codepoint(0);

    /// The codepoint whose number is `value`: what a decoder, a table or
    /// an integer channel (a `typval` number, an option's value) answers.
    #[inline]
    pub const fn new(value: i32) -> Codepoint {
        Codepoint(value)
    }

    /// The character the byte `b` is, read as Latin-1 the way upstream
    /// widens a `char_u`. A `const` spelling of `Codepoint::from(b)`, for
    /// constants and `match` patterns.
    #[inline]
    pub const fn byte(b: u8) -> Codepoint {
        Codepoint(b as i32)
    }

    /// The number: for arithmetic, and for the integer channels a
    /// codepoint leaves through.
    #[inline]
    pub const fn get(self) -> i32 {
        self.0
    }

    /// The character as a `char`, when it is a Unicode scalar value.
    #[inline]
    pub fn to_char(self) -> Option<char> {
        u32::try_from(self.0).ok().and_then(char::from_u32)
    }

    /// The character as one byte, when it is below 0x100.
    #[inline]
    pub fn as_byte(self) -> Option<u8> {
        u8::try_from(self.0).ok()
    }

    /// Whether this is NUL.
    #[inline]
    pub const fn is_nul(self) -> bool {
        self.0 == 0
    }

    /// Whether this is an ASCII character, `U+0000..=U+007F`.
    #[inline]
    pub const fn is_ascii(self) -> bool {
        0 <= self.0 && self.0 < 0x80
    }

    /// Whether this is `A`..=`Z`.
    #[inline]
    pub fn is_ascii_uppercase(self) -> bool {
        self.as_byte().is_some_and(|b| b.is_ascii_uppercase())
    }

    /// Whether this is `a`..=`z`.
    #[inline]
    pub fn is_ascii_lowercase(self) -> bool {
        self.as_byte().is_some_and(|b| b.is_ascii_lowercase())
    }

    /// Whether this is `0`..=`9`.
    #[inline]
    pub fn is_ascii_digit(self) -> bool {
        self.as_byte().is_some_and(|b| b.is_ascii_digit())
    }

    /// How many bytes this takes in the codec's six-byte UTF-8: one for
    /// anything below 0x80 (a special key's negative code included).
    #[inline]
    pub const fn utf8_len(self) -> usize {
        let c = self.0;
        if c < 0x80 {
            1
        } else if c < 0x800 {
            2
        } else if c < 0x1_0000 {
            3
        } else if c < 0x20_0000 {
            4
        } else if c < 0x400_0000 {
            5
        } else {
            6
        }
    }
}

impl From<u8> for Codepoint {
    #[inline]
    fn from(b: u8) -> Codepoint {
        Codepoint(i32::from(b))
    }
}

impl From<char> for Codepoint {
    #[inline]
    fn from(c: char) -> Codepoint {
        // A scalar value is at most 0x10ffff: it fits, and is never negative.
        Codepoint(i32::try_from(u32::from(c)).expect("a scalar value fits an i32"))
    }
}

impl From<Codepoint> for i32 {
    /// The number, for a reader that takes anything that widens to an
    /// `int` (the [`crate::ascii`] predicates).
    #[inline]
    fn from(c: Codepoint) -> i32 {
        c.0
    }
}

impl From<Codepoint> for i64 {
    /// The number, as the `VarNumber` a script sees.
    #[inline]
    fn from(c: Codepoint) -> i64 {
        i64::from(c.0)
    }
}

impl TryFrom<Codepoint> for u8 {
    type Error = core::num::TryFromIntError;

    /// The character as one byte, when it is below 0x100.
    #[inline]
    fn try_from(c: Codepoint) -> Result<u8, Self::Error> {
        u8::try_from(c.0)
    }
}

impl TryFrom<Codepoint> for char {
    type Error = core::char::CharTryFromError;

    #[inline]
    fn try_from(c: Codepoint) -> Result<char, Self::Error> {
        char::try_from(c.0.cast_unsigned())
    }
}

impl PartialEq<u8> for Codepoint {
    /// Whether this is the character the byte is (`c == b'\\'`).
    #[inline]
    fn eq(&self, b: &u8) -> bool {
        self.0 == i32::from(*b)
    }
}

impl PartialOrd<u8> for Codepoint {
    /// Order against the character the byte is (`c < b' '`).
    #[inline]
    fn partial_cmp(&self, b: &u8) -> Option<Ordering> {
        Some(self.0.cmp(&i32::from(*b)))
    }
}

impl PartialEq<Codepoint> for u8 {
    #[inline]
    fn eq(&self, c: &Codepoint) -> bool {
        c == self
    }
}

impl PartialOrd<Codepoint> for u8 {
    /// The mirror of `Codepoint: PartialOrd<u8>`, so a byte range holds a
    /// codepoint (`(b'A'..=b'Z').contains(&c)`).
    #[inline]
    fn partial_cmp(&self, c: &Codepoint) -> Option<Ordering> {
        Some(i32::from(*self).cmp(&c.0))
    }
}

impl PartialEq<char> for Codepoint {
    /// Whether this is the character `c` (`c == '…'`).
    #[inline]
    fn eq(&self, c: &char) -> bool {
        *self == Codepoint::from(*c)
    }
}

impl fmt::Display for Codepoint {
    /// `U+XXXX`, the way Unicode names a codepoint.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "U+{:04X}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_byte_compares_as_the_character_it_widens_to() {
        let backslash = Codepoint::byte(b'\\');
        assert!(backslash == b'\\');
        assert!(backslash != b'/');
        assert!(Codepoint::byte(b' ') < b'!');
        assert!(Codepoint::new(0x100) > b'\xff');
        assert_eq!(Codepoint::byte(0xe9), Codepoint::from('é'));
        assert!(Codepoint::from('é') == 'é');
        assert!((b'!'..=b'~').contains(&Codepoint::byte(b'Q')));
        assert!(!(b'!'..=b'~').contains(&Codepoint::new(0x151)));
        assert!(Codepoint::byte(b'Q').is_ascii_uppercase());
        assert!(!Codepoint::new(0x151).is_ascii_uppercase());
        assert!(Codepoint::byte(b'q').is_ascii_lowercase());
        assert!(Codepoint::byte(b'7').is_ascii_digit());
    }

    #[test]
    fn decoded_text_holds_more_than_a_char() {
        for value in [0xd800, 0x11_0000, 0x7fff_ffff] {
            let c = Codepoint::new(value);
            assert_eq!(c.to_char(), None);
            assert!(char::try_from(c).is_err());
            assert_eq!(c.get(), value);
        }
        assert_eq!(Codepoint::from('€').to_char(), Some('€'));
        assert_eq!(Codepoint::new(0x41).as_byte(), Some(b'A'));
        assert_eq!(Codepoint::new(0x100).as_byte(), None);
    }

    #[test]
    fn lengths_follow_the_six_byte_codec() {
        for (value, len) in [
            (0, 1),
            (0x7f, 1),
            (0x80, 2),
            (0x800, 3),
            (0x1_0000, 4),
            (0x20_0000, 5),
        ] {
            assert_eq!(Codepoint::new(value).utf8_len(), len, "{value:#x}");
        }
        assert_eq!(Codepoint::new(0x400_0000).utf8_len(), 6);
        assert!(Codepoint::NUL.is_nul() && Codepoint::NUL.is_ascii());
        assert_eq!(format!("{}", Codepoint::from('€')), "U+20AC");
    }
}
