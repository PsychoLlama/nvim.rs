//! Fold method checks and fold state queries for Neovim
//!
//! This crate provides Rust implementations of folding-related functions
//! from `src/nvim/fold.c`. It uses an opaque handle pattern where
//! `win_T*` pointers are treated as opaque handles, with field access
//! done through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)] // Character literals are safe ASCII values

pub mod commands;
pub mod display;
pub mod level;
pub mod markers;
pub mod methods;
pub mod session;
pub mod tree;
pub mod update;

// Re-export key types
pub use commands::{
    CreateFoldCmd, DeleteFoldCmd, FoldCmdContext, FoldCmdResult, FoldNavCmd, FoldNavResult, FoldOp,
};
pub use display::{FoldColumnChar, FoldColumnConfig, FoldDisplayInfo, FoldTextComponents};
pub use methods::{FoldLevelResult, FoldMethod};
pub use tree::{FoldRange, FoldState, FoldTreeInfo, FoldUpdateRequest, FoldUpdateType};

use std::ffi::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use nvim_buffer::BufHandle;
use nvim_window::WinHandle;

/// Line number type (matches linenr_T in C).
pub type LineNr = i32;

// ============================================================================
// Statics migrated from fold_shim.c (Phase 5, Pass 5)
// ============================================================================

/// Whether any folds changed during the last update (fold_changed in C).
static FOLD_CHANGED: AtomicBool = AtomicBool::new(false);

/// Top of the invalid fold range during an update (invalid_top in C).
static INVALID_TOP: AtomicI32 = AtomicI32::new(0);

/// Bottom of the invalid fold range during an update (invalid_bot in C).
static INVALID_BOT: AtomicI32 = AtomicI32::new(0);

/// Line number whose level is cached for foldlevel() chicken-egg problem (prev_lnum in C).
static PREV_LNUM: AtomicI32 = AtomicI32::new(0);

/// Fold level for prev_lnum; -1 means not available (prev_lnum_lvl in C).
static PREV_LNUM_LVL: AtomicI32 = AtomicI32::new(-1);

#[inline]
pub(crate) fn fold_changed() -> bool {
    FOLD_CHANGED.load(Ordering::Relaxed)
}

#[inline]
pub(crate) fn set_fold_changed(val: bool) {
    FOLD_CHANGED.store(val, Ordering::Relaxed);
}

#[inline]
pub(crate) fn invalid_top() -> LineNr {
    INVALID_TOP.load(Ordering::Relaxed)
}

#[inline]
pub(crate) fn set_invalid_top(val: LineNr) {
    INVALID_TOP.store(val, Ordering::Relaxed);
}

#[inline]
pub(crate) fn invalid_bot() -> LineNr {
    INVALID_BOT.load(Ordering::Relaxed)
}

#[inline]
pub(crate) fn set_invalid_bot(val: LineNr) {
    INVALID_BOT.store(val, Ordering::Relaxed);
}

#[inline]
pub(crate) fn prev_lnum() -> LineNr {
    PREV_LNUM.load(Ordering::Relaxed)
}

#[inline]
pub(crate) fn set_prev_lnum(val: LineNr) {
    PREV_LNUM.store(val, Ordering::Relaxed);
}

#[inline]
pub(crate) fn prev_lnum_lvl() -> c_int {
    PREV_LNUM_LVL.load(Ordering::Relaxed)
}

#[inline]
pub(crate) fn set_prev_lnum_lvl(val: c_int) {
    PREV_LNUM_LVL.store(val, Ordering::Relaxed);
}

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

/// Result struct matching C `foldinfo_T` layout.
///
/// Used by `fold_info()` to return fold information.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldInfoResult {
    /// Line number where fold starts.
    pub fi_lnum: LineNr,
    /// Fold level (0 means no fold info).
    pub fi_level: c_int,
    /// Lowest fold level that starts in the same line.
    pub fi_low_level: c_int,
    /// Number of folded lines (0 if not folded).
    pub fi_lines: LineNr,
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
///
/// Note: In C these are stored as `char` but used as integers in comparisons.
/// We use `c_int` here for convenience in Rust arithmetic and comparisons.
pub mod fold_flags {
    use std::ffi::c_int;

    /// Fold is open (nested ones can be closed).
    pub const FD_OPEN: c_int = 0;
    /// Fold is closed.
    pub const FD_CLOSED: c_int = 1;
    /// Fold depends on 'foldlevel' (nested folds too).
    pub const FD_LEVEL: c_int = 2;
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
    fn nvim_fold_get_fd_flags(fp: FoldHandle) -> c_int;

    /// Get the w_foldinvalid field from a window (reserved for future use).
    #[allow(dead_code)]
    fn nvim_win_get_w_foldinvalid(wp: WinHandle) -> bool;

    /// Set the fd_flags field of a fold.
    fn nvim_fold_set_fd_flags(fp: FoldHandle, flags: c_int);

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
    fn plines_win_nofold(wp: WinHandle, lnum: LineNr) -> c_int;

    // ========================================================================
    // Phase 1: Foundation function accessors
    // ========================================================================

    /// Initialize the folds garray for a window.
    fn nvim_ga_init_folds(gap: GArrayHandle);

    // ========================================================================
    // Phase 2: Fold navigation accessors
    // ========================================================================

    /// Get the w_lines_valid field from a window.
    fn nvim_win_get_lines_valid(wp: WinHandle) -> c_int;

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

    /// Free the ga_data pointer of a garray (for nested folds).
    fn nvim_ga_free_data(gap: GArrayHandle);

    /// Clear (free and reset) a garray: frees ga_data, resets ga_len/ga_maxlen.
    fn nvim_ga_clear(gap: GArrayHandle);

    // ========================================================================
    // Phase 2: Fold State Management accessors
    // ========================================================================

    /// Get the w_fold_manual field from a window.
    fn nvim_win_get_w_fold_manual(wp: WinHandle) -> c_int;

    /// Set the w_fold_manual field in a window.
    fn nvim_win_set_w_fold_manual(wp: WinHandle, val: bool);

    /// Call changed_window_setting for a window.
    fn changed_window_setting(wp: WinHandle);

    /// Emit the "no fold found" error message.
    fn nvim_emsg_nofold();

    /// Get the w_p_scb (scrollbind) field from a window.
    fn nvim_win_get_p_scb(wp: WinHandle) -> bool;

    /// Get the cursor lnum from a window.
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> LineNr;

    /// Get the first window in the current tab.
    fn nvim_get_first_win_in_tab() -> WinHandle;

    /// Get the next window in a tab (from w_next).
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;

    /// Wrapper for diff_lnum_win.
    #[link_name = "rs_diff_lnum_win"]
    fn nvim_diff_lnum_win(lnum: LineNr, wp: WinHandle) -> LineNr;

    /// Set the w_p_fdl (foldlevel) field in a window.
    fn nvim_win_set_p_fdl(wp: WinHandle, fdl: c_int);

    // ========================================================================
    // Phase 3: Fold Navigation FFI
    // ========================================================================

    /// Set the pc mark (for jump list).
    fn nvim_setpcmark();

    /// Get the p_fcl option value (pointer to NUL-terminated string).
    fn nvim_get_p_fcl() -> *const c_char;
}

// ============================================================================
// Fold Method Checks
// ============================================================================

