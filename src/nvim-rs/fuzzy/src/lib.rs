//! Fuzzy matching algorithm for Neovim
//!
//! This module provides fuzzy string matching with scoring and position tracking.
//! The algorithm is ported from fzy (<https://github.com/jhawthorn/fzy>) with
//! extensions for multibyte/Unicode support.
//!
//! # License
//!
//! Portions adapted from fzy:
//!   Copyright (c) 2014 John Hawthorn
//!   Licensed under the MIT License.

#![allow(unsafe_code)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::similar_names)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::manual_let_else)]

use libc::{c_char, c_int};
use std::ffi::CStr;

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of characters that can be matched
pub const MATCH_MAX_LEN: usize = 1024;

/// Score for no match (sentinel value)
pub const SCORE_NONE: i32 = i32::MIN;

/// Scaling factor for converting f64 scores to i32
const SCORE_SCALE: f64 = 1000.0;

// Scoring constants from fzy
const SCORE_GAP_LEADING: f64 = -0.005;
const SCORE_GAP_TRAILING: f64 = -0.005;
const SCORE_GAP_INNER: f64 = -0.01;
const SCORE_MATCH_CONSECUTIVE: f64 = 1.0;
const SCORE_MATCH_SLASH: f64 = 0.9;
const SCORE_MATCH_WORD: f64 = 0.8;
const SCORE_MATCH_CAPITAL: f64 = 0.7;
const SCORE_MATCH_DOT: f64 = 0.6;

// =============================================================================
// Character Classification Helpers
// =============================================================================

/// Check if character is a word separator (for bonus calculation)
#[inline]
const fn is_word_sep(c: char) -> bool {
    c == '-' || c == '_' || c == ' '
}

/// Check if character is a path separator
#[inline]
const fn is_path_sep(c: char) -> bool {
    c == '/'
}

/// Check if character is a dot
#[inline]
const fn is_dot(c: char) -> bool {
    c == '.'
}

/// Check if character is alphanumeric or a word character
#[inline]
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

// =============================================================================
// Core Algorithm Types
// =============================================================================

/// Internal state for the matching algorithm
struct MatchState {
    needle_len: usize,
    haystack_len: usize,
    lower_needle: [char; MATCH_MAX_LEN],
    lower_haystack: [char; MATCH_MAX_LEN],
    match_bonus: [f64; MATCH_MAX_LEN],
}

impl MatchState {
    /// Initialize match state from needle and haystack strings
    fn new(needle: &str, haystack: &str) -> Self {
        let mut state = Self {
            needle_len: 0,
            haystack_len: 0,
            lower_needle: ['\0'; MATCH_MAX_LEN],
            lower_haystack: ['\0'; MATCH_MAX_LEN],
            match_bonus: [0.0; MATCH_MAX_LEN],
        };

        // Process needle - convert to lowercase
        for c in needle.chars().take(MATCH_MAX_LEN) {
            state.lower_needle[state.needle_len] = c.to_lowercase().next().unwrap_or(c);
            state.needle_len += 1;
        }

        // Process haystack - convert to lowercase and compute bonuses
        let mut prev_c = '/'; // Treat start as after path separator
        for c in haystack.chars().take(MATCH_MAX_LEN) {
            state.lower_haystack[state.haystack_len] = c.to_lowercase().next().unwrap_or(c);
            state.match_bonus[state.haystack_len] = compute_bonus(prev_c, c);
            prev_c = c;
            state.haystack_len += 1;
        }

        state
    }
}

/// Compute the bonus score for a character based on its context
fn compute_bonus(last_c: char, c: char) -> f64 {
    if is_word_char(c) {
        if is_path_sep(last_c) {
            SCORE_MATCH_SLASH
        } else if is_word_sep(last_c) {
            SCORE_MATCH_WORD
        } else if is_dot(last_c) {
            SCORE_MATCH_DOT
        } else if c.is_uppercase() && last_c.is_lowercase() {
            SCORE_MATCH_CAPITAL
        } else {
            0.0
        }
    } else {
        0.0
    }
}

