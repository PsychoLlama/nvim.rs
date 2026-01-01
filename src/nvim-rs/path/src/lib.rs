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
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::ptr_as_ptr)]

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

// ============================================================================
// Shell detection functions
// ============================================================================

extern "C" {
    fn nvim_get_p_sh() -> *const c_char;
    fn nvim_get_p_ffs() -> *const c_char;
}

/// Check if 'shell' option contains "csh" in the tail.
///
/// Returns 1 if the shell appears to be csh-like, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_csh_like_shell() -> c_int {
    unsafe {
        let p_sh = nvim_get_p_sh();
        if p_sh.is_null() {
            return 0;
        }
        let tail = rs_path_tail(p_sh);
        if tail.is_null() {
            return 0;
        }
        // Check if "csh" substring exists in tail
        let mut p = tail;
        while *p != 0 {
            if *p as u8 == b'c' {
                let next = p.add(1);
                if *next as u8 == b's' {
                    let next2 = p.add(2);
                    if *next2 as u8 == b'h' {
                        return 1;
                    }
                }
            }
            p = p.add(1);
        }
        0
    }
}

/// Check if 'shell' option contains "fish" in the tail.
///
/// Returns 1 if the shell appears to be fish-like, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_fish_like_shell() -> c_int {
    unsafe {
        let p_sh = nvim_get_p_sh();
        if p_sh.is_null() {
            return 0;
        }
        let tail = rs_path_tail(p_sh);
        if tail.is_null() {
            return 0;
        }
        // Check if "fish" substring exists in tail
        let mut p = tail;
        while *p != 0 {
            if *p as u8 == b'f' {
                let next1 = p.add(1);
                if *next1 as u8 == b'i' {
                    let next2 = p.add(2);
                    if *next2 as u8 == b's' {
                        let next3 = p.add(3);
                        if *next3 as u8 == b'h' {
                            return 1;
                        }
                    }
                }
            }
            p = p.add(1);
        }
        0
    }
}

// EOL type values matching C definitions
const EOL_UNIX: c_int = 0;
const EOL_DOS: c_int = 1;
const EOL_MAC: c_int = 2;

