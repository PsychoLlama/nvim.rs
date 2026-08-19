//! The `.un~` file format: its magic numbers and its integer encoding.
//!
//! This is the on-disk contract. Nothing here may change without breaking
//! every undo file that already exists, so the constants are spelled as the
//! literals a hex dump shows and the encoder is pinned by round-trip tests.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

/// The nine bytes every undo file starts with.
pub const UF_START_MAGIC: [u8; 9] = *b"Vim\x9fUnDo\xe5";
pub const UF_START_MAGIC_LEN: c_int = 9;

/// The format revision written after [`UF_START_MAGIC`]. A file claiming any
/// other version is rejected outright rather than guessed at.
pub const UF_VERSION: c_int = 3;

/// Introduces an undo header record.
pub const UF_HEADER_MAGIC: c_int = 0x5fd0;
/// Closes an undo header record.
pub const UF_HEADER_END_MAGIC: c_int = 0xe7aa;
/// Introduces an undo entry (or, in the header, an extmark record).
pub const UF_ENTRY_MAGIC: c_int = 0xf518;
/// Closes a run of undo entries.
pub const UF_ENTRY_END_MAGIC: c_int = 0x3581;

/// The file header's optional-field tag for `b_u_save_nr_last`.
pub const UF_LAST_SAVE_NR: c_int = 1;
/// An undo header's optional-field tag for `uh_save_nr`.
pub const UHP_SAVE_NR: c_int = 1;

/// Encodes `nr` as `len` bytes, most significant first — the file's one and
/// only integer representation. Fields are 1, 2, 4 or 8 bytes wide.
///
/// The C wrote through an eight-byte stack buffer with only `len > 0`
/// asserted, so a wider field would have run off the end of it; this rejects
/// that outright. No caller asks for one.
pub fn encode_be(nr: u64, len: usize) -> [u8; 8] {
    assert!((1..=8).contains(&len), "undo file fields are 1..=8 bytes");
    let mut buf = [0u8; 8];
    // The low `len` bytes, most significant first: the tail of the number's
    // own big-endian encoding.
    buf[..len].copy_from_slice(&nr.to_be_bytes()[8 - len..]);
    buf
}

/// The inverse of [`encode_be`], over the bytes the reader collected.
pub fn decode_be(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0u64, |acc, &b| (acc << 8) | u64::from(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_start_magic_is_what_is_already_on_disk() {
        assert_eq!(&UF_START_MAGIC, b"Vim\x9fUnDo\xe5");
        assert_eq!(UF_START_MAGIC.len(), UF_START_MAGIC_LEN as usize);
    }

    #[test]
    fn integers_are_big_endian() {
        assert_eq!(&encode_be(UF_HEADER_MAGIC as u64, 2)[..2], &[0x5f, 0xd0]);
        assert_eq!(&encode_be(1, 1)[..1], &[1]);
        assert_eq!(&encode_be(0x0123_4567, 4)[..4], &[0x01, 0x23, 0x45, 0x67]);
    }

    #[test]
    fn a_value_wider_than_its_field_keeps_its_low_bytes() {
        // The C truncated silently by shifting; nothing checks the range.
        assert_eq!(&encode_be(0x1234, 1)[..1], &[0x34]);
        assert_eq!(&encode_be(u64::MAX, 2)[..2], &[0xff, 0xff]);
    }

    #[test]
    fn round_trips() {
        for &(nr, len) in &[
            (0u64, 1usize),
            (255, 1),
            (0xe7aa, 2),
            (0x7fff_ffff, 4),
            (0x0102_0304_0506_0708, 8),
            (u64::MAX, 8),
        ] {
            assert_eq!(decode_be(&encode_be(nr, len)[..len]), nr, "{nr:#x}/{len}");
        }
    }

    #[test]
    #[should_panic(expected = "1..=8")]
    fn a_nine_byte_field_is_rejected() {
        encode_be(0, 9);
    }
}
