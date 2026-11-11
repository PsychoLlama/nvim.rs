//! [`Key`]: one key of input, the value `vgetc` answers.
//!
//! A key is a character or a special key, in one integer the way upstream
//! carries it: a character is its codepoint (non-negative, `NUL` meaning
//! "nothing"), a special key is the negative code [`SpecialKey`] names. The
//! control characters are themselves (`CTRL-V` is 0x16), and the modifiers
//! that did not fold into the key travel beside it in `MOD_MASK`.
//!
//! The newtype keeps that representation -- `#[repr(transparent)]` over the
//! `c_int`, every constructor a `const fn` -- so the typeahead and the
//! tables cost what they did; what it changes is that a key no longer
//! passes for a count, a column or a byte. [`Key::code`] is the integer, for
//! the typeahead's escape spelling and for the callers that still carry one.
//!
//! A reader that dispatches on a key matches [`Key::kind`], the enum of the
//! three halves the integer packs: a character ([`KeyKind::Char`], matched
//! against `Key` constants such as [`Key::CTRL_W`]), a named special key
//! ([`KeyKind::Special`]) or a [`Pseudo`] key -- a code that travels where a
//! key does but that no key types. The enum is the *view*, not the storage: a
//! stored enum would need a tag beside a payload that already uses every bit
//! a character can (text decodes to codepoints a `char` cannot hold), and the
//! typeahead and the key tables would double. The newtype stays four bytes.

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
use crate::mbyte::Codepoint;

/// One key of input: a character, a control character, or a special key.
///
/// The default is [`Key::NUL`], no key. Keys order as upstream's integers
/// do, special keys (negative) below every character, for the range tests
/// upstream writes as `c < ' '`.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Key(c_int);

const _: () = assert!(
    size_of::<Key>() == 4,
    "the typeahead stores keys at four bytes"
);
const _: () = assert!(size_of::<KeyKind>() <= 8, "the view stays in two registers");

/// A code that travels where a key does but that no key types: the operator
/// and indent machinery hand these to a reader of keys to say something a
/// key cannot.
///
/// They are negative, below every [`SpecialKey`] (whose codes are at least
/// 256 away from zero), so no typed key or character is ever one -- unlike
/// upstream's `KEY_OPEN_FORW`, `KEY_OPEN_BACK` and `KEY_COMPLETE`, which were
/// U+0101..U+0103 and so were also typed by `ā`, `Ă` and `ă`.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub(crate) enum Pseudo {
    /// `r CTRL-V <CR>` over a selection: the literal carriage return, not a
    /// line break. Upstream's `REPLACE_CR_NCHAR`.
    ReplaceCr = -1,
    /// `r CTRL-V <NL>` over a selection: the literal line feed. Upstream's
    /// `REPLACE_NL_NCHAR`.
    ReplaceNl = -2,
    /// The `o` command opened a line: the 'cinkeys' `o` item.
    OpenForward = -3,
    /// The `O` command opened a line: the 'cinkeys' `O` item.
    OpenBackward = -4,
    /// A completion just ended: the 'cinkeys' `=word` items.
    Completed = -5,
}

impl Pseudo {
    /// The pseudo-key whose code is `code`, if it is one.
    const fn from_code(code: c_int) -> Option<Pseudo> {
        Some(match code {
            -1 => Pseudo::ReplaceCr,
            -2 => Pseudo::ReplaceNl,
            -3 => Pseudo::OpenForward,
            -4 => Pseudo::OpenBackward,
            -5 => Pseudo::Completed,
            _ => return None,
        })
    }
}

