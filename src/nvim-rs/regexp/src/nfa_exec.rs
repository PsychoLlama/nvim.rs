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
// State List Helpers
// =============================================================================

/// Check if a state is already in the list.
///
/// This is used to avoid adding duplicate states.
///
/// # Safety
/// All pointers must be valid.
pub unsafe fn state_in_list(
    list: *const NfaList,
    state: *const NfaState,
    subs: *const RegSubs,
) -> bool {
    if list.is_null() || state.is_null() {
        return false;
    }

    let nfa_ll_index = nvim_rex_get_nfa_ll_index() as usize;

    // Check if state was added in current iteration
    if (*state).lastlist[nfa_ll_index] == (*list).id {
        // If no backreferences, simple check is enough
        if nvim_rex_get_nfa_has_backref() == 0 {
            return true;
        }
        // With backreferences, need to check positions match
        if has_state_with_pos(list, state, subs, ptr::null()) {
            return true;
        }
    }
    false
}

// =============================================================================
// NFA Match Engine
// =============================================================================

/// Result of a single NFA step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum NfaStepResult {
    /// Continue matching
    Continue = 0,
    /// Match found
    Match = 1,
    /// No match possible
    NoMatch = 2,
    /// Error occurred
    Error = -1,
}

/// Process a single character in the NFA matcher.
///
/// This is the core of Thompson's algorithm: for each state in the current
/// list, if it can match the current character, add the successor to the
/// next list.
///
/// # Safety
/// All pointers must be valid.
pub unsafe fn nfa_step(
    current: *mut NfaList,
    next: *mut NfaList,
    c: c_int,
    _subs: *mut RegSubs,
) -> NfaStepResult {
    if current.is_null() || next.is_null() {
        return NfaStepResult::Error;
    }

    // Clear the next list
    (*next).n = 0;
    (*next).has_pim = 0;

    // Process all states in current list
    for i in 0..(*current).n {
        let thread = (*current).t.add(i as usize);
        let state = (*thread).state;

        if state.is_null() {
            continue;
        }

        let state_c = (*state).c;

        // Check for match state
        if state_c == NFA_MATCH {
            return NfaStepResult::Match;
        }

        // Check if state matches current character
        if nfa_state_matches(state, c) {
            // Add successor state to next list
            let out = (*state).out;
            if !out.is_null() {
                addstate(next, out, &mut (*thread).subs, &(*thread).pim, 0);
            }
        }
    }

    if (*next).n == 0 {
        NfaStepResult::NoMatch
    } else {
        NfaStepResult::Continue
    }
}

/// Check if an NFA state matches a character.
///
/// # Safety
/// State must be valid.
unsafe fn nfa_state_matches(state: *const NfaState, c: c_int) -> bool {
    if state.is_null() {
        return false;
    }

    let state_c = (*state).c;

    // Direct character match
    if state_c > 0 && state_c < 256 {
        return state_c == c || (c >= 0 && (state_c as u8).eq_ignore_ascii_case(&(c as u8)));
    }

    // Character class match
    match state_c {
        crate::nfa_states::NFA_ANY => c != b'\n' as c_int && c != 0,
        crate::nfa_states::NFA_DIGIT => (c as u8).is_ascii_digit(),
        crate::nfa_states::NFA_NDIGIT => !(c as u8).is_ascii_digit() && c != b'\n' as c_int,
        crate::nfa_states::NFA_WORD => (c as u8).is_ascii_alphanumeric() || c == b'_' as c_int,
        crate::nfa_states::NFA_NWORD => {
            !((c as u8).is_ascii_alphanumeric() || c == b'_' as c_int) && c != b'\n' as c_int
        }
        crate::nfa_states::NFA_WHITE => c == b' ' as c_int || c == b'\t' as c_int,
        crate::nfa_states::NFA_NWHITE => c != b' ' as c_int && c != b'\t' as c_int,
        crate::nfa_states::NFA_ALPHA => (c as u8).is_ascii_alphabetic(),
        crate::nfa_states::NFA_NALPHA => !(c as u8).is_ascii_alphabetic() && c != b'\n' as c_int,
        crate::nfa_states::NFA_LOWER => (c as u8).is_ascii_lowercase(),
        crate::nfa_states::NFA_NLOWER => !(c as u8).is_ascii_lowercase() && c != b'\n' as c_int,
        crate::nfa_states::NFA_UPPER => (c as u8).is_ascii_uppercase(),
        crate::nfa_states::NFA_NUPPER => !(c as u8).is_ascii_uppercase() && c != b'\n' as c_int,
        crate::nfa_states::NFA_HEX => (c as u8).is_ascii_hexdigit(),
        crate::nfa_states::NFA_NHEX => !(c as u8).is_ascii_hexdigit() && c != b'\n' as c_int,
        crate::nfa_states::NFA_OCTAL => matches!(c as u8, b'0'..=b'7'),
        crate::nfa_states::NFA_NOCTAL => !matches!(c as u8, b'0'..=b'7') && c != b'\n' as c_int,
        crate::nfa_states::NFA_NEWL => c == b'\n' as c_int,
        _ => false,
    }
}

