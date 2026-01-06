//! Simple string option callback implementations
//!
//! This module contains Rust implementations of simple string option validation
//! callbacks. These callbacks primarily validate option values against allowed
//! sets of values or characters.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit

use std::ffi::c_char;

use super::{callback_ok, CallbackResult};

// =============================================================================
// Constants - Flag Characters for List Options
// =============================================================================

/// All valid flags for 'comments' option
const COM_ALL: &[u8] = b"nbsmexflrO";

/// All valid flags for 'concealcursor' option
const COCU_ALL: &[u8] = b"nvic";

/// All valid flags for 'cpoptions' option (vi compatibility)
const CPO_VI: &[u8] = b"aAbBcCdDeEfFiIJKlLmMnoOpPqrRsStuvWxXyZ$!%+>;~_";

/// All valid flags for 'formatoptions' option
const FO_ALL: &[u8] = b"tcro/q2vlb1mMBn,aw]jp";

/// All valid flags for 'mouse' option
const MOUSE_ALL: &[u8] = b"anvichr";

/// All valid flags for 'shortmess' option
const SHM_ALL: &[u8] = b"rwoOstTWIcCqaAFnlxfiS";

/// All valid flags for 'whichwrap' option
const WW_ALL: &[u8] = b"bshl<>[]~";

// =============================================================================
// Error Messages
// =============================================================================

/// Error: Invalid argument
const E_INVARG: *const c_char = c"E474: Invalid argument".as_ptr();

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if a byte is in a set of allowed bytes.
#[inline]
fn byte_in_set(b: u8, set: &[u8]) -> bool {
    set.contains(&b)
}

/// Validate that a string contains only characters from an allowed set.
/// Returns NULL on success, or an error message pointer on failure.
#[inline]
unsafe fn validate_listflag(value: *const c_char, allowed: &[u8]) -> CallbackResult {
    if value.is_null() {
        return callback_ok();
    }

    let mut p = value;
    while *p != 0 {
        let ch = *p as u8;
        if !byte_in_set(ch, allowed) {
            return E_INVARG;
        }
        p = p.add(1);
    }
    callback_ok()
}

// =============================================================================
// List Flag Option Validators
// =============================================================================

/// Validate 'concealcursor' option value.
/// Valid characters are: n, v, i, c
#[no_mangle]
pub unsafe extern "C" fn rs_validate_concealcursor(value: *const c_char) -> CallbackResult {
    validate_listflag(value, COCU_ALL)
}

/// Validate 'cpoptions' option value.
/// Valid characters are the vi compatibility flags.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_cpoptions(value: *const c_char) -> CallbackResult {
    validate_listflag(value, CPO_VI)
}

/// Validate 'formatoptions' option value.
/// Valid characters are the text formatting flags.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_formatoptions(value: *const c_char) -> CallbackResult {
    validate_listflag(value, FO_ALL)
}

/// Validate 'mouse' option value.
/// Valid characters are: a, n, v, i, c, h, r
#[no_mangle]
pub unsafe extern "C" fn rs_validate_mouse(value: *const c_char) -> CallbackResult {
    validate_listflag(value, MOUSE_ALL)
}

/// Validate 'shortmess' option value.
/// Valid characters are the message shortening flags.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_shortmess(value: *const c_char) -> CallbackResult {
    validate_listflag(value, SHM_ALL)
}

/// Validate 'whichwrap' option value.
/// Valid characters are: b, s, h, l, <, >, [, ], ~
#[no_mangle]
pub unsafe extern "C" fn rs_validate_whichwrap(value: *const c_char) -> CallbackResult {
    validate_listflag(value, WW_ALL)
}

// =============================================================================
// 'comments' Option Validation
// =============================================================================

