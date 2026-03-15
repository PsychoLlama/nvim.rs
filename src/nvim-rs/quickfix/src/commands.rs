//! Quickfix command helpers.
//!
//! This module provides helper functions for parsing and validating
//! quickfix command arguments and implementing command logic.

use std::ffi::{c_char, c_int, c_void};

use crate::{
    nvim_qf_get_count, nvim_qf_get_curlist_idx, nvim_qf_get_index, nvim_qf_get_listcount,
    nvim_qf_get_title, QfInfoHandle, QfListHandle,
};

// =============================================================================
// ex_helpgrep types and FFI declarations
// =============================================================================

/// Opaque handle to `exarg_T`
type EapHandle = *mut c_void;

/// Opaque handle to `qf_info_T` (mutable)
type QfInfoHandleMut = *mut c_void;

extern "C" {
    fn nvim_get_ql_info() -> QfInfoHandleMut;
    // nvim_hgr_pre_check deleted (Phase 4): inlined via nvim_qf_apply_autocmd_pre
    fn nvim_qf_apply_autocmd_pre(au_name: *const c_char) -> bool;
    // nvim_hgr_save_cpo renamed to nvim_save_cpo_set_empty (Phase 4)
    fn nvim_save_cpo_set_empty() -> *mut c_void;
    // nvim_hgr_is_loclist_cmd deleted (Phase 4): use nvim_is_loclist_cmd
    fn nvim_is_loclist_cmd(cmdidx: c_int) -> bool;
    // nvim_hgr_get_ll deleted (Phase 4): inlined in Rust
    fn nvim_qf_curwin_buf_is_help() -> bool;
    fn nvim_qf_find_help_win() -> *mut c_void;
    fn nvim_qf_win_get_llist(win: *const c_void) -> *mut c_void;
    #[link_name = "rs_incr_quickfix_busy"]
    fn nvim_incr_quickfix_busy();
    #[link_name = "rs_decr_quickfix_busy"]
    fn nvim_decr_quickfix_busy();
    // nvim_hgr_compile_and_search deleted (Phase 3): inlined into Rust below.
    // nvim_hgr_regex_search deleted (Phase 16): inlined into rs_ex_helpgrep below.
    fn nvim_eap_get_cmdlinep_deref_make(eap: EapHandle) -> *mut c_char;
    // Phase 16: helpgrep regex inlining
    fn check_help_lang(arg: *mut c_char) -> *mut c_char;
    fn vim_regcomp(pat: *const c_char, flags: c_int) -> *mut c_void;
    fn nvim_qf_regmatch_create(prog: *mut c_void, ic: bool) -> *mut c_void;
    fn nvim_qf_regmatch_extract_prog(rm: *mut c_void) -> *mut c_void;
    fn vim_regfree(prog: *mut c_void);
    fn rs_hgr_search_in_rtp(qfl: *mut c_void, regmatch: *mut c_void, lang: *const c_char);
    // nvim_hgr_restore_cpo renamed to nvim_restore_cpo (Phase 4)
    fn nvim_restore_cpo(saved_cpo: *mut c_void);
    #[link_name = "rs_qf_update_buffer"]
    fn nvim_qf_update_buffer(qi: QfInfoHandleMut, old_last: *const c_void);
    // nvim_hgr_post_autocmd deleted (Phase 4): inlined via autocmd_post + is_ll_stack + find_win
    fn nvim_qf_apply_autocmd_post(au_name: *const c_char);
    fn nvim_qf_is_ll_stack_qi(qi: *const c_void) -> bool;
    fn nvim_qf_find_win_with_loclist(ll: *const c_void) -> *mut c_void;
    // nvim_hgr_jump_or_nomatch deleted (Phase 4): inlined via rs_qf_list_empty + rs_qf_jump_newwin
    fn semsg(fmt: *const std::ffi::c_char, ...) -> bool;
    // (nvim_semsg_nomatch2 deleted: use semsg directly)
    // nvim_hgr_is_lhelpgrep deleted (Phase 4): use nvim_eap_get_cmdidx comparison
    // nvim_hgr_cleanup deleted (Phase 4): inlined via curwin accessors + rs_ll_free_all
    fn nvim_qf_get_curwin() -> *mut c_void;
    fn nvim_win_set_llist(win: *mut c_void, qi: *mut c_void);
    // Used for finalization after compile+search (Phase 3)
    fn nvim_qf_set_nonevalid(qfl: *mut c_void, nonevalid: bool);
    fn nvim_qf_set_ptr(qfl: *mut c_void, ptr: *const c_void);
    fn nvim_qf_set_index(qfl: *mut c_void, idx: c_int);
    fn nvim_qf_get_start(qfl: *const c_void) -> *const c_void;
}

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
pub unsafe extern "C" fn rs_qf_parse_grep_pattern(
    input: *const std::ffi::c_char,
) -> GrepPatternResult {
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
    if matches!(
        first_char,
        '/' | '?' | '@' | '!' | '#' | '$' | '%' | '^' | '&'
    ) {
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

    // ==========================================================================
    // Phase Q4: Ex Command Helper Tests
    // ==========================================================================

    #[test]
    fn test_cc_default_errornr() {
        // Test without address
        assert_eq!(rs_qf_cc_default_errornr(CcCmdType::Cc), 0);
        assert_eq!(rs_qf_cc_default_errornr(CcCmdType::Ll), 0);
        assert_eq!(rs_qf_cc_default_errornr(CcCmdType::CRewind), 1);
        assert_eq!(rs_qf_cc_default_errornr(CcCmdType::LRewind), 1);
        assert_eq!(rs_qf_cc_default_errornr(CcCmdType::CFirst), 1);
        assert_eq!(rs_qf_cc_default_errornr(CcCmdType::LFirst), 1);
        assert_eq!(rs_qf_cc_default_errornr(CcCmdType::CLast), 32767);
        assert_eq!(rs_qf_cc_default_errornr(CcCmdType::LLast), 32767);
    }

    #[test]
    fn test_cnext_direction() {
        assert_eq!(
            rs_qf_cnext_direction(CnextCmdType::CNext),
            QfDirection::Forward
        );
        assert_eq!(
            rs_qf_cnext_direction(CnextCmdType::LNext),
            QfDirection::Forward
        );
        assert_eq!(
            rs_qf_cnext_direction(CnextCmdType::CPrevious),
            QfDirection::Backward
        );
        assert_eq!(
            rs_qf_cnext_direction(CnextCmdType::CPrev),
            QfDirection::Backward
        );
    }
}

// =============================================================================
// Phase Q4: :cc Command Helpers
// =============================================================================

/// Command type for :cc style commands
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CcCmdType {
    /// :cc - jump to error N
    #[default]
    Cc,
    /// :ll - location list version
    Ll,
    /// :crewind - jump to first error
    CRewind,
    /// :lrewind
    LRewind,
    /// :cfirst - jump to first error
    CFirst,
    /// :lfirst
    LFirst,
    /// :clast - jump to last error
    CLast,
    /// :llast
    LLast,
}

/// Get the default error number for :cc style commands (when no address given).
///
/// - :cc/:ll -> 0 (current error)
/// - :crewind/:cfirst/:lrewind/:lfirst -> 1 (first error)
/// - :clast/:llast -> 32767 (last error, clamped)
#[no_mangle]
pub const extern "C" fn rs_qf_cc_default_errornr(cmd_type: CcCmdType) -> c_int {
    match cmd_type {
        CcCmdType::Cc | CcCmdType::Ll => 0,
        CcCmdType::CRewind | CcCmdType::LRewind | CcCmdType::CFirst | CcCmdType::LFirst => 1,
        CcCmdType::CLast | CcCmdType::LLast => 32767,
    }
}

/// Check if a :cc command type is a location list command.
#[no_mangle]
pub const extern "C" fn rs_qf_cc_is_loclist(cmd_type: CcCmdType) -> bool {
    matches!(
        cmd_type,
        CcCmdType::Ll | CcCmdType::LRewind | CcCmdType::LFirst | CcCmdType::LLast
    )
}

// =============================================================================
// Phase Q4: :cnext Command Helpers
// =============================================================================

/// Command type for :cnext style commands
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CnextCmdType {
    /// :cnext - next error
    #[default]
    CNext,
    /// :lnext
    LNext,
    /// :cprevious - previous error
    CPrevious,
    /// :cprev - same as cprevious
    CPrev,
    /// :lprevious
    LPrevious,
    /// :lprev
    LPrev,
    /// :cNext - previous error (capital N)
    CNextBig,
    /// :lNext
    LNextBig,
    /// :cnfile - first error in next file
    CNFile,
    /// :lnfile
    LNFile,
    /// :cpfile - last error in previous file
    CPFile,
    /// :lpfile
    LPFile,
    /// :cNfile - first error in previous file
    CNFileBig,
    /// :lNfile
    LNFileBig,
}

/// Get the navigation direction for :cnext style commands.
#[no_mangle]
pub const extern "C" fn rs_qf_cnext_direction(cmd_type: CnextCmdType) -> QfDirection {
    match cmd_type {
        CnextCmdType::CNext | CnextCmdType::LNext | CnextCmdType::CNFile | CnextCmdType::LNFile => {
            QfDirection::Forward
        }
        CnextCmdType::CPrevious
        | CnextCmdType::CPrev
        | CnextCmdType::LPrevious
        | CnextCmdType::LPrev
        | CnextCmdType::CNextBig
        | CnextCmdType::LNextBig
        | CnextCmdType::CPFile
        | CnextCmdType::LPFile
        | CnextCmdType::CNFileBig
        | CnextCmdType::LNFileBig => QfDirection::Backward,
    }
}

/// Check if a :cnext command type is a location list command.
#[no_mangle]
pub const extern "C" fn rs_qf_cnext_is_loclist(cmd_type: CnextCmdType) -> bool {
    matches!(
        cmd_type,
        CnextCmdType::LNext
            | CnextCmdType::LPrevious
            | CnextCmdType::LPrev
            | CnextCmdType::LNextBig
            | CnextCmdType::LNFile
            | CnextCmdType::LPFile
            | CnextCmdType::LNFileBig
    )
}

/// Check if a :cnext command type navigates by file (nfile/pfile).
#[no_mangle]
pub const extern "C" fn rs_qf_cnext_is_file_nav(cmd_type: CnextCmdType) -> bool {
    matches!(
        cmd_type,
        CnextCmdType::CNFile
            | CnextCmdType::LNFile
            | CnextCmdType::CPFile
            | CnextCmdType::LPFile
            | CnextCmdType::CNFileBig
            | CnextCmdType::LNFileBig
    )
}

// =============================================================================
// ex_helpgrep — :helpgrep/:lhelpgrep command
// =============================================================================

// CMD_helpgrep and CMD_lhelpgrep enum values (from ex_cmds_enum.generated.h)
const CMD_HELPGREP: c_int = 178;
const CMD_LHELPGREP: c_int = 241;
// Location list type constant (matches quickfix.h QFLT_LOCATION)
const QFLT_LOCATION: c_int = 1;

extern "C" {
    fn rs_qf_alloc_stack(qfl_type: c_int, maxcount: c_int) -> *mut c_void;
    fn rs_qf_list_empty(qfl: *const c_void) -> bool;
    fn rs_qf_jump_newwin(qi: *mut c_void, dir: c_int, errornr: c_int, forceit: c_int, newwin: bool);
    fn rs_ll_free_all(pqi: *mut *mut c_void);
}

/// Compute the helpgrep autocmd name from the command index.
/// Returns `Some("helpgrep")` or `Some("lhelpgrep")`, or `None` for other commands.
const fn hgr_au_name(cmdidx: c_int) -> Option<&'static std::ffi::CStr> {
    match cmdidx {
        x if x == CMD_HELPGREP => Some(c"helpgrep"),
        x if x == CMD_LHELPGREP => Some(c"lhelpgrep"),
        _ => None,
    }
}

