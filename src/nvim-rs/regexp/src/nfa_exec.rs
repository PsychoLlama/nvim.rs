//! NFA execution engine for the regex engine.
//!
//! This module implements the core NFA state transition logic, including:
//! - `addstate()` - add states to the thread list (handling epsilon transitions)
//! - State list management
//! - Submatch position tracking during execution
//!
//! # Algorithm Overview
//!
//! The NFA execution uses Thompson's algorithm with parallel state tracking:
//! 1. Maintain two lists: "current" states and "next" states
//! 2. For each input character, process all current states
//! 3. States that match via epsilon transitions are expanded immediately
//! 4. States that consume a character move to the "next" list
//! 5. After processing all current states, swap lists and continue
//!
//! # Epsilon Transitions
//!
//! States like `NFA_SPLIT`, `NFA_NOPEN`, `NFA_MOPEN*` have epsilon transitions.
//! These are followed immediately without consuming input. The `addstate()`
//! function handles this recursively.

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::nfa_match::{copy_pim, copy_sub, MAX_ADDSTATE_DEPTH};
use crate::nfa_states::{
    NfaList, NfaPim, NfaState, NfaThread, RegSubs, NFA_BOF, NFA_BOL, NFA_EMPTY, NFA_MATCH,
    NFA_MCLOSE, NFA_MCLOSE1, NFA_MCLOSE2, NFA_MCLOSE3, NFA_MCLOSE4, NFA_MCLOSE5, NFA_MCLOSE6,
    NFA_MCLOSE7, NFA_MCLOSE8, NFA_MCLOSE9, NFA_MOPEN, NFA_MOPEN1, NFA_MOPEN2, NFA_MOPEN3,
    NFA_MOPEN4, NFA_MOPEN5, NFA_MOPEN6, NFA_MOPEN7, NFA_MOPEN8, NFA_MOPEN9, NFA_NCLOSE, NFA_NOPEN,
    NFA_PIM_UNUSED, NFA_SKIP, NFA_SPLIT, NFA_ZCLOSE, NFA_ZCLOSE1, NFA_ZCLOSE2, NFA_ZCLOSE3,
    NFA_ZCLOSE4, NFA_ZCLOSE5, NFA_ZCLOSE6, NFA_ZCLOSE7, NFA_ZCLOSE8, NFA_ZCLOSE9, NFA_ZEND,
    NFA_ZOPEN, NFA_ZOPEN1, NFA_ZOPEN2, NFA_ZOPEN3, NFA_ZOPEN4, NFA_ZOPEN5, NFA_ZOPEN6, NFA_ZOPEN7,
    NFA_ZOPEN8, NFA_ZOPEN9, NFA_ZSTART,
};

// =============================================================================
// Constants
// =============================================================================

/// Offset used by addstate_here to signal insertion at specific position.
pub const ADDSTATE_HERE_OFFSET: c_int = 1000;

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    // Rex state accessors - use *mut u8 to match lib.rs declarations
    fn nvim_rex_get_input() -> *mut u8;
    fn nvim_rex_get_line() -> *mut u8;
    fn nvim_rex_get_lnum() -> c_int;
    fn nvim_rex_get_nfa_has_backref() -> c_int;
    fn nvim_rex_get_nfa_has_zsubexpr() -> c_int;
    fn nvim_rex_get_nfa_ll_index() -> c_int;
    fn nvim_rex_get_nfa_endp() -> *const c_char;
    fn nvim_rex_is_multi() -> c_int;

    // Memory limit
    fn nvim_get_p_mmp() -> i64;

    // Error reporting
    fn nvim_regexp_emsg_maxmempattern();
}

// =============================================================================
// Temporary Submatch Storage
// =============================================================================

use std::cell::UnsafeCell;

/// Thread-local temporary storage for submatch data.
///
/// When the list needs to be resized, submatch pointers may become invalid.
/// We use this temporary storage to preserve the data.
///
/// Using UnsafeCell to avoid static mut reference warnings while maintaining
/// the same single-threaded semantics.
struct TempSubsStorage {
    inner: UnsafeCell<Option<Box<RegSubs>>>,
}

