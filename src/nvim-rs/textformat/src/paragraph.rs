//! Paragraph detection helpers for text formatting.
//!
//! This module provides functions to detect paragraph boundaries,
//! comment leaders, and whitespace patterns in text.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::{c_char, c_int};
use std::ptr;

// =============================================================================
// Constants
// =============================================================================

/// NUL character
const NUL: c_char = 0;

/// Comment flag: start of multi-line comment ('s')
const COM_START: c_char = b's' as c_char;

/// Comment flag: middle of multi-line comment ('m')
const COM_MIDDLE: c_char = b'm' as c_char;

/// Comment flag: end of multi-line comment ('e')
const COM_END: c_char = b'e' as c_char;

/// Comment flag: first line only ('f')
const COM_FIRST: c_char = b'f' as c_char;

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    /// Get line content at lnum.
    fn nvim_textfmt_ml_get(lnum: c_int) -> *const c_char;

    /// Get line length at lnum.
    fn nvim_textfmt_ml_get_len(lnum: c_int) -> c_int;

    /// Get comment leader length. Returns length of leader or 0.
    /// If flags is not NULL, it will point to the flags for the leader.
    fn nvim_textfmt_get_leader_len(
        line: *const c_char,
        flags: *mut *mut c_char,
        backward: bool,
        include_space: bool,
    ) -> c_int;

    /// Check if line starts a paragraph or section.
    fn nvim_textfmt_startPS(lnum: c_int, para: c_int, both: bool) -> bool;

    /// Check if format option is set (via has_format_option).
    #[link_name = "has_format_option"]
    fn rs_has_format_option(x: c_int) -> bool;

    /// Get number indent for a line. Returns indent or -1.
    fn nvim_textfmt_get_number_indent(lnum: c_int) -> c_int;
}

// =============================================================================
// Format Option Constants
// =============================================================================

/// Format option: format comments with 'q'
const FO_Q_COMS: c_int = b'q' as c_int;

/// Format option: trailing whitespace means paragraph continues
const FO_WHITE_PAR: c_int = b'w' as c_int;

/// Format option: numbered list
const FO_Q_NUMBER: c_int = b'n' as c_int;

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if a character is ASCII whitespace (space or tab).
#[inline]
const fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Check if format option 'x' is set.
#[inline]
fn has_format_option(x: c_int) -> bool {
    unsafe { rs_has_format_option(x) }
}

// =============================================================================
// Paragraph Detection Functions
// =============================================================================

/// Check if line `lnum` ends in a whitespace character.
///
/// Returns `true` if the last character of the line is a space or tab.
pub(crate) unsafe fn ends_in_white_impl(lnum: c_int) -> bool {
    let s = nvim_textfmt_ml_get(lnum);
    if s.is_null() || *s == NUL {
        return false;
    }

    let len = nvim_textfmt_ml_get_len(lnum);
    if len <= 0 {
        return false;
    }

    let last_char = *s.add((len - 1) as usize) as u8;
    ascii_iswhite(last_char)
}

/// Check if line is a paragraph boundary.
///
/// Blank lines and lines containing only the comment leader are left
/// untouched by formatting. This function returns `true` in such cases.
/// It also returns `true` when a line starts with the end of a comment
/// ('e' in comment flags), so that this line is skipped and not joined
/// to the previous line.
///
/// A new paragraph starts after a blank line, or when the comment leader changes.
///
/// # Arguments
/// * `lnum` - Line number to check
/// * `leader_len` - Output: length of comment leader
/// * `leader_flags` - Output: pointer to flags for the leader
/// * `do_comments` - Whether to consider comment leaders
///
/// # Returns
/// `true` if line is a paragraph boundary.
pub(crate) unsafe fn fmt_check_par_impl(
    lnum: c_int,
    leader_len: *mut c_int,
    leader_flags: *mut *mut c_char,
    do_comments: bool,
) -> bool {
    let ptr = nvim_textfmt_ml_get(lnum);

    if do_comments {
        *leader_len = nvim_textfmt_get_leader_len(ptr, leader_flags, false, true);
    } else {
        *leader_len = 0;
    }

    // Check for 'e' flag (end of comment) in leader flags
    let mut has_end_flag = false;
    if *leader_len > 0 {
        let flags = *leader_flags;
        if !flags.is_null() {
            let mut p = flags;
            while *p != NUL && *p != b':' as c_char && *p != COM_END {
                p = p.add(1);
            }
            has_end_flag = *p == COM_END;
        }
    }

    // Skip past the leader
    let after_leader = ptr.add(*leader_len as usize);

    // Skip whitespace after leader
    let mut p = after_leader;
    while *p != NUL && ascii_iswhite(*p as u8) {
        p = p.add(1);
    }

    // Return true if:
    // - Line is blank (only whitespace after leader)
    // - Line starts with end-of-comment marker
    // - Line starts a paragraph/section
    *p == NUL || has_end_flag || nvim_textfmt_startPS(lnum, 0, false)
}

