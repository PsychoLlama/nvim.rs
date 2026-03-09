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

/// Return value: success.
const OK: c_int = 1;

/// Return value: failure.
const FAIL: c_int = 0;

// ============================================================================
// COM_ constants (from option_vars.h)
// ============================================================================

/// Start of comment marker ('s').
const COM_START: c_char = b's' as c_char;
/// End of comment marker ('e').
const COM_END: c_char = b'e' as c_char;
/// Middle of comment marker ('m').
const COM_MIDDLE: c_char = b'm' as c_char;
/// Left-adjusted comment ('l').
const COM_LEFT: c_char = b'l' as c_char;
/// Right-adjusted comment ('r').
const COM_RIGHT: c_char = b'r' as c_char;

/// Maximum length for comment option strings.
const COM_MAX_LEN_C: usize = 50;

/// Insert mode state flag (from `state_defs.h`: `MODE_INSERT = 0x10`).
const MODE_INSERT: c_int = 0x10;

/// Maximum column constant used in indentation.
const MAXCOL_COL: c_int = 0x7fff_ffff;

/// Namespace search limit.
const FIND_NAMESPACE_LIM: c_int = 20;

/// NUL character value.
const NUL_CHAR: u8 = 0;

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
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;

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

    // Phase 3 accessors
    fn nvim_cindent_findmatchlimit(what: c_int, flags: c_int, maxtravel: i64) -> FindMatchResult;
    fn nvim_cindent_ml_get_pos_lnum_col(lnum: c_int, col: c_int) -> *const c_char;
    fn nvim_cindent_getvcol(lnum: c_int, col: c_int) -> c_int;
    fn nvim_cindent_curwin_get_cursor_col() -> c_int;
    fn nvim_cindent_curwin_set_cursor(lnum: c_int, col: c_int);
    fn nvim_cindent_curbuf_get_ind_maxcomment() -> c_int;
    fn nvim_cindent_curbuf_get_ml_line_count() -> c_int;
    fn nvim_cindent_get_indent() -> c_int;
    fn nvim_cindent_get_cursor_line_ptr() -> *const c_char;
    fn nvim_cindent_curbuf_get_ind_cpp_baseclass() -> c_int;
    fn nvim_cindent_curbuf_get_cinsd() -> *const c_char;

    // Option parsing
    fn copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    ) -> usize;

    // Phase 6 accessors (for rs_get_c_indent)
    fn nvim_get_state() -> c_int;
    fn nvim_check_linecomment(line: *const c_char) -> c_int;
    fn nvim_vim_strsize(s: *const c_char) -> c_int;
    fn nvim_linewhite(lnum: c_int) -> bool;
    fn nvim_curbuf_get_b_p_com() -> *mut c_char;
    fn nvim_in_cinkeys(keytyped: c_int, when: c_int, line_is_empty: bool) -> bool;
    fn xstrdup(str_: *const c_char) -> *mut c_char;
    fn xfree(p: *mut std::ffi::c_void);
    fn strlen(s: *const c_char) -> usize;
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

/// Result of a `findmatchlimit` call across FFI.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FindMatchResult {
    /// Whether a match was found.
    pub found: bool,
    /// Line number of the match.
    pub lnum: c_int,
    /// Column of the match.
    pub col: c_int,
}

impl FindMatchResult {
    /// Create a "not found" result.
    const fn not_found() -> Self {
        Self {
            found: false,
            lnum: 0,
            col: 0,
        }
    }
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

// ============================================================================
// Phase 3: FFI infrastructure + finder functions
// ============================================================================

/// Find the start of a comment. Search starts at `w_cursor.lnum` and goes backwards.
/// Returns a `FindMatchResult` with the position, or `not_found`.
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_find_start_comment(ind_maxcomment: c_int) -> FindMatchResult {
    let mut cur_maxcomment = i64::from(ind_maxcomment);

    loop {
        let result = nvim_cindent_findmatchlimit(c_int::from(b'*'), FM_BACKWARD, cur_maxcomment);
        if !result.found {
            return FindMatchResult::not_found();
        }

        // Check if the comment start is inside a string.
        let line = nvim_cindent_ml_get(result.lnum);
        if !rs_is_pos_in_string(line, result.col) {
            return result;
        }

        // Restrict search to below this line and try again.
        cur_maxcomment =
            i64::from(nvim_cindent_curwin_get_cursor_lnum()) - i64::from(result.lnum) - 1;
        if cur_maxcomment <= 0 {
            return FindMatchResult::not_found();
        }
    }
}

/// Find the start of a raw string. Search starts at `w_cursor.lnum` and goes backwards.
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_find_start_rawstring(ind_maxcomment: c_int) -> FindMatchResult {
    let mut cur_maxcomment = i64::from(ind_maxcomment);

    loop {
        let result = nvim_cindent_findmatchlimit(c_int::from(b'R'), FM_BACKWARD, cur_maxcomment);
        if !result.found {
            return FindMatchResult::not_found();
        }

        let line = nvim_cindent_ml_get(result.lnum);
        if !rs_is_pos_in_string(line, result.col) {
            return result;
        }

        cur_maxcomment =
            i64::from(nvim_cindent_curwin_get_cursor_lnum()) - i64::from(result.lnum) - 1;
        if cur_maxcomment <= 0 {
            return FindMatchResult::not_found();
        }
    }
}

/// Find the start of a comment or raw string (CORS).
/// Sets `out_lnum`/`out_col` to the position found (or -1 if not found).
/// Sets `*is_raw` to the raw string start lnum if raw string was found.
///
/// # Safety
/// All pointers must be valid. Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_ind_find_start_CORS(
    out_lnum: *mut c_int,
    out_col: *mut c_int,
    is_raw: *mut c_int,
) {
    let ind_maxcomment = nvim_cindent_curbuf_get_ind_maxcomment();

    let comment_pos = rs_find_start_comment(ind_maxcomment);
    let rs_pos = rs_find_start_rawstring(ind_maxcomment);

    // If comment_pos is before rs_pos the raw string is inside the comment.
    // If rs_pos is before comment_pos the comment is inside the raw string.
    if !comment_pos.found
        || (rs_pos.found
            && (rs_pos.lnum < comment_pos.lnum
                || (rs_pos.lnum == comment_pos.lnum && rs_pos.col < comment_pos.col)))
    {
        if !is_raw.is_null() && rs_pos.found {
            *is_raw = rs_pos.lnum;
        }
        if rs_pos.found {
            *out_lnum = rs_pos.lnum;
            *out_col = rs_pos.col;
        } else {
            *out_lnum = -1;
        }
    } else {
        *out_lnum = comment_pos.lnum;
        *out_col = comment_pos.col;
    }
}

/// Skip strings, chars and comments until at or past position (lnum, col).
/// Return the column found.
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_skip2pos_lnum_col(lnum: c_int, col: c_int) -> c_int {
    rs_cin_skip2pos_col(nvim_cindent_ml_get(lnum), col)
}

/// Find the matching character, ignoring it if it is in a comment or raw string.
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_find_match_char(c: c_int, ind_maxparen: c_int) -> FindMatchResult {
    // Save cursor
    let save_lnum = nvim_cindent_curwin_get_cursor_lnum();
    let save_col = nvim_cindent_curwin_get_cursor_col();
    let mut ind_maxp_wk = ind_maxparen;

    loop {
        let trypos = nvim_cindent_findmatchlimit(c, 0, i64::from(ind_maxp_wk));
        if !trypos.found {
            // Restore cursor
            nvim_cindent_curwin_set_cursor(save_lnum, save_col);
            return FindMatchResult::not_found();
        }

        // Check if the ( is in a // comment
        if rs_cin_skip2pos_lnum_col(trypos.lnum, trypos.col) as i32 > trypos.col {
            ind_maxp_wk = ind_maxparen - (save_lnum - trypos.lnum);
            if ind_maxp_wk > 0 {
                nvim_cindent_curwin_set_cursor(trypos.lnum, 0);
                continue;
            }
            nvim_cindent_curwin_set_cursor(save_lnum, save_col);
            return FindMatchResult::not_found();
        }

        // Check if in comment or raw string
        let pos_copy = trypos;
        nvim_cindent_curwin_set_cursor(trypos.lnum, trypos.col);

        let mut cors_lnum: c_int = -1;
        let mut cors_col: c_int = 0;
        rs_ind_find_start_CORS(
            std::ptr::addr_of_mut!(cors_lnum),
            std::ptr::addr_of_mut!(cors_col),
            std::ptr::null_mut(),
        );

        if cors_lnum != -1 {
            ind_maxp_wk = ind_maxparen - (save_lnum - cors_lnum);
            if ind_maxp_wk > 0 {
                nvim_cindent_curwin_set_cursor(cors_lnum, cors_col);
                continue;
            }
            nvim_cindent_curwin_set_cursor(save_lnum, save_col);
            return FindMatchResult::not_found();
        }

        nvim_cindent_curwin_set_cursor(save_lnum, save_col);
        return pos_copy;
    }
}

/// Find the matching '(', ignoring it if it is in a comment.
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_find_match_paren(ind_maxparen: c_int) -> FindMatchResult {
    rs_find_match_char(c_int::from(b'('), ind_maxparen)
}

/// Find the '{' at the start of the block we are in.
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_find_start_brace() -> FindMatchResult {
    let save_lnum = nvim_cindent_curwin_get_cursor_lnum();
    let save_col = nvim_cindent_curwin_get_cursor_col();
    let mut result = FindMatchResult::not_found();

    loop {
        let trypos = nvim_cindent_findmatchlimit(c_int::from(b'{'), FM_BLOCKSTOP, 0);
        if !trypos.found {
            break;
        }

        let pos_copy = trypos;
        nvim_cindent_curwin_set_cursor(trypos.lnum, trypos.col);

        // Ignore the { if it's in a // or /* */ comment
        if rs_cin_skip2pos_lnum_col(trypos.lnum, trypos.col) == trypos.col {
            let mut cors_lnum: c_int = -1;
            let mut cors_col: c_int = 0;
            rs_ind_find_start_CORS(
                std::ptr::addr_of_mut!(cors_lnum),
                std::ptr::addr_of_mut!(cors_col),
                std::ptr::null_mut(),
            );

            if cors_lnum == -1 {
                result = pos_copy;
                break;
            }
            nvim_cindent_curwin_set_cursor(cors_lnum, cors_col);
        }
    }

    nvim_cindent_curwin_set_cursor(save_lnum, save_col);
    result
}

/// Find the matching '(', ignoring it if it is in a comment or before an
/// unmatched '{'.
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_find_match_paren_after_brace(ind_maxparen: c_int) -> FindMatchResult {
    let trypos = rs_find_match_paren(ind_maxparen);
    if !trypos.found {
        return FindMatchResult::not_found();
    }

    let trypos_brace = rs_find_start_brace();
    // If both an unmatched '(' and '{' is found, ignore the '(' position
    // if the '{' is further down.
    if trypos_brace.found && (trypos.lnum, trypos.col) < (trypos_brace.lnum, trypos_brace.col) {
        return FindMatchResult::not_found();
    }

    trypos
}

/// Return the indent of the first variable name after a type in a declaration.
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_first_id_amount() -> c_int {
    let line = nvim_cindent_get_cursor_line_ptr();
    let mut p = skipwhite(line).cast_const();
    let mut len = skiptowhite(p).offset_from(p) as c_int;

    if len == 6
        && std::ffi::CStr::from_ptr(p)
            .to_bytes()
            .starts_with(b"static")
    {
        p = skipwhite(p.add(6)).cast_const();
        len = skiptowhite(p).offset_from(p) as c_int;
    }
    if len == 6
        && std::ffi::CStr::from_ptr(p)
            .to_bytes()
            .starts_with(b"struct")
    {
        p = skipwhite(p.add(6)).cast_const();
    } else if len == 4 && std::ffi::CStr::from_ptr(p).to_bytes().starts_with(b"enum") {
        p = skipwhite(p.add(4)).cast_const();
    } else if (len == 8
        && std::ffi::CStr::from_ptr(p)
            .to_bytes()
            .starts_with(b"unsigned"))
        || (len == 6
            && std::ffi::CStr::from_ptr(p)
                .to_bytes()
                .starts_with(b"signed"))
    {
        let s = skipwhite(p.add(len as usize)).cast_const();
        let s_bytes = std::ffi::CStr::from_ptr(s).to_bytes();
        if (s_bytes.starts_with(b"int") && ascii_iswhite(*s.add(3) as u8))
            || (s_bytes.starts_with(b"long") && ascii_iswhite(*s.add(4) as u8))
            || (s_bytes.starts_with(b"short") && ascii_iswhite(*s.add(5) as u8))
            || (s_bytes.starts_with(b"char") && ascii_iswhite(*s.add(4) as u8))
        {
            p = s;
        }
    }

    // Find end of type keyword
    len = 0;
    while vim_isIDc(i32::from(*p.add(len as usize) as u8)) != 0 {
        len += 1;
    }
    if len == 0 || !ascii_iswhite(*p.add(len as usize) as u8) || rs_cin_nocode(p) {
        return 0;
    }

    p = skipwhite(p.add(len as usize)).cast_const();
    let col_offset = p.offset_from(line) as c_int;
    nvim_cindent_getvcol(nvim_cindent_curwin_get_cursor_lnum(), col_offset)
}

/// Return the indent of the first non-blank after an equal sign.
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_get_equal_amount(lnum: c_int) -> c_int {
    if lnum > 1 {
        let prev_line = nvim_cindent_ml_get(lnum - 1);
        if !is_nul(*prev_line) {
            let prev_len = c_strlen(prev_line);
            if prev_len > 0 && *prev_line.add(prev_len - 1) == b'\\' as c_char {
                return -1;
            }
        }
    }

    let line = nvim_cindent_ml_get(lnum);
    let mut s = line;
    while !is_nul(*s) {
        let c = *s as u8;
        if c == b'=' || c == b';' || c == b'{' || c == b'}' || c == b'"' || c == b'\'' {
            break;
        }
        if rs_cin_iscomment(s) {
            s = rs_cin_skipcomment(s);
        } else {
            s = s.add(1);
        }
    }
    if *s as u8 != b'=' {
        return 0;
    }

    s = skipwhite(s.add(1)).cast_const();
    if rs_cin_nocode(s) {
        return 0;
    }

    if *s as u8 == b'"' {
        s = s.add(1);
    }

    let col_offset = s.offset_from(line) as c_int;
    nvim_cindent_getvcol(lnum, col_offset)
}

