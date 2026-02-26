//! Match state and current state helper operations.
//!
//! This module migrates from C:
//! - nvim_syn_set_next_match_state  (bulk setter for next_match_* statics)
//! - nvim_syn_pop_current_state     (pop top stateitem, unref extmatch)
//! - nvim_syn_push_current_state    (push new stateitem, set si_idx)
//! - nvim_syn_set_cur_state_item    (set stateitem fields at index)
//! - nvim_syn_count_fold_items      (count HL_FOLD items)
//! - nvim_syn_state_item_spans_line (check if item spans past given line)
//! - nvim_syn_clear_current_state   (GA_DEEP_CLEAR with extmatch unref)
//! - nvim_stateitem_prev_if_trans_cont (walk back through HL_TRANS_CONT items)

use std::ffi::c_int;

use crate::types::{ExtMatchHandle, IdListHandle, StateItemHandle, HL_FOLD, HL_TRANS_CONT};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Current state length/grow
    fn nvim_syn_get_current_state_len() -> c_int;
    fn nvim_syn_set_current_state_len(len: c_int);
    fn nvim_syn_grow_current_state(size: c_int);

    // Stateitem access
    fn nvim_syn_get_stateitem(index: c_int) -> StateItemHandle;

    // Stateitem field accessors
    fn nvim_stateitem_get_flags(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_extmatch(item: StateItemHandle) -> ExtMatchHandle;
    fn nvim_stateitem_get_m_endpos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_m_endpos_col(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_h_startpos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_h_startpos_col(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_h_endpos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_h_endpos_col(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_eoe_pos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_end_idx(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_set_idx(item: StateItemHandle, idx: c_int);
    fn nvim_stateitem_set_flags(item: StateItemHandle, flags: c_int);
    fn nvim_stateitem_set_seqnr(item: StateItemHandle, seqnr: c_int);
    fn nvim_stateitem_set_cchar(item: StateItemHandle, cchar: c_int);
    fn nvim_stateitem_set_extmatch(item: StateItemHandle, em: ExtMatchHandle);
    fn nvim_stateitem_set_ends(item: StateItemHandle, ends: c_int);
    fn nvim_stateitem_set_m_lnum(item: StateItemHandle, lnum: c_int);
    fn nvim_stateitem_set_next_list(item: StateItemHandle, list: IdListHandle);

    // Extmatch reference management
    fn nvim_syn_unref_extmatch(em: ExtMatchHandle);
    fn nvim_syn_ref_extmatch(em: ExtMatchHandle) -> ExtMatchHandle;

    // next_match_idx
    fn nvim_syn_set_next_match_idx_val(idx: c_int);

    // keepend_level
    fn nvim_syn_get_keepend_level() -> c_int;
    fn nvim_syn_set_keepend_level(level: c_int);

    // Stateitem append helper (GA_APPEND_VIA_PTR + CLEAR_POINTER)
    fn nvim_syn_append_new_stateitem() -> StateItemHandle;

    // Pattern next_list lookup
    fn nvim_syn_get_pattern_next_list(idx: c_int) -> IdListHandle;

    // Individual next_match setters
    fn nvim_syn_set_next_match_col_val(col: c_int);
    fn nvim_syn_set_next_match_m_endpos(lnum: c_int, col: c_int);
    fn nvim_syn_set_next_match_h_endpos(lnum: c_int, col: c_int);
    fn nvim_syn_set_next_match_h_startpos(lnum: c_int, col: c_int);
    fn nvim_syn_set_next_match_flags(flags: c_int);
    fn nvim_syn_set_next_match_eos_pos(lnum: c_int, col: c_int);
    fn nvim_syn_set_next_match_eoe_pos(lnum: c_int, col: c_int);
    fn nvim_syn_set_next_match_end_idx(idx: c_int);
    fn nvim_syn_set_next_match_extmatch_raw(em: ExtMatchHandle);
    fn nvim_syn_get_next_match_extmatch() -> ExtMatchHandle;
}

// =============================================================================
// Phase 2 Rust implementations
// =============================================================================

/// Bulk setter for all next_match_* static variables.
///
/// Replaces C `nvim_syn_set_next_match_state`.
///
/// # Safety
/// Accesses C global syntax state; must be called from main thread.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rs_syn_set_next_match_state(
    idx: c_int,
    col: c_int,
    m_endpos_lnum: c_int,
    m_endpos_col: c_int,
    h_endpos_lnum: c_int,
    h_endpos_col: c_int,
    h_startpos_lnum: c_int,
    h_startpos_col: c_int,
    flags: c_int,
    eos_pos_lnum: c_int,
    eos_pos_col: c_int,
    eoe_pos_lnum: c_int,
    eoe_pos_col: c_int,
    end_idx: c_int,
    extmatch: ExtMatchHandle,
) {
    nvim_syn_set_next_match_idx_val(idx);
    nvim_syn_set_next_match_col_val(col);
    nvim_syn_set_next_match_m_endpos(m_endpos_lnum, m_endpos_col);
    nvim_syn_set_next_match_h_endpos(h_endpos_lnum, h_endpos_col);
    nvim_syn_set_next_match_h_startpos(h_startpos_lnum, h_startpos_col);
    nvim_syn_set_next_match_flags(flags);
    nvim_syn_set_next_match_eos_pos(eos_pos_lnum, eos_pos_col);
    nvim_syn_set_next_match_eoe_pos(eoe_pos_lnum, eoe_pos_col);
    nvim_syn_set_next_match_end_idx(end_idx);
    // Unref old extmatch, then set new
    let old_em = nvim_syn_get_next_match_extmatch();
    nvim_syn_unref_extmatch(old_em);
    nvim_syn_set_next_match_extmatch_raw(extmatch);
}

/// Pop the top item from current_state.
/// Unrefs si_extmatch, decrements ga_len, resets next_match_idx,
/// and adjusts keepend_level if needed.
///
/// Replaces C `nvim_syn_pop_current_state`.
///
/// # Safety
/// Accesses C global syntax state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_pop_current_state() {
    let len = nvim_syn_get_current_state_len();
    if len <= 0 {
        return;
    }
    let top = nvim_syn_get_stateitem(len - 1);
    if !top.is_null() {
        let em = nvim_stateitem_get_extmatch(top);
        nvim_syn_unref_extmatch(em);
    }
    nvim_syn_set_current_state_len(len - 1);
    // After end of a pattern, try matching a keyword or pattern next time
    nvim_syn_set_next_match_idx_val(-1);
    // If first state with "keepend" is popped, reset keepend_level
    let keepend = nvim_syn_get_keepend_level();
    if keepend >= len - 1 {
        nvim_syn_set_keepend_level(-1);
    }
}

/// Push an item onto current_state.
/// Appends, zeros, and sets si_idx.
///
/// Replaces C `nvim_syn_push_current_state`.
///
/// # Safety
/// Accesses C global syntax state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_push_current_state(idx: c_int) {
    let p = nvim_syn_append_new_stateitem();
    if !p.is_null() {
        nvim_stateitem_set_idx(p, idx);
    }
}

/// Set stateitem fields at the given index in current_state.
///
/// Replaces C `nvim_syn_set_cur_state_item`.
///
/// # Safety
/// Accesses C global syntax state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_set_cur_state_item(
    idx: c_int,
    si_idx: c_int,
    si_flags: c_int,
    si_seqnr: c_int,
    si_cchar: c_int,
    extmatch: ExtMatchHandle,
) {
    let len = nvim_syn_get_current_state_len();
    if idx < 0 || idx >= len {
        return;
    }
    let item = nvim_syn_get_stateitem(idx);
    if item.is_null() {
        return;
    }
    nvim_stateitem_set_idx(item, si_idx);
    nvim_stateitem_set_flags(item, si_flags);
    nvim_stateitem_set_seqnr(item, si_seqnr);
    nvim_stateitem_set_cchar(item, si_cchar);
    let new_em = nvim_syn_ref_extmatch(extmatch);
    nvim_stateitem_set_extmatch(item, new_em);
    nvim_stateitem_set_ends(item, 0);
    nvim_stateitem_set_m_lnum(item, 0);
    // Set si_next_list based on pattern's sp_next_list
    let next_list = if si_idx >= 0 {
        nvim_syn_get_pattern_next_list(si_idx)
    } else {
        IdListHandle::null()
    };
    nvim_stateitem_set_next_list(item, next_list);
}