/// Validate 'comments' option value.
/// Format: {flags}:{text},{flags}:{text},...
/// Returns NULL on success, or an error message pointer on failure.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_comments(value: *const c_char) -> CallbackResult {
    if value.is_null() {
        return callback_ok();
    }

    let mut s = value;

    while *s != 0 {
        // Parse flags before the colon
        while *s != 0 && *s as u8 != b':' {
            let ch = *s as u8;
            // Flags must be in COM_ALL, or a digit, or '-'
            if !byte_in_set(ch, COM_ALL) && !ch.is_ascii_digit() && ch != b'-' {
                return E_INVARG;
            }
            s = s.add(1);
        }

        // Must have a colon
        if *s as u8 != b':' {
            // E524: Missing colon
            return c"E524: Missing colon".as_ptr();
        }
        s = s.add(1);

        // Must have text after colon (not empty, not starting with comma)
        if *s as u8 == b',' || *s == 0 {
            // E525: Zero length string
            return c"E525: Zero length string".as_ptr();
        }

        // Skip to next part (after comma)
        while *s != 0 && *s as u8 != b',' {
            // Handle backslash escapes
            if *s as u8 == b'\\' && *s.add(1) != 0 {
                s = s.add(1);
            }
            s = s.add(1);
        }

        // Skip the comma and whitespace
        if *s as u8 == b',' {
            s = s.add(1);
            while *s as u8 == b' ' {
                s = s.add(1);
            }
        }
    }

    callback_ok()
}

// =============================================================================
// 'commentstring' Option Validation
// =============================================================================

/// Validate 'commentstring' option value.
/// Must be empty or contain %s.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_commentstring(value: *const c_char) -> CallbackResult {
    if value.is_null() {
        return callback_ok();
    }

    // Empty string is OK
    if *value == 0 {
        return callback_ok();
    }

    // Must contain %s
    let mut p = value;
    while *p != 0 {
        if *p as u8 == b'%' && *p.add(1) as u8 == b's' {
            return callback_ok();
        }
        p = p.add(1);
    }

    c"E537: 'commentstring' must be empty or contain %s".as_ptr()
}

// =============================================================================
// 'complete' Option Validation
// =============================================================================

/// Valid first characters for 'complete' option items
const COMPLETE_CHARS: &[u8] = b".wbuksid]tUfFo";

/// Characters that can have additional arguments in 'complete' option
const COMPLETE_WITH_ARGS: &[u8] = b"ksF";

/// Validate 'complete' option value.
/// Format: comma-separated list of completion sources.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_complete(value: *const c_char) -> CallbackResult {
    if value.is_null() {
        return callback_ok();
    }

    let mut p = value;

    while *p != 0 {
        // Skip escaped commas and collect the item
        let item_start = p;
        let mut escape = false;

        while *p != 0 && (*p as u8 != b',' || escape) {
            if *p as u8 == b'\\' && *p.add(1) as u8 == b',' {
                escape = true;
                p = p.add(1);
            } else {
                escape = false;
            }
            p = p.add(1);
        }

        // Check the first character of the item
        let first = *item_start as u8;
        if !byte_in_set(first, COMPLETE_CHARS) {
            return E_INVARG;
        }

        // For items not in COMPLETE_WITH_ARGS, only allow '^' as second char
        if !byte_in_set(first, COMPLETE_WITH_ARGS) {
            let second = *item_start.add(1) as u8;
            if second != 0 && second != b',' && second != b'^' {
                return E_INVARG;
            }
        }

        // Skip comma
        if *p as u8 == b',' {
            p = p.add(1);
        }
    }

    callback_ok()
}

// =============================================================================
// 'matchpairs' Option Validation
// =============================================================================

/// Validate 'matchpairs' option value.
/// Format: x:y,a:b,... where x and y are single characters (or multibyte).
#[no_mangle]
pub unsafe extern "C" fn rs_validate_matchpairs(value: *const c_char) -> CallbackResult {
    if value.is_null() {
        return callback_ok();
    }

    let mut p = value;

    while *p != 0 {
        // Skip first character (possibly multibyte)
        if *p == 0 {
            return E_INVARG;
        }

        // For ASCII, just skip one byte
        // For multibyte, we should use utf_ptr2len but for simplicity
        // we'll check if it's ASCII or skip until we find ':'
        if (*p as u8) < 0x80 {
            p = p.add(1);
        } else {
            // Skip multibyte char - find next ':' or ','
            while *p != 0 && *p as u8 != b':' && *p as u8 != b',' {
                p = p.add(1);
            }
        }

        // Expect ':'
        if *p as u8 != b':' {
            return E_INVARG;
        }
        p = p.add(1);

        // Skip second character
        if *p == 0 {
            return E_INVARG;
        }

        if (*p as u8) < 0x80 {
            p = p.add(1);
        } else {
            // Skip multibyte char
            while *p != 0 && *p as u8 != b',' {
                p = p.add(1);
            }
        }

        // Expect ',' or end
        if *p != 0 {
            if *p as u8 != b',' {
                return E_INVARG;
            }
            p = p.add(1);
        }
    }

    callback_ok()
}

