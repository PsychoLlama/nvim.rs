//! Fold marker parsing and level calculation for marker-based folding.
//!
//! This module implements the fold marker system used when 'foldmethod' is "marker".
//! Fold markers are special text patterns (default `{{{` and `}}}`) that define
//! fold boundaries in the buffer text.

use std::ffi::{c_char, c_int};
use std::ptr;

use nvim_buffer::BufHandle;
use nvim_window::WinHandle;

use crate::LineNr;

/// Maximum fold level supported by Neovim.
pub const MAX_LEVEL: c_int = 20;

/// Parsed fold marker information.
///
/// Contains the parsed start and end markers along with their lengths.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldMarkerInfo {
    /// Pointer to the start marker string (from w_p_fmr).
    pub start_marker: *const c_char,
    /// Length of the start marker (excluding optional digit).
    pub start_marker_len: usize,
    /// Pointer to the end marker string (after the comma in w_p_fmr).
    pub end_marker: *const c_char,
    /// Length of the end marker (excluding optional digit).
    pub end_marker_len: usize,
}

impl Default for FoldMarkerInfo {
    fn default() -> Self {
        Self {
            start_marker: ptr::null(),
            start_marker_len: 0,
            end_marker: ptr::null(),
            end_marker_len: 0,
        }
    }
}

// C accessor functions
extern "C" {
    /// Get the w_p_fmr (foldmarker option) field from a window.
    fn nvim_win_get_p_fmr(wp: WinHandle) -> *const c_char;

    /// Get the buffer from a window.
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;

    /// Get a line from a buffer.
    fn nvim_ml_get_buf(buf: BufHandle, lnum: LineNr) -> *const c_char;
}

/// Parse the 'foldmarker' option into start and end markers.
///
/// The 'foldmarker' option has the format "startmarker,endmarker" (e.g., "{{{,}}}").
/// This function locates the comma separator and calculates the lengths.
///
/// Returns a `FoldMarkerInfo` struct with pointers into the original option string.
/// The pointers are only valid as long as the window's foldmarker option is unchanged.
#[must_use]
pub fn parse_marker_impl(wp: WinHandle) -> FoldMarkerInfo {
    if wp.is_null() {
        return FoldMarkerInfo::default();
    }

    let fmr = unsafe { nvim_win_get_p_fmr(wp) };
    if fmr.is_null() {
        return FoldMarkerInfo::default();
    }

    // Find the comma separator
    let mut end_marker = fmr;
    unsafe {
        while *end_marker != 0 && *end_marker != b',' as c_char {
            end_marker = end_marker.add(1);
        }

        if *end_marker != b',' as c_char {
            // No comma found, invalid format - return default
            return FoldMarkerInfo::default();
        }

        // SAFETY: We know end_marker >= fmr since we started from fmr and only moved forward.
        // The result is always non-negative.
        #[allow(clippy::cast_sign_loss)]
        let start_marker_len = end_marker.offset_from(fmr) as usize;

        // Move past the comma
        end_marker = end_marker.add(1);

        // Calculate end marker length
        let end_marker_len = if *end_marker == 0 {
            0
        } else {
            // Use strlen equivalent
            let mut len = 0;
            let mut p = end_marker;
            while *p != 0 {
                len += 1;
                p = p.add(1);
            }
            len
        };

        FoldMarkerInfo {
            start_marker: fmr,
            start_marker_len,
            end_marker,
            end_marker_len,
        }
    }
}

/// Result of fold level calculation for a line.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldLevelMarkerResult {
    /// Current fold level.
    pub lvl: c_int,
    /// Fold level for the next line.
    pub lvl_next: c_int,
    /// Number of folds that start at this line (>0 means fold(s) start here).
    pub start: c_int,
}