/// Rust implementation of `:helpgrep` / `:lhelpgrep`.
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_helpgrep(eap: EapHandle) {
    let cmdidx = nvim_eap_get_cmdidx(eap);

    // 1. Pre-check: fire EVENT_QUICKFIXCMDPRE, check aborting()
    // Inlined from nvim_hgr_pre_check.
    let au_name_cstr = hgr_au_name(cmdidx);
    let au_name_ptr = au_name_cstr.map_or(std::ptr::null(), std::ffi::CStr::as_ptr);
    if !nvim_qf_apply_autocmd_pre(au_name_ptr) {
        return;
    }

    // 2. Save cpoptions and set to empty
    // Inlined from nvim_hgr_save_cpo (renamed nvim_save_cpo_set_empty).
    let saved_cpo = nvim_save_cpo_set_empty();

    // 3. Setup qi — use global ql_info or allocate location list
    // Inlined from nvim_hgr_is_loclist_cmd + nvim_hgr_get_ll.
    let mut qi = nvim_get_ql_info();
    let mut new_qi = false;
    if nvim_is_loclist_cmd(cmdidx) {
        // Inline hgr_get_ll: find help window, use its llist or allocate new.
        let wp = if nvim_qf_curwin_buf_is_help() {
            // curwin is a help window — use it directly
            nvim_qf_get_curwin()
        } else {
            nvim_qf_find_help_win()
        };
        let wp_llist = if wp.is_null() {
            std::ptr::null_mut()
        } else {
            nvim_qf_win_get_llist(wp.cast_const())
        };
        if wp_llist.is_null() {
            // Allocate a new location list for help text matches.
            qi = rs_qf_alloc_stack(QFLT_LOCATION, 1);
            new_qi = true;
        } else {
            qi = wp_llist;
        }
    }

    nvim_incr_quickfix_busy();

    // 4a. Build command title and create new quickfix list
    let eap_arg = nvim_eap_get_arg(eap);
    let cmdline = nvim_eap_get_cmdlinep_deref_make(eap);
    let mut title_buf = [0i8; 1024];
    rs_qf_cmdtitle(cmdline, title_buf.as_mut_ptr(), title_buf.len());
    crate::rs_qf_new_list(qi, title_buf.as_ptr());

    // 4b. Compile regex, search help files, free regex.
    // Inlined from nvim_hgr_regex_search (Phase 16).
    // RE_MAGIC=256, RE_STRING=4
    let lang = check_help_lang(eap_arg);
    let prog = vim_regcomp(eap_arg.cast_const(), 256 + 4);
    let updated = if prog.is_null() {
        false
    } else {
        let regmatch = nvim_qf_regmatch_create(prog, false);
        let qfl = nvim_qf_get_curlist_mut(qi);
        rs_hgr_search_in_rtp(qfl, regmatch, lang.cast_const());
        let prog_out = nvim_qf_regmatch_extract_prog(regmatch);
        vim_regfree(prog_out);
        true
    };

    // 4c. Finalize the list (set ptr/index/nonevalid/changedtick)
    if updated {
        let qfl = nvim_qf_get_curlist_mut(qi);
        nvim_qf_set_nonevalid(qfl, false);
        nvim_qf_set_ptr(qfl, nvim_qf_get_start(qfl.cast_const()));
        nvim_qf_set_index(qfl, 1);
        crate::rs_qf_incr_changedtick(qfl.cast());
    }

    // 5. Restore cpoptions (handles plugin interference)
    // Inlined from nvim_hgr_restore_cpo (renamed nvim_restore_cpo).
    nvim_restore_cpo(saved_cpo);

    // 6. Update quickfix buffer if list was populated
    if updated {
        nvim_qf_update_buffer(qi, std::ptr::null());
    }

    // 7. Post autocmd — may invalidate location list
    // Inlined from nvim_hgr_post_autocmd.
    nvim_qf_apply_autocmd_post(au_name_ptr);
    if !new_qi
        && nvim_qf_is_ll_stack_qi(qi.cast_const())
        && nvim_qf_find_win_with_loclist(qi.cast_const()).is_null()
    {
        nvim_decr_quickfix_busy();
        return;
    }

    // 8. Jump to first match or show "no match" error
    // Inlined from nvim_hgr_jump_or_nomatch.
    let qfl = nvim_qf_get_curlist_mut(qi);
    if rs_qf_list_empty(qfl.cast_const()) {
        semsg(c"E480: No match: %s".as_ptr(), eap_arg.cast_const());
    } else {
        rs_qf_jump_newwin(qi, 0, 0, 0, false);
    }

    nvim_decr_quickfix_busy();

    // 9. Cleanup: free location list if :lhelpgrep and not needed
    // Inlined from nvim_hgr_is_lhelpgrep + nvim_hgr_cleanup.
    if cmdidx == CMD_LHELPGREP {
        // If the help window is not opened or if it already points to the
        // correct location list, then free the new location list.
        let curwin = nvim_qf_get_curwin();
        let curwin_llist = nvim_qf_win_get_llist(curwin.cast_const());
        if !nvim_qf_curwin_buf_is_help() || curwin_llist == qi {
            if new_qi {
                rs_ll_free_all(&raw mut qi);
            }
        } else if curwin_llist.is_null() && new_qi {
            // current window didn't have a location list — associate now.
            nvim_win_set_llist(curwin, qi);
        }
    }
}

