//! Hash map/set implementation compatible with nvim's map.c
//!
//! Derived from khash.h (klib, MIT license). This module provides C-compatible
//! implementations of all map/set operations, type-specialized via macros.

// Macro-generated FFI functions produce many similar patterns that clippy
// flags individually.  Suppress at module level rather than per-invocation.
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::inline_always)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ref_as_ptr)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::as_ptr_cast_mut)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use nvim_memory::{xcalloc, xfree, xrealloc};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Sentinel value: bucket has never been used (hash == 0) or was deleted
/// (hash == MH_TOMBSTONE).
const MH_TOMBSTONE: u32 = u32::MAX;

/// Load factor threshold – matches the C `UPPER_FILL` constant.
const UPPER_FILL: f64 = 0.77;

// ---------------------------------------------------------------------------
// FFI struct definitions – must match C layouts exactly
// ---------------------------------------------------------------------------

/// Core hash table bookkeeping – matches C `MapHash` (map_defs.h).
#[repr(C)]
pub struct MapHash {
    pub n_buckets: u32,
    pub size: u32,
    pub n_occupied: u32,
    pub upper_bound: u32,
    pub n_keys: u32,
    pub keys_capacity: u32,
    pub hash: *mut u32,
}

/// Status returned by `mh_put_*` – matches C `MHPutStatus`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MHPutStatus {
    Existing = 0,
    NewKeyDidFit = 1,
    NewKeyRealloc = 2,
}

/// API `String` type – matches C `String` in api/private/defs.h.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NvimString {
    pub data: *mut c_char,
    pub size: usize,
}

/// Highlight attributes – matches C `HlAttrs` (highlight_defs.h).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HlAttrs {
    pub rgb_ae_attr: i16,
    pub cterm_ae_attr: i16,
    pub rgb_fg_color: i32, // RgbValue
    pub rgb_bg_color: i32,
    pub rgb_sp_color: i32,
    pub cterm_fg_color: i16,
    pub cterm_bg_color: i16,
    pub hl_blend: i32,
    pub url: i32,
}

/// Highlight kind – matches C `HlKind` enum (highlight_defs.h).
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HlKind {
    Unknown = 0,
    UI = 1,
    Syntax = 2,
    Terminal = 3,
    Combine = 4,
    Blend = 5,
    BlendThrough = 6,
    Invalid = 7,
}

/// Highlight entry – matches C `HlEntry` (highlight_defs.h).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HlEntry {
    pub attr: HlAttrs,
    pub kind: HlKind,
    pub id1: c_int,
    pub id2: c_int,
    pub winid: c_int,
}

/// Color key – matches C `ColorKey` (highlight_defs.h).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ColorKey {
    pub ns_id: c_int,
    pub syn_id: c_int,
}

/// Color item – matches C `ColorItem` (highlight_defs.h).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ColorItem {
    pub attr_id: c_int,
    pub link_id: c_int,
    pub version: c_int,
    pub is_default: bool,
    pub link_global: bool,
}

// Size assertions – if these fail, the Rust struct layout doesn't match C.
const _: () = assert!(size_of::<MapHash>() == 4 * 6 + 8); // 6 u32 + 1 pointer
const _: () = assert!(size_of::<NvimString>() == size_of::<*mut c_char>() + size_of::<usize>());
const _: () = assert!(size_of::<ColorKey>() == 8);

// ---------------------------------------------------------------------------
// Hash bucket helpers (inline, matching C macros)
// ---------------------------------------------------------------------------

#[inline(always)]
fn mh_is_empty(hash: *const u32, i: u32) -> bool {
    unsafe { *hash.add(i as usize) == 0 }
}

#[inline(always)]
fn mh_is_del(hash: *const u32, i: u32) -> bool {
    unsafe { *hash.add(i as usize) == MH_TOMBSTONE }
}

#[inline(always)]
fn mh_is_either(hash: *const u32, i: u32) -> bool {
    unsafe { (*hash.add(i as usize)).wrapping_add(1) <= 1 }
}

// ---------------------------------------------------------------------------
// Roundup32 – next power of two ≥ input, minimum 16
// ---------------------------------------------------------------------------

/// Equivalent to the C `roundup32` macro (with pre-decrement).
fn roundup32(mut x: u32) -> u32 {
    x = x.wrapping_sub(1);
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x.wrapping_add(1)
}

// ---------------------------------------------------------------------------
// Hash functions
// ---------------------------------------------------------------------------

