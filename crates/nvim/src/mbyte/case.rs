//! Case folding, case mapping, and case-insensitive comparison.
//!
//! Three different questions that are easy to confuse.
//!
//! **Folding** ([`utf_fold`]) maps a character to a canonical form for
//! *matching*: it is what `==?`, `'ignorecase'` and `sort(l, 'i')` compare.
//! It is not lowercasing and the result need not be a character anyone would
//! type — the only promise is that two characters that should match fold to
//! the same value.
//!
//! **Case mapping** ([`mb_toupper`]/[`mb_tolower`]) produces the character a
//! user asked for with `gU`/`gu`, and answers to `'casemap'`: `internal` uses
//! utf8proc's tables, its absence hands the job to the C library's
//! locale-sensitive `towupper`/`towlower`, and `keepascii` pins ASCII to the
//! ASCII answer whatever the locale thinks.
//!
//! **Comparison** ([`utf_strnicmp`]) is built on folding, and has to decode
//! both sides rather than walk bytes: two characters that fold together can
//! encode to different numbers of bytes.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::os::cshim::{towlower, towupper, wint_t};
use crate::types::NUL;
use core::ffi::{c_char, c_int, c_uint};

/// The case-folded form of `a`, for matching.
///
/// Two characters match case-insensitively exactly when their folds are equal.
#[unsafe(no_mangle)]
pub extern "C" fn utf_fold(a: c_int) -> c_int {
    if a < 0x80 {
        // ASCII, without consulting a table.
        return if (0x41..=0x5a).contains(&a) {
            a + 32
        } else {
            a
        };
    }

    // utf8proc only implements *full* case folding, where one character can
    // fold to several. Two characters are held back from it because that
    // breaks nvim elsewhere: ß folds to "ss", which makes the spell tests
    // raise E763 against the checked-in spell files, and İ folds to "i" plus
    // a combining dot. Upstream calls this a workaround and so is this.
    if a == 0xdf || a == 0x130 {
        return a;
    }

    let mut folded = [0 as utf8proc_int32_t; 1];
    let n = utf8proc_decompose_char(a as utf8proc_int32_t, &mut folded, UTF8PROC_CASEFOLD, None);
    // Anything that folds to more than one character keeps its own value.
    if n == 1 { folded[0] as c_int } else { a }
}

/// Is `'casemap'` asking for this flag?
fn casemap_has(flag: c_uint) -> bool {
    cmp_flags.get() & flag != 0
}

/// The uppercase of `a`, honouring `'casemap'`.
pub fn mb_toupper(a: c_int) -> c_int {
    if a < 128 && casemap_has(kOptCmpFlagKeepascii as c_uint) {
        return if (b'a' as c_int..=b'z' as c_int).contains(&a) {
            a - 32
        } else {
            a
        };
    }
    if !casemap_has(kOptCmpFlagInternal as c_uint) {
        // SAFETY: `towupper` is a pure libc function over one wide character.
        return unsafe { towupper(a as wint_t) } as c_int;
    }
    if a < 128 {
        // Locale-sensitive, which is the point: the locale may map ASCII
        // differently (Turkish dotless i).
        // SAFETY: `toupper` is a pure libc function over one byte value.
        return unsafe { toupper(a) };
    }
    utf8proc_toupper(a as utf8proc_int32_t) as c_int
}

/// The lowercase of `a`, honouring `'casemap'`. The mirror of [`mb_toupper`].
pub fn mb_tolower(a: c_int) -> c_int {
    if a < 128 && casemap_has(kOptCmpFlagKeepascii as c_uint) {
        return if (b'A' as c_int..=b'Z' as c_int).contains(&a) {
            a + 32
        } else {
            a
        };
    }
    if !casemap_has(kOptCmpFlagInternal as c_uint) {
        // SAFETY: `towlower` is a pure libc function over one wide character.
        return unsafe { towlower(a as wint_t) } as c_int;
    }
    if a < 128 {
        // SAFETY: `tolower` is a pure libc function over one byte value.
        return unsafe { tolower(a) };
    }
    utf8proc_tolower(a as utf8proc_int32_t) as c_int
}

/// Has `a` an uppercase form? Which is what "is lowercase" means here — a
/// character with no case at all answers no.
pub fn mb_islower(a: c_int) -> bool {
    mb_toupper(a) != a
}

