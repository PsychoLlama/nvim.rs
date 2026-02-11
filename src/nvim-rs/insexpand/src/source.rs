//! Completion source management.
//!
//! This module provides helper functions for managing completion sources
//! and the 'complete' option.

#![allow(clippy::missing_const_for_fn)]

use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_matches() -> c_int;
    fn nvim_curbuf_get_b_p_cpt() -> *const c_char;
    fn nvim_get_cpt_start_tv() -> u64;
    fn nvim_get_compl_timeout_ms() -> u64;
    fn nvim_set_compl_time_slice_expired(val: c_int);
    fn nvim_decay_compl_timeout();
    fn os_hrtime() -> u64;
}

// CTRL-X mode constants
const CTRL_X_NORMAL: c_int = 0;
const CTRL_X_NOT_DEFINED_YET: c_int = 1;

/// Check if completion needs to start scanning sources.
///
/// Returns true if completion has started but no matches found yet.
#[no_mangle]
pub unsafe extern "C" fn rs_source_needs_scan() -> c_int {
    let started = nvim_get_compl_started();
    let matches = nvim_get_compl_matches();
    c_int::from(started != 0 && matches == 0)
}

/// Check if we're in initial completion mode (before CTRL-X pressed).
#[no_mangle]
pub unsafe extern "C" fn rs_source_is_initial_mode() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    c_int::from(mode == CTRL_X_NORMAL || mode == CTRL_X_NOT_DEFINED_YET)
}

/// Check if completion has any matches.
#[no_mangle]
pub unsafe extern "C" fn rs_source_has_matches() -> c_int {
    let matches = nvim_get_compl_matches();
    c_int::from(matches > 0)
}

/// Get the current match count.
#[no_mangle]
pub unsafe extern "C" fn rs_source_match_count() -> c_int {
    nvim_get_compl_matches()
}

/// FFI export: Get CTRL_X_NORMAL constant.
#[no_mangle]
pub extern "C" fn rs_ctrl_x_normal() -> c_int {
    CTRL_X_NORMAL
}

/// FFI export: Get CTRL_X_NOT_DEFINED_YET constant.
#[no_mangle]
pub extern "C" fn rs_ctrl_x_not_defined_yet() -> c_int {
    CTRL_X_NOT_DEFINED_YET
}

/// FFI export: Get current CTRL-X mode.
#[no_mangle]
pub unsafe extern "C" fn rs_get_ctrl_x_mode() -> c_int {
    nvim_get_ctrl_x_mode()
}

/// FFI export: Check if completion started.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_is_started() -> c_int {
    nvim_get_compl_started()
}

#[allow(clippy::cast_possible_wrap)]
const COMMA: c_char = b',' as c_char;
#[allow(clippy::cast_possible_wrap)]
const SPACE: c_char = b' ' as c_char;

/// Count comma-separated segments in a C string.
///
/// Parses the option string, skipping commas and spaces as delimiters,
/// and counts each non-empty segment.
///
/// # Safety
/// `ptr` must point to a valid NUL-terminated C string (or be null).
unsafe fn count_cpt_segments(ptr: *const c_char) -> c_int {
    if ptr.is_null() {
        return 0;
    }

    let mut p = ptr;
    let mut count: c_int = 0;

    loop {
        let ch = *p;
        if ch == 0 {
            break;
        }

        // Skip delimiters (comma and space)
        if ch == COMMA || ch == SPACE {
            p = p.add(1);
            continue;
        }

        // Found start of a segment — advance past it to the next comma or end
        count += 1;
        while *p != 0 && *p != COMMA {
            p = p.add(1);
        }
    }

    count
}

/// Count comma-separated segments in the 'complete' option (b_p_cpt).
///
/// # Safety
/// Requires valid curbuf state.
#[no_mangle]
pub unsafe extern "C" fn rs_get_cpt_sources_count() -> c_int {
    count_cpt_segments(nvim_curbuf_get_b_p_cpt())
}

/// Check if the current completion source has exceeded its time budget.
///
/// Compares elapsed time since cpt_sources_array[cpt_sources_index].compl_start_tv
/// against compl_timeout_ms. If exceeded, sets compl_time_slice_expired and
/// decays the timeout.
///
/// # Safety
/// Requires valid cpt_sources_array state.
#[no_mangle]
pub unsafe extern "C" fn rs_check_elapsed_time() {
    let start_tv = nvim_get_cpt_start_tv();
    let elapsed_ms = (os_hrtime() - start_tv) / 1_000_000;

    if elapsed_ms > nvim_get_compl_timeout_ms() {
        nvim_set_compl_time_slice_expired(1);
        nvim_decay_compl_timeout();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(CTRL_X_NORMAL, 0);
        assert_eq!(CTRL_X_NOT_DEFINED_YET, 1);
    }

    /// Helper to call count_cpt_segments with a Rust string literal.
    unsafe fn count(s: &[u8]) -> c_int {
        count_cpt_segments(s.as_ptr().cast::<c_char>())
    }

    #[test]
    fn test_count_cpt_segments_standard() {
        unsafe {
            // Standard 'complete' value: ".,w,b,u,t"
            assert_eq!(count(b".,w,b,u,t\0"), 5);
        }
    }

    #[test]
    fn test_count_cpt_segments_single() {
        unsafe {
            assert_eq!(count(b".\0"), 1);
        }
    }

    #[test]
    fn test_count_cpt_segments_empty() {
        unsafe {
            assert_eq!(count(b"\0"), 0);
        }
    }

    #[test]
    fn test_count_cpt_segments_null() {
        unsafe {
            assert_eq!(count_cpt_segments(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_count_cpt_segments_trailing_comma() {
        unsafe {
            assert_eq!(count(b".,w,\0"), 2);
        }
    }

    #[test]
    fn test_count_cpt_segments_consecutive_commas() {
        unsafe {
            assert_eq!(count(b".,,,w\0"), 2);
        }
    }

    #[test]
    fn test_count_cpt_segments_spaces() {
        unsafe {
            assert_eq!(count(b". , w , b\0"), 3);
        }
    }

    #[test]
    fn test_count_cpt_segments_leading_commas() {
        unsafe {
            assert_eq!(count(b",,.\0"), 1);
        }
    }
}
