//! `HashTab` end to end: the probe sequence, the tombstone bookkeeping and
//! the rehash, driven through the same probe/insert/remove the editor's
//! tables use.
//!
//! The slot numbers here are hand-computed from the C's hash fold, so they
//! fail if the placement ever drifts — callers walk the slots directly and
//! Vim shows the result, so placement is observable behaviour and not an
//! implementation detail.

use neovim::hashtab::{HT_INIT_SIZE, SlotEntry, hash_bytes};
use neovim::types::HashTab;

/// A table entry: a 1-based index into [`Table::keys`], with `0` empty and
/// `u32::MAX` the tombstone -- the shape the syntax keyword tables use.
#[derive(Copy, Clone, PartialEq, Debug)]
struct Key(u32);

impl SlotEntry for Key {
    const EMPTY: Self = Key(0);

    fn is_empty(self) -> bool {
        self.0 == 0
    }

    fn is_removed(self) -> bool {
        self.0 == u32::MAX
    }

    fn removed() -> Self {
        Key(u32::MAX)
    }
}

/// A table of strings over the editor's `HashTab`, driven through the same
/// probe/insert/remove the dictionaries use.
struct Table {
    ht: HashTab<Key>,
    keys: Vec<Vec<u8>>,
}

impl Table {
    fn new() -> Self {
        Table {
            ht: HashTab::init(),
            keys: Vec::new(),
        }
    }

    fn key(&self, entry: Key) -> &[u8] {
        &self.keys[entry.0 as usize - 1]
    }

    /// The slot `key` holds, or the one it belongs in.
    fn find(&self, key: &str) -> neovim::hashtab::Slot<Key> {
        self.ht.probe(hash_bytes(key.as_bytes()), |entry| {
            self.key(entry) == key.as_bytes()
        })
    }

    fn slot_of(&self, key: &str) -> usize {
        self.find(key).index()
    }

    /// Add `key`, refusing a duplicate.
    fn add(&mut self, key: &str) -> Result<(), ()> {
        let hi = self.find(key);
        if hi.is_kept() {
            return Err(());
        }
        self.keys.push(key.as_bytes().to_vec());
        let entry = Key(u32::try_from(self.keys.len()).unwrap());
        self.ht.insert_at(hi, entry, hash_bytes(key.as_bytes()));
        Ok(())
    }

    fn remove(&mut self, key: &str) {
        let hi = self.find(key);
        assert!(hi.is_kept(), "{key} is in the table");
        self.ht.remove_at(hi);
    }

    /// Every live key, in slot order: what `TV_DICT_ITER` -- and so
    /// `keys()`, `values()`, `items()` and every `:echo` of a Dictionary --
    /// hands out.
    fn keys_in_slot_order(&self) -> Vec<String> {
        self.ht
            .items()
            .map(|hi| String::from_utf8(self.key(hi.hi_key).to_vec()).unwrap())
            .collect()
    }
}

/// Hand-computed against the C: `hash_hash("a")` is 97, so on a 16-slot
/// table `a` lands at 97 & 15 == 1. `hash_hash("q")` is 113, which also
/// masks to 1, so `q` takes the second probe: 1 * 5 + 113 + 1 == 119,
/// masked to 7.
#[test]
fn a_collision_lands_on_the_second_probe() {
    let mut t = Table::new();
    assert_eq!(t.ht.size(), HT_INIT_SIZE);
    assert_eq!(t.add("a"), Ok(()));
    assert_eq!(t.add("q"), Ok(()));
    assert_eq!(t.add("q"), Err(()), "a duplicate is refused");
    assert_eq!(t.slot_of("a"), 1);
    assert_eq!(t.slot_of("q"), 7);
    assert_eq!(t.ht.ht_used, 2);
    assert_eq!(t.ht.ht_filled, 2);
}

/// Removing an item leaves a tombstone that a later lookup walks through
/// but a later insertion reuses — that is the whole point of keeping the
/// two counters apart.
#[test]
fn a_removed_key_leaves_a_reusable_tombstone() {
    let mut t = Table::new();
    let _ = t.add("a");
    let _ = t.add("q");
    // Lock the table: removing would otherwise be free to compact the
    // tombstone away immediately.
    t.ht.lock();

    let a = t.slot_of("a");
    t.remove("a");
    assert!(t.ht.slot(a).is_removed());
    assert_eq!(t.ht.ht_used, 1);
    assert_eq!(t.ht.ht_filled, 2, "a tombstone still occupies its slot");

    // `q` is still reachable: the walk crosses the tombstone.
    assert_eq!(t.slot_of("q"), 7);
    // A miss whose walk crossed the tombstone reports the tombstone,
    // not the empty slot that ended the walk.
    assert_eq!(t.slot_of("A"), 1);

    let _ = t.add("A");
    assert_eq!(t.slot_of("A"), 1);
    assert_eq!(t.ht.ht_used, 2);
    assert_eq!(t.ht.ht_filled, 2, "reusing a tombstone fills nothing new");

    t.ht.unlock();
}

