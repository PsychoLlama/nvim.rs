//! Context menu and UI integration.
//!
//! This module provides helper functions for context menu popup
//! positioning and UI flush operations.

use std::ffi::c_int;

// C accessor functions for context menu state.
extern "C" {
    /// Get the `pum_row` static variable.
    fn nvim_get_pum_row() -> c_int;
    /// Get the `pum_height` static variable.
    fn nvim_get_pum_height() -> c_int;
    /// Get the `pum_above` static variable.
    fn nvim_get_pum_above() -> c_int;
    /// Get the `pum_left_col` static variable.
    fn nvim_get_pum_left_col() -> c_int;
    /// Get the `pum_is_drawn` static variable.
    fn nvim_get_pum_is_drawn() -> c_int;
    /// Get the `pum_external` static variable.
    fn nvim_get_pum_external() -> c_int;
    /// Get the `pum_anchor_grid` static variable.
    fn nvim_get_pum_anchor_grid() -> c_int;
    /// Get the `pum_win_row_offset` static variable.
    fn nvim_get_pum_win_row_offset() -> c_int;
    /// Get the `pum_win_col_offset` static variable.
    fn nvim_get_pum_win_col_offset() -> c_int;
}

/// Result of UI flush position calculation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumFlushPos {
    /// Whether a flush is needed.
    pub needs_flush: c_int,
    /// Row offset to apply.
    pub row_off: c_int,
    /// Adjusted row position.
    pub row: c_int,
    /// Left column position.
    pub col: c_int,
    /// Grid anchor ("NW" or "SW").
    pub is_above: c_int,
}

/// Calculate position for UI flush operation.
///
/// This is used when the popup menu position needs to be updated
/// in multigrid mode.
///
/// # Arguments
/// * `grid_handle` - Grid handle for popup menu (0 if none)
/// * `pending_update` - Whether there's a pending compositor index update
/// * `has_multigrid` - Whether multigrid mode is active
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_calc_flush_pos(
    grid_handle: c_int,
    pending_update: c_int,
    has_multigrid: c_int,
) -> PumFlushPos {
    // Check if flush is needed
    let is_drawn = nvim_get_pum_is_drawn() != 0;
    let is_external = nvim_get_pum_external() != 0;
    let has_handle = grid_handle != 0;
    let is_pending = pending_update != 0;
    let is_multigrid = has_multigrid != 0;

    if !is_multigrid || !is_drawn || is_external || !has_handle || !is_pending {
        return PumFlushPos {
            needs_flush: 0,
            row_off: 0,
            row: 0,
            col: 0,
            is_above: 0,
        };
    }

    let pum_above = nvim_get_pum_above() != 0;
    let pum_height = nvim_get_pum_height();
    let pum_row = nvim_get_pum_row();
    let pum_left_col = nvim_get_pum_left_col();
    let win_row_offset = nvim_get_pum_win_row_offset();
    let win_col_offset = nvim_get_pum_win_col_offset();

    let row_off = if pum_above { -pum_height } else { 0 };
    let row = pum_row - row_off - win_row_offset;
    let col = pum_left_col - win_col_offset;

    PumFlushPos {
        needs_flush: 1,
        row_off,
        row,
        col,
        is_above: pum_above as c_int,
    }
}

/// Get the anchor grid for UI positioning.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_get_anchor_grid() -> c_int {
    nvim_get_pum_anchor_grid()
}

/// Calculate mouse position for context menu at cursor.
///
/// # Arguments
/// * `grid_row_offset` - Grid row offset
/// * `wrow` - Window row position
/// * `grid_col_offset` - Grid column offset
/// * `wcol` - Window column position
/// * `view_width` - Window view width
/// * `is_rl` - Whether right-to-left mode
///
/// Returns (row, col) position.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumCursorPos {
    /// Mouse row position.
    pub row: c_int,
    /// Mouse column position.
    pub col: c_int,
}

#[no_mangle]
pub const extern "C" fn rs_pum_cursor_to_mouse_pos(
    grid_row_offset: c_int,
    wrow: c_int,
    grid_col_offset: c_int,
    wcol: c_int,
    view_width: c_int,
    is_rl: c_int,
) -> PumCursorPos {
    let row = grid_row_offset + wrow;
    let col = grid_col_offset
        + if is_rl != 0 {
            view_width - wcol - 1
        } else {
            wcol
        };

    PumCursorPos { row, col }
}

