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

    // Phase 2 accessors
    fn nvim_cindent_curwin_get_cursor_lnum() -> c_int;
    fn nvim_cindent_curbuf_get_ind_maxparen() -> c_int;
    fn nvim_cindent_curbuf_get_cinw() -> *const c_char;
    fn nvim_cindent_ml_get(lnum: c_int) -> *const c_char;
    fn nvim_cindent_get_indent_lnum(lnum: c_int) -> c_int;

    // Multi-byte character width
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    // Option parsing
    fn copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    ) -> usize;
}

/// Check if a character is whitespace (space or tab).
#[inline]
const fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Check if a character is a digit ('0'-'9').
#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

// ============================================================================
// Helper functions
// ============================================================================

/// Check if a character is a NUL (end of string).
#[inline]
const fn is_nul(c: c_char) -> bool {
    c == NUL
}

/// Return the length of a null-terminated C string (equivalent to strlen).
///
/// # Safety
/// `s` must be a valid null-terminated C string pointer.
#[inline]
#[allow(clippy::missing_const_for_fn)]
unsafe fn c_strlen(s: *const c_char) -> usize {
    std::ffi::CStr::from_ptr(s).to_bytes().len()
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
                while ascii_isdigit(*ptr.add(i - 1) as u8) {
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
// Case and switch label detection
// ============================================================================

/// Recognize a switch label: "case .*:" or "default:".
///
/// # Arguments
/// * `s` - The string to check
/// * `strict` - If true, stop at strings (for C/C++); if false, allow strings (for JS)
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_iscase(s: *const c_char, strict: bool) -> bool {
    if s.is_null() {
        return false;
    }

    let mut p = rs_cin_skipcomment(s);

    // Check for "case"
    if rs_cin_starts_with(p, c"case".as_ptr()) {
        p = p.add(4);
        while !is_nul(*p) {
            p = rs_cin_skipcomment(p);
            if is_nul(*p) {
                break;
            }
            if *p == b':' as c_char {
                if *p.add(1) == b':' as c_char {
                    // skip over "::" for C++
                    p = p.add(1);
                } else {
                    return true;
                }
            }
            if *p == b'\'' as c_char && !is_nul(*p.add(1)) && *p.add(2) == b'\'' as c_char {
                // skip over ':'
                p = p.add(2);
            } else if *p == b'/' as c_char
                && (*p.add(1) == b'*' as c_char || *p.add(1) == b'/' as c_char)
            {
                // stop at comment
                return false;
            } else if *p == b'"' as c_char {
                // JS etc.
                if strict {
                    return false; // stop at string
                }
                return true;
            }
            if !is_nul(*p) {
                p = p.add(1);
            }
        }
        return false;
    }

    // Check for "default:"
    rs_cin_isdefault(p)
}

// ============================================================================
// JavaScript key detection
// ============================================================================

/// Checks if `text` starts with "key:".
///
/// Used for JavaScript object property detection.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_has_js_key(text: *const c_char) -> bool {
    if text.is_null() {
        return false;
    }

    let mut s = skip_whitespace(text);

    let mut quote: c_char = 0;
    if *s == b'\'' as c_char || *s == b'"' as c_char {
        // can be 'key': or "key":
        quote = *s;
        s = s.add(1);
    }

    // need at least one ID character
    if vim_isIDc(i32::from(*s as u8)) == 0 {
        return false;
    }

    while vim_isIDc(i32::from(*s as u8)) != 0 {
        s = s.add(1);
    }

    if !is_nul(*s) && *s == quote {
        s = s.add(1);
    }

    s = rs_cin_skipcomment(s);

    // "::" is not a label, it's C++
    *s == b':' as c_char && *s.add(1) != b':' as c_char
}

// ============================================================================
// C++ namespace detection
// ============================================================================

/// Recognize a "namespace" scope declaration.
///
/// Handles:
/// - `namespace foo {`
/// - `inline namespace foo {`
/// - `export namespace foo {`
/// - `namespace foo::bar {` (C++17 nested namespaces)
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_cpp_namespace(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let mut p = rs_cin_skipcomment(s);

    // skip over "inline" and "export" in any order
    loop {
        let is_inline = *p == b'i' as c_char
            && *p.add(1) == b'n' as c_char
            && *p.add(2) == b'l' as c_char
            && *p.add(3) == b'i' as c_char
            && *p.add(4) == b'n' as c_char
            && *p.add(5) == b'e' as c_char
            && (is_nul(*p.add(6)) || vim_iswordc(i32::from(*p.add(6) as u8)) == 0);

        let is_export = *p == b'e' as c_char
            && *p.add(1) == b'x' as c_char
            && *p.add(2) == b'p' as c_char
            && *p.add(3) == b'o' as c_char
            && *p.add(4) == b'r' as c_char
            && *p.add(5) == b't' as c_char
            && (is_nul(*p.add(6)) || vim_iswordc(i32::from(*p.add(6) as u8)) == 0);

        if is_inline || is_export {
            p = rs_cin_skipcomment(skip_whitespace(p.add(6)));
        } else {
            break;
        }
    }

    // Check for "namespace"
    if *p != b'n' as c_char
        || *p.add(1) != b'a' as c_char
        || *p.add(2) != b'm' as c_char
        || *p.add(3) != b'e' as c_char
        || *p.add(4) != b's' as c_char
        || *p.add(5) != b'p' as c_char
        || *p.add(6) != b'a' as c_char
        || *p.add(7) != b'c' as c_char
        || *p.add(8) != b'e' as c_char
    {
        return false;
    }
    if !is_nul(*p.add(9)) && vim_iswordc(i32::from(*p.add(9) as u8)) != 0 {
        return false;
    }

    p = rs_cin_skipcomment(skip_whitespace(p.add(9)));

    let mut has_name = false;
    let mut has_name_start = false;

    while !is_nul(*p) {
        if ascii_iswhite(*p as u8) {
            has_name = true; // found end of a name
            p = rs_cin_skipcomment(skip_whitespace(p));
        } else if *p == b'{' as c_char {
            break;
        } else if vim_iswordc(i32::from(*p as u8)) != 0 {
            has_name_start = true;
            if has_name {
                return false; // word character after skipping past name
            }
            p = p.add(1);
        } else if *p == b':' as c_char
            && *p.add(1) == b':' as c_char
            && vim_iswordc(i32::from(*p.add(2) as u8)) != 0
        {
            if !has_name_start || has_name {
                return false;
            }
            // C++ 17 nested namespace
            p = p.add(3);
        } else {
            return false;
        }
    }

    true
}

// ============================================================================
// String ending detection
// ============================================================================

/// Return true if string "s" ends with the string "find", possibly followed by
/// white space and comments. Skip strings and comments while searching.
///
/// # Safety
/// Both pointers must point to valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_ends_in(s: *const c_char, find: *const c_char) -> bool {
    if s.is_null() || find.is_null() {
        return false;
    }

    // Calculate find length
    let mut find_len: usize = 0;
    while !is_nul(*find.add(find_len)) {
        find_len += 1;
    }

    let mut p = s;

    while !is_nul(*p) {
        p = rs_cin_skipcomment(p);

        // Check if we found the string
        let mut matches = true;
        for i in 0..find_len {
            if *p.add(i) != *find.add(i) {
                matches = false;
                break;
            }
        }

        if matches {
            let r = skip_whitespace(p.add(find_len));
            if rs_cin_nocode(r) {
                return true;
            }
        }

        if !is_nul(*p) {
            p = p.add(1);
        }
    }

    false
}

