//! Syntax pattern infrastructure.
//!
//! This module handles syntax patterns including:
//! - Pattern type (SPTYPE_MATCH, SPTYPE_START, SPTYPE_END, SPTYPE_SKIP)
//! - Pattern flags (HL_* constants)
//! - Pattern compilation and storage
//! - Pattern lookup and matching

use std::ffi::{c_char, c_int};

use crate::types::{
    IdListHandle, RegProgHandle, SynBlockHandle, SynPatHandle, HL_CONTAINED, HL_FOLD, HL_KEEPEND,
    HL_TRANSP, SPTYPE_END, SPTYPE_MATCH, SPTYPE_SKIP, SPTYPE_START,
};

// =============================================================================
// FFI declarations for pattern accessors
// =============================================================================

extern "C" {
    // synblock_T pattern accessors
    fn nvim_synblock_get_pattern_count(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_pattern(block: SynBlockHandle, idx: c_int) -> SynPatHandle;
    fn nvim_synblock_get_folditems(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_pattern_is_syncing(block: SynBlockHandle, idx: c_int) -> c_int;
    fn nvim_synblock_count_patterns_for_id(block: SynBlockHandle, id: c_int) -> c_int;

    // synpat_T field accessors
    fn nvim_synpat_get_type(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_syncing(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_syn_match_id(pat: SynPatHandle) -> i16;
    fn nvim_synpat_get_off_flags(pat: SynPatHandle) -> i16;
    fn nvim_synpat_get_flags(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_cchar(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_ic(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_sync_idx(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_pattern(pat: SynPatHandle) -> *const c_char;
    fn nvim_synpat_get_syn_id(pat: SynPatHandle) -> i16;
    fn nvim_synpat_get_syn_inc_tag(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_hl_group(pat: SynPatHandle) -> c_int;

    // Pattern program accessors
    fn nvim_synpat_get_prog(pat: SynPatHandle) -> RegProgHandle;
    fn nvim_synpat_has_prog(pat: SynPatHandle) -> c_int;

    // Pattern list accessors
    fn nvim_synpat_get_cont_list(pat: SynPatHandle) -> IdListHandle;
    fn nvim_synpat_get_next_list(pat: SynPatHandle) -> IdListHandle;
    fn nvim_synpat_get_cont_in_list(pat: SynPatHandle) -> IdListHandle;
    fn nvim_synpat_has_cont_list(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_has_next_list(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_has_cont_in_list(pat: SynPatHandle) -> c_int;

    // Pattern index-based accessors (for current synblock)
    fn nvim_synblock_pattern_type(idx: c_int) -> c_int;
    fn nvim_synblock_pattern_flags(idx: c_int) -> c_int;
    fn nvim_synblock_pattern_syn_id(idx: c_int) -> c_int;
    fn nvim_synblock_pattern_match_id(idx: c_int) -> c_int;
    fn nvim_synblock_pattern_cont_list(idx: c_int) -> IdListHandle;
    fn nvim_synblock_pattern_next_list(idx: c_int) -> IdListHandle;
    fn nvim_synblock_pattern_ic(pat_idx: c_int) -> c_int;

    fn nvim_syn_get_pattern_flags(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_cchar(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_next_list(idx: c_int) -> IdListHandle;
    fn nvim_syn_get_pattern_type(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_syn_match_id(idx: c_int) -> c_int;
}

// =============================================================================
// Pattern type enum
// =============================================================================

/// Represents the type of a syntax pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    /// Match pattern (single pattern match)
    Match,
    /// Start pattern (beginning of a region)
    Start,
    /// End pattern (end of a region)
    End,
    /// Skip pattern (skip within a region)
    Skip,
    /// Unknown type
    Unknown(i32),
}

impl From<i32> for PatternType {
    fn from(value: i32) -> Self {
        match value {
            v if v == SPTYPE_MATCH => PatternType::Match,
            v if v == SPTYPE_START => PatternType::Start,
            v if v == SPTYPE_END => PatternType::End,
            v if v == SPTYPE_SKIP => PatternType::Skip,
            other => PatternType::Unknown(other),
        }
    }
}

impl From<PatternType> for i32 {
    fn from(pt: PatternType) -> Self {
        match pt {
            PatternType::Match => SPTYPE_MATCH,
            PatternType::Start => SPTYPE_START,
            PatternType::End => SPTYPE_END,
            PatternType::Skip => SPTYPE_SKIP,
            PatternType::Unknown(v) => v,
        }
    }
}

// =============================================================================
// Synblock pattern accessors
// =============================================================================

/// Get the number of syntax patterns in a synblock
#[must_use]
pub fn synblock_pattern_count(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_pattern_count(block) }
}

/// Get a pattern at the given index from a synblock
#[must_use]
pub fn synblock_get_pattern(block: SynBlockHandle, idx: i32) -> Option<SynPatHandle> {
    if block.is_null() || idx < 0 {
        return None;
    }
    let pat = unsafe { nvim_synblock_get_pattern(block, idx) };
    if pat.is_null() {
        None
    } else {
        Some(pat)
    }
}

/// Get the number of patterns with HL_FOLD flag
#[must_use]
pub fn synblock_fold_item_count(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_folditems(block) }
}

/// Check if a pattern at the given index is for syncing
#[must_use]
pub fn synblock_pattern_is_syncing(block: SynBlockHandle, idx: i32) -> bool {
    if block.is_null() || idx < 0 {
        return false;
    }
    unsafe { nvim_synblock_pattern_is_syncing(block, idx) != 0 }
}

/// Count patterns with a specific highlight group ID
#[must_use]
pub fn synblock_count_patterns_for_id(block: SynBlockHandle, id: i32) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_count_patterns_for_id(block, id) }
}

// =============================================================================
// Pattern field accessors
// =============================================================================

/// Get the pattern type (SPTYPE_* value)
#[must_use]
pub fn synpat_type(pat: SynPatHandle) -> PatternType {
    if pat.is_null() {
        return PatternType::Unknown(0);
    }
    PatternType::from(unsafe { nvim_synpat_get_type(pat) })
}

/// Get the pattern type as raw integer
#[must_use]
pub fn synpat_type_raw(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_type(pat) }
}

/// Get the flags for a pattern
#[must_use]
pub fn synpat_flags(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_flags(pat) }
}

/// Get the highlight group ID for a pattern
#[must_use]
pub fn synpat_syn_id(pat: SynPatHandle) -> i16 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_syn_id(pat) }
}

/// Get the match ID for a pattern
#[must_use]
pub fn synpat_match_id(pat: SynPatHandle) -> i16 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_syn_match_id(pat) }
}

/// Get the include tag for a pattern
#[must_use]
pub fn synpat_inc_tag(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_syn_inc_tag(pat) }
}

/// Get the offset flags for a pattern
#[must_use]
pub fn synpat_off_flags(pat: SynPatHandle) -> i16 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_off_flags(pat) }
}

