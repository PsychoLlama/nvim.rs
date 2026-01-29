//! Syntax state machine core.
//!
//! This module handles the syntax state machine:
//! - synstate_T (syntax state at line start) accessors
//! - stateitem_T (current state stack item) accessors
//! - Current state management (current_state garray)
//! - State initialization, validation, and cleanup

use std::ffi::c_int;

use crate::types::{
    BufStateHandle, ExtMatchHandle, IdListHandle, StateItemHandle, SynBlockHandle, SynStateHandle,
    KEYWORD_IDX,
};

// =============================================================================
// FFI declarations for state accessors
// =============================================================================

extern "C" {
    // -------------------------------------------------------------------------
    // synstate_T accessors
    // -------------------------------------------------------------------------
    fn nvim_synstate_get_next(state: SynStateHandle) -> SynStateHandle;
    fn nvim_synstate_get_lnum(state: SynStateHandle) -> c_int;
    fn nvim_synstate_get_stacksize(state: SynStateHandle) -> c_int;
    fn nvim_synstate_get_next_flags(state: SynStateHandle) -> c_int;
    fn nvim_synstate_get_tick(state: SynStateHandle) -> c_int;
    fn nvim_synstate_get_change_lnum(state: SynStateHandle) -> c_int;
    fn nvim_synstate_get_next_list(state: SynStateHandle) -> IdListHandle;
    fn nvim_synstate_get_bufstate(state: SynStateHandle, idx: c_int) -> BufStateHandle;
    fn nvim_synstate_set_change_lnum(state: SynStateHandle, lnum: c_int);

    // -------------------------------------------------------------------------
    // bufstate_T accessors
    // -------------------------------------------------------------------------
    fn nvim_bufstate_get_idx(bs: BufStateHandle) -> c_int;
    fn nvim_bufstate_get_flags(bs: BufStateHandle) -> c_int;
    fn nvim_bufstate_get_seqnr(bs: BufStateHandle) -> c_int;
    fn nvim_bufstate_get_cchar(bs: BufStateHandle) -> c_int;
    fn nvim_bufstate_get_extmatch(bs: BufStateHandle) -> ExtMatchHandle;

    // -------------------------------------------------------------------------
    // stateitem_T accessors
    // -------------------------------------------------------------------------
    fn nvim_stateitem_get_idx(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_id(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_trans_id(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_m_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_m_startcol(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_attr(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_flags(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_seqnr(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_cchar(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_end_idx(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_ends(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_cont_list(item: StateItemHandle) -> IdListHandle;
    fn nvim_stateitem_get_next_list(item: StateItemHandle) -> IdListHandle;
    fn nvim_stateitem_get_extmatch(item: StateItemHandle) -> ExtMatchHandle;

    // Stateitem position accessors
    fn nvim_stateitem_get_m_endpos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_m_endpos_col(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_h_startpos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_h_startpos_col(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_h_endpos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_h_endpos_col(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_eoe_pos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_eoe_pos_col(item: StateItemHandle) -> c_int;

    // Stateitem flag checks
    fn nvim_stateitem_has_trans_cont(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_has_match(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_has_cont_list(item: StateItemHandle) -> c_int;

    // Stateitem setters
    fn nvim_stateitem_set_idx(item: StateItemHandle, idx: c_int);
    fn nvim_stateitem_set_id(item: StateItemHandle, id: c_int);
    fn nvim_stateitem_set_trans_id(item: StateItemHandle, trans_id: c_int);
    fn nvim_stateitem_set_attr(item: StateItemHandle, attr: c_int);
    fn nvim_stateitem_set_flags(item: StateItemHandle, flags: c_int);
    fn nvim_stateitem_add_flags(item: StateItemHandle, flags: c_int);
    fn nvim_stateitem_or_flags(item: StateItemHandle, flags: c_int);
    fn nvim_stateitem_set_seqnr(item: StateItemHandle, seqnr: c_int);
    fn nvim_stateitem_set_cchar(item: StateItemHandle, cchar: c_int);
    fn nvim_stateitem_set_end_idx(item: StateItemHandle, end_idx: c_int);
    fn nvim_stateitem_set_ends(item: StateItemHandle, ends: c_int);
    fn nvim_stateitem_set_cont_list(item: StateItemHandle, list: IdListHandle);
    fn nvim_stateitem_set_next_list(item: StateItemHandle, list: IdListHandle);
    fn nvim_stateitem_set_extmatch(item: StateItemHandle, em: ExtMatchHandle);
    fn nvim_stateitem_set_m_lnum(item: StateItemHandle, lnum: c_int);
    fn nvim_stateitem_set_m_startcol(item: StateItemHandle, col: c_int);
    fn nvim_stateitem_set_m_endpos(item: StateItemHandle, lnum: c_int, col: c_int);
    fn nvim_stateitem_set_h_startpos(item: StateItemHandle, lnum: c_int, col: c_int);
    fn nvim_stateitem_set_h_endpos(item: StateItemHandle, lnum: c_int, col: c_int);
    fn nvim_stateitem_set_eoe_pos(item: StateItemHandle, lnum: c_int, col: c_int);

    // -------------------------------------------------------------------------
    // Synblock state accessors
    // -------------------------------------------------------------------------
    fn nvim_synblock_get_sst_first(block: SynBlockHandle) -> SynStateHandle;
    fn nvim_synblock_get_sst_firstfree(block: SynBlockHandle) -> SynStateHandle;
    fn nvim_synblock_has_sst_array(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_sst_len(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_sst_freecount(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_sst_check_lnum(block: SynBlockHandle) -> c_int;

    // -------------------------------------------------------------------------
    // Current state accessors
    // -------------------------------------------------------------------------
    fn nvim_syn_get_current_lnum() -> c_int;
    fn nvim_syn_get_current_col() -> c_int;
    fn nvim_syn_is_current_finished() -> c_int;
    fn nvim_syn_is_current_state_stored() -> c_int;
    fn nvim_syn_get_current_state_len() -> c_int;
    fn nvim_syn_is_current_state_valid() -> c_int;
    fn nvim_syn_get_current_id() -> c_int;
    fn nvim_syn_get_current_trans_id() -> c_int;
    fn nvim_syn_get_current_attr() -> c_int;
    fn nvim_syn_get_current_flags() -> c_int;
    fn nvim_syn_get_current_seqnr() -> c_int;
    fn nvim_syn_get_current_sub_char() -> c_int;
    fn nvim_syn_get_current_next_flags() -> c_int;
    fn nvim_syn_get_keepend_level() -> c_int;
    fn nvim_syn_get_cur_state(idx: c_int) -> StateItemHandle;
    fn nvim_syn_current_state_len() -> c_int;
    fn nvim_syn_current_state_is_empty() -> c_int;
    fn nvim_syn_is_current_state_empty() -> c_int;
    fn nvim_syn_get_stateitem(index: c_int) -> StateItemHandle;
    fn nvim_syn_get_top_stateitem() -> StateItemHandle;
    fn nvim_syn_count_fold_items() -> c_int;

    // Current state setters
    fn nvim_syn_set_state_stored(stored: c_int);
    fn nvim_syn_clear_current_state();
    fn nvim_syn_validate_current_state();
    fn nvim_syn_invalidate_current_state();
    fn nvim_syn_set_keepend_level(level: c_int);
    fn nvim_syn_grow_current_state(size: c_int);
    fn nvim_syn_set_current_state_len(len: c_int);
    fn nvim_syn_set_current_next_list(list: IdListHandle);
    fn nvim_syn_set_current_next_flags(flags: c_int);
    fn nvim_syn_set_current_lnum(lnum: c_int);
    fn nvim_syn_set_current_finished(finished: c_int);
    fn nvim_syn_set_current_id(id: c_int);
    fn nvim_syn_set_current_trans_id(id: c_int);
    fn nvim_syn_set_current_flags(flags: c_int);
    fn nvim_syn_set_current_seqnr(seqnr: c_int);
    fn nvim_syn_pop_current_state();
    fn nvim_syn_push_current_state(idx: c_int);

    // State item field setter helper
    fn nvim_syn_set_cur_state_item(
        idx: c_int,
        si_idx: c_int,
        si_flags: c_int,
        si_seqnr: c_int,
        si_cchar: c_int,
        em: ExtMatchHandle,
    );

    // -------------------------------------------------------------------------
    // Stack management functions
    // -------------------------------------------------------------------------
    fn nvim_syn_stack_free_all(block: SynBlockHandle);
    fn nvim_syn_stack_apply_changes(buf: crate::types::BufHandle);
    fn nvim_buf_get_mod_top(buf: crate::types::BufHandle) -> c_int;
    fn nvim_buf_get_mod_bot(buf: crate::types::BufHandle) -> c_int;
    fn nvim_buf_get_mod_xlines(buf: crate::types::BufHandle) -> c_int;
    fn nvim_synblock_get_linebreaks(block: SynBlockHandle) -> c_int;
    fn nvim_synstate_set_lnum(state: SynStateHandle, lnum: c_int);
    fn nvim_synstate_next_list_eq(a: SynStateHandle, b: SynStateHandle) -> c_int;
}

// =============================================================================
// Position struct for line:col pairs
// =============================================================================

/// A line:column position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub lnum: i32,
    pub col: i32,
}

impl Position {
    /// Create a new position
    #[must_use]
    pub const fn new(lnum: i32, col: i32) -> Self {
        Self { lnum, col }
    }

    /// Check if this is a zero/unset position
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.lnum == 0
    }
}

// =============================================================================
// synstate_T accessors
// =============================================================================

/// Get the next state in the list
#[must_use]
pub fn synstate_next(state: SynStateHandle) -> SynStateHandle {
    if state.is_null() {
        return SynStateHandle::null();
    }
    unsafe { nvim_synstate_get_next(state) }
}

/// Get the line number for a syntax state
#[must_use]
pub fn synstate_lnum(state: SynStateHandle) -> i32 {
    if state.is_null() {
        return 0;
    }
    unsafe { nvim_synstate_get_lnum(state) }
}

/// Get the stack size for a syntax state
#[must_use]
pub fn synstate_stacksize(state: SynStateHandle) -> i32 {
    if state.is_null() {
        return 0;
    }
    unsafe { nvim_synstate_get_stacksize(state) }
}

/// Get the next flags for a syntax state
#[must_use]
pub fn synstate_next_flags(state: SynStateHandle) -> i32 {
    if state.is_null() {
        return 0;
    }
    unsafe { nvim_synstate_get_next_flags(state) }
}

/// Get the tick (when last displayed) for a syntax state
#[must_use]
pub fn synstate_tick(state: SynStateHandle) -> i32 {
    if state.is_null() {
        return 0;
    }
    unsafe { nvim_synstate_get_tick(state) }
}

/// Get the change line number for a syntax state
/// When non-zero, a change may have invalidated the state
#[must_use]
pub fn synstate_change_lnum(state: SynStateHandle) -> i32 {
    if state.is_null() {
        return 0;
    }
    unsafe { nvim_synstate_get_change_lnum(state) }
}

/// Set the change line number for a syntax state
pub fn synstate_set_change_lnum(state: SynStateHandle, lnum: i32) {
    if !state.is_null() {
        unsafe { nvim_synstate_set_change_lnum(state, lnum) }
    }
}

/// Check if a syntax state is valid (not invalidated by changes)
#[must_use]
pub fn synstate_is_valid(state: SynStateHandle) -> bool {
    if state.is_null() {
        return false;
    }
    unsafe { nvim_synstate_get_change_lnum(state) == 0 }
}

/// Get the nextgroup list for a syntax state
#[must_use]
pub fn synstate_next_list(state: SynStateHandle) -> IdListHandle {
    if state.is_null() {
        return IdListHandle::null();
    }
    unsafe { nvim_synstate_get_next_list(state) }
}

/// Get a bufstate item from a synstate at the given index
#[must_use]
pub fn synstate_bufstate(state: SynStateHandle, idx: i32) -> BufStateHandle {
    if state.is_null() || idx < 0 {
        return BufStateHandle::null();
    }
    unsafe { nvim_synstate_get_bufstate(state, idx) }
}

// =============================================================================
// bufstate_T accessors
// =============================================================================

/// Get the pattern index from a bufstate
#[must_use]
pub fn bufstate_idx(bs: BufStateHandle) -> i32 {
    if bs.is_null() {
        return 0;
    }
    unsafe { nvim_bufstate_get_idx(bs) }
}

/// Get the flags from a bufstate
#[must_use]
pub fn bufstate_flags(bs: BufStateHandle) -> i32 {
    if bs.is_null() {
        return 0;
    }
    unsafe { nvim_bufstate_get_flags(bs) }
}

/// Get the sequence number from a bufstate
#[must_use]
pub fn bufstate_seqnr(bs: BufStateHandle) -> i32 {
    if bs.is_null() {
        return 0;
    }
    unsafe { nvim_bufstate_get_seqnr(bs) }
}

/// Get the conceal character from a bufstate
#[must_use]
pub fn bufstate_cchar(bs: BufStateHandle) -> i32 {
    if bs.is_null() {
        return 0;
    }
    unsafe { nvim_bufstate_get_cchar(bs) }
}

/// Get the external match from a bufstate
#[must_use]
pub fn bufstate_extmatch(bs: BufStateHandle) -> ExtMatchHandle {
    if bs.is_null() {
        return ExtMatchHandle::null();
    }
    unsafe { nvim_bufstate_get_extmatch(bs) }
}

// =============================================================================
// stateitem_T accessors
// =============================================================================

/// Get the pattern index from a state item
#[must_use]
pub fn stateitem_idx(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_idx(item) }
}

/// Check if a state item is for a keyword
#[must_use]
pub fn stateitem_is_keyword(item: StateItemHandle) -> bool {
    stateitem_idx(item) == KEYWORD_IDX
}

/// Get the highlight group ID for a state item
#[must_use]
pub fn stateitem_id(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_id(item) }
}

/// Get the transparent ID for a state item
#[must_use]
pub fn stateitem_trans_id(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_trans_id(item) }
}

/// Get the match line number for a state item
#[must_use]
pub fn stateitem_m_lnum(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_m_lnum(item) }
}

/// Get the match start column for a state item
#[must_use]
pub fn stateitem_m_startcol(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_m_startcol(item) }
}

/// Get the attributes for a state item
#[must_use]
pub fn stateitem_attr(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_attr(item) }
}

/// Get the flags for a state item
#[must_use]
pub fn stateitem_flags(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_flags(item) }
}

/// Get the sequence number for a state item
#[must_use]
pub fn stateitem_seqnr(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_seqnr(item) }
}

/// Get the conceal character for a state item
#[must_use]
pub fn stateitem_cchar(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_cchar(item) }
}

/// Get the end pattern index for a state item
#[must_use]
pub fn stateitem_end_idx(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_end_idx(item) }
}

/// Get whether match ends before si_m_endpos
#[must_use]
pub fn stateitem_ends(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_ends(item) }
}

/// Get the contains list for a state item
#[must_use]
pub fn stateitem_cont_list(item: StateItemHandle) -> IdListHandle {
    if item.is_null() {
        return IdListHandle::null();
    }
    unsafe { nvim_stateitem_get_cont_list(item) }
}

/// Get the nextgroup list for a state item
#[must_use]
pub fn stateitem_next_list(item: StateItemHandle) -> IdListHandle {
    if item.is_null() {
        return IdListHandle::null();
    }
    unsafe { nvim_stateitem_get_next_list(item) }
}

/// Get the external match for a state item
#[must_use]
pub fn stateitem_extmatch(item: StateItemHandle) -> ExtMatchHandle {
    if item.is_null() {
        return ExtMatchHandle::null();
    }
    unsafe { nvim_stateitem_get_extmatch(item) }
}

// =============================================================================
// stateitem_T position accessors
// =============================================================================

/// Get the match end position for a state item
#[must_use]
pub fn stateitem_m_endpos(item: StateItemHandle) -> Position {
    if item.is_null() {
        return Position::default();
    }
    Position::new(unsafe { nvim_stateitem_get_m_endpos_lnum(item) }, unsafe {
        nvim_stateitem_get_m_endpos_col(item)
    })
}

/// Get the highlight start position for a state item
#[must_use]
pub fn stateitem_h_startpos(item: StateItemHandle) -> Position {
    if item.is_null() {
        return Position::default();
    }
    Position::new(
        unsafe { nvim_stateitem_get_h_startpos_lnum(item) },
        unsafe { nvim_stateitem_get_h_startpos_col(item) },
    )
}

/// Get the highlight end position for a state item
#[must_use]
pub fn stateitem_h_endpos(item: StateItemHandle) -> Position {
    if item.is_null() {
        return Position::default();
    }
    Position::new(unsafe { nvim_stateitem_get_h_endpos_lnum(item) }, unsafe {
        nvim_stateitem_get_h_endpos_col(item)
    })
}

/// Get the end-of-end position for a state item
#[must_use]
pub fn stateitem_eoe_pos(item: StateItemHandle) -> Position {
    if item.is_null() {
        return Position::default();
    }
    Position::new(unsafe { nvim_stateitem_get_eoe_pos_lnum(item) }, unsafe {
        nvim_stateitem_get_eoe_pos_col(item)
    })
}

// =============================================================================
// stateitem_T flag checks
// =============================================================================

/// Check if a state item has the HL_TRANS_CONT flag
#[must_use]
pub fn stateitem_has_trans_cont(item: StateItemHandle) -> bool {
    if item.is_null() {
        return false;
    }
    unsafe { nvim_stateitem_has_trans_cont(item) != 0 }
}

/// Check if a state item has the HL_MATCH flag
#[must_use]
pub fn stateitem_has_match(item: StateItemHandle) -> bool {
    if item.is_null() {
        return false;
    }
    unsafe { nvim_stateitem_has_match(item) != 0 }
}

/// Check if a state item has a contains list
#[must_use]
pub fn stateitem_has_cont_list(item: StateItemHandle) -> bool {
    if item.is_null() {
        return false;
    }
    unsafe { nvim_stateitem_has_cont_list(item) != 0 }
}

// =============================================================================
// stateitem_T setters
// =============================================================================

/// Set the pattern index for a state item
pub fn stateitem_set_idx(item: StateItemHandle, idx: i32) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_idx(item, idx) }
    }
}

/// Set the highlight group ID for a state item
pub fn stateitem_set_id(item: StateItemHandle, id: i32) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_id(item, id) }
    }
}

