//! Decoration range priority and insertion logic
//!
//! This module contains Rust implementations for decoration range
//! priority comparison and insertion, migrated from `src/nvim/decoration.c`.

use std::ffi::c_int;

use crate::decor::DECOR_PRIORITY_BASE;

// =============================================================================
// Priority Types
// =============================================================================

/// Internal priority representation.
/// The upper 16 bits are the user-specified priority,
/// the lower 16 bits are the sub-priority for ordering within same priority.
pub type DecorPriorityInternal = u32;

/// Extract user priority from internal priority.
#[must_use]
pub const fn priority_user(internal: DecorPriorityInternal) -> u16 {
    (internal >> 16) as u16
}

/// Extract sub-priority from internal priority.
#[must_use]
pub const fn priority_sub(internal: DecorPriorityInternal) -> u16 {
    (internal & 0xFFFF) as u16
}

/// Create internal priority from user priority and sub-priority.
#[must_use]
pub const fn priority_make(user: u16, sub: u16) -> DecorPriorityInternal {
    ((user as u32) << 16) | (sub as u32)
}

/// FFI: Extract user priority.
#[no_mangle]
pub extern "C" fn rs_priority_user(internal: DecorPriorityInternal) -> u16 {
    priority_user(internal)
}

/// FFI: Extract sub-priority.
#[no_mangle]
pub extern "C" fn rs_priority_sub(internal: DecorPriorityInternal) -> u16 {
    priority_sub(internal)
}

/// FFI: Make internal priority.
#[no_mangle]
pub extern "C" fn rs_priority_make(user: u16, sub: u16) -> DecorPriorityInternal {
    priority_make(user, sub)
}

// =============================================================================
// Range Position
// =============================================================================

/// Position within a buffer (row, col pair).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RangePos {
    pub row: c_int,
    pub col: c_int,
}

impl RangePos {
    /// Create a new position.
    #[must_use]
    pub const fn new(row: c_int, col: c_int) -> Self {
        Self { row, col }
    }

    /// Compare two positions.
    /// Returns: -1 if self < other, 0 if equal, 1 if self > other
    #[must_use]
    pub const fn cmp(&self, other: &Self) -> c_int {
        if self.row < other.row {
            -1
        } else if self.row > other.row {
            1
        } else if self.col < other.col {
            -1
        } else if self.col > other.col {
            1
        } else {
            0
        }
    }

    /// Check if this position is before another.
    #[must_use]
    pub const fn is_before(&self, other: &Self) -> bool {
        self.row < other.row || (self.row == other.row && self.col < other.col)
    }

    /// Check if this position is at or before another.
    #[must_use]
    pub const fn is_at_or_before(&self, other: &Self) -> bool {
        self.row < other.row || (self.row == other.row && self.col <= other.col)
    }

    /// Check if this position is after another.
    #[must_use]
    pub const fn is_after(&self, other: &Self) -> bool {
        self.row > other.row || (self.row == other.row && self.col > other.col)
    }
}

impl Ord for RangePos {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.row.cmp(&other.row) {
            std::cmp::Ordering::Equal => self.col.cmp(&other.col),
            ord => ord,
        }
    }
}

impl PartialOrd for RangePos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

/// FFI: Create range position.
#[no_mangle]
pub extern "C" fn rs_range_pos_new(row: c_int, col: c_int) -> RangePos {
    RangePos::new(row, col)
}

/// FFI: Compare positions.
#[no_mangle]
pub extern "C" fn rs_range_pos_cmp(a: RangePos, b: RangePos) -> c_int {
    a.cmp(&b)
}

/// FFI: Check if position is before.
#[no_mangle]
pub extern "C" fn rs_range_pos_is_before(a: RangePos, b: RangePos) -> c_int {
    c_int::from(a.is_before(&b))
}

// =============================================================================
// Range Bounds
// =============================================================================

/// Bounds of a decoration range.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RangeBounds {
    pub start: RangePos,
    pub end: RangePos,
}

impl RangeBounds {
    /// Create new bounds.
    #[must_use]
    pub const fn new(start_row: c_int, start_col: c_int, end_row: c_int, end_col: c_int) -> Self {
        Self {
            start: RangePos::new(start_row, start_col),
            end: RangePos::new(end_row, end_col),
        }
    }

    /// Check if this range contains a position.
    #[must_use]
    pub const fn contains(&self, pos: &RangePos) -> bool {
        self.start.is_at_or_before(pos) && pos.is_before(&self.end)
    }