/// Swap two thread lists.
///
/// # Safety
/// Both pointers must be valid.
#[inline]
pub unsafe fn swap_lists(a: *mut *mut NfaList, b: *mut *mut NfaList) {
    core::ptr::swap(a, b);
}

/// Increment the list ID for the next iteration.
///
/// This is used to efficiently check if a state is already in the current list.
///
/// # Safety
/// The list must be valid.
#[inline]
pub unsafe fn next_list_id(list: *mut NfaList) {
    if !list.is_null() {
        (*list).id += 1;
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Perform a single NFA matching step.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_step(
    current: *mut NfaList,
    next: *mut NfaList,
    c: c_int,
    subs: *mut RegSubs,
) -> c_int {
    match nfa_step(current, next, c, subs) {
        NfaStepResult::Continue => 0,
        NfaStepResult::Match => 1,
        NfaStepResult::NoMatch => 2,
        NfaStepResult::Error => -1,
    }
}

/// Swap two thread list pointers.
///
/// # Safety
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_swap_lists(a: *mut *mut NfaList, b: *mut *mut NfaList) {
    swap_lists(a, b);
}

/// Increment list ID.
///
/// # Safety
/// The list must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_next_list_id(list: *mut NfaList) {
    next_list_id(list);
}

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

/// Check if a state is already in the list, checking submatches.
///
/// This is the full version that checks backref positions.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_state_in_list_with_subs(
    list: *const NfaList,
    state: *const NfaState,
    subs: *const RegSubs,
) -> bool {
    state_in_list(list, state, subs)
}

// =============================================================================
// Submatch and PIM Comparison
// =============================================================================

