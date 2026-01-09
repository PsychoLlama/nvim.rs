//! Fold command handlers
//!
//! This module provides Rust implementations for fold commands,
//! including open, close, toggle, and fold creation/deletion.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::return_self_not_must_use)]

use std::ffi::c_int;

/// Line number type
type LinenrT = i32;

// =============================================================================
// Fold Command Types
// =============================================================================

/// Types of fold operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldOp {
    /// Open a fold
    Open = 0,
    /// Close a fold
    Close = 1,
    /// Toggle fold open/closed
    Toggle = 2,
    /// Open all folds recursively
    OpenRecursive = 3,
    /// Close all folds recursively
    CloseRecursive = 4,
    /// Open to cursor level
    OpenCursor = 5,
    /// Close folds except cursor
    CloseMore = 6,
    /// Open more folds
    OpenMore = 7,
}

impl FoldOp {
    /// Check if this operation opens folds
    pub const fn is_opening(self) -> bool {
        matches!(
            self,
            Self::Open | Self::OpenRecursive | Self::OpenCursor | Self::OpenMore
        )
    }

    /// Check if this operation closes folds
    pub const fn is_closing(self) -> bool {
        matches!(self, Self::Close | Self::CloseRecursive | Self::CloseMore)
    }

    /// Check if this operation is recursive
    pub const fn is_recursive(self) -> bool {
        matches!(self, Self::OpenRecursive | Self::CloseRecursive)
    }
}

// =============================================================================
// Fold Command Result
// =============================================================================

/// Result of a fold command
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldCmdResult {
    /// Command succeeded
    Success = 0,
    /// No fold found at position
    NoFold = 1,
    /// Fold already in desired state
    AlreadyState = 2,
    /// Invalid range
    InvalidRange = 3,
    /// Cannot modify (e.g., non-manual method)
    CannotModify = 4,
    /// General error
    Error = 5,
}

impl FoldCmdResult {
    /// Check if result is success
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if result is an error
    pub const fn is_error(self) -> bool {
        !self.is_success() && !matches!(self, Self::AlreadyState)
    }
}

// =============================================================================
// Fold Command Context
// =============================================================================

/// Context for fold command execution
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldCmdContext {
    /// Start line of range
    pub line1: LinenrT,
    /// End line of range
    pub line2: LinenrT,
    /// Count argument
    pub count: c_int,
    /// Whether to force the operation
    pub force: bool,
    /// Whether to apply recursively
    pub recursive: bool,
}

impl Default for FoldCmdContext {
    fn default() -> Self {
        Self {
            line1: 0,
            line2: 0,
            count: 1,
            force: false,
            recursive: false,
        }
    }
}

impl FoldCmdContext {
    /// Create for a single line
    pub const fn at_line(line: LinenrT) -> Self {
        Self {
            line1: line,
            line2: line,
            count: 1,
            force: false,
            recursive: false,
        }
    }

    /// Create for a range
    pub const fn for_range(line1: LinenrT, line2: LinenrT) -> Self {
        Self {
            line1,
            line2,
            count: 1,
            force: false,
            recursive: false,
        }
    }

    /// Set count
    pub const fn with_count(mut self, count: c_int) -> Self {
        self.count = count;
        self
    }

    /// Set recursive flag
    pub const fn with_recursive(mut self) -> Self {
        self.recursive = true;
        self
    }

    /// Check if this is a single line context
    pub const fn is_single_line(&self) -> bool {
        self.line1 == self.line2
    }

    /// Get range size
    pub const fn range_size(&self) -> LinenrT {
        if self.line2 >= self.line1 {
            self.line2 - self.line1 + 1
        } else {
            0
        }
    }
}

// =============================================================================
// Create/Delete Fold Commands
// =============================================================================

/// Create fold command
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CreateFoldCmd {
    /// Start line
    pub start_line: LinenrT,
    /// End line
    pub end_line: LinenrT,
    /// Whether to create with markers
    pub use_markers: bool,
}

impl CreateFoldCmd {
    /// Create a new fold creation command
    pub const fn new(start: LinenrT, end: LinenrT) -> Self {
        Self {
            start_line: start,
            end_line: end,
            use_markers: false,
        }
    }

    /// Create with marker method
    pub const fn with_markers(mut self) -> Self {
        self.use_markers = true;
        self
    }

    /// Check if range is valid
    pub const fn is_valid(&self) -> bool {
        self.start_line > 0 && self.end_line >= self.start_line
    }

    /// Get line count
    pub const fn line_count(&self) -> LinenrT {
        if self.is_valid() {
            self.end_line - self.start_line + 1
        } else {
            0
        }
    }
}

/// Delete fold command
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DeleteFoldCmd {
    /// Line to delete fold at
    pub line: LinenrT,
    /// Whether to delete recursively
    pub recursive: bool,
    /// Whether to delete markers
    pub delete_markers: bool,
}

