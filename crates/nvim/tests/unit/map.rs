//! The khash-derived tables the editor still keeps, through their public
//! surface.
//!
//! What is pinned here is the *observable* numbering, not just the answers:
//! the three remaining callers — the TUI's URL set, the glyph cache and the
//! marktree's splice damage — read the dense index a key was given, so the
//! dense packing and the insertion order are behaviour. The bucket placement
//! itself is pinned slot for slot by `map`'s in-crate tests.
//!
//! The swap-remove half of khash's contract has no caller left here (nothing
//! deletes from these three); `registry.rs`'s own tests carry it, for the
//! `SlotTable` that reproduces it.

use neovim::map::{DenseMap, DenseSet};

/// Keys are appended to a dense array and the index a lookup answers is the
/// position in it, so insertion order is visible to every caller that walks
/// the keys — and to every caller that walks the values beside them, which is
/// what `marktree_splice`'s damage walk does.
#[test]
fn keys_are_dense_and_in_insertion_order() {
    let mut map = DenseMap::new();
    for key in [10u64, 20, 30, 40] {
        *map.get_or_insert(key, 0) = key + 1;
    }
    assert_eq!(map.keys(), &[10, 20, 30, 40]);
    assert_eq!(map.values(), &[11, 21, 31, 41]);
    assert_eq!(map.get(&99), None);

    // Re-inserting hands back the existing slot and adds nothing.
    assert_eq!(*map.get_or_insert(20, 0), 21);
    assert_eq!(map.len(), 4);
}

/// Past the load factor the bucket table doubles and everything is rehashed;
/// the dense indices are untouched by that.
#[test]
fn growing_past_the_load_factor_keeps_every_key() {
    let mut set = DenseSet::new();
    for key in 0..500u64 {
        set.insert(key * 7 + 1);
    }
    assert_eq!(set.len(), 500);
    for key in 0..500u32 {
        assert_eq!(set.get(&(u64::from(key) * 7 + 1)), Some(key));
    }
    assert_eq!(set.get(&0), None);
}

/// String keys compare by content, not by address — two distinct buffers
/// holding the same bytes are the same key, and the *first* one inserted is
/// the one the table keeps. This is `tui/attrs.rs`' URL set, whose dense
/// index is the OSC 8 id it writes.
#[test]
fn string_keys_compare_by_content() {
    let mut set = DenseSet::<Vec<u8>>::new();
    let alpha = b"alpha".to_vec();
    assert_eq!(
        set.insert_with(b"alpha".as_slice(), <[u8]>::to_vec),
        (0, true)
    );
    assert_eq!(set.get(alpha.as_slice()), Some(0));
    assert_eq!(
        set.insert_with(alpha.as_slice(), <[u8]>::to_vec),
        (0, false),
        "same bytes, same key"
    );
    assert_eq!(set.keys(), &[b"alpha".to_vec()]);
    assert_eq!(set.get(b"beta".as_slice()), None);
}
