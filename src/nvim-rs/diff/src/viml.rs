//! VimL function support for diff mode
//!
//! This module provides Rust implementations and helpers for diff-related
//! VimL functions. It handles:
//! - diff_filler() computation
//! - diff_hlID() highlight detection
//! - Caching for performance optimization

use std::ffi::c_int;
use std::ffi::c_void;

use crate::buffer::{DiffBlockHandle, WinHandle, DB_COUNT};

// Line number type matching linenr_T (i32)
type LinenrT = i32;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_next(dp: DiffBlockHandle) -> DiffBlockHandle;

    // Phase 1: xdiff_out and f_diff_filler accessors
    fn nvim_diffout_append_hunk(
        dout: *mut c_void,
        lnum_orig: LinenrT,
        count_orig: c_int,
        lnum_new: LinenrT,
        count_new: c_int,
    );
    #[link_name = "tv_get_lnum"]
    fn nvim_diff_tv_get_lnum(argvars: *mut c_void) -> LinenrT;
    fn nvim_get_curwin() -> WinHandle;
    fn rs_diff_check_fill(wp: WinHandle, lnum: LinenrT) -> c_int;
    fn nvim_fold_rettv_set_number(rettv: *mut c_void, nr: i64);

    // Phase 3: f_diff_hlID accessors
    fn nvim_get_diff_flags() -> c_int;
    fn nvim_curbuf_changedtick_i64() -> i64;
    fn nvim_curbuf_fnum() -> c_int;
    fn nvim_diff_tv_get_number_idx(argvars: *mut c_void, idx: c_int) -> c_int;
    fn nvim_diff_hlf_add() -> c_int;
    fn nvim_diff_hlf_chd() -> c_int;
    fn nvim_diff_hlf_txd() -> c_int;
    fn nvim_diff_hlf_txa() -> c_int;
    fn nvim_diff_diffline_get_change(dl: *mut c_void, i: c_int) -> *mut c_void;
    fn nvim_diffchange_get_start(dc: *mut c_void, idx: c_int) -> c_int;
    fn nvim_diffchange_get_end(dc: *mut c_void, idx: c_int) -> c_int;
    fn rs_diff_check_with_linestatus(wp: WinHandle, lnum: LinenrT, linestatus: *mut c_int)
        -> c_int;
    fn rs_diff_find_change(wp: WinHandle, lnum: LinenrT, diffline: *mut c_void) -> bool;
    fn rs_diff_change_parse(
        diffline: *mut c_void,
        change: *mut c_void,
        change_start: *mut c_int,
        change_end: *mut c_int,
    ) -> bool;
}

// =============================================================================
// diff_filler() support
// =============================================================================

/// Result of diff_filler calculation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffFillerResult {
    /// Number of filler lines at this position
    pub count: c_int,
    /// Whether the result is valid
    pub valid: bool,
}

/// Calculate filler lines for a position.
///
/// This provides the core computation for the diff_filler() VimL function.
/// Returns the number of virtual filler lines above the given line number.
///
/// # Safety
/// `first_dp` must be a valid diff block handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_filler_at_line(
    first_dp: DiffBlockHandle,
    buf_idx: c_int,
    lnum: LinenrT,
) -> DiffFillerResult {
    if first_dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int || lnum <= 0 {
        return DiffFillerResult::default();
    }

    let mut dp = first_dp;
    while !dp.is_null() {
        let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
        let block_count = nvim_diffblock_get_count(dp, buf_idx);

        // If we're past this block, continue
        if block_lnum + block_count <= lnum && block_count > 0 {
            dp = nvim_diffblock_get_next(dp);
            continue;
        }

        // If we're at the start of a block
        if lnum == block_lnum {
            // Find the maximum count across all buffers
            let mut max_count: LinenrT = 0;
            for i in 0..DB_COUNT as c_int {
                let count = nvim_diffblock_get_count(dp, i);
                if count > max_count {
                    max_count = count;
                }
            }

            // Filler count is max minus our count
            let filler = max_count - block_count;
            return DiffFillerResult {
                count: filler.max(0),
                valid: true,
            };
        }

        // Past this block
        break;
    }

    DiffFillerResult {
        count: 0,
        valid: true,
    }
}

