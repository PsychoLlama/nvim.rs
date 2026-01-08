//! Fold method checks and fold state queries for Neovim
//!
//! This crate provides Rust implementations of folding-related functions
//! from `src/nvim/fold.c`. It uses an opaque handle pattern where
//! `win_T*` pointers are treated as opaque handles, with field access
//! done through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)] // Character literals are safe ASCII values

pub mod level;
pub mod markers;

use std::ffi::{c_char, c_int};

use nvim_buffer::BufHandle;
use nvim_window::WinHandle;

/// Line number type (matches linenr_T in C).
pub type LineNr = i32;

/// Result struct for hasFoldingWin.
///
/// This struct contains all the output values from hasFoldingWin:
/// - has_folding: whether the line is in a closed fold
/// - first: first line of the fold (only valid if has_folding is true)
/// - last: last line of the fold (only valid if has_folding is true)
/// - fi_level: fold level
/// - fi_lnum: line number where fold starts
/// - fi_low_level: lowest fold level that starts in the same line
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldingResult {
    /// Whether the line is in a closed fold (0 = false, non-zero = true).
    pub has_folding: c_int,
    /// First line of the fold (0 if not folded).
    pub first: LineNr,
    /// Last line of the fold (0 if not folded).
    pub last: LineNr,
    /// Fold level (fi_level).
    pub fi_level: c_int,
    /// Line number where fold starts (fi_lnum).
    pub fi_lnum: LineNr,
    /// Lowest fold level that starts in the same line (fi_low_level).
    pub fi_low_level: c_int,
}

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

/// TriState values (matching C enum in `types_defs.h`).
pub mod tristate {
    use std::ffi::c_int;

    /// kNone - undefined/unknown state.
    pub const K_NONE: c_int = -1;
    /// kFalse - false state.
    pub const K_FALSE: c_int = 0;
    /// kTrue - true state.
    pub const K_TRUE: c_int = 1;
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

    /// Set the fd_flags field of a fold.
    fn nvim_fold_set_fd_flags(fp: FoldHandle, flags: c_char);

    /// Get the fd_small field from a fold.
    #[allow(dead_code)]
    fn nvim_fold_get_fd_small(fp: FoldHandle) -> c_int;

    /// Set the fd_small field of a fold.
    fn nvim_fold_set_fd_small(fp: FoldHandle, small: c_int);

    /// Swap two fold entries in a garray.
    fn nvim_fold_swap(gap: GArrayHandle, idx1: c_int, idx2: c_int);

    // ========================================================================
    // Phase 3: State query accessors
    // ========================================================================

    /// Get the w_p_fml (foldminlines) field from a window.
    fn nvim_win_get_p_fml(wp: WinHandle) -> c_int;

    /// Get the number of screen lines for a physical line (no fold consideration).
    fn nvim_plines_win_nofold(wp: WinHandle, lnum: LineNr) -> c_int;

    // ========================================================================
    // Phase 1: Foundation function accessors
    // ========================================================================

    /// Initialize the folds garray for a window.
    fn nvim_ga_init_folds(gap: GArrayHandle);

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

    /// Get the wl_folded field from a wline_T.
    fn nvim_wline_get_folded(wl: WlineHandle) -> bool;

    // ========================================================================
    // Phase 2: Core query accessors
    // ========================================================================

    /// Get the line count of the window's buffer.
    fn nvim_win_get_buf_line_count(wp: WinHandle) -> LineNr;

    // ========================================================================
    // Phase 1: Fold Tree Manipulation accessors
    // ========================================================================

    /// Grow a garray to hold at least n more fold_T entries.
    fn nvim_ga_grow_folds(gap: GArrayHandle, n: c_int);

    /// Set the fd_top field of a fold.
    fn nvim_fold_set_fd_top(fp: FoldHandle, top: LineNr);

    /// Set the fd_len field of a fold.
    fn nvim_fold_set_fd_len(fp: FoldHandle, len: LineNr);

    /// Get the ga_data pointer from a garray (as fold_T*).
    fn nvim_ga_get_fold_data(gap: GArrayHandle) -> FoldHandle;

    /// Set the ga_len field of a garray.
    fn nvim_ga_set_len(gap: GArrayHandle, len: c_int);

    /// Move fold entries within a garray.
    fn nvim_fold_memmove(gap: GArrayHandle, dst_idx: c_int, src_idx: c_int, count: c_int);

    /// Copy a fold entry from one location to another.
    fn nvim_fold_copy(dst: FoldHandle, src: FoldHandle);

    /// Get the buffer from a window.
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;

    /// Call deleteFoldRecurse from Rust (to recursively free nested fold memory).
    fn nvim_deleteFoldRecurse(buf: BufHandle, gap: GArrayHandle);

    /// Free the ga_data pointer of a garray (for nested folds).
    fn nvim_ga_free_data(gap: GArrayHandle);

    /// Set the fold_changed flag.
    fn nvim_set_fold_changed(changed: bool);

