//! Help system utilities for Neovim
//!
//! This module provides Rust implementations of help system functions from
//! `src/nvim/help.c`, including search heuristics, tag comparison, and
//! language detection.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};

/// Check if a byte is an ASCII alphanumeric character (0-9, a-z, A-Z).
#[inline]
fn ascii_isalnum(c: u8) -> bool {
    c.is_ascii_alphanumeric()
}

/// Calculate a heuristic score for how well a matched string matches a help query.
///
/// The scoring considers:
/// - Langstroth's strnicmp algorithm
/// - More langnum letters is langbetter
/// - Match towards the start is better
/// - Match starting with "+" is worse (feature instead of command)
///
/// # Safety
/// The `matched_string` pointer must be valid and point to a null-terminated C string.
///
/// # Arguments
/// * `matched_string` - The matched help tag string
/// * `offset` - Offset into the string where the match occurred
/// * `wrong_case` - True if matching was case-insensitive
///
/// # Returns
/// A heuristic score (lower is better)
#[no_mangle]
pub unsafe extern "C" fn rs_help_heuristic(
    matched_string: *const c_char,
    offset: c_int,
    wrong_case: bool,
) -> c_int {
    if matched_string.is_null() {
        return c_int::MAX;
    }

    let cstr = unsafe { CStr::from_ptr(matched_string) };
    let bytes = cstr.to_bytes();

    // Count alphanumeric characters
    let num_letters = bytes.iter().filter(|&&c| ascii_isalnum(c)).count() as c_int;

    let mut offset_score = offset;

    // If the match starts in the middle of a word, add 10000 to put it
    // somewhere in the last half.
    // If the match is more than 2 chars from the start, multiply by 200 to
    // put it after matches at the start.
    if offset > 0 {
        let offset_usize = offset as usize;
        if offset_usize < bytes.len()
            && offset_usize > 0
            && ascii_isalnum(bytes[offset_usize])
            && ascii_isalnum(bytes[offset_usize - 1])
        {
            offset_score += 10000;
        } else if offset > 2 {
            offset_score *= 200;
        }
    }

    // If there only is a match while ignoring case, add 5000.
    if wrong_case {
        offset_score += 5000;
    }

    // Features are less interesting than the subjects themselves, but "+"
    // alone is not a feature.
    if !bytes.is_empty() && bytes[0] == b'+' && bytes.len() > 1 {
        offset_score += 100;
    }

    // Multiply the number of letters by 100 to give it a much bigger
    // weighting than the number of characters.
    100 * num_letters + (bytes.len() as c_int) + offset_score
}

/// Parse `@xx` language suffix from a help argument.
///
/// If the argument ends with `@xx` where both characters are ASCII alphabetic,
/// sets a NUL byte at the `@` position and returns a pointer to the two-letter
/// language code. Otherwise returns null.
///
/// # Safety
/// `arg` must be a valid, mutable, NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_check_help_lang(arg: *mut c_char) -> *mut c_char {
    if arg.is_null() {
        return std::ptr::null_mut();
    }

    let bytes = unsafe { CStr::from_ptr(arg) }.to_bytes();
    let len = bytes.len();

    if len >= 3 {
        let at_pos = len - 3;
        if bytes[at_pos] == b'@'
            && bytes[at_pos + 1].is_ascii_alphabetic()
            && bytes[at_pos + 2].is_ascii_alphabetic()
        {
            // Set NUL at the '@' position to truncate the string
            unsafe { *arg.add(at_pos) = 0 };
            // Return pointer to the two-letter language code
            return unsafe { arg.add(at_pos + 1) };
        }
    }

    std::ptr::null_mut()
}

