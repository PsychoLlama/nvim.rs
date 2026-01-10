//! Diff mode support for Neovim
//!
//! This crate provides Rust implementations for diff mode functionality,
//! including option parsing, buffer management, highlighting, and navigation.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]

// Submodules
pub mod buffer;
pub mod commands;
pub mod helpers;
pub mod highlight;
pub mod navigate;

// Re-export key types from submodules
pub use buffer::{BufHandle, DiffBlockHandle, TabpageHandle, WinHandle, DB_COUNT};
pub use commands::{DiffBlockInfo, DiffOpResult, DiffOperation, DiffRange};
pub use highlight::{
    DiffChangeResult, DiffHighlightGroup, DiffInlineMode, DiffLineChange, DiffLineInfo,
    DiffLineStatus,
};
pub use navigate::{DiffHunkBounds, DiffNavResult, Direction};

use std::ffi::c_char;
use std::os::raw::c_int;

// Result constants matching Neovim's OK/FAIL
const OK: c_int = 1;
const FAIL: c_int = 0;

// Line number type matching linenr_T (i32)
type LinenrT = i32;

// Diff flags (from diff.c)
// These must match the C #define values exactly
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

// Extended diff flags (inline highlighting and anchors)
const DIFF_INLINE_NONE: c_int = 0x2000;
const DIFF_INLINE_SIMPLE: c_int = 0x4000;
const DIFF_INLINE_CHAR: c_int = 0x8000;
const DIFF_INLINE_WORD: c_int = 0x10000;
const DIFF_ANCHOR: c_int = 0x20000;

// Combination masks for inline highlighting modes
const ALL_INLINE: c_int =
    DIFF_INLINE_NONE | DIFF_INLINE_SIMPLE | DIFF_INLINE_CHAR | DIFF_INLINE_WORD;
const ALL_INLINE_DIFF: c_int = DIFF_INLINE_CHAR | DIFF_INLINE_WORD;

// Combination mask for all whitespace diff flags
const ALL_WHITE_DIFF: c_int = DIFF_IWHITE | DIFF_IWHITEALL | DIFF_IWHITEEOL;

// XDiff algorithm flags (from xdiff.h)
const XDF_NEED_MINIMAL: c_int = 1 << 0;
const XDF_PATIENCE_DIFF: c_int = 1 << 14;
const XDF_HISTOGRAM_DIFF: c_int = 1 << 15;
const XDF_INDENT_HEURISTIC: c_int = 1 << 23;

use std::ffi::c_void;

// Use opaque pointers for FFI to avoid type conflicts with buffer module
type DiffBlockPtr = *mut c_void;
type BufPtr = *mut c_void;

// C accessor for the static diff_flags variable
extern "C" {
    fn nvim_get_diff_flags() -> c_int;
    fn nvim_is_diffexpr_empty() -> bool;
    fn nvim_get_curtab_diffbuf(idx: c_int) -> BufPtr;
    fn nvim_get_curtab_diff_invalid() -> c_int;
    fn nvim_get_diff_first_block() -> DiffBlockPtr;
    fn nvim_diffblock_get_next(dp: DiffBlockPtr) -> DiffBlockPtr;
    fn nvim_diffblock_get_lnum(dp: DiffBlockPtr, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockPtr, idx: c_int) -> LinenrT;

    // UTF-8 functions for diff_cmp
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_fold(c: c_int) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;

    // Multibyte string comparison (case-insensitive)
    fn mb_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // Diff block setters for diff_copy_entry
    fn nvim_diffblock_set_lnum(dp: DiffBlockPtr, idx: c_int, lnum: LinenrT);
    fn nvim_diffblock_set_count(dp: DiffBlockPtr, idx: c_int, count: LinenrT);
}

/// Check if 'diffopt' contains "horizontal".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_horizontal() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_HORIZONTAL) != 0)
}

/// Check if 'diffopt' contains "hiddenoff".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_hiddenoff() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_HIDDEN_OFF) != 0)
}

/// Check if 'diffopt' contains "closeoff".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_closeoff() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_CLOSE_OFF) != 0)
}

/// Check if 'diffopt' contains "filler".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_filler() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_FILLER) != 0)
}

/// Return true if the options are set to use the internal diff library.
///
/// Note that if the internal diff failed for one of the buffers, the external
/// diff will be used anyway.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_internal() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_INTERNAL) != 0 && nvim_is_diffexpr_empty())
}

/// Check if 'diffopt' contains "vertical".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_vertical() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_VERTICAL) != 0)
}

/// Check if 'diffopt' contains "icase" (ignore case).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_icase() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_ICASE) != 0)
}

/// Check if 'diffopt' contains "iwhite" (ignore whitespace changes).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_iwhite() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_IWHITE) != 0)
}

/// Check if 'diffopt' contains "iwhiteall" (ignore all whitespace).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_iwhiteall() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_IWHITEALL) != 0)
}

/// Check if 'diffopt' contains "iwhiteeol" (ignore whitespace at EOL).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_iwhiteeol() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_IWHITEEOL) != 0)
}

/// Check if 'diffopt' contains "iblank" (ignore blank lines).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_iblank() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_IBLANK) != 0)
}

/// Check if 'diffopt' contains "followwrap" (follow wrap option).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_followwrap() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_FOLLOWWRAP) != 0)
}

/// Check if 'diffopt' contains "linematch" (match similar lines).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_linematch() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_LINEMATCH) != 0)
}

// =============================================================================
// Extended Diff Flags (Inline Highlighting and Anchors)
// =============================================================================

/// Check if 'diffopt' contains "inline:none" (disable inline highlight).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_inline_none() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_INLINE_NONE) != 0)
}

/// Check if 'diffopt' contains "inline:simple" (simple inline highlight).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_inline_simple() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_INLINE_SIMPLE) != 0)
}

/// Check if 'diffopt' contains "inline:char" (character diff inline highlight).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_inline_char() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_INLINE_CHAR) != 0)
}

/// Check if 'diffopt' contains "inline:word" (word diff inline highlight).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_inline_word() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_INLINE_WORD) != 0)
}

/// Check if 'diffopt' contains "anchor" (use diff anchors).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_anchor() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_ANCHOR) != 0)
}

/// Check if any inline highlighting mode is set.
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_inline_any() -> c_int {
    c_int::from((nvim_get_diff_flags() & ALL_INLINE) != 0)
}

/// Check if actual inline diff computation is enabled (char or word mode).
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_inline_diff() -> c_int {
    c_int::from((nvim_get_diff_flags() & ALL_INLINE_DIFF) != 0)
}

// DiffInlineMode is now defined in highlight.rs and re-exported at the top

// =============================================================================
// Diff Hunk Parsing
// =============================================================================

/// Diff hunk structure matching diffhunk_T in diff.c
#[repr(C)]
pub struct DiffHunk {
    pub lnum_orig: LinenrT,
    pub count_orig: c_int,
    pub lnum_new: LinenrT,
    pub count_new: c_int,
}

/// Check if a byte is an ASCII digit
#[inline]
const fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Parse digits from a string, advancing the pointer.
/// Returns the parsed number.
#[inline]
unsafe fn parse_digits(pp: &mut *const u8) -> i32 {
    let mut result: i32 = 0;
    while is_digit(**pp) {
        result = result
            .saturating_mul(10)
            .saturating_add(i32::from(**pp - b'0'));
        *pp = pp.add(1);
    }
    result
}

