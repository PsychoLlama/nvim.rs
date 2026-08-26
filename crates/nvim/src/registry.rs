//! [`SlotTable`]: the shape an editor registry takes once it is owned Rust.
//!
//! The editor keeps its long-lived objects — timers, channels, and (from
//! phase 23's later slices) buffers, windows and tab pages — in a table
//! keyed by the monotone id the user sees. Upstream spells that as a khash
//! `Map_*` reached through a raw pointer; this is the same table with the
//! raw pointer gone.
//!
//! # The order is part of the contract
//!
//! khash's `Map_*` is not a plain hash map: its keys live in a *dense*
//! array (`set.keys[..set.h.n_keys]`) that every caller iterates directly,
//! so the order objects come back in is insertion order, and deleting one
//! moves the **last** entry into the hole it left (`map::table::delete`).
//! Both halves are visible to users — `timer_info()` answers timers in that
//! order, and a broadcast to every RPC channel goes out in it — so this type
//! reproduces them exactly: a `Vec` of slots plus a `HashMap` from key to
//! slot index, removed with `swap_remove`. It is an index map, not a
//! `HashMap`, for the same reason `memfile::BlockTable` is.
//!
//! # Reentrancy
//!
//! A timer's callback and a channel's callback both enter Vimscript or Lua,
//! and what runs there can start a timer, stop one, open a channel or close
//! one — that is, mutate this table while a caller is walking it. So the
//! type is built so that **no borrow of the table can be held across such a
//! call**:
//!
//! - No method hands out a reference into the table. `V: Copy`, and every
//!   accessor copies the value out, so the borrow ends with the expression
//!   that took it. A registry lives in a [`GlobalCell`], and the
//!   `with`/`with_mut` closure around one of these calls is a leaf: it
//!   cannot re-enter, which is what phase 22's ruling 6 asks for.
//! - Walking is done over a **snapshot** — [`SlotTable::snapshot_keys`] and
//!   [`SlotTable::snapshot_values`] answer an owned `Vec`, so the table is
//!   unborrowed by the time the first callback runs. The snapshot is a
//!   `Vec`, not an iterator, precisely so that it cannot borrow.
//!
//! The cost of the snapshot is one allocation on paths that fire callbacks
//! (teardown, garbage collection, a broadcast, `timer_info()`); the lookups
//! that are actually hot — [`SlotTable::get`] — do not allocate.
//!
//! [`GlobalCell`]: crate::global_cell::GlobalCell

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::hash::{BuildHasherDefault, Hash, Hasher};
use std::collections::HashMap;

/// A registry: values found by id, iterated in khash's order.
///
/// See the module docs for why the order is modelled rather than left to a
/// `HashMap`, and for the reentrancy rule the API enforces.
pub(crate) struct SlotTable<K, V> {
    /// The slots, in iteration order. `swap_remove` keeps it dense.
    slots: Vec<(K, V)>,
    /// Key to its position in `slots`.
    index: HashMap<K, usize, BuildHasherDefault<IdHasher>>,
}

impl<K, V> SlotTable<K, V> {
    /// An empty table. `const`, so a registry can be a `static`.
    pub(crate) const fn new() -> Self {
        SlotTable {
            slots: Vec::new(),
            index: HashMap::with_hasher(BuildHasherDefault::new()),
        }
    }
}

impl<K, V> Default for SlotTable<K, V> {
    fn default() -> Self {
        SlotTable::new()
    }
}

impl<K: Copy + Eq + Hash, V: Copy> SlotTable<K, V> {
    /// The value registered under `key`, if any.
    pub(crate) fn get(&self, key: K) -> Option<V> {
        let i = *self.index.get(&key)?;
        Some(self.slots[i].1)
    }

    /// File `value` under `key`.
    ///
    /// A key that is already present keeps its place in the order and has
    /// its value overwritten, which is what khash's `map_put_ref` followed
    /// by a store does. Registries hand out monotone ids, so in practice
    /// every insert is a fresh key at the end.
    pub(crate) fn insert(&mut self, key: K, value: V) {
        match self.index.get(&key) {
            Some(&i) => self.slots[i].1 = value,
            None => {
                self.index.insert(key, self.slots.len());
                self.slots.push((key, value));
            }
        }
    }

    /// Take `key` out, moving the last slot into the hole it leaves.
    pub(crate) fn remove(&mut self, key: K) -> Option<V> {
        let i = self.index.remove(&key)?;
        let (_, value) = self.slots.swap_remove(i);
        if let Some(&(moved, _)) = self.slots.get(i) {
            self.index.insert(moved, i);
        }
        Some(value)
    }

    /// Every key, in order, as an owned `Vec` — see the module docs on
    /// reentrancy. Callers walk this, never the table.
    pub(crate) fn snapshot_keys(&self) -> Vec<K> {
        self.slots.iter().map(|&(key, _)| key).collect()
    }

    /// Every value, in order, as an owned `Vec`. As [`Self::snapshot_keys`].
    pub(crate) fn snapshot_values(&self) -> Vec<V> {
        self.slots.iter().map(|&(_, value)| value).collect()
    }
}

