//! Fold method implementations
//!
//! This module provides Rust implementations for different fold methods:
//! manual, indent, expr, syntax, diff, and marker.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::derivable_impls)]

use std::ffi::c_int;

// =============================================================================
// Fold Method Constants
// =============================================================================

/// Fold method: manual folding
pub const FDM_MANUAL: c_int = 0;
/// Fold method: fold by indent
pub const FDM_INDENT: c_int = 1;
/// Fold method: fold by expression
pub const FDM_EXPR: c_int = 2;
/// Fold method: fold by markers
pub const FDM_MARKER: c_int = 3;
/// Fold method: fold by syntax highlighting
pub const FDM_SYNTAX: c_int = 4;
/// Fold method: fold by diff
pub const FDM_DIFF: c_int = 5;

/// Minimum fold method value
pub const FDM_MIN: c_int = FDM_MANUAL;
/// Maximum fold method value
pub const FDM_MAX: c_int = FDM_DIFF;

// =============================================================================
// Fold Method Type
// =============================================================================

/// Fold method enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldMethod {
    /// Manual folding - user creates folds manually
    Manual = 0,
    /// Indent folding - folds based on indentation level
    Indent = 1,
    /// Expression folding - folds based on `foldexpr` option
    Expr = 2,
    /// Marker folding - folds based on markers like `{{{`
    Marker = 3,
    /// Syntax folding - folds based on syntax highlighting
    Syntax = 4,
    /// Diff folding - folds unchanged text in diff mode
    Diff = 5,
}

impl FoldMethod {
    /// Create from raw integer value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            FDM_MANUAL => Some(Self::Manual),
            FDM_INDENT => Some(Self::Indent),
            FDM_EXPR => Some(Self::Expr),
            FDM_MARKER => Some(Self::Marker),
            FDM_SYNTAX => Some(Self::Syntax),
            FDM_DIFF => Some(Self::Diff),
            _ => None,
        }
    }

    /// Convert to raw integer value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this method requires manual fold creation
    pub const fn is_manual(self) -> bool {
        matches!(self, Self::Manual)
    }

    /// Check if this method uses automatic fold detection
    pub const fn is_automatic(self) -> bool {
        !self.is_manual()
    }

    /// Check if this method is based on syntax
    pub const fn is_syntax_based(self) -> bool {
        matches!(self, Self::Syntax | Self::Expr)
    }

    /// Check if this method uses markers
    pub const fn is_marker_based(self) -> bool {
        matches!(self, Self::Marker)
    }

    /// Check if this method uses indentation
    pub const fn is_indent_based(self) -> bool {
        matches!(self, Self::Indent)
    }

    /// Check if this method is diff-based
    pub const fn is_diff_based(self) -> bool {
        matches!(self, Self::Diff)
    }

    /// Get a short name for the method
    pub const fn short_name(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Indent => "indent",
            Self::Expr => "expr",
            Self::Marker => "marker",
            Self::Syntax => "syntax",
            Self::Diff => "diff",
        }
    }
}

impl Default for FoldMethod {
    fn default() -> Self {
        Self::Manual
    }
}

// =============================================================================
// Fold Level Calculation
// =============================================================================

/// Result of fold level calculation
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldLevelResult {
    /// The calculated fold level
    pub level: c_int,
    /// Whether this line starts a fold
    pub start: bool,
    /// Whether this line ends a fold
    pub end: bool,
    /// Special flags (for expr method)
    pub flags: c_int,
}

impl Default for FoldLevelResult {
    fn default() -> Self {
        Self {
            level: 0,
            start: false,
            end: false,
            flags: 0,
        }
    }
}

impl FoldLevelResult {
    /// Create with just a level
    pub const fn with_level(level: c_int) -> Self {
        Self {
            level,
            start: false,
            end: false,
            flags: 0,
        }
    }

    /// Create with level and start flag
    pub const fn with_start(level: c_int) -> Self {
        Self {
            level,
            start: true,
            end: false,
            flags: 0,
        }
    }