/// Skip over strings, comments, and concatenated strings/comments.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_skip_comment_and_string(s: *const c_char) -> *const c_char {
    if s.is_null() {
        return s;
    }

    let mut r: *const c_char;
    let mut p = s;

    loop {
        r = p;
        p = rs_cin_skipcomment(p);
        if !is_nul(*p) {
            p = rs_skip_string(p);
        }
        if p == r {
            break;
        }
    }

    p
}

// ============================================================================
// Bracket and brace matching utilities
// ============================================================================

/// Result of finding an unmatched bracket in a line.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BracketMatch {
    /// Whether an unmatched bracket was found.
    pub found: bool,
    /// Column (0-based) of the last unmatched bracket, if found.
    pub col: c_int,
}

/// Find the position of the last unmatched closing bracket in a line.
///
/// Searches for the last unmatched ')' or '}' (depending on `start` and `end`).
/// Ignores brackets in comments and strings.
///
/// # Arguments
/// * `line` - The line to search
/// * `start` - The opening bracket character (e.g., '(' or '{')
/// * `end` - The closing bracket character (e.g., ')' or '}')
///
/// # Returns
/// A `BracketMatch` with `found=true` and the column if an unmatched bracket
/// was found, otherwise `found=false`.
///
/// # Safety
/// The `line` pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_find_last_paren(
    line: *const c_char,
    start: c_char,
    end: c_char,
) -> BracketMatch {
    if line.is_null() {
        return BracketMatch {
            found: false,
            col: 0,
        };
    }

    let mut col: c_int = 0;
    let mut found = false;
    let mut open_count: c_int = 0;
    let mut i: c_int = 0;

    while !is_nul(*line.add(i as usize)) {
        // Skip comments
        let after_comment = rs_cin_skipcomment(line.add(i as usize));
        i = after_comment.offset_from(line) as c_int;

        // Skip strings
        let after_string = rs_skip_string(line.add(i as usize));
        i = after_string.offset_from(line) as c_int;

        if is_nul(*line.add(i as usize)) {
            break;
        }

        let c = *line.add(i as usize);
        if c == start {
            open_count += 1;
        } else if c == end {
            if open_count > 0 {
                open_count -= 1;
            } else {
                col = i;
                found = true;
            }
        }

        i += 1;
    }

    BracketMatch { found, col }
}

/// Count unmatched opening brackets in a line up to a given column.
///
/// Returns the nesting level (number of unmatched opening brackets).
/// Ignores brackets in comments and strings.
///
/// # Arguments
/// * `line` - The line to search
/// * `start` - The opening bracket character (e.g., '(' or '{')
/// * `end` - The closing bracket character (e.g., ')' or '}')
/// * `max_col` - Maximum column to search (exclusive), or -1 for entire line
///
/// # Safety
/// The `line` pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_count_unmatched_open(
    line: *const c_char,
    start: c_char,
    end: c_char,
    max_col: c_int,
) -> c_int {
    if line.is_null() {
        return 0;
    }

    let mut count: c_int = 0;
    let mut i: c_int = 0;

    while !is_nul(*line.add(i as usize)) {
        if max_col >= 0 && i >= max_col {
            break;
        }

        // Skip comments
        let after_comment = rs_cin_skipcomment(line.add(i as usize));
        let new_i = after_comment.offset_from(line) as c_int;
        if new_i != i {
            i = new_i;
            continue;
        }

        // Skip strings
        let after_string = rs_skip_string(line.add(i as usize));
        let new_i = after_string.offset_from(line) as c_int;
        if new_i != i {
            i = new_i;
            continue;
        }

        if is_nul(*line.add(i as usize)) {
            break;
        }

        let c = *line.add(i as usize);
        if c == start {
            count += 1;
        } else if c == end && count > 0 {
            count -= 1;
        }

        i += 1;
    }

    count
}

/// Check if a position is inside parentheses/brackets.
///
/// Returns true if position `col` is inside unmatched parentheses.
///
/// # Arguments
/// * `line` - The line to check
/// * `col` - The column position to check
/// * `start` - The opening bracket character (e.g., '(' or '{')
/// * `end` - The closing bracket character (e.g., ')' or '}')
///
/// # Safety
/// The `line` pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_is_inside_brackets(
    line: *const c_char,
    col: c_int,
    start: c_char,
    end: c_char,
) -> bool {
    if line.is_null() {
        return false;
    }

    let count = rs_count_unmatched_open(line, start, end, col);
    count > 0
}

/// Skip over the contents of a C string at position `col` in `line`.
///
/// If position is at a quote character, returns the position after the
/// closing quote. Otherwise returns the original column.
///
/// # Safety
/// The `line` pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_skip2pos_col(line: *const c_char, col: c_int) -> c_int {
    if line.is_null() || col < 0 {
        return col;
    }

    let mut p = line;
    while !is_nul(*p) && (p.offset_from(line) as c_int) < col {
        if rs_cin_iscomment(p) {
            p = rs_cin_skipcomment(p);
        } else {
            let new_p = rs_skip_string(p);
            if new_p == p {
                p = p.add(1);
            } else {
                p = new_p;
            }
        }
    }

    p.offset_from(line) as c_int
}

