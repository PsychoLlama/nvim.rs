//! Tag completion support.
//!
//! This module provides helper functions for tag completion (CTRL-X CTRL-]).
//! The core tag lookup operations remain in C.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    // nvim_get_next_tag_completion_impl: deleted (Phase 14), inlined below

    // Helpers for inlined nvim_get_next_tag_completion_impl
    fn ignorecase(pat: *mut c_char) -> c_int;
    fn find_tags(
        pat: *mut c_char,
        num_matches: *mut c_int,
        matchesp: *mut *mut *mut c_char,
        flags: c_int,
        mincount: c_int,
        buf_ffname: *mut c_char,
    ) -> c_int;
    fn rs_ins_compl_add_matches(num_matches: c_int, matches: *mut *mut c_char, icase: c_int);
    fn rs_ctrl_x_mode_not_default() -> c_int;
    fn nvim_get_curbuf_ffname() -> *const c_char;
    #[link_name = "p_ic"]
    static mut p_ic_tag: c_int;
    #[link_name = "g_tag_at_cursor"]
    static mut g_tag_at_cursor: bool;
}

// CTRL-X mode constant
const CTRL_X_TAGS: c_int = 5 + 0x100; // 5 + CTRL_X_WANT_IDENT

// find_tags flags (from tag.h)
const TAG_NAMES: c_int = 2;
const TAG_REGEXP: c_int = 4;
const TAG_NOIC: c_int = 8;
const TAG_VERBOSE: c_int = 32;
const TAG_INS_COMP: c_int = 64;
const TAG_MANY: c_int = 300;

// Return code
const OK: c_int = 1;

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
    // Save and override p_ic based on pattern case
    let save_p_ic = p_ic_tag;
    p_ic_tag = ignorecase(crate::vars::compl_pattern.data);

    // Find up to TAG_MANY matches. Avoids enormous result sets when pattern is empty.
    g_tag_at_cursor = true;
    let mut matches: *mut *mut c_char = core::ptr::null_mut();
    let mut num_matches: c_int = 0;
    let flags = TAG_REGEXP
        | TAG_NAMES
        | TAG_NOIC
        | TAG_INS_COMP
        | if rs_ctrl_x_mode_not_default() != 0 {
            TAG_VERBOSE
        } else {
            0
        };
    if find_tags(
        crate::vars::compl_pattern.data,
        &raw mut num_matches,
        &raw mut matches,
        flags,
        TAG_MANY,
        nvim_get_curbuf_ffname().cast_mut(),
    ) == OK
        && num_matches > 0
    {
        rs_ins_compl_add_matches(num_matches, matches, p_ic_tag);
    }
    g_tag_at_cursor = false;
    p_ic_tag = save_p_ic;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constant() {
        assert_eq!(CTRL_X_TAGS, 5 + 0x100);
    }
}
