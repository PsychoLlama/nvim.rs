//! Path utilities for Neovim
//!
//! Provides portable path manipulation functions compatible with nvim's path.c.
//!
//! Key functions:
//! - `vim_ispathsep` - Check if a character is a path separator
//! - `path_tail` - Get the filename component of a path
//! - `path_head_length` - Get the length of the path head
//! - `after_pathsep` - Check if position is just after a path separator

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{c_char, c_int};

use nvim_mbyte::utf_head_off;

// ============================================================================
// Internal helpers
// ============================================================================

/// Check if a byte is a path separator (internal use).
#[inline]
fn is_pathsep(c: u8) -> bool {
    #[cfg(unix)]
    {
        c == b'/'
    }

    #[cfg(windows)]
    {
        c == b':' || c == b'/' || c == b'\\'
    }

    #[cfg(not(any(unix, windows)))]
    {
        c == b'/'
    }
}

// ============================================================================
// FFI exports
// ============================================================================

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

/// Check if path begins with characters denoting the head of a path.
///
/// On Unix, returns true if the path starts with a path separator ('/').
/// On Windows, returns true if the path starts with a drive letter (e.g., "D:").
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_is_path_head(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    #[cfg(windows)]
    {
        let c0 = *path as u8;
        let c1 = *path.add(1) as u8;
        c_int::from(c0.is_ascii_alphabetic() && c1 == b':')
    }

    #[cfg(not(windows))]
    {
        // On Unix, check if path starts with a path separator
        c_int::from(rs_vim_ispathsep(*path as c_int) != 0)
    }
}

/// Get a pointer to one character past the head of a path name.
///
/// On Unix: returns pointer past leading "/" characters.
/// On Windows: returns pointer past "C:\" (drive letter + separators).
/// If there is no head, the original path is returned.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_past_head(path: *const c_char) -> *const c_char {
    if path.is_null() {
        return path;
    }

    let mut retval = path;

    #[cfg(windows)]
    {
        // May skip "c:"
        if rs_is_path_head(path) != 0 {
            retval = path.add(2);
        }
    }

    // Skip past path separators
    while rs_vim_ispathsep(*retval as c_int) != 0 {
        retval = retval.add(1);
    }

    retval
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

/// Check if a path has a Windows drive letter.
///
/// A drive letter is: alpha followed by ':' or '|', then optionally
/// followed by '/', '\', '?', or '#'.
///
/// # Safety
///
/// `p` must be a valid pointer to at least `path_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_path_has_drive_letter(p: *const c_char, path_len: usize) -> c_int {
    if p.is_null() || path_len < 2 {
        return 0;
    }

    let c0 = *p as u8;
    let c1 = *p.add(1) as u8;

    // First char must be alpha
    if !c0.is_ascii_alphabetic() {
        return 0;
    }

    // Second char must be ':' or '|'
    if c1 != b':' && c1 != b'|' {
        return 0;
    }

    // If only 2 chars, that's a valid drive letter
    if path_len == 2 {
        return 1;
    }

    // Third char must be '/', '\', '?', or '#'
    let c2 = *p.add(2) as u8;
    c_int::from(c2 == b'/' || c2 == b'\\' || c2 == b'?' || c2 == b'#')
}

/// Check if a path starts with a URL scheme.
///
/// Returns `URL_SLASH` (1) for "scheme://", `URL_BACKSLASH` (2) for "scheme:\\",
/// or 0 if not a URL.
///
/// This function checks for a valid URL scheme following RFC3986:
/// - Must start with an alpha character
/// - Body can contain: alpha, digit, '+', '-', '.'
/// - Must end with a valid scheme character (not '+', '-', or '.')
/// - Must be followed by ":/" or ":\\"
///
/// # Safety
///
/// `fname` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_path_with_url(fname: *const c_char) -> c_int {
    if fname.is_null() {
        return 0;
    }

    let c0 = *fname as u8;

    // First character must be alpha
    if !c0.is_ascii_alphabetic() {
        return 0;
    }

    // Check for drive letter - if it looks like a drive letter, not a URL
    // Need to find length first
    let mut len = 0;
    let mut p = fname;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    if rs_path_has_drive_letter(fname, len) != 0 {
        return 0;
    }

    // Scan the scheme body: alpha, digit, '+', '-', '.'
    p = fname.add(1);
    while {
        let c = *p as u8;
        c.is_ascii_alphanumeric() || c == b'+' || c == b'-' || c == b'.'
    } {
        p = p.add(1);
    }

    // Check last char before p is not '+', '-', or '.'
    // p points to first non-scheme char, so check p-1
    if p > fname.add(1) {
        let last = *p.sub(1) as u8;
        if last == b'+' || last == b'-' || last == b'.' {
            return 0;
        }
    }

    // Check for ":/" or ":\\"
    rs_path_is_url(p)
}

