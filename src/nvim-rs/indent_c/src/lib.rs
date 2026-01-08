//! C/C++/Java indentation (cindent) for Neovim.
//!
//! This crate provides Rust implementations of C-style indentation logic
//! from `src/nvim/indent_c.c`. It handles smart indentation for C, C++, and Java
//! code using the 'cindent' feature.
//!
//! ## Architecture
//!
//! The crate uses an opaque handle pattern where `buf_T*` and `win_T*` pointers
//! are treated as opaque handles, with field access done through C accessor
//! functions.
//!
//! ## Key Components
//!
//! - Position and context utilities (comment/string detection)
//! - Bracket and brace matching
//! - Statement analysis (case, label, preprocessor detection)
//! - Indentation calculation core

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(dead_code)] // Some items are for future phases

use std::ffi::{c_char, c_int};

// Re-export handle types from dependencies
pub use nvim_buffer::BufHandle;
pub use nvim_window::WinHandle;

// ============================================================================
// Constants
// ============================================================================

/// NUL character (end of string).
const NUL: c_char = 0;

/// Space character (used in future phases).
const SPACE: c_char = b' ' as c_char;

/// Tab character (used in future phases).
const TAB: c_char = b'\t' as c_char;

// ============================================================================
// Lookfor state constants for get_c_indent
// ============================================================================

/// Initial state - looking for something to align with.
pub const LOOKFOR_INITIAL: c_int = 0;

/// Looking for an "if" to match an "else".
pub const LOOKFOR_IF: c_int = 1;

/// Looking for a "do" to match a "while".
pub const LOOKFOR_DO: c_int = 2;

/// Looking for a "case" or "default" label.
pub const LOOKFOR_CASE: c_int = 3;

/// Looking for any statement.
pub const LOOKFOR_ANY: c_int = 4;

/// Looking for a terminated statement.
pub const LOOKFOR_TERM: c_int = 5;

/// Looking for an unterminated statement.
pub const LOOKFOR_UNTERM: c_int = 6;

/// Looking for a scope declaration (public, private, protected).
pub const LOOKFOR_SCOPEDECL: c_int = 7;

/// Looking for a statement without a break.
pub const LOOKFOR_NOBREAK: c_int = 8;

/// Looking for a C++ base class declaration.
pub const LOOKFOR_CPP_BASECLASS: c_int = 9;

/// Looking for an enum or structure initialization.
pub const LOOKFOR_ENUM_OR_INIT: c_int = 10;

/// Looking for a JavaScript key.
pub const LOOKFOR_JS_KEY: c_int = 11;

/// Looking for a comma.
pub const LOOKFOR_COMMA: c_int = 12;

// ============================================================================
// Brace position constants
// ============================================================================

/// '{' is in column 0.
pub const BRACE_IN_COL0: c_int = 1;

/// '{' is at start of line (after whitespace).
pub const BRACE_AT_START: c_int = 2;

/// '{' is at end of line.
pub const BRACE_AT_END: c_int = 3;

// ============================================================================
// Maximum comment length constant
// ============================================================================

/// Maximum length for comment strings from 'comments' option.
pub const COM_MAX_LEN: usize = 50;

// ============================================================================
// Find match direction constants
// ============================================================================

/// Search backwards (used with findmatchlimit).
pub const FM_BACKWARD: c_int = 0x01;

/// Stop at start of block (used with findmatchlimit).
pub const FM_BLOCKSTOP: c_int = 0x02;

// ============================================================================
// MAXCOL constant
// ============================================================================

/// Maximum column value.
pub const MAXCOL: c_int = 0x7fff_ffff;

/// Maximum line number value.
pub const MAXLNUM: i64 = 0x7fff_ffff;

// ============================================================================
// C accessor function declarations
// ============================================================================

