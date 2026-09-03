//! The khash-derived tables the editor still keeps, through the entry points
//! it calls them by.
//!
//! What is pinned here is the *observable* layout, not just the answers: the
//! three remaining callers — the TUI's URL set, the glyph cache and the
//! marktree's splice damage — iterate `set.keys[..set.h.n_keys]` and index
//! `map.values` with the index `mh_get` returns, so the dense packing and the
//! insertion order are behaviour.
//!
//! The swap-remove half of khash's contract has no caller left here (nothing
//! deletes from these three); `registry.rs`'s own tests carry it, for the
//! `SlotTable` that reproduces it.

use std::ffi::{CStr, c_char, c_void};
use std::{ptr, slice};

use neovim::map::{
    MH_TOMBSTONE, kMHExisting, map_put_ref_uint64_t_mt_damage_pair, mh_clear, mh_get_cstr_t,
    mh_get_uint64_t, mh_put_cstr_t,
};
use neovim::memory::xfree;
use neovim::types::{
    MTDamagePair, Map_uint64_t_MTDamagePair, MapHash, Set_cstr_t, Set_uint64_t, uint32_t,
};

const EMPTY_HASH: MapHash = MapHash {
    n_buckets: 0,
    size: 0,
    n_occupied: 0,
    upper_bound: 0,
    n_keys: 0,
    keys_capacity: 0,
    hash: ptr::null_mut(),
};

/// An all-zero map, which is how every embedded one starts life.
fn empty_damage_map() -> Map_uint64_t_MTDamagePair {
    Map_uint64_t_MTDamagePair {
        set: Set_uint64_t {
            h: EMPTY_HASH,
            keys: ptr::null_mut(),
        },
        values: ptr::null_mut(),
    }
}

/// Insert `key`, tagging its value's `old_i` with `key` so the slot can be
/// told apart, and say whether it was new.
unsafe fn put(map: &mut Map_uint64_t_MTDamagePair, key: u64) -> bool {
    let mut is_new = false;
    unsafe {
        let slot = map_put_ref_uint64_t_mt_damage_pair(map, key, ptr::null_mut(), &raw mut is_new);
        if is_new {
            (*slot).start.old_i = key as i32;
        }
    }
    is_new
}

unsafe fn free_damage_map(map: &mut Map_uint64_t_MTDamagePair) {
    unsafe {
        xfree(map.set.h.hash as *mut c_void);
        xfree(map.set.keys as *mut c_void);
        xfree(map.values as *mut c_void);
    }
}

unsafe fn keys(map: &Map_uint64_t_MTDamagePair) -> &[u64] {
    if map.set.h.n_keys == 0 {
        return &[];
    }
    unsafe { slice::from_raw_parts(map.set.keys, map.set.h.n_keys as usize) }
}

/// Keys are appended to a dense array and the index `mh_get` answers is the
/// position in it, so insertion order is visible to every caller that walks
/// `keys` — and to every caller that indexes `values` with it, which is what
/// `marktree_splice`'s damage walk does.
#[test]
fn keys_are_dense_and_in_insertion_order() {
    let mut map = empty_damage_map();
    unsafe {
        for key in [10u64, 20, 30, 40] {
            assert!(put(&mut map, key));
        }
        assert_eq!(keys(&map), &[10, 20, 30, 40]);
        assert_eq!(map.set.h.size, 4);
        for (n, key) in [10u64, 20, 30, 40].into_iter().enumerate() {
            let index = mh_get_uint64_t(&raw mut map.set, key);
            assert_eq!(index, n as uint32_t);
            assert_eq!((*map.values.add(index as usize)).start.old_i, key as i32);
        }
        assert_eq!(mh_get_uint64_t(&raw mut map.set, 99), MH_TOMBSTONE);

        // Re-inserting reports "not new" and hands back the existing slot.
        assert!(!put(&mut map, 20));
        assert_eq!(map.set.h.n_keys, 4);
        free_damage_map(&mut map);
    }
}

/// A fresh slot is written with the value type's zero — `MTDAMAGE_PAIR_INIT`
/// for this map — before the caller sees it.
#[test]
fn a_fresh_slot_starts_from_the_value_types_zero() {
    let mut map = empty_damage_map();
    unsafe {
        let mut is_new = false;
        let slot =
            map_put_ref_uint64_t_mt_damage_pair(&raw mut map, 1, ptr::null_mut(), &raw mut is_new);
        assert!(is_new);
        assert!(!map.values.is_null());
        assert_eq!(slot, map.values);
        assert_eq!(map.set.h.keys_capacity, 8);
        let pair: &MTDamagePair = &*slot;
        assert!(pair.start.old.is_null() && pair.end.new.is_null());
        assert_eq!(pair.start.old_i, 0);
        free_damage_map(&mut map);
    }
}

/// Past the load factor the bucket table doubles and everything is rehashed;
/// the dense indices are untouched by that.
#[test]
fn growing_past_the_load_factor_keeps_every_key() {
    let mut map = empty_damage_map();
    unsafe {
        for key in 0..500u64 {
            put(&mut map, key * 7 + 1);
        }
        assert_eq!(map.set.h.n_keys, 500);
        assert_eq!(map.set.h.size, 500);
        assert!(map.set.h.n_buckets >= 1024, "{}", map.set.h.n_buckets);
        assert_eq!(
            map.set.h.n_buckets.count_ones(),
            1,
            "buckets stay a power of two"
        );
        for key in 0..500u64 {
            assert_eq!(
                mh_get_uint64_t(&raw mut map.set, key * 7 + 1),
                key as uint32_t
            );
        }
        assert_eq!(mh_get_uint64_t(&raw mut map.set, 0), MH_TOMBSTONE);

        // `mh_clear` forgets the entries but keeps the allocation.
        let buckets = map.set.h.n_buckets;
        mh_clear(&raw mut map.set.h);
        assert_eq!(map.set.h.n_keys, 0);
        assert_eq!(map.set.h.size, 0);
        assert_eq!(map.set.h.n_buckets, buckets);
        assert_eq!(mh_get_uint64_t(&raw mut map.set, 8), MH_TOMBSTONE);
        free_damage_map(&mut map);
    }
}

/// String keys compare by content, not by address — two distinct buffers
/// holding the same bytes are the same key, and the *first* one inserted is
/// the one the table keeps. This is `tui/attrs.rs`' URL set, whose dense
/// index is the OSC 8 id it writes.
#[test]
fn string_keys_compare_by_content() {
    let mut set = Set_cstr_t {
        h: EMPTY_HASH,
        keys: ptr::null_mut(),
    };
    let alpha: Vec<c_char> = c"alpha"
        .to_bytes_with_nul()
        .iter()
        .map(|&b| b as c_char)
        .collect();
    unsafe {
        let mut status = kMHExisting;
        mh_put_cstr_t(&raw mut set, c"alpha".as_ptr(), &raw mut status);
        assert_ne!(status, kMHExisting);
        assert_eq!(mh_get_cstr_t(&raw mut set, alpha.as_ptr()), 0);
        mh_put_cstr_t(&raw mut set, alpha.as_ptr(), &raw mut status);
        assert_eq!(status, kMHExisting, "same bytes, same key");
        assert_eq!(set.h.n_keys, 1);
        assert_eq!(CStr::from_ptr(*set.keys), c"alpha");
        assert_eq!(mh_get_cstr_t(&raw mut set, c"beta".as_ptr()), MH_TOMBSTONE);

        xfree(set.h.hash as *mut c_void);
        xfree(set.keys as *mut c_void);
    }
}
