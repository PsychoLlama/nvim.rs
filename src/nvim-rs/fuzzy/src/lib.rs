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

use libc::{c_char, c_int, c_void};
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

/// Get the `SCORE_NONE` constant value (sentinel for no match).
#[unsafe(no_mangle)]
pub const extern "C" fn rs_fuzzy_score_none() -> c_int {
    SCORE_NONE
}

/// Get the maximum match length constant.
#[unsafe(no_mangle)]
pub const extern "C" fn rs_fuzzy_max_len() -> c_int {
    MATCH_MAX_LEN as c_int
}

/// Get the gap leading penalty (scaled to integer).
#[unsafe(no_mangle)]
pub extern "C" fn rs_fuzzy_penalty_gap_leading() -> c_int {
    (SCORE_GAP_LEADING * SCORE_SCALE) as c_int
}

/// Get the gap trailing penalty (scaled to integer).
#[unsafe(no_mangle)]
pub extern "C" fn rs_fuzzy_penalty_gap_trailing() -> c_int {
    (SCORE_GAP_TRAILING * SCORE_SCALE) as c_int
}

/// Get the inner gap penalty (scaled to integer).
#[unsafe(no_mangle)]
pub extern "C" fn rs_fuzzy_penalty_gap_inner() -> c_int {
    (SCORE_GAP_INNER * SCORE_SCALE) as c_int
}

/// Get the consecutive match bonus (scaled to integer).
#[unsafe(no_mangle)]
pub extern "C" fn rs_fuzzy_bonus_consecutive() -> c_int {
    (SCORE_MATCH_CONSECUTIVE * SCORE_SCALE) as c_int
}

/// Get the slash (path separator) boundary bonus (scaled to integer).
#[unsafe(no_mangle)]
pub extern "C" fn rs_fuzzy_bonus_slash() -> c_int {
    (SCORE_MATCH_SLASH * SCORE_SCALE) as c_int
}

/// Get the word boundary bonus (scaled to integer).
#[unsafe(no_mangle)]
pub extern "C" fn rs_fuzzy_bonus_word() -> c_int {
    (SCORE_MATCH_WORD * SCORE_SCALE) as c_int
}

/// Get the capital letter (camelCase) bonus (scaled to integer).
#[unsafe(no_mangle)]
pub extern "C" fn rs_fuzzy_bonus_capital() -> c_int {
    (SCORE_MATCH_CAPITAL * SCORE_SCALE) as c_int
}

/// Get the dot boundary bonus (scaled to integer).
#[unsafe(no_mangle)]
pub extern "C" fn rs_fuzzy_bonus_dot() -> c_int {
    (SCORE_MATCH_DOT * SCORE_SCALE) as c_int
}

/// Check if a fuzzy match exists without computing score.
///
/// This is faster than `rs_fuzzy_match` when you only need to know if a match exists.
///
/// # Safety
///
/// `str_ptr` and `pat_ptr` must be valid null-terminated C strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_fuzzy_has_match(
    str_ptr: *const c_char,
    pat_ptr: *const c_char,
) -> bool {
    let haystack = match cstr_to_str(str_ptr) {
        Some(s) => s,
        None => return false,
    };

    let pattern = match cstr_to_str(pat_ptr) {
        Some(s) => s,
        None => return false,
    };

    has_match(pattern, haystack)
}

/// Compute the boundary bonus for a character at given position.
///
/// This function computes what bonus a character would get based on the
/// preceding character context.
///
/// # Safety
///
/// `prev_char` and `curr_char` should be valid UTF-8 character codes.
#[unsafe(no_mangle)]
pub extern "C" fn rs_fuzzy_compute_bonus(prev_char: u32, curr_char: u32) -> c_int {
    let prev_c = char::from_u32(prev_char).unwrap_or('\0');
    let curr_c = char::from_u32(curr_char).unwrap_or('\0');
    let bonus = compute_bonus(prev_c, curr_c);
    (bonus * SCORE_SCALE) as c_int
}

