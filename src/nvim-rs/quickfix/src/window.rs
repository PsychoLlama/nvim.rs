//! Quickfix window integration and display formatting.
//!
//! This module provides functions for window positioning, cursor management,
//! and display-related calculations in the quickfix window.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

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
// Text Formatting
// =============================================================================

/// Format quickfix entry text for display.
///
/// Converts newlines to spaces and collapses whitespace after newlines.
/// Writes the result to `out` buffer with max `out_size` bytes.
///
/// # Safety
///
/// - `text` may be null (writes empty string)
/// - `out` must be a valid pointer to a buffer of at least `out_size` bytes
/// - If non-null, `text` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_fmt_text(
    text: *const c_char,
    out: *mut c_char,
    out_size: usize,
) -> usize {
    if out.is_null() || out_size == 0 {
        return 0;
    }

    // Write empty string if text is null
    if text.is_null() {
        *out = 0;
        return 0;
    }

    let Ok(text_str) = std::ffi::CStr::from_ptr(text).to_str() else {
        *out = 0;
        return 0;
    };

    let mut written = 0;
    let max_write = out_size - 1; // Reserve space for null terminator
    let mut chars = text_str.chars().peekable();

    while let Some(c) = chars.next() {
        if written >= max_write {
            break;
        }

        if c == '\n' {
            // Replace newline with space
            *out.add(written) = b' ' as c_char;
            written += 1;

            // Skip following whitespace and newlines
            while let Some(&next) = chars.peek() {
                if next == ' ' || next == '\t' || next == '\n' {
                    chars.next();
                } else {
                    break;
                }
            }
        } else {
            // Copy character (only single-byte chars for C compatibility)
            let byte = c as u32;
            if byte <= 127 {
                *out.add(written) = byte as c_char;
                written += 1;
            } else {
                // For multi-byte UTF-8, copy each byte
                let mut buf = [0u8; 4];
                let encoded = c.encode_utf8(&mut buf);
                for &b in encoded.as_bytes() {
                    if written >= max_write {
                        break;
                    }
                    *out.add(written) = b as c_char;
                    written += 1;
                }
            }
        }
    }

    // Null terminate
    *out.add(written) = 0;
    written
}

/// Format range text for a quickfix entry (e.g., "10-15 col 5-8").
///
/// Writes the formatted range to `out` buffer with max `out_size` bytes.
///
/// # Safety
///
/// - `out` must be a valid pointer to a buffer of at least `out_size` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_qf_range_text(
    lnum: LinenrT,
    end_lnum: LinenrT,
    col: c_int,
    end_col: c_int,
    out: *mut c_char,
    out_size: usize,
) -> usize {
    use std::fmt::Write;

    if out.is_null() || out_size == 0 {
        return 0;
    }

    let mut result = String::with_capacity(64);

    // Write line number
    let _ = write!(result, "{lnum}");

    // Write end line if different
    if end_lnum > 0 && end_lnum != lnum {
        let _ = write!(result, "-{end_lnum}");
    }

    // Write column if present
    if col > 0 {
        let _ = write!(result, " col {col}");

        // Write end column if different
        if end_col > 0 && end_col != col {
            let _ = write!(result, "-{end_col}");
        }
    }

    // Copy to output buffer
    let bytes = result.as_bytes();
    let copy_len = bytes.len().min(out_size - 1);

    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out.cast::<u8>(), copy_len);
    *out.add(copy_len) = 0;

    copy_len
}

