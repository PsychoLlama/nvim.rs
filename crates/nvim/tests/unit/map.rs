//! `Set_*`/`Map_*` end to end, through the entry points the editor calls.
//!
//! What is pinned here is the *observable* layout, not just the answers:
//! callers iterate `set.keys[..set.h.n_keys]` and index `map.values` with the
//! index `mh_get` returns, so the dense packing, the insertion order and the
//! swap-with-last that `mh_delete` performs are all behaviour.

use std::ffi::{CStr, c_char, c_void};
use std::{ptr, slice};

use neovim::map::{
    MH_TOMBSTONE, kMHExisting, map_del_uint64_t_ptr_t, map_put_ref_uint64_t_ptr_t, mh_clear,
    mh_delete_uint32_t, mh_get_cstr_t, mh_get_uint32_t, mh_get_uint64_t, mh_put_cstr_t,
    mh_put_uint32_t,
};
use neovim::memory::xfree;
use neovim::types::{
    Map_cstr_t_ptr_t, Map_uint64_t_ptr_t, MapHash, Set_cstr_t, Set_uint32_t, Set_uint64_t, uint32_t,
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
fn empty_u64_map() -> Map_uint64_t_ptr_t {
    Map_uint64_t_ptr_t {
        set: Set_uint64_t {
            h: EMPTY_HASH,
            keys: ptr::null_mut(),
        },
        values: ptr::null_mut(),
    }
}

/// Insert `key` with `key` itself (as an address) for its value, and say
/// whether it was new.
unsafe fn put(map: &mut Map_uint64_t_ptr_t, key: u64) -> bool {
    let mut is_new = false;
    unsafe {
        let slot = map_put_ref_uint64_t_ptr_t(map, key, ptr::null_mut(), &raw mut is_new);
        if is_new {
            *slot = ptr::without_provenance_mut(key as usize);
        }
    }
    is_new
}

unsafe fn free_u64_map(map: &mut Map_uint64_t_ptr_t) {
    unsafe {
        xfree(map.set.h.hash as *mut c_void);
        xfree(map.set.keys as *mut c_void);
        xfree(map.values as *mut c_void);
    }
}

unsafe fn keys(map: &Map_uint64_t_ptr_t) -> &[u64] {
    if map.set.h.n_keys == 0 {
        return &[];
    }
    unsafe { slice::from_raw_parts(map.set.keys, map.set.h.n_keys as usize) }
}

/// Keys are appended to a dense array and the index `mh_get` answers is the
/// position in it, so insertion order is visible to every caller that walks
/// `keys` — and to every caller that indexes `values` with it.
#[test]
fn keys_are_dense_and_in_insertion_order() {
    let mut map = empty_u64_map();
    unsafe {
        for key in [10u64, 20, 30, 40] {
            assert!(put(&mut map, key));
        }
        assert_eq!(keys(&map), &[10, 20, 30, 40]);
        assert_eq!(map.set.h.size, 4);
        for (n, key) in [10u64, 20, 30, 40].into_iter().enumerate() {
            let index = mh_get_uint64_t(&raw mut map.set, key);
            assert_eq!(index, n as uint32_t);
            assert_eq!((*map.values.add(index as usize)).addr(), key as usize);
        }
        assert_eq!(mh_get_uint64_t(&raw mut map.set, 99), MH_TOMBSTONE);

        // Re-inserting reports "not new" and hands back the existing slot.
        assert!(!put(&mut map, 20));
        assert_eq!(map.set.h.n_keys, 4);
        free_u64_map(&mut map);
    }
}

/// Deleting from the middle moves the *last* key into the hole, value and
/// all. Callers that walk `keys` while deleting rely on this: it is why they
/// re-read the same index instead of advancing.
#[test]
fn deleting_moves_the_last_entry_into_the_hole() {
    let mut map = empty_u64_map();
    unsafe {
        for key in [10u64, 20, 30, 40] {
            put(&mut map, key);
        }

        let removed = map_del_uint64_t_ptr_t(&raw mut map, 20, ptr::null_mut());
        assert_eq!(removed.addr(), 20);
        assert_eq!(keys(&map), &[10, 40, 30]);
        assert_eq!(mh_get_uint64_t(&raw mut map.set, 40), 1);
        assert_eq!((*map.values.add(1)).addr(), 40);
        assert_eq!(mh_get_uint64_t(&raw mut map.set, 20), MH_TOMBSTONE);

        // A miss answers the value initializer — null for a pointer map, so
        // indistinguishable from a stored null, as upstream.
        assert!(map_del_uint64_t_ptr_t(&raw mut map, 999, ptr::null_mut()).is_null());

        // Deleting the last entry needs no move.
        assert_eq!(
            map_del_uint64_t_ptr_t(&raw mut map, 30, ptr::null_mut()).addr(),
            30
        );
        assert_eq!(keys(&map), &[10, 40]);
        free_u64_map(&mut map);
    }
}

/// The bucket a deletion vacates becomes a tombstone: a later lookup walks
/// past it, and a later insertion reuses it. `uint32_t` hashes to itself, so
/// on the initial 16 buckets 1, 17 and 33 all land on bucket 1 and take
/// successive probes — which is what makes the walk observable.
#[test]
fn a_deleted_bucket_is_a_tombstone_that_the_next_insertion_reuses() {
    let mut set = Set_uint32_t {
        h: EMPTY_HASH,
        keys: ptr::null_mut(),
    };
    unsafe {
        let mut status = kMHExisting;
        for key in [1u32, 17, 33] {
            mh_put_uint32_t(&raw mut set, key, &raw mut status);
        }
        assert_eq!(set.h.n_buckets, 16);
        let occupied = set.h.n_occupied;

        let mut victim = 17u32;
        assert_eq!(mh_delete_uint32_t(&raw mut set, &raw mut victim), 1);
        assert_eq!(victim, 17, "the stored key is handed back");
        assert_eq!(set.h.size, 2);
        assert_eq!(
            set.h.n_occupied, occupied,
            "a tombstone still occupies its bucket"
        );
        // 33's probe walks past the tombstone; it moved into the vacated
        // dense slot.
        assert_eq!(mh_get_uint32_t(&raw mut set, 33), 1);
        assert_eq!(mh_get_uint32_t(&raw mut set, 1), 0);
        assert_eq!(mh_get_uint32_t(&raw mut set, 17), MH_TOMBSTONE);

        mh_put_uint32_t(&raw mut set, 17, &raw mut status);
        assert_ne!(status, kMHExisting);
        assert_eq!(
            set.h.n_occupied, occupied,
            "the insertion reused the tombstone rather than filling a bucket"
        );
        xfree(set.h.hash as *mut c_void);
        xfree(set.keys as *mut c_void);
    }
}

/// Past the load factor the bucket table doubles and everything is rehashed;
/// the dense indices are untouched by that.
#[test]
fn growing_past_the_load_factor_keeps_every_key() {
    let mut map = empty_u64_map();
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
        free_u64_map(&mut map);
    }
}

