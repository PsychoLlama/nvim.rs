//! Quickfix command helpers.
//!
//! This module provides helper functions for parsing and validating
//! quickfix command arguments and implementing command logic.

use std::ffi::c_int;

use crate::{
    nvim_qf_get_count, nvim_qf_get_curlist_idx, nvim_qf_get_index, nvim_qf_get_listcount,
    nvim_qf_get_title, QfInfoHandle, QfListHandle,
};

// =============================================================================
// Command Direction
// =============================================================================

/// Direction for quickfix navigation commands.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QfDirection {
    /// Move forward (next, newer).
    #[default]
    Forward = 1,
    /// Move backward (previous, older).
    Backward = -1,
}

/// Check if a direction is forward.
#[no_mangle]
pub const extern "C" fn rs_qf_is_forward(dir: QfDirection) -> bool {
    matches!(dir, QfDirection::Forward)
}

/// Check if a direction is backward.
#[no_mangle]
pub const extern "C" fn rs_qf_is_backward(dir: QfDirection) -> bool {
    matches!(dir, QfDirection::Backward)
}

// =============================================================================
// Stack Navigation
// =============================================================================

/// Check if we can navigate to an older quickfix list.
///
/// # Safety
///
/// - `qi` may be null (returns false)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_can_go_older(qi: QfInfoHandle) -> bool {
    if qi.is_null() {
        return false;
    }

    nvim_qf_get_curlist_idx(qi) > 0
}

/// Check if we can navigate to a newer quickfix list.
///
/// # Safety
///
/// - `qi` may be null (returns false)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_can_go_newer(qi: QfInfoHandle) -> bool {
    if qi.is_null() {
        return false;
    }

    let curlist = nvim_qf_get_curlist_idx(qi);
    let listcount = nvim_qf_get_listcount(qi);

    curlist < listcount - 1
}

/// Calculate the target list index for age navigation.
///
/// Returns the target list index (0-based), or -1 if at boundary.
///
/// # Safety
///
/// - `qi` may be null (returns -1)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_calc_age_target(
    qi: QfInfoHandle,
    count: c_int,
    go_older: bool,
) -> c_int {
    if qi.is_null() || count <= 0 {
        return -1;
    }

    let curlist = nvim_qf_get_curlist_idx(qi);
    let listcount = nvim_qf_get_listcount(qi);

    if go_older {
        let target = curlist - count;
        if target < 0 {
            -1
        } else {
            target
        }
    } else {
        let target = curlist + count;
        if target >= listcount {
            -1
        } else {
            target
        }
    }
}

/// Get the number of steps possible in a direction.
///
/// Returns how many older/newer lists can be navigated to.
///
/// # Safety
///
/// - `qi` may be null (returns 0)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_available_age_steps(qi: QfInfoHandle, go_older: bool) -> c_int {
    if qi.is_null() {
        return 0;
    }

    let curlist = nvim_qf_get_curlist_idx(qi);
    let listcount = nvim_qf_get_listcount(qi);

    if go_older {
        curlist
    } else {
        listcount - curlist - 1
    }
}

// =============================================================================
// Entry Navigation
// =============================================================================

/// Check if we can navigate to a next entry.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_can_go_next(qfl: QfListHandle) -> bool {
    if qfl.is_null() {
        return false;
    }

    let idx = nvim_qf_get_index(qfl);
    let count = nvim_qf_get_count(qfl);

    idx < count
}

/// Check if we can navigate to a previous entry.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_can_go_prev(qfl: QfListHandle) -> bool {
    if qfl.is_null() {
        return false;
    }

    nvim_qf_get_index(qfl) > 1
}

/// Calculate target entry index for navigation.
///
/// Returns the target index (1-based), clamped to valid range.
/// Returns 0 if the list is empty.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_calc_nav_target(
    qfl: QfListHandle,
    count: c_int,
    forward: bool,
) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let current = nvim_qf_get_index(qfl);
    let total = nvim_qf_get_count(qfl);

    if total == 0 {
        return 0;
    }

    let target = if forward {
        current + count
    } else {
        current - count
    };

    // Clamp to valid range
    target.clamp(1, total)
}