#[inline]
fn hash_uint64(key: u64) -> u32 {
    (key.wrapping_shr(33) ^ key ^ key.wrapping_shl(11)) as u32
}

#[inline]
fn hash_uint32(key: u32) -> u32 {
    key
}

#[inline]
fn hash_int(key: c_int) -> u32 {
    key as u32
}

#[inline]
fn hash_int64(key: i64) -> u32 {
    hash_uint64(key as u64)
}

#[inline]
fn hash_ptr(key: *mut c_void) -> u32 {
    #[cfg(target_pointer_width = "64")]
    {
        hash_uint64(key as u64)
    }
    #[cfg(target_pointer_width = "32")]
    {
        hash_uint32(key as u32)
    }
}

#[inline]
unsafe fn hash_cstr(s: *const c_char) -> u32 {
    let mut h: u32 = 0;
    let mut p = s as *const u8;
    while *p != 0 {
        h = h
            .wrapping_shl(5)
            .wrapping_sub(h)
            .wrapping_add(u32::from(*p));
        p = p.add(1);
    }
    h
}

#[inline]
unsafe fn hash_nvim_string(s: NvimString) -> u32 {
    let mut h: u32 = 0;
    let data = s.data as *const u8;
    for i in 0..s.size {
        h = h
            .wrapping_shl(5)
            .wrapping_sub(h)
            .wrapping_add(u32::from(*data.add(i)));
    }
    h
}

/// Byte-wise hash for fixed-size types (HlEntry, ColorKey).
#[inline]
unsafe fn hash_bytes<T>(val: *const T) -> u32 {
    let data = val as *const u8;
    let mut h: u32 = 0;
    for i in 0..size_of::<T>() {
        h = h
            .wrapping_shl(5)
            .wrapping_sub(h)
            .wrapping_add(u32::from(*data.add(i)));
    }
    h
}

// ---------------------------------------------------------------------------
// Equality functions
// ---------------------------------------------------------------------------

#[inline]
fn equal_simple<T: PartialEq>(a: T, b: T) -> bool {
    a == b
}

/// NULL-safe string equality — matches `strequal()` semantics.
#[inline]
unsafe fn equal_cstr(a: *const c_char, b: *const c_char) -> bool {
    if a == b {
        return true;
    }
    if a.is_null() || b.is_null() {
        return false;
    }
    libc::strcmp(a, b) == 0
}

#[inline]
unsafe fn equal_nvim_string(a: NvimString, b: NvimString) -> bool {
    if a.size != b.size {
        return false;
    }
    a.size == 0 || libc::memcmp(a.data.cast(), b.data.cast(), a.size) == 0
}

#[inline]
unsafe fn equal_bytes<T>(a: *const T, b: *const T) -> bool {
    libc::memcmp(a.cast(), b.cast(), size_of::<T>()) == 0
}

// ---------------------------------------------------------------------------
// Core non-generic functions: mh_realloc, mh_clear
// ---------------------------------------------------------------------------

/// Reallocate the hash bucket array. Matches C `mh_realloc`.
///
/// # Safety
///
/// `h` must point to a valid `MapHash`. `h->hash` must be NULL or a valid
/// allocation from `xcalloc`.
#[unsafe(export_name = "mh_realloc")]
pub unsafe extern "C" fn rs_mh_realloc(h: *mut MapHash, n_min_buckets: u32) {
    let h = &mut *h;
    xfree(h.hash.cast());
    let n_buckets = if n_min_buckets < 16 {
        16u32
    } else {
        roundup32(n_min_buckets)
    };
    h.hash = xcalloc(n_buckets as usize, size_of::<u32>()).cast();
    h.n_occupied = 0;
    h.size = 0;
    h.n_buckets = n_buckets;
    h.upper_bound = (f64::from(n_buckets) * UPPER_FILL + 0.5) as u32;
}

/// Clear all entries. Matches C `mh_clear`.
///
/// # Safety
///
/// `h` must point to a valid `MapHash`.
#[unsafe(export_name = "mh_clear")]
pub unsafe extern "C" fn rs_mh_clear(h: *mut MapHash) {
    let h = &mut *h;
    if !h.hash.is_null() {
        ptr::write_bytes(h.hash, 0, h.n_buckets as usize);
        h.size = 0;
        h.n_occupied = 0;
        h.n_keys = 0;
    }
}

// ---------------------------------------------------------------------------
// Key-type macro: generates Set struct + 5 functions per key type
// ---------------------------------------------------------------------------