/// Return the default fileformat from 'fileformats' option.
///
/// Returns EOL_MAC (2) for 'mac', EOL_DOS (1) for 'dos', EOL_UNIX (0) otherwise.
#[no_mangle]
pub extern "C" fn rs_default_fileformat() -> c_int {
    unsafe {
        let p_ffs = nvim_get_p_ffs();
        if p_ffs.is_null() || *p_ffs == 0 {
            return EOL_UNIX;
        }
        match *p_ffs as u8 {
            b'm' => EOL_MAC,
            b'd' => EOL_DOS,
            _ => EOL_UNIX,
        }
    }
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

    #[test]
    fn test_url_type_constants() {
        // Verify URL_SLASH and URL_BACKSLASH constants match C definitions
        assert_eq!(URL_SLASH, 1);
        assert_eq!(URL_BACKSLASH, 2);
    }

    #[test]
    fn test_eol_type_constants() {
        // Verify EOL_* constants match C definitions
        assert_eq!(EOL_UNIX, 0);
        assert_eq!(EOL_DOS, 1);
        assert_eq!(EOL_MAC, 2);
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

// ============================================================================
// File name comparison functions
// ============================================================================

extern "C" {
    /// Get the 'fileignorecase' option value.
    fn nvim_get_p_fic() -> c_int;
}

/// Compare two file names, respecting 'fileignorecase'.
///
/// Handles '/' and '\\' correctly on Windows.
///
/// # Safety
/// - `fname1` and `fname2` must be valid null-terminated C strings.
///
/// # Returns
/// 0 if they are equal, non-zero otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_path_fnamecmp(fname1: *const c_char, fname2: *const c_char) -> c_int {
    if fname1.is_null() || fname2.is_null() {
        if fname1.is_null() && fname2.is_null() {
            return 0;
        }
        return if fname1.is_null() { -1 } else { 1 };
    }

    #[cfg(windows)]
    {
        // On Windows, use path_fnamencmp with max of both lengths
        let len1 = {
            let mut p = fname1;
            let mut count = 0usize;
            while *p != 0 {
                p = p.add(1);
                count += 1;
            }
            count
        };
        let len2 = {
            let mut p = fname2;
            let mut count = 0usize;
            while *p != 0 {
                p = p.add(1);
                count += 1;
            }
            count
        };
        let max_len = if len1 > len2 { len1 } else { len2 };
        rs_path_fnamencmp(fname1, fname2, max_len)
    }

    #[cfg(not(windows))]
    {
        // On Unix, use mb_strcmp_ic with p_fic
        let fic = nvim_get_p_fic() != 0;
        nvim_mbyte::rs_mb_strcmp_ic(fic, fname1, fname2)
    }
}

/// Compare two file names up to `len` bytes, respecting 'fileignorecase'.
///
/// Handles '/' and '\\' correctly on Windows.
///
/// # Safety
/// - `fname1` and `fname2` must be valid null-terminated C strings.
///
/// # Returns
/// 0 if they are equal, non-zero otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_path_fnamencmp(
    fname1: *const c_char,
    fname2: *const c_char,
    len: usize,
) -> c_int {
    if fname1.is_null() || fname2.is_null() {
        if fname1.is_null() && fname2.is_null() {
            return 0;
        }
        return if fname1.is_null() { -1 } else { 1 };
    }

    let fic = nvim_get_p_fic() != 0;

    #[cfg(windows)]
    {
        // On Windows, need to handle / and \ as equivalent
        let mut c1: c_int = 0;
        let mut c2: c_int = 0;
        let mut p1 = fname1;
        let mut p2 = fname2;
        let mut remaining = len;

        while remaining > 0 {
            let slice1 = std::slice::from_raw_parts(p1 as *const u8, 6.min(remaining + 1));
            let slice2 = std::slice::from_raw_parts(p2 as *const u8, 6.min(remaining + 1));
            c1 = nvim_mbyte::utf_ptr2char(slice1);
            c2 = nvim_mbyte::utf_ptr2char(slice2);

            // Check for end of string or mismatch
            let both_pathsep = (c1 == b'/' as i32 || c1 == b'\\' as i32)
                && (c2 == b'/' as i32 || c2 == b'\\' as i32);

            if c1 == 0 || c2 == 0 || !both_pathsep {
                let mismatch = if fic {
                    c1 != c2 && nvim_mbyte::utf_fold(c1) != nvim_mbyte::utf_fold(c2)
                } else {
                    c1 != c2
                };
                if mismatch {
                    break;
                }
            }

            let l1 = nvim_mbyte::utfc_ptr2len(slice1);
            remaining = remaining.saturating_sub(l1);
            p1 = p1.add(l1);
            p2 = p2.add(nvim_mbyte::utfc_ptr2len(slice2));
        }

        if fic {
            (nvim_mbyte::utf_fold(c1) - nvim_mbyte::utf_fold(c2)) as c_int
        } else {
            (c1 - c2) as c_int
        }
    }

    #[cfg(not(windows))]
    {
        // On Unix, use simple comparison
        if fic {
            nvim_mbyte::rs_mb_strnicmp(fname1, fname2, len)
        } else {
            // Use libc strncmp
            libc::strncmp(fname1, fname2, len) as c_int
        }
    }
}

// ============================================================================
// gettail_dir - Get end of directory name
// ============================================================================

