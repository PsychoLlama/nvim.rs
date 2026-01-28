//! Popup menu utilities.
//!
//! This module provides helper functions for popup menu operations,
//! including finding the appropriate popup menu for the current mode
//! and generating mode-suffixed popup menu names.

use std::ffi::{c_char, c_int, CStr};

use crate::handle::VimMenuHandle;
use crate::menu_modes::{
    MENU_INDEX_CMDLINE, MENU_INDEX_INSERT, MENU_INDEX_INVALID, MENU_INDEX_NORMAL,
    MENU_INDEX_OP_PENDING, MENU_INDEX_SELECT, MENU_INDEX_TERMINAL, MENU_INDEX_TIP,
    MENU_INDEX_VISUAL,
};

/// Mode character strings for popup menus.
/// These match the C definition: `static char *menu_mode_chars[]`
/// Index corresponds to MENU_INDEX_* values.
pub const MENU_MODE_CHARS: [&str; 8] = ["n", "v", "s", "o", "i", "c", "tl", "t"];

/// The "PopUp" prefix used for popup menus.
const POPUP_PREFIX: &[u8] = b"PopUp";

/// Get the mode character string for a menu mode index.
///
/// Returns the mode characters used in popup menu names:
/// - 0 (Normal) -> "n"
/// - 1 (Visual) -> "v"
/// - 2 (Select) -> "s"
/// - 3 (Op-pending) -> "o"
/// - 4 (Insert) -> "i"
/// - 5 (Cmdline) -> "c"
/// - 6 (Terminal) -> "tl"
/// - 7 (Tip) -> "t"
#[no_mangle]
pub extern "C" fn rs_get_menu_mode_chars(mode_idx: c_int) -> *const c_char {
    match mode_idx {
        MENU_INDEX_NORMAL => c"n".as_ptr(),
        MENU_INDEX_VISUAL => c"v".as_ptr(),
        MENU_INDEX_SELECT => c"s".as_ptr(),
        MENU_INDEX_OP_PENDING => c"o".as_ptr(),
        MENU_INDEX_INSERT => c"i".as_ptr(),
        MENU_INDEX_CMDLINE => c"c".as_ptr(),
        MENU_INDEX_TERMINAL => c"tl".as_ptr(),
        MENU_INDEX_TIP => c"t".as_ptr(),
        _ => c"".as_ptr(),
    }
}

/// Get the length of the mode character string for a given mode index.
#[no_mangle]
pub extern "C" fn rs_get_menu_mode_chars_len(mode_idx: c_int) -> c_int {
    match mode_idx {
        MENU_INDEX_NORMAL => 1,
        MENU_INDEX_VISUAL => 1,
        MENU_INDEX_SELECT => 1,
        MENU_INDEX_OP_PENDING => 1,
        MENU_INDEX_INSERT => 1,
        MENU_INDEX_CMDLINE => 1,
        MENU_INDEX_TERMINAL => 2, // "tl"
        MENU_INDEX_TIP => 1,
        _ => 0,
    }
}

