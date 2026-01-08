//! Error format parsing and entry validation for quickfix lists.
//!
//! This module handles parsing error format lines and validating quickfix entries.
//! It implements the core parsing logic from `qf_parse_line()` and related functions.

use std::ffi::{c_char, c_int, c_void};

/// Line number type (matches `linenr_T` in Neovim)
type LinenrT = i32;

/// Opaque handle to `qfline_T` (quickfix entry)
type QfLineHandle = *const c_void;

// QF parsing status codes
#[allow(dead_code)]
const QF_OK: c_int = 0;

// =============================================================================
// External C accessor functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Entry accessors
    fn nvim_qfline_get_lnum(qfp: QfLineHandle) -> LinenrT;
    fn nvim_qfline_get_col(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_valid(qfp: QfLineHandle) -> bool;
    fn nvim_qfline_get_type(qfp: QfLineHandle) -> c_char;
    fn nvim_qfline_get_fnum(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_end_lnum(qfp: QfLineHandle) -> LinenrT;
    fn nvim_qfline_get_end_col(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_text(qfp: QfLineHandle) -> *const c_char;
    fn nvim_qfline_get_nr(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_module(qfp: QfLineHandle) -> *const c_char;
}

// =============================================================================
// Entry Validation Types
// =============================================================================

/// Result of validating a quickfix entry
#[repr(C)]
#[derive(Default)]
pub struct QfEntryValidation {
    /// Is the entry valid (has file/line/col)?
    pub valid: bool,
    /// Does the entry have a filename?
    pub has_filename: bool,
    /// Does the entry have a valid line number?
    pub has_lnum: bool,
    /// Does the entry have a valid column?
    pub has_col: bool,
    /// Does the entry have text?
    pub has_text: bool,
    /// Is the entry an error type?
    pub is_error: bool,
    /// Is the entry a warning type?
    pub is_warning: bool,
}

// =============================================================================
// Parsing Result Types
// =============================================================================

/// Result of parsing an error format line
#[repr(C)]
pub struct QfParseResult {
    /// Status code (`QF_OK`, `QF_FAIL`, etc.)
    pub status: c_int,
    /// Index of the matched format prefix
    pub prefix_idx: c_int,
    /// Whether this is a continuation line
    pub is_continuation: bool,
    /// Whether to ignore this line
    pub ignore: bool,
}

impl Default for QfParseResult {
    fn default() -> Self {
        Self {
            status: QF_OK,
            prefix_idx: 0,
            is_continuation: false,
            ignore: false,
        }
    }
}

// =============================================================================
// Entry Validation Functions
// =============================================================================

/// Validate a quickfix entry and return detailed validation info.
///
/// This function checks various properties of a quickfix entry to determine
/// its validity and characteristics.
///
/// # Safety
///
/// - `qfp` may be null (returns default validation with all false)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_validate_entry(qfp: QfLineHandle) -> QfEntryValidation {
    if qfp.is_null() {
        return QfEntryValidation::default();
    }

    let lnum = nvim_qfline_get_lnum(qfp);
    let col = nvim_qfline_get_col(qfp);
    let fnum = nvim_qfline_get_fnum(qfp);
    let type_char = nvim_qfline_get_type(qfp);
    let text = nvim_qfline_get_text(qfp);
    let valid = nvim_qfline_get_valid(qfp);

    let has_lnum = lnum > 0;
    let has_col = col > 0;
    let has_filename = fnum > 0;
    let has_text = !text.is_null() && unsafe { *text != 0 };

    // Check error/warning types - convert to u8 for safe comparison
    #[allow(clippy::cast_sign_loss)]
    let type_byte = type_char as u8;
    let type_upper = if type_byte.is_ascii_lowercase() {
        type_byte.to_ascii_uppercase()
    } else {
        type_byte
    };

    let is_error = type_upper == b'E';
    let is_warning = type_upper == b'W';

    QfEntryValidation {
        valid,
        has_filename,
        has_lnum,
        has_col,
        has_text,
        is_error,
        is_warning,
    }
}

/// Check if an entry has all required fields for a complete error.
///
/// A complete error has: filename, line number, and either text or valid flag.
///
/// # Safety
///
/// - `qfp` may be null (returns false)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_is_complete(qfp: QfLineHandle) -> bool {
    if qfp.is_null() {
        return false;
    }

    let validation = rs_qf_validate_entry(qfp);
    validation.has_filename && validation.has_lnum && (validation.has_text || validation.valid)
}

/// Check if an entry represents a diagnostic (error, warning, info, note).
///
/// # Safety
///
/// - `qfp` may be null (returns false)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_is_diagnostic(qfp: QfLineHandle) -> bool {
    if qfp.is_null() {
        return false;
    }

    let type_char = nvim_qfline_get_type(qfp);
    #[allow(clippy::cast_sign_loss)]
    let type_byte = type_char as u8;
    let type_upper = if type_byte.is_ascii_lowercase() {
        type_byte.to_ascii_uppercase()
    } else {
        type_byte
    };

    matches!(type_upper, b'E' | b'W' | b'I' | b'N')
}