    /// Create with level and end flag
    pub const fn with_end(level: c_int) -> Self {
        Self {
            level,
            start: false,
            end: true,
            flags: 0,
        }
    }
}

// =============================================================================
// Indent Level Calculation
// =============================================================================

/// Calculate fold level based on indentation
pub fn calculate_indent_level(indent: c_int, shiftwidth: c_int) -> c_int {
    if shiftwidth <= 0 {
        return 0;
    }
    indent / shiftwidth
}

/// Check if a line's indent indicates a new fold
pub fn indent_starts_fold(prev_indent: c_int, curr_indent: c_int, shiftwidth: c_int) -> bool {
    if shiftwidth <= 0 {
        return false;
    }
    let prev_level = prev_indent / shiftwidth;
    let curr_level = curr_indent / shiftwidth;
    curr_level > prev_level
}

/// Check if a line's indent indicates end of fold
pub fn indent_ends_fold(curr_indent: c_int, next_indent: c_int, shiftwidth: c_int) -> bool {
    if shiftwidth <= 0 {
        return false;
    }
    let curr_level = curr_indent / shiftwidth;
    let next_level = next_indent / shiftwidth;
    next_level < curr_level
}

// =============================================================================
// Expr Fold Level Flags
// =============================================================================

/// Flags for expression fold level results
pub const FOLD_LEVEL_START: c_int = 0x100;
pub const FOLD_LEVEL_END: c_int = 0x200;
pub const FOLD_LEVEL_UNDEFINED: c_int = -1;

/// Parse expr fold level result
pub fn parse_expr_level(value: c_int) -> FoldLevelResult {
    if value == FOLD_LEVEL_UNDEFINED {
        return FoldLevelResult {
            level: -1,
            start: false,
            end: false,
            flags: FOLD_LEVEL_UNDEFINED,
        };
    }

    let start = (value & FOLD_LEVEL_START) != 0;
    let end = (value & FOLD_LEVEL_END) != 0;
    let level = value & 0xFF;

    FoldLevelResult {
        level,
        start,
        end,
        flags: value,
    }
}

// =============================================================================
// Method Comparison
// =============================================================================

/// Check if fold method requires recalculation on change
pub const fn method_needs_update_on_change(method: c_int) -> bool {
    // All automatic methods need update when text changes
    method != FDM_MANUAL
}

/// Check if fold method supports nested folds
pub const fn method_supports_nesting(method: c_int) -> bool {
    // All methods support nesting except diff
    method != FDM_DIFF
}

/// Check if fold method needs syntax info
pub const fn method_needs_syntax(method: c_int) -> bool {
    method == FDM_SYNTAX
}