// =============================================================================
// diff_hlID() support
// =============================================================================

/// Highlight ID types for diff mode.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffHlType {
    /// No highlighting
    #[default]
    None = 0,
    /// Added line (DiffAdd)
    Add = 1,
    /// Changed line (DiffChange)
    Change = 2,
    /// Deleted line (DiffDelete)
    Delete = 3,
    /// Changed text within line (DiffText)
    Text = 4,
}

/// Result of diff_hlID calculation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffHlIdResult {
    /// The highlight type
    pub hl_type: DiffHlType,
    /// Start column of changed text (if Text type)
    pub change_start: c_int,
    /// End column of changed text (if Text type)
    pub change_end: c_int,
    /// Whether result is valid
    pub valid: bool,
}

/// State for caching diff_hlID results.
///
/// The diff_hlID() function is called frequently during screen updates,
/// so caching the last result significantly improves performance.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffHlIdCache {
    /// Previously queried line number
    pub prev_lnum: LinenrT,
    /// Buffer's changedtick when cached
    pub changedtick: i64,
    /// Buffer file number
    pub fnum: c_int,
    /// Diff flags when cached
    pub diff_flags: c_int,
    /// Cached change start column
    pub change_start: c_int,
    /// Cached change end column
    pub change_end: c_int,
    /// Cached highlight type
    pub hl_type: DiffHlType,
    /// Whether cache is valid
    pub valid: bool,
}

impl Default for DiffHlIdCache {
    fn default() -> Self {
        Self {
            prev_lnum: 0,
            changedtick: 0,
            fnum: 0,
            diff_flags: 0,
            change_start: 0,
            change_end: 0,
            hl_type: DiffHlType::None,
            valid: false,
        }
    }
}

/// Initialize diff_hlID cache.
#[no_mangle]
pub extern "C" fn rs_diff_hlid_cache_init() -> DiffHlIdCache {
    DiffHlIdCache::default()
}

/// Check if cache is valid for current query.
#[no_mangle]
pub const extern "C" fn rs_diff_hlid_cache_valid(
    cache: &DiffHlIdCache,
    lnum: LinenrT,
    changedtick: i64,
    fnum: c_int,
    diff_flags: c_int,
    use_cache: bool,
) -> bool {
    use_cache
        && cache.valid
        && cache.prev_lnum == lnum
        && cache.changedtick == changedtick
        && cache.fnum == fnum
        && cache.diff_flags == diff_flags
}

/// Update cache with new values.
#[no_mangle]
pub const extern "C" fn rs_diff_hlid_cache_update(
    cache: &mut DiffHlIdCache,
    lnum: LinenrT,
    changedtick: i64,
    fnum: c_int,
    diff_flags: c_int,
    change_start: c_int,
    change_end: c_int,
    hl_type: DiffHlType,
) {
    cache.prev_lnum = lnum;
    cache.changedtick = changedtick;
    cache.fnum = fnum;
    cache.diff_flags = diff_flags;
    cache.change_start = change_start;
    cache.change_end = change_end;
    cache.hl_type = hl_type;
    cache.valid = true;
}

/// Get cached highlight result.
#[no_mangle]
pub const extern "C" fn rs_diff_hlid_cache_get(cache: &DiffHlIdCache) -> DiffHlIdResult {
    DiffHlIdResult {
        hl_type: cache.hl_type,
        change_start: cache.change_start,
        change_end: cache.change_end,
        valid: cache.valid,
    }
}

/// Invalidate cache.
#[no_mangle]
pub const extern "C" fn rs_diff_hlid_cache_invalidate(cache: &mut DiffHlIdCache) {
    cache.valid = false;
}

// =============================================================================
// Highlight ID Resolution
// =============================================================================