/// Parse an ED style diff line.
///
/// The line must be one of three formats:
/// - change: `{first}[,{last}]c{first}[,{last}]`
/// - append: `{first}a{first}[,{last}]`
/// - delete: `{first}[,{last}]d{first}`
///
/// Returns OK if successfully parsed, FAIL otherwise.
fn parse_diff_ed_impl(line: *const u8, hunk: &mut DiffHunk) -> c_int {
    if line.is_null() {
        return FAIL;
    }

    unsafe {
        let mut p = line;

        // Parse f1
        if !is_digit(*p) {
            return FAIL;
        }
        let f1 = parse_digits(&mut p);

        // Parse optional ,l1
        let l1 = if *p == b',' {
            p = p.add(1);
            parse_digits(&mut p)
        } else {
            f1
        };

        // Check for diff type character
        let difftype = *p;
        if difftype != b'a' && difftype != b'c' && difftype != b'd' {
            return FAIL;
        }
        p = p.add(1);

        // Parse f2
        let f2 = parse_digits(&mut p);

        // Parse optional ,l2
        let l2 = if *p == b',' {
            p = p.add(1);
            parse_digits(&mut p)
        } else {
            f2
        };

        // Validate ranges
        if l1 < f1 || l2 < f2 {
            return FAIL;
        }

        // Fill hunk based on diff type
        if difftype == b'a' {
            hunk.lnum_orig = f1 + 1;
            hunk.count_orig = 0;
        } else {
            hunk.lnum_orig = f1;
            hunk.count_orig = l1 - f1 + 1;
        }

        if difftype == b'd' {
            hunk.lnum_new = f2 + 1;
            hunk.count_new = 0;
        } else {
            hunk.lnum_new = f2;
            hunk.count_new = l2 - f2 + 1;
        }

        OK
    }
}

/// Parse a unified diff hunk header.
///
/// Format: `@@ -oldline,oldcount +newline,newcount @@`
///
/// Returns OK if successfully parsed, FAIL otherwise.
fn parse_diff_unified_impl(line: *const u8, hunk: &mut DiffHunk) -> c_int {
    if line.is_null() {
        return FAIL;
    }

    unsafe {
        let mut p = line;

        // Check for "@@ -"
        if *p != b'@' {
            return FAIL;
        }
        p = p.add(1);
        if *p != b'@' {
            return FAIL;
        }
        p = p.add(1);
        if *p != b' ' {
            return FAIL;
        }
        p = p.add(1);
        if *p != b'-' {
            return FAIL;
        }
        p = p.add(1);

        // Parse oldline
        let mut oldline = parse_digits(&mut p);

        // Parse optional ,oldcount
        let oldcount = if *p == b',' {
            p = p.add(1);
            parse_digits(&mut p)
        } else {
            1
        };

        // Check for " +"
        if *p != b' ' {
            return FAIL;
        }
        p = p.add(1);
        if *p != b'+' {
            return FAIL;
        }
        p = p.add(1);

        // Parse newline
        let mut newline = parse_digits(&mut p);

        // Parse optional ,newcount
        let newcount = if *p == b',' {
            p = p.add(1);
            parse_digits(&mut p)
        } else {
            1
        };

        // Adjust for zero counts
        if oldcount == 0 {
            oldline += 1;
        }
        if newcount == 0 {
            newline += 1;
        }
        if newline == 0 {
            newline = 1;
        }

        hunk.lnum_orig = oldline;
        hunk.count_orig = oldcount;
        hunk.lnum_new = newline;
        hunk.count_new = newcount;

        OK
    }
}

/// Parse an ED style diff line.
///
/// # Safety
/// - `line` must be a valid null-terminated string
/// - `hunk` must be a valid pointer to a DiffHunk struct
#[no_mangle]
pub unsafe extern "C" fn rs_parse_diff_ed(line: *const c_char, hunk: *mut DiffHunk) -> c_int {
    if hunk.is_null() {
        return FAIL;
    }
    parse_diff_ed_impl(line.cast::<u8>(), &mut *hunk)
}

/// Parse a unified diff hunk header.
///
/// # Safety
/// - `line` must be a valid null-terminated string
/// - `hunk` must be a valid pointer to a DiffHunk struct
#[no_mangle]
pub unsafe extern "C" fn rs_parse_diff_unified(line: *const c_char, hunk: *mut DiffHunk) -> c_int {
    if hunk.is_null() {
        return FAIL;
    }
    parse_diff_unified_impl(line.cast::<u8>(), &mut *hunk)
}

// =============================================================================
// Diff Buffer State Queries
// =============================================================================

/// Find the index of a buffer in the diff list.
///
/// Returns the buffer index (0 to DB_COUNT-1) or -1 if not found.
fn diff_buf_idx_impl(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return -1;
    }

    unsafe {
        for i in 0..DB_COUNT {
            let diffbuf = nvim_get_curtab_diffbuf(i);
            if !diffbuf.is_null() && diffbuf == buf.as_ptr() {
                return i;
            }
        }
        -1
    }
}

/// Check if the diff list is invalid (needs update).
fn diff_check_invalid_impl() -> bool {
    unsafe { nvim_get_curtab_diff_invalid() != 0 }
}

/// Count the number of active diff buffers.
fn diff_count_buffers_impl() -> c_int {
    unsafe {
        let mut count = 0;
        for i in 0..DB_COUNT {
            if !nvim_get_curtab_diffbuf(i).is_null() {
                count += 1;
            }
        }
        count
    }
}

/// Check if a buffer is in diff mode.
fn diff_buf_is_diffed_impl(buf: BufHandle) -> bool {
    diff_buf_idx_impl(buf) >= 0
}

/// Find the diff block that contains a given line number.
///
/// Returns the diff block pointer or null if not found.
fn diff_find_block_for_line_impl(buf_idx: c_int, lnum: LinenrT) -> DiffBlockPtr {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return std::ptr::null_mut();
    }

    unsafe {
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);

            // Check if lnum is within this block
            if lnum >= block_lnum && lnum < block_lnum + block_count.max(1) {
                return dp;
            }

            // If we've passed the line, stop searching
            if block_lnum > lnum {
                break;
            }

            dp = nvim_diffblock_get_next(dp);
        }
        std::ptr::null_mut()
    }
}

/// Calculate the number of filler lines at a given line.
///
/// Filler lines are displayed to align diff blocks between buffers.
fn diff_get_filler_lines_impl(buf_idx: c_int, lnum: LinenrT) -> c_int {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return 0;
    }

    unsafe {
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);

            // Filler lines appear above the diff block
            if lnum == block_lnum && block_count == 0 {
                // This is a pure insertion in other buffer(s)
                // Count max lines in other buffers
                let mut max_count = 0;
                for i in 0..DB_COUNT {
                    if i != buf_idx && !nvim_get_curtab_diffbuf(i).is_null() {
                        let count = nvim_diffblock_get_count(dp, i);
                        max_count = max_count.max(count);
                    }
                }
                return max_count;
            }

            // If we've passed the line, stop searching
            if block_lnum > lnum {
                break;
            }

            dp = nvim_diffblock_get_next(dp);
        }
        0
    }
}

/// FFI export: Find buffer index in diff list.
///
/// # Safety
/// `buf` must be a valid buffer handle or null.
#[no_mangle]
pub extern "C" fn rs_diff_buf_idx(buf: BufHandle) -> c_int {
    diff_buf_idx_impl(buf)
}

/// FFI export: Check if diff list needs update.
#[no_mangle]
pub extern "C" fn rs_diff_check_invalid() -> c_int {
    c_int::from(diff_check_invalid_impl())
}

/// FFI export: Count active diff buffers.
#[no_mangle]
pub extern "C" fn rs_diff_count_buffers() -> c_int {
    diff_count_buffers_impl()
}

/// FFI export: Check if buffer is in diff mode.
///
/// # Safety
/// `buf` must be a valid buffer handle or null.
#[no_mangle]
pub extern "C" fn rs_diff_buf_is_diffed(buf: BufHandle) -> c_int {
    c_int::from(diff_buf_is_diffed_impl(buf))
}

/// FFI export: Find diff block containing a line.
///
/// # Safety
/// `buf_idx` must be a valid buffer index (0 to DB_COUNT-1).
#[no_mangle]
pub extern "C" fn rs_diff_find_block_for_line(buf_idx: c_int, lnum: LinenrT) -> DiffBlockPtr {
    diff_find_block_for_line_impl(buf_idx, lnum)
}

