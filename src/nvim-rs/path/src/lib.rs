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

use std::ffi::{c_char, c_int, c_void};

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

// ============================================================================
// add_pathsep - Add path separator to filename
// ============================================================================

/// Maximum path length (matches MAXPATHL in C code)
const MAXPATHL: usize = 4096;

/// Adds a path separator to a filename, unless it already ends in one.
///
/// # Safety
/// - `p` must be a valid pointer to a mutable, null-terminated C string
/// - The buffer pointed to by `p` must have enough space for an additional
///   path separator character (at most 2 extra bytes for separator + NUL)
///
/// # Returns
/// - 1 (true) if the path separator was added or already existed.
/// - 0 (false) if the filename is too long.
#[no_mangle]
pub unsafe extern "C" fn rs_add_pathsep(p: *mut c_char) -> c_int {
    if p.is_null() {
        return 0;
    }

    // Get the length of the string
    let len = libc::strlen(p);
    if len == 0 {
        return 1; // Empty string - nothing to do (matches C behavior)
    }

    // Check if already ends with a path separator
    if rs_after_pathsep(p, p.add(len)) != 0 {
        return 1; // Already has separator
    }

    // Check if there's enough space (need room for separator + NUL)
    // pathsep_len is 1 on Unix (just '/'), could be different on Windows
    let pathsep_len = 1;
    if len > MAXPATHL - pathsep_len - 1 {
        return 0; // No space for trailing slash
    }

    // Add the path separator
    #[cfg(unix)]
    {
        *p.add(len) = b'/' as c_char;
    }
    #[cfg(windows)]
    {
        *p.add(len) = b'\\' as c_char;
    }
    #[cfg(not(any(unix, windows)))]
    {
        *p.add(len) = b'/' as c_char;
    }

    // Add NUL terminator
    *p.add(len + 1) = 0;

    1
}

// ============================================================================
// append_path - Append path component with separator
// ============================================================================

/// Append to_append to path with a slash in between.
///
/// Does not append empty string or a single ".".
///
/// # Safety
/// - `path` must be a valid pointer to a mutable, null-terminated C string
/// - `to_append` must be a valid pointer to a null-terminated C string
/// - The buffer pointed to by `path` must have at least `max_len` bytes capacity
///
/// # Returns
/// - 1 (OK) on success
/// - 0 (FAIL) if not enough space
#[no_mangle]
pub unsafe extern "C" fn rs_append_path(
    path: *mut c_char,
    to_append: *const c_char,
    max_len: usize,
) -> c_int {
    const OK: c_int = 1;
    const FAIL: c_int = 0;

    if path.is_null() || to_append.is_null() {
        return FAIL;
    }

    let current_length = libc::strlen(path);
    let to_append_length = libc::strlen(to_append);

    // Do not append empty string or a dot.
    if to_append_length == 0 {
        return OK;
    }
    // Check for "." - single dot
    if to_append_length == 1 && *to_append == b'.' as c_char {
        return OK;
    }

    let mut write_pos = current_length;

    // Combine the path segments, separated by a slash
    if current_length > 0 && rs_vim_ispathsep_nocolon(*path.add(current_length - 1) as c_int) == 0 {
        // Need to add a separator
        // +1 for the separator, +1 for the NUL at the end
        if current_length + 1 + 1 > max_len {
            return FAIL; // No space for trailing slash
        }
        #[cfg(unix)]
        {
            *path.add(write_pos) = b'/' as c_char;
        }
        #[cfg(windows)]
        {
            *path.add(write_pos) = b'\\' as c_char;
        }
        #[cfg(not(any(unix, windows)))]
        {
            *path.add(write_pos) = b'/' as c_char;
        }
        write_pos += 1;
    }

    // +1 for the NUL at the end
    if write_pos + to_append_length + 1 > max_len {
        return FAIL;
    }

    // Copy to_append (including NUL terminator)
    libc::memcpy(
        path.add(write_pos) as *mut libc::c_void,
        to_append as *const libc::c_void,
        to_append_length + 1,
    );

    OK
}

// ============================================================================
// path_with_extension - Check if path has given extension
// ============================================================================

/// Check if a path ends with a specific extension.
///
/// Comparison respects 'fileignorecase' option.
///
/// # Safety
/// - `path` and `extension` must be valid null-terminated C strings.
///
/// # Returns
/// - 1 (true) if path ends with the given extension
/// - 0 (false) otherwise
#[no_mangle]
pub unsafe extern "C" fn rs_path_with_extension(
    path: *const c_char,
    extension: *const c_char,
) -> c_int {
    if path.is_null() || extension.is_null() {
        return 0;
    }

    // Find the last '.' in path
    let last_dot = libc::strrchr(path, b'.' as c_int);
    if last_dot.is_null() {
        return 0;
    }

    // Compare extension (skip the dot)
    let fic = nvim_get_p_fic() != 0;
    let ext_start = last_dot.add(1);
    c_int::from(nvim_mbyte::rs_mb_strcmp_ic(fic, ext_start, extension) == 0)
}

// ============================================================================
// path_shorten_fname - Make absolute path relative to directory
// ============================================================================

/// Try to find a shortname by comparing the fullname with `dir_name`.
///
/// # Safety
/// - `full_path` may be NULL
/// - `dir_name` must be a valid null-terminated C string (not NULL)
///
/// # Returns
/// - Pointer into `full_path` if shortened.
/// - NULL if no shorter name is possible or if `full_path` is NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_path_shorten_fname(
    full_path: *mut c_char,
    dir_name: *mut c_char,
) -> *mut c_char {
    if full_path.is_null() {
        return std::ptr::null_mut();
    }

    // dir_name should not be NULL (asserted in C code)
    if dir_name.is_null() {
        return std::ptr::null_mut();
    }

    let len = libc::strlen(dir_name);

    // If full_path and dir_name do not match, it's impossible to make one
    // relative to the other.
    if rs_path_fnamencmp(dir_name, full_path, len) != 0 {
        return std::ptr::null_mut();
    }

    // If dir_name is a path head, full_path can always be made relative.
    if len == rs_path_head_length() as usize && rs_is_path_head(dir_name) != 0 {
        return full_path.add(len);
    }

    let p = full_path.add(len);

    // If *p is not pointing to a path separator, this means that full_path's
    // last directory name is longer than *dir_name's last directory, so they
    // don't actually match.
    if rs_vim_ispathsep(*p as c_int) == 0 {
        return std::ptr::null_mut();
    }

    p.add(1)
}

// ============================================================================
// shorten_dir_len / shorten_dir - Shorten directory components in path
// ============================================================================

/// Shorten the path of a file from "~/foo/../.bar/fname" to "~/f/../.b/fname"
/// "trim_len" specifies how many characters to keep for each directory.
/// Must be 1 or more.
/// It's done in-place.
///
/// # Safety
/// - `str` must be a valid pointer to a mutable, null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_shorten_dir_len(str: *mut c_char, trim_len: c_int) {
    if str.is_null() {
        return;
    }

    let tail = rs_path_tail(str);
    let mut d = str;
    let mut skip = false;
    let mut dirchunk_len: c_int = 0;
    let mut s = str;

    loop {
        if s >= tail.cast_mut() {
            // Copy the whole tail
            *d = *s;
            d = d.add(1);
            if *s == 0 {
                break;
            }
        } else if rs_vim_ispathsep(*s as c_int) != 0 {
            // Copy '/' and next char
            *d = *s;
            d = d.add(1);
            skip = false;
            dirchunk_len = 0;
        } else if !skip {
            // Copy next char
            *d = *s;
            d = d.add(1);
            // Don't count leading "~" and "." for the directory chunk length
            if *s as u8 != b'~' && *s as u8 != b'.' {
                dirchunk_len += 1;
                // Keep copying chars until we have our preferred length
                if dirchunk_len >= trim_len {
                    skip = true;
                }
            }
            // Handle multi-byte characters
            let slice = std::slice::from_raw_parts(s as *const u8, 8);
            let mut l = nvim_mbyte::utfc_ptr2len(slice) as isize;
            l -= 1;
            while l > 0 {
                s = s.offset(1);
                *d = *s;
                d = d.add(1);
                l -= 1;
            }
        }
        s = s.add(1);
    }
}

/// Shorten the path of a file from "~/foo/../.bar/fname" to "~/f/../.b/fname"
/// It's done in-place.
///
/// # Safety
/// - `str` must be a valid pointer to a mutable, null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_shorten_dir(str: *mut c_char) {
    rs_shorten_dir_len(str, 1);
}

#[cfg(test)]
mod add_pathsep_tests {
    use super::*;

    #[test]
    fn test_add_pathsep_no_sep() {
        let mut buf = [0i8; 100];
        let path = b"/home/user\0";
        for (i, &b) in path.iter().enumerate() {
            buf[i] = b as i8;
        }
        let result = unsafe { rs_add_pathsep(buf.as_mut_ptr()) };
        assert_eq!(result, 1);
        let result_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        assert_eq!(result_str.to_str().unwrap(), "/home/user/");
    }

    #[test]
    fn test_add_pathsep_already_has_sep() {
        let mut buf = [0i8; 100];
        let path = b"/home/user/\0";
        for (i, &b) in path.iter().enumerate() {
            buf[i] = b as i8;
        }
        let result = unsafe { rs_add_pathsep(buf.as_mut_ptr()) };
        assert_eq!(result, 1);
        let result_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        assert_eq!(result_str.to_str().unwrap(), "/home/user/");
    }

    #[test]
    fn test_add_pathsep_empty() {
        let mut buf = [0i8; 100];
        buf[0] = 0;
        let result = unsafe { rs_add_pathsep(buf.as_mut_ptr()) };
        assert_eq!(result, 1);
    }
}

#[cfg(test)]
mod append_path_tests {
    use super::*;

    #[test]
    fn test_append_path_basic() {
        let mut buf = [0i8; 100];
        let path = b"/home\0";
        for (i, &b) in path.iter().enumerate() {
            buf[i] = b as i8;
        }
        let to_append = std::ffi::CString::new("user").unwrap();
        let result = unsafe { rs_append_path(buf.as_mut_ptr(), to_append.as_ptr(), 100) };
        assert_eq!(result, 1); // OK
        let result_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        assert_eq!(result_str.to_str().unwrap(), "/home/user");
    }

    #[test]
    fn test_append_path_already_has_sep() {
        let mut buf = [0i8; 100];
        let path = b"/home/\0";
        for (i, &b) in path.iter().enumerate() {
            buf[i] = b as i8;
        }
        let to_append = std::ffi::CString::new("user").unwrap();
        let result = unsafe { rs_append_path(buf.as_mut_ptr(), to_append.as_ptr(), 100) };
        assert_eq!(result, 1); // OK
        let result_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        assert_eq!(result_str.to_str().unwrap(), "/home/user");
    }

    #[test]
    fn test_append_path_empty() {
        let mut buf = [0i8; 100];
        let path = b"/home\0";
        for (i, &b) in path.iter().enumerate() {
            buf[i] = b as i8;
        }
        let to_append = std::ffi::CString::new("").unwrap();
        let result = unsafe { rs_append_path(buf.as_mut_ptr(), to_append.as_ptr(), 100) };
        assert_eq!(result, 1); // OK - does nothing
        let result_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        assert_eq!(result_str.to_str().unwrap(), "/home");
    }

    #[test]
    fn test_append_path_dot() {
        let mut buf = [0i8; 100];
        let path = b"/home\0";
        for (i, &b) in path.iter().enumerate() {
            buf[i] = b as i8;
        }
        let to_append = std::ffi::CString::new(".").unwrap();
        let result = unsafe { rs_append_path(buf.as_mut_ptr(), to_append.as_ptr(), 100) };
        assert_eq!(result, 1); // OK - does nothing
        let result_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        assert_eq!(result_str.to_str().unwrap(), "/home");
    }

    #[test]
    fn test_append_path_no_space() {
        let mut buf = [0i8; 10];
        let path = b"/home\0";
        for (i, &b) in path.iter().enumerate() {
            buf[i] = b as i8;
        }
        let to_append = std::ffi::CString::new("very_long_name").unwrap();
        let result = unsafe { rs_append_path(buf.as_mut_ptr(), to_append.as_ptr(), 10) };
        assert_eq!(result, 0); // FAIL
    }
}

#[cfg(test)]
mod shorten_dir_tests {
    use super::*;

