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

use crate::level::FoldLevelResult;
use crate::{fold_flags, tristate, FoldHandle};

extern "C" {
    // Global state access
    fn nvim_get_got_int() -> c_int;
    fn nvim_line_breakcheck();

    // Buffer accessors
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_fold_buf_get_line_count(buf: BufHandle) -> LinenrT;
    fn nvim_win_get_foldinvalid(wp: WinHandle) -> c_int;
    fn nvim_win_set_foldinvalid(wp: WinHandle, val: c_int);

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

    // Fold method checks
    fn nvim_foldmethod_is_diff(wp: WinHandle) -> c_int;

    // Fold operations
    #[allow(dead_code)]
    fn nvim_foldLevelWin(wp: WinHandle, lnum: LinenrT) -> c_int;
    fn nvim_changed_window_setting(wp: WinHandle);
    fn nvim_redraw_win_range_later(wp: WinHandle, top: LinenrT, bot: LinenrT);

    // Level getters (call C implementations)
    fn nvim_foldlevelIndent(wp: WinHandle, lnum: LinenrT, off: LinenrT) -> FoldLevelResult;
    fn nvim_foldlevelDiff(wp: WinHandle, lnum: LinenrT, off: LinenrT) -> FoldLevelResult;
    #[allow(dead_code)]
    fn nvim_foldlevelMarker(
        wp: WinHandle,
        lnum: LinenrT,
        off: LinenrT,
        current_lvl: c_int,
    ) -> FoldLevelResult;

    // Marker parsing
    #[allow(dead_code)]
    fn nvim_parseMarker(wp: WinHandle);

    // Fold method check
    #[allow(dead_code)]
    fn nvim_foldmethod_is_marker(wp: WinHandle) -> c_int;

    // Global fold changed flag
    fn nvim_get_fold_changed() -> bool;
    fn nvim_set_fold_changed(val: bool);

    // Diff context
    fn nvim_get_diff_context() -> LinenrT;

    // Fold manipulation
    fn nvim_foldRemove(wp: WinHandle, gap: GArrayHandle, top: LinenrT, bot: LinenrT);
    fn nvim_foldInsert(gap: GArrayHandle, i: c_int);
    fn nvim_foldSplit(buf: BufHandle, gap: GArrayHandle, i: c_int, top: LinenrT, bot: LinenrT);
    fn nvim_deleteFoldEntry(wp: WinHandle, gap: GArrayHandle, idx: c_int, recursive: c_int);
    fn nvim_foldMerge(wp: WinHandle, fp1_idx: c_int, gap: GArrayHandle, fp2_idx: c_int);
    fn nvim_foldMarkAdjustRecurse(
        wp: WinHandle,
        gap: GArrayHandle,
        line1: LinenrT,
        line2: LinenrT,
        amount: LinenrT,
        amount_after: LinenrT,
    );

    // setSmallMaybe
    fn nvim_setSmallMaybe(gap: GArrayHandle);

    // foldFind - returns index or -1 if not found
    fn nvim_foldFind(gap: GArrayHandle, lnum: LinenrT, found_idx: *mut c_int) -> c_int;

    // Window option: w_p_fen (fold enable)
    fn nvim_win_get_p_fen(wp: WinHandle) -> c_int;
}

/// MAXLNUM constant
const MAXLNUM: LinenrT = 0x7fff_ffff;

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
#[repr(C)]
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
        }
    }
}

/// Call the indent level getter
fn call_indent_level_getter(flp: &mut FlineT) {
    let result = unsafe { nvim_foldlevelIndent(flp.wp, flp.lnum, flp.off) };
    flp.lvl = result.lvl;
    flp.lvl_next = result.lvl_next;
    flp.start = result.start;
    flp.had_end = flp.end;
    // For indent method, end stays at MAX_LEVEL + 1 (not set by level getter)
    flp.end = MAX_LEVEL + 1;
}

/// Call the diff level getter
fn call_diff_level_getter(flp: &mut FlineT) {
    let result = unsafe { nvim_foldlevelDiff(flp.wp, flp.lnum, flp.off) };
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
    let result = unsafe { nvim_foldlevelMarker(flp.wp, flp.lnum, flp.off, flp.lvl) };
    flp.lvl = result.lvl;
    flp.lvl_next = result.lvl_next;
    flp.start = result.start;
    flp.had_end = flp.end;
    // For marker method, end stays at MAX_LEVEL + 1 (not set by level getter)
    flp.end = MAX_LEVEL + 1;
}

/// Type of level getter to use
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum LevelGetterKind {
    Indent,
    Diff,
    Marker,
}

/// Call the appropriate level getter based on kind
#[allow(dead_code)]
fn call_level_getter(flp: &mut FlineT, kind: LevelGetterKind) {
    match kind {
        LevelGetterKind::Indent => call_indent_level_getter(flp),
        LevelGetterKind::Diff => call_diff_level_getter(flp),
        LevelGetterKind::Marker => call_marker_level_getter(flp),
    }
}

