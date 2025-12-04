//! Path utilities for Neovim
//!
//! Provides portable path manipulation functions compatible with nvim's path.c.
//!
//! Key functions:
//! - `vim_ispathsep` - Check if a character is a path separator
//! - `path_tail` - Get the filename component of a path
//! - `path_head_length` - Get the length of the path head

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{c_char, c_int};

/// Check if a character is a path separator.
///
/// On Unix, only '/' is a separator.
/// On Windows, '/', '\', and ':' are separators.
#[no_mangle]
pub extern "C" fn rs_vim_ispathsep(c: c_int) -> c_int {
    #[cfg(unix)]
    {
        c_int::from(c == b'/' as c_int)
    }

    #[cfg(windows)]
    {
        c_int::from(c == b':' as c_int || c == b'/' as c_int || c == b'\\' as c_int)
    }

    #[cfg(not(any(unix, windows)))]
    {
        c_int::from(c == b'/' as c_int)
    }
}

/// Check if a character is a path separator, excluding colon.
///
/// Like `rs_vim_ispathsep`, but excludes ':' on Windows.
#[no_mangle]
pub extern "C" fn rs_vim_ispathsep_nocolon(c: c_int) -> c_int {
    #[cfg(unix)]
    {
        c_int::from(c == b'/' as c_int)
    }

    #[cfg(windows)]
    {
        c_int::from(c == b'/' as c_int || c == b'\\' as c_int)
    }

    #[cfg(not(any(unix, windows)))]
    {
        c_int::from(c == b'/' as c_int)
    }
}

/// Check if a character is a path list separator.
///
/// On Unix, ':' is the separator (e.g., in $PATH).
/// On Windows, ';' is the separator.
#[no_mangle]
pub extern "C" fn rs_vim_ispathlistsep(c: c_int) -> c_int {
    #[cfg(unix)]
    {
        c_int::from(c == b':' as c_int)
    }

    #[cfg(not(unix))]
    {
        c_int::from(c == b';' as c_int)
    }
}

/// Get the length of the path head.
///
/// Returns 3 on Windows (for "C:\"), 1 otherwise (for "/").
#[no_mangle]
pub extern "C" fn rs_path_head_length() -> c_int {
    #[cfg(windows)]
    {
        3
    }

    #[cfg(not(windows))]
    {
        1
    }
}

/// Check if a path is absolute.
///
/// On Unix, a path is absolute if it starts with '/' or '~'.
/// On Windows, a path is absolute if it starts with a drive letter (e.g., "C:\")
/// or a UNC path (e.g., "\\server\share").
///
/// # Safety
///
/// `path` must be a valid null-terminated C string, or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_path_is_absolute(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let first = *path as u8;

    #[cfg(unix)]
    {
        // UNIX: starts with '/' or '~'
        c_int::from(first == b'/' || first == b'~')
    }

    #[cfg(windows)]
    {
        // Empty string is not absolute
        if first == 0 {
            return 0;
        }
        // Check for drive letter (e.g., "C:\") - must have path separator after colon
        if first.is_ascii_alphabetic() {
            let second = *path.add(1) as u8;
            if second == b':' {
                let third = *path.add(2) as u8;
                // Use vim_ispathsep_nocolon semantics (/ or \)
                if third == b'/' || third == b'\\' {
                    return 1;
                }
            }
        }
        // Check for UNC path (e.g., "\\server" or "//server")
        // Must have two identical separators
        if (first == b'\\' || first == b'/') {
            let second = *path.add(1) as u8;
            if first == second {
                return 1;
            }
        }
        0
    }

    #[cfg(not(any(unix, windows)))]
    {
        c_int::from(first == b'/' || first == b'~')
    }
}

/// Get the tail (filename) of a path.
///
/// Returns a pointer to the last component of the path.
/// If the path ends with a separator, returns an empty string.
/// If the path is NULL, returns an empty string.
///
/// # Safety
///
/// `fname` must be a valid null-terminated C string, or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_path_tail(fname: *const c_char) -> *const c_char {
    static EMPTY: &[u8] = b"\0";

    if fname.is_null() {
        return EMPTY.as_ptr().cast();
    }

    let mut p = fname;
    let mut tail = fname;

    // Skip past any drive letter or UNC prefix on Windows
    #[cfg(windows)]
    {
        let first = *p as u8;
        if first.is_ascii_alphabetic() {
            let second = *p.add(1) as u8;
            if second == b':' {
                p = p.add(2);
                tail = p;
            }
        } else if (first == b'/' || first == b'\\') {
            let second = *p.add(1) as u8;
            if second == b'/' || second == b'\\' {
                p = p.add(2);
                // Skip server name
                while *p != 0 && *p as u8 != b'/' && *p as u8 != b'\\' {
                    p = p.add(1);
                }
                if *p != 0 {
                    p = p.add(1);
                    tail = p;
                }
            }
        }
    }

    // Find the last path separator
    unsafe {
        while *p != 0 {
            let c = *p as u8;
            #[cfg(unix)]
            let is_sep = c == b'/';
            #[cfg(windows)]
            let is_sep = c == b'/' || c == b'\\';
            #[cfg(not(any(unix, windows)))]
            let is_sep = c == b'/';

            if is_sep {
                tail = p.add(1);
            }
            p = p.add(1);
        }
    }

    tail
}

