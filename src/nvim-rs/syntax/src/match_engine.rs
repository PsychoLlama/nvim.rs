//! Main matching engine for syntax highlighting.
//!
//! This module handles:
//! - Line parsing and state transitions
//! - Match result aggregation
//! - Current state management for matching

use std::ffi::{c_char, c_int};

use crate::state::Position;
use crate::types::{ExtMatchHandle, IdListHandle, StateItemHandle};

// =============================================================================
// FFI declarations for match engine operations
// =============================================================================

extern "C" {
    // Current state status (non-static ones remain)
    fn nvim_syn_is_current_state_valid() -> c_int;
    fn nvim_syn_is_current_state_empty() -> c_int;

    // Current state management
    fn nvim_syn_get_current_state_len() -> c_int;
    fn nvim_syn_set_current_state_len(len: c_int);
    #[link_name = "rs_validate_current_state"]
    fn nvim_syn_validate_current_state();
    #[link_name = "rs_invalidate_current_state"]
    fn nvim_syn_invalidate_current_state();
    fn nvim_syn_grow_current_state(size: c_int);

    // Next list management

    // Next match accessors
    #[link_name = "rs_push_next_match"]
    fn nvim_syn_push_next_match() -> StateItemHandle;

    // Line operations
    #[link_name = "rs_syn_start_line"]
    fn nvim_syn_start_line();
    #[link_name = "rs_syn_finish_line"]
    fn nvim_syn_finish_line(syncing: c_int) -> c_int;
    #[link_name = "rs_syn_update_ends"]
    fn nvim_syn_update_ends(startofline: c_int);
    #[link_name = "rs_syn_getcurline"]
    fn nvim_syn_getcurline() -> *mut c_char;
    fn nvim_syn_getcurline_at_col() -> c_char;
    #[link_name = "rs_check_state_ends"]
    fn nvim_syn_check_state_ends();
    fn nvim_syn_line_breakcheck();

    // Extmatch management
    fn nvim_syn_ref_extmatch(em: ExtMatchHandle) -> ExtMatchHandle;
    fn nvim_syn_unref_extmatch(em: ExtMatchHandle);

    // Update state item attribute
    fn nvim_syn_update_si_attr(idx: c_int);
}

// =============================================================================
// Current position accessors
// =============================================================================

/// Get the current line number being processed.
#[must_use]
pub fn current_lnum() -> i32 {
    unsafe { crate::statics::CURRENT_LNUM }
}

/// Get the current column being processed.
#[must_use]
pub fn current_col() -> i32 {
    unsafe { crate::statics::CURRENT_COL }
}

/// Get the current position (lnum, col).
#[must_use]
pub fn current_pos() -> Position {
    Position {
        lnum: current_lnum(),
        col: current_col(),
    }
}

/// Set the current line number.
///
/// # Safety
/// This modifies global state.
pub unsafe fn set_current_lnum(lnum: i32) {
    crate::statics::CURRENT_LNUM = lnum;
}

// =============================================================================
// Current state status
// =============================================================================

/// Check if the current line processing is finished.
#[must_use]
pub fn is_current_finished() -> bool {
    unsafe { crate::statics::CURRENT_FINISHED != 0 }
}

/// Check if the current state has been stored.
#[must_use]
pub fn is_current_state_stored() -> bool {
    unsafe { crate::statics::CURRENT_STATE_STORED != 0 }
}

/// Check if the current state is valid.
#[must_use]
pub fn is_current_state_valid() -> bool {
    unsafe { nvim_syn_is_current_state_valid() != 0 }
}

/// Check if the current state stack is empty.
#[must_use]
pub fn is_current_state_empty() -> bool {
    unsafe { nvim_syn_is_current_state_empty() != 0 }
}

/// Set whether the current line is finished.
///
/// # Safety
/// This modifies global state.
pub unsafe fn set_current_finished(finished: bool) {
    crate::statics::CURRENT_FINISHED = if finished { 1 } else { 0 };
}

