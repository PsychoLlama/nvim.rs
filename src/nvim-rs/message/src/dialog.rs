//! Dialog and confirmation handling
//!
//! Provides utilities for managing confirmation dialogs and their state.
//! This includes the confirm message display, button parsing, and dialog
//! response handling.

use std::ffi::{c_char, c_int};
use std::ptr;

// C accessor declarations
extern "C" {
    static mut msg_silent: c_int;
    // confirm_msg, confirm_buttons, confirm_msg_used: Rust-owned statics (dialog.rs)
    /// Direct C global: silent_mode
    static silent_mode: bool;
    /// xfree wrapper
    fn xfree(ptr: *mut std::ffi::c_void);
}

// ============================================================================
// Rust-owned statics (previously file-local in message.c)
// ============================================================================

/// Confirm message text (replaces C static confirm_msg)
#[no_mangle]
pub static mut confirm_msg: *mut c_char = std::ptr::null_mut();

/// Confirm buttons text (replaces C static confirm_buttons)
#[no_mangle]
pub static mut confirm_buttons: *mut c_char = std::ptr::null_mut();

/// Number of times confirm message has been displayed (replaces C static confirm_msg_used)
#[no_mangle]
pub static mut confirm_msg_used: c_int = 0;

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
    confirm_msg
}

/// Set the confirm message.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_confirm_msg(msg: *const c_char) {
    confirm_msg = msg.cast_mut();
}

/// Check if confirm message is currently being displayed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_confirm_msg_used() -> c_int {
    confirm_msg_used
}

/// Check if we are in the middle of displaying a confirm message.
///
/// Returns true if confirm_msg_used > 0.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_confirm_msg_used() -> c_int {
    c_int::from(confirm_msg_used > 0)
}

/// Increment the confirm message used counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_confirm_msg_enter() {
    confirm_msg_used += 1;
}

/// Decrement the confirm message used counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_confirm_msg_leave() {
    if confirm_msg_used > 0 {
        confirm_msg_used -= 1;
    }
}

/// Get the current confirm buttons string.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_confirm_buttons() -> *const c_char {
    confirm_buttons
}

/// Set the confirm buttons string.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_confirm_buttons(buttons: *const c_char) {
    confirm_buttons = buttons.cast_mut();
}

/// Check if a confirm message exists.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_has_confirm_msg() -> c_int {
    c_int::from(!confirm_msg.is_null())
}

/// Clear the confirm message, freeing its memory.
///
/// # Safety
/// Calls C accessor functions and frees memory.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_confirm_msg() {
    let msg = confirm_msg;
    if !msg.is_null() {
        xfree(msg.cast());
        confirm_msg = ptr::null_mut();
    }
}

/// Clear the confirm buttons, freeing their memory.
///
/// # Safety
/// Calls C accessor functions and frees memory.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_confirm_buttons() {
    let buttons = confirm_buttons;
    if !buttons.is_null() {
        xfree(buttons.cast());
        confirm_buttons = ptr::null_mut();
    }
}

/// Check if we're in silent mode (no dialogs).
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_silent_mode() -> c_int {
    c_int::from(silent_mode)
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
    /// UI active check
    fn ui_active() -> c_int;
    static mut no_wait_return: c_int;
}

/// Check if dialogs should be suppressed.
///
/// Returns true if silent_mode or msg_silent is set.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_dialogs_suppressed() -> c_int {
    c_int::from(silent_mode || msg_silent != 0)
}

/// Check if dialog can be shown.
///
/// Returns true if not in silent mode.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_dialog_can_show() -> c_int {
    c_int::from(!silent_mode)
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
    no_wait_return += 1;
}

/// Leave dialog context.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_dialog_leave() {
    if no_wait_return > 0 {
        no_wait_return -= 1;
    }
}

/// Check if currently in dialog context.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_in_dialog() -> c_int {
    c_int::from(no_wait_return > 0)
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

// ============================================================================
// Phase 3.3: Dialog Function Wrappers
// ============================================================================

extern "C" {
    /// Call do_dialog() C function
    fn do_dialog(
        dialog_type: c_int,
        title: *const c_char,
        message: *const c_char,
        buttons: *const c_char,
        dfltbutton: c_int,
        textfield: *const c_char,
        ex_cmd: c_int,
    ) -> c_int;

    /// Call vim_dialog_yesno() C function
    fn vim_dialog_yesno(
        dialog_type: c_int,
        title: *const c_char,
        message: *const c_char,
        dflt: c_int,
    ) -> c_int;

    /// Call vim_dialog_yesnocancel() C function
    fn vim_dialog_yesnocancel(
        dialog_type: c_int,
        title: *const c_char,
        message: *const c_char,
        dflt: c_int,
    ) -> c_int;

    /// Call vim_dialog_yesnoallcancel() C function
    fn vim_dialog_yesnoallcancel(
        dialog_type: c_int,
        title: *const c_char,
        message: *const c_char,
        dflt: c_int,
    ) -> c_int;
}

