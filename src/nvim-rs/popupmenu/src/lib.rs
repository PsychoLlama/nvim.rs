//! Popup menu state queries for Neovim
//!
//! This crate provides Rust implementations of popup menu functions
//! from `src/nvim/popupmenu.c`.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::c_int;

// C accessor functions for popup menu state.
extern "C" {
    /// Get the `pum_is_visible` static variable.
    fn nvim_get_pum_is_visible() -> c_int;
    /// Get the `pum_external` static variable.
    fn nvim_get_pum_external() -> c_int;
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

#[cfg(test)]
mod tests {
    // Note: Tests would need to mock the C accessor functions
}
