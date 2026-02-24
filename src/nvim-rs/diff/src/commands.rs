//! Diff Ex command implementations
//!
//! This module provides Rust implementations for diff-related Ex commands
//! like :diffget, :diffput, and related operations.

#![allow(clippy::must_use_candidate)]

use std::ffi::c_void;
use std::os::raw::c_int;

use crate::buffer::{BufHandle, DiffBlockHandle, WinHandle, DB_COUNT};

/// Line number type matching linenr_T (i32).
type LinenrT = i32;

/// Result constants.
const OK: c_int = 1;
const FAIL: c_int = 0;

// =============================================================================
// External C Functions
// =============================================================================

use crate::buffer::TabpageHandle;
use std::ffi::c_char;

#[allow(dead_code)]
extern "C" {
    fn nvim_get_curtab_diffbuf(idx: c_int) -> BufHandle;
    fn nvim_get_diff_first_block() -> DiffBlockHandle;
    fn nvim_diffblock_get_next(dp: DiffBlockHandle) -> DiffBlockHandle;
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> LinenrT;

    // Phase 2: nv_diffgetput and ex_diffthis accessors
    fn nvim_bt_prompt_curbuf() -> bool;
    fn nvim_vim_beep_operator();
    fn nvim_get_curwin_cursor_lnum() -> c_int;
    fn nvim_docmd_cmd_diffget() -> c_int;
    fn nvim_docmd_cmd_diffput() -> c_int;
    fn nvim_get_curwin() -> WinHandle;

    // Phase 3: ex_diffgetput accessors
    fn nvim_diff_emsg_e99();
    fn nvim_diff_emsg_e793();
    fn nvim_diff_emsg_e100();
    fn nvim_diff_emsg_e101();
    fn nvim_diff_semsg_e102(arg: *const c_char);
    fn nvim_diff_semsg_e103(arg: *const c_char);
    fn nvim_diff_emsg_e787();
    fn nvim_diff_buf_is_modifiable(buf: BufHandle) -> bool;
    fn nvim_diff_get_curbuf() -> BufHandle;
    fn nvim_diff_buflist_findpat(arg: *const c_char, end: *const c_char) -> c_int;
    fn nvim_diff_buflist_findnr(nr: c_int) -> BufHandle;
    fn nvim_diff_aucmd_prepbuf_idx(idx: c_int);
    fn nvim_diff_aucmd_restbuf();
    fn nvim_diff_change_warning_curbuf();
    fn nvim_diff_curbuf_changed() -> bool;
    fn nvim_diff_key_typed() -> bool;
    fn nvim_diff_u_sync();
    fn nvim_diff_check_cursor_curwin();
    fn nvim_diff_changed_line_abv_curs();
    fn nvim_diff_call_diffgetput(
        addr_count: c_int,
        idx_cur: c_int,
        idx_from: c_int,
        idx_to: c_int,
        line1: LinenrT,
        line2: LinenrT,
    );
    fn nvim_diff_get_CMD_diffget() -> c_int;
    fn nvim_diff_get_CMD_diffput() -> c_int;
    fn nvim_diff_curbuf_ml_line_count() -> LinenrT;
    fn nvim_diff_curtab_first_diff_is_null() -> bool;
    fn nvim_diff_win_get_w_p_fdm_starts_d(wp: WinHandle) -> bool;
    fn nvim_diff_get_curtab_diffbuf_idx(idx: c_int) -> BufHandle;
    fn nvim_diff_curbuf_is_curtab_diffbuf(idx_to: c_int) -> bool;
    fn nvim_diff_fire_diffupdated_curbuf();
    fn nvim_diff_set_busy(val: bool);
    fn nvim_diff_get_need_update() -> bool;
    fn nvim_diff_set_need_update(val: bool);
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_win_get_p_diff(wp: WinHandle) -> c_int;
    fn nvim_tabpage_first_win(tp: TabpageHandle) -> WinHandle;
    fn nvim_win_next(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_w_p_fen(wp: WinHandle) -> bool;
    fn rs_foldUpdateAll(wp: WinHandle);
    fn rs_diff_redraw(dofold: bool);
    fn rs_diff_ex_diffupdate(eap: *const c_void);
    fn nvim_eap_get_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_eap_get_cmdidx(eap: *const c_void) -> c_int;
    fn nvim_eap_get_addr_count(eap: *const c_void) -> c_int;
    fn nvim_eap_get_line1(eap: *const c_void) -> LinenrT;
    fn nvim_eap_get_line2(eap: *const c_void) -> LinenrT;
    fn nvim_eap_set_line1(eap: *mut c_void, line: LinenrT);
    fn nvim_eap_set_line2(eap: *mut c_void, line: LinenrT);
    fn nvim_diff_call_nv_ex_diffgetput(
        cmdidx: c_int,
        arg: *const c_char,
        addr_count: c_int,
        line1: LinenrT,
        line2: LinenrT,
    );
}

// =============================================================================
// Diff Get/Put Operations
// =============================================================================

/// Operation type for diffget/diffput.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffOperation {
    /// Get changes from another buffer.
    Get = 0,
    /// Put changes to another buffer.
    Put = 1,
}