/// Get the end of the directory name.
///
/// Returns a pointer to the end of the directory name, on the first path separator.
///
/// Examples:
/// - "/path/file" -> pointer to '/' before "file"
/// - "/path/dir/" -> pointer to '/' before "dir/"
/// - "/path//dir" -> pointer to first '/' before "/dir"
/// - "/file" -> pointer to start (fname)
///
/// # Safety
/// - `fname` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_gettail_dir(fname: *const c_char) -> *const c_char {
    if fname.is_null() {
        return fname;
    }

    let mut dir_end = fname;
    let mut next_dir_end = fname;
    let mut look_for_sep = true;
    let mut p = fname;

    while *p != 0 {
        if rs_vim_ispathsep(*p as c_int) != 0 {
            if look_for_sep {
                next_dir_end = p;
                look_for_sep = false;
            }
        } else {
            if !look_for_sep {
                dir_end = next_dir_end;
            }
            look_for_sep = true;
        }
        // MB_PTR_ADV: advance by UTF-8 character length
        // Create a slice for utfc_ptr2len
        let slice = std::slice::from_raw_parts(p as *const u8, 8);
        let len = nvim_mbyte::utfc_ptr2len(slice);
        p = p.add(len);
    }

    dir_end
}

// ============================================================================
// path_next_component - Get the next path component
// ============================================================================

/// Get the next path component of a path name.
///
/// Returns a pointer to the first found path separator + 1.
/// Returns an empty string if `fname` doesn't contain a path separator.
///
/// # Safety
/// - `fname` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_path_next_component(fname: *const c_char) -> *const c_char {
    if fname.is_null() {
        return fname;
    }

    let mut p = fname;
    // Skip until we find a path separator or end of string
    while *p != 0 && rs_vim_ispathsep(*p as c_int) == 0 {
        // MB_PTR_ADV: advance by UTF-8 character length
        let slice = std::slice::from_raw_parts(p as *const u8, 8);
        let len = nvim_mbyte::utfc_ptr2len(slice);
        p = p.add(len);
    }
    // If we found a separator, skip past it
    if *p != 0 {
        p = p.add(1);
    }
    p
}

// ============================================================================
// path_tail_with_sep - Get path tail including leading separator
// ============================================================================

/// Get the path tail, but include the last path separator.
///
/// Returns:
/// - Pointer to the last path separator of `fname`, if there is any.
/// - `fname` if it contains no path separator.
/// - Never NULL.
///
/// # Safety
/// - `fname` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_path_tail_with_sep(fname: *const c_char) -> *const c_char {
    if fname.is_null() {
        return fname;
    }

    // Don't remove the '/' from "c:/file".
    let past_head = rs_get_past_head(fname);
    let mut tail = rs_path_tail(fname);

    // Move back past path separators (but don't go before past_head)
    while tail > past_head && rs_after_pathsep(fname, tail) != 0 {
        tail = tail.sub(1);
    }

    tail
}

#[cfg(test)]
mod path_next_component_tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_path_next_component_basic() {
        let path = CString::new("path/to/file").unwrap();
        let result = unsafe { rs_path_next_component(path.as_ptr()) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        assert_eq!(result_str.to_str().unwrap(), "to/file");
    }

