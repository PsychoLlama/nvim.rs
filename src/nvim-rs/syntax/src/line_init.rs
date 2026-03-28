//! Line initialization helpers for syntax highlighting.
//!
//! This module migrates the line-level initialization functions:
//! - `syn_update_ends`: check/update end positions for state stack items
//! - `syn_start_line`: prepare current state for start of a new line
//! - `syn_match_linecont`: check if line-continuation pattern matches
//! - `clear_syn_state`: release extmatch pointers from a synstate
//! - `validate_current_state`: set itemsize/growsize on current_state garray
//! - `syn_getcurline`: get current line from syntax buffer
//! - `syn_getcurline_len`: get current line length
//! - `save_chartab` / `restore_chartab`: save/restore buffer chartab
//! - `syn_clear_time`: zero out a syn_time_T struct

use std::ffi::{c_char, c_int, c_void};

use crate::types::{
    BufStateHandle, ExtMatchHandle, StateItemHandle, SynStateHandle, HL_EXTEND, HL_KEEPEND,
    HL_MATCHCONT, SPTYPE_MATCH, SST_FIX_STATES,
};

// =============================================================================
// FFI declarations for line initialization
// =============================================================================

extern "C" {
    // Current state globals

    fn nvim_syn_get_syn_block() -> crate::types::SynBlockHandle;

    // validate_current_state

    // syn_getcurline / syn_getcurline_len (direct, avoids circularity with rs_syn_getcurline)
    fn nvim_syn_do_getcurline() -> *mut c_char;
    fn nvim_syn_do_getcurline_len() -> c_int;

    // chartab save/restore
    fn nvim_syn_block_isk_is_empty() -> c_int;
    fn nvim_syn_buf_chartab_get(dst: *mut c_char);
    fn nvim_syn_buf_chartab_set(src: *const c_char);
    fn nvim_syn_win_chartab_get(dst: *mut c_char);
    fn nvim_syn_win_isk_not_empty() -> c_int;

    // (nvim_syn_exec_linecont replaced by crate::regexec::syn_exec_linecont)

    // clear_syn_state accessors (Phase 11)
    fn nvim_synstate_ga_clear(state: SynStateHandle);
    fn nvim_syn_unref_extmatch(em: ExtMatchHandle);
    fn nvim_synstate_get_stacksize(state: SynStateHandle) -> c_int;
    fn nvim_synstate_get_bufstate(state: SynStateHandle, idx: c_int) -> BufStateHandle;

    // Already-Rust functions called from syn_update_ends / syn_start_line
    fn rs_update_si_end(sip: StateItemHandle, startcol: c_int, force: c_int);
    fn rs_check_keepend();
    fn rs_check_state_ends();
}

// =============================================================================
// Phase 8: Line initialization (replaces C implementations)
// =============================================================================

/// Check for items in the stack that need their end updated.
///
/// Replaces static C `syn_update_ends`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_update_ends(startofline: c_int) {
    let state_len = crate::statics::CURRENT_STATE.ga_len;
    let current_lnum = crate::statics::CURRENT_LNUM;
    let startofline = startofline != 0;

    if startofline {
        // Check for matches carried over from a previous line.
        for i in 0..state_len {
            let si_idx = (*crate::statics::current_state_item(i).as_ptr()).si_idx;
            if si_idx >= 0
                && {
                    let block = nvim_syn_get_syn_block();
                    let p = crate::statics::syn_item_at(block, si_idx);
                    !p.is_null() && (*p).sp_type as c_int == SPTYPE_MATCH
                }
                && (*crate::statics::current_state_item(i).as_ptr())
                    .si_m_endpos
                    .lnum
                    < current_lnum
            {
                crate::state_ops::rs_cur_state_set_matchcont(i);
            }
        }
    }

    // Find starting point for the second pass
    let mut i = state_len - 1;
    let keepend_level = crate::statics::KEEPEND_LEVEL;
    if keepend_level >= 0 {
        while i > keepend_level {
            if ((*crate::statics::current_state_item(i).as_ptr()).si_flags & HL_EXTEND) != 0 {
                break;
            }
            i -= 1;
        }
    }

    // Update end positions for relevant items
    let current_col = crate::statics::CURRENT_COL;
    let mut seen_keepend = false;
    while i < state_len {
        let flags = (*crate::statics::current_state_item(i).as_ptr()).si_flags;
        let is_last = i == state_len - 1;
        if (flags & HL_KEEPEND) != 0 || (seen_keepend && !startofline) || (is_last && startofline) {
            {
                let _p = crate::statics::current_state_item(i).as_ptr();
                if !_p.is_null() {
                    (*_p).si_h_startpos.col = 0;
                    (*_p).si_h_startpos.lnum = crate::statics::CURRENT_LNUM;
                }
            };
            if (flags & HL_MATCHCONT) == 0 {
                let si_ptr = crate::statics::current_state_item(i);
                if !si_ptr.is_null() {
                    rs_update_si_end(si_ptr, current_col, if !startofline { 1 } else { 0 });
                }
            }
            if !startofline && (flags & HL_KEEPEND) != 0 {
                seen_keepend = true;
            }
        }
        i += 1;
    }
    rs_check_keepend();
}

