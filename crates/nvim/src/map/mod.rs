#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! What is left of the khash-derived open-addressing tables the editor used
//! to keep everything in: a [`DenseSet`], a [`DenseMap`] over one, and the
//! [`ProbeTable`] the glyph cache shares.
//!
//! Three callers remain, and none of them wants a `HashMap`, because each
//! reads the *dense index* — the position of a key in insertion order:
//!
//! - `tui/attrs.rs`' URL set, whose dense index is the OSC 8 id written to
//!   the terminal;
//! - `grid/schar.rs`' glyph cache, whose keys are a packed run of
//!   NUL-terminated strings and whose "dense index" is therefore a byte
//!   offset — it keeps its own keys and shares only the [`ProbeTable`]
//!   (`map_glyph_cache.rs`);
//! - `marktree/splice.rs`' damage map, walked in insertion order.
//!
//! Everything else the editor keeps is an `IdMap`, an `IdSet` or a
//! `SlotTable` (see [`crate::registry`]). Nothing deletes from the three, so
//! khash's swap-remove half is gone with them.
//!
//! Derived, via upstream's `map.c`, from klib's `khash.h`, Copyright (c)
//! 2008, 2009, 2011 Attractive Chaos, under the MIT license; the notice is
//! reproduced in licenses/klib-LICENSE.txt.

mod policy;
mod table;

use core::borrow::Borrow;

pub use table::{MH_TOMBSTONE, MapKey, ProbeTable, probe};

/// A dense index: the position of a key in insertion order. Upstream's
/// tables count in `uint32_t`, and the OSC 8 id is built from one.
fn dense_index(len: usize) -> u32 {
    u32::try_from(len).expect("a khash table holds fewer than 2^32 keys")
}

/// A set of keys numbered in insertion order: `keys()[i]` is the key whose
/// dense index is `i`, and the index never changes while the set lives.
#[derive(Clone, Debug, Default)]
pub struct DenseSet<K> {
    table: ProbeTable,
    keys: Vec<K>,
}

impl<K> DenseSet<K> {
    /// Nothing in it and nothing allocated; the first insert sizes it.
    pub const fn new() -> Self {
        DenseSet {
            table: ProbeTable::new(),
            keys: Vec::new(),
        }
    }

    /// The keys, by dense index.
    pub fn keys(&self) -> &[K] {
        &self.keys
    }

    /// How many keys the set holds.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Whether the set holds no key.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// The bucket table, for the placement tests.
    #[cfg(test)]
    fn buckets(&self) -> &[u32] {
        self.table.buckets()
    }
}

impl<K: MapKey + Eq> DenseSet<K> {
    /// The dense index of `key`, if it is in the set.
    pub fn get<Q>(&self, key: &Q) -> Option<u32>
    where
        K: Borrow<Q>,
        Q: MapKey + Eq + ?Sized,
    {
        let keys = &self.keys;
        self.table
            .get(key.map_hash(), |k| keys[k as usize].borrow() == key)
    }

    /// Insert `key`, or find it. Answers its dense index and whether it is
    /// new.
    pub fn insert(&mut self, key: K) -> (u32, bool) {
        match self.lookup_or_slot(&key) {
            Ok(dense) => (dense, false),
            Err(idx) => (self.push_at(idx, key), true),
        }
    }

    /// [`DenseSet::insert`] by a borrowed form: `make` builds the owned key
    /// only when `key` is new.
    pub fn insert_with<Q>(&mut self, key: &Q, make: impl FnOnce(&Q) -> K) -> (u32, bool)
    where
        K: Borrow<Q>,
        Q: MapKey + Eq + ?Sized,
    {
        match self.lookup_or_slot(key) {
            Ok(dense) => (dense, false),
            Err(idx) => (self.push_at(idx, make(key)), true),
        }
    }

    /// Make room, then find `key`: `Ok(dense index)` when it is there,
    /// `Err(bucket)` naming where it goes when it is not.
    fn lookup_or_slot<Q>(&mut self, key: &Q) -> Result<u32, u32>
    where
        K: Borrow<Q>,
        Q: MapKey + Eq + ?Sized,
    {
        if self.table.make_room() {
            let keys = &self.keys;
            for (k, stored) in keys.iter().enumerate() {
                self.table.rehash(stored.map_hash(), dense_index(k), |s| {
                    keys[s as usize] == *stored
                });
            }
        }
        let keys = &self.keys;
        let idx = self
            .table
            .find(key.map_hash(), true, |k| keys[k as usize].borrow() == key);
        self.table.entry(idx).ok_or(idx)
    }

    /// Append `key` as the next dense index and point bucket `idx` at it.
    fn push_at(&mut self, idx: u32, key: K) -> u32 {
        let dense = dense_index(self.keys.len());
        self.keys.push(key);
        self.table.occupy(idx, dense);
        dense
    }
}

/// A [`DenseSet`] with a value beside each key, by the same dense index.
#[derive(Clone, Debug, Default)]
pub struct DenseMap<K, V> {
    set: DenseSet<K>,
    values: Vec<V>,
}

