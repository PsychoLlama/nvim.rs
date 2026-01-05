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

/// Line number type (matches linenr_T in C).
pub type LineNr = i64;

// ============================================================================
// Opaque Handle Types
// ============================================================================

/// Opaque handle to a growarray (`garray_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GArrayHandle(*mut std::ffi::c_void);

impl GArrayHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a fold (`fold_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoldHandle(*mut std::ffi::c_void);

impl FoldHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a window line entry (`wline_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WlineHandle(*mut std::ffi::c_void);

impl WlineHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Fold flags (matching C defines in `fold.c`).
pub mod fold_flags {
    /// Fold is open (nested ones can be closed).
    pub const FD_OPEN: c_char = 0;
    /// Fold is closed.
    pub const FD_CLOSED: c_char = 1;
    /// Fold depends on 'foldlevel' (nested folds too).
    pub const FD_LEVEL: c_char = 2;

    use std::ffi::c_char;
}

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

    // ========================================================================
    // Phase 1: Fold state query accessors
    // ========================================================================

    /// Get the w_p_fdl (foldlevel) field from a window.
    fn nvim_win_get_p_fdl(wp: WinHandle) -> c_int;

    /// Get a pointer to the window's folds growarray.
    fn nvim_win_get_folds(wp: WinHandle) -> GArrayHandle;

    /// Get the length of a garray.
    fn nvim_ga_len(gap: GArrayHandle) -> c_int;

    /// Get a fold_T pointer at index in a garray.
    fn nvim_ga_fold_at(gap: GArrayHandle, idx: c_int) -> FoldHandle;

    /// Get the fd_top field from a fold.
    fn nvim_fold_get_fd_top(fp: FoldHandle) -> LineNr;

    /// Get the fd_len field from a fold.
    fn nvim_fold_get_fd_len(fp: FoldHandle) -> LineNr;

    /// Get a pointer to the nested folds growarray.
    fn nvim_fold_get_fd_nested(fp: FoldHandle) -> GArrayHandle;

    /// Get the fd_flags field from a fold.
    fn nvim_fold_get_fd_flags(fp: FoldHandle) -> c_char;

    /// Get the w_foldinvalid field from a window (reserved for future use).
    #[allow(dead_code)]
    fn nvim_win_get_w_foldinvalid(wp: WinHandle) -> bool;

    /// Call checkupdate for a window.
    fn nvim_checkupdate(wp: WinHandle);

    // ========================================================================
    // Phase 2: Fold navigation accessors
    // ========================================================================

    /// Get the w_lines_valid field from a window.
    fn nvim_win_get_w_lines_valid(wp: WinHandle) -> c_int;

    /// Get a wline_T pointer at index in a window's w_lines array.
    fn nvim_win_get_wl_entry(wp: WinHandle, idx: c_int) -> WlineHandle;

    /// Get the wl_lnum field from a wline_T.
    fn nvim_wline_get_lnum(wl: WlineHandle) -> LineNr;

    /// Get the wl_foldend field from a wline_T.
    fn nvim_wline_get_foldend(wl: WlineHandle) -> LineNr;

    /// Get the wl_valid field from a wline_T.
    fn nvim_wline_get_valid(wl: WlineHandle) -> bool;
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
// Fold Level and Nesting
// ============================================================================

/// Binary search for a fold containing line number `lnum` in a garray.
///
/// Returns `Some((fp, true))` if a fold contains `lnum`, where `fp` is the fold.
/// Returns `Some((fp, false))` if no fold contains `lnum`, where `fp` is the
/// first fold below `lnum`.
/// Returns `None` if the array is empty.
fn fold_find(gap: GArrayHandle, lnum: LineNr) -> Option<(FoldHandle, bool)> {
    if gap.is_null() {
        return None;
    }

    let len = unsafe { nvim_ga_len(gap) };
    if len == 0 {
        return None;
    }

    // Binary search
    let mut low: i32 = 0;
    let mut high: i32 = len - 1;

    while low <= high {
        let mid = i32::midpoint(low, high);
        let fp = unsafe { nvim_ga_fold_at(gap, mid) };
        if fp.is_null() {
            return None;
        }

        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
        let fd_len = unsafe { nvim_fold_get_fd_len(fp) };

        if fd_top > lnum {
            // fold below lnum, adjust high
            high = mid - 1;
        } else if fd_top + fd_len <= lnum {
            // fold above lnum, adjust low
            low = mid + 1;
        } else {
            // lnum is inside this fold
            return Some((fp, true));
        }
    }

    // Return fold at `low` index (first fold below lnum)
    let fp = unsafe { nvim_ga_fold_at(gap, low) };
    if fp.is_null() {
        None
    } else {
        Some((fp, false))
    }
}

/// Get fold level at line number `lnum` in window `wp`.
///
/// Recursively searches for folds that contain `lnum`.
fn fold_level_win_impl(wp: WinHandle, lnum: LineNr) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let mut lnum_rel = lnum;
    let mut level: c_int = 0;
    let mut gap = unsafe { nvim_win_get_folds(wp) };

    while let Some((fp, true)) = fold_find(gap, lnum_rel) {
        // Found a fold containing lnum_rel
        // Check nested folds. Line number is relative to containing fold.
        gap = unsafe { nvim_fold_get_fd_nested(fp) };
        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
        lnum_rel -= fd_top;
        level += 1;
    }

    level
}

