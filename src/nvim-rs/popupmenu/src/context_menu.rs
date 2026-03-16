//! Context menu and UI integration.
//!
//! This module provides helper functions for context menu popup
//! positioning and UI flush operations.

use std::ffi::c_int;

use crate::PUM_STATE;

/// Batch key constants for popup menu key handling.
///
/// Filled by `nvim_pum_get_key_constants()`. Layout must match
/// `PumKeyConstants` in `popupmenu.h`.
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct PumKeyConstants {
    key_esc: c_int,
    key_ctrl_c: c_int,
    key_car: c_int,
    key_nl: c_int,
    key_k_up: c_int,
    key_k_down: c_int,
    key_k_mouseup: c_int,
    key_k_mousedown: c_int,
    key_k_rightmouse: c_int,
    key_k_leftdrag: c_int,
    key_k_rightdrag: c_int,
    key_k_mousemove: c_int,
    key_k_leftmouse: c_int,
    key_k_leftmouse_nm: c_int,
    key_k_rightrelease: c_int,
}

/// Batch curwin geometry for popup menu positioning.
///
/// Filled by `nvim_pum_get_curwin_geometry()`. Layout must match
/// `PumCurwinGeometry` in `popupmenu.h`.
#[repr(C)]
struct PumCurwinGeometry {
    row_offset: c_int,
    col_offset: c_int,
    wrow: c_int,
    wcol: c_int,
    p_rl: c_int,
    view_width: c_int,
    winrow: c_int,
    wincol: c_int,
    grid_target_handle: c_int,
    grid_target_is_default: c_int,
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
    let is_drawn = PUM_STATE.is_drawn != 0;
    let is_external = PUM_STATE.external != 0;
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