/// Set the transparent ID for a state item
pub fn stateitem_set_trans_id(item: StateItemHandle, trans_id: i32) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_trans_id(item, trans_id) }
    }
}

/// Set the attributes for a state item
pub fn stateitem_set_attr(item: StateItemHandle, attr: i32) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_attr(item, attr) }
    }
}

/// Set the flags for a state item
pub fn stateitem_set_flags(item: StateItemHandle, flags: i32) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_flags(item, flags) }
    }
}

/// Add flags to a state item
pub fn stateitem_add_flags(item: StateItemHandle, flags: i32) {
    if !item.is_null() {
        unsafe { nvim_stateitem_add_flags(item, flags) }
    }
}

/// OR flags into a state item
pub fn stateitem_or_flags(item: StateItemHandle, flags: i32) {
    if !item.is_null() {
        unsafe { nvim_stateitem_or_flags(item, flags) }
    }
}

/// Set the sequence number for a state item
pub fn stateitem_set_seqnr(item: StateItemHandle, seqnr: i32) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_seqnr(item, seqnr) }
    }
}

/// Set the conceal character for a state item
pub fn stateitem_set_cchar(item: StateItemHandle, cchar: i32) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_cchar(item, cchar) }
    }
}

/// Set the end pattern index for a state item
pub fn stateitem_set_end_idx(item: StateItemHandle, end_idx: i32) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_end_idx(item, end_idx) }
    }
}