    #[test]
    fn test_path_next_component_absolute() {
        let path = CString::new("/home/user").unwrap();
        let result = unsafe { rs_path_next_component(path.as_ptr()) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        assert_eq!(result_str.to_str().unwrap(), "home/user");
    }

    #[test]
    fn test_path_next_component_no_sep() {
        let path = CString::new("filename").unwrap();
        let result = unsafe { rs_path_next_component(path.as_ptr()) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        // Should return empty string (pointing to end)
        assert_eq!(result_str.to_str().unwrap(), "");
    }

    #[test]
    fn test_path_next_component_null() {
        let result = unsafe { rs_path_next_component(std::ptr::null()) };
        assert!(result.is_null());
    }
}

#[cfg(test)]
mod path_tail_with_sep_tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_path_tail_with_sep_basic() {
        let path = CString::new("/home/user/file.txt").unwrap();
        let result = unsafe { rs_path_tail_with_sep(path.as_ptr()) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        // Should point to "/file.txt"
        assert_eq!(result_str.to_str().unwrap(), "/file.txt");
    }

    #[test]
    fn test_path_tail_with_sep_no_sep() {
        let path = CString::new("file.txt").unwrap();
        let result = unsafe { rs_path_tail_with_sep(path.as_ptr()) };
        // Should return original pointer
        assert_eq!(result, path.as_ptr());
    }

    #[test]
    fn test_path_tail_with_sep_null() {
        let result = unsafe { rs_path_tail_with_sep(std::ptr::null()) };
        assert!(result.is_null());
    }
}

// ============================================================================
// invocation_path_tail - Find executable in an invocation string
// ============================================================================

/// Finds the path tail (or executable) in an invocation.
///
/// Given a program invocation in the form "path/to/exe [args]", returns
/// a pointer to the executable name (after the last path separator, before args).
///
/// # Arguments
/// * `invocation` - A program invocation string
/// * `len` - Optional output parameter for the length of the executable name
///
/// # Returns
/// The position of the last path separator + 1.
///
/// # Safety
/// - `invocation` must be a valid null-terminated C string.
/// - `len` may be null, otherwise must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_invocation_path_tail(
    invocation: *const c_char,
    len: *mut usize,
) -> *const c_char {
    if invocation.is_null() {
        if !len.is_null() {
            *len = 0;
        }
        return invocation;
    }

    let mut tail = rs_get_past_head(invocation);
    let mut p = tail;

    while *p != 0 && *p != b' ' as c_char {
        let was_sep = rs_vim_ispathsep_nocolon(*p as c_int) != 0;
        // MB_PTR_ADV: advance by UTF-8 character length
        let slice = std::slice::from_raw_parts(p as *const u8, 8);
        let char_len = nvim_mbyte::utfc_ptr2len(slice);
        p = p.add(char_len);
        if was_sep {
            tail = p; // Now tail points one past the separator.
        }
    }

    if !len.is_null() {
        *len = (p as usize) - (tail as usize);
    }

    tail
}

// ============================================================================
// path_has_wildcard - Check for wildcard characters in path
// ============================================================================

/// Checks if a path has a wildcard character including '~', unless at the end.
///
/// Returns true if the path contains wildcard characters:
/// - Unix: *?[{`'$
/// - Windows: ?*$[`
///
/// Also returns true if '~' is found and not at the end.
///
/// # Safety
/// - `p` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_path_has_wildcard(p: *const c_char) -> c_int {
    #[cfg(unix)]
    const WILDCARDS: &[u8] = b"*?[{`'$\0";
    #[cfg(not(unix))]
    const WILDCARDS: &[u8] = b"?*$[`\0";

    if p.is_null() {
        return 0;
    }

    let mut ptr = p;
    while *ptr != 0 {
        #[cfg(unix)]
        {
            // On Unix, backslash escapes the next character
            if *ptr == b'\\' as c_char && *ptr.add(1) != 0 {
                ptr = ptr.add(1);
                // Skip past escaped char
                let slice = std::slice::from_raw_parts(ptr as *const u8, 8);
                let char_len = nvim_mbyte::utfc_ptr2len(slice);
                ptr = ptr.add(char_len);
                continue;
            }
        }

        let c = *ptr as u8;
        // Check if character is in wildcards list
        if WILDCARDS[..WILDCARDS.len() - 1].contains(&c) {
            return 1;
        }
        // Check for ~ not at end
        if c == b'~' && *ptr.add(1) != 0 {
            return 1;
        }

        // MB_PTR_ADV
        let slice = std::slice::from_raw_parts(ptr as *const u8, 8);
        let char_len = nvim_mbyte::utfc_ptr2len(slice);
        ptr = ptr.add(char_len);
    }

    0
}

// ============================================================================
// path_has_exp_wildcard - Check for expandable wildcard characters
// ============================================================================

