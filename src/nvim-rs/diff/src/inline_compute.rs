//! Inline diff computation in Rust
//!
//! This module ports `diff_find_change_inline_diff` and
//! `diff_refine_inline_char_highlight` from C to Rust, replacing the
//! `nvim_diff_compute_inline` Rust->C roundtrip.

#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::c_char;
use std::os::raw::c_int;

use crate::buffer::{BufHandle, DiffBlockHandle, DiffioHandle, DB_COUNT};

// ============================================================================
// Diff flags (must match C defines in diff_shim.c)
// ============================================================================

const DIFF_ICASE: c_int = 0x004;
const DIFF_IWHITE: c_int = 0x008;
const DIFF_IWHITEALL: c_int = 0x010;
const DIFF_IWHITEEOL: c_int = 0x020;
const DIFF_INLINE_CHAR: c_int = 0x8000;
const DIFF_INLINE_WORD: c_int = 0x10000;

const OK: c_int = 1;

/// NUL byte (replacement for embedded NUL in Neovim lines)
const NUL: u8 = 0;
/// NL byte (Neovim's internal representation of NUL in buffer lines)
const NL: u8 = b'\n';

/// MAXCOL sentinel -- signals "column beyond end of line"
const MAXCOL: i32 = i32::MAX;

// XDF_INDENT_HEURISTIC flag value (1 << 23)
const XDF_INDENT_HEURISTIC: c_int = 1 << 23;

// ============================================================================
// Linemap entry -- maps from xdiff line number back to original buffer line/col
// ============================================================================

#[derive(Clone, Copy, Debug, Default)]
struct LinemapEntry {
    byte_start: i32,
    num_bytes: i32,
    lineoff: i32,
}

// ============================================================================
// C FFI declarations
// ============================================================================

extern "C" {
    fn nvim_get_diff_flags() -> c_int;
    fn nvim_diff_get_algorithm() -> c_int;
    fn nvim_diff_set_options(
        flags: c_int,
        context: c_int,
        linematch: c_int,
        foldcol: c_int,
        algorithm: c_int,
    );
    fn nvim_diff_get_context() -> c_int;
    fn nvim_diff_get_linematch_lines() -> c_int;
    fn nvim_diff_get_foldcolumn() -> c_int;

    fn nvim_get_diff_first_block() -> DiffBlockHandle;
    fn nvim_tabpage_set_first_diff(tp: *mut std::ffi::c_void, dp: DiffBlockHandle);
    fn nvim_tabpage_set_diffbuf(tp: *mut std::ffi::c_void, idx: c_int, buf: BufHandle);
    fn nvim_get_curtab_diffbuf(idx: c_int) -> BufHandle;

    fn nvim_diff_buf_is_loaded(buf: BufHandle) -> bool;

    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> i32;
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> i32;
    fn nvim_diffblock_get_next(dp: DiffBlockHandle) -> DiffBlockHandle;
    fn nvim_diffblock_set_count(dp: DiffBlockHandle, idx: c_int, count: i32);
    fn nvim_diff_set_next(dp: DiffBlockHandle, next: DiffBlockHandle);

    fn nvim_diffblock_set_has_changes(dp: DiffBlockHandle, val: bool);
    fn nvim_diffblock_reset_changes_len(dp: DiffBlockHandle);

    fn nvim_diffblock_append_change(
        dp: DiffBlockHandle,
        dc_start: *const i32,
        dc_end: *const i32,
        dc_start_lnum_off: *const i32,
        dc_end_lnum_off: *const i32,
    );

    fn nvim_diffio_new(use_internal: bool) -> DiffioHandle;
    fn nvim_diffio_free(dio: DiffioHandle);
    fn nvim_diffio_init_ga(dio: DiffioHandle);
    fn nvim_diffio_clear_output(dio: DiffioHandle);

    fn nvim_xdiff_internal_run(
        orig_data: *const u8,
        orig_size: c_int,
        new_data: *const u8,
        new_size: c_int,
        dio: DiffioHandle,
    ) -> c_int;

    fn nvim_diffbuf_get_chartab(idx: c_int) -> *const u64;

    fn nvim_diff_ml_get_buf_diffbuf(idx: c_int, nr: i32) -> *const c_char;

    fn rs_diff_read(idx_orig: c_int, idx_new: c_int, dio: DiffioHandle);
    fn rs_diff_clear(tp: *mut std::ffi::c_void);
    fn rs_clear_diffblock(dp: DiffBlockHandle);
    fn nvim_get_curtab() -> *mut std::ffi::c_void;

    // UTF-8 helpers
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_fold(c: c_int) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utf_char2len(c: c_int) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn skipwhite(p: *const c_char) -> *const c_char;

    fn mb_get_class_tab(p: *const c_char, chartab: *const u64) -> c_int;
}

