//! Decoration range priority and insertion logic
//!
//! This module contains Rust implementations for decoration range
//! priority comparison and insertion, migrated from `src/nvim/decoration.c`.

use std::ffi::{c_int, c_void};

use crate::decor::{
    DECOR_ID_INVALID, DECOR_PRIORITY_BASE, KSH_CONCEAL, KSH_IS_SIGN, KSH_SPELL_OFF, KSH_SPELL_ON,
    KSH_UI_WATCHED, KSH_UI_WATCHED_OVERLAY,
};
use crate::{DecorKind, DecorStateHandle, VirtTextPos, KVT_IS_LINES};

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
// Phase 4: External C Functions
// =============================================================================

/// Opaque handle to DecorSignHighlight.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct DecorShHandle(*mut c_void);

impl DecorShHandle {
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to DecorVirtText.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct DecorVtHandle(*mut c_void);

impl DecorVtHandle {
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

extern "C" {
    // Range insertion helpers (construct DecorRange in C, call decor_range_insert)
    fn nvim_decor_range_insert_vt(
        state: DecorStateHandle,
        start_row: c_int,
        start_col: c_int,
        end_row: c_int,
        end_col: c_int,
        vt_ptr: DecorVtHandle,
        owned: bool,
        kind: c_int,
        priority_internal: u32,
    );
    fn nvim_decor_range_insert_hl(
        state: DecorStateHandle,
        start_row: c_int,
        start_col: c_int,
        end_row: c_int,
        end_col: c_int,
        sh_ptr: DecorShHandle,
        owned: bool,
        priority_internal: u32,
        attr_id: c_int,
    );
    fn nvim_decor_range_insert_ui(
        state: DecorStateHandle,
        start_row: c_int,
        start_col: c_int,
        end_row: c_int,
        end_col: c_int,
        ns_id: u32,
        mark_id: u32,
        pos: c_int,
        owned: bool,
        priority_internal: u32,
        attr_id: c_int,
    );

    // DecorSignHighlight field accessors
    fn nvim_decor_sh_get_flags(sh: DecorShHandle) -> u16;
    fn nvim_decor_sh_ptr_get_priority(sh: DecorShHandle) -> u16;
    fn nvim_decor_sh_ptr_get_hl_id(sh: DecorShHandle) -> c_int;
    fn nvim_decor_sh_ptr_get_url(sh: DecorShHandle) -> *const std::ffi::c_char;
    fn nvim_decor_sh_ptr_get_next(sh: DecorShHandle) -> u32;

    // DecorVirtText field accessors
    fn nvim_decor_vt_ptr_get_flags(vt: DecorVtHandle) -> u8;
    fn nvim_decor_vt_ptr_get_priority(vt: DecorVtHandle) -> u16;
    fn nvim_decor_vt_ptr_get_next(vt: DecorVtHandle) -> DecorVtHandle;

    // decor_items global accessors
    fn nvim_decor_items_get(idx: u32) -> DecorShHandle;

    // syn_id2attr
    fn nvim_syn_id2attr(hl_id: c_int) -> c_int;
}

// =============================================================================
// Phase 4: Range Creation Functions
// =============================================================================

/// Add a virtual text decoration range to the state.
///
/// Constructs the appropriate DecorRange (VirtText or VirtLines) and inserts it.
///
/// Rust implementation of `decor_range_add_virt()`.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_range_add_virt(
    state: DecorStateHandle,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
    vt: DecorVtHandle,
    owned: bool,
) {
    if state.is_null() || vt.is_null() {
        return;
    }

    let vt_flags = nvim_decor_vt_ptr_get_flags(vt);
    let is_lines = vt_flags & KVT_IS_LINES != 0;
    let kind = if is_lines {
        DecorKind::VirtLines as c_int
    } else {
        DecorKind::VirtText as c_int
    };
    let priority = nvim_decor_vt_ptr_get_priority(vt);
    let priority_internal = u32::from(priority) << 16;

    nvim_decor_range_insert_vt(
        state,
        start_row,
        start_col,
        end_row,
        end_col,
        vt,
        owned,
        kind,
        priority_internal,
    );
}

/// Add a sign/highlight decoration range to the state.
///
/// Skips signs. For highlights, inserts if the sh has hl_id, url, conceal, or spell.
/// For UI watched items, inserts an additional UI watched range.
///
/// Rust implementation of `decor_range_add_sh()`.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_range_add_sh(
    state: DecorStateHandle,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
    sh: DecorShHandle,
    owned: bool,
    ns: u32,
    mark_id: u32,
    subpriority: u16,
) {
    if state.is_null() || sh.is_null() {
        return;
    }

    let flags = nvim_decor_sh_get_flags(sh);

    // Skip signs
    if flags & KSH_IS_SIGN != 0 {
        return;
    }

    let priority = nvim_decor_sh_ptr_get_priority(sh);
    let priority_internal = (u32::from(priority) << 16) + u32::from(subpriority);
    let hl_id = nvim_decor_sh_ptr_get_hl_id(sh);
    let url = nvim_decor_sh_ptr_get_url(sh);

    // Insert highlight range if there's something to highlight
    let has_hl = hl_id != 0;
    let has_url = !url.is_null();
    let has_conceal = flags & KSH_CONCEAL != 0;
    let has_spell_on = flags & KSH_SPELL_ON != 0;
    let has_spell_off = flags & KSH_SPELL_OFF != 0;

    if has_hl || has_url || has_conceal || has_spell_on || has_spell_off {
        let attr_id = if has_hl { nvim_syn_id2attr(hl_id) } else { 0 };

        nvim_decor_range_insert_hl(
            state,
            start_row,
            start_col,
            end_row,
            end_col,
            sh,
            owned,
            priority_internal,
            attr_id,
        );
    }

    // Insert UI watched range if applicable
    if flags & KSH_UI_WATCHED != 0 {
        let pos = if flags & KSH_UI_WATCHED_OVERLAY != 0 {
            VirtTextPos::Overlay as c_int
        } else {
            VirtTextPos::EndOfLine as c_int
        };
        // attr_id for UI watched ranges is 0 (or same as highlight if also highlight)
        let attr_id = if has_hl { nvim_syn_id2attr(hl_id) } else { 0 };

        nvim_decor_range_insert_ui(
            state,
            start_row,
            start_col,
            end_row,
            end_col,
            ns,
            mark_id,
            pos,
            owned,
            priority_internal,
            attr_id,
        );
    }
}

