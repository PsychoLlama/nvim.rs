//! `hashtab_T` end to end: the probe sequence, the tombstone bookkeeping and
//! the rehash, driven through the same entry points the editor uses.
//!
//! The slot numbers here are hand-computed from the C's hash fold, so they
//! fail if the placement ever drifts — callers walk `ht_array` directly, so
//! placement is observable behaviour, not an implementation detail.

use std::ffi::{CStr, CString, c_char, c_uint, c_void};
use std::{ptr, slice};

use neovim::hashtab::{
    HT_INIT_SIZE, hash_add, hash_clear_all, hash_find, hash_init, hash_lock, hash_remove,
    hash_unlock,
};
use neovim::memory::{xcalloc, xfree};
use neovim::types::{hashitem_T, hashtab_T};

const EMPTY_ITEM: hashitem_T = hashitem_T {
    hi_hash: 0,
    hi_key: ptr::null_mut(),
};

/// An all-zero table, as a freshly `xcalloc`'d struct's field is.
/// `hash_init` points `ht_array` at the struct's own `ht_smallarray`, so
/// it has to run where the table is going to live — this cannot return an
/// initialized one.
const ZEROED: hashtab_T = hashtab_T {
    ht_mask: 0,
    ht_used: 0,
    ht_filled: 0,
    ht_changed: 0,
    ht_locked: 0,
    ht_array: ptr::null_mut(),
    ht_smallarray: [EMPTY_ITEM; HT_INIT_SIZE],
};

/// A key the table can own: `hash_clear_all` frees keys with `xfree`, and
/// the crate's allocator is libc's.
fn owned_key(text: &str) -> *mut c_char {
    CString::new(text).unwrap().into_raw()
}

fn slot_of(ht: &hashtab_T, key: &CStr) -> usize {
    let hi = unsafe { hash_find(ht, key.as_ptr()) };
    (hi.addr() - ht.ht_array.addr()) / size_of::<hashitem_T>()
}

/// Hand-computed against the C: `hash_hash("a")` is 97, so on a 16-slot
/// table `a` lands at 97 & 15 == 1. `hash_hash("q")` is 113, which also
/// masks to 1, so `q` takes the second probe: 1 * 5 + 113 + 1 == 119,
/// masked to 7.
#[test]
fn a_collision_lands_on_the_second_probe() {
    let mut ht = ZEROED;
    unsafe {
        hash_init(&raw mut ht);
        assert_eq!(ht.ht_mask, 15);
        assert_eq!(hash_add(&raw mut ht, owned_key("a")), Ok(()));
        assert_eq!(hash_add(&raw mut ht, owned_key("q")), Ok(()));
        assert_eq!(slot_of(&ht, c"a"), 1);
        assert_eq!(slot_of(&ht, c"q"), 7);
        assert_eq!(ht.ht_used, 2);
        assert_eq!(ht.ht_filled, 2);
        hash_clear_all(&raw mut ht, 0);
    }
}

/// Removing an item leaves a tombstone that a later lookup walks through
/// but a later insertion reuses — that is the whole point of keeping the
/// two counters apart.
#[test]
fn a_removed_key_leaves_a_reusable_tombstone() {
    let mut ht = ZEROED;
    unsafe {
        hash_init(&raw mut ht);
        let _ = hash_add(&raw mut ht, owned_key("a"));
        let _ = hash_add(&raw mut ht, owned_key("q"));
        // Lock the table: removing would otherwise be free to compact the
        // tombstone away immediately.
        hash_lock(&raw mut ht);

        let a = hash_find(&ht, c"a".as_ptr());
        xfree((*a).hi_key as *mut c_void);
        hash_remove(&raw mut ht, a);
        assert!((*a).is_removed());
        assert_eq!(ht.ht_used, 1);
        assert_eq!(ht.ht_filled, 2, "a tombstone still occupies its slot");

        // `q` is still reachable: the walk crosses the tombstone.
        assert_eq!(slot_of(&ht, c"q"), 7);
        // A miss whose walk crossed the tombstone reports the tombstone,
        // not the empty slot that ended the walk.
        assert_eq!(slot_of(&ht, c"A"), 1);

        let _ = hash_add(&raw mut ht, owned_key("A"));
        assert_eq!(slot_of(&ht, c"A"), 1);
        assert_eq!(ht.ht_used, 2);
        assert_eq!(ht.ht_filled, 2, "reusing a tombstone fills nothing new");

        hash_unlock(&raw mut ht);
        hash_clear_all(&raw mut ht, 0);
    }
}

/// Past the small array's load factor the table moves to the heap, keeps
/// every key findable, and reports `filled == used` again because the
/// rehash drops the tombstones.
#[test]
fn growing_off_the_small_array_keeps_every_key() {
    let mut ht = ZEROED;
    let keys: Vec<CString> = (0..64)
        .map(|i| CString::new(format!("key{i}")).unwrap())
        .collect();
    unsafe {
        hash_init(&raw mut ht);
        let small = (&raw mut ht.ht_smallarray) as *mut hashitem_T;
        for key in &keys {
            assert_eq!(
                hash_add(&raw mut ht, owned_key(key.to_str().unwrap())),
                Ok(())
            );
        }
        assert_eq!(ht.ht_used, 64);
        assert_eq!(ht.ht_filled, 64);
        assert!(ht.ht_mask + 1 >= 256, "mask {}", ht.ht_mask);
        assert!(ht.ht_array != small);
        for key in &keys {
            let hi = hash_find(&ht, key.as_ptr());
            assert!((*hi).is_kept());
            assert_eq!(CStr::from_ptr((*hi).hi_key), key.as_c_str());
        }

        // Every slot is either empty, a tombstone, or a key we put there;
        // exactly `ht_used` of them are live.
        let items = slice::from_raw_parts(ht.ht_array, ht.ht_mask + 1);
        assert_eq!(items.iter().filter(|hi| hi.is_kept()).count(), 64);

        for key in keys.iter().take(50) {
            let hi = hash_find(&ht, key.as_ptr());
            xfree((*hi).hi_key as *mut c_void);
            hash_remove(&raw mut ht, hi);
        }
        assert_eq!(ht.ht_used, 14);
        for key in keys.iter().take(50) {
            assert!(!(*hash_find(&ht, key.as_ptr())).is_kept());
        }
        for key in keys.iter().skip(50) {
            assert!((*hash_find(&ht, key.as_ptr())).is_kept());
        }
        hash_clear_all(&raw mut ht, 0);
    }
}

/// `hash_clear_all` takes the offset of the key inside its allocation, for
/// tables whose keys are a trailing member of a larger struct.
#[test]
fn clear_all_frees_the_allocation_the_key_sits_in() {
    #[repr(C)]
    struct Entry {
        payload: u64,
        key: [c_char; 4],
    }
    let mut ht = ZEROED;
    unsafe {
        hash_init(&raw mut ht);
        for (i, text) in ["ab", "cd"].iter().enumerate() {
            let entry = xcalloc(1, size_of::<Entry>()) as *mut Entry;
            (*entry).payload = i as u64;
            (*entry).key[0] = text.as_bytes()[0] as c_char;
            (*entry).key[1] = text.as_bytes()[1] as c_char;
            assert_eq!(
                hash_add(&raw mut ht, (&raw mut (*entry).key) as *mut c_char),
                Ok(())
            );
        }
        assert_eq!(ht.ht_used, 2);
        hash_clear_all(&raw mut ht, core::mem::offset_of!(Entry, key) as c_uint);
    }
}