// =============================================================================
// Core Algorithm Implementation
// =============================================================================

/// Quick check if needle characters exist in haystack (in order)
#[must_use]
pub fn has_match(needle: &str, haystack: &str) -> bool {
    if needle.is_empty() {
        return false;
    }

    let mut needle_chars = needle.chars();
    let mut current_needle = match needle_chars.next() {
        Some(c) => c.to_lowercase().next().unwrap_or(c),
        None => return false,
    };

    for h_char in haystack.chars() {
        let h_lower = h_char.to_lowercase().next().unwrap_or(h_char);
        let n_upper = current_needle
            .to_uppercase()
            .next()
            .unwrap_or(current_needle);

        // Match if lowercase matches or uppercase of needle matches original
        if current_needle == h_lower || n_upper == h_char {
            current_needle = match needle_chars.next() {
                Some(c) => c.to_lowercase().next().unwrap_or(c),
                None => return true, // All needle chars matched
            };
        }
    }

    false
}

/// Compute a single row of the DP matrix
fn match_row(
    state: &MatchState,
    row: usize,
    curr_d: &mut [f64],
    curr_m: &mut [f64],
    last_d: &[f64],
    last_m: &[f64],
) {
    let n = state.needle_len;
    let m = state.haystack_len;
    let i = row;

    let gap_score = if i == n - 1 {
        SCORE_GAP_TRAILING
    } else {
        SCORE_GAP_INNER
    };

    let mut prev_score = f64::NEG_INFINITY;
    let mut prev_d = f64::NEG_INFINITY;
    let mut prev_m = f64::NEG_INFINITY;

    for j in 0..m {
        if state.lower_needle[i] == state.lower_haystack[j] {
            let score = if i == 0 {
                // First needle char - gap penalty for leading chars
                (j as f64).mul_add(SCORE_GAP_LEADING, state.match_bonus[j])
            } else if j > 0 {
                // Subsequent chars - best of: prev match + bonus OR consecutive match
                f64::max(
                    prev_m + state.match_bonus[j],
                    prev_d + SCORE_MATCH_CONSECUTIVE,
                )
            } else {
                f64::NEG_INFINITY
            };

            prev_d = last_d[j];
            prev_m = last_m[j];
            curr_d[j] = score;
            curr_m[j] = f64::max(score, prev_score + gap_score);
            prev_score = curr_m[j];
        } else {
            prev_d = last_d[j];
            prev_m = last_m[j];
            curr_d[j] = f64::NEG_INFINITY;
            curr_m[j] = prev_score + gap_score;
            prev_score = curr_m[j];
        }
    }
}