/// Get the conceal character for a pattern
#[must_use]
pub fn synpat_cchar(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_cchar(pat) }
}

/// Get the ignore-case flag for a pattern
#[must_use]
pub fn synpat_ic(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_get_ic(pat) != 0 }
}

/// Get the sync index for a pattern (syncing only)
#[must_use]
pub fn synpat_sync_idx(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_sync_idx(pat) }
}

/// Get the pattern string
#[must_use]
pub fn synpat_pattern_str(pat: SynPatHandle) -> *const c_char {
    if pat.is_null() {
        return std::ptr::null();
    }
    unsafe { nvim_synpat_get_pattern(pat) }
}

/// Get the highlight group from a pattern (minus 1)
#[must_use]
pub fn synpat_hl_group(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return -1;
    }
    unsafe { nvim_synpat_get_hl_group(pat) }
}

// =============================================================================
// Pattern flag checks
// =============================================================================

/// Check if a pattern is for syncing
#[must_use]
pub fn synpat_is_syncing(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_get_syncing(pat) != 0 }
}

/// Check if a pattern is transparent
#[must_use]
pub fn synpat_is_transparent(pat: SynPatHandle) -> bool {
    synpat_flags(pat) & HL_TRANSP != 0
}

/// Check if a pattern is contained
#[must_use]
pub fn synpat_is_contained(pat: SynPatHandle) -> bool {
    synpat_flags(pat) & HL_CONTAINED != 0
}

