//! TUI output functions
//!
//! This module contains functions for terminal output, including attribute
//! comparison and ANSI escape sequence generation.

use nvim_highlight::hl_attr_flags::HL_UNDERLINE_MASK;
use nvim_highlight::HlAttrs;
use std::ffi::c_int;

// ============================================================================
// Attribute Comparison
// ============================================================================

/// Check if two attribute IDs have different visual attributes.
///
/// This function compares two highlight attribute entries to determine if they
/// would produce different visual output. It's used to optimize terminal output
/// by avoiding redundant attribute changes.
///
/// # Arguments
///
/// * `id1` - First attribute ID
/// * `id2` - Second attribute ID
/// * `rgb` - Whether we're in RGB (truecolor) mode
/// * `attrs` - Pointer to the HlAttrs array
/// * `attrs_size` - Size of the attrs array
///
/// # Safety
///
/// - `attrs` must be a valid pointer to an array of at least `attrs_size` HlAttrs
/// - The caller must ensure the array remains valid for the duration of the call
#[no_mangle]
pub unsafe extern "C" fn rs_attrs_differ(
    id1: c_int,
    id2: c_int,
    rgb: bool,
    attrs: *const HlAttrs,
    attrs_size: usize,
) -> bool {
    attrs_differ_impl(id1, id2, rgb, attrs, attrs_size)
}

/// Implementation of attrs_differ that can be tested
unsafe fn attrs_differ_impl(
    id1: c_int,
    id2: c_int,
    rgb: bool,
    attrs: *const HlAttrs,
    attrs_size: usize,
) -> bool {
    // Same ID means same attributes
    if id1 == id2 {
        return false;
    }

    // Negative IDs indicate special/missing attributes - always differ
    if id1 < 0 || id2 < 0 {
        return true;
    }

    let idx1 = id1 as usize;
    let idx2 = id2 as usize;

    // Bounds check
    if idx1 >= attrs_size || idx2 >= attrs_size {
        return true;
    }

    let a1 = *attrs.add(idx1);
    let a2 = *attrs.add(idx2);

    // URL always matters
    if a1.url != a2.url {
        return true;
    }

    if rgb {
        // RGB mode: compare RGB colors and attributes
        a1.rgb_fg_color != a2.rgb_fg_color
            || a1.rgb_bg_color != a2.rgb_bg_color
            || a1.rgb_ae_attr != a2.rgb_ae_attr
            || a1.rgb_sp_color != a2.rgb_sp_color
    } else {
        // cterm mode: compare cterm colors and attributes
        // Also check sp_color for underline styles
        a1.cterm_fg_color != a2.cterm_fg_color
            || a1.cterm_bg_color != a2.cterm_bg_color
            || a1.cterm_ae_attr != a2.cterm_ae_attr
            || (a1.cterm_ae_attr & HL_UNDERLINE_MASK != 0 && a1.rgb_sp_color != a2.rgb_sp_color)
    }
}

// ============================================================================
// Grid Cursor Position
// ============================================================================

/// Opaque handle to TUIData struct in C
#[repr(C)]
pub struct TuiHandle {
    _private: [u8; 0],
}

extern "C" {
    fn nvim_tui_set_row(tui: *mut TuiHandle, row: c_int);
    fn nvim_tui_set_col(tui: *mut TuiHandle, col: c_int);
    fn nvim_tui_set_attrs(tui: *mut TuiHandle, idx: usize, attrs: HlAttrs);
    fn nvim_tui_set_clear_attrs(tui: *mut TuiHandle, attrs: HlAttrs);
    fn nvim_tui_set_print_attr_id(tui: *mut TuiHandle, id: c_int);
    fn nvim_tui_set_default_colors_flag(tui: *mut TuiHandle, value: bool);
    fn nvim_tui_get_grid_height(tui: *mut TuiHandle) -> c_int;
    fn nvim_tui_get_grid_width(tui: *mut TuiHandle) -> c_int;
    fn nvim_tui_invalidate(tui: *mut TuiHandle, top: c_int, bot: c_int, left: c_int, right: c_int);
}

/// Set cursor position for the grid.
///
/// This function stores the cursor row and column position in the TUIData struct.
/// The actual cursor movement happens during tui_flush.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_grid_cursor_goto(tui: *mut TuiHandle, row: i64, col: i64) {
    if tui.is_null() {
        return;
    }

    // cursor position is validated in tui_flush
    nvim_tui_set_row(tui, row as c_int);
    nvim_tui_set_col(tui, col as c_int);
}

// ============================================================================
// Highlight Attribute Definition
// ============================================================================

/// Store highlight attributes in the TUI attributes array.
///
/// This function merges RGB and cterm attributes and stores them at the
/// specified index in the TUI's highlight attribute array.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_hl_attr_define(
    tui: *mut TuiHandle,
    id: i64,
    mut attrs: HlAttrs,
    cterm_attrs: HlAttrs,
) {
    if tui.is_null() {
        return;
    }

    // Merge cterm attributes into the main attrs struct
    attrs.cterm_ae_attr = cterm_attrs.cterm_ae_attr;
    attrs.cterm_fg_color = cterm_attrs.cterm_fg_color;
    attrs.cterm_bg_color = cterm_attrs.cterm_bg_color;

    nvim_tui_set_attrs(tui, id as usize, attrs);
}

