//! Multiline and buffer matching support.
//!
//! This module provides support for matching regex patterns across multiple
//! lines in a buffer. This is used for operations like search, substitute,
//! and syntax highlighting.
//!
//! # Key Concepts
//!
//! - **Buffer matching**: Patterns are matched against lines in a Neovim buffer
//! - **Line fetching**: Callback functions fetch line content from the buffer
//! - **Position tracking**: Matches are reported as (line, column) pairs
//! - **Cross-line patterns**: Patterns with `\n` can match across line boundaries
//!
//! # API
//!
//! The main entry point is [`regexec_multi`] which performs multiline matching.

use std::ffi::c_int;
use std::ptr;

use crate::api::{LPos, RegMmatch, RegProg, NSUBEXP};

// =============================================================================
// Line Fetcher Callback
// =============================================================================

/// Type for line fetching callback.
///
/// This function is called to get the content of a line from the buffer.
/// Returns a pointer to the line content (NUL-terminated), or null if
/// the line doesn't exist.
///
/// # Parameters
/// - `lnum`: Line number (1-based)
/// - `userdata`: User-provided context data
pub type LineFetcher =
    unsafe extern "C" fn(lnum: c_int, userdata: *mut std::ffi::c_void) -> *const u8;

// =============================================================================
// Multiline Match Context
// =============================================================================

/// Context for multiline matching.
#[repr(C)]
pub struct MultiMatchContext {
    /// Compiled regex program
    pub prog: *mut RegProg,

    /// Start line number (1-based)
    pub start_lnum: c_int,

    /// Start column (0-based)
    pub start_col: c_int,

    /// Maximum line to search (0 = no limit)
    pub max_lnum: c_int,

    /// Maximum column on first line (0 = no limit)
    pub max_col: c_int,

    /// Line fetcher callback
    pub line_fetcher: Option<LineFetcher>,

    /// User data for line fetcher
    pub userdata: *mut std::ffi::c_void,

    /// Whether to search for longest match
    pub longest: bool,

    /// Buffer number (for caching)
    pub buf_id: c_int,
}

impl Default for MultiMatchContext {
    fn default() -> Self {
        Self {
            prog: ptr::null_mut(),
            start_lnum: 1,
            start_col: 0,
            max_lnum: 0,
            max_col: 0,
            line_fetcher: None,
            userdata: ptr::null_mut(),
            longest: false,
            buf_id: 0,
        }
    }
}

// =============================================================================
// Position Handling
// =============================================================================

/// Multiline position (line number and column).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MultiPos {
    /// Line number (1-based, 0 means unset)
    pub lnum: c_int,
    /// Column (0-based byte offset)
    pub col: c_int,
}

impl MultiPos {
    /// Create a new multiline position.
    pub fn new(lnum: c_int, col: c_int) -> Self {
        Self { lnum, col }
    }

    /// Check if position is set.
    pub fn is_set(&self) -> bool {
        self.lnum > 0
    }

    /// Convert to LPos.
    pub fn to_lpos(&self) -> LPos {
        LPos {
            lnum: self.lnum,
            col: self.col,
            coladd: 0,
        }
    }
}

// =============================================================================
// Multiline Match Result
// =============================================================================

/// Result of a multiline match.
#[repr(C)]
pub struct MultiMatchResult {
    /// Whether a match was found
    pub matched: bool,
    /// Start positions of submatches
    pub startpos: [MultiPos; NSUBEXP],
    /// End positions of submatches
    pub endpos: [MultiPos; NSUBEXP],
}

impl Default for MultiMatchResult {
    fn default() -> Self {
        Self {
            matched: false,
            startpos: [MultiPos::default(); NSUBEXP],
            endpos: [MultiPos::default(); NSUBEXP],
        }
    }
}

impl MultiMatchResult {
    /// Copy results to RegMmatch structure.
    ///
    /// # Safety
    /// `rm` must be a valid pointer.
    pub unsafe fn copy_to_regmmatch(&self, rm: *mut RegMmatch) {
        if rm.is_null() {
            return;
        }

        for i in 0..NSUBEXP {
            (*rm).startpos[i] = self.startpos[i].to_lpos();
            (*rm).endpos[i] = self.endpos[i].to_lpos();
        }
    }
}

// =============================================================================
// Multiline Matching Functions
// =============================================================================

/// Execute a multiline regex match.
///
/// This is the main entry point for buffer matching. It searches for
/// the pattern starting at the given position and returns match information.
///
/// # Safety
/// All pointers must be valid.
///
/// # Returns
/// - 0: No match found
/// - 1: Match found
/// - -1: Error
pub unsafe fn regexec_multi(
    _ctx: *const MultiMatchContext,
    _result: *mut MultiMatchResult,
) -> c_int {
    // TODO: Implement full multiline matching
    // For now, return no match
    0
}