/// FFI export: Get filler lines at a line number.
///
/// # Safety
/// `buf_idx` must be a valid buffer index (0 to DB_COUNT-1).
#[no_mangle]
pub extern "C" fn rs_diff_get_filler_lines(buf_idx: c_int, lnum: LinenrT) -> c_int {
    diff_get_filler_lines_impl(buf_idx, lnum)
}

// =============================================================================
// Diff Hunk Navigation
// =============================================================================

/// Find the next diff hunk after a given line number.
///
/// Returns the line number of the next hunk's start, or 0 if not found.
fn diff_find_next_hunk_impl(buf_idx: c_int, lnum: LinenrT) -> LinenrT {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return 0;
    }

    unsafe {
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);

            // If this block starts after our line, it's the next hunk
            if block_lnum > lnum {
                return block_lnum;
            }

            // If we're inside this block, find the next one
            let block_end = block_lnum + block_count.max(1) - 1;
            if lnum >= block_lnum && lnum <= block_end {
                // We're inside this hunk, find the next one
                let next_dp = nvim_diffblock_get_next(dp);
                if !next_dp.is_null() {
                    return nvim_diffblock_get_lnum(next_dp, buf_idx);
                }
                return 0;
            }

            dp = nvim_diffblock_get_next(dp);
        }
        0
    }
}

/// Find the previous diff hunk before a given line number.
///
/// Returns the line number of the previous hunk's start, or 0 if not found.
fn diff_find_prev_hunk_impl(buf_idx: c_int, lnum: LinenrT) -> LinenrT {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return 0;
    }

    unsafe {
        let mut prev_lnum: LinenrT = 0;
        let mut dp = nvim_get_diff_first_block();

        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);

            // If this block starts at or after our line, return the previous one
            if block_lnum >= lnum {
                return prev_lnum;
            }

            // If we're inside this block, return the previous one
            let block_end = block_lnum + block_count.max(1) - 1;
            if lnum <= block_end {
                return prev_lnum;
            }

            prev_lnum = block_lnum;
            dp = nvim_diffblock_get_next(dp);
        }

        // If we've gone through all blocks and lnum is after them, return the last one
        prev_lnum
    }
}

/// Check if a line is inside a diff hunk.
///
/// Returns true if the line is within a diff block, false otherwise.
fn diff_lnum_in_hunk_impl(buf_idx: c_int, lnum: LinenrT) -> bool {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return false;
    }

    unsafe {
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);

            // If this block is past our line, stop searching
            if block_lnum > lnum {
                return false;
            }

            // Check if we're in this block
            let block_end = block_lnum + block_count.max(1) - 1;
            if lnum >= block_lnum && lnum <= block_end {
                return true;
            }

            dp = nvim_diffblock_get_next(dp);
        }
        false
    }
}

// DiffHunkBounds is now defined in navigate.rs and re-exported at the top

/// Get the start and end lines of the hunk at a given position.
///
/// If the line is not in a hunk, returns a not_found result.
fn diff_hunk_start_end_impl(buf_idx: c_int, lnum: LinenrT) -> DiffHunkBounds {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return DiffHunkBounds::not_found();
    }

    unsafe {
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);

            // If this block is past our line, stop searching
            if block_lnum > lnum {
                return DiffHunkBounds::not_found();
            }

            // Check if we're in this block
            let block_end = block_lnum + block_count.max(1) - 1;
            if lnum >= block_lnum && lnum <= block_end {
                return DiffHunkBounds {
                    start_lnum: block_lnum,
                    end_lnum: block_end,
                    found: 1,
                };
            }

            dp = nvim_diffblock_get_next(dp);
        }
        DiffHunkBounds::not_found()
    }
}

/// FFI export: Find next diff hunk.
///
/// Returns the line number of the next hunk's start, or 0 if not found.
#[no_mangle]
pub extern "C" fn rs_diff_find_next_hunk(buf_idx: c_int, lnum: LinenrT) -> LinenrT {
    diff_find_next_hunk_impl(buf_idx, lnum)
}

/// FFI export: Find previous diff hunk.
///
/// Returns the line number of the previous hunk's start, or 0 if not found.
#[no_mangle]
pub extern "C" fn rs_diff_find_prev_hunk(buf_idx: c_int, lnum: LinenrT) -> LinenrT {
    diff_find_prev_hunk_impl(buf_idx, lnum)
}

/// FFI export: Check if line is in a hunk.
///
/// Returns 1 if the line is in a diff hunk, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_diff_lnum_in_hunk(buf_idx: c_int, lnum: LinenrT) -> c_int {
    c_int::from(diff_lnum_in_hunk_impl(buf_idx, lnum))
}

/// FFI export: Get hunk boundaries at a position.
///
/// Returns a DiffHunkBounds struct with start/end lines and found flag.
#[no_mangle]
pub extern "C" fn rs_diff_hunk_start_end(buf_idx: c_int, lnum: LinenrT) -> DiffHunkBounds {
    diff_hunk_start_end_impl(buf_idx, lnum)
}

// =============================================================================
// Diff String Comparison
// =============================================================================

/// Check if character is ASCII whitespace (space or tab).
#[inline]
const fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Skip leading whitespace in a string.
///
/// Returns pointer to first non-whitespace character.
#[inline]
#[allow(clippy::missing_const_for_fn)]
unsafe fn skipwhite(p: *const c_char) -> *const c_char {
    let mut ptr = p;
    #[allow(clippy::cast_sign_loss)]
    while ascii_iswhite(*ptr as u8) {
        ptr = ptr.add(1);
    }
    ptr
}

/// Compare two characters for equality, possibly ignoring case.
///
/// If characters are equal (possibly after case folding), returns the byte
/// length of the character. Otherwise returns 0.
///
/// This handles multibyte UTF-8 characters correctly.
#[allow(clippy::cast_sign_loss)]
unsafe fn diff_equal_char(p1: *const c_char, p2: *const c_char, diff_flags: c_int) -> c_int {
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

/// Compare two strings according to 'diffopt'.
///
/// Returns non-zero when the strings are different.
///
/// This function handles:
/// - DIFF_IBLANK: Treat lines with only whitespace as equal
/// - DIFF_ICASE: Case-insensitive comparison
/// - DIFF_IWHITE: Ignore changes in whitespace amount
/// - DIFF_IWHITEALL: Ignore all whitespace
/// - DIFF_IWHITEEOL: Ignore trailing whitespace
#[allow(clippy::cast_sign_loss)]
fn diff_cmp_impl(s1: *const c_char, s2: *const c_char, diff_flags: c_int) -> c_int {
    if s1.is_null() || s2.is_null() {
        return c_int::from(s1 != s2);
    }

    unsafe {
        // DIFF_IBLANK: If one of the lines contains only whitespace, treat as equal
        if (diff_flags & DIFF_IBLANK) != 0 && (*skipwhite(s1) == 0 || *skipwhite(s2) == 0) {
            return 0;
        }

        // No special flags: use simple strcmp
        if (diff_flags & (DIFF_ICASE | ALL_WHITE_DIFF)) == 0 {
            return libc::strcmp(s1, s2);
        }

        // Case-insensitive only (no whitespace handling): use mb_stricmp
        if (diff_flags & DIFF_ICASE) != 0 && (diff_flags & ALL_WHITE_DIFF) == 0 {
            return mb_stricmp(s1, s2);
        }

        // Complex comparison: handle whitespace and possibly case
        let mut p1 = s1;
        let mut p2 = s2;

        while *p1 != 0 && *p2 != 0 {
            let c1 = *p1 as u8;
            let c2 = *p2 as u8;

            // DIFF_IWHITE: Both chars are whitespace, skip all whitespace
            if (diff_flags & DIFF_IWHITE) != 0 && ascii_iswhite(c1) && ascii_iswhite(c2) {
                p1 = skipwhite(p1);
                p2 = skipwhite(p2);
            // DIFF_IWHITEALL: Either char is whitespace, skip all whitespace
            } else if (diff_flags & DIFF_IWHITEALL) != 0 && (ascii_iswhite(c1) || ascii_iswhite(c2))
            {
                p1 = skipwhite(p1);
                p2 = skipwhite(p2);
            } else {
                // Compare characters
                let l = diff_equal_char(p1, p2, diff_flags);
                if l == 0 {
                    break;
                }
                p1 = p1.add(l as usize);
                p2 = p2.add(l as usize);
            }
        }

        // Ignore trailing whitespace (always, due to DIFF_IWHITEEOL or cleanup)
        p1 = skipwhite(p1);
        p2 = skipwhite(p2);

        // If both strings are exhausted, they're equal
        c_int::from(*p1 != 0 || *p2 != 0)
    }
}

/// FFI export: Compare two strings according to 'diffopt'.
///
/// Returns non-zero when the strings are different.
///
/// # Safety
///
/// - `s1` and `s2` must be valid null-terminated strings or null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_cmp(s1: *const c_char, s2: *const c_char) -> c_int {
    diff_cmp_impl(s1, s2, nvim_get_diff_flags())
}

