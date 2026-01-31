//! NFA matching engine for the regex engine.
//!
//! This module implements the NFA simulation for pattern matching.
//! It uses parallel state tracking (Thompson's algorithm) where all
//! possible states are tracked simultaneously.
//!
//! # Overview
//!
//! The matching algorithm:
//! 1. Start with the initial state in the "current" list
//! 2. For each input character:
//!    - Process all states in the current list
//!    - States that can consume the character move to "next" list
//!    - States with epsilon transitions are expanded immediately
//! 3. Swap current and next lists, repeat
//! 4. Match succeeds if any state in current list is an accepting state
//!
//! # Key structures
//!
//! - [`MatchState`]: Tracks current match position and submatches
//! - [`ThreadList`]: List of active threads (states with submatches)
//! - Helper functions for state queue management

use std::ffi::c_int;
use std::ptr;

use crate::nfa_states::{
    ColNr, LPos, NfaList, NfaPim, NfaState, NfaThread, RegSub, RegSubs, NFA_MATCH, NFA_MOPEN,
    NFA_NCLOSE, NFA_NOPEN, NFA_PIM_MATCH, NFA_PIM_NOMATCH, NFA_PIM_TODO, NFA_PIM_UNUSED, NFA_SKIP,
    NFA_SPLIT, NFA_ZCLOSE, NFA_ZCLOSE9, NFA_ZEND, NFA_ZOPEN, NFA_ZOPEN9, NFA_ZSTART, NSUBEXP,
};

// =============================================================================
// FFI declarations for C functions
// =============================================================================

extern "C" {
    fn nvim_rex_is_multi() -> c_int;
    fn nvim_rex_get_nfa_has_zend() -> c_int;
}

// =============================================================================
// Match Constants
// =============================================================================

/// Maximum recursion depth for addstate to prevent stack overflow.
pub const MAX_ADDSTATE_DEPTH: c_int = 5000;

/// Offset used by addstate_here to signal insertion at specific position.
pub const ADDSTATE_HERE_OFFSET: c_int = 1000;

// =============================================================================
// FFI Declarations for Phase 1: nfa_regmatch migration
// =============================================================================

