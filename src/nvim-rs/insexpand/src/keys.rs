//! Completion key handling support.
//!
//! This module provides helper functions for handling keys during completion.
//! The actual key processing remains in C, but Rust provides utilities.

use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_compl_started() -> c_int;
}

// Key constants (from keycode definitions)
const CTRL_N: c_int = 14; // ^N
const CTRL_P: c_int = 16; // ^P
const CTRL_Y: c_int = 25; // ^Y
const CTRL_E: c_int = 5; // ^E
const BS: c_int = 8; // Backspace
const TAB: c_int = 9; // Tab
const CR: c_int = 13; // Carriage return / Enter
const ESC: c_int = 27; // Escape

/// Check if a key should navigate forward in completion.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_is_forward_key(key: c_int) -> c_int {
    c_int::from(key == CTRL_N || key == TAB)
}

/// Check if a key should navigate backward in completion.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_is_backward_key(key: c_int) -> c_int {
    c_int::from(key == CTRL_P)
}

/// Check if a key should accept the current completion.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_is_accept_key(key: c_int) -> c_int {
    c_int::from(key == CTRL_Y || key == CR)
}

/// Check if a key should cancel completion.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_is_cancel_key(key: c_int) -> c_int {
    c_int::from(key == CTRL_E || key == ESC)
}

/// Check if a key is backspace.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_is_bs_key(key: c_int) -> c_int {
    c_int::from(key == BS)
}

/// Check if a key should trigger completion restart.
///
/// Returns true for printable characters that should restart completion.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_is_printable(key: c_int) -> c_int {
    // Basic ASCII printable range (space to ~)
    c_int::from((32..=126).contains(&key))
}

/// Check if completion is active and should handle the key.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_should_handle() -> c_int {
    nvim_get_compl_started()
}

// =============================================================================
// Phase 7: Extended Key Handling Functions
// =============================================================================

// Additional C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_col() -> c_int;
    fn nvim_get_compl_length() -> c_int;
    fn nvim_get_cursor_col() -> c_int;
}

// CTRL-X mode constants
const CTRL_X_OMNI: c_int = 13;
const CTRL_X_EVAL: c_int = 16;

/// Check if we're in omni completion mode.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_ctrl_x_mode_omni() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_OMNI)
}

/// Check if we're in eval completion mode.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_ctrl_x_mode_eval() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_EVAL)
}

/// Check if backspace should stop completion.
///
/// Returns true if the backspace would delete past the completion start,
/// which should stop completion.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_bs_should_stop(new_col: c_int) -> c_int {
    let compl_col = nvim_get_compl_col();
    let mode = nvim_get_ctrl_x_mode();

    // Stop if we'd delete past the completion column
    if new_col < compl_col {
        return 1;
    }

    // Stop if we'd delete exactly to the completion column, except for omni
    if new_col == compl_col && mode != CTRL_X_OMNI {
        return 1;
    }

    // Stop for eval mode regardless of position
    if mode == CTRL_X_EVAL {
        return 1;
    }

    0
}

/// Check if backspace should restart completion.
///
/// Returns true if cursor is at or before the end of typed completion text.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_bs_should_restart(new_col: c_int) -> c_int {
    let compl_col = nvim_get_compl_col();
    let compl_length = nvim_get_compl_length();
    c_int::from(new_col <= compl_col + compl_length)
}

/// Calculate how much typed text remains after backspace.
///
/// Returns the number of characters remaining from the completion start.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_bs_remaining_len(new_col: c_int) -> c_int {
    let compl_col = nvim_get_compl_col();
    let len = new_col - compl_col;
    if len < 0 {
        0
    } else {
        len
    }
}

/// Check if a key is a delete key (backspace or delete).
#[no_mangle]
pub extern "C" fn rs_keys_is_delete_key(key: c_int) -> c_int {
    // DEL = 127
    c_int::from(key == BS || key == 127)
}

/// Check if a key modifies the typed text.
///
/// Returns true for keys that change the completion pattern (printable or delete).
#[no_mangle]
pub extern "C" fn rs_keys_modifies_text(key: c_int) -> c_int {
    // Printable characters or backspace/delete
    c_int::from((32..=126).contains(&key) || key == BS || key == 127)
}

