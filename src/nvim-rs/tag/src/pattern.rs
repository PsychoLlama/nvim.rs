//! Tag pattern matching infrastructure for Neovim C-to-Rust migration
//!
//! This module provides Rust implementations for tag pattern handling including
//! pattern compilation, head extraction for binary search, and matching.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

/// Maximum number of subexpressions in a regexp
const NSUBEXP: usize = 10;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    /// Check if 'magic' option is set
    fn magic_isset() -> bool;

    /// Get the value of 'taglength' option
    fn nvim_get_p_tl() -> i64;

    /// Find a character in a string
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;

    /// Compile a regular expression
    fn vim_regcomp(pat: *const c_char, flags: c_int) -> *mut c_void;

    /// Execute a regular expression match
    fn vim_regexec(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> bool;

    /// Free a compiled regexp
    fn vim_regfree(prog: *mut c_void);
}

// =============================================================================
// Types matching C structures
// =============================================================================

/// Opaque handle to regprog_T
type RegProgHandle = *mut c_void;

/// Structure matching regmatch_T for single-line matching
#[repr(C)]
pub struct RegMatch {
    /// The compiled regexp program
    pub regprog: RegProgHandle,
    /// Start positions of submatches
    pub startp: [*mut c_char; NSUBEXP],
    /// End positions of submatches
    pub endp: [*mut c_char; NSUBEXP],
    /// Match start without "\zs"
    pub rm_matchcol: c_int,
    /// Ignore case flag
    pub rm_ic: bool,
}

impl Default for RegMatch {
    fn default() -> Self {
        Self {
            regprog: ptr::null_mut(),
            startp: [ptr::null_mut(); NSUBEXP],
            endp: [ptr::null_mut(); NSUBEXP],
            rm_matchcol: 0,
            rm_ic: false,
        }
    }
}

/// Structure to hold info about a tag pattern.
///
/// Mirrors the C `pat_T` structure.
#[repr(C)]
pub struct TagPattern {
    /// The pattern string
    pub pat: *mut c_char,
    /// Length of pat[]
    pub len: c_int,
    /// Start of pattern head (for binary search)
    pub head: *mut c_char,
    /// Length of head[]
    pub headlen: c_int,
    /// Regexp program, may be NULL
    pub regmatch: RegMatch,
}

impl Default for TagPattern {
    fn default() -> Self {
        Self {
            pat: ptr::null_mut(),
            len: 0,
            head: ptr::null_mut(),
            headlen: 0,
            regmatch: RegMatch::default(),
        }
    }
}

// =============================================================================
// RE_MAGIC constant
// =============================================================================

/// Magic flag for vim_regcomp
const RE_MAGIC: c_int = 1;

// =============================================================================
// Pattern preparation and matching
// =============================================================================

/// Prepare a tag pattern for searching.
///
/// Extracts the head portion of the pattern that can be used for binary
/// searching (the fixed prefix before any regexp metacharacters).
///
/// # Safety
///
/// - `pats` must be a valid pointer to a TagPattern struct
/// - `pats.pat` must be a valid null-terminated string
#[no_mangle]
pub unsafe extern "C" fn rs_prepare_pats(pats: *mut TagPattern, has_re: bool) {
    if pats.is_null() {
        return;
    }

    let pats = &mut *pats;

    if pats.pat.is_null() {
        return;
    }

    pats.head = pats.pat;
    pats.headlen = pats.len;

    if has_re {
        // When the pattern starts with '^' or "\\<", binary searching can be
        // used (much faster).
        if *pats.pat == b'^' as c_char {
            pats.head = pats.pat.add(1);
        } else if *pats.pat == b'\\' as c_char && *pats.pat.add(1) == b'<' as c_char {
            pats.head = pats.pat.add(2);
        }

        if pats.head == pats.pat {
            pats.headlen = 0;
        } else {
            // Find length of fixed prefix before regexp metacharacters
            let meta_chars = if magic_isset() {
                c".[~*\\$".as_ptr()
            } else {
                c"\\$".as_ptr()
            };

            pats.headlen = 0;
            while *pats.head.add(pats.headlen as usize) != 0 {
                let c = *pats.head.add(pats.headlen as usize);
                if !vim_strchr(meta_chars, c as c_int).is_null() {
                    break;
                }
                pats.headlen += 1;
            }
        }

        // Adjust for 'taglength' option
        let p_tl = nvim_get_p_tl();
        if p_tl != 0 && pats.headlen > p_tl as c_int {
            pats.headlen = p_tl as c_int;
        }
    }

    // Compile the regexp if using regex
    if has_re {
        let flags = if magic_isset() { RE_MAGIC } else { 0 };
        pats.regmatch.regprog = vim_regcomp(pats.pat, flags);
    } else {
        pats.regmatch.regprog = ptr::null_mut();
    }
}

