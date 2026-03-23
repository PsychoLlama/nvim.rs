//! Fold update algorithm
//!
//! This module provides infrastructure for fold update operations.
//! The main algorithm is `foldUpdateIEMS` which handles:
//! - Indent-based folding
//! - Expression-based folding
//! - Marker-based folding
//! - Syntax-based folding

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::similar_names)]

use std::ffi::c_int;

use crate::markers::{foldlevel_marker_impl, parse_marker_impl, FoldMarkerInfo};
use crate::GArrayHandle;
use nvim_window::WinHandle;

/// Line number type
type LinenrT = i32;

/// Maximum fold nesting level
pub const MAX_LEVEL: c_int = 20;

// =============================================================================
// Fold Update State
// =============================================================================

/// State for fold line processing
#[repr(C)]
#[derive(Debug, Clone)]
pub struct FoldLineState {
    /// Window being processed
    pub wp: WinHandle,
    /// Current line number
    pub lnum: LinenrT,
    /// Offset added to lnum for relative line numbers
    pub off: LinenrT,
    /// Current fold level
    pub lvl: c_int,
    /// Fold level for next line
    pub lvl_next: c_int,
    /// Fold start indicator
    pub start: c_int,
    /// Fold end level
    pub end: c_int,
    /// Previous end level
    pub had_end: c_int,
}

impl Default for FoldLineState {
    fn default() -> Self {
        Self {
            wp: WinHandle::null(),
            lnum: 0,
            off: 0,
            lvl: 0,
            lvl_next: -1,
            start: 0,
            end: MAX_LEVEL + 1,
            had_end: MAX_LEVEL + 1,
        }
    }
}

impl FoldLineState {
    /// Create new state for a window
    pub fn new(wp: WinHandle) -> Self {
        Self {
            wp,
            ..Default::default()
        }
    }

    /// Initialize for fold update at a top line
    pub fn init_for_update(&mut self, top: LinenrT) {
        self.off = 0;
        self.lvl = 0;
        self.lvl_next = -1;
        self.start = 0;
        self.end = MAX_LEVEL + 1;
        self.had_end = MAX_LEVEL + 1;
        self.lnum = top;
    }

    /// Check if fold should start at current position
    pub const fn should_start_fold(&self) -> bool {
        self.start > 0 || self.lvl_next > self.lvl
    }

    /// Check if fold should end at current position
    pub const fn should_end_fold(&self, level: c_int) -> bool {
        self.lvl_next < level
    }
}

// =============================================================================
// Fold Update Context
// =============================================================================

/// Context for fold update algorithm
#[repr(C)]
#[derive(Debug, Clone)]
pub struct FoldUpdateContext {
    /// Top line of update range
    pub top: LinenrT,
    /// Bottom line of update range
    pub bot: LinenrT,
    /// Current fold level being processed
    pub level: c_int,
    /// Start line of current section
    pub start_lnum: LinenrT,
    /// Whether any folds changed
    pub fold_changed: bool,
}

impl FoldUpdateContext {
    /// Create new update context for a range
    pub const fn new(top: LinenrT, bot: LinenrT) -> Self {
        Self {
            top,
            bot,
            level: 1,
            start_lnum: 0,
            fold_changed: false,
        }
    }

    /// Check if line is within update range
    pub const fn is_in_range(&self, lnum: LinenrT) -> bool {
        lnum >= self.top && lnum <= self.bot
    }

    /// Extend bottom of update range if needed
    pub fn extend_bot(&mut self, new_bot: LinenrT) {
        if new_bot > self.bot {
            self.bot = new_bot;
        }
    }
}

// =============================================================================
// Fold Update Request Types
// =============================================================================

/// Types of fold update requests
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldUpdateType {
    /// Update all folds in the buffer
    All,
    /// Update folds in a specific range
    Range(LinenrT, LinenrT),
    /// Lines inserted - adjust folds
    Insert(LinenrT, LinenrT),
    /// Lines deleted - adjust folds
    Delete(LinenrT, LinenrT),
    /// Lines changed - recalculate folds
    Changed(LinenrT, LinenrT),
}

impl FoldUpdateType {
    /// Get the line range affected by this update
    pub const fn range(&self) -> (LinenrT, LinenrT) {
        match *self {
            Self::All => (1, LinenrT::MAX),
            Self::Range(top, bot)
            | Self::Insert(top, bot)
            | Self::Delete(top, bot)
            | Self::Changed(top, bot) => (top, bot),
        }
    }
}

// =============================================================================
// Level Getter Types
// =============================================================================

/// Type of level getter function
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelGetterType {
    /// No level getter (shouldn't happen)
    None,
    /// Indent-based folding
    Indent,
    /// Expression-based folding
    Expr,
    /// Marker-based folding
    Marker,
    /// Syntax-based folding
    Syntax,
    /// Diff-based folding
    Diff,
}

impl LevelGetterType {
    /// Determine level getter type from window fold method
    #[allow(clippy::items_after_statements)]
    pub fn from_window(wp: WinHandle) -> Self {
        if wp.is_null() {
            return Self::None;
        }

        use crate::{
            foldmethod_is_diff_impl, foldmethod_is_expr_impl, foldmethod_is_indent_impl,
            foldmethod_is_marker_impl, foldmethod_is_syntax_impl,
        };

        if foldmethod_is_marker_impl(wp) {
            Self::Marker
        } else if foldmethod_is_expr_impl(wp) {
            Self::Expr
        } else if foldmethod_is_syntax_impl(wp) {
            Self::Syntax
        } else if foldmethod_is_diff_impl(wp) {
            Self::Diff
        } else if foldmethod_is_indent_impl(wp) {
            Self::Indent
        } else {
            Self::None
        }
    }