    #[test]
    fn test_shorten_dir_basic() {
        let mut buf = [0i8; 100];
        let path = b"/home/user/file.txt\0";
        for (i, &b) in path.iter().enumerate() {
            buf[i] = b as i8;
        }
        unsafe { rs_shorten_dir(buf.as_mut_ptr()) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        assert_eq!(result_str.to_str().unwrap(), "/h/u/file.txt");
    }

    #[test]
    fn test_shorten_dir_with_dot() {
        let mut buf = [0i8; 100];
        let path = b"/home/.config/file.txt\0";
        for (i, &b) in path.iter().enumerate() {
            buf[i] = b as i8;
        }
        unsafe { rs_shorten_dir(buf.as_mut_ptr()) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        // Leading dots are preserved
        assert_eq!(result_str.to_str().unwrap(), "/h/.c/file.txt");
    }

    #[test]
    fn test_shorten_dir_with_tilde() {
        let mut buf = [0i8; 100];
        let path = b"~/foo/file.txt\0";
        for (i, &b) in path.iter().enumerate() {
            buf[i] = b as i8;
        }
        unsafe { rs_shorten_dir(buf.as_mut_ptr()) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        // Tilde is preserved
        assert_eq!(result_str.to_str().unwrap(), "~/f/file.txt");
    }

    #[test]
    fn test_shorten_dir_len_2() {
        let mut buf = [0i8; 100];
        let path = b"/home/user/file.txt\0";
        for (i, &b) in path.iter().enumerate() {
            buf[i] = b as i8;
        }
        unsafe { rs_shorten_dir_len(buf.as_mut_ptr(), 2) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        assert_eq!(result_str.to_str().unwrap(), "/ho/us/file.txt");
    }

    #[test]
    fn test_shorten_dir_dotdot() {
        let mut buf = [0i8; 100];
        let path = b"/home/../bar/file.txt\0";
        for (i, &b) in path.iter().enumerate() {
            buf[i] = b as i8;
        }
        unsafe { rs_shorten_dir(buf.as_mut_ptr()) };
        let result_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        // ".." is preserved (dots are not counted)
        assert_eq!(result_str.to_str().unwrap(), "/h/../b/file.txt");
    }
}

// ============================================================================
// Phase 1: Constants and Simple Functions
// ============================================================================

// FileComparison enum values (matches C path.h)
// Used in later migration phases.
#[allow(dead_code)]
const K_EQUAL_FILES: c_int = 1;
#[allow(dead_code)]
const K_DIFFERENT_FILES: c_int = 2;
#[allow(dead_code)]
const K_BOTH_FILES_MISSING: c_int = 4;
#[allow(dead_code)]
const K_ONE_FILE_MISSING: c_int = 6;
#[allow(dead_code)]
const K_EQUAL_FILE_NAMES: c_int = 7;

// EW_* flags for expand_wildcards (matches C path.h)
// Used in later migration phases.
#[allow(dead_code)]
const EW_DIR: c_int = 0x01;
#[allow(dead_code)]
const EW_FILE: c_int = 0x02;
#[allow(dead_code)]
const EW_NOTFOUND: c_int = 0x04;
#[allow(dead_code)]
const EW_ADDSLASH: c_int = 0x08;
#[allow(dead_code)]
const EW_KEEPALL: c_int = 0x10;
#[allow(dead_code)]
const EW_SILENT: c_int = 0x20;
#[allow(dead_code)]
const EW_EXEC: c_int = 0x40;
#[allow(dead_code)]
const EW_PATH: c_int = 0x80;
#[allow(dead_code)]
const EW_ICASE: c_int = 0x100;
#[allow(dead_code)]
const EW_NOERROR: c_int = 0x200;
#[allow(dead_code)]
const EW_NOTWILD: c_int = 0x400;
#[allow(dead_code)]
const EW_KEEPDOLLAR: c_int = 0x800;
#[allow(dead_code)]
const EW_ALLLINKS: c_int = 0x1000;
#[allow(dead_code)]
const EW_SHELLCMD: c_int = 0x2000;
#[allow(dead_code)]
const EW_DODOT: c_int = 0x4000;
#[allow(dead_code)]
const EW_EMPTYOK: c_int = 0x8000;
#[allow(dead_code)]
const EW_NOTENV: c_int = 0x10000;
#[allow(dead_code)]
const EW_CDPATH: c_int = 0x20000;
#[allow(dead_code)]
const EW_NOBREAK: c_int = 0x40000;

#[allow(dead_code)]
const OK: c_int = 1;
#[allow(dead_code)]
const FAIL: c_int = 0;

/// Unix special wildcard characters that need shell expansion.
#[cfg(unix)]
#[allow(dead_code)]
const SPECIAL_WILDCHAR: &[u8] = b"`'{\"";

extern "C" {
    fn nvim_path_os_isdir(name: *const c_char) -> c_int;
    fn nvim_path_xmalloc(size: usize) -> *mut c_char;
    fn nvim_path_xrealloc(ptr: *mut c_char, size: usize) -> *mut c_char;
    fn nvim_path_xfree(ptr: *mut c_char);
}

/// qsort comparator for path strings — dereferences two `char **` pointers
/// and calls `rs_pathcmp` with maxlen=-1 (compare full strings).
///
/// # Safety
/// `a` and `b` must be valid pointers to `*const c_char`.
#[no_mangle]
pub unsafe extern "C" fn rs_pstrcmp(a: *const c_void, b: *const c_void) -> c_int {
    let pa = *(a as *const *const c_char);
    let pb = *(b as *const *const c_char);
    rs_pathcmp(pa, pb, -1)
}

/// Return true if we can expand this backtick thing here.
///
/// Checks that `p` starts and ends with '`' and has at least one char between.
///
/// # Safety
/// `p` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_backtick(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }
    let first = *p as u8;
    if first != b'`' {
        return 0;
    }
    // Check second char exists
    if *p.add(1) == 0 {
        return 0;
    }
    // Find end
    let len = libc::strlen(p);
    c_int::from(*(p.add(len - 1)) as u8 == b'`')
}

/// Return true if "p" contains what looks like an environment variable.
/// Allowing for escaping.
///
/// # Safety
/// `p` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_has_env_var(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }
    let mut ptr = p;
    while *ptr != 0 {
        if *ptr as u8 == b'\\' && *ptr.add(1) != 0 {
            // Skip escaped character
            ptr = ptr.add(1);
            let slice = std::slice::from_raw_parts(ptr as *const u8, 8);
            let char_len = nvim_mbyte::utfc_ptr2len(slice);
            ptr = ptr.add(char_len);
            continue;
        }
        if *ptr as u8 == b'$' {
            return 1;
        }
        // MB_PTR_ADV
        let slice = std::slice::from_raw_parts(ptr as *const u8, 8);
        let char_len = nvim_mbyte::utfc_ptr2len(slice);
        ptr = ptr.add(char_len);
    }
    0
}

/// Free the list of files returned by expand_wildcards() or other expansion functions.
///
/// # Safety
/// `files` must be a valid pointer to an array of `count` allocated C strings,
/// or NULL. Each string and the array itself are freed.
#[no_mangle]
pub unsafe extern "C" fn rs_FreeWild(count: c_int, files: *mut *mut c_char) {
    if count <= 0 || files.is_null() {
        return;
    }
    let mut i = count;
    while i > 0 {
        i -= 1;
        nvim_path_xfree(*files.add(i as usize));
    }
    nvim_path_xfree(files as *mut c_char);
}

/// Return true if the directory of "fname" exists, false otherwise.
/// Also returns true if there is no directory name.
/// "fname" must be writable!
///
/// # Safety
/// `fname` must be a valid pointer to a mutable, null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_dir_of_file_exists(fname: *mut c_char) -> c_int {
    if fname.is_null() {
        return 0;
    }
    let p = rs_path_tail_with_sep(fname).cast_mut();
    if p == fname {
        return 1; // no directory name
    }
    let saved = *p;
    *p = 0; // NUL-terminate at separator
    let retval = nvim_path_os_isdir(fname);
    *p = saved;
    retval
}

/// Concatenate file names in-place.
///
/// Appends `fname2` (len2 bytes) to `fname1` at offset `len1`.
/// If `sep` is true and fname1 doesn't end with a path separator, inserts one.
///
/// # Safety
/// `fname1` buffer must have enough space for len1 + len2 + 2 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_do_concat_fnames(
    fname1: *mut c_char,
    len1: usize,
    fname2: *const c_char,
    len2: usize,
    sep: c_int,
) -> *mut c_char {
    if fname1.is_null() || fname2.is_null() {
        return fname1;
    }
    if sep != 0 && *fname1 != 0 && rs_after_pathsep(fname1, fname1.add(len1)) == 0 {
        // Insert path separator
        #[cfg(unix)]
        {
            *fname1.add(len1) = b'/' as c_char;
        }
        #[cfg(windows)]
        {
            *fname1.add(len1) = b'\\' as c_char;
        }
        #[cfg(not(any(unix, windows)))]
        {
            *fname1.add(len1) = b'/' as c_char;
        }
        std::ptr::copy(
            fname2 as *const u8,
            fname1.add(len1 + 1) as *mut u8,
            len2 + 1,
        );
    } else {
        std::ptr::copy(fname2 as *const u8, fname1.add(len1) as *mut u8, len2 + 1);
    }
    fname1
}

/// Concatenate file names fname1 and fname2 into allocated memory.
///
/// Only add a '/' or '\\' when 'sep' is true and it is necessary.
///
/// # Safety
/// `fname1` and `fname2` must be valid null-terminated C strings.
/// Returns an allocated string that must be freed by the caller.
#[no_mangle]
pub unsafe extern "C" fn rs_concat_fnames(
    fname1: *const c_char,
    fname2: *const c_char,
    sep: c_int,
) -> *mut c_char {
    if fname1.is_null() || fname2.is_null() {
        return std::ptr::null_mut();
    }
    let len1 = libc::strlen(fname1);
    let len2 = libc::strlen(fname2);
    let dest = nvim_path_xmalloc(len1 + len2 + 3);
    std::ptr::copy_nonoverlapping(fname1 as *const u8, dest as *mut u8, len1 + 1);
    rs_do_concat_fnames(dest, len1, fname2, len2, sep)
}

/// Concatenate file names, reallocating fname1.
///
/// Like concat_fnames(), but in place of allocating new memory it reallocates
/// fname1.
///
/// # Safety
/// `fname1` must have been allocated with xmalloc. `fname2` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_concat_fnames_realloc(
    fname1: *mut c_char,
    fname2: *const c_char,
    sep: c_int,
) -> *mut c_char {
    if fname1.is_null() || fname2.is_null() {
        return fname1;
    }
    let len1 = libc::strlen(fname1);
    let len2 = libc::strlen(fname2);
    let dest = nvim_path_xrealloc(fname1, len1 + len2 + 3);
    rs_do_concat_fnames(dest, len1, fname2, len2, sep)
}

