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

    // Prompt handling
    fn msg_end_prompt();

    // Delay checking
    fn msg_check_for_delay(check_msg_scroll: c_int);

    // UI refresh
    fn msg_ui_refresh();
    fn msg_ui_flush();
    fn msg_scroll_flush();

    // Cursor positioning
    fn msg_cursor_goto(row: c_int, col: c_int);

    // Command line clearing
    fn msg_clr_cmdline();
    fn msg_clr_eos_force();

    // State accessors
    fn nvim_get_msg_silent() -> c_int;
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

/// End a prompt message.
///
/// Called after prompt input to clean up message state.
///
/// # Safety
/// Calls C function that modifies global state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_end_prompt() {
    msg_end_prompt();
}

/// Check if a delay is needed before next message.
///
/// Used to ensure messages are visible before proceeding.
///
/// # Arguments
/// * `check_msg_scroll` - If true, also check msg_scroll state
///
/// # Safety
/// Calls C function that may block.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_check_for_delay(check_msg_scroll: c_int) {
    msg_check_for_delay(check_msg_scroll);
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
/// Calls C function that modifies display state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_clr_cmdline() {
    msg_clr_cmdline();
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