/// Past the small array's load factor the table moves to the heap, keeps
/// every key findable, and reports `filled == used` again because the
/// rehash drops the tombstones.
#[test]
fn growing_off_the_small_array_keeps_every_key() {
    let mut t = Table::new();
    let keys: Vec<String> = (0..64).map(|i| format!("key{i}")).collect();
    for key in &keys {
        assert_eq!(t.add(key), Ok(()));
    }
    assert_eq!(t.ht.ht_used, 64);
    assert_eq!(t.ht.ht_filled, 64);
    assert!(t.ht.size() >= 256, "size {}", t.ht.size());
    for key in &keys {
        let hi = t.find(key);
        assert!(hi.is_kept());
        assert_eq!(t.key(hi.hi_key), key.as_bytes());
    }

    // Every slot is either empty, a tombstone, or a key we put there;
    // exactly `ht_used` of them are live.
    assert_eq!(t.ht.items().count(), 64);

    for key in keys.iter().take(50) {
        t.remove(key);
    }
    assert_eq!(t.ht.ht_used, 14);
    for key in keys.iter().take(50) {
        assert!(!t.find(key).is_kept());
    }
    for key in keys.iter().skip(50) {
        assert!(t.find(key).is_kept());
    }
}

/// The iteration order is Vim-visible, so it is pinned here as a literal
/// rather than merely asserted to be stable. The slots are hand-computed
/// from the fold (`hash * 101 + byte`, seeded with the first byte) and the
/// probe sequence: on a 16-slot table `a`(97), `q`(113), `A`(65) and `Q`(81)
/// all mask to 1 and walk on to 7, 6 and 15; `b`(98) and `r`(114) mask to 2
/// and `r` walks on to 13.
#[test]
fn iteration_visits_slots_in_index_order() {
    let mut t = Table::new();
    for key in ["a", "q", "A", "Q", "b", "r"] {
        assert_eq!(t.add(key), Ok(()));
    }
    assert_eq!(t.keys_in_slot_order(), ["a", "b", "A", "q", "r", "Q"]);
}

/// A tombstone is reused by the first insertion whose probe sequence crosses
/// it, which puts the new key *earlier* than the empty slot that would have
/// ended the walk.
#[test]
fn a_reused_tombstone_keeps_its_slot_in_the_order() {
    let mut t = Table::new();
    for key in ["a", "q", "A"] {
        assert_eq!(t.add(key), Ok(()));
    }
    t.remove("q");
    // `Q` probes 1, 7, 6, 15 and stops at the tombstone in 7.
    assert_eq!(t.add("Q"), Ok(()));
    assert_eq!(t.keys_in_slot_order(), ["a", "A", "Q"]);
}

/// Growing off the initial array rehashes by the stored hash, which reorders
/// the walk -- the reordering a caller sees across an insertion is part of
/// the behaviour, so the new order is pinned too.
#[test]
fn growth_reorders_the_walk_by_the_bigger_mask() {
    let mut t = Table::new();
    for i in 0..20 {
        assert_eq!(t.add(&format!("k{i}")), Ok(()));
    }
    assert_eq!(t.ht.size(), 64);
    let mut expected = vec!["k18".to_owned(), "k19".to_owned()];
    expected.extend((0..18).map(|i| format!("k{i}")));
    assert_eq!(t.keys_in_slot_order(), expected);
}

/// Emptying a grown table takes it back to the initial size. The order it
/// comes back with is *not* the one the same keys had before the growth: a
/// rehash walks the old array in index order and re-probes, so the surviving
/// keys are re-inserted in the grown table's order and collide differently.
/// That is behaviour a caller can see, so it is pinned rather than described.
#[test]
fn shrinking_back_reprobes_in_the_grown_table_order() {
    let mut t = Table::new();
    for key in ["a", "q", "A"] {
        assert_eq!(t.add(key), Ok(()));
    }
    let filler: Vec<String> = (0..40).map(|i| format!("f{i}")).collect();
    for key in &filler {
        assert_eq!(t.add(key), Ok(()));
    }
    assert!(t.ht.size() > HT_INIT_SIZE);
    for key in &filler {
        t.remove(key);
    }
    assert_eq!(t.ht.size(), HT_INIT_SIZE, "back to the initial size");
    assert_eq!(t.keys_in_slot_order(), ["A", "q", "a"]);
}