extern "C" {
    // Global state accessors
    fn nvim_get_p_paste() -> c_int;
    fn nvim_curbuf_get_p_cin() -> c_int;
    fn nvim_curbuf_get_inde_nonempty() -> c_int;
    fn nvim_curbuf_get_p_si() -> c_int;

    // Buffer cindent option accessors (from indent_c.c or buffer.c)
    fn nvim_curbuf_get_ind_hash_comment() -> c_int;

    // String functions
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn skiptowhite(p: *const c_char) -> *mut c_char;
    fn vim_strchr(s: *const c_char, c: c_int) -> *const c_char;

    // Character classification
    fn vim_iswordc(c: c_int) -> c_int;
    fn vim_isIDc(c: c_int) -> c_int;
    fn ascii_iswhite(c: c_int) -> c_int;
    fn ascii_isdigit(c: c_int) -> c_int;
}

// ============================================================================
// Helper functions
// ============================================================================

/// Check if a character is a NUL (end of string).
#[inline]
const fn is_nul(c: c_char) -> bool {
    c == NUL
}

/// Check if a character is whitespace (space or tab).
#[inline]
const fn is_whitespace(c: c_char) -> bool {
    c == SPACE || c == TAB
}

// ============================================================================
// String skip utilities (safe wrappers)
// ============================================================================

/// Skip over whitespace characters.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[inline]
unsafe fn skip_whitespace(p: *const c_char) -> *const c_char {
    skipwhite(p)
}

/// Skip to the next whitespace character.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[inline]
unsafe fn skip_to_whitespace(p: *const c_char) -> *const c_char {
    skiptowhite(p)
}

// ============================================================================
// Comment and string detection
// ============================================================================

/// Check if position starts with a C or C++ comment.
///
/// Returns true if `p` points to "/\*" or "//".
///
/// # Safety
/// The pointer must point to a valid null-terminated C string with at least 2 bytes.
#[no_mangle]
pub const unsafe extern "C" fn rs_cin_iscomment(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    let c0 = *p;
    if c0 != b'/' as c_char {
        return false;
    }
    let c1 = *p.add(1);
    c1 == b'*' as c_char || c1 == b'/' as c_char
}

/// Check if position starts with a "//" line comment.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string with at least 2 bytes.
#[no_mangle]
pub const unsafe extern "C" fn rs_cin_islinecomment(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'/' as c_char && *p.add(1) == b'/' as c_char
}

/// Check if position starts with a preprocessor directive ('#' after whitespace).
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_ispreproc(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }
    let p = skip_whitespace(s);
    *p == b'#' as c_char
}

// ============================================================================
// Comment skipping
// ============================================================================

/// Skip over white space and C comments within the line.
/// Also skip over Perl/shell comments if desired (based on `b_ind_hash_comment`).
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_skipcomment(s: *const c_char) -> *const c_char {
    if s.is_null() {
        return s;
    }

    let hash_comment = nvim_curbuf_get_ind_hash_comment();
    let mut p = s;

    while !is_nul(*p) {
        let prev_p = p;

        // Skip whitespace
        p = skip_whitespace(p);

        // Perl/shell # comment continues until eol. Require a space
        // before # to avoid recognizing $#array.
        if hash_comment != 0 && p != prev_p && *p == b'#' as c_char {
            // Skip to end of line
            while !is_nul(*p) {
                p = p.add(1);
            }
            break;
        }

        if *p != b'/' as c_char {
            break;
        }
        p = p.add(1);

        if *p == b'/' as c_char {
            // slash-slash comment continues till eol
            while !is_nul(*p) {
                p = p.add(1);
            }
            break;
        }

        if *p != b'*' as c_char {
            break;
        }

        // Skip slash-star comment
        p = p.add(1);
        while !is_nul(*p) {
            if *p == b'*' as c_char && *p.add(1) == b'/' as c_char {
                p = p.add(2);
                break;
            }
            p = p.add(1);
        }
    }

    p
}

