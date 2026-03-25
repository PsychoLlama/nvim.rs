//! Popup menu display orchestration.
//!
//! This module provides helper functions for showing, hiding, and managing
//! the popup menu display state.

use std::ffi::c_int;

use crate::PUM_STATE;

// External functions needed (not PumState fields)
extern "C" {
    static Columns: c_int;
    static mut State: c_int;
    /// Set the `must_redraw_pum` global variable.
    fn nvim_set_must_redraw_pum(val: c_int);
}

// External UI functions.
extern "C" {
    /// Check if UI has a capability.
    fn ui_has(what: c_int) -> bool;
}

// Direct grid function declarations.
extern "C" {
    /// Compose grid (calls `ui_comp_compose_grid`).
    fn ui_comp_compose_grid(grid: *mut crate::ScreenGrid);
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
    let state = State;
    let is_cmdline = (state & MODE_CMDLINE) != 0;

    // Only change draw mode when popup is not visible
    let external = if is_visible == 0 {
        let has_popupmenu = ui_has(K_UI_POPUPMENU);
        let has_wildmenu = ui_has(K_UI_WILDMENU);
        c_int::from(has_popupmenu || (is_cmdline && has_wildmenu))
    } else {
        PUM_STATE.external
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
    PUM_STATE.is_visible = 1;
    PUM_STATE.is_drawn = 1;
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
    PUM_STATE.external = external;
    PUM_STATE.rl = rl;
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
    PUM_STATE.is_visible = 0;
    PUM_STATE.array = std::ptr::null_mut();
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
    c_int::from(PUM_STATE.is_visible == 0 && PUM_STATE.is_drawn != 0)
}

/// Mark the popup as cleared from display.
///
/// Call this after successfully clearing the popup display.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_mark_cleared() {
    PUM_STATE.is_drawn = 0;
    PUM_STATE.external = 0;
}

/// Clear the popup menu scroll position.
///
/// Resets `pum_first` to 0.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_clear_scroll() {
    PUM_STATE.first = 0;
}

/// Mark the popup menu as invalid (needs redraw).
///
/// Called when the screen was cleared.
///
/// # Safety
/// Calls C accessor function.
#[export_name = "pum_invalidate"]
pub unsafe extern "C" fn rs_pum_invalidate() {
    PUM_STATE.invalid = 1;
}

/// Check if the popup menu is marked invalid.
///
/// Returns 1 if invalid, 0 otherwise.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_is_invalid() -> c_int {
    PUM_STATE.invalid
}

/// Clear the invalid flag.
///
/// Call this after redrawing the popup.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_clear_invalid() {
    PUM_STATE.invalid = 0;
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
    let pum_size = PUM_STATE.size;
    let is_visible = PUM_STATE.is_visible != 0;

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
    PUM_STATE.external
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
    PUM_STATE.external
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

// Direct C function declarations for check_clear.
extern "C" {
    /// Call `ui_call_popupmenu_hide()`.
    fn ui_call_popupmenu_hide();
    /// Remove grid from compositor.
    fn ui_comp_remove_grid(grid: *mut crate::ScreenGrid);
    /// Notify UI of window close.
    fn ui_call_win_close(grid: i64);
    /// Notify UI of grid destruction.
    fn ui_call_grid_destroy(grid: i64);
    /// Free a `ScreenGrid`.
    fn grid_free(grid: *mut crate::ScreenGrid);
    /// Find the floating preview window (returns NULL if none).
    fn win_float_find_preview() -> *mut WinHandle;
    /// Close a window.
    fn win_close(wp: *mut WinHandle, free_buf: bool, force: bool) -> c_int;
}

