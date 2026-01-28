//! Menu lookup and search utilities.
//!
//! This module provides functions for finding menus in the menu tree by name
//! or path. These are building blocks that can be used by higher-level
//! functions that need to locate menus.

use std::ffi::{c_char, c_int, CStr};

use crate::handle::VimMenuHandle;
use crate::path::rs_menu_name_equal;

/// Result of a menu search operation.
#[repr(C)]
pub struct MenuSearchResult {
    /// The found menu, or null if not found.
    pub menu: VimMenuHandle,
    /// Error code: 0 = success, 1 = not found, 2 = not a submenu, 3 = wrong mode.
    pub error: c_int,
}

/// Error codes for menu search.
pub mod search_error {
    use std::ffi::c_int;

    /// Search succeeded.
    pub const OK: c_int = 0;
    /// Menu was not found.
    pub const NOT_FOUND: c_int = 1;
    /// Path requires a submenu but found a leaf item.
    pub const NOT_SUBMENU: c_int = 2;
    /// Menu exists but not in the requested mode.
    pub const WRONG_MODE: c_int = 3;
}

/// Find a menu among siblings by name.
///
/// Searches through the linked list of menus starting at `menu` to find
/// one whose name matches `name`.
///
/// # Safety
/// The `name` pointer must be valid and point to a null-terminated C string.
/// The `menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_find_menu_sibling(
    menu: VimMenuHandle,
    name: *const c_char,
) -> VimMenuHandle {
    if name.is_null() {
        return VimMenuHandle::null();
    }

    let mut current = menu;
    while !current.is_null() {
        if unsafe { rs_menu_name_equal(name, current) } {
            return current;
        }
        current = current.next();
    }

    VimMenuHandle::null()
}

/// Find a menu by simple name (single component, no path).
///
/// This searches the sibling list for a menu whose name matches,
/// and checks that the menu exists in the requested modes.
///
/// # Safety
/// The `name` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_find_menu_by_name(
    menu: VimMenuHandle,
    name: *const c_char,
    modes: c_int,
) -> MenuSearchResult {
    if name.is_null() {
        return MenuSearchResult {
            menu: VimMenuHandle::null(),
            error: search_error::NOT_FOUND,
        };
    }

    let found = unsafe { rs_find_menu_sibling(menu, name) };

    if found.is_null() {
        return MenuSearchResult {
            menu: VimMenuHandle::null(),
            error: search_error::NOT_FOUND,
        };
    }

    // Check modes
    if (found.modes() & modes) == 0 {
        return MenuSearchResult {
            menu: found,
            error: search_error::WRONG_MODE,
        };
    }

    MenuSearchResult {
        menu: found,
        error: search_error::OK,
    }
}

/// Count the number of menus in a sibling list.
///
/// This counts all menus in the linked list starting at `menu`.
///
/// # Safety
/// The `menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_count_menu_siblings(menu: VimMenuHandle) -> c_int {
    let mut count = 0;
    let mut current = menu;
    while !current.is_null() {
        count += 1;
        current = current.next();
    }
    count
}

/// Check if any menu in the sibling list has the given name.
///
/// # Safety
/// The `name` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_exists(menu: VimMenuHandle, name: *const c_char) -> bool {
    !unsafe { rs_find_menu_sibling(menu, name) }.is_null()
}

/// Get the depth of a menu path (number of dot-separated components).
///
/// For example:
/// - "File" returns 1
/// - "File.Open" returns 2
/// - "File.Open.Recent" returns 3
///
/// # Safety
/// The `path` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_path_depth(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let cstr = unsafe { CStr::from_ptr(path) };
    let bytes = cstr.to_bytes();

    if bytes.is_empty() {
        return 0;
    }

    let mut depth = 1;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'.' {
            depth += 1;
        } else if bytes[i] == b'\\' && i + 1 < bytes.len() {
            // Skip escaped character
            i += 1;
        }
        i += 1;
    }

    depth
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn test_str(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn test_menu_path_depth() {
        unsafe {
            assert_eq!(rs_menu_path_depth(test_str("File").as_ptr()), 1);
            assert_eq!(rs_menu_path_depth(test_str("File.Open").as_ptr()), 2);
            assert_eq!(rs_menu_path_depth(test_str("File.Open.Recent").as_ptr()), 3);
            assert_eq!(rs_menu_path_depth(test_str("").as_ptr()), 0);
            assert_eq!(rs_menu_path_depth(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_menu_path_depth_escaped() {
        unsafe {
            // Escaped dot should not increase depth
            assert_eq!(rs_menu_path_depth(test_str("File\\.Name").as_ptr()), 1);
            assert_eq!(rs_menu_path_depth(test_str("File\\.Name.Open").as_ptr()), 2);
        }
    }

    // Note: Tests that require traversing the menu tree (find_menu_sibling,
    // count_menu_siblings, menu_exists) need C FFI linkage and are tested
    // through integration tests rather than unit tests.
}
