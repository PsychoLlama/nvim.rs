#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The open-addressing index behind [`DenseSet`](super::DenseSet),
//! [`DenseMap`](super::DenseMap) and the glyph cache.
//!
//! [`ProbeTable`] is a table of `n_buckets` slots holding *one-based*
//! indices into a dense keys array its owner keeps (`0` means empty,
//! [`MH_TOMBSTONE`] means deleted). The dense index of a key is its position
//! in that array — insertion order — so it does not depend on where the key
//! landed here; the probe sequence, the load factors and the placement are
//! still upstream's bit for bit, which the tests below and in the parent
//! pin.
//!
//! Derived, via upstream's `map.c`, from klib's `khash.h`, Copyright (c)
//! 2008, 2009, 2011 Attractive Chaos, under the MIT license; the notice is
//! reproduced in licenses/klib-LICENSE.txt.

use super::policy;

/// A bucket slot that held a key that was deleted; also what a lookup that
/// misses answers from [`probe`].
pub const MH_TOMBSTONE: u32 = u32::MAX;

/// What a key type owes the index: upstream's 32-bit hash. Equality is the
/// type's own `Eq`.
pub trait MapKey {
    fn map_hash(&self) -> u32;
}

/// `h * 31 + byte`, spelled `(h << 5) - h + byte` as upstream does.
fn fold_bytes(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0, |h, &b| {
        (h << 5).wrapping_sub(h).wrapping_add(u32::from(b))
    })
}

/// The 64-bit integer mix, applied to `uint64_t`, `int64_t` and pointers.
///
/// The result is upstream's `(uint32_t)` truncation of the mixed word, taken
/// as its low four bytes so that no narrowing cast is spelled.
fn mix64(k: u64) -> u32 {
    let [a, b, c, d, ..] = (k >> 33 ^ k ^ k << 11).to_le_bytes();
    u32::from_le_bytes([a, b, c, d])
}

impl MapKey for u32 {
    fn map_hash(&self) -> u32 {
        *self
    }
}

impl MapKey for u64 {
    fn map_hash(&self) -> u32 {
        mix64(*self)
    }
}

/// A string key: upstream's `cstr_t` and `String` hash the same bytes the
/// same way, and the empty key folds to zero.
impl MapKey for [u8] {
    fn map_hash(&self) -> u32 {
        fold_bytes(self)
    }
}

impl MapKey for Vec<u8> {
    fn map_hash(&self) -> u32 {
        fold_bytes(self)
    }
}

/// The probe sequence: quadratic, `i += ++step`, masked to the table.
///
/// With `put`, a miss answers the slot to write: the first tombstone the walk
/// crossed, or the empty slot that ended it. Without, a miss answers
/// `MH_TOMBSTONE`.
///
/// `is_stored` is handed the *dense index* a live bucket names — the entry
/// minus one — and answers whether that is the key being looked for. Pulling
/// the comparison out is what lets the glyph cache, whose keys array is a
/// packed run of NUL-terminated strings rather than fixed-size entries, share
/// this walk instead of copying it.
pub fn probe(
    buckets: &[u32],
    mask: u32,
    hash: u32,
    put: bool,
    is_stored: impl Fn(u32) -> bool,
) -> u32 {
    let mut i = hash & mask;
    let last = i;
    let mut site = if put { last } else { MH_TOMBSTONE };
    let mut step = 0;
    while buckets[i as usize] != 0 {
        if buckets[i as usize] == MH_TOMBSTONE {
            if site == last {
                site = i;
            }
        } else if is_stored(buckets[i as usize] - 1) {
            return i;
        }
        step += 1;
        i = i.wrapping_add(step) & mask;
        assert!(i != last, "map: the bucket table is full");
    }
    if site == last {
        site = i;
    }
    site
}

/// The bucket half of a khash table: the slots and the counts that decide
/// when it grows. The keys array is the owner's.
#[derive(Clone, Debug, Default)]
pub struct ProbeTable {
    /// `n_buckets` slots, a power of two, or none before the first insert.
    buckets: Vec<u32>,
    /// Live entries.
    size: u32,
    /// Live entries plus tombstones.
    n_occupied: u32,
    /// How many slots may be occupied before the table is resized.
    upper_bound: u32,
}

impl ProbeTable {
    /// No slots: the first insert sizes the table.
    pub const fn new() -> Self {
        ProbeTable {
            buckets: Vec::new(),
            size: 0,
            n_occupied: 0,
            upper_bound: 0,
        }
    }

