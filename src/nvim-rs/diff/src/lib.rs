//! Diff option checking for Neovim
//!
//! This module provides Rust implementations for checking diff options
//! from the 'diffopt' setting.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]

use std::os::raw::c_int;

// Diff flags (from diff.c)
// These must match the C #define values exactly
const DIFF_FILLER: c_int = 0x001;
const DIFF_IBLANK: c_int = 0x002;
const DIFF_ICASE: c_int = 0x004;
const DIFF_IWHITE: c_int = 0x008;
const DIFF_IWHITEALL: c_int = 0x010;
const DIFF_IWHITEEOL: c_int = 0x020;
const DIFF_HORIZONTAL: c_int = 0x040;
const DIFF_VERTICAL: c_int = 0x080;
const DIFF_HIDDEN_OFF: c_int = 0x100;
const DIFF_INTERNAL: c_int = 0x200;
const DIFF_CLOSE_OFF: c_int = 0x400;
const DIFF_FOLLOWWRAP: c_int = 0x800;
const DIFF_LINEMATCH: c_int = 0x1000;

// C accessor for the static diff_flags variable
extern "C" {
    fn nvim_get_diff_flags() -> c_int;
    fn nvim_is_diffexpr_empty() -> bool;
}

/// Check if 'diffopt' contains "horizontal".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_horizontal() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_HORIZONTAL) != 0)
}

/// Check if 'diffopt' contains "hiddenoff".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_hiddenoff() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_HIDDEN_OFF) != 0)
}

/// Check if 'diffopt' contains "closeoff".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_closeoff() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_CLOSE_OFF) != 0)
}

/// Check if 'diffopt' contains "filler".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_filler() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_FILLER) != 0)
}

/// Return true if the options are set to use the internal diff library.
///
/// Note that if the internal diff failed for one of the buffers, the external
/// diff will be used anyway.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_internal() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_INTERNAL) != 0 && nvim_is_diffexpr_empty())
}

/// Check if 'diffopt' contains "vertical".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_vertical() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_VERTICAL) != 0)
}

/// Check if 'diffopt' contains "icase" (ignore case).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_icase() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_ICASE) != 0)
}

/// Check if 'diffopt' contains "iwhite" (ignore whitespace changes).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_iwhite() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_IWHITE) != 0)
}

/// Check if 'diffopt' contains "iwhiteall" (ignore all whitespace).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_iwhiteall() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_IWHITEALL) != 0)
}

/// Check if 'diffopt' contains "iwhiteeol" (ignore whitespace at EOL).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_iwhiteeol() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_IWHITEEOL) != 0)
}

/// Check if 'diffopt' contains "iblank" (ignore blank lines).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_iblank() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_IBLANK) != 0)
}

/// Check if 'diffopt' contains "followwrap" (follow wrap option).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_followwrap() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_FOLLOWWRAP) != 0)
}