/// Result of a diff get/put operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffOpResult {
    /// Status of the operation (OK or FAIL).
    pub status: c_int,
    /// Number of lines affected.
    pub lines_changed: LinenrT,
    /// First line affected.
    pub first_line: LinenrT,
    /// Last line affected.
    pub last_line: LinenrT,
}

impl DiffOpResult {
    /// Create a failed result.
    #[must_use]
    pub const fn fail() -> Self {
        Self {
            status: FAIL,
            lines_changed: 0,
            first_line: 0,
            last_line: 0,
        }
    }

    /// Create a success result.
    #[must_use]
    pub const fn success(lines: LinenrT, first: LinenrT, last: LinenrT) -> Self {
        Self {
            status: OK,
            lines_changed: lines,
            first_line: first,
            last_line: last,
        }
    }
}

/// Range for diff operations.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffRange {
    /// First line (inclusive).
    pub first: LinenrT,
    /// Last line (inclusive).
    pub last: LinenrT,
}

impl DiffRange {
    /// Create a new range.
    #[must_use]
    pub const fn new(first: LinenrT, last: LinenrT) -> Self {
        Self { first, last }
    }

    /// Create an empty range.
    #[must_use]
    pub const fn empty() -> Self {
        Self { first: 0, last: 0 }
    }

    /// Check if the range is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.first > 0 && self.last >= self.first
    }

    /// Get the number of lines in the range.
    #[must_use]
    pub const fn count(&self) -> LinenrT {
        if self.is_valid() {
            self.last - self.first + 1
        } else {
            0
        }
    }
}

// =============================================================================
// Diff Block Selection
// =============================================================================

/// Find the diff block(s) that overlap with a line range.
///
/// Returns the first and last diff blocks that overlap with the range.
pub fn diff_find_blocks_in_range(
    buf_idx: c_int,
    range: DiffRange,
) -> (DiffBlockHandle, DiffBlockHandle) {
    if !(0..DB_COUNT).contains(&buf_idx) || !range.is_valid() {
        return (DiffBlockHandle::null(), DiffBlockHandle::null());
    }

    unsafe {
        let mut first_dp = DiffBlockHandle::null();
        let mut last_dp = DiffBlockHandle::null();

        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);
            let block_end = block_lnum + block_count.max(1) - 1;

            // Check if this block overlaps with the range
            if block_end >= range.first && block_lnum <= range.last {
                if first_dp.is_null() {
                    first_dp = dp;
                }
                last_dp = dp;
            }

            // If we've passed the range, stop
            if block_lnum > range.last {
                break;
            }

            dp = nvim_diffblock_get_next(dp);
        }

        (first_dp, last_dp)
    }
}

