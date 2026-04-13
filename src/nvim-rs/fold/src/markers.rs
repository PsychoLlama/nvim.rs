//! Fold marker parsing and level calculation for marker-based folding.
//!
//! This module implements the fold marker system used when 'foldmethod' is "marker".
//! Fold markers are special text patterns (default `{{{` and `}}}`) that define
//! fold boundaries in the buffer text.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use nvim_buffer::BufHandle;
use nvim_window::win_struct::win_ref;
use nvim_window::WinHandle;

use crate::{FoldHandle, GArrayHandle, LineNr};

/// Maximum fold level supported by Neovim.
pub const MAX_LEVEL: c_int = 20;

/// Parsed fold marker information.
///
/// Contains the parsed start and end markers along with their lengths.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldMarkerInfo {
    /// Pointer to the start marker string (from w_p_fmr).
    pub start_marker: *const c_char,
    /// Length of the start marker (excluding optional digit).
    pub start_marker_len: usize,
    /// Pointer to the end marker string (after the comma in w_p_fmr).
    pub end_marker: *const c_char,
    /// Length of the end marker (excluding optional digit).
    pub end_marker_len: usize,
}

impl Default for FoldMarkerInfo {
    fn default() -> Self {
        Self {
            start_marker: ptr::null(),
            start_marker_len: 0,
            end_marker: ptr::null(),
            end_marker_len: 0,
        }
    }
}

// C accessor functions
extern "C" {
    /// Get the w_p_fmr (foldmarker option) field from a window.
    fn nvim_win_get_p_fmr(wp: WinHandle) -> *const c_char;

    /// Get a line from a buffer.
    #[link_name = "ml_get_buf"]
    fn nvim_ml_get_buf(buf: BufHandle, lnum: LineNr) -> *const c_char;

    // ---- Phase 3: new accessors for marker manipulation ----

    /// Get the length of a buffer line.
    fn nvim_fold_ml_get_buf_len(buf: BufHandle, lnum: LineNr) -> c_int;

    /// Replace a buffer line, consuming ownership of newline (must be xmalloc'd).
    fn nvim_fold_ml_replace_buf(buf: BufHandle, lnum: LineNr, newline: *mut c_char) -> c_int;

    /// Save undo for lnum range: u_save(lnum-1, lnum+1). Returns OK(1) or FAIL(0).
    fn nvim_fold_u_save(lnum: LineNr) -> c_int;

    /// Wrapper for extmark_splice_cols (lnum_0 is 0-based row).
    fn nvim_fold_extmark_splice_cols(
        buf: BufHandle,
        lnum_0: c_int,
        col: c_int,
        old_col: c_int,
        new_col: c_int,
    );

    /// Check if a line ends with an unclosed comment; sets *out_is_comment.
    fn nvim_fold_skip_comment(line: *const c_char, out_is_comment: *mut c_int);

    /// Get the commentstring option (b_p_cms) for a buffer.
    fn nvim_fold_get_buf_b_p_cms(buf: BufHandle) -> *const c_char;

    /// Allocate size bytes via xmalloc (Neovim's allocator; never returns null).
    fn nvim_fold_xmalloc(size: usize) -> *mut c_void;

    /// Check if buffer is modifiable.
    fn nvim_fold_buf_is_modifiable(buf: BufHandle) -> c_int;

    /// Emit the "not modifiable" error message.
    fn nvim_fold_emsg_modifiable();

    /// Notify changed lines.
    #[link_name = "changed_lines"]
    fn nvim_changed_lines(
        buf: BufHandle,
        first: LineNr,
        col: c_int,
        last: LineNr,
        xtra: LineNr,
        add_undo: bool,
    );

    /// Send buffer update events.
    #[link_name = "buf_updates_send_changes"]
    fn nvim_buf_updates_send_changes(
        buf: BufHandle,
        firstlnum: LineNr,
        num_added: i64,
        num_removed: i64,
    );

    /// Get number of items in a garray.
    fn nvim_ga_len(gap: GArrayHandle) -> c_int;

    /// Get fold_T at index in garray.
    fn nvim_ga_fold_at(gap: GArrayHandle, idx: c_int) -> FoldHandle;

    /// Get fd_top from a fold.
    fn nvim_fold_get_fd_top(fp: FoldHandle) -> LineNr;

    /// Get fd_len from a fold.
    fn nvim_fold_get_fd_len(fp: FoldHandle) -> LineNr;

    /// Get fd_nested garray from a fold.
    fn nvim_fold_get_fd_nested(fp: FoldHandle) -> GArrayHandle;

    /// Get line count for a buffer.
    fn nvim_fold_buf_get_line_count(buf: BufHandle) -> LineNr;
}

