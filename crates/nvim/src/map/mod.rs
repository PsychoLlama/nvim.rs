#![deny(unsafe_op_in_unsafe_fn)]

//! `Set_*` and `Map_*`: what is left of the khash-derived open-addressing
//! tables the editor used to keep everything in.
//!
//! Three callers remain, and none of them wants a `HashMap`:
//!
//! - `tui/attrs.rs`' URL set, whose **dense index is the OSC 8 id** written
//!   to the terminal;
//! - `grid/schar.rs`' glyph cache, whose keys array is a packed run of
//!   NUL-terminated strings rather than fixed-size entries — it keeps its own
//!   array and shares only [`probe`] and [`mh_realloc`] (`map_glyph_cache.rs`);
//! - `marktree/splice.rs`' damage map, whose walk is open-coded over the same
//!   dense array.
//!
//! Everything else the editor keeps is an `IdMap`, an `IdSet` or a
//! `SlotTable` (see [`crate::registry`]), so the surface here is three
//! entry points plus the sizing and probe primitives. Nothing deletes from
//! the three, so `table`'s swap-remove half is gone with them.
//!
//! Their internals are public because callers iterate
//! `set.keys[..set.h.n_keys]` and index `map.values` with what these entry
//! points answer.
//!
//! # Safety
//! Every entry point here takes a raw pointer to a live, initialized
//! `Set_*`/`Map_*` — an all-zero one counts as initialized and empty. Keys of
//! pointer type (`cstr_t`, `String`) must stay valid and unchanged for as
//! long as they are in the table.
//!
//! Derived, via upstream's `map.c`, from klib's `khash.h`, Copyright (c)
//! 2008, 2009, 2011 Attractive Chaos, under the MIT license; the notice is
//! reproduced in licenses/klib-LICENSE.txt.

mod policy;
mod table;

use crate::types::{
    MHPutStatus, MTDamage, MTDamagePair, MTNode, Map_uint64_t_MTDamagePair, MapHash, Set_cstr_t,
    Set_uint64_t, cstr_t, uint32_t, uint64_t,
};
pub use policy::grown_keys_capacity;
pub use table::{MH_TOMBSTONE, MapKey, kMHExisting, kMHNewKeyDidFit, kMHNewKeyRealloc, probe};

pub const MTDAMAGE_INIT: MTDamage = MTDamage {
    old: ::core::ptr::null_mut::<MTNode>(),
    new: ::core::ptr::null_mut::<MTNode>(),
    old_i: 0,
    new_i: 0,
};
pub const MTDAMAGE_PAIR_INIT: MTDamagePair = MTDamagePair {
    start: MTDAMAGE_INIT,
    end: MTDAMAGE_INIT,
};

/// The two fields every `Set_*` is: the bucket index, and the dense keys
/// array its one-based entries point into. Handed out as disjoint borrows,
/// because [`table::put`] reseats the keys pointer while it holds the index.
trait SetFields {
    type Key: MapKey;
    fn fields(&mut self) -> (&mut MapHash, &mut *mut Self::Key);
}

/// A `Map_*` is a `Set_*` plus a values array indexed by the same dense
/// index. Same disjointness requirement, one field wider.
trait MapFields {
    type Key: MapKey;
    type Value: Copy;
    fn fields(&mut self) -> (&mut MapHash, &mut *mut Self::Key, &mut *mut Self::Value);
}

macro_rules! set_fields {
    ($($set:ty => $key:ty,)*) => {$(
        impl SetFields for $set {
            type Key = $key;
            fn fields(&mut self) -> (&mut MapHash, &mut *mut $key) {
                (&mut self.h, &mut self.keys)
            }
        }
    )*};
}

macro_rules! map_fields {
    ($($map:ty => $key:ty, $value:ty,)*) => {$(
        impl MapFields for $map {
            type Key = $key;
            type Value = $value;
            fn fields(&mut self) -> (&mut MapHash, &mut *mut $key, &mut *mut $value) {
                (&mut self.set.h, &mut self.set.keys, &mut self.values)
            }
        }
    )*};
}

