//! Menu Ex command utilities.
//!
//! This module provides functions for parsing menu command names and
//! extracting mode information from them.

use std::ffi::{c_char, c_int, CStr};

use crate::menu_modes::{
    MENU_CMDLINE_MODE, MENU_INSERT_MODE, MENU_NORMAL_MODE, MENU_OP_PENDING_MODE, MENU_SELECT_MODE,
    MENU_TERMINAL_MODE, MENU_TIP_MODE, MENU_VISUAL_MODE,
};

/// REMAP values for menu mappings.
pub mod remap {
    use std::ffi::c_int;

    /// Yes, remap the menu mapping.
    pub const REMAP_YES: c_int = 0;
    /// No, don't remap (nore- prefix).
    pub const REMAP_NONE: c_int = -1;
    /// Remap script-local mappings only.
    pub const REMAP_SCRIPT: c_int = -2;
}

/// Result of parsing a menu command.
#[repr(C)]
pub struct MenuCmdResult {
    /// The modes for this command.
    pub modes: c_int,
    /// REMAP_YES or REMAP_NONE.
    pub noremap: c_int,
    /// True if this is an unmenu command.
    pub unmenu: bool,
    /// Number of characters consumed from the command.
    pub consumed: c_int,
}

/// Parse a menu command and return its modes.
///
/// Given a command like "nmenu", "vmenu", "amenu", "nunmenu", etc.,
/// this returns the modes and flags associated with that command.
///
/// # Arguments
/// * `cmd` - The command string (e.g., "nmenu", "menu", "vmenu")
/// * `forceit` - Whether the command had a bang (!)
///
/// # Safety
/// The `cmd` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_menu_cmd_modes(cmd: *const c_char, forceit: bool) -> MenuCmdResult {
    if cmd.is_null() {
        return MenuCmdResult {
            modes: 0,
            noremap: remap::REMAP_YES,
            unmenu: false,
            consumed: 0,
        };
    }

    let cstr = unsafe { CStr::from_ptr(cmd) };
    let bytes = cstr.to_bytes();

    if bytes.is_empty() {
        return MenuCmdResult {
            modes: 0,
            noremap: remap::REMAP_YES,
            unmenu: false,
            consumed: 0,
        };
    }

    let mut idx = 0;
    let first = bytes[idx];
    idx += 1;

    let modes = match first {
        b'v' => MENU_VISUAL_MODE | MENU_SELECT_MODE,
        b'x' => MENU_VISUAL_MODE,
        b's' => MENU_SELECT_MODE,
        b'o' => MENU_OP_PENDING_MODE,
        b'i' => MENU_INSERT_MODE,
        b't' => {
            if idx < bytes.len() && bytes[idx] == b'l' {
                idx += 1;
                MENU_TERMINAL_MODE
            } else {
                MENU_TIP_MODE
            }
        }
        b'c' => MENU_CMDLINE_MODE,
        b'a' => {
            MENU_INSERT_MODE
                | MENU_CMDLINE_MODE
                | MENU_NORMAL_MODE
                | MENU_VISUAL_MODE
                | MENU_SELECT_MODE
                | MENU_OP_PENDING_MODE
        }
        b'n' => {
            // Check if it's "nmenu" or start of "noremenu"
            if idx < bytes.len() && bytes[idx] != b'o' {
                MENU_NORMAL_MODE
            } else {
                // It's noremenu or similar, revert
                idx -= 1;
                default_modes(forceit)
            }
        }
        _ => {
            // Default case, revert the increment
            idx -= 1;
            default_modes(forceit)
        }
    };

    // Determine noremap and unmenu from remaining command
    let noremap = if idx < bytes.len() && bytes[idx] == b'n' {
        remap::REMAP_NONE
    } else {
        remap::REMAP_YES
    };

    let unmenu = idx < bytes.len() && bytes[idx] == b'u';

    MenuCmdResult {
        modes,
        noremap,
        unmenu,
        consumed: idx as c_int,
    }
}

/// Get the default modes based on forceit flag.
fn default_modes(forceit: bool) -> c_int {
    if forceit {
        // menu!!
        MENU_INSERT_MODE | MENU_CMDLINE_MODE
    } else {
        // menu
        MENU_NORMAL_MODE | MENU_VISUAL_MODE | MENU_SELECT_MODE | MENU_OP_PENDING_MODE
    }
}