/// Get the severity level of an entry (0=none, 1=note, 2=info, 3=warning, 4=error).
///
/// Higher numbers indicate more severe issues.
///
/// # Safety
///
/// - `qfp` may be null (returns 0)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_severity(qfp: QfLineHandle) -> c_int {
    if qfp.is_null() {
        return 0;
    }

    let type_char = nvim_qfline_get_type(qfp);
    #[allow(clippy::cast_sign_loss)]
    let type_byte = type_char as u8;
    let type_upper = if type_byte.is_ascii_lowercase() {
        type_byte.to_ascii_uppercase()
    } else {
        type_byte
    };

    match type_upper {
        b'E' => 4, // Error
        b'W' => 3, // Warning
        b'I' => 2, // Info
        b'N' => 1, // Note
        _ => 0,    // Unknown/none
    }
}

// =============================================================================
// Error Format Pattern Matching (supplements existing lib.rs functions)
// =============================================================================

/// Check if an errorformat prefix is a file prefix.
///
/// File prefixes are 'O', 'P', 'Q'.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_sign_loss)]
pub extern "C" fn rs_efm_is_file_prefix(prefix: c_char) -> bool {
    matches!(prefix as u8, b'O' | b'P' | b'Q')
}

/// Check if an errorformat prefix is a directory prefix.
///
/// Directory prefixes are 'D' and 'X'.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_sign_loss)]
pub extern "C" fn rs_efm_is_dir_prefix(prefix: c_char) -> bool {
    matches!(prefix as u8, b'D' | b'X')
}

// =============================================================================
// Line Classification
// =============================================================================

/// Line classification result
#[repr(C)]
#[derive(Default)]
pub struct QfLineClass {
    /// Looks like a compiler error/warning line
    pub looks_like_error: bool,
    /// Has filename:line:col pattern
    pub has_location: bool,
    /// Appears to be a continuation line
    pub is_continuation: bool,
    /// Is blank or whitespace only
    pub is_blank: bool,
    /// Contains common error keywords
    pub has_error_keyword: bool,
    /// Contains common warning keywords
    pub has_warning_keyword: bool,
}

/// Classify a line to determine its likely purpose.
///
/// This is used for heuristic line classification when no format matches.
///
/// # Safety
///
/// - `linebuf` may be null (returns default classification)
/// - If non-null, must point to a valid buffer of at least `linelen` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_qf_classify_line(
    linebuf: *const c_char,
    linelen: usize,
) -> QfLineClass {
    if linebuf.is_null() || linelen == 0 {
        return QfLineClass::default();
    }

    let bytes = std::slice::from_raw_parts(linebuf.cast::<u8>(), linelen);

    // Check for blank line
    let is_blank = bytes
        .iter()
        .all(|&b| b == b' ' || b == b'\t' || b == b'\n' || b == b'\r');
    if is_blank {
        return QfLineClass {
            is_blank: true,
            ..Default::default()
        };
    }

    // Check for continuation (starts with whitespace)
    let is_continuation = bytes.first().is_some_and(|&b| b == b' ' || b == b'\t');

    // Try to convert to string for pattern matching
    let Ok(s) = std::str::from_utf8(bytes) else {
        return QfLineClass {
            is_continuation,
            ..Default::default()
        };
    };

    let line_lower = s.to_lowercase();

    // Check for error/warning keywords
    let has_error_keyword = line_lower.contains("error")
        || line_lower.contains("fatal")
        || line_lower.contains("failed");

    let has_warning_keyword = line_lower.contains("warning")
        || line_lower.contains("caution")
        || line_lower.contains("note:");

    // Check for location pattern (file:line or file:line:col)
    let has_location = s.contains(':')
        && s.split(':')
            .nth(1)
            .is_some_and(|part| part.chars().take(10).all(|c| c.is_ascii_digit()));

    let looks_like_error = has_error_keyword || has_warning_keyword || has_location;

    QfLineClass {
        looks_like_error,
        has_location,
        is_continuation,
        is_blank,
        has_error_keyword,
        has_warning_keyword,
    }
}