/// Checks if a path has a character path_expand can expand.
///
/// Returns true if the path contains:
/// - Unix: *?[{
/// - Windows: *?[
///
/// # Safety
/// - `p` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_path_has_exp_wildcard(p: *const c_char) -> c_int {
    #[cfg(unix)]
    const WILDCARDS: &[u8] = b"*?[{\0";
    #[cfg(not(unix))]
    const WILDCARDS: &[u8] = b"*?[\0";

    if p.is_null() {
        return 0;
    }

    let mut ptr = p;
    while *ptr != 0 {
        #[cfg(unix)]
        {
            // On Unix, backslash escapes the next character
            if *ptr == b'\\' as c_char && *ptr.add(1) != 0 {
                ptr = ptr.add(1);
                // Skip past escaped char
                let slice = std::slice::from_raw_parts(ptr as *const u8, 8);
                let char_len = nvim_mbyte::utfc_ptr2len(slice);
                ptr = ptr.add(char_len);
                continue;
            }
        }

        let c = *ptr as u8;
        // Check if character is in wildcards list
        if WILDCARDS[..WILDCARDS.len() - 1].contains(&c) {
            return 1;
        }

        // MB_PTR_ADV
        let slice = std::slice::from_raw_parts(ptr as *const u8, 8);
        let char_len = nvim_mbyte::utfc_ptr2len(slice);
        ptr = ptr.add(char_len);
    }

    0
}

// ============================================================================
// pathcmp - Compare paths with separator awareness
// ============================================================================

/// Compare path "p" to "q".
///
/// If `maxlen` >= 0, compare at most `maxlen` bytes.
/// Return value like strcmp(p, q), but consider path separators.
///
/// Key behaviors:
/// - Path separators are considered "less than" any other character
/// - On Windows, '/' and '\\' are treated as equivalent
/// - If `fic` is true, comparison is case-insensitive using utf_fold
/// - A trailing slash is ignored (unless it would be "//" or ":/")
///
/// # Safety
/// - `p` and `q` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_pathcmp(
    path1: *const c_char,
    path2: *const c_char,
    maxlen: c_int,
) -> c_int {
    if path1.is_null() || path2.is_null() {
        if path1.is_null() && path2.is_null() {
            return 0;
        }
        return if path1.is_null() { -1 } else { 1 };
    }

    let fic = nvim_get_p_fic() != 0;
    let mut idx1: usize = 0;
    let mut idx2: usize = 0;
    let mut shorter_path: *const c_char = std::ptr::null();
    let mut shorter_offset: usize = 0;

    loop {
        // Check maxlen condition
        if maxlen >= 0 && (idx1 >= maxlen as usize || idx2 >= maxlen as usize) {
            break;
        }

        let slice1 = std::slice::from_raw_parts(path1.add(idx1) as *const u8, 8);
        let slice2 = std::slice::from_raw_parts(path2.add(idx2) as *const u8, 8);
        let char1 = nvim_mbyte::utf_ptr2char(slice1);
        let char2 = nvim_mbyte::utf_ptr2char(slice2);

        // End of path1: check if path2 also ends or just has a slash
        if char1 == 0 {
            if char2 == 0 {
                // full match
                return 0;
            }
            shorter_path = path2;
            shorter_offset = idx2;
            break;
        }

        // End of path2: check if path1 just has a slash
        if char2 == 0 {
            shorter_path = path1;
            shorter_offset = idx1;
            break;
        }

        // Check for character match, considering case and path separators
        #[cfg(windows)]
        let both_pathsep = (char1 == b'/' as i32 || char1 == b'\\' as i32)
            && (char2 == b'/' as i32 || char2 == b'\\' as i32);
        #[cfg(not(windows))]
        let both_pathsep = false;

        let chars_match = if fic {
            char1 == char2 || nvim_mbyte::utf_fold(char1) == nvim_mbyte::utf_fold(char2)
        } else {
            char1 == char2
        };

        if !chars_match && !both_pathsep {
            // Characters don't match
            if rs_vim_ispathsep(char1) != 0 {
                return -1;
            }
            if rs_vim_ispathsep(char2) != 0 {
                return 1;
            }
            return if fic {
                nvim_mbyte::utf_fold(char1) - nvim_mbyte::utf_fold(char2)
            } else {
                char1 - char2
            };
        }

        // Advance pointers
        idx1 += nvim_mbyte::utfc_ptr2len(slice1);
        idx2 += nvim_mbyte::utfc_ptr2len(slice2);
    }

    // idx1 or idx2 ran into "maxlen" without finding a difference
    if shorter_path.is_null() {
        return 0;
    }

    // Check for trailing slash case
    let slice_s = std::slice::from_raw_parts(shorter_path.add(shorter_offset) as *const u8, 8);
    let trail_char = nvim_mbyte::utf_ptr2char(slice_s);
    let char_len = nvim_mbyte::utfc_ptr2len(slice_s);
    let slice_s2 =
        std::slice::from_raw_parts(shorter_path.add(shorter_offset + char_len) as *const u8, 8);
    let next_char = nvim_mbyte::utf_ptr2char(slice_s2);

    // Ignore a trailing slash, but not "//" or ":/"
    #[cfg(windows)]
    let is_trailing_sep = trail_char == b'/' as i32 || trail_char == b'\\' as i32;
    #[cfg(not(windows))]
    let is_trailing_sep = trail_char == b'/' as i32;

    if next_char == 0
        && shorter_offset > 0
        && rs_after_pathsep(shorter_path, shorter_path.add(shorter_offset)) == 0
        && is_trailing_sep
    {
        return 0; // match with trailing slash
    }

    if shorter_path == path2 {
        -1 // no match
    } else {
        1
    }
}

