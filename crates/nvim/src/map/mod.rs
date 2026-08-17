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

mod policy;
mod table;

use crate::types::{
    MHPutStatus, MTDamage, MTDamagePair, MTNode, Map_String_int, Map_cstr_t_int, Map_cstr_t_ptr_t,
    Map_int_String, Map_int_ptr_t, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_ptr_t_ptr_t,
    Map_uint32_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_MTDamagePair, Map_uint64_t_int,
    Map_uint64_t_ptr_t, MapHash, Set_String, Set_cstr_t, Set_int, Set_int64_t, Set_ptr_t,
    Set_uint32_t, Set_uint64_t, String_0, cstr_t, int64_t, ptr_t, uint32_t, uint64_t,
};
pub use table::{MH_TOMBSTONE, MapKey, kMHExisting, kMHNewKeyDidFit, kMHNewKeyRealloc};

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
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0,
};
/// Discard a bucket table and start again with room for `n_min_buckets`. The
/// caller rehashes: the glyph cache keeps its own keys array and calls this
/// directly.
pub unsafe fn mh_realloc(h: *mut MapHash, n_min_buckets: uint32_t) {
    table::realloc(&mut *h, n_min_buckets);
}

/// Forget every entry, keeping the bucket table's allocation.
pub unsafe fn mh_clear(h: *mut MapHash) {
    table::clear(&mut *h);
}

// The dense index of a key in a set, or `MH_TOMBSTONE`.

pub unsafe fn mh_get_int(set: *mut Set_int, key: ::core::ffi::c_int) -> uint32_t {
    table::get(&(*set).h, (*set).keys, &key)
}

pub unsafe fn mh_get_cstr_t(set: *mut Set_cstr_t, key: cstr_t) -> uint32_t {
    table::get(&(*set).h, (*set).keys, &key)
}

pub unsafe fn mh_get_ptr_t(set: *mut Set_ptr_t, key: ptr_t) -> uint32_t {
    table::get(&(*set).h, (*set).keys, &key)
}

pub unsafe fn mh_get_uint64_t(set: *mut Set_uint64_t, key: uint64_t) -> uint32_t {
    table::get(&(*set).h, (*set).keys, &key)
}

pub unsafe fn mh_get_int64_t(set: *mut Set_int64_t, key: int64_t) -> uint32_t {
    table::get(&(*set).h, (*set).keys, &key)
}

pub unsafe fn mh_get_uint32_t(set: *mut Set_uint32_t, key: uint32_t) -> uint32_t {
    table::get(&(*set).h, (*set).keys, &key)
}

pub unsafe fn mh_get_String(set: *mut Set_String, key: String_0) -> uint32_t {
    table::get(&(*set).h, (*set).keys, &key)
}

// Membership.

/// Whether `key` is in `set`.
pub unsafe fn set_has_uint32_t(set: *mut Set_uint32_t, key: uint32_t) -> bool {
    mh_get_uint32_t(set, key) != MH_TOMBSTONE
}

// Insert a key into a set, or find it. See `table::put`.

pub unsafe fn mh_put_cstr_t(
    set: *mut Set_cstr_t,
    key: cstr_t,
    status: *mut MHPutStatus,
) -> uint32_t {
    table::put(&mut (*set).h, &mut (*set).keys, key, &mut *status)
}

pub unsafe fn mh_put_ptr_t(set: *mut Set_ptr_t, key: ptr_t, status: *mut MHPutStatus) -> uint32_t {
    table::put(&mut (*set).h, &mut (*set).keys, key, &mut *status)
}

pub unsafe fn mh_put_uint32_t(
    set: *mut Set_uint32_t,
    key: uint32_t,
    status: *mut MHPutStatus,
) -> uint32_t {
    table::put(&mut (*set).h, &mut (*set).keys, key, &mut *status)
}

