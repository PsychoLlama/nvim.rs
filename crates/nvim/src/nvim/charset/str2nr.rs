//! The radix arithmetic behind [`vim_str2nr`](super::vim_str2nr).
//!
//! Only the decisions are here; the scan itself stays in `charset.rs`,
//! because it walks a raw pointer lazily and must stop at the first byte
//! that is not a digit rather than measuring the string first.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use crate::src::nvim::types::{uvarnumber_T, varnumber_T};

const VARNUMBER_MAX: uvarnumber_T = 9223372036854775807;
const VARNUMBER_MIN: varnumber_T = -9223372036854775808;

/// A base `vim_str2nr` can parse in, together with the prefix letter it
/// reports through its `prep` out-argument.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Radix {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

impl Radix {
    /// The multiplier per digit.
    pub fn base(self) -> uvarnumber_T {
        match self {
            Radix::Binary => 2,
            Radix::Octal => 8,
            Radix::Decimal => 10,
            Radix::Hexadecimal => 16,
        }
    }

    /// The value of `byte` as a digit in this radix, if it is one.
    pub fn digit(self, byte: u8) -> Option<uvarnumber_T> {
        let value = match byte {
            b'0'..=b'9' => uvarnumber_T::from(byte - b'0'),
            b'a'..=b'f' => uvarnumber_T::from(byte - b'a') + 10,
            b'A'..=b'F' => uvarnumber_T::from(byte - b'A') + 10,
            _ => return None,
        };
        (value < self.base()).then_some(value)
    }
}

/// Append `digit` to `accumulated`.
///
/// Answers the new value and whether it saturated. Note the bound the C
/// uses: past the first `/ base` comparison it only rejects the final digit
/// when the base is ten, so an over-long hex, octal or binary literal
/// saturates one digit later than a decimal one would. Preserved, because
/// the saturated value is `UVARNUMBER_MAX` either way and `overflow` is
/// reported in both.
pub fn accumulate(
    accumulated: uvarnumber_T,
    digit: uvarnumber_T,
    radix: Radix,
) -> (uvarnumber_T, bool) {
    let base = radix.base();
    let fits = accumulated < uvarnumber_T::MAX / base
        || (accumulated == uvarnumber_T::MAX / base
            && (radix != Radix::Decimal || digit <= uvarnumber_T::MAX % 10));
    if fits {
        (base * accumulated + digit, false)
    } else {
        (uvarnumber_T::MAX, true)
    }
}

/// The signed value `vim_str2nr` reports for an unsigned magnitude, and
/// whether it had to be clamped.
///
/// A negative magnitude past `VARNUMBER_MAX` clamps to `VARNUMBER_MIN`; a
/// positive one clamps to `VARNUMBER_MAX`. Note that `-VARNUMBER_MIN` itself
/// is over the bound and therefore clamps rather than round-tripping.
pub fn signed(magnitude: uvarnumber_T, negative: bool) -> (varnumber_T, bool) {
    if magnitude > VARNUMBER_MAX {
        if negative {
            (VARNUMBER_MIN, true)
        } else {
            (VARNUMBER_MAX as varnumber_T, true)
        }
    } else if negative {
        (-(magnitude as varnumber_T), false)
    } else {
        (magnitude as varnumber_T, false)
    }
}

/// Whether a strict parse must reject what follows the number: a letter or
/// digit means the caller was handed something that is not a number at all.
pub fn strict_reject(next: u8) -> bool {
    next.is_ascii_alphanumeric()
}

/// The radix a `STR2NR_FORCE` request names, given the flags with `FORCE`
/// and `QUOTE` masked off. `None` means plain decimal.
pub fn forced_radix(what_without_force: c_int) -> Option<Radix> {
    match what_without_force {
        super::STR2NR_HEX => Some(Radix::Hexadecimal),
        super::STR2NR_BIN => Some(Radix::Binary),
        // Both octal spellings are forced the same way.
        super::STR2NR_OCT | super::STR2NR_OOCT | 10 => Some(Radix::Octal),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_are_bounded_by_the_radix() {
        assert_eq!(Radix::Binary.digit(b'1'), Some(1));
        assert_eq!(Radix::Binary.digit(b'2'), None);
        assert_eq!(Radix::Octal.digit(b'7'), Some(7));
        assert_eq!(Radix::Octal.digit(b'8'), None);
        assert_eq!(Radix::Decimal.digit(b'9'), Some(9));
        assert_eq!(Radix::Decimal.digit(b'a'), None);
        assert_eq!(Radix::Hexadecimal.digit(b'f'), Some(15));
        assert_eq!(Radix::Hexadecimal.digit(b'F'), Some(15));
        assert_eq!(Radix::Hexadecimal.digit(b'g'), None);
    }

    #[test]
    fn accumulation_saturates_at_the_unsigned_maximum() {
        assert_eq!(accumulate(0, 1, Radix::Decimal), (1, false));
        assert_eq!(accumulate(12, 3, Radix::Decimal), (123, false));
        assert_eq!(accumulate(0xf, 0xf, Radix::Hexadecimal), (0xff, false));

        // u64::MAX is 18446744073709551615: the last digit that fits is 5.
        let near = uvarnumber_T::MAX / 10;
        assert_eq!(
            accumulate(near, 5, Radix::Decimal),
            (uvarnumber_T::MAX, false)
        );
        assert_eq!(
            accumulate(near, 6, Radix::Decimal),
            (uvarnumber_T::MAX, true)
        );
        assert_eq!(
            accumulate(near + 1, 0, Radix::Decimal),
            (uvarnumber_T::MAX, true)
        );
    }

    #[test]
    fn only_decimal_checks_the_last_digit() {
        // The C's macro leaves the `% 10` test in place for every base, so a
        // hex literal at exactly MAX/16 accepts any digit and wraps the top
        // bits away rather than saturating. Pinned as it stands.
        let near = uvarnumber_T::MAX / 16;
        assert_eq!(
            accumulate(near, 15, Radix::Hexadecimal),
            (uvarnumber_T::MAX, false)
        );
    }

    #[test]
    fn signed_conversion_clamps_at_both_ends() {
        assert_eq!(signed(42, false), (42, false));
        assert_eq!(signed(42, true), (-42, false));
        assert_eq!(
            signed(VARNUMBER_MAX, false),
            (VARNUMBER_MAX as varnumber_T, false)
        );
        assert_eq!(
            signed(VARNUMBER_MAX + 1, false),
            (VARNUMBER_MAX as varnumber_T, true)
        );
        // -(2^63) is representable, but the magnitude is not, so it clamps.
        assert_eq!(signed(VARNUMBER_MAX + 1, true), (VARNUMBER_MIN, true));
    }

    #[test]
    fn a_strict_parse_only_rejects_alphanumerics() {
        assert!(strict_reject(b'x'));
        assert!(strict_reject(b'9'));
        assert!(!strict_reject(b'\''));
        assert!(!strict_reject(b'-'));
        assert!(!strict_reject(0));
    }
}