/// Calculate steps available in a direction.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_available_nav_steps(qfl: QfListHandle, forward: bool) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let current = nvim_qf_get_index(qfl);
    let total = nvim_qf_get_count(qfl);

    if forward {
        total - current
    } else {
        current - 1
    }
}

// =============================================================================
// Command Result Information
// =============================================================================

/// Result of a quickfix command operation.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct QfCmdResult {
    /// Operation succeeded.
    pub success: bool,
    /// New current index (for entry navigation).
    pub new_idx: c_int,
    /// Number of entries affected.
    pub count: c_int,
    /// Whether to update the window.
    pub update_window: bool,
}

/// Calculate result for a :cc / :ll style jump command.
///
/// # Safety
///
/// - `qfl` may be null (returns failure result)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_cmd_cc_result(qfl: QfListHandle, target_idx: c_int) -> QfCmdResult {
    let mut result = QfCmdResult::default();

    if qfl.is_null() {
        return result;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 || target_idx < 1 || target_idx > count {
        return result;
    }

    result.success = true;
    result.new_idx = target_idx;
    result.count = 1;
    result.update_window = true;

    result
}

/// Calculate result for a :cnext / :cprev style navigation command.
///
/// # Safety
///
/// - `qfl` may be null (returns failure result)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_cmd_nav_result(
    qfl: QfListHandle,
    count: c_int,
    forward: bool,
) -> QfCmdResult {
    let mut result = QfCmdResult::default();

    if qfl.is_null() || count <= 0 {
        return result;
    }

    let current = nvim_qf_get_index(qfl);
    let total = nvim_qf_get_count(qfl);

    if total == 0 {
        return result;
    }

    let target = if forward {
        (current + count).min(total)
    } else {
        (current - count).max(1)
    };

    // Check if we actually moved
    if target == current {
        return result;
    }

    result.success = true;
    result.new_idx = target;
    result.count = (target - current).abs();
    result.update_window = true;

    result
}

// =============================================================================
// List Information for Commands
// =============================================================================

/// Summary info for a quickfix list (for :clist output).
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct QfListInfo {
    /// List index (0-based).
    pub list_idx: c_int,
    /// Number of entries.
    pub count: c_int,
    /// Current entry index (1-based).
    pub current_idx: c_int,
    /// Whether this is the current list.
    pub is_current: bool,
    /// Whether the list has a title.
    pub has_title: bool,
}

/// Get info about a quickfix list.
///
/// # Safety
///
/// - `qi` may be null (returns default info)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_list_info(qi: QfInfoHandle, list_idx: c_int) -> QfListInfo {
    let mut info = QfListInfo::default();

    if qi.is_null() {
        return info;
    }

    let listcount = nvim_qf_get_listcount(qi);
    if list_idx < 0 || list_idx >= listcount {
        return info;
    }

    info.list_idx = list_idx;
    info.is_current = list_idx == nvim_qf_get_curlist_idx(qi);

    // Would need to get the specific list to fill in count/current_idx/has_title
    // For now just set basic info

    info
}

/// Get info about the current quickfix list.
///
/// # Safety
///
/// - `qfl` may be null (returns default info)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_current_list_info(qfl: QfListHandle) -> QfListInfo {
    let mut info = QfListInfo::default();

    if qfl.is_null() {
        return info;
    }

    info.count = nvim_qf_get_count(qfl);
    info.current_idx = nvim_qf_get_index(qfl);
    info.is_current = true;

    let title = nvim_qf_get_title(qfl);
    info.has_title = !title.is_null();

    info
}

// =============================================================================
// Range Validation
// =============================================================================