/// Compare two fuzzy match items for sorting (descending by score, stable by index).
///
/// Returns:
/// - negative if item1 should come before item2
/// - positive if item1 should come after item2
/// - zero if they are equal
#[unsafe(no_mangle)]
#[allow(clippy::comparison_chain, clippy::missing_const_for_fn)]
pub extern "C" fn rs_fuzzy_match_item_compare(
    score1: c_int,
    idx1: c_int,
    score2: c_int,
    idx2: c_int,
) -> c_int {
    if score1 == score2 {
        // Stable sort by index
        if idx1 == idx2 {
            0
        } else if idx1 > idx2 {
            1
        } else {
            -1
        }
    } else if score1 > score2 {
        -1 // Higher score comes first (descending order)
    } else {
        1
    }
}

/// Compare two fuzzy match strings for sorting (descending by score, stable by index).
///
/// Same as `rs_fuzzy_match_item_compare` but for string matches.
#[unsafe(no_mangle)]
pub extern "C" fn rs_fuzzy_match_str_compare(
    score1: c_int,
    idx1: c_int,
    score2: c_int,
    idx2: c_int,
) -> c_int {
    rs_fuzzy_match_item_compare(score1, idx1, score2, idx2)
}

/// Compare two fuzzy function matches for sorting.
///
/// This is similar to string comparison but moves `<SNR>` functions to the end.
///
/// # Safety
///
/// `str1` and `str2` must be valid null-terminated C strings.
#[unsafe(no_mangle)]
#[allow(clippy::comparison_chain)]
pub unsafe extern "C" fn rs_fuzzy_match_func_compare(
    str1: *const c_char,
    score1: c_int,
    idx1: c_int,
    str2: *const c_char,
    score2: c_int,
    idx2: c_int,
) -> c_int {
    let s1 = cstr_to_str(str1).unwrap_or("");
    let s2 = cstr_to_str(str2).unwrap_or("");

    let s1_is_snr = s1.starts_with('<');
    let s2_is_snr = s2.starts_with('<');

    // Move <SNR> functions to the end
    if !s1_is_snr && s2_is_snr {
        return -1;
    }
    if s1_is_snr && !s2_is_snr {
        return 1;
    }

    // Otherwise, sort by score (descending), then by index (stable)
    if score1 == score2 {
        if idx1 == idx2 {
            0
        } else if idx1 > idx2 {
            1
        } else {
            -1
        }
    } else if score1 > score2 {
        -1
    } else {
        1
    }
}

// =============================================================================
// Memory management FFI
// =============================================================================

/// Opaque handle for C `buf_T *`.
type BufHandle = *mut c_void;

extern "C" {
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn ga_init(gap: *mut GArray, itemsize: c_int, growsize: c_int);
    fn ga_clear(gap: *mut GArray);
    fn ga_grow(gap: *mut GArray, n: c_int);
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn rs_find_word_start(ptr: *mut c_char) -> *mut c_char;
    fn rs_find_word_end(ptr: *mut c_char) -> *mut c_char;
    fn rs_find_line_end(ptr: *mut c_char) -> *mut c_char;
    #[link_name = "vim_iswordp"]
    fn rs_vim_iswordp(p: *const c_char) -> bool;
    fn rs_ctrl_x_mode_whole_line() -> c_int;
    fn rs_equalpos(a: PosT, b: PosT) -> c_int;
    fn nvim_ml_get_buf(buf: BufHandle, lnum: c_int) -> *mut c_char;
    fn nvim_ml_get_buf_len(buf: BufHandle, lnum: c_int) -> c_int;
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> c_int;
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_get_p_ws() -> c_int;
}

// =============================================================================
// PosT - matches C pos_T layout from pos_defs.h
// =============================================================================

/// Position in file. Matches C `pos_T` layout.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PosT {
    pub lnum: c_int,
    pub col: c_int,
    pub coladd: c_int,
}

