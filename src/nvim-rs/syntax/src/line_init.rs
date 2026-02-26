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
    StateItemHandle, SynStateHandle, HL_EXTEND, HL_KEEPEND, HL_MATCHCONT, SPTYPE_MATCH,
};

// =============================================================================
// FFI declarations for line initialization
// =============================================================================

extern "C" {
    // Current state globals
    fn nvim_syn_get_current_state_len_val() -> c_int;
    fn nvim_syn_get_current_col_val() -> c_int;
    fn nvim_syn_get_current_lnum() -> c_int;
    fn nvim_syn_get_keepend_level_val() -> c_int;
    fn nvim_syn_set_current_finished_val(v: c_int);
    fn nvim_syn_set_current_col_val(col: c_int);
    fn nvim_syn_set_next_match_idx_val(idx: c_int);
    fn nvim_syn_incr_current_line_id_val();
    fn nvim_syn_reset_next_seqnr();
    fn nvim_syn_current_state_nonempty() -> c_int;

    // CUR_STATE accessors for syn_update_ends
    fn nvim_cur_state_get_si_idx(i: c_int) -> c_int;
    fn nvim_cur_state_get_m_endpos_lnum(i: c_int) -> c_int;
    fn nvim_cur_state_get_si_flags(i: c_int) -> c_int;
    fn nvim_cur_state_set_h_startpos_cur(i: c_int);
    fn nvim_cur_state_ptr(i: c_int) -> StateItemHandle;
    fn nvim_syn_get_sptype_at(idx: c_int) -> c_int;

    // validate_current_state
    fn nvim_syn_do_validate_current_state();

    // syn_getcurline / syn_getcurline_len
    fn nvim_syn_do_getcurline() -> *mut c_char;
    fn nvim_syn_do_getcurline_len() -> c_int;

    // chartab save/restore
    fn nvim_syn_block_isk_is_empty() -> c_int;
    fn nvim_syn_buf_chartab_get(dst: *mut c_char);
    fn nvim_syn_buf_chartab_set(src: *const c_char);
    fn nvim_syn_win_chartab_get(dst: *mut c_char);
    fn nvim_syn_win_isk_not_empty() -> c_int;

    // syn_match_linecont (fully wrapped in C)
    fn nvim_syn_exec_linecont(lnum: c_int) -> c_int;

    // clear_syn_state (wraps the real impl in C)
    fn nvim_syn_clear_syn_state(p: SynStateHandle);

    // syn_clear_time (wraps syn_clear_time)
    fn nvim_syn_do_clear_time(st: *mut c_void);

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
    let state_len = nvim_syn_get_current_state_len_val();
    let current_lnum = nvim_syn_get_current_lnum();
    let startofline = startofline != 0;

    if startofline {
        // Check for matches carried over from a previous line.
        for i in 0..state_len {
            let si_idx = nvim_cur_state_get_si_idx(i);
            if si_idx >= 0
                && nvim_syn_get_sptype_at(si_idx) == SPTYPE_MATCH
                && nvim_cur_state_get_m_endpos_lnum(i) < current_lnum
            {
                crate::state_ops::rs_cur_state_set_matchcont(i);
            }
        }
    }

    // Find starting point for the second pass
    let mut i = state_len - 1;
    let keepend_level = nvim_syn_get_keepend_level_val();
    if keepend_level >= 0 {
        while i > keepend_level {
            if (nvim_cur_state_get_si_flags(i) & HL_EXTEND) != 0 {
                break;
            }
            i -= 1;
        }
    }

    // Update end positions for relevant items
    let current_col = nvim_syn_get_current_col_val();
    let mut seen_keepend = false;
    while i < state_len {
        let flags = nvim_cur_state_get_si_flags(i);
        let is_last = i == state_len - 1;
        if (flags & HL_KEEPEND) != 0 || (seen_keepend && !startofline) || (is_last && startofline) {
            nvim_cur_state_set_h_startpos_cur(i);
            if (flags & HL_MATCHCONT) == 0 {
                let si_ptr = nvim_cur_state_ptr(i);
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
    nvim_syn_set_current_finished_val(0);
    nvim_syn_set_current_col_val(0);

    if nvim_syn_current_state_nonempty() != 0 {
        rs_syn_update_ends(1); // startofline = true
        rs_check_state_ends();
    }

    nvim_syn_set_next_match_idx_val(-1);
    nvim_syn_incr_current_line_id_val();
    nvim_syn_reset_next_seqnr();
}

/// Check if the line-continuation pattern matches in line "lnum".
///
/// Replaces static C `syn_match_linecont`.
/// Uses C helper nvim_syn_exec_linecont to keep regmmatch_T in C.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_match_linecont(lnum: c_int) -> c_int {
    nvim_syn_exec_linecont(lnum)
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
/// Replaces static C `clear_syn_state` (via existing nvim_syn_clear_syn_state).
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_syn_state(p: SynStateHandle) {
    nvim_syn_clear_syn_state(p);
}

/// Set itemsize/growsize on current_state garray.
///
/// Replaces static C `validate_current_state`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_current_state() {
    nvim_syn_do_validate_current_state();
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
/// Accesses C structs; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_clear_time(st: *mut c_void) {
    nvim_syn_do_clear_time(st);
}