#[cfg(test)]
mod invocation_path_tail_tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_invocation_path_tail_basic() {
        let inv = CString::new("/usr/bin/nvim --version").unwrap();
        let mut len: usize = 0;
        let result = unsafe { rs_invocation_path_tail(inv.as_ptr(), std::ptr::addr_of_mut!(len)) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        assert_eq!(result_str.to_str().unwrap(), "nvim --version");
        assert_eq!(len, 4); // "nvim"
    }

    #[test]
    fn test_invocation_path_tail_no_args() {
        let inv = CString::new("/path/to/program").unwrap();
        let mut len: usize = 0;
        let result = unsafe { rs_invocation_path_tail(inv.as_ptr(), std::ptr::addr_of_mut!(len)) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        assert_eq!(result_str.to_str().unwrap(), "program");
        assert_eq!(len, 7); // "program"
    }

    #[test]
    fn test_invocation_path_tail_null_len() {
        let inv = CString::new("/bin/ls").unwrap();
        let result = unsafe { rs_invocation_path_tail(inv.as_ptr(), std::ptr::null_mut()) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        assert_eq!(result_str.to_str().unwrap(), "ls");
    }
}

#[cfg(test)]
mod wildcard_tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_path_has_wildcard_star() {
        let path = CString::new("*.txt").unwrap();
        assert_eq!(unsafe { rs_path_has_wildcard(path.as_ptr()) }, 1);
    }

    #[test]
    fn test_path_has_wildcard_question() {
        let path = CString::new("file?.txt").unwrap();
        assert_eq!(unsafe { rs_path_has_wildcard(path.as_ptr()) }, 1);
    }

    #[test]
    fn test_path_has_wildcard_bracket() {
        let path = CString::new("file[0-9].txt").unwrap();
        assert_eq!(unsafe { rs_path_has_wildcard(path.as_ptr()) }, 1);
    }

    #[test]
    fn test_path_has_wildcard_tilde_not_at_end() {
        let path = CString::new("~/documents").unwrap();
        assert_eq!(unsafe { rs_path_has_wildcard(path.as_ptr()) }, 1);
    }

    #[test]
    fn test_path_has_wildcard_tilde_at_end() {
        let path = CString::new("backup~").unwrap();
        // Tilde at end should NOT trigger wildcard
        assert_eq!(unsafe { rs_path_has_wildcard(path.as_ptr()) }, 0);
    }

    #[test]
    fn test_path_has_wildcard_none() {
        let path = CString::new("/home/user/file.txt").unwrap();
        assert_eq!(unsafe { rs_path_has_wildcard(path.as_ptr()) }, 0);
    }

    #[test]
    fn test_path_has_exp_wildcard_star() {
        let path = CString::new("*.txt").unwrap();
        assert_eq!(unsafe { rs_path_has_exp_wildcard(path.as_ptr()) }, 1);
    }

    #[test]
    fn test_path_has_exp_wildcard_none() {
        let path = CString::new("/home/user/file.txt").unwrap();
        assert_eq!(unsafe { rs_path_has_exp_wildcard(path.as_ptr()) }, 0);
    }

    #[test]
    #[cfg(unix)]
    fn test_path_has_exp_wildcard_brace() {
        // Brace is only a wildcard on Unix
        let path = CString::new("file{a,b}.txt").unwrap();
        assert_eq!(unsafe { rs_path_has_exp_wildcard(path.as_ptr()) }, 1);
    }
}

#[cfg(test)]
mod gettail_dir_tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_gettail_dir_basic() {
        let path = CString::new("/path/file").unwrap();
        let result = unsafe { rs_gettail_dir(path.as_ptr()) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        // Should point to "/file" (the '/' before "file")
        assert_eq!(result_str.to_str().unwrap(), "/file");
    }

