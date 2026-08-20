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
use core::mem::{offset_of, size_of};

use crate::types::{ExtmarkMove, ExtmarkSplice, bcount_t};

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

/// How many bytes an extmark record's payload occupies.
///
/// The payload is the one field in the whole format that is not a
/// big-endian integer: it is the *native memory image* of an
/// [`ExtmarkSplice`] or an [`ExtmarkMove`], which is why both keep
/// `#[repr(C)]`. Both have the same shape — six `c_int`s, then three
/// `bcount_t`s — and the codec below spells that shape out field by field so
/// that reading a payload never has to reinterpret bytes as a struct.
pub const EXTMARK_PAYLOAD_LEN: usize = 48;

/// Where each field of the image begins.
const PAYLOAD_INTS: usize = 6;
const INT_LEN: usize = size_of::<c_int>();
const BYTE_LEN: usize = size_of::<bcount_t>();
const BYTES_AT: usize = PAYLOAD_INTS * INT_LEN;

/// The claim the codec rests on: the layout it writes out by hand is exactly
/// the layout the two structs already have, so the file's bytes do not move.
const _: () = {
    assert!(
        INT_LEN == 4 && BYTE_LEN == 8,
        "the payload is 6 i32 + 3 i64"
    );
    assert!(size_of::<ExtmarkSplice>() == EXTMARK_PAYLOAD_LEN);
    assert!(size_of::<ExtmarkMove>() == EXTMARK_PAYLOAD_LEN);
    assert!(offset_of!(ExtmarkSplice, start_row) == 0);
    assert!(offset_of!(ExtmarkSplice, start_col) == INT_LEN);
    assert!(offset_of!(ExtmarkSplice, old_row) == 2 * INT_LEN);
    assert!(offset_of!(ExtmarkSplice, old_col) == 3 * INT_LEN);
    assert!(offset_of!(ExtmarkSplice, new_row) == 4 * INT_LEN);
    assert!(offset_of!(ExtmarkSplice, new_col) == 5 * INT_LEN);
    assert!(offset_of!(ExtmarkSplice, start_byte) == BYTES_AT);
    assert!(offset_of!(ExtmarkSplice, old_byte) == BYTES_AT + BYTE_LEN);
    assert!(offset_of!(ExtmarkSplice, new_byte) == BYTES_AT + 2 * BYTE_LEN);
    assert!(offset_of!(ExtmarkMove, start_row) == 0);
    assert!(offset_of!(ExtmarkMove, start_col) == INT_LEN);
    assert!(offset_of!(ExtmarkMove, extent_row) == 2 * INT_LEN);
    assert!(offset_of!(ExtmarkMove, extent_col) == 3 * INT_LEN);
    assert!(offset_of!(ExtmarkMove, new_row) == 4 * INT_LEN);
    assert!(offset_of!(ExtmarkMove, new_col) == 5 * INT_LEN);
    assert!(offset_of!(ExtmarkMove, start_byte) == BYTES_AT);
    assert!(offset_of!(ExtmarkMove, extent_byte) == BYTES_AT + BYTE_LEN);
    assert!(offset_of!(ExtmarkMove, new_byte) == BYTES_AT + 2 * BYTE_LEN);
};

/// The largest row or column an extmark undo record may name, and the
/// largest byte offset.
///
/// Nothing in the format bounds these — they are raw memory, written and
/// read back unchecked — and the apply path adds them together
/// (`start_row + old_row`, `start_byte + old_byte`, ...) without checking
/// either. Confining every coordinate to a quarter of its type's range means
/// every such sum still fits, whatever else the file says. A real record is
/// bounded by the buffer it describes and is nowhere near this.
const MAX_COORD: c_int = c_int::MAX / 4;
const MAX_BYTE: bcount_t = bcount_t::MAX / 4;

