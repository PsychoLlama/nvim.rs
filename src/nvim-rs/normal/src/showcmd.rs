//! Showcmd display routines for Normal mode.
//!
//! This module provides the Rust implementation of `clear_showcmd()` from
//! `src/nvim/normal.c`. The complex Visual mode character counting and
//! formatting is delegated to a C helper function.

use std::ffi::c_int;

extern "C" {
    fn nvim_get_p_sc() -> c_int;
    fn nvim_get_showcmd_is_clear() -> bool;
    fn nvim_set_showcmd_visual(val: bool);
    fn nvim_normal_showcmd_buf_ptr() -> *mut std::ffi::c_char;
    fn nvim_normal_display_showcmd();
    fn nvim_clear_showcmd_visual_info() -> bool;
}

/// Clear the showcmd display area.
///
/// In Visual mode, computes and displays the size of the visual selection
/// (delegated to C). Otherwise, clears the showcmd buffer and updates
/// the display.
///
/// This is the Rust implementation of `clear_showcmd()` from normal.c.
#[no_mangle]
pub extern "C" fn rs_clear_showcmd() {
    unsafe {
        if nvim_get_p_sc() == 0 {
            return;
        }

        if nvim_clear_showcmd_visual_info() {
            // Visual info was computed and written into showcmd_buf.
            nvim_set_showcmd_visual(true);
        } else {
            // Not in Visual mode or char_avail() returned true.
            let buf = nvim_normal_showcmd_buf_ptr();
            *buf = 0; // NUL
            nvim_set_showcmd_visual(false);

            // Don't actually display something if there is nothing to clear.
            if nvim_get_showcmd_is_clear() {
                return;
            }
        }

        nvim_normal_display_showcmd();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    #[test]
    fn test_showcmd_constants() {
        // SHOWCMD_COLS = 10, SHOWCMD_BUFLEN = 10 + 1 + 30 = 41
        // These are verified with _Static_assert in C, just document here
        assert_eq!(10 + 1 + 30, 41);
    }
}