pub unsafe fn mh_put_String(
    set: *mut Set_String,
    key: String_0,
    status: *mut MHPutStatus,
) -> uint32_t {
    table::put(&mut (*set).h, &mut (*set).keys, key, &mut *status)
}

// Remove a key from a set. See `table::delete`.

pub unsafe fn mh_delete_ptr_t(set: *mut Set_ptr_t, key: *mut ptr_t) -> uint32_t {
    table::delete(&mut (*set).h, (*set).keys, &mut *key)
}

pub unsafe fn mh_delete_uint32_t(set: *mut Set_uint32_t, key: *mut uint32_t) -> uint32_t {
    table::delete(&mut (*set).h, (*set).keys, &mut *key)
}

// The value a map holds for a key, or null.

pub unsafe fn map_ref_cstr_t_ptr_t(
    map: *mut Map_cstr_t_ptr_t,
    key: cstr_t,
    key_alloc: *mut *mut cstr_t,
) -> *mut ptr_t {
    table::map_ref(
        &(*map).set.h,
        (*map).set.keys,
        (*map).values,
        key,
        key_alloc,
    )
}

pub unsafe fn map_ref_int64_t_int64_t(
    map: *mut Map_int64_t_int64_t,
    key: int64_t,
    key_alloc: *mut *mut int64_t,
) -> *mut int64_t {
    table::map_ref(
        &(*map).set.h,
        (*map).set.keys,
        (*map).values,
        key,
        key_alloc,
    )
}

pub unsafe fn map_ref_uint32_t_uint32_t(
    map: *mut Map_uint32_t_uint32_t,
    key: uint32_t,
    key_alloc: *mut *mut uint32_t,
) -> *mut uint32_t {
    table::map_ref(
        &(*map).set.h,
        (*map).set.keys,
        (*map).values,
        key,
        key_alloc,
    )
}

pub unsafe fn map_ref_String_int(
    map: *mut Map_String_int,
    key: String_0,
    key_alloc: *mut *mut String_0,
) -> *mut ::core::ffi::c_int {
    table::map_ref(
        &(*map).set.h,
        (*map).set.keys,
        (*map).values,
        key,
        key_alloc,
    )
}

// The slot a map holds for a key, inserting it if absent.

pub unsafe fn map_put_ref_cstr_t_int(
    map: *mut Map_cstr_t_int,
    key: cstr_t,
    key_alloc: *mut *mut cstr_t,
    new_item: *mut bool,
) -> *mut ::core::ffi::c_int {
    table::map_put_ref(
        &mut (*map).set.h,
        &mut (*map).set.keys,
        &mut (*map).values,
        key,
        0,
        key_alloc,
        new_item,
    )
}

pub unsafe fn map_put_ref_cstr_t_ptr_t(
    map: *mut Map_cstr_t_ptr_t,
    key: cstr_t,
    key_alloc: *mut *mut cstr_t,
    new_item: *mut bool,
) -> *mut ptr_t {
    table::map_put_ref(
        &mut (*map).set.h,
        &mut (*map).set.keys,
        &mut (*map).values,
        key,
        ::core::ptr::null_mut(),
        key_alloc,
        new_item,
    )
}

pub unsafe fn map_put_ref_int64_t_int64_t(
    map: *mut Map_int64_t_int64_t,
    key: int64_t,
    key_alloc: *mut *mut int64_t,
    new_item: *mut bool,
) -> *mut int64_t {
    table::map_put_ref(
        &mut (*map).set.h,
        &mut (*map).set.keys,
        &mut (*map).values,
        key,
        0,
        key_alloc,
        new_item,
    )
}

pub unsafe fn map_put_ref_int64_t_ptr_t(
    map: *mut Map_int64_t_ptr_t,
    key: int64_t,
    key_alloc: *mut *mut int64_t,
    new_item: *mut bool,
) -> *mut ptr_t {
    table::map_put_ref(
        &mut (*map).set.h,
        &mut (*map).set.keys,
        &mut (*map).values,
        key,
        ::core::ptr::null_mut(),
        key_alloc,
        new_item,
    )
}