/// Parse the 'foldmarker' option into start and end markers.
///
/// The 'foldmarker' option has the format "startmarker,endmarker" (e.g., "{{{,}}}").
/// This function locates the comma separator and calculates the lengths.
///
/// Returns a `FoldMarkerInfo` struct with pointers into the original option string.
/// The pointers are only valid as long as the window's foldmarker option is unchanged.
#[must_use]
pub fn parse_marker_impl(wp: WinHandle) -> FoldMarkerInfo {
    if wp.is_null() {
        return FoldMarkerInfo::default();
    }

    let fmr = unsafe { nvim_win_get_p_fmr(wp) };
    if fmr.is_null() {
        return FoldMarkerInfo::default();
    }

    // Find the comma separator
    let mut end_marker = fmr;
    unsafe {
        while *end_marker != 0 && *end_marker != b',' as c_char {
            end_marker = end_marker.add(1);
        }

        if *end_marker != b',' as c_char {
            // No comma found, invalid format - return default
            return FoldMarkerInfo::default();
        }

        // SAFETY: We know end_marker >= fmr since we started from fmr and only moved forward.
        // The result is always non-negative.
        #[allow(clippy::cast_sign_loss)]
        let start_marker_len = end_marker.offset_from(fmr) as usize;

        // Move past the comma
        end_marker = end_marker.add(1);

        // Calculate end marker length
        let end_marker_len = if *end_marker == 0 {
            0
        } else {
            // Use strlen equivalent
            let mut len = 0;
            let mut p = end_marker;
            while *p != 0 {
                len += 1;
                p = p.add(1);
            }
            len
        };

        FoldMarkerInfo {
            start_marker: fmr,
            start_marker_len,
            end_marker,
            end_marker_len,
        }
    }
}

/// Result of fold level calculation for a line.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldLevelMarkerResult {
    /// Current fold level.
    pub lvl: c_int,
    /// Fold level for the next line.
    pub lvl_next: c_int,
    /// Number of folds that start at this line (>0 means fold(s) start here).
    pub start: c_int,
}

/// Calculate fold level for a line using marker method.
///
/// This is the low-level function to get the foldlevel for the "marker" method.
/// It scans the line for fold markers and updates the fold level accordingly.
///
/// # Arguments
/// * `wp` - Window handle
/// * `lnum` - Line number to check (1-based)
/// * `off` - Offset to add to lnum for actual buffer line
/// * `current_lvl` - Current fold level (from previous line)
/// * `marker_info` - Parsed fold marker information
///
/// # Returns
/// `FoldLevelMarkerResult` with updated levels and start count.
///
/// # Note
/// Requires that `current_lvl` is set to the fold level of the previous line!
/// This means you can't call this function twice on the same line without
/// passing the updated level.
#[must_use]
pub fn foldlevel_marker_impl(
    wp: WinHandle,
    lnum: LineNr,
    off: LineNr,
    current_lvl: c_int,
    marker_info: &FoldMarkerInfo,
) -> FoldLevelMarkerResult {
    if wp.is_null()
        || marker_info.start_marker.is_null()
        || marker_info.end_marker.is_null()
        || marker_info.start_marker_len == 0
    {
        return FoldLevelMarkerResult {
            lvl: current_lvl,
            lvl_next: current_lvl,
            start: 0,
        };
    }

    let buf = unsafe { BufHandle::from_ptr(win_ref(wp).w_buffer) };
    if buf.is_null() {
        return FoldLevelMarkerResult {
            lvl: current_lvl,
            lvl_next: current_lvl,
            start: 0,
        };
    }

    let line_ptr = unsafe { nvim_ml_get_buf(buf, lnum + off) };
    if line_ptr.is_null() {
        return FoldLevelMarkerResult {
            lvl: current_lvl,
            lvl_next: current_lvl,
            start: 0,
        };
    }

    let start_lvl = current_lvl;
    let mut lvl = current_lvl;
    let mut lvl_next = current_lvl;
    let mut start = 0;

    unsafe {
        // Cache marker info for speed
        let start_marker = marker_info.start_marker;
        let start_marker_len = marker_info.start_marker_len;
        let end_marker = marker_info.end_marker;
        let end_marker_len = marker_info.end_marker_len;

        // Get first characters of markers for quick comparison
        let cstart = *start_marker;
        // Start marker comparison begins from second character
        let start_marker_rest = start_marker.add(1);
        let cend = *end_marker;

        let mut s = line_ptr;
        while *s != 0 {
            if *s == cstart && start_marker_len > 0 {
                // Check if this is a start marker
                let matches = if start_marker_len == 1 {
                    true
                } else {
                    // Compare rest of marker (excluding first char we already matched)
                    strncmp_impl(s.add(1), start_marker_rest, start_marker_len - 1)
                };

                if matches {
                    // Found start marker
                    s = s.add(start_marker_len);

                    // Check for optional digit
                    if is_ascii_digit(*s) {
                        let n = atoi_impl(s);
                        if n > 0 {
                            lvl = n;
                            lvl_next = n;
                            start = (n - start_lvl).max(1);
                        }
                    } else {
                        lvl += 1;
                        lvl_next += 1;
                        start += 1;
                    }
                    continue;
                }
            }

            if *s == cend && end_marker_len > 0 {
                // Check if this is an end marker
                let matches = if end_marker_len == 1 {
                    true
                } else {
                    strncmp_impl(s.add(1), end_marker.add(1), end_marker_len - 1)
                };

                if matches {
                    // Found end marker
                    s = s.add(end_marker_len);

                    // Check for optional digit
                    if is_ascii_digit(*s) {
                        let n = atoi_impl(s);
                        if n > 0 {
                            lvl = n;
                            lvl_next = n - 1;
                            // Never start a fold with an end marker
                            lvl_next = lvl_next.min(start_lvl);
                        }
                    } else {
                        lvl_next -= 1;
                    }
                    continue;
                }
            }

            // Advance to next character (handle multi-byte)
            s = mb_ptr_adv(s);
        }
    }

    // The level can't go negative, must be missing a start marker.
    lvl_next = lvl_next.max(0);

    FoldLevelMarkerResult {
        lvl,
        lvl_next,
        start,
    }
}

