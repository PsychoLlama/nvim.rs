#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The open-addressing index behind every `Set_*`/`Map_*` instantiation.
//!
//! Upstream generates this per key type from a macro; c2rust turned that into
//! nine near-identical copies. It is one generic implementation here, with the
//! monomorphic entry points in the parent module.
//!
//! The layout is fixed and shared by all of them: `MapHash::hash` is a table
//! of `n_buckets` slots holding *one-based* indices into a dense `keys` array
//! (`0` means empty, `MH_TOMBSTONE` means deleted), and the values array of a
//! `Map` is indexed the same way. So iteration over a set or map is iteration
//! over `keys[..n_keys]` in insertion order — with the caveat that deleting an
//! entry moves the last key into the hole, which callers that iterate while
//! deleting depend on.
//!
//! The probe sequence, the load factors and the placement are bit-exact with
//! the C: `mh_delete`'s swap and the one-based encoding are observable through
//! the raw arrays, which callers walk directly.
//!
//! Derived, via upstream's `map.c`, from klib's `khash.h`, Copyright (c)
//! 2008, 2009, 2011 Attractive Chaos, under the MIT license; the notice is
//! reproduced in licenses/klib-LICENSE.txt.

use core::ffi::c_void;
use core::slice;

use super::policy;
use crate::memory::{strequal, xcalloc, xfree, xrealloc};
use crate::types::{MHPutStatus, MapHash, String_0, cstr_t, int64_t, ptr_t, uint32_t, uint64_t};

/// The bucket slot of a key that is not in the table.
pub const MH_TOMBSTONE: uint32_t = uint32_t::MAX;

pub const kMHExisting: MHPutStatus = 0;
pub const kMHNewKeyDidFit: MHPutStatus = 1;
pub const kMHNewKeyRealloc: MHPutStatus = 2;

/// What a key type owes the index: a 32-bit hash and an equality test.
///
/// Taken by reference so that a raw-pointer key (`cstr_t`, `ptr_t`) does not
/// make these raw-pointer *parameters* — the implementations for those do read
/// through the pointer, and carry the same obligation the C did: a key handed
/// to the table must stay valid and unchanged while it is in it.
pub trait MapKey: Copy {
    fn map_hash(&self) -> uint32_t;
    fn map_eq(&self, other: &Self) -> bool;
}

/// `h * 31 + byte`, spelled `(h << 5) - h + byte` as upstream does.
fn fold_bytes(bytes: &[u8]) -> uint32_t {
    bytes.iter().fold(0, |h, &b| {
        (h << 5).wrapping_sub(h).wrapping_add(uint32_t::from(b))
    })
}

/// The 64-bit integer mix, applied to `uint64_t`, `int64_t` and pointers.
///
/// The result is upstream's `(uint32_t)` truncation of the mixed word, taken
/// as its low four bytes so that no narrowing cast is spelled.
fn mix64(k: uint64_t) -> uint32_t {
    let [a, b, c, d, ..] = (k >> 33 ^ k ^ k << 11).to_le_bytes();
    uint32_t::from_le_bytes([a, b, c, d])
}

