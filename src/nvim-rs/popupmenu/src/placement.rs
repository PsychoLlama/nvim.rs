//! Popup menu placement calculations.
//!
//! This module handles computing the position and size of the popup menu
//! based on cursor position, screen boundaries, and available space.

use std::ffi::c_int;

/// Default popup menu height.
pub const PUM_DEF_HEIGHT: c_int = 10;

// FFI declarations for placement calculations
extern "C" {
    /// Get the `pumheight` option value.
    fn nvim_get_p_ph() -> i64;
    /// Get the `pumwidth` option value.
    fn nvim_get_p_pw() -> i64;
    /// Get the `pummaxwidth` option value.
    fn nvim_get_p_pmw() -> i64;
}

/// Result of vertical placement calculation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumVerticalResult {
    /// Row position for popup menu.
    pub row: c_int,
    /// Height of popup menu.
    pub height: c_int,
    /// Whether popup is above cursor (1 = above, 0 = below).
    pub above: c_int,
}

/// Result of horizontal placement calculation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumHorizontalResult {
    /// Column position for popup menu.
    pub col: c_int,
    /// Width of popup menu.
    pub width: c_int,
}

/// Result of size computation (width calculations from items).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumSizeResult {
    /// Width of text/abbr column.
    pub base_width: c_int,
    /// Width of kind column.
    pub kind_width: c_int,
    /// Width of extra/menu column.
    pub extra_width: c_int,
}

/// Try to set width so that it fits within `available_width`.
///
/// Returns (width, fits) where `fits` is true if width fits within available.
#[allow(clippy::cast_possible_truncation, clippy::similar_names)]
fn compute_width_aligned_with_cursor(
    width: c_int,
    available_width: c_int,
    pumwidth: i64,
    pummaxwidth: i64,
) -> (c_int, bool) {
    let (w, end_padding) = if (width as i64) < pumwidth {
        (pumwidth as c_int, false)
    } else if pummaxwidth > 0 && (width as i64) > pummaxwidth {
        (pummaxwidth as c_int, false)
    } else {
        (width, true)
    };

    let pum_width = w + c_int::from(end_padding && (w as i64) >= pumwidth);
    (pum_width, available_width >= pum_width)
}

/// Compute horizontal placement for the popup menu.
///
/// # Arguments
/// * `cursor_col` - Column position of cursor
/// * `max_col` - Maximum column (screen width or window boundary)
/// * `pum_rl` - Whether right-to-left mode is active (non-zero = true)
/// * `pum_scrollbar` - Scrollbar width (0 or 1)
/// * `pum_base_width` - Base width of items
/// * `pum_kind_width` - Width of kind column
/// * `pum_extra_width` - Width of extra column
///
/// # Safety
/// Calls C accessor functions for option values.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_pum_compute_horizontal(
    cursor_col: c_int,
    max_col: c_int,
    pum_rl: c_int,
    pum_scrollbar: c_int,
    pum_base_width: c_int,
    pum_kind_width: c_int,
    pum_extra_width: c_int,
) -> PumHorizontalResult {
    let pumwidth = nvim_get_p_pw();
    let pummaxwidth = nvim_get_p_pmw();
    let is_rl = pum_rl != 0;
    let desired_width = pum_base_width + pum_kind_width + pum_extra_width;

    let available_width = if is_rl {
        cursor_col - pum_scrollbar + 1
    } else {
        max_col - cursor_col - pum_scrollbar
    };

    // Try to align with cursor
    let (width, fits) =
        compute_width_aligned_with_cursor(desired_width, available_width, pumwidth, pummaxwidth);
    if fits {
        return PumHorizontalResult {
            col: cursor_col,
            width,
        };
    }

    // Show truncated if at least as wide as 'pumwidth'
    if (available_width as i64) > pumwidth {
        return PumHorizontalResult {
            col: cursor_col,
            width: available_width,
        };
    }

    // Truncated pum no longer aligned with cursor
    let available_width2 = if is_rl {
        max_col - pum_scrollbar
    } else {
        available_width + cursor_col
    };

    if (available_width2 as i64) > pumwidth {
        let width = pumwidth as c_int + 1;
        let col = if is_rl {
            width + pum_scrollbar
        } else {
            max_col - width - pum_scrollbar
        };
        return PumHorizontalResult { col, width };
    }

    // Not enough room anywhere
    let col = if is_rl { max_col - 1 } else { 0 };
    PumHorizontalResult {
        col,
        width: max_col - pum_scrollbar,
    }
}

