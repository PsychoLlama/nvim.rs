//! Popup menu mouse handling.
//!
//! This module provides helper functions for mouse position detection
//! and selection in the popup menu.

use std::ffi::c_int;

// C accessor functions for mouse state.
extern "C" {
    /// Get the `pum_row` static variable.
    fn nvim_get_pum_row() -> c_int;
    /// Get the `pum_width` static variable.
    fn nvim_get_pum_width() -> c_int;
    /// Get the `pum_height` static variable.
    fn nvim_get_pum_height() -> c_int;
    /// Get the `pum_scrollbar` static variable.
    fn nvim_get_pum_scrollbar() -> c_int;
    /// Get the `pum_rl` static variable.
    fn nvim_get_pum_rl() -> c_int;
    /// Get the `pum_left_col` static variable.
    fn nvim_get_pum_left_col() -> c_int;
    /// Get the `pum_right_col` static variable.
    fn nvim_get_pum_right_col() -> c_int;
    /// Get the `pum_win_row_offset` static variable.
    fn nvim_get_pum_win_row_offset() -> c_int;
    /// Get the `pum_win_col_offset` static variable.
    fn nvim_get_pum_win_col_offset() -> c_int;
    /// Get the `pum_anchor_grid` static variable.
    fn nvim_get_pum_anchor_grid() -> c_int;
}

/// Result of mouse position check within popup menu.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumMousePos {
    /// Whether the mouse is within the popup menu area.
    pub in_pum: c_int,
    /// The row index within the popup menu (0-based), or -1 if outside.
    pub row_idx: c_int,
    /// Whether the mouse is on the scrollbar column.
    pub on_scrollbar: c_int,
}

/// Check if a grid position is within the popup menu.
///
/// # Arguments
/// * `grid` - Grid handle where mouse event occurred
/// * `row` - Row position in the grid
/// * `col` - Column position in the grid
/// * `pum_grid_handle` - Handle of the popup menu grid (0 if not drawn)
///
/// Returns position information including whether click is in popup.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_check_mouse_pos(
    grid: c_int,
    row: c_int,
    col: c_int,
    pum_grid_handle: c_int,
) -> PumMousePos {
    let pum_row = nvim_get_pum_row();
    let pum_height = nvim_get_pum_height();
    let pum_left_col = nvim_get_pum_left_col();
    let pum_right_col = nvim_get_pum_right_col();
    let pum_anchor_grid = nvim_get_pum_anchor_grid();
    let pum_win_row_offset = nvim_get_pum_win_row_offset();
    let pum_win_col_offset = nvim_get_pum_win_col_offset();
    let pum_scrollbar = nvim_get_pum_scrollbar();
    let pum_rl = nvim_get_pum_rl() != 0;

    // If clicking directly on the pum grid
    if pum_grid_handle != 0 && grid == pum_grid_handle {
        let on_scrollbar = if pum_scrollbar != 0 {
            if pum_rl {
                (col == 0) as c_int
            } else {
                let pum_width = nvim_get_pum_width();
                (col >= pum_width) as c_int
            }
        } else {
            0
        };

        return PumMousePos {
            in_pum: 1,
            row_idx: row,
            on_scrollbar,
        };
    }

    // Check if on the anchor grid within popup bounds
    if grid != pum_anchor_grid {
        return PumMousePos {
            in_pum: 0,
            row_idx: -1,
            on_scrollbar: 0,
        };
    }

    let adjusted_col_left = pum_left_col - pum_win_col_offset;
    let adjusted_col_right = pum_right_col - pum_win_col_offset;

    if col < adjusted_col_left || col >= adjusted_col_right {
        return PumMousePos {
            in_pum: 0,
            row_idx: -1,
            on_scrollbar: 0,
        };
    }

    let adjusted_row = pum_row - pum_win_row_offset;
    let row_idx = row - adjusted_row;

    if row_idx < 0 || row_idx >= pum_height {
        return PumMousePos {
            in_pum: 0,
            row_idx: -1,
            on_scrollbar: 0,
        };
    }

    // Check if on scrollbar
    let on_scrollbar = if pum_scrollbar != 0 {
        if pum_rl {
            (col == adjusted_col_left) as c_int
        } else {
            (col == adjusted_col_right - 1) as c_int
        }
    } else {
        0
    };

    PumMousePos {
        in_pum: 1,
        row_idx,
        on_scrollbar,
    }
}

