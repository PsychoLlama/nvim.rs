//! Popup menu display orchestration.
//!
//! This module provides helper functions for showing, hiding, and managing
//! the popup menu display state.

use std::ffi::{c_int, c_void};

// C accessor functions for display state.
extern "C" {
    /// Get the `pum_is_visible` static variable.
    fn nvim_get_pum_is_visible() -> c_int;
    /// Set the `pum_is_visible` static variable.
    fn nvim_set_pum_is_visible(val: c_int);
    /// Get the `pum_is_drawn` static variable.
    fn nvim_get_pum_is_drawn() -> c_int;
    /// Set the `pum_is_drawn` static variable.
    fn nvim_set_pum_is_drawn(val: c_int);
    /// Get the `pum_external` static variable.
    fn nvim_get_pum_external() -> c_int;
    /// Set the `pum_external` static variable.
    fn nvim_set_pum_external(val: c_int);
    /// Get the `pum_invalid` static variable.
    fn nvim_get_pum_invalid() -> c_int;
    /// Set the `pum_invalid` static variable.
    fn nvim_set_pum_invalid(val: c_int);
    /// Set the `must_redraw_pum` static variable.
    fn nvim_set_must_redraw_pum(val: c_int);
    /// Get the `pum_size` static variable (number of items).
    fn nvim_get_pum_size() -> c_int;
    /// Set the `pum_first` static variable.
    fn nvim_set_pum_first(val: c_int);
    /// Clear the `pum_array` pointer (set to NULL).
    fn nvim_clear_pum_array();
    /// Set the `pum_rl` static variable.
    fn nvim_set_pum_rl(val: c_int);
}

// External UI functions.
extern "C" {
    /// Check if UI has a capability.
    fn ui_has(what: c_int) -> bool;
    /// Get the current State variable.
    fn nvim_get_State() -> c_int;
}

// Grid accessor functions for pum_grid fields.
extern "C" {
    /// Get pointer to `pum_grid` (`ScreenGrid`*).
    fn nvim_pum_get_grid_ptr() -> *mut c_void;
    /// Compose grid (calls `ui_comp_compose_grid`).
    fn ui_comp_compose_grid(grid: *mut c_void);
    /// Get `pum_grid.handle`.
    fn nvim_pum_grid_get_handle() -> c_int;
    /// Get `pum_grid.pending_comp_index_update`.
    fn nvim_pum_grid_get_pending_comp_index_update() -> c_int;
    /// Set `pum_grid.pending_comp_index_update`.
    fn nvim_pum_grid_set_pending_comp_index_update(val: c_int);
    /// Get `pum_grid.zindex`.
    fn nvim_pum_grid_get_zindex() -> c_int;
    /// Get `pum_grid.comp_index`.
    fn nvim_pum_grid_get_comp_index() -> c_int;
    /// Get `pum_grid.comp_row`.
    fn nvim_pum_grid_get_comp_row() -> c_int;
    /// Get `pum_grid.comp_col`.
    fn nvim_pum_grid_get_comp_col() -> c_int;
    /// Call `ui_call_win_float_pos` with pum parameters.
    fn nvim_pum_ui_call_win_float_pos(
        handle: c_int,
        anchor: *const std::ffi::c_char,
        anchor_grid: c_int,
        row: c_int,
        col: c_int,
        zindex: c_int,
        comp_index: c_int,
        comp_row: c_int,
        comp_col: c_int,
    );
}

// Pum position state accessors (used by ui_flush).
extern "C" {
    /// Get the `pum_above` static variable.
    fn nvim_get_pum_above() -> c_int;
    /// Get the `pum_height` static variable.
    fn nvim_get_pum_height() -> c_int;
    /// Get the `pum_row` static variable.
    fn nvim_get_pum_row() -> c_int;
    /// Get the `pum_left_col` static variable.
    fn nvim_get_pum_left_col() -> c_int;
    /// Get the `pum_win_row_offset` static variable.
    fn nvim_get_pum_win_row_offset() -> c_int;
    /// Get the `pum_win_col_offset` static variable.
    fn nvim_get_pum_win_col_offset() -> c_int;
    /// Get the `pum_anchor_grid` static variable.
    fn nvim_get_pum_anchor_grid() -> c_int;
}