/// Calculate fold level for a line using marker method.
///
/// This is the low-level function to get the foldlevel for the "marker" method.
/// It scans the line for fold markers and updates the fold level accordingly.
///
/// # Arguments
/// * `wp` - Window handle
/// * `lnum` - Line number to check (1-based)
/// * `off` - Offset to add to lnum for actual buffer line
/// * `current_lvl` - Current fold level (from previous line)
/// * `marker_info` - Parsed fold marker information
///
/// # Returns
/// `FoldLevelMarkerResult` with updated levels and start count.
///
/// # Note
/// Requires that `current_lvl` is set to the fold level of the previous line!
/// This means you can't call this function twice on the same line without
/// passing the updated level.
#[must_use]
pub fn foldlevel_marker_impl(
    wp: WinHandle,
    lnum: LineNr,
    off: LineNr,
    current_lvl: c_int,
    marker_info: &FoldMarkerInfo,
) -> FoldLevelMarkerResult {
    if wp.is_null()
        || marker_info.start_marker.is_null()
        || marker_info.end_marker.is_null()
        || marker_info.start_marker_len == 0
    {
        return FoldLevelMarkerResult {
            lvl: current_lvl,
            lvl_next: current_lvl,
            start: 0,
        };
    }

    let buf = unsafe { nvim_win_get_buffer(wp) };
    if buf.is_null() {
        return FoldLevelMarkerResult {
            lvl: current_lvl,
            lvl_next: current_lvl,
            start: 0,
        };
    }

    let line_ptr = unsafe { nvim_ml_get_buf(buf, lnum + off) };
    if line_ptr.is_null() {
        return FoldLevelMarkerResult {
            lvl: current_lvl,
            lvl_next: current_lvl,
            start: 0,
        };
    }

    let start_lvl = current_lvl;
    let mut lvl = current_lvl;
    let mut lvl_next = current_lvl;
    let mut start = 0;

    unsafe {
        // Cache marker info for speed
        let start_marker = marker_info.start_marker;
        let start_marker_len = marker_info.start_marker_len;
        let end_marker = marker_info.end_marker;
        let end_marker_len = marker_info.end_marker_len;

        // Get first characters of markers for quick comparison
        let cstart = *start_marker;
        // Start marker comparison begins from second character
        let start_marker_rest = start_marker.add(1);
        let cend = *end_marker;

        let mut s = line_ptr;
        while *s != 0 {
            if *s == cstart && start_marker_len > 0 {
                // Check if this is a start marker
                let matches = if start_marker_len == 1 {
                    true
                } else {
                    // Compare rest of marker (excluding first char we already matched)
                    strncmp_impl(s.add(1), start_marker_rest, start_marker_len - 1)
                };

                if matches {
                    // Found start marker
                    s = s.add(start_marker_len);

                    // Check for optional digit
                    if is_ascii_digit(*s) {
                        let n = atoi_impl(s);
                        if n > 0 {
                            lvl = n;
                            lvl_next = n;
                            start = (n - start_lvl).max(1);
                        }
                    } else {
                        lvl += 1;
                        lvl_next += 1;
                        start += 1;
                    }
                    continue;
                }
            }

            if *s == cend && end_marker_len > 0 {
                // Check if this is an end marker
                let matches = if end_marker_len == 1 {
                    true
                } else {
                    strncmp_impl(s.add(1), end_marker.add(1), end_marker_len - 1)
                };

                if matches {
                    // Found end marker
                    s = s.add(end_marker_len);

                    // Check for optional digit
                    if is_ascii_digit(*s) {
                        let n = atoi_impl(s);
                        if n > 0 {
                            lvl = n;
                            lvl_next = n - 1;
                            // Never start a fold with an end marker
                            lvl_next = lvl_next.min(start_lvl);
                        }
                    } else {
                        lvl_next -= 1;
                    }
                    continue;
                }
            }

            // Advance to next character (handle multi-byte)
            s = mb_ptr_adv(s);
        }
    }

    // The level can't go negative, must be missing a start marker.
    lvl_next = lvl_next.max(0);

    FoldLevelMarkerResult {
        lvl,
        lvl_next,
        start,
    }
}

/// Check if a character is an ASCII digit.
#[inline]
const fn is_ascii_digit(c: c_char) -> bool {
    c >= b'0' as c_char && c <= b'9' as c_char
}

/// Simple atoi implementation for parsing fold level digits.
unsafe fn atoi_impl(s: *const c_char) -> c_int {
    let mut result: c_int = 0;
    let mut p = s;
    while is_ascii_digit(*p) {
        result = result * 10 + c_int::from((*p) - b'0' as c_char);
        p = p.add(1);
    }
    result
}

/// Compare two strings up to n characters.
#[inline]
unsafe fn strncmp_impl(s1: *const c_char, s2: *const c_char, n: usize) -> bool {
    for i in 0..n {
        let c1 = *s1.add(i);
        let c2 = *s2.add(i);
        if c1 != c2 {
            return false;
        }
        if c1 == 0 {
            return true;
        }
    }
    true
}

/// Advance pointer past one (possibly multi-byte) character.
/// For simplicity, this treats each byte as a character.
/// The full implementation would use MB_PTR_ADV from mbyte.h.
#[inline]
const unsafe fn mb_ptr_adv(s: *const c_char) -> *const c_char {
    if (*s) == 0 {
        s
    } else {
        s.add(1)
    }
}

// ============================================================================
// FFI Exports
// ============================================================================

/// Parse the 'foldmarker' option into start and end markers.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_parseMarker(wp: WinHandle) -> FoldMarkerInfo {
    parse_marker_impl(wp)
}

/// Calculate fold level for a line using marker method.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
/// The `marker_info` must be a valid pointer to a FoldMarkerInfo struct.
#[no_mangle]
pub unsafe extern "C" fn rs_foldlevelMarker(
    wp: WinHandle,
    lnum: LineNr,
    off: LineNr,
    current_lvl: c_int,
    marker_info: *const FoldMarkerInfo,
) -> FoldLevelMarkerResult {
    if marker_info.is_null() {
        return FoldLevelMarkerResult {
            lvl: current_lvl,
            lvl_next: current_lvl,
            start: 0,
        };
    }
    foldlevel_marker_impl(wp, lnum, off, current_lvl, &*marker_info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ascii_digit() {
        assert!(is_ascii_digit(b'0' as c_char));
        assert!(is_ascii_digit(b'5' as c_char));
        assert!(is_ascii_digit(b'9' as c_char));
        assert!(!is_ascii_digit(b'a' as c_char));
        assert!(!is_ascii_digit(b' ' as c_char));
        assert!(!is_ascii_digit(0));
    }

    #[test]
    fn test_atoi_impl() {
        unsafe {
            let s = c"123".as_ptr();
            assert_eq!(atoi_impl(s), 123);

            let s = c"0".as_ptr();
            assert_eq!(atoi_impl(s), 0);

            let s = c"42abc".as_ptr();
            assert_eq!(atoi_impl(s), 42);
        }
    }

    #[test]
    fn test_strncmp_impl() {
        unsafe {
            let s1 = c"hello".as_ptr();
            let s2 = c"hello".as_ptr();
            assert!(strncmp_impl(s1, s2, 5));

            let s1 = c"hello".as_ptr();
            let s2 = c"world".as_ptr();
            assert!(!strncmp_impl(s1, s2, 5));

            let s1 = c"hello".as_ptr();
            let s2 = c"help".as_ptr();
            assert!(strncmp_impl(s1, s2, 3));
            assert!(!strncmp_impl(s1, s2, 4));
        }
    }
}
