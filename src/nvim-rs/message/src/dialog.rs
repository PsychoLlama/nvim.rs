//! Dialog and confirmation handling
//!
//! Provides utilities for managing confirmation dialogs and their state.
//! This includes the confirm message display, button parsing, and dialog
//! response handling.

use std::ffi::{c_char, c_int};
use std::ptr;

// C accessor declarations
extern "C" {
    /// Get `confirm_msg` pointer
    fn nvim_get_confirm_msg() -> *const c_char;
    /// Set `confirm_msg` pointer
    fn nvim_set_confirm_msg(msg: *const c_char);
    /// Get `confirm_msg_used` counter
    fn nvim_get_confirm_msg_used() -> c_int;
    /// Set `confirm_msg_used` counter
    fn nvim_set_confirm_msg_used(val: c_int);
    /// Get `confirm_buttons` pointer
    fn nvim_get_confirm_buttons() -> *const c_char;
    /// Set `confirm_buttons` pointer
    fn nvim_set_confirm_buttons(buttons: *const c_char);
    /// Get `silent_mode` flag
    fn nvim_get_silent_mode() -> c_int;
    /// xfree wrapper
    fn xfree(ptr: *mut std::ffi::c_void);
}

/// Dialog button separator character (as c_int for comparison)
pub const DLG_BUTTON_SEP: c_int = b'\n' as c_int;

/// Dialog hotkey marker character (as c_int for comparison)
pub const DLG_HOTKEY_CHAR: c_int = b'&' as c_int;

/// Dialog response values (matches VIM_* in C)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogResponse(pub c_int);

impl DialogResponse {
    /// Yes response
    pub const YES: Self = Self(2);
    /// No response
    pub const NO: Self = Self(3);
    /// Cancel response
    pub const CANCEL: Self = Self(4);
    /// All response
    pub const ALL: Self = Self(5);
    /// Discard all response
    pub const DISCARD_ALL: Self = Self(6);
}

/// Get the current confirm message.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_confirm_msg() -> *const c_char {
    nvim_get_confirm_msg()
}

/// Set the confirm message.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_confirm_msg(msg: *const c_char) {
    nvim_set_confirm_msg(msg);
}

/// Check if confirm message is currently being displayed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_confirm_msg_used() -> c_int {
    nvim_get_confirm_msg_used()
}

/// Check if we are in the middle of displaying a confirm message.
///
/// Returns true if confirm_msg_used > 0.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_confirm_msg_used() -> c_int {
    c_int::from(nvim_get_confirm_msg_used() > 0)
}

/// Increment the confirm message used counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_confirm_msg_enter() {
    let val = nvim_get_confirm_msg_used();
    nvim_set_confirm_msg_used(val + 1);
}

/// Decrement the confirm message used counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_confirm_msg_leave() {
    let val = nvim_get_confirm_msg_used();
    if val > 0 {
        nvim_set_confirm_msg_used(val - 1);
    }
}

/// Get the current confirm buttons string.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_confirm_buttons() -> *const c_char {
    nvim_get_confirm_buttons()
}

/// Set the confirm buttons string.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_confirm_buttons(buttons: *const c_char) {
    nvim_set_confirm_buttons(buttons);
}

/// Check if a confirm message exists.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_has_confirm_msg() -> c_int {
    c_int::from(!nvim_get_confirm_msg().is_null())
}

/// Clear the confirm message, freeing its memory.
///
/// # Safety
/// Calls C accessor functions and frees memory.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_confirm_msg() {
    let msg = nvim_get_confirm_msg();
    if !msg.is_null() {
        xfree(msg as *mut std::ffi::c_void);
        nvim_set_confirm_msg(ptr::null());
    }
}

/// Clear the confirm buttons, freeing their memory.
///
/// # Safety
/// Calls C accessor functions and frees memory.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_confirm_buttons() {
    let buttons = nvim_get_confirm_buttons();
    if !buttons.is_null() {
        xfree(buttons as *mut std::ffi::c_void);
        nvim_set_confirm_buttons(ptr::null());
    }
}

/// Check if we're in silent mode (no dialogs).
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_silent_mode() -> c_int {
    nvim_get_silent_mode()
}

/// Check if a character is a dialog button separator.
#[no_mangle]
pub const extern "C" fn rs_is_dialog_button_sep(c: c_int) -> c_int {
    (c == DLG_BUTTON_SEP) as c_int
}