    /// Get the fold_changed flag.
    #[allow(dead_code)]
    fn nvim_get_fold_changed() -> bool;
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
// Phase 1: Pure Recursive Functions
// ============================================================================

/// Check if folds should close recursively based on foldlevel.
///
/// Only manually opened folds (FD_OPEN) may need to be closed.
/// If level <= 0 and lnum is outside the fold, reset to FD_LEVEL.
/// Otherwise recurse into nested folds.
fn check_close_rec_impl(gap: GArrayHandle, lnum: LineNr, level: c_int) -> bool {
    if gap.is_null() {
        return false;
    }

    let len = unsafe { nvim_ga_len(gap) };
    let mut retval = false;

    for i in 0..len {
        let fp = unsafe { nvim_ga_fold_at(gap, i) };
        if fp.is_null() {
            continue;
        }

        let flags = unsafe { nvim_fold_get_fd_flags(fp) };

        // Only manually opened folds may need to be closed.
        if flags == fold_flags::FD_OPEN {
            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
            let fd_len = unsafe { nvim_fold_get_fd_len(fp) };

            if level <= 0 && (lnum < fd_top || lnum >= fd_top + fd_len) {
                // lnum is outside this fold, reset to FD_LEVEL
                unsafe { nvim_fold_set_fd_flags(fp, fold_flags::FD_LEVEL) };
                retval = true;
            } else {
                // Check nested folds (lnum relative to containing fold)
                let nested = unsafe { nvim_fold_get_fd_nested(fp) };
                retval |= check_close_rec_impl(nested, lnum - fd_top, level - 1);
            }
        }
    }

    retval
}

/// Open all nested folds in a fold recursively.
///
/// Sets FD_OPEN flag on all nested folds.
fn fold_open_nested_impl(fp: FoldHandle) {
    if fp.is_null() {
        return;
    }

    let nested = unsafe { nvim_fold_get_fd_nested(fp) };
    if nested.is_null() {
        return;
    }

    let len = unsafe { nvim_ga_len(nested) };
    for i in 0..len {
        let nested_fp = unsafe { nvim_ga_fold_at(nested, i) };
        if nested_fp.is_null() {
            continue;
        }

        // First recurse into this fold's nested folds
        fold_open_nested_impl(nested_fp);

        // Then set this fold to open
        unsafe { nvim_fold_set_fd_flags(nested_fp, fold_flags::FD_OPEN) };
    }
}

/// Set small flags in a fold array to kNone.
///
/// This resets the fd_small field so it will be recalculated.
fn set_small_maybe_impl(gap: GArrayHandle) {
    if gap.is_null() {
        return;
    }

    let len = unsafe { nvim_ga_len(gap) };
    for i in 0..len {
        let fp = unsafe { nvim_ga_fold_at(gap, i) };
        if fp.is_null() {
            continue;
        }

        unsafe { nvim_fold_set_fd_small(fp, tristate::K_NONE) };
    }
}

/// Reverse the order of fold entries in a garray.
///
/// Reverses entries from start_arg to end_arg (inclusive).
#[allow(clippy::cast_possible_truncation)]
fn fold_reverse_order_impl(gap: GArrayHandle, start_arg: LineNr, end_arg: LineNr) {
    if gap.is_null() {
        return;
    }

    let mut start = start_arg;
    let mut end = end_arg;

    // Indices are bounded by garray length, which fits in c_int
    while start < end {
        unsafe { nvim_fold_swap(gap, start as c_int, end as c_int) };
        start += 1;
        end -= 1;
    }
}

// ============================================================================
// Phase 1: Foundation Functions
// ============================================================================

/// Initialize the fold garray for a new window.
fn fold_init_win_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    let gap = unsafe { nvim_win_get_folds(wp) };
    if gap.is_null() {
        return;
    }

    unsafe { nvim_ga_init_folds(gap) };
}

// ============================================================================
// Phase 2: Core Query Functions
// ============================================================================