// =============================================================================
// GArray - matches C garray_T layout from garray_defs.h
// =============================================================================

/// Growing array structure - matches C `garray_T` layout exactly.
#[repr(C)]
#[allow(clippy::struct_field_names)]
pub struct GArray {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

// =============================================================================
// FuzmatchStr - matches C fuzmatch_str_T layout from fuzzy.h
// =============================================================================

/// Fuzzy matched string list item. Matches C `fuzmatch_str_T` layout.
#[repr(C)]
pub struct FuzmatchStr {
    pub idx: c_int,
    pub str_ptr: *mut c_char,
    pub score: c_int,
}

/// Free an array of fuzzy string matches.
///
/// # Safety
///
/// `fuzmatch` must be a valid pointer to an array of `count` `FuzmatchStr` items
/// allocated with `xmalloc`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_fuzmatch_str_free(fuzmatch: *mut FuzmatchStr, count: c_int) {
    if fuzmatch.is_null() {
        return;
    }
    let items = std::slice::from_raw_parts_mut(fuzmatch, count as usize);
    for item in items {
        xfree(item.str_ptr.cast::<c_void>());
    }
    xfree(fuzmatch.cast::<c_void>());
}

/// Sort fuzzy matches and copy the string pointers into a newly allocated
/// `char **` array. Frees the `fuzmatch` array (but not the strings, which
/// are transferred to `*matches`).
///
/// # Safety
///
/// `fuzmatch` must be a valid pointer to an array of `count` `FuzmatchStr` items.
/// `matches` must be a valid pointer to a `*mut *mut c_char`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_fuzzymatches_to_strmatches(
    fuzmatch: *mut FuzmatchStr,
    matches: *mut *mut *mut c_char,
    count: c_int,
    funcsort: bool,
) {
    if count <= 0 {
        xfree(fuzmatch.cast::<c_void>());
        return;
    }

    let items = std::slice::from_raw_parts_mut(fuzmatch, count as usize);

    // Sort in-place using Rust's stable sort
    if funcsort {
        items.sort_by(|a, b| {
            let cmp = rs_fuzzy_match_func_compare(
                a.str_ptr.cast_const(),
                a.score,
                a.idx,
                b.str_ptr.cast_const(),
                b.score,
                b.idx,
            );
            cmp.cmp(&0)
        });
    } else {
        items.sort_by(|a, b| {
            let cmp = rs_fuzzy_match_str_compare(a.score, a.idx, b.score, b.idx);
            cmp.cmp(&0)
        });
    }

    // Allocate the output array
    let out = xmalloc(count as usize * std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();
    *matches = out;

    // Transfer string pointers
    let out_slice = std::slice::from_raw_parts_mut(out, count as usize);
    for (i, item) in items.iter().enumerate() {
        out_slice[i] = item.str_ptr;
    }

    // Free the fuzmatch array (strings are now owned by out)
    xfree(fuzmatch.cast::<c_void>());
}

/// Fuzzy match the position of string "pat" in string "str".
/// Returns a heap-allocated `GArray` of matching positions, or null on no match.
///
/// Uses `matchseq=false` (multi-word matching with whitespace splitting).
///
/// # Safety
///
/// `str_ptr` and `pat_ptr` must be valid null-terminated C strings or null.
/// Caller is responsible for freeing the returned `GArray` with `ga_clear` + `xfree`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_fuzzy_match_str_with_pos(
    str_ptr: *const c_char,
    pat_ptr: *const c_char,
) -> *mut GArray {
    if str_ptr.is_null() || pat_ptr.is_null() {
        return std::ptr::null_mut();
    }

    // Allocate and initialize the GArray
    let match_positions = xmalloc(std::mem::size_of::<GArray>()).cast::<GArray>();
    ga_init(match_positions, std::mem::size_of::<u32>() as c_int, 10);

    // Try fuzzy match with matchseq=false
    let mut score: c_int = SCORE_NONE;
    let mut matches = [0u32; MATCH_MAX_LEN];
    let matched = rs_fuzzy_match(
        str_ptr,
        pat_ptr,
        false,
        std::ptr::addr_of_mut!(score),
        matches.as_mut_ptr(),
        MATCH_MAX_LEN as c_int,
    );

    if !matched || score == SCORE_NONE {
        ga_clear(match_positions);
        xfree(match_positions.cast::<c_void>());
        return std::ptr::null_mut();
    }

    // Walk the pattern, skipping whitespace, appending positions
    let mut j: usize = 0;
    let mut p = pat_ptr;
    while *p != 0 {
        // Check if current char is NOT ascii whitespace
        let byte = *p as u8;
        let is_white = byte == b' ' || byte == b'\t' || byte == b'\n' || byte == b'\r';
        if !is_white {
            // Append match position to GArray
            ga_grow(match_positions, 1);
            let ga = &mut *match_positions;
            let data = ga.ga_data.cast::<u32>();
            *data.add(ga.ga_len as usize) = matches[j];
            ga.ga_len += 1;
            j += 1;
        }
        // Advance by one UTF-8 character (MB_PTR_ADV equivalent)
        let char_len = utfc_ptr2len(p);
        p = p.add(if char_len > 0 { char_len as usize } else { 1 });
    }

    match_positions
}