/// Free resources associated with a TagPattern.
///
/// # Safety
///
/// - `pats` must be a valid pointer to a TagPattern struct
#[no_mangle]
pub unsafe extern "C" fn rs_free_pats(pats: *mut TagPattern) {
    if pats.is_null() {
        return;
    }

    let pats = &mut *pats;

    if !pats.regmatch.regprog.is_null() {
        vim_regfree(pats.regmatch.regprog);
        pats.regmatch.regprog = ptr::null_mut();
    }
}

/// Check if a tag name matches the pattern.
///
/// Returns true if the tag name matches the pattern, considering:
/// - Whether the pattern uses regexp
/// - Whether case should be ignored
///
/// # Safety
///
/// - `pats` must be a valid pointer to a TagPattern struct
/// - `tagname` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_tag_pattern_matches(
    pats: *mut TagPattern,
    tagname: *const c_char,
    ignore_case: bool,
) -> bool {
    if pats.is_null() || tagname.is_null() {
        return false;
    }

    let pats = &mut *pats;

    // If we have a compiled regexp, use it
    if !pats.regmatch.regprog.is_null() {
        pats.regmatch.rm_ic = ignore_case;
        return vim_regexec(std::ptr::addr_of_mut!(pats.regmatch), tagname, 0);
    }

    // Otherwise do a simple string comparison
    let pat = pats.pat;
    if pat.is_null() {
        return false;
    }

    // Compare strings
    if ignore_case {
        compare_strings_icase(pat, tagname, pats.len as usize)
    } else {
        compare_strings(pat, tagname, pats.len as usize)
    }
}

/// Compare two strings for equality (case-sensitive).
unsafe fn compare_strings(s1: *const c_char, s2: *const c_char, len: usize) -> bool {
    for i in 0..len {
        let c1 = *s1.add(i);
        let c2 = *s2.add(i);
        if c2 == 0 || c1 != c2 {
            return false;
        }
    }
    true
}

/// Compare two strings for equality (case-insensitive).
unsafe fn compare_strings_icase(s1: *const c_char, s2: *const c_char, len: usize) -> bool {
    for i in 0..len {
        let c1 = (*s1.add(i) as u8).to_ascii_lowercase();
        let c2 = *s2.add(i) as u8;
        if c2 == 0 || c1 != c2.to_ascii_lowercase() {
            return false;
        }
    }
    true
}

/// Check if the head of the pattern matches the beginning of the tag name.
///
/// This is used for the initial binary search filter before doing full
/// pattern matching.
///
/// # Safety
///
/// - `pats` must be a valid pointer to a TagPattern struct
/// - `tagname` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_tag_head_matches(
    pats: *const TagPattern,
    tagname: *const c_char,
    ignore_case: bool,
) -> bool {
    if pats.is_null() || tagname.is_null() {
        return false;
    }

    let pats = &*pats;

    if pats.headlen == 0 {
        return true; // Empty head matches everything
    }

    if pats.head.is_null() {
        return false;
    }

    if ignore_case {
        compare_strings_icase(pats.head, tagname, pats.headlen as usize)
    } else {
        compare_strings(pats.head, tagname, pats.headlen as usize)
    }
}

