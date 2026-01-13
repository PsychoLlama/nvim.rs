//! View Loading Infrastructure
//!
//! This module provides FFI helpers for :loadview command and view restoration.
//! Phase 181 of Rust migration.
//!
//! View files store per-buffer state (cursor, folds, local options) and are
//! named using an encoded version of the buffer's full path.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int};

// =============================================================================
// View Directory Constants
// =============================================================================

/// Default view directory suffix (appended to data directory)
static VIEW_DIR_SUFFIX: &[u8] = b"view\0";

/// View file extension
static VIEW_FILE_EXT: &[u8] = b".vim\0";

/// Default view number (when no number specified)
pub const DEFAULT_VIEW_NUM: c_int = 0;

/// Maximum view number (1-9 are valid numbered views)
pub const MAX_VIEW_NUM: c_int = 9;

// =============================================================================
// View File Name Building
// =============================================================================

/// Get the view directory suffix.
#[no_mangle]
pub extern "C" fn rs_view_dir_suffix() -> *const c_char {
    VIEW_DIR_SUFFIX.as_ptr().cast::<c_char>()
}

/// Get the view file extension.
#[no_mangle]
pub extern "C" fn rs_view_file_ext() -> *const c_char {
    VIEW_FILE_EXT.as_ptr().cast::<c_char>()
}

/// Calculate the extra length needed when encoding a path for view file name.
///
/// Each '=' or path separator in the source path needs one extra character
/// (they become two-character sequences like "=+" or "==").
///
/// Returns the number of extra characters needed.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_view_path_extra_len(
    path: *const c_char,
    path_len: c_int,
    is_path_sep: extern "C" fn(c: c_int) -> c_int,
) -> c_int {
    if path.is_null() || path_len <= 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(path.cast::<u8>(), path_len as usize);
    let mut extra = 0;

    for &c in slice {
        if c == b'=' || is_path_sep(c_int::from(c)) != 0 {
            extra += 1;
        }
    }

    extra
}

/// Encode a path into the view file name format.
///
/// - Path separators become "=+"
/// - ':' becomes "=-" (on Windows)
/// - '=' becomes "=="
///
/// # Safety
/// - `src` must point to a valid path string of `src_len` bytes.
/// - `dst` must have space for `src_len + extra_len` bytes, where
///   `extra_len` comes from `rs_view_path_extra_len`.
///
/// Returns the number of bytes written to dst.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_view_encode_path(
    src: *const c_char,
    src_len: c_int,
    dst: *mut c_char,
    is_path_sep: extern "C" fn(c: c_int) -> c_int,
    is_colon_sep: c_int,
) -> c_int {
    if src.is_null() || dst.is_null() || src_len <= 0 {
        return 0;
    }

    let src_slice = std::slice::from_raw_parts(src.cast::<u8>(), src_len as usize);
    let dst_ptr = dst.cast::<u8>();
    let mut written = 0;

    for &c in src_slice {
        if c == b'=' {
            *dst_ptr.add(written) = b'=';
            *dst_ptr.add(written + 1) = b'=';
            written += 2;
        } else if is_path_sep(c_int::from(c)) != 0 {
            *dst_ptr.add(written) = b'=';
            // Use '-' for colon (Windows drive separator), '+' otherwise
            if is_colon_sep != 0 && c == b':' {
                *dst_ptr.add(written + 1) = b'-';
            } else {
                *dst_ptr.add(written + 1) = b'+';
            }
            written += 2;
        } else {
            *dst_ptr.add(written) = c;
            written += 1;
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        written as c_int
    }
}

/// Calculate the total length needed for a view file path.
///
/// viewdir + '/' + encoded_path + '=' + view_char + ".vim" + NUL
#[no_mangle]
pub extern "C" fn rs_view_file_total_len(
    viewdir_len: c_int,
    encoded_path_len: c_int,
    has_view_num: c_int,
) -> c_int {
    // viewdir + '/' + encoded_path + '=' + maybe_view_char + ".vim" + NUL
    // = viewdir_len + 1 + encoded_path_len + 1 + (0 or 1) + 4 + 1
    let base = viewdir_len + 1 + encoded_path_len + 1 + 4 + 1;
    if has_view_num != 0 {
        base + 1
    } else {
        base
    }
}

/// Get the view suffix character for a view number.
///
/// Returns the character ('1'-'9') for numbered views, or '\0' for default view.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub extern "C" fn rs_view_num_char(view_num: c_int) -> c_char {
    if (1..=9).contains(&view_num) {
        (b'0' + view_num as u8) as c_char
    } else {
        0
    }
}

