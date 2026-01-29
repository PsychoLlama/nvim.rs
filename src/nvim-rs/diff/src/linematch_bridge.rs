//! Linematch algorithm bridge for diff block alignment
//!
//! This module bridges the linematch crate with diff block management,
//! providing functions to run the linematch algorithm on diff blocks
//! and apply the results by subdividing blocks.

use std::os::raw::c_int;

use libc::c_long;

use crate::buffer::{BufHandle, DiffBlockHandle, TabpageHandle, DB_COUNT};

/// Line number type matching linenr_T (i32).
type LinenrT = i32;

// =============================================================================
// Diff Flags
// =============================================================================

const DIFF_IWHITE: c_int = 0x008;
const DIFF_IWHITEALL: c_int = 0x010;

// =============================================================================
// External Types
// =============================================================================

/// Memory-mapped file type (matches xdiff's `mmfile_t`)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MmFile {
    pub ptr: *mut i8,
    pub size: c_long,
}

impl Default for MmFile {
    fn default() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            size: 0,
        }
    }
}

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_diff_flags() -> c_int;
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_get_curtab_diffbuf(idx: c_int) -> BufHandle;

    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_set_lnum(dp: DiffBlockHandle, idx: c_int, lnum: LinenrT);
    fn nvim_diffblock_set_count(dp: DiffBlockHandle, idx: c_int, count: LinenrT);
    fn nvim_diffblock_set_linematched(dp: DiffBlockHandle, val: bool);
    fn nvim_diffblock_get_next(dp: DiffBlockHandle) -> DiffBlockHandle;

    fn nvim_diff_alloc_new(
        tp: TabpageHandle,
        prev: DiffBlockHandle,
        next: DiffBlockHandle,
    ) -> DiffBlockHandle;

    fn nvim_diff_write_buffer(
        buf: BufHandle,
        mm: *mut std::ffi::c_void,
        start: LinenrT,
        end: LinenrT,
    );

    // Linematch algorithm
    fn linematch_nbuffers(
        diffbufs: *const *const MmFile,
        diff_length: *const c_int,
        ndiffs: usize,
        decisions: *mut *mut c_int,
        iwhite: bool,
    ) -> usize;

    // Memory functions
    fn xfree(ptr: *mut std::ffi::c_void);
}

// =============================================================================
// Linematch Results Application
// =============================================================================

/// State for tracking line numbers during result application.
struct LineNumberState {
    line_numbers: [LinenrT; DB_COUNT as usize],
    outputmap: [c_int; DB_COUNT as usize],
    ndiffs: usize,
}

impl LineNumberState {
    /// Initialize state from a diff block.
    ///
    /// # Safety
    /// `dp` must be valid.
    #[allow(clippy::cast_sign_loss)]
    unsafe fn from_diff_block(dp: DiffBlockHandle, tp: TabpageHandle) -> Self {
        let mut state = Self {
            line_numbers: [0; DB_COUNT as usize],
            outputmap: [0; DB_COUNT as usize],
            ndiffs: 0,
        };

        for i in 0..DB_COUNT {
            let buf = nvim_get_curtab_diffbuf(i);
            if !buf.is_null() {
                state.line_numbers[i as usize] = nvim_diffblock_get_lnum(dp, i);
                nvim_diffblock_set_count(dp, i, 0);
                state.outputmap[state.ndiffs] = i;
                state.ndiffs += 1;
            }
        }

        let _ = tp; // Mark as used
        state
    }
}