/// Return true if there is no code at *s. White space and comments are
/// not considered code.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_nocode(s: *const c_char) -> bool {
    if s.is_null() {
        return true;
    }
    is_nul(*rs_cin_skipcomment(s))
}

// ============================================================================
// String skipping
// ============================================================================

/// Skip to the end of a "string" and a 'c' character.
/// If there is no string or character, return argument unmodified.
///
/// Handles:
/// - Single-quoted characters: 'c', '\n', '\000'
/// - Double-quoted strings: "string"
/// - Raw strings (C++11): R"delim(...)delim"
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_string(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return p;
    }

    let mut ptr = p;

    // We loop, because strings may be concatenated: "date""time".
    loop {
        let c = *ptr;

        if c == b'\'' as c_char {
            // 'c' or '\n' or '\000'
            if is_nul(*ptr.add(1)) {
                // ' at end of line
                break;
            }
            let mut i: usize = 2;
            if *ptr.add(1) == b'\\' as c_char && !is_nul(*ptr.add(2)) {
                // '\n' or '\000'
                i += 1;
                while ascii_isdigit(i32::from(*ptr.add(i - 1) as u8)) != 0 {
                    // '\000'
                    i += 1;
                }
            }
            if !is_nul(*ptr.add(i - 1)) && *ptr.add(i) == b'\'' as c_char {
                // check for trailing '
                ptr = ptr.add(i);
                ptr = ptr.add(1);
                continue;
            }
        } else if c == b'"' as c_char {
            // start of string
            ptr = ptr.add(1);
            while !is_nul(*ptr) {
                if *ptr == b'\\' as c_char && !is_nul(*ptr.add(1)) {
                    ptr = ptr.add(1);
                } else if *ptr == b'"' as c_char {
                    // end of string
                    break;
                }
                ptr = ptr.add(1);
            }
            if *ptr == b'"' as c_char {
                ptr = ptr.add(1);
                continue; // continue for another string
            }
        } else if c == b'R' as c_char && *ptr.add(1) == b'"' as c_char {
            // Raw string: R"[delim](...)[delim]"
            let delim = ptr.add(2);

            // Find the opening paren
            let mut paren = delim;
            while !is_nul(*paren) && *paren != b'(' as c_char {
                paren = paren.add(1);
            }

            if !is_nul(*paren) {
                let delim_len = paren.offset_from(delim) as usize;

                ptr = ptr.add(3); // Skip R"(
                while !is_nul(*ptr) {
                    if *ptr == b')' as c_char {
                        // Check if delimiter matches
                        let mut matches = true;
                        for j in 0..delim_len {
                            if *ptr.add(1 + j) != *delim.add(j) {
                                matches = false;
                                break;
                            }
                        }
                        if matches && *ptr.add(delim_len + 1) == b'"' as c_char {
                            ptr = ptr.add(delim_len + 1);
                            break;
                        }
                    }
                    ptr = ptr.add(1);
                }
                if *ptr == b'"' as c_char {
                    ptr = ptr.add(1);
                    continue; // continue for another string
                }
            }
        }

        break; // no string found
    }

    if is_nul(*ptr) && ptr != p {
        ptr = ptr.sub(1); // backup from NUL
    }

    ptr
}

/// Check if "line[col]" is inside a C string.
///
/// # Safety
/// - `line` must point to a valid null-terminated C string.
/// - `col` must be a valid column within the line.
#[no_mangle]
pub unsafe extern "C" fn rs_is_pos_in_string(line: *const c_char, col: c_int) -> bool {
    if line.is_null() {
        return false;
    }

    let mut p = line;
    while !is_nul(*p) && (p.offset_from(line) as c_int) < col {
        p = rs_skip_string(p);
        if !is_nul(*p) {
            p = p.add(1);
        }
    }
    (p.offset_from(line) as c_int) > col
}

// ============================================================================
// Keyword detection helpers
// ============================================================================