/// Format a quickfix entry for display in the quickfix buffer.
///
/// Produces a line in the format: "filename|lnum col type|text"
///
/// # Safety
///
/// - `qfp` may be null (writes empty string)
/// - `out` must be a valid pointer to a buffer of at least `out_size` bytes
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_format_entry_line(
    qfp: QfLineHandle,
    fname: *const c_char,
    out: *mut c_char,
    out_size: usize,
) -> usize {
    use std::fmt::Write;

    if out.is_null() || out_size == 0 {
        return 0;
    }

    if qfp.is_null() {
        *out = 0;
        return 0;
    }

    let mut result = String::with_capacity(256);

    // Add filename if present
    if !fname.is_null() {
        if let Ok(fname_str) = std::ffi::CStr::from_ptr(fname).to_str() {
            result.push_str(fname_str);
        }
    }

    result.push('|');

    // Add position info
    let lnum = nvim_qfline_get_lnum(qfp);
    if lnum > 0 {
        let _ = write!(result, "{lnum}");

        let end_lnum = nvim_qfline_get_end_lnum(qfp);
        if end_lnum > 0 && end_lnum != lnum {
            let _ = write!(result, "-{end_lnum}");
        }

        let col = nvim_qfline_get_col(qfp);
        if col > 0 {
            let _ = write!(result, " col {col}");

            let end_col = nvim_qfline_get_end_col(qfp);
            if end_col > 0 && end_col != col {
                let _ = write!(result, "-{end_col}");
            }
        }
    }

    // Add entry type
    let entry_type = nvim_qfline_get_type(qfp);
    if entry_type != 0 && entry_type != b' ' as c_char {
        let _ = write!(result, " {}", entry_type as u8 as char);
    }

    result.push('|');

    // Copy to output buffer (text is added by caller)
    let bytes = result.as_bytes();
    let copy_len = bytes.len().min(out_size - 1);

    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out.cast::<u8>(), copy_len);
    *out.add(copy_len) = 0;

    copy_len
}

// =============================================================================
// Phase Q5: Additional Window Integration Helpers
// =============================================================================

/// Window size constraints for quickfix window.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QfWindowSize {
    /// Minimum height (lines)
    pub min_height: c_int,
    /// Maximum height (lines)
    pub max_height: c_int,
    /// Preferred height based on entry count
    pub preferred_height: c_int,
    /// Whether to use a horizontal split
    pub horizontal: bool,
}

impl Default for QfWindowSize {
    fn default() -> Self {
        Self {
            min_height: 3,
            max_height: 10,
            preferred_height: 10,
            horizontal: true,
        }
    }
}

/// Calculate optimal quickfix window size.
///
/// # Safety
///
/// - `qfl` may be null (returns defaults)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_calc_window_size(
    qfl: QfListHandle,
    min_height: c_int,
    max_height: c_int,
) -> QfWindowSize {
    let mut size = QfWindowSize {
        min_height: min_height.max(1),
        max_height: max_height.max(min_height),
        ..Default::default()
    };

    if qfl.is_null() {
        size.preferred_height = size.min_height;
        return size;
    }

    let count = nvim_qf_get_count(qfl);
    size.preferred_height = count.clamp(size.min_height, size.max_height);

    size
}

/// Check if quickfix window should be opened/updated.
///
/// Returns true if there are entries to display.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_should_open_window(qfl: QfListHandle) -> bool {
    if qfl.is_null() {
        return false;
    }

    nvim_qf_get_count(qfl) > 0
}

/// Calculate the line to scroll to after an update.
///
/// Returns the optimal top line to show the current entry.
///
/// # Safety
///
/// - `qfl` may be null (returns 1)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_scroll_to_current(
    qfl: QfListHandle,
    window_height: c_int,
    context_lines: c_int,
) -> LinenrT {
    if qfl.is_null() || window_height <= 0 {
        return 1;
    }

    let current = nvim_qf_get_index(qfl);
    let count = nvim_qf_get_count(qfl);

    if current <= 0 || count <= 0 {
        return 1;
    }

    // Keep context_lines above the current entry if possible
    let desired_top = (current - context_lines).max(1);

    // But don't leave blank space at bottom
    let max_top = (count - window_height + 1).max(1);

    desired_top.min(max_top)
}