// =============================================================================
// Phase 2: Ex command implementations
// =============================================================================

// Direction constants (matching vim_defs.h)
const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;
const FORWARD_FILE: c_int = 3;
const BACKWARD_FILE: c_int = -3;

// CMD_* enum values (from ex_cmds_enum.generated.h)
const CMD_CC: c_int = 62;
const CMD_LL: c_int = 246;
const CMD_CREWIND: c_int = 107;
const CMD_LREWIND: c_int = 264;
const CMD_CFIRST: c_int = 70;
const CMD_LFIRST: c_int = 238;
const CMD_CDO: c_int = 65;
const CMD_LDO: c_int = 231;
const CMD_CFDO: c_int = 69;
const CMD_LFDO: c_int = 237;
const CMD_CPREVIOUS: c_int = 104;
const CMD_LPREVIOUS: c_int = 262;
const CMD_CNFILE: c_int = 89;
const CMD_LNFILE: c_int = 255;
const CMD_CPFILE: c_int = 105;
const CMD_LPFILE: c_int = 263;
const CMD_CNFILE_BIG: c_int = 48; // CMD_cNfile
const CMD_LNFILE_BIG: c_int = 215; // CMD_lNfile
const CMD_CNEXT_BIG: c_int = 47; // CMD_cNext
const CMD_LNEXT_BIG: c_int = 214; // CMD_lNext
const CMD_COLDER: c_int = 94;
const CMD_LOLDER: c_int = 260;
const CMD_CABOVE: c_int = 51;
const CMD_CBELOW: c_int = 60;
const CMD_LABOVE: c_int = 217;
const CMD_LBELOW: c_int = 226;
const CMD_CAFTER: c_int = 55;
const CMD_CBEFORE: c_int = 59;
const CMD_LBEFORE: c_int = 225;

// BUF_HAS_* flags
const BUF_HAS_QF_ENTRY: c_int = 1;
const BUF_HAS_LL_ENTRY: c_int = 2;

type LinenrT = i64;

extern "C" {
    #[link_name = "rs_qf_cmd_get_stack"]
    fn nvim_qf_cmd_get_stack(eap: EapHandle, print_emsg: bool) -> QfInfoHandleMut;
    fn nvim_eap_get_cmdidx(eap: EapHandle) -> c_int;
    fn nvim_eap_get_addr_count(eap: EapHandle) -> c_int;
    fn nvim_eap_get_line1(eap: EapHandle) -> LinenrT;
    fn nvim_eap_get_line2(eap: EapHandle) -> LinenrT;
    fn nvim_eap_get_forceit(eap: EapHandle) -> bool;
    #[link_name = "rs_qf_msg"]
    fn nvim_qf_msg(qi: QfInfoHandleMut, which: c_int, lead: *const u8);
    fn emsg(msg: *const std::ffi::c_char) -> bool;
    fn msg(s: *const std::ffi::c_char, hl_id: c_int) -> bool;
    // (nvim_emsg_loclist, nvim_emsg_no_errors, nvim_emsg_at_bottom, nvim_emsg_at_top,
    //  nvim_msg_no_entries, nvim_emsg_invrange deleted: use emsg/msg directly)
    fn nvim_qf_curwin_is_ll() -> bool;
    fn nvim_qf_curwin_get_loclist() -> QfInfoHandleMut;
    fn nvim_qf_get_cursor_lnum() -> LinenrT;
    fn nvim_do_cmdline_cmd(cmd: *const u8);

    // From crate (already declared in lib.rs)
    fn nvim_qf_get_curlist_mut(qi: QfInfoHandleMut) -> *mut c_void;
    fn nvim_qf_set_curlist_idx(qi: QfInfoHandleMut, idx: c_int);

    fn rs_qf_stack_empty(qi: *const c_void) -> bool;
    fn rs_qf_find_nth_adj_entry(
        qfl: *const c_void,
        bnr: c_int,
        pos: *const c_void,
        n: LinenrT,
        dir: c_int,
        linewise: bool,
    ) -> c_int;

    fn nvim_qf_curbuf_has_flag(flag: c_int) -> bool;
    fn nvim_qf_curbuf_fnum() -> c_int;
    fn nvim_qf_curwin_pos_adj() -> *const c_void;
    // (nvim_emsg_e_no_more_items deleted: use emsg directly)
}

