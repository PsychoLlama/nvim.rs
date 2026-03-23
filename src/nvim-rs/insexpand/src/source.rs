//! Completion source management.
//!
//! This module provides helper functions for managing completion sources
//! and the 'complete' option.

#![allow(dead_code, unused_imports)]
#![allow(clippy::missing_const_for_fn)]

use std::os::raw::{c_char, c_int, c_uint};

// C accessor functions
extern "C" {
    fn nvim_curbuf_get_b_p_cpt() -> *const c_char;
    fn nvim_get_cpt_start_tv() -> u64;
    fn nvim_get_compl_timeout_ms() -> u64;
    fn nvim_decay_compl_timeout();
    fn os_hrtime() -> u64;

    // Multibyte helpers
    fn utfc_ptr2len(ptr: *const c_char) -> c_int;

    // Mode and option checks (from lib.rs / search crate)
    fn rs_ctrl_x_mode_dictionary() -> c_int;
    fn rs_ctrl_x_mode_thesaurus() -> c_int;
    fn rs_magic_isset() -> c_int;

    // Accessors for Phase 4 (pass 4) inline implementations
    fn nvim_p_cto() -> c_int;
    fn nvim_set_cpt_sources_start_tv(idx: c_int, ts: u64);
    fn nvim_semsg_list_index_out_of_range(idx: c_int);
}

// CTRL-X mode constants
const CTRL_X_NORMAL: c_int = 0;
const CTRL_X_NOT_DEFINED_YET: c_int = 1;

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
        crate::vars::nvim_set_compl_time_slice_expired(1);
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
        let mb_len = utfc_ptr2len(src) - 1;
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

// =============================================================================
// Phase 4 (pass 4): compl_source_start_timer and advance_cpt_sources_index_safe
// =============================================================================

/// Start the timer for a completion source.
///
/// Sets the start timestamp for the specified source index and clears the
/// time_slice_expired flag. Only active when autocomplete or cto timeout is set.
///
/// # Safety
/// Requires valid cpt_sources_array state with source_idx in bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_compl_source_start_timer(source_idx: c_int) {
    if crate::vars::nvim_get_compl_autocomplete() != 0 || nvim_p_cto() > 0 {
        nvim_set_cpt_sources_start_tv(source_idx, os_hrtime());
        crate::vars::nvim_set_compl_time_slice_expired(0);
    }
}

/// Safely advance cpt_sources_index by one.
///
/// Increments cpt_sources_index if it is within valid bounds.
/// Issues an error message and returns 0 (FAIL) if out of range.
/// Returns 1 (OK) on success.
///
/// # Safety
/// Requires valid cpt_sources_array state.
#[no_mangle]
pub unsafe extern "C" fn rs_advance_cpt_sources_index_safe() -> c_int {
    let idx = nvim_get_cpt_sources_index();
    let count = nvim_get_cpt_sources_count();
    if idx >= 0 && idx < count - 1 {
        nvim_set_cpt_sources_index(idx + 1);
        1 // OK
    } else {
        nvim_semsg_list_index_out_of_range(idx);
        0 // FAIL
    }
}

// =============================================================================
// Phase 5 (pass 12): setup_cpt_sources, prepare_cpt_compl_funcs,
//                    get_cpt_func_completion_matches -- full Rust implementations
// =============================================================================

use std::os::raw::c_void;

const LSIZE: usize = 512;
const FAIL: c_int = -1;
#[allow(dead_code)]
const OK: c_int = 0;

#[allow(clippy::cast_possible_wrap)]
const CARET: c_char = b'^' as c_char;

extern "C" {
    // cpt_sources_array management
    fn nvim_cpt_sources_alloc(count: c_int);
    fn nvim_cpt_sources_set_flag(idx: c_int, flag: c_int);
    fn nvim_cpt_sources_set_max_matches(idx: c_int, val: c_int);
    fn nvim_cpt_sources_set_startcol(idx: c_int, val: c_int);
    fn nvim_cpt_sources_set_refresh_always(idx: c_int, val: c_int);
    fn nvim_cpt_sources_get_refresh_always(idx: c_int) -> c_int;
    fn nvim_get_cpt_source_startcol(idx: c_int) -> c_int;
    fn nvim_get_cpt_sources_index() -> c_int;
    fn nvim_get_cpt_sources_count() -> c_int;
    fn nvim_set_cpt_sources_index(val: c_int);

    // Option parsing helpers
    fn nvim_copy_option_part_ffi(
        src: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: c_int,
        sep: *const c_char,
    ) -> usize;
    fn nvim_copy_option_part_iobuff_ffi(src: *mut *mut c_char) -> usize;
    #[link_name = "vim_strchr"]
    fn nvim_vim_strchr_ffi(s: *const c_char, c: c_int) -> *const c_char;
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_xfree(ptr: *mut u8);

    // Callback/function-source accessors
    fn nvim_get_callback_if_cpt_func_impl(p: *const c_char, idx: c_int) -> *mut c_void;
    fn rs_get_userdefined_compl_info(
        curs_col: c_int,
        cb_opaque: *mut c_void,
        startcol: *mut c_int,
    ) -> c_int;
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_expand_by_function_with_cb(cb_opaque: *mut c_void);
    // Completion state
    fn rs_set_compl_globals(startcol: c_int, curs_col: c_int, is_cpt_compl: c_int);
    fn nvim_ins_compl_insert_bytes(p: *const c_char, len: c_int);
    fn rs_ins_compl_delete(new_leader: c_int);
    fn rs_ins_compl_leader() -> *const c_char;
}

