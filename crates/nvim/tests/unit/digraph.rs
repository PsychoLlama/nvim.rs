//! Default digraph table integrity. The table is user-visible contract
//! (`:digraphs`, CTRL-K, `digraph_get()`), so it is locked down with a
//! checksum computed from the transpiled table it replaced (which itself
//! matched `v0.12.4` `digraph.c` entry for entry).

use c2rust_neovim::src::nvim::digraph::{default_digraphs, lookup_default};
use std::collections::HashSet;

#[test]
fn default_table_is_unchanged() {
    let table = default_digraphs();
    assert_eq!(table.len(), 1366);

    // FNV-1a over (char1, char2, result as little-endian u32), matching the
    // extraction of the original table.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for d in table {
        let mut bytes = vec![d.char1, d.char2];
        bytes.extend_from_slice(&(d.result as u32).to_le_bytes());
        for b in bytes {
            hash = (hash ^ b as u64).wrapping_mul(0x100_0000_01b3);
        }
    }
    assert_eq!(hash, 0x4abb_9183_e972_bd54);
}

#[test]
fn default_table_well_formed() {
    let table = default_digraphs();
    let mut seen = HashSet::new();
    for d in table {
        // Keys are printable ASCII pairs, results are valid nonzero
        // codepoints, and no pair is defined twice.
        assert!(d.char1.is_ascii() && !d.char1.is_ascii_control(), "{d:?}");
        assert!(d.char2.is_ascii() && !d.char2.is_ascii_control(), "{d:?}");
        assert!(
            d.result > 0 && char::from_u32(d.result as u32).is_some(),
            "{d:?}"
        );
        assert!(seen.insert((d.char1, d.char2)), "duplicate {d:?}");
    }
}

#[test]
fn lookup_finds_known_digraphs() {
    let get = |c1: u8, c2: u8| lookup_default(c1 as i32, c2 as i32);
    // First and last entries plus well-known digraphs from the docs.
    assert_eq!(get(b'N', b'U'), Some(0x0a));
    assert_eq!(get(b's', b't'), Some(0xfb06));
    assert_eq!(get(b'a', b':'), Some(0xe4)); // ä
    assert_eq!(get(b'E', b'u'), Some(0x20ac)); // €
    assert_eq!(get(b'C', b'o'), Some(0xa9)); // ©
    assert_eq!(get(b'-', b'>'), Some(0x2192)); // →
    assert_eq!(get(b'0', b'u'), Some(0x263a)); // ☺
    assert_eq!(get(b'?', b'I'), Some(0xbf)); // ¿
    assert_eq!(get(b'S', b'E'), Some(0xa7)); // §
    assert_eq!(get(b'x', b'x'), None);
}