#[cfg(test)]
mod phase1_tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_pstrcmp() {
        unsafe {
            let s1 = CString::new("/a/b").unwrap();
            let s2 = CString::new("/a/c").unwrap();
            let s3 = CString::new("/a/b").unwrap();

            let p1 = s1.as_ptr();
            let p2 = s2.as_ptr();
            let p3 = s3.as_ptr();

            // Same strings should compare equal
            assert_eq!(
                rs_pstrcmp(
                    std::ptr::addr_of!(p1) as *const c_void,
                    std::ptr::addr_of!(p3) as *const c_void,
                ),
                0
            );
            // /a/b < /a/c
            assert!(
                rs_pstrcmp(
                    std::ptr::addr_of!(p1) as *const c_void,
                    std::ptr::addr_of!(p2) as *const c_void,
                ) < 0
            );
        }
    }

    #[test]
    fn test_vim_backtick() {
        unsafe {
            let bt = CString::new("`echo hello`").unwrap();
            assert_eq!(rs_vim_backtick(bt.as_ptr()), 1);

            let no_bt = CString::new("echo hello").unwrap();
            assert_eq!(rs_vim_backtick(no_bt.as_ptr()), 0);

            let single = CString::new("`").unwrap();
            assert_eq!(rs_vim_backtick(single.as_ptr()), 0);

            let empty = CString::new("``").unwrap();
            assert_eq!(rs_vim_backtick(empty.as_ptr()), 1);
        }
    }

    #[test]
    fn test_has_env_var() {
        unsafe {
            let with_var = CString::new("$HOME/file").unwrap();
            assert_eq!(rs_has_env_var(with_var.as_ptr()), 1);

            let no_var = CString::new("/home/user/file").unwrap();
            assert_eq!(rs_has_env_var(no_var.as_ptr()), 0);

            let escaped = CString::new("\\$HOME/file").unwrap();
            assert_eq!(rs_has_env_var(escaped.as_ptr()), 0);

            let escaped_then_var = CString::new("\\$HOME/$USER").unwrap();
            assert_eq!(rs_has_env_var(escaped_then_var.as_ptr()), 1);
        }
    }

    #[test]
    fn test_do_concat_fnames() {
        unsafe {
            // With separator
            let mut buf = [0i8; 100];
            let src = b"/home\0";
            for (i, &b) in src.iter().enumerate() {
                buf[i] = b as i8;
            }
            let fname2 = CString::new("user").unwrap();
            let result = rs_do_concat_fnames(buf.as_mut_ptr(), 5, fname2.as_ptr(), 4, 1);
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "/home/user");

            // Without separator (already has one)
            let mut buf2 = [0i8; 100];
            let src2 = b"/home/\0";
            for (i, &b) in src2.iter().enumerate() {
                buf2[i] = b as i8;
            }
            let result2 = rs_do_concat_fnames(buf2.as_mut_ptr(), 6, fname2.as_ptr(), 4, 1);
            let result_str2 = std::ffi::CStr::from_ptr(result2);
            assert_eq!(result_str2.to_str().unwrap(), "/home/user");

            // sep=false, no separator added
            let mut buf3 = [0i8; 100];
            let src3 = b"/home\0";
            for (i, &b) in src3.iter().enumerate() {
                buf3[i] = b as i8;
            }
            let result3 = rs_do_concat_fnames(buf3.as_mut_ptr(), 5, fname2.as_ptr(), 4, 0);
            let result_str3 = std::ffi::CStr::from_ptr(result3);
            assert_eq!(result_str3.to_str().unwrap(), "/homeuser");
        }
    }

    #[test]
    fn test_constants() {
        // Verify FileComparison values match path.h
        assert_eq!(K_EQUAL_FILES, 1);
        assert_eq!(K_DIFFERENT_FILES, 2);
        assert_eq!(K_BOTH_FILES_MISSING, 4);
        assert_eq!(K_ONE_FILE_MISSING, 6);
        assert_eq!(K_EQUAL_FILE_NAMES, 7);

        // Verify EW_* flags match path.h
        assert_eq!(EW_DIR, 0x01);
        assert_eq!(EW_FILE, 0x02);
        assert_eq!(EW_NOTFOUND, 0x04);
        assert_eq!(EW_ADDSLASH, 0x08);
        assert_eq!(EW_KEEPALL, 0x10);
        assert_eq!(EW_SILENT, 0x20);
        assert_eq!(EW_EXEC, 0x40);
        assert_eq!(EW_PATH, 0x80);
        assert_eq!(EW_ICASE, 0x100);
        assert_eq!(EW_NOERROR, 0x200);
        assert_eq!(EW_NOTWILD, 0x400);
        assert_eq!(EW_KEEPDOLLAR, 0x800);
        assert_eq!(EW_ALLLINKS, 0x1000);
        assert_eq!(EW_SHELLCMD, 0x2000);
        assert_eq!(EW_DODOT, 0x4000);
        assert_eq!(EW_EMPTYOK, 0x8000);
        assert_eq!(EW_NOTENV, 0x10000);
        assert_eq!(EW_CDPATH, 0x20000);
        assert_eq!(EW_NOBREAK, 0x40000);
    }

    #[test]
    fn test_ok_fail() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
    }

    #[cfg(unix)]
    #[test]
    fn test_special_wildchar() {
        assert_eq!(SPECIAL_WILDCHAR, b"`'{\"");
    }
}

// ============================================================================
// Phase 2: Path Resolution Chain
// ============================================================================

extern "C" {
    fn nvim_path_os_dirname(buf: *mut c_char, len: usize) -> c_int;
    fn nvim_path_os_realpath(name: *const c_char, buf: *mut c_char, len: usize) -> *mut c_char;
    fn nvim_path_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_path_xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;
}

/// Get the absolute name of the given relative directory.
///
/// # Safety
/// All pointers must be valid. `buffer` must have at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_path_full_dir_name(
    directory: *const c_char,
    buffer: *mut c_char,
    len: usize,
) -> c_int {
    if directory.is_null() || buffer.is_null() {
        return FAIL;
    }

    // Empty directory string -> get current directory
    if *directory == 0 {
        return nvim_path_os_dirname(buffer, len);
    }

    // Try realpath first
    if !nvim_path_os_realpath(directory, buffer, len).is_null() {
        return OK;
    }

    // Path does not exist (yet). For a full path fail.
    if rs_path_is_absolute(directory) != 0 {
        return FAIL;
    }

    // For a relative path use the current directory and append the file name.
    let mut old_dir = [0i8; MAXPATHL];
    if nvim_path_os_dirname(old_dir.as_mut_ptr(), MAXPATHL) == FAIL {
        return FAIL;
    }

    nvim_path_xstrlcpy(buffer, old_dir.as_ptr(), len);
    if rs_append_path(buffer, directory, len) == FAIL {
        return FAIL;
    }

    OK
}

/// Expand a filename to its full absolute path.
///
/// Used by `vim_FullName` and `fix_fname`.
///
/// # Safety
/// All pointers must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_path_to_absolute(
    fname: *const c_char,
    buf: *mut c_char,
    len: usize,
    force: c_int,
) -> c_int {
    if fname.is_null() || buf.is_null() {
        return FAIL;
    }

    *buf = 0;

    let relative_directory = nvim_path_xmalloc(len);
    let mut end_of_path = fname;

    // Expand if forced or not an absolute path
    if force != 0 || rs_path_is_absolute(fname) == 0 {
        let mut p = libc::strrchr(fname, b'/' as c_int);

        #[cfg(windows)]
        {
            if p.is_null() {
                p = libc::strrchr(fname, b'\\' as c_int);
            }
        }

        // Handle ".." without path separators
        if p.is_null() && libc::strcmp(fname, c"..".as_ptr()) == 0 {
            p = fname.add(2).cast_mut();
        }

        if p.is_null() {
            *relative_directory = 0;
        } else {
            // For "/path/dir/.." include the "/.."
            if rs_vim_ispathsep_nocolon(*p as c_int) != 0
                && libc::strcmp(p.add(1), c"..".as_ptr()) == 0
            {
                p = p.add(3);
            }
            let offset = (p as usize) - (fname as usize);
            std::ptr::copy_nonoverlapping(
                fname as *const u8,
                relative_directory as *mut u8,
                offset + 1,
            );
            *relative_directory.add(offset + 1) = 0;
            end_of_path = if rs_vim_ispathsep_nocolon(*p as c_int) != 0 {
                p.add(1)
            } else {
                p.cast_const()
            };
        }

        if rs_path_full_dir_name(relative_directory, buf, len) == FAIL {
            nvim_path_xfree(relative_directory);
            return FAIL;
        }
    }
    nvim_path_xfree(relative_directory);
    rs_append_path(buf, end_of_path, len)
}

/// Save absolute file name to buf[len].
///
/// # Safety
/// `buf` must have at least `len` bytes. `fname` may be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_FullName(
    fname: *const c_char,
    buf: *mut c_char,
    len: usize,
    force: c_int,
) -> c_int {
    *buf = 0;
    if fname.is_null() {
        return FAIL;
    }

    if libc::strlen(fname) > len - 1 {
        nvim_path_xstrlcpy(buf, fname, len); // truncate
        #[cfg(windows)]
        {
            rs_slash_adjust(buf);
        }
        return FAIL;
    }

    if rs_path_with_url(fname) != 0 {
        nvim_path_xstrlcpy(buf, fname, len);
        return OK;
    }

    let rv = rs_path_to_absolute(fname, buf, len, force);
    if rv == FAIL {
        nvim_path_xstrlcpy(buf, fname, len); // something failed; use the filename
    }
    #[cfg(windows)]
    {
        rs_slash_adjust(buf);
    }
    rv
}

/// Get an allocated copy of the full path to a file.
///
/// # Safety
/// `fname` may be NULL. Returns an allocated string or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_FullName_save(fname: *const c_char, force: c_int) -> *mut c_char {
    if fname.is_null() {
        return std::ptr::null_mut();
    }

    let buf = nvim_path_xmalloc(MAXPATHL);
    if rs_vim_FullName(fname, buf, MAXPATHL, force) == FAIL {
        nvim_path_xfree(buf);
        return nvim_path_xstrdup(fname);
    }
    buf
}

/// Saves the absolute path.
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_save_abs_path(name: *const c_char) -> *mut c_char {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    if rs_path_is_absolute(name) == 0 {
        return rs_FullName_save(name, 1);
    }
    nvim_path_xstrdup(name)
}

/// Get the full resolved path for fname.
///
/// On Unix, this is just FullName_save(fname, true).
///
/// # Safety
/// `fname` may be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_fix_fname(fname: *const c_char) -> *mut c_char {
    if fname.is_null() {
        return std::ptr::null_mut();
    }

    #[cfg(unix)]
    {
        rs_FullName_save(fname, 1)
    }

    #[cfg(not(unix))]
    {
        if rs_vim_isAbsName(fname) == 0
            || !libc::strstr(fname, b"..\0".as_ptr() as *const c_char).is_null()
            || !libc::strstr(fname, b"//\0".as_ptr() as *const c_char).is_null()
        {
            return rs_FullName_save(fname, 0);
        }
        // xstrdup + path_fix_case would go here for Windows
        nvim_path_xstrdup(fname)
    }
}

/// Return true if file names "f1" and "f2" are in the same directory.
/// "f1" may be a short name, "f2" must be a full path.
///
/// # Safety
/// `f1` and `f2` may be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_same_directory(f1: *const c_char, f2: *const c_char) -> c_int {
    if f1.is_null() || f2.is_null() {
        return 0;
    }

    let mut ffname = [0i8; MAXPATHL];
    rs_vim_FullName(f1, ffname.as_mut_ptr(), MAXPATHL, 0);
    let t1 = rs_path_tail_with_sep(ffname.as_ptr());
    let t2 = rs_path_tail_with_sep(f2);
    let len1 = (t1 as usize) - (ffname.as_ptr() as usize);
    let len2 = (t2 as usize) - (f2 as usize);
    if len1 != len2 {
        return 0;
    }
    c_int::from(rs_pathcmp(ffname.as_ptr(), f2, len1 as c_int) == 0)
}

/// Try to find a shortname by comparing the fullname with the current directory.
///
/// # Safety
/// `full_path` may be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_path_try_shorten_fname(full_path: *mut c_char) -> *mut c_char {
    if full_path.is_null() {
        return full_path;
    }

    let dirname = nvim_path_xmalloc(MAXPATHL);
    let mut p = full_path;

    if nvim_path_os_dirname(dirname, MAXPATHL) == OK {
        let short = rs_path_shorten_fname(full_path, dirname);
        if !short.is_null() && *short != 0 {
            p = short;
        }
    }
    nvim_path_xfree(dirname);
    p
}

/// Replace all slashes by backslashes (or vice versa on Windows).
/// No-op on Unix.
///
/// # Safety
/// `p` must be a valid pointer to a mutable, null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_slash_adjust(p: *mut c_char) {
    if p.is_null() {
        return;
    }

    // No-op on Unix
    #[cfg(not(windows))]
    {
        let _ = p;
    }

    #[cfg(windows)]
    {
        if rs_path_with_url(p) != 0 {
            return;
        }

        // Don't replace backslash in backtick quoted strings
        if *p as u8 == b'`' {
            let len = libc::strlen(p);
            if len > 2 && *p.add(len - 1) as u8 == b'`' {
                return;
            }
        }

        // Replace slashes based on 'shellslash' option
        // On Windows, this would check psepc/psepcN - simplified for now
        let mut ptr = p;
        while *ptr != 0 {
            if *ptr as u8 == b'/' {
                *ptr = b'\\' as c_char;
            }
            let slice = std::slice::from_raw_parts(ptr as *const u8, 8);
            let char_len = nvim_mbyte::utfc_ptr2len(slice);
            ptr = ptr.add(char_len);
        }
    }
}

// ============================================================================
// Phase 3: File Comparison, Case Fixing, Suffix Matching, and Path Utilities
// ============================================================================

/// Size of the opaque FileID buffer (matches C static assert).
const FILE_ID_SIZE: usize = 16;

/// Opaque FileID buffer type.
type FileIdBuf = [u8; FILE_ID_SIZE];

extern "C" {
    fn nvim_path_expand_env(src: *const c_char, dst: *mut c_char, dstlen: usize);
    fn nvim_path_os_fileid(fname: *const c_char, id_out: *mut u8) -> c_int;
    fn nvim_path_os_fileid_equal(a: *const u8, b: *const u8) -> c_int;
    fn nvim_path_file_exists_link(name: *const c_char) -> c_int;
    fn nvim_path_scandir_open(path: *const c_char, dir_out: *mut *mut c_void) -> c_int;
    fn nvim_path_scandir_next(dir: *mut c_void) -> *const c_char;
    fn nvim_path_scandir_close(dir: *mut c_void);
    fn nvim_path_STRICMP(a: *const c_char, b: *const c_char) -> c_int;
    fn nvim_path_copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep: *const c_char,
    ) -> usize;
    fn nvim_path_get_p_su() -> *const c_char;
    fn nvim_path_os_getenv(name: *const c_char) -> *mut c_char;
    fn nvim_path_os_can_exe(
        name: *const c_char,
        abspath: *mut *mut c_char,
        use_path: c_int,
    ) -> c_int;
    fn nvim_path_vim_env_iter(
        sep: c_char,
        val: *const c_char,
        iter: *const c_void,
        dir: *mut *const c_char,
        len: *mut usize,
    ) -> *const c_void;
    fn nvim_path_get_NameBuff() -> *mut c_char;
    fn nvim_path_get_NameBuff_size() -> usize;
    fn nvim_path_xstrlcat(dst: *mut c_char, src: *const c_char, dstsize: usize) -> usize;
    fn nvim_path_xmemcpyz(dst: *mut c_char, src: *const c_char, len: usize);
    /// Access ga_len field of a garray_T (opaque).
    fn nvim_path_ga_len(gap: *const c_void) -> c_int;
    /// Access ga_data[i] as a char* from a garray_T of char* pointers.
    fn nvim_path_ga_get_string(gap: *const c_void, i: c_int) -> *const c_char;
}