/// Implementation of hasFoldingWin.
///
/// Search for folds containing `lnum` and determine if it's in a closed fold.
/// If `cache` is true, first check the cached w_lines[] array.
fn has_folding_win_impl(win: WinHandle, lnum: LineNr, cache: bool) -> FoldingResult {
    if win.is_null() {
        return FoldingResult {
            has_folding: 0,
            first: 0,
            last: 0,
            fi_level: 0,
            fi_lnum: 0,
            fi_low_level: 0,
        };
    }

    // First update folds
    unsafe { nvim_checkupdate(win) };

    // Return quickly when there is no folding at all in this window.
    if !has_any_folding_impl(win) {
        return FoldingResult {
            has_folding: 0,
            first: 0,
            last: 0,
            fi_level: 0,
            fi_lnum: 0,
            fi_low_level: 0,
        };
    }

    let mut had_folded = false;
    let mut first: LineNr = 0;
    let mut last: LineNr = 0;

    // Check cache if requested
    if cache {
        let x = find_wl_entry_impl(win, lnum);
        if x >= 0 {
            let wl = unsafe { nvim_win_get_wl_entry(win, x) };
            if !wl.is_null() {
                first = unsafe { nvim_wline_get_lnum(wl) };
                last = unsafe { nvim_wline_get_foldend(wl) };
                had_folded = unsafe { nvim_wline_get_folded(wl) };
            }
        }
    }

    let mut lnum_rel = lnum;
    let mut level: c_int = 0;
    let mut low_level: c_int = 0;
    let mut maybe_small = false;
    let mut use_level = false;

    if first == 0 {
        // Recursively search for a fold that contains "lnum".
        let mut gap = unsafe { nvim_win_get_folds(win) };

        while let Some((fp, true)) = fold_find(gap, lnum_rel) {
            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };

            // Remember lowest level of fold that starts in "lnum".
            if lnum_rel == fd_top && low_level == 0 {
                low_level = level + 1;
            }

            first += fd_top;
            last += fd_top;

            // is this fold closed?
            had_folded = check_closed_impl(
                win,
                fp,
                &mut use_level,
                level,
                &mut maybe_small,
                lnum - lnum_rel,
            );
            if had_folded {
                // Fold closed: Set last and quit loop.
                let fd_len = unsafe { nvim_fold_get_fd_len(fp) };
                last += fd_len - 1;
                break;
            }

            // Fold found, but it's open: Check nested folds.  Line number is
            // relative to containing fold.
            gap = unsafe { nvim_fold_get_fd_nested(fp) };
            lnum_rel -= fd_top;
            level += 1;
        }
    }

    if !had_folded {
        return FoldingResult {
            has_folding: 0,
            first: 0,
            last: 0,
            fi_level: level,
            fi_lnum: lnum - lnum_rel,
            fi_low_level: if low_level == 0 { level } else { low_level },
        };
    }

    // Clamp last to buffer line count
    let line_count = unsafe { nvim_win_get_buf_line_count(win) };
    if last > line_count {
        last = line_count;
    }

    FoldingResult {
        has_folding: 1,
        first,
        last,
        fi_level: level + 1,
        fi_lnum: first,
        fi_low_level: if low_level == 0 { level + 1 } else { low_level },
    }
}

// ============================================================================
// Phase 3: State Query Functions
// ============================================================================

/// Update fd_small field of fold "fp".
///
/// Checks if a fold is "small" based on foldminlines setting.
/// A fold is small if its total screen lines <= foldminlines.
fn check_small_impl(wp: WinHandle, fp: FoldHandle, lnum_off: LineNr) {
    if wp.is_null() || fp.is_null() {
        return;
    }

    let fd_small = unsafe { nvim_fold_get_fd_small(fp) };
    if fd_small != tristate::K_NONE {
        return;
    }

    // Mark any nested folds to maybe-small
    let nested = unsafe { nvim_fold_get_fd_nested(fp) };
    set_small_maybe_impl(nested);

    let fd_len = unsafe { nvim_fold_get_fd_len(fp) };
    let fml = unsafe { nvim_win_get_p_fml(wp) };

    if fd_len > LineNr::from(fml) {
        unsafe { nvim_fold_set_fd_small(fp, tristate::K_FALSE) };
    } else {
        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
        let mut count: c_int = 0;
        for n in 0..fd_len {
            count += unsafe { nvim_plines_win_nofold(wp, fd_top + lnum_off + n) };
            if count > fml {
                unsafe { nvim_fold_set_fd_small(fp, tristate::K_FALSE) };
                return;
            }
        }
        unsafe { nvim_fold_set_fd_small(fp, tristate::K_TRUE) };
    }
}

/// Check if a fold is closed and update info needed for nested fold checks.
///
/// Returns true if the fold is closed.
/// Updates use_level and maybe_small for tracking state across nested folds.
fn check_closed_impl(
    wp: WinHandle,
    fp: FoldHandle,
    use_level: &mut bool,
    level: c_int,
    maybe_small: &mut bool,
    lnum_off: LineNr,
) -> bool {
    if wp.is_null() || fp.is_null() {
        return false;
    }

    let fdl = unsafe { nvim_win_get_p_fdl(wp) };
    let flags = unsafe { nvim_fold_get_fd_flags(fp) };
    let mut closed = false;

    // Check if this fold is closed. If flag is FD_LEVEL, this fold
    // and all folds it contains depend on 'foldlevel'.
    if *use_level || flags == fold_flags::FD_LEVEL {
        *use_level = true;
        if level >= fdl {
            closed = true;
        }
    } else if flags == fold_flags::FD_CLOSED {
        closed = true;
    }

    // Small fold isn't closed anyway.
    let fd_small = unsafe { nvim_fold_get_fd_small(fp) };
    if fd_small == tristate::K_NONE {
        *maybe_small = true;
    }

    if closed {
        if *maybe_small {
            unsafe { nvim_fold_set_fd_small(fp, tristate::K_NONE) };
        }
        check_small_impl(wp, fp, lnum_off);
        let fd_small_after = unsafe { nvim_fold_get_fd_small(fp) };
        if fd_small_after == tristate::K_TRUE {
            closed = false;
        }
    }

    closed
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
// Phase 1: Foundation Functions - FFI Exports
// ============================================================================

/// Initialize the fold garray for a new window.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldInitWin(wp: WinHandle) {
    fold_init_win_impl(wp);
}

