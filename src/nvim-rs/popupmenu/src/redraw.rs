//! Popup menu redraw logic helpers.
//!
//! This module provides helper functions for drawing the popup menu,
//! including scrollbar calculations, row rendering, and grid management.

use std::ffi::c_int;

// External C functions for redraw operations
extern "C" {
    /// Get `pum_first` static variable (index of top visible item).
    fn nvim_get_pum_first() -> c_int;
    /// Set `pum_first` static variable.
    fn nvim_set_pum_first(val: c_int);
    /// Get `pum_height` static variable.
    fn nvim_get_pum_height() -> c_int;
    /// Get `pum_size` static variable.
    fn nvim_get_pum_size() -> c_int;
    /// Get `pum_selected` static variable.
    fn nvim_get_pum_selected() -> c_int;
    /// Get `pum_base_width` static variable.
    fn nvim_get_pum_base_width() -> c_int;
    /// Get `pum_kind_width` static variable.
    fn nvim_get_pum_kind_width() -> c_int;
    /// Get `pum_extra_width` static variable.
    fn nvim_get_pum_extra_width() -> c_int;
}

/// Result of grid width calculation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumGridWidthResult {
    /// Total grid width.
    pub grid_width: c_int,
    /// Column offset for content.
    pub col_off: c_int,
    /// Whether extra space padding is added.
    pub extra_space: c_int,
}

/// Calculate the grid width and column offset for LTR mode.
///
/// # Arguments
/// * `pum_width` - Width of popup menu content
/// * `pum_col` - Column position
/// * `pum_scrollbar` - Whether scrollbar is present (0 or 1)
/// * `has_border` - Whether border is present (non-zero = yes)
///
/// Returns grid width, column offset, and extra space flag.
#[no_mangle]
pub const extern "C" fn rs_pum_grid_width_ltr(
    pum_width: c_int,
    pum_col: c_int,
    pum_scrollbar: c_int,
    has_border: c_int,
) -> PumGridWidthResult {
    let mut grid_width = pum_width;
    let min_col = 0;
    let extra_space = pum_col > min_col;
    let col_off = if extra_space { 1 } else { 0 };

    if extra_space {
        grid_width += 1;
    }

    // Add scrollbar width if present and no border
    if pum_scrollbar > 0 && has_border == 0 {
        grid_width += 1;
    }

    PumGridWidthResult {
        grid_width,
        col_off,
        extra_space: extra_space as c_int,
    }
}

/// Calculate the grid width and column offset for RTL mode.
///
/// # Arguments
/// * `pum_width` - Width of popup menu content
/// * `pum_col` - Column position
/// * `win_end_col` - End column of window
/// * `pum_scrollbar` - Whether scrollbar is present (0 or 1)
/// * `has_border` - Whether border is present (non-zero = yes)
///
/// Returns grid width, column offset, and extra space flag.
#[no_mangle]
pub const extern "C" fn rs_pum_grid_width_rtl(
    pum_width: c_int,
    pum_col: c_int,
    win_end_col: c_int,
    pum_scrollbar: c_int,
    has_border: c_int,
) -> PumGridWidthResult {
    let mut grid_width = pum_width;
    let mut col_off = pum_width - 1;
    let extra_space = pum_col < win_end_col - 1;

    if extra_space {
        grid_width += 1;
    }

    // Add scrollbar width if present and no border
    if pum_scrollbar > 0 && has_border == 0 {
        grid_width += 1;
        col_off += 1;
    }

    PumGridWidthResult {
        grid_width,
        col_off,
        extra_space: extra_space as c_int,
    }
}

/// Clamp `pum_first` to valid scroll range.
///
/// Ensures first visible item doesn't show empty space at bottom.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_clamp_first() {
    let pum_size = nvim_get_pum_size();
    let pum_height = nvim_get_pum_height();
    let pum_first = nvim_get_pum_first();

    let scroll_range = pum_size - pum_height;
    if pum_first > scroll_range {
        nvim_set_pum_first(scroll_range);
    }
}

/// Calculate the scroll range for the popup menu.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_scroll_range() -> c_int {
    nvim_get_pum_size() - nvim_get_pum_height()
}