/// `:cc`, `:crewind`, `:cfirst`, `:clast`, `:ll`, `:lrewind`, `:lfirst`,
/// `:llast`, `:cdo`, `:ldo`, `:cfdo`, `:lfdo`
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_ex_cc(eap: EapHandle) {
    let qi = nvim_qf_cmd_get_stack(eap, true);
    if qi.is_null() {
        return;
    }

    let cmdidx = nvim_eap_get_cmdidx(eap);
    let addr_count = nvim_eap_get_addr_count(eap);
    let is_do_cmd =
        cmdidx == CMD_CDO || cmdidx == CMD_LDO || cmdidx == CMD_CFDO || cmdidx == CMD_LFDO;

    // For cdo/ldo/cfdo/lfdo commands, jump to the nth valid error/file entry.
    // For other commands, compute the target error number from the address or command type.
    let errornr = if is_do_cmd {
        let n = if addr_count > 0 {
            let line1 = nvim_eap_get_line1(eap);
            if line1 >= 0 {
                line1 as c_int
            } else {
                1
            }
        } else {
            1
        };
        let fdo = cmdidx == CMD_CFDO || cmdidx == CMD_LFDO;
        let qfl = nvim_qf_get_curlist_mut(qi);
        crate::filter::rs_qf_get_nth_valid_entry_do(qfl.cast_const(), n, fdo)
    } else if addr_count > 0 {
        nvim_eap_get_line2(eap) as c_int
    } else {
        match cmdidx {
            CMD_CC | CMD_LL => 0,
            CMD_CREWIND | CMD_LREWIND | CMD_CFIRST | CMD_LFIRST => 1,
            _ => 32767, // CMD_clast, CMD_llast
        }
    };

    crate::navigate::jump_machinery::rs_qf_jump_newwin(
        qi,
        0,
        errornr,
        c_int::from(nvim_eap_get_forceit(eap)),
        false,
    );
}

/// `:cnext`, `:cprevious`, `:cNext`, `:cnfile`, `:cNfile`, `:cpfile`,
/// `:lnext`, `:lprevious`, `:lNext`, `:lnfile`, `:lNfile`, `:lpfile`,
/// `:cdo`, `:ldo`, `:cfdo`, `:lfdo`
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_ex_cnext(eap: EapHandle) {
    let qi = nvim_qf_cmd_get_stack(eap, true);
    if qi.is_null() {
        return;
    }

    let cmdidx = nvim_eap_get_cmdidx(eap);
    let addr_count = nvim_eap_get_addr_count(eap);

    let errornr: c_int = if addr_count > 0
        && cmdidx != CMD_CDO
        && cmdidx != CMD_LDO
        && cmdidx != CMD_CFDO
        && cmdidx != CMD_LFDO
    {
        nvim_eap_get_line2(eap) as c_int
    } else {
        1
    };

    // Depending on the command jump to either next or previous entry/file.
    let dir = match cmdidx {
        CMD_CPREVIOUS | CMD_LPREVIOUS | CMD_CNEXT_BIG | CMD_LNEXT_BIG => BACKWARD,
        CMD_CNFILE | CMD_LNFILE | CMD_CFDO | CMD_LFDO => FORWARD_FILE,
        CMD_CPFILE | CMD_LPFILE | CMD_CNFILE_BIG | CMD_LNFILE_BIG => BACKWARD_FILE,
        _ => FORWARD, // CMD_cnext, CMD_lnext, CMD_cdo, CMD_ldo
    };

    crate::navigate::jump_machinery::rs_qf_jump_newwin(
        qi,
        dir,
        errornr,
        c_int::from(nvim_eap_get_forceit(eap)),
        false,
    );
}

/// `:colder`, `:cnewer`, `:lolder`, `:lnewer`
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_qf_age(eap: EapHandle) {
    let qi = nvim_qf_cmd_get_stack(eap, true);
    if qi.is_null() {
        return;
    }

    let addr_count = nvim_eap_get_addr_count(eap);
    let cmdidx = nvim_eap_get_cmdidx(eap);

    let mut count = if addr_count != 0 {
        nvim_eap_get_line2(eap) as c_int
    } else {
        1
    };

    while count > 0 {
        count -= 1;
        if cmdidx == CMD_COLDER || cmdidx == CMD_LOLDER {
            if nvim_qf_get_curlist_idx(qi) == 0 {
                emsg(c"E380: At bottom of quickfix stack".as_ptr());
                break;
            }
            let new_idx = nvim_qf_get_curlist_idx(qi) - 1;
            nvim_qf_set_curlist_idx(qi, new_idx);
        } else {
            let cur = nvim_qf_get_curlist_idx(qi);
            let lc = nvim_qf_get_listcount(qi);
            if cur >= lc - 1 {
                emsg(c"E381: At top of quickfix stack".as_ptr());
                break;
            }
            nvim_qf_set_curlist_idx(qi, cur + 1);
        }
    }

    let cur = nvim_qf_get_curlist_idx(qi);
    nvim_qf_msg(qi, cur, c"".as_ptr().cast::<u8>());
    nvim_qf_update_buffer(qi, std::ptr::null());
}

/// `:chistory`, `:lhistory`
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_qf_history(eap: EapHandle) {
    let qi = nvim_qf_cmd_get_stack(eap, false);

    if nvim_eap_get_addr_count(eap) > 0 {
        if qi.is_null() {
            emsg(c"E776: No location list".as_ptr());
            return;
        }

        // Jump to the specified quickfix list
        let line2 = nvim_eap_get_line2(eap) as c_int;
        let listcount = nvim_qf_get_listcount(qi);
        if line2 > 0 && line2 <= listcount {
            nvim_qf_set_curlist_idx(qi, line2 - 1);
            nvim_qf_msg(qi, line2 - 1, c"".as_ptr().cast::<u8>());
            nvim_qf_update_buffer(qi, std::ptr::null());
        } else {
            emsg(c"E16: Invalid range".as_ptr());
        }

        return;
    }

    if rs_qf_stack_empty(qi) {
        msg(c"No entries".as_ptr(), 0);
    } else {
        let listcount = nvim_qf_get_listcount(qi);
        let curlist = nvim_qf_get_curlist_idx(qi);
        for i in 0..listcount {
            let lead = if i == curlist {
                c"> ".as_ptr().cast::<u8>()
            } else {
                c"  ".as_ptr().cast::<u8>()
            };
            nvim_qf_msg(qi, i, lead);
        }
    }
}

