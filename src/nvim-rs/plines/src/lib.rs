//! Physical line calculations and display sizing for Neovim
//!
//! This crate provides Rust implementations of display calculation functions
//! from `src/nvim/drawscreen.c` and `src/nvim/plines.c`. It uses an opaque
//! handle pattern where `win_T*` pointers are treated as opaque handles,
//! with field access done through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)] // Character literals are safe ASCII values
#![allow(clippy::cast_sign_loss)] // We know the values are non-negative
#![allow(clippy::cast_lossless)] // Character literals fit in c_int
#![allow(clippy::cast_possible_truncation)] // OptInt values fit in c_int for these options
#![allow(clippy::similar_names)] // p_nu and p_rnu are standard Vim option names

use std::ffi::{c_char, c_int};

use nvim_buffer::BufHandle;
use nvim_window::WinHandle;

// C accessor functions
extern "C" {
    // Window display properties
    fn nvim_win_get_view_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_view_height(wp: WinHandle) -> c_int;
    fn nvim_win_fdccol_count(wp: WinHandle) -> c_int;
    fn nvim_win_is_curwin(wp: WinHandle) -> c_int;

    // Window options
    fn nvim_win_get_p_rnu(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_nu(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_nuw(wp: WinHandle) -> i64;
    fn nvim_win_get_p_stc(wp: WinHandle) -> *const c_char;
    fn nvim_win_get_p_cocu(wp: WinHandle) -> *const c_char;
    fn nvim_win_get_minscwidth(wp: WinHandle) -> c_int;

    // Window cache fields
    fn nvim_win_get_nrwidth_line_count(wp: WinHandle) -> i64;
    fn nvim_win_set_nrwidth_line_count(wp: WinHandle, val: i64);
    fn nvim_win_get_nrwidth_width(wp: WinHandle) -> c_int;
    fn nvim_win_set_nrwidth_width(wp: WinHandle, val: c_int);
    fn nvim_win_set_statuscol_line_count(wp: WinHandle, val: i64);

    // Buffer properties
    fn nvim_win_buf_line_count(wp: WinHandle) -> i64;
    fn nvim_win_buf_meta_total_signtext(wp: WinHandle) -> c_int;

    // Global state
    fn nvim_get_p_wmw() -> i64;
    fn nvim_get_State() -> c_int;
    fn nvim_get_real_state() -> c_int;

    // String utilities
    fn nvim_vim_strchr(s: *const c_char, c: c_int) -> *const c_char;

    // Buffer properties for charsize
    fn nvim_buf_get_p_ts(buf: BufHandle) -> i64;
    fn nvim_buf_get_p_vts_array(buf: BufHandle) -> *const c_int;

    // Window properties for win_may_fill
    fn nvim_win_get_p_diff(wp: WinHandle) -> c_int;
    fn nvim_win_buf_meta_total_lines(wp: WinHandle) -> c_int;
    fn nvim_diffopt_filler() -> c_int;

    // Existing Rust functions we can call
    fn rs_tabstop_padding(col: c_int, ts: i64, vts: *const c_int) -> c_int;
    fn rs_ptr2cells(p: *const c_char) -> c_int;

    // Window properties for win_col_off
    fn nvim_win_is_cmdwin(wp: WinHandle) -> c_int;
    fn nvim_win_get_scwidth(wp: WinHandle) -> c_int;
    fn nvim_get_p_cpo() -> *const c_char;

    // Window properties for showbreak
    fn nvim_win_get_p_sbr(wp: WinHandle) -> *const c_char;
    fn nvim_get_p_sbr() -> *const c_char;
    fn nvim_get_empty_string_option() -> *const c_char;

    // Window properties for sms_marker_overlap
    fn nvim_win_get_p_list(wp: WinHandle) -> c_int;
    fn nvim_win_get_lcs_prec(wp: WinHandle) -> u32;

    // Window properties for win_cursorline_standout
    fn nvim_win_get_p_cul(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_cole(wp: WinHandle) -> i64;

    // Window properties for scrolloff
    fn nvim_win_get_p_so(wp: WinHandle) -> i64;
    fn nvim_win_get_p_siso(wp: WinHandle) -> i64;
    fn nvim_get_p_so() -> i64;
    fn nvim_get_p_siso() -> i64;

    // Terminal mode check
    fn nvim_win_buf_is_terminal(wp: WinHandle) -> c_int;

    // Global options for statusline/winbar
    fn nvim_get_p_ls() -> i64;
    fn nvim_get_p_wbr_empty() -> c_int;
}

// Mode constants (matching Neovim's state.h)
const MODE_VISUAL: c_int = 0x02;
const MODE_INSERT: c_int = 0x10;
const MODE_NORMAL: c_int = 0x01;
const MODE_CMDLINE: c_int = 0x04;
const MODE_TERMINAL: c_int = 0x1000;

// Statusline constants (matching Neovim's window.h)
const STATUS_HEIGHT: c_int = 1;

// Sign column constants (matching Neovim's optionstr.c)
const SCL_NUM: c_int = -1;

// Display constants
const SIGN_WIDTH: c_int = 2;
const CPO_NUMCOL: c_int = b'n' as c_int;

// ============================================================================
// Display Calculations
// ============================================================================

/// Compute the width of the foldcolumn.
///
/// Based on 'foldcolumn' and how much space is available for window "wp",
/// minus "col".
#[inline]
fn compute_foldcolumn_impl(wp: WinHandle, col: c_int) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let fdc = nvim_win_fdccol_count(wp);
        let is_curwin = nvim_win_is_curwin(wp) != 0;
        let p_wmw = nvim_get_p_wmw() as c_int;

        let wmw = if is_curwin && p_wmw == 0 { 1 } else { p_wmw };
        let view_width = nvim_win_get_view_width(wp);
        let n = view_width - (col + wmw);

        // MIN(fdc, n)
        if fdc < n {
            fdc
        } else {
            n
        }
    }
}

/// Return the width of the 'number' and 'relativenumber' column.
///
/// Caller may need to check if 'number' or 'relativenumber' is set.
/// Otherwise it depends on 'numberwidth' and the line count.
#[inline]
fn number_width_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let p_rnu = nvim_win_get_p_rnu(wp) != 0;
        let p_nu = nvim_win_get_p_nu(wp) != 0;

        // Determine the line count to use
        let lnum: i64 = if p_rnu && !p_nu {
            // cursor line shows "0"
            nvim_win_get_view_height(wp) as i64
        } else {
            // cursor line shows absolute line number
            nvim_win_buf_line_count(wp)
        };

        // Check cache
        let cached_line_count = nvim_win_get_nrwidth_line_count(wp);
        if lnum == cached_line_count {
            return nvim_win_get_nrwidth_width(wp);
        }
        nvim_win_set_nrwidth_line_count(wp, lnum);

        // Check for 'statuscolumn'
        let p_stc = nvim_win_get_p_stc(wp);
        if !p_stc.is_null() && *p_stc != 0 {
            nvim_win_set_statuscol_line_count(wp, 0); // make sure width is re-estimated
            let width = i32::from(p_nu || p_rnu) * (nvim_win_get_p_nuw(wp) as c_int);
            nvim_win_set_nrwidth_width(wp, width);
            return width;
        }

        // Count digits
        let mut temp_lnum = lnum;
        let mut n: c_int = 0;
        loop {
            temp_lnum /= 10;
            n += 1;
            if temp_lnum <= 0 {
                break;
            }
        }

        // 'numberwidth' gives the minimal width plus one
        let p_nuw = nvim_win_get_p_nuw(wp) as c_int;
        let nuw_minus_one = if p_nuw > 1 { p_nuw - 1 } else { 0 };
        if n < nuw_minus_one {
            n = nuw_minus_one;
        }

        // If 'signcolumn' is set to 'number' and there is a sign to display,
        // then the minimal width for the number column is 2.
        let has_signs = nvim_win_buf_meta_total_signtext(wp) != 0;
        let minscwidth = nvim_win_get_minscwidth(wp);
        if n < 2 && has_signs && minscwidth == SCL_NUM {
            n = 2;
        }

        nvim_win_set_nrwidth_width(wp, n);
        n
    }
}

