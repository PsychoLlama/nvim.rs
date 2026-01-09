//! Region handling for syntax highlighting.
//!
//! This module handles:
//! - Region start/skip/end pattern matching
//! - Region nesting logic
//! - Matchgroup handling

use std::ffi::c_int;

use crate::state::Position;
use crate::types::{
    ExtMatchHandle, StateItemHandle, SynPatHandle, SPTYPE_END, SPTYPE_MATCH, SPTYPE_SKIP,
    SPTYPE_START,
};

// =============================================================================
// FFI declarations for region operations
// =============================================================================

extern "C" {
    // Pattern type accessors (from syntax.c)
    fn nvim_synpat_get_type(pat: SynPatHandle) -> c_int;
    fn nvim_synblock_pattern_type(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_type(idx: c_int) -> c_int;

    // Region pattern matching
    fn nvim_syn_update_si_end(sip: StateItemHandle, startcol: c_int, force: c_int);
    fn nvim_syn_find_endpos(
        idx: c_int,
        start_lnum: c_int,
        start_col: c_int,
        m_endpos_lnum: *mut c_int,
        m_endpos_col: *mut c_int,
        hl_endpos_lnum: *mut c_int,
        hl_endpos_col: *mut c_int,
        flags: *mut c_int,
        end_endpos_lnum: *mut c_int,
        end_endpos_col: *mut c_int,
        end_idx: *mut c_int,
        start_ext: ExtMatchHandle,
    );

    // Matchgroup accessors
    fn nvim_synpat_get_syn_match_id(pat: SynPatHandle) -> i16;
    fn nvim_syn_get_pattern_syn_match_id(idx: c_int) -> c_int;

    // State item region accessors
    fn nvim_stateitem_get_end_idx(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_eoe_pos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_eoe_pos_col(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_set_eoe_pos(item: StateItemHandle, lnum: c_int, col: c_int);
    fn nvim_stateitem_set_end_idx(item: StateItemHandle, end_idx: c_int);
}

// =============================================================================
// Pattern type enumeration
// =============================================================================

/// Type of syntax pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    /// Match pattern (single match).
    Match,
    /// Start pattern (beginning of a region).
    Start,
    /// End pattern (end of a region).
    End,
    /// Skip pattern (skip within a region).
    Skip,
    /// Unknown pattern type.
    Unknown(i32),
}

impl PatternType {
    /// Create from a C integer constant.
    #[must_use]
    pub const fn from_c_int(val: i32) -> Self {
        match val {
            SPTYPE_MATCH => Self::Match,
            SPTYPE_START => Self::Start,
            SPTYPE_END => Self::End,
            SPTYPE_SKIP => Self::Skip,
            _ => Self::Unknown(val),
        }
    }

    /// Convert to a C integer constant.
    #[must_use]
    pub const fn to_c_int(self) -> i32 {
        match self {
            Self::Match => SPTYPE_MATCH,
            Self::Start => SPTYPE_START,
            Self::End => SPTYPE_END,
            Self::Skip => SPTYPE_SKIP,
            Self::Unknown(val) => val,
        }
    }

    /// Check if this is a region pattern type (start, end, or skip).
    #[must_use]
    pub const fn is_region_type(self) -> bool {
        matches!(self, Self::Start | Self::End | Self::Skip)
    }

    /// Check if this is the start of a region.
    #[must_use]
    pub const fn is_start(self) -> bool {
        matches!(self, Self::Start)
    }

    /// Check if this is the end of a region.
    #[must_use]
    pub const fn is_end(self) -> bool {
        matches!(self, Self::End)
    }

    /// Check if this is a skip pattern.
    #[must_use]
    pub const fn is_skip(self) -> bool {
        matches!(self, Self::Skip)
    }
}

// =============================================================================
// Pattern type accessors
// =============================================================================

/// Get the pattern type for a synpat_T.
#[must_use]
pub fn synpat_type(pat: SynPatHandle) -> PatternType {
    if pat.is_null() {
        return PatternType::Unknown(0);
    }
    PatternType::from_c_int(unsafe { nvim_synpat_get_type(pat) })
}

/// Get the pattern type for the current synblock pattern by index.
#[must_use]
pub fn synblock_pattern_type_at_idx(idx: i32) -> PatternType {
    PatternType::from_c_int(unsafe { nvim_synblock_pattern_type(idx) })
}

/// Get the pattern type for the current synblock's pattern by index.
#[must_use]
pub fn pattern_type_at_idx(idx: i32) -> PatternType {
    PatternType::from_c_int(unsafe { nvim_syn_get_pattern_type(idx) })
}

// =============================================================================
// Region end position finding
// =============================================================================

/// Result of finding a region end position.
#[derive(Debug, Clone, Copy, Default)]
pub struct RegionEndResult {
    /// End position of the match.
    pub m_endpos: Position,
    /// End position of the highlighting.
    pub hl_endpos: Position,
    /// End position of the end pattern itself.
    pub end_endpos: Position,
    /// Flags from the matching.
    pub flags: i32,
    /// Index of the end pattern that matched (or 0).
    pub end_idx: i32,
}

/// Find the end position of a region.
///
/// # Arguments
/// * `idx` - Index of the start pattern
/// * `start` - Start position to search from
/// * `start_ext` - External match data from the start pattern
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[must_use]
pub unsafe fn find_region_end(
    idx: i32,
    start: Position,
    start_ext: ExtMatchHandle,
) -> RegionEndResult {
    let mut m_endpos_lnum: c_int = 0;
    let mut m_endpos_col: c_int = 0;
    let mut hl_endpos_lnum: c_int = 0;
    let mut hl_endpos_col: c_int = 0;
    let mut flags: c_int = 0;
    let mut end_endpos_lnum: c_int = 0;
    let mut end_endpos_col: c_int = 0;
    let mut end_idx: c_int = 0;

    nvim_syn_find_endpos(
        idx,
        start.lnum,
        start.col,
        &mut m_endpos_lnum,
        &mut m_endpos_col,
        &mut hl_endpos_lnum,
        &mut hl_endpos_col,
        &mut flags,
        &mut end_endpos_lnum,
        &mut end_endpos_col,
        &mut end_idx,
        start_ext,
    );

    RegionEndResult {
        m_endpos: Position {
            lnum: m_endpos_lnum,
            col: m_endpos_col,
        },
        hl_endpos: Position {
            lnum: hl_endpos_lnum,
            col: hl_endpos_col,
        },
        end_endpos: Position {
            lnum: end_endpos_lnum,
            col: end_endpos_col,
        },
        flags,
        end_idx,
    }
}

/// Update the end position for a state item.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
pub unsafe fn update_state_item_end(sip: StateItemHandle, startcol: i32, force: bool) {
    if sip.is_null() {
        return;
    }
    nvim_syn_update_si_end(sip, startcol, if force { 1 } else { 0 });
}

// =============================================================================
// Matchgroup handling
// =============================================================================

/// Get the matchgroup ID for a pattern.
#[must_use]
pub fn synpat_matchgroup_id(pat: SynPatHandle) -> i16 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_syn_match_id(pat) }
}