/// Compute vertical placement for the popup menu.
///
/// # Arguments
/// * `size` - Number of items in popup menu
/// * `pum_win_row` - Row position of window cursor
/// * `above_row` - Row of preview window above (0 if none)
/// * `below_row` - Row of area below cursor
/// * `pum_border_size` - Border size for popup
/// * `cmdline_row` - Row where cmdline starts
/// * `is_cmdline` - Whether in cmdline mode (non-zero = true)
/// * `has_target_win` - Whether there's a target window (non-zero = true)
/// * `context_above` - Context lines available above (`w_wrow` - `w_cline_row`)
/// * `context_below` - Context lines available below (cline visible offset)
///
/// # Safety
/// Calls C accessor functions for option values.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_pum_compute_vertical(
    size: c_int,
    pum_win_row: c_int,
    above_row: c_int,
    below_row: c_int,
    pum_border_size: c_int,
    cmdline_row: c_int,
    is_cmdline: c_int,
    has_target_win: c_int,
    context_above: c_int,
    context_below: c_int,
) -> PumVerticalResult {
    let p_ph = nvim_get_p_ph();
    let is_cmdline = is_cmdline != 0;
    let has_target_win = has_target_win != 0;

    // Figure out the size and position of the pum
    let mut pum_height = size.min(PUM_DEF_HEIGHT);
    if p_ph > 0 && (pum_height as i64) > p_ph {
        pum_height = p_ph as c_int;
    }

    let mut pum_row;
    let pum_above;

    // Put the pum below "pum_win_row" if possible.
    // If there are few lines decide on where there is more room.
    if pum_win_row + 2 + pum_border_size >= below_row - pum_height
        && pum_win_row - above_row > (below_row - above_row) / 2
    {
        // pum above "pum_win_row"
        pum_above = true;

        let context_lines = if is_cmdline && !has_target_win {
            0
        } else {
            context_above.min(2)
        };

        if pum_win_row >= size + context_lines {
            pum_row = pum_win_row - size - context_lines;
            pum_height = size;
        } else {
            pum_row = 0;
            pum_height = pum_win_row - context_lines;
        }

        if p_ph > 0 && (pum_height as i64) > p_ph {
            pum_row += pum_height - p_ph as c_int;
            pum_height = p_ph as c_int;
        }

        if pum_border_size > 0 && pum_border_size + pum_row + pum_height >= pum_win_row {
            if pum_row < 2 {
                pum_height -= pum_border_size;
            } else {
                pum_row -= pum_border_size;
            }
        }
    } else {
        // pum below "pum_win_row"
        pum_above = false;

        let context_lines = if is_cmdline && !has_target_win {
            0
        } else {
            context_below.min(3)
        };

        pum_row = pum_win_row + context_lines;
        pum_height = (below_row - pum_row).min(size);

        if p_ph > 0 && (pum_height as i64) > p_ph {
            pum_height = p_ph as c_int;
        }

        if pum_row + pum_height + pum_border_size >= cmdline_row {
            pum_height -= pum_border_size;
        }
    }

    // If there is a preview window above avoid drawing over it.
    if above_row > 0 && pum_row < above_row && pum_height > above_row {
        pum_row = above_row;
        pum_height = pum_win_row - above_row;
    }

    PumVerticalResult {
        row: pum_row,
        height: pum_height,
        above: c_int::from(pum_above),
    }
}

/// Compute whether a scrollbar is needed for the popup menu.
///
/// Returns 1 if scrollbar needed, 0 otherwise.
#[no_mangle]
pub const extern "C" fn rs_pum_needs_scrollbar(pum_height: c_int, pum_size: c_int) -> c_int {
    (pum_height < pum_size) as c_int
}

/// Adjust popup menu row/height for screen boundaries.
///
/// # Arguments
/// * `row` - Current row position
/// * `height` - Current height
/// * `min_row` - Minimum allowed row
/// * `max_row` - Maximum allowed row
/// * `size` - Total number of items
/// * `above` - Whether popup is above cursor (non-zero = above)
///
/// Returns adjusted (row, height, above).
#[no_mangle]
pub const extern "C" fn rs_pum_adjust_bounds(
    row: c_int,
    height: c_int,
    min_row: c_int,
    max_row: c_int,
    size: c_int,
    above: c_int,
) -> PumVerticalResult {
    let mut pum_row = row;
    let mut pum_height = height;

    if above != 0 {
        // Above cursor - ensure we don't go above min_row
        if pum_row < min_row {
            pum_height += pum_row - min_row;
            pum_row = min_row;
        }
    } else {
        // Below cursor - ensure we don't go below max_row
        if pum_row + pum_height > max_row {
            pum_height = max_row - pum_row;
        }
    }

    // Ensure height doesn't exceed size
    if pum_height > size {
        pum_height = size;
    }

    PumVerticalResult {
        row: pum_row,
        height: pum_height,
        above,
    }
}

/// Compute the maximum column for popup menu placement.
///
/// # Arguments
/// * `screen_cols` - Total screen columns
/// * `win_col` - Window column offset
/// * `win_view_width` - Window view width
///
/// Returns the maximum column to use for placement.
#[no_mangle]
pub const extern "C" fn rs_pum_max_col(
    screen_cols: c_int,
    win_col: c_int,
    win_view_width: c_int,
) -> c_int {
    let win_end = win_col + win_view_width;
    if win_end > screen_cols {
        win_end
    } else {
        screen_cols
    }
}

/// Result of grid offset calculation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumGridOffset {
    /// Total grid width.
    pub grid_width: c_int,
    /// Column offset for content.
    pub col_offset: c_int,
    /// Whether extra space is added (0 or 1).
    pub extra_space: c_int,
}

