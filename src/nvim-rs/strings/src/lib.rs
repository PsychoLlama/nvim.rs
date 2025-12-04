//! String utilities for Neovim
//!
//! Provides string manipulation functions compatible with nvim's strings.c.
//!
//! Key functions:
//! - `vim_stricmp` - Case-insensitive string comparison (ASCII)
//! - `vim_strnicmp` - Case-insensitive string comparison with length limit (ASCII)
//! - `vim_strchr` - Find character in string (handles multibyte for ASCII range)

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::as_ptr_cast_mut)]

use std::ffi::{c_char, c_int};

/// Convert ASCII uppercase to lowercase.
#[inline]
fn tolower_asc(c: u8) -> u8 {
    if c.is_ascii_uppercase() {
        c + (b'a' - b'A')
    } else {
        c
    }
}

/// Compare two strings, ignoring case (ASCII only).
///
/// This is a locale-independent comparison that only handles ASCII characters.
/// Returns 0 for match, < 0 if s1 < s2, > 0 if s1 > s2.
///
/// # Safety
///
/// Both `s1` and `s2` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    if s1.is_null() || s2.is_null() {
        if s1.is_null() && s2.is_null() {
            return 0;
        }
        return if s1.is_null() { -1 } else { 1 };
    }

    let mut p1 = s1;
    let mut p2 = s2;

    loop {
        let c1 = unsafe { *p1 as u8 };
        let c2 = unsafe { *p2 as u8 };

        let diff = c_int::from(tolower_asc(c1)) - c_int::from(tolower_asc(c2));
        if diff != 0 {
            return diff;
        }
        if c1 == 0 {
            break;
        }
        p1 = unsafe { p1.add(1) };
        p2 = unsafe { p2.add(1) };
    }

    0
}

/// Compare two strings for a given length, ignoring case (ASCII only).
///
/// This is a locale-independent comparison that only handles ASCII characters.
/// Returns 0 for match, < 0 if s1 < s2, > 0 if s1 > s2.
///
/// # Safety
///
/// Both `s1` and `s2` must be valid pointers to at least `len` bytes,
/// or null-terminated before `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_strnicmp(
    s1: *const c_char,
    s2: *const c_char,
    len: usize,
) -> c_int {
    if len == 0 {
        return 0;
    }

    if s1.is_null() || s2.is_null() {
        if s1.is_null() && s2.is_null() {
            return 0;
        }
        return if s1.is_null() { -1 } else { 1 };
    }

    let mut p1 = s1;
    let mut p2 = s2;
    let mut remaining = len;

    while remaining > 0 {
        let c1 = unsafe { *p1 as u8 };
        let c2 = unsafe { *p2 as u8 };

        let diff = c_int::from(tolower_asc(c1)) - c_int::from(tolower_asc(c2));
        if diff != 0 {
            return diff;
        }
        if c1 == 0 {
            break;
        }
        p1 = unsafe { p1.add(1) };
        p2 = unsafe { p2.add(1) };
        remaining -= 1;
    }

    0
}

/// Check if two strings are equal, ignoring case (ASCII only).
///
/// Returns true if the strings are equal (case-insensitive), false otherwise.
/// NULL strings are considered equal to each other.
///
/// # Safety
///
/// Both `s1` and `s2` must be valid null-terminated C strings, or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_striequal(s1: *const c_char, s2: *const c_char) -> c_int {
    c_int::from(unsafe { rs_vim_stricmp(s1, s2) } == 0)
}

/// Find a character in a string.
///
/// For ASCII characters (< 128), uses standard strchr.
/// For non-ASCII characters, this simplified version returns NULL
/// (full multibyte support requires mbyte functions).
///
/// Returns NULL if:
/// - The string is NULL
/// - The character is <= 0
/// - The character is not found
/// - The character is >= 128 (multibyte not supported in this version)
///
/// # Safety
///
/// `string` must be a valid null-terminated C string, or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_strchr(string: *const c_char, c: c_int) -> *const c_char {
    if string.is_null() || c <= 0 {
        return std::ptr::null();
    }

    // For ASCII characters, use simple search
    if c < 0x80 {
        let mut p = string;
        let target = c as u8;
        loop {
            let ch = unsafe { *p as u8 };
            if ch == 0 {
                return std::ptr::null();
            }
            if ch == target {
                return p;
            }
            p = unsafe { p.add(1) };
        }
    }

    // For non-ASCII, we'd need utf_char2bytes which is in mbyte
    // Return NULL for now - full support will be added when mbyte is migrated
    std::ptr::null()
}

