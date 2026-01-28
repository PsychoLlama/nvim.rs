//! File pattern matching for autocommands
//!
//! This module provides Rust implementations for converting shell-style glob
//! patterns to regular expressions, as used in autocommand file patterns.
//!
//! The conversion follows Vim's glob pattern rules:
//! - `*` matches any characters (converts to `.*`)
//! - `?` matches a single character (converts to `.`)
//! - `{a,b}` alternation (converts to `\(a\|b\)`)
//! - `.` and `~` are escaped (converts to `\.` and `\~`)
//! - Leading/trailing `*` affect anchoring

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::similar_names)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::too_many_lines)]

use std::cmp::Ordering;
use std::ffi::{c_char, c_int};

// =============================================================================
// Pattern Conversion Result
// =============================================================================

/// Result of converting a glob pattern to a regex pattern.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PatternConvertResult {
    /// Whether the conversion succeeded
    pub success: c_int,
    /// Whether the pattern contains path separators (can match directories)
    pub allow_dirs: c_int,
    /// Error code (0 = success, 1 = unmatched '{', 2 = unmatched '}')
    pub error: c_int,
}

impl Default for PatternConvertResult {
    fn default() -> Self {
        Self {
            success: 1,
            allow_dirs: 0,
            error: 0,
        }
    }
}

// =============================================================================
// Glob to Regex Conversion
// =============================================================================

/// Convert a glob pattern to a regex pattern string.
///
/// This function implements Vim's `file_pat_to_reg_pat()` algorithm:
/// - `*` → `.*`
/// - `?` → `.`
/// - `{a,b}` → `\(a\|b\)`
/// - `.` and `~` → `\.` and `\~`
/// - Leading `*` removes `^` anchor
/// - Trailing `*` removes `$` anchor
///
/// # Arguments
/// * `pattern` - The glob pattern bytes
///
/// # Returns
/// A tuple of (regex_string, allow_dirs, error_code)
/// where error_code is: 0 = success, 1 = unmatched '{', 2 = unmatched '}'
pub fn glob_to_regex(pattern: &[u8]) -> (String, bool, i32) {
    if pattern.is_empty() {
        return ("^$".to_string(), false, 0);
    }

    let mut result = String::with_capacity(pattern.len() * 2 + 2);
    let mut allow_dirs = false;
    let mut nested: i32 = 0;

    // Skip leading '*' characters and don't add '^' anchor if pattern starts with '*'
    let mut start = 0;
    while start < pattern.len() && pattern[start] == b'*' {
        start += 1;
    }
    if start == 0 {
        result.push('^');
    }

    // Find effective end (skip trailing '*' characters)
    let mut end = pattern.len();
    while end > start && pattern[end - 1] == b'*' {
        end -= 1;
    }
    let add_dollar = end == pattern.len();

    // If we skipped all characters, handle edge case
    if start > 0 && start >= pattern.len() {
        return (".*".to_string(), false, 0);
    }

    let mut i = 0;
    let slice = &pattern[..end.max(1)];

    while i < slice.len() {
        let c = slice[i];
        match c {
            b'*' => {
                // Skip consecutive '*' characters
                result.push('.');
                result.push('*');
                while i + 1 < slice.len() && slice[i + 1] == b'*' {
                    i += 1;
                }
            }
            b'.' | b'~' => {
                result.push('\\');
                result.push(c as char);
            }
            b'?' => {
                result.push('.');
            }
            b'\\' => {
                // Handle escapes
                if i + 1 < slice.len() {
                    let next = slice[i + 1];
                    match next {
                        // Unescape these characters
                        b',' | b'%' | b'#' | b' ' | b'\t' | b'{' | b'}' => {
                            result.push(next as char);
                            i += 1;
                        }
                        // Check for \\\{ -> \{
                        b'\\' if i + 3 < slice.len() && slice[i + 2] == b'\\' && slice[i + 3] == b'{' => {
                            result.push('\\');
                            result.push('{');
                            i += 3;
                        }
                        // Path separator - set allow_dirs
                        b'/' => {
                            allow_dirs = true;
                            result.push('\\');
                            result.push('/');
                            i += 1;
                        }
                        _ => {
                            // Keep the backslash and next char
                            result.push('\\');
                            result.push(next as char);
                            i += 1;
                        }
                    }
                }
            }
            b'/' => {
                allow_dirs = true;
                result.push('/');
            }
            b'{' => {
                result.push('\\');
                result.push('(');
                nested += 1;
            }
            b'}' => {
                result.push('\\');
                result.push(')');
                nested -= 1;
            }
            b',' => {
                if nested > 0 {
                    result.push('\\');
                    result.push('|');
                } else {
                    result.push(',');
                }
            }
            _ => {
                result.push(c as char);
            }
        }
        i += 1;
    }

    if add_dollar {
        result.push('$');
    }

    // Check for unmatched braces
    let error = match nested.cmp(&0) {
        Ordering::Greater => 1, // Missing '}'
        Ordering::Less => 2,    // Missing '{'
        Ordering::Equal => 0,
    };

    (result, allow_dirs, error)
}