/// Thumb position and height result.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumThumbInfo {
    /// Position of thumb (row index).
    pub pos: c_int,
    /// Height of thumb in rows.
    pub height: c_int,
}

/// Compute scrollbar thumb position and height.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_compute_thumb_from_state() -> PumThumbInfo {
    let pum_first = nvim_get_pum_first();
    let pum_height = nvim_get_pum_height();
    let pum_size = nvim_get_pum_size();

    if pum_size <= pum_height {
        return PumThumbInfo {
            pos: 0,
            height: pum_height,
        };
    }

    let scroll_range = pum_size - pum_height;
    let mut thumb_height = pum_height * pum_height / pum_size;
    if thumb_height == 0 {
        thumb_height = 1;
    }

    let thumb_pos = (pum_first * (pum_height - thumb_height) + scroll_range / 2) / scroll_range;

    PumThumbInfo {
        pos: thumb_pos,
        height: thumb_height,
    }
}

/// Check if a row index is within the scrollbar thumb.
///
/// # Arguments
/// * `row` - Row index relative to popup start
/// * `thumb_pos` - Start position of thumb
/// * `thumb_height` - Height of thumb
///
/// Returns 1 if row is in thumb, 0 otherwise.
#[no_mangle]
pub const extern "C" fn rs_pum_row_in_thumb(
    row: c_int,
    thumb_pos: c_int,
    thumb_height: c_int,
) -> c_int {
    (row >= thumb_pos && row < thumb_pos + thumb_height) as c_int
}

/// Calculate the item index for a given row.
///
/// # Arguments
/// * `row` - Row index (0-based, relative to popup content start)
///
/// Returns the item index in the items array.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_row_to_item(row: c_int) -> c_int {
    row + nvim_get_pum_first()
}

/// Check if a given item index is selected.
///
/// # Arguments
/// * `item_idx` - Item index to check
///
/// Returns 1 if selected, 0 otherwise.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_item_is_selected(item_idx: c_int) -> c_int {
    (item_idx == nvim_get_pum_selected()) as c_int
}

/// Column widths for popup menu rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumColumnWidths {
    /// Width of text/abbr column.
    pub base_width: c_int,
    /// Width of kind column.
    pub kind_width: c_int,
    /// Width of extra/menu column.
    pub extra_width: c_int,
}

/// Get the current column widths for popup menu rendering.
///
/// # Safety
/// Calls C accessor functions for global state.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_get_column_widths() -> PumColumnWidths {
    PumColumnWidths {
        base_width: nvim_get_pum_base_width(),
        kind_width: nvim_get_pum_kind_width(),
        extra_width: nvim_get_pum_extra_width(),
    }
}

/// Row rendering state for one popup menu row.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumRowState {
    /// Item index in the items array.
    pub item_idx: c_int,
    /// Whether this item is selected.
    pub is_selected: c_int,
    /// Current grid column position.
    pub grid_col: c_int,
    /// Total width used so far.
    pub total_width: c_int,
    /// Whether truncation indicator is needed.
    pub needs_trunc: c_int,
}

/// Initialize row rendering state for a given row.
///
/// # Arguments
/// * `row` - Row index (0-based)
/// * `col_off` - Column offset for content
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_init_row_state(row: c_int, col_off: c_int) -> PumRowState {
    let item_idx = row + nvim_get_pum_first();
    let is_selected = (item_idx == nvim_get_pum_selected()) as c_int;

    PumRowState {
        item_idx,
        is_selected,
        grid_col: col_off,
        total_width: 0,
        needs_trunc: 0,
    }
}

/// Calculate the fill position range for padding between columns.
///
/// # Arguments
/// * `col_off` - Column offset for content
/// * `grid_col` - Current grid column position
/// * `basic_width` - Width of first column
/// * `n` - Additional spacing needed
/// * `is_rl` - Whether right-to-left mode
///
/// Returns `(start_col, end_col)` for fill operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumFillRange {
    /// Start column for fill.
    pub start: c_int,
    /// End column for fill (exclusive).
    pub end: c_int,
    /// New grid column position after fill.
    pub new_grid_col: c_int,
}

