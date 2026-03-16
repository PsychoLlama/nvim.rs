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
//! - nvim_syn_extmatch_equal        (pointer comparison, tri-state)
//! - nvim_syn_extmatch_strings_equal (per-subidx string comparison)
//! - nvim_cur_state_set_matchcont   (set HL_MATCHCONT, clear positions, set si_ends)

use std::ffi::{c_char, c_int};

use crate::types::{
    ExtMatchHandle, IdListHandle, StateItemHandle, HL_FOLD, HL_MATCHCONT, HL_TRANS_CONT,
};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Current state length/grow

    // Stateitem access

    // Extmatch reference management
    fn nvim_syn_unref_extmatch(em: ExtMatchHandle);
    fn nvim_syn_ref_extmatch(em: ExtMatchHandle) -> ExtMatchHandle;

    // next_match_idx

    // keepend_level

    // Stateitem append helper (GA_APPEND_VIA_PTR + CLEAR_POINTER)

    // Pattern next_list lookup
    fn nvim_syn_get_pattern_next_list(idx: c_int) -> IdListHandle;

    // next_match col setter (kept; used standalone)

    // Phase 3: extmatch string comparison helpers
    fn nvim_extmatch_get_string(em: ExtMatchHandle, subidx: c_int) -> *const c_char;
    fn nvim_syn_mb_strcmp_ic(ic: c_int, a: *const c_char, b: *const c_char) -> c_int;
    fn nvim_synblock_pattern_ic(pat_idx: c_int) -> c_int;

}

