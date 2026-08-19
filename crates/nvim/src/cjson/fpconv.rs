//! `fpconv.c`: the two floating-point conversions Lua CJSON needs, in safe
//! Rust.
//!
//! Upstream exists to work around a *locale*: C's `strtod` and `printf` read
//! and write whatever `LC_NUMERIC` says the decimal separator is, so
//! upstream copies the number into a scratch buffer, swaps `.` for the
//! locale's character, calls libc, and swaps back. Rust's float formatting
//! and parsing are not locale-sensitive, so that whole layer — the probe
//! that `%g` of 0.5 prints as `0.5`, the `locale_decimal_point` static, the
//! translation buffers and `fpconv_init` — has no job here and is gone.
//! (nvim forces `LC_NUMERIC` back to `"C"` at startup and after every
//! `:language` anyway, so it was already dead in practice; see
//! `os/lang.rs`.)
//!
//! What remains is the observable behaviour, and it is a **byte contract**:
//! `vim.json.encode(0.1)` must answer `0.1`, `encode(1e16)` must answer
//! `1e+16`, and `decode("0x10")` must answer 16. [`append_g_fmt`] is C's
//! `%.*g` and [`strtod`] is C's `strtod`, both reimplemented here rather
//! than delegated, because `format!("{}")` and `str::parse` are neither one
//! of them. `decodediff` section 3 is the gate; `tests/unit/fpconv.rs`
//! cross-checks both against libc over a randomised sample.
//!
//! Ported from Lua CJSON's `fpconv.c`, Copyright (c) 2011-2012 Mark Pulford,
//! under the MIT license; the notice is reproduced in
//! licenses/lua-cjson-LICENSE.txt.

#![forbid(unsafe_code)]

use core::fmt::Write as _;

/// The buffer size upstream's callers reserve per conversion. Kept because
/// `%.16g` of an f64 is at most 23 bytes and the encoder still reserves a
/// fixed slab per number.
pub const G_FMT_BUFSIZE: usize = 32;

/// A stack buffer `core::fmt` can write into, so the scientific rendering
/// [`append_g_fmt`] reshapes costs no allocation.
struct Scratch {
    buf: [u8; G_FMT_BUFSIZE],
    len: usize,
}

