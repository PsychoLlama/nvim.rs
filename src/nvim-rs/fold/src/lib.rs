//! Fold method checks and fold state queries for Neovim
//!
//! This crate provides Rust implementations of folding-related functions
//! from `src/nvim/fold.c`. It uses an opaque handle pattern where
//! `win_T*` pointers are treated as opaque handles, with field access
//! done through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)] // Character literals are safe ASCII values

use std::ffi::{c_char, c_int};

use nvim_window::WinHandle;

// C accessor functions
extern "C" {
    /// Get a character from the window's foldmethod string at the given index.
    /// Returns the character at wp->w_p_fdm[idx].
    fn nvim_win_get_fdm_char(wp: WinHandle, idx: c_int) -> c_char;

    /// Get the w_p_fen (foldenable) field from a window.
    fn nvim_win_get_p_fen(wp: WinHandle) -> c_int;

    /// Check if window's buffer has a terminal.
    fn nvim_win_buf_has_terminal(wp: WinHandle) -> c_int;

    /// Check if window's folds growarray is empty.
    fn nvim_win_folds_empty(wp: WinHandle) -> c_int;

    /// Get the current window.
    fn nvim_get_curwin() -> WinHandle;

    /// Emit error message for cannot create fold with current foldmethod.
    fn nvim_emsg_fold_cannot_create();

    /// Emit error message for cannot delete fold with current foldmethod.
    fn nvim_emsg_fold_cannot_delete();
}

// ============================================================================
// Fold Method Checks
// ============================================================================

/// Check if 'foldmethod' is "manual".
///
/// Manual folding requires explicit fold creation by the user.
/// The check is `wp->w_p_fdm[3] == 'u'` (matching "man**u**al").
#[inline]
fn foldmethod_is_manual_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    // "manual" - check character at index 3 is 'u'
    unsafe { nvim_win_get_fdm_char(wp, 3) == b'u' as c_char }
}

/// Check if 'foldmethod' is "indent".
///
/// Indent folding creates folds based on line indentation.
/// The check is `wp->w_p_fdm[0] == 'i'` (matching "**i**ndent").
#[inline]
fn foldmethod_is_indent_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    // "indent" - check character at index 0 is 'i'
    unsafe { nvim_win_get_fdm_char(wp, 0) == b'i' as c_char }
}

/// Check if 'foldmethod' is "expr".
///
/// Expression folding uses 'foldexpr' to determine fold levels.
/// The check is `wp->w_p_fdm[1] == 'x'` (matching "e**x**pr").
#[inline]
fn foldmethod_is_expr_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    // "expr" - check character at index 1 is 'x'
    unsafe { nvim_win_get_fdm_char(wp, 1) == b'x' as c_char }
}

/// Check if 'foldmethod' is "marker".
///
/// Marker folding uses special markers in the text (e.g., `{{{` and `}}}`).
/// The check is `wp->w_p_fdm[2] == 'r'` (matching "ma**r**ker").
#[inline]
fn foldmethod_is_marker_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    // "marker" - check character at index 2 is 'r'
    unsafe { nvim_win_get_fdm_char(wp, 2) == b'r' as c_char }
}

/// Check if 'foldmethod' is "syntax".
///
/// Syntax folding uses syntax highlighting to determine folds.
/// The check is `wp->w_p_fdm[0] == 's'` (matching "**s**yntax").
#[inline]
fn foldmethod_is_syntax_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    // "syntax" - check character at index 0 is 's'
    unsafe { nvim_win_get_fdm_char(wp, 0) == b's' as c_char }
}

/// Check if 'foldmethod' is "diff".
///
/// Diff folding creates folds for unchanged text in diff mode.
/// The check is `wp->w_p_fdm[0] == 'd'` (matching "**d**iff").
#[inline]
fn foldmethod_is_diff_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    // "diff" - check character at index 0 is 'd'
    unsafe { nvim_win_get_fdm_char(wp, 0) == b'd' as c_char }
}

