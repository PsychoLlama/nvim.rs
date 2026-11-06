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

/// The UTF-16 surrogate range, which JSON's `\u` escapes use to spell a code
/// point above U+FFFF as two escapes.
const SURROGATE_HI_START: UVarNumber = 0xd800;
const SURROGATE_HI_END: UVarNumber = 0xdbff;
const SURROGATE_LO_START: UVarNumber = 0xdc00;
const SURROGATE_LO_END: UVarNumber = 0xdfff;
const SURROGATE_FIRST_CHAR: c_int = 0x10000;

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
        if ch >= 0x80 && c_int::from(byte) == ch && !(ch == 0xc3 && p + 1 < e && buf[p + 1] == 0x83)
        {
            return fail(at, p, DecodeError::OnlyUtf8(dec.rest(p)));
        }
        if ch > 0x10ffff {
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
        let n = encode_char(ch, &mut encoded);
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
            if (SURROGATE_HI_START..=SURROGATE_HI_END).contains(&ch) {
                flush(&mut out, &mut fst_in_pair);
                fst_in_pair = code;
            } else if (SURROGATE_LO_START..=SURROGATE_LO_END).contains(&ch) && fst_in_pair != 0 {
                let low = c_int::try_from(ch - SURROGATE_LO_START).expect("ten bits");
                let high = fst_in_pair - c_int::try_from(SURROGATE_HI_START).expect("a code unit");
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
    let e = buf.len();
    let s = *at;
    let mut p = s;

    let mut fracs: Option<usize> = None;
    let mut exps: Option<usize> = None;
    let mut exps_s: Option<usize> = None;

    if buf[p] == b'-' {
        p += 1;
    }
    let ints = p;
    // Everything below is "scan as far as the grammar allows"; running
    // out of input is not an error here, it is what the checks below are
    // for.
    'scan: {
        if p >= e {
            break 'scan;
        }
        while p < e && ascii_isdigit(c_int::from(buf[p])) {
            p += 1;
        }
        if p != ints + 1 && buf[ints] == b'0' {
            return fail(at, p, DecodeError::LeadingZeroes(dec.rest(s)));
        }
        if p >= e || p == ints {
            break 'scan;
        }
        if buf[p] == b'.' {
            p += 1;
            fracs = Some(p);
            while p < e && ascii_isdigit(c_int::from(buf[p])) {
                p += 1;
            }
            if p >= e || Some(p) == fracs {
                break 'scan;
            }
        }
        if buf[p] == b'e' || buf[p] == b'E' {
            p += 1;
            exps_s = Some(p);
            if p < e && (buf[p] == b'-' || buf[p] == b'+') {
                p += 1;
            }
            exps = Some(p);
            while p < e && ascii_isdigit(c_int::from(buf[p])) {
                p += 1;
            }
        }
    }

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

/// The `parse_json_*_fail` tail: publish the cursor and answer failure.
///
/// Both scanners write `*pp = p` on the way out whichever exit they take, so
/// that the caller's own error reporting sees where the scan stopped.
fn fail(at: &mut usize, p: usize, refusal: DecodeError) -> Result<(), DecodeError> {
    *at = p;
    Err(refusal)
}