/// Mixes an id so the top bits — which the hash table probes first — depend
/// on all of it. Registry ids are small and consecutive, which the identity
/// hash spreads badly and SipHash costs too much for. As
/// `memfile::BlockNrHasher`, whose keys have the same shape.
#[derive(Default)]
struct IdHasher(u64);

impl IdHasher {
    fn mix(&mut self, n: u64) {
        // splitmix64's finalizer.
        let mut z = n;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        self.0 = z ^ (z >> 31);
    }
}

impl Hasher for IdHasher {
    fn write_u64(&mut self, n: u64) {
        self.mix(n);
    }

    fn write_i64(&mut self, n: i64) {
        self.mix(n.cast_unsigned());
    }

    fn write_u32(&mut self, n: u32) {
        self.mix(u64::from(n));
    }

    fn write_i32(&mut self, n: i32) {
        self.mix(u64::from(n.cast_unsigned()));
    }

    fn write_usize(&mut self, n: usize) {
        self.mix(n as u64);
    }

    /// The fallback, for a key that is not one of the integers above. No
    /// registry uses one; it is here so the type is a total `Hasher`.
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.mix(self.0.wrapping_mul(31).wrapping_add(u64::from(b)));
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SlotTable;

    /// The table's own view of itself, checked after every mutation: the
    /// index agrees with the slots, and every key is findable.
    fn check(table: &SlotTable<u64, u32>, expected: &[u64]) {
        assert_eq!(table.snapshot_keys(), expected, "iteration order");
        assert_eq!(table.snapshot_values().len(), expected.len());
        for (i, &key) in expected.iter().enumerate() {
            assert_eq!(table.get(key), Some(u32::try_from(key).unwrap() * 10));
            assert_eq!(table.snapshot_values()[i], u32::try_from(key).unwrap() * 10);
        }
    }

    fn filled(keys: &[u64]) -> SlotTable<u64, u32> {
        let mut table = SlotTable::new();
        for &key in keys {
            table.insert(key, u32::try_from(key).unwrap() * 10);
        }
        table
    }

    #[test]
    fn empty_table_answers_nothing() {
        let table: SlotTable<u64, u32> = SlotTable::new();
        assert_eq!(table.get(1), None);
        assert!(table.snapshot_keys().is_empty());
        assert!(table.snapshot_values().is_empty());
    }

    #[test]
    fn insertion_order_is_the_iteration_order() {
        check(&filled(&[10, 20, 30, 40]), &[10, 20, 30, 40]);
    }

    /// khash moves the last key into the hole; `tests/unit/map.rs` asserts
    /// exactly this for `Map_uint64_t_ptr_t`, and the two must agree.
    #[test]
    fn removal_swaps_the_last_slot_into_the_hole() {
        let mut table = filled(&[10, 20, 30, 40]);
        assert_eq!(table.remove(20), Some(200));
        check(&table, &[10, 40, 30]);
        assert_eq!(table.get(20), None);
    }

    #[test]
    fn removing_the_last_slot_leaves_the_rest_alone() {
        let mut table = filled(&[10, 20, 30]);
        assert_eq!(table.remove(30), Some(300));
        check(&table, &[10, 20]);
    }

    #[test]
    fn removing_an_absent_key_is_a_no_op() {
        let mut table = filled(&[10, 20]);
        assert_eq!(table.remove(99), None);
        check(&table, &[10, 20]);
    }

    #[test]
    fn reinsertion_keeps_the_slot_and_overwrites_the_value() {
        let mut table = filled(&[10, 20, 30]);
        table.insert(20, 7);
        assert_eq!(table.snapshot_keys(), [10, 20, 30]);
        assert_eq!(table.get(20), Some(7));
        assert_eq!(table.snapshot_values(), [100, 7, 300]);
    }

    #[test]
    fn a_removed_key_can_come_back_at_the_end() {
        let mut table = filled(&[10, 20, 30]);
        table.remove(10);
        check(&table, &[30, 20]);
        table.insert(10, 100);
        check(&table, &[30, 20, 10]);
    }

    /// Churn the table the way the editor does — ids handed out in order,
    /// entries removed from the middle — and check the index never drifts
    /// from the slots.
    #[test]
    fn churn_keeps_the_index_and_the_slots_in_step() {
        let mut table: SlotTable<u64, u32> = SlotTable::new();
        let mut live: Vec<u64> = Vec::new();
        let mut rng: u64 = 0x9e37_79b9_7f4a_7c15;
        for id in 1..200u64 {
            table.insert(id, u32::try_from(id).unwrap() * 10);
            live.push(id);
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            if live.len() > 3 && rng.is_multiple_of(3) {
                let victim = live[usize::try_from(rng >> 33).unwrap() % live.len()];
                assert_eq!(
                    table.remove(victim),
                    Some(u32::try_from(victim).unwrap() * 10)
                );
                // The model: swap the last live key into the hole.
                let i = live.iter().position(|&k| k == victim).unwrap();
                live.swap_remove(i);
            }
            check(&table, &live);
        }
        for key in live.clone() {
            table.remove(key);
        }
        assert!(table.snapshot_keys().is_empty());
    }
}
