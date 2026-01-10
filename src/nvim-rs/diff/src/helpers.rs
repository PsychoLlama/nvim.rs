//! Diff helper functions for Neovim
//!
//! This module provides additional Rust helper functions for diff operations.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::manual_range_contains)]

use std::ffi::c_int;

// Line number type matching linenr_T (i32)
type LinenrT = i32;

// Result constants matching Neovim's OK/FAIL
const OK: c_int = 1;
const FAIL: c_int = 0;

// Diff flags (from diff.c) - must match exactly
const DIFF_FILLER: c_int = 0x001;
const DIFF_IBLANK: c_int = 0x002;
const DIFF_ICASE: c_int = 0x004;
const DIFF_IWHITE: c_int = 0x008;
const DIFF_IWHITEALL: c_int = 0x010;
const DIFF_IWHITEEOL: c_int = 0x020;
const DIFF_HORIZONTAL: c_int = 0x040;
const DIFF_VERTICAL: c_int = 0x080;
const DIFF_HIDDEN_OFF: c_int = 0x100;
const DIFF_INTERNAL: c_int = 0x200;
const DIFF_CLOSE_OFF: c_int = 0x400;
const DIFF_FOLLOWWRAP: c_int = 0x800;
const DIFF_LINEMATCH: c_int = 0x1000;
const DIFF_INLINE_NONE: c_int = 0x2000;
const DIFF_INLINE_SIMPLE: c_int = 0x4000;
const DIFF_INLINE_CHAR: c_int = 0x8000;
const DIFF_INLINE_WORD: c_int = 0x10000;
const DIFF_ANCHOR: c_int = 0x20000;

// Combination masks
const ALL_WHITE_DIFF: c_int = DIFF_IWHITE | DIFF_IWHITEALL | DIFF_IWHITEEOL;
const ALL_INLINE: c_int =
    DIFF_INLINE_NONE | DIFF_INLINE_SIMPLE | DIFF_INLINE_CHAR | DIFF_INLINE_WORD;

// DB_COUNT - maximum number of diff buffers
const DB_COUNT: c_int = 8;

// =============================================================================
// Flag Checking Helpers
// =============================================================================

/// Check if filler flag is set in given flags.
#[no_mangle]
pub extern "C" fn rs_diff_has_filler(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_FILLER) != 0)
}

/// Check if iblank flag is set in given flags.
#[no_mangle]
pub extern "C" fn rs_diff_has_iblank(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_IBLANK) != 0)
}

/// Check if icase flag is set in given flags.
#[no_mangle]
pub extern "C" fn rs_diff_has_icase(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_ICASE) != 0)
}

/// Check if iwhite flag is set in given flags.
#[no_mangle]
pub extern "C" fn rs_diff_has_iwhite(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_IWHITE) != 0)
}

/// Check if iwhiteall flag is set in given flags.
#[no_mangle]
pub extern "C" fn rs_diff_has_iwhiteall(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_IWHITEALL) != 0)
}

/// Check if iwhiteeol flag is set in given flags.
#[no_mangle]
pub extern "C" fn rs_diff_has_iwhiteeol(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_IWHITEEOL) != 0)
}

/// Check if any whitespace flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_any_white(flags: c_int) -> c_int {
    c_int::from((flags & ALL_WHITE_DIFF) != 0)
}

/// Check if horizontal flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_horizontal(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_HORIZONTAL) != 0)
}

/// Check if vertical flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_vertical(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_VERTICAL) != 0)
}

/// Check if hiddenoff flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_hiddenoff(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_HIDDEN_OFF) != 0)
}

/// Check if internal flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_internal(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_INTERNAL) != 0)
}

/// Check if closeoff flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_closeoff(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_CLOSE_OFF) != 0)
}

/// Check if followwrap flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_followwrap(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_FOLLOWWRAP) != 0)
}

/// Check if linematch flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_linematch(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_LINEMATCH) != 0)
}

/// Check if anchor flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_anchor(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_ANCHOR) != 0)
}

/// Check if any inline flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_any_inline(flags: c_int) -> c_int {
    c_int::from((flags & ALL_INLINE) != 0)
}

/// Check if inline:none flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_inline_none(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_INLINE_NONE) != 0)
}