// ============================================================================
// path_full_compare - Compare two file paths by identity
// ============================================================================

/// Compare two file names and return a `FileComparison` value.
///
/// It tries to resolve file identities (via `os_fileid`). If `expandenv` is set,
/// environment variables in `s1` are expanded first.
///
/// If both files are missing but `checkname` is set, the full names are compared
/// to detect if they refer to the same (not-yet-existing) path.
///
/// # Safety
/// `s1` and `s2` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_path_full_compare(
    s1: *const c_char,
    s2: *const c_char,
    checkname: c_int,
    expandenv: c_int,
) -> c_int {
    let mut exp1 = [0i8; MAXPATHL];
    let mut full1 = [0i8; MAXPATHL];
    let mut full2 = [0i8; MAXPATHL];
    let mut file_id_1: FileIdBuf = [0u8; FILE_ID_SIZE];
    let mut file_id_2: FileIdBuf = [0u8; FILE_ID_SIZE];

    if expandenv != 0 {
        nvim_path_expand_env(s1, exp1.as_mut_ptr(), MAXPATHL);
    } else {
        nvim_path_xstrlcpy(exp1.as_mut_ptr(), s1, MAXPATHL);
    }

    let id_ok_1 = nvim_path_os_fileid(exp1.as_ptr(), file_id_1.as_mut_ptr()) != 0;
    let id_ok_2 = nvim_path_os_fileid(s2, file_id_2.as_mut_ptr()) != 0;

    if !id_ok_1 && !id_ok_2 {
        // If os_fileid() doesn't work, may compare the names.
        if checkname != 0 {
            rs_vim_FullName(exp1.as_ptr(), full1.as_mut_ptr(), MAXPATHL, 0);
            rs_vim_FullName(s2, full2.as_mut_ptr(), MAXPATHL, 0);
            if rs_path_fnamecmp(full1.as_ptr(), full2.as_ptr()) == 0 {
                return K_EQUAL_FILE_NAMES;
            }
        }
        return K_BOTH_FILES_MISSING;
    }
    if !id_ok_1 || !id_ok_2 {
        return K_ONE_FILE_MISSING;
    }
    if nvim_path_os_fileid_equal(file_id_1.as_ptr(), file_id_2.as_ptr()) != 0 {
        return K_EQUAL_FILES;
    }
    K_DIFFERENT_FILES
}

// ============================================================================
// path_fix_case - Fix filename case by scanning directory
// ============================================================================

/// Correct the case of a file name on case-insensitive file systems.
///
/// Scans the directory for a matching entry with different case but the same
/// inode, and overwrites the tail of `name` in-place.
///
/// # Safety
/// `name` must be a valid pointer to a mutable, null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_path_fix_case(name: *mut c_char) {
    if name.is_null() || *name == 0 {
        return;
    }

    // Check that the file exists (via lstat semantics).
    if nvim_path_file_exists_link(name) == 0 {
        return;
    }

    // Get the file identity of the original name.
    let mut orig_id: FileIdBuf = [0u8; FILE_ID_SIZE];
    if nvim_path_os_fileid(name, orig_id.as_mut_ptr()) == 0 {
        return;
    }

    // Find the tail (filename part) and open the parent directory.
    let slash = libc::strrchr(name, b'/' as c_int);
    let tail: *mut c_char;
    let mut dir_handle: *mut c_void = std::ptr::null_mut();
    let ok: c_int;

    if slash.is_null() {
        ok = nvim_path_scandir_open(c".".as_ptr(), &raw mut dir_handle);
        tail = name;
    } else {
        *slash = 0; // NUL-terminate at separator
        ok = nvim_path_scandir_open(name, &raw mut dir_handle);
        *slash = b'/' as c_char; // restore
        tail = slash.add(1);
    }

    if ok == 0 || dir_handle.is_null() {
        return;
    }

    let taillen = libc::strlen(tail);
    loop {
        let entry = nvim_path_scandir_next(dir_handle);
        if entry.is_null() {
            break;
        }

        // Only accept names that differ in case and are the same byte length.
        if nvim_path_STRICMP(tail, entry) == 0 && taillen == libc::strlen(entry) {
            let mut newname = [0i8; MAXPATHL + 1];

            // Build the full new name for verification.
            nvim_path_xstrlcpy(newname.as_mut_ptr(), name, MAXPATHL + 1);
            let tail_offset = (tail as usize) - (name as usize);
            nvim_path_xstrlcpy(
                newname.as_mut_ptr().add(tail_offset),
                entry,
                MAXPATHL - tail_offset + 1,
            );

            // Verify the inode is equal.
            let mut new_id: FileIdBuf = [0u8; FILE_ID_SIZE];
            if nvim_path_os_fileid(newname.as_ptr(), new_id.as_mut_ptr()) != 0
                && nvim_path_os_fileid_equal(orig_id.as_ptr(), new_id.as_ptr()) != 0
            {
                // Copy the correctly-cased entry name into the tail.
                let entry_len = libc::strlen(entry);
                std::ptr::copy_nonoverlapping(entry as *const u8, tail as *mut u8, entry_len + 1);
                break;
            }
        }
    }

    nvim_path_scandir_close(dir_handle);
}

// ============================================================================
// match_suffix - Check if filename matches 'suffixes' option
// ============================================================================

/// Maximum length of a file suffix.
const MAXSUFLEN: usize = 30;

/// Check if `fname` matches one of the file suffixes in the 'suffixes' option.
///
/// Returns true if the filename ends with one of the suffixes from `p_su`,
/// or if `p_su` contains an empty entry and the filename has no dot in its tail.
///
/// # Safety
/// `fname` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_match_suffix(fname: *const c_char) -> c_int {
    if fname.is_null() {
        return 0;
    }

    let fnamelen = libc::strlen(fname);
    let mut setsuflen: usize = 0;
    let mut setsuf = nvim_path_get_p_su().cast_mut();

    while *setsuf != 0 {
        let mut suf_buf = [0i8; MAXSUFLEN];
        setsuflen = nvim_path_copy_option_part(
            &raw mut setsuf,
            suf_buf.as_mut_ptr(),
            MAXSUFLEN,
            c".,".as_ptr(),
        );
        if setsuflen == 0 {
            let tail = rs_path_tail(fname);
            // Empty entry: match name without a '.'
            if libc::strchr(tail, b'.' as c_int).is_null() {
                setsuflen = 1;
                break;
            }
        } else {
            if fnamelen >= setsuflen
                && rs_path_fnamencmp(suf_buf.as_ptr(), fname.add(fnamelen - setsuflen), setsuflen)
                    == 0
            {
                break;
            }
            setsuflen = 0;
        }
    }
    c_int::from(setsuflen != 0)
}

// ============================================================================
// path_guess_exepath - Guess full executable path from argv[0]
// ============================================================================

/// Builds a full path from an invocation name `argv0`, based on heuristics.
///
/// 1. If `$PATH` is unset or `argv0` is absolute, use `argv0` directly.
/// 2. If `argv0` starts with `.` or contains a path separator, resolve relative to CWD.
/// 3. Otherwise, search `$PATH` for the executable.
///
/// # Safety
/// `argv0` and `buf` must be valid. `buf` must have at least `bufsize` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_path_guess_exepath(
    argv0: *const c_char,
    buf: *mut c_char,
    bufsize: usize,
) {
    if argv0.is_null() || buf.is_null() || bufsize == 0 {
        return;
    }

    let path = nvim_path_os_getenv(c"PATH".as_ptr());

    if path.is_null() || rs_path_is_absolute(argv0) != 0 {
        nvim_path_xstrlcpy(buf, argv0, bufsize);
    } else if *argv0 as u8 == b'.' || !libc::strchr(argv0, b'/' as c_int).is_null() {
        // Relative to CWD.
        if nvim_path_os_dirname(buf, MAXPATHL) != OK {
            *buf = 0;
        }
        nvim_path_xstrlcat(buf, c"/".as_ptr(), bufsize);
        nvim_path_xstrlcat(buf, argv0, bufsize);
    } else {
        // Search $PATH for plausible location.
        let mut iter: *const c_void = std::ptr::null();
        loop {
            let mut dir: *const c_char = std::ptr::null();
            let mut dir_len: usize = 0;
            iter =
                nvim_path_vim_env_iter(b':' as c_char, path, iter, &raw mut dir, &raw mut dir_len);
            if dir.is_null() || dir_len == 0 {
                break;
            }
            let namebuff = nvim_path_get_NameBuff();
            let namebuff_size = nvim_path_get_NameBuff_size();
            if dir_len + 1 > namebuff_size {
                if iter.is_null() {
                    break;
                }
                continue;
            }
            nvim_path_xmemcpyz(namebuff, dir, dir_len);
            nvim_path_xstrlcat(namebuff, c"/".as_ptr(), namebuff_size);
            nvim_path_xstrlcat(namebuff, argv0, namebuff_size);
            if nvim_path_os_can_exe(namebuff, std::ptr::null_mut(), 0) != 0 {
                nvim_path_xstrlcpy(buf, namebuff, bufsize);
                nvim_path_xfree(path);
                return;
            }
            if iter.is_null() {
                break;
            }
        }
        // Not found in $PATH, fall back to argv0.
        nvim_path_xstrlcpy(buf, argv0, bufsize);
    }
    nvim_path_xfree(path);
}

// ============================================================================
// find_previous_pathsep - Find previous path separator in a string
// ============================================================================

/// Find the previous path separator in `path`, searching backwards from `*psep`.
///
/// On entry, if `*psep` is on a separator, it is first skipped.
/// Returns `OK` (1) if a separator was found, updating `*psep` to point to it.
/// Returns `FAIL` (0) if no separator was found.
///
/// # Safety
/// `path` and `psep` must be valid. `*psep` must point within `path`.
#[no_mangle]
pub unsafe extern "C" fn rs_find_previous_pathsep(
    path: *const c_char,
    psep: *mut *mut c_char,
) -> c_int {
    if path.is_null() || psep.is_null() || (*psep).is_null() {
        return FAIL;
    }

    // Skip the current separator.
    if *psep > path.cast_mut() && rs_vim_ispathsep(*(*psep) as c_int) != 0 {
        *psep = (*psep).sub(1);
    }

    // Find the previous separator.
    while *psep > path.cast_mut() {
        if rs_vim_ispathsep(*(*psep) as c_int) != 0 {
            return OK;
        }
        // MB_PTR_BACK(path, *psep): p -= utf_head_off(s, p-1) + 1
        let base = path as *const u8;
        let cur = (*psep) as *const u8;
        let prev = cur.sub(1);
        let off = prev as usize - base as usize;
        let slice = std::slice::from_raw_parts(base, off + 1);
        let head_off = nvim_mbyte::utf_head_off(slice, off);
        *psep = (*psep).sub(head_off + 1);
    }

    FAIL
}

// ============================================================================
// is_unique - Check if a path suffix is unique in a garray
// ============================================================================

/// Returns true if `maybe_unique` is unique with respect to other paths in `gap`.
///
/// `maybe_unique` is the end portion of `((char **)gap->ga_data)[i]`.
/// Compares against all other entries to see if any shares the same suffix.
///
/// # Safety
/// `maybe_unique` must be a valid null-terminated C string.
/// `gap` must be a valid pointer to a `garray_T` containing `char *` entries.
#[no_mangle]
pub unsafe extern "C" fn rs_is_unique(
    maybe_unique: *const c_char,
    gap: *const c_void,
    i: c_int,
) -> c_int {
    if maybe_unique.is_null() || gap.is_null() {
        return 1; // treat as unique if invalid
    }

    let candidate_len = libc::strlen(maybe_unique);
    let ga_len = nvim_path_ga_len(gap);

    for j in 0..ga_len {
        if j == i {
            continue; // don't compare it with itself
        }
        let other = nvim_path_ga_get_string(gap, j);
        if other.is_null() {
            continue;
        }
        let other_path_len = libc::strlen(other);
        if other_path_len < candidate_len {
            continue; // it's different when it's shorter
        }
        let rival = other.add(other_path_len - candidate_len);
        if rs_path_fnamecmp(maybe_unique, rival) == 0
            && (rival == other || rs_vim_ispathsep(*rival.sub(1) as c_int) != 0)
        {
            return 0; // match found
        }
    }
    1 // no match found
}

