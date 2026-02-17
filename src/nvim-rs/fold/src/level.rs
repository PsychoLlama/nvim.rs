//! Fold level calculation functions for different fold methods.
//!
//! This module implements fold level calculation for indent and diff methods.
//! The expr and syntax methods require complex VimL evaluation and syntax
//! highlighting integration, so they remain in C.

use std::ffi::{c_char, c_int};

use nvim_buffer::BufHandle;
use nvim_window::WinHandle;

use crate::LineNr;

// C accessor functions
extern "C" {
    /// Get the buffer from a window.
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;

    /// Get a line from a buffer.
    fn nvim_ml_get_buf(buf: BufHandle, lnum: LineNr) -> *const c_char;

    /// Get the line count of the window's buffer.
    fn nvim_win_get_buf_line_count(wp: WinHandle) -> LineNr;

    /// Get the w_p_fdi (foldignore option) field from a window.
    fn nvim_win_get_p_fdi(wp: WinHandle) -> *const c_char;

    /// Get the w_p_fdn (foldnestmax option) field from a window.
    fn nvim_win_get_p_fdn(wp: WinHandle) -> c_int;

    /// Get the indentation of a buffer line.
    fn nvim_get_indent_buf(buf: BufHandle, lnum: LineNr) -> c_int;

    /// Get the shiftwidth value for a buffer.
    fn nvim_get_sw_value(buf: BufHandle) -> c_int;

    /// Check if a line is in a diff fold.
    fn rs_diff_infold(wp: WinHandle, lnum: LineNr) -> bool;

    /// Skip whitespace at the beginning of a string.
    fn nvim_skipwhite(s: *const c_char) -> *const c_char;

    /// Find a character in a string (like vim_strchr).
    fn nvim_vim_strchr(s: *const c_char, c: c_int) -> *const c_char;
}

/// Result of fold level calculation for a line.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FoldLevelResult {
    /// Current fold level (-1 means depends on surrounding lines).
    pub lvl: c_int,
    /// Fold level for the next line.
    pub lvl_next: c_int,
    /// Number of folds that start at this line.
    pub start: c_int,
    /// Level of fold forced to end below this line.
    /// Set by expr method ('s' and '<' codes). Default: MAX_LEVEL + 1.
    pub end: c_int,
}

/// Calculate fold level using the "indent" method.
///
/// Returns -1 if the fold level depends on surrounding lines (empty lines or
/// lines starting with a character in 'foldignore').
///
/// # Arguments
/// * `wp` - Window handle
/// * `lnum` - Line number to check (1-based)
/// * `off` - Offset to add to lnum for actual buffer line
///
/// # Returns
/// Fold level for the line, or -1 if it depends on surrounding lines.
fn foldlevel_indent_impl(wp: WinHandle, lnum: LineNr, off: LineNr) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let actual_lnum = lnum + off;

    let buf = unsafe { nvim_win_get_buffer(wp) };
    if buf.is_null() {
        return 0;
    }

    let line_ptr = unsafe { nvim_ml_get_buf(buf, actual_lnum) };
    if line_ptr.is_null() {
        return 0;
    }

    // Skip whitespace to check if line is empty or starts with foldignore char
    let s = unsafe { nvim_skipwhite(line_ptr) };

    let lvl = unsafe {
        // Empty line check - first char is NUL
        if *s == 0 {
            // First and last line can't be undefined, use level 0
            let line_count = nvim_win_get_buf_line_count(wp);
            if actual_lnum == 1 || actual_lnum == line_count {
                0
            } else {
                -1 // Depends on surrounding lines
            }
        } else {
            // Check if line starts with a character in 'foldignore'
            let fdi = nvim_win_get_p_fdi(wp);
            // Cast c_char to c_int for vim_strchr - this is safe as we're just
            // looking up the character value
            #[allow(clippy::cast_sign_loss)]
            let char_val = c_int::from(*s as u8);
            if !fdi.is_null() && !nvim_vim_strchr(fdi, char_val).is_null() {
                // First and last line can't be undefined, use level 0
                let line_count = nvim_win_get_buf_line_count(wp);
                if actual_lnum == 1 || actual_lnum == line_count {
                    0
                } else {
                    -1 // Depends on surrounding lines
                }
            } else {
                // Calculate level from indentation
                let indent = nvim_get_indent_buf(buf, actual_lnum);
                let sw = nvim_get_sw_value(buf);
                if sw > 0 {
                    indent / sw
                } else {
                    0
                }
            }
        }
    };

    // Clamp to foldnestmax
    let fdn = unsafe { nvim_win_get_p_fdn(wp) };
    let max_level = if fdn > 0 { fdn } else { 0 };
    lvl.min(max_level)
}

