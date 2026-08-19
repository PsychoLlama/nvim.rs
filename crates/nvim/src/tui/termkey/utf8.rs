#![forbid(unsafe_code)]

//! UTF-8 as libtermkey reads and writes it.
//!
//! This is the editor's historical six-byte encoding (`utf_char2bytes`), not
//! the four-byte one the standard settled on, because a key's `utf8` field is
//! seven bytes and callers rely on being able to hold any of them.
//!
//! Ported from libtermkey, Copyright (c) 2007-2011 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libtermkey-LICENSE.txt.

use core::ffi::c_int;

/// U+FFFD, substituted for anything that does not decode.
pub const UNICODE_INVALID: c_int = 0xfffd;

/// The longest encoding this produces.
pub const UTF8_MAX_BYTES: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decoded {
    /// A codepoint, and how many bytes it took. `UNICODE_INVALID` here means
    /// the bytes were malformed and that many of them should be skipped.
    Char { codepoint: c_int, len: usize },
    /// A valid prefix: more bytes could still complete it.
    Incomplete,
}

/// How many bytes `encode` will produce for `codepoint`.
pub fn encoded_len(codepoint: c_int) -> usize {
    match codepoint {
        ..0x80 => 1,
        0x80..0x800 => 2,
        0x800..0x10000 => 3,
        0x10000..0x200000 => 4,
        0x200000..0x4000000 => 5,
        _ => 6,
    }
}

/// Encode a codepoint. Returns the buffer and how much of it is used.
pub fn encode(codepoint: c_int) -> ([u8; UTF8_MAX_BYTES], usize) {
    let mut out = [0u8; UTF8_MAX_BYTES];
    let len = encoded_len(codepoint);
    let value = codepoint as u32;
    // The lead byte's marker is the pattern for its length; every continuation
    // byte carries six more bits under 0b10.
    const LEAD: [u32; 7] = [0, 0x00, 0xc0, 0xe0, 0xf0, 0xf8, 0xfc];
    out[0] = LEAD[len].wrapping_add(value >> (6 * (len - 1))) as u8;
    for (i, byte) in out[1..len].iter_mut().enumerate() {
        *byte = 0x80u32.wrapping_add(value >> (6 * (len - 2 - i)) & 0x3f) as u8;
    }
    (out, len)
}

/// Decode the codepoint at the head of `bytes`, which must not be empty.
pub fn decode(bytes: &[u8]) -> Decoded {
    let invalid = |len| Decoded::Char {
        codepoint: UNICODE_INVALID,
        len,
    };
    let lead = bytes[0];
    let (len, mut codepoint) = match lead {
        ..0x80 => {
            return Decoded::Char {
                codepoint: lead as c_int,
                len: 1,
            };
        }
        // A continuation byte with nothing to continue.
        0x80..0xc0 => return invalid(1),
        0xc0..0xe0 => (2, (lead & 0x1f) as c_int),
        0xe0..0xf0 => (3, (lead & 0x0f) as c_int),
        0xf0..0xf8 => (4, (lead & 0x07) as c_int),
        0xf8..0xfc => (5, (lead & 0x03) as c_int),
        0xfc..0xfe => (6, (lead & 0x01) as c_int),
        _ => return invalid(1),
    };
    for (i, &byte) in bytes.iter().enumerate().take(len).skip(1) {
        if !(0x80..0xc0).contains(&byte) {
            // The sequence was cut short by something that is not a
            // continuation; the offending byte is left for the next read.
            return invalid(i);
        }
        codepoint = codepoint << 6 | (byte & 0x3f) as c_int;
    }
    if bytes.len() < len {
        return Decoded::Incomplete;
    }
    // An overlong encoding, a surrogate half or a non-character decodes, but is
    // not the codepoint it claims to be.
    if len > encoded_len(codepoint)
        || (0xd800..=0xdfff).contains(&codepoint)
        || codepoint == 0xfffe
        || codepoint == 0xffff
    {
        codepoint = UNICODE_INVALID;
    }
    Decoded::Char { codepoint, len }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decoded(bytes: &[u8]) -> Decoded {
        decode(bytes)
    }

    fn ch(codepoint: c_int, len: usize) -> Decoded {
        Decoded::Char { codepoint, len }
    }

    #[test]
    fn round_trips_every_encoding_length() {
        for codepoint in [0x41, 0xa0, 0x7ff, 0x800, 0xfffd, 0x10000, 0x10ffff] {
            let (buf, len) = encode(codepoint);
            assert_eq!(len, encoded_len(codepoint));
            assert_eq!(decoded(&buf[..len]), ch(codepoint, len), "U+{codepoint:X}");
        }
    }

    #[test]
    fn encodes_past_the_standard_range_in_five_and_six_bytes() {
        assert_eq!(encoded_len(0x200000), 5);
        assert_eq!(encoded_len(0x4000000), 6);
        let (buf, len) = encode(0x4000000);
        assert_eq!(len, 6);
        assert_eq!(buf, [0xfc, 0x84, 0x80, 0x80, 0x80, 0x80]);
    }

    #[test]
    fn a_truncated_sequence_asks_for_more() {
        assert_eq!(decoded(b"\xc2"), Decoded::Incomplete);
        assert_eq!(decoded(b"\xe0"), Decoded::Incomplete);
        assert_eq!(decoded(b"\xe0\xa0"), Decoded::Incomplete);
        assert_eq!(decoded(b"\xf0\x90\x80"), Decoded::Incomplete);
    }

    #[test]
    fn a_bad_continuation_stops_at_the_offending_byte() {
        // The '!' is left in the buffer for the next read.
        assert_eq!(decoded(b"\xc2!"), ch(UNICODE_INVALID, 1));
        assert_eq!(decoded(b"\xe0\xa0!"), ch(UNICODE_INVALID, 2));
        assert_eq!(decoded(b"\xf0\x90\x80!"), ch(UNICODE_INVALID, 3));
    }

    #[test]
    fn a_stray_continuation_or_invalid_lead_takes_one_byte() {
        assert_eq!(decoded(b"\x80"), ch(UNICODE_INVALID, 1));
        assert_eq!(decoded(b"\xbf"), ch(UNICODE_INVALID, 1));
        assert_eq!(decoded(b"\xfe"), ch(UNICODE_INVALID, 1));
        assert_eq!(decoded(b"\xff"), ch(UNICODE_INVALID, 1));
    }

    #[test]
    fn overlong_surrogate_and_non_characters_decode_as_invalid() {
        // U+0041 spelled in two bytes.
        assert_eq!(decoded(b"\xc1\x81"), ch(UNICODE_INVALID, 2));
        // U+D800, a surrogate half.
        assert_eq!(decoded(b"\xed\xa0\x80"), ch(UNICODE_INVALID, 3));
        assert_eq!(decoded(b"\xef\xbf\xbe"), ch(UNICODE_INVALID, 3));
        assert_eq!(decoded(b"\xef\xbf\xbf"), ch(UNICODE_INVALID, 3));
        // U+FFFD itself is fine.
        assert_eq!(decoded(b"\xef\xbf\xbd"), ch(0xfffd, 3));
    }
}