// =============================================================================
// TagPattern accessor functions
// =============================================================================

/// Create a new TagPattern structure.
#[no_mangle]
pub extern "C" fn rs_tag_pattern_new() -> *mut TagPattern {
    Box::into_raw(Box::new(TagPattern::default()))
}

/// Free a TagPattern structure.
///
/// # Safety
///
/// - `pats` must have been created by `rs_tag_pattern_new`
#[no_mangle]
pub unsafe extern "C" fn rs_tag_pattern_free(pats: *mut TagPattern) {
    if !pats.is_null() {
        // Free the regexp if it exists
        rs_free_pats(pats);
        drop(Box::from_raw(pats));
    }
}

/// Initialize a TagPattern structure with a pattern string.
///
/// # Safety
///
/// - `pats` must be a valid pointer
/// - `pat` must be a valid null-terminated C string
/// - The pattern string must remain valid for the lifetime of the TagPattern
#[no_mangle]
pub unsafe extern "C" fn rs_tag_pattern_init(pats: *mut TagPattern, pat: *mut c_char, len: c_int) {
    if pats.is_null() {
        return;
    }

    let pats = &mut *pats;
    pats.pat = pat;
    pats.len = len;
    pats.head = pat;
    pats.headlen = len;
    pats.regmatch = RegMatch::default();
}

/// Get the pattern string from TagPattern.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_pattern_get_pat(pats: *const TagPattern) -> *const c_char {
    if pats.is_null() {
        return ptr::null();
    }
    (*pats).pat
}

/// Get the pattern length from TagPattern.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_pattern_get_len(pats: *const TagPattern) -> c_int {
    if pats.is_null() {
        return 0;
    }
    (*pats).len
}

/// Get the head pointer from TagPattern.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_pattern_get_head(pats: *const TagPattern) -> *const c_char {
    if pats.is_null() {
        return ptr::null();
    }
    (*pats).head
}

/// Get the head length from TagPattern.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_pattern_get_headlen(pats: *const TagPattern) -> c_int {
    if pats.is_null() {
        return 0;
    }
    (*pats).headlen
}

/// Check if the pattern has a compiled regexp.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_pattern_has_regexp(pats: *const TagPattern) -> bool {
    if pats.is_null() {
        return false;
    }
    !(*pats).regmatch.regprog.is_null()
}

/// Set the ignore-case flag for pattern matching.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_pattern_set_ic(pats: *mut TagPattern, ic: bool) {
    if pats.is_null() {
        return;
    }
    (*pats).regmatch.rm_ic = ic;
}

/// Get the ignore-case flag from pattern matching.
#[no_mangle]
pub unsafe extern "C" fn rs_tag_pattern_get_ic(pats: *const TagPattern) -> bool {
    if pats.is_null() {
        return false;
    }
    (*pats).regmatch.rm_ic
}

// =============================================================================
// Helper functions for pattern matching
// =============================================================================

/// Check if a character is a pattern metacharacter.
///
/// Returns true if the character could have special meaning in a pattern.
#[no_mangle]
pub unsafe extern "C" fn rs_is_pattern_meta(c: c_int, use_magic: bool) -> bool {
    let c = c as u8;
    if use_magic {
        matches!(c, b'.' | b'[' | b'~' | b'*' | b'\\' | b'$')
    } else {
        matches!(c, b'\\' | b'$')
    }
}