/// Calculate fold level using the "diff" method.
///
/// Lines in a diff fold get level 1, others get level 0.
///
/// # Arguments
/// * `wp` - Window handle
/// * `lnum` - Line number to check (1-based)
/// * `off` - Offset to add to lnum for actual buffer line
///
/// # Returns
/// 1 if the line is in a diff fold, 0 otherwise.
fn foldlevel_diff_impl(wp: WinHandle, lnum: LineNr, off: LineNr) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let actual_lnum = lnum + off;

    let in_fold = unsafe { rs_diff_infold(wp, actual_lnum) };
    c_int::from(in_fold)
}

// ============================================================================
// FFI Exports
// ============================================================================

/// Calculate fold level for a line using the "indent" method.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldlevelIndent(wp: WinHandle, lnum: LineNr, off: LineNr) -> c_int {
    foldlevel_indent_impl(wp, lnum, off)
}

/// Calculate fold level for a line using the "diff" method.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldlevelDiff(wp: WinHandle, lnum: LineNr, off: LineNr) -> c_int {
    foldlevel_diff_impl(wp, lnum, off)
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // FoldLevelResult Tests
    // =========================================================================

    #[test]
    fn test_fold_level_result_default() {
        let result = FoldLevelResult::default();
        assert_eq!(result.lvl, 0);
        assert_eq!(result.lvl_next, 0);
        assert_eq!(result.start, 0);
    }

    #[test]
    fn test_fold_level_result_clone() {
        let result = FoldLevelResult {
            lvl: 2,
            lvl_next: 3,
            start: 1,
            end: 21,
        };
        let cloned = result;
        assert_eq!(cloned.lvl, 2);
        assert_eq!(cloned.lvl_next, 3);
        assert_eq!(cloned.start, 1);
    }

    #[test]
    fn test_fold_level_result_copy() {
        let result = FoldLevelResult {
            lvl: 5,
            lvl_next: 6,
            start: 2,
            end: 21,
        };
        let copied = result;
        // Original should still be valid (Copy trait)
        assert_eq!(result.lvl, 5);
        assert_eq!(copied.lvl, 5);
    }

    #[test]
    fn test_fold_level_result_size() {
        // Should be 4 * c_int (16 bytes on most platforms)
        assert_eq!(
            std::mem::size_of::<FoldLevelResult>(),
            4 * std::mem::size_of::<c_int>()
        );
    }

    #[test]
    fn test_fold_level_result_repr_c() {
        // Verify it's suitable for C FFI by checking alignment
        assert!(std::mem::align_of::<FoldLevelResult>() <= std::mem::align_of::<c_int>() * 2);
    }

    #[test]
    fn test_fold_level_result_negative_lvl() {
        // -1 is a valid value meaning "depends on surrounding lines"
        let result = FoldLevelResult {
            lvl: -1,
            lvl_next: 0,
            start: 0,
            end: 21,
        };
        assert_eq!(result.lvl, -1);
    }

    #[test]
    fn test_fold_level_result_debug() {
        let result = FoldLevelResult {
            lvl: 1,
            lvl_next: 2,
            start: 0,
            end: 21,
        };
        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("lvl: 1"));
        assert!(debug_str.contains("lvl_next: 2"));
        assert!(debug_str.contains("start: 0"));
    }

    #[test]
    fn test_fold_level_result_high_values() {
        // Test with high fold levels (edge case)
        let result = FoldLevelResult {
            lvl: 100,
            lvl_next: 101,
            start: 50,
            end: 21,
        };
        assert_eq!(result.lvl, 100);
        assert_eq!(result.lvl_next, 101);
        assert_eq!(result.start, 50);
    }

    #[test]
    fn test_fold_level_result_zero_values() {
        // Explicit zero initialization
        let result = FoldLevelResult {
            lvl: 0,
            lvl_next: 0,
            start: 0,
            end: 21,
        };
        assert_eq!(result.lvl, 0);
        assert_eq!(result.lvl_next, 0);
        assert_eq!(result.start, 0);
    }

    // Note: FFI-dependent tests (foldlevel_indent_impl, foldlevel_diff_impl)
    // require the full neovim binary to be linked and cannot be run in isolation.
    // They are tested through integration tests instead.
}
