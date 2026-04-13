//! Fold display logic
//!
//! This module provides Rust implementations for fold display,
//! including fold text generation and visual representation.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};

use nvim_charset::rs_vim_isprintc;
use nvim_mbyte::utfc_ptr2len;
use nvim_memory::{xfree, xmalloc, xstrdup};
use nvim_window::WinHandle;

use crate::markers::parse_marker_impl;

/// Line number type
type LinenrT = i32;

// =============================================================================
// C accessor declarations for foldtext_cleanup
// =============================================================================

// VV_ (vim variable) index constants -- verified from eval_defs.h enum (0-indexed).
const VV_FOLDSTART: c_int = 23;
const VV_FOLDEND: c_int = 24;
const VV_FOLDDASHES: c_int = 25;
const VV_FOLDLEVEL: c_int = 26;

// Maximum fold level depth (from fold_shim.c MAX_LEVEL).
const MAX_LEVEL: usize = 20;

// Object type constants matching C kObjectType enum.
const K_OBJECT_TYPE_STRING: c_int = 4;
const K_OBJECT_TYPE_ARRAY: c_int = 5;

extern "C" {
    /// Get curbuf's commentstring option (b_p_cms).
    fn nvim_get_curbuf_b_p_cms() -> *const c_char;
    /// Get the current window handle.
    fn nvim_get_curwin() -> WinHandle;
    /// Skip whitespace at the beginning of a string.
    fn skipwhite(s: *const c_char) -> *const c_char;

    // Generic vim variable accessors
    /// Get a vim variable as a number by index.
    fn nvim_fold_get_vim_var_nr(vv_idx: c_int) -> i64;
    /// Set a vim variable as a number by index.
    fn nvim_fold_set_vim_var_nr(vv_idx: c_int, val: i64);
    /// Get a vim variable as a string by index.
    fn nvim_fold_get_vim_var_str(vv_idx: c_int) -> *const c_char;
    /// Set a vim variable as a string by index (from change_ffi.c).
    fn set_vim_var_string(vv_idx: c_int, val: *const c_char, len: c_int);

    // Generic line/buffer accessors (from change_ffi.c, move.c)
    /// Check if a line contains only whitespace (from change_ffi.c).
    fn linewhite(lnum: LinenrT) -> bool;
    /// Get a buffer line from curbuf (from change_ffi.c).
    fn nvim_ml_get(lnum: LinenrT) -> *const c_char;
    /// Get curbuf's line count (from move.c).
    fn nvim_curbuf_line_count() -> LinenrT;
    /// Concatenate two strings into a new xmalloc'd string (from change_ffi.c).
    #[link_name = "concat_str"]
    fn nvim_concat_str(s1: *const c_char, s2: *const c_char) -> *mut c_char;

    // NGETTEXT accessors (macro-wrapping required)
    /// Get the localized fold text format string.
    fn nvim_fold_ngettext_foldtext(count: c_int) -> *const c_char;
    /// Get the localized default fold text format string.
    fn nvim_fold_ngettext_default(count: c_int) -> *const c_char;

    // Phase 3 thin C accessors
    /// Save current_sctx into *out_saved and set current_sctx from wp's foldtext ctx.
    fn nvim_fold_save_sctx_foldtext(wp: *mut c_void, out_saved: *mut c_void);
    /// Restore current_sctx from *saved.
    fn nvim_fold_restore_sctx(saved: *mut c_void);
    /// Call parse_virt_text on obj->data.array; write result to vt_out.
    fn nvim_fold_parse_virt_text_from_obj(
        obj_ptr: *mut c_void,
        vt_out: *mut c_void,
        out_error: *mut c_int,
    );
    /// Call rs_eval_foldtext (Rust-to-Rust via extern "C").
    fn rs_eval_foldtext(wp: *mut c_void, out: *mut c_void);
    /// Free an Object (via api_free_object from api crate).
    #[link_name = "api_free_object"]
    fn rs_api_free_object(obj: nvim_api::Object);
    /// Get emsg_off value (from message.c).
    static mut emsg_off: c_int;
    /// Set emsg_off value (from message.c).
    /// Save curwin/curbuf and set to wp/wp->w_buffer. Returns old curwin.
    fn nvim_fold_save_curwin(wp: WinHandle) -> WinHandle;
    /// Restore curwin/curbuf from saved_win.
    fn nvim_fold_restore_curwin(saved_win: WinHandle);
    /// Get wp->w_p_fdt (foldtext option string).
    fn nvim_fold_win_get_p_fdt(wp: WinHandle) -> *const c_char;
    /// Get did_emsg (generic accessor from message.c).
    static mut did_emsg: c_int;
    /// Get next chunk from a VirtText (from decoration.c).
    fn nvim_next_virt_text_chunk(
        vt_ptr: *mut c_void,
        pos: *mut usize,
        attr: *mut c_int,
    ) -> *const c_char;
    /// Clear a VirtText.
    fn nvim_clear_virttext(vt_ptr: *mut std::ffi::c_void);
}

// =============================================================================
// Fold Display Constants
// =============================================================================

/// Default fold fill character
pub const FOLD_FILL_CHAR: u8 = b'-';

