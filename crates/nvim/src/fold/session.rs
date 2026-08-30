//! What `:mkview` writes out to rebuild a window's manual folds.
//!
//! Only `foldmethod=manual` folds are written as `:fold` commands — every
//! other method recomputes them — but the *open/closed* state is written for
//! any window the user has touched by hand.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::ex_session::{put_eol, put_line};
use crate::types::Failed;
use ::libc::fprintf;
use core::ffi::{c_char, c_int};

use super::*;

/// Write commands to "fd" to restore the manual folds in window "wp".
///
/// Answers `Err` if writing fails.
///
/// # Safety
/// `fd` must be an open stream.
pub unsafe fn put_folds(fd: *mut FILE, wp: Win) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- an open stream.
    if foldmethod_is_manual(wp)
        && (unsafe { put_line(fd, c"silent! normal! zE".as_ptr() as *mut c_char) }.is_err()
            || unsafe { put_folds_recurse(fd, window_folds(wp), 0) }.is_err()
            || unsafe { put_line(fd, c"let &fdl = &fdl".as_ptr() as *mut c_char) }.is_err())
    {
        return Err(Failed);
    }
    if wp.w_fold_manual {
        return unsafe { put_foldopen_recurse(fd, wp, window_folds(wp), 0) };
    }
    Ok(())
}

/// Write commands to "fd" to recreate manually created folds.
///
/// Answers `Err` when writing failed.
///
/// # Safety
/// `fd` must be an open stream.
pub(super) unsafe fn put_folds_recurse(
    fd: *mut FILE,
    folds: FoldList,
    off: linenr_T,
) -> Result<(), Failed> {
    for fold in folds.folds() {
        // The nested folds are written first, because `:fold` over a range
        // that already holds folds swallows them.
        // SAFETY: the caller's promise.
        unsafe { put_folds_recurse(fd, fold.nested(), off + fold.top()) }?;
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
        if wrote < 0 || unsafe { put_eol(fd) }.is_err() {
            return Err(Failed);
        }
    }
    Ok(())
}

/// Write commands to "fd" to open and close manually opened/closed folds.
///
/// Answers `Err` when writing failed.
///
/// # Safety
/// `fd` must be an open stream.
pub(super) unsafe fn put_foldopen_recurse(
    fd: *mut FILE,
    wp: Win,
    folds: FoldList,
    off: linenr_T,
) -> Result<(), Failed> {
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
                || unsafe { put_eol(fd) }.is_err()
                || unsafe { put_line(fd, c"sil! normal! zo".as_ptr() as *mut c_char) }.is_err()
            {
                return Err(Failed);
            }
            // SAFETY: the caller's promise.
            if unsafe { put_foldopen_recurse(fd, wp, fold.nested(), off + fold.top()) }.is_err() {
                return Err(Failed);
            }
            // SAFETY: the caller's promise.
            if fold.is(FD_CLOSED) && unsafe { put_fold_open_close(fd, fold, off) }.is_err() {
                return Err(Failed);
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
        if differs && unsafe { put_fold_open_close(fd, fold, off) }.is_err() {
            return Err(Failed);
        }
    }
    Ok(())
}

/// Write the open or close command to "fd".
///
/// Answers `Err` when writing failed.
///
/// # Safety
/// `fd` must be an open stream.
pub(super) unsafe fn put_fold_open_close(
    fd: *mut FILE,
    fold: Fold,
    off: linenr_T,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise; both formats match their arguments.
    if unsafe { fprintf(fd, c"%d".as_ptr(), fold.top() + off) } < 0
        || unsafe { put_eol(fd) }.is_err()
        || unsafe {
            fprintf(
                fd,
                c"sil! normal! z%c".as_ptr(),
                if fold.is(FD_CLOSED) {
                    'c' as c_int
                } else {
                    'o' as c_int
                },
            )
        } < 0
        || unsafe { put_eol(fd) }.is_err()
    {
        return Err(Failed);
    }
    Ok(())
}