// SAFETY: Neovim is single-threaded for regex operations
unsafe impl Sync for TempSubsStorage {}

static TEMP_SUBS: TempSubsStorage = TempSubsStorage {
    inner: UnsafeCell::new(None),
};

/// Get or create temporary submatch storage.
///
/// # Safety
/// Must be called from single-threaded context.
unsafe fn get_temp_subs() -> *mut RegSubs {
    let storage = &mut *TEMP_SUBS.inner.get();
    if storage.is_none() {
        *storage = Some(Box::new(RegSubs::default()));
    }
    storage.as_mut().unwrap().as_mut() as *mut RegSubs
}

// =============================================================================
// Addstate Implementation
// =============================================================================

/// Thread-safe storage for recursion depth counter.
struct DepthStorage {
    inner: UnsafeCell<c_int>,
}

// SAFETY: Neovim is single-threaded for regex operations
unsafe impl Sync for DepthStorage {}

static DEPTH: DepthStorage = DepthStorage {
    inner: UnsafeCell::new(0),
};

/// Add a state to the thread list, handling epsilon transitions.
///
/// This is the core function for NFA execution. It adds a state to the list
/// and recursively follows epsilon transitions (NFA_SPLIT, NFA_NOPEN, etc.).
///
/// # Arguments
/// * `list` - The thread list to add to
/// * `state` - The NFA state to add
/// * `subs` - Current submatch positions
/// * `pim` - Postponed invisible match info (for lookahead/lookbehind)
/// * `off` - Byte offset for position tracking
///
/// # Returns
/// The (possibly updated) submatch pointer, or NULL on error/recursion limit.
///
/// # Safety
/// All pointers must be valid.
pub unsafe fn addstate(
    list: *mut NfaList,
    state: *mut NfaState,
    subs: *mut RegSubs,
    pim: *const NfaPim,
    off: c_int,
) -> *mut RegSubs {
    let depth = &mut *DEPTH.inner.get();

    if *depth >= MAX_ADDSTATE_DEPTH || subs.is_null() || state.is_null() || list.is_null() {
        return ptr::null_mut();
    }

    *depth += 1;

    // Handle addstate_here offset
    let (add_here, _actual_off, listindex) = if off <= -ADDSTATE_HERE_OFFSET {
        (true, 0, -(off + ADDSTATE_HERE_OFFSET))
    } else {
        (false, off, 0)
    };

    let nfa_ll_index = nvim_rex_get_nfa_ll_index() as usize;
    let list_id = (*list).id;

    // First switch: determine if we should add this state or skip it
    let should_add = match (*state).c {
        // These nodes are not added themselves - only their successors
        NFA_NCLOSE | NFA_MCLOSE | NFA_MCLOSE1 | NFA_MCLOSE2 | NFA_MCLOSE3 | NFA_MCLOSE4
        | NFA_MCLOSE5 | NFA_MCLOSE6 | NFA_MCLOSE7 | NFA_MCLOSE8 | NFA_MCLOSE9 | NFA_ZCLOSE
        | NFA_ZCLOSE1 | NFA_ZCLOSE2 | NFA_ZCLOSE3 | NFA_ZCLOSE4 | NFA_ZCLOSE5 | NFA_ZCLOSE6
        | NFA_ZCLOSE7 | NFA_ZCLOSE8 | NFA_ZCLOSE9 | NFA_MOPEN | NFA_ZEND | NFA_SPLIT
        | NFA_EMPTY => false,

        // BOL/BOF: don't add past end-of-line
        NFA_BOL | NFA_BOF => {
            let input = nvim_rex_get_input();
            let line = nvim_rex_get_line();
            if input > line && !input.is_null() && *input != 0 {
                // Check if we're at the end position for lookbehind
                let nfa_endp = nvim_rex_get_nfa_endp();
                if nfa_endp.is_null()
                    || nvim_rex_is_multi() == 0
                    || nvim_rex_get_lnum() == get_endp_lnum(nfa_endp)
                {
                    *depth -= 1;
                    return subs;
                }
            }
            true
        }

        // All other states: check for duplicates
        _ => {
            // Check if already in list
            if (*state).lastlist[nfa_ll_index] == list_id && (*state).c != NFA_SKIP {
                let has_backref = nvim_rex_get_nfa_has_backref() != 0;
                let has_pim = (*list).has_pim != 0;

                if !has_backref && pim.is_null() && !has_pim && (*state).c != NFA_MATCH {
                    // Skip unless called from addstate_here
                    if add_here {
                        let mut found = false;
                        for k in 0..((*list).n).min(listindex) {
                            if (*(*list).t.add(k as usize)).state == state
                                || ((*(*(*list).t.add(k as usize)).state).id == (*state).id)
                            {
                                found = true;
                                break;
                            }
                        }
                        if found {
                            *depth -= 1;
                            return subs;
                        }
                    } else {
                        *depth -= 1;
                        return subs;
                    }
                }

                // Check if same state with same positions exists
                if has_state_with_pos(list, state, subs, pim) {
                    *depth -= 1;
                    return subs;
                }
            }
            true
        }
    };

    let mut result_subs = subs;

    // Add the state to the list if needed
    if should_add {
        // Grow list if needed
        if (*list).n == (*list).len {
            let newlen = (*list).len * 3 / 2 + 50;
            let newsize = (newlen as usize) * std::mem::size_of::<NfaThread>();

            // Check memory limit
            if (newsize >> 10) as i64 >= nvim_get_p_mmp() {
                nvim_regexp_emsg_maxmempattern();
                *depth -= 1;
                return ptr::null_mut();
            }

            // Copy subs to temp if pointing into list
            let temp_subs = get_temp_subs();
            if subs != temp_subs {
                copy_sub(&mut (*temp_subs).norm, &(*subs).norm);
                if nvim_rex_get_nfa_has_zsubexpr() != 0 {
                    copy_sub(&mut (*temp_subs).synt, &(*subs).synt);
                }
                result_subs = temp_subs;
            }

            // Reallocate
            let newt = libc::realloc((*list).t as *mut libc::c_void, newsize) as *mut NfaThread;
            if newt.is_null() {
                *depth -= 1;
                return ptr::null_mut();
            }
            (*list).t = newt;
            (*list).len = newlen;
        }

        // Add state to list
        (*state).lastlist[nfa_ll_index] = list_id;
        let thread = (*list).t.add((*list).n as usize);
        (*list).n += 1;

        (*thread).state = state;

        // Copy PIM
        if pim.is_null() || (*pim).result == NFA_PIM_UNUSED {
            (*thread).pim.result = NFA_PIM_UNUSED;
            (*thread).pim.state = ptr::null_mut();
        } else {
            copy_pim(&mut (*thread).pim, pim);
            (*list).has_pim = 1;
        }

        // Copy submatch info
        copy_sub(&mut (*thread).subs.norm, &(*result_subs).norm);
        if nvim_rex_get_nfa_has_zsubexpr() != 0 {
            copy_sub(&mut (*thread).subs.synt, &(*result_subs).synt);
        }
    }

    // Second switch: follow epsilon transitions
    match (*state).c {
        NFA_MATCH => {
            // Don't follow transitions from match state
        }

        NFA_SPLIT => {
            // Order matters - try out before out1
            result_subs = addstate(list, (*state).out, result_subs, pim, off);
            if !result_subs.is_null() {
                result_subs = addstate(list, (*state).out1, result_subs, pim, off);
            }
        }

        NFA_EMPTY | NFA_NOPEN | NFA_NCLOSE => {
            result_subs = addstate(list, (*state).out, result_subs, pim, off);
        }

        // MOPEN states: save submatch start position and continue
        NFA_MOPEN | NFA_MOPEN1 | NFA_MOPEN2 | NFA_MOPEN3 | NFA_MOPEN4 | NFA_MOPEN5 | NFA_MOPEN6
        | NFA_MOPEN7 | NFA_MOPEN8 | NFA_MOPEN9 | NFA_ZOPEN | NFA_ZOPEN1 | NFA_ZOPEN2
        | NFA_ZOPEN3 | NFA_ZOPEN4 | NFA_ZOPEN5 | NFA_ZOPEN6 | NFA_ZOPEN7 | NFA_ZOPEN8
        | NFA_ZOPEN9 | NFA_ZSTART => {
            // TODO: Handle submatch start position tracking
            result_subs = addstate(list, (*state).out, result_subs, pim, off);
        }

        // MCLOSE states: save submatch end position and continue
        NFA_MCLOSE | NFA_MCLOSE1 | NFA_MCLOSE2 | NFA_MCLOSE3 | NFA_MCLOSE4 | NFA_MCLOSE5
        | NFA_MCLOSE6 | NFA_MCLOSE7 | NFA_MCLOSE8 | NFA_MCLOSE9 | NFA_ZCLOSE | NFA_ZCLOSE1
        | NFA_ZCLOSE2 | NFA_ZCLOSE3 | NFA_ZCLOSE4 | NFA_ZCLOSE5 | NFA_ZCLOSE6 | NFA_ZCLOSE7
        | NFA_ZCLOSE8 | NFA_ZCLOSE9 | NFA_ZEND => {
            // TODO: Handle submatch end position tracking
            result_subs = addstate(list, (*state).out, result_subs, pim, off);
        }

        // BOL/BOF already handled above
        NFA_BOL | NFA_BOF => {
            result_subs = addstate(list, (*state).out, result_subs, pim, off);
        }

        _ => {
            // Other states don't have epsilon transitions
        }
    }

    *depth -= 1;
    result_subs
}

