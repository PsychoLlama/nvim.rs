//! File and path completion support.
//!
//! This module provides helper functions for filename and path completion.
//! The core wildcard expansion and file matching remain in C, but Rust
//! provides utilities for path manipulation and state management.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int, c_void};

// C accessor functions

// CTRL-X mode constants
const CTRL_X_FILES: c_int = 4;
const CTRL_X_PATH_PATTERNS: c_int = 6 + 0x100; // 6 + CTRL_X_WANT_IDENT
const CTRL_X_PATH_DEFINES: c_int = 7 + 0x100; // 7 + CTRL_X_WANT_IDENT

// Path separator (Unix)
#[allow(clippy::cast_possible_wrap)]
const PATH_SEP: c_char = b'/' as c_char;

/// Find the last path separator in a string.
///
/// Returns the byte offset of the last separator, or -1 if not found.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_find_last_sep(path: *const c_char) -> c_int {
    if path.is_null() {
        return -1;
    }

    let mut last_pos: c_int = -1;
    let mut pos = 0;
    let mut ptr = path;

    while *ptr != 0 {
        if *ptr == PATH_SEP {
            last_pos = pos;
        }
        ptr = ptr.add(1);
        pos += 1;
    }

    last_pos
}

/// Find the last path separator in a string (Windows-aware).
///
/// Also checks for backslash as a separator.
/// Returns the byte offset of the last separator, or -1 if not found.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_find_last_sep_any(path: *const c_char) -> c_int {
    if path.is_null() {
        return -1;
    }

    let mut last_pos: c_int = -1;
    let mut pos = 0;
    let mut ptr = path;

    while *ptr != 0 {
        if *ptr == b'/' as c_char || *ptr == b'\\' as c_char {
            last_pos = pos;
        }
        ptr = ptr.add(1);
        pos += 1;
    }

    last_pos
}

/// Get the length of the directory part of a path.
///
/// Returns 0 if there's no directory component.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_file_dir_len(path: *const c_char) -> c_int {
    let last_sep = rs_file_find_last_sep_any(path);
    if last_sep < 0 {
        0
    } else {
        last_sep + 1
    }
}

/// Get the length of the filename part of a path.
///
/// Returns the string length if there's no directory component.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_basename_len(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let last_sep = rs_file_find_last_sep_any(path);
    let mut len = 0;
    let mut ptr = path;

    while *ptr != 0 {
        len += 1;
        ptr = ptr.add(1);
    }

    if last_sep < 0 {
        len
    } else {
        len - last_sep - 1
    }
}

/// Check if a path is absolute.
///
/// Returns 1 if path starts with / (Unix) or has a drive letter (Windows).
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_is_absolute(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }

    // Unix absolute path
    if *path == b'/' as c_char {
        return 1;
    }

    // Windows drive letter (e.g., C:\)
    let first = *path;
    let second = *path.add(1);
    if ((first >= b'A' as c_char && first <= b'Z' as c_char)
        || (first >= b'a' as c_char && first <= b'z' as c_char))
        && second == b':' as c_char
    {
        return 1;
    }

    0
}

/// Check if a path ends with a path separator.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_ends_with_sep(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }

    // Find end of string
    let mut ptr = path;
    while *ptr != 0 {
        ptr = ptr.add(1);
    }

    // Check last character
    let last = *ptr.sub(1);
    c_int::from(last == b'/' as c_char || last == b'\\' as c_char)
}

/// Count path components in a path.
///
/// Returns the number of directory levels.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_count_path_components(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }

    let mut count = 0;
    let mut ptr = path;
    let mut in_component = false;

    while *ptr != 0 {
        if *ptr == b'/' as c_char || *ptr == b'\\' as c_char {
            if in_component {
                count += 1;
                in_component = false;
            }
        } else {
            in_component = true;
        }
        ptr = ptr.add(1);
    }

    // Count last component if not ending with separator
    if in_component {
        count += 1;
    }

    count
}

// =============================================================================
// Phase 4: Extended File Completion Functions
// =============================================================================

// Additional C accessor functions
extern "C" {
    fn xmalloc(size: usize) -> *mut c_char;
    fn nvim_get_cursor_col() -> c_int;
}

/// Check if path has a wildcard character.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_file_has_wildcard(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let mut ptr = path;
    while *ptr != 0 {
        let c = *ptr;
        if c == b'*' as c_char || c == b'?' as c_char || c == b'[' as c_char {
            return 1;
        }
        ptr = ptr.add(1);
    }

    0
}

/// Check if path is a hidden file (starts with dot).
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_is_hidden(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }

    // Find basename
    let last_sep = rs_file_find_last_sep_any(path);
    let basename = if last_sep < 0 {
        path
    } else {
        #[allow(clippy::cast_sign_loss)]
        path.add(last_sep as usize + 1)
    };

    // Check if starts with dot
    c_int::from(*basename == b'.' as c_char)
}