/// Set whether match ends before si_m_endpos
pub fn stateitem_set_ends(item: StateItemHandle, ends: i32) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_ends(item, ends) }
    }
}

/// Set the contains list for a state item
pub fn stateitem_set_cont_list(item: StateItemHandle, list: IdListHandle) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_cont_list(item, list) }
    }
}

/// Set the nextgroup list for a state item
pub fn stateitem_set_next_list(item: StateItemHandle, list: IdListHandle) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_next_list(item, list) }
    }
}

/// Set the external match for a state item
pub fn stateitem_set_extmatch(item: StateItemHandle, em: ExtMatchHandle) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_extmatch(item, em) }
    }
}

/// Set the match line number for a state item
pub fn stateitem_set_m_lnum(item: StateItemHandle, lnum: i32) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_m_lnum(item, lnum) }
    }
}

/// Set the match start column for a state item
pub fn stateitem_set_m_startcol(item: StateItemHandle, col: i32) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_m_startcol(item, col) }
    }
}

/// Set the match end position for a state item
pub fn stateitem_set_m_endpos(item: StateItemHandle, pos: Position) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_m_endpos(item, pos.lnum, pos.col) }
    }
}

/// Set the highlight start position for a state item
pub fn stateitem_set_h_startpos(item: StateItemHandle, pos: Position) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_h_startpos(item, pos.lnum, pos.col) }
    }
}

