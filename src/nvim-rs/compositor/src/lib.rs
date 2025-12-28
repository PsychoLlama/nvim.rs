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

/// Type alias for screen attribute (matches C's `sattr_T` which is `int16_t`).
pub type SattrT = i16;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screengrid_handle_null() {
        let null_handle = ScreenGridHandle(std::ptr::null_mut());
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
}
