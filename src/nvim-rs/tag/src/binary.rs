//! Binary tag file search operations
//!
//! This module provides Rust implementations for binary search operations
//! in tag files, including offset calculation and sorted file handling.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::manual_midpoint)]

use std::ffi::{c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

/// Tag file sorted state - not sorted
pub const TAG_FILE_SORTED_NO: u8 = b'0';
/// Tag file sorted state - sorted case sensitive
pub const TAG_FILE_SORTED_CASE: u8 = b'1';
/// Tag file sorted state - sorted case insensitive (foldcase)
pub const TAG_FILE_SORTED_FOLDCASE: u8 = b'2';

/// Binary search state - searching
pub const TS_BINARY: c_int = 3;
/// State - skipping back after match
pub const TS_SKIP_BACK: c_int = 4;
/// State - linear search (fallback)
pub const TS_LINEAR: c_int = 5;
/// State - step forward
pub const TS_STEP_FORWARD: c_int = 6;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to file offset type
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileOffset(i64);

impl FileOffset {
    /// Create from raw value
    pub const fn from_raw(v: i64) -> Self {
        Self(v)
    }

    /// Get raw value
    pub const fn as_raw(self) -> i64 {
        self.0
    }

    /// Check if offset is valid (non-negative)
    pub const fn is_valid(self) -> bool {
        self.0 >= 0
    }

    /// Invalid offset sentinel
    pub const fn invalid() -> Self {
        Self(-1)
    }

    /// Zero offset
    pub const fn zero() -> Self {
        Self(0)
    }
}

/// Opaque handle to binary search info
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BinarySearchHandle(*mut c_void);

impl BinarySearchHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

// =============================================================================
// Binary Search State
// =============================================================================

/// Binary search state for tag file searching
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BinarySearchState {
    /// Low bound of search range
    pub low_offset: FileOffset,
    /// High bound of search range
    pub high_offset: FileOffset,
    /// Current search position
    pub curr_offset: FileOffset,
    /// Where the match was found
    pub match_offset: FileOffset,
    /// File size (for bounds checking)
    pub file_size: FileOffset,
    /// Current search state
    pub state: c_int,
    /// Number of iterations
    pub iterations: c_int,
}

impl Default for BinarySearchState {
    fn default() -> Self {
        Self {
            low_offset: FileOffset::zero(),
            high_offset: FileOffset::zero(),
            curr_offset: FileOffset::zero(),
            match_offset: FileOffset::invalid(),
            file_size: FileOffset::zero(),
            state: TS_LINEAR,
            iterations: 0,
        }
    }
}

impl BinarySearchState {
    /// Initialize for binary search with file size
    pub fn init(file_size: i64) -> Self {
        Self {
            low_offset: FileOffset::zero(),
            high_offset: FileOffset::from_raw(file_size),
            curr_offset: FileOffset::zero(),
            match_offset: FileOffset::invalid(),
            file_size: FileOffset::from_raw(file_size),
            state: TS_BINARY,
            iterations: 0,
        }
    }

    /// Check if binary search is active
    pub const fn is_binary_search(&self) -> bool {
        self.state == TS_BINARY
    }

    /// Check if we're in skip-back state
    pub const fn is_skip_back(&self) -> bool {
        self.state == TS_SKIP_BACK
    }

    /// Check if we're in binary or skip-back state
    pub const fn is_binary_mode(&self) -> bool {
        self.state == TS_BINARY || self.state == TS_SKIP_BACK
    }

    /// Check if search is complete
    pub const fn is_complete(&self) -> bool {
        self.state == TS_LINEAR || self.state == TS_STEP_FORWARD
    }

    /// Check if we found a match
    pub const fn has_match(&self) -> bool {
        self.match_offset.is_valid()
    }

    /// Calculate next binary search offset
    pub fn next_offset(&self) -> FileOffset {
        let low = self.low_offset.as_raw();
        let high = self.high_offset.as_raw();
        FileOffset::from_raw((low + high) / 2)
    }

    /// Update state after comparing tag
    pub fn update_after_compare(&mut self, cmp_result: i32) {
        self.iterations += 1;

        if cmp_result < 0 {
            // Tag we're looking for comes before current
            self.high_offset = self.curr_offset;
        } else if cmp_result > 0 {
            // Tag we're looking for comes after current
            self.low_offset = self.curr_offset;
        }
        // cmp_result == 0 means match, handled separately
    }

    /// Record a match and enter skip-back state
    pub fn record_match(&mut self, offset: FileOffset) {
        self.match_offset = offset;
        self.state = TS_SKIP_BACK;
    }

    /// Check if binary search bounds have converged
    pub const fn bounds_converged(&self) -> bool {
        let diff = self.high_offset.as_raw() - self.low_offset.as_raw();
        diff <= 1
    }

    /// Transition to linear search
    pub fn to_linear(&mut self) {
        self.state = TS_LINEAR;
    }

    /// Transition to step forward mode
    pub fn to_step_forward(&mut self) {
        self.state = TS_STEP_FORWARD;
    }
}

// =============================================================================
// Tag File Sort Info
// =============================================================================

/// Information about tag file sorting
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagFileSortInfo {
    /// Sort type character ('0', '1', or '2')
    pub sort_type: u8,
    /// Whether file is sorted case-insensitively
    pub is_foldcase: bool,
    /// Whether a sort error was detected
    pub sort_error: bool,
}

impl Default for TagFileSortInfo {
    fn default() -> Self {
        Self {
            sort_type: 0, // NUL - unknown
            is_foldcase: false,
            sort_error: false,
        }
    }
}

impl TagFileSortInfo {
    /// Check if file is sorted
    pub const fn is_sorted(&self) -> bool {
        self.sort_type == TAG_FILE_SORTED_CASE || self.sort_type == TAG_FILE_SORTED_FOLDCASE
    }

    /// Check if file is unsorted
    pub const fn is_unsorted(&self) -> bool {
        self.sort_type == TAG_FILE_SORTED_NO
    }

    /// Check if sort type is unknown
    pub const fn is_unknown(&self) -> bool {
        self.sort_type == 0
    }

    /// Check if binary search can be used
    pub fn can_use_binary(&self, ignore_case: bool) -> bool {
        if self.sort_error || !self.is_sorted() {
            return false;
        }
        // If searching case-insensitively, need foldcase sorted file
        if ignore_case && !self.is_foldcase {
            return false;
        }
        true
    }

    /// Parse sort type from tag file header value
    pub fn from_header_value(value: u8) -> Self {
        Self {
            sort_type: value,
            is_foldcase: value == TAG_FILE_SORTED_FOLDCASE,
            sort_error: false,
        }
    }

    /// Mark as having a sort error
    pub fn set_sort_error(&mut self) {
        self.sort_error = true;
    }
}

// =============================================================================
// Binary Search Result
// =============================================================================

/// Result of a binary search step
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinarySearchResult {
    /// Continue searching
    Continue = 0,
    /// Found exact match
    Found = 1,
    /// Search complete, no match
    NotFound = 2,
    /// Error occurred
    Error = 3,
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Create binary search state
#[no_mangle]
pub extern "C" fn rs_tag_binary_search_init(file_size: i64) -> BinarySearchState {
    BinarySearchState::init(file_size)
}

/// FFI export: Check if binary search is active
#[no_mangle]
pub extern "C" fn rs_tag_binary_is_active(state: *const BinarySearchState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).is_binary_mode() })
}

