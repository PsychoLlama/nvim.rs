//! Validation utilities for buffer writing
//!
//! This module provides functions for validating write parameters,
//! checking file permissions, and other pre-write checks.

#![allow(clippy::if_same_then_else)]

use std::ffi::c_int;

// =============================================================================
// Write Flags
// =============================================================================

/// Write flag: append to existing file
pub const WRITE_APPEND: u32 = 0x01;
/// Write flag: force overwrite
pub const WRITE_FORCE: u32 = 0x02;
/// Write flag: preserve permissions
pub const WRITE_PERM: u32 = 0x04;
/// Write flag: write whole buffer
pub const WRITE_WHOLEFILE: u32 = 0x08;
/// Write flag: do not backup
pub const WRITE_NOBACKUP: u32 = 0x10;
/// Write flag: exiting Vim
pub const WRITE_EXIT: u32 = 0x20;
/// Write flag: filter through command
pub const WRITE_FILTER: u32 = 0x40;
/// Write flag: :wall command
pub const WRITE_ALL: u32 = 0x80;

// =============================================================================
// Validation Functions
// =============================================================================

/// Check if a line range is valid for writing.
///
/// # Arguments
///
/// * `start` - Start line number (1-based)
/// * `end` - End line number (1-based)
/// * `line_count` - Total number of lines in buffer
///
/// # Returns
///
/// `true` if the range is valid
#[must_use]
pub const fn is_valid_line_range(start: i64, end: i64, line_count: i64) -> bool {
    start >= 1 && start <= end && end <= line_count
}

/// Check if a file size is within acceptable limits.
///
/// This is mainly for 32-bit systems where files > 2GB may cause issues.
#[must_use]
pub const fn is_valid_file_size(size: u64) -> bool {
    // On modern 64-bit systems, practically any size is valid
    // On 32-bit systems, check for 2GB limit (signed 32-bit max)
    #[cfg(target_pointer_width = "64")]
    {
        size < u64::MAX
    }
    #[cfg(not(target_pointer_width = "64"))]
    {
        size < 0x7FFF_FFFF
    }
}

/// Check if append mode is requested.
#[must_use]
pub const fn is_append_mode(flags: u32) -> bool {
    flags & WRITE_APPEND != 0
}

/// Check if force overwrite is requested.
#[must_use]
pub const fn is_force_write(flags: u32) -> bool {
    flags & WRITE_FORCE != 0
}

/// Check if backup should be skipped.
#[must_use]
pub const fn should_skip_backup(flags: u32) -> bool {
    flags & WRITE_NOBACKUP != 0
}

/// Check if this is a whole buffer write.
#[must_use]
pub const fn is_whole_buffer(flags: u32) -> bool {
    flags & WRITE_WHOLEFILE != 0
}

/// Check if writing during exit.
#[must_use]
pub const fn is_exit_write(flags: u32) -> bool {
    flags & WRITE_EXIT != 0
}

/// Check if writing through filter.
#[must_use]
pub const fn is_filter_write(flags: u32) -> bool {
    flags & WRITE_FILTER != 0
}

/// Validate a filename for writing.
///
/// Returns `true` if the filename appears valid (non-empty, no null bytes).
#[must_use]
pub fn is_valid_filename(name: &[u8]) -> bool {
    !name.is_empty() && !name.contains(&0)
}

// =============================================================================
// Buffer Size Calculations
// =============================================================================

/// Calculate buffer size for write operations.
///
/// Returns a reasonable buffer size based on file size hint.
#[must_use]
pub const fn calc_write_buffer_size(file_size_hint: usize) -> usize {
    const MIN_BUFFER: usize = 4096;
    const MAX_BUFFER: usize = 64 * 1024; // 64KB

    if file_size_hint == 0 {
        MIN_BUFFER
    } else if file_size_hint < MIN_BUFFER {
        MIN_BUFFER
    } else if file_size_hint > MAX_BUFFER {
        MAX_BUFFER
    } else {
        // Round up to nearest 4KB
        (file_size_hint + 4095) & !4095
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if line range is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_valid_range(start: i64, end: i64, line_count: i64) -> c_int {
    c_int::from(is_valid_line_range(start, end, line_count))
}

/// Check if append mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_is_append(flags: u32) -> c_int {
    c_int::from(is_append_mode(flags))
}

/// Check if force write.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_is_force(flags: u32) -> c_int {
    c_int::from(is_force_write(flags))
}

/// Check if backup should be skipped.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_skip_backup(flags: u32) -> c_int {
    c_int::from(should_skip_backup(flags))
}

/// Check if whole buffer write.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_is_whole(flags: u32) -> c_int {
    c_int::from(is_whole_buffer(flags))
}

/// Check if exit write.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_is_exit(flags: u32) -> c_int {
    c_int::from(is_exit_write(flags))
}

/// Check if filter write.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_is_filter(flags: u32) -> c_int {
    c_int::from(is_filter_write(flags))
}

/// Calculate write buffer size.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_calc_buffer_size(hint: usize) -> usize {
    calc_write_buffer_size(hint)
}

/// Check if filename is valid.
///
/// # Safety
///
/// `name` must point to valid memory of `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_bufwrite_valid_filename(name: *const u8, len: usize) -> c_int {
    if name.is_null() {
        return 0;
    }
    // SAFETY: name is checked for null and caller guarantees len bytes
    let slice = unsafe { std::slice::from_raw_parts(name, len) };
    c_int::from(is_valid_filename(slice))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_line_range() {
        // Valid ranges
        assert!(is_valid_line_range(1, 1, 10));
        assert!(is_valid_line_range(1, 10, 10));
        assert!(is_valid_line_range(5, 5, 10));

        // Invalid ranges
        assert!(!is_valid_line_range(0, 10, 10)); // start < 1
        assert!(!is_valid_line_range(1, 11, 10)); // end > line_count
        assert!(!is_valid_line_range(5, 4, 10)); // start > end
    }

    #[test]
    fn test_write_flags() {
        let flags = WRITE_APPEND | WRITE_FORCE;
        assert!(is_append_mode(flags));
        assert!(is_force_write(flags));
        assert!(!should_skip_backup(flags));

        let flags = WRITE_NOBACKUP | WRITE_EXIT;
        assert!(should_skip_backup(flags));
        assert!(is_exit_write(flags));
        assert!(!is_append_mode(flags));
    }

    #[test]
    fn test_is_valid_filename() {
        assert!(is_valid_filename(b"test.txt"));
        assert!(is_valid_filename(b"a"));
        assert!(!is_valid_filename(b""));
        assert!(!is_valid_filename(b"test\0.txt"));
    }

    #[test]
    fn test_calc_write_buffer_size() {
        assert_eq!(calc_write_buffer_size(0), 4096);
        assert_eq!(calc_write_buffer_size(100), 4096);
        assert_eq!(calc_write_buffer_size(5000), 8192); // Round up to 8KB
        assert_eq!(calc_write_buffer_size(100_000), 65536); // Cap at 64KB
    }
}