/// Validate a range for :clist style commands.
///
/// Returns true if the range is valid.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_valid_list_range(
    qfl: QfListHandle,
    start: c_int,
    end: c_int,
) -> bool {
    if qfl.is_null() {
        return false;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        return false;
    }

    start >= 1 && end >= start && start <= count
}

/// Clamp a range to valid bounds.
///
/// # Safety
///
/// - `qfl` may be null (returns 0, 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `out_start` and `out_end` must be valid pointers
#[no_mangle]
pub unsafe extern "C" fn rs_qf_clamp_range(
    qfl: QfListHandle,
    start: c_int,
    end: c_int,
    out_start: *mut c_int,
    out_end: *mut c_int,
) {
    if out_start.is_null() || out_end.is_null() {
        return;
    }

    if qfl.is_null() {
        *out_start = 0;
        *out_end = 0;
        return;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        *out_start = 0;
        *out_end = 0;
        return;
    }

    *out_start = start.clamp(1, count);
    *out_end = end.clamp(*out_start, count);
}

// =============================================================================
// Window Height Calculation
// =============================================================================

/// Calculate the optimal height for the quickfix window.
///
/// Returns a height between `min_height` and `max_height`, based on entry count.
///
/// # Safety
///
/// - `qfl` may be null (returns `min_height`)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_calc_window_height(
    qfl: QfListHandle,
    min_height: c_int,
    max_height: c_int,
) -> c_int {
    if qfl.is_null() {
        return min_height.max(1);
    }

    let count = nvim_qf_get_count(qfl);
    count.clamp(min_height.max(1), max_height)
}

// =============================================================================
// Phase 6: Command Type Classification
// =============================================================================

/// Command types for quickfix operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QfCmdType {
    /// Create new list (e.g., :make, :grep, :vimgrep)
    Create = 0,
    /// Get/add to list (e.g., :cgetfile, :caddfile)
    Get = 1,
    /// Add to existing list (e.g., :grepadd, :vimgrepadd)
    Add = 2,
}

/// Check if command creates a new list
#[no_mangle]
pub const extern "C" fn rs_qf_cmd_creates_list(cmd_type: QfCmdType) -> bool {
    matches!(cmd_type, QfCmdType::Create)
}

/// Check if command adds to existing list
#[no_mangle]
pub const extern "C" fn rs_qf_cmd_adds_to_list(cmd_type: QfCmdType) -> bool {
    matches!(cmd_type, QfCmdType::Add)
}

// =============================================================================
// Grep Pattern Parsing
// =============================================================================

/// Result of parsing a grep/vimgrep pattern
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GrepPatternResult {
    /// Offset where pattern starts in input
    pub pattern_start: c_int,
    /// Length of the pattern
    pub pattern_len: c_int,
    /// Delimiter character used (0 if none)
    pub delimiter: u8,
    /// Whether pattern is valid
    pub valid: bool,
    /// Whether the pattern uses a delimiter
    pub has_delimiter: bool,
}

/// Parse a vimgrep-style pattern (e.g., /pattern/ or word)
///
/// # Safety
///
/// - `input` may be null (returns invalid result)
/// - If non-null, `input` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parse_grep_pattern(input: *const std::ffi::c_char) -> GrepPatternResult {
    use std::ffi::CStr;

    let mut result = GrepPatternResult::default();

    if input.is_null() {
        return result;
    }

    let Ok(input_str) = CStr::from_ptr(input).to_str() else {
        return result;
    };

    let trimmed = input_str.trim_start();
    let Some(first_char) = trimmed.chars().next() else {
        return result;
    };

    // Check for delimiter-based pattern
    if matches!(first_char, '/' | '?' | '@' | '!' | '#' | '$' | '%' | '^' | '&') {
        result.delimiter = first_char as u8;
        result.has_delimiter = true;

        // Find closing delimiter
        let rest = &trimmed[1..];
        let mut pattern_end = None;
        let mut prev_was_escape = false;

        for (i, c) in rest.chars().enumerate() {
            if c == first_char && !prev_was_escape {
                pattern_end = Some(i);
                break;
            }
            prev_was_escape = c == '\\' && !prev_was_escape;
        }

        if let Some(end) = pattern_end {
            #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
            {
                result.pattern_start = (input_str.len() - trimmed.len() + 1) as c_int;
                result.pattern_len = end as c_int;
            }
            result.valid = end > 0;
        }
    } else {
        // Word-based pattern (until whitespace)
        result.has_delimiter = false;
        let pattern_len = trimmed.chars().take_while(|c| !c.is_whitespace()).count();
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        {
            result.pattern_start = (input_str.len() - trimmed.len()) as c_int;
            result.pattern_len = pattern_len as c_int;
        }
        result.valid = pattern_len > 0;
    }

    result
}