// Return values for path_is_url
const URL_SLASH: c_int = 1;
const URL_BACKSLASH: c_int = 2;

/// Check if string starts with ":/" or ":\\".
///
/// Returns `URL_SLASH` (1) for ":/", `URL_BACKSLASH` (2) for ":\\", 0 otherwise.
/// This is a helper for `path_with_url` - it checks if we're at the scheme separator.
///
/// # Safety
///
/// `p` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_path_is_url(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }

    let c0 = *p as u8;
    let c1 = *p.add(1) as u8;

    if c0 == b':' && c1 == b'/' {
        return URL_SLASH;
    }

    if c0 == b':' && c1 == b'\\' {
        let c2 = *p.add(2) as u8;
        if c2 == b'\\' {
            return URL_BACKSLASH;
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_ispathsep() {
        #[cfg(unix)]
        {
            assert_eq!(rs_vim_ispathsep(b'/' as c_int), 1);
            assert_eq!(rs_vim_ispathsep(b'\\' as c_int), 0);
            assert_eq!(rs_vim_ispathsep(b':' as c_int), 0);
            assert_eq!(rs_vim_ispathsep(b'a' as c_int), 0);
        }
    }

    #[test]
    fn test_ispathsep_nocolon() {
        #[cfg(unix)]
        {
            assert_eq!(rs_vim_ispathsep_nocolon(b'/' as c_int), 1);
            assert_eq!(rs_vim_ispathsep_nocolon(b':' as c_int), 0);
        }
    }

    #[test]
    fn test_ispathlistsep() {
        #[cfg(unix)]
        {
            assert_eq!(rs_vim_ispathlistsep(b':' as c_int), 1);
            assert_eq!(rs_vim_ispathlistsep(b';' as c_int), 0);
        }
    }

    #[test]
    fn test_path_head_length() {
        #[cfg(not(windows))]
        {
            assert_eq!(rs_path_head_length(), 1);
        }
    }

    #[test]
    fn test_path_is_absolute() {
        #[cfg(unix)]
        {
            let abs = CString::new("/home/user").unwrap();
            assert_eq!(unsafe { rs_path_is_absolute(abs.as_ptr()) }, 1);

            let tilde = CString::new("~/documents").unwrap();
            assert_eq!(unsafe { rs_path_is_absolute(tilde.as_ptr()) }, 1);

            let rel = CString::new("home/user").unwrap();
            assert_eq!(unsafe { rs_path_is_absolute(rel.as_ptr()) }, 0);

            let dot = CString::new("./file").unwrap();
            assert_eq!(unsafe { rs_path_is_absolute(dot.as_ptr()) }, 0);
        }
    }

    #[test]
    fn test_path_tail() {
        let path = CString::new("/home/user/file.txt").unwrap();
        let tail = unsafe { rs_path_tail(path.as_ptr()) };
        let tail_str = unsafe { std::ffi::CStr::from_ptr(tail) };
        assert_eq!(tail_str.to_str().unwrap(), "file.txt");

        let just_file = CString::new("file.txt").unwrap();
        let tail = unsafe { rs_path_tail(just_file.as_ptr()) };
        let tail_str = unsafe { std::ffi::CStr::from_ptr(tail) };
        assert_eq!(tail_str.to_str().unwrap(), "file.txt");

        let trailing_slash = CString::new("/home/user/").unwrap();
        let tail = unsafe { rs_path_tail(trailing_slash.as_ptr()) };
        let tail_str = unsafe { std::ffi::CStr::from_ptr(tail) };
        assert_eq!(tail_str.to_str().unwrap(), "");
    }

    #[test]
    fn test_path_is_url() {
        // path_is_url checks if string starts with ":/" or ":\\"
        // It's called from path_with_url after scanning past the scheme name
        let url_slash = CString::new("://example.com").unwrap();
        assert_eq!(unsafe { rs_path_is_url(url_slash.as_ptr()) }, URL_SLASH);

        let url_backslash = CString::new(":\\\\server\\share").unwrap();
        assert_eq!(unsafe { rs_path_is_url(url_backslash.as_ptr()) }, URL_BACKSLASH);

        let just_colon = CString::new(":foo").unwrap();
        assert_eq!(unsafe { rs_path_is_url(just_colon.as_ptr()) }, 0);

        let no_colon = CString::new("/home/user").unwrap();
        assert_eq!(unsafe { rs_path_is_url(no_colon.as_ptr()) }, 0);
    }
}
