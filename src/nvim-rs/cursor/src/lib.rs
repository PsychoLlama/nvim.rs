//! Cursor positioning and validation for Neovim
//!
//! This crate provides Rust implementations of cursor-related functions
//! from `src/nvim/cursor.c`. It handles:
//! - Cursor position validation
//! - Line and column bounds checking
//! - Virtual column handling
//! - Cursor state management
//!
//! The crate uses the opaque handle pattern for window access.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
#![allow(dead_code)] // Some extern declarations are pre-declared for future use

use std::ffi::{c_char, c_int, c_void};

use nvim_buffer::buf_struct::BufStruct;
use nvim_window::win_struct::{win_mut, win_ref};
use nvim_window::WinHandle;

/// Get `&BufStruct` from a `BufHandle`.
///
/// # Safety
/// `buf` must be a valid, non-null `buf_T` pointer.
#[inline]
unsafe fn bref(buf: BufHandle) -> &'static BufStruct {
    &*(buf.0.cast::<BufStruct>())
}

/// Get `&BufStruct` from a raw `*mut c_void`.
///
/// # Safety
/// `buf` must be a valid, non-null `buf_T` pointer.
#[inline]
unsafe fn bref_raw(buf: *mut c_void) -> &'static BufStruct {
    &*(buf.cast::<BufStruct>())
}

// =============================================================================
// Position Types
// =============================================================================

/// Cursor position type matching C `pos_T`.
///
/// This represents a position in a buffer with line number, column, and
/// virtual column addition for 'virtualedit'.
///
/// IMPORTANT: This must exactly match the C `pos_T` layout:
///   `linenr_T` lnum (`int32_t`) + `colnr_T` col (int) + `colnr_T` coladd (int) = 12 bytes
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CursorPos {
    /// Line number (1-based), matches C `linenr_T` = `int32_t`
    pub lnum: i32,
    /// Column (0-based byte offset)
    pub col: i32,
    /// Column addition for 'virtualedit'
    pub coladd: i32,
}

impl CursorPos {
    /// Create a new cursor position.
    #[inline]
    #[must_use]
    pub const fn new(lnum: i32, col: i32, coladd: i32) -> Self {
        Self { lnum, col, coladd }
    }

    /// Create a position at the beginning of a line.
    #[inline]
    #[must_use]
    pub const fn line_start(lnum: i32) -> Self {
        Self {
            lnum,
            col: 0,
            coladd: 0,
        }
    }

    /// Check if this position is valid (non-zero line number).
    #[inline]
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.lnum > 0
    }
}

/// Opaque buffer handle type matching C `buf_T *`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct BufHandle(*mut std::ffi::c_void);

impl BufHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// Cursor Validation Constants
// =============================================================================

/// Maximum column value (used for end of line positioning)
pub const MAXCOL: i32 = i32::MAX;

// =============================================================================
// Return Value Constants
// =============================================================================

/// Success return value
pub const OK: c_int = 1;
/// Failure return value
pub const FAIL: c_int = 0;

// =============================================================================
// Virtual Edit Flags (from option_vars.h)
// =============================================================================

/// Virtual edit flag: block mode
pub const VE_BLOCK: c_int = 0x01;
/// Virtual edit flag: insert mode
pub const VE_INSERT: c_int = 0x02;
/// Virtual edit flag: all modes
pub const VE_ALL: c_int = 0x04;
/// Virtual edit flag: one more than end of line
pub const VE_ONEMORE: c_int = 0x08;
/// Virtual edit flag: none set
pub const VE_NONE: c_int = 0x10;
/// Virtual edit flag: none or onemore
pub const VE_NONEU: c_int = 0x20;

// =============================================================================
// C Accessor Functions
// =============================================================================

/// `VALID_VIRTCOL` flag for `wp->w_valid` (0x04)
const VALID_VIRTCOL: c_int = 0x04;