// Phase 8: Display orchestrator C accessor functions.
extern "C" {
    /// Validate cursor column in the given window.
    fn validate_cursor_col(wp: *mut WinHandle);
    /// Compute display geometry (`pum_win_row`, `cursor_col`, anchor, offsets, above/below).
    fn nvim_pum_compute_geometry(cmd_startcol: c_int) -> PumDisplayGeometry;
    /// Send external popupmenu show event with Arena-allocated arrays.
    fn nvim_pum_ext_show(
        array: *mut crate::item::PumItemArray,
        size: c_int,
        selected: c_int,
        pum_win_row: c_int,
        cursor_col: c_int,
        anchor_grid: c_int,
        win_row_offset: c_int,
        win_col_offset: c_int,
    );
    /// Send external popupmenu select event.
    fn ui_call_popupmenu_select(selected: i64);
    /// Find preview window and compute above/below row adjustments.
    fn nvim_pum_find_pvwin_rows(above_row_out: *mut c_int, below_row_out: *mut c_int);
    /// Compute vertical placement (writes `PUM_STATE.row`, `.height`, `.above`).
    fn nvim_pum_compute_vp(
        size: c_int,
        pum_win_row: c_int,
        above_row: c_int,
        below_row: c_int,
        border_width: c_int,
    );
    /// Compute horizontal placement (writes `PUM_STATE.col`, `.width`).
    fn nvim_pum_compute_hp(cursor_col: c_int);
    /// Get `w_p_rl` for a window.
    #[link_name = "nvim_win_get_p_rl"]
    fn nvim_win_get_w_p_rl(wp: *mut WinHandle) -> c_int;
    /// Get `Columns`.
    /// Set selected item (Rust function via extern "C").
    fn rs_pum_set_selected(n: c_int, repeat: c_int) -> c_int;
    /// Get border width from Rust.
    fn rs_pum_border_width() -> c_int;
    /// Compute item widths and write to `PUM_STATE` (Rust function via extern "C").
    fn rs_pum_compute_size(array: *const crate::item::PumItemArray);
}

// C globals used by display.
extern "C" {
    /// C global: `curwin` (current window pointer).
    static mut curwin: *mut WinHandle;
    /// C global: `pum_grid` (the popup menu grid).
    static mut pum_grid: crate::ScreenGrid;
}

