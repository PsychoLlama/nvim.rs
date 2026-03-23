//! Warning and informational message handling
//!
//! Provides Rust implementations for warning messages, info messages,
//! and related display functions.

use std::ffi::{c_char, c_int};

// C function declarations
extern "C" {
    static Columns: c_int;
    static mut msg_silent: c_int;
    // State accessors
    static mut msg_col: c_int;

    // For give_warning
    static mut no_wait_return: c_int;
    fn nvim_set_vim_var_warningmsg(s: *const c_char);
    fn nvim_set_keep_msg_raw(s: *const c_char);
    static mut keep_msg_hl_id: c_int;
    fn nvim_msg_ext_kind_is_null() -> c_int;
    fn nvim_set_msg_ext_kind(kind: *const c_char);
    fn msg_ext_ui_flush();
    fn msg(s: *const c_char, hl_id: c_int) -> bool;
    fn set_keep_msg(s: *const c_char, hl_id: c_int);
    static mut msg_scrolled: c_int;
    static mut msg_didout: bool;
    static mut msg_nowait: bool;
}

/// Highlight face for warning messages (HLF_W = 26)
const HLF_W: c_int = 26;

/// "wmsg" kind string for ext_messages
const WMSG_KIND: &std::ffi::CStr = c"wmsg";

// ============================================================================
// Warning Message Functions
// ============================================================================

/// Display a warning message (for searching).
///
/// Use 'w' highlighting and may repeat the message after redrawing.
/// Does nothing if msg_silent is set.
///
/// # Arguments
/// * `message` - The warning message string (NUL-terminated)
/// * `hl` - If true, use warning highlight (HLF_W)
///
/// # Safety
/// - `message` must be a valid NUL-terminated C string
#[export_name = "give_warning"]
pub unsafe extern "C" fn rs_give_warning(message: *const c_char, hl: c_int) {
    // Don't do this for ":silent".
    if msg_silent != 0 {
        return;
    }

    // Don't want a hit-enter prompt here.
    no_wait_return += 1;

    nvim_set_vim_var_warningmsg(message);
    nvim_set_keep_msg_raw(std::ptr::null());
    let hl_id = if hl != 0 { HLF_W } else { 0 };
    keep_msg_hl_id = hl_id;

    if nvim_msg_ext_kind_is_null() != 0 {
        msg_ext_ui_flush();
        nvim_set_msg_ext_kind(WMSG_KIND.as_ptr());
    }

    if msg(message, hl_id) && msg_scrolled == 0 {
        set_keep_msg(message, hl_id);
    }
    msg_didout = false; // Overwrite this message.
    msg_nowait = true; // Don't wait for this message.
    msg_col = 0;

    if no_wait_return > 0 {
        no_wait_return -= 1;
    }
}

/// Display a warning message with highlight.
///
/// Convenience wrapper that always uses warning highlight.
///
/// # Arguments
/// * `message` - The warning message string (NUL-terminated)
///
/// # Safety
/// - `message` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_give_warning_hl(message: *const c_char) {
    rs_give_warning(message, 1);
}

/// Display a warning message without highlight.
///
/// Convenience wrapper that never uses warning highlight.
///
/// # Arguments
/// * `message` - The warning message string (NUL-terminated)
///
/// # Safety
/// - `message` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_give_warning_plain(message: *const c_char) {
    rs_give_warning(message, 0);
}

// ============================================================================
// Message Position Functions
// ============================================================================

/// Calculate spaces needed to advance to a column.
///
/// Returns the number of spaces that would need to be output
/// to reach the target column from the current position.
///
/// # Arguments
/// * `col` - Target column
///
/// # Returns
/// Number of spaces needed (may be 0 if already at or past target)
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_advance_count(col: c_int) -> c_int {
    let columns = Columns;

    // Clamp target to valid range
    let target = if col >= columns { columns - 1 } else { col };

    if msg_col >= target {
        0
    } else {
        target - msg_col
    }
}

/// Check if advancing to column is needed.
///
/// Returns true if current column is less than target.
///
/// # Arguments
/// * `col` - Target column
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_need_advance(col: c_int) -> c_int {
    c_int::from(msg_col < col)
}

/// Set message column directly (for silent mode).
///
/// # Arguments
/// * `col` - Column to set
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_set_col_silent(col: c_int) {
    if msg_silent != 0 {
        msg_col = col;
    }
}

// ============================================================================
// Warning/Info Message Helpers
// ============================================================================

// Note: HLF_W is defined in error.rs to avoid duplication

/// Highlight face for info messages (HLF_MSG)
pub const HLF_MSG: c_int = 63;

/// Check if warning messages should be shown.
///
/// Returns false if msg_silent is set.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_warning_can_show() -> c_int {
    c_int::from(msg_silent == 0)
}

/// Get the warning highlight ID.
///
/// Returns HLF_W constant for use with message functions.
#[no_mangle]
pub const extern "C" fn rs_warning_hl_id() -> c_int {
    crate::error::HLF_W
}

/// Get the info/message highlight ID.
///
/// Returns HLF_MSG constant for use with message functions.
#[no_mangle]
pub const extern "C" fn rs_info_hl_id() -> c_int {
    HLF_MSG
}

// ============================================================================
// Column Calculation Functions
// ============================================================================

/// Calculate available space on current message line.
///
/// Returns the number of columns remaining from current position
/// to the end of the line.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_line_space() -> c_int {
    let columns = Columns;
    columns - msg_col
}

/// Check if there's enough space for a message of given width.
///
/// # Arguments
/// * `width` - Required width in columns
///
/// # Returns
/// 1 if enough space, 0 otherwise
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_has_space(width: c_int) -> c_int {
    c_int::from(rs_msg_line_space() >= width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hl_constants() {
        // Verify highlight ID getter functions work correctly
        let warning_id = rs_warning_hl_id();
        let info_id = rs_info_hl_id();

        // These should be positive values
        assert_ne!(warning_id, 0);
        assert_ne!(info_id, 0);

        // Info and warning should have different IDs
        assert_ne!(warning_id, info_id);
    }
}
