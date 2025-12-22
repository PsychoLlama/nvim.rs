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
}

// Mode constants (matching Neovim's state.h)
const MODE_VISUAL: c_int = 0x02;
const MODE_INSERT: c_int = 0x10;
const MODE_NORMAL: c_int = 0x01;
const MODE_CMDLINE: c_int = 0x04;

// Sign column constants (matching Neovim's optionstr.c)
const SCL_NUM: c_int = -1;

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

#[cfg(test)]
mod tests {
    // Tests require FFI stubs which aren't available in pure Rust testing.
    // Integration testing is done via the full Neovim build.
}