/// Compute fuzzy match score and optionally return match positions
///
/// Returns the score as f64, or `f64::NEG_INFINITY` if no match.
/// If `positions` is provided, fills it with the matched character indices.
#[must_use]
pub fn match_positions(needle: &str, haystack: &str, positions: Option<&mut [u32]>) -> f64 {
    if needle.is_empty() || haystack.is_empty() {
        return f64::NEG_INFINITY;
    }

    let state = MatchState::new(needle, haystack);
    let n = state.needle_len;
    let m = state.haystack_len;

    if m > MATCH_MAX_LEN || n > m {
        return f64::NEG_INFINITY;
    }

    // Handle exact match (same length means same chars ignoring case)
    if n == m {
        if let Some(pos) = positions {
            for (i, p) in pos.iter_mut().enumerate().take(n) {
                *p = i as u32;
            }
        }
        return f64::INFINITY;
    }

    // Allocate DP matrices
    // D[i][j] = best score ending with a match at needle[i], haystack[j]
    // M[i][j] = best possible score at needle[i], haystack[j]
    let mut d_matrix = vec![vec![f64::NEG_INFINITY; m]; n];
    let mut m_matrix = vec![vec![f64::NEG_INFINITY; m]; n];

    // Compute first row (use dummy slices for last_d/last_m since they're not used)
    let dummy = vec![f64::NEG_INFINITY; m];
    match_row(
        &state,
        0,
        &mut d_matrix[0],
        &mut m_matrix[0],
        &dummy,
        &dummy,
    );

    // Compute remaining rows
    for i in 1..n {
        // Clone previous row for reading while writing current row
        let prev_d = d_matrix[i - 1].clone();
        let prev_m = m_matrix[i - 1].clone();
        match_row(
            &state,
            i,
            &mut d_matrix[i],
            &mut m_matrix[i],
            &prev_d,
            &prev_m,
        );
    }

    // Backtrack to find positions
    if let Some(pos) = positions {
        let mut match_required = false;
        let mut j = m - 1;

        for i in (0..n).rev() {
            loop {
                if d_matrix[i][j] != f64::NEG_INFINITY
                    && (match_required || (d_matrix[i][j] - m_matrix[i][j]).abs() < f64::EPSILON)
                {
                    // Check if this match was consecutive (requires previous match)
                    match_required = i > 0
                        && j > 0
                        && (m_matrix[i][j] - (d_matrix[i - 1][j - 1] + SCORE_MATCH_CONSECUTIVE))
                            .abs()
                            < f64::EPSILON;
                    pos[i] = j as u32;
                    j = j.saturating_sub(1);
                    break;
                }
                if j == 0 {
                    break;
                }
                j -= 1;
            }
        }
    }

    m_matrix[n - 1][m - 1]
}

/// Simple fuzzy match - returns only the score
#[must_use]
pub fn fuzzy_score(needle: &str, haystack: &str) -> Option<i32> {
    if !has_match(needle, haystack) {
        return None;
    }

    let score = match_positions(needle, haystack, None);

    if score == f64::NEG_INFINITY {
        None
    } else if score == f64::INFINITY {
        Some(i32::MAX)
    } else if score < 0.0 {
        Some(score.mul_add(SCORE_SCALE, -0.5).ceil() as i32)
    } else {
        Some(score.mul_add(SCORE_SCALE, 0.5).floor() as i32)
    }
}

// =============================================================================
// FFI Interface
// =============================================================================

/// Convert C string pointer to Rust &str, returning None for null or invalid UTF-8
unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}

/// Fuzzy match with multi-word support (FFI version)
///
/// When `matchseq` is false, the pattern is split by whitespace and each word
/// must match in sequence.
///
/// # Safety
///
/// `str_ptr` and `pat_ptr` must be valid null-terminated C strings.
/// `out_score` and `matches` must be valid pointers if non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_fuzzy_match(
    str_ptr: *const c_char,
    pat_ptr: *const c_char,
    matchseq: bool,
    out_score: *mut c_int,
    matches: *mut u32,
    max_matches: c_int,
) -> bool {
    let haystack = if let Some(s) = cstr_to_str(str_ptr) {
        s
    } else {
        if !out_score.is_null() {
            *out_score = SCORE_NONE;
        }
        return false;
    };

    let pattern = if let Some(s) = cstr_to_str(pat_ptr) {
        s
    } else {
        if !out_score.is_null() {
            *out_score = SCORE_NONE;
        }
        return false;
    };

    if !out_score.is_null() {
        *out_score = 0;
    }

    let mut total_score: i64 = 0;
    let mut num_matches: usize = 0;
    let max_matches = max_matches as usize;

    // Process pattern - either as sequence or split by whitespace
    let words: Vec<&str> = if matchseq {
        vec![pattern]
    } else {
        pattern.split_whitespace().collect()
    };

    for word in words {
        if word.is_empty() {
            continue;
        }

        if !has_match(word, haystack) {
            if !out_score.is_null() {
                *out_score = SCORE_NONE;
            }
            return false;
        }

        // Get positions if requested
        let score = if !matches.is_null() && num_matches < max_matches {
            let remaining = max_matches - num_matches;
            let positions = std::slice::from_raw_parts_mut(
                matches.add(num_matches),
                remaining.min(word.chars().count()),
            );
            match_positions(word, haystack, Some(positions))
        } else {
            match_positions(word, haystack, None)
        };

        if score == f64::NEG_INFINITY {
            if !out_score.is_null() {
                *out_score = SCORE_NONE;
            }
            return false;
        }

        // Convert score to i32 and accumulate
        let int_score = if score == f64::INFINITY {
            i32::MAX
        } else if score < 0.0 {
            score.mul_add(SCORE_SCALE, -0.5).ceil() as i32
        } else {
            score.mul_add(SCORE_SCALE, 0.5).floor() as i32
        };

        // Saturating addition
        total_score = total_score.saturating_add(i64::from(int_score));

        num_matches += word.chars().count();

        if matchseq || num_matches >= max_matches {
            break;
        }
    }

    if !out_score.is_null() {
        *out_score = total_score.clamp(i64::from(i32::MIN) + 1, i64::from(i32::MAX)) as c_int;
    }

    num_matches > 0
}

