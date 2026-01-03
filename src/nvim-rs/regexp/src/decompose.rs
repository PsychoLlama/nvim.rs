//! Hebrew character decomposition for case-insensitive matching.
//!
//! This module handles decomposition of Hebrew presentation forms
//! (U+FB20 to U+FB4F) into their constituent base characters and
//! combining marks.

use std::ffi::c_int;

// =============================================================================
// Decomposition Table
// =============================================================================

/// Hebrew decomposition entry: (base, combining1, combining2)
type DecompEntry = (c_int, c_int, c_int);

/// Decomposition table for Hebrew presentation forms (U+FB20 - U+FB4F).
/// Each entry contains up to 3 components of the decomposed form.
#[rustfmt::skip]
static DECOMP_TABLE: [DecompEntry; 48] = [
    (0x5e2, 0, 0),          // 0xfb20  alt ayin
    (0x5d0, 0, 0),          // 0xfb21  alt alef
    (0x5d3, 0, 0),          // 0xfb22  alt dalet
    (0x5d4, 0, 0),          // 0xfb23  alt he
    (0x5db, 0, 0),          // 0xfb24  alt kaf
    (0x5dc, 0, 0),          // 0xfb25  alt lamed
    (0x5dd, 0, 0),          // 0xfb26  alt mem-sofit
    (0x5e8, 0, 0),          // 0xfb27  alt resh
    (0x5ea, 0, 0),          // 0xfb28  alt tav
    (b'+' as c_int, 0, 0),  // 0xfb29  alt plus
    (0x5e9, 0x5c1, 0),      // 0xfb2a  shin+shin-dot
    (0x5e9, 0x5c2, 0),      // 0xfb2b  shin+sin-dot
    (0x5e9, 0x5c1, 0x5bc),  // 0xfb2c  shin+shin-dot+dagesh
    (0x5e9, 0x5c2, 0x5bc),  // 0xfb2d  shin+sin-dot+dagesh
    (0x5d0, 0x5b7, 0),      // 0xfb2e  alef+patah
    (0x5d0, 0x5b8, 0),      // 0xfb2f  alef+qamats
    (0x5d0, 0x5b4, 0),      // 0xfb30  alef+hiriq
    (0x5d1, 0x5bc, 0),      // 0xfb31  bet+dagesh
    (0x5d2, 0x5bc, 0),      // 0xfb32  gimel+dagesh
    (0x5d3, 0x5bc, 0),      // 0xfb33  dalet+dagesh
    (0x5d4, 0x5bc, 0),      // 0xfb34  he+dagesh
    (0x5d5, 0x5bc, 0),      // 0xfb35  vav+dagesh
    (0x5d6, 0x5bc, 0),      // 0xfb36  zayin+dagesh
    (0xfb37, 0, 0),         // 0xfb37  -- UNUSED
    (0x5d8, 0x5bc, 0),      // 0xfb38  tet+dagesh
    (0x5d9, 0x5bc, 0),      // 0xfb39  yud+dagesh
    (0x5da, 0x5bc, 0),      // 0xfb3a  kaf sofit+dagesh
    (0x5db, 0x5bc, 0),      // 0xfb3b  kaf+dagesh
    (0x5dc, 0x5bc, 0),      // 0xfb3c  lamed+dagesh
    (0xfb3d, 0, 0),         // 0xfb3d  -- UNUSED
    (0x5de, 0x5bc, 0),      // 0xfb3e  mem+dagesh
    (0xfb3f, 0, 0),         // 0xfb3f  -- UNUSED
    (0x5e0, 0x5bc, 0),      // 0xfb40  nun+dagesh
    (0x5e1, 0x5bc, 0),      // 0xfb41  samech+dagesh
    (0xfb42, 0, 0),         // 0xfb42  -- UNUSED
    (0x5e3, 0x5bc, 0),      // 0xfb43  pe sofit+dagesh
    (0x5e4, 0x5bc, 0),      // 0xfb44  pe+dagesh
    (0xfb45, 0, 0),         // 0xfb45  -- UNUSED
    (0x5e6, 0x5bc, 0),      // 0xfb46  tsadi+dagesh
    (0x5e7, 0x5bc, 0),      // 0xfb47  qof+dagesh
    (0x5e8, 0x5bc, 0),      // 0xfb48  resh+dagesh
    (0x5e9, 0x5bc, 0),      // 0xfb49  shin+dagesh
    (0x5ea, 0x5bc, 0),      // 0xfb4a  tav+dagesh
    (0x5d5, 0x5b9, 0),      // 0xfb4b  vav+holam
    (0x5d1, 0x5bf, 0),      // 0xfb4c  bet+rafe
    (0x5db, 0x5bf, 0),      // 0xfb4d  kaf+rafe
    (0x5e4, 0x5bf, 0),      // 0xfb4e  pe+rafe
    (0x5d0, 0x5dc, 0),      // 0xfb4f  alef-lamed
];

