//! Popup menu display orchestration.
//!
//! This module provides helper functions for showing, hiding, and managing
//! the popup menu display state.

use std::ffi::c_int;

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

/// UI capability for popup menu.
const K_UI_POPUPMENU: c_int = 8;
/// UI capability for wildmenu.
const K_UI_WILDMENU: c_int = 12;
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

// C _impl functions for Phase 3/5/8 migration.
extern "C" {
    /// Display the popup menu (implementation).
    fn nvim_pum_display_impl(
        array: *mut crate::item::PumItemArray,
        size: c_int,
        selected: c_int,
        array_changed: c_int,
        cmd_startcol: c_int,
    );
    /// Recompose the popup menu grid.
    fn nvim_pum_recompose_impl();
    /// Check and clear the popup menu display.
    fn nvim_pum_check_clear_impl();
    /// Flush the popup menu UI position.
    fn nvim_pum_ui_flush_impl();
    /// Set preview text in a buffer.
    fn nvim_pum_preview_set_text_impl(
        buf: *mut BufHandle,
        info: *mut std::ffi::c_char,
        lnum: *mut i32,
        max_width: *mut c_int,
    );
    /// Adjust floating info preview window position.
    fn nvim_pum_adjust_info_position_impl(wp: *mut WinHandle, width: c_int);
    /// Set info for a completed item.
    fn nvim_pum_set_info_impl(selected: c_int, info: *mut std::ffi::c_char) -> *mut WinHandle;
    /// Set the selected item index (scrolling, preview).
    fn nvim_pum_set_selected_impl(n: c_int, repeat: c_int) -> c_int;
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
/// # Safety
/// Calls C `_impl` function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_recompose() {
    nvim_pum_recompose_impl();
}

/// Check and clear the popup menu display if needed.
///
/// # Safety
/// Calls C `_impl` function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_check_clear() {
    nvim_pum_check_clear_impl();
}

/// Flush the popup menu UI position in multigrid mode.
///
/// # Safety
/// Calls C `_impl` function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_ui_flush() {
    nvim_pum_ui_flush_impl();
}

/// Set the informational text in the preview buffer.
///
/// # Safety
/// Calls C `_impl` function. All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_preview_set_text(
    buf: *mut BufHandle,
    info: *mut std::ffi::c_char,
    lnum: *mut i32,
    max_width: *mut c_int,
) {
    nvim_pum_preview_set_text_impl(buf, info, lnum, max_width);
}

/// Adjust floating info preview window position.
///
/// # Safety
/// Calls C `_impl` function. `wp` must be a valid `win_T` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_adjust_info_position(wp: *mut WinHandle, width: c_int) {
    nvim_pum_adjust_info_position_impl(wp, width);
}

/// Set info for a completed item, returning a window pointer.
///
/// # Safety
/// Calls C `_impl` function. `info` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_set_info(
    selected: c_int,
    info: *mut std::ffi::c_char,
) -> *mut WinHandle {
    nvim_pum_set_info_impl(selected, info)
}

/// Set the selected item index, handle scrolling and preview.
///
/// Returns 1 if the window was resized and repositioning is needed.
///
/// # Safety
/// Calls C `_impl` function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_set_selected(n: c_int, repeat: c_int) -> c_int {
    nvim_pum_set_selected_impl(n, repeat)
}

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