/// Apply results from the linematch algorithm to a diff block.
///
/// This subdivides the original diff block into multiple adjacent blocks
/// based on the linematch decisions.
///
/// # Safety
/// - `dp` must be a valid diff block handle
/// - `decisions` must be a valid pointer with `decisions_length` elements
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_apply_linematch_results(
    dp: DiffBlockHandle,
    decisions_length: usize,
    decisions: *const c_int,
) {
    if dp.is_null() || decisions.is_null() || decisions_length == 0 {
        return;
    }

    let tp = nvim_get_curtab();
    let mut state = LineNumberState::from_diff_block(dp, tp);

    // Write diffs starting with the current diff block
    let mut dp_s = dp;

    for i in 0..decisions_length {
        let current_decision = *decisions.add(i);

        // Don't allocate on first iter since we can reuse the initial diffblock
        if i != 0 {
            let prev_decision = *decisions.add(i - 1);
            if prev_decision != current_decision {
                // Create new sub diff block
                let next = nvim_diffblock_get_next(dp_s);
                dp_s = nvim_diff_alloc_new(tp, dp_s, next);
                if dp_s.is_null() {
                    break;
                }
                nvim_diffblock_set_linematched(dp_s, true);

                // Initialize line numbers for new block
                for j in 0..DB_COUNT {
                    let buf = nvim_get_curtab_diffbuf(j);
                    if !buf.is_null() {
                        nvim_diffblock_set_lnum(dp_s, j, state.line_numbers[j as usize]);
                        nvim_diffblock_set_count(dp_s, j, 0);
                    }
                }
            }
        }

        // Update counts based on decision bitmask
        for j in 0..state.ndiffs {
            if (current_decision & (1 << j)) != 0 {
                let buf_idx = state.outputmap[j];
                let count = nvim_diffblock_get_count(dp_s, buf_idx);
                nvim_diffblock_set_count(dp_s, buf_idx, count + 1);
                state.line_numbers[buf_idx as usize] += 1;
            }
        }
    }

    nvim_diffblock_set_linematched(dp, true);
}

// =============================================================================
// Linematch Algorithm Execution
// =============================================================================

/// Run the linematch algorithm on a diff block.
///
/// This extracts buffer contents, runs the algorithm, and applies results.
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_run_linematch(dp: DiffBlockHandle) {
    if dp.is_null() {
        return;
    }

    let _tp = nvim_get_curtab();
    let diff_flags = nvim_get_diff_flags();

    // Define buffers for diff algorithm
    let mut diffbufs_mm: [MmFile; DB_COUNT as usize] = [MmFile::default(); DB_COUNT as usize];
    let mut diffbufs: [*const MmFile; DB_COUNT as usize] = [std::ptr::null(); DB_COUNT as usize];
    let mut diff_length: [c_int; DB_COUNT as usize] = [0; DB_COUNT as usize];
    let mut ndiffs: usize = 0;

    // Collect buffer contents
    for i in 0..DB_COUNT {
        let buf = nvim_get_curtab_diffbuf(i);
        if !buf.is_null() {
            let count = nvim_diffblock_get_count(dp, i);
            if count > 0 {
                let lnum = nvim_diffblock_get_lnum(dp, i);
                nvim_diff_write_buffer(
                    buf,
                    std::ptr::addr_of_mut!(diffbufs_mm[ndiffs]).cast(),
                    lnum,
                    lnum + count - 1,
                );
            } else {
                diffbufs_mm[ndiffs].size = 0;
                diffbufs_mm[ndiffs].ptr = std::ptr::null_mut();
            }

            diffbufs[ndiffs] = std::ptr::addr_of!(diffbufs_mm[ndiffs]);
            diff_length[ndiffs] = count;
            ndiffs += 1;
        }
    }

    if ndiffs < 2 {
        // Need at least 2 buffers
        return;
    }

    // Run linematch algorithm
    let mut decisions: *mut c_int = std::ptr::null_mut();
    let iwhite = (diff_flags & (DIFF_IWHITEALL | DIFF_IWHITE)) != 0;

    let decisions_length = linematch_nbuffers(
        diffbufs.as_ptr(),
        diff_length.as_ptr(),
        ndiffs,
        std::ptr::addr_of_mut!(decisions),
        iwhite,
    );

    // Clean up buffer memory
    for mm in diffbufs_mm.iter().take(ndiffs) {
        if !mm.ptr.is_null() {
            xfree(mm.ptr.cast());
        }
    }

    // Apply results
    if !decisions.is_null() && decisions_length > 0 {
        rs_apply_linematch_results(dp, decisions_length, decisions);
        xfree(decisions.cast());
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mmfile_default() {
        let mm = MmFile::default();
        assert!(mm.ptr.is_null());
        assert_eq!(mm.size, 0);
    }
}
