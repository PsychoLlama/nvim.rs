//! Popup menu state queries for Neovim
//!
//! This crate provides Rust implementations of popup menu functions
//! from `src/nvim/popupmenu.c`.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::cast_lossless)] // FFI needs flexible casts between c_int and i64

use std::ffi::c_int;

/// Default popup menu height.
const PUM_DEF_HEIGHT: c_int = 10;

// C accessor functions for popup menu state.
#[allow(dead_code)]
extern "C" {
    /// Get the `pum_is_visible` static variable.
    fn nvim_get_pum_is_visible() -> c_int;
    /// Get the `pum_external` static variable.
    fn nvim_get_pum_external() -> c_int;
    /// Get the `pum_height` static variable.
    fn nvim_get_pum_height() -> c_int;
    /// Set the `pum_height` static variable.
    fn nvim_set_pum_height(val: c_int);
    /// Get the UI popup menu height (iterates over UIs).
    fn ui_pum_get_height() -> c_int;
    /// Get the `pum_size` static variable (number of items).
    fn nvim_get_pum_size() -> c_int;
    /// Get the `pum_selected` static variable (selected index or -1).
    fn nvim_get_pum_selected() -> c_int;
    /// Set the `pum_selected` static variable.
    fn nvim_set_pum_selected(val: c_int);
    /// Get the `pum_first` static variable (index of top item).
    fn nvim_get_pum_first() -> c_int;
    /// Set the `pum_first` static variable.
    fn nvim_set_pum_first(val: c_int);
    /// Get the `pum_width` static variable.
    fn nvim_get_pum_width() -> c_int;
    /// Set the `pum_width` static variable.
    fn nvim_set_pum_width(val: c_int);
    /// Get the `pum_row` static variable.
    fn nvim_get_pum_row() -> c_int;
    /// Set the `pum_row` static variable.
    fn nvim_set_pum_row(val: c_int);
    /// Get the `pum_col` static variable.
    fn nvim_get_pum_col() -> c_int;
    /// Set the `pum_col` static variable.
    fn nvim_set_pum_col(val: c_int);
    /// Get the `pum_scrollbar` static variable.
    fn nvim_get_pum_scrollbar() -> c_int;
    /// Set the `pum_scrollbar` static variable.
    fn nvim_set_pum_scrollbar(val: c_int);
    /// Get the `pum_base_width` static variable.
    fn nvim_get_pum_base_width() -> c_int;
    /// Set the `pum_base_width` static variable.
    fn nvim_set_pum_base_width(val: c_int);
    /// Get the `pum_kind_width` static variable.
    fn nvim_get_pum_kind_width() -> c_int;
    /// Set the `pum_kind_width` static variable.
    fn nvim_set_pum_kind_width(val: c_int);
    /// Get the `pum_extra_width` static variable.
    fn nvim_get_pum_extra_width() -> c_int;
    /// Set the `pum_extra_width` static variable.
    fn nvim_set_pum_extra_width(val: c_int);
    /// Get the `pum_above` static variable.
    fn nvim_get_pum_above() -> c_int;
    /// Set the `pum_above` static variable.
    fn nvim_set_pum_above(val: c_int);
    /// Get the `pum_rl` static variable (right-to-left).
    fn nvim_get_pum_rl() -> c_int;
    /// Set the `pum_rl` static variable.
    fn nvim_set_pum_rl(val: c_int);

    // Global accessors
    /// Get the global Columns value.
    fn nvim_get_Columns() -> c_int;
    /// Get the global `cmdline_row` value.
    fn nvim_get_cmdline_row() -> c_int;
    /// Get the 'pumheight' option value.
    fn nvim_get_p_ph() -> i64;
    /// Get the 'pumwidth' option value.
    fn nvim_get_p_pw() -> i64;
    /// Get the 'pummaxwidth' option value.
    fn nvim_get_p_pmw() -> i64;
}

/// Check if the popup menu is displayed.
///
/// # Safety
/// Calls C accessor function for `pum_is_visible`.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_visible() -> c_int {
    nvim_get_pum_is_visible()
}

/// Check if the popup menu is displayed and drawn on the grid.
///
/// Returns true if visible and not external.
///
/// # Safety
/// Calls C accessor functions for `pum_is_visible` and `pum_external`.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_drawn() -> c_int {
    c_int::from(nvim_get_pum_is_visible() != 0 && nvim_get_pum_external() == 0)
}

/// Gets the height of the popup menu.
///
/// Returns the number of entries visible in the popup menu.
/// If the popup is external and a UI provides a height, returns that instead.
///
/// # Safety
/// Calls C accessor functions and `ui_pum_get_height`.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_get_height() -> c_int {
    if nvim_get_pum_external() != 0 {
        let ui_height = ui_pum_get_height();
        if ui_height != 0 {
            return ui_height;
        }
    }
    nvim_get_pum_height()
}

/// Result of vertical placement calculation.
#[repr(C)]
pub struct PumVerticalResult {
    /// Row position for popup menu.
    pub row: c_int,
    /// Height of popup menu.
    pub height: c_int,
    /// Whether popup is above cursor.
    pub above: c_int,
}

/// Result of horizontal placement calculation.
#[repr(C)]
pub struct PumHorizontalResult {
    /// Column position for popup menu.
    pub col: c_int,
    /// Width of popup menu.
    pub width: c_int,
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
/// * `pum_rl` - Whether right-to-left mode is active
/// * `pum_scrollbar` - Scrollbar width (0 or 1)
/// * `pum_base_width` - Base width of items
/// * `pum_kind_width` - Width of kind column
/// * `pum_extra_width` - Width of extra column
///
/// # Safety
/// Calls C accessor functions for option values.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_pum_compute_horizontal_placement(
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
/// * `is_cmdline` - Whether in cmdline mode
/// * `has_target_win` - Whether there's a target window
/// * `context_above` - Context lines available above (`w_wrow` - `w_cline_row`)
/// * `context_below` - Context lines available below (cline visible offset)
///
/// # Safety
/// Calls C accessor functions for option values.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_pum_compute_vertical_placement(
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
pub const extern "C" fn rs_pum_compute_scrollbar(pum_height: c_int, pum_size: c_int) -> c_int {
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
/// * `above` - Whether popup is above cursor
///
/// Returns adjusted (row, height).
#[no_mangle]
pub const extern "C" fn rs_pum_adjust_for_screen(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_scrollbar() {
        assert_eq!(rs_pum_compute_scrollbar(10, 20), 1);
        assert_eq!(rs_pum_compute_scrollbar(20, 20), 0);
        assert_eq!(rs_pum_compute_scrollbar(20, 10), 0);
    }

    #[test]
    fn test_adjust_for_screen_above() {
        const RESULT: PumVerticalResult = rs_pum_adjust_for_screen(5, 10, 0, 30, 15, 1);
        assert_eq!(RESULT.row, 5);
        assert_eq!(RESULT.height, 10);
    }

    #[test]
    fn test_adjust_for_screen_below_overflow() {
        const RESULT: PumVerticalResult = rs_pum_adjust_for_screen(25, 10, 0, 30, 15, 0);
        assert_eq!(RESULT.row, 25);
        assert_eq!(RESULT.height, 5); // 30 - 25 = 5
    }
}