/// Check if a path is a full (absolute) path name or URL.
///
/// Returns true if the path is either:
/// - A URL (starts with a scheme like "http://")
/// - An absolute path (starts with "/" on Unix, "C:\" on Windows)
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_isAbsName(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }
    c_int::from(rs_path_with_url(name) != 0 || rs_path_is_absolute(name) != 0)
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
        assert_eq!(
            unsafe { rs_path_is_url(url_backslash.as_ptr()) },
            URL_BACKSLASH
        );

        let just_colon = CString::new(":foo").unwrap();
        assert_eq!(unsafe { rs_path_is_url(just_colon.as_ptr()) }, 0);

        let no_colon = CString::new("/home/user").unwrap();
        assert_eq!(unsafe { rs_path_is_url(no_colon.as_ptr()) }, 0);
    }

    #[test]
    fn test_is_path_head() {
        #[cfg(unix)]
        {
            // On Unix, path head starts with '/'
            let root = CString::new("/home/user").unwrap();
            assert_eq!(unsafe { rs_is_path_head(root.as_ptr()) }, 1);

            let slash = CString::new("/").unwrap();
            assert_eq!(unsafe { rs_is_path_head(slash.as_ptr()) }, 1);

            let rel = CString::new("home/user").unwrap();
            assert_eq!(unsafe { rs_is_path_head(rel.as_ptr()) }, 0);

            let dot = CString::new("./file").unwrap();
            assert_eq!(unsafe { rs_is_path_head(dot.as_ptr()) }, 0);

            let tilde = CString::new("~/file").unwrap();
            assert_eq!(unsafe { rs_is_path_head(tilde.as_ptr()) }, 0);

            let empty = CString::new("").unwrap();
            assert_eq!(unsafe { rs_is_path_head(empty.as_ptr()) }, 0);

            // NULL returns 0
            assert_eq!(unsafe { rs_is_path_head(std::ptr::null()) }, 0);
        }
    }

    #[test]
    fn test_get_past_head() {
        #[cfg(unix)]
        {
            // Skip past leading slashes
            let root = CString::new("/home/user").unwrap();
            let past = unsafe { rs_get_past_head(root.as_ptr()) };
            let past_str = unsafe { std::ffi::CStr::from_ptr(past) };
            assert_eq!(past_str.to_str().unwrap(), "home/user");

            // Multiple slashes
            let multi = CString::new("///home/user").unwrap();
            let past = unsafe { rs_get_past_head(multi.as_ptr()) };
            let past_str = unsafe { std::ffi::CStr::from_ptr(past) };
            assert_eq!(past_str.to_str().unwrap(), "home/user");

            // Just slash
            let slash = CString::new("/").unwrap();
            let past = unsafe { rs_get_past_head(slash.as_ptr()) };
            let past_str = unsafe { std::ffi::CStr::from_ptr(past) };
            assert_eq!(past_str.to_str().unwrap(), "");

            // No head - returns original
            let rel = CString::new("home/user").unwrap();
            let past = unsafe { rs_get_past_head(rel.as_ptr()) };
            let past_str = unsafe { std::ffi::CStr::from_ptr(past) };
            assert_eq!(past_str.to_str().unwrap(), "home/user");

            // Empty string
            let empty = CString::new("").unwrap();
            let past = unsafe { rs_get_past_head(empty.as_ptr()) };
            let past_str = unsafe { std::ffi::CStr::from_ptr(past) };
            assert_eq!(past_str.to_str().unwrap(), "");

            // NULL returns NULL
            let past = unsafe { rs_get_past_head(std::ptr::null()) };
            assert!(past.is_null());
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_vim_isAbsName() {
        #[cfg(unix)]
        {
            // Absolute paths
            let abs = CString::new("/home/user").unwrap();
            assert_eq!(unsafe { rs_vim_isAbsName(abs.as_ptr()) }, 1);

            let tilde = CString::new("~/documents").unwrap();
            assert_eq!(unsafe { rs_vim_isAbsName(tilde.as_ptr()) }, 1);

            // URLs
            let http = CString::new("http://example.com").unwrap();
            assert_eq!(unsafe { rs_vim_isAbsName(http.as_ptr()) }, 1);

            let https = CString::new("https://example.com").unwrap();
            assert_eq!(unsafe { rs_vim_isAbsName(https.as_ptr()) }, 1);

            let file = CString::new("file:///path").unwrap();
            assert_eq!(unsafe { rs_vim_isAbsName(file.as_ptr()) }, 1);

            // Relative paths (not absolute, not URL)
            let rel = CString::new("home/user").unwrap();
            assert_eq!(unsafe { rs_vim_isAbsName(rel.as_ptr()) }, 0);

            let dot = CString::new("./file").unwrap();
            assert_eq!(unsafe { rs_vim_isAbsName(dot.as_ptr()) }, 0);

            let dotdot = CString::new("../file").unwrap();
            assert_eq!(unsafe { rs_vim_isAbsName(dotdot.as_ptr()) }, 0);

            let file_only = CString::new("file.txt").unwrap();
            assert_eq!(unsafe { rs_vim_isAbsName(file_only.as_ptr()) }, 0);

            // Empty and NULL
            let empty = CString::new("").unwrap();
            assert_eq!(unsafe { rs_vim_isAbsName(empty.as_ptr()) }, 0);

            assert_eq!(unsafe { rs_vim_isAbsName(std::ptr::null()) }, 0);
        }
    }
}

// ============================================================================
// after_pathsep - Check if position is just after a path separator
// ============================================================================

/// Return true if "p_offset" points to just after a path separator.
/// Takes care of multi-byte characters.
/// "base" must contain the full path/filename.
///
/// # Arguments
/// * `base` - The base string (start of the file name)
/// * `p_offset` - Offset into base where we're checking
///
/// # Returns
/// true if the character before p_offset is a path separator and it's
/// the start of a UTF-8 sequence (not a continuation byte).
#[inline]
pub fn after_pathsep(base: &[u8], p_offset: usize) -> bool {
    if p_offset == 0 || p_offset > base.len() {
        return false;
    }

    // Check if the byte before p is a path separator
    let prev_byte = base[p_offset - 1];
    if !is_pathsep(prev_byte) {
        return false;
    }

    // Check if the byte before p is the start of a UTF-8 sequence
    // (utf_head_off returns 0 if we're at the start of a character)
    utf_head_off(base, p_offset - 1) == 0
}

/// FFI wrapper for `after_pathsep`.
///
/// # Safety
/// - `b` and `p` must be valid pointers
/// - `p` must point to a position within or at the end of the string starting at `b`
#[no_mangle]
pub unsafe extern "C" fn rs_after_pathsep(b: *const c_char, p: *const c_char) -> c_int {
    if b.is_null() || p.is_null() || p < b {
        return 0;
    }

    let p_offset = (p as usize) - (b as usize);

    // We need at least p_offset bytes, but also room for utf_head_off to look back
    // Use p_offset + 8 as a conservative estimate
    let len = p_offset + 8;

    let slice = std::slice::from_raw_parts(b as *const u8, len);
    c_int::from(after_pathsep(slice, p_offset))
}

#[cfg(test)]
mod after_pathsep_tests {
    use super::*;

    #[test]
    fn test_after_pathsep_basic() {
        // After a slash
        let path = b"/home/user";
        assert!(after_pathsep(path, 1)); // After first '/'
        assert!(after_pathsep(path, 6)); // After '/' before 'user'

        // Not after slash
        assert!(!after_pathsep(path, 0)); // At start
        assert!(!after_pathsep(path, 2)); // After 'h'
        assert!(!after_pathsep(path, 5)); // After 'e'
    }

    #[test]
    fn test_after_pathsep_multibyte() {
        // Path with multibyte character: "/中/file"
        // '中' is 3 bytes: E4 B8 AD
        let path = b"/\xE4\xB8\xAD/file";

        assert!(after_pathsep(path, 1)); // After first '/'
        assert!(after_pathsep(path, 5)); // After second '/' (position of 'f')

        // In the middle of '中' - not after pathsep
        assert!(!after_pathsep(path, 2)); // Second byte of '中'
        assert!(!after_pathsep(path, 3)); // Third byte of '中'
        assert!(!after_pathsep(path, 4)); // This is '/' but checking position after it
    }

    #[test]
    fn test_after_pathsep_edge_cases() {
        let path = b"/";
        assert!(after_pathsep(path, 1)); // After the only '/'
        assert!(!after_pathsep(path, 0)); // At start

        let empty = b"";
        assert!(!after_pathsep(empty, 0)); // Empty string
    }

    #[test]
    #[cfg(unix)]
    fn test_after_pathsep_unix_only() {
        // On Unix, only '/' is a path separator
        let path = b"C:\\Windows";
        assert!(!after_pathsep(path, 3)); // After '\' - not a pathsep on Unix
    }
}