/// IEMS update for indent and diff methods only.
///
/// This is a simplified version that handles just indent and diff fold methods.
/// Marker, expr, and syntax methods continue to use the C implementation.
#[allow(clippy::too_many_lines)]
pub fn fold_update_iems_indent_impl(wp: WinHandle, mut top: LinenrT, mut bot: LinenrT) {
    if wp.is_null() {
        return;
    }

    // Check if we're being called recursively (invalid_top != 0 check in C)
    // We use a simple guard by checking got_int state
    let buf = unsafe { nvim_win_get_buffer(wp) };
    if buf.is_null() {
        return;
    }

    // Handle w_foldinvalid
    if unsafe { nvim_win_get_foldinvalid(wp) } != 0 {
        top = 1;
        bot = unsafe { nvim_fold_buf_get_line_count(buf) };
        unsafe { nvim_win_set_foldinvalid(wp, 0) };

        // Mark all folds as maybe-small
        let gap = unsafe { nvim_win_get_folds(wp) };
        unsafe { nvim_setSmallMaybe(gap) };
    }

    // Add context for diff folding
    let is_diff = unsafe { nvim_foldmethod_is_diff(wp) } != 0;
    if is_diff {
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
    flp.lnum = top;

    unsafe { nvim_set_fold_changed(false) };

    // Determine the level getter
    let use_indent = !is_diff;

    // For indent method, start one line back to handle undefined fold levels
    if use_indent && top > 1 {
        flp.lnum -= 1;
    }

    // Backup to a line for which the fold level is defined
    flp.lvl = -1;
    while unsafe { nvim_get_got_int() } == 0 {
        flp.lvl_next = -1;

        if use_indent {
            call_indent_level_getter(&mut flp);
        } else {
            call_diff_level_getter(&mut flp);
        }

        if flp.lvl >= 0 {
            break;
        }

        if flp.lnum <= 1 {
            break;
        }
        flp.lnum -= 1;
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

        // For indent/diff methods, stop when past end
        if flp.lnum > end {
            break;
        }

        // A level 1 fold starts at a line with foldlevel > 0
        if flp.lvl > 0 {
            end = fold_update_iems_recurse_indent(
                wp,
                gap,
                1,
                start,
                &mut flp,
                end,
                fold_flags::FD_LEVEL,
                use_indent,
            );
            start = flp.lnum;
        } else {
            if flp.lnum == line_count {
                break;
            }
            flp.lnum += 1;
            flp.lvl = flp.lvl_next;

            if use_indent {
                call_indent_level_getter(&mut flp);
            } else {
                call_diff_level_getter(&mut flp);
            }
        }
    }

    // Remove any remaining folds from start to end
    unsafe { nvim_foldRemove(wp, gap, start, end) };

    // Redraw if folds changed
    if unsafe { nvim_get_fold_changed() } && unsafe { nvim_win_get_p_fen(wp) } != 0 {
        unsafe { nvim_changed_window_setting(wp) };
    }

    // Redraw past bot if we updated further
    if end != bot {
        unsafe { nvim_redraw_win_range_later(wp, top, end) };
    }
}

/// Recursive fold update for indent/diff methods.
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::useless_let_if_seq)]
#[allow(clippy::precedence)]
#[allow(clippy::bool_to_int_with_if)]
#[allow(clippy::if_not_else)]
fn fold_update_iems_recurse_indent(
    wp: WinHandle,
    gap: GArrayHandle,
    level: c_int,
    startlnum: LinenrT,
    flp: &mut FlineT,
    mut bot: LinenrT,
    topflags: c_int,
    use_indent: bool,
) -> LinenrT {
    let buf = unsafe { nvim_win_get_buffer(wp) };
    let line_count = unsafe { nvim_fold_buf_get_line_count(buf) } - flp.off;

    let firstlnum = flp.lnum;
    let mut startlnum2 = startlnum;
    flp.lnum_save = flp.lnum;

    let mut fp_idx: c_int = -1;

    // Main loop
    while unsafe { nvim_get_got_int() } == 0 {
        unsafe { nvim_line_breakcheck() };

        // Set lvl, clamped to MAX_LEVEL
        let mut lvl = min(flp.lvl, MAX_LEVEL as LinenrT) as c_int;

        // Force fold end if we're past the first line and conditions are met
        if flp.lnum > firstlnum && (level > lvl - flp.start || level >= flp.had_end) {
            lvl = 0;
        }

        // For indent/diff, stop when past bot
        if flp.lnum > bot {
            break;
        }

        // Handle fold creation/reuse at the start of a nested fold
        if fp_idx < 0
            && (lvl != level
                || flp.lnum_save >= bot
                || flp.start != 0
                || flp.had_end <= MAX_LEVEL as c_int
                || flp.lnum == line_count)
        {
            // Find or create a fold
            let concat = if flp.start != 0 || flp.had_end <= MAX_LEVEL as c_int {
                0
            } else {
                1
            };

            let gap_len = unsafe { nvim_ga_len(gap) };

            if gap_len > 0 {
                let mut found_idx: c_int = 0;
                let found = unsafe { nvim_foldFind(gap, startlnum, &raw mut found_idx) };

                if found != 0 || found_idx < gap_len {
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
                                    nvim_foldMarkAdjustRecurse(
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
                                    nvim_foldMarkAdjustRecurse(
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
                                nvim_set_fold_changed(true);
                            }
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
                            unsafe {
                                nvim_foldRemove(wp, nested, breakstart - fd_top, breakend - fd_top);
                                nvim_foldSplit(buf, gap, found_idx, breakstart, breakend - 1);
                            }
                            fp_idx = found_idx + 1;
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
                                    unsafe {
                                        nvim_foldMerge(wp, fp_idx - 1, gap, fp_idx);
                                    }
                                    fp_idx -= 1;
                                }
                            }
                        }
                    } else if fd_top >= startlnum {
                        // Delete fold that's no longer valid
                        unsafe {
                            nvim_deleteFoldEntry(wp, gap, found_idx, 1);
                        }
                    } else {
                        // Truncate fold that extends past startlnum
                        unsafe {
                            nvim_fold_set_fd_len(fp, startlnum - fd_top);
                            let nested = nvim_fold_get_fd_nested(fp);
                            nvim_foldMarkAdjustRecurse(
                                wp,
                                nested,
                                startlnum - fd_top,
                                MAXLNUM,
                                MAXLNUM,
                                0,
                            );
                            nvim_set_fold_changed(true);
                        }
                    }
                }
            }

            // If no fold found/created, insert a new one
            if fp_idx < 0 {
                let gap_len = unsafe { nvim_ga_len(gap) };
                let mut idx = 0;

                if gap_len > 0 {
                    let mut found_idx: c_int = 0;
                    unsafe { nvim_foldFind(gap, startlnum, &raw mut found_idx) };
                    idx = found_idx;
                }

                unsafe { nvim_foldInsert(gap, idx) };
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
                    nvim_set_fold_changed(true);
                }
                fp_idx = idx;
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

            bot = fold_update_iems_recurse_indent(
                wp,
                nested,
                level + 1,
                startlnum2 - fd_top,
                flp,
                bot - fd_top,
                fd_flags,
                use_indent,
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
                flp.lnum += 1;
                if flp.lnum > line_count {
                    break;
                }
                flp.lvl = flp.lvl_next;

                if use_indent {
                    call_indent_level_getter(flp);
                } else {
                    call_diff_level_getter(flp);
                }

                if flp.lvl >= 0 || flp.had_end <= MAX_LEVEL as c_int {
                    break;
                }
            }

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
            nvim_set_fold_changed(true);
        }
    } else if fd_top + fd_len > line_count {
        unsafe {
            nvim_fold_set_fd_len(fp, line_count - fd_top + 1);
        }
    }

    // Delete contained folds from end of last found to current position
    let nested = unsafe { nvim_fold_get_fd_nested(fp) };
    let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
    unsafe {
        nvim_foldRemove(wp, nested, startlnum2 - fd_top, flp.lnum - 1 - fd_top);
    }

    // Handle fold end
    let lvl = min(flp.lvl, MAX_LEVEL as LinenrT) as c_int;
    if lvl < level {
        let fd_len = unsafe { nvim_fold_get_fd_len(fp) };
        if fd_len != flp.lnum - fd_top {
            if fd_top + fd_len - 1 > bot {
                // For indent/diff: split fold below bot
                unsafe {
                    nvim_foldSplit(buf, gap, fp_idx, flp.lnum, bot);
                }
            } else {
                unsafe {
                    nvim_fold_set_fd_len(fp, flp.lnum - fd_top);
                }
            }
            unsafe { nvim_set_fold_changed(true) };
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
                    nvim_foldMarkAdjustRecurse(
                        wp,
                        nested,
                        0,
                        flp.lnum - fd2_top - 1,
                        MAXLNUM,
                        fd2_top - flp.lnum,
                    );
                    nvim_fold_set_fd_len(fp2, fd2_len - (flp.lnum - fd2_top));
                    nvim_fold_set_fd_top(fp2, flp.lnum);
                    nvim_set_fold_changed(true);
                }
            }

            if lvl >= level {
                // Merge with following fold
                unsafe {
                    nvim_foldMerge(wp, fp_idx, gap, fp_idx + 1);
                }
            }
            break;
        }

        unsafe {
            nvim_set_fold_changed(true);
            nvim_deleteFoldEntry(wp, gap, fp_idx + 1, 1);
        }
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