// =============================================================================
// Fuzzy search in lines/buffers
// =============================================================================

/// FORWARD direction constant, matching C `FORWARD = 1` from `vim_defs.h`.
const FORWARD: c_int = 1;

/// Advance a pointer by one UTF-8 character (`MB_PTR_ADV` equivalent).
unsafe fn mb_ptr_adv(p: *mut c_char) -> *mut c_char {
    let len = utfc_ptr2len(p);
    p.add(if len > 0 { len as usize } else { 1 })
}

/// Split a line into words and fuzzy-match each against `pat`.
///
/// On match: `*ptr` → start of matched word, `*len_out` → word length,
/// `*score_out` → match score, `current_pos.col` advanced past the word.
///
/// On no match: `*ptr` → end of line.
///
/// # Safety
///
/// All pointer parameters must be valid. `current_pos` may be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_fuzzy_match_str_in_line(
    ptr: *mut *mut c_char,
    pat: *mut c_char,
    len_out: *mut c_int,
    current_pos: *mut PosT,
    score_out: *mut c_int,
) -> bool {
    let str_start = *ptr;
    let str_begin = str_start;

    if str_start.is_null() || pat.is_null() {
        return false;
    }

    let line_end = rs_find_line_end(str_start);
    let mut str_cur = str_start;

    while str_cur < line_end {
        // Skip non-word characters
        let start = rs_find_word_start(str_cur);
        if *start == 0 {
            break;
        }
        let end = rs_find_word_end(start);

        // Temporarily null-terminate the word
        let save_end = *end;
        *end = 0;

        // Perform fuzzy match
        *score_out = rs_fuzzy_match_str(start.cast_const(), pat.cast_const());
        *end = save_end;

        if *score_out != SCORE_NONE {
            *len_out = end.offset_from(start) as c_int;
            *ptr = start;
            if !current_pos.is_null() {
                (*current_pos).col += end.offset_from(str_begin) as c_int;
            }
            return true;
        }

        // Move to end of current word, then skip non-word chars
        str_cur = end;
        while *str_cur != 0 && !rs_vim_iswordp(str_cur.cast_const()) {
            str_cur = mb_ptr_adv(str_cur);
        }
    }

    *ptr = line_end;
    false
}