/// Count the number of diff blocks in a range.
pub fn diff_count_blocks_in_range(buf_idx: c_int, range: DiffRange) -> c_int {
    if !(0..DB_COUNT).contains(&buf_idx) || !range.is_valid() {
        return 0;
    }

    unsafe {
        let mut count = 0;
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);
            let block_end = block_lnum + block_count.max(1) - 1;

            // Check if this block overlaps with the range
            if block_end >= range.first && block_lnum <= range.last {
                count += 1;
            }

            // If we've passed the range, stop
            if block_lnum > range.last {
                break;
            }

            dp = nvim_diffblock_get_next(dp);
        }
        count
    }
}

// =============================================================================
// Diff Validation
// =============================================================================

/// Validate that a diff block is usable for get/put operations.
pub fn diff_validate_block(dp: DiffBlockHandle) -> bool {
    if dp.is_null() {
        return false;
    }

    unsafe {
        // Check that at least two buffers have content
        let mut valid_count = 0;
        for i in 0..DB_COUNT {
            let buf = nvim_get_curtab_diffbuf(i);
            if !buf.is_null() {
                valid_count += 1;
            }
        }
        valid_count >= 2
    }
}

/// Find the source buffer for a diffget operation.
///
/// If there's only one other buffer in diff mode, use that.
/// Otherwise, return -1 to indicate ambiguity.
pub fn diff_find_source_buffer(cur_idx: c_int) -> c_int {
    if !(0..DB_COUNT).contains(&cur_idx) {
        return -1;
    }

    unsafe {
        let mut source_idx = -1;
        let mut count = 0;

        for i in 0..DB_COUNT {
            if i != cur_idx && !nvim_get_curtab_diffbuf(i).is_null() {
                source_idx = i;
                count += 1;
            }
        }

        // Only return a source if there's exactly one other buffer
        if count == 1 {
            source_idx
        } else {
            -1
        }
    }
}

/// Calculate the line adjustment after a diff operation.
///
/// Returns the number of lines added (positive) or removed (negative).
pub fn diff_calc_line_adjustment(dp: DiffBlockHandle, idx_from: c_int, idx_to: c_int) -> LinenrT {
    if dp.is_null() {
        return 0;
    }

    unsafe {
        let count_from = nvim_diffblock_get_count(dp, idx_from);
        let count_to = nvim_diffblock_get_count(dp, idx_to);
        count_from - count_to
    }
}

// =============================================================================
// Corresponding Line Calculation
// =============================================================================

/// Calculate the corresponding line in another buffer.
///
/// This is used to position the cursor after switching between diff buffers.
pub fn diff_get_corresponding_line(from_idx: c_int, to_idx: c_int, lnum: LinenrT) -> LinenrT {
    if !(0..DB_COUNT).contains(&from_idx) || !(0..DB_COUNT).contains(&to_idx) {
        return lnum;
    }

    unsafe {
        let mut baseline: LinenrT = 0;
        let mut dp = nvim_get_diff_first_block();

        while !dp.is_null() {
            let from_lnum = nvim_diffblock_get_lnum(dp, from_idx);
            let from_count = nvim_diffblock_get_count(dp, from_idx);
            let to_lnum = nvim_diffblock_get_lnum(dp, to_idx);
            let to_count = nvim_diffblock_get_count(dp, to_idx);

            if from_lnum > lnum {
                // Line is before this diff block
                return lnum - baseline;
            }

            if from_lnum + from_count > lnum {
                // Line is inside this diff block
                let offset = lnum - from_lnum;
                let adjusted_offset = offset.min(to_count);
                return to_lnum + adjusted_offset;
            }

            // Update baseline for the next iteration
            baseline = (from_lnum + from_count) - (to_lnum + to_count);
            dp = nvim_diffblock_get_next(dp);
        }

        // Line is after all diff blocks
        lnum - baseline
    }
}

