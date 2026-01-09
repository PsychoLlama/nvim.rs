//! Fold tree data structure
//!
//! This module provides Rust implementations for fold tree management,
//! including fold hierarchy navigation and manipulation.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::return_self_not_must_use)]

use std::ffi::{c_int, c_void};

/// Line number type
type LinenrT = i32;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to fold array (garray_T of fold_T)
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoldArrayHandle(*mut c_void);

impl FoldArrayHandle {
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

/// Opaque handle to a single fold (fold_T)
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoldHandle(*mut c_void);

impl FoldHandle {
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
// Fold State
// =============================================================================

/// Fold state flags
pub const FOLD_FLAG_OPEN: c_int = 0x01;
pub const FOLD_FLAG_SMALL: c_int = 0x02;
pub const FOLD_FLAG_CHANGED: c_int = 0x04;

/// Fold state information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldState {
    /// First line of fold (1-based)
    pub first_line: LinenrT,
    /// Number of lines in fold
    pub line_count: LinenrT,
    /// Fold level (1-based)
    pub level: c_int,
    /// State flags
    pub flags: c_int,
}

impl Default for FoldState {
    fn default() -> Self {
        Self {
            first_line: 0,
            line_count: 0,
            level: 0,
            flags: 0,
        }
    }
}

impl FoldState {
    /// Check if fold is open
    pub const fn is_open(&self) -> bool {
        (self.flags & FOLD_FLAG_OPEN) != 0
    }

    /// Check if fold is closed
    pub const fn is_closed(&self) -> bool {
        !self.is_open()
    }

    /// Check if fold is marked as small
    pub const fn is_small(&self) -> bool {
        (self.flags & FOLD_FLAG_SMALL) != 0
    }

    /// Check if fold has been changed
    pub const fn is_changed(&self) -> bool {
        (self.flags & FOLD_FLAG_CHANGED) != 0
    }

    /// Get last line of fold (1-based)
    pub const fn last_line(&self) -> LinenrT {
        self.first_line + self.line_count - 1
    }

    /// Check if a line is contained in this fold
    pub const fn contains_line(&self, line: LinenrT) -> bool {
        line >= self.first_line && line <= self.last_line()
    }
}

// =============================================================================
// Fold Navigation
// =============================================================================

/// Result of fold search
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldSearchResult {
    /// Whether a fold was found
    pub found: bool,
    /// Index of the fold (if found)
    pub index: c_int,
    /// Whether the line is before (-1), in (0), or after (1) the fold
    pub position: c_int,
}

impl FoldSearchResult {
    /// Create a not-found result
    pub const fn not_found() -> Self {
        Self {
            found: false,
            index: -1,
            position: 0,
        }
    }

    /// Create a found result
    pub const fn found_at(index: c_int) -> Self {
        Self {
            found: true,
            index,
            position: 0,
        }
    }
}

/// Direction for fold navigation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldDirection {
    /// Move to previous fold
    Previous = -1,
    /// Stay at current fold
    Current = 0,
    /// Move to next fold
    Next = 1,
}

impl FoldDirection {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            -1 => Self::Previous,
            1 => Self::Next,
            _ => Self::Current,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Fold Tree Info
// =============================================================================

/// Information about the fold tree structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldTreeInfo {
    /// Total number of top-level folds
    pub top_level_count: c_int,
    /// Maximum nesting depth
    pub max_depth: c_int,
    /// Total number of folds at all levels
    pub total_count: c_int,
    /// Whether tree needs recalculation
    pub needs_update: bool,
}

impl Default for FoldTreeInfo {
    fn default() -> Self {
        Self {
            top_level_count: 0,
            max_depth: 0,
            total_count: 0,
            needs_update: false,
        }
    }
}

impl FoldTreeInfo {
    /// Check if tree is empty
    pub const fn is_empty(&self) -> bool {
        self.total_count == 0
    }

    /// Check if tree has nested folds
    pub const fn has_nested(&self) -> bool {
        self.max_depth > 1
    }
}

// =============================================================================
// Fold Range Operations
// =============================================================================

/// A range of lines for fold operations
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldRange {
    /// Start line (1-based)
    pub start: LinenrT,
    /// End line (1-based, inclusive)
    pub end: LinenrT,
}