/// Check if inline:simple flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_inline_simple(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_INLINE_SIMPLE) != 0)
}

/// Check if inline:char flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_inline_char(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_INLINE_CHAR) != 0)
}

/// Check if inline:word flag is set.
#[no_mangle]
pub extern "C" fn rs_diff_has_inline_word(flags: c_int) -> c_int {
    c_int::from((flags & DIFF_INLINE_WORD) != 0)
}

// =============================================================================
// Flag Constants
// =============================================================================

/// Get DIFF_FILLER constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_filler() -> c_int {
    DIFF_FILLER
}

/// Get DIFF_IBLANK constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_iblank() -> c_int {
    DIFF_IBLANK
}

/// Get DIFF_ICASE constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_icase() -> c_int {
    DIFF_ICASE
}

/// Get DIFF_IWHITE constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_iwhite() -> c_int {
    DIFF_IWHITE
}

/// Get DIFF_IWHITEALL constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_iwhiteall() -> c_int {
    DIFF_IWHITEALL
}

/// Get DIFF_IWHITEEOL constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_iwhiteeol() -> c_int {
    DIFF_IWHITEEOL
}

/// Get DIFF_HORIZONTAL constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_horizontal() -> c_int {
    DIFF_HORIZONTAL
}

/// Get DIFF_VERTICAL constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_vertical() -> c_int {
    DIFF_VERTICAL
}

/// Get DIFF_HIDDEN_OFF constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_hiddenoff() -> c_int {
    DIFF_HIDDEN_OFF
}

/// Get DIFF_INTERNAL constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_internal() -> c_int {
    DIFF_INTERNAL
}

/// Get DIFF_CLOSE_OFF constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_closeoff() -> c_int {
    DIFF_CLOSE_OFF
}

/// Get DIFF_FOLLOWWRAP constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_followwrap() -> c_int {
    DIFF_FOLLOWWRAP
}

/// Get DIFF_LINEMATCH constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_linematch() -> c_int {
    DIFF_LINEMATCH
}

/// Get DIFF_ANCHOR constant.
#[no_mangle]
pub extern "C" fn rs_diff_flag_anchor() -> c_int {
    DIFF_ANCHOR
}

/// Get ALL_WHITE_DIFF mask.
#[no_mangle]
pub extern "C" fn rs_diff_mask_all_white() -> c_int {
    ALL_WHITE_DIFF
}

/// Get ALL_INLINE mask.
#[no_mangle]
pub extern "C" fn rs_diff_mask_all_inline() -> c_int {
    ALL_INLINE
}

// =============================================================================
// Buffer Index Helpers
// =============================================================================

/// Get DB_COUNT constant (max diff buffers).
#[no_mangle]
pub extern "C" fn rs_diff_db_count() -> c_int {
    DB_COUNT
}

/// Check if buffer index is valid.
#[no_mangle]
pub extern "C" fn rs_diff_idx_valid(idx: c_int) -> c_int {
    c_int::from(idx >= 0 && idx < DB_COUNT)
}

/// Get next buffer index (wrapping).
#[no_mangle]
pub extern "C" fn rs_diff_idx_next(idx: c_int) -> c_int {
    (idx + 1) % DB_COUNT
}

/// Get previous buffer index (wrapping).
#[no_mangle]
pub extern "C" fn rs_diff_idx_prev(idx: c_int) -> c_int {
    if idx <= 0 {
        DB_COUNT - 1
    } else {
        idx - 1
    }
}

// =============================================================================
// Line Number Helpers
// =============================================================================

/// Check if a line number is valid (positive).
#[no_mangle]
pub extern "C" fn rs_diff_lnum_valid(lnum: LinenrT) -> c_int {
    c_int::from(lnum > 0)
}

/// Check if a line count is valid (non-negative).
#[no_mangle]
pub extern "C" fn rs_diff_count_valid(count: LinenrT) -> c_int {
    c_int::from(count >= 0)
}

/// Calculate end line from start and count.
#[no_mangle]
pub extern "C" fn rs_diff_block_end(lnum: LinenrT, count: LinenrT) -> LinenrT {
    if count <= 0 {
        lnum
    } else {
        lnum + count - 1
    }
}