/// Check if a string contains a non-ASCII character (128 or higher).
///
/// Returns 1 if the string contains non-ASCII, 0 otherwise.
/// Returns 0 if the string is NULL.
///
/// # Safety
///
/// `s` must be a valid null-terminated C string, or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_has_non_ascii(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }

    let mut p = s;
    loop {
        let c = unsafe { *p as u8 };
        if c == 0 {
            break;
        }
        if c >= 128 {
            return 1;
        }
        p = unsafe { p.add(1) };
    }

    0
}

/// Concatenate two strings into a newly allocated buffer.
///
/// Returns a pointer to a newly allocated string containing s1 + s2.
/// The caller is responsible for freeing the memory.
///
/// Returns NULL if both inputs are NULL or if allocation fails.
///
/// # Safety
///
/// `s1` and `s2` must be valid null-terminated C strings, or NULL.
/// The returned pointer must be freed with the appropriate allocator.
#[no_mangle]
pub unsafe extern "C" fn rs_concat_str(s1: *const c_char, s2: *const c_char) -> *mut c_char {
    let len1 = if s1.is_null() {
        0
    } else {
        unsafe { libc::strlen(s1) }
    };
    let len2 = if s2.is_null() {
        0
    } else {
        unsafe { libc::strlen(s2) }
    };

    if len1 == 0 && len2 == 0 {
        return std::ptr::null_mut();
    }

    let total_len = len1 + len2 + 1;
    let result = unsafe { libc::malloc(total_len) as *mut c_char };
    if result.is_null() {
        return std::ptr::null_mut();
    }

    if len1 > 0 {
        unsafe { std::ptr::copy_nonoverlapping(s1, result, len1) };
    }
    if len2 > 0 {
        unsafe { std::ptr::copy_nonoverlapping(s2, result.add(len1), len2) };
    }
    unsafe { *result.add(len1 + len2) = 0 };

    result
}

/// Sort an array of strings using strcmp.
///
/// # Safety
///
/// `files` must be a valid pointer to an array of at least `count` string pointers.
/// Each string must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sort_strings(files: *mut *mut c_char, count: c_int) {
    if files.is_null() || count <= 0 {
        return;
    }

    let slice = unsafe { std::slice::from_raw_parts_mut(files, count as usize) };
    slice.sort_by(|a, b| {
        let a_cstr = unsafe { std::ffi::CStr::from_ptr(*a) };
        let b_cstr = unsafe { std::ffi::CStr::from_ptr(*b) };
        a_cstr.cmp(b_cstr)
    });
}

/// Check if a character is ASCII alphanumeric.
#[inline]
fn ascii_isalnum(c: u8) -> bool {
    c.is_ascii_alphanumeric()
}

/// Check if a string is a valid name: only alphanumeric ASCII or allowed characters.
///
/// Returns true if `val` consists only of alphanumeric ASCII characters
/// or characters that appear in `allowed`.
///
/// # Safety
///
/// Both `val` and `allowed` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_valid_name(val: *const c_char, allowed: *const c_char) -> c_int {
    if val.is_null() {
        return 1; // Empty/null is considered valid
    }

    let mut s = val;
    loop {
        let c = *s as u8;
        if c == 0 {
            break;
        }

        // Check if alphanumeric
        if !ascii_isalnum(c) {
            // Check if in allowed set
            if allowed.is_null() || rs_vim_strchr(allowed, c_int::from(c)).is_null() {
                return 0;
            }
        }

        s = s.add(1);
    }

    1
}

