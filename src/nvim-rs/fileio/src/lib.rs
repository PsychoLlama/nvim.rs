//! File I/O utilities for Neovim
//!
//! Provides utility functions for file operations.

#![allow(unsafe_code)]

use std::ffi::{c_char, c_int, CStr};

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

/// Check if a filename is a /dev/fd/ special file.
///
/// The /dev/fd/ mechanism is provided by some shells on some operating systems,
/// e.g., bash on SunOS. Do not accept "/dev/fd/[012]" since opening these may
/// hang Vim (stdin/stdout/stderr).
///
/// Pattern must match:
/// - Starts with "/dev/fd/"
/// - Followed by one or more digits
/// - Nothing after the digits
/// - Not just "/dev/fd/0", "/dev/fd/1", or "/dev/fd/2" (single digit 0, 1, or 2)
#[inline]
fn is_dev_fd_file_impl(fname: &[u8]) -> bool {
    // Must start with "/dev/fd/"
    if !fname.starts_with(b"/dev/fd/") {
        return false;
    }

    let after_prefix = &fname[8..];

    // Must have at least one digit
    if after_prefix.is_empty() || !after_prefix[0].is_ascii_digit() {
        return false;
    }

    // Find end of digits
    let mut digit_end = 0;
    for (i, &c) in after_prefix.iter().enumerate() {
        if c.is_ascii_digit() {
            digit_end = i + 1;
        } else {
            break;
        }
    }

    // Must be NUL-terminated (no trailing chars) - for C strings, the byte after digits
    // is either NUL (end of slice from CStr) or we check if there's anything after
    if digit_end < after_prefix.len() && after_prefix[digit_end] != 0 {
        return false;
    }

    // Now check: if it's a single digit 0, 1, or 2, reject it
    // Accept if: more than one digit, OR single digit that's not 0/1/2
    if after_prefix.len() == 1 || (digit_end == 1 && after_prefix.len() > 1) {
        // Single digit case - reject 0, 1, 2
        let single = after_prefix[0];
        if single == b'0' || single == b'1' || single == b'2' {
            return false;
        }
    }

    true
}

/// FFI wrapper for `is_dev_fd_file`.
///
/// Check if fname is a /dev/fd/N path (excluding 0, 1, 2).
///
/// # Safety
/// `fname` must be a valid C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_is_dev_fd_file(fname: *const c_char) -> bool {
    if fname.is_null() {
        return false;
    }

    let c_str = unsafe { CStr::from_ptr(fname) };
    is_dev_fd_file_impl(c_str.to_bytes())
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

    #[test]
    fn test_is_dev_fd_file() {
        // Valid /dev/fd/N paths (N >= 3 or multiple digits)
        assert!(is_dev_fd_file_impl(b"/dev/fd/3"));
        assert!(is_dev_fd_file_impl(b"/dev/fd/4"));
        assert!(is_dev_fd_file_impl(b"/dev/fd/5"));
        assert!(is_dev_fd_file_impl(b"/dev/fd/9"));
        assert!(is_dev_fd_file_impl(b"/dev/fd/10"));
        assert!(is_dev_fd_file_impl(b"/dev/fd/123"));
        assert!(is_dev_fd_file_impl(b"/dev/fd/63")); // max on most systems

        // Invalid: /dev/fd/0, /dev/fd/1, /dev/fd/2 (stdin/stdout/stderr)
        assert!(!is_dev_fd_file_impl(b"/dev/fd/0"));
        assert!(!is_dev_fd_file_impl(b"/dev/fd/1"));
        assert!(!is_dev_fd_file_impl(b"/dev/fd/2"));

        // Invalid: wrong prefix or no prefix
        assert!(!is_dev_fd_file_impl(b"/dev/fd"));
        assert!(!is_dev_fd_file_impl(b"/dev/fd/"));
        assert!(!is_dev_fd_file_impl(b"/dev/fd/abc"));
        assert!(!is_dev_fd_file_impl(b"dev/fd/5"));
        assert!(!is_dev_fd_file_impl(b"/dev/null"));
        assert!(!is_dev_fd_file_impl(b"/tmp/file"));
        assert!(!is_dev_fd_file_impl(b""));

        // Invalid: trailing characters after digits
        assert!(!is_dev_fd_file_impl(b"/dev/fd/5abc"));
        assert!(!is_dev_fd_file_impl(b"/dev/fd/10.txt"));
    }
}