// ============================================================================
// has_special_wildchar - Check for shell-expanding wildcard characters
// ============================================================================

/// Check if string `p` contains special wildcard characters that require
/// shell expansion.
///
/// On Unix, the special characters are `` ` ' { ``.
///
/// Rules:
/// - Backslash-escaped characters are skipped.
/// - Line breaks (`\r`, `\n`) terminate the scan.
/// - `{` requires `EW_NOTFOUND` flag or a matching `}` to count.
/// - `` ` `` and `'` require a matching pair to count.
///
/// # Safety
/// `p` must be a valid null-terminated C string.
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn rs_has_special_wildchar(p: *const c_char, flags: c_int) -> c_int {
    if p.is_null() {
        return 0;
    }

    let mut ptr = p;
    while *ptr != 0 {
        let c = *ptr as u8;

        // Disallow line break characters.
        if c == b'\r' || c == b'\n' {
            break;
        }

        // Allow for escaping.
        if c == b'\\'
            && *ptr.add(1) != 0
            && *ptr.add(1) as u8 != b'\r'
            && *ptr.add(1) as u8 != b'\n'
        {
            ptr = ptr.add(1);
        } else if c == b'`' || c == b'\'' || c == b'{' {
            // Need a shell for curly braces only when including non-existing files.
            if c == b'{' && (flags & EW_NOTFOUND) == 0 {
                let slice = std::slice::from_raw_parts(ptr as *const u8, 8);
                let char_len = nvim_mbyte::utfc_ptr2len(slice);
                ptr = ptr.add(char_len);
                continue;
            }
            // A { must be followed by a matching }.
            if c == b'{' && libc::strchr(ptr, b'}' as c_int).is_null() {
                let slice = std::slice::from_raw_parts(ptr as *const u8, 8);
                let char_len = nvim_mbyte::utfc_ptr2len(slice);
                ptr = ptr.add(char_len);
                continue;
            }
            // A quote and backtick must be followed by another one.
            if (c == b'`' || c == b'\'') && libc::strchr(ptr.add(1), c as c_int).is_null() {
                let slice = std::slice::from_raw_parts(ptr as *const u8, 8);
                let char_len = nvim_mbyte::utfc_ptr2len(slice);
                ptr = ptr.add(char_len);
                continue;
            }
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
// get_path_cutoff - Get the portion of fname matching the longest path in gap
// ============================================================================

/// Returns a pointer to the file or directory name in `fname` that matches
/// the longest path in `gap`, or NULL if there is no match.
///
/// Example:
/// ```text
///    path: /foo/bar/baz
///   fname: /foo/bar/baz/quux.txt
/// returns:              ^this
/// ```
///
/// # Safety
/// `fname` must be a valid null-terminated C string.
/// `gap` must be a valid pointer to a `garray_T` containing `char *` entries.
#[no_mangle]
pub unsafe extern "C" fn rs_get_path_cutoff(
    fname: *const c_char,
    gap: *const c_void,
) -> *const c_char {
    if fname.is_null() || gap.is_null() {
        return std::ptr::null();
    }

    let mut maxlen: usize = 0;
    let mut cutoff: *const c_char = std::ptr::null();
    let ga_len = nvim_path_ga_len(gap);

    for i in 0..ga_len {
        let path_part = nvim_path_ga_get_string(gap, i);
        if path_part.is_null() {
            continue;
        }
        let mut j: usize = 0;

        // Compare characters, treating path separators as equivalent on Windows.
        loop {
            let fc = *fname.add(j) as u8;
            let pc = *path_part.add(j) as u8;

            #[cfg(not(windows))]
            let chars_match = fc == pc;
            #[cfg(windows)]
            let chars_match = fc == pc
                || (rs_vim_ispathsep(fc as c_int) != 0 && rs_vim_ispathsep(pc as c_int) != 0);

            if !chars_match || fc == 0 || pc == 0 {
                break;
            }
            j += 1;
        }

        if j > maxlen {
            maxlen = j;
            cutoff = fname.add(j);
        }
    }

    // Skip to the file or directory name (past path separators).
    if !cutoff.is_null() {
        while rs_vim_ispathsep(*cutoff as c_int) != 0 {
            // MB_PTR_ADV
            let slice = std::slice::from_raw_parts(cutoff as *const u8, 8);
            let char_len = nvim_mbyte::utfc_ptr2len(slice);
            cutoff = cutoff.add(char_len);
        }
    }

    cutoff
}

// ============================================================================
// Phase 4: addfile + scandir_next_with_dots
// ============================================================================

extern "C" {
    fn nvim_path_os_path_exists(fname: *const c_char) -> c_int;
    fn nvim_path_ga_append_string(gap: *mut c_void, s: *mut c_char);
}

/// Static counter for `scandir_next_with_dots`.
static mut SCANDIR_DOT_COUNT: c_int = 0;

/// Yields `.` and `..` before delegating to `os_scandir_next`.
///
/// Call with `dir == NULL` to reset the counter. After reset, the first two
/// calls return `.` and `..`, then subsequent calls delegate to
/// `nvim_path_scandir_next`.
///
/// # Safety
/// `dir` must be a valid `Directory` handle (from `nvim_path_scandir_open`) or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_scandir_next_with_dots(dir: *mut c_void) -> *const c_char {
    if dir.is_null() {
        // Reset
        SCANDIR_DOT_COUNT = 0;
        return std::ptr::null();
    }

    SCANDIR_DOT_COUNT += 1;
    if SCANDIR_DOT_COUNT == 1 {
        return c".".as_ptr();
    }
    if SCANDIR_DOT_COUNT == 2 {
        return c"..".as_ptr();
    }
    nvim_path_scandir_next(dir)
}

/// Add a file to a file list. Accepted flags:
/// - `EW_DIR`: add directories
/// - `EW_FILE`: add files
/// - `EW_EXEC`: add executable files
/// - `EW_NOTFOUND`: add even when it doesn't exist
/// - `EW_ADDSLASH`: add slash after directory name
/// - `EW_ALLLINKS`: add symlink also when the referred file does not exist
/// - `EW_SHELLCMD`: when invoked from expand_shellcmd(), do not use `$PATH`
///
/// # Safety
/// `gap` must be a valid pointer to a `garray_T` of `char *`.
/// `f` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_addfile(gap: *mut c_void, f: *const c_char, flags: c_int) {
    if gap.is_null() || f.is_null() {
        return;
    }

    // If the file/dir/link doesn't exist, may not add it.
    if (flags & EW_NOTFOUND) == 0 {
        if (flags & EW_ALLLINKS) != 0 {
            if nvim_path_file_exists_link(f) == 0 {
                return;
            }
        } else if nvim_path_os_path_exists(f) == 0 {
            return;
        }
    }

    let isdir = nvim_path_os_isdir(f) != 0;
    if (isdir && (flags & EW_DIR) == 0) || (!isdir && (flags & EW_FILE) == 0) {
        return;
    }

    // If the file isn't executable, may not add it. Do accept directories.
    // When invoked from expand_shellcmd() do not use $PATH.
    if !isdir
        && (flags & EW_EXEC) != 0
        && nvim_path_os_can_exe(
            f,
            std::ptr::null_mut(),
            i32::from((flags & EW_SHELLCMD) == 0),
        ) == 0
    {
        return;
    }

    let flen = libc::strlen(f);
    let p = nvim_path_xmalloc(flen + 1 + usize::from(isdir)) as *mut c_char;
    std::ptr::copy_nonoverlapping(f as *const u8, p as *mut u8, flen + 1);

    // On Windows, adjust slashes (no-op on Unix since BACKSLASH_IN_FILENAME is not defined).
    #[cfg(windows)]
    rs_slash_adjust(p);

    // Append a slash after directory names if none is present.
    if isdir && (flags & EW_ADDSLASH) != 0 {
        rs_add_pathsep(p);
    }

    nvim_path_ga_append_string(gap, p);
}

// ============================================================================
// simplify_filename - Simplify a file name in place
// ============================================================================

/// Size of an opaque buffer large enough to hold a C `FileInfo` struct.
/// `FileInfo` contains a `uv_stat_t` which is ~160 bytes on Linux x86_64.
/// 256 bytes provides a comfortable margin for any platform.
const FILEINFO_SIZE: usize = 256;

extern "C" {
    fn nvim_path_os_fileinfo(fname: *const c_char, info_out: *mut u8) -> c_int;
    fn nvim_path_os_fileinfo_link(fname: *const c_char, info_out: *mut u8) -> c_int;
    fn nvim_path_os_fileinfo_id_equal(a: *const u8, b: *const u8) -> c_int;
}

/// Simplify a file name in place, by stripping `.`, `..` and duplicate `/`.
///
/// This is the Rust replacement for `simplify_filename()` in path.c.
/// The file name is modified in-place and the resulting length is returned.
///
/// # Safety
/// `filename` must be a valid, writable, NUL-terminated C string.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_simplify_filename(filename: *mut c_char) -> usize {
    let mut components: i32 = 0;
    let mut stripping_disabled = false;
    let mut relative = true;

    let mut p = filename;

    // On Windows, skip "x:" drive prefix.
    #[cfg(windows)]
    {
        if *p != 0 && *p.add(1) == b':' as c_char {
            p = p.add(2);
        }
    }

    if rs_vim_ispathsep(*p as c_int) != 0 {
        relative = false;
        loop {
            p = p.add(1);
            if rs_vim_ispathsep(*p as c_int) == 0 {
                break;
            }
        }
    }

    let start = p; // remember start after "c:/" or "/" or "///"
    let mut p_end = p.add(libc::strlen(p)); // points to NUL at end of string

    // Posix says that "//path" is unchanged but "///path" is "/path".
    #[cfg(unix)]
    {
        if start > filename.add(2) {
            let move_len = (p_end as usize - p as usize) + 1; // +1 for NUL
            std::ptr::copy(p as *const u8, filename.add(1) as *mut u8, move_len);
            p_end = p_end.sub(p as usize - filename.add(1) as usize);
            p = filename.add(1);
            // start is reassigned below (shadowed in C, we just rebind)
        }
    }

    // On Unix, start may have been reassigned above.
    #[cfg(unix)]
    let start = if start > filename.add(2) {
        filename.add(1)
    } else {
        start
    };
    #[cfg(not(unix))]
    let start = start;

    loop {
        // At this point "p" is pointing to the char following a single "/"
        // or "p" is at the "start" of the (absolute or relative) path name.
        if rs_vim_ispathsep(*p as c_int) != 0 {
            // Remove duplicate "/"
            let src = p.add(1);
            let move_len = (p_end as usize - src as usize) + 1;
            std::ptr::copy(src as *const u8, p as *mut u8, move_len);
            p_end = p_end.sub(1);
        } else if *p == b'.' as c_char
            && (rs_vim_ispathsep(*p.add(1) as c_int) != 0 || *p.add(1) == 0)
        {
            if p == start && relative {
                // keep single "." or leading "./"
                p = p.add(1 + usize::from(*p.add(1) != 0));
            } else {
                // Strip "./" or ".///".  If we are at the end of the file name
                // and there is no trailing path separator, either strip "/." if
                // we are after "start", or strip "." if we are at the beginning
                // of an absolute path name.
                let mut tail = p.add(1);
                if *p.add(1) != 0 {
                    while rs_vim_ispathsep(*tail as c_int) != 0 {
                        // MB_PTR_ADV(tail)
                        let slice = std::slice::from_raw_parts(tail as *const u8, 8);
                        let char_len = nvim_mbyte::utfc_ptr2len(slice);
                        tail = tail.add(char_len);
                    }
                } else if p > start {
                    p = p.sub(1); // strip preceding path separator
                }
                let move_len = (p_end as usize - tail as usize) + 1;
                std::ptr::copy(tail as *const u8, p as *mut u8, move_len);
                p_end = p_end.sub(tail as usize - p as usize);
            }
        } else if *p == b'.' as c_char
            && *p.add(1) == b'.' as c_char
            && (rs_vim_ispathsep(*p.add(2) as c_int) != 0 || *p.add(2) == 0)
        {
            // Skip to after ".." or "../" or "..///".
            let mut tail = p.add(2);
            while rs_vim_ispathsep(*tail as c_int) != 0 {
                // MB_PTR_ADV(tail)
                let slice = std::slice::from_raw_parts(tail as *const u8, 8);
                let char_len = nvim_mbyte::utfc_ptr2len(slice);
                tail = tail.add(char_len);
            }

            if components > 0 {
                // Strip one preceding component
                let mut do_strip = false;

                // Don't strip for an erroneous file name.
                if !stripping_disabled {
                    // If the preceding component does not exist in the file
                    // system, we strip it.  On Unix, we don't accept a symbolic
                    // link that refers to a non-existent file.
                    let saved_char = *p.sub(1);
                    *p.sub(1) = 0;
                    let mut file_info = [0u8; FILEINFO_SIZE];
                    if nvim_path_os_fileinfo_link(filename.cast_const(), file_info.as_mut_ptr())
                        == 0
                    {
                        do_strip = true;
                    }
                    *p.sub(1) = saved_char;

                    p = p.sub(1);
                    // Skip back to after previous '/'.
                    while p > start && rs_after_pathsep(start.cast_const(), p.cast_const()) == 0 {
                        // MB_PTR_BACK(start, p)
                        let base = start as *const u8;
                        let cur = p as *const u8;
                        let prev = cur.sub(1);
                        let off = prev as usize - base as usize;
                        let slice = std::slice::from_raw_parts(base, off + 1);
                        let head_off = utf_head_off(slice, off);
                        p = p.sub(head_off + 1);
                    }

                    if !do_strip {
                        // If the component exists in the file system, check
                        // that stripping it won't change the meaning of the
                        // file name.
                        let saved_char2 = *tail;
                        *tail = 0;
                        let mut file_info2 = [0u8; FILEINFO_SIZE];
                        if nvim_path_os_fileinfo(filename.cast_const(), file_info2.as_mut_ptr())
                            != 0
                        {
                            do_strip = true;
                        } else {
                            stripping_disabled = true;
                        }
                        *tail = saved_char2;

                        if do_strip {
                            // The parent of the directory pointed to by the link
                            // must be the same as the stripped file name.
                            let mut new_file_info = [0u8; FILEINFO_SIZE];
                            if p == start && relative {
                                nvim_path_os_fileinfo(c".".as_ptr(), new_file_info.as_mut_ptr());
                            } else {
                                let saved_char3 = *p;
                                *p = 0;
                                nvim_path_os_fileinfo(
                                    filename.cast_const(),
                                    new_file_info.as_mut_ptr(),
                                );
                                *p = saved_char3;
                            }

                            if nvim_path_os_fileinfo_id_equal(
                                file_info2.as_ptr(),
                                new_file_info.as_ptr(),
                            ) == 0
                            {
                                do_strip = false;
                                // We don't disable stripping of later
                                // components since the unstripped path name is
                                // still valid.
                            }
                        }
                    }
                }

                if do_strip {
                    // Strip previous component.  If the result would get empty
                    // and there is no trailing path separator, leave a single
                    // "." instead.  If we are at the end of the file name and
                    // there is no trailing path separator and a preceding
                    // component is left after stripping, strip its trailing
                    // path separator as well.
                    if p == start && relative && *tail.sub(1) == b'.' as c_char {
                        *p = b'.' as c_char;
                        p = p.add(1);
                        *p = 0;
                    } else {
                        if p > start && *tail.sub(1) == b'.' as c_char {
                            p = p.sub(1);
                        }
                        let move_len = (p_end as usize - tail as usize) + 1;
                        std::ptr::copy(tail as *const u8, p as *mut u8, move_len);
                        p_end = p_end.sub(tail as usize - p as usize);
                    }

                    components -= 1;
                } else {
                    // Skip the ".." or "../" and reset the counter for the
                    // components that might be stripped later on.
                    p = tail;
                    components = 0;
                }
            } else if p == start && !relative {
                // Leading "/.." or "/../"
                let move_len = (p_end as usize - tail as usize) + 1;
                std::ptr::copy(tail as *const u8, p as *mut u8, move_len);
                p_end = p_end.sub(tail as usize - p as usize);
            } else {
                if p == start.add(2) && *p.sub(2) == b'.' as c_char {
                    // Leading "./../" — strip leading "./"
                    let move_len = (p_end as usize - p as usize) + 1;
                    std::ptr::copy(p as *const u8, p.sub(2) as *mut u8, move_len);
                    p_end = p_end.sub(2);
                    tail = tail.sub(2);
                }
                p = tail; // skip to char after ".." or "../"
            }
        } else {
            components += 1; // Simple path component.
            p = rs_path_next_component(p.cast_const()).cast_mut();
        }

        if *p == 0 {
            break;
        }
    }

    p_end as usize - filename as usize
}