/// String keys compare by content, not by address — two distinct buffers
/// holding the same bytes are the same key, and the *first* one inserted is
/// the one the table keeps.
#[test]
fn string_keys_compare_by_content() {
    let mut map = Map_cstr_t_ptr_t {
        set: Set_cstr_t {
            h: EMPTY_HASH,
            keys: ptr::null_mut(),
        },
        values: ptr::null_mut(),
    };
    let alpha: Vec<c_char> = c"alpha"
        .to_bytes_with_nul()
        .iter()
        .map(|&b| b as c_char)
        .collect();
    unsafe {
        let mut status = kMHExisting;
        mh_put_cstr_t(&raw mut map.set, c"alpha".as_ptr(), &raw mut status);
        assert_ne!(status, kMHExisting);
        assert_eq!(mh_get_cstr_t(&raw mut map.set, alpha.as_ptr()), 0);
        mh_put_cstr_t(&raw mut map.set, alpha.as_ptr(), &raw mut status);
        assert_eq!(status, kMHExisting, "same bytes, same key");
        assert_eq!(map.set.h.n_keys, 1);
        assert_eq!(CStr::from_ptr(*map.set.keys), c"alpha");
        assert_eq!(
            mh_get_cstr_t(&raw mut map.set, c"beta".as_ptr()),
            MH_TOMBSTONE
        );

        xfree(map.set.h.hash as *mut c_void);
        xfree(map.set.keys as *mut c_void);
    }
}

/// The value slot `map_put_ref` hands back is `values[index]`, and the array
/// tracks `keys_capacity` — a caller that holds the pointer across another
/// insertion is holding a dangling one, as it always was.
#[test]
fn the_returned_slot_is_the_values_array() {
    let mut map = empty_u64_map();
    unsafe {
        let mut is_new = false;
        let slot = map_put_ref_uint64_t_ptr_t(&raw mut map, 1, ptr::null_mut(), &raw mut is_new);
        assert!(is_new);
        assert!(!map.values.is_null());
        assert_eq!(slot, map.values);
        assert_eq!(map.set.h.keys_capacity, 8);
        free_u64_map(&mut map);
    }
}