/// What a [`Key`] is, for a `match` over all of it: [`Key::kind`].
///
/// A character keeps its `Key`, so an arm names it by the constants --
/// `KeyKind::Char(Key::CTRL_J | DOWN)` beside `KeyKind::Special(SpecialKey::Down)`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum KeyKind {
    /// A character: a codepoint, `NUL` and the control characters included.
    Char(Key),
    /// A key the `<>` notation names.
    Special(SpecialKey),
    /// A code no key types.
    Pseudo(Pseudo),
    /// A negative code nothing names: a termcap entry with no name here.
    Unnamed(Key),
}

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

    /// [`Pseudo::ReplaceCr`].
    pub const REPLACE_CR: Key = Key::pseudo(Pseudo::ReplaceCr);
    /// [`Pseudo::ReplaceNl`].
    pub const REPLACE_NL: Key = Key::pseudo(Pseudo::ReplaceNl);
    /// [`Pseudo::OpenForward`].
    pub const OPEN_FORWARD: Key = Key::pseudo(Pseudo::OpenForward);
    /// [`Pseudo::OpenBackward`].
    pub const OPEN_BACKWARD: Key = Key::pseudo(Pseudo::OpenBackward);
    /// [`Pseudo::Completed`].
    pub const COMPLETED: Key = Key::pseudo(Pseudo::Completed);

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

    /// The code a pseudo-key travels as.
    #[inline]
    pub(crate) const fn pseudo(key: Pseudo) -> Key {
        Key(key as c_int)
    }

    /// What this key is: a character, a named key or a pseudo-key.
    #[inline]
    pub(crate) fn kind(self) -> KeyKind {
        if self.0 >= 0 {
            KeyKind::Char(self)
        } else if let Some(key) = self.special() {
            KeyKind::Special(key)
        } else if let Some(key) = Pseudo::from_code(self.0) {
            KeyKind::Pseudo(key)
        } else {
            KeyKind::Unnamed(self)
        }
    }

    /// The key a special key is.
    #[inline]
    pub const fn from_special(key: SpecialKey) -> Key {
        Key(key.code())
    }

    /// The key that types the codepoint `c`: the crossing from text (a
    /// character read from a buffer, a register, a mapping's string) to
    /// input. `c` may be above `U+10FFFF` or a surrogate, which a `char`
    /// cannot hold; the typeahead carries it unchanged.
    #[inline]
    pub const fn from_code(c: Codepoint) -> Key {
        Key(c.get())
    }

    /// The character this key types, as text: the crossing from input
    /// back to a codepoint, for a reader that encodes, classifies or
    /// compares it. A special key passes its negative code through, as
    /// upstream's `int` did; the display and width functions render that
    /// as its `~@` form, and every other reader asks [`Key::is_char`]
    /// first.
    #[inline]
    pub const fn code(self) -> Codepoint {
        Codepoint::new(self.0)
    }

    /// The key whose upstream integer is `raw`: an integer channel that
    /// carries keys and special keys alike -- the escape spelling's
    /// decode, a termcap table, an option such as 'wildchar' that holds a
    /// key code, a `typval` number.
    #[inline]
    pub const fn from_raw(raw: c_int) -> Key {
        Key(raw)
    }

    /// The upstream integer, for the escape spelling, the key tables and
    /// the integer channels a key leaves through.
    #[inline]
    pub const fn raw(self) -> c_int {
        self.0
    }

    /// Whether this is no key at all.
    #[inline]
    pub const fn is_nul(self) -> bool {
        self.0 == 0
    }

    /// Whether this is a character other than NUL -- neither "no key" nor a
    /// special key (upstream's `c > 0`).
    #[inline]
    pub const fn is_char(self) -> bool {
        self.0 > 0
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

    /// The value of an ASCII decimal digit, when this is one.
    #[inline]
    pub fn digit(self) -> Option<u8> {
        self.as_byte().filter(u8::is_ascii_digit).map(|b| b - b'0')
    }

    /// The character as one byte, when it is below 0x100.
    #[inline]
    pub fn as_byte(self) -> Option<u8> {
        u8::try_from(self.0).ok()
    }

    /// The named special key, when this is one.
    #[inline]
    pub fn special(self) -> Option<SpecialKey> {
        SpecialKey::from_code(self.0)
    }

    /// How many bytes the character takes in UTF-8 -- one for a special
    /// key, which is upstream's answer for a negative number.
    #[inline]
    pub fn utf8_len(self) -> usize {
        self.code().utf8_len()
    }
}

impl From<SpecialKey> for Key {
    #[inline]
    fn from(key: SpecialKey) -> Key {
        Key::from_special(key)
    }
}

impl From<Pseudo> for Key {
    #[inline]
    fn from(key: Pseudo) -> Key {
        Key::pseudo(key)
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

#[cfg(test)]
mod tests {
    use super::Key;
    use crate::keycodes::SpecialKey;

    #[test]
    fn the_control_consts_are_the_control_characters() {
        let letters = [
            (Key::CTRL_A, b'A'),
            (Key::CTRL_H, b'H'),
            (Key::CTRL_O, b'O'),
            (Key::CTRL_V, b'V'),
            (Key::CTRL_Z, b'Z'),
            (Key::CTRL_BSL, b'\\'),
            (Key::CTRL_RSB, b']'),
            (Key::CTRL_HAT, b'^'),
            (Key::CTRL_UNDERSCORE, b'_'),
        ];
        for (key, letter) in letters {
            assert_eq!(key, Key::ctrl(letter), "{}", char::from(letter));
        }
        assert_eq!(Key::CTRL_I, Key::TAB);
        assert_eq!(Key::CTRL_M, Key::CAR);
        assert_eq!(Key::SPACE, Key::ascii(b' '));
    }

    #[test]
    fn kind_names_each_half_and_no_pseudo_key_is_typed() {
        use super::{KeyKind, Pseudo};
        assert_eq!(Key::ascii(b'w').kind(), KeyKind::Char(Key::ascii(b'w')));
        assert_eq!(Key::NUL.kind(), KeyKind::Char(Key::NUL));
        assert_eq!(
            Key::from(SpecialKey::Down).kind(),
            KeyKind::Special(SpecialKey::Down)
        );
        for p in [
            Pseudo::ReplaceCr,
            Pseudo::ReplaceNl,
            Pseudo::OpenForward,
            Pseudo::OpenBackward,
            Pseudo::Completed,
        ] {
            let key = Key::from(p);
            assert_eq!(key.kind(), KeyKind::Pseudo(p));
            assert!(key.special().is_none(), "{p:?} is a named key");
            assert!(
                SpecialKey::ALL.iter().all(|k| k.code().abs() > 255),
                "{p:?}"
            );
        }
        assert_eq!(
            Key::from_raw(-300).kind(),
            KeyKind::Unnamed(Key::from_raw(-300))
        );
        // Upstream's `KEY_OPEN_FORW` was U+0101: a character, here only that.
        assert_eq!(
            Key::from_char('\u{101}').kind(),
            KeyKind::Char(Key::from_char('\u{101}'))
        );
    }

    #[test]
    fn a_char_is_neither_nul_nor_special_and_a_digit_is_ascii() {
        assert!(Key::ascii(b'a').is_char());
        assert!(Key::from_char('\u{e9}').is_char());
        assert!(!Key::NUL.is_char());
        assert!(!Key::from(SpecialKey::Left).is_char());
        for (b, d) in (b'0'..=b'9').zip(0..) {
            assert_eq!(Key::byte(b).digit(), Some(d));
        }
        for key in [
            Key::ascii(b'a'),
            Key::NUL,
            Key::from_char('\u{661}'),
            Key::from(SpecialKey::K0),
        ] {
            assert_eq!(key.digit(), None, "{key:?}");
        }
    }
}
