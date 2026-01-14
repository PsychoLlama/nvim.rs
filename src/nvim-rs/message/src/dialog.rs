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

/// Dialog type values (matches VIM_* in C)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogType(pub c_int);

impl DialogType {
    /// Generic dialog
    pub const GENERIC: Self = Self(0);
    /// Error dialog
    pub const ERROR: Self = Self(1);
    /// Warning dialog
    pub const WARNING: Self = Self(2);
    /// Info dialog
    pub const INFO: Self = Self(3);
    /// Question dialog
    pub const QUESTION: Self = Self(4);
}

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

/// Get the VIM_GENERIC dialog type value.
#[no_mangle]
pub const extern "C" fn rs_dialog_type_generic() -> c_int {
    DialogType::GENERIC.0
}

/// Get the VIM_ERROR dialog type value.
#[no_mangle]
pub const extern "C" fn rs_dialog_type_error() -> c_int {
    DialogType::ERROR.0
}

/// Get the VIM_WARNING dialog type value.
#[no_mangle]
pub const extern "C" fn rs_dialog_type_warning() -> c_int {
    DialogType::WARNING.0
}

/// Get the VIM_INFO dialog type value.
#[no_mangle]
pub const extern "C" fn rs_dialog_type_info() -> c_int {
    DialogType::INFO.0
}

/// Get the VIM_QUESTION dialog type value.
#[no_mangle]
pub const extern "C" fn rs_dialog_type_question() -> c_int {
    DialogType::QUESTION.0
}

/// Check if a dialog type is an error or warning.
#[no_mangle]
pub const extern "C" fn rs_dialog_is_error_or_warning(dialog_type: c_int) -> c_int {
    if dialog_type == DialogType::ERROR.0 || dialog_type == DialogType::WARNING.0 {
        1
    } else {
        0
    }
}

// ============================================================================
// Phase 428: Additional Dialog System Functions
// ============================================================================

extern "C" {
    /// Get `msg_silent` flag
    fn nvim_get_msg_silent() -> c_int;
    /// UI active check
    fn ui_active() -> c_int;
    /// Get `no_wait_return` counter
    fn no_wait_return_get() -> c_int;
    /// Increment `no_wait_return`
    fn no_wait_return_inc();
    /// Decrement `no_wait_return`
    fn no_wait_return_dec();
}

/// Check if dialogs should be suppressed.
///
/// Returns true if silent_mode or msg_silent is set.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_dialogs_suppressed() -> c_int {
    let silent_mode = nvim_get_silent_mode();
    let msg_silent = nvim_get_msg_silent();
    c_int::from(silent_mode != 0 || msg_silent != 0)
}

/// Check if dialog can be shown.
///
/// Returns true if not in silent mode.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_dialog_can_show() -> c_int {
    c_int::from(nvim_get_silent_mode() == 0)
}

/// Check if UI is ready for dialogs.
///
/// Returns true if a UI is active.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_dialog_ui_ready() -> c_int {
    ui_active()
}

/// Enter dialog context - suppress wait_return.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_dialog_enter() {
    no_wait_return_inc();
}

/// Leave dialog context.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_dialog_leave() {
    no_wait_return_dec();
}

/// Check if currently in dialog context.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_in_dialog() -> c_int {
    c_int::from(no_wait_return_get() > 0)
}

/// Clear all dialog state.
///
/// Frees confirm_msg and confirm_buttons.
///
/// # Safety
/// Calls C accessor functions and frees memory.
#[no_mangle]
pub unsafe extern "C" fn rs_dialog_clear_all() {
    rs_clear_confirm_msg();
    rs_clear_confirm_buttons();
}

/// Check if dialog response indicates acceptance.
///
/// Returns true for YES or ALL.
#[no_mangle]
pub const extern "C" fn rs_dialog_response_accept(response: c_int) -> c_int {
    if response == DialogResponse::YES.0 || response == DialogResponse::ALL.0 {
        1
    } else {
        0
    }
}

/// Check if dialog response indicates rejection.
///
/// Returns true for NO or DISCARD_ALL.
#[no_mangle]
pub const extern "C" fn rs_dialog_response_reject(response: c_int) -> c_int {
    if response == DialogResponse::NO.0 || response == DialogResponse::DISCARD_ALL.0 {
        1
    } else {
        0
    }
}

/// Check if dialog response indicates cancellation.
///
/// Returns true for CANCEL or 0 (cancelled).
#[no_mangle]
pub const extern "C" fn rs_dialog_response_cancel(response: c_int) -> c_int {
    if response == DialogResponse::CANCEL.0 || response == 0 {
        1
    } else {
        0
    }
}

