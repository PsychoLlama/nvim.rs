//! Menu name classification utilities for Neovim
//!
//! This module provides Rust implementations of menu type detection functions from
//! `src/nvim/menu.c`. These are pure string prefix checks with no external dependencies.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::ffi::CStr;
use std::os::raw::c_char;

/// Hidden menu character (']')
const MNU_HIDDEN_CHAR: u8 = b']';

/// Check if name is a window toolbar menu name.
/// Returns true if name starts with "WinBar".
///
/// # Safety
/// The `name` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_is_winbar(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(name) };
    cstr.to_bytes().starts_with(b"WinBar")
}

/// Check if name is a popup menu name.
/// Returns true if name starts with "PopUp".
///
/// # Safety
/// The `name` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_is_popup(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(name) };
    cstr.to_bytes().starts_with(b"PopUp")
}

/// Check if name is a toolbar menu name.
/// Returns true if name starts with "ToolBar".
///
/// # Safety
/// The `name` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_is_toolbar(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(name) };
    cstr.to_bytes().starts_with(b"ToolBar")
}

/// Check if name can be a menu in the MenuBar.
/// Returns true if not popup, toolbar, winbar, and doesn't start with hidden char.
///
/// # Safety
/// The `name` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_is_menubar(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(name) };
    let bytes = cstr.to_bytes();

    if bytes.is_empty() {
        return true; // Empty name is menubar
    }

    // Not a menubar if starts with hidden char or is popup/toolbar/winbar
    if bytes[0] == MNU_HIDDEN_CHAR {
        return false;
    }

    !bytes.starts_with(b"PopUp") && !bytes.starts_with(b"ToolBar") && !bytes.starts_with(b"WinBar")
}

/// Check if name is a menu separator identifier.
/// Returns true if name starts and ends with '-'.
///
/// # Safety
/// The `name` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_is_separator(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(name) };
    let bytes = cstr.to_bytes();

    if bytes.is_empty() {
        return false;
    }

    bytes[0] == b'-' && bytes[bytes.len() - 1] == b'-'
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn test_str(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn test_menu_is_winbar() {
        unsafe {
            assert!(rs_menu_is_winbar(test_str("WinBar").as_ptr()));
            assert!(rs_menu_is_winbar(test_str("WinBar.item").as_ptr()));
            assert!(!rs_menu_is_winbar(test_str("WinBa").as_ptr()));
            assert!(!rs_menu_is_winbar(test_str("winbar").as_ptr()));
            assert!(!rs_menu_is_winbar(test_str("").as_ptr()));
            assert!(!rs_menu_is_winbar(std::ptr::null()));
        }
    }

    #[test]
    fn test_menu_is_popup() {
        unsafe {
            assert!(rs_menu_is_popup(test_str("PopUp").as_ptr()));
            assert!(rs_menu_is_popup(test_str("PopUp.item").as_ptr()));
            assert!(!rs_menu_is_popup(test_str("PopU").as_ptr()));
            assert!(!rs_menu_is_popup(test_str("popup").as_ptr()));
            assert!(!rs_menu_is_popup(test_str("").as_ptr()));
            assert!(!rs_menu_is_popup(std::ptr::null()));
        }
    }

    #[test]
    fn test_menu_is_toolbar() {
        unsafe {
            assert!(rs_menu_is_toolbar(test_str("ToolBar").as_ptr()));
            assert!(rs_menu_is_toolbar(test_str("ToolBar.item").as_ptr()));
            assert!(!rs_menu_is_toolbar(test_str("ToolBa").as_ptr()));
            assert!(!rs_menu_is_toolbar(test_str("toolbar").as_ptr()));
            assert!(!rs_menu_is_toolbar(test_str("").as_ptr()));
            assert!(!rs_menu_is_toolbar(std::ptr::null()));
        }
    }

    #[test]
    fn test_menu_is_menubar() {
        unsafe {
            // Menubar items
            assert!(rs_menu_is_menubar(test_str("File").as_ptr()));
            assert!(rs_menu_is_menubar(test_str("Edit").as_ptr()));
            assert!(rs_menu_is_menubar(test_str("").as_ptr()));

            // Not menubar items
            assert!(!rs_menu_is_menubar(test_str("PopUp").as_ptr()));
            assert!(!rs_menu_is_menubar(test_str("ToolBar").as_ptr()));
            assert!(!rs_menu_is_menubar(test_str("WinBar").as_ptr()));
            assert!(!rs_menu_is_menubar(test_str("]hidden").as_ptr()));
            assert!(!rs_menu_is_menubar(std::ptr::null()));
        }
    }

    #[test]
    fn test_menu_is_separator() {
        unsafe {
            assert!(rs_menu_is_separator(test_str("-").as_ptr()));
            assert!(rs_menu_is_separator(test_str("--").as_ptr()));
            assert!(rs_menu_is_separator(test_str("-sep-").as_ptr()));
            assert!(!rs_menu_is_separator(test_str("item").as_ptr()));
            assert!(!rs_menu_is_separator(test_str("-item").as_ptr()));
            assert!(!rs_menu_is_separator(test_str("item-").as_ptr()));
            assert!(!rs_menu_is_separator(test_str("").as_ptr()));
            assert!(!rs_menu_is_separator(std::ptr::null()));
        }
    }

    #[test]
    fn test_mnu_hidden_char_constant() {
        // Verify MNU_HIDDEN_CHAR matches C definition
        assert_eq!(MNU_HIDDEN_CHAR, b']');
    }

    #[test]
    fn test_menu_prefixes_are_distinct() {
        // Verify menu prefixes are mutually exclusive (no prefix is a substring of another)
        let prefixes = [b"WinBar".as_slice(), b"PopUp", b"ToolBar"];
        for i in 0..prefixes.len() {
            for j in 0..prefixes.len() {
                if i != j {
                    assert!(!prefixes[i].starts_with(prefixes[j]));
                    assert!(!prefixes[j].starts_with(prefixes[i]));
                }
            }
        }
    }

    #[test]
    fn test_menu_hidden_is_ascii() {
        // Hidden char should be an ASCII printable character
        let hidden = MNU_HIDDEN_CHAR;
        assert!((0x20..0x7f).contains(&hidden));
    }
}