/// Check if a pattern has keepend
#[must_use]
pub fn synpat_has_keepend(pat: SynPatHandle) -> bool {
    synpat_flags(pat) & HL_KEEPEND != 0
}

/// Check if a pattern defines a fold
#[must_use]
pub fn synpat_defines_fold(pat: SynPatHandle) -> bool {
    synpat_flags(pat) & HL_FOLD != 0
}

/// Check if a pattern is a match type
#[must_use]
pub fn synpat_is_match(pat: SynPatHandle) -> bool {
    matches!(synpat_type(pat), PatternType::Match)
}

/// Check if a pattern is a start type
#[must_use]
pub fn synpat_is_start(pat: SynPatHandle) -> bool {
    matches!(synpat_type(pat), PatternType::Start)
}

/// Check if a pattern is an end type
#[must_use]
pub fn synpat_is_end(pat: SynPatHandle) -> bool {
    matches!(synpat_type(pat), PatternType::End)
}

/// Check if a pattern is a skip type
#[must_use]
pub fn synpat_is_skip(pat: SynPatHandle) -> bool {
    matches!(synpat_type(pat), PatternType::Skip)
}

// =============================================================================
// Pattern program accessors
// =============================================================================

/// Check if a pattern has a compiled regex program
#[must_use]
pub fn synpat_has_prog(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_has_prog(pat) != 0 }
}

/// Get the compiled regex program for a pattern
#[must_use]
pub fn synpat_prog(pat: SynPatHandle) -> Option<RegProgHandle> {
    if pat.is_null() {
        return None;
    }
    let prog = unsafe { nvim_synpat_get_prog(pat) };
    if prog.is_null() {
        None
    } else {
        Some(prog)
    }
}

// =============================================================================
// Pattern list accessors
// =============================================================================

/// Check if a pattern has a contains list
#[must_use]
pub fn synpat_has_contains(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_has_cont_list(pat) != 0 }
}

/// Get the contains list for a pattern
#[must_use]
pub fn synpat_contains_list(pat: SynPatHandle) -> Option<IdListHandle> {
    if pat.is_null() {
        return None;
    }
    let list = unsafe { nvim_synpat_get_cont_list(pat) };
    if list.is_null() {
        None
    } else {
        Some(list)
    }
}

/// Check if a pattern has a nextgroup list
#[must_use]
pub fn synpat_has_nextgroup(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_has_next_list(pat) != 0 }
}

/// Get the nextgroup list for a pattern
#[must_use]
pub fn synpat_nextgroup_list(pat: SynPatHandle) -> Option<IdListHandle> {
    if pat.is_null() {
        return None;
    }
    let list = unsafe { nvim_synpat_get_next_list(pat) };
    if list.is_null() {
        None
    } else {
        Some(list)
    }
}

/// Check if a pattern has a containedin list
#[must_use]
pub fn synpat_has_containedin(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_has_cont_in_list(pat) != 0 }
}

/// Get the containedin list for a pattern
#[must_use]
pub fn synpat_containedin_list(pat: SynPatHandle) -> Option<IdListHandle> {
    if pat.is_null() {
        return None;
    }
    let list = unsafe { nvim_synpat_get_cont_in_list(pat) };
    if list.is_null() {
        None
    } else {
        Some(list)
    }
}

// =============================================================================
// Index-based pattern accessors (for current synblock)
// =============================================================================

/// Get pattern type by index from current synblock
#[must_use]
pub fn pattern_type_by_idx(idx: i32) -> PatternType {
    PatternType::from(unsafe { nvim_synblock_pattern_type(idx) })
}

/// Get pattern type as raw integer by index from current synblock
#[must_use]
pub fn pattern_type_raw_by_idx(idx: i32) -> i32 {
    unsafe { nvim_synblock_pattern_type(idx) }
}

/// Get pattern flags by index from current synblock
#[must_use]
pub fn pattern_flags_by_idx(idx: i32) -> i32 {
    unsafe { nvim_synblock_pattern_flags(idx) }
}

/// Get pattern syn_id by index from current synblock
#[must_use]
pub fn pattern_syn_id_by_idx(idx: i32) -> i32 {
    unsafe { nvim_synblock_pattern_syn_id(idx) }
}

/// Get pattern match_id by index from current synblock
#[must_use]
pub fn pattern_match_id_by_idx(idx: i32) -> i32 {
    unsafe { nvim_synblock_pattern_match_id(idx) }
}

