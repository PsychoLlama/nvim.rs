//! Match navigation support.
//!
//! This module provides helper functions for navigating through completion matches.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_int, c_void};

use crate::match_list::ComplMatch;

// C accessor functions
extern "C" {
    fn nvim_compl_get_first_match() -> ComplMatch;

    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_get_prev(m: ComplMatch) -> ComplMatch;

    fn nvim_compl_is_first_match(m: ComplMatch) -> c_int;
    fn nvim_compl_match_at_original_text(m: ComplMatch) -> c_int;

    fn nvim_get_compl_direction() -> c_int;
}

// Direction constants
const FORWARD: c_int = 1;
#[allow(dead_code)]
const BACKWARD: c_int = -1;

/// Check if a match is at the original text position.
#[inline]
unsafe fn match_at_original_text(m: ComplMatch) -> bool {
    !m.is_null() && nvim_compl_match_at_original_text(m) != 0
}

/// Check if a match is the first match.
#[inline]
unsafe fn is_first_match(m: ComplMatch) -> bool {
    !m.is_null() && nvim_compl_is_first_match(m) != 0
}

// =============================================================================
// Phase 2: Extended Navigation Functions
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
    }
}
