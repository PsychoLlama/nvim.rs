//! File and path completion support.
//!
//! This module provides helper functions for filename and path completion.
//! The core wildcard expansion and file matching remain in C, but Rust
//! provides utilities for path manipulation and state management.

use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    // CTRL-X mode checking
    fn nvim_get_ctrl_x_mode() -> c_int;

    // State accessors
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
}

// CTRL-X mode constants
const CTRL_X_FILES: c_int = 4;
const CTRL_X_PATH_PATTERNS: c_int = 6 + 0x100; // 6 + CTRL_X_WANT_IDENT
const CTRL_X_PATH_DEFINES: c_int = 7 + 0x100; // 7 + CTRL_X_WANT_IDENT

// Path separator (Unix)
#[allow(clippy::cast_possible_wrap)]
const PATH_SEP: c_char = b'/' as c_char;

/// Check if we're in filename completion mode.
#[no_mangle]
pub unsafe extern "C" fn rs_file_is_files_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_FILES)
}

/// Check if we're in path patterns mode (include file completion).
#[no_mangle]
pub unsafe extern "C" fn rs_file_is_path_patterns_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_PATH_PATTERNS)
}

/// Check if we're in path defines mode.
#[no_mangle]
pub unsafe extern "C" fn rs_file_is_path_defines_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_PATH_DEFINES)
}

/// Check if we're in any file/path completion mode.
#[no_mangle]
pub unsafe extern "C" fn rs_file_is_file_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_FILES || mode == CTRL_X_PATH_PATTERNS || mode == CTRL_X_PATH_DEFINES)
}

/// Check if completion was interrupted during file search.
#[no_mangle]
pub unsafe extern "C" fn rs_file_was_interrupted() -> c_int {
    nvim_get_compl_interrupted()
}

/// Get the current completion direction for file search.
#[no_mangle]
pub unsafe extern "C" fn rs_file_get_direction() -> c_int {
    nvim_get_compl_direction()
}

/// Find the last path separator in a string.
///
/// Returns the byte offset of the last separator, or -1 if not found.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_find_last_sep(path: *const c_char) -> c_int {
    if path.is_null() {
        return -1;
    }

    let mut last_pos: c_int = -1;
    let mut pos = 0;
    let mut ptr = path;

    while *ptr != 0 {
        if *ptr == PATH_SEP {
            last_pos = pos;
        }
        ptr = ptr.add(1);
        pos += 1;
    }

    last_pos
}

/// Find the last path separator in a string (Windows-aware).
///
/// Also checks for backslash as a separator.
/// Returns the byte offset of the last separator, or -1 if not found.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_find_last_sep_any(path: *const c_char) -> c_int {
    if path.is_null() {
        return -1;
    }

    let mut last_pos: c_int = -1;
    let mut pos = 0;
    let mut ptr = path;

    while *ptr != 0 {
        if *ptr == b'/' as c_char || *ptr == b'\\' as c_char {
            last_pos = pos;
        }
        ptr = ptr.add(1);
        pos += 1;
    }

    last_pos
}

/// Get the length of the directory part of a path.
///
/// Returns 0 if there's no directory component.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_file_dir_len(path: *const c_char) -> c_int {
    let last_sep = rs_file_find_last_sep_any(path);
    if last_sep < 0 {
        0
    } else {
        last_sep + 1
    }
}

/// Get the length of the filename part of a path.
///
/// Returns the string length if there's no directory component.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_basename_len(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let last_sep = rs_file_find_last_sep_any(path);
    let mut len = 0;
    let mut ptr = path;

    while *ptr != 0 {
        len += 1;
        ptr = ptr.add(1);
    }

    if last_sep < 0 {
        len
    } else {
        len - last_sep - 1
    }
}

/// Check if a path is absolute.
///
/// Returns 1 if path starts with / (Unix) or has a drive letter (Windows).
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_is_absolute(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }

    // Unix absolute path
    if *path == b'/' as c_char {
        return 1;
    }

    // Windows drive letter (e.g., C:\)
    let first = *path;
    let second = *path.add(1);
    if ((first >= b'A' as c_char && first <= b'Z' as c_char)
        || (first >= b'a' as c_char && first <= b'z' as c_char))
        && second == b':' as c_char
    {
        return 1;
    }

    0
}

