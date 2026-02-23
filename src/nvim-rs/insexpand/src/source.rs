//! Completion source management.
//!
//! This module provides helper functions for managing completion sources
//! and the 'complete' option.

#![allow(clippy::missing_const_for_fn)]

use std::os::raw::{c_char, c_int, c_uint};

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

    // Multibyte helpers
    fn rs_utfc_ptr2len(ptr: *const c_char) -> c_int;

    // Mode and option checks (from lib.rs / search crate)
    fn rs_ctrl_x_mode_dictionary() -> c_int;
    fn rs_ctrl_x_mode_thesaurus() -> c_int;
    fn rs_magic_isset() -> c_int;
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

/// Escape regex metacharacters in a completion pattern string.
///
/// When `dest` is null, counts the number of bytes the escaped output would
/// require (including a trailing NUL). When `dest` is non-null, writes the
/// escaped output. Returns the number of output bytes (including NUL).
///
/// Characters escaped depend on the current CTRL-X mode and magic setting:
///
/// - `.`, `*`, `[`: escaped unless dictionary/thesaurus mode
/// - `~`: escaped only when magic is set (and same dict/thes exception)
/// - `\\`: escaped unless dictionary/thesaurus mode
/// - `^`, `$`: always escaped
///
/// Multibyte characters are copied verbatim (remaining bytes after the first).
///
/// # Safety
/// `src` must point to a valid byte sequence of at least `len` bytes.
/// `dest`, if non-null, must have room for the returned count of bytes.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_quote_meta(dest: *mut c_char, src: *mut c_char, len: c_int) -> c_uint {
    let mut m = len as c_uint + 1; // one extra for the NUL
    let mut src = src;
    let mut dest = dest;
    let mut remaining = len;

    let dict_or_thes = rs_ctrl_x_mode_dictionary() != 0 || rs_ctrl_x_mode_thesaurus() != 0;
    let magic = rs_magic_isset() != 0;

    while remaining > 0 {
        remaining -= 1;
        #[allow(clippy::cast_sign_loss)]
        let ch = *src as u8;
        let needs_escape = match ch {
            b'~' => !dict_or_thes && magic,
            b'.' | b'*' | b'[' | b'\\' => !dict_or_thes,
            b'^' | b'$' => true,
            _ => false,
        };

        if needs_escape {
            m = m.wrapping_add(1);
            if !dest.is_null() {
                #[allow(clippy::cast_possible_wrap)]
                {
                    *dest = b'\\' as c_char;
                }
                dest = dest.add(1);
            }
        }

        if !dest.is_null() {
            *dest = *src;
            dest = dest.add(1);
        }

        // Copy remaining bytes of a multibyte character.
        let mb_len = rs_utfc_ptr2len(src) - 1;
        if mb_len > 0 && remaining >= mb_len {
            let mut i = 0;
            while i < mb_len {
                remaining -= 1;
                src = src.add(1);
                if !dest.is_null() {
                    *dest = *src;
                    dest = dest.add(1);
                }
                i += 1;
            }
        }

        src = src.add(1);
    }

    if !dest.is_null() {
        *dest = 0;
    }

    m
}

/// Strip `^<digits>` segments from a 'complete' option string in place.
///
/// Removes max-matches notation (e.g., `^5`) from comma-separated entries
/// in the 'cpt' option string. Walks the string byte-by-byte, skipping
/// any `^<digits>` sequence that appears before a comma or NUL terminator.
///
/// # Safety
/// `str` must point to a valid NUL-terminated C string, or be null.
#[no_mangle]
pub unsafe extern "C" fn rs_strip_caret_numbers_in_place(str: *mut c_char) {
    if str.is_null() {
        return;
    }

    let mut read = str;
    let mut write = str;

    while *read != 0 {
        #[allow(clippy::cast_possible_wrap)]
        if *read == b'^' as c_char {
            // Check if followed by one or more digits and then comma/NUL
            let mut p = read.add(1);
            #[allow(clippy::cast_sign_loss)]
            while *p != 0 && (*p as u8).is_ascii_digit() {
                p = p.add(1);
            }
            // Valid ^N suffix: at least one digit, followed by comma or NUL
            #[allow(clippy::cast_possible_wrap)]
            if (*p == b',' as c_char || *p == 0) && p != read.add(1) {
                read = p;
                continue;
            }
        }
        *write = *read;
        write = write.add(1);
        read = read.add(1);
    }
    *write = 0;
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

    unsafe fn strip(s: &[u8]) -> String {
        let mut buf = s.to_vec();
        rs_strip_caret_numbers_in_place(buf.as_mut_ptr().cast::<c_char>());
        let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        String::from_utf8_lossy(&buf[..end]).into_owned()
    }

    #[test]
    fn test_strip_caret_numbers_basic() {
        unsafe {
            assert_eq!(strip(b".,w^5,b\0"), ".,w,b");
        }
    }

    #[test]
    fn test_strip_caret_numbers_at_end() {
        unsafe {
            assert_eq!(strip(b".,w^10\0"), ".,w");
        }
    }

    #[test]
    fn test_strip_caret_numbers_no_digits() {
        unsafe {
            // "^" not followed by digits should be preserved
            assert_eq!(strip(b".,^w\0"), ".,^w");
        }
    }

    #[test]
    fn test_strip_caret_numbers_null() {
        unsafe {
            // Should not crash
            rs_strip_caret_numbers_in_place(std::ptr::null_mut());
        }
    }

    #[test]
    fn test_strip_caret_numbers_multiple() {
        unsafe {
            assert_eq!(strip(b"a^3,b^12,c\0"), "a,b,c");
        }
    }
}