// =============================================================================
// Current match attributes
// =============================================================================

/// Get the current syntax ID.
#[must_use]
pub fn current_id() -> i32 {
    unsafe { crate::statics::CURRENT_ID }
}

/// Get the current transparent ID.
#[must_use]
pub fn current_trans_id() -> i32 {
    unsafe { crate::statics::CURRENT_TRANS_ID }
}

/// Get the current attribute.
#[must_use]
pub fn current_attr() -> i32 {
    unsafe { crate::statics::CURRENT_ATTR }
}

/// Get the current flags.
#[must_use]
pub fn current_flags() -> i32 {
    unsafe { crate::statics::CURRENT_FLAGS }
}

/// Get the current sequence number.
#[must_use]
pub fn current_seqnr() -> i32 {
    unsafe { crate::statics::CURRENT_SEQNR }
}

/// Get the current substitute character (for conceal).
#[must_use]
pub fn current_sub_char() -> i32 {
    unsafe { crate::statics::CURRENT_SUB_CHAR }
}

/// Get the current next flags.
#[must_use]
pub fn current_next_flags() -> i32 {
    unsafe { crate::statics::CURRENT_NEXT_FLAGS }
}

// =============================================================================
// Current match setters
// =============================================================================

/// Set the current syntax ID.
///
/// # Safety
/// This modifies global state.
pub unsafe fn set_current_id(id: i32) {
    crate::statics::CURRENT_ID = id;
}

/// Set the current transparent ID.
///
/// # Safety
/// This modifies global state.
pub unsafe fn set_current_trans_id(id: i32) {
    crate::statics::CURRENT_TRANS_ID = id;
}

/// Set the current flags.
///
/// # Safety
/// This modifies global state.
pub unsafe fn set_current_flags(flags: i32) {
    crate::statics::CURRENT_FLAGS = flags;
}

/// Set the current sequence number.
///
/// # Safety
/// This modifies global state.
pub unsafe fn set_current_seqnr(seqnr: i32) {
    crate::statics::CURRENT_SEQNR = seqnr;
}

// =============================================================================
// Current state management
// =============================================================================

/// Get the length of the current state stack.
#[must_use]
pub fn current_state_len() -> i32 {
    unsafe { nvim_syn_get_current_state_len() }
}

/// Set the length of the current state stack.
///
/// # Safety
/// This modifies global state.
pub unsafe fn set_current_state_len(len: i32) {
    nvim_syn_set_current_state_len(len);
}

/// Clear the current state.
///
/// # Safety
/// This modifies global state.
pub unsafe fn clear_current_state() {
    crate::state_ops::rs_syn_clear_current_state();
}

/// Validate the current state.
///
/// # Safety
/// This modifies global state.
pub unsafe fn validate_current_state() {
    nvim_syn_validate_current_state();
}

/// Invalidate the current state.
///
/// # Safety
/// This modifies global state.
pub unsafe fn invalidate_current_state() {
    nvim_syn_invalidate_current_state();
}

/// Grow the current state stack to hold at least `size` items.
///
/// # Safety
/// This modifies global state.
pub unsafe fn grow_current_state(size: i32) {
    nvim_syn_grow_current_state(size);
}

/// Pop an item from the current state stack.
///
/// # Safety
/// This modifies global state.
pub unsafe fn pop_current_state() {
    crate::state_ops::rs_syn_pop_current_state();
}

/// Push a state item index onto the current state stack.
///
/// # Safety
/// This modifies global state.
pub unsafe fn push_current_state(idx: i32) {
    crate::state_ops::rs_syn_push_current_state(idx);
}

// =============================================================================
// Next list management
// =============================================================================

/// Get the current next list.
#[must_use]
pub fn current_next_list() -> IdListHandle {
    unsafe { IdListHandle(crate::statics::CURRENT_NEXT_LIST) }
}

