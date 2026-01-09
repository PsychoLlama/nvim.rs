//! Tag line parsing functions for Neovim C-to-Rust migration
//!
//! This module provides Rust implementations for parsing tag file lines and
//! extracting tag components (name, filename, command, kind, `user_data`, etc.).

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
#![allow(clippy::missing_safety_doc)] // Safety docs handled at extern "C" boundary
#![allow(clippy::doc_markdown)] // Don't require backticks for TagPtrs etc

use std::ffi::{c_char, c_int, CStr};
use std::ptr;

/// Tab character constant
const TAB: u8 = b'\t';

/// Line number type (matches `linenr_T` in Neovim)
type LinenrT = i32;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    /// Find a character in a string (like strchr)
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;

    /// Skip over a regexp pattern
    fn skip_regexp(startp: *mut c_char, delim: c_int, magic: bool) -> *mut c_char;

    /// Skip over decimal digits
    fn skipdigits(p: *const c_char) -> *mut c_char;

    /// Get length of UTF-8 character
    fn utfc_ptr2len(p: *const c_char) -> c_int;
}

// =============================================================================
// Inline ASCII character classification (avoids linking to C macros)
// =============================================================================

/// Check if character is ASCII alpha
#[inline]
const fn ascii_isalpha(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

/// Check if character is ASCII digit
#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c.is_ascii_digit()
}

// =============================================================================
// TagPtrs structure - mirrors tagptrs_T from tag.c
// =============================================================================

/// Structure to hold pointers to various items in a tag line.
///
/// This mirrors the C `tagptrs_T` structure and holds pointers into
/// a tag line buffer without taking ownership of the memory.
#[repr(C)]
pub struct TagPtrs {
    // filled in by parse_tag_line():
    /// Start of tag name (skip "file:")
    pub tagname: *mut c_char,
    /// Char after tag name
    pub tagname_end: *mut c_char,
    /// First char of file name
    pub fname: *mut c_char,
    /// Char after file name
    pub fname_end: *mut c_char,
    /// First char of command
    pub command: *mut c_char,
    // filled in by parse_match():
    /// First char after command
    pub command_end: *mut c_char,
    /// File name of the tags file (used when 'tr' is set)
    pub tag_fname: *mut c_char,
    /// "kind:" value
    pub tagkind: *mut c_char,
    /// End of tagkind
    pub tagkind_end: *mut c_char,
    /// `user_data` string
    pub user_data: *mut c_char,
    /// End of `user_data`
    pub user_data_end: *mut c_char,
    /// "line:" value
    pub tagline: LinenrT,
}

impl Default for TagPtrs {
    fn default() -> Self {
        Self {
            tagname: ptr::null_mut(),
            tagname_end: ptr::null_mut(),
            fname: ptr::null_mut(),
            fname_end: ptr::null_mut(),
            command: ptr::null_mut(),
            command_end: ptr::null_mut(),
            tag_fname: ptr::null_mut(),
            tagkind: ptr::null_mut(),
            tagkind_end: ptr::null_mut(),
            user_data: ptr::null_mut(),
            user_data_end: ptr::null_mut(),
            tagline: 0,
        }
    }
}

// =============================================================================
// Core parsing functions
// =============================================================================

/// Parse a tag line from a tags file.
///
/// Extracts the tag name, file name, and command from a standard tags file line.
/// The expected format is: `tagname<TAB>filename<TAB>command`
///
/// # Safety
///
/// - `lbuf` must be a valid pointer to a null-terminated C string
/// - `tagp` must be a valid pointer to a `TagPtrs` struct
/// - The `TagPtrs` struct will contain pointers into the `lbuf` buffer
///
/// # Returns
///
/// - `OK` (1) on success
/// - `FAIL` (0) if there is a format error in the line
#[no_mangle]
pub unsafe extern "C" fn rs_parse_tag_line(lbuf: *mut c_char, tagp: *mut TagPtrs) -> c_int {
    const OK: c_int = 1;
    const FAIL: c_int = 0;

    if lbuf.is_null() || tagp.is_null() {
        return FAIL;
    }

    let tagp = &mut *tagp;

    // Isolate the tagname, from lbuf up to the first tab
    tagp.tagname = lbuf;
    let p = vim_strchr(lbuf, c_int::from(TAB));
    if p.is_null() {
        return FAIL;
    }
    tagp.tagname_end = p;

    // Isolate file name, from first to second tab
    let mut p = p;
    if *p != 0 {
        p = p.add(1);
    }
    tagp.fname = p;
    let p = vim_strchr(p, c_int::from(TAB));
    if p.is_null() {
        return FAIL;
    }
    tagp.fname_end = p;

    // Find start of search command, after second tab
    let mut p = p;
    if *p != 0 {
        p = p.add(1);
    }
    if *p == 0 {
        return FAIL;
    }
    tagp.command = p;

    OK
}