/// Check if 'diffopt' contains "linematch" (match similar lines).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_linematch() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_LINEMATCH) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_flag_constants() {
        // Verify the diff flag constants match expected values from diff.c
        assert_eq!(DIFF_FILLER, 0x001);
        assert_eq!(DIFF_IBLANK, 0x002);
        assert_eq!(DIFF_ICASE, 0x004);
        assert_eq!(DIFF_IWHITE, 0x008);
        assert_eq!(DIFF_IWHITEALL, 0x010);
        assert_eq!(DIFF_IWHITEEOL, 0x020);
        assert_eq!(DIFF_HORIZONTAL, 0x040);
        assert_eq!(DIFF_VERTICAL, 0x080);
        assert_eq!(DIFF_HIDDEN_OFF, 0x100);
        assert_eq!(DIFF_INTERNAL, 0x200);
        assert_eq!(DIFF_CLOSE_OFF, 0x400);
        assert_eq!(DIFF_FOLLOWWRAP, 0x800);
        assert_eq!(DIFF_LINEMATCH, 0x1000);
    }

    #[test]
    fn test_diff_flags_are_distinct() {
        // Ensure all flags are distinct (no overlap)
        let flags = [
            DIFF_FILLER,
            DIFF_IBLANK,
            DIFF_ICASE,
            DIFF_IWHITE,
            DIFF_IWHITEALL,
            DIFF_IWHITEEOL,
            DIFF_HORIZONTAL,
            DIFF_VERTICAL,
            DIFF_HIDDEN_OFF,
            DIFF_INTERNAL,
            DIFF_CLOSE_OFF,
            DIFF_FOLLOWWRAP,
            DIFF_LINEMATCH,
        ];

        for i in 0..flags.len() {
            for j in (i + 1)..flags.len() {
                assert_eq!(
                    flags[i] & flags[j],
                    0,
                    "Flags at indices {i} and {j} overlap"
                );
            }
        }
    }

    #[test]
    fn test_diff_flags_are_single_bit() {
        // Each flag should be a single bit (power of 2)
        let flags = [
            DIFF_FILLER,
            DIFF_IBLANK,
            DIFF_ICASE,
            DIFF_IWHITE,
            DIFF_IWHITEALL,
            DIFF_IWHITEEOL,
            DIFF_HORIZONTAL,
            DIFF_VERTICAL,
            DIFF_HIDDEN_OFF,
            DIFF_INTERNAL,
            DIFF_CLOSE_OFF,
            DIFF_FOLLOWWRAP,
            DIFF_LINEMATCH,
        ];

        for flag in flags {
            // A power of 2 has exactly one bit set
            // n & (n - 1) == 0 for powers of 2
            assert_eq!(flag & (flag - 1), 0, "Flag {flag:#x} is not a power of 2");
            assert_ne!(flag, 0, "Flag should not be zero");
        }
    }

    #[test]
    fn test_diff_flag_bit_positions() {
        // Verify exact bit positions for each flag
        assert_eq!(DIFF_FILLER, 1 << 0); // bit 0
        assert_eq!(DIFF_IBLANK, 1 << 1); // bit 1
        assert_eq!(DIFF_ICASE, 1 << 2); // bit 2
        assert_eq!(DIFF_IWHITE, 1 << 3); // bit 3
        assert_eq!(DIFF_IWHITEALL, 1 << 4); // bit 4
        assert_eq!(DIFF_IWHITEEOL, 1 << 5); // bit 5
        assert_eq!(DIFF_HORIZONTAL, 1 << 6); // bit 6
        assert_eq!(DIFF_VERTICAL, 1 << 7); // bit 7
        assert_eq!(DIFF_HIDDEN_OFF, 1 << 8); // bit 8
        assert_eq!(DIFF_INTERNAL, 1 << 9); // bit 9
        assert_eq!(DIFF_CLOSE_OFF, 1 << 10); // bit 10
        assert_eq!(DIFF_FOLLOWWRAP, 1 << 11); // bit 11
        assert_eq!(DIFF_LINEMATCH, 1 << 12); // bit 12
    }

    #[test]
    fn test_diff_flag_combinations() {
        // Test that combining flags works correctly
        let combined = DIFF_FILLER | DIFF_HORIZONTAL | DIFF_INTERNAL | DIFF_ICASE;

        // Check each flag is set in the combination
        assert_ne!(combined & DIFF_FILLER, 0);
        assert_ne!(combined & DIFF_HORIZONTAL, 0);
        assert_ne!(combined & DIFF_INTERNAL, 0);
        assert_ne!(combined & DIFF_ICASE, 0);

        // Check other flags are not set
        assert_eq!(combined & DIFF_HIDDEN_OFF, 0);
        assert_eq!(combined & DIFF_CLOSE_OFF, 0);
        assert_eq!(combined & DIFF_VERTICAL, 0);
    }

    #[test]
    fn test_diff_all_flags_combined() {
        // All flags combined should produce a valid mask
        let all_flags = DIFF_FILLER
            | DIFF_IBLANK
            | DIFF_ICASE
            | DIFF_IWHITE
            | DIFF_IWHITEALL
            | DIFF_IWHITEEOL
            | DIFF_HORIZONTAL
            | DIFF_VERTICAL
            | DIFF_HIDDEN_OFF
            | DIFF_INTERNAL
            | DIFF_CLOSE_OFF
            | DIFF_FOLLOWWRAP
            | DIFF_LINEMATCH;
        // Verify it's positive (no overflow from OR operations)
        assert!(all_flags > 0);
        // Verify expected combined value: all bits 0-12 set = 0x1FFF
        assert_eq!(all_flags, 0x1FFF);
    }

    #[test]
    fn test_diff_flag_count() {
        // There should be exactly 13 defined flags
        let flags = [
            DIFF_FILLER,
            DIFF_IBLANK,
            DIFF_ICASE,
            DIFF_IWHITE,
            DIFF_IWHITEALL,
            DIFF_IWHITEEOL,
            DIFF_HORIZONTAL,
            DIFF_VERTICAL,
            DIFF_HIDDEN_OFF,
            DIFF_INTERNAL,
            DIFF_CLOSE_OFF,
            DIFF_FOLLOWWRAP,
            DIFF_LINEMATCH,
        ];
        assert_eq!(flags.len(), 13);
    }

    #[test]
    fn test_diff_filler_is_lowest_bit() {
        // DIFF_FILLER should be the lowest bit set in any flag
        let all_flags = DIFF_FILLER
            | DIFF_IBLANK
            | DIFF_ICASE
            | DIFF_IWHITE
            | DIFF_IWHITEALL
            | DIFF_IWHITEEOL
            | DIFF_HORIZONTAL
            | DIFF_VERTICAL
            | DIFF_HIDDEN_OFF
            | DIFF_INTERNAL
            | DIFF_CLOSE_OFF
            | DIFF_FOLLOWWRAP
            | DIFF_LINEMATCH;
        // trailing_zeros of all flags combined should be 0 (DIFF_FILLER is bit 0)
        assert_eq!(all_flags.trailing_zeros(), 0);
    }

    #[test]
    fn test_whitespace_flags_group() {
        // Test the ALL_WHITE_DIFF group
        let all_white = DIFF_IWHITE | DIFF_IWHITEALL | DIFF_IWHITEEOL;
        assert_eq!(all_white, 0x038); // bits 3, 4, 5
    }
}