extern "C" {
    /// Get length of a line in bytes (lnum is `linenr_T` = `int32_t`)
    fn nvim_buf_get_line_len(buf: *mut std::ffi::c_void, lnum: i32) -> i32;

    /// Check if 'virtualedit' allows cursor past end of line
    fn nvim_virtual_active(win: WinHandle) -> bool;

    /// Get current mode state (`MODE_INSERT`, `MODE_TERMINAL`, etc)
    fn nvim_get_state() -> c_int;

    /// Get 've' option flags for window
    fn nvim_get_ve_flags(win: WinHandle) -> c_int;

    /// Check if insert mode restart is pending
    static mut restart_edit: c_int;

    /// Check if Visual mode is active
    static mut VIsual_active: bool;

    /// Visual mode selection position (global `VIsual`)
    static mut VIsual: CursorPos;

    /// Current window pointer (global `curwin`)
    static mut curwin: WinHandle;

    /// Current buffer pointer (global `curbuf`, as raw `*mut c_void`)
    static mut curbuf: *mut c_void;

    /// Get 'selection' option first character
    fn nvim_get_p_sel_first() -> c_int;

    // -------------------------------------------------------------------------
    // Cursor/Screen Column Functions
    // -------------------------------------------------------------------------

    /// Wrapper for `getvvcol` - gets virtual column positions
    #[link_name = "getvvcol"]
    fn nvim_getvvcol(
        wp: WinHandle,
        pos: *const CursorPos,
        scol: *mut i32,
        ccol: *mut i32,
        ecol: *mut i32,
    );

    /// Wrapper for `set_valid_virtcol`
    #[link_name = "set_valid_virtcol"]
    fn nvim_set_valid_virtcol(wp: WinHandle, vcol: i32);

    /// Clear validity bits in `wp->w_valid`
    fn nvim_win_clear_valid_bits(wp: WinHandle, bits: c_int);

    /// Get window cursor pointer (`wp->w_cursor`)
    fn nvim_win_get_cursor_ptr(wp: WinHandle) -> *mut CursorPos;

    // -------------------------------------------------------------------------
    // Cursor Increment/Decrement Functions
    // -------------------------------------------------------------------------

    /// Wrapper for `inc_cursor()` - increment cursor position
    fn nvim_inc_cursor() -> c_int;

    /// Wrapper for `dec_cursor()` - decrement cursor position
    fn nvim_dec_cursor() -> c_int;

    // -------------------------------------------------------------------------
    // Folding Functions (from fold.c)
    // -------------------------------------------------------------------------

    /// Check if any folding is present in the window
    fn rs_hasAnyFolding(wp: WinHandle) -> c_int;

    /// Check if a line is folded, returns fold boundaries.
    /// (lnum and pointers are `linenr_T` = `int32_t`)
    fn nvim_hasFolding(wp: WinHandle, lnum: i32, firstp: *mut i32, lastp: *mut i32) -> c_int;

    // -------------------------------------------------------------------------
    // Window State Functions (for set_leftcol)
    // -------------------------------------------------------------------------

    /// Get window col offset
    #[link_name = "win_col_off"]
    fn nvim_win_col_off(wp: WinHandle) -> c_int;

    fn changed_cline_bef_curs(wp: WinHandle);

    /// Call `validate_virtcol`
    #[link_name = "validate_virtcol"]
    fn nvim_validate_virtcol(wp: WinHandle);

    /// Get `sidescrolloff` value
    fn rs_get_sidescrolloff_value(wp: WinHandle) -> c_int;

    /// Call `redraw_later`
    #[link_name = "redraw_later"]
    fn nvim_redraw_later(wp: WinHandle, r#type: c_int);

    /// Wrapper for `mark_mb_adjustpos`
    #[link_name = "mark_mb_adjustpos"]
    fn nvim_mark_mb_adjustpos(buf: *mut c_void, lp: *mut CursorPos);
}

// =============================================================================
// coladvance2 FFI declarations
// =============================================================================

extern "C" {
    /// Get pointer to a line in a buffer (`ml_get_buf`)
    #[link_name = "ml_get_buf"]
    fn nvim_ml_get_buf(buf: *mut c_void, lnum: i32) -> *mut c_char;

    /// Get length of a line in a buffer (`ml_get_buf_len`)
    #[link_name = "ml_get_buf_len"]
    fn nvim_ml_get_buf_len(buf: *mut c_void, lnum: i32) -> c_int;

    /// Get line width as displayed (`linetabsize(wp, lnum)`)
    fn linetabsize(wp: WinHandle, lnum: i32) -> c_int;

    /// Get line width including end-of-line (`linetabsize_eol(wp, lnum)`)
    fn linetabsize_eol(wp: WinHandle, lnum: i32) -> c_int;

    /// Get visual column positions for a position (`getvcol`)
    fn getvcol(
        wp: WinHandle,
        pos: *const CursorPos,
        scol: *mut i32,
        ccol: *mut i32,
        ecol: *mut i32,
    );

    /// Insert bytes into a line (`inserted_bytes`)
    fn inserted_bytes(lnum: i32, col: i32, old_col: i32, new_col: i32);

    /// Allocate zeroed memory of `size` bytes plus NUL (`xmallocz`)
    fn xmallocz(size: usize) -> *mut c_void;

    /// Replace a line in a buffer (`ml_replace`)
    fn ml_replace(lnum: i32, line: *mut c_char, copy: bool) -> c_int;

    /// Initialize charsize argument buffer
    fn init_charsize_arg(csarg: *mut c_void, wp: WinHandle, lnum: i32, line: *const c_char)
        -> bool;

    /// Compute character size (fast path)
    fn charsize_fast(
        csarg: *mut c_void,
        cur: *const c_char,
        vcol: c_int,
        cur_char: i32,
    ) -> CharSize;

    /// Compute character size (regular path)
    fn charsize_regular(
        csarg: *mut c_void,
        cur: *const c_char,
        vcol: c_int,
        cur_char: i32,
    ) -> CharSize;

    /// Advance to next character in string (slow path, used by `utfc_next`)
    fn utfc_next_impl(cur: StrCharInfo) -> StrCharInfo;

    /// Decode UTF-8 character at pointer (slow path, used by `utf_ptr2str_char_info`)
    fn utf_ptr2CharInfo_impl(p: *const u8, len: usize) -> i32;

    /// UTF-8 byte-length table
    static utf8len_tab: [u8; 256];

}

// =============================================================================
// coladvance2 struct definitions (mirroring C types)
// =============================================================================

/// Mirror of C `CharSize` from `plines.h`: `{int width; int head;}`.
#[repr(C)]
struct CharSize {
    width: c_int,
    head: c_int,
}

/// Mirror of C `CharInfo` from `mbyte_defs.h`: `{int32_t value; int len;}`.
#[repr(C)]
#[derive(Clone, Copy)]
struct CharInfo {
    value: i32,
    len: c_int,
}

/// Mirror of C `StrCharInfo` from `mbyte_defs.h`: `{char *ptr; CharInfo chr;}`.
#[repr(C)]
#[derive(Clone, Copy)]
struct StrCharInfo {
    ptr: *mut c_char,
    chr: CharInfo,
}

/// Opaque stack buffer for `CharsizeArg`.
/// The C struct size is ~264 bytes; 320 gives comfortable margin. Alignment >= 8.
#[repr(C, align(8))]
struct CharsizeArgBuf([u8; 320]);

// =============================================================================
// coladvance2 helper functions
// =============================================================================

/// Dispatch to `charsize_fast` or `charsize_regular` based on `cstype`.
/// `cstype = false` → fast, `cstype = true` → regular (matches C `CSType` bool convention).
#[inline]
unsafe fn win_charsize(
    cstype: bool,
    vcol: c_int,
    ptr: *const c_char,
    chr: i32,
    csarg: *mut c_void,
) -> CharSize {
    if cstype {
        charsize_regular(csarg, ptr, vcol, chr)
    } else {
        charsize_fast(csarg, ptr, vcol, chr)
    }
}

/// Inline `utf_ptr2StrCharInfo`: construct `StrCharInfo` for the character at `ptr`.
#[inline]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
unsafe fn utf_ptr2str_char_info(ptr: *mut c_char) -> StrCharInfo {
    let p = ptr.cast::<u8>();
    let first = *p;
    if first < 0x80 {
        StrCharInfo {
            ptr,
            chr: CharInfo {
                value: i32::from(first),
                len: 1,
            },
        }
    } else {
        let len = utf8len_tab[first as usize] as usize;
        let code_point = utf_ptr2CharInfo_impl(p, len);
        let (code_point, len) = if code_point < 0 {
            (code_point, 1)
        } else {
            (code_point, len as c_int)
        };
        StrCharInfo {
            ptr,
            chr: CharInfo {
                value: code_point,
                len,
            },
        }
    }
}

/// Inline `utfc_next`: advance to the next character in a string.
#[inline]
#[allow(clippy::cast_sign_loss)]
unsafe fn utfc_next(cur: StrCharInfo) -> StrCharInfo {
    let first = *cur.ptr as u8;
    if first < 0x80 {
        let next_ptr = cur.ptr.add(1);
        let next_first = *next_ptr as u8;
        if next_first < 0x80 {
            return StrCharInfo {
                ptr: next_ptr,
                chr: CharInfo {
                    value: i32::from(next_first),
                    len: 1,
                },
            };
        }
    }
    utfc_next_impl(cur)
}

// =============================================================================
// coladvance2 Rust implementation
// =============================================================================

/// Advance position `pos` to screen column `wcol` in window `wp`.
///
/// - `addspaces`: if true, insert spaces/break tabs to fill the gap (only for `curwin`)
/// - `finetune`: if true, set `pos.coladd` for virtual positions
///
/// Returns `OK` if the desired column is reached, `FAIL` otherwise.
///
/// # Safety
/// All pointers must be valid. `wp` must be a valid window handle.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::absurd_extreme_comparisons,
    clippy::manual_let_else
)]
unsafe fn coladvance2(
    wp: WinHandle,
    pos: *mut CursorPos,
    addspaces: bool,
    finetune: bool,
    wcol_arg: i32,
) -> c_int {
    debug_assert!(!addspaces || wp == curwin);

    let mut wcol = wcol_arg;
    let mut idx: c_int;
    let mut col: i32;
    let mut head: c_int = 0;

    let state = nvim_get_state();
    let one_more = c_int::from(
        (state & MODE_INSERT) != 0
            || (state & MODE_TERMINAL) != 0
            || restart_edit != 0
            || (VIsual_active && nvim_get_p_sel_first() != i32::from(b'o'))
            || ((nvim_get_ve_flags(wp) & VE_ONEMORE) != 0 && wcol < MAXCOL),
    );

    let buf = win_ref(wp).w_buffer;
    let line = nvim_ml_get_buf(buf, (*pos).lnum);
    let linelen = nvim_ml_get_buf_len(buf, (*pos).lnum);

    if wcol >= MAXCOL {
        idx = linelen - 1 + one_more;
        col = wcol;

        if (addspaces || finetune) && !VIsual_active {
            let ts = linetabsize(wp, (*pos).lnum) + one_more;
            win_mut(wp).w_curswant = ts;
            let cw = win_ref(wp).w_curswant;
            if cw > 0 {
                win_mut(wp).w_curswant = cw - 1;
            }
        }
    } else {
        let view_width = win_ref(wp).w_view_width;
        let width = view_width - nvim_win_col_off(wp);
        let mut csize: c_int = 0;

        if finetune && win_ref(wp).w_p_wrap() != 0 && view_width != 0 && wcol >= width && width > 0
        {
            csize = linetabsize_eol(wp, (*pos).lnum);
            if csize > 0 {
                csize -= 1;
            }

            if wcol / width > csize / width && ((state & MODE_INSERT) == 0 || wcol > csize + 1) {
                wcol = (csize / width + 1) * width - 1;
            }
        }

        let mut csarg_buf = CharsizeArgBuf([0u8; 320]);
        let csarg = (&raw mut csarg_buf).cast::<c_void>();
        let cstype = init_charsize_arg(csarg, wp, (*pos).lnum, line);
        let mut ci = utf_ptr2str_char_info(line);
        col = 0;
        while col <= wcol && *ci.ptr != 0 {
            let cs = win_charsize(cstype, col, ci.ptr, ci.chr.value, csarg);
            csize = cs.width;
            head = cs.head;
            col += cs.width;
            ci = utfc_next(ci);
        }
        idx = ci.ptr.offset_from(line) as c_int;

        if col > wcol || (!nvim_virtual_active(wp) && one_more == 0) {
            idx -= 1;
            csize -= head;
            col -= csize;
        }

        if nvim_virtual_active(wp)
            && addspaces
            && wcol >= 0
            && ((col != wcol && col != wcol + 1) || csize > 1)
        {
            if *line.add(idx as usize) == 0 {
                // Append spaces
                let correct = wcol - col;
                let newline_size = match (idx as usize).checked_add(correct as usize) {
                    Some(n) => n,
                    None => return FAIL,
                };
                let newline = xmallocz(newline_size).cast::<c_char>();
                std::ptr::copy_nonoverlapping(line, newline, idx as usize);
                std::ptr::write_bytes(newline.add(idx as usize), b' ', correct as usize);

                ml_replace((*pos).lnum, newline, false);
                inserted_bytes((*pos).lnum, idx, 0, correct);
                idx += correct;
                col = wcol;
            } else {
                // Break a tab
                let correct = wcol - col - csize + 1; // negative
                if -correct > csize {
                    return FAIL;
                }

                let n = match (linelen as usize - 1).checked_add(csize as usize) {
                    Some(v) => v,
                    None => return FAIL,
                };
                let newline = xmallocz(n).cast::<c_char>();
                // Copy first idx chars
                std::ptr::copy_nonoverlapping(line, newline, idx as usize);
                // Replace idx'th char with csize spaces
                std::ptr::write_bytes(newline.add(idx as usize), b' ', csize as usize);
                // Copy the rest of the line
                let rest = match (linelen as usize)
                    .checked_sub(idx as usize)
                    .and_then(|v| v.checked_sub(1))
                {
                    Some(v) => v,
                    None => return FAIL,
                };
                std::ptr::copy_nonoverlapping(
                    line.add(idx as usize + 1),
                    newline.add(idx as usize + csize as usize),
                    rest,
                );

                ml_replace((*pos).lnum, newline, false);
                inserted_bytes((*pos).lnum, idx, 1, csize);
                idx += csize - 1 + correct;
                col += correct;
            }
        }
    }

    (*pos).col = idx.max(0);
    (*pos).coladd = 0;

    if finetune {
        if wcol == MAXCOL {
            if one_more == 0 {
                let mut scol: i32 = 0;
                let mut ecol: i32 = 0;
                getvcol(wp, pos, &raw mut scol, std::ptr::null_mut(), &raw mut ecol);
                (*pos).coladd = ecol - scol;
            }
        } else {
            let b = wcol - col;
            if b > 0 && b < MAXCOL - 2 * win_ref(wp).w_view_width {
                (*pos).coladd = b;
            }
            col += b;
        }
    }

    let buf = win_ref(wp).w_buffer;
    nvim_mark_mb_adjustpos(buf, pos);

    if wcol < 0 || col < wcol {
        return FAIL;
    }
    OK
}