/// Find the end of the tag address.
///
/// This function looks for the end of a tag's search pattern/line number
/// and checks if it's followed by `;"` which marks the start of extended fields.
///
/// # Safety
///
/// - `pp` must be a valid pointer to a pointer to a null-terminated C string
///
/// # Returns
///
/// - `OK` (1) if `;"` is following
/// - `FAIL` (0) otherwise
#[no_mangle]
pub unsafe extern "C" fn rs_find_extra(pp: *mut *mut c_char) -> c_int {
    const OK: c_int = 1;
    const FAIL: c_int = 0;

    if pp.is_null() || (*pp).is_null() {
        return FAIL;
    }

    let mut str = *pp;
    let first_char = *str;

    // Repeat for addresses separated with ';'
    loop {
        if ascii_isdigit(*str as u8) {
            str = skipdigits(str.add(1));
        } else if *str == b'/' as c_char || *str == b'?' as c_char {
            str = skip_regexp(str.add(1), *str as c_int, false);
            if *str == first_char {
                str = str.add(1);
            } else {
                str = ptr::null_mut();
            }
        } else {
            // Not a line number or search string, look for terminator.
            str = find_str(str, c"|;\"".as_ptr().cast());
            if !str.is_null() {
                str = str.add(1);
                break;
            }
        }

        if str.is_null()
            || *str != b';' as c_char
            || !(ascii_isdigit(*str.add(1) as u8)
                || *str.add(1) == b'/' as c_char
                || *str.add(1) == b'?' as c_char)
        {
            break;
        }
        str = str.add(1); // skip ';'
    }

    if !str.is_null() && *str == b';' as c_char && *str.add(1) == b'"' as c_char {
        *pp = str;
        return OK;
    }
    FAIL
}

/// Helper function to find a substring in a string (like strstr)
unsafe fn find_str(haystack: *const c_char, needle: *const c_char) -> *mut c_char {
    if haystack.is_null() || needle.is_null() {
        return ptr::null_mut();
    }

    let needle_cstr = CStr::from_ptr(needle);
    let needle_bytes = needle_cstr.to_bytes();
    if needle_bytes.is_empty() {
        return haystack.cast_mut();
    }

    let mut p = haystack;
    while *p != 0 {
        let mut match_found = true;
        for (i, &needle_byte) in needle_bytes.iter().enumerate() {
            if *p.add(i) as u8 != needle_byte {
                match_found = false;
                break;
            }
        }
        if match_found {
            return p.cast_mut();
        }
        p = p.add(1);
    }

    ptr::null_mut()
}