// =============================================================================
// Entry Span Validation
// =============================================================================

/// Check if an entry's line range is valid (end >= start or no end).
///
/// # Safety
///
/// - `qfp` may be null (returns true - null entries are vacuously valid)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_has_valid_range(qfp: QfLineHandle) -> bool {
    if qfp.is_null() {
        return true;
    }

    let lnum = nvim_qfline_get_lnum(qfp);
    let end_lnum = nvim_qfline_get_end_lnum(qfp);

    // No end line means valid (single line entry)
    if end_lnum == 0 {
        return true;
    }

    // End must be >= start
    end_lnum >= lnum
}

/// Check if an entry's column range is valid (end >= start or no end).
///
/// # Safety
///
/// - `qfp` may be null (returns true - null entries are vacuously valid)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_has_valid_col_range(qfp: QfLineHandle) -> bool {
    if qfp.is_null() {
        return true;
    }

    let col = nvim_qfline_get_col(qfp);
    let end_col = nvim_qfline_get_end_col(qfp);
    let lnum = nvim_qfline_get_lnum(qfp);
    let end_lnum = nvim_qfline_get_end_lnum(qfp);

    // No end column means valid
    if end_col == 0 {
        return true;
    }

    // If multi-line, column comparison doesn't apply
    if end_lnum > 0 && end_lnum != lnum {
        return true;
    }

    // End must be >= start on the same line
    end_col >= col
}

/// Get the length (in lines) of an entry's span.
///
/// Returns 1 for single-line entries.
///
/// # Safety
///
/// - `qfp` may be null (returns 0)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_line_span(qfp: QfLineHandle) -> c_int {
    if qfp.is_null() {
        return 0;
    }

    let lnum = nvim_qfline_get_lnum(qfp);
    let end_lnum = nvim_qfline_get_end_lnum(qfp);

    if lnum <= 0 {
        return 0;
    }

    if end_lnum <= 0 || end_lnum < lnum {
        return 1;
    }

    end_lnum - lnum + 1
}

// =============================================================================
// Parse Type Detection
// =============================================================================

/// Detect the type of entry from text content heuristics.
///
/// This is used when the errorformat doesn't specify a type.
///
/// # Safety
///
/// - `text` may be null (returns 0)
/// - If non-null, must point to a valid C string
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_qf_detect_type_from_text(text: *const c_char) -> c_char {
    if text.is_null() {
        return 0;
    }

    let Ok(c_str) = std::ffi::CStr::from_ptr(text).to_str() else {
        return 0;
    };

    let lower = c_str.to_lowercase();

    // Check for error patterns first (highest priority)
    if lower.contains("error:") || lower.contains("error[") || lower.contains(": error") {
        return b'E' as c_char;
    }

    // Then warning patterns
    if lower.contains("warning:") || lower.contains("warning[") || lower.contains(": warning") {
        return b'W' as c_char;
    }

    // Info patterns
    if lower.contains("info:") || lower.contains("info[") || lower.contains(": info") {
        return b'I' as c_char;
    }

    // Note patterns
    if lower.contains("note:") || lower.contains("note[") || lower.contains(": note") {
        return b'N' as c_char;
    }

    // No type detected
    0
}

// =============================================================================
// Module Entry Parsing
// =============================================================================

/// Check if an entry has a module name set.
///
/// # Safety
///
/// - `qfp` may be null (returns false)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_has_module(qfp: QfLineHandle) -> bool {
    if qfp.is_null() {
        return false;
    }

    let module = nvim_qfline_get_module(qfp);
    !module.is_null() && *module != 0
}