/// Open the entry/result under the cursor.
/// When `split` is true, open in a new window.
///
/// # Safety
///
/// Must be called with valid global state (curwin, curbuf).
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_qf_view_result(split: bool) {
    let qi = if nvim_qf_curwin_is_ll() {
        nvim_qf_curwin_get_loclist()
    } else {
        nvim_get_ql_info()
    };

    let qfl = nvim_qf_get_curlist_mut(qi);
    if crate::rs_qf_list_empty(qfl) {
        emsg(c"E42: No Errors".as_ptr());
        return;
    }

    if split {
        // Open the selected entry in a new window
        let lnum = nvim_qf_get_cursor_lnum() as c_int;
        crate::navigate::jump_machinery::rs_qf_jump_newwin(qi, 0, lnum, 0, true);
        nvim_do_cmdline_cmd(c"clearjumps".as_ptr().cast::<u8>());
        return;
    }

    if nvim_qf_curwin_is_ll() {
        nvim_do_cmdline_cmd(c".ll".as_ptr().cast::<u8>());
    } else {
        nvim_do_cmdline_cmd(c".cc".as_ptr().cast::<u8>());
    }
}

/// `:cabove`, `:cbelow`, `:labove`, `:lbelow`,
/// `:cafter`, `:cbefore`, `:lafter`, `:lbefore`
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_cbelow(eap: EapHandle) {
    let addr_count = nvim_eap_get_addr_count(eap);
    if addr_count > 0 && nvim_eap_get_line2(eap) <= 0 {
        emsg(c"E16: Invalid range".as_ptr());
        return;
    }

    let cmdidx = nvim_eap_get_cmdidx(eap);

    // Check whether the current buffer has any quickfix entries
    let buf_has_flag = if cmdidx == CMD_CABOVE
        || cmdidx == CMD_CBELOW
        || cmdidx == CMD_CBEFORE
        || cmdidx == CMD_CAFTER
    {
        BUF_HAS_QF_ENTRY
    } else {
        BUF_HAS_LL_ENTRY
    };

    if !nvim_qf_curbuf_has_flag(buf_has_flag) {
        emsg(c"E42: No Errors".as_ptr());
        return;
    }

    let qi = nvim_qf_cmd_get_stack(eap, true);
    if qi.is_null() {
        return;
    }

    let qfl = nvim_qf_get_curlist_mut(qi);
    if !crate::rs_qf_list_has_valid_entries(qfl) {
        emsg(c"E42: No Errors".as_ptr());
        return;
    }

    let n = if addr_count > 0 {
        nvim_eap_get_line2(eap)
    } else {
        0
    };

    let dir: c_int = if cmdidx == CMD_CABOVE || cmdidx == CMD_LABOVE {
        BACKWARD
    } else if cmdidx == CMD_CBELOW || cmdidx == CMD_LBELOW {
        FORWARD
    } else if cmdidx == CMD_CBEFORE || cmdidx == CMD_LBEFORE {
        BACKWARD
    } else {
        FORWARD // CMD_cafter, CMD_lafter
    };

    let linewise = cmdidx == CMD_CABOVE
        || cmdidx == CMD_LABOVE
        || cmdidx == CMD_CBELOW
        || cmdidx == CMD_LBELOW;

    let bnr = nvim_qf_curbuf_fnum();
    let pos = nvim_qf_curwin_pos_adj();
    let errornr = rs_qf_find_nth_adj_entry(qfl, bnr, pos, n, dir, linewise);
    if errornr > 0 {
        crate::navigate::jump_machinery::rs_qf_jump_newwin(qi, 0, errornr, 0, false);
    } else {
        emsg(c"E553: No more items".as_ptr());
    }
}

// =============================================================================
// Phase W2: Window Ex command implementations
// =============================================================================

/// C `linenr_T` is `int32_t`.
type CLinenrT = i32;

extern "C" {
    fn nvim_qf_win_close(win: *mut c_void);
    fn nvim_qf_win_get_cursor_lnum(win: *const c_void) -> CLinenrT;
    fn nvim_qf_win_get_buf_line_count(win: *const c_void) -> CLinenrT;
    // nvim_qf_win_goto_lnum delegates to Rust rs_qf_win_goto_impl
    fn nvim_qf_win_goto_lnum(win: *mut c_void, lnum: CLinenrT);
}

/// Find the quickfix window for a given stack.
/// Returns a mutable pointer to the window, or null.
unsafe fn find_win_for_stack(qi: QfInfoHandleMut) -> *mut c_void {
    // Use the Rust implementation directly (migrated in Phase 10, Pass 10).
    crate::rs_qf_find_win_for_stack(qi.cast_const()).cast_mut()
}

/// `:cclose` / `:lclose` -- close the quickfix/location list window.
///
/// # Safety
/// `eap` must be a valid pointer to a C `exarg_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_cclose(eap: EapHandle) {
    let qi = nvim_qf_cmd_get_stack(eap, false);
    if qi.is_null() {
        return;
    }

    let win = find_win_for_stack(qi);
    if !win.is_null() {
        nvim_qf_win_close(win);
    }
}

/// `:cbottom` / `:lbottom` -- move cursor to last line in qf window.
///
/// # Safety
/// `eap` must be a valid pointer to a C `exarg_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_cbottom(eap: EapHandle) {
    let qi = nvim_qf_cmd_get_stack(eap, true);
    if qi.is_null() {
        return;
    }

    let win = find_win_for_stack(qi);
    if win.is_null() {
        return;
    }

    let cursor_lnum = nvim_qf_win_get_cursor_lnum(win);
    let line_count = nvim_qf_win_get_buf_line_count(win);
    if cursor_lnum != line_count {
        nvim_qf_win_goto_lnum(win, line_count);
    }
}

/// `:cwindow` / `:lwindow` -- open qf window if errors, close if not.
///
/// # Safety
/// `eap` must be a valid pointer to a C `exarg_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_cwindow(eap: EapHandle) {
    let qi = nvim_qf_cmd_get_stack(eap, true);
    if qi.is_null() {
        return;
    }

    let qfl = nvim_qf_get_curlist_mut(qi);

    // Look for an existing quickfix window
    let win = find_win_for_stack(qi);

    // If stack is empty, no valid entries, or list is empty -> close window
    // Otherwise if no window exists -> open it
    if crate::rs_qf_stack_empty(qi.cast_const())
        || crate::nvim_qf_get_nonevalid(qfl.cast_const())
        || crate::rs_qf_list_empty(qfl.cast_const())
    {
        if !win.is_null() {
            rs_ex_cclose(eap);
        }
    } else if win.is_null() {
        rs_ex_copen(eap);
    }
}

// =============================================================================
// Phase W4: ex_copen + qf_goto_cwindow
// =============================================================================

// Constants
const QF_WINHEIGHT: c_int = 10;
const OK_VAL: c_int = 1;
const FAIL_VAL: c_int = 0;
const WSP_VERT: c_int = 0x02;

