//! Hidden menu detection.
//!
//! This module provides functions to detect hidden menus. A menu is hidden if:
//! - Its name starts with the hidden character (']')
//! - It's a PopUp menu with a mode suffix (e.g., "PopUpn", "PopUpi")

use std::ffi::CStr;
use std::os::raw::c_char;

use crate::MNU_HIDDEN_CHAR;

/// Check if a menu is hidden.
///
/// A menu is hidden if its name starts with the hidden character (']'),
/// or if it's a PopUp menu with a mode suffix (e.g., "PopUpn" for normal mode).
///
/// This matches the C function `menu_is_hidden` in menu.c:
/// ```c
/// static bool menu_is_hidden(char *name)
/// {
///   return (name[0] == MNU_HIDDEN_CHAR)
///          || (menu_is_popup(name) && name[5] != NUL);
/// }
/// ```
///
/// # Safety
/// The `name` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_is_hidden(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }

    let cstr = unsafe { CStr::from_ptr(name) };
    let bytes = cstr.to_bytes();

    if bytes.is_empty() {
        return false;
    }

    // Hidden if starts with hidden char
    if bytes[0] == MNU_HIDDEN_CHAR {
        return true;
    }

    // Hidden if it's a PopUp menu with a mode suffix
    // (i.e., "PopUp" followed by at least one more character)
    if bytes.starts_with(b"PopUp") && bytes.len() > 5 {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn test_str(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn test_hidden_char() {
        unsafe {
            // Names starting with ']' are hidden
            assert!(rs_menu_is_hidden(test_str("]hidden").as_ptr()));
            assert!(rs_menu_is_hidden(test_str("]").as_ptr()));
            assert!(rs_menu_is_hidden(test_str("]anything").as_ptr()));
        }
    }

    #[test]
    fn test_popup_with_mode_suffix() {
        unsafe {
            // PopUp with mode suffix is hidden
            assert!(rs_menu_is_hidden(test_str("PopUpn").as_ptr())); // normal mode
            assert!(rs_menu_is_hidden(test_str("PopUpi").as_ptr())); // insert mode
            assert!(rs_menu_is_hidden(test_str("PopUpv").as_ptr())); // visual mode
            assert!(rs_menu_is_hidden(test_str("PopUpc").as_ptr())); // cmdline mode
            assert!(rs_menu_is_hidden(test_str("PopUptl").as_ptr())); // terminal mode
            assert!(rs_menu_is_hidden(test_str("PopUpn.SubMenu").as_ptr())); // with submenu
        }
    }

    #[test]
    fn test_not_hidden() {
        unsafe {
            // Regular menus are not hidden
            assert!(!rs_menu_is_hidden(test_str("File").as_ptr()));
            assert!(!rs_menu_is_hidden(test_str("Edit").as_ptr()));
            assert!(!rs_menu_is_hidden(test_str("Help").as_ptr()));

            // PopUp without suffix is not hidden
            assert!(!rs_menu_is_hidden(test_str("PopUp").as_ptr()));

            // ToolBar and WinBar are not hidden
            assert!(!rs_menu_is_hidden(test_str("ToolBar").as_ptr()));
            assert!(!rs_menu_is_hidden(test_str("WinBar").as_ptr()));
            assert!(!rs_menu_is_hidden(test_str("ToolBar.item").as_ptr()));
            assert!(!rs_menu_is_hidden(test_str("WinBar.item").as_ptr()));

            // Empty string and null are not hidden
            assert!(!rs_menu_is_hidden(test_str("").as_ptr()));
            assert!(!rs_menu_is_hidden(std::ptr::null()));
        }
    }

    #[test]
    fn test_popup_submenu_not_hidden() {
        unsafe {
            // "PopUp.SubMenu" is NOT hidden (it's the parent menu, not a mode-specific one)
            // The dot makes it a path, not a mode suffix
            // Wait, let's check - "PopUp.item" has length > 5, so it would be hidden
            // Actually looking at the C code: name[5] != NUL
            // "PopUp" is 5 chars, so name[5] would be '.' in "PopUp.item"
            // So "PopUp.item" IS hidden per the C code
            assert!(rs_menu_is_hidden(test_str("PopUp.item").as_ptr()));
        }
    }
}