/// Result of geometry computation from C.
#[repr(C)]
struct PumDisplayGeometry {
    pum_win_row: c_int,
    cursor_col: c_int,
    anchor_grid: c_int,
    win_row_offset: c_int,
    win_col_offset: c_int,
    above_row: c_int,
    below_row: c_int,
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
#[export_name = "pum_recompose"]
pub unsafe extern "C" fn rs_pum_recompose() {
    ui_comp_compose_grid(&raw mut pum_grid);
}

/// Check and clear the popup menu display if needed.
///
/// If the popup is not visible but still drawn, tears down the grid and
/// closes the floating preview window. Handles both external and internal
/// popup display modes.
///
/// # Safety
/// Calls C accessor and UI functions.
#[export_name = "pum_check_clear"]
pub unsafe extern "C" fn rs_pum_check_clear() {
    let is_visible = PUM_STATE.is_visible != 0;
    let is_drawn = PUM_STATE.is_drawn != 0;

    if !is_visible && is_drawn {
        let is_external = PUM_STATE.external != 0;
        if is_external {
            ui_call_popupmenu_hide();
        } else {
            ui_comp_remove_grid(&raw mut pum_grid);
            if ui_has(K_UI_MULTIGRID) {
                ui_call_win_close(pum_grid.handle as i64);
                ui_call_grid_destroy(pum_grid.handle as i64);
            }
            grid_free(&raw mut pum_grid);
        }
        PUM_STATE.is_drawn = 0;
        PUM_STATE.external = 0;

        let wp = win_float_find_preview();
        if !wp.is_null() {
            win_close(wp, false, false);
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
#[export_name = "pum_ui_flush"]
pub unsafe extern "C" fn rs_pum_ui_flush() {
    let has_multigrid = ui_has(K_UI_MULTIGRID);
    let is_drawn = PUM_STATE.is_drawn != 0;
    let is_external = PUM_STATE.external != 0;
    let handle = pum_grid.handle;
    let pending = pum_grid.pending_comp_index_update != 0;

    if has_multigrid && is_drawn && !is_external && handle != 0 && pending {
        let pum_above = PUM_STATE.above != 0;
        let pum_height = PUM_STATE.height;
        let anchor = if pum_above {
            c"SW".as_ptr()
        } else {
            c"NW".as_ptr()
        };
        let row_off = if pum_above { -pum_height } else { 0 };
        let pum_row = PUM_STATE.row;
        let pum_left_col = PUM_STATE.left_col;
        let win_row_offset = PUM_STATE.win_row_offset;
        let win_col_offset = PUM_STATE.win_col_offset;
        let anchor_grid = PUM_STATE.anchor_grid;
        let zindex = pum_grid.zindex;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let comp_index = pum_grid.comp_index as c_int;
        let comp_row = pum_grid.comp_row;
        let comp_col = pum_grid.comp_col;

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
        pum_grid.pending_comp_index_update = 0;
    }
}

// rs_pum_preview_set_text: moved to preview.rs
// rs_pum_adjust_info_position: moved to preview.rs
// rs_pum_set_info: moved to preview.rs
// rs_pum_set_selected: moved to selection.rs

/// Display the popup menu.
///
/// Shows the popup menu with the given items array. Handles:
/// - Display mode determination (external vs internal)
/// - Geometry computation (position, size, anchor)
/// - External UI events (`popupmenu_show`/`select`)
/// - Internal rendering (vertical/horizontal placement, redraw)
/// - Preview window row adjustments
///
/// # Safety
/// `array` must be a valid `pumitem_T` array pointer with at least `size` elements.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_pum_display(
    array: *mut crate::item::PumItemArray,
    size: c_int,
    selected: c_int,
    array_changed: c_int,
    cmd_startcol: c_int,
) {
    let mut redo_count: c_int = 0;

    // Determine display mode (external/rl) only when not already visible
    let is_visible = PUM_STATE.is_visible;
    let state = State;
    let is_cmdline = (state & MODE_CMDLINE) != 0;

    if is_visible == 0 {
        let has_popupmenu = ui_has(K_UI_POPUPMENU);
        let has_wildmenu = ui_has(K_UI_WILDMENU);
        PUM_STATE.external = c_int::from(has_popupmenu || (is_cmdline && has_wildmenu));
    }

    let curwin_rl = nvim_win_get_w_p_rl(curwin);
    PUM_STATE.rl = if is_cmdline { 0 } else { curwin_rl };

    let border_width = rs_pum_border_width();

    loop {
        // Mark as visible early to avoid must_redraw when 'cursorcolumn' is on
        PUM_STATE.is_visible = 1;
        PUM_STATE.is_drawn = 1;
        validate_cursor_col(curwin);

        // Compute geometry from C (handles target_win, cmdline_win, grid offsets)
        let geom = nvim_pum_compute_geometry(cmd_startcol);
        PUM_STATE.win_row_offset = geom.win_row_offset;
        PUM_STATE.win_col_offset = geom.win_col_offset;
        PUM_STATE.anchor_grid = geom.anchor_grid;

        let pum_win_row = geom.pum_win_row;
        let cursor_col = geom.cursor_col;

        if PUM_STATE.external != 0 {
            if array_changed != 0 {
                nvim_pum_ext_show(
                    array,
                    size,
                    selected,
                    pum_win_row,
                    cursor_col,
                    geom.anchor_grid,
                    geom.win_row_offset,
                    geom.win_col_offset,
                );
            } else {
                ui_call_popupmenu_select(i64::from(selected));
                return;
            }
        }

        // Find preview window and adjust above/below rows
        let mut above_row = geom.above_row;
        let mut below_row = geom.below_row;
        let mut pvwin_above: c_int = 0;
        let mut pvwin_below: c_int = 0;
        nvim_pum_find_pvwin_rows(
            std::ptr::addr_of_mut!(pvwin_above),
            std::ptr::addr_of_mut!(pvwin_below),
        );
        if pvwin_above > 0 {
            above_row = pvwin_above;
        }
        if pvwin_below > 0 {
            below_row = pvwin_below;
        }

        // Compute vertical placement (writes PUM_STATE.row, .height, .above)
        nvim_pum_compute_vp(size, pum_win_row, above_row, below_row, border_width);

        // Don't display when we only have room for one line
        let pum_height = PUM_STATE.height;
        if border_width == 0 && (pum_height < 1 || (pum_height == 1 && size > 1)) {
            return;
        }

        // Set array and size
        PUM_STATE.array = array;
        PUM_STATE.size = size;

        if PUM_STATE.external != 0 {
            return;
        }

        // Compute item widths (writes PUM_STATE.base_width, .kind_width, .extra_width)
        rs_pum_compute_size(PUM_STATE.array);

        // If there are more items than room we need a scrollbar
        let pum_height = PUM_STATE.height;
        PUM_STATE.scrollbar = c_int::from(pum_height < size);

        // Compute horizontal placement (writes PUM_STATE.col, .width)
        nvim_pum_compute_hp(cursor_col);

        // Adjust for border overflow
        let pum_col = PUM_STATE.col;
        let pum_width = PUM_STATE.width;
        let columns = Columns;
        if pum_col + border_width + pum_width > columns {
            PUM_STATE.col = pum_col - border_width;
        }

        // Set selected item and redraw. If the window size changed need to
        // redo the positioning. Limit to two times.
        let resized = rs_pum_set_selected(selected, redo_count) != 0;
        redo_count += 1;
        if !resized || redo_count > 2 {
            break;
        }
    }

    // kZIndexCmdlinePopupMenu = 250, kZIndexPopupMenu = 100
    pum_grid.zindex = if (State & MODE_CMDLINE) != 0 {
        250
    } else {
        100
    };
    crate::redraw::rs_pum_redraw();
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