/// Find a popup menu for the given mode index.
///
/// Searches through the root menu list to find a menu whose name
/// matches "PopUp" followed by the mode characters (e.g., "PopUpn" for Normal).
///
/// # Safety
/// The `root_menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_find_popup_menu(
    root_menu: VimMenuHandle,
    mode_idx: c_int,
) -> VimMenuHandle {
    if root_menu.is_null() || !is_valid_mode_index(mode_idx) {
        return VimMenuHandle::null();
    }

    let mode_chars: &[u8] = match mode_idx {
        MENU_INDEX_NORMAL => b"n",
        MENU_INDEX_VISUAL => b"v",
        MENU_INDEX_SELECT => b"s",
        MENU_INDEX_OP_PENDING => b"o",
        MENU_INDEX_INSERT => b"i",
        MENU_INDEX_CMDLINE => b"c",
        MENU_INDEX_TERMINAL => b"tl",
        MENU_INDEX_TIP => b"t",
        _ => return VimMenuHandle::null(),
    };

    let mut current = root_menu;
    while !current.is_null() {
        let name_ptr = current.name();
        if !name_ptr.is_null() {
            let name_cstr = unsafe { CStr::from_ptr(name_ptr) };
            let name_bytes = name_cstr.to_bytes();

            // Check if name starts with "PopUp" and then has the mode chars
            if name_bytes.starts_with(POPUP_PREFIX) {
                let suffix = &name_bytes[POPUP_PREFIX.len()..];
                if suffix.starts_with(mode_chars) {
                    // Ensure the mode chars match exactly (e.g., "PopUpn" not "PopUpnormal")
                    if suffix.len() == mode_chars.len()
                        || (suffix.len() > mode_chars.len()
                            && !suffix[mode_chars.len()].is_ascii_alphanumeric())
                    {
                        return current;
                    }
                }
            }
        }
        current = current.next();
    }

    VimMenuHandle::null()
}

/// Check if a popup menu should be shown (has children).
///
/// # Safety
/// The `menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_popup_menu_should_show(menu: VimMenuHandle) -> bool {
    if menu.is_null() {
        return false;
    }
    !menu.children().is_null()
}

/// Check if a menu name is a popup menu name for a specific mode.
///
/// The matching is exact: "PopUpn" matches Normal mode, "PopUptl" matches Terminal mode.
/// "PopUptl" does NOT match Tip mode (which uses "t") because "tl" != "t".
///
/// # Safety
/// The `name` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_is_popup_menu_for_mode(name: *const c_char, mode_idx: c_int) -> bool {
    if name.is_null() || !is_valid_mode_index(mode_idx) {
        return false;
    }

    let mode_chars = match mode_idx {
        MENU_INDEX_NORMAL => b"n".as_slice(),
        MENU_INDEX_VISUAL => b"v".as_slice(),
        MENU_INDEX_SELECT => b"s".as_slice(),
        MENU_INDEX_OP_PENDING => b"o".as_slice(),
        MENU_INDEX_INSERT => b"i".as_slice(),
        MENU_INDEX_CMDLINE => b"c".as_slice(),
        MENU_INDEX_TERMINAL => b"tl".as_slice(),
        MENU_INDEX_TIP => b"t".as_slice(),
        _ => return false,
    };

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_bytes = name_cstr.to_bytes();

    if !name_bytes.starts_with(POPUP_PREFIX) {
        return false;
    }

    let suffix = &name_bytes[POPUP_PREFIX.len()..];

    // Check that suffix starts with mode_chars AND that:
    // - suffix is exactly mode_chars, OR
    // - the character after mode_chars is not alphanumeric (e.g., "PopUpn.Foo")
    if !suffix.starts_with(mode_chars) {
        return false;
    }

    // If suffix is longer than mode_chars, ensure the next character is not alphanumeric
    // to avoid "PopUptl" matching "t" (TIP) mode
    if suffix.len() > mode_chars.len() {
        let next_char = suffix[mode_chars.len()];
        // If the next character could extend the mode string, it's not a match
        // "tl" should not match if we're looking for "t"
        if next_char.is_ascii_alphanumeric() {
            return false;
        }
    }

    true
}

/// Calculate the required buffer size for a popup mode name.
///
/// Given a base name like "PopUp.Foo" and a mode index, returns the
/// size needed for the modified name (e.g., "PopUpn.Foo").
///
/// # Safety
/// The `name` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_popup_mode_name_len(name: *const c_char, mode_idx: c_int) -> c_int {
    if name.is_null() || !is_valid_mode_index(mode_idx) {
        return 0;
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_len = name_cstr.to_bytes().len();

    let mode_chars_len = match mode_idx {
        MENU_INDEX_TERMINAL => 2,
        _ => 1,
    };

    // name_len + mode_chars_len + 1 (null terminator)
    (name_len + mode_chars_len + 1) as c_int
}