/// Range for Hebrew presentation forms
const DECOMP_START: c_int = 0xfb20;
const DECOMP_END: c_int = 0xfb4f;

// =============================================================================
// Decomposition Functions
// =============================================================================

/// Decompose a Hebrew presentation form character into its components.
///
/// For characters in the range U+FB20 to U+FB4F, returns the decomposition
/// as up to 3 components. For characters outside this range, returns the
/// character itself with zeros for the remaining components.
#[inline]
pub fn mb_decompose(c: c_int) -> (c_int, c_int, c_int) {
    if (DECOMP_START..=DECOMP_END).contains(&c) {
        DECOMP_TABLE[(c - DECOMP_START) as usize]
    } else {
        (c, 0, 0)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Decompose a Hebrew presentation form character.
///
/// # Safety
/// c1, c2, c3 must be valid pointers to writable memory.
#[no_mangle]
pub unsafe extern "C" fn rs_mb_decompose(c: c_int, c1: *mut c_int, c2: *mut c_int, c3: *mut c_int) {
    let (a, b, d) = mb_decompose(c);
    *c1 = a;
    *c2 = b;
    *c3 = d;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decomp_table_size() {
        assert_eq!(DECOMP_TABLE.len(), 48);
        assert_eq!(DECOMP_END - DECOMP_START + 1, 48);
    }

    #[test]
    fn test_alt_ayin() {
        // 0xfb20 = alt ayin -> 0x5e2 (ayin)
        let (c1, c2, c3) = mb_decompose(0xfb20);
        assert_eq!(c1, 0x5e2);
        assert_eq!(c2, 0);
        assert_eq!(c3, 0);
    }

    #[test]
    fn test_shin_shin_dot_dagesh() {
        // 0xfb2c = shin + shin-dot + dagesh
        let (c1, c2, c3) = mb_decompose(0xfb2c);
        assert_eq!(c1, 0x5e9); // shin
        assert_eq!(c2, 0x5c1); // shin-dot
        assert_eq!(c3, 0x5bc); // dagesh
    }

    #[test]
    fn test_alef_lamed() {
        // 0xfb4f = alef-lamed ligature
        let (c1, c2, c3) = mb_decompose(0xfb4f);
        assert_eq!(c1, 0x5d0); // alef
        assert_eq!(c2, 0x5dc); // lamed
        assert_eq!(c3, 0);
    }

    #[test]
    fn test_unused_entry() {
        // 0xfb37 is unused, returns itself
        let (c1, c2, c3) = mb_decompose(0xfb37);
        assert_eq!(c1, 0xfb37);
        assert_eq!(c2, 0);
        assert_eq!(c3, 0);
    }

    #[test]
    fn test_out_of_range() {
        // ASCII character - not in range
        let (c1, c2, c3) = mb_decompose(b'A' as c_int);
        assert_eq!(c1, b'A' as c_int);
        assert_eq!(c2, 0);
        assert_eq!(c3, 0);

        // Character before range
        let (c1, c2, c3) = mb_decompose(0xfb1f);
        assert_eq!(c1, 0xfb1f);
        assert_eq!(c2, 0);
        assert_eq!(c3, 0);

        // Character after range
        let (c1, c2, c3) = mb_decompose(0xfb50);
        assert_eq!(c1, 0xfb50);
        assert_eq!(c2, 0);
        assert_eq!(c3, 0);
    }

    #[test]
    fn test_bet_dagesh() {
        // 0xfb31 = bet + dagesh
        let (c1, c2, c3) = mb_decompose(0xfb31);
        assert_eq!(c1, 0x5d1); // bet
        assert_eq!(c2, 0x5bc); // dagesh
        assert_eq!(c3, 0);
    }

    #[test]
    fn test_alt_plus() {
        // 0xfb29 = alt plus
        let (c1, c2, c3) = mb_decompose(0xfb29);
        assert_eq!(c1, b'+' as c_int);
        assert_eq!(c2, 0);
        assert_eq!(c3, 0);
    }
}