/// Set the highlight end position for a state item
pub fn stateitem_set_h_endpos(item: StateItemHandle, pos: Position) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_h_endpos(item, pos.lnum, pos.col) }
    }
}

/// Set the end-of-end position for a state item
pub fn stateitem_set_eoe_pos(item: StateItemHandle, pos: Position) {
    if !item.is_null() {
        unsafe { nvim_stateitem_set_eoe_pos(item, pos.lnum, pos.col) }
    }
}

// =============================================================================
// Synblock state accessors
// =============================================================================

/// Get the first used state in the state array
#[must_use]
pub fn synblock_first_state(block: SynBlockHandle) -> SynStateHandle {
    if block.is_null() {
        return SynStateHandle::null();
    }
    unsafe { nvim_synblock_get_sst_first(block) }
}

/// Get the first free state in the state array
#[must_use]
pub fn synblock_first_free_state(block: SynBlockHandle) -> SynStateHandle {
    if block.is_null() {
        return SynStateHandle::null();
    }
    unsafe { nvim_synblock_get_sst_firstfree(block) }
}

/// Check if the synblock has a state array allocated
#[must_use]
pub fn synblock_has_state_array(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_has_sst_array(block) != 0 }
}

/// Get the state array length
#[must_use]
pub fn synblock_state_array_len(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sst_len(block) }
}

/// Get the number of free entries in the state array
#[must_use]
pub fn synblock_free_state_count(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sst_freecount(block) }
}

/// Get the check line number (entries after this need to be checked)
#[must_use]
pub fn synblock_check_lnum(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sst_check_lnum(block) }
}

// =============================================================================
// Current state accessors
// =============================================================================

/// Get the current line number being processed
#[must_use]
pub fn current_lnum() -> i32 {
    unsafe { nvim_syn_get_current_lnum() }
}

/// Get the current column being processed
#[must_use]
pub fn current_col() -> i32 {
    unsafe { nvim_syn_get_current_col() }
}

/// Check if the current line has been finished
#[must_use]
pub fn is_current_finished() -> bool {
    unsafe { nvim_syn_is_current_finished() != 0 }
}

/// Check if the current state has been stored
#[must_use]
pub fn is_current_state_stored() -> bool {
    unsafe { nvim_syn_is_current_state_stored() != 0 }
}

/// Get the current state stack length
#[must_use]
pub fn current_state_len() -> i32 {
    unsafe { nvim_syn_get_current_state_len() }
}

/// Check if the current state is valid
#[must_use]
pub fn is_current_state_valid() -> bool {
    unsafe { nvim_syn_is_current_state_valid() != 0 }
}

/// Check if the current state is empty
#[must_use]
pub fn is_current_state_empty() -> bool {
    unsafe { nvim_syn_is_current_state_empty() != 0 }
}

/// Get the current highlight ID
#[must_use]
pub fn current_id() -> i32 {
    unsafe { nvim_syn_get_current_id() }
}

/// Get the current transparent ID
#[must_use]
pub fn current_trans_id() -> i32 {
    unsafe { nvim_syn_get_current_trans_id() }
}

/// Get the current attribute
#[must_use]
pub fn current_attr() -> i32 {
    unsafe { nvim_syn_get_current_attr() }
}

/// Get the current flags
#[must_use]
pub fn current_flags() -> i32 {
    unsafe { nvim_syn_get_current_flags() }
}

/// Get the current sequence number
#[must_use]
pub fn current_seqnr() -> i32 {
    unsafe { nvim_syn_get_current_seqnr() }
}

/// Get the current substitution character
#[must_use]
pub fn current_sub_char() -> i32 {
    unsafe { nvim_syn_get_current_sub_char() }
}

/// Get the current next flags
#[must_use]
pub fn current_next_flags() -> i32 {
    unsafe { nvim_syn_get_current_next_flags() }
}