/// Return true if the cursor line in window "wp" may be concealed,
/// according to the 'concealcursor' option.
#[inline]
fn conceal_cursor_line_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let p_cocu = nvim_win_get_p_cocu(wp);
        if p_cocu.is_null() || *p_cocu == 0 {
            return false;
        }

        let real_state = nvim_get_real_state();
        let state = nvim_get_State();

        let c: c_int = if (real_state & MODE_VISUAL) != 0 {
            b'v' as c_int
        } else if (state & MODE_INSERT) != 0 {
            b'i' as c_int
        } else if (state & MODE_NORMAL) != 0 {
            b'n' as c_int
        } else if (state & MODE_CMDLINE) != 0 {
            b'c' as c_int
        } else {
            return false;
        };

        !nvim_vim_strchr(p_cocu, c).is_null()
    }
}

// Tab character constant
const TAB: i32 = 0x09;

// Invalid byte display width (from mbyte.h)
const K_INVALID_BYTE_CELLS: c_int = 4;

/// Get the number of cells taken up on the screen at given virtual column.
///
/// Handles tabs, invalid bytes, and normal characters.
#[inline]
fn charsize_nowrap_impl(
    buf: BufHandle,
    cur: *const c_char,
    use_tabstop: bool,
    vcol: c_int,
    cur_char: i32,
) -> c_int {
    if buf.is_null() {
        return 1;
    }

    unsafe {
        if cur_char == TAB && use_tabstop {
            let ts = nvim_buf_get_p_ts(buf);
            let vts = nvim_buf_get_p_vts_array(buf);
            rs_tabstop_padding(vcol, ts, vts)
        } else if cur_char < 0 {
            K_INVALID_BYTE_CELLS
        } else {
            rs_ptr2cells(cur)
        }
    }
}