/// Check if a character is an ASCII digit.
#[inline]
const fn is_ascii_digit(c: c_char) -> bool {
    c >= b'0' as c_char && c <= b'9' as c_char
}

/// Simple atoi implementation for parsing fold level digits.
unsafe fn atoi_impl(s: *const c_char) -> c_int {
    let mut result: c_int = 0;
    let mut p = s;
    while is_ascii_digit(*p) {
        result = result * 10 + c_int::from((*p) - b'0' as c_char);
        p = p.add(1);
    }
    result
}

/// Compare two strings up to n characters.
#[inline]
unsafe fn strncmp_impl(s1: *const c_char, s2: *const c_char, n: usize) -> bool {
    for i in 0..n {
        let c1 = *s1.add(i);
        let c2 = *s2.add(i);
        if c1 != c2 {
            return false;
        }
        if c1 == 0 {
            return true;
        }
    }
    true
}

/// Advance pointer past one (possibly multi-byte) character.
/// For simplicity, this treats each byte as a character.
/// The full implementation would use MB_PTR_ADV from mbyte.h.
#[inline]
const unsafe fn mb_ptr_adv(s: *const c_char) -> *const c_char {
    if (*s) == 0 {
        s
    } else {
        s.add(1)
    }
}

/// C OK/FAIL constants.
const OK: c_int = 1;

// ============================================================================
// Phase 3: Marker Manipulation Functions
// ============================================================================

