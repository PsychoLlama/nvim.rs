//! Version checking utilities for Neovim
//!
//! Provides functions to check Neovim version compatibility.

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::manual_let_else)]

use std::ffi::{c_char, c_int};

// FFI declarations to access version constants from C
extern "C" {
    /// Get the major version number
    fn nvim_get_version_major() -> c_int;
    /// Get the minor version number
    fn nvim_get_version_minor() -> c_int;
    /// Get the patch version number
    fn nvim_get_version_patch() -> c_int;
    /// Get the minimum supported Vim version (e.g., 801 for 8.01)
    fn nvim_get_min_vim_version() -> c_int;
    /// Get the highest patch number for the minimum Vim version
    fn nvim_get_highest_patch() -> c_int;
    /// Get the number of Vim versions in the `vim_versions` array
    fn nvim_get_vim_versions_count() -> usize;
    /// Get the Vim version at the given index
    fn nvim_get_vim_version_at(idx: usize) -> c_int;
    /// Get the number of patches for a given version index
    fn nvim_get_num_patches_at(idx: usize) -> c_int;
    /// Get a patch number at the given indices
    fn nvim_get_patch_at(version_idx: usize, patch_idx: c_int) -> c_int;
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
    let len = version_str
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(version_str.len());
    let s = &version_str[..len];

    let (req_major, req_minor, req_patch) = match parse_version_string(s) {
        Some(v) => v,
        None => return false,
    };

    let (cur_major, cur_minor, cur_patch) = get_nvim_version();

    // Compare versions
    req_major < cur_major
        || (req_major == cur_major
            && (req_minor < cur_minor || (req_minor == cur_minor && req_patch <= cur_patch)))
}

/// FFI wrapper for `has_nvim_version`.
///
/// # Safety
/// - `version_str` must be a valid null-terminated C string
#[export_name = "has_nvim_version"]
pub unsafe extern "C" fn rs_has_nvim_version(version_str: *const c_char) -> bool {
    if version_str.is_null() {
        return false;
    }

    // Find string length (up to reasonable max)
    let mut len = 0;
    while len < 64 && *version_str.add(len) != 0 {
        len += 1;
    }

    let slice = std::slice::from_raw_parts(version_str.cast::<u8>(), len);
    has_nvim_version(slice)
}

/// Returns the minimum supported Vim version.
///
/// This is `vim_versions[0]`, e.g., 801 for Vim 8.01.
///
/// # Safety
///
/// Calls external C function to access static array.
#[export_name = "min_vim_version"]
pub unsafe extern "C" fn rs_min_vim_version() -> c_int {
    nvim_get_min_vim_version()
}

/// Returns the highest patch number for the minimum Vim version.
///
/// This is `included_patchsets[0][0]`, the first entry in the patchset array.
///
/// # Safety
///
/// Calls external C function to access static array.
#[export_name = "highest_patch"]
pub unsafe extern "C" fn rs_highest_patch() -> c_int {
    nvim_get_highest_patch()
}

/// Checks whether a Vim patch has been included.
///
/// Performs a binary search in the `included_patchsets` array for the given
/// patch number. The patches are sorted in descending order.
///
/// # Arguments
/// * `n` - Patch number to check
/// * `major_minor_version` - (major * 100 + minor) Vim version, or 0 for default
///
/// # Returns
/// `true` if patch `n` has been included
#[inline]
fn has_vim_patch_impl(n: c_int, major_minor_version: c_int) -> bool {
    unsafe {
        let count = nvim_get_vim_versions_count();

        // Handle the version index
        let v_i: usize = if major_minor_version > 0 {
            let first_version = nvim_get_vim_version_at(0);
            if major_minor_version < first_version {
                // Older than our minimum version - all patches included
                return true;
            }

            // Find the version index
            let mut found_idx: Option<usize> = None;
            for i in 0..count {
                if nvim_get_vim_version_at(i) == major_minor_version {
                    found_idx = Some(i);
                    break;
                }
            }
            match found_idx {
                Some(idx) => idx,
                None => return false, // Version not found
            }
        } else {
            0 // Use minimum version
        };

        // Perform binary search (patches are in descending order)
        let num_patches = nvim_get_num_patches_at(v_i);
        if num_patches <= 0 {
            return false;
        }

        let mut l = 0;
        let mut h = num_patches - 1;
        loop {
            let m = i32::midpoint(l, h);
            let patch = nvim_get_patch_at(v_i, m);
            if patch == n {
                return true;
            }
            if l == h {
                break;
            }
            // Patches are in descending order, so if patch < n, search lower (higher indices)
            if patch < n {
                h = m;
            } else {
                l = m + 1;
            }
        }
        false
    }
}

/// FFI wrapper for `has_vim_patch`.
///
/// # Safety
/// Calls external C functions to access static arrays.
#[export_name = "has_vim_patch"]
pub extern "C" fn rs_has_vim_patch(n: c_int, major_minor_version: c_int) -> bool {
    has_vim_patch_impl(n, major_minor_version)
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

    #[test]
    fn test_parse_version_string_large_numbers() {
        // Test with larger version numbers
        assert_eq!(parse_version_string(b"10.20.30"), Some((10, 20, 30)));
        assert_eq!(parse_version_string(b"100.200.300"), Some((100, 200, 300)));
    }

    #[test]
    fn test_parse_version_string_edge_cases() {
        // Version with zeros
        assert_eq!(parse_version_string(b"0.0.0"), Some((0, 0, 0)));
        // Just zero
        assert_eq!(parse_version_string(b"0"), Some((0, 0, 0)));
        // Invalid: multiple dots
        assert_eq!(parse_version_string(b"1..2"), None);
        // Invalid: leading dot
        assert_eq!(parse_version_string(b".1.2"), None);
    }

    #[test]
    fn test_parse_int_edge_cases() {
        // Single digit
        assert_eq!(parse_int(b"5"), Some(5));
        // Leading zeros are allowed
        assert_eq!(parse_int(b"007"), Some(7));
        // All digits
        assert_eq!(parse_int(b"1234567890"), Some(1_234_567_890));
    }

    #[test]
    fn test_parse_int_overflow() {
        // Very large number that would overflow - should return None
        assert!(parse_int(b"99999999999999999999").is_none());
    }

    #[test]
    fn test_parse_version_with_null_terminator() {
        // Version string with null terminator (as from C)
        assert_eq!(parse_version_string(b"1.2.3\0"), Some((1, 2, 3)));
        // But the parsing stops at non-digits anyway
        assert_eq!(parse_version_string(b"1.2.3extra"), Some((1, 2, 3)));
    }

    #[test]
    fn test_parse_version_whitespace_handling() {
        // Leading whitespace is skipped
        assert_eq!(parse_version_string(b" 1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version_string(b"\t1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version_string(b"  1.2.3"), Some((1, 2, 3)));
        // Only whitespace returns None
        assert_eq!(parse_version_string(b"   "), None);
    }

    #[test]
    fn test_parse_int_single_digits() {
        // All single digits
        for i in 0..=9 {
            let s = [b'0' + i];
            assert_eq!(parse_int(&s), Some(i as i32));
        }
    }

    #[test]
    fn test_parse_version_boundary() {
        // i32::MAX is 2147483647
        // Check that large but valid versions work
        assert_eq!(
            parse_version_string(b"2147.483.647"),
            Some((2147, 483, 647))
        );
    }

    // Note: has_nvim_version tests require FFI linking.
    // Integration testing is done via the full Neovim build.
}