/// Fold closed indicator
pub const FOLD_CLOSED_CHAR: u8 = b'+';

/// Fold open indicator
pub const FOLD_OPEN_CHAR: u8 = b'-';

/// Maximum foldtext length
pub const FOLDTEXT_MAX_LEN: usize = 256;

// =============================================================================
// Fold Column Display
// =============================================================================

/// Fold column character types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldColumnChar {
    /// No fold at this position
    None = 0,
    /// Fold starts here (closed)
    ClosedStart = 1,
    /// Fold starts here (open)
    OpenStart = 2,
    /// Inside a fold (vertical bar)
    Inside = 3,
    /// Fold ends here
    End = 4,
    /// Nested fold indicator
    Nested = 5,
}

impl FoldColumnChar {
    /// Get display character for this type
    pub const fn as_char(self) -> u8 {
        match self {
            Self::None => b' ',
            Self::ClosedStart => b'+',
            Self::OpenStart => b'-',
            Self::Inside => b'|',
            Self::End => b'|',
            Self::Nested => b'>',
        }
    }

    /// Check if this represents a fold start
    pub const fn is_start(self) -> bool {
        matches!(self, Self::ClosedStart | Self::OpenStart)
    }

    /// Check if this represents a clickable fold
    pub const fn is_clickable(self) -> bool {
        matches!(self, Self::ClosedStart | Self::OpenStart)
    }
}

// =============================================================================
// Fold Display Info
// =============================================================================

/// Information about a fold for display purposes
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldDisplayInfo {
    /// First line of fold (1-based)
    pub first_line: LinenrT,
    /// Last line of fold (1-based)
    pub last_line: LinenrT,
    /// Number of lines in fold
    pub line_count: LinenrT,
    /// Fold level (1-based)
    pub level: c_int,
    /// Whether fold is closed
    pub closed: bool,
    /// Whether fold has nested folds
    pub has_nested: bool,
}

impl Default for FoldDisplayInfo {
    fn default() -> Self {
        Self {
            first_line: 0,
            last_line: 0,
            line_count: 0,
            level: 0,
            closed: false,
            has_nested: false,
        }
    }
}

impl FoldDisplayInfo {
    /// Create display info for a closed fold
    pub const fn closed(first: LinenrT, last: LinenrT, level: c_int) -> Self {
        Self {
            first_line: first,
            last_line: last,
            line_count: last - first + 1,
            level,
            closed: true,
            has_nested: false,
        }
    }

    /// Create display info for an open fold
    pub const fn open(first: LinenrT, last: LinenrT, level: c_int) -> Self {
        Self {
            first_line: first,
            last_line: last,
            line_count: last - first + 1,
            level,
            closed: false,
            has_nested: false,
        }
    }

    /// Check if this is a valid fold
    pub const fn is_valid(&self) -> bool {
        self.first_line > 0 && self.last_line >= self.first_line
    }
}

// =============================================================================
// Fold Text Generation
// =============================================================================

/// Format string for default fold text
pub const DEFAULT_FOLDTEXT_FORMAT: &str = "+-- %d lines: %s";

/// Components for fold text
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldTextComponents {
    /// Number of lines in fold
    pub line_count: LinenrT,
    /// Level of the fold
    pub level: c_int,
    /// Number of dashes to show (based on level)
    pub dash_count: c_int,
    /// Whether to show percentage
    pub show_percent: bool,
}

impl FoldTextComponents {
    /// Create components for a fold
    pub const fn new(line_count: LinenrT, level: c_int) -> Self {
        let dashes = level.saturating_sub(1);
        Self {
            line_count,
            level,
            dash_count: if dashes > 0 { dashes } else { 0 },
            show_percent: false,
        }
    }

    /// Calculate percentage of buffer that fold represents
    pub fn percent_of_buffer(&self, total_lines: LinenrT) -> u8 {
        if total_lines <= 0 {
            return 0;
        }
        let percent = (self.line_count as i64 * 100) / total_lines as i64;
        percent.min(100) as u8
    }
}

// =============================================================================
// Fold Column Configuration
// =============================================================================

/// Fold column configuration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldColumnConfig {
    /// Width of fold column (0 = hidden)
    pub width: c_int,
    /// Maximum level to show
    pub max_level: c_int,
    /// Whether to show level numbers
    pub show_numbers: bool,
    /// Fill character
    pub fill_char: u8,
}

impl Default for FoldColumnConfig {
    fn default() -> Self {
        Self {
            width: 0,
            max_level: 20,
            show_numbers: false,
            fill_char: b' ',
        }
    }
}

impl FoldColumnConfig {
    /// Create with width
    pub const fn with_width(width: c_int) -> Self {
        Self {
            width,
            max_level: 20,
            show_numbers: false,
            fill_char: b' ',
        }
    }

    /// Check if fold column is visible
    pub const fn is_visible(&self) -> bool {
        self.width > 0
    }

    /// Clamp level to displayable range
    pub const fn clamp_level(&self, level: c_int) -> c_int {
        if level < 1 {
            1
        } else if level > self.max_level {
            self.max_level
        } else {
            level
        }
    }
}