/// Check if fold method needs buffer text
pub const fn method_needs_text(method: c_int) -> bool {
    method == FDM_INDENT || method == FDM_MARKER || method == FDM_EXPR
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if fold method is valid
#[no_mangle]
pub extern "C" fn rs_fold_method_valid(method: c_int) -> c_int {
    c_int::from(FoldMethod::from_raw(method).is_some())
}

/// FFI export: Check if method is manual
#[no_mangle]
pub extern "C" fn rs_fold_method_is_manual(method: c_int) -> c_int {
    c_int::from(method == FDM_MANUAL)
}

/// FFI export: Check if method is automatic
#[no_mangle]
pub extern "C" fn rs_fold_method_is_automatic(method: c_int) -> c_int {
    c_int::from(method != FDM_MANUAL)
}

/// FFI export: Calculate indent level
#[no_mangle]
pub extern "C" fn rs_fold_calculate_indent_level(indent: c_int, shiftwidth: c_int) -> c_int {
    calculate_indent_level(indent, shiftwidth)
}

/// FFI export: Check if method needs update on change
#[no_mangle]
pub extern "C" fn rs_fold_method_needs_update(method: c_int) -> c_int {
    c_int::from(method_needs_update_on_change(method))
}

/// FFI export: Check if method supports nesting
#[no_mangle]
pub extern "C" fn rs_fold_method_supports_nesting(method: c_int) -> c_int {
    c_int::from(method_supports_nesting(method))
}

/// FFI export: Parse expr level
#[no_mangle]
pub extern "C" fn rs_fold_parse_expr_level(value: c_int) -> FoldLevelResult {
    parse_expr_level(value)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_method_from_raw() {
        assert_eq!(FoldMethod::from_raw(0), Some(FoldMethod::Manual));
        assert_eq!(FoldMethod::from_raw(1), Some(FoldMethod::Indent));
        assert_eq!(FoldMethod::from_raw(5), Some(FoldMethod::Diff));
        assert_eq!(FoldMethod::from_raw(6), None);
        assert_eq!(FoldMethod::from_raw(-1), None);
    }

    #[test]
    fn test_fold_method_properties() {
        assert!(FoldMethod::Manual.is_manual());
        assert!(!FoldMethod::Manual.is_automatic());

        assert!(!FoldMethod::Indent.is_manual());
        assert!(FoldMethod::Indent.is_automatic());
        assert!(FoldMethod::Indent.is_indent_based());

        assert!(FoldMethod::Marker.is_marker_based());
        assert!(FoldMethod::Syntax.is_syntax_based());
        assert!(FoldMethod::Diff.is_diff_based());
    }

    #[test]
    fn test_calculate_indent_level() {
        assert_eq!(calculate_indent_level(0, 4), 0);
        assert_eq!(calculate_indent_level(4, 4), 1);
        assert_eq!(calculate_indent_level(8, 4), 2);
        assert_eq!(calculate_indent_level(6, 4), 1); // Floor division
        assert_eq!(calculate_indent_level(10, 0), 0); // Zero shiftwidth
    }

    #[test]
    fn test_indent_starts_fold() {
        assert!(indent_starts_fold(0, 4, 4));
        assert!(indent_starts_fold(4, 8, 4));
        assert!(!indent_starts_fold(4, 4, 4));
        assert!(!indent_starts_fold(8, 4, 4));
    }

    #[test]
    fn test_indent_ends_fold() {
        assert!(indent_ends_fold(8, 4, 4));
        assert!(indent_ends_fold(4, 0, 4));
        assert!(!indent_ends_fold(4, 4, 4));
        assert!(!indent_ends_fold(4, 8, 4));
    }

    #[test]
    fn test_parse_expr_level() {
        let result = parse_expr_level(3);
        assert_eq!(result.level, 3);
        assert!(!result.start);
        assert!(!result.end);

        let result = parse_expr_level(2 | FOLD_LEVEL_START);
        assert_eq!(result.level, 2);
        assert!(result.start);
        assert!(!result.end);

        let result = parse_expr_level(FOLD_LEVEL_UNDEFINED);
        assert_eq!(result.level, -1);
    }

    #[test]
    fn test_method_properties() {
        assert!(!method_needs_update_on_change(FDM_MANUAL));
        assert!(method_needs_update_on_change(FDM_INDENT));

        assert!(method_supports_nesting(FDM_MANUAL));
        assert!(method_supports_nesting(FDM_INDENT));
        assert!(!method_supports_nesting(FDM_DIFF));

        assert!(method_needs_syntax(FDM_SYNTAX));
        assert!(!method_needs_syntax(FDM_INDENT));

        assert!(method_needs_text(FDM_INDENT));
        assert!(method_needs_text(FDM_MARKER));
        assert!(!method_needs_text(FDM_SYNTAX));
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_fold_method_valid(FDM_MANUAL), 1);
        assert_eq!(rs_fold_method_valid(100), 0);

        assert_eq!(rs_fold_method_is_manual(FDM_MANUAL), 1);
        assert_eq!(rs_fold_method_is_manual(FDM_INDENT), 0);

        assert_eq!(rs_fold_method_is_automatic(FDM_INDENT), 1);
        assert_eq!(rs_fold_method_is_automatic(FDM_MANUAL), 0);
    }
}
