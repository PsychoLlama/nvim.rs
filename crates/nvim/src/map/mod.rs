#![deny(unsafe_op_in_unsafe_fn)]

//! `Set_*` and `Map_*`: the khash-derived open-addressing tables the editor
//! keeps almost everything in.
//!
//! Upstream generates one copy of the algorithm per key type; the algorithm
//! lives once in [`table`] here, and this file is the monomorphic surface the
//! rest of the tree calls. `table::MapKey` is what a key type has to
//! implement.
//!
//! The `Set_*`/`Map_*` structs stay `repr(C)`: they are embedded *by value* in
//! structs all over the tree, so they cannot become std collections until
//! those graduate. Their internals are public because callers iterate
//! `set.keys[..set.h.n_keys]` and index `map.values` with what these entry
//! points answer.
//!
//! # Safety
//! Every entry point here takes a raw pointer to a live, initialized
//! `Set_*`/`Map_*` — an all-zero one counts as initialized and empty. Keys of
//! pointer type (`cstr_t`, `ptr_t`, `String`) must stay valid and unchanged
//! for as long as they are in the table.
//!
//! Derived, via upstream's `map.c`, from klib's `khash.h`, Copyright (c)
//! 2008, 2009, 2011 Attractive Chaos, under the MIT license; the notice is
//! reproduced in licenses/klib-LICENSE.txt.

mod policy;
mod table;

use crate::types::{
    MHPutStatus, MTDamage, MTDamagePair, MTNode, Map_String_int, Map_cstr_t_int, Map_cstr_t_ptr_t,
    Map_int_String, Map_int_ptr_t, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_ptr_t_ptr_t,
    Map_uint32_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_MTDamagePair, Map_uint64_t_int,
    Map_uint64_t_ptr_t, MapHash, Set_String, Set_cstr_t, Set_int, Set_int64_t, Set_ptr_t,
    Set_uint32_t, Set_uint64_t, String_0, cstr_t, int64_t, ptr_t, uint32_t, uint64_t,
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
    Set_String => String_0,
    Set_cstr_t => cstr_t,
    Set_int => ::core::ffi::c_int,
    Set_int64_t => int64_t,
    Set_ptr_t => ptr_t,
    Set_uint32_t => uint32_t,
    Set_uint64_t => uint64_t,
}

map_fields! {
    Map_String_int => String_0, ::core::ffi::c_int,
    Map_cstr_t_int => cstr_t, ::core::ffi::c_int,
    Map_cstr_t_ptr_t => cstr_t, ptr_t,
    Map_int64_t_int64_t => int64_t, int64_t,
    Map_int64_t_ptr_t => int64_t, ptr_t,
    Map_int_String => ::core::ffi::c_int, String_0,
    Map_int_ptr_t => ::core::ffi::c_int, ptr_t,
    Map_ptr_t_ptr_t => ptr_t, ptr_t,
    Map_uint32_t_ptr_t => uint32_t, ptr_t,
    Map_uint32_t_uint32_t => uint32_t, uint32_t,
    Map_uint64_t_MTDamagePair => uint64_t, MTDamagePair,
    Map_uint64_t_int => uint64_t, ::core::ffi::c_int,
    Map_uint64_t_ptr_t => uint64_t, ptr_t,
}

// The six generic entry points. Every monomorph below is one call to one of
// them: the deref of the caller's pointer and the call are the whole of the
// unchecked work, and it is written down once here rather than forty times.

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
/// As [`set_get`]; `key` must be a writable key slot.
unsafe fn set_delete<S: SetFields>(set: *mut S, key: *mut S::Key) -> uint32_t {
    let (h, keys) = unsafe { &mut *set }.fields();
    unsafe { table::delete(h, *keys, &mut *key) }
}

