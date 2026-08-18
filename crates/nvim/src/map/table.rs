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
        (h << 5).wrapping_sub(h).wrapping_add(b as uint32_t)
    })
}

/// The 64-bit integer mix, applied to `uint64_t`, `int64_t` and pointers.
fn mix64(k: uint64_t) -> uint32_t {
    (k >> 33 ^ k ^ k << 11) as uint32_t
}

/// Hash a value by its bytes, padding included — which is what `memcmp`-based
/// equality compares, so the two agree.
fn fold_value<T>(value: &T) -> uint32_t {
    fold_bytes(unsafe { slice::from_raw_parts((value as *const T).cast::<u8>(), size_of::<T>()) })
}

fn bytes_eq<T>(a: &T, b: &T) -> bool {
    unsafe {
        slice::from_raw_parts((a as *const T).cast::<u8>(), size_of::<T>())
            == slice::from_raw_parts((b as *const T).cast::<u8>(), size_of::<T>())
    }
}

impl MapKey for core::ffi::c_int {
    fn map_hash(&self) -> uint32_t {
        *self as uint32_t
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
        mix64(*self as uint64_t)
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
        if self.size == 0 {
            return 0;
        }
        fold_bytes(unsafe { slice::from_raw_parts(self.data.cast::<u8>(), self.size) })
    }
    fn map_eq(&self, other: &Self) -> bool {
        // A zero-length String may carry a null `data`, so the length check
        // has to come first — upstream's `memcmp` never sees a null.
        self.size == other.size
            && (self.size == 0
                || unsafe {
                    slice::from_raw_parts(self.data.cast::<u8>(), self.size)
                        == slice::from_raw_parts(other.data.cast::<u8>(), other.size)
                })
    }
}

/// Discard the bucket table and start again with room for `n_min_buckets`.
/// The keys are untouched: the caller rehashes them.
pub fn realloc(h: &mut MapHash, n_min_buckets: uint32_t) {
    let n_buckets = policy::bucket_count(n_min_buckets);
    unsafe {
        xfree(h.hash as *mut c_void);
        h.hash = xcalloc(n_buckets as usize, size_of::<uint32_t>()) as *mut uint32_t;
    }
    h.size = 0;
    h.n_occupied = 0;
    h.n_buckets = n_buckets;
    h.upper_bound = policy::upper_bound(n_buckets);
}

/// Forget every entry, keeping the bucket table's allocation.
pub fn clear(h: &mut MapHash) {
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
    slice::from_raw_parts(h.hash, h.n_buckets as usize)
}

/// # Safety
/// As [`buckets`], and no other reference to the table may be live.
unsafe fn buckets_mut(h: &mut MapHash) -> &mut [uint32_t] {
    slice::from_raw_parts_mut(h.hash, h.n_buckets as usize)
}

