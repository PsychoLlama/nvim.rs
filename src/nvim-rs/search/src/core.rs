//! Core search logic
//!
//! This module provides Rust implementations and wrappers for core search
//! functionality from search.c. Due to the complex dependencies of searchit()
//! and do_search(), this module focuses on helper functions and state
//! management that can be cleanly migrated.

use std::ffi::{c_char, c_int};

use crate::direction;
use crate::helpers;
use crate::state;

// =============================================================================
// C External Functions
// =============================================================================

extern "C" {
    fn nvim_get_p_ws() -> c_int;
}

// =============================================================================
// Search Result Constants
// =============================================================================

/// Search succeeded
pub const SEARCH_OK: c_int = 1;
/// Search failed (pattern not found)
pub const SEARCH_FAIL: c_int = 0;
/// Search succeeded with line offset added
pub const SEARCH_OK_WITH_OFFSET: c_int = 2;

// =============================================================================
// Search Loop State
// =============================================================================

/// State for managing the search loop (wrap detection, etc.)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SearchLoopState {
    /// Whether we've wrapped around the buffer
    pub wrapped: bool,
    /// The starting line number
    pub start_lnum: i32,
    /// The starting column
    pub start_col: i32,
    /// Current loop iteration (0 or 1 for wrap)
    pub loop_iter: c_int,
    /// Number of matches found so far
    pub match_count: c_int,
}

impl Default for SearchLoopState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchLoopState {
    /// Create a new search loop state.
    pub const fn new() -> Self {
        Self {
            wrapped: false,
            start_lnum: 0,
            start_col: 0,
            loop_iter: 0,
            match_count: 0,
        }
    }

    /// Initialize with starting position.
    pub fn at_pos(lnum: i32, col: i32) -> Self {
        Self {
            wrapped: false,
            start_lnum: lnum,
            start_col: col,
            loop_iter: 0,
            match_count: 0,
        }
    }

    /// Mark that the search has wrapped around.
    pub fn mark_wrapped(&mut self) {
        self.wrapped = true;
        self.loop_iter = 1;
    }

    /// Increment the match count.
    pub fn found_match(&mut self) {
        self.match_count += 1;
    }

    /// Check if wrapscan should continue the loop.
    pub fn should_wrap(&self) -> bool {
        // SAFETY: Accessing global option
        let p_ws = unsafe { nvim_get_p_ws() != 0 };
        p_ws && self.loop_iter == 0
    }
}

// =============================================================================
// Search Direction Helpers
// =============================================================================

/// Compute the effective search direction considering SEARCH_REV option.
#[inline]
pub fn effective_direction(dirc: c_int, options: c_int) -> c_int {
    if helpers::has_search_rev(options) {
        // Reverse the direction
        if dirc == direction::DIR_FORWARD as c_int || dirc == b'/' as c_int {
            b'?' as c_int
        } else {
            b'/' as c_int
        }
    } else {
        dirc
    }
}

/// Check if a direction character represents forward search.
#[inline]
pub fn is_forward_dirc(dirc: c_int) -> bool {
    dirc == b'/' as c_int
}

/// Check if a direction character represents backward search.
#[inline]
pub fn is_backward_dirc(dirc: c_int) -> bool {
    dirc == b'?' as c_int
}

// =============================================================================
// Pattern Preparation
// =============================================================================

/// Check if a pattern needs to use a previous pattern.
///
/// Returns true if pat is NULL or empty string.
///
/// # Safety
/// If pat is non-null, it must point to valid memory.
#[inline]
pub unsafe fn needs_previous_pattern(pat: *const c_char) -> bool {
    if pat.is_null() {
        return true;
    }
    *pat == 0
}

/// Determine which stored pattern index to use.
///
/// If pat_use is RE_LAST, returns the actual last_idx; otherwise returns pat_use.
#[inline]
pub fn get_pattern_use_index(pat_use: c_int) -> c_int {
    if pat_use == state::RE_LAST {
        state::get_last_idx()
    } else {
        pat_use
    }
}

/// Check if a stored pattern is available at the given index.
#[inline]
pub fn stored_pattern_available(idx: c_int) -> bool {
    state::get_spat_patlen(idx) > 0
}

// =============================================================================
// Search Options Processing
// =============================================================================

/// Combined options for search messaging.
#[derive(Debug, Clone, Copy)]
pub struct SearchMsgOptions {
    /// Show any messages
    pub show_msg: bool,
    /// Show "pattern not found" message
    pub show_not_found: bool,
}