/// Adjust mouse position for non-multigrid mode.
///
/// # Arguments
/// * `row` - Current row position
/// * `col` - Current column position
/// * `winrow` - Window row position
/// * `wincol` - Window column position
///
/// Returns adjusted (row, col).
#[no_mangle]
pub const extern "C" fn rs_pum_adjust_mouse_pos_non_multigrid(
    row: c_int,
    col: c_int,
    winrow: c_int,
    wincol: c_int,
) -> PumCursorPos {
    PumCursorPos {
        row: row + winrow,
        col: col + wincol,
    }
}

/// Key codes for context menu handling.
pub mod keys {
    use std::ffi::c_int;

    /// Escape key.
    pub const ESC: c_int = 27;
    /// Ctrl-C.
    pub const CTRL_C: c_int = 3;
    /// Carriage return.
    pub const CAR: c_int = 13;
    /// Newline.
    pub const NL: c_int = 10;
}

/// Result of context menu key handling.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumMenuKeyResult {
    /// Action to take.
    /// 0 = continue loop
    /// 1 = break (exit menu)
    /// 2 = execute and break
    /// 3 = select previous
    /// 4 = select next
    /// 5 = reposition (right mouse)
    /// 6 = mouse move select
    /// 7 = mouse click select
    pub action: c_int,
}

/// Determine action for context menu key input.
///
/// Returns action code indicating what the menu should do.
#[no_mangle]
pub const extern "C" fn rs_pum_menu_key_action(
    key: c_int,
    has_array: c_int,
    k_up: c_int,
    k_down: c_int,
    k_mouseup: c_int,
    k_mousedown: c_int,
    k_rightmouse: c_int,
    k_leftdrag: c_int,
    k_rightdrag: c_int,
    k_mousemove: c_int,
    k_leftmouse: c_int,
    k_leftmouse_nm: c_int,
    k_rightrelease: c_int,
) -> PumMenuKeyResult {
    // Check for exit conditions
    if key == keys::ESC || key == keys::CTRL_C || has_array == 0 {
        return PumMenuKeyResult { action: 1 }; // break
    }

    // Enter: execute and break
    if key == keys::CAR || key == keys::NL {
        return PumMenuKeyResult { action: 2 }; // execute and break
    }

    // Cursor up / k
    if key == b'k' as c_int || key == k_up || key == k_mouseup {
        return PumMenuKeyResult { action: 3 }; // select previous
    }

    // Cursor down / j
    if key == b'j' as c_int || key == k_down || key == k_mousedown {
        return PumMenuKeyResult { action: 4 }; // select next
    }

    // Right mouse: reposition
    if key == k_rightmouse {
        return PumMenuKeyResult { action: 5 }; // reposition
    }

    // Mouse move: select at position
    if key == k_leftdrag || key == k_rightdrag || key == k_mousemove {
        return PumMenuKeyResult { action: 6 }; // mouse move select
    }

    // Mouse click: select and maybe execute
    if key == k_leftmouse || key == k_leftmouse_nm || key == k_rightrelease {
        return PumMenuKeyResult { action: 7 }; // mouse click select
    }

    // Default: continue loop
    PumMenuKeyResult { action: 0 }
}

/// Opaque handle to a `vimmenu_T`.
#[repr(C)]
pub struct VimMenuHandle {
    _private: [u8; 0],
}

// C accessor functions for menu traversal.
extern "C" {
    /// Get `menu->children` (first child menu item).
    fn nvim_pum_menu_children(menu: *mut VimMenuHandle) -> *mut VimMenuHandle;
    /// Get `menu->next` (next sibling menu item).
    fn nvim_pum_menu_next(menu: *mut VimMenuHandle) -> *mut VimMenuHandle;
    /// Check if menu item matches mode: `(mp->modes & mp->enabled & mode) != 0`.
    fn nvim_pum_menu_matches_mode(menu: *mut VimMenuHandle, mode: c_int) -> c_int;
    /// Execute a menu item (`CLEAR_FIELD(ea); execute_menu(&ea, mp, -1)`).
    fn nvim_pum_execute_menu_item(menu: *mut VimMenuHandle);
    /// Get the `pum_selected` static variable.
    fn nvim_get_pum_selected() -> c_int;
}