// ============================================================================
// Phase 2: Core Query Functions - FFI Exports
// ============================================================================

/// Check if line is in a closed fold and return fold information.
///
/// Returns a FoldingResult struct containing:
/// - has_folding: whether the line is in a closed fold
/// - first/last: fold boundaries (only valid if has_folding is true)
/// - fi_level/fi_lnum/fi_low_level: fold info
///
/// # Safety
/// The `win` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_hasFoldingWin(win: WinHandle, lnum: LineNr, cache: bool) -> FoldingResult {
    has_folding_win_impl(win, lnum, cache)
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

// ============================================================================
// Phase 1: Pure Recursive Functions - FFI Exports
// ============================================================================

/// Check if folds should close recursively based on foldlevel.
///
/// Returns true if any fold was changed.
///
/// # Safety
/// The `gap` parameter must be a valid `garray_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_checkCloseRec(gap: GArrayHandle, lnum: LineNr, level: c_int) -> c_int {
    c_int::from(check_close_rec_impl(gap, lnum, level))
}

/// Open all nested folds in a fold recursively.
///
/// Sets FD_OPEN flag on all nested folds.
///
/// # Safety
/// The `fp` parameter must be a valid `fold_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldOpenNested(fp: FoldHandle) {
    fold_open_nested_impl(fp);
}

/// Set small flags in a fold array to kNone.
///
/// # Safety
/// The `gap` parameter must be a valid `garray_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_setSmallMaybe(gap: GArrayHandle) {
    set_small_maybe_impl(gap);
}

/// Reverse the order of fold entries in a garray.
///
/// Reverses entries from start to end (inclusive).
///
/// # Safety
/// The `gap` parameter must be a valid `garray_T*` pointer or null.
/// Start and end must be valid indices.
#[no_mangle]
pub extern "C" fn rs_foldReverseOrder(gap: GArrayHandle, start: LineNr, end: LineNr) {
    fold_reverse_order_impl(gap, start, end);
}

// ============================================================================
// Phase 3: State Query Functions - FFI Exports
// ============================================================================

/// Update fd_small field of fold "fp".
///
/// Checks if a fold is "small" based on foldminlines setting.
///
/// # Safety
/// The `wp` and `fp` parameters must be valid pointers or null.
#[no_mangle]
pub extern "C" fn rs_checkSmall(wp: WinHandle, fp: FoldHandle, lnum_off: LineNr) {
    check_small_impl(wp, fp, lnum_off);
}

/// Check if a fold is closed and update info needed for nested fold checks.
///
/// Returns true if the fold is closed.
///
/// # Safety
/// The `wp` and `fp` parameters must be valid pointers or null.
/// The `use_level` and `maybe_small` pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_check_closed(
    wp: WinHandle,
    fp: FoldHandle,
    use_level: *mut bool,
    level: c_int,
    maybe_small: *mut bool,
    lnum_off: LineNr,
) -> c_int {
    if use_level.is_null() || maybe_small.is_null() {
        return 0;
    }
    let use_level_ref = &mut *use_level;
    let maybe_small_ref = &mut *maybe_small;
    c_int::from(check_closed_impl(
        wp,
        fp,
        use_level_ref,
        level,
        maybe_small_ref,
        lnum_off,
    ))
}

// ============================================================================
// Phase 1: Fold Tree Manipulation Functions
// ============================================================================

/// Insert a new fold in "gap" at position "i".
///
/// Grows the array if needed, shifts existing folds, and initializes the new fold's
/// nested garray.
fn fold_insert_impl(gap: GArrayHandle, i: c_int) {
    if gap.is_null() || i < 0 {
        return;
    }

    // Grow array by 1
    unsafe { nvim_ga_grow_folds(gap, 1) };

    let len = unsafe { nvim_ga_len(gap) };

    // Shift existing folds if inserting in middle
    if len > 0 && i < len {
        unsafe { nvim_fold_memmove(gap, i + 1, i, len - i) };
    }

    // Increment length
    unsafe { nvim_ga_set_len(gap, len + 1) };

    // Initialize the nested garray for the new fold
    let fp = unsafe { nvim_ga_fold_at(gap, i) };
    if !fp.is_null() {
        let nested = unsafe { nvim_fold_get_fd_nested(fp) };
        if !nested.is_null() {
            unsafe { nvim_ga_init_folds(nested) };
        }
    }
}