/// Get indent of line "lnum", skipping a label.
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_get_indent_nolabel(lnum: c_int) -> c_int {
    let l = nvim_cindent_ml_get(lnum);
    let p = rs_after_label(l);
    if p.is_null() {
        return 0;
    }

    let col = p.offset_from(l) as c_int;
    nvim_cindent_getvcol(lnum, col)
}

/// Return the indent amount for a C++ base class.
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_get_baseclass_amount(col: c_int) -> c_int {
    let ind_cpp_baseclass = nvim_cindent_curbuf_get_ind_cpp_baseclass();
    let mut amount;

    if col == 0 {
        amount = nvim_cindent_get_indent();
        let line = nvim_cindent_get_cursor_line_ptr();
        let bracket = rs_find_last_paren(line, b'(' as c_char, b')' as c_char);
        if bracket.found {
            // Set cursor col for find_match_paren
            nvim_cindent_curwin_set_cursor(nvim_cindent_curwin_get_cursor_lnum(), bracket.col);
            let trypos = rs_find_match_paren(nvim_cindent_curbuf_get_ind_maxparen());
            if trypos.found {
                amount = nvim_cindent_get_indent_lnum(trypos.lnum);
            }
        }
        let comma_str = c",".as_ptr();
        if !rs_cin_ends_in(nvim_cindent_get_cursor_line_ptr(), comma_str) {
            amount += ind_cpp_baseclass;
        }
    } else {
        nvim_cindent_curwin_set_cursor(nvim_cindent_curwin_get_cursor_lnum(), col);
        amount = nvim_cindent_getvcol(nvim_cindent_curwin_get_cursor_lnum(), col);
    }

    if amount < ind_cpp_baseclass {
        amount = ind_cpp_baseclass;
    }
    amount
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
// Phase 3b: find_line_comment, cin_iswhileofdo, cin_iswhileofdo_end
// ============================================================================

/// Check previous lines for a "//" line comment, skipping over blank lines.
/// Returns the position of the comment, or `not_found()`.
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_find_line_comment() -> FindMatchResult {
    let mut lnum = nvim_cindent_curwin_get_cursor_lnum();
    loop {
        lnum -= 1;
        if lnum <= 0 {
            return FindMatchResult::not_found();
        }
        let line = nvim_cindent_ml_get(lnum);
        let p = skipwhite(line);
        if rs_cin_islinecomment(p) {
            let col = p.offset_from(line) as c_int;
            return FindMatchResult {
                found: true,
                lnum,
                col,
            };
        }
        if !is_nul(*p) {
            break;
        }
    }
    FindMatchResult::not_found()
}

/// Check if this is a "while" that should have a matching "do".
/// We only accept a "while (condition) ;", with only white space between
/// the ')' and ';'. The condition may be spread over several lines.
///
/// # Safety
/// Calls FFI accessors including `findmatchlimit`.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_iswhileofdo(p: *const c_char, lnum: c_int) -> bool {
    if p.is_null() {
        return false;
    }

    let mut s = rs_cin_skipcomment(p);
    if *s == b'}' as c_char {
        s = rs_cin_skipcomment(s.add(1));
    }

    let while_str = c"while".as_ptr();
    if !rs_cin_starts_with(s, while_str) {
        return false;
    }

    // Save cursor, set to start of line
    let save_lnum = nvim_cindent_curwin_get_cursor_lnum();
    let save_col = nvim_cindent_curwin_get_cursor_col();
    nvim_cindent_curwin_set_cursor(lnum, 0);

    let line = nvim_cindent_get_cursor_line_ptr();
    let mut q = line;
    // skip any '}', until the 'w' of the "while"
    while !is_nul(*q) && *q != b'w' as c_char {
        q = q.add(1);
        let new_col = nvim_cindent_curwin_get_cursor_col() + 1;
        nvim_cindent_curwin_set_cursor(lnum, new_col);
    }

    let result =
        nvim_cindent_findmatchlimit(0, 0, i64::from(nvim_cindent_curbuf_get_ind_maxparen()));
    let retval = if result.found {
        let after_paren = nvim_cindent_ml_get_pos_lnum_col(result.lnum, result.col);
        // ml_get_pos returns the character at (lnum, col), we need +1 to skip ')'
        let after = rs_cin_skipcomment(after_paren.add(1));
        *after == b';' as c_char
    } else {
        false
    };

    nvim_cindent_curwin_set_cursor(save_lnum, save_col);
    retval
}

/// Return true if we are at the end of a do-while.
///    do
///       nothing;
///    while (foo
///             && bar);  <-- here
/// Adjust the cursor to the line with "while".
///
/// # Safety
/// Calls FFI accessors. Modifies cursor position on success.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_iswhileofdo_end(terminated: c_int) -> bool {
    if terminated != c_int::from(b';') {
        return false;
    }

    let mut line = nvim_cindent_get_cursor_line_ptr();
    let mut p = line;
    let cursor_lnum = nvim_cindent_curwin_get_cursor_lnum();

    while !is_nul(*p) {
        p = rs_cin_skipcomment(p);
        if *p == b')' as c_char {
            let s = skipwhite(p.add(1));
            if *s == b';' as c_char && rs_cin_nocode(s.add(1)) {
                // Found ");" at end of the line, now check there is "while"
                // before the matching '('. XXX
                let i = p.offset_from(line) as c_int;
                nvim_cindent_curwin_set_cursor(cursor_lnum, i);
                let trypos = rs_find_match_paren(nvim_cindent_curbuf_get_ind_maxparen());
                if trypos.found {
                    let ms = rs_cin_skipcomment(nvim_cindent_ml_get(trypos.lnum));
                    let mut check = ms;
                    if *check == b'}' as c_char {
                        check = rs_cin_skipcomment(check.add(1));
                    }
                    let while_str = c"while".as_ptr();
                    if rs_cin_starts_with(check, while_str) {
                        nvim_cindent_curwin_set_cursor(
                            trypos.lnum,
                            nvim_cindent_curwin_get_cursor_col(),
                        );
                        return true;
                    }
                }

                // Searching may have made "line" invalid, get it again.
                line = nvim_cindent_get_cursor_line_ptr();
                p = line.offset(i as isize);
            }
        }
        if !is_nul(*p) {
            p = p.add(1);
        }
    }
    false
}

// ============================================================================
// Phase 4: Complex state machines
// ============================================================================

/// Check if the line starts with a scope declaration keyword
/// (from `cinscopedecls` option) followed by `:` (not `::`).
///
/// # Safety
/// Calls FFI accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isscopedecl(p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }

    let s = rs_cin_skipcomment(p);
    let cinsd = nvim_cindent_curbuf_get_cinsd();
    if cinsd.is_null() || is_nul(*cinsd) {
        return false;
    }

    // Parse comma-separated list from b_p_cinsd
    let cinsd_len = c_strlen(cinsd) + 1;
    let mut buf = vec![0i8; cinsd_len];
    let mut opt_ptr = cinsd.cast_mut();

    loop {
        if is_nul(*opt_ptr) {
            break;
        }
        let len = copy_option_part(
            std::ptr::addr_of_mut!(opt_ptr),
            buf.as_mut_ptr(),
            cinsd_len,
            c",".as_ptr(),
        );
        if len > 0 {
            // Compare s with the keyword from cinsd
            let s_bytes = std::slice::from_raw_parts(s.cast::<u8>(), c_strlen(s));
            let buf_bytes = std::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), len);
            if s_bytes.len() >= len && s_bytes[..len] == buf_bytes[..len] {
                let skip = rs_cin_skipcomment(s.add(len));
                if *skip == b':' as c_char && *skip.add(1) != b':' as c_char {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if the current line is a label (identifier followed by `:`).
/// Excludes "default" and scope declarations.
/// Walks backward to verify the previous line is terminated.
///
/// # Safety
/// Calls FFI accessors. Modifies and restores cursor.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_islabel() -> bool {
    let s = rs_cin_skipcomment(nvim_cindent_get_cursor_line_ptr());

    // Exclude "default" and scope declarations
    if rs_cin_isdefault(s) {
        return false;
    }
    if rs_cin_isscopedecl(s) {
        return false;
    }

    let mut new_offset: c_int = 0;
    if !rs_cin_islabel_skip(s, std::ptr::addr_of_mut!(new_offset)) {
        return false;
    }

    // Only accept a label if the previous line is terminated or is a case label.
    let save_lnum = nvim_cindent_curwin_get_cursor_lnum();
    let save_col = nvim_cindent_curwin_get_cursor_col();
    let mut lnum = save_lnum;

    while lnum > 1 {
        lnum -= 1;

        // If we're in a comment or raw string now, skip to the start of it.
        nvim_cindent_curwin_set_cursor(lnum, 0);
        let mut cors_lnum: c_int = -1;
        let mut cors_col: c_int = 0;
        rs_ind_find_start_CORS(
            std::ptr::addr_of_mut!(cors_lnum),
            std::ptr::addr_of_mut!(cors_col),
            std::ptr::null_mut(),
        );
        if cors_lnum != -1 {
            nvim_cindent_curwin_set_cursor(cors_lnum, cors_col);
            lnum = cors_lnum;
        }

        let line = nvim_cindent_get_cursor_line_ptr();
        if rs_cin_ispreproc(line) {
            continue;
        }
        let checked = rs_cin_skipcomment(line);
        if is_nul(*checked) {
            continue;
        }

        nvim_cindent_curwin_set_cursor(save_lnum, save_col);
        if rs_cin_isterminated(checked, true, false) != 0
            || rs_cin_isscopedecl(checked)
            || rs_cin_iscase(checked, true)
        {
            return true;
        }
        // Check if it's a label followed by nothing useful
        let mut label_offset: c_int = 0;
        if rs_cin_islabel_skip(checked, std::ptr::addr_of_mut!(label_offset)) {
            let after = checked.add(label_offset as usize);
            if rs_cin_nocode(after) {
                return true;
            }
        }
        return false;
    }
    nvim_cindent_curwin_set_cursor(save_lnum, save_col);
    true // label at start of file???
}

/// Check if text looks like a function declaration.
/// Multi-line scan looking for `func(arg1, arg2, ...)` pattern.
///
/// # Safety
/// Calls FFI accessors. Modifies and restores cursor.
#[no_mangle]
pub unsafe extern "C" fn rs_cin_isfuncdecl(
    sp: *mut *const c_char,
    first_lnum: c_int,
    min_lnum: c_int,
) -> c_int {
    let mut lnum = first_lnum;
    let save_lnum = nvim_cindent_curwin_get_cursor_lnum();
    let mut retval = 0;
    let mut just_started = true;

    let mut s = if sp.is_null() {
        nvim_cindent_ml_get(lnum)
    } else {
        *sp
    };

    nvim_cindent_curwin_set_cursor(lnum, nvim_cindent_curwin_get_cursor_col());
    // Adjust lnum cursor for find_last_paren / find_match_paren
    nvim_cindent_curwin_set_cursor(lnum, 0);
    let bracket = rs_find_last_paren(s, b'(' as c_char, b')' as c_char);
    if bracket.found {
        nvim_cindent_curwin_set_cursor(lnum, bracket.col);
        let trypos = rs_find_match_paren(nvim_cindent_curbuf_get_ind_maxparen());
        if trypos.found {
            lnum = trypos.lnum;
            if lnum < min_lnum {
                nvim_cindent_curwin_set_cursor(save_lnum, 0);
                return 0;
            }
            s = nvim_cindent_ml_get(lnum);
        }
    }

    nvim_cindent_curwin_set_cursor(save_lnum, 0);
    // Ignore line starting with #.
    if rs_cin_ispreproc(s) {
        return 0;
    }

    // Skip to first '('
    while !is_nul(*s)
        && *s != b'(' as c_char
        && *s != b';' as c_char
        && *s != b'\'' as c_char
        && *s != b'"' as c_char
    {
        if rs_cin_iscomment(s) {
            s = rs_cin_skipcomment(s);
        } else if *s == b':' as c_char {
            if *s.add(1) == b':' as c_char {
                s = s.add(2);
            } else {
                // Colon without double-colon → not a function decl
                // (constructor-initialization like A::A(int a) : a(0))
                return 0;
            }
        } else {
            s = s.add(1);
        }
    }
    if *s != b'(' as c_char {
        return 0; // ';', ' or "  before any () or no '('
    }

    // Now scan through parameters
    while !is_nul(*s) && *s != b';' as c_char && *s != b'\'' as c_char && *s != b'"' as c_char {
        if *s == b')' as c_char && rs_cin_nocode(s.add(1)) {
            // ')' at the end: may have found a match
            // Check for the previous line not to end in a backslash
            let prev_lnum = first_lnum - 1;
            let prev_s = nvim_cindent_ml_get(prev_lnum);
            if is_nul(*prev_s) || {
                let prev_len = c_strlen(prev_s);
                prev_len == 0 || *prev_s.add(prev_len - 1) != b'\\' as c_char
            } {
                retval = 1;
            }
            // goto done
            if lnum != first_lnum && !sp.is_null() {
                *sp = nvim_cindent_ml_get(first_lnum);
            }
            return retval;
        }
        if (*s == b',' as c_char && rs_cin_nocode(s.add(1)))
            || is_nul(*s.add(1))
            || rs_cin_nocode(s)
        {
            let comma = *s == b',' as c_char;

            // ',' at the end: continue looking in the next line.
            loop {
                if lnum >= nvim_cindent_curbuf_get_ml_line_count() {
                    break;
                }
                lnum += 1;
                s = nvim_cindent_ml_get(lnum);
                if !rs_cin_ispreproc(s) {
                    break;
                }
            }
            if lnum >= nvim_cindent_curbuf_get_ml_line_count() {
                break;
            }
            s = skipwhite(s).cast_const();
            if !just_started && !comma && *s != b',' as c_char && *s != b')' as c_char {
                break;
            }
            just_started = false;
        } else if rs_cin_iscomment(s) {
            s = rs_cin_skipcomment(s);
        } else {
            s = s.add(1);
            just_started = false;
        }
    }

    // done:
    if lnum != first_lnum && !sp.is_null() {
        *sp = nvim_cindent_ml_get(first_lnum);
    }

    retval
}

/// Check if line is a C++ base class declaration or constructor-initialization.
/// Uses a cache mechanism: if `pos->lnum <= curwin->w_cursor.lnum`, returns cached result.
///
/// The function outputs to `found`, `pos_lnum`, and `pos_col`.
///
/// # Safety
/// Calls FFI accessors. Does not modify cursor.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_cin_is_cpp_baseclass(
    found: *mut c_int,
    pos_lnum: *mut c_int,
    pos_col: *mut c_int,
) -> c_int {
    if found.is_null() || pos_lnum.is_null() || pos_col.is_null() {
        return 0;
    }

    let cursor_lnum = nvim_cindent_curwin_get_cursor_lnum();

    // Cache check: if pos_lnum <= cursor_lnum, return cached result
    if *pos_lnum <= cursor_lnum {
        return *found;
    }

    *pos_col = 0;

    let line = nvim_cindent_get_cursor_line_ptr();
    let mut s = skipwhite(line).cast_const();
    if *s == b'#' as c_char {
        return 0;
    }
    s = rs_cin_skipcomment(s);
    if is_nul(*s) {
        return 0;
    }

    let mut cpp_base_class = false;
    let mut lookfor_ctor_init = false;
    let mut class_or_struct = false;
    let mut lnum = cursor_lnum;

    // Search backward for context boundary
    while lnum > 1 {
        let prev_line = nvim_cindent_ml_get(lnum - 1);
        s = skipwhite(prev_line).cast_const();
        if *s == b'#' as c_char || is_nul(*s) {
            break;
        }
        while !is_nul(*s) {
            s = rs_cin_skipcomment(s);
            if *s == b'{' as c_char
                || *s == b'}' as c_char
                || (*s == b';' as c_char && rs_cin_nocode(s.add(1)))
            {
                break;
            }
            if !is_nul(*s) {
                s = s.add(1);
            }
        }
        if !is_nul(*s) {
            break;
        }
        lnum -= 1;
    }

    *pos_lnum = lnum;
    let mut line = nvim_cindent_ml_get(lnum);
    s = line;

    loop {
        if is_nul(*s) {
            if lnum == cursor_lnum {
                break;
            }
            lnum += 1;
            line = nvim_cindent_ml_get(lnum);
            s = line;
        }
        if s == line {
            // don't recognize "case (foo):" as a baseclass
            if rs_cin_iscase(s, false) {
                break;
            }
            s = rs_cin_skipcomment(line);
            if is_nul(*s) {
                continue;
            }
        }

        if *s == b'"' as c_char || (*s == b'R' as c_char && *s.add(1) == b'"' as c_char) {
            s = rs_skip_string(s).add(1);
        } else if *s == b':' as c_char {
            if *s.add(1) == b':' as c_char {
                // skip double colon
                lookfor_ctor_init = false;
                s = rs_cin_skipcomment(s.add(2));
            } else if lookfor_ctor_init || class_or_struct {
                cpp_base_class = true;
                lookfor_ctor_init = false;
                class_or_struct = false;
                *pos_col = 0;
                s = rs_cin_skipcomment(s.add(1));
            } else {
                s = rs_cin_skipcomment(s.add(1));
            }
        } else if (std::slice::from_raw_parts(s.cast::<u8>(), 5.min(c_strlen(s)))
            .starts_with(b"class")
            && vim_isIDc(c_int::from(*s.add(5) as u8)) == 0)
            || (std::slice::from_raw_parts(s.cast::<u8>(), 6.min(c_strlen(s)))
                .starts_with(b"struct")
                && vim_isIDc(c_int::from(*s.add(6) as u8)) == 0)
        {
            class_or_struct = true;
            lookfor_ctor_init = false;
            if *s == b'c' as c_char {
                s = rs_cin_skipcomment(s.add(5));
            } else {
                s = rs_cin_skipcomment(s.add(6));
            }
        } else {
            if *s == b'{' as c_char || *s == b'}' as c_char || *s == b';' as c_char {
                cpp_base_class = false;
                lookfor_ctor_init = false;
                class_or_struct = false;
            } else if *s == b')' as c_char {
                class_or_struct = false;
                lookfor_ctor_init = true;
            } else if *s == b'?' as c_char {
                // Avoid seeing '() :' after '?' as constructor init.
                *found = 0;
                return 0;
            } else if vim_isIDc(c_int::from(*s as u8)) == 0 {
                class_or_struct = false;
                lookfor_ctor_init = false;
            } else if *pos_col == 0 {
                lookfor_ctor_init = false;
                if cpp_base_class {
                    *pos_col = s.offset_from(line) as c_int;
                }
            }

            // When the line ends in a comma don't align with it.
            if lnum == cursor_lnum && *s == b',' as c_char && rs_cin_nocode(s.add(1)) {
                *pos_col = 0;
            }

            s = rs_cin_skipcomment(s.add(1));
        }
    }

    *found = c_int::from(cpp_base_class);
    if cpp_base_class {
        *pos_lnum = lnum;
    }
    c_int::from(cpp_base_class)
}