/// Result of mouse position calculation for popup placement.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumMousePlacement {
    /// Row position for popup menu.
    pub row: c_int,
    /// Column position for popup menu.
    pub col: c_int,
    /// Width of popup menu.
    pub width: c_int,
    /// Height of popup menu.
    pub height: c_int,
    /// Whether popup is above mouse position.
    pub above: c_int,
}

/// Calculate popup placement at mouse position.
///
/// # Arguments
/// * `mouse_row` - Mouse row position (adjusted for window)
/// * `mouse_col` - Mouse column position (adjusted for window)
/// * `pum_size` - Number of items in popup
/// * `pum_height` - Current height of popup (may be adjusted)
/// * `base_width` - Base width of popup content
/// * `min_width` - Minimum width required
/// * `min_row` - Minimum row boundary
/// * `min_col` - Minimum column boundary
/// * `max_row` - Maximum row boundary
/// * `max_col` - Maximum column boundary
/// * `is_rl` - Whether right-to-left mode
///
/// Returns placement result with row, col, width, height, and above flag.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn rs_pum_calc_mouse_placement(
    mouse_row: c_int,
    mouse_col: c_int,
    pum_size: c_int,
    pum_height: c_int,
    base_width: c_int,
    min_width: c_int,
    min_row: c_int,
    min_col: c_int,
    max_row: c_int,
    max_col: c_int,
    is_rl: c_int,
) -> PumMousePlacement {
    let is_rl = is_rl != 0;
    let mut height = pum_height;
    let row;
    let above;

    // Vertical placement
    if max_row - mouse_row > pum_size || max_row - mouse_row > mouse_row - min_row {
        // Enough space below, or more space below than above
        above = 0;
        row = mouse_row + 1;
        if height > max_row - row {
            height = max_row - row;
        }
    } else {
        // Show above mouse row
        above = 1;
        let mut r = mouse_row - pum_size;
        if r < min_row {
            height += r - min_row;
            r = min_row;
        }
        row = r;
    }

    // Horizontal placement
    let col;
    let width;
    let effective_min_width = if base_width < min_width {
        min_width
    } else {
        base_width
    };

    if is_rl {
        if mouse_col - min_col + 1 >= base_width || mouse_col - min_col + 1 > min_width {
            // Enough space at mouse column
            col = mouse_col;
        } else {
            // Left align with window
            col = min_col + effective_min_width.min(min_width) - 1;
        }
        width = (col - min_col + 1).min(base_width + 1);
    } else {
        if max_col - mouse_col >= base_width || max_col - mouse_col > min_width {
            // Enough space at mouse column
            col = mouse_col;
        } else {
            // Right align with window
            col = max_col - effective_min_width.min(min_width);
        }
        width = (max_col - col).min(base_width + 1);
    }

    PumMousePlacement {
        row,
        col,
        width,
        height,
        above,
    }
}

/// Check if mouse position is on the scrollbar.
///
/// # Arguments
/// * `col` - Column position of mouse
/// * `pum_left_col` - Left column of popup
/// * `pum_right_col` - Right column of popup
/// * `has_scrollbar` - Whether popup has scrollbar
/// * `is_rl` - Whether right-to-left mode
///
/// Returns 1 if on scrollbar, 0 otherwise.
#[no_mangle]
pub const extern "C" fn rs_pum_mouse_on_scrollbar(
    col: c_int,
    pum_left_col: c_int,
    pum_right_col: c_int,
    has_scrollbar: c_int,
    is_rl: c_int,
) -> c_int {
    if has_scrollbar == 0 {
        return 0;
    }

    if is_rl != 0 {
        (col == pum_left_col) as c_int
    } else {
        (col == pum_right_col - 1) as c_int
    }
}

