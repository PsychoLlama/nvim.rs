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
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::items_after_statements)]

pub mod viml;

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

/// Convert ASCII lowercase to uppercase.
#[inline]
fn toupper_asc(c: u8) -> u8 {
    if c.is_ascii_lowercase() {
        c - (b'a' - b'A')
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
#[export_name = "striequal"]
pub unsafe extern "C" fn rs_striequal(s1: *const c_char, s2: *const c_char) -> bool {
    unsafe { rs_vim_stricmp(s1, s2) == 0 }
}

/// Convert a string to uppercase in-place (ASCII only).
///
/// This is a locale-independent conversion that only handles ASCII 'a'-'z'.
/// Non-ASCII characters are left unchanged.
///
/// # Safety
///
/// `p` must be a valid null-terminated C string.
#[export_name = "vim_strup"]
pub unsafe extern "C" fn rs_vim_strup(p: *mut c_char) {
    if p.is_null() {
        return;
    }

    let mut ptr = p as *mut u8;
    loop {
        let c = *ptr;
        if c == 0 {
            break;
        }
        *ptr = toupper_asc(c);
        ptr = ptr.add(1);
    }
}

/// Copy a string while converting to uppercase (ASCII only).
///
/// Copies from `src` to `dst`, converting lowercase a-z to uppercase A-Z.
/// The destination is null-terminated.
///
/// # Safety
///
/// - `dst` and `src` must be valid, non-overlapping pointers.
/// - `dst` must have enough space for `strlen(src) + 1` bytes.
/// - `src` must be a valid null-terminated C string.
#[export_name = "vim_strcpy_up"]
pub unsafe extern "C" fn rs_vim_strcpy_up(dst: *mut c_char, src: *const c_char) {
    if dst.is_null() || src.is_null() {
        return;
    }

    let mut d = dst as *mut u8;
    let mut s = src as *const u8;

    loop {
        let c = *s;
        if c == 0 {
            *d = 0;
            break;
        }
        *d = toupper_asc(c);
        d = d.add(1);
        s = s.add(1);
    }
}

/// Copy a string with length limit while converting to uppercase (ASCII only).
///
/// Copies up to `n` characters from `src` to `dst`, converting lowercase a-z
/// to uppercase A-Z. The destination is always null-terminated.
///
/// # Safety
///
/// - `dst` and `src` must be valid, non-overlapping pointers.
/// - `dst` must have enough space for `n + 1` bytes.
/// - `src` must be a valid null-terminated C string or at least `n` bytes.
#[export_name = "vim_strncpy_up"]
pub unsafe extern "C" fn rs_vim_strncpy_up(dst: *mut c_char, src: *const c_char, n: usize) {
    if dst.is_null() || src.is_null() {
        if !dst.is_null() {
            *(dst as *mut u8) = 0;
        }
        return;
    }

    let mut d = dst as *mut u8;
    let mut s = src as *const u8;
    let mut remaining = n;

    while remaining > 0 {
        let c = *s;
        if c == 0 {
            break;
        }
        *d = toupper_asc(c);
        d = d.add(1);
        s = s.add(1);
        remaining -= 1;
    }

    *d = 0;
}

/// Copy memory while converting to uppercase (ASCII only).
///
/// Copies exactly `n` bytes from `src` to `dst`, converting lowercase a-z
/// to uppercase A-Z. Does NOT null-terminate.
///
/// # Safety
///
/// - `dst` and `src` must be valid, non-overlapping pointers.
/// - `dst` must have enough space for `n` bytes.
/// - `src` must point to at least `n` bytes.
#[export_name = "vim_memcpy_up"]
pub unsafe extern "C" fn rs_vim_memcpy_up(dst: *mut c_char, src: *const c_char, n: usize) {
    if dst.is_null() || src.is_null() || n == 0 {
        return;
    }

    let mut d = dst as *mut u8;
    let mut s = src as *const u8;
    let mut remaining = n;

    while remaining > 0 {
        let c = *s;
        *d = toupper_asc(c);
        d = d.add(1);
        s = s.add(1);
        remaining -= 1;
    }
}

/// Delete trailing whitespace from a string.
///
/// Removes trailing spaces and tabs from the end of the string,
/// but only if the whitespace is not preceded by a backslash ('\\') or Ctrl_V (0x16).
/// This preserves intentionally escaped trailing whitespace.
///
/// Note: This function stops at the first character of the string. If the entire
/// string is whitespace (e.g. "   "), only the characters after the first will
/// be deleted (matches C behavior).
///
/// # Safety
///
/// `ptr` must be a valid null-terminated mutable C string.
#[export_name = "del_trailing_spaces"]
pub unsafe extern "C" fn rs_del_trailing_spaces(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    // Find the length of the string
    let p = ptr as *mut u8;
    let len = libc::strlen(ptr as *const libc::c_char);
    if len == 0 {
        return;
    }

    // Start at the end of the string (pointing to the NUL terminator)
    let mut q = p.add(len);
    const BACKSLASH: u8 = b'\\';
    const CTRL_V: u8 = 0x16; // ASCII 22

    // Match the C code: while (--q > ptr && ascii_iswhite(q[0]) && q[-1] != '\\' && q[-1] != Ctrl_V)
    loop {
        // First decrement q
        q = q.sub(1);

        // Then check if q > p (not at start of string)
        if q <= p {
            break;
        }

        // Check if current char is whitespace (space or tab)
        let c = *q;
        if c != b' ' && c != b'\t' {
            break;
        }

        // Check if preceded by backslash or Ctrl_V
        let prev = *q.sub(1);
        if prev == BACKSLASH || prev == CTRL_V {
            break;
        }

        // Delete this trailing whitespace character
        *q = 0;
    }
}

/// Maximum bytes in a UTF-8 sequence (from nvim's macros)
const MB_MAXBYTES: usize = 6;

/// Find a character in a string.
///
/// For ASCII characters (< 128), uses simple byte search.
/// For non-ASCII characters, converts the codepoint to UTF-8 and uses strstr.
///
/// Returns NULL if:
/// - The string is NULL
/// - The character is <= 0
/// - The character is not found
///
/// # Safety
///
/// `string` must be a valid null-terminated C string, or NULL.
#[export_name = "vim_strchr"]
pub unsafe extern "C" fn rs_vim_strchr(string: *const c_char, c: c_int) -> *mut c_char {
    if string.is_null() || c <= 0 {
        return std::ptr::null_mut();
    }

    // For ASCII characters, use simple search
    if c < 0x80 {
        let mut p = string;
        let target = c as u8;
        loop {
            let ch = unsafe { *p as u8 };
            if ch == 0 {
                return std::ptr::null_mut();
            }
            if ch == target {
                return p.cast_mut();
            }
            p = unsafe { p.add(1) };
        }
    }

    // For non-ASCII, convert to UTF-8 and use strstr
    let mut u8char = [0u8; MB_MAXBYTES + 1];
    let len = nvim_mbyte::utf_char2bytes(c, &mut u8char);
    u8char[len] = 0; // null-terminate

    // Use libc strstr to find the UTF-8 sequence
    libc::strstr(string, u8char.as_ptr() as *const c_char) as *mut c_char
}

/// Check if a string contains a non-ASCII character (128 or higher).
///
/// Returns true if the string contains non-ASCII, false otherwise.
/// Returns false if the string is NULL.
///
/// # Safety
///
/// `s` must be a valid null-terminated C string, or NULL.
#[export_name = "has_non_ascii"]
pub unsafe extern "C" fn rs_has_non_ascii(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let mut p = s;
    loop {
        let c = unsafe { *p as u8 };
        if c == 0 {
            break;
        }
        if c >= 128 {
            return true;
        }
        p = unsafe { p.add(1) };
    }

    false
}

/// Check if string contains non-ASCII characters (with length limit).
///
/// Returns true if any byte has the high bit set within the first `len` bytes.
///
/// # Safety
///
/// `s` must be a valid pointer to at least `len` bytes.
#[export_name = "has_non_ascii_len"]
pub unsafe extern "C" fn rs_has_non_ascii_len(s: *const c_char, len: usize) -> bool {
    if s.is_null() || len == 0 {
        return false;
    }

    let bytes = unsafe { std::slice::from_raw_parts(s as *const u8, len) };
    for &b in bytes {
        if b >= 128 {
            return true;
        }
    }

    false
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
#[export_name = "concat_str"]
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

/// Like vim_strsave(), but make all characters uppercase.
///
/// Returns a newly allocated uppercase copy of the string.
/// This uses ASCII lower-to-upper case translation, language independent.
///
/// # Safety
///
/// `string` must be a valid null-terminated C string.
/// The returned pointer must be freed by the caller.
#[export_name = "vim_strsave_up"]
pub unsafe extern "C" fn rs_vim_strsave_up(string: *const c_char) -> *mut c_char {
    if string.is_null() {
        return std::ptr::null_mut();
    }

    let len = libc::strlen(string);
    let result = libc::malloc(len + 1) as *mut c_char;
    if result.is_null() {
        return std::ptr::null_mut();
    }

    rs_vim_strcpy_up(result, string);
    result
}

/// Like xstrnsave(), but make all characters uppercase.
///
/// Returns a newly allocated uppercase copy of up to `len` characters.
/// This uses ASCII lower-to-upper case translation, language independent.
///
/// # Safety
///
/// `string` must be a valid null-terminated C string or at least `len` bytes.
/// The returned pointer must be freed by the caller.
#[export_name = "vim_strnsave_up"]
pub unsafe extern "C" fn rs_vim_strnsave_up(string: *const c_char, len: usize) -> *mut c_char {
    if string.is_null() {
        return std::ptr::null_mut();
    }

    let result = libc::malloc(len + 1) as *mut c_char;
    if result.is_null() {
        return std::ptr::null_mut();
    }

    rs_vim_strncpy_up(result, string, len);
    result
}

/// Sort an array of strings using strcmp.
///
/// # Safety
///
/// `files` must be a valid pointer to an array of at least `count` string pointers.
/// Each string must be a valid null-terminated C string.
#[export_name = "sort_strings"]
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
/// This is a Rust-callable helper function.
pub fn valid_name(val: &[u8], allowed: &[u8]) -> bool {
    for &c in val {
        if c == 0 {
            break;
        }
        if !c.is_ascii_alphanumeric() && !allowed.contains(&c) {
            return false;
        }
    }
    true
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
/// TTY options are: "term", "ttytype", `t_XX`, or `<t_XX>` style keycodes.
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

/// Skip to the next part of an option argument: skip space and comma.
///
/// Returns a pointer to the first non-space, non-comma character.
/// If the string starts with a comma, it is skipped first.
/// Then any spaces are skipped.
///
/// # Safety
///
/// `p` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_to_option_part(p: *const c_char) -> *const c_char {
    let mut ptr = p;

    // Skip leading comma
    if *ptr as u8 == b',' {
        ptr = ptr.add(1);
    }

    // Skip spaces
    while *ptr as u8 == b' ' {
        ptr = ptr.add(1);
    }

    ptr
}

/// Replace all occurrences of character `c` with character `x` in a null-terminated string.
///
/// This modifies the string in place.
///
/// # Safety
///
/// - `str` must be a valid, mutable null-terminated C string
/// - `c` must not be NUL (0), as that would cause undefined behavior
#[no_mangle]
pub unsafe extern "C" fn rs_strchrsub(str: *mut c_char, c: c_char, x: c_char) {
    if str.is_null() || c == 0 {
        return;
    }

    let mut p = str as *mut u8;
    let target = c as u8;
    let replacement = x as u8;

    loop {
        let ch = *p;
        if ch == 0 {
            break;
        }
        if ch == target {
            *p = replacement;
        }
        p = p.add(1);
    }
}

/// Replace all occurrences of byte `c` with byte `x` in a memory region.
///
/// This modifies the memory region in place. Unlike strchrsub, this does not
/// stop at NUL bytes and can be used on binary data.
///
/// # Safety
///
/// - `data` must be a valid, mutable pointer to at least `len` bytes
/// - The memory region must not overlap with any other mutable references
#[no_mangle]
pub unsafe extern "C" fn rs_memchrsub(data: *mut c_char, c: c_char, x: c_char, len: usize) {
    if data.is_null() || len == 0 {
        return;
    }

    let target = c as u8;
    let replacement = x as u8;
    let bytes = std::slice::from_raw_parts_mut(data as *mut u8, len);

    for byte in bytes {
        if *byte == target {
            *byte = replacement;
        }
    }
}

/// Copy a NUL-terminated string from src to dst.
///
/// Returns a pointer to the terminating NUL character in dst.
/// This is the only difference with strcpy(), which returns dst.
///
/// WARNING: If copying takes place between objects that overlap, the behavior
/// is undefined.
///
/// This is the Neovim version of POSIX 2008 stpcpy(3).
///
/// # Safety
///
/// - `dst` must be a valid, mutable pointer with enough space for strlen(src) + 1 bytes
/// - `src` must be a valid NUL-terminated C string
/// - `dst` and `src` must not overlap
#[no_mangle]
pub unsafe extern "C" fn rs_xstpcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char {
    if dst.is_null() || src.is_null() {
        return dst;
    }

    let len = libc::strlen(src);
    std::ptr::copy_nonoverlapping(src, dst, len + 1);
    dst.add(len)
}

/// Copy at most maxlen bytes from src to dst.
///
/// If src is shorter than maxlen, zeros are written to the remaining bytes.
/// Returns a pointer to the first NUL character written, or &dst[maxlen] if
/// no NUL was written.
///
/// WARNING: xstpncpy will ALWAYS write maxlen bytes.
///
/// # Safety
///
/// - `dst` must be a valid, mutable pointer with space for at least maxlen bytes
/// - `src` must be a valid NUL-terminated C string or at least maxlen bytes
/// - `dst` and `src` must not overlap
#[no_mangle]
pub unsafe extern "C" fn rs_xstpncpy(
    dst: *mut c_char,
    src: *const c_char,
    maxlen: usize,
) -> *mut c_char {
    if dst.is_null() || src.is_null() || maxlen == 0 {
        return dst;
    }

    // Find NUL in src within maxlen
    let p = libc::memchr(src.cast(), 0, maxlen);
    if p.is_null() {
        std::ptr::copy_nonoverlapping(src, dst, maxlen);
        dst.add(maxlen)
    } else {
        let srclen = (p as usize) - (src as usize);
        std::ptr::copy_nonoverlapping(src, dst, srclen);
        std::ptr::write_bytes(dst.add(srclen), 0, maxlen - srclen);
        dst.add(srclen)
    }
}

/// Copy a NUL-terminated string into a sized buffer.
///
/// Compatible with *BSD strlcpy: the result is always a valid NUL-terminated
/// string that fits in the buffer (unless the buffer size is zero).
/// It does not pad out the result like strncpy() does.
///
/// Returns the length of src (may be greater than dsize - 1, indicating truncation).
///
/// # Safety
///
/// - `dst` must be a valid, mutable pointer with space for at least dsize bytes
/// - `src` must be a valid NUL-terminated C string
/// - `dst` and `src` must not overlap
#[no_mangle]
pub unsafe extern "C" fn rs_xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize {
    if src.is_null() {
        return 0;
    }

    let slen = libc::strlen(src);

    if dsize > 0 && !dst.is_null() {
        let copy_len = if slen < dsize { slen } else { dsize - 1 };
        std::ptr::copy_nonoverlapping(src, dst, copy_len);
        *dst.add(copy_len) = 0;
    }

    slen
}

/// Append src to string dst of size dsize.
///
/// At most dsize-1 characters will be copied. Always NUL terminates.
/// src and dst may overlap.
///
/// Returns the length of the resulting string as if dsize was unlimited
/// (may be greater than dsize - 1, indicating truncation).
///
/// # Safety
///
/// - `dst` must be a valid, mutable NUL-terminated string with space for at least dsize bytes
/// - `src` must be a valid NUL-terminated C string
/// - dsize must be > 0
#[no_mangle]
pub unsafe extern "C" fn rs_xstrlcat(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize {
    if dst.is_null() || src.is_null() || dsize == 0 {
        return 0;
    }

    let dlen = libc::strlen(dst);
    let slen = libc::strlen(src);

    if slen > dsize - dlen - 1 {
        // Need to truncate
        libc::memmove(dst.add(dlen).cast(), src.cast(), dsize - dlen - 1);
        *dst.add(dsize - 1) = 0;
    } else {
        // Can copy full string + NUL
        libc::memmove(dst.add(dlen).cast(), src.cast(), slen + 1);
    }

    slen + dlen
}

// External FFI functions from other Rust crates
extern "C" {
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    #[link_name = "rem_backslash"]
    fn rs_rem_backslash(s: *const c_char) -> bool;
}

/// Save a copy of a string with given length.
///
/// Returns a newly allocated string containing up to `len` bytes from `string`.
/// The result is always null-terminated.
///
/// # Safety
///
/// `string` must be a valid pointer to at least `len` bytes.
/// The returned pointer must be freed by the caller.
#[export_name = "xstrnsave"]
pub unsafe extern "C" fn rs_xstrnsave(string: *const c_char, len: usize) -> *mut c_char {
    if string.is_null() {
        return std::ptr::null_mut();
    }

    // Allocate len + 1 bytes for null terminator
    let result = libc::malloc(len + 1) as *mut c_char;
    if result.is_null() {
        return std::ptr::null_mut();
    }

    // Copy the string
    std::ptr::copy_nonoverlapping(string, result, len);
    // Null-terminate
    *result.add(len) = 0;

    result
}

/// Reverse text into allocated memory.
///
/// Properly handles multi-byte UTF-8 characters, reversing them as complete
/// code points rather than individual bytes.
///
/// Returns the allocated reversed string.
///
/// # Safety
///
/// `s` must be a valid null-terminated C string.
/// The returned pointer must be freed by the caller.
#[export_name = "reverse_text"]
pub unsafe extern "C" fn rs_reverse_text(s: *const c_char) -> *mut c_char {
    if s.is_null() {
        return std::ptr::null_mut();
    }

    let len = libc::strlen(s);
    let rev = libc::malloc(len + 1) as *mut c_char;
    if rev.is_null() {
        return std::ptr::null_mut();
    }

    let mut s_i: usize = 0;
    let mut rev_i = len;

    while s_i < len {
        let mb_len = utfc_ptr2len(s.add(s_i)) as usize;
        rev_i -= mb_len;
        std::ptr::copy_nonoverlapping(s.add(s_i), rev.add(rev_i), mb_len);
        s_i += mb_len;
    }

    *rev.add(len) = 0;
    rev
}

/// Replace all occurrences of `what` with `rep` in `src`.
///
/// If no replacement happens, returns NULL.
/// Otherwise returns a newly allocated string with all replacements made.
///
/// # Safety
///
/// All string parameters must be valid null-terminated C strings.
/// The returned pointer must be freed by the caller.
#[export_name = "strrep"]
pub unsafe extern "C" fn rs_strrep(
    src: *const c_char,
    what: *const c_char,
    rep: *const c_char,
) -> *mut c_char {
    if src.is_null() || what.is_null() || rep.is_null() {
        return std::ptr::null_mut();
    }

    let whatlen = libc::strlen(what);
    if whatlen == 0 {
        return std::ptr::null_mut();
    }

    // Count occurrences
    let mut count: usize = 0;
    let mut pos = libc::strstr(src, what);
    while !pos.is_null() {
        count += 1;
        pos = libc::strstr(pos.add(whatlen), what);
    }

    if count == 0 {
        return std::ptr::null_mut();
    }

    let srclen = libc::strlen(src);
    let replen = libc::strlen(rep);
    let new_len = srclen + count * replen - count * whatlen;

    let ret = libc::malloc(new_len + 1) as *mut c_char;
    if ret.is_null() {
        return std::ptr::null_mut();
    }

    let mut ptr = ret;
    let mut src_ptr = src;
    pos = libc::strstr(src_ptr, what);
    while !pos.is_null() {
        let idx = (pos as usize) - (src_ptr as usize);
        std::ptr::copy_nonoverlapping(src_ptr, ptr, idx);
        ptr = ptr.add(idx);
        std::ptr::copy_nonoverlapping(rep, ptr, replen);
        ptr = ptr.add(replen);
        src_ptr = pos.add(whatlen);
        pos = libc::strstr(src_ptr, what);
    }

    // Copy remaining
    let remaining = libc::strlen(src_ptr);
    std::ptr::copy_nonoverlapping(src_ptr, ptr, remaining);
    ptr = ptr.add(remaining);
    *ptr = 0;

    ret
}

/// Save a copy of a string with characters escaped.
///
/// Characters in `esc_chars` are preceded by `cc` (usually backslash).
/// If `bsl` is true, also escape characters where rem_backslash() would remove the backslash.
///
/// Properly handles multi-byte UTF-8 characters.
///
/// # Safety
///
/// All string parameters must be valid null-terminated C strings.
/// The returned pointer must be freed by the caller.
#[export_name = "vim_strsave_escaped_ext"]
pub unsafe extern "C" fn rs_vim_strsave_escaped_ext(
    string: *const c_char,
    esc_chars: *const c_char,
    cc: c_char,
    bsl: bool,
) -> *mut c_char {
    if string.is_null() || esc_chars.is_null() {
        return std::ptr::null_mut();
    }

    // First count the number of escape characters required
    let mut length: usize = 1; // count the trailing NUL
    let mut p = string;
    while *p != 0 {
        let l = utfc_ptr2len(p) as usize;
        if l > 1 {
            length += l; // count a multibyte char
            p = p.add(l);
            continue;
        }
        if !rs_vim_strchr(esc_chars, *p as u8 as c_int).is_null() || (bsl && rs_rem_backslash(p)) {
            length += 1; // count an escape character
        }
        length += 1; // count an ordinary char
        p = p.add(1);
    }

    let escaped_string = libc::malloc(length) as *mut c_char;
    if escaped_string.is_null() {
        return std::ptr::null_mut();
    }

    let mut p2 = escaped_string;
    p = string;
    while *p != 0 {
        let l = utfc_ptr2len(p) as usize;
        if l > 1 {
            std::ptr::copy_nonoverlapping(p, p2, l);
            p2 = p2.add(l);
            p = p.add(l);
            continue;
        }
        if !rs_vim_strchr(esc_chars, *p as u8 as c_int).is_null() || (bsl && rs_rem_backslash(p)) {
            *p2 = cc;
            p2 = p2.add(1);
        }
        *p2 = *p;
        p2 = p2.add(1);
        p = p.add(1);
    }
    *p2 = 0;

    escaped_string
}

/// Save a copy of a string with characters escaped.
///
/// Characters in `esc_chars` are preceded by a backslash.
///
/// # Safety
///
/// All string parameters must be valid null-terminated C strings.
/// The returned pointer must be freed by the caller.
#[export_name = "vim_strsave_escaped"]
pub unsafe extern "C" fn rs_vim_strsave_escaped(
    string: *const c_char,
    esc_chars: *const c_char,
) -> *mut c_char {
    rs_vim_strsave_escaped_ext(string, esc_chars, b'\\' as c_char, false)
}

/// Save a copy of an unquoted string.
///
/// Turns string like `a\bc"def\"ghi\\\n"jkl` into `a\bcdef"ghi\\njkl`, for use
/// in shell_build_argv: the only purpose of backslash is making next character
/// be treated literally inside the double quotes, if this character is
/// backslash or quote.
///
/// # Safety
///
/// `string` must be a valid pointer to at least `length` bytes.
/// The returned pointer must be freed by the caller.
#[export_name = "vim_strnsave_unquoted"]
pub unsafe extern "C" fn rs_vim_strnsave_unquoted(
    string: *const c_char,
    length: usize,
) -> *mut c_char {
    if string.is_null() {
        return std::ptr::null_mut();
    }

    // Helper macro to check escape condition
    fn escape_cond(p: *const c_char, inquote: bool, string_end: *const c_char) -> bool {
        unsafe {
            *p as u8 == b'\\'
                && inquote
                && p.add(1) < string_end
                && (*p.add(1) as u8 == b'\\' || *p.add(1) as u8 == b'"')
        }
    }

    // First pass: count result length
    let mut ret_length: usize = 0;
    let mut inquote = false;
    let string_end = string.add(length);
    let mut p = string;
    while p < string_end {
        if *p as u8 == b'"' {
            inquote = !inquote;
        } else if escape_cond(p, inquote, string_end) {
            ret_length += 1;
            p = p.add(1);
        } else {
            ret_length += 1;
        }
        p = p.add(1);
    }

    // Allocate result
    let ret = libc::malloc(ret_length + 1) as *mut c_char;
    if ret.is_null() {
        return std::ptr::null_mut();
    }

    // Second pass: copy with unquoting
    let mut rp = ret;
    inquote = false;
    p = string;
    while p < string_end {
        if *p as u8 == b'"' {
            inquote = !inquote;
        } else if escape_cond(p, inquote, string_end) {
            p = p.add(1);
            *rp = *p;
            rp = rp.add(1);
        } else {
            *rp = *p;
            rp = rp.add(1);
        }
        p = p.add(1);
    }
    *rp = 0;

    ret
}

// FFI declarations for mbyte functions
extern "C" {
    fn utf_ptr2len(p: *const c_char) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utf_char2len(c: c_int) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn mb_toupper(c: c_int) -> c_int;
    fn mb_tolower(c: c_int) -> c_int;
}

/// Type alias for case conversion function pointer.
/// Takes a codepoint and returns the converted codepoint.
pub type CaseConvertFn = unsafe extern "C" fn(c_int) -> c_int;

/// Convert a string to upper or lower case.
///
/// This function converts all characters in the string using the provided
/// case conversion function (typically mb_toupper or mb_tolower from C).
///
/// The result may be larger than the input if case conversion changes
/// byte lengths (e.g., some Unicode case conversions).
///
/// # Safety
///
/// - `orig` must be a valid null-terminated C string
/// - `case_fn` must be a valid function pointer (typically mb_toupper or mb_tolower)
/// - The returned pointer must be freed by the caller using xfree
#[no_mangle]
pub unsafe extern "C" fn rs_strcase_save(
    orig: *const c_char,
    case_fn: CaseConvertFn,
) -> *mut c_char {
    if orig.is_null() {
        return std::ptr::null_mut();
    }

    // Calculate initial length
    let mut orig_len = 0usize;
    let mut p = orig;
    while *p != 0 {
        orig_len += 1;
        p = p.add(1);
    }

    // Allocate result buffer (initial size = orig_len + 1 for NUL)
    let mut res = libc::malloc(orig_len + 1) as *mut c_char;
    if res.is_null() {
        return std::ptr::null_mut();
    }
    let mut res_capacity = orig_len;

    // Index in result string
    let mut res_index = 0usize;
    // Current position in original string
    p = orig;

    while *p != 0 {
        // Get UTF-8 character length and codepoint
        let char_len = utf_ptr2len(p) as usize;
        let codepoint = utf_ptr2char(p);

        // Handle invalid sequences: use byte value directly
        let c = if char_len == 0 || codepoint < 0 {
            *p as u8 as c_int
        } else {
            codepoint
        };

        // Apply case conversion
        let converted_char = case_fn(c);

        // Get byte length of new character
        let converted_len = utf_char2len(converted_char) as usize;

        // Check if we need more space
        if res_index + converted_len > res_capacity {
            // Need more space: allocate extra for the new character + NUL
            let new_capacity = res_index + converted_len + 1;
            let new_res = libc::realloc(res as *mut libc::c_void, new_capacity + 1) as *mut c_char;
            if new_res.is_null() {
                libc::free(res as *mut libc::c_void);
                return std::ptr::null_mut();
            }
            res = new_res;
            res_capacity = new_capacity;
        }

        // Write the converted character
        utf_char2bytes(converted_char, res.add(res_index));
        res_index += converted_len;

        // Move to next character
        let advance = if char_len > 0 { char_len } else { 1 };
        p = p.add(advance);
    }

    // NUL-terminate the result
    *res.add(res_index) = 0;
    res
}

/// Compare two ASCII strings for length `len`, ignoring case, ignoring locale.
///
/// # Safety
///
/// Both `s1` and `s2` must be valid pointers to at least `len` bytes.
#[export_name = "vim_strnicmp_asc"]
pub unsafe extern "C" fn rs_vim_strnicmp_asc(
    s1: *const c_char,
    s2: *const c_char,
    len: usize,
) -> c_int {
    unsafe { rs_vim_strnicmp(s1, s2, len) }
}

/// Make a copy of `orig` with all characters converted to upper or lower case.
///
/// When `upper` is true, converts to uppercase using `mb_toupper`.
/// When `upper` is false, converts to lowercase using `mb_tolower`.
///
/// # Safety
///
/// `orig` must be a valid null-terminated C string.
/// The returned pointer must be freed by the caller using xfree.
#[export_name = "strcase_save"]
pub unsafe extern "C" fn rs_strcase_save_export(orig: *const c_char, upper: bool) -> *mut c_char {
    let case_fn: CaseConvertFn = if upper { mb_toupper } else { mb_tolower };
    unsafe { rs_strcase_save(orig, case_fn) }
}

// =============================================================================
// keyvalue_T comparators
// =============================================================================

/// C-compatible repr for keyvalue_T (from strings.h)
#[repr(C)]
pub struct KeyValue {
    pub key: c_int,
    pub value: *mut c_char,
    pub length: usize,
}

/// Compare two keyvalue_T structs by case-sensitive value.
#[export_name = "cmp_keyvalue_value"]
pub unsafe extern "C" fn rs_cmp_keyvalue_value(a: *const KeyValue, b: *const KeyValue) -> c_int {
    let kv1 = unsafe { &*a };
    let kv2 = unsafe { &*b };
    unsafe { libc::strcmp(kv1.value, kv2.value) }
}

/// Compare two keyvalue_T structs by value with length.
#[export_name = "cmp_keyvalue_value_n"]
pub unsafe extern "C" fn rs_cmp_keyvalue_value_n(a: *const KeyValue, b: *const KeyValue) -> c_int {
    let kv1 = unsafe { &*a };
    let kv2 = unsafe { &*b };
    let len = kv1.length.max(kv2.length);
    unsafe { libc::strncmp(kv1.value, kv2.value, len) }
}

/// Compare two keyvalue_T structs by case-insensitive value.
#[export_name = "cmp_keyvalue_value_i"]
pub unsafe extern "C" fn rs_cmp_keyvalue_value_i(a: *const KeyValue, b: *const KeyValue) -> c_int {
    let kv1 = unsafe { &*a };
    let kv2 = unsafe { &*b };
    unsafe { libc::strcasecmp(kv1.value, kv2.value) }
}

/// Compare two keyvalue_T structs by case-insensitive value with length.
#[export_name = "cmp_keyvalue_value_ni"]
pub unsafe extern "C" fn rs_cmp_keyvalue_value_ni(a: *const KeyValue, b: *const KeyValue) -> c_int {
    let kv1 = unsafe { &*a };
    let kv2 = unsafe { &*b };
    let len = kv1.length.max(kv2.length);
    unsafe { libc::strncasecmp(kv1.value, kv2.value, len) }
}

// =============================================================================
// Shell Escaping
// =============================================================================

extern "C" {
    fn rs_csh_like_shell() -> c_int;
    fn rs_fish_like_shell() -> c_int;
    fn find_cmdline_var(src: *const c_char, usedlen: *mut usize) -> isize;
    fn mb_copy_char(fp: *mut *const c_char, tp: *mut *mut c_char);
}

/// Escape a string for use as a shell argument.
///
/// Wraps the string in single quotes and escapes special characters.
/// `do_special` enables escaping of `!` and `%`/`#` cmdline variables.
/// `do_newline` enables escaping of newlines.
///
/// # Safety
///
/// `string` must be a valid null-terminated C string.
/// Returns a newly allocated string that the caller must free.
#[export_name = "vim_strsave_shellescape"]
pub unsafe extern "C" fn rs_vim_strsave_shellescape(
    string: *const c_char,
    do_special: bool,
    do_newline: bool,
) -> *mut c_char {
    let csh_like = unsafe { rs_csh_like_shell() } != 0;
    let fish_like = unsafe { rs_fish_like_shell() } != 0;

    // First pass: count bytes needed.
    // strlen(string) + 2 quotes + NUL, then extra bytes per special char.
    let string_len = unsafe { libc::strlen(string) };
    let mut length: usize = string_len + 3;
    let mut p = string;
    unsafe {
        while *p != 0 {
            if *p as u8 == b'\'' {
                length += 3; // ' => '\''
            }
            if (*p as u8 == b'\n' && (csh_like || do_newline))
                || (*p as u8 == b'!' && (csh_like || do_special))
            {
                length += 1; // insert backslash
                if csh_like && do_special {
                    length += 1; // insert extra backslash for csh
                }
            }
            if do_special {
                let mut l: usize = 0;
                if find_cmdline_var(p, &raw mut l) >= 0 {
                    length += 1; // insert backslash
                    p = p.add(l);
                    continue;
                }
            }
            if *p as u8 == b'\\' && fish_like {
                length += 1; // insert backslash
            }
            // Advance by UTF-8 char length
            let char_len = utfc_ptr2len(p);
            let advance = if char_len > 0 { char_len as usize } else { 1 };
            p = p.add(advance);
        }
    }

    let escaped_string = unsafe { libc::malloc(length) as *mut c_char };
    if escaped_string.is_null() {
        return std::ptr::null_mut();
    }

    // Add opening single quote
    let mut d = escaped_string;
    unsafe {
        *d = b'\'' as c_char;
        d = d.add(1);
    }

    // Second pass: copy with escaping
    let mut p = string;
    unsafe {
        while *p != 0 {
            if *p as u8 == b'\'' {
                *d = b'\'' as c_char;
                d = d.add(1);
                *d = b'\\' as c_char;
                d = d.add(1);
                *d = b'\'' as c_char;
                d = d.add(1);
                *d = b'\'' as c_char;
                d = d.add(1);
                p = p.add(1);
                continue;
            }
            if (*p as u8 == b'\n' && (csh_like || do_newline))
                || (*p as u8 == b'!' && (csh_like || do_special))
            {
                *d = b'\\' as c_char;
                d = d.add(1);
                if csh_like && do_special {
                    *d = b'\\' as c_char;
                    d = d.add(1);
                }
                *d = *p;
                d = d.add(1);
                p = p.add(1);
                continue;
            }
            if do_special {
                let mut l: usize = 0;
                if find_cmdline_var(p, &raw mut l) >= 0 {
                    *d = b'\\' as c_char;
                    d = d.add(1);
                    std::ptr::copy_nonoverlapping(p, d, l);
                    d = d.add(l);
                    p = p.add(l);
                    continue;
                }
            }
            if *p as u8 == b'\\' && fish_like {
                *d = b'\\' as c_char;
                d = d.add(1);
                *d = *p;
                d = d.add(1);
                p = p.add(1);
                continue;
            }
            mb_copy_char(&raw mut p, &raw mut d);
        }

        // Add closing quote and NUL
        *d = b'\'' as c_char;
        d = d.add(1);
        *d = 0;
    }

    escaped_string
}

#[cfg(test)]
#[allow(clippy::cast_lossless)]
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
    fn test_toupper_asc() {
        assert_eq!(toupper_asc(b'a'), b'A');
        assert_eq!(toupper_asc(b'z'), b'Z');
        assert_eq!(toupper_asc(b'A'), b'A');
        assert_eq!(toupper_asc(b'Z'), b'Z');
        assert_eq!(toupper_asc(b'0'), b'0');
        assert_eq!(toupper_asc(b' '), b' ');
        assert_eq!(toupper_asc(0x80), 0x80); // Non-ASCII unchanged
    }

    #[test]
    fn test_vim_strup() {
        // Basic lowercase to uppercase
        let mut s = *b"hello\0";
        unsafe { rs_vim_strup(s.as_mut_ptr().cast()) };
        assert_eq!(&s[..5], b"HELLO");

        // Mixed case
        let mut s = *b"HeLLo WoRLD\0";
        unsafe { rs_vim_strup(s.as_mut_ptr().cast()) };
        assert_eq!(&s[..11], b"HELLO WORLD");

        // Already uppercase
        let mut s = *b"HELLO\0";
        unsafe { rs_vim_strup(s.as_mut_ptr().cast()) };
        assert_eq!(&s[..5], b"HELLO");

        // Empty string
        let mut s = *b"\0";
        unsafe { rs_vim_strup(s.as_mut_ptr().cast()) };
        assert_eq!(s[0], 0);

        // Numbers and symbols unchanged
        let mut s = *b"hello123!@#\0";
        unsafe { rs_vim_strup(s.as_mut_ptr().cast()) };
        assert_eq!(&s[..11], b"HELLO123!@#");

        // NULL handling - should not crash
        unsafe { rs_vim_strup(std::ptr::null_mut()) };
    }

    #[test]
    fn test_vim_strcpy_up() {
        // Basic copy with uppercase
        let src = b"hello\0";
        let mut dst = [0u8; 10];
        unsafe { rs_vim_strcpy_up(dst.as_mut_ptr().cast(), src.as_ptr().cast()) };
        assert_eq!(&dst[..6], b"HELLO\0");

        // Mixed case
        let src = b"HeLLo WoRLD\0";
        let mut dst = [0u8; 15];
        unsafe { rs_vim_strcpy_up(dst.as_mut_ptr().cast(), src.as_ptr().cast()) };
        assert_eq!(&dst[..12], b"HELLO WORLD\0");

        // Empty string
        let src = b"\0";
        let mut dst = [0u8; 5];
        unsafe { rs_vim_strcpy_up(dst.as_mut_ptr().cast(), src.as_ptr().cast()) };
        assert_eq!(dst[0], 0);

        // NULL handling - should not crash
        unsafe {
            let mut dst = [0u8; 5];
            rs_vim_strcpy_up(dst.as_mut_ptr().cast(), std::ptr::null());
            rs_vim_strcpy_up(std::ptr::null_mut(), src.as_ptr().cast());
        };
    }

    #[test]
    fn test_vim_strncpy_up() {
        // Copy with limit
        let src = b"hello world\0";
        let mut dst = [0u8; 10];
        unsafe { rs_vim_strncpy_up(dst.as_mut_ptr().cast(), src.as_ptr().cast(), 5) };
        assert_eq!(&dst[..6], b"HELLO\0");

        // Copy entire string (limit larger than string)
        let src = b"hi\0";
        let mut dst = [0u8; 10];
        unsafe { rs_vim_strncpy_up(dst.as_mut_ptr().cast(), src.as_ptr().cast(), 100) };
        assert_eq!(&dst[..3], b"HI\0");

        // Zero length
        let src = b"hello\0";
        let mut dst = [0u8; 10];
        dst[0] = b'X';
        unsafe { rs_vim_strncpy_up(dst.as_mut_ptr().cast(), src.as_ptr().cast(), 0) };
        assert_eq!(dst[0], 0); // Should write NUL

        // NULL handling
        unsafe {
            let mut dst = [0u8; 5];
            rs_vim_strncpy_up(dst.as_mut_ptr().cast(), std::ptr::null(), 5);
            assert_eq!(dst[0], 0); // Should write NUL
        };
    }

    #[test]
    fn test_vim_memcpy_up() {
        // Basic memcpy with uppercase (no NUL termination)
        let src = b"hello";
        let mut dst = [0u8; 10];
        unsafe { rs_vim_memcpy_up(dst.as_mut_ptr().cast(), src.as_ptr().cast(), 5) };
        assert_eq!(&dst[..5], b"HELLO");
        assert_eq!(dst[5], 0); // Unwritten bytes unchanged

        // Partial copy
        let src = b"hello world";
        let mut dst = [0u8; 10];
        unsafe { rs_vim_memcpy_up(dst.as_mut_ptr().cast(), src.as_ptr().cast(), 5) };
        assert_eq!(&dst[..5], b"HELLO");

        // Zero length - should not crash or write anything
        let src = b"hello";
        let mut dst = [b'X'; 5];
        unsafe { rs_vim_memcpy_up(dst.as_mut_ptr().cast(), src.as_ptr().cast(), 0) };
        assert_eq!(&dst, b"XXXXX");

        // NULL handling
        unsafe {
            rs_vim_memcpy_up(std::ptr::null_mut(), src.as_ptr().cast(), 5);
            let mut dst = [0u8; 5];
            rs_vim_memcpy_up(dst.as_mut_ptr().cast(), std::ptr::null(), 5);
        };
    }

    #[test]
    fn test_del_trailing_spaces() {
        // Basic trailing spaces removal
        let mut s = *b"hello   \0";
        unsafe { rs_del_trailing_spaces(s.as_mut_ptr().cast()) };
        assert_eq!(&s[..5], b"hello");
        assert_eq!(s[5], 0);

        // Trailing tabs removal
        let mut s = *b"hello\t\t\0";
        unsafe { rs_del_trailing_spaces(s.as_mut_ptr().cast()) };
        assert_eq!(&s[..5], b"hello");
        assert_eq!(s[5], 0);

        // Mixed spaces and tabs
        let mut s = *b"hello \t \0";
        unsafe { rs_del_trailing_spaces(s.as_mut_ptr().cast()) };
        assert_eq!(&s[..5], b"hello");
        assert_eq!(s[5], 0);

        // No trailing whitespace - should be unchanged
        let mut s = *b"hello\0xxx";
        unsafe { rs_del_trailing_spaces(s.as_mut_ptr().cast()) };
        assert_eq!(&s[..6], b"hello\0");

        // Escaped space with backslash - should NOT be deleted
        let mut s = *b"hello\\ \0";
        unsafe { rs_del_trailing_spaces(s.as_mut_ptr().cast()) };
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(s.as_ptr().cast()) }
                .to_str()
                .unwrap(),
            "hello\\ "
        );

        // Escaped space with Ctrl_V (0x16) - should NOT be deleted
        let mut s = [b'h', b'e', b'l', b'l', b'o', 0x16, b' ', 0];
        unsafe { rs_del_trailing_spaces(s.as_mut_ptr().cast()) };
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(s.as_ptr().cast()) }
                .to_str()
                .unwrap()
                .len(),
            7 // h, e, l, l, o, Ctrl_V, space
        );

        // Empty string
        let mut s = *b"\0";
        unsafe { rs_del_trailing_spaces(s.as_mut_ptr().cast()) };
        assert_eq!(s[0], 0);

        // All whitespace - first char stays (C behavior: --q > ptr stops at first char)
        let mut s = *b"   \0";
        unsafe { rs_del_trailing_spaces(s.as_mut_ptr().cast()) };
        // C logic: loops while q > ptr, so ptr[0] never gets deleted
        assert_eq!(s[0], b' ');
        assert_eq!(s[1], 0);

        // NULL pointer - should not crash
        unsafe {
            rs_del_trailing_spaces(std::ptr::null_mut());
        };
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
            assert!(rs_striequal(hello1.as_ptr(), hello2.as_ptr()));
            assert!(!rs_striequal(hello1.as_ptr(), world.as_ptr()));
            assert!(rs_striequal(std::ptr::null(), std::ptr::null()));
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
        }
    }

    #[test]
    fn test_vim_strchr_multibyte() {
        // String with UTF-8 characters: "Hello世界emoji😀end"
        let utf8_str = CString::new("Hello世界emoji😀end").unwrap();

        unsafe {
            // Find ASCII character in mixed string
            let result = rs_vim_strchr(utf8_str.as_ptr(), b'H' as c_int);
            assert!(!result.is_null());
            assert_eq!(*result as u8, b'H');

            // Find CJK character 世 (U+4E16)
            let result = rs_vim_strchr(utf8_str.as_ptr(), 0x4E16);
            assert!(!result.is_null());
            // First byte of 世 in UTF-8 is 0xE4
            assert_eq!(*result as u8, 0xE4);

            // Find CJK character 界 (U+754C)
            let result = rs_vim_strchr(utf8_str.as_ptr(), 0x754C);
            assert!(!result.is_null());
            // First byte of 界 in UTF-8 is 0xE7
            assert_eq!(*result as u8, 0xE7);

            // Find emoji 😀 (U+1F600)
            let result = rs_vim_strchr(utf8_str.as_ptr(), 0x1F600);
            assert!(!result.is_null());
            // First byte of 😀 in UTF-8 is 0xF0
            assert_eq!(*result as u8, 0xF0);

            // Character not in string - CJK 日 (U+65E5)
            let result = rs_vim_strchr(utf8_str.as_ptr(), 0x65E5);
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_has_non_ascii() {
        let ascii = CString::new("Hello, World!").unwrap();
        let non_ascii = CString::new("Hello, 世界!").unwrap();
        let empty = CString::new("").unwrap();

        unsafe {
            assert!(!rs_has_non_ascii(ascii.as_ptr()));
            assert!(rs_has_non_ascii(non_ascii.as_ptr()));
            assert!(!rs_has_non_ascii(empty.as_ptr()));
            assert!(!rs_has_non_ascii(std::ptr::null()));
        }
    }

    #[test]
    fn test_has_non_ascii_len() {
        // "Hello" + UTF-8 for 世界 + "!"
        let data: &[u8] = b"Hello\xe4\xb8\x96\xe7\x95\x8c!";

        unsafe {
            // First 5 bytes are ASCII only
            assert!(!rs_has_non_ascii_len(data.as_ptr().cast(), 5));

            // First 6 bytes include non-ASCII
            assert!(rs_has_non_ascii_len(data.as_ptr().cast(), 6));

            // Full string has non-ASCII
            assert!(rs_has_non_ascii_len(data.as_ptr().cast(), data.len()));

            // Zero length returns false
            assert!(!rs_has_non_ascii_len(data.as_ptr().cast(), 0));

            // NULL returns false
            assert!(!rs_has_non_ascii_len(std::ptr::null(), 10));
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

    #[test]
    fn test_skip_to_option_part() {
        let comma_space = CString::new(", next").unwrap();
        let just_comma = CString::new(",next").unwrap();
        let just_spaces = CString::new("   next").unwrap();
        let comma_multi_space = CString::new(",   next").unwrap();
        let no_skip = CString::new("next").unwrap();
        let empty = CString::new("").unwrap();
        let only_comma = CString::new(",").unwrap();
        let only_spaces = CString::new("   ").unwrap();

        unsafe {
            // ", next" -> skip comma and space
            let result = rs_skip_to_option_part(comma_space.as_ptr());
            assert_eq!(*result as u8, b'n');

            // ",next" -> skip comma only
            let result = rs_skip_to_option_part(just_comma.as_ptr());
            assert_eq!(*result as u8, b'n');

            // "   next" -> skip spaces only
            let result = rs_skip_to_option_part(just_spaces.as_ptr());
            assert_eq!(*result as u8, b'n');

            // ",   next" -> skip comma and multiple spaces
            let result = rs_skip_to_option_part(comma_multi_space.as_ptr());
            assert_eq!(*result as u8, b'n');

            // "next" -> no skipping needed
            let result = rs_skip_to_option_part(no_skip.as_ptr());
            assert_eq!(*result as u8, b'n');

            // "" -> returns pointer to empty
            let result = rs_skip_to_option_part(empty.as_ptr());
            assert_eq!(*result as u8, 0);

            // "," -> returns pointer to NUL after comma
            let result = rs_skip_to_option_part(only_comma.as_ptr());
            assert_eq!(*result as u8, 0);

            // "   " -> returns pointer to NUL after spaces
            let result = rs_skip_to_option_part(only_spaces.as_ptr());
            assert_eq!(*result as u8, 0);
        }
    }

    #[test]
    fn test_vim_strsave_up() {
        unsafe {
            // Basic lowercase to uppercase
            let src = CString::new("hello").unwrap();
            let result = rs_vim_strsave_up(src.as_ptr());
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "HELLO");
            libc::free(result as *mut libc::c_void);

            // Mixed case
            let src = CString::new("HeLLo WoRLD").unwrap();
            let result = rs_vim_strsave_up(src.as_ptr());
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "HELLO WORLD");
            libc::free(result as *mut libc::c_void);

            // Empty string
            let src = CString::new("").unwrap();
            let result = rs_vim_strsave_up(src.as_ptr());
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "");
            libc::free(result as *mut libc::c_void);

            // NULL handling
            let result = rs_vim_strsave_up(std::ptr::null());
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_vim_strnsave_up() {
        unsafe {
            // Copy with limit
            let src = CString::new("hello world").unwrap();
            let result = rs_vim_strnsave_up(src.as_ptr(), 5);
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "HELLO");
            libc::free(result as *mut libc::c_void);

            // Copy with limit larger than string
            let src = CString::new("hi").unwrap();
            let result = rs_vim_strnsave_up(src.as_ptr(), 100);
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "HI");
            libc::free(result as *mut libc::c_void);

            // Zero length
            let src = CString::new("hello").unwrap();
            let result = rs_vim_strnsave_up(src.as_ptr(), 0);
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "");
            libc::free(result as *mut libc::c_void);

            // NULL handling
            let result = rs_vim_strnsave_up(std::ptr::null(), 5);
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_xstrnsave() {
        unsafe {
            // Basic copy with length limit
            let src = CString::new("hello world").unwrap();
            let result = rs_xstrnsave(src.as_ptr(), 5);
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "hello");
            libc::free(result as *mut libc::c_void);

            // Copy entire string
            let src = CString::new("test").unwrap();
            let result = rs_xstrnsave(src.as_ptr(), 4);
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "test");
            libc::free(result as *mut libc::c_void);

            // Zero length
            let src = CString::new("hello").unwrap();
            let result = rs_xstrnsave(src.as_ptr(), 0);
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "");
            libc::free(result as *mut libc::c_void);

            // NULL handling
            let result = rs_xstrnsave(std::ptr::null(), 5);
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_strrep() {
        unsafe {
            // Basic replacement
            let src = CString::new("hello world").unwrap();
            let what = CString::new("world").unwrap();
            let rep = CString::new("rust").unwrap();
            let result = rs_strrep(src.as_ptr(), what.as_ptr(), rep.as_ptr());
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "hello rust");
            libc::free(result as *mut libc::c_void);

            // Multiple replacements
            let src = CString::new("aaa bbb aaa").unwrap();
            let what = CString::new("aaa").unwrap();
            let rep = CString::new("X").unwrap();
            let result = rs_strrep(src.as_ptr(), what.as_ptr(), rep.as_ptr());
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "X bbb X");
            libc::free(result as *mut libc::c_void);

            // Replace with longer string
            let src = CString::new("ab ab").unwrap();
            let what = CString::new("ab").unwrap();
            let rep = CString::new("XYZ").unwrap();
            let result = rs_strrep(src.as_ptr(), what.as_ptr(), rep.as_ptr());
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "XYZ XYZ");
            libc::free(result as *mut libc::c_void);

            // No match - returns NULL
            let src = CString::new("hello").unwrap();
            let what = CString::new("world").unwrap();
            let rep = CString::new("X").unwrap();
            let result = rs_strrep(src.as_ptr(), what.as_ptr(), rep.as_ptr());
            assert!(result.is_null());

            // Empty what - returns NULL
            let src = CString::new("hello").unwrap();
            let what = CString::new("").unwrap();
            let rep = CString::new("X").unwrap();
            let result = rs_strrep(src.as_ptr(), what.as_ptr(), rep.as_ptr());
            assert!(result.is_null());

            // NULL handling
            let src = CString::new("hello").unwrap();
            let what = CString::new("h").unwrap();
            let rep = CString::new("X").unwrap();
            assert!(rs_strrep(std::ptr::null(), what.as_ptr(), rep.as_ptr()).is_null());
            assert!(rs_strrep(src.as_ptr(), std::ptr::null(), rep.as_ptr()).is_null());
            assert!(rs_strrep(src.as_ptr(), what.as_ptr(), std::ptr::null()).is_null());
        }
    }

    #[test]
    fn test_vim_strnsave_unquoted() {
        unsafe {
            // Simple unquoted string (no quotes)
            let src = CString::new("hello world").unwrap();
            let result = rs_vim_strnsave_unquoted(src.as_ptr(), 11);
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "hello world");
            libc::free(result as *mut libc::c_void);

            // String with quotes - quotes removed
            let src = b"hello\"world\"end\0";
            let result = rs_vim_strnsave_unquoted(src.as_ptr() as *const c_char, 15);
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            // Quotes are stripped, content between quotes preserved
            assert_eq!(result_str.to_str().unwrap(), "helloworldend");
            libc::free(result as *mut libc::c_void);

            // Escaped backslash inside quotes
            let src = b"a\"b\\\\c\"d\0";
            let result = rs_vim_strnsave_unquoted(src.as_ptr() as *const c_char, 9);
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            // Inside quotes, \\ becomes \
            assert_eq!(result_str.to_str().unwrap(), "ab\\cd");
            libc::free(result as *mut libc::c_void);

            // Escaped quote inside quotes
            let src = b"a\"b\\\"c\"d\0";
            let result = rs_vim_strnsave_unquoted(src.as_ptr() as *const c_char, 9);
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            // Inside quotes, \" becomes "
            assert_eq!(result_str.to_str().unwrap(), "ab\"cd");
            libc::free(result as *mut libc::c_void);

            // Empty string
            let src = CString::new("").unwrap();
            let result = rs_vim_strnsave_unquoted(src.as_ptr(), 0);
            assert!(!result.is_null());
            let result_str = std::ffi::CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "");
            libc::free(result as *mut libc::c_void);

            // NULL handling
            let result = rs_vim_strnsave_unquoted(std::ptr::null(), 5);
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_strchrsub() {
        // Basic character replacement
        let mut s = *b"hello\0";
        unsafe { rs_strchrsub(s.as_mut_ptr().cast(), b'l' as c_char, b'x' as c_char) };
        assert_eq!(&s[..6], b"hexxo\0");

        // No match - string unchanged
        let mut s = *b"hello\0";
        unsafe { rs_strchrsub(s.as_mut_ptr().cast(), b'z' as c_char, b'x' as c_char) };
        assert_eq!(&s[..6], b"hello\0");

        // Replace all same chars
        let mut s = *b"aaa\0";
        unsafe { rs_strchrsub(s.as_mut_ptr().cast(), b'a' as c_char, b'b' as c_char) };
        assert_eq!(&s[..4], b"bbb\0");

        // Empty string - should not crash
        let mut s = *b"\0";
        unsafe { rs_strchrsub(s.as_mut_ptr().cast(), b'a' as c_char, b'b' as c_char) };
        assert_eq!(s[0], 0);

        // NULL pointer - should not crash
        unsafe { rs_strchrsub(std::ptr::null_mut(), b'a' as c_char, b'b' as c_char) };

        // c == NUL - should not modify (safety check)
        let mut s = *b"hello\0";
        unsafe { rs_strchrsub(s.as_mut_ptr().cast(), 0, b'x' as c_char) };
        assert_eq!(&s[..6], b"hello\0");
    }

    #[test]
    fn test_memchrsub() {
        // Basic byte replacement
        let mut data = *b"hello";
        unsafe { rs_memchrsub(data.as_mut_ptr().cast(), b'l' as c_char, b'x' as c_char, 5) };
        assert_eq!(&data, b"hexxo");

        // No match - data unchanged
        let mut data = *b"hello";
        unsafe { rs_memchrsub(data.as_mut_ptr().cast(), b'z' as c_char, b'x' as c_char, 5) };
        assert_eq!(&data, b"hello");

        // Partial replacement (only first 3 bytes)
        let mut data = *b"lllll";
        unsafe { rs_memchrsub(data.as_mut_ptr().cast(), b'l' as c_char, b'x' as c_char, 3) };
        assert_eq!(&data, b"xxxll");

        // Binary data with NUL bytes
        let mut data = [b'a', 0, b'a', 0, b'a'];
        unsafe { rs_memchrsub(data.as_mut_ptr().cast(), b'a' as c_char, b'b' as c_char, 5) };
        assert_eq!(&data, &[b'b', 0, b'b', 0, b'b']);

        // Replace NUL bytes
        let mut data = [0u8, 1, 0, 1, 0];
        unsafe { rs_memchrsub(data.as_mut_ptr().cast(), 0, b'x' as c_char, 5) };
        assert_eq!(&data, &[b'x', 1, b'x', 1, b'x']);

        // Zero length - should not crash
        let mut data = *b"hello";
        unsafe { rs_memchrsub(data.as_mut_ptr().cast(), b'l' as c_char, b'x' as c_char, 0) };
        assert_eq!(&data, b"hello"); // Unchanged

        // NULL pointer - should not crash
        unsafe { rs_memchrsub(std::ptr::null_mut(), b'a' as c_char, b'b' as c_char, 5) };
    }

    #[test]
    fn test_xstpcpy() {
        // Basic copy
        let src = CString::new("hello").unwrap();
        let mut dst = [0i8; 10];
        unsafe {
            let result = rs_xstpcpy(dst.as_mut_ptr(), src.as_ptr());
            // Result should point to the NUL terminator
            assert_eq!(result, dst.as_mut_ptr().add(5));
            assert_eq!(*result, 0);
            // Verify the copy
            let copied = std::ffi::CStr::from_ptr(dst.as_ptr());
            assert_eq!(copied.to_str().unwrap(), "hello");
        }

        // Empty string
        let src = CString::new("").unwrap();
        let mut dst = [0i8; 10];
        unsafe {
            let result = rs_xstpcpy(dst.as_mut_ptr(), src.as_ptr());
            assert_eq!(result, dst.as_mut_ptr());
            assert_eq!(*result, 0);
        }
    }

    #[test]
    fn test_xstpncpy() {
        // Copy with padding
        let src = CString::new("hi").unwrap();
        let mut dst = [b'X' as i8; 10];
        unsafe {
            let result = rs_xstpncpy(dst.as_mut_ptr(), src.as_ptr(), 5);
            // Result should point to the NUL position
            assert_eq!(result, dst.as_mut_ptr().add(2));
            // Check first 5 bytes: "hi\0\0\0"
            assert_eq!(dst[0], b'h' as i8);
            assert_eq!(dst[1], b'i' as i8);
            assert_eq!(dst[2], 0);
            assert_eq!(dst[3], 0);
            assert_eq!(dst[4], 0);
            // Rest should be unchanged
            assert_eq!(dst[5], b'X' as i8);
        }

        // Copy without NUL in maxlen
        let src = CString::new("hello world").unwrap();
        let mut dst = [0i8; 10];
        unsafe {
            let result = rs_xstpncpy(dst.as_mut_ptr(), src.as_ptr(), 5);
            // Result should point to &dst[5] (no NUL written)
            assert_eq!(result, dst.as_mut_ptr().add(5));
            // First 5 bytes are "hello" (no NUL)
            assert_eq!(dst[0], b'h' as i8);
            assert_eq!(dst[4], b'o' as i8);
        }
    }

    #[test]
    fn test_xstrlcpy() {
        // Basic copy that fits
        let src = CString::new("hello").unwrap();
        let mut dst = [0i8; 10];
        unsafe {
            let result = rs_xstrlcpy(dst.as_mut_ptr(), src.as_ptr(), 10);
            assert_eq!(result, 5); // Length of "hello"
            let copied = std::ffi::CStr::from_ptr(dst.as_ptr());
            assert_eq!(copied.to_str().unwrap(), "hello");
        }

        // Truncation
        let src = CString::new("hello world").unwrap();
        let mut dst = [0i8; 6];
        unsafe {
            let result = rs_xstrlcpy(dst.as_mut_ptr(), src.as_ptr(), 6);
            assert_eq!(result, 11); // Full length of "hello world"
            let copied = std::ffi::CStr::from_ptr(dst.as_ptr());
            assert_eq!(copied.to_str().unwrap(), "hello"); // Truncated
        }

        // Zero dsize - no write
        let src = CString::new("hello").unwrap();
        let mut dst = [b'X' as i8; 5];
        unsafe {
            let result = rs_xstrlcpy(dst.as_mut_ptr(), src.as_ptr(), 0);
            assert_eq!(result, 5);
            assert_eq!(dst[0], b'X' as i8); // Unchanged
        }

        // dsize = 1 - only NUL
        let src = CString::new("hello").unwrap();
        let mut dst = [b'X' as i8; 5];
        unsafe {
            let result = rs_xstrlcpy(dst.as_mut_ptr(), src.as_ptr(), 1);
            assert_eq!(result, 5);
            assert_eq!(dst[0], 0); // Just NUL
        }
    }

    #[test]
    fn test_xstrlcat() {
        // Basic append
        let mut dst = [0i8; 20];
        dst[0] = b'h' as i8;
        dst[1] = b'i' as i8;
        dst[2] = 0;
        let src = CString::new(" there").unwrap();
        unsafe {
            let result = rs_xstrlcat(dst.as_mut_ptr(), src.as_ptr(), 20);
            assert_eq!(result, 8); // "hi" (2) + " there" (6)
            let combined = std::ffi::CStr::from_ptr(dst.as_ptr());
            assert_eq!(combined.to_str().unwrap(), "hi there");
        }

        // Truncation
        let mut dst = [0i8; 10];
        dst[0] = b'h' as i8;
        dst[1] = b'e' as i8;
        dst[2] = b'l' as i8;
        dst[3] = b'l' as i8;
        dst[4] = b'o' as i8;
        dst[5] = 0;
        let src = CString::new(" world").unwrap();
        unsafe {
            let result = rs_xstrlcat(dst.as_mut_ptr(), src.as_ptr(), 10);
            assert_eq!(result, 11); // "hello" (5) + " world" (6)
            let combined = std::ffi::CStr::from_ptr(dst.as_ptr());
            assert_eq!(combined.to_str().unwrap(), "hello wor"); // Truncated at 9 chars + NUL
        }

        // Append to empty
        let mut dst = [0i8; 10];
        let src = CString::new("hello").unwrap();
        unsafe {
            let result = rs_xstrlcat(dst.as_mut_ptr(), src.as_ptr(), 10);
            assert_eq!(result, 5);
            let combined = std::ffi::CStr::from_ptr(dst.as_ptr());
            assert_eq!(combined.to_str().unwrap(), "hello");
        }
    }

    #[test]
    fn test_mb_maxbytes_constant() {
        // Verify MB_MAXBYTES matches C definition (max bytes for a UTF-8 character)
        // UTF-8 can encode up to 6 bytes per character in the original spec
        assert_eq!(MB_MAXBYTES, 6);
    }
}