#[no_mangle]
pub const extern "C" fn rs_pum_fill_range(
    col_off: c_int,
    grid_col: c_int,
    basic_width: c_int,
    n: c_int,
    is_rl: c_int,
) -> PumFillRange {
    if is_rl != 0 {
        PumFillRange {
            start: col_off - basic_width - n + 1,
            end: grid_col + 1,
            new_grid_col: col_off - basic_width - n,
        }
    } else {
        PumFillRange {
            start: grid_col,
            end: col_off + basic_width + n,
            new_grid_col: col_off + basic_width + n,
        }
    }
}

/// Calculate the scrollbar column position.
///
/// # Arguments
/// * `col_off` - Column offset for content
/// * `pum_width` - Width of popup menu content
/// * `is_rl` - Whether right-to-left mode
///
/// Returns the column for scrollbar rendering.
#[no_mangle]
pub const extern "C" fn rs_pum_scrollbar_col(
    col_off: c_int,
    pum_width: c_int,
    is_rl: c_int,
) -> c_int {
    if is_rl != 0 {
        col_off - pum_width
    } else {
        col_off + pum_width
    }
}

/// Calculate spacing value 'n' for column padding.
///
/// This determines additional spacing between columns based on column
/// order and item types.
///
/// # Arguments
/// * `j` - Current column index (0, 1, or 2)
/// * `items_width_kind` - Width of kind column
/// * `last_is_abbr` - Whether last column is abbr type
/// * `order_j` - Type of column j in order
///
/// Returns spacing value.
#[no_mangle]
pub const extern "C" fn rs_pum_column_spacing(
    j: c_int,
    items_width_kind: c_int,
    last_is_abbr: c_int,
    order_j: c_int,
) -> c_int {
    // CPT_ABBR = 0
    const CPT_ABBR: c_int = 0;

    if j > 0 {
        items_width_kind + if last_is_abbr != 0 { 0 } else { 1 }
    } else if order_j == CPT_ABBR {
        1
    } else {
        0
    }
}

/// Check if we should stop rendering columns for this row.
///
/// # Arguments
/// * `j` - Current column index (0, 1, or 2)
/// * `next_is_empty` - Whether next column is empty
/// * `next_next_is_empty` - Whether column after next is empty
/// * `basic_width` - Width of first column
/// * `n` - Additional spacing
/// * `pum_width` - Total popup width
///
/// Returns 1 if should stop, 0 otherwise.
#[no_mangle]
pub const extern "C" fn rs_pum_should_stop_columns(
    j: c_int,
    next_is_empty: c_int,
    next_next_is_empty: c_int,
    basic_width: c_int,
    n: c_int,
    pum_width: c_int,
) -> c_int {
    let stop = (j == 2)
        || (next_is_empty != 0 && (j == 1 || (j == 0 && next_next_is_empty != 0)))
        || (basic_width + n >= pum_width);
    stop as c_int
}

/// Compute the truncation fill range at end of row.
///
/// # Arguments
/// * `col_off` - Column offset for content
/// * `grid_col` - Current grid column position
/// * `pum_width` - Total popup width
/// * `is_rl` - Whether right-to-left mode
///
/// Returns fill range for end of row.
#[no_mangle]
pub const extern "C" fn rs_pum_end_fill_range(
    col_off: c_int,
    grid_col: c_int,
    pum_width: c_int,
    is_rl: c_int,
) -> PumFillRange {
    if is_rl != 0 {
        let lcol = col_off - pum_width + 1;
        PumFillRange {
            start: lcol,
            end: grid_col + 1,
            new_grid_col: lcol,
        }
    } else {
        let rcol = col_off + pum_width;
        PumFillRange {
            start: grid_col,
            end: rcol,
            new_grid_col: rcol,
        }
    }
}

