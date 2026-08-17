#![forbid(unsafe_code)]

//! The memo tables in front of the attribute set.
//!
//! Combining or blending two attribute sets is a pure function of the pair,
//! and the screen asks for the same pairs relentlessly — once per cell of a
//! spelling error, once per cell under a floating window. Each *answer* is a
//! new entry in the attribute table, so without a memo the table would fill
//! with duplicates and hit `MAX_TYPENR` in seconds.
//!
//! One cache per operation ([`combine`](super::hl_combine_attr) and the two
//! directions of [`blend`](super::blend)); the key is the pair of ids packed
//! into a `u64`, which is what upstream's `HlAttrKey` did too.

use core::ffi::c_int;
use core::hash::{BuildHasherDefault, Hasher};
use std::collections::HashMap;

/// Mixes a packed id pair so that the top bits — which the table probes
/// first — depend on all of it. Ids are small and consecutive, which the
/// identity hash spreads badly, and this sits on the per-cell path where
/// SipHash's setup is not worth paying.
#[derive(Default)]
struct AttrPairHasher(u64);

impl Hasher for AttrPairHasher {
    fn write_u64(&mut self, n: u64) {
        // splitmix64's finalizer.
        let mut z = n;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        self.0 = z ^ (z >> 31);
    }

    fn write(&mut self, bytes: &[u8]) {
        // Unreached: the only key type is `u64`. Defined anyway, since the
        // trait lets any `Hash` impl route here.
        for &b in bytes {
            self.write_u64(self.0.wrapping_mul(31).wrapping_add(u64::from(b)));
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

/// Attribute ids remembered by the pair they came from.
pub struct AttrCache(HashMap<u64, c_int, BuildHasherDefault<AttrPairHasher>>);

impl AttrCache {
    pub const fn new() -> Self {
        Self(HashMap::with_hasher(BuildHasherDefault::new()))
    }

    /// What `(first, second)` resolved to last time, or 0 for "not asked
    /// yet". Id 0 is the empty attribute set, which no combination ever
    /// produces, so it doubles as absent — as upstream's zero-valued map
    /// default did.
    #[inline]
    pub fn get(&self, first: c_int, second: c_int) -> c_int {
        self.0.get(&pair(first, second)).copied().unwrap_or(0)
    }

    #[inline]
    pub fn insert(&mut self, first: c_int, second: c_int, id: c_int) {
        self.0.insert(pair(first, second), id);
    }

    /// Forget every answer, keeping the allocation. Called when the
    /// attribute table they index into is rebuilt.
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl Default for AttrCache {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
const fn pair(first: c_int, second: c_int) -> u64 {
    ((first as u32 as u64) << 32) | second as u32 as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unasked_pair_reads_as_zero_and_the_two_halves_do_not_swap() {
        let mut cache = AttrCache::new();
        assert_eq!(cache.get(3, 4), 0);
        cache.insert(3, 4, 7);
        assert_eq!(cache.get(3, 4), 7);
        assert_eq!(cache.get(4, 3), 0);
        cache.clear();
        assert_eq!(cache.get(3, 4), 0);
    }
}