/// Check if dialog response applies to all items.
///
/// Returns true for ALL or DISCARD_ALL.
#[no_mangle]
pub const extern "C" fn rs_dialog_response_all(response: c_int) -> c_int {
    if response == DialogResponse::ALL.0 || response == DialogResponse::DISCARD_ALL.0 {
        1
    } else {
        0
    }
}

/// Count hotkeys in a button string.
///
/// Counts occurrences of the hotkey marker character.
///
/// # Safety
/// `buttons` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_count_dialog_hotkeys(buttons: *const c_char) -> c_int {
    if buttons.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut p = buttons;

    while *p != 0 {
        if c_int::from(*p) == DLG_HOTKEY_CHAR {
            count += 1;
        }
        p = p.add(1);
    }

    count
}

/// Check if all buttons in a string have explicit hotkeys.
///
/// # Safety
/// `buttons` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_dialog_all_have_hotkeys(buttons: *const c_char) -> c_int {
    let button_count = rs_count_dialog_buttons(buttons);
    let hotkey_count = rs_count_dialog_hotkeys(buttons);
    c_int::from(button_count > 0 && hotkey_count >= button_count)
}

/// Maximum buttons with tracked hotkey state.
pub const HAS_HOTKEY_LEN: c_int = 30;

/// Maximum bytes for a single hotkey character.
pub const HOTK_LEN: c_int = 6; // MB_MAXBYTES

/// Get the HAS_HOTKEY_LEN constant.
#[no_mangle]
pub const extern "C" fn rs_has_hotkey_len() -> c_int {
    HAS_HOTKEY_LEN
}

/// Get the HOTK_LEN constant.
#[no_mangle]
pub const extern "C" fn rs_hotk_len() -> c_int {
    HOTK_LEN
}

/// Calculate memory needed for hotkey storage.
///
/// Returns (button_count * HOTK_LEN) + 1 for NUL.
#[no_mangle]
pub const extern "C" fn rs_dialog_hotkey_bufsize(button_count: c_int) -> c_int {
    if button_count <= 0 {
        1 // Just NUL
    } else {
        button_count * HOTK_LEN + 1
    }
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
    fn test_dialog_type_values() {
        assert_eq!(DialogType::GENERIC.0, 0);
        assert_eq!(DialogType::ERROR.0, 1);
        assert_eq!(DialogType::WARNING.0, 2);
        assert_eq!(DialogType::INFO.0, 3);
        assert_eq!(DialogType::QUESTION.0, 4);
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

    #[test]
    fn test_dialog_response_accept() {
        assert_eq!(rs_dialog_response_accept(DialogResponse::YES.0), 1);
        assert_eq!(rs_dialog_response_accept(DialogResponse::ALL.0), 1);
        assert_eq!(rs_dialog_response_accept(DialogResponse::NO.0), 0);
        assert_eq!(rs_dialog_response_accept(DialogResponse::CANCEL.0), 0);
    }

    #[test]
    fn test_dialog_response_reject() {
        assert_eq!(rs_dialog_response_reject(DialogResponse::NO.0), 1);
        assert_eq!(rs_dialog_response_reject(DialogResponse::DISCARD_ALL.0), 1);
        assert_eq!(rs_dialog_response_reject(DialogResponse::YES.0), 0);
    }

    #[test]
    fn test_dialog_response_cancel() {
        assert_eq!(rs_dialog_response_cancel(DialogResponse::CANCEL.0), 1);
        assert_eq!(rs_dialog_response_cancel(0), 1);
        assert_eq!(rs_dialog_response_cancel(DialogResponse::YES.0), 0);
    }

    #[test]
    fn test_dialog_response_all() {
        assert_eq!(rs_dialog_response_all(DialogResponse::ALL.0), 1);
        assert_eq!(rs_dialog_response_all(DialogResponse::DISCARD_ALL.0), 1);
        assert_eq!(rs_dialog_response_all(DialogResponse::YES.0), 0);
    }

    #[test]
    fn test_dialog_hotkey_bufsize() {
        assert_eq!(rs_dialog_hotkey_bufsize(0), 1);
        assert_eq!(rs_dialog_hotkey_bufsize(1), HOTK_LEN + 1);
        assert_eq!(rs_dialog_hotkey_bufsize(3), 3 * HOTK_LEN + 1);
    }

    #[test]
    fn test_dialog_constants_phase428() {
        assert_eq!(HAS_HOTKEY_LEN, 30);
        assert_eq!(HOTK_LEN, 6);
        assert_eq!(rs_has_hotkey_len(), 30);
        assert_eq!(rs_hotk_len(), 6);
    }
}
