//! Quickfix window integration and display formatting.
//!
//! This module provides functions for window positioning, cursor management,
//! and display-related calculations in the quickfix window.

use std::ffi::{c_char, c_int};

use crate::{
    nvim_qf_get_count, nvim_qf_get_index, nvim_qf_get_ptr, nvim_qf_get_start, nvim_qfline_get_col,
    nvim_qfline_get_end_col, nvim_qfline_get_end_lnum, nvim_qfline_get_fnum, nvim_qfline_get_lnum,
    nvim_qfline_get_next, nvim_qfline_get_nr, nvim_qfline_get_type, nvim_qfline_get_valid, LinenrT,
    QfLineHandle, QfListHandle,
};

// =============================================================================
// Cursor Positioning
// =============================================================================

/// Calculate the optimal cursor line for the quickfix window.
///
/// Returns the line number (1-based) to position the cursor at,
/// based on the current quickfix index.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_cursor_line(qfl: QfListHandle) -> LinenrT {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        return 0;
    }

    // The cursor line in the qf window corresponds to the current index
    let idx = nvim_qf_get_index(qfl);
    if idx > 0 && idx <= count {
        idx
    } else {
        1
    }
}

/// Check if cursor should be updated after an operation.
///
/// Returns true if the old and new indices differ.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_should_update_cursor(qfl: QfListHandle, old_idx: c_int) -> bool {
    if qfl.is_null() {
        return false;
    }

    let new_idx = nvim_qf_get_index(qfl);
    new_idx != old_idx
}

// =============================================================================
// Display Range Calculations
// =============================================================================

/// Window display range information.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct QfDisplayRange {
    /// First entry to display (1-based).
    pub start_idx: c_int,
    /// Last entry to display (1-based).
    pub end_idx: c_int,
    /// Total number of entries.
    pub total: c_int,
    /// Current entry index.
    pub current: c_int,
}

/// Calculate the display range for the quickfix window.
///
/// Calculates which entries should be visible in the window, ensuring
/// the current entry is visible with appropriate context.
///
/// # Safety
///
/// - `qfl` may be null (returns zeroed range)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_calc_display_range(
    qfl: QfListHandle,
    window_height: c_int,
) -> QfDisplayRange {
    let mut range = QfDisplayRange::default();

    if qfl.is_null() || window_height <= 0 {
        return range;
    }

    let count = nvim_qf_get_count(qfl);
    let current = nvim_qf_get_index(qfl);

    range.total = count;
    range.current = current;

    if count == 0 {
        return range;
    }

    // If all entries fit in window, show all
    if count <= window_height {
        range.start_idx = 1;
        range.end_idx = count;
        return range;
    }

    // Center the current entry in the window if possible
    let half_window = window_height / 2;
    let initial_start = current - half_window;

    let (start, end) = if initial_start < 1 {
        // Adjust if we go before the first entry
        (1, window_height)
    } else {
        let initial_end = initial_start + window_height - 1;
        if initial_end > count {
            // Adjust if we go past the last entry
            let adjusted_start = (count - window_height + 1).max(1);
            (adjusted_start, count)
        } else {
            (initial_start, initial_end)
        }
    };

    range.start_idx = start;
    range.end_idx = end;

    range
}

// =============================================================================
// Entry Display Information
// =============================================================================

/// Display information for a single quickfix entry.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct QfEntryDisplay {
    /// File buffer number (0 if no file).
    pub fnum: c_int,
    /// Line number (0 if not applicable).
    pub lnum: LinenrT,
    /// Column number (0 if not applicable).
    pub col: c_int,
    /// End line number (0 if not a range).
    pub end_lnum: LinenrT,
    /// End column (0 if not a range).
    pub end_col: c_int,
    /// Entry number (0 if not set).
    pub nr: c_int,
    /// Entry type character ('E', 'W', 'I', 'N', ' ').
    pub entry_type: c_char,
    /// Whether this is a valid entry.
    pub valid: bool,
    /// Whether this entry has position info.
    pub has_position: bool,
    /// Whether this entry has a range.
    pub has_range: bool,
}

