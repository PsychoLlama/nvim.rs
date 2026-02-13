//! Popup menu event information.
//!
//! This module provides helper functions for getting popup menu
//! position and size information for event dictionaries.

use std::ffi::c_int;

// C accessor functions for popup state.
extern "C" {
    /// Get the `pum_row` static variable.
    fn nvim_get_pum_row() -> c_int;
    /// Get the `pum_col` static variable.
    fn nvim_get_pum_col() -> c_int;
    /// Get the `pum_width` static variable.
    fn nvim_get_pum_width() -> c_int;
    /// Get the `pum_height` static variable.
    fn nvim_get_pum_height() -> c_int;
    /// Get the `pum_size` static variable.
    fn nvim_get_pum_size() -> c_int;
    /// Get the `pum_scrollbar` static variable.
    fn nvim_get_pum_scrollbar() -> c_int;
    /// Get the `pum_is_visible` static variable.
    fn nvim_get_pum_is_visible() -> c_int;
}

/// Popup menu position and size information.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumEventInfo {
    /// Whether the popup menu is visible.
    pub visible: c_int,
    /// Width of the popup menu.
    pub width: c_int,
    /// Height of the popup menu.
    pub height: c_int,
    /// Row position.
    pub row: c_int,
    /// Column position.
    pub col: c_int,
    /// Total number of items.
    pub size: c_int,
    /// Whether scrollbar is visible.
    pub scrollbar: c_int,
}

/// Get popup menu event information.
///
/// Returns struct with position and size info for event dictionary.
/// If the popup is not visible, returns struct with `visible = 0`.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_get_event_info() -> PumEventInfo {
    if nvim_get_pum_is_visible() == 0 {
        return PumEventInfo {
            visible: 0,
            width: 0,
            height: 0,
            row: 0,
            col: 0,
            size: 0,
            scrollbar: 0,
        };
    }

    PumEventInfo {
        visible: 1,
        width: nvim_get_pum_width(),
        height: nvim_get_pum_height(),
        row: nvim_get_pum_row(),
        col: nvim_get_pum_col(),
        size: nvim_get_pum_size(),
        scrollbar: nvim_get_pum_scrollbar(),
    }
}

/// Check if the popup menu is visible.
///
/// Returns 1 if visible, 0 otherwise.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_event_visible() -> c_int {
    nvim_get_pum_is_visible()
}

/// External UI item selection request.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumWant {
    /// Whether there is an active selection request.
    pub active: c_int,
    /// Item index to select (-1 for none).
    pub item: c_int,
    /// Whether to insert the item.
    pub insert: c_int,
    /// Whether to finish completion.
    pub finish: c_int,
}

// C accessor functions for pum_want.
extern "C" {
    /// Get `pum_want.active`.
    fn nvim_get_pum_want_active() -> c_int;
    /// Set `pum_want.active`.
    fn nvim_set_pum_want_active(val: c_int);
    /// Get `pum_want.item`.
    fn nvim_get_pum_want_item() -> c_int;
    /// Set `pum_want.item`.
    fn nvim_set_pum_want_item(val: c_int);
    /// Get `pum_want.insert`.
    fn nvim_get_pum_want_insert() -> c_int;
    /// Set `pum_want.insert`.
    fn nvim_set_pum_want_insert(val: c_int);
    /// Get `pum_want.finish`.
    fn nvim_get_pum_want_finish() -> c_int;
    /// Set `pum_want.finish`.
    fn nvim_set_pum_want_finish(val: c_int);
}

/// Get the current external UI selection request.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_get_want() -> PumWant {
    PumWant {
        active: nvim_get_pum_want_active(),
        item: nvim_get_pum_want_item(),
        insert: nvim_get_pum_want_insert(),
        finish: nvim_get_pum_want_finish(),
    }
}

/// Set the external UI selection request.
///
/// # Arguments
/// * `item` - Item index to select (-1 for none)
/// * `insert` - Whether to insert the item
/// * `finish` - Whether to finish completion
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_set_want(item: c_int, insert: c_int, finish: c_int) {
    nvim_set_pum_want_active(1);
    nvim_set_pum_want_item(item);
    nvim_set_pum_want_insert(insert);
    nvim_set_pum_want_finish(finish);
}

/// Clear the external UI selection request.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_clear_want() {
    nvim_set_pum_want_active(0);
}

/// Check if there is an active external UI selection request.
///
/// Returns 1 if active, 0 otherwise.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_has_want() -> c_int {
    nvim_get_pum_want_active()
}

/// Opaque handle to a `dict_T`.
#[repr(C)]
pub struct DictHandle {
    _private: [u8; 0],
}

// C _impl function for Phase 3 migration.
extern "C" {
    /// Set event info into a `dict_T`.
    fn nvim_pum_set_event_info_impl(dict: *mut DictHandle);
}

/// Add size information about the popup menu to the given dictionary.
///
/// # Safety
/// Calls C `_impl` function. `dict` must be a valid `dict_T` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_set_event_info(dict: *mut DictHandle) {
    nvim_pum_set_event_info_impl(dict);
}

#[cfg(test)]
mod tests {
    // Tests for pure functions would go here
    // The unsafe functions need C environment
}