/// Check if a view number is valid.
#[no_mangle]
pub extern "C" fn rs_view_num_valid(view_num: c_int) -> c_int {
    c_int::from((0..=9).contains(&view_num))
}

/// Parse a view number from an argument character.
///
/// Returns the view number (0-9) or -1 if invalid.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_view_parse_num(arg: c_char) -> c_int {
    let c = arg as u8;
    if c.is_ascii_digit() {
        c_int::from(c - b'0')
    } else {
        -1
    }
}

// =============================================================================
// View Directory Management
// =============================================================================

/// Check if a path looks like it's in the view directory.
///
/// View files typically have "=+" or "==" patterns in their names.
///
/// # Safety
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_view_is_in_viewdir(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let path_cstr = std::ffi::CStr::from_ptr(path);
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    // View files have encoded path separators
    c_int::from(path_str.contains("=+") || path_str.contains("==") || path_str.contains("=-"))
}

/// Check if a view file exists based on its pattern.
///
/// Returns 1 if the path ends with "=.vim" or "=N.vim" where N is 1-9.
///
/// # Safety
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_view_file_pattern_valid(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let path_cstr = std::ffi::CStr::from_ptr(path);
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    // Must end with .vim
    if !path_str.ends_with(".vim") {
        return 0;
    }

    // Check for "=.vim" (default view) or "=N.vim" (numbered view)
    let before_ext = &path_str[..path_str.len() - 4];
    if before_ext.ends_with('=') {
        return 1;
    }

    // Check for "=N" where N is 1-9
    if before_ext.len() >= 2 {
        let last_two = &before_ext[before_ext.len() - 2..];
        if last_two.starts_with('=') && last_two.chars().last().is_some_and(|c| c.is_ascii_digit())
        {
            return 1;
        }
    }

    0
}

// =============================================================================
// View Loading State
// =============================================================================

/// View loading error codes
pub const VIEW_OK: c_int = 0;
pub const VIEW_ERR_NO_NAME: c_int = 1;
pub const VIEW_ERR_NO_FILE: c_int = 2;
pub const VIEW_ERR_SOURCE: c_int = 3;
pub const VIEW_ERR_INVALID: c_int = 4;

/// Get the error message for a view loading error.
#[no_mangle]
pub extern "C" fn rs_view_error_msg(error: c_int) -> *const c_char {
    static NO_NAME: &[u8] = b"No file name\0";
    static NO_FILE: &[u8] = b"View file not found\0";
    static SOURCE: &[u8] = b"Error sourcing view file\0";
    static INVALID: &[u8] = b"Invalid view number\0";
    static OK: &[u8] = b"\0";

    let msg = match error {
        VIEW_ERR_NO_NAME => NO_NAME,
        VIEW_ERR_NO_FILE => NO_FILE,
        VIEW_ERR_SOURCE => SOURCE,
        VIEW_ERR_INVALID => INVALID,
        _ => OK,
    };
    msg.as_ptr().cast::<c_char>()
}

/// Check which error applies based on conditions.
#[no_mangle]
pub extern "C" fn rs_view_check_error(
    has_fname: c_int,
    view_num: c_int,
    file_exists: c_int,
) -> c_int {
    if has_fname == 0 {
        return VIEW_ERR_NO_NAME;
    }
    if !(-1..=9).contains(&view_num) {
        return VIEW_ERR_INVALID;
    }
    if file_exists == 0 {
        return VIEW_ERR_NO_FILE;
    }
    VIEW_OK
}