// =============================================================================
// Visual Range
// =============================================================================

/// A visual range in the display (for highlighting)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldVisualRange {
    /// Start column (0-based)
    pub start_col: c_int,
    /// End column (0-based, exclusive)
    pub end_col: c_int,
    /// Highlight group ID
    pub hl_id: c_int,
}

impl FoldVisualRange {
    /// Create a new visual range
    pub const fn new(start: c_int, end: c_int, hl_id: c_int) -> Self {
        Self {
            start_col: start,
            end_col: end,
            hl_id,
        }
    }

    /// Check if range is valid
    pub const fn is_valid(&self) -> bool {
        self.end_col > self.start_col
    }

    /// Get width of range
    pub const fn width(&self) -> c_int {
        self.end_col - self.start_col
    }
}

// =============================================================================
// Fold Highlight Info
// =============================================================================

/// Highlight information for fold display
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FoldHighlight {
    /// Highlight ID for fold text
    pub text_hl: c_int,
    /// Highlight ID for fold column
    pub column_hl: c_int,
    /// Highlight ID for fold sign
    pub sign_hl: c_int,
}

// =============================================================================
// foldtext_cleanup implementation
// =============================================================================

/// Check if a character byte is ASCII whitespace (space or tab).
#[inline]
const fn is_ascii_white(c: c_char) -> bool {
    c == b' ' as c_char || c == b'\t' as c_char
}

/// Check if a character byte is an ASCII digit.
#[inline]
const fn is_ascii_digit(c: c_char) -> bool {
    c >= b'0' as c_char && c <= b'9' as c_char
}

/// Compare `n` bytes starting at `s1` and `s2`.
/// Returns true if they are equal.
#[inline]
unsafe fn strncmp_bytes(s1: *const c_char, s2: *const c_char, n: usize) -> bool {
    for i in 0..n {
        if *s1.add(i) != *s2.add(i) {
            return false;
        }
        if *s1.add(i) == 0 {
            return true;
        }
    }
    true
}

/// Count bytes in a NUL-terminated string.
unsafe fn cstr_len(s: *const c_char) -> usize {
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
    }
    #[allow(clippy::cast_sign_loss)]
    let len = p.offset_from(s) as usize;
    len
}

/// Find the offset of `%s` within the first `len` bytes of `s`.
/// Returns `Some(offset)` if found, `None` otherwise.
unsafe fn find_percent_s(s: *const c_char, len: usize) -> Option<usize> {
    if len < 2 {
        return None;
    }
    (0..=(len - 2)).find(|&i| *s.add(i) == b'%' as c_char && *s.add(i + 1) == b's' as c_char)
}

/// Remove 'foldmarker' and 'commentstring' from `str` (in-place).
///
/// This is the Rust reimplementation of the C `foldtext_cleanup` function.
/// The string `str` is modified in-place: fold markers and the commentstring
/// wrapping them are removed.
///
/// # Safety
/// `str` must be a valid, NUL-terminated, mutable C string pointer.
/// The string may be modified in-place.
#[allow(clippy::too_many_lines)]
unsafe fn foldtext_cleanup_impl(s: *mut c_char) {
    if s.is_null() {
        return;
    }

    // Get curbuf's commentstring option and skip leading whitespace.
    let cms_raw = nvim_get_curbuf_b_p_cms();
    if cms_raw.is_null() {
        return;
    }
    let cms_start = skipwhite(cms_raw);

    // Compute strlen(cms_start), then trim trailing whitespace.
    let mut cms_slen = cstr_len(cms_start);
    while cms_slen > 0 && is_ascii_white(*(cms_start.add(cms_slen - 1))) {
        cms_slen -= 1;
    }

    // Locate "%s" in commentstring; split into start and end parts.
    let cms_end_ptr: *const c_char;
    let mut cms_end_len: usize = 0;
    if let Some(offset) = find_percent_s(cms_start, cms_slen) {
        let raw_cms_end = cms_start.add(offset);
        cms_end_len = cms_slen - offset;
        // exclude white space before "%s"
        let mut new_slen = offset;
        while new_slen > 0 && is_ascii_white(*(cms_start.add(new_slen - 1))) {
            new_slen -= 1;
        }
        cms_slen = new_slen;
        // skip "%s" and white space after it
        let after_pct_s = raw_cms_end.add(2);
        let cms_end_skip = skipwhite(after_pct_s);
        #[allow(clippy::cast_sign_loss)]
        let skip_count = cms_end_skip.offset_from(raw_cms_end) as usize;
        cms_end_len -= skip_count;
        cms_end_ptr = cms_end_skip;
    } else {
        cms_end_ptr = ptr::null();
    }

    // Parse fold markers for curwin
    let wp = nvim_get_curwin();
    let marker_info = parse_marker_impl(wp);

    let mut did1 = false;
    let mut did2 = false;

    let mut cur = s;
    while *cur != 0 {
        // Determine if current position is a fold marker (start or end).
        let marker_match_len: usize = if !marker_info.start_marker.is_null()
            && marker_info.start_marker_len > 0
            && strncmp_bytes(cur, marker_info.start_marker, marker_info.start_marker_len)
        {
            marker_info.start_marker_len
        } else if !marker_info.end_marker.is_null()
            && marker_info.end_marker_len > 0
            && strncmp_bytes(cur, marker_info.end_marker, marker_info.end_marker_len)
        {
            marker_info.end_marker_len
        } else {
            0
        };
        let mut len = marker_match_len;

        if len > 0 {
            // Found a fold marker; if followed by a digit, include it
            if is_ascii_digit(*(cur.add(len))) {
                len += 1;
            }

            // May remove 'commentstring' start before the marker.
            // Walk backwards past whitespace to find potential cms_start.
            if cms_slen > 0 {
                let mut p = cur;
                while p > s && is_ascii_white(*(p.sub(1))) {
                    p = p.sub(1);
                }
                #[allow(clippy::cast_sign_loss)]
                let back = p.offset_from(s) as usize;
                if back >= cms_slen && strncmp_bytes(p.sub(cms_slen), cms_start, cms_slen) {
                    // Include the whitespace and cms_start in the removal
                    #[allow(clippy::cast_sign_loss)]
                    let extra = cur.offset_from(p.sub(cms_slen)) as usize;
                    len += extra;
                    cur = p.sub(cms_slen);
                }
            }
        } else if !cms_end_ptr.is_null() {
            // No marker found; check for commentstring parts
            if !did1 && cms_slen > 0 && strncmp_bytes(cur, cms_start, cms_slen) {
                len = cms_slen;
                did1 = true;
            } else if !did2 && cms_end_len > 0 && strncmp_bytes(cur, cms_end_ptr, cms_end_len) {
                len = cms_end_len;
                did2 = true;
            }
        }

        if len != 0 {
            // Skip trailing whitespace after the removed region
            while is_ascii_white(*(cur.add(len))) {
                len += 1;
            }
            // STRMOVE(cur, cur + len)
            let src = cur.add(len);
            let src_len = cstr_len(src);
            ptr::copy(src, cur, src_len + 1);
        } else {
            // Advance past current character (MB_PTR_ADV)
            let remaining = cstr_len(cur);
            if remaining == 0 {
                break;
            }
            let slice = std::slice::from_raw_parts(cur as *const u8, remaining + 1);
            let advance = utfc_ptr2len(slice).max(1);
            cur = cur.add(advance);
        }
    }
}