/// Calculate the corresponding line, clamped to buffer bounds.
pub fn diff_get_corresponding_line_clamped(
    from_idx: c_int,
    to_idx: c_int,
    lnum: LinenrT,
) -> LinenrT {
    let result = diff_get_corresponding_line(from_idx, to_idx, lnum);

    unsafe {
        let to_buf = nvim_get_curtab_diffbuf(to_idx);
        if to_buf.is_null() {
            return result;
        }
        let max_line = nvim_buf_get_ml_line_count(to_buf);
        result.min(max_line).max(1)
    }
}

// =============================================================================
// Diff Block Information
// =============================================================================

/// Information about a diff block for commands.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffBlockInfo {
    /// Handle to the diff block.
    pub handle: DiffBlockHandle,
    /// Line numbers for each buffer.
    pub lnum: [LinenrT; DB_COUNT as usize],
    /// Line counts for each buffer.
    pub count: [LinenrT; DB_COUNT as usize],
}

impl DiffBlockInfo {
    /// Create info from a diff block handle.
    ///
    /// # Safety
    /// The handle must be valid.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub unsafe fn from_handle(dp: DiffBlockHandle) -> Self {
        let mut info = Self {
            handle: dp,
            lnum: [0; DB_COUNT as usize],
            count: [0; DB_COUNT as usize],
        };

        if !dp.is_null() {
            for i in 0..DB_COUNT {
                info.lnum[i as usize] = nvim_diffblock_get_lnum(dp, i);
                info.count[i as usize] = nvim_diffblock_get_count(dp, i);
            }
        }

