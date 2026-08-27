//! What `:mkview` writes out to rebuild a window's manual folds.
//!
//! Only `foldmethod=manual` folds are written as `:fold` commands — every
//! other method recomputes them — but the *open/closed* state is written for
//! any window the user has touched by hand.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::ex_session::{put_eol, put_line};
use ::libc::fprintf;
use core::ffi::{c_char, c_int};

use super::*;

/// Write commands to "fd" to restore the manual folds in window "wp".
///
/// Returns FAIL if writing fails.
///
/// # Safety
/// `fd` must be an open stream.
pub unsafe fn put_folds(fd: *mut FILE, wp: Win) -> c_int {
    // SAFETY: the caller's promise -- an open stream.
    unsafe {
        if foldmethod_is_manual(wp)
            && (put_line(fd, c"silent! normal! zE".as_ptr() as *mut c_char) == FAIL
                || put_folds_recurse(fd, window_folds(wp), 0) == FAIL
                || put_line(fd, c"let &fdl = &fdl".as_ptr() as *mut c_char) == FAIL)
        {
            return FAIL;
        }
        if wp.w_fold_manual {
            return put_foldopen_recurse(fd, wp, window_folds(wp), 0);
        }
    }
    OK
}

/// Write commands to "fd" to recreate manually created folds.
///
/// Returns FAIL when writing failed.
///
/// # Safety
/// `fd` must be an open stream.
pub(super) unsafe fn put_folds_recurse(fd: *mut FILE, folds: FoldList, off: linenr_T) -> c_int {
    for fold in folds.folds() {
        // The nested folds are written first, because `:fold` over a range
        // that already holds folds swallows them.
        // SAFETY: the caller's promise.
        if unsafe { put_folds_recurse(fd, fold.nested(), off + fold.top()) } == FAIL {
            return FAIL;
        }
        // SAFETY: the caller's promise; the format string matches the two
        // `int64_t` arguments.
        let wrote = unsafe {
            fprintf(
                fd,
                c"sil! %ld,%ldfold".as_ptr(),
                (fold.top() + off) as int64_t,
                (fold.last() + off) as int64_t,
            )
        };
        // SAFETY: the caller's promise.
        if wrote < 0 || unsafe { put_eol(fd) } == FAIL {
            return FAIL;
        }
    }
    OK
}

/// Write commands to "fd" to open and close manually opened/closed folds.
///
/// Returns FAIL when writing failed.
///
/// # Safety
/// `fd` must be an open stream.
pub(super) unsafe fn put_foldopen_recurse(
    fd: *mut FILE,
    wp: Win,
    folds: FoldList,
    off: linenr_T,
) -> c_int {
    for fold in folds.folds() {
        if fold.is(FD_LEVEL) {
            // It follows 'foldlevel', so there is nothing to remember.
            continue;
        }
        if !fold.nested().is_empty() {
            // Open it first, so the nested commands can reach inside.
            // SAFETY: the caller's promise; the format matches its argument.
            let wrote = unsafe { fprintf(fd, c"%ld".as_ptr(), (fold.top() + off) as int64_t) };
            // SAFETY: the caller's promise.
            if wrote < 0
                || unsafe { put_eol(fd) } == FAIL
                || unsafe { put_line(fd, c"sil! normal! zo".as_ptr() as *mut c_char) } == FAIL
            {
                return FAIL;
            }
            // SAFETY: the caller's promise.
            if unsafe { put_foldopen_recurse(fd, wp, fold.nested(), off + fold.top()) } == FAIL {
                return FAIL;
            }
            // SAFETY: the caller's promise.
            if fold.is(FD_CLOSED) && unsafe { put_fold_open_close(fd, fold, off) } == FAIL {
                return FAIL;
            }
            continue;
        }
        // A leaf: only write the command when its state differs from what
        // 'foldlevel' would give it anyway.
        let level = fold_level_win(wp, off + fold.top());
        let foldlevel = wp.w_onebuf_opt.wo_fdl;
        let differs = if fold.is(FD_CLOSED) {
            foldlevel >= level as OptInt
        } else {
            foldlevel < level as OptInt
        };
        // SAFETY: the caller's promise.
        if differs && unsafe { put_fold_open_close(fd, fold, off) } == FAIL {
            return FAIL;
        }
    }
    OK
}

/// Write the open or close command to "fd".
///
/// Returns FAIL when writing failed.
///
/// # Safety
/// `fd` must be an open stream.
pub(super) unsafe fn put_fold_open_close(fd: *mut FILE, fold: Fold, off: linenr_T) -> c_int {
    // SAFETY: the caller's promise; both formats match their arguments.
    unsafe {
        if fprintf(fd, c"%d".as_ptr(), fold.top() + off) < 0
            || put_eol(fd) == FAIL
            || fprintf(
                fd,
                c"sil! normal! z%c".as_ptr(),
                if fold.is(FD_CLOSED) {
                    'c' as c_int
                } else {
                    'o' as c_int
                },
            ) < 0
            || put_eol(fd) == FAIL
        {
            return FAIL;
        }
    }
    OK
}