// =============================================================================
// f_foldtext implementation
// =============================================================================

/// Allocate and format fold text header, appending line text.
///
/// Formats `txt` (e.g., "+-%s%3d line: ") with `dashes` and `count`, then
/// appends `line_text`. Returns an xmalloc'd string; `out_header_len` receives
/// the byte length of the formatted header portion (before `line_text`).
///
/// # Safety
/// All pointer arguments must be valid NUL-terminated C strings.
unsafe fn build_foldtext_impl(
    txt: *const c_char,
    dashes: *const c_char,
    count: c_int,
    line_text: *const c_char,
    out_header_len: *mut usize,
) -> *mut c_char {
    let txt_len = cstr_len(txt);
    let dashes_len = cstr_len(dashes);
    let line_text_len = cstr_len(line_text);
    // Extra 20 bytes: headroom for the formatted count and any extra characters.
    let total = txt_len + dashes_len + 20 + line_text_len + 1;
    let r = xmalloc(total).cast::<c_char>();
    libc::snprintf(r, total, txt, dashes, count);
    let header_len = cstr_len(r);
    *out_header_len = header_len;
    // Append line_text after the header.
    ptr::copy_nonoverlapping(line_text, r.add(header_len), line_text_len + 1);
    r
}

/// Format default fold text "+--%3d line(s) folded" into `buf`.
///
/// Writes into `buf` (which must have at least FOLD_TEXT_LEN bytes).
///
/// # Safety
/// `buf` must point to at least `FOLD_TEXT_LEN` bytes of writable memory.
unsafe fn format_default_foldtext(buf: *mut c_char, count: c_int) {
    let fmt = nvim_fold_ngettext_default(count);
    libc::snprintf(buf, FOLD_TEXT_LEN, fmt, count);
}

// =============================================================================
// Phase 3: eval_foldtext_full, set/clear vvars, virt_text_concat
// =============================================================================

/// Set v:foldstart, v:foldend, v:folddashes, v:foldlevel.
/// `level` is clamped to MAX_LEVEL (matching C behavior).
///
/// # Safety
/// Must be called with valid global state (vvars initialized).
unsafe fn set_fold_vvars_impl(start: LinenrT, end: LinenrT, level: c_int) {
    let clamped = level.min(MAX_LEVEL as c_int);
    // Build the dashes string in Rust: `clamped` '-' bytes + NUL.
    let mut dashes = [b'-' as c_char; MAX_LEVEL + 1];
    dashes[clamped as usize] = 0;
    nvim_fold_set_vim_var_nr(VV_FOLDSTART, i64::from(start));
    nvim_fold_set_vim_var_nr(VV_FOLDEND, i64::from(end));
    set_vim_var_string(VV_FOLDDASHES, dashes.as_ptr(), -1);
    nvim_fold_set_vim_var_nr(VV_FOLDLEVEL, i64::from(clamped));
}