// =============================================================================
// FFI Exports
// =============================================================================

/// Convert a glob pattern to a regex pattern.
///
/// # Safety
/// `pat` must be a valid pointer to at least `patlen` bytes.
/// `out_buf` must be a valid pointer to at least `out_buf_size` bytes.
/// `result` must be a valid pointer to a PatternConvertResult.
///
/// Returns the length of the output regex pattern, or 0 on error.
/// If the buffer is too small, returns the required size without writing.
#[no_mangle]
pub unsafe extern "C" fn rs_glob_to_regex(
    pat: *const c_char,
    patlen: usize,
    out_buf: *mut c_char,
    out_buf_size: usize,
    result: *mut PatternConvertResult,
) -> usize {
    if pat.is_null() || result.is_null() {
        return 0;
    }

    // Convert input to slice
    let pattern = std::slice::from_raw_parts(pat.cast::<u8>(), patlen);

    // Convert glob to regex
    let (regex, allow_dirs, error) = glob_to_regex(pattern);

    // Fill result
    (*result).success = c_int::from(error == 0);
    (*result).allow_dirs = c_int::from(allow_dirs);
    (*result).error = error;

    let regex_bytes = regex.as_bytes();
    let regex_len = regex_bytes.len();

    // If buffer provided and large enough, copy the result
    if !out_buf.is_null() && out_buf_size > regex_len {
        std::ptr::copy_nonoverlapping(regex_bytes.as_ptr(), out_buf.cast::<u8>(), regex_len);
        *out_buf.add(regex_len) = 0; // NUL terminate
    }

    regex_len
}

/// Check if a filename matches a glob pattern.
///
/// This implements simplified glob matching without requiring the full regex
/// engine. Uses Rust's pattern matching capabilities.
///
/// # Safety
/// `pattern` and `filename` must be valid NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_glob_match_simple(
    pattern: *const c_char,
    patlen: usize,
    filename: *const c_char,
    filelen: usize,
    ignore_case: c_int,
) -> c_int {
    if pattern.is_null() || filename.is_null() {
        return 0;
    }

    let pat = std::slice::from_raw_parts(pattern.cast::<u8>(), patlen);
    let name = std::slice::from_raw_parts(filename.cast::<u8>(), filelen);

    c_int::from(glob_match(pat, name, ignore_case != 0))
}