impl DeleteFoldCmd {
    /// Create a new fold deletion command
    pub const fn at_line(line: LinenrT) -> Self {
        Self {
            line,
            recursive: false,
            delete_markers: false,
        }
    }

    /// Set recursive flag
    pub const fn with_recursive(mut self) -> Self {
        self.recursive = true;
        self
    }

    /// Set delete markers flag
    pub const fn with_markers(mut self) -> Self {
        self.delete_markers = true;
        self
    }
}

// =============================================================================
// Fold Level Commands
// =============================================================================

/// Set fold level command
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SetFoldLevelCmd {
    /// Target fold level
    pub level: c_int,
    /// Line range start (0 = current line)
    pub line1: LinenrT,
    /// Line range end (0 = current line)
    pub line2: LinenrT,
}

impl SetFoldLevelCmd {
    /// Create for whole buffer
    pub const fn global(level: c_int) -> Self {
        Self {
            level,
            line1: 1,
            line2: 0x7FFF_FFFF, // Max line number
        }
    }

    /// Create for specific line
    pub const fn at_line(level: c_int, line: LinenrT) -> Self {
        Self {
            level,
            line1: line,
            line2: line,
        }
    }

    /// Check if level is valid
    pub const fn is_valid_level(&self) -> bool {
        self.level >= 0
    }
}

// =============================================================================
// Fold Navigation Commands
// =============================================================================

/// Direction for fold navigation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldNavDirection {
    /// Go to previous fold
    Previous = -1,
    /// Go to next fold
    Next = 1,
    /// Go to parent fold
    Parent = 2,
    /// Go to first child fold
    Child = 3,
}

/// Fold navigation command
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldNavCmd {
    /// Direction to navigate
    pub direction: FoldNavDirection,
    /// Count (number of folds to skip)
    pub count: c_int,
    /// Starting line
    pub from_line: LinenrT,
    /// Whether to stay in closed folds
    pub closed_only: bool,
}

impl FoldNavCmd {
    /// Create navigation to next fold
    pub const fn next(from: LinenrT) -> Self {
        Self {
            direction: FoldNavDirection::Next,
            count: 1,
            from_line: from,
            closed_only: false,
        }
    }

    /// Create navigation to previous fold
    pub const fn previous(from: LinenrT) -> Self {
        Self {
            direction: FoldNavDirection::Previous,
            count: 1,
            from_line: from,
            closed_only: false,
        }
    }

    /// Set count
    pub const fn with_count(mut self, count: c_int) -> Self {
        self.count = count;
        self
    }
}

/// Result of fold navigation
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldNavResult {
    /// Target line (-1 if not found)
    pub line: LinenrT,
    /// Whether fold was found
    pub found: bool,
}

impl FoldNavResult {
    /// Create a not-found result
    pub const fn not_found() -> Self {
        Self {
            line: -1,
            found: false,
        }
    }

    /// Create a found result
    pub const fn found_at(line: LinenrT) -> Self {
        Self { line, found: true }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if fold op is opening
#[no_mangle]
pub extern "C" fn rs_fold_op_is_opening(op: FoldOp) -> c_int {
    c_int::from(op.is_opening())
}

/// FFI export: Check if fold op is closing
#[no_mangle]
pub extern "C" fn rs_fold_op_is_closing(op: FoldOp) -> c_int {
    c_int::from(op.is_closing())
}

/// FFI export: Check if fold op is recursive
#[no_mangle]
pub extern "C" fn rs_fold_op_is_recursive(op: FoldOp) -> c_int {
    c_int::from(op.is_recursive())
}

/// FFI export: Check if result is success
#[no_mangle]
pub extern "C" fn rs_fold_cmd_result_success(result: FoldCmdResult) -> c_int {
    c_int::from(result.is_success())
}

/// FFI export: Create fold context for line
#[no_mangle]
pub extern "C" fn rs_fold_context_at_line(line: LinenrT) -> FoldCmdContext {
    FoldCmdContext::at_line(line)
}

/// FFI export: Create fold context for range
#[no_mangle]
pub extern "C" fn rs_fold_context_for_range(line1: LinenrT, line2: LinenrT) -> FoldCmdContext {
    FoldCmdContext::for_range(line1, line2)
}

/// FFI export: Get context range size
#[no_mangle]
pub extern "C" fn rs_fold_context_range_size(ctx: *const FoldCmdContext) -> LinenrT {
    if ctx.is_null() {
        return 0;
    }
    unsafe { (*ctx).range_size() }
}

/// FFI export: Create fold command
#[no_mangle]
pub extern "C" fn rs_fold_create_cmd_new(start: LinenrT, end: LinenrT) -> CreateFoldCmd {
    CreateFoldCmd::new(start, end)
}

/// FFI export: Check if create fold is valid
#[no_mangle]
pub extern "C" fn rs_fold_create_cmd_is_valid(cmd: *const CreateFoldCmd) -> c_int {
    if cmd.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*cmd).is_valid() })
}

