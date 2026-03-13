//! Miscellaneous message functions
//!
//! Provides Rust implementations for various message-related utilities
//! that don't fit neatly into other modules.

use std::ffi::{c_char, c_int};

// ============================================================================
// C Function Declarations
// ============================================================================

extern "C" {
    // Output translation functions (basic versions in format.rs)
    fn msg_outtrans_long(longstr: *const c_char, hl_id: c_int);

    // Home directory handling
    fn msg_home_replace(fname: *const c_char);

    // Note: msg_source is wrapped in error.rs

    // UI refresh
    fn msg_ui_refresh();
    fn msg_ui_flush();
    fn msg_scroll_flush();

    // For msg_clr_cmdline
    fn nvim_get_cmdline_row() -> c_int;
    fn nvim_set_msg_row(val: c_int);
    fn nvim_set_msg_col(val: c_int);
    fn msg_clr_eos_force();

    // Cursor positioning
    fn msg_cursor_goto(row: c_int, col: c_int);

    // State accessors
    fn nvim_get_msg_silent() -> c_int;
    fn nvim_get_emsg_on_display() -> c_int;
    fn nvim_set_emsg_on_display(val: c_int);
    fn nvim_get_msg_scroll() -> c_int;
    fn nvim_set_msg_scroll(val: c_int);
    fn nvim_get_did_wait_return() -> c_int;
    fn nvim_get_emsg_silent() -> c_int;
    // nvim_get_in_assert_fails returns bool (defined in normal_shim.c)
    fn nvim_get_in_assert_fails() -> bool;
    fn nvim_ui_has_messages() -> c_int;
    // nvim_ui_flush is defined in change_ffi.c
    fn nvim_ui_flush();
    // nvim_os_delay is defined in change_ffi.c (long ms, bool allow_input)
    fn nvim_os_delay(ms: std::ffi::c_long, allow_input: bool);

    // keep_msg raw setters (nvim_set_keep_msg_raw in message.c does xfree+xstrdup)
    fn nvim_set_keep_msg_raw(s: *const c_char);
    fn nvim_set_keep_msg_more(val: c_int);
    fn nvim_set_keep_msg_hl_id(val: c_int);
}

// ============================================================================
// Output Translation Functions
// ============================================================================

// Note: rs_msg_outtrans() and rs_msg_outtrans_len() are defined in format.rs

/// Output a potentially long string with truncation.
///
/// If the string is too long for the screen, shows "..." at the end.
///
/// # Arguments
/// * `longstr` - The string to output
/// * `hl_id` - Highlight group ID
///
/// # Safety
/// - `longstr` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_msg_outtrans_long(longstr: *const c_char, hl_id: c_int) {
    msg_outtrans_long(longstr, hl_id);
}

// ============================================================================
// Path Display Functions
// ============================================================================

/// Display a filename with home directory replaced by ~.
///
/// # Arguments
/// * `fname` - The filename to display
///
/// # Safety
/// - `fname` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_msg_home_replace(fname: *const c_char) {
    msg_home_replace(fname);
}

// ============================================================================
// Source Location Functions
// ============================================================================

// Note: rs_msg_source() is defined in error.rs

// ============================================================================
// Prompt and Delay Functions
// ============================================================================

/// Check if a delay is needed before next message.
///
/// Used to ensure messages are visible before proceeding.
///
/// # Arguments
/// * `check_msg_scroll` - If true, also check msg_scroll state
///
/// # Safety
/// Calls C accessor functions and may block.
#[export_name = "msg_check_for_delay"]
pub unsafe extern "C" fn rs_msg_check_for_delay(check_msg_scroll: c_int) {
    let check = check_msg_scroll != 0;
    if (nvim_get_emsg_on_display() != 0 || (check && nvim_get_msg_scroll() != 0))
        && nvim_get_did_wait_return() == 0
        && nvim_get_emsg_silent() == 0
        && !nvim_get_in_assert_fails()
        && nvim_ui_has_messages() == 0
    {
        nvim_ui_flush();
        nvim_os_delay(1006, true);
        nvim_set_emsg_on_display(0);
        if check {
            nvim_set_msg_scroll(0);
        }
    }
}

// ============================================================================
// Keep Message Functions
// ============================================================================

/// Set the "keep_msg" string that is re-displayed after redraw.
///
/// Frees the old value. Skips when ext_messages UI is active.
/// Sets keep_msg_more to false and updates highlight.
///
/// # Arguments
/// * `s` - The message string, or NULL to clear
/// * `hl_id` - Highlight group ID for the message
///
/// # Safety
/// Calls C accessor/mutator functions that manage allocated memory.
#[export_name = "set_keep_msg"]
pub unsafe extern "C" fn rs_set_keep_msg(s: *const c_char, hl_id: c_int) {
    // Kept message is not cleared and re-emitted with ext_messages: #20416.
    if nvim_ui_has_messages() != 0 {
        return;
    }

    if s.is_null() || nvim_get_msg_silent() != 0 {
        nvim_set_keep_msg_raw(std::ptr::null());
    } else {
        nvim_set_keep_msg_raw(s);
    }
    nvim_set_keep_msg_more(0);
    nvim_set_keep_msg_hl_id(hl_id);
}

// ============================================================================
// UI Coordination Functions
// ============================================================================

/// Refresh the message area UI.
///
/// # Safety
/// Calls C function that modifies UI state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ui_refresh() {
    msg_ui_refresh();
}

/// Flush pending UI updates for messages.
///
/// # Safety
/// Calls C function that emits UI events.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ui_flush() {
    msg_ui_flush();
}

/// Flush scroll-related UI updates.
///
/// # Safety
/// Calls C function that modifies UI state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_scroll_flush() {
    msg_scroll_flush();
}

// Note: rs_msg_reset_scroll() is defined in scrollback.rs

// ============================================================================
// Cursor Functions
// ============================================================================

/// Position the cursor in the message area.
///
/// # Arguments
/// * `row` - Target row
/// * `col` - Target column
///
/// # Safety
/// Calls C function that modifies display state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_cursor_goto(row: c_int, col: c_int) {
    msg_cursor_goto(row, col);
}

// ============================================================================
// Clearing Functions
// ============================================================================

/// Clear the command line area.
///
/// # Safety
/// Calls C accessor functions that modify display state.
#[export_name = "msg_clr_cmdline"]
pub unsafe extern "C" fn rs_msg_clr_cmdline() {
    nvim_set_msg_row(nvim_get_cmdline_row());
    nvim_set_msg_col(0);
    msg_clr_eos_force();
}

/// Force clear to end of screen even if not needed.
///
/// # Safety
/// Calls C function that modifies display state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_clr_eos_force() {
    msg_clr_eos_force();
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Check if message output should be suppressed.
///
/// Returns true if msg_silent is set.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_should_suppress() -> c_int {
    c_int::from(nvim_get_msg_silent() != 0)
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
