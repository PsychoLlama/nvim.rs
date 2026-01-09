//! `:read` command implementation.
//!
//! The `:read` command reads a file or shell command output into the buffer
//! at the specified line position.
//!
//! ## Usage
//! - `:r[ead] [file]` - Read file content after current line
//! - `:r[ead] !{cmd}` - Read shell command output after current line
//! - `:{range}r[ead] [file]` - Read after specified line
//!
//! ## Implementation Notes
//!
//! The actual file reading is performed by Neovim's `readfile()` function.
//! This module provides:
//! - Type definitions for read operations
//! - Validation utilities
//! - Helper functions for the C implementation

use std::ffi::c_int;

use crate::range::LineNr;

/// Result of a read operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ReadResult {
    /// Read succeeded
    Ok = 0,
    /// File not found or unreadable
    NotFound = 1,
    /// Read was interrupted
    Interrupted = 2,
    /// Memory allocation failed
    OutOfMemory = 3,
    /// Invalid file encoding
    EncodingError = 4,
    /// Other error
    Error = 99,
}

impl ReadResult {
    /// Check if the read was successful.
    #[inline]
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, ReadResult::Ok)
    }

    /// Convert from C integer return value.
    ///
    /// In C, 0 typically means success, non-zero means error.
    #[inline]
    #[must_use]
    pub fn from_c_ok_fail(value: c_int) -> Self {
        if value == 0 {
            ReadResult::Ok
        } else {
            ReadResult::Error
        }
    }

    /// Convert to C integer for return.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_int {
        self as c_int
    }
}

/// Options for the `:read` command.
#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    /// Line number to insert after (0 = before first line).
    pub after_line: LineNr,
    /// Whether reading from a filter command (`:r !cmd`).
    pub from_filter: bool,
    /// Force binary mode.
    pub force_binary: bool,
    /// Specific encoding to use (empty = auto-detect).
    pub encoding: Option<String>,
    /// Bad character handling override.
    pub bad_char: Option<i32>,
}

impl ReadOptions {
    /// Create options for reading after the current cursor line.
    #[must_use]
    pub fn at_cursor(cursor_line: LineNr) -> Self {
        Self {
            after_line: cursor_line,
            ..Default::default()
        }
    }

    /// Create options for reading at start of buffer (before line 1).
    #[must_use]
    pub fn at_start() -> Self {
        Self {
            after_line: 0,
            ..Default::default()
        }
    }

    /// Create options for reading from a filter command.
    #[must_use]
    pub fn from_filter(after_line: LineNr) -> Self {
        Self {
            after_line,
            from_filter: true,
            ..Default::default()
        }
    }
}

/// Validate a read target line number.
///
/// Returns the clamped line number if valid, or an error.
///
/// # Arguments
/// * `line` - The line number to insert after (0 = before first line)
/// * `line_count` - Total lines in the buffer
pub fn validate_read_position(line: LineNr, line_count: LineNr) -> Result<LineNr, ReadError> {
    if line < 0 {
        return Err(ReadError::InvalidLine(line));
    }
    // Line 0 means "before first line", which is valid
    // Lines 1..=line_count are valid
    // Lines > line_count should be clamped to line_count (append at end)
    if line > line_count {
        Ok(line_count)
    } else {
        Ok(line)
    }
}

/// Error type for read operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadError {
    /// Invalid line number.
    InvalidLine(LineNr),
    /// File path is empty.
    EmptyPath,
    /// No filename set for current buffer.
    NoFilename,
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::InvalidLine(line) => write!(f, "invalid line number: {line}"),
            ReadError::EmptyPath => write!(f, "empty file path"),
            ReadError::NoFilename => write!(f, "no file name"),
        }
    }
}

impl std::error::Error for ReadError {}

// =============================================================================
// FFI Exports
// =============================================================================

/// Validate read position and return clamped line number.
///
/// Returns the clamped line number, or -1 on error (negative input).
#[no_mangle]
pub extern "C" fn rs_validate_read_position(line: c_int, line_count: c_int) -> c_int {
    validate_read_position(line, line_count).unwrap_or(-1)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_result() {
        assert!(ReadResult::Ok.is_ok());
        assert!(!ReadResult::NotFound.is_ok());
        assert!(!ReadResult::Error.is_ok());
    }

    #[test]
    fn test_read_result_from_c() {
        assert_eq!(ReadResult::from_c_ok_fail(0), ReadResult::Ok);
        assert_eq!(ReadResult::from_c_ok_fail(1), ReadResult::Error);
        assert_eq!(ReadResult::from_c_ok_fail(-1), ReadResult::Error);
    }

    #[test]
    fn test_validate_read_position() {
        // Normal case: line within buffer
        assert_eq!(validate_read_position(5, 100), Ok(5));

        // Line 0 is valid (insert before first line)
        assert_eq!(validate_read_position(0, 100), Ok(0));

        // Line at end of buffer
        assert_eq!(validate_read_position(100, 100), Ok(100));

        // Line beyond buffer: clamp to end
        assert_eq!(validate_read_position(150, 100), Ok(100));

        // Negative line is error
        assert!(matches!(
            validate_read_position(-1, 100),
            Err(ReadError::InvalidLine(-1))
        ));
    }

    #[test]
    fn test_validate_read_position_empty_buffer() {
        // Empty buffer (0 lines)
        assert_eq!(validate_read_position(0, 0), Ok(0));
        assert_eq!(validate_read_position(1, 0), Ok(0));
    }

    #[test]
    fn test_read_options_default() {
        let opts = ReadOptions::default();
        assert_eq!(opts.after_line, 0);
        assert!(!opts.from_filter);
        assert!(!opts.force_binary);
        assert!(opts.encoding.is_none());
    }

    #[test]
    fn test_read_options_at_cursor() {
        let opts = ReadOptions::at_cursor(42);
        assert_eq!(opts.after_line, 42);
        assert!(!opts.from_filter);
    }

    #[test]
    fn test_read_options_at_start() {
        let opts = ReadOptions::at_start();
        assert_eq!(opts.after_line, 0);
    }

    #[test]
    fn test_read_options_from_filter() {
        let opts = ReadOptions::from_filter(10);
        assert_eq!(opts.after_line, 10);
        assert!(opts.from_filter);
    }

    #[test]
    fn test_read_error_display() {
        let err = ReadError::InvalidLine(-5);
        assert_eq!(format!("{err}"), "invalid line number: -5");

        let err = ReadError::EmptyPath;
        assert_eq!(format!("{err}"), "empty file path");

        let err = ReadError::NoFilename;
        assert_eq!(format!("{err}"), "no file name");
    }
}