    /// Check if this method needs the line above for context
    pub const fn needs_line_above(&self) -> bool {
        matches!(self, Self::Expr | Self::Indent)
    }
}

// =============================================================================
// Fold Recursion State
// =============================================================================

/// State for recursive fold update
#[repr(C)]
#[derive(Debug)]
pub struct FoldRecursionState {
    /// Current gap being processed
    pub gap: GArrayHandle,
    /// Current recursion level
    pub level: c_int,
    /// Start line for this recursion level
    pub start_lnum: LinenrT,
    /// End line to process to
    pub end_lnum: LinenrT,
    /// Whether we're at the first fold in this gap
    pub first_in_gap: bool,
}

impl FoldRecursionState {
    /// Create new recursion state
    pub fn new(gap: GArrayHandle, level: c_int, start: LinenrT, end: LinenrT) -> Self {
        Self {
            gap,
            level,
            start_lnum: start,
            end_lnum: end,
            first_in_gap: true,
        }
    }
}

// =============================================================================
// IEMS Algorithm - Indent fold method support
// =============================================================================

use nvim_buffer::BufHandle;

use crate::level::{
    foldlevel_diff_result, foldlevel_expr_result, foldlevel_indent_result, foldlevel_syntax_result,
};
use crate::{fold_flags, tristate, FoldHandle};

extern "C" {
    static mut State: c_int;
    // Global state access
    fn nvim_get_got_int() -> c_int;
    fn nvim_line_breakcheck();

    // Buffer accessors
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_fold_buf_get_line_count(buf: BufHandle) -> LinenrT;
    fn nvim_win_get_w_foldinvalid(wp: WinHandle) -> bool;
    fn nvim_win_set_w_foldinvalid(wp: WinHandle, val: bool);

    // Fold array accessors
    fn nvim_win_get_folds(wp: WinHandle) -> GArrayHandle;
    fn nvim_ga_len(gap: GArrayHandle) -> c_int;
    fn nvim_ga_fold_at(gap: GArrayHandle, idx: c_int) -> FoldHandle;
    fn nvim_fold_get_fd_top(fp: FoldHandle) -> LinenrT;
    fn nvim_fold_get_fd_len(fp: FoldHandle) -> LinenrT;
    fn nvim_fold_get_fd_flags(fp: FoldHandle) -> c_int;
    fn nvim_fold_get_fd_nested(fp: FoldHandle) -> GArrayHandle;
    fn nvim_fold_set_fd_top(fp: FoldHandle, top: LinenrT);
    fn nvim_fold_set_fd_len(fp: FoldHandle, len: LinenrT);
    fn nvim_fold_set_fd_flags(fp: FoldHandle, flags: c_int);
    fn nvim_fold_set_fd_small(fp: FoldHandle, small: c_int);

    // Fold operations
    fn nvim_changed_window_setting(wp: WinHandle);
    fn nvim_redraw_win_range_later(wp: WinHandle, top: LinenrT, bot: LinenrT);

    // Diff context
    fn nvim_get_diff_context() -> LinenrT;

    // Window option: w_p_fen (fold enable)
    fn nvim_win_get_p_fen(wp: WinHandle) -> c_int;

    // Global state for foldUpdate
    fn nvim_get_disable_fold_update() -> c_int;
    fn nvim_get_need_diff_redraw() -> c_int;
    fn nvim_set_got_int(val: c_int);
}

/// MODE_INSERT flag value (from state_defs.h).
const MODE_INSERT: c_int = 0x10;

/// MAXLNUM constant
const MAXLNUM: LinenrT = 0x7fff_ffff;

/// Binary search for line `lnum` in folds of growarray `gap`.
///
/// Returns `(found, idx)` where:
/// - `found` is true if lnum is inside a fold
/// - `idx` is the index of the fold containing lnum (if found) or the
///   first fold below lnum (careful: may equal ga_len if beyond the end)
///
/// Mirrors the C `nvim_foldFind` / `foldFind` behavior exactly.
fn fold_find_impl(gap: GArrayHandle, lnum: LinenrT) -> (bool, c_int) {
    let len = unsafe { nvim_ga_len(gap) };
    if len == 0 {
        return (false, 0);
    }

    // Binary search: low..=high are inclusive candidate indices
    let mut low: c_int = 0;
    let mut high: c_int = len - 1;
    while low <= high {
        let i = i32::midpoint(low, high);
        let fp = unsafe { nvim_ga_fold_at(gap, i) };
        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
        let fd_len = unsafe { nvim_fold_get_fd_len(fp) };
        if fd_top > lnum {
            // fold below lnum, adjust high
            high = i - 1;
        } else if fd_top + fd_len <= lnum {
            // fold above lnum, adjust low
            low = i + 1;
        } else {
            // lnum is inside this fold
            return (true, i);
        }
    }
    (false, low)
}

/// Minimum of two values
#[inline]
const fn min(a: LinenrT, b: LinenrT) -> LinenrT {
    if a < b {
        a
    } else {
        b
    }
}

/// Maximum of two values
#[inline]
const fn max(a: LinenrT, b: LinenrT) -> LinenrT {
    if a > b {
        a
    } else {
        b
    }
}

/// Extended fline state that includes lnum_save
#[derive(Debug, Clone)]
struct FlineT {
    wp: WinHandle,
    lnum: LinenrT,
    off: LinenrT,
    lnum_save: LinenrT,
    lvl: c_int,
    lvl_next: c_int,
    start: c_int,
    end: c_int,
    had_end: c_int,
    /// Parsed fold marker info (only valid for Marker method)
    marker_info: FoldMarkerInfo,
}