/// Get display information for a quickfix entry.
///
/// # Safety
///
/// - `qfp` may be null (returns default/empty info)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_display_info(qfp: QfLineHandle) -> QfEntryDisplay {
    let mut info = QfEntryDisplay::default();

    if qfp.is_null() {
        return info;
    }

    info.fnum = nvim_qfline_get_fnum(qfp);
    info.lnum = nvim_qfline_get_lnum(qfp);
    info.col = nvim_qfline_get_col(qfp);
    info.end_lnum = nvim_qfline_get_end_lnum(qfp);
    info.end_col = nvim_qfline_get_end_col(qfp);
    info.nr = nvim_qfline_get_nr(qfp);
    info.entry_type = nvim_qfline_get_type(qfp);
    info.valid = nvim_qfline_get_valid(qfp);
    info.has_position = info.lnum > 0;
    info.has_range = info.end_lnum > 0 || info.end_col > 0;

    info
}

/// Get display information for the current entry.
///
/// # Safety
///
/// - `qfl` may be null (returns default/empty info)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_current_entry_display(qfl: QfListHandle) -> QfEntryDisplay {
    if qfl.is_null() {
        return QfEntryDisplay::default();
    }

    let qfp = nvim_qf_get_ptr(qfl);
    rs_qf_entry_display_info(qfp)
}

// =============================================================================
// Highlight Information
// =============================================================================

/// Highlight region in a quickfix line.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct QfHighlightRegion {
    /// Start column (0-based byte offset).
    pub start_col: c_int,
    /// End column (0-based byte offset, exclusive).
    pub end_col: c_int,
    /// Highlight group ID (to be set by caller).
    pub hl_id: c_int,
}

/// Highlight information for a quickfix line.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct QfLineHighlights {
    /// Filename region.
    pub filename: QfHighlightRegion,
    /// Line number region.
    pub lnum: QfHighlightRegion,
    /// Column number region.
    pub col: QfHighlightRegion,
    /// Separator regions (the '|' characters).
    pub sep1: QfHighlightRegion,
    /// Second separator.
    pub sep2: QfHighlightRegion,
    /// Error type region.
    pub error_type: QfHighlightRegion,
    /// Valid entry (has any highlight regions).
    pub valid: bool,
}

/// Calculate highlight regions for a quickfix display line.
///
/// The line format is: `filename|lnum col type|text` or `filename|pattern|text`
///
/// # Safety
///
/// - `line` may be null (returns empty highlights)
/// - If non-null, `line` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_calc_line_highlights(line: *const c_char) -> QfLineHighlights {
    let mut hl = QfLineHighlights::default();

    if line.is_null() {
        return hl;
    }

    let Ok(line_str) = std::ffi::CStr::from_ptr(line).to_str() else {
        return hl;
    };

    // Find the first '|' separator
    let Some(sep1_pos) = line_str.find('|') else {
        return hl;
    };

    // Filename is before the first '|'
    hl.filename.start_col = 0;
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    {
        hl.filename.end_col = sep1_pos as c_int;
    }

    // First separator
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    {
        hl.sep1.start_col = sep1_pos as c_int;
        hl.sep1.end_col = (sep1_pos + 1) as c_int;
    }

    // Find the second '|' separator
    let rest = &line_str[sep1_pos + 1..];
    if let Some(sep2_offset) = rest.find('|') {
        let sep2_pos = sep1_pos + 1 + sep2_offset;

        // Line/col info is between the separators
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        {
            hl.lnum.start_col = (sep1_pos + 1) as c_int;
            hl.lnum.end_col = sep2_pos as c_int;
        }

        // Second separator
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        {
            hl.sep2.start_col = sep2_pos as c_int;
            hl.sep2.end_col = (sep2_pos + 1) as c_int;
        }

        hl.valid = true;
    }

    hl
}

// =============================================================================
// Window State
// =============================================================================

/// Check if the quickfix window needs to be scrolled to show the current entry.
///
/// Returns:
/// - 0 if the current entry is already visible
/// - Positive number: lines to scroll down
/// - Negative number: lines to scroll up
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_calc_scroll(
    qfl: QfListHandle,
    top_line: LinenrT,
    window_height: c_int,
) -> c_int {
    if qfl.is_null() || window_height <= 0 {
        return 0;
    }

    let current = nvim_qf_get_index(qfl);
    if current <= 0 {
        return 0;
    }

    let bottom_line = top_line + window_height - 1;

    if current < top_line {
        // Need to scroll up
        current - top_line
    } else if current > bottom_line {
        // Need to scroll down
        current - bottom_line
    } else {
        // Current entry is visible
        0
    }
}

