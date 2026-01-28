//! Popup menu (completion menu) implementation for Neovim.
//!
//! This crate provides Rust implementations of popup menu functions
//! from `src/nvim/popupmenu.c`. The popup menu is used for completion
//! suggestions, context menus, and other popup UI elements.
//!
//! # Architecture
//!
//! The popup menu consists of several components:
//!
//! - **State management**: Visibility, selection, scroll position
//! - **Layout calculation**: Positioning and sizing relative to cursor
//! - **Rendering**: Drawing items with highlight attributes and scrollbar
//! - **Event handling**: Mouse interaction and keyboard navigation
//!
//! # Modules
//!
//! - [`item`]: Item access, column ordering, and highlight attribute handling
//! - [`placement`]: Position and size calculations for vertical/horizontal layout
//! - [`render`]: Text rendering with fuzzy match highlighting and attributes
//! - [`redraw`]: Core redraw logic helpers for the rendering loop
//! - [`display`]: Display orchestration (show/hide, external UI handling)
//! - [`mouse`]: Mouse position detection and scroll handling
//! - [`event`]: Event information for dictionary population
//! - [`context_menu`]: Context menu (right-click popup) and UI flush

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::cast_lossless)] // FFI needs flexible casts between c_int and i64

pub mod context_menu;
pub mod display;
pub mod event;
pub mod item;
pub mod mouse;
pub mod placement;
pub mod redraw;
pub mod render;

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

    // pum_want struct accessors
    /// Set the `pum_want.active` field.
    fn nvim_set_pum_want_active(val: c_int);
    /// Set the `pum_want.item` field.
    fn nvim_set_pum_want_item(val: c_int);
    /// Set the `pum_want.insert` field.
    fn nvim_set_pum_want_insert(val: c_int);
    /// Set the `pum_want.finish` field.
    fn nvim_set_pum_want_finish(val: c_int);

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

/// Result of selection scroll calculation.
#[repr(C)]
pub struct PumScrollResult {
    /// New value for `pum_first` (index of top visible item).
    pub first: c_int,
}

/// Compute new scroll position when selection changes.
///
/// This calculates the new `pum_first` value to ensure the selected item
/// is visible with appropriate context.
///
/// # Arguments
/// * `selected` - Index of newly selected item
/// * `current_first` - Current top visible item index
/// * `height` - Number of visible items
/// * `size` - Total number of items
///
/// Returns new value for `pum_first`.
#[no_mangle]
pub const extern "C" fn rs_pum_compute_scroll(
    selected: c_int,
    current_first: c_int,
    height: c_int,
    size: c_int,
) -> c_int {
    // If selected is out of range, don't change first
    if selected < 0 || selected >= size {
        return current_first;
    }

    let scroll_offset = selected - height;
    let mut first = current_first;

    if first > selected - 4 {
        // scroll down; when we did a jump it's probably a PageUp then
        // scroll a whole page
        if first > selected - 2 {
            first -= height - 2;
            if first < 0 {
                first = 0;
            } else if first > selected {
                first = selected;
            }
        } else {
            first = selected;
        }
    } else if first < scroll_offset + 5 {
        // scroll up; when we did a jump it's probably a PageDown then
        // scroll a whole page
        if first < scroll_offset + 3 {
            let new_first = first + height - 2;
            first = if new_first > scroll_offset + 1 {
                new_first
            } else {
                scroll_offset + 1
            };
        } else {
            first = scroll_offset + 1;
        }
    }

    // Give a few lines of context when possible.
    let context = if height / 2 < 3 { height / 2 } else { 3 };

    if height > 2 {
        if first > selected - context {
            // scroll down
            first = if selected - context > 0 {
                selected - context
            } else {
                0
            };
        } else if first < selected + context - height + 1 {
            // scroll up
            first = selected + context - height + 1;
        }
    }

    // adjust for the number of lines displayed
    let max_first = size - height;
    if first > max_first {
        first = max_first;
    }

    first
}

/// Bounds-check selection for page up operation.
///
/// # Arguments
/// * `current` - Current selected index
/// * `first` - Current first visible item
/// * `height` - Number of visible items
///
/// Returns new selected index.
#[no_mangle]
pub const extern "C" fn rs_pum_page_up(current: c_int, first: c_int, height: c_int) -> c_int {
    if current == first {
        // Already at top of visible area, go up one page
        let new_sel = current - height;
        if new_sel < 0 {
            0
        } else {
            new_sel
        }
    } else {
        // Go to top of visible area
        first
    }
}