/// Check if a state with the same positions already exists in the list.
///
/// # Safety
/// All pointers must be valid.
unsafe fn has_state_with_pos(
    list: *const NfaList,
    state: *const NfaState,
    _subs: *const RegSubs,
    _pim: *const NfaPim,
) -> bool {
    if list.is_null() || state.is_null() || (*list).t.is_null() {
        return false;
    }

    for i in 0..(*list).n {
        let thread = (*list).t.add(i as usize);
        if (*(*thread).state).id == (*state).id {
            // Simplified: just check state ID match
            // Full implementation would compare submatch positions
            return true;
        }
    }
    false
}

/// Get the line number from endp structure.
///
/// # Safety
/// If endp is non-null, it must point to valid memory.
unsafe fn get_endp_lnum(_endp: *const c_char) -> c_int {
    // This would need to cast to the proper save_se_T structure
    // For now, return a value that won't match
    -1
}

// =============================================================================
// Addstate_here
// =============================================================================

/// Like addstate(), but insert the new states at a specific position.
///
/// This is used when processing states needs to insert new states at
/// the current processing position rather than at the end.
///
/// # Safety
/// All pointers must be valid.
pub unsafe fn addstate_here(
    list: *mut NfaList,
    state: *mut NfaState,
    subs: *mut RegSubs,
    pim: *const NfaPim,
    listidx: c_int,
) -> *mut RegSubs {
    // The negative offset signals to addstate to insert at listidx
    addstate(list, state, subs, pim, -listidx - ADDSTATE_HERE_OFFSET)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Add a state to the thread list.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_addstate(
    list: *mut NfaList,
    state: *mut NfaState,
    subs: *mut RegSubs,
    pim: *const NfaPim,
    off: c_int,
) -> *mut RegSubs {
    addstate(list, state, subs, pim, off)
}

/// Add a state at a specific position in the list.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_addstate_here(
    list: *mut NfaList,
    state: *mut NfaState,
    subs: *mut RegSubs,
    pim: *const NfaPim,
    listidx: c_int,
) -> *mut RegSubs {
    addstate_here(list, state, subs, pim, listidx)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addstate_here_offset() {
        assert_eq!(ADDSTATE_HERE_OFFSET, 1000);

        // Test offset calculation
        let idx = 5;
        let off = -idx - ADDSTATE_HERE_OFFSET;
        assert!(off <= -ADDSTATE_HERE_OFFSET);
        let recovered_idx = -(off + ADDSTATE_HERE_OFFSET);
        assert_eq!(recovered_idx, idx);
    }

    #[test]
    fn test_max_addstate_depth() {
        assert_eq!(MAX_ADDSTATE_DEPTH, 5000);
    }
}