/// Check if a path ends with a path separator.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_ends_with_sep(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }

    // Find end of string
    let mut ptr = path;
    while *ptr != 0 {
        ptr = ptr.add(1);
    }

    // Check last character
    let last = *ptr.sub(1);
    c_int::from(last == b'/' as c_char || last == b'\\' as c_char)
}

/// Count path components in a path.
///
/// Returns the number of directory levels.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_count_path_components(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }

    let mut count = 0;
    let mut ptr = path;
    let mut in_component = false;

    while *ptr != 0 {
        if *ptr == b'/' as c_char || *ptr == b'\\' as c_char {
            if in_component {
                count += 1;
                in_component = false;
            }
        } else {
            in_component = true;
        }
        ptr = ptr.add(1);
    }

    // Count last component if not ending with separator
    if in_component {
        count += 1;
    }

    count
}

// =============================================================================
// Phase 4: Extended File Completion Functions
// =============================================================================

// Additional C accessor functions
extern "C" {
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_col() -> c_int;
    fn nvim_get_cursor_col() -> c_int;
}

/// Check if file completion is active.
#[no_mangle]
pub unsafe extern "C" fn rs_file_is_active() -> c_int {
    let started = nvim_get_compl_started();
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(
        started != 0
            && (mode == CTRL_X_FILES
                || mode == CTRL_X_PATH_PATTERNS
                || mode == CTRL_X_PATH_DEFINES),
    )
}

/// Check if file completion can continue.
#[no_mangle]
pub unsafe extern "C" fn rs_file_can_continue() -> c_int {
    let started = nvim_get_compl_started();
    let interrupted = nvim_get_compl_interrupted();
    c_int::from(started != 0 && interrupted == 0)
}

/// Get the length of text typed for file completion.
#[no_mangle]
pub unsafe extern "C" fn rs_file_typed_len() -> c_int {
    let cursor_col = nvim_get_cursor_col();
    let compl_col = nvim_get_compl_col();
    let len = cursor_col - compl_col;
    if len < 0 {
        0
    } else {
        len
    }
}

/// Check if path has a wildcard character.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_file_has_wildcard(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let mut ptr = path;
    while *ptr != 0 {
        let c = *ptr;
        if c == b'*' as c_char || c == b'?' as c_char || c == b'[' as c_char {
            return 1;
        }
        ptr = ptr.add(1);
    }

    0
}

/// Check if path is a hidden file (starts with dot).
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_is_hidden(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }

    // Find basename
    let last_sep = rs_file_find_last_sep_any(path);
    let basename = if last_sep < 0 {
        path
    } else {
        #[allow(clippy::cast_sign_loss)]
        path.add(last_sep as usize + 1)
    };

    // Check if starts with dot
    c_int::from(*basename == b'.' as c_char)
}

/// Check if path represents current or parent directory.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_is_dot_or_dotdot(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }

    // Find basename
    let last_sep = rs_file_find_last_sep_any(path);
    let basename = if last_sep < 0 {
        path
    } else {
        #[allow(clippy::cast_sign_loss)]
        path.add(last_sep as usize + 1)
    };

    // Check for "." or ".."
    if *basename == b'.' as c_char {
        let next = *basename.add(1);
        if next == 0 {
            return 1; // "."
        }
        if next == b'.' as c_char && *basename.add(2) == 0 {
            return 1; // ".."
        }
    }

    0
}

/// Get the extension of a filename.
///
/// Returns the offset of the extension (including the dot), or -1 if no extension.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_file_extension_offset(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return -1;
    }

    // Find basename first
    let last_sep = rs_file_find_last_sep_any(path);
    let basename_start = if last_sep < 0 {
        0
    } else {
        (last_sep + 1) as usize
    };

    // Find last dot in basename
    let mut last_dot: c_int = -1;
    let mut pos = basename_start;
    let mut ptr = path.add(basename_start);

    while *ptr != 0 {
        if *ptr == b'.' as c_char {
            #[allow(clippy::cast_possible_truncation)]
            {
                last_dot = pos as c_int;
            }
        }
        ptr = ptr.add(1);
        pos += 1;
    }

    // Don't count dot at start of basename as extension
    #[allow(clippy::cast_possible_truncation)]
    if last_dot == basename_start as c_int {
        -1
    } else {
        last_dot
    }
}

