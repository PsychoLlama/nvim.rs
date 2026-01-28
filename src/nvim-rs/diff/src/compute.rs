//! Internal diff computation utilities
//!
//! This module provides structures and helpers for internal diff computation
//! using the xdiff library. It handles:
//! - Diff algorithm selection
//! - Memory file (mmfile) management helpers
//! - Hunk processing structures
//!
//! The actual xdiff calls remain in C, but this module provides the Rust
//! infrastructure for managing the computation process.

use std::ffi::c_int;

use crate::buffer::{DiffBlockHandle, TabpageHandle, DB_COUNT};

// Line number type matching linenr_T (i32)
type LinenrT = i32;

// Result constants
const OK: c_int = 1;
const FAIL: c_int = 0;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    fn nvim_tabpage_get_diffbuf(tp: TabpageHandle, idx: c_int) -> *mut std::ffi::c_void;
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
}

// =============================================================================
// Diff Algorithm Selection
// =============================================================================

/// Diff algorithm types.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffAlgorithm {
    /// Myers algorithm (default).
    #[default]
    Myers = 0,
    /// Minimal diff algorithm.
    Minimal = 1,
    /// Patience diff algorithm.
    Patience = 2,
    /// Histogram diff algorithm.
    Histogram = 3,
}

/// Get the XDF flag for a diff algorithm.
#[no_mangle]
pub const extern "C" fn rs_diff_algorithm_to_xdf(algo: DiffAlgorithm) -> c_int {
    const XDF_NEED_MINIMAL: c_int = 1 << 0;
    const XDF_PATIENCE_DIFF: c_int = 1 << 14;
    const XDF_HISTOGRAM_DIFF: c_int = 1 << 15;

    match algo {
        DiffAlgorithm::Myers => 0,
        DiffAlgorithm::Minimal => XDF_NEED_MINIMAL,
        DiffAlgorithm::Patience => XDF_PATIENCE_DIFF,
        DiffAlgorithm::Histogram => XDF_HISTOGRAM_DIFF,
    }
}

/// Parse a diff algorithm from a string option.
///
/// # Safety
/// `opt` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_parse_algorithm(opt: *const std::ffi::c_char) -> DiffAlgorithm {
    if opt.is_null() {
        return DiffAlgorithm::Myers;
    }

    let opt_str = {
        let c_str = std::ffi::CStr::from_ptr(opt);
        c_str.to_str().unwrap_or("")
    };

    match opt_str {
        "minimal" => DiffAlgorithm::Minimal,
        "patience" => DiffAlgorithm::Patience,
        "histogram" => DiffAlgorithm::Histogram,
        _ => DiffAlgorithm::Myers,
    }
}

// =============================================================================
// Diff Hunk Structure
// =============================================================================

/// A single diff hunk from xdiff output.
/// Named XdiffHunk to distinguish from DiffHunk in lib.rs.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct XdiffHunk {
    /// Starting line in buffer A (0-based from xdiff, needs +1 for 1-based).
    pub start_a: c_int,
    /// Number of lines in buffer A.
    pub count_a: c_int,
    /// Starting line in buffer B (0-based from xdiff, needs +1 for 1-based).
    pub start_b: c_int,
    /// Number of lines in buffer B.
    pub count_b: c_int,
}

impl XdiffHunk {
    /// Check if this hunk represents an addition (no lines from A).
    #[must_use]
    pub const fn is_addition(&self) -> bool {
        self.count_a == 0 && self.count_b > 0
    }

    /// Check if this hunk represents a deletion (no lines from B).
    #[must_use]
    pub const fn is_deletion(&self) -> bool {
        self.count_a > 0 && self.count_b == 0
    }

    /// Check if this hunk represents a change (lines from both).
    #[must_use]
    pub const fn is_change(&self) -> bool {
        self.count_a > 0 && self.count_b > 0
    }

    /// Check if this hunk is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.count_a == 0 && self.count_b == 0
    }

    /// Convert xdiff's 0-based start_a to 1-based line number.
    #[must_use]
    pub const fn lnum_a(&self) -> LinenrT {
        // xdiff returns the line after the last unchanged line
        // For additions at the start, this would be 0 -> becomes line 1
        (self.start_a + 1) as LinenrT
    }

    /// Convert xdiff's 0-based start_b to 1-based line number.
    #[must_use]
    pub const fn lnum_b(&self) -> LinenrT {
        (self.start_b + 1) as LinenrT
    }
}

/// Check if a hunk is an addition.
#[no_mangle]
pub const extern "C" fn rs_diff_hunk_is_addition(hunk: &XdiffHunk) -> bool {
    hunk.is_addition()
}

