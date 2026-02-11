//! `:write` command implementation.
//!
//! The `:write` command writes buffer content to a file.
//!
//! ## Usage
//! - `:w[rite]` - Write current buffer
//! - `:w[rite] {file}` - Write to specified file
//! - `:w[rite]!` - Force write (overwrite readonly)
//! - `:{range}w[rite] [file]` - Write specified lines
//! - `:w[rite] >> {file}` - Append to file
//! - `:w[rite] !{cmd}` - Write to shell command (filter)
//! - `:up[date]` - Write only if modified
//! - `:sav[eas] {file}` - Write to file and change buffer name
//!
//! ## Implementation Notes
//!
//! The actual file writing is performed by Neovim's `buf_write()` function.
//! This module provides:
//! - Type definitions for write operations
//! - Validation utilities
//! - Helper functions for the C implementation

use std::ffi::c_int;

use crate::range::{LineNr, LineRange};

/// Result of a write operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WriteResult {
    /// Write succeeded
    Ok = 0,
    /// File is readonly
    Readonly = 1,
    /// Directory does not exist
    NoDirectory = 2,
    /// Permission denied
    PermissionDenied = 3,
    /// File already exists (and no ! given)
    FileExists = 4,
    /// Write was interrupted
    Interrupted = 5,
    /// Disk full or quota exceeded
    NoSpace = 6,
    /// Buffer not modified (for :update)
    NotModified = 7,
    /// Other error
    Error = 99,
}

impl WriteResult {
    /// Check if the write was successful.
    #[inline]
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, WriteResult::Ok)
    }

    /// Check if this result indicates the write was skipped (not an error).
    #[inline]
    #[must_use]
    pub const fn is_skipped(self) -> bool {
        matches!(self, WriteResult::NotModified)
    }

    /// Convert from C integer return value (0 = success, non-zero = error).
    #[inline]
    #[must_use]
    pub fn from_c_ok_fail(value: c_int) -> Self {
        if value == 0 {
            WriteResult::Ok
        } else {
            WriteResult::Error
        }
    }

    /// Convert to C integer for return.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_int {
        self as c_int
    }
}

/// Options for the `:write` command.
#[derive(Debug, Clone, Default)]
pub struct WriteOptions {
    /// Range of lines to write.
    pub range: LineRange,
    /// Force write (ignore readonly, overwrite existing).
    pub force: bool,
    /// Append to file (>>).
    pub append: bool,
    /// Write to filter command.
    pub filter: bool,
    /// Force binary mode.
    pub force_binary: bool,
    /// Force text mode.
    pub force_text: bool,
    /// Specific encoding to use.
    pub encoding: Option<String>,
    /// Create parent directories if needed (++p).
    pub mkdir_p: bool,
}

impl WriteOptions {
    /// Create options for writing the whole buffer.
    #[must_use]
    pub fn whole_buffer(line_count: LineNr) -> Self {
        Self {
            range: LineRange::whole_buffer(line_count),
            ..Default::default()
        }
    }

    /// Create options for writing a specific range.
    #[must_use]
    pub fn with_range(range: LineRange) -> Self {
        Self {
            range,
            ..Default::default()
        }
    }

    /// Create options for appending to a file.
    #[must_use]
    pub fn append_to(range: LineRange) -> Self {
        Self {
            range,
            append: true,
            ..Default::default()
        }
    }

    /// Create options for force-writing.
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Set the range.
    #[must_use]
    pub fn range(mut self, range: LineRange) -> Self {
        self.range = range;
        self
    }
}

/// Mode for the write operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WriteMode {
    /// Normal write
    #[default]
    Normal,
    /// Only write if buffer is modified (:update)
    Update,
    /// Write and change buffer name (:saveas)
    SaveAs,
}

/// Error type for write operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteError {
    /// Invalid line range.
    InvalidRange,
    /// File path is empty.
    EmptyPath,
    /// Buffer is readonly.
    Readonly,
    /// File already exists.
    FileExists(String),
    /// Permission denied.
    PermissionDenied(String),
    /// Parent directory does not exist.
    NoDirectory(String),
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::InvalidRange => write!(f, "invalid range"),
            WriteError::EmptyPath => write!(f, "empty file path"),
            WriteError::Readonly => write!(f, "buffer is readonly"),
            WriteError::FileExists(path) => write!(f, "file already exists: {path}"),
            WriteError::PermissionDenied(path) => write!(f, "permission denied: {path}"),
            WriteError::NoDirectory(path) => write!(f, "directory does not exist: {path}"),
        }
    }
}

impl std::error::Error for WriteError {}