/// Clear v:folddashes (set to NULL).
///
/// # Safety
/// Must be called with valid global state.
unsafe fn clear_fold_vvars_impl() {
    set_vim_var_string(VV_FOLDDASHES, ptr::null(), -1);
}

/// Concatenate all chunks from a VirtText into a single xmalloc'd string.
///
/// `vt_ptr` must be a valid `VirtText*`. Returns an xmalloc'd string.
///
/// # Safety
/// `vt_ptr` must point to a valid, initialized VirtText.
unsafe fn virt_text_concat_impl(vt_ptr: *mut c_void) -> *mut c_char {
    let mut text = xstrdup(c"".as_ptr());
    let mut pos: usize = 0;
    loop {
        let mut attr: c_int = 0;
        let chunk = nvim_next_virt_text_chunk(vt_ptr, &raw mut pos, &raw mut attr);
        if chunk.is_null() {
            break;
        }
        let new_text = nvim_concat_str(text, chunk);
        xfree(text.cast::<c_void>());
        text = new_text;
    }
    text
}

/// Evaluate 'foldtext' option for window `wp`.
///
/// This is the Rust reimplementation of C `nvim_fold_eval_foldtext_full`.
///
/// # Safety
/// All pointers must be valid. `vt_out` must point to a zero-initialized VirtText.
/// `buf_out` must point to a FOLD_TEXT_LEN-byte buffer.
unsafe fn eval_foldtext_full_impl(
    wp: WinHandle,
    vt_out: *mut c_void,
    buf_out: *mut c_char,
    out_text: *mut *mut c_char,
    out_has_virt_text: *mut c_int,
    out_had_error: *mut c_int,
) {
    *out_text = ptr::null_mut();
    *out_has_virt_text = 0;
    *out_had_error = 0;

    // Save curwin/curbuf and set to wp/wp->w_buffer.
    let save_curwin = nvim_fold_save_curwin(wp);
    // Save current_sctx and set to wp->w_p_script_ctx[kWinOptFoldtext].
    // sctx_T = {int sc_sid, int sc_seq, linenr_T sc_lnum, (pad 4), uint64_t sc_chan} = 24 bytes.
    let mut saved_sctx = [0u8; 24];
    nvim_fold_save_sctx_foldtext(wp.as_ptr(), saved_sctx.as_mut_ptr().cast::<c_void>());

    // emsg_off++: suppress error display during foldtext eval.
    let old_emsg_off = emsg_off;
    emsg_off = old_emsg_off + 1;

    // Call rs_eval_foldtext directly (Rust-to-Rust via extern "C").
    // Object = {c_int obj_type (4) + pad(4) + ObjectData (24)} = 32 bytes.
    let mut obj = nvim_api::Object {
        obj_type: 0,
        data: nvim_api::ObjectData { integer: 0 },
    };
    rs_eval_foldtext(wp.as_ptr(), ptr::addr_of_mut!(obj).cast::<c_void>());

    if obj.obj_type == K_OBJECT_TYPE_ARRAY {
        let mut had_parse_error: c_int = 0;
        nvim_fold_parse_virt_text_from_obj(
            ptr::addr_of_mut!(obj).cast::<c_void>(),
            vt_out,
            &raw mut had_parse_error,
        );
        if had_parse_error == 0 {
            *buf_out = 0; // NUL: signals "use vt_out"
            *out_has_virt_text = 1;
        } else {
            *out_had_error = 1;
        }
        // Free the Array data (not the VirtText chunks - those are separately allocated).
    } else if obj.obj_type == K_OBJECT_TYPE_STRING {
        // Transfer string data ownership to caller.
        *out_text = obj.data.string.data;
        // Prevent double-free: mark Object as NIL before freeing.
        obj.obj_type = 0; // kObjectTypeNil
    } else {
        *out_had_error = 1;
    }
    rs_api_free_object(obj);

    // emsg_off--
    emsg_off = old_emsg_off;

    if ((*out_text).is_null() && *out_has_virt_text == 0) || did_emsg != 0 {
        *out_had_error = 1;
    }

    // Restore curwin/curbuf and current_sctx.
    nvim_fold_restore_curwin(save_curwin);
    nvim_fold_restore_sctx(saved_sctx.as_mut_ptr().cast::<c_void>());
}