/// Add `marker[0..marker_len]` to the end of line `lnum` in `buf`,
/// wrapped in `'commentstring'` if applicable.
///
/// Mirrors the C `foldAddMarker` function.
///
/// # Safety
/// `marker` must be a valid pointer to at least `marker_len` bytes.
pub unsafe fn fold_add_marker_impl(
    buf: BufHandle,
    lnum: LineNr,
    marker: *const c_char,
    marker_len: usize,
) {
    if buf.is_null() || marker.is_null() || marker_len == 0 {
        return;
    }

    // Save undo before modifying the line.
    if unsafe { nvim_fold_u_save(lnum) } != OK {
        return;
    }

    let line = unsafe { nvim_ml_get_buf(buf, lnum) };
    if line.is_null() {
        return;
    }

    #[allow(clippy::cast_sign_loss)]
    let line_len = unsafe { nvim_fold_ml_get_buf_len(buf, lnum) } as usize;

    let cms_raw = unsafe { nvim_fold_get_buf_b_p_cms(buf) };
    // Find "%s" placeholder in commentstring.
    let (cms_before, cms_after): (&[u8], &[u8]) = if cms_raw.is_null() {
        (&[], &[])
    } else {
        find_percent_s_in_cms(cms_raw)
    };

    let mut is_comment_c: c_int = 0;
    unsafe { nvim_fold_skip_comment(line, &raw mut is_comment_c) };
    let line_is_comment = is_comment_c != 0;

    // Calculate size: original line + marker + optional cms wrapper + NUL
    let added: usize;
    let new_len: usize;
    if cms_before.is_empty() || line_is_comment {
        // No commentstring wrapping: just append marker directly.
        new_len = line_len + marker_len + 1;
        added = marker_len;
    } else {
        // Append commentstring around marker: cms_before + marker + cms_after
        new_len = line_len + cms_before.len() + marker_len + cms_after.len() + 1;
        added = marker_len + cms_before.len() + cms_after.len();
    }

    // Allocate with xmalloc so C can free it via xfree.
    let newline = unsafe { nvim_fold_xmalloc(new_len).cast::<u8>() };
    if newline.is_null() {
        return;
    }

    unsafe {
        // Copy the original line content.
        std::ptr::copy_nonoverlapping(line.cast::<u8>(), newline, line_len);

        let dst = newline.add(line_len);
        if cms_before.is_empty() || line_is_comment {
            // Append marker directly.
            std::ptr::copy_nonoverlapping(marker.cast::<u8>(), dst, marker_len);
            *dst.add(marker_len) = 0;
        } else {
            // Append: cms_before + marker + cms_after + NUL
            std::ptr::copy_nonoverlapping(cms_before.as_ptr(), dst, cms_before.len());
            let dst2 = dst.add(cms_before.len());
            std::ptr::copy_nonoverlapping(marker.cast::<u8>(), dst2, marker_len);
            let dst3 = dst2.add(marker_len);
            std::ptr::copy_nonoverlapping(cms_after.as_ptr(), dst3, cms_after.len());
            *dst3.add(cms_after.len()) = 0;
        }

        // ml_replace_buf takes ownership of newline (copy=false).
        nvim_fold_ml_replace_buf(buf, lnum, newline.cast::<c_char>());

        if added > 0 {
            #[allow(clippy::cast_possible_truncation)]
            nvim_fold_extmark_splice_cols(buf, lnum - 1, line_len as c_int, 0, added as c_int);
        }
    }
}

/// Find the `%s` placeholder in a commentstring and return (before, after) slices.
///
/// For example `"/* %s */"` returns `(b"/* ", b" */")`.
/// If there is no `%s`, returns `(&[], &[])`.
fn find_percent_s_in_cms(cms_raw: *const c_char) -> (&'static [u8], &'static [u8]) {
    // SAFETY: cms_raw is a valid C string (from buf->b_p_cms).
    let cms_bytes = unsafe {
        let len = {
            let mut p = cms_raw;
            while *p != 0 {
                p = p.add(1);
            }
            #[allow(clippy::cast_sign_loss)]
            (p.offset_from(cms_raw) as usize)
        };
        std::slice::from_raw_parts(cms_raw.cast::<u8>(), len)
    };

    // Find "%s" in cms_bytes.
    cms_bytes
        .windows(2)
        .position(|w| w == b"%s")
        .map_or((&[], &[]), |pos| {
            let before = &cms_bytes[..pos];
            let after = &cms_bytes[pos + 2..];
            (before, after)
        })
}

