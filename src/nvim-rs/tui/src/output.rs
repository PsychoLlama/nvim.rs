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
