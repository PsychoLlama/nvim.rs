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

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::nfa_states::{
    ColNr, LPos, NfaList, NfaPim, NfaState, NfaThread, RegSub, RegSubs, NFA_ALPHA, NFA_ANY,
    NFA_ANY_COMPOSING, NFA_BOF, NFA_BOL, NFA_COMPOSING, NFA_DIGIT, NFA_END_INVISIBLE,
    NFA_END_INVISIBLE_NEG, NFA_END_PATTERN, NFA_FNAME, NFA_HEAD, NFA_HEX, NFA_IDENT, NFA_KWORD,
    NFA_LOWER, NFA_LOWER_IC, NFA_MATCH, NFA_MCLOSE, NFA_MCLOSE9, NFA_MOPEN, NFA_NALPHA, NFA_NCLOSE,
    NFA_NDIGIT, NFA_NEWL, NFA_NHEAD, NFA_NHEX, NFA_NLOWER, NFA_NLOWER_IC, NFA_NOCTAL, NFA_NOPEN,
    NFA_NUPPER, NFA_NUPPER_IC, NFA_NWHITE, NFA_NWORD, NFA_OCTAL, NFA_PIM_MATCH, NFA_PIM_NOMATCH,
    NFA_PIM_TODO, NFA_PIM_UNUSED, NFA_PRINT, NFA_SFNAME, NFA_SIDENT, NFA_SKIP, NFA_SKWORD,
    NFA_SPLIT, NFA_SPRINT, NFA_START_COLL, NFA_START_INVISIBLE, NFA_START_INVISIBLE_BEFORE,
    NFA_START_INVISIBLE_BEFORE_FIRST, NFA_START_INVISIBLE_BEFORE_NEG,
    NFA_START_INVISIBLE_BEFORE_NEG_FIRST, NFA_START_INVISIBLE_FIRST, NFA_START_INVISIBLE_NEG,
    NFA_START_INVISIBLE_NEG_FIRST, NFA_START_NEG_COLL, NFA_UPPER, NFA_UPPER_IC, NFA_WHITE,
    NFA_WORD, NFA_ZCLOSE, NFA_ZCLOSE9, NFA_ZEND, NFA_ZOPEN, NFA_ZOPEN9, NFA_ZSTART, NSUBEXP,
};
use crate::RegprogHandle;

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
    // nvim_rex_get_nfa_endp is declared in the nfa_regmatch FFI section below

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

        c if (NFA_MCLOSE..=NFA_MCLOSE9).contains(&c) => {
            // End of capturing group
            let n = c - NFA_MCLOSE;
            return handle_mclose(list, state, subs_in, pim, off, depth, n);
        }

        NFA_BOL | NFA_BOF => {
            // "^" won't match past end-of-line, don't bother trying.
            // Except when at the end of the line, or when we are going to the
            // next line for a look-behind match.
            let input = nvim_rex_get_input();
            let line = nvim_rex_get_line();

            if input > line && *input != 0 {
                // Check if we have nfa_endp (look-behind match)
                let nfa_endp = nvim_rex_get_nfa_endp();
                if nfa_endp.is_null()
                    || nvim_rex_is_multi() == 0
                    || nvim_rex_get_lnum() == (*(nfa_endp as *const LPos)).lnum
                {
                    // Skip adding this state - BOL/BOF can't match here
                    return subs_in as *mut RegSubs;
                }
            }
            // Fall through to add to list
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

/// Handle MCLOSE (end of capturing group).
///
/// This sets the end position for the capturing group and then recurses
/// on the output state. The end position is saved and restored to allow
/// backtracking.
///
/// # Safety
/// All pointers must be valid.
unsafe fn handle_mclose(
    list: *mut NfaList,
    state: *mut NfaState,
    subs_in: *const RegSubs,
    pim: *const NfaPim,
    off: c_int,
    depth: c_int,
    n: c_int,
) -> *mut RegSubs {
    // Special case for MCLOSE (group 0): if \ze was used, don't overwrite
    if n == 0 && nvim_rex_get_nfa_has_zend() != 0 {
        let is_multi = nvim_rex_is_multi() != 0;
        let has_end = if is_multi {
            (*subs_in).norm.list.multi[0].end_lnum >= 0
        } else {
            !(*subs_in).norm.list.line[0].end.is_null()
        };

        if has_end {
            // \ze already set the end position, don't overwrite it
            return addstate_impl(list, (*state).out, subs_in, pim, off, depth + 1);
        }
    }

    let temp_subs = get_temp_subs();
    copy_subs(temp_subs, subs_in, nvim_rex_get_nfa_has_zsubexpr() != 0);

    // Save the current end position and in_use count for restoration
    let save_in_use = (*temp_subs).norm.in_use;

    // Update in_use if needed
    if (*temp_subs).norm.in_use <= n {
        (*temp_subs).norm.in_use = n + 1;
    }

    // Set the end position for group n
    let is_multi = nvim_rex_is_multi() != 0;
    if is_multi {
        let save_multipos = (*temp_subs).norm.list.multi[n as usize];

        if off == -1 {
            // Going to next line
            (*temp_subs).norm.list.multi[n as usize].end_lnum = nvim_rex_get_lnum() + 1;
            (*temp_subs).norm.list.multi[n as usize].end_col = 0;
        } else {
            (*temp_subs).norm.list.multi[n as usize].end_lnum = nvim_rex_get_lnum();
            let input = nvim_rex_get_input();
            let line = nvim_rex_get_line();
            (*temp_subs).norm.list.multi[n as usize].end_col =
                input.offset_from(line) as ColNr + off as ColNr;
        }

        let result = addstate_impl(list, (*state).out, temp_subs, pim, off, depth + 1);
        if result.is_null() {
            return ptr::null_mut();
        }

        // Restore the end position
        (*result).norm.list.multi[n as usize] = save_multipos;
        (*result).norm.in_use = save_in_use;

        result
    } else {
        let save_end = (*temp_subs).norm.list.line[n as usize].end;

        (*temp_subs).norm.list.line[n as usize].end = nvim_rex_get_input().add(off as usize);

        let result = addstate_impl(list, (*state).out, temp_subs, pim, off, depth + 1);
        if result.is_null() {
            return ptr::null_mut();
        }

        // Restore the end position
        (*result).norm.list.line[n as usize].end = save_end;
        (*result).norm.in_use = save_in_use;

        result
    }
}

// =============================================================================
// addstate_here
// =============================================================================

// Error message for maxmempattern
static E_MAXMEMPATTERN: &[u8] = b"E363: pattern uses more memory than 'maxmempattern'\0";

// xmalloc, xfree, and emsg are declared in the nfa_regmatch FFI section below

/// Add a state at a specific position in the current list.
///
/// This is used when adding states during processing of the current list,
/// to ensure they are processed in the current pass rather than the next.
/// The listidx pointer is updated to point before the inserted states.
///
/// # Safety
/// All pointers must be valid.
pub unsafe fn addstate_here(
    list: *mut NfaList,
    state: *mut NfaState,
    subs: *mut RegSubs,
    pim: *const NfaPim,
    ip: *mut c_int,
) -> *mut RegSubs {
    let tlen = (*list).n;
    let listidx = *ip;

    // First add the state(s) at the end, so that we know how many there are.
    // Pass the listidx as offset (avoids adding another argument to addstate).
    let r = addstate(list, state, subs, pim, -listidx - ADDSTATE_HERE_OFFSET);
    if r.is_null() {
        return ptr::null_mut();
    }

    // When "*ip" was at the end of the list, nothing to do
    if listidx + 1 == tlen {
        return r;
    }

    // Re-order to put the new state at the current position
    let count = (*list).n - tlen;
    if count == 0 {
        return r; // no state got added
    }

    if count == 1 {
        // Overwrite the current state
        *(*list).t.add(listidx as usize) = ptr::read((*list).t.add(((*list).n - 1) as usize));
    } else if count > 1 {
        if (*list).n + count > (*list).len {
            // Not enough space to move the new states, reallocate the list
            // and move the states to the right position
            let newlen = (*list).len * 3 / 2 + 50;
            let newsize = (newlen as usize) * std::mem::size_of::<NfaThread>();

            let p_mmp = nvim_get_p_mmp();
            if (newsize >> 10) as i64 >= p_mmp {
                emsg(E_MAXMEMPATTERN.as_ptr() as *const i8);
                return ptr::null_mut();
            }

            let newl = xmalloc(newsize) as *mut NfaThread;
            (*list).len = newlen;

            // Copy threads before listidx
            ptr::copy_nonoverlapping((*list).t, newl, listidx as usize);

            // Copy new threads to listidx position
            ptr::copy_nonoverlapping(
                (*list).t.add(((*list).n - count) as usize),
                newl.add(listidx as usize),
                count as usize,
            );

            // Copy threads after listidx
            ptr::copy_nonoverlapping(
                (*list).t.add((listidx + 1) as usize),
                newl.add((listidx + count) as usize),
                ((*list).n - count - listidx - 1) as usize,
            );

            xfree((*list).t as *mut std::ffi::c_void);
            (*list).t = newl;
        } else {
            // Make space for new states, then move them from the end to current position
            ptr::copy(
                (*list).t.add((listidx + 1) as usize),
                (*list).t.add((listidx + count) as usize),
                ((*list).n - listidx - 1) as usize,
            );
            ptr::copy_nonoverlapping(
                (*list).t.add(((*list).n - 1) as usize),
                (*list).t.add(listidx as usize),
                count as usize,
            );
        }
    }
    (*list).n -= 1;
    *ip = listidx - 1;

    r
}

// =============================================================================
// State List Helpers
// =============================================================================

/// Check if two submatches are equal.
///
/// Returns true if both have the same start positions. When using back-references,
/// also checks the end position. Handles the case where in_use differs by treating
/// missing entries as unset (-1/NULL).
///
/// # Safety
/// Both pointers must be valid.
pub unsafe fn sub_equal(sub1: *const RegSub, sub2: *const RegSub) -> bool {
    if sub1.is_null() || sub2.is_null() {
        return false;
    }

    let in_use1 = (*sub1).in_use;
    let in_use2 = (*sub2).in_use;
    let todo = if in_use1 > in_use2 { in_use1 } else { in_use2 };

    let is_multi = nvim_rex_is_multi() != 0;
    let has_backref = nvim_rex_get_nfa_has_backref() != 0;

    if is_multi {
        for i in 0..todo as usize {
            // Get start_lnum for sub1 (treat missing as -1)
            let s1 = if (i as c_int) < in_use1 {
                (*sub1).list.multi[i].start_lnum
            } else {
                -1
            };
            // Get start_lnum for sub2 (treat missing as -1)
            let s2 = if (i as c_int) < in_use2 {
                (*sub2).list.multi[i].start_lnum
            } else {
                -1
            };
            if s1 != s2 {
                return false;
            }
            // Compare start_col if both have valid start_lnum
            if s1 != -1 && (*sub1).list.multi[i].start_col != (*sub2).list.multi[i].start_col {
                return false;
            }

            // Check end position if backrefs are used
            if has_backref {
                let e1 = if (i as c_int) < in_use1 {
                    (*sub1).list.multi[i].end_lnum
                } else {
                    -1
                };
                let e2 = if (i as c_int) < in_use2 {
                    (*sub2).list.multi[i].end_lnum
                } else {
                    -1
                };
                if e1 != e2 {
                    return false;
                }
                if e1 != -1 && (*sub1).list.multi[i].end_col != (*sub2).list.multi[i].end_col {
                    return false;
                }
            }
        }
    } else {
        for i in 0..todo as usize {
            // Get start ptr for sub1 (treat missing as NULL)
            let sp1 = if (i as c_int) < in_use1 {
                (*sub1).list.line[i].start
            } else {
                ptr::null_mut()
            };
            // Get start ptr for sub2 (treat missing as NULL)
            let sp2 = if (i as c_int) < in_use2 {
                (*sub2).list.line[i].start
            } else {
                ptr::null_mut()
            };
            if sp1 != sp2 {
                return false;
            }

            // Check end position if backrefs are used
            if has_backref {
                let ep1 = if (i as c_int) < in_use1 {
                    (*sub1).list.line[i].end
                } else {
                    ptr::null_mut()
                };
                let ep2 = if (i as c_int) < in_use2 {
                    (*sub2).list.line[i].end
                } else {
                    ptr::null_mut()
                };
                if ep1 != ep2 {
                    return false;
                }
            }
        }
    }

    true
}

/// Check if two PIMs are equal.
///
/// Returns true if both are unused, or if both have the same state ID
/// and end position.
///
/// # Safety
/// If pointers are non-null, they must point to valid NfaPim structs.
pub unsafe fn pim_equal(one: *const NfaPim, two: *const NfaPim) -> bool {
    let one_unused = one.is_null() || (*one).result == NFA_PIM_UNUSED;
    let two_unused = two.is_null() || (*two).result == NFA_PIM_UNUSED;

    if one_unused {
        // one is unused: equal when two is also unused
        return two_unused;
    }
    if two_unused {
        // one is used and two is not: not equal
        return false;
    }

    // Compare the state id
    if (*(*one).state).id != (*(*two).state).id {
        return false;
    }

    // Compare the position
    let is_multi = nvim_rex_is_multi() != 0;
    if is_multi {
        (*one).end.pos.lnum == (*two).end.pos.lnum && (*one).end.pos.col == (*two).end.pos.col
    } else {
        (*one).end.ptr == (*two).end.ptr
    }
}

/// Check if a state is already in the list with the same positions.
///
/// This prevents adding duplicate states which would cause infinite loops
/// or unnecessary work.
///
/// # Safety
/// All pointers must be valid.
pub unsafe fn has_state_with_pos(
    list: *const NfaList,
    state: *const NfaState,
    subs: *const RegSubs,
    pim: *const NfaPim,
) -> bool {
    if list.is_null() || state.is_null() || subs.is_null() {
        return false;
    }

    let has_zsubexpr = nvim_rex_get_nfa_has_zsubexpr() != 0;

    for i in 0..(*list).n {
        let thread = (*list).t.add(i as usize);
        if (*(*thread).state).id == (*state).id
            && sub_equal(&(*thread).subs.norm, &(*subs).norm)
            && (!has_zsubexpr || sub_equal(&(*thread).subs.synt, &(*subs).synt))
            && pim_equal(&(*thread).pim, pim)
        {
            return true;
        }
    }
    false
}

/// Check if the given state leads to a match without advancing input.
///
/// This is used to determine if a zero-width assertion might match
/// at the current position.
///
/// # Safety
/// State pointer must be valid.
pub unsafe fn match_follows(startstate: *const NfaState, depth: c_int) -> bool {
    // Avoid too much recursion
    if depth > 10 {
        return false;
    }

    let mut state = startstate;
    while !state.is_null() {
        match (*state).c {
            // These states indicate a match without consuming input
            NFA_MATCH
            | NFA_MCLOSE
            | NFA_END_INVISIBLE
            | NFA_END_INVISIBLE_NEG
            | NFA_END_PATTERN => {
                return true;
            }

            // Split: recurse on both branches
            NFA_SPLIT => {
                return match_follows((*state).out, depth + 1)
                    || match_follows((*state).out1, depth + 1);
            }

            // Invisible match states: skip ahead to the continuation
            NFA_START_INVISIBLE
            | NFA_START_INVISIBLE_FIRST
            | NFA_START_INVISIBLE_BEFORE
            | NFA_START_INVISIBLE_BEFORE_FIRST
            | NFA_START_INVISIBLE_NEG
            | NFA_START_INVISIBLE_NEG_FIRST
            | NFA_START_INVISIBLE_BEFORE_NEG
            | NFA_START_INVISIBLE_BEFORE_NEG_FIRST
            | NFA_COMPOSING => {
                // Skip ahead to next state
                state = (*(*state).out1).out;
                continue;
            }

            // Character classes and character matching - will advance input
            NFA_ANY | NFA_ANY_COMPOSING | NFA_IDENT | NFA_SIDENT | NFA_KWORD | NFA_SKWORD
            | NFA_FNAME | NFA_SFNAME | NFA_PRINT | NFA_SPRINT | NFA_WHITE | NFA_NWHITE
            | NFA_DIGIT | NFA_NDIGIT | NFA_HEX | NFA_NHEX | NFA_OCTAL | NFA_NOCTAL | NFA_WORD
            | NFA_NWORD | NFA_HEAD | NFA_NHEAD | NFA_ALPHA | NFA_NALPHA | NFA_LOWER
            | NFA_NLOWER | NFA_UPPER | NFA_NUPPER | NFA_LOWER_IC | NFA_NLOWER_IC | NFA_UPPER_IC
            | NFA_NUPPER_IC | NFA_START_COLL | NFA_START_NEG_COLL | NFA_NEWL => {
                // State will advance input
                return false;
            }

            c => {
                if c > 0 {
                    // Positive character code means it matches a character
                    return false;
                }
                // Others: zero-width or possibly zero-width, might still find
                // a match at the same position, keep looking.
            }
        }
        state = (*state).out;
    }
    false
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
    ip: *mut c_int,
) -> *mut RegSubs {
    addstate_here(list, state, subs, pim, ip)
}

/// Check if two PIMs are equal.
///
/// # Safety
/// If pointers are non-null, they must point to valid NfaPim structs.
#[no_mangle]
pub unsafe extern "C" fn rs_pim_equal(one: *const NfaPim, two: *const NfaPim) -> c_int {
    c_int::from(pim_equal(one, two))
}

/// Check if a state is already in the list with the same positions.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_has_state_with_pos(
    list: *const NfaList,
    state: *const NfaState,
    subs: *const RegSubs,
    pim: *const NfaPim,
) -> c_int {
    c_int::from(has_state_with_pos(list, state, subs, pim))
}