/// Calculate the length of the fixed prefix in a pattern.
///
/// Returns the length of the portion that can be used for binary searching.
#[no_mangle]
pub unsafe extern "C" fn rs_pattern_head_len(pat: *const c_char, use_magic: bool) -> c_int {
    if pat.is_null() {
        return 0;
    }

    let mut len = 0;
    let mut p = pat;

    // Skip leading ^ or \<
    if *p == b'^' as c_char {
        p = p.add(1);
    } else if *p == b'\\' as c_char && *p.add(1) == b'<' as c_char {
        p = p.add(2);
    } else {
        return 0; // No fixed prefix
    }

    while *p != 0 {
        if rs_is_pattern_meta(*p as c_int, use_magic) {
            break;
        }
        len += 1;
        p = p.add(1);
    }

    len
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_pattern_default() {
        let pats = TagPattern::default();
        assert!(pats.pat.is_null());
        assert_eq!(pats.len, 0);
        assert!(pats.head.is_null());
        assert_eq!(pats.headlen, 0);
        assert!(pats.regmatch.regprog.is_null());
    }

    #[test]
    fn test_regmatch_default() {
        let rm = RegMatch::default();
        assert!(rm.regprog.is_null());
        assert_eq!(rm.rm_matchcol, 0);
        assert!(!rm.rm_ic);
        for i in 0..NSUBEXP {
            assert!(rm.startp[i].is_null());
            assert!(rm.endp[i].is_null());
        }
    }

    #[test]
    fn test_tag_pattern_new_free() {
        let pats = rs_tag_pattern_new();
        assert!(!pats.is_null());
        unsafe {
            assert!((*pats).pat.is_null());
            rs_tag_pattern_free(pats);
        }
    }

    #[test]
    fn test_accessors_null() {
        unsafe {
            assert!(rs_tag_pattern_get_pat(ptr::null()).is_null());
            assert_eq!(rs_tag_pattern_get_len(ptr::null()), 0);
            assert!(rs_tag_pattern_get_head(ptr::null()).is_null());
            assert_eq!(rs_tag_pattern_get_headlen(ptr::null()), 0);
            assert!(!rs_tag_pattern_has_regexp(ptr::null()));
            assert!(!rs_tag_pattern_get_ic(ptr::null()));
        }
    }

    #[test]
    fn test_is_pattern_meta() {
        unsafe {
            // With magic
            assert!(rs_is_pattern_meta(b'.' as c_int, true));
            assert!(rs_is_pattern_meta(b'[' as c_int, true));
            assert!(rs_is_pattern_meta(b'*' as c_int, true));
            assert!(rs_is_pattern_meta(b'$' as c_int, true));
            assert!(!rs_is_pattern_meta(b'a' as c_int, true));

            // Without magic
            assert!(!rs_is_pattern_meta(b'.' as c_int, false));
            assert!(rs_is_pattern_meta(b'\\' as c_int, false));
            assert!(rs_is_pattern_meta(b'$' as c_int, false));
        }
    }

    #[test]
    fn test_compare_strings() {
        unsafe {
            let s1 = c"hello".as_ptr();
            let s2 = c"hello".as_ptr();
            let s3 = c"world".as_ptr();
            let s4 = c"HELLO".as_ptr();

            assert!(compare_strings(s1, s2, 5));
            assert!(!compare_strings(s1, s3, 5));
            assert!(!compare_strings(s1, s4, 5)); // case sensitive

            assert!(compare_strings_icase(s1, s2, 5));
            assert!(!compare_strings_icase(s1, s3, 5));
            assert!(compare_strings_icase(s1, s4, 5)); // case insensitive
        }
    }

    #[test]
    fn test_tag_head_matches_null() {
        unsafe {
            assert!(!rs_tag_head_matches(ptr::null(), c"test".as_ptr(), false));
            let pats = rs_tag_pattern_new();
            assert!(!rs_tag_head_matches(pats, ptr::null(), false));
            // Empty head matches everything
            assert!(rs_tag_head_matches(pats, c"anything".as_ptr(), false));
            rs_tag_pattern_free(pats);
        }
    }

    #[test]
    fn test_tag_pattern_matches_null() {
        unsafe {
            assert!(!rs_tag_pattern_matches(
                ptr::null_mut(),
                c"test".as_ptr(),
                false
            ));
            let pats = rs_tag_pattern_new();
            assert!(!rs_tag_pattern_matches(pats, ptr::null(), false));
            rs_tag_pattern_free(pats);
        }
    }

    #[test]
    fn test_pattern_head_len_null() {
        unsafe {
            assert_eq!(rs_pattern_head_len(ptr::null(), true), 0);
        }
    }
}