/// Get the buffer line number for an entry index.
///
/// In the quickfix window, the buffer line number equals the entry index.
/// This function validates the index and returns the corresponding line.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_to_buf_line(qfl: QfListHandle, entry_idx: c_int) -> LinenrT {
    if qfl.is_null() || entry_idx <= 0 {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if entry_idx > count {
        return 0;
    }

    entry_idx
}

/// Get the entry index for a buffer line number.
///
/// Inverse of `rs_qf_entry_to_buf_line`.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_buf_line_to_entry(qfl: QfListHandle, buf_line: LinenrT) -> c_int {
    if qfl.is_null() || buf_line <= 0 {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if buf_line > count {
        return 0;
    }

    buf_line
}

/// Check if the quickfix window buffer is valid.
///
/// Returns true if the buffer line count matches the quickfix entry count.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_window_buf_valid(
    qfl: QfListHandle,
    buf_line_count: LinenrT,
) -> bool {
    if qfl.is_null() {
        return false;
    }

    let count = nvim_qf_get_count(qfl);
    buf_line_count == count
}

/// Calculate how many lines need to be added/removed to sync buffer with list.
///
/// Returns:
/// - Positive: lines to add
/// - Negative: lines to remove
/// - Zero: buffer is in sync
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_window_buf_delta(
    qfl: QfListHandle,
    buf_line_count: LinenrT,
) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    count - buf_line_count
}

/// Information about quickfix window state.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct QfWindowInfo {
    /// Whether the quickfix window exists
    pub exists: bool,
    /// Current entry index (1-based)
    pub current_idx: c_int,
    /// Total entry count
    pub total_count: c_int,
    /// Number of valid entries
    pub valid_count: c_int,
    /// Whether current entry is valid
    pub current_is_valid: bool,
}