/// UI capability for popup menu (kUIPopupmenu = 1).
const K_UI_POPUPMENU: c_int = 1;
/// UI capability for wildmenu (kUIWildmenu = 3).
const K_UI_WILDMENU: c_int = 3;
/// Mode flag for command line.
const MODE_CMDLINE: c_int = 0x08;

/// Result of display mode determination.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumDisplayMode {
    /// Whether to use external popup menu.
    pub external: c_int,
    /// Whether in right-to-left mode.
    pub rl: c_int,
}

/// Determine the display mode for the popup menu.
///
/// # Arguments
/// * `is_visible` - Whether the popup is currently visible
/// * `curwin_rl` - Whether the current window is right-to-left
///
/// # Safety
/// Calls C functions to check UI capabilities and state.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_determine_display_mode(
    is_visible: c_int,
    curwin_rl: c_int,
) -> PumDisplayMode {
    let state = nvim_get_State();
    let is_cmdline = (state & MODE_CMDLINE) != 0;

    // Only change draw mode when popup is not visible
    let external = if is_visible == 0 {
        let has_popupmenu = ui_has(K_UI_POPUPMENU);
        let has_wildmenu = ui_has(K_UI_WILDMENU);
        c_int::from(has_popupmenu || (is_cmdline && has_wildmenu))
    } else {
        nvim_get_pum_external()
    };

    // RL only applies in non-cmdline mode
    let rl = if is_cmdline { 0 } else { curwin_rl };

    PumDisplayMode { external, rl }
}

/// Mark the popup menu as visible (before any position calculations).
///
/// This should be called early in `pum_display` to prevent `must_redraw`
/// from being set when 'cursorcolumn' is on.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_mark_visible() {
    nvim_set_pum_is_visible(1);
    nvim_set_pum_is_drawn(1);
}

/// Set the external mode and RL flags for display.
///
/// # Arguments
/// * `external` - Whether to use external popup menu
/// * `rl` - Whether to use right-to-left mode
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_set_display_mode(external: c_int, rl: c_int) {
    nvim_set_pum_external(external);
    nvim_set_pum_rl(rl);
}

/// Undisplay the popup menu.
///
/// This marks the popup as not visible and clears the array pointer.
/// If `immediate` is true, also triggers clearing.
///
/// # Arguments
/// * `immediate` - Whether to immediately clear the popup display
///
/// Returns 1 if `pum_check_clear` should be called, 0 otherwise.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_undisplay(immediate: c_int) -> c_int {
    nvim_set_pum_is_visible(0);
    nvim_clear_pum_array();
    nvim_set_must_redraw_pum(0);

    // Return whether caller should call pum_check_clear
    immediate
}

/// Check if the popup menu should be cleared from display.
///
/// Returns 1 if the popup should be cleared (not visible but drawn), 0 otherwise.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_should_clear() -> c_int {
    c_int::from(nvim_get_pum_is_visible() == 0 && nvim_get_pum_is_drawn() != 0)
}

/// Mark the popup as cleared from display.
///
/// Call this after successfully clearing the popup display.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_mark_cleared() {
    nvim_set_pum_is_drawn(0);
    nvim_set_pum_external(0);
}

/// Clear the popup menu scroll position.
///
/// Resets `pum_first` to 0.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_clear_scroll() {
    nvim_set_pum_first(0);
}

/// Mark the popup menu as invalid (needs redraw).
///
/// Called when the screen was cleared.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_invalidate() {
    nvim_set_pum_invalid(1);
}

/// Check if the popup menu is marked invalid.
///
/// Returns 1 if invalid, 0 otherwise.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_is_invalid() -> c_int {
    nvim_get_pum_invalid()
}

/// Clear the invalid flag.
///
/// Call this after redrawing the popup.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_clear_invalid() {
    nvim_set_pum_invalid(0);
}

/// Result for external UI item selection.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumExtSelectResult {
    /// Whether the selection is valid.
    pub valid: c_int,
    /// The item index to select.
    pub item: c_int,
    /// Whether to insert the item.
    pub insert: c_int,
    /// Whether to finish completion.
    pub finish: c_int,
}