/// Check if a mode index is valid.
fn is_valid_mode_index(mode_idx: c_int) -> bool {
    mode_idx != MENU_INDEX_INVALID && (0..8).contains(&mode_idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_mode_chars() {
        assert_eq!(MENU_MODE_CHARS[MENU_INDEX_NORMAL as usize], "n");
        assert_eq!(MENU_MODE_CHARS[MENU_INDEX_VISUAL as usize], "v");
        assert_eq!(MENU_MODE_CHARS[MENU_INDEX_SELECT as usize], "s");
        assert_eq!(MENU_MODE_CHARS[MENU_INDEX_OP_PENDING as usize], "o");
        assert_eq!(MENU_MODE_CHARS[MENU_INDEX_INSERT as usize], "i");
        assert_eq!(MENU_MODE_CHARS[MENU_INDEX_CMDLINE as usize], "c");
        assert_eq!(MENU_MODE_CHARS[MENU_INDEX_TERMINAL as usize], "tl");
        assert_eq!(MENU_MODE_CHARS[MENU_INDEX_TIP as usize], "t");
    }

    #[test]
    fn test_get_menu_mode_chars_len() {
        assert_eq!(rs_get_menu_mode_chars_len(MENU_INDEX_NORMAL), 1);
        assert_eq!(rs_get_menu_mode_chars_len(MENU_INDEX_TERMINAL), 2);
        assert_eq!(rs_get_menu_mode_chars_len(MENU_INDEX_INVALID), 0);
        assert_eq!(rs_get_menu_mode_chars_len(100), 0);
    }

    #[test]
    fn test_is_valid_mode_index() {
        assert!(is_valid_mode_index(MENU_INDEX_NORMAL));
        assert!(is_valid_mode_index(MENU_INDEX_TIP));
        assert!(!is_valid_mode_index(MENU_INDEX_INVALID));
        assert!(!is_valid_mode_index(-5));
        assert!(!is_valid_mode_index(100));
    }

    #[test]
    fn test_is_popup_menu_for_mode() {
        use std::ffi::CString;

        unsafe {
            let popup_n = CString::new("PopUpn").unwrap();
            assert!(rs_is_popup_menu_for_mode(
                popup_n.as_ptr(),
                MENU_INDEX_NORMAL
            ));
            assert!(!rs_is_popup_menu_for_mode(
                popup_n.as_ptr(),
                MENU_INDEX_VISUAL
            ));

            let popup_v = CString::new("PopUpv").unwrap();
            assert!(rs_is_popup_menu_for_mode(
                popup_v.as_ptr(),
                MENU_INDEX_VISUAL
            ));
            assert!(!rs_is_popup_menu_for_mode(
                popup_v.as_ptr(),
                MENU_INDEX_NORMAL
            ));

            let popup_tl = CString::new("PopUptl").unwrap();
            assert!(rs_is_popup_menu_for_mode(
                popup_tl.as_ptr(),
                MENU_INDEX_TERMINAL
            ));
            assert!(!rs_is_popup_menu_for_mode(
                popup_tl.as_ptr(),
                MENU_INDEX_TIP
            ));

            let not_popup = CString::new("File").unwrap();
            assert!(!rs_is_popup_menu_for_mode(
                not_popup.as_ptr(),
                MENU_INDEX_NORMAL
            ));
        }
    }

    #[test]
    fn test_popup_mode_name_len() {
        use std::ffi::CString;

        unsafe {
            let name = CString::new("PopUp.Foo").unwrap();
            // "PopUp.Foo" (9) + "n" (1) + null (1) = 11
            assert_eq!(rs_popup_mode_name_len(name.as_ptr(), MENU_INDEX_NORMAL), 11);
            // "PopUp.Foo" (9) + "tl" (2) + null (1) = 12
            assert_eq!(
                rs_popup_mode_name_len(name.as_ptr(), MENU_INDEX_TERMINAL),
                12
            );
        }
    }
}