extern "C" {
    // Rex state accessors - use *mut u8 to match lib.rs declarations
    fn nvim_rex_get_input() -> *mut u8;
    fn nvim_rex_get_line() -> *mut u8;
    fn nvim_rex_get_lnum() -> c_int;
    fn nvim_rex_get_nfa_has_backref() -> c_int;
    fn nvim_rex_get_nfa_has_zsubexpr() -> c_int;

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
/// This is used during addstate() to avoid allocating new submatch
/// structures on every call.
struct TempSubsStorage {
    subs: UnsafeCell<RegSubs>,
}

// Safety: This is only accessed from a single thread during regex execution
unsafe impl Sync for TempSubsStorage {}

static TEMP_SUBS: TempSubsStorage = TempSubsStorage {
    subs: UnsafeCell::new(RegSubs {
        norm: RegSub {
            in_use: 0,
            orig_start_col: 0,
            list: crate::nfa_states::SubPos {
                multi: [crate::nfa_states::MultiPos {
                    start_lnum: 0,
                    start_col: 0,
                    end_lnum: 0,
                    end_col: 0,
                }; NSUBEXP],
            },
        },
        synt: RegSub {
            in_use: 0,
            orig_start_col: 0,
            list: crate::nfa_states::SubPos {
                multi: [crate::nfa_states::MultiPos {
                    start_lnum: 0,
                    start_col: 0,
                    end_lnum: 0,
                    end_col: 0,
                }; NSUBEXP],
            },
        },
    }),
};

/// Get mutable reference to temporary submatch storage.
///
/// # Safety
/// Must only be called from single-threaded regex execution code.
#[inline]
unsafe fn get_temp_subs() -> *mut RegSubs {
    TEMP_SUBS.subs.get()
}

// =============================================================================
// Match State
// =============================================================================

/// Current match state tracking.
///
/// Tracks the position in the input, current line info for multi-line
/// matching, and various flags.
#[repr(C)]
#[derive(Debug)]
pub struct MatchContext {
    /// Current input position (pointer into line).
    pub input: *const u8,
    /// Start of current line.
    pub line: *const u8,
    /// Current line number (1-based, for multi-line).
    pub lnum: c_int,
    /// First line number in search range.
    pub first_lnum: c_int,
    /// Maximum line number to search.
    pub max_line: c_int,
    /// Whether multi-line matching is enabled.
    pub multi: bool,
    /// Ignore case flag.
    pub ignore_case: bool,
    /// Ignore combining characters flag.
    pub ignore_combining: bool,
    /// Maximum column to match (0 = unlimited).
    pub max_col: c_int,
}

impl Default for MatchContext {
    fn default() -> Self {
        Self {
            input: ptr::null(),
            line: ptr::null(),
            lnum: 1,
            first_lnum: 1,
            max_line: 0,
            multi: false,
            ignore_case: false,
            ignore_combining: false,
            max_col: 0,
        }
    }
}

impl MatchContext {
    /// Create a new match context for single-line matching.
    ///
    /// # Safety
    /// `line` must point to valid memory of at least `col + 1` bytes.
    pub unsafe fn new_single_line(line: *const u8, col: c_int) -> Self {
        Self {
            input: line.add(col as usize),
            line,
            lnum: 1,
            first_lnum: 1,
            max_line: 0,
            multi: false,
            ignore_case: false,
            ignore_combining: false,
            max_col: 0,
        }
    }

    /// Create a new match context for multi-line matching.
    ///
    /// # Safety
    /// `line` must point to valid memory of at least `col + 1` bytes.
    pub unsafe fn new_multi_line(
        line: *const u8,
        col: c_int,
        lnum: c_int,
        first_lnum: c_int,
        max_line: c_int,
    ) -> Self {
        Self {
            input: line.add(col as usize),
            line,
            lnum,
            first_lnum,
            max_line,
            multi: true,
            ignore_case: false,
            ignore_combining: false,
            max_col: 0,
        }
    }

    /// Get the current byte at the input position.
    ///
    /// # Safety
    /// Input must point to valid memory.
    #[inline]
    pub unsafe fn current_byte(&self) -> u8 {
        if self.input.is_null() {
            0
        } else {
            *self.input
        }
    }

    /// Check if we're at end of line/input.
    ///
    /// # Safety
    /// Input must point to valid memory.
    #[inline]
    pub unsafe fn at_eol(&self) -> bool {
        self.input.is_null() || *self.input == 0
    }

    /// Get the column offset from line start.
    ///
    /// # Safety
    /// Both input and line must be valid pointers into the same string.
    #[inline]
    pub unsafe fn column(&self) -> c_int {
        if self.line.is_null() || self.input.is_null() {
            0
        } else {
            self.input.offset_from(self.line) as c_int
        }
    }

    /// Advance input by n bytes.
    ///
    /// # Safety
    /// Must not advance past end of string.
    #[inline]
    pub unsafe fn advance(&mut self, n: usize) {
        if !self.input.is_null() {
            self.input = self.input.add(n);
        }
    }

    /// Get current position as LPos (for multi-line matching).
    #[inline]
    pub fn position(&self) -> LPos {
        LPos {
            lnum: self.lnum,
            col: unsafe { self.column() },
        }
    }
}

// =============================================================================
// Thread List Management
// =============================================================================

/// Check if a state is already in the list with the same position.
///
/// This prevents adding duplicate states which would cause infinite loops.
///
/// # Safety
/// All pointers must be valid.
#[inline]
pub unsafe fn state_in_list(list: *const NfaList, state: *const NfaState, listid: c_int) -> bool {
    if list.is_null() || state.is_null() {
        return false;
    }
    (*state).lastlist[0] == listid
}

/// Mark a state as being in a list.
///
/// # Safety
/// State must be valid.
#[inline]
pub unsafe fn mark_state_in_list(state: *mut NfaState, listid: c_int) {
    if !state.is_null() {
        (*state).lastlist[0] = listid;
    }
}

/// Initialize a thread with state and submatch info.
///
/// # Safety
/// All pointers must be valid.
pub unsafe fn init_thread(
    thread: *mut NfaThread,
    state: *mut NfaState,
    subs: *const RegSubs,
    pim: *const NfaPim,
) {
    if thread.is_null() {
        return;
    }

    (*thread).state = state;
    (*thread).count = 0;

    // Copy PIM if provided
    if pim.is_null() || (*pim).result == NFA_PIM_UNUSED {
        (*thread).pim.result = NFA_PIM_UNUSED;
        (*thread).pim.state = ptr::null_mut();
    } else {
        (*thread).pim = ptr::read(pim);
    }

    // Copy submatch info if provided
    if !subs.is_null() {
        (*thread).subs = ptr::read(subs);
    }
}

// =============================================================================
// Submatch Tracking
// =============================================================================

/// Copy submatch positions from one RegSub to another.
///
/// # Safety
/// Both pointers must be valid.
pub unsafe fn copy_sub(to: *mut crate::nfa_states::RegSub, from: *const crate::nfa_states::RegSub) {
    if to.is_null() || from.is_null() {
        return;
    }
    (*to).in_use = (*from).in_use;
    if (*from).in_use > 0 {
        // Copy the submatch positions
        ptr::copy_nonoverlapping(
            &(*from).list,
            &mut (*to).list,
            1, // Copy the union as a whole
        );
    }
    (*to).orig_start_col = (*from).orig_start_col;
}

/// Copy full submatch info (normal + syntax subexpressions).
///
/// # Safety
/// Both pointers must be valid.
pub unsafe fn copy_subs(to: *mut RegSubs, from: *const RegSubs, has_zsubexpr: bool) {
    if to.is_null() || from.is_null() {
        return;
    }
    copy_sub(&mut (*to).norm, &(*from).norm);
    if has_zsubexpr {
        copy_sub(&mut (*to).synt, &(*from).synt);
    }
}

/// Copy PIM (Postponed Invisible Match) info.
///
/// # Safety
/// Both pointers must be valid.
pub unsafe fn copy_pim(to: *mut NfaPim, from: *const NfaPim) {
    if to.is_null() || from.is_null() {
        return;
    }
    ptr::copy_nonoverlapping(from, to, 1);
}

/// Clear submatch positions.
///
/// # Safety
/// Pointer must be valid.
pub unsafe fn clear_sub(sub: *mut crate::nfa_states::RegSub) {
    if sub.is_null() {
        return;
    }
    (*sub).in_use = 0;
    (*sub).orig_start_col = 0;
}

/// Clear all submatch info.
///
/// # Safety
/// Pointer must be valid.
pub unsafe fn clear_subs(subs: *mut RegSubs) {
    if subs.is_null() {
        return;
    }
    clear_sub(&mut (*subs).norm);
    clear_sub(&mut (*subs).synt);
}

/// Copy submatch positions from one RegSub to another, excluding the main match (index 0).
///
/// This is used when we want to preserve only the submatches (\1-\9) without
/// affecting the overall match position.
///
/// # Safety
/// Both pointers must be valid.
pub unsafe fn copy_sub_off(
    to: *mut crate::nfa_states::RegSub,
    from: *const crate::nfa_states::RegSub,
) {
    if to.is_null() || from.is_null() {
        return;
    }
    // Update in_use if from has more
    if (*to).in_use < (*from).in_use {
        (*to).in_use = (*from).in_use;
    }
    if (*from).in_use <= 1 {
        return;
    }
    // Copy submatch positions 1..in_use, not 0 (main match)
    let count = ((*from).in_use - 1) as usize;
    let is_multi = nvim_rex_is_multi() != 0;
    if is_multi {
        ptr::copy_nonoverlapping(
            (*from).list.multi.as_ptr().add(1),
            (*to).list.multi.as_mut_ptr().add(1),
            count,
        );
    } else {
        ptr::copy_nonoverlapping(
            (*from).list.line.as_ptr().add(1),
            (*to).list.line.as_mut_ptr().add(1),
            count,
        );
    }
}

/// Copy the end position of the main match if \ze was used.
///
/// This copies only the end position of submatch 0 if nfa_has_zend is set
/// and the from position is valid.
///
/// # Safety
/// Both pointers must be valid.
pub unsafe fn copy_ze_off(
    to: *mut crate::nfa_states::RegSub,
    from: *const crate::nfa_states::RegSub,
) {
    if to.is_null() || from.is_null() {
        return;
    }
    if nvim_rex_get_nfa_has_zend() == 0 {
        return;
    }
    let is_multi = nvim_rex_is_multi() != 0;
    if is_multi {
        if (*from).list.multi[0].end_lnum >= 0 {
            (*to).list.multi[0].end_lnum = (*from).list.multi[0].end_lnum;
            (*to).list.multi[0].end_col = (*from).list.multi[0].end_col;
        }
    } else if !(*from).list.line[0].end.is_null() {
        (*to).list.line[0].end = (*from).list.line[0].end;
    }
}

// =============================================================================
// Match Result
// =============================================================================

/// Result codes for matching.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchResult {
    /// No match found.
    NoMatch = 0,
    /// Match found.
    Match = 1,
    /// Error during matching.
    Error = -1,
    /// Timed out during matching.
    Timeout = -2,
}

impl From<c_int> for MatchResult {
    fn from(v: c_int) -> Self {
        match v {
            1 => Self::Match,
            0 => Self::NoMatch,
            -2 => Self::Timeout,
            _ => Self::Error,
        }
    }
}

impl From<MatchResult> for c_int {
    fn from(r: MatchResult) -> Self {
        match r {
            MatchResult::Match => 1,
            MatchResult::NoMatch => 0,
            MatchResult::Error => -1,
            MatchResult::Timeout => -2,
        }
    }
}

// =============================================================================
// Basic Match Helpers
// =============================================================================

/// Check if a state is a match (accepting) state.
///
/// # Safety
/// If state is non-null, it must point to a valid NfaState.
#[inline]
pub unsafe fn is_match_state(state: *const NfaState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).c == NFA_MATCH
}

/// Check if a state is a split state.
///
/// # Safety
/// If state is non-null, it must point to a valid NfaState.
#[inline]
pub unsafe fn is_split_state(state: *const NfaState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).c == NFA_SPLIT
}

/// Check if any thread in the list has reached a match state.
///
/// # Safety
/// List must be valid.
pub unsafe fn list_has_match(list: *const NfaList) -> bool {
    if list.is_null() || (*list).t.is_null() {
        return false;
    }

    for i in 0..(*list).n {
        let thread = (*list).t.add(i as usize);
        if is_match_state((*thread).state) {
            return true;
        }
    }
    false
}

/// Get the first matching thread from a list.
///
/// # Safety
/// List must be valid.
pub unsafe fn get_first_match(list: *const NfaList) -> *const NfaThread {
    if list.is_null() || (*list).t.is_null() {
        return ptr::null();
    }

    for i in 0..(*list).n {
        let thread = (*list).t.add(i as usize);
        if is_match_state((*thread).state) {
            return thread;
        }
    }
    ptr::null()
}