/// Check if a hunk is a deletion.
#[no_mangle]
pub const extern "C" fn rs_diff_hunk_is_deletion(hunk: &XdiffHunk) -> bool {
    hunk.is_deletion()
}

/// Check if a hunk is a change.
#[no_mangle]
pub const extern "C" fn rs_diff_hunk_is_change(hunk: &XdiffHunk) -> bool {
    hunk.is_change()
}

/// Get the 1-based line number for buffer A.
#[no_mangle]
pub const extern "C" fn rs_diff_hunk_lnum_a(hunk: &XdiffHunk) -> LinenrT {
    hunk.lnum_a()
}

/// Get the 1-based line number for buffer B.
#[no_mangle]
pub const extern "C" fn rs_diff_hunk_lnum_b(hunk: &XdiffHunk) -> LinenrT {
    hunk.lnum_b()
}

// =============================================================================
// Diff Computation State
// =============================================================================

/// State for tracking diff computation between multiple buffers.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffComputeState {
    /// Tabpage being computed.
    pub tp: TabpageHandle,
    /// Index of the "original" buffer for comparison.
    pub idx_orig: c_int,
    /// Index of the "new" buffer for comparison.
    pub idx_new: c_int,
    /// Whether computation is in progress.
    pub in_progress: bool,
    /// Whether computation succeeded.
    pub succeeded: bool,
    /// Error code if failed.
    pub error_code: c_int,
}

impl Default for DiffComputeState {
    fn default() -> Self {
        Self {
            tp: TabpageHandle::null(),
            idx_orig: -1,
            idx_new: -1,
            in_progress: false,
            succeeded: false,
            error_code: 0,
        }
    }
}

/// Create a default diff compute state.
#[no_mangle]
pub extern "C" fn rs_diff_compute_state_default() -> DiffComputeState {
    DiffComputeState::default()
}

/// Initialize computation state for comparing two buffers.
///
/// # Safety
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_compute_state_init(
    tp: TabpageHandle,
    idx_orig: c_int,
    idx_new: c_int,
) -> DiffComputeState {
    if tp.is_null()
        || idx_orig < 0
        || idx_orig >= DB_COUNT as c_int
        || idx_new < 0
        || idx_new >= DB_COUNT as c_int
    {
        return DiffComputeState::default();
    }

    // Validate buffers exist
    if nvim_tabpage_get_diffbuf(tp, idx_orig).is_null()
        || nvim_tabpage_get_diffbuf(tp, idx_new).is_null()
    {
        return DiffComputeState::default();
    }

    DiffComputeState {
        tp,
        idx_orig,
        idx_new,
        in_progress: true,
        succeeded: false,
        error_code: 0,
    }
}

// =============================================================================
// Buffer Pair Iteration
// =============================================================================

/// Pair of buffer indices for diff computation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffBufferPair {
    /// Index of first buffer.
    pub idx1: c_int,
    /// Index of second buffer.
    pub idx2: c_int,
    /// Whether this is a valid pair.
    pub valid: bool,
}

impl Default for DiffBufferPair {
    fn default() -> Self {
        Self {
            idx1: -1,
            idx2: -1,
            valid: false,
        }
    }
}

/// Find the first pair of buffers that need comparison.
///
/// Returns idx_orig as the first non-null buffer, idx_new as the next one.
///
/// # Safety
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_find_buffer_pair(tp: TabpageHandle) -> DiffBufferPair {
    if tp.is_null() {
        return DiffBufferPair::default();
    }

    let mut idx_orig: c_int = -1;

    for i in 0..DB_COUNT {
        if !nvim_tabpage_get_diffbuf(tp, i).is_null() {
            if idx_orig < 0 {
                idx_orig = i;
            } else {
                // Found second buffer
                return DiffBufferPair {
                    idx1: idx_orig,
                    idx2: i,
                    valid: true,
                };
            }
        }
    }

    // Didn't find two buffers
    DiffBufferPair::default()
}

/// Find all buffer indices in a tabpage for diff.
///
/// Returns a bitmap where bit i is set if buffer i is in diff.
///
/// # Safety
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_buffer_bitmap(tp: TabpageHandle) -> c_int {
    if tp.is_null() {
        return 0;
    }

    let mut bitmap: c_int = 0;
    for i in 0..DB_COUNT {
        if !nvim_tabpage_get_diffbuf(tp, i).is_null() {
            bitmap |= 1 << i;
        }
    }
    bitmap
}

/// Count bits set in a buffer bitmap.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub const extern "C" fn rs_diff_bitmap_count(bitmap: c_int) -> c_int {
    bitmap.count_ones() as c_int
}

