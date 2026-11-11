//! The two JSON values with a syntax of their own: strings and numbers.
//!
//! Both start at the cursor, scan forward, answer their own refusals and hand
//! the finished value to [`Decoder::finish_value`] themselves — so like the
//! scanner that calls them they may come back having *rewound* the cursor,
//! with [`Decoder::next_map_special`] set.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::charset::{Str2NrBases, str2nr_in, string2float_in};
use crate::eval::encode::{
    SURROGATE_FIRST_CHAR, SURROGATE_HI_END, SURROGATE_HI_START, SURROGATE_LO_END,
    SURROGATE_LO_START,
};
use crate::mbyte::Codepoint;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use core::ffi::c_int;

use super::error::DecodeError;
use super::stack::Decoder;
use super::{BS, CAR, FF, NL, TAB};
use crate::ascii::{ascii_isdigit, ascii_isxdigit};
use crate::eval::decode::decode_owned_string;
use crate::mbyte::{char_at, char_len, encode_char, utf_char2len};
use crate::types::{TypVal, UVarNumber, VAR_STRING};

/// Scan a double-quoted JSON string and store it.
///
/// `at` points at the opening `"` and comes back on the closing one — or
/// rewound, if storing the value restarted the container.  The string is
/// measured in one pass and decoded in a second, so the output buffer is
/// allocated exactly once and handed to the typval, which then owns it.
///
/// `at` indexes `dec.buf` and points at a `"`.
pub(crate) fn parse_json_string(dec: &mut Decoder, at: &mut usize) -> Result<(), DecodeError> {
    let buf = dec.buf;
    let e = buf.len();
    let s = *at + 1;

    // Pass one: validate, and total up what the decoded bytes will need.
    // A `\u` escape is charged three bytes, which is the most one below
    // U+10000 can take — a surrogate pair spends both escapes' budget on
    // the four bytes the pair really needs.
    let mut p = s;
    let mut len: usize = 0;
    while p < e && buf[p] != b'"' {
        if buf[p] == b'\\' {
            p += 1;
            if p == e {
                return fail(at, p, DecodeError::UnfinishedEscape(dec.rest(0)));
            }
            match buf[p] {
                b'u' => {
                    if p + 4 >= e {
                        return fail(at, p, DecodeError::UnfinishedUnicode(dec.rest(0)));
                    }
                    if !buf[p + 1..p + 5]
                        .iter()
                        .all(|&c| ascii_isxdigit(c_int::from(c)))
                    {
                        return fail(at, p, DecodeError::FourHexDigits(dec.rest(p - 1)));
                    }
                    len += 3;
                    p += 5;
                }
                b'\\' | b'/' | b'"' | b't' | b'b' | b'n' | b'r' | b'f' => {
                    len += 1;
                    p += 1;
                }
                _ => {
                    return fail(at, p, DecodeError::UnknownEscape(dec.rest(p - 1)));
                }
            }
            continue;
        }

        // unescaped = %x20-21 / %x23-5B / %x5D-10FFFF
        let byte = buf[p];
        if byte < 0x20 {
            return fail(at, p, DecodeError::ControlChars(dec.rest(p)));
        }
        let ch = char_at(&buf[p..]);
        // Every code point above U+007F is two or more bytes, so it can
        // never equal the byte it starts with — except that `char_at` answers
        // a byte that starts no complete sequence as itself.  U+00C3 is the
        // one real exception, spelled 0xC3 0x83.
        if ch >= 0x80
            && c_int::from(byte) == ch.get()
            && !(ch == 0xc3 && p + 1 < e && buf[p + 1] == 0x83)
        {
            return fail(at, p, DecodeError::OnlyUtf8(dec.rest(p)));
        }
        if ch.get() > 0x10ffff {
            return fail(at, p, DecodeError::Above10ffff(dec.rest(p)));
        }
        let ch_len = usize::try_from(utf_char2len(ch)).unwrap_or(1);
        debug_assert!(ch_len == if ch != 0 { char_len(&buf[p..]) } else { 1 });
        len += ch_len;
        p += ch_len;
    }
    // Upstream reaches `p > e` when `utf_ptr2char` read a lead byte whose
    // continuation bytes ran past the document; the slice form answers such
    // a byte as itself, which pass one refuses above.
    if p >= e || buf[p] != b'"' {
        return fail(at, p, DecodeError::StringEnd(dec.rest(0)));
    }

    // Pass two: write the decoded bytes, which the typval takes over -- with
    // room for the string's NUL.
    let mut out: Vec<u8> = Vec::with_capacity(len + 1);
    // A `\uD800`-range escape is held back until the next escape says
    // whether it is the first half of a surrogate pair.
    let mut fst_in_pair: c_int = 0;
    let put_char = |out: &mut Vec<u8>, ch: c_int| {
        let mut encoded = [0u8; 6];
        let n = encode_char(Codepoint::new(ch), &mut encoded);
        out.extend_from_slice(&encoded[..n]);
    };
    // `PUT_FST_IN_PAIR`: emit a held-back high surrogate as the lone code
    // point it turned out to be.
    let flush = |out: &mut Vec<u8>, fst: &mut c_int| {
        if *fst != 0 {
            put_char(out, *fst);
            *fst = 0;
        }
    };

    let mut t = s;
    while t < p {
        if buf[t] != b'\\' || buf[t + 1] != b'u' {
            flush(&mut out, &mut fst_in_pair);
        }
        if buf[t] != b'\\' {
            out.push(buf[t]);
            t += 1;
            continue;
        }
        t += 1;
        if buf[t] == b'u' {
            let what = Str2NrBases::HEX | Str2NrBases::FORCE;
            let ch: UVarNumber = str2nr_in(&buf[t + 1..t + 5], what, true).magnitude;
            t += 4;
            let code = c_int::try_from(ch).expect("four hex digits");
            if (SURROGATE_HI_START..=SURROGATE_HI_END).contains(&code) {
                flush(&mut out, &mut fst_in_pair);
                fst_in_pair = code;
            } else if (SURROGATE_LO_START..=SURROGATE_LO_END).contains(&code) && fst_in_pair != 0 {
                let low = code - SURROGATE_LO_START;
                let high = fst_in_pair - SURROGATE_HI_START;
                put_char(&mut out, low + (high << 10) + SURROGATE_FIRST_CHAR);
                fst_in_pair = 0;
            } else {
                flush(&mut out, &mut fst_in_pair);
                put_char(&mut out, code);
            }
        } else {
            out.push(match buf[t] {
                b'\\' => b'\\',
                b'/' => b'/',
                b'"' => b'"',
                b't' => TAB,
                b'b' => BS,
                b'n' => NL,
                b'r' => CAR,
                b'f' => FF,
                _ => unreachable!("pass one accepted no other escape"),
            });
        }
        t += 1;
    }
    flush(&mut out, &mut fst_in_pair);

    let obj = decode_owned_string(out);
    // A string carrying an embedded NUL came back as a blob wrapped in a
    // special dictionary, which can be a dictionary value but not a key.
    let is_special_string = obj.v_type() != VAR_STRING;
    let value = dec.value(obj, is_special_string);
    let stored = dec.finish_value(value, &mut p);
    *at = p;
    stored
}