/// Simple glob matching implementation.
///
/// Supports:
/// - `*` matches any sequence of characters
/// - `?` matches any single character
/// - Other characters match literally
fn glob_match(pattern: &[u8], text: &[u8], ignore_case: bool) -> bool {
    let mut pi = 0; // pattern index
    let mut ti = 0; // text index
    let mut star_pi = None; // position after last '*' in pattern
    let mut star_ti = 0; // position in text when we saw last '*'

    while ti < text.len() {
        if pi < pattern.len() {
            match pattern[pi] {
                b'*' => {
                    // Remember this position for backtracking
                    star_pi = Some(pi + 1);
                    star_ti = ti;
                    pi += 1;
                    continue;
                }
                b'?' => {
                    // Match any single character
                    pi += 1;
                    ti += 1;
                    continue;
                }
                c => {
                    let tc = text[ti];
                    let matches = if ignore_case {
                        c.eq_ignore_ascii_case(&tc)
                    } else {
                        c == tc
                    };
                    if matches {
                        pi += 1;
                        ti += 1;
                        continue;
                    }
                }
            }
        }

        // No match at current position - try backtracking to last '*'
        if let Some(spi) = star_pi {
            pi = spi;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }

    // Check remaining pattern is all '*'
    while pi < pattern.len() && pattern[pi] == b'*' {
        pi += 1;
    }

    pi == pattern.len()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_to_regex_simple() {
        // Simple pattern
        let (regex, allow_dirs, error) = glob_to_regex(b"*.c");
        assert_eq!(error, 0);
        assert!(!allow_dirs);
        assert!(regex.contains(".*"));
        assert!(regex.ends_with("c$"));
    }

    #[test]
    fn test_glob_to_regex_question() {
        let (regex, _, error) = glob_to_regex(b"file?.txt");
        assert_eq!(error, 0);
        assert!(regex.contains('.')); // ? becomes .
    }

    #[test]
    fn test_glob_to_regex_braces() {
        let (regex, _, error) = glob_to_regex(b"*.{c,h}");
        assert_eq!(error, 0);
        assert!(regex.contains("\\("));
        assert!(regex.contains("\\|"));
        assert!(regex.contains("\\)"));
    }

    #[test]
    fn test_glob_to_regex_unmatched_brace() {
        let (_, _, error) = glob_to_regex(b"*.{c,h");
        assert_eq!(error, 1); // Missing '}'

        let (_, _, error) = glob_to_regex(b"*.c,h}");
        assert_eq!(error, 2); // Missing '{'
    }

    #[test]
    fn test_glob_to_regex_path() {
        let (_, allow_dirs, error) = glob_to_regex(b"src/*.c");
        assert_eq!(error, 0);
        assert!(allow_dirs);
    }

    #[test]
    fn test_glob_to_regex_escaped() {
        let (_regex, _, error) = glob_to_regex(b"file\\.txt");
        assert_eq!(error, 0);
        // The backslash-dot should remain as literal
    }

    #[test]
    fn test_glob_to_regex_empty() {
        let (regex, _, error) = glob_to_regex(b"");
        assert_eq!(error, 0);
        assert_eq!(regex, "^$");
    }

    #[test]
    fn test_glob_match_simple() {
        assert!(glob_match(b"*.c", b"file.c", false));
        assert!(glob_match(b"*.c", b"path/to/file.c", false));
        assert!(!glob_match(b"*.c", b"file.h", false));
    }

    #[test]
    fn test_glob_match_question() {
        assert!(glob_match(b"file?.txt", b"file1.txt", false));
        assert!(glob_match(b"file?.txt", b"fileX.txt", false));
        assert!(!glob_match(b"file?.txt", b"file12.txt", false));
    }

    #[test]
    fn test_glob_match_multiple_stars() {
        assert!(glob_match(b"*.c*", b"file.c", false));
        assert!(glob_match(b"*.c*", b"file.cpp", false));
        assert!(glob_match(b"*/*", b"dir/file", false));
    }

    #[test]
    fn test_glob_match_case_insensitive() {
        assert!(glob_match(b"*.C", b"file.c", true));
        assert!(glob_match(b"FILE.c", b"file.C", true));
        assert!(!glob_match(b"*.C", b"file.c", false));
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match(b"Makefile", b"Makefile", false));
        assert!(!glob_match(b"Makefile", b"makefile", false));
        assert!(glob_match(b"Makefile", b"makefile", true));
    }
}