        info
    }

    /// Create empty info.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            handle: DiffBlockHandle::null(),
            lnum: [0; DB_COUNT as usize],
            count: [0; DB_COUNT as usize],
        }
    }

    /// Get the end line for a buffer index.
    #[must_use]
    pub const fn end_lnum(&self, idx: usize) -> LinenrT {
        if idx < DB_COUNT as usize {
            let count_adj = self.count[idx].saturating_sub(1);
            self.lnum[idx] + if count_adj > 0 { count_adj } else { 0 }
        } else {
            0
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Find diff blocks overlapping a range.
///
/// # Safety
/// `out_first` and `out_last` must be valid pointers if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_find_blocks_in_range(
    buf_idx: c_int,
    first: LinenrT,
    last: LinenrT,
    out_first: *mut DiffBlockHandle,
    out_last: *mut DiffBlockHandle,
) {
    let range = DiffRange::new(first, last);
    let (first_dp, last_dp) = diff_find_blocks_in_range(buf_idx, range);
    if !out_first.is_null() {
        *out_first = first_dp;
    }
    if !out_last.is_null() {
        *out_last = last_dp;
    }
}

/// FFI export: Count blocks in a range.
#[no_mangle]
pub extern "C" fn rs_diff_count_blocks_in_range(
    buf_idx: c_int,
    first: LinenrT,
    last: LinenrT,
) -> c_int {
    let range = DiffRange::new(first, last);
    diff_count_blocks_in_range(buf_idx, range)
}

/// FFI export: Validate a diff block.
#[no_mangle]
pub extern "C" fn rs_diff_validate_block(dp: DiffBlockHandle) -> c_int {
    c_int::from(diff_validate_block(dp))
}

/// FFI export: Find source buffer for diffget.
#[no_mangle]
pub extern "C" fn rs_diff_find_source_buffer(cur_idx: c_int) -> c_int {
    diff_find_source_buffer(cur_idx)
}

/// FFI export: Calculate line adjustment.
#[no_mangle]
pub extern "C" fn rs_diff_calc_line_adjustment(
    dp: DiffBlockHandle,
    idx_from: c_int,
    idx_to: c_int,
) -> LinenrT {
    diff_calc_line_adjustment(dp, idx_from, idx_to)
}

/// FFI export: Get corresponding line in another buffer (by index).
#[no_mangle]
pub extern "C" fn rs_diff_get_corresponding_line_by_idx(
    from_idx: c_int,
    to_idx: c_int,
    lnum: LinenrT,
) -> LinenrT {
    diff_get_corresponding_line(from_idx, to_idx, lnum)
}

/// FFI export: Get corresponding line, clamped to buffer bounds.
#[no_mangle]
pub extern "C" fn rs_diff_get_corresponding_line_clamped(
    from_idx: c_int,
    to_idx: c_int,
    lnum: LinenrT,
) -> LinenrT {
    diff_get_corresponding_line_clamped(from_idx, to_idx, lnum)
}

/// FFI export: Get diff block info.
#[no_mangle]
pub extern "C" fn rs_diff_get_block_info(dp: DiffBlockHandle) -> DiffBlockInfo {
    unsafe { DiffBlockInfo::from_handle(dp) }
}

// =============================================================================
// Phase 2 Migrations: nv_diffgetput and ex_diffthis
// =============================================================================

/// Normal mode "dp" and "do" commands -- Rust implementation.
///
/// Checks for prompt buffer, then builds an exarg_T and calls rs_ex_diffgetput.
///
/// # Safety
/// Calls C functions that access global state (curbuf, curwin).
#[no_mangle]
pub unsafe extern "C" fn rs_nv_diffgetput(put: bool, count: usize) {
    if nvim_bt_prompt_curbuf() {
        nvim_vim_beep_operator();
        return;
    }

    let lnum = nvim_get_curwin_cursor_lnum();
    let cmdidx = if put {
        nvim_docmd_cmd_diffput()
    } else {
        nvim_docmd_cmd_diffget()
    };

    // Build a minimal exarg_T on the stack and call rs_ex_diffgetput.
    // We use the C accessor nvim_diff_call_nv_diffgetput_impl to build the eap.
    if count == 0 {
        let empty: &[u8] = &[0u8];
        nvim_diff_call_nv_ex_diffgetput(cmdidx, empty.as_ptr().cast(), 0, lnum, lnum);
    } else {
        let mut buf = [0u8; 32];
        let s = format_usize_to_buf(count, &mut buf);
        nvim_diff_call_nv_ex_diffgetput(cmdidx, s.as_ptr().cast(), 0, lnum, lnum);
    }
}

/// Format a usize into a fixed buffer as ASCII digits, null-terminated.
/// Returns a slice pointing to the formatted string.
#[allow(clippy::cast_possible_truncation)]
fn format_usize_to_buf(mut n: usize, buf: &mut [u8; 32]) -> &[u8] {
    if n == 0 {
        buf[0] = b'0';
        buf[1] = 0;
        return &buf[..2];
    }
    let mut end = 0usize;
    while n > 0 {
        // n % 10 is always 0-9, safe to cast to u8
        buf[end] = b'0' + (n % 10) as u8;
        n /= 10;
        end += 1;
    }
    buf[..end].reverse();
    buf[end] = 0;
    &buf[..=end]
}

/// ":diffthis" command -- Rust implementation.
///
/// Calls rs_diff_win_options(curwin, true) directly (Rust-to-Rust).
///
/// # Safety
/// Calls C functions that access global state.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_diffthis(_eap: *mut c_void) {
    let curwin = nvim_get_curwin();
    crate::winopts::rs_diff_win_options(curwin, true);
}

// =============================================================================
// Phase 3 Migrations: ex_diffgetput
// =============================================================================

