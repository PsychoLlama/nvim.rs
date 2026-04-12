//! Match highlighting utilities for Neovim
//!
//! This crate provides Rust implementations for match highlighting operations,
//! including the `:match` command and `matchadd()`/`matchaddpos()` functions.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_const_for_fn)]

pub mod add;
pub mod core;
pub mod delete;
pub mod display;
pub mod highlight;
pub mod id;
pub mod item;
pub mod lookup;
pub mod position;
pub mod prepare;
pub mod priority;
pub mod range;
pub mod search;
pub mod search_pos;
pub mod viml;

use std::ffi::c_int;

// =============================================================================
// Constants
// =============================================================================

/// `RE_MAGIC` flag for `vim_regcomp` (`regexp.h`: `#define RE_MAGIC 1`).
/// Guarded by `_Static_assert` in `match.c`.
pub const RE_MAGIC: c_int = 1;

/// `UPD_SOME_VALID` redraw type (`drawscreen.h`: `UPD_SOME_VALID = 35`).
/// Guarded by `_Static_assert` in `match.c`.
pub const UPD_SOME_VALID: c_int = 35;

/// `UPD_VALID` redraw type (`drawscreen.h`: `UPD_VALID = 10`).
/// Guarded by `_Static_assert` in `match.c`.
pub const UPD_VALID: c_int = 10;

/// `HLF_L` highlight group (last search string), enum value 8.
/// Guarded by `_Static_assert` in `match.c`.
pub const HLF_L: c_int = 8;

/// `HLF_LC` highlight group (current search match), enum value 9.
/// Guarded by `_Static_assert` in `match.c`.
pub const HLF_LC: c_int = 9;

/// Search highlight priority (hlsearch)
pub const SEARCH_HL_PRIORITY: i32 = 0;

/// Default match priority
pub const DEFAULT_MATCH_PRIORITY: i32 = 10;

/// Minimum valid match ID
pub const MIN_MATCH_ID: i32 = 1;

/// Maximum position matches per `matchaddpos()` call
pub const MAX_POS_MATCHES: usize = 8;

/// Match ID for built-in :match command (slot 1)
pub const MATCH_ID_1: i32 = 1;

/// Match ID for built-in :2match command (slot 2)
pub const MATCH_ID_2: i32 = 2;

/// Match ID for built-in :3match command (slot 3)
pub const MATCH_ID_3: i32 = 3;

// =============================================================================
// FFI Exports
// =============================================================================

/// Get the search highlight priority constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_search_hl_priority() -> c_int {
    SEARCH_HL_PRIORITY
}

/// Get the default match priority constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_default_priority() -> c_int {
    DEFAULT_MATCH_PRIORITY
}

/// Get the minimum valid match ID.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_min_id() -> c_int {
    MIN_MATCH_ID
}

/// Get the maximum position matches.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_max_pos_matches() -> c_int {
    MAX_POS_MATCHES as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(SEARCH_HL_PRIORITY, 0);
        assert_eq!(DEFAULT_MATCH_PRIORITY, 10);
        assert_eq!(MIN_MATCH_ID, 1);
        assert_eq!(MAX_POS_MATCHES, 8);
    }

    #[test]
    fn test_match_ids() {
        assert_eq!(MATCH_ID_1, 1);
        assert_eq!(MATCH_ID_2, 2);
        assert_eq!(MATCH_ID_3, 3);
    }
}