// =============================================================================
// Make/Grep Command Helpers
// =============================================================================

/// Check if a character is valid in a shell filename
#[no_mangle]
pub const extern "C" fn rs_qf_is_shell_filename_char(c: u8) -> bool {
    // Most printable ASCII characters are valid, except shell special chars
    matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' |
             b'_' | b'-' | b'.' | b'/' | b'~' | b'@' | b'+' | b'=' | b',' | b':')
}

/// Calculate the number of files from a file list pattern
///
/// Returns the estimated number of files based on the argument count.
#[no_mangle]
pub const extern "C" fn rs_qf_estimate_file_count(has_pattern: bool, arg_count: c_int) -> c_int {
    if has_pattern {
        // Pattern might match many files, estimate conservatively
        arg_count.saturating_mul(10)
    } else {
        // Direct file list
        arg_count
    }
}

/// Information about a make/grep command
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MakeGrepInfo {
    /// Whether this is a grep command (vs make)
    pub is_grep: bool,
    /// Whether this adds to existing list
    pub is_add: bool,
    /// Whether this is for location list
    pub is_loclist: bool,
    /// Whether to jump to first error
    pub jump_first: bool,
}

/// Get action character for a make/grep command
#[no_mangle]
pub const extern "C" fn rs_qf_make_grep_action(info: MakeGrepInfo) -> u8 {
    if info.is_add {
        b'a'
    } else {
        b' '
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction() {
        assert!(rs_qf_is_forward(QfDirection::Forward));
        assert!(!rs_qf_is_forward(QfDirection::Backward));
        assert!(rs_qf_is_backward(QfDirection::Backward));
        assert!(!rs_qf_is_backward(QfDirection::Forward));
    }

    #[test]
    fn test_null_safety_stack() {
        unsafe {
            assert!(!rs_qf_can_go_older(std::ptr::null()));
            assert!(!rs_qf_can_go_newer(std::ptr::null()));
            assert_eq!(rs_qf_calc_age_target(std::ptr::null(), 1, true), -1);
            assert_eq!(rs_qf_available_age_steps(std::ptr::null(), true), 0);
        }
    }

    #[test]
    fn test_null_safety_nav() {
        unsafe {
            assert!(!rs_qf_can_go_next(std::ptr::null()));
            assert!(!rs_qf_can_go_prev(std::ptr::null()));
            assert_eq!(rs_qf_calc_nav_target(std::ptr::null(), 1, true), 0);
            assert_eq!(rs_qf_available_nav_steps(std::ptr::null(), true), 0);
        }
    }

    #[test]
    fn test_null_safety_results() {
        unsafe {
            let result = rs_qf_cmd_cc_result(std::ptr::null(), 1);
            assert!(!result.success);

            let result = rs_qf_cmd_nav_result(std::ptr::null(), 1, true);
            assert!(!result.success);
        }
    }

    #[test]
    fn test_null_safety_info() {
        unsafe {
            let info = rs_qf_get_list_info(std::ptr::null(), 0);
            assert!(!info.is_current);

            let info = rs_qf_current_list_info(std::ptr::null());
            assert_eq!(info.count, 0);
        }
    }

    #[test]
    fn test_null_safety_range() {
        unsafe {
            assert!(!rs_qf_valid_list_range(std::ptr::null(), 1, 10));

            let mut start = 0;
            let mut end = 0;
            rs_qf_clamp_range(std::ptr::null(), 1, 10, &raw mut start, &raw mut end);
            assert_eq!(start, 0);
            assert_eq!(end, 0);
        }
    }

    #[test]
    fn test_null_safety_height() {
        unsafe {
            assert_eq!(rs_qf_calc_window_height(std::ptr::null(), 3, 10), 3);
            assert_eq!(rs_qf_calc_window_height(std::ptr::null(), 0, 10), 1);
        }
    }

    #[test]
    fn test_cmd_result_default() {
        let result = QfCmdResult::default();
        assert!(!result.success);
        assert_eq!(result.new_idx, 0);
        assert_eq!(result.count, 0);
        assert!(!result.update_window);
    }

    #[test]
    fn test_list_info_default() {
        let info = QfListInfo::default();
        assert_eq!(info.list_idx, 0);
        assert_eq!(info.count, 0);
        assert!(!info.is_current);
        assert!(!info.has_title);
    }

    #[test]
    fn test_cmd_type() {
        assert!(rs_qf_cmd_creates_list(QfCmdType::Create));
        assert!(!rs_qf_cmd_creates_list(QfCmdType::Get));
        assert!(!rs_qf_cmd_creates_list(QfCmdType::Add));

        assert!(rs_qf_cmd_adds_to_list(QfCmdType::Add));
        assert!(!rs_qf_cmd_adds_to_list(QfCmdType::Create));
        assert!(!rs_qf_cmd_adds_to_list(QfCmdType::Get));
    }

    #[test]
    fn test_parse_grep_pattern_delimited() {
        unsafe {
            let input = c"/pattern/";
            let result = rs_qf_parse_grep_pattern(input.as_ptr());
            assert!(result.valid);
            assert!(result.has_delimiter);
            assert_eq!(result.delimiter, b'/');
            assert_eq!(result.pattern_len, 7);
        }
    }

    #[test]
    fn test_parse_grep_pattern_word() {
        unsafe {
            let input = c"pattern file.txt";
            let result = rs_qf_parse_grep_pattern(input.as_ptr());
            assert!(result.valid);
            assert!(!result.has_delimiter);
            assert_eq!(result.pattern_len, 7);
        }
    }

    #[test]
    fn test_parse_grep_pattern_null() {
        unsafe {
            let result = rs_qf_parse_grep_pattern(std::ptr::null());
            assert!(!result.valid);
        }
    }

    #[test]
    fn test_shell_filename_char() {
        assert!(rs_qf_is_shell_filename_char(b'a'));
        assert!(rs_qf_is_shell_filename_char(b'Z'));
        assert!(rs_qf_is_shell_filename_char(b'0'));
        assert!(rs_qf_is_shell_filename_char(b'_'));
        assert!(rs_qf_is_shell_filename_char(b'/'));
        assert!(!rs_qf_is_shell_filename_char(b' '));
        assert!(!rs_qf_is_shell_filename_char(b'*'));
        assert!(!rs_qf_is_shell_filename_char(b'?'));
    }

    #[test]
    fn test_estimate_file_count() {
        assert_eq!(rs_qf_estimate_file_count(false, 5), 5);
        assert_eq!(rs_qf_estimate_file_count(true, 5), 50);
    }

    #[test]
    fn test_make_grep_action() {
        let add_info = MakeGrepInfo {
            is_add: true,
            ..Default::default()
        };
        assert_eq!(rs_qf_make_grep_action(add_info), b'a');

        let create_info = MakeGrepInfo {
            is_add: false,
            ..Default::default()
        };
        assert_eq!(rs_qf_make_grep_action(create_info), b' ');
    }
}