// =============================================================================
// Mode Constants (from state_defs.h)
// =============================================================================

/// Insert mode flag
pub const MODE_INSERT: c_int = 0x10;
/// Terminal mode flag
pub const MODE_TERMINAL: c_int = 0x2000;

// =============================================================================
// Redraw Constants (from drawscreen.h)
// =============================================================================

/// Buffer needs complete redraw
pub const UPD_NOT_VALID: c_int = 40;

// =============================================================================
// Position Comparison Functions
// =============================================================================

/// Compare two cursor positions.
///
/// Returns:
/// - negative if a < b
/// - 0 if a == b
/// - positive if a > b
#[no_mangle]
pub extern "C" fn rs_cursor_pos_cmp(a: &CursorPos, b: &CursorPos) -> c_int {
    if a.lnum != b.lnum {
        if a.lnum < b.lnum {
            -1
        } else {
            1
        }
    } else if a.col != b.col {
        if a.col < b.col {
            -1
        } else {
            1
        }
    } else if a.coladd != b.coladd {
        if a.coladd < b.coladd {
            -1
        } else {
            1
        }
    } else {
        0
    }
}

/// Check if two positions are equal.
#[no_mangle]
pub extern "C" fn rs_cursor_pos_equal(a: &CursorPos, b: &CursorPos) -> bool {
    a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd
}