// ============================================================================
// Tokenizer helpers
// ============================================================================

/// Returns true if `c` is ASCII whitespace (space or tab).
#[inline]
const fn is_ascii_white(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Emit one character/token into `file_data` with case folding if requested.
///
/// Returns the number of source bytes consumed.
#[allow(clippy::too_many_arguments)]
unsafe fn emit_char(
    s: *const c_char,
    byte: u8,
    diff_flags: c_int,
    file_data: &mut Vec<u8>,
) -> c_int {
    if byte == NL {
        // NL is the internal substitute for NUL in buffer lines
        file_data.push(NUL);
        return 1;
    }

    // Determine how many source bytes this token spans
    let char_len = utfc_ptr2len(s);
    let effective_len = if is_ascii_white(byte) && (diff_flags & DIFF_IWHITE) != 0 {
        // Entire whitespace span counts as one token
        (skipwhite(s) as usize - s as usize) as c_int
    } else {
        char_len
    };

    if (diff_flags & DIFF_ICASE) != 0 {
        // Case-fold: decode, fold, re-encode
        let c = utf_ptr2char(s);
        let c_len = utf_char2len(c);
        let c_folded = utf_fold(c);
        let mut cbuf = [0u8; 8]; // MB_MAXBYTES + 1
        let c_fold_len = utf_char2bytes(c_folded, cbuf.as_mut_ptr().cast());
        file_data.extend_from_slice(&cbuf[..c_fold_len as usize]);
        // Append remaining composing characters (no case folding needed)
        if effective_len > c_len {
            let extra = std::slice::from_raw_parts(
                s.offset(c_len as isize).cast::<u8>(),
                (effective_len - c_len) as usize,
            );
            file_data.extend_from_slice(extra);
        }
    } else {
        let bytes = std::slice::from_raw_parts(s.cast::<u8>(), effective_len as usize);
        file_data.extend_from_slice(bytes);
    }

    effective_len
}

// ============================================================================
// Tokenizer
// ============================================================================

/// Tokenize buffer lines into newline-delimited char/word tokens.
///
/// Returns `(file_data, linemap)` where `file_data` is the token byte stream
/// and `linemap` maps each xdiff "line" (token) back to the original
/// buffer line-offset and byte range.
///
/// # Safety
///
/// Callers must ensure `idx` is a valid diffbuf index and `dp` is valid.
unsafe fn tokenize_lines(
    dp: DiffBlockHandle,
    idx: c_int,
    file1_idx: c_int,
    diff_flags: c_int,
) -> (Vec<u8>, Vec<LinemapEntry>) {
    let mut file_data: Vec<u8> = Vec::with_capacity(4096);
    let mut linemap: Vec<LinemapEntry> = Vec::with_capacity(512);

    let count = nvim_diffblock_get_count(dp, idx);
    let lnum_base = nvim_diffblock_get_lnum(dp, idx);

    // Chartab for word-class detection (always from the first buffer)
    let chartab_ptr = if (diff_flags & DIFF_INLINE_WORD) != 0 {
        nvim_diffbuf_get_chartab(file1_idx)
    } else {
        std::ptr::null()
    };

    for off in 0..count {
        let line_ptr = nvim_diff_ml_get_buf_diffbuf(idx, lnum_base + off);
        if line_ptr.is_null() {
            continue;
        }

        tokenize_line(
            line_ptr,
            off,
            diff_flags,
            chartab_ptr,
            &mut file_data,
            &mut linemap,
        );
    }

    (file_data, linemap)
}

/// Tokenize a single buffer line, appending to `file_data` and `linemap`.
///
/// # Safety
///
/// `line_ptr` must be a valid NUL-terminated C string.
unsafe fn tokenize_line(
    line_ptr: *const c_char,
    off: i32,
    diff_flags: c_int,
    chartab_ptr: *const u64,
    file_data: &mut Vec<u8>,
    linemap: &mut Vec<LinemapEntry>,
) {
    let mut in_keyword = false;
    let mut last_white = false;
    let mut eol_file_len: i32 = -1;
    let mut eol_linemap_len: i32 = -1;
    let mut eol_numlines: i32 = -1;
    let mut numlines: i32 = 0;

    let mut s = line_ptr;

    loop {
        let byte = *s as u8;
        if byte == 0 {
            break; // NUL == end of line
        }

        let new_in_keyword = (diff_flags & DIFF_INLINE_WORD) != 0
            && !chartab_ptr.is_null()
            && mb_get_class_tab(s, chartab_ptr) == 2;

        // Close the current keyword token when transitioning out of keyword
        if in_keyword && !new_in_keyword {
            file_data.push(NL);
            numlines += 1;
        }

        // Whitespace handling
        if is_ascii_white(byte) {
            if (diff_flags & DIFF_IWHITEALL) != 0 {
                in_keyword = false;
                s = skipwhite(s);
                continue;
            }
            if (diff_flags & (DIFF_IWHITEEOL | DIFF_IWHITE)) != 0 && !last_white {
                eol_file_len = file_data.len() as i32;
                eol_linemap_len = linemap.len() as i32;
                eol_numlines = numlines;
                last_white = true;
            }
        } else if (diff_flags & (DIFF_IWHITEEOL | DIFF_IWHITE)) != 0 {
            last_white = false;
            eol_file_len = -1;
            eol_linemap_len = -1;
            eol_numlines = -1;
        }

        let effective_len = emit_char(s, byte, diff_flags, file_data);

        // Emit NL separator after each char/token (not while accumulating a word)
        if !new_in_keyword {
            file_data.push(NL);
            numlines += 1;
        }

        // Update linemap
        let cur_byte_start = (s as usize - line_ptr as usize) as i32;
        if !new_in_keyword || !in_keyword {
            // New token: create a new mapping entry
            linemap.push(LinemapEntry {
                lineoff: off,
                byte_start: cur_byte_start,
                num_bytes: effective_len,
            });
        } else if let Some(last) = linemap.last_mut() {
            // Still inside a keyword: extend the last entry's byte count
            last.num_bytes += effective_len;
        }

        in_keyword = new_in_keyword;
        s = s.offset(effective_len as isize);
    }

    // Close the final keyword token
    if in_keyword {
        file_data.push(NL);
        numlines += 1;
    }

    // Trim trailing whitespace for iwhiteeol / iwhite
    if (diff_flags & (DIFF_IWHITEEOL | DIFF_IWHITE)) != 0 && eol_file_len >= 0 {
        file_data.truncate(eol_file_len as usize);
        linemap.truncate(eol_linemap_len as usize);
        numlines = eol_numlines;
    }

    // Add an EOL token (empty line mapped to the line's NUL terminator) unless iwhiteall
    if (diff_flags & DIFF_IWHITEALL) == 0 {
        file_data.push(NL);
        numlines += 1;
        let eol_byte_start = (s as usize - line_ptr as usize) as i32;
        linemap.push(LinemapEntry {
            lineoff: off,
            byte_start: eol_byte_start,
            num_bytes: 1,
        });
    }

    // numlines is kept for logical consistency; silence the unused warning
    let _ = numlines;
}

// ============================================================================
// Refinement pass
// ============================================================================

/// Refine inline char-highlight diff blocks.
///
/// Merges adjacent long diff blocks if they are only separated by a small gap,
/// producing cleaner visual output.
///
/// # Safety
///
/// `dp_orig` must be a valid diff block chain; `linemap` must be indexed by
/// buffer index up to `DB_COUNT`.
unsafe fn refine_inline_char_highlight(
    dp_orig: DiffBlockHandle,
    linemap: &[Vec<LinemapEntry>],
    idx1: c_int,
) {
    // Up to 4 passes so that newly merged blocks can trigger further merges
    for _ in 0..4u32 {
        if !refine_pass(dp_orig, linemap, idx1) {
            break;
        }
    }
}

/// One refinement pass. Returns true if any merge was performed AND there were
/// unmerged gaps remaining (i.e. another pass is worthwhile).
unsafe fn refine_pass(
    dp_orig: DiffBlockHandle,
    linemap: &[Vec<LinemapEntry>],
    idx1: c_int,
) -> bool {
    let mut has_unmerged_gaps = false;
    let mut has_merged_gaps = false;
    let mut dp = dp_orig;
    let lmap = &linemap[idx1 as usize];

    while !dp.is_null() {
        let dp_next = nvim_diffblock_get_next(dp);
        if dp_next.is_null() {
            break;
        }

        let dp_end_lnum =
            nvim_diffblock_get_lnum(dp, idx1) + nvim_diffblock_get_count(dp, idx1) - 1;
        let dp_next_lnum = nvim_diffblock_get_lnum(dp_next, idx1) - 1;

        if dp_end_lnum as usize >= lmap.len() || dp_next_lnum as usize >= lmap.len() {
            dp = dp_next;
            continue;
        }

        // Only merge if gap lies within the same source line
        if lmap[dp_end_lnum as usize].lineoff != lmap[dp_next_lnum as usize].lineoff {
            dp = dp_next;
            continue;
        }

        let gap = nvim_diffblock_get_lnum(dp_next, idx1)
            - (nvim_diffblock_get_lnum(dp, idx1) + nvim_diffblock_get_count(dp, idx1));

        if gap <= 3 {
            let mut max_df_count = 0i32;
            for i in 0..DB_COUNT {
                let combined =
                    nvim_diffblock_get_count(dp, i) + nvim_diffblock_get_count(dp_next, i);
                max_df_count = max_df_count.max(combined);
            }

            if max_df_count >= gap * 4 {
                // Merge dp_next into dp
                for i in 0..DB_COUNT {
                    let new_count = nvim_diffblock_get_lnum(dp_next, i)
                        + nvim_diffblock_get_count(dp_next, i)
                        - nvim_diffblock_get_lnum(dp, i);
                    nvim_diffblock_set_count(dp, i, new_count);
                }
                nvim_diff_set_next(dp, nvim_diffblock_get_next(dp_next));
                rs_clear_diffblock(dp_next);
                has_merged_gaps = true;
                // Do not advance dp -- try to merge again with the next block
                continue;
            }
            has_unmerged_gaps = true;
        }
        dp = dp_next;
    }

    has_merged_gaps && has_unmerged_gaps
}

// ============================================================================
// Linemap-to-change conversion
// ============================================================================

/// Convert the new diff blocks (stored in curtab's scratch diff list) into
/// `diffline_change_T` entries appended to `dp->df_changes`, using the
/// linemap to recover original buffer positions.
unsafe fn write_changes(dp: DiffBlockHandle, linemaps: &[Vec<LinemapEntry>]) {
    nvim_diffblock_reset_changes_len(dp);

    let mut cur_diff = nvim_get_diff_first_block();
    while !cur_diff.is_null() {
        let mut dc_start = [0i32; DB_COUNT as usize];
        let mut dc_end = [0i32; DB_COUNT as usize];
        let mut dc_start_lnum_off = [0i32; DB_COUNT as usize];
        let mut dc_end_lnum_off = [0i32; DB_COUNT as usize];

        for i in 0..DB_COUNT as usize {
            let raw_lnum = nvim_diffblock_get_lnum(cur_diff, i as c_int);
            if raw_lnum <= 0 {
                dc_start[i] = MAXCOL;
                dc_start_lnum_off[i] = i32::MAX;
                dc_end[i] = MAXCOL;
                dc_end_lnum_off[i] = i32::MAX;
                continue;
            }
            let diff_lnum = (raw_lnum - 1) as usize; // zero-indexed
            let diff_lnum_end = diff_lnum + nvim_diffblock_get_count(cur_diff, i as c_int) as usize;
            let lmap = &linemaps[i];

            if diff_lnum >= lmap.len() {
                dc_start[i] = MAXCOL;
                dc_start_lnum_off[i] = i32::MAX;
            } else {
                dc_start[i] = lmap[diff_lnum].byte_start;
                dc_start_lnum_off[i] = lmap[diff_lnum].lineoff;
            }

            if diff_lnum == diff_lnum_end {
                dc_end[i] = dc_start[i];
                dc_end_lnum_off[i] = dc_start_lnum_off[i];
            } else if diff_lnum_end > lmap.len() {
                dc_end[i] = MAXCOL;
                dc_end_lnum_off[i] = i32::MAX;
            } else {
                let entry = &lmap[diff_lnum_end - 1];
                dc_end[i] = entry.byte_start + entry.num_bytes;
                dc_end_lnum_off[i] = entry.lineoff;
            }
        }

        nvim_diffblock_append_change(
            dp,
            dc_start.as_ptr(),
            dc_end.as_ptr(),
            dc_start_lnum_off.as_ptr(),
            dc_end_lnum_off.as_ptr(),
        );

        cur_diff = nvim_diffblock_get_next(cur_diff);
    }
}

// ============================================================================
// Main orchestrator
// ============================================================================

/// Compute inline diff for a diff block.
///
/// Replaces `diff_find_change_inline_diff` in C and eliminates the
/// `nvim_diff_compute_inline` Rust->C roundtrip.
///
/// # Safety
///
/// `dp` must be a valid non-null diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_compute_inline_diff(dp: DiffBlockHandle) {
    if dp.is_null() {
        return;
    }

    let diff_flags = nvim_get_diff_flags();
    let save_algorithm = nvim_diff_get_algorithm();
    let save_context = nvim_diff_get_context();
    let save_linematch = nvim_diff_get_linematch_lines();
    let save_foldcol = nvim_diff_get_foldcolumn();

    // Inline diff always uses the indent heuristic
    nvim_diff_set_options(
        diff_flags,
        save_context,
        save_linematch,
        save_foldcol,
        save_algorithm | XDF_INDENT_HEURISTIC,
    );

    // Save curtab scratch state (tp_first_diff and tp_diffbuf)
    let orig_first_diff = nvim_get_diff_first_block();
    nvim_tabpage_set_first_diff(nvim_get_curtab(), DiffBlockHandle::null());
    let mut orig_diffbuf = [BufHandle::null(); DB_COUNT as usize];
    for i in 0..DB_COUNT {
        orig_diffbuf[i as usize] = nvim_get_curtab_diffbuf(i);
    }

    let dio = nvim_diffio_new(true);
    nvim_diffio_init_ga(dio);

    let mut file1_data: Vec<u8> = Vec::new();
    let mut linemaps: [Vec<LinemapEntry>; DB_COUNT as usize] = Default::default();
    let mut file1_idx: c_int = -1;
    let mut failed = false;

    for i in 0..DB_COUNT {
        let buf = nvim_get_curtab_diffbuf(i);
        if buf.is_null() || !nvim_diff_buf_is_loaded(buf) {
            continue;
        }
        if nvim_diffblock_get_count(dp, i) == 0 {
            // Multi-buffer diff: skip buffers with no text in this block
            nvim_tabpage_set_diffbuf(nvim_get_curtab(), i, BufHandle::null());
            continue;
        }

        if file1_idx == -1 {
            file1_idx = i;
        }

        let (buf_tokens, linemap) = tokenize_lines(dp, i, file1_idx, diff_flags);
        linemaps[i as usize] = linemap;

        if file1_idx == i {
            file1_data = buf_tokens;
        } else {
            let status = nvim_xdiff_internal_run(
                file1_data.as_ptr(),
                file1_data.len() as c_int,
                buf_tokens.as_ptr(),
                buf_tokens.len() as c_int,
                dio,
            );
            if status == OK {
                rs_diff_read(0, i, dio);
            } else {
                failed = true;
            }
            nvim_diffio_clear_output(dio);
        }
    }

    // Refinement for char mode
    let new_diff = nvim_get_diff_first_block();
    if (diff_flags & DIFF_INLINE_CHAR) != 0 && file1_idx != -1 && !new_diff.is_null() {
        refine_inline_char_highlight(new_diff, &linemaps, file1_idx);
    }

    // Map diff results back to original buffer positions
    if !failed {
        write_changes(dp, &linemaps);
    }

    nvim_diffblock_set_has_changes(dp, true);

    // Clean up the scratch diff state
    rs_diff_clear(nvim_get_curtab());

    // Restore curtab state
    nvim_tabpage_set_first_diff(nvim_get_curtab(), orig_first_diff);
    for i in 0..DB_COUNT {
        nvim_tabpage_set_diffbuf(nvim_get_curtab(), i, orig_diffbuf[i as usize]);
    }

    // Restore diff options
    nvim_diff_set_options(
        diff_flags,
        save_context,
        save_linematch,
        save_foldcol,
        save_algorithm,
    );

    nvim_diffio_free(dio);
}