/// Delete fold "idx" from growarray "gap".
///
/// When `recursive` is true, also delete all the folds contained in it.
/// When `recursive` is false, contained folds are moved one level up.
fn delete_fold_entry_impl(wp: WinHandle, gap: GArrayHandle, idx: c_int, recursive: bool) {
    if wp.is_null() || gap.is_null() || idx < 0 {
        return;
    }

    let len = unsafe { nvim_ga_len(gap) };
    if idx >= len {
        return;
    }

    let fp = unsafe { nvim_ga_fold_at(gap, idx) };
    if fp.is_null() {
        return;
    }

    let nested = unsafe { nvim_fold_get_fd_nested(fp) };
    let nested_len = if nested.is_null() {
        0
    } else {
        unsafe { nvim_ga_len(nested) }
    };

    let buf = unsafe { nvim_win_get_buffer(wp) };

    if recursive || nested_len == 0 {
        // Recursively delete the contained folds
        if !nested.is_null() {
            unsafe { nvim_deleteFoldRecurse(buf, nested) };
        }

        // Shift remaining folds down
        if idx < len - 1 {
            unsafe { nvim_fold_memmove(gap, idx, idx + 1, len - idx - 1) };
        }

        // Decrement length
        unsafe { nvim_ga_set_len(gap, len - 1) };
    } else {
        // Move nested folds one level up
        // Need to grow gap to hold (nested_len - 1) more entries
        unsafe { nvim_ga_grow_folds(gap, nested_len - 1) };

        // Re-fetch fp after potential realloc
        let fp = unsafe { nvim_ga_fold_at(gap, idx) };
        if fp.is_null() {
            return;
        }
        let nested = unsafe { nvim_fold_get_fd_nested(fp) };
        if nested.is_null() {
            return;
        }

        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
        let fd_flags = unsafe { nvim_fold_get_fd_flags(fp) };
        let fd_small = unsafe { nvim_fold_get_fd_small(fp) };

        // Adjust fd_top and fd_flags for the nested folds
        for i in 0..nested_len {
            let nfp = unsafe { nvim_ga_fold_at(nested, i) };
            if nfp.is_null() {
                continue;
            }
            let nfp_top = unsafe { nvim_fold_get_fd_top(nfp) };
            unsafe { nvim_fold_set_fd_top(nfp, nfp_top + fd_top) };
            if fd_flags == fold_flags::FD_LEVEL {
                unsafe { nvim_fold_set_fd_flags(nfp, fold_flags::FD_LEVEL) };
            }
            if fd_small == tristate::K_NONE {
                unsafe { nvim_fold_set_fd_small(nfp, tristate::K_NONE) };
            }
        }

        // Move existing folds down to make room
        let new_len = len + nested_len - 1;
        if idx + 1 < len {
            unsafe {
                nvim_fold_memmove(gap, idx + nested_len, idx + 1, len - (idx + 1));
            }
        }

        // Copy nested folds to gap starting at idx
        for i in 0..nested_len {
            let nfp = unsafe { nvim_ga_fold_at(nested, i) };
            let dst = unsafe { nvim_ga_fold_at(gap, idx + i) };
            if !nfp.is_null() && !dst.is_null() {
                unsafe { nvim_fold_copy(dst, nfp) };
            }
        }

        // Free the nested array data (but not the folds, they're now in gap)
        unsafe { nvim_ga_free_data(nested) };

        // Set new length
        unsafe { nvim_ga_set_len(gap, new_len) };
    }
}

/// Split the "i"th fold in "gap", which starts before "top" and ends below
/// "bot" in two pieces, one ending above "top" and the other starting below "bot".
///
/// The caller must first have taken care of any nested folds from "top" to "bot"!
fn fold_split_impl(buf: BufHandle, gap: GArrayHandle, i: c_int, top: LineNr, bot: LineNr) {
    if buf.is_null() || gap.is_null() || i < 0 {
        return;
    }

    // The fold continues below bot, need to split it.
    fold_insert_impl(gap, i + 1);

    // After insert, refetch the fold pointer (array may have moved)
    let fp = unsafe { nvim_ga_fold_at(gap, i) };
    let fp1 = unsafe { nvim_ga_fold_at(gap, i + 1) };
    if fp.is_null() || fp1.is_null() {
        return;
    }

    let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
    let fd_len = unsafe { nvim_fold_get_fd_len(fp) };
    let fd_flags = unsafe { nvim_fold_get_fd_flags(fp) };

    // Set up the new fold (below bot)
    let new_top = bot + 1;
    // Check for wrap around
    debug_assert!(new_top > bot);
    let new_len = fd_len - (new_top - fd_top);

    unsafe {
        nvim_fold_set_fd_top(fp1, new_top);
        nvim_fold_set_fd_len(fp1, new_len);
        nvim_fold_set_fd_flags(fp1, fd_flags);
        nvim_fold_set_fd_small(fp1, tristate::K_NONE);
        nvim_fold_set_fd_small(fp, tristate::K_NONE);
    }

    // Move nested folds below bot to new fold.
    // There can't be any between top and bot, they have been removed by the caller.
    let gap1 = unsafe { nvim_fold_get_fd_nested(fp) };
    let gap2 = unsafe { nvim_fold_get_fd_nested(fp1) };

    if !gap1.is_null() && !gap2.is_null() {
        // Find first nested fold at or below (bot + 1 - fd_top)
        let split_lnum = bot + 1 - fd_top;
        if let Some((_, found_idx)) = fold_find_with_idx(gap1, split_lnum) {
            let gap1_len = unsafe { nvim_ga_len(gap1) };
            let move_count = gap1_len - found_idx;

            if move_count > 0 {
                unsafe {
                    nvim_ga_grow_folds(gap2, move_count);

                    // Copy folds to gap2 and adjust their fd_top
                    let top_offset = new_top - fd_top;
                    for j in 0..move_count {
                        let src = nvim_ga_fold_at(gap1, found_idx + j);
                        let dst = nvim_ga_fold_at(gap2, j);
                        if !src.is_null() && !dst.is_null() {
                            nvim_fold_copy(dst, src);
                            let src_top = nvim_fold_get_fd_top(dst);
                            nvim_fold_set_fd_top(dst, src_top - top_offset);
                        }
                    }

                    nvim_ga_set_len(gap2, move_count);
                    nvim_ga_set_len(gap1, found_idx);
                }
            }
        }
    }

    // Truncate original fold
    unsafe {
        nvim_fold_set_fd_len(fp, top - fd_top);
        nvim_set_fold_changed(true);
    }
}

