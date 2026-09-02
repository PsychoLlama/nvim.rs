//! `hashtab_T` end to end: the probe sequence, the tombstone bookkeeping and
//! the rehash, driven through the same entry points the editor uses.
//!
//! The slot numbers here are hand-computed from the C's hash fold, so they
//! fail if the placement ever drifts — callers walk the slots directly and
//! Vim shows the result, so placement is observable behaviour and not an
//! implementation detail.

use std::ffi::{CStr, CString, c_char, c_uint, c_void};

use neovim::hashtab::{
    HT_INIT_SIZE, hash_add, hash_clear_all, hash_find, hash_lock, hash_remove, hash_unlock,
};
use neovim::memory::{xcalloc, xfree};
use neovim::types::hashtab_T;

/// A key the table can own: `hash_clear_all` frees keys with `xfree`, and
/// the crate's allocator is libc's.
fn owned_key(text: &str) -> *mut c_char {
    CString::new(text).unwrap().into_raw()
}

fn slot_of(ht: &hashtab_T, key: &CStr) -> usize {
    unsafe { hash_find(ht, key.as_ptr()) }.index()
}

/// Hand-computed against the C: `hash_hash("a")` is 97, so on a 16-slot
/// table `a` lands at 97 & 15 == 1. `hash_hash("q")` is 113, which also
/// masks to 1, so `q` takes the second probe: 1 * 5 + 113 + 1 == 119,
/// masked to 7.
#[test]
fn a_collision_lands_on_the_second_probe() {
    let mut ht = hashtab_T::init();
    unsafe {
        assert_eq!(ht.size(), HT_INIT_SIZE);
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
    let mut ht = hashtab_T::init();
    unsafe {
        let _ = hash_add(&raw mut ht, owned_key("a"));
        let _ = hash_add(&raw mut ht, owned_key("q"));
        // Lock the table: removing would otherwise be free to compact the
        // tombstone away immediately.
        hash_lock(&raw mut ht);

        let a = hash_find(&ht, c"a".as_ptr());
        xfree(a.hi_key as *mut c_void);
        hash_remove(&raw mut ht, a);
        assert!(ht.slot(a.index()).is_removed());
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
    let mut ht = hashtab_T::init();
    let keys: Vec<CString> = (0..64)
        .map(|i| CString::new(format!("key{i}")).unwrap())
        .collect();
    unsafe {
        for key in &keys {
            assert_eq!(
                hash_add(&raw mut ht, owned_key(key.to_str().unwrap())),
                Ok(())
            );
        }
        assert_eq!(ht.ht_used, 64);
        assert_eq!(ht.ht_filled, 64);
        assert!(ht.size() >= 256, "size {}", ht.size());
        for key in &keys {
            let hi = hash_find(&ht, key.as_ptr());
            assert!(hi.is_kept());
            assert_eq!(CStr::from_ptr(hi.hi_key), key.as_c_str());
        }

        // Every slot is either empty, a tombstone, or a key we put there;
        // exactly `ht_used` of them are live.
        assert_eq!(ht.items().count(), 64);

        for key in keys.iter().take(50) {
            let hi = hash_find(&ht, key.as_ptr());
            xfree(hi.hi_key as *mut c_void);
            hash_remove(&raw mut ht, hi);
        }
        assert_eq!(ht.ht_used, 14);
        for key in keys.iter().take(50) {
            assert!(!hash_find(&ht, key.as_ptr()).is_kept());
        }
        for key in keys.iter().skip(50) {
            assert!(hash_find(&ht, key.as_ptr()).is_kept());
        }
        hash_clear_all(&raw mut ht, 0);
    }
}

/// Every live key, in slot order: what `TV_DICT_ITER` -- and so `keys()`,
/// `values()`, `items()` and every `:echo` of a Dictionary -- hands out.
unsafe fn keys_in_slot_order(ht: &hashtab_T) -> Vec<String> {
    ht.items()
        .map(|hi| {
            unsafe { CStr::from_ptr(hi.hi_key) }
                .to_str()
                .unwrap()
                .to_owned()
        })
        .collect()
}

/// The iteration order is Vim-visible, so it is pinned here as a literal
/// rather than merely asserted to be stable. The slots are hand-computed
/// from the fold (`hash * 101 + byte`, seeded with the first byte) and the
/// probe sequence: on a 16-slot table `a`(97), `q`(113), `A`(65) and `Q`(81)
/// all mask to 1 and walk on to 7, 6 and 15; `b`(98) and `r`(114) mask to 2
/// and `r` walks on to 13.
#[test]
fn iteration_visits_slots_in_index_order() {
    let mut ht = hashtab_T::init();
    unsafe {
        for key in ["a", "q", "A", "Q", "b", "r"] {
            assert_eq!(hash_add(&raw mut ht, owned_key(key)), Ok(()));
        }
        assert_eq!(keys_in_slot_order(&ht), ["a", "b", "A", "q", "r", "Q"]);
        hash_clear_all(&raw mut ht, 0);
    }
}

/// A tombstone is reused by the first insertion whose probe sequence crosses
/// it, which puts the new key *earlier* than the empty slot that would have
/// ended the walk.
#[test]
fn a_reused_tombstone_keeps_its_slot_in_the_order() {
    let mut ht = hashtab_T::init();
    unsafe {
        for key in ["a", "q", "A"] {
            assert_eq!(hash_add(&raw mut ht, owned_key(key)), Ok(()));
        }
        let q = hash_find(&ht, c"q".as_ptr());
        xfree(q.hi_key as *mut c_void);
        hash_remove(&raw mut ht, q);
        // `Q` probes 1, 7, 6, 15 and stops at the tombstone in 7.
        assert_eq!(hash_add(&raw mut ht, owned_key("Q")), Ok(()));
        assert_eq!(keys_in_slot_order(&ht), ["a", "A", "Q"]);
        hash_clear_all(&raw mut ht, 0);
    }
}

/// Growing off the initial array rehashes by the stored hash, which reorders
/// the walk -- the reordering a caller sees across an insertion is part of
/// the behaviour, so the new order is pinned too.
#[test]
fn growth_reorders_the_walk_by_the_bigger_mask() {
    let mut ht = hashtab_T::init();
    unsafe {
        for i in 0..20 {
            assert_eq!(hash_add(&raw mut ht, owned_key(&format!("k{i}"))), Ok(()));
        }
        assert_eq!(ht.size(), 64);
        let mut expected = vec!["k18".to_owned(), "k19".to_owned()];
        expected.extend((0..18).map(|i| format!("k{i}")));
        assert_eq!(keys_in_slot_order(&ht), expected);
        hash_clear_all(&raw mut ht, 0);
    }
}

/// Emptying a grown table takes it back to the initial size. The order it
/// comes back with is *not* the one the same keys had before the growth: a
/// rehash walks the old array in index order and re-probes, so the surviving
/// keys are re-inserted in the grown table's order and collide differently.
/// That is behaviour a caller can see, so it is pinned rather than described.
#[test]
fn shrinking_back_reprobes_in_the_grown_table_order() {
    let mut ht = hashtab_T::init();
    unsafe {
        for key in ["a", "q", "A"] {
            assert_eq!(hash_add(&raw mut ht, owned_key(key)), Ok(()));
        }
        let filler: Vec<CString> = (0..40)
            .map(|i| CString::new(format!("f{i}")).unwrap())
            .collect();
        for key in &filler {
            assert_eq!(
                hash_add(&raw mut ht, owned_key(key.to_str().unwrap())),
                Ok(())
            );
        }
        assert!(ht.size() > HT_INIT_SIZE);
        for key in &filler {
            let hi = hash_find(&ht, key.as_ptr());
            xfree(hi.hi_key as *mut c_void);
            hash_remove(&raw mut ht, hi);
        }
        assert_eq!(ht.size(), HT_INIT_SIZE, "back to the initial size");
        assert_eq!(keys_in_slot_order(&ht), ["A", "q", "a"]);
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
    let mut ht = hashtab_T::init();
    unsafe {
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