    let pum_above = PUM_STATE.above != 0;
    let pum_height = PUM_STATE.height;
    let pum_row = PUM_STATE.row;
    let pum_left_col = PUM_STATE.left_col;
    let win_row_offset = PUM_STATE.win_row_offset;
    let win_col_offset = PUM_STATE.win_col_offset;

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
    PUM_STATE.anchor_grid
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

// C globals used by context menu.
extern "C" {
    /// C global: `p_mousemev`.
    static p_mousemev: c_int;
    /// C global: `curwin` (current window pointer).
    static mut curwin: *mut crate::display::WinHandle;
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
}

// C accessor functions for make_popup.
extern "C" {
    /// Check if UI has a capability.
    fn ui_has(what: c_int) -> bool;
    /// Set `mouse_grid`.
    fn nvim_set_mouse_grid(val: c_int);
    /// Get `mouse_row`.
    fn nvim_get_mouse_row() -> c_int;
    /// Set `mouse_row`.
    fn nvim_set_mouse_row(val: c_int);
    /// Get `mouse_col`.
    fn nvim_get_mouse_col() -> c_int;
    /// Set `mouse_col`.
    fn nvim_set_mouse_col(val: c_int);
    /// Batch curwin geometry accessor.
    fn nvim_pum_get_curwin_geometry() -> PumCurwinGeometry;
    /// Find menu by path name (returns NULL if not found).
    fn nvim_pum_menu_find(path_name: *const std::ffi::c_char) -> *mut VimMenuHandle;
}

/// UI capability for multigrid mode (kUIMultigrid = 6).
const K_UI_MULTIGRID: c_int = 6;

// C accessor functions for show_popupmenu.
extern "C" {
    /// Get menu mode flag.
    fn nvim_pum_get_menu_mode_flag() -> c_int;
    /// Check if menu item is a separator.
    fn nvim_pum_menu_is_separator(menu: *mut VimMenuHandle) -> c_int;
    /// Get menu item display name.
    fn nvim_pum_menu_get_dname(menu: *mut VimMenuHandle) -> *const std::ffi::c_char;
    /// Allocate zeroed memory.
    fn xcalloc(count: usize, size: usize) -> *mut std::ffi::c_void;
    /// Duplicate a C string.
    fn xstrdup(s: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    /// Free memory.
    fn xfree(ptr: *mut std::ffi::c_void);
    /// Compute item widths and write to `PUM_STATE` (Rust function via extern "C").
    fn rs_pum_compute_size(array: *const crate::item::PumItemArray);
    /// Get `w_p_rl` for a window.
    fn nvim_win_get_w_p_rl(wp: *mut crate::display::WinHandle) -> c_int;
    /// Position popup at mouse.
    fn rs_pum_position_at_mouse(min_width: c_int);
    /// Set mousemoveevent UI option.
    fn nvim_pum_ui_set_mousemoveevent(val: c_int);
    /// Set `pum_grid.zindex` to `kZIndexCmdlinePopupMenu`.
    fn nvim_pum_grid_set_zindex_cmdline();
    /// Call `setcursor_mayforce(wp, force)`.
    fn setcursor_mayforce(wp: *mut crate::display::WinHandle, force: bool);
    /// Call `vgetc()`.
    fn vgetc() -> c_int;
    /// Call `vungetc(c)`.
    fn vungetc(c: c_int);
    /// Select popup entry at mouse position.
    fn rs_pum_select_mouse_pos();
    /// Emit error for wrong menu mode.
    fn nvim_pum_emsg_menu_mode();
    /// Batch key constants accessor.
    fn nvim_pum_get_key_constants() -> PumKeyConstants;
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
    let pum_selected = PUM_STATE.selected;
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

/// Open the terminal version of the popup menu and run its event loop.
///
/// Builds a `pumitem_T` array from the menu's children, displays the popup,
/// and enters a blocking `vgetc()` loop to handle keyboard and mouse input.
/// The popup is closed when the user selects an item, presses Esc, or
/// right-clicks to reposition.
///
/// # Safety
/// Calls C accessor functions. `menu` must be a valid `vimmenu_T` pointer.
#[export_name = "pum_show_popupmenu"]
#[allow(clippy::too_many_lines, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_pum_show_popupmenu(menu: *mut VimMenuHandle) {
    crate::display::rs_pum_undisplay(1);
    PUM_STATE.size = 0;
    let mode = nvim_pum_get_menu_mode_flag();

    // Count matching menu items
    let mut count = 0;
    let mut mp = nvim_pum_menu_children(menu);
    while !mp.is_null() {
        if nvim_pum_menu_is_separator(mp) != 0 || nvim_pum_menu_matches_mode(mp, mode) != 0 {
            count += 1;
        }
        mp = nvim_pum_menu_next(mp);
    }
    PUM_STATE.size = count;

    if count <= 0 {
        nvim_pum_emsg_menu_mode();
        return;
    }

    // Build pumitem_T array from menu children
    let array = xcalloc(
        count as usize,
        std::mem::size_of::<crate::item::PumItemArray>(),
    )
    .cast::<crate::item::PumItemArray>();
    let mut idx = 0;
    mp = nvim_pum_menu_children(menu);
    while !mp.is_null() {
        let is_sep = nvim_pum_menu_is_separator(mp) != 0;
        let matches_mode = nvim_pum_menu_matches_mode(mp, mode) != 0;
        if is_sep || matches_mode {
            let text = if is_sep {
                c"".as_ptr()
            } else {
                nvim_pum_menu_get_dname(mp)
            };
            (*array.offset(idx as isize)).pum_text = xstrdup(text);
            idx += 1;
        }
        mp = nvim_pum_menu_next(mp);
    }

    PUM_STATE.array = array;
    rs_pum_compute_size(PUM_STATE.array);
    PUM_STATE.scrollbar = 0;
    PUM_STATE.height = count;
    PUM_STATE.rl = nvim_win_get_w_p_rl(curwin);
    rs_pum_position_at_mouse(20);

    PUM_STATE.selected = -1;
    PUM_STATE.first = 0;
    let mousemev_was_off = p_mousemev == 0;
    if mousemev_was_off {
        nvim_pum_ui_set_mousemoveevent(1);
    }

    // Cache key constants via batch accessor
    let keys = nvim_pum_get_key_constants();
    let key_esc = keys.key_esc;
    let key_ctrl_c = keys.key_ctrl_c;
    let key_car = keys.key_car;
    let key_nl = keys.key_nl;
    let key_k_up = keys.key_k_up;
    let key_k_down = keys.key_k_down;
    let key_k_mouseup = keys.key_k_mouseup;
    let key_k_mousedown = keys.key_k_mousedown;
    let key_k_rightmouse = keys.key_k_rightmouse;
    let key_k_leftdrag = keys.key_k_leftdrag;
    let key_k_rightdrag = keys.key_k_rightdrag;
    let key_k_mousemove = keys.key_k_mousemove;
    let key_k_leftmouse = keys.key_k_leftmouse;
    let key_k_leftmouse_nm = keys.key_k_leftmouse_nm;
    let key_k_rightrelease = keys.key_k_rightrelease;

    loop {
        PUM_STATE.is_visible = 1;
        PUM_STATE.is_drawn = 1;
        nvim_pum_grid_set_zindex_cmdline();
        crate::redraw::rs_pum_redraw();
        setcursor_mayforce(curwin, true);

        let c = vgetc();

        // Bail out on Esc, Ctrl-C, or if a callback cleared pum_array
        if c == key_esc || c == key_ctrl_c || PUM_STATE.array.is_null() {
            break;
        } else if c == key_car || c == key_nl {
            // Enter: select current item and close
            rs_pum_execute_menu(menu, mode);
            break;
        } else if c == i32::from(b'k') || c == key_k_up || c == key_k_mouseup {
            // Cursor up: select previous item
            let mut sel = PUM_STATE.selected;
            while sel > 0 {
                sel -= 1;
                if !(*PUM_STATE.array.offset(sel as isize)).pum_text.is_null()
                    && *(*PUM_STATE.array.offset(sel as isize)).pum_text != 0
                {
                    break;
                }
            }
            PUM_STATE.selected = sel;
        } else if c == i32::from(b'j') || c == key_k_down || c == key_k_mousedown {
            // Cursor down: select next item
            let pum_size = PUM_STATE.size;
            let mut sel = PUM_STATE.selected;
            while sel < pum_size - 1 {
                sel += 1;
                if !(*PUM_STATE.array.offset(sel as isize)).pum_text.is_null()
                    && *(*PUM_STATE.array.offset(sel as isize)).pum_text != 0
                {
                    break;
                }
            }
            PUM_STATE.selected = sel;
        } else if c == key_k_rightmouse {
            // Right mouse: reposition the menu
            vungetc(c);
            break;
        } else if c == key_k_leftdrag || c == key_k_rightdrag || c == key_k_mousemove {
            // Mouse moved: select item at mouse position
            rs_pum_select_mouse_pos();
        } else if c == key_k_leftmouse || c == key_k_leftmouse_nm || c == key_k_rightrelease {
            // Mouse click: select and maybe close
            rs_pum_select_mouse_pos();
            if PUM_STATE.selected >= 0 {
                rs_pum_execute_menu(menu, mode);
                break;
            }
            if c == key_k_leftmouse || c == key_k_leftmouse_nm {
                break;
            }
        }
    }

    // Free each item's text, then the array itself
    for i in 0..count as isize {
        xfree((*array.offset(i)).pum_text.cast::<std::ffi::c_void>());
    }
    xfree(array.cast::<std::ffi::c_void>());
    crate::display::rs_pum_undisplay(1);
    if mousemev_was_off {
        nvim_pum_ui_set_mousemoveevent(0);
    }
}

/// Create a popup from a menu path.
///
/// If `use_mouse_pos` is false, sets the mouse position to the cursor
/// location so the menu appears near the cursor. Then finds the menu
/// by path name and shows it as a popup.
///
/// # Safety
/// Calls C accessor functions. `path_name` must be a valid C string.
#[export_name = "pum_make_popup"]
pub unsafe extern "C" fn rs_pum_make_popup(
    path_name: *const std::ffi::c_char,
    use_mouse_pos: c_int,
) {
    if use_mouse_pos == 0 {
        // Set mouse position at the cursor so the menu pops up there.
        let cw = nvim_pum_get_curwin_geometry();
        let p_rl = cw.p_rl != 0;

        nvim_set_mouse_row(cw.row_offset + cw.wrow);
        nvim_set_mouse_col(
            cw.col_offset
                + if p_rl {
                    cw.view_width - cw.wcol - 1
                } else {
                    cw.wcol
                },
        );

        if ui_has(K_UI_MULTIGRID) {
            nvim_set_mouse_grid(cw.grid_target_handle);
        } else if cw.grid_target_is_default == 0 {
            nvim_set_mouse_grid(0);
            nvim_set_mouse_row(nvim_get_mouse_row() + cw.winrow);
            nvim_set_mouse_col(nvim_get_mouse_col() + cw.wincol);
        }
    }

    let menu = nvim_pum_menu_find(path_name);
    if !menu.is_null() {
        rs_pum_show_popupmenu(menu);
    }
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