/// Delete marker `marker[0..marker_len]` from the end of line `lnum` in `buf`.
///
/// Also removes the surrounding `'commentstring'` if it matches.
/// If the marker is not found, no error is emitted.
///
/// Mirrors the C `foldDelMarker` function.
///
/// # Safety
/// `marker` must be a valid pointer to at least `marker_len` bytes.
pub unsafe fn fold_del_marker_impl(
    buf: BufHandle,
    lnum: LineNr,
    marker: *const c_char,
    marker_len: usize,
) {
    if buf.is_null() || marker.is_null() || marker_len == 0 {
        return;
    }

    // End marker may be missing; fold extends below last line.
    let line_count = unsafe { nvim_fold_buf_get_line_count(buf) };
    if lnum > line_count {
        return;
    }

    let line = unsafe { nvim_ml_get_buf(buf, lnum) };
    if line.is_null() {
        return;
    }

    let cms_raw = unsafe { nvim_fold_get_buf_b_p_cms(buf) };
    let (cms_before, cms_after): (&[u8], &[u8]) = if cms_raw.is_null() {
        (&[], &[])
    } else {
        find_percent_s_in_cms(cms_raw)
    };

    // Scan through the line to find the marker.
    unsafe {
        let mut p = line;
        while *p != 0 {
            // Check if marker starts here.
            let matches = if marker_len == 1 {
                *p == *marker
            } else {
                strncmp_impl(p, marker, marker_len)
            };

            if !matches {
                p = p.add(1);
                continue;
            }

            // Found the marker. Include trailing digit if present.
            let mut len = marker_len;
            if is_ascii_digit(*p.add(len)) {
                len += 1;
            }

            // Calculate start of deletion (may expand to include commentstring).
            #[allow(clippy::cast_sign_loss)]
            let p_offset = p.offset_from(line) as usize;
            let mut del_start = p_offset;
            let mut del_len = len;

            if !cms_before.is_empty() {
                // Check if commentstring wraps this marker.
                if p_offset >= cms_before.len()
                    && strncmp_bytes(line.add(p_offset - cms_before.len()), cms_before)
                    && strncmp_bytes(p.add(len), cms_after)
                {
                    del_start = p_offset - cms_before.len();
                    del_len = cms_before.len() + len + cms_after.len();
                }
            }

            if nvim_fold_u_save(lnum) == OK {
                #[allow(clippy::cast_sign_loss)]
                let line_len = nvim_fold_ml_get_buf_len(buf, lnum) as usize;
                let new_len = line_len - del_len + 1;
                let newline = nvim_fold_xmalloc(new_len).cast::<u8>();
                if newline.is_null() {
                    break;
                }
                // Copy: text before marker + text after marker + NUL.
                std::ptr::copy_nonoverlapping(line.cast::<u8>(), newline, del_start);
                let src_after = line.add(del_start + del_len);
                let remaining = line_len - del_start - del_len;
                std::ptr::copy_nonoverlapping(
                    src_after.cast::<u8>(),
                    newline.add(del_start),
                    remaining,
                );
                *newline.add(del_start + remaining) = 0;

                nvim_fold_ml_replace_buf(buf, lnum, newline.cast::<c_char>());
                #[allow(clippy::cast_possible_truncation)]
                nvim_fold_extmark_splice_cols(
                    buf,
                    lnum - 1,
                    del_start as c_int,
                    del_len as c_int,
                    0,
                );
            }
            break;
        }
    }
}

/// Compare bytes at `s1` with the slice `expected`.
#[inline]
unsafe fn strncmp_bytes(s1: *const c_char, expected: &[u8]) -> bool {
    for (i, &exp_byte) in expected.iter().enumerate() {
        if (*s1.add(i)).cast_unsigned() != exp_byte {
            return false;
        }
    }
    true
}

/// Delete markers for fold `fp` (and nested folds if `recursive`).
///
/// Mirrors C `deleteFoldMarkers`.
pub fn delete_fold_markers_impl(wp: WinHandle, fp: FoldHandle, recursive: bool, lnum_off: LineNr) {
    if wp.is_null() || fp.is_null() {
        return;
    }

    let marker_info = parse_marker_impl(wp);
    if marker_info.start_marker.is_null() || marker_info.end_marker.is_null() {
        return;
    }

    delete_fold_markers_recurse(wp, fp, recursive, lnum_off, &marker_info);
}

/// Internal recursive implementation for `delete_fold_markers_impl`.
fn delete_fold_markers_recurse(
    wp: WinHandle,
    fp: FoldHandle,
    recursive: bool,
    lnum_off: LineNr,
    marker_info: &FoldMarkerInfo,
) {
    if recursive {
        let nested = unsafe { nvim_fold_get_fd_nested(fp) };
        let count = unsafe { nvim_ga_len(nested) };
        for i in 0..count {
            let child_fp = unsafe { nvim_ga_fold_at(nested, i) };
            if !child_fp.is_null() {
                let child_top = unsafe { nvim_fold_get_fd_top(fp) };
                delete_fold_markers_recurse(wp, child_fp, true, lnum_off + child_top, marker_info);
            }
        }
    }

    let buf = unsafe { BufHandle::from_ptr(win_ref(wp).w_buffer) };
    let fd_top = unsafe { nvim_fold_get_fd_top(fp) };
    let fd_len = unsafe { nvim_fold_get_fd_len(fp) };

    // Delete start marker.
    // SAFETY: marker_info fields are valid pointers from parse_marker_impl.
    unsafe {
        fold_del_marker_impl(
            buf,
            fd_top + lnum_off,
            marker_info.start_marker,
            marker_info.start_marker_len,
        );
        // Delete end marker.
        fold_del_marker_impl(
            buf,
            fd_top + lnum_off + fd_len - 1,
            marker_info.end_marker,
            marker_info.end_marker_len,
        );
    }
}

