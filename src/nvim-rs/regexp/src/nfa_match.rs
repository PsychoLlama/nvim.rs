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
    LPos, NfaList, NfaPim, NfaState, NfaThread, RegSubs, NFA_MATCH, NFA_PIM_MATCH, NFA_PIM_NOMATCH,
    NFA_PIM_TODO, NFA_PIM_UNUSED, NFA_SPLIT,
};

// =============================================================================
// FFI declarations for C functions
// =============================================================================

extern "C" {
    fn nvim_rex_is_multi() -> c_int;
    fn nvim_rex_get_nfa_has_zend() -> c_int;
    fn nvim_rex_get_nfa_nsubexpr() -> c_int;
}

// =============================================================================
// Match Constants
// =============================================================================

/// Maximum recursion depth for addstate to prevent stack overflow.
pub const MAX_ADDSTATE_DEPTH: c_int = 5000;

/// Offset used by addstate_here to signal insertion at specific position.
pub const ADDSTATE_HERE_OFFSET: c_int = 1000;

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
/// Matches C behavior: only copies in_use entries, not the whole union.
///
/// # Safety
/// Both pointers must be valid.
pub unsafe fn copy_sub(to: *mut crate::nfa_states::RegSub, from: *const crate::nfa_states::RegSub) {
    if to.is_null() || from.is_null() {
        return;
    }
    (*to).in_use = (*from).in_use;
    if (*from).in_use <= 0 {
        return;
    }

    // Copy only the match start and end positions that are in use
    let in_use = (*from).in_use as usize;
    if nvim_rex_is_multi() != 0 {
        ptr::copy_nonoverlapping(
            (*from).list.multi.as_ptr(),
            (*to).list.multi.as_mut_ptr(),
            in_use,
        );
        (*to).orig_start_col = (*from).orig_start_col;
    } else {
        ptr::copy_nonoverlapping(
            (*from).list.line.as_ptr(),
            (*to).list.line.as_mut_ptr(),
            in_use,
        );
    }
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

extern "C" {
    fn nvim_rex_get_nfa_has_zsubexpr() -> c_int;
}

/// Copy PIM (Postponed Invisible Match) info.
///
/// Matches C behavior: copies result, state, subs (conditionally for synt), and end.
///
/// # Safety
/// Both pointers must be valid.
pub unsafe fn copy_pim(to: *mut NfaPim, from: *const NfaPim) {
    if to.is_null() || from.is_null() {
        return;
    }
    (*to).result = (*from).result;
    (*to).state = (*from).state;
    copy_sub(&mut (*to).subs.norm, &(*from).subs.norm);
    if nvim_rex_get_nfa_has_zsubexpr() != 0 {
        copy_sub(&mut (*to).subs.synt, &(*from).subs.synt);
    }
    (*to).end = (*from).end;
}

/// Clear submatch positions.
///
/// Matches C behavior: for multi-line mode, fills list.multi with 0xff
/// (setting lnum to -1); for single-line mode, fills list.line with 0.
///
/// # Safety
/// Pointer must be valid.
pub unsafe fn clear_sub(sub: *mut crate::nfa_states::RegSub) {
    if sub.is_null() {
        return;
    }
    let nsubexpr = nvim_rex_get_nfa_nsubexpr() as usize;
    if nvim_rex_is_multi() != 0 {
        // Use 0xff to set lnum to -1
        let multi_ptr = (*sub).list.multi.as_mut_ptr() as *mut u8;
        let multi_size = std::mem::size_of::<crate::nfa_states::MultiPos>() * nsubexpr;
        ptr::write_bytes(multi_ptr, 0xff, multi_size);
    } else {
        // Zero out line positions
        let line_ptr = (*sub).list.line.as_mut_ptr() as *mut u8;
        let line_size = std::mem::size_of::<crate::nfa_states::LinePos>() * nsubexpr;
        ptr::write_bytes(line_ptr, 0, line_size);
    }
    (*sub).in_use = 0;
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
// FFI Exports
// =============================================================================

/// Create a new match context for single-line matching.
///
/// # Safety
/// `line` must point to valid memory of at least `col + 1` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_match_context_new(line: *const u8, col: c_int) -> MatchContext {
    MatchContext::new_single_line(line, col)
}

/// Create a new match context for multi-line matching.
///
/// # Safety
/// `line` must point to valid memory of at least `col + 1` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_match_context_new_multi(
    line: *const u8,
    col: c_int,
    lnum: c_int,
    first_lnum: c_int,
    max_line: c_int,
) -> MatchContext {
    MatchContext::new_multi_line(line, col, lnum, first_lnum, max_line)
}