/// Get the keepend level (-1 if none)
#[must_use]
pub fn keepend_level() -> i32 {
    unsafe { nvim_syn_get_keepend_level() }
}

/// Get a state item from the current state at the given index
#[must_use]
pub fn get_cur_state(idx: i32) -> Option<StateItemHandle> {
    let item = unsafe { nvim_syn_get_cur_state(idx) };
    if item.is_null() {
        None
    } else {
        Some(item)
    }
}

/// Get a state item from current_state by index
#[must_use]
pub fn get_stateitem(index: i32) -> StateItemHandle {
    unsafe { nvim_syn_get_stateitem(index) }
}

/// Get the top state item from current_state
#[must_use]
pub fn get_top_stateitem() -> StateItemHandle {
    unsafe { nvim_syn_get_top_stateitem() }
}

/// Count items with HL_FOLD flag in the current state
#[must_use]
pub fn count_fold_items() -> i32 {
    unsafe { nvim_syn_count_fold_items() }
}

// =============================================================================
// Current state setters
// =============================================================================

/// Set whether the current state has been stored
pub fn set_state_stored(stored: bool) {
    unsafe { nvim_syn_set_state_stored(if stored { 1 } else { 0 }) }
}

/// Clear the current state
pub fn clear_current_state() {
    unsafe { nvim_syn_clear_current_state() }
}

/// Validate the current state
pub fn validate_current_state() {
    unsafe { nvim_syn_validate_current_state() }
}

/// Invalidate the current state
pub fn invalidate_current_state() {
    unsafe { nvim_syn_invalidate_current_state() }
}

/// Set the keepend level
pub fn set_keepend_level(level: i32) {
    unsafe { nvim_syn_set_keepend_level(level) }
}

/// Grow the current state array
pub fn grow_current_state(size: i32) {
    unsafe { nvim_syn_grow_current_state(size) }
}

/// Set the current state length
pub fn set_current_state_len(len: i32) {
    unsafe { nvim_syn_set_current_state_len(len) }
}

/// Set the current nextgroup list
pub fn set_current_next_list(list: IdListHandle) {
    unsafe { nvim_syn_set_current_next_list(list) }
}

/// Set the current next flags
pub fn set_current_next_flags(flags: i32) {
    unsafe { nvim_syn_set_current_next_flags(flags) }
}

/// Set the current line number
pub fn set_current_lnum(lnum: i32) {
    unsafe { nvim_syn_set_current_lnum(lnum) }
}

/// Set whether the current line is finished
pub fn set_current_finished(finished: bool) {
    unsafe { nvim_syn_set_current_finished(if finished { 1 } else { 0 }) }
}

/// Set the current highlight ID
pub fn set_current_id(id: i32) {
    unsafe { nvim_syn_set_current_id(id) }
}

/// Set the current transparent ID
pub fn set_current_trans_id(id: i32) {
    unsafe { nvim_syn_set_current_trans_id(id) }
}

/// Set the current flags
pub fn set_current_flags(flags: i32) {
    unsafe { nvim_syn_set_current_flags(flags) }
}

/// Set the current sequence number
pub fn set_current_seqnr(seqnr: i32) {
    unsafe { nvim_syn_set_current_seqnr(seqnr) }
}

/// Pop the top item from the current state stack
pub fn pop_current_state() {
    unsafe { nvim_syn_pop_current_state() }
}

/// Push an item onto the current state stack
pub fn push_current_state(idx: i32) {
    unsafe { nvim_syn_push_current_state(idx) }
}

/// Set state item fields at the given index (used by load_current_state)
pub fn set_cur_state_item(
    idx: i32,
    si_idx: i32,
    si_flags: i32,
    si_seqnr: i32,
    si_cchar: i32,
    em: ExtMatchHandle,
) {
    unsafe { nvim_syn_set_cur_state_item(idx, si_idx, si_flags, si_seqnr, si_cchar, em) }
}

// =============================================================================
// Stack management functions
// =============================================================================

/// Free all syntax state entries for a synblock.
///
/// # Safety
/// The caller must ensure the synblock handle is valid.
pub fn stack_free_all(block: SynBlockHandle) {
    if !block.is_null() {
        unsafe { nvim_syn_stack_free_all(block) }
    }
}

/// Apply buffer changes to syntax states.
///
/// This function invalidates or updates cached syntax states
/// when the buffer is modified.
///
/// # Safety
/// The caller must ensure the buffer handle is valid.
pub fn stack_apply_changes(buf: crate::types::BufHandle) {
    if !buf.is_null() {
        unsafe { nvim_syn_stack_apply_changes(buf) }
    }
}

/// Get the line where a buffer change starts.
#[must_use]
pub fn buf_mod_top(buf: crate::types::BufHandle) -> i32 {
    if buf.is_null() {
        return 0;
    }
    unsafe { nvim_buf_get_mod_top(buf) }
}

/// Get the line after a buffer change.
#[must_use]
pub fn buf_mod_bot(buf: crate::types::BufHandle) -> i32 {
    if buf.is_null() {
        return 0;
    }
    unsafe { nvim_buf_get_mod_bot(buf) }
}

/// Get the number of extra lines from a buffer change.
#[must_use]
pub fn buf_mod_xlines(buf: crate::types::BufHandle) -> i32 {
    if buf.is_null() {
        return 0;
    }
    unsafe { nvim_buf_get_mod_xlines(buf) }
}

/// Get the sync linebreaks setting from a synblock.
#[must_use]
pub fn synblock_linebreaks(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_linebreaks(block) }
}

/// Set the line number for a syntax state.
pub fn synstate_set_lnum(state: SynStateHandle, lnum: i32) {
    if !state.is_null() {
        unsafe { nvim_synstate_set_lnum(state, lnum) }
    }
}

/// Check if two synstates have equal next_list pointers.
#[must_use]
pub fn synstate_next_list_eq(a: SynStateHandle, b: SynStateHandle) -> bool {
    if a.is_null() || b.is_null() {
        return a.is_null() && b.is_null();
    }
    unsafe { nvim_synstate_next_list_eq(a, b) != 0 }
}

// =============================================================================
// State machine transition helpers
// =============================================================================

/// Describes the current state of the syntax state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateMachineState {
    /// Initial state, no highlighting active
    Initial,
    /// Inside a region waiting for end pattern
    InRegion,
    /// Processing a match pattern
    InMatch,
    /// State is invalid and needs recomputation
    Invalid,
}

