//! Inline diff detection and highlighting
//!
//! This module provides Rust implementations for detecting changes within
//! diff lines, supporting both simple and advanced (char/word) inline diff modes.
//!
//! The main entry point is [`rs_diff_find_change`] which is called during
//! diff line rendering to determine what portion of a line has changed.

use std::ffi::c_char;
use std::os::raw::c_int;

use crate::buffer::{rs_diff_check_sanity, BufHandle, DiffBlockHandle, TabpageHandle, DB_COUNT};
use crate::highlight::{ascii_iswhite, DiffLineChange};

/// Line number type matching linenr_T (i32).
type LinenrT = i32;

/// Column number type.
type ColnrT = i32;

// =============================================================================
// Diff Flags (must match C definitions)
// =============================================================================

const DIFF_IWHITE: c_int = 0x008;
const DIFF_IWHITEALL: c_int = 0x010;
const DIFF_INLINE_NONE: c_int = 0x2000;
const ALL_INLINE_DIFF: c_int = 0x8000 | 0x10000; // DIFF_INLINE_CHAR | DIFF_INLINE_WORD

const MAXCOL: ColnrT = i32::MAX;
const FAIL: c_int = 0;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_diff_flags() -> c_int;
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_get_curtab_diffbuf(idx: c_int) -> BufHandle;
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_has_changes(dp: DiffBlockHandle) -> bool;
    fn nvim_diffblock_get_changes_len(dp: DiffBlockHandle) -> c_int;
    fn nvim_diffblock_get_change(dp: DiffBlockHandle, change_idx: c_int) -> *const DiffLineChange;

    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_diff_buf_idx(buf: BufHandle, tp: TabpageHandle) -> c_int;

    fn ml_get_buf(buf: BufHandle, lnum: LinenrT) -> *const c_char;

    // UTF-8 functions
    fn utf_head_off(base: *const c_char, ptr: *const c_char) -> c_int;
}

/// Opaque window handle.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WinHandle(*mut std::ffi::c_void);

impl WinHandle {
    /// Create a null handle.
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if the handle is null.
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// Diffline Structure (FFI compatible)
// =============================================================================

/// Result structure for diff_find_change.
///
/// This is passed back to C to indicate what changes were found on the line.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DifflineResult {
    /// Pointer to the first change for this line (may be null).
    pub changes: *const DiffLineChange,
    /// Number of changes for this line.
    pub num_changes: c_int,
    /// Buffer index in the diff list.
    pub bufidx: c_int,
    /// Line offset within the diff block.
    pub lineoff: c_int,
}

impl DifflineResult {
    /// Create an empty result.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            changes: std::ptr::null(),
            num_changes: 0,
            bufidx: -1,
            lineoff: 0,
        }
    }
}

// =============================================================================
// Static storage for simple diffline change
// =============================================================================

/// Simple diffline change - used when not using inline diff algorithm.
static mut SIMPLE_DIFFLINE_CHANGE: DiffLineChange = DiffLineChange::empty();

// =============================================================================
// Helper Functions
// =============================================================================

/// Compare two characters for diff equality, handling case sensitivity.
///
/// Returns true if characters are equal according to diff flags.
#[allow(clippy::cast_sign_loss)]
unsafe fn diff_equal_char(s1: *const c_char, s2: *const c_char, len: &mut c_int) -> bool {
    let diff_flags = nvim_get_diff_flags();
    let icase = (diff_flags & 0x004) != 0; // DIFF_ICASE

    let c1 = *s1 as u8;
    let c2 = *s2 as u8;

    *len = 1;

    // Simple ASCII comparison for single-byte chars
    if c1 < 0x80 && c2 < 0x80 {
        if icase {
            c1.eq_ignore_ascii_case(&c2)
        } else {
            c1 == c2
        }
    } else {
        // For multibyte, compare byte-by-byte (simplified)
        // In full implementation, would use utf_ptr2char and utf_fold
        c1 == c2
    }
}