/// Get the current byte at the match context's input position.
///
/// # Safety
/// Context's input must point to valid memory.
#[no_mangle]
pub unsafe extern "C" fn rs_match_context_current_byte(ctx: *const MatchContext) -> u8 {
    if ctx.is_null() {
        0
    } else {
        (*ctx).current_byte()
    }
}

/// Check if the match context is at end of line.
///
/// # Safety
/// Context's input must point to valid memory.
#[no_mangle]
pub unsafe extern "C" fn rs_match_context_at_eol(ctx: *const MatchContext) -> c_int {
    if ctx.is_null() {
        1
    } else {
        c_int::from((*ctx).at_eol())
    }
}

/// Advance the match context's input position.
///
/// # Safety
/// Must not advance past end of string.
#[no_mangle]
pub unsafe extern "C" fn rs_match_context_advance(ctx: *mut MatchContext, n: c_int) {
    if !ctx.is_null() && n > 0 {
        (*ctx).advance(n as usize);
    }
}

/// Check if a state is in the list.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_state_in_list(
    list: *const NfaList,
    state: *const NfaState,
    listid: c_int,
) -> c_int {
    c_int::from(state_in_list(list, state, listid))
}

/// Mark a state as being in a list.
///
/// # Safety
/// State must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_state_in_list(state: *mut NfaState, listid: c_int) {
    mark_state_in_list(state, listid);
}

/// Initialize a thread.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_init_thread(
    thread: *mut NfaThread,
    state: *mut NfaState,
    subs: *const RegSubs,
    pim: *const NfaPim,
) {
    init_thread(thread, state, subs, pim);
}

/// Copy submatch info.
///
/// # Safety
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_copy_subs(to: *mut RegSubs, from: *const RegSubs, has_zsubexpr: c_int) {
    copy_subs(to, from, has_zsubexpr != 0);
}

/// Copy PIM info.
///
/// # Safety
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_copy_pim(to: *mut NfaPim, from: *const NfaPim) {
    copy_pim(to, from);
}

/// Clear submatch info.
///
/// # Safety
/// Pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_subs(subs: *mut RegSubs) {
    clear_subs(subs);
}

/// Check if list has a match state.
///
/// # Safety
/// List must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_list_has_match(list: *const NfaList) -> c_int {
    c_int::from(list_has_match(list))
}

/// Check if a state is a match state.
///
/// # Safety
/// If state is non-null, it must point to a valid NfaState.
#[no_mangle]
pub unsafe extern "C" fn rs_is_match_state(state: *const NfaState) -> c_int {
    c_int::from(is_match_state(state))
}

/// Check if a state is a split state.
///
/// # Safety
/// If state is non-null, it must point to a valid NfaState.
#[no_mangle]
pub unsafe extern "C" fn rs_is_split_state(state: *const NfaState) -> c_int {
    c_int::from(is_split_state(state))
}

/// Check if PIM needs execution.
///
/// # Safety
/// If pim is non-null, it must point to a valid NfaPim.
#[no_mangle]
pub unsafe extern "C" fn rs_pim_needs_exec(pim: *const NfaPim) -> c_int {
    c_int::from(pim_needs_exec(pim))
}

/// Check if PIM matched.
///
/// # Safety
/// If pim is non-null, it must point to a valid NfaPim.
#[no_mangle]
pub unsafe extern "C" fn rs_pim_matched(pim: *const NfaPim) -> c_int {
    c_int::from(pim_matched(pim))
}

/// Set PIM as matched.
///
/// # Safety
/// Pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_set_pim_matched(pim: *mut NfaPim) {
    set_pim_matched(pim);
}

/// Set PIM as not matched.
///
/// # Safety
/// Pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_set_pim_nomatch(pim: *mut NfaPim) {
    set_pim_nomatch(pim);
}

/// Copy submatch positions, excluding main match (index 0).
///
/// # Safety
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_copy_sub_off(
    to: *mut crate::nfa_states::RegSub,
    from: *const crate::nfa_states::RegSub,
) {
    copy_sub_off(to, from);
}