// ============================================================================
// Phase 6: do_path_expand / path_expand
// ============================================================================

/// Depth counter for `**` expansion.  Mirrors the C `static int stardepth`.
static mut STARDEPTH: c_int = 0;

extern "C" {
    // --- regex opaque helpers (Phase 6) ---
    /// Compile a file-matching regex pattern.
    /// `ic` != 0 means ignore case. Returns an opaque handle (NULL on failure).
    fn nvim_path_compile_pattern(pat: *const c_char, flags: c_int, ic: c_int) -> *mut c_void;
    /// Execute a compiled regex against `s`. Returns non-zero on match.
    fn nvim_path_match_pattern(handle: *const c_void, s: *const c_char) -> c_int;
    /// Free a compiled regex handle.
    fn nvim_path_free_pattern(handle: *mut c_void);

    // --- misc C accessors (Phase 6) ---
    fn nvim_path_os_breakcheck();
    fn nvim_path_get_got_int() -> c_int;
    fn nvim_path_get_p_fic() -> c_int;
    fn nvim_path_os_file_is_readable(fname: *const c_char) -> c_int;
    fn nvim_path_file_pat_to_reg_pat(
        pat: *const c_char,
        pat_end: *const c_char,
        allow_dirs: *mut c_char,
        no_bslash: c_int,
    ) -> *mut c_char;
    fn nvim_path_rem_backslash(s: *const c_char) -> c_int;
    fn nvim_path_backslash_halve(s: *mut c_char);
    fn nvim_path_emsg_silent_inc();
    fn nvim_path_emsg_silent_dec();
    fn nvim_path_mb_isalpha(c: c_int) -> c_int;
    fn nvim_path_utf_ptr2char(p: *const c_char) -> c_int;
    /// Sort `ga_data[start..]` (array of `char *`) using `pstrcmp`.
    fn nvim_path_ga_sort_strings(gap: *mut c_void, start: c_int);
}

/// RE_MAGIC flag for `vim_regcomp`.
const RE_MAGIC: c_int = 1;
/// RE_NOBREAK flag for `vim_regcomp`.
const RE_NOBREAK: c_int = 16;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Check if a byte is in the set of wildcard characters that `path_expand`
/// recognises.  On Unix: `*?[{~$`.  On Windows: `*?[~`.
#[inline]
fn is_wild_char(c: u8) -> bool {
    #[cfg(unix)]
    {
        matches!(c, b'*' | b'?' | b'[' | b'{' | b'~' | b'$')
    }
    #[cfg(not(unix))]
    {
        matches!(c, b'*' | b'?' | b'[' | b'~')
    }
}

/// Copy a NUL-terminated C string from `src` into `dst`.
/// Returns the number of bytes copied (excluding the NUL terminator).
/// At most `dst_size - 1` bytes are copied; the result is always NUL-terminated
/// when `dst_size > 0`.
#[inline]
unsafe fn copy_cstr(dst: *mut c_char, dst_size: usize, src: *const c_char) -> usize {
    if dst_size == 0 {
        return 0;
    }
    let src_len = libc::strlen(src);
    let copy_len = src_len.min(dst_size - 1);
    std::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, copy_len);
    *dst.add(copy_len) = 0;
    copy_len
}

// ---------------------------------------------------------------------------
// rs_path_expand
// ---------------------------------------------------------------------------

/// Expand wildcards in `path`, appending matches to the growarray `gap`.
///
/// Thin wrapper around `rs_do_path_expand` with `wildoff = 0` and
/// `didstar = false`.
///
/// # Safety
/// - `gap` must be a valid pointer to a `garray_T` of `char *`.
/// - `path` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_path_expand(
    gap: *mut c_void,
    path: *const c_char,
    flags: c_int,
) -> usize {
    rs_do_path_expand(gap, path, 0, flags, 0)
}

// ---------------------------------------------------------------------------
// rs_do_path_expand
// ---------------------------------------------------------------------------

/// Recursive implementation of `path_expand`.
///
/// Characters before `path + wildoff` are not subject to wildcard expansion.
///
/// # Safety
/// - `gap` must be a valid pointer to a `garray_T` of `char *`.
/// - `path` must be a valid NUL-terminated C string.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_do_path_expand(
    gap: *mut c_void,
    path: *const c_char,
    wildoff: usize,
    flags: c_int,
    didstar: c_int,
) -> usize {
    let start_len: c_int = nvim_path_ga_len(gap);
    let mut starstar = false;

    // Expanding "**" may take a long time, check for CTRL-C.
    if STARDEPTH > 0 && (flags & EW_NOBREAK) == 0 {
        nvim_path_os_breakcheck();
        if nvim_path_get_got_int() != 0 {
            return 0;
        }
    }

    // Make room for file name (a bit too much to stay on the safe side).
    let path_strlen = libc::strlen(path);
    let buflen: usize = path_strlen + MAXPATHL;
    let buf: *mut c_char = nvim_path_xmalloc(buflen) as *mut c_char;

    // Find the first part in the path name that contains a wildcard.
    // When EW_ICASE is set every letter is considered to be a wildcard.
    // Copy it into "buf", including the preceding characters.
    let mut p: *mut c_char = buf;
    let mut s: *mut c_char = buf;
    let mut e: *mut c_char = std::ptr::null_mut();
    let mut path_end: *const c_char = path;

    while *path_end != 0 {
        // May ignore a wildcard that has a backslash before it; it will
        // be removed by rem_backslash() or file_pat_to_reg_pat() below.
        if path_end >= path.add(wildoff) && nvim_path_rem_backslash(path_end) != 0 {
            *p = *path_end;
            p = p.add(1);
            path_end = path_end.add(1);
        } else if rs_vim_ispathsep_nocolon(*path_end as c_int) != 0 {
            if !e.is_null() {
                break;
            }
            s = p.add(1);
        } else if path_end >= path.add(wildoff) && {
            #[cfg(unix)]
            {
                is_wild_char(*path_end as u8)
                    || (nvim_path_get_p_fic() == 0
                        && (flags & EW_ICASE) != 0
                        && nvim_path_mb_isalpha(nvim_path_utf_ptr2char(path_end)) != 0)
            }
            #[cfg(not(unix))]
            {
                is_wild_char(*path_end as u8)
            }
        } {
            e = p;
        }

        // MB_PTR_ADV — advance by multibyte character length
        let slice = std::slice::from_raw_parts(path_end as *const u8, 8);
        let charlen = nvim_mbyte::utfc_ptr2len(slice);
        std::ptr::copy_nonoverlapping(path_end as *const u8, p as *mut u8, charlen);
        p = p.add(charlen);
        path_end = path_end.add(charlen);
    }
    e = p;
    *e = 0;

    // Now we have one wildcard component between "s" and "e".
    // Remove backslashes between "wildoff" and the start of the wildcard
    // component.
    p = buf.add(wildoff);
    while p < s {
        if nvim_path_rem_backslash(p.cast_const()) != 0 {
            // STRMOVE(p, p + 1): move bytes from p+1..=e to p..=e-1
            let move_len = (e as usize - p.add(1) as usize) + 1; // includes NUL
            std::ptr::copy(p.add(1) as *const u8, p as *mut u8, move_len);
            e = e.sub(1);
            s = s.sub(1);
        } else {
            p = p.add(1);
        }
    }

    // Check for "**" between "s" and "e".
    p = s;
    while p < e {
        if *p == b'*' as c_char && *p.add(1) == b'*' as c_char {
            starstar = true;
        }
        p = p.add(1);
    }

    // Convert the file pattern to a regexp pattern.
    let starts_with_dot: c_int = i32::from(*s == b'.' as c_char);
    let pat: *mut c_char =
        nvim_path_file_pat_to_reg_pat(s.cast_const(), e.cast_const(), std::ptr::null_mut(), 0);
    if pat.is_null() {
        nvim_path_xfree(buf);
        return 0;
    }

    // Compile the regexp into a program.
    #[cfg(unix)]
    let ic: c_int = i32::from((flags & EW_ICASE) != 0 || nvim_path_get_p_fic() != 0);
    #[cfg(not(unix))]
    let ic: c_int = 1; // Always ignore case on Windows.

    if (flags & (EW_NOERROR | EW_NOTWILD)) != 0 {
        nvim_path_emsg_silent_inc();
    }
    let nobreak: bool = (flags & EW_NOBREAK) != 0;
    let re_flags: c_int = RE_MAGIC | if nobreak { RE_NOBREAK } else { 0 };
    let reghandle: *mut c_void = nvim_path_compile_pattern(pat, re_flags, ic);
    if (flags & (EW_NOERROR | EW_NOTWILD)) != 0 {
        nvim_path_emsg_silent_dec();
    }
    nvim_path_xfree(pat);

    if reghandle.is_null() && (flags & EW_NOTWILD) == 0 {
        nvim_path_xfree(buf);
        return 0;
    }

    let mut len: usize = (s as usize) - (buf as usize);

    // If "**" is by itself, this is the first time we encounter it and more
    // is following then find matches without any directory.
    if didstar == 0
        && STARDEPTH < 100
        && starstar
        && (e as usize - s as usize) == 2
        && *path_end == b'/' as c_char
    {
        copy_cstr(s, buflen - len, path_end.add(1));
        STARDEPTH += 1;
        rs_do_path_expand(gap, buf.cast_const(), len, flags, 1);
        STARDEPTH -= 1;
    }
    *s = 0;

    // Open the directory.
    let mut dir_handle: *mut c_void = std::ptr::null_mut();
    let dirpath: *const c_char = if *buf == 0 {
        c".".as_ptr()
    } else {
        buf.cast_const()
    };

    if nvim_path_os_file_is_readable(dirpath) != 0
        && nvim_path_scandir_open(dirpath, &raw mut dir_handle) != 0
    {
        // Find all matching entries.
        rs_scandir_next_with_dots(std::ptr::null_mut()); // initialize

        loop {
            if nvim_path_get_got_int() != 0 {
                break;
            }
            let name: *const c_char = rs_scandir_next_with_dots(dir_handle);
            if name.is_null() {
                break;
            }

            len = (s as usize) - (buf as usize);

            // Filter: skip dot-files unless the pattern starts with '.' or
            // EW_DODOT is set (and it isn't plain "." or "..").
            let name0 = *name as u8;
            let dot_ok = name0 != b'.'
                || starts_with_dot != 0
                || ((flags & EW_DODOT) != 0
                    && *name.add(1) != 0
                    && (*name.add(1) as u8 != b'.' || *name.add(2) != 0));
            if !dot_ok {
                continue;
            }

            // Match: regex or literal (EW_NOTWILD).
            let matched = (!reghandle.is_null()
                && nvim_path_match_pattern(reghandle.cast_const(), name) != 0)
                || ((flags & EW_NOTWILD) != 0
                    && rs_path_fnamencmp(path.add(len), name, (e as usize) - (s as usize)) == 0);
            if !matched {
                continue;
            }

            // Copy the matched name into buf after the directory prefix.
            let name_len = copy_cstr(s, buflen - len, name);
            len += name_len;
            if len + 1 >= buflen {
                continue;
            }

            // For "**" in the pattern first go deeper in the tree to
            // find matches.
            if starstar && STARDEPTH < 100 {
                // Write "/**<path_end>" into buf+len.
                let tail_space = buflen - len;
                if tail_space > 3 {
                    *buf.add(len) = b'/' as c_char;
                    *buf.add(len + 1) = b'*' as c_char;
                    *buf.add(len + 2) = b'*' as c_char;
                    copy_cstr(buf.add(len + 3), tail_space - 3, path_end);
                }
                STARDEPTH += 1;
                rs_do_path_expand(gap, buf.cast_const(), len + 1, flags, 1);
                STARDEPTH -= 1;
            }

            // Append remaining path_end.
            copy_cstr(buf.add(len), buflen - len, path_end);

            if rs_path_has_exp_wildcard(path_end) != 0 {
                // Handle more wildcards.
                if STARDEPTH < 100 {
                    STARDEPTH += 1;
                    // Need to expand another component of the path.
                    // Remove backslashes for the remaining components only.
                    rs_do_path_expand(gap, buf.cast_const(), len + 1, flags, 0);
                    STARDEPTH -= 1;
                }
            } else {
                // No more wildcards, check if there is a match.
                // Remove backslashes for the remaining components only.
                if *path_end != 0 {
                    nvim_path_backslash_halve(buf.add(len + 1));
                }
                // Add existing file or symbolic link.
                if (flags & EW_ALLLINKS) != 0 {
                    if nvim_path_file_exists_link(buf.cast_const()) != 0 {
                        rs_addfile(gap, buf.cast_const(), flags);
                    }
                } else if nvim_path_os_path_exists(buf.cast_const()) != 0 {
                    rs_addfile(gap, buf.cast_const(), flags);
                }
            }
        }

        nvim_path_scandir_close(dir_handle);
    }

    nvim_path_xfree(buf);
    nvim_path_free_pattern(reghandle);

    // When interrupted the matches probably won't be used and sorting can be
    // slow, thus skip it.
    let matches: usize = (nvim_path_ga_len(gap) - start_len) as usize;
    if matches > 0 && nvim_path_get_got_int() == 0 {
        nvim_path_ga_sort_strings(gap, start_len);
    }
    matches
}

