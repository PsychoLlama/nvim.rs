//! Case folding, case mapping, and case-insensitive comparison.
//!
//! Three different questions that are easy to confuse.
//!
//! **Folding** ([`Codepoint::fold`]) maps a character to a canonical form for
//! *matching*: it is what `==?`, `'ignorecase'` and `sort(l, 'i')` compare.
//! It is not lowercasing and the result need not be a character anyone would
//! type — the only promise is that two characters that should match fold to
//! the same value.
//!
//! **Case mapping** ([`Codepoint::to_upper`]/[`Codepoint::to_lower`]) produces the character a
//! user asked for with `gU`/`gu`, and answers to `'casemap'`: `internal` uses
//! utf8proc's tables, its absence hands the job to the C library's
//! locale-sensitive `towupper`/`towlower`, and `keepascii` pins ASCII to the
//! ASCII answer whatever the locale thinks.
//!
//! **Comparison** ([`strnicmp_in`]) is built on folding, and has to decode
//! both sides rather than walk bytes: two characters that fold together can
//! encode to different numbers of bytes.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::os::cshim::{byte_tolower, byte_toupper, wide_tolower, wide_toupper};
use core::ffi::{c_int, c_uint};

/// Is `'casemap'` asking for this flag?
fn casemap_has(flag: c_uint) -> bool {
    CMP_FLAGS.get() & flag != 0
}

impl Codepoint {
    /// The case-folded form, for matching: upstream's `utf_fold`.
    ///
    /// Two characters match case-insensitively exactly when their folds are
    /// equal.
    pub fn fold(self) -> Codepoint {
        let a = self.get();
        if a < 0x80 {
            // ASCII, without consulting a table.
            return if (0x41..=0x5a).contains(&a) {
                Codepoint::new(a + 32)
            } else {
                self
            };
        }

        // utf8proc only implements *full* case folding, where one character
        // can fold to several. Two characters are held back from it because
        // that breaks nvim elsewhere: ß folds to "ss", which makes the spell
        // tests raise E763 against the checked-in spell files, and İ folds to
        // "i" plus a combining dot. Upstream calls this a workaround and so
        // is this.
        if a == 0xdf || a == 0x130 {
            return self;
        }

        let mut folded = [0 as utf8proc_int32_t; 1];
        let n = utf8proc_decompose_char(a, &mut folded, UTF8PROC_CASEFOLD, None);
        // Anything that folds to more than one character keeps its own value.
        if n == 1 {
            Codepoint::new(folded[0])
        } else {
            self
        }
    }

    /// The uppercase, honouring `'casemap'`: upstream's `mb_toupper`.
    pub fn to_upper(self) -> Codepoint {
        let a = self.get();
        if a < 128 && casemap_has(OPT_CMP_FLAG_KEEPASCII as c_uint) {
            return if u8::try_from(a).is_ok_and(|b| b.is_ascii_lowercase()) {
                Codepoint::new(a - 32)
            } else {
                self
            };
        }
        if !casemap_has(OPT_CMP_FLAG_INTERNAL as c_uint) {
            // `towupper` takes and answers the `wint_t` reinterpretation.
            return Codepoint::new(wide_toupper(a.cast_unsigned()).cast_signed());
        }
        if a < 128 {
            // Locale-sensitive, which is the point: the locale may map ASCII
            // differently (Turkish dotless i).
            return Codepoint::new(byte_toupper(a));
        }
        Codepoint::new(utf8proc_toupper(a))
    }

    /// The lowercase, honouring `'casemap'`: upstream's `mb_tolower`. The
    /// mirror of [`Codepoint::to_upper`].
    pub fn to_lower(self) -> Codepoint {
        let a = self.get();
        if a < 128 && casemap_has(OPT_CMP_FLAG_KEEPASCII as c_uint) {
            return if u8::try_from(a).is_ok_and(|b| b.is_ascii_uppercase()) {
                Codepoint::new(a + 32)
            } else {
                self
            };
        }
        if !casemap_has(OPT_CMP_FLAG_INTERNAL as c_uint) {
            return Codepoint::new(wide_tolower(a.cast_unsigned()).cast_signed());
        }
        if a < 128 {
            return Codepoint::new(byte_tolower(a));
        }
        Codepoint::new(utf8proc_tolower(a))
    }

    /// Has it an uppercase form? Which is what "is lowercase" means here --
    /// a character with no case at all answers no. Upstream's `mb_islower`.
    pub fn is_lower(self) -> bool {
        self.to_upper() != self
    }

    /// Has it a lowercase form? Upstream's `mb_isupper`.
    pub fn is_upper(self) -> bool {
        self.to_lower() != self
    }