impl SearchMsgOptions {
    /// Parse message options from search options.
    pub fn from_options(options: c_int) -> Self {
        let msg_bits = options & helpers::options::SEARCH_MSG;
        Self {
            show_msg: msg_bits != 0,
            show_not_found: msg_bits == helpers::options::SEARCH_MSG,
        }
    }
}

/// Combined options for search behavior.
#[derive(Debug, Clone, Copy)]
pub struct SearchBehavior {
    /// Accept match at starting position
    pub accept_at_start: bool,
    /// Return position at end of match
    pub return_end: bool,
    /// Keep previous search pattern
    pub keep_pattern: bool,
    /// Put pattern in history
    pub add_to_history: bool,
    /// Check for typed char to cancel
    pub allow_peek: bool,
    /// Start at column instead of zero
    pub use_column: bool,
    /// Match only once in closed fold
    pub fold_mode: bool,
}

impl SearchBehavior {
    /// Parse behavior options from search options.
    pub fn from_options(options: c_int) -> Self {
        Self {
            accept_at_start: helpers::has_search_start(options),
            return_end: helpers::has_search_end(options),
            keep_pattern: helpers::has_search_keep(options),
            add_to_history: helpers::has_search_his(options),
            allow_peek: helpers::has_search_peek(options),
            use_column: helpers::has_search_col(options),
            fold_mode: (options & 0x80) != 0, // SEARCH_FOLD
        }
    }
}

// =============================================================================
// Extra Column Calculation
// =============================================================================