pub unsafe fn map_put_ref_int_ptr_t(
    map: *mut Map_int_ptr_t,
    key: ::core::ffi::c_int,
    key_alloc: *mut *mut ::core::ffi::c_int,
    new_item: *mut bool,
) -> *mut ptr_t {
    table::map_put_ref(
        &mut (*map).set.h,
        &mut (*map).set.keys,
        &mut (*map).values,
        key,
        ::core::ptr::null_mut(),
        key_alloc,
        new_item,
    )
}

pub unsafe fn map_put_ref_int_String(
    map: *mut Map_int_String,
    key: ::core::ffi::c_int,
    key_alloc: *mut *mut ::core::ffi::c_int,
    new_item: *mut bool,
) -> *mut String_0 {
    table::map_put_ref(
        &mut (*map).set.h,
        &mut (*map).set.keys,
        &mut (*map).values,
        key,
        STRING_INIT,
        key_alloc,
        new_item,
    )
}

pub unsafe fn map_put_ref_ptr_t_ptr_t(
    map: *mut Map_ptr_t_ptr_t,
    key: ptr_t,
    key_alloc: *mut *mut ptr_t,
    new_item: *mut bool,
) -> *mut ptr_t {
    table::map_put_ref(
        &mut (*map).set.h,
        &mut (*map).set.keys,
        &mut (*map).values,
        key,
        ::core::ptr::null_mut(),
        key_alloc,
        new_item,
    )
}

pub unsafe fn map_put_ref_String_int(
    map: *mut Map_String_int,
    key: String_0,
    key_alloc: *mut *mut String_0,
    new_item: *mut bool,
) -> *mut ::core::ffi::c_int {
    table::map_put_ref(
        &mut (*map).set.h,
        &mut (*map).set.keys,
        &mut (*map).values,
        key,
        0,
        key_alloc,
        new_item,
    )
}

pub unsafe fn map_put_ref_uint32_t_ptr_t(
    map: *mut Map_uint32_t_ptr_t,
    key: uint32_t,
    key_alloc: *mut *mut uint32_t,
    new_item: *mut bool,
) -> *mut ptr_t {
    table::map_put_ref(
        &mut (*map).set.h,
        &mut (*map).set.keys,
        &mut (*map).values,
        key,
        ::core::ptr::null_mut(),
        key_alloc,
        new_item,
    )
}

pub unsafe fn map_put_ref_uint32_t_uint32_t(
    map: *mut Map_uint32_t_uint32_t,
    key: uint32_t,
    key_alloc: *mut *mut uint32_t,
    new_item: *mut bool,
) -> *mut uint32_t {
    table::map_put_ref(
        &mut (*map).set.h,
        &mut (*map).set.keys,
        &mut (*map).values,
        key,
        0,
        key_alloc,
        new_item,
    )
}

pub unsafe fn map_put_ref_uint64_t_int(
    map: *mut Map_uint64_t_int,
    key: uint64_t,
    key_alloc: *mut *mut uint64_t,
    new_item: *mut bool,
) -> *mut ::core::ffi::c_int {
    table::map_put_ref(
        &mut (*map).set.h,
        &mut (*map).set.keys,
        &mut (*map).values,
        key,
        0,
        key_alloc,
        new_item,
    )
}

pub unsafe fn map_put_ref_uint64_t_MTDamagePair(
    map: *mut Map_uint64_t_MTDamagePair,
    key: uint64_t,
    key_alloc: *mut *mut uint64_t,
    new_item: *mut bool,
) -> *mut MTDamagePair {
    table::map_put_ref(
        &mut (*map).set.h,
        &mut (*map).set.keys,
        &mut (*map).values,
        key,
        MTDAMAGE_PAIR_INIT,
        key_alloc,
        new_item,
    )
}

