//! File format state tracking.
//!
//! This module provides functions for tracking and comparing file format
//! state (fileformat, fileencoding, etc.) to determine if a buffer has
//! changed from its original state.

use std::ffi::{c_char, c_int};

use crate::{BufHandle, BF_NEVERLOADED, BF_NEW, NUL};

// =============================================================================
// C Accessor Functions (extern declarations)
// =============================================================================

extern "C" {
    static mut redraw_tabline: bool;
    static mut need_maketitle: bool;
}

#[allow(dead_code)]
extern "C" {
    // Buffer field accessors
    fn nvim_buf_get_b_changed(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_changed(buf: BufHandle, val: bool);
    fn nvim_buf_set_b_changed_invalid(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_flags(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_ml_ml_line_count(buf: BufHandle) -> crate::LinenrT;

    // File format accessors
    fn nvim_buf_get_b_start_ffc(buf: BufHandle) -> c_char;
    fn nvim_buf_set_b_start_ffc(buf: BufHandle, val: c_char);
    fn nvim_buf_get_b_start_eof(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_start_eof(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_start_eol(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_start_eol(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_start_bomb(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_start_bomb(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_start_fenc(buf: BufHandle) -> *mut c_char;
    fn nvim_buf_set_b_start_fenc(buf: BufHandle, val: *mut c_char);
    fn nvim_buf_get_b_p_ff_first_char(buf: BufHandle) -> c_char;
    fn nvim_buf_get_b_p_eof(buf: BufHandle) -> bool;
    fn nvim_buf_get_b_p_eol(buf: BufHandle) -> bool;
    fn nvim_buf_get_b_p_bomb(buf: BufHandle) -> bool;
    fn nvim_buf_get_b_p_bin(buf: BufHandle) -> bool;
    fn nvim_buf_get_b_p_fixeol(buf: BufHandle) -> bool;
    fn nvim_buf_get_b_p_fenc(buf: BufHandle) -> *const c_char;

    // Other functions
    fn nvim_ml_setflags(buf: BufHandle);
    fn nvim_buf_inc_changedtick(buf: BufHandle);
    fn nvim_redraw_buf_status_later(buf: BufHandle);
    // Memory functions
    fn nvim_xfree(ptr: *mut std::ffi::c_void);
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_strcmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // Line access
    fn nvim_ml_get_buf(buf: BufHandle, lnum: crate::LinenrT) -> *mut c_char;
}

// =============================================================================
// File Format State Functions
// =============================================================================

/// Check if file format differs from saved state.
///
/// Return true if 'fileformat' and/or 'fileencoding' has a different value
/// from when editing started (save_file_ff() called).
/// Also when 'endofline' was changed and 'binary' is set, or when 'bomb' was
/// changed and 'binary' is not set.
/// Also when 'endofline' was changed and 'fixeol' is not set.
/// When "ignore_empty" is true don't consider a new, empty buffer to be
/// changed.
fn file_ff_differs_impl(buf: BufHandle, ignore_empty: bool) -> bool {
    // SAFETY: All accessors are safe C functions
    unsafe {
        let flags = nvim_buf_get_b_flags(buf);

        // In a buffer that was never loaded the options are not valid.
        if (flags & BF_NEVERLOADED) != 0 {
            return false;
        }

        // Check for new, empty buffer
        if ignore_empty && (flags & BF_NEW) != 0 && nvim_buf_get_b_ml_ml_line_count(buf) == 1 {
            let line = nvim_ml_get_buf(buf, 1);
            if !line.is_null() && *line == NUL {
                return false;
            }
        }

        // Check if file format changed
        if nvim_buf_get_b_start_ffc(buf) != nvim_buf_get_b_p_ff_first_char(buf) {
            return true;
        }

        // Check end-of-line/end-of-file changes
        let b_p_bin = nvim_buf_get_b_p_bin(buf);
        let b_p_fixeol = nvim_buf_get_b_p_fixeol(buf);
        if (b_p_bin || !b_p_fixeol)
            && (nvim_buf_get_b_start_eof(buf) != nvim_buf_get_b_p_eof(buf)
                || nvim_buf_get_b_start_eol(buf) != nvim_buf_get_b_p_eol(buf))
        {
            return true;
        }

        // Check bomb setting change
        if !b_p_bin && nvim_buf_get_b_start_bomb(buf) != nvim_buf_get_b_p_bomb(buf) {
            return true;
        }

        // Check file encoding change
        let start_fenc = nvim_buf_get_b_start_fenc(buf);
        let p_fenc = nvim_buf_get_b_p_fenc(buf);
        if start_fenc.is_null() {
            return !p_fenc.is_null() && *p_fenc != NUL;
        }
        nvim_strcmp(start_fenc, p_fenc) != 0
    }
}

/// FFI wrapper for `file_ff_differs`.
///
/// Return true if 'fileformat' and/or 'fileencoding' has a different value
/// from when editing started.
#[export_name = "file_ff_differs"]
pub extern "C" fn rs_file_ff_differs(buf: BufHandle, ignore_empty: bool) -> bool {
    file_ff_differs_impl(buf, ignore_empty)
}

/// Save the current values of 'fileformat' and 'fileencoding'.
///
/// Save the current values so that we know the file must be considered
/// changed when the value is different.
fn save_file_ff_impl(buf: BufHandle) {
    // SAFETY: All accessors are safe C functions
    unsafe {
        nvim_buf_set_b_start_ffc(buf, nvim_buf_get_b_p_ff_first_char(buf));
        nvim_buf_set_b_start_eof(buf, nvim_buf_get_b_p_eof(buf));
        nvim_buf_set_b_start_eol(buf, nvim_buf_get_b_p_eol(buf));
        nvim_buf_set_b_start_bomb(buf, nvim_buf_get_b_p_bomb(buf));

        // Only use free/alloc when necessary, they take time.
        let start_fenc = nvim_buf_get_b_start_fenc(buf);
        let p_fenc = nvim_buf_get_b_p_fenc(buf);

        if start_fenc.is_null() || nvim_strcmp(start_fenc, p_fenc) != 0 {
            nvim_xfree(start_fenc.cast());
            nvim_buf_set_b_start_fenc(buf, nvim_xstrdup(p_fenc));
        }
    }
}

/// FFI wrapper for `save_file_ff`.
///
/// Save the current values of 'fileformat' and 'fileencoding', so that we know
/// the file must be considered changed when the value is different.
#[export_name = "save_file_ff"]
pub extern "C" fn rs_save_file_ff(buf: BufHandle) {
    save_file_ff_impl(buf);
}

/// Mark buffer as unchanged.
///
/// If "ff" is true, also save file format state.
/// If "always_inc_changedtick" is true, increment changedtick even if
/// buffer wasn't changed.
fn unchanged_impl(buf: BufHandle, ff: bool, always_inc_changedtick: bool) {
    // SAFETY: All accessors are safe C functions
    unsafe {
        if nvim_buf_get_b_changed(buf) || (ff && file_ff_differs_impl(buf, false)) {
            nvim_buf_set_b_changed(buf, false);
            nvim_buf_set_b_changed_invalid(buf, true);
            nvim_ml_setflags(buf);
            if ff {
                save_file_ff_impl(buf);
            }
            nvim_redraw_buf_status_later(buf);
            redraw_tabline = true;
            need_maketitle = true; // set window title later
            nvim_buf_inc_changedtick(buf);
        } else if always_inc_changedtick {
            nvim_buf_inc_changedtick(buf);
        }
    }
}

/// FFI wrapper for `unchanged`.
///
/// Mark buffer as unchanged. If "ff" is true, also save file format state.
#[export_name = "unchanged"]
pub extern "C" fn rs_unchanged(buf: BufHandle, ff: bool, always_inc_changedtick: bool) {
    unchanged_impl(buf, ff, always_inc_changedtick);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        // Verify buffer flag constants are correct
        assert_eq!(BF_NEVERLOADED, 0x04);
        assert_eq!(BF_NEW, 0x10);
    }
}