    #[test]
    fn test_gettail_dir_trailing_slash() {
        let path = CString::new("/path/dir/").unwrap();
        let result = unsafe { rs_gettail_dir(path.as_ptr()) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        // Should point to "/dir/" (the '/' before "dir/")
        assert_eq!(result_str.to_str().unwrap(), "/dir/");
    }

    #[test]
    fn test_gettail_dir_double_slash() {
        let path = CString::new("/path//dir").unwrap();
        let result = unsafe { rs_gettail_dir(path.as_ptr()) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        // Should point to "//dir" (first '/' of the double slash)
        assert_eq!(result_str.to_str().unwrap(), "//dir");
    }

    #[test]
    fn test_gettail_dir_just_file() {
        let path = CString::new("/file").unwrap();
        let result = unsafe { rs_gettail_dir(path.as_ptr()) };
        // Should point to start (fname)
        assert_eq!(result, path.as_ptr());
    }

    #[test]
    fn test_gettail_dir_no_slash() {
        let path = CString::new("file.txt").unwrap();
        let result = unsafe { rs_gettail_dir(path.as_ptr()) };
        // Should point to start (fname)
        assert_eq!(result, path.as_ptr());
    }

    #[test]
    fn test_gettail_dir_null() {
        let result = unsafe { rs_gettail_dir(std::ptr::null()) };
        assert!(result.is_null());
    }
}

#[cfg(test)]
mod fnamecmp_tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    #[cfg(unix)]
    fn test_path_fnamecmp_basic() {
        unsafe {
            let f1 = CString::new("/home/user/file.txt").unwrap();
            let f2 = CString::new("/home/user/file.txt").unwrap();
            // Same file - should be equal (0)
            // Note: actual result depends on p_fic which we can't control in unit tests
            let result = rs_path_fnamecmp(f1.as_ptr(), f2.as_ptr());
            assert_eq!(result, 0);

            // Different files
            let f3 = CString::new("/home/user/other.txt").unwrap();
            let result = rs_path_fnamecmp(f1.as_ptr(), f3.as_ptr());
            assert_ne!(result, 0);
        }
    }

    #[test]
    fn test_path_fnamecmp_null() {
        unsafe {
            let f1 = CString::new("test.txt").unwrap();

            // Both null - equal
            assert_eq!(rs_path_fnamecmp(std::ptr::null(), std::ptr::null()), 0);

            // One null - not equal
            assert_ne!(rs_path_fnamecmp(f1.as_ptr(), std::ptr::null()), 0);
            assert_ne!(rs_path_fnamecmp(std::ptr::null(), f1.as_ptr()), 0);
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_path_fnamencmp_basic() {
        unsafe {
            let f1 = CString::new("file.txt").unwrap();
            let f2 = CString::new("file.txt").unwrap();
            let f3 = CString::new("file.txT").unwrap();

            // Same - should be equal
            assert_eq!(rs_path_fnamencmp(f1.as_ptr(), f2.as_ptr(), 8), 0);

            // Compare only first 4 chars "file" - should be equal
            assert_eq!(rs_path_fnamencmp(f1.as_ptr(), f3.as_ptr(), 4), 0);
        }
    }
}