/// Remove folds within the range "top" to and including "bot".
///
/// Check for these situations:
///      1  2  3
///      1  2  3
/// top     2  3  4  5
///     2  3  4  5
/// bot     2  3  4  5
///        3     5  6
///        3     5  6
///
/// 1: not changed
/// 2: truncate to stop above "top"
/// 3: split in two parts, one stops above "top", other starts below "bot".
/// 4: deleted
/// 5: made to start below "bot".
/// 6: not changed
fn fold_remove_impl(wp: WinHandle, gap: GArrayHandle, top: LineNr, bot: LineNr) {
    if wp.is_null() || gap.is_null() || bot < top {
        return;
    }

    let buf = unsafe { nvim_win_get_buffer(wp) };

    loop {
        let len = unsafe { nvim_ga_len(gap) };
        if len == 0 {
            break;
        }

        // Find fold that includes top or a following one.
        let (found, fp_idx) = match fold_find_with_idx(gap, top) {
            Some((true, idx)) => (true, idx),
            Some((false, idx)) => (false, idx),
            None => break,
        };

        let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
        if fp.is_null() {
            break;
        }

        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
        let fd_len = unsafe { nvim_fold_get_fd_len(fp) };

        if found && fd_top < top {
            // 2: or 3: need to delete nested folds
            let nested = unsafe { nvim_fold_get_fd_nested(fp) };
            fold_remove_impl(wp, nested, top - fd_top, bot - fd_top);

            if fd_top + fd_len - 1 > bot {
                // 3: need to split it.
                fold_split_impl(buf, gap, fp_idx, top, bot);
            } else {
                // 2: truncate fold at "top".
                unsafe { nvim_fold_set_fd_len(fp, top - fd_top) };
            }
            unsafe { nvim_set_fold_changed(true) };
            continue;
        }

        let data = unsafe { nvim_ga_get_fold_data(gap) };
        let len = unsafe { nvim_ga_len(gap) };
        if data.is_null() || fp_idx >= len || fd_top > bot {
            // 6: Found a fold below bot, can stop looking.
            break;
        }

        if fd_top >= top {
            // Found an entry below top.
            unsafe { nvim_set_fold_changed(true) };

            if fd_top + fd_len - 1 > bot {
                // 5: Make fold that includes bot start below bot.
                let nested = unsafe { nvim_fold_get_fd_nested(fp) };
                fold_mark_adjust_recurse_impl(
                    wp,
                    nested,
                    0,
                    bot - fd_top,
                    LineNr::MAX,
                    fd_top - bot - 1,
                );
                unsafe {
                    nvim_fold_set_fd_len(fp, fd_len - (bot - fd_top + 1));
                    nvim_fold_set_fd_top(fp, bot + 1);
                }
                break;
            }

            // 4: Delete completely contained fold.
            delete_fold_entry_impl(wp, gap, fp_idx, true);
        }
    }
}

