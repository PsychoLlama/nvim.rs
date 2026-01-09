//! Diff highlighting logic
//!
//! This module provides Rust implementations for diff highlighting,
//! including inline change detection, diff line status, and change parsing.

#![allow(clippy::must_use_candidate)]

use std::ffi::c_char;
use std::os::raw::c_int;

use crate::buffer::{BufHandle, DiffBlockHandle, DB_COUNT};

/// Line number type matching linenr_T (i32).
type LinenrT = i32;

/// Column number type.
type ColnrT = i32;

// =============================================================================
// Diff Flags (must match C definitions)
// =============================================================================

#[allow(dead_code)]
const DIFF_ICASE: c_int = 0x004;
#[allow(dead_code)]
const DIFF_IWHITE: c_int = 0x008;
#[allow(dead_code)]
const DIFF_IWHITEALL: c_int = 0x010;
#[allow(dead_code)]
const DIFF_IWHITEEOL: c_int = 0x020;
const DIFF_INLINE_NONE: c_int = 0x2000;
const DIFF_INLINE_SIMPLE: c_int = 0x4000;
const DIFF_INLINE_CHAR: c_int = 0x8000;
const DIFF_INLINE_WORD: c_int = 0x10000;

const ALL_INLINE: c_int =
    DIFF_INLINE_NONE | DIFF_INLINE_SIMPLE | DIFF_INLINE_CHAR | DIFF_INLINE_WORD;
const ALL_INLINE_DIFF: c_int = DIFF_INLINE_CHAR | DIFF_INLINE_WORD;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_diff_flags() -> c_int;
    fn nvim_get_curtab_diffbuf(idx: c_int) -> BufHandle;
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;

    // UTF-8 functions
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_fold(c: c_int) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
}

// =============================================================================
// Inline Highlight Mode
// =============================================================================

/// Inline highlight mode enumeration.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffInlineMode {
    /// No inline highlighting.
    None = 0,
    /// Simple inline highlighting (just highlight changed region).
    Simple = 1,
    /// Character-level diff.
    Char = 2,
    /// Word-level diff.
    Word = 3,
}

impl DiffInlineMode {
    /// Get the current inline mode from diff flags.
    #[must_use]
    pub fn current() -> Self {
        unsafe {
            let flags = nvim_get_diff_flags();
            if (flags & DIFF_INLINE_WORD) != 0 {
                Self::Word
            } else if (flags & DIFF_INLINE_CHAR) != 0 {
                Self::Char
            } else if (flags & DIFF_INLINE_SIMPLE) != 0 {
                Self::Simple
            } else if (flags & DIFF_INLINE_NONE) != 0 {
                Self::None
            } else {
                // Default to Char if no inline mode is explicitly set
                Self::Char
            }
        }
    }

    /// Check if any inline mode is active.
    #[must_use]
    pub fn is_any_active() -> bool {
        unsafe { (nvim_get_diff_flags() & ALL_INLINE) != 0 }
    }

    /// Check if actual inline diff computation is enabled (char or word mode).
    #[must_use]
    pub fn is_diff_active() -> bool {
        unsafe { (nvim_get_diff_flags() & ALL_INLINE_DIFF) != 0 }
    }
}

// =============================================================================
// Diff Line Status
// =============================================================================

/// Line status in diff mode.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineStatus {
    /// Line is not in a diff block.
    NotInDiff = 0,
    /// Line is added (only in this buffer).
    Added = 1,
    /// Line is changed (differs from other buffers).
    Changed = 2,
    /// Line is equal in all buffers.
    Equal = 3,
}

/// Diff line information structure.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DiffLineInfo {
    /// Buffer index in diff list.
    pub buf_idx: c_int,
    /// Line number in the buffer.
    pub lnum: LinenrT,
    /// Offset within the diff block.
    pub line_offset: c_int,
    /// Line status.
    pub status: DiffLineStatus,
    /// Diff block handle (null if not in diff).
    pub block: DiffBlockHandle,
}

impl DiffLineInfo {
    /// Create info for a line not in diff.
    #[must_use]
    pub const fn not_in_diff() -> Self {
        Self {
            buf_idx: -1,
            lnum: 0,
            line_offset: 0,
            status: DiffLineStatus::NotInDiff,
            block: DiffBlockHandle::null(),
        }
    }
}

// =============================================================================
// Inline Change Tracking
// =============================================================================

/// Change positions within a diff line.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DiffLineChange {
    /// Start byte positions for each buffer.
    pub start: [ColnrT; DB_COUNT as usize],
    /// End byte positions for each buffer.
    pub end: [ColnrT; DB_COUNT as usize],
    /// Start line offset for each buffer.
    pub start_lnum_off: [c_int; DB_COUNT as usize],
    /// End line offset for each buffer.
    pub end_lnum_off: [c_int; DB_COUNT as usize],
}