// ============================================================================
// Default Colors
// ============================================================================

/// Set default colors and invalidate the entire grid.
///
/// This function sets the clear_attrs used for background clearing,
/// resets print_attr_id to force attribute re-emission, and invalidates
/// the entire grid so it will be redrawn with the new colors.
///
/// # Safety
///
/// - `tui` must be a valid pointer to a TUIData struct
#[no_mangle]
pub unsafe extern "C" fn rs_tui_default_colors_set(
    tui: *mut TuiHandle,
    rgb_fg: i64,
    rgb_bg: i64,
    rgb_sp: i64,
    cterm_fg: i64,
    cterm_bg: i64,
) {
    if tui.is_null() {
        return;
    }

    // Build the clear_attrs struct
    let clear_attrs = HlAttrs {
        rgb_ae_attr: 0,
        cterm_ae_attr: 0,
        rgb_fg_color: rgb_fg as i32,
        rgb_bg_color: rgb_bg as i32,
        rgb_sp_color: rgb_sp as i32,
        cterm_fg_color: cterm_fg as i16,
        cterm_bg_color: cterm_bg as i16,
        hl_blend: -1,
        url: -1,
    };

    nvim_tui_set_clear_attrs(tui, clear_attrs);
    nvim_tui_set_print_attr_id(tui, -1);
    nvim_tui_set_default_colors_flag(tui, true);

    // Invalidate entire grid
    let height = nvim_tui_get_grid_height(tui);
    let width = nvim_tui_get_grid_width(tui);
    nvim_tui_invalidate(tui, 0, height, 0, width);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_attrs(count: usize) -> Vec<HlAttrs> {
        vec![HlAttrs::new(); count]
    }

    #[test]
    fn test_same_id() {
        let attrs = make_attrs(5);
        unsafe {
            assert!(!attrs_differ_impl(2, 2, true, attrs.as_ptr(), attrs.len()));
            assert!(!attrs_differ_impl(2, 2, false, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_negative_id() {
        let attrs = make_attrs(5);
        unsafe {
            assert!(attrs_differ_impl(-1, 2, true, attrs.as_ptr(), attrs.len()));
            assert!(attrs_differ_impl(2, -1, true, attrs.as_ptr(), attrs.len()));
            assert!(attrs_differ_impl(-1, -1, true, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_out_of_bounds() {
        let attrs = make_attrs(5);
        unsafe {
            assert!(attrs_differ_impl(10, 2, true, attrs.as_ptr(), attrs.len()));
            assert!(attrs_differ_impl(2, 10, true, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_identical_attrs() {
        let attrs = make_attrs(5);
        unsafe {
            // All default attrs are the same
            assert!(!attrs_differ_impl(0, 1, true, attrs.as_ptr(), attrs.len()));
            assert!(!attrs_differ_impl(0, 1, false, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_different_rgb_fg() {
        let mut attrs = make_attrs(5);
        attrs[0].rgb_fg_color = 0xFF0000; // red
        attrs[1].rgb_fg_color = 0x00FF00; // green
        unsafe {
            assert!(attrs_differ_impl(0, 1, true, attrs.as_ptr(), attrs.len()));
            // In cterm mode, RGB colors don't matter
            assert!(!attrs_differ_impl(0, 1, false, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_different_cterm_fg() {
        let mut attrs = make_attrs(5);
        attrs[0].cterm_fg_color = 1;
        attrs[1].cterm_fg_color = 2;
        unsafe {
            // In RGB mode, cterm colors don't matter
            assert!(!attrs_differ_impl(0, 1, true, attrs.as_ptr(), attrs.len()));
            assert!(attrs_differ_impl(0, 1, false, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_different_url() {
        let mut attrs = make_attrs(5);
        attrs[0].url = 0;
        attrs[1].url = 1;
        unsafe {
            // URL always matters regardless of RGB mode
            assert!(attrs_differ_impl(0, 1, true, attrs.as_ptr(), attrs.len()));
            assert!(attrs_differ_impl(0, 1, false, attrs.as_ptr(), attrs.len()));
        }
    }

    #[test]
    fn test_underline_sp_color() {
        let mut attrs = make_attrs(5);
        // Set underline attribute and different sp_color
        attrs[0].cterm_ae_attr = HL_UNDERLINE_MASK;
        attrs[0].rgb_sp_color = 0xFF0000;
        attrs[1].cterm_ae_attr = HL_UNDERLINE_MASK;
        attrs[1].rgb_sp_color = 0x00FF00;
        unsafe {
            // In cterm mode with underline, sp_color matters
            assert!(attrs_differ_impl(0, 1, false, attrs.as_ptr(), attrs.len()));
        }

        // Without underline, sp_color doesn't matter in cterm mode
        attrs[0].cterm_ae_attr = 0;
        attrs[1].cterm_ae_attr = 0;
        unsafe {
            assert!(!attrs_differ_impl(0, 1, false, attrs.as_ptr(), attrs.len()));
        }
    }
}