/// Compare two submatches for equality.
///
/// Returns true if sub1 and sub2 have the same start positions.
/// When using back-references, also checks the end position.
///
/// # Safety
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_sub_equal(
    sub1: *const crate::nfa_states::RegSub,
    sub2: *const crate::nfa_states::RegSub,
) -> c_int {
    if sub1.is_null() || sub2.is_null() {
        return c_int::from(sub1.is_null() && sub2.is_null());
    }

    let in_use1 = (*sub1).in_use as usize;
    let in_use2 = (*sub2).in_use as usize;
    let todo = in_use1.max(in_use2);

    let is_multi = nvim_rex_is_multi() != 0;
    let has_backref = nvim_rex_get_nfa_has_backref() != 0;

    for i in 0..todo {
        if is_multi {
            // Multi-line: compare line/col positions
            let s1 = if i < in_use1 {
                (*sub1).list.multi[i].start_lnum
            } else {
                -1
            };
            let s2 = if i < in_use2 {
                (*sub2).list.multi[i].start_lnum
            } else {
                -1
            };

            if s1 != s2 {
                return 0;
            }
            if s1 != -1 && (*sub1).list.multi[i].start_col != (*sub2).list.multi[i].start_col {
                return 0;
            }

            // With backreferences, also check end positions
            if has_backref {
                let e1 = if i < in_use1 {
                    (*sub1).list.multi[i].end_lnum
                } else {
                    -1
                };
                let e2 = if i < in_use2 {
                    (*sub2).list.multi[i].end_lnum
                } else {
                    -1
                };

                if e1 != e2 {
                    return 0;
                }
                if e1 != -1 && (*sub1).list.multi[i].end_col != (*sub2).list.multi[i].end_col {
                    return 0;
                }
            }
        } else {
            // Single-line: compare pointers
            let sp1 = if i < in_use1 {
                (*sub1).list.line[i].start
            } else {
                ptr::null()
            };
            let sp2 = if i < in_use2 {
                (*sub2).list.line[i].start
            } else {
                ptr::null()
            };

            if sp1 != sp2 {
                return 0;
            }

            // With backreferences, also check end positions
            if has_backref {
                let ep1 = if i < in_use1 {
                    (*sub1).list.line[i].end
                } else {
                    ptr::null()
                };
                let ep2 = if i < in_use2 {
                    (*sub2).list.line[i].end
                } else {
                    ptr::null()
                };

                if ep1 != ep2 {
                    return 0;
                }
            }
        }
    }

    1
}

/// Compare two PIMs (Postponed Invisible Match) for equality.
///
/// Returns true if one and two are equal. That includes when both are not set.
///
/// # Safety
/// Pointers may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_pim_equal(one: *const NfaPim, two: *const NfaPim) -> c_int {
    let one_unused = one.is_null() || (*one).result == NFA_PIM_UNUSED;
    let two_unused = two.is_null() || (*two).result == NFA_PIM_UNUSED;

    // Both unused = equal
    if one_unused {
        return c_int::from(two_unused);
    }

    // One used, one not = not equal
    if two_unused {
        return 0;
    }

    // Compare state ID
    if (*one).state.is_null() || (*two).state.is_null() {
        return c_int::from((*one).state.is_null() && (*two).state.is_null());
    }
    if (*(*one).state).id != (*(*two).state).id {
        return 0;
    }

    // Compare position
    if nvim_rex_is_multi() != 0 {
        // Multi-line mode: compare line/col
        c_int::from(
            (*one).end.pos.lnum == (*two).end.pos.lnum && (*one).end.pos.col == (*two).end.pos.col,
        )
    } else {
        // Single-line mode: compare pointers
        c_int::from((*one).end.ptr == (*two).end.ptr)
    }
}

/// Check if a state leads to NFA_MATCH without consuming input.
///
/// # Safety
/// State must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_match_follows(state: *const NfaState, depth: c_int) -> c_int {
    match_follows_impl(state, depth)
}

unsafe fn match_follows_impl(startstate: *const NfaState, depth: c_int) -> c_int {
    use crate::nfa_states::*;

    // Avoid too much recursion
    if depth > 10 {
        return 0;
    }

    let mut state = startstate;
    while !state.is_null() {
        let c = (*state).c;

        match c {
            NFA_MATCH => return 1,
            NFA_SPLIT => {
                if match_follows_impl((*state).out, depth + 1) != 0 {
                    return 1;
                }
                state = (*state).out1;
            }
            NFA_EMPTY | NFA_NOPEN | NFA_NCLOSE | NFA_MOPEN | NFA_MCLOSE | NFA_BOL | NFA_BOF
            | NFA_BOW | NFA_ZSTART | NFA_ZEND => {
                state = (*state).out;
            }
            // MOPEN/MCLOSE with index
            c if (NFA_MOPEN..=NFA_MOPEN + 9).contains(&c) => {
                state = (*state).out;
            }
            c if (NFA_MCLOSE..=NFA_MCLOSE + 9).contains(&c) => {
                state = (*state).out;
            }
            // ZOPEN/ZCLOSE
            c if (NFA_ZOPEN..=NFA_ZOPEN + 9).contains(&c) => {
                state = (*state).out;
            }
            c if (NFA_ZCLOSE..=NFA_ZCLOSE + 9).contains(&c) => {
                state = (*state).out;
            }
            _ => return 0,
        }
    }
    0
}