/// Check if there may be filler lines anywhere in window "wp".
#[inline]
fn win_may_fill_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let p_diff = nvim_win_get_p_diff(wp) != 0;
        let diffopt_fill = nvim_diffopt_filler() != 0;
        let has_meta_lines = nvim_win_buf_meta_total_lines(wp) != 0;

        (p_diff && diffopt_fill) || has_meta_lines
    }
}

/// Return the offset for the window's first column.
///
/// Takes into account line numbers, fold column, sign column, and command-line window.
#[inline]
fn win_col_off_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let p_nu = nvim_win_get_p_nu(wp) != 0;
        let p_rnu = nvim_win_get_p_rnu(wp) != 0;
        let p_stc = nvim_win_get_p_stc(wp);
        let has_stc = !p_stc.is_null() && *p_stc != 0;

        // Number column contribution
        let num_col = if p_nu || p_rnu || has_stc {
            rs_number_width(wp) + c_int::from(!has_stc)
        } else {
            0
        };

        // Command-line window adds 1 column
        let cmdwin_col = c_int::from(nvim_win_is_cmdwin(wp) != 0);

        // Fold column
        let fdc = nvim_win_fdccol_count(wp);

        // Sign column
        let scwidth = nvim_win_get_scwidth(wp);

        num_col + cmdwin_col + fdc + (scwidth * SIGN_WIDTH)
    }
}

/// Return the offset for wrapped lines (second screen line onwards).
///
/// It's positive if 'number' or 'relativenumber' is on and 'n' is in 'cpoptions'.
#[inline]
fn win_col_off2_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let p_nu = nvim_win_get_p_nu(wp) != 0;
        let p_rnu = nvim_win_get_p_rnu(wp) != 0;
        let p_stc = nvim_win_get_p_stc(wp);
        let has_stc = !p_stc.is_null() && *p_stc != 0;

        if (p_nu || p_rnu || has_stc) && !nvim_vim_strchr(nvim_get_p_cpo(), CPO_NUMCOL).is_null() {
            rs_number_width(wp) + c_int::from(!has_stc)
        } else {
            0
        }
    }
}