/// Simple fuzzy match - returns score only (FFI version)
///
/// # Safety
///
/// `str_ptr` and `pat_ptr` must be valid null-terminated C strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_fuzzy_match_str(
    str_ptr: *const c_char,
    pat_ptr: *const c_char,
) -> c_int {
    let haystack = match cstr_to_str(str_ptr) {
        Some(s) => s,
        None => return SCORE_NONE,
    };

    let pattern = match cstr_to_str(pat_ptr) {
        Some(s) => s,
        None => return SCORE_NONE,
    };

    fuzzy_score(pattern, haystack).unwrap_or(SCORE_NONE)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::borrow_as_ptr)]
mod tests {
    use super::*;

    #[test]
    fn test_has_match_basic() {
        assert!(has_match("abc", "abc"));
        assert!(has_match("abc", "aXbXc"));
        assert!(has_match("abc", "ABC"));
        assert!(has_match("ABC", "abc"));
        assert!(!has_match("abc", "ab"));
        assert!(!has_match("abc", "acb"));
        assert!(!has_match("", "abc"));
    }

    #[test]
    fn test_has_match_unicode() {
        assert!(has_match("über", "ÜBER"));
        assert!(has_match("日本", "日本語"));
    }

    #[test]
    fn test_fuzzy_score_basic() {
        // Exact match should give high score
        assert!(fuzzy_score("abc", "abc").is_some());
        let exact = fuzzy_score("abc", "abc").unwrap();
        assert_eq!(exact, i32::MAX); // Exact match returns max

        // Substring match should give positive score
        let score = fuzzy_score("abc", "xabc").unwrap();
        assert!(score > 0);

        // No match should return None
        assert!(fuzzy_score("xyz", "abc").is_none());
    }

    #[test]
    fn test_fuzzy_score_word_boundary() {
        // Word boundary should score higher
        let boundary = fuzzy_score("fb", "foo_bar").unwrap();
        let no_boundary = fuzzy_score("fb", "fxxxb").unwrap();
        assert!(boundary > no_boundary);
    }

    #[test]
    fn test_fuzzy_score_consecutive() {
        // Consecutive matches should score higher
        let consecutive = fuzzy_score("abc", "xabcx").unwrap();
        let scattered = fuzzy_score("abc", "xaxbxcx").unwrap();
        assert!(consecutive > scattered);
    }

    #[test]
    fn test_match_positions() {
        let mut positions = [0u32; 3];
        let score = match_positions("abc", "xxaxxbxxc", Some(&mut positions));
        assert!(score > f64::NEG_INFINITY);
        assert_eq!(positions[0], 2); // 'a' at index 2
        assert_eq!(positions[1], 5); // 'b' at index 5
        assert_eq!(positions[2], 8); // 'c' at index 8
    }

