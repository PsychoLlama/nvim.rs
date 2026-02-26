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
    // Current state
    fn nvim_syn_get_current_state_len() -> c_int;
    fn nvim_syn_get_current_lnum() -> c_int;
    fn nvim_syn_get_current_col() -> c_int;
    fn nvim_syn_get_stateitem(index: c_int) -> StateItemHandle;
    fn nvim_syn_is_current_state_empty() -> c_int;
    fn nvim_syn_get_keepend_level() -> c_int;
    fn nvim_syn_set_keepend_level(level: c_int);

    // Next match
    fn nvim_syn_get_next_match_idx() -> c_int;
    fn nvim_syn_set_next_match_idx(idx: c_int);
    fn nvim_syn_set_next_match_col(col: c_int);
    fn nvim_syn_get_next_match_flags() -> c_int;
    fn nvim_syn_get_next_match_end_idx() -> c_int;
    fn nvim_syn_get_next_match_extmatch() -> ExtMatchHandle;
    fn nvim_syn_get_next_match_h_startpos(lnum: *mut c_int, col: *mut c_int);
    fn nvim_syn_get_next_match_m_endpos(lnum: *mut c_int, col: *mut c_int);
    fn nvim_syn_get_next_match_h_endpos(lnum: *mut c_int, col: *mut c_int);
    fn nvim_syn_get_next_match_eos_pos(lnum: *mut c_int, col: *mut c_int);
    fn nvim_syn_get_next_match_eoe_pos(lnum: *mut c_int, col: *mut c_int);
    fn nvim_syn_get_next_match_col() -> c_int;
    fn nvim_syn_set_next_match_col_val(col: c_int);

    // Next sequence number
    fn nvim_syn_incr_next_seqnr() -> c_int;

    // State item accessors
    fn nvim_stateitem_get_idx(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_id(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_flags(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_ends(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_end_idx(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_m_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_m_startcol(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_m_endpos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_m_endpos_col(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_h_endpos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_h_endpos_col(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_eoe_pos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_eoe_pos_col(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_next_list(item: StateItemHandle) -> IdListHandle;
    fn nvim_stateitem_get_attr(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_trans_id(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_cont_list(item: StateItemHandle) -> IdListHandle;
    fn nvim_stateitem_get_seqnr(item: StateItemHandle) -> c_int;

    // State item setters
    fn nvim_stateitem_set_idx(item: StateItemHandle, idx: c_int);
    fn nvim_stateitem_set_id(item: StateItemHandle, id: c_int);
    fn nvim_stateitem_set_trans_id(item: StateItemHandle, trans_id: c_int);
    fn nvim_stateitem_set_attr(item: StateItemHandle, attr: c_int);
    fn nvim_stateitem_set_flags(item: StateItemHandle, flags: c_int);
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

    // Current next list
    fn nvim_syn_set_current_next_list(list: IdListHandle);
    fn nvim_syn_set_current_next_flags(flags: c_int);

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
    let current_col = nvim_syn_get_current_col();
    let current_lnum = nvim_syn_get_current_lnum();

    // Check current state stack from top to bottom
    let mut i = state_len - 1;
    while i >= 0 {
        let item = nvim_syn_get_stateitem(i);
        if !item.is_null()
            && nvim_stateitem_get_m_startcol(item) == current_col
            && nvim_stateitem_get_m_lnum(item) == current_lnum
            && nvim_stateitem_get_idx(item) == idx
        {
            return true;
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

    let si_idx = nvim_stateitem_get_idx(sip);
    if si_idx < 0 {
        return;
    }

    let si_flags = nvim_stateitem_get_flags(sip);

    // Set highlight ID
    let hl_id = if si_flags & HL_MATCH != 0 {
        nvim_syn_get_pattern_syn_match_id(si_idx)
    } else {
        nvim_syn_get_pattern_syn_id(si_idx)
    };
    nvim_stateitem_set_id(sip, hl_id);
    nvim_stateitem_set_attr(sip, nvim_syn_id2attr_wrapper(hl_id));
    nvim_stateitem_set_trans_id(sip, hl_id);

    // Set continue list
    if si_flags & HL_MATCH != 0 {
        nvim_stateitem_set_cont_list(sip, IdListHandle::null());
    } else {
        nvim_stateitem_set_cont_list(sip, nvim_syn_get_pattern_cont_list(si_idx));
    }

    // For transparent items, take attr from outer item.
    let pat_flags = nvim_syn_get_pattern_flags(si_idx);
    if (pat_flags & HL_TRANSP != 0) && (si_flags & HL_MATCH == 0) {
        if stack_idx == 0 {
            nvim_stateitem_set_attr(sip, 0);
            nvim_stateitem_set_trans_id(sip, 0);
            let cont_list = nvim_stateitem_get_cont_list(sip);
            if cont_list.is_null() {
                nvim_stateitem_set_cont_list(sip, nvim_syn_get_id_list_all());
            }
        } else {
            let parent = nvim_syn_get_stateitem(stack_idx - 1);
            nvim_stateitem_set_attr(sip, nvim_stateitem_get_attr(parent));
            nvim_stateitem_set_trans_id(sip, nvim_stateitem_get_trans_id(parent));
            let cont_list = nvim_stateitem_get_cont_list(sip);
            if cont_list.is_null() {
                nvim_stateitem_or_flags(sip, HL_TRANS_CONT);
                nvim_stateitem_set_cont_list(sip, nvim_stateitem_get_cont_list(parent));
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
    let keepend_level = nvim_syn_get_keepend_level();
    if keepend_level < 0 {
        return;
    }

    let state_len = nvim_syn_get_current_state_len();

    // Find the last index of an "extend" item. "keepend" items before that
    // won't do anything.
    let mut i = state_len - 1;
    while i > keepend_level {
        let item = nvim_syn_get_stateitem(i);
        if !item.is_null() && (nvim_stateitem_get_flags(item) & HL_EXTEND != 0) {
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

        if maxpos.lnum != 0 {
            // Limit all end positions to the keepend boundary
            let mut m_endpos = Position {
                lnum: nvim_stateitem_get_m_endpos_lnum(sip),
                col: nvim_stateitem_get_m_endpos_col(sip),
            };
            limit_pos_zero(&mut m_endpos, &maxpos);
            nvim_stateitem_set_m_endpos(sip, m_endpos.lnum, m_endpos.col);

            let mut h_endpos = Position {
                lnum: nvim_stateitem_get_h_endpos_lnum(sip),
                col: nvim_stateitem_get_h_endpos_col(sip),
            };
            limit_pos_zero(&mut h_endpos, &maxpos_h);
            nvim_stateitem_set_h_endpos(sip, h_endpos.lnum, h_endpos.col);

            let mut eoe_pos = Position {
                lnum: nvim_stateitem_get_eoe_pos_lnum(sip),
                col: nvim_stateitem_get_eoe_pos_col(sip),
            };
            limit_pos_zero(&mut eoe_pos, &maxpos);
            nvim_stateitem_set_eoe_pos(sip, eoe_pos.lnum, eoe_pos.col);

            nvim_stateitem_set_ends(sip, 1);
        }

        let si_ends = nvim_stateitem_get_ends(sip);
        let si_flags = nvim_stateitem_get_flags(sip);
        if si_ends != 0 && (si_flags & HL_KEEPEND != 0) {
            let m_end = Position {
                lnum: nvim_stateitem_get_m_endpos_lnum(sip),
                col: nvim_stateitem_get_m_endpos_col(sip),
            };
            if maxpos.lnum == 0
                || maxpos.lnum > m_end.lnum
                || (maxpos.lnum == m_end.lnum && maxpos.col > m_end.col)
            {
                maxpos = m_end;
            }
            let h_end = Position {
                lnum: nvim_stateitem_get_h_endpos_lnum(sip),
                col: nvim_stateitem_get_h_endpos_col(sip),
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
    let next_match_idx = nvim_syn_get_next_match_idx();
    let current_col = nvim_syn_get_current_col();
    let current_lnum = nvim_syn_get_current_lnum();

    let pat_type = nvim_syn_get_pattern_type(next_match_idx);
    let pat_flags = nvim_syn_get_pattern_flags(next_match_idx);
    let pat_cchar = nvim_syn_get_pattern_cchar(next_match_idx);

    // Push the item in current_state stack
    crate::state_ops::rs_syn_push_current_state(next_match_idx);

    let state_len = nvim_syn_get_current_state_len();
    let cur_si = nvim_syn_get_stateitem(state_len - 1);

    // Read next_match position values
    let mut h_start_lnum: c_int = 0;
    let mut h_start_col: c_int = 0;
    nvim_syn_get_next_match_h_startpos(&mut h_start_lnum, &mut h_start_col);
    nvim_stateitem_set_h_startpos(cur_si, h_start_lnum, h_start_col);
    nvim_stateitem_set_m_startcol(cur_si, current_col);
    nvim_stateitem_set_m_lnum(cur_si, current_lnum);
    nvim_stateitem_set_flags(cur_si, pat_flags);
    let seqnr = nvim_syn_incr_next_seqnr();
    nvim_stateitem_set_seqnr(cur_si, seqnr);
    nvim_stateitem_set_cchar(cur_si, pat_cchar);

    // Inherit conceal flag from parent
    let new_state_len = nvim_syn_get_current_state_len();
    if new_state_len > 1 {
        let parent = nvim_syn_get_stateitem(new_state_len - 2);
        let parent_flags = nvim_stateitem_get_flags(parent);
        nvim_stateitem_or_flags(cur_si, parent_flags & HL_CONCEAL);
    }

    let next_list = nvim_syn_get_pattern_next_list(next_match_idx);
    nvim_stateitem_set_next_list(cur_si, next_list);
    let extmatch = nvim_syn_get_next_match_extmatch();
    let reffed = nvim_syn_ref_extmatch(extmatch);
    nvim_stateitem_set_extmatch(cur_si, reffed);

    if pat_type == SPTYPE_START && (pat_flags & HL_ONELINE == 0) {
        // Try to find the end pattern in the current line
        let mut m_end_lnum: c_int = 0;
        let mut m_end_col: c_int = 0;
        nvim_syn_get_next_match_m_endpos(&mut m_end_lnum, &mut m_end_col);
        crate::region::update_si_end(cur_si, m_end_col, true);
        check_keepend();
    } else {
        let mut m_end_lnum: c_int = 0;
        let mut m_end_col: c_int = 0;
        nvim_syn_get_next_match_m_endpos(&mut m_end_lnum, &mut m_end_col);
        nvim_stateitem_set_m_endpos(cur_si, m_end_lnum, m_end_col);

        let mut h_end_lnum: c_int = 0;
        let mut h_end_col: c_int = 0;
        nvim_syn_get_next_match_h_endpos(&mut h_end_lnum, &mut h_end_col);
        nvim_stateitem_set_h_endpos(cur_si, h_end_lnum, h_end_col);

        nvim_stateitem_set_ends(cur_si, 1);
        nvim_stateitem_or_flags(cur_si, nvim_syn_get_next_match_flags());

        let mut eoe_lnum: c_int = 0;
        let mut eoe_col: c_int = 0;
        nvim_syn_get_next_match_eoe_pos(&mut eoe_lnum, &mut eoe_col);
        nvim_stateitem_set_eoe_pos(cur_si, eoe_lnum, eoe_col);
        nvim_stateitem_set_end_idx(cur_si, nvim_syn_get_next_match_end_idx());
    }

    let keepend_level = nvim_syn_get_keepend_level();
    let cur_flags = nvim_stateitem_get_flags(cur_si);
    if keepend_level < 0 && (cur_flags & HL_KEEPEND != 0) {
        let sl = nvim_syn_get_current_state_len();
        nvim_syn_set_keepend_level(sl - 1);
    }
    check_keepend();
    update_si_attr(nvim_syn_get_current_state_len() - 1);

    let save_flags = nvim_stateitem_get_flags(cur_si) & (HL_CONCEAL | HL_CONCEALENDS);

    // If the start pattern has another highlight group, push another item
    // on the stack for the start pattern.
    let pat_match_id = nvim_syn_get_pattern_syn_match_id(next_match_idx);
    if pat_type == SPTYPE_START && pat_match_id != 0 {
        crate::state_ops::rs_syn_push_current_state(next_match_idx);
        let sl = nvim_syn_get_current_state_len();
        let mg_si = nvim_syn_get_stateitem(sl - 1);

        nvim_stateitem_set_h_startpos(mg_si, h_start_lnum, h_start_col);
        nvim_stateitem_set_m_startcol(mg_si, current_col);
        nvim_stateitem_set_m_lnum(mg_si, current_lnum);

        let mut eos_lnum: c_int = 0;
        let mut eos_col: c_int = 0;
        nvim_syn_get_next_match_eos_pos(&mut eos_lnum, &mut eos_col);
        nvim_stateitem_set_m_endpos(mg_si, eos_lnum, eos_col);
        nvim_stateitem_set_h_endpos(mg_si, eos_lnum, eos_col);
        nvim_stateitem_set_ends(mg_si, 1);
        nvim_stateitem_set_end_idx(mg_si, 0);
        nvim_stateitem_set_flags(mg_si, HL_MATCH);
        let mg_seqnr = nvim_syn_incr_next_seqnr();
        nvim_stateitem_set_seqnr(mg_si, mg_seqnr);
        nvim_stateitem_or_flags(mg_si, save_flags);
        let mg_flags = nvim_stateitem_get_flags(mg_si);
        if mg_flags & HL_CONCEALENDS != 0 {
            nvim_stateitem_or_flags(mg_si, HL_CONCEAL);
        }
        nvim_stateitem_set_next_list(mg_si, IdListHandle::null());
        check_keepend();
        let new_sl = nvim_syn_get_current_state_len();
        update_si_attr(new_sl - 1);
    }

    nvim_syn_set_next_match_idx(-1); // try other match next time

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

    let current_lnum = nvim_syn_get_current_lnum();
    let current_col = nvim_syn_get_current_col();

    loop {
        let sl = nvim_syn_get_current_state_len();
        if sl == 0 {
            break;
        }
        let cur_si = nvim_syn_get_stateitem(sl - 1);
        if cur_si.is_null() {
            break;
        }

        let si_ends = nvim_stateitem_get_ends(cur_si);
        let m_end_lnum = nvim_stateitem_get_m_endpos_lnum(cur_si);
        let m_end_col = nvim_stateitem_get_m_endpos_col(cur_si);

        if si_ends != 0
            && (m_end_lnum < current_lnum
                || (m_end_lnum == current_lnum && m_end_col <= current_col))
        {
            // Check if there is an end pattern group ID to highlight
            let end_idx = nvim_stateitem_get_end_idx(cur_si);
            let eoe_lnum = nvim_stateitem_get_eoe_pos_lnum(cur_si);
            let eoe_col = nvim_stateitem_get_eoe_pos_col(cur_si);

            if end_idx != 0
                && (eoe_lnum > current_lnum || (eoe_lnum == current_lnum && eoe_col > current_col))
            {
                // Switch the item to show the end pattern match
                nvim_stateitem_set_idx(cur_si, end_idx);
                nvim_stateitem_set_end_idx(cur_si, 0);
                nvim_stateitem_set_m_endpos(cur_si, eoe_lnum, eoe_col);
                nvim_stateitem_set_h_endpos(cur_si, eoe_lnum, eoe_col);
                nvim_stateitem_or_flags(cur_si, HL_MATCH);
                let seqnr = nvim_syn_incr_next_seqnr();
                nvim_stateitem_set_seqnr(cur_si, seqnr);
                let si_flags = nvim_stateitem_get_flags(cur_si);
                if si_flags & HL_CONCEALENDS != 0 {
                    nvim_stateitem_or_flags(cur_si, HL_CONCEAL);
                }
                let new_sl = nvim_syn_get_current_state_len();
                update_si_attr(new_sl - 1);

                // nextgroup= should not match in the end pattern
                nvim_syn_set_current_next_list(IdListHandle::null());

                // What matches next may be different now, clear it
                nvim_syn_set_next_match_idx(0);
                nvim_syn_set_next_match_col_val(MAXCOL);
                break;
            }

            // Handle next_list
            let si_next_list = nvim_stateitem_get_next_list(cur_si);
            let si_flags = nvim_stateitem_get_flags(cur_si);
            nvim_syn_set_current_next_list(si_next_list);
            nvim_syn_set_current_next_flags(si_flags);

            // Unless at end of line and no "skipnl" or "skipempty"
            let next_flags = si_flags;
            if (next_flags & (HL_SKIPNL | HL_SKIPEMPTY) == 0)
                && nvim_syn_getcurline_byte_at(current_col) == NUL
            {
                nvim_syn_set_current_next_list(IdListHandle::null());
            }

            // When the ended item has "extend", another item with "keepend"
            // now needs to check for its end.
            let had_extend = si_flags & HL_EXTEND;

            crate::state_ops::rs_syn_pop_current_state();

            if nvim_syn_is_current_state_empty() != 0 {
                break;
            }

            let keepend_level = nvim_syn_get_keepend_level();
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
            let new_idx = nvim_stateitem_get_idx(new_cur);
            let new_flags = nvim_stateitem_get_flags(new_cur);

            if new_idx >= 0
                && nvim_syn_get_pattern_type(new_idx) == SPTYPE_START
                && (new_flags & (HL_MATCH | HL_KEEPEND) == 0)
            {
                crate::region::update_si_end(new_cur, current_col, true);
                check_keepend();
                if (next_flags & HL_HAS_EOL != 0)
                    && nvim_syn_get_keepend_level() < 0
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
