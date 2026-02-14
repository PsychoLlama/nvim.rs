//! Path manipulation and expansion
//!
//! This module handles path manipulation for runtime file searching.

use std::ffi::{c_char, c_int};

// =============================================================================
// Path Component Checking
// =============================================================================

/// Check if a path ends with a path separator.
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_ends_with_sep(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }

    // Find the end of the string
    let mut p = path;
    while !(*p == 0) {
        p = p.add(1);
    }

    // Check if previous char is a separator
    if p > path {
        let last = *p.sub(1) as u8;
        return last == b'/' || last == b'\\';
    }

    false
}

/// Check if a character is a path separator.
pub fn rs_is_path_sep(c: c_int) -> bool {
    let c = c as u8;
    c == b'/' || c == b'\\'
}

/// Check if a path is absolute.
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_is_absolute(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }

    let first = *path as u8;

    // Unix absolute path
    if first == b'/' {
        return true;
    }

    // Windows absolute path (drive letter)
    if first.is_ascii_alphabetic() {
        let second = *path.add(1) as u8;
        if second == b':' {
            return true;
        }
    }

    // Windows UNC path
    if first == b'\\' {
        let second = *path.add(1) as u8;
        if second == b'\\' {
            return true;
        }
    }

    false
}

/// Get the length of a path string.
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_strlen(path: *const c_char) -> usize {
    if path.is_null() {
        return 0;
    }

    let mut len = 0usize;
    let mut p = path;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }
    len
}

// =============================================================================
// Path Pattern Matching
// =============================================================================

/// Check if a path contains a wildcard character.
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_has_wildcard(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }

    let mut p = path;
    while *p != 0 {
        let c = *p as u8;
        if c == b'*' || c == b'?' || c == b'[' {
            return true;
        }
        p = p.add(1);
    }

    false
}

/// Check if a pattern matches the "after" directory.
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_is_after_dir(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }

    // Check for "after" at start or after separator
    static AFTER: &[u8] = b"after";

    let mut p = path;
    let mut i = 0;

    while i < AFTER.len() {
        if *p == 0 || (*p as u8) != AFTER[i] {
            return false;
        }
        p = p.add(1);
        i += 1;
    }

    // Must be followed by separator or end
    let c = *p as u8;
    c == 0 || c == b'/' || c == b'\\'
}

// =============================================================================
// Path Extension Checking
// =============================================================================

/// Check if a path has a Vim script extension (.vim).
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_has_vim_ext(path: *const c_char) -> bool {
    rs_path_has_ext(path, b".vim\0".as_ptr().cast())
}

/// Check if a path has a Lua script extension (.lua).
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_has_lua_ext(path: *const c_char) -> bool {
    rs_path_has_ext(path, b".lua\0".as_ptr().cast())
}

/// Check if a path ends with a given extension (case-insensitive).
///
/// # Safety
///
/// Both `path` and `ext` must be null or valid null-terminated C strings.
pub unsafe fn rs_path_has_ext(path: *const c_char, ext: *const c_char) -> bool {
    if path.is_null() || ext.is_null() {
        return false;
    }

    let path_len = rs_path_strlen(path);
    let ext_len = rs_path_strlen(ext);

    if ext_len == 0 || path_len < ext_len {
        return false;
    }

    // Compare from the end
    let path_suffix = path.add(path_len - ext_len);
    let mut p = path_suffix;
    let mut e = ext;

    while *e != 0 {
        let pc = (*p as u8).to_ascii_lowercase();
        let ec = (*e as u8).to_ascii_lowercase();
        if pc != ec {
            return false;
        }
        p = p.add(1);
        e = e.add(1);
    }

    true
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_is_path_sep() {
        assert!(rs_is_path_sep(b'/' as c_int));
        assert!(rs_is_path_sep(b'\\' as c_int));
        assert!(!rs_is_path_sep(b'a' as c_int));
        assert!(!rs_is_path_sep(0));
    }

    #[test]
    fn test_path_ends_with_sep() {
        unsafe {
            let path1 = CString::new("/home/user/").unwrap();
            assert!(rs_path_ends_with_sep(path1.as_ptr()));

            let path2 = CString::new("/home/user").unwrap();
            assert!(!rs_path_ends_with_sep(path2.as_ptr()));

            let path3 = CString::new("C:\\Users\\").unwrap();
            assert!(rs_path_ends_with_sep(path3.as_ptr()));

            assert!(!rs_path_ends_with_sep(std::ptr::null()));
        }
    }

    #[test]
    fn test_path_is_absolute() {
        unsafe {
            let unix = CString::new("/home/user").unwrap();
            assert!(rs_path_is_absolute(unix.as_ptr()));

            let win = CString::new("C:\\Users").unwrap();
            assert!(rs_path_is_absolute(win.as_ptr()));

            let unc = CString::new("\\\\server\\share").unwrap();
            assert!(rs_path_is_absolute(unc.as_ptr()));

            let relative = CString::new("home/user").unwrap();
            assert!(!rs_path_is_absolute(relative.as_ptr()));

            assert!(!rs_path_is_absolute(std::ptr::null()));
        }
    }

    #[test]
    fn test_path_has_wildcard() {
        unsafe {
            let wild1 = CString::new("*.vim").unwrap();
            assert!(rs_path_has_wildcard(wild1.as_ptr()));

            let wild2 = CString::new("file?.txt").unwrap();
            assert!(rs_path_has_wildcard(wild2.as_ptr()));

            let wild3 = CString::new("file[0-9].txt").unwrap();
            assert!(rs_path_has_wildcard(wild3.as_ptr()));

            let plain = CString::new("file.txt").unwrap();
            assert!(!rs_path_has_wildcard(plain.as_ptr()));
        }
    }

    #[test]
    fn test_path_has_ext() {
        unsafe {
            let vim = CString::new("file.vim").unwrap();
            assert!(rs_path_has_vim_ext(vim.as_ptr()));
            assert!(!rs_path_has_lua_ext(vim.as_ptr()));

            let lua = CString::new("file.lua").unwrap();
            assert!(rs_path_has_lua_ext(lua.as_ptr()));
            assert!(!rs_path_has_vim_ext(lua.as_ptr()));

            // Case insensitive
            let vim_upper = CString::new("file.VIM").unwrap();
            assert!(rs_path_has_vim_ext(vim_upper.as_ptr()));
        }
    }

    #[test]
    fn test_path_is_after_dir() {
        unsafe {
            let after = CString::new("after").unwrap();
            assert!(rs_path_is_after_dir(after.as_ptr()));

            let after_slash = CString::new("after/").unwrap();
            assert!(rs_path_is_after_dir(after_slash.as_ptr()));

            let not_after = CString::new("before").unwrap();
            assert!(!rs_path_is_after_dir(not_after.as_ptr()));

            let afterx = CString::new("afterx").unwrap();
            assert!(!rs_path_is_after_dir(afterx.as_ptr()));
        }
    }
}