impl DiffLineChange {
    /// Create an empty change with all zeros.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            start: [0; DB_COUNT as usize],
            end: [0; DB_COUNT as usize],
            start_lnum_off: [0; DB_COUNT as usize],
            end_lnum_off: [0; DB_COUNT as usize],
        }
    }

    /// Create a change covering entire lines.
    #[must_use]
    pub const fn full_line() -> Self {
        Self {
            start: [0; DB_COUNT as usize],
            end: [i32::MAX; DB_COUNT as usize],
            start_lnum_off: [0; DB_COUNT as usize],
            end_lnum_off: [0; DB_COUNT as usize],
        }
    }
}

/// Result of parsing a diff change.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DiffChangeResult {
    /// Start byte offset of the change.
    pub start: c_int,
    /// End byte offset of the change.
    pub end: c_int,
    /// Whether this is an addition (line only exists in this buffer).
    pub is_added: c_int,
}

impl DiffChangeResult {
    /// Create a result for no change.
    #[must_use]
    pub const fn no_change() -> Self {
        Self {
            start: 0,
            end: 0,
            is_added: 0,
        }
    }
}

// =============================================================================
// Diff String Comparison
// =============================================================================

/// Check if character is ASCII whitespace (space or tab).
#[inline]
pub const fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Skip leading whitespace in a string.
///
/// Returns the number of bytes skipped.
#[inline]
pub fn skip_whitespace(s: &[u8]) -> usize {
    s.iter().take_while(|&&c| ascii_iswhite(c)).count()
}

/// Compare two characters for equality, possibly ignoring case.
///
/// If characters are equal (possibly after case folding), returns the byte
/// length of the character. Otherwise returns 0.
///
/// # Safety
/// Pointers must be valid null-terminated strings.
#[allow(clippy::cast_sign_loss)]
pub unsafe fn diff_equal_char(p1: *const c_char, p2: *const c_char, diff_flags: c_int) -> c_int {
    let l = utfc_ptr2len(p1);

    // Characters must have the same byte length
    if l != utfc_ptr2len(p2) {
        return 0;
    }

    if l > 1 {
        // Multibyte character: compare bytes first
        if libc::strncmp(p1, p2, l as usize) != 0 {
            // Bytes differ, check if case-insensitive comparison matches
            if (diff_flags & DIFF_ICASE) == 0 {
                return 0;
            }
            // Compare case-folded characters
            if utf_fold(utf_ptr2char(p1)) != utf_fold(utf_ptr2char(p2)) {
                return 0;
            }
        }
    } else {
        // Single-byte character
        let c1 = *p1 as u8;
        let c2 = *p2 as u8;
        if c1 != c2 {
            if (diff_flags & DIFF_ICASE) == 0 {
                return 0;
            }
            // Compare lowercase versions
            let l1 = if c1.is_ascii_uppercase() {
                c1 + (b'a' - b'A')
            } else {
                c1
            };
            let l2 = if c2.is_ascii_uppercase() {
                c2 + (b'a' - b'A')
            } else {
                c2
            };
            if l1 != l2 {
                return 0;
            }
        }
    }

    l
}

// =============================================================================
// Diff Filler Lines
// =============================================================================

/// Calculate the number of filler lines for a diff block.
///
/// Filler lines are displayed to align buffers in diff mode.
pub fn diff_calc_filler(dp: DiffBlockHandle, buf_idx: c_int) -> c_int {
    if dp.is_null() || !(0..DB_COUNT).contains(&buf_idx) {
        return 0;
    }

    unsafe {
        let count = nvim_diffblock_get_count(dp, buf_idx);
        if count > 0 {
            return 0; // No filler lines if we have actual lines
        }

        // Find the maximum line count in other buffers
        let mut max_count = 0;
        for i in 0..DB_COUNT {
            if i != buf_idx && !nvim_get_curtab_diffbuf(i).is_null() {
                let other_count = nvim_diffblock_get_count(dp, i);
                max_count = max_count.max(other_count);
            }
        }
        max_count
    }
}

/// Check if a line is a filler line (virtual line for alignment).
pub fn diff_is_filler_line(dp: DiffBlockHandle, buf_idx: c_int, lnum: LinenrT) -> bool {
    if dp.is_null() || !(0..DB_COUNT).contains(&buf_idx) {
        return false;
    }

    unsafe {
        let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
        let count = nvim_diffblock_get_count(dp, buf_idx);

        // Filler lines are at the start of a block with count == 0
        count == 0 && lnum == block_lnum
    }
}

// =============================================================================
// Highlight Group IDs
// =============================================================================