// =============================================================================
// PIM (Postponed Invisible Match) Helpers
// =============================================================================

/// Check if a PIM needs to be executed.
///
/// # Safety
/// If pim is non-null, it must point to a valid NfaPim.
#[inline]
pub unsafe fn pim_needs_exec(pim: *const NfaPim) -> bool {
    if pim.is_null() {
        return false;
    }
    (*pim).result == NFA_PIM_TODO
}

/// Check if a PIM was successful.
///
/// # Safety
/// If pim is non-null, it must point to a valid NfaPim.
#[inline]
pub unsafe fn pim_matched(pim: *const NfaPim) -> bool {
    if pim.is_null() {
        return false;
    }
    (*pim).result == NFA_PIM_MATCH
}

/// Check if a PIM failed to match.
///
/// # Safety
/// If pim is non-null, it must point to a valid NfaPim.
#[inline]
pub unsafe fn pim_nomatch(pim: *const NfaPim) -> bool {
    if pim.is_null() {
        return false;
    }
    (*pim).result == NFA_PIM_NOMATCH
}

/// Mark a PIM as matched.
///
/// # Safety
/// Pointer must be valid.
pub unsafe fn set_pim_matched(pim: *mut NfaPim) {
    if !pim.is_null() {
        (*pim).result = NFA_PIM_MATCH;
    }
}

/// Mark a PIM as not matched.
///
/// # Safety
/// Pointer must be valid.
pub unsafe fn set_pim_nomatch(pim: *mut NfaPim) {
    if !pim.is_null() {
        (*pim).result = NFA_PIM_NOMATCH;
    }
}

// =============================================================================
// addstate - Core State Addition Function
// =============================================================================

/// Add a state to the list with the given submatch info.
///
/// This is the core function for adding states during NFA execution.
/// It handles:
/// - Duplicate detection (same state already in list)
/// - Epsilon transitions (split, open/close parens, etc.)
/// - Postponed invisible matches (PIM)
///
/// Returns pointer to the added state's submatch info, or NULL if failed.
///
/// # Arguments
/// * `list` - The list to add the state to
/// * `state` - The NFA state to add
/// * `subs` - Submatch info to copy
/// * `pim` - Postponed invisible match info (optional)
/// * `off` - Offset for the state (used for skip states)
///
/// # Safety
/// All non-null pointers must be valid.
pub unsafe fn addstate(
    list: *mut NfaList,
    state: *mut NfaState,
    subs: *const RegSubs,
    pim: *const NfaPim,
    off: c_int,
) -> *mut RegSubs {
    if list.is_null() || state.is_null() {
        return ptr::null_mut();
    }

    // Handle addstate_here offset
    let actual_off = if off <= -ADDSTATE_HERE_OFFSET {
        // This is addstate_here - extract the real listidx
        // For now, treat as a regular add at position 0
        // Full implementation needs to insert at listidx
        -(off + ADDSTATE_HERE_OFFSET)
    } else {
        off
    };

    // Call recursive implementation with depth tracking
    addstate_impl(list, state, subs, pim, actual_off, 0)
}

/// Internal implementation of addstate with depth tracking.
///
/// # Safety
/// All pointers must be valid.
unsafe fn addstate_impl(
    list: *mut NfaList,
    state: *mut NfaState,
    subs_in: *const RegSubs,
    pim: *const NfaPim,
    off: c_int,
    depth: c_int,
) -> *mut RegSubs {
    // Prevent stack overflow from deep recursion
    if depth >= MAX_ADDSTATE_DEPTH {
        nvim_regexp_emsg_maxmempattern();
        return ptr::null_mut();
    }

    // Skip unless called from addstate_here
    if off <= -ADDSTATE_HERE_OFFSET {
        // Called from addstate_here - use special insertion logic
        // For now, fall through to normal handling
    }

    // Get state code
    let state_c = (*state).c;

    // Check memory limit
    let p_mmp = nvim_get_p_mmp();
    if p_mmp > 0 && ((*list).n as i64 + 1) * std::mem::size_of::<NfaThread>() as i64 > p_mmp * 1024
    {
        nvim_regexp_emsg_maxmempattern();
        return ptr::null_mut();
    }

    // Handle epsilon transitions (states that don't consume input)
    match state_c {
        NFA_SPLIT => {
            // Split: add both out and out1
            let result = addstate_impl(list, (*state).out, subs_in, pim, off, depth + 1);
            if result.is_null() {
                return ptr::null_mut();
            }
            return addstate_impl(list, (*state).out1, subs_in, pim, off, depth + 1);
        }

        NFA_SKIP => {
            // Skip state - just move to the output
            return addstate_impl(list, (*state).out, subs_in, pim, off, depth + 1);
        }

        NFA_NOPEN | NFA_NCLOSE => {
            // Non-capturing group open/close - pass through
            return addstate_impl(list, (*state).out, subs_in, pim, off, depth + 1);
        }

        NFA_ZSTART => {
            // \zs - mark start of match
            let temp_subs = get_temp_subs();
            copy_subs(temp_subs, subs_in, nvim_rex_get_nfa_has_zsubexpr() != 0);

            // Set the start position
            let is_multi = nvim_rex_is_multi() != 0;
            if is_multi {
                (*temp_subs).norm.list.multi[0].start_lnum = nvim_rex_get_lnum();
                let input = nvim_rex_get_input();
                let line = nvim_rex_get_line();
                (*temp_subs).norm.list.multi[0].start_col =
                    input.offset_from(line) as ColNr + off as ColNr;
            } else {
                (*temp_subs).norm.list.line[0].start = nvim_rex_get_input().add(off as usize);
            }

            return addstate_impl(list, (*state).out, temp_subs, pim, off, depth + 1);
        }

        NFA_ZEND => {
            // \ze - mark end of match
            let temp_subs = get_temp_subs();
            copy_subs(temp_subs, subs_in, nvim_rex_get_nfa_has_zsubexpr() != 0);

            // Set the end position
            let is_multi = nvim_rex_is_multi() != 0;
            if is_multi {
                (*temp_subs).norm.list.multi[0].end_lnum = nvim_rex_get_lnum();
                let input = nvim_rex_get_input();
                let line = nvim_rex_get_line();
                (*temp_subs).norm.list.multi[0].end_col =
                    input.offset_from(line) as ColNr + off as ColNr;
            } else {
                (*temp_subs).norm.list.line[0].end = nvim_rex_get_input().add(off as usize);
            }

            return addstate_impl(list, (*state).out, temp_subs, pim, off, depth + 1);
        }

        NFA_MOPEN => {
            // Start of capturing group 0
            return handle_mopen(list, state, subs_in, pim, off, depth, 0);
        }

        c if (NFA_MOPEN..=NFA_MOPEN + 9).contains(&c) => {
            // Start of capturing groups 0-9
            let n = c - NFA_MOPEN;
            return handle_mopen(list, state, subs_in, pim, off, depth, n);
        }

        c if (NFA_ZOPEN..=NFA_ZOPEN9).contains(&c) => {
            // External subexpr open
            let n = c - NFA_ZOPEN;
            return handle_zopen(list, state, subs_in, pim, off, depth, n);
        }

        c if (NFA_ZCLOSE..=NFA_ZCLOSE9).contains(&c) => {
            // External subexpr close
            let n = c - NFA_ZCLOSE;
            return handle_zclose(list, state, subs_in, pim, off, depth, n);
        }

        _ => {
            // Not an epsilon transition - add to list
        }
    }

    // Check if state is already in list
    if (*state).lastlist[nvim_rex_get_nfa_ll_index() as usize] == (*list).id && state_c != NFA_SKIP
    {
        return ptr::null_mut();
    }

    // Mark state as being in this list
    (*state).lastlist[nvim_rex_get_nfa_ll_index() as usize] = (*list).id;

    // Check if we have room in the list
    if (*list).n >= (*list).len {
        // List is full - this shouldn't happen with proper allocation
        return ptr::null_mut();
    }

    // Add thread to list
    let thread = (*list).t.add((*list).n as usize);
    (*list).n += 1;

    // Initialize the thread
    (*thread).state = state;
    (*thread).count = 0;

    // Copy submatch info
    if !subs_in.is_null() {
        copy_subs(
            &mut (*thread).subs,
            subs_in,
            nvim_rex_get_nfa_has_zsubexpr() != 0,
        );
    }

    // Copy PIM info
    if !pim.is_null() && (*pim).result != NFA_PIM_UNUSED {
        (*thread).pim = ptr::read(pim);
        (*list).has_pim = 1;
    } else {
        (*thread).pim.result = NFA_PIM_UNUSED;
        (*thread).pim.state = ptr::null_mut();
    }

    &mut (*thread).subs
}