/// Calculate grid column offset for popup menu display.
///
/// # Arguments
/// * `pum_width` - Width of popup menu
/// * `pum_col` - Column position
/// * `pum_scrollbar` - Whether scrollbar present (non-zero = yes)
/// * `is_rl` - Whether right-to-left mode (non-zero = yes)
/// * `has_border` - Whether border is present (non-zero = yes)
///
/// Returns grid width, column offset, and extra space flag.
#[no_mangle]
pub const extern "C" fn rs_pum_grid_offset(
    pum_width: c_int,
    pum_col: c_int,
    pum_scrollbar: c_int,
    is_rl: c_int,
    has_border: c_int,
) -> PumGridOffset {
    let is_rl = is_rl != 0;
    let has_scrollbar = pum_scrollbar != 0 && has_border == 0;

    if is_rl {
        // Right-to-left: col_off is at right side
        let col_off = pum_width - 1;
        // Check if there's room for extra space on right
        let extra_space = 1; // Assume yes for RTL
        let mut grid_width = pum_width;
        if extra_space != 0 {
            grid_width += 1;
        }
        if has_scrollbar {
            grid_width += 1;
        }
        PumGridOffset {
            grid_width,
            col_offset: col_off + has_scrollbar as c_int,
            extra_space,
        }
    } else {
        // Left-to-right: col_off depends on position
        let extra_space = (pum_col > 0) as c_int;
        let col_off = extra_space;
        let mut grid_width = pum_width;
        if extra_space != 0 {
            grid_width += 1;
        }
        if has_scrollbar {
            grid_width += 1;
        }
        PumGridOffset {
            grid_width,
            col_offset: col_off,
            extra_space,
        }
    }
}

/// Compute left and right columns for the popup grid.
///
/// # Arguments
/// * `pum_col` - Column position
/// * `col_offset` - Column offset within grid
/// * `grid_width` - Total grid width
///
/// Returns `(left_col, right_col)`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumGridBounds {
    /// Left column of pum grid.
    pub left_col: c_int,
    /// Right column of pum grid.
    pub right_col: c_int,
}

#[no_mangle]
pub const extern "C" fn rs_pum_grid_bounds(
    pum_col: c_int,
    col_offset: c_int,
    grid_width: c_int,
) -> PumGridBounds {
    let left_col = pum_col - col_offset;
    let right_col = left_col + grid_width;
    PumGridBounds {
        left_col,
        right_col,
    }
}

/// Adjust column for border width overflow.
///
/// If `col + border_width + width > max_cols`, adjusts col.
///
/// Returns adjusted column.
#[no_mangle]
pub const extern "C" fn rs_pum_adjust_col_for_border(
    col: c_int,
    border_width: c_int,
    width: c_int,
    max_cols: c_int,
) -> c_int {
    if col + border_width + width > max_cols {
        col - border_width
    } else {
        col
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_scrollbar() {
        assert_eq!(rs_pum_needs_scrollbar(10, 20), 1);
        assert_eq!(rs_pum_needs_scrollbar(20, 20), 0);
        assert_eq!(rs_pum_needs_scrollbar(20, 10), 0);
    }

    #[test]
    fn test_adjust_bounds_above() {
        let result = rs_pum_adjust_bounds(5, 10, 0, 30, 15, 1);
        assert_eq!(result.row, 5);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_adjust_bounds_below_overflow() {
        let result = rs_pum_adjust_bounds(25, 10, 0, 30, 15, 0);
        assert_eq!(result.row, 25);
        assert_eq!(result.height, 5); // 30 - 25 = 5
    }

    #[test]
    fn test_max_col_screen() {
        assert_eq!(rs_pum_max_col(80, 0, 40), 80);
    }

    #[test]
    fn test_max_col_window() {
        assert_eq!(rs_pum_max_col(80, 50, 40), 90);
    }

    #[test]
    fn test_grid_offset_ltr() {
        let result = rs_pum_grid_offset(20, 5, 1, 0, 0);
        assert!(result.grid_width >= 20);
        assert_eq!(result.extra_space, 1); // col > 0, so extra space
        assert_eq!(result.col_offset, 1);
    }

    #[test]
    fn test_grid_offset_rtl() {
        let result = rs_pum_grid_offset(20, 5, 0, 1, 0);
        assert!(result.grid_width >= 20);
        assert_eq!(result.extra_space, 1);
        assert_eq!(result.col_offset, 19); // pum_width - 1
    }

    #[test]
    fn test_grid_bounds() {
        let result = rs_pum_grid_bounds(10, 1, 22);
        assert_eq!(result.left_col, 9);
        assert_eq!(result.right_col, 31);
    }

    #[test]
    fn test_adjust_col_for_border_no_change() {
        assert_eq!(rs_pum_adjust_col_for_border(10, 2, 20, 80), 10);
    }

    #[test]
    fn test_adjust_col_for_border_adjust() {
        assert_eq!(rs_pum_adjust_col_for_border(60, 2, 20, 80), 58);
    }
}