// =============================================================================
// 'backspace' Option Validation
// =============================================================================

/// Validate 'backspace' option value.
/// Can be "2" (numeric) or a comma-separated list of keywords.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_backspace(value: *const c_char) -> CallbackResult {
    if value.is_null() {
        return callback_ok();
    }

    let first = *value as u8;

    // Numeric form: only "2" is valid
    if first.is_ascii_digit() {
        if first != b'2' || *value.add(1) != 0 {
            return E_INVARG;
        }
        return callback_ok();
    }

    // Otherwise, it's validated by did_set_str_generic in C
    // (which checks against the allowed value list)
    callback_ok()
}

// =============================================================================
// 'foldmarker' Option Validation
// =============================================================================

/// Validate 'foldmarker' option value.
/// Format: start,end (two markers separated by comma).
#[no_mangle]
pub unsafe extern "C" fn rs_validate_foldmarker(value: *const c_char) -> CallbackResult {
    if value.is_null() {
        return callback_ok();
    }

    // Find the comma
    let mut p = value;
    while *p != 0 && *p as u8 != b',' {
        p = p.add(1);
    }

    // Must have a comma
    if *p as u8 != b',' {
        return E_INVARG;
    }

    // Must have text before comma
    if p == value {
        return E_INVARG;
    }

    // Must have text after comma
    if *p.add(1) == 0 {
        return E_INVARG;
    }

    callback_ok()
}

// =============================================================================
// 'lispoptions' Option Validation
// =============================================================================

/// Validate 'lispoptions' option value.
/// Valid values: empty, "expr:0", or "expr:1"
#[no_mangle]
pub unsafe extern "C" fn rs_validate_lispoptions(value: *const c_char) -> CallbackResult {
    if value.is_null() || *value == 0 {
        return callback_ok();
    }

    // Check for "expr:0" or "expr:1"
    // First check the length to avoid reading out of bounds
    let mut len = 0;
    let mut p = value;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    if len == 6 {
        let bytes = std::slice::from_raw_parts(value.cast::<u8>(), 6);
        if bytes[0] == b'e'
            && bytes[1] == b'x'
            && bytes[2] == b'p'
            && bytes[3] == b'r'
            && bytes[4] == b':'
            && (bytes[5] == b'0' || bytes[5] == b'1')
        {
            return callback_ok();
        }
    }

    E_INVARG
}

// =============================================================================
// 'breakindentopt' Option Validation
// =============================================================================

/// Validate 'breakindentopt' option value.
/// Format: comma-separated list of keywords with optional :N suffix.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_breakindentopt(value: *const c_char) -> CallbackResult {
    if value.is_null() || *value == 0 {
        return callback_ok();
    }

    // Keywords: min, shift, sbr, list, column
    // Format: keyword or keyword:N
    let valid_keywords: &[&[u8]] = &[b"min", b"shift", b"sbr", b"list", b"column"];

    let mut p = value;

    while *p != 0 {
        // Find end of this keyword
        let start = p;
        while *p != 0 && *p as u8 != b':' && *p as u8 != b',' {
            p = p.add(1);
        }

        let keyword_len = p.offset_from(start) as usize;

        // Check if keyword is valid
        let mut found = false;
        for kw in valid_keywords {
            if keyword_len == kw.len() {
                let mut matches = true;
                for i in 0..keyword_len {
                    if *start.add(i) as u8 != kw[i] {
                        matches = false;
                        break;
                    }
                }
                if matches {
                    found = true;
                    break;
                }
            }
        }

        if !found {
            return E_INVARG;
        }

        // Optional :N suffix
        if *p as u8 == b':' {
            p = p.add(1);
            // Skip optional minus sign
            if *p as u8 == b'-' {
                p = p.add(1);
            }
            // Must have at least one digit
            if !(*p as u8).is_ascii_digit() {
                return E_INVARG;
            }
            while (*p as u8).is_ascii_digit() {
                p = p.add(1);
            }
        }

        // Skip comma
        if *p as u8 == b',' {
            p = p.add(1);
        }
    }

    callback_ok()
}

