//! [`Key`]: one key of input, the value `vgetc` answers.
//!
//! A key is a character or a special key, in one integer the way upstream
//! carries it: a character is its codepoint (non-negative, `NUL` meaning
//! "nothing"), a special key is the negative code [`SpecialKey`] names. The
//! control characters are themselves (`CTRL-V` is 0x16), and the modifiers
//! that did not fold into the key travel beside it in `mod_mask`.
//!
//! The newtype keeps that representation -- `#[repr(transparent)]` over the
//! `c_int`, every constructor a `const fn` -- so the typeahead and the
//! tables cost what they did; what it changes is that a key no longer
//! passes for a count, a column or a byte. [`Key::code`] is the integer, for
//! the typeahead's escape spelling and for the callers that still carry one.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;
use core::fmt;

use crate::keycodes::SpecialKey;

/// One key of input: a character, a control character, or a special key.
///
/// The default is [`Key::NUL`], no key.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct Key(c_int);

impl Key {
    /// No key: what a read answers when there is nothing to read.
    pub const NUL: Key = Key(0);
    /// `<Tab>`, CTRL-I.
    pub const TAB: Key = Key(0x09);
    /// `<NL>`, CTRL-J.
    pub const NL: Key = Key(0x0a);
    /// `<CR>`, CTRL-M.
    pub const CAR: Key = Key(0x0d);
    /// `<Esc>`, CTRL-[.
    pub const ESC: Key = Key(0x1b);
    /// `<BS>` as the byte, CTRL-H (not [`SpecialKey::Bs`]).
    pub const BS: Key = Key(0x08);
    /// `<Del>` as the byte, 0x7f (not [`SpecialKey::Del`]).
    pub const DEL: Key = Key(0x7f);
    /// `<Space>`.
    pub const SPACE: Key = Key(0x20);

    // The control characters by the letter that types them, upstream's
    // `Ctrl_*`: named so that a `match` can test for them.
    /// CTRL-A.
    pub const CTRL_A: Key = Key::ctrl(b'A');
    /// CTRL-B.
    pub const CTRL_B: Key = Key::ctrl(b'B');
    /// CTRL-C.
    pub const CTRL_C: Key = Key::ctrl(b'C');
    /// CTRL-D.
    pub const CTRL_D: Key = Key::ctrl(b'D');
    /// CTRL-E.
    pub const CTRL_E: Key = Key::ctrl(b'E');
    /// CTRL-F.
    pub const CTRL_F: Key = Key::ctrl(b'F');
    /// CTRL-G.
    pub const CTRL_G: Key = Key::ctrl(b'G');
    /// CTRL-H.
    pub const CTRL_H: Key = Key::ctrl(b'H');
    /// CTRL-I.
    pub const CTRL_I: Key = Key::ctrl(b'I');
    /// CTRL-J.
    pub const CTRL_J: Key = Key::ctrl(b'J');
    /// CTRL-K.
    pub const CTRL_K: Key = Key::ctrl(b'K');
    /// CTRL-L.
    pub const CTRL_L: Key = Key::ctrl(b'L');
    /// CTRL-M.
    pub const CTRL_M: Key = Key::ctrl(b'M');
    /// CTRL-N.
    pub const CTRL_N: Key = Key::ctrl(b'N');
    /// CTRL-O.
    pub const CTRL_O: Key = Key::ctrl(b'O');
    /// CTRL-P.
    pub const CTRL_P: Key = Key::ctrl(b'P');
    /// CTRL-Q.
    pub const CTRL_Q: Key = Key::ctrl(b'Q');
    /// CTRL-R.
    pub const CTRL_R: Key = Key::ctrl(b'R');
    /// CTRL-S.
    pub const CTRL_S: Key = Key::ctrl(b'S');
    /// CTRL-T.
    pub const CTRL_T: Key = Key::ctrl(b'T');
    /// CTRL-U.
    pub const CTRL_U: Key = Key::ctrl(b'U');
    /// CTRL-V.
    pub const CTRL_V: Key = Key::ctrl(b'V');
    /// CTRL-W.
    pub const CTRL_W: Key = Key::ctrl(b'W');
    /// CTRL-X.
    pub const CTRL_X: Key = Key::ctrl(b'X');
    /// CTRL-Y.
    pub const CTRL_Y: Key = Key::ctrl(b'Y');
    /// CTRL-Z.
    pub const CTRL_Z: Key = Key::ctrl(b'Z');
    /// CTRL-\\.
    pub const CTRL_BSL: Key = Key(0x1c);
    /// CTRL-].
    pub const CTRL_RSB: Key = Key(0x1d);
    /// CTRL-^.
    pub const CTRL_HAT: Key = Key(0x1e);
    /// CTRL-_.
    pub const CTRL_UNDERSCORE: Key = Key(0x1f);