/// Check if there is a current next list.
#[must_use]
pub fn has_current_next_list() -> bool {
    unsafe { !crate::statics::CURRENT_NEXT_LIST.is_null() }
}

/// Set the current next list.
///
/// # Safety
/// This modifies global state.
pub unsafe fn set_current_next_list(list: IdListHandle) {
    crate::statics::CURRENT_NEXT_LIST = list.0;
}

/// Set the current next flags.
///
/// # Safety
/// This modifies global state.
pub unsafe fn set_current_next_flags(flags: i32) {
    crate::statics::CURRENT_NEXT_FLAGS = flags;
}

/// Set the current next list pointer.
///
/// # Safety
/// This modifies global state.
pub unsafe fn set_current_next_list_ptr(list: IdListHandle) {
    crate::statics::CURRENT_NEXT_LIST = list.0;
}

/// Get the current next list pointer.
#[must_use]
pub fn current_next_list_ptr() -> IdListHandle {
    unsafe { IdListHandle(crate::statics::CURRENT_NEXT_LIST) }
}

// =============================================================================
// Next match accessors
// =============================================================================

/// Result of querying next match information.
#[derive(Debug, Clone, Copy, Default)]
pub struct NextMatchInfo {
    /// Pattern index (-1 if no match).
    pub idx: i32,
    /// Column where match starts.
    pub col: i32,
    /// Match flags.
    pub flags: i32,
    /// End pattern index.
    pub end_idx: i32,
    /// Start of highlighting.
    pub h_startpos: Position,
    /// End of match.
    pub m_endpos: Position,
    /// End of highlighting.
    pub h_endpos: Position,
    /// End-of-start position.
    pub eos_pos: Position,
    /// End-of-end position.
    pub eoe_pos: Position,
}

/// Get the next match pattern index.
#[must_use]
pub fn next_match_idx() -> i32 {
    unsafe { crate::statics::NEXT_MATCH_IDX }
}

/// Get the next match column.
#[must_use]
pub fn next_match_col() -> i32 {
    unsafe { crate::statics::NEXT_MATCH_COL }
}

/// Check if there is a pending next match.
#[must_use]
pub fn has_next_match() -> bool {
    unsafe { crate::statics::NEXT_MATCH_IDX >= 0 }
}

/// Get the next match index value.
#[must_use]
pub fn next_match_idx_value() -> i32 {
    unsafe { crate::statics::NEXT_MATCH_IDX }
}

/// Set the next match index.
///
/// # Safety
/// This modifies global state.
pub unsafe fn set_next_match_idx(idx: i32) {
    crate::statics::NEXT_MATCH_IDX = idx;
}

/// Set the next match column.
///
/// # Safety
/// This modifies global state.
pub unsafe fn set_next_match_col(col: i32) {
    crate::statics::NEXT_MATCH_COL = col;
}

/// Get the next match flags.
#[must_use]
pub fn next_match_flags() -> i32 {
    unsafe { crate::statics::NEXT_MATCH_FLAGS }
}

/// Get the next match end index.
#[must_use]
pub fn next_match_end_idx() -> i32 {
    unsafe { crate::statics::NEXT_MATCH_END_IDX }
}

/// Get the next match extmatch handle.
#[must_use]
pub fn next_match_extmatch() -> ExtMatchHandle {
    unsafe { ExtMatchHandle(crate::statics::NEXT_MATCH_EXTMATCH) }
}

/// All 5 next_match position fields in one bulk call.
pub struct NextMatchPositions {
    pub h_startpos: Position,
    pub m_endpos: Position,
    pub h_endpos: Position,
    pub eos_pos: Position,
    pub eoe_pos: Position,
}