    /// Has it a case at all? Which is this port's test for "is a letter".
    /// Upstream's `mb_isalpha`.
    pub fn is_alpha(self) -> bool {
        self.is_lower() || self.is_upper()
    }
}

/// Read one character off the front of `text`, stepping past it.
///
/// Answers 0 when nothing is left (a NUL reads as nothing left, and is
/// stepped over), and −1 for a byte sequence that is not a character —
/// including a valid-looking one whose value equals its own first byte,
/// which is how an overlong encoding is caught. `0xC3 0x83` (Ã) is exempted
/// because it decodes to 0xC3, the value of its own lead byte, and is a real
/// character. A sequence that is not read is not stepped over either.
fn read_char(text: &mut &[u8]) -> c_int {
    let Some(&first) = text.first() else {
        return 0;
    };
    let len = usize::from(UTF8LEN_TAB_ZERO[usize::from(first)]);
    if len == 1 {
        *text = &text[1..];
        return c_int::from(first);
    }
    if len <= text.len() {
        let c = char_at(text);
        if c != first || (c == 0xc3 && text.get(1) == Some(&0x83)) {
            *text = &text[len..];
            return c.get();
        }
    }
    -1
}

/// Compare two strings case-insensitively, each to its own end. A NUL ends a
/// string early.
///
/// Characters are compared folded, so the answer does not depend on the case
/// either side was written in. Once either side runs out or hits a byte
/// sequence that is not a character, the comparison finishes **bytewise** —
/// which is upstream's, and deliberate: an arbitrary but consistent answer
/// for malformed input keeps `<?`/`>?` transitive, which a "these are
/// incomparable" answer would not.
///
/// The slice form of `utf_strnicmp`.
pub fn strnicmp_in(mut s1: &[u8], mut s2: &[u8]) -> c_int {
    let (c1, c2) = loop {
        let c1 = read_char(&mut s1);
        let c2 = read_char(&mut s2);
        if c1 <= 0 || c2 <= 0 {
            break (c1, c2);
        }
        if c1 == c2 {
            continue;
        }
        let cdiff = Codepoint::new(c1).fold().get() - Codepoint::new(c2).fold().get();
        if cdiff != 0 {
            return cdiff;
        }
    };

    // A string ended. The shorter one sorts first; both ended, equal.
    if c1 == 0 || c2 == 0 {
        return match (c1 == 0, c2 == 0) {
            (true, true) => 0,
            (true, false) => -1,
            _ => 1,
        };
    }

    // Exactly one side failed to decode. Compare the *folded* form of the
    // good side against the bad side's raw bytes, so the answer does not
    // depend on which case the good side was written in. Folding one
    // character is enough: the first byte comparison decides it.
    let mut folded = [0u8; MB_MAXCHAR];
    let (s1, s2): (&[u8], &[u8]) = if c1 != -1 && c2 == -1 {
        let len = encode_char(Codepoint::new(c1).fold(), &mut folded);
        (&folded[..len], s2)
    } else if c2 != -1 && c1 == -1 {
        let len = encode_char(Codepoint::new(c2).fold(), &mut folded);
        (s1, &folded[..len])
    } else {
        (s1, s2)
    };

    // Bytewise, each side ending at its end or its first NUL.
    let end = |s: &[u8]| s.iter().position(|&b| b == 0).unwrap_or(s.len());
    let (s1, s2) = (&s1[..end(s1)], &s2[..end(s2)]);
    for (&b1, &b2) in s1.iter().zip(s2) {
        let cdiff = c_int::from(b1) - c_int::from(b2);
        if cdiff != 0 {
            return cdiff;
        }
    }
    s1.len().cmp(&s2.len()) as c_int
}

/// `strcmp` or [`strnicmp_in`], whichever `ic` asks for. The slice form of
/// `mb_strcmp_ic`.
pub fn strcmp_ic_in(ic: bool, s1: &[u8], s2: &[u8]) -> c_int {
    if ic {
        strnicmp_in(s1, s2)
    } else {
        s1.cmp(s2) as c_int
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ASCII fast path, and the two characters held back from utf8proc.
    #[test]
    fn folding() {
        for c in 0..0x80 {
            let want = if (0x41..=0x5a).contains(&c) {
                c + 32
            } else {
                c
            };
            assert_eq!(Codepoint::new(c).fold(), Codepoint::new(want), "{c:#x}");
        }
        assert_eq!(Codepoint::new(0xdf).fold(), Codepoint::new(0xdf)); // ß, held back
        assert_eq!(Codepoint::new(0x130).fold(), Codepoint::new(0x130)); // İ, held back
        assert_eq!(Codepoint::new(0xc4).fold(), Codepoint::new(0xe4)); // Ä folds to ä
        assert_eq!(Codepoint::new(0x391).fold(), Codepoint::new(0x3b1)); // Α folds to α
    }
}