/// Scan a JSON number — `-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?` — and store it.
///
/// `at` points at the leading digit or minus sign and comes back on the last
/// character of the number, so that the scanner's own `+= 1` lands past it.
///
/// `at` indexes `dec.buf` and points at `-` or a digit.
pub(crate) fn parse_json_number(dec: &mut Decoder, at: &mut usize) -> Result<(), DecodeError> {
    let buf = dec.buf;
    let s = *at;
    let mut p = s;

    if buf[p] == b'-' {
        p += 1;
    }
    let ints = p;
    let NumberExtent {
        end,
        fracs,
        exps_s,
        exps,
    } = match number_extent(buf, ints) {
        Ok(extent) => extent,
        Err(stop) => return fail(at, stop, DecodeError::LeadingZeroes(dec.rest(s))),
    };
    p = end;

    if p == ints {
        return fail(at, p, DecodeError::AfterMinus(dec.rest(s)));
    }
    if Some(p) == fracs || (fracs.is_some() && exps_s == fracs.map(|f| f + 1)) {
        return fail(at, p, DecodeError::AfterDot(dec.rest(s)));
    }
    if Some(p) == exps {
        return fail(at, p, DecodeError::MissingExponent(dec.rest(s)));
    }

    let text = &buf[s..p];
    let want = text.len();
    let mut tv = TypVal::Number(0);
    if fracs.is_some() || exps.is_some() {
        let (parsed, got) = string2float_in(text);
        tv.write_float(parsed);
        if want != got {
            let shown = msg_bytes(text);
            semsg!(
                "E685: internal error: while converting number \"{shown}\" to float \
                 string2float consumed {got} bytes in place of {want}"
            );
        }
    } else {
        let parsed = str2nr_in(text, Str2NrBases::NONE, true);
        if want != parsed.len {
            let (shown, got) = (msg_bytes(text), parsed.len);
            semsg!(
                "E685: internal error: while converting number \"{shown}\" to integer \
                 vim_str2nr consumed {got} bytes in place of {want}"
            );
        }
        tv.write_number(parsed.value);
    }

    let value = dec.value(tv, false);
    if let Err(refusal) = dec.finish_value(value, &mut p) {
        *at = p;
        return Err(refusal);
    }
    if !dec.next_map_special {
        // The scanner's loop advances past the last character itself.
        p -= 1;
    }
    *at = p;
    Ok(())
}