impl core::fmt::Write for Scratch {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let end = self.len + s.len();
        let room = self.buf.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        room.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// Append `num` to `out` exactly as C's `printf("%.*g", precision, num)`
/// would.
///
/// `%g` picks between `%e` and `%f` by the *rounded* value's decimal
/// exponent X: fixed notation when `-4 <= X < precision`, scientific
/// otherwise, and in both cases trailing fractional zeros are dropped. So
/// the rounding is done once, up front, by asking Rust for the scientific
/// form with `precision - 1` fractional digits — which is correctly rounded
/// and therefore agrees with glibc digit for digit — and the rest of this
/// function only moves the decimal point around.
pub fn append_g_fmt(out: &mut Vec<u8>, num: f64, precision: u32) {
    // Upstream asserts 1..=16 here. The encoder's only caller passes 16 and
    // nvim removed the setter that could change it, so clamping is the
    // honest reading of "must not be out of range" — and 17 digits is where
    // an f64 stops gaining information.
    let precision = precision.clamp(1, 17) as usize;

    if num.is_sign_negative() {
        out.push(b'-');
    }
    let num = num.abs();
    if num.is_nan() {
        // Unreachable from the encoder (`encode_invalid_numbers` is 0, so
        // NaN and infinity raise before they get here), but this is what
        // glibc prints and the contract is what glibc prints.
        out.extend_from_slice(b"nan");
        return;
    }
    if num.is_infinite() {
        out.extend_from_slice(b"inf");
        return;
    }
    if num == 0.0 {
        out.push(b'0');
        return;
    }

    // `{:.*e}` renders as `d.ddd…e<exp>` (or `de<exp>` at precision 1), with
    // Rust's bare exponent rather than C's signed two-digit one.
    let mut sci = Scratch {
        buf: [0; G_FMT_BUFSIZE],
        len: 0,
    };
    let _ = write!(sci, "{:.*e}", precision - 1, num);
    let rendered = &sci.buf[..sci.len];
    let at_e = rendered.iter().position(|&b| b == b'e').unwrap_or(0);

    // The significand's digits, with the point taken out: exactly
    // `precision` of them, the first of them non-zero.
    let mut digits = [b'0'; 18];
    let mut count = 0;
    for &b in &rendered[..at_e] {
        if b != b'.' {
            digits[count] = b;
            count += 1;
        }
    }
    let digits = &digits[..count];

    let mut exponent = 0i32;
    let mut negative_exponent = false;
    for &b in &rendered[at_e + 1..] {
        match b {
            b'-' => negative_exponent = true,
            b'+' => {}
            _ => exponent = exponent * 10 + i32::from(b - b'0'),
        }
    }
    if negative_exponent {
        exponent = -exponent;
    }

    /// The digits worth printing: `%g` without `#` drops trailing zeros.
    fn trimmed(digits: &[u8]) -> &[u8] {
        let end = digits
            .iter()
            .rposition(|&b| b != b'0')
            .map_or(0, |last| last + 1);
        &digits[..end]
    }

    if exponent < -4 || exponent >= precision as i32 {
        out.push(digits[0]);
        let fraction = trimmed(&digits[1..]);
        if !fraction.is_empty() {
            out.push(b'.');
            out.extend_from_slice(fraction);
        }
        out.push(b'e');
        out.push(if exponent < 0 { b'-' } else { b'+' });
        let magnitude = exponent.unsigned_abs();
        if magnitude < 10 {
            out.push(b'0');
        }
        let mut tens = 1;
        while magnitude / tens >= 10 {
            tens *= 10;
        }
        while tens > 0 {
            out.push(b'0' + (magnitude / tens % 10) as u8);
            tens /= 10;
        }
    } else if exponent >= 0 {
        let whole = exponent as usize + 1;
        out.extend_from_slice(&digits[..whole]);
        let fraction = trimmed(&digits[whole..]);
        if !fraction.is_empty() {
            out.push(b'.');
            out.extend_from_slice(fraction);
        }
    } else {
        out.extend_from_slice(b"0.");
        out.extend(core::iter::repeat_n(b'0', (-exponent - 1) as usize));
        out.extend_from_slice(trimmed(digits));
    }
}

/// C's `strtod`: read the longest floating-point prefix of `s` and answer it
/// with the number of bytes consumed.
///
/// A consumed count of zero means "no conversion", which is how the decoder
/// tells a malformed number from a valid one — so the *length* matters as
/// much as the value. The full C grammar is here rather than just JSON's,
/// because the decoder deliberately leans on it: `decode_invalid_numbers` is
/// on by default, so `"0x10"` reaches this as a hex float and `"inf"` and
/// `"nan"` reach it as themselves.
pub fn strtod(s: &[u8]) -> (f64, usize) {
    let mut at = 0;
    while matches!(s.get(at), Some(b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)) {
        at += 1;
    }
    let negative = match s.get(at) {
        Some(b'-') => {
            at += 1;
            true
        }
        Some(b'+') => {
            at += 1;
            false
        }
        _ => false,
    };
    let body = &s[at..];

    let (magnitude, used) = if starts_with_ignoring_case(body, b"0x") {
        // `0x` with no hex digit after it is not a failed conversion — the
        // longest valid prefix is the leading `0`, and `x` is left for the
        // caller. That distinction is what tells the decoder `0x` is the
        // number 0 followed by junk rather than an invalid number.
        parse_hex(&body[2..]).map_or_else(|| parse_decimal(body), |(v, n)| (v, n + 2))
    } else if starts_with_ignoring_case(body, b"infinity") {
        (f64::INFINITY, 8)
    } else if starts_with_ignoring_case(body, b"inf") {
        (f64::INFINITY, 3)
    } else if starts_with_ignoring_case(body, b"nan") {
        // `nan(n-char-sequence)` is one token; the payload is discarded, and
        // it only counts when it closes and holds nothing but identifier
        // characters.
        let payload = body[3..].strip_prefix(b"(").map(|rest| {
            let end = rest
                .iter()
                .take_while(|b| b.is_ascii_alphanumeric() || **b == b'_')
                .count();
            (rest.get(end) == Some(&b')')).then_some(end + 2)
        });
        (f64::NAN, 3 + payload.flatten().unwrap_or(0))
    } else {
        parse_decimal(body)
    };

    if used == 0 {
        // No conversion: C leaves `endptr` at the *original* pointer, sign
        // and whitespace included.
        return (0.0, 0);
    }
    (if negative { -magnitude } else { magnitude }, at + used)
}

fn starts_with_ignoring_case(s: &[u8], prefix: &[u8]) -> bool {
    s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix)
}