// =============================================================================
// Restoration Helpers
// =============================================================================

/// Check if view should restore cursor position.
#[no_mangle]
pub extern "C" fn rs_view_should_restore_cursor(flags: u32) -> c_int {
    use crate::SessionFlags;
    let f = SessionFlags::from_bits_truncate(flags);
    c_int::from(f.contains(SessionFlags::CURSOR))
}

/// Check if view should restore folds.
#[no_mangle]
pub extern "C" fn rs_view_should_restore_folds(flags: u32) -> c_int {
    use crate::SessionFlags;
    let f = SessionFlags::from_bits_truncate(flags);
    c_int::from(f.contains(SessionFlags::FOLDS))
}

/// Check if view should restore local options.
#[no_mangle]
pub extern "C" fn rs_view_should_restore_options(flags: u32) -> c_int {
    use crate::SessionFlags;
    let f = SessionFlags::from_bits_truncate(flags);
    c_int::from(f.contains(SessionFlags::LOCALOPTIONS))
}

// =============================================================================
// Decode Helpers (for debugging/testing)
// =============================================================================

/// Decode a single view path character.
///
/// Returns the decoded character, or 0 if not part of an escape sequence.
#[no_mangle]
pub extern "C" fn rs_view_decode_char(c1: c_char, c2: c_char) -> c_char {
    let b1 = c1 as u8;
    let b2 = c2 as u8;

    if b1 != b'=' {
        return 0;
    }

    match b2 {
        b'=' => b'=' as c_char, // "==" -> '='
        b'+' => b'/' as c_char, // "=+" -> '/'
        b'-' => b':' as c_char, // "=-" -> ':'
        _ => 0,                 // Unknown sequence
    }
}

