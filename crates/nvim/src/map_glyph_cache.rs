#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The glyph cache's index: the one khash set in the tree whose dense keys
//! array is a *packed run of NUL-terminated byte strings* rather than
//! fixed-size entries. A "dense index" here is therefore a byte offset into
//! that run, which is what [`crate::grid`] stores in the high bits of a
//! `ScreenChar`.
//!
//! Upstream generates this from the same `map.c` macro as every other set,
//! with `MH_KEY_DECL`/`MH_KEY_SET` overridden for the packed layout. Only the
//! *key layout* differs, so the probe sequence and the growth policy are
//! [`crate::map`]'s own [`ProbeTable`], shared rather than copied.
//!
//! Derived, via upstream's `map.c`, from klib's `khash.h`, Copyright (c)
//! 2008, 2009, 2011 Attractive Chaos, under the MIT license; the notice is
//! reproduced in licenses/klib-LICENSE.txt.

use crate::cstr::until_nul;
use crate::map::{MapKey, ProbeTable};

/// A glyph's length as the index counts it: bytes, and never near 2^32
/// because [`crate::grid::MAX_SCHAR_SIZE`] caps it at 32.
fn key_len(key: &[u8]) -> u32 {
    u32::try_from(key.len()).expect("a glyph is at most MAX_SCHAR_SIZE bytes")
}

/// Interned glyphs, each named by the byte offset it starts at.
pub(crate) struct GlyphCache {
    table: ProbeTable,
    /// The packed run: every glyph followed by its NUL.
    keys: Vec<u8>,
}

impl GlyphCache {
    /// Nothing interned and nothing allocated.
    pub(crate) const fn new() -> Self {
        GlyphCache {
            table: ProbeTable::new(),
            keys: Vec::new(),
        }
    }

    /// The glyph stored at byte offset `pos`, without its NUL.
    ///
    /// A checked index. Upstream reads the arena at whatever offset a
    /// `ScreenChar` carries, guarded by a debug-only assertion; the
    /// assertion is kept, and a release build answers the empty glyph for an
    /// offset past the end -- one a stale `ScreenChar` can still carry after
    /// [`GlyphCache::clear`] -- where upstream would read past the arena.
    pub(crate) fn glyph(&self, pos: u32) -> &[u8] {
        debug_assert!((pos as usize) < self.keys.len(), "idx < n_keys");
        self.keys.get(pos as usize..).map_or(&[], until_nul)
    }

    /// Bytes of interned glyphs: every valid offset is below it.
    pub(crate) fn packed_len(&self) -> u32 {
        u32::try_from(self.keys.len()).expect("the cache is cleared long before 2^32 bytes")
    }

    /// The bucket `key` belongs in. See [`crate::map::probe`].
    fn find_bucket(table: &ProbeTable, keys: &[u8], key: &[u8]) -> u32 {
        // A live bucket holds the one-based byte offset of a NUL-terminated
        // glyph in `keys`.
        table.find(key.map_hash(), true, |pos| {
            until_nul(&keys[pos as usize..]) == key
        })
    }

    /// Re-point every bucket at its glyph after the table was resized.
    ///
    /// The walk is over the packed keys rather than an index, so it steps by
    /// each glyph's own length: `strlen + 1`.
    fn rehash(&mut self) {
        let keys = &self.keys;
        let mut at = 0;
        while (at as usize) < keys.len() {
            let key = until_nul(&keys[at as usize..]);
            self.table.rehash(key.map_hash(), at, |pos| {
                until_nul(&keys[pos as usize..]) == key
            });
            at += key_len(key) + 1;
        }
    }

    /// Intern `key`, or find the copy already there, answering its byte
    /// offset. `key` must hold no NUL and be shorter than
    /// [`crate::grid::MAX_SCHAR_SIZE`].
    pub(crate) fn intern(&mut self, key: &[u8]) -> u32 {
        if self.table.make_room() {
            self.rehash();
        }

        let idx = Self::find_bucket(&self.table, &self.keys, key);
        if let Some(pos) = self.table.entry(idx) {
            debug_assert!(self.glyph(pos) == key);
            return pos;
        }

        let pos = self.packed_len();
        self.keys.extend_from_slice(key);
        self.keys.push(0);
        self.table.occupy(idx, pos);
        pos
    }

    /// Forget every glyph: the allocations are kept, their contents are
    /// not.
    pub(crate) fn clear(&mut self) {
        self.table.clear();
        self.keys.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A glyph's index is the byte offset it was packed at, the same glyph
    /// interned again answers the same offset, and growing the bucket table
    /// moves none of them.
    #[test]
    fn offsets_are_packing_order_and_survive_growth() {
        let mut cache = GlyphCache::new();
        let glyphs: Vec<Vec<u8>> = (1..=40u8)
            .map(|i| vec![b'a' + i % 26, i, 0xcc, 0x80, i])
            .collect();
        let mut at = 0;
        for glyph in &glyphs {
            assert_eq!(cache.intern(glyph), at);
            at += key_len(glyph) + 1;
        }
        assert_eq!(cache.packed_len(), at);
        let mut at = 0;
        for glyph in &glyphs {
            assert_eq!(cache.intern(glyph), at);
            assert_eq!(cache.glyph(at), glyph.as_slice());
            at += key_len(glyph) + 1;
        }
        cache.clear();
        assert_eq!(cache.packed_len(), 0);
        assert_eq!(cache.intern(&glyphs[7]), 0);
    }
}