/// Determine highlight type from line status.
///
/// Given the result of diff_check_with_linestatus, determine the
/// appropriate highlight type.
#[no_mangle]
pub const extern "C" fn rs_diff_hlid_from_linestatus(
    linestatus: c_int,
    has_change_start: bool,
) -> DiffHlType {
    if linestatus < 0 {
        // Deleted or changed line
        if linestatus == -1 {
            // Changed line with potential text change
            if has_change_start {
                DiffHlType::Text
            } else {
                DiffHlType::Change
            }
        } else {
            // -2 means deleted (filler lines)
            DiffHlType::Delete
        }
    } else if linestatus > 0 {
        // Added line (filler count > 0)
        DiffHlType::Add
    } else {
        DiffHlType::None
    }
}

/// Determine highlight type for a column position.
///
/// Given the line status and column, determine if this specific
/// position should have DiffText highlighting.
#[no_mangle]
pub extern "C" fn rs_diff_hlid_for_column(
    col: c_int,
    change_start: c_int,
    change_end: c_int,
    base_hl: DiffHlType,
) -> DiffHlType {
    // If we have a text change region and column is within it
    if base_hl == DiffHlType::Change && change_start <= col && col < change_end {
        DiffHlType::Text
    } else {
        base_hl
    }
}

// =============================================================================
// Line Information for VimL
// =============================================================================

/// Comprehensive line information for VimL functions.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffLineVimInfo {
    /// Filler lines above this line
    pub filler_above: c_int,
    /// Whether line is in a diff block
    pub in_diff_block: bool,
    /// Whether line was added
    pub is_added: bool,
    /// Whether line was changed
    pub is_changed: bool,
    /// Whether line corresponds to deleted lines in other buffer
    pub is_deleted: bool,
    /// Change start column (-1 if whole line or none)
    pub change_start: c_int,
    /// Change end column (-1 if whole line or none)
    pub change_end: c_int,
}

/// Get comprehensive line information for VimL functions.
///
/// This combines the information needed by both diff_filler() and diff_hlID()
/// into a single query.
///
/// # Safety
/// `first_dp` must be a valid diff block handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_get_line_vim_info(
    first_dp: DiffBlockHandle,
    buf_idx: c_int,
    lnum: LinenrT,
    linestatus: c_int,
    change_start: c_int,
    change_end: c_int,
) -> DiffLineVimInfo {
    if first_dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int || lnum <= 0 {
        return DiffLineVimInfo::default();
    }

    let mut info = DiffLineVimInfo {
        change_start,
        change_end,
        ..Default::default()
    };

    // Find the block containing this line
    let mut dp = first_dp;
    while !dp.is_null() {
        let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
        let block_count = nvim_diffblock_get_count(dp, buf_idx);
        let block_end = block_lnum + block_count;

        if lnum >= block_lnum && (lnum < block_end || block_count == 0) {
            info.in_diff_block = true;

            // Calculate filler lines at block start
            if lnum == block_lnum {
                let mut max_count: LinenrT = 0;
                for i in 0..DB_COUNT as c_int {
                    let count = nvim_diffblock_get_count(dp, i);
                    if count > max_count {
                        max_count = count;
                    }
                }
                info.filler_above = (max_count - block_count).max(0);
            }

            break;
        }

        if block_lnum > lnum {
            break;
        }

        dp = nvim_diffblock_get_next(dp);
    }

    // Interpret linestatus
    match linestatus.cmp(&0) {
        std::cmp::Ordering::Less => {
            if linestatus == -1 {
                info.is_changed = true;
            } else {
                info.is_deleted = true;
            }
        }
        std::cmp::Ordering::Greater => {
            info.is_added = true;
        }
        std::cmp::Ordering::Equal => {}
    }

    info
}

// =============================================================================
// diffline_T layout (matches C buffer_defs.h DifflineRepr)
// =============================================================================

/// Mirror of C diffline_T struct layout.
/// Must match: { diffline_change_T *changes; int num_changes; int bufidx; int lineoff; }
#[repr(C)]
struct DifflineRepr {
    changes: *mut c_void,
    num_changes: c_int,
    bufidx: c_int,
    lineoff: c_int,
}

