//! Random number generation functions for VimL.
//!
//! This module implements the PRNG helpers used by `rand()` and `srand()`:
//! - `splitmix32()` - initialization helper
//! - `shuffle_xoshiro128starstar()` - main PRNG algorithm

#![allow(clippy::many_single_char_names)]

/// SplitMix32 hash function used for PRNG state initialization.
///
/// This is a simple hash function that produces well-distributed values
/// from a simple state variable.
#[inline]
pub fn splitmix32(x: &mut u32) -> u32 {
    *x = x.wrapping_add(0x9e37_79b9);
    let mut z = *x;
    z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
    z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
    z ^ (z >> 16)
}

/// Rotate left helper for xoshiro128**
#[inline]
const fn rotl(x: u32, k: u32) -> u32 {
    (x << k) | (x >> (32 - k))
}

/// Xoshiro128** PRNG algorithm.
///
/// A fast, high-quality PRNG with a period of 2^128 - 1.
/// The state consists of four 32-bit integers that are modified in place.
#[inline]
pub fn shuffle_xoshiro128starstar(x: &mut u32, y: &mut u32, z: &mut u32, w: &mut u32) -> u32 {
    let result = rotl(y.wrapping_mul(5), 7).wrapping_mul(9);
    let t = *y << 9;
    *z ^= *x;
    *w ^= *y;
    *y ^= *z;
    *x ^= *w;
    *z ^= t;
    *w = rotl(*w, 11);
    result
}

// =============================================================================
// FFI Interface
// =============================================================================

use std::ffi::c_uint;

/// FFI wrapper for splitmix32
///
/// # Safety
/// `x` must be a valid pointer to a u32.
#[no_mangle]
pub unsafe extern "C" fn rs_splitmix32(x: *mut c_uint) -> c_uint {
    if x.is_null() {
        return 0;
    }
    let x_ref = &mut *x;
    splitmix32(x_ref)
}

/// FFI wrapper for shuffle_xoshiro128starstar
///
/// # Safety
/// All pointers must be valid pointers to u32 values.
#[no_mangle]
pub unsafe extern "C" fn rs_shuffle_xoshiro128starstar(
    x: *mut c_uint,
    y: *mut c_uint,
    z: *mut c_uint,
    w: *mut c_uint,
) -> c_uint {
    if x.is_null() || y.is_null() || z.is_null() || w.is_null() {
        return 0;
    }
    shuffle_xoshiro128starstar(&mut *x, &mut *y, &mut *z, &mut *w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splitmix32_deterministic() {
        let mut x1 = 12345u32;
        let mut x2 = 12345u32;

        let r1 = splitmix32(&mut x1);
        let r2 = splitmix32(&mut x2);

        assert_eq!(r1, r2);
        assert_eq!(x1, x2);
    }

    #[test]
    fn test_splitmix32_changes_state() {
        let mut x = 0u32;
        let original = x;

        splitmix32(&mut x);

        assert_ne!(x, original);
    }

    #[test]
    fn test_xoshiro_deterministic() {
        let (mut x1, mut y1, mut z1, mut w1) = (1u32, 2u32, 3u32, 4u32);
        let (mut x2, mut y2, mut z2, mut w2) = (1u32, 2u32, 3u32, 4u32);

        let r1 = shuffle_xoshiro128starstar(&mut x1, &mut y1, &mut z1, &mut w1);
        let r2 = shuffle_xoshiro128starstar(&mut x2, &mut y2, &mut z2, &mut w2);

        assert_eq!(r1, r2);
        assert_eq!((x1, y1, z1, w1), (x2, y2, z2, w2));
    }

    #[test]
    fn test_xoshiro_produces_variety() {
        let (mut x, mut y, mut z, mut w) = (1u32, 2u32, 3u32, 4u32);

        let r1 = shuffle_xoshiro128starstar(&mut x, &mut y, &mut z, &mut w);
        let r2 = shuffle_xoshiro128starstar(&mut x, &mut y, &mut z, &mut w);
        let r3 = shuffle_xoshiro128starstar(&mut x, &mut y, &mut z, &mut w);

        // These should all be different
        assert_ne!(r1, r2);
        assert_ne!(r2, r3);
    }

    #[test]
    fn test_rotl() {
        assert_eq!(rotl(0b1, 1), 0b10);
        assert_eq!(rotl(0b1000_0000_0000_0000_0000_0000_0000_0000, 1), 0b1);
        assert_eq!(rotl(0xFFFF_FFFF, 16), 0xFFFF_FFFF);
    }
}