/// Check if a character is a dialog hotkey marker.
#[no_mangle]
pub const extern "C" fn rs_is_dialog_hotkey_char(c: c_int) -> c_int {
    (c == DLG_HOTKEY_CHAR) as c_int
}

/// Get the dialog button separator character.
#[no_mangle]
pub const extern "C" fn rs_dialog_button_sep() -> c_int {
    DLG_BUTTON_SEP
}

/// Get the dialog hotkey marker character.
#[no_mangle]
pub const extern "C" fn rs_dialog_hotkey_char() -> c_int {
    DLG_HOTKEY_CHAR
}

/// Get the VIM_YES response value.
#[no_mangle]
pub const extern "C" fn rs_dialog_yes() -> c_int {
    DialogResponse::YES.0
}

/// Get the VIM_NO response value.
#[no_mangle]
pub const extern "C" fn rs_dialog_no() -> c_int {
    DialogResponse::NO.0
}

/// Get the VIM_CANCEL response value.
#[no_mangle]
pub const extern "C" fn rs_dialog_cancel() -> c_int {
    DialogResponse::CANCEL.0
}

/// Get the VIM_ALL response value.
#[no_mangle]
pub const extern "C" fn rs_dialog_all() -> c_int {
    DialogResponse::ALL.0
}

/// Get the VIM_DISCARDALL response value.
#[no_mangle]
pub const extern "C" fn rs_dialog_discard_all() -> c_int {
    DialogResponse::DISCARD_ALL.0
}

/// Count the number of buttons in a button string.
///
/// Buttons are separated by `DLG_BUTTON_SEP` (newline).
///
/// # Safety
/// `buttons` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_count_dialog_buttons(buttons: *const c_char) -> c_int {
    if buttons.is_null() {
        return 0;
    }

    let mut count = 1; // At least one button if string is non-empty
    let mut p = buttons;

    // Check if string is empty
    if *p == 0 {
        return 0;
    }

    while *p != 0 {
        if c_int::from(*p) == DLG_BUTTON_SEP {
            count += 1;
        }
        p = p.add(1);
    }

    count
}

/// Find the position of the nth button in a button string.
///
/// # Arguments
/// * `buttons` - Button string with buttons separated by newlines
/// * `n` - Button number (1-indexed)
///
/// # Returns
/// Pointer to the start of the nth button, or null if not found.
///
/// # Safety
/// `buttons` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_find_dialog_button(buttons: *const c_char, n: c_int) -> *const c_char {
    if buttons.is_null() || n < 1 {
        return ptr::null();
    }

    let mut button_num = 1;
    let mut p = buttons;

    // First button starts at the beginning
    if n == 1 {
        return buttons;
    }

    while *p != 0 {
        if c_int::from(*p) == DLG_BUTTON_SEP {
            button_num += 1;
            if button_num == n {
                // Return pointer to character after separator
                return p.add(1);
            }
        }
        p = p.add(1);
    }

    ptr::null()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_constants() {
        assert_eq!(DLG_BUTTON_SEP, c_int::from(b'\n'));
        assert_eq!(DLG_HOTKEY_CHAR, c_int::from(b'&'));
    }

    #[test]
    fn test_dialog_response_values() {
        assert_eq!(DialogResponse::YES.0, 2);
        assert_eq!(DialogResponse::NO.0, 3);
        assert_eq!(DialogResponse::CANCEL.0, 4);
        assert_eq!(DialogResponse::ALL.0, 5);
        assert_eq!(DialogResponse::DISCARD_ALL.0, 6);
    }

    #[test]
    fn test_is_dialog_button_sep() {
        assert_eq!(rs_is_dialog_button_sep(c_int::from(b'\n')), 1);
        assert_eq!(rs_is_dialog_button_sep(c_int::from(b'&')), 0);
        assert_eq!(rs_is_dialog_button_sep(c_int::from(b'a')), 0);
    }

    #[test]
    fn test_is_dialog_hotkey_char() {
        assert_eq!(rs_is_dialog_hotkey_char(c_int::from(b'&')), 1);
        assert_eq!(rs_is_dialog_hotkey_char(c_int::from(b'\n')), 0);
        assert_eq!(rs_is_dialog_hotkey_char(c_int::from(b'a')), 0);
    }
}