/// Where a number's parts start and end, as far as the grammar allows.
#[derive(Debug, PartialEq, Eq)]
struct NumberExtent {
    /// One past the last byte of the number.
    end: usize,
    /// The first digit after the `.`, if there is one.
    fracs: Option<usize>,
    /// The byte after the `e`/`E`, where a sign may sit, if there is one.
    exps_s: Option<usize>,
    /// The first exponent digit's place, past any sign.
    exps: Option<usize>,
}

/// Scan a number's integer, fraction and exponent from `ints`, the byte past
/// any minus sign, as far as the grammar allows.
///
/// Running out of input is not an error here: the caller's checks read the
/// extent and say what is missing. The one refusal made while scanning is a
/// leading zero, answered as `Err` with the place the integer part ended.
fn number_extent(buf: &[u8], ints: usize) -> Result<NumberExtent, usize> {
    let e = buf.len();
    let mut extent = NumberExtent {
        end: ints,
        fracs: None,
        exps_s: None,
        exps: None,
    };
    let mut p = ints;
    if p >= e {
        return Ok(extent);
    }
    while p < e && ascii_isdigit(c_int::from(buf[p])) {
        p += 1;
    }
    if p != ints + 1 && buf[ints] == b'0' {
        return Err(p);
    }
    extent.end = p;
    if p >= e || p == ints {
        return Ok(extent);
    }
    if buf[p] == b'.' {
        p += 1;
        extent.fracs = Some(p);
        while p < e && ascii_isdigit(c_int::from(buf[p])) {
            p += 1;
        }
        extent.end = p;
        if p >= e || Some(p) == extent.fracs {
            return Ok(extent);
        }
    }
    if buf[p] == b'e' || buf[p] == b'E' {
        p += 1;
        extent.exps_s = Some(p);
        if p < e && (buf[p] == b'-' || buf[p] == b'+') {
            p += 1;
        }
        extent.exps = Some(p);
        while p < e && ascii_isdigit(c_int::from(buf[p])) {
            p += 1;
        }
        extent.end = p;
    }
    Ok(extent)
}

/// The `parse_json_*_fail` tail: publish the cursor and answer failure.
///
/// Both scanners write `*pp = p` on the way out whichever exit they take, so
/// that the caller's own error reporting sees where the scan stopped.
fn fail(at: &mut usize, p: usize, refusal: DecodeError) -> Result<(), DecodeError> {
    *at = p;
    Err(refusal)
}

#[cfg(test)]
mod tests {
    use super::{NumberExtent, number_extent};

    fn extent(
        end: usize,
        fracs: Option<usize>,
        exps_s: Option<usize>,
        exps: Option<usize>,
    ) -> NumberExtent {
        NumberExtent {
            end,
            fracs,
            exps_s,
            exps,
        }
    }

    #[test]
    fn number_extent_reads_each_part_and_stops_at_the_first_byte_past_it() {
        assert_eq!(number_extent(b"42,", 0), Ok(extent(2, None, None, None)));
        assert_eq!(
            number_extent(b"-1.5]", 1),
            Ok(extent(4, Some(3), None, None))
        );
        assert_eq!(
            number_extent(b"1.5e-3", 0),
            Ok(extent(6, Some(2), Some(4), Some(5)))
        );
        assert_eq!(
            number_extent(b"2E9", 0),
            Ok(extent(3, None, Some(2), Some(2)))
        );
        // A missing part is the caller's to refuse: the extent shows it.
        assert_eq!(number_extent(b"-", 1), Ok(extent(1, None, None, None)));
        assert_eq!(number_extent(b"1.", 0), Ok(extent(2, Some(2), None, None)));
        assert_eq!(
            number_extent(b"1e", 0),
            Ok(extent(2, None, Some(2), Some(2)))
        );
    }

    #[test]
    fn number_extent_refuses_a_leading_zero_where_the_integer_ends() {
        assert_eq!(number_extent(b"012", 0), Err(3));
        assert_eq!(number_extent(b"-00", 1), Err(3));
        assert_eq!(number_extent(b"0.5", 0), Ok(extent(3, Some(2), None, None)));
    }
}