/// Validate and prepare external UI item selection.
///
/// # Arguments
/// * `item` - Item index to select (-1 for no selection)
/// * `insert` - Whether to insert the item
/// * `finish` - Whether to finish completion
///
/// Returns validation result with potentially adjusted values.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_validate_ext_select(
    item: c_int,
    insert: c_int,
    finish: c_int,
) -> PumExtSelectResult {
    let pum_size = nvim_get_pum_size();
    let is_visible = nvim_get_pum_is_visible() != 0;

    // Check if selection is valid
    if !is_visible || item < -1 || item >= pum_size {
        return PumExtSelectResult {
            valid: 0,
            item,
            insert,
            finish,
        };
    }

    PumExtSelectResult {
        valid: 1,
        item,
        insert,
        finish,
    }
}

/// Check if the popup menu is using external display.
///
/// Returns 1 if external, 0 otherwise.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_is_external() -> c_int {
    nvim_get_pum_external()
}

/// Check if the popup needs a scrollbar.
///
/// Returns 1 if scrollbar is needed, 0 otherwise.
///
/// # Arguments
/// * `height` - Visible height
/// * `size` - Total number of items
#[no_mangle]
pub const extern "C" fn rs_pum_display_needs_scrollbar(height: c_int, size: c_int) -> c_int {
    (height < size) as c_int
}

/// Check if display should return early for external mode.
///
/// In external mode, after sending the items to the UI, we should return
/// early without doing internal rendering.
///
/// Returns 1 if should return early, 0 otherwise.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_should_return_external() -> c_int {
    nvim_get_pum_external()
}

/// Check if there is enough room to display the popup.
///
/// Returns 1 if there is enough room, 0 otherwise.
///
/// # Arguments
/// * `height` - Computed height
/// * `size` - Total number of items
/// * `border_width` - Border width (0 if no border)
#[no_mangle]
pub const extern "C" fn rs_pum_has_room(height: c_int, size: c_int, border_width: c_int) -> c_int {
    // Don't display when we only have room for one line
    if border_width == 0 && (height < 1 || (height == 1 && size > 1)) {
        return 0;
    }
    1
}

/// UI capability for multigrid mode (kUIMultigrid = 6).
const K_UI_MULTIGRID: c_int = 6;

// C accessor functions for check_clear.
extern "C" {
    /// Call `ui_call_popupmenu_hide()`.
    fn nvim_pum_ui_call_popupmenu_hide();
    /// Call `ui_comp_remove_grid(&pum_grid)`.
    fn nvim_pum_ui_comp_remove_grid();
    /// Call `ui_call_win_close(pum_grid.handle)`.
    fn nvim_pum_ui_call_win_close_grid();
    /// Call `ui_call_grid_destroy(pum_grid.handle)`.
    fn nvim_pum_ui_call_grid_destroy();
    /// Call `grid_free(&pum_grid)`.
    fn nvim_pum_grid_free();
    /// Find the floating preview window (returns NULL if none).
    fn nvim_pum_win_float_find_preview() -> *mut WinHandle;
    /// Close a window.
    fn nvim_pum_win_close(wp: *mut WinHandle);
}

// C _impl functions for later phase migrations.
extern "C" {
    /// Display the popup menu (implementation).
    fn nvim_pum_display_impl(
        array: *mut crate::item::PumItemArray,
        size: c_int,
        selected: c_int,
        array_changed: c_int,
        cmd_startcol: c_int,
    );
}

/// Opaque handle to a `buf_T`.
#[repr(C)]
pub struct BufHandle {
    _private: [u8; 0],
}

/// Opaque handle to a `win_T`.
#[repr(C)]
pub struct WinHandle {
    _private: [u8; 0],
}

/// Recompose the popup menu grid.
///
/// Calls `ui_comp_compose_grid` on the `pum_grid` to recompose the area
/// under the popup menu. Needed when options affecting composition change
/// (e.g. 'pumblend').
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_recompose() {
    let grid = nvim_pum_get_grid_ptr();
    ui_comp_compose_grid(grid);
}