/// Get the string representation of menu modes.
///
/// This is the opposite of `get_menu_cmd_modes`.
#[no_mangle]
pub extern "C" fn rs_get_menu_mode_str(modes: c_int) -> *const c_char {
    const AMENU_MODES: c_int = MENU_INSERT_MODE
        | MENU_CMDLINE_MODE
        | MENU_NORMAL_MODE
        | MENU_VISUAL_MODE
        | MENU_SELECT_MODE
        | MENU_OP_PENDING_MODE;

    const MENU_MODES: c_int =
        MENU_NORMAL_MODE | MENU_VISUAL_MODE | MENU_SELECT_MODE | MENU_OP_PENDING_MODE;

    const MENU_BANG_MODES: c_int = MENU_INSERT_MODE | MENU_CMDLINE_MODE;

    const VMENU_MODES: c_int = MENU_VISUAL_MODE | MENU_SELECT_MODE;

    if (modes & AMENU_MODES) == AMENU_MODES {
        return c"a".as_ptr();
    }
    if (modes & MENU_MODES) == MENU_MODES {
        return c" ".as_ptr();
    }
    if (modes & MENU_BANG_MODES) == MENU_BANG_MODES {
        return c"!".as_ptr();
    }
    if (modes & VMENU_MODES) == VMENU_MODES {
        return c"v".as_ptr();
    }
    if (modes & MENU_VISUAL_MODE) != 0 {
        return c"x".as_ptr();
    }
    if (modes & MENU_SELECT_MODE) != 0 {
        return c"s".as_ptr();
    }
    if (modes & MENU_OP_PENDING_MODE) != 0 {
        return c"o".as_ptr();
    }
    if (modes & MENU_INSERT_MODE) != 0 {
        return c"i".as_ptr();
    }
    if (modes & MENU_TERMINAL_MODE) != 0 {
        return c"tl".as_ptr();
    }
    if (modes & MENU_CMDLINE_MODE) != 0 {
        return c"c".as_ptr();
    }
    if (modes & MENU_NORMAL_MODE) != 0 {
        return c"n".as_ptr();
    }
    if (modes & MENU_TIP_MODE) != 0 {
        return c"t".as_ptr();
    }

    c"".as_ptr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn test_str(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn test_get_menu_cmd_modes_normal() {
        unsafe {
            let result = rs_get_menu_cmd_modes(test_str("nmenu").as_ptr(), false);
            assert_eq!(result.modes, MENU_NORMAL_MODE);
        }
    }

    #[test]
    fn test_get_menu_cmd_modes_visual() {
        unsafe {
            let result = rs_get_menu_cmd_modes(test_str("vmenu").as_ptr(), false);
            assert_eq!(result.modes, MENU_VISUAL_MODE | MENU_SELECT_MODE);

            let result = rs_get_menu_cmd_modes(test_str("xmenu").as_ptr(), false);
            assert_eq!(result.modes, MENU_VISUAL_MODE);
        }
    }

    #[test]
    fn test_get_menu_cmd_modes_insert() {
        unsafe {
            let result = rs_get_menu_cmd_modes(test_str("imenu").as_ptr(), false);
            assert_eq!(result.modes, MENU_INSERT_MODE);
        }
    }

    #[test]
    fn test_get_menu_cmd_modes_amenu() {
        unsafe {
            let result = rs_get_menu_cmd_modes(test_str("amenu").as_ptr(), false);
            assert_eq!(
                result.modes,
                MENU_INSERT_MODE
                    | MENU_CMDLINE_MODE
                    | MENU_NORMAL_MODE
                    | MENU_VISUAL_MODE
                    | MENU_SELECT_MODE
                    | MENU_OP_PENDING_MODE
            );
        }
    }

    #[test]
    fn test_get_menu_cmd_modes_terminal() {
        unsafe {
            let result = rs_get_menu_cmd_modes(test_str("tlmenu").as_ptr(), false);
            assert_eq!(result.modes, MENU_TERMINAL_MODE);

            let result = rs_get_menu_cmd_modes(test_str("tmenu").as_ptr(), false);
            assert_eq!(result.modes, MENU_TIP_MODE);
        }
    }

    #[test]
    fn test_get_menu_cmd_modes_default() {
        unsafe {
            let result = rs_get_menu_cmd_modes(test_str("menu").as_ptr(), false);
            assert_eq!(
                result.modes,
                MENU_NORMAL_MODE | MENU_VISUAL_MODE | MENU_SELECT_MODE | MENU_OP_PENDING_MODE
            );
        }
    }

    #[test]
    fn test_get_menu_cmd_modes_forceit() {
        unsafe {
            let result = rs_get_menu_cmd_modes(test_str("menu").as_ptr(), true);
            assert_eq!(result.modes, MENU_INSERT_MODE | MENU_CMDLINE_MODE);
        }
    }

    #[test]
    fn test_get_menu_mode_str() {
        // Test various mode combinations
        assert!(!rs_get_menu_mode_str(MENU_NORMAL_MODE).is_null());
        assert!(!rs_get_menu_mode_str(MENU_VISUAL_MODE).is_null());
    }
}