/// Get the nth set bit position in a bitmap.
///
/// Returns -1 if n is out of range.
#[no_mangle]
pub const extern "C" fn rs_diff_bitmap_nth(bitmap: c_int, n: c_int) -> c_int {
    if n < 0 {
        return -1;
    }

    let mut count = 0;
    let mut i = 0;
    while i < DB_COUNT as c_int {
        if (bitmap & (1 << i)) != 0 {
            if count == n {
                return i;
            }
            count += 1;
        }
        i += 1;
    }
    -1
}

// =============================================================================
// Diff Block Creation Helpers
// =============================================================================

/// Information for creating a new diff block from a hunk.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NewBlockInfo {
    /// Line numbers for each buffer.
    pub lnum: [LinenrT; 8],
    /// Line counts for each buffer.
    pub count: [LinenrT; 8],
    /// Whether this info is valid.
    pub valid: bool,
}

/// Create new block info from a hunk and buffer indices.
#[no_mangle]
pub extern "C" fn rs_diff_hunk_to_block_info(
    hunk: &XdiffHunk,
    idx_orig: c_int,
    idx_new: c_int,
) -> NewBlockInfo {
    if idx_orig < 0 || idx_orig >= DB_COUNT as c_int || idx_new < 0 || idx_new >= DB_COUNT as c_int
    {
        return NewBlockInfo::default();
    }

    let mut info = NewBlockInfo {
        lnum: [0; 8],
        count: [0; 8],
        valid: true,
    };

    #[allow(clippy::cast_sign_loss)]
    let orig_idx = idx_orig as usize;
    #[allow(clippy::cast_sign_loss)]
    let new_idx = idx_new as usize;

    info.lnum[orig_idx] = hunk.lnum_a();
    info.count[orig_idx] = hunk.count_a;
    info.lnum[new_idx] = hunk.lnum_b();
    info.count[new_idx] = hunk.count_b;

    info
}

// =============================================================================
// Block Comparison
// =============================================================================

/// Check if two diff blocks have matching content (same line counts).
///
/// This is used to verify if blocks are still valid after recomputation.
///
/// # Safety
/// `dp1` and `dp2` must be valid handles.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_blocks_match(
    dp1: DiffBlockHandle,
    dp2: DiffBlockHandle,
    idx: c_int,
) -> bool {
    if dp1.is_null() || dp2.is_null() || idx < 0 || idx >= DB_COUNT as c_int {
        return false;
    }

    nvim_diffblock_get_lnum(dp1, idx) == nvim_diffblock_get_lnum(dp2, idx)
        && nvim_diffblock_get_count(dp1, idx) == nvim_diffblock_get_count(dp2, idx)
}

/// Compute the total line difference between two blocks.
///
/// Returns (count in dp1) - (count in dp2) for the given buffer.
///
/// # Safety
/// `dp1` and `dp2` must be valid handles.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_count_diff(
    dp1: DiffBlockHandle,
    dp2: DiffBlockHandle,
    idx: c_int,
) -> LinenrT {
    if dp1.is_null() || dp2.is_null() || idx < 0 || idx >= DB_COUNT as c_int {
        return 0;
    }

    nvim_diffblock_get_count(dp1, idx) - nvim_diffblock_get_count(dp2, idx)
}

// =============================================================================
// Validation
// =============================================================================

