#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
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
//! `ScreenChar`.
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
use core::mem::MaybeUninit;
use core::slice;

use crate::cstr::until_nul;
use crate::map::{MH_TOMBSTONE, MapKey, grown_keys_capacity, mh_clear, mh_realloc, probe};
use crate::memory::xrealloc;
use crate::types::{Set_glyph, uint32_t};

/// A glyph's length as the index counts it: bytes, and never near 2^32
/// because [`crate::grid::MAX_SCHAR_SIZE`] caps it at 32.
fn key_len(key: &[u8]) -> uint32_t {
    uint32_t::try_from(key.len()).expect("a glyph is at most MAX_SCHAR_SIZE bytes")
}

impl Set_glyph {
    /// The bucket table: `h.n_buckets` slots, each 0 (empty), a tombstone,
    /// or a glyph's byte offset plus one.
    fn buckets(&self) -> &[uint32_t] {
        if self.h.hash.is_null() {
            return &[];
        }
        // SAFETY: a live set's `h.hash` is its own allocation of
        // `h.n_buckets` slots, which `mh_realloc` keeps in step.
        unsafe { slice::from_raw_parts(self.h.hash, self.h.n_buckets as usize) }
    }

    /// [`Set_glyph::buckets`], writable.
    fn buckets_mut(&mut self) -> &mut [uint32_t] {
        if self.h.hash.is_null() {
            return &mut [];
        }
        // SAFETY: as `buckets`, and the borrow of `self` is exclusive.
        unsafe { slice::from_raw_parts_mut(self.h.hash, self.h.n_buckets as usize) }
    }

    /// The packed run of interned glyphs: `n_keys` bytes, each glyph
    /// followed by its NUL.
    fn packed_keys(&self) -> &[u8] {
        if self.keys.is_null() {
            return &[];
        }
        // SAFETY: a live set's `keys` holds `h.n_keys` bytes of
        // NUL-terminated glyphs -- what [`Set_glyph::intern`] wrote -- and
        // the borrow of `self` keeps them from being grown under the slice.
        unsafe { slice::from_raw_parts(self.keys.cast::<u8>(), self.h.n_keys as usize) }
    }

    /// The glyph stored at byte offset `pos`, without its NUL.
    ///
    /// A checked index. Upstream reads the arena at whatever offset a
    /// `ScreenChar` carries, guarded by a debug-only assertion; the
    /// assertion is kept, and a release build answers the empty glyph for an
    /// offset past the end -- one a stale `ScreenChar` can still carry after
    /// [`Set_glyph::clear`] -- where upstream would read past the arena.
    pub(crate) fn glyph(&self, pos: uint32_t) -> &[u8] {
        let keys = self.packed_keys();
        debug_assert!((pos as usize) < keys.len(), "idx < n_keys");
        keys.get(pos as usize..).map_or(&[], until_nul)
    }

    /// Bytes of interned glyphs: every valid offset is below it.
    pub(crate) fn packed_len(&self) -> uint32_t {
        self.h.n_keys
    }

    /// The bucket `key` belongs in. See [`crate::map::probe`].
    fn find_bucket(&self, key: &[u8], put: bool) -> uint32_t {
        let keys = self.packed_keys();
        probe(
            self.buckets(),
            self.h.n_buckets - 1,
            key.map_hash(),
            put,
            |pos| {
                // A live bucket holds the one-based byte offset of a
                // NUL-terminated glyph in `keys`.
                until_nul(&keys[pos as usize..]) == key
            },
        )
    }

    /// Re-point every bucket at its glyph after the table was resized: the
    /// table must be all zero.
    ///
    /// The walk is over the packed keys rather than an index, so it steps by
    /// each glyph's own length: `strlen + 1`.
    fn rehash(&mut self) {
        let n_keys = self.h.n_keys;
        let mut at = 0;
        while at < n_keys {
            let (idx, len) = {
                let key = until_nul(&self.packed_keys()[at as usize..]);
                (self.find_bucket(key, true), key_len(key))
            };
            let slot = &mut self.buckets_mut()[idx as usize];
            assert!(
                *slot == 0,
                "glyph cache: rehash landed on an occupied bucket"
            );
            *slot = at + 1;
            at += len + 1;
        }
        // Nothing was deleted, so every key is live and no bucket is a
        // tombstone.
        self.h.size = n_keys;
        self.h.n_occupied = n_keys;
    }

    /// Intern `key`, or find the copy already there, answering its byte
    /// offset. `key` must hold no NUL and be shorter than
    /// [`crate::grid::MAX_SCHAR_SIZE`].
    pub(crate) fn intern(&mut self, key: &[u8]) -> uint32_t {
        // The keys array's first heap size. Bytes, not entries, so it is
        // eight times the fixed-size sets' floor -- upstream's own number for
        // this layout. Function-local so it stays out of the FFI golden.
        const MIN_KEY_BYTES: uint32_t = 64;

        if self.h.n_occupied >= self.h.upper_bound {
            // The cache never deletes, so there are no tombstones to shed and
            // no `should_grow` decision to make: growing is the only way back
            // under the bound.
            // SAFETY: the set's own index. `mh_realloc` leaves an all-zero
            // table, which is what `rehash` needs.
            unsafe { mh_realloc(&raw mut self.h, self.h.n_buckets + 1) };
            self.rehash();
        }

        let idx = self.find_bucket(key, true);
        let entry = self.buckets()[idx as usize];
        if entry != 0 && entry != MH_TOMBSTONE {
            let pos = entry - 1;
            debug_assert!(self.glyph(pos) == key);
            return pos;
        }

        let width = key_len(key) + 1;
        self.h.size += 1;
        self.h.n_occupied += 1;
        let pos = self.h.n_keys;
        self.h.n_keys += width;

        if self.h.n_keys > self.h.keys_capacity {
            // `keys_capacity` counts bytes here rather than entries.
            self.h.keys_capacity = grown_keys_capacity(self.h.keys_capacity, MIN_KEY_BYTES);
            let bytes = self.h.keys_capacity as usize * size_of::<c_char>();
            // SAFETY: `xrealloc` grows the set's own allocation.
            self.keys = unsafe { xrealloc(self.keys.cast::<c_void>(), bytes) }.cast::<c_char>();
        }

        // SAFETY: `pos + width` is within the capacity just checked or grown
        // to; the bytes are not yet written, which is why they are
        // `MaybeUninit`.
        let room = unsafe {
            slice::from_raw_parts_mut(
                self.keys.add(pos as usize).cast::<MaybeUninit<u8>>(),
                width as usize,
            )
        };
        for (dst, &byte) in room.iter_mut().zip(key) {
            dst.write(byte);
        }
        room[key.len()].write(0);
        self.buckets_mut()[idx as usize] = pos + 1;
        pos
    }

    /// Forget every glyph: the arena is kept, its contents are not.
    pub(crate) fn clear(&mut self) {
        // SAFETY: the set's own index.
        unsafe { mh_clear(&raw mut self.h) };
    }
}