/// Copy end position of main match if \ze was used.
///
/// # Safety
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_copy_ze_off(
    to: *mut crate::nfa_states::RegSub,
    from: *const crate::nfa_states::RegSub,
) {
    copy_ze_off(to, from);
}

/// Copy submatch positions from one RegSub to another.
///
/// # Safety
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_copy_sub(
    to: *mut crate::nfa_states::RegSub,
    from: *const crate::nfa_states::RegSub,
) {
    copy_sub(to, from);
}

/// Clear submatch positions in a RegSub.
///
/// # Safety
/// Pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_sub(sub: *mut crate::nfa_states::RegSub) {
    clear_sub(sub);
}

// =============================================================================
// Position Matching Functions
// =============================================================================

extern "C" {
    /// Get character class from C.
    fn mb_get_class_tab(ptr: *const u8, chartab: *const u64) -> c_int;
    /// Get previous character's class (wrapper around static reg_prev_class).
    fn nvim_rex_reg_prev_class() -> c_int;
    /// Get the rex global state.
    fn nvim_rex_get_input() -> *mut u8;
    fn nvim_rex_get_line() -> *mut u8;
    fn nvim_rex_get_lnum() -> c_int;
    fn nvim_rex_get_reg_firstlnum() -> c_int;
    fn nvim_rex_get_reg_maxline() -> c_int;
    fn nvim_rex_get_reg_buf_chartab() -> *const u64;
}

/// Check if current position matches beginning of line (BOL).
///
/// Returns 1 if at beginning of line, 0 otherwise.
///
/// # Safety
/// Requires valid rex global state.
#[no_mangle]
pub unsafe extern "C" fn rs_check_bol() -> c_int {
    let input = nvim_rex_get_input();
    let line = nvim_rex_get_line();
    c_int::from(input == line)
}

/// Check if current position matches end of line (EOL).
///
/// Returns 1 if at end of line (NUL byte), 0 otherwise.
///
/// # Safety
/// Requires valid rex global state with `curc` being the current character.
#[no_mangle]
pub unsafe extern "C" fn rs_check_eol(curc: c_int) -> c_int {
    c_int::from(curc == 0)
}

/// Check if current position matches beginning of word (BOW).
///
/// A word boundary exists when the current character is a word character
/// and the previous character is not (or doesn't exist).
///
/// Returns 1 if at beginning of word, 0 otherwise.
///
/// # Safety
/// Requires valid rex global state.
#[no_mangle]
pub unsafe extern "C" fn rs_check_bow(curc: c_int) -> c_int {
    // If at EOL, not a BOW
    if curc == 0 {
        return 0;
    }

    let input = nvim_rex_get_input();
    let chartab = nvim_rex_get_reg_buf_chartab();

    // Get class of current character
    let this_class = mb_get_class_tab(input, chartab);

    // Classes 0 and 1 are whitespace/punctuation, not word characters
    if this_class <= 1 {
        return 0;
    }

    // Check if previous character is same class (not a word boundary)
    let prev_class = nvim_rex_reg_prev_class();
    if prev_class == this_class {
        return 0;
    }

    1
}

/// Check if current position matches end of word (EOW).
///
/// A word boundary exists when the previous character is a word character
/// and the current character is not (or we're at end of line).
///
/// Returns 1 if at end of word, 0 otherwise.
///
/// # Safety
/// Requires valid rex global state.
#[no_mangle]
pub unsafe extern "C" fn rs_check_eow() -> c_int {
    let input = nvim_rex_get_input();
    let line = nvim_rex_get_line();

    // If at start of line, not an EOW
    if input == line {
        return 0;
    }

    let chartab = nvim_rex_get_reg_buf_chartab();

    // Get class of current and previous characters
    let this_class = mb_get_class_tab(input, chartab);
    let prev_class = nvim_rex_reg_prev_class();

    // EOW if previous was a word char and current is not, or different class
    // Classes 0 and 1 are whitespace/punctuation
    if this_class == prev_class || prev_class == 0 || prev_class == 1 {
        return 0;
    }

    1
}

/// Check if current position matches beginning of file (BOF).
///
/// Returns 1 if at very beginning of buffer, 0 otherwise.
///
/// # Safety
/// Requires valid rex global state.
#[no_mangle]
pub unsafe extern "C" fn rs_check_bof() -> c_int {
    let input = nvim_rex_get_input();
    let line = nvim_rex_get_line();
    let lnum = nvim_rex_get_lnum();
    let first_lnum = nvim_rex_get_reg_firstlnum();
    let is_multi = nvim_rex_is_multi() != 0;

    // At BOF if: line 0, at start of line, and (not multi-line or first_lnum is 1)
    c_int::from(lnum == 0 && input == line && (!is_multi || first_lnum == 1))
}