/// Fetch all next_match position fields from Rust statics.
///
/// # Safety
/// Accesses global syntax state; must be called from main thread.
#[must_use]
pub unsafe fn next_match_positions() -> NextMatchPositions {
    use crate::statics::{
        NEXT_MATCH_EOE_POS, NEXT_MATCH_EOS_POS, NEXT_MATCH_H_ENDPOS, NEXT_MATCH_H_STARTPOS,
        NEXT_MATCH_M_ENDPOS,
    };
    NextMatchPositions {
        h_startpos: Position {
            lnum: NEXT_MATCH_H_STARTPOS.lnum,
            col: NEXT_MATCH_H_STARTPOS.col,
        },
        m_endpos: Position {
            lnum: NEXT_MATCH_M_ENDPOS.lnum,
            col: NEXT_MATCH_M_ENDPOS.col,
        },
        h_endpos: Position {
            lnum: NEXT_MATCH_H_ENDPOS.lnum,
            col: NEXT_MATCH_H_ENDPOS.col,
        },
        eos_pos: Position {
            lnum: NEXT_MATCH_EOS_POS.lnum,
            col: NEXT_MATCH_EOS_POS.col,
        },
        eoe_pos: Position {
            lnum: NEXT_MATCH_EOE_POS.lnum,
            col: NEXT_MATCH_EOE_POS.col,
        },
    }
}

/// Get the next match highlight start position.
#[must_use]
pub fn next_match_h_startpos() -> Position {
    unsafe { next_match_positions().h_startpos }
}

/// Get the next match end position.
#[must_use]
pub fn next_match_m_endpos() -> Position {
    unsafe { next_match_positions().m_endpos }
}

/// Get the next match highlight end position.
#[must_use]
pub fn next_match_h_endpos() -> Position {
    unsafe { next_match_positions().h_endpos }
}

/// Get the next match end-of-start position.
#[must_use]
pub fn next_match_eos_pos() -> Position {
    unsafe { next_match_positions().eos_pos }
}

/// Get the next match end-of-end position.
#[must_use]
pub fn next_match_eoe_pos() -> Position {
    unsafe { next_match_positions().eoe_pos }
}

/// Get all next match information at once.
#[must_use]
pub fn next_match_info() -> NextMatchInfo {
    NextMatchInfo {
        idx: next_match_idx(),
        col: next_match_col(),
        flags: next_match_flags(),
        end_idx: next_match_end_idx(),
        h_startpos: next_match_h_startpos(),
        m_endpos: next_match_m_endpos(),
        h_endpos: next_match_h_endpos(),
        eos_pos: next_match_eos_pos(),
        eoe_pos: next_match_eoe_pos(),
    }
}

/// Push the next match onto the state stack.
///
/// # Safety
/// This modifies global state.
#[must_use]
pub unsafe fn push_next_match() -> StateItemHandle {
    nvim_syn_push_next_match()
}

// =============================================================================
// Line operations
// =============================================================================

/// Start processing a new line.
///
/// # Safety
/// This modifies global state.
pub unsafe fn start_line() {
    nvim_syn_start_line();
}

/// Finish processing the current line.
///
/// # Safety
/// This modifies global state.
///
/// # Returns
/// The result of finishing the line.
pub unsafe fn finish_line(syncing: bool) -> i32 {
    nvim_syn_finish_line(if syncing { 1 } else { 0 })
}

/// Update end positions at the start of a line.
///
/// # Safety
/// This modifies global state.
pub unsafe fn update_ends(startofline: bool) {
    nvim_syn_update_ends(if startofline { 1 } else { 0 });
}

/// Get the current line being processed.
///
/// # Safety
/// The returned pointer is only valid until the next pattern match.
#[must_use]
pub unsafe fn getcurline() -> *mut c_char {
    nvim_syn_getcurline()
}

/// Get the character at the current column.
#[must_use]
pub fn getcurline_at_col() -> i8 {
    unsafe { nvim_syn_getcurline_at_col() }
}