// =============================================================================
// Diff Block Copying
// =============================================================================

/// Copy diff block entry from one buffer index to another.
///
/// This computes the line number for `idx_new` based on the offset between
/// the two buffers from the previous diff block.
///
/// # Arguments
///
/// * `dprev` - The previous diff block (for computing offset), or null
/// * `dp` - The current diff block to update
/// * `idx_orig` - The source buffer index
/// * `idx_new` - The destination buffer index
fn diff_copy_entry_impl(dprev: DiffBlockPtr, dp: DiffBlockPtr, idx_orig: c_int, idx_new: c_int) {
    if dp.is_null() {
        return;
    }

    unsafe {
        let off = if dprev.is_null() {
            0
        } else {
            // Calculate offset: (prev_lnum_orig + prev_count_orig) - (prev_lnum_new + prev_count_new)
            (nvim_diffblock_get_lnum(dprev, idx_orig) + nvim_diffblock_get_count(dprev, idx_orig))
                - (nvim_diffblock_get_lnum(dprev, idx_new)
                    + nvim_diffblock_get_count(dprev, idx_new))
        };

        // dp->df_lnum[idx_new] = dp->df_lnum[idx_orig] - off
        nvim_diffblock_set_lnum(dp, idx_new, nvim_diffblock_get_lnum(dp, idx_orig) - off);
        // dp->df_count[idx_new] = dp->df_count[idx_orig]
        nvim_diffblock_set_count(dp, idx_new, nvim_diffblock_get_count(dp, idx_orig));
    }
}

/// FFI export: Copy diff block entry from one buffer index to another.
///
/// # Safety
///
/// - `dprev` must be a valid diff block pointer or null.
/// - `dp` must be a valid diff block pointer or null.
/// - `idx_orig` and `idx_new` must be valid buffer indices (0 to DB_COUNT-1).
#[no_mangle]
pub extern "C" fn rs_diff_copy_entry(
    dprev: DiffBlockPtr,
    dp: DiffBlockPtr,
    idx_orig: c_int,
    idx_new: c_int,
) {
    diff_copy_entry_impl(dprev, dp, idx_orig, idx_new);
}

// =============================================================================
// Diffopt Parsing
// =============================================================================

/// Result of parsing diffopt string.
/// This struct holds all the values that need to be set after parsing.
#[repr(C)]
pub struct DiffoptResult {
    /// Parsed diff flags
    pub diff_flags: c_int,
    /// Parsed diff algorithm
    pub diff_algorithm: c_int,
    /// Parsed context lines
    pub diff_context: c_int,
    /// Parsed fold column width
    pub diff_foldcolumn: c_int,
    /// Parsed linematch lines
    pub linematch_lines: c_int,
    /// Whether parsing succeeded (OK or FAIL)
    pub result: c_int,
}

impl DiffoptResult {
    /// Create a failed result
    const fn fail() -> Self {
        Self {
            diff_flags: 0,
            diff_algorithm: 0,
            diff_context: 6,
            diff_foldcolumn: 2,
            linematch_lines: 0,
            result: FAIL,
        }
    }
}

/// Check if byte is an ASCII digit
#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Parse digits from a pointer, advancing it past the digits.
/// Returns the parsed integer value.
unsafe fn getdigits(pp: &mut *const u8) -> c_int {
    let mut result: c_int = 0;
    while ascii_isdigit(**pp) {
        result = result
            .saturating_mul(10)
            .saturating_add(c_int::from(**pp - b'0'));
        *pp = pp.add(1);
    }
    result
}

/// Check if string starts with a given prefix
unsafe fn starts_with(s: *const u8, prefix: &[u8]) -> bool {
    for (i, &b) in prefix.iter().enumerate() {
        if *s.add(i) != b {
            return false;
        }
    }
    true
}