/// Has `a` a lowercase form?
pub fn mb_isupper(a: c_int) -> bool {
    mb_tolower(a) != a
}

/// Has `a` a case at all? Which is this port's test for "is a letter".
pub fn mb_isalpha(a: c_int) -> bool {
    mb_islower(a) || mb_isupper(a)
}

/// Compare two strings case-insensitively, at most `n1`/`n2` bytes each.
///
/// Characters are compared folded, so the answer does not depend on the case
/// either side was written in. Once either side runs out or hits a byte
/// sequence that is not a character, the comparison finishes **bytewise** —
/// which is upstream's, and deliberate: an arbitrary but consistent answer
/// for malformed input keeps `<?`/`>?` transitive, which a "these are
/// incomparable" answer would not.
///
/// # Safety
///
/// `s1` and `s2` must point at `n1` and `n2` readable bytes.
pub unsafe fn utf_strnicmp(
    mut s1: *const c_char,
    mut s2: *const c_char,
    mut n1: size_t,
    mut n2: size_t,
) -> c_int {
    unsafe {
        // `utf_safe_read_char_adv` answers 0 at the end of its span and -1 for
        // a sequence it could not decode.
        let (c1, c2) = loop {
            let c1 = utf_safe_read_char_adv(&raw mut s1, &raw mut n1);
            let c2 = utf_safe_read_char_adv(&raw mut s2, &raw mut n2);
            if c1 <= 0 || c2 <= 0 {
                break (c1, c2);
            }
            if c1 == c2 {
                continue;
            }
            let cdiff = utf_fold(c1) - utf_fold(c2);
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
        let mut folded = [0 as c_char; MB_MAXCHAR];
        if c1 != -1 && c2 == -1 {
            n1 = utf_char2bytes(utf_fold(c1), folded.as_mut_ptr()) as size_t;
            s1 = folded.as_ptr();
        } else if c2 != -1 && c1 == -1 {
            n2 = utf_char2bytes(utf_fold(c2), folded.as_mut_ptr()) as size_t;
            s2 = folded.as_ptr();
        }

        while n1 > 0 && n2 > 0 && *s1 != NUL as c_char && *s2 != NUL as c_char {
            let cdiff = *s1 as u8 as c_int - *s2 as u8 as c_int;
            if cdiff != 0 {
                return cdiff;
            }
            s1 = s1.offset(1);
            s2 = s2.offset(1);
            n1 -= 1;
            n2 -= 1;
        }

        // A NUL ends a side early, however many bytes were allowed.
        if n1 > 0 && *s1 == NUL as c_char {
            n1 = 0;
        }
        if n2 > 0 && *s2 == NUL as c_char {
            n2 = 0;
        }
        match (n1 == 0, n2 == 0) {
            (true, true) => 0,
            (true, false) => -1,
            _ => 1,
        }
    }
}

/// [`utf_strnicmp`] with one length for both sides.
///
/// # Safety
///
/// Both strings must have `nn` readable bytes.
pub unsafe fn mb_strnicmp(s1: *const c_char, s2: *const c_char, nn: size_t) -> c_int {
    unsafe { utf_strnicmp(s1, s2, nn, nn) }
}

/// [`utf_strnicmp`] over two NUL-terminated strings.
///
/// # Safety
///
/// Both strings must be NUL-terminated.
pub unsafe fn mb_stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe { mb_strnicmp(s1, s2, MAXCOL as size_t) }
}

/// `strcmp` or [`mb_stricmp`], whichever `ic` asks for.
///
/// # Safety
///
/// Both strings must be NUL-terminated.
pub unsafe fn mb_strcmp_ic(ic: bool, s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe {
        if ic {
            mb_stricmp(s1, s2)
        } else {
            strcmp(s1, s2)
        }
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
            assert_eq!(utf_fold(c), want, "{c:#x}");
        }
        assert_eq!(utf_fold(0xdf), 0xdf); // ß, held back
        assert_eq!(utf_fold(0x130), 0x130); // İ, held back
        assert_eq!(utf_fold(0xc4), 0xe4); // Ä folds to ä
        assert_eq!(utf_fold(0x391), 0x3b1); // Α folds to α
    }
}