    #[test]
    fn test_ffi_fuzzy_match() {
        use std::ffi::CString;

        let haystack = CString::new("foo_bar_baz").unwrap();
        let pattern = CString::new("fbb").unwrap();

        unsafe {
            let mut score: c_int = 0;
            let mut positions = [0u32; 10];

            let result = rs_fuzzy_match(
                haystack.as_ptr(),
                pattern.as_ptr(),
                true, // matchseq
                &mut score,
                positions.as_mut_ptr(),
                10,
            );

            assert!(result);
            assert!(score > 0);
            assert_eq!(positions[0], 0); // 'f' at index 0
            assert_eq!(positions[1], 4); // 'b' at index 4
            assert_eq!(positions[2], 8); // 'b' at index 8
        }
    }

    #[test]
    fn test_ffi_fuzzy_match_str() {
        use std::ffi::CString;

        let haystack = CString::new("hello world").unwrap();
        let pattern = CString::new("hwd").unwrap();

        unsafe {
            let score = rs_fuzzy_match_str(haystack.as_ptr(), pattern.as_ptr());
            assert!(score > SCORE_NONE);
        }
    }

    #[test]
    fn test_ffi_null_handling() {
        use std::ptr;

        unsafe {
            let mut score: c_int = 0;

            // Null haystack
            let result = rs_fuzzy_match(
                ptr::null(),
                ptr::null(),
                true,
                &mut score,
                ptr::null_mut(),
                0,
            );
            assert!(!result);
            assert_eq!(score, SCORE_NONE);

            // Null pattern
            let haystack = std::ffi::CString::new("test").unwrap();
            let result = rs_fuzzy_match(
                haystack.as_ptr(),
                ptr::null(),
                true,
                &mut score,
                ptr::null_mut(),
                0,
            );
            assert!(!result);
        }
    }

    #[test]
    fn test_fuzzy_constants() {
        // Verify MATCH_MAX_LEN and SCORE_NONE constants
        assert_eq!(MATCH_MAX_LEN, 1024);
        assert_eq!(SCORE_NONE, i32::MIN);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_score_constants() {
        // Verify scoring constants have expected values
        assert_eq!(SCORE_SCALE, 1000.0);
        assert_eq!(SCORE_GAP_LEADING, -0.005);
        assert_eq!(SCORE_GAP_TRAILING, -0.005);
        assert_eq!(SCORE_GAP_INNER, -0.01);
        assert_eq!(SCORE_MATCH_CONSECUTIVE, 1.0);
        assert_eq!(SCORE_MATCH_SLASH, 0.9);
        assert_eq!(SCORE_MATCH_WORD, 0.8);
        assert_eq!(SCORE_MATCH_CAPITAL, 0.7);
        assert_eq!(SCORE_MATCH_DOT, 0.6);
    }

    #[test]
    fn test_is_word_sep() {
        // Word separators
        assert!(is_word_sep('-'));
        assert!(is_word_sep('_'));
        assert!(is_word_sep(' '));

        // Not word separators
        assert!(!is_word_sep('a'));
        assert!(!is_word_sep('Z'));
        assert!(!is_word_sep('0'));
        assert!(!is_word_sep('/'));
        assert!(!is_word_sep('.'));
    }

    #[test]
    fn test_is_path_sep() {
        // Path separator
        assert!(is_path_sep('/'));

        // Not path separators
        assert!(!is_path_sep('\\'));
        assert!(!is_path_sep('-'));
        assert!(!is_path_sep('.'));
    }

    #[test]
    fn test_is_dot() {
        assert!(is_dot('.'));
        assert!(!is_dot(','));
        assert!(!is_dot(':'));
    }

    #[test]
    fn test_is_word_char() {
        // Alphanumeric and underscore are word chars
        assert!(is_word_char('a'));
        assert!(is_word_char('Z'));
        assert!(is_word_char('0'));
        assert!(is_word_char('9'));
        assert!(is_word_char('_'));

        // Non-word chars
        assert!(!is_word_char('-'));
        assert!(!is_word_char(' '));
        assert!(!is_word_char('.'));
        assert!(!is_word_char('/'));
    }
}
