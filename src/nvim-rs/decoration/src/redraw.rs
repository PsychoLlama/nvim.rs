//! Decoration redraw functions for line rendering
//!
//! This module contains Rust implementations for decoration redraw operations
//! during line rendering, migrated from `src/nvim/decoration.c`.

use std::ffi::c_int;

use crate::{
    DecorKind, DecorRangeHandle, DecorStateHandle, VirtTextPos, WinHandle, DRAW_COL_JUST_ADDED,
    DRAW_COL_UNSET,
};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // DecorState accessors
    fn nvim_decor_state_get_row(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_set_eol_col(state: DecorStateHandle, val: c_int);
    fn nvim_decor_state_get_current_end(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_future_begin(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_ranges_count(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_range_by_idx(state: DecorStateHandle, idx: c_int) -> DecorRangeHandle;
    fn nvim_decor_state_get_range(state: DecorStateHandle, idx: c_int) -> DecorRangeHandle;

    // DecorRange accessors
    fn nvim_decor_range_get_start_row(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_kind(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_draw_col(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_has_virt_pos(range: DecorRangeHandle) -> bool;
    fn nvim_decor_range_get_virt_pos_kind(range: DecorRangeHandle) -> c_int;

    // DecorRange virt_text accessors
    fn nvim_decor_range_get_virt_text(range: DecorRangeHandle) -> crate::DecorVirtTextHandle;
    fn nvim_decor_virt_text_get_width(vt: crate::DecorVirtTextHandle) -> c_int;

    // C functions for decor operations
    fn decor_redraw_start(wp: WinHandle, top_row: c_int, state: DecorStateHandle) -> bool;
}

// =============================================================================
// Redraw Line State
// =============================================================================

/// State for redrawing a decoration line.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RedrawLineState {
    /// Current row being processed
    pub row: c_int,
    /// Current column until decorations are valid
    pub col_until: c_int,
    /// EOL column for this line
    pub eol_col: c_int,
    /// Whether any virtual text was encountered
    pub has_virt_text: bool,
    /// Total width of EOL virtual text
    pub eol_virt_width: c_int,
    /// Total width of right-aligned EOL virtual text
    pub eol_right_width: c_int,
}

impl RedrawLineState {
    /// Create new state for a row.
    #[must_use]
    pub const fn new(row: c_int) -> Self {
        Self {
            row,
            col_until: -1,
            eol_col: -1,
            has_virt_text: false,
            eol_virt_width: 0,
            eol_right_width: 0,
        }
    }

    /// Reset for a new row.
    pub fn reset(&mut self, row: c_int) {
        self.row = row;
        self.col_until = -1;
        self.eol_col = -1;
        self.has_virt_text = false;
        self.eol_virt_width = 0;
        self.eol_right_width = 0;
    }
}

/// FFI: Create redraw line state.
#[no_mangle]
pub extern "C" fn rs_redraw_line_state_new(row: c_int) -> RedrawLineState {
    RedrawLineState::new(row)
}

/// FFI: Reset redraw line state.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_redraw_line_state_reset(state: *mut RedrawLineState, row: c_int) {
    if !state.is_null() {
        (*state).reset(row);
    }
}

// =============================================================================
// Redraw Line Functions
// =============================================================================

/// Check if there are more decorations on the current row.
///
/// This checks both active and future ranges to determine if redraw
/// should continue processing decorations.
#[no_mangle]
pub extern "C" fn rs_decor_has_more_decorations(state: DecorStateHandle, _row: c_int) -> c_int {
    if state.is_null() {
        return 0;
    }

    let current_end = unsafe { nvim_decor_state_get_current_end(state) };
    let future_begin = unsafe { nvim_decor_state_get_future_begin(state) };
    let ranges_count = unsafe { nvim_decor_state_get_ranges_count(state) };

    // If there are active or pending ranges
    if current_end != 0 || future_begin != ranges_count {
        return 1;
    }

    // Check marktree iterator (handled by C caller)
    0
}

/// Calculate EOL virtual text widths for the current row.
///
/// This iterates through active decoration ranges to sum up:
/// - Total EOL virtual text width
/// - Right-aligned EOL virtual text width
///
/// # Safety
/// `total_width` and `right_width` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_calc_eol_virt_widths(
    state: DecorStateHandle,
    row: c_int,
    total_width: *mut c_int,
    right_width: *mut c_int,
) {
    if state.is_null() {
        return;
    }

    let current_end = unsafe { nvim_decor_state_get_current_end(state) };
    let mut total: c_int = 0;
    let mut right: c_int = 0;

    for i in 0..current_end {
        let range = unsafe { nvim_decor_state_get_range(state, i) };
        if range.is_null() {
            continue;
        }

        let start_row = unsafe { nvim_decor_range_get_start_row(range) };
        if start_row != row {
            continue;
        }

        if !unsafe { nvim_decor_range_has_virt_pos(range) } {
            continue;
        }

        let draw_col = unsafe { nvim_decor_range_get_draw_col(range) };
        if draw_col != DRAW_COL_UNSET {
            continue;
        }

        let pos_kind = unsafe { nvim_decor_range_get_virt_pos_kind(range) };
        let pos = VirtTextPos::from_c_int(pos_kind);

        let kind_raw = unsafe { nvim_decor_range_get_kind(range) };
        let kind = DecorKind::from_c_int(kind_raw);

        if kind != Some(DecorKind::VirtText) {
            continue;
        }

        let vt = unsafe { nvim_decor_range_get_virt_text(range) };
        if vt.is_null() {
            continue;
        }

        let width = unsafe { nvim_decor_virt_text_get_width(vt) };

        match pos {
            Some(VirtTextPos::EndOfLine) => {
                total += width + 1; // +1 for spacing
            }
            Some(VirtTextPos::EndOfLineRightAlign) => {
                total += width + 1;
                right += width + 1;
            }
            _ => {}
        }
    }

    // Remove trailing spacing
    if total > 0 {
        total -= 1;
    }
    if right > 0 {
        right -= 1;
    }

    if !total_width.is_null() {
        *total_width = total;
    }
    if !right_width.is_null() {
        *right_width = right;
    }
}

/// Check if any decoration range on the current row has virtual text position.
#[no_mangle]
pub extern "C" fn rs_decor_row_has_virt_pos(state: DecorStateHandle, row: c_int) -> c_int {
    if state.is_null() {
        return 0;
    }

    let current_end = unsafe { nvim_decor_state_get_current_end(state) };

    for i in 0..current_end {
        let range = unsafe { nvim_decor_state_get_range(state, i) };
        if range.is_null() {
            continue;
        }

        let start_row = unsafe { nvim_decor_range_get_start_row(range) };
        if start_row == row && unsafe { nvim_decor_range_has_virt_pos(range) } {
            return 1;
        }
    }

    0
}

/// Count virtual text items at a specific position type on the current row.
#[no_mangle]
pub extern "C" fn rs_count_virt_at_pos(
    state: DecorStateHandle,
    row: c_int,
    pos_type: c_int,
) -> c_int {
    if state.is_null() {
        return 0;
    }

    let target_pos = VirtTextPos::from_c_int(pos_type);
    let current_end = unsafe { nvim_decor_state_get_current_end(state) };
    let mut count: c_int = 0;

    for i in 0..current_end {
        let range = unsafe { nvim_decor_state_get_range(state, i) };
        if range.is_null() {
            continue;
        }

        let start_row = unsafe { nvim_decor_range_get_start_row(range) };
        if start_row != row {
            continue;
        }

        let kind_raw = unsafe { nvim_decor_range_get_kind(range) };
        if DecorKind::from_c_int(kind_raw) != Some(DecorKind::VirtText) {
            continue;
        }

        let pos_kind = unsafe { nvim_decor_range_get_virt_pos_kind(range) };
        if VirtTextPos::from_c_int(pos_kind) == target_pos {
            count += 1;
        }
    }

    count
}

/// Get the next pending virtual text item on the current row.
///
/// Returns the range index, or -1 if none found.
#[no_mangle]
pub extern "C" fn rs_next_pending_virt_text(
    state: DecorStateHandle,
    row: c_int,
    pos_type: c_int,
) -> c_int {
    if state.is_null() {
        return -1;
    }

    let target_pos = VirtTextPos::from_c_int(pos_type);
    let current_end = unsafe { nvim_decor_state_get_current_end(state) };

    for i in 0..current_end {
        let range = unsafe { nvim_decor_state_get_range(state, i) };
        if range.is_null() {
            continue;
        }

        let start_row = unsafe { nvim_decor_range_get_start_row(range) };
        if start_row != row {
            continue;
        }

        let draw_col = unsafe { nvim_decor_range_get_draw_col(range) };
        if draw_col != DRAW_COL_UNSET && draw_col != DRAW_COL_JUST_ADDED {
            continue;
        }

        let kind_raw = unsafe { nvim_decor_range_get_kind(range) };
        if DecorKind::from_c_int(kind_raw) != Some(DecorKind::VirtText) {
            continue;
        }

        let pos_kind = unsafe { nvim_decor_range_get_virt_pos_kind(range) };
        if VirtTextPos::from_c_int(pos_kind) == target_pos {
            return i;
        }
    }

    -1
}

// =============================================================================
// Range Iteration Helpers
// =============================================================================

/// Iterator state for walking decoration ranges.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RangeIterator {
    /// Current index in ranges_i array
    pub index: c_int,
    /// End index (current_end for active, ranges_count for all)
    pub end_index: c_int,
    /// Filter by row (-1 = no filter)
    pub filter_row: c_int,
    /// Filter by kind (-1 = no filter)
    pub filter_kind: c_int,
}

impl RangeIterator {
    /// Create iterator for active ranges.
    #[must_use]
    pub const fn active(current_end: c_int) -> Self {
        Self {
            index: 0,
            end_index: current_end,
            filter_row: -1,
            filter_kind: -1,
        }
    }

    /// Create iterator for ranges on a specific row.
    #[must_use]
    pub const fn for_row(current_end: c_int, row: c_int) -> Self {
        Self {
            index: 0,
            end_index: current_end,
            filter_row: row,
            filter_kind: -1,
        }
    }

    /// Create iterator for ranges of a specific kind.
    #[must_use]
    pub const fn for_kind(current_end: c_int, kind: DecorKind) -> Self {
        Self {
            index: 0,
            end_index: current_end,
            filter_row: -1,
            filter_kind: kind as c_int,
        }
    }

    /// Check if iterator has more items.
    #[must_use]
    pub const fn has_next(&self) -> bool {
        self.index < self.end_index
    }

    /// Advance to next item.
    pub fn advance(&mut self) {
        self.index += 1;
    }
}

/// FFI: Create active range iterator.
#[no_mangle]
pub extern "C" fn rs_range_iterator_active(state: DecorStateHandle) -> RangeIterator {
    if state.is_null() {
        return RangeIterator::default();
    }
    let current_end = unsafe { nvim_decor_state_get_current_end(state) };
    RangeIterator::active(current_end)
}

/// FFI: Create row-filtered iterator.
#[no_mangle]
pub extern "C" fn rs_range_iterator_for_row(state: DecorStateHandle, row: c_int) -> RangeIterator {
    if state.is_null() {
        return RangeIterator::default();
    }
    let current_end = unsafe { nvim_decor_state_get_current_end(state) };
    RangeIterator::for_row(current_end, row)
}

/// FFI: Check if iterator has next.
#[no_mangle]
pub extern "C" fn rs_range_iterator_has_next(iter: RangeIterator) -> c_int {
    c_int::from(iter.has_next())
}

/// FFI: Advance iterator.
///
/// # Safety
/// `iter` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_range_iterator_advance(iter: *mut RangeIterator) {
    if !iter.is_null() {
        (*iter).advance();
    }
}

/// FFI: Get current range from iterator.
#[no_mangle]
pub extern "C" fn rs_range_iterator_current(
    state: DecorStateHandle,
    iter: RangeIterator,
) -> DecorRangeHandle {
    if state.is_null() || !iter.has_next() {
        return DecorRangeHandle(std::ptr::null_mut());
    }
    unsafe { nvim_decor_state_get_range(state, iter.index) }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redraw_line_state() {
        let state = RedrawLineState::new(10);
        assert_eq!(state.row, 10);
        assert_eq!(state.col_until, -1);
        assert_eq!(state.eol_col, -1);
        assert!(!state.has_virt_text);
    }

    #[test]
    fn test_redraw_line_state_reset() {
        let mut state = RedrawLineState::new(10);
        state.has_virt_text = true;
        state.eol_virt_width = 50;

        state.reset(20);
        assert_eq!(state.row, 20);
        assert!(!state.has_virt_text);
        assert_eq!(state.eol_virt_width, 0);
    }

    #[test]
    fn test_range_iterator() {
        let iter = RangeIterator::active(5);
        assert_eq!(iter.index, 0);
        assert_eq!(iter.end_index, 5);
        assert!(iter.has_next());
        assert_eq!(iter.filter_row, -1);
    }

    #[test]
    fn test_range_iterator_for_row() {
        let iter = RangeIterator::for_row(10, 42);
        assert_eq!(iter.filter_row, 42);
        assert!(iter.has_next());
    }

    #[test]
    fn test_range_iterator_advance() {
        let mut iter = RangeIterator::active(2);
        assert!(iter.has_next());
        iter.advance();
        assert!(iter.has_next());
        iter.advance();
        assert!(!iter.has_next());
    }

    #[test]
    fn test_range_iterator_empty() {
        let iter = RangeIterator::active(0);
        assert!(!iter.has_next());
    }
}