/// Check state ends for the current position.
///
/// # Safety
/// This modifies global state.
pub unsafe fn check_state_ends() {
    nvim_syn_check_state_ends();
}

/// Perform a line breakcheck.
///
/// # Safety
/// This may modify global state.
pub unsafe fn line_breakcheck() {
    nvim_syn_line_breakcheck();
}

// =============================================================================
// Line ID tracking
// =============================================================================

/// Get the current line ID.
#[must_use]
pub fn current_line_id() -> i32 {
    unsafe { crate::statics::CURRENT_LINE_ID }
}

/// Increment the current line ID.
///
/// # Safety
/// This modifies global state.
pub unsafe fn incr_current_line_id() {
    crate::statics::CURRENT_LINE_ID += 1;
}

// =============================================================================
// State item spans check
// =============================================================================

/// Check if a state item spans to a given line.
#[must_use]
pub fn state_item_spans_line(idx: i32, lnum: i32) -> bool {
    unsafe { crate::state_ops::rs_syn_state_item_spans_line(idx, lnum) != 0 }
}

// =============================================================================
// Extmatch management
// =============================================================================

/// Increment the reference count on an extmatch.
///
/// # Safety
/// The extmatch must be valid or null.
#[must_use]
pub unsafe fn ref_extmatch(em: ExtMatchHandle) -> ExtMatchHandle {
    nvim_syn_ref_extmatch(em)
}

/// Decrement the reference count on an extmatch.
///
/// # Safety
/// The extmatch must be valid or null.
pub unsafe fn unref_extmatch(em: ExtMatchHandle) {
    nvim_syn_unref_extmatch(em);
}

/// Check if two extmatches are equal.
#[must_use]
pub fn extmatch_equal(a: ExtMatchHandle, b: ExtMatchHandle) -> bool {
    unsafe { crate::state_ops::rs_syn_extmatch_equal(a, b) != 0 }
}

/// Check if two extmatch strings at a given index are equal.
#[must_use]
pub fn extmatch_strings_equal(
    a: ExtMatchHandle,
    b: ExtMatchHandle,
    subidx: i32,
    pat_idx: i32,
) -> bool {
    unsafe { crate::state_ops::rs_syn_extmatch_strings_equal(a, b, subidx, pat_idx) != 0 }
}

// =============================================================================
// Update state item attribute
// =============================================================================

/// Update the attribute for a state item.
///
/// # Safety
/// This modifies global state.
pub unsafe fn update_si_attr(idx: i32) {
    nvim_syn_update_si_attr(idx);
}

/// Call update_si_attr (wrapper for external use).
///
/// # Safety
/// This modifies global state.
pub unsafe fn call_update_si_attr(idx: i32) {
    nvim_syn_update_si_attr(idx);
}

// =============================================================================
// Match result aggregation
// =============================================================================

/// Current match result containing all relevant attributes.
#[derive(Debug, Clone, Copy, Default)]
pub struct MatchResult {
    /// Syntax ID of the match.
    pub id: i32,
    /// Transparent ID (for transparent groups).
    pub trans_id: i32,
    /// Highlight attribute.
    pub attr: i32,
    /// Match flags.
    pub flags: i32,
    /// Sequence number.
    pub seqnr: i32,
    /// Substitute character (for conceal).
    pub sub_char: i32,
}

/// Get the current match result.
#[must_use]
pub fn current_match_result() -> MatchResult {
    MatchResult {
        id: current_id(),
        trans_id: current_trans_id(),
        attr: current_attr(),
        flags: current_flags(),
        seqnr: current_seqnr(),
        sub_char: current_sub_char(),
    }
}

/// Set the current match result.
///
/// # Safety
/// This modifies global state.
pub unsafe fn set_current_match_result(result: &MatchResult) {
    set_current_id(result.id);
    set_current_trans_id(result.trans_id);
    set_current_flags(result.flags);
    set_current_seqnr(result.seqnr);
}

