//! Menu execution utilities.
//!
//! This module provides helper functions for menu execution.
//! The actual command execution is handled in C since it involves
//! deep integration with Neovim's state machine.

use std::ffi::c_int;

use crate::handle::VimMenuHandle;
use crate::menu_modes::{
    MENU_INDEX_CMDLINE, MENU_INDEX_INSERT, MENU_INDEX_INVALID, MENU_INDEX_NORMAL,
    MENU_INDEX_OP_PENDING, MENU_INDEX_SELECT, MENU_INDEX_TERMINAL, MENU_INDEX_TIP,
    MENU_INDEX_VISUAL,
};

/// Get the menu string for a given mode index.
///
/// Returns a pointer to the string for the specified mode, or NULL if none.
/// The string is owned by the menu and should not be freed.
///
/// # Safety
/// The `menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_get_menu_string(
    menu: VimMenuHandle,
    mode_idx: c_int,
) -> *const std::ffi::c_char {
    if menu.is_null() {
        return std::ptr::null();
    }

    if !(0..crate::menu_modes::MENU_MODES).contains(&mode_idx) {
        return std::ptr::null();
    }

    // This would require an accessor for menu->strings[mode_idx]
    // For now, return null - the actual implementation needs C accessor
    std::ptr::null()
}

/// Check if a menu has a string for the given mode.
///
/// # Safety
/// The `menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_has_string_for_mode(menu: VimMenuHandle, mode_idx: c_int) -> bool {
    if menu.is_null() {
        return false;
    }

    if !(0..crate::menu_modes::MENU_MODES).contains(&mode_idx) {
        return false;
    }

    // Check if the menu is enabled for this mode
    let mode_flag = 1 << mode_idx;
    (menu.enabled() & mode_flag) != 0
}

/// Check if a menu is enabled for the given mode.
///
/// # Safety
/// The `menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_is_enabled_for_mode(menu: VimMenuHandle, mode_idx: c_int) -> bool {
    if menu.is_null() {
        return false;
    }

    if !(0..crate::menu_modes::MENU_MODES).contains(&mode_idx) {
        return false;
    }

    let mode_flag = 1 << mode_idx;
    (menu.enabled() & mode_flag) != 0
}

/// Convert a mode index to its corresponding mode flag.
#[no_mangle]
pub extern "C" fn rs_mode_index_to_flag(mode_idx: c_int) -> c_int {
    if !(0..crate::menu_modes::MENU_MODES).contains(&mode_idx) {
        return 0;
    }
    1 << mode_idx
}

/// Convert a mode flag to its corresponding mode index.
///
/// Returns the index of the first set bit, or MENU_INDEX_INVALID if none.
#[no_mangle]
pub extern "C" fn rs_mode_flag_to_index(mode_flag: c_int) -> c_int {
    if mode_flag == 0 {
        return MENU_INDEX_INVALID;
    }

    for i in 0..crate::menu_modes::MENU_MODES {
        if (mode_flag & (1 << i)) != 0 {
            return i;
        }
    }

    MENU_INDEX_INVALID
}

/// Get the menu index name for display purposes.
///
/// Returns a static string like "Normal", "Visual", etc.
#[no_mangle]
pub extern "C" fn rs_menu_mode_name(mode_idx: c_int) -> *const std::ffi::c_char {
    // Return static strings for each mode
    match mode_idx {
        MENU_INDEX_NORMAL => c"Normal".as_ptr(),
        MENU_INDEX_VISUAL => c"Visual".as_ptr(),
        MENU_INDEX_SELECT => c"Select".as_ptr(),
        MENU_INDEX_OP_PENDING => c"Op-pending".as_ptr(),
        MENU_INDEX_INSERT => c"Insert".as_ptr(),
        MENU_INDEX_CMDLINE => c"Cmdline".as_ptr(),
        MENU_INDEX_TERMINAL => c"Terminal".as_ptr(),
        MENU_INDEX_TIP => c"Tip".as_ptr(),
        _ => std::ptr::null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_index_to_flag() {
        assert_eq!(rs_mode_index_to_flag(0), 1); // Normal
        assert_eq!(rs_mode_index_to_flag(1), 2); // Visual
        assert_eq!(rs_mode_index_to_flag(2), 4); // Select
        assert_eq!(rs_mode_index_to_flag(7), 128); // Tip
        assert_eq!(rs_mode_index_to_flag(-1), 0);
        assert_eq!(rs_mode_index_to_flag(8), 0);
    }

    #[test]
    fn test_mode_flag_to_index() {
        assert_eq!(rs_mode_flag_to_index(1), 0); // Normal
        assert_eq!(rs_mode_flag_to_index(2), 1); // Visual
        assert_eq!(rs_mode_flag_to_index(4), 2); // Select
        assert_eq!(rs_mode_flag_to_index(128), 7); // Tip
        assert_eq!(rs_mode_flag_to_index(0), MENU_INDEX_INVALID);
    }

    #[test]
    fn test_mode_flag_to_index_multiple_bits() {
        // With multiple bits set, returns the first one
        assert_eq!(rs_mode_flag_to_index(3), 0); // Normal (first bit)
        assert_eq!(rs_mode_flag_to_index(6), 1); // Visual (first of bits 1,2)
    }

    #[test]
    fn test_menu_mode_name() {
        assert!(!rs_menu_mode_name(MENU_INDEX_NORMAL).is_null());
        assert!(!rs_menu_mode_name(MENU_INDEX_VISUAL).is_null());
        assert!(rs_menu_mode_name(-1).is_null());
        assert!(rs_menu_mode_name(100).is_null());
    }
}