/// Setup completion sources from the 'complete' option.
///
/// Rust port of C `setup_cpt_sources()`. Allocates cpt_sources_array and
/// initializes each source's cs_flag and cs_max_matches from 'cpt' parsing.
///
/// # Safety
/// Requires valid `curbuf` state.
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_setup_cpt_sources() {
    let count = rs_get_cpt_sources_count();
    nvim_cpt_sources_alloc(count);
    if count == 0 {
        return;
    }

    let cpt_ptr = nvim_curbuf_get_b_p_cpt();
    let mut p = cpt_ptr.cast_mut();

    let mut buf = [0u8; LSIZE];
    let mut idx: c_int = 0;

    while *p != 0 {
        // Skip delimiters
        while *p == COMMA || *p == SPACE {
            p = p.add(1);
        }
        if *p == 0 {
            break;
        }

        // cs_flag = first char of segment
        let flag = c_int::from(*p);
        nvim_cpt_sources_set_flag(idx, flag);

        // Reset buf and call copy_option_part to advance p
        buf.fill(0);
        let buf_ptr = buf.as_mut_ptr().cast::<c_char>();
        #[allow(clippy::manual_c_str_literals)]
        let sep = b",\0".as_ptr().cast::<c_char>();
        let slen = nvim_copy_option_part_ffi(&raw mut p, buf_ptr, LSIZE as c_int, sep);

        if slen > 0 {
            // Find caret in buf
            let caret_ptr = nvim_vim_strchr_ffi(buf_ptr.cast_const(), c_int::from(CARET));
            if !caret_ptr.is_null() {
                // Parse integer after the caret
                let digits_start = caret_ptr.add(1);
                let max_matches = parse_c_int(digits_start);
                nvim_cpt_sources_set_max_matches(idx, max_matches);
            }
        }

        idx += 1;
    }
}

/// Call user-defined completion function(s) with findstart=1 to get startcols.
///
/// Rust port of C `prepare_cpt_compl_funcs()`. For each function source in
/// 'cpt', calls the callback with findstart=1 and stores the returned startcol
/// (or -2/-3 on failure/cancel) in cpt_sources_array[idx].cs_startcol.
///
/// # Safety
/// Requires valid `cpt_sources_array` and `curbuf` state.
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_prepare_cpt_compl_funcs() {
    let cpt_ptr = nvim_curbuf_get_b_p_cpt();
    // Make a copy of 'cpt' in case the buffer gets wiped out
    let cpt = nvim_xstrdup(cpt_ptr);
    rs_strip_caret_numbers_in_place(cpt);

    let mut p = cpt;
    let curs_col = nvim_get_cursor_col();
    let mut idx: c_int = 0;

    loop {
        // Skip delimiters
        while *p == COMMA || *p == SPACE {
            p = p.add(1);
        }
        if *p == 0 {
            break;
        }

        let cb = nvim_get_callback_if_cpt_func_impl(p.cast_const(), idx);
        if cb.is_null() {
            nvim_cpt_sources_set_startcol(idx, -3);
        } else {
            let mut startcol: c_int = 0;
            let ret = rs_get_userdefined_compl_info(curs_col, cb, &raw mut startcol);
            if ret == FAIL {
                if startcol == -3 {
                    nvim_cpt_sources_set_refresh_always(idx, 0);
                } else {
                    startcol = -2;
                }
            } else if startcol < 0 || startcol > curs_col {
                startcol = curs_col;
            }
            nvim_cpt_sources_set_startcol(idx, startcol);
        }

        // Advance p past this segment using IObuff
        nvim_copy_option_part_iobuff_ffi(&raw mut p);
        idx += 1;
    }

    nvim_xfree(cpt.cast::<u8>());
}

/// Retrieve completion matches from a specific 'cpt' function source.
///
/// Rust port of C `get_cpt_func_completion_matches()`. Gets startcol from
/// cpt_sources_array, sets completion globals, inserts leader, calls
/// expand_by_function with the callback, and cleans up.
///
/// # Safety
/// `cb_opaque` must be a valid `Callback *`.
/// Requires valid `cpt_sources_array` and `cpt_sources_index` state.
#[no_mangle]
pub unsafe extern "C" fn rs_get_cpt_func_completion_matches(cb_opaque: *mut c_void) {
    let src_idx = nvim_get_cpt_sources_index();
    let startcol = nvim_get_cpt_source_startcol(src_idx);

    if startcol == -2 || startcol == -3 {
        return;
    }

    let curs_col = nvim_get_cursor_col();
    rs_set_compl_globals(startcol, curs_col, 1);

    let refresh_always = nvim_cpt_sources_get_refresh_always(src_idx) != 0;

    if !refresh_always {
        nvim_ins_compl_insert_bytes(rs_ins_compl_leader(), -1);
    }

    nvim_expand_by_function_with_cb(cb_opaque);

    if !refresh_always {
        rs_ins_compl_delete(0);
    }

    // cpt_src->cs_refresh_always = compl_opt_refresh_always; compl_opt_refresh_always = false;
    let opt_refresh = crate::vars::nvim_get_compl_opt_refresh_always() != 0;
    nvim_cpt_sources_set_refresh_always(src_idx, c_int::from(opt_refresh));
    crate::vars::nvim_set_compl_opt_refresh_always(0);
}

/// Parse a C-string as an ASCII decimal integer (like atoi).
///
/// # Safety
/// `s` must point to a valid NUL-terminated C string.
#[inline]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
unsafe fn parse_c_int(s: *const c_char) -> c_int {
    let mut result: c_int = 0;
    let mut p = s;
    while *p != 0 && (*p as u8).is_ascii_digit() {
        result = result * 10 + (c_int::from(*p) - c_int::from(b'0'));
        p = p.add(1);
    }
    result
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