/// Check if path represents current or parent directory.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_is_dot_or_dotdot(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }

    // Find basename
    let last_sep = rs_file_find_last_sep_any(path);
    let basename = if last_sep < 0 {
        path
    } else {
        #[allow(clippy::cast_sign_loss)]
        path.add(last_sep as usize + 1)
    };

    // Check for "." or ".."
    if *basename == b'.' as c_char {
        let next = *basename.add(1);
        if next == 0 {
            return 1; // "."
        }
        if next == b'.' as c_char && *basename.add(2) == 0 {
            return 1; // ".."
        }
    }

    0
}

/// Get the extension of a filename.
///
/// Returns the offset of the extension (including the dot), or -1 if no extension.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_file_extension_offset(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return -1;
    }

    // Find basename first
    let last_sep = rs_file_find_last_sep_any(path);
    let basename_start = if last_sep < 0 {
        0
    } else {
        (last_sep + 1) as usize
    };

    // Find last dot in basename
    let mut last_dot: c_int = -1;
    let mut pos = basename_start;
    let mut ptr = path.add(basename_start);

    while *ptr != 0 {
        if *ptr == b'.' as c_char {
            #[allow(clippy::cast_possible_truncation)]
            {
                last_dot = pos as c_int;
            }
        }
        ptr = ptr.add(1);
        pos += 1;
    }

    // Don't count dot at start of basename as extension
    #[allow(clippy::cast_possible_truncation)]
    if last_dot == basename_start as c_int {
        -1
    } else {
        last_dot
    }
}

/// Compare two paths for equality, normalizing separators.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_file_paths_equal(path1: *const c_char, path2: *const c_char) -> c_int {
    if path1.is_null() || path2.is_null() {
        return c_int::from(path1 == path2); // Both null = equal
    }

    let mut p1 = path1;
    let mut p2 = path2;

    while *p1 != 0 && *p2 != 0 {
        let c1 = if *p1 == b'\\' as c_char {
            b'/' as c_char
        } else {
            *p1
        };
        let c2 = if *p2 == b'\\' as c_char {
            b'/' as c_char
        } else {
            *p2
        };

        if c1 != c2 {
            return 0;
        }

        p1 = p1.add(1);
        p2 = p2.add(1);
    }

    c_int::from(*p1 == 0 && *p2 == 0)
}

/// Check if path1 is a prefix of path2.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_file_is_prefix(prefix: *const c_char, path: *const c_char) -> c_int {
    if prefix.is_null() || path.is_null() {
        return 0;
    }

    let mut p1 = prefix;
    let mut p2 = path;

    while *p1 != 0 {
        if *p2 == 0 {
            return 0; // path is shorter
        }

        let c1 = if *p1 == b'\\' as c_char {
            b'/' as c_char
        } else {
            *p1
        };
        let c2 = if *p2 == b'\\' as c_char {
            b'/' as c_char
        } else {
            *p2
        };

        if c1 != c2 {
            return 0;
        }

        p1 = p1.add(1);
        p2 = p2.add(1);
    }

    1
}

// =============================================================================
// Phase 15: rs_get_next_filename_completion -- full Rust implementation
// =============================================================================

extern "C" {
    // Leader and fuzzy helpers
    fn rs_ins_compl_leader() -> *const c_char;
    fn rs_ins_compl_leader_len() -> usize;
    fn rs_cot_fuzzy() -> c_int;
    fn rs_fuzzy_longest_match();

    // Wildcard expansion
    fn nvim_expand_wildcards_files(
        count: c_int,
        pat: *mut *mut c_char,
        num_matches: *mut c_int,
        matches: *mut *mut *mut c_char,
    ) -> c_int;
    fn nvim_tilde_replace_wrap(pat: *mut c_char, num_matches: c_int, matches: *mut *mut c_char);
    // nvim_get_p_fic_or_wic: inlined in vars.rs (Phase 29)

    // nvim_compl_pattern_set_star: deleted (Phase 30), inlined below via cbuf_to_string
    #[link_name = "cbuf_to_string"]
    fn cbuf_to_string_file(buf: *const c_char, size: usize) -> crate::vars::NvimString;

    // Fuzzy matching
    fn fuzzy_match_str(str_: *mut c_char, pat: *const c_char) -> c_int;

    // Match addition (direct call)
    fn rs_ins_compl_add(
        str_: *const c_char,
        len: c_int,
        fname: *const c_char,
        cptext: *const c_void,
        cptext_allocated: c_int,
        user_data: *const c_void,
        dir: c_int,
        flags: c_int,
        adup: c_int,
        user_hl: *const c_int,
        score: c_int,
    ) -> c_int;
    fn rs_ins_compl_add_matches(num_matches: c_int, matches: *mut *mut c_char, icase: c_int);

    // Completion state (nvim_get_compl_direction already declared above)

    // xmalloc / FreeWild
    #[link_name = "FreeWild"]
    fn FreeWild(count: c_int, files: *mut *mut c_char);
}

