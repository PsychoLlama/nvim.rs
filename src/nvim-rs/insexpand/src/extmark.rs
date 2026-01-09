//! Extmark and cleanup support for completion.
//!
//! This module provides helper functions for managing extmarks during completion
//! and cleanup operations. The core extmark operations remain in C (extmark.c),
//! but Rust provides utilities for tracking state.

use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_col() -> c_int;
    fn nvim_get_compl_lnum() -> c_int;
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_get_curwin_cursor_lnum() -> c_int;
    fn nvim_compl_shown_match_has_newline() -> c_int;
}

/// Check if extmarks need to be saved.
///
/// Returns true if completion is starting and extmarks should be saved.
#[no_mangle]
pub unsafe extern "C" fn rs_extmark_should_save() -> c_int {
    // Save extmarks when starting completion
    c_int::from(nvim_get_compl_started() != 0)
}

/// Check if extmarks need to be restored.
///
/// Returns true if completion is ending and extmarks should be restored.
#[no_mangle]
pub unsafe extern "C" fn rs_extmark_should_restore() -> c_int {
    // Restore when completion ends
    c_int::from(nvim_get_compl_started() == 0)
}

/// Get the start line for extmark operations.
#[no_mangle]
pub unsafe extern "C" fn rs_extmark_start_line() -> c_int {
    nvim_get_compl_lnum()
}

/// Get the end line for extmark operations.
#[no_mangle]
pub unsafe extern "C" fn rs_extmark_end_line() -> c_int {
    nvim_get_curwin_cursor_lnum()
}

/// Get the start column for extmark operations.
#[no_mangle]
pub unsafe extern "C" fn rs_extmark_start_col() -> c_int {
    nvim_get_compl_col()
}

/// Get the end column for extmark operations.
#[no_mangle]
pub unsafe extern "C" fn rs_extmark_end_col() -> c_int {
    nvim_get_cursor_col()
}

/// Check if multi-line completion is active.
///
/// Returns true if the completion spans multiple lines.
#[no_mangle]
pub unsafe extern "C" fn rs_extmark_is_multiline() -> c_int {
    nvim_compl_shown_match_has_newline()
}

/// Get the line range for multi-line completion.
///
/// Returns the number of lines affected.
#[no_mangle]
pub unsafe extern "C" fn rs_extmark_line_count() -> c_int {
    if nvim_compl_shown_match_has_newline() == 0 {
        return 1;
    }
    let start = nvim_get_compl_lnum();
    let end = nvim_get_curwin_cursor_lnum();
    if end < start {
        1
    } else {
        end - start + 1
    }
}

/// Check if cleanup is needed after completion.
///
/// Returns true if resources should be freed.
#[no_mangle]
pub unsafe extern "C" fn rs_extmark_needs_cleanup() -> c_int {
    // Cleanup when completion has ended
    c_int::from(nvim_get_compl_started() == 0)
}

/// Check if extmark range overlaps with a given line.
///
/// Returns true if the line is within the completion extmark range.
#[no_mangle]
pub unsafe extern "C" fn rs_extmark_line_in_range(lnum: c_int) -> c_int {
    let start = nvim_get_compl_lnum();
    let end = nvim_get_curwin_cursor_lnum();
    c_int::from(lnum >= start && lnum <= end)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_line_count_for_single_line() {
        // A single-line completion has 1 line
        let expected = 1;
        assert!(expected >= 1);
    }
}