/// Parse a line from a matching tag.
///
/// The line format for stored matches is:
/// `<mtt><tag_fname><NUL><NUL><lbuf>`
///
/// Where:
/// - `mtt` is the match type (1 byte)
/// - `tag_fname` is the tags file name
/// - `lbuf` is the original tag line
///
/// # Safety
///
/// - `lbuf` must be a valid pointer to a formatted match line
/// - `tagp` must be a valid pointer to a `TagPtrs` struct
///
/// # Returns
///
/// - `OK` (1) on success
/// - `FAIL` (0) on failure
#[no_mangle]
pub unsafe extern "C" fn rs_parse_match(lbuf: *mut c_char, tagp: *mut TagPtrs) -> c_int {
    const OK: c_int = 1;
    const FAIL: c_int = 0;

    if lbuf.is_null() || tagp.is_null() {
        return FAIL;
    }

    let tagp = &mut *tagp;

    // Extract tag_fname (starts after the mtt byte)
    tagp.tag_fname = lbuf.add(1);

    // Skip past tag_fname and two NUL bytes to get to lbuf
    let fname_len = strlen_safe(tagp.tag_fname);
    let tag_line = lbuf.add(1).add(fname_len).add(2);

    // Parse the actual tag line
    let retval = rs_parse_tag_line(tag_line, tagp);

    // Initialize extended field pointers
    tagp.tagkind = ptr::null_mut();
    tagp.user_data = ptr::null_mut();
    tagp.tagline = 0;
    tagp.command_end = ptr::null_mut();

    if retval != OK {
        return retval;
    }

    // Try to find extended fields after the command
    let mut p = tagp.command;
    if rs_find_extra(std::ptr::addr_of_mut!(p)) == OK {
        tagp.command_end = p;
        // Check for trailing bar and adjust
        if p > tagp.command && *p.sub(1) == b'|' as c_char {
            tagp.command_end = p.sub(1);
        }
        p = p.add(2); // skip ";\"
        if *p == TAB as c_char {
            p = p.add(1);
            // Parse extended fields
            while ascii_isalpha(*p as u8) || utfc_ptr2len(p) > 1 {
                if starts_with(p, c"kind:".as_ptr().cast()) {
                    tagp.tagkind = p.add(5);
                } else if starts_with(p, c"user_data:".as_ptr().cast()) {
                    tagp.user_data = p.add(10);
                } else if starts_with(p, c"line:".as_ptr().cast()) {
                    tagp.tagline = parse_number(p.add(5));
                }

                if !tagp.tagkind.is_null() && !tagp.user_data.is_null() {
                    break;
                }

                // Check for standalone kind (no "kind:" prefix)
                let pc = vim_strchr(p, b':' as c_int);
                let pt = vim_strchr(p, c_int::from(TAB));
                if (pc.is_null() || (!pt.is_null() && pc > pt)) && tagp.tagkind.is_null() {
                    tagp.tagkind = p;
                }
                if pt.is_null() {
                    break;
                }
                p = pt;
                // Skip one character (MB_PTR_ADV)
                let char_len = utfc_ptr2len(p);
                p = p.add(char_len.max(1) as usize);
            }
        }
    }

    // Find end of tagkind
    if !tagp.tagkind.is_null() {
        p = tagp.tagkind;
        while *p != 0 && *p != TAB as c_char && *p != b'\r' as c_char && *p != b'\n' as c_char {
            let char_len = utfc_ptr2len(p);
            p = p.add(char_len.max(1) as usize);
        }
        tagp.tagkind_end = p;
    }

    // Find end of user_data
    if !tagp.user_data.is_null() {
        p = tagp.user_data;
        while *p != 0 && *p != TAB as c_char && *p != b'\r' as c_char && *p != b'\n' as c_char {
            let char_len = utfc_ptr2len(p);
            p = p.add(char_len.max(1) as usize);
        }
        tagp.user_data_end = p;
    }

    retval
}

/// Check if string `s` starts with prefix `prefix`
unsafe fn starts_with(s: *const c_char, prefix: *const c_char) -> bool {
    if s.is_null() || prefix.is_null() {
        return false;
    }
    let prefix_cstr = CStr::from_ptr(prefix);
    let prefix_bytes = prefix_cstr.to_bytes();

    for (i, &byte) in prefix_bytes.iter().enumerate() {
        if *s.add(i) as u8 != byte {
            return false;
        }
    }
    true
}

/// Parse a number from a string (pure Rust implementation)
unsafe fn parse_number(s: *const c_char) -> LinenrT {
    if s.is_null() {
        return 0;
    }
    let mut result: LinenrT = 0;
    let mut p = s;
    while (*p as u8).is_ascii_digit() {
        result = result * 10 + ((*p as u8 - b'0') as LinenrT);
        p = p.add(1);
    }
    result
}

/// Safe strlen that handles null pointers
unsafe fn strlen_safe(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut len = 0;
    let mut p = s;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }
    len
}

/// Check if tagname indicates a static tag.
///
/// Static tags produced by the older ctags have the format:
/// `'file:tag  file  /pattern'`
///
/// Static tags produced by the new ctags have the format:
/// `'tag  file  /pattern/;"<Tab>file:'`
///
/// # Safety
///
/// - `tagp` must be a valid pointer to a `TagPtrs` struct
///
/// # Returns
///
/// - `true` if it is a static tag
/// - `false` otherwise
#[no_mangle]
pub unsafe extern "C" fn rs_test_for_static(tagp: *const TagPtrs) -> bool {
    if tagp.is_null() {
        return false;
    }

    let tagp = &*tagp;
    if tagp.command.is_null() {
        return false;
    }

    // Check for new style static tag ":...<Tab>file:[<Tab>...]"
    let mut p = tagp.command;
    loop {
        p = vim_strchr(p, c_int::from(TAB));
        if p.is_null() {
            break;
        }
        p = p.add(1);
        if starts_with(p, c"file:".as_ptr().cast()) {
            return true;
        }
    }

    false
}