// Constants matching C definitions
const OK: c_int = 1;
const FORWARD: c_int = 1;
const CP_FAST: c_int = 32;
const CP_ICASE: c_int = 16;
// FUZZY_SCORE_NONE = INT_MIN (matches C fuzzy.h definition)
const FUZZY_SCORE_NONE: c_int = c_int::MIN;

/// Get the next set of filename matching compl_pattern.
///
/// Rust port of C `get_next_filename_completion()`. Performs wildcard
/// expansion, optional fuzzy matching with score-sorted ordering, and adds
/// filename matches to the completion list.
///
/// # Safety
/// Must be called from insert mode with valid completion state.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_get_next_filename_completion() {
    let mut matches: *mut *mut c_char = std::ptr::null_mut();
    let mut num_matches: c_int = 0;

    let leader = rs_ins_compl_leader().cast_mut();
    let leader_len = rs_ins_compl_leader_len();
    let mut in_fuzzy_collect = rs_cot_fuzzy() != 0 && leader_len > 0;
    let need_collect_bests = in_fuzzy_collect && crate::vars::nvim_get_compl_get_longest() != 0;
    let mut max_score: c_int = 0;
    let mut dir = crate::vars::nvim_get_compl_direction();

    // On Linux/non-Windows, pathsep is always '/'.
    // BACKSLASH_IN_FILENAME blocks are Windows-only dead code on this platform.
    let pathsep = b'/' as c_char;

    let mut fuzzy_leader: *mut c_char = leader;
    let mut fuzzy_leader_len = leader_len;

    if in_fuzzy_collect {
        // Find last path separator in leader
        let mut last_sep: *const c_char = std::ptr::null();
        let mut ptr: *mut c_char = leader;
        while *ptr != 0 {
            if *ptr == pathsep {
                last_sep = ptr.cast_const();
            }
            ptr = ptr.add(1);
        }

        if last_sep.is_null() {
            // No path separator: fuzzy match the whole leader against "*"
            // nvim_compl_pattern_set_star inlined (Phase 30)
            crate::vars::nvim_compl_clear_pattern();
            crate::vars::compl_pattern = cbuf_to_string_file(c"*".as_ptr(), 1);
        } else if *last_sep.add(1) == 0 {
            // Separator is the last char: can't do fuzzy on empty file part
            in_fuzzy_collect = false;
        } else {
            // Split leader into path + file parts: "path/*"
            let path_len = (last_sep as usize - leader as usize) + 1; // includes sep
            let buf_size = path_len + 2; // "path/*\0"
            let buf: *mut c_char = xmalloc(buf_size).cast();

            // Build "path_prefix*" string: copy path_len bytes then append '*'
            std::ptr::copy_nonoverlapping(leader.cast::<u8>(), buf.cast::<u8>(), path_len);
            *buf.add(path_len) = b'*' as c_char;
            *buf.add(path_len + 1) = 0;

            // Transfer ownership to compl_pattern
            crate::vars::nvim_compl_pattern_set_from_alloc(buf, path_len + 1);

            // Advance leader to the file part
            fuzzy_leader = last_sep.add(1).cast_mut();
            fuzzy_leader_len = leader_len - path_len;
        }
    }

    // Expand wildcards using compl_pattern.data
    let pat_data = crate::vars::nvim_compl_pattern_get_data();
    if pat_data.is_null() {
        return;
    }

    // pat_data is already *mut c_char (from nvim_compl_pattern_get_data)
    let mut pat_ptr = pat_data;
    if nvim_expand_wildcards_files(
        1,
        std::ptr::addr_of_mut!(pat_ptr),
        std::ptr::addr_of_mut!(num_matches),
        std::ptr::addr_of_mut!(matches),
    ) != OK
    {
        return;
    }

    // May change home directory back to "~"
    nvim_tilde_replace_wrap(pat_ptr, num_matches, matches);
    // (BACKSLASH_IN_FILENAME path conversion omitted -- Linux only)

    if in_fuzzy_collect {
        // Collect fuzzy-matching indices with their scores, then sort descending.
        let mut fuzzy_indices: Vec<c_int> = Vec::new();
        let mut fuzzy_scores: Vec<c_int> = vec![0; num_matches as usize];

        for (i, score_slot) in fuzzy_scores.iter_mut().enumerate() {
            let ptr = *matches.add(i);
            let score = fuzzy_match_str(ptr, fuzzy_leader.cast_const());
            if score != FUZZY_SCORE_NONE {
                fuzzy_indices.push(i as c_int);
                *score_slot = score;
            }
        }

        if !fuzzy_indices.is_empty() {
            // Sort descending by score; ascending by index on ties
            let scores = &fuzzy_scores;
            fuzzy_indices.sort_unstable_by(|&a, &b| {
                let sa = scores[a as usize];
                let sb = scores[b as usize];
                // descending score; ascending index for ties
                sb.cmp(&sa).then_with(|| a.cmp(&b))
            });

            let flags = CP_FAST
                | if crate::vars::nvim_get_p_fic_or_wic() != 0 {
                    CP_ICASE
                } else {
                    0
                };
            for (i, &fidx) in fuzzy_indices.iter().enumerate() {
                let idx = fidx as usize;
                let match_str = *matches.add(idx);
                let current_score = fuzzy_scores[idx];
                if rs_ins_compl_add(
                    match_str.cast_const(),
                    -1,
                    std::ptr::null(),
                    std::ptr::null(),
                    0,
                    std::ptr::null(),
                    dir,
                    flags,
                    0,
                    std::ptr::null(),
                    current_score,
                ) == OK
                {
                    dir = FORWARD;
                }

                if need_collect_bests && (i == 0 || current_score == max_score) {
                    crate::vars::nvim_set_compl_num_bests(
                        crate::vars::nvim_get_compl_num_bests() + 1,
                    );
                    max_score = current_score;
                }
            }

            FreeWild(num_matches, matches);
        } else if fuzzy_leader_len > 0 {
            FreeWild(num_matches, matches);
        }

        if crate::vars::nvim_get_compl_num_bests() > 0
            && crate::vars::nvim_get_compl_get_longest() != 0
        {
            rs_fuzzy_longest_match();
        }
        return;
    }

    if num_matches > 0 {
        rs_ins_compl_add_matches(num_matches, matches, crate::vars::nvim_get_p_fic_or_wic());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(CTRL_X_FILES, 4);
        assert_eq!(CTRL_X_PATH_PATTERNS, 6 + 0x100);
        assert_eq!(CTRL_X_PATH_DEFINES, 7 + 0x100);
    }

    #[test]
    fn test_find_last_sep() {
        unsafe {
            let path1 = b"/home/user/file.txt\0";
            let path2 = b"file.txt\0";
            let path3 = b"/\0";

            assert_eq!(rs_file_find_last_sep(path1.as_ptr().cast::<c_char>()), 10);
            assert_eq!(rs_file_find_last_sep(path2.as_ptr().cast::<c_char>()), -1);
            assert_eq!(rs_file_find_last_sep(path3.as_ptr().cast::<c_char>()), 0);
        }
    }

    #[test]
    fn test_dir_len() {
        unsafe {
            let path1 = b"/home/user/file.txt\0";
            let path2 = b"file.txt\0";

            assert_eq!(rs_file_dir_len(path1.as_ptr().cast::<c_char>()), 11);
            assert_eq!(rs_file_dir_len(path2.as_ptr().cast::<c_char>()), 0);
        }
    }

    #[test]
    fn test_basename_len() {
        unsafe {
            let path1 = b"/home/user/file.txt\0";
            let path2 = b"file.txt\0";

            assert_eq!(rs_file_basename_len(path1.as_ptr().cast::<c_char>()), 8);
            assert_eq!(rs_file_basename_len(path2.as_ptr().cast::<c_char>()), 8);
        }
    }

    #[test]
    fn test_is_absolute() {
        unsafe {
            let abs_unix = b"/home/user\0";
            let abs_win = b"C:\\Users\0";
            let relative = b"./file.txt\0";

            assert_eq!(rs_file_is_absolute(abs_unix.as_ptr().cast::<c_char>()), 1);
            assert_eq!(rs_file_is_absolute(abs_win.as_ptr().cast::<c_char>()), 1);
            assert_eq!(rs_file_is_absolute(relative.as_ptr().cast::<c_char>()), 0);
        }
    }

    #[test]
    fn test_ends_with_sep() {
        unsafe {
            let with_sep = b"/home/user/\0";
            let without = b"/home/user\0";

            assert_eq!(rs_file_ends_with_sep(with_sep.as_ptr().cast::<c_char>()), 1);
            assert_eq!(rs_file_ends_with_sep(without.as_ptr().cast::<c_char>()), 0);
        }
    }

    #[test]
    fn test_count_path_components() {
        unsafe {
            let path1 = b"/home/user/file.txt\0";
            let path2 = b"file.txt\0";
            let path3 = b"/\0";

            assert_eq!(
                rs_file_count_path_components(path1.as_ptr().cast::<c_char>()),
                3
            );
            assert_eq!(
                rs_file_count_path_components(path2.as_ptr().cast::<c_char>()),
                1
            );
            assert_eq!(
                rs_file_count_path_components(path3.as_ptr().cast::<c_char>()),
                0
            );
        }
    }
}
