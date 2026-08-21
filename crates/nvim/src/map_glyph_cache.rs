#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The glyph cache's index: a `Set_glyph`, the one set in the tree whose
//! dense keys array is a *packed run of NUL-terminated byte strings* rather
//! than fixed-size entries. A "dense index" here is therefore a byte offset
//! into that run, which is what [`crate::grid`] stores in the high bits of a
//! `schar_T`.
//!
//! Upstream generates this from the same `map.c` macro as every other set,
//! with `MH_KEY_DECL`/`MH_KEY_SET` overridden for the packed layout. Only the
//! *key layout* differs, so the probe sequence and the growth policy are
//! [`crate::map`]'s own — shared rather than copied, because bucket placement
//! is observable through the raw arrays and the two must not drift.
//!
//! Derived, via upstream's `map.c`, from klib's `khash.h`, Copyright (c)
//! 2008, 2009, 2011 Attractive Chaos, under the MIT license; the notice is
//! reproduced in licenses/klib-LICENSE.txt.

use core::ffi::{c_char, c_void};
use core::slice;

use crate::api::private::helpers::cstr_as_string;
use crate::map::{
    MH_TOMBSTONE, MapKey, grown_keys_capacity, kMHExisting, kMHNewKeyDidFit, kMHNewKeyRealloc,
    mh_realloc, probe,
};
use crate::memory::xrealloc;
use crate::types::{MHPutStatus, Set_glyph, String_0, uint32_t};

/// A glyph's length as the index counts it: bytes, and never near 2^32
/// because [`crate::grid::MAX_SCHAR_SIZE`] caps it at 32.
fn key_len(key: String_0) -> uint32_t {
    uint32_t::try_from(key.len()).expect("a glyph is at most MAX_SCHAR_SIZE bytes")
}

/// The bucket `key` belongs in. See [`crate::map::probe`].
///
/// # Safety
/// `set` must point at a live `Set_glyph`: `h.hash` at `h.n_buckets` slots,
/// `keys` at `h.n_keys` bytes of NUL-terminated glyphs.
unsafe fn find_bucket(set: *const Set_glyph, key: String_0, put: bool) -> uint32_t {
    // SAFETY: the caller promises a live `Set_glyph`.
    let set = unsafe { &*set };
    // SAFETY: as above — `h.hash` points at `h.n_buckets` slots.
    let buckets = unsafe { slice::from_raw_parts(set.h.hash, set.h.n_buckets as usize) };
    probe(buckets, set.h.n_buckets - 1, key.map_hash(), put, |pos| {
        // SAFETY: a live bucket holds the one-based byte offset of a
        // NUL-terminated glyph in `keys`.
        unsafe { cstr_as_string(set.keys.add(pos as usize)) }.map_eq(&key)
    })
}

/// Re-point every bucket at its glyph after the table was resized.
///
/// The walk is over the packed keys rather than an index, so it steps by
/// each glyph's own length: `strlen + 1`.
///
/// # Safety
/// As [`find_bucket`], with an all-zero bucket table.
unsafe fn rehash(set: *mut Set_glyph) {
    // SAFETY: the caller promises a live `Set_glyph`.
    let (keys, hash, n_keys) = unsafe { ((*set).keys, (*set).h.hash, (*set).h.n_keys) };
    let mut at = 0;
    while at < n_keys {
        // SAFETY: `at` is the offset of a NUL-terminated glyph.
        let key = unsafe { cstr_as_string(keys.add(at as usize)) };
        // SAFETY: as above; the bucket table is all-zero.
        let idx = unsafe { find_bucket(set, key, true) };
        // SAFETY: `find_bucket` answers a slot of the live bucket table.
        let slot = unsafe { &mut *hash.add(idx as usize) };
        assert!(
            *slot == 0,
            "glyph cache: rehash landed on an occupied bucket"
        );
        *slot = at + 1;
        at += key_len(key) + 1;
    }
    // SAFETY: the caller promises a live `Set_glyph`. Nothing was deleted,
    // so every key is live and no bucket is a tombstone.
    unsafe {
        (*set).h.size = n_keys;
        (*set).h.n_occupied = n_keys;
    }
}

/// Intern `key`, or find the copy already there. Answers the byte offset of
/// the stored copy and reports through `status` whether it is new and whether
/// the keys array moved.
///
/// # Safety
/// As [`find_bucket`]; `status` must be writable, and `key` must have
/// [`len`](String_0::len) readable bytes.
pub unsafe fn mh_put_glyph(
    set: *mut Set_glyph,
    key: String_0,
    status: *mut MHPutStatus,
) -> uint32_t {
    // The keys array's first heap size. Bytes, not entries, so it is eight
    // times the fixed-size sets' floor -- upstream's own number for this
    // layout. Function-local so it stays out of the FFI golden.
    const MIN_KEY_BYTES: uint32_t = 64;

    // SAFETY: the caller promises a live `Set_glyph`.
    let h = unsafe { &raw mut (*set).h };
    // SAFETY: as above.
    if unsafe { (*h).n_occupied >= (*h).upper_bound } {
        // The cache never deletes, so there are no tombstones to shed and
        // no `should_grow` decision to make: growing is the only way back
        // under the bound.
        // SAFETY: as above; `mh_realloc` leaves an all-zero table, which is
        // what `rehash` needs.
        unsafe {
            mh_realloc(h, (*h).n_buckets + 1);
            rehash(set);
        }
    }

    // SAFETY: as above.
    let (idx, entry) = unsafe {
        let idx = find_bucket(set, key, true);
        (idx, *(*h).hash.add(idx as usize))
    };
    if entry != 0 && entry != MH_TOMBSTONE {
        let pos = entry - 1;
        // SAFETY: `status` is the caller's, and `pos` names a live glyph.
        unsafe {
            *status = kMHExisting;
            debug_assert!(cstr_as_string((*set).keys.add(pos as usize)).map_eq(&key));
        }
        return pos;
    }

    let width = key_len(key) + 1;
    // SAFETY: `h` is the live index; `pos` is where this glyph's bytes go.
    let pos = unsafe {
        (*h).size += 1;
        (*h).n_occupied += 1;
        let pos = (*h).n_keys;
        (*h).n_keys += width;
        pos
    };

    // SAFETY: as above. `xrealloc` grows the caller's own allocation, and
    // `keys_capacity` counts bytes here rather than entries.
    unsafe {
        if (*h).n_keys > (*h).keys_capacity {
            (*h).keys_capacity = grown_keys_capacity((*h).keys_capacity, MIN_KEY_BYTES);
            let bytes = (*h).keys_capacity as usize * size_of::<c_char>();
            (*set).keys = xrealloc((*set).keys.cast::<c_void>(), bytes).cast::<c_char>();
            *status = kMHNewKeyRealloc;
        } else {
            *status = kMHNewKeyDidFit;
        }
    }

    // SAFETY: `pos + width` is within the capacity just checked or grown to,
    // and `key` has `key.len()` readable bytes.
    unsafe {
        let room =
            slice::from_raw_parts_mut((*set).keys.add(pos as usize).cast::<u8>(), width as usize);
        let bytes = key.as_bytes();
        room[..bytes.len()].copy_from_slice(bytes);
        room[bytes.len()] = 0;
        *(*h).hash.add(idx as usize) = pos + 1;
    }
    pos
}