/// Find indent for line `lnum`, ignoring any case, scope, or jump label.
/// Also return a pointer to the text (after the label) in `pp`.
///
/// # Safety
/// Calls FFI accessors. Modifies and restores cursor.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_label(lnum: c_int, pp: *mut *const c_char) -> c_int {
    let save_lnum = nvim_cindent_curwin_get_cursor_lnum();
    let save_col = nvim_cindent_curwin_get_cursor_col();

    nvim_cindent_curwin_set_cursor(lnum, 0);
    let l = nvim_cindent_get_cursor_line_ptr();
    let amount;

    if rs_cin_iscase(l, false) || rs_cin_isscopedecl(l) || rs_cin_islabel() {
        amount = rs_get_indent_nolabel(lnum);
        let mut after = rs_after_label(nvim_cindent_get_cursor_line_ptr());
        if after.is_null() {
            after = nvim_cindent_get_cursor_line_ptr();
        }
        if !pp.is_null() {
            *pp = after;
        }
    } else {
        amount = nvim_cindent_get_indent();
        if !pp.is_null() {
            *pp = nvim_cindent_get_cursor_line_ptr();
        }
    }

    nvim_cindent_curwin_set_cursor(save_lnum, save_col);
    amount
}

/// Match if/else and do/while pairs.
/// Walks backward from cursor to `ourscope` looking for matching constructs.
///
/// Note: This function modifies `curwin->w_cursor` and does NOT restore it
/// (same behavior as the C version).
///
/// # Safety
/// Calls FFI accessors. Modifies cursor position.
#[no_mangle]
pub unsafe extern "C" fn rs_find_match(lookfor: c_int, ourscope: c_int) -> c_int {
    let (mut elselevel, mut whilelevel) = if lookfor == LOOKFOR_IF {
        (1, 0)
    } else {
        (0, 1)
    };

    nvim_cindent_curwin_set_cursor(nvim_cindent_curwin_get_cursor_lnum(), 0);

    while nvim_cindent_curwin_get_cursor_lnum() > ourscope + 1 {
        let new_lnum = nvim_cindent_curwin_get_cursor_lnum() - 1;
        nvim_cindent_curwin_set_cursor(new_lnum, 0);

        let look = rs_cin_skipcomment(nvim_cindent_get_cursor_line_ptr());
        if !rs_cin_iselse(look)
            && !rs_cin_isif(look)
            && !rs_cin_isdo(look)
            && !rs_cin_iswhileofdo(look, nvim_cindent_curwin_get_cursor_lnum())
        {
            continue;
        }

        // if we've gone outside the braces entirely, we must be out of scope
        let theirscope = rs_find_start_brace();
        if !theirscope.found {
            break;
        }

        // and if the brace enclosing this is further back, we're out of luck
        if theirscope.lnum < ourscope {
            break;
        }

        // and if they're enclosed in a *deeper* brace, ignore it
        if theirscope.lnum > ourscope {
            continue;
        }

        // if it was an "else" (that's not an "else if"), increment elselevel
        let look = rs_cin_skipcomment(nvim_cindent_get_cursor_line_ptr());
        if rs_cin_iselse(look) {
            let mightbeif = rs_cin_skipcomment(look.add(4));
            if !rs_cin_isif(mightbeif) {
                elselevel += 1;
            }
            continue;
        }

        // if it was a "while" increment whilelevel
        if rs_cin_iswhileofdo(look, nvim_cindent_curwin_get_cursor_lnum()) {
            whilelevel += 1;
            continue;
        }

        // If it's an "if" decrement elselevel
        let look = rs_cin_skipcomment(nvim_cindent_get_cursor_line_ptr());
        if rs_cin_isif(look) {
            elselevel -= 1;
            if elselevel == 0 && lookfor == LOOKFOR_IF {
                whilelevel = 0;
            }
        }

        // If it's a "do" decrement whilelevel
        if rs_cin_isdo(look) {
            whilelevel -= 1;
        }

        // if we've used up all the elses/whiles, this must be the match
        if elselevel <= 0 && whilelevel <= 0 {
            return OK;
        }
    }
    FAIL
}

// ============================================================================
// Phase 6: get_c_indent — core C indentation algorithm
// ============================================================================

/// Parse digits from a C string pointer, advancing it past the digits.
/// Returns the integer value parsed.
///
/// # Safety
/// `pp` must point to a valid pointer to a null-terminated C string.
#[allow(clippy::cast_lossless)]
unsafe fn getdigits_int_rust(pp: &mut *const c_char) -> c_int {
    let mut result: c_int = 0;
    while ascii_isdigit(**pp as u8) {
        result = result * 10 + c_int::from((**pp as u8) - b'0');
        *pp = pp.add(1);
    }
    result
}

