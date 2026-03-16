//! State end checking and match pushing for syntax highlighting.
//!
//! This module contains the migrated implementations of:
//! - check_state_ends() — checks if current state has ended
//! - push_next_match() — pushes next matched item onto state stack
//! - check_keepend() — propagates keepend bounds
//! - update_si_attr() — updates item highlight attributes
//! - did_match_already() — prevents infinite zero-width loops

use std::ffi::c_int;

use crate::offset::limit_pos_zero;
use crate::state::Position;
use crate::types::{
    ExtMatchHandle, IdListHandle, StateItemHandle, HL_CONCEAL, HL_CONCEALENDS, HL_EXTEND,
    HL_HAS_EOL, HL_KEEPEND, HL_MATCH, HL_ONELINE, HL_SKIPEMPTY, HL_SKIPNL, HL_TRANSP,
    HL_TRANS_CONT, SPTYPE_START,
};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Current state (non-static)
    fn nvim_syn_get_current_state_len() -> c_int;
    fn nvim_syn_get_stateitem(index: c_int) -> StateItemHandle;
    fn nvim_syn_is_current_state_empty() -> c_int;

    // Next match

    // Current next list

    // Pattern accessors
    fn nvim_syn_get_pattern_type(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_flags(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_cchar(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_syn_match_id(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_syn_id(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_next_list(idx: c_int) -> IdListHandle;
    fn nvim_syn_get_pattern_cont_list(idx: c_int) -> IdListHandle;

    // Line operations
    fn nvim_syn_getcurline_byte_at(col: c_int) -> c_int;
    #[link_name = "rs_syn_update_ends"]
    fn nvim_syn_update_ends(startofline: c_int);
    fn nvim_syn_update_si_attr(idx: c_int);

    // Extmatch
    fn nvim_syn_ref_extmatch(em: ExtMatchHandle) -> ExtMatchHandle;

    // Highlight
    fn nvim_syn_id2attr_wrapper(syn_id: c_int) -> c_int;

    // ID list
    fn nvim_syn_get_id_list_all() -> IdListHandle;
}

const MAXCOL: i32 = 0x7fffffff;
const NUL: i32 = 0;

// =============================================================================
// did_match_already — prevents infinite zero-width loops
// =============================================================================

/// Check if a pattern has already matched at the current position.
///
/// Prevents infinite loops on zero-width matches by checking:
/// 1. The current state stack for items with the same idx at current position
/// 2. The zero-width matches garray for the same idx
///
/// # Safety
/// Accesses C global state.
pub unsafe fn did_match_already(idx: i32, gap_ptr: *mut c_int, gap_len: i32) -> bool {
    let state_len = nvim_syn_get_current_state_len();
    let current_col = crate::statics::CURRENT_COL;
    let current_lnum = crate::statics::CURRENT_LNUM;

    // Check current state stack from top to bottom
    let mut i = state_len - 1;
    while i >= 0 {
        let item = nvim_syn_get_stateitem(i);
        if !item.is_null() {
            let p = item.as_ptr();
            if (*p).si_m_startcol == current_col
                && (*p).si_m_lnum == current_lnum
                && (*p).si_idx == idx
            {
                return true;
            }
        }
        i -= 1;
    }

    // Check zero-width matches array
    if !gap_ptr.is_null() {
        for i in 0..gap_len {
            if *gap_ptr.offset(i as isize) == idx {
                return true;
            }
        }
    }

    false
}

// =============================================================================
// update_si_attr — updates item highlight attributes
// =============================================================================

/// Update an item's highlight ID, attribute, and containment list.
///
/// # Safety
/// Accesses C global state.
pub unsafe fn update_si_attr(stack_idx: i32) {
    let state_len = nvim_syn_get_current_state_len();
    if stack_idx < 0 || stack_idx >= state_len {
        return;
    }

    let sip = nvim_syn_get_stateitem(stack_idx);
    if sip.is_null() {
        return;
    }

    let p = sip.as_ptr();
    let si_idx = (*p).si_idx;
    if si_idx < 0 {
        return;
    }

    let si_flags = (*p).si_flags;

    // Set highlight ID
    let hl_id = if si_flags & HL_MATCH != 0 {
        nvim_syn_get_pattern_syn_match_id(si_idx)
    } else {
        nvim_syn_get_pattern_syn_id(si_idx)
    };
    (*p).si_id = hl_id;
    (*p).si_attr = nvim_syn_id2attr_wrapper(hl_id);
    (*p).si_trans_id = hl_id;

    // Set continue list
    if si_flags & HL_MATCH != 0 {
        (*p).si_cont_list = std::ptr::null_mut();
    } else {
        (*p).si_cont_list = nvim_syn_get_pattern_cont_list(si_idx).0;
    }

    // For transparent items, take attr from outer item.
    let pat_flags = nvim_syn_get_pattern_flags(si_idx);
    if (pat_flags & HL_TRANSP != 0) && (si_flags & HL_MATCH == 0) {
        if stack_idx == 0 {
            (*p).si_attr = 0;
            (*p).si_trans_id = 0;
            if (*p).si_cont_list.is_null() {
                (*p).si_cont_list = nvim_syn_get_id_list_all().0;
            }
        } else {
            let parent = nvim_syn_get_stateitem(stack_idx - 1);
            let pp = parent.as_ptr();
            (*p).si_attr = (*pp).si_attr;
            (*p).si_trans_id = (*pp).si_trans_id;
            if (*p).si_cont_list.is_null() {
                (*p).si_flags |= HL_TRANS_CONT;
                (*p).si_cont_list = (*pp).si_cont_list;
            }
        }
    }
}

// =============================================================================
// check_keepend — propagates keepend bounds
// =============================================================================

/// Check the current stack for patterns with "keepend" flag.
/// Propagate the match-end to contained items, until a "skipend" item is found.
///
/// # Safety
/// Accesses C global state.
pub unsafe fn check_keepend() {
    let keepend_level = crate::statics::KEEPEND_LEVEL;
    if keepend_level < 0 {
        return;
    }

    let state_len = nvim_syn_get_current_state_len();

    // Find the last index of an "extend" item. "keepend" items before that
    // won't do anything.
    let mut i = state_len - 1;
    while i > keepend_level {
        let item = nvim_syn_get_stateitem(i);
        if !item.is_null() && ((*item.as_ptr()).si_flags & HL_EXTEND != 0) {
            break;
        }
        i -= 1;
    }

    let mut maxpos = Position { lnum: 0, col: 0 };
    let mut maxpos_h = Position { lnum: 0, col: 0 };

    while i < state_len {
        let sip = nvim_syn_get_stateitem(i);
        if sip.is_null() {
            i += 1;
            continue;
        }

        let p = sip.as_ptr();

        if maxpos.lnum != 0 {
            // Limit all end positions to the keepend boundary
            let mut m_endpos = Position {
                lnum: (*p).si_m_endpos.lnum,
                col: (*p).si_m_endpos.col,
            };
            limit_pos_zero(&mut m_endpos, &maxpos);

            let mut h_endpos = Position {
                lnum: (*p).si_h_endpos.lnum,
                col: (*p).si_h_endpos.col,
            };
            limit_pos_zero(&mut h_endpos, &maxpos_h);

            let mut eoe_pos = Position {
                lnum: (*p).si_eoe_pos.lnum,
                col: (*p).si_eoe_pos.col,
            };
            limit_pos_zero(&mut eoe_pos, &maxpos);

            (*p).si_m_endpos.lnum = m_endpos.lnum;
            (*p).si_m_endpos.col = m_endpos.col;
            (*p).si_h_endpos.lnum = h_endpos.lnum;
            (*p).si_h_endpos.col = h_endpos.col;
            (*p).si_eoe_pos.lnum = eoe_pos.lnum;
            (*p).si_eoe_pos.col = eoe_pos.col;

            (*p).si_ends = 1;
        }

        let si_ends = (*p).si_ends;
        let si_flags = (*p).si_flags;
        if si_ends != 0 && (si_flags & HL_KEEPEND != 0) {
            let m_end = Position {
                lnum: (*p).si_m_endpos.lnum,
                col: (*p).si_m_endpos.col,
            };
            if maxpos.lnum == 0
                || maxpos.lnum > m_end.lnum
                || (maxpos.lnum == m_end.lnum && maxpos.col > m_end.col)
            {
                maxpos = m_end;
            }
            let h_end = Position {
                lnum: (*p).si_h_endpos.lnum,
                col: (*p).si_h_endpos.col,
            };
            if maxpos_h.lnum == 0
                || maxpos_h.lnum > h_end.lnum
                || (maxpos_h.lnum == h_end.lnum && maxpos_h.col > h_end.col)
            {
                maxpos_h = h_end;
            }
        }

        i += 1;
    }
}

// =============================================================================
// push_next_match — pushes next matched item onto state stack
// =============================================================================

/// Push the next match onto the state stack.
///
/// # Safety
/// Accesses C global state extensively.
pub unsafe fn push_next_match() -> StateItemHandle {
    let next_match_idx = crate::statics::NEXT_MATCH_IDX;
    let current_col = crate::statics::CURRENT_COL;
    let current_lnum = crate::statics::CURRENT_LNUM;

    let pat_type = nvim_syn_get_pattern_type(next_match_idx);
    let pat_flags = nvim_syn_get_pattern_flags(next_match_idx);
    let pat_cchar = nvim_syn_get_pattern_cchar(next_match_idx);

    // Push the item in current_state stack
    crate::state_ops::rs_syn_push_current_state(next_match_idx);

    let state_len = nvim_syn_get_current_state_len();
    let cur_si = nvim_syn_get_stateitem(state_len - 1);

    // Read all next_match position values in a single bulk call
    let positions = crate::match_engine::next_match_positions();
    let h_start_lnum = positions.h_startpos.lnum;
    let h_start_col = positions.h_startpos.col;

    let cp = cur_si.as_ptr();
    (*cp).si_m_lnum = current_lnum;
    (*cp).si_m_startcol = current_col;
    (*cp).si_h_startpos.lnum = h_start_lnum;
    (*cp).si_h_startpos.col = h_start_col;
    (*cp).si_flags = pat_flags;
    let seqnr = {
        let _s = crate::statics::NEXT_SEQNR;
        crate::statics::NEXT_SEQNR += 1;
        _s
    };
    (*cp).si_seqnr = seqnr;
    (*cp).si_cchar = pat_cchar;

    // Inherit conceal flag from parent
    let new_state_len = nvim_syn_get_current_state_len();
    if new_state_len > 1 {
        let parent = nvim_syn_get_stateitem(new_state_len - 2);
        (*cp).si_flags |= (*parent.as_ptr()).si_flags & HL_CONCEAL;
    }

    (*cp).si_next_list = nvim_syn_get_pattern_next_list(next_match_idx).0;
    let extmatch = ExtMatchHandle(crate::statics::NEXT_MATCH_EXTMATCH);
    let reffed = nvim_syn_ref_extmatch(extmatch);
    (*cp).si_extmatch = reffed.0 as *mut _;

    if pat_type == SPTYPE_START && (pat_flags & HL_ONELINE == 0) {
        // Try to find the end pattern in the current line
        crate::region::update_si_end(cur_si, positions.m_endpos.col, true);
        check_keepend();
    } else {
        (*cp).si_m_endpos.lnum = positions.m_endpos.lnum;
        (*cp).si_m_endpos.col = positions.m_endpos.col;
        (*cp).si_h_endpos.lnum = positions.h_endpos.lnum;
        (*cp).si_h_endpos.col = positions.h_endpos.col;
        (*cp).si_eoe_pos.lnum = positions.eoe_pos.lnum;
        (*cp).si_eoe_pos.col = positions.eoe_pos.col;
        (*cp).si_ends = 1;
        (*cp).si_flags |= crate::statics::NEXT_MATCH_FLAGS;
        (*cp).si_end_idx = crate::statics::NEXT_MATCH_END_IDX;
    }

    let keepend_level = crate::statics::KEEPEND_LEVEL;
    let cur_flags = (*cp).si_flags;
    if keepend_level < 0 && (cur_flags & HL_KEEPEND != 0) {
        let sl = nvim_syn_get_current_state_len();
        crate::statics::KEEPEND_LEVEL = sl - 1;
    }
    check_keepend();
    update_si_attr(nvim_syn_get_current_state_len() - 1);

    let save_flags = (*cp).si_flags & (HL_CONCEAL | HL_CONCEALENDS);

    // If the start pattern has another highlight group, push another item
    // on the stack for the start pattern.
    let pat_match_id = nvim_syn_get_pattern_syn_match_id(next_match_idx);
    if pat_type == SPTYPE_START && pat_match_id != 0 {
        crate::state_ops::rs_syn_push_current_state(next_match_idx);
        let sl = nvim_syn_get_current_state_len();
        let mg_si = nvim_syn_get_stateitem(sl - 1);
        let mp = mg_si.as_ptr();

        (*mp).si_m_lnum = current_lnum;
        (*mp).si_m_startcol = current_col;
        (*mp).si_m_endpos.lnum = positions.eos_pos.lnum;
        (*mp).si_m_endpos.col = positions.eos_pos.col;
        (*mp).si_h_startpos.lnum = h_start_lnum;
        (*mp).si_h_startpos.col = h_start_col;
        (*mp).si_h_endpos.lnum = positions.eos_pos.lnum;
        (*mp).si_h_endpos.col = positions.eos_pos.col;
        (*mp).si_ends = 1;
        (*mp).si_end_idx = 0;
        (*mp).si_flags = HL_MATCH;
        let mg_seqnr = {
            let _s = crate::statics::NEXT_SEQNR;
            crate::statics::NEXT_SEQNR += 1;
            _s
        };
        (*mp).si_seqnr = mg_seqnr;
        (*mp).si_flags |= save_flags;
        if (*mp).si_flags & HL_CONCEALENDS != 0 {
            (*mp).si_flags |= HL_CONCEAL;
        }
        (*mp).si_next_list = std::ptr::null_mut();
        check_keepend();
        let new_sl = nvim_syn_get_current_state_len();
        update_si_attr(new_sl - 1);
    }

    crate::statics::NEXT_MATCH_IDX = -1; // try other match next time

    // Return the top state item
    let final_len = nvim_syn_get_current_state_len();
    nvim_syn_get_stateitem(final_len - 1)
}

// =============================================================================
// check_state_ends — checks if current state has ended
// =============================================================================

/// Check for end of current state (and the states before it).
///
/// # Safety
/// Accesses C global state extensively.
pub unsafe fn check_state_ends() {
    let state_len = nvim_syn_get_current_state_len();
    if state_len == 0 {
        return;
    }

    let current_lnum = crate::statics::CURRENT_LNUM;
    let current_col = crate::statics::CURRENT_COL;

    loop {
        let sl = nvim_syn_get_current_state_len();
        if sl == 0 {
            break;
        }
        let cur_si = nvim_syn_get_stateitem(sl - 1);
        if cur_si.is_null() {
            break;
        }

        let p = cur_si.as_ptr();
        let si_ends = (*p).si_ends;
        let m_end_lnum = (*p).si_m_endpos.lnum;
        let m_end_col = (*p).si_m_endpos.col;

        if si_ends != 0
            && (m_end_lnum < current_lnum
                || (m_end_lnum == current_lnum && m_end_col <= current_col))
        {
            // Check if there is an end pattern group ID to highlight
            let end_idx = (*p).si_end_idx;
            let eoe_lnum = (*p).si_eoe_pos.lnum;
            let eoe_col = (*p).si_eoe_pos.col;

            if end_idx != 0
                && (eoe_lnum > current_lnum || (eoe_lnum == current_lnum && eoe_col > current_col))
            {
                // Switch the item to show the end pattern match
                (*p).si_idx = end_idx;
                (*p).si_end_idx = 0;
                (*p).si_m_endpos.lnum = eoe_lnum;
                (*p).si_m_endpos.col = eoe_col;
                (*p).si_h_endpos.lnum = eoe_lnum;
                (*p).si_h_endpos.col = eoe_col;
                (*p).si_flags |= HL_MATCH;
                let seqnr = {
                    let _s = crate::statics::NEXT_SEQNR;
                    crate::statics::NEXT_SEQNR += 1;
                    _s
                };
                (*p).si_seqnr = seqnr;
                if (*p).si_flags & HL_CONCEALENDS != 0 {
                    (*p).si_flags |= HL_CONCEAL;
                }
                let new_sl = nvim_syn_get_current_state_len();
                update_si_attr(new_sl - 1);

                // nextgroup= should not match in the end pattern
                crate::statics::CURRENT_NEXT_LIST = std::ptr::null_mut();

                // What matches next may be different now, clear it
                crate::statics::NEXT_MATCH_IDX = 0;
                crate::statics::NEXT_MATCH_COL = MAXCOL;
                break;
            }

            // Handle next_list
            let si_next_list = IdListHandle((*p).si_next_list);
            let si_flags = (*p).si_flags;
            crate::statics::CURRENT_NEXT_LIST = si_next_list.0;
            crate::statics::CURRENT_NEXT_FLAGS = si_flags;

            // Unless at end of line and no "skipnl" or "skipempty"
            let next_flags = si_flags;
            if (next_flags & (HL_SKIPNL | HL_SKIPEMPTY) == 0)
                && nvim_syn_getcurline_byte_at(current_col) == NUL
            {
                crate::statics::CURRENT_NEXT_LIST = std::ptr::null_mut();
            }

            // When the ended item has "extend", another item with "keepend"
            // now needs to check for its end.
            let had_extend = si_flags & HL_EXTEND;

            crate::state_ops::rs_syn_pop_current_state();

            if nvim_syn_is_current_state_empty() != 0 {
                break;
            }

            let keepend_level = crate::statics::KEEPEND_LEVEL;
            if had_extend != 0 && keepend_level >= 0 {
                nvim_syn_update_ends(0);
                if nvim_syn_is_current_state_empty() != 0 {
                    break;
                }
            }

            // Only for a region the search for the end continues after the end
            // of the contained item.
            let new_sl = nvim_syn_get_current_state_len();
            let new_cur = nvim_syn_get_stateitem(new_sl - 1);
            let new_idx = (*new_cur.as_ptr()).si_idx;
            let new_flags = (*new_cur.as_ptr()).si_flags;

            if new_idx >= 0
                && nvim_syn_get_pattern_type(new_idx) == SPTYPE_START
                && (new_flags & (HL_MATCH | HL_KEEPEND) == 0)
            {
                crate::region::update_si_end(new_cur, current_col, true);
                check_keepend();
                if (next_flags & HL_HAS_EOL != 0)
                    && crate::statics::KEEPEND_LEVEL < 0
                    && nvim_syn_getcurline_byte_at(current_col) == NUL
                {
                    break;
                }
            }
        } else {
            break;
        }
    }
}

// =============================================================================
// Exported FFI functions
// =============================================================================
/// Rust implementation of update_si_attr.
#[no_mangle]
pub unsafe extern "C" fn rs_update_si_attr(idx: c_int) {
    update_si_attr(idx);
}

/// Rust implementation of check_keepend.
#[no_mangle]
pub unsafe extern "C" fn rs_check_keepend() {
    check_keepend();
}

/// Rust implementation of push_next_match.
#[no_mangle]
pub unsafe extern "C" fn rs_push_next_match() -> StateItemHandle {
    push_next_match()
}

/// Rust implementation of check_state_ends.
#[no_mangle]
pub unsafe extern "C" fn rs_check_state_ends() {
    check_state_ends();
}