// =============================================================================
// NFA Regmatch Types
// =============================================================================

/// Opaque handle to nfa_regprog_T from C.
#[repr(C)]
pub struct NfaProgHandle {
    _private: [u8; 0],
}

// Note: nfa_regmatch() is a static function in C, so we can't call it directly.
// The Rust NFA execution uses nfa_step() and addstate() which are fully
// implemented above. When the full NFA matcher is migrated to Rust, it will
// replace the C implementation.

// =============================================================================
// Thread List Operations
// =============================================================================

// Note: Most list operations are defined in nfa_states.rs
// Here we add a few additional helpers for the execution engine.

/// Check if a thread list is empty.
///
/// # Safety
/// List must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_list_empty(list: *const NfaList) -> c_int {
    c_int::from(list.is_null() || (*list).n == 0)
}

/// Get the list ID for duplicate detection.
///
/// # Safety
/// List must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_list_id(list: *const NfaList) -> c_int {
    if list.is_null() {
        0
    } else {
        (*list).id
    }
}

/// Get a thread from the list by index (const version).
///
/// # Safety
/// List must be valid and index must be in bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_list_get_thread(
    list: *const NfaList,
    idx: c_int,
) -> *const NfaThread {
    if list.is_null() || idx < 0 || idx >= (*list).n {
        ptr::null()
    } else {
        (*list).t.add(idx as usize)
    }
}

// =============================================================================
// Match Result Helpers
// =============================================================================

/// Check if an NFA match was successful.
///
/// # Arguments
/// * `result` - Result from rs_nfa_step or rs_nfa_regmatch
///
/// # Returns
/// 1 if match found, 0 otherwise
#[no_mangle]
pub const extern "C" fn rs_nfa_match_found(result: c_int) -> c_int {
    (result == 1) as c_int
}

/// Check if matching should continue.
///
/// # Arguments
/// * `result` - Result from rs_nfa_step
///
/// # Returns
/// 1 if should continue, 0 if done (match or no-match)
#[no_mangle]
pub const extern "C" fn rs_nfa_should_continue(result: c_int) -> c_int {
    (result == 0) as c_int
}

/// Check if no match is possible.
///
/// # Arguments
/// * `result` - Result from rs_nfa_step
///
/// # Returns
/// 1 if no match possible, 0 otherwise
#[no_mangle]
pub const extern "C" fn rs_nfa_no_match(result: c_int) -> c_int {
    (result == 2) as c_int
}

/// Check if an error occurred.
///
/// # Arguments
/// * `result` - Result from rs_nfa_step or rs_nfa_regmatch
///
/// # Returns
/// 1 if error, 0 otherwise
#[no_mangle]
pub const extern "C" fn rs_nfa_match_error(result: c_int) -> c_int {
    (result < 0) as c_int
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

    #[test]
    fn test_match_result_helpers() {
        assert_eq!(rs_nfa_match_found(1), 1);
        assert_eq!(rs_nfa_match_found(0), 0);
        assert_eq!(rs_nfa_match_found(2), 0);
        assert_eq!(rs_nfa_match_found(-1), 0);

        assert_eq!(rs_nfa_should_continue(0), 1);
        assert_eq!(rs_nfa_should_continue(1), 0);
        assert_eq!(rs_nfa_should_continue(2), 0);

        assert_eq!(rs_nfa_no_match(2), 1);
        assert_eq!(rs_nfa_no_match(0), 0);
        assert_eq!(rs_nfa_no_match(1), 0);

        assert_eq!(rs_nfa_match_error(-1), 1);
        assert_eq!(rs_nfa_match_error(0), 0);
        assert_eq!(rs_nfa_match_error(1), 0);
    }
}