/// Create a fold from `start_lnum` to `end_lnum` (inclusive) by adding markers.
///
/// Mirrors C `foldCreateMarkers`.
pub fn fold_create_markers_impl(wp: WinHandle, start_lnum: LineNr, end_lnum: LineNr) {
    if wp.is_null() {
        return;
    }

    let buf = unsafe { BufHandle::from_ptr(win_ref(wp).w_buffer) };
    if buf.is_null() {
        return;
    }

    if unsafe { nvim_fold_buf_is_modifiable(buf) } == 0 {
        unsafe { nvim_fold_emsg_modifiable() };
        return;
    }

    let marker_info = parse_marker_impl(wp);
    if marker_info.start_marker.is_null() || marker_info.end_marker.is_null() {
        return;
    }

    // SAFETY: marker_info fields are valid pointers from parse_marker_impl.
    unsafe {
        // Add start marker to start_lnum.
        fold_add_marker_impl(
            buf,
            start_lnum,
            marker_info.start_marker,
            marker_info.start_marker_len,
        );
        // Add end marker to end_lnum.
        fold_add_marker_impl(
            buf,
            end_lnum,
            marker_info.end_marker,
            marker_info.end_marker_len,
        );
    }

    // Update both changes here to avoid cascading fold updates.
    unsafe {
        nvim_changed_lines(buf, start_lnum, 0, end_lnum, 0, false);

        let num_changed = i64::from(1 + end_lnum - start_lnum);
        nvim_buf_updates_send_changes(buf, start_lnum, num_changed, num_changed);
    }
}

// ============================================================================
// FFI Exports
// ============================================================================

/// Parse the 'foldmarker' option into start and end markers.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_parseMarker(wp: WinHandle) -> FoldMarkerInfo {
    parse_marker_impl(wp)
}

/// Calculate fold level for a line using marker method.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
/// The `marker_info` must be a valid pointer to a FoldMarkerInfo struct.
#[no_mangle]
pub unsafe extern "C" fn rs_foldlevelMarker(
    wp: WinHandle,
    lnum: LineNr,
    off: LineNr,
    current_lvl: c_int,
    marker_info: *const FoldMarkerInfo,
) -> FoldLevelMarkerResult {
    if marker_info.is_null() {
        return FoldLevelMarkerResult {
            lvl: current_lvl,
            lvl_next: current_lvl,
            start: 0,
        };
    }
    foldlevel_marker_impl(wp, lnum, off, current_lvl, &*marker_info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ascii_digit() {
        assert!(is_ascii_digit(b'0' as c_char));
        assert!(is_ascii_digit(b'5' as c_char));
        assert!(is_ascii_digit(b'9' as c_char));
        assert!(!is_ascii_digit(b'a' as c_char));
        assert!(!is_ascii_digit(b' ' as c_char));
        assert!(!is_ascii_digit(0));
    }

    #[test]
    fn test_atoi_impl() {
        unsafe {
            let s = c"123".as_ptr();
            assert_eq!(atoi_impl(s), 123);

            let s = c"0".as_ptr();
            assert_eq!(atoi_impl(s), 0);

            let s = c"42abc".as_ptr();
            assert_eq!(atoi_impl(s), 42);
        }
    }

    #[test]
    fn test_strncmp_impl() {
        unsafe {
            let s1 = c"hello".as_ptr();
            let s2 = c"hello".as_ptr();
            assert!(strncmp_impl(s1, s2, 5));

            let s1 = c"hello".as_ptr();
            let s2 = c"world".as_ptr();
            assert!(!strncmp_impl(s1, s2, 5));

            let s1 = c"hello".as_ptr();
            let s2 = c"help".as_ptr();
            assert!(strncmp_impl(s1, s2, 3));
            assert!(!strncmp_impl(s1, s2, 4));
        }
    }
}