/// Whether a decoded payload names coordinates a buffer could actually have.
fn in_range(ints: [c_int; PAYLOAD_INTS], bytes: [bcount_t; 3]) -> bool {
    ints.iter().all(|&n| (0..=MAX_COORD).contains(&n))
        && bytes.iter().all(|&n| (0..=MAX_BYTE).contains(&n))
}

/// Splits an extmark payload into its six counts and three byte offsets.
fn payload_fields(image: &[u8; EXTMARK_PAYLOAD_LEN]) -> ([c_int; PAYLOAD_INTS], [bcount_t; 3]) {
    let int_at = |i: usize| {
        let at = i * INT_LEN;
        c_int::from_ne_bytes(image[at..at + INT_LEN].try_into().expect("4 bytes"))
    };
    let byte_at = |i: usize| {
        let at = BYTES_AT + i * BYTE_LEN;
        bcount_t::from_ne_bytes(image[at..at + BYTE_LEN].try_into().expect("8 bytes"))
    };
    (
        [
            int_at(0),
            int_at(1),
            int_at(2),
            int_at(3),
            int_at(4),
            int_at(5),
        ],
        [byte_at(0), byte_at(1), byte_at(2)],
    )
}

/// Lays six counts and three byte offsets back out as an extmark payload.
fn payload_image(ints: [c_int; PAYLOAD_INTS], bytes: [bcount_t; 3]) -> [u8; EXTMARK_PAYLOAD_LEN] {
    let mut image = [0u8; EXTMARK_PAYLOAD_LEN];
    for (i, n) in ints.iter().enumerate() {
        image[i * INT_LEN..(i + 1) * INT_LEN].copy_from_slice(&n.to_ne_bytes());
    }
    for (i, n) in bytes.iter().enumerate() {
        let at = BYTES_AT + i * BYTE_LEN;
        image[at..at + BYTE_LEN].copy_from_slice(&n.to_ne_bytes());
    }
    image
}

/// Reads a `kExtmarkSplice` payload, or refuses one whose coordinates no
/// change to a buffer could have produced.
pub fn decode_splice(image: &[u8; EXTMARK_PAYLOAD_LEN]) -> Option<ExtmarkSplice> {
    let (ints, bytes) = payload_fields(image);
    in_range(ints, bytes).then(|| ExtmarkSplice {
        start_row: ints[0],
        start_col: ints[1],
        old_row: ints[2],
        old_col: ints[3],
        new_row: ints[4],
        new_col: ints[5],
        start_byte: bytes[0],
        old_byte: bytes[1],
        new_byte: bytes[2],
    })
}

/// [`decode_splice`] for a `kExtmarkMove` payload.
pub fn decode_move(image: &[u8; EXTMARK_PAYLOAD_LEN]) -> Option<ExtmarkMove> {
    let (ints, bytes) = payload_fields(image);
    in_range(ints, bytes).then(|| ExtmarkMove {
        start_row: ints[0],
        start_col: ints[1],
        extent_row: ints[2],
        extent_col: ints[3],
        new_row: ints[4],
        new_col: ints[5],
        start_byte: bytes[0],
        extent_byte: bytes[1],
        new_byte: bytes[2],
    })
}

/// The bytes [`decode_splice`] reads back.
pub fn encode_splice(splice: &ExtmarkSplice) -> [u8; EXTMARK_PAYLOAD_LEN] {
    payload_image(
        [
            splice.start_row,
            splice.start_col,
            splice.old_row,
            splice.old_col,
            splice.new_row,
            splice.new_col,
        ],
        [splice.start_byte, splice.old_byte, splice.new_byte],
    )
}