/// Check if the character at position is in a comment (not a string).
///
/// # Safety
/// The `line` pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_iscomment_pos(line: *const c_char, col: c_int) -> bool {
    if line.is_null() || col < 0 {
        return false;
    }

    let skipped_col = rs_cin_skip2pos_col(line, col);
    skipped_col > col
}

// ============================================================================
// Statement analysis utilities
// ============================================================================

/// Check if a line is terminated with a statement terminator.
///
/// Recognizes lines that start with '{' or '}', or end with ';', ',', '{' or '}'.
/// Does not consider "} else" a terminated line.
///
/// # Arguments
/// * `s` - The line to check
/// * `incl_open` - Include '{' at the end as terminator
/// * `incl_comma` - Recognize a trailing comma
///
/// # Returns
/// The terminating character, or '\0' if not terminated.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isterminated(
    s: *const c_char,
    incl_open: bool,
    incl_comma: bool,
) -> c_char {
    if s.is_null() {
        return NUL;
    }

    let mut found_start: c_char = 0;
    let mut n_open: c_int = 0;
    let mut is_else = false;

    let mut p = rs_cin_skipcomment(s);

    // Check for '{' or '}' at start (but not "} else")
    if *p == b'{' as c_char || (*p == b'}' as c_char && !rs_cin_iselse(p)) {
        found_start = *p;
    }

    if found_start == 0 {
        is_else = rs_cin_iselse(p);
    }

    while !is_nul(*p) {
        // Skip over comments, strings and characters
        p = rs_skip_string(rs_cin_skipcomment(p));

        if is_nul(*p) {
            break;
        }

        if *p == b'}' as c_char && n_open > 0 {
            n_open -= 1;
        }

        if (!is_else || n_open == 0)
            && (*p == b';' as c_char
                || *p == b'}' as c_char
                || (incl_comma && *p == b',' as c_char))
            && rs_cin_nocode(p.add(1))
        {
            return *p;
        } else if *p == b'{' as c_char {
            if incl_open && rs_cin_nocode(p.add(1)) {
                return *p;
            }
            n_open += 1;
        }

        if !is_nul(*p) {
            p = p.add(1);
        }
    }

    found_start
}

/// Check if a line is terminated.
///
/// Convenience wrapper that returns true if the line ends with a statement
/// terminator.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_terminated(s: *const c_char) -> bool {
    rs_cin_isterminated(s, false, false) != NUL
}

/// Check if line starts with "while" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_iswhile(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'w' as c_char
        && *p.add(1) == b'h' as c_char
        && *p.add(2) == b'i' as c_char
        && *p.add(3) == b'l' as c_char
        && *p.add(4) == b'e' as c_char
        && vim_isIDc(i32::from(*p.add(5) as u8)) == 0
}

/// Check if line starts with "for" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isfor(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'f' as c_char
        && *p.add(1) == b'o' as c_char
        && *p.add(2) == b'r' as c_char
        && vim_isIDc(i32::from(*p.add(3) as u8)) == 0
}

/// Check if line starts with "return" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isreturn(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'r' as c_char
        && *p.add(1) == b'e' as c_char
        && *p.add(2) == b't' as c_char
        && *p.add(3) == b'u' as c_char
        && *p.add(4) == b'r' as c_char
        && *p.add(5) == b'n' as c_char
        && vim_isIDc(i32::from(*p.add(6) as u8)) == 0
}

/// Check if line starts with "continue" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_iscontinue(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'c' as c_char
        && *p.add(1) == b'o' as c_char
        && *p.add(2) == b'n' as c_char
        && *p.add(3) == b't' as c_char
        && *p.add(4) == b'i' as c_char
        && *p.add(5) == b'n' as c_char
        && *p.add(6) == b'u' as c_char
        && *p.add(7) == b'e' as c_char
        && vim_isIDc(i32::from(*p.add(8) as u8)) == 0
}

/// Check if line starts with "goto" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isgoto(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'g' as c_char
        && *p.add(1) == b'o' as c_char
        && *p.add(2) == b't' as c_char
        && *p.add(3) == b'o' as c_char
        && vim_isIDc(i32::from(*p.add(4) as u8)) == 0
}

/// Check if line starts with "switch" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isswitch(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b's' as c_char
        && *p.add(1) == b'w' as c_char
        && *p.add(2) == b'i' as c_char
        && *p.add(3) == b't' as c_char
        && *p.add(4) == b'c' as c_char
        && *p.add(5) == b'h' as c_char
        && vim_isIDc(i32::from(*p.add(6) as u8)) == 0
}

/// Find an '=' character in the line, skipping strings and comments.
///
/// Returns the column of the '=' or -1 if not found.
/// Stops at ';', '{', '}', single/double quotes.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_find_equal(s: *const c_char) -> c_int {
    if s.is_null() {
        return -1;
    }

    let mut p = s;
    while !is_nul(*p) {
        // Check for characters that stop the search
        if *p == b';' as c_char
            || *p == b'{' as c_char
            || *p == b'}' as c_char
            || *p == b'\'' as c_char
            || *p == b'"' as c_char
        {
            return -1;
        }

        // Skip comments
        if rs_cin_iscomment(p) {
            p = rs_cin_skipcomment(p);
            continue;
        }

        // Found '='
        if *p == b'=' as c_char {
            return p.offset_from(s) as c_int;
        }

        p = p.add(1);
    }

    -1
}

