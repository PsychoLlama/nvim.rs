//! Popup menu event information.
//!
//! This module provides helper functions for getting popup menu
//! position and size information for event dictionaries.

use std::ffi::{c_char, c_int};

use crate::PUM_STATE;

// Direct C functions for event info dict population.
extern "C" {
    /// Get UI-provided popup position.
    fn ui_pum_get_pos(pwidth: *mut f64, pheight: *mut f64, prow: *mut f64, pcol: *mut f64) -> bool;
    /// Add a float value to a `dict_T`.
    fn tv_dict_add_float(
        dict: *mut DictHandle,
        key: *const c_char,
        key_len: usize,
        val: f64,
    ) -> c_int;
    /// Add an integer value to a `dict_T` (`varnumber_T` = `int64_t`).
    fn tv_dict_add_nr(dict: *mut DictHandle, key: *const c_char, key_len: usize, val: i64)
        -> c_int;
    /// Add a boolean value to a `dict_T` (`BoolVarValue`: 0 = false, 1 = true).
    fn tv_dict_add_bool(
        dict: *mut DictHandle,
        key: *const c_char,
        key_len: usize,
        val: c_int,
    ) -> c_int;
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
    if PUM_STATE.is_visible == 0 {
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
        width: PUM_STATE.width,
        height: PUM_STATE.height,
        row: PUM_STATE.row,
        col: PUM_STATE.col,
        size: PUM_STATE.size,
        scrollbar: PUM_STATE.scrollbar,
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
    PUM_STATE.is_visible
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

use crate::pum_want;

/// Get the current external UI selection request.
///
/// # Safety
/// Reads `pum_want` global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_get_want() -> PumWant {
    PumWant {
        active: c_int::from(pum_want.active),
        item: pum_want.item,
        insert: c_int::from(pum_want.insert),
        finish: c_int::from(pum_want.finish),
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
/// Writes `pum_want` global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_set_want(item: c_int, insert: c_int, finish: c_int) {
    pum_want.active = 1;
    pum_want.item = item;
    pum_want.insert = (insert != 0) as u8;
    pum_want.finish = (finish != 0) as u8;
}

/// Clear the external UI selection request.
///
/// # Safety
/// Writes `pum_want` global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_clear_want() {
    pum_want.active = 0;
}

/// Check if there is an active external UI selection request.
///
/// Returns 1 if active, 0 otherwise.
///
/// # Safety
/// Reads `pum_want` global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_has_want() -> c_int {
    c_int::from(pum_want.active)
}

/// Opaque handle to a `dict_T`.
#[repr(C)]
pub struct DictHandle {
    _private: [u8; 0],
}

/// Add size information about the popup menu to the given dictionary.
///
/// Populates the dictionary with height, width, row, col (as floats from
/// the UI if available, otherwise from internal state), size (integer),
/// and scrollbar (boolean).
///
/// # Safety
/// Calls C accessor functions. `dict` must be a valid `dict_T` pointer.
#[export_name = "pum_set_event_info"]
pub unsafe extern "C" fn rs_pum_set_event_info(dict: *mut DictHandle) {
    if PUM_STATE.is_visible == 0 {
        return;
    }

    // Try to get position from the UI; fall back to internal state.
    let mut w: f64 = 0.0;
    let mut h: f64 = 0.0;
    let mut r: f64 = 0.0;
    let mut c: f64 = 0.0;
    if !ui_pum_get_pos(&raw mut w, &raw mut h, &raw mut r, &raw mut c) {
        w = f64::from(PUM_STATE.width);
        h = f64::from(PUM_STATE.height);
        r = f64::from(PUM_STATE.row);
        c = f64::from(PUM_STATE.col);
    }

    tv_dict_add_float(dict, c"height".as_ptr(), 6, h);
    tv_dict_add_float(dict, c"width".as_ptr(), 5, w);
    tv_dict_add_float(dict, c"row".as_ptr(), 3, r);
    tv_dict_add_float(dict, c"col".as_ptr(), 3, c);
    tv_dict_add_nr(dict, c"size".as_ptr(), 4, i64::from(PUM_STATE.size));
    // kBoolVarFalse=0, kBoolVarTrue=1
    tv_dict_add_bool(dict, c"scrollbar".as_ptr(), 9, PUM_STATE.scrollbar);
}

#[cfg(test)]
mod tests {
    // Tests for pure functions would go here
    // The unsafe functions need C environment
}