/// Merge two adjacent folds (and the nested ones in them).
///
/// This only works correctly when the folds are really adjacent! Thus "fp1"
/// must end just above "fp2".
/// The resulting fold is "fp1", nested folds are moved from "fp2" to "fp1".
/// Fold entry "fp2" in "gap" is deleted.
fn fold_merge_impl(wp: WinHandle, fp1_idx: c_int, gap: GArrayHandle, fp2_idx: c_int) {
    if wp.is_null() || gap.is_null() {
        return;
    }

    let fp1 = unsafe { nvim_ga_fold_at(gap, fp1_idx) };
    let fp2 = unsafe { nvim_ga_fold_at(gap, fp2_idx) };
    if fp1.is_null() || fp2.is_null() {
        return;
    }

    let fp1_len = unsafe { nvim_fold_get_fd_len(fp1) };
    let gap1 = unsafe { nvim_fold_get_fd_nested(fp1) };
    let gap2 = unsafe { nvim_fold_get_fd_nested(fp2) };

    // If the last nested fold in fp1 touches the first nested fold in fp2,
    // merge them recursively.
    if !gap1.is_null() && !gap2.is_null() {
        let gap1_len = unsafe { nvim_ga_len(gap1) };
        let gap2_len = unsafe { nvim_ga_len(gap2) };

        if gap1_len > 0 && gap2_len > 0 {
            // Check if last of gap1 touches first of gap2
            if let Some((true, fp3_idx)) = fold_find_with_idx(gap1, fp1_len - 1) {
                if let Some((true, _)) = fold_find_with_idx(gap2, 0) {
                    fold_merge_impl(wp, fp3_idx, gap2, 0);
                }
            }
        }
    }

    // Move nested folds in fp2 to the end of fp1.
    if !gap2.is_null() {
        let gap2_len = unsafe { nvim_ga_len(gap2) };
        if gap2_len > 0 && !gap1.is_null() {
            unsafe { nvim_ga_grow_folds(gap1, gap2_len) };

            let gap1_len = unsafe { nvim_ga_len(gap1) };
            for j in 0..gap2_len {
                let src = unsafe { nvim_ga_fold_at(gap2, j) };
                let dst = unsafe { nvim_ga_fold_at(gap1, gap1_len + j) };
                if !src.is_null() && !dst.is_null() {
                    unsafe {
                        nvim_fold_copy(dst, src);
                        // Adjust fd_top
                        let src_top = nvim_fold_get_fd_top(dst);
                        nvim_fold_set_fd_top(dst, src_top + fp1_len);
                    }
                }
            }

            unsafe {
                nvim_ga_set_len(gap1, gap1_len + gap2_len);
                nvim_ga_set_len(gap2, 0);
            }
        }
    }

    // Refetch fp1 and fp2 (may have changed due to nested operations)
    let fp1 = unsafe { nvim_ga_fold_at(gap, fp1_idx) };
    let fp2 = unsafe { nvim_ga_fold_at(gap, fp2_idx) };
    if !fp1.is_null() && !fp2.is_null() {
        let fp1_len = unsafe { nvim_fold_get_fd_len(fp1) };
        let fp2_len = unsafe { nvim_fold_get_fd_len(fp2) };
        unsafe { nvim_fold_set_fd_len(fp1, fp1_len + fp2_len) };
    }

    delete_fold_entry_impl(wp, gap, fp2_idx, true);
    unsafe { nvim_set_fold_changed(true) };
}

/// Helper function that returns both found status and index.
fn fold_find_with_idx(gap: GArrayHandle, lnum: LineNr) -> Option<(bool, c_int)> {
    if gap.is_null() {
        return None;
    }

    let len = unsafe { nvim_ga_len(gap) };
    if len == 0 {
        return None;
    }

    // Binary search
    let mut low: c_int = 0;
    let mut high: c_int = len - 1;

    while low <= high {
        let mid = c_int::midpoint(low, high);
        let fp = unsafe { nvim_ga_fold_at(gap, mid) };
        if fp.is_null() {
            return None;
        }

        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
        let fd_len = unsafe { nvim_fold_get_fd_len(fp) };

        if fd_top > lnum {
            high = mid - 1;
        } else if fd_top + fd_len <= lnum {
            low = mid + 1;
        } else {
            // lnum is inside this fold
            return Some((true, mid));
        }
    }

    // Return fold at `low` index (first fold below lnum)
    Some((false, low))
}

