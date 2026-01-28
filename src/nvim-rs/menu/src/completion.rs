//! Menu completion utilities.
//!
//! This module provides helper functions for command-line completion
//! of menu names (used by wildmenu).

use std::ffi::{c_char, c_int, CStr};

use crate::handle::VimMenuHandle;
use crate::menu_modes::MENU_ALL_MODES;

/// Result of analyzing a menu path for completion.
#[repr(C)]
pub struct MenuCompletionContext {
    /// The menu to complete from (where to search for children).
    pub menu: VimMenuHandle,
    /// Offset where the pattern to match starts.
    pub pattern_offset: c_int,
    /// Whether completion found a valid context.
    pub valid: bool,
}

/// Check if a character is ASCII whitespace.
fn is_ascii_whitespace(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r'
}

/// Check if a character is ASCII digit.
fn is_ascii_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

/// Analyze a menu argument string to find the completion context.
///
/// Given a partial menu path like "File.Op", this determines:
/// - Which parent menu to search in (File)
/// - Where the pattern to complete starts (after "File.")
///
/// # Safety
/// The `arg` pointer must be valid and point to a null-terminated C string.
/// The `root_menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_completion_context(
    arg: *const c_char,
    root_menu: VimMenuHandle,
    modes: c_int,
) -> MenuCompletionContext {
    if arg.is_null() {
        return MenuCompletionContext {
            menu: root_menu,
            pattern_offset: 0,
            valid: false,
        };
    }

    let cstr = unsafe { CStr::from_ptr(arg) };
    let bytes = cstr.to_bytes();

    // Skip leading whitespace and priority numbers
    let mut p = 0;
    while p < bytes.len() && (is_ascii_digit(bytes[p]) || bytes[p] == b'.') {
        p += 1;
    }

    // Check for enable/disable
    if p < bytes.len() && !is_ascii_whitespace(bytes[p]) {
        if bytes[p..].starts_with(b"enable")
            && (p + 6 >= bytes.len() || is_ascii_whitespace(bytes[p + 6]))
        {
            p += 6;
        } else if bytes[p..].starts_with(b"disable")
            && (p + 7 >= bytes.len() || is_ascii_whitespace(bytes[p + 7]))
        {
            p += 7;
        }
    }

    // Skip whitespace
    while p < bytes.len() && is_ascii_whitespace(bytes[p]) {
        p += 1;
    }

    let arg_start = p;
    let mut after_dot = p;

    // Find the last dot (path separator)
    while p < bytes.len() && !is_ascii_whitespace(bytes[p]) {
        if bytes[p] == b'\\' && p + 1 < bytes.len() {
            p += 2; // Skip escaped char
        } else if bytes[p] == b'.' {
            after_dot = p + 1;
            p += 1;
        } else {
            p += 1;
        }
    }

    // If we hit whitespace, no completion context
    if p < bytes.len() && is_ascii_whitespace(bytes[p]) {
        return MenuCompletionContext {
            menu: root_menu,
            pattern_offset: after_dot as c_int,
            valid: false,
        };
    }

    // If there's a path (after_dot > arg_start), find the parent menu
    let menu = if after_dot > arg_start {
        find_parent_menu(root_menu, bytes, arg_start, after_dot - 1, modes)
    } else {
        root_menu
    };

    MenuCompletionContext {
        menu,
        pattern_offset: after_dot as c_int,
        valid: !menu.is_null(),
    }
}

/// Find the parent menu given a path prefix.
fn find_parent_menu(
    root: VimMenuHandle,
    bytes: &[u8],
    start: usize,
    end: usize,
    modes: c_int,
) -> VimMenuHandle {
    // This would need to parse the path and find the parent menu
    // For now, return root - the actual implementation needs path parsing
    // integration with menu_name_skip and menu_name_equal
    let _ = (bytes, start, end, modes);
    root
}

/// Count the number of completable menu items.
///
/// # Safety
/// The `menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_count_completable_menus(menu: VimMenuHandle, modes: c_int) -> c_int {
    let mut count = 0;
    let mut current = menu;

    while !current.is_null() {
        if (current.modes() & modes) != 0 {
            count += 1;
        }
        current = current.next();
    }

    count
}

/// Check if a menu is completable (has modes matching the completion context).
///
/// # Safety
/// The `menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_is_completable(menu: VimMenuHandle, modes: c_int) -> bool {
    if menu.is_null() {
        return false;
    }
    (menu.modes() & modes) != 0
}

/// Check if we should include all modes for completion.
///
/// When completing for :menu (not :unmenu), we include all modes
/// because the user might want to add a new menu with the same name.
#[no_mangle]
pub extern "C" fn rs_should_expand_all_modes(is_unmenu: bool) -> c_int {
    if is_unmenu {
        0 // Use specified modes
    } else {
        MENU_ALL_MODES // Include all modes
    }
}

// Note: Tests that don't require C FFI linkage

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_expand_all_modes() {
        assert_eq!(rs_should_expand_all_modes(true), 0);
        assert_eq!(rs_should_expand_all_modes(false), MENU_ALL_MODES);
    }

    #[test]
    fn test_is_ascii_whitespace() {
        assert!(is_ascii_whitespace(b' '));
        assert!(is_ascii_whitespace(b'\t'));
        assert!(is_ascii_whitespace(b'\n'));
        assert!(!is_ascii_whitespace(b'a'));
        assert!(!is_ascii_whitespace(b'.'));
    }

    #[test]
    fn test_is_ascii_digit() {
        assert!(is_ascii_digit(b'0'));
        assert!(is_ascii_digit(b'5'));
        assert!(is_ascii_digit(b'9'));
        assert!(!is_ascii_digit(b'a'));
        assert!(!is_ascii_digit(b' '));
    }
}