/// Check if line contains an assignment or return with initialization.
///
/// Returns true if line has '=' or 'return' followed by initializer-like content.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_compound_init(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let mut p = rs_cin_skipcomment(s);
    let mut r: *const c_char = std::ptr::null();

    // Look for '=' or "return"
    while !is_nul(*p) {
        if *p == b'=' as c_char {
            r = rs_cin_skipcomment(p.add(1));
            p = r;
        } else if rs_cin_isreturn(p) {
            r = rs_cin_skipcomment(p.add(6));
            p = r;
        } else {
            p = rs_cin_skip_comment_and_string(p.add(1));
        }

        // If we found '=' or "return", r is now set
        if !r.is_null() {
            break;
        }
    }

    if r.is_null() {
        return false;
    }

    // p now points after '=' or "return"
    if rs_cin_nocode(p) {
        return true;
    }

    // Skip '&' if present
    if *p == b'&' as c_char {
        p = rs_cin_skipcomment(p.add(1));
    }

    // Skip a typecast (...)
    if *p == b'(' as c_char {
        let mut open_count: c_int = 1;
        loop {
            p = rs_cin_skip_comment_and_string(p.add(1));
            if rs_cin_nocode(p) {
                return true;
            }
            if *p == b'(' as c_char {
                open_count += 1;
            } else if *p == b')' as c_char {
                open_count -= 1;
            }
            if open_count == 0 {
                break;
            }
        }
        p = rs_cin_skipcomment(p.add(1));
        if rs_cin_nocode(p) {
            return true;
        }
    }

    // Skip opening braces
    while *p == b'{' as c_char {
        p = rs_cin_skipcomment(p.add(1));
    }

    rs_cin_nocode(p)
}

/// Check if line starts with "typedef" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_istypedef(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b't' as c_char
        && *p.add(1) == b'y' as c_char
        && *p.add(2) == b'p' as c_char
        && *p.add(3) == b'e' as c_char
        && *p.add(4) == b'd' as c_char
        && *p.add(5) == b'e' as c_char
        && *p.add(6) == b'f' as c_char
        && vim_isIDc(i32::from(*p.add(7) as u8)) == 0
}

/// Check if line starts with "static" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isstatic(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b's' as c_char
        && *p.add(1) == b't' as c_char
        && *p.add(2) == b'a' as c_char
        && *p.add(3) == b't' as c_char
        && *p.add(4) == b'i' as c_char
        && *p.add(5) == b'c' as c_char
        && vim_isIDc(i32::from(*p.add(6) as u8)) == 0
}

/// Check if line starts with "public" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_ispublic(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'p' as c_char
        && *p.add(1) == b'u' as c_char
        && *p.add(2) == b'b' as c_char
        && *p.add(3) == b'l' as c_char
        && *p.add(4) == b'i' as c_char
        && *p.add(5) == b'c' as c_char
        && vim_isIDc(i32::from(*p.add(6) as u8)) == 0
}

/// Check if line starts with "protected" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isprotected(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'p' as c_char
        && *p.add(1) == b'r' as c_char
        && *p.add(2) == b'o' as c_char
        && *p.add(3) == b't' as c_char
        && *p.add(4) == b'e' as c_char
        && *p.add(5) == b'c' as c_char
        && *p.add(6) == b't' as c_char
        && *p.add(7) == b'e' as c_char
        && *p.add(8) == b'd' as c_char
        && vim_isIDc(i32::from(*p.add(9) as u8)) == 0
}

/// Check if line starts with "private" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isprivate(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'p' as c_char
        && *p.add(1) == b'r' as c_char
        && *p.add(2) == b'i' as c_char
        && *p.add(3) == b'v' as c_char
        && *p.add(4) == b'a' as c_char
        && *p.add(5) == b't' as c_char
        && *p.add(6) == b'e' as c_char
        && vim_isIDc(i32::from(*p.add(7) as u8)) == 0
}

/// Check if line starts with "enum" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isenum(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'e' as c_char
        && *p.add(1) == b'n' as c_char
        && *p.add(2) == b'u' as c_char
        && *p.add(3) == b'm' as c_char
        && vim_isIDc(i32::from(*p.add(4) as u8)) == 0
}

/// Check if line starts with "struct" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isstruct(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b's' as c_char
        && *p.add(1) == b't' as c_char
        && *p.add(2) == b'r' as c_char
        && *p.add(3) == b'u' as c_char
        && *p.add(4) == b'c' as c_char
        && *p.add(5) == b't' as c_char
        && vim_isIDc(i32::from(*p.add(6) as u8)) == 0
}

/// Check if line starts with "class" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isclass(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'c' as c_char
        && *p.add(1) == b'l' as c_char
        && *p.add(2) == b'a' as c_char
        && *p.add(3) == b's' as c_char
        && *p.add(4) == b's' as c_char
        && vim_isIDc(i32::from(*p.add(5) as u8)) == 0
}

/// Check if line starts with "union" keyword.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isunion(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    *p == b'u' as c_char
        && *p.add(1) == b'n' as c_char
        && *p.add(2) == b'i' as c_char
        && *p.add(3) == b'o' as c_char
        && *p.add(4) == b'n' as c_char
        && vim_isIDc(i32::from(*p.add(5) as u8)) == 0
}

// ============================================================================
// Indentation calculation helpers
// ============================================================================

