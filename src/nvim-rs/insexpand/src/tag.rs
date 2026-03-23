//! Tag completion support.
//!
//! This module provides helper functions for tag completion (CTRL-X CTRL-]).
//! The core tag lookup operations remain in C.

#![allow(dead_code, unused_imports)]
use std::os::raw::c_int;

// C accessor functions
extern "C" {

    // Compound accessor for tag completion
    fn nvim_get_next_tag_completion_impl();
}

// CTRL-X mode constant
const CTRL_X_TAGS: c_int = 5 + 0x100; // 5 + CTRL_X_WANT_IDENT

// =============================================================================
// Phase 3 (pass 4): get_next_tag_completion
// =============================================================================

/// Get the next set of tag completion matches.
///
/// Saves/restores p_ic, calls find_tags with tag-completion flags,
/// and adds any matches found via ins_compl_add_matches.
///
/// # Safety
/// Requires valid completion state; called from insert mode completion only.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_tag_completion() {
    nvim_get_next_tag_completion_impl();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constant() {
        assert_eq!(CTRL_X_TAGS, 5 + 0x100);
    }
}