/// Calculate the top line to center the current entry in the window.
///
/// # Safety
///
/// - `qfl` may be null (returns 1)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_calc_centered_top(
    qfl: QfListHandle,
    window_height: c_int,
) -> LinenrT {
    if qfl.is_null() || window_height <= 0 {
        return 1;
    }

    let current = nvim_qf_get_index(qfl);
    let count = nvim_qf_get_count(qfl);

    if current <= 0 || count <= 0 {
        return 1;
    }

    // Center the current entry
    let half = window_height / 2;
    let top = current - half;

    if top < 1 {
        1
    } else if top + window_height - 1 > count {
        // Don't leave blank space at the bottom
        let adjusted = count - window_height + 1;
        if adjusted < 1 {
            1
        } else {
            adjusted
        }
    } else {
        top
    }
}

// =============================================================================
// Entry Iteration for Display
// =============================================================================

/// Get the entry at a specific display line (1-based).
///
/// # Safety
///
/// - `qfl` may be null (returns null)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_at_line(qfl: QfListHandle, line: c_int) -> QfLineHandle {
    if qfl.is_null() || line <= 0 {
        return std::ptr::null();
    }

    let count = nvim_qf_get_count(qfl);
    if line > count {
        return std::ptr::null();
    }

    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() && idx < line {
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    qfp
}

/// Count entries that have position information (for sign placement).
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_with_position(qfl: QfListHandle) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        if nvim_qfline_get_lnum(qfp) > 0 {
            count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
    }

    count
}

/// Count entries that have file association (for virtual text/signs).
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_with_file(qfl: QfListHandle) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        if nvim_qfline_get_fnum(qfp) > 0 {
            count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
    }

    count
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_safety() {
        unsafe {
            assert_eq!(rs_qf_cursor_line(std::ptr::null()), 0);
            assert!(!rs_qf_should_update_cursor(std::ptr::null(), 0));

            let range = rs_qf_calc_display_range(std::ptr::null(), 10);
            assert_eq!(range.total, 0);
            assert_eq!(range.current, 0);

            let info = rs_qf_entry_display_info(std::ptr::null());
            assert_eq!(info.fnum, 0);
            assert!(!info.valid);

            let info = rs_qf_current_entry_display(std::ptr::null());
            assert_eq!(info.fnum, 0);

            let hl = rs_qf_calc_line_highlights(std::ptr::null());
            assert!(!hl.valid);

            assert_eq!(rs_qf_calc_scroll(std::ptr::null(), 1, 10), 0);
            assert_eq!(rs_qf_calc_centered_top(std::ptr::null(), 10), 1);
            assert!(rs_qf_entry_at_line(std::ptr::null(), 1).is_null());
            assert_eq!(rs_qf_count_with_position(std::ptr::null()), 0);
            assert_eq!(rs_qf_count_with_file(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_display_range_default() {
        let range = QfDisplayRange::default();
        assert_eq!(range.start_idx, 0);
        assert_eq!(range.end_idx, 0);
        assert_eq!(range.total, 0);
        assert_eq!(range.current, 0);
    }

    #[test]
    fn test_entry_display_default() {
        let info = QfEntryDisplay::default();
        assert_eq!(info.fnum, 0);
        assert_eq!(info.lnum, 0);
        assert!(!info.valid);
        assert!(!info.has_position);
        assert!(!info.has_range);
    }

    #[test]
    fn test_highlight_regions() {
        unsafe {
            // Test a typical quickfix line format
            let line = c"file.c|10 col 5 error|some message";
            let hl = rs_qf_calc_line_highlights(line.as_ptr());
            assert!(hl.valid);
            assert_eq!(hl.filename.start_col, 0);
            assert_eq!(hl.filename.end_col, 6); // "file.c"
            assert_eq!(hl.sep1.start_col, 6);
            assert_eq!(hl.sep1.end_col, 7);
        }
    }

    #[test]
    fn test_highlight_no_second_separator() {
        unsafe {
            // Line with only one separator
            let line = c"something|without second sep";
            let hl = rs_qf_calc_line_highlights(line.as_ptr());
            assert!(!hl.valid); // No second separator found
        }
    }

    #[test]
    fn test_highlight_no_separator() {
        unsafe {
            // Line with no separators
            let line = c"no separators here";
            let hl = rs_qf_calc_line_highlights(line.as_ptr());
            assert!(!hl.valid);
        }
    }
}