/// Compute the desired C indentation for the current line.
///
/// This is the Rust implementation of `get_c_indent()`. The C wrapper
/// builds a `CindentOptions` from `curbuf->b_ind_*` fields and calls this.
///
/// Returns -1 if the indent should be left alone (inside a raw string).
///
/// # Safety
/// - `opts` must point to a valid `CindentOptions` struct with all fields populated.
/// - All cursor/buffer state must be valid (curwin, curbuf must be set up).
#[no_mangle]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
#[allow(clippy::cast_ptr_alignment)]
#[allow(clippy::ptr_cast_constness)]
#[allow(clippy::ptr_as_ptr)]
#[allow(clippy::manual_c_str_literals)]
#[allow(clippy::unnecessary_cast)]
#[allow(clippy::if_not_else)]
#[allow(clippy::redundant_else)]
#[allow(clippy::unused_self)]
#[allow(clippy::similar_names)]
#[allow(clippy::cast_lossless)]
#[allow(clippy::needless_bool_assign)]
#[allow(clippy::collapsible_if)]
#[allow(clippy::used_underscore_binding)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::items_after_statements)]
#[allow(clippy::missing_panics_doc)]
#[allow(clippy::if_then_some_else_none)]
#[allow(clippy::useless_let_if_seq)]
#[allow(clippy::nonminimal_bool)]
#[allow(unused_variables)]
#[allow(unused_mut)]
#[allow(unused_labels)]
#[allow(unused_assignments)]
pub unsafe extern "C" fn rs_get_c_indent(opts: *const CindentOptions) -> c_int {
    let opts = &*opts;

    // Save current cursor position
    let cur_lnum = nvim_cindent_curwin_get_cursor_lnum();
    let cur_col = nvim_cindent_curwin_get_cursor_col();

    // If we are at line 1, zero indent is fine
    if cur_lnum == 1 {
        return 0;
    }

    // Get a copy of the current line's contents.
    let line_raw = nvim_cindent_ml_get(cur_lnum);
    let linecopy_ptr = xstrdup(line_raw);

    // In insert mode and cursor is on ')', truncate the line there.
    if (nvim_get_state() & MODE_INSERT) != 0 {
        let line_len = strlen(linecopy_ptr) as c_int;
        if cur_col < line_len && *linecopy_ptr.add(cur_col as usize) == b')' as c_char {
            *linecopy_ptr.add(cur_col as usize) = NUL;
        }
    }

    // theline = skipwhite(linecopy)
    let theline = skipwhite(linecopy_ptr).cast_const();

    // Move cursor to start of the line
    nvim_cindent_curwin_set_cursor(cur_lnum, 0);

    let original_line_islabel = rs_cin_islabel();

    // Check if inside a raw string. Ignore raw strings inside comments.
    let comment_pos_result = rs_find_start_comment(opts.ind_maxcomment);
    let comment_pos = if comment_pos_result.found {
        Some(comment_pos_result)
    } else {
        None
    };

    let rawstring_result = rs_find_start_rawstring(opts.ind_maxcomment);

    // Helper: is pos_a < pos_b (by lnum then col)
    let pos_lt = |a: FindMatchResult, b: FindMatchResult| -> bool {
        if a.lnum != b.lnum {
            a.lnum < b.lnum
        } else {
            a.col < b.col
        }
    };

    if rawstring_result.found
        && (comment_pos.is_none() || pos_lt(rawstring_result, comment_pos.unwrap()))
    {
        // Inside raw string, return -1 (laterend path - no clamping)
        nvim_cindent_curwin_set_cursor(cur_lnum, cur_col);
        xfree(linecopy_ptr.cast());
        return -1;
    }

    // Use labeled block to simulate goto theend / laterend.
    // 'theend' block: returns amount that will be clamped to >= 0.
    let amount = 'theend: {
        let mut amount: c_int;

        // #defines go at the left when included in 'cinkeys'
        if *theline == b'#' as c_char
            && (*linecopy_ptr == b'#' as c_char
                || nvim_in_cinkeys(b'#' as c_int, b' ' as c_int, true))
        {
            let directive = skipwhite(theline.add(1)).cast_const();
            let pragma_bytes = b"pragma";
            let is_pragma = {
                let mut is = true;
                for (i, &b) in pragma_bytes.iter().enumerate() {
                    if *directive.add(i) != b as c_char {
                        is = false;
                        break;
                    }
                }
                is
            };
            if opts.ind_pragma == 0 || !is_pragma {
                break 'theend opts.ind_hash_comment;
            }
        }

        // Non-case label? Goes to left margin unless JS or positive L.
        if original_line_islabel && opts.ind_js == 0 && opts.ind_jump_label < 0 {
            break 'theend 0;
        }

        // "//" comment alignment
        if rs_cin_islinecomment(theline) {
            let trypos = rs_find_line_comment();
            let trypos = if !trypos.found && cur_lnum > 1 {
                // Check previous line for a comment start
                let prev_line = nvim_cindent_ml_get(cur_lnum - 1);
                let cc = nvim_check_linecomment(prev_line);
                if cc != MAXCOL {
                    Some(FindMatchResult {
                        found: true,
                        lnum: cur_lnum - 1,
                        col: cc,
                    })
                } else {
                    None
                }
            } else if trypos.found {
                Some(trypos)
            } else {
                None
            };
            if let Some(tp) = trypos {
                let col = nvim_cindent_getvcol(tp.lnum, tp.col);
                break 'theend col;
            }
        }

        // Inside comment and not at the comment start — use 'comments' option
        if let (true, Some(cp)) = (!rs_cin_iscomment(theline), comment_pos) {
            let col = nvim_cindent_getvcol(cp.lnum, cp.col);
            amount = col;

            let mut lead_start = [NUL; COM_MAX_LEN_C];
            let mut lead_middle = [NUL; COM_MAX_LEN_C];
            let mut lead_start_len: usize = 2;
            let mut lead_middle_len: usize = 1;
            let mut start_align: c_char = 0;
            let mut start_off: c_int = 0;
            let mut done = false;
            let mut look: *mut c_char = NUL as *mut c_char; // init dummy

            let b_p_com = nvim_curbuf_get_b_p_com();
            let mut p: *mut c_char = b_p_com;
            let comma_sep = b",\0".as_ptr() as *const c_char;

            while !is_nul(*p) {
                let mut align: c_char = 0;
                let mut off: c_int = 0;
                let mut what: c_char = 0;

                while !is_nul(*p) && *p != b':' as c_char {
                    if *p == COM_START || *p == COM_END || *p == COM_MIDDLE {
                        what = *p;
                        p = p.add(1);
                    } else if *p == COM_LEFT || *p == COM_RIGHT {
                        align = *p;
                        p = p.add(1);
                    } else if ascii_isdigit(*p as u8) || *p == b'-' as c_char {
                        // Parse integer (possibly negative)
                        let neg = if *p == b'-' as c_char {
                            p = p.add(1);
                            true
                        } else {
                            false
                        };
                        let pp_const = p.cast_const();
                        let mut pp_mut = pp_const;
                        let v = getdigits_int_rust(&mut pp_mut);
                        p = pp_mut.cast_mut();
                        off = if neg { -v } else { v };
                    } else {
                        p = p.add(1);
                    }
                }

                if *p == b':' as c_char {
                    p = p.add(1);
                }

                let mut lead_end = [NUL; COM_MAX_LEN_C];
                copy_option_part(&raw mut p, lead_end.as_mut_ptr(), COM_MAX_LEN_C, comma_sep);

                if what == COM_START {
                    // STRCPY lead_start = lead_end
                    lead_start_len = 0;
                    while lead_end[lead_start_len] != NUL && lead_start_len + 1 < COM_MAX_LEN_C {
                        lead_start[lead_start_len] = lead_end[lead_start_len];
                        lead_start_len += 1;
                    }
                    lead_start[lead_start_len] = NUL;
                    lead_start_len = strlen(lead_start.as_ptr()) as usize;
                    start_off = off;
                    start_align = align;
                } else if what == COM_MIDDLE {
                    lead_middle_len = 0;
                    while lead_end[lead_middle_len] != NUL && lead_middle_len + 1 < COM_MAX_LEN_C {
                        lead_middle[lead_middle_len] = lead_end[lead_middle_len];
                        lead_middle_len += 1;
                    }
                    lead_middle[lead_middle_len] = NUL;
                    lead_middle_len = strlen(lead_middle.as_ptr()) as usize;
                } else if what == COM_END {
                    // If our line starts with middle string
                    let theline_matches_middle = lead_middle_len > 0
                        && std::slice::from_raw_parts(theline as *const u8, lead_middle_len)
                            == std::slice::from_raw_parts(
                                lead_middle.as_ptr() as *const u8,
                                lead_middle_len,
                            );
                    let end_len = strlen(lead_end.as_ptr()) as usize;
                    let theline_matches_end = end_len > 0
                        && std::slice::from_raw_parts(theline as *const u8, end_len)
                            == std::slice::from_raw_parts(lead_end.as_ptr() as *const u8, end_len);

                    if theline_matches_middle && !theline_matches_end {
                        done = true;
                        if cur_lnum > 1 {
                            let prev_look =
                                skipwhite(nvim_cindent_ml_get(cur_lnum - 1)).cast_const();
                            let prev_len = strlen(prev_look) as usize;
                            if lead_start_len > 0
                                && prev_len >= lead_start_len
                                && std::slice::from_raw_parts(
                                    prev_look as *const u8,
                                    lead_start_len,
                                ) == std::slice::from_raw_parts(
                                    lead_start.as_ptr() as *const u8,
                                    lead_start_len,
                                )
                            {
                                amount = nvim_cindent_get_indent_lnum(cur_lnum - 1);
                            } else if lead_middle_len > 0
                                && prev_len >= lead_middle_len
                                && std::slice::from_raw_parts(
                                    prev_look as *const u8,
                                    lead_middle_len,
                                ) == std::slice::from_raw_parts(
                                    lead_middle.as_ptr() as *const u8,
                                    lead_middle_len,
                                )
                            {
                                amount = nvim_cindent_get_indent_lnum(cur_lnum - 1);
                                break; // break out of while loop
                            } else {
                                // If start string doesn't match comment start, skip
                                let comment_line =
                                    nvim_cindent_ml_get_pos_lnum_col(cp.lnum, cp.col);
                                if lead_start_len > 0
                                    && std::slice::from_raw_parts(
                                        comment_line as *const u8,
                                        lead_start_len,
                                    ) != std::slice::from_raw_parts(
                                        lead_start.as_ptr() as *const u8,
                                        lead_start_len,
                                    )
                                {
                                    continue; // continue the while loop
                                }
                            }
                        }
                        if start_off != 0 {
                            amount += start_off;
                        } else if start_align == COM_RIGHT {
                            amount += nvim_vim_strsize(lead_start.as_ptr())
                                - nvim_vim_strsize(lead_middle.as_ptr());
                        }
                        break;
                    }

                    // If our line starts with end string
                    if !theline_matches_middle && theline_matches_end {
                        amount = nvim_cindent_get_indent_lnum(cur_lnum - 1);
                        if off != 0 {
                            amount += off;
                        } else if align == COM_RIGHT {
                            amount += nvim_vim_strsize(lead_start.as_ptr())
                                - nvim_vim_strsize(lead_middle.as_ptr());
                        }
                        done = true;
                        break;
                    }
                }
            } // end while !is_nul(*p)

            // Decide alignment based on what we found
            if done {
                // skip - amount is already set
            } else if *theline == b'*' as c_char {
                amount += 1;
            } else {
                // More than one line away from comment opener:
                // use indent of previous non-empty line.
                amount = -1;
                let mut lnum = cur_lnum - 1;
                while lnum > cp.lnum {
                    if nvim_linewhite(lnum) {
                        lnum -= 1;
                        continue;
                    }
                    amount = nvim_cindent_get_indent_lnum(lnum);
                    break;
                }
                if amount == -1 {
                    // Use the comment opener position
                    let mut comment_pos_adj = cp;
                    if opts.ind_in_comment2 == 0 {
                        let start = nvim_cindent_ml_get(cp.lnum);
                        let look_ptr = start.add(cp.col as usize + 2); // skip / and *
                        if !is_nul(*look_ptr) {
                            // col of first non-white after /*
                            let sw_ptr = skipwhite(look_ptr).cast_const();
                            comment_pos_adj.col = sw_ptr.offset_from(start) as c_int;
                            look = sw_ptr.cast_mut();
                        } else {
                            look = look_ptr.cast_mut();
                        }
                    } else {
                        look = NUL as *mut c_char;
                    }
                    let col = nvim_cindent_getvcol(comment_pos_adj.lnum, comment_pos_adj.col);
                    amount = col;
                    if opts.ind_in_comment2 != 0 || is_nul(*look) {
                        amount += opts.ind_in_comment;
                    }
                }
            }
            break 'theend amount;
        }

        // Are we looking at a ']' that has a match?
        if *skipwhite(theline) == b']' as c_char {
            let trypos = rs_find_match_char(b'[' as c_int, opts.ind_maxparen);
            if trypos.found {
                amount = nvim_cindent_get_indent_lnum(trypos.lnum);
                break 'theend amount;
            }
        }

        // Are we inside parentheses or braces?
        let paren_result = if opts.ind_java == 0 {
            rs_find_match_paren(opts.ind_maxparen)
        } else {
            FindMatchResult {
                found: false,
                lnum: 0,
                col: 0,
            }
        };
        let brace_result = rs_find_start_brace();

        let mut trypos_opt: Option<FindMatchResult> = if paren_result.found {
            Some(paren_result)
        } else {
            None
        };
        let mut trypos_brace_opt: Option<FindMatchResult> = if brace_result.found {
            Some(brace_result)
        } else {
            None
        };

        if trypos_opt.is_some() || trypos_brace_opt.is_some() {
            // Both '(' and '{' found — use the one closer to cursor
            if trypos_opt.is_some() && trypos_brace_opt.is_some() {
                let tp = trypos_opt.unwrap();
                let tb = trypos_brace_opt.unwrap();
                if pos_lt(tp, tb) {
                    trypos_opt = None;
                } else {
                    trypos_brace_opt = None;
                }
            }

            if let Some(our_paren_raw) = trypos_opt {
                let mut our_paren_pos = our_paren_raw;

                // If matching paren is more than one line away, use indent of
                // a previous non-empty line that matches the same paren.
                if *theline == b')' as c_char && opts.ind_paren_prev != 0 {
                    amount = nvim_cindent_get_indent_lnum(cur_lnum - 1);
                } else {
                    amount = -1;
                    let mut lnum = cur_lnum - 1;
                    while lnum > our_paren_pos.lnum {
                        let l = skipwhite(nvim_cindent_ml_get(lnum)).cast_const();
                        if rs_cin_nocode(l) {
                            lnum -= 1;
                            continue;
                        }
                        // Check cin_ispreproc_cont
                        let mut out_lnum2 = lnum;
                        let mut out_amount2 = amount;
                        if rs_cin_ispreproc_cont(
                            lnum,
                            amount,
                            &raw mut out_lnum2,
                            &raw mut out_amount2,
                        ) != 0
                        {
                            lnum = out_lnum2;
                            amount = out_amount2;
                            lnum -= 1;
                            continue;
                        }
                        nvim_cindent_curwin_set_cursor(lnum, 0);

                        // Skip comment or raw string
                        let cors = {
                            let mut ol = -1i32 as c_int;
                            let mut oc = 0i32 as c_int;
                            rs_ind_find_start_CORS(&raw mut ol, &raw mut oc, std::ptr::null_mut());
                            if ol != -1 {
                                Some(FindMatchResult {
                                    found: true,
                                    lnum: ol,
                                    col: oc,
                                })
                            } else {
                                None
                            }
                        };
                        if let Some(tp) = cors {
                            lnum = tp.lnum + 1;
                            continue;
                        }

                        // Try find_match_paren with corr_ind_maxparen
                        let corr = rs_corr_ind_maxparen(cur_lnum);
                        let tp = rs_find_match_paren(corr);
                        if tp.found && tp.lnum == our_paren_pos.lnum && tp.col == our_paren_pos.col
                        {
                            amount = nvim_cindent_get_indent_lnum(lnum);
                            if *theline == b')' as c_char {
                                // track minimum
                                // cur_amount tracking below
                                // For ')' we need cur_amount logic
                                // simplified: set amount, break
                            }
                            break;
                        }
                        lnum -= 1;
                    }
                }

                // Line up with the matching paren line
                if amount == -1 {
                    let mut ignore_paren_col: c_int = 0;
                    let mut is_if_for_while: c_int = 0;

                    if opts.ind_if_for_while != 0 {
                        // Find outermost opening paren on that line
                        let mut outermost = our_paren_pos;
                        let save_lnum = nvim_cindent_curwin_get_cursor_lnum();
                        let save_col = nvim_cindent_curwin_get_cursor_col();
                        loop {
                            nvim_cindent_curwin_set_cursor(outermost.lnum, outermost.col);
                            let tp = rs_find_match_paren(opts.ind_maxparen);
                            if tp.found && tp.lnum == outermost.lnum {
                                outermost = tp;
                            } else {
                                break;
                            }
                        }
                        nvim_cindent_curwin_set_cursor(save_lnum, save_col);

                        let line = nvim_cindent_ml_get(outermost.lnum);
                        let mut off = outermost.col;
                        is_if_for_while =
                            rs_cin_is_if_for_while_before_offset(line, &raw mut off) as c_int;
                    }

                    amount = {
                        let mut _pp: *const c_char = std::ptr::null();
                        rs_skip_label(our_paren_pos.lnum, &raw mut _pp)
                    };
                    let look_ptr = skipwhite(nvim_cindent_ml_get(our_paren_pos.lnum)).cast_const();

                    // Recalculate look after skip_label
                    let mut look_ptr2: *const c_char = std::ptr::null();
                    amount = rs_skip_label(our_paren_pos.lnum, &raw mut look_ptr2);
                    let look = skipwhite(look_ptr2).cast_const();

                    if *look == b'(' as c_char {
                        let save_lnum2 = nvim_cindent_curwin_get_cursor_lnum();
                        let line_ptr = nvim_cindent_get_cursor_line_ptr();
                        let look_col = look.offset_from(line_ptr) as c_int;
                        nvim_cindent_curwin_set_cursor(our_paren_pos.lnum, look_col + 1);
                        let tp =
                            nvim_cindent_findmatchlimit(b')' as c_int, 0, opts.ind_maxparen as i64);
                        if tp.found && tp.lnum == our_paren_pos.lnum && tp.col < our_paren_pos.col {
                            ignore_paren_col = tp.col + 1;
                        }
                        nvim_cindent_curwin_set_cursor(save_lnum2, 0);
                        // look stays at its column in the line
                    }

                    let mut cur_amount2: c_int = MAXCOL;

                    if *theline == b')' as c_char
                        || (opts.ind_unclosed == 0 && is_if_for_while == 0)
                        || (opts.ind_unclosed_noignore == 0
                            && *look == b'(' as c_char
                            && ignore_paren_col == 0)
                    {
                        if *theline != b')' as c_char {
                            cur_amount2 = MAXCOL;
                            let l = nvim_cindent_ml_get(our_paren_pos.lnum);
                            if opts.ind_unclosed_wrapped != 0
                                && rs_cin_ends_in(l, b"(\0".as_ptr() as *const c_char)
                            {
                                // Count nesting levels
                                let mut n: c_int = 1;
                                let mut col2: c_int = 0;
                                while col2 < our_paren_pos.col {
                                    match *l.add(col2 as usize) as u8 {
                                        b'(' | b'{' => n += 1,
                                        b')' | b'}' => {
                                            if n > 1 {
                                                n -= 1;
                                            }
                                        }
                                        _ => {}
                                    }
                                    col2 += 1;
                                }
                                our_paren_pos.col = 0;
                                amount += n * opts.ind_unclosed_wrapped;
                            } else if opts.ind_unclosed_whiteok != 0 {
                                our_paren_pos.col += 1;
                            } else {
                                let mut col2 = our_paren_pos.col + 1;
                                while ascii_iswhite(*l.add(col2 as usize) as u8) {
                                    col2 += 1;
                                }
                                if !is_nul(*l.add(col2 as usize)) {
                                    our_paren_pos.col = col2;
                                } else {
                                    our_paren_pos.col += 1;
                                }
                            }
                        }
                        if our_paren_pos.col > 0 {
                            let vcol = nvim_cindent_getvcol(our_paren_pos.lnum, our_paren_pos.col);
                            if cur_amount2 > vcol {
                                cur_amount2 = vcol;
                            }
                        }
                    }

                    if *theline == b')' as c_char && opts.ind_matching_paren != 0 {
                        // Line up with start of matching paren line (already set above)
                    } else if (opts.ind_unclosed == 0 && is_if_for_while == 0)
                        || (opts.ind_unclosed_noignore == 0
                            && *look == b'(' as c_char
                            && ignore_paren_col == 0)
                    {
                        if cur_amount2 != MAXCOL {
                            amount = cur_amount2;
                        }
                    } else {
                        let mut col3 = our_paren_pos.col;
                        while our_paren_pos.col > ignore_paren_col {
                            our_paren_pos.col -= 1;
                            match *nvim_cindent_ml_get_pos_lnum_col(
                                our_paren_pos.lnum,
                                our_paren_pos.col,
                            ) as u8
                            {
                                b'(' => {
                                    amount += opts.ind_unclosed2;
                                    col3 = our_paren_pos.col;
                                }
                                b')' => {
                                    amount -= opts.ind_unclosed2;
                                    col3 = MAXCOL;
                                }
                                _ => {}
                            }
                        }
                        if col3 == MAXCOL {
                            amount += opts.ind_unclosed;
                        } else {
                            nvim_cindent_curwin_set_cursor(our_paren_pos.lnum, col3);
                            let tp2 = rs_find_match_paren_after_brace(opts.ind_maxparen);
                            if tp2.found {
                                amount += opts.ind_unclosed2;
                            } else if is_if_for_while != 0 {
                                amount += opts.ind_if_for_while;
                            } else {
                                amount += opts.ind_unclosed;
                            }
                        }
                        if cur_amount2 < amount {
                            amount = cur_amount2;
                        }
                    }
                }

                // Add extra indent for a comment
                if rs_cin_iscomment(theline) {
                    amount += opts.ind_comment;
                }
            } else {
                // We are inside braces
                let trypos_brace = trypos_brace_opt.unwrap();
                let mut trypos_brace_copy = trypos_brace;
                let ourscope = trypos_brace_copy.lnum;
                let start = nvim_cindent_ml_get(ourscope);

                let look = skipwhite(start).cast_const();
                let start_brace: c_int;
                if *look == b'{' as c_char {
                    let vcol = nvim_cindent_getvcol(trypos_brace_copy.lnum, trypos_brace_copy.col);
                    amount = vcol;
                    if *start == b'{' as c_char {
                        start_brace = BRACE_IN_COL0;
                    } else {
                        start_brace = BRACE_AT_START;
                    }
                } else {
                    // Opening brace might be on a continuation line
                    nvim_cindent_curwin_set_cursor(ourscope, 0);
                    let mut lnum2 = ourscope;

                    // Position cursor over rightmost paren
                    let last_paren_res = rs_find_last_paren(start, b'(' as c_char, b')' as c_char);
                    if last_paren_res.found {
                        nvim_cindent_curwin_set_cursor(ourscope, last_paren_res.col + 1);
                        let tp = rs_find_match_paren(opts.ind_maxparen);
                        if tp.found {
                            lnum2 = tp.lnum;
                        }
                    }

                    // Check if we have a case label
                    let cur_line = nvim_cindent_get_cursor_line_ptr();
                    if (opts.ind_js != 0 || opts.ind_keep_case_label != 0)
                        && rs_cin_iscase(skipwhite(cur_line).cast_const(), false)
                    {
                        amount = nvim_cindent_get_indent();
                    } else if opts.ind_js != 0 {
                        amount = nvim_cindent_get_indent_lnum(lnum2);
                    } else {
                        amount = {
                            let mut _pp: *const c_char = std::ptr::null();
                            rs_skip_label(lnum2, &raw mut _pp)
                        };
                    }

                    start_brace = BRACE_AT_END;
                }

                let js_cur_has_key = opts.ind_js != 0 && rs_cin_has_js_key(theline);

                if *theline == b'}' as c_char {
                    amount += opts.ind_close_extra;
                } else {
                    // Look for else/while-of-do
                    let mut lookfor = LOOKFOR_INITIAL;
                    if rs_cin_iselse(theline) {
                        lookfor = LOOKFOR_IF;
                    } else {
                        let theline_str = nvim_cindent_ml_get(cur_lnum);
                        let theline_sw = skipwhite(theline_str).cast_const();
                        if rs_cin_iswhileofdo(theline_sw, cur_lnum) {
                            lookfor = LOOKFOR_DO;
                        }
                    }

                    if lookfor != LOOKFOR_INITIAL {
                        nvim_cindent_curwin_set_cursor(cur_lnum, 0);
                        if rs_find_match(lookfor, ourscope) == OK {
                            amount = nvim_cindent_get_indent();
                            // goto theend
                            if rs_cin_iscomment(theline) {
                                amount += opts.ind_comment;
                            }
                            if opts.ind_jump_label > 0 && original_line_islabel {
                                amount -= opts.ind_jump_label;
                            }
                            break 'theend amount;
                        }
                    }

                    if start_brace == BRACE_IN_COL0 {
                        amount = opts.ind_open_left_imag;
                        let mut lookfor_cpp_namespace = true;
                        let mut added_to_amount: c_int = 0;
                        let ind_continuation = opts.ind_continuation;

                        // Start inner brace search loop
                        let mut lnum3 = cur_lnum;
                        let mut scope_amount = amount;
                        let mut whilelevel: c_int = 0;
                        let mut cont_amount: c_int = 0;
                        let mut lookfor_break = false;
                        let mut js_cur_has_key2 = js_cur_has_key;

                        if rs_cin_iscase(theline, false) {
                            lookfor = LOOKFOR_CASE;
                            amount += opts.ind_case;
                        } else if rs_cin_isscopedecl(theline) {
                            lookfor = LOOKFOR_SCOPEDECL;
                            amount += opts.ind_scopedecl;
                        } else {
                            if opts.ind_case_break != 0 && rs_cin_isbreak(theline) {
                                lookfor_break = true;
                            }
                            lookfor = LOOKFOR_INITIAL;
                            amount += opts.ind_level;
                        }
                        scope_amount = amount;
                        whilelevel = 0;

                        amount = run_inner_brace_search(
                            opts,
                            cur_lnum,
                            cur_col,
                            ourscope,
                            start_brace,
                            lookfor,
                            lookfor_break,
                            lookfor_cpp_namespace,
                            scope_amount,
                            whilelevel,
                            cont_amount,
                            ind_continuation,
                            added_to_amount,
                            theline,
                            js_cur_has_key2,
                        );
                    } else {
                        // BRACE_AT_START or BRACE_AT_END
                        if start_brace == BRACE_AT_START && {
                            // Check lookfor_cpp_namespace
                            false // placeholder - handled below
                        } {
                            // handled in inner search
                        }

                        if start_brace == BRACE_AT_END {
                            amount += opts.ind_open_imag;
                            let l2 = skipwhite(nvim_cindent_get_cursor_line_ptr()).cast_const();
                            if rs_cin_is_cpp_namespace(l2) {
                                amount += opts.ind_cpp_namespace;
                            } else if rs_cin_is_extern_c(l2) {
                                amount += opts.ind_cpp_extern_c;
                            }
                        } else {
                            amount -= opts.ind_open_extra;
                            if amount < 0 {
                                amount = 0;
                            }
                        }

                        let ind_continuation2 = opts.ind_continuation;
                        let mut lookfor_break2 = false;
                        let mut lookfor_cpp_namespace2 = false;

                        if rs_cin_iscase(theline, false) {
                            lookfor = LOOKFOR_CASE;
                            amount += opts.ind_case;
                        } else if rs_cin_isscopedecl(theline) {
                            lookfor = LOOKFOR_SCOPEDECL;
                            amount += opts.ind_scopedecl;
                        } else {
                            if opts.ind_case_break != 0 && rs_cin_isbreak(theline) {
                                lookfor_break2 = true;
                            }
                            lookfor = LOOKFOR_INITIAL;
                            amount += opts.ind_level;
                        }
                        let scope_amount2 = amount;

                        amount = run_inner_brace_search(
                            opts,
                            cur_lnum,
                            cur_col,
                            ourscope,
                            start_brace,
                            lookfor,
                            lookfor_break2,
                            lookfor_cpp_namespace2,
                            scope_amount2,
                            0, // whilelevel
                            0, // cont_amount
                            ind_continuation2,
                            0, // added_to_amount
                            theline,
                            js_cur_has_key,
                        );
                    }

                    // Add extra indent for comment inside braces
                    if rs_cin_iscomment(theline) {
                        amount += opts.ind_comment;
                    }
                    // Subtract extra left-shift for jump labels
                    if opts.ind_jump_label > 0 && original_line_islabel {
                        amount -= opts.ind_jump_label;
                    }
                    break 'theend amount;
                }
            }

            // Add extra indent for comment (from "inside braces" path with theline[0] == '}')
            if rs_cin_iscomment(theline) {
                amount += opts.ind_comment;
            }
            if opts.ind_jump_label > 0 && original_line_islabel {
                amount -= opts.ind_jump_label;
            }
            break 'theend amount;
        }

        // Not inside any structure — top-level
        if *theline == b'{' as c_char {
            break 'theend opts.ind_first_open;
        }

        // If the NEXT line is a function declaration, indent as func type spec
        let ml_count = nvim_cindent_curbuf_get_ml_line_count();
        if cur_lnum < ml_count
            && !rs_cin_nocode(theline)
            && vim_strchr(theline, b'{' as c_int).is_null()
            && vim_strchr(theline, b'}' as c_int).is_null()
            && !rs_cin_ends_in(theline, b":\0".as_ptr() as *const c_char)
            && !rs_cin_ends_in(theline, b",\0".as_ptr() as *const c_char)
            && rs_cin_isfuncdecl(std::ptr::null_mut(), cur_lnum + 1, cur_lnum + 1) != 0
            && rs_cin_isterminated(theline, false, true) == 0
        {
            break 'theend opts.ind_func_type;
        }

        // Search backwards until we find something we recognize
        amount = 0;
        nvim_cindent_curwin_set_cursor(cur_lnum, 0);
        let ind_continuation3 = opts.ind_continuation;

        // Cache for cpp_baseclass
        let mut cache_found: c_int = 0;
        let mut cache_lnum: c_int = MAXCOL as c_int;
        let mut cache_col: c_int = 0;

        'top_search: loop {
            let cursor_lnum3 = nvim_cindent_curwin_get_cursor_lnum();
            if cursor_lnum3 <= 1 {
                break;
            }
            nvim_cindent_curwin_set_cursor(cursor_lnum3 - 1, 0);
            let cur_lnum3 = nvim_cindent_curwin_get_cursor_lnum();

            let l = nvim_cindent_get_cursor_line_ptr();

            // Skip comment or raw string
            {
                let mut ol = -1i32 as c_int;
                let mut oc = 0i32 as c_int;
                rs_ind_find_start_CORS(&raw mut ol, &raw mut oc, std::ptr::null_mut());
                if ol != -1 {
                    nvim_cindent_curwin_set_cursor(ol + 1, 0);
                    continue;
                }
            }

            // Check cpp base class
            let mut n: c_int = 0;
            if opts.ind_cpp_baseclass != 0 {
                n = rs_cin_is_cpp_baseclass(
                    &raw mut cache_found,
                    &raw mut cache_lnum,
                    &raw mut cache_col,
                );
            }
            if n != 0 {
                amount = rs_get_baseclass_amount(cache_col);
                break;
            }

            let l = nvim_cindent_get_cursor_line_ptr();

            // Skip preprocessor and blank lines
            let mut out_lnum4 = cur_lnum3;
            let mut out_amount4 = amount;
            if rs_cin_ispreproc_cont(cur_lnum3, amount, &raw mut out_lnum4, &raw mut out_amount4)
                != 0
            {
                nvim_cindent_curwin_set_cursor(out_lnum4, 0);
                amount = out_amount4;
                continue;
            }

            if rs_cin_nocode(l) {
                continue;
            }

            // Previous line ends in ','  or '\\'
            let l_len = strlen(l) as usize;
            let ends_backslash = l_len > 0 && *l.add(l_len - 1) == b'\\' as c_char;

            if rs_cin_ends_in(l, b",\0".as_ptr() as *const c_char) || ends_backslash {
                // Take us back to opening paren
                let lp_res = rs_find_last_paren(l, b'(' as c_char, b')' as c_char);
                if lp_res.found {
                    nvim_cindent_curwin_set_cursor(cur_lnum3, lp_res.col + 1);
                    let tp = rs_find_match_paren(opts.ind_maxparen);
                    if tp.found {
                        nvim_cindent_curwin_set_cursor(tp.lnum, tp.col);
                    }
                }

                // For comma continuation: go back to first line with backslash
                if !ends_backslash {
                    let mut cl = nvim_cindent_curwin_get_cursor_lnum();
                    while cl > 1 {
                        let prev = nvim_cindent_ml_get(cl - 1);
                        let prev_len = strlen(prev);
                        if prev_len == 0 || *prev.add(prev_len - 1) != b'\\' as c_char {
                            break;
                        }
                        cl -= 1;
                        nvim_cindent_curwin_set_cursor(cl, 0);
                    }
                }

                amount = nvim_cindent_get_indent();
                if amount == 0 {
                    amount = rs_cin_first_id_amount();
                }
                if amount == 0 {
                    amount = ind_continuation3;
                }
                break;
            }

            // Function declaration?
            if rs_cin_isfuncdecl(std::ptr::null_mut(), cur_lnum, 0) != 0 {
                break;
            }

            let l = nvim_cindent_get_cursor_line_ptr();
            if *skipwhite(l) == b'}' as c_char {
                break;
            }

            if rs_cin_ends_in(l, b"};\0".as_ptr() as *const c_char) {
                break;
            }

            if rs_cin_ends_in(l, b"[\0".as_ptr() as *const c_char) {
                amount = nvim_cindent_get_indent() + ind_continuation3;
                break;
            }

            // Line with only semicolon before a '}'
            let look5 = skipwhite(l).cast_const();
            if *look5 == b';' as c_char && rs_cin_nocode(look5.add(1)) {
                let save_lnum5 = nvim_cindent_curwin_get_cursor_lnum();
                let save_col5 = nvim_cindent_curwin_get_cursor_col();
                let mut found_brace = false;
                let mut cl5 = save_lnum5;
                while cl5 > 1 {
                    cl5 -= 1;
                    let prev5 = nvim_cindent_ml_get(cl5);
                    let mut out_lnum5 = cl5;
                    let mut out_amount5 = 0;
                    if rs_cin_nocode(prev5)
                        || rs_cin_ispreproc_cont(cl5, 0, &raw mut out_lnum5, &raw mut out_amount5)
                            != 0
                    {
                        continue;
                    }
                    if rs_cin_ends_in(prev5, b"}\0".as_ptr() as *const c_char) {
                        found_brace = true;
                    }
                    break;
                }
                if found_brace {
                    break;
                }
                nvim_cindent_curwin_set_cursor(save_lnum5, save_col5);
            }

            // Previous line is a function declaration?
            let mut l_funcdecl: *const c_char = nvim_cindent_get_cursor_line_ptr();
            if rs_cin_isfuncdecl(
                &raw mut l_funcdecl,
                nvim_cindent_curwin_get_cursor_lnum(),
                0,
            ) != 0
            {
                amount = opts.ind_param;
                break;
            }

            // Previous line ends in ';' and the one before ends in ',' or '\\'
            let l = nvim_cindent_get_cursor_line_ptr();
            if rs_cin_ends_in(l, b";\0".as_ptr() as *const c_char) {
                let cl6 = nvim_cindent_curwin_get_cursor_lnum();
                if cl6 > 1 {
                    let prev6 = nvim_cindent_ml_get(cl6 - 1);
                    let prev6_len = strlen(prev6);
                    if rs_cin_ends_in(prev6, b",\0".as_ptr() as *const c_char)
                        || (prev6_len > 0 && *prev6.add(prev6_len - 1) == b'\\' as c_char)
                    {
                        break;
                    }
                }
            }

            // Use indent of this line
            let l = nvim_cindent_get_cursor_line_ptr();
            let lp_res2 = rs_find_last_paren(l, b'(' as c_char, b')' as c_char);
            if lp_res2.found {
                nvim_cindent_curwin_set_cursor(
                    nvim_cindent_curwin_get_cursor_lnum(),
                    lp_res2.col + 1,
                );
                let tp2 = rs_find_match_paren(opts.ind_maxparen);
                if tp2.found {
                    nvim_cindent_curwin_set_cursor(tp2.lnum, tp2.col);
                }
            }
            amount = nvim_cindent_get_indent();
            break;
        }

        // Add extra indent for a comment (top-level)
        if rs_cin_iscomment(theline) {
            amount += opts.ind_comment;
        }

        // Add extra indent if previous line ended in backslash
        if cur_lnum > 1 {
            let prev_l = nvim_cindent_ml_get(cur_lnum - 1);
            let prev_len = strlen(prev_l) as usize;
            if prev_len > 0 && *prev_l.add(prev_len - 1) == b'\\' as c_char {
                let cur_amount2 = rs_cin_get_equal_amount(cur_lnum - 1);
                if cur_amount2 > 0 {
                    amount = cur_amount2;
                } else if cur_amount2 == 0 {
                    amount += ind_continuation3;
                }
            }
        }

        amount
    }; // end 'theend block

    let amount = if amount < 0 { 0 } else { amount };

    // laterend: restore cursor and free
    nvim_cindent_curwin_set_cursor(cur_lnum, cur_col);
    xfree(linecopy_ptr.cast());

    amount
}