// ============================================================================
// Phase 7: expand_path_option, uniquefy_paths, gen_expand_wildcards,
//          expand_wildcards
// ============================================================================

extern "C" {
    // --- Phase 7 accessors ---
    /// Get path option: curbuf->b_p_path if non-empty, else p_path
    fn nvim_path_get_path_option() -> *const c_char;
    /// Get curbuf->b_ffname (may be NULL)
    fn nvim_path_curbuf_ffname() -> *const c_char;
    /// expand_env_save_opt(src, true) -- may return NULL
    fn nvim_path_expand_env_save_opt(src: *mut c_char, one: c_int) -> *mut c_char;
    /// backslash_halve_save(str) -- allocates copy
    fn nvim_path_backslash_halve_save(s: *const c_char) -> *mut c_char;
    /// p_wig option value
    fn nvim_path_get_p_wig() -> *const c_char;
    /// match_file_list(list, fname, ffname)
    fn nvim_path_match_file_list(
        list: *const c_char,
        fname: *const c_char,
        ffname: *const c_char,
    ) -> c_int;
    /// os_expand_wildcards(num_pat, pat, num_file, file, flags)
    fn nvim_path_os_expand_wildcards(
        num_pat: c_int,
        pat: *mut *mut c_char,
        num_file: *mut c_int,
        file: *mut *mut *mut c_char,
        flags: c_int,
    ) -> c_int;
    /// expand_backtick(gap, pat, flags) -- stays in C
    fn nvim_path_expand_backtick(gap: *mut c_void, pat: *mut c_char, flags: c_int) -> c_int;
    /// expand_in_path(gap, pat, flags) -- stays in C
    fn nvim_path_expand_in_path(gap: *mut c_void, pat: *mut c_char, flags: c_int) -> c_int;
    /// Heap-allocate a garray_T for strings and ga_init it
    fn nvim_path_ga_alloc_strings(growsize: c_int) -> *mut c_void;
    /// Free the garray_T handle (NOT ga_clear)
    fn nvim_path_ga_free_handle(gap: *mut c_void);
    /// ga_clear_strings
    fn nvim_path_ga_clear_strings(gap: *mut c_void);
    /// ga_remove_duplicate_strings
    fn nvim_path_ga_remove_duplicate_strings(gap: *mut c_void);
    /// Get ga_data pointer (array of char*)
    fn nvim_path_ga_get_data(gap: *const c_void) -> *mut *mut c_char;
    /// Set ga_data[i] to a new value
    fn nvim_path_ga_set_string(gap: *mut c_void, i: c_int, s: *mut c_char);
    /// xmemdupz(s, len) -- allocates len+1 bytes
    fn nvim_path_xmemdupz(s: *const c_char, len: usize) -> *mut c_char;
    /// xcalloc(count, size)
    fn nvim_path_xcalloc(count: usize, size: usize) -> *mut c_void;
    /// Get ga_data as void*
    fn nvim_path_ga_get_data_ptr(gap: *const c_void) -> *mut c_void;
}

/// RE_STRING flag for `vim_regcomp`.
const RE_STRING: c_int = 2;

/// Path separator character.
#[cfg(unix)]
const PATHSEP: u8 = b'/';

#[cfg(not(unix))]
const PATHSEP: u8 = b'/';

/// Format a relative path: writes "./" + short_name into dst.
///
/// # Safety
/// `dst` must have at least `dstsize` bytes. `short_name` must be a valid
/// NUL-terminated C string.
unsafe fn format_relative(dst: *mut c_char, dstsize: usize, short_name: *const c_char) {
    if dstsize < 3 {
        return;
    }
    *dst = b'.' as c_char;
    *dst.add(1) = PATHSEP as c_char;
    let slen = libc::strlen(short_name);
    let copy_len = slen.min(dstsize - 3);
    std::ptr::copy_nonoverlapping(short_name as *const u8, dst.add(2) as *mut u8, copy_len);
    *dst.add(2 + copy_len) = 0;
}

// ---------------------------------------------------------------------------
// rs_expand_path_option
// ---------------------------------------------------------------------------

/// Split the 'path' option into an array of strings in garray_T.
///
/// Relative paths are expanded to their equivalent fullpath. This includes
/// the "." (relative to current buffer directory) and empty path (relative
/// to current directory) notations.
///
/// # Safety
/// All pointers must be valid. `gap` must be a valid garray_T of `char *`.
#[no_mangle]
pub unsafe extern "C" fn rs_expand_path_option(
    curdir: *mut c_char,
    path_option: *mut c_char,
    gap: *mut c_void,
) {
    let buf: *mut c_char = nvim_path_xmalloc(MAXPATHL) as *mut c_char;
    let mut curdirlen: usize = 0;
    let mut opt_ptr = path_option;

    while *opt_ptr != 0 {
        let buflen = nvim_path_copy_option_part(&raw mut opt_ptr, buf, MAXPATHL, c" ,".as_ptr());
        let mut final_buflen = buflen;

        if *buf as u8 == b'.' && (*buf.add(1) == 0 || rs_vim_ispathsep(*buf.add(1) as c_int) != 0) {
            // Relative to current buffer:
            let ffname = nvim_path_curbuf_ffname();
            if ffname.is_null() {
                continue;
            }
            let p = rs_path_tail(ffname);
            let plen = p as usize - ffname as usize;
            if plen + libc::strlen(buf) >= MAXPATHL {
                continue;
            }
            if *buf.add(1) == 0 {
                *buf.add(plen) = 0;
            } else {
                std::ptr::copy(buf.add(2), buf.add(plen), (buflen - 2) + 1);
            }
            std::ptr::copy(ffname as *const u8, buf as *mut u8, plen);
            final_buflen = rs_simplify_filename(buf);
        } else if *buf == 0 {
            // Relative to current directory.
            libc::strcpy(buf, curdir);
            if curdirlen == 0 {
                curdirlen = libc::strlen(curdir);
            }
            final_buflen = curdirlen;
        } else if rs_path_with_url(buf) != 0 {
            continue;
        } else if rs_path_is_absolute(buf) == 0 {
            // Expand relative path to full path equivalent.
            if curdirlen == 0 {
                curdirlen = libc::strlen(curdir);
            }
            if curdirlen + buflen + 3 > MAXPATHL {
                continue;
            }
            std::ptr::copy(
                buf as *const u8,
                buf.add(curdirlen + 1) as *mut u8,
                buflen + 1,
            );
            libc::strcpy(buf, curdir);
            *buf.add(curdirlen) = PATHSEP as c_char;
            final_buflen = rs_simplify_filename(buf);
        }

        let dup = nvim_path_xmemdupz(buf, final_buflen);
        nvim_path_ga_append_string(gap, dup);
    }

    nvim_path_xfree(buf);
}

// ---------------------------------------------------------------------------
// rs_uniquefy_paths
// ---------------------------------------------------------------------------