/// Find the start and end positions of changes between two lines.
///
/// This implements the simple diff algorithm that compares characters
/// from both ends to find the changed region.
#[allow(clippy::cast_sign_loss)]
unsafe fn find_change_positions(
    line_org: *const c_char,
    line_new: *const c_char,
    startp: &mut ColnrT,
    endp: &mut ColnrT,
) {
    let diff_flags = nvim_get_diff_flags();

    // Search for start of difference
    let mut si_org: c_int = 0;
    let mut si_new: c_int = 0;

    while *line_org.offset(si_org as isize) != 0 {
        let c_org = *line_org.offset(si_org as isize) as u8;
        let c_new = *line_new.offset(si_new as isize) as u8;

        // Handle whitespace flags
        if ((diff_flags & DIFF_IWHITE) != 0 && ascii_iswhite(c_org) && ascii_iswhite(c_new))
            || ((diff_flags & DIFF_IWHITEALL) != 0
                && (ascii_iswhite(c_org) || ascii_iswhite(c_new)))
        {
            // Skip whitespace
            while *line_org.offset(si_org as isize) != 0
                && ascii_iswhite(*line_org.offset(si_org as isize) as u8)
            {
                si_org += 1;
            }
            while *line_new.offset(si_new as isize) != 0
                && ascii_iswhite(*line_new.offset(si_new as isize) as u8)
            {
                si_new += 1;
            }
        } else {
            let mut len = 0;
            if !diff_equal_char(
                line_org.offset(si_org as isize),
                line_new.offset(si_new as isize),
                &mut len,
            ) {
                break;
            }
            si_org += len;
            si_new += len;
        }
    }

    // Move back to first byte of character
    si_org -= utf_head_off(line_org, line_org.offset(si_org as isize));
    si_new -= utf_head_off(line_new, line_new.offset(si_new as isize));

    *startp = (*startp).min(si_org);

    // Search for end of difference
    if *line_org.offset(si_org as isize) != 0 || *line_new.offset(si_new as isize) != 0 {
        // Find string lengths
        let mut ei_org = 0;
        while *line_org.offset(ei_org as isize) != 0 {
            ei_org += 1;
        }

        let mut ei_new = 0;
        while *line_new.offset(ei_new as isize) != 0 {
            ei_new += 1;
        }

        // Search backwards for end of difference
        while ei_org >= *startp && ei_new >= si_new && ei_org >= 0 && ei_new >= 0 {
            let c_org = *line_org.offset(ei_org as isize) as u8;
            let c_new = *line_new.offset(ei_new as isize) as u8;

            if ((diff_flags & DIFF_IWHITE) != 0 && ascii_iswhite(c_org) && ascii_iswhite(c_new))
                || ((diff_flags & DIFF_IWHITEALL) != 0
                    && (ascii_iswhite(c_org) || ascii_iswhite(c_new)))
            {
                // Skip whitespace backwards
                while ei_org >= *startp && ascii_iswhite(*line_org.offset(ei_org as isize) as u8) {
                    ei_org -= 1;
                }
                while ei_new >= si_new && ascii_iswhite(*line_new.offset(ei_new as isize) as u8) {
                    ei_new -= 1;
                }
            } else {
                let mut len = 0;
                if !diff_equal_char(
                    line_org.offset(ei_org as isize),
                    line_new.offset(ei_new as isize),
                    &mut len,
                ) {
                    break;
                }
                ei_org -= len;
                ei_new -= len;
            }
        }

        *endp = (*endp).max(ei_org);
    }
}

// =============================================================================
// Main Entry Point
// =============================================================================