/// Skip over the name of a TTY option or keycode option.
///
/// Returns a pointer to the character after the option name, or NULL if
/// the option is not a TTY or keycode option.
///
/// TTY options are: "term", "ttytype", "t_XX", or "<t_XX>" style keycodes.
///
/// # Safety
///
/// `arg` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_find_tty_option_end(arg: *const c_char) -> *const c_char {
    if arg.is_null() {
        return std::ptr::null();
    }

    // Check for "term"
    let term = b"term\0";
    if strequal_bytes(arg, term) {
        return arg.add(4); // length of "term"
    }

    // Check for "ttytype"
    let ttytype = b"ttytype\0";
    if strequal_bytes(arg, ttytype) {
        return arg.add(7); // length of "ttytype"
    }

    let mut p = arg;
    let mut delimit = false;

    // Check for <t_XX> style
    if *arg as u8 == b'<' {
        delimit = true;
        p = p.add(1);
    }

    // Check for t_XX pattern
    if *p as u8 == b't' && *p.add(1) as u8 == b'_' && *p.add(2) != 0 && *p.add(3) != 0 {
        p = p.add(4);
    } else if delimit {
        // Search for delimiting >
        while *p != 0 && *p as u8 != b'>' {
            p = p.add(1);
        }
    }

    // Return NULL when delimiting > is not found
    if delimit {
        if *p as u8 != b'>' {
            return std::ptr::null();
        }
        p = p.add(1);
    }

    if arg == p {
        std::ptr::null()
    } else {
        p
    }
}

/// Helper to compare C string with byte literal
#[inline]
fn strequal_bytes(s: *const c_char, bytes: &[u8]) -> bool {
    unsafe {
        for (i, &b) in bytes.iter().enumerate() {
            if b == 0 {
                return true; // Reached end of literal, strings match up to here
            }
            if *s.add(i) as u8 != b {
                return false;
            }
        }
        true
    }
}