/// Get information about the quickfix window state.
///
/// # Safety
///
/// - `qfl` may be null (returns defaults)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_window_info(qfl: QfListHandle) -> QfWindowInfo {
    let mut info = QfWindowInfo::default();

    if qfl.is_null() {
        return info;
    }

    info.exists = true;
    info.current_idx = nvim_qf_get_index(qfl);
    info.total_count = nvim_qf_get_count(qfl);

    // Count valid entries and check if current is valid
    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() {
        if nvim_qfline_get_valid(qfp) {
            info.valid_count += 1;
            if idx == info.current_idx {
                info.current_is_valid = true;
            }
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    info
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

    #[test]
    fn test_fmt_text_basic() {
        unsafe {
            let mut out = [0i8; 256];
            let text = c"hello world";
            let len = rs_qf_fmt_text(text.as_ptr(), out.as_mut_ptr(), out.len());
            assert_eq!(len, 11);
            let result = std::ffi::CStr::from_ptr(out.as_ptr());
            assert_eq!(result.to_str().unwrap(), "hello world");
        }
    }

    #[test]
    fn test_fmt_text_newlines() {
        unsafe {
            let mut out = [0i8; 256];
            let text = c"line1\nline2\nline3";
            let len = rs_qf_fmt_text(text.as_ptr(), out.as_mut_ptr(), out.len());
            let result = std::ffi::CStr::from_ptr(out.as_ptr());
            assert_eq!(result.to_str().unwrap(), "line1 line2 line3");
            assert_eq!(len, 17);
        }
    }

    #[test]
    fn test_fmt_text_collapse_whitespace() {
        unsafe {
            let mut out = [0i8; 256];
            let text = c"foo\n   bar\n\t\tbaz";
            let len = rs_qf_fmt_text(text.as_ptr(), out.as_mut_ptr(), out.len());
            let result = std::ffi::CStr::from_ptr(out.as_ptr());
            // After newline, whitespace is collapsed
            assert_eq!(result.to_str().unwrap(), "foo bar baz");
            assert_eq!(len, 11);
        }
    }

    #[test]
    fn test_fmt_text_null() {
        unsafe {
            let mut out = [0i8; 256];
            let len = rs_qf_fmt_text(std::ptr::null(), out.as_mut_ptr(), out.len());
            assert_eq!(len, 0);
            assert_eq!(out[0], 0);
        }
    }

    #[test]
    fn test_range_text_line_only() {
        unsafe {
            let mut out = [0i8; 64];
            let len = rs_qf_range_text(10, 0, 0, 0, out.as_mut_ptr(), out.len());
            let result = std::ffi::CStr::from_ptr(out.as_ptr());
            assert_eq!(result.to_str().unwrap(), "10");
            assert_eq!(len, 2);
        }
    }

    #[test]
    fn test_range_text_line_range() {
        unsafe {
            let mut out = [0i8; 64];
            let len = rs_qf_range_text(10, 15, 0, 0, out.as_mut_ptr(), out.len());
            let result = std::ffi::CStr::from_ptr(out.as_ptr());
            assert_eq!(result.to_str().unwrap(), "10-15");
            assert_eq!(len, 5);
        }
    }

    #[test]
    fn test_range_text_with_col() {
        unsafe {
            let mut out = [0i8; 64];
            let len = rs_qf_range_text(10, 0, 5, 0, out.as_mut_ptr(), out.len());
            let result = std::ffi::CStr::from_ptr(out.as_ptr());
            assert_eq!(result.to_str().unwrap(), "10 col 5");
            assert_eq!(len, 8);
        }
    }

    #[test]
    fn test_range_text_full() {
        unsafe {
            let mut out = [0i8; 64];
            let len = rs_qf_range_text(10, 15, 5, 8, out.as_mut_ptr(), out.len());
            let result = std::ffi::CStr::from_ptr(out.as_ptr());
            assert_eq!(result.to_str().unwrap(), "10-15 col 5-8");
            assert_eq!(len, 13);
        }
    }

    // Phase Q5 tests
    #[test]
    fn test_window_size_default() {
        let size = QfWindowSize::default();
        assert_eq!(size.min_height, 3);
        assert_eq!(size.max_height, 10);
        assert!(size.horizontal);
    }

    #[test]
    fn test_null_calc_window_size() {
        unsafe {
            let size = rs_qf_calc_window_size(std::ptr::null(), 3, 10);
            assert_eq!(size.min_height, 3);
            assert_eq!(size.max_height, 10);
            assert_eq!(size.preferred_height, 3);
        }
    }

    #[test]
    fn test_null_should_open_window() {
        unsafe {
            assert!(!rs_qf_should_open_window(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_scroll_to_current() {
        unsafe {
            assert_eq!(rs_qf_scroll_to_current(std::ptr::null(), 10, 3), 1);
        }
    }

    #[test]
    fn test_null_entry_to_buf_line() {
        unsafe {
            assert_eq!(rs_qf_entry_to_buf_line(std::ptr::null(), 1), 0);
        }
    }

    #[test]
    fn test_null_buf_line_to_entry() {
        unsafe {
            assert_eq!(rs_qf_buf_line_to_entry(std::ptr::null(), 1), 0);
        }
    }

    #[test]
    fn test_null_window_buf_valid() {
        unsafe {
            assert!(!rs_qf_window_buf_valid(std::ptr::null(), 10));
        }
    }

    #[test]
    fn test_null_window_buf_delta() {
        unsafe {
            assert_eq!(rs_qf_window_buf_delta(std::ptr::null(), 10), 0);
        }
    }

    #[test]
    fn test_null_window_info() {
        unsafe {
            let info = rs_qf_window_info(std::ptr::null());
            assert!(!info.exists);
            assert_eq!(info.current_idx, 0);
            assert_eq!(info.total_count, 0);
        }
    }

    #[test]
    fn test_window_info_default() {
        let info = QfWindowInfo::default();
        assert!(!info.exists);
        assert_eq!(info.current_idx, 0);
        assert_eq!(info.total_count, 0);
        assert_eq!(info.valid_count, 0);
        assert!(!info.current_is_valid);
    }
}