/// Check if position a is less than position b.
#[no_mangle]
pub extern "C" fn rs_cursor_pos_less(a: &CursorPos, b: &CursorPos) -> bool {
    if a.lnum != b.lnum {
        a.lnum < b.lnum
    } else if a.col != b.col {
        a.col < b.col
    } else {
        a.coladd < b.coladd
    }
}

/// Check if position a is less than or equal to position b.
#[no_mangle]
pub extern "C" fn rs_cursor_pos_leq(a: &CursorPos, b: &CursorPos) -> bool {
    if a.lnum != b.lnum {
        a.lnum < b.lnum
    } else if a.col != b.col {
        a.col < b.col
    } else {
        a.coladd <= b.coladd
    }
}

// =============================================================================
// Cursor Position Helpers
// =============================================================================

/// Get the line count for a window's buffer.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_get_line_count(win: WinHandle) -> i32 {
    let buf = win_ref(win).w_buffer;
    if buf.is_null() {
        return 0;
    }
    bref_raw(buf).ml_line_count
}

/// Clamp a line number to valid buffer range.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_clamp_lnum(win: WinHandle, lnum: i32) -> i32 {
    let line_count = rs_cursor_get_line_count(win);
    if lnum < 1 {
        1
    } else if lnum > line_count {
        line_count.max(1)
    } else {
        lnum
    }
}

/// Check if a line number is valid for a window's buffer.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_valid_lnum(win: WinHandle, lnum: i32) -> bool {
    if lnum < 1 {
        return false;
    }
    let line_count = rs_cursor_get_line_count(win);
    lnum <= line_count
}

// =============================================================================
// Cursor Movement Helpers
// =============================================================================

/// Check if cursor can move up from current line.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_can_move_up(_win: WinHandle, lnum: i32) -> bool {
    lnum > 1
}

/// Check if cursor can move down from current line.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_can_move_down(win: WinHandle, lnum: i32) -> bool {
    let line_count = rs_cursor_get_line_count(win);
    lnum < line_count
}

/// Calculate the target line number when moving up by count lines.
/// Clamps to line 1 if count exceeds available lines.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_line_up(_win: WinHandle, lnum: i32, count: i32) -> i32 {
    let target = lnum - count;
    if target < 1 {
        1
    } else {
        target
    }
}

/// Calculate the target line number when moving down by count lines.
/// Clamps to last line if count exceeds available lines.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_line_down(win: WinHandle, lnum: i32, count: i32) -> i32 {
    let line_count = rs_cursor_get_line_count(win);
    let target = lnum + count;
    if target > line_count {
        line_count.max(1)
    } else {
        target
    }
}

/// Get the clamped column position for a line.
/// Returns the minimum of col and `line_len` - 1 (or 0 for empty lines).
/// When `allow_past_end` is true, allows col == `line_len`.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_clamp_col(
    win: WinHandle,
    lnum: i32,
    col: i32,
    allow_past_end: bool,
) -> i32 {
    let buf = win_ref(win).w_buffer;
    if buf.is_null() {
        return 0;
    }
    let line_len = nvim_buf_get_line_len(buf, lnum);
    if line_len == 0 {
        return 0;
    }

    let max_col = if allow_past_end {
        line_len
    } else {
        (line_len - 1).max(0)
    };

    if col < 0 {
        0
    } else if col > max_col {
        max_col
    } else {
        col
    }
}

/// Check if the `one_more` condition is true.
/// This allows cursor to be past end of line when:
/// - In Insert mode
/// - In Terminal mode
/// - Insert mode restart is pending
/// - Visual mode is active with 'selection' != "old"
/// - 'virtualedit' has onemore flag
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_one_more(win: WinHandle) -> bool {
    let state = nvim_get_state();
    let ve_flags = nvim_get_ve_flags(win);
    let visual_active = VIsual_active;
    let sel_first = nvim_get_p_sel_first();

    // Check each condition
    (state & MODE_INSERT) != 0
        || (state & MODE_TERMINAL) != 0
        || restart_edit != 0
        || (visual_active && sel_first != i32::from(b'o'))
        || (ve_flags & VE_ONEMORE) != 0
}

/// Check if position is at end of line (on the NUL byte).
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_at_eol(win: WinHandle, lnum: i32, col: i32) -> bool {
    let buf = win_ref(win).w_buffer;
    if buf.is_null() {
        return true;
    }
    let line_len = nvim_buf_get_line_len(buf, lnum);
    col >= line_len
}

/// Check if position is at beginning of line.
#[no_mangle]
pub extern "C" fn rs_cursor_at_bol(col: i32) -> bool {
    col == 0
}

/// Check if position is at first line of buffer.
#[no_mangle]
pub extern "C" fn rs_cursor_at_first_line(lnum: i32) -> bool {
    lnum <= 1
}