/// Bounds-check selection for page down operation.
///
/// # Arguments
/// * `current` - Current selected index
/// * `first` - Current first visible item
/// * `height` - Number of visible items
/// * `size` - Total number of items
///
/// Returns new selected index.
#[no_mangle]
pub const extern "C" fn rs_pum_page_down(
    current: c_int,
    first: c_int,
    height: c_int,
    size: c_int,
) -> c_int {
    let last_visible = first + height - 1;
    if current == last_visible {
        // Already at bottom of visible area, go down one page
        let new_sel = current + height;
        if new_sel >= size {
            size - 1
        } else {
            new_sel
        }
    } else {
        // Go to bottom of visible area
        if last_visible >= size {
            size - 1
        } else {
            last_visible
        }
    }
}

/// Cycle selection with wrapping.
///
/// # Arguments
/// * `current` - Current selected index (-1 for none)
/// * `delta` - Amount to move (+1 for next, -1 for prev)
/// * `size` - Total number of items
/// * `wrap` - Whether to wrap around
///
/// Returns new selected index.
#[no_mangle]
pub const extern "C" fn rs_pum_cycle_selected(
    current: c_int,
    delta: c_int,
    size: c_int,
    wrap: c_int,
) -> c_int {
    if size <= 0 {
        return -1;
    }

    let wrap = wrap != 0;

    // Handle initial selection
    if current < 0 {
        return if delta > 0 { 0 } else { size - 1 };
    }

    let new_sel = current + delta;

    if wrap {
        if new_sel < 0 {
            size - 1
        } else if new_sel >= size {
            0
        } else {
            new_sel
        }
    } else if new_sel < 0 {
        0
    } else if new_sel >= size {
        size - 1
    } else {
        new_sel
    }
}

/// Bounds-check a selected index.
///
/// Returns the index clamped to valid range, or -1 if no items.
#[no_mangle]
pub const extern "C" fn rs_pum_bound_selection(selected: c_int, size: c_int) -> c_int {
    if size <= 0 {
        return -1;
    }
    if selected < 0 {
        return -1;
    }
    if selected >= size {
        return size - 1;
    }
    selected
}

/// Completion item type constants.
pub const CPT_ABBR: c_int = 0;
pub const CPT_KIND: c_int = 1;
pub const CPT_MENU: c_int = 2;

/// Result of column layout calculation.
#[repr(C)]
pub struct PumColumnLayout {
    /// Width allocated for the first column (abbr).
    pub first_width: c_int,
    /// Width allocated for the second column (kind).
    pub second_width: c_int,
    /// Width allocated for the third column (menu/extra).
    pub third_width: c_int,
    /// Total width used.
    pub total_width: c_int,
}

/// Result of align order parsing.
#[repr(C)]
pub struct PumAlignOrder {
    /// First column type.
    pub first: c_int,
    /// Second column type.
    pub second: c_int,
    /// Third column type.
    pub third: c_int,
}

/// Parse the completion item align flags into column order.
///
/// # Arguments
/// * `cia_flags` - The completion item align flags (0 for default)
///
/// Returns struct with [first, second, third] column types.
#[no_mangle]
pub const extern "C" fn rs_pum_get_align_order(cia_flags: c_int) -> PumAlignOrder {
    if cia_flags == 0 {
        // Default order: abbr, kind, menu
        PumAlignOrder {
            first: CPT_ABBR,
            second: CPT_KIND,
            third: CPT_MENU,
        }
    } else {
        // Parse flags: hundreds = first, tens = second, units = third
        PumAlignOrder {
            first: cia_flags / 100,
            second: (cia_flags / 10) % 10,
            third: cia_flags % 10,
        }
    }
}

/// Compute column widths for popup menu layout.
///
/// # Arguments
/// * `available_width` - Total available width
/// * `base_width` - Width of text/abbr column
/// * `kind_width` - Width of kind column
/// * `extra_width` - Width of extra/menu column
/// * `scrollbar` - Whether scrollbar is present
///
/// Returns column layout with widths.
#[no_mangle]
pub const extern "C" fn rs_pum_compute_column_widths(
    available_width: c_int,
    base_width: c_int,
    kind_width: c_int,
    extra_width: c_int,
    scrollbar: c_int,
) -> PumColumnLayout {
    let scrollbar_width = if scrollbar != 0 { 1 } else { 0 };
    let content_width = available_width - scrollbar_width;

    let total_needed = base_width + kind_width + extra_width;

    if total_needed <= content_width {
        // All columns fit
        PumColumnLayout {
            first_width: base_width,
            second_width: kind_width,
            third_width: extra_width,
            total_width: total_needed,
        }
    } else {
        // Need to truncate - prioritize base_width, then kind, then extra
        let mut remaining = content_width;

        let first = if base_width <= remaining {
            remaining -= base_width;
            base_width
        } else {
            let w = remaining;
            remaining = 0;
            w
        };

        let second = if kind_width <= remaining {
            remaining -= kind_width;
            kind_width
        } else {
            let w = remaining;
            remaining = 0;
            w
        };

        let third = if extra_width <= remaining {
            extra_width
        } else {
            remaining
        };

        PumColumnLayout {
            first_width: first,
            second_width: second,
            third_width: third,
            total_width: first + second + third,
        }
    }
}

