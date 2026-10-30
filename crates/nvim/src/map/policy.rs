#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The sizing arithmetic of the khash-derived index: how many buckets a
//! request rounds up to, and how full they may get. Both are upstream's, bit
//! for bit — the numbers decide where keys land.
//!
//! Derived, via upstream's `map.c`, from klib's `khash.h`, Copyright (c)
//! 2008, 2009, 2011 Attractive Chaos, under the MIT license; the notice is
//! reproduced in licenses/klib-LICENSE.txt.

use crate::narrow::float_as_i64;

mod limits {
    /// Smallest bucket table. Below this the probe sequence is not worth it.
    pub(super) const MIN_BUCKETS: u32 = 16;

    /// Occupancy at which the table is considered full.
    pub(super) const UPPER_FILL: f64 = 0.77;
}

/// Round a request up to a power of two, never below `MIN_BUCKETS`.
pub(super) fn bucket_count(n_min_buckets: u32) -> u32 {
    let n = n_min_buckets.max(limits::MIN_BUCKETS);
    // `next_power_of_two` of an exact power of two is itself, which is what
    // the C's decrement-smear-increment produces.
    n.next_power_of_two()
}

/// How many buckets may be occupied before the table is resized.
pub(super) fn upper_bound(n_buckets: u32) -> u32 {
    let bound = float_as_i64(f64::from(n_buckets) * limits::UPPER_FILL + 0.5);
    u32::try_from(bound).expect("the fill fraction keeps the bound inside a u32")
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
}
