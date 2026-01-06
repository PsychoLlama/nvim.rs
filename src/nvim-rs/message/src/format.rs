//! Message formatting utilities
//!
//! Provides Rust implementations for message formatting operations
//! including string truncation, column calculation, and attribute handling.

use std::ffi::{c_char, c_int};

// C accessor declarations
extern "C" {
    /// Get `msg_col` global
    fn nvim_get_msg_col() -> c_int;
    /// Set `msg_col` global
    fn nvim_set_msg_col(col: c_int);
    /// Get `msg_row` global
    fn nvim_get_msg_row() -> c_int;
    /// Set `msg_row` global
    fn nvim_set_msg_row(row: c_int);
    /// Get `Rows` global (screen rows)
    fn nvim_get_rows() -> c_int;
    /// Get `Columns` global (screen columns)
    fn nvim_get_columns() -> c_int;
    /// Get `msg_scrolled` global
    fn nvim_get_msg_scrolled() -> c_int;
    /// Get `sc_col` global (showcmd column)
    fn nvim_get_sc_col() -> c_int;
    /// Get `msg_scroll` flag
    fn nvim_get_msg_scroll() -> c_int;
    /// Get `need_wait_return` flag
    fn nvim_get_need_wait_return() -> c_int;
    /// Check shortmess option flag
    fn nvim_shortmess(flag: c_int) -> c_int;
    /// Get `exmode_active` flag
    fn nvim_get_exmode_active() -> c_int;
    /// Get `msg_silent` count
    fn nvim_get_msg_silent() -> c_int;
    /// Check if UI has messages capability
    fn nvim_ui_has_messages() -> c_int;
    /// Calculate string width in cells
    fn nvim_vim_strsize(s: *const c_char) -> c_int;
    /// Calculate truncation point (kept for future use)
    #[allow(dead_code)]
    fn nvim_mb_trunc_len(s: *const c_char, width: c_int) -> c_int;
}

/// Get the current message column.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_col() -> c_int {
    nvim_get_msg_col()
}

/// Set the current message column.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_col(col: c_int) {
    nvim_set_msg_col(col);
}

/// Get the current message row.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_row() -> c_int {
    nvim_get_msg_row()
}

/// Set the current message row.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_row(row: c_int) {
    nvim_set_msg_row(row);
}

/// Calculate available room for a message without causing scrolling.
///
/// Returns the number of screen cells available for a message.
/// Takes into account current position and whether scrolling has occurred.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_room() -> c_int {
    let rows = nvim_get_rows();
    let columns = nvim_get_columns();
    let msg_row = nvim_get_msg_row();
    let msg_scrolled = nvim_get_msg_scrolled();
    let sc_col = nvim_get_sc_col();

    if msg_scrolled != 0 {
        // Use all the columns
        (rows - msg_row) * columns - 1
    } else {
        // Use up to 'showcmd' column
        (rows - msg_row - 1) * columns + sc_col - 1
    }
}

/// Check if message truncation should be applied.
///
/// Returns true when:
/// - Not scrolling messages AND
/// - Don't need wait_return AND
/// - Shortmess has truncall flag AND
/// - Not in ex mode AND
/// - Message is silent (0) AND
/// - Not using external messages UI
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_should_truncate() -> c_int {
    const SHM_TRUNCALL: c_int = b'T' as c_int;

    let msg_scroll = nvim_get_msg_scroll() != 0;
    let need_wait_return = nvim_get_need_wait_return() != 0;
    let has_truncall = nvim_shortmess(SHM_TRUNCALL) != 0;
    let exmode_active = nvim_get_exmode_active() != 0;
    let msg_silent = nvim_get_msg_silent();
    let ui_has_messages = nvim_ui_has_messages() != 0;

    c_int::from(
        !msg_scroll
            && !need_wait_return
            && has_truncall
            && !exmode_active
            && msg_silent == 0
            && !ui_has_messages,
    )
}

/// Calculate the width a string would take on screen.
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_strsize(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }
    nvim_vim_strsize(s)
}

/// Check if a message would need truncation.
///
/// Returns true if the string width exceeds the available room.
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_needs_truncation(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }

    let width = nvim_vim_strsize(s);
    let room = rs_msg_room();

    c_int::from(width > room && room > 0)
}

/// Advance the message position to a specific column.
///
/// If the current column is already past the target, inserts a newline first.
/// Otherwise, pads with spaces to reach the target column.
///
/// Returns the number of spaces/newlines that would need to be output.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_advance_spaces(col: c_int) -> c_int {
    let msg_col = nvim_get_msg_col();

    if msg_col >= col {
        // Need to wrap to next line first
        -1 // Signal to output newline first
    } else {
        col - msg_col // Number of spaces to add
    }
}

/// Get the number of screen columns.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_screen_columns() -> c_int {
    nvim_get_columns()
}

/// Get the number of screen rows.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_screen_rows() -> c_int {
    nvim_get_rows()
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