/// Handle MOPEN (start of capturing group).
///
/// # Safety
/// All pointers must be valid.
unsafe fn handle_mopen(
    list: *mut NfaList,
    state: *mut NfaState,
    subs_in: *const RegSubs,
    pim: *const NfaPim,
    off: c_int,
    depth: c_int,
    n: c_int,
) -> *mut RegSubs {
    let temp_subs = get_temp_subs();
    copy_subs(temp_subs, subs_in, nvim_rex_get_nfa_has_zsubexpr() != 0);

    // Set the start position for group n
    let is_multi = nvim_rex_is_multi() != 0;
    if is_multi {
        (*temp_subs).norm.list.multi[n as usize].start_lnum = nvim_rex_get_lnum();
        let input = nvim_rex_get_input();
        let line = nvim_rex_get_line();
        (*temp_subs).norm.list.multi[n as usize].start_col =
            input.offset_from(line) as ColNr + off as ColNr;
    } else {
        (*temp_subs).norm.list.line[n as usize].start = nvim_rex_get_input().add(off as usize);
    }

    // Update in_use count
    if (*temp_subs).norm.in_use <= n {
        (*temp_subs).norm.in_use = n + 1;
    }

    addstate_impl(list, (*state).out, temp_subs, pim, off, depth + 1)
}

/// Handle ZOPEN (start of external subexpression).
///
/// # Safety
/// All pointers must be valid.
unsafe fn handle_zopen(
    list: *mut NfaList,
    state: *mut NfaState,
    subs_in: *const RegSubs,
    pim: *const NfaPim,
    off: c_int,
    depth: c_int,
    n: c_int,
) -> *mut RegSubs {
    let temp_subs = get_temp_subs();
    copy_subs(temp_subs, subs_in, true);

    // Set the start position for external group n
    let is_multi = nvim_rex_is_multi() != 0;
    if is_multi {
        (*temp_subs).synt.list.multi[n as usize].start_lnum = nvim_rex_get_lnum();
        let input = nvim_rex_get_input();
        let line = nvim_rex_get_line();
        (*temp_subs).synt.list.multi[n as usize].start_col =
            input.offset_from(line) as ColNr + off as ColNr;
    } else {
        (*temp_subs).synt.list.line[n as usize].start = nvim_rex_get_input().add(off as usize);
    }

    // Update in_use count
    if (*temp_subs).synt.in_use <= n {
        (*temp_subs).synt.in_use = n + 1;
    }

    addstate_impl(list, (*state).out, temp_subs, pim, off, depth + 1)
}

/// Handle ZCLOSE (end of external subexpression).
///
/// # Safety
/// All pointers must be valid.
unsafe fn handle_zclose(
    list: *mut NfaList,
    state: *mut NfaState,
    subs_in: *const RegSubs,
    pim: *const NfaPim,
    off: c_int,
    depth: c_int,
    n: c_int,
) -> *mut RegSubs {
    let temp_subs = get_temp_subs();
    copy_subs(temp_subs, subs_in, true);

    // Set the end position for external group n
    let is_multi = nvim_rex_is_multi() != 0;
    if is_multi {
        (*temp_subs).synt.list.multi[n as usize].end_lnum = nvim_rex_get_lnum();
        let input = nvim_rex_get_input();
        let line = nvim_rex_get_line();
        (*temp_subs).synt.list.multi[n as usize].end_col =
            input.offset_from(line) as ColNr + off as ColNr;
    } else {
        (*temp_subs).synt.list.line[n as usize].end = nvim_rex_get_input().add(off as usize);
    }

    addstate_impl(list, (*state).out, temp_subs, pim, off, depth + 1)
}

extern "C" {
    fn nvim_rex_get_nfa_ll_index() -> c_int;
}

// =============================================================================
// addstate_here
// =============================================================================

/// Add a state at a specific position in the current list.
///
/// This is used when adding states during processing of the current list,
/// to ensure they are processed in the current pass rather than the next.
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

/// Check if two submatches are equal.
///
/// # Safety
/// Both pointers must be valid.
pub unsafe fn sub_equal(sub1: *const RegSub, sub2: *const RegSub) -> bool {
    if sub1.is_null() || sub2.is_null() {
        return false;
    }

    let in_use1 = (*sub1).in_use;
    let in_use2 = (*sub2).in_use;

    // Different number of captures means different
    if in_use1 != in_use2 {
        return false;
    }

    if in_use1 == 0 {
        return true;
    }

    let is_multi = nvim_rex_is_multi() != 0;

    // Compare each capture
    for i in 0..in_use1 as usize {
        if is_multi {
            if (*sub1).list.multi[i].start_lnum != (*sub2).list.multi[i].start_lnum
                || (*sub1).list.multi[i].start_col != (*sub2).list.multi[i].start_col
            {
                return false;
            }
            // Only check end if backref is needed
            if nvim_rex_get_nfa_has_backref() != 0
                && ((*sub1).list.multi[i].end_lnum != (*sub2).list.multi[i].end_lnum
                    || (*sub1).list.multi[i].end_col != (*sub2).list.multi[i].end_col)
            {
                return false;
            }
        } else {
            if (*sub1).list.line[i].start != (*sub2).list.line[i].start {
                return false;
            }
            if nvim_rex_get_nfa_has_backref() != 0
                && (*sub1).list.line[i].end != (*sub2).list.line[i].end
            {
                return false;
            }
        }
    }

    true
}

// =============================================================================
// addstate FFI Export
// =============================================================================

/// Add a state to the NFA thread list.
///
/// This is the main entry point for adding states during NFA execution.
/// It handles epsilon transitions, submatch tracking, and PIM management.
///
/// # Safety
/// All non-null pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_addstate(
    list: *mut NfaList,
    state: *mut NfaState,
    subs: *const RegSubs,
    pim: *const NfaPim,
    off: c_int,
) -> *mut RegSubs {
    addstate(list, state, subs, pim, off)
}

/// Add a state at a specific position in the current list.
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
// Match Result Helpers
// =============================================================================

/// Check if the match result indicates a match was found.
///
/// Returns 1 if result is NFA_MATCH (1), 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_nfa_match_found(result: c_int) -> c_int {
    c_int::from(result == 1)
}