/// Check if a key is a navigation key (up/down/page keys).
#[no_mangle]
pub extern "C" fn rs_keys_is_nav_key(key: c_int) -> c_int {
    c_int::from(
        key == CTRL_N
            || key == CTRL_P
            || key == K_UP
            || key == K_DOWN
            || key == K_PAGEUP
            || key == K_PAGEDOWN
            || key == K_S_UP
            || key == K_S_DOWN,
    )
}

/// Check if a key should scroll the popup menu.
#[no_mangle]
pub extern "C" fn rs_keys_is_scroll_key(key: c_int) -> c_int {
    c_int::from(key == K_PAGEUP || key == K_PAGEDOWN || key == K_S_UP || key == K_S_DOWN)
}

/// Get the key action type for completion.
///
/// Returns:
/// - 0: Unknown/other
/// - 1: Navigation (up/down)
/// - 2: Accept (ctrl-y, enter)
/// - 3: Cancel (ctrl-e, escape)
/// - 4: Delete (backspace, delete)
/// - 5: Printable (adds to pattern)
#[no_mangle]
pub extern "C" fn rs_keys_action_type(key: c_int) -> c_int {
    if key == CTRL_N || key == CTRL_P || key == K_UP || key == K_DOWN {
        return 1; // Navigation
    }
    if key == CTRL_Y || key == CR {
        return 2; // Accept
    }
    if key == CTRL_E || key == ESC {
        return 3; // Cancel
    }
    if key == BS || key == 127 {
        return 4; // Delete
    }
    if (32..=126).contains(&key) {
        return 5; // Printable
    }
    0 // Unknown
}

/// Calculate completion offset after a character is typed.
///
/// The offset is cursor column minus completion start column.
#[no_mangle]
pub unsafe extern "C" fn rs_keys_compl_offset() -> c_int {
    let cursor_col = nvim_get_cursor_col();
    let compl_col = nvim_get_compl_col();
    let offset = cursor_col - compl_col;
    if offset < 0 {
        0
    } else {
        offset
    }
}

// Special key codes (from keycode definitions)
const K_UP: c_int = -42; // Placeholder - actual values from keycodes.h
const K_DOWN: c_int = -43;
const K_PAGEUP: c_int = -48;
const K_PAGEDOWN: c_int = -49;
const K_S_UP: c_int = -50;
const K_S_DOWN: c_int = -51;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_constants() {
        assert_eq!(CTRL_N, 14);
        assert_eq!(CTRL_P, 16);
        assert_eq!(CTRL_Y, 25);
        assert_eq!(CTRL_E, 5);
        assert_eq!(BS, 8);
        assert_eq!(TAB, 9);
        assert_eq!(CR, 13);
        assert_eq!(ESC, 27);
    }

    #[test]
    fn test_forward_key() {
        unsafe {
            assert_eq!(rs_keys_is_forward_key(CTRL_N), 1);
            assert_eq!(rs_keys_is_forward_key(TAB), 1);
            assert_eq!(rs_keys_is_forward_key(CTRL_P), 0);
        }
    }

    #[test]
    fn test_backward_key() {
        unsafe {
            assert_eq!(rs_keys_is_backward_key(CTRL_P), 1);
            assert_eq!(rs_keys_is_backward_key(CTRL_N), 0);
        }
    }

    #[test]
    fn test_accept_key() {
        unsafe {
            assert_eq!(rs_keys_is_accept_key(CTRL_Y), 1);
            assert_eq!(rs_keys_is_accept_key(CR), 1);
            assert_eq!(rs_keys_is_accept_key(ESC), 0);
        }
    }

    #[test]
    fn test_cancel_key() {
        unsafe {
            assert_eq!(rs_keys_is_cancel_key(CTRL_E), 1);
            assert_eq!(rs_keys_is_cancel_key(ESC), 1);
            assert_eq!(rs_keys_is_cancel_key(CTRL_Y), 0);
        }
    }

    #[test]
    fn test_printable() {
        unsafe {
            assert_eq!(rs_keys_is_printable(65), 1); // 'A'
            assert_eq!(rs_keys_is_printable(32), 1); // space
            assert_eq!(rs_keys_is_printable(10), 0); // newline
        }
    }
}