// =============================================================================
// Phase 2 Rust implementations
// =============================================================================

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
    let len = crate::statics::CURRENT_STATE.ga_len;
    if len <= 0 {
        return;
    }
    let top = crate::statics::current_state_item(len - 1);
    if !top.is_null() {
        let em = ExtMatchHandle((*top.as_ptr()).si_extmatch as *mut _);
        nvim_syn_unref_extmatch(em);
    }
    crate::statics::CURRENT_STATE.ga_len = len - 1;
    // After end of a pattern, try matching a keyword or pattern next time
    crate::statics::NEXT_MATCH_IDX = -1;
    // If first state with "keepend" is popped, reset keepend_level
    let keepend = crate::statics::KEEPEND_LEVEL;
    if keepend >= len - 1 {
        crate::statics::KEEPEND_LEVEL = -1;
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
    let p = crate::statics::current_state_append();
    if !p.is_null() {
        (*p.as_ptr()).si_idx = idx;
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
    let len = crate::statics::CURRENT_STATE.ga_len;
    if idx < 0 || idx >= len {
        return;
    }
    let item = crate::statics::current_state_item(idx);
    if item.is_null() {
        return;
    }
    {
        let p = item.as_ptr();
        (*p).si_idx = si_idx;
        (*p).si_flags = si_flags;
        (*p).si_seqnr = si_seqnr;
        (*p).si_cchar = si_cchar;
    }
    let new_em = nvim_syn_ref_extmatch(extmatch);
    (*item.as_ptr()).si_extmatch = new_em.0 as *mut _;
    {
        let p = item.as_ptr();
        (*p).si_ends = 0;
        (*p).si_m_lnum = 0;
    }
    // Set si_next_list based on pattern's sp_next_list
    let next_list = if si_idx >= 0 {
        nvim_syn_get_pattern_next_list(si_idx)
    } else {
        IdListHandle::null()
    };
    (*item.as_ptr()).si_next_list = next_list.0;
}

/// Count items with HL_FOLD flag in current_state.
///
/// Replaces C `nvim_syn_count_fold_items`.
///
/// # Safety
/// Accesses C global syntax state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_count_fold_items() -> c_int {
    let len = crate::statics::CURRENT_STATE.ga_len;
    let mut count = 0;
    for i in 0..len {
        let item = crate::statics::current_state_item(i);
        if !item.is_null() && ((*item.as_ptr()).si_flags & HL_FOLD) != 0 {
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
    let len = crate::statics::CURRENT_STATE.ga_len;
    if idx < 0 || idx >= len {
        return 0;
    }
    let item = crate::statics::current_state_item(idx);
    if item.is_null() {
        return 0;
    }
    let (m_end_lnum, h_start_lnum, h_end_lnum, eoe_lnum, si_end_idx) = {
        let p = item.as_ptr();
        (
            (*p).si_m_endpos.lnum,
            (*p).si_h_startpos.lnum,
            (*p).si_h_endpos.lnum,
            (*p).si_eoe_pos.lnum,
            (*p).si_end_idx,
        )
    };
    if h_start_lnum >= lnum
        || m_end_lnum >= lnum
        || h_end_lnum >= lnum
        || (si_end_idx != 0 && eoe_lnum >= lnum)
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
    let len = crate::statics::CURRENT_STATE.ga_len;
    for i in 0..len {
        let item = crate::statics::current_state_item(i);
        if !item.is_null() {
            let em = ExtMatchHandle((*item.as_ptr()).si_extmatch as *mut _);
            nvim_syn_unref_extmatch(em);
        }
    }
    crate::statics::CURRENT_STATE.ga_len = 0;
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
    let len = crate::statics::CURRENT_STATE.ga_len;
    // Walk backwards: we need the index of 'item' in current_state.
    // Find it by scanning (the C code uses pointer arithmetic).
    let mut idx = -1i32;
    for i in 0..len {
        let candidate = crate::statics::current_state_item(i);
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
        let cur = crate::statics::current_state_item(cur_idx);
        if cur.is_null() || ((*cur.as_ptr()).si_flags & HL_TRANS_CONT) == 0 {
            return cur;
        }
        cur_idx -= 1;
    }
    // Return the item at cur_idx (either HL_TRANS_CONT-free or bottom)
    crate::statics::current_state_item(cur_idx)
}

// =============================================================================
// Phase 3 Rust implementations
// =============================================================================

/// Compare two extmatch pointers.
///
/// Returns 1 if same pointer (trivially equal), 0 if one is NULL and the other
/// isn't, or -1 if both are non-null and different (caller must compare strings).
///
/// Replaces C `nvim_syn_extmatch_equal`.
///
/// # Safety
/// Handles may be null; null-checks are performed before dereferencing.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_extmatch_equal(a: ExtMatchHandle, b: ExtMatchHandle) -> c_int {
    if a.0 == b.0 {
        return 1;
    }
    if a.is_null() || b.is_null() {
        return 0;
    }
    -1 // Both non-null and different: need string comparison
}

/// Compare extmatch strings at a given sub-index using the pattern's ignore-case flag.
///
/// Returns 1 if equal, 0 if different.
///
/// Replaces C `nvim_syn_extmatch_strings_equal`.
///
/// # Safety
/// a and b must be valid non-null `reg_extmatch_T` pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_extmatch_strings_equal(
    a: ExtMatchHandle,
    b: ExtMatchHandle,
    subidx: c_int,
    pat_idx: c_int,
) -> c_int {
    let nsubexp = crate::types::NSUBEXP;
    if subidx < 0 || subidx >= nsubexp {
        return 0;
    }

    let sa = nvim_extmatch_get_string(a, subidx);
    let sb = nvim_extmatch_get_string(b, subidx);

    // Same pointer (including both NULL) = equal
    if sa == sb {
        return 1;
    }
    // One is NULL and the other isn't = not equal
    if sa.is_null() || sb.is_null() {
        return 0;
    }

    let ic = nvim_synblock_pattern_ic(pat_idx);
    if nvim_syn_mb_strcmp_ic(ic, sa, sb) == 0 {
        1
    } else {
        0
    }
}

/// Set HL_MATCHCONT flag on current_state item i, clear m_endpos/h_endpos, set si_ends.
///
/// Replaces C `nvim_cur_state_set_matchcont`.
///
/// # Safety
/// Accesses C global syntax state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_cur_state_set_matchcont(i: c_int) {
    let len = crate::statics::CURRENT_STATE.ga_len;
    if i < 0 || i >= len {
        return;
    }
    let item = crate::statics::current_state_item(i);
    if item.is_null() {
        return;
    }
    {
        let p = item.as_ptr();
        (*p).si_flags |= HL_MATCHCONT;
        (*p).si_m_endpos.lnum = 0;
        (*p).si_m_endpos.col = 0;
        (*p).si_h_endpos.lnum = 0;
        (*p).si_h_endpos.col = 0;
        (*p).si_ends = 1;
    }
}