/// Count items with HL_FOLD flag in current_state.
///
/// Replaces C `nvim_syn_count_fold_items`.
///
/// # Safety
/// Accesses C global syntax state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_count_fold_items() -> c_int {
    let len = nvim_syn_get_current_state_len();
    let mut count = 0;
    for i in 0..len {
        let item = nvim_syn_get_stateitem(i);
        if !item.is_null() && (nvim_stateitem_get_flags(item) & HL_FOLD) != 0 {
            count += 1;
        }
    }
    count
}

/// Check if a state item at index idx has positions spanning past lnum.
///
/// Replaces C `nvim_syn_state_item_spans_line`.
///
/// # Safety
/// Accesses C global syntax state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_item_spans_line(idx: c_int, lnum: c_int) -> c_int {
    let len = nvim_syn_get_current_state_len();
    if idx < 0 || idx >= len {
        return 0;
    }
    let item = nvim_syn_get_stateitem(idx);
    if item.is_null() {
        return 0;
    }
    if nvim_stateitem_get_h_startpos_lnum(item) >= lnum
        || nvim_stateitem_get_m_endpos_lnum(item) >= lnum
        || nvim_stateitem_get_h_endpos_lnum(item) >= lnum
        || (nvim_stateitem_get_end_idx(item) != 0 && nvim_stateitem_get_eoe_pos_lnum(item) >= lnum)
    {
        return 1;
    }
    0
}