    /// Check if this range overlaps with another.
    #[must_use]
    pub const fn overlaps(&self, other: &Self) -> bool {
        self.start.is_before(&other.end) && other.start.is_before(&self.end)
    }

    /// Check if end is at or before a position (range is "finished").
    #[must_use]
    pub const fn end_at_or_before(&self, pos: &RangePos) -> bool {
        self.end.is_at_or_before(pos)
    }

    /// Check if start is after a position (range hasn't started).
    #[must_use]
    pub const fn start_after(&self, pos: &RangePos) -> bool {
        self.start.is_after(pos)
    }

    /// Check if this range is on a specific row.
    #[must_use]
    pub const fn on_row(&self, row: c_int) -> bool {
        self.start.row <= row && row <= self.end.row
    }

    /// Check if range is zero-width (start == end).
    #[must_use]
    pub const fn is_zero_width(&self) -> bool {
        self.start.row == self.end.row && self.start.col == self.end.col
    }
}

/// FFI: Create range bounds.
#[no_mangle]
pub extern "C" fn rs_range_bounds_new(
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
) -> RangeBounds {
    RangeBounds::new(start_row, start_col, end_row, end_col)
}

/// FFI: Check if range contains position.
#[no_mangle]
pub extern "C" fn rs_range_bounds_contains(bounds: RangeBounds, pos: RangePos) -> c_int {
    c_int::from(bounds.contains(&pos))
}

/// FFI: Check if ranges overlap.
#[no_mangle]
pub extern "C" fn rs_range_bounds_overlaps(a: RangeBounds, b: RangeBounds) -> c_int {
    c_int::from(a.overlaps(&b))
}

/// FFI: Check if range end is at or before position.
#[no_mangle]
pub extern "C" fn rs_range_bounds_end_at_or_before(bounds: RangeBounds, pos: RangePos) -> c_int {
    c_int::from(bounds.end_at_or_before(&pos))
}

/// FFI: Check if range start is after position.
#[no_mangle]
pub extern "C" fn rs_range_bounds_start_after(bounds: RangeBounds, pos: RangePos) -> c_int {
    c_int::from(bounds.start_after(&pos))
}

// =============================================================================
// Insertion Point Calculation
// =============================================================================

/// Parameters for finding insertion point.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InsertionParams {
    /// Start position of the range to insert
    pub start: RangePos,
    /// Internal priority
    pub priority: DecorPriorityInternal,
    /// Ordering (for tie-breaking within same priority)
    pub ordering: c_int,
}

impl InsertionParams {
    /// Create new params.
    #[must_use]
    pub const fn new(
        start_row: c_int,
        start_col: c_int,
        priority: DecorPriorityInternal,
        ordering: c_int,
    ) -> Self {
        Self {
            start: RangePos::new(start_row, start_col),
            priority,
            ordering,
        }
    }
}

/// FFI: Create insertion params.
#[no_mangle]
pub extern "C" fn rs_insertion_params_new(
    start_row: c_int,
    start_col: c_int,
    priority: DecorPriorityInternal,
    ordering: c_int,
) -> InsertionParams {
    InsertionParams::new(start_row, start_col, priority, ordering)
}

/// Compare two ranges for insertion ordering.
///
/// Ranges are ordered by:
/// 1. Priority (higher priority = later in list = rendered on top)
/// 2. Ordering (for tie-breaking within same priority)
///
/// Returns: -1 if a < b, 0 if equal, 1 if a > b
#[must_use]
pub const fn insertion_order_cmp(
    priority_a: DecorPriorityInternal,
    ordering_a: c_int,
    priority_b: DecorPriorityInternal,
    ordering_b: c_int,
) -> c_int {
    if priority_a < priority_b {
        -1
    } else if priority_a > priority_b {
        1
    } else if ordering_a < ordering_b {
        -1
    } else if ordering_a > ordering_b {
        1
    } else {
        0
    }
}

/// FFI: Compare insertion order.
#[no_mangle]
pub extern "C" fn rs_insertion_order_cmp(
    priority_a: DecorPriorityInternal,
    ordering_a: c_int,
    priority_b: DecorPriorityInternal,
    ordering_b: c_int,
) -> c_int {
    insertion_order_cmp(priority_a, ordering_a, priority_b, ordering_b)
}