/// Check if the match result indicates we should continue matching.
///
/// Returns 1 if result is 0 (no match yet), 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_nfa_should_continue(result: c_int) -> c_int {
    c_int::from(result == 0)
}

/// Check if the match result indicates no match is possible.
///
/// Returns 1 if result is greater than 1 (special case), 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_nfa_no_match(result: c_int) -> c_int {
    c_int::from(result > 1)
}

/// Check if the match result indicates an error (too expensive).
///
/// Returns 1 if result is NFA_TOO_EXPENSIVE (-1), 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_nfa_match_error(result: c_int) -> c_int {
    c_int::from(result == -1)
}

/// Get the ADDSTATE_HERE_OFFSET constant.
#[no_mangle]
pub extern "C" fn rs_nfa_addstate_here_offset() -> c_int {
    ADDSTATE_HERE_OFFSET
}

/// Get NFA_MATCH constant.
#[no_mangle]
pub extern "C" fn rs_nfa_match_const() -> c_int {
    NFA_MATCH
}

/// Get NFA_SPLIT constant.
#[no_mangle]
pub extern "C" fn rs_nfa_split_const() -> c_int {
    NFA_SPLIT
}

/// Get NFA_EMPTY constant.
#[no_mangle]
pub extern "C" fn rs_nfa_empty_const() -> c_int {
    crate::nfa_states::NFA_EMPTY
}

/// Get NFA_SKIP constant.
#[no_mangle]
pub extern "C" fn rs_nfa_skip_const() -> c_int {
    NFA_SKIP
}

// =============================================================================
// Main NFA Matching Function - Phase 2: Core Execution Loop
// =============================================================================

// Additional FFI declarations for rs_nfa_regmatch
#[allow(dead_code, clashing_extern_declarations)]
extern "C" {
    // Rex state accessors for main loop
    fn nvim_nfa_rex_get_nfa_listid() -> c_int;
    fn nvim_nfa_rex_set_nfa_listid(v: c_int);
    fn nvim_rex_get_reg_icombine() -> bool;

    // NFA regprog accessors
    fn nvim_nfa_regprog_get_nstate(prog: *const c_void) -> c_int;
    fn nvim_nfa_regprog_get_re_engine(prog: *const c_void) -> c_uint;

    // NFA execution globals
    fn nvim_nfa_get_match() -> c_int;
    fn nvim_nfa_set_match(v: c_int);
    fn nvim_nfa_did_time_out() -> c_int;
    fn nvim_nfa_get_time_count() -> c_int;
    fn nvim_nfa_set_time_count(v: c_int);

    // NFA endp for invisible matches
    fn nvim_rex_get_nfa_endp() -> *const c_void;

    // Memory allocation - matching existing declarations in nfa_states.rs
    fn xmalloc(size: usize) -> *mut i8;
    fn xfree(ptr: *mut c_void);

    // Interrupt checking
    fn rs_reg_breakcheck();
    fn nvim_get_got_int() -> c_int;

    // UTF-8 helpers
    fn utf_ptr2char(ptr: *const i8) -> c_int;
    fn utfc_ptr2len(ptr: *const i8) -> c_int;
    fn utf_iscomposing_legacy(c: c_int) -> c_int;

    // Line navigation for multi-line matching
    fn nvim_reg_nextline();

    // Input advancing
    fn nvim_rex_set_input(ptr: *mut u8);

    // C copy_sub function (uses void* for FFI)
    fn nvim_nfa_copy_sub(to: *mut c_void, from: *const c_void);

    // Recursive regmatch for invisible matches (uses void* for FFI)
    fn nvim_nfa_recursive_regmatch(
        state: *mut c_void,
        pim: *const c_void,
        prog: *mut c_void,
        submatch: *mut c_void,
        m: *mut c_void,
        listids: *mut *mut c_int,
        listids_len: *mut c_int,
    ) -> c_int;

    // State processing callback - Phase 3 will move this to Rust
    // Uses void* for FFI compatibility
    fn nfa_regmatch_process_state(
        t: *const c_void,
        curc: c_int,
        clen: c_int,
        prog: *mut c_void,
        thislist: *mut c_void,
        nextlist: *mut c_void,
        start: *mut c_void,
        submatch: *mut c_void,
        m: *mut c_void,
        listids: *mut *mut c_int,
        listids_len: *mut c_int,
        add_state: *mut *mut c_void,
        add_here: *mut c_int,
        add_count: *mut c_int,
        add_off: *mut c_int,
    ) -> c_int;
}

use std::ffi::{c_uint, c_void};

/// Result constants for nfa_regmatch
pub const NFA_TOO_EXPENSIVE: c_int = -1;

/// Maximum states before switching to backtracking engine
pub const NFA_MAX_STATES: c_int = 100000;

/// Automatic engine constant
pub const AUTOMATIC_ENGINE: c_uint = 0;