/// Check if the next two characters form an escape sequence.
#[no_mangle]
pub extern "C" fn rs_view_is_escape_seq(c1: c_char, c2: c_char) -> c_int {
    let b1 = c1 as u8;
    let b2 = c2 as u8;

    c_int::from(b1 == b'=' && matches!(b2, b'=' | b'+' | b'-'))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[allow(clippy::cast_lossless)]
    extern "C" fn test_is_path_sep(c: c_int) -> c_int {
        c_int::from(c == b'/' as c_int || c == b'\\' as c_int || c == b':' as c_int)
    }

    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn test_path_extra_len() {
        unsafe {
            let path = CString::new("/home/user/file.txt").unwrap();
            let extra = rs_view_path_extra_len(
                path.as_ptr(),
                path.as_bytes().len() as c_int,
                test_is_path_sep,
            );
            // 3 path separators
            assert_eq!(extra, 3);
        }
    }

    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn test_encode_path() {
        unsafe {
            let src = CString::new("/home/test").unwrap();
            let mut dst = [0i8; 20];
            let len = rs_view_encode_path(
                src.as_ptr(),
                src.as_bytes().len() as c_int,
                dst.as_mut_ptr(),
                test_is_path_sep,
                0,
            );
            // /home/test -> =+home=+test (12 chars)
            assert_eq!(len, 12);

            let dst_str: Vec<u8> = dst[..12].iter().map(|&c| c as u8).collect();
            assert_eq!(&dst_str, b"=+home=+test");
        }
    }

    #[test]
    fn test_view_file_total_len() {
        // viewdir(10) + '/'(1) + encoded_path(20) + '='(1) + ".vim"(4) + NUL(1) = 37
        assert_eq!(rs_view_file_total_len(10, 20, 0), 37);
        // With view number: +1
        assert_eq!(rs_view_file_total_len(10, 20, 1), 38);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_view_num_char() {
        assert_eq!(rs_view_num_char(1), b'1' as c_char);
        assert_eq!(rs_view_num_char(9), b'9' as c_char);
        assert_eq!(rs_view_num_char(0), 0);
        assert_eq!(rs_view_num_char(10), 0);
    }

    #[test]
    fn test_view_num_valid() {
        assert_eq!(rs_view_num_valid(0), 1);
        assert_eq!(rs_view_num_valid(5), 1);
        assert_eq!(rs_view_num_valid(9), 1);
        assert_eq!(rs_view_num_valid(-1), 0);
        assert_eq!(rs_view_num_valid(10), 0);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_view_parse_num() {
        assert_eq!(rs_view_parse_num(b'0' as c_char), 0);
        assert_eq!(rs_view_parse_num(b'5' as c_char), 5);
        assert_eq!(rs_view_parse_num(b'9' as c_char), 9);
        assert_eq!(rs_view_parse_num(b'a' as c_char), -1);
    }

    #[test]
    fn test_is_in_viewdir() {
        unsafe {
            let view = CString::new("~/.local/share/nvim/view/=+home=+user=+file=.vim").unwrap();
            assert_eq!(rs_view_is_in_viewdir(view.as_ptr()), 1);

            let other = CString::new("/home/user/file.vim").unwrap();
            assert_eq!(rs_view_is_in_viewdir(other.as_ptr()), 0);
        }
    }

    #[test]
    fn test_file_pattern_valid() {
        unsafe {
            // Default view: ends with "=.vim"
            let default = CString::new("path=+to=+file=.vim").unwrap();
            assert_eq!(rs_view_file_pattern_valid(default.as_ptr()), 1);

            // Numbered view: ends with "=N.vim"
            let numbered = CString::new("path=+to=+file=1.vim").unwrap();
            assert_eq!(rs_view_file_pattern_valid(numbered.as_ptr()), 1);

            // Invalid: wrong extension
            let invalid = CString::new("path=+to=+file.txt").unwrap();
            assert_eq!(rs_view_file_pattern_valid(invalid.as_ptr()), 0);
        }
    }

    #[test]
    fn test_check_error() {
        assert_eq!(rs_view_check_error(0, 0, 1), VIEW_ERR_NO_NAME);
        assert_eq!(rs_view_check_error(1, 10, 1), VIEW_ERR_INVALID);
        assert_eq!(rs_view_check_error(1, 0, 0), VIEW_ERR_NO_FILE);
        assert_eq!(rs_view_check_error(1, 0, 1), VIEW_OK);
    }

    #[test]
    fn test_should_restore() {
        use crate::SessionFlags;

        let cursor_flags = SessionFlags::CURSOR.bits();
        assert_eq!(rs_view_should_restore_cursor(cursor_flags), 1);
        assert_eq!(rs_view_should_restore_cursor(0), 0);

        let folds_flags = SessionFlags::FOLDS.bits();
        assert_eq!(rs_view_should_restore_folds(folds_flags), 1);

        let opts_flags = SessionFlags::LOCALOPTIONS.bits();
        assert_eq!(rs_view_should_restore_options(opts_flags), 1);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_decode_char() {
        assert_eq!(
            rs_view_decode_char(b'=' as c_char, b'=' as c_char),
            b'=' as c_char
        );
        assert_eq!(
            rs_view_decode_char(b'=' as c_char, b'+' as c_char),
            b'/' as c_char
        );
        assert_eq!(
            rs_view_decode_char(b'=' as c_char, b'-' as c_char),
            b':' as c_char
        );
        assert_eq!(rs_view_decode_char(b'a' as c_char, b'b' as c_char), 0);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_is_escape_seq() {
        assert_eq!(rs_view_is_escape_seq(b'=' as c_char, b'=' as c_char), 1);
        assert_eq!(rs_view_is_escape_seq(b'=' as c_char, b'+' as c_char), 1);
        assert_eq!(rs_view_is_escape_seq(b'=' as c_char, b'-' as c_char), 1);
        assert_eq!(rs_view_is_escape_seq(b'=' as c_char, b'x' as c_char), 0);
        assert_eq!(rs_view_is_escape_seq(b'a' as c_char, b'+' as c_char), 0);
    }
}
