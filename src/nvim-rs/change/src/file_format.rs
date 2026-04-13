//! File format state tracking.
//!
//! This module provides functions for tracking and comparing file format
//! state (fileformat, fileencoding, etc.) to determine if a buffer has
//! changed from its original state.

use std::ffi::{c_char, c_int};

use crate::{buf_mut, buf_ref, BufHandle, BF_NEVERLOADED, BF_NEW, NUL};

// =============================================================================
// C Accessor Functions (extern declarations)
// =============================================================================

extern "C" {
    static mut redraw_tabline: bool;
    static mut need_maketitle: bool;
}

#[allow(dead_code)]
extern "C" {
    // Other functions
    fn ml_setflags(buf: BufHandle);
    fn nvim_buf_inc_changedtick(buf: BufHandle);
    fn redraw_buf_status_later(buf: BufHandle);
    // Memory functions
    #[link_name = "xfree"]
    fn nvim_xfree(ptr: *mut std::ffi::c_void);
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;

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
        let b = buf_ref(buf);
        let flags = b.b_flags;

        // In a buffer that was never loaded the options are not valid.
        if (flags & BF_NEVERLOADED) != 0 {
            return false;
        }

        // Check for new, empty buffer
        if ignore_empty && (flags & BF_NEW) != 0 && b.ml_line_count == 1 {
            let line = nvim_ml_get_buf(buf, 1);
            if !line.is_null() && *line == NUL {
                return false;
            }
        }

        // Check if file format changed: b_start_ffc (c_int) vs b_p_ff[0]
        let ff_char = if b.b_p_ff.is_null() {
            0 as c_char
        } else {
            *b.b_p_ff
        };
        if b.b_start_ffc as c_char != ff_char {
            return true;
        }

        // Check end-of-line/end-of-file changes
        let b_p_bin = b.b_p_bin != 0;
        let b_p_fixeol = b.b_p_fixeol != 0;
        if (b_p_bin || !b_p_fixeol)
            && ((b.b_start_eof != 0) != (b.b_p_eof != 0)
                || (b.b_start_eol != 0) != (b.b_p_eol != 0))
        {
            return true;
        }

        // Check bomb setting change
        if !b_p_bin && (b.b_start_bomb != 0) != (b.b_p_bomb != 0) {
            return true;
        }

        // Check file encoding change
        let start_fenc = b.b_start_fenc;
        let p_fenc = b.b_p_fenc;
        if start_fenc.is_null() {
            return !p_fenc.is_null() && *p_fenc != NUL;
        }
        libc::strcmp(start_fenc, p_fenc) != 0
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
        {
            let b = buf_mut(buf);
            let ff_char = if b.b_p_ff.is_null() {
                0 as c_int
            } else {
                *b.b_p_ff as c_int
            };
            b.b_start_ffc = ff_char;
            b.b_start_eof = b.b_p_eof;
            b.b_start_eol = b.b_p_eol;
            b.b_start_bomb = b.b_p_bomb;
        }

        // Only use free/alloc when necessary, they take time.
        let start_fenc = buf_ref(buf).b_start_fenc;
        let p_fenc = buf_ref(buf).b_p_fenc;

        if start_fenc.is_null() || libc::strcmp(start_fenc, p_fenc) != 0 {
            nvim_xfree(start_fenc.cast_mut().cast());
            buf_mut(buf).b_start_fenc = nvim_xstrdup(p_fenc);
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
        if buf_ref(buf).b_changed != 0 || (ff && file_ff_differs_impl(buf, false)) {
            buf_mut(buf).b_changed = 0;
            buf_mut(buf).b_changed_invalid = 1;
            ml_setflags(buf);
            if ff {
                save_file_ff_impl(buf);
            }
            redraw_buf_status_later(buf);
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