/// Result of thumb calculation.
#[repr(C)]
pub struct PumThumbResult {
    /// Position of thumb (row index).
    pub pos: c_int,
    /// Height of thumb in rows.
    pub height: c_int,
}

/// Calculate thumb (scrollbar indicator) position and height.
///
/// # Arguments
/// * `pum_first` - Index of first visible item
/// * `pum_height` - Number of visible items
/// * `pum_size` - Total number of items
///
/// Returns thumb position and height.
#[no_mangle]
pub const extern "C" fn rs_pum_compute_thumb(
    pum_first: c_int,
    pum_height: c_int,
    pum_size: c_int,
) -> PumThumbResult {
    if pum_size <= pum_height {
        // No scrollbar needed
        return PumThumbResult {
            pos: 0,
            height: pum_height,
        };
    }

    let scroll_range = pum_size - pum_height;

    // Calculate thumb height (proportional to visible/total)
    let mut thumb_height = pum_height * pum_height / pum_size;
    if thumb_height == 0 {
        thumb_height = 1;
    }

    // Calculate thumb position
    let thumb_pos = (pum_first * (pum_height - thumb_height) + scroll_range / 2) / scroll_range;

    PumThumbResult {
        pos: thumb_pos,
        height: thumb_height,
    }
}

/// Clear the popup menu.
///
/// Currently only resets the offset to the first displayed item (`pum_first = 0`).
///
/// # Safety
/// Calls C accessor function to set `pum_first`.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_clear() {
    nvim_set_pum_first(0);
}

/// Select an item in the popup menu for external UI.
///
/// # Arguments
/// * `item` - Item index to select (-1 to deselect)
/// * `insert` - Whether to insert the selected item
/// * `finish` - Whether this is the final selection
///
/// # Safety
/// Calls C accessor functions for `pum_want` and `pum_size`.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_ext_select_item(item: c_int, insert: c_int, finish: c_int) {
    // Check if visible and item is in valid range
    if nvim_get_pum_is_visible() == 0 || item < -1 || item >= nvim_get_pum_size() {
        return;
    }
    nvim_set_pum_want_active(1);
    nvim_set_pum_want_item(item);
    nvim_set_pum_want_insert(insert);
    nvim_set_pum_want_finish(finish);
}

/// Check if a row is within the scrollbar thumb area.
///
/// # Arguments
/// * `row` - Row index to check
/// * `thumb_pos` - Start position of thumb
/// * `thumb_height` - Height of thumb
///
/// Returns 1 if row is in thumb, 0 otherwise.
#[no_mangle]
pub const extern "C" fn rs_pum_is_thumb_row(
    row: c_int,
    thumb_pos: c_int,
    thumb_height: c_int,
) -> c_int {
    (row >= thumb_pos && row < thumb_pos + thumb_height) as c_int
}

/// Compute the width needed for text truncation indicator.
///
/// Returns 1 if truncation indicator takes one column.
#[no_mangle]
pub const extern "C" fn rs_pum_truncation_width() -> c_int {
    1 // The truncation indicator (< or >) takes 1 column
}

/// Result of grid offset calculation.
#[repr(C)]
pub struct PumGridOffset {
    /// Total grid width.
    pub grid_width: c_int,
    /// Column offset for content.
    pub col_offset: c_int,
    /// Whether extra space is added (0 or 1).
    pub extra_space: c_int,
}