/// Find insertion point in a sorted array using binary search.
///
/// Given arrays of priorities and orderings, find the index where
/// a new entry with the given priority/ordering should be inserted
/// to maintain sorted order.
///
/// Returns the insertion index.
#[no_mangle]
pub unsafe extern "C" fn rs_find_insertion_point(
    priorities: *const DecorPriorityInternal,
    orderings: *const c_int,
    count: c_int,
    new_priority: DecorPriorityInternal,
    new_ordering: c_int,
) -> c_int {
    if priorities.is_null() || orderings.is_null() || count <= 0 {
        return 0;
    }

    let mut begin: c_int = 0;
    let mut end: c_int = count;

    while begin < end {
        let mid = begin + ((end - begin) >> 1);
        let mid_priority = *priorities.add(mid as usize);
        let mid_ordering = *orderings.add(mid as usize);

        let cmp = insertion_order_cmp(mid_priority, mid_ordering, new_priority, new_ordering);
        if cmp < 0 {
            begin = mid + 1;
        } else {
            end = mid;
        }
    }

    begin
}

// =============================================================================
// Sign Priority Comparison
// =============================================================================

/// Compare two signs for ordering.
///
/// Signs are ordered by:
/// 1. Priority (higher = first)
/// 2. Mark ID (higher = first)
/// 3. Sign add ID (higher = first)
///
/// Returns: -1 if sign1 < sign2, 0 if equal, 1 if sign1 > sign2
#[must_use]
pub const fn sign_cmp(
    priority1: c_int,
    mark_id1: u32,
    add_id1: c_int,
    priority2: c_int,
    mark_id2: u32,
    add_id2: c_int,
) -> c_int {
    // Higher priority comes first
    if priority1 > priority2 {
        return -1;
    }
    if priority1 < priority2 {
        return 1;
    }

    // Higher mark_id comes first
    if mark_id1 > mark_id2 {
        return -1;
    }
    if mark_id1 < mark_id2 {
        return 1;
    }

    // Higher add_id comes first
    if add_id1 > add_id2 {
        return -1;
    }
    if add_id1 < add_id2 {
        return 1;
    }

    0
}

// Note: rs_sign_item_cmp is defined in nvim-sign crate

// =============================================================================
// Virtual Text Column Management
// =============================================================================

/// State for managing virtual text column positions.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtColState {
    /// Current window column
    pub win_col: c_int,
    /// Right margin column
    pub right_col: c_int,
    /// Total EOL virtual text width consumed
    pub eol_consumed: c_int,
    /// Right-aligned width remaining
    pub right_remaining: c_int,
}

impl VirtColState {
    /// Create new state.
    #[must_use]
    pub const fn new(win_col: c_int, right_col: c_int, right_width: c_int) -> Self {
        Self {
            win_col,
            right_col,
            eol_consumed: 0,
            right_remaining: right_width,
        }
    }

    /// Advance by some width.
    pub fn advance(&mut self, width: c_int) {
        self.win_col += width;
        self.eol_consumed += width;
    }

    /// Consume right-aligned width.
    pub fn consume_right(&mut self, width: c_int) {
        self.right_remaining -= width;
        if self.right_remaining < 0 {
            self.right_remaining = 0;
        }
    }

    /// Get available width before right margin.
    #[must_use]
    pub const fn available_width(&self) -> c_int {
        if self.right_col > self.win_col {
            self.right_col - self.win_col
        } else {
            0
        }
    }
}

/// FFI: Create virt col state.
#[no_mangle]
pub extern "C" fn rs_virt_col_state_new(
    win_col: c_int,
    right_col: c_int,
    right_width: c_int,
) -> VirtColState {
    VirtColState::new(win_col, right_col, right_width)
}

/// FFI: Advance virt col state.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_virt_col_state_advance(state: *mut VirtColState, width: c_int) {
    if !state.is_null() {
        (*state).advance(width);
    }
}

/// FFI: Get available width.
#[no_mangle]
pub extern "C" fn rs_virt_col_state_available(state: VirtColState) -> c_int {
    state.available_width()
}

// =============================================================================
// Default Priority
// =============================================================================