/// Check and clear the popup menu display if needed.
///
/// If the popup is not visible but still drawn, tears down the grid and
/// closes the floating preview window. Handles both external and internal
/// popup display modes.
///
/// # Safety
/// Calls C accessor and UI functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_check_clear() {
    let is_visible = nvim_get_pum_is_visible() != 0;
    let is_drawn = nvim_get_pum_is_drawn() != 0;

    if !is_visible && is_drawn {
        let is_external = nvim_get_pum_external() != 0;
        if is_external {
            nvim_pum_ui_call_popupmenu_hide();
        } else {
            nvim_pum_ui_comp_remove_grid();
            if ui_has(K_UI_MULTIGRID) {
                nvim_pum_ui_call_win_close_grid();
                nvim_pum_ui_call_grid_destroy();
            }
            nvim_pum_grid_free();
        }
        nvim_set_pum_is_drawn(0);
        nvim_set_pum_external(0);

        let wp = nvim_pum_win_float_find_preview();
        if !wp.is_null() {
            nvim_pum_win_close(wp);
        }
    }
}

/// Flush the popup menu UI position in multigrid mode.
///
/// Updates the floating window position for the popup menu grid when
/// there is a pending compositor index update. Only applies in multigrid
/// mode when the popup is drawn internally (not external).
///
/// # Safety
/// Calls C accessor functions and UI call wrappers.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_ui_flush() {
    let has_multigrid = ui_has(K_UI_MULTIGRID);
    let is_drawn = nvim_get_pum_is_drawn() != 0;
    let is_external = nvim_get_pum_external() != 0;
    let handle = nvim_pum_grid_get_handle();
    let pending = nvim_pum_grid_get_pending_comp_index_update() != 0;

    if has_multigrid && is_drawn && !is_external && handle != 0 && pending {
        let pum_above = nvim_get_pum_above() != 0;
        let pum_height = nvim_get_pum_height();
        let anchor = if pum_above {
            c"SW".as_ptr()
        } else {
            c"NW".as_ptr()
        };
        let row_off = if pum_above { -pum_height } else { 0 };
        let pum_row = nvim_get_pum_row();
        let pum_left_col = nvim_get_pum_left_col();
        let win_row_offset = nvim_get_pum_win_row_offset();
        let win_col_offset = nvim_get_pum_win_col_offset();
        let anchor_grid = nvim_get_pum_anchor_grid();
        let zindex = nvim_pum_grid_get_zindex();
        let comp_index = nvim_pum_grid_get_comp_index();
        let comp_row = nvim_pum_grid_get_comp_row();
        let comp_col = nvim_pum_grid_get_comp_col();

        nvim_pum_ui_call_win_float_pos(
            handle,
            anchor,
            anchor_grid,
            pum_row - row_off - win_row_offset,
            pum_left_col - win_col_offset,
            zindex,
            comp_index,
            comp_row,
            comp_col,
        );
        nvim_pum_grid_set_pending_comp_index_update(0);
    }
}

// rs_pum_preview_set_text: moved to preview.rs
// rs_pum_adjust_info_position: moved to preview.rs
// rs_pum_set_info: moved to preview.rs
// rs_pum_set_selected: moved to selection.rs

/// Display the popup menu.
///
/// # Safety
/// Calls C `_impl` function. `array` must be a valid `pumitem_T` array pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_display(
    array: *mut crate::item::PumItemArray,
    size: c_int,
    selected: c_int,
    array_changed: c_int,
    cmd_startcol: c_int,
) {
    nvim_pum_display_impl(array, size, selected, array_changed, cmd_startcol);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_room_no_border() {
        // Not enough room
        assert_eq!(rs_pum_has_room(0, 5, 0), 0);
        assert_eq!(rs_pum_has_room(1, 5, 0), 0);
        // Enough room
        assert_eq!(rs_pum_has_room(1, 1, 0), 1);
        assert_eq!(rs_pum_has_room(5, 10, 0), 1);
    }

    #[test]
    fn test_has_room_with_border() {
        // With border, different rules apply
        assert_eq!(rs_pum_has_room(0, 5, 1), 1);
        assert_eq!(rs_pum_has_room(1, 5, 1), 1);
    }

    #[test]
    fn test_needs_scrollbar() {
        assert_eq!(rs_pum_display_needs_scrollbar(5, 10), 1);
        assert_eq!(rs_pum_display_needs_scrollbar(10, 10), 0);
        assert_eq!(rs_pum_display_needs_scrollbar(10, 5), 0);
    }
}