/// Make fullpath names in "gap" unique while conserving pattern matches.
///
/// Sorts, removes duplicates and modifies all the fullpath names in "gap" so
/// that they are unique with respect to each other while conserving the part
/// that matches the pattern. Beware, this is at least O(n^2) wrt "gap->ga_len".
///
/// # Safety
/// All pointers must be valid. `gap` must be a valid garray_T of `char *`.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_uniquefy_paths(
    gap: *mut c_void,
    pattern: *mut c_char,
    path_option: *mut c_char,
) {
    let mut sort_again = false;

    nvim_path_ga_remove_duplicate_strings(gap);
    let path_ga = nvim_path_ga_alloc_strings(1);

    // Prepend '*' to pattern for regex matching anywhere in the path.
    let len = libc::strlen(pattern);
    let file_pattern: *mut c_char = nvim_path_xmalloc(len + 2) as *mut c_char;
    *file_pattern = b'*' as c_char;
    *file_pattern.add(1) = 0;
    libc::strcpy(file_pattern.add(1), pattern);
    let pat =
        nvim_path_file_pat_to_reg_pat(file_pattern, std::ptr::null(), std::ptr::null_mut(), 0);
    nvim_path_xfree(file_pattern);
    if pat.is_null() {
        nvim_path_ga_clear_strings(path_ga);
        nvim_path_ga_free_handle(path_ga);
        return;
    }

    // Compile the regex with ignore-case.
    let reghandle = nvim_path_compile_pattern(pat, RE_MAGIC + RE_STRING, 1);
    nvim_path_xfree(pat);
    if reghandle.is_null() {
        nvim_path_ga_clear_strings(path_ga);
        nvim_path_ga_free_handle(path_ga);
        return;
    }

    let curdir: *mut c_char = nvim_path_xmalloc(MAXPATHL) as *mut c_char;
    nvim_path_os_dirname(curdir, MAXPATHL);
    rs_expand_path_option(curdir, path_option, path_ga);

    let ga_len = nvim_path_ga_len(gap);
    let in_curdir: *mut *mut c_char =
        nvim_path_xcalloc(ga_len as usize, std::mem::size_of::<*mut c_char>()) as *mut *mut c_char;

    // First loop: shorten paths while maintaining uniqueness.
    let mut i = 0;
    while i < nvim_path_ga_len(gap) && nvim_path_get_got_int() == 0 {
        let fnames = nvim_path_ga_get_data(gap);
        let path = *fnames.add(i as usize);
        let dir_end = rs_gettail_dir(path);

        let path_len = libc::strlen(path);
        let dir_end_offset = dir_end as usize - path as usize;
        let is_in_curdir = rs_path_fnamencmp(curdir, path, dir_end_offset) == 0
            && *curdir.add(dir_end_offset) == 0;
        if is_in_curdir {
            *in_curdir.add(i as usize) = nvim_path_xmemdupz(path, path_len);
        }

        // Shorten the filename while maintaining its uniqueness.
        let path_cutoff = rs_get_path_cutoff(path, path_ga);

        if *pattern as u8 == b'*'
            && *pattern.add(1) as u8 == b'*'
            && rs_vim_ispathsep_nocolon(*pattern.add(2) as c_int) != 0
            && !path_cutoff.is_null()
            && nvim_path_match_pattern(reghandle, path_cutoff) != 0
            && rs_is_unique(path_cutoff, gap, i) != 0
        {
            sort_again = true;
            let cutoff_len = libc::strlen(path_cutoff);
            std::ptr::copy(path_cutoff as *const u8, path as *mut u8, cutoff_len + 1);
        } else {
            // Get shortest unique path starting from the end.
            let mut pathsep_p: *mut c_char = path.add(path_len.wrapping_sub(1));
            while rs_find_previous_pathsep(path, &raw mut pathsep_p) == OK {
                let after_sep = pathsep_p.add(1);
                if nvim_path_match_pattern(reghandle, after_sep) != 0
                    && rs_is_unique(after_sep, gap, i) != 0
                    && !path_cutoff.is_null()
                    && after_sep as usize >= path_cutoff as usize
                {
                    sort_again = true;
                    let move_len = (path as usize + path_len) - after_sep as usize + 1;
                    std::ptr::copy(after_sep as *const u8, path as *mut u8, move_len);
                    break;
                }
            }
        }

        if rs_path_is_absolute(path) != 0 {
            let short_name = rs_path_shorten_fname(path, curdir);
            if !short_name.is_null() && short_name as usize > path as usize + 1 {
                format_relative(path, MAXPATHL, short_name);
            }
        }
        nvim_path_os_breakcheck();
        i += 1;
    }

    // Second loop: shorten filenames in the current directory.
    i = 0;
    while i < nvim_path_ga_len(gap) && nvim_path_get_got_int() == 0 {
        let fnames = nvim_path_ga_get_data(gap);
        let path = *in_curdir.add(i as usize);
        if path.is_null() {
            i += 1;
            continue;
        }

        let mut short_name = rs_path_shorten_fname(path, curdir);
        if short_name.is_null() {
            short_name = path;
        }
        if rs_is_unique(short_name, gap, i) != 0 {
            libc::strcpy(*fnames.add(i as usize), short_name);
            i += 1;
            continue;
        }

        let slen = libc::strlen(short_name);
        // "./" + short_name + NUL
        let rel_pathsize: usize = 1 + 1 + slen + 1;
        let rel_path: *mut c_char = nvim_path_xmalloc(rel_pathsize) as *mut c_char;
        format_relative(rel_path, rel_pathsize, short_name);

        nvim_path_xfree(*fnames.add(i as usize));
        nvim_path_ga_set_string(gap, i, rel_path);
        sort_again = true;
        nvim_path_os_breakcheck();
        i += 1;
    }

    nvim_path_xfree(curdir);
    let final_ga_len = nvim_path_ga_len(gap);
    for j in 0..final_ga_len {
        let p = *in_curdir.add(j as usize);
        if !p.is_null() {
            nvim_path_xfree(p);
        }
    }
    nvim_path_xfree(in_curdir as *mut c_char);
    nvim_path_ga_clear_strings(path_ga);
    nvim_path_ga_free_handle(path_ga);
    nvim_path_free_pattern(reghandle);

    if sort_again {
        nvim_path_ga_remove_duplicate_strings(gap);
    }
}

// ---------------------------------------------------------------------------
// rs_gen_expand_wildcards
// ---------------------------------------------------------------------------

/// Static recursion guard for `gen_expand_wildcards`.
static mut GEN_EXPAND_RECURSIVE: bool = false;

/// Expand wildcards for a list of patterns, collecting matches in a growarray.
///
/// # Safety
/// All pointer arguments must be valid. `pat` must point to an array of
/// `num_pat` valid C strings.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_gen_expand_wildcards(
    num_pat: c_int,
    pat: *mut *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
    flags: c_int,
) -> c_int {
    let mut did_expand_in_path = false;
    let path_option = nvim_path_get_path_option();

    // If already recursing, delegate to os_expand_wildcards (Unix) or fail.
    if GEN_EXPAND_RECURSIVE {
        #[cfg(unix)]
        {
            return nvim_path_os_expand_wildcards(num_pat, pat, num_file, file, flags);
        }
        #[cfg(not(unix))]
        {
            return FAIL;
        }
    }

    // Check for special wildcard characters that need the shell.
    #[cfg(unix)]
    {
        for idx in 0..num_pat {
            let p = *pat.add(idx as usize);
            if rs_has_special_wildchar(p, flags) != 0
                && !(rs_vim_backtick(p) != 0 && *p.add(1) as u8 == b'=')
            {
                return nvim_path_os_expand_wildcards(num_pat, pat, num_file, file, flags);
            }
        }
    }

    GEN_EXPAND_RECURSIVE = true;

    let ga = nvim_path_ga_alloc_strings(30);

    let mut i = 0;
    while i < num_pat && nvim_path_get_got_int() == 0 {
        let mut add_pat: c_int = -1;
        let mut p: *mut c_char = *pat.add(i as usize);

        if rs_vim_backtick(p) != 0 {
            add_pat = nvim_path_expand_backtick(ga, p, flags);
            if add_pat == -1 {
                GEN_EXPAND_RECURSIVE = false;
                nvim_path_ga_clear_strings(ga);
                nvim_path_ga_free_handle(ga);
                *num_file = 0;
                *file = std::ptr::null_mut();
                return FAIL;
            }
        } else {
            // First expand environment variables, "~/" and "~user/".
            if (rs_has_env_var(p) != 0 && (flags & EW_NOTENV) == 0) || *p as u8 == b'~' {
                p = nvim_path_expand_env_save_opt(p, 1);
                if p.is_null() {
                    p = *pat.add(i as usize);
                } else {
                    #[cfg(unix)]
                    {
                        // On Unix, if expand_env() can't expand an environment
                        // variable, use the shell to do that.
                        if rs_has_env_var(p) != 0 || *p as u8 == b'~' {
                            nvim_path_xfree(p);
                            nvim_path_ga_clear_strings(ga);
                            nvim_path_ga_free_handle(ga);
                            let result = nvim_path_os_expand_wildcards(
                                num_pat,
                                pat,
                                num_file,
                                file,
                                flags | EW_KEEPDOLLAR,
                            );
                            GEN_EXPAND_RECURSIVE = false;
                            return result;
                        }
                    }
                }
            }

            // If there are wildcards or case-insensitive expansion is
            // required: Expand file names and add each match to the list.
            if rs_path_has_exp_wildcard(p) != 0 || (flags & EW_ICASE) != 0 {
                GEN_EXPAND_RECURSIVE = false;
                if (flags & (EW_PATH | EW_CDPATH)) != 0
                    && rs_path_is_absolute(p) == 0
                    && !(*p as u8 == b'.'
                        && (rs_vim_ispathsep(*p.add(1) as c_int) != 0
                            || (*p.add(1) as u8 == b'.'
                                && rs_vim_ispathsep(*p.add(2) as c_int) != 0)))
                {
                    // :find completion where 'path' is used.
                    add_pat = nvim_path_expand_in_path(ga, p, flags);
                    did_expand_in_path = true;
                } else {
                    let tmp_add_pat = rs_path_expand(ga, p, flags);
                    add_pat = tmp_add_pat as c_int;
                }
                GEN_EXPAND_RECURSIVE = true;
            }
        }

        if add_pat == -1 || (add_pat == 0 && (flags & EW_NOTFOUND) != 0) {
            let t = nvim_path_backslash_halve_save(p);

            if (flags & EW_NOTFOUND) != 0 {
                rs_addfile(ga, t, flags | EW_DIR | EW_FILE);
            } else {
                rs_addfile(ga, t, flags);
            }

            if t != p {
                nvim_path_xfree(t);
            }
        }

        if did_expand_in_path && nvim_path_ga_len(ga) > 0 && (flags & (EW_PATH | EW_CDPATH)) != 0 {
            GEN_EXPAND_RECURSIVE = false;
            rs_uniquefy_paths(ga, p, path_option.cast_mut());
            GEN_EXPAND_RECURSIVE = true;
        }
        if p != *pat.add(i as usize) {
            nvim_path_xfree(p);
        }
        i += 1;
    }

    *num_file = nvim_path_ga_len(ga);
    let data = nvim_path_ga_get_data_ptr(ga);
    *file = if data.is_null() {
        std::ptr::null_mut()
    } else {
        data as *mut *mut c_char
    };

    GEN_EXPAND_RECURSIVE = false;

    // Free the garray handle but NOT the data (it's returned to the caller).
    // We need to be careful: ga_free_handle just frees the struct, not ga_data.
    nvim_path_ga_free_handle(ga);

    if (flags & EW_EMPTYOK) != 0 || !(*file).is_null() {
        OK
    } else {
        FAIL
    }
}

// ---------------------------------------------------------------------------
// rs_expand_wildcards
// ---------------------------------------------------------------------------

/// Expand wildcards. Calls `gen_expand_wildcards()` and removes files matching
/// 'wildignore', then moves files matching 'suffixes' to the end.
///
/// # Safety
/// All pointer arguments must be valid. `pat` must point to an array of
/// `num_pat` valid C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_expand_wildcards(
    num_pat: c_int,
    pat: *mut *mut c_char,
    num_files: *mut c_int,
    files: *mut *mut *mut c_char,
    flags: c_int,
) -> c_int {
    let retval = rs_gen_expand_wildcards(num_pat, pat, num_files, files, flags);

    // When keeping all matches, return here.
    if (flags & EW_KEEPALL) != 0 || retval == FAIL {
        return retval;
    }

    // Remove names that match 'wildignore'.
    let p_wig = nvim_path_get_p_wig();
    if !p_wig.is_null() && *p_wig != 0 {
        let mut i: c_int = 0;
        while i < *num_files {
            let cur_name = *(*files).add(i as usize);
            let full_name = rs_FullName_save(cur_name, 0);
            if !full_name.is_null() && nvim_path_match_file_list(p_wig, cur_name, full_name) != 0 {
                // Remove this matching file from the list.
                nvim_path_xfree(cur_name);
                let mut j = i;
                while j + 1 < *num_files {
                    *(*files).add(j as usize) = *(*files).add((j + 1) as usize);
                    j += 1;
                }
                *num_files -= 1;
                i -= 1;
            }
            if !full_name.is_null() {
                nvim_path_xfree(full_name);
            }
            i += 1;
        }
    }

    // Move the names where 'suffixes' match to the end.
    if *num_files > 1 && nvim_path_get_got_int() == 0 {
        let mut non_suf_match: c_int = 0;
        for i in 0..*num_files {
            if rs_match_suffix(*(*files).add(i as usize)) == 0 {
                // Move the name without matching suffix to the front.
                let p = *(*files).add(i as usize);
                let mut j = i;
                while j > non_suf_match {
                    *(*files).add(j as usize) = *(*files).add((j - 1) as usize);
                    j -= 1;
                }
                *(*files).add(non_suf_match as usize) = p;
                non_suf_match += 1;
            }
        }
    }

    // Free empty array of matches.
    if *num_files == 0 {
        if !(*files).is_null() {
            nvim_path_xfree(*files as *mut c_char);
            *files = std::ptr::null_mut();
        }
        return FAIL;
    }

    retval
}