/// Dispatch inline decoration data to the appropriate range addition functions.
///
/// For ext decorations: walks the vt linked list and sh linked list.
/// For inline highlights: converts to DecorSignHighlight and adds.
///
/// Rust implementation of `decor_range_add_from_inline()`.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_range_add_from_inline(
    state: DecorStateHandle,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
    ext: bool,
    vt: DecorVtHandle,
    sh_idx: u32,
    hl_flags: u16,
    hl_priority: u16,
    hl_hl_id: c_int,
    hl_conceal_char: u32,
    owned: bool,
    ns: u32,
    mark_id: u32,
) {
    if state.is_null() {
        return;
    }

    if ext {
        // Walk virtual text linked list
        let mut cur_vt = vt;
        while !cur_vt.is_null() {
            rs_decor_range_add_virt(state, start_row, start_col, end_row, end_col, cur_vt, owned);
            cur_vt = nvim_decor_vt_ptr_get_next(cur_vt);
        }

        // Walk sign/highlight linked list
        let mut idx = sh_idx;
        while idx != DECOR_ID_INVALID {
            let sh = nvim_decor_items_get(idx);
            rs_decor_range_add_sh(
                state, start_row, start_col, end_row, end_col, sh, owned, ns, mark_id, 0,
            );
            idx = nvim_decor_sh_ptr_get_next(sh);
        }
    } else {
        // Inline highlight - create a temporary DecorSignHighlight via
        // decor_sh_from_inline and add it. We call through the existing
        // rs_decor_sh_from_inline to get the struct, then use the C helper
        // to build the range.
        //
        // For inline highlights, we create a stack-local DecorSignHighlight
        // by calling the C-side decor_sh_from_inline and then passing it.
        // However, since we can't easily construct the struct in Rust,
        // we call a C helper that does the conversion and insertion.
        nvim_decor_range_add_from_inline_hl(
            state,
            start_row,
            start_col,
            end_row,
            end_col,
            hl_flags,
            hl_priority,
            hl_hl_id,
            hl_conceal_char,
            owned,
            ns,
            mark_id,
        );
    }
}

extern "C" {
    fn nvim_decor_range_add_from_inline_hl(
        state: DecorStateHandle,
        start_row: c_int,
        start_col: c_int,
        end_row: c_int,
        end_col: c_int,
        hl_flags: u16,
        hl_priority: u16,
        hl_hl_id: c_int,
        hl_conceal_char: u32,
        owned: bool,
        ns: u32,
        mark_id: u32,
    );
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