/// Parse the diffopt string and return the parsed values.
///
/// This function parses the comma-separated diffopt string and extracts
/// all flag values, algorithm settings, context/foldcolumn numbers, etc.
#[allow(clippy::too_many_lines)]
fn diffopt_parse_impl(p_dip: *const c_char) -> DiffoptResult {
    if p_dip.is_null() {
        return DiffoptResult::fail();
    }

    let mut diff_context_new: c_int = 6;
    let mut linematch_lines_new: c_int = 0;
    let mut diff_flags_new: c_int = 0;
    let mut diff_foldcolumn_new: c_int = 2;
    let mut diff_algorithm_new: c_int = 0;
    let mut diff_indent_heuristic: c_int = 0;

    unsafe {
        let mut p = p_dip.cast::<u8>();

        while *p != 0 {
            // Note: Keep this in sync with opt_dip_values
            if starts_with(p, b"filler") {
                p = p.add(6);
                diff_flags_new |= DIFF_FILLER;
            } else if starts_with(p, b"anchor") {
                p = p.add(6);
                diff_flags_new |= DIFF_ANCHOR;
            } else if starts_with(p, b"context:") && ascii_isdigit(*p.add(8)) {
                p = p.add(8);
                diff_context_new = getdigits(&mut p);
            } else if starts_with(p, b"iblank") {
                p = p.add(6);
                diff_flags_new |= DIFF_IBLANK;
            } else if starts_with(p, b"icase") {
                p = p.add(5);
                diff_flags_new |= DIFF_ICASE;
            } else if starts_with(p, b"iwhiteall") {
                p = p.add(9);
                diff_flags_new |= DIFF_IWHITEALL;
            } else if starts_with(p, b"iwhiteeol") {
                p = p.add(9);
                diff_flags_new |= DIFF_IWHITEEOL;
            } else if starts_with(p, b"iwhite") {
                p = p.add(6);
                diff_flags_new |= DIFF_IWHITE;
            } else if starts_with(p, b"horizontal") {
                p = p.add(10);
                diff_flags_new |= DIFF_HORIZONTAL;
            } else if starts_with(p, b"vertical") {
                p = p.add(8);
                diff_flags_new |= DIFF_VERTICAL;
            } else if starts_with(p, b"foldcolumn:") && ascii_isdigit(*p.add(11)) {
                p = p.add(11);
                diff_foldcolumn_new = getdigits(&mut p);
            } else if starts_with(p, b"hiddenoff") {
                p = p.add(9);
                diff_flags_new |= DIFF_HIDDEN_OFF;
            } else if starts_with(p, b"closeoff") {
                p = p.add(8);
                diff_flags_new |= DIFF_CLOSE_OFF;
            } else if starts_with(p, b"followwrap") {
                p = p.add(10);
                diff_flags_new |= DIFF_FOLLOWWRAP;
            } else if starts_with(p, b"indent-heuristic") {
                p = p.add(16);
                diff_indent_heuristic = XDF_INDENT_HEURISTIC;
            } else if starts_with(p, b"internal") {
                p = p.add(8);
                diff_flags_new |= DIFF_INTERNAL;
            } else if starts_with(p, b"algorithm:") {
                // Note: Keep this in sync with opt_dip_algorithm_values
                p = p.add(10);
                if starts_with(p, b"myers") {
                    p = p.add(5);
                    diff_algorithm_new = 0;
                } else if starts_with(p, b"minimal") {
                    p = p.add(7);
                    diff_algorithm_new = XDF_NEED_MINIMAL;
                } else if starts_with(p, b"patience") {
                    p = p.add(8);
                    diff_algorithm_new = XDF_PATIENCE_DIFF;
                } else if starts_with(p, b"histogram") {
                    p = p.add(9);
                    diff_algorithm_new = XDF_HISTOGRAM_DIFF;
                } else {
                    return DiffoptResult::fail();
                }
            } else if starts_with(p, b"inline:") {
                // Note: Keep this in sync with opt_dip_inline_values
                p = p.add(7);
                if starts_with(p, b"none") {
                    p = p.add(4);
                    diff_flags_new &= !ALL_INLINE;
                    diff_flags_new |= DIFF_INLINE_NONE;
                } else if starts_with(p, b"simple") {
                    p = p.add(6);
                    diff_flags_new &= !ALL_INLINE;
                    diff_flags_new |= DIFF_INLINE_SIMPLE;
                } else if starts_with(p, b"char") {
                    p = p.add(4);
                    diff_flags_new &= !ALL_INLINE;
                    diff_flags_new |= DIFF_INLINE_CHAR;
                } else if starts_with(p, b"word") {
                    p = p.add(4);
                    diff_flags_new &= !ALL_INLINE;
                    diff_flags_new |= DIFF_INLINE_WORD;
                } else {
                    return DiffoptResult::fail();
                }
            } else if starts_with(p, b"linematch:") && ascii_isdigit(*p.add(10)) {
                p = p.add(10);
                linematch_lines_new = getdigits(&mut p);
                diff_flags_new |= DIFF_LINEMATCH;
                // linematch does not make sense without filler set
                diff_flags_new |= DIFF_FILLER;
            } else {
                // Unknown option or end of string
                if *p != b',' && *p != 0 {
                    return DiffoptResult::fail();
                }
            }

            // Check for separator
            if *p != b',' && *p != 0 {
                return DiffoptResult::fail();
            }

            if *p == b',' {
                p = p.add(1);
            }
        }
    }

    // Combine algorithm with indent heuristic
    diff_algorithm_new |= diff_indent_heuristic;

    // Can't have both "horizontal" and "vertical"
    if (diff_flags_new & DIFF_HORIZONTAL) != 0 && (diff_flags_new & DIFF_VERTICAL) != 0 {
        return DiffoptResult::fail();
    }

    // Ensure diff_context is at least 1 (0 means use 1)
    if diff_context_new == 0 {
        diff_context_new = 1;
    }

    DiffoptResult {
        diff_flags: diff_flags_new,
        diff_algorithm: diff_algorithm_new,
        diff_context: diff_context_new,
        diff_foldcolumn: diff_foldcolumn_new,
        linematch_lines: linematch_lines_new,
        result: OK,
    }
}

