//! Line matching algorithm for diff alignment.
//!
//! Implements an algorithm to find optimal alignment of lines across multiple
//! diff buffers. Uses dynamic programming with a multi-dimensional tensor
//! to compare lines and find the best matching.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::similar_names)]
#![allow(clippy::needless_range_loop)]

use std::ffi::c_int;

use libc::c_long;
use nvim_memory::xmalloc;

// =============================================================================
// Constants
// =============================================================================

const LN_MAX_BUFS: usize = 8;
const LN_DECISION_MAX: usize = 255; // 2^8 - 1
const MATCH_CHAR_MAX_LEN: usize = 800;

// =============================================================================
// Types
// =============================================================================

/// Memory-mapped file type (matches xdiff's `mmfile_t`)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MmFile {
    pub ptr: *mut i8,
    pub size: c_long,
}

impl Default for MmFile {
    fn default() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            size: 0,
        }
    }
}

/// Line number type
type LinenrT = i32;

/// Path comparison state for the diff algorithm
struct DiffCmpPath {
    /// Total score of this path
    df_lev_score: c_int,
    /// Current number of paths
    df_path_n: usize,
    /// Memoization for choice results
    df_choice_mem: [c_int; LN_DECISION_MAX + 1],
    /// Choice at each step
    df_choice: [c_int; LN_DECISION_MAX],
    /// Decision pointers
    df_decision: [usize; LN_DECISION_MAX],
    /// Optimal choice index
    df_optimal_choice: usize,
}

