//! Menu creation utilities.
//!
//! This module provides helper functions for menu creation operations.
//! The actual menu allocation and insertion is handled in C since it
//! involves complex memory management and global state.

use std::ffi::{c_char, c_int};

use crate::handle::VimMenuHandle;

/// Find the insertion point for a new menu based on priority.
///
/// Given a menu list, finds where to insert a new menu with the given
/// priority. Returns a pointer to the "next" field that should point
/// to the new menu.
///
/// This implements the priority-based sorting logic used for menu insertion:
/// new menus are inserted after all menus with lower or equal priority.
///
/// # Safety
/// The `menu` handle must be valid or null.
/// The `priority` value should be a valid menu priority.
#[no_mangle]
pub unsafe extern "C" fn rs_find_menu_insert_point(
    menu: VimMenuHandle,
    priority: c_int,
) -> VimMenuHandle {
    let mut current = menu;
    let mut prev = VimMenuHandle::null();

    while !current.is_null() {
        if current.priority() > priority {
            // Found a menu with higher priority - insert before it
            break;
        }
        prev = current;
        current = current.next();
    }

    prev
}

/// Check if a menu item would be a duplicate (same name already exists).
///
/// # Safety
/// The `menu` handle must be valid or null.
/// The `name` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_item_exists(menu: VimMenuHandle, name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }
    crate::lookup::rs_menu_exists(menu, name)
}

/// Compute the modes for a new menu based on parent and command modes.
///
/// When creating a new menu, the modes are initially set based on the
/// command's modes. This returns the combined modes.
#[no_mangle]
pub extern "C" fn rs_compute_new_menu_modes(parent_modes: c_int, command_modes: c_int) -> c_int {
    // Initially, a new menu has the modes from the command
    // It will be further modified based on existing menu state
    command_modes & (!parent_modes | parent_modes)
}

/// Validate a menu name for creation.
///
/// Returns true if the name is valid for creating a menu.
/// Invalid names include:
/// - NULL or empty strings
/// - Names that are only whitespace
///
/// # Safety
/// The `name` pointer must be valid and point to a null-terminated C string,
/// or be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_menu_name(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }

    let cstr = unsafe { std::ffi::CStr::from_ptr(name) };
    let bytes = cstr.to_bytes();

    if bytes.is_empty() {
        return false;
    }

    // Check if it's all whitespace
    !bytes.iter().all(|&b| b == b' ' || b == b'\t')
}

/// Check if creating this menu would require creating parent menus.
///
/// Returns true if the menu path contains dots (separators).
///
/// # Safety
/// The `path` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_path_needs_parents(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }

    crate::lookup::rs_menu_path_depth(path) > 1
}

// Note: Tests for functions that don't require C FFI linkage

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn test_str(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn test_compute_new_menu_modes() {
        // Basic mode computation
        assert_eq!(rs_compute_new_menu_modes(0, 1), 1);
        assert_eq!(rs_compute_new_menu_modes(0, 7), 7);
        assert_eq!(rs_compute_new_menu_modes(1, 3), 3);
    }

    #[test]
    fn test_validate_menu_name() {
        unsafe {
            assert!(rs_validate_menu_name(test_str("File").as_ptr()));
            assert!(rs_validate_menu_name(test_str("Edit.Open").as_ptr()));
            assert!(!rs_validate_menu_name(test_str("").as_ptr()));
            assert!(!rs_validate_menu_name(test_str("   ").as_ptr()));
            assert!(!rs_validate_menu_name(test_str("\t").as_ptr()));
            assert!(!rs_validate_menu_name(std::ptr::null()));
        }
    }

    #[test]
    fn test_menu_path_needs_parents() {
        unsafe {
            assert!(!rs_menu_path_needs_parents(test_str("File").as_ptr()));
            assert!(rs_menu_path_needs_parents(test_str("File.Open").as_ptr()));
            assert!(rs_menu_path_needs_parents(
                test_str("File.Open.Recent").as_ptr()
            ));
            assert!(!rs_menu_path_needs_parents(std::ptr::null()));
        }
    }
}