/// FFI export: Parse the diffopt string.
///
/// Returns a DiffoptResult struct with all parsed values and a result code.
///
/// # Safety
///
/// `p_dip` must be a valid null-terminated string or null.
#[no_mangle]
pub extern "C" fn rs_diffopt_parse(p_dip: *const c_char) -> DiffoptResult {
    diffopt_parse_impl(p_dip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_flag_constants() {
        // Verify the diff flag constants match expected values from diff.c
        assert_eq!(DIFF_FILLER, 0x001);
        assert_eq!(DIFF_IBLANK, 0x002);
        assert_eq!(DIFF_ICASE, 0x004);
        assert_eq!(DIFF_IWHITE, 0x008);
        assert_eq!(DIFF_IWHITEALL, 0x010);
        assert_eq!(DIFF_IWHITEEOL, 0x020);
        assert_eq!(DIFF_HORIZONTAL, 0x040);
        assert_eq!(DIFF_VERTICAL, 0x080);
        assert_eq!(DIFF_HIDDEN_OFF, 0x100);
        assert_eq!(DIFF_INTERNAL, 0x200);
        assert_eq!(DIFF_CLOSE_OFF, 0x400);
        assert_eq!(DIFF_FOLLOWWRAP, 0x800);
        assert_eq!(DIFF_LINEMATCH, 0x1000);
        // Extended flags
        assert_eq!(DIFF_INLINE_NONE, 0x2000);
        assert_eq!(DIFF_INLINE_SIMPLE, 0x4000);
        assert_eq!(DIFF_INLINE_CHAR, 0x8000);
        assert_eq!(DIFF_INLINE_WORD, 0x10000);
        assert_eq!(DIFF_ANCHOR, 0x20000);
    }

    #[test]
    fn test_diff_flags_are_distinct() {
        // Ensure all flags are distinct (no overlap)
        let flags = [
            DIFF_FILLER,
            DIFF_IBLANK,
            DIFF_ICASE,
            DIFF_IWHITE,
            DIFF_IWHITEALL,
            DIFF_IWHITEEOL,
            DIFF_HORIZONTAL,
            DIFF_VERTICAL,
            DIFF_HIDDEN_OFF,
            DIFF_INTERNAL,
            DIFF_CLOSE_OFF,
            DIFF_FOLLOWWRAP,
            DIFF_LINEMATCH,
            DIFF_INLINE_NONE,
            DIFF_INLINE_SIMPLE,
            DIFF_INLINE_CHAR,
            DIFF_INLINE_WORD,
            DIFF_ANCHOR,
        ];

        for i in 0..flags.len() {
            for j in (i + 1)..flags.len() {
                assert_eq!(
                    flags[i] & flags[j],
                    0,
                    "Flags at indices {i} and {j} overlap"
                );
            }
        }
    }

    #[test]
    fn test_diff_flags_are_single_bit() {
        // Each flag should be a single bit (power of 2)
        let flags = [
            DIFF_FILLER,
            DIFF_IBLANK,
            DIFF_ICASE,
            DIFF_IWHITE,
            DIFF_IWHITEALL,
            DIFF_IWHITEEOL,
            DIFF_HORIZONTAL,
            DIFF_VERTICAL,
            DIFF_HIDDEN_OFF,
            DIFF_INTERNAL,
            DIFF_CLOSE_OFF,
            DIFF_FOLLOWWRAP,
            DIFF_LINEMATCH,
            DIFF_INLINE_NONE,
            DIFF_INLINE_SIMPLE,
            DIFF_INLINE_CHAR,
            DIFF_INLINE_WORD,
            DIFF_ANCHOR,
        ];

        for flag in flags {
            // A power of 2 has exactly one bit set
            // n & (n - 1) == 0 for powers of 2
            assert_eq!(flag & (flag - 1), 0, "Flag {flag:#x} is not a power of 2");
            assert_ne!(flag, 0, "Flag should not be zero");
        }
    }

    #[test]
    fn test_diff_flag_bit_positions() {
        // Verify exact bit positions for each flag
        assert_eq!(DIFF_FILLER, 1 << 0); // bit 0
        assert_eq!(DIFF_IBLANK, 1 << 1); // bit 1
        assert_eq!(DIFF_ICASE, 1 << 2); // bit 2
        assert_eq!(DIFF_IWHITE, 1 << 3); // bit 3
        assert_eq!(DIFF_IWHITEALL, 1 << 4); // bit 4
        assert_eq!(DIFF_IWHITEEOL, 1 << 5); // bit 5
        assert_eq!(DIFF_HORIZONTAL, 1 << 6); // bit 6
        assert_eq!(DIFF_VERTICAL, 1 << 7); // bit 7
        assert_eq!(DIFF_HIDDEN_OFF, 1 << 8); // bit 8
        assert_eq!(DIFF_INTERNAL, 1 << 9); // bit 9
        assert_eq!(DIFF_CLOSE_OFF, 1 << 10); // bit 10
        assert_eq!(DIFF_FOLLOWWRAP, 1 << 11); // bit 11
        assert_eq!(DIFF_LINEMATCH, 1 << 12); // bit 12
                                             // Extended flags
        assert_eq!(DIFF_INLINE_NONE, 1 << 13); // bit 13
        assert_eq!(DIFF_INLINE_SIMPLE, 1 << 14); // bit 14
        assert_eq!(DIFF_INLINE_CHAR, 1 << 15); // bit 15
        assert_eq!(DIFF_INLINE_WORD, 1 << 16); // bit 16
        assert_eq!(DIFF_ANCHOR, 1 << 17); // bit 17
    }

    #[test]
    fn test_diff_flag_combinations() {
        // Test that combining flags works correctly
        let combined = DIFF_FILLER | DIFF_HORIZONTAL | DIFF_INTERNAL | DIFF_ICASE;

        // Check each flag is set in the combination
        assert_ne!(combined & DIFF_FILLER, 0);
        assert_ne!(combined & DIFF_HORIZONTAL, 0);
        assert_ne!(combined & DIFF_INTERNAL, 0);
        assert_ne!(combined & DIFF_ICASE, 0);

        // Check other flags are not set
        assert_eq!(combined & DIFF_HIDDEN_OFF, 0);
        assert_eq!(combined & DIFF_CLOSE_OFF, 0);
        assert_eq!(combined & DIFF_VERTICAL, 0);
    }

    #[test]
    fn test_diff_all_flags_combined() {
        // All flags combined should produce a valid mask
        let all_flags = DIFF_FILLER
            | DIFF_IBLANK
            | DIFF_ICASE
            | DIFF_IWHITE
            | DIFF_IWHITEALL
            | DIFF_IWHITEEOL
            | DIFF_HORIZONTAL
            | DIFF_VERTICAL
            | DIFF_HIDDEN_OFF
            | DIFF_INTERNAL
            | DIFF_CLOSE_OFF
            | DIFF_FOLLOWWRAP
            | DIFF_LINEMATCH
            | DIFF_INLINE_NONE
            | DIFF_INLINE_SIMPLE
            | DIFF_INLINE_CHAR
            | DIFF_INLINE_WORD
            | DIFF_ANCHOR;
        // Verify it's positive (no overflow from OR operations)
        assert!(all_flags > 0);
        // Verify expected combined value: all bits 0-17 set = 0x3FFFF
        assert_eq!(all_flags, 0x3FFFF);
    }

    #[test]
    fn test_diff_flag_count() {
        // There should be exactly 18 defined flags
        let flags = [
            DIFF_FILLER,
            DIFF_IBLANK,
            DIFF_ICASE,
            DIFF_IWHITE,
            DIFF_IWHITEALL,
            DIFF_IWHITEEOL,
            DIFF_HORIZONTAL,
            DIFF_VERTICAL,
            DIFF_HIDDEN_OFF,
            DIFF_INTERNAL,
            DIFF_CLOSE_OFF,
            DIFF_FOLLOWWRAP,
            DIFF_LINEMATCH,
            DIFF_INLINE_NONE,
            DIFF_INLINE_SIMPLE,
            DIFF_INLINE_CHAR,
            DIFF_INLINE_WORD,
            DIFF_ANCHOR,
        ];
        assert_eq!(flags.len(), 18);
    }

    #[test]
    fn test_diff_filler_is_lowest_bit() {
        // DIFF_FILLER should be the lowest bit set in any flag
        let all_flags = DIFF_FILLER
            | DIFF_IBLANK
            | DIFF_ICASE
            | DIFF_IWHITE
            | DIFF_IWHITEALL
            | DIFF_IWHITEEOL
            | DIFF_HORIZONTAL
            | DIFF_VERTICAL
            | DIFF_HIDDEN_OFF
            | DIFF_INTERNAL
            | DIFF_CLOSE_OFF
            | DIFF_FOLLOWWRAP
            | DIFF_LINEMATCH
            | DIFF_INLINE_NONE
            | DIFF_INLINE_SIMPLE
            | DIFF_INLINE_CHAR
            | DIFF_INLINE_WORD
            | DIFF_ANCHOR;
        // trailing_zeros of all flags combined should be 0 (DIFF_FILLER is bit 0)
        assert_eq!(all_flags.trailing_zeros(), 0);
    }

    #[test]
    fn test_whitespace_flags_group() {
        // Test the ALL_WHITE_DIFF group
        let all_white = DIFF_IWHITE | DIFF_IWHITEALL | DIFF_IWHITEEOL;
        assert_eq!(all_white, 0x038); // bits 3, 4, 5
    }

    #[test]
    fn test_inline_flags_group() {
        // Test the ALL_INLINE group
        assert_eq!(
            ALL_INLINE,
            DIFF_INLINE_NONE | DIFF_INLINE_SIMPLE | DIFF_INLINE_CHAR | DIFF_INLINE_WORD
        );
        assert_eq!(ALL_INLINE, 0x1E000); // bits 13-16
    }

    #[test]
    fn test_inline_diff_flags_group() {
        // Test the ALL_INLINE_DIFF group (only char and word modes)
        assert_eq!(ALL_INLINE_DIFF, DIFF_INLINE_CHAR | DIFF_INLINE_WORD);
        assert_eq!(ALL_INLINE_DIFF, 0x18000); // bits 15-16
    }

    #[test]
    fn test_diff_inline_mode_enum() {
        // Test DiffInlineMode enum values
        assert_eq!(DiffInlineMode::None as i32, 0);
        assert_eq!(DiffInlineMode::Simple as i32, 1);
        assert_eq!(DiffInlineMode::Char as i32, 2);
        assert_eq!(DiffInlineMode::Word as i32, 3);
    }

    #[test]
    fn test_diff_inline_mode_size() {
        // Enum should be the size of a c_int for FFI
        assert_eq!(
            std::mem::size_of::<DiffInlineMode>(),
            std::mem::size_of::<c_int>()
        );
    }

    // =========================================================================
    // Diff Parsing Tests
    // =========================================================================

    fn make_hunk() -> DiffHunk {
        DiffHunk {
            lnum_orig: 0,
            count_orig: 0,
            lnum_new: 0,
            count_new: 0,
        }
    }

    #[test]
    fn test_parse_diff_ed_change() {
        let mut hunk = make_hunk();
        // "1,3c4,6" - change lines 1-3 to lines 4-6
        let line = b"1,3c4,6\0";
        let result = parse_diff_ed_impl(line.as_ptr(), &mut hunk);
        assert_eq!(result, OK);
        assert_eq!(hunk.lnum_orig, 1);
        assert_eq!(hunk.count_orig, 3);
        assert_eq!(hunk.lnum_new, 4);
        assert_eq!(hunk.count_new, 3);
    }

    #[test]
    fn test_parse_diff_ed_change_single() {
        let mut hunk = make_hunk();
        // "5c10" - change line 5 to line 10
        let line = b"5c10\0";
        let result = parse_diff_ed_impl(line.as_ptr(), &mut hunk);
        assert_eq!(result, OK);
        assert_eq!(hunk.lnum_orig, 5);
        assert_eq!(hunk.count_orig, 1);
        assert_eq!(hunk.lnum_new, 10);
        assert_eq!(hunk.count_new, 1);
    }

    #[test]
    fn test_parse_diff_ed_append() {
        let mut hunk = make_hunk();
        // "3a4,7" - after line 3, append lines 4-7
        let line = b"3a4,7\0";
        let result = parse_diff_ed_impl(line.as_ptr(), &mut hunk);
        assert_eq!(result, OK);
        assert_eq!(hunk.lnum_orig, 4); // f1 + 1
        assert_eq!(hunk.count_orig, 0);
        assert_eq!(hunk.lnum_new, 4);
        assert_eq!(hunk.count_new, 4); // 7 - 4 + 1
    }

    #[test]
    fn test_parse_diff_ed_delete() {
        let mut hunk = make_hunk();
        // "2,5d1" - delete lines 2-5, was at line 1 in new
        let line = b"2,5d1\0";
        let result = parse_diff_ed_impl(line.as_ptr(), &mut hunk);
        assert_eq!(result, OK);
        assert_eq!(hunk.lnum_orig, 2);
        assert_eq!(hunk.count_orig, 4); // 5 - 2 + 1
        assert_eq!(hunk.lnum_new, 2); // f2 + 1
        assert_eq!(hunk.count_new, 0);
    }

    #[test]
    fn test_parse_diff_ed_invalid_no_type() {
        let mut hunk = make_hunk();
        let line = b"1,3x4,6\0"; // 'x' is not a valid type
        let result = parse_diff_ed_impl(line.as_ptr(), &mut hunk);
        assert_eq!(result, FAIL);
    }

    #[test]
    fn test_parse_diff_ed_invalid_range() {
        let mut hunk = make_hunk();
        let line = b"5,3c4,6\0"; // l1 < f1 is invalid
        let result = parse_diff_ed_impl(line.as_ptr(), &mut hunk);
        assert_eq!(result, FAIL);
    }

    #[test]
    fn test_parse_diff_ed_no_digits() {
        let mut hunk = make_hunk();
        let line = b"abc\0";
        let result = parse_diff_ed_impl(line.as_ptr(), &mut hunk);
        assert_eq!(result, FAIL);
    }

    #[test]
    fn test_parse_diff_unified_basic() {
        let mut hunk = make_hunk();
        // "@@ -10,5 +20,3 @@"
        let line = b"@@ -10,5 +20,3 @@\0";
        let result = parse_diff_unified_impl(line.as_ptr(), &mut hunk);
        assert_eq!(result, OK);
        assert_eq!(hunk.lnum_orig, 10);
        assert_eq!(hunk.count_orig, 5);
        assert_eq!(hunk.lnum_new, 20);
        assert_eq!(hunk.count_new, 3);
    }

    #[test]
    fn test_parse_diff_unified_no_count() {
        let mut hunk = make_hunk();
        // "@@ -10 +20 @@" - no counts means 1
        let line = b"@@ -10 +20 @@\0";
        let result = parse_diff_unified_impl(line.as_ptr(), &mut hunk);
        assert_eq!(result, OK);
        assert_eq!(hunk.lnum_orig, 10);
        assert_eq!(hunk.count_orig, 1);
        assert_eq!(hunk.lnum_new, 20);
        assert_eq!(hunk.count_new, 1);
    }

    #[test]
    fn test_parse_diff_unified_zero_oldcount() {
        let mut hunk = make_hunk();
        // "@@ -5,0 +10,3 @@" - zero old count means insertion
        let line = b"@@ -5,0 +10,3 @@\0";
        let result = parse_diff_unified_impl(line.as_ptr(), &mut hunk);
        assert_eq!(result, OK);
        assert_eq!(hunk.lnum_orig, 6); // 5 + 1 because count is 0
        assert_eq!(hunk.count_orig, 0);
        assert_eq!(hunk.lnum_new, 10);
        assert_eq!(hunk.count_new, 3);
    }

    #[test]
    fn test_parse_diff_unified_zero_newcount() {
        let mut hunk = make_hunk();
        // "@@ -5,3 +10,0 @@" - zero new count means deletion
        let line = b"@@ -5,3 +10,0 @@\0";
        let result = parse_diff_unified_impl(line.as_ptr(), &mut hunk);
        assert_eq!(result, OK);
        assert_eq!(hunk.lnum_orig, 5);
        assert_eq!(hunk.count_orig, 3);
        assert_eq!(hunk.lnum_new, 11); // 10 + 1 because count is 0
        assert_eq!(hunk.count_new, 0);
    }

    #[test]
    fn test_parse_diff_unified_invalid_prefix() {
        let mut hunk = make_hunk();
        let line = b"-- -10,5 +20,3 @@\0"; // wrong prefix
        let result = parse_diff_unified_impl(line.as_ptr(), &mut hunk);
        assert_eq!(result, FAIL);
    }

    #[test]
    fn test_parse_diff_unified_missing_plus() {
        let mut hunk = make_hunk();
        let line = b"@@ -10,5 -20,3 @@\0"; // missing +
        let result = parse_diff_unified_impl(line.as_ptr(), &mut hunk);
        assert_eq!(result, FAIL);
    }

    #[test]
    fn test_parse_diff_null_line() {
        let mut hunk = make_hunk();
        let result = parse_diff_ed_impl(std::ptr::null(), &mut hunk);
        assert_eq!(result, FAIL);

        let result = parse_diff_unified_impl(std::ptr::null(), &mut hunk);
        assert_eq!(result, FAIL);
    }

    #[test]
    fn test_diffhunk_struct_size() {
        // Verify the struct is properly sized for C interop
        // Should be 4 fields * 4 bytes = 16 bytes
        assert_eq!(std::mem::size_of::<DiffHunk>(), 16);
    }

    // =========================================================================
    // Diff Buffer State Tests
    // =========================================================================

    #[test]
    fn test_db_count_constant() {
        // DB_COUNT should be 8
        assert_eq!(DB_COUNT, 8);
    }

    #[test]
    fn test_diff_block_handle_null() {
        let handle = DiffBlockHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_buf_handle_null() {
        let handle = BufHandle::null();
        assert!(handle.is_null());
    }

    // Note: Tests for diff_buf_idx_impl, diff_buf_is_diffed_impl,
    // diff_find_block_for_line_impl, and diff_get_filler_lines_impl
    // are not included here because they require C FFI calls that are
    // only available when linked with the full Neovim binary.
    // These functions are tested through integration tests.

    #[test]
    fn test_diff_block_handle_size() {
        // Should be pointer-sized
        assert_eq!(
            std::mem::size_of::<DiffBlockHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
    }

    #[test]
    fn test_buf_handle_size() {
        // Should be pointer-sized
        assert_eq!(
            std::mem::size_of::<BufHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
    }

    // =========================================================================
    // Diff Hunk Navigation Tests
    // =========================================================================

    #[test]
    fn test_diff_hunk_bounds_not_found() {
        let bounds = DiffHunkBounds::not_found();
        assert_eq!(bounds.start_lnum, 0);
        assert_eq!(bounds.end_lnum, 0);
        assert_eq!(bounds.found, 0);
    }

    #[test]
    fn test_diff_hunk_bounds_size() {
        // Should be 3 * 4 = 12 bytes (2 LinenrT + 1 c_int)
        assert_eq!(std::mem::size_of::<DiffHunkBounds>(), 12);
    }

    // Note: Tests for diff_find_next_hunk_impl, diff_find_prev_hunk_impl,
    // diff_lnum_in_hunk_impl, and diff_hunk_start_end_impl
    // are not included here because they require C FFI calls that are
    // only available when linked with the full Neovim binary.

    // =========================================================================
    // Diff String Comparison Tests
    // =========================================================================

    #[test]
    fn test_ascii_iswhite() {
        assert!(ascii_iswhite(b' '));
        assert!(ascii_iswhite(b'\t'));
        assert!(!ascii_iswhite(b'a'));
        assert!(!ascii_iswhite(b'\n'));
        assert!(!ascii_iswhite(0));
    }

    #[test]
    fn test_all_white_diff_constant() {
        // ALL_WHITE_DIFF should be the combination of IWHITE, IWHITEALL, IWHITEEOL
        assert_eq!(
            ALL_WHITE_DIFF,
            DIFF_IWHITE | DIFF_IWHITEALL | DIFF_IWHITEEOL
        );
        assert_eq!(ALL_WHITE_DIFF, 0x038);
    }

    // Note: Tests for diff_cmp_impl and diff_copy_entry_impl
    // are not included here because they require C FFI calls that are
    // only available when linked with the full Neovim binary.

    // =========================================================================
    // XDF Algorithm Constants Tests
    // =========================================================================

    #[test]
    fn test_xdf_constants() {
        assert_eq!(XDF_NEED_MINIMAL, 1 << 0);
        assert_eq!(XDF_PATIENCE_DIFF, 1 << 14);
        assert_eq!(XDF_HISTOGRAM_DIFF, 1 << 15);
        assert_eq!(XDF_INDENT_HEURISTIC, 1 << 23);
    }

    // =========================================================================
    // Diffopt Parsing Tests
    // =========================================================================

    #[test]
    fn test_ascii_isdigit() {
        for c in b'0'..=b'9' {
            assert!(ascii_isdigit(c));
        }
        assert!(!ascii_isdigit(b'a'));
        assert!(!ascii_isdigit(b' '));
        assert!(!ascii_isdigit(0));
    }

    #[test]
    fn test_diffopt_result_size() {
        // 6 * 4 bytes = 24 bytes
        assert_eq!(std::mem::size_of::<DiffoptResult>(), 24);
    }

    #[test]
    fn test_diffopt_parse_empty() {
        let opt = b"\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_eq!(result.diff_flags, 0);
        assert_eq!(result.diff_algorithm, 0);
        assert_eq!(result.diff_context, 6);
        assert_eq!(result.diff_foldcolumn, 2);
        assert_eq!(result.linematch_lines, 0);
    }

    #[test]
    fn test_diffopt_parse_filler() {
        let opt = b"filler\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_ne!(result.diff_flags & DIFF_FILLER, 0);
    }

    #[test]
    fn test_diffopt_parse_multiple() {
        let opt = b"filler,internal,icase\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_ne!(result.diff_flags & DIFF_FILLER, 0);
        assert_ne!(result.diff_flags & DIFF_INTERNAL, 0);
        assert_ne!(result.diff_flags & DIFF_ICASE, 0);
    }

    #[test]
    fn test_diffopt_parse_context() {
        let opt = b"context:10\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_eq!(result.diff_context, 10);
    }

    #[test]
    fn test_diffopt_parse_context_zero() {
        let opt = b"context:0\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_eq!(result.diff_context, 1); // 0 becomes 1
    }

    #[test]
    fn test_diffopt_parse_foldcolumn() {
        let opt = b"foldcolumn:5\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_eq!(result.diff_foldcolumn, 5);
    }

    #[test]
    fn test_diffopt_parse_horizontal() {
        let opt = b"horizontal\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_ne!(result.diff_flags & DIFF_HORIZONTAL, 0);
    }

    #[test]
    fn test_diffopt_parse_vertical() {
        let opt = b"vertical\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_ne!(result.diff_flags & DIFF_VERTICAL, 0);
    }

    #[test]
    fn test_diffopt_parse_horizontal_vertical_fail() {
        let opt = b"horizontal,vertical\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, FAIL); // Can't have both
    }

    #[test]
    fn test_diffopt_parse_algorithm_myers() {
        let opt = b"algorithm:myers\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_eq!(result.diff_algorithm, 0);
    }

    #[test]
    fn test_diffopt_parse_algorithm_minimal() {
        let opt = b"algorithm:minimal\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_eq!(result.diff_algorithm, XDF_NEED_MINIMAL);
    }

    #[test]
    fn test_diffopt_parse_algorithm_patience() {
        let opt = b"algorithm:patience\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_eq!(result.diff_algorithm, XDF_PATIENCE_DIFF);
    }

    #[test]
    fn test_diffopt_parse_algorithm_histogram() {
        let opt = b"algorithm:histogram\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_eq!(result.diff_algorithm, XDF_HISTOGRAM_DIFF);
    }

    #[test]
    fn test_diffopt_parse_algorithm_invalid() {
        let opt = b"algorithm:unknown\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, FAIL);
    }

    #[test]
    fn test_diffopt_parse_indent_heuristic() {
        let opt = b"indent-heuristic\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_ne!(result.diff_algorithm & XDF_INDENT_HEURISTIC, 0);
    }

    #[test]
    fn test_diffopt_parse_algorithm_with_indent() {
        let opt = b"algorithm:patience,indent-heuristic\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_eq!(
            result.diff_algorithm,
            XDF_PATIENCE_DIFF | XDF_INDENT_HEURISTIC
        );
    }

    #[test]
    fn test_diffopt_parse_inline_none() {
        let opt = b"inline:none\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_ne!(result.diff_flags & DIFF_INLINE_NONE, 0);
        assert_eq!(result.diff_flags & ALL_INLINE, DIFF_INLINE_NONE);
    }

    #[test]
    fn test_diffopt_parse_inline_simple() {
        let opt = b"inline:simple\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_ne!(result.diff_flags & DIFF_INLINE_SIMPLE, 0);
    }

    #[test]
    fn test_diffopt_parse_inline_char() {
        let opt = b"inline:char\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_ne!(result.diff_flags & DIFF_INLINE_CHAR, 0);
    }

    #[test]
    fn test_diffopt_parse_inline_word() {
        let opt = b"inline:word\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_ne!(result.diff_flags & DIFF_INLINE_WORD, 0);
    }

    #[test]
    fn test_diffopt_parse_inline_invalid() {
        let opt = b"inline:unknown\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, FAIL);
    }

    #[test]
    fn test_diffopt_parse_linematch() {
        let opt = b"linematch:100\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_eq!(result.linematch_lines, 100);
        assert_ne!(result.diff_flags & DIFF_LINEMATCH, 0);
        assert_ne!(result.diff_flags & DIFF_FILLER, 0); // FILLER is always set with linematch
    }

    #[test]
    fn test_diffopt_parse_whitespace_flags() {
        let opt = b"iwhite,iwhiteall,iwhiteeol,iblank\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_ne!(result.diff_flags & DIFF_IWHITE, 0);
        assert_ne!(result.diff_flags & DIFF_IWHITEALL, 0);
        assert_ne!(result.diff_flags & DIFF_IWHITEEOL, 0);
        assert_ne!(result.diff_flags & DIFF_IBLANK, 0);
    }

    #[test]
    fn test_diffopt_parse_other_flags() {
        let opt = b"hiddenoff,closeoff,followwrap,anchor\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_ne!(result.diff_flags & DIFF_HIDDEN_OFF, 0);
        assert_ne!(result.diff_flags & DIFF_CLOSE_OFF, 0);
        assert_ne!(result.diff_flags & DIFF_FOLLOWWRAP, 0);
        assert_ne!(result.diff_flags & DIFF_ANCHOR, 0);
    }

    #[test]
    fn test_diffopt_parse_invalid_option() {
        let opt = b"unknownoption\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, FAIL);
    }

    #[test]
    fn test_diffopt_parse_null() {
        let result = diffopt_parse_impl(std::ptr::null());
        assert_eq!(result.result, FAIL);
    }

    #[test]
    fn test_diffopt_parse_complex() {
        // Test a realistic diffopt string
        let opt = b"internal,filler,closeoff,algorithm:histogram,context:3,inline:char\0";
        let result = diffopt_parse_impl(opt.as_ptr().cast::<c_char>());
        assert_eq!(result.result, OK);
        assert_ne!(result.diff_flags & DIFF_INTERNAL, 0);
        assert_ne!(result.diff_flags & DIFF_FILLER, 0);
        assert_ne!(result.diff_flags & DIFF_CLOSE_OFF, 0);
        assert_ne!(result.diff_flags & DIFF_INLINE_CHAR, 0);
        assert_eq!(result.diff_algorithm, XDF_HISTOGRAM_DIFF);
        assert_eq!(result.diff_context, 3);
    }
}