impl FlineT {
    fn new(wp: WinHandle) -> Self {
        Self {
            wp,
            lnum: 0,
            off: 0,
            lnum_save: 0,
            lvl: 0,
            lvl_next: -1,
            start: 0,
            end: MAX_LEVEL + 1,
            had_end: MAX_LEVEL + 1,
            marker_info: FoldMarkerInfo::default(),
        }
    }
}

/// Call the indent level getter
fn call_indent_level_getter(flp: &mut FlineT) {
    let result = foldlevel_indent_result(flp.wp, flp.lnum, flp.off);
    flp.lvl = result.lvl;
    flp.lvl_next = result.lvl_next;
    flp.start = result.start;
    flp.had_end = flp.end;
    // For indent method, end stays at MAX_LEVEL + 1 (not set by level getter)
    flp.end = MAX_LEVEL + 1;
}

/// Call the diff level getter
fn call_diff_level_getter(flp: &mut FlineT) {
    let result = foldlevel_diff_result(flp.wp, flp.lnum, flp.off);
    flp.lvl = result.lvl;
    flp.lvl_next = result.lvl_next;
    flp.start = result.start;
    flp.had_end = flp.end;
    // For diff method, end stays at MAX_LEVEL + 1 (not set by level getter)
    flp.end = MAX_LEVEL + 1;
}

/// Call the marker level getter
fn call_marker_level_getter(flp: &mut FlineT) {
    // Marker method requires passing current level - it tracks fold state across lines
    let result = foldlevel_marker_impl(flp.wp, flp.lnum, flp.off, flp.lvl, &flp.marker_info);
    flp.lvl = result.lvl;
    flp.lvl_next = result.lvl_next;
    flp.start = result.start;
    flp.had_end = flp.end;
    // For marker method, end stays at MAX_LEVEL + 1 (not set by level getter)
    flp.end = MAX_LEVEL + 1;
}

/// Type of level getter to use
#[derive(Clone, Copy, PartialEq, Eq)]
enum LevelGetterKind {
    Indent,
    Diff,
    Marker,
    Expr,
    Syntax,
}

/// Call the appropriate level getter based on kind
fn call_level_getter(flp: &mut FlineT, kind: LevelGetterKind) {
    match kind {
        LevelGetterKind::Indent => call_indent_level_getter(flp),
        LevelGetterKind::Diff => call_diff_level_getter(flp),
        LevelGetterKind::Marker => call_marker_level_getter(flp),
        LevelGetterKind::Expr => call_expr_level_getter(flp),
        LevelGetterKind::Syntax => call_syntax_level_getter(flp),
    }
}

/// Call the expr level getter
///
/// The expr level getter modifies `end` field (for 's' and '<' codes).
/// It also requires the current level for 'a' and 's' codes.
fn call_expr_level_getter(flp: &mut FlineT) {
    let result = foldlevel_expr_result(flp.wp, flp.lnum, flp.off, flp.lvl);
    flp.had_end = flp.end;
    flp.lvl = result.lvl;
    flp.lvl_next = result.lvl_next;
    flp.start = result.start;
    flp.end = result.end; // expr sets end for 's' and '<' codes
}

/// Call the syntax level getter
fn call_syntax_level_getter(flp: &mut FlineT) {
    let result = foldlevel_syntax_result(flp.wp, flp.lnum, flp.off);
    flp.lvl = result.lvl;
    flp.lvl_next = result.lvl_next;
    flp.start = result.start;
    flp.had_end = flp.end;
    flp.end = MAX_LEVEL + 1; // syntax doesn't set end
}

/// IEMS update for indent and diff methods only.
///
/// This delegates to the unified `fold_update_iems_impl` with the appropriate
/// level getter kind for indent or diff methods.
pub fn fold_update_iems_indent_impl(wp: WinHandle, top: LinenrT, bot: LinenrT) {
    let kind = if crate::foldmethod_is_diff_impl(wp) {
        LevelGetterKind::Diff
    } else {
        LevelGetterKind::Indent
    };
    fold_update_iems_impl(wp, top, bot, kind);
}

/// IEMS update for all fold methods.
///
/// Determines the fold method from the window and dispatches to the unified
/// IEMS implementation.
pub fn fold_update_iems_all_impl(wp: WinHandle, top: LinenrT, bot: LinenrT) {
    if wp.is_null() {
        return;
    }

    let kind = if crate::foldmethod_is_marker_impl(wp) {
        LevelGetterKind::Marker
    } else if crate::foldmethod_is_expr_impl(wp) {
        LevelGetterKind::Expr
    } else if crate::foldmethod_is_syntax_impl(wp) {
        LevelGetterKind::Syntax
    } else if crate::foldmethod_is_diff_impl(wp) {
        LevelGetterKind::Diff
    } else {
        LevelGetterKind::Indent
    };
    fold_update_iems_impl(wp, top, bot, kind);
}