/// Implement the `foldtext()` VimL function.
///
/// Returns an xmalloc'd string (caller must xfree), or NULL if conditions
/// are not met (e.g., foldstart/foldend out of range).
unsafe fn f_foldtext_impl() -> *mut c_char {
    let foldstart = nvim_fold_get_vim_var_nr(VV_FOLDSTART) as LinenrT;
    let foldend = nvim_fold_get_vim_var_nr(VV_FOLDEND) as LinenrT;
    let dashes = nvim_fold_get_vim_var_str(VV_FOLDDASHES);
    let line_count = nvim_curbuf_line_count();

    if foldstart <= 0 || foldend > line_count {
        return ptr::null_mut();
    }

    // Find first non-empty line in the fold.
    let mut lnum = foldstart;
    while lnum < foldend {
        if !linewhite(lnum) {
            break;
        }
        lnum += 1;
    }

    // Find interesting text in this line, skip C comment-start.
    let raw_line = nvim_ml_get(lnum);
    let mut s = skipwhite(raw_line);

    // Skip C comment-start: "/*" or "//"
    if *s == b'/' as c_char && (*s.add(1) == b'*' as c_char || *s.add(1) == b'/' as c_char) {
        s = skipwhite(s.add(2));
        // If nothing remains and there's a next line, try it
        if *skipwhite(s) == 0 && lnum + 1 < foldend {
            s = skipwhite(nvim_ml_get(lnum + 1));
            if *s == b'*' as c_char {
                s = skipwhite(s.add(1));
            }
        }
    }

    let count = foldend - foldstart + 1;
    let txt = nvim_fold_ngettext_foldtext(count);

    let mut header_len: usize = 0;
    let r = build_foldtext_impl(txt, dashes, count, s, &raw mut header_len);

    // Remove 'foldmarker' and 'commentstring' from the appended line text
    rs_foldtext_cleanup(r.add(header_len));

    r
}

// =============================================================================
// get_foldtext implementation
// =============================================================================

/// Stack buffer size for fold text (matches C FOLD_TEXT_LEN = 51).
pub const FOLD_TEXT_LEN: usize = 51;

/// Result struct returned by rs_get_foldtext to drawline.c.
/// Mirrors what the old get_foldtext() returned, but with ownership flag.
#[repr(C)]
pub struct FoldTextResult {
    /// Pointer to fold text string. Either points into buf (not allocated)
    /// or is an xmalloc'd string (when text_is_allocated is true).
    pub text: *mut c_char,
    /// True if text was heap-allocated (caller must xfree it).
    pub text_is_allocated: bool,
    /// True if vt_out was populated with VirtText chunks.
    pub has_virt_text: bool,
}

impl Default for FoldTextResult {
    fn default() -> Self {
        Self {
            text: ptr::null_mut(),
            text_is_allocated: false,
            has_virt_text: false,
        }
    }
}

// Static state for get_foldtext (replaces C statics in get_foldtext).
static GOT_FDT_ERROR: AtomicBool = AtomicBool::new(false);
static LAST_WP: AtomicUsize = AtomicUsize::new(0);
static LAST_LNUM: AtomicI32 = AtomicI32::new(0);

/// Core implementation of get_foldtext logic.
///
/// `wp` -- window handle
/// `lnum` -- first line of fold (1-based)
/// `lnume` -- last line of fold (1-based)
/// `fi_level` -- fold level from foldinfo
/// `buf` -- stack buffer of FOLD_TEXT_LEN bytes (already memset to spaces)
/// `vt_out` -- pointer to VirtText to populate if 'foldtext' returns Array
///
/// Returns a FoldTextResult.
///
/// # Safety
/// `buf` must be valid with at least FOLD_TEXT_LEN bytes.
/// `vt_out` must be a valid pointer to a zero-initialized VirtText.
#[allow(clippy::too_many_arguments)]
unsafe fn get_foldtext_impl(
    wp: WinHandle,
    lnum: LinenrT,
    lnume: LinenrT,
    fi_level: c_int,
    buf: *mut c_char,
    vt_out: *mut c_void,
) -> FoldTextResult {
    let mut result = FoldTextResult::default();

    // Use the raw pointer value as a key to detect window changes.
    let wp_id = wp.as_ptr() as usize;

    let last_wp = LAST_WP.load(Ordering::Relaxed);
    let last_lnum = LAST_LNUM.load(Ordering::Relaxed);

    if last_wp == 0 || last_wp != wp_id || last_lnum > lnum || last_lnum == 0 {
        // window changed or first call - reset error flag
        GOT_FDT_ERROR.store(false, Ordering::Relaxed);
    }

    let got_fdt_error = GOT_FDT_ERROR.load(Ordering::Relaxed);
    let save_did_emsg = did_emsg;

    if !got_fdt_error {
        // A previous error should not abort evaluating 'foldexpr'
        did_emsg = 0;
    }

    let p_fdt = nvim_fold_win_get_p_fdt(wp);

    if !p_fdt.is_null() && *p_fdt != 0 {
        // Set v:foldstart, v:foldend, v:folddashes, v:foldlevel
        set_fold_vvars_impl(lnum, lnume, fi_level);

        // Skip evaluating 'foldtext' if there was an error last time
        if !got_fdt_error {
            let mut out_text: *mut c_char = ptr::null_mut();
            let mut out_has_virt_text: c_int = 0;
            let mut out_had_error: c_int = 0;

            eval_foldtext_full_impl(
                wp,
                vt_out,
                buf,
                &raw mut out_text,
                &raw mut out_has_virt_text,
                &raw mut out_had_error,
            );

            if out_has_virt_text != 0 {
                result.text = buf;
                result.has_virt_text = true;
            } else if !out_text.is_null() {
                result.text = out_text;
                result.text_is_allocated = true;
            }

            if result.text.is_null() || did_emsg != 0 {
                GOT_FDT_ERROR.store(true, Ordering::Relaxed);
            }
        }

        LAST_LNUM.store(lnum, Ordering::Relaxed);
        LAST_WP.store(wp_id, Ordering::Relaxed);
        clear_fold_vvars_impl();

        if did_emsg == 0 && save_did_emsg != 0 {
            did_emsg = save_did_emsg;
        }

        // Replace unprintable characters in text (if text is a string, not VirtText)
        if !result.text.is_null() && !result.has_virt_text {
            let mut p = result.text;
            loop {
                if *p == 0 {
                    break;
                }
                let remaining = {
                    let mut q = p;
                    while *q != 0 {
                        q = q.add(1);
                    }
                    #[allow(clippy::cast_sign_loss)]
                    let len = q.offset_from(p) as usize;
                    len
                };
                let slice = std::slice::from_raw_parts(p as *const u8, remaining + 1);
                let len = utfc_ptr2len(slice);

                if len > 1 {
                    let cp = nvim_mbyte::utf_ptr2char(slice);
                    if !rs_vim_isprintc(cp) {
                        break;
                    }
                    p = p.add(len - 1);
                } else if *p as u8 == b'\t' {
                    *p = b' ' as c_char;
                } else if nvim_charset::rs_ptr2cells(p) > 1 {
                    break;
                }
                p = p.add(1);
            }
            if *p != 0 {
                // Found unprintable chars - replace with transstr
                let new_text = nvim_charset::rs_transstr(result.text, true);
                if result.text_is_allocated {
                    xfree(result.text.cast::<c_void>());
                }
                result.text = new_text;
                result.text_is_allocated = true;
            }
        }
    }

    if result.text.is_null() {
        // Default fold text: "+--%3d line(s) folded"
        let count = lnume - lnum + 1;
        format_default_foldtext(buf, count);
        result.text = buf;
        result.text_is_allocated = false;
        result.has_virt_text = false;
    }

    result
}