/// Check if a line is within a block (lnum to lnum+count-1).
#[no_mangle]
pub extern "C" fn rs_diff_lnum_in_block(
    lnum: LinenrT,
    block_lnum: LinenrT,
    count: LinenrT,
) -> c_int {
    let end = if count <= 0 {
        block_lnum
    } else {
        block_lnum + count - 1
    };
    c_int::from(lnum >= block_lnum && lnum <= end)
}

/// Calculate offset between two block positions.
#[no_mangle]
pub extern "C" fn rs_diff_block_offset(
    lnum1: LinenrT,
    count1: LinenrT,
    lnum2: LinenrT,
    count2: LinenrT,
) -> LinenrT {
    (lnum1 + count1) - (lnum2 + count2)
}

// =============================================================================
// Diff Block Counting Helpers
// =============================================================================

/// Check if a diff block is empty (zero count).
#[no_mangle]
pub extern "C" fn rs_diff_block_is_empty(count: LinenrT) -> c_int {
    c_int::from(count == 0)
}

/// Check if a diff block is an insertion (count is zero in one buffer).
#[no_mangle]
pub extern "C" fn rs_diff_block_is_insert(count: LinenrT) -> c_int {
    c_int::from(count == 0)
}

/// Check if a diff block is a deletion (count > 0 but will be zero in target).
#[no_mangle]
pub extern "C" fn rs_diff_block_is_delete(count: LinenrT, target_count: LinenrT) -> c_int {
    c_int::from(count > 0 && target_count == 0)
}

/// Check if a diff block is a change (both counts > 0).
#[no_mangle]
pub extern "C" fn rs_diff_block_is_change(count1: LinenrT, count2: LinenrT) -> c_int {
    c_int::from(count1 > 0 && count2 > 0)
}

/// Calculate filler lines for a block (max of other counts - this count).
#[no_mangle]
pub extern "C" fn rs_diff_filler_count(this_count: LinenrT, max_other_count: LinenrT) -> LinenrT {
    if max_other_count > this_count {
        max_other_count - this_count
    } else {
        0
    }
}

// =============================================================================
// Diff Result Helpers
// =============================================================================

/// Return OK constant.
#[no_mangle]
pub extern "C" fn rs_diff_ok() -> c_int {
    OK
}

/// Return FAIL constant.
#[no_mangle]
pub extern "C" fn rs_diff_fail() -> c_int {
    FAIL
}

/// Check if result is OK.
#[no_mangle]
pub extern "C" fn rs_diff_is_ok(result: c_int) -> c_int {
    c_int::from(result == OK)
}

/// Check if result is FAIL.
#[no_mangle]
pub extern "C" fn rs_diff_is_fail(result: c_int) -> c_int {
    c_int::from(result == FAIL)
}

// =============================================================================
// Context Line Helpers
// =============================================================================

/// Default diff context lines.
const DEFAULT_CONTEXT: c_int = 6;

/// Get default context lines.
#[no_mangle]
pub extern "C" fn rs_diff_default_context() -> c_int {
    DEFAULT_CONTEXT
}

/// Ensure context is at least 1 (0 becomes 1).
#[no_mangle]
pub extern "C" fn rs_diff_context_min1(context: c_int) -> c_int {
    if context <= 0 {
        1
    } else {
        context
    }
}

/// Check if a line is within context of a block.
#[no_mangle]
pub extern "C" fn rs_diff_lnum_in_context(
    lnum: LinenrT,
    block_lnum: LinenrT,
    count: LinenrT,
    context: c_int,
) -> c_int {
    let start = block_lnum - context;
    let end = if count <= 0 {
        block_lnum + context
    } else {
        block_lnum + count - 1 + context
    };
    c_int::from(lnum >= start && lnum <= end)
}

// =============================================================================
// Fold Column Helpers
// =============================================================================

/// Default fold column width.
const DEFAULT_FOLDCOLUMN: c_int = 2;

/// Get default fold column width.
#[no_mangle]
pub extern "C" fn rs_diff_default_foldcolumn() -> c_int {
    DEFAULT_FOLDCOLUMN
}

/// Clamp fold column to valid range (0-12).
#[no_mangle]
pub extern "C" fn rs_diff_foldcolumn_clamp(foldcol: c_int) -> c_int {
    if foldcol < 0 {
        0
    } else if foldcol > 12 {
        12
    } else {
        foldcol
    }
}