/// Diff highlight group IDs.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffHighlightGroup {
    /// DiffAdd - added lines.
    Add = 0,
    /// DiffChange - changed lines.
    Change = 1,
    /// DiffDelete - deleted lines.
    Delete = 2,
    /// DiffText - changed text within a line.
    Text = 3,
}

/// Get the appropriate highlight group for a diff line.
pub const fn diff_get_highlight_group(
    status: DiffLineStatus,
    has_inline_change: bool,
) -> DiffHighlightGroup {
    match status {
        DiffLineStatus::Added => DiffHighlightGroup::Add,
        DiffLineStatus::Changed => {
            if has_inline_change {
                DiffHighlightGroup::Text
            } else {
                DiffHighlightGroup::Change
            }
        }
        DiffLineStatus::Equal | DiffLineStatus::NotInDiff => DiffHighlightGroup::Change,
    }
}

// FFI exports removed - the main ones are in lib.rs to maintain existing API compatibility.
// Additional exports that aren't in lib.rs:

/// FFI export: Calculate filler lines for a diff block.
#[no_mangle]
pub extern "C" fn rs_diff_calc_filler(dp: DiffBlockHandle, buf_idx: c_int) -> c_int {
    diff_calc_filler(dp, buf_idx)
}

/// FFI export: Check if line is a filler line.
#[no_mangle]
pub extern "C" fn rs_diff_is_filler_line(
    dp: DiffBlockHandle,
    buf_idx: c_int,
    lnum: LinenrT,
) -> c_int {
    c_int::from(diff_is_filler_line(dp, buf_idx, lnum))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_inline_mode_values() {
        assert_eq!(DiffInlineMode::None as c_int, 0);
        assert_eq!(DiffInlineMode::Simple as c_int, 1);
        assert_eq!(DiffInlineMode::Char as c_int, 2);
        assert_eq!(DiffInlineMode::Word as c_int, 3);
    }

    #[test]
    fn test_diff_line_status_values() {
        assert_eq!(DiffLineStatus::NotInDiff as c_int, 0);
        assert_eq!(DiffLineStatus::Added as c_int, 1);
        assert_eq!(DiffLineStatus::Changed as c_int, 2);
        assert_eq!(DiffLineStatus::Equal as c_int, 3);
    }

    #[test]
    fn test_diff_line_info_not_in_diff() {
        let info = DiffLineInfo::not_in_diff();
        assert_eq!(info.buf_idx, -1);
        assert_eq!(info.lnum, 0);
        assert_eq!(info.status, DiffLineStatus::NotInDiff);
        assert!(info.block.is_null());
    }

    #[test]
    fn test_diff_line_change_empty() {
        let change = DiffLineChange::empty();
        for i in 0..DB_COUNT as usize {
            assert_eq!(change.start[i], 0);
            assert_eq!(change.end[i], 0);
        }
    }

    #[test]
    fn test_diff_line_change_full_line() {
        let change = DiffLineChange::full_line();
        for i in 0..DB_COUNT as usize {
            assert_eq!(change.start[i], 0);
            assert_eq!(change.end[i], i32::MAX);
        }
    }

    #[test]
    fn test_ascii_iswhite() {
        assert!(ascii_iswhite(b' '));
        assert!(ascii_iswhite(b'\t'));
        assert!(!ascii_iswhite(b'a'));
        assert!(!ascii_iswhite(b'\n'));
        assert!(!ascii_iswhite(0));
    }

    #[test]
    fn test_skip_whitespace() {
        assert_eq!(skip_whitespace(b"  hello"), 2);
        assert_eq!(skip_whitespace(b"\t\thello"), 2);
        assert_eq!(skip_whitespace(b"hello"), 0);
        assert_eq!(skip_whitespace(b""), 0);
        assert_eq!(skip_whitespace(b"   "), 3);
    }

    #[test]
    fn test_diff_highlight_group() {
        assert_eq!(
            diff_get_highlight_group(DiffLineStatus::Added, false),
            DiffHighlightGroup::Add
        );
        assert_eq!(
            diff_get_highlight_group(DiffLineStatus::Changed, false),
            DiffHighlightGroup::Change
        );
        assert_eq!(
            diff_get_highlight_group(DiffLineStatus::Changed, true),
            DiffHighlightGroup::Text
        );
    }

    #[test]
    fn test_struct_sizes() {
        use std::mem::size_of;

        // DiffLineChange should be 8 arrays * DB_COUNT * 4 bytes
        assert_eq!(size_of::<DiffLineChange>(), 4 * DB_COUNT as usize * 4);

        // DiffChangeResult should be 3 * 4 = 12 bytes
        assert_eq!(size_of::<DiffChangeResult>(), 12);
    }
}