/// Validate that a diff computation can proceed.
///
/// Returns OK if valid, FAIL otherwise.
///
/// # Safety
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_validate_compute(
    tp: TabpageHandle,
    idx_orig: c_int,
    idx_new: c_int,
) -> c_int {
    if tp.is_null() {
        return FAIL;
    }

    if idx_orig < 0 || idx_orig >= DB_COUNT as c_int {
        return FAIL;
    }

    if idx_new < 0 || idx_new >= DB_COUNT as c_int {
        return FAIL;
    }

    if idx_orig == idx_new {
        return FAIL;
    }

    if nvim_tabpage_get_diffbuf(tp, idx_orig).is_null() {
        return FAIL;
    }

    if nvim_tabpage_get_diffbuf(tp, idx_new).is_null() {
        return FAIL;
    }

    OK
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_algorithm_default() {
        assert_eq!(DiffAlgorithm::default(), DiffAlgorithm::Myers);
    }

    #[test]
    fn test_diff_algorithm_values() {
        assert_eq!(DiffAlgorithm::Myers as c_int, 0);
        assert_eq!(DiffAlgorithm::Minimal as c_int, 1);
        assert_eq!(DiffAlgorithm::Patience as c_int, 2);
        assert_eq!(DiffAlgorithm::Histogram as c_int, 3);
    }

    #[test]
    fn test_diff_algorithm_to_xdf() {
        assert_eq!(rs_diff_algorithm_to_xdf(DiffAlgorithm::Myers), 0);
        assert_eq!(rs_diff_algorithm_to_xdf(DiffAlgorithm::Minimal), 1);
        assert_eq!(rs_diff_algorithm_to_xdf(DiffAlgorithm::Patience), 1 << 14);
        assert_eq!(rs_diff_algorithm_to_xdf(DiffAlgorithm::Histogram), 1 << 15);
    }

    #[test]
    fn test_diff_hunk_default() {
        let hunk = XdiffHunk::default();
        assert_eq!(hunk.start_a, 0);
        assert_eq!(hunk.count_a, 0);
        assert_eq!(hunk.start_b, 0);
        assert_eq!(hunk.count_b, 0);
        assert!(hunk.is_empty());
    }

    #[test]
    fn test_diff_hunk_addition() {
        let hunk = XdiffHunk {
            start_a: 5,
            count_a: 0,
            start_b: 5,
            count_b: 3,
        };
        assert!(hunk.is_addition());
        assert!(!hunk.is_deletion());
        assert!(!hunk.is_change());
    }

    #[test]
    fn test_diff_hunk_deletion() {
        let hunk = XdiffHunk {
            start_a: 5,
            count_a: 3,
            start_b: 5,
            count_b: 0,
        };
        assert!(!hunk.is_addition());
        assert!(hunk.is_deletion());
        assert!(!hunk.is_change());
    }

    #[test]
    fn test_diff_hunk_change() {
        let hunk = XdiffHunk {
            start_a: 5,
            count_a: 2,
            start_b: 5,
            count_b: 3,
        };
        assert!(!hunk.is_addition());
        assert!(!hunk.is_deletion());
        assert!(hunk.is_change());
    }

    #[test]
    fn test_diff_hunk_lnum() {
        let hunk = XdiffHunk {
            start_a: 0, // 0-based from xdiff
            count_a: 1,
            start_b: 5, // 0-based from xdiff
            count_b: 2,
        };
        assert_eq!(hunk.lnum_a(), 1); // Convert to 1-based
        assert_eq!(hunk.lnum_b(), 6); // Convert to 1-based
    }

    #[test]
    fn test_diff_compute_state_default() {
        let state = DiffComputeState::default();
        assert!(state.tp.is_null());
        assert_eq!(state.idx_orig, -1);
        assert_eq!(state.idx_new, -1);
        assert!(!state.in_progress);
        assert!(!state.succeeded);
    }

    #[test]
    fn test_diff_buffer_pair_default() {
        let pair = DiffBufferPair::default();
        assert_eq!(pair.idx1, -1);
        assert_eq!(pair.idx2, -1);
        assert!(!pair.valid);
    }

    #[test]
    fn test_diff_bitmap_count() {
        assert_eq!(rs_diff_bitmap_count(0), 0);
        assert_eq!(rs_diff_bitmap_count(0b1), 1);
        assert_eq!(rs_diff_bitmap_count(0b11), 2);
        assert_eq!(rs_diff_bitmap_count(0b1010), 2);
        assert_eq!(rs_diff_bitmap_count(0b1111_1111), 8);
    }

    #[test]
    fn test_diff_bitmap_nth() {
        let bitmap = 0b1010_0101; // bits 0, 2, 5, 7 are set
        assert_eq!(rs_diff_bitmap_nth(bitmap, 0), 0);
        assert_eq!(rs_diff_bitmap_nth(bitmap, 1), 2);
        assert_eq!(rs_diff_bitmap_nth(bitmap, 2), 5);
        assert_eq!(rs_diff_bitmap_nth(bitmap, 3), 7);
        assert_eq!(rs_diff_bitmap_nth(bitmap, 4), -1);
        assert_eq!(rs_diff_bitmap_nth(bitmap, -1), -1);
    }

    #[test]
    fn test_new_block_info_default() {
        let info = NewBlockInfo::default();
        assert!(!info.valid);
        for i in 0..8 {
            assert_eq!(info.lnum[i], 0);
            assert_eq!(info.count[i], 0);
        }
    }

    #[test]
    fn test_diff_hunk_to_block_info() {
        let hunk = XdiffHunk {
            start_a: 9, // 0-based
            count_a: 3,
            start_b: 9, // 0-based
            count_b: 5,
        };
        let info = rs_diff_hunk_to_block_info(&hunk, 0, 1);
        assert!(info.valid);
        assert_eq!(info.lnum[0], 10); // 1-based
        assert_eq!(info.count[0], 3);
        assert_eq!(info.lnum[1], 10); // 1-based
        assert_eq!(info.count[1], 5);
    }

    #[test]
    fn test_diff_hunk_to_block_info_invalid_indices() {
        let hunk = XdiffHunk::default();
        let info = rs_diff_hunk_to_block_info(&hunk, -1, 0);
        assert!(!info.valid);
        let info = rs_diff_hunk_to_block_info(&hunk, 0, 10);
        assert!(!info.valid);
    }
}