/// Calculate grid column offset for right-to-left display.
///
/// # Arguments
/// * `pum_width` - Width of popup menu
/// * `pum_col` - Column position
/// * `pum_scrollbar` - Whether scrollbar present
/// * `is_rl` - Whether right-to-left mode
///
/// Returns grid width, column offset, and extra space flag.
#[no_mangle]
pub const extern "C" fn rs_pum_compute_grid_offset(
    pum_width: c_int,
    pum_col: c_int,
    pum_scrollbar: c_int,
    is_rl: c_int,
) -> PumGridOffset {
    let is_rl = is_rl != 0;
    let has_scrollbar = pum_scrollbar != 0;

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

    #[test]
    fn test_cycle_selected_forward() {
        assert_eq!(rs_pum_cycle_selected(0, 1, 5, 0), 1);
        assert_eq!(rs_pum_cycle_selected(4, 1, 5, 0), 4); // No wrap, stay at end
        assert_eq!(rs_pum_cycle_selected(4, 1, 5, 1), 0); // Wrap to start
    }

    #[test]
    fn test_cycle_selected_backward() {
        assert_eq!(rs_pum_cycle_selected(1, -1, 5, 0), 0);
        assert_eq!(rs_pum_cycle_selected(0, -1, 5, 0), 0); // No wrap, stay at start
        assert_eq!(rs_pum_cycle_selected(0, -1, 5, 1), 4); // Wrap to end
    }

    #[test]
    fn test_cycle_selected_initial() {
        assert_eq!(rs_pum_cycle_selected(-1, 1, 5, 0), 0); // Start from beginning
        assert_eq!(rs_pum_cycle_selected(-1, -1, 5, 0), 4); // Start from end
    }

    #[test]
    fn test_page_up() {
        assert_eq!(rs_pum_page_up(5, 5, 10), 0); // At top of visible, page up
        assert_eq!(rs_pum_page_up(8, 5, 10), 5); // Not at top, go to top
    }

    #[test]
    fn test_page_down() {
        assert_eq!(rs_pum_page_down(14, 5, 10, 20), 19); // At bottom, page down limited by size
        assert_eq!(rs_pum_page_down(8, 5, 10, 20), 14); // Not at bottom, go to bottom
    }

    #[test]
    fn test_bound_selection() {
        assert_eq!(rs_pum_bound_selection(5, 10), 5);
        assert_eq!(rs_pum_bound_selection(15, 10), 9);
        assert_eq!(rs_pum_bound_selection(-1, 10), -1);
        assert_eq!(rs_pum_bound_selection(5, 0), -1);
    }

    #[test]
    fn test_compute_scroll_basic() {
        // Selected item already visible, no change needed
        let result = rs_pum_compute_scroll(5, 0, 10, 20);
        assert!(result >= 0);
    }

    #[test]
    fn test_get_align_order_default() {
        let order = rs_pum_get_align_order(0);
        assert_eq!(order.first, CPT_ABBR);
        assert_eq!(order.second, CPT_KIND);
        assert_eq!(order.third, CPT_MENU);
    }

    #[test]
    fn test_get_align_order_custom() {
        // flags = 210 means abbr=2, kind=1, menu=0
        let order = rs_pum_get_align_order(210);
        assert_eq!(order.first, 2);
        assert_eq!(order.second, 1);
        assert_eq!(order.third, 0);
    }

    #[test]
    fn test_compute_column_widths_all_fit() {
        let layout = rs_pum_compute_column_widths(50, 20, 10, 15, 0);
        assert_eq!(layout.first_width, 20);
        assert_eq!(layout.second_width, 10);
        assert_eq!(layout.third_width, 15);
        assert_eq!(layout.total_width, 45);
    }

    #[test]
    fn test_compute_column_widths_truncate() {
        let layout = rs_pum_compute_column_widths(30, 20, 10, 15, 0);
        assert_eq!(layout.first_width, 20);
        assert_eq!(layout.second_width, 10);
        assert_eq!(layout.third_width, 0);
    }

    #[test]
    fn test_compute_thumb() {
        let result = rs_pum_compute_thumb(0, 10, 20);
        assert!(result.height >= 1);
        assert!(result.pos >= 0);
        assert!(result.pos + result.height <= 10);
    }

    #[test]
    fn test_is_thumb_row() {
        assert_eq!(rs_pum_is_thumb_row(5, 4, 3), 1); // 5 is in [4, 7)
        assert_eq!(rs_pum_is_thumb_row(3, 4, 3), 0); // 3 is not in [4, 7)
        assert_eq!(rs_pum_is_thumb_row(7, 4, 3), 0); // 7 is not in [4, 7)
    }

    #[test]
    fn test_compute_grid_offset_ltr() {
        let result = rs_pum_compute_grid_offset(20, 5, 1, 0);
        assert!(result.grid_width >= 20);
        assert_eq!(result.extra_space, 1); // col > 0, so extra space
        assert_eq!(result.col_offset, 1);
    }

    #[test]
    fn test_compute_grid_offset_rtl() {
        let result = rs_pum_compute_grid_offset(20, 5, 0, 1);
        assert!(result.grid_width >= 20);
        assert_eq!(result.extra_space, 1);
        assert_eq!(result.col_offset, 19); // pum_width - 1
    }
}