/// Check if two submatches are equal.
///
/// # Safety
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_sub_equal(sub1: *const RegSub, sub2: *const RegSub) -> c_int {
    c_int::from(sub_equal(sub1, sub2))
}

/// Check if a state leads to a match without advancing input.
///
/// # Safety
/// State pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_match_follows(state: *const NfaState, depth: c_int) -> c_int {
    c_int::from(match_follows(state, depth))
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
// Recursive Regmatch - Phase 10d
// =============================================================================

/// Recursively call nfa_regmatch() for invisible/lookahead matches.
///
/// This function handles the setup and teardown for recursive matching,
/// including saving/restoring rex state and managing listids.
///
/// # Safety
/// All pointers must be valid. The listids pointer may be reallocated.
#[no_mangle]
pub unsafe extern "C" fn rs_recursive_regmatch(
    state: *mut NfaState,
    pim: *const NfaPim,
    prog: *mut c_void,
    submatch: *mut RegSubs,
    m: *mut RegSubs,
    listids: *mut *mut c_int,
    listids_len: *mut c_int,
) -> c_int {
    recursive_regmatch(state, pim, prog, submatch, m, listids, listids_len)
}

/// Internal implementation of recursive_regmatch.
///
/// # Safety
/// All pointers must be valid.
unsafe fn recursive_regmatch(
    state: *mut NfaState,
    pim: *const NfaPim,
    prog: *mut c_void,
    submatch: *mut RegSubs,
    m: *mut RegSubs,
    listids: *mut *mut c_int,
    listids_len: *mut c_int,
) -> c_int {
    // Save current rex state
    let save_reginput_col = nvim_rex_get_input().offset_from(nvim_rex_get_line()) as c_int;
    let save_reglnum = nvim_rex_get_lnum();
    let save_nfa_match = nvim_nfa_get_match();
    let save_nfa_listid = nvim_rex_get_nfa_listid();
    let save_nfa_endp = nvim_rex_get_nfa_endp();
    let mut endpos = SaveSe::default();
    let mut endposp: *mut SaveSe = std::ptr::null_mut();
    let mut need_restore = false;

    // If pim is set, start at the position where the postponed match was
    if !pim.is_null() {
        if nvim_rex_is_multi() != 0 {
            let line = nvim_rex_get_line();
            nvim_rex_set_input(line.add((*pim).end.pos.col as usize));
        } else {
            nvim_rex_set_input((*pim).end.ptr);
        }
    }

    // Check if this is a BEFORE variant (lookbehind)
    let state_c = (*state).c;
    if state_c == NFA_START_INVISIBLE_BEFORE
        || state_c == NFA_START_INVISIBLE_BEFORE_FIRST
        || state_c == NFA_START_INVISIBLE_BEFORE_NEG
        || state_c == NFA_START_INVISIBLE_BEFORE_NEG_FIRST
    {
        // The recursive match must end at the current position
        endposp = &mut endpos;
        if nvim_rex_is_multi() != 0 {
            if pim.is_null() {
                endpos.se_u.pos.col =
                    nvim_rex_get_input().offset_from(nvim_rex_get_line()) as c_int;
                endpos.se_u.pos.lnum = nvim_rex_get_lnum();
            } else {
                endpos.se_u.pos = (*pim).end.pos;
            }
        } else if pim.is_null() {
            endpos.se_u.ptr = nvim_rex_get_input();
        } else {
            endpos.se_u.ptr = (*pim).end.ptr;
        }

        // Go back the specified number of bytes
        let state_val = (*state).val;
        if state_val <= 0 {
            if nvim_rex_is_multi() != 0 {
                let new_lnum = nvim_rex_get_lnum() - 1;
                let line = nvim_reg_getline(new_lnum) as *mut u8;
                if line.is_null() {
                    // Can't go before the first line
                    let cur_line = nvim_reg_getline(nvim_rex_get_lnum()) as *mut u8;
                    nvim_rex_set_line(cur_line);
                } else {
                    nvim_rex_set_lnum(new_lnum);
                    nvim_rex_set_line(line);
                }
            }
            nvim_rex_set_input(nvim_rex_get_line());
        } else {
            let input = nvim_rex_get_input();
            let line = nvim_rex_get_line();
            let offset = input.offset_from(line) as c_int;

            if nvim_rex_is_multi() != 0 && offset < state_val {
                // Not enough bytes in this line, go to end of previous line
                let new_lnum = nvim_rex_get_lnum() - 1;
                let prev_line = nvim_reg_getline(new_lnum) as *mut u8;
                if prev_line.is_null() {
                    // Can't go before the first line
                    let cur_line = nvim_reg_getline(nvim_rex_get_lnum()) as *mut u8;
                    nvim_rex_set_line(cur_line);
                    nvim_rex_set_input(cur_line);
                } else {
                    nvim_rex_set_lnum(new_lnum);
                    nvim_rex_set_line(prev_line);
                    let len = nvim_reg_getline_len(new_lnum);
                    nvim_rex_set_input(prev_line.add(len as usize));
                }
            }

            // Now go back state_val bytes (if possible)
            let input = nvim_rex_get_input();
            let line = nvim_rex_get_line();
            let offset = input.offset_from(line) as c_int;

            if offset >= state_val {
                let new_input = input.sub(state_val as usize);
                // Adjust for UTF-8 head offset
                let head_off = utf_head_off(line, new_input);
                nvim_rex_set_input(new_input.sub(head_off as usize));
            } else {
                nvim_rex_set_input(line);
            }
        }
    }

    // Handle listids for recursive calls
    let nfa_ll_index = nvim_rex_get_nfa_ll_index();
    if nfa_ll_index == 1 {
        // Already calling nfa_regmatch() recursively. Save the lastlist[1]
        // values and clear them.
        let nstate = nvim_nfa_prog_get_nstate(prog);
        if (*listids).is_null() || *listids_len < nstate {
            xfree(*listids as *mut c_void);
            *listids = xmalloc(std::mem::size_of::<c_int>() * nstate as usize) as *mut c_int;
            *listids_len = nstate;
        }
        nvim_nfa_save_listids(prog, *listids);
        need_restore = true;
        // Any value of rex.nfa_listid will do
    } else {
        // First recursive nfa_regmatch() call, switch to the second lastlist entry
        nvim_rex_set_nfa_ll_index(nfa_ll_index + 1);
        let nfa_listid = nvim_rex_get_nfa_listid();
        let nfa_alt_listid = nvim_rex_get_nfa_alt_listid();
        if nfa_listid <= nfa_alt_listid {
            nvim_rex_set_nfa_listid(nfa_alt_listid);
        }
    }

    // Call nfa_regmatch() to check if the current concat matches at this position
    nvim_rex_set_nfa_endp(endposp as *mut c_void);
    let result = nvim_nfa_regmatch(
        prog,
        (*state).out as *mut c_void,
        submatch as *mut c_void,
        m as *mut c_void,
    );

    // Restore listids
    if need_restore {
        nvim_nfa_restore_listids(prog, *listids);
    } else {
        nvim_rex_set_nfa_ll_index(nvim_rex_get_nfa_ll_index() - 1);
        nvim_rex_set_nfa_alt_listid(nvim_rex_get_nfa_listid());
    }

    // Restore position in input text
    nvim_rex_set_lnum(save_reglnum);
    if nvim_rex_is_multi() != 0 {
        let line = nvim_reg_getline(save_reglnum) as *mut u8;
        nvim_rex_set_line(line);
    }
    nvim_rex_set_input(nvim_rex_get_line().add(save_reginput_col as usize));

    if result != NFA_TOO_EXPENSIVE {
        nvim_nfa_set_match(save_nfa_match);
        nvim_rex_set_nfa_listid(save_nfa_listid);
    }
    nvim_rex_set_nfa_endp(save_nfa_endp as *mut c_void);

    result
}