/// Get the current state machine state
#[must_use]
pub fn get_state_machine_state() -> StateMachineState {
    if !is_current_state_valid() {
        return StateMachineState::Invalid;
    }
    if is_current_state_empty() {
        return StateMachineState::Initial;
    }
    let len = current_state_len();
    if len > 0 {
        let top = unsafe { nvim_syn_get_stateitem(len - 1) };
        if !top.is_null() {
            // Check if the top item is a match (HL_MATCH flag)
            if unsafe { nvim_stateitem_has_match(top) != 0 } {
                return StateMachineState::InMatch;
            }
        }
    }
    StateMachineState::InRegion
}

/// State iterator for traversing the state stack from bottom to top
pub struct StateStackIter {
    index: i32,
    len: i32,
}

impl StateStackIter {
    /// Create a new state stack iterator
    #[must_use]
    pub fn new() -> Self {
        Self {
            index: 0,
            len: current_state_len(),
        }
    }
}

impl Default for StateStackIter {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for StateStackIter {
    type Item = StateItemHandle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }
        let item = unsafe { nvim_syn_get_stateitem(self.index) };
        self.index += 1;
        if item.is_null() {
            None
        } else {
            Some(item)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.index) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for StateStackIter {}

/// Synstate linked list iterator
pub struct SynStateIter {
    current: SynStateHandle,
}

impl SynStateIter {
    /// Create a new synstate iterator starting from the given state
    #[must_use]
    pub fn new(start: SynStateHandle) -> Self {
        Self { current: start }
    }

    /// Create an iterator for a synblock's state list
    #[must_use]
    pub fn for_block(block: SynBlockHandle) -> Self {
        Self::new(synblock_first_state(block))
    }
}

impl Iterator for SynStateIter {
    type Item = SynStateHandle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }
        let result = self.current;
        self.current = synstate_next(self.current);
        Some(result)
    }
}

/// Summary of a state item for debugging/inspection
#[derive(Debug, Clone, Copy)]
pub struct StateItemSummary {
    pub idx: i32,
    pub id: i32,
    pub trans_id: i32,
    pub attr: i32,
    pub flags: i32,
    pub is_keyword: bool,
    pub is_match: bool,
    pub has_cont_list: bool,
}

impl StateItemSummary {
    /// Create a summary from a state item handle
    ///
    /// # Safety
    /// This function calls extern FFI functions. The item handle must be valid
    /// or null (returns a zeroed summary for null).
    #[must_use]
    pub fn from_item(item: StateItemHandle) -> Self {
        if item.is_null() {
            return Self {
                idx: 0,
                id: 0,
                trans_id: 0,
                attr: 0,
                flags: 0,
                is_keyword: false,
                is_match: false,
                has_cont_list: false,
            };
        }
        Self {
            idx: stateitem_idx(item),
            id: stateitem_id(item),
            trans_id: stateitem_trans_id(item),
            attr: stateitem_attr(item),
            flags: stateitem_flags(item),
            is_keyword: stateitem_is_keyword(item),
            is_match: stateitem_has_match(item),
            has_cont_list: stateitem_has_cont_list(item),
        }
    }
}

// =============================================================================
// FFI exports for state machine core (Phase Y2)
// =============================================================================

use std::ffi::c_void;

/// Opaque pointer to synblock for FFI
pub type SynBlockPtr = *const c_void;

// State machine state constants
const STATE_INVALID: c_int = 0;
const STATE_INITIAL: c_int = 1;
const STATE_IN_REGION: c_int = 2;
const STATE_IN_MATCH: c_int = 3;

/// Get the current state machine state as an integer constant.
///
/// Returns:
/// - 0: Invalid (needs recomputation)
/// - 1: Initial (no highlighting active)
/// - 2: InRegion (inside a region)
/// - 3: InMatch (processing a match)
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_machine_state() -> c_int {
    match get_state_machine_state() {
        StateMachineState::Invalid => STATE_INVALID,
        StateMachineState::Initial => STATE_INITIAL,
        StateMachineState::InRegion => STATE_IN_REGION,
        StateMachineState::InMatch => STATE_IN_MATCH,
    }
}

/// Check if the current state is valid and non-empty (ready for highlighting)
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_ready() -> c_int {
    if is_current_state_valid() && !is_current_state_empty() {
        1
    } else {
        0
    }
}

/// Check if the current state stack needs to grow for a push operation.
/// Returns 1 if grow is needed, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_needs_grow() -> c_int {
    let len = current_state_len();
    // Check if we're at capacity - this is a heuristic
    if len >= 250 {
        1
    } else {
        0
    }
}

/// Get the nesting depth of the current state (stack length)
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_depth() -> c_int {
    current_state_len()
}

/// Check if we're at the top level (depth 0 or 1)
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_at_top_level() -> c_int {
    if current_state_len() <= 1 { 1 } else { 0 }
}

/// Get the highlight group ID from the top of the state stack.
/// Returns 0 if the stack is empty.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_top_id() -> c_int {
    let len = current_state_len();
    if len <= 0 {
        return 0;
    }
    let top = nvim_syn_get_stateitem(len - 1);
    if top.is_null() {
        return 0;
    }
    nvim_stateitem_get_id(top)
}

/// Get the transparent ID from the top of the state stack.
/// Returns 0 if the stack is empty.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_top_trans_id() -> c_int {
    let len = current_state_len();
    if len <= 0 {
        return 0;
    }
    let top = nvim_syn_get_stateitem(len - 1);
    if top.is_null() {
        return 0;
    }
    nvim_stateitem_get_trans_id(top)
}

/// Get the attribute from the top of the state stack.
/// Returns 0 if the stack is empty.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_top_attr() -> c_int {
    let len = current_state_len();
    if len <= 0 {
        return 0;
    }
    let top = nvim_syn_get_stateitem(len - 1);
    if top.is_null() {
        return 0;
    }
    nvim_stateitem_get_attr(top)
}

/// Get the flags from the top of the state stack.
/// Returns 0 if the stack is empty.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_top_flags() -> c_int {
    let len = current_state_len();
    if len <= 0 {
        return 0;
    }
    let top = nvim_syn_get_stateitem(len - 1);
    if top.is_null() {
        return 0;
    }
    nvim_stateitem_get_flags(top)
}