// C `_impl` functions for later phase migrations.
extern "C" {
    /// Show the terminal popup menu.
    fn nvim_pum_show_popupmenu_impl(menu: *mut VimMenuHandle);
    /// Create a popup from a menu path.
    fn nvim_pum_make_popup_impl(path_name: *const std::ffi::c_char, use_mouse_pos: c_int);
}

/// Execute the currently selected popup menu item.
///
/// Walks the menu's children linked list, counting items that match
/// the given mode. When the count matches `pum_selected`, executes
/// that menu item.
///
/// # Safety
/// Calls C accessor functions. `menu` must be a valid `vimmenu_T` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_execute_menu(menu: *mut VimMenuHandle, mode: c_int) {
    let pum_selected = nvim_get_pum_selected();
    let mut idx = 0;
    let mut mp = nvim_pum_menu_children(menu);
    while !mp.is_null() {
        if nvim_pum_menu_matches_mode(mp, mode) != 0 {
            if idx == pum_selected {
                nvim_pum_execute_menu_item(mp);
                return;
            }
            idx += 1;
        }
        mp = nvim_pum_menu_next(mp);
    }
}

/// Open the terminal version of the popup menu.
///
/// # Safety
/// Calls C `_impl` function. `menu` must be a valid `vimmenu_T` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_show_popupmenu(menu: *mut VimMenuHandle) {
    nvim_pum_show_popupmenu_impl(menu);
}

/// Create a popup from a menu path.
///
/// # Safety
/// Calls C `_impl` function. `path_name` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_make_popup(
    path_name: *const std::ffi::c_char,
    use_mouse_pos: c_int,
) {
    nvim_pum_make_popup_impl(path_name, use_mouse_pos);
}

/// Move selection in context menu, skipping separators.
///
/// # Arguments
/// * `current` - Current selection
/// * `direction` - -1 for previous, 1 for next
/// * `size` - Total number of items
///
/// Returns new selection index.
#[no_mangle]
pub const extern "C" fn rs_pum_menu_move_selection(
    current: c_int,
    direction: c_int,
    size: c_int,
) -> c_int {
    if direction < 0 {
        // Moving up
        if current > 0 {
            current - 1
        } else {
            current
        }
    } else {
        // Moving down
        if current < size - 1 {
            current + 1
        } else {
            current
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_to_mouse_ltr() {
        let pos = rs_pum_cursor_to_mouse_pos(5, 10, 3, 15, 80, 0);
        assert_eq!(pos.row, 15); // 5 + 10
        assert_eq!(pos.col, 18); // 3 + 15
    }

    #[test]
    fn test_cursor_to_mouse_rtl() {
        let pos = rs_pum_cursor_to_mouse_pos(5, 10, 3, 15, 80, 1);
        assert_eq!(pos.row, 15);
        assert_eq!(pos.col, 67); // 3 + (80 - 15 - 1)
    }

    #[test]
    fn test_adjust_non_multigrid() {
        let pos = rs_pum_adjust_mouse_pos_non_multigrid(10, 20, 5, 3);
        assert_eq!(pos.row, 15);
        assert_eq!(pos.col, 23);
    }

    #[test]
    fn test_menu_move_selection() {
        // Move down
        assert_eq!(rs_pum_menu_move_selection(0, 1, 5), 1);
        assert_eq!(rs_pum_menu_move_selection(4, 1, 5), 4); // at end

        // Move up
        assert_eq!(rs_pum_menu_move_selection(3, -1, 5), 2);
        assert_eq!(rs_pum_menu_move_selection(0, -1, 5), 0); // at start
    }

    #[test]
    fn test_menu_key_action_escape() {
        let result = rs_pum_menu_key_action(27, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(result.action, 1); // break
    }

    #[test]
    fn test_menu_key_action_enter() {
        let result = rs_pum_menu_key_action(13, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(result.action, 2); // execute and break
    }
}
