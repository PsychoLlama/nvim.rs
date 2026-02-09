//! Grid compositor for Neovim multi-grid UI
//!
//! This crate provides Rust implementations of compositor functions
//! from `src/nvim/ui_compositor.c`. The compositor merges floating grids
//! with the main grid for display in TUI and non-multigrid UIs.
//!
//! Layer-based compositing: <https://en.wikipedia.org/wiki/Digital_compositing>

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]

pub mod floating;
pub mod linebuf;
pub mod state;

use std::ffi::{c_int, c_void};

/// Opaque handle to C's ScreenGrid
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ScreenGridHandle(*mut c_void);

impl ScreenGridHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Type alias for screen character (matches C's `schar_T` which is `uint32_t`).
pub type ScharT = u32;

/// Type alias for screen attribute (matches C's `sattr_T` which is `int32_t`).
pub type SattrT = i32;

/// Z-index constants for grid layering
pub mod zindex {
    /// Default grid (main editor)
    pub const DEFAULT_GRID: i32 = 0;
    /// Default z-index for floating windows
    pub const FLOAT_DEFAULT: i32 = 50;
    /// Popup menu z-index
    pub const POPUP_MENU: i32 = 100;
    /// Messages z-index
    pub const MESSAGES: i32 = 200;
    /// Cmdline popup menu z-index
    pub const CMDLINE_POPUP_MENU: i32 = 250;
}

// =============================================================================
// C Accessor Functions
// =============================================================================

/// Handle type for grids (matches C's `handle_T` which is `int`).
pub type HandleT = c_int;

/// Opaque handle to C's win_T.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WinHandle(*mut c_void);

impl WinHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

extern "C" {
    // Compositor state accessors
    fn nvim_get_composed_uis() -> c_int;
    fn nvim_get_valid_screen() -> c_int;

    // Layer stack accessors
    fn nvim_layers_size() -> usize;
    fn nvim_layers_get(i: usize) -> ScreenGridHandle;

    // Message grid accessors
    fn nvim_get_msg_grid() -> ScreenGridHandle;
    fn nvim_get_msg_current_row() -> c_int;
    fn nvim_get_msg_was_scrolled() -> bool;

    // Current grid accessors
    fn nvim_get_curgrid() -> ScreenGridHandle;
    fn nvim_set_curgrid(grid: ScreenGridHandle);
    fn nvim_screengrid_get_comp_index(grid: ScreenGridHandle) -> usize;
    fn nvim_screengrid_get_handle(grid: ScreenGridHandle) -> HandleT;

    // Grid dimension accessors
    fn nvim_screengrid_get_comp_row(grid: ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_comp_col(grid: ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_rows(grid: ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_cols(grid: ScreenGridHandle) -> c_int;

    // Grid modification accessors
    fn nvim_layers_set(i: usize, grid: ScreenGridHandle);
    fn nvim_layers_pop();
    fn nvim_layers_push(grid: ScreenGridHandle);
    fn nvim_screengrid_set_comp_index(grid: ScreenGridHandle, val: usize);
    fn nvim_screengrid_set_pending_comp_index_update(grid: ScreenGridHandle, val: bool);
    fn nvim_screengrid_set_comp_row(grid: ScreenGridHandle, val: c_int);
    fn nvim_screengrid_set_comp_col(grid: ScreenGridHandle, val: c_int);
    fn nvim_screengrid_set_comp_width(grid: ScreenGridHandle, val: c_int);
    fn nvim_screengrid_set_comp_height(grid: ScreenGridHandle, val: c_int);
    fn nvim_screengrid_set_comp_disabled(grid: ScreenGridHandle, val: bool);
    fn nvim_screengrid_get_zindex(grid: ScreenGridHandle) -> c_int;

    // Default grid and curwin grid accessors
    fn nvim_get_default_grid() -> ScreenGridHandle;
    fn nvim_get_curwin_grid_alloc() -> ScreenGridHandle;

    // Composition function
    fn nvim_compose_area(startrow: c_int, endrow: c_int, startcol: c_int, endcol: c_int);

    // Cursor/grid functions
    fn nvim_curgrid_is_default() -> bool;
    fn nvim_ui_composed_call_grid_cursor_goto(grid_handle: c_int, row: c_int, col: c_int);

    // Screen validity
    fn nvim_set_valid_screen(valid: bool);
    fn nvim_set_msg_sep_row(row: c_int);

    // Window iteration accessors
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_grid_alloc(wp: WinHandle) -> ScreenGridHandle;
    fn nvim_win_get_config_hide(wp: WinHandle) -> c_int;
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;
    fn nvim_win_get_wincol(wp: WinHandle) -> c_int;

    // Grid mouse state
    fn nvim_screengrid_get_mouse_enabled(grid: ScreenGridHandle) -> bool;

    // UI extension check
    fn nvim_get_ui_ext(ext: c_int) -> c_int;
}

/// UI extension constants (from ui_defs.h)
pub mod ui_ext {
    /// kUIMultigrid = 6 in the C enum
    pub const MULTIGRID: i32 = 6;
}

// =============================================================================
// Compositor Functions
// =============================================================================

/// Check if compositor should draw (has composed UIs and valid screen).
///
/// Returns true if there are composed UIs attached and the screen is valid.
fn ui_comp_should_draw_impl() -> bool {
    unsafe { nvim_get_composed_uis() != 0 && nvim_get_valid_screen() != 0 }
}

/// Check if curgrid is covered on row or above.
///
/// This checks if there are layers above the current grid that would cover
/// the given row. Currently only handles the message row case.
///
/// Returns true if curgrid is covered at or above the given row.
fn curgrid_covered_above_impl(row: c_int) -> bool {
    unsafe {
        let layers_size = nvim_layers_size();
        if layers_size == 0 {
            return false;
        }

        let curgrid = nvim_get_curgrid();
        if curgrid.is_null() {
            return false;
        }

        let last_layer = nvim_layers_get(layers_size - 1);
        let msg_grid = nvim_get_msg_grid();

        // Check if we're above the message row
        let above_msg = last_layer.0 == msg_grid.0 && {
            let msg_current_row = nvim_get_msg_current_row();
            let msg_was_scrolled = nvim_get_msg_was_scrolled();
            row < msg_current_row - c_int::from(msg_was_scrolled)
        };

        let curgrid_index = nvim_screengrid_get_comp_index(curgrid);
        let effective_layers = layers_size - usize::from(above_msg);

        effective_layers > curgrid_index + 1
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if compositor should draw.
///
/// # Safety
/// This function accesses global compositor state.
#[no_mangle]
pub extern "C" fn rs_ui_comp_should_draw() -> c_int {
    c_int::from(ui_comp_should_draw_impl())
}

/// Check if curgrid is covered on row or above.
///
/// # Safety
/// This function accesses global compositor state (layers, curgrid, msg_grid).
#[no_mangle]
pub extern "C" fn rs_curgrid_covered_above(row: c_int) -> bool {
    curgrid_covered_above_impl(row)
}

/// Set the current grid for compositor operations.
///
/// Searches through the layer stack for a grid with the given handle
/// and sets it as the current grid.
///
/// Returns true if a grid with the handle was found and set.
fn ui_comp_set_grid_impl(handle: HandleT) -> bool {
    unsafe {
        let curgrid = nvim_get_curgrid();
        if !curgrid.is_null() && nvim_screengrid_get_handle(curgrid) == handle {
            return true;
        }

        let layers_size = nvim_layers_size();
        for i in 0..layers_size {
            let grid = nvim_layers_get(i);
            if !grid.is_null() && nvim_screengrid_get_handle(grid) == handle {
                nvim_set_curgrid(grid);
                return true;
            }
        }
        false
    }
}

/// FFI wrapper for `ui_comp_set_grid`.
///
/// Sets the current grid for compositor operations by handle.
///
/// # Safety
/// This function accesses global compositor state.
#[no_mangle]
pub extern "C" fn rs_ui_comp_set_grid(handle: HandleT) -> c_int {
    c_int::from(ui_comp_set_grid_impl(handle))
}

/// Compose a grid's area onto the screen.
///
/// This triggers composition of the entire grid area if compositor
/// drawing is enabled.
fn ui_comp_compose_grid_impl(grid: ScreenGridHandle) {
    if !ui_comp_should_draw_impl() {
        return;
    }
    unsafe {
        let comp_row = nvim_screengrid_get_comp_row(grid);
        let comp_col = nvim_screengrid_get_comp_col(grid);
        let rows = nvim_screengrid_get_rows(grid);
        let cols = nvim_screengrid_get_cols(grid);
        nvim_compose_area(comp_row, comp_row + rows, comp_col, comp_col + cols);
    }
}

/// FFI wrapper for `ui_comp_compose_grid`.
///
/// Composes the given grid's area onto the screen.
///
/// # Safety
/// This function accesses global compositor state and grid dimensions.
#[no_mangle]
pub extern "C" fn rs_ui_comp_compose_grid(grid: ScreenGridHandle) {
    ui_comp_compose_grid_impl(grid);
}

/// Raise a grid to a new position in the layer stack.
///
/// This moves a grid from its current position to a higher position in the
/// layer stack. Grids at higher positions are drawn on top of lower ones.
///
/// After moving the grid, composes the overlapping areas between the raised
/// grid and all grids that were moved down.
fn ui_comp_raise_grid_impl(grid: ScreenGridHandle, new_index: usize) {
    unsafe {
        let old_index = nvim_screengrid_get_comp_index(grid);

        // Shift layers down: move each layer at position i+1 to position i
        for i in old_index..new_index {
            let next_grid = nvim_layers_get(i + 1);
            nvim_layers_set(i, next_grid);
            nvim_screengrid_set_comp_index(next_grid, i);
            nvim_screengrid_set_pending_comp_index_update(next_grid, true);
        }

        // Place the grid at its new position
        nvim_layers_set(new_index, grid);
        nvim_screengrid_set_comp_index(grid, new_index);
        nvim_screengrid_set_pending_comp_index_update(grid, true);

        // Compose overlapping areas between the raised grid and grids that moved down
        let src_row = nvim_screengrid_get_comp_row(grid);
        let src_col = nvim_screengrid_get_comp_col(grid);
        let src_rows = nvim_screengrid_get_rows(grid);
        let src_cols = nvim_screengrid_get_cols(grid);

        for i in old_index..new_index {
            let other = nvim_layers_get(i);
            let other_row = nvim_screengrid_get_comp_row(other);
            let other_col = nvim_screengrid_get_comp_col(other);
            let other_rows = nvim_screengrid_get_rows(other);
            let other_cols = nvim_screengrid_get_cols(other);

            // Calculate overlapping area
            let startcol = src_col.max(other_col);
            let endcol = (src_col + src_cols).min(other_col + other_cols);
            let startrow = src_row.max(other_row);
            let endrow = (src_row + src_rows).min(other_row + other_rows);

            nvim_compose_area(startrow, endrow, startcol, endcol);
        }
    }
}

/// FFI wrapper for `ui_comp_raise_grid`.
///
/// Raises a grid to a new position in the layer stack.
///
/// # Safety
/// This function modifies global compositor state (layer ordering).
#[no_mangle]
pub extern "C" fn rs_ui_comp_raise_grid(grid: ScreenGridHandle, new_index: usize) {
    ui_comp_raise_grid_impl(grid, new_index);
}

/// Remove a grid from the layer stack.
///
/// This removes a grid from the compositor layers, shifts remaining layers
/// down to fill the gap, and recomposes the area that was covered.
fn ui_comp_remove_grid_impl(grid: ScreenGridHandle) {
    unsafe {
        let comp_index = nvim_screengrid_get_comp_index(grid);

        // Grid wasn't present (comp_index == 0 means not in layers)
        if comp_index == 0 {
            return;
        }

        // If curgrid == grid, reset to default grid
        let curgrid = nvim_get_curgrid();
        if curgrid.0 == grid.0 {
            let default_grid = nvim_get_default_grid();
            nvim_set_curgrid(default_grid);
        }

        // Shift layers down
        let layers_size = nvim_layers_size();
        for i in comp_index..(layers_size - 1) {
            let next_grid = nvim_layers_get(i + 1);
            nvim_layers_set(i, next_grid);
            nvim_screengrid_set_comp_index(next_grid, i);
            nvim_screengrid_set_pending_comp_index_update(next_grid, true);
        }

        // Pop the last element
        nvim_layers_pop();

        // Reset grid's comp_index
        nvim_screengrid_set_comp_index(grid, 0);
        nvim_screengrid_set_pending_comp_index_update(grid, true);

        // Recompose the area under the removed grid
        ui_comp_compose_grid_impl(grid);
    }
}

/// FFI wrapper for `ui_comp_remove_grid`.
///
/// Removes a grid from the compositor layer stack.
///
/// # Safety
/// This function modifies global compositor state.
#[no_mangle]
pub extern "C" fn rs_ui_comp_remove_grid(grid: ScreenGridHandle) {
    ui_comp_remove_grid_impl(grid);
}

/// Put a grid at a specific position in the compositor.
///
/// If the grid is already in the layer stack, it will be moved to the new position.
/// If the grid is new, it will be inserted at the appropriate z-index position.
///
/// Returns true if the grid position changed.
#[allow(clippy::too_many_arguments)]
fn ui_comp_put_grid_impl(
    grid: ScreenGridHandle,
    row: c_int,
    col: c_int,
    height: c_int,
    width: c_int,
    valid: bool,
    on_top: bool,
) -> bool {
    unsafe {
        nvim_screengrid_set_pending_comp_index_update(grid, true);
        nvim_screengrid_set_comp_height(grid, height);
        nvim_screengrid_set_comp_width(grid, width);

        let comp_index = nvim_screengrid_get_comp_index(grid);
        let moved: bool;

        if comp_index != 0 {
            // Grid is already in layers - check if it moved
            let old_row = nvim_screengrid_get_comp_row(grid);
            let old_col = nvim_screengrid_get_comp_col(grid);
            moved = row != old_row || col != old_col;

            if ui_comp_should_draw_impl() {
                // Redraw the area covered by the old position that is not covered
                // by the new position. Disable the grid so compose_area won't use it.
                let grid_rows = nvim_screengrid_get_rows(grid);
                let grid_cols = nvim_screengrid_get_cols(grid);

                nvim_screengrid_set_comp_disabled(grid, true);

                // Top area (above new position)
                nvim_compose_area(old_row, row, old_col, old_col + grid_cols);

                // Left area (between old and new, vertically overlapping)
                if old_col < col {
                    nvim_compose_area(
                        row.max(old_row),
                        (row + height).min(old_row + grid_rows),
                        old_col,
                        col,
                    );
                }

                // Right area (between old and new, vertically overlapping)
                if col + width < old_col + grid_cols {
                    nvim_compose_area(
                        row.max(old_row),
                        (row + height).min(old_row + grid_rows),
                        col + width,
                        old_col + grid_cols,
                    );
                }

                // Bottom area (below new position)
                nvim_compose_area(
                    row + height,
                    old_row + grid_rows,
                    old_col,
                    old_col + grid_cols,
                );

                nvim_screengrid_set_comp_disabled(grid, false);
            }

            nvim_screengrid_set_comp_row(grid, row);
            nvim_screengrid_set_comp_col(grid, col);
        } else {
            // New grid - find insertion point based on z-index
            moved = true;
            let layers_size = nvim_layers_size();
            let mut insert_at = layers_size;
            let grid_zindex = nvim_screengrid_get_zindex(grid);

            // Find the right position based on z-index
            while insert_at > 0 {
                let prev_grid = nvim_layers_get(insert_at - 1);
                let prev_zindex = nvim_screengrid_get_zindex(prev_grid);
                if prev_zindex <= grid_zindex {
                    break;
                }
                insert_at -= 1;
            }

            // Special case: if inserting after curwin's grid with same z-index and not on_top
            let curwin_grid = nvim_get_curwin_grid_alloc();
            if !curwin_grid.is_null() && insert_at > 0 {
                let prev_grid = nvim_layers_get(insert_at - 1);
                let prev_zindex = nvim_screengrid_get_zindex(prev_grid);
                if prev_grid.0 == curwin_grid.0 && prev_zindex == grid_zindex && !on_top {
                    insert_at -= 1;
                }
            }

            // Push a new slot and shift grids to make room
            nvim_layers_push(grid); // This just adds space
            let new_size = nvim_layers_size();
            for i in (insert_at + 1..new_size).rev() {
                let prev = nvim_layers_get(i - 1);
                nvim_layers_set(i, prev);
                nvim_screengrid_set_comp_index(prev, i);
                nvim_screengrid_set_pending_comp_index_update(prev, true);
            }
            nvim_layers_set(insert_at, grid);

            nvim_screengrid_set_comp_row(grid, row);
            nvim_screengrid_set_comp_col(grid, col);
            nvim_screengrid_set_comp_index(grid, insert_at);
            nvim_screengrid_set_pending_comp_index_update(grid, true);
        }

        // Compose the new grid area if it moved and is valid
        if moved && valid && ui_comp_should_draw_impl() {
            let grid_rows = nvim_screengrid_get_rows(grid);
            let grid_cols = nvim_screengrid_get_cols(grid);
            let comp_row = nvim_screengrid_get_comp_row(grid);
            let comp_col = nvim_screengrid_get_comp_col(grid);
            nvim_compose_area(
                comp_row,
                comp_row + grid_rows,
                comp_col,
                comp_col + grid_cols,
            );
        }

        moved
    }
}

/// FFI wrapper for `ui_comp_put_grid`.
///
/// Puts a grid at a specific position in the compositor.
///
/// # Safety
/// This function modifies global compositor state.
#[no_mangle]
pub extern "C" fn rs_ui_comp_put_grid(
    grid: ScreenGridHandle,
    row: c_int,
    col: c_int,
    height: c_int,
    width: c_int,
    valid: bool,
    on_top: bool,
) -> bool {
    ui_comp_put_grid_impl(grid, row, col, height, width, valid, on_top)
}

/// Handle cursor positioning for grid compositor.
///
/// This sets the current grid by handle, computes the absolute cursor position,
/// optionally raises the grid in the layer stack, and sends the cursor position
/// to composed UIs.
fn ui_comp_grid_cursor_goto_impl(grid_handle: HandleT, r: i64, c: i64) {
    unsafe {
        // Set the current grid; if not found, bail out
        if !ui_comp_set_grid_impl(grid_handle) {
            return;
        }

        let curgrid = nvim_get_curgrid();
        let cursor_row = nvim_screengrid_get_comp_row(curgrid) + r as c_int;
        let cursor_col = nvim_screengrid_get_comp_col(curgrid) + c as c_int;

        // Raise the grid in layer stack if it's not the default grid
        // This ensures the focused grid is drawn on top of others with same/lower z-index
        if !nvim_curgrid_is_default() {
            let layers_size = nvim_layers_size();
            let mut new_index = layers_size.saturating_sub(1);
            let curgrid_zindex = nvim_screengrid_get_zindex(curgrid);

            // Find the appropriate position based on z-index
            while new_index > 1 {
                let layer = nvim_layers_get(new_index);
                if nvim_screengrid_get_zindex(layer) <= curgrid_zindex {
                    break;
                }
                new_index -= 1;
            }

            let comp_index = nvim_screengrid_get_comp_index(curgrid);
            if comp_index < new_index {
                ui_comp_raise_grid_impl(curgrid, new_index);
            }
        }

        // Bounds check: cursor must be within the default grid
        let default_grid = nvim_get_default_grid();
        let default_cols = nvim_screengrid_get_cols(default_grid);
        let default_rows = nvim_screengrid_get_rows(default_grid);

        if cursor_col >= default_cols || cursor_row >= default_rows {
            return;
        }

        // Send cursor position to composed UIs (grid handle 1 = default/composed)
        nvim_ui_composed_call_grid_cursor_goto(1, cursor_row, cursor_col);
    }
}

/// FFI wrapper for `ui_comp_grid_cursor_goto`.
///
/// Handles cursor positioning for grid compositor, including raising
/// the grid in the layer stack if needed.
///
/// # Safety
/// This function accesses global compositor state.
#[no_mangle]
pub extern "C" fn rs_ui_comp_grid_cursor_goto(grid_handle: i64, r: i64, c: i64) {
    ui_comp_grid_cursor_goto_impl(grid_handle as HandleT, r, c);
}

/// Adjust a layer's position in the stack based on z-index.
///
/// This function moves a layer up (raise=true) or down (raise=false) in the
/// layer stack to maintain z-index ordering. Layers are swapped one position
/// at a time until the layer is in the correct position.
fn ui_comp_layers_adjust_impl(layer_idx: usize, raise: bool) {
    unsafe {
        let size = nvim_layers_size();
        let layer = nvim_layers_get(layer_idx);
        let layer_zindex = nvim_screengrid_get_zindex(layer);
        let mut idx = layer_idx;

        if raise {
            // Move layer up (towards higher indices) while its zindex > next layer's zindex
            while idx < size - 1 {
                let next_layer = nvim_layers_get(idx + 1);
                let next_zindex = nvim_screengrid_get_zindex(next_layer);
                if layer_zindex <= next_zindex {
                    break;
                }
                // Swap: move next_layer down to current position
                nvim_layers_set(idx, next_layer);
                nvim_screengrid_set_comp_index(next_layer, idx);
                nvim_screengrid_set_pending_comp_index_update(next_layer, true);
                idx += 1;
            }
        } else {
            // Move layer down (towards lower indices) while its zindex < prev layer's zindex
            while idx > 0 {
                let prev_layer = nvim_layers_get(idx - 1);
                let prev_zindex = nvim_screengrid_get_zindex(prev_layer);
                if layer_zindex >= prev_zindex {
                    break;
                }
                // Swap: move prev_layer up to current position
                nvim_layers_set(idx, prev_layer);
                nvim_screengrid_set_comp_index(prev_layer, idx);
                nvim_screengrid_set_pending_comp_index_update(prev_layer, true);
                idx -= 1;
            }
        }

        // Place the layer at its final position
        nvim_layers_set(idx, layer);
        nvim_screengrid_set_comp_index(layer, idx);
        nvim_screengrid_set_pending_comp_index_update(layer, true);
    }
}

/// FFI wrapper for `ui_comp_layers_adjust`.
///
/// Adjusts a layer's position in the stack based on z-index ordering.
///
/// # Safety
/// This function modifies global compositor state.
#[no_mangle]
pub extern "C" fn rs_ui_comp_layers_adjust(layer_idx: usize, raise: bool) {
    ui_comp_layers_adjust_impl(layer_idx, raise);
}

/// Set screen validity and return old value.
///
/// When setting to invalid, also resets the message separator row.
fn ui_comp_set_screen_valid_impl(valid: bool) -> bool {
    unsafe {
        let old_val = nvim_get_valid_screen() != 0;
        nvim_set_valid_screen(valid);
        if !valid {
            nvim_set_msg_sep_row(-1);
        }
        old_val
    }
}

/// FFI wrapper for `ui_comp_set_screen_valid`.
///
/// Sets the screen validity flag and returns the old value.
///
/// # Safety
/// This function modifies global compositor state.
#[no_mangle]
pub extern "C" fn rs_ui_comp_set_screen_valid(valid: bool) -> bool {
    ui_comp_set_screen_valid_impl(valid)
}

/// Check if a point (row, col) is within a grid's bounds.
#[inline]
fn point_in_grid(row: c_int, col: c_int, grid: ScreenGridHandle) -> bool {
    unsafe {
        let grid_row = nvim_screengrid_get_comp_row(grid);
        let grid_col = nvim_screengrid_get_comp_col(grid);
        let grid_rows = nvim_screengrid_get_rows(grid);
        let grid_cols = nvim_screengrid_get_cols(grid);

        row >= grid_row
            && row < grid_row + grid_rows
            && col >= grid_col
            && col < grid_col + grid_cols
    }
}

/// Get the topmost grid at given screen coordinates.
///
/// This function first checks the compositor layers (floating windows, etc.)
/// from top to bottom, then falls back to checking window grids in the
/// current tab, and finally returns the default grid if no match is found.
fn ui_comp_get_grid_at_coord_impl(row: c_int, col: c_int) -> ScreenGridHandle {
    unsafe {
        // Check compositor layers from top to bottom (skip layer 0 which is default_grid)
        let layers_size = nvim_layers_size();
        if layers_size > 1 {
            for i in (1..layers_size).rev() {
                let grid = nvim_layers_get(i);
                if point_in_grid(row, col, grid) {
                    return grid;
                }
            }
        }

        // Check window grids in current tab
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            let grid = nvim_win_get_grid_alloc(wp);
            let is_hidden = nvim_win_get_config_hide(wp) != 0;
            if point_in_grid(row, col, grid) && !is_hidden {
                return grid;
            }
            wp = nvim_win_get_next(wp);
        }

        // Fall back to default grid
        nvim_get_default_grid()
    }
}

/// FFI wrapper for `ui_comp_get_grid_at_coord`.
///
/// Returns the topmost grid at the given screen coordinates.
///
/// # Safety
/// This function accesses global compositor state and window list.
#[no_mangle]
pub extern "C" fn rs_ui_comp_get_grid_at_coord(row: c_int, col: c_int) -> ScreenGridHandle {
    ui_comp_get_grid_at_coord_impl(row, col)
}

/// Check if a point is within a grid's bounds with mouse check.
#[inline]
fn point_in_grid_with_mouse(
    row: c_int,
    col: c_int,
    grid: ScreenGridHandle,
    grid_row: c_int,
    grid_col: c_int,
) -> bool {
    unsafe {
        let mouse_enabled = nvim_screengrid_get_mouse_enabled(grid);
        if !mouse_enabled {
            return false;
        }
        let grid_rows = nvim_screengrid_get_rows(grid);
        let grid_cols = nvim_screengrid_get_cols(grid);

        row >= grid_row
            && row < grid_row + grid_rows
            && col >= grid_col
            && col < grid_col + grid_cols
    }
}

/// Get the grid that should receive mouse focus at given coordinates.
///
/// Similar to `ui_comp_get_grid_at_coord` but:
/// - Checks `mouse_enabled` flag on grids
/// - Only checks window grids if multigrid UI is enabled
/// - Uses window position instead of grid comp position for window grids
/// - Returns null handle if no suitable grid found
fn ui_comp_mouse_focus_impl(row: c_int, col: c_int) -> ScreenGridHandle {
    unsafe {
        // Check compositor layers from top to bottom (skip layer 0 which is default_grid)
        let layers_size = nvim_layers_size();
        if layers_size > 1 {
            for i in (1..layers_size).rev() {
                let grid = nvim_layers_get(i);
                let grid_row = nvim_screengrid_get_comp_row(grid);
                let grid_col = nvim_screengrid_get_comp_col(grid);
                if point_in_grid_with_mouse(row, col, grid, grid_row, grid_col) {
                    return grid;
                }
            }
        }

        // Check window grids only if multigrid is enabled
        if nvim_get_ui_ext(ui_ext::MULTIGRID) != 0 {
            let mut wp = nvim_get_firstwin();
            while !wp.is_null() {
                let grid = nvim_win_get_grid_alloc(wp);
                let winrow = nvim_win_get_winrow(wp);
                let wincol = nvim_win_get_wincol(wp);
                if point_in_grid_with_mouse(row, col, grid, winrow, wincol) {
                    return grid;
                }
                wp = nvim_win_get_next(wp);
            }
        }

        // Return null handle if no grid found
        ScreenGridHandle(std::ptr::null_mut())
    }
}

/// FFI wrapper for `ui_comp_mouse_focus`.
///
/// Returns the grid that should receive mouse focus at the given coordinates.
///
/// # Safety
/// This function accesses global compositor state and window list.
#[no_mangle]
pub extern "C" fn rs_ui_comp_mouse_focus(row: c_int, col: c_int) -> ScreenGridHandle {
    ui_comp_mouse_focus_impl(row, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screengrid_handle_null() {
        let null_handle = ScreenGridHandle(std::ptr::null_mut());
        assert!(null_handle.is_null());
    }

    #[test]
    fn test_win_handle_null() {
        let null_handle = WinHandle(std::ptr::null_mut());
        assert!(null_handle.is_null());
    }

    #[test]
    fn test_zindex_constants() {
        // Verify z-index ordering using runtime values to avoid const optimization
        let default = zindex::DEFAULT_GRID;
        let float = zindex::FLOAT_DEFAULT;
        let popup = zindex::POPUP_MENU;
        let messages = zindex::MESSAGES;
        let cmdline = zindex::CMDLINE_POPUP_MENU;

        assert!(default < float);
        assert!(float < popup);
        assert!(popup < messages);
        assert!(messages < cmdline);
    }

    #[test]
    fn test_zindex_values() {
        // Verify specific z-index values match C definitions
        assert_eq!(zindex::DEFAULT_GRID, 0);
        assert_eq!(zindex::FLOAT_DEFAULT, 50);
        assert_eq!(zindex::POPUP_MENU, 100);
        assert_eq!(zindex::MESSAGES, 200);
        assert_eq!(zindex::CMDLINE_POPUP_MENU, 250);
    }

    #[test]
    fn test_type_alias_sizes() {
        // ScharT should be 4 bytes (uint32_t)
        assert_eq!(std::mem::size_of::<ScharT>(), 4);
        // SattrT should be 4 bytes (int32_t)
        assert_eq!(std::mem::size_of::<SattrT>(), 4);
        // HandleT should be c_int size (typically 4 bytes)
        assert_eq!(std::mem::size_of::<HandleT>(), std::mem::size_of::<c_int>());
    }

    #[test]
    fn test_handle_sizes() {
        // All handles should be pointer-sized
        let ptr_size = std::mem::size_of::<*mut c_void>();
        assert_eq!(std::mem::size_of::<ScreenGridHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<WinHandle>(), ptr_size);
    }

    #[test]
    fn test_ui_ext_multigrid_constant() {
        // Verify kUIMultigrid = 6 from ui_defs.h
        assert_eq!(ui_ext::MULTIGRID, 6);
    }

    #[test]
    fn test_zindex_gaps() {
        // Verify there's room between z-index levels for custom windows
        let levels = [
            zindex::DEFAULT_GRID,
            zindex::FLOAT_DEFAULT,
            zindex::POPUP_MENU,
            zindex::MESSAGES,
            zindex::CMDLINE_POPUP_MENU,
        ];
        for window in levels.windows(2) {
            let gap = window[1] - window[0];
            assert!(gap >= 50, "Gap between z-index levels should be >= 50");
        }
    }
}