/// Check if the top state item is a keyword
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_top_is_keyword() -> c_int {
    let len = current_state_len();
    if len <= 0 {
        return 0;
    }
    let top = nvim_syn_get_stateitem(len - 1);
    if top.is_null() {
        return 0;
    }
    if nvim_stateitem_get_idx(top) == KEYWORD_IDX {
        1
    } else {
        0
    }
}

/// Check if the top state item is a match (not a region)
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_top_is_match() -> c_int {
    let len = current_state_len();
    if len <= 0 {
        return 0;
    }
    let top = nvim_syn_get_stateitem(len - 1);
    if top.is_null() {
        return 0;
    }
    nvim_stateitem_has_match(top)
}

/// Find the keepend level in the stack (topmost item with keepend flag).
/// Returns -1 if no keepend item found.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_find_keepend_level() -> c_int {
    keepend_level()
}

/// Count items in the state stack that have the HL_FOLD flag
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_fold_count() -> c_int {
    count_fold_items()
}

/// Transition struct for state operations
#[repr(C)]
pub struct SynStateTransition {
    /// Whether the transition is valid
    pub valid: c_int,
    /// The previous state (before transition)
    pub prev_state: c_int,
    /// The new state (after transition)
    pub new_state: c_int,
    /// The depth change (-1, 0, or 1)
    pub depth_change: c_int,
}

/// Simulate a push transition and return the result.
/// Does NOT actually modify the state - just returns what would happen.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_simulate_push() -> SynStateTransition {
    let prev = match get_state_machine_state() {
        StateMachineState::Invalid => STATE_INVALID,
        StateMachineState::Initial => STATE_INITIAL,
        StateMachineState::InRegion => STATE_IN_REGION,
        StateMachineState::InMatch => STATE_IN_MATCH,
    };

    // A push always goes to InRegion
    SynStateTransition {
        valid: 1,
        prev_state: prev,
        new_state: STATE_IN_REGION,
        depth_change: 1,
    }
}

/// Simulate a pop transition and return the result.
/// Does NOT actually modify the state - just returns what would happen.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_simulate_pop() -> SynStateTransition {
    let len = current_state_len();
    let prev = match get_state_machine_state() {
        StateMachineState::Invalid => STATE_INVALID,
        StateMachineState::Initial => STATE_INITIAL,
        StateMachineState::InRegion => STATE_IN_REGION,
        StateMachineState::InMatch => STATE_IN_MATCH,
    };

    if len <= 0 {
        // Can't pop from empty stack
        return SynStateTransition {
            valid: 0,
            prev_state: prev,
            new_state: prev,
            depth_change: 0,
        };
    }

    // After pop, we either go to InRegion (if depth > 1) or Initial (if depth == 1)
    let new = if len > 1 {
        STATE_IN_REGION
    } else {
        STATE_INITIAL
    };

    SynStateTransition {
        valid: 1,
        prev_state: prev,
        new_state: new,
        depth_change: -1,
    }
}

/// Get the synstate count for a synblock (number of cached states).
#[no_mangle]
pub unsafe extern "C" fn rs_syn_synblock_state_count(block: SynBlockPtr) -> c_int {
    let handle = SynBlockHandle(block as *mut c_void);
    if handle.is_null() {
        return 0;
    }
    // Count states in the linked list
    let mut count = 0;
    let mut current = nvim_synblock_get_sst_first(handle);
    while !current.is_null() {
        count += 1;
        current = nvim_synstate_get_next(current);
    }
    count
}

/// Check if a synblock has any cached state entries.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_synblock_has_states(block: SynBlockPtr) -> c_int {
    let handle = SynBlockHandle(block as *mut c_void);
    if handle.is_null() {
        return 0;
    }
    if !nvim_synblock_get_sst_first(handle).is_null() {
        1
    } else {
        0
    }
}

/// Get the line number of the first cached state in a synblock.
/// Returns 0 if no states cached.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_synblock_first_state_lnum(block: SynBlockPtr) -> c_int {
    let handle = SynBlockHandle(block as *mut c_void);
    if handle.is_null() {
        return 0;
    }
    let first = nvim_synblock_get_sst_first(handle);
    if first.is_null() {
        return 0;
    }
    nvim_synstate_get_lnum(first)
}

/// Get the line number of the last cached state in a synblock.
/// Returns 0 if no states cached.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_synblock_last_state_lnum(block: SynBlockPtr) -> c_int {
    let handle = SynBlockHandle(block as *mut c_void);
    if handle.is_null() {
        return 0;
    }
    let mut last_lnum = 0;
    let mut current = nvim_synblock_get_sst_first(handle);
    while !current.is_null() {
        last_lnum = nvim_synstate_get_lnum(current);
        current = nvim_synstate_get_next(current);
    }
    last_lnum
}

/// Check if a specific line has a cached state in the synblock.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_synblock_has_state_at(block: SynBlockPtr, lnum: c_int) -> c_int {
    let handle = SynBlockHandle(block as *mut c_void);
    if handle.is_null() {
        return 0;
    }
    let mut current = nvim_synblock_get_sst_first(handle);
    while !current.is_null() {
        if nvim_synstate_get_lnum(current) == lnum {
            return 1;
        }
        current = nvim_synstate_get_next(current);
    }
    0
}

/// Find the closest cached state at or before a given line.
/// Returns the line number of the state, or 0 if none found.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_synblock_closest_state_before(
    block: SynBlockPtr,
    lnum: c_int,
) -> c_int {
    let handle = SynBlockHandle(block as *mut c_void);
    if handle.is_null() {
        return 0;
    }
    let mut best_lnum = 0;
    let mut current = nvim_synblock_get_sst_first(handle);
    while !current.is_null() {
        let state_lnum = nvim_synstate_get_lnum(current);
        if state_lnum <= lnum && state_lnum > best_lnum {
            best_lnum = state_lnum;
        }
        current = nvim_synstate_get_next(current);
    }
    best_lnum
}

/// Check if a synstate entry is valid (not invalidated by changes)
#[no_mangle]
pub unsafe extern "C" fn rs_syn_synstate_entry_valid(state: SynStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    if nvim_synstate_get_change_lnum(state) == 0 {
        1
    } else {
        0
    }
}