// ============================================================================
// Fold Permission Checks
// ============================================================================

/// Check if manual fold creation or deletion is allowed.
///
/// Returns true if foldmethod is "manual" or "marker".
/// Otherwise, emits an error message and returns false.
#[inline]
fn fold_manual_allowed_impl(create: bool) -> bool {
    let curwin = unsafe { nvim_get_curwin() };
    if foldmethod_is_manual_impl(curwin) || foldmethod_is_marker_impl(curwin) {
        return true;
    }
    if create {
        unsafe { nvim_emsg_fold_cannot_create() };
    } else {
        unsafe { nvim_emsg_fold_cannot_delete() };
    }
    false
}

// ============================================================================
// Fold State Queries
// ============================================================================

/// Check if there may be folded lines in the given window.
///
/// Returns true if:
/// - The buffer is not a terminal
/// - Folding is enabled (w_p_fen)
/// - Either foldmethod is not "manual", or there are manual folds defined
#[inline]
fn has_any_folding_impl(win: WinHandle) -> bool {
    if win.is_null() {
        return false;
    }

    unsafe {
        // Check: !win->w_buffer->terminal
        let has_terminal = nvim_win_buf_has_terminal(win) != 0;
        if has_terminal {
            return false;
        }

        // Check: win->w_p_fen (foldenable)
        let fold_enabled = nvim_win_get_p_fen(win) != 0;
        if !fold_enabled {
            return false;
        }

        // Check: !foldmethodIsManual(win) || !GA_EMPTY(&win->w_folds)
        let is_manual = foldmethod_is_manual_impl(win);
        if !is_manual {
            return true;
        }

        // For manual folding, check if there are any folds defined
        let folds_empty = nvim_win_folds_empty(win) != 0;
        !folds_empty
    }
}

// ============================================================================
// FFI Exports
// ============================================================================

/// Check if 'foldmethod' is "manual".
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldmethodIsManual(wp: WinHandle) -> c_int {
    c_int::from(foldmethod_is_manual_impl(wp))
}

/// Check if 'foldmethod' is "indent".
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldmethodIsIndent(wp: WinHandle) -> c_int {
    c_int::from(foldmethod_is_indent_impl(wp))
}

/// Check if 'foldmethod' is "expr".
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldmethodIsExpr(wp: WinHandle) -> c_int {
    c_int::from(foldmethod_is_expr_impl(wp))
}

/// Check if 'foldmethod' is "marker".
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldmethodIsMarker(wp: WinHandle) -> c_int {
    c_int::from(foldmethod_is_marker_impl(wp))
}

/// Check if 'foldmethod' is "syntax".
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldmethodIsSyntax(wp: WinHandle) -> c_int {
    c_int::from(foldmethod_is_syntax_impl(wp))
}

/// Check if 'foldmethod' is "diff".
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldmethodIsDiff(wp: WinHandle) -> c_int {
    c_int::from(foldmethod_is_diff_impl(wp))
}

/// Check if there may be folded lines in the given window.
///
/// Returns true if the buffer is not a terminal, folding is enabled,
/// and either the foldmethod is not "manual" or there are manual folds defined.
///
/// # Safety
/// The `win` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_hasAnyFolding(win: WinHandle) -> c_int {
    c_int::from(has_any_folding_impl(win))
}

/// Check if manual fold creation or deletion is allowed.
///
/// Returns true if foldmethod is "manual" or "marker".
/// Otherwise, emits an error message and returns false.
///
/// # Safety
/// Requires curwin to be valid.
#[no_mangle]
pub extern "C" fn rs_foldManualAllowed(create: bool) -> c_int {
    c_int::from(fold_manual_allowed_impl(create))
}

#[cfg(test)]
mod tests {
    // Tests require FFI stubs which aren't available in pure Rust testing.
    // Integration testing is done via the full Neovim build.
}