extern "C" {
    #[link_name = "rs_reset_VIsual_and_resel"]
    fn nvim_qf_reset_visual();
    fn nvim_qf_get_cmdmod_tab() -> c_int;
    fn nvim_qf_get_cmdmod_split() -> c_int;
    #[link_name = "rs_qf_open_new_cwindow"]
    fn nvim_qf_open_new_cwindow(qi: QfInfoHandleMut, height: c_int) -> c_int;
    fn nvim_qf_curwin_set_cursor(lnum: CLinenrT, col: c_int);
    fn nvim_qf_check_cursor_curwin();
    fn nvim_qf_update_topline_curwin();
    fn nvim_qf_win_get_width(win: *const c_void) -> c_int;
    fn nvim_qf_win_get_height(win: *const c_void) -> c_int;
    fn nvim_qf_win_get_hsep_height(win: *const c_void) -> c_int;
    fn nvim_qf_win_get_status_height(win: *const c_void) -> c_int;
    #[link_name = "rs_tabline_height"]
    fn nvim_qf_tabline_height() -> c_int;
    fn nvim_qf_cmdline_row() -> c_int;
    #[link_name = "rs_win_setwidth"]
    fn nvim_qf_win_setwidth(width: c_int);
    #[link_name = "rs_win_setheight"]
    fn nvim_qf_win_setheight(height: c_int);
    fn nvim_qf_win_goto(win: *mut c_void);
    fn nvim_qf_curwin_handle() -> c_int;
    fn nvim_qf_get_curbuf() -> *mut c_void;
}

/// Goto a quickfix or location list window, optionally resizing.
/// Returns OK if found, FAIL if not.
#[allow(clippy::cast_possible_truncation)]
unsafe fn qf_goto_cwindow_rs(
    qi: QfInfoHandleMut,
    resize: bool,
    sz: c_int,
    vertsplit: bool,
) -> c_int {
    let win = find_win_for_stack(qi);
    if win.is_null() {
        return FAIL_VAL;
    }

    nvim_qf_win_goto(win);
    if resize {
        if vertsplit {
            if sz != nvim_qf_win_get_width(win) {
                nvim_qf_win_setwidth(sz);
            }
        } else {
            let win_h = nvim_qf_win_get_height(win);
            let hsep = nvim_qf_win_get_hsep_height(win);
            let status = nvim_qf_win_get_status_height(win);
            let tabline = nvim_qf_tabline_height();
            let cmdrow = nvim_qf_cmdline_row();
            if sz != win_h && (win_h + hsep + status + tabline < cmdrow) {
                nvim_qf_win_setheight(sz);
            }
        }
    }

    OK_VAL
}

/// `:copen` / `:lopen` -- open a window showing the quickfix/location list.
///
/// # Safety
/// `eap` must be a valid pointer to a C `exarg_T`.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_ex_copen(eap: EapHandle) {
    let qi = nvim_qf_cmd_get_stack(eap, true);
    if qi.is_null() {
        return;
    }

    nvim_incr_quickfix_busy();

    let addr_count = nvim_eap_get_addr_count(eap);
    let height = if addr_count != 0 {
        nvim_eap_get_line2(eap) as c_int
    } else {
        let qfl_temp = nvim_qf_get_curlist_mut(qi);
        rs_qf_calc_window_height(qfl_temp.cast_const(), 3, QF_WINHEIGHT)
    };

    nvim_qf_reset_visual();

    // Find an existing quickfix window, or open a new one
    let status = if nvim_qf_get_cmdmod_tab() == 0 {
        qf_goto_cwindow_rs(
            qi,
            addr_count != 0,
            height,
            (nvim_qf_get_cmdmod_split() & WSP_VERT) != 0,
        )
    } else {
        FAIL_VAL
    };
    if status == FAIL_VAL && nvim_qf_open_new_cwindow(qi, height) == FAIL_VAL {
        nvim_decr_quickfix_busy();
        return;
    }

    let qfl = nvim_qf_get_curlist_mut(qi);
    crate::rs_qf_set_title_var(qfl.cast_const());

    // Calculate cursor line
    let mut lnum = crate::window::rs_qf_cursor_line(qfl.cast_const());
    if lnum == 0 {
        lnum = 1;
    }

    // Fill the buffer with quickfix list
    let curbuf = nvim_qf_get_curbuf();
    let curwin_handle = nvim_qf_curwin_handle();
    crate::display::rs_qf_fill_buffer(qfl, curbuf, std::ptr::null(), curwin_handle);

    nvim_decr_quickfix_busy();

    // Position cursor
    nvim_qf_curwin_set_cursor(lnum, 0);
    nvim_qf_check_cursor_curwin();
    nvim_qf_update_topline_curwin();
}

// =============================================================================
// Phase 1: Auname lookups — cmdidx-to-string for QuickFixCmdPre/Post autocmds
// =============================================================================
//
// CMD_* constants from ex_cmds_enum.generated.h (validated with _Static_assert
// in quickfix_shim.c).

const CMD_MAKE: c_int = 273;
const CMD_LMAKE: c_int = 248;
const CMD_GREP: c_int = 172;
const CMD_LGREP: c_int = 239;
const CMD_GREPADD: c_int = 173;
const CMD_LGREPADD: c_int = 240;

const CMD_CFILE: c_int = 65;
const CMD_CGETFILE: c_int = 68;
const CMD_CADDFILE: c_int = 51;
const CMD_LFILE: c_int = 233;
const CMD_LGETFILE: c_int = 236;
const CMD_LADDFILE: c_int = 218;

const CMD_CBUFFER: c_int = 55;
const CMD_CGETBUFFER: c_int = 69;
const CMD_CADDBUFFER: c_int = 49;
const CMD_LBUFFER: c_int = 221;
const CMD_LGETBUFFER: c_int = 237;
const CMD_LADDBUFFER: c_int = 217;

const CMD_CEXPR: c_int = 64;
const CMD_CGETEXPR: c_int = 70;
const CMD_CADDEXPR: c_int = 50;
const CMD_LEXPR: c_int = 232;
const CMD_LGETEXPR: c_int = 238;
const CMD_LADDEXPR: c_int = 216;

const CMD_VIMGREP: c_int = 509;
const CMD_LVIMGREP: c_int = 267;
const CMD_VIMGREPADD: c_int = 510;
const CMD_LVIMGREPADD: c_int = 268;

/// Return the autocmd name for `:make`/`:grep` commands, or NULL.
///
/// # Safety
///
/// Always safe — returns a pointer to a static string literal or null.
#[no_mangle]
pub const extern "C" fn rs_make_get_auname(cmdidx: c_int) -> *const std::ffi::c_char {
    match cmdidx {
        CMD_MAKE => c"make".as_ptr(),
        CMD_LMAKE => c"lmake".as_ptr(),
        CMD_GREP => c"grep".as_ptr(),
        CMD_LGREP => c"lgrep".as_ptr(),
        CMD_GREPADD => c"grepadd".as_ptr(),
        CMD_LGREPADD => c"lgrepadd".as_ptr(),
        _ => std::ptr::null(),
    }
}

/// Return the autocmd name for `:cfile`/`:lfile` commands, or NULL.
///
/// # Safety
///
/// Always safe — returns a pointer to a static string literal or null.
#[no_mangle]
pub const extern "C" fn rs_cfile_get_auname(cmdidx: c_int) -> *const std::ffi::c_char {
    match cmdidx {
        CMD_CFILE => c"cfile".as_ptr(),
        CMD_CGETFILE => c"cgetfile".as_ptr(),
        CMD_CADDFILE => c"caddfile".as_ptr(),
        CMD_LFILE => c"lfile".as_ptr(),
        CMD_LGETFILE => c"lgetfile".as_ptr(),
        CMD_LADDFILE => c"laddfile".as_ptr(),
        _ => std::ptr::null(),
    }
}

