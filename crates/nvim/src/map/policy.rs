#![forbid(unsafe_code)]

//! The sizing arithmetic of the khash-derived index: how many buckets a
//! request rounds up to, when the table has to grow rather than just shed its
//! tombstones, and how the dense keys array grows. All of it is upstream's,
//! bit for bit — the numbers decide where keys land, and callers walk the
//! arrays directly.
//!
//! Derived, via upstream's `map.c`, from klib's `khash.h`, Copyright (c)
//! 2008, 2009, 2011 Attractive Chaos, under the MIT license; the notice is
//! reproduced in licenses/klib-LICENSE.txt.

use crate::types::uint32_t;

mod limits {
    use crate::types::uint32_t;

    /// Smallest bucket table. Below this the probe sequence is not worth it.
    pub(super) const MIN_BUCKETS: uint32_t = 16;

    /// Occupancy at which the table is considered full. `n_occupied` counts
    /// tombstones as well as live entries.
    pub(super) const UPPER_FILL: f64 = 0.77;

    /// A table whose live entries are at least this fraction of the upper
    /// bound has genuinely outgrown it; below it, the pressure is tombstones.
    pub(super) const GROW_AT: f64 = 0.9;

    /// The dense keys array's first heap size.
    pub(crate) const MIN_KEYS: uint32_t = 8;
}

pub(super) use limits::MIN_KEYS;

/// Round a request up to a power of two, never below `MIN_BUCKETS`.
pub(super) fn bucket_count(n_min_buckets: uint32_t) -> uint32_t {
    let n = n_min_buckets.max(limits::MIN_BUCKETS);
    // `next_power_of_two` of an exact power of two is itself, which is what
    // the C's decrement-smear-increment produces.
    n.next_power_of_two()
}

/// How many buckets may be occupied before the table is resized.
pub(super) fn upper_bound(n_buckets: uint32_t) -> uint32_t {
    (n_buckets as f64 * limits::UPPER_FILL + 0.5) as uint32_t
}

/// At the upper bound: grow the table, or just drop the tombstones and rehash
/// in place?
pub(super) fn should_grow(size: uint32_t, upper_bound: uint32_t) -> bool {
    size as f64 >= upper_bound as f64 * limits::GROW_AT
}

/// The next capacity of a dense array that has just run out. `floor` is the
/// first heap size — 8 for a keys array of fixed-size entries.
pub fn grown_keys_capacity(capacity: uint32_t, floor: uint32_t) -> uint32_t {
    capacity.wrapping_mul(2).max(floor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_counts_round_up_to_a_power_of_two() {
        assert_eq!(bucket_count(0), 16);
        assert_eq!(bucket_count(1), 16);
        assert_eq!(bucket_count(16), 16);
        assert_eq!(bucket_count(17), 32);
        assert_eq!(bucket_count(33), 64);
        assert_eq!(bucket_count(1 << 20), 1 << 20);
        assert_eq!(bucket_count((1 << 20) + 1), 1 << 21);
    }

    /// `n_buckets + 1` is how a full table asks for the next size, so the
    /// rounding has to double rather than stay put.
    #[test]
    fn asking_for_one_more_than_a_power_of_two_doubles() {
        let mut n = bucket_count(0);
        for expected in [32, 64, 128, 256] {
            n = bucket_count(n + 1);
            assert_eq!(n, expected);
        }
    }

    /// 0.77 of the table, rounded to nearest.
    #[test]
    fn upper_bound_is_the_load_factor_rounded() {
        assert_eq!(upper_bound(16), 12); // 12.32
        assert_eq!(upper_bound(32), 25); // 24.64
        assert_eq!(upper_bound(64), 49); // 49.28
        assert_eq!(upper_bound(1024), 788); // 788.48 + 0.5 truncates to 788
    }

    /// The distinction that keeps a delete-heavy table from growing without
    /// bound: at the occupancy limit, a table that is mostly tombstones is
    /// rehashed in place instead.
    #[test]
    fn a_tombstone_heavy_table_is_rehashed_rather_than_grown() {
        let bound = upper_bound(64); // 49
        assert!(should_grow(49, bound));
        assert!(should_grow(45, bound)); // 45 >= 44.1
        assert!(!should_grow(44, bound));
        assert!(!should_grow(0, bound));
    }

    #[test]
    fn keys_capacity_doubles_from_a_floor() {
        assert_eq!(grown_keys_capacity(0, MIN_KEYS), 8);
        assert_eq!(grown_keys_capacity(8, MIN_KEYS), 16);
        assert_eq!(grown_keys_capacity(16, MIN_KEYS), 32);
        // The glyph cache uses a byte array with a larger floor.
        assert_eq!(grown_keys_capacity(0, 64), 64);
        assert_eq!(grown_keys_capacity(64, 64), 128);
    }
}
