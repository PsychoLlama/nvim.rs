//! External integration for quickfix.
//!
//! This module provides functions for integrating quickfix with external
//! tools like compilers, grep, make, and other programs that produce
//! error/warning output.

use std::ffi::{c_char, c_int};

use crate::{QfLinePtr, QfListHandle};

// =============================================================================
// External C accessor functions
// =============================================================================

extern "C" {
    fn nvim_qf_get_start(qfl: QfListHandle) -> QfLinePtr;
}

// =============================================================================
// Output Source Types
// =============================================================================

/// Source type for quickfix entries.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QfSource {
    /// Unknown or default source.
    #[default]
    Unknown = 0,
    /// Make command output.
    Make = 1,
    /// Grep command output.
    Grep = 2,
    /// Compiler output.
    Compiler = 3,
    /// LSP diagnostics.
    Lsp = 4,
    /// Vimgrep command.
    Vimgrep = 5,
    /// Help grep command.
    Helpgrep = 6,
    /// Buffer search.
    Buffer = 7,
}

/// Check if a source is from an external command.
#[no_mangle]
pub const extern "C" fn rs_qf_is_external_source(source: QfSource) -> bool {
    matches!(source, QfSource::Make | QfSource::Grep | QfSource::Compiler)
}

/// Check if a source is from a search operation.
#[no_mangle]
pub const extern "C" fn rs_qf_is_search_source(source: QfSource) -> bool {
    matches!(
        source,
        QfSource::Grep | QfSource::Vimgrep | QfSource::Helpgrep | QfSource::Buffer
    )
}

// =============================================================================
// Output Line Classification
// =============================================================================

/// Classification of an output line.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputLineKind {
    /// Unknown or unclassified line.
    #[default]
    Unknown = 0,
    /// Error message.
    Error = 1,
    /// Warning message.
    Warning = 2,
    /// Note/information message.
    Note = 3,
    /// Continuation of previous message.
    Continuation = 4,
    /// Context line (e.g., code snippet).
    Context = 5,
    /// Empty or blank line.
    Empty = 6,
}

/// Classify a character as an output line type.
///
/// Maps error format type characters to line kinds.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub const extern "C" fn rs_qf_classify_type_char(type_char: c_char) -> OutputLineKind {
    match type_char as u8 {
        b'E' | b'e' => OutputLineKind::Error,
        b'W' | b'w' => OutputLineKind::Warning,
        b'I' | b'i' | b'N' | b'n' => OutputLineKind::Note,
        b'C' | b'c' => OutputLineKind::Continuation,
        _ => OutputLineKind::Unknown,
    }
}

// =============================================================================
// Output Statistics
// =============================================================================

/// Statistics about quickfix output.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct QfOutputStats {
    /// Total number of entries.
    pub total: c_int,
    /// Number of errors.
    pub errors: c_int,
    /// Number of warnings.
    pub warnings: c_int,
    /// Number of notes/info.
    pub notes: c_int,
    /// Number of unique files.
    pub files: c_int,
    /// First file buffer number (0 if none).
    pub first_file: c_int,
}

/// Collect statistics about quickfix output.
///
/// # Safety
///
/// - `qfl` may be null (returns zeroed stats)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_output_stats(qfl: QfListHandle) -> QfOutputStats {
    let mut stats = QfOutputStats::default();

    if qfl.is_null() {
        return stats;
    }

    let mut last_fnum = -1;
    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        stats.total += 1;

        let type_char = (*qfp).qf_type;
        match rs_qf_classify_type_char(type_char) {
            OutputLineKind::Error => stats.errors += 1,
            OutputLineKind::Warning => stats.warnings += 1,
            OutputLineKind::Note => stats.notes += 1,
            _ => {}
        }

        let fnum = (*qfp).qf_fnum;
        if fnum > 0 && fnum != last_fnum {
            stats.files += 1;
            if stats.first_file == 0 {
                stats.first_file = fnum;
            }
            last_fnum = fnum;
        }

        qfp = (*qfp).qf_next;
    }

    stats
}

// =============================================================================
// Output Format Detection
// =============================================================================