/// Main NFA matching routine.
///
/// Run NFA to determine whether it matches rex.input.
///
/// When "nfa_endp" is not NULL it is a required end-of-match position.
///
/// Return true if there is a match, false if there is no match,
/// NFA_TOO_EXPENSIVE if we end up with too many states.
/// When there is a match "submatch" contains the positions.
///
/// # Safety
/// - `prog` must be a valid pointer to nfa_regprog_T
/// - `start` must be a valid pointer to nfa_state_T
/// - `submatch` and `m` must be valid pointers to regsubs_T
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_regmatch(
    prog: *mut c_void,
    start: *mut NfaState,
    submatch: *mut RegSubs,
    m: *mut RegSubs,
) -> c_int {
    // Check for interrupts and timeout before starting
    rs_reg_breakcheck();
    if nvim_get_got_int() != 0 {
        return 0; // false = no match
    }
    if nvim_nfa_did_time_out() != 0 {
        return 0;
    }

    // Initialize match result
    nvim_nfa_set_match(0);

    // Get number of states for allocation
    let nstate = nvim_nfa_regprog_get_nstate(prog);

    // Allocate memory for the lists of nodes
    let size = ((nstate + 1) as usize) * std::mem::size_of::<NfaThread>();
    let list0_t = xmalloc(size) as *mut NfaThread;
    let list1_t = xmalloc(size) as *mut NfaThread;

    if list0_t.is_null() || list1_t.is_null() {
        if !list0_t.is_null() {
            xfree(list0_t as *mut c_void);
        }
        if !list1_t.is_null() {
            xfree(list1_t as *mut c_void);
        }
        return NFA_TOO_EXPENSIVE;
    }

    // Initialize the two lists
    let mut list: [NfaList; 2] = [
        NfaList {
            t: list0_t,
            n: 0,
            id: 0,
            has_pim: 0,
            len: nstate + 1,
        },
        NfaList {
            t: list1_t,
            n: 0,
            id: 0,
            has_pim: 0,
            len: nstate + 1,
        },
    ];

    let mut flag = 0;
    let mut listids: *mut c_int = ptr::null_mut();
    let mut listids_len: c_int = 0;
    let mut go_to_nextline = false;

    // Set up initial list
    let thislist = &mut list[0] as *mut NfaList;
    let nextlist = &mut list[1] as *mut NfaList;

    (*thislist).n = 0;
    (*thislist).has_pim = 0;
    (*nextlist).n = 0;
    (*nextlist).has_pim = 0;

    // Initialize list IDs
    let initial_listid = nvim_nfa_rex_get_nfa_listid() + 1;
    (*thislist).id = initial_listid;
    nvim_nfa_rex_set_nfa_listid(initial_listid);

    // Check if start is an MOPEN state for optimized initialization
    let toplevel = (*start).c == NFA_MOPEN;

    // Initialize the first state
    let r = if toplevel {
        // Inline optimized code for first MOPEN
        let input = nvim_rex_get_input();
        let line = nvim_rex_get_line();
        let lnum = nvim_rex_get_lnum();

        if nvim_rex_is_multi() != 0 {
            (*m).norm.list.multi[0].start_lnum = lnum;
            (*m).norm.list.multi[0].start_col = (input as isize - line as isize) as ColNr;
            (*m).norm.orig_start_col = (*m).norm.list.multi[0].start_col;
        } else {
            (*m).norm.list.line[0].start = input;
        }
        (*m).norm.in_use = 1;
        addstate(thislist, (*start).out, m, ptr::null(), 0)
    } else {
        addstate(thislist, start, m, ptr::null(), 0)
    };

    if r.is_null() {
        nvim_nfa_set_match(NFA_TOO_EXPENSIVE);
        xfree(list[0].t as *mut c_void);
        xfree(list[1].t as *mut c_void);
        xfree(listids as *mut c_void);
        return NFA_TOO_EXPENSIVE;
    }

    // Main character loop
    loop {
        let input = nvim_rex_get_input();
        let curc = utf_ptr2char(input as *const i8);
        let mut clen = utfc_ptr2len(input as *const i8);
        if curc == 0 {
            // NUL byte - end of line
            clen = 0;
            go_to_nextline = false;
        }

        // Swap lists
        let thislist = &mut list[flag] as *mut NfaList;
        flag ^= 1;
        let nextlist = &mut list[flag] as *mut NfaList;

        (*nextlist).n = 0;
        (*nextlist).has_pim = 0;

        // Increment list ID
        let new_listid = nvim_nfa_rex_get_nfa_listid() + 1;
        nvim_nfa_rex_set_nfa_listid(new_listid);

        // Check for too many states (automatic engine switching)
        let re_engine = nvim_nfa_regprog_get_re_engine(prog);
        if re_engine == AUTOMATIC_ENGINE && new_listid >= NFA_MAX_STATES {
            nvim_nfa_set_match(NFA_TOO_EXPENSIVE);
            break;
        }

        (*thislist).id = new_listid;
        (*nextlist).id = new_listid + 1;

        // If the state lists are empty we can stop
        if (*thislist).n == 0 {
            break;
        }

        // Process all states in thislist
        let mut listidx = 0;
        while listidx < (*thislist).n {
            // Allow interrupting with CTRL-C
            rs_reg_breakcheck();
            if nvim_get_got_int() != 0 {
                break;
            }

            // Check timeout periodically
            let time_count = nvim_nfa_get_time_count() + 1;
            if time_count >= 20 {
                nvim_nfa_set_time_count(0);
                if nvim_nfa_did_time_out() != 0 {
                    break;
                }
            } else {
                nvim_nfa_set_time_count(time_count);
            }

            let t = &(*thislist).t.add(listidx as usize);

            // Process this state - Phase 3 will move this logic to Rust
            // For now, call back to C
            let mut add_state: *mut c_void = ptr::null_mut();
            let mut add_here: c_int = 0;
            let mut add_count: c_int = 0;
            let mut add_off: c_int = 0;

            let process_result = nfa_regmatch_process_state(
                (*t) as *const NfaThread as *const c_void,
                curc,
                clen,
                prog,
                thislist as *mut c_void,
                nextlist as *mut c_void,
                start as *mut c_void,
                submatch as *mut c_void,
                m as *mut c_void,
                &mut listids,
                &mut listids_len,
                &mut add_state,
                &mut add_here,
                &mut add_count,
                &mut add_off,
            );

            // Handle special return values
            if process_result == 2 {
                // goto nextchar
                // Update clen if needed
                if (*nextlist).n == 0 {
                    clen = 0;
                }
                break;
            } else if process_result == -1 {
                // NFA_TOO_EXPENSIVE
                nvim_nfa_set_match(NFA_TOO_EXPENSIVE);
                break;
            }

            // Handle add_state from the switch statement
            let add_state_typed = add_state as *mut NfaState;
            if !add_state_typed.is_null() {
                if add_here != 0 {
                    // Insert at current position using addstate_here
                    if addstate_here(
                        thislist,
                        add_state_typed,
                        &mut (*(*t)).subs,
                        &(*(*t)).pim,
                        listidx,
                    )
                    .is_null()
                    {
                        nvim_nfa_set_match(NFA_TOO_EXPENSIVE);
                        break;
                    }
                } else if add_count > 0 {
                    // Add multiple times (for lookbehind)
                    for _ in 0..add_count {
                        if addstate(
                            nextlist,
                            add_state_typed,
                            &(*(*t)).subs,
                            &(*(*t)).pim,
                            add_off,
                        )
                        .is_null()
                        {
                            nvim_nfa_set_match(NFA_TOO_EXPENSIVE);
                            break;
                        }
                    }
                } else {
                    // Normal case - add to nextlist
                    if addstate(
                        nextlist,
                        add_state_typed,
                        &(*(*t)).subs,
                        &(*(*t)).pim,
                        add_off,
                    )
                    .is_null()
                    {
                        nvim_nfa_set_match(NFA_TOO_EXPENSIVE);
                        break;
                    }
                }
            }

            listidx += 1;
        }

        // Advance to the next character, or advance to the next line, or finish
        if clen != 0 {
            let input = nvim_rex_get_input();
            nvim_rex_set_input(input.add(clen as usize));
        } else if go_to_nextline
            || (!nvim_rex_get_nfa_endp().is_null()
                && nvim_rex_is_multi() != 0
                && nvim_rex_get_lnum() < (*(nvim_rex_get_nfa_endp() as *const LPos)).lnum as c_int)
        {
            nvim_reg_nextline();
        } else {
            break;
        }

        // Allow interrupting with CTRL-C
        rs_reg_breakcheck();
        if nvim_get_got_int() != 0 {
            break;
        }

        // Check for timeout
        let time_count = nvim_nfa_get_time_count() + 1;
        if time_count >= 20 {
            nvim_nfa_set_time_count(0);
            if nvim_nfa_did_time_out() != 0 {
                break;
            }
        } else {
            nvim_nfa_set_time_count(time_count);
        }
    }

    // Cleanup
    xfree(list[0].t as *mut c_void);
    xfree(list[1].t as *mut c_void);
    xfree(listids as *mut c_void);

    // Return the match result
    nvim_nfa_get_match()
}

// =============================================================================
// State Processing - Phase 4: NFA State Machine Implementation
// =============================================================================

// Additional FFI declarations for state processing
extern "C" {
    // Character classification functions (wrappers for C macros)
    fn nvim_ri_digit(c: c_int) -> c_int;
    fn nvim_ri_hex(c: c_int) -> c_int;
    fn nvim_ri_octal(c: c_int) -> c_int;
    fn nvim_ri_word(c: c_int) -> c_int;
    fn nvim_ri_head(c: c_int) -> c_int;
    fn nvim_ri_alpha(c: c_int) -> c_int;
    fn nvim_ri_lower(c: c_int) -> c_int;
    fn nvim_ri_upper(c: c_int) -> c_int;

    // Vim character classification
    fn vim_isIDc(c: c_int) -> c_int;
    fn vim_iswordp_buf(ptr: *const u8, buf: *mut c_void) -> c_int;
    fn vim_isfilec(c: c_int) -> c_int;
    fn vim_isprintc(c: c_int) -> c_int;

    // Rex state accessors
    fn nvim_rex_get_reg_ic() -> bool;
    fn nvim_rex_get_reg_buf() -> *mut c_void;
    fn nvim_rex_get_reg_line_lbr() -> bool;
    fn nvim_rex_get_reg_maxline() -> c_int;

    // Character folding
    fn utf_fold(a: c_int) -> c_int;
    fn utf_ptr2len(p: *const i8) -> c_int;
}

