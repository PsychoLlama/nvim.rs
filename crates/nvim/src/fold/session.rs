//! What `:mkview` writes out to rebuild a window's manual folds.
//!
//! Only `foldmethod=manual` folds are written as `:fold` commands — every
//! other method recomputes them — but the *open/closed* state is written for
//! any window the user has touched by hand.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::types::Failed;
use std::io::Write;

use super::*;

/// Write one line of the view: `args` and a newline.
fn put_line(out: &mut dyn Write, args: core::fmt::Arguments<'_>) -> Result<(), Failed> {
    out.write_fmt(args)
        .and_then(|()| out.write_all(b"\n"))
        .map_err(|_| Failed)
}

/// Write commands to `out` to restore the manual folds in window `window`.
///
/// Answers `Err` if writing fails.
pub fn put_folds(out: &mut dyn Write, window: Win) -> Result<(), Failed> {
    if foldmethod_is_manual(window) {
        put_line(out, format_args!("silent! normal! zE"))?;
        put_folds_recurse(out, window_folds(window), 0)?;
        put_line(out, format_args!("let &fdl = &fdl"))?;
    }
    if window.w_fold_manual {
        return put_foldopen_recurse(out, window, window_folds(window), 0);
    }
    Ok(())
}

/// Write commands to `out` to recreate manually created folds.
///
/// Answers `Err` when writing failed.
pub(super) fn put_folds_recurse(
    out: &mut dyn Write,
    folds: FoldList,
    off: LineNr,
) -> Result<(), Failed> {
    for fold in folds.folds() {
        // The nested folds are written first, because `:fold` over a range
        // that already holds folds swallows them.
        put_folds_recurse(out, fold.nested(), off + fold.top())?;
        let (top, last) = (
            int64_t::from(fold.top() + off),
            int64_t::from(fold.last() + off),
        );
        put_line(out, format_args!("sil! {top},{last}fold"))?;
    }
    Ok(())
}

/// Write commands to `out` to open and close manually opened/closed folds.
///
/// Answers `Err` when writing failed.
pub(super) fn put_foldopen_recurse(
    out: &mut dyn Write,
    window: Win,
    folds: FoldList,
    off: LineNr,
) -> Result<(), Failed> {
    for fold in folds.folds() {
        if fold.is(FD_LEVEL) {
            // It follows 'foldlevel', so there is nothing to remember.
            continue;
        }
        if !fold.nested().is_empty() {
            // Open it first, so the nested commands can reach inside.
            put_line(out, format_args!("{}", int64_t::from(fold.top() + off)))?;
            put_line(out, format_args!("sil! normal! zo"))?;
            put_foldopen_recurse(out, window, fold.nested(), off + fold.top())?;
            if fold.is(FD_CLOSED) {
                put_fold_open_close(out, fold, off)?;
            }
            continue;
        }
        // A leaf: only write the command when its state differs from what
        // 'foldlevel' would give it anyway.
        let level = fold_level_win(window, off + fold.top());
        let foldlevel = window.w_onebuf_opt.wo_fdl;
        let differs = if fold.is(FD_CLOSED) {
            foldlevel >= OptInt::from(level)
        } else {
            foldlevel < OptInt::from(level)
        };
        if differs {
            put_fold_open_close(out, fold, off)?;
        }
    }
    Ok(())
}

/// Write the open or close command to `out`.
///
/// Answers `Err` when writing failed.
pub(super) fn put_fold_open_close(
    out: &mut dyn Write,
    fold: FoldRef,
    off: LineNr,
) -> Result<(), Failed> {
    put_line(out, format_args!("{}", fold.top() + off))?;
    let action = if fold.is(FD_CLOSED) { 'c' } else { 'o' };
    put_line(out, format_args!("sil! normal! z{action}"))
}