/// Check if string "s" starts with "word" and then a non-ID character.
///
/// # Safety
/// Both pointers must point to valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_starts_with(s: *const c_char, word: *const c_char) -> bool {
    if s.is_null() || word.is_null() {
        return false;
    }

    // Calculate word length
    let mut len: usize = 0;
    while !is_nul(*word.add(len)) {
        len += 1;
    }

    // Compare
    for i in 0..len {
        if *s.add(i) != *word.add(i) {
            return false;
        }
    }

    // Check that the next character is not an ID character
    vim_isIDc(i32::from(*s.add(len) as u8)) == 0
}

/// Check if the string starts with "if" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isif(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'i' as c_char
        && *p.add(1) == b'f' as c_char
        && vim_isIDc(i32::from(*p.add(2) as u8)) == 0
}

/// Check if the string starts with "else" keyword.
/// Also accepts "} else".
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_iselse(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    let mut ptr = p;
    if *ptr == b'}' as c_char {
        // accept "} else"
        ptr = rs_cin_skipcomment(ptr.add(1));
    }
    *ptr == b'e' as c_char
        && *ptr.add(1) == b'l' as c_char
        && *ptr.add(2) == b's' as c_char
        && *ptr.add(3) == b'e' as c_char
        && vim_isIDc(i32::from(*ptr.add(4) as u8)) == 0
}

/// Check if the string starts with "do" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isdo(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'd' as c_char
        && *p.add(1) == b'o' as c_char
        && vim_isIDc(i32::from(*p.add(2) as u8)) == 0
}

/// Check if the string starts with "break" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isbreak(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'b' as c_char
        && *p.add(1) == b'r' as c_char
        && *p.add(2) == b'e' as c_char
        && *p.add(3) == b'a' as c_char
        && *p.add(4) == b'k' as c_char
        && vim_isIDc(i32::from(*p.add(5) as u8)) == 0
}