/// The bucket `key` belongs in.
///
/// With `put`, a miss answers the slot to write: the first tombstone the walk
/// crossed, or the empty slot that ended it. Without, a miss answers
/// `MH_TOMBSTONE`.
///
/// # Safety
/// `keys` must point at `h.n_keys` live keys and `h.hash` at `h.n_buckets`
/// slots.
pub unsafe fn find_bucket<K: MapKey>(h: &MapHash, keys: *const K, key: &K, put: bool) -> uint32_t {
    let buckets = buckets(h);
    let mask = h.n_buckets - 1;
    let mut i = key.map_hash() & mask;
    let last = i;
    let mut site = if put { last } else { MH_TOMBSTONE };
    let mut step = 0;
    while buckets[i as usize] != 0 {
        if buckets[i as usize] == MH_TOMBSTONE {
            if site == last {
                site = i;
            }
        } else if (*keys.add(buckets[i as usize] as usize - 1)).map_eq(key) {
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

/// The dense index of `key`, or `MH_TOMBSTONE` if it is not present.
///
/// # Safety
/// As [`find_bucket`].
pub unsafe fn get<K: MapKey>(h: &MapHash, keys: *const K, key: &K) -> uint32_t {
    if h.n_buckets == 0 {
        return MH_TOMBSTONE;
    }
    let idx = find_bucket(h, keys, key, false);
    if idx == MH_TOMBSTONE {
        return MH_TOMBSTONE;
    }
    let buckets = buckets(h);
    buckets[idx as usize] - 1
}

/// Re-point every bucket at its key after the table was resized or cleared.
///
/// # Safety
/// As [`find_bucket`], with an all-zero bucket table.
unsafe fn rehash<K: MapKey>(h: &mut MapHash, keys: *const K) {
    for k in 0..h.n_keys {
        let key = &*keys.add(k as usize);
        let idx = find_bucket(h, keys, key, true) as usize;
        let buckets = buckets_mut(h);
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
pub unsafe fn put<K: MapKey>(
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
            buckets_mut(h).fill(0);
            h.size = 0;
            h.n_occupied = 0;
        }
        rehash(h, *keys);
    }
    let idx = find_bucket(h, *keys, &key, true) as usize;
    let slot = buckets(h)[idx];
    if slot != 0 && slot != MH_TOMBSTONE {
        *status = kMHExisting;
        let pos = slot - 1;
        assert!(
            (*keys.add(pos as usize)).map_eq(&key),
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
        *keys = xrealloc(
            *keys as *mut c_void,
            h.keys_capacity as usize * size_of::<K>(),
        ) as *mut K;
        *status = kMHNewKeyRealloc;
    } else {
        *status = kMHNewKeyDidFit;
    }
    keys.add(pos as usize).write(key);
    let buckets = buckets_mut(h);
    buckets[idx] = pos + 1;
    pos
}

/// Remove `key`. Answers the dense index it occupied — into which the last
/// key was moved — or `MH_TOMBSTONE` if it was absent. `key` is overwritten
/// with the *stored* key, which the caller may own.
///
/// # Safety
/// As [`find_bucket`].
pub unsafe fn delete<K: MapKey>(h: &mut MapHash, keys: *mut K, key: &mut K) -> uint32_t {
    if h.size == 0 {
        return MH_TOMBSTONE;
    }
    let idx = find_bucket(h, keys, key, false);
    if idx == MH_TOMBSTONE {
        return MH_TOMBSTONE;
    }
    let k = {
        let buckets = buckets_mut(h);
        let k = buckets[idx as usize] - 1;
        buckets[idx as usize] = MH_TOMBSTONE;
        k
    };
    h.n_keys -= 1;
    h.size -= 1;
    let last = h.n_keys;
    *key = keys.add(k as usize).read();
    if last != k {
        // Move the last key into the hole and re-point its bucket, so the
        // keys array stays dense.
        let moved = &*keys.add(last as usize);
        let idx2 = find_bucket(h, keys, moved, false);
        let buckets = buckets_mut(h);
        assert!(
            buckets[idx2 as usize] == last + 1,
            "map: the moved key's bucket is stale"
        );
        buckets[idx2 as usize] = k + 1;
        keys.add(k as usize).write(*moved);
    }
    k
}

/// A `Map`'s value for `key`, or null. `key_alloc`, when given, receives the
/// address of the stored key.
///
/// # Safety
/// As [`find_bucket`]; `values` must be as long as `keys`.
pub unsafe fn map_ref<K: MapKey, V>(
    h: &MapHash,
    keys: *mut K,
    values: *mut V,
    key: K,
    key_alloc: *mut *mut K,
) -> *mut V {
    let k = get(h, keys, &key);
    if k == MH_TOMBSTONE {
        return core::ptr::null_mut();
    }
    if !key_alloc.is_null() {
        *key_alloc = keys.add(k as usize);
    }
    values.add(k as usize)
}

/// A `Map`'s slot for `key`, inserting it (with `init` as the value) if it is
/// absent. `new_item`, when given, reports whether it was.
///
/// # Safety
/// As [`put`]; `values` must be the `values` field itself.
pub unsafe fn map_put_ref<K: MapKey, V: Copy>(
    h: &mut MapHash,
    keys: &mut *mut K,
    values: &mut *mut V,
    key: K,
    init: V,
    key_alloc: *mut *mut K,
    new_item: *mut bool,
) -> *mut V {
    let mut status = kMHExisting;
    let k = put(h, keys, key, &mut status);
    if status != kMHExisting {
        if status == kMHNewKeyRealloc {
            *values = xrealloc(
                *values as *mut c_void,
                h.keys_capacity as usize * size_of::<V>(),
            ) as *mut V;
        }
        values.add(k as usize).write(init);
    }
    if !new_item.is_null() {
        *new_item = status != kMHExisting;
    }
    if !key_alloc.is_null() {
        *key_alloc = keys.add(k as usize);
    }
    values.add(k as usize)
}

/// Remove `key` from a `Map` and answer its value, or `init` if it was absent.
/// `key_alloc`, when given, receives the stored key.
///
/// # Safety
/// As [`delete`]; `values` must be as long as `keys`.
pub unsafe fn map_del<K: MapKey, V: Copy>(
    h: &mut MapHash,
    keys: *mut K,
    values: *mut V,
    mut key: K,
    init: V,
    key_alloc: *mut K,
) -> V {
    let k = delete(h, keys, &mut key);
    if k == MH_TOMBSTONE {
        return init;
    }
    if !key_alloc.is_null() {
        *key_alloc = key;
    }
    let value = values.add(k as usize).read();
    if k != h.n_keys {
        // `delete` moved the last key into this hole; move its value too.
        values
            .add(k as usize)
            .write(values.add(h.n_keys as usize).read());
    }
    value
}