// ALL_INLINE_DIFF flag (must match C definition in diff_shim.c)
// ALL_INLINE_DIFF = DIFF_INLINE_CHAR | DIFF_INLINE_WORD = 0x8000 | 0x10000
const ALL_INLINE_DIFF: c_int = 0x18000;

// MAXCOL value (matches C pos_defs.h)
const MAXCOL: c_int = 0x7fff_ffff;

// =============================================================================
// Phase 1 Migrations: xdiff_out callback and f_diff_filler
// =============================================================================

/// xdiff callback: appends a diff hunk to the diffout_T grow array.
///
/// This is the Rust implementation of the xdiff_out callback previously in C.
/// Called by xdl_diff for each diff hunk found.
///
/// # Safety
/// `priv_data` must be a valid pointer to a diffout_T (passed as void*).
#[no_mangle]
pub unsafe extern "C" fn rs_xdiff_out(
    start_a: c_int,
    count_a: c_int,
    start_b: c_int,
    count_b: c_int,
    priv_data: *mut c_void,
) -> c_int {
    nvim_diffout_append_hunk(
        priv_data,
        start_a + 1, // convert 0-based to 1-based line numbers
        count_a,
        start_b + 1,
        count_b,
    );
    0
}

/// VimL diff_filler() function -- returns number of filler lines above {lnum}.
///
/// This is the Rust implementation of f_diff_filler previously in C.
///
/// # Safety
/// `argvars` and `rettv` must be valid pointers to typval_T values.
#[export_name = "f_diff_filler"]
pub unsafe extern "C" fn rs_f_diff_filler(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let curwin = nvim_get_curwin();
    let lnum = nvim_diff_tv_get_lnum(argvars);
    let fill = rs_diff_check_fill(curwin, lnum);
    nvim_fold_rettv_set_number(rettv, i64::from(fill.max(0)));
}

// =============================================================================
// Phase 3 Migration: f_diff_hlID
// =============================================================================

// Static cache for f_diff_hlID -- mirrors C static local variables.
// SAFETY: Neovim is single-threaded; these are equivalent to C file-scope statics.
static mut HLID_PREV_LNUM: LinenrT = 0;
static mut HLID_CHANGEDTICK: i64 = 0;
static mut HLID_FNUM: c_int = 0;
static mut HLID_PREV_DIFF_FLAGS: c_int = 0;
static mut HLID_CHANGE_START: c_int = 0;
static mut HLID_CHANGE_END: c_int = 0;
static mut HLID_VALUE: c_int = 0;