/// Return the length of a matching tag line.
///
/// # Safety
///
/// - `lbuf` must be a valid pointer to a formatted match line
#[no_mangle]
pub unsafe extern "C" fn rs_matching_line_len(lbuf: *const c_char) -> usize {
    if lbuf.is_null() {
        return 0;
    }

    // Skip the mtt byte
    let p = lbuf.add(1);
    // Skip past tag_fname to NUL
    let fname_len = strlen_safe(p);
    // Skip past the second NUL and get to lbuf content
    let lbuf_start = p.add(fname_len).add(1);
    let lbuf_len = strlen_safe(lbuf_start);

    // Total length: 1 (mtt) + fname_len + 1 (NUL) + lbuf_len
    1 + fname_len + 1 + lbuf_len
}

// =============================================================================
// Tag pointer accessor functions (for C interop)
// =============================================================================

/// Get the tag name from TagPtrs.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_get_tagname(tagp: *const TagPtrs) -> *const c_char {
    if tagp.is_null() {
        return ptr::null();
    }
    (*tagp).tagname
}

/// Get the tag name length from TagPtrs.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_tagname_len(tagp: *const TagPtrs) -> usize {
    if tagp.is_null() {
        return 0;
    }
    let tagp = &*tagp;
    if tagp.tagname.is_null() || tagp.tagname_end.is_null() {
        return 0;
    }
    tagp.tagname_end.offset_from(tagp.tagname) as usize
}

/// Get the file name from TagPtrs.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_get_fname(tagp: *const TagPtrs) -> *const c_char {
    if tagp.is_null() {
        return ptr::null();
    }
    (*tagp).fname
}

/// Get the file name length from TagPtrs.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_fname_len(tagp: *const TagPtrs) -> usize {
    if tagp.is_null() {
        return 0;
    }
    let tagp = &*tagp;
    if tagp.fname.is_null() || tagp.fname_end.is_null() {
        return 0;
    }
    tagp.fname_end.offset_from(tagp.fname) as usize
}

/// Get the command from TagPtrs.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_get_command(tagp: *const TagPtrs) -> *const c_char {
    if tagp.is_null() {
        return ptr::null();
    }
    (*tagp).command
}

/// Get the kind from TagPtrs.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_get_tagkind(tagp: *const TagPtrs) -> *const c_char {
    if tagp.is_null() {
        return ptr::null();
    }
    (*tagp).tagkind
}

/// Get the kind length from TagPtrs.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_tagkind_len(tagp: *const TagPtrs) -> usize {
    if tagp.is_null() {
        return 0;
    }
    let tagp = &*tagp;
    if tagp.tagkind.is_null() || tagp.tagkind_end.is_null() {
        return 0;
    }
    tagp.tagkind_end.offset_from(tagp.tagkind) as usize
}

/// Get the user_data from TagPtrs.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_get_user_data(tagp: *const TagPtrs) -> *const c_char {
    if tagp.is_null() {
        return ptr::null();
    }
    (*tagp).user_data
}

/// Get the user_data length from TagPtrs.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_user_data_len(tagp: *const TagPtrs) -> usize {
    if tagp.is_null() {
        return 0;
    }
    let tagp = &*tagp;
    if tagp.user_data.is_null() || tagp.user_data_end.is_null() {
        return 0;
    }
    tagp.user_data_end.offset_from(tagp.user_data) as usize
}

/// Get the line number from TagPtrs.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_get_tagline(tagp: *const TagPtrs) -> LinenrT {
    if tagp.is_null() {
        return 0;
    }
    (*tagp).tagline
}

/// Get the tag file name from TagPtrs.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_get_tag_fname(tagp: *const TagPtrs) -> *const c_char {
    if tagp.is_null() {
        return ptr::null();
    }
    (*tagp).tag_fname
}

/// Check if TagPtrs has a valid tagkind.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_has_kind(tagp: *const TagPtrs) -> bool {
    if tagp.is_null() {
        return false;
    }
    !(*tagp).tagkind.is_null()
}

/// Check if TagPtrs has valid user_data.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_has_user_data(tagp: *const TagPtrs) -> bool {
    if tagp.is_null() {
        return false;
    }
    !(*tagp).user_data.is_null()
}

/// Initialize a TagPtrs structure to all nulls/zeros.
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_init(tagp: *mut TagPtrs) {
    if tagp.is_null() {
        return;
    }
    *tagp = TagPtrs::default();
}

/// Create a new TagPtrs structure.
///
/// Returns an opaque handle that must be freed with `rs_tagptrs_free`.
#[no_mangle]
pub extern "C" fn rs_tagptrs_new() -> *mut TagPtrs {
    Box::into_raw(Box::new(TagPtrs::default()))
}