/// Get the matchgroup ID for a pattern by index.
#[must_use]
pub fn matchgroup_id_at_idx(idx: i32) -> i32 {
    unsafe { nvim_syn_get_pattern_syn_match_id(idx) }
}

// =============================================================================
// State item region accessors
// =============================================================================

/// Get the end pattern index for a state item.
#[must_use]
pub fn stateitem_end_idx(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_end_idx(item) }
}

/// Get the end-of-end position for a state item.
#[must_use]
pub fn stateitem_eoe_pos(item: StateItemHandle) -> Position {
    if item.is_null() {
        return Position::default();
    }
    Position {
        lnum: unsafe { nvim_stateitem_get_eoe_pos_lnum(item) },
        col: unsafe { nvim_stateitem_get_eoe_pos_col(item) },
    }
}

/// Set the end-of-end position for a state item.
///
/// # Safety
/// The item must be a valid non-null pointer.
pub unsafe fn set_stateitem_eoe_pos(item: StateItemHandle, pos: Position) {
    if !item.is_null() {
        nvim_stateitem_set_eoe_pos(item, pos.lnum, pos.col);
    }
}

/// Set the end pattern index for a state item.
///
/// # Safety
/// The item must be a valid non-null pointer.
pub unsafe fn set_stateitem_end_idx(item: StateItemHandle, end_idx: i32) {
    if !item.is_null() {
        nvim_stateitem_set_end_idx(item, end_idx);
    }
}

