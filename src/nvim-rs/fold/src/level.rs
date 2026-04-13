//! Fold level calculation functions for different fold methods.
//!
//! This module implements fold level calculation for indent, diff, expr, and
//! syntax fold methods. All four methods are now implemented directly in Rust.

use std::ffi::{c_char, c_int};

use nvim_buffer::BufHandle;
use nvim_window::win_struct::win_ref;
use nvim_window::WinHandle;

use crate::LineNr;

/// Maximum fold nesting level (matches C MAX_LEVEL constant).
const MAX_LEVEL: c_int = 20;

// C accessor functions
extern "C" {
    /// Get a line from a buffer.
    #[link_name = "ml_get_buf"]
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
    fn skipwhite(s: *const c_char) -> *const c_char;

    /// Find a character in a string (like vim_strchr).
    fn vim_strchr(s: *const c_char, c: c_int) -> *const c_char;

    /// Get the syntax fold level for a line.
    /// Wraps syn_get_foldlevel(wp, lnum).
    fn nvim_syn_get_foldlevel(wp: WinHandle, lnum: LineNr) -> c_int;

    /// Evaluate 'foldexpr' for window wp at line v:lnum.
    /// Returns the numeric value; sets *out_char to the prefix character
    /// ('a', 's', '>', '<', '=', or 0 for a plain integer).
    fn nvim_fold_eval_foldexpr(wp: WinHandle, out_char: *mut c_int) -> c_int;

    /// Save curwin/curbuf and set them to wp/wp->w_buffer.
    /// Returns old curwin (opaque win_T pointer).
    fn nvim_fold_save_curwin(wp: WinHandle) -> WinHandle;

    /// Restore curwin/curbuf from saved_win (returned by nvim_fold_save_curwin).
    fn nvim_fold_restore_curwin(saved_win: WinHandle);

    /// Get and save the KeyTyped global (returns its current value).
    fn nvim_fold_get_keytyped() -> c_int;

    /// Restore the KeyTyped global to saved value.
    fn nvim_fold_set_keytyped(val: c_int);

    /// Set v:lnum vim variable to lnum.
    fn nvim_fold_set_vim_var_nr_lnum(lnum: LineNr);

    /// Get the line count of curbuf (after curwin/curbuf have been set).
    fn nvim_fold_get_curbuf_line_count() -> LineNr;
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

/// Calculate fold level using the "indent" method, returning full result.
///
/// Returns -1 in `lvl` if the fold level depends on surrounding lines
/// (empty lines or lines starting with a character in 'foldignore').
///
/// # Arguments
/// * `wp` - Window handle
/// * `lnum` - Line number (1-based)
/// * `off` - Offset to add to lnum for actual buffer line
#[must_use]
pub fn foldlevel_indent_result(wp: WinHandle, lnum: LineNr, off: LineNr) -> FoldLevelResult {
    let mut result = FoldLevelResult {
        lvl: 0,
        lvl_next: -1,
        start: 0,
        end: MAX_LEVEL + 1,
    };

    if wp.is_null() {
        result.lvl = 0;
        result.lvl_next = -1;
        return result;
    }

    let actual_lnum = lnum + off;

    let buf = unsafe { BufHandle::from_ptr(win_ref(wp).w_buffer) };
    if buf.is_null() {
        result.lvl = 0;
        result.lvl_next = -1;
        return result;
    }

    let line_ptr = unsafe { nvim_ml_get_buf(buf, actual_lnum) };
    if line_ptr.is_null() {
        result.lvl = 0;
        result.lvl_next = -1;
        return result;
    }

    // Skip whitespace to check if line is empty or starts with foldignore char
    let s = unsafe { skipwhite(line_ptr) };

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
            #[allow(clippy::cast_sign_loss)]
            let char_val = c_int::from(*s as u8);
            if !fdi.is_null() && !vim_strchr(fdi, char_val).is_null() {
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
    let lvl = lvl.min(max_level);

    // For indent method, lvl_next is the same as lvl (not modified by level getter)
    result.lvl = lvl;
    result.lvl_next = lvl;
    result
}

/// Calculate fold level using the "diff" method, returning full result.
///
/// Lines in a diff fold get level 1, others get level 0.
///
/// # Arguments
/// * `wp` - Window handle
/// * `lnum` - Line number (1-based)
/// * `off` - Offset to add to lnum for actual buffer line
#[must_use]
pub fn foldlevel_diff_result(wp: WinHandle, lnum: LineNr, off: LineNr) -> FoldLevelResult {
    let mut result = FoldLevelResult {
        lvl: 0,
        lvl_next: -1,
        start: 0,
        end: MAX_LEVEL + 1,
    };

    if wp.is_null() {
        return result;
    }

    let actual_lnum = lnum + off;
    let in_fold = unsafe { rs_diff_infold(wp, actual_lnum) };
    let lvl = c_int::from(in_fold);
    result.lvl = lvl;
    result.lvl_next = lvl;
    result
}

/// Calculate fold level using the "expr" method, returning full result.
///
/// This evaluates the window's 'foldexpr' option and interprets the result
/// according to the fold expression protocol (a, s, >, <, =, or plain int).
///
/// # Arguments
/// * `wp` - Window handle
/// * `lnum` - Line number (1-based)
/// * `off` - Offset to add to lnum for actual buffer line
/// * `current_lvl` - The current fold level (needed for 'a' and 's' codes)
#[must_use]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
pub fn foldlevel_expr_result(
    wp: WinHandle,
    lnum: LineNr,
    off: LineNr,
    current_lvl: c_int,
) -> FoldLevelResult {
    let actual_lnum = lnum + off;

    let mut result = FoldLevelResult {
        lvl: current_lvl,
        lvl_next: -1,
        start: 0,
        end: MAX_LEVEL + 1,
    };

    // Set start = 0, had_end is used externally
    result.start = 0;
    if actual_lnum <= 1 {
        result.lvl = 0;
    }

    // Save curwin/curbuf and set to wp/wp->w_buffer
    let saved_win = unsafe { nvim_fold_save_curwin(wp) };

    // Set v:lnum
    unsafe { nvim_fold_set_vim_var_nr_lnum(actual_lnum) };

    // Save KeyTyped (may be reset by do_cmdline during foldexpr evaluation)
    let saved_keytyped = unsafe { nvim_fold_get_keytyped() };

    // Evaluate foldexpr
    let mut c: c_int = 0;
    let n = unsafe { nvim_fold_eval_foldexpr(wp, &raw mut c) };

    // Restore KeyTyped
    unsafe { nvim_fold_set_keytyped(saved_keytyped) };

    // Interpret the result
    match c as u8 {
        // "a1", "a2", .. : add to the fold level
        b'a' => {
            if result.lvl >= 0 {
                result.lvl += n;
                result.lvl_next = result.lvl;
            }
            result.start = n;
        }

        // "s1", "s2", .. : subtract from the fold level
        b's' => {
            if result.lvl >= 0 {
                result.lvl_next = if n > result.lvl { 0 } else { result.lvl - n };
                result.end = result.lvl_next + 1;
            }
        }

        // ">1", ">2", .. : start a fold with a certain level
        b'>' => {
            result.lvl = n;
            result.lvl_next = n;
            result.start = 1;
        }

        // "<1", "<2", .. : end a fold with a certain level
        b'<' => {
            // To prevent an unexpected start of a new fold, the next
            // level must not exceed the level of the current fold.
            result.lvl_next = result.lvl.min(n - 1);
            result.end = n;
        }

        // "=": No change in level
        b'=' => {
            result.lvl_next = result.lvl;
        }

        // "-1", "0", "1", ..: set fold level
        _ => {
            if n < 0 {
                // Use the current level for the next line
                result.lvl_next = result.lvl;
            } else {
                result.lvl_next = n;
            }
            result.lvl = n;
        }
    }

    // If the level is unknown for the first or the last line in the file, use level 0.
    if result.lvl < 0 {
        if actual_lnum <= 1 {
            result.lvl = 0;
            result.lvl_next = 0;
        }
        let line_count = unsafe { nvim_fold_get_curbuf_line_count() };
        if actual_lnum == line_count {
            result.lvl_next = 0;
        }
    }

    // Restore curwin/curbuf
    unsafe { nvim_fold_restore_curwin(saved_win) };

    result
}

/// Calculate fold level using the "syntax" method, returning full result.
///
/// Uses the maximum fold level at the start of this line and the next.
///
/// # Arguments
/// * `wp` - Window handle
/// * `lnum` - Line number (1-based)
/// * `off` - Offset to add to lnum for actual buffer line
#[must_use]
pub fn foldlevel_syntax_result(wp: WinHandle, lnum: LineNr, off: LineNr) -> FoldLevelResult {
    let actual_lnum = lnum + off;

    let mut result = FoldLevelResult {
        lvl: 0,
        lvl_next: -1,
        start: 0,
        end: MAX_LEVEL + 1,
    };

    if wp.is_null() {
        return result;
    }

    // Use the maximum fold level at the start of this line and the next.
    let lvl = unsafe { nvim_syn_get_foldlevel(wp, actual_lnum) };
    result.lvl = lvl;
    result.start = 0;

    let line_count = unsafe { nvim_win_get_buf_line_count(wp) };
    if actual_lnum < line_count {
        let n = unsafe { nvim_syn_get_foldlevel(wp, actual_lnum + 1) };
        if n > lvl {
            result.start = n - lvl; // fold(s) start here
            result.lvl = n;
        }
    }

    result.lvl_next = result.lvl;
    result
}

// ============================================================================
// FFI Exports (kept for backward compatibility; update.rs calls Rust directly)
// ============================================================================

/// Calculate fold level for a line using the "indent" method.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldlevelIndent(wp: WinHandle, lnum: LineNr, off: LineNr) -> c_int {
    foldlevel_indent_result(wp, lnum, off).lvl
}

/// Calculate fold level for a line using the "diff" method.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldlevelDiff(wp: WinHandle, lnum: LineNr, off: LineNr) -> c_int {
    foldlevel_diff_result(wp, lnum, off).lvl
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

    // Note: FFI-dependent tests (foldlevel_*_result) require the full neovim
    // binary to be linked and cannot be run in isolation.
    // They are tested through integration tests instead.
}