/// Helper to compute MAX(a, b) like the C macro.
#[inline]
fn max_u32(a: u32, b: u32) -> u32 {
    if a > b {
        a
    } else {
        b
    }
}

macro_rules! map_key_impl {
    // $name:      type suffix used in C identifiers (e.g. `int`, `ptr_t`)
    // $key_ty:    Rust type for the key
    // $hash_fn:   expression that hashes a key value
    // $equal_fn:  expression that compares two key values for equality
    ($name:ident, $key_ty:ty, $hash_fn:expr, $equal_fn:expr) => {
        paste::paste! {
            /// Set type for this key – matches C `Set_<name>`.
            #[repr(C)]
            pub struct [<Set_ $name>] {
                pub h: MapHash,
                pub keys: *mut $key_ty,
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<rs_mh_find_bucket_ $name>](
                set: *mut [<Set_ $name>],
                key: $key_ty,
                put: bool,
            ) -> u32 {
                let set = &mut *set;
                let h = &set.h;
                let mask = h.n_buckets - 1;
                let k = ($hash_fn)(key);
                let mut i = k & mask;
                let last = i;
                let mut site: u32 = if put { last } else { MH_TOMBSTONE };
                let mut step: u32 = 0;
                while !mh_is_empty(h.hash, i) {
                    if mh_is_del(h.hash, i) {
                        if site == last {
                            site = i;
                        }
                    } else {
                        let idx = *h.hash.add(i as usize) - 1;
                        if ($equal_fn)(*set.keys.add(idx as usize), key) {
                            return i;
                        }
                    }
                    step += 1;
                    i = (i.wrapping_add(step)) & mask;
                    if i == last {
                        std::process::abort();
                    }
                }
                if site == last {
                    site = i;
                }
                site
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<rs_mh_get_ $name>](
                set: *mut [<Set_ $name>],
                key: $key_ty,
            ) -> u32 {
                let set = &*set;
                if set.h.n_buckets == 0 {
                    return MH_TOMBSTONE;
                }
                let idx = [<rs_mh_find_bucket_ $name>](set as *const _ as *mut _, key, false);
                if idx != MH_TOMBSTONE {
                    *(*set).h.hash.add(idx as usize) - 1
                } else {
                    MH_TOMBSTONE
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<rs_mh_rehash_ $name>](set: *mut [<Set_ $name>]) {
                let set = &mut *set;
                for k in 0..set.h.n_keys {
                    let key = *set.keys.add(k as usize);
                    let idx = [<rs_mh_find_bucket_ $name>](set, key, true);
                    if !mh_is_empty(set.h.hash, idx) {
                        std::process::abort();
                    }
                    *set.h.hash.add(idx as usize) = k + 1;
                }
                set.h.n_occupied = set.h.n_keys;
                set.h.size = set.h.n_keys;
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<rs_mh_put_ $name>](
                set: *mut [<Set_ $name>],
                key: $key_ty,
                new: *mut MHPutStatus,
            ) -> u32 {
                // Use raw pointer access to avoid borrow checker issues
                // with the realloc path that needs both &mut set.h and set.h fields.
                let h = &raw mut (*set).h;

                if (*h).n_occupied >= (*h).upper_bound {
                    if (*h).size >= (f64::from((*h).upper_bound) * 0.9) as u32 {
                        rs_mh_realloc(h, (*h).n_buckets + 1);
                    } else {
                        ptr::write_bytes((*h).hash, 0, (*h).n_buckets as usize);
                        (*h).size = 0;
                        (*h).n_occupied = 0;
                    }
                    [<rs_mh_rehash_ $name>](set);
                }

                let idx = [<rs_mh_find_bucket_ $name>](set, key, true);

                if mh_is_either((*h).hash, idx) {
                    (*h).size += 1;
                    if mh_is_empty((*h).hash, idx) {
                        (*h).n_occupied += 1;
                    }

                    let pos = (*h).n_keys;
                    (*h).n_keys += 1;
                    if pos >= (*h).keys_capacity {
                        (*h).keys_capacity = max_u32((*h).keys_capacity * 2, 8);
                        (*set).keys = xrealloc(
                            (*set).keys.cast(),
                            ((*h).keys_capacity as usize) * size_of::<$key_ty>(),
                        )
                        .cast();
                        *new = MHPutStatus::NewKeyRealloc;
                    } else {
                        *new = MHPutStatus::NewKeyDidFit;
                    }
                    *(*set).keys.add(pos as usize) = key;
                    *(*h).hash.add(idx as usize) = pos + 1;
                    pos
                } else {
                    *new = MHPutStatus::Existing;
                    let pos = *(*h).hash.add(idx as usize) - 1;
                    if !($equal_fn)(*(*set).keys.add(pos as usize), key) {
                        std::process::abort();
                    }
                    pos
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<rs_mh_delete_ $name>](
                set: *mut [<Set_ $name>],
                key: *mut $key_ty,
            ) -> u32 {
                let set = &mut *set;
                if set.h.size == 0 {
                    return MH_TOMBSTONE;
                }
                let idx = [<rs_mh_find_bucket_ $name>](set, *key, false);
                if idx != MH_TOMBSTONE {
                    let k = *set.h.hash.add(idx as usize) - 1;
                    *set.h.hash.add(idx as usize) = MH_TOMBSTONE;

                    let last = set.h.n_keys - 1;
                    set.h.n_keys = last;
                    *key = *set.keys.add(k as usize);
                    set.h.size -= 1;
                    if last != k {
                        let idx2 = [<rs_mh_find_bucket_ $name>](
                            set,
                            *set.keys.add(last as usize),
                            false,
                        );
                        if *set.h.hash.add(idx2 as usize) != last + 1 {
                            std::process::abort();
                        }
                        *set.h.hash.add(idx2 as usize) = k + 1;
                        *set.keys.add(k as usize) = *set.keys.add(last as usize);
                    }
                    k
                } else {
                    MH_TOMBSTONE
                }
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Value-type macro: generates Map struct + 3 functions per map type pair
// ---------------------------------------------------------------------------

macro_rules! map_value_impl {
    // $kname:     key type suffix
    // $vname:     value type suffix
    // $key_ty:    Rust key type
    // $val_ty:    Rust value type
    // $val_init:  default/initial value for the value type
    ($kname:ident, $vname:ident, $key_ty:ty, $val_ty:ty, $val_init:expr) => {
        paste::paste! {
            #[repr(C)]
            pub struct [<Map_ $kname $vname>] {
                pub set: [<Set_ $kname>],
                pub values: *mut $val_ty,
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<rs_map_ref_ $kname $vname>](
                map: *mut [<Map_ $kname $vname>],
                key: $key_ty,
                key_alloc: *mut *mut $key_ty,
            ) -> *mut $val_ty {
                let map = &mut *map;
                let k = [<rs_mh_get_ $kname>](&mut map.set, key);
                if k == MH_TOMBSTONE {
                    return ptr::null_mut();
                }
                if !key_alloc.is_null() {
                    *key_alloc = map.set.keys.add(k as usize);
                }
                map.values.add(k as usize)
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<rs_map_put_ref_ $kname $vname>](
                map: *mut [<Map_ $kname $vname>],
                key: $key_ty,
                key_alloc: *mut *mut $key_ty,
                new_item: *mut bool,
            ) -> *mut $val_ty {
                let map = &mut *map;
                let mut status = MHPutStatus::Existing;
                let k = [<rs_mh_put_ $kname>](&mut map.set, key, &mut status);
                if status != MHPutStatus::Existing {
                    if status == MHPutStatus::NewKeyRealloc {
                        map.values = xrealloc(
                            map.values.cast(),
                            (map.set.h.keys_capacity as usize) * size_of::<$val_ty>(),
                        )
                        .cast();
                    }
                    *map.values.add(k as usize) = $val_init;
                }
                if !new_item.is_null() {
                    *new_item = status != MHPutStatus::Existing;
                }
                if !key_alloc.is_null() {
                    *key_alloc = map.set.keys.add(k as usize);
                }
                map.values.add(k as usize)
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<rs_map_del_ $kname $vname>](
                map: *mut [<Map_ $kname $vname>],
                key: $key_ty,
                key_alloc: *mut $key_ty,
            ) -> $val_ty {
                let map = &mut *map;
                let rv: $val_ty = $val_init;
                let mut key_copy = key;
                let k = [<rs_mh_delete_ $kname>](&mut map.set, &mut key_copy);
                if k == MH_TOMBSTONE {
                    return rv;
                }
                if !key_alloc.is_null() {
                    *key_alloc = key_copy;
                }
                let rv = *map.values.add(k as usize);
                if k != map.set.h.n_keys {
                    *map.values.add(k as usize) = *map.values.add(map.set.h.n_keys as usize);
                }
                rv
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Instantiate key types (9 types × 5 functions = 45 functions)
// ---------------------------------------------------------------------------

map_key_impl!(int, c_int, hash_int, equal_simple);
map_key_impl!(ptr_t, *mut c_void, hash_ptr, equal_simple);
map_key_impl!(cstr_t, *const c_char, |k| hash_cstr(k), |a, b| equal_cstr(
    a, b
));
map_key_impl!(String, NvimString, |s| hash_nvim_string(s), |a, b| {
    equal_nvim_string(a, b)
});
map_key_impl!(uint32_t, u32, hash_uint32, equal_simple);
map_key_impl!(uint64_t, u64, hash_uint64, equal_simple);
map_key_impl!(int64_t, i64, |k| hash_int64(k), equal_simple);
map_key_impl!(
    HlEntry,
    HlEntry,
    |k: HlEntry| hash_bytes(&k),
    |a: HlEntry, b: HlEntry| equal_bytes(&a, &b)
);
map_key_impl!(
    ColorKey,
    ColorKey,
    |k: ColorKey| hash_bytes(&k),
    |a: ColorKey, b: ColorKey| equal_bytes(&a, &b)
);

// ---------------------------------------------------------------------------
// Instantiate value types (15 map pairs × 3 functions = 45 functions)
// ---------------------------------------------------------------------------

// int -> int, ptr_t, String
map_value_impl!(int, int, c_int, c_int, 0);
map_value_impl!(int, ptr_t, c_int, *mut c_void, ptr::null_mut());
map_value_impl!(
    int,
    String,
    c_int,
    NvimString,
    NvimString {
        data: ptr::null_mut(),
        size: 0
    }
);

// ptr_t -> ptr_t
map_value_impl!(ptr_t, ptr_t, *mut c_void, *mut c_void, ptr::null_mut());

// cstr_t -> ptr_t, int
map_value_impl!(cstr_t, ptr_t, *const c_char, *mut c_void, ptr::null_mut());
map_value_impl!(cstr_t, int, *const c_char, c_int, 0);

// String -> int
map_value_impl!(String, int, NvimString, c_int, 0);

// uint32_t -> ptr_t, uint32_t
map_value_impl!(uint32_t, ptr_t, u32, *mut c_void, ptr::null_mut());
map_value_impl!(uint32_t, uint32_t, u32, u32, 0);

// uint64_t -> ptr_t, ssize_t, uint64_t
map_value_impl!(uint64_t, ptr_t, u64, *mut c_void, ptr::null_mut());
map_value_impl!(uint64_t, ssize_t, u64, isize, -1);
map_value_impl!(uint64_t, uint64_t, u64, u64, 0);

// int64_t -> ptr_t, int64_t
map_value_impl!(int64_t, ptr_t, i64, *mut c_void, ptr::null_mut());
map_value_impl!(int64_t, int64_t, i64, i64, 0);

// ColorKey -> ColorItem
map_value_impl!(
    ColorKey,
    ColorItem,
    ColorKey,
    ColorItem,
    ColorItem {
        attr_id: -1,
        link_id: -1,
        version: -1,
        is_default: false,
        link_global: false,
    }
);

// ---------------------------------------------------------------------------
// pmap_del2
// ---------------------------------------------------------------------------

/// Delete a key:value pair from a string:pointer map and free both key and
/// value storage. Matches C `pmap_del2`.
#[unsafe(export_name = "pmap_del2")]
pub unsafe extern "C" fn rs_pmap_del2(map: *mut Map_cstr_tptr_t, key: *const c_char) {
    let mut key_alloc: *const c_char = ptr::null();
    let val = rs_map_del_cstr_tptr_t(map, key, &mut key_alloc as *mut *const c_char);
    xfree(key_alloc as *mut c_void);
    xfree(val);
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_uint64() {
        // Verify specific values to catch overflow behavior mismatches
        assert_eq!(hash_uint64(0), 0);
        assert_eq!(hash_uint64(1), 1 ^ (1u64.wrapping_shl(11) as u32));
        // Large value
        let big: u64 = 0xDEAD_BEEF_CAFE_BABE;
        let expected = (big.wrapping_shr(33) ^ big ^ big.wrapping_shl(11)) as u32;
        assert_eq!(hash_uint64(big), expected);
    }

    #[test]
    fn test_hash_uint32() {
        assert_eq!(hash_uint32(0), 0);
        assert_eq!(hash_uint32(42), 42);
        assert_eq!(hash_uint32(u32::MAX), u32::MAX);
    }

    #[test]
    fn test_hash_int() {
        assert_eq!(hash_int(0), 0);
        assert_eq!(hash_int(-1), u32::MAX); // -1 as u32
        assert_eq!(hash_int(42), 42);
    }

    #[test]
    fn test_hash_int64() {
        assert_eq!(hash_int64(0), hash_uint64(0));
        assert_eq!(hash_int64(-1), hash_uint64(u64::MAX));
        assert_eq!(hash_int64(12345), hash_uint64(12345));
    }

    #[test]
    fn test_hash_cstr() {
        unsafe {
            let s = b"hello\0".as_ptr() as *const c_char;
            let h = hash_cstr(s);
            // Manually compute: h=0, then for each byte of "hello"
            let mut expected: u32 = 0;
            for &b in b"hello" {
                expected = expected
                    .wrapping_shl(5)
                    .wrapping_sub(expected)
                    .wrapping_add(u32::from(b));
            }
            assert_eq!(h, expected);

            // Empty string
            let empty = b"\0".as_ptr() as *const c_char;
            assert_eq!(hash_cstr(empty), 0);
        }
    }

    #[test]
    fn test_hash_nvim_string() {
        unsafe {
            let data = b"hello";
            let s = NvimString {
                data: data.as_ptr() as *mut c_char,
                size: 5,
            };
            let h = hash_nvim_string(s);
            // Should match hash_cstr for the same content
            let cstr = b"hello\0".as_ptr() as *const c_char;
            assert_eq!(h, hash_cstr(cstr));

            // Empty
            let empty = NvimString {
                data: ptr::null_mut(),
                size: 0,
            };
            assert_eq!(hash_nvim_string(empty), 0);
        }
    }

    #[test]
    fn test_equal_cstr_null_safety() {
        unsafe {
            // Both null → true
            assert!(equal_cstr(ptr::null(), ptr::null()));
            // One null → false
            let s = b"hello\0".as_ptr() as *const c_char;
            assert!(!equal_cstr(s, ptr::null()));
            assert!(!equal_cstr(ptr::null(), s));
            // Same pointer → true
            assert!(equal_cstr(s, s));
            // Equal content
            let s2 = b"hello\0".as_ptr() as *const c_char;
            assert!(equal_cstr(s, s2));
            // Different content
            let s3 = b"world\0".as_ptr() as *const c_char;
            assert!(!equal_cstr(s, s3));
        }
    }

    #[test]
    fn test_equal_nvim_string() {
        unsafe {
            let a = NvimString {
                data: b"hello".as_ptr() as *mut c_char,
                size: 5,
            };
            let b = NvimString {
                data: b"hello".as_ptr() as *mut c_char,
                size: 5,
            };
            assert!(equal_nvim_string(a, b));

            // Different sizes
            let c = NvimString {
                data: b"hell".as_ptr() as *mut c_char,
                size: 4,
            };
            assert!(!equal_nvim_string(a, c));

            // Both empty
            let d = NvimString {
                data: ptr::null_mut(),
                size: 0,
            };
            let e = NvimString {
                data: ptr::null_mut(),
                size: 0,
            };
            assert!(equal_nvim_string(d, e));
        }
    }

    #[test]
    fn test_roundup32() {
        assert_eq!(roundup32(0), 0); // wrapping: 0-1 = MAX → all bits set → MAX+1 = 0
        assert_eq!(roundup32(1), 1);
        assert_eq!(roundup32(2), 2);
        assert_eq!(roundup32(3), 4);
        assert_eq!(roundup32(5), 8);
        assert_eq!(roundup32(16), 16);
        assert_eq!(roundup32(17), 32);
    }

    #[test]
    fn test_mh_put_status_values() {
        assert_eq!(MHPutStatus::Existing as i32, 0);
        assert_eq!(MHPutStatus::NewKeyDidFit as i32, 1);
        assert_eq!(MHPutStatus::NewKeyRealloc as i32, 2);
    }

    #[test]
    fn test_hash_bytes_color_key() {
        unsafe {
            let k1 = ColorKey {
                ns_id: 1,
                syn_id: 2,
            };
            let k2 = ColorKey {
                ns_id: 1,
                syn_id: 2,
            };
            let k3 = ColorKey {
                ns_id: 1,
                syn_id: 3,
            };
            assert_eq!(hash_bytes(&k1), hash_bytes(&k2));
            assert_ne!(hash_bytes(&k1), hash_bytes(&k3));
            assert!(equal_bytes(&k1, &k2));
            assert!(!equal_bytes(&k1, &k3));
        }
    }
}