/// Get the column position for truncation indicator.
///
/// # Arguments
/// * `col_off` - Column offset for content
/// * `pum_width` - Total popup width
/// * `is_rl` - Whether right-to-left mode
///
/// Returns column for truncation indicator.
#[no_mangle]
pub const extern "C" fn rs_pum_trunc_col(col_off: c_int, pum_width: c_int, is_rl: c_int) -> c_int {
    if is_rl != 0 {
        col_off - pum_width + 1
    } else {
        col_off + pum_width - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_width_ltr_with_extra() {
        let result = rs_pum_grid_width_ltr(20, 5, 1, 0);
        assert_eq!(result.grid_width, 22); // 20 + 1 (extra) + 1 (scrollbar)
        assert_eq!(result.col_off, 1);
        assert_eq!(result.extra_space, 1);
    }

    #[test]
    fn test_grid_width_ltr_at_zero() {
        let result = rs_pum_grid_width_ltr(20, 0, 0, 0);
        assert_eq!(result.grid_width, 20);
        assert_eq!(result.col_off, 0);
        assert_eq!(result.extra_space, 0);
    }

    #[test]
    fn test_grid_width_rtl() {
        let result = rs_pum_grid_width_rtl(20, 30, 80, 1, 0);
        assert_eq!(result.grid_width, 22); // 20 + 1 (extra) + 1 (scrollbar)
        assert_eq!(result.col_off, 20); // 19 + 1 for scrollbar
        assert_eq!(result.extra_space, 1);
    }

    #[test]
    fn test_row_in_thumb() {
        assert_eq!(rs_pum_row_in_thumb(5, 4, 3), 1); // 5 is in [4, 7)
        assert_eq!(rs_pum_row_in_thumb(3, 4, 3), 0); // 3 is not in [4, 7)
        assert_eq!(rs_pum_row_in_thumb(7, 4, 3), 0); // 7 is not in [4, 7)
    }

    #[test]
    fn test_fill_range_ltr() {
        let result = rs_pum_fill_range(1, 10, 15, 2, 0);
        assert_eq!(result.start, 10);
        assert_eq!(result.end, 18); // 1 + 15 + 2
        assert_eq!(result.new_grid_col, 18);
    }

    #[test]
    fn test_fill_range_rtl() {
        let result = rs_pum_fill_range(25, 20, 15, 2, 1);
        assert_eq!(result.start, 9); // 25 - 15 - 2 + 1
        assert_eq!(result.end, 21);
        assert_eq!(result.new_grid_col, 8); // 25 - 15 - 2
    }

    #[test]
    fn test_scrollbar_col() {
        assert_eq!(rs_pum_scrollbar_col(1, 20, 0), 21); // LTR
        assert_eq!(rs_pum_scrollbar_col(25, 20, 1), 5); // RTL
    }

    #[test]
    fn test_column_spacing() {
        assert_eq!(rs_pum_column_spacing(0, 5, 0, 0), 1); // abbr column
        assert_eq!(rs_pum_column_spacing(0, 5, 0, 1), 0); // kind column
        assert_eq!(rs_pum_column_spacing(1, 5, 0, 0), 6); // j=1, not last_is_abbr
        assert_eq!(rs_pum_column_spacing(1, 5, 1, 0), 5); // j=1, last_is_abbr
    }

    #[test]
    fn test_should_stop_columns() {
        // j=2 always stops
        assert_eq!(rs_pum_should_stop_columns(2, 0, 0, 10, 2, 30), 1);
        // j=1 with next empty
        assert_eq!(rs_pum_should_stop_columns(1, 1, 0, 10, 2, 30), 1);
        // j=0 with next two empty
        assert_eq!(rs_pum_should_stop_columns(0, 1, 1, 10, 2, 30), 1);
        // width overflow
        assert_eq!(rs_pum_should_stop_columns(0, 0, 0, 20, 15, 30), 1);
        // normal continue
        assert_eq!(rs_pum_should_stop_columns(0, 0, 0, 10, 2, 30), 0);
    }

    #[test]
    fn test_trunc_col() {
        assert_eq!(rs_pum_trunc_col(1, 20, 0), 20); // LTR: col_off + pum_width - 1
        assert_eq!(rs_pum_trunc_col(25, 20, 1), 6); // RTL: col_off - pum_width + 1
    }
}