/// Check if two comment leaders are the same.
///
/// Compares the comment leaders of two adjacent lines to determine
/// if they should be treated as part of the same paragraph.
///
/// # Arguments
/// * `lnum` - Line number of the first line
/// * `leader1_len` - Length of the first leader
/// * `leader1_flags` - Flags for the first leader
/// * `leader2_len` - Length of the second leader (on line lnum+1)
/// * `leader2_flags` - Flags for the second leader
///
/// # Returns
/// `true` if the leaders match (lines can be joined).
pub(crate) unsafe fn same_leader_impl(
    lnum: c_int,
    leader1_len: c_int,
    leader1_flags: *mut c_char,
    leader2_len: c_int,
    leader2_flags: *mut c_char,
) -> bool {
    if leader1_len == 0 {
        return leader2_len == 0;
    }

    // Check flags on the first leader
    if !leader1_flags.is_null() {
        let mut p = leader1_flags;
        while *p != NUL && *p != b':' as c_char {
            // 'f' flag: first line only - can join only if second has no leader
            if *p == COM_FIRST {
                return leader2_len == 0;
            }
            // 'e' flag: end of comment - can never join
            if *p == COM_END {
                return false;
            }
            // 's' flag: start of comment - need text after and 'm' flag on second
            if *p == COM_START {
                let line_len = nvim_textfmt_ml_get_len(lnum);
                if line_len <= leader1_len {
                    return false;
                }
                if leader2_flags.is_null() || leader2_len == 0 {
                    return false;
                }
                // Check if second line has 'm' (middle) flag
                let mut p2 = leader2_flags;
                while *p2 != NUL && *p2 != b':' as c_char {
                    if *p2 == COM_MIDDLE {
                        return true;
                    }
                    p2 = p2.add(1);
                }
                return false;
            }
            p = p.add(1);
        }
    }

    // Get the actual text of both lines and compare leaders
    // We need to copy line1 since ml_get can invalidate previous results
    let line1_ptr = nvim_textfmt_ml_get(lnum);
    let line1_len = nvim_textfmt_ml_get_len(lnum) as usize;

    // Allocate and copy line1
    let line1 = libc::malloc(line1_len + 1) as *mut c_char;
    if line1.is_null() {
        return false;
    }
    ptr::copy_nonoverlapping(line1_ptr, line1, line1_len + 1);

    // Now get line2 (this call may invalidate line1_ptr, but we have a copy)
    let line2 = nvim_textfmt_ml_get(lnum + 1);

    // Skip leading whitespace in line1
    let mut idx1: usize = 0;
    while ascii_iswhite(*line1.add(idx1) as u8) {
        idx1 += 1;
    }

    // Compare characters
    let mut idx2: usize = 0;
    while (idx2 as c_int) < leader2_len {
        let c2 = *line2.add(idx2);
        if !ascii_iswhite(c2 as u8) {
            let c1 = *line1.add(idx1);
            if c1 != c2 {
                break;
            }
            idx1 += 1;
        } else {
            // Skip whitespace in line1 as well
            while ascii_iswhite(*line1.add(idx1) as u8) {
                idx1 += 1;
            }
        }
        idx2 += 1;
    }

    libc::free(line1 as *mut libc::c_void);

    idx2 as c_int == leader2_len && idx1 as c_int == leader1_len
}