set_fields! {
    Set_cstr_t => cstr_t,
    Set_uint64_t => uint64_t,
}

map_fields! {
    Map_uint64_t_MTDamagePair => uint64_t, MTDamagePair,
}

// The three generic entry points left. Every monomorph below is one call to
// one of them: the deref of the caller's pointer and the call are the whole
// of the unchecked work, and it is written down once here rather than three
// times.

/// # Safety
/// `set` must point at a live `Set_*`, as the module note says.
unsafe fn set_get<S: SetFields>(set: *mut S, key: &S::Key) -> uint32_t {
    let (h, keys) = unsafe { &mut *set }.fields();
    unsafe { table::get(h, *keys, key) }
}

/// # Safety
/// As [`set_get`]; `status` must be writable.
unsafe fn set_put<S: SetFields>(set: *mut S, key: S::Key, status: *mut MHPutStatus) -> uint32_t {
    let (h, keys) = unsafe { &mut *set }.fields();
    unsafe { table::put(h, keys, key, &mut *status) }
}

/// # Safety
/// `map` must point at a live `Map_*`; the out-parameters are null or
/// writable.
unsafe fn map_put_ref<M: MapFields>(
    map: *mut M,
    key: M::Key,
    init: M::Value,
    key_alloc: *mut *mut M::Key,
    new_item: *mut bool,
) -> *mut M::Value {
    let (h, keys, values) = unsafe { &mut *map }.fields();
    unsafe { table::map_put_ref(h, keys, values, key, init, key_alloc, new_item) }
}

/// Discard a bucket table and start again with room for `n_min_buckets`. The
/// caller rehashes: the glyph cache keeps its own keys array and calls this
/// directly.
///
/// # Safety
/// `h` must point at a live `MapHash`.
pub unsafe fn mh_realloc(h: *mut MapHash, n_min_buckets: uint32_t) {
    table::realloc(unsafe { &mut *h }, n_min_buckets);
}

/// Forget every entry, keeping the bucket table's allocation.
///
/// # Safety
/// `h` must point at a live `MapHash`.
pub unsafe fn mh_clear(h: *mut MapHash) {
    table::clear(unsafe { &mut *h });
}

// The dense index of a key in a set, or `MH_TOMBSTONE`. See `table::get`.

/// # Safety
/// `set` must point at a live `Set_cstr_t`.
pub unsafe fn mh_get_cstr_t(set: *mut Set_cstr_t, key: cstr_t) -> uint32_t {
    unsafe { set_get(set, &key) }
}

/// # Safety
/// `set` must point at a live `Set_uint64_t`.
pub unsafe fn mh_get_uint64_t(set: *mut Set_uint64_t, key: uint64_t) -> uint32_t {
    unsafe { set_get(set, &key) }
}

// Insert a key into a set, or find it. See `table::put`.

/// # Safety
/// `set` must point at a live `Set_cstr_t` and `status` must be writable.
pub unsafe fn mh_put_cstr_t(
    set: *mut Set_cstr_t,
    key: cstr_t,
    status: *mut MHPutStatus,
) -> uint32_t {
    unsafe { set_put(set, key, status) }
}

// The slot a map holds for a key, inserting it if absent. The `init` passed
// is the C's zero value for the value type, which is what upstream's
// generated `map_put_ref` writes into a fresh slot.
// See `table::map_put_ref`.

/// # Safety
/// `map` must point at a live `Map_uint64_t_MTDamagePair`; the out-parameters
/// are null or writable.
pub unsafe fn map_put_ref_uint64_t_mt_damage_pair(
    map: *mut Map_uint64_t_MTDamagePair,
    key: uint64_t,
    key_alloc: *mut *mut uint64_t,
    new_item: *mut bool,
) -> *mut MTDamagePair {
    unsafe { map_put_ref(map, key, MTDAMAGE_PAIR_INIT, key_alloc, new_item) }
}