/// Search for the next fuzzy match in a buffer, with direction and wrapping.
///
/// # Safety
///
/// All pointer parameters must be valid. `buf` must be a valid `buf_T *`.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rs_search_for_fuzzy_match(
    buf: BufHandle,
    pos: *mut PosT,
    pattern: *mut c_char,
    dir: c_int,
    start_pos: *mut PosT,
    len_out: *mut c_int,
    ptr_out: *mut *mut c_char,
    score_out: *mut c_int,
) -> bool {
    let mut current_pos = *pos;
    let mut found_new_match = false;
    let mut looped_around = false;

    let whole_line = rs_ctrl_x_mode_whole_line() != 0;
    let line_count = nvim_buf_get_ml_line_count(buf);

    let circly_end = if buf == nvim_get_curbuf() {
        *start_pos
    } else {
        PosT {
            lnum: line_count,
            col: 0,
            coladd: 0,
        }
    };

    if whole_line && (*start_pos).lnum != (*pos).lnum {
        current_pos.lnum += dir;
    }

    loop {
        // Check if looped around and back to start position
        if looped_around && rs_equalpos(current_pos, circly_end) != 0 {
            break;
        }

        // Ensure current_pos is valid
        if current_pos.lnum >= 1 && current_pos.lnum <= line_count {
            // Get the current line buffer
            *ptr_out = nvim_ml_get_buf(buf, current_pos.lnum);
            if !whole_line {
                *ptr_out = (*ptr_out).add(current_pos.col as usize);
            }

            // If end of line not reached
            if !(*ptr_out).is_null() && **ptr_out != 0 {
                if !whole_line {
                    found_new_match = rs_fuzzy_match_str_in_line(
                        ptr_out,
                        pattern,
                        len_out,
                        std::ptr::addr_of_mut!(current_pos),
                        score_out,
                    );
                    if found_new_match {
                        *pos = current_pos;
                        break;
                    } else if looped_around && current_pos.lnum == circly_end.lnum {
                        break;
                    }
                } else if rs_fuzzy_match_str((*ptr_out).cast_const(), pattern.cast_const())
                    != SCORE_NONE
                {
                    found_new_match = true;
                    *pos = current_pos;
                    *len_out = nvim_ml_get_buf_len(buf, current_pos.lnum);
                    break;
                }
            }
        }

        // Move to next/previous line
        if dir == FORWARD {
            current_pos.lnum += 1;
            if current_pos.lnum > line_count {
                if nvim_get_p_ws() != 0 {
                    current_pos.lnum = 1;
                    looped_around = true;
                } else {
                    break;
                }
            }
        } else {
            current_pos.lnum -= 1;
            if current_pos.lnum < 1 {
                if nvim_get_p_ws() != 0 {
                    current_pos.lnum = line_count;
                    looped_around = true;
                } else {
                    break;
                }
            }
        }
        current_pos.col = 0;
    }

    found_new_match
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

    #[test]
    fn test_ffi_score_none() {
        assert_eq!(rs_fuzzy_score_none(), SCORE_NONE);
    }

    #[test]
    fn test_ffi_max_len() {
        assert_eq!(rs_fuzzy_max_len(), MATCH_MAX_LEN as c_int);
    }

    #[test]
    fn test_ffi_penalties() {
        assert_eq!(rs_fuzzy_penalty_gap_leading(), -5);
        assert_eq!(rs_fuzzy_penalty_gap_trailing(), -5);
        assert_eq!(rs_fuzzy_penalty_gap_inner(), -10);
    }

    #[test]
    fn test_ffi_bonuses() {
        assert_eq!(rs_fuzzy_bonus_consecutive(), 1000);
        assert_eq!(rs_fuzzy_bonus_slash(), 900);
        assert_eq!(rs_fuzzy_bonus_word(), 800);
        assert_eq!(rs_fuzzy_bonus_capital(), 700);
        assert_eq!(rs_fuzzy_bonus_dot(), 600);
    }

    #[test]
    fn test_ffi_has_match() {
        use std::ffi::CString;

        let haystack = CString::new("foo_bar").unwrap();
        let pattern = CString::new("fb").unwrap();
        let bad_pattern = CString::new("xyz").unwrap();

        unsafe {
            assert!(rs_fuzzy_has_match(haystack.as_ptr(), pattern.as_ptr()));
            assert!(!rs_fuzzy_has_match(haystack.as_ptr(), bad_pattern.as_ptr()));
        }
    }

    #[test]
    fn test_ffi_compute_bonus() {
        // Test word boundary bonus
        let bonus = rs_fuzzy_compute_bonus('_' as u32, 'a' as u32);
        assert_eq!(bonus, 800); // SCORE_MATCH_WORD * SCALE

        // Test slash boundary bonus
        let bonus = rs_fuzzy_compute_bonus('/' as u32, 'a' as u32);
        assert_eq!(bonus, 900); // SCORE_MATCH_SLASH * SCALE

        // Test capital bonus (camelCase)
        let bonus = rs_fuzzy_compute_bonus('a' as u32, 'B' as u32);
        assert_eq!(bonus, 700); // SCORE_MATCH_CAPITAL * SCALE

        // Test dot boundary bonus
        let bonus = rs_fuzzy_compute_bonus('.' as u32, 'a' as u32);
        assert_eq!(bonus, 600); // SCORE_MATCH_DOT * SCALE

        // Test no bonus
        let bonus = rs_fuzzy_compute_bonus('a' as u32, 'b' as u32);
        assert_eq!(bonus, 0);
    }

    #[test]
    fn test_ffi_match_item_compare() {
        // Higher score should come first
        assert!(rs_fuzzy_match_item_compare(100, 0, 50, 1) < 0);
        assert!(rs_fuzzy_match_item_compare(50, 0, 100, 1) > 0);

        // Same score - stable sort by index
        assert_eq!(rs_fuzzy_match_item_compare(100, 0, 100, 0), 0);
        assert!(rs_fuzzy_match_item_compare(100, 1, 100, 0) > 0);
        assert!(rs_fuzzy_match_item_compare(100, 0, 100, 1) < 0);
    }

    #[test]
    fn test_ffi_match_str_compare() {
        // Should behave the same as item compare
        assert!(rs_fuzzy_match_str_compare(100, 0, 50, 1) < 0);
        assert!(rs_fuzzy_match_str_compare(50, 0, 100, 1) > 0);
    }

    #[test]
    fn test_ffi_match_func_compare() {
        use std::ffi::CString;

        let normal_func = CString::new("foo").unwrap();
        let snr_func = CString::new("<SNR>123_bar").unwrap();

        unsafe {
            // Normal function should come before <SNR> function
            assert!(
                rs_fuzzy_match_func_compare(
                    normal_func.as_ptr(),
                    100,
                    0,
                    snr_func.as_ptr(),
                    100,
                    1
                ) < 0
            );

            // <SNR> function should come after normal function
            assert!(
                rs_fuzzy_match_func_compare(
                    snr_func.as_ptr(),
                    100,
                    0,
                    normal_func.as_ptr(),
                    100,
                    1
                ) > 0
            );

            // Two normal functions - sort by score
            let func2 = CString::new("bar").unwrap();
            assert!(
                rs_fuzzy_match_func_compare(normal_func.as_ptr(), 100, 0, func2.as_ptr(), 50, 1)
                    < 0
            );
        }
    }

    #[test]
    fn test_fuzmatch_str_layout() {
        // Verify FuzmatchStr matches C fuzmatch_str_T layout:
        // { int idx; char *str; int score; }
        // On x86-64: idx@0, str@8 (padded), score@16, total=24
        assert_eq!(std::mem::size_of::<FuzmatchStr>(), 24);
        assert_eq!(std::mem::offset_of!(FuzmatchStr, idx), 0);
        assert_eq!(std::mem::offset_of!(FuzmatchStr, str_ptr), 8);
        assert_eq!(std::mem::offset_of!(FuzmatchStr, score), 16);
    }
}