/// Check if an entry has an error number set.
///
/// # Safety
///
/// - `qfp` may be null (returns false)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_has_nr(qfp: QfLineHandle) -> bool {
    if qfp.is_null() {
        return false;
    }

    nvim_qfline_get_nr(qfp) != 0
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_is_file_prefix() {
        assert!(rs_efm_is_file_prefix(b'O' as c_char));
        assert!(rs_efm_is_file_prefix(b'P' as c_char));
        assert!(rs_efm_is_file_prefix(b'Q' as c_char));
        assert!(!rs_efm_is_file_prefix(b'E' as c_char));
        assert!(!rs_efm_is_file_prefix(b'D' as c_char));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_is_dir_prefix() {
        assert!(rs_efm_is_dir_prefix(b'D' as c_char));
        assert!(rs_efm_is_dir_prefix(b'X' as c_char));
        assert!(!rs_efm_is_dir_prefix(b'E' as c_char));
        assert!(!rs_efm_is_dir_prefix(b'O' as c_char));
    }

    #[test]
    fn test_null_validate_entry() {
        unsafe {
            let validation = rs_qf_validate_entry(std::ptr::null());
            assert!(!validation.valid);
            assert!(!validation.has_filename);
            assert!(!validation.has_lnum);
        }
    }

    #[test]
    fn test_null_entry_is_complete() {
        unsafe {
            assert!(!rs_qf_entry_is_complete(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_entry_is_diagnostic() {
        unsafe {
            assert!(!rs_qf_entry_is_diagnostic(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_entry_severity() {
        unsafe {
            assert_eq!(rs_qf_entry_severity(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_classify_line() {
        unsafe {
            let class = rs_qf_classify_line(std::ptr::null(), 0);
            assert!(!class.looks_like_error);
            // Null input returns default, which has is_blank = false
            assert!(!class.is_blank);
        }
    }

    #[test]
    fn test_classify_blank_line() {
        unsafe {
            let blank = b"   \t  \n";
            let class = rs_qf_classify_line(blank.as_ptr().cast(), blank.len());
            assert!(class.is_blank);
        }
    }

    #[test]
    fn test_classify_error_line() {
        unsafe {
            let error_line = b"error: something went wrong";
            let class = rs_qf_classify_line(error_line.as_ptr().cast(), error_line.len());
            assert!(class.looks_like_error);
            assert!(class.has_error_keyword);
            assert!(!class.has_warning_keyword);
        }
    }

    #[test]
    fn test_classify_warning_line() {
        unsafe {
            let warning_line = b"warning: this might be a problem";
            let class = rs_qf_classify_line(warning_line.as_ptr().cast(), warning_line.len());
            assert!(class.looks_like_error);
            assert!(class.has_warning_keyword);
        }
    }

    #[test]
    fn test_classify_location_line() {
        unsafe {
            let loc_line = b"file.c:123:45: message";
            let class = rs_qf_classify_line(loc_line.as_ptr().cast(), loc_line.len());
            assert!(class.has_location);
            assert!(class.looks_like_error);
        }
    }

    #[test]
    fn test_null_entry_has_valid_range() {
        unsafe {
            assert!(rs_qf_entry_has_valid_range(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_entry_line_span() {
        unsafe {
            assert_eq!(rs_qf_entry_line_span(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_detect_type_from_text() {
        unsafe {
            assert_eq!(rs_qf_detect_type_from_text(std::ptr::null()), 0);
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_detect_type_error() {
        unsafe {
            let text = b"error: something failed\0";
            assert_eq!(
                rs_qf_detect_type_from_text(text.as_ptr().cast()),
                b'E' as c_char
            );
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_detect_type_warning() {
        unsafe {
            let text = b"warning: deprecated function\0";
            assert_eq!(
                rs_qf_detect_type_from_text(text.as_ptr().cast()),
                b'W' as c_char
            );
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_detect_type_info() {
        unsafe {
            let text = b"info: additional context\0";
            assert_eq!(
                rs_qf_detect_type_from_text(text.as_ptr().cast()),
                b'I' as c_char
            );
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_detect_type_note() {
        unsafe {
            let text = b"note: see previous definition\0";
            assert_eq!(
                rs_qf_detect_type_from_text(text.as_ptr().cast()),
                b'N' as c_char
            );
        }
    }

    #[test]
    fn test_null_entry_has_module() {
        unsafe {
            assert!(!rs_qf_entry_has_module(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_entry_has_nr() {
        unsafe {
            assert!(!rs_qf_entry_has_nr(std::ptr::null()));
        }
    }
}