/// Get pattern contains list by index from current synblock
#[must_use]
pub fn pattern_cont_list_by_idx(idx: i32) -> IdListHandle {
    unsafe { nvim_synblock_pattern_cont_list(idx) }
}

/// Get pattern nextgroup list by index from current synblock
#[must_use]
pub fn pattern_next_list_by_idx(idx: i32) -> IdListHandle {
    unsafe { nvim_synblock_pattern_next_list(idx) }
}

/// Get pattern ignore-case flag by index from current synblock
#[must_use]
pub fn pattern_ic_by_idx(idx: i32) -> bool {
    unsafe { nvim_synblock_pattern_ic(idx) != 0 }
}

// =============================================================================
// Pattern accessors via nvim_syn_* functions
// =============================================================================

/// Get pattern flags using nvim_syn_get_pattern_flags
#[must_use]
pub fn get_pattern_flags(idx: i32) -> i32 {
    unsafe { nvim_syn_get_pattern_flags(idx) }
}

/// Get pattern cchar using nvim_syn_get_pattern_cchar
#[must_use]
pub fn get_pattern_cchar(idx: i32) -> i32 {
    unsafe { nvim_syn_get_pattern_cchar(idx) }
}

/// Get pattern next_list using nvim_syn_get_pattern_next_list
#[must_use]
pub fn get_pattern_next_list(idx: i32) -> IdListHandle {
    unsafe { nvim_syn_get_pattern_next_list(idx) }
}

/// Get pattern type using nvim_syn_get_pattern_type
#[must_use]
pub fn get_pattern_type(idx: i32) -> PatternType {
    PatternType::from(unsafe { nvim_syn_get_pattern_type(idx) })
}

/// Get pattern type as raw integer using nvim_syn_get_pattern_type
#[must_use]
pub fn get_pattern_type_raw(idx: i32) -> i32 {
    unsafe { nvim_syn_get_pattern_type(idx) }
}

/// Get pattern syn_match_id using nvim_syn_get_pattern_syn_match_id
#[must_use]
pub fn get_pattern_syn_match_id(idx: i32) -> i32 {
    unsafe { nvim_syn_get_pattern_syn_match_id(idx) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_type_conversion() {
        assert_eq!(PatternType::from(SPTYPE_MATCH), PatternType::Match);
        assert_eq!(PatternType::from(SPTYPE_START), PatternType::Start);
        assert_eq!(PatternType::from(SPTYPE_END), PatternType::End);
        assert_eq!(PatternType::from(SPTYPE_SKIP), PatternType::Skip);
        assert_eq!(PatternType::from(99), PatternType::Unknown(99));

        assert_eq!(i32::from(PatternType::Match), SPTYPE_MATCH);
        assert_eq!(i32::from(PatternType::Start), SPTYPE_START);
        assert_eq!(i32::from(PatternType::End), SPTYPE_END);
        assert_eq!(i32::from(PatternType::Skip), SPTYPE_SKIP);
        assert_eq!(i32::from(PatternType::Unknown(99)), 99);
    }

    #[test]
    fn test_pattern_type_matches() {
        // Test the matches! implementations used in is_match, is_start, etc.
        assert!(matches!(PatternType::Match, PatternType::Match));
        assert!(!matches!(PatternType::Match, PatternType::Start));
        assert!(matches!(PatternType::Start, PatternType::Start));
        assert!(matches!(PatternType::End, PatternType::End));
        assert!(matches!(PatternType::Skip, PatternType::Skip));
    }

    #[test]
    fn test_handle_null_detection() {
        // These tests don't call extern functions
        let null_block = SynBlockHandle(std::ptr::null_mut());
        let null_pat = SynPatHandle(std::ptr::null_mut());

        assert!(null_block.is_null());
        assert!(null_pat.is_null());

        // Non-null handles (using dangling_mut for clippy)
        let non_null_block = SynBlockHandle(std::ptr::dangling_mut::<std::ffi::c_void>());
        let non_null_pat = SynPatHandle(std::ptr::dangling_mut::<std::ffi::c_void>());

        assert!(!non_null_block.is_null());
        assert!(!non_null_pat.is_null());
    }
}