// =============================================================================
// 'spelloptions' Option Validation
// =============================================================================

/// Validate 'spelloptions' option value.
/// Valid values: empty, "camel", or "noplainbuffer"
#[no_mangle]
pub unsafe extern "C" fn rs_validate_spelloptions(value: *const c_char) -> CallbackResult {
    if value.is_null() || *value == 0 {
        return callback_ok();
    }

    // Validate by checking against allowed values
    // Allowed: camel, noplainbuffer (comma-separated)
    let mut p = value;

    while *p != 0 {
        let start = p;

        // Find end of item
        while *p != 0 && *p as u8 != b',' {
            p = p.add(1);
        }

        let len = p.offset_from(start) as usize;

        // Check valid values
        let valid = match len {
            5 => {
                // "camel"
                *start as u8 == b'c'
                    && *start.add(1) as u8 == b'a'
                    && *start.add(2) as u8 == b'm'
                    && *start.add(3) as u8 == b'e'
                    && *start.add(4) as u8 == b'l'
            }
            13 => {
                // "noplainbuffer"
                let expected = b"noplainbuffer";
                let mut matches = true;
                for (i, &expected_byte) in expected.iter().enumerate() {
                    if *start.add(i) as u8 != expected_byte {
                        matches = false;
                        break;
                    }
                }
                matches
            }
            _ => false,
        };

        if !valid {
            return E_INVARG;
        }

        // Skip comma
        if *p as u8 == b',' {
            p = p.add(1);
        }
    }

    callback_ok()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn test_validate_concealcursor() {
        unsafe {
            let valid = CString::new("nvic").unwrap();
            let invalid = CString::new("xyz").unwrap();
            let empty = CString::new("").unwrap();

            assert!(rs_validate_concealcursor(valid.as_ptr()).is_null());
            assert!(!rs_validate_concealcursor(invalid.as_ptr()).is_null());
            assert!(rs_validate_concealcursor(empty.as_ptr()).is_null());
            assert!(rs_validate_concealcursor(ptr::null()).is_null());
        }
    }

    #[test]
    fn test_validate_formatoptions() {
        unsafe {
            let valid = CString::new("tcroq").unwrap();
            let invalid = CString::new("xyz").unwrap();

            assert!(rs_validate_formatoptions(valid.as_ptr()).is_null());
            assert!(!rs_validate_formatoptions(invalid.as_ptr()).is_null());
        }
    }

    #[test]
    fn test_validate_mouse() {
        unsafe {
            let valid = CString::new("a").unwrap();
            let valid2 = CString::new("nvi").unwrap();
            let invalid = CString::new("xyz").unwrap();

            assert!(rs_validate_mouse(valid.as_ptr()).is_null());
            assert!(rs_validate_mouse(valid2.as_ptr()).is_null());
            assert!(!rs_validate_mouse(invalid.as_ptr()).is_null());
        }
    }

    #[test]
    fn test_validate_comments() {
        unsafe {
            // Valid: flags:text
            let valid = CString::new("s1:/*,mb:*,ex:*/").unwrap();
            let valid2 = CString::new(":#").unwrap();

            // Invalid: missing colon
            let no_colon = CString::new("abc").unwrap();

            // Invalid: zero length text
            let zero_len = CString::new(":").unwrap();

            assert!(rs_validate_comments(valid.as_ptr()).is_null());
            assert!(rs_validate_comments(valid2.as_ptr()).is_null());
            assert!(!rs_validate_comments(no_colon.as_ptr()).is_null());
            assert!(!rs_validate_comments(zero_len.as_ptr()).is_null());
        }
    }

    #[test]
    fn test_validate_commentstring() {
        unsafe {
            let valid = CString::new("/* %s */").unwrap();
            let valid_empty = CString::new("").unwrap();
            let invalid = CString::new("no percent s").unwrap();

            assert!(rs_validate_commentstring(valid.as_ptr()).is_null());
            assert!(rs_validate_commentstring(valid_empty.as_ptr()).is_null());
            assert!(!rs_validate_commentstring(invalid.as_ptr()).is_null());
        }
    }

    #[test]
    fn test_validate_complete() {
        unsafe {
            let valid = CString::new(".,w,b,u,t").unwrap();
            let valid2 = CString::new("kspell").unwrap();
            let invalid = CString::new("x").unwrap();

            assert!(rs_validate_complete(valid.as_ptr()).is_null());
            assert!(rs_validate_complete(valid2.as_ptr()).is_null());
            assert!(!rs_validate_complete(invalid.as_ptr()).is_null());
        }
    }

    #[test]
    fn test_validate_backspace() {
        unsafe {
            let valid_2 = CString::new("2").unwrap();
            let invalid_1 = CString::new("1").unwrap();
            let invalid_3 = CString::new("3").unwrap();
            let keywords = CString::new("indent,eol,start").unwrap();

            assert!(rs_validate_backspace(valid_2.as_ptr()).is_null());
            assert!(!rs_validate_backspace(invalid_1.as_ptr()).is_null());
            assert!(!rs_validate_backspace(invalid_3.as_ptr()).is_null());
            // Keywords are validated by C side
            assert!(rs_validate_backspace(keywords.as_ptr()).is_null());
        }
    }

    #[test]
    fn test_validate_foldmarker() {
        unsafe {
            let valid = CString::new("{{{,}}}").unwrap();
            let no_comma = CString::new("{{{").unwrap();
            let empty_before = CString::new(",}}}").unwrap();
            let empty_after = CString::new("{{{,").unwrap();

            assert!(rs_validate_foldmarker(valid.as_ptr()).is_null());
            assert!(!rs_validate_foldmarker(no_comma.as_ptr()).is_null());
            assert!(!rs_validate_foldmarker(empty_before.as_ptr()).is_null());
            assert!(!rs_validate_foldmarker(empty_after.as_ptr()).is_null());
        }
    }

    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn test_validate_matchpairs() {
        unsafe {
            // Note: {:} looks like format args but is actually matchpairs syntax
            let valid = CString::new("(:),{:},[:]").unwrap();
            let invalid = CString::new("abc").unwrap();

            assert!(rs_validate_matchpairs(valid.as_ptr()).is_null());
            assert!(!rs_validate_matchpairs(invalid.as_ptr()).is_null());
        }
    }

    #[test]
    fn test_validate_breakindentopt() {
        unsafe {
            let valid = CString::new("min:20,shift:4").unwrap();
            let valid2 = CString::new("sbr").unwrap();
            let invalid = CString::new("invalid").unwrap();

            assert!(rs_validate_breakindentopt(valid.as_ptr()).is_null());
            assert!(rs_validate_breakindentopt(valid2.as_ptr()).is_null());
            assert!(!rs_validate_breakindentopt(invalid.as_ptr()).is_null());
        }
    }

    #[test]
    fn test_validate_spelloptions() {
        unsafe {
            let valid = CString::new("camel").unwrap();
            let valid2 = CString::new("noplainbuffer").unwrap();
            let valid3 = CString::new("camel,noplainbuffer").unwrap();
            let invalid = CString::new("invalid").unwrap();
            let empty = CString::new("").unwrap();

            assert!(rs_validate_spelloptions(valid.as_ptr()).is_null());
            assert!(rs_validate_spelloptions(valid2.as_ptr()).is_null());
            assert!(rs_validate_spelloptions(valid3.as_ptr()).is_null());
            assert!(!rs_validate_spelloptions(invalid.as_ptr()).is_null());
            assert!(rs_validate_spelloptions(empty.as_ptr()).is_null());
        }
    }
}