// =============================================================================
// Match status helpers
// =============================================================================

/// Status of the matching engine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchStatus {
    /// No match at current position
    NoMatch,
    /// Match found and in progress
    Matching,
    /// Inside a region but no specific match at this column
    InRegion,
    /// Finished processing current line
    Finished,
    /// State is invalid/needs sync
    Invalid,
}

/// Get the current match status
#[must_use]
pub fn match_status() -> MatchStatus {
    if !is_current_state_valid() {
        return MatchStatus::Invalid;
    }
    if is_current_finished() {
        return MatchStatus::Finished;
    }
    if current_id() > 0 {
        return MatchStatus::Matching;
    }
    if !is_current_state_empty() {
        return MatchStatus::InRegion;
    }
    MatchStatus::NoMatch
}

/// Check if we're actively matching something at the current position
#[must_use]
pub fn is_actively_matching() -> bool {
    is_current_state_valid() && current_id() > 0
}

/// Check if we're inside any syntax region (regardless of highlight)
#[must_use]
pub fn is_in_syntax_context() -> bool {
    is_current_state_valid() && !is_current_state_empty()
}

/// Get the depth of the current syntax nesting
#[must_use]
pub fn syntax_nesting_depth() -> i32 {
    current_state_len()
}

/// Summary of the current matching state for debugging
#[derive(Debug, Clone, Copy, Default)]
pub struct MatchStateSummary {
    pub lnum: i32,
    pub col: i32,
    pub id: i32,
    pub trans_id: i32,
    pub attr: i32,
    pub flags: i32,
    pub nesting_depth: i32,
    pub is_finished: bool,
    pub is_stored: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_match_info_default() {
        let info = NextMatchInfo::default();
        assert_eq!(info.idx, 0);
        assert_eq!(info.col, 0);
        assert_eq!(info.flags, 0);
        assert_eq!(info.end_idx, 0);
        assert_eq!(info.h_startpos, Position::default());
        assert_eq!(info.m_endpos, Position::default());
        assert_eq!(info.h_endpos, Position::default());
        assert_eq!(info.eos_pos, Position::default());
        assert_eq!(info.eoe_pos, Position::default());
    }

    #[test]
    fn test_match_result_default() {
        let result = MatchResult::default();
        assert_eq!(result.id, 0);
        assert_eq!(result.trans_id, 0);
        assert_eq!(result.attr, 0);
        assert_eq!(result.flags, 0);
        assert_eq!(result.seqnr, 0);
        assert_eq!(result.sub_char, 0);
    }

    #[test]
    fn test_null_handles() {
        let null_ext = ExtMatchHandle(std::ptr::null_mut());
        assert!(null_ext.is_null());

        let null_list = IdListHandle::null();
        assert!(null_list.is_null());
    }

    #[test]
    fn test_position_creation() {
        let pos = Position { lnum: 10, col: 5 };
        assert_eq!(pos.lnum, 10);
        assert_eq!(pos.col, 5);
    }

    #[test]
    fn test_match_status_enum_values() {
        // Test that all match status variants are distinct
        assert_ne!(MatchStatus::NoMatch, MatchStatus::Matching);
        assert_ne!(MatchStatus::Matching, MatchStatus::InRegion);
        assert_ne!(MatchStatus::InRegion, MatchStatus::Finished);
        assert_ne!(MatchStatus::Finished, MatchStatus::Invalid);
    }

    #[test]
    fn test_match_state_summary_default() {
        let summary = MatchStateSummary::default();
        assert_eq!(summary.lnum, 0);
        assert_eq!(summary.col, 0);
        assert_eq!(summary.id, 0);
        assert_eq!(summary.trans_id, 0);
        assert_eq!(summary.attr, 0);
        assert_eq!(summary.flags, 0);
        assert_eq!(summary.nesting_depth, 0);
        assert!(!summary.is_finished);
        assert!(!summary.is_stored);
    }
}