impl MapKey for core::ffi::c_int {
    fn map_hash(&self) -> uint32_t {
        self.cast_unsigned()
    }
    fn map_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl MapKey for uint32_t {
    fn map_hash(&self) -> uint32_t {
        *self
    }
    fn map_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl MapKey for uint64_t {
    fn map_hash(&self) -> uint32_t {
        mix64(*self)
    }
    fn map_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl MapKey for int64_t {
    fn map_hash(&self) -> uint32_t {
        mix64(self.cast_unsigned())
    }
    fn map_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl MapKey for ptr_t {
    fn map_hash(&self) -> uint32_t {
        mix64(self.expose_provenance() as uint64_t)
    }
    fn map_eq(&self, other: &Self) -> bool {
        // Address comparison, as the C's `(uint64_t)a == (uint64_t)b` was.
        self.addr() == other.addr()
    }
}

impl MapKey for cstr_t {
    fn map_hash(&self) -> uint32_t {
        if self.is_null() {
            // Upstream dereferenced unconditionally; `strequal` (which backs
            // `map_eq`) does treat null as a value, so answer for it.
            return 0;
        }
        fold_bytes(unsafe { core::ffi::CStr::from_ptr(*self) }.to_bytes())
    }
    fn map_eq(&self, other: &Self) -> bool {
        unsafe { strequal(*self, *other) }
    }
}

impl MapKey for String_0 {
    fn map_hash(&self) -> uint32_t {
        // An empty `String` carries a null `data` — the C's fold never ran a
        // single iteration for it, so it never noticed.
        if self.is_empty() {
            return 0;
        }
        fold_bytes(unsafe { self.as_bytes() })
    }
    fn map_eq(&self, other: &Self) -> bool {
        // A zero-length String may carry a null `data`, so the length check
        // has to come first — upstream's `memcmp` never sees a null.
        self.len() == other.len()
            && (self.is_empty() || unsafe { self.as_bytes() == other.as_bytes() })
    }
}

/// Discard the bucket table and start again with room for `n_min_buckets`.
/// The keys are untouched: the caller rehashes them.
pub(super) fn realloc(h: &mut MapHash, n_min_buckets: uint32_t) {
    let n_buckets = policy::bucket_count(n_min_buckets);
    // SAFETY: `h.hash` is this module's own allocation, or null.
    let table = unsafe {
        xfree(h.hash.cast::<c_void>());
        xcalloc(n_buckets as usize, size_of::<uint32_t>())
    };
    h.hash = table.cast::<uint32_t>();
    h.size = 0;
    h.n_occupied = 0;
    h.n_buckets = n_buckets;
    h.upper_bound = policy::upper_bound(n_buckets);
}

/// Forget every entry, keeping the bucket table's allocation.
pub(super) fn clear(h: &mut MapHash) {
    if h.hash.is_null() {
        return;
    }
    unsafe { buckets_mut(h) }.fill(0);
    h.size = 0;
    h.n_occupied = 0;
    h.n_keys = 0;
}

/// # Safety
/// `h.hash` must point at `h.n_buckets` slots.
unsafe fn buckets(h: &MapHash) -> &[uint32_t] {
    unsafe { slice::from_raw_parts(h.hash, h.n_buckets as usize) }
}

/// # Safety
/// As [`buckets`], and no other reference to the table may be live.
unsafe fn buckets_mut(h: &mut MapHash) -> &mut [uint32_t] {
    unsafe { slice::from_raw_parts_mut(h.hash, h.n_buckets as usize) }
}

/// The `i`th key of a dense keys array.
///
/// # Safety
/// `keys` must point at more than `i` live keys.
unsafe fn key_at<K: MapKey>(keys: *const K, i: uint32_t) -> K {
    unsafe { *keys.add(i as usize) }
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
/// this walk instead of copying it: the placement is observable through the
/// arrays, so the two must not be able to drift.
pub fn probe(
    buckets: &[uint32_t],
    mask: uint32_t,
    hash: uint32_t,
    put: bool,
    is_stored: impl Fn(uint32_t) -> bool,
) -> uint32_t {
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

/// The bucket `key` belongs in. See [`probe`].
///
/// # Safety
/// `keys` must point at `h.n_keys` live keys and `h.hash` at `h.n_buckets`
/// slots.
pub(super) unsafe fn find_bucket<K: MapKey>(
    h: &MapHash,
    keys: *const K,
    key: &K,
    put: bool,
) -> uint32_t {
    // SAFETY: the caller promises the table and the keys array.
    let buckets = unsafe { buckets(h) };
    probe(buckets, h.n_buckets - 1, key.map_hash(), put, |k| {
        // SAFETY: a live bucket holds a one-based index into `keys`.
        unsafe { key_at(keys, k) }.map_eq(key)
    })
}

/// The dense index of `key`, or `MH_TOMBSTONE` if it is not present.
///
/// # Safety
/// As [`find_bucket`].
pub(super) unsafe fn get<K: MapKey>(h: &MapHash, keys: *const K, key: &K) -> uint32_t {
    if h.n_buckets == 0 {
        return MH_TOMBSTONE;
    }
    // SAFETY: the caller promises the table and the keys array.
    let idx = unsafe { find_bucket(h, keys, key, false) };
    if idx == MH_TOMBSTONE {
        return MH_TOMBSTONE;
    }
    // SAFETY: as above.
    let buckets = unsafe { buckets(h) };
    buckets[idx as usize] - 1
}

/// Re-point every bucket at its key after the table was resized or cleared.
///
/// # Safety
/// As [`find_bucket`], with an all-zero bucket table.
unsafe fn rehash<K: MapKey>(h: &mut MapHash, keys: *const K) {
    for k in 0..h.n_keys {
        // SAFETY: the caller promises `h.n_keys` live keys and the table.
        let idx = unsafe { find_bucket(h, keys, &key_at(keys, k), true) } as usize;
        // SAFETY: as above; no other reference to the table is live here.
        let buckets = unsafe { buckets_mut(h) };
        assert!(
            buckets[idx] == 0,
            "map: rehash landed on an occupied bucket"
        );
        buckets[idx] = k + 1;
    }
    h.size = h.n_keys;
    h.n_occupied = h.size;
}

/// Insert `key`, or find it. Answers its dense index and reports through
/// `status` whether it is new and whether the keys array moved — the caller
/// grows its parallel values array on `kMHNewKeyRealloc`.
///
/// # Safety
/// As [`find_bucket`]; `keys` must be the `keys` field itself, since this may
/// reallocate it.
pub(super) unsafe fn put<K: MapKey>(
    h: &mut MapHash,
    keys: &mut *mut K,
    key: K,
    status: &mut MHPutStatus,
) -> uint32_t {
    if h.n_occupied >= h.upper_bound {
        if policy::should_grow(h.size, h.upper_bound) {
            realloc(h, h.n_buckets + 1);
        } else {
            // Enough of the occupied buckets are tombstones that dropping
            // them alone gets back under the bound.
            // SAFETY: `realloc` and the caller both leave `h.hash` sized.
            unsafe { buckets_mut(h) }.fill(0);
            h.size = 0;
            h.n_occupied = 0;
        }
        // SAFETY: the bucket table is all-zero and the keys are untouched.
        unsafe { rehash(h, *keys) };
    }
    // SAFETY: the caller promises the table and the keys array.
    let (idx, slot) = unsafe {
        let idx = find_bucket(h, *keys, &key, true) as usize;
        (idx, buckets(h)[idx])
    };
    if slot != 0 && slot != MH_TOMBSTONE {
        *status = kMHExisting;
        let pos = slot - 1;
        // SAFETY: a live bucket holds a one-based index into `keys`.
        assert!(
            unsafe { key_at(*keys, pos) }.map_eq(&key),
            "map: bucket points at the wrong key"
        );
        return pos;
    }

    h.size += 1;
    if slot == 0 {
        h.n_occupied += 1;
    }
    let pos = h.n_keys;
    h.n_keys += 1;
    if pos >= h.keys_capacity {
        h.keys_capacity = policy::grown_keys_capacity(h.keys_capacity, policy::MIN_KEYS);
        let capacity = h.keys_capacity as usize * size_of::<K>();
        // SAFETY: `*keys` is this module's own allocation, or null.
        *keys = unsafe { xrealloc(keys.cast::<c_void>(), capacity) }.cast::<K>();
        *status = kMHNewKeyRealloc;
    } else {
        *status = kMHNewKeyDidFit;
    }
    // SAFETY: `pos` is inside the capacity just checked or grown to, and no
    // other reference to the bucket table is live.
    let buckets = unsafe {
        keys.add(pos as usize).write(key);
        buckets_mut(h)
    };
    buckets[idx] = pos + 1;
    pos
}

/// Remove `key`. Answers the dense index it occupied — into which the last
/// key was moved — or `MH_TOMBSTONE` if it was absent. `key` is overwritten
/// with the *stored* key, which the caller may own.
///
/// # Safety
/// As [`find_bucket`].
pub(super) unsafe fn delete<K: MapKey>(h: &mut MapHash, keys: *mut K, key: &mut K) -> uint32_t {
    if h.size == 0 {
        return MH_TOMBSTONE;
    }
    // SAFETY: the caller promises the table and the keys array.
    let idx = unsafe { find_bucket(h, keys, key, false) };
    if idx == MH_TOMBSTONE {
        return MH_TOMBSTONE;
    }
    // SAFETY: as above; no other reference to the table is live.
    let buckets = unsafe { buckets_mut(h) };
    let k = buckets[idx as usize] - 1;
    buckets[idx as usize] = MH_TOMBSTONE;
    h.n_keys -= 1;
    h.size -= 1;
    let last = h.n_keys;
    // SAFETY: `k` came from a live bucket, so it indexes a live key.
    *key = unsafe { key_at(keys, k) };
    if last != k {
        // Move the last key into the hole and re-point its bucket, so the
        // keys array stays dense.
        // SAFETY: `last` was live until the decrement above.
        let (moved, idx2) = unsafe {
            let moved = key_at(keys, last);
            (moved, find_bucket(h, keys, &moved, false))
        };
        // SAFETY: as above; the borrow from `find_bucket` has ended.
        let buckets = unsafe { buckets_mut(h) };
        assert!(
            buckets[idx2 as usize] == last + 1,
            "map: the moved key's bucket is stale"
        );
        buckets[idx2 as usize] = k + 1;
        // SAFETY: `k` indexes inside the (shrunken) keys array.
        unsafe { keys.add(k as usize).write(moved) };
    }
    k
}

/// A `Map`'s value for `key`, or null. `key_alloc`, when given, receives the
/// address of the stored key.
///
/// # Safety
/// As [`find_bucket`]; `values` must be as long as `keys`.
pub(super) unsafe fn map_ref<K: MapKey, V>(
    h: &MapHash,
    keys: *mut K,
    values: *mut V,
    key: K,
    key_alloc: *mut *mut K,
) -> *mut V {
    // SAFETY: the caller promises the table, the keys and the values.
    let k = unsafe { get(h, keys, &key) };
    if k == MH_TOMBSTONE {
        return core::ptr::null_mut();
    }
    // SAFETY: `k` indexes a live key, so it indexes a value too.
    unsafe {
        if !key_alloc.is_null() {
            *key_alloc = keys.add(k as usize);
        }
        values.add(k as usize)
    }
}

/// A `Map`'s slot for `key`, inserting it (with `init` as the value) if it is
/// absent. `new_item`, when given, reports whether it was.
///
/// # Safety
/// As [`put`]; `values` must be the `values` field itself.
pub(super) unsafe fn map_put_ref<K: MapKey, V: Copy>(
    h: &mut MapHash,
    keys: &mut *mut K,
    values: &mut *mut V,
    key: K,
    init: V,
    key_alloc: *mut *mut K,
    new_item: *mut bool,
) -> *mut V {
    let mut status = kMHExisting;
    // SAFETY: the caller promises the table and both arrays.
    let k = unsafe { put(h, keys, key, &mut status) };
    if status != kMHExisting {
        if status == kMHNewKeyRealloc {
            let capacity = h.keys_capacity as usize * size_of::<V>();
            // SAFETY: `*values` is this module's own allocation, or null,
            // and `put` has just grown `keys_capacity` to match.
            *values = unsafe { xrealloc(values.cast::<c_void>(), capacity) }.cast::<V>();
        }
        // SAFETY: `k` is the slot `put` just claimed.
        unsafe { values.add(k as usize).write(init) };
    }
    // SAFETY: the two out-parameters are the caller's, and `k` indexes a
    // live key, so it indexes a value too.
    unsafe {
        if !new_item.is_null() {
            *new_item = status != kMHExisting;
        }
        if !key_alloc.is_null() {
            *key_alloc = keys.add(k as usize);
        }
        values.add(k as usize)
    }
}

/// Remove `key` from a `Map` and answer its value, or `init` if it was absent.
/// `key_alloc`, when given, receives the stored key.
///
/// # Safety
/// As [`delete`]; `values` must be as long as `keys`.
pub(super) unsafe fn map_del<K: MapKey, V: Copy>(
    h: &mut MapHash,
    keys: *mut K,
    values: *mut V,
    mut key: K,
    init: V,
    key_alloc: *mut K,
) -> V {
    // SAFETY: the caller promises the table and both arrays.
    let k = unsafe { delete(h, keys, &mut key) };
    if k == MH_TOMBSTONE {
        return init;
    }
    // SAFETY: `key_alloc` is the caller's, and `k` indexed a live key.
    let value = unsafe {
        if !key_alloc.is_null() {
            *key_alloc = key;
        }
        values.add(k as usize).read()
    };
    if k != h.n_keys {
        // `delete` moved the last key into this hole; move its value too.
        // SAFETY: `h.n_keys` was live until `delete` shrank it.
        unsafe {
            let last = values.add(h.n_keys as usize).read();
            values.add(k as usize).write(last);
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bucket table plus the one-based entries a set with `keys` in it
    /// would hold, built by walking `probe` exactly as `put` does.
    fn build(n_buckets: uint32_t, keys: &[uint32_t]) -> Vec<uint32_t> {
        let mut buckets = vec![0; n_buckets as usize];
        for (k, &key) in keys.iter().enumerate() {
            let idx = probe(&buckets, n_buckets - 1, key.map_hash(), true, |slot| {
                keys[slot as usize] == key
            });
            buckets[idx as usize] = uint32_t::try_from(k).expect("fits") + 1;
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
        let keys: Vec<uint32_t> = (0..12).map(|i| i * 7 + 3).collect();
        let buckets = build(16, &keys);
        for (k, &key) in keys.iter().enumerate() {
            let idx = probe(&buckets, 15, key.map_hash(), false, |slot| {
                keys[slot as usize] == key
            });
            assert_ne!(idx, MH_TOMBSTONE);
            assert_eq!(buckets[idx as usize], uint32_t::try_from(k).unwrap() + 1);
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
            assert_eq!(uint64_t::from(mix64(k)), expected);
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