    /// The key that types `c`.
    #[inline]
    pub const fn from_char(c: char) -> Key {
        // A scalar value is at most 0x10ffff: it fits, and is never negative.
        Key(c as c_int)
    }

    /// The key that types the ASCII character `b`.
    ///
    /// For literals (`Key::ascii(b'x')`); a byte above 0x7f is a lead or
    /// continuation byte, not a character, and is refused.
    #[inline]
    pub const fn ascii(b: u8) -> Key {
        assert!(b < 0x80, "not an ASCII character");
        Key(b as c_int)
    }

    /// A byte read as the key of the same number -- what one byte of the
    /// typeahead answers before the bytes of a character are assembled.
    #[inline]
    pub const fn byte(b: u8) -> Key {
        Key(b as c_int)
    }

    /// CTRL with `letter`: `Key::ctrl(b'V')` is 0x16. Upstream's `Ctrl_chr`.
    #[inline]
    pub const fn ctrl(letter: u8) -> Key {
        Key((letter & 0x1f) as c_int)
    }

    /// The key a special key is.
    #[inline]
    pub const fn from_special(key: SpecialKey) -> Key {
        Key(key.code())
    }

    /// The key whose upstream integer is `code`: a codepoint (decoded from
    /// text, so possibly above `U+10FFFF` or a surrogate, which a `char`
    /// cannot hold) or a special key's negative code.
    #[inline]
    pub const fn from_code(code: c_int) -> Key {
        Key(code)
    }

    /// The upstream integer: what the escape spelling encodes and what the
    /// code still carrying keys as `c_int` reads.
    #[inline]
    pub const fn code(self) -> c_int {
        self.0
    }

    /// Whether this is no key at all.
    #[inline]
    pub const fn is_nul(self) -> bool {
        self.0 == 0
    }

    /// Whether this is a special key rather than a character (upstream's
    /// `IS_SPECIAL`).
    #[inline]
    pub const fn is_special(self) -> bool {
        self.0 < 0
    }

    /// Whether this is the key that types `c`.
    #[inline]
    pub const fn is(self, c: char) -> bool {
        self.0 == c as c_int
    }

    /// The character, when this is one a `char` can hold.
    #[inline]
    pub fn as_char(self) -> Option<char> {
        u32::try_from(self.0).ok().and_then(char::from_u32)
    }

    /// The character as one byte, when it is below 0x100.
    #[inline]
    pub fn as_byte(self) -> Option<u8> {
        u8::try_from(self.0).ok()
    }

    /// The named special key, when this is one.
    #[inline]
    pub fn special(self) -> Option<SpecialKey> {
        SpecialKey::try_from(self.0).ok()
    }

    /// How many bytes the character takes in UTF-8 -- one for a special
    /// key, which is upstream's answer for a negative number.
    #[inline]
    pub fn utf8_len(self) -> usize {
        crate::mbyte::utf8_len(self.0)
    }
}

impl From<SpecialKey> for Key {
    #[inline]
    fn from(key: SpecialKey) -> Key {
        Key::from_special(key)
    }
}

impl From<char> for Key {
    #[inline]
    fn from(c: char) -> Key {
        Key::from_char(c)
    }
}

impl PartialEq<SpecialKey> for Key {
    #[inline]
    fn eq(&self, other: &SpecialKey) -> bool {
        self.0 == other.code()
    }
}

impl PartialEq<Key> for SpecialKey {
    #[inline]
    fn eq(&self, other: &Key) -> bool {
        self.code() == other.0
    }
}

/// A character prints as itself, anything else in `<>` notation.
impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_char() {
            Some(c) if !c.is_control() => write!(f, "{c}"),
            _ => {
                let name =
                    crate::keycodes::get_special_key_name(*self, crate::keycodes::ModMask::NONE);
                let len = name.iter().position(|&b| b == 0).unwrap_or(name.len());
                f.write_str(&String::from_utf8_lossy(&name[..len]))
            }
        }
    }
}