// =============================================================================
// Region state helpers
// =============================================================================

/// Check if a state item represents a region (has a start pattern type).
#[must_use]
pub fn stateitem_is_region(item: StateItemHandle, si_idx: i32) -> bool {
    if item.is_null() || si_idx < 0 {
        return false;
    }
    pattern_type_at_idx(si_idx).is_start()
}

/// Check if a state item has a pending end pattern match.
#[must_use]
pub fn stateitem_has_end_match(item: StateItemHandle) -> bool {
    if item.is_null() {
        return false;
    }
    stateitem_end_idx(item) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_type() {
        assert_eq!(PatternType::from_c_int(SPTYPE_MATCH), PatternType::Match);
        assert_eq!(PatternType::from_c_int(SPTYPE_START), PatternType::Start);
        assert_eq!(PatternType::from_c_int(SPTYPE_END), PatternType::End);
        assert_eq!(PatternType::from_c_int(SPTYPE_SKIP), PatternType::Skip);
        assert_eq!(PatternType::from_c_int(999), PatternType::Unknown(999));

        assert_eq!(PatternType::Match.to_c_int(), SPTYPE_MATCH);
        assert_eq!(PatternType::Start.to_c_int(), SPTYPE_START);
        assert_eq!(PatternType::End.to_c_int(), SPTYPE_END);
        assert_eq!(PatternType::Skip.to_c_int(), SPTYPE_SKIP);
        assert_eq!(PatternType::Unknown(999).to_c_int(), 999);
    }

    #[test]
    fn test_pattern_type_checks() {
        assert!(!PatternType::Match.is_region_type());
        assert!(PatternType::Start.is_region_type());
        assert!(PatternType::End.is_region_type());
        assert!(PatternType::Skip.is_region_type());

        assert!(PatternType::Start.is_start());
        assert!(!PatternType::End.is_start());

        assert!(PatternType::End.is_end());
        assert!(!PatternType::Start.is_end());

        assert!(PatternType::Skip.is_skip());
        assert!(!PatternType::Start.is_skip());
    }

    #[test]
    fn test_null_handle_checks() {
        // Test null handle creation and checking
        // Note: Cannot call functions that use extern FFI even with null handles
        let null_pat = SynPatHandle(std::ptr::null_mut());
        let null_item = StateItemHandle(std::ptr::null_mut());

        assert!(null_pat.is_null());
        assert!(null_item.is_null());

        // Non-null handle creation for testing
        let fake_ptr = std::ptr::dangling_mut::<std::ffi::c_void>();
        let non_null_pat = SynPatHandle(fake_ptr);
        assert!(!non_null_pat.is_null());
    }

    #[test]
    fn test_region_end_result_default() {
        let result = RegionEndResult::default();
        assert_eq!(result.m_endpos, Position::default());
        assert_eq!(result.hl_endpos, Position::default());
        assert_eq!(result.end_endpos, Position::default());
        assert_eq!(result.flags, 0);
        assert_eq!(result.end_idx, 0);
    }
}