/// Get the stack size of a cached synstate
#[no_mangle]
pub unsafe extern "C" fn rs_syn_synstate_stack_size(state: SynStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    nvim_synstate_get_stacksize(state)
}

/// Compare two synstates for stack equality (same pattern indices and flags).
/// Returns 1 if equal, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_synstate_stack_equal(
    a: SynStateHandle,
    b: SynStateHandle,
) -> c_int {
    if a.is_null() || b.is_null() {
        return if a.is_null() && b.is_null() { 1 } else { 0 };
    }
    // Compare stack sizes
    let size_a = nvim_synstate_get_stacksize(a);
    let size_b = nvim_synstate_get_stacksize(b);
    if size_a != size_b {
        return 0;
    }
    // Compare nextgroup lists
    if nvim_synstate_next_list_eq(a, b) == 0 {
        return 0;
    }
    // Compare bufstate items
    for i in 0..size_a {
        let bs_a = nvim_synstate_get_bufstate(a, i);
        let bs_b = nvim_synstate_get_bufstate(b, i);
        if bs_a.is_null() != bs_b.is_null() {
            return 0;
        }
        if !bs_a.is_null() {
            if nvim_bufstate_get_idx(bs_a) != nvim_bufstate_get_idx(bs_b) {
                return 0;
            }
            if nvim_bufstate_get_flags(bs_a) != nvim_bufstate_get_flags(bs_b) {
                return 0;
            }
        }
    }
    1
}

/// State machine constant accessors
#[no_mangle]
pub const extern "C" fn rs_syn_state_invalid() -> c_int {
    STATE_INVALID
}

#[no_mangle]
pub const extern "C" fn rs_syn_state_initial() -> c_int {
    STATE_INITIAL
}

#[no_mangle]
pub const extern "C" fn rs_syn_state_in_region() -> c_int {
    STATE_IN_REGION
}

#[no_mangle]
pub const extern "C" fn rs_syn_state_in_match() -> c_int {
    STATE_IN_MATCH
}

/// Get the stateitem at a specific depth from bottom (0-indexed).
/// Returns null handle if index is out of range.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_item_at_depth(depth: c_int) -> StateItemHandle {
    if depth < 0 || depth >= current_state_len() {
        return StateItemHandle(std::ptr::null_mut());
    }
    nvim_syn_get_stateitem(depth)
}

/// Get the stateitem at a specific depth from top (0 = top, 1 = one below top, etc.).
/// Returns null handle if index is out of range.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_item_from_top(offset: c_int) -> StateItemHandle {
    let len = current_state_len();
    let idx = len - 1 - offset;
    if idx < 0 || idx >= len {
        return StateItemHandle(std::ptr::null_mut());
    }
    nvim_syn_get_stateitem(idx)
}

/// Count consecutive items from top that have a specific flag set.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_count_with_flag(flag: c_int) -> c_int {
    let len = current_state_len();
    let mut count = 0;
    for i in (0..len).rev() {
        let item = nvim_syn_get_stateitem(i);
        if item.is_null() {
            break;
        }
        if (nvim_stateitem_get_flags(item) & flag) != 0 {
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Find the topmost item in the stack with a specific flag.
/// Returns the depth index (from bottom), or -1 if not found.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_find_with_flag(flag: c_int) -> c_int {
    let len = current_state_len();
    for i in (0..len).rev() {
        let item = nvim_syn_get_stateitem(i);
        if item.is_null() {
            continue;
        }
        if (nvim_stateitem_get_flags(item) & flag) != 0 {
            return i;
        }
    }
    -1
}

/// Check if any item in the state stack has the given flag
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_any_has_flag(flag: c_int) -> c_int {
    let len = current_state_len();
    for i in 0..len {
        let item = nvim_syn_get_stateitem(i);
        if !item.is_null() && (nvim_stateitem_get_flags(item) & flag) != 0 {
            return 1;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_default() {
        let pos = Position::default();
        assert_eq!(pos.lnum, 0);
        assert_eq!(pos.col, 0);
        assert!(pos.is_zero());
    }

    #[test]
    fn test_position_new() {
        let pos = Position::new(10, 5);
        assert_eq!(pos.lnum, 10);
        assert_eq!(pos.col, 5);
        assert!(!pos.is_zero());
    }

    #[test]
    fn test_position_equality() {
        let pos1 = Position::new(10, 5);
        let pos2 = Position::new(10, 5);
        let pos3 = Position::new(10, 6);

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_handle_null_checks() {
        // These tests don't call extern functions, only check is_null()
        let null_state = SynStateHandle::null();
        let null_item = StateItemHandle(std::ptr::null_mut());
        let null_block = SynBlockHandle(std::ptr::null_mut());
        let null_bufstate = BufStateHandle::null();

        assert!(null_state.is_null());
        assert!(null_item.is_null());
        assert!(null_block.is_null());
        assert!(null_bufstate.is_null());

        // Non-null handles
        let non_null_state = SynStateHandle(std::ptr::dangling_mut::<std::ffi::c_void>());
        let non_null_item = StateItemHandle(std::ptr::dangling_mut::<std::ffi::c_void>());

        assert!(!non_null_state.is_null());
        assert!(!non_null_item.is_null());
    }

    #[test]
    fn test_keyword_idx_constant() {
        // Test that KEYWORD_IDX is the expected value
        assert_eq!(KEYWORD_IDX, -1);
    }

    #[test]
    fn test_state_machine_state_enum() {
        // Test that all state variants are distinct
        assert_ne!(StateMachineState::Initial, StateMachineState::InRegion);
        assert_ne!(StateMachineState::InRegion, StateMachineState::InMatch);
        assert_ne!(StateMachineState::InMatch, StateMachineState::Invalid);
        assert_ne!(StateMachineState::Initial, StateMachineState::Invalid);
    }

    #[test]
    fn test_synstate_iter_null() {
        // Test that SynStateIter handles null start correctly
        let iter = SynStateIter::new(SynStateHandle::null());
        assert!(iter.current.is_null());
    }

    // Note: test_state_stack_iter_default cannot be included here because
    // StateStackIter::new() calls current_state_len() which is an extern FFI function.
    // Such tests are covered by integration tests with the full build.

    // Note: test_state_item_summary_null cannot be included here because
    // StateItemSummary::from_item calls extern FFI functions even for null handles.
    // Such tests are covered by integration tests with the full build.
}