/// Get the maximum fold nesting depth in a garray recursively.
fn get_deepest_nesting_recurse(gap: GArrayHandle) -> c_int {
    if gap.is_null() {
        return 0;
    }

    let len = unsafe { nvim_ga_len(gap) };
    let mut max_level: c_int = 0;

    for i in 0..len {
        let fp = unsafe { nvim_ga_fold_at(gap, i) };
        if fp.is_null() {
            continue;
        }
        let nested = unsafe { nvim_fold_get_fd_nested(fp) };
        let level = get_deepest_nesting_recurse(nested) + 1;
        if level > max_level {
            max_level = level;
        }
    }

    max_level
}

/// Get the maximum fold nesting depth in window `wp`.
///
/// First ensures folds are up to date via checkupdate.
fn get_deepest_nesting_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    // First, update folds if needed
    unsafe { nvim_checkupdate(wp) };

    let gap = unsafe { nvim_win_get_folds(wp) };
    get_deepest_nesting_recurse(gap)
}

/// Find an entry in win->w_lines[] for buffer line "lnum".
///
/// Returns index of entry or -1 if not found.
/// Only valid entries are considered.
fn find_wl_entry_impl(win: WinHandle, lnum: LineNr) -> c_int {
    if win.is_null() {
        return -1;
    }

    let lines_valid = unsafe { nvim_win_get_w_lines_valid(win) };

    for i in 0..lines_valid {
        let wl = unsafe { nvim_win_get_wl_entry(win, i) };
        if wl.is_null() {
            continue;
        }

        let valid = unsafe { nvim_wline_get_valid(wl) };
        if !valid {
            continue;
        }

        let wl_lnum = unsafe { nvim_wline_get_lnum(wl) };
        if lnum < wl_lnum {
            return -1;
        }

        let wl_foldend = unsafe { nvim_wline_get_foldend(wl) };
        if lnum <= wl_foldend {
            return i;
        }
    }

    -1
}

/// Check if line is inside a closed fold (low level, no caching).
fn line_folded_impl(win: WinHandle, lnum: LineNr) -> bool {
    if win.is_null() {
        return false;
    }

    // First check if there's any folding at all
    if !has_any_folding_impl(win) {
        return false;
    }

    // Check if the line is in a closed fold by walking the fold tree
    let foldlevel = unsafe { nvim_win_get_p_fdl(win) };
    let mut lnum_rel = lnum;
    let mut gap = unsafe { nvim_win_get_folds(win) };
    let mut level = 0;
    let mut use_level = false;

    while let Some((fp, true)) = fold_find(gap, lnum_rel) {
        let flags = unsafe { nvim_fold_get_fd_flags(fp) };

        // Check if this fold is closed
        // Once FD_LEVEL is seen, all nested folds also use level comparison
        if use_level || flags == fold_flags::FD_LEVEL {
            use_level = true;
            if level >= foldlevel {
                return true;
            }
        } else if flags == fold_flags::FD_CLOSED {
            return true;
        }

        // Check nested folds
        gap = unsafe { nvim_fold_get_fd_nested(fp) };
        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
        lnum_rel -= fd_top;
        level += 1;
    }

    false
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

// ============================================================================
// Phase 1 & 2: Fold State Queries and Navigation
// ============================================================================

/// Get fold level at line number `lnum` in window `wp`.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldLevelWin(wp: WinHandle, lnum: LineNr) -> c_int {
    fold_level_win_impl(wp, lnum)
}

/// Get the maximum fold nesting depth in window `wp`.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_getDeepestNesting(wp: WinHandle) -> c_int {
    get_deepest_nesting_impl(wp)
}

/// Find an entry in win->w_lines[] for buffer line "lnum".
///
/// Returns index of entry or -1 if not found.
///
/// # Safety
/// The `win` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_find_wl_entry(win: WinHandle, lnum: LineNr) -> c_int {
    find_wl_entry_impl(win, lnum)
}

/// Check if line is inside a closed fold (low level, no caching).
///
/// # Safety
/// The `win` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_lineFolded(win: WinHandle, lnum: LineNr) -> c_int {
    c_int::from(line_folded_impl(win, lnum))
}

#[cfg(test)]
mod tests {
    // Tests require FFI stubs which aren't available in pure Rust testing.
    // Integration testing is done via the full Neovim build.
}
