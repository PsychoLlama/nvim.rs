//! Menu deletion utilities.
//!
//! This module provides helper functions for menu deletion operations.
//! The actual menu deallocation is handled in C since it involves
//! complex memory management and linked list manipulation.

use std::ffi::c_int;

use crate::handle::VimMenuHandle;
use crate::menu_modes::{MENU_ALL_MODES, MENU_TIP_MODE};

/// Check if a menu should be fully deleted after removing modes.
///
/// A menu should be deleted when it has no modes left after removal.
///
/// # Safety
/// The `menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_should_delete_menu(
    menu: VimMenuHandle,
    modes_to_remove: c_int,
) -> bool {
    if menu.is_null() {
        return false;
    }

    let remaining_modes = menu.modes() & !modes_to_remove;
    (remaining_modes & MENU_ALL_MODES) == 0
}

/// Calculate the remaining modes after removal.
///
/// # Safety
/// The `menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_remaining_modes(
    menu: VimMenuHandle,
    modes_to_remove: c_int,
) -> c_int {
    if menu.is_null() {
        return 0;
    }

    menu.modes() & !modes_to_remove
}

/// Check if tip mode string should be freed.
///
/// The tip mode string should be freed when MENU_TIP_MODE is being removed.
#[no_mangle]
pub extern "C" fn rs_should_free_tip(modes_to_remove: c_int) -> bool {
    (modes_to_remove & MENU_TIP_MODE) != 0
}

/// Recalculate parent modes based on children.
///
/// After removing children, the parent's modes should be the union
/// of all remaining children's modes.
///
/// # Safety
/// The `first_child` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_recalculate_parent_modes(first_child: VimMenuHandle) -> c_int {
    let mut modes = 0;
    let mut child = first_child;
    while !child.is_null() {
        modes |= child.modes();
        child = child.next();
    }
    modes
}

/// Check if a menu has any valid modes remaining.
///
/// # Safety
/// The `menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_has_modes(menu: VimMenuHandle) -> bool {
    if menu.is_null() {
        return false;
    }
    (menu.modes() & MENU_ALL_MODES) != 0
}

/// Check if a menu is empty (no children and can be pruned).
///
/// # Safety
/// The `menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_is_empty(menu: VimMenuHandle) -> bool {
    if menu.is_null() {
        return true;
    }
    menu.children().is_null() && (menu.modes() & MENU_ALL_MODES) == 0
}

// Note: Tests for functions that don't require C FFI linkage

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_free_tip() {
        assert!(rs_should_free_tip(MENU_TIP_MODE));
        assert!(rs_should_free_tip(MENU_TIP_MODE | 1));
        assert!(!rs_should_free_tip(1));
        assert!(!rs_should_free_tip(0));
    }

    // Note: Tests for rs_should_delete_menu and other functions that use
    // VimMenuHandle require C FFI linkage and are tested through integration tests.
}