/// Return the autocmd name for `:cbuffer`/`:lbuffer` commands, or NULL.
///
/// # Safety
///
/// Always safe — returns a pointer to a static string literal or null.
#[no_mangle]
pub const extern "C" fn rs_cbuffer_get_auname(cmdidx: c_int) -> *const std::ffi::c_char {
    match cmdidx {
        CMD_CBUFFER => c"cbuffer".as_ptr(),
        CMD_CGETBUFFER => c"cgetbuffer".as_ptr(),
        CMD_CADDBUFFER => c"caddbuffer".as_ptr(),
        CMD_LBUFFER => c"lbuffer".as_ptr(),
        CMD_LGETBUFFER => c"lgetbuffer".as_ptr(),
        CMD_LADDBUFFER => c"laddbuffer".as_ptr(),
        _ => std::ptr::null(),
    }
}

/// Return the autocmd name for `:cexpr`/`:lexpr` commands, or NULL.
///
/// # Safety
///
/// Always safe — returns a pointer to a static string literal or null.
#[no_mangle]
pub const extern "C" fn rs_cexpr_get_auname(cmdidx: c_int) -> *const std::ffi::c_char {
    match cmdidx {
        CMD_CEXPR => c"cexpr".as_ptr(),
        CMD_CGETEXPR => c"cgetexpr".as_ptr(),
        CMD_CADDEXPR => c"caddexpr".as_ptr(),
        CMD_LEXPR => c"lexpr".as_ptr(),
        CMD_LGETEXPR => c"lgetexpr".as_ptr(),
        CMD_LADDEXPR => c"laddexpr".as_ptr(),
        _ => std::ptr::null(),
    }
}

/// Return the autocmd name for `:vimgrep`/`:grep` commands, or NULL.
///
/// # Safety
///
/// Always safe — returns a pointer to a static string literal or null.
#[no_mangle]
pub const extern "C" fn rs_vgr_get_auname(cmdidx: c_int) -> *const std::ffi::c_char {
    match cmdidx {
        CMD_VIMGREP => c"vimgrep".as_ptr(),
        CMD_LVIMGREP => c"lvimgrep".as_ptr(),
        CMD_VIMGREPADD => c"vimgrepadd".as_ptr(),
        CMD_LVIMGREPADD => c"lvimgrepadd".as_ptr(),
        CMD_GREP => c"grep".as_ptr(),
        CMD_LGREP => c"lgrep".as_ptr(),
        CMD_GREPADD => c"grepadd".as_ptr(),
        CMD_LGREPADD => c"lgrepadd".as_ptr(),
        _ => std::ptr::null(),
    }
}

// =============================================================================
// Phase 4 pass: Stack query entry points
// =============================================================================

// CMD_* constants for valid-size counting (from ex_cmds_enum.generated.h,
// validated by _Static_assert in quickfix_shim.c)
const CMD_CDO_P4: c_int = 62; // CMD_cdo
const CMD_LDO_P4: c_int = 228; // CMD_ldo
const CMD_CFDO_P4: c_int = 66; // CMD_cfdo
const CMD_LFDO_P4: c_int = 234; // CMD_lfdo

// CMD_* constants for grep_internal (validated by _Static_assert in quickfix_shim.c)
const CMD_GREP_P4: c_int = 172;
const CMD_LGREP_P4: c_int = 239;
const CMD_GREPADD_P4: c_int = 173;
const CMD_LGREPADD_P4: c_int = 240;

extern "C" {
    fn nvim_grep_uses_internal() -> bool;
}

/// Returns the number of entries in the current quickfix/location list.
///
/// Thin Rust wrapper around `qf_cmd_get_stack` + list count.
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[export_name = "qf_get_size"]
pub unsafe extern "C" fn rs_qf_get_size_eap(eap: EapHandle) -> usize {
    let qi = nvim_qf_cmd_get_stack(eap, false);
    if qi.is_null() {
        return 0;
    }
    let qfl = nvim_qf_get_curlist_mut(qi).cast_const();
    #[allow(clippy::cast_sign_loss)]
    {
        nvim_qf_get_count(qfl).max(0) as usize
    }
}

/// Returns the number of valid entries in the current quickfix/location list.
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[export_name = "qf_get_valid_size"]
pub unsafe extern "C" fn rs_qf_get_valid_size_eap(eap: EapHandle) -> usize {
    let qi = nvim_qf_cmd_get_stack(eap, false);
    if qi.is_null() {
        return 0;
    }
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let count_files = !(cmdidx == CMD_CDO_P4 || cmdidx == CMD_LDO_P4);
    let qfl = nvim_qf_get_curlist_mut(qi).cast_const();
    #[allow(clippy::cast_sign_loss)]
    {
        crate::navigate::rs_qf_get_valid_size(qfl, count_files).max(0) as usize
    }
}

/// Returns the current entry index (0 if error).
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[export_name = "qf_get_cur_idx"]
pub unsafe extern "C" fn rs_qf_get_cur_idx_eap(eap: EapHandle) -> usize {
    let qi = nvim_qf_cmd_get_stack(eap, false);
    if qi.is_null() {
        return 0;
    }
    let qfl = nvim_qf_get_curlist_mut(qi).cast_const();
    let idx = nvim_qf_get_index(qfl);
    debug_assert!(idx >= 0);
    #[allow(clippy::cast_sign_loss)]
    {
        idx.max(0) as usize
    }
}

/// Returns the current valid entry index (1 if no valid entries).
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[export_name = "qf_get_cur_valid_idx"]
pub unsafe extern "C" fn rs_qf_get_cur_valid_idx_eap(eap: EapHandle) -> c_int {
    let qi = nvim_qf_cmd_get_stack(eap, false);
    if qi.is_null() {
        return 1;
    }
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let count_files = cmdidx == CMD_CFDO_P4 || cmdidx == CMD_LFDO_P4;
    let qfl = nvim_qf_get_curlist_mut(qi).cast_const();
    let qf_index = nvim_qf_get_index(qfl);
    crate::navigate::rs_qf_get_cur_valid_idx(qfl, qf_index, count_files)
}

/// Opaque window handle
type WinHandle = *const c_void;

extern "C" {
    fn nvim_qf_is_ll_window(wp: WinHandle) -> bool;
    fn nvim_win_get_llist_ref(wp: WinHandle) -> QfInfoHandle;
}

/// Returns the line number of the current entry in the quickfix/location list
/// for the given window (used for cursor positioning in the qf window).
///
/// # Panics
///
/// Panics if the global quickfix info pointer is null.
///
/// # Safety
///
/// `wp` must be a valid pointer to a C `win_T`.
#[export_name = "qf_current_entry"]
#[must_use]
pub unsafe extern "C" fn rs_qf_current_entry(wp: WinHandle) -> i32 {
    let mut qi: QfInfoHandleMut = nvim_get_ql_info();
    assert!(!qi.is_null());

    if nvim_qf_is_ll_window(wp) {
        qi = nvim_win_get_llist_ref(wp).cast_mut();
    }

    let qfl = nvim_qf_get_curlist_mut(qi).cast_const();
    nvim_qf_get_index(qfl)
}