/// Check if a paragraph starts at line `lnum`.
///
/// Used for auto-formatting. Returns `true` when a paragraph starts
/// at the given line, `false` when the previous line is in the same paragraph.
///
/// A new paragraph starts:
/// - At the start of the file (lnum <= 1)
/// - After an empty line
/// - After a non-paragraph line (blank or comment-only)
/// - When the comment leader changes
/// - When 'w' is in formatoptions and the previous line doesn't end in whitespace
/// - When 'n' is in formatoptions and a numbered item starts
unsafe fn paragraph_start_impl(lnum: c_int) -> bool {
    // Start of file
    if lnum <= 1 {
        return true;
    }

    // After empty line
    let prev_line = nvim_textfmt_ml_get(lnum - 1);
    if prev_line.is_null() || *prev_line == NUL {
        return true;
    }

    let do_comments = has_format_option(FO_Q_COMS);

    // Check if previous line is not a paragraph line
    let mut leader_len: c_int = 0;
    let mut leader_flags: *mut c_char = ptr::null_mut();
    if fmt_check_par_impl(lnum - 1, &mut leader_len, &mut leader_flags, do_comments) {
        return true;
    }

    // Check if current line is not a paragraph line
    let mut next_leader_len: c_int = 0;
    let mut next_leader_flags: *mut c_char = ptr::null_mut();
    if fmt_check_par_impl(
        lnum,
        &mut next_leader_len,
        &mut next_leader_flags,
        do_comments,
    ) {
        return true;
    }

    // Check for missing trailing whitespace when 'w' is in formatoptions
    if has_format_option(FO_WHITE_PAR) && !ends_in_white_impl(lnum - 1) {
        return true;
    }

    // Check for numbered item when 'n' is in formatoptions
    if has_format_option(FO_Q_NUMBER) && nvim_textfmt_get_number_indent(lnum) > 0 {
        return true;
    }

    // Check if comment leader changed
    if !same_leader_impl(
        lnum - 1,
        leader_len,
        leader_flags,
        next_leader_len,
        next_leader_flags,
    ) {
        return true;
    }

    false
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if line `lnum` ends in a whitespace character.
///
/// # Safety
/// Accesses buffer memory via C functions.
#[export_name = "ends_in_white"]
pub unsafe extern "C" fn rs_ends_in_white(lnum: c_int) -> bool {
    ends_in_white_impl(lnum)
}

/// Check if line is a paragraph boundary.
///
/// # Safety
/// Accesses buffer memory via C functions.
#[export_name = "fmt_check_par"]
pub unsafe extern "C" fn rs_fmt_check_par(
    lnum: c_int,
    leader_len: *mut c_int,
    leader_flags: *mut *mut c_char,
    do_comments: c_int,
) -> c_int {
    c_int::from(fmt_check_par_impl(
        lnum,
        leader_len,
        leader_flags,
        do_comments != 0,
    ))
}

/// Check if two comment leaders are the same.
///
/// # Safety
/// Accesses buffer memory via C functions.
#[export_name = "same_leader"]
pub unsafe extern "C" fn rs_same_leader(
    lnum: c_int,
    leader1_len: c_int,
    leader1_flags: *mut c_char,
    leader2_len: c_int,
    leader2_flags: *mut c_char,
) -> c_int {
    c_int::from(same_leader_impl(
        lnum,
        leader1_len,
        leader1_flags,
        leader2_len,
        leader2_flags,
    ))
}

/// Check if a paragraph starts at line `lnum`.
///
/// # Safety
/// Accesses buffer memory via C functions.
#[export_name = "paragraph_start"]
pub unsafe extern "C" fn rs_paragraph_start(lnum: c_int) -> bool {
    paragraph_start_impl(lnum)
}

#[cfg(test)]
mod tests {
    // Integration testing is done via the full Neovim build.
}