/// Check if 'foldmethod' is "manual".
///
/// Manual folding requires explicit fold creation by the user.
/// The check is `wp->w_p_fdm[3] == 'u'` (matching "man**u**al").
#[inline]
pub(crate) fn foldmethod_is_manual_impl(wp: WinHandle) -> bool {
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
pub(crate) fn foldmethod_is_indent_impl(wp: WinHandle) -> bool {
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
pub(crate) fn foldmethod_is_expr_impl(wp: WinHandle) -> bool {
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
pub(crate) fn foldmethod_is_marker_impl(wp: WinHandle) -> bool {
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
pub(crate) fn foldmethod_is_syntax_impl(wp: WinHandle) -> bool {
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
pub(crate) fn foldmethod_is_diff_impl(wp: WinHandle) -> bool {
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
// delete_fold_recurse_impl (Phase 5 Pass 5)
// ============================================================================

/// Recursively delete nested folds in a garray, then clear the garray.
///
/// Equivalent to C's GA_DEEP_CLEAR with DELETE_FOLD_NESTED: for each fold,
/// recurse into its fd_nested array, then call ga_clear on the top-level array.
pub(crate) fn delete_fold_recurse_impl(gap: GArrayHandle) {
    if gap.is_null() {
        return;
    }
    let len = unsafe { nvim_ga_len(gap) };
    for i in 0..len {
        let fp = unsafe { nvim_ga_fold_at(gap, i) };
        if !fp.is_null() {
            let nested = unsafe { nvim_fold_get_fd_nested(fp) };
            if !nested.is_null() {
                delete_fold_recurse_impl(nested);
            }
        }
    }
    unsafe { nvim_ga_clear(gap) };
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
pub(crate) fn fold_level_win_impl(wp: WinHandle, lnum: LineNr) -> c_int {
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
    checkupdate_impl(wp);

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

    let lines_valid = unsafe { nvim_win_get_lines_valid(win) };

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
pub(crate) fn set_small_maybe_impl(gap: GArrayHandle) {
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
    checkupdate_impl(win);

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
            count += unsafe { plines_win_nofold(wp, fd_top + lnum_off + n) };
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

/// Full hasFoldingWin Rust export (Phase 5 Pass 5).
///
/// Returns true if `lnum` is in a closed fold. Writes first/last fold line
/// to `firstp`/`lastp` (if non-null). Writes fold info to `infop` (if non-null).
///
/// # Safety
/// All pointer arguments must be null or valid pointers.
#[no_mangle]
pub unsafe extern "C" fn hasFoldingWin(
    win: WinHandle,
    lnum: LineNr,
    firstp: *mut LineNr,
    lastp: *mut LineNr,
    cache: bool,
    infop: *mut c_void,
) -> bool {
    let result = has_folding_win_impl(win, lnum, cache);

    if !infop.is_null() {
        // foldinfo_T layout: fi_lnum (i32), fi_level (i32), fi_low_level (i32), fi_lines (i32)
        let fi = infop.cast::<FoldInfoResult>();
        (*fi).fi_lnum = result.fi_lnum;
        (*fi).fi_level = result.fi_level;
        (*fi).fi_low_level = result.fi_low_level;
        // fi_lines is not set by hasFoldingWin (it's for fold_info())
    }

    if result.has_folding != 0 {
        if !lastp.is_null() {
            *lastp = result.last;
        }
        if !firstp.is_null() {
            *firstp = result.first;
        }
        return true;
    }

    false
}

/// hasFolding Rust export (Phase 5 Pass 5).
///
/// Returns true if `lnum` is in a closed fold in `win`.
/// Sets `*firstp`/`*lastp` to first/last line of the fold.
///
/// # Safety
/// `win` must be a valid win_T* or null. `firstp`/`lastp` must be null or valid.
#[no_mangle]
pub unsafe extern "C" fn hasFolding(
    win: WinHandle,
    lnum: LineNr,
    firstp: *mut LineNr,
    lastp: *mut LineNr,
) -> bool {
    hasFoldingWin(win, lnum, firstp, lastp, true, std::ptr::null_mut())
}

/// nvim_hasFolding Rust export (Phase 5 Pass 5): integer return for Rust FFI.
///
/// # Safety
/// `wp` must be a valid win_T* or null.
#[no_mangle]
pub unsafe extern "C" fn nvim_hasFolding(
    wp: WinHandle,
    lnum: LineNr,
    firstp: *mut LineNr,
    lastp: *mut LineNr,
) -> c_int {
    c_int::from(hasFolding(wp, lnum, firstp, lastp))
}

/// deleteFoldRecurse Rust export (Phase 5 Pass 5).
///
/// Recursively delete nested folds in a garray, then clear it.
/// Matches GA_DEEP_CLEAR behavior from the original C implementation.
///
/// # Safety
/// `gap` must be a valid garray_T* or null. `bp` is accepted for ABI
/// compatibility but not used (the Rust implementation does not need it).
#[no_mangle]
pub unsafe extern "C" fn deleteFoldRecurse(_bp: BufHandle, gap: GArrayHandle) {
    delete_fold_recurse_impl(gap);
}

/// nvim_foldUpdateAll_c Rust export (Phase 5 Pass 5).
///
/// Sets w_foldinvalid and schedules a NOT_VALID redraw for the window.
///
/// # Safety
/// `win` must be a valid win_T* or null.
#[no_mangle]
pub unsafe extern "C" fn nvim_foldUpdateAll_c(win: WinHandle) {
    fold_update_all_impl(win);
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
pub(crate) fn fold_insert_impl(gap: GArrayHandle, i: c_int) {
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
pub(crate) fn delete_fold_entry_impl(
    wp: WinHandle,
    gap: GArrayHandle,
    idx: c_int,
    recursive: bool,
) {
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

    if recursive || nested_len == 0 {
        // Recursively delete the contained folds
        if !nested.is_null() {
            delete_fold_recurse_impl(nested);
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
pub(crate) fn fold_split_impl(
    buf: BufHandle,
    gap: GArrayHandle,
    i: c_int,
    top: LineNr,
    bot: LineNr,
) {
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
        set_fold_changed(true);
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
pub(crate) fn fold_remove_impl(wp: WinHandle, gap: GArrayHandle, top: LineNr, bot: LineNr) {
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
            set_fold_changed(true);
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
            set_fold_changed(true);

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
pub(crate) fn fold_merge_impl(wp: WinHandle, fp1_idx: c_int, gap: GArrayHandle, fp2_idx: c_int) {
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
    set_fold_changed(true);
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
pub(crate) fn fold_mark_adjust_recurse_impl(
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

    // In Insert mode an inserted line at the top of a fold is considered part
    // of the fold, otherwise it isn't.
    let top = if (unsafe { nvim_get_state() } & MODE_INSERT) != 0 && amount == 1 && line2 == MAXLNUM
    {
        line1 + 1
    } else {
        line1
    };

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
            // 5. fold is below line1 and contains line2; need to
            // correct nested folds too
            let nested = unsafe { nvim_fold_get_fd_nested(fp) };
            if amount == LineNr::MAX {
                fold_mark_adjust_recurse_impl(
                    wp,
                    nested,
                    0,
                    line2 - fd_top,
                    amount,
                    amount_after + (fd_top - top),
                );
                unsafe {
                    nvim_fold_set_fd_len(fp, fd_len - (line2 - fd_top + 1));
                    nvim_fold_set_fd_top(fp, line1);
                }
            } else {
                fold_mark_adjust_recurse_impl(
                    wp,
                    nested,
                    0,
                    line2 - fd_top,
                    amount,
                    amount_after - amount,
                );
                unsafe {
                    nvim_fold_set_fd_len(fp, fd_len + amount_after - amount);
                    nvim_fold_set_fd_top(fp, fd_top + amount);
                }
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

// ============================================================================
// Phase 2: Fold State Management Functions
// ============================================================================

/// Flags for open/close done state.
mod done_flags {
    use std::ffi::c_int;

    /// No action taken.
    pub const DONE_NOTHING: c_int = 0;
    /// Found a fold.
    pub const DONE_FOLD: c_int = 1;
    /// Opened or closed a fold.
    pub const DONE_ACTION: c_int = 2;
}

/// Maximum line number constant.
const MAXLNUM: LineNr = LineNr::MAX;

/// Open or close the fold in window "wp" which contains "lnum".
///
/// "donep", when not NULL, points to flag that is set to DONE_FOLD when some
/// fold was found and to DONE_ACTION when some fold was opened or closed.
/// When "donep" is NULL give an error message when no fold was found for
/// "lnum", but only if "wp" is "curwin".
///
/// Returns the line number of the next line that could be closed.
/// It's only valid when "opening" is true!
#[allow(clippy::too_many_lines)]
fn set_manual_fold_win_impl(
    wp: WinHandle,
    lnum: LineNr,
    opening: bool,
    recurse: bool,
    done_out: *mut c_int,
) -> LineNr {
    if wp.is_null() {
        return MAXLNUM;
    }

    // checkupdate(wp)
    checkupdate_impl(wp);

    let mut level = 0;
    let mut use_level = false;
    let mut found_fold = false;
    let mut next = MAXLNUM;
    let mut off: LineNr = 0;
    let mut done: c_int = done_flags::DONE_NOTHING;
    let mut found_idx: Option<(GArrayHandle, c_int)> = None;
    let mut lnum = lnum;

    // Find the fold, open or close it.
    let mut gap = unsafe { nvim_win_get_folds(wp) };

    loop {
        // Find fold containing lnum
        let Some((found, fp_idx)) = fold_find_with_idx(gap, lnum) else {
            // No fold found at all
            break;
        };

        if !found {
            // If there is a following fold, continue there next time.
            let len = unsafe { nvim_ga_len(gap) };
            if fp_idx < len {
                let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
                if !fp.is_null() {
                    let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
                    next = fd_top + off;
                }
            }
            break;
        }

        let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
        if fp.is_null() {
            break;
        }

        // lnum is inside this fold
        found_fold = true;

        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
        let fd_flags = unsafe { nvim_fold_get_fd_flags(fp) };
        let nested = unsafe { nvim_fold_get_fd_nested(fp) };

        // If there is a following fold, continue there next time.
        let len = unsafe { nvim_ga_len(gap) };
        if fp_idx + 1 < len {
            let fp_next = unsafe { nvim_ga_fold_at(gap, fp_idx + 1) };
            if !fp_next.is_null() {
                let next_top = unsafe { nvim_fold_get_fd_top(fp_next) };
                next = next_top + off;
            }
        }

        let fdl = unsafe { nvim_win_get_p_fdl(wp) };

        // Change from level-dependent folding to manual.
        if use_level || fd_flags == fold_flags::FD_LEVEL {
            use_level = true;
            let new_flags = if level >= fdl {
                fold_flags::FD_CLOSED
            } else {
                fold_flags::FD_OPEN
            };
            unsafe { nvim_fold_set_fd_flags(fp, new_flags) };

            // Set nested folds to FD_LEVEL
            if !nested.is_null() {
                unsafe {
                    let nested_len = nvim_ga_len(nested);
                    for j in 0..nested_len {
                        let nfp = nvim_ga_fold_at(nested, j);
                        if !nfp.is_null() {
                            nvim_fold_set_fd_flags(nfp, fold_flags::FD_LEVEL);
                        }
                    }
                }
            }
        }

        // Re-read fd_flags since we may have changed it
        let fd_flags = unsafe { nvim_fold_get_fd_flags(fp) };

        // Simple case: Close recursively means closing the fold.
        if !opening && recurse {
            if fd_flags != fold_flags::FD_CLOSED {
                done |= done_flags::DONE_ACTION;
                unsafe { nvim_fold_set_fd_flags(fp, fold_flags::FD_CLOSED) };
            }
        } else if fd_flags == fold_flags::FD_CLOSED {
            // When opening, open topmost closed fold.
            if opening {
                unsafe { nvim_fold_set_fd_flags(fp, fold_flags::FD_OPEN) };
                done |= done_flags::DONE_ACTION;
                if recurse {
                    fold_open_nested_impl(fp);
                }
            }
            break;
        }

        // fold is open, check nested folds
        found_idx = Some((gap, fp_idx));
        gap = nested;
        lnum -= fd_top;
        off += fd_top;
        level += 1;
    }

    if found_fold {
        // When closing and not recurse, close deepest open fold.
        if !opening {
            if let Some((saved_gap, saved_idx)) = found_idx {
                let found_fp = unsafe { nvim_ga_fold_at(saved_gap, saved_idx) };
                if !found_fp.is_null() {
                    unsafe { nvim_fold_set_fd_flags(found_fp, fold_flags::FD_CLOSED) };
                    done |= done_flags::DONE_ACTION;
                }
            }
        }
        unsafe { nvim_win_set_w_fold_manual(wp, true) };
        if done & done_flags::DONE_ACTION != 0 {
            unsafe { changed_window_setting(wp) };
        }
        done |= done_flags::DONE_FOLD;
    } else if done_out.is_null() {
        // Emit error if donep is NULL and this is curwin
        let curwin = unsafe { nvim_get_curwin() };
        if wp == curwin {
            unsafe { nvim_emsg_nofold() };
        }
    }

    if !done_out.is_null() {
        unsafe { *done_out |= done };
    }

    next
}

/// Open or close fold for current window at position `pos`.
/// Repeat "count" times.
fn set_fold_repeat_impl(lnum: LineNr, count: c_int, do_open: bool) {
    let curwin = unsafe { nvim_get_curwin() };

    for n in 0..count {
        let mut done: c_int = done_flags::DONE_NOTHING;
        set_manual_fold_win_impl(curwin, lnum, do_open, false, &raw mut done);
        if done & done_flags::DONE_ACTION == 0 {
            // Only give an error message when no fold could be opened.
            if n == 0 && done & done_flags::DONE_FOLD == 0 {
                unsafe { nvim_emsg_nofold() };
            }
            break;
        }
    }
}

/// Open or close the fold in the current window which contains "lnum".
/// Also does this for other windows in diff mode when needed.
fn set_manual_fold_impl(
    lnum: LineNr,
    opening: bool,
    recurse: bool,
    done_out: *mut c_int,
) -> LineNr {
    let curwin = unsafe { nvim_get_curwin() };

    // Check if in diff mode with scrollbind
    if foldmethod_is_diff_impl(curwin) && unsafe { nvim_win_get_p_scb(curwin) } {
        let cursor_lnum = unsafe { nvim_win_get_cursor_lnum(curwin) };

        // Do the same operation in other windows in diff mode.
        let mut wp = unsafe { nvim_get_first_win_in_tab() };
        while !wp.is_null() {
            if wp != curwin && foldmethod_is_diff_impl(wp) && unsafe { nvim_win_get_p_scb(wp) } {
                let dlnum = unsafe { nvim_diff_lnum_win(cursor_lnum, wp) };
                if dlnum != 0 {
                    set_manual_fold_win_impl(wp, dlnum, opening, recurse, std::ptr::null_mut());
                }
            }
            wp = unsafe { nvim_win_get_next(wp) };
        }
    }

    set_manual_fold_win_impl(curwin, lnum, opening, recurse, done_out)
}

/// Set new foldlevel for a window.
fn new_fold_level_win_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    checkupdate_impl(wp);

    let w_fold_manual = unsafe { nvim_win_get_w_fold_manual(wp) };
    if w_fold_manual != 0 {
        // Set all flags for the first level of folds to FD_LEVEL.
        unsafe {
            let gap = nvim_win_get_folds(wp);
            let len = nvim_ga_len(gap);
            for i in 0..len {
                let fp = nvim_ga_fold_at(gap, i);
                if !fp.is_null() {
                    nvim_fold_set_fd_flags(fp, fold_flags::FD_LEVEL);
                }
            }
            nvim_win_set_w_fold_manual(wp, false);
        }
    }

    unsafe { changed_window_setting(wp) };
}

/// Set new foldlevel for current window.
fn new_fold_level_impl() {
    let curwin = unsafe { nvim_get_curwin() };
    new_fold_level_win_impl(curwin);

    // If in diff mode with scrollbind, set the same foldlevel in other windows
    if foldmethod_is_diff_impl(curwin) && unsafe { nvim_win_get_p_scb(curwin) } {
        let fdl = unsafe { nvim_win_get_p_fdl(curwin) };

        let mut wp = unsafe { nvim_get_first_win_in_tab() };
        while !wp.is_null() {
            if wp != curwin && foldmethod_is_diff_impl(wp) && unsafe { nvim_win_get_p_scb(wp) } {
                // Set w_p_fdl
                unsafe { nvim_win_set_p_fdl(wp, fdl) };
                new_fold_level_win_impl(wp);
            }
            wp = unsafe { nvim_win_get_next(wp) };
        }
    }
}

// ============================================================================
// Phase 2: Fold State Management FFI Exports
// ============================================================================

/// Open or close fold in a specific window.
///
/// # Safety
/// The `wp` and `donep` parameters must be valid pointers or null.
#[no_mangle]
pub extern "C" fn rs_setManualFoldWin(
    wp: WinHandle,
    lnum: LineNr,
    opening: bool,
    recurse: bool,
    donep: *mut c_int,
) -> LineNr {
    set_manual_fold_win_impl(wp, lnum, opening, recurse, donep)
}

/// Open or close fold for current window, with diff mode handling.
///
/// # Safety
/// The `donep` parameter must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_setManualFold(
    lnum: LineNr,
    opening: bool,
    recurse: bool,
    donep: *mut c_int,
) -> LineNr {
    set_manual_fold_impl(lnum, opening, recurse, donep)
}

/// Open or close fold repeatedly.
///
/// # Safety
/// This function is safe to call from C.
#[no_mangle]
pub extern "C" fn rs_setFoldRepeat(lnum: LineNr, count: c_int, do_open: bool) {
    set_fold_repeat_impl(lnum, count, do_open);
}

/// Set new foldlevel for a window.
///
/// # Safety
/// The `wp` parameter must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_newFoldLevelWin(wp: WinHandle) {
    new_fold_level_win_impl(wp);
}

/// Set new foldlevel for current window.
///
/// # Safety
/// This function is safe to call from C.
#[no_mangle]
pub extern "C" fn rs_newFoldLevel() {
    new_fold_level_impl();
}

// ============================================================================
// Phase 3: Fold Creation and Deletion
// ============================================================================

extern "C" {
    /// Initialize a garray with specified itemsize and growsize.
    fn nvim_ga_init_folds_ex(gap: GArrayHandle, itemsize: c_int, growsize: c_int);

    /// Get the ga_itemsize field from a garray.
    fn nvim_ga_get_itemsize(gap: GArrayHandle) -> c_int;

    /// Get the ga_growsize field from a garray.
    fn nvim_ga_get_growsize(gap: GArrayHandle) -> c_int;

    /// Check if a garray is empty.
    fn nvim_ga_is_empty(gap: GArrayHandle) -> bool;

    /// Set the w_foldinvalid field in a window.
    fn nvim_win_set_w_foldinvalid(wp: WinHandle, val: bool);

}

/// Deep copy a garray of folds.
///
/// This recursively clones all folds and their nested folds.
fn clone_fold_grow_array_impl(from: GArrayHandle, to: GArrayHandle) {
    if from.is_null() || to.is_null() {
        return;
    }

    unsafe {
        let itemsize = nvim_ga_get_itemsize(from);
        let growsize = nvim_ga_get_growsize(from);
        nvim_ga_init_folds_ex(to, itemsize, growsize);

        if nvim_ga_is_empty(from) {
            return;
        }

        let from_len = nvim_ga_len(from);
        nvim_ga_grow_folds(to, from_len);

        for i in 0..from_len {
            let from_fp = nvim_ga_fold_at(from, i);
            let to_fp = nvim_ga_fold_at(to, i);
            if from_fp.is_null() || to_fp.is_null() {
                continue;
            }

            // Copy basic fields
            let fd_top = nvim_fold_get_fd_top(from_fp);
            let fd_len = nvim_fold_get_fd_len(from_fp);
            let fd_flags = nvim_fold_get_fd_flags(from_fp);
            let fd_small = nvim_fold_get_fd_small(from_fp);

            nvim_fold_set_fd_top(to_fp, fd_top);
            nvim_fold_set_fd_len(to_fp, fd_len);
            nvim_fold_set_fd_flags(to_fp, fd_flags);
            nvim_fold_set_fd_small(to_fp, fd_small);

            // Recursively clone nested folds
            let from_nested = nvim_fold_get_fd_nested(from_fp);
            let to_nested = nvim_fold_get_fd_nested(to_fp);
            clone_fold_grow_array_impl(from_nested, to_nested);

            // Increment ga_len
            let to_len = nvim_ga_len(to);
            nvim_ga_set_len(to, to_len + 1);
        }
    }
}

/// Copy folding state from one window to another.
fn copy_folding_state_impl(wp_from: WinHandle, wp_to: WinHandle) {
    if wp_from.is_null() || wp_to.is_null() {
        return;
    }

    unsafe {
        // Copy w_fold_manual
        let fold_manual = nvim_win_get_w_fold_manual(wp_from);
        nvim_win_set_w_fold_manual(wp_to, fold_manual != 0);

        // Copy w_foldinvalid
        let foldinvalid = nvim_win_get_w_foldinvalid(wp_from);
        nvim_win_set_w_foldinvalid(wp_to, foldinvalid);

        // Clone the folds
        let from_folds = nvim_win_get_folds(wp_from);
        let to_folds = nvim_win_get_folds(wp_to);
        clone_fold_grow_array_impl(from_folds, to_folds);
    }
}

/// Remove all folding for a window.
fn clear_folding_impl(win: WinHandle) {
    if win.is_null() {
        return;
    }

    let folds = unsafe { nvim_win_get_folds(win) };
    delete_fold_recurse_impl(folds);
    unsafe { nvim_win_set_w_foldinvalid(win, false) };
}

// ============================================================================
// Phase 3: FFI Exports
// ============================================================================

/// Deep copy a garray of folds.
///
/// # Safety
/// The `from` and `to` parameters must be valid `garray_T*` pointers or null.
#[no_mangle]
pub extern "C" fn rs_cloneFoldGrowArray(from: GArrayHandle, to: GArrayHandle) {
    clone_fold_grow_array_impl(from, to);
}

/// Copy folding state from one window to another.
///
/// # Safety
/// The `wp_from` and `wp_to` parameters must be valid `win_T*` pointers or null.
#[no_mangle]
pub extern "C" fn rs_copyFoldingState(wp_from: WinHandle, wp_to: WinHandle) {
    copy_folding_state_impl(wp_from, wp_to);
}

/// Remove all folding for a window.
///
/// # Safety
/// The `win` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_clearFolding(win: WinHandle) {
    clear_folding_impl(win);
}

// ============================================================================
// Phase 4: Fold Update System
// ============================================================================

extern "C" {
    /// Get the state variable (MODE_INSERT, etc.).
    fn nvim_get_state() -> c_int;

    /// Redraw later with specified type.
    #[link_name = "redraw_later"]
    fn nvim_redraw_later(wp: WinHandle, redraw_type: c_int);

}

/// UPD_NOT_VALID redraw type.
const UPD_NOT_VALID: c_int = 40;

/// MODE_INSERT state flag.
const MODE_INSERT: c_int = 0x10;

/// Update all lines in a window for folding.
fn fold_update_all_impl(win: WinHandle) {
    if win.is_null() {
        return;
    }

    unsafe {
        nvim_win_set_w_foldinvalid(win, true);
        nvim_redraw_later(win, UPD_NOT_VALID);
    }
}

/// Updates folds when leaving insert-mode.
fn fold_update_after_insert_impl() {
    let curwin = unsafe { nvim_get_curwin() };

    // foldmethod=manual: No need to update.
    // These foldmethods are too slow, do not auto-update on insert-leave.
    if foldmethod_is_manual_impl(curwin)
        || foldmethod_is_syntax_impl(curwin)
        || foldmethod_is_expr_impl(curwin)
    {
        return;
    }

    fold_update_all_impl(curwin);
    fold_open_cursor_impl();
}

/// Update line numbers of folds for inserted/deleted lines.
fn fold_mark_adjust_impl(
    wp: WinHandle,
    mut line1: LineNr,
    mut line2: LineNr,
    amount: LineNr,
    amount_after: LineNr,
) {
    if wp.is_null() {
        return;
    }

    // If deleting marks from line1 to line2, but not deleting all those
    // lines, set line2 so that only deleted lines have their folds removed.
    if amount == MAXLNUM && line2 >= line1 && line2 - line1 >= -amount_after {
        line2 = line1 - amount_after - 1;
    }
    if line2 < line1 {
        line2 = line1;
    }
    // If appending a line in Insert mode, it should be included in the fold
    // just above the line.
    let state = unsafe { nvim_get_state() };
    if (state & MODE_INSERT) != 0 && amount == 1 && line2 == MAXLNUM {
        line1 -= 1;
    }

    let gap = unsafe { nvim_win_get_folds(wp) };
    fold_mark_adjust_recurse_impl(wp, gap, line1, line2, amount, amount_after);
}

// ============================================================================
// Phase 4: FFI Exports
// ============================================================================

/// Update all lines in a window for folding.
///
/// # Safety
/// The `win` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldUpdateAll(win: WinHandle) {
    fold_update_all_impl(win);
}

/// Updates folds when leaving insert-mode.
///
/// # Safety
/// This function is safe to call from C.
#[no_mangle]
pub extern "C" fn rs_foldUpdateAfterInsert() {
    fold_update_after_insert_impl();
}

/// Update line numbers of folds for inserted/deleted lines.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldMarkAdjust(
    wp: WinHandle,
    line1: LineNr,
    line2: LineNr,
    amount: LineNr,
    amount_after: LineNr,
) {
    fold_mark_adjust_impl(wp, line1, line2, amount, amount_after);
}

/// Update folds using IEMS algorithm for indent/diff methods.
///
/// This function handles fold updates for 'foldmethod' values of "indent" and "diff".
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldUpdateIEMS_indent(wp: WinHandle, top: LineNr, bot: LineNr) {
    update::fold_update_iems_indent_impl(wp, top, bot);
}

/// Update folds using IEMS algorithm for all fold methods.
///
/// This function handles fold updates for all 'foldmethod' values that use the
/// IEMS algorithm: indent, diff, marker, expr, and syntax.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldUpdateIEMS(wp: WinHandle, top: LineNr, bot: LineNr) {
    update::fold_update_iems_all_impl(wp, top, bot);
}

/// Update folds for changes in the buffer of a window.
///
/// Replaces the C `foldUpdate` function. Checks disable flags,
/// marks folds as maybe-small, and invokes the IEMS algorithm.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldUpdate(wp: WinHandle, top: LineNr, bot: LineNr) {
    update::fold_update_impl(wp, top, bot);
}

// ============================================================================
// Phase 5: Navigation and Display
// ============================================================================

extern "C" {
    /// Get the VIsual_active global.
    static mut VIsual_active: bool;

    /// Get VIsual position lnum.
    fn nvim_get_VIsual_lnum() -> c_int;

    /// Set VIsual lnum.
    fn nvim_set_VIsual_lnum(lnum: c_int);

    /// Set VIsual col.
    fn nvim_set_VIsual_col(col: c_int);

    /// Set cursor lnum in window.
    fn nvim_win_set_cursor_lnum(wp: WinHandle, lnum: LineNr);

    /// Set cursor col in window.
    fn nvim_win_set_cursor_col(wp: WinHandle, col: ColNr);

    /// Get length of line.
    fn ml_get_len(lnum: LineNr) -> ColNr;

    /// Get p_sel option first char ('o' for old, 'e' for exclusive).
    fn nvim_get_p_sel_first() -> c_char;

    /// Call mb_adjust_cursor().
    fn nvim_mb_adjust_cursor();
}

/// Column number type.
type ColNr = c_int;

/// Move the cursor to the first line of a closed fold.
fn fold_adjust_cursor_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    let cursor_lnum = unsafe { nvim_win_get_cursor_lnum(wp) };
    let result = has_folding_win_impl(wp, cursor_lnum, true);
    if result.has_folding != 0 {
        unsafe { nvim_win_set_cursor_lnum(wp, result.first) };
    }
}

/// Adjust the Visual area to include any fold at the start or end completely.
fn fold_adjust_visual_impl() {
    let curwin = unsafe { nvim_get_curwin() };

    // Check VIsual_active and hasAnyFolding
    if !unsafe { VIsual_active } || !has_any_folding_impl(curwin) {
        return;
    }

    // Determine start and end positions
    let visual_lnum = LineNr::from(unsafe { nvim_get_VIsual_lnum() });
    let cursor_lnum = unsafe { nvim_win_get_cursor_lnum(curwin) };

    // Check which is start vs end (ltoreq comparison)
    let (start_lnum, end_lnum, start_is_visual) = if visual_lnum <= cursor_lnum {
        (visual_lnum, cursor_lnum, true)
    } else {
        (cursor_lnum, visual_lnum, false)
    };

    // Adjust start position
    {
        let r = has_folding_win_impl(curwin, start_lnum, true);
        if r.has_folding != 0 {
            let first_lnum = r.first;
            if start_is_visual {
                #[allow(clippy::cast_possible_truncation)]
                unsafe {
                    nvim_set_VIsual_lnum(first_lnum as c_int);
                    nvim_set_VIsual_col(0);
                }
            } else {
                unsafe {
                    nvim_win_set_cursor_lnum(curwin, first_lnum);
                    nvim_win_set_cursor_col(curwin, 0);
                }
            }
        }
    }

    // Adjust end position
    let r2 = has_folding_win_impl(curwin, end_lnum, true);
    let last_lnum = if r2.has_folding != 0 {
        r2.last
    } else {
        return;
    };

    let line_len = unsafe { ml_get_len(last_lnum) };
    let mut end_col = line_len;
    if end_col > 0 {
        let p_sel = unsafe { nvim_get_p_sel_first() };
        if p_sel == b'o' as c_char {
            end_col -= 1;
        }
    }

    if start_is_visual {
        // end is cursor
        unsafe {
            nvim_win_set_cursor_lnum(curwin, last_lnum);
            nvim_win_set_cursor_col(curwin, end_col);
        }
    } else {
        // end is VIsual
        #[allow(clippy::cast_possible_truncation)]
        unsafe {
            nvim_set_VIsual_lnum(last_lnum as c_int);
            nvim_set_VIsual_col(end_col);
        }
    }

    // Prevent cursor from moving on the trail byte
    unsafe { nvim_mb_adjust_cursor() };
}

// ============================================================================
// Phase 3: Fold Navigation Implementation
// ============================================================================

/// Result values for fold operations (matching C OK/FAIL).
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Direction constant for forward (matches C FORWARD from vim_defs.h).
const FORWARD: c_int = 1;

/// Move cursor to fold boundary.
///
/// Move cursor to the start or end of the fold at the cursor line, or the
/// next/previous fold.
///
/// # Arguments
/// * `updown` - true = "[z" or "]z" (to start/end of fold), false = "zj" or "zk"
/// * `dir` - direction: FORWARD to end/next, BACKWARD to start/prev
/// * `count` - number of times to repeat
///
/// # Returns
/// OK on success, FAIL if no fold found
#[allow(clippy::too_many_lines)]
fn fold_move_to_impl(updown: bool, dir: c_int, count: c_int) -> c_int {
    let curwin = unsafe { nvim_get_curwin() };
    let mut retval = FAIL;

    // Update folds first
    checkupdate_impl(curwin);

    // Repeat "count" times
    for _ in 0..count {
        // Find nested folds. Stop when a fold is closed. The deepest fold
        // that moves the cursor is used.
        let mut lnum_off: LineNr = 0;
        let mut gap = unsafe { nvim_win_get_folds(curwin) };
        let gap_len = unsafe { nvim_ga_len(gap) };
        if gap_len == 0 {
            break;
        }

        let mut use_level = false;
        let mut maybe_small = false;
        let cursor_lnum = unsafe { nvim_win_get_cursor_lnum(curwin) };
        let mut lnum_found = cursor_lnum;
        let mut level: c_int = 0;
        let mut last = false;

        loop {
            // Try to find a fold containing cursor_lnum - lnum_off
            let find_result = fold_find(gap, cursor_lnum - lnum_off);

            let fp = match find_result {
                Some((fp, true)) => fp,
                Some((fp, false)) => {
                    // Fold not found (we got the first fold below lnum)
                    if !updown {
                        break;
                    }
                    let gap_len = unsafe { nvim_ga_len(gap) };
                    if gap_len == 0 {
                        break;
                    }

                    // When moving up, consider a fold above the cursor;
                    // when moving down consider a fold below the cursor.
                    let fp_data = unsafe { nvim_ga_fold_at(gap, 0) };
                    let fp_idx =
                        fold_find_with_idx(gap, cursor_lnum - lnum_off).map_or(0, |(_, idx)| idx);

                    if dir == FORWARD {
                        // Moving forward - need the fold at fp_idx
                        if fp_idx >= gap_len {
                            break;
                        }
                        // Get previous fold (fp - 1 in C)
                        if fp_idx == 0 {
                            break;
                        }
                        let prev_fp = unsafe { nvim_ga_fold_at(gap, fp_idx - 1) };
                        last = true;
                        prev_fp
                    } else {
                        // Moving backward
                        if fp_data.is_null() {
                            break;
                        }
                        last = true;
                        fp
                    }
                }
                None => break,
            };

            if fp.is_null() {
                break;
            }

            if !last {
                // Check if this fold is closed.
                let is_closed = check_closed_impl(
                    curwin,
                    fp,
                    &mut use_level,
                    level,
                    &mut maybe_small,
                    lnum_off,
                );
                if is_closed {
                    last = true;
                }

                // "[z" and "]z" stop at closed fold
                if last && !updown {
                    break;
                }
            }

            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
            let fd_len = unsafe { nvim_fold_get_fd_len(fp) };

            if updown {
                // Get fold index for neighbor access
                let fp_idx =
                    fold_find_with_idx(gap, cursor_lnum - lnum_off).map_or(0, |(_, idx)| idx);
                let gap_len = unsafe { nvim_ga_len(gap) };

                if dir == FORWARD {
                    // to start of next fold if there is one
                    if fp_idx + 1 < gap_len {
                        let next_fp = unsafe { nvim_ga_fold_at(gap, fp_idx + 1) };
                        if !next_fp.is_null() {
                            let next_top = unsafe { nvim_fold_get_fd_top(next_fp) };
                            let lnum = next_top + lnum_off;
                            if lnum > cursor_lnum {
                                lnum_found = lnum;
                            }
                        }
                    }
                } else {
                    // to end of previous fold if there is one
                    if fp_idx > 0 {
                        let prev_fp = unsafe { nvim_ga_fold_at(gap, fp_idx - 1) };
                        if !prev_fp.is_null() {
                            let prev_top = unsafe { nvim_fold_get_fd_top(prev_fp) };
                            let prev_len = unsafe { nvim_fold_get_fd_len(prev_fp) };
                            let lnum = prev_top + lnum_off + prev_len - 1;
                            if lnum < cursor_lnum {
                                lnum_found = lnum;
                            }
                        }
                    }
                }
            } else {
                // Open fold found, set cursor to its start/end and then check
                // nested folds.
                if dir == FORWARD {
                    let lnum = fd_top + lnum_off + fd_len - 1;
                    if lnum > cursor_lnum {
                        lnum_found = lnum;
                    }
                } else {
                    let lnum = fd_top + lnum_off;
                    if lnum < cursor_lnum {
                        lnum_found = lnum;
                    }
                }
            }

            if last {
                break;
            }

            // Check nested folds (if any).
            gap = unsafe { nvim_fold_get_fd_nested(fp) };
            lnum_off += fd_top;
            level += 1;
        }

        if lnum_found == cursor_lnum {
            break;
        }
        if retval == FAIL {
            unsafe { nvim_setpcmark() };
        }
        unsafe {
            nvim_win_set_cursor_lnum(curwin, lnum_found);
            nvim_win_set_cursor_col(curwin, 0);
        }
        retval = OK;
    }

    retval
}

// ============================================================================
// Phase 5: FFI Exports
// ============================================================================

/// Move the cursor to the first line of a closed fold.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldAdjustCursor(wp: WinHandle) {
    fold_adjust_cursor_impl(wp);
}

/// Adjust the Visual area to include any fold at the start or end completely.
///
/// # Safety
/// This function is safe to call from C.
#[no_mangle]
pub extern "C" fn rs_foldAdjustVisual() {
    fold_adjust_visual_impl();
}

/// Move cursor to fold boundary.
///
/// Move cursor to the start or end of the fold at the cursor line, or the
/// next/previous fold.
///
/// # Arguments
/// * `updown` - true = "[z" or "]z" (to start/end of fold), false = "zj" or "zk"
/// * `dir` - direction: FORWARD to end/next, BACKWARD to start/prev
/// * `count` - number of times to repeat
///
/// # Safety
/// This function is safe to call from C.
#[no_mangle]
pub extern "C" fn rs_foldMoveTo(updown: bool, dir: c_int, count: c_int) -> c_int {
    fold_move_to_impl(updown, dir, count)
}

// ============================================================================
// Phase 1: Manual Fold Operations
// ============================================================================

extern "C" {
    /// Check if foldmethod is marker for the window.
    #[allow(dead_code)]
    fn nvim_foldmethodIsMarker(wp: WinHandle) -> c_int;

    /// Call check_cursor_col for window.
    fn nvim_check_cursor_col(wp: WinHandle);

    /// Call changed_lines for buffer.
    fn nvim_changed_lines(
        buf: BufHandle,
        first: LineNr,
        col: c_int,
        last: LineNr,
        xtra: LineNr,
        add_undo: bool,
    );

    /// Send buffer update events.
    fn nvim_buf_updates_send_changes(
        buf: BufHandle,
        firstlnum: LineNr,
        num_added: i64,
        num_removed: i64,
    );

    /// Redraw the current buffer later.
    fn redraw_curbuf_later(redraw_type: c_int);
}

/// UPD_INVERTED redraw type (from drawscreen.h).
const UPD_INVERTED: c_int = 20;

/// Create a fold from line `start_lnum` to line `end_lnum` (inclusive) in the window.
///
/// For marker method, this adds fold markers to the buffer.
/// For manual method, this creates a fold entry in the fold tree.
#[allow(clippy::too_many_lines)]
fn fold_create_impl(wp: WinHandle, mut start_lnum: LineNr, mut end_lnum: LineNr) {
    if wp.is_null() {
        return;
    }

    // Swap if start > end
    if start_lnum > end_lnum {
        std::mem::swap(&mut start_lnum, &mut end_lnum);
    }

    // When 'foldmethod' is "marker" add markers, which creates the folds.
    if foldmethod_is_marker_impl(wp) {
        markers::fold_create_markers_impl(wp, start_lnum, end_lnum);
        return;
    }

    // checkupdate(wp)
    checkupdate_impl(wp);

    let mut use_level = false;
    let mut closed = false;
    let mut level = 0;
    let mut start_rel = start_lnum;
    let mut end_rel = end_lnum;

    // Find the place to insert the new fold
    let mut gap = unsafe { nvim_win_get_folds(wp) };
    let mut i: c_int;

    let len = unsafe { nvim_ga_len(gap) };
    if len == 0 {
        i = 0;
    } else {
        // Find fold containing start_rel
        loop {
            let Some((found, fp_idx)) = fold_find_with_idx(gap, start_rel) else {
                i = 0;
                break;
            };

            if !found {
                i = fp_idx;
                break;
            }

            let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
            if fp.is_null() {
                i = fp_idx;
                break;
            }

            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
            let fd_len = unsafe { nvim_fold_get_fd_len(fp) };

            if fd_top + fd_len > end_rel {
                // New fold is completely inside this fold: Go one level deeper.
                let nested = unsafe { nvim_fold_get_fd_nested(fp) };
                gap = nested;
                start_rel -= fd_top;
                end_rel -= fd_top;

                let fdl = unsafe { nvim_win_get_p_fdl(wp) };
                let fd_flags = unsafe { nvim_fold_get_fd_flags(fp) };

                if use_level || fd_flags == fold_flags::FD_LEVEL {
                    use_level = true;
                    if level >= fdl {
                        closed = true;
                    }
                } else if fd_flags == fold_flags::FD_CLOSED {
                    closed = true;
                }
                level += 1;
            } else {
                // This fold and new fold overlap: Insert here and move some folds inside the new fold.
                i = fp_idx;
                break;
            }
        }

        // Handle case where we descended into nested folds and found empty array
        let gap_len = unsafe { nvim_ga_len(gap) };
        if gap_len == 0 {
            i = 0;
        }
    }

    // Grow the array to make room for new fold
    unsafe { nvim_ga_grow_folds(gap, 1) };

    // Count number of folds that will be contained in the new fold.
    let mut cont: c_int = 0;
    let gap_len = unsafe { nvim_ga_len(gap) };
    for j in i..gap_len {
        let fp = unsafe { nvim_ga_fold_at(gap, j) };
        if fp.is_null() {
            break;
        }
        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
        if fd_top > end_rel {
            break;
        }
        cont += 1;
    }

    if cont > 0 {
        // Adjust start_rel to be the minimum of it and the first contained fold's top
        let fp_first = unsafe { nvim_ga_fold_at(gap, i) };
        if !fp_first.is_null() {
            let first_top = unsafe { nvim_fold_get_fd_top(fp_first) };
            start_rel = start_rel.min(first_top);
        }

        // When last contained fold isn't completely contained, adjust end of new fold.
        let fp_last = unsafe { nvim_ga_fold_at(gap, i + cont - 1) };
        if !fp_last.is_null() {
            let last_top = unsafe { nvim_fold_get_fd_top(fp_last) };
            let last_len = unsafe { nvim_fold_get_fd_len(fp_last) };
            end_rel = end_rel.max(last_top + last_len - 1);
        }

        // Move folds after the contained ones down by (1 - cont) positions
        // i.e., we're removing `cont` folds and adding 1 fold
        if i + cont < gap_len {
            unsafe {
                nvim_fold_memmove(gap, i + 1, i + cont, gap_len - (i + cont));
            }
        }

        // Set new length: gap_len + 1 - cont
        let new_len = gap_len + 1 - cont;
        unsafe { nvim_ga_set_len(gap, new_len) };
    } else {
        // No contained folds - simpler case
        // Shift existing folds to make room
        if i < gap_len {
            unsafe {
                nvim_fold_memmove(gap, i + 1, i, gap_len - i);
            }
        }

        // Increment length
        unsafe { nvim_ga_set_len(gap, gap_len + 1) };
    }

    // Get pointer to where new fold will be inserted (after potential shift)
    let fp = unsafe { nvim_ga_fold_at(gap, i) };
    if fp.is_null() {
        return;
    }

    // Initialize nested array
    let nested = unsafe { nvim_fold_get_fd_nested(fp) };
    if !nested.is_null() {
        unsafe { nvim_ga_init_folds(nested) };
    }

    // Set up the new fold
    unsafe {
        nvim_fold_set_fd_top(fp, start_rel);
        nvim_fold_set_fd_len(fp, end_rel - start_rel + 1);
        nvim_fold_set_fd_flags(fp, fold_flags::FD_CLOSED);
        nvim_fold_set_fd_small(fp, tristate::K_NONE);
    }

    // We want the new fold to be closed. If it would remain open because
    // of using 'foldlevel', need to adjust fd_flags of containing folds.
    let fdl = unsafe { nvim_win_get_p_fdl(wp) };
    if use_level && !closed && level < fdl {
        set_fold_repeat_impl(start_lnum, 1, false);
    }
    if !use_level {
        unsafe { nvim_win_set_w_fold_manual(wp, true) };
    }

    // Redraw
    unsafe { changed_window_setting(wp) };
}

/// Delete folds from line `start` to `end` (inclusive).
///
/// When `recursive` is true, delete recursively.
/// When `had_visual` is true, a visual selection was used.
#[allow(clippy::too_many_lines)]
fn delete_fold_impl(wp: WinHandle, start: LineNr, end: LineNr, recursive: bool, had_visual: bool) {
    if wp.is_null() {
        return;
    }

    // checkupdate(wp)
    checkupdate_impl(wp);

    let mut lnum = start;
    let mut did_one = false;
    let mut first_lnum = MAXLNUM;
    let mut last_lnum: LineNr = 0;

    while lnum <= end {
        // Find the deepest fold for "lnum".
        let mut gap = unsafe { nvim_win_get_folds(wp) };
        let mut found_ga: Option<GArrayHandle> = None;
        let mut found_fp: FoldHandle = FoldHandle(std::ptr::null_mut());
        let mut found_idx: c_int = 0;
        let mut found_off: LineNr = 0;
        let mut lnum_off: LineNr = 0;
        let mut use_level = false;
        let mut level = 0;

        loop {
            let Some((found, fp_idx)) = fold_find_with_idx(gap, lnum - lnum_off) else {
                break;
            };

            if !found {
                break;
            }

            let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
            if fp.is_null() {
                break;
            }

            // lnum is inside this fold, remember info
            found_ga = Some(gap);
            found_fp = fp;
            found_idx = fp_idx;
            found_off = lnum_off;

            // if "lnum" is folded, don't check nesting
            let mut maybe_small = false;
            let is_closed =
                check_closed_impl(wp, fp, &mut use_level, level, &mut maybe_small, lnum_off);
            if is_closed {
                break;
            }

            // check nested folds
            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
            gap = unsafe { nvim_fold_get_fd_nested(fp) };
            lnum_off += fd_top;
            level += 1;
        }

        if let Some(found_gap) = found_ga {
            let fd_top = unsafe { nvim_fold_get_fd_top(found_fp) };
            let fd_len = unsafe { nvim_fold_get_fd_len(found_fp) };
            lnum = fd_top + fd_len + found_off;

            if foldmethod_is_manual_impl(wp) {
                delete_fold_entry_impl(wp, found_gap, found_idx, recursive);
            } else {
                first_lnum = first_lnum.min(fd_top + found_off);
                last_lnum = last_lnum.max(lnum);
                markers::delete_fold_markers_impl(wp, found_fp, recursive, found_off);
            }
            did_one = true;

            // redraw window
            unsafe { changed_window_setting(wp) };
        } else {
            lnum += 1;
        }
    }

    if did_one {
        // Deleting markers may make cursor column invalid
        unsafe { nvim_check_cursor_col(wp) };
    } else {
        unsafe { nvim_emsg_nofold() };
        // Force a redraw to remove the Visual highlighting.
        if had_visual {
            let buf = unsafe { nvim_win_get_buffer(wp) };
            unsafe { redraw_buf_later(buf, UPD_INVERTED) };
        }
    }

    if last_lnum > 0 {
        let buf = unsafe { nvim_win_get_buffer(wp) };
        unsafe { nvim_changed_lines(buf, first_lnum, 0, last_lnum, 0, false) };

        // send one nvim_buf_lines_event at the end
        let num_changed = i64::from(last_lnum - first_lnum);
        unsafe { nvim_buf_updates_send_changes(buf, first_lnum, num_changed, num_changed) };
    }
}

extern "C" {
    /// Redraw buffer later.
    fn redraw_buf_later(buf: BufHandle, redraw_type: c_int);
}

/// Open or close folds for current window in lines "first" to "last".
///
/// Used for "zo", "zO", "zc" and "zC" in Visual mode.
fn op_fold_range_impl(
    first_lnum: LineNr,
    last_lnum: LineNr,
    opening: bool,
    recurse: bool,
    had_visual: bool,
) {
    let curwin = unsafe { nvim_get_curwin() };
    let mut done = done_flags::DONE_NOTHING;

    let mut lnum = first_lnum;
    while lnum <= last_lnum {
        let mut lnum_next = lnum;

        // Opening one level only: next fold to open is after the one going to be opened.
        if opening && !recurse {
            let r = has_folding_win_impl(curwin, lnum, true);
            if r.has_folding != 0 {
                lnum_next = r.last;
            }
        }

        set_manual_fold_impl(lnum, opening, recurse, &raw mut done);

        // Closing one level only: next line to close a fold is after just closed fold.
        if !opening && !recurse {
            let r = has_folding_win_impl(curwin, lnum, true);
            if r.has_folding != 0 {
                lnum_next = r.last;
            }
        }

        lnum = lnum_next + 1;
    }

    if done == done_flags::DONE_NOTHING {
        unsafe { nvim_emsg_nofold() };
    }

    // Force a redraw to remove the Visual highlighting.
    if had_visual {
        unsafe { redraw_curbuf_later(UPD_INVERTED) };
    }
}

/// Open folds until the cursor line is not in a closed fold.
fn fold_open_cursor_impl() {
    let curwin = unsafe { nvim_get_curwin() };

    // checkupdate(curwin)
    checkupdate_impl(curwin);

    if !has_any_folding_impl(curwin) {
        return;
    }

    loop {
        let mut done: c_int = done_flags::DONE_NOTHING;
        let cursor_lnum = unsafe { nvim_win_get_cursor_lnum(curwin) };
        set_manual_fold_win_impl(curwin, cursor_lnum, true, false, &raw mut done);
        if done & done_flags::DONE_ACTION == 0 {
            break;
        }
    }
}

// ============================================================================
// Phase 1: Manual Fold Operations FFI Exports
// ============================================================================

/// Create a fold from line `start_lnum` to line `end_lnum` (inclusive).
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_foldCreate(wp: WinHandle, start_lnum: LineNr, end_lnum: LineNr) {
    fold_create_impl(wp, start_lnum, end_lnum);
}

/// Delete folds from line `start` to `end` (inclusive).
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_deleteFold(
    wp: WinHandle,
    start: LineNr,
    end: LineNr,
    recursive: c_int,
    had_visual: bool,
) {
    delete_fold_impl(wp, start, end, recursive != 0, had_visual);
}

/// Open or close folds for current window in lines "first" to "last".
///
/// # Safety
/// This function is safe to call from C.
#[no_mangle]
pub extern "C" fn rs_opFoldRange(
    first_lnum: LineNr,
    last_lnum: LineNr,
    opening: c_int,
    recurse: c_int,
    had_visual: bool,
) {
    op_fold_range_impl(
        first_lnum,
        last_lnum,
        opening != 0,
        recurse != 0,
        had_visual,
    );
}

/// Open folds until the cursor line is not in a closed fold.
///
/// # Safety
/// This function is safe to call from C.
#[no_mangle]
pub extern "C" fn rs_foldOpenCursor() {
    fold_open_cursor_impl();
}

// ============================================================================
// Phase 3: Fold Level Query Chain
// ============================================================================

/// Check if the folds in window "wp" are invalid and update them if needed.
pub(crate) fn checkupdate_impl(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    let foldinvalid = unsafe { nvim_win_get_w_foldinvalid(wp) };
    if !foldinvalid {
        return;
    }
    update::fold_update_impl(wp, 1, MAXLNUM);
    unsafe { nvim_win_set_w_foldinvalid(wp, false) };
}

/// Get fold level at line number "lnum" in the current window.
///
/// Uses cached values from the IEMS update algorithm. While updating,
/// lines between invalid_top and invalid_bot have undefined fold level.
fn fold_level_impl(lnum: LineNr) -> c_int {
    let curwin = unsafe { nvim_get_curwin() };

    let inv_top = invalid_top();
    if inv_top == 0 {
        checkupdate_impl(curwin);
    } else {
        let p_lnum = prev_lnum();
        let p_lnum_lvl = prev_lnum_lvl();
        if lnum == p_lnum && p_lnum_lvl >= 0 {
            return p_lnum_lvl;
        }
        let inv_bot = invalid_bot();
        if lnum >= inv_top && lnum <= inv_bot {
            return -1;
        }
    }

    // Return quickly when there is no folding at all in this window.
    if !has_any_folding_impl(curwin) {
        return 0;
    }

    fold_level_win_impl(curwin, lnum)
}

/// Count the number of lines that are folded at line number "lnum".
///
/// Returns fold info including fi_lines (number of folded lines, 0 if not folded).
fn fold_info_impl(win: WinHandle, lnum: LineNr) -> FoldInfoResult {
    let result = has_folding_win_impl(win, lnum, false);
    if result.has_folding != 0 {
        FoldInfoResult {
            fi_lnum: result.fi_lnum,
            fi_level: result.fi_level,
            fi_low_level: result.fi_low_level,
            fi_lines: result.last - lnum + 1,
        }
    } else {
        FoldInfoResult {
            fi_lnum: result.fi_lnum,
            fi_level: result.fi_level,
            fi_low_level: result.fi_low_level,
            fi_lines: 0,
        }
    }
}

/// Apply 'foldclose' to all folds that don't contain the cursor.
fn fold_check_close_impl() {
    let p_fcl = unsafe { nvim_get_p_fcl() };
    // p_fcl is NUL if empty
    if p_fcl.is_null() || unsafe { *p_fcl } == 0 {
        return;
    }

    let curwin = unsafe { nvim_get_curwin() };
    // 'foldclose' can only be "all" right now
    checkupdate_impl(curwin);

    let gap = unsafe { nvim_win_get_folds(curwin) };
    let cursor_lnum = unsafe { nvim_win_get_cursor_lnum(curwin) };
    let fdl = unsafe { nvim_win_get_p_fdl(curwin) };

    if check_close_rec_impl(gap, cursor_lnum, fdl) {
        unsafe { changed_window_setting(curwin) };
    }
}

/// FFI export for foldLevel.
#[no_mangle]
pub extern "C" fn rs_foldLevel(lnum: LineNr) -> c_int {
    fold_level_impl(lnum)
}

/// FFI export for checkupdate.
#[no_mangle]
pub extern "C" fn rs_checkupdate(wp: WinHandle) {
    checkupdate_impl(wp);
}

/// FFI export for fold_info.
///
/// Returns a FoldInfoResult matching C foldinfo_T layout.
#[no_mangle]
pub extern "C" fn rs_fold_info(win: WinHandle, lnum: LineNr) -> FoldInfoResult {
    fold_info_impl(win, lnum)
}

/// FFI export for foldCheckClose.
#[no_mangle]
pub extern "C" fn rs_foldCheckClose() {
    fold_check_close_impl();
}

// ============================================================================
// Fold Move Range
// ============================================================================

/// Helper: get the end line of a fold (fd_top + fd_len - 1).
#[inline]
fn fold_end(gap: GArrayHandle, idx: c_int) -> LineNr {
    let fp = unsafe { nvim_ga_fold_at(gap, idx) };
    let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
    let fd_len = unsafe { nvim_fold_get_fd_len(fp) };
    fd_top + fd_len - 1
}

/// Helper: check if index is a valid fold in the garray.
#[inline]
fn valid_fold(gap: GArrayHandle, idx: c_int) -> bool {
    let len = unsafe { nvim_ga_len(gap) };
    len > 0 && idx < len
}

/// Truncate a fold to end at `end` (inclusive).
///
/// Removes nested folds past `end` and sets fd_len accordingly.
fn truncate_fold_impl(wp: WinHandle, gap: GArrayHandle, idx: c_int, end: LineNr) {
    let fp = unsafe { nvim_ga_fold_at(gap, idx) };
    if fp.is_null() {
        return;
    }
    let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
    let nested = unsafe { nvim_fold_get_fd_nested(fp) };
    // foldRemove stops *above* top, so we add 1 to stop *at* end
    let remove_top = end + 1 - fd_top;
    fold_remove_impl(wp, nested, remove_top, MAXLNUM);
    unsafe { nvim_fold_set_fd_len(fp, end + 1 - fd_top) };
}

/// Move folds within a garray when buffer lines are moved (`:move` command).
///
/// Handles 10 distinct cases for folds relative to the moved range
/// [line1, line2] and destination `dest` (where dest > line2).
///
/// Assumes dest > line2 (downward move).
#[allow(clippy::too_many_lines)]
fn fold_move_range_impl(
    wp: WinHandle,
    gap: GArrayHandle,
    line1: LineNr,
    line2: LineNr,
    dest: LineNr,
) {
    let range_len = line2 - line1 + 1;
    let move_len = dest - line2;

    // Find fold at or containing line1 - 1
    let Some((at_start, mut fp_idx)) = fold_find_with_idx(gap, line1 - 1) else {
        return;
    };

    if at_start {
        if fold_end(gap, fp_idx) > dest {
            // Case 4 -- don't have to change this fold, but have to move nested
            // folds.
            let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
            let nested = unsafe { nvim_fold_get_fd_nested(fp) };
            fold_move_range_impl(wp, nested, line1 - fd_top, line2 - fd_top, dest - fd_top);
            return;
        } else if fold_end(gap, fp_idx) > line2 {
            // Case 3 -- Remove nested folds between line1 and line2 & reduce the
            // length of fold by "range_len".
            // Folds after this one must be dealt with.
            let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
            let fd_len = unsafe { nvim_fold_get_fd_len(fp) };
            let nested = unsafe { nvim_fold_get_fd_nested(fp) };
            fold_mark_adjust_recurse_impl(
                wp,
                nested,
                line1 - fd_top,
                line2 - fd_top,
                MAXLNUM,
                -range_len,
            );
            unsafe { nvim_fold_set_fd_len(fp, fd_len - range_len) };
        } else {
            // Case 2 -- truncate fold *above* line1.
            // Folds after this one must be dealt with.
            truncate_fold_impl(wp, gap, fp_idx, line1 - 1);
        }
        // Look at the next fold, and treat that one as if it were the first after
        // "line1" (because now it is).
        fp_idx += 1;
    }

    if !valid_fold(gap, fp_idx) {
        return;
    }
    let fd_top = unsafe { nvim_fold_get_fd_top(nvim_ga_fold_at(gap, fp_idx)) };
    if fd_top > dest {
        // No folds after "line1" and before "dest" — Case 10.
        return;
    }

    if fd_top > line2 {
        // Cases 8 and 9: folds between line2 and dest.
        while valid_fold(gap, fp_idx) && fold_end(gap, fp_idx) <= dest {
            // Case 9 -- shift up.
            let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
            let top = unsafe { nvim_fold_get_fd_top(fp) };
            unsafe { nvim_fold_set_fd_top(fp, top - range_len) };
            fp_idx += 1;
        }
        if valid_fold(gap, fp_idx) {
            let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
            let top = unsafe { nvim_fold_get_fd_top(fp) };
            if top <= dest {
                // Case 8 -- ensure truncated at dest, shift up
                truncate_fold_impl(wp, gap, fp_idx, dest);
                // Re-fetch after possible realloc
                let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
                let top = unsafe { nvim_fold_get_fd_top(fp) };
                unsafe { nvim_fold_set_fd_top(fp, top - range_len) };
            }
        }
        return;
    }

    if fold_end(gap, fp_idx) > dest {
        // Case 7 -- remove nested folds and shrink
        let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
        let fd_top_val = unsafe { nvim_fold_get_fd_top(fp) };
        let fd_len_val = unsafe { nvim_fold_get_fd_len(fp) };
        let nested = unsafe { nvim_fold_get_fd_nested(fp) };
        fold_mark_adjust_recurse_impl(
            wp,
            nested,
            line2 + 1 - fd_top_val,
            dest - fd_top_val,
            MAXLNUM,
            -move_len,
        );
        unsafe {
            nvim_fold_set_fd_len(fp, fd_len_val - move_len);
            nvim_fold_set_fd_top(fp, fd_top_val + move_len);
        }
        return;
    }

    // Cases 5 and 6: changes rely on whether there are folds between the end of
    // this fold and "dest".
    let move_start = fp_idx;
    let mut move_end: c_int = 0;
    while valid_fold(gap, fp_idx) {
        let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
        let top = unsafe { nvim_fold_get_fd_top(fp) };
        if top > dest {
            break;
        }

        if top <= line2 {
            // 5, or 6
            if fold_end(gap, fp_idx) > line2 {
                // 6, truncate before moving
                truncate_fold_impl(wp, gap, fp_idx, line2);
            }
            // Re-fetch fp after possible realloc in truncate_fold_impl
            let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
            let top = unsafe { nvim_fold_get_fd_top(fp) };
            unsafe { nvim_fold_set_fd_top(fp, top + move_len) };
            fp_idx += 1;
            continue;
        }

        // Record index of the first fold after the moved range.
        if move_end == 0 {
            move_end = fp_idx;
        }

        if fold_end(gap, fp_idx) > dest {
            truncate_fold_impl(wp, gap, fp_idx, dest);
        }

        // Re-fetch fp after possible realloc
        let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
        let top = unsafe { nvim_fold_get_fd_top(fp) };
        unsafe { nvim_fold_set_fd_top(fp, top - range_len) };
        fp_idx += 1;
    }
    let dest_index = fp_idx;

    // All folds are now correct, but not necessarily in the correct order.
    // We must swap folds in the range [move_end, dest_index) with those in the
    // range [move_start, move_end).
    if move_end == 0 {
        // There are no folds after those moved, so none were moved out of order.
        return;
    }
    fold_reverse_order_impl(gap, move_start as LineNr, (dest_index - 1) as LineNr);
    fold_reverse_order_impl(
        gap,
        move_start as LineNr,
        (move_start + dest_index - move_end - 1) as LineNr,
    );
    fold_reverse_order_impl(
        gap,
        (move_start + dest_index - move_end) as LineNr,
        (dest_index - 1) as LineNr,
    );
}

/// Move folds when buffer lines are moved (`:move` command).
///
/// # Safety
/// All pointer parameters must be valid or null.
#[no_mangle]
pub extern "C" fn rs_foldMoveRange(
    wp: WinHandle,
    gap: GArrayHandle,
    line1: LineNr,
    line2: LineNr,
    dest: LineNr,
) {
    fold_move_range_impl(wp, gap, line1, line2, dest);
}

// ============================================================================
// Phase 5: VimL Function Wrappers
// ============================================================================

extern "C" {
    /// Get line number from argvars[0] (tv_get_lnum).
    fn nvim_fold_tv_get_lnum(argvars: *const c_void) -> LineNr;

    /// Set rettv->vval.v_number.
    fn nvim_fold_rettv_set_number(rettv: *mut c_void, nr: i64);

    /// Set rettv->v_type = VAR_STRING and rettv->vval.v_string = s.
    fn nvim_fold_rettv_init_string(rettv: *mut c_void, s: *mut c_char);

    /// Get the line count of curbuf.
    fn nvim_fold_get_curbuf_line_count() -> LineNr;

    /// Get the Rust-implemented foldtext string.
    fn rs_f_foldtext_impl() -> *mut c_char;
}

/// Implement `foldclosed()` or `foldclosedend()` VimL functions.
///
/// # Safety
/// `argvars` and `rettv` must be valid pointers to `typval_T`.
unsafe fn f_foldclosed_impl(argvars: *const c_void, rettv: *mut c_void, end: bool) {
    let lnum = nvim_fold_tv_get_lnum(argvars);
    let curbuf_lc = nvim_fold_get_curbuf_line_count();
    let mut result: LineNr = -1;
    if lnum >= 1 && lnum <= curbuf_lc {
        let curwin = nvim_get_curwin();
        let fr = rs_hasFoldingWin(curwin, lnum, false);
        if fr.has_folding != 0 {
            result = if end { fr.last } else { fr.first };
        }
    }
    nvim_fold_rettv_set_number(rettv, i64::from(result));
}

/// FFI: `foldclosed()` VimL function - replace C `f_foldclosed`.
///
/// # Safety
/// `argvars` and `rettv` must be valid `typval_T*` pointers.
/// `fptr` is ignored.
#[no_mangle]
pub unsafe extern "C" fn rs_f_foldclosed(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *const c_void,
) {
    f_foldclosed_impl(argvars, rettv, false);
}

/// FFI: `foldclosedend()` VimL function - replace C `f_foldclosedend`.
///
/// # Safety
/// `argvars` and `rettv` must be valid `typval_T*` pointers.
/// `fptr` is ignored.
#[no_mangle]
pub unsafe extern "C" fn rs_f_foldclosedend(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *const c_void,
) {
    f_foldclosed_impl(argvars, rettv, true);
}

/// FFI: `foldlevel()` VimL function - replace C `f_foldlevel`.
///
/// # Safety
/// `argvars` and `rettv` must be valid `typval_T*` pointers.
/// `fptr` is ignored.
#[no_mangle]
pub unsafe extern "C" fn rs_f_foldlevel(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *const c_void,
) {
    let lnum = nvim_fold_tv_get_lnum(argvars);
    let curbuf_lc = nvim_fold_get_curbuf_line_count();
    if lnum >= 1 && lnum <= curbuf_lc {
        let level = fold_level_impl(lnum);
        nvim_fold_rettv_set_number(rettv, i64::from(level));
    }
}

/// FFI: `foldtext()` VimL function - replace C `f_foldtext`.
///
/// # Safety
/// `rettv` must be a valid `typval_T*` pointer.
/// `fptr` is ignored.
#[no_mangle]
pub unsafe extern "C" fn rs_f_foldtext(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *const c_void,
) {
    let s = rs_f_foldtext_impl();
    nvim_fold_rettv_init_string(rettv, s);
}

/// FFI: `foldtextresult()` VimL function - replace C `f_foldtextresult`.
///
/// # Safety
/// `argvars` and `rettv` must be valid `typval_T*` pointers.
/// `fptr` is ignored.
#[no_mangle]
pub unsafe extern "C" fn rs_f_foldtextresult(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *const c_void,
) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static ENTERED: AtomicBool = AtomicBool::new(false);

    // Initialize rettv to empty string (VAR_STRING, NULL).
    nvim_fold_rettv_init_string(rettv, std::ptr::null_mut());

    // Reentrancy guard: reject recursive calls.
    if ENTERED
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {
        return;
    }

    let mut lnum = nvim_fold_tv_get_lnum(argvars);
    // Treat illegal types and illegal string values for {lnum} the same.
    if lnum < 0 {
        lnum = 0;
    }

    let info = fold_info_impl(nvim_get_curwin(), lnum);
    if info.fi_lines > 0 {
        let curwin = nvim_get_curwin();
        let lnume = lnum + info.fi_lines - 1;
        let text = crate::display::get_foldtext_concat_impl(curwin, lnum, lnume, info.fi_level);
        nvim_fold_rettv_init_string(rettv, text);
    }

    ENTERED.store(false, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    // Tests require FFI stubs which aren't available in pure Rust testing.
    // Integration testing is done via the full Neovim build.
}