/// Check if position is at last line of buffer.
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_at_last_line(win: WinHandle, lnum: i32) -> bool {
    let line_count = rs_cursor_get_line_count(win);
    lnum >= line_count
}

// =============================================================================
// Character Access Functions
// =============================================================================

/// Get the character at the cursor position.
///
/// Returns the Unicode codepoint of the character under the cursor.
///
/// # Safety
/// Requires valid global state (curwin, curbuf).
#[must_use]
#[export_name = "gchar_cursor"]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_gchar_cursor() -> c_int {
    let cursor = nvim_win_get_cursor_ptr(curwin);
    let line = nvim_ml_get_buf(curbuf, (*cursor).lnum);
    let pos_ptr = line.add((*cursor).col as usize);
    nvim_mbyte::rs_utf_ptr2char(pos_ptr)
}

// =============================================================================
// Screen Column Functions
// =============================================================================

/// Get the screen column of the cursor in the current window.
///
/// Returns the virtual column position (accounting for tabs, wide characters, etc.).
///
/// # Safety
/// Requires valid global state (curwin).
#[must_use]
#[export_name = "getviscol"]
pub unsafe extern "C" fn rs_getviscol() -> c_int {
    let cursor = nvim_win_get_cursor_ptr(curwin);
    let mut x: i32 = 0;
    nvim_getvvcol(
        curwin,
        cursor,
        &raw mut x,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    x
}

/// Get the screen column for a given column and coladd in the cursor line.
///
/// # Arguments
/// * `col` - Column byte offset
/// * `coladd` - Virtual column addition
///
/// # Returns
/// The screen column position.
///
/// # Safety
/// Requires valid global state (curwin).
#[must_use]
#[export_name = "getviscol2"]
pub unsafe extern "C" fn rs_getviscol2(col: i32, coladd: i32) -> c_int {
    let cursor = nvim_win_get_cursor_ptr(curwin);

    // Build a temporary position with the cursor's line but specified col/coladd
    let pos = CursorPos {
        lnum: (*cursor).lnum,
        col,
        coladd,
    };

    let mut x: i32 = 0;
    nvim_getvvcol(
        curwin,
        &raw const pos,
        &raw mut x,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    x
}

// =============================================================================
// Column Advancement Functions
// =============================================================================

/// Return in `pos` the position of the cursor advanced to screen column `wcol`.
///
/// # Arguments
/// * `wp` - Window handle
/// * `pos` - Position to update
/// * `wcol` - Target screen column
///
/// # Returns
/// `OK` if desired column is reached, `FAIL` if not.
///
/// # Safety
/// `wp` and `pos` must be valid pointers.
#[export_name = "getvpos"]
pub unsafe extern "C" fn rs_getvpos(wp: WinHandle, pos: *mut CursorPos, wcol: i32) -> c_int {
    coladvance2(wp, pos, false, nvim_virtual_active(wp), wcol)
}

/// Try to advance the cursor to the specified screen column.
///
/// If virtual editing is enabled, fine tunes the cursor position.
/// All virtual positions off the end of a line share a cursor.col value
/// (equal to strlen(line)), beginning at coladd 0.
///
/// # Arguments
/// * `wp` - Window handle
/// * `wcol` - Target screen column
///
/// # Returns
/// `OK` if desired column is reached, `FAIL` if not.
///
/// # Safety
/// `wp` must be a valid window handle.
#[must_use]
#[export_name = "coladvance"]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_coladvance(wp: WinHandle, wcol: i32) -> c_int {
    let cursor = nvim_win_get_cursor_ptr(wp);
    let rc = coladvance2(wp, cursor, false, nvim_virtual_active(wp), wcol);

    if wcol == MAXCOL || rc == FAIL {
        nvim_win_clear_valid_bits(wp, VALID_VIRTCOL);
    } else {
        // Virtcol is valid when not on a TAB
        let buf = win_ref(wp).w_buffer;
        let is_tab =
            *nvim_ml_get_buf(buf, (*cursor).lnum).add((*cursor).col as usize) == b'\t' as c_char;
        if !is_tab {
            // Note: curwin is used here to match C behavior
            nvim_set_valid_virtcol(curwin, wcol);
        }
    }
    rc
}

/// Go to column "wcol", and add/insert white space as necessary to get the
/// cursor in that column.
///
/// # Arguments
/// * `wcol` - Target screen column
///
/// # Returns
/// `OK` if desired column is reached, `FAIL` if not.
///
/// # Safety
/// Requires valid global state (curwin).
#[must_use]
#[export_name = "coladvance_force"]
pub unsafe extern "C" fn rs_coladvance_force(wcol: i32) -> c_int {
    let cursor = nvim_win_get_cursor_ptr(curwin);
    let rc = coladvance2(curwin, cursor, true, false, wcol);
    if wcol == MAXCOL {
        nvim_win_clear_valid_bits(curwin, VALID_VIRTCOL);
    } else {
        nvim_set_valid_virtcol(curwin, wcol);
    }
    rc
}

// =============================================================================
// Cursor Line Accessor Functions
// =============================================================================

/// Get pointer to the cursor line.
///
/// # Returns
/// Pointer to the current line buffer.
///
/// # Safety
/// Requires valid global state (curwin, curbuf).
#[must_use]
#[export_name = "get_cursor_line_ptr"]
pub unsafe extern "C" fn rs_get_cursor_line_ptr() -> *mut c_char {
    let cursor = nvim_win_get_cursor_ptr(curwin);
    nvim_ml_get_buf(curbuf, (*cursor).lnum)
}

/// Get pointer to cursor position in the line.
///
/// # Returns
/// Pointer to the position within the current line.
///
/// # Safety
/// Requires valid global state (curwin, curbuf).
#[must_use]
#[export_name = "get_cursor_pos_ptr"]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_get_cursor_pos_ptr() -> *mut c_char {
    let cursor = nvim_win_get_cursor_ptr(curwin);
    let line = nvim_ml_get_buf(curbuf, (*cursor).lnum);
    line.add((*cursor).col as usize)
}

/// Get length of the cursor line (excluding NUL).
///
/// # Returns
/// Length in bytes of the cursor line.
///
/// # Safety
/// Requires valid global state (curwin, curbuf).
#[must_use]
#[export_name = "get_cursor_line_len"]
pub unsafe extern "C" fn rs_get_cursor_line_len() -> i32 {
    let cursor = nvim_win_get_cursor_ptr(curwin);
    nvim_ml_get_buf_len(curbuf, (*cursor).lnum)
}