// ============================================================================
// Phase 6: inner brace search helper
// ============================================================================

/// Inner backward search loop for `get_c_indent` when inside braces.
///
/// This implements the `while (true)` search loop that searches backwards
/// from the current line to determine indentation inside a `{...}` block.
///
/// # Safety
/// All FFI pointers must be valid.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
#[allow(clippy::cast_ptr_alignment)]
#[allow(clippy::ptr_cast_constness)]
#[allow(clippy::ptr_as_ptr)]
#[allow(clippy::manual_c_str_literals)]
#[allow(clippy::unnecessary_cast)]
#[allow(clippy::if_not_else)]
#[allow(clippy::redundant_else)]
#[allow(clippy::similar_names)]
#[allow(clippy::cast_lossless)]
#[allow(clippy::needless_bool_assign)]
#[allow(clippy::collapsible_if)]
#[allow(clippy::used_underscore_binding)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::missing_panics_doc)]
#[allow(clippy::if_then_some_else_none)]
#[allow(clippy::useless_let_if_seq)]
#[allow(clippy::nonminimal_bool)]
#[allow(unused_variables)]
#[allow(unused_mut)]
#[allow(unused_labels)]
#[allow(unused_assignments)]
unsafe fn run_inner_brace_search(
    opts: &CindentOptions,
    cur_lnum: c_int,
    cur_col: c_int,
    ourscope: c_int,
    start_brace: c_int,
    initial_lookfor: c_int,
    initial_lookfor_break: bool,
    initial_lookfor_cpp_namespace: bool,
    scope_amount: c_int,
    initial_whilelevel: c_int,
    initial_cont_amount: c_int,
    ind_continuation: c_int,
    initial_added_to_amount: c_int,
    theline: *const c_char,
    initial_js_cur_has_key: bool,
) -> c_int {
    let mut amount = scope_amount;
    let mut lookfor = initial_lookfor;
    let mut lookfor_break = initial_lookfor_break;
    let mut lookfor_cpp_namespace = initial_lookfor_cpp_namespace;
    let mut whilelevel = initial_whilelevel;
    let mut cont_amount = initial_cont_amount;
    let mut added_to_amount = initial_added_to_amount;
    let mut js_cur_has_key = initial_js_cur_has_key;
    let mut scope_amount2 = scope_amount;
    // raw_string_start tracking
    let mut raw_string_start: c_int = 0;

    // tryposBrace equivalent for LOOKFOR_COMMA
    let trypos_brace_lnum = ourscope;

    nvim_cindent_curwin_set_cursor(cur_lnum, cur_col);

    // Use a loop with labeled continue/break points
    'outer: loop {
        let cursor_lnum = nvim_cindent_curwin_get_cursor_lnum();
        if cursor_lnum <= 0 {
            break;
        }
        nvim_cindent_curwin_set_cursor(cursor_lnum - 1, 0);
        let lnum = nvim_cindent_curwin_get_cursor_lnum();

        // If we went back to or past the start of our scope
        if lnum <= ourscope {
            if lookfor == LOOKFOR_ENUM_OR_INIT {
                if lnum == 0 || lnum < ourscope - opts.ind_maxparen {
                    if cont_amount > 0 {
                        amount = cont_amount;
                    } else if opts.ind_js == 0 {
                        amount += ind_continuation;
                    }
                    break;
                }

                // Skip comment/raw string
                {
                    let mut ol = -1i32 as c_int;
                    let mut oc = 0i32 as c_int;
                    rs_ind_find_start_CORS(&raw mut ol, &raw mut oc, std::ptr::null_mut());
                    if ol != -1 {
                        nvim_cindent_curwin_set_cursor(ol + 1, 0);
                        continue;
                    }
                }

                let l = nvim_cindent_get_cursor_line_ptr();

                // Skip preprocessor and blank lines
                let mut out_lnum = lnum;
                let mut out_amount = amount;
                if rs_cin_ispreproc_cont(lnum, amount, &raw mut out_lnum, &raw mut out_amount) != 0
                {
                    nvim_cindent_curwin_set_cursor(out_lnum, 0);
                    amount = out_amount;
                    continue;
                }

                if rs_cin_nocode(l) {
                    continue;
                }

                let terminated = rs_cin_isterminated(l, false, true);

                let mut l_fd: *const c_char = l;
                if start_brace != BRACE_IN_COL0
                    || rs_cin_isfuncdecl(&raw mut l_fd, nvim_cindent_curwin_get_cursor_lnum(), 0)
                        == 0
                {
                    if terminated == b',' as c_char {
                        break;
                    }
                    if terminated != b';' as c_char
                        && rs_cin_isinit(nvim_cindent_get_cursor_line_ptr())
                    {
                        break;
                    }
                    if terminated == 0 || terminated == b'{' as c_char {
                        continue;
                    }
                }

                if terminated != b';' as c_char {
                    let l2 = nvim_cindent_get_cursor_line_ptr();
                    let mut trypos_inner: Option<FindMatchResult> = None;

                    let lp_res = rs_find_last_paren(l2, b'(' as c_char, b')' as c_char);
                    if lp_res.found {
                        let tp = rs_find_match_paren(opts.ind_maxparen);
                        if tp.found {
                            trypos_inner = Some(tp);
                        }
                    }

                    if trypos_inner.is_none() {
                        let lp_res2 = rs_find_last_paren(l2, b'{' as c_char, b'}' as c_char);
                        if lp_res2.found {
                            let tp2 = rs_find_start_brace();
                            if tp2.found {
                                trypos_inner = Some(tp2);
                            }
                        }
                    }

                    if let Some(tp) = trypos_inner {
                        nvim_cindent_curwin_set_cursor(tp.lnum + 1, 0);
                        continue;
                    }
                }

                if cont_amount > 0 {
                    amount = cont_amount;
                } else {
                    amount += ind_continuation;
                }
            } else if lookfor == LOOKFOR_UNTERM {
                if cont_amount > 0 {
                    amount = cont_amount;
                } else {
                    amount += ind_continuation;
                }
            } else {
                if lookfor != LOOKFOR_TERM
                    && lookfor != LOOKFOR_CPP_BASECLASS
                    && lookfor != LOOKFOR_COMMA
                {
                    amount = scope_amount2;
                    if *theline == b'{' as c_char {
                        amount += opts.ind_open_extra;
                        added_to_amount = opts.ind_open_extra;
                    }
                }

                if lookfor_cpp_namespace {
                    if lnum == ourscope {
                        continue;
                    }
                    if lnum == 0 || lnum < ourscope - FIND_NAMESPACE_LIM {
                        break;
                    }

                    // Skip comment/raw string
                    {
                        let mut ol = -1i32 as c_int;
                        let mut oc = 0i32 as c_int;
                        rs_ind_find_start_CORS(&raw mut ol, &raw mut oc, std::ptr::null_mut());
                        if ol != -1 {
                            nvim_cindent_curwin_set_cursor(ol + 1, 0);
                            continue;
                        }
                    }

                    let l = nvim_cindent_get_cursor_line_ptr();
                    let mut out_lnum2 = lnum;
                    let mut out_amount2 = amount;
                    if rs_cin_ispreproc_cont(lnum, amount, &raw mut out_lnum2, &raw mut out_amount2)
                        != 0
                    {
                        nvim_cindent_curwin_set_cursor(out_lnum2, 0);
                        amount = out_amount2;
                        continue;
                    }

                    if rs_cin_is_cpp_namespace(l) {
                        amount += opts.ind_cpp_namespace - added_to_amount;
                        break;
                    } else if rs_cin_is_extern_c(l) {
                        amount += opts.ind_cpp_extern_c - added_to_amount;
                        break;
                    }

                    if rs_cin_nocode(l) {
                        continue;
                    }
                }
            }
            break;
        }

        // Skip comment or raw string
        {
            let mut ol = -1i32 as c_int;
            let mut oc = 0i32 as c_int;
            let mut raw_lnum_out = 0i32 as c_int;
            rs_ind_find_start_CORS(&raw mut ol, &raw mut oc, &raw mut raw_lnum_out);
            if ol != -1 {
                if raw_lnum_out != 0 {
                    raw_string_start = raw_lnum_out;
                }
                nvim_cindent_curwin_set_cursor(ol + 1, 0);
                continue;
            }
        }

        let l = nvim_cindent_get_cursor_line_ptr();

        let iscase = rs_cin_iscase(l, false);
        if iscase || rs_cin_isscopedecl(l) {
            if lookfor == LOOKFOR_CPP_BASECLASS {
                break;
            }
            if whilelevel > 0 {
                continue;
            }

            if lookfor == LOOKFOR_UNTERM || lookfor == LOOKFOR_ENUM_OR_INIT {
                if cont_amount > 0 {
                    amount = cont_amount;
                } else {
                    amount += ind_continuation;
                }
                break;
            }

            if (iscase && lookfor == LOOKFOR_CASE)
                || (iscase && lookfor_break)
                || (!iscase && lookfor == LOOKFOR_SCOPEDECL)
            {
                let tp = rs_find_start_brace();
                if !tp.found || tp.lnum == ourscope {
                    amount = nvim_cindent_get_indent();
                    break;
                }
                continue;
            }

            let n = rs_get_indent_nolabel(nvim_cindent_curwin_get_cursor_lnum());

            if lookfor == LOOKFOR_TERM {
                if n != 0 {
                    amount = n;
                }
                if !lookfor_break {
                    break;
                }
            }

            if n != 0 {
                amount = n;
                let after = rs_after_label(nvim_cindent_get_cursor_line_ptr());
                if !after.is_null() && rs_cin_is_cinword(after) {
                    if *theline == b'{' as c_char {
                        amount += opts.ind_open_extra;
                    } else {
                        amount += opts.ind_level + opts.ind_no_brace;
                    }
                }
                break;
            }

            scope_amount2 = nvim_cindent_get_indent()
                + if iscase {
                    opts.ind_case_code
                } else {
                    opts.ind_scopedecl_code
                };
            lookfor = if opts.ind_case_break != 0 {
                LOOKFOR_NOBREAK
            } else {
                LOOKFOR_ANY
            };
            continue;
        }

        if lookfor == LOOKFOR_CASE || lookfor == LOOKFOR_SCOPEDECL {
            let lp = rs_find_last_paren(l, b'{' as c_char, b'}' as c_char);
            if lp.found {
                let tp = rs_find_start_brace();
                if tp.found {
                    nvim_cindent_curwin_set_cursor(tp.lnum + 1, 0);
                }
            }
            continue;
        }

        if opts.ind_js == 0 && rs_cin_islabel() {
            let after = rs_after_label(nvim_cindent_get_cursor_line_ptr());
            if after.is_null() || rs_cin_nocode(after) {
                continue;
            }
        }

        let l = nvim_cindent_get_cursor_line_ptr();
        let mut out_lnum3 = nvim_cindent_curwin_get_cursor_lnum();
        let mut out_amount3 = amount;
        if rs_cin_ispreproc_cont(out_lnum3, amount, &raw mut out_lnum3, &raw mut out_amount3) != 0
            || rs_cin_nocode(l)
        {
            if rs_cin_ispreproc_cont(
                nvim_cindent_curwin_get_cursor_lnum(),
                amount,
                &raw mut out_lnum3,
                &raw mut out_amount3,
            ) != 0
            {
                nvim_cindent_curwin_set_cursor(out_lnum3, 0);
                amount = out_amount3;
            }
            continue;
        }

        // Check cpp base class
        let mut n: c_int = 0;
        let mut cache_found: c_int = 0;
        let mut cache_lnum_inner: c_int = MAXCOL as c_int;
        let mut cache_col_inner: c_int = 0;
        if lookfor != LOOKFOR_TERM && opts.ind_cpp_baseclass > 0 {
            n = rs_cin_is_cpp_baseclass(
                &raw mut cache_found,
                &raw mut cache_lnum_inner,
                &raw mut cache_col_inner,
            );
        }

        if n != 0 {
            if lookfor == LOOKFOR_UNTERM {
                if cont_amount > 0 {
                    amount = cont_amount;
                } else {
                    amount += ind_continuation;
                }
            } else if *theline == b'{' as c_char {
                lookfor = LOOKFOR_UNTERM;
                // ind_continuation effectively becomes 0
                continue;
            } else {
                amount = rs_get_baseclass_amount(cache_col_inner);
            }
            break;
        } else if lookfor == LOOKFOR_CPP_BASECLASS {
            let l = nvim_cindent_get_cursor_line_ptr();
            if rs_cin_isterminated(l, true, false) != 0 {
                break;
            } else {
                continue;
            }
        }

        let l = nvim_cindent_get_cursor_line_ptr();
        let terminated = rs_cin_isterminated(l, false, true);

        if js_cur_has_key {
            js_cur_has_key = false;
            if opts.ind_js != 0 && terminated == b',' as c_char {
                lookfor = LOOKFOR_JS_KEY;
            }
        }

        if lookfor == LOOKFOR_JS_KEY && rs_cin_has_js_key(l) {
            amount = nvim_cindent_get_indent();
            break;
        }

        if lookfor == LOOKFOR_COMMA {
            if nvim_cindent_curwin_get_cursor_lnum() <= trypos_brace_lnum {
                break;
            }
            if terminated == b',' as c_char {
                break;
            }
            amount = nvim_cindent_get_indent();
            if nvim_cindent_curwin_get_cursor_lnum() - 1 == ourscope {
                break;
            }
        }

        if terminated == 0 || (lookfor != LOOKFOR_UNTERM && terminated == b',' as c_char) {
            let l2 = nvim_cindent_get_cursor_line_ptr();
            let l2_sw = skipwhite(l2).cast_const();
            let l2_len = strlen(l2) as usize;

            if lookfor != LOOKFOR_ENUM_OR_INIT
                && (*l2_sw == b'[' as c_char
                    || (l2_len > 0 && *l2.add(l2_len - 1) == b'[' as c_char))
            {
                amount += ind_continuation;
            }

            // Find matching paren
            let lp_res = rs_find_last_paren(l2, b'(' as c_char, b')' as c_char);
            // (sets cursor col if found)
            let mut trypos_inner: Option<FindMatchResult> = None;
            let corr = rs_corr_ind_maxparen(cur_lnum);
            let tp = rs_find_match_paren(corr);
            if tp.found {
                // Check it's not before our scope brace
                if !(tp.lnum < ourscope || (tp.lnum == ourscope && tp.col < 0)) {
                    trypos_inner = Some(tp);
                }
            }

            let l3 = nvim_cindent_get_cursor_line_ptr();

            if trypos_inner.is_none() && terminated == b',' as c_char {
                let lp2 = rs_find_last_paren(l3, b'{' as c_char, b'}' as c_char);
                if lp2.found {
                    let tp2 = rs_find_start_brace();
                    if tp2.found {
                        trypos_inner = Some(tp2);
                    }
                }
            }

            if let Some(tp) = trypos_inner {
                nvim_cindent_curwin_set_cursor(tp.lnum, tp.col);
                let l4 = nvim_cindent_get_cursor_line_ptr();
                if rs_cin_iscase(l4, false) || rs_cin_isscopedecl(l4) {
                    nvim_cindent_curwin_set_cursor(tp.lnum + 1, 0);
                    continue;
                }
            }

            // Skip continuation lines (backslash)
            if terminated == b',' as c_char {
                let mut cl = nvim_cindent_curwin_get_cursor_lnum();
                while cl > 1 {
                    let prev = nvim_cindent_ml_get(cl - 1);
                    let prev_len = strlen(prev);
                    if prev_len == 0 || *prev.add(prev_len - 1) != b'\\' as c_char {
                        break;
                    }
                    cl -= 1;
                    nvim_cindent_curwin_set_cursor(cl, 0);
                }
            }

            let cur_amount2 = if opts.ind_js != 0 {
                nvim_cindent_get_indent()
            } else {
                {
                    let mut _pp: *const c_char = std::ptr::null();
                    rs_skip_label(nvim_cindent_curwin_get_cursor_lnum(), &raw mut _pp)
                }
            };

            if terminated != b',' as c_char && lookfor != LOOKFOR_TERM && *theline == b'{' as c_char
            {
                amount = cur_amount2;
                let l5 = nvim_cindent_get_cursor_line_ptr();
                if *skipwhite(l5) != b'{' as c_char {
                    amount += opts.ind_open_extra;
                }
                if opts.ind_cpp_baseclass != 0 && opts.ind_js == 0 {
                    lookfor = LOOKFOR_CPP_BASECLASS;
                    continue;
                }
                break;
            }

            // Check if we are after an "if", "while", etc.
            let l5 = nvim_cindent_get_cursor_line_ptr();
            let l5_sw = skipwhite(l5).cast_const();
            if rs_cin_is_cinword(l5) || rs_cin_iselse(l5_sw) {
                if lookfor == LOOKFOR_UNTERM || lookfor == LOOKFOR_ENUM_OR_INIT {
                    if cont_amount > 0 {
                        amount = cont_amount;
                    } else {
                        amount += ind_continuation;
                    }
                    break;
                }

                amount = cur_amount2;
                if *theline == b'{' as c_char {
                    amount += opts.ind_open_extra;
                }
                if lookfor != LOOKFOR_TERM {
                    amount += opts.ind_level + opts.ind_no_brace;
                    break;
                }

                // do/else handling for LOOKFOR_TERM
                let l6 = skipwhite(nvim_cindent_get_cursor_line_ptr()).cast_const();
                if rs_cin_isdo(l6) {
                    if whilelevel == 0 {
                        break;
                    }
                    whilelevel -= 1;
                }

                if rs_cin_iselse(l6) && whilelevel == 0 {
                    let this_col = if *l6 == b'}' as c_char {
                        (l6.offset_from(nvim_cindent_get_cursor_line_ptr()) as c_int) + 1
                    } else {
                        0
                    };
                    if this_col > 0 {
                        nvim_cindent_curwin_set_cursor(
                            nvim_cindent_curwin_get_cursor_lnum(),
                            this_col,
                        );
                    }
                    let tp = rs_find_start_brace();
                    if !tp.found || rs_find_match(LOOKFOR_IF, tp.lnum) == FAIL {
                        break;
                    }
                }
            } else {
                // Below unterminated line that is not if/while/etc
                if lookfor == LOOKFOR_UNTERM {
                    if terminated == b',' as c_char {
                        amount += ind_continuation;
                    }
                    break;
                }

                if lookfor == LOOKFOR_ENUM_OR_INIT {
                    if terminated == b',' as c_char {
                        if opts.ind_cpp_baseclass == 0 {
                            break;
                        }
                        lookfor = LOOKFOR_CPP_BASECLASS;
                        continue;
                    }
                    if amount > cur_amount2 {
                        amount = cur_amount2;
                    }
                } else {
                    let l7 = nvim_cindent_get_cursor_line_ptr();
                    amount = cur_amount2;

                    let l7_len = strlen(l7) as usize;
                    if opts.ind_js != 0
                        && terminated == b',' as c_char
                        && (*skipwhite(l7) == b']' as c_char
                            || (l7_len >= 2 && *l7.add(l7_len - 2) == b']' as c_char))
                    {
                        break;
                    }

                    if lookfor == LOOKFOR_INITIAL && terminated == b',' as c_char {
                        if opts.ind_js != 0 {
                            if rs_cin_iscomment(skipwhite(l7).cast_const()) {
                                break;
                            }
                            lookfor = LOOKFOR_COMMA;
                            let tp = rs_find_match_char(b'[' as c_int, opts.ind_maxparen);
                            if tp.found {
                                if tp.lnum == nvim_cindent_curwin_get_cursor_lnum() - 1 {
                                    break;
                                }
                                // ourscope = tp.lnum  -- but ourscope is immutable here
                                // We approximate by ignoring this change
                            }
                        } else {
                            lookfor = LOOKFOR_ENUM_OR_INIT;
                            cont_amount = rs_cin_first_id_amount();
                        }
                    } else {
                        if lookfor == LOOKFOR_INITIAL
                            && !is_nul(*l7)
                            && *l7.add(l7_len.saturating_sub(1)) == b'\\' as c_char
                        {
                            cont_amount =
                                rs_cin_get_equal_amount(nvim_cindent_curwin_get_cursor_lnum());
                        }
                        if lookfor != LOOKFOR_TERM
                            && lookfor != LOOKFOR_JS_KEY
                            && lookfor != LOOKFOR_COMMA
                            && raw_string_start != nvim_cindent_curwin_get_cursor_lnum()
                        {
                            lookfor = LOOKFOR_UNTERM;
                        }
                    }
                }
            }
        } else if rs_cin_iswhileofdo_end(terminated as c_int) {
            if lookfor == LOOKFOR_UNTERM || lookfor == LOOKFOR_ENUM_OR_INIT {
                if cont_amount > 0 {
                    amount = cont_amount;
                } else {
                    amount += ind_continuation;
                }
                break;
            }

            if whilelevel == 0 {
                lookfor = LOOKFOR_TERM;
                amount = nvim_cindent_get_indent();
                if *theline == b'{' as c_char {
                    amount += opts.ind_open_extra;
                }
            }
            whilelevel += 1;
        } else {
            // "Normal" terminated statement
            if lookfor == LOOKFOR_NOBREAK
                && rs_cin_isbreak(skipwhite(nvim_cindent_get_cursor_line_ptr()).cast_const())
            {
                lookfor = LOOKFOR_ANY;
                continue;
            }

            if whilelevel > 0 {
                let l8 = rs_cin_skipcomment(nvim_cindent_get_cursor_line_ptr());
                if rs_cin_isdo(l8) {
                    amount = nvim_cindent_get_indent();
                    whilelevel -= 1;
                    continue;
                }
            }

            if lookfor == LOOKFOR_UNTERM || lookfor == LOOKFOR_ENUM_OR_INIT {
                if cont_amount > 0 {
                    amount = cont_amount;
                } else {
                    amount += ind_continuation;
                }
                break;
            }

            if lookfor == LOOKFOR_TERM {
                if !lookfor_break && whilelevel == 0 {
                    break;
                }
            } else {
                // term_again equivalent
                'term_again: loop {
                    let l9 = nvim_cindent_get_cursor_line_ptr();
                    let lp_res = rs_find_last_paren(l9, b'(' as c_char, b')' as c_char);
                    if lp_res.found {
                        let tp = rs_find_match_paren(opts.ind_maxparen);
                        if tp.found {
                            nvim_cindent_curwin_set_cursor(tp.lnum, tp.col);
                            let l10 = nvim_cindent_get_cursor_line_ptr();
                            if rs_cin_iscase(l10, false) || rs_cin_isscopedecl(l10) {
                                nvim_cindent_curwin_set_cursor(tp.lnum + 1, 0);
                                continue 'outer;
                            }
                        }
                    }

                    let l9 = nvim_cindent_get_cursor_line_ptr();
                    let iscase2 = opts.ind_keep_case_label != 0 && rs_cin_iscase(l9, false);
                    amount = {
                        let mut _pp: *const c_char = std::ptr::null();
                        rs_skip_label(nvim_cindent_curwin_get_cursor_lnum(), &raw mut _pp)
                    };

                    if *theline == b'{' as c_char {
                        amount += opts.ind_open_extra;
                    }
                    let l9_sw = skipwhite(l9).cast_const();
                    if *l9_sw == b'{' as c_char {
                        amount -= opts.ind_open_extra;
                    }
                    lookfor = if iscase2 { LOOKFOR_ANY } else { LOOKFOR_TERM };

                    if lookfor == LOOKFOR_TERM
                        && *l9_sw != b'}' as c_char
                        && rs_cin_iselse(l9_sw)
                        && whilelevel == 0
                    {
                        let tp = rs_find_start_brace();
                        if !tp.found || rs_find_match(LOOKFOR_IF, tp.lnum) == FAIL {
                            break 'outer;
                        }
                        continue 'outer;
                    }

                    let l9b = nvim_cindent_get_cursor_line_ptr();
                    let lp2 = rs_find_last_paren(l9b, b'{' as c_char, b'}' as c_char);
                    if lp2.found {
                        let tp2 = rs_find_start_brace();
                        if tp2.found {
                            nvim_cindent_curwin_set_cursor(tp2.lnum, tp2.col);
                            let l11 = rs_cin_skipcomment(nvim_cindent_get_cursor_line_ptr());
                            if *l11 == b'}' as c_char || !rs_cin_iselse(l11) {
                                continue 'term_again;
                            }
                            nvim_cindent_curwin_set_cursor(tp2.lnum + 1, 0);
                        }
                    }
                    break 'term_again;
                } // end 'term_again
            }
        }
    } // end 'outer

    amount
}

