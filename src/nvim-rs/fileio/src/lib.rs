//! File I/O utilities for Neovim
//!
//! Provides utility functions for file operations.

#![allow(unsafe_code)]

use std::ffi::c_int;

/// Check if file times differ.
///
/// On Linux/Windows, there's a FAT filesystem tolerance: the seconds portion
/// can differ by up to 1 second due to FAT's 5-bit second storage limitation.
///
/// # Arguments
/// * `file_sec` - File modification time (seconds)
/// * `file_nsec` - File modification time (nanoseconds)
/// * `mtime` - Expected modification time (seconds)
/// * `mtime_ns` - Expected modification time (nanoseconds)
/// * `fat_tolerance` - Whether to apply FAT filesystem tolerance (Linux/Windows)
///
/// # Returns
/// `true` if the times differ, `false` if they match
#[inline]
pub fn time_differs(
    file_sec: i64,
    file_nsec: i64,
    mtime: i64,
    mtime_ns: i64,
    fat_tolerance: bool,
) -> bool {
    if file_nsec != mtime_ns {
        return true;
    }

    if fat_tolerance {
        // On FAT filesystem, there are only 5 bits to store the seconds.
        // The time may change unexpectedly by one second during inode flush.
        let diff = file_sec - mtime;
        !(-1..=1).contains(&diff)
    } else {
        file_sec != mtime
    }
}

/// FFI wrapper for `time_differs`.
///
/// # Safety
/// All parameters are plain integers, so this is safe.
#[no_mangle]
pub extern "C" fn rs_time_differs(
    file_sec: i64,
    file_nsec: i64,
    mtime: i64,
    mtime_ns: i64,
    fat_tolerance: c_int,
) -> c_int {
    c_int::from(time_differs(
        file_sec,
        file_nsec,
        mtime,
        mtime_ns,
        fat_tolerance != 0,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_differs_exact_match() {
        // Exact match, no tolerance
        assert!(!time_differs(1000, 500, 1000, 500, false));
        assert!(!time_differs(1000, 500, 1000, 500, true));
    }

    #[test]
    fn test_time_differs_nanosec_mismatch() {
        // Nanoseconds differ - always different
        assert!(time_differs(1000, 500, 1000, 501, false));
        assert!(time_differs(1000, 500, 1000, 501, true));
    }

    #[test]
    fn test_time_differs_sec_mismatch_no_tolerance() {
        // Seconds differ by 1, no FAT tolerance
        assert!(time_differs(1001, 500, 1000, 500, false));
        assert!(time_differs(999, 500, 1000, 500, false));
    }

    #[test]
    fn test_time_differs_fat_tolerance() {
        // Seconds differ by exactly 1 - FAT tolerance allows this
        assert!(!time_differs(1001, 500, 1000, 500, true));
        assert!(!time_differs(999, 500, 1000, 500, true));

        // Seconds differ by more than 1 - FAT tolerance rejects this
        assert!(time_differs(1002, 500, 1000, 500, true));
        assert!(time_differs(998, 500, 1000, 500, true));
    }

    #[test]
    fn test_ffi_time_differs() {
        // Exact match
        assert_eq!(rs_time_differs(1000, 500, 1000, 500, 0), 0);
        assert_eq!(rs_time_differs(1000, 500, 1000, 500, 1), 0);

        // Nanosec differ
        assert_eq!(rs_time_differs(1000, 500, 1000, 501, 0), 1);

        // Sec differ by 1, FAT tolerance
        assert_eq!(rs_time_differs(1001, 500, 1000, 500, 1), 0);

        // Sec differ by 2, FAT tolerance
        assert_eq!(rs_time_differs(1002, 500, 1000, 500, 1), 1);
    }
}