/// Check if the string starts with "default:" label.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isdefault(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }
    // Check for "default"
    if *s != b'd' as c_char
        || *s.add(1) != b'e' as c_char
        || *s.add(2) != b'f' as c_char
        || *s.add(3) != b'a' as c_char
        || *s.add(4) != b'u' as c_char
        || *s.add(5) != b'l' as c_char
        || *s.add(6) != b't' as c_char
    {
        return false;
    }

    let after = rs_cin_skipcomment(s.add(7));
    *after == b':' as c_char && *after.add(1) != b':' as c_char
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_is_nul() {
        assert!(is_nul(0));
        assert!(!is_nul(b'a' as c_char));
    }

    #[test]
    fn test_is_whitespace() {
        assert!(is_whitespace(b' ' as c_char));
        assert!(is_whitespace(b'\t' as c_char));
        assert!(!is_whitespace(b'a' as c_char));
        assert!(!is_whitespace(0));
    }

    #[test]
    fn test_cin_iscomment() {
        unsafe {
            let slash_star = CString::new("/* comment */").unwrap();
            assert!(rs_cin_iscomment(slash_star.as_ptr()));

            let slash_slash = CString::new("// comment").unwrap();
            assert!(rs_cin_iscomment(slash_slash.as_ptr()));

            let not_comment = CString::new("not a comment").unwrap();
            assert!(!rs_cin_iscomment(not_comment.as_ptr()));

            let single_slash = CString::new("/not").unwrap();
            assert!(!rs_cin_iscomment(single_slash.as_ptr()));

            assert!(!rs_cin_iscomment(std::ptr::null()));
        }
    }

    #[test]
    fn test_cin_islinecomment() {
        unsafe {
            let line_comment = CString::new("// comment").unwrap();
            assert!(rs_cin_islinecomment(line_comment.as_ptr()));

            let block_comment = CString::new("/* comment */").unwrap();
            assert!(!rs_cin_islinecomment(block_comment.as_ptr()));

            let not_comment = CString::new("not a comment").unwrap();
            assert!(!rs_cin_islinecomment(not_comment.as_ptr()));
        }
    }

    #[test]
    fn test_cin_ispreproc() {
        unsafe {
            let preproc = CString::new("#define FOO").unwrap();
            assert!(rs_cin_ispreproc(preproc.as_ptr()));

            let preproc_with_space = CString::new("  #include").unwrap();
            assert!(rs_cin_ispreproc(preproc_with_space.as_ptr()));

            let not_preproc = CString::new("int x;").unwrap();
            assert!(!rs_cin_ispreproc(not_preproc.as_ptr()));
        }
    }

    #[test]
    fn test_cin_isif() {
        unsafe {
            let if_stmt = CString::new("if (x)").unwrap();
            assert!(rs_cin_isif(if_stmt.as_ptr()));

            let ifdef = CString::new("ifdef").unwrap();
            assert!(!rs_cin_isif(ifdef.as_ptr()));

            let ifs = CString::new("ifs").unwrap();
            assert!(!rs_cin_isif(ifs.as_ptr()));
        }
    }

    #[test]
    fn test_cin_iselse() {
        unsafe {
            let else_stmt = CString::new("else").unwrap();
            assert!(rs_cin_iselse(else_stmt.as_ptr()));

            let brace_else = CString::new("} else").unwrap();
            assert!(rs_cin_iselse(brace_else.as_ptr()));

            let elsewhere = CString::new("elsewhere").unwrap();
            assert!(!rs_cin_iselse(elsewhere.as_ptr()));
        }
    }

    #[test]
    fn test_cin_isdo() {
        unsafe {
            let do_stmt = CString::new("do {").unwrap();
            assert!(rs_cin_isdo(do_stmt.as_ptr()));

            let double_stmt = CString::new("double x").unwrap();
            assert!(!rs_cin_isdo(double_stmt.as_ptr()));
        }
    }

    #[test]
    fn test_cin_isbreak() {
        unsafe {
            let break_stmt = CString::new("break;").unwrap();
            assert!(rs_cin_isbreak(break_stmt.as_ptr()));

            let breakfast = CString::new("breakfast").unwrap();
            assert!(!rs_cin_isbreak(breakfast.as_ptr()));
        }
    }

    #[test]
    fn test_cin_isdefault() {
        unsafe {
            let default_label = CString::new("default:").unwrap();
            assert!(rs_cin_isdefault(default_label.as_ptr()));

            // "default::" is C++ scope, not a label
            let default_scope = CString::new("default::foo").unwrap();
            assert!(!rs_cin_isdefault(default_scope.as_ptr()));

            let defaults = CString::new("defaults").unwrap();
            assert!(!rs_cin_isdefault(defaults.as_ptr()));
        }
    }

    #[test]
    fn test_cin_starts_with() {
        unsafe {
            let line = CString::new("while (x)").unwrap();
            let word = CString::new("while").unwrap();
            assert!(rs_cin_starts_with(line.as_ptr(), word.as_ptr()));

            let line2 = CString::new("whilex").unwrap();
            assert!(!rs_cin_starts_with(line2.as_ptr(), word.as_ptr()));

            let line3 = CString::new("for (i)").unwrap();
            assert!(!rs_cin_starts_with(line3.as_ptr(), word.as_ptr()));
        }
    }

    #[test]
    fn test_constants() {
        assert_eq!(LOOKFOR_INITIAL, 0);
        assert_eq!(LOOKFOR_IF, 1);
        assert_eq!(LOOKFOR_DO, 2);
        assert_eq!(LOOKFOR_CASE, 3);

        assert_eq!(BRACE_IN_COL0, 1);
        assert_eq!(BRACE_AT_START, 2);
        assert_eq!(BRACE_AT_END, 3);

        assert_eq!(FM_BACKWARD, 0x01);
        assert_eq!(FM_BLOCKSTOP, 0x02);
    }
}