/// Free a TagPtrs structure created by `rs_tagptrs_new`.
///
/// # Safety
///
/// - `tagp` must have been created by `rs_tagptrs_new`
/// - `tagp` must not be used after this call
#[no_mangle]
pub unsafe extern "C" fn rs_tagptrs_free(tagp: *mut TagPtrs) {
    if !tagp.is_null() {
        drop(Box::from_raw(tagp));
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tagptrs_default() {
        let tagp = TagPtrs::default();
        assert!(tagp.tagname.is_null());
        assert!(tagp.fname.is_null());
        assert!(tagp.command.is_null());
        assert_eq!(tagp.tagline, 0);
    }

    #[test]
    fn test_tagptrs_new_free() {
        let tagp = rs_tagptrs_new();
        assert!(!tagp.is_null());
        unsafe {
            assert!((*tagp).tagname.is_null());
            rs_tagptrs_free(tagp);
        }
    }

    #[test]
    fn test_null_matching_line_len() {
        unsafe {
            assert_eq!(rs_matching_line_len(ptr::null()), 0);
        }
    }

    #[test]
    fn test_strlen_safe() {
        unsafe {
            assert_eq!(strlen_safe(ptr::null()), 0);
            assert_eq!(strlen_safe(c"hello".as_ptr()), 5);
            assert_eq!(strlen_safe(c"".as_ptr()), 0);
        }
    }

    #[test]
    fn test_starts_with() {
        unsafe {
            assert!(starts_with(c"kind:f".as_ptr(), c"kind:".as_ptr()));
            assert!(!starts_with(c"user:f".as_ptr(), c"kind:".as_ptr()));
            assert!(!starts_with(ptr::null(), c"kind:".as_ptr()));
            assert!(!starts_with(c"kind:f".as_ptr(), ptr::null()));
        }
    }

    #[test]
    fn test_parse_number() {
        unsafe {
            assert_eq!(parse_number(c"123".as_ptr()), 123);
            assert_eq!(parse_number(c"0".as_ptr()), 0);
            assert_eq!(parse_number(c"42abc".as_ptr()), 42);
            assert_eq!(parse_number(ptr::null()), 0);
        }
    }

    // Accessor tests (pure Rust, no FFI dependencies)

    #[test]
    fn test_tagptrs_accessors_null() {
        unsafe {
            assert!(rs_tagptrs_get_tagname(ptr::null()).is_null());
            assert_eq!(rs_tagptrs_tagname_len(ptr::null()), 0);
            assert!(rs_tagptrs_get_fname(ptr::null()).is_null());
            assert_eq!(rs_tagptrs_fname_len(ptr::null()), 0);
            assert!(rs_tagptrs_get_command(ptr::null()).is_null());
            assert!(rs_tagptrs_get_tagkind(ptr::null()).is_null());
            assert_eq!(rs_tagptrs_tagkind_len(ptr::null()), 0);
            assert!(rs_tagptrs_get_user_data(ptr::null()).is_null());
            assert_eq!(rs_tagptrs_user_data_len(ptr::null()), 0);
            assert_eq!(rs_tagptrs_get_tagline(ptr::null()), 0);
            assert!(rs_tagptrs_get_tag_fname(ptr::null()).is_null());
            assert!(!rs_tagptrs_has_kind(ptr::null()));
            assert!(!rs_tagptrs_has_user_data(ptr::null()));
        }
    }

    #[test]
    fn test_tagptrs_init() {
        let mut tagp = TagPtrs {
            tagname: 0x1234 as *mut c_char, // non-null dummy
            tagname_end: ptr::null_mut(),
            fname: ptr::null_mut(),
            fname_end: ptr::null_mut(),
            command: ptr::null_mut(),
            command_end: ptr::null_mut(),
            tag_fname: ptr::null_mut(),
            tagkind: ptr::null_mut(),
            tagkind_end: ptr::null_mut(),
            user_data: ptr::null_mut(),
            user_data_end: ptr::null_mut(),
            tagline: 42,
        };

        unsafe {
            rs_tagptrs_init(std::ptr::addr_of_mut!(tagp));
        }

        assert!(tagp.tagname.is_null());
        assert_eq!(tagp.tagline, 0);
    }

    #[test]
    fn test_find_str() {
        unsafe {
            // Test finding a substring
            let haystack = c"hello|;\"world".as_ptr();
            let needle = c"|;\"".as_ptr();
            let result = find_str(haystack, needle);
            assert!(!result.is_null());
            assert_eq!(*result as u8, b'|');

            // Test not finding
            let result = find_str(c"hello".as_ptr(), c"xyz".as_ptr());
            assert!(result.is_null());

            // Test null handling
            assert!(find_str(ptr::null(), needle).is_null());
            assert!(find_str(haystack, ptr::null()).is_null());
        }
    }
}