/// Get length from cursor position to end of line (excluding NUL).
///
/// # Returns
/// Length in bytes from cursor to end of line.
///
/// # Safety
/// Requires valid global state (curwin, curbuf).
#[must_use]
#[export_name = "get_cursor_pos_len"]
pub unsafe extern "C" fn rs_get_cursor_pos_len() -> i32 {
    let cursor = nvim_win_get_cursor_ptr(curwin);
    nvim_ml_get_buf_len(curbuf, (*cursor).lnum) - (*cursor).col
}

// =============================================================================
// Character Access Functions
// =============================================================================

/// Return the character immediately before the cursor.
///
/// # Returns
/// The Unicode codepoint of the character before the cursor, or -1 if at
/// the start of the line.
///
/// # Safety
/// Requires valid global state (curwin, curbuf).
#[must_use]
#[export_name = "char_before_cursor"]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_char_before_cursor() -> c_int {
    let col = win_ref(curwin).w_cursor.col;
    if col <= 0 {
        return -1;
    }

    let cursor = nvim_win_get_cursor_ptr(curwin);
    let line = nvim_ml_get_buf(curbuf, (*cursor).lnum);
    // col is guaranteed > 0 here, so safe to cast
    let p = line.add(col as usize);
    // Find start of previous character
    let prev_len = nvim_mbyte::rs_utf_head_off(line, p.sub(1)) + 1;
    // prev_len is always >= 1 here (head_off returns >= 0, plus 1)
    nvim_mbyte::rs_utf_ptr2char(p.sub(prev_len as usize))
}

/// Make sure curwin->w_cursor is not on the NUL at the end of the line.
/// Allow it when in Visual mode and 'selection' is not "old".
///
/// # Safety
/// Requires valid global state (curwin, curbuf).
#[export_name = "adjust_cursor_col"]
pub unsafe extern "C" fn rs_adjust_cursor_col() {
    let col = win_ref(curwin).w_cursor.col;
    if col > 0 {
        let visual_active = VIsual_active;
        let sel_is_old = nvim_get_p_sel_first() == i32::from(b'o');
        // Only adjust if not in Visual mode or 'selection' is "old"
        if !visual_active || sel_is_old {
            // If cursor is on NUL (end of line), move back one character
            if rs_gchar_cursor() == 0 {
                win_mut(curwin).w_cursor.col = col - 1;
            }
        }
    }
}

// =============================================================================
// Cursor Validation Functions
// =============================================================================

/// Make sure `pos.lnum` and `pos.col` are valid in `buf`.
/// This allows for the col to be on the NUL byte.
///
/// # Safety
/// `buf` and `pos` must be valid pointers.
#[export_name = "check_pos"]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_check_pos(buf: BufHandle, pos: *mut CursorPos) {
    if buf.is_null() || pos.is_null() {
        return;
    }

    let line_count = i64::from(bref(buf).ml_line_count);

    // Clamp line number to buffer range
    // line_count fits in i64 from i32, but lnum is i32 (linenr_T)
    if i64::from((*pos).lnum) > line_count {
        (*pos).lnum = line_count as i32;
    }

    // Clamp column to line length (allowing position on NUL)
    if (*pos).col > 0 {
        let line_len = nvim_buf_get_line_len(buf.0, (*pos).lnum);
        if (*pos).col > line_len {
            (*pos).col = line_len;
        }
    }
}

/// Make sure `win->w_cursor.lnum` is valid.
///
/// # Safety
/// `win` must be a valid window handle.
#[export_name = "check_cursor_lnum"]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_check_cursor_lnum(win: WinHandle) {
    let buf = win_ref(win).w_buffer;
    if buf.is_null() {
        return;
    }

    let cursor_lnum = win_ref(win).w_cursor.lnum;
    let line_count = i64::from(bref_raw(buf).ml_line_count);

    if i64::from(cursor_lnum) > line_count {
        // If there is a closed fold at the end of the file, put the cursor in
        // its first line. Otherwise in the last line.
        // Inline nvim_check_folding_at_end: check fold at end of buffer
        let mut first_lnum: i32 = 0;
        if nvim_hasFolding(
            win,
            line_count as i32,
            &raw mut first_lnum,
            std::ptr::null_mut(),
        ) != 0
        {
            win_mut(win).w_cursor.lnum = first_lnum;
        } else {
            win_mut(win).w_cursor.lnum = line_count as i32;
        }
    }

    // Re-read in case it was modified above
    let cursor_lnum = win_ref(win).w_cursor.lnum;
    if cursor_lnum <= 0 {
        win_mut(win).w_cursor.lnum = 1;
    }
}

/// Make sure `win->w_cursor.col` is valid. Special handling of insert-mode.
///
/// # Safety
/// `win` must be a valid window handle.
#[export_name = "check_cursor_col"]
pub unsafe extern "C" fn rs_check_cursor_col(win: WinHandle) {
    let buf = win_ref(win).w_buffer;
    if buf.is_null() {
        return;
    }

    let oldcol = win_ref(win).w_cursor.col;
    let oldcoladd = oldcol + win_ref(win).w_cursor.coladd;
    let cur_ve_flags = nvim_get_ve_flags(win);
    let cursor_lnum = win_ref(win).w_cursor.lnum;

    let len = nvim_buf_get_line_len(buf, cursor_lnum);

    if len == 0 {
        win_mut(win).w_cursor.col = 0;
    } else if oldcol >= len {
        // Allow cursor past end-of-line when:
        // - in Insert mode or restarting Insert mode
        // - in Terminal mode
        // - in Visual mode and 'selection' isn't "old"
        // - 'virtualedit' is set
        let state = nvim_get_state();
        let visual_active = VIsual_active;
        let sel_first = nvim_get_p_sel_first();
        let virtual_active = nvim_virtual_active(win);

        if (state & MODE_INSERT) != 0
            || restart_edit != 0
            || (state & MODE_TERMINAL) != 0
            || (visual_active && sel_first != i32::from(b'o'))
            || (cur_ve_flags & VE_ONEMORE) != 0
            || virtual_active
        {
            win_mut(win).w_cursor.col = len;
        } else {
            win_mut(win).w_cursor.col = len - 1;
            // Move the cursor to the head byte.
            let cursor = nvim_win_get_cursor_ptr(win);
            nvim_mark_mb_adjustpos(buf, cursor);
        }
    } else if oldcol < 0 {
        win_mut(win).w_cursor.col = 0;
    }

    // If virtual editing is on, we can leave the cursor on the old position,
    // only we must set it to virtual. But don't do it when at the end of the
    // line.
    let newcol = win_ref(win).w_cursor.col;
    if oldcol == MAXCOL {
        win_mut(win).w_cursor.coladd = 0;
    } else if cur_ve_flags == VE_ALL {
        if oldcoladd > newcol {
            let mut coladd = oldcoladd - newcol;

            // Make sure that coladd is not more than the char width.
            // Not for the last character, coladd is then used when the cursor
            // is actually after the last character.
            if newcol + 1 < len {
                let cursor = nvim_win_get_cursor_ptr(win);
                let mut cs: i32 = 0;
                let mut ce: i32 = 0;
                getvcol(win, cursor, &raw mut cs, std::ptr::null_mut(), &raw mut ce);
                let char_width = ce - cs;
                if coladd > char_width {
                    coladd = char_width;
                }
            }
            win_mut(win).w_cursor.coladd = coladd;
        } else {
            // avoid weird number when there is a miscalculation or overflow
            win_mut(win).w_cursor.coladd = 0;
        }
    }
}