/// Update line numbers of folds in a garray for inserted/deleted lines.
///
/// Recursive helper used by foldMarkAdjust.
fn fold_mark_adjust_recurse_impl(
    wp: WinHandle,
    gap: GArrayHandle,
    line1: LineNr,
    line2: LineNr,
    amount: LineNr,
    amount_after: LineNr,
) {
    if wp.is_null() || gap.is_null() {
        return;
    }

    let len = unsafe { nvim_ga_len(gap) };
    if len == 0 {
        return;
    }

    // Determine top boundary
    // In Insert mode an inserted line at the top of a fold is considered part
    // of the fold, otherwise it isn't.
    // Note: We don't have access to State here, so we use simpler logic.
    // The C code checks (State & MODE_INSERT) && amount == 1 && line2 == MAXLNUM
    // For now, we use line1 as the top.
    let top = line1;

    // Find the fold containing or just below line1
    let Some((_, start_idx)) = fold_find_with_idx(gap, line1) else {
        return;
    };

    // Adjust all folds at or below "line1" that are affected.
    let mut i = start_idx;
    while i < unsafe { nvim_ga_len(gap) } {
        let fp = unsafe { nvim_ga_fold_at(gap, i) };
        if fp.is_null() {
            break;
        }

        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
        let fd_len = unsafe { nvim_fold_get_fd_len(fp) };
        let last = fd_top + fd_len - 1;

        // 1. fold completely above line1: nothing to do
        if last < line1 {
            i += 1;
            continue;
        }

        // 6. fold below line2: only adjust for amount_after
        if fd_top > line2 {
            if amount_after == 0 {
                break;
            }
            unsafe { nvim_fold_set_fd_top(fp, fd_top + amount_after) };
        } else if fd_top >= top && last <= line2 {
            // 4. fold completely contained in range
            if amount == LineNr::MAX {
                // Deleting lines: delete the fold completely
                delete_fold_entry_impl(wp, gap, i, true);
                // Don't increment i, the next fold is now at this index
                continue;
            }
            unsafe { nvim_fold_set_fd_top(fp, fd_top + amount) };
        } else if fd_top < top {
            // 2 or 3: need to correct nested folds too
            let nested = unsafe { nvim_fold_get_fd_nested(fp) };
            fold_mark_adjust_recurse_impl(
                wp,
                nested,
                line1 - fd_top,
                line2 - fd_top,
                amount,
                amount_after,
            );

            if last <= line2 {
                // 2. fold contains line1, line2 is below fold
                if amount == LineNr::MAX {
                    unsafe { nvim_fold_set_fd_len(fp, line1 - fd_top) };
                } else {
                    unsafe { nvim_fold_set_fd_len(fp, fd_len + amount) };
                }
            } else {
                // 3. fold contains both line1 and line2
                unsafe { nvim_fold_set_fd_len(fp, fd_len + amount_after) };
            }
        } else {
            // 5. line1 is inside fold, line2 is below fold end
            if amount == LineNr::MAX {
                // Move nested folds
                let nested = unsafe { nvim_fold_get_fd_nested(fp) };
                fold_mark_adjust_recurse_impl(
                    wp,
                    nested,
                    0,
                    line2 - fd_top,
                    LineNr::MAX,
                    fd_top - line2 - 1,
                );
                unsafe {
                    nvim_fold_set_fd_len(fp, fd_len - (line2 - fd_top + 1));
                    nvim_fold_set_fd_top(fp, line2 + 1 + amount_after);
                }
            } else {
                unsafe { nvim_fold_set_fd_top(fp, fd_top + amount) };
            }
        }
        i += 1;
    }
}

// ============================================================================
// Phase 1: Fold Tree Manipulation FFI Exports
// ============================================================================

/// Insert a new fold in "gap" at position "i".
///
/// # Safety
/// The `gap` parameter must be a valid `garray_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldInsert(gap: GArrayHandle, i: c_int) {
    fold_insert_impl(gap, i);
}

/// Delete fold "idx" from growarray "gap".
///
/// When `recursive` is true, also delete all the folds contained in it.
/// When `recursive` is false, contained folds are moved one level up.
///
/// # Safety
/// The `wp` and `gap` parameters must be valid pointers or null.
#[no_mangle]
pub extern "C" fn rs_deleteFoldEntry(
    wp: WinHandle,
    gap: GArrayHandle,
    idx: c_int,
    recursive: bool,
) {
    delete_fold_entry_impl(wp, gap, idx, recursive);
}

/// Split the "i"th fold in "gap".
///
/// # Safety
/// The `buf` and `gap` parameters must be valid pointers or null.
#[no_mangle]
pub extern "C" fn rs_foldSplit(
    buf: BufHandle,
    gap: GArrayHandle,
    i: c_int,
    top: LineNr,
    bot: LineNr,
) {
    fold_split_impl(buf, gap, i, top, bot);
}

/// Remove folds within the range "top" to and including "bot".
///
/// # Safety
/// The `wp` and `gap` parameters must be valid pointers or null.
#[no_mangle]
pub extern "C" fn rs_foldRemove(wp: WinHandle, gap: GArrayHandle, top: LineNr, bot: LineNr) {
    fold_remove_impl(wp, gap, top, bot);
}

/// Merge two adjacent folds.
///
/// # Safety
/// The `wp` and `gap` parameters must be valid pointers or null.
#[no_mangle]
pub extern "C" fn rs_foldMerge(wp: WinHandle, fp1_idx: c_int, gap: GArrayHandle, fp2_idx: c_int) {
    fold_merge_impl(wp, fp1_idx, gap, fp2_idx);
}

/// Update line numbers of folds for inserted/deleted lines (recursive).
///
/// # Safety
/// The `wp` and `gap` parameters must be valid pointers or null.
#[no_mangle]
pub extern "C" fn rs_foldMarkAdjustRecurse(
    wp: WinHandle,
    gap: GArrayHandle,
    line1: LineNr,
    line2: LineNr,
    amount: LineNr,
    amount_after: LineNr,
) {
    fold_mark_adjust_recurse_impl(wp, gap, line1, line2, amount, amount_after);
}

#[cfg(test)]
mod tests {
    // Tests require FFI stubs which aren't available in pure Rust testing.
    // Integration testing is done via the full Neovim build.
}