/// Check that virtual column "vcol" is in the rightmost column of window "wp".
///
/// Used for determining if a double-width character wraps at the end of a line.
#[inline]
fn in_win_border_impl(wp: WinHandle, vcol: c_int) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let view_width = nvim_win_get_view_width(wp);
        if view_width == 0 {
            // there is no border
            return false;
        }

        // width of first line (after line number, etc.)
        let width1 = view_width - rs_win_col_off(wp);

        if vcol < width1 - 1 {
            return false;
        }

        if vcol == width1 - 1 {
            return true;
        }

        // width of further lines
        let width2 = width1 + rs_win_col_off2(wp);

        if width2 <= 0 {
            return false;
        }

        (vcol - width1) % width2 == width2 - 1
    }
}

/// Get the 'showbreak' value for a window.
///
/// Returns window-local showbreak if set, otherwise global showbreak.
/// Returns empty string if window showbreak is "NONE".
#[inline]
fn get_showbreak_value_impl(wp: WinHandle) -> *const c_char {
    if wp.is_null() {
        unsafe { return nvim_get_p_sbr(); }
    }

    unsafe {
        let w_sbr = nvim_win_get_p_sbr(wp);

        // If window showbreak is NULL or empty, use global
        if w_sbr.is_null() || *w_sbr == 0 {
            return nvim_get_p_sbr();
        }

        // Check for "NONE" (case-sensitive)
        // "NONE" = 'N', 'O', 'N', 'E', '\0'
        if *w_sbr == b'N' as c_char
            && *w_sbr.add(1) == b'O' as c_char
            && *w_sbr.add(2) == b'N' as c_char
            && *w_sbr.add(3) == b'E' as c_char
            && *w_sbr.add(4) == 0
        {
            return nvim_get_empty_string_option();
        }

        w_sbr
    }
}

/// Calculate the smoothscroll marker overlap.
///
/// Calculates how much the 'listchars' "precedes" or 'smoothscroll' "<<<"
/// marker overlaps with buffer text for window "wp".
/// Parameter "extra2" should be the padding on the 2nd line, not the first
/// line. When "extra2" is -1 calculate the padding.
/// Returns the number of columns of overlap with buffer text, excluding the
/// extra padding on the ledge.
#[inline]
fn sms_marker_overlap_impl(wp: WinHandle, extra2: c_int) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let extra2 = if extra2 == -1 {
        rs_win_col_off(wp) - rs_win_col_off2(wp)
    } else {
        extra2
    };

    // There is no marker overlap when in showbreak mode, thus no need to
    // account for it. See wlv_put_linebuf().
    unsafe {
        let sbr = get_showbreak_value_impl(wp);
        if !sbr.is_null() && *sbr != 0 {
            return 0;
        }

        // Overlap when 'list' and 'listchars' "precedes" are set is 1.
        let p_list = nvim_win_get_p_list(wp) != 0;
        let prec = nvim_win_get_lcs_prec(wp);
        if p_list && prec != 0 {
            return 1;
        }
    }

    // The marker is "<<<" which takes 3 columns, so overlap is 3 - extra2
    // but only when extra2 <= 3
    if extra2 > 3 { 0 } else { 3 - extra2 }
}

/// Whether cursorline is drawn in a special way.
///
/// If true, both old and new cursorline will need to be redrawn when moving cursor within windows.
#[inline]
fn win_cursorline_standout_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let cul = nvim_win_get_p_cul(wp) != 0;
        let is_curwin = nvim_win_is_curwin(wp) != 0;
        let cole = nvim_win_get_p_cole(wp);
        let conceal_cursor = rs_conceal_cursor_line(wp) != 0;

        cul || (is_curwin && cole > 0 && !conceal_cursor)
    }
}

/// Return the effective 'scrolloff' value for the current window.
///
/// Uses the global value when window value is negative.
/// Disallows scrolloff in terminal-mode for terminal buffers.
#[inline]
fn get_scrolloff_value_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let state = nvim_get_State();
        let is_terminal_buf = nvim_win_buf_is_terminal(wp) != 0;

        // Disallow scrolloff in terminal-mode for terminal buffers
        if (state & MODE_TERMINAL) != 0 && is_terminal_buf {
            return 0;
        }

        let w_so = nvim_win_get_p_so(wp);
        if w_so < 0 {
            nvim_get_p_so() as c_int
        } else {
            w_so as c_int
        }
    }
}