/// `digits[.digits][(e|E)[sign]digits]`, handed to Rust's parser once its
/// extent is known. `f64::from_str` is correctly rounded over arbitrarily
/// many digits, which is the property that makes this equal to `strtod`.
fn parse_decimal(s: &[u8]) -> (f64, usize) {
    let whole = s.iter().take_while(|b| b.is_ascii_digit()).count();
    let mut at = whole;
    let mut fraction = 0;
    if s.get(at) == Some(&b'.') {
        fraction = s[at + 1..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        at += 1 + fraction;
    }
    if whole == 0 && fraction == 0 {
        return (0.0, 0);
    }
    let mantissa_end = at;

    // An exponent only counts when it is complete; `1e` is the number 1
    // followed by the letter e.
    if matches!(s.get(at), Some(b'e' | b'E')) {
        let mut after = at + 1;
        if matches!(s.get(after), Some(b'+' | b'-')) {
            after += 1;
        }
        let exponent = s[after..].iter().take_while(|b| b.is_ascii_digit()).count();
        if exponent > 0 {
            at = after + exponent;
        }
    }

    // `str::parse` rejects a trailing bare point (`1.`) and a leading one
    // (`.5` is fine, but `1.e5` is not), so hand it a normalised copy.
    let mut text = String::with_capacity(at + 2);
    text.push_str(core::str::from_utf8(&s[..whole]).unwrap_or("0"));
    if text.is_empty() {
        text.push('0');
    }
    text.push('.');
    if fraction > 0 {
        text.push_str(core::str::from_utf8(&s[whole + 1..mantissa_end]).unwrap_or("0"));
    } else {
        text.push('0');
    }
    if at > mantissa_end {
        text.push_str(core::str::from_utf8(&s[mantissa_end..at]).unwrap_or("e0"));
    }
    (text.parse::<f64>().unwrap_or(0.0), at)
}

/// `0x` hex floats: `hexdigits[.hexdigits][(p|P)[sign]decimaldigits]`.
///
/// The significand is exact in a `u128` up to 32 hex digits; anything past
/// that can only push the value up, so it is folded into a sticky bit and
/// the rounding below stays round-to-nearest-even. C makes the binary
/// exponent (`p…`) optional, unlike C++'s `hexfloat`.
fn parse_hex(s: &[u8]) -> Option<(f64, usize)> {
    let mut significand: u128 = 0;
    let mut sticky = false;
    let mut digits = 0;
    let mut at = 0;
    // Bit position of the value's units place, relative to `significand`.
    let mut scale: i32 = 0;

    /// Shift one hex digit in, or answer `false` once there is no room —
    /// at which point the digit can only contribute a sticky bit, 128 bits
    /// being more than twice what an f64 keeps.
    fn take(b: u8, significand: &mut u128, sticky: &mut bool) -> bool {
        let value = u128::from((b as char).to_digit(16).unwrap_or(0));
        if *significand > (u128::MAX >> 4) {
            *sticky |= value != 0;
            return false;
        }
        *significand = (*significand << 4) | value;
        true
    }

    while let Some(&b) = s.get(at) {
        if !b.is_ascii_hexdigit() {
            break;
        }
        if !take(b, &mut significand, &mut sticky) {
            scale += 4;
        }
        digits += 1;
        at += 1;
    }
    if s.get(at) == Some(&b'.') {
        at += 1;
        while let Some(&b) = s.get(at) {
            if !b.is_ascii_hexdigit() {
                break;
            }
            if take(b, &mut significand, &mut sticky) {
                scale -= 4;
            }
            digits += 1;
            at += 1;
        }
    }
    if digits == 0 {
        return None;
    }

    if matches!(s.get(at), Some(b'p' | b'P')) {
        let mut after = at + 1;
        let sign = match s.get(after) {
            Some(b'-') => {
                after += 1;
                -1
            }
            Some(b'+') => {
                after += 1;
                1
            }
            _ => 1,
        };
        let mut exponent: i32 = 0;
        let mut seen = 0;
        while let Some(&b) = s.get(after + seen) {
            if !b.is_ascii_digit() {
                break;
            }
            // Saturate: a wrapped exponent would answer a finite number
            // where C answers infinity.
            exponent = exponent
                .saturating_mul(10)
                .saturating_add(i32::from(b - b'0'));
            seen += 1;
        }
        if seen > 0 {
            at = after + seen;
            scale = scale.saturating_add(sign * exponent);
        }
    }

    Some((scale_to_f64(significand, sticky, scale), at))
}

/// `significand * 2^scale`, rounded to nearest with ties to even, where
/// `sticky` says a non-zero bit was dropped off the bottom of `significand`.
///
/// The rounding is done **here**, on the integer, rather than left to
/// `significand as f64` followed by a scaling multiply: when the answer is
/// subnormal those are two roundings of the same value, and double rounding
/// puts the last bit one off what `strtod` gives. Rounding once to the width
/// the answer is actually allowed leaves a multiply that is exact.
fn scale_to_f64(significand: u128, sticky: bool, scale: i32) -> f64 {
    if significand == 0 {
        return 0.0;
    }
    let width = 128 - significand.leading_zeros() as i32;
    // The value is `1.f * 2^exponent`.
    let exponent = scale.saturating_add(width - 1);
    if exponent > 1023 {
        return f64::INFINITY;
    }
    // A normal f64 keeps 53 bits; a subnormal one keeps fewer, and below
    // `2^-1075` there is not even half a bit left to round up from.
    let keep = if exponent >= -1022 {
        53
    } else {
        53 + exponent + 1022
    };
    if keep <= 0 {
        return 0.0;
    }

    let dropped = width - keep;
    let mut kept = if dropped <= 0 {
        significand << (-dropped) as u32
    } else {
        let shift = dropped as u32;
        let kept = significand >> shift;
        let remainder = significand & ((1u128 << shift) - 1);
        let half = 1u128 << (shift - 1);
        let round_up = remainder > half || (remainder == half && (sticky || kept & 1 == 1));
        kept + u128::from(round_up)
    };
    // Rounding up can carry into an extra bit (0x1.fff… -> 0x2), which is
    // representable at the same width, so nothing needs re-rounding — but
    // the value gained a factor of two the scaling below has to see.
    let mut power = scale + dropped;
    if kept >> keep != 0 {
        kept >>= 1;
        power += 1;
    }

    // `kept` is at most 53 bits, so this conversion is exact, and every
    // step below multiplies by a power of two: no further rounding until
    // the result overflows to infinity, which is the right answer when it
    // does.
    let mut value = kept as f64;
    while power > 0 {
        let step = power.min(500);
        value *= power_of_two(step);
        power -= step;
    }
    while power < 0 {
        let step = (-power).min(500);
        value /= power_of_two(step);
        power += step;
    }
    value
}

/// `2^exponent`, exactly, for an `exponent` a normal f64 can hold.
///
/// Built out of the bit pattern rather than `powi`, which is not required
/// to be exact and under Miri deliberately is not — `0x10` came back as
/// `16.000000000000007`, which is a real warning about relying on it.
fn power_of_two(exponent: i32) -> f64 {
    debug_assert!((-1022..=1023).contains(&exponent));
    f64::from_bits(((exponent + 1023) as u64) << 52)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(num: f64, precision: u32) -> String {
        let mut out = Vec::new();
        append_g_fmt(&mut out, num, precision);
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn the_encoder_contract_at_precision_16() {
        // Every one of these is a row of `decodediff` section 3b.
        for (num, text) in [
            (0.0, "0"),
            (-0.0, "-0"),
            (1.0, "1"),
            (0.5, "0.5"),
            (0.1, "0.1"),
            (0.3, "0.3"),
            (1.0 / 3.0, "0.3333333333333333"),
            (2.0 / 3.0, "0.6666666666666666"),
            (1e-5, "1e-05"),
            (1e-10, "1e-10"),
            (1e-100, "1e-100"),
            (5e-324, "4.940656458412465e-324"),
            (2.2250738585072014e-308, "2.225073858507201e-308"),
            (1e10, "10000000000"),
            (1e15, "1000000000000000"),
            (1e16, "1e+16"),
            (1e100, "1e+100"),
            (1.7976931348623157e308, "1.797693134862316e+308"),
            (core::f64::consts::PI, "3.141592653589793"),
            (123_456_789.123_456_79, "123456789.1234568"),
            (12345678901234567.0, "1.234567890123457e+16"),
            (0.30000000000000004, "0.3"),
            (4.35, "4.35"),
            (100.0, "100"),
            (-1e-7, "-1e-07"),
            (9007199254740992.0, "9007199254740992"),
            (-9.223372036854776e18, "-9.223372036854776e+18"),
        ] {
            assert_eq!(g(num, 16), text, "%.16g of {num:e}");
        }
    }

    #[test]
    fn the_style_switch_is_the_rounded_exponent() {
        // 999999.9 rounds *up* into the next decade at precision 6, and the
        // style is chosen from the exponent it lands on, not the one it had.
        assert_eq!(g(999999.9, 6), "1e+06");
        assert_eq!(g(999999.4, 6), "999999");
        assert_eq!(g(0.0001, 6), "0.0001");
        assert_eq!(g(0.00001, 6), "1e-05");
        assert_eq!(g(1.0, 1), "1");
        assert_eq!(g(1.5, 1), "2");
    }

    #[test]
    fn non_finite_values_print_the_way_glibc_does() {
        assert_eq!(g(f64::INFINITY, 16), "inf");
        assert_eq!(g(f64::NEG_INFINITY, 16), "-inf");
        assert_eq!(g(f64::NAN, 16), "nan");
        assert_eq!(g(-f64::NAN, 16), "-nan");
    }

    #[test]
    fn strtod_reads_the_longest_valid_prefix() {
        assert_eq!(strtod(b"1"), (1.0, 1));
        assert_eq!(strtod(b"-1.5"), (-1.5, 4));
        assert_eq!(strtod(b".5"), (0.5, 2));
        assert_eq!(strtod(b"1."), (1.0, 2));
        assert_eq!(strtod(b"1.2.3"), (1.2, 3));
        assert_eq!(strtod(b"1e"), (1.0, 1));
        assert_eq!(strtod(b"1e+"), (1.0, 1));
        assert_eq!(strtod(b"1e5x"), (100000.0, 3));
        assert_eq!(strtod(b"  12"), (12.0, 4));
        assert_eq!(strtod(b"9x"), (9.0, 1));
        assert_eq!(strtod(b"0.1"), (0.1, 3));
    }

    #[test]
    fn no_conversion_consumes_nothing() {
        assert_eq!(strtod(b""), (0.0, 0));
        assert_eq!(strtod(b"-"), (0.0, 0));
        assert_eq!(strtod(b"x"), (0.0, 0));
        assert_eq!(strtod(b"."), (0.0, 0));
        assert_eq!(strtod(b".e5"), (0.0, 0));
        assert_eq!(strtod(b"  "), (0.0, 0));
        // `0x` alone *is* a conversion: the `0` converts and the `x` does
        // not, which is not the same as failing.
        assert_eq!(strtod(b"0x"), (0.0, 1));
    }

    #[test]
    fn the_decoder_leans_on_the_non_json_forms() {
        // `decode_invalid_numbers` is on, so all of these reach `strtod`.
        assert_eq!(strtod(b"0x10"), (16.0, 4));
        assert_eq!(strtod(b"0X1p4"), (16.0, 5));
        assert_eq!(strtod(b"0x1.8p1"), (3.0, 7));
        assert_eq!(strtod(b"0x.8"), (0.5, 4));
        assert_eq!(strtod(b"-0xffp-4"), (-15.9375, 8));
        assert_eq!(strtod(b"inf"), (f64::INFINITY, 3));
        assert_eq!(strtod(b"-Infinity"), (f64::NEG_INFINITY, 9));
        assert_eq!(strtod(b"INF"), (f64::INFINITY, 3));
        let (value, used) = strtod(b"nan");
        assert!(value.is_nan() && used == 3);
        let (value, used) = strtod(b"nan(123)");
        assert!(value.is_nan() && used == 8);
    }

    #[test]
    fn overflow_and_underflow_saturate_the_way_strtod_does() {
        assert_eq!(strtod(b"1e400"), (f64::INFINITY, 5));
        assert_eq!(strtod(b"-1e400"), (f64::NEG_INFINITY, 6));
        assert_eq!(strtod(b"1e-400"), (0.0, 6));
        assert_eq!(strtod(b"0x1p99999"), (f64::INFINITY, 9));
        assert_eq!(strtod(b"0x1p-99999"), (0.0, 10));
        // Past 2^53 the representable values step by two, so both of these
        // are exact ties and both round to the even neighbour — which is
        // *below* for one and *above* for the other.
        assert_eq!(strtod(b"0x20000000000001"), (9007199254740992.0, 16));
        assert_eq!(strtod(b"0x20000000000003"), (9007199254740996.0, 16));
        // The smallest subnormal, and half of it, which rounds to even = 0.
        assert_eq!(strtod(b"0x1p-1074"), (5e-324, 9));
        assert_eq!(strtod(b"0x1p-1075"), (0.0, 9));
        assert_eq!(strtod(b"0x1.8p-1074"), (1e-323, 11));
    }

    #[test]
    fn every_double_round_trips_through_the_encoder_at_17_digits() {
        for bits in [
            0x0000_0000_0000_0001u64,
            0x000f_ffff_ffff_ffff,
            0x3ff0_0000_0000_0001,
            0x4009_21fb_5444_2d18,
            0x7fef_ffff_ffff_ffff,
            0xc1d2_3456_789a_bcde,
        ] {
            let num = f64::from_bits(bits);
            let text = g(num, 17);
            assert_eq!(strtod(text.as_bytes()).0, num, "{text}");
        }
    }
}