/// Get the default priority for decorations.
#[no_mangle]
pub extern "C" fn rs_decor_default_priority() -> u16 {
    DECOR_PRIORITY_BASE
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_functions() {
        let internal = priority_make(0x1234, 0x5678);
        assert_eq!(priority_user(internal), 0x1234);
        assert_eq!(priority_sub(internal), 0x5678);
    }

    #[test]
    fn test_range_pos_comparison() {
        let a = RangePos::new(5, 10);
        let b = RangePos::new(5, 20);
        let c = RangePos::new(6, 0);

        assert!(a.is_before(&b));
        assert!(a.is_before(&c));
        assert!(b.is_before(&c));
        assert!(!b.is_before(&a));
        assert!(!c.is_before(&a));
    }

    #[test]
    fn test_range_pos_equality() {
        let a = RangePos::new(5, 10);
        let b = RangePos::new(5, 10);
        assert_eq!(a.cmp(&b), 0);
        assert!(!a.is_before(&b));
        assert!(a.is_at_or_before(&b));
    }

    #[test]
    fn test_range_bounds_contains() {
        let bounds = RangeBounds::new(5, 10, 5, 20);

        assert!(bounds.contains(&RangePos::new(5, 10)));
        assert!(bounds.contains(&RangePos::new(5, 15)));
        assert!(!bounds.contains(&RangePos::new(5, 20))); // end is exclusive
        assert!(!bounds.contains(&RangePos::new(5, 5)));
        assert!(!bounds.contains(&RangePos::new(6, 0)));
    }

    #[test]
    fn test_range_bounds_overlaps() {
        let a = RangeBounds::new(5, 10, 5, 20);
        let b = RangeBounds::new(5, 15, 5, 25);
        let c = RangeBounds::new(5, 20, 5, 30);
        let d = RangeBounds::new(5, 0, 5, 10);

        assert!(a.overlaps(&b));
        assert!(!a.overlaps(&c)); // adjacent, not overlapping
        assert!(!a.overlaps(&d)); // adjacent, not overlapping
    }

    #[test]
    fn test_range_bounds_multirow() {
        let bounds = RangeBounds::new(5, 10, 7, 5);

        assert!(bounds.on_row(5));
        assert!(bounds.on_row(6));
        assert!(bounds.on_row(7));
        assert!(!bounds.on_row(4));
        assert!(!bounds.on_row(8));
    }

    #[test]
    fn test_insertion_order_cmp() {
        // Different priorities
        assert_eq!(insertion_order_cmp(100, 0, 200, 0), -1);
        assert_eq!(insertion_order_cmp(200, 0, 100, 0), 1);

        // Same priority, different ordering
        assert_eq!(insertion_order_cmp(100, 5, 100, 10), -1);
        assert_eq!(insertion_order_cmp(100, 10, 100, 5), 1);

        // Equal
        assert_eq!(insertion_order_cmp(100, 5, 100, 5), 0);
    }

    #[test]
    fn test_sign_cmp() {
        // Different priorities - higher first
        assert_eq!(sign_cmp(100, 0, 0, 50, 0, 0), -1);
        assert_eq!(sign_cmp(50, 0, 0, 100, 0, 0), 1);

        // Same priority, different mark_id - higher first
        assert_eq!(sign_cmp(100, 10, 0, 100, 5, 0), -1);
        assert_eq!(sign_cmp(100, 5, 0, 100, 10, 0), 1);

        // Same priority and mark_id, different add_id
        assert_eq!(sign_cmp(100, 10, 5, 100, 10, 3), -1);
        assert_eq!(sign_cmp(100, 10, 3, 100, 10, 5), 1);

        // Equal
        assert_eq!(sign_cmp(100, 10, 5, 100, 10, 5), 0);
    }

    #[test]
    fn test_virt_col_state() {
        let mut state = VirtColState::new(10, 80, 20);
        assert_eq!(state.available_width(), 70);

        state.advance(15);
        assert_eq!(state.win_col, 25);
        assert_eq!(state.eol_consumed, 15);
        assert_eq!(state.available_width(), 55);

        state.consume_right(10);
        assert_eq!(state.right_remaining, 10);
    }

    #[test]
    fn test_find_insertion_point() {
        let priorities: [DecorPriorityInternal; 5] = [100, 200, 300, 400, 500];
        let orderings: [c_int; 5] = [0, 0, 0, 0, 0];

        unsafe {
            // Insert at beginning
            assert_eq!(
                rs_find_insertion_point(priorities.as_ptr(), orderings.as_ptr(), 5, 50, 0),
                0
            );

            // Insert in middle
            assert_eq!(
                rs_find_insertion_point(priorities.as_ptr(), orderings.as_ptr(), 5, 250, 0),
                2
            );

            // Insert at end
            assert_eq!(
                rs_find_insertion_point(priorities.as_ptr(), orderings.as_ptr(), 5, 600, 0),
                5
            );

            // Insert with same priority (uses ordering)
            assert_eq!(
                rs_find_insertion_point(priorities.as_ptr(), orderings.as_ptr(), 5, 300, 1),
                3
            );
        }
    }
}