/// Return the effective 'sidescrolloff' value for the current window.
///
/// Uses the global value when window value is negative.
#[inline]
fn get_sidescrolloff_value_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let w_siso = nvim_win_get_p_siso(wp);
        if w_siso < 0 {
            nvim_get_p_siso() as c_int
        } else {
            w_siso as c_int
        }
    }
}

/// Return the number of lines used by the global statusline.
#[inline]
fn global_stl_height_impl() -> c_int {
    unsafe {
        if nvim_get_p_ls() == 3 {
            STATUS_HEIGHT
        } else {
            0
        }
    }
}

/// Return the number of lines used by default by the window bar.
#[inline]
fn global_winbar_height_impl() -> c_int {
    unsafe { c_int::from(nvim_get_p_wbr_empty() == 0) }
}

// ============================================================================
// FFI Exports
// ============================================================================

/// Compute the width of the foldcolumn.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_compute_foldcolumn(wp: WinHandle, col: c_int) -> c_int {
    compute_foldcolumn_impl(wp, col)
}

/// Return the width of the 'number' and 'relativenumber' column.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_number_width(wp: WinHandle) -> c_int {
    number_width_impl(wp)
}

/// Return true if the cursor line may be concealed.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_conceal_cursor_line(wp: WinHandle) -> c_int {
    c_int::from(conceal_cursor_line_impl(wp))
}

/// Get the number of cells taken up on the screen at given virtual column.
///
/// # Safety
/// The `buf` parameter must be a valid `buf_T*` pointer or null.
/// The `cur` parameter must be a valid pointer to a character.
#[no_mangle]
pub extern "C" fn rs_charsize_nowrap(
    buf: BufHandle,
    cur: *const c_char,
    use_tabstop: c_int,
    vcol: c_int,
    cur_char: i32,
) -> c_int {
    charsize_nowrap_impl(buf, cur, use_tabstop != 0, vcol, cur_char)
}

/// Check if there may be filler lines anywhere in window "wp".
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_win_may_fill(wp: WinHandle) -> c_int {
    c_int::from(win_may_fill_impl(wp))
}

/// Return the offset for the window's first column.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_win_col_off(wp: WinHandle) -> c_int {
    win_col_off_impl(wp)
}

/// Return the offset for wrapped lines (second screen line onwards).
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_win_col_off2(wp: WinHandle) -> c_int {
    win_col_off2_impl(wp)
}

/// Check if vcol is in the rightmost column of window.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_in_win_border(wp: WinHandle, vcol: c_int) -> c_int {
    c_int::from(in_win_border_impl(wp, vcol))
}

/// Get the 'showbreak' value for a window.
///
/// Returns window-local showbreak if set, otherwise global showbreak.
/// Returns empty string if window showbreak is "NONE".
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_get_showbreak_value(wp: WinHandle) -> *const c_char {
    get_showbreak_value_impl(wp)
}

/// Calculate the smoothscroll marker overlap.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_sms_marker_overlap(wp: WinHandle, extra2: c_int) -> c_int {
    sms_marker_overlap_impl(wp, extra2)
}

/// Whether cursorline is drawn in a special way.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_win_cursorline_standout(wp: WinHandle) -> c_int {
    c_int::from(win_cursorline_standout_impl(wp))
}

/// Return the effective 'scrolloff' value for the current window.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_get_scrolloff_value(wp: WinHandle) -> c_int {
    get_scrolloff_value_impl(wp)
}

/// Return the effective 'sidescrolloff' value for the current window.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_get_sidescrolloff_value(wp: WinHandle) -> c_int {
    get_sidescrolloff_value_impl(wp)
}

/// Return the number of lines used by the global statusline.
#[no_mangle]
pub extern "C" fn rs_global_stl_height() -> c_int {
    global_stl_height_impl()
}

/// Return the number of lines used by default by the window bar.
#[no_mangle]
pub extern "C" fn rs_global_winbar_height() -> c_int {
    global_winbar_height_impl()
}

#[cfg(test)]
mod tests {
    // Tests require FFI stubs which aren't available in pure Rust testing.
    // Integration testing is done via the full Neovim build.
}