/// Update folds for changes in the buffer of a window.
///
/// Mirrors the C `foldUpdate` function. Note that inserted/deleted lines
/// must have already been taken care of by calling foldMarkAdjust().
/// The changes are in lines from top to bot (inclusive).
pub fn fold_update_impl(wp: WinHandle, top: LinenrT, bot: LinenrT) {
    if wp.is_null() {
        return;
    }

    // Skip update when disabled or in insert mode with non-indent foldmethod.
    let disable = unsafe { nvim_get_disable_fold_update() };
    if disable != 0 {
        return;
    }
    let state = unsafe { State };
    if state & MODE_INSERT != 0 && !crate::foldmethod_is_indent_impl(wp) {
        return;
    }

    // Skip when a diff redraw is pending (will update later).
    if unsafe { nvim_get_need_diff_redraw() } != 0 {
        return;
    }

    // Mark all folds from top to bot (or bot to top) as maybe-small.
    let gap = unsafe { nvim_win_get_folds(wp) };
    let gap_len = unsafe { nvim_ga_len(gap) };
    if gap_len > 0 {
        let maybe_small_start = top.min(bot);
        let maybe_small_end = top.max(bot);

        let (_, start_idx) = fold_find_impl(gap, maybe_small_start);
        let mut idx = start_idx;
        loop {
            if idx >= gap_len {
                break;
            }
            let fp = unsafe { nvim_ga_fold_at(gap, idx) };
            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
            if fd_top > maybe_small_end {
                break;
            }
            unsafe { nvim_fold_set_fd_small(fp, tristate::K_NONE) };
            idx += 1;
        }
    }

    // Run the IEMS algorithm if applicable.
    if crate::foldmethod_is_indent_impl(wp)
        || crate::foldmethod_is_diff_impl(wp)
        || crate::foldmethod_is_expr_impl(wp)
        || crate::foldmethod_is_marker_impl(wp)
        || crate::foldmethod_is_syntax_impl(wp)
    {
        let save_got_int = unsafe { nvim_get_got_int() };
        unsafe { nvim_set_got_int(0) };
        fold_update_iems_all_impl(wp, top, bot);
        let cur_got_int = unsafe { nvim_get_got_int() };
        unsafe { nvim_set_got_int(cur_got_int | save_got_int) };
    }
}

/// Unified IEMS update for all fold methods (indent, diff, marker, expr, syntax).
///
/// This function handles the top-level IEMS algorithm, dispatching to the
/// appropriate level getter based on `kind`.
#[allow(clippy::too_many_lines)]
fn fold_update_iems_impl(wp: WinHandle, mut top: LinenrT, mut bot: LinenrT, kind: LevelGetterKind) {
    if wp.is_null() {
        return;
    }

    let buf = unsafe { nvim_win_get_buffer(wp) };
    if buf.is_null() {
        return;
    }

    // Handle w_foldinvalid
    if unsafe { nvim_win_get_w_foldinvalid(wp) } {
        top = 1;
        bot = unsafe { nvim_fold_buf_get_line_count(buf) };
        unsafe { nvim_win_set_w_foldinvalid(wp, false) };

        // Mark all folds as maybe-small
        let gap = unsafe { nvim_win_get_folds(wp) };
        crate::set_small_maybe_impl(gap);
    }

    // Add context for diff folding
    if kind == LevelGetterKind::Diff {
        let diff_context = unsafe { nvim_get_diff_context() };
        if top > diff_context {
            top -= diff_context;
        } else {
            top = 1;
        }
        bot += diff_context;
    }

    // Clamp top to buffer size
    let line_count = unsafe { nvim_fold_buf_get_line_count(buf) };
    top = min(top, line_count);

    // Initialize fold line state
    let mut flp = FlineT::new(wp);

    crate::set_fold_changed(false);
    crate::set_invalid_top(top);
    crate::set_invalid_bot(bot);

    if kind == LevelGetterKind::Marker {
        // Init marker variables to speed up foldlevelMarker()
        flp.marker_info = parse_marker_impl(wp);

        // Need to get the level of the line above top, it is used if there is
        // no marker at the top.
        if top > 1 {
            let level = crate::fold_level_win_impl(wp, top - 1);
            flp.lnum = top - 1;
            flp.lvl = level;
            call_marker_level_getter(&mut flp);

            // If a fold started here, we already had the level, if it stops
            // here, we need to use lvl_next. Could also start and end a fold
            // in the same line.
            if flp.lvl > level {
                flp.lvl = level - (flp.lvl - flp.lvl_next);
            } else {
                flp.lvl = flp.lvl_next;
            }
        }
        flp.lnum = top;
        call_marker_level_getter(&mut flp);
    } else {
        flp.lnum = top;
        if (kind == LevelGetterKind::Expr || kind == LevelGetterKind::Indent) && top > 1 {
            // Start one line back:
            // - For expr: a "<1" may indicate the end of a fold in the topline
            // - For indent: the line above "top" may have an undefined fold
            //   level, so folding it relies on the line under it.
            flp.lnum -= 1;
        }

        // Backup to a line for which the fold level is defined. Since it's
        // always defined for line one, we will stop there.
        flp.lvl = -1;
        while unsafe { nvim_get_got_int() } == 0 {
            flp.lvl_next = -1;
            call_level_getter(&mut flp, kind);

            if flp.lvl >= 0 {
                break;
            }

            if flp.lnum <= 1 {
                break;
            }
            flp.lnum -= 1;
        }
    }

    // If folding is defined by the syntax, it is possible that a change in
    // one line will cause all sub-folds of the current fold to change.
    // Adjust "bot" to point to the end of the current fold.
    if kind == LevelGetterKind::Syntax {
        let gap = unsafe { nvim_win_get_folds(wp) };
        let mut current_fdl: c_int = 0;
        let mut fold_start_lnum: LinenrT = 0;
        let mut lnum_rel = flp.lnum;
        let mut cur_gap = gap;
        let mut fpn_len: LinenrT = 0;
        let mut found = false;

        while current_fdl < flp.lvl {
            let (ff, inner_idx) = fold_find_impl(cur_gap, lnum_rel);
            if !ff {
                break;
            }
            current_fdl += 1;
            let inner_fp = unsafe { nvim_ga_fold_at(cur_gap, inner_idx) };
            let top_val = unsafe { nvim_fold_get_fd_top(inner_fp) };
            fpn_len = unsafe { nvim_fold_get_fd_len(inner_fp) };
            fold_start_lnum += top_val;
            cur_gap = unsafe { nvim_fold_get_fd_nested(inner_fp) };
            lnum_rel -= top_val;
            found = true;
        }
        if found && current_fdl == flp.lvl {
            let fold_end_lnum = fold_start_lnum + fpn_len;
            bot = max(bot, fold_end_lnum);
        }
    }

    let mut start = flp.lnum;
    let mut end = bot;

    // Do at least one line
    if start > end && end < line_count {
        end = start;
    }

    let gap = unsafe { nvim_win_get_folds(wp) };

    // Main loop
    while unsafe { nvim_get_got_int() } == 0 {
        // Stop at end of file
        if flp.lnum > line_count {
            break;
        }

        if flp.lnum > end {
            // For marker/expr/syntax methods: If a change caused a fold to be
            // removed, we need to continue at least until where it ended.
            if !kind_uses_finish(kind) {
                break;
            }

            let mut should_break = true;
            // Try to find fold at 'end' that extends beyond it
            let (ff1, idx1) = fold_find_impl(gap, end);
            // Try to find fold at flp.lnum (for level 0 check)
            let (ff2, idx2) = fold_find_impl(gap, flp.lnum);
            let extend_end = if start <= end && ff1 {
                let fp = unsafe { nvim_ga_fold_at(gap, idx1) };
                let ft = unsafe { nvim_fold_get_fd_top(fp) };
                let fl = unsafe { nvim_fold_get_fd_len(fp) };
                if ft + fl - 1 > end {
                    Some((ft, fl))
                } else {
                    None
                }
            } else {
                None
            };
            let extend_lnum = if flp.lvl == 0 && ff2 {
                let fp = unsafe { nvim_ga_fold_at(gap, idx2) };
                let ft = unsafe { nvim_fold_get_fd_top(fp) };
                let fl = unsafe { nvim_fold_get_fd_len(fp) };
                if ft < flp.lnum {
                    Some((ft, fl))
                } else {
                    None
                }
            } else {
                None
            };
            if let Some((ft, fl)) = extend_end.or(extend_lnum) {
                end = ft + fl - 1;
                should_break = false;
            } else if kind == LevelGetterKind::Syntax
                && crate::fold_level_win_impl(wp, flp.lnum) != flp.lvl
            {
                // For "syntax" method: Compare the foldlevel that the syntax
                // tells us to the foldlevel from the existing folds.
                end = flp.lnum;
                should_break = false;
            }

            if should_break {
                break;
            }
        }

        // A level 1 fold starts at a line with foldlevel > 0
        if flp.lvl > 0 {
            crate::set_invalid_top(flp.lnum);
            crate::set_invalid_bot(end);
            end = fold_update_iems_recurse(
                wp,
                gap,
                1,
                start,
                &mut flp,
                end,
                fold_flags::FD_LEVEL,
                kind,
            );
            start = flp.lnum;
        } else {
            if flp.lnum == line_count {
                break;
            }
            flp.lnum += 1;
            flp.lvl = flp.lvl_next;
            call_level_getter(&mut flp, kind);
        }
    }

    // Remove any remaining folds from start to end
    crate::fold_remove_impl(wp, gap, start, end);

    // Redraw if folds changed
    if crate::fold_changed() && unsafe { nvim_win_get_p_fen(wp) } != 0 {
        unsafe { nvim_changed_window_setting(wp) };
    }

    // Redraw past bot if we updated further
    if end != bot {
        unsafe { nvim_redraw_win_range_later(wp, top, end) };
    }

    crate::set_invalid_top(0);
}