/// Make sure `win->w_cursor` is on a valid character.
///
/// # Safety
/// `win` must be a valid window handle.
#[export_name = "check_cursor"]
pub unsafe extern "C" fn rs_check_cursor(win: WinHandle) {
    rs_check_cursor_lnum(win);
    rs_check_cursor_col(win);
}

/// Check if `VIsual` position is valid, correct it if not.
/// Can be called when in Visual mode and a change has been made.
///
/// # Safety
/// Requires valid global state (curbuf).
#[export_name = "check_visual_pos"]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_check_visual_pos() {
    if curbuf.is_null() {
        return;
    }

    let line_count = i64::from(bref_raw(curbuf).ml_line_count);

    if i64::from(VIsual.lnum) > line_count {
        VIsual.lnum = line_count as i32;
        VIsual.col = 0;
        VIsual.coladd = 0;
    } else {
        let len = nvim_buf_get_line_len(curbuf, VIsual.lnum);
        if VIsual.col > len {
            VIsual.col = len;
            VIsual.coladd = 0;
        }
    }
}

// =============================================================================
// Cursor Increment/Decrement Functions
// =============================================================================

/// Increment the cursor position.
///
/// See `inc()` for return values:
/// - 0: still within line, moved to next char (but not at NUL)
/// - 1: moved to next line (first char)
/// - 2: moved to NUL at end of line
/// - -1: at end of file, cannot move
///
/// # Safety
/// Requires valid global state (curwin, curbuf).
#[unsafe(export_name = "inc_cursor")]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_inc_cursor() -> c_int {
    nvim_inc_cursor()
}

/// Decrement the cursor position.
///
/// See `dec()` for return values:
/// - 0: moved within line or corrected MAXCOL
/// - 1: moved to previous line (last char)
/// - -1: at start of file, cannot move
///
/// # Safety
/// Requires valid global state (curwin, curbuf).
#[unsafe(export_name = "dec_cursor")]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_dec_cursor() -> c_int {
    nvim_dec_cursor()
}

// =============================================================================
// Character Writing Functions
// =============================================================================

/// Write a character at the current cursor position.
///
/// This writes directly into the buffer's block, bypassing any undo or
/// change tracking. The caller is responsible for ensuring the line is
/// properly allocated and the cursor position is valid.
///
/// # Safety
/// Requires valid global state (curwin, curbuf).
/// The cursor must be at a valid position within the line.
#[export_name = "pchar_cursor"]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_pchar_cursor(c: c_char) {
    let cursor = nvim_win_get_cursor_ptr(curwin);
    let line = nvim_ml_get_buf(curbuf, (*cursor).lnum);
    // col is always >= 0 when cursor is at a valid position
    *line.add((*cursor).col as usize) = c;
}

// =============================================================================
// Cursor Relative Line Number Functions
// =============================================================================

/// Get the line number relative to the current cursor position.
///
/// This calculates the difference between `lnum` and the cursor position,
/// but only counts lines that can be visible (folded lines don't count).
///
/// # Arguments
/// * `wp` - Window handle
/// * `lnum` - Line number to get the relative position for
///
/// # Returns
/// The relative line number (negative if above cursor, positive if below).
///
/// # Safety
/// `wp` must be a valid window handle.
#[must_use]
#[export_name = "get_cursor_rel_lnum"]
pub unsafe extern "C" fn rs_get_cursor_rel_lnum(wp: WinHandle, lnum: i32) -> i32 {
    let cursor = win_ref(wp).w_cursor.lnum;

    // Fast path: same line or no folding
    if lnum == cursor || rs_hasAnyFolding(wp) == 0 {
        return lnum - cursor;
    }

    // Determine direction and range
    let (from_line, to_line) = if lnum < cursor {
        (lnum, cursor)
    } else {
        (cursor, lnum)
    };

    let mut retval: i32 = 0;
    let mut from = from_line;

    // Loop until we reach to_line, skipping folds
    while from < to_line {
        // If from is in a fold, set it to the last line of that fold
        let mut fold_last: i32 = 0;
        if nvim_hasFolding(wp, from, std::ptr::null_mut(), &raw mut fold_last) != 0 {
            from = fold_last;
        }
        from += 1;
        retval += 1;
    }

    // If to_line is in a closed fold, the line count is off by +1. Correct it.
    if from > to_line {
        retval -= 1;
    }

    if lnum < cursor {
        -retval
    } else {
        retval
    }
}

// =============================================================================
// Scroll/Leftcol Functions
// =============================================================================