/// save_se_T equivalent structure
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct SaveSe {
    se_u: SaveSeUnion,
}

#[repr(C)]
#[derive(Clone, Copy)]
union SaveSeUnion {
    ptr: *mut u8,
    pos: LPos,
}

impl Default for SaveSeUnion {
    fn default() -> Self {
        SaveSeUnion {
            ptr: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// State Processing - Phase 4: NFA State Machine Implementation
// =============================================================================

// Constants for nfa_regmatch (also used by recursive_regmatch)
pub const NFA_TOO_EXPENSIVE: c_int = -1;

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
    fn nvim_rex_get_reg_icombine() -> bool;
    fn nvim_rex_get_reg_buf() -> *mut c_void;
    fn nvim_rex_get_reg_line_lbr() -> bool;
    fn nvim_rex_get_reg_maxline() -> c_int;
    fn nvim_rex_get_reg_firstlnum() -> c_int;
    fn nvim_rex_get_reg_win() -> *mut c_void;
    fn nvim_rex_get_cursor_lnum() -> c_int;
    fn nvim_rex_get_cursor_col() -> c_int;

    // Character folding
    fn utf_fold(a: c_int) -> c_int;
    fn utf_ptr2len(p: *const i8) -> c_int;

    // UTF-8 character length from codepoint
    fn rs_utf_char2len(c: c_int) -> c_int;

    // Backreference matching
    fn nvim_nfa_match_backref(sub: *const c_void, subidx: c_int, bytelen: *mut c_int) -> c_int;
    fn nvim_nfa_match_zref(subidx: c_int, bytelen: *mut c_int) -> c_int;

    // Position matching wrappers (Phase 5.8)
    fn nvim_nfa_check_vcol(val: c_int, op: c_int) -> bool;
    fn nvim_nfa_check_mark(mark_id: c_int, op: c_int) -> bool;
    fn nvim_nfa_check_visual() -> bool;

    // Invisible/lookaround wrappers (Phase 5.9)
    fn nvim_nfa_check_end_invisible(
        state_c: c_int,
        t_subs: *const c_void,
        m: *mut c_void,
        nextlist_n: c_int,
        nfa_endp_ptr: *const c_void,
    ) -> c_int;

    // NFA execution globals
    fn nvim_nfa_get_match() -> c_int;
    fn nvim_nfa_set_match(v: c_int);

    // NFA endp for invisible matches
    fn nvim_rex_get_nfa_endp() -> *const c_void;
    fn nvim_rex_set_nfa_endp(p: *mut c_void);

    // NFA ll_index for recursive calls
    fn nvim_rex_get_nfa_ll_index() -> c_int;
    fn nvim_rex_set_nfa_ll_index(v: c_int);

    // Rex state setters for recursive regmatch
    fn nvim_rex_set_line(ptr: *mut u8);
    fn nvim_rex_set_lnum(lnum: c_int);
    fn nvim_rex_get_nfa_listid() -> c_int;
    fn nvim_rex_set_nfa_listid(v: c_int);
    fn nvim_rex_get_nfa_alt_listid() -> c_int;
    fn nvim_rex_set_nfa_alt_listid(v: c_int);

    // Line navigation (use c_char for signature consistency with helpers.rs)
    fn nvim_reg_getline(lnum: c_int) -> *mut c_char;
    fn nvim_reg_getline_len(lnum: c_int) -> c_int;

    // UTF-8 helpers (use u8 for signature consistency with helpers.rs)
    fn utf_head_off(base: *const u8, p: *const u8) -> c_int;

    // Listid save/restore
    fn nvim_nfa_save_listids(prog: *mut c_void, list: *mut c_int);
    fn nvim_nfa_restore_listids(prog: *mut c_void, list: *const c_int);

    // Program accessors
    fn nvim_nfa_prog_get_nstate(prog: *const c_void) -> c_int;

    // nfa_regmatch - calls back into C main loop
    fn nvim_nfa_regmatch(
        prog: *mut c_void,
        start: *mut c_void,
        submatch: *mut c_void,
        m: *mut c_void,
    ) -> c_int;

    // Memory allocation and error reporting
    fn xmalloc(size: usize) -> *mut i8;
    fn xfree(ptr: *mut c_void);
    fn emsg(s: *const i8);

    // UTF-8 helpers for character handling
    fn utf_ptr2char(ptr: *const i8) -> c_int;
    fn utf_iscomposing_legacy(c: c_int) -> c_int;

    // Input advancing
    fn nvim_rex_set_input(ptr: *mut u8);

    // Copy submatch info (not main match position)
    fn nvim_nfa_copy_sub_off(to: *mut c_void, from: *const c_void);

    // Copy \ze end position if present
    fn nvim_nfa_copy_ze_off(to: *mut c_void, from: *const c_void);

    // Check if state is in list (for NFA_START_PATTERN optimization)
    fn nvim_nfa_state_in_list(
        list: *const c_void,
        state: *const c_void,
        subs: *const c_void,
    ) -> c_int;
}

// Use Rust implementations from ascii crate
use nvim_ascii::{rs_ascii_isdigit, rs_ascii_iswhite};

// Import additional state constants (many already imported at top of file)
use crate::nfa_states::{
    NFA_BACKREF1, NFA_BACKREF9, NFA_BOW, NFA_COL, NFA_COL_GT, NFA_COL_LT, NFA_CURSOR, NFA_END_COLL,
    NFA_END_COMPOSING, NFA_EOF, NFA_EOL, NFA_EOW, NFA_LNUM, NFA_LNUM_GT, NFA_LNUM_LT, NFA_MARK,
    NFA_MARK_GT, NFA_MARK_LT, NFA_RANGE_MIN, NFA_START_PATTERN, NFA_VCOL, NFA_VCOL_GT, NFA_VCOL_LT,
    NFA_VISUAL, NFA_ZREF1, NFA_ZREF9,
};

// Import check_char_class for POSIX classes in collections
use crate::nfa_states::check_char_class_impl;

// Import anchor checking functions
use crate::nfa_match::{rs_check_bof, rs_check_bol, rs_check_bow, rs_check_eof, rs_check_eow};

// Import number comparison for position matching
use crate::parser::nfa_re_num_cmp;

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

/// Maximum combining characters (fixed value for 'maxcombine').
const MAX_MCO: usize = 6;

/// Process NFA_COMPOSING state - match composing character sequences.
///
/// Handles matching of patterns that involve composing characters,
/// such as `\Z` mode or explicit composing character patterns.
///
/// Returns true if the state was handled (match or no match),
/// false if this is not a composing state.
///
/// # Safety
/// All pointers must be valid.
#[inline]
unsafe fn process_composing(
    curc: c_int,
    clen: c_int,
    state: *mut NfaState,
    result: &mut StateProcessResult,
) -> bool {
    // Get the first state in the composing chain (state->out)
    let mut sta = (*state).out;
    let mut len: c_int = 0;
    let mc = curc;

    // Check if pattern starts with a composing character
    if utf_iscomposing_legacy((*sta).c) != 0 {
        // Only match composing character(s), ignore base character.
        // Used for ".{composing}" and "{composing}" (no preceding character).
        len += rs_utf_char2len(mc);
    }

    // Check for \Z mode (rex.reg_icombine)
    if nvim_rex_get_reg_icombine() && len == 0 {
        // If \Z was present, then ignore composing characters.
        // When ignoring the base character this always matches.
        let matched = (*sta).c == curc;

        // Skip to NFA_END_COMPOSING
        while (*sta).c != NFA_END_COMPOSING {
            sta = (*sta).out;
        }

        if matched {
            // Match - use out1 (NFA_END_COMPOSING state)
            result.add_state = (*state).out1;
            result.add_off = clen;
        }
        return true;
    }

    // Check if base character matches (or was already skipped due to composing)
    if len > 0 || mc == (*sta).c {
        // Check base character matches first, unless ignored.
        if len == 0 {
            len += rs_utf_char2len(mc);
            sta = (*sta).out;
        }

        // Collect composing characters from input.
        // We don't care about the order of composing characters.
        let mut cchars: [c_int; MAX_MCO] = [0; MAX_MCO];
        let mut ccount: usize = 0;

        let input = nvim_rex_get_input();
        while len < clen {
            let char_mc = utf_ptr2char(input.add(len as usize) as *const i8);
            if ccount < MAX_MCO {
                cchars[ccount] = char_mc;
                ccount += 1;
            }
            len += rs_utf_char2len(char_mc);
            if ccount == MAX_MCO {
                break;
            }
        }

        // Check that each composing char in the pattern matches a
        // composing char in the text. We do not check if all
        // composing chars are matched.
        let mut matched = true;
        while (*sta).c != NFA_END_COMPOSING {
            let found = cchars[..ccount].iter().any(|&c| c == (*sta).c);
            if !found {
                matched = false;
                break;
            }
            sta = (*sta).out;
        }

        if matched {
            // Match - use out1 (NFA_END_COMPOSING state)
            result.add_state = (*state).out1;
            result.add_off = clen;
        }
    }
    // If we didn't match, add_state remains NULL (no match)

    true
}

/// Process NFA_START_COLL and NFA_START_NEG_COLL - character collection matching.
///
/// Handles patterns like [abc], [^abc], [a-z], [:alpha:], etc.
/// Collection matching walks through a chain of states until NFA_END_COLL.
///
/// Returns true if the state was handled.
///
/// # Safety
/// All pointers must be valid.
#[inline]
unsafe fn process_collection(
    state_c: c_int,
    curc: c_int,
    clen: c_int,
    state: *mut NfaState,
    result: &mut StateProcessResult,
) -> bool {
    // Never match EOL. If it's part of the collection it is added
    // as a separate state with an OR.
    if curc == 0 {
        return true; // Handled but no match
    }

    let result_if_matched = state_c == NFA_START_COLL;
    let mut coll_state = (*state).out;
    let mut matched = false;

    loop {
        let sc = (*coll_state).c;

        // Handle NFA_COMPOSING within collection
        if sc == NFA_COMPOSING {
            // This handles composing characters in collections - complex case
            // We need to check if the composing sequence matches
            let mut sta = (*(*state).out).out;
            let mut len: c_int = 0;
            let mc = curc;

            if utf_iscomposing_legacy((*sta).c) != 0 {
                len += rs_utf_char2len(mc);
            }

            if nvim_rex_get_reg_icombine() && len == 0 {
                // \Z mode - ignore composing characters
                matched = (*sta).c == curc;
                while (*sta).c != NFA_END_COMPOSING {
                    sta = (*sta).out;
                }
            } else if len > 0 || mc == (*sta).c {
                if len == 0 {
                    len += rs_utf_char2len(mc);
                    sta = (*sta).out;
                }

                // Collect composing characters from input
                let mut cchars: [c_int; MAX_MCO] = [0; MAX_MCO];
                let mut ccount: usize = 0;
                let input = nvim_rex_get_input();
                while len < clen {
                    let char_mc = utf_ptr2char(input.add(len as usize) as *const i8);
                    if ccount < MAX_MCO {
                        cchars[ccount] = char_mc;
                        ccount += 1;
                    }
                    len += rs_utf_char2len(char_mc);
                    if ccount == MAX_MCO {
                        break;
                    }
                }

                // Check that each composing char in the pattern matches one in input
                matched = true;
                while (*sta).c != NFA_END_COMPOSING {
                    if !cchars[..ccount].iter().any(|&c| c == (*sta).c) {
                        matched = false;
                        break;
                    }
                    sta = (*sta).out;
                }
            }

            // Check if out1 of collection's out is NFA_END_COMPOSING
            if (*(*(*state).out).out1).c == NFA_END_COMPOSING && matched == result_if_matched {
                result.add_state = (*(*(*state).out).out1).out;
                result.add_off = clen;
            }
            return true;
        }

        // Check for end of collection
        if sc == NFA_END_COLL {
            // Did not match anything - XOR with result_if_matched
            matched = !result_if_matched;
            break;
        }

        // Check for character range: NFA_RANGE_MIN followed by NFA_RANGE_MAX
        if sc == NFA_RANGE_MIN {
            let c1 = (*coll_state).val;
            coll_state = (*coll_state).out; // advance to NFA_RANGE_MAX
            let c2 = (*coll_state).val;

            // Direct range check
            if curc >= c1 && curc <= c2 {
                matched = result_if_matched;
                break;
            }

            // Case-insensitive range check
            if nvim_rex_get_reg_ic() {
                let curc_low = utf_fold(curc);
                let mut c1_iter = c1;
                while c1_iter <= c2 {
                    if utf_fold(c1_iter) == curc_low {
                        matched = result_if_matched;
                        break;
                    }
                    c1_iter += 1;
                }
                if matched == result_if_matched {
                    break;
                }
            }
        } else if sc < 0 {
            // POSIX character class (negative state value)
            if check_char_class_impl(sc, curc) != 0 {
                matched = result_if_matched;
                break;
            }
        } else {
            // Literal character match
            if curc == sc || (nvim_rex_get_reg_ic() && utf_fold(curc) == utf_fold(sc)) {
                matched = result_if_matched;
                break;
            }
        }

        coll_state = (*coll_state).out;
    }

    if matched {
        // Next state is in out of the NFA_END_COLL, out1 of START points to END
        result.add_state = (*(*state).out1).out;
        result.add_off = clen;
    }

    true
}

/// Process NFA_END_INVISIBLE, NFA_END_INVISIBLE_NEG, NFA_END_PATTERN states.
///
/// These states mark the end of a lookahead/lookbehind pattern.
/// When matched, set nfa_match and return control to the parent nfa_regmatch().
///
/// Returns: true if handled
/// Sets result.return_code to 2 (goto nextchar) on match.
///
/// # Safety
/// All pointers must be valid.
#[inline]
unsafe fn process_end_invisible(
    state_c: c_int,
    t: *const NfaThread,
    m: *mut RegSubs,
    nextlist_n: c_int,
    result: &mut StateProcessResult,
) -> bool {
    let nfa_endp = nvim_rex_get_nfa_endp();

    // Call the C wrapper which handles the complex endp comparison
    let check_result = nvim_nfa_check_end_invisible(
        state_c,
        &(*t).subs as *const _ as *const c_void,
        m as *mut c_void,
        nextlist_n,
        nfa_endp,
    );

    match check_result {
        0 => {
            // No match (break in C code)
            // result remains default (no add_state)
        }
        1 => {
            // Match found! Set nfa_match and goto nextchar
            nvim_nfa_set_match(1);
            result.return_code = 2; // goto nextchar
        }
        _ => {
            // Unexpected result - shouldn't happen
        }
    }

    true
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

/// Process position matching states (NFA_LNUM, NFA_COL, NFA_CURSOR).
///
/// Returns true if the state was handled, false if it should be handled by C.
///
/// # Safety
/// All pointers must be valid.
#[inline]
unsafe fn process_position(
    state_c: c_int,
    state: *mut NfaState,
    result: &mut StateProcessResult,
) -> bool {
    let matched = match state_c {
        NFA_LNUM | NFA_LNUM_GT | NFA_LNUM_LT => {
            // Only works in multi-line mode
            if nvim_rex_is_multi() == 0 {
                return true; // Handled but no match
            }
            let val = (*state).val as u64;
            let lnum = nvim_rex_get_lnum();
            let firstlnum = nvim_rex_get_reg_firstlnum();
            let pos = (lnum + firstlnum) as u64;
            let op = state_c - NFA_LNUM;
            nfa_re_num_cmp(val, op, pos)
        }
        NFA_COL | NFA_COL_GT | NFA_COL_LT => {
            let val = (*state).val as u64;
            let input = nvim_rex_get_input();
            let line = nvim_rex_get_line();
            let col = input.offset_from(line) as u64 + 1;
            let op = state_c - NFA_COL;
            nfa_re_num_cmp(val, op, col)
        }
        NFA_CURSOR => {
            // Check if we're at cursor position
            let reg_win = nvim_rex_get_reg_win();
            if reg_win.is_null() {
                false
            } else {
                let lnum = nvim_rex_get_lnum();
                let firstlnum = nvim_rex_get_reg_firstlnum();
                let cursor_lnum = nvim_rex_get_cursor_lnum();
                let input = nvim_rex_get_input();
                let line = nvim_rex_get_line();
                let col = input.offset_from(line) as c_int;
                let cursor_col = nvim_rex_get_cursor_col();
                lnum + firstlnum == cursor_lnum && col == cursor_col
            }
        }
        NFA_VCOL | NFA_VCOL_GT | NFA_VCOL_LT => {
            let val = (*state).val;
            let op = state_c - NFA_VCOL;
            nvim_nfa_check_vcol(val, op)
        }
        NFA_MARK | NFA_MARK_GT | NFA_MARK_LT => {
            let mark_id = (*state).val;
            let op = state_c - NFA_MARK;
            nvim_nfa_check_mark(mark_id, op)
        }
        NFA_VISUAL => nvim_nfa_check_visual(),
        _ => return false, // Not a position state we handle
    };

    if matched {
        result.add_here = 1;
        result.add_state = (*state).out;
    }
    true
}

/// Process backreference states (NFA_BACKREF1-9, NFA_ZREF1-9).
///
/// Returns true if the state was handled, false if it should be handled by C.
///
/// # Safety
/// All pointers must be valid.
#[inline]
unsafe fn process_backref(
    state_c: c_int,
    clen: c_int,
    t: *const NfaThread,
    state: *mut NfaState,
    result: &mut StateProcessResult,
) -> bool {
    let (bytelen, matched) = if (NFA_BACKREF1..=NFA_BACKREF9).contains(&state_c) {
        // Normal backreference \1..\9
        let subidx = state_c - NFA_BACKREF1 + 1;
        let mut bytelen: c_int = 0;
        let matched = nvim_nfa_match_backref(
            &(*t).subs.norm as *const _ as *const c_void,
            subidx,
            &mut bytelen,
        ) != 0;
        (bytelen, matched)
    } else if (NFA_ZREF1..=NFA_ZREF9).contains(&state_c) {
        // External submatch reference \z1..\z9
        let subidx = state_c - NFA_ZREF1 + 1;
        let mut bytelen: c_int = 0;
        let matched = nvim_nfa_match_zref(subidx, &mut bytelen) != 0;
        (bytelen, matched)
    } else {
        return false; // Not a backref state
    };

    if matched {
        if bytelen == 0 {
            // Empty match always works, output of NFA_SKIP to be used next
            result.add_here = 1;
            result.add_state = (*(*state).out).out;
        } else if bytelen <= clen {
            // Match current character, jump ahead to out of NFA_SKIP
            result.add_state = (*(*state).out).out;
            result.add_off = clen;
        } else {
            // Skip over matched characters, set character count in NFA_SKIP
            result.add_state = (*state).out;
            result.add_off = bytelen;
            result.add_count = bytelen - clen;
        }
    }
    true
}

/// Check if state_c is a _FIRST variant that requires direct processing
#[inline]
fn is_first_invisible_state(state_c: c_int) -> bool {
    state_c == NFA_START_INVISIBLE_FIRST
        || state_c == NFA_START_INVISIBLE_NEG_FIRST
        || state_c == NFA_START_INVISIBLE_BEFORE_FIRST
        || state_c == NFA_START_INVISIBLE_BEFORE_NEG_FIRST
}

/// Check if state_c is a negative (NEG) invisible state
#[inline]
fn is_neg_invisible_state(state_c: c_int) -> bool {
    state_c == NFA_START_INVISIBLE_NEG
        || state_c == NFA_START_INVISIBLE_NEG_FIRST
        || state_c == NFA_START_INVISIBLE_BEFORE_NEG
        || state_c == NFA_START_INVISIBLE_BEFORE_NEG_FIRST
}

/// Process NFA_START_INVISIBLE* states - lookahead/lookbehind matching.
///
/// This handles both:
/// - DIRECT EXECUTION case (when there's already a PIM or it's a _FIRST state)
/// - POSTPONED case (creates a new PIM and calls addstate_here)
///
/// Returns:
/// - true with add_state set: handled, add the state via state_handled
/// - true with add_state NULL and return_code 3: fully handled (addstate_here called)
/// - true with add_state NULL and return_code 0: handled but no state to add
/// - return_code -1: error (NFA_TOO_EXPENSIVE)
///
/// # Safety
/// All pointers must be valid.
#[inline]
#[allow(clippy::too_many_arguments)]
unsafe fn process_start_invisible(
    state_c: c_int,
    t: *const NfaThread,
    thislist: *mut NfaList,
    listidx: *mut c_int,
    prog: *mut c_void,
    submatch: *mut RegSubs,
    m: *mut RegSubs,
    listids: *mut *mut c_int,
    listids_len: *mut c_int,
    result: &mut StateProcessResult,
) -> bool {
    let state = (*t).state;

    // Check if we should execute directly or create a postponed match
    // Execute directly if:
    // 1. There already is a PIM (t->pim.result != NFA_PIM_UNUSED)
    // 2. The state is a _FIRST variant (nfa_postprocess detected it works better)
    let pim_unused = (*t).pim.result == NFA_PIM_UNUSED;
    let is_first = is_first_invisible_state(state_c);

    if pim_unused && !is_first {
        // POSTPONED case - create a PIM and call addstate_here
        // If listidx is null, we can't handle this case (called from old code path)
        if listidx.is_null() {
            // Let C handle it via the old code path
            return false;
        }
        // First try matching what follows. Only if a match is found
        // verify the invisible match matches.
        let mut pim = NfaPim {
            state,
            result: NFA_PIM_TODO,
            subs: RegSubs {
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
            },
            end: crate::nfa_states::PimEnd {
                ptr: ptr::null_mut(),
            },
        };

        // Set the end position
        if nvim_rex_is_multi() != 0 {
            let input = nvim_rex_get_input();
            let line = nvim_rex_get_line();
            pim.end.pos.col = (input as isize - line as isize) as ColNr;
            pim.end.pos.lnum = nvim_rex_get_lnum();
        } else {
            pim.end.ptr = nvim_rex_get_input();
        }

        // t->state->out1 is the corresponding END_INVISIBLE node;
        // Add its out to the current list (zero-width match).
        let r = addstate_here(
            thislist,
            (*(*state).out1).out,
            &(*t).subs as *const RegSubs as *mut RegSubs,
            &pim,
            listidx,
        );
        if r.is_null() {
            result.return_code = -1; // NFA_TOO_EXPENSIVE
            return true;
        }

        // State was fully handled via addstate_here, return code 3
        result.return_code = 3;
        return true;
    }

    // DIRECT EXECUTION case - call recursive_regmatch
    let in_use = (*m).norm.in_use;

    // Copy submatch info for the recursive call
    nvim_nfa_copy_sub_off(
        &mut (*m).norm as *mut _ as *mut c_void,
        &(*t).subs.norm as *const _ as *const c_void,
    );
    if nvim_rex_get_nfa_has_zsubexpr() != 0 {
        nvim_nfa_copy_sub_off(
            &mut (*m).synt as *mut _ as *mut c_void,
            &(*t).subs.synt as *const _ as *const c_void,
        );
    }

    // First try matching the invisible match, then what follows
    let recursive_result =
        recursive_regmatch(state, ptr::null(), prog, submatch, m, listids, listids_len);

    if recursive_result == NFA_TOO_EXPENSIVE {
        result.return_code = -1; // Signal error
        return true;
    }

    // For \@! and \@<! it is a match when the result is false
    let is_neg = is_neg_invisible_state(state_c);
    let matched = (recursive_result != 0) != is_neg;

    if matched {
        // Copy submatch info from the recursive call
        // Note: We cast t to *mut to modify subs, matching C behavior
        let t_mut = t as *mut NfaThread;
        nvim_nfa_copy_sub_off(
            &mut (*t_mut).subs.norm as *mut _ as *mut c_void,
            &(*m).norm as *const _ as *const c_void,
        );
        if nvim_rex_get_nfa_has_zsubexpr() != 0 {
            nvim_nfa_copy_sub_off(
                &mut (*t_mut).subs.synt as *mut _ as *mut c_void,
                &(*m).synt as *const _ as *const c_void,
            );
        }
        // If the pattern has \ze and it matched in the sub pattern, use it
        nvim_nfa_copy_ze_off(
            &mut (*t_mut).subs.norm as *mut _ as *mut c_void,
            &(*m).norm as *const _ as *const c_void,
        );

        // t->state->out1 is the corresponding END_INVISIBLE node;
        // Add its out to the current list (zero-width match)
        result.add_here = 1;
        result.add_state = (*(*state).out1).out;
    }

    (*m).norm.in_use = in_use;
    true
}

/// Process NFA_START_PATTERN state - \@> match.
///
/// Returns true if the state was handled, false to let C handle it.
///
/// # Safety
/// All pointers must be valid.
#[inline]
#[allow(clippy::too_many_arguments)]
unsafe fn process_start_pattern(
    t: *const NfaThread,
    clen: c_int,
    prog: *mut c_void,
    thislist: *mut NfaList,
    nextlist: *mut NfaList,
    submatch: *mut RegSubs,
    m: *mut RegSubs,
    listids: *mut *mut c_int,
    listids_len: *mut c_int,
    result: &mut StateProcessResult,
) -> bool {
    let state = (*t).state;

    // There is no point in trying to match the pattern if the
    // output state is not going to be added to the list.
    let skip = nvim_nfa_state_in_list(
        nextlist as *const c_void,
        (*(*state).out1).out as *const c_void,
        &(*t).subs as *const _ as *const c_void,
    ) != 0
        || nvim_nfa_state_in_list(
            nextlist as *const c_void,
            (*(*(*state).out1).out).out as *const c_void,
            &(*t).subs as *const _ as *const c_void,
        ) != 0
        || nvim_nfa_state_in_list(
            thislist as *const c_void,
            (*(*(*state).out1).out).out as *const c_void,
            &(*t).subs as *const _ as *const c_void,
        ) != 0;

    if skip {
        // Output state is already in list, skip matching
        // Return true with no add_state to indicate "handled but no state to add"
        return true;
    }

    // Copy submatch info to the recursive call
    nvim_nfa_copy_sub_off(
        &mut (*m).norm as *mut _ as *mut c_void,
        &(*t).subs.norm as *const _ as *const c_void,
    );
    if nvim_rex_get_nfa_has_zsubexpr() != 0 {
        nvim_nfa_copy_sub_off(
            &mut (*m).synt as *mut _ as *mut c_void,
            &(*t).subs.synt as *const _ as *const c_void,
        );
    }

    // First try matching the pattern
    let recursive_result =
        recursive_regmatch(state, ptr::null(), prog, submatch, m, listids, listids_len);

    if recursive_result == NFA_TOO_EXPENSIVE {
        result.return_code = -1;
        return true;
    }

    if recursive_result != 0 {
        // Copy submatch info from the recursive call
        let t_mut = t as *mut NfaThread;
        nvim_nfa_copy_sub_off(
            &mut (*t_mut).subs.norm as *mut _ as *mut c_void,
            &(*m).norm as *const _ as *const c_void,
        );
        if nvim_rex_get_nfa_has_zsubexpr() != 0 {
            nvim_nfa_copy_sub_off(
                &mut (*t_mut).subs.synt as *mut _ as *mut c_void,
                &(*m).synt as *const _ as *const c_void,
            );
        }

        // Now we need to skip over the matched text and then continue
        let bytelen = if nvim_rex_is_multi() != 0 {
            // Multi-line match
            (*m).norm.list.multi[0].end_col
                - (nvim_rex_get_input() as isize - nvim_rex_get_line() as isize) as ColNr
        } else {
            ((*m).norm.list.line[0].end as isize - nvim_rex_get_input() as isize) as c_int
        };

        if bytelen == 0 {
            // Empty match, output of corresponding NFA_END_PATTERN/NFA_SKIP
            // to be used at current position
            result.add_here = 1;
            result.add_state = (*(*(*state).out1).out).out;
        } else if bytelen <= clen {
            // Match current character, output of corresponding
            // NFA_END_PATTERN to be used at next position
            result.add_state = (*(*(*state).out1).out).out;
            result.add_off = clen;
        } else {
            // Skip over the matched characters, set character count in NFA_SKIP
            result.add_state = (*(*state).out1).out;
            result.add_off = bytelen;
            result.add_count = bytelen - clen;
        }
    }

    true
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
/// * `listidx` - Current list index (modified by addstate_here)
///
/// # Returns
/// A StateProcessResult containing:
/// * `add_state` - State to add, or NULL
/// * `add_here` - True if using addstate_here
/// * `add_count` - Count for NFA_SKIP
/// * `add_off` - Offset for addstate
/// * `return_code` - 0=continue, 2=goto nextchar, 3=state handled (skip state_handled), -1=error
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_process_state(
    t: *const NfaThread,
    curc: c_int,
    clen: c_int,
    prog: *mut c_void,
    thislist: *mut NfaList,
    nextlist: *mut NfaList,
    _start: *mut NfaState,
    submatch: *mut RegSubs,
    m: *mut RegSubs,
    listids: *mut *mut c_int,
    listids_len: *mut c_int,
    listidx: *mut c_int,
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
    } else if process_position(state_c, state, &mut result) {
        // Handled by position matching (NFA_LNUM, NFA_COL, NFA_CURSOR)
    } else if process_backref(state_c, clen, t, state, &mut result) {
        // Handled by backreference matching (NFA_BACKREF1-9, NFA_ZREF1-9)
    } else {
        // Handle other state types
        match state_c {
            NFA_MATCH => {
                process_match(curc, t, submatch, nextlist, &mut result);
            }
            NFA_ANY => process_any(curc, clen, state, &mut result),
            NFA_ANY_COMPOSING => process_any_composing(curc, clen, state, &mut result),
            NFA_COMPOSING => {
                process_composing(curc, clen, state, &mut result);
            }
            NFA_START_COLL | NFA_START_NEG_COLL => {
                process_collection(state_c, curc, clen, state, &mut result);
            }
            NFA_END_INVISIBLE | NFA_END_INVISIBLE_NEG | NFA_END_PATTERN => {
                process_end_invisible(state_c, t, m, (*nextlist).n, &mut result);
            }
            NFA_NEWL => process_newl(curc, state, &mut result),
            NFA_SKIP => process_skip(t, clen, state, &mut result),
            // Invisible states - handle both direct and postponed cases
            NFA_START_INVISIBLE
            | NFA_START_INVISIBLE_FIRST
            | NFA_START_INVISIBLE_NEG
            | NFA_START_INVISIBLE_NEG_FIRST
            | NFA_START_INVISIBLE_BEFORE
            | NFA_START_INVISIBLE_BEFORE_FIRST
            | NFA_START_INVISIBLE_BEFORE_NEG
            | NFA_START_INVISIBLE_BEFORE_NEG_FIRST => {
                process_start_invisible(
                    state_c,
                    t,
                    thislist,
                    listidx,
                    prog,
                    submatch,
                    m,
                    listids,
                    listids_len,
                    &mut result,
                );
            }
            NFA_START_PATTERN => {
                process_start_pattern(
                    t,
                    clen,
                    prog,
                    thislist,
                    nextlist,
                    submatch,
                    m,
                    listids,
                    listids_len,
                    &mut result,
                );
            }
            _ => {
                // Try literal character matching (positive state values)
                if !process_literal(state_c, curc, clen, state, &mut result) {
                    // Not a literal character, return 0 to continue with C fallback
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

// =============================================================================
// Phase 12a: NFA Execution Wrapper Helpers
// =============================================================================

/// OK/FAIL return values matching C definitions.
const OK: c_int = 1;
const FAIL: c_int = 0;

#[allow(clashing_extern_declarations)]
extern "C" {
    /// Get the length of a UTF-8 character.
    fn utf_char2len(c: c_int) -> c_int;

    /// Cleanup subexpressions if needed.
    fn nvim_cleanup_subexpr();

    /// Check if timeout has been reached.
    fn profile_passed_limit(tm: ProfTime) -> c_int;
}

// Import rs_cstrchr from helpers module
use crate::helpers::rs_cstrchr;

/// Opaque proftime_T type from C.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProfTime {
    _data: [u8; 8], // sizeof(proftime_T) is typically 8 bytes (int64_t or struct timespec)
}

/// Skip until the character 'c' we know a match must start with.
///
/// Searches rex.line starting at column *colp for character c.
/// Updates *colp to point to the found character.
///
/// Returns OK if found, FAIL otherwise.
///
/// # Safety
/// colp must be a valid pointer. rex.line must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_to_start(c: c_int, colp: *mut ColNr) -> c_int {
    if colp.is_null() {
        return FAIL;
    }

    let line = nvim_rex_get_line();
    if line.is_null() {
        return FAIL;
    }

    let start = line.add(*colp as usize);
    let found = rs_cstrchr(start as *const c_char, c);

    if found.is_null() {
        return FAIL;
    }

    *colp = (found as usize - line as usize) as ColNr;
    OK
}

/// Check for a match with match_text.
///
/// Called after skip_to_start() has found regstart.
/// This is a fast-path for literal text matching.
///
/// Returns 0 for no match, 1 for a match.
/// Updates *startcol to the match start column.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_find_match_text(
    startcol: *mut ColNr,
    regstart: c_int,
    match_text: *const u8,
) -> c_int {
    if startcol.is_null() || match_text.is_null() {
        return 0;
    }

    let mut col = *startcol;
    let regstart_len = utf_char2len(regstart);

    loop {
        let line = nvim_rex_get_line();
        if line.is_null() {
            break;
        }

        let mut is_match = true;
        let mut s1 = match_text;

        // Calculate actual regstart_len - may differ due to case-folding
        let mut regstart_len2 = regstart_len;
        if regstart_len2 > 1 {
            let ptr_at_col = line.add(col as usize);
            let actual_len = utf_ptr2len(ptr_at_col as *const c_char);
            if actual_len != regstart_len2 {
                // Case-folding may have changed the byte length
                regstart_len2 = utf_char2len(utf_fold(regstart));
            }
        }

        // s2 points past the regstart character
        let mut s2 = line.add((col + regstart_len2) as usize);

        // Compare match_text with text after regstart
        while *s1 != 0 {
            let c1_len = utf_ptr2len(s1 as *const c_char);
            let c1 = utf_ptr2char(s1 as *const c_char);
            let c2_len = utf_ptr2len(s2 as *const c_char);
            let c2 = utf_ptr2char(s2 as *const c_char);

            // Check if characters match (case-insensitive if rex.reg_ic)
            if c1 != c2 && (!nvim_rex_get_reg_ic() || utf_fold(c1) != utf_fold(c2)) {
                is_match = false;
                break;
            }
            s1 = s1.add(c1_len as usize);
            s2 = s2.add(c2_len as usize);
        }

        // Check that no composing character follows (for a proper match boundary)
        if is_match && utf_iscomposing_legacy(utf_ptr2char(s2 as *const c_char)) == 0 {
            nvim_cleanup_subexpr();

            if nvim_rex_is_multi() != 0 {
                // Multi-line: set startpos and endpos
                let lnum = nvim_rex_get_lnum();
                let startpos = nvim_rex_get_reg_startpos();
                let endpos = nvim_rex_get_reg_endpos();
                if !startpos.is_null() && !endpos.is_null() {
                    // startpos[0] and endpos[0]
                    let sp = startpos as *mut LPos;
                    let ep = endpos as *mut LPos;
                    (*sp).lnum = lnum;
                    (*sp).col = col;
                    (*ep).lnum = lnum;
                    (*ep).col = (s2 as usize - line as usize) as ColNr;
                }
            } else {
                // Single-line: set startp and endp
                let startp = nvim_rex_get_reg_startp();
                let endp = nvim_rex_get_reg_endp();
                if !startp.is_null() && !endp.is_null() {
                    *startp = line.add(col as usize);
                    *endp = s2;
                }
            }

            *startcol = col;
            return 1;
        }

        // Try finding regstart after the current match
        col += regstart_len; // skip regstart
        if rs_skip_to_start(regstart, &mut col) == FAIL {
            break;
        }
    }

    *startcol = col;
    0
}

/// Check if NFA execution timed out.
///
/// This is the Rust implementation of nfa_did_time_out().
/// Checks if the time limit has been reached.
///
/// Returns true if timed out, false otherwise.
///
/// # Safety
/// Must be called with valid nfa_time_limit if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_did_time_out() -> c_int {
    let time_limit = nvim_nfa_get_time_limit();
    if time_limit.is_null() {
        return 0;
    }

    if profile_passed_limit(*time_limit) != 0 {
        let timed_out = nvim_nfa_get_timed_out_ptr();
        if !timed_out.is_null() {
            *timed_out = 1;
        }
        return 1;
    }

    0
}

extern "C" {
    /// Get nfa_time_limit pointer.
    fn nvim_nfa_get_time_limit() -> *mut ProfTime;

    /// Get nfa_timed_out pointer.
    fn nvim_nfa_get_timed_out_ptr() -> *mut c_int;

    /// Get rex.reg_startp array.
    fn nvim_rex_get_reg_startp() -> *mut *mut u8;

    /// Get rex.reg_endp array.
    fn nvim_rex_get_reg_endp() -> *mut *mut u8;

    /// Get rex.reg_startpos array.
    fn nvim_rex_get_reg_startpos() -> *mut c_void;

    /// Get rex.reg_endpos array.
    fn nvim_rex_get_reg_endpos() -> *mut c_void;

    /// Set nfa_time_limit pointer.
    fn nvim_nfa_set_time_limit(p: *mut ProfTime);

    /// Set nfa_timed_out pointer.
    fn nvim_nfa_set_timed_out_ptr(p: *mut c_int);

    /// Set nfa_time_count.
    fn nvim_nfa_set_time_count(v: c_int);

    /// Get rex.reg_mmatch.
    fn nvim_rex_get_reg_mmatch() -> *mut c_void;

    /// Get regmmatch_T.rmm_matchcol accessor - sets the field.
    fn nvim_regmmatch_set_rmm_matchcol(m: *mut c_void, col: ColNr);

    /// Clear submatch info.
    fn rs_clear_sub(sub: *mut c_void);

    /// Handle extmatch results for \z() patterns (Phase 12b helper).
    /// This packages found \z(...\) matches for export.
    fn nvim_nfa_handle_extmatch(prog: *mut c_void, subs_synt: *const c_void);
}

// =============================================================================
// Phase 12b: nfa_regtry migration
// =============================================================================

/// Try match of "prog" at rex.line["col"].
///
/// This is the Rust implementation of nfa_regtry().
///
/// @param prog       NFA program
/// @param col        Column to start matching
/// @param tm         Timeout limit or NULL
/// @param timed_out  Flag set on timeout or NULL
///
/// @return <= 0 for failure, number of lines contained in the match otherwise.
///
/// # Safety
/// All pointers must be valid. prog must be a valid nfa_regprog_T*.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_regtry(
    prog: *mut c_void,
    col: ColNr,
    tm: *mut ProfTime,
    timed_out: *mut c_int,
) -> c_int {
    if prog.is_null() {
        return 0;
    }

    // Set up rex.input and timing
    let line = nvim_rex_get_line();
    nvim_rex_set_input(line.add(col as usize));
    nvim_nfa_set_time_limit(tm);
    nvim_nfa_set_timed_out_ptr(timed_out);
    nvim_nfa_set_time_count(0);

    // Get start state from prog
    let start = nvim_nfa_regprog_get_start(prog);

    // Initialize submatch structures
    let mut subs = RegSubs::default();
    let mut m = RegSubs::default();

    // Clear submatch info
    rs_clear_sub(&mut subs.norm as *mut _ as *mut c_void);
    rs_clear_sub(&mut m.norm as *mut _ as *mut c_void);
    rs_clear_sub(&mut subs.synt as *mut _ as *mut c_void);
    rs_clear_sub(&mut m.synt as *mut _ as *mut c_void);

    // Run the NFA match
    let result = nvim_nfa_regmatch(
        prog,
        start,
        &mut subs as *mut _ as *mut c_void,
        &mut m as *mut _ as *mut c_void,
    );

    if result == 0 {
        return 0;
    } else if result == NFA_TOO_EXPENSIVE {
        return result;
    }

    // Copy results to rex state
    nvim_cleanup_subexpr();

    let is_multi = nvim_rex_is_multi() != 0;

    if is_multi {
        // Multi-line mode: copy to reg_startpos/reg_endpos
        let startpos = nvim_rex_get_reg_startpos() as *mut LPos;
        let endpos = nvim_rex_get_reg_endpos() as *mut LPos;

        for i in 0..subs.norm.in_use as usize {
            let sp = startpos.add(i);
            let ep = endpos.add(i);
            (*sp).lnum = subs.norm.list.multi[i].start_lnum;
            (*sp).col = subs.norm.list.multi[i].start_col;
            (*ep).lnum = subs.norm.list.multi[i].end_lnum;
            (*ep).col = subs.norm.list.multi[i].end_col;
        }

        // Set rmm_matchcol
        let mmatch = nvim_rex_get_reg_mmatch();
        if !mmatch.is_null() {
            nvim_regmmatch_set_rmm_matchcol(mmatch, subs.norm.orig_start_col);
        }

        // Fix up startpos[0] if it wasn't set
        let sp0 = startpos;
        if (*sp0).lnum < 0 {
            (*sp0).lnum = 0;
            (*sp0).col = col;
        }

        // Fix up endpos[0] if it wasn't set (pattern has \ze but didn't match)
        let ep0 = endpos;
        if (*ep0).lnum < 0 {
            (*ep0).lnum = nvim_rex_get_lnum();
            (*ep0).col = (nvim_rex_get_input() as usize - line as usize) as ColNr;
        } else {
            // Use line number of "\ze"
            nvim_rex_set_lnum((*ep0).lnum);
        }
    } else {
        // Single-line mode: copy to reg_startp/reg_endp
        let startp = nvim_rex_get_reg_startp();
        let endp = nvim_rex_get_reg_endp();

        for i in 0..subs.norm.in_use as usize {
            *startp.add(i) = subs.norm.list.line[i].start;
            *endp.add(i) = subs.norm.list.line[i].end;
        }

        // Fix up startp[0] if NULL
        if (*startp).is_null() {
            *startp = line.add(col as usize);
        }

        // Fix up endp[0] if NULL
        if (*endp).is_null() {
            *endp = nvim_rex_get_input();
        }
    }

    // Handle \z() external matches - delegate to C for memory management
    nvim_nfa_handle_extmatch(prog, &subs.synt as *const _ as *const c_void);

    1 + nvim_rex_get_lnum()
}

#[allow(clashing_extern_declarations)]
extern "C" {
    /// Get NFA program start state.
    fn nvim_nfa_regprog_get_start(prog: *const c_void) -> *mut c_void;

    /// Get regflags from program.
    fn nvim_regprog_get_regflags(prog: RegprogHandle) -> c_int;

    /// Get reganch from NFA program.
    fn nvim_nfa_regprog_get_reganch(prog: *const c_void) -> c_int;

    /// Get regstart from NFA program.
    fn nvim_nfa_regprog_get_regstart(prog: *const c_void) -> c_int;

    /// Get match_text from NFA program.
    fn nvim_nfa_regprog_get_match_text(prog: *const c_void) -> *const u8;

    /// Get has_zend from NFA program.
    fn nvim_nfa_regprog_get_has_zend(prog: *const c_void) -> c_int;

    /// Get has_backref from NFA program.
    fn nvim_nfa_regprog_get_has_backref(prog: *const c_void) -> c_int;

    /// Get nsubexp from NFA program.
    fn nvim_nfa_regprog_get_nsubexp(prog: *const c_void) -> c_int;

    /// Get reghasz from NFA program.
    fn nvim_nfa_regprog_get_reghasz(prog: *const c_void) -> c_int;

    /// Get regprog from regmatch_T.
    fn nvim_regmatch_get_regprog(m: *mut c_void) -> *mut c_void;

    /// Get startp from regmatch_T.
    fn nvim_regmatch_get_startp(m: *mut c_void) -> *mut *mut u8;

    /// Get endp from regmatch_T.
    fn nvim_regmatch_get_endp(m: *mut c_void) -> *mut *mut u8;

    /// Set rm_matchcol in regmatch_T.
    fn nvim_regmatch_set_rm_matchcol(m: *mut c_void, col: ColNr);

    /// Get regprog from regmmatch_T.
    fn nvim_regmmatch_get_regprog(m: *mut c_void) -> *mut c_void;

    /// Get startpos from regmmatch_T.
    fn nvim_regmmatch_get_startpos(m: *mut c_void) -> *mut c_void;

    /// Get endpos from regmmatch_T.
    fn nvim_regmmatch_get_endpos(m: *mut c_void) -> *mut c_void;

    /// Set global nstate.
    fn nvim_set_nstate(v: c_int);

    /// Initialize NFA program states for execution.
    fn nvim_nfa_init_prog_states(prog: *mut c_void);

    /// Report internal error.
    fn iemsg(s: *const c_char);
}

// RF_ flags for regflags
const RF_ICASE: c_int = 1;
const RF_NOICASE: c_int = 2;
const RF_ICOMBINE: c_int = 8;
const REX_SET: c_int = 1;

// =============================================================================
// Phase 12c: nfa_regexec_both migration
// =============================================================================

/// Match a regexp against a string ("line" points to the string) or multiple
/// lines (if "line" is NULL, use reg_getline()).
///
/// This is the Rust implementation of nfa_regexec_both().
///
/// # Arguments
/// * `line` - String to match or NULL for multiline
/// * `startcol` - Column to start looking for match
/// * `tm` - Timeout limit or NULL
/// * `timed_out` - Flag set on timeout or NULL
///
/// # Returns
/// <= 0 if no match, number of lines contained in the match otherwise.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_regexec_both(
    mut line: *mut u8,
    startcol: ColNr,
    tm: *mut ProfTime,
    timed_out: *mut c_int,
) -> c_int {
    let prog: *mut c_void;
    let mut col = startcol;
    let is_multi = nvim_rex_is_multi() != 0;

    // Get the program and set up pointers
    if is_multi {
        let mmatch = nvim_rex_get_reg_mmatch();
        prog = nvim_regmmatch_get_regprog(mmatch);
        line = nvim_reg_getline(0) as *mut u8; // relative to the cursor
        let startpos = nvim_regmmatch_get_startpos(mmatch);
        let endpos = nvim_regmmatch_get_endpos(mmatch);
        nvim_rex_set_reg_startpos(startpos);
        nvim_rex_set_reg_endpos(endpos);
    } else {
        let match_ = nvim_rex_get_reg_match();
        prog = nvim_regmatch_get_regprog(match_);
        let startp = nvim_regmatch_get_startp(match_);
        let endp = nvim_regmatch_get_endp(match_);
        nvim_rex_set_reg_startp(startp);
        nvim_rex_set_reg_endp(endp);
    }

    // Be paranoid...
    if prog.is_null() || line.is_null() {
        // Using a static string for e_null error
        static E_NULL: &[u8] = b"E685: Internal error: NULL\0";
        iemsg(E_NULL.as_ptr() as *const c_char);
        return finalize_match_result(0, col, is_multi);
    }

    // Get regflags and check for case sensitivity overrides
    let regflags = nvim_regprog_get_regflags(RegprogHandle(prog));

    if regflags & RF_ICASE != 0 {
        nvim_rex_set_reg_ic(true);
    } else if regflags & RF_NOICASE != 0 {
        nvim_rex_set_reg_ic(false);
    }

    // If pattern contains "\Z" overrule value of rex.reg_icombine
    if regflags & RF_ICOMBINE != 0 {
        nvim_rex_set_reg_icombine(true);
    }

    nvim_rex_set_line(line);
    nvim_rex_set_lnum(0); // relative to line

    // Set up NFA execution state
    nvim_rex_set_nfa_has_zend(nvim_nfa_regprog_get_has_zend(prog));
    nvim_rex_set_nfa_has_backref(nvim_nfa_regprog_get_has_backref(prog));
    nvim_rex_set_nfa_nsubexpr(nvim_nfa_regprog_get_nsubexp(prog));
    nvim_rex_set_nfa_listid(1);
    nvim_rex_set_nfa_alt_listid(2);

    // Check for anchored pattern at col > 0
    let reganch = nvim_nfa_regprog_get_reganch(prog);
    if reganch != 0 && col > 0 {
        return 0;
    }

    nvim_rex_set_need_clear_subexpr(1);

    // Clear the external match subpointers if necessary
    let reghasz = nvim_nfa_regprog_get_reghasz(prog);
    if reghasz == REX_SET {
        nvim_rex_set_nfa_has_zsubexpr(1);
        nvim_rex_set_need_clear_zsubexpr(1);
    } else {
        nvim_rex_set_nfa_has_zsubexpr(0);
        nvim_rex_set_need_clear_zsubexpr(0);
    }

    let regstart = nvim_nfa_regprog_get_regstart(prog);
    if regstart != 0 {
        // Skip ahead until a character we know the match must start with.
        // When there is none there is no match.
        if rs_skip_to_start(regstart, &mut col) == FAIL {
            return 0;
        }

        // If match_text is set it contains the full text that must match.
        // Nothing else to try. Doesn't handle combining chars well.
        let match_text = nvim_nfa_regprog_get_match_text(prog);
        if !match_text.is_null() && *match_text != 0 && !nvim_rex_get_reg_icombine() {
            let retval = rs_find_match_text(&mut col, regstart, match_text);
            if is_multi {
                let mmatch = nvim_rex_get_reg_mmatch();
                nvim_regmmatch_set_rmm_matchcol(mmatch, col);
            } else {
                let match_ = nvim_rex_get_reg_match();
                nvim_regmatch_set_rm_matchcol(match_, col);
            }
            return retval;
        }
    }

    // If the start column is past the maximum column: no need to try.
    let maxcol = nvim_rex_get_reg_maxcol();
    if maxcol > 0 && col >= maxcol {
        return finalize_match_result(0, col, is_multi);
    }

    // Set the "nstate" used by nfa_regcomp() to zero to trigger an error when
    // it's accidentally used during execution.
    nvim_set_nstate(0);

    // Initialize NFA program states
    nvim_nfa_init_prog_states(prog);

    // Call nfa_regtry
    let retval = rs_nfa_regtry(prog, col, tm, timed_out);

    finalize_match_result(retval, col, is_multi)
}

/// Finalize match result - ensure end is never before start.
///
/// # Safety
/// Assumes rex state is valid.
#[inline]
unsafe fn finalize_match_result(retval: c_int, col: ColNr, is_multi: bool) -> c_int {
    if retval > 0 {
        // Make sure the end is never before the start. Can happen when \zs and
        // \ze are used.
        if is_multi {
            let mmatch = nvim_rex_get_reg_mmatch();
            let startpos = nvim_regmmatch_get_startpos(mmatch) as *const LPos;
            let endpos = nvim_regmmatch_get_endpos(mmatch) as *mut LPos;

            let start = &*startpos;
            let end = &mut *endpos;

            if end.lnum < start.lnum || (end.lnum == start.lnum && end.col < start.col) {
                *end = *start;
            }
        } else {
            let match_ = nvim_rex_get_reg_match();
            let startp = nvim_regmatch_get_startp(match_);
            let endp = nvim_regmatch_get_endp(match_);

            if !startp.is_null() && !endp.is_null() {
                if (*endp as usize) < (*startp as usize) {
                    *endp = *startp;
                }

                // startpos[0] may be set by "\zs", also return the column where
                // the whole pattern matched.
                nvim_regmatch_set_rm_matchcol(match_, col);
            }
        }
    }

    retval
}

extern "C" {
    /// Set rex.reg_startp.
    fn nvim_rex_set_reg_startp(p: *mut *mut u8);

    /// Set rex.reg_endp.
    fn nvim_rex_set_reg_endp(p: *mut *mut u8);

    /// Set rex.reg_startpos.
    fn nvim_rex_set_reg_startpos(p: *mut c_void);

    /// Set rex.reg_endpos.
    fn nvim_rex_set_reg_endpos(p: *mut c_void);

    /// Get rex.reg_match.
    fn nvim_rex_get_reg_match() -> *mut c_void;

    /// Set rex.reg_ic.
    fn nvim_rex_set_reg_ic(ic: bool);

    /// Set rex.reg_icombine.
    fn nvim_rex_set_reg_icombine(v: bool);

    /// Set rex.nfa_has_zend.
    fn nvim_rex_set_nfa_has_zend(v: c_int);

    /// Set rex.nfa_has_backref.
    fn nvim_rex_set_nfa_has_backref(v: c_int);

    /// Set rex.nfa_nsubexpr.
    fn nvim_rex_set_nfa_nsubexpr(v: c_int);

    /// Get rex.reg_maxcol.
    fn nvim_rex_get_reg_maxcol() -> ColNr;

    /// Set rex.need_clear_subexpr.
    fn nvim_rex_set_need_clear_subexpr(v: c_int);

    /// Set rex.nfa_has_zsubexpr.
    fn nvim_rex_set_nfa_has_zsubexpr(v: c_int);

    /// Set rex.need_clear_zsubexpr.
    fn nvim_rex_set_need_clear_zsubexpr(v: c_int);
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