/// Check if an option name is a TTY option.
///
/// Returns 1 if the option is a TTY option, 0 otherwise.
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_is_tty_option(name: *const c_char) -> c_int {
    c_int::from(!rs_find_tty_option_end(name).is_null())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_tolower_asc() {
        assert_eq!(tolower_asc(b'A'), b'a');
        assert_eq!(tolower_asc(b'Z'), b'z');
        assert_eq!(tolower_asc(b'a'), b'a');
        assert_eq!(tolower_asc(b'z'), b'z');
        assert_eq!(tolower_asc(b'0'), b'0');
        assert_eq!(tolower_asc(b' '), b' ');
    }

    #[test]
    fn test_vim_stricmp() {
        let hello1 = CString::new("Hello").unwrap();
        let hello2 = CString::new("HELLO").unwrap();
        let hello3 = CString::new("hello").unwrap();
        let world = CString::new("World").unwrap();
        let empty1 = CString::new("").unwrap();
        let empty2 = CString::new("").unwrap();

        unsafe {
            // Case-insensitive equality
            assert_eq!(rs_vim_stricmp(hello1.as_ptr(), hello2.as_ptr()), 0);
            assert_eq!(rs_vim_stricmp(hello1.as_ptr(), hello3.as_ptr()), 0);

            // Different strings
            assert!(rs_vim_stricmp(hello1.as_ptr(), world.as_ptr()) < 0);
            assert!(rs_vim_stricmp(world.as_ptr(), hello1.as_ptr()) > 0);

            // Empty strings
            assert_eq!(rs_vim_stricmp(empty1.as_ptr(), empty2.as_ptr()), 0);

            // NULL handling
            assert_eq!(rs_vim_stricmp(std::ptr::null(), std::ptr::null()), 0);
            assert!(rs_vim_stricmp(std::ptr::null(), hello1.as_ptr()) < 0);
            assert!(rs_vim_stricmp(hello1.as_ptr(), std::ptr::null()) > 0);
        }
    }

    #[test]
    fn test_vim_strnicmp() {
        let hello1 = CString::new("Hello World").unwrap();
        let hello2 = CString::new("HELLO UNIVERSE").unwrap();

        unsafe {
            // First 5 characters match (case-insensitive)
            assert_eq!(rs_vim_strnicmp(hello1.as_ptr(), hello2.as_ptr(), 5), 0);

            // First 6 characters differ (space vs space - same, but 7th differs)
            assert_eq!(rs_vim_strnicmp(hello1.as_ptr(), hello2.as_ptr(), 6), 0);
            assert!(rs_vim_strnicmp(hello1.as_ptr(), hello2.as_ptr(), 7) != 0);

            // Zero length always matches
            assert_eq!(rs_vim_strnicmp(hello1.as_ptr(), hello2.as_ptr(), 0), 0);

            // NULL handling
            assert_eq!(rs_vim_strnicmp(std::ptr::null(), std::ptr::null(), 5), 0);
        }
    }

    #[test]
    fn test_striequal() {
        let hello1 = CString::new("Hello").unwrap();
        let hello2 = CString::new("HELLO").unwrap();
        let world = CString::new("World").unwrap();

        unsafe {
            assert_eq!(rs_striequal(hello1.as_ptr(), hello2.as_ptr()), 1);
            assert_eq!(rs_striequal(hello1.as_ptr(), world.as_ptr()), 0);
            assert_eq!(rs_striequal(std::ptr::null(), std::ptr::null()), 1);
        }
    }

    #[test]
    fn test_vim_strchr() {
        let hello = CString::new("Hello, World!").unwrap();

        unsafe {
            // Find 'o'
            let result = rs_vim_strchr(hello.as_ptr(), b'o' as c_int);
            assert!(!result.is_null());
            assert_eq!(*result as u8, b'o');

            // Find ','
            let result = rs_vim_strchr(hello.as_ptr(), b',' as c_int);
            assert!(!result.is_null());
            assert_eq!(*result as u8, b',');

            // Character not found
            let result = rs_vim_strchr(hello.as_ptr(), b'x' as c_int);
            assert!(result.is_null());

            // Invalid inputs
            assert!(rs_vim_strchr(std::ptr::null(), b'o' as c_int).is_null());
            assert!(rs_vim_strchr(hello.as_ptr(), 0).is_null());
            assert!(rs_vim_strchr(hello.as_ptr(), -1).is_null());

            // Non-ASCII (not supported yet)
            assert!(rs_vim_strchr(hello.as_ptr(), 0x80).is_null());
        }
    }

    #[test]
    fn test_has_non_ascii() {
        let ascii = CString::new("Hello, World!").unwrap();
        let non_ascii = CString::new("Hello, 世界!").unwrap();
        let empty = CString::new("").unwrap();

        unsafe {
            assert_eq!(rs_has_non_ascii(ascii.as_ptr()), 0);
            assert_eq!(rs_has_non_ascii(non_ascii.as_ptr()), 1);
            assert_eq!(rs_has_non_ascii(empty.as_ptr()), 0);
            assert_eq!(rs_has_non_ascii(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_concat_str() {
        let hello = CString::new("Hello, ").unwrap();
        let world = CString::new("World!").unwrap();

        unsafe {
            let result = rs_concat_str(hello.as_ptr(), world.as_ptr());
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "Hello, World!");
            libc::free(result as *mut libc::c_void);

            // NULL handling
            let result = rs_concat_str(hello.as_ptr(), std::ptr::null());
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "Hello, ");
            libc::free(result as *mut libc::c_void);

            let result = rs_concat_str(std::ptr::null(), world.as_ptr());
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "World!");
            libc::free(result as *mut libc::c_void);

            // Both NULL
            let result = rs_concat_str(std::ptr::null(), std::ptr::null());
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_sort_strings() {
        let s1 = CString::new("zebra").unwrap();
        let s2 = CString::new("apple").unwrap();
        let s3 = CString::new("mango").unwrap();

        // Need mutable pointers for the array
        let mut ptrs: [*mut c_char; 3] = [
            s1.as_ptr() as *mut c_char,
            s2.as_ptr() as *mut c_char,
            s3.as_ptr() as *mut c_char,
        ];

        unsafe {
            rs_sort_strings(ptrs.as_mut_ptr(), 3);

            let sorted: Vec<&str> = ptrs
                .iter()
                .map(|p| std::ffi::CStr::from_ptr(*p).to_str().unwrap())
                .collect();

            assert_eq!(sorted, vec!["apple", "mango", "zebra"]);
        }
    }

    #[test]
    fn test_valid_name() {
        let alphanumeric = CString::new("hello123").unwrap();
        let with_underscore = CString::new("hello_world").unwrap();
        let with_special = CString::new("hello!world").unwrap();
        let empty = CString::new("").unwrap();
        let allowed_underscore = CString::new("_").unwrap();
        let allowed_chars = CString::new("_-").unwrap();

        unsafe {
            // Pure alphanumeric is always valid
            assert_eq!(rs_valid_name(alphanumeric.as_ptr(), std::ptr::null()), 1);

            // Underscore not allowed by default
            assert_eq!(rs_valid_name(with_underscore.as_ptr(), std::ptr::null()), 0);

            // Underscore allowed when in allowed set
            assert_eq!(
                rs_valid_name(with_underscore.as_ptr(), allowed_underscore.as_ptr()),
                1
            );

            // Special char not allowed
            assert_eq!(rs_valid_name(with_special.as_ptr(), std::ptr::null()), 0);
            assert_eq!(
                rs_valid_name(with_special.as_ptr(), allowed_chars.as_ptr()),
                0
            );

            // Empty string is valid
            assert_eq!(rs_valid_name(empty.as_ptr(), std::ptr::null()), 1);

            // NULL is valid
            assert_eq!(rs_valid_name(std::ptr::null(), std::ptr::null()), 1);
        }
    }

    #[test]
    fn test_find_tty_option_end() {
        let term = CString::new("term").unwrap();
        let ttytype = CString::new("ttytype").unwrap();
        let t_xx = CString::new("t_ab").unwrap();
        let t_xx_delim = CString::new("<t_cd>").unwrap();
        let not_tty = CString::new("noterm").unwrap();
        let t_short = CString::new("t_a").unwrap();
        let delim_no_close = CString::new("<t_ab").unwrap();

        unsafe {
            // "term" should return pointer after "term"
            let result = rs_find_tty_option_end(term.as_ptr());
            assert!(!result.is_null());
            assert_eq!(result.offset_from(term.as_ptr()), 4);

            // "ttytype" should return pointer after "ttytype"
            let result = rs_find_tty_option_end(ttytype.as_ptr());
            assert!(!result.is_null());
            assert_eq!(result.offset_from(ttytype.as_ptr()), 7);

            // "t_ab" should return pointer after "t_ab"
            let result = rs_find_tty_option_end(t_xx.as_ptr());
            assert!(!result.is_null());
            assert_eq!(result.offset_from(t_xx.as_ptr()), 4);

            // "<t_cd>" should return pointer after ">"
            let result = rs_find_tty_option_end(t_xx_delim.as_ptr());
            assert!(!result.is_null());
            assert_eq!(result.offset_from(t_xx_delim.as_ptr()), 6);

            // "noterm" should return NULL
            let result = rs_find_tty_option_end(not_tty.as_ptr());
            assert!(result.is_null());

            // "t_a" (too short) should return NULL
            let result = rs_find_tty_option_end(t_short.as_ptr());
            assert!(result.is_null());

            // "<t_ab" (no closing >) should return NULL
            let result = rs_find_tty_option_end(delim_no_close.as_ptr());
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_is_tty_option() {
        let term = CString::new("term").unwrap();
        let ttytype = CString::new("ttytype").unwrap();
        let t_xx = CString::new("t_ab").unwrap();
        let not_tty = CString::new("noterm").unwrap();

        unsafe {
            assert_eq!(rs_is_tty_option(term.as_ptr()), 1);
            assert_eq!(rs_is_tty_option(ttytype.as_ptr()), 1);
            assert_eq!(rs_is_tty_option(t_xx.as_ptr()), 1);
            assert_eq!(rs_is_tty_option(not_tty.as_ptr()), 0);
        }
    }
}