impl FoldRange {
    /// Create a new range
    pub const fn new(start: LinenrT, end: LinenrT) -> Self {
        Self { start, end }
    }

    /// Create an invalid/empty range
    pub const fn invalid() -> Self {
        Self { start: 0, end: 0 }
    }

    /// Check if range is valid
    pub const fn is_valid(&self) -> bool {
        self.start > 0 && self.end >= self.start
    }

    /// Get number of lines in range
    pub const fn line_count(&self) -> LinenrT {
        if self.is_valid() {
            self.end - self.start + 1
        } else {
            0
        }
    }

    /// Check if this range contains a line
    pub const fn contains(&self, line: LinenrT) -> bool {
        self.is_valid() && line >= self.start && line <= self.end
    }

    /// Check if this range overlaps with another
    #[allow(clippy::suspicious_operation_groupings)]
    pub const fn overlaps(&self, other: &Self) -> bool {
        // Two ranges overlap if one starts before or at the other's end AND ends at or after the other's start
        self.is_valid() && other.is_valid() && self.start <= other.end && self.end >= other.start
    }

    /// Merge with another range (union)
    pub fn merge(&self, other: &Self) -> Self {
        if !self.is_valid() {
            return *other;
        }
        if !other.is_valid() {
            return *self;
        }
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Intersect with another range
    pub fn intersect(&self, other: &Self) -> Self {
        if !self.overlaps(other) {
            return Self::invalid();
        }
        Self {
            start: self.start.max(other.start),
            end: self.end.min(other.end),
        }
    }
}

// =============================================================================
// Fold Update Types
// =============================================================================

/// Type of fold tree update
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldUpdateType {
    /// No update needed
    None = 0,
    /// Update specific range
    Range = 1,
    /// Full recalculation needed
    Full = 2,
    /// Just mark dirty
    MarkDirty = 3,
}

/// Fold update request
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldUpdateRequest {
    /// Type of update
    pub update_type: FoldUpdateType,
    /// Range to update (for Range type)
    pub range: FoldRange,
    /// Whether to force update
    pub force: bool,
}

impl Default for FoldUpdateRequest {
    fn default() -> Self {
        Self {
            update_type: FoldUpdateType::None,
            range: FoldRange::invalid(),
            force: false,
        }
    }
}

impl FoldUpdateRequest {
    /// Create a full update request
    pub const fn full() -> Self {
        Self {
            update_type: FoldUpdateType::Full,
            range: FoldRange::invalid(),
            force: true,
        }
    }

    /// Create a range update request
    pub const fn for_range(start: LinenrT, end: LinenrT) -> Self {
        Self {
            update_type: FoldUpdateType::Range,
            range: FoldRange::new(start, end),
            force: false,
        }
    }

    /// Create a mark-dirty request
    pub const fn mark_dirty() -> Self {
        Self {
            update_type: FoldUpdateType::MarkDirty,
            range: FoldRange::invalid(),
            force: false,
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if fold state is open
#[no_mangle]
pub extern "C" fn rs_fold_state_is_open(state: *const FoldState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).is_open() })
}

/// FFI export: Check if fold state is closed
#[no_mangle]
pub extern "C" fn rs_fold_state_is_closed(state: *const FoldState) -> c_int {
    if state.is_null() {
        return 1;
    }
    c_int::from(unsafe { (*state).is_closed() })
}

/// FFI export: Get fold last line
#[no_mangle]
pub extern "C" fn rs_fold_state_last_line(state: *const FoldState) -> LinenrT {
    if state.is_null() {
        return 0;
    }
    unsafe { (*state).last_line() }
}

/// FFI export: Check if fold contains line
#[no_mangle]
pub extern "C" fn rs_fold_state_contains_line(state: *const FoldState, line: LinenrT) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).contains_line(line) })
}

/// FFI export: Create fold range
#[no_mangle]
pub extern "C" fn rs_fold_range_new(start: LinenrT, end: LinenrT) -> FoldRange {
    FoldRange::new(start, end)
}

/// FFI export: Check if range is valid
#[no_mangle]
pub extern "C" fn rs_fold_range_is_valid(range: *const FoldRange) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*range).is_valid() })
}