/// # Safety
/// `map` must point at a live `Map_*`; `key_alloc` is null or writable.
unsafe fn map_ref<M: MapFields>(
    map: *mut M,
    key: M::Key,
    key_alloc: *mut *mut M::Key,
) -> *mut M::Value {
    let (h, keys, values) = unsafe { &mut *map }.fields();
    unsafe { table::map_ref(h, *keys, *values, key, key_alloc) }
}

/// # Safety
/// As [`map_ref`]; `new_item` is null or writable.
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

/// # Safety
/// As [`map_ref`].
unsafe fn map_del<M: MapFields>(
    map: *mut M,
    key: M::Key,
    init: M::Value,
    key_alloc: *mut M::Key,
) -> M::Value {
    let (h, keys, values) = unsafe { &mut *map }.fields();
    unsafe { table::map_del(h, *keys, *values, key, init, key_alloc) }
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
/// `set` must point at a live `Set_int`.
pub unsafe fn mh_get_int(set: *mut Set_int, key: ::core::ffi::c_int) -> uint32_t {
    unsafe { set_get(set, &key) }
}

/// # Safety
/// `set` must point at a live `Set_cstr_t`.
pub unsafe fn mh_get_cstr_t(set: *mut Set_cstr_t, key: cstr_t) -> uint32_t {
    unsafe { set_get(set, &key) }
}

/// # Safety
/// `set` must point at a live `Set_ptr_t`.
pub unsafe fn mh_get_ptr_t(set: *mut Set_ptr_t, key: ptr_t) -> uint32_t {
    unsafe { set_get(set, &key) }
}

/// # Safety
/// `set` must point at a live `Set_uint64_t`.
pub unsafe fn mh_get_uint64_t(set: *mut Set_uint64_t, key: uint64_t) -> uint32_t {
    unsafe { set_get(set, &key) }
}

/// # Safety
/// `set` must point at a live `Set_int64_t`.
pub unsafe fn mh_get_int64_t(set: *mut Set_int64_t, key: int64_t) -> uint32_t {
    unsafe { set_get(set, &key) }
}

/// # Safety
/// `set` must point at a live `Set_uint32_t`.
pub unsafe fn mh_get_uint32_t(set: *mut Set_uint32_t, key: uint32_t) -> uint32_t {
    unsafe { set_get(set, &key) }
}

/// # Safety
/// `set` must point at a live `Set_String`.
pub unsafe fn mh_get_string(set: *mut Set_String, key: String_0) -> uint32_t {
    unsafe { set_get(set, &key) }
}

/// Whether `key` is in `set`.
///
/// # Safety
/// `set` must point at a live `Set_uint32_t`.
pub unsafe fn set_has_uint32_t(set: *mut Set_uint32_t, key: uint32_t) -> bool {
    unsafe { mh_get_uint32_t(set, key) != MH_TOMBSTONE }
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

/// # Safety
/// `set` must point at a live `Set_ptr_t` and `status` must be writable.
pub unsafe fn mh_put_ptr_t(set: *mut Set_ptr_t, key: ptr_t, status: *mut MHPutStatus) -> uint32_t {
    unsafe { set_put(set, key, status) }
}

/// # Safety
/// `set` must point at a live `Set_uint32_t` and `status` must be writable.
pub unsafe fn mh_put_uint32_t(
    set: *mut Set_uint32_t,
    key: uint32_t,
    status: *mut MHPutStatus,
) -> uint32_t {
    unsafe { set_put(set, key, status) }
}

/// # Safety
/// `set` must point at a live `Set_String` and `status` must be writable.
pub unsafe fn mh_put_string(
    set: *mut Set_String,
    key: String_0,
    status: *mut MHPutStatus,
) -> uint32_t {
    unsafe { set_put(set, key, status) }
}

// Remove a key from a set. See `table::delete`.

/// # Safety
/// `set` must point at a live `Set_ptr_t` and `key` must be writable.
pub unsafe fn mh_delete_ptr_t(set: *mut Set_ptr_t, key: *mut ptr_t) -> uint32_t {
    unsafe { set_delete(set, key) }
}

/// # Safety
/// `set` must point at a live `Set_uint32_t` and `key` must be writable.
pub unsafe fn mh_delete_uint32_t(set: *mut Set_uint32_t, key: *mut uint32_t) -> uint32_t {
    unsafe { set_delete(set, key) }
}

// The value a map holds for a key, or null. See `table::map_ref`.

/// # Safety
/// `map` must point at a live `Map_cstr_t_ptr_t`; `key_alloc` is null or writable.
pub unsafe fn map_ref_cstr_t_ptr_t(
    map: *mut Map_cstr_t_ptr_t,
    key: cstr_t,
    key_alloc: *mut *mut cstr_t,
) -> *mut ptr_t {
    unsafe { map_ref(map, key, key_alloc) }
}

/// # Safety
/// `map` must point at a live `Map_int64_t_int64_t`; `key_alloc` is null or writable.
pub unsafe fn map_ref_int64_t_int64_t(
    map: *mut Map_int64_t_int64_t,
    key: int64_t,
    key_alloc: *mut *mut int64_t,
) -> *mut int64_t {
    unsafe { map_ref(map, key, key_alloc) }
}

/// # Safety
/// `map` must point at a live `Map_uint32_t_uint32_t`; `key_alloc` is null or writable.
pub unsafe fn map_ref_uint32_t_uint32_t(
    map: *mut Map_uint32_t_uint32_t,
    key: uint32_t,
    key_alloc: *mut *mut uint32_t,
) -> *mut uint32_t {
    unsafe { map_ref(map, key, key_alloc) }
}

/// # Safety
/// `map` must point at a live `Map_String_int`; `key_alloc` is null or writable.
pub unsafe fn map_ref_string_int(
    map: *mut Map_String_int,
    key: String_0,
    key_alloc: *mut *mut String_0,
) -> *mut ::core::ffi::c_int {
    unsafe { map_ref(map, key, key_alloc) }
}

// The slot a map holds for a key, inserting it if absent. The `init` each
// one passes is the C's zero value for the value type, which is what
// upstream's generated `map_put_ref` writes into a fresh slot.
// See `table::map_put_ref`.

/// # Safety
/// `map` must point at a live `Map_cstr_t_int`; the out-parameters
/// are null or writable.
pub unsafe fn map_put_ref_cstr_t_int(
    map: *mut Map_cstr_t_int,
    key: cstr_t,
    key_alloc: *mut *mut cstr_t,
    new_item: *mut bool,
) -> *mut ::core::ffi::c_int {
    unsafe { map_put_ref(map, key, 0, key_alloc, new_item) }
}

/// # Safety
/// `map` must point at a live `Map_cstr_t_ptr_t`; the out-parameters
/// are null or writable.
pub unsafe fn map_put_ref_cstr_t_ptr_t(
    map: *mut Map_cstr_t_ptr_t,
    key: cstr_t,
    key_alloc: *mut *mut cstr_t,
    new_item: *mut bool,
) -> *mut ptr_t {
    unsafe { map_put_ref(map, key, ::core::ptr::null_mut(), key_alloc, new_item) }
}

/// # Safety
/// `map` must point at a live `Map_int64_t_int64_t`; the out-parameters
/// are null or writable.
pub unsafe fn map_put_ref_int64_t_int64_t(
    map: *mut Map_int64_t_int64_t,
    key: int64_t,
    key_alloc: *mut *mut int64_t,
    new_item: *mut bool,
) -> *mut int64_t {
    unsafe { map_put_ref(map, key, 0, key_alloc, new_item) }
}

/// # Safety
/// `map` must point at a live `Map_int64_t_ptr_t`; the out-parameters
/// are null or writable.
pub unsafe fn map_put_ref_int64_t_ptr_t(
    map: *mut Map_int64_t_ptr_t,
    key: int64_t,
    key_alloc: *mut *mut int64_t,
    new_item: *mut bool,
) -> *mut ptr_t {
    unsafe { map_put_ref(map, key, ::core::ptr::null_mut(), key_alloc, new_item) }
}

/// # Safety
/// `map` must point at a live `Map_int_ptr_t`; the out-parameters
/// are null or writable.
pub unsafe fn map_put_ref_int_ptr_t(
    map: *mut Map_int_ptr_t,
    key: ::core::ffi::c_int,
    key_alloc: *mut *mut ::core::ffi::c_int,
    new_item: *mut bool,
) -> *mut ptr_t {
    unsafe { map_put_ref(map, key, ::core::ptr::null_mut(), key_alloc, new_item) }
}

/// # Safety
/// `map` must point at a live `Map_int_String`; the out-parameters
/// are null or writable.
pub unsafe fn map_put_ref_int_string(
    map: *mut Map_int_String,
    key: ::core::ffi::c_int,
    key_alloc: *mut *mut ::core::ffi::c_int,
    new_item: *mut bool,
) -> *mut String_0 {
    unsafe { map_put_ref(map, key, String_0::NULL, key_alloc, new_item) }
}

/// # Safety
/// `map` must point at a live `Map_ptr_t_ptr_t`; the out-parameters
/// are null or writable.
pub unsafe fn map_put_ref_ptr_t_ptr_t(
    map: *mut Map_ptr_t_ptr_t,
    key: ptr_t,
    key_alloc: *mut *mut ptr_t,
    new_item: *mut bool,
) -> *mut ptr_t {
    unsafe { map_put_ref(map, key, ::core::ptr::null_mut(), key_alloc, new_item) }
}

/// # Safety
/// `map` must point at a live `Map_String_int`; the out-parameters
/// are null or writable.
pub unsafe fn map_put_ref_string_int(
    map: *mut Map_String_int,
    key: String_0,
    key_alloc: *mut *mut String_0,
    new_item: *mut bool,
) -> *mut ::core::ffi::c_int {
    unsafe { map_put_ref(map, key, 0, key_alloc, new_item) }
}

/// # Safety
/// `map` must point at a live `Map_uint32_t_ptr_t`; the out-parameters
/// are null or writable.
pub unsafe fn map_put_ref_uint32_t_ptr_t(
    map: *mut Map_uint32_t_ptr_t,
    key: uint32_t,
    key_alloc: *mut *mut uint32_t,
    new_item: *mut bool,
) -> *mut ptr_t {
    unsafe { map_put_ref(map, key, ::core::ptr::null_mut(), key_alloc, new_item) }
}

/// # Safety
/// `map` must point at a live `Map_uint32_t_uint32_t`; the out-parameters
/// are null or writable.
pub unsafe fn map_put_ref_uint32_t_uint32_t(
    map: *mut Map_uint32_t_uint32_t,
    key: uint32_t,
    key_alloc: *mut *mut uint32_t,
    new_item: *mut bool,
) -> *mut uint32_t {
    unsafe { map_put_ref(map, key, 0, key_alloc, new_item) }
}

/// # Safety
/// `map` must point at a live `Map_uint64_t_int`; the out-parameters
/// are null or writable.
pub unsafe fn map_put_ref_uint64_t_int(
    map: *mut Map_uint64_t_int,
    key: uint64_t,
    key_alloc: *mut *mut uint64_t,
    new_item: *mut bool,
) -> *mut ::core::ffi::c_int {
    unsafe { map_put_ref(map, key, 0, key_alloc, new_item) }
}

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

/// # Safety
/// `map` must point at a live `Map_uint64_t_ptr_t`; the out-parameters
/// are null or writable.
pub unsafe fn map_put_ref_uint64_t_ptr_t(
    map: *mut Map_uint64_t_ptr_t,
    key: uint64_t,
    key_alloc: *mut *mut uint64_t,
    new_item: *mut bool,
) -> *mut ptr_t {
    unsafe { map_put_ref(map, key, ::core::ptr::null_mut(), key_alloc, new_item) }
}

// Remove a key from a map and answer its value, or the value type's zero if
// it was absent. See `table::map_del`.

/// # Safety
/// `map` must point at a live `Map_cstr_t_ptr_t`; `key_alloc` is null or writable.
pub unsafe fn map_del_cstr_t_ptr_t(
    map: *mut Map_cstr_t_ptr_t,
    key: cstr_t,
    key_alloc: *mut cstr_t,
) -> ptr_t {
    unsafe { map_del(map, key, ::core::ptr::null_mut(), key_alloc) }
}

/// # Safety
/// `map` must point at a live `Map_int64_t_int64_t`; `key_alloc` is null or writable.
pub unsafe fn map_del_int64_t_int64_t(
    map: *mut Map_int64_t_int64_t,
    key: int64_t,
    key_alloc: *mut int64_t,
) -> int64_t {
    unsafe { map_del(map, key, 0, key_alloc) }
}

/// # Safety
/// `map` must point at a live `Map_int64_t_ptr_t`; `key_alloc` is null or writable.
pub unsafe fn map_del_int64_t_ptr_t(
    map: *mut Map_int64_t_ptr_t,
    key: int64_t,
    key_alloc: *mut int64_t,
) -> ptr_t {
    unsafe { map_del(map, key, ::core::ptr::null_mut(), key_alloc) }
}

/// # Safety
/// `map` must point at a live `Map_int_ptr_t`; `key_alloc` is null or writable.
pub unsafe fn map_del_int_ptr_t(
    map: *mut Map_int_ptr_t,
    key: ::core::ffi::c_int,
    key_alloc: *mut ::core::ffi::c_int,
) -> ptr_t {
    unsafe { map_del(map, key, ::core::ptr::null_mut(), key_alloc) }
}

/// # Safety
/// `map` must point at a live `Map_int_String`; `key_alloc` is null or writable.
pub unsafe fn map_del_int_string(
    map: *mut Map_int_String,
    key: ::core::ffi::c_int,
    key_alloc: *mut ::core::ffi::c_int,
) -> String_0 {
    unsafe { map_del(map, key, String_0::NULL, key_alloc) }
}

/// # Safety
/// `map` must point at a live `Map_String_int`; `key_alloc` is null or writable.
pub unsafe fn map_del_string_int(
    map: *mut Map_String_int,
    key: String_0,
    key_alloc: *mut String_0,
) -> ::core::ffi::c_int {
    unsafe { map_del(map, key, 0, key_alloc) }
}

/// # Safety
/// `map` must point at a live `Map_uint32_t_ptr_t`; `key_alloc` is null or writable.
pub unsafe fn map_del_uint32_t_ptr_t(
    map: *mut Map_uint32_t_ptr_t,
    key: uint32_t,
    key_alloc: *mut uint32_t,
) -> ptr_t {
    unsafe { map_del(map, key, ::core::ptr::null_mut(), key_alloc) }
}

/// # Safety
/// `map` must point at a live `Map_uint32_t_uint32_t`; `key_alloc` is null or writable.
pub unsafe fn map_del_uint32_t_uint32_t(
    map: *mut Map_uint32_t_uint32_t,
    key: uint32_t,
    key_alloc: *mut uint32_t,
) -> uint32_t {
    unsafe { map_del(map, key, 0, key_alloc) }
}

/// # Safety
/// `map` must point at a live `Map_uint64_t_ptr_t`; `key_alloc` is null or writable.
pub unsafe fn map_del_uint64_t_ptr_t(
    map: *mut Map_uint64_t_ptr_t,
    key: uint64_t,
    key_alloc: *mut uint64_t,
) -> ptr_t {
    unsafe { map_del(map, key, ::core::ptr::null_mut(), key_alloc) }
}