/// VimL diff_hlID() function handler -- Rust implementation.
///
/// Contains static caching state matching the original C statics.
///
/// # Safety
/// Accesses global state and static mut variables. Single-threaded Neovim only.
#[export_name = "f_diff_hlID"]
#[allow(static_mut_refs, non_snake_case)]
pub unsafe extern "C" fn rs_f_diff_hl_id(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let hlf_add = nvim_diff_hlf_add();
    let hlf_chd = nvim_diff_hlf_chd();
    let hlf_text_changed = nvim_diff_hlf_txd();
    let hlf_text_added = nvim_diff_hlf_txa();

    let diff_flags = nvim_get_diff_flags();
    // cache_results = !(diff_flags & ALL_INLINE_DIFF)
    let cache_results = (diff_flags & ALL_INLINE_DIFF) == 0;

    let mut lnum = nvim_diff_tv_get_lnum(argvars);
    if lnum < 0 {
        lnum = 0;
    }

    let curwin = nvim_get_curwin();
    let cur_changedtick = nvim_curbuf_changedtick_i64();
    let cur_fnum = nvim_curbuf_fnum();

    let need_recompute = !cache_results
        || lnum != HLID_PREV_LNUM
        || cur_changedtick != HLID_CHANGEDTICK
        || cur_fnum != HLID_FNUM
        || diff_flags != HLID_PREV_DIFF_FLAGS;

    // diffline is allocated on the stack for use in both the recompute branch
    // and the non-cache HLF_CHD column-check branch below.
    let mut diffline = DifflineRepr {
        changes: std::ptr::null_mut(),
        num_changes: 0,
        bufidx: 0,
        lineoff: 0,
    };
    let diffline_ptr: *mut c_void = std::ptr::addr_of_mut!(diffline).cast();

    if need_recompute {
        let mut linestatus: c_int = 0;
        rs_diff_check_with_linestatus(curwin, lnum, std::ptr::addr_of_mut!(linestatus));

        if linestatus < 0 {
            if linestatus == -1 {
                HLID_CHANGE_START = MAXCOL;
                HLID_CHANGE_END = -1;
                if rs_diff_find_change(curwin, lnum, diffline_ptr) {
                    HLID_VALUE = hlf_add; // added line
                } else {
                    HLID_VALUE = hlf_chd; // changed line
                    if diffline.num_changes > 0 && cache_results {
                        let bufidx = diffline.bufidx;
                        let change0 = nvim_diff_diffline_get_change(diffline_ptr, 0);
                        if !change0.is_null() {
                            HLID_CHANGE_START = nvim_diffchange_get_start(change0, bufidx);
                            HLID_CHANGE_END = nvim_diffchange_get_end(change0, bufidx);
                        }
                    }
                }
            } else {
                HLID_VALUE = hlf_add; // added line (filler)
            }
        } else {
            HLID_VALUE = 0; // no diff
        }

        if cache_results {
            HLID_PREV_LNUM = lnum;
            HLID_CHANGEDTICK = cur_changedtick;
            HLID_FNUM = cur_fnum;
            HLID_PREV_DIFF_FLAGS = diff_flags;
        }
    }

    if HLID_VALUE == hlf_chd || HLID_VALUE == hlf_text_changed {
        let col = nvim_diff_tv_get_number_idx(argvars, 1) - 1;

        if cache_results {
            if col >= HLID_CHANGE_START && col < HLID_CHANGE_END {
                HLID_VALUE = hlf_text_changed; // Changed text
            } else {
                HLID_VALUE = hlf_chd; // Changed line
            }
        } else {
            // Non-cache path: iterate over diffline.changes.
            // diffline was populated above in the need_recompute block
            // (since !cache_results implies need_recompute=true).
            HLID_VALUE = hlf_chd;
            let num_changes = diffline.num_changes;
            for i in 0..num_changes {
                let change = nvim_diff_diffline_get_change(diffline_ptr, i);
                if change.is_null() {
                    break;
                }
                let mut cs: c_int = 0;
                let mut ce: c_int = 0;
                let added = rs_diff_change_parse(
                    diffline_ptr,
                    change,
                    std::ptr::addr_of_mut!(cs),
                    std::ptr::addr_of_mut!(ce),
                );
                if col >= cs && col < ce {
                    HLID_VALUE = if added {
                        hlf_text_added
                    } else {
                        hlf_text_changed
                    };
                    break;
                }
                if col < cs {
                    // remaining changes are past this column
                    break;
                }
            }
        }
    }

    nvim_fold_rettv_set_number(rettv, i64::from(HLID_VALUE));
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_filler_result_default() {
        let result = DiffFillerResult::default();
        assert_eq!(result.count, 0);
        assert!(!result.valid);
    }

    #[test]
    fn test_diff_hl_type_values() {
        assert_eq!(DiffHlType::None as c_int, 0);
        assert_eq!(DiffHlType::Add as c_int, 1);
        assert_eq!(DiffHlType::Change as c_int, 2);
        assert_eq!(DiffHlType::Delete as c_int, 3);
        assert_eq!(DiffHlType::Text as c_int, 4);
    }

    #[test]
    fn test_diff_hlid_cache_init() {
        let cache = rs_diff_hlid_cache_init();
        assert_eq!(cache.prev_lnum, 0);
        assert!(!cache.valid);
    }

    #[test]
    fn test_diff_hlid_cache_valid() {
        let mut cache = DiffHlIdCache::default();

        // Invalid cache should return false
        assert!(!rs_diff_hlid_cache_valid(&cache, 10, 1, 1, 0, true));

        // Update cache
        rs_diff_hlid_cache_update(&mut cache, 10, 1, 1, 0, 5, 15, DiffHlType::Change);

        // Same values should return true
        assert!(rs_diff_hlid_cache_valid(&cache, 10, 1, 1, 0, true));

        // Different line should return false
        assert!(!rs_diff_hlid_cache_valid(&cache, 11, 1, 1, 0, true));

        // Different changedtick should return false
        assert!(!rs_diff_hlid_cache_valid(&cache, 10, 2, 1, 0, true));

        // use_cache=false should return false
        assert!(!rs_diff_hlid_cache_valid(&cache, 10, 1, 1, 0, false));
    }

    #[test]
    fn test_diff_hlid_cache_update_and_get() {
        let mut cache = DiffHlIdCache::default();

        rs_diff_hlid_cache_update(&mut cache, 100, 50, 2, 0x100, 10, 20, DiffHlType::Text);

        assert_eq!(cache.prev_lnum, 100);
        assert_eq!(cache.changedtick, 50);
        assert_eq!(cache.fnum, 2);
        assert_eq!(cache.diff_flags, 0x100);
        assert!(cache.valid);

        let result = rs_diff_hlid_cache_get(&cache);
        assert_eq!(result.hl_type, DiffHlType::Text);
        assert_eq!(result.change_start, 10);
        assert_eq!(result.change_end, 20);
        assert!(result.valid);
    }

    #[test]
    fn test_diff_hlid_cache_invalidate() {
        let mut cache = DiffHlIdCache::default();
        rs_diff_hlid_cache_update(&mut cache, 10, 1, 1, 0, 0, 0, DiffHlType::Add);
        assert!(cache.valid);

        rs_diff_hlid_cache_invalidate(&mut cache);
        assert!(!cache.valid);
    }

    #[test]
    fn test_diff_hlid_from_linestatus() {
        // Deleted/filler lines
        assert_eq!(rs_diff_hlid_from_linestatus(-2, false), DiffHlType::Delete);

        // Changed line without text change
        assert_eq!(rs_diff_hlid_from_linestatus(-1, false), DiffHlType::Change);

        // Changed line with text change
        assert_eq!(rs_diff_hlid_from_linestatus(-1, true), DiffHlType::Text);

        // Added lines
        assert_eq!(rs_diff_hlid_from_linestatus(1, false), DiffHlType::Add);
        assert_eq!(rs_diff_hlid_from_linestatus(5, false), DiffHlType::Add);

        // No diff
        assert_eq!(rs_diff_hlid_from_linestatus(0, false), DiffHlType::None);
    }

    #[test]
    fn test_diff_hlid_for_column() {
        // Column outside change region
        assert_eq!(
            rs_diff_hlid_for_column(5, 10, 20, DiffHlType::Change),
            DiffHlType::Change
        );

        // Column inside change region
        assert_eq!(
            rs_diff_hlid_for_column(15, 10, 20, DiffHlType::Change),
            DiffHlType::Text
        );

        // Column at start of change region
        assert_eq!(
            rs_diff_hlid_for_column(10, 10, 20, DiffHlType::Change),
            DiffHlType::Text
        );

        // Column at end of change region (exclusive)
        assert_eq!(
            rs_diff_hlid_for_column(20, 10, 20, DiffHlType::Change),
            DiffHlType::Change
        );

        // Non-change base type should not become Text
        assert_eq!(
            rs_diff_hlid_for_column(15, 10, 20, DiffHlType::Add),
            DiffHlType::Add
        );
    }

    #[test]
    fn test_diff_line_vim_info_default() {
        let info = DiffLineVimInfo::default();
        assert_eq!(info.filler_above, 0);
        assert!(!info.in_diff_block);
        assert!(!info.is_added);
        assert!(!info.is_changed);
        assert!(!info.is_deleted);
        assert_eq!(info.change_start, 0);
        assert_eq!(info.change_end, 0);
    }
}