    /// The slots, each 0, a tombstone, or a dense index plus one.
    pub fn buckets(&self) -> &[u32] {
        &self.buckets
    }

    /// Live entries.
    pub fn size(&self) -> u32 {
        self.size
    }

    fn n_buckets(&self) -> u32 {
        u32::try_from(self.buckets.len()).expect("a bucket count fits a u32")
    }

    /// The bucket a key hashing to `hash` is in (or, with `put`, belongs
    /// in). See [`probe`]. The table must have been sized.
    pub fn find(&self, hash: u32, put: bool, is_stored: impl Fn(u32) -> bool) -> u32 {
        probe(&self.buckets, self.n_buckets() - 1, hash, put, is_stored)
    }

    /// The dense index a lookup for `hash` finds, if any.
    pub fn get(&self, hash: u32, is_stored: impl Fn(u32) -> bool) -> Option<u32> {
        if self.buckets.is_empty() {
            return None;
        }
        let idx = self.find(hash, false, is_stored);
        (idx != MH_TOMBSTONE).then(|| self.buckets[idx as usize] - 1)
    }

    /// The entry in bucket `idx`: `Some(dense index)` when it is live.
    pub fn entry(&self, idx: u32) -> Option<u32> {
        let slot = self.buckets[idx as usize];
        (slot != 0 && slot != MH_TOMBSTONE).then(|| slot - 1)
    }

    /// Before an insert: when the table is at its bound, grow it. `true` when
    /// the owner must now [`rehash`](Self::rehash) every key into the
    /// all-zero table.
    ///
    /// Nothing deletes from these tables, so there are no tombstones and the
    /// occupied count is the live count: upstream's "shed the tombstones
    /// instead of growing" branch (`should_grow`) can never be taken.
    pub fn make_room(&mut self) -> bool {
        if self.n_occupied < self.upper_bound {
            return false;
        }
        self.resize(self.n_buckets() + 1);
        true
    }

    /// Discard the slots and start again with room for `n_min_buckets`.
    fn resize(&mut self, n_min_buckets: u32) {
        let n_buckets = policy::bucket_count(n_min_buckets);
        self.buckets.clear();
        self.buckets.resize(n_buckets as usize, 0);
        self.size = 0;
        self.n_occupied = 0;
        self.upper_bound = policy::upper_bound(n_buckets);
    }

    /// Re-point a bucket at dense index `dense` after [`make_room`]
    /// (Self::make_room) emptied the table: the next key of the rehash walk.
    pub fn rehash(&mut self, hash: u32, dense: u32, is_stored: impl Fn(u32) -> bool) {
        let idx = self.find(hash, true, is_stored) as usize;
        assert!(
            self.buckets[idx] == 0,
            "map: rehash landed on an occupied bucket"
        );
        self.buckets[idx] = dense + 1;
        self.size += 1;
        self.n_occupied += 1;
    }

    /// Point bucket `idx` — a miss [`find`](Self::find) answered with `put`
    /// — at the new dense index `dense`.
    pub fn occupy(&mut self, idx: u32, dense: u32) {
        let slot = &mut self.buckets[idx as usize];
        debug_assert!(*slot == 0 || *slot == MH_TOMBSTONE, "bucket is free");
        if *slot == 0 {
            self.n_occupied += 1;
        }
        self.size += 1;
        *slot = dense + 1;
    }

