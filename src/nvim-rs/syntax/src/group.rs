//! Syntax group management.
//!
//! This module handles syntax-specific group operations:
//! - Syntax group ID to highlight group mapping
//! - Group name resolution for syntax items
//! - Integration with the highlight crate for actual group storage
//!
//! Note: Most highlight group management is in the `highlight` crate.
//! This module provides syntax-specific wrappers and helpers.

use std::ffi::c_int;

use crate::types::{SynBlockHandle, SynPatHandle, WinHandle};

// =============================================================================
// FFI declarations for group operations
// =============================================================================

extern "C" {
    // Pattern group accessors
    fn nvim_synpat_get_syn_id(pat: SynPatHandle) -> i16;
    fn nvim_synpat_get_syn_match_id(pat: SynPatHandle) -> i16;
    fn nvim_synpat_get_hl_group(pat: SynPatHandle) -> c_int;

    // Synblock group settings
    fn nvim_synblock_get_topgrp(block: SynBlockHandle) -> c_int;

    // Syntax ID to attribute conversion (via highlight crate)
    fn nvim_syn_id2attr_wrapper(syn_id: c_int) -> c_int;

    // Current state group accessors
    fn nvim_syn_get_current_id() -> c_int;
    fn nvim_syn_get_current_trans_id() -> c_int;

    // syn_get_id - main entry point for getting syntax ID at position
    fn syn_get_id(
        wp: WinHandle,
        lnum: c_int,
        col: c_int,
        trans: c_int,
        spellp: *mut c_int,
        keep_state: c_int,
    ) -> c_int;
}

// =============================================================================
// Pattern group accessors
// =============================================================================

/// Get the syntax group ID from a pattern.
#[must_use]
pub fn synpat_syn_id(pat: SynPatHandle) -> i16 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_syn_id(pat) }
}

/// Get the match group ID from a pattern (for contained matches).
#[must_use]
pub fn synpat_match_id(pat: SynPatHandle) -> i16 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_syn_match_id(pat) }
}

/// Get the highlight group from a pattern.
#[must_use]
pub fn synpat_hl_group(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_hl_group(pat) }
}

// =============================================================================
// Synblock group settings
// =============================================================================

/// Get the topgrp setting for a synblock (for :syntax include).
#[must_use]
pub fn synblock_topgrp(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_topgrp(block) }
}

// =============================================================================
// Syntax ID to attribute conversion
// =============================================================================

/// Convert a syntax ID to a highlight attribute.
///
/// This is the main function for resolving syntax highlighting
/// to actual display attributes. It delegates to the highlight crate.
#[must_use]
pub fn syn_id2attr(syn_id: i32) -> i32 {
    unsafe { nvim_syn_id2attr_wrapper(syn_id) }
}

// =============================================================================
// Current state group accessors
// =============================================================================

/// Get the current highlight group ID.
#[must_use]
pub fn current_id() -> i32 {
    unsafe { nvim_syn_get_current_id() }
}

/// Get the current transparent group ID.
#[must_use]
pub fn current_trans_id() -> i32 {
    unsafe { nvim_syn_get_current_trans_id() }
}

// =============================================================================
// Main syntax ID retrieval
// =============================================================================

/// Get the syntax group ID for a position in the buffer.
///
/// This is the main entry point for getting syntax highlighting
/// at a specific position.
///
/// # Arguments
/// * `wp` - Window handle
/// * `lnum` - Line number (1-based)
/// * `col` - Column number (0-based)
/// * `trans` - If true, return the transparent group ID
/// * `keep_state` - If true, keep the syntax state for further queries
///
/// # Returns
/// The syntax group ID, or 0 if none.
///
/// # Safety
/// The window handle must be valid.
#[must_use]
pub unsafe fn get_syntax_id(
    wp: WinHandle,
    lnum: i32,
    col: i32,
    trans: bool,
    keep_state: bool,
) -> i32 {
    syn_get_id(
        wp,
        lnum,
        col,
        if trans { 1 } else { 0 },
        std::ptr::null_mut(),
        if keep_state { 1 } else { 0 },
    )
}

/// Get the syntax group ID for a position, also returning spell check info.
///
/// # Arguments
/// * `wp` - Window handle
/// * `lnum` - Line number (1-based)
/// * `col` - Column number (0-based)
/// * `trans` - If true, return the transparent group ID
/// * `keep_state` - If true, keep the syntax state for further queries
///
/// # Returns
/// A tuple of (syntax_id, spell_check_needed).
///
/// # Safety
/// The window handle must be valid.
#[must_use]
pub unsafe fn get_syntax_id_spell(
    wp: WinHandle,
    lnum: i32,
    col: i32,
    trans: bool,
    keep_state: bool,
) -> (i32, bool) {
    let mut spellp: c_int = 0;
    let id = syn_get_id(
        wp,
        lnum,
        col,
        if trans { 1 } else { 0 },
        &mut spellp,
        if keep_state { 1 } else { 0 },
    );
    (id, spellp != 0)
}

// =============================================================================
// Group ID utilities
// =============================================================================

/// Check if a syntax group ID is valid (non-zero and not a special ID).
#[must_use]
pub fn is_valid_group_id(id: i32) -> bool {
    id > 0
}

/// Check if a syntax ID represents a keyword match.
/// Keywords use KEYWORD_IDX (-1) as their pattern index.
#[must_use]
pub fn is_keyword_id(pattern_idx: i32) -> bool {
    pattern_idx == crate::types::KEYWORD_IDX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_group_id() {
        assert!(!is_valid_group_id(0));
        assert!(!is_valid_group_id(-1));
        assert!(is_valid_group_id(1));
        assert!(is_valid_group_id(100));
    }

    #[test]
    fn test_is_keyword_id() {
        assert!(is_keyword_id(crate::types::KEYWORD_IDX));
        assert!(is_keyword_id(-1));
        assert!(!is_keyword_id(0));
        assert!(!is_keyword_id(1));
    }

    #[test]
    fn test_null_handle_detection() {
        // Test that null handles are properly detected (doesn't call FFI)
        let null_pat = SynPatHandle(std::ptr::null_mut());
        let null_block = SynBlockHandle(std::ptr::null_mut());

        assert!(null_pat.is_null());
        assert!(null_block.is_null());

        // Non-null handles
        let non_null_pat = SynPatHandle(std::ptr::dangling_mut::<std::ffi::c_void>());
        let non_null_block = SynBlockHandle(std::ptr::dangling_mut::<std::ffi::c_void>());

        assert!(!non_null_pat.is_null());
        assert!(!non_null_block.is_null());
    }
}