/// Check if a level getter kind uses the "finish" behavior
/// (marker, expr, syntax methods need to continue until fold end is found).
fn kind_uses_finish(kind: LevelGetterKind) -> bool {
    matches!(
        kind,
        LevelGetterKind::Marker | LevelGetterKind::Expr | LevelGetterKind::Syntax
    )
}

/// Recursive fold update for all IEMS methods.
///
/// Generalized version that handles indent, diff, marker, expr, and syntax
/// fold methods. The `kind` parameter controls method-specific behavior at
/// four divergence points:
/// 1. Marker pre-scan: marker method may use a previous fold
/// 2. Bot extension: marker/expr/syntax extend past bot when folds changed
/// 3. Finish flag: marker/expr/syntax set finish after splits/inserts
/// 4. Truncate vs split: marker/expr/syntax truncate, indent/diff split
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::useless_let_if_seq)]
#[allow(clippy::precedence)]
#[allow(clippy::bool_to_int_with_if)]
#[allow(clippy::if_not_else)]
fn fold_update_iems_recurse(
    wp: WinHandle,
    gap: GArrayHandle,
    level: c_int,
    startlnum: LinenrT,
    flp: &mut FlineT,
    mut bot: LinenrT,
    topflags: c_int,
    kind: LevelGetterKind,
) -> LinenrT {
    let buf = unsafe { nvim_win_get_buffer(wp) };
    let line_count = unsafe { nvim_fold_buf_get_line_count(buf) } - flp.off;

    let firstlnum = flp.lnum;
    let mut startlnum2 = startlnum;
    flp.lnum_save = flp.lnum;
    let mut finish = false;

    let mut fp_idx: c_int = -1;

    // Divergence point 1: Marker pre-scan
    // If using the marker method, the start line is not the start of a fold
    // at the level we're dealing with and the level is non-zero, we must use
    // the previous fold. But ignore a fold that starts at or below startlnum.
    if kind == LevelGetterKind::Marker && flp.start <= flp.lvl - level && flp.lvl > 0 {
        let (_, found_idx) = fold_find_impl(gap, startlnum - 1);
        let gap_len = unsafe { nvim_ga_len(gap) };
        if found_idx < gap_len {
            let fp = unsafe { nvim_ga_fold_at(gap, found_idx) };
            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
            if fd_top < startlnum {
                fp_idx = found_idx;
            }
        }
    }

    // Main loop
    while unsafe { nvim_get_got_int() } == 0 {
        unsafe { nvim_line_breakcheck() };

        // Set lvl, clamped to MAX_LEVEL
        let mut lvl = min(flp.lvl, MAX_LEVEL as LinenrT) as c_int;

        // Force fold end if we're past the first line and conditions are met
        if flp.lnum > firstlnum && (level > lvl - flp.start || level >= flp.had_end) {
            lvl = 0;
        }

        // Divergence point 2: Bot extension
        // When past bot, marker/expr/syntax check for nested fold changes
        // and may extend bot. Indent/diff just break.
        if flp.lnum > bot && !finish && fp_idx >= 0 {
            if !kind_uses_finish(kind) {
                break;
            }

            // For marker/expr/syntax: check if nested folds changed
            let mut i = 0;
            let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
            let fd_len = unsafe { nvim_fold_get_fd_len(fp) };

            if lvl >= level {
                // Compute how deep the folds currently are
                let mut ll = flp.lnum - fd_top;
                let mut nested = unsafe { nvim_fold_get_fd_nested(fp) };
                let (mut ff_inner, mut inner_idx) = fold_find_impl(nested, ll);
                while ff_inner {
                    i += 1;
                    let inner_fp = unsafe { nvim_ga_fold_at(nested, inner_idx) };
                    let inner_top = unsafe { nvim_fold_get_fd_top(inner_fp) };
                    ll -= inner_top;
                    nested = unsafe { nvim_fold_get_fd_nested(inner_fp) };
                    let (nff, nidx) = fold_find_impl(nested, ll);
                    ff_inner = nff;
                    inner_idx = nidx;
                }
            }

            if lvl < level + i {
                // Need to delete a nested fold - extend bot
                let nested = unsafe { nvim_fold_get_fd_nested(fp) };
                let (ff2, fp2_idx) = fold_find_impl(nested, flp.lnum - fd_top);
                if ff2 {
                    let fp2 = unsafe { nvim_ga_fold_at(nested, fp2_idx) };
                    let fp2_top = unsafe { nvim_fold_get_fd_top(fp2) };
                    let fp2_len = unsafe { nvim_fold_get_fd_len(fp2) };
                    bot = fp2_top + fp2_len - 1 + fd_top;
                }
            } else if fd_top + fd_len <= flp.lnum && lvl >= level {
                // Fold continues, need to process until end
                finish = true;
            } else {
                break;
            }
        }

        // Handle fold creation/reuse at the start of a nested fold
        if fp_idx < 0
            && (lvl != level
                || flp.lnum_save >= bot
                || flp.start != 0
                || flp.had_end <= MAX_LEVEL as c_int
                || flp.lnum == line_count)
        {
            // Inner loop: find or create folds
            while unsafe { nvim_get_got_int() } == 0 {
                // Find or create a fold
                let concat = if flp.start != 0 || flp.had_end <= MAX_LEVEL as c_int {
                    0
                } else {
                    1
                };

                let gap_len = unsafe { nvim_ga_len(gap) };

                if gap_len > 0 {
                    let (found1, idx1) = fold_find_impl(gap, startlnum);
                    let (found2, idx2) = fold_find_impl(gap, firstlnum - concat);
                    // Pick the matching index: first check idx1 conditions,
                    // then idx2 conditions (mirrors C's sequential found_idx mutation).
                    let idx1_matches = found1
                        || (idx1 < gap_len && {
                            let fp = unsafe { nvim_ga_fold_at(gap, idx1) };
                            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
                            fd_top <= firstlnum
                        });
                    let idx2_matches = found2
                        || (idx2 < gap_len && {
                            let fp = unsafe { nvim_ga_fold_at(gap, idx2) };
                            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
                            (lvl < level && fd_top < flp.lnum)
                                || (lvl >= level && fd_top <= flp.lnum_save)
                        });
                    let matched_idx = if idx1_matches {
                        Some(idx1)
                    } else if idx2_matches {
                        Some(idx2)
                    } else {
                        None
                    };

                    if let Some(found_idx) = matched_idx {
                        let fp = unsafe { nvim_ga_fold_at(gap, found_idx) };
                        let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
                        let fd_len = unsafe { nvim_fold_get_fd_len(fp) };

                        if fd_top + fd_len + concat > firstlnum {
                            // Use existing fold
                            if fd_top == firstlnum {
                                // Perfect match
                                fp_idx = found_idx;
                            } else if fd_top >= startlnum {
                                // Adjust fold position
                                if fd_top > firstlnum {
                                    unsafe {
                                        let nested = nvim_fold_get_fd_nested(fp);
                                        crate::fold_mark_adjust_recurse_impl(
                                            wp,
                                            nested,
                                            0,
                                            MAXLNUM,
                                            fd_top - firstlnum,
                                            0,
                                        );
                                    }
                                } else {
                                    unsafe {
                                        let nested = nvim_fold_get_fd_nested(fp);
                                        crate::fold_mark_adjust_recurse_impl(
                                            wp,
                                            nested,
                                            0,
                                            firstlnum - fd_top - 1,
                                            MAXLNUM,
                                            fd_top - firstlnum,
                                        );
                                    }
                                }
                                unsafe {
                                    nvim_fold_set_fd_len(fp, fd_len + fd_top - firstlnum);
                                    nvim_fold_set_fd_top(fp, firstlnum);
                                    nvim_fold_set_fd_small(fp, tristate::K_NONE);
                                }
                                crate::set_fold_changed(true);
                                fp_idx = found_idx;
                            } else if flp.start != 0 && lvl == level || firstlnum != startlnum {
                                // Need to split the fold
                                let (breakstart, breakend) = if firstlnum != startlnum {
                                    (startlnum, firstlnum)
                                } else {
                                    (flp.lnum, flp.lnum)
                                };

                                let fp = unsafe { nvim_ga_fold_at(gap, found_idx) };
                                let nested = unsafe { nvim_fold_get_fd_nested(fp) };
                                let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
                                crate::fold_remove_impl(
                                    wp,
                                    nested,
                                    breakstart - fd_top,
                                    breakend - fd_top,
                                );
                                crate::fold_split_impl(
                                    buf,
                                    gap,
                                    found_idx,
                                    breakstart,
                                    breakend - 1,
                                );
                                fp_idx = found_idx + 1;

                                // Divergence point 3a: finish flag after split
                                if kind_uses_finish(kind) {
                                    finish = true;
                                }
                            }

                            // Try to merge with previous fold if possible
                            if fp_idx >= 0 {
                                let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
                                let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
                                if fd_top == startlnum && concat != 0 && fp_idx > 0 {
                                    let fp2 = unsafe { nvim_ga_fold_at(gap, fp_idx - 1) };
                                    let fd2_top = unsafe { nvim_fold_get_fd_top(fp2) };
                                    let fd2_len = unsafe { nvim_fold_get_fd_len(fp2) };
                                    if fd2_top + fd2_len == fd_top {
                                        crate::fold_merge_impl(wp, fp_idx - 1, gap, fp_idx);
                                        fp_idx -= 1;
                                    }
                                }
                            }
                            break;
                        }
                        if fd_top >= startlnum {
                            // Delete fold that's no longer valid. Continue
                            // looking for the next one.
                            crate::delete_fold_entry_impl(wp, gap, found_idx, true);
                        } else {
                            // Truncate fold that extends past startlnum
                            unsafe {
                                nvim_fold_set_fd_len(fp, startlnum - fd_top);
                                let nested = nvim_fold_get_fd_nested(fp);
                                crate::fold_mark_adjust_recurse_impl(
                                    wp,
                                    nested,
                                    startlnum - fd_top,
                                    MAXLNUM,
                                    MAXLNUM,
                                    0,
                                );
                            }
                            crate::set_fold_changed(true);
                        }
                    } else {
                        // No existing fold found, insert a new one
                        let gap_len2 = unsafe { nvim_ga_len(gap) };
                        let mut idx = 0;

                        if gap_len2 > 0 {
                            let (_, fi) = fold_find_impl(gap, startlnum);
                            idx = fi;
                        }

                        crate::fold_insert_impl(gap, idx);
                        let fp = unsafe { nvim_ga_fold_at(gap, idx) };

                        unsafe {
                            nvim_fold_set_fd_top(fp, firstlnum);
                            nvim_fold_set_fd_len(fp, bot - firstlnum + 1);

                            if topflags == fold_flags::FD_OPEN {
                                nvim_win_set_w_fold_manual(wp, true);
                                nvim_fold_set_fd_flags(fp, fold_flags::FD_OPEN);
                            } else if idx <= 0 {
                                nvim_fold_set_fd_flags(fp, topflags);
                                if topflags != fold_flags::FD_LEVEL {
                                    nvim_win_set_w_fold_manual(wp, true);
                                }
                            } else {
                                let prev_fp = nvim_ga_fold_at(gap, idx - 1);
                                let prev_flags = nvim_fold_get_fd_flags(prev_fp);
                                nvim_fold_set_fd_flags(fp, prev_flags);
                            }
                            nvim_fold_set_fd_small(fp, tristate::K_NONE);
                        }
                        crate::set_fold_changed(true);
                        fp_idx = idx;

                        // Divergence point 3b: finish flag after insert
                        if kind_uses_finish(kind) {
                            finish = true;
                        }
                        break;
                    }
                } else {
                    // gap is empty, insert a new fold
                    let idx = 0;
                    crate::fold_insert_impl(gap, idx);
                    let fp = unsafe { nvim_ga_fold_at(gap, idx) };

                    unsafe {
                        nvim_fold_set_fd_top(fp, firstlnum);
                        nvim_fold_set_fd_len(fp, bot - firstlnum + 1);

                        if topflags == fold_flags::FD_OPEN {
                            nvim_win_set_w_fold_manual(wp, true);
                            nvim_fold_set_fd_flags(fp, fold_flags::FD_OPEN);
                        } else {
                            nvim_fold_set_fd_flags(fp, topflags);
                            if topflags != fold_flags::FD_LEVEL {
                                nvim_win_set_w_fold_manual(wp, true);
                            }
                        }
                        nvim_fold_set_fd_small(fp, tristate::K_NONE);
                    }
                    crate::set_fold_changed(true);
                    fp_idx = idx;

                    // Divergence point 3b: finish flag after insert
                    if kind_uses_finish(kind) {
                        finish = true;
                    }
                    break;
                }
            }
        }

        // Found a line with lower foldlevel - fold ends
        if lvl < level || flp.lnum > line_count {
            break;
        }

        // Handle nested folds
        if lvl > level && fp_idx >= 0 {
            bot = max(bot, flp.lnum);

            let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
            let nested = unsafe { nvim_fold_get_fd_nested(fp) };
            let fd_flags = unsafe { nvim_fold_get_fd_flags(fp) };

            flp.lnum = flp.lnum_save - fd_top;
            flp.off += fd_top;

            bot = fold_update_iems_recurse(
                wp,
                nested,
                level + 1,
                startlnum2 - fd_top,
                flp,
                bot - fd_top,
                fd_flags,
                kind,
            );

            // Re-fetch fp after recursion (array may have changed)
            let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
            let fd_top = unsafe { nvim_fold_get_fd_top(fp) };

            flp.lnum += fd_top;
            flp.lnum_save += fd_top;
            flp.off -= fd_top;
            bot += fd_top;
            startlnum2 = flp.lnum;
        } else {
            // Get level of next line
            flp.lnum = flp.lnum_save;
            let ll = flp.lnum + 1;

            while unsafe { nvim_get_got_int() } == 0 {
                // Make the previous level available to foldlevel()
                crate::set_prev_lnum(flp.lnum);
                crate::set_prev_lnum_lvl(flp.lvl);

                flp.lnum += 1;
                if flp.lnum > line_count {
                    break;
                }
                flp.lvl = flp.lvl_next;

                call_level_getter(flp, kind);

                if flp.lvl >= 0 || flp.had_end <= MAX_LEVEL as c_int {
                    break;
                }
            }
            crate::set_prev_lnum(0);

            if flp.lnum > line_count {
                break;
            }

            flp.lnum_save = flp.lnum;
            flp.lnum = ll;
        }
    }

    if fp_idx < 0 {
        return bot;
    }

    // Finalize the fold
    let fp = unsafe { nvim_ga_fold_at(gap, fp_idx) };
    let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
    let fd_len = unsafe { nvim_fold_get_fd_len(fp) };

    // Extend fold if needed
    if fd_len < flp.lnum - fd_top {
        unsafe {
            nvim_fold_set_fd_len(fp, flp.lnum - fd_top);
            nvim_fold_set_fd_small(fp, tristate::K_NONE);
        }
        crate::set_fold_changed(true);
    } else if fd_top + fd_len > line_count {
        unsafe {
            nvim_fold_set_fd_len(fp, line_count - fd_top + 1);
        }
    }

    // Delete contained folds from end of last found to current position
    let nested = unsafe { nvim_fold_get_fd_nested(fp) };
    let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
    crate::fold_remove_impl(wp, nested, startlnum2 - fd_top, flp.lnum - 1 - fd_top);

    // Handle fold end
    // Divergence point 4: Truncate vs split
    let lvl = min(flp.lvl, MAX_LEVEL as LinenrT) as c_int;
    if lvl < level {
        let fd_len = unsafe { nvim_fold_get_fd_len(fp) };
        if fd_len != flp.lnum - fd_top {
            if fd_top + fd_len - 1 > bot {
                if kind_uses_finish(kind) {
                    // marker/expr/syntax: truncate the fold and make sure the
                    // previously included lines are processed again
                    bot = fd_top + fd_len - 1;
                    unsafe {
                        nvim_fold_set_fd_len(fp, flp.lnum - fd_top);
                    }
                } else {
                    // indent/diff: split fold to create a new one below bot
                    crate::fold_split_impl(buf, gap, fp_idx, flp.lnum, bot);
                }
            } else {
                unsafe {
                    nvim_fold_set_fd_len(fp, flp.lnum - fd_top);
                }
            }
            crate::set_fold_changed(true);
        }
    }

    // Delete/adjust following folds
    loop {
        let gap_len = unsafe { nvim_ga_len(gap) };
        if fp_idx + 1 >= gap_len {
            break;
        }

        let fp2 = unsafe { nvim_ga_fold_at(gap, fp_idx + 1) };
        let fd2_top = unsafe { nvim_fold_get_fd_top(fp2) };
        let fd2_len = unsafe { nvim_fold_get_fd_len(fp2) };

        if fd2_top > flp.lnum {
            break;
        }

        if fd2_top + fd2_len > flp.lnum {
            if fd2_top < flp.lnum {
                // Make fold start at lnum
                let nested = unsafe { nvim_fold_get_fd_nested(fp2) };
                unsafe {
                    crate::fold_mark_adjust_recurse_impl(
                        wp,
                        nested,
                        0,
                        flp.lnum - fd2_top - 1,
                        MAXLNUM,
                        fd2_top - flp.lnum,
                    );
                    nvim_fold_set_fd_len(fp2, fd2_len - (flp.lnum - fd2_top));
                    nvim_fold_set_fd_top(fp2, flp.lnum);
                }
                crate::set_fold_changed(true);
            }

            if lvl >= level {
                // Merge with following fold
                crate::fold_merge_impl(wp, fp_idx, gap, fp_idx + 1);
            }
            break;
        }

        crate::set_fold_changed(true);
        crate::delete_fold_entry_impl(wp, gap, fp_idx + 1, true);
    }

    max(bot, flp.lnum - 1)
}

