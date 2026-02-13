//! Menu lookup and search utilities.
//!
//! This module provides functions for finding menus in the menu tree by name
//! or path. These are building blocks that can be used by higher-level
//! functions that need to locate menus.

use std::ffi::{c_char, c_int, c_void, CStr};

use crate::handle::VimMenuHandle;
use crate::path::{rs_menu_name_equal, rs_menu_name_skip};

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

// Error message strings
const E_NOTSUBMENU: *const c_char = c"E327: Part of menu-item path is not sub-menu".as_ptr();
const E_NOMENU: *const c_char = c"E329: No menu \"%s\"".as_ptr();
const E_MENU_ONLY_EXISTS_IN_ANOTHER_MODE: *const c_char =
    c"E328: Menu only exists in another mode".as_ptr();
const E333_MENU_PATH_MUST_LEAD_TO_ITEM: *const c_char =
    c"E333: Menu path must lead to a menu item".as_ptr();
const E334_MENU_NOT_FOUND: *const c_char = c"E334: Menu not found: %s".as_ptr();
const E336_MENU_PATH_MUST_LEAD_TO_SUBMENU: *const c_char =
    c"E336: Menu path must lead to a sub-menu".as_ptr();
const E337_MENU_NOT_FOUND: *const c_char = c"E337: Menu not found - check menu names".as_ptr();

extern "C" {
    fn emsg(s: *const c_char) -> bool;
    fn semsg(s: *const c_char, ...) -> bool;
    fn nvim_menu_get_root_menu() -> VimMenuHandle;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn nvim_gettext(s: *const c_char) -> *const c_char;
}

/// Find menu matching `name` and `modes`. Does not handle empty `name`.
///
/// Walks the menu tree by dot-separated path components. Each component
/// is matched against menu names using `menu_name_equal`.
///
/// This is the Rust implementation of C `find_menu()`.
///
/// # Safety
/// All pointers must be valid. `name` is modified in-place by `menu_name_skip`.
#[no_mangle]
pub unsafe extern "C" fn rs_find_menu(
    menu: VimMenuHandle,
    name: *mut c_char,
    modes: c_int,
) -> VimMenuHandle {
    if name.is_null() {
        return VimMenuHandle::null();
    }

    let mut cur_menu = menu;
    let mut cur_name = name;

    while unsafe { *cur_name } != 0 {
        // find the end of one dot-separated name and put a NUL at the dot
        let p = unsafe { rs_menu_name_skip(cur_name) };

        while !cur_menu.is_null() {
            if unsafe { rs_menu_name_equal(cur_name, cur_menu) } {
                // Found menu
                if unsafe { *p } != 0 && cur_menu.children().is_null() {
                    unsafe { emsg(nvim_gettext(E_NOTSUBMENU)) };
                    return VimMenuHandle::null();
                } else if (cur_menu.modes() & modes) == 0 {
                    unsafe { emsg(nvim_gettext(E_MENU_ONLY_EXISTS_IN_ANOTHER_MODE)) };
                    return VimMenuHandle::null();
                } else if unsafe { *p } == 0 {
                    // found a full match
                    return cur_menu;
                }
                break;
            }
            cur_menu = cur_menu.next();
        }

        if cur_menu.is_null() {
            unsafe { semsg(nvim_gettext(E_NOMENU), cur_name) };
            return VimMenuHandle::null();
        }
        // Found a match, search the sub-menu.
        cur_name = p;
        cur_menu = cur_menu.children();
    }

    // Should not reach here with valid input (name is non-empty on entry
    // and we return inside the loop). Match C behavior: abort.
    VimMenuHandle::null()
}

/// Lookup a menu by the descriptor name e.g. "File.New".
///
/// Returns NULL if the menu is not found. Only finds leaf menus.
///
/// This is the Rust implementation of C `menu_getbyname()`.
///
/// # Safety
/// `name_arg` must be a valid pointer to a NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_getbyname(name_arg: *const c_char) -> VimMenuHandle {
    if name_arg.is_null() {
        return VimMenuHandle::null();
    }

    let saved_name = unsafe { xstrdup(name_arg) };
    let mut menu = unsafe { nvim_menu_get_root_menu() };
    let mut name = saved_name;
    let mut gave_emsg = false;

    while unsafe { *name } != 0 {
        // Find in the menu hierarchy
        let p = unsafe { rs_menu_name_skip(name) };

        while !menu.is_null() {
            if unsafe { rs_menu_name_equal(name, menu) } {
                if unsafe { *p } == 0 && !menu.children().is_null() {
                    unsafe { emsg(nvim_gettext(E333_MENU_PATH_MUST_LEAD_TO_ITEM)) };
                    gave_emsg = true;
                    menu = VimMenuHandle::null();
                } else if unsafe { *p } != 0 && menu.children().is_null() {
                    unsafe { emsg(nvim_gettext(E_NOTSUBMENU)) };
                    menu = VimMenuHandle::null();
                }
                break;
            }
            menu = menu.next();
        }
        if menu.is_null() || unsafe { *p } == 0 {
            break;
        }
        menu = menu.children();
        name = p;
    }

    unsafe { xfree(saved_name as *mut c_void) };

    if menu.is_null() {
        if !gave_emsg {
            unsafe { semsg(nvim_gettext(E334_MENU_NOT_FOUND), name_arg) };
        }
        return VimMenuHandle::null();
    }

    menu
}

/// Given a menu descriptor, e.g. "File.New", find it in the menu hierarchy.
///
/// Returns a submenu (not a leaf item). Emits error messages on failure.
///
/// This is the Rust implementation of C `menu_find()`.
///
/// # Safety
/// `path_name` must be a valid pointer to a NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_find(path_name: *const c_char) -> VimMenuHandle {
    if path_name.is_null() {
        return VimMenuHandle::null();
    }

    let mut menu = unsafe { nvim_menu_get_root_menu() };
    let saved_name = unsafe { xstrdup(path_name) };
    let mut name = saved_name;

    while unsafe { *name } != 0 {
        // find the end of one dot-separated name and put a NUL at the dot
        let p = unsafe { rs_menu_name_skip(name) };

        while !menu.is_null() {
            if unsafe { rs_menu_name_equal(name, menu) } {
                if menu.children().is_null() {
                    // found a menu item instead of a sub-menu
                    if unsafe { *p } == 0 {
                        unsafe {
                            emsg(nvim_gettext(E336_MENU_PATH_MUST_LEAD_TO_SUBMENU));
                        }
                    } else {
                        unsafe { emsg(nvim_gettext(E_NOTSUBMENU)) };
                    }
                    menu = VimMenuHandle::null();
                    unsafe { xfree(saved_name as *mut c_void) };
                    return menu;
                }
                if unsafe { *p } == 0 {
                    // found a full match
                    unsafe { xfree(saved_name as *mut c_void) };
                    return menu;
                }
                break;
            }
            menu = menu.next();
        }
        if menu.is_null() {
            // didn't find it
            break;
        }

        // Found a match, search the sub-menu.
        menu = menu.children();
        name = p;
    }

    if menu.is_null() {
        unsafe { emsg(nvim_gettext(E337_MENU_NOT_FOUND)) };
    }
    unsafe { xfree(saved_name as *mut c_void) };
    menu
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