pub unsafe fn map_put_ref_uint64_t_ptr_t(
    map: *mut Map_uint64_t_ptr_t,
    key: uint64_t,
    key_alloc: *mut *mut uint64_t,
    new_item: *mut bool,
) -> *mut ptr_t {
    table::map_put_ref(
        &mut (*map).set.h,
        &mut (*map).set.keys,
        &mut (*map).values,
        key,
        ::core::ptr::null_mut(),
        key_alloc,
        new_item,
    )
}

// Remove a key from a map and answer its value.

pub unsafe fn map_del_cstr_t_ptr_t(
    map: *mut Map_cstr_t_ptr_t,
    key: cstr_t,
    key_alloc: *mut cstr_t,
) -> ptr_t {
    table::map_del(
        &mut (*map).set.h,
        (*map).set.keys,
        (*map).values,
        key,
        ::core::ptr::null_mut(),
        key_alloc,
    )
}

pub unsafe fn map_del_int64_t_int64_t(
    map: *mut Map_int64_t_int64_t,
    key: int64_t,
    key_alloc: *mut int64_t,
) -> int64_t {
    table::map_del(
        &mut (*map).set.h,
        (*map).set.keys,
        (*map).values,
        key,
        0,
        key_alloc,
    )
}

pub unsafe fn map_del_int64_t_ptr_t(
    map: *mut Map_int64_t_ptr_t,
    key: int64_t,
    key_alloc: *mut int64_t,
) -> ptr_t {
    table::map_del(
        &mut (*map).set.h,
        (*map).set.keys,
        (*map).values,
        key,
        ::core::ptr::null_mut(),
        key_alloc,
    )
}

pub unsafe fn map_del_int_ptr_t(
    map: *mut Map_int_ptr_t,
    key: ::core::ffi::c_int,
    key_alloc: *mut ::core::ffi::c_int,
) -> ptr_t {
    table::map_del(
        &mut (*map).set.h,
        (*map).set.keys,
        (*map).values,
        key,
        ::core::ptr::null_mut(),
        key_alloc,
    )
}

pub unsafe fn map_del_int_String(
    map: *mut Map_int_String,
    key: ::core::ffi::c_int,
    key_alloc: *mut ::core::ffi::c_int,
) -> String_0 {
    table::map_del(
        &mut (*map).set.h,
        (*map).set.keys,
        (*map).values,
        key,
        STRING_INIT,
        key_alloc,
    )
}

pub unsafe fn map_del_String_int(
    map: *mut Map_String_int,
    key: String_0,
    key_alloc: *mut String_0,
) -> ::core::ffi::c_int {
    table::map_del(
        &mut (*map).set.h,
        (*map).set.keys,
        (*map).values,
        key,
        0,
        key_alloc,
    )
}

pub unsafe fn map_del_uint32_t_ptr_t(
    map: *mut Map_uint32_t_ptr_t,
    key: uint32_t,
    key_alloc: *mut uint32_t,
) -> ptr_t {
    table::map_del(
        &mut (*map).set.h,
        (*map).set.keys,
        (*map).values,
        key,
        ::core::ptr::null_mut(),
        key_alloc,
    )
}

pub unsafe fn map_del_uint32_t_uint32_t(
    map: *mut Map_uint32_t_uint32_t,
    key: uint32_t,
    key_alloc: *mut uint32_t,
) -> uint32_t {
    table::map_del(
        &mut (*map).set.h,
        (*map).set.keys,
        (*map).values,
        key,
        0,
        key_alloc,
    )
}

pub unsafe fn map_del_uint64_t_ptr_t(
    map: *mut Map_uint64_t_ptr_t,
    key: uint64_t,
    key_alloc: *mut uint64_t,
) -> ptr_t {
    table::map_del(
        &mut (*map).set.h,
        (*map).set.keys,
        (*map).values,
        key,
        ::core::ptr::null_mut(),
        key_alloc,
    )
}