/// Compare function for qsort() used by find_help_tags().
///
/// Each match string has a heuristic number stored after the tag name's NUL byte.
/// We compare by that heuristic number first, then by the tag string as a tie-breaker.
///
/// # Safety
/// `s1` and `s2` must point to valid `*const c_char` pointers (i.e., `char **`),
/// and each pointed-to string must be NUL-terminated with a second NUL-terminated
/// string immediately following.
#[no_mangle]
pub unsafe extern "C" fn rs_help_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    let p1_str = unsafe { *(s1 as *const *const c_char) };
    let p2_str = unsafe { *(s2 as *const *const c_char) };

    // Find the heuristic number stored after the tag name's NUL byte.
    let p1_len = unsafe { CStr::from_ptr(p1_str) }.to_bytes().len();
    let p2_len = unsafe { CStr::from_ptr(p2_str) }.to_bytes().len();

    let p1_heur = unsafe { p1_str.add(p1_len + 1) };
    let p2_heur = unsafe { p2_str.add(p2_len + 1) };

    // Compare by heuristic number first.
    let cmp = unsafe { libc::strcmp(p1_heur, p2_heur) };
    if cmp != 0 {
        return cmp;
    }

    // Compare by tag strings as tie-breaker.
    unsafe { libc::strcmp(p1_str, p2_str) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn test_str(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn test_help_heuristic_basic() {
        unsafe {
            // Simple match at start
            let score1 = rs_help_heuristic(test_str("abc").as_ptr(), 0, false);
            // 3 letters * 100 + 3 length + 0 offset = 303
            assert_eq!(score1, 303);
        }
    }

    #[test]
    fn test_help_heuristic_wrong_case() {
        unsafe {
            let score_correct = rs_help_heuristic(test_str("abc").as_ptr(), 0, false);
            let score_wrong = rs_help_heuristic(test_str("abc").as_ptr(), 0, true);
            // Wrong case adds 5000
            assert_eq!(score_wrong, score_correct + 5000);
        }
    }

    #[test]
    fn test_help_heuristic_offset() {
        unsafe {
            // Match at start
            let score_start = rs_help_heuristic(test_str("abc").as_ptr(), 0, false);

            // Match at offset 3 (more than 2, multiplies by 200)
            let score_offset3 = rs_help_heuristic(test_str("abc").as_ptr(), 3, false);
            // 303 + 3*200 = 903
            assert_eq!(score_offset3, 303 + 3 * 200);

            // Verify start is better (lower score)
            assert!(score_start < score_offset3);
        }
    }

    #[test]
    fn test_help_heuristic_mid_word() {
        unsafe {
            // Match in middle of word (abcdef, offset 3, both d and c are alnum)
            let score = rs_help_heuristic(test_str("abcdef").as_ptr(), 3, false);
            // 6 letters * 100 + 6 length + 3 offset + 10000 = 10609
            assert_eq!(score, 10609);
        }
    }

    #[test]
    fn test_help_heuristic_feature() {
        unsafe {
            // Feature starting with "+" adds 100 penalty
            // Use strings with same number of alnum chars to isolate the feature penalty
            // "+abc" has 3 alnum chars (a, b, c), "-abc" also has 3 alnum chars
            let score_feature = rs_help_heuristic(test_str("+abc").as_ptr(), 0, false);
            let score_normal = rs_help_heuristic(test_str("-abc").as_ptr(), 0, false);
            // Both have: 3 letters * 100 + 4 length = 304
            // Feature adds 100: 304 + 100 = 404
            assert_eq!(score_normal, 304);
            assert_eq!(score_feature, 404);
        }
    }

    #[test]
    fn test_help_heuristic_plus_alone() {
        unsafe {
            // "+" alone is not treated as feature
            let score_plus = rs_help_heuristic(test_str("+").as_ptr(), 0, false);
            // 0 letters * 100 + 1 length + 0 offset = 1
            assert_eq!(score_plus, 1);
        }
    }

    #[test]
    fn test_help_heuristic_null() {
        unsafe {
            let score = rs_help_heuristic(std::ptr::null(), 0, false);
            assert_eq!(score, c_int::MAX);
        }
    }

    #[test]
    fn test_help_heuristic_empty_string() {
        unsafe {
            let score = rs_help_heuristic(test_str("").as_ptr(), 0, false);
            // 0 letters * 100 + 0 length + 0 offset = 0
            assert_eq!(score, 0);
        }
    }

    #[test]
    fn test_help_heuristic_non_alpha() {
        unsafe {
            // String with no alphanumeric characters
            let score = rs_help_heuristic(test_str("---").as_ptr(), 0, false);
            // 0 letters * 100 + 3 length + 0 offset = 3
            assert_eq!(score, 3);
        }
    }

    #[test]
    fn test_check_help_lang_with_suffix() {
        unsafe {
            let s = CString::new("foo@en").unwrap();
            let ptr = s.into_raw();
            let lang = rs_check_help_lang(ptr);
            assert!(!lang.is_null());
            // The language code should be "en"
            assert_eq!(*lang as u8, b'e');
            assert_eq!(*lang.add(1) as u8, b'n');
            // The original string should be truncated to "foo"
            let truncated = CStr::from_ptr(ptr);
            assert_eq!(truncated.to_bytes(), b"foo");
            // Clean up
            let _ = CString::from_raw(ptr);
        }
    }

    #[test]
    fn test_check_help_lang_too_short() {
        unsafe {
            let s = CString::new("ab").unwrap();
            let ptr = s.into_raw();
            let lang = rs_check_help_lang(ptr);
            assert!(lang.is_null());
            let _ = CString::from_raw(ptr);
        }
    }

    #[test]
    fn test_check_help_lang_non_alpha() {
        unsafe {
            let s = CString::new("foo@1x").unwrap();
            let ptr = s.into_raw();
            let lang = rs_check_help_lang(ptr);
            assert!(lang.is_null());
            let _ = CString::from_raw(ptr);
        }
    }

    #[test]
    fn test_check_help_lang_no_at() {
        unsafe {
            let s = CString::new("foobar").unwrap();
            let ptr = s.into_raw();
            let lang = rs_check_help_lang(ptr);
            assert!(lang.is_null());
            let _ = CString::from_raw(ptr);
        }
    }

    #[test]
    fn test_check_help_lang_null() {
        unsafe {
            let lang = rs_check_help_lang(std::ptr::null_mut());
            assert!(lang.is_null());
        }
    }

    #[test]
    fn test_check_help_lang_exactly_three() {
        unsafe {
            // "@en" is exactly 3 chars - the arg part before @ is empty
            let s = CString::new("@en").unwrap();
            let ptr = s.into_raw();
            let lang = rs_check_help_lang(ptr);
            assert!(!lang.is_null());
            assert_eq!(*lang as u8, b'e');
            assert_eq!(*lang.add(1) as u8, b'n');
            let truncated = CStr::from_ptr(ptr);
            assert_eq!(truncated.to_bytes(), b"");
            let _ = CString::from_raw(ptr);
        }
    }

    /// Helper to create a mock help tag match string with an embedded heuristic number.
    /// The format is: "tagname\0heuristic_number\0"
    fn make_help_match(tag: &str, heuristic: &str) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(tag.as_bytes());
        v.push(0); // NUL separator
        v.extend_from_slice(heuristic.as_bytes());
        v.push(0); // NUL terminator
        v
    }

    #[test]
    fn test_help_compare_different_heuristic() {
        unsafe {
            let m1 = make_help_match("tag_a", "0100");
            let m2 = make_help_match("tag_b", "0200");
            let p1 = m1.as_ptr() as *const c_char;
            let p2 = m2.as_ptr() as *const c_char;
            let result = rs_help_compare(
                &p1 as *const _ as *const c_void,
                &p2 as *const _ as *const c_void,
            );
            // "0100" < "0200" so result should be negative
            assert!(result < 0);

            // Reverse order
            let result2 = rs_help_compare(
                &p2 as *const _ as *const c_void,
                &p1 as *const _ as *const c_void,
            );
            assert!(result2 > 0);
        }
    }

    #[test]
    fn test_help_compare_same_heuristic_different_tag() {
        unsafe {
            let m1 = make_help_match("alpha", "0100");
            let m2 = make_help_match("beta", "0100");
            let p1 = m1.as_ptr() as *const c_char;
            let p2 = m2.as_ptr() as *const c_char;
            let result = rs_help_compare(
                &p1 as *const _ as *const c_void,
                &p2 as *const _ as *const c_void,
            );
            // Same heuristic, so compare by tag: "alpha" < "beta"
            assert!(result < 0);
        }
    }

    #[test]
    fn test_help_compare_identical() {
        unsafe {
            let m1 = make_help_match("same", "0100");
            let m2 = make_help_match("same", "0100");
            let p1 = m1.as_ptr() as *const c_char;
            let p2 = m2.as_ptr() as *const c_char;
            let result = rs_help_compare(
                &p1 as *const _ as *const c_void,
                &p2 as *const _ as *const c_void,
            );
            assert_eq!(result, 0);
        }
    }

    #[test]
    fn test_ascii_isalnum() {
        // Digits
        for c in b'0'..=b'9' {
            assert!(ascii_isalnum(c));
        }
        // Lowercase
        for c in b'a'..=b'z' {
            assert!(ascii_isalnum(c));
        }
        // Uppercase
        for c in b'A'..=b'Z' {
            assert!(ascii_isalnum(c));
        }
        // Non-alphanumeric
        assert!(!ascii_isalnum(b' '));
        assert!(!ascii_isalnum(b'-'));
        assert!(!ascii_isalnum(b'_'));
        assert!(!ascii_isalnum(b'+'));
    }
}