/// Set `curwin->w_leftcol` to `leftcol`.
///
/// Adjusts the cursor position if needed to keep it visible on screen.
///
/// # Arguments
/// * `leftcol` - The new left column value
///
/// # Returns
/// `true` if the cursor was moved, `false` otherwise.
///
/// # Safety
/// Requires valid global state (curwin).
#[must_use]
#[export_name = "set_leftcol"]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_set_leftcol(leftcol: i32) -> bool {
    // Return quickly when there is no change
    if win_ref(curwin).w_leftcol == leftcol {
        return false;
    }

    win_mut(curwin).w_leftcol = leftcol;
    changed_cline_bef_curs(curwin);

    // Calculate the last visible column
    let view_width = win_ref(curwin).w_view_width;
    let col_off = nvim_win_col_off(curwin);
    let lastcol = i64::from(leftcol) + i64::from(view_width) - i64::from(col_off) - 1;

    nvim_validate_virtcol(curwin);

    let mut retval = false;

    // If the cursor is right or left of the screen, move it to last or first
    // visible character
    let siso = rs_get_sidescrolloff_value(curwin);
    let virtcol = i64::from(win_ref(curwin).w_virtcol);

    if virtcol > lastcol - i64::from(siso) {
        retval = true;
        let _ = rs_coladvance(curwin, (lastcol - i64::from(siso)) as i32);
    } else if virtcol < i64::from(leftcol) + i64::from(siso) {
        retval = true;
        let _ = rs_coladvance(curwin, leftcol + siso);
    }

    // If the start of the character under the cursor is not on the screen,
    // advance the cursor one more char. If this fails (last char of the
    // line) adjust the scrolling.
    let cursor = nvim_win_get_cursor_ptr(curwin);
    let mut s: i32 = 0;
    let mut e: i32 = 0;
    nvim_getvvcol(curwin, cursor, &raw mut s, std::ptr::null_mut(), &raw mut e);

    if i64::from(e) > lastcol {
        retval = true;
        let _ = rs_coladvance(curwin, s - 1);
    } else if s < leftcol {
        retval = true;
        if rs_coladvance(curwin, e + 1) == FAIL {
            // there isn't another character, adjust w_leftcol instead
            win_mut(curwin).w_leftcol = s;
            changed_cline_bef_curs(curwin);
        }
    }

    if retval {
        win_mut(curwin).w_set_curswant = 1;
    }
    nvim_redraw_later(curwin, UPD_NOT_VALID);
    retval
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_pos_new() {
        let pos = CursorPos::new(10, 5, 2);
        assert_eq!(pos.lnum, 10);
        assert_eq!(pos.col, 5);
        assert_eq!(pos.coladd, 2);
    }

    #[test]
    fn test_cursor_pos_line_start() {
        let pos = CursorPos::line_start(5);
        assert_eq!(pos.lnum, 5);
        assert_eq!(pos.col, 0);
        assert_eq!(pos.coladd, 0);
    }

    #[test]
    fn test_cursor_pos_is_valid() {
        assert!(CursorPos::new(1, 0, 0).is_valid());
        assert!(!CursorPos::new(0, 0, 0).is_valid());
        assert!(!CursorPos::new(-1, 0, 0).is_valid());
    }

    #[test]
    fn test_cursor_pos_default() {
        let pos = CursorPos::default();
        assert_eq!(pos.lnum, 0);
        assert_eq!(pos.col, 0);
        assert_eq!(pos.coladd, 0);
        assert!(!pos.is_valid());
    }

    #[test]
    fn test_cursor_pos_cmp() {
        let a = CursorPos::new(1, 0, 0);
        let b = CursorPos::new(2, 0, 0);
        assert!(rs_cursor_pos_cmp(&a, &b) < 0);
        assert!(rs_cursor_pos_cmp(&b, &a) > 0);
        assert_eq!(rs_cursor_pos_cmp(&a, &a), 0);

        // Same line, different column
        let c = CursorPos::new(1, 5, 0);
        assert!(rs_cursor_pos_cmp(&a, &c) < 0);

        // Same line and column, different coladd
        let d = CursorPos::new(1, 5, 2);
        assert!(rs_cursor_pos_cmp(&c, &d) < 0);
    }

    #[test]
    fn test_cursor_pos_equal() {
        let a = CursorPos::new(1, 5, 2);
        let b = CursorPos::new(1, 5, 2);
        let c = CursorPos::new(1, 5, 3);

        assert!(rs_cursor_pos_equal(&a, &b));
        assert!(!rs_cursor_pos_equal(&a, &c));
    }

    #[test]
    fn test_cursor_pos_less() {
        let a = CursorPos::new(1, 5, 2);
        let b = CursorPos::new(1, 5, 3);
        let c = CursorPos::new(2, 0, 0);

        assert!(rs_cursor_pos_less(&a, &b));
        assert!(rs_cursor_pos_less(&a, &c));
        assert!(!rs_cursor_pos_less(&a, &a));
    }

    #[test]
    fn test_cursor_pos_leq() {
        let a = CursorPos::new(1, 5, 2);
        let b = CursorPos::new(1, 5, 2);
        let c = CursorPos::new(1, 5, 3);

        assert!(rs_cursor_pos_leq(&a, &b));
        assert!(rs_cursor_pos_leq(&a, &c));
        assert!(!rs_cursor_pos_leq(&c, &a));
    }

    #[test]
    fn test_maxcol() {
        assert_eq!(MAXCOL, i32::MAX);
    }

    #[test]
    fn test_ve_flags() {
        assert_eq!(VE_BLOCK, 0x01);
        assert_eq!(VE_INSERT, 0x02);
        assert_eq!(VE_ALL, 0x04);
        assert_eq!(VE_ONEMORE, 0x08);
        assert_eq!(VE_NONE, 0x10);
        assert_eq!(VE_NONEU, 0x20);
    }

    #[test]
    fn test_mode_flags() {
        assert_eq!(MODE_INSERT, 0x10);
        assert_eq!(MODE_TERMINAL, 0x2000);
    }

    #[test]
    fn test_cursor_at_bol() {
        assert!(rs_cursor_at_bol(0));
        assert!(!rs_cursor_at_bol(1));
        assert!(!rs_cursor_at_bol(-1));
    }

    #[test]
    fn test_cursor_at_first_line() {
        assert!(rs_cursor_at_first_line(1));
        assert!(rs_cursor_at_first_line(0));
        assert!(rs_cursor_at_first_line(-1));
        assert!(!rs_cursor_at_first_line(2));
    }

    #[test]
    fn test_return_value_constants() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
    }

    #[test]
    fn test_cursor_pos_layout() {
        // CursorPos must match C pos_T: 3 x int32_t = 12 bytes
        assert_eq!(std::mem::size_of::<CursorPos>(), 12);
        assert_eq!(std::mem::align_of::<CursorPos>(), 4);
    }
}
