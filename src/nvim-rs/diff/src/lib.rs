//! Diff option checking for Neovim
//!
//! This module provides Rust implementations for checking diff options
//! from the 'diffopt' setting.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]

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

use std::ffi::c_void;

/// Maximum number of diff buffers (matches DB_COUNT in C).
pub const DB_COUNT: c_int = 8;

/// Opaque handle to a diff block (diff_T).
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct DiffBlockHandle(*mut c_void);

impl DiffBlockHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

/// Opaque handle to a buffer (buf_T).
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct BufHandle(*mut c_void);

impl BufHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

// C accessor for the static diff_flags variable
extern "C" {
    fn nvim_get_diff_flags() -> c_int;
    fn nvim_is_diffexpr_empty() -> bool;
    fn nvim_get_curtab_diffbuf(idx: c_int) -> BufHandle;
    fn nvim_get_curtab_diff_invalid() -> c_int;
    fn nvim_get_diff_first_block() -> DiffBlockHandle;
    fn nvim_diffblock_get_next(dp: DiffBlockHandle) -> DiffBlockHandle;
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
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
            if !diffbuf.is_null() && diffbuf.0 == buf.0 {
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
/// Returns the diff block handle or null if not found.
fn diff_find_block_for_line_impl(buf_idx: c_int, lnum: LinenrT) -> DiffBlockHandle {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return DiffBlockHandle::null();
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
        DiffBlockHandle::null()
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
pub extern "C" fn rs_diff_find_block_for_line(buf_idx: c_int, lnum: LinenrT) -> DiffBlockHandle {
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
            | DIFF_LINEMATCH;
        // Verify it's positive (no overflow from OR operations)
        assert!(all_flags > 0);
        // Verify expected combined value: all bits 0-12 set = 0x1FFF
        assert_eq!(all_flags, 0x1FFF);
    }

    #[test]
    fn test_diff_flag_count() {
        // There should be exactly 13 defined flags
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
        ];
        assert_eq!(flags.len(), 13);
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
            | DIFF_LINEMATCH;
        // trailing_zeros of all flags combined should be 0 (DIFF_FILLER is bit 0)
        assert_eq!(all_flags.trailing_zeros(), 0);
    }

    #[test]
    fn test_whitespace_flags_group() {
        // Test the ALL_WHITE_DIFF group
        let all_white = DIFF_IWHITE | DIFF_IWHITEALL | DIFF_IWHITEEOL;
        assert_eq!(all_white, 0x038); // bits 3, 4, 5
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
}