/// FFI export: Calculate next binary search offset
#[no_mangle]
pub extern "C" fn rs_tag_binary_next_offset(state: *const BinarySearchState) -> i64 {
    if state.is_null() {
        return -1;
    }
    unsafe { (*state).next_offset().as_raw() }
}

/// FFI export: Check if bounds have converged
#[no_mangle]
pub extern "C" fn rs_tag_binary_bounds_converged(state: *const BinarySearchState) -> c_int {
    if state.is_null() {
        return 1;
    }
    c_int::from(unsafe { (*state).bounds_converged() })
}

/// FFI export: Check if file is sorted by sort type byte
#[no_mangle]
pub extern "C" fn rs_tag_file_is_sorted_byte(sort_type: u8) -> c_int {
    let info = TagFileSortInfo::from_header_value(sort_type);
    c_int::from(info.is_sorted())
}

/// FFI export: Check if binary search can be used
#[no_mangle]
pub extern "C" fn rs_tag_can_use_binary(sort_type: u8, ignore_case: c_int) -> c_int {
    let info = TagFileSortInfo::from_header_value(sort_type);
    c_int::from(info.can_use_binary(ignore_case != 0))
}

/// FFI export: Get search state value
#[no_mangle]
pub extern "C" fn rs_tag_binary_get_state(state: *const BinarySearchState) -> c_int {
    if state.is_null() {
        return TS_LINEAR;
    }
    unsafe { (*state).state }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_offset() {
        let offset = FileOffset::from_raw(1000);
        assert_eq!(offset.as_raw(), 1000);
        assert!(offset.is_valid());

        let invalid = FileOffset::invalid();
        assert!(!invalid.is_valid());
        assert_eq!(invalid.as_raw(), -1);
    }

    #[test]
    fn test_binary_search_state_init() {
        let state = BinarySearchState::init(10000);
        assert_eq!(state.low_offset.as_raw(), 0);
        assert_eq!(state.high_offset.as_raw(), 10000);
        assert!(state.is_binary_search());
        assert!(!state.has_match());
    }

    #[test]
    fn test_binary_search_next_offset() {
        let mut state = BinarySearchState::init(10000);
        let next = state.next_offset();
        assert_eq!(next.as_raw(), 5000);

        state.low_offset = FileOffset::from_raw(2500);
        let next = state.next_offset();
        assert_eq!(next.as_raw(), 6250);
    }

    #[test]
    fn test_binary_search_update() {
        let mut state = BinarySearchState::init(10000);
        state.curr_offset = FileOffset::from_raw(5000);

        // Tag before current
        state.update_after_compare(-1);
        assert_eq!(state.high_offset.as_raw(), 5000);
        assert_eq!(state.iterations, 1);

        // Tag after current
        state.update_after_compare(1);
        assert_eq!(state.low_offset.as_raw(), 5000);
    }

    #[test]
    fn test_binary_search_record_match() {
        let mut state = BinarySearchState::init(10000);
        assert!(!state.has_match());

        state.record_match(FileOffset::from_raw(5000));
        assert!(state.has_match());
        assert!(state.is_skip_back());
    }

    #[test]
    fn test_tag_file_sort_info() {
        let unknown = TagFileSortInfo::default();
        assert!(unknown.is_unknown());
        assert!(!unknown.is_sorted());

        let sorted_case = TagFileSortInfo::from_header_value(TAG_FILE_SORTED_CASE);
        assert!(sorted_case.is_sorted());
        assert!(!sorted_case.is_foldcase);
        assert!(sorted_case.can_use_binary(false));
        assert!(!sorted_case.can_use_binary(true)); // Can't use for ignore-case

        let sorted_fold = TagFileSortInfo::from_header_value(TAG_FILE_SORTED_FOLDCASE);
        assert!(sorted_fold.is_sorted());
        assert!(sorted_fold.is_foldcase);
        assert!(sorted_fold.can_use_binary(true));
    }

    #[test]
    fn test_bounds_converged() {
        let mut state = BinarySearchState::init(10000);
        assert!(!state.bounds_converged());

        state.high_offset = FileOffset::from_raw(1);
        state.low_offset = FileOffset::from_raw(0);
        assert!(state.bounds_converged());
    }

    #[test]
    fn test_ffi_functions() {
        let state = rs_tag_binary_search_init(10000);
        assert_eq!(state.state, TS_BINARY);

        assert_eq!(rs_tag_file_is_sorted_byte(TAG_FILE_SORTED_CASE), 1);
        assert_eq!(rs_tag_file_is_sorted_byte(TAG_FILE_SORTED_NO), 0);

        assert_eq!(rs_tag_can_use_binary(TAG_FILE_SORTED_CASE, 0), 1);
        assert_eq!(rs_tag_can_use_binary(TAG_FILE_SORTED_CASE, 1), 0);
        assert_eq!(rs_tag_can_use_binary(TAG_FILE_SORTED_FOLDCASE, 1), 1);
    }
}