impl Default for DiffCmpPath {
    fn default() -> Self {
        Self {
            df_lev_score: 0,
            df_path_n: 0,
            df_choice_mem: [-1; LN_DECISION_MAX + 1],
            df_choice: [0; LN_DECISION_MAX],
            df_decision: [0; LN_DECISION_MAX],
            df_optimal_choice: 0,
        }
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Get the length of a line (up to newline)
fn line_len(m: &MmFile) -> usize {
    if m.ptr.is_null() || m.size <= 0 {
        return 0;
    }

    let size = m.size as usize;
    // SAFETY: ptr is valid and size is the buffer length
    let slice = unsafe { std::slice::from_raw_parts(m.ptr as *const u8, size) };
    slice.iter().position(|&c| c == b'\n').unwrap_or(size)
}

/// Return matching characters between two strings respecting sequence order.
///
/// Uses LCS (Longest Common Subsequence) algorithm.
///
/// Examples:
/// - `matching_chars("aabc", "acba")` -> 2 ('a' and 'b' in common)
/// - `matching_chars("123hello567", "he123ll567o")` -> 8 ('123', 'll' and '567')
/// - `matching_chars("abcdefg", "gfedcba")` -> 1 (only 1 in sequence)
fn matching_chars(m1: &MmFile, m2: &MmFile) -> c_int {
    let s1len = std::cmp::min(MATCH_CHAR_MAX_LEN - 1, line_len(m1));
    let s2len = std::cmp::min(MATCH_CHAR_MAX_LEN - 1, line_len(m2));

    if s1len == 0 || s2len == 0 {
        return 0;
    }

    // SAFETY: bounds checked above and MmFile has valid ptr
    let s1 = unsafe { std::slice::from_raw_parts(m1.ptr as *const u8, s1len) };
    let s2 = unsafe { std::slice::from_raw_parts(m2.ptr as *const u8, s2len) };

    // Space-optimized LCS: only need 2 rows
    let mut matrix = [[0i32; MATCH_CHAR_MAX_LEN]; 2];
    let mut icur: usize = 1;

    for i in 0..s1len {
        icur = 1 - icur;
        let (prev_row, curr_row) = if icur == 0 {
            let (a, b) = matrix.split_at_mut(1);
            (&b[0], &mut a[0])
        } else {
            let (a, b) = matrix.split_at_mut(1);
            (&a[0], &mut b[0])
        };

        for j in 0..s2len {
            // Skip char in s1
            if prev_row[j + 1] > curr_row[j + 1] {
                curr_row[j + 1] = prev_row[j + 1];
            }
            // Skip char in s2
            if curr_row[j] > curr_row[j + 1] {
                curr_row[j + 1] = curr_row[j];
            }
            // Compare char in s1 and s2
            if s1[i] == s2[j] && (prev_row[j] + 1) > curr_row[j + 1] {
                curr_row[j + 1] = prev_row[j] + 1;
            }
        }
    }

    matrix[icur][s2len]
}

/// Same as `matching_chars` but ignores whitespace
fn matching_chars_iwhite(s1: &MmFile, s2: &MmFile) -> c_int {
    // Remove whitespace from both strings
    let mut p1 = [0u8; MATCH_CHAR_MAX_LEN];
    let mut p2 = [0u8; MATCH_CHAR_MAX_LEN];
    let mut pi1 = 0usize;
    let mut pi2 = 0usize;

    let slen1 = std::cmp::min(MATCH_CHAR_MAX_LEN - 1, line_len(s1));
    let slen2 = std::cmp::min(MATCH_CHAR_MAX_LEN - 1, line_len(s2));

    if slen1 > 0 && !s1.ptr.is_null() {
        let slice = unsafe { std::slice::from_raw_parts(s1.ptr as *const u8, slen1) };
        for &e in slice {
            if e != b' ' && e != b'\t' {
                p1[pi1] = e;
                pi1 += 1;
            }
        }
    }

    if slen2 > 0 && !s2.ptr.is_null() {
        let slice = unsafe { std::slice::from_raw_parts(s2.ptr as *const u8, slen2) };
        for &e in slice {
            if e != b' ' && e != b'\t' {
                p2[pi2] = e;
                pi2 += 1;
            }
        }
    }

    let sp1 = MmFile {
        ptr: p1.as_ptr() as *mut i8,
        size: pi1 as c_long,
    };
    let sp2 = MmFile {
        ptr: p2.as_ptr() as *mut i8,
        size: pi2 as c_long,
    };

    matching_chars(&sp1, &sp2)
}

/// Count matching characters between n strings
fn count_n_matched_chars(sp: &[MmFile], n: usize, iwhite: bool) -> c_int {
    let mut matched_chars = 0;
    let mut matched = 0;

    for i in 0..n {
        for j in (i + 1)..n {
            if !sp[i].ptr.is_null() && !sp[j].ptr.is_null() {
                matched += 1;
                matched_chars += if iwhite {
                    matching_chars_iwhite(&sp[i], &sp[j])
                } else {
                    matching_chars(&sp[i], &sp[j])
                };
            }
        }
    }

    // Prioritize a match of 3+ lines equally to a match of 2 lines
    if matched >= 2 {
        matched_chars = matched_chars * 2 / matched;
    }

    matched_chars
}

/// Fast-forward buffer to line number
fn fastforward_buf_to_lnum(mut s: MmFile, lnum: LinenrT) -> MmFile {
    for _ in 0..(lnum - 1) {
        if s.ptr.is_null() || s.size <= 0 {
            break;
        }

        let size = s.size as usize;
        let slice = unsafe { std::slice::from_raw_parts(s.ptr as *const u8, size) };

        if let Some(pos) = slice.iter().position(|&c| c == b'\n') {
            s.ptr = unsafe { s.ptr.add(pos + 1) };
            s.size -= (pos + 1) as c_long;
        } else {
            s.ptr = std::ptr::null_mut();
            s.size = 0;
            break;
        }
    }
    s
}

/// Unwrap indexes to access n-dimensional tensor
fn unwrap_indexes(values: &[c_int], diff_len: &[c_int], ndiffs: usize) -> usize {
    let mut num_unwrap_scalar: usize = 1;
    for k in 0..ndiffs {
        num_unwrap_scalar *= (diff_len[k] + 1) as usize;
    }

    let mut path_idx: usize = 0;
    for k in 0..ndiffs {
        num_unwrap_scalar /= (diff_len[k] + 1) as usize;
        path_idx += num_unwrap_scalar * values[k] as usize;
    }
    path_idx
}

/// Try all possible paths to find best matching
fn try_possible_paths(
    df_iters: &[c_int],
    paths: &[usize],
    npaths: usize,
    path_idx: usize,
    choice: &mut c_int,
    diffcmppath: &mut [DiffCmpPath],
    diff_len: &[c_int],
    ndiffs: usize,
    diff_blk: &[*const MmFile],
    iwhite: bool,
) {
    if path_idx == npaths {
        if *choice > 0 {
            let mut from_vals = [0i32; LN_MAX_BUFS];
            let mut mm = [MmFile::default(); LN_MAX_BUFS];

            for k in 0..ndiffs {
                from_vals[k] = df_iters[k];
                if (*choice & (1 << k)) != 0 {
                    from_vals[k] -= 1;
                    // SAFETY: diff_blk[k] is valid
                    let blk = unsafe { &*diff_blk[k] };
                    mm[k] = fastforward_buf_to_lnum(*blk, df_iters[k]);
                }
            }

            let unwrapped_idx_from = unwrap_indexes(&from_vals, diff_len, ndiffs);
            let unwrapped_idx_to = unwrap_indexes(df_iters, diff_len, ndiffs);
            let matched_chars = count_n_matched_chars(&mm, ndiffs, iwhite);
            let score = diffcmppath[unwrapped_idx_from].df_lev_score + matched_chars;

            if score > diffcmppath[unwrapped_idx_to].df_lev_score {
                diffcmppath[unwrapped_idx_to].df_path_n = 1;
                diffcmppath[unwrapped_idx_to].df_decision[0] = unwrapped_idx_from;
                diffcmppath[unwrapped_idx_to].df_choice[0] = *choice;
                diffcmppath[unwrapped_idx_to].df_lev_score = score;
            } else if score == diffcmppath[unwrapped_idx_to].df_lev_score {
                let k = diffcmppath[unwrapped_idx_to].df_path_n;
                diffcmppath[unwrapped_idx_to].df_path_n += 1;
                diffcmppath[unwrapped_idx_to].df_decision[k] = unwrapped_idx_from;
                diffcmppath[unwrapped_idx_to].df_choice[k] = *choice;
            }
        }
        return;
    }

    let bit_place = paths[path_idx];
    *choice |= 1 << bit_place; // set to 1
    try_possible_paths(
        df_iters,
        paths,
        npaths,
        path_idx + 1,
        choice,
        diffcmppath,
        diff_len,
        ndiffs,
        diff_blk,
        iwhite,
    );
    *choice &= !(1 << bit_place); // set to 0
    try_possible_paths(
        df_iters,
        paths,
        npaths,
        path_idx + 1,
        choice,
        diffcmppath,
        diff_len,
        ndiffs,
        diff_blk,
        iwhite,
    );
}

/// Populate the tensor with optimal paths
fn populate_tensor(
    df_iters: &mut [c_int],
    ch_dim: usize,
    diffcmppath: &mut [DiffCmpPath],
    diff_len: &[c_int],
    ndiffs: usize,
    diff_blk: &[*const MmFile],
    iwhite: bool,
) {
    if ch_dim == ndiffs {
        let mut npaths = 0;
        let mut paths = [0usize; LN_MAX_BUFS];

        for j in 0..ndiffs {
            if df_iters[j] > 0 {
                paths[npaths] = j;
                npaths += 1;
            }
        }

        let mut choice = 0;
        let unwrapper_idx_to = unwrap_indexes(df_iters, diff_len, ndiffs);
        diffcmppath[unwrapper_idx_to].df_lev_score = -1;
        try_possible_paths(
            df_iters,
            &paths,
            npaths,
            0,
            &mut choice,
            diffcmppath,
            diff_len,
            ndiffs,
            diff_blk,
            iwhite,
        );
        return;
    }

    for i in 0..=diff_len[ch_dim] {
        df_iters[ch_dim] = i;
        populate_tensor(
            df_iters,
            ch_dim + 1,
            diffcmppath,
            diff_len,
            ndiffs,
            diff_blk,
            iwhite,
        );
    }
}

/// Find minimum path changes from start to end with memoization
fn test_charmatch_paths(
    diffcmppath: &mut [DiffCmpPath],
    node_idx: usize,
    lastdecision: c_int,
) -> usize {
    let lastdecision_idx = lastdecision as usize;

    // Memoization check
    if diffcmppath[node_idx].df_choice_mem[lastdecision_idx] == -1 {
        if diffcmppath[node_idx].df_path_n == 0 {
            // End of tree
            diffcmppath[node_idx].df_choice_mem[lastdecision_idx] = 0;
        } else {
            let mut minimum_turns = usize::MAX;
            let path_n = diffcmppath[node_idx].df_path_n;

            for i in 0..path_n {
                let decision_idx = diffcmppath[node_idx].df_decision[i];
                let choice = diffcmppath[node_idx].df_choice[i];

                // Recurse
                let t = test_charmatch_paths(diffcmppath, decision_idx, choice)
                    + if lastdecision != choice { 1 } else { 0 };

                if t < minimum_turns {
                    diffcmppath[node_idx].df_optimal_choice = i;
                    minimum_turns = t;
                }
            }
            diffcmppath[node_idx].df_choice_mem[lastdecision_idx] = minimum_turns as c_int;
        }
    }

    diffcmppath[node_idx].df_choice_mem[lastdecision_idx] as usize
}

// =============================================================================
// Public FFI
// =============================================================================

/// Fast-forward buffer to a specific line number.
///
/// # Safety
/// - `s` must contain a valid pointer if size > 0
#[no_mangle]
pub unsafe extern "C" fn rs_fastforward_buf_to_lnum(s: MmFile, lnum: LinenrT) -> MmFile {
    fastforward_buf_to_lnum(s, lnum)
}

/// Find optimal line alignment across multiple diff buffers.
///
/// # Safety
/// - `diff_blk` must point to `ndiffs` valid `MmFile` pointers
/// - `diff_len` must point to `ndiffs` valid integers
/// - `decisions` must be a valid pointer to receive allocated result
/// - `ndiffs` must be <= 8
#[no_mangle]
pub unsafe extern "C" fn rs_linematch_nbuffers(
    diff_blk: *const *const MmFile,
    diff_len: *const c_int,
    ndiffs: usize,
    decisions: *mut *mut c_int,
    iwhite: bool,
) -> usize {
    assert!(ndiffs <= LN_MAX_BUFS);

    // Copy inputs to slices
    let diff_blk_slice = std::slice::from_raw_parts(diff_blk, ndiffs);
    let diff_len_slice = std::slice::from_raw_parts(diff_len, ndiffs);

    // Calculate tensor size
    let mut memsize: usize = 1;
    let mut memsize_decisions: usize = 0;
    for i in 0..ndiffs {
        assert!(diff_len_slice[i] >= 0);
        memsize *= (diff_len_slice[i] + 1) as usize;
        memsize_decisions += diff_len_slice[i] as usize;
    }

    // Allocate path tensor
    let mut diffcmppath: Vec<DiffCmpPath> = Vec::with_capacity(memsize);
    let n = 1usize << ndiffs; // 2^ndiffs
    for _ in 0..memsize {
        let mut path = DiffCmpPath::default();
        for j in 0..n {
            path.df_choice_mem[j] = -1;
        }
        diffcmppath.push(path);
    }

    // Populate tensor
    let mut df_iters = [0i32; LN_MAX_BUFS];
    populate_tensor(
        &mut df_iters,
        0,
        &mut diffcmppath,
        diff_len_slice,
        ndiffs,
        diff_blk_slice,
        iwhite,
    );

    // Find start node
    let u = unwrap_indexes(diff_len_slice, diff_len_slice, ndiffs);

    // Allocate result
    let result_ptr: *mut c_int = xmalloc(std::mem::size_of::<c_int>() * memsize_decisions).cast();
    let result_slice = std::slice::from_raw_parts_mut(result_ptr, memsize_decisions);

    // Trace optimal path
    let mut n_optimal = 0;
    test_charmatch_paths(&mut diffcmppath, u, 0);

    let mut current_idx = u;
    while diffcmppath[current_idx].df_path_n > 0 {
        let j = diffcmppath[current_idx].df_optimal_choice;
        result_slice[n_optimal] = diffcmppath[current_idx].df_choice[j];
        n_optimal += 1;
        current_idx = diffcmppath[current_idx].df_decision[j];
    }

    // Reverse array
    for i in 0..(n_optimal / 2) {
        result_slice.swap(i, n_optimal - 1 - i);
    }

    *decisions = result_ptr;
    n_optimal
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mmfile(s: &str) -> MmFile {
        MmFile {
            ptr: s.as_ptr() as *mut i8,
            size: s.len() as c_long,
        }
    }

    #[test]
    fn test_matching_chars_basic() {
        let m1 = make_mmfile("aabc");
        let m2 = make_mmfile("acba");
        assert_eq!(matching_chars(&m1, &m2), 2); // 'a' and 'b'
    }

    #[test]
    fn test_matching_chars_longer() {
        let m1 = make_mmfile("123hello567");
        let m2 = make_mmfile("he123ll567o");
        assert_eq!(matching_chars(&m1, &m2), 8); // '123', 'll', '567'
    }

    #[test]
    fn test_matching_chars_reversed() {
        let m1 = make_mmfile("abcdefg");
        let m2 = make_mmfile("gfedcba");
        assert_eq!(matching_chars(&m1, &m2), 1); // only 1 in sequence
    }

    #[test]
    fn test_matching_chars_identical() {
        let m1 = make_mmfile("hello");
        let m2 = make_mmfile("hello");
        assert_eq!(matching_chars(&m1, &m2), 5);
    }

    #[test]
    fn test_matching_chars_empty() {
        let m1 = make_mmfile("");
        let m2 = make_mmfile("hello");
        assert_eq!(matching_chars(&m1, &m2), 0);
    }

    #[test]
    fn test_matching_chars_iwhite() {
        let m1 = make_mmfile("a b c");
        let m2 = make_mmfile("abc");
        assert_eq!(matching_chars_iwhite(&m1, &m2), 3);
    }

    #[test]
    fn test_line_len() {
        let m1 = make_mmfile("hello\nworld");
        assert_eq!(line_len(&m1), 5);

        let m2 = make_mmfile("hello");
        assert_eq!(line_len(&m2), 5);
    }

    #[test]
    fn test_fastforward_buf_to_lnum() {
        let m = make_mmfile("line1\nline2\nline3");
        let result = fastforward_buf_to_lnum(m, 2);
        assert!(!result.ptr.is_null());
        let slice = unsafe { std::slice::from_raw_parts(result.ptr as *const u8, 5) };
        assert_eq!(slice, b"line2");
    }

    #[test]
    fn test_unwrap_indexes() {
        let values = [1, 2];
        let diff_len = [3, 3];
        // 4x4 grid, index = 1*4 + 2 = 6
        assert_eq!(unwrap_indexes(&values, &diff_len, 2), 6);
    }
}