/// Detected output format.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Unknown format.
    #[default]
    Unknown = 0,
    /// GCC-style: `file:line:col: type: message`.
    Gcc = 1,
    /// MSVC-style: `file(line): type code: message`.
    Msvc = 2,
    /// Python traceback style.
    Python = 3,
    /// Grep output: `file:line:text`.
    Grep = 4,
    /// Lint output (various linters).
    Lint = 5,
}

/// Detect output format from a line prefix.
///
/// This is a simple heuristic based on common patterns.
///
/// # Safety
///
/// - `line` may be null (returns Unknown)
/// - If non-null, must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_detect_format(line: *const c_char) -> OutputFormat {
    if line.is_null() {
        return OutputFormat::Unknown;
    }

    let Ok(line_str) = std::ffi::CStr::from_ptr(line).to_str() else {
        return OutputFormat::Unknown;
    };

    // Check for GCC style: "file:line:col: error:"
    if line_str.contains(": error:") || line_str.contains(": warning:") {
        if line_str.contains("): error") || line_str.contains("): warning") {
            return OutputFormat::Msvc;
        }
        return OutputFormat::Gcc;
    }

    // Check for MSVC style: "file(line):"
    if let Some(pos) = line_str.find('(') {
        if line_str[pos..].starts_with('(')
            && line_str[pos + 1..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
        {
            if let Some(close) = line_str[pos..].find(')') {
                if line_str[pos + close + 1..].starts_with(':') {
                    return OutputFormat::Msvc;
                }
            }
        }
    }

    // Check for Python traceback
    if line_str.starts_with("  File \"") || line_str.contains("Traceback (most recent") {
        return OutputFormat::Python;
    }

    // Check for grep style: "file:line:text" (at least one colon after filename)
    let colon_count = line_str
        .chars()
        .take_while(|&c| c != ' ')
        .filter(|&c| c == ':')
        .count();
    if colon_count >= 2 {
        return OutputFormat::Grep;
    }

    OutputFormat::Unknown
}

// =============================================================================
// File Extraction
// =============================================================================

/// Extract unique file buffer numbers from quickfix entries.
///
/// Returns the count of unique files found. Fills `out_fnums` with up to
/// `max_files` unique buffer numbers.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `out_fnums` may be null (just counts files)
/// - If non-null, must point to an array of at least `max_files` `c_int`
#[no_mangle]
pub unsafe extern "C" fn rs_qf_extract_files(
    qfl: QfListHandle,
    out_fnums: *mut c_int,
    max_files: c_int,
) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut last_fnum = -1;
    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        let fnum = (*qfp).qf_fnum;
        if fnum > 0 && fnum != last_fnum {
            if !out_fnums.is_null() && count < max_files {
                #[allow(clippy::cast_possible_wrap)]
                let offset = count as isize;
                *out_fnums.offset(offset) = fnum;
            }
            count += 1;
            last_fnum = fnum;
        }
        qfp = (*qfp).qf_next;
    }

    count
}

// =============================================================================
// Output Severity
// =============================================================================

/// Severity level for output entries.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum QfSeverity {
    /// No severity / informational.
    #[default]
    None = 0,
    /// Hint level.
    Hint = 1,
    /// Information level.
    Info = 2,
    /// Warning level.
    Warning = 3,
    /// Error level.
    Error = 4,
}

/// Get the severity for an entry type character.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub const extern "C" fn rs_qf_type_severity(type_char: c_char) -> QfSeverity {
    match type_char as u8 {
        b'E' | b'e' => QfSeverity::Error,
        b'W' | b'w' => QfSeverity::Warning,
        b'I' | b'i' => QfSeverity::Info,
        b'N' | b'n' => QfSeverity::Hint,
        _ => QfSeverity::None,
    }
}

/// Get the maximum severity in a quickfix list.
///
/// # Safety
///
/// - `qfl` may be null (returns None)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_max_severity(qfl: QfListHandle) -> QfSeverity {
    if qfl.is_null() {
        return QfSeverity::None;
    }

    let mut max = QfSeverity::None;
    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        let severity = rs_qf_type_severity((*qfp).qf_type);
        if severity > max {
            max = severity;
            if max == QfSeverity::Error {
                break; // Can't get higher
            }
        }
        qfp = (*qfp).qf_next;
    }

    max
}