/// Show a dialog and wait for a response.
///
/// This is the main dialog function that handles all types of dialogs.
///
/// # Arguments
/// * `dialog_type` - Type of dialog (VIM_GENERIC, VIM_ERROR, etc.)
/// * `title` - Dialog title (may be NULL)
/// * `message` - Main message text
/// * `buttons` - Newline-separated button names with & hotkey markers
/// * `dfltbutton` - Default button number (1-indexed)
/// * `textfield` - Input field content for inputdialog(), or NULL
/// * `ex_cmd` - If true, pressing : accepts default and starts Ex command
///
/// # Returns
/// * 0 if cancelled
/// * Otherwise the nth button (1-indexed)
///
/// # Safety
/// - `title`, `message`, `buttons` must be valid C strings or NULL
/// - `textfield` must be valid C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_do_dialog(
    dialog_type: c_int,
    title: *const c_char,
    message: *const c_char,
    buttons: *const c_char,
    dfltbutton: c_int,
    textfield: *const c_char,
    ex_cmd: c_int,
) -> c_int {
    do_dialog(
        dialog_type,
        title,
        message,
        buttons,
        dfltbutton,
        textfield,
        ex_cmd,
    )
}

/// Show a Yes/No dialog.
///
/// Convenience function for simple yes/no confirmations.
///
/// # Arguments
/// * `dialog_type` - Type of dialog
/// * `title` - Dialog title (may be NULL, defaults to "Question")
/// * `message` - Question text
/// * `dflt` - Default button (1=Yes, 2=No)
///
/// # Returns
/// * VIM_YES (2) if Yes was selected
/// * VIM_NO (3) if No was selected
///
/// # Safety
/// - `title`, `message` must be valid C strings or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_dialog_yesno(
    dialog_type: c_int,
    title: *const c_char,
    message: *const c_char,
    dflt: c_int,
) -> c_int {
    vim_dialog_yesno(dialog_type, title, message, dflt)
}

/// Show a Yes/No/Cancel dialog.
///
/// Convenience function for confirmations with cancel option.
///
/// # Arguments
/// * `dialog_type` - Type of dialog
/// * `title` - Dialog title (may be NULL, defaults to "Question")
/// * `message` - Question text
/// * `dflt` - Default button (1=Yes, 2=No, 3=Cancel)
///
/// # Returns
/// * VIM_YES (2) if Yes was selected
/// * VIM_NO (3) if No was selected
/// * VIM_CANCEL (4) if Cancel was selected or dialog was cancelled
///
/// # Safety
/// - `title`, `message` must be valid C strings or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_dialog_yesnocancel(
    dialog_type: c_int,
    title: *const c_char,
    message: *const c_char,
    dflt: c_int,
) -> c_int {
    vim_dialog_yesnocancel(dialog_type, title, message, dflt)
}

/// Show a Yes/No/All/Discard All/Cancel dialog.
///
/// Convenience function for batch operations with multiple options.
///
/// # Arguments
/// * `dialog_type` - Type of dialog
/// * `title` - Dialog title (may be NULL, defaults to "Question")
/// * `message` - Question text
/// * `dflt` - Default button (1=Yes, 2=No, 3=All, 4=Discard All, 5=Cancel)
///
/// # Returns
/// * VIM_YES (2) if Yes was selected
/// * VIM_NO (3) if No was selected
/// * VIM_ALL (5) if Save All was selected
/// * VIM_DISCARDALL (6) if Discard All was selected
/// * VIM_CANCEL (4) if Cancel was selected or dialog was cancelled
///
/// # Safety
/// - `title`, `message` must be valid C strings or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_dialog_yesnoallcancel(
    dialog_type: c_int,
    title: *const c_char,
    message: *const c_char,
    dflt: c_int,
) -> c_int {
    vim_dialog_yesnoallcancel(dialog_type, title, message, dflt)
}

/// Quick yes/no dialog with default type.
///
/// Uses VIM_QUESTION type and defaults to Yes.
///
/// # Safety
/// - `message` must be a valid C string
#[no_mangle]
pub unsafe extern "C" fn rs_confirm_yesno(message: *const c_char) -> c_int {
    vim_dialog_yesno(DialogType::QUESTION.0, ptr::null(), message, 1)
}

/// Quick yes/no/cancel dialog with default type.
///
/// Uses VIM_QUESTION type and defaults to Yes.
///
/// # Safety
/// - `message` must be a valid C string
#[no_mangle]
pub unsafe extern "C" fn rs_confirm_yesnocancel(message: *const c_char) -> c_int {
    vim_dialog_yesnocancel(DialogType::QUESTION.0, ptr::null(), message, 1)
}

/// Check if dialog result was affirmative (Yes or All).
///
/// # Arguments
/// * `result` - Result from dialog function
///
/// # Returns
/// * 1 if result was Yes or All
/// * 0 otherwise
#[no_mangle]
pub const extern "C" fn rs_dialog_affirmed(result: c_int) -> c_int {
    if result == DialogResponse::YES.0 || result == DialogResponse::ALL.0 {
        1
    } else {
        0
    }
}

/// Convert dialog result to boolean (true if Yes/All, false otherwise).
///
/// # Arguments
/// * `result` - Result from dialog function
///
/// # Returns
/// * 1 if result indicates acceptance
/// * 0 if result indicates rejection or cancellation
#[no_mangle]
pub const extern "C" fn rs_dialog_to_bool(result: c_int) -> c_int {
    rs_dialog_affirmed(result)
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
