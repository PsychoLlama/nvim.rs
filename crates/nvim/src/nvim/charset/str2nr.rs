//! The scan behind [`vim_str2nr`](super::vim_str2nr): which base a literal
//! is in, where its digits end, and what they add up to.
//!
//! All of it is safe code. The cursor it walks ([`Scan`](super::Scan)) is a
//! raw pointer, but its reads are checked once, where the cursor is built,
//! so the algorithm itself never touches one — which is why this file can
//! forbid unsafe outright while carrying the whole of `vim_str2nr` bar its
//! out-arguments.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::{
    STR2NR_BIN, STR2NR_DEC, STR2NR_FORCE, STR2NR_HEX, STR2NR_OCT, STR2NR_OOCT, STR2NR_QUOTE, Scan,
    is_bdigit, is_digit, is_odigit, is_xdigit,
};
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

    /// The two spellings of this base's `0x`-style prefix letter.
    pub fn prefix(self) -> Option<(u8, u8)> {
        match self {
            Radix::Binary => Some((b'b', b'B')),
            Radix::Octal => Some((b'o', b'O')),
            Radix::Hexadecimal => Some((b'x', b'X')),
            Radix::Decimal => None,
        }
    }

    /// Whether `byte` may separate two digits of this base. The C tests the
    /// *decimal* digit classes here for every base except binary, which
    /// tests only `0`/`1`. Preserved.
    fn separator_digit(self, byte: u8) -> bool {
        match self {
            Radix::Binary => is_bdigit(byte),
            Radix::Octal => is_odigit(byte),
            Radix::Decimal => is_digit(byte),
            Radix::Hexadecimal => is_xdigit(byte),
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
/// and `QUOTE` masked off. `None` is the C's `abort()` arm.
fn forced_radix(what_without_force: c_int) -> Option<Radix> {
    match what_without_force {
        STR2NR_DEC => Some(Radix::Decimal),
        STR2NR_HEX => Some(Radix::Hexadecimal),
        STR2NR_BIN => Some(Radix::Binary),
        // Both octal spellings are forced the same way.
        STR2NR_OCT | STR2NR_OOCT | 10 => Some(Radix::Octal),
        _ => None,
    }
}

/// What [`scan`] found: the prefix letter `vim_str2nr` reports through its
/// `prep` out-argument, the magnitude the digits add up to, and whether
/// accumulating them saturated.
pub(super) struct Scanned {
    pub pre: c_int,
    pub magnitude: uvarnumber_T,
    pub overflowed: bool,
}

/// Decide the base of the number at the cursor and read its digits, leaving
/// the cursor on the first byte that is not part of it.
///
/// `None` means `what` forces a base its remaining flags do not name, which
/// upstream answers with `abort()`.
pub(super) fn scan(scan: &mut Scan, what: c_int) -> Option<Scanned> {
    // `pre` is the prefix letter the caller is told about: `0`, `b`, `B`,
    // `o`, `O`, `x` or `X`, and zero for a plain decimal number or a forced
    // base.
    let mut pre: c_int = 0;
    let radix = if what & STR2NR_FORCE != 0 {
        // When forcing, the only question is whether there is a prefix to
        // skip; decimal has none.
        let radix = forced_radix(what & !(STR2NR_FORCE | STR2NR_QUOTE))?;
        if let Some((lower, upper)) = radix.prefix()
            && scan.within(2)
            && scan.at(0) == b'0'
            && (scan.at(1) == lower || scan.at(1) == upper)
            && radix.digit(scan.at(2)).is_some()
        {
            scan.advance(2);
        }
        radix
    } else if what & (STR2NR_HEX | STR2NR_OCT | STR2NR_OOCT | STR2NR_BIN) != 0
        && scan.within(1)
        && scan.at(0) == b'0'
        && scan.at(1) != b'8'
        && scan.at(1) != b'9'
    {
        pre = scan.at(1) as c_int;
        let prefixed = [
            (STR2NR_HEX, Radix::Hexadecimal, b'x', b'X'),
            (STR2NR_BIN, Radix::Binary, b'b', b'B'),
            (STR2NR_OOCT, Radix::Octal, b'o', b'O'),
        ]
        .into_iter()
        .find(|&(flag, base, lower, upper)| {
            what & flag != 0
                && scan.within(2)
                && (pre == upper as c_int || pre == lower as c_int)
                && base.digit(scan.at(2)).is_some()
        });
        if let Some((_, base, _, _)) = prefixed {
            scan.advance(2);
            base
        } else {
            pre = 0;
            // A leading zero means octal only if every digit that follows is
            // one; `0548` is decimal.
            let mut octal = what & STR2NR_OCT != 0 && is_odigit(scan.at(1));
            if octal {
                let mut i = 2;
                while scan.within(i) && is_digit(scan.at(i)) {
                    if scan.at(i) > b'7' {
                        octal = false;
                        break;
                    }
                    i += 1;
                }
            }
            if octal {
                pre = '0' as c_int;
                Radix::Octal
            } else {
                Radix::Decimal
            }
        }
    } else {
        Radix::Decimal
    };

    // Accumulate the digits. A quote is only a separator between digits, so
    // it never ends the number by itself.
    let after_prefix = scan.consumed();
    let mut magnitude: uvarnumber_T = 0;
    let mut overflowed = false;
    while scan.within(0) {
        if what & STR2NR_QUOTE != 0 && scan.consumed() > after_prefix && scan.at(0) == b'\'' {
            scan.advance(1);
            // The bounds test comes first, as it does in the C: without it
            // the byte after a trailing quote would be read past `maxlen`.
            if scan.within(0) && radix.separator_digit(scan.at(0)) {
                continue;
            }
            scan.advance(-1);
        }
        let Some(digit) = radix.digit(scan.at(0)) else {
            break;
        };
        let (next, saturated) = accumulate(magnitude, digit, radix);
        magnitude = next;
        overflowed |= saturated;
        scan.advance(1);
    }

    Some(Scanned {
        pre,
        magnitude,
        overflowed,
    })
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