/// Check if a quickfix list has any errors.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_has_errors(qfl: QfListHandle) -> bool {
    rs_qf_max_severity(qfl) >= QfSeverity::Error
}

/// Check if a quickfix list has any warnings or errors.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_has_warnings_or_errors(qfl: QfListHandle) -> bool {
    rs_qf_max_severity(qfl) >= QfSeverity::Warning
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_classification() {
        assert!(rs_qf_is_external_source(QfSource::Make));
        assert!(rs_qf_is_external_source(QfSource::Grep));
        assert!(rs_qf_is_external_source(QfSource::Compiler));
        assert!(!rs_qf_is_external_source(QfSource::Lsp));
        assert!(!rs_qf_is_external_source(QfSource::Unknown));

        assert!(rs_qf_is_search_source(QfSource::Grep));
        assert!(rs_qf_is_search_source(QfSource::Vimgrep));
        assert!(!rs_qf_is_search_source(QfSource::Make));
    }

    #[test]
    fn test_type_char_classification() {
        #[allow(clippy::cast_possible_wrap)]
        {
            assert_eq!(
                rs_qf_classify_type_char(b'E' as c_char),
                OutputLineKind::Error
            );
            assert_eq!(
                rs_qf_classify_type_char(b'e' as c_char),
                OutputLineKind::Error
            );
            assert_eq!(
                rs_qf_classify_type_char(b'W' as c_char),
                OutputLineKind::Warning
            );
            assert_eq!(
                rs_qf_classify_type_char(b'I' as c_char),
                OutputLineKind::Note
            );
            assert_eq!(
                rs_qf_classify_type_char(b'N' as c_char),
                OutputLineKind::Note
            );
            assert_eq!(
                rs_qf_classify_type_char(b'C' as c_char),
                OutputLineKind::Continuation
            );
            assert_eq!(
                rs_qf_classify_type_char(b' ' as c_char),
                OutputLineKind::Unknown
            );
        }
    }

    #[test]
    fn test_severity() {
        assert!(QfSeverity::Error > QfSeverity::Warning);
        assert!(QfSeverity::Warning > QfSeverity::Info);
        assert!(QfSeverity::Info > QfSeverity::Hint);
        assert!(QfSeverity::Hint > QfSeverity::None);

        #[allow(clippy::cast_possible_wrap)]
        {
            assert_eq!(rs_qf_type_severity(b'E' as c_char), QfSeverity::Error);
            assert_eq!(rs_qf_type_severity(b'W' as c_char), QfSeverity::Warning);
            assert_eq!(rs_qf_type_severity(b'I' as c_char), QfSeverity::Info);
            assert_eq!(rs_qf_type_severity(b'N' as c_char), QfSeverity::Hint);
            assert_eq!(rs_qf_type_severity(b' ' as c_char), QfSeverity::None);
        }
    }

    #[test]
    fn test_null_safety() {
        unsafe {
            let stats = rs_qf_output_stats(std::ptr::null());
            assert_eq!(stats.total, 0);
            assert_eq!(stats.errors, 0);

            assert_eq!(rs_qf_detect_format(std::ptr::null()), OutputFormat::Unknown);
            assert_eq!(
                rs_qf_extract_files(std::ptr::null(), std::ptr::null_mut(), 10),
                0
            );
            assert_eq!(rs_qf_max_severity(std::ptr::null()), QfSeverity::None);
            assert!(!rs_qf_has_errors(std::ptr::null()));
            assert!(!rs_qf_has_warnings_or_errors(std::ptr::null()));
        }
    }

    #[test]
    fn test_format_detection() {
        unsafe {
            // GCC style
            let gcc = c"main.c:10:5: error: undeclared identifier";
            assert_eq!(rs_qf_detect_format(gcc.as_ptr()), OutputFormat::Gcc);

            // Grep style
            let grep = c"file.txt:42:some text here";
            assert_eq!(rs_qf_detect_format(grep.as_ptr()), OutputFormat::Grep);

            // Python traceback
            let python = c"  File \"test.py\", line 10";
            assert_eq!(rs_qf_detect_format(python.as_ptr()), OutputFormat::Python);
        }
    }
}