impl<K, V> DenseMap<K, V> {
    /// Nothing in it and nothing allocated.
    pub const fn new() -> Self {
        DenseMap {
            set: DenseSet::new(),
            values: Vec::new(),
        }
    }

    /// The keys, by dense index.
    pub fn keys(&self) -> &[K] {
        self.set.keys()
    }

    /// The values, by dense index.
    pub fn values(&self) -> &[V] {
        &self.values
    }

    /// Every entry, in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.set.keys().iter().zip(&self.values)
    }

    /// How many entries the map holds.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the map holds no entry.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl<K: MapKey + Eq, V> DenseMap<K, V> {
    /// The value under `key`, if it is in the map.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.set.get(key).map(|k| &self.values[k as usize])
    }

    /// The value under `key`, inserting `init` first when it is absent —
    /// upstream's `map_put_ref`.
    pub fn get_or_insert(&mut self, key: K, init: V) -> &mut V {
        let (k, new) = self.set.insert(key);
        if new {
            self.values.push(init);
        }
        &mut self.values[k as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Keys are numbered in insertion order, and re-inserting one answers
    /// the number it already has.
    #[test]
    fn dense_indices_are_insertion_order() {
        let mut set = DenseSet::new();
        for (n, key) in [10u64, 20, 30, 40].into_iter().enumerate() {
            assert_eq!(set.insert(key), (dense_index(n), true));
        }
        assert_eq!(set.keys(), &[10, 20, 30, 40]);
        assert_eq!(set.insert(20), (1, false));
        assert_eq!(set.get(&30), Some(2));
        assert_eq!(set.get(&99), None);
        assert_eq!(set.len(), 4);
    }

    /// The bucket placement is upstream's, slot for slot: these are the
    /// tables the khash port built for the same insertions (the `u64` run
    /// grows past 16 buckets and rehashes on the way).
    #[test]
    fn placement_is_upstreams() {
        let mut ints = DenseSet::new();
        for key in [10u64, 20, 30, 40, 7, 1 << 40, 99, 12345, 0, 3, 5, 8, 13, 21] {
            ints.insert(key);
        }
        assert_eq!(
            ints.buckets(),
            &[
                6, 9, 0, 7, 10, 11, 0, 5, 4, 12, 1, 0, 0, 13, 0, 0, 0, 0, 0, 0, 2, 14, 0, 0, 0, 8,
                0, 0, 0, 0, 3, 0
            ]
        );

        let mut urls = DenseSet::<Vec<u8>>::new();
        let names: [&[u8]; 6] = [
            b"https://a",
            b"https://b",
            b"http://neovim.io",
            b"",
            b"x",
            b"file:///tmp/a",
        ];
        for url in names {
            urls.insert_with(url, <[u8]>::to_vec);
        }
        assert_eq!(
            urls.buckets(),
            &[3, 4, 1, 2, 0, 0, 0, 0, 5, 6, 0, 0, 0, 0, 0, 0]
        );
    }

    /// A string key is found by its bytes, and the first copy inserted is
    /// the one kept: `make` runs for a new key only.
    #[test]
    fn string_keys_compare_by_content() {
        let mut set = DenseSet::<Vec<u8>>::new();
        let mut made = 0;
        let mut add = |set: &mut DenseSet<Vec<u8>>, key: &[u8]| {
            set.insert_with(key, |k| {
                made += 1;
                k.to_vec()
            })
        };
        assert_eq!(add(&mut set, b"alpha"), (0, true));
        assert_eq!(add(&mut set, b"alpha".to_vec().as_slice()), (0, false));
        assert_eq!(add(&mut set, b"beta"), (1, true));
        assert_eq!(made, 2);
        assert_eq!(set.get(b"alpha".as_slice()), Some(0));
        assert_eq!(set.get(b"gamma".as_slice()), None);
    }

    /// Past the load factor the bucket table doubles and everything is
    /// rehashed; the dense indices are untouched by that.
    #[test]
    fn growing_keeps_every_index() {
        let mut set = DenseSet::new();
        for key in 0..500u64 {
            set.insert(key * 7 + 1);
        }
        assert_eq!(set.len(), 500);
        assert!(set.buckets().len() >= 1024);
        assert_eq!(set.buckets().len().count_ones(), 1, "a power of two");
        for key in 0..500u32 {
            assert_eq!(set.get(&(u64::from(key) * 7 + 1)), Some(key));
        }
        assert_eq!(set.get(&0), None);
    }

    /// A map's fresh slot starts from the value handed in, and later
    /// lookups answer the same slot.
    #[test]
    fn a_map_slot_starts_from_its_initial_value() {
        let mut map = DenseMap::new();
        *map.get_or_insert(5u64, 0) += 1;
        *map.get_or_insert(9u64, 100) += 1;
        *map.get_or_insert(5u64, 0) += 1;
        assert_eq!(map.get(&5), Some(&2));
        assert_eq!(map.get(&9), Some(&101));
        assert_eq!(map.get(&7), None);
        let entries: Vec<_> = map.iter().map(|(&k, &v)| (k, v)).collect();
        assert_eq!(entries, [(5, 2), (9, 101)]);
    }
}