/// Cindent options structure.
///
/// Mirrors the `b_ind_*` fields in `buf_T` for cindent settings.
/// Used to pass indentation options from C to Rust.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CindentOptions {
    /// Indent for each block level (default: shiftwidth).
    pub ind_level: c_int,
    /// Extra indent for open brace at end of line.
    pub ind_open_imag: c_int,
    /// Indent for line without opening brace.
    pub ind_no_brace: c_int,
    /// First open brace column (for function definitions).
    pub ind_first_open: c_int,
    /// Extra indent for open brace.
    pub ind_open_extra: c_int,
    /// Extra indent for close brace.
    pub ind_close_extra: c_int,
    /// Imaginary indent for open brace in column 0.
    pub ind_open_left_imag: c_int,
    /// Jump label indent (-1 means column 1).
    pub ind_jump_label: c_int,
    /// Indent for case labels.
    pub ind_case: c_int,
    /// Indent for code after case label.
    pub ind_case_code: c_int,
    /// Break lineup with case.
    pub ind_case_break: c_int,
    /// Scope declaration indent (public:, etc).
    pub ind_scopedecl: c_int,
    /// Code after scope declaration.
    pub ind_scopedecl_code: c_int,
    /// K&R style parameter indent.
    pub ind_param: c_int,
    /// Function type spec indent.
    pub ind_func_type: c_int,
    /// C++ base class indent.
    pub ind_cpp_baseclass: c_int,
    /// Continuation line indent.
    pub ind_continuation: c_int,
    /// Unclosed parenthesis indent.
    pub ind_unclosed: c_int,
    /// Second unclosed parenthesis indent.
    pub ind_unclosed2: c_int,
    /// Don't ignore unclosed paren indent.
    pub ind_unclosed_noignore: c_int,
    /// Wrapped unclosed paren indent.
    pub ind_unclosed_wrapped: c_int,
    /// Allow whitespace in unclosed paren lineup.
    pub ind_unclosed_whiteok: c_int,
    /// Match paren lineup.
    pub ind_matching_paren: c_int,
    /// Previous line paren indent.
    pub ind_paren_prev: c_int,
    /// Extra indent for comments.
    pub ind_comment: c_int,
    /// Inside comment indent.
    pub ind_in_comment: c_int,
    /// Force inside comment indent.
    pub ind_in_comment2: c_int,
    /// Max lines to search for paren.
    pub ind_maxparen: c_int,
    /// Max lines to search for comment.
    pub ind_maxcomment: c_int,
    /// Java brace handling.
    pub ind_java: c_int,
    /// JavaScript mode.
    pub ind_js: c_int,
    /// Keep case label indent.
    pub ind_keep_case_label: c_int,
    /// C++ namespace indent.
    pub ind_cpp_namespace: c_int,
    /// if/for/while continuation indent.
    pub ind_if_for_while: c_int,
    /// Hash comment indent.
    pub ind_hash_comment: c_int,
    /// C++ extern "C" indent.
    pub ind_cpp_extern_c: c_int,
    /// Pragma indent.
    pub ind_pragma: c_int,
}

/// Result of analyzing indentation context.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IndentContext {
    /// Recommended base indent amount.
    pub base_indent: c_int,
    /// Additional indent adjustment.
    pub adjustment: c_int,
    /// Type of construct determining indent.
    pub context_type: c_int,
    /// Line number providing context.
    pub context_lnum: i64,
}

/// Context types for indent decisions.
pub const INDENT_CTX_NONE: c_int = 0;
pub const INDENT_CTX_BLOCK: c_int = 1;
pub const INDENT_CTX_CASE: c_int = 2;
pub const INDENT_CTX_PAREN: c_int = 3;
pub const INDENT_CTX_COMMENT: c_int = 4;
pub const INDENT_CTX_STRING: c_int = 5;
pub const INDENT_CTX_PREPROC: c_int = 6;
pub const INDENT_CTX_CONTINUATION: c_int = 7;

/// Check if a line ends with a backslash (continuation).
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_ends_in_backslash(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    // Find end of string
    let mut p = s;
    let mut last_non_nul = p;
    while !is_nul(*p) {
        if !is_nul(*p) {
            last_non_nul = p;
        }
        p = p.add(1);
    }

    // Check if last character is backslash
    if last_non_nul == s && is_nul(*s) {
        return false;
    }
    *last_non_nul == b'\\' as c_char
}

/// Check if a line is blank (only whitespace or empty).
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_linewhite(s: *const c_char) -> bool {
    if s.is_null() {
        return true;
    }

    let p = skip_whitespace(s);
    is_nul(*p)
}

/// Calculate indent for a label line.
///
/// Returns the appropriate indent based on whether this is a
/// case label, default label, or regular label.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_get_label_indent(
    s: *const c_char,
    opts: *const CindentOptions,
    base_indent: c_int,
) -> c_int {
    if s.is_null() || opts.is_null() {
        return base_indent;
    }

    let opts = &*opts;
    let p = rs_cin_skipcomment(s);

    // Check for case or default
    if rs_cin_iscase(p, false) {
        return base_indent + opts.ind_case;
    }

    // Check for scope declaration (public:, etc)
    // Note: We can't fully implement cin_isscopedecl without access to
    // curbuf->b_p_cinsd, so we check for common ones
    let p_skip = rs_cin_skipcomment(p);
    if rs_cin_ispublic(p_skip) || rs_cin_isprotected(p_skip) || rs_cin_isprivate(p_skip) {
        // Check for trailing colon
        let mut check = p_skip;
        while vim_isIDc(i32::from(*check as u8)) != 0 {
            check = check.add(1);
        }
        check = rs_cin_skipcomment(check);
        if *check == b':' as c_char && *check.add(1) != b':' as c_char {
            return base_indent + opts.ind_scopedecl;
        }
    }

    // Regular label - use jump_label setting
    if opts.ind_jump_label < 0 {
        0 // Column 0
    } else {
        base_indent + opts.ind_jump_label
    }
}

/// Determine if line looks like a function declaration.
///
/// Basic heuristic: has '(' but no ';' before it, and ends with ')' or '){'
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_looks_like_funcdecl(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    // Skip preprocessor
    if rs_cin_ispreproc(s) {
        return false;
    }

    let mut has_paren = false;
    let mut p = s;

    while !is_nul(*p) {
        // Skip comments and strings
        if rs_cin_iscomment(p) {
            p = rs_cin_skipcomment(p);
            continue;
        }

        let new_p = rs_skip_string(p);
        if new_p != p {
            p = new_p;
            continue;
        }

        // Semicolon before paren means not a function decl
        if *p == b';' as c_char && !has_paren {
            return false;
        }

        if *p == b'(' as c_char {
            has_paren = true;
        }

        // Check for ')' followed by optional whitespace/comment and '{' or EOL
        if *p == b')' as c_char {
            let after = rs_cin_skipcomment(p.add(1));
            if is_nul(*after) || *after == b'{' as c_char {
                return has_paren;
            }
        }

        p = p.add(1);
    }

    false
}

/// Check if line appears to be starting a K&R style parameter list.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_kr_param(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let p = rs_cin_skipcomment(s);

    // K&R params are type declarations
    // e.g., "int x;" after function header "foo(x)"

    // Skip any storage class
    let mut check = p;
    if rs_cin_isstatic(check) {
        check = rs_cin_skipcomment(check.add(6));
    }

    // Look for type followed by identifier and semicolon
    // This is a simplified check
    if !vim_isIDc(i32::from(*check as u8)) == 0 {
        return false;
    }

    // Skip type name
    while vim_isIDc(i32::from(*check as u8)) != 0 {
        check = check.add(1);
    }
    check = rs_cin_skipcomment(check);

    // Check for pointer indicator
    while *check == b'*' as c_char {
        check = rs_cin_skipcomment(check.add(1));
    }

    // Skip variable name
    if vim_isIDc(i32::from(*check as u8)) == 0 {
        return false;
    }
    while vim_isIDc(i32::from(*check as u8)) != 0 {
        check = check.add(1);
    }
    check = rs_cin_skipcomment(check);

    // Should end with semicolon
    *check == b';' as c_char
}