// =============================================================================
// Linematch Helpers
// =============================================================================

/// Default linematch lines.
const DEFAULT_LINEMATCH: c_int = 40;

/// Get default linematch lines.
#[no_mangle]
pub extern "C" fn rs_diff_default_linematch() -> c_int {
    DEFAULT_LINEMATCH
}

/// Check if linematch is enabled (lines > 0).
#[no_mangle]
pub extern "C" fn rs_diff_linematch_enabled(lines: c_int) -> c_int {
    c_int::from(lines > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_checking() {
        let flags = DIFF_FILLER | DIFF_ICASE | DIFF_INTERNAL;
        assert_eq!(rs_diff_has_filler(flags), 1);
        assert_eq!(rs_diff_has_icase(flags), 1);
        assert_eq!(rs_diff_has_internal(flags), 1);
        assert_eq!(rs_diff_has_iblank(flags), 0);
        assert_eq!(rs_diff_has_horizontal(flags), 0);
    }

    #[test]
    fn test_whitespace_flags() {
        let flags = DIFF_IWHITE | DIFF_IWHITEEOL;
        assert_eq!(rs_diff_has_any_white(flags), 1);
        assert_eq!(rs_diff_has_iwhite(flags), 1);
        assert_eq!(rs_diff_has_iwhiteeol(flags), 1);
        assert_eq!(rs_diff_has_iwhiteall(flags), 0);
    }

    #[test]
    fn test_idx_helpers() {
        assert_eq!(rs_diff_db_count(), 8);
        assert_eq!(rs_diff_idx_valid(0), 1);
        assert_eq!(rs_diff_idx_valid(7), 1);
        assert_eq!(rs_diff_idx_valid(8), 0);
        assert_eq!(rs_diff_idx_valid(-1), 0);

        assert_eq!(rs_diff_idx_next(0), 1);
        assert_eq!(rs_diff_idx_next(7), 0);
        assert_eq!(rs_diff_idx_prev(0), 7);
        assert_eq!(rs_diff_idx_prev(3), 2);
    }

    #[test]
    fn test_block_helpers() {
        assert_eq!(rs_diff_block_end(10, 5), 14);
        assert_eq!(rs_diff_block_end(10, 0), 10);
        assert_eq!(rs_diff_block_end(10, 1), 10);

        assert_eq!(rs_diff_lnum_in_block(12, 10, 5), 1);
        assert_eq!(rs_diff_lnum_in_block(9, 10, 5), 0);
        assert_eq!(rs_diff_lnum_in_block(15, 10, 5), 0);
    }

    #[test]
    fn test_block_type_helpers() {
        assert_eq!(rs_diff_block_is_empty(0), 1);
        assert_eq!(rs_diff_block_is_empty(5), 0);

        assert_eq!(rs_diff_block_is_insert(0), 1);
        assert_eq!(rs_diff_block_is_delete(5, 0), 1);
        assert_eq!(rs_diff_block_is_change(3, 5), 1);
    }

    #[test]
    fn test_filler_count() {
        assert_eq!(rs_diff_filler_count(0, 5), 5);
        assert_eq!(rs_diff_filler_count(3, 5), 2);
        assert_eq!(rs_diff_filler_count(5, 3), 0);
        assert_eq!(rs_diff_filler_count(5, 5), 0);
    }

    #[test]
    fn test_context_helpers() {
        assert_eq!(rs_diff_default_context(), 6);
        assert_eq!(rs_diff_context_min1(0), 1);
        assert_eq!(rs_diff_context_min1(3), 3);

        // Line 8 should be in context of block at 10 with context 3
        assert_eq!(rs_diff_lnum_in_context(8, 10, 3, 3), 1);
        // Line 5 should not be in context of block at 10 with context 3
        assert_eq!(rs_diff_lnum_in_context(5, 10, 3, 3), 0);
    }

    #[test]
    fn test_foldcolumn_clamp() {
        assert_eq!(rs_diff_foldcolumn_clamp(-1), 0);
        assert_eq!(rs_diff_foldcolumn_clamp(0), 0);
        assert_eq!(rs_diff_foldcolumn_clamp(5), 5);
        assert_eq!(rs_diff_foldcolumn_clamp(12), 12);
        assert_eq!(rs_diff_foldcolumn_clamp(20), 12);
    }
}
