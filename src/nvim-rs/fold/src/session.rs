//! Session persistence for fold state.
//!
//! Implements `put_folds` and helpers that write Ex commands to a session file
//! to restore manual folds and open/closed fold state.

use std::ffi::c_int;
use std::fmt::Write as FmtWrite;

use nvim_window::WinHandle;

use crate::{FoldHandle, GArrayHandle, LineNr};

// ============================================================================
// Constants
// ============================================================================

/// Fold flag: fold is closed.
const FD_CLOSED: c_int = 1;
/// Fold flag: depends on 'foldlevel' (nested folds too).
const FD_LEVEL: c_int = 2;

/// C OK constant.
const OK: c_int = 1;
/// C FAIL constant.
const FAIL: c_int = 0;

// ============================================================================
// C accessor functions
// ============================================================================

extern "C" {
    fn nvim_win_get_folds(wp: WinHandle) -> GArrayHandle;
    fn nvim_win_get_w_fold_manual(wp: WinHandle) -> c_int;
    fn nvim_ga_len(gap: GArrayHandle) -> c_int;
    fn nvim_ga_fold_at(gap: GArrayHandle, idx: c_int) -> FoldHandle;
    fn nvim_fold_get_fd_top(fp: FoldHandle) -> LineNr;
    fn nvim_fold_get_fd_len(fp: FoldHandle) -> LineNr;
    fn nvim_fold_get_fd_nested(fp: FoldHandle) -> GArrayHandle;
    fn nvim_fold_get_fd_flags(fp: FoldHandle) -> c_int;
    fn nvim_win_get_p_fdl(wp: WinHandle) -> c_int;
}

/// A simple wrapper around `libc::FILE*` for writing session output.
struct FoldWriter {
    fd: *mut libc::FILE,
}

impl FoldWriter {
    /// # Safety
    /// `fd` must be a valid open `FILE*` for the lifetime of this writer.
    const unsafe fn new(fd: *mut libc::FILE) -> Self {
        Self { fd }
    }

    fn write_bytes(&mut self, s: &[u8]) -> bool {
        if s.is_empty() {
            return true;
        }
        let written =
            unsafe { libc::fwrite(s.as_ptr().cast::<libc::c_void>(), 1, s.len(), self.fd) };
        written == s.len()
    }

    /// Write a newline. Returns OK or FAIL.
    fn put_eol(&mut self) -> c_int {
        if self.write_bytes(b"\n") {
            OK
        } else {
            FAIL
        }
    }

    /// Write a string followed by a newline. Returns OK or FAIL.
    fn put_line(&mut self, s: &[u8]) -> c_int {
        if self.write_bytes(s) && self.write_bytes(b"\n") {
            OK
        } else {
            FAIL
        }
    }

    /// Format and write a string. Returns OK or FAIL.
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> c_int {
        let mut buf = String::new();
        if buf.write_fmt(args).is_ok() && self.write_bytes(buf.as_bytes()) {
            OK
        } else {
            FAIL
        }
    }
}

// ============================================================================
// Implementation
// ============================================================================

/// Write commands to restore fold state for window `wp`.
///
/// Mirrors C `put_folds`.
///
/// # Safety
/// `fd` must be a valid open `FILE*`.
pub unsafe fn put_folds_impl(fd: *mut libc::FILE, wp: WinHandle) -> c_int {
    if fd.is_null() || wp.is_null() {
        return FAIL;
    }

    let mut w = FoldWriter::new(fd);

    if crate::foldmethod_is_manual_impl(wp) {
        let gap = unsafe { nvim_win_get_folds(wp) };
        if w.put_line(b"silent! normal! zE") == FAIL
            || put_folds_recurse(&mut w, gap, 0) == FAIL
            || w.put_line(b"let &fdl = &fdl") == FAIL
        {
            return FAIL;
        }
    }

    // If some folds are manually opened/closed, restore that.
    if unsafe { nvim_win_get_w_fold_manual(wp) } != 0 {
        let gap = unsafe { nvim_win_get_folds(wp) };
        return put_foldopen_recurse(&mut w, wp, gap, 0);
    }

    OK
}