/// Clear current_state: unref all si_extmatch fields and set ga_len = 0.
///
/// Replaces C `nvim_syn_clear_current_state` (which used GA_DEEP_CLEAR).
///
/// # Safety
/// Accesses C global syntax state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_clear_current_state() {
    let len = nvim_syn_get_current_state_len();
    for i in 0..len {
        let item = nvim_syn_get_stateitem(i);
        if !item.is_null() {
            let em = nvim_stateitem_get_extmatch(item);
            nvim_syn_unref_extmatch(em);
        }
    }
    nvim_syn_set_current_state_len(0);
}

/// Walk backwards through current_state items while the item has HL_TRANS_CONT.
/// Returns the first item without HL_TRANS_CONT (or the bottom item).
///
/// Replaces C `nvim_stateitem_prev_if_trans_cont`.
///
/// # Safety
/// Accesses C global syntax state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_prev_if_trans_cont(item: StateItemHandle) -> StateItemHandle {
    if item.is_null() {
        return item;
    }
    // Find the index of this item in current_state
    let len = nvim_syn_get_current_state_len();
    // Walk backwards: we need the index of 'item' in current_state.
    // Find it by scanning (the C code uses pointer arithmetic).
    let mut idx = -1i32;
    for i in 0..len {
        let candidate = nvim_syn_get_stateitem(i);
        if candidate.0 == item.0 {
            idx = i;
            break;
        }
    }
    if idx < 0 {
        // Item not found in current_state; return as-is
        return item;
    }
    // Walk backward while HL_TRANS_CONT is set and we're not at the first item
    let mut cur_idx = idx;
    while cur_idx > 0 {
        let cur = nvim_syn_get_stateitem(cur_idx);
        if cur.is_null() || (nvim_stateitem_get_flags(cur) & HL_TRANS_CONT) == 0 {
            return cur;
        }
        cur_idx -= 1;
    }
    // Return the item at cur_idx (either HL_TRANS_CONT-free or bottom)
    nvim_syn_get_stateitem(cur_idx)
}