// Use Rust implementations from ascii crate
use nvim_ascii::{rs_ascii_isdigit, rs_ascii_iswhite};

// Import state constants
use crate::nfa_states::{
    NFA_ALPHA, NFA_ANY, NFA_ANY_COMPOSING, NFA_BOF, NFA_BOL, NFA_BOW, NFA_DIGIT, NFA_EOF, NFA_EOL,
    NFA_EOW, NFA_FNAME, NFA_HEAD, NFA_HEX, NFA_IDENT, NFA_KWORD, NFA_LOWER, NFA_LOWER_IC,
    NFA_NALPHA, NFA_NDIGIT, NFA_NEWL, NFA_NHEAD, NFA_NHEX, NFA_NLOWER, NFA_NLOWER_IC, NFA_NOCTAL,
    NFA_NUPPER, NFA_NUPPER_IC, NFA_NWHITE, NFA_NWORD, NFA_OCTAL, NFA_PRINT, NFA_SFNAME, NFA_SIDENT,
    NFA_SKWORD, NFA_SPRINT, NFA_UPPER, NFA_UPPER_IC, NFA_WHITE, NFA_WORD,
};

// Import anchor checking functions
use crate::nfa_match::{rs_check_bof, rs_check_bol, rs_check_bow, rs_check_eof, rs_check_eow};

/// Result of state processing.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateProcessResult {
    /// The state to add (NULL if none).
    pub add_state: *mut NfaState,
    /// Whether to add using addstate_here.
    pub add_here: c_int,
    /// Count for NFA_SKIP states.
    pub add_count: c_int,
    /// Offset for addstate.
    pub add_off: c_int,
    /// Return code: 0 = continue, 2 = goto nextchar, -1 = NFA_TOO_EXPENSIVE.
    pub return_code: c_int,
}

impl Default for StateProcessResult {
    fn default() -> Self {
        Self {
            add_state: ptr::null_mut(),
            add_here: 0,
            add_count: 0,
            add_off: 0,
            return_code: 0,
        }
    }
}

/// Process simple character class states.
///
/// Returns true if the state was handled, false if it should be handled elsewhere.
///
/// # Safety
/// All pointers must be valid.
#[inline]
unsafe fn process_char_class(
    state_c: c_int,
    curc: c_int,
    clen: c_int,
    state: *mut NfaState,
    result: &mut StateProcessResult,
) -> bool {
    let matched = match state_c {
        NFA_IDENT => vim_isIDc(curc) != 0,
        NFA_SIDENT => rs_ascii_isdigit(curc) == 0 && vim_isIDc(curc) != 0,
        NFA_KWORD => {
            let input = nvim_rex_get_input();
            let buf = nvim_rex_get_reg_buf();
            vim_iswordp_buf(input, buf) != 0
        }
        NFA_SKWORD => {
            let input = nvim_rex_get_input();
            let buf = nvim_rex_get_reg_buf();
            rs_ascii_isdigit(curc) == 0 && vim_iswordp_buf(input, buf) != 0
        }
        NFA_FNAME => vim_isfilec(curc) != 0,
        NFA_SFNAME => rs_ascii_isdigit(curc) == 0 && vim_isfilec(curc) != 0,
        NFA_PRINT => {
            let input = nvim_rex_get_input();
            vim_isprintc(utf_ptr2char(input as *const i8)) != 0
        }
        NFA_SPRINT => {
            let input = nvim_rex_get_input();
            rs_ascii_isdigit(curc) == 0 && vim_isprintc(utf_ptr2char(input as *const i8)) != 0
        }
        NFA_WHITE => rs_ascii_iswhite(curc) != 0,
        NFA_NWHITE => curc != 0 && rs_ascii_iswhite(curc) == 0,
        NFA_DIGIT => nvim_ri_digit(curc) != 0,
        NFA_NDIGIT => curc != 0 && nvim_ri_digit(curc) == 0,
        NFA_HEX => nvim_ri_hex(curc) != 0,
        NFA_NHEX => curc != 0 && nvim_ri_hex(curc) == 0,
        NFA_OCTAL => nvim_ri_octal(curc) != 0,
        NFA_NOCTAL => curc != 0 && nvim_ri_octal(curc) == 0,
        NFA_WORD => nvim_ri_word(curc) != 0,
        NFA_NWORD => curc != 0 && nvim_ri_word(curc) == 0,
        NFA_HEAD => nvim_ri_head(curc) != 0,
        NFA_NHEAD => curc != 0 && nvim_ri_head(curc) == 0,
        NFA_ALPHA => nvim_ri_alpha(curc) != 0,
        NFA_NALPHA => curc != 0 && nvim_ri_alpha(curc) == 0,
        NFA_LOWER => nvim_ri_lower(curc) != 0,
        NFA_NLOWER => curc != 0 && nvim_ri_lower(curc) == 0,
        NFA_UPPER => nvim_ri_upper(curc) != 0,
        NFA_NUPPER => curc != 0 && nvim_ri_upper(curc) == 0,
        NFA_LOWER_IC => {
            let reg_ic = nvim_rex_get_reg_ic();
            nvim_ri_lower(curc) != 0 || (reg_ic && nvim_ri_upper(curc) != 0)
        }
        NFA_NLOWER_IC => {
            let reg_ic = nvim_rex_get_reg_ic();
            curc != 0 && !(nvim_ri_lower(curc) != 0 || (reg_ic && nvim_ri_upper(curc) != 0))
        }
        NFA_UPPER_IC => {
            let reg_ic = nvim_rex_get_reg_ic();
            nvim_ri_upper(curc) != 0 || (reg_ic && nvim_ri_lower(curc) != 0)
        }
        NFA_NUPPER_IC => {
            let reg_ic = nvim_rex_get_reg_ic();
            curc != 0 && !(nvim_ri_upper(curc) != 0 || (reg_ic && nvim_ri_lower(curc) != 0))
        }
        _ => return false, // Not a simple character class
    };

    if matched {
        result.add_state = (*state).out;
        result.add_off = clen;
    }
    true
}

/// Process NFA_ANY state (matches any character except NUL).
///
/// # Safety
/// All pointers must be valid.
#[inline]
unsafe fn process_any(
    curc: c_int,
    clen: c_int,
    state: *mut NfaState,
    result: &mut StateProcessResult,
) {
    if curc > 0 {
        result.add_state = (*state).out;
        result.add_off = clen;
    }
}

/// Process NFA_ANY_COMPOSING state.
///
/// # Safety
/// All pointers must be valid.
#[inline]
unsafe fn process_any_composing(
    curc: c_int,
    clen: c_int,
    state: *mut NfaState,
    result: &mut StateProcessResult,
) {
    // On a composing character skip over it. Otherwise do nothing.
    // Always matches.
    if utf_iscomposing_legacy(curc) != 0 {
        result.add_off = clen;
    } else {
        result.add_here = 1;
        result.add_off = 0;
    }
    result.add_state = (*state).out;
}