/// Prepare the current state for the start of a line.
///
/// Replaces static C `syn_start_line`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_start_line() {
    crate::statics::CURRENT_FINISHED = 0;
    crate::statics::CURRENT_COL = 0;

    if !crate::statics::current_state_is_empty() {
        rs_syn_update_ends(1); // startofline = true
        rs_check_state_ends();
    }

    crate::statics::NEXT_MATCH_IDX = -1;
    crate::statics::CURRENT_LINE_ID += 1;
    crate::statics::NEXT_SEQNR = 1;
}

/// Check if the line-continuation pattern matches in line "lnum".
///
/// Replaces static C `syn_match_linecont`.
/// Delegates to Rust regexec module.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_match_linecont(lnum: c_int) -> c_int {
    crate::regexec::syn_exec_linecont(lnum)
}

/// Save buffer chartab, overriding with syn iskeyword if set.
///
/// Replaces static C `save_chartab`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_save_chartab(chartab: *mut c_char) {
    if nvim_syn_block_isk_is_empty() != 0 {
        return;
    }
    nvim_syn_buf_chartab_get(chartab);
    let mut tmp = [0u8; 32];
    nvim_syn_win_chartab_get(tmp.as_mut_ptr() as *mut c_char);
    nvim_syn_buf_chartab_set(tmp.as_ptr() as *const c_char);
}

/// Restore buffer chartab from saved value.
///
/// Replaces static C `restore_chartab`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_restore_chartab(chartab: *const c_char) {
    if nvim_syn_win_isk_not_empty() != 0 {
        nvim_syn_buf_chartab_set(chartab);
    }
}

/// Release extmatch pointers from a synstate entry.
///
/// Replaces static C `clear_syn_state`. Iterates through the bufstate entries
/// and unrefs each extmatch, then clears the growarray if the GA path was used.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_syn_state(p: SynStateHandle) {
    if p.is_null() {
        return;
    }
    let stacksize = nvim_synstate_get_stacksize(p);
    let sst_fix_states = SST_FIX_STATES;
    // Unref all extmatch pointers
    for i in 0..stacksize {
        let bs = nvim_synstate_get_bufstate(p, i);
        if !bs.0.is_null() {
            let em = ExtMatchHandle(unsafe { (*bs.as_ptr()).bs_extmatch as *mut _ });
            nvim_syn_unref_extmatch(em);
        }
    }
    // If the growarray path was used, free the ga_data
    if stacksize > sst_fix_states {
        nvim_synstate_ga_clear(p);
    }
}

/// Set itemsize/growsize on current_state garray.
///
/// Replaces static C `validate_current_state`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_current_state() {
    crate::statics::current_state_validate();
}

/// Get current line from syntax buffer.
///
/// Replaces static C `syn_getcurline`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_getcurline() -> *mut c_char {
    nvim_syn_do_getcurline()
}

/// Get current line length from syntax buffer.
///
/// Replaces static C `syn_getcurline_len`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_getcurline_len() -> c_int {
    nvim_syn_do_getcurline_len()
}

/// Zero out a syn_time_T struct.
///
/// Replaces static C `syn_clear_time`.
///
/// # Safety
/// `st` must be a valid pointer to a syn_time_T struct (or null).
#[no_mangle]
pub unsafe extern "C" fn rs_syn_clear_time(st: *mut c_void) {
    if st.is_null() {
        return;
    }
    let t = &mut *(st as *mut crate::ffi_types::SynTime);
    t.total = 0;
    t.slowest = 0;
    t.count = 0;
    t.match_ = 0;
}