/// FFI export: Create navigation to next
#[no_mangle]
pub extern "C" fn rs_fold_nav_next(from: LinenrT) -> FoldNavCmd {
    FoldNavCmd::next(from)
}

/// FFI export: Create navigation to previous
#[no_mangle]
pub extern "C" fn rs_fold_nav_previous(from: LinenrT) -> FoldNavCmd {
    FoldNavCmd::previous(from)
}

/// FFI export: Create not-found nav result
#[no_mangle]
pub extern "C" fn rs_fold_nav_result_not_found() -> FoldNavResult {
    FoldNavResult::not_found()
}

/// FFI export: Create found nav result
#[no_mangle]
pub extern "C" fn rs_fold_nav_result_at(line: LinenrT) -> FoldNavResult {
    FoldNavResult::found_at(line)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_op() {
        assert!(FoldOp::Open.is_opening());
        assert!(!FoldOp::Open.is_closing());
        assert!(!FoldOp::Open.is_recursive());

        assert!(FoldOp::Close.is_closing());
        assert!(!FoldOp::Close.is_opening());

        assert!(FoldOp::OpenRecursive.is_opening());
        assert!(FoldOp::OpenRecursive.is_recursive());

        assert!(FoldOp::CloseRecursive.is_closing());
        assert!(FoldOp::CloseRecursive.is_recursive());
    }

    #[test]
    fn test_fold_cmd_result() {
        assert!(FoldCmdResult::Success.is_success());
        assert!(!FoldCmdResult::Success.is_error());

        assert!(!FoldCmdResult::NoFold.is_success());
        assert!(FoldCmdResult::NoFold.is_error());

        // AlreadyState is not an error
        assert!(!FoldCmdResult::AlreadyState.is_error());
    }

    #[test]
    fn test_fold_cmd_context() {
        let single = FoldCmdContext::at_line(10);
        assert!(single.is_single_line());
        assert_eq!(single.range_size(), 1);

        let range = FoldCmdContext::for_range(10, 20);
        assert!(!range.is_single_line());
        assert_eq!(range.range_size(), 11);

        let with_count = range.with_count(5);
        assert_eq!(with_count.count, 5);

        let recursive = range.with_recursive();
        assert!(recursive.recursive);
    }

    #[test]
    fn test_create_fold_cmd() {
        let cmd = CreateFoldCmd::new(10, 20);
        assert!(cmd.is_valid());
        assert_eq!(cmd.line_count(), 11);
        assert!(!cmd.use_markers);

        let with_markers = cmd.with_markers();
        assert!(with_markers.use_markers);

        let invalid = CreateFoldCmd::new(20, 10);
        assert!(!invalid.is_valid());
        assert_eq!(invalid.line_count(), 0);
    }

    #[test]
    fn test_delete_fold_cmd() {
        let cmd = DeleteFoldCmd::at_line(15);
        assert_eq!(cmd.line, 15);
        assert!(!cmd.recursive);
        assert!(!cmd.delete_markers);

        let recursive = cmd.with_recursive();
        assert!(recursive.recursive);

        let with_markers = cmd.with_markers();
        assert!(with_markers.delete_markers);
    }

    #[test]
    fn test_set_fold_level_cmd() {
        let global = SetFoldLevelCmd::global(3);
        assert_eq!(global.level, 3);
        assert!(global.is_valid_level());

        let at_line = SetFoldLevelCmd::at_line(2, 10);
        assert_eq!(at_line.line1, 10);
        assert_eq!(at_line.line2, 10);
    }

    #[test]
    fn test_fold_nav_cmd() {
        let next = FoldNavCmd::next(10);
        assert_eq!(next.direction, FoldNavDirection::Next);
        assert_eq!(next.from_line, 10);
        assert_eq!(next.count, 1);

        let prev = FoldNavCmd::previous(20).with_count(3);
        assert_eq!(prev.direction, FoldNavDirection::Previous);
        assert_eq!(prev.count, 3);
    }

    #[test]
    fn test_fold_nav_result() {
        let not_found = FoldNavResult::not_found();
        assert!(!not_found.found);
        assert_eq!(not_found.line, -1);

        let found = FoldNavResult::found_at(15);
        assert!(found.found);
        assert_eq!(found.line, 15);
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_fold_op_is_opening(FoldOp::Open), 1);
        assert_eq!(rs_fold_op_is_opening(FoldOp::Close), 0);

        assert_eq!(rs_fold_cmd_result_success(FoldCmdResult::Success), 1);
        assert_eq!(rs_fold_cmd_result_success(FoldCmdResult::Error), 0);
    }
}
