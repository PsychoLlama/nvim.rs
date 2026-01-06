//! Popup menu state queries for Neovim
//!
//! This crate provides Rust implementations of popup menu functions
//! from `src/nvim/popupmenu.c`.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::c_int;

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

#[cfg(test)]
mod tests {
    // Note: Tests would need to mock the C accessor functions
}