/// FFI export: Get range line count
#[no_mangle]
pub extern "C" fn rs_fold_range_line_count(range: *const FoldRange) -> LinenrT {
    if range.is_null() {
        return 0;
    }
    unsafe { (*range).line_count() }
}

/// FFI export: Check if ranges overlap
#[no_mangle]
pub extern "C" fn rs_fold_range_overlaps(r1: *const FoldRange, r2: *const FoldRange) -> c_int {
    if r1.is_null() || r2.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*r1).overlaps(&*r2) })
}

/// FFI export: Check if tree info is empty
#[no_mangle]
pub extern "C" fn rs_fold_tree_is_empty(info: *const FoldTreeInfo) -> c_int {
    if info.is_null() {
        return 1;
    }
    c_int::from(unsafe { (*info).is_empty() })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_state() {
        let state = FoldState {
            first_line: 10,
            line_count: 5,
            level: 1,
            flags: FOLD_FLAG_OPEN,
        };

        assert!(state.is_open());
        assert!(!state.is_closed());
        assert_eq!(state.last_line(), 14);
        assert!(state.contains_line(10));
        assert!(state.contains_line(14));
        assert!(!state.contains_line(9));
        assert!(!state.contains_line(15));
    }

    #[test]
    fn test_fold_state_flags() {
        let state = FoldState {
            flags: FOLD_FLAG_SMALL | FOLD_FLAG_CHANGED,
            ..Default::default()
        };

        assert!(state.is_small());
        assert!(state.is_changed());
        assert!(state.is_closed()); // No OPEN flag
    }

    #[test]
    fn test_fold_direction() {
        assert_eq!(FoldDirection::from_raw(-1), FoldDirection::Previous);
        assert_eq!(FoldDirection::from_raw(0), FoldDirection::Current);
        assert_eq!(FoldDirection::from_raw(1), FoldDirection::Next);
        assert_eq!(FoldDirection::from_raw(99), FoldDirection::Current);
    }

    #[test]
    fn test_fold_range() {
        let range = FoldRange::new(10, 20);
        assert!(range.is_valid());
        assert_eq!(range.line_count(), 11);
        assert!(range.contains(10));
        assert!(range.contains(15));
        assert!(range.contains(20));
        assert!(!range.contains(9));
        assert!(!range.contains(21));

        let invalid = FoldRange::invalid();
        assert!(!invalid.is_valid());
        assert_eq!(invalid.line_count(), 0);
    }

    #[test]
    fn test_fold_range_overlap() {
        let r1 = FoldRange::new(10, 20);
        let r2 = FoldRange::new(15, 25);
        let r3 = FoldRange::new(25, 30);

        assert!(r1.overlaps(&r2));
        assert!(r2.overlaps(&r1));
        assert!(!r1.overlaps(&r3));
    }

    #[test]
    fn test_fold_range_merge() {
        let r1 = FoldRange::new(10, 20);
        let r2 = FoldRange::new(15, 30);
        let merged = r1.merge(&r2);

        assert_eq!(merged.start, 10);
        assert_eq!(merged.end, 30);
    }

    #[test]
    fn test_fold_range_intersect() {
        let r1 = FoldRange::new(10, 20);
        let r2 = FoldRange::new(15, 30);
        let intersected = r1.intersect(&r2);

        assert_eq!(intersected.start, 15);
        assert_eq!(intersected.end, 20);

        let r3 = FoldRange::new(25, 30);
        let no_intersect = r1.intersect(&r3);
        assert!(!no_intersect.is_valid());
    }

    #[test]
    fn test_fold_tree_info() {
        let empty = FoldTreeInfo::default();
        assert!(empty.is_empty());
        assert!(!empty.has_nested());

        let with_folds = FoldTreeInfo {
            top_level_count: 5,
            max_depth: 3,
            total_count: 10,
            needs_update: false,
        };
        assert!(!with_folds.is_empty());
        assert!(with_folds.has_nested());
    }

    #[test]
    fn test_fold_update_request() {
        let full = FoldUpdateRequest::full();
        assert_eq!(full.update_type, FoldUpdateType::Full);
        assert!(full.force);

        let range = FoldUpdateRequest::for_range(10, 20);
        assert_eq!(range.update_type, FoldUpdateType::Range);
        assert!(range.range.is_valid());
    }
}