/// Find the difference within a changed line.
///
/// This is the main entry point for inline diff detection during rendering.
/// Returns true if the line was added (no other buffer has it).
///
/// # Safety
/// - `wp` must be a valid window handle
/// - `diffline` must be a valid pointer to write the result
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_diff_find_change(
    wp: WinHandle,
    lnum: LinenrT,
    dp: DiffBlockHandle,
    diffline: *mut DifflineResult,
) -> c_int {
    if wp.is_null() || dp.is_null() || diffline.is_null() {
        return 0;
    }

    let curtab = nvim_get_curtab();
    let buf = nvim_win_get_buffer(wp);
    let idx = nvim_diff_buf_idx(buf, curtab);

    if idx == DB_COUNT as c_int {
        return 0;
    }

    // Sanity check the diff block
    if rs_diff_check_sanity(curtab, dp) == FAIL {
        return 0;
    }

    let off = lnum - nvim_diffblock_get_lnum(dp, idx);
    let diff_flags = nvim_get_diff_flags();

    if (diff_flags & ALL_INLINE_DIFF) == 0 {
        // Use simple algorithm
        let mut change_start: ColnrT = MAXCOL;
        let mut change_end: ColnrT = -1;

        let added = diff_find_change_simple_impl(
            wp,
            buf,
            lnum,
            dp,
            idx,
            curtab,
            &mut change_start,
            &mut change_end,
        );

        // Convert from inclusive end to exclusive end
        change_end += 1;

        // Create a mock diffline struct
        SIMPLE_DIFFLINE_CHANGE = DiffLineChange::empty();
        SIMPLE_DIFFLINE_CHANGE.start[idx as usize] = change_start;
        SIMPLE_DIFFLINE_CHANGE.end[idx as usize] = change_end;
        SIMPLE_DIFFLINE_CHANGE.start_lnum_off[idx as usize] = off;
        SIMPLE_DIFFLINE_CHANGE.end_lnum_off[idx as usize] = off;

        (*diffline).changes = std::ptr::addr_of!(SIMPLE_DIFFLINE_CHANGE);
        (*diffline).num_changes = 1;
        (*diffline).bufidx = idx;
        (*diffline).lineoff = off;

        return c_int::from(added);
    }

    // Use inline diff algorithm
    // The inline diff requires calling diff_find_change_inline_diff() from C
    // to populate the changes. Check if changes are already cached.
    if !nvim_diffblock_has_changes(dp) {
        // Changes not computed yet - return indicating caller should use C path
        (*diffline).changes = std::ptr::null();
        (*diffline).num_changes = 0;
        (*diffline).bufidx = idx;
        (*diffline).lineoff = off;
        return -1; // Signal to caller to compute inline diff in C
    }

    let changes_len = nvim_diffblock_get_changes_len(dp);

    // Use linear search to find the first change for this line
    let mut num_changes = 0;
    let mut first_change: *const DiffLineChange = std::ptr::null();

    for change_idx in 0..changes_len {
        let change = nvim_diffblock_get_change(dp, change_idx);
        if change.is_null() {
            continue;
        }

        let end_lnum_off = (*change).end_lnum_off[idx as usize];
        let start_lnum_off = (*change).start_lnum_off[idx as usize];

        if end_lnum_off < off {
            continue;
        }
        if start_lnum_off > off {
            break;
        }
        if first_change.is_null() {
            first_change = change;
        }
        num_changes += 1;
    }

    (*diffline).changes = first_change;
    (*diffline).num_changes = num_changes;
    (*diffline).bufidx = idx;
    (*diffline).lineoff = off;

    // Detect simple cases of added lines
    let added = if num_changes == 1 {
        // Check if all other buffers consider this an addition
        let mut is_added = true;
        for i in 0..DB_COUNT {
            if i == idx {
                continue;
            }
            let other_buf = nvim_get_curtab_diffbuf(i);
            if other_buf.is_null() {
                continue;
            }
            // If any other buffer has content at this position, it's not purely added
            if !first_change.is_null() {
                let start = (*first_change).start[i as usize];
                let end = (*first_change).end[i as usize];
                if start < end {
                    is_added = false;
                    break;
                }
            }
        }
        is_added
    } else {
        false
    };

    c_int::from(added)
}

/// Simple diff algorithm implementation.
///
/// Compares line from current buffer against corresponding lines in other
/// diff buffers to find the changed region.
#[allow(clippy::too_many_arguments)]
unsafe fn diff_find_change_simple_impl(
    _wp: WinHandle,
    buf: BufHandle,
    lnum: LinenrT,
    dp: DiffBlockHandle,
    idx: c_int,
    _tp: TabpageHandle,
    startp: &mut ColnrT,
    endp: &mut ColnrT,
) -> bool {
    let diff_flags = nvim_get_diff_flags();

    // Get the original line
    let line_org_ptr = if (diff_flags & DIFF_INLINE_NONE) != 0 {
        // Only care about return value, not string comparisons
        std::ptr::null()
    } else {
        ml_get_buf(buf, lnum)
    };

    let off = lnum - nvim_diffblock_get_lnum(dp, idx);
    let mut added = true;

    for i in 0..DB_COUNT {
        let other_buf = nvim_get_curtab_diffbuf(i);
        if other_buf.is_null() || i == idx {
            continue;
        }

        // Skip lines not in the other change (filler lines)
        let other_count = nvim_diffblock_get_count(dp, i);
        if off >= other_count {
            continue;
        }

        added = false;

        if (diff_flags & DIFF_INLINE_NONE) != 0 {
            break; // Early terminate
        }

        let other_lnum = nvim_diffblock_get_lnum(dp, i) + off;
        let line_new = ml_get_buf(other_buf, other_lnum);

        if !line_org_ptr.is_null() && !line_new.is_null() {
            find_change_positions(line_org_ptr, line_new, startp, endp);
        }
    }

    added
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diffline_result_empty() {
        let result = DifflineResult::empty();
        assert!(result.changes.is_null());
        assert_eq!(result.num_changes, 0);
        assert_eq!(result.bufidx, -1);
        assert_eq!(result.lineoff, 0);
    }
}
