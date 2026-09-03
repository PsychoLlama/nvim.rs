//! The codec over `&[u8]`, for callers that have the text rather than a
//! pointer into it.
//!
//! Every function here is the twin of a pointer form in
//! [`super`], and answers exactly what that form answers for the same bytes
//! -- with one deliberate difference, stated once because it applies to all
//! of them: **the end of the slice is the end of the string**. The pointer
//! forms stop at a NUL because that is where their string ends; a slice
//! carries its own length, so a NUL inside one is an ordinary byte. It is
//! one character long, not zero, which is what keeps a walk written against
//! these functions moving instead of looping forever on an embedded NUL.
//!
//! The point of them is that a caller rewriting a `*const c_char` walk into
//! `&[u8]` + an index does not have to re-derive UTF-8 boundary handling: a
//! bound the slice already carries replaces the NUL the pointer form relied
//! on, and nothing else about the answers changes. The equivalence is
//! tested rather than argued -- see the tests at the bottom, which run both
//! forms over the same bytes.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{LEAD_PAYLOAD, LEAD_PREFIX, utf_char2len, utf_is_trail_byte, utf8len_tab};

/// How many bytes the character at the start of `bytes` occupies, or 0 if
/// there is no character there because `bytes` is empty.
///
/// The slice form of [`utf_ptr2len`](super::utf_ptr2len). Answers 1 for
/// anything that is not a complete sequence *within the slice*, so a walk
/// always makes progress and never steps past the end: unlike
/// [`utf_ptr2len_len`](super::utf_ptr2len_len), which reports a truncated
/// sequence's full length so its caller can tell "incomplete" from
/// "invalid", the answer here is never larger than `bytes.len()`.
pub fn char_len(bytes: &[u8]) -> usize {
    let Some(&first) = bytes.first() else {
        return 0;
    };
    let len = usize::from(utf8len_tab[usize::from(first)]);
    if len > bytes.len() || !bytes[1..len].iter().all(|&byte| utf_is_trail_byte(byte)) {
        return 1;
    }
    len
}

/// The codepoint at the start of `bytes`, or the first byte's own value if
/// there is no complete sequence there. An empty slice answers 0.
///
/// The slice form of [`utf_ptr2char`](super::utf_ptr2char). Forgiving in the
/// same way: nothing is ever an error, so a walk over arbitrary bytes always
/// gets something back.
pub fn char_at(bytes: &[u8]) -> i32 {
    let Some(&first) = bytes.first() else {
        return 0;
    };
    let unchanged = i32::from(first);
    if first < 0x80 {
        return unchanged;
    }
    let len = usize::from(utf8len_tab[usize::from(first)]);
    // A continuation byte or 0xFE/0xFF is not a lead byte at all, and a
    // sequence the slice does not hold in full is not one either.
    if len < 2 || len > bytes.len() {
        return unchanged;
    }
    let mut code_point = u32::from(first) & LEAD_PAYLOAD[len];
    for &cur in &bytes[1..len] {
        if !utf_is_trail_byte(cur) {
            return unchanged;
        }
        code_point = (code_point << 6) | (u32::from(cur) & 0x3f);
    }
    // The widest sequence this codec accepts is six bytes carrying 31 bits,
    // so the value always fits: `0xFD` masks down to one payload bit.
    code_point.cast_signed()
}

/// Encode `c` into `out`, answering how many bytes it took.
///
/// The slice form of [`utf_char2bytes`](super::utf_char2bytes), which
/// delegates here. Panics if `out` is shorter than
/// [`utf_char2len`](super::utf_char2len) -- up to `MB_MAXCHAR` bytes --
/// which is the bound the pointer form asks its caller for and cannot check.
pub fn encode_char(c: i32, out: &mut [u8]) -> usize {
    let len = usize::try_from(utf_char2len(c)).expect("a character's length is positive");
    let out = &mut out[..len];
    if len == 1 {
        out[0] = u8::try_from(c & 0xff).expect("masked to a byte");
        return 1;
    }
    let u = c.cast_unsigned();
    // The lead byte carries the top bits under its `1..10` prefix; every
    // continuation byte carries six more under `10`.
    out[0] = truncate(LEAD_PREFIX[len] | (u >> (6 * (len - 1))));
    for (i, byte) in out.iter_mut().enumerate().skip(1) {
        *byte = truncate(0x80 | ((u >> (6 * (len - 1 - i))) & 0x3f));
    }
    len
}

/// The low byte of `bits`. Every caller has already masked or shifted the
/// value into a byte; this is where that is said once.
fn truncate(bits: u32) -> u8 {
    u8::try_from(bits & 0xff).expect("masked to a byte")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The one place the slice forms deliberately differ from the pointer
    /// ones: an embedded NUL is a byte, not the end of the string, so a walk
    /// over it advances instead of standing still.
    #[test]
    fn an_embedded_nul_is_one_ordinary_byte() {
        assert_eq!(char_len(b"\0a"), 1);
        assert_eq!(char_at(b"\0a"), 0);
    }

    /// An empty slice is the end, which is what a NUL means to the pointer
    /// forms: no character, length zero.
    #[test]
    fn nothing_is_left_at_the_end_of_the_slice() {
        assert_eq!(char_len(b""), 0);
        assert_eq!(char_at(b""), 0);
    }

    /// A sequence the slice does not hold in full is one byte long, never
    /// the length its lead byte promises -- the property that keeps an index
    /// walk in bounds.
    #[test]
    fn a_truncated_sequence_never_reaches_past_the_end() {
        for whole in [
            b"\xc3\xa9".as_slice(),
            b"\xe2\x82\xac",
            b"\xf0\x9f\x92\xa9",
            b"\xfc\x84\x80\x80\x80\x80",
        ] {
            assert_eq!(char_len(whole), whole.len());
            for cut in 1..whole.len() {
                assert_eq!(char_len(&whole[..cut]), 1, "{whole:?} cut to {cut}");
                assert_eq!(char_at(&whole[..cut]), i32::from(whole[0]));
            }
        }
    }

    #[test]
    fn encode_char_round_trips_every_length() {
        for c in [0, 0x41, 0x7f, 0x80, 0x7ff, 0x800, 0xffff, 0x10000, 0x1f4a9] {
            let mut buf = [0u8; 6];
            let len = encode_char(c, &mut buf);
            assert_eq!(len, usize::try_from(utf_char2len(c)).unwrap());
            assert_eq!(char_len(&buf[..len]), len, "{c:#x}");
            assert_eq!(char_at(&buf[..len]), c, "{c:#x}");
        }
    }
}