/// Returns true when using `:vimgrep` for `:grep`
/// (i.e., cmdidx is a grep command and 'grepprg' is "internal").
///
/// # Safety
///
/// Always safe — reads global state only.
#[export_name = "grep_internal"]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_grep_internal(cmdidx: c_int) -> c_int {
    let is_grep_cmd = cmdidx == CMD_GREP_P4
        || cmdidx == CMD_LGREP_P4
        || cmdidx == CMD_GREPADD_P4
        || cmdidx == CMD_LGREPADD_P4;
    i32::from(is_grep_cmd && nvim_grep_uses_internal())
}

// =============================================================================
// Phase 2: qf_cmdtitle helper
// =============================================================================

/// Write `:cmd` into `buf` (at most `bufsz` bytes including NUL terminator).
///
/// Returns the number of bytes written, not counting the NUL terminator.
///
/// # Safety
///
/// - `cmd` must be a valid null-terminated C string, or NULL.
/// - `buf` must be a valid writable buffer of at least `bufsz` bytes.
/// - `bufsz` must be > 0.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_cmdtitle(
    cmd: *const std::ffi::c_char,
    buf: *mut std::ffi::c_char,
    bufsz: usize,
) -> usize {
    use std::io::Write;

    if buf.is_null() || bufsz == 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts_mut(buf.cast::<u8>(), bufsz);

    let cmd_str = if cmd.is_null() {
        ""
    } else {
        std::ffi::CStr::from_ptr(cmd).to_str().unwrap_or_default()
    };

    let mut cursor = std::io::Cursor::new(&mut slice[..]);
    let _ = write!(cursor, ":{cmd_str}");
    #[allow(clippy::cast_possible_truncation)]
    let written = cursor.position() as usize;
    let end = written.min(bufsz - 1);
    slice[end] = 0;
    end
}

// =============================================================================
// Phase 4: rs_ex_clist — :clist/:llist command
// =============================================================================

extern "C" {
    // nvim_eap_get_arg from ex_docmd.c — returns mut char* (we only read)
    fn nvim_eap_get_arg(eap: EapHandle) -> *mut std::ffi::c_char;
    // nvim_eap_get_forceit from indent_ffi.c — returns bool
    // (already declared in the existing extern block above)
    fn nvim_get_list_range(
        arg: *mut *mut std::ffi::c_char,
        idx1: *mut c_int,
        idx2: *mut c_int,
    ) -> bool;
    // nvim_semsg_trailing_arg: now in nvim_eval::errors
    fn nvim_shorten_fnames_qf();
    fn nvim_syn_name2id_qf(name: *const std::ffi::c_char) -> c_int;
    fn nvim_hlf_d() -> c_int;
    fn nvim_hlf_n() -> c_int;
    fn nvim_got_int_qf() -> bool;
    fn nvim_os_breakcheck_qf();
    fn rs_qf_list_entry(
        qfp: *const c_void,
        qf_idx: c_int,
        cursel: bool,
        qf_file_hl_id: c_int,
        qf_sep_hl_id: c_int,
        qf_line_hl_id: c_int,
    );
}

/// `:clist` / `:llist` — list all quickfix/location list entries.
///
/// # Safety
///
/// `eap` must be a valid pointer to a C `exarg_T`.
#[export_name = "qf_list"]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_ex_clist(eap: EapHandle) {
    let qi = nvim_qf_cmd_get_stack(eap, true);
    if qi.is_null() {
        return;
    }

    let qfl = crate::nvim_qf_get_curlist(qi);
    if crate::rs_qf_stack_empty(qi) || crate::rs_qf_list_empty(qfl) {
        emsg(c"E42: No Errors".as_ptr());
        return;
    }

    // Get the arg pointer (mutable C string for get_list_range)
    let mut arg = nvim_eap_get_arg(eap);
    let forceit = nvim_eap_get_forceit(eap);

    // Handle '+' prefix: list from current entry
    #[allow(clippy::cast_possible_wrap)]
    let plus = if !arg.is_null() && *arg == b'+' as std::ffi::c_char {
        arg = arg.add(1);
        true
    } else {
        false
    };

    let mut idx1: c_int = 1;
    let mut idx2: c_int = -1;

    if !nvim_get_list_range(&raw mut arg, &raw mut idx1, &raw mut idx2)
        || (!arg.is_null() && *arg != 0)
    {
        nvim_eval::errors::semsg_trailing_arg(arg.cast_const());
        return;
    }

    // Compute range
    let (idx1, idx2) = if plus {
        let cur = crate::nvim_qf_get_index(qfl);
        let new_idx2 = cur + idx1;
        (cur, new_idx2)
    } else {
        let count = crate::nvim_qf_get_count(qfl);
        let i1 = if idx1 < 0 {
            if -idx1 > count {
                0
            } else {
                idx1 + count + 1
            }
        } else {
            idx1
        };
        let i2 = if idx2 < 0 {
            if -idx2 > count {
                0
            } else {
                idx2 + count + 1
            }
        } else {
            idx2
        };
        (i1, i2)
    };

    // Shorten all file names for display
    nvim_shorten_fnames_qf();

    // Set up highlight IDs
    let mut qf_file_hl_id = nvim_syn_name2id_qf(c"qfFileName".as_ptr());
    if qf_file_hl_id == 0 {
        qf_file_hl_id = nvim_hlf_d();
    }
    let mut qf_sep_hl_id = nvim_syn_name2id_qf(c"qfSeparator".as_ptr());
    if qf_sep_hl_id == 0 {
        qf_sep_hl_id = nvim_hlf_d();
    }
    let mut qf_line_hl_id = nvim_syn_name2id_qf(c"qfLineNr".as_ptr());
    if qf_line_hl_id == 0 {
        qf_line_hl_id = nvim_hlf_n();
    }

    // If nonevalid is set, show all entries (forceit is bool from nvim_eap_get_forceit)
    let all = forceit || crate::nvim_qf_get_nonevalid(qfl);

    // Iterate: FOR_ALL_QFL_ITEMS equivalent
    let qf_index = crate::nvim_qf_get_index(qfl);
    let qf_count = crate::nvim_qf_get_count(qfl);
    let mut i: c_int = 1;
    let mut qfp = crate::nvim_qf_get_start(qfl);
    while !nvim_got_int_qf() && i <= qf_count && !qfp.is_null() {
        let valid = crate::nvim_qfline_get_valid(qfp);
        if (valid || all) && idx1 <= i && i <= idx2 {
            rs_qf_list_entry(
                qfp,
                i,
                i == qf_index,
                qf_file_hl_id,
                qf_sep_hl_id,
                qf_line_hl_id,
            );
        }
        nvim_os_breakcheck_qf();
        i += 1;
        qfp = crate::nvim_qfline_get_next(qfp);
    }
}
