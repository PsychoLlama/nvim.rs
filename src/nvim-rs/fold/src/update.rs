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