/// Calculate the effective indent for unclosed parentheses.
///
/// # Arguments
/// * `base_indent` - The base indent level
/// * `paren_col` - Column of the opening parenthesis
/// * `opts` - Cindent options
/// * `is_first_paren` - True if this is the first unclosed paren
///
/// # Returns
/// The calculated indent amount.
///
/// # Safety
/// The `opts` pointer must be valid or null.
#[no_mangle]
pub const unsafe extern "C" fn rs_calc_paren_indent(
    base_indent: c_int,
    paren_col: c_int,
    opts: *const CindentOptions,
    is_first_paren: bool,
) -> c_int {
    if opts.is_null() {
        return paren_col + 1;
    }

    let opts = &*opts;

    if opts.ind_unclosed == 0 {
        // Line up with the character after the paren
        paren_col + 1
    } else if is_first_paren {
        base_indent + opts.ind_unclosed
    } else {
        base_indent + opts.ind_unclosed2
    }
}

// ============================================================================
// Special cases and edge handling
// ============================================================================

/// Check if line contains a C++ lambda expression start.
///
/// Looks for `[` followed by capture clause patterns like `[=]`, `[&]`, `[this]`.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_cpp_lambda(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let mut p = rs_cin_skipcomment(s);

    // Skip over statements until we find '[' not in a string/comment
    while !is_nul(*p) {
        if rs_cin_iscomment(p) {
            p = rs_cin_skipcomment(p);
            continue;
        }

        let new_p = rs_skip_string(p);
        if new_p != p {
            p = new_p;
            continue;
        }

        // Found potential lambda start
        if *p == b'[' as c_char {
            let next = rs_cin_skipcomment(p.add(1));

            // Check for common capture patterns
            // [=], [&], [this], [=, ...], [&, ...], []
            if *next == b']' as c_char
                || *next == b'=' as c_char
                || *next == b'&' as c_char
                || (*next == b't' as c_char
                    && *next.add(1) == b'h' as c_char
                    && *next.add(2) == b'i' as c_char
                    && *next.add(3) == b's' as c_char)
            {
                return true;
            }

            // Could also be [identifier, ...] capture
            if vim_isIDc(i32::from(*next as u8)) != 0 {
                // Skip identifier
                let mut check = next;
                while vim_isIDc(i32::from(*check as u8)) != 0 {
                    check = check.add(1);
                }
                check = rs_cin_skipcomment(check);
                // If followed by ']' or ',' it's likely a lambda
                if *check == b']' as c_char || *check == b',' as c_char {
                    return true;
                }
            }
        }

        p = p.add(1);
    }

    false
}

/// Check if line contains a C++ template declaration.
///
/// Looks for `template` followed by `<`.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_template_decl(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let p = rs_cin_skipcomment(s);

    // Check for "template"
    if *p != b't' as c_char
        || *p.add(1) != b'e' as c_char
        || *p.add(2) != b'm' as c_char
        || *p.add(3) != b'p' as c_char
        || *p.add(4) != b'l' as c_char
        || *p.add(5) != b'a' as c_char
        || *p.add(6) != b't' as c_char
        || *p.add(7) != b'e' as c_char
    {
        return false;
    }

    if vim_isIDc(i32::from(*p.add(8) as u8)) != 0 {
        return false;
    }

    // Look for '<' after template
    let after = rs_cin_skipcomment(p.add(8));
    *after == b'<' as c_char
}

/// Check if line contains an extern "C" block.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_extern_c(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let p = rs_cin_skipcomment(s);

    // Check for "extern"
    if *p != b'e' as c_char
        || *p.add(1) != b'x' as c_char
        || *p.add(2) != b't' as c_char
        || *p.add(3) != b'e' as c_char
        || *p.add(4) != b'r' as c_char
        || *p.add(5) != b'n' as c_char
    {
        return false;
    }

    if vim_isIDc(i32::from(*p.add(6) as u8)) != 0 {
        return false;
    }

    // Look for "C" or "C++"
    let after = rs_cin_skipcomment(p.add(6));
    if *after == b'"' as c_char {
        let inside = after.add(1);
        // Check for "C" or "C++"
        if *inside == b'C' as c_char {
            let next = inside.add(1);
            // "C" alone or "C++"
            if *next == b'"' as c_char
                || (*next == b'+' as c_char
                    && *next.add(1) == b'+' as c_char
                    && *next.add(2) == b'"' as c_char)
            {
                return true;
            }
        }
    }

    false
}

/// Check if the line starts a multi-line comment.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_starts_multiline_comment(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let p = skip_whitespace(s);

    // Look for /* not followed by */
    if *p == b'/' as c_char && *p.add(1) == b'*' as c_char {
        // Check if it closes on the same line
        let mut check = p.add(2);
        while !is_nul(*check) {
            if *check == b'*' as c_char && *check.add(1) == b'/' as c_char {
                return false; // Closes on same line
            }
            check = check.add(1);
        }
        return true;
    }

    false
}

/// Check if line is inside a multi-line comment (starts with * but not //).
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_inside_multiline_comment(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let p = skip_whitespace(s);

    // Common pattern: line starting with * (continuation of comment)
    // But not if it's "* /" which closes the comment
    if *p == b'*' as c_char {
        let next = p.add(1);
        // If next is '/' then this closes the comment
        if *next == b'/' as c_char {
            return false;
        }
        return true;
    }

    false
}

/// Check if line closes a multi-line comment.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub const unsafe extern "C" fn rs_cin_ends_multiline_comment(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    // Look for */ in the line
    let mut p = s;
    while !is_nul(*p) {
        if *p == b'*' as c_char && *p.add(1) == b'/' as c_char {
            return true;
        }
        p = p.add(1);
    }

    false
}