/// Concatenate fold text result into a single xmalloc'd string.
/// Used by rs_f_foldtextresult as a replacement for nvim_get_foldtext.
///
/// # Safety
/// `wp` must be a valid window handle.
pub(crate) unsafe fn get_foldtext_concat_impl(
    wp: WinHandle,
    lnum: LinenrT,
    lnume: LinenrT,
    fi_level: c_int,
) -> *mut c_char {
    let mut buf = [b' ' as c_char; FOLD_TEXT_LEN];
    // Stack-allocate a VirtText (VirtText = kvec_t, initially zeroed = VIRTTEXT_EMPTY)
    // We pass an opaque *mut c_void to C.
    // Since we need a VirtText on the stack, we use a [u8; 24] (kvec_t is {size_t n, size_t m, T*items} = 24 bytes on 64-bit)
    let mut vt_storage = [0u8; 24];
    let vt_ptr = vt_storage.as_mut_ptr().cast::<c_void>();

    let result = get_foldtext_impl(wp, lnum, lnume, fi_level, buf.as_mut_ptr(), vt_ptr);

    if result.has_virt_text {
        // VirtText was populated - concatenate chunks.
        // kvec_t layout: { size_t n, size_t m, T *items } -- read n (first field) inline.
        let vt_n = vt_storage.as_ptr().cast::<usize>().read_unaligned();
        let text = if vt_n > 0 {
            virt_text_concat_impl(vt_ptr)
        } else {
            xstrdup(c"".as_ptr())
        };
        nvim_clear_virttext(vt_ptr);
        text
    } else if result.text_is_allocated {
        result.text
    } else if !result.text.is_null() {
        // text points into buf (stack) - must copy to heap
        xstrdup(result.text)
    } else {
        ptr::null_mut()
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Remove 'foldmarker' and 'commentstring' from `str` (in-place).
///
/// # Safety
/// `str` must be a valid, NUL-terminated, mutable C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_foldtext_cleanup(str: *mut c_char) {
    foldtext_cleanup_impl(str);
}

/// Implement the `foldtext()` VimL function.
///
/// Returns an xmalloc'd string that must be xfree'd by C, or NULL if the
/// fold variables are out of range.
///
/// # Safety
/// Must be called with valid C global state (curbuf, curwin, vvars set).
#[no_mangle]
pub unsafe extern "C" fn rs_f_foldtext_impl() -> *mut c_char {
    f_foldtext_impl()
}

/// FFI export: generate fold text for a closed fold line.
///
/// Replaces C `get_foldtext()`. Called from `drawline.c`.
///
/// `buf` must be a stack buffer of FOLD_TEXT_LEN bytes (pre-filled with spaces).
/// `vt_out` must be a pointer to a zero-initialized VirtText (stack-allocated
/// in the caller as `VirtText fold_vt = VIRTTEXT_EMPTY`).
///
/// Returns a FoldTextResult. The caller (drawline.c) handles:
/// - If text points into buf: use directly (no free needed)
/// - If text_is_allocated: must xfree text after use
/// - If has_virt_text: draw fold_vt, then xfree/clear as before
///
/// # Safety
/// All pointers must be valid. `vt_out` must remain live until caller clears it.
#[no_mangle]
pub unsafe extern "C" fn rs_get_foldtext(
    wp: WinHandle,
    lnum: LinenrT,
    lnume: LinenrT,
    fi_level: c_int,
    buf: *mut c_char,
    vt_out: *mut c_void,
) -> FoldTextResult {
    get_foldtext_impl(wp, lnum, lnume, fi_level, buf, vt_out)
}

/// FFI export: Get fold column character
#[no_mangle]
pub extern "C" fn rs_fold_column_char(char_type: FoldColumnChar) -> u8 {
    char_type.as_char()
}

/// FFI export: Check if fold column char is clickable
#[no_mangle]
pub extern "C" fn rs_fold_column_is_clickable(char_type: c_int) -> c_int {
    let char_type = match char_type {
        1 => FoldColumnChar::ClosedStart,
        2 => FoldColumnChar::OpenStart,
        _ => FoldColumnChar::None,
    };
    c_int::from(char_type.is_clickable())
}

/// FFI export: Create fold text components
#[no_mangle]
pub extern "C" fn rs_fold_text_components(line_count: LinenrT, level: c_int) -> FoldTextComponents {
    FoldTextComponents::new(line_count, level)
}

/// FFI export: Calculate fold percent of buffer
#[no_mangle]
pub extern "C" fn rs_fold_percent_of_buffer(line_count: LinenrT, total_lines: LinenrT) -> c_int {
    let components = FoldTextComponents::new(line_count, 1);
    c_int::from(components.percent_of_buffer(total_lines))
}

/// FFI export: Check if fold column is visible
#[no_mangle]
pub extern "C" fn rs_fold_column_is_visible(width: c_int) -> c_int {
    c_int::from(width > 0)
}

/// FFI export: Clamp fold level
#[no_mangle]
pub extern "C" fn rs_fold_clamp_level(level: c_int, max_level: c_int) -> c_int {
    let config = FoldColumnConfig {
        max_level,
        ..Default::default()
    };
    config.clamp_level(level)
}

/// FFI export: Create fold display info
#[no_mangle]
pub extern "C" fn rs_fold_display_info(
    first: LinenrT,
    last: LinenrT,
    level: c_int,
    closed: c_int,
) -> FoldDisplayInfo {
    if closed != 0 {
        FoldDisplayInfo::closed(first, last, level)
    } else {
        FoldDisplayInfo::open(first, last, level)
    }
}

/// FFI export: Check if display info is valid
#[no_mangle]
pub extern "C" fn rs_fold_display_is_valid(info: *const FoldDisplayInfo) -> c_int {
    if info.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*info).is_valid() })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_column_char() {
        assert_eq!(FoldColumnChar::None.as_char(), b' ');
        assert_eq!(FoldColumnChar::ClosedStart.as_char(), b'+');
        assert_eq!(FoldColumnChar::OpenStart.as_char(), b'-');
        assert_eq!(FoldColumnChar::Inside.as_char(), b'|');

        assert!(FoldColumnChar::ClosedStart.is_start());
        assert!(FoldColumnChar::OpenStart.is_start());
        assert!(!FoldColumnChar::Inside.is_start());

        assert!(FoldColumnChar::ClosedStart.is_clickable());
        assert!(!FoldColumnChar::Inside.is_clickable());
    }

    #[test]
    fn test_fold_display_info() {
        let closed = FoldDisplayInfo::closed(10, 20, 1);
        assert!(closed.is_valid());
        assert_eq!(closed.line_count, 11);
        assert!(closed.closed);

        let open = FoldDisplayInfo::open(5, 15, 2);
        assert!(open.is_valid());
        assert!(!open.closed);

        let invalid = FoldDisplayInfo::default();
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_fold_text_components() {
        let components = FoldTextComponents::new(100, 2);
        assert_eq!(components.line_count, 100);
        assert_eq!(components.level, 2);
        assert_eq!(components.dash_count, 1);

        assert_eq!(components.percent_of_buffer(1000), 10);
        assert_eq!(components.percent_of_buffer(100), 100);
        assert_eq!(components.percent_of_buffer(0), 0);
    }

    #[test]
    fn test_fold_column_config() {
        let config = FoldColumnConfig::default();
        assert!(!config.is_visible());

        let config = FoldColumnConfig::with_width(4);
        assert!(config.is_visible());
        assert_eq!(config.clamp_level(0), 1);
        assert_eq!(config.clamp_level(5), 5);
        assert_eq!(config.clamp_level(100), 20);
    }

    #[test]
    fn test_fold_visual_range() {
        let range = FoldVisualRange::new(0, 10, 1);
        assert!(range.is_valid());
        assert_eq!(range.width(), 10);

        let empty = FoldVisualRange::new(5, 5, 1);
        assert!(!empty.is_valid());
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_fold_column_char(FoldColumnChar::ClosedStart), b'+');
        assert_eq!(rs_fold_column_is_clickable(1), 1);
        assert_eq!(rs_fold_column_is_clickable(3), 0);

        assert_eq!(rs_fold_percent_of_buffer(50, 100), 50);
        assert_eq!(rs_fold_clamp_level(25, 20), 20);
    }
}
