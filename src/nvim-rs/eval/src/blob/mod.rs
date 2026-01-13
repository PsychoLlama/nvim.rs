//! Blob operations.
//!
//! This module provides helpers for blob operations:
//! blob_alloc, blob_append, blob_get/set

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;
use std::ptr::NonNull;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a blob_T structure.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct BlobHandle(NonNull<std::ffi::c_void>);

// =============================================================================
// Blob Operation Constants
// =============================================================================

/// Blob is empty.
pub const BLOB_EMPTY: c_int = 0;
/// Blob has data.
pub const BLOB_HAS_DATA: c_int = 1;

// =============================================================================
// Blob Flags
// =============================================================================

/// Blob is locked (cannot be modified).
pub const BLOB_LOCKED: c_int = 0x01;

// =============================================================================
// Blob Literal Constants
// =============================================================================

/// Blob literal prefix (0z).
pub const BLOB_PREFIX_0Z: c_int = 0;
/// Blob literal uppercase (0Z).
pub const BLOB_PREFIX_0Z_UPPER: c_int = 1;

// =============================================================================
// Blob Helpers
// =============================================================================

/// Check if index is valid for given length.
fn is_valid_blob_index(idx: i64, len: i64) -> bool {
    if idx >= 0 {
        idx < len
    } else {
        idx.abs() <= len
    }
}

/// Normalize negative index to positive.
fn normalize_blob_index(idx: i64, len: i64) -> i64 {
    if idx < 0 {
        len + idx
    } else {
        idx
    }
}

/// Check if byte value is valid (0-255).
fn is_valid_byte(val: i64) -> bool {
    (0..=255).contains(&val)
}

/// Check if blob is locked.
fn is_blob_locked(flags: c_int) -> bool {
    (flags & BLOB_LOCKED) != 0
}

/// Check if character is valid hex digit.
fn is_blob_hex_digit(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

/// Convert hex character to value.
fn hex_to_value(c: u8) -> c_int {
    match c {
        b'0'..=b'9' => c_int::from(c - b'0'),
        b'a'..=b'f' => c_int::from(c - b'a' + 10),
        b'A'..=b'F' => c_int::from(c - b'A' + 10),
        _ => -1,
    }
}

/// Convert value to hex character.
fn value_to_hex(val: c_int, uppercase: bool) -> u8 {
    let v = val & 0x0F;
    if v < 10 {
        b'0' + v as u8
    } else if uppercase {
        b'A' + (v - 10) as u8
    } else {
        b'a' + (v - 10) as u8
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get BLOB_EMPTY constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_blob_empty() -> c_int {
    BLOB_EMPTY
}

/// FFI: Get BLOB_HAS_DATA constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_blob_has_data() -> c_int {
    BLOB_HAS_DATA
}

/// FFI: Get BLOB_LOCKED constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_blob_locked_flag() -> c_int {
    BLOB_LOCKED
}

/// FFI: Check if valid blob index.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_valid_blob_index(idx: i64, len: i64) -> c_int {
    c_int::from(is_valid_blob_index(idx, len))
}

/// FFI: Normalize blob index.
#[unsafe(no_mangle)]
pub extern "C" fn rs_normalize_blob_index(idx: i64, len: i64) -> i64 {
    normalize_blob_index(idx, len)
}

/// FFI: Check if valid byte value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_valid_byte(val: i64) -> c_int {
    c_int::from(is_valid_byte(val))
}

/// FFI: Check if blob is locked.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_blob_locked(flags: c_int) -> c_int {
    c_int::from(is_blob_locked(flags))
}

/// FFI: Check if character is blob hex digit.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_blob_hex_digit(c: c_int) -> c_int {
    c_int::from(is_blob_hex_digit(c as u8))
}

/// FFI: Convert hex character to value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_hex_to_value(c: c_int) -> c_int {
    hex_to_value(c as u8)
}

/// FFI: Convert value to hex character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_value_to_hex(val: c_int, uppercase: c_int) -> c_int {
    c_int::from(value_to_hex(val, uppercase != 0))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_constants() {
        assert_eq!(BLOB_EMPTY, 0);
        assert_eq!(BLOB_HAS_DATA, 1);
    }

    #[test]
    fn test_blob_flags() {
        assert_eq!(BLOB_LOCKED, 0x01);
    }

    #[test]
    fn test_is_valid_blob_index() {
        assert!(is_valid_blob_index(0, 10));
        assert!(is_valid_blob_index(9, 10));
        assert!(!is_valid_blob_index(10, 10));
        assert!(is_valid_blob_index(-1, 10));
        assert!(is_valid_blob_index(-10, 10));
        assert!(!is_valid_blob_index(-11, 10));
    }

    #[test]
    fn test_normalize_blob_index() {
        assert_eq!(normalize_blob_index(0, 10), 0);
        assert_eq!(normalize_blob_index(5, 10), 5);
        assert_eq!(normalize_blob_index(-1, 10), 9);
        assert_eq!(normalize_blob_index(-10, 10), 0);
    }

    #[test]
    fn test_is_valid_byte() {
        assert!(is_valid_byte(0));
        assert!(is_valid_byte(255));
        assert!(is_valid_byte(128));
        assert!(!is_valid_byte(-1));
        assert!(!is_valid_byte(256));
    }

    #[test]
    fn test_is_blob_locked() {
        assert!(is_blob_locked(BLOB_LOCKED));
        assert!(!is_blob_locked(0));
    }

    #[test]
    fn test_is_blob_hex_digit() {
        assert!(is_blob_hex_digit(b'0'));
        assert!(is_blob_hex_digit(b'9'));
        assert!(is_blob_hex_digit(b'a'));
        assert!(is_blob_hex_digit(b'f'));
        assert!(is_blob_hex_digit(b'A'));
        assert!(is_blob_hex_digit(b'F'));
        assert!(!is_blob_hex_digit(b'g'));
        assert!(!is_blob_hex_digit(b'G'));
    }

    #[test]
    fn test_hex_to_value() {
        assert_eq!(hex_to_value(b'0'), 0);
        assert_eq!(hex_to_value(b'9'), 9);
        assert_eq!(hex_to_value(b'a'), 10);
        assert_eq!(hex_to_value(b'f'), 15);
        assert_eq!(hex_to_value(b'A'), 10);
        assert_eq!(hex_to_value(b'F'), 15);
        assert_eq!(hex_to_value(b'g'), -1);
    }

    #[test]
    fn test_value_to_hex() {
        assert_eq!(value_to_hex(0, false), b'0');
        assert_eq!(value_to_hex(9, false), b'9');
        assert_eq!(value_to_hex(10, false), b'a');
        assert_eq!(value_to_hex(15, false), b'f');
        assert_eq!(value_to_hex(10, true), b'A');
        assert_eq!(value_to_hex(15, true), b'F');
    }
}