    /// Forget every entry, keeping the slots' allocation.
    pub fn clear(&mut self) {
        self.buckets.fill(0);
        self.size = 0;
        self.n_occupied = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bucket table plus the one-based entries a set with `keys` in it
    /// would hold, built by walking `probe` exactly as `put` does.
    fn build(n_buckets: u32, keys: &[u32]) -> Vec<u32> {
        let mut buckets = vec![0; n_buckets as usize];
        for (k, &key) in keys.iter().enumerate() {
            let idx = probe(&buckets, n_buckets - 1, key.map_hash(), true, |slot| {
                keys[slot as usize] == key
            });
            buckets[idx as usize] = u32::try_from(k).expect("fits") + 1;
        }
        buckets
    }

    /// The empty table answers the home slot, and a lookup that misses says
    /// so rather than pointing at one.
    #[test]
    fn an_empty_table_answers_the_home_slot() {
        let buckets = vec![0; 16];
        assert_eq!(probe(&buckets, 15, 33, true, |_| false), 1);
        assert_eq!(probe(&buckets, 15, 33, false, |_| false), MH_TOMBSTONE);
    }

    /// The walk is `i += ++step`: 1, 3, 6, 10 away from home. Filling those
    /// four slots and asking again lands on the fifth, 15 away.
    #[test]
    fn the_step_grows_by_one_each_time() {
        let mut buckets = vec![0; 32];
        for offset in [0, 1, 3, 6, 10] {
            buckets[(1 + offset) % 32] = 99;
        }
        assert_eq!(probe(&buckets, 31, 33, true, |_| false), 16);
    }

    /// Every key finds itself again, and nothing else does.
    #[test]
    fn every_key_is_found_where_it_was_put() {
        let keys: Vec<u32> = (0..12).map(|i| i * 7 + 3).collect();
        let buckets = build(16, &keys);
        for (k, &key) in keys.iter().enumerate() {
            let idx = probe(&buckets, 15, key.map_hash(), false, |slot| {
                keys[slot as usize] == key
            });
            assert_ne!(idx, MH_TOMBSTONE);
            assert_eq!(buckets[idx as usize], u32::try_from(k).unwrap() + 1);
        }
        let absent = 1000;
        let idx = probe(&buckets, 15, absent.map_hash(), false, |slot| {
            keys[slot as usize] == absent
        });
        assert_eq!(idx, MH_TOMBSTONE);
    }

    /// A `put` walk reuses the first tombstone it crossed rather than the
    /// empty slot that ends the walk — but only after checking the whole
    /// chain, so a key that is still there is still found.
    #[test]
    fn a_put_reuses_the_first_tombstone_but_finds_the_key_first() {
        let mut buckets = vec![0; 16];
        // Home of hash 1 is slot 1; the chain is 1, 2, 4, 7.
        buckets[1] = 6;
        buckets[2] = MH_TOMBSTONE;
        assert_eq!(probe(&buckets, 15, 1, true, |_| false), 2);
        assert_eq!(probe(&buckets, 15, 1, true, |slot| slot == 5), 1);
        // Without `put` the miss is reported rather than a slot.
        assert_eq!(probe(&buckets, 15, 1, false, |_| false), MH_TOMBSTONE);
    }

    /// `site == last` doubles as "no tombstone yet", so a tombstone sitting
    /// on the *home* slot is not recognised as one and the walk keeps the
    /// empty slot it ended on. Upstream's quirk, and placement is observable
    /// through the raw arrays, so it is kept deliberately.
    #[test]
    fn a_tombstone_on_the_home_slot_is_not_reused() {
        let mut buckets = vec![0; 16];
        buckets[1] = MH_TOMBSTONE;
        buckets[2] = 6;
        assert_eq!(probe(&buckets, 15, 1, true, |_| false), 4);
    }

    /// A table with no empty slot left would loop forever; the walk stops
    /// itself instead.
    #[test]
    #[should_panic(expected = "the bucket table is full")]
    fn a_full_table_is_caught_rather_than_spun_on() {
        let buckets = vec![7; 16];
        probe(&buckets, 15, 0, true, |_| false);
    }

    /// `mix64` is upstream's `(k >> 33 ^ k ^ k << 11)`, truncated. The
    /// spelling changed to drop the cast; the numbers may not.
    #[test]
    fn the_integer_mix_is_upstreams() {
        for k in [0u64, 1, 0xffff_ffff, 0x1234_5678_9abc_def0, u64::MAX] {
            let expected = (k >> 33 ^ k ^ k << 11) & 0xffff_ffff;
            assert_eq!(u64::from(mix64(k)), expected);
        }
    }

    /// `h * 31 + byte`, and the empty slice hashes to 0.
    #[test]
    fn the_byte_fold_is_upstreams() {
        assert_eq!(fold_bytes(b""), 0);
        assert_eq!(fold_bytes(b"a"), 97);
        assert_eq!(fold_bytes(b"ab"), 97 * 31 + 98);
        // It wraps rather than overflowing: 40 bytes is well past 32 bits.
        let long = [0xffu8; 40];
        let expected = long
            .iter()
            .fold(0u32, |h, &b| h.wrapping_mul(31).wrapping_add(u32::from(b)));
        assert_eq!(fold_bytes(&long), expected);
    }
}