/// Check if line contains only a closing brace.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_closing_brace_line(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let p = rs_cin_skipcomment(s);

    // Must be just '}' possibly followed by whitespace/comments
    if *p != b'}' as c_char {
        return false;
    }

    rs_cin_nocode(p.add(1))
}

/// Check if line contains only an opening brace.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_opening_brace_line(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let p = rs_cin_skipcomment(s);

    // Must be just '{' possibly followed by whitespace/comments
    if *p != b'{' as c_char {
        return false;
    }

    rs_cin_nocode(p.add(1))
}

/// Check if line is a continuation of a ternary operator.
///
/// Looks for `?` or `:` that might be part of `? :` expression.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_ternary_continuation(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let p = rs_cin_skipcomment(s);

    // Line starting with ':' (not ::) or '?' might be ternary continuation
    if *p == b':' as c_char && *p.add(1) != b':' as c_char {
        return true;
    }

    if *p == b'?' as c_char {
        return true;
    }

    false
}

/// Check if line is a continuation of a boolean expression.
///
/// Looks for lines starting with `&&`, `||`, `and`, `or`.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_bool_continuation(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let p = rs_cin_skipcomment(s);

    // Check for && or ||
    if (*p == b'&' as c_char && *p.add(1) == b'&' as c_char)
        || (*p == b'|' as c_char && *p.add(1) == b'|' as c_char)
    {
        return true;
    }

    // Check for "and" or "or" (C++ alternative operators)
    if *p == b'a' as c_char
        && *p.add(1) == b'n' as c_char
        && *p.add(2) == b'd' as c_char
        && vim_isIDc(i32::from(*p.add(3) as u8)) == 0
    {
        return true;
    }

    if *p == b'o' as c_char
        && *p.add(1) == b'r' as c_char
        && vim_isIDc(i32::from(*p.add(2) as u8)) == 0
    {
        return true;
    }

    false
}

/// Determine brace position type.
///
/// # Arguments
/// * `line` - The line containing the brace
/// * `brace_col` - Column of the brace
///
/// # Returns
/// One of `BRACE_IN_COL0`, `BRACE_AT_START`, or `BRACE_AT_END`.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_brace_position(line: *const c_char, brace_col: c_int) -> c_int {
    if line.is_null() || brace_col < 0 {
        return BRACE_AT_END;
    }

    // Check if at column 0
    if brace_col == 0 {
        return BRACE_IN_COL0;
    }

    // Check if at start of line (after whitespace)
    let p = skip_whitespace(line);
    if p.offset_from(line) as c_int == brace_col {
        return BRACE_AT_START;
    }

    BRACE_AT_END
}

/// Check if line starts with an arithmetic/assignment operator.
///
/// Looks for `+`, `-`, `*`, `/`, `%`, `=`, etc. at start of line.
///
/// # Safety
/// The pointer must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_operator_continuation(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }

    let p = rs_cin_skipcomment(s);
    let c = *p;

    // Check for common operators that might start continuation lines
    if c == b'+' as c_char
        || c == b'-' as c_char
        || c == b'*' as c_char
        || c == b'/' as c_char
        || c == b'%' as c_char
        || c == b'=' as c_char
        || c == b'<' as c_char
        || c == b'>' as c_char
        || c == b'^' as c_char
        || c == b'~' as c_char
    {
        return true;
    }

    // Check for . or -> member access
    if c == b'.' as c_char {
        return true;
    }
    if c == b'-' as c_char && *p.add(1) == b'>' as c_char {
        return true;
    }

    false
}

// ============================================================================
// Phase 2: Pure logic helpers
// ============================================================================

/// Check if string matches "label:" pattern (identifier followed by colon).
/// Returns true if found, and sets `*new_offset` to the byte offset past the ':'.
///
/// # Safety
/// All pointers must be valid. `s` must point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_islabel_skip(s: *const c_char, new_offset: *mut c_int) -> bool {
    if s.is_null() || new_offset.is_null() {
        return false;
    }

    let mut p = s;

    // Need at least one ID character
    if vim_isIDc(i32::from(*p as u8)) == 0 {
        return false;
    }

    // Skip over identifier characters
    while vim_isIDc(i32::from(*p as u8)) != 0 {
        let len = utfc_ptr2len(p);
        p = p.add(len as usize);
    }

    // Skip comments
    p = rs_cin_skipcomment(p);

    // "::" is not a label, it's C++
    if *p == b':' as c_char {
        p = p.add(1);
        if *p != b':' as c_char {
            *new_offset = p.offset_from(s) as c_int;
            return true;
        }
    }
    false
}

/// Return a pointer to the first non-empty non-comment character after a ':'.
/// Return NULL if not found.
///
/// # Safety
/// `l` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_after_label(l: *const c_char) -> *const c_char {
    if l.is_null() {
        return std::ptr::null();
    }

    let mut p = l;
    while !is_nul(*p) {
        if *p == b':' as c_char {
            if *p.add(1) == b':' as c_char {
                // skip over "::" for C++
                p = p.add(1);
            } else if !rs_cin_iscase(p.add(1), false) {
                break;
            }
        } else if *p == b'\'' as c_char && !is_nul(*p.add(1)) && *p.add(2) == b'\'' as c_char {
            p = p.add(2); // skip over 'x'
        }
        p = p.add(1);
    }
    if is_nul(*p) {
        return std::ptr::null();
    }
    let result = rs_cin_skipcomment(p.add(1));
    if is_nul(*result) {
        return std::ptr::null();
    }
    result
}