/// Recursively write fold creation commands for `gap`.
///
/// Mirrors C `put_folds_recurse`.
fn put_folds_recurse(w: &mut FoldWriter, gap: GArrayHandle, off: LineNr) -> c_int {
    if gap.is_null() {
        return OK;
    }
    let len = unsafe { nvim_ga_len(gap) };
    for i in 0..len {
        let fp = unsafe { nvim_ga_fold_at(gap, i) };
        if fp.is_null() {
            continue;
        }
        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
        let fd_len = unsafe { nvim_fold_get_fd_len(fp) };
        let nested = unsafe { nvim_fold_get_fd_nested(fp) };

        // Do nested folds first; they will be created closed.
        if put_folds_recurse(w, nested, off + fd_top) == FAIL {
            return FAIL;
        }

        let top = i64::from(fd_top + off);
        let bot = i64::from(fd_top + off + fd_len - 1);
        if w.write_fmt(format_args!("sil! {top},{bot}fold")) == FAIL || w.put_eol() == FAIL {
            return FAIL;
        }
    }
    OK
}

/// Recursively write fold open/close state commands.
///
/// Mirrors C `put_foldopen_recurse`.
fn put_foldopen_recurse(
    w: &mut FoldWriter,
    wp: WinHandle,
    gap: GArrayHandle,
    off: LineNr,
) -> c_int {
    if gap.is_null() {
        return OK;
    }
    let len = unsafe { nvim_ga_len(gap) };
    for i in 0..len {
        let fp = unsafe { nvim_ga_fold_at(gap, i) };
        if fp.is_null() {
            continue;
        }
        let fd_flags = unsafe { nvim_fold_get_fd_flags(fp) };
        if fd_flags == FD_LEVEL {
            continue;
        }
        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
        let nested = unsafe { nvim_fold_get_fd_nested(fp) };
        let nested_len = if nested.is_null() {
            0
        } else {
            unsafe { nvim_ga_len(nested) }
        };

        if nested_len > 0 {
            // Open nested folds while this fold is open.
            if w.write_fmt(format_args!("{}", i64::from(fd_top + off))) == FAIL
                || w.put_eol() == FAIL
                || w.put_line(b"sil! normal! zo") == FAIL
            {
                return FAIL;
            }
            if put_foldopen_recurse(w, wp, nested, off + fd_top) == FAIL {
                return FAIL;
            }
            // Close the parent when needed.
            if fd_flags == FD_CLOSED && put_fold_open_close(w, fp, off) == FAIL {
                return FAIL;
            }
        } else {
            // Open or close the leaf according to the window foldlevel.
            // Do not close a leaf that is already closed (it would close the parent).
            let level = crate::fold_level_win_impl(wp, off + fd_top);
            let p_fdl = unsafe { nvim_win_get_p_fdl(wp) };
            if ((fd_flags == FD_CLOSED && p_fdl >= level)
                || (fd_flags != FD_CLOSED && p_fdl < level))
                && put_fold_open_close(w, fp, off) == FAIL
            {
                return FAIL;
            }
        }
    }
    OK
}

/// Write the open or close command for fold `fp`.
///
/// Mirrors C `put_fold_open_close`.
fn put_fold_open_close(w: &mut FoldWriter, fp: FoldHandle, off: LineNr) -> c_int {
    let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
    let fd_flags = unsafe { nvim_fold_get_fd_flags(fp) };
    let flag_char = if fd_flags == FD_CLOSED { b'c' } else { b'o' };

    if w.write_fmt(format_args!("{}", i64::from(fd_top + off))) == FAIL
        || w.put_eol() == FAIL
        || w.write_fmt(format_args!("sil! normal! z{}", flag_char as char)) == FAIL
        || w.put_eol() == FAIL
    {
        return FAIL;
    }
    OK
}

// ============================================================================
// FFI Export
// ============================================================================

/// Write commands to "fd" to restore fold state for window "wp".
///
/// Replaces C `put_folds`. Returns OK (1) or FAIL (0).
///
/// # Safety
/// `fd` must be a valid open `FILE*`. `wp` must be a valid `win_T*`.
#[no_mangle]
pub unsafe extern "C" fn rs_put_folds(fd: *mut libc::FILE, wp: WinHandle) -> c_int {
    put_folds_impl(fd, wp)
}