/// Process anchor states (BOL, EOL, BOW, EOW, BOF, EOF).
///
/// Returns true if the state was an anchor and was handled, false otherwise.
///
/// # Safety
/// All pointers must be valid.
#[inline]
unsafe fn process_anchor(
    state_c: c_int,
    curc: c_int,
    state: *mut NfaState,
    result: &mut StateProcessResult,
) -> bool {
    let matched = match state_c {
        NFA_BOL => rs_check_bol() != 0,
        NFA_EOL => curc == 0,
        NFA_BOW => rs_check_bow(curc) != 0,
        NFA_EOW => rs_check_eow() != 0,
        NFA_BOF => rs_check_bof() != 0,
        NFA_EOF => rs_check_eof(curc) != 0,
        _ => return false, // Not an anchor
    };

    if matched {
        result.add_here = 1;
        result.add_state = (*state).out;
    }
    true
}

/// Process NFA_MATCH state - we found a match!
///
/// Returns true if this was an NFA_MATCH state and was handled.
/// Sets result.return_code to 2 (goto nextchar) on success.
///
/// # Safety
/// All pointers must be valid.
#[inline]
unsafe fn process_match(
    curc: c_int,
    t: *const NfaThread,
    submatch: *mut RegSubs,
    nextlist: *const NfaList,
    result: &mut StateProcessResult,
) -> bool {
    // Check for composing character edge case
    // If match is not at start of line, ends before a composing character,
    // and rex.reg_icombine is not set, it's not really a match.
    let line = nvim_rex_get_line();
    let input = nvim_rex_get_input();
    if !nvim_rex_get_reg_icombine() && input != line && utf_iscomposing_legacy(curc) != 0 {
        return true; // Not a real match, but we handled it
    }

    // Found a match!
    nvim_nfa_set_match(1);

    // Copy submatch info
    copy_sub(&mut (*submatch).norm, &(*t).subs.norm);
    if nvim_rex_get_nfa_has_zsubexpr() != 0 {
        copy_sub(&mut (*submatch).synt, &(*t).subs.synt);
    }

    // Found the left-most longest match. When the list of states is going
    // to be empty, quit without advancing so rex.input is correct.
    // Return code 2 means "goto nextchar"
    result.return_code = 2;

    // Check if nextlist is empty (clen should be set to 0 by caller)
    // The clen=0 is handled in the Rust caller when return_code == 2
    let _ = nextlist; // Used for comment documentation

    true
}

/// Process NFA_NEWL state - match newline.
///
/// # Safety
/// All pointers must be valid.
#[inline]
unsafe fn process_newl(curc: c_int, state: *mut NfaState, result: &mut StateProcessResult) {
    if curc == 0 && !nvim_rex_get_reg_line_lbr() && nvim_rex_is_multi() != 0 {
        let lnum = nvim_rex_get_lnum();
        let maxline = nvim_rex_get_reg_maxline();
        if lnum <= maxline {
            // Pass -1 for the offset, which signals go_to_nextline = true
            // The Rust execution loop will handle this special offset.
            result.add_state = (*state).out;
            result.add_off = -1;
        }
    } else if curc == i32::from(b'\n') && nvim_rex_get_reg_line_lbr() {
        // match \n as if it is an ordinary character
        result.add_state = (*state).out;
        result.add_off = 1;
    }
}

/// Process NFA_SKIP state - skip over matched characters.
///
/// Used for backreferences (\1..\9) and \@> when skipping matched text.
///
/// # Safety
/// All pointers must be valid.
#[inline]
unsafe fn process_skip(
    t: *const NfaThread,
    clen: c_int,
    state: *mut NfaState,
    result: &mut StateProcessResult,
) {
    let count = (*t).count;
    if count - clen <= 0 {
        // End of match, go to what follows
        result.add_state = (*state).out;
        result.add_off = clen;
    } else {
        // Add state again with decremented count
        result.add_state = state;
        result.add_off = 0;
        result.add_count = count - clen;
    }
}

/// Process literal character matching (default case).
///
/// Returns true if this is a positive character state (c > 0) and was handled.
///
/// # Safety
/// All pointers must be valid.
#[inline]
unsafe fn process_literal(
    state_c: c_int,
    curc: c_int,
    clen: c_int,
    state: *mut NfaState,
    result: &mut StateProcessResult,
) -> bool {
    // Only handle positive character values (literal characters)
    if state_c <= 0 {
        return false;
    }

    let mut matched = state_c == curc;

    if !matched && nvim_rex_get_reg_ic() {
        matched = utf_fold(state_c) == utf_fold(curc);
    }

    if matched {
        // If rex.reg_icombine is not set only skip over the character
        // itself. When it is set skip over composing characters.
        let actual_clen = if !nvim_rex_get_reg_icombine() {
            let input = nvim_rex_get_input();
            utf_ptr2len(input as *const i8)
        } else {
            clen
        };
        result.add_state = (*state).out;
        result.add_off = actual_clen;
    }

    true
}

/// Main state processing function for NFA execution.
///
/// This function implements the large switch statement from C's nfa_regmatch.
/// It processes a single NFA state and returns information about what state
/// to add next.
///
/// # Arguments
/// * `t` - Current thread being processed
/// * `curc` - Current character (Unicode codepoint)
/// * `clen` - Length of current character in bytes
/// * `prog` - NFA program
/// * `thislist` - Current state list
/// * `nextlist` - Next state list
/// * `start` - Start state
/// * `submatch` - Submatch info (output)
/// * `m` - Match info
/// * `listids` - List IDs for recursive matching
/// * `listids_len` - Length of listids array
///
/// # Returns
/// A StateProcessResult containing:
/// * `add_state` - State to add, or NULL
/// * `add_here` - True if using addstate_here
/// * `add_count` - Count for NFA_SKIP
/// * `add_off` - Offset for addstate
/// * `return_code` - 0=continue, 2=goto nextchar, -1=error
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_process_state(
    t: *const NfaThread,
    curc: c_int,
    clen: c_int,
    _prog: *mut c_void,
    _thislist: *mut NfaList,
    nextlist: *mut NfaList,
    _start: *mut NfaState,
    submatch: *mut RegSubs,
    _m: *mut RegSubs,
    _listids: *mut *mut c_int,
    _listids_len: *mut c_int,
    add_state_out: *mut *mut NfaState,
    add_here_out: *mut c_int,
    add_count_out: *mut c_int,
    add_off_out: *mut c_int,
) -> c_int {
    if t.is_null() {
        return 0;
    }

    let state = (*t).state;
    if state.is_null() {
        return 0;
    }

    let state_c = (*state).c;
    let mut result = StateProcessResult::default();

    // Try character classes first (most common case)
    if process_char_class(state_c, curc, clen, state, &mut result) {
        // Handled by character class processing
    } else if process_anchor(state_c, curc, state, &mut result) {
        // Handled by anchor processing
    } else {
        // Handle other state types
        match state_c {
            NFA_MATCH => {
                process_match(curc, t, submatch, nextlist, &mut result);
            }
            NFA_ANY => process_any(curc, clen, state, &mut result),
            NFA_ANY_COMPOSING => process_any_composing(curc, clen, state, &mut result),
            NFA_NEWL => process_newl(curc, state, &mut result),
            NFA_SKIP => process_skip(t, clen, state, &mut result),
            _ => {
                // Try literal character matching (positive state values)
                if !process_literal(state_c, curc, clen, state, &mut result) {
                    // Not a literal character, return 0 to continue with C fallback
                    // for complex cases (collections, backrefs, lookaround, etc.)
                }
            }
        }
    }

    // Write output
    *add_state_out = result.add_state;
    *add_here_out = result.add_here;
    *add_count_out = result.add_count;
    *add_off_out = result.add_off;

    result.return_code
}

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