/// Check if current position matches end of file (EOF).
///
/// Returns 1 if at very end of buffer, 0 otherwise.
///
/// # Safety
/// Requires valid rex global state.
#[no_mangle]
pub unsafe extern "C" fn rs_check_eof(curc: c_int) -> c_int {
    let lnum = nvim_rex_get_lnum();
    let max_line = nvim_rex_get_reg_maxline();

    // At EOF if: at last line and current char is NUL
    c_int::from(lnum == max_line && curc == 0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nfa_states::NSUBEXP;

    #[test]
    fn test_match_context_default() {
        let ctx = MatchContext::default();
        assert!(ctx.input.is_null());
        assert!(ctx.line.is_null());
        assert_eq!(ctx.lnum, 1);
        assert!(!ctx.multi);
    }

    #[test]
    fn test_match_context_single_line() {
        let line = b"hello world\0";
        // Safety: line is valid and col 6 is within bounds
        let ctx = unsafe { MatchContext::new_single_line(line.as_ptr(), 6) };
        assert!(!ctx.input.is_null());
        assert!(!ctx.line.is_null());
        assert!(!ctx.multi);

        unsafe {
            assert_eq!(ctx.current_byte(), b'w');
            assert_eq!(ctx.column(), 6);
            assert!(!ctx.at_eol());
        }
    }

    #[test]
    fn test_match_context_at_eol() {
        let line = b"hi\0";
        // Safety: line is valid and col 2 is within bounds (at the NUL terminator)
        let ctx = unsafe { MatchContext::new_single_line(line.as_ptr(), 2) };
        unsafe {
            assert_eq!(ctx.current_byte(), 0);
            assert!(ctx.at_eol());
        }
    }

    #[test]
    fn test_match_context_advance() {
        let line = b"hello\0";
        // Safety: line is valid and col 0 is within bounds
        let mut ctx = unsafe { MatchContext::new_single_line(line.as_ptr(), 0) };
        unsafe {
            assert_eq!(ctx.current_byte(), b'h');
            ctx.advance(2);
            assert_eq!(ctx.current_byte(), b'l');
            assert_eq!(ctx.column(), 2);
        }
    }

    #[test]
    fn test_match_context_position() {
        let ctx = MatchContext {
            input: ptr::null(),
            line: ptr::null(),
            lnum: 5,
            first_lnum: 1,
            max_line: 10,
            multi: true,
            ignore_case: false,
            ignore_combining: false,
            max_col: 0,
        };
        let pos = ctx.position();
        assert_eq!(pos.lnum, 5);
        assert_eq!(pos.col, 0);
    }

    #[test]
    fn test_match_result_conversion() {
        assert_eq!(MatchResult::Match as c_int, 1);
        assert_eq!(MatchResult::NoMatch as c_int, 0);
        assert_eq!(MatchResult::Error as c_int, -1);
        assert_eq!(MatchResult::Timeout as c_int, -2);

        assert_eq!(MatchResult::from(1), MatchResult::Match);
        assert_eq!(MatchResult::from(0), MatchResult::NoMatch);
        assert_eq!(MatchResult::from(-2), MatchResult::Timeout);
        assert_eq!(MatchResult::from(-99), MatchResult::Error);
    }

    #[test]
    fn test_is_match_state() {
        // Safety: passing null is explicitly handled by these functions
        unsafe {
            assert!(!is_match_state(ptr::null()));
        }
        // We can't easily test with a real NfaState without unsafe construction
    }

    #[test]
    fn test_pim_helpers() {
        // Safety: passing null is explicitly handled by these functions
        unsafe {
            assert!(!pim_needs_exec(ptr::null()));
            assert!(!pim_matched(ptr::null()));
            assert!(!pim_nomatch(ptr::null()));
        }
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_ADDSTATE_DEPTH, 5000);
        assert_eq!(ADDSTATE_HERE_OFFSET, 1000);
    }

    #[test]
    fn test_nsubexp() {
        assert_eq!(NSUBEXP, 10);
    }
}