/// The bytes [`decode_move`] reads back.
pub fn encode_move(move_0: &ExtmarkMove) -> [u8; EXTMARK_PAYLOAD_LEN] {
    payload_image(
        [
            move_0.start_row,
            move_0.start_col,
            move_0.extent_row,
            move_0.extent_col,
            move_0.new_row,
            move_0.new_col,
        ],
        [move_0.start_byte, move_0.extent_byte, move_0.new_byte],
    )
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

    /// The splice `1787204786-undoevil.sh` finds in a file nvim wrote:
    /// rows 2..4 collapsed to one line.
    fn a_splice() -> ExtmarkSplice {
        ExtmarkSplice {
            start_row: 2,
            start_col: 0,
            old_row: 2,
            old_col: 0,
            new_row: 1,
            new_col: 0,
            start_byte: 24,
            old_byte: 24,
            new_byte: 8,
        }
    }

    #[test]
    fn an_extmark_payload_is_the_native_image_of_the_struct() {
        let image = encode_splice(&a_splice());
        assert_eq!(image.len(), EXTMARK_PAYLOAD_LEN);
        // Field by field, exactly where the `#[repr(C)]` layout puts them.
        assert_eq!(&image[..4], &2i32.to_ne_bytes());
        assert_eq!(&image[8..12], &2i32.to_ne_bytes());
        assert_eq!(&image[16..20], &1i32.to_ne_bytes());
        assert_eq!(&image[24..32], &24i64.to_ne_bytes());
        assert_eq!(&image[40..48], &8i64.to_ne_bytes());
    }

    #[test]
    fn an_extmark_payload_round_trips() {
        let splice = decode_splice(&encode_splice(&a_splice())).expect("a real record");
        assert_eq!(splice.start_row, 2);
        assert_eq!(splice.old_row, 2);
        assert_eq!(splice.new_row, 1);
        assert_eq!(splice.start_byte, 24);
        assert_eq!(splice.new_byte, 8);

        let move_0 = ExtmarkMove {
            start_row: 3,
            start_col: 1,
            extent_row: 2,
            extent_col: 0,
            new_row: 9,
            new_col: 4,
            start_byte: 30,
            extent_byte: 20,
            new_byte: 90,
        };
        let back = decode_move(&encode_move(&move_0)).expect("a real record");
        assert_eq!(back.extent_row, 2);
        assert_eq!(back.new_row, 9);
        assert_eq!(back.extent_byte, 20);
    }

    #[test]
    fn a_patched_extmark_payload_is_refused() {
        // What the reproducer writes over the payload: every field at its
        // type's limit, so that `start_row + old_row` overflows on undo.
        let mut evil = a_splice();
        evil.start_row = c_int::MAX;
        evil.old_row = c_int::MAX;
        assert!(decode_splice(&encode_splice(&evil)).is_none());

        let mut huge_byte = a_splice();
        huge_byte.old_byte = bcount_t::MAX;
        assert!(decode_splice(&encode_splice(&huge_byte)).is_none());

        // Negative coordinates are not something a change produces either,
        // and `extmark_setraw` would place a mark at one.
        let mut negative = a_splice();
        negative.start_col = -1;
        assert!(decode_splice(&encode_splice(&negative)).is_none());

        let mut moved = ExtmarkMove {
            start_row: 0,
            start_col: 0,
            extent_row: 1,
            extent_col: 0,
            new_row: 0,
            new_col: 0,
            start_byte: 0,
            extent_byte: 1,
            new_byte: 0,
        };
        assert!(decode_move(&encode_move(&moved)).is_some());
        moved.new_byte = bcount_t::MIN;
        assert!(decode_move(&encode_move(&moved)).is_none());
    }

    #[test]
    fn the_range_a_payload_may_use_is_a_quarter_of_the_type() {
        let mut at_limit = a_splice();
        at_limit.start_row = MAX_COORD;
        at_limit.old_row = MAX_COORD;
        let ok = decode_splice(&encode_splice(&at_limit)).expect("still in range");
        // The sum the apply path takes, which is what the bound is for.
        assert!(ok.start_row.checked_add(ok.old_row).is_some());

        let mut past = at_limit;
        past.start_row = MAX_COORD + 1;
        assert!(decode_splice(&encode_splice(&past)).is_none());
    }
}