/// Calculate scroll amount from mouse wheel event.
///
/// # Arguments
/// * `lines` - Number of lines to scroll (negative = up, positive = down)
/// * `first` - Current first visible item
/// * `height` - Number of visible items
/// * `size` - Total number of items
///
/// Returns new first visible item index.
#[no_mangle]
pub const extern "C" fn rs_pum_mouse_scroll(
    lines: c_int,
    first: c_int,
    height: c_int,
    size: c_int,
) -> c_int {
    let new_first = first + lines;

    if new_first < 0 {
        0
    } else if new_first > size - height {
        if size > height {
            size - height
        } else {
            0
        }
    } else {
        new_first
    }
}

/// Calculate item index from scrollbar click position.
///
/// # Arguments
/// * `click_row` - Row position of click (relative to popup)
/// * `height` - Height of popup
/// * `size` - Total number of items
///
/// Returns the item index that should become the first visible item.
#[no_mangle]
pub const extern "C" fn rs_pum_scrollbar_click_to_first(
    click_row: c_int,
    height: c_int,
    size: c_int,
) -> c_int {
    if height >= size || height <= 0 {
        return 0;
    }

    // Calculate proportional position
    let scroll_range = size - height;
    let first = (click_row * scroll_range + height / 2) / height;

    if first < 0 {
        0
    } else if first > scroll_range {
        scroll_range
    } else {
        first
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_scroll_down() {
        assert_eq!(rs_pum_mouse_scroll(3, 0, 10, 20), 3);
        assert_eq!(rs_pum_mouse_scroll(3, 7, 10, 20), 10);
    }

    #[test]
    fn test_mouse_scroll_up() {
        assert_eq!(rs_pum_mouse_scroll(-3, 5, 10, 20), 2);
        assert_eq!(rs_pum_mouse_scroll(-3, 1, 10, 20), 0);
    }

    #[test]
    fn test_mouse_scroll_bounds() {
        assert_eq!(rs_pum_mouse_scroll(100, 0, 10, 20), 10);
        assert_eq!(rs_pum_mouse_scroll(-100, 15, 10, 20), 0);
    }

    #[test]
    fn test_scrollbar_click() {
        // Click at top of scrollbar
        assert_eq!(rs_pum_scrollbar_click_to_first(0, 10, 20), 0);
        // Click at middle
        let first = rs_pum_scrollbar_click_to_first(5, 10, 20);
        assert!((0..=10).contains(&first));
        // Click at bottom
        assert_eq!(rs_pum_scrollbar_click_to_first(9, 10, 20), 9);
    }

    #[test]
    fn test_mouse_on_scrollbar_ltr() {
        assert_eq!(rs_pum_mouse_on_scrollbar(19, 0, 20, 1, 0), 1);
        assert_eq!(rs_pum_mouse_on_scrollbar(10, 0, 20, 1, 0), 0);
        assert_eq!(rs_pum_mouse_on_scrollbar(19, 0, 20, 0, 0), 0);
    }

    #[test]
    fn test_mouse_on_scrollbar_rtl() {
        assert_eq!(rs_pum_mouse_on_scrollbar(0, 0, 20, 1, 1), 1);
        assert_eq!(rs_pum_mouse_on_scrollbar(10, 0, 20, 1, 1), 0);
    }

    #[test]
    fn test_mouse_placement_below() {
        let result = rs_pum_calc_mouse_placement(5, 10, 5, 5, 20, 10, 0, 0, 30, 80, 0);
        assert_eq!(result.row, 6); // mouse_row + 1
        assert_eq!(result.above, 0);
    }

    #[test]
    fn test_mouse_placement_above() {
        let result = rs_pum_calc_mouse_placement(25, 10, 10, 10, 20, 10, 0, 0, 30, 80, 0);
        assert_eq!(result.above, 1);
    }
}