/// Check whether in "line" there is an "if", "for" or "while" before "*poffset".
/// Returns true if found, and updates "*poffset" to point to the found keyword.
///
/// # Safety
/// `line` must be valid. `poffset` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_if_for_while_before_offset(
    line: *const c_char,
    poffset: *mut c_int,
) -> bool {
    if line.is_null() || poffset.is_null() {
        return false;
    }

    let mut offset = *poffset;
    offset -= 1;
    if offset < 2 {
        return false;
    }

    // Skip whitespace backwards
    while offset > 2 && ascii_iswhite(*line.add(offset as usize) as u8) {
        offset -= 1;
    }

    // Check for "if" (2 chars)
    let check_offset = offset - 1;
    if std::ffi::CStr::from_ptr(line.add(check_offset as usize))
        .to_bytes()
        .starts_with(b"if")
        && (check_offset == 0
            || vim_isIDc(i32::from(*line.add((check_offset - 1) as usize) as u8)) == 0)
    {
        *poffset = check_offset;
        return true;
    }

    // Check for "for" (3 chars)
    if check_offset >= 1 {
        let for_offset = check_offset - 1;
        if std::ffi::CStr::from_ptr(line.add(for_offset as usize))
            .to_bytes()
            .starts_with(b"for")
            && (for_offset == 0
                || vim_isIDc(i32::from(*line.add((for_offset - 1) as usize) as u8)) == 0)
        {
            *poffset = for_offset;
            return true;
        }

        // Check for "while" (5 chars)
        if for_offset >= 2 {
            let while_offset = for_offset - 2;
            if std::ffi::CStr::from_ptr(line.add(while_offset as usize))
                .to_bytes()
                .starts_with(b"while")
                && (while_offset == 0
                    || vim_isIDc(i32::from(*line.add((while_offset - 1) as usize) as u8)) == 0)
            {
                *poffset = while_offset;
                return true;
            }
        }
    }

    false
}

/// Return `ind_maxparen` corrected for the difference in line number between the
/// cursor position and `startpos_lnum`.
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_corr_ind_maxparen(startpos_lnum: c_int) -> c_int {
    let cursor_lnum = nvim_cindent_curwin_get_cursor_lnum();
    let ind_maxparen = nvim_cindent_curbuf_get_ind_maxparen();
    let n = startpos_lnum - cursor_lnum;

    if n > 0 && n < ind_maxparen / 2 {
        ind_maxparen - n
    } else {
        ind_maxparen
    }
}

/// Recognize enumerations: "[typedef] [static|public|protected|private] enum"
/// Also recognizes compound literal initialization.
///
/// # Safety
/// `line` must be a valid null-terminated C string. Calls FFI functions.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isinit(line: *const c_char) -> bool {
    if line.is_null() {
        return false;
    }

    let mut s = rs_cin_skipcomment(line);

    // Check for "typedef" prefix
    let typedef_kw = b"typedef\0";
    if rs_cin_starts_with(s, typedef_kw.as_ptr().cast()) {
        s = rs_cin_skipcomment(s.add(7));
    }

    // Skip any combination of "static", "public", "protected", "private"
    loop {
        let mut found = false;
        for &(kw, len) in &[
            (&b"static\0"[..], 6),
            (&b"public\0"[..], 6),
            (&b"protected\0"[..], 9),
            (&b"private\0"[..], 7),
        ] {
            if rs_cin_starts_with(s, kw.as_ptr().cast()) {
                s = rs_cin_skipcomment(s.add(len));
                found = true;
                break;
            }
        }
        if !found {
            break;
        }
    }

    // Check for "enum"
    let enum_kw = b"enum\0";
    if rs_cin_starts_with(s, enum_kw.as_ptr().cast()) {
        return true;
    }

    rs_cin_is_compound_init(s)
}

/// Check if the string "line" starts with a word from 'cinwords'.
///
/// # Safety
/// `line` must be a valid null-terminated C string. Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_is_cinword(line: *const c_char) -> bool {
    if line.is_null() {
        return false;
    }

    let cinw_ptr = nvim_cindent_curbuf_get_cinw();
    if cinw_ptr.is_null() || is_nul(*cinw_ptr) {
        return false;
    }

    let line_skip = skipwhite(line);

    // We need to parse the comma-separated cinwords option.
    // Use copy_option_part to iterate.
    let cinw_len = c_strlen(cinw_ptr) + 1;
    let mut buf: Vec<u8> = vec![0u8; cinw_len];
    let cinw_buf: *mut c_char = buf.as_mut_ptr().cast();

    let mut cinw: *mut c_char = cinw_ptr.cast_mut();
    let sep = b",\0";
    while !is_nul(*cinw) {
        let len = copy_option_part(
            std::ptr::addr_of_mut!(cinw),
            cinw_buf,
            cinw_len,
            sep.as_ptr().cast(),
        );
        // Compare line_skip with cinw_buf for len bytes
        let line_bytes = std::slice::from_raw_parts(line_skip as *const u8, len);
        let buf_bytes = std::slice::from_raw_parts(cinw_buf as *const u8, len);
        if line_bytes == buf_bytes
            && (vim_iswordc(i32::from(*line_skip.add(len) as u8)) == 0
                || vim_iswordc(i32::from(*line_skip.add(len - 1) as u8)) == 0)
        {
            return true;
        }
    }

    false
}

/// Result struct for `cin_ispreproc_cont`.
#[repr(C)]
pub struct PreprocContResult {
    /// Whether it's a preprocessor continuation.
    pub found: bool,
    /// The starting line number of the preprocessor statement.
    pub lnum: c_int,
    /// The adjusted indent amount.
    pub amount: c_int,
}

/// Check if line at `lnum` is a preprocessor statement or continuation line.
/// Returns: 1 if it is (and updates `out_lnum`/`out_amount`), 0 otherwise.
///
/// # Safety
/// `out_lnum` and `out_amount` must be valid pointers. Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_ispreproc_cont(
    lnum: c_int,
    amount: c_int,
    out_lnum: *mut c_int,
    out_amount: *mut c_int,
) -> c_int {
    if out_lnum.is_null() || out_amount.is_null() {
        return 0;
    }

    let mut cur_lnum = lnum;
    let mut line = nvim_cindent_ml_get(cur_lnum);
    let mut candidate_amount = amount;

    // Check if line ends in backslash
    if !is_nul(*line) {
        let len = c_strlen(line);
        if len > 0 && *line.add(len - 1) == b'\\' as c_char {
            candidate_amount = nvim_cindent_get_indent_lnum(cur_lnum);
        }
    }

    loop {
        if rs_cin_ispreproc(line) {
            *out_lnum = cur_lnum;
            *out_amount = candidate_amount;
            return 1;
        }
        if cur_lnum == 1 {
            break;
        }
        cur_lnum -= 1;
        line = nvim_cindent_ml_get(cur_lnum);
        if is_nul(*line) {
            break;
        }
        let len = c_strlen(line);
        if len == 0 || *line.add(len - 1) != b'\\' as c_char {
            break;
        }
    }

    0
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
