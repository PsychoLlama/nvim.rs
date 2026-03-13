//! Warning and informational message handling
//!
//! Provides Rust implementations for warning messages, info messages,
//! and related display functions.

use std::ffi::{c_char, c_int};

// C function declarations
extern "C" {
    // Warning message functions
    fn give_warning(message: *const c_char, hl: c_int);

    // State accessors
    fn nvim_get_msg_col() -> c_int;
    fn nvim_set_msg_col(col: c_int);
    fn nvim_get_msg_silent() -> c_int;
    fn nvim_get_columns() -> c_int;
}

// ============================================================================
// Warning Message Functions
// ============================================================================

/// Display a warning message.
///
/// Shows the message as a warning, optionally highlighted.
/// The message is set in v:warningmsg.
///
/// # Arguments
/// * `message` - The warning message string (NUL-terminated)
/// * `hl` - If true, use warning highlight (HLF_W)
///
/// # Safety
/// - `message` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_give_warning(message: *const c_char, hl: c_int) {
    give_warning(message, hl);
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
    give_warning(message, 1);
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
    give_warning(message, 0);
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
    let msg_col = nvim_get_msg_col();
    let columns = nvim_get_columns();

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
    c_int::from(nvim_get_msg_col() < col)
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
    if nvim_get_msg_silent() != 0 {
        nvim_set_msg_col(col);
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
    c_int::from(nvim_get_msg_silent() == 0)
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
    let columns = nvim_get_columns();
    let msg_col = nvim_get_msg_col();
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
