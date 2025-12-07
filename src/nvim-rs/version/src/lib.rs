//! Version checking utilities for Neovim
//!
//! Provides functions to check Neovim version compatibility.

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_char, c_int};

// FFI declarations to access version constants from C
extern "C" {
    /// Get the major version number
    fn nvim_get_version_major() -> c_int;
    /// Get the minor version number
    fn nvim_get_version_minor() -> c_int;
    /// Get the patch version number
    fn nvim_get_version_patch() -> c_int;
}

/// Get the current Neovim version as (major, minor, patch)
#[inline]
fn get_nvim_version() -> (i32, i32, i32) {
    unsafe {
        (
            nvim_get_version_major() as i32,
            nvim_get_version_minor() as i32,
            nvim_get_version_patch() as i32,
        )
    }
}

/// Parse a version string like "1.2.3" into (major, minor, patch)
///
/// Returns None if the string is invalid.
fn parse_version_string(s: &[u8]) -> Option<(i32, i32, i32)> {
    if s.is_empty() {
        return None;
    }

    let mut pos = 0;

    // Skip leading whitespace (shouldn't happen but be safe)
    while pos < s.len() && s[pos].is_ascii_whitespace() {
        pos += 1;
    }

    if pos >= s.len() || !s[pos].is_ascii_digit() {
        return None;
    }

    // Parse major version
    let major_start = pos;
    while pos < s.len() && s[pos].is_ascii_digit() {
        pos += 1;
    }
    let major = parse_int(&s[major_start..pos])?;

    // Check for minor version
    let minor = if pos < s.len() && s[pos] == b'.' {
        pos += 1; // skip dot
        if pos >= s.len() || !s[pos].is_ascii_digit() {
            return None;
        }
        let minor_start = pos;
        while pos < s.len() && s[pos].is_ascii_digit() {
            pos += 1;
        }
        parse_int(&s[minor_start..pos])?
    } else {
        0
    };

    // Check for patch version
    let patch = if pos < s.len() && s[pos] == b'.' {
        pos += 1; // skip dot
        if pos >= s.len() || !s[pos].is_ascii_digit() {
            return None;
        }
        let patch_start = pos;
        while pos < s.len() && s[pos].is_ascii_digit() {
            pos += 1;
        }
        parse_int(&s[patch_start..pos])?
    } else {
        0
    };

    Some((major, minor, patch))
}

/// Parse a slice of ASCII digits into an integer
fn parse_int(s: &[u8]) -> Option<i32> {
    if s.is_empty() {
        return None;
    }
    let mut result: i32 = 0;
    for &b in s {
        if !b.is_ascii_digit() {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((b - b'0') as i32)?;
    }
    Some(result)
}

/// Check if the current Neovim version is at or above the given version.
///
/// # Arguments
/// * `version_str` - Version string like "1.2.3" or "0.10" or "1"
///
/// # Returns
/// `true` if Neovim is at or above the specified version
#[inline]
pub fn has_nvim_version(version_str: &[u8]) -> bool {
    // Find the null terminator if present
    let len = version_str.iter().position(|&b| b == 0).unwrap_or(version_str.len());
    let s = &version_str[..len];

    let (req_major, req_minor, req_patch) = match parse_version_string(s) {
        Some(v) => v,
        None => return false,
    };

    let (cur_major, cur_minor, cur_patch) = get_nvim_version();

    // Compare versions
    req_major < cur_major
        || (req_major == cur_major
            && (req_minor < cur_minor
                || (req_minor == cur_minor && req_patch <= cur_patch)))
}

/// FFI wrapper for has_nvim_version
///
/// # Safety
/// - `version_str` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_has_nvim_version(version_str: *const c_char) -> c_int {
    if version_str.is_null() {
        return 0;
    }

    // Find string length (up to reasonable max)
    let mut len = 0;
    while len < 64 && *version_str.add(len) != 0 {
        len += 1;
    }

    let slice = std::slice::from_raw_parts(version_str as *const u8, len);
    c_int::from(has_nvim_version(slice))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_string() {
        assert_eq!(parse_version_string(b"1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version_string(b"0.10.0"), Some((0, 10, 0)));
        assert_eq!(parse_version_string(b"1.0"), Some((1, 0, 0)));
        assert_eq!(parse_version_string(b"2"), Some((2, 0, 0)));
        assert_eq!(parse_version_string(b""), None);
        assert_eq!(parse_version_string(b"abc"), None);
        assert_eq!(parse_version_string(b"1."), None);
        assert_eq!(parse_version_string(b"1.2."), None);
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(parse_int(b"0"), Some(0));
        assert_eq!(parse_int(b"123"), Some(123));
        assert_eq!(parse_int(b""), None);
        assert_eq!(parse_int(b"12a"), None);
    }
}