/// Validate a write range against buffer bounds.
///
/// # Arguments
/// * `range` - The line range to validate
/// * `line_count` - Total lines in the buffer
///
/// # Returns
/// A clamped valid range, or an error if the range is completely invalid.
pub fn validate_write_range(range: LineRange, line_count: LineNr) -> Result<LineRange, WriteError> {
    if line_count == 0 {
        // Empty buffer - any write is technically valid (writes nothing)
        return Ok(LineRange::empty());
    }

    let clamped = range.clamp(line_count);
    if clamped.is_empty() && !range.is_empty() {
        // The range was non-empty but clamped to empty - that's invalid
        return Err(WriteError::InvalidRange);
    }

    Ok(clamped)
}

/// Check if a write should proceed for :update command.
///
/// # Arguments
/// * `is_modified` - Whether the buffer has been modified
/// * `mode` - The write mode
///
/// # Returns
/// `true` if the write should proceed, `false` if it should be skipped.
#[inline]
#[must_use]
pub fn should_write(is_modified: bool, mode: WriteMode) -> bool {
    match mode {
        WriteMode::Update => is_modified,
        WriteMode::Normal | WriteMode::SaveAs => true,
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Validate write range and return validity.
///
/// Returns 1 if valid, 0 if invalid.
pub extern "C" fn rs_validate_write_range(start: c_int, end: c_int, line_count: c_int) -> c_int {
    let range = LineRange::new(start, end);
    c_int::from(validate_write_range(range, line_count).is_ok())
}

/// Check if write should proceed for update command.
///
/// Returns 1 if should write, 0 if should skip.
pub extern "C" fn rs_should_write_update(is_modified: c_int) -> c_int {
    c_int::from(should_write(is_modified != 0, WriteMode::Update))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_result() {
        assert!(WriteResult::Ok.is_ok());
        assert!(!WriteResult::Readonly.is_ok());
        assert!(!WriteResult::Error.is_ok());

        assert!(WriteResult::NotModified.is_skipped());
        assert!(!WriteResult::Ok.is_skipped());
    }

    #[test]
    fn test_write_result_from_c() {
        assert_eq!(WriteResult::from_c_ok_fail(0), WriteResult::Ok);
        assert_eq!(WriteResult::from_c_ok_fail(1), WriteResult::Error);
    }

    #[test]
    fn test_validate_write_range() {
        // Normal range
        let range = LineRange::new(5, 10);
        let result = validate_write_range(range, 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), range);

        // Range extending beyond buffer - gets clamped
        let range = LineRange::new(5, 150);
        let result = validate_write_range(range, 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LineRange::new(5, 100));

        // Empty buffer
        let range = LineRange::new(1, 10);
        let result = validate_write_range(range, 0);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_should_write() {
        // Normal mode always writes
        assert!(should_write(true, WriteMode::Normal));
        assert!(should_write(false, WriteMode::Normal));

        // Update mode only writes if modified
        assert!(should_write(true, WriteMode::Update));
        assert!(!should_write(false, WriteMode::Update));

        // SaveAs always writes
        assert!(should_write(true, WriteMode::SaveAs));
        assert!(should_write(false, WriteMode::SaveAs));
    }

    #[test]
    fn test_write_options_whole_buffer() {
        let opts = WriteOptions::whole_buffer(100);
        assert_eq!(opts.range, LineRange::whole_buffer(100));
        assert!(!opts.force);
        assert!(!opts.append);
    }

    #[test]
    fn test_write_options_with_range() {
        let range = LineRange::new(5, 20);
        let opts = WriteOptions::with_range(range);
        assert_eq!(opts.range, range);
    }

    #[test]
    fn test_write_options_append() {
        let range = LineRange::new(5, 20);
        let opts = WriteOptions::append_to(range);
        assert!(opts.append);
        assert_eq!(opts.range, range);
    }

    #[test]
    fn test_write_options_builder() {
        let opts = WriteOptions::with_range(LineRange::new(1, 10)).force();
        assert!(opts.force);
        assert_eq!(opts.range.start, 1);
        assert_eq!(opts.range.end, 10);
    }

    #[test]
    fn test_write_error_display() {
        let err = WriteError::InvalidRange;
        assert_eq!(format!("{err}"), "invalid range");

        let err = WriteError::EmptyPath;
        assert_eq!(format!("{err}"), "empty file path");

        let err = WriteError::Readonly;
        assert_eq!(format!("{err}"), "buffer is readonly");

        let err = WriteError::FileExists("test.txt".to_string());
        assert_eq!(format!("{err}"), "file already exists: test.txt");
    }

    #[test]
    fn test_rs_validate_write_range() {
        assert_eq!(rs_validate_write_range(1, 10, 100), 1);
        assert_eq!(rs_validate_write_range(5, 150, 100), 1); // Gets clamped
    }

    #[test]
    fn test_rs_should_write_update() {
        assert_eq!(rs_should_write_update(1), 1);
        assert_eq!(rs_should_write_update(0), 0);
    }
}