/// Calculate the extra column offset for search.
///
/// When not accepting a match at the start position, we need to offset
/// by the width of the character at the current position.
#[inline]
pub fn calc_extra_col(char_len: c_int, forward: bool, accept_at_start: bool) -> c_int {
    if forward {
        if accept_at_start {
            0
        } else {
            char_len
        }
    } else if accept_at_start {
        char_len
    } else {
        0
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get SEARCH_OK constant.
#[no_mangle]
pub extern "C" fn rs_search_ok() -> c_int {
    SEARCH_OK
}

/// FFI: Get SEARCH_FAIL constant.
#[no_mangle]
pub extern "C" fn rs_search_fail() -> c_int {
    SEARCH_FAIL
}

/// FFI: Get SEARCH_OK_WITH_OFFSET constant.
#[no_mangle]
pub extern "C" fn rs_search_ok_with_offset() -> c_int {
    SEARCH_OK_WITH_OFFSET
}

/// FFI: Compute effective search direction.
#[no_mangle]
pub extern "C" fn rs_effective_direction(dirc: c_int, options: c_int) -> c_int {
    effective_direction(dirc, options)
}

/// FFI: Check if direction is forward.
#[no_mangle]
pub extern "C" fn rs_is_forward_dirc(dirc: c_int) -> c_int {
    c_int::from(is_forward_dirc(dirc))
}

/// FFI: Check if direction is backward.
#[no_mangle]
pub extern "C" fn rs_is_backward_dirc(dirc: c_int) -> c_int {
    c_int::from(is_backward_dirc(dirc))
}

/// FFI: Check if pattern needs previous pattern.
///
/// # Safety
/// If pat is non-null, it must point to valid memory.
#[no_mangle]
pub unsafe extern "C" fn rs_needs_previous_pattern(pat: *const c_char) -> c_int {
    c_int::from(needs_previous_pattern(pat))
}

/// FFI: Get the pattern index to use.
#[no_mangle]
pub extern "C" fn rs_get_pattern_use_index(pat_use: c_int) -> c_int {
    get_pattern_use_index(pat_use)
}

/// FFI: Check if stored pattern is available.
#[no_mangle]
pub extern "C" fn rs_stored_pattern_available(idx: c_int) -> c_int {
    c_int::from(stored_pattern_available(idx))
}

/// FFI: Calculate extra column offset.
#[no_mangle]
pub extern "C" fn rs_calc_extra_col(
    char_len: c_int,
    forward: c_int,
    accept_at_start: c_int,
) -> c_int {
    calc_extra_col(char_len, forward != 0, accept_at_start != 0)
}

/// FFI: Check if options indicate showing messages.
#[no_mangle]
pub extern "C" fn rs_search_should_show_msg(options: c_int) -> c_int {
    c_int::from(SearchMsgOptions::from_options(options).show_msg)
}

/// FFI: Check if options indicate showing not-found message.
#[no_mangle]
pub extern "C" fn rs_search_should_show_not_found(options: c_int) -> c_int {
    c_int::from(SearchMsgOptions::from_options(options).show_not_found)
}

/// FFI: Initialize search loop state.
#[no_mangle]
pub extern "C" fn rs_search_loop_init(lnum: c_int, col: c_int) -> SearchLoopState {
    SearchLoopState::at_pos(lnum, col)
}

/// FFI: Check if search loop should wrap.
///
/// # Safety
/// The caller must ensure `state` points to valid, properly aligned memory.
#[no_mangle]
pub unsafe extern "C" fn rs_search_loop_should_wrap(state: *const SearchLoopState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from((*state).should_wrap())
}

/// FFI: Mark search loop as wrapped.
///
/// # Safety
/// The caller must ensure `state` points to valid, properly aligned memory.
#[no_mangle]
pub unsafe extern "C" fn rs_search_loop_mark_wrapped(state: *mut SearchLoopState) {
    if !state.is_null() {
        (*state).mark_wrapped();
    }
}

/// FFI: Check if search loop has wrapped.
///
/// # Safety
/// The caller must ensure `state` points to valid, properly aligned memory.
#[no_mangle]
pub unsafe extern "C" fn rs_search_loop_has_wrapped(state: *const SearchLoopState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from((*state).wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_constants() {
        assert_eq!(SEARCH_OK, 1);
        assert_eq!(SEARCH_FAIL, 0);
        assert_eq!(SEARCH_OK_WITH_OFFSET, 2);
    }

    #[test]
    fn test_is_forward_backward_dirc() {
        assert!(is_forward_dirc(b'/' as c_int));
        assert!(!is_forward_dirc(b'?' as c_int));
        assert!(!is_backward_dirc(b'/' as c_int));
        assert!(is_backward_dirc(b'?' as c_int));
    }

    #[test]
    fn test_needs_previous_pattern() {
        // SAFETY: Testing with controlled inputs
        unsafe {
            assert!(needs_previous_pattern(std::ptr::null()));

            let empty = b"\0";
            assert!(needs_previous_pattern(empty.as_ptr() as *const c_char));

            let non_empty = b"test\0";
            assert!(!needs_previous_pattern(non_empty.as_ptr() as *const c_char));
        }
    }

    #[test]
    fn test_calc_extra_col() {
        // Forward search, not accepting at start
        assert_eq!(calc_extra_col(3, true, false), 3);
        // Forward search, accepting at start
        assert_eq!(calc_extra_col(3, true, true), 0);
        // Backward search, not accepting at start
        assert_eq!(calc_extra_col(3, false, false), 0);
        // Backward search, accepting at start
        assert_eq!(calc_extra_col(3, false, true), 3);
    }

    #[test]
    fn test_search_loop_state() {
        let state = SearchLoopState::new();
        assert!(!state.wrapped);
        assert_eq!(state.loop_iter, 0);
        assert_eq!(state.match_count, 0);

        let mut state = SearchLoopState::at_pos(10, 5);
        assert_eq!(state.start_lnum, 10);
        assert_eq!(state.start_col, 5);

        state.mark_wrapped();
        assert!(state.wrapped);
        assert_eq!(state.loop_iter, 1);

        state.found_match();
        assert_eq!(state.match_count, 1);
    }

    #[test]
    fn test_search_msg_options() {
        use crate::helpers::options::*;

        // No message options
        let opts = SearchMsgOptions::from_options(0);
        assert!(!opts.show_msg);
        assert!(!opts.show_not_found);

        // Only NFMSG (show only "not found" messages)
        let opts = SearchMsgOptions::from_options(SEARCH_NFMSG);
        assert!(opts.show_msg);
        assert!(!opts.show_not_found);

        // Full MSG (show all messages)
        let opts = SearchMsgOptions::from_options(SEARCH_MSG);
        assert!(opts.show_msg);
        assert!(opts.show_not_found);
    }

    #[test]
    fn test_search_behavior() {
        use crate::helpers::options::*;

        let opts = SEARCH_START | SEARCH_END | SEARCH_HIS;
        let behavior = SearchBehavior::from_options(opts);
        assert!(behavior.accept_at_start);
        assert!(behavior.return_end);
        assert!(behavior.add_to_history);
        assert!(!behavior.keep_pattern);
        assert!(!behavior.allow_peek);
        assert!(!behavior.use_column);
    }
}