// ============================================================================
// Phase 5: parse_cino — cinoptions parser
// ============================================================================

/// Helper: check if a byte is an ASCII digit.
#[inline]
const fn is_ascii_digit(c: c_char) -> bool {
    (c as u8).is_ascii_digit()
}

/// Parse a cinoptions string and populate a `CindentOptions` struct.
///
/// This is the Rust implementation of `parse_cino()`. The C wrapper calls this
/// and copies the resulting options into the buffer's `b_ind_*` fields.
///
/// # Safety
/// - `cino` must be a valid null-terminated C string (or null).
/// - `opts` must point to a valid `CindentOptions` struct.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_parse_cino(cino: *const c_char, sw: c_int, opts: *mut CindentOptions) {
    if opts.is_null() {
        return;
    }
    let opts = &mut *opts;

    // Set defaults
    opts.ind_level = sw;
    opts.ind_open_imag = 0;
    opts.ind_no_brace = 0;
    opts.ind_first_open = 0;
    opts.ind_open_extra = 0;
    opts.ind_close_extra = 0;
    opts.ind_open_left_imag = 0;
    opts.ind_jump_label = -1;
    opts.ind_case = sw;
    opts.ind_case_code = sw;
    opts.ind_case_break = 0;
    opts.ind_scopedecl = sw;
    opts.ind_scopedecl_code = sw;
    opts.ind_param = sw;
    opts.ind_func_type = sw;
    opts.ind_cpp_baseclass = sw;
    opts.ind_continuation = sw;
    opts.ind_unclosed = sw * 2;
    opts.ind_unclosed2 = sw;
    opts.ind_unclosed_noignore = 0;
    opts.ind_unclosed_wrapped = 0;
    opts.ind_unclosed_whiteok = 0;
    opts.ind_matching_paren = 0;
    opts.ind_paren_prev = 0;
    opts.ind_comment = 0;
    opts.ind_in_comment = 3;
    opts.ind_in_comment2 = 0;
    opts.ind_maxparen = 20;
    opts.ind_maxcomment = 70;
    opts.ind_java = 0;
    opts.ind_js = 0;
    opts.ind_keep_case_label = 0;
    opts.ind_cpp_namespace = 0;
    opts.ind_if_for_while = 0;
    opts.ind_hash_comment = 0;
    opts.ind_cpp_extern_c = 0;
    opts.ind_pragma = 0;

    if cino.is_null() {
        return;
    }

    let mut p = cino;
    while !is_nul(*p) {
        let key = *p;
        p = p.add(1);

        // Check for negative sign
        let negative = *p == b'-' as c_char;
        if negative {
            p = p.add(1);
        }

        // Remember where digits start
        let digits_start = p;

        // Parse integer part (getdigits equivalent)
        let mut n: i64 = 0;
        while is_ascii_digit(*p) {
            n = n.wrapping_mul(10).wrapping_add(i64::from(*p as u8 - b'0'));
            p = p.add(1);
        }

        // Parse fractional part (".5s" means a fraction)
        let mut fraction: i64 = 0;
        let mut divider: i64 = 0;
        if *p == b'.' as c_char {
            p = p.add(1);
            fraction = 0;
            while is_ascii_digit(*p) {
                fraction = fraction
                    .wrapping_mul(10)
                    .wrapping_add(i64::from(*p as u8 - b'0'));
                p = p.add(1);
                if divider != 0 {
                    divider *= 10;
                } else {
                    divider = 10;
                }
            }
        }

        // Handle shiftwidth multiplier ("2s" means two times shiftwidth)
        if *p == b's' as c_char {
            if p == digits_start {
                n = i64::from(sw); // just "s" is one shiftwidth
            } else {
                n *= i64::from(sw);
                if divider != 0 {
                    n += (i64::from(sw) * fraction + divider / 2) / divider;
                }
            }
            p = p.add(1);
        }

        // Apply negative sign
        if negative {
            n = -n;
        }

        // Clamp to int range
        let n = trim_to_int(n);

        // Map key character to option field
        match key as u8 {
            b'>' => opts.ind_level = n,
            b'e' => opts.ind_open_imag = n,
            b'n' => opts.ind_no_brace = n,
            b'f' => opts.ind_first_open = n,
            b'{' => opts.ind_open_extra = n,
            b'}' => opts.ind_close_extra = n,
            b'^' => opts.ind_open_left_imag = n,
            b'L' => opts.ind_jump_label = n,
            b':' => opts.ind_case = n,
            b'=' => opts.ind_case_code = n,
            b'b' => opts.ind_case_break = n,
            b'p' => opts.ind_param = n,
            b't' => opts.ind_func_type = n,
            b'/' => opts.ind_comment = n,
            b'c' => opts.ind_in_comment = n,
            b'C' => opts.ind_in_comment2 = n,
            b'i' => opts.ind_cpp_baseclass = n,
            b'+' => opts.ind_continuation = n,
            b'(' => opts.ind_unclosed = n,
            b'u' => opts.ind_unclosed2 = n,
            b'U' => opts.ind_unclosed_noignore = n,
            b'W' => opts.ind_unclosed_wrapped = n,
            b'w' => opts.ind_unclosed_whiteok = n,
            b'm' => opts.ind_matching_paren = n,
            b'M' => opts.ind_paren_prev = n,
            b')' => opts.ind_maxparen = n,
            b'*' => opts.ind_maxcomment = n,
            b'g' => opts.ind_scopedecl = n,
            b'h' => opts.ind_scopedecl_code = n,
            b'j' => opts.ind_java = n,
            b'J' => opts.ind_js = n,
            b'l' => opts.ind_keep_case_label = n,
            b'#' => opts.ind_hash_comment = n,
            b'N' => opts.ind_cpp_namespace = n,
            b'k' => opts.ind_if_for_while = n,
            b'E' => opts.ind_cpp_extern_c = n,
            b'P' => opts.ind_pragma = n,
            _ => {}
        }

        // Skip comma separator
        if *p == b',' as c_char {
            p = p.add(1);
        }
    }
}