/// Check if a character is ASCII whitespace (space or tab).
#[inline]
const fn is_ascii_white(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Resolve the "other" buffer index when no argument is given.
///
/// Finds the single other diff buffer, enforcing that only one exists.
/// Returns `Ok(idx)` on success, `Err(())` if an error was emitted.
///
/// # Safety
/// Calls C functions that access global state.
unsafe fn diffgetput_resolve_auto(
    curbuf: BufHandle,
    cmdidx: c_int,
    cmd_diffput: c_int,
) -> Result<c_int, ()> {
    let mut found_not_ma = false;
    let mut found = DB_COUNT;
    let mut o = 0;
    while o < DB_COUNT {
        let diffbuf = nvim_diff_get_curtab_diffbuf_idx(o);
        if !diffbuf.is_null() && diffbuf != curbuf {
            if cmdidx != cmd_diffput || nvim_diff_buf_is_modifiable(diffbuf) {
                found = o;
                break;
            }
            found_not_ma = true;
        }
        o += 1;
    }

    if found == DB_COUNT {
        if found_not_ma {
            nvim_diff_emsg_e793();
        } else {
            nvim_diff_emsg_e100();
        }
        return Err(());
    }

    // Check that there isn't a third qualifying buffer in the list.
    let mut i = found + 1;
    while i < DB_COUNT {
        let diffbuf = nvim_diff_get_curtab_diffbuf_idx(i);
        if !diffbuf.is_null()
            && diffbuf != curbuf
            && (cmdidx != cmd_diffput || nvim_diff_buf_is_modifiable(diffbuf))
        {
            nvim_diff_emsg_e101();
            return Err(());
        }
        i += 1;
    }

    Ok(found)
}

/// Resolve the "other" buffer index from an argument string.
///
/// Parses a buffer number or pattern from `arg_ptr`. Returns `Ok(idx)` on
/// success, `Err(())` if an error was emitted or nothing needs to be done.
///
/// # Safety
/// `arg_ptr` must be a valid non-null, non-empty NUL-terminated C string.
/// Calls C functions that access global state.
unsafe fn diffgetput_resolve_arg(
    arg_ptr: *const c_char,
    curbuf: BufHandle,
    curtab: crate::buffer::TabpageHandle,
) -> Result<c_int, ()> {
    use crate::buffer::rs_diff_buf_idx_tp;

    // Find length of arg, then trim trailing ASCII whitespace.
    let mut arg_len = 0usize;
    while *arg_ptr.add(arg_len) != 0 {
        arg_len += 1;
    }
    let mut arg_end = arg_ptr.add(arg_len).cast_mut();
    while arg_end > arg_ptr.cast_mut() && is_ascii_white((*arg_end.sub(1)).cast_unsigned()) {
        arg_end = arg_end.sub(1);
    }
    let arg_end: *const c_char = arg_end.cast_const();

    // Check whether all characters are ASCII digits.
    let mut all_digits = true;
    let mut p = arg_ptr;
    while p < arg_end {
        if !(*p).cast_unsigned().is_ascii_digit() {
            all_digits = false;
            break;
        }
        p = p.add(1);
    }

    let bufnr = if all_digits && arg_end > arg_ptr {
        // digits only -- parse as buffer number using atol equivalent
        let mut n: c_int = 0;
        let mut p2 = arg_ptr;
        while p2 < arg_end {
            n = n
                .wrapping_mul(10)
                .wrapping_add(c_int::from((*p2).cast_unsigned() - b'0'));
            p2 = p2.add(1);
        }
        n
    } else {
        let nr = nvim_diff_buflist_findpat(arg_ptr, arg_end);
        if nr < 0 {
            // error message already given by buflist_findpat
            return Err(());
        }
        nr
    };

    let buf = nvim_diff_buflist_findnr(bufnr);
    if buf.is_null() {
        nvim_diff_semsg_e102(arg_ptr);
        return Err(());
    }

    if buf == curbuf {
        // nothing to do
        return Err(());
    }

    let other_idx = rs_diff_buf_idx_tp(buf, curtab);
    if other_idx == DB_COUNT {
        nvim_diff_semsg_e103(arg_ptr);
        return Err(());
    }

    Ok(other_idx)
}

/// Adjust the range when no address count was given.
///
/// # Safety
/// `eap` must be a valid pointer. Calls C functions.
#[allow(clippy::cast_possible_wrap)]
unsafe fn diffgetput_adjust_range(eap: *mut c_void) {
    use crate::buffer::rs_diff_check_with_linestatus;

    let line1 = nvim_eap_get_line1(eap);
    let curwin = nvim_get_curwin();
    let line_count = nvim_diff_curbuf_ml_line_count();

    // Make it possible that ":diffget" on the last line gets the line below
    // the cursor when there is no difference above the cursor.
    let do_increment = if line1 == line_count {
        let mut ls0: c_int = 0;
        let check0 = rs_diff_check_with_linestatus(curwin, line1, &raw mut ls0);
        if check0 == 0 && ls0 == 0 {
            if line1 == 1 {
                true
            } else {
                let mut ls1: c_int = 0;
                let c1 = rs_diff_check_with_linestatus(curwin, line1 - 1, &raw mut ls1);
                c1 >= 0 && ls1 == 0
            }
        } else {
            false
        }
    } else {
        false
    };

    if do_increment {
        nvim_eap_set_line2(eap, nvim_eap_get_line2(eap) + 1);
    } else if line1 > 0 {
        nvim_eap_set_line1(eap, line1 - 1);
    }
}

/// Post-operation cleanup after diffgetput: cursor check, fold update, redraw.
///
/// # Safety
/// Calls C functions that access global state.
unsafe fn diffgetput_cleanup() {
    nvim_diff_set_busy(false);

    if nvim_diff_get_need_update() {
        rs_diff_ex_diffupdate(std::ptr::null());
    }

    // Check that the cursor is on a valid character and update its position.
    nvim_diff_check_cursor_curwin();
    nvim_diff_changed_line_abv_curs();

    // If all diffs are gone, update folds in all diff windows.
    if nvim_diff_curtab_first_diff_is_null() {
        let tp = nvim_get_curtab();
        let mut wp = nvim_tabpage_first_win(tp);
        while !wp.is_null() {
            if nvim_win_get_p_diff(wp) != 0
                && nvim_diff_win_get_w_p_fdm_starts_d(wp)
                && nvim_win_get_w_p_fen(wp)
            {
                rs_foldUpdateAll(wp);
            }
            wp = nvim_win_next(wp);
        }
    }

    if nvim_diff_get_need_update() {
        // Redraw already done by rs_diff_ex_diffupdate().
        nvim_diff_set_need_update(false);
    } else {
        // Also need to redraw the other buffers.
        rs_diff_redraw(false);
        nvim_diff_fire_diffupdated_curbuf();
    }
}

/// ":diffget" and ":diffput" commands -- Rust implementation.
///
/// Finds the current buffer's diff index, resolves the other buffer (by
/// argument or auto-detect), manages aucmd_prepbuf/restbuf, calls diffgetput,
/// and handles post-operation cursor/fold/redraw updates.
///
/// # Safety
/// Calls C functions that access global state (curbuf, curwin, curtab).
#[no_mangle]
pub unsafe extern "C" fn rs_ex_diffgetput(eap: *mut c_void) {
    use crate::buffer::rs_diff_buf_idx_tp;

    let curtab = nvim_get_curtab();
    let curbuf = nvim_diff_get_curbuf();

    // Find the current buffer in the list of diff buffers.
    let idx_cur = rs_diff_buf_idx_tp(curbuf, curtab);
    if idx_cur == DB_COUNT {
        nvim_diff_emsg_e99();
        return;
    }

    let cmdidx = nvim_eap_get_cmdidx(eap);
    let cmd_diffput = nvim_diff_get_CMD_diffput();
    let cmd_diffget = nvim_diff_get_CMD_diffget();

    // Resolve the "other" buffer index.
    let arg_ptr = nvim_eap_get_arg(eap);
    let arg_empty = arg_ptr.is_null() || *arg_ptr == 0;

    let idx_other = if arg_empty {
        match diffgetput_resolve_auto(curbuf, cmdidx, cmd_diffput) {
            Ok(idx) => idx,
            Err(()) => return,
        }
    } else {
        match diffgetput_resolve_arg(arg_ptr, curbuf, curtab) {
            Ok(idx) => idx,
            Err(()) => return,
        }
    };

    nvim_diff_set_busy(true);

    // When no range given, include the line above or below the cursor.
    if nvim_eap_get_addr_count(eap) == 0 {
        diffgetput_adjust_range(eap);
    }

    let is_diffput = cmdidx != cmd_diffget;

    if is_diffput {
        // Need to make the other buffer current to be able to make changes.
        nvim_diff_aucmd_prepbuf_idx(idx_other);
    }

    let idx_from = if cmdidx == cmd_diffget {
        idx_other
    } else {
        idx_cur
    };
    let idx_to = if cmdidx == cmd_diffget {
        idx_cur
    } else {
        idx_other
    };

    // May give the warning for a changed buffer here, which can trigger
    // FileChangedRO autocommand, which may do nasty things and mess
    // everything up.
    if !nvim_diff_curbuf_changed() {
        nvim_diff_change_warning_curbuf();
        if !nvim_diff_curbuf_is_curtab_diffbuf(idx_to) {
            nvim_diff_emsg_e787();
            // goto theend: skip diffgetput and aucmd_restbuf, matching C behavior
            // when FileChangedRO may have already altered state.
            diffgetput_cleanup();
            return;
        }
    }

    nvim_diff_call_diffgetput(
        nvim_eap_get_addr_count(eap),
        idx_cur,
        idx_from,
        idx_to,
        nvim_eap_get_line1(eap),
        nvim_eap_get_line2(eap),
    );

    // Restore curwin/curbuf and a few other things.
    if is_diffput {
        // Syncing undo only works for the current buffer, but we changed
        // another buffer. Sync undo if the command was typed.
        if nvim_diff_key_typed() {
            nvim_diff_u_sync();
        }
        nvim_diff_aucmd_restbuf();
    }

    diffgetput_cleanup();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_operation_values() {
        assert_eq!(DiffOperation::Get as c_int, 0);
        assert_eq!(DiffOperation::Put as c_int, 1);
    }

    #[test]
    fn test_diff_op_result_fail() {
        let result = DiffOpResult::fail();
        assert_eq!(result.status, FAIL);
        assert_eq!(result.lines_changed, 0);
    }

    #[test]
    fn test_diff_op_result_success() {
        let result = DiffOpResult::success(5, 10, 14);
        assert_eq!(result.status, OK);
        assert_eq!(result.lines_changed, 5);
        assert_eq!(result.first_line, 10);
        assert_eq!(result.last_line, 14);
    }

    #[test]
    fn test_diff_range() {
        let range = DiffRange::new(10, 20);
        assert!(range.is_valid());
        assert_eq!(range.count(), 11);

        let empty = DiffRange::empty();
        assert!(!empty.is_valid());
        assert_eq!(empty.count(), 0);

        let invalid = DiffRange::new(20, 10);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_diff_block_info_empty() {
        let info = DiffBlockInfo::empty();
        assert!(info.handle.is_null());
        for i in 0..DB_COUNT as usize {
            assert_eq!(info.lnum[i], 0);
            assert_eq!(info.count[i], 0);
        }
    }

    #[test]
    fn test_struct_sizes() {
        use std::mem::size_of;

        // DiffOpResult: 4 * 4 = 16 bytes
        assert_eq!(size_of::<DiffOpResult>(), 16);

        // DiffRange: 2 * 4 = 8 bytes
        assert_eq!(size_of::<DiffRange>(), 8);
    }
}
