//! The `.un~` file format. These bytes outlive the editor: a build that
//! writes a header another build cannot read loses every user's persistent
//! undo, so the constants and the encoder are asserted from outside the
//! crate as well as from inside it.

use c2rust_neovim::src::nvim::undo::format::*;

/// The nine bytes at offset 0 of every undo file since the format's
/// inception. `Vim`, 0x9f, `UnDo`, 0xe5.
#[test]
fn the_header_starts_with_the_documented_magic() {
    assert_eq!(&UF_START_MAGIC, b"Vim\x9fUnDo\xe5");
    assert_eq!(UF_START_MAGIC_LEN as usize, UF_START_MAGIC.len());
}

/// What a hex dump of the first eleven bytes shows.
#[test]
fn the_version_follows_the_magic_as_two_big_endian_bytes() {
    let mut head = Vec::from(UF_START_MAGIC);
    head.extend_from_slice(&encode_be(UF_VERSION as u64, 2)[..2]);
    assert_eq!(head, b"Vim\x9fUnDo\xe5\x00\x03");
}

#[test]
fn the_record_markers_are_unchanged() {
    assert_eq!(UF_HEADER_MAGIC, 0x5fd0);
    assert_eq!(UF_HEADER_END_MAGIC, 0xe7aa);
    assert_eq!(UF_ENTRY_MAGIC, 0xf518);
    assert_eq!(UF_ENTRY_END_MAGIC, 0x3581);
    assert_eq!(UF_LAST_SAVE_NR, 1);
    assert_eq!(UHP_SAVE_NR, 1);
}

/// Every field width the writer actually uses, both directions.
#[test]
fn every_field_width_round_trips() {
    let cases: &[(u64, usize)] = &[
        (0, 1),
        (1, 1),
        (0xff, 1),
        (UF_HEADER_MAGIC as u64, 2),
        (UF_ENTRY_END_MAGIC as u64, 2),
        (0xffff, 2),
        (0x7fff_ffff, 4),
        (0xffff_ffff, 4),
        // `uh_time` and `b_u_time_cur` are eight-byte fields.
        (0x0000_0000_6800_0000, 8),
        (u64::MAX, 8),
    ];
    for &(nr, len) in cases {
        let encoded = encode_be(nr, len);
        assert_eq!(decode_be(&encoded[..len]), nr, "{nr:#x} in {len} bytes");
        // Most significant byte first, and nothing written past the field.
        assert_eq!(encoded[0], (nr >> ((len - 1) * 8)) as u8);
        assert!(encoded[len..].iter().all(|&b| b == 0));
    }
}

/// A sequence number wider than its field keeps its low bytes rather than
/// saturating or erroring — the C shifted and truncated, and undo files in
/// the wild were written that way.
#[test]
fn an_oversized_value_truncates_from_the_top() {
    assert_eq!(&encode_be(0x1234, 1)[..1], &[0x34]);
    assert_eq!(&encode_be(0x1_0000_0001, 4)[..4], &[0, 0, 0, 1]);
}