extern "C" {
    fn nvim_win_set_w_fold_manual(wp: WinHandle, val: bool);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_line_state_default() {
        let state = FoldLineState::default();
        assert_eq!(state.lvl, 0);
        assert_eq!(state.lvl_next, -1);
        assert_eq!(state.start, 0);
        assert_eq!(state.end, MAX_LEVEL + 1);
    }

    #[test]
    fn test_fold_update_context_range() {
        let ctx = FoldUpdateContext::new(10, 50);
        assert!(ctx.is_in_range(10));
        assert!(ctx.is_in_range(30));
        assert!(ctx.is_in_range(50));
        assert!(!ctx.is_in_range(9));
        assert!(!ctx.is_in_range(51));
    }

    #[test]
    fn test_fold_update_type_range() {
        assert_eq!(FoldUpdateType::All.range(), (1, LinenrT::MAX));
        assert_eq!(FoldUpdateType::Range(5, 15).range(), (5, 15));
        assert_eq!(FoldUpdateType::Insert(10, 20).range(), (10, 20));
    }

    #[test]
    fn test_level_getter_needs_line_above() {
        assert!(LevelGetterType::Expr.needs_line_above());
        assert!(LevelGetterType::Indent.needs_line_above());
        assert!(!LevelGetterType::Marker.needs_line_above());
        assert!(!LevelGetterType::Syntax.needs_line_above());
        assert!(!LevelGetterType::Diff.needs_line_above());
    }
}