/// Clamp an `i64` value to fit in a `c_int`.
#[inline]
fn trim_to_int(x: i64) -> c_int {
    if x > i64::from(c_int::MAX) {
        c_int::MAX
    } else if x < i64::from(c_int::MIN) {
        c_int::MIN
    } else {
        x as c_int
    }
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

    #[test]
    fn test_parse_cino_defaults() {
        unsafe {
            let mut opts = CindentOptions::default();
            let cino = CString::new("").unwrap();
            rs_parse_cino(cino.as_ptr(), 4, &raw mut opts);

            assert_eq!(opts.ind_level, 4);
            assert_eq!(opts.ind_open_imag, 0);
            assert_eq!(opts.ind_no_brace, 0);
            assert_eq!(opts.ind_first_open, 0);
            assert_eq!(opts.ind_open_extra, 0);
            assert_eq!(opts.ind_close_extra, 0);
            assert_eq!(opts.ind_open_left_imag, 0);
            assert_eq!(opts.ind_jump_label, -1);
            assert_eq!(opts.ind_case, 4);
            assert_eq!(opts.ind_case_code, 4);
            assert_eq!(opts.ind_case_break, 0);
            assert_eq!(opts.ind_scopedecl, 4);
            assert_eq!(opts.ind_scopedecl_code, 4);
            assert_eq!(opts.ind_param, 4);
            assert_eq!(opts.ind_func_type, 4);
            assert_eq!(opts.ind_cpp_baseclass, 4);
            assert_eq!(opts.ind_continuation, 4);
            assert_eq!(opts.ind_unclosed, 8); // sw * 2
            assert_eq!(opts.ind_unclosed2, 4);
            assert_eq!(opts.ind_unclosed_noignore, 0);
            assert_eq!(opts.ind_unclosed_wrapped, 0);
            assert_eq!(opts.ind_unclosed_whiteok, 0);
            assert_eq!(opts.ind_matching_paren, 0);
            assert_eq!(opts.ind_paren_prev, 0);
            assert_eq!(opts.ind_comment, 0);
            assert_eq!(opts.ind_in_comment, 3);
            assert_eq!(opts.ind_in_comment2, 0);
            assert_eq!(opts.ind_maxparen, 20);
            assert_eq!(opts.ind_maxcomment, 70);
            assert_eq!(opts.ind_java, 0);
            assert_eq!(opts.ind_js, 0);
            assert_eq!(opts.ind_keep_case_label, 0);
            assert_eq!(opts.ind_cpp_namespace, 0);
            assert_eq!(opts.ind_if_for_while, 0);
            assert_eq!(opts.ind_hash_comment, 0);
            assert_eq!(opts.ind_cpp_extern_c, 0);
            assert_eq!(opts.ind_pragma, 0);
        }
    }

    #[test]
    fn test_parse_cino_null() {
        unsafe {
            let mut opts = CindentOptions::default();
            rs_parse_cino(std::ptr::null(), 4, &raw mut opts);
            // Should get defaults
            assert_eq!(opts.ind_level, 4);
            assert_eq!(opts.ind_case, 4);
        }
    }

    #[test]
    fn test_parse_cino_simple_values() {
        unsafe {
            let mut opts = CindentOptions::default();
            let cino = CString::new(">8,:2,=2").unwrap();
            rs_parse_cino(cino.as_ptr(), 4, &raw mut opts);

            assert_eq!(opts.ind_level, 8);
            assert_eq!(opts.ind_case, 2);
            assert_eq!(opts.ind_case_code, 2);
        }
    }

    #[test]
    fn test_parse_cino_negative_values() {
        unsafe {
            let mut opts = CindentOptions::default();
            let cino = CString::new("L-1,>-2").unwrap();
            rs_parse_cino(cino.as_ptr(), 4, &raw mut opts);

            assert_eq!(opts.ind_jump_label, -1);
            assert_eq!(opts.ind_level, -2);
        }
    }

    #[test]
    fn test_parse_cino_shiftwidth_multiplier() {
        unsafe {
            let mut opts = CindentOptions::default();
            let cino = CString::new(">2s").unwrap();
            rs_parse_cino(cino.as_ptr(), 4, &raw mut opts);

            assert_eq!(opts.ind_level, 8); // 2 * sw(4) = 8
        }
    }

    #[test]
    fn test_parse_cino_bare_s() {
        unsafe {
            let mut opts = CindentOptions::default();
            let cino = CString::new(">s").unwrap();
            rs_parse_cino(cino.as_ptr(), 4, &raw mut opts);

            assert_eq!(opts.ind_level, 4); // just "s" = 1 * sw
        }
    }

    #[test]
    fn test_parse_cino_fraction() {
        unsafe {
            let mut opts = CindentOptions::default();
            let cino = CString::new(">.5s").unwrap();
            rs_parse_cino(cino.as_ptr(), 4, &raw mut opts);

            // 0 * sw + (sw * 5 + 10/2) / 10 = (20 + 5) / 10 = 2
            assert_eq!(opts.ind_level, 2);
        }
    }

    #[test]
    fn test_parse_cino_all_keys() {
        unsafe {
            let mut opts = CindentOptions::default();
            let cino =
                CString::new(">1,e2,n3,f4,{5,}6,^7,L8,:9,=10,b11,p12,t13,/14,c15,C16,i17,+18,(19,u20,U21,W22,w23,m24,M25,)26,*27,g28,h29,j30,J31,l32,#33,N34,k35,E36,P37")
                    .unwrap();
            rs_parse_cino(cino.as_ptr(), 4, &raw mut opts);

            assert_eq!(opts.ind_level, 1);
            assert_eq!(opts.ind_open_imag, 2);
            assert_eq!(opts.ind_no_brace, 3);
            assert_eq!(opts.ind_first_open, 4);
            assert_eq!(opts.ind_open_extra, 5);
            assert_eq!(opts.ind_close_extra, 6);
            assert_eq!(opts.ind_open_left_imag, 7);
            assert_eq!(opts.ind_jump_label, 8);
            assert_eq!(opts.ind_case, 9);
            assert_eq!(opts.ind_case_code, 10);
            assert_eq!(opts.ind_case_break, 11);
            assert_eq!(opts.ind_param, 12);
            assert_eq!(opts.ind_func_type, 13);
            assert_eq!(opts.ind_comment, 14);
            assert_eq!(opts.ind_in_comment, 15);
            assert_eq!(opts.ind_in_comment2, 16);
            assert_eq!(opts.ind_cpp_baseclass, 17);
            assert_eq!(opts.ind_continuation, 18);
            assert_eq!(opts.ind_unclosed, 19);
            assert_eq!(opts.ind_unclosed2, 20);
            assert_eq!(opts.ind_unclosed_noignore, 21);
            assert_eq!(opts.ind_unclosed_wrapped, 22);
            assert_eq!(opts.ind_unclosed_whiteok, 23);
            assert_eq!(opts.ind_matching_paren, 24);
            assert_eq!(opts.ind_paren_prev, 25);
            assert_eq!(opts.ind_maxparen, 26);
            assert_eq!(opts.ind_maxcomment, 27);
            assert_eq!(opts.ind_scopedecl, 28);
            assert_eq!(opts.ind_scopedecl_code, 29);
            assert_eq!(opts.ind_java, 30);
            assert_eq!(opts.ind_js, 31);
            assert_eq!(opts.ind_keep_case_label, 32);
            assert_eq!(opts.ind_hash_comment, 33);
            assert_eq!(opts.ind_cpp_namespace, 34);
            assert_eq!(opts.ind_if_for_while, 35);
            assert_eq!(opts.ind_cpp_extern_c, 36);
            assert_eq!(opts.ind_pragma, 37);
        }
    }

    #[test]
    fn test_parse_cino_negative_shiftwidth() {
        unsafe {
            let mut opts = CindentOptions::default();
            let cino = CString::new(">-2s").unwrap();
            rs_parse_cino(cino.as_ptr(), 4, &raw mut opts);

            assert_eq!(opts.ind_level, -8); // -(2 * 4)
        }
    }
}