/// Compare two paths for equality, normalizing separators.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_paths_equal(path1: *const c_char, path2: *const c_char) -> c_int {
    if path1.is_null() || path2.is_null() {
        return c_int::from(path1 == path2); // Both null = equal
    }

    let mut p1 = path1;
    let mut p2 = path2;

    while *p1 != 0 && *p2 != 0 {
        let c1 = if *p1 == b'\\' as c_char {
            b'/' as c_char
        } else {
            *p1
        };
        let c2 = if *p2 == b'\\' as c_char {
            b'/' as c_char
        } else {
            *p2
        };

        if c1 != c2 {
            return 0;
        }

        p1 = p1.add(1);
        p2 = p2.add(1);
    }

    c_int::from(*p1 == 0 && *p2 == 0)
}

/// Check if path1 is a prefix of path2.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_file_is_prefix(prefix: *const c_char, path: *const c_char) -> c_int {
    if prefix.is_null() || path.is_null() {
        return 0;
    }

    let mut p1 = prefix;
    let mut p2 = path;

    while *p1 != 0 {
        if *p2 == 0 {
            return 0; // path is shorter
        }

        let c1 = if *p1 == b'\\' as c_char {
            b'/' as c_char
        } else {
            *p1
        };
        let c2 = if *p2 == b'\\' as c_char {
            b'/' as c_char
        } else {
            *p2
        };

        if c1 != c2 {
            return 0;
        }

        p1 = p1.add(1);
        p2 = p2.add(1);
    }

    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(CTRL_X_FILES, 4);
        assert_eq!(CTRL_X_PATH_PATTERNS, 6 + 0x100);
        assert_eq!(CTRL_X_PATH_DEFINES, 7 + 0x100);
    }

    #[test]
    fn test_find_last_sep() {
        unsafe {
            let path1 = b"/home/user/file.txt\0";
            let path2 = b"file.txt\0";
            let path3 = b"/\0";

            assert_eq!(rs_file_find_last_sep(path1.as_ptr().cast::<c_char>()), 10);
            assert_eq!(rs_file_find_last_sep(path2.as_ptr().cast::<c_char>()), -1);
            assert_eq!(rs_file_find_last_sep(path3.as_ptr().cast::<c_char>()), 0);
        }
    }

    #[test]
    fn test_dir_len() {
        unsafe {
            let path1 = b"/home/user/file.txt\0";
            let path2 = b"file.txt\0";

            assert_eq!(rs_file_dir_len(path1.as_ptr().cast::<c_char>()), 11);
            assert_eq!(rs_file_dir_len(path2.as_ptr().cast::<c_char>()), 0);
        }
    }

    #[test]
    fn test_basename_len() {
        unsafe {
            let path1 = b"/home/user/file.txt\0";
            let path2 = b"file.txt\0";

            assert_eq!(rs_file_basename_len(path1.as_ptr().cast::<c_char>()), 8);
            assert_eq!(rs_file_basename_len(path2.as_ptr().cast::<c_char>()), 8);
        }
    }

    #[test]
    fn test_is_absolute() {
        unsafe {
            let abs_unix = b"/home/user\0";
            let abs_win = b"C:\\Users\0";
            let relative = b"./file.txt\0";

            assert_eq!(rs_file_is_absolute(abs_unix.as_ptr().cast::<c_char>()), 1);
            assert_eq!(rs_file_is_absolute(abs_win.as_ptr().cast::<c_char>()), 1);
            assert_eq!(rs_file_is_absolute(relative.as_ptr().cast::<c_char>()), 0);
        }
    }

    #[test]
    fn test_ends_with_sep() {
        unsafe {
            let with_sep = b"/home/user/\0";
            let without = b"/home/user\0";

            assert_eq!(rs_file_ends_with_sep(with_sep.as_ptr().cast::<c_char>()), 1);
            assert_eq!(rs_file_ends_with_sep(without.as_ptr().cast::<c_char>()), 0);
        }
    }

    #[test]
    fn test_count_path_components() {
        unsafe {
            let path1 = b"/home/user/file.txt\0";
            let path2 = b"file.txt\0";
            let path3 = b"/\0";

            assert_eq!(
                rs_file_count_path_components(path1.as_ptr().cast::<c_char>()),
                3
            );
            assert_eq!(
                rs_file_count_path_components(path2.as_ptr().cast::<c_char>()),
                1
            );
            assert_eq!(
                rs_file_count_path_components(path3.as_ptr().cast::<c_char>()),
                0
            );
        }
    }
}