/// Check if a line matches a pattern.
///
/// This is a simpler helper that checks if the pattern matches anywhere
/// on a single line.
///
/// # Safety
/// All pointers must be valid.
pub unsafe fn line_matches(_prog: *const RegProg, line: *const u8, _col: c_int) -> bool {
    if line.is_null() {
        return false;
    }

    // TODO: Implement single-line check within multiline context
    false
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create a new multiline match context.
#[no_mangle]
pub extern "C" fn rs_multi_match_context_new() -> *mut MultiMatchContext {
    Box::into_raw(Box::new(MultiMatchContext::default()))
}

/// Free a multiline match context.
///
/// # Safety
/// `ctx` must be a valid pointer from `rs_multi_match_context_new`.
#[no_mangle]
pub unsafe extern "C" fn rs_multi_match_context_free(ctx: *mut MultiMatchContext) {
    if !ctx.is_null() {
        drop(Box::from_raw(ctx));
    }
}

/// Set the program for a multiline match context.
///
/// # Safety
/// `ctx` and `prog` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_multi_match_context_set_prog(
    ctx: *mut MultiMatchContext,
    prog: *mut RegProg,
) {
    if !ctx.is_null() {
        (*ctx).prog = prog;
    }
}

/// Set the start position for multiline matching.
///
/// # Safety
/// `ctx` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_multi_match_context_set_start(
    ctx: *mut MultiMatchContext,
    lnum: c_int,
    col: c_int,
) {
    if !ctx.is_null() {
        (*ctx).start_lnum = lnum;
        (*ctx).start_col = col;
    }
}

/// Set the line fetcher callback.
///
/// # Safety
/// `ctx` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_multi_match_context_set_fetcher(
    ctx: *mut MultiMatchContext,
    fetcher: LineFetcher,
    userdata: *mut std::ffi::c_void,
) {
    if !ctx.is_null() {
        (*ctx).line_fetcher = Some(fetcher);
        (*ctx).userdata = userdata;
    }
}

/// Create a new multiline match result.
#[no_mangle]
pub extern "C" fn rs_multi_match_result_new() -> *mut MultiMatchResult {
    Box::into_raw(Box::new(MultiMatchResult::default()))
}

/// Free a multiline match result.
///
/// # Safety
/// `result` must be a valid pointer from `rs_multi_match_result_new`.
#[no_mangle]
pub unsafe extern "C" fn rs_multi_match_result_free(result: *mut MultiMatchResult) {
    if !result.is_null() {
        drop(Box::from_raw(result));
    }
}

/// Check if a multiline match result has a match.
///
/// # Safety
/// `result` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_multi_match_result_matched(result: *const MultiMatchResult) -> c_int {
    if result.is_null() {
        0
    } else {
        c_int::from((*result).matched)
    }
}

/// Get the start line of a submatch.
///
/// # Safety
/// `result` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_multi_match_result_start_lnum(
    result: *const MultiMatchResult,
    idx: c_int,
) -> c_int {
    if result.is_null() || idx < 0 || idx >= NSUBEXP as c_int {
        0
    } else {
        (*result).startpos[idx as usize].lnum
    }
}

/// Get the start column of a submatch.
///
/// # Safety
/// `result` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_multi_match_result_start_col(
    result: *const MultiMatchResult,
    idx: c_int,
) -> c_int {
    if result.is_null() || idx < 0 || idx >= NSUBEXP as c_int {
        0
    } else {
        (*result).startpos[idx as usize].col
    }
}

/// Get the end line of a submatch.
///
/// # Safety
/// `result` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_multi_match_result_end_lnum(
    result: *const MultiMatchResult,
    idx: c_int,
) -> c_int {
    if result.is_null() || idx < 0 || idx >= NSUBEXP as c_int {
        0
    } else {
        (*result).endpos[idx as usize].lnum
    }
}

/// Get the end column of a submatch.
///
/// # Safety
/// `result` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_multi_match_result_end_col(
    result: *const MultiMatchResult,
    idx: c_int,
) -> c_int {
    if result.is_null() || idx < 0 || idx >= NSUBEXP as c_int {
        0
    } else {
        (*result).endpos[idx as usize].col
    }
}

/// Execute multiline regex match.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_regexec_multi(
    ctx: *const MultiMatchContext,
    result: *mut MultiMatchResult,
) -> c_int {
    regexec_multi(ctx, result)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_pos_default() {
        let pos = MultiPos::default();
        assert_eq!(pos.lnum, 0);
        assert_eq!(pos.col, 0);
        assert!(!pos.is_set());
    }

    #[test]
    fn test_multi_pos_new() {
        let pos = MultiPos::new(5, 10);
        assert_eq!(pos.lnum, 5);
        assert_eq!(pos.col, 10);
        assert!(pos.is_set());
    }

    #[test]
    fn test_multi_pos_to_lpos() {
        let pos = MultiPos::new(3, 7);
        let lpos = pos.to_lpos();
        assert_eq!(lpos.lnum, 3);
        assert_eq!(lpos.col, 7);
        assert_eq!(lpos.coladd, 0);
    }

    #[test]
    fn test_multi_match_context_default() {
        let ctx = MultiMatchContext::default();
        assert!(ctx.prog.is_null());
        assert_eq!(ctx.start_lnum, 1);
        assert_eq!(ctx.start_col, 0);
        assert!(ctx.line_fetcher.is_none());
    }

    #[test]
    fn test_multi_match_result_default() {
        let result = MultiMatchResult::default();
        assert!(!result.matched);
        for pos in &result.startpos {
            assert!(!pos.is_set());
        }
        for pos in &result.endpos {
            assert!(!pos.is_set());
        }
    }
}
