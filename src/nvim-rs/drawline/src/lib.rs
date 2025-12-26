//! Line drawing functions for Neovim
//!
//! This crate provides Rust implementations of line drawing functions
//! from `src/nvim/drawline.c`, focusing on column rendering and helpers.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::c_int;

use nvim_window::WinHandle;

/// schar_T is stored as a u32.
type ScharT = u32;

/// Line number type.
type LinenrT = i64;

/// Column number type.
type ColnrT = i32;

/// Fold info structure (matching C foldinfo_T).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldInfo {
    /// Line number where fold starts.
    pub fi_lnum: LinenrT,
    /// Level of the fold (0 = no fold).
    pub fi_level: c_int,
    /// Lowest fold level that starts in the same line.
    pub fi_low_level: c_int,
    /// Number of lines the fold spans (0 if not closed).
    pub fi_lines: LinenrT,
}

// Highlight group constants
pub const HLF_FC: c_int = 35; // Fold column
pub const HLF_CLF: c_int = 43; // Cursor line fold column

// C accessor functions
extern "C" {
    fn nvim_win_get_p_wrap(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_list(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_cuc(wp: WinHandle) -> c_int;
    fn nvim_win_get_wrap_flags(wp: WinHandle) -> c_int;
    fn nvim_win_get_lcs_ext(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_foldclosed(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_foldopen(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_foldsep(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_foldinner(wp: WinHandle) -> ScharT;
    fn nvim_win_get_view_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_virtcol(wp: WinHandle) -> ColnrT;

    // Grid functions for schar operations
    fn rs_schar_from_char(c: c_int) -> ScharT;

    // Display width functions
    fn rs_win_col_off(wp: WinHandle) -> c_int;
    fn rs_win_col_off2(wp: WinHandle) -> c_int;
}

/// Flag for insecure wrap option.
const K_OPT_FLAG_INSECURE: c_int = 0x04;

/// Get the 'listchars' "extends" character to use for "wp", or 0 if it
/// shouldn't be used.
///
/// This is the Rust equivalent of `get_lcs_ext()` in drawline.c.
fn get_lcs_ext_impl(wp: WinHandle) -> ScharT {
    unsafe {
        // Line never continues beyond the right of the screen with 'wrap'.
        if nvim_win_get_p_wrap(wp) != 0 {
            return 0;
        }
        // If 'nowrap' was set from a modeline, forcibly use '>'.
        if nvim_win_get_wrap_flags(wp) & K_OPT_FLAG_INSECURE != 0 {
            return rs_schar_from_char(c_int::from(b'>'));
        }
        if nvim_win_get_p_list(wp) != 0 {
            nvim_win_get_lcs_ext(wp)
        } else {
            0
        }
    }
}

/// Compute the margins for 'cursorlineopt' "screenline".
///
/// Used when 'cursorlineopt' contains "screenline": compute the margins between
/// which the highlighting is used.
///
/// This is the Rust equivalent of `margin_columns_win()` in drawline.c.
/// Note: The C version uses static caching which we replicate here.
#[allow(clippy::cast_possible_truncation)]
fn margin_columns_win_impl(wp: WinHandle) -> (c_int, c_int) {
    // NOTE: The C version has static caching. For thread safety in Rust,
    // we compute fresh each time. The caller can cache if needed.
    unsafe {
        let cur_col_off = rs_win_col_off(wp);
        let width1 = nvim_win_get_view_width(wp) - cur_col_off;
        let width2 = width1 + rs_win_col_off2(wp);
        let virtcol = nvim_win_get_virtcol(wp);

        let right_col = if virtcol >= width1 && width2 > 0 {
            width1 + ((virtcol - width1) / width2 + 1) as c_int * width2
        } else {
            width1
        };

        let left_col = if virtcol >= width1 && width2 > 0 {
            ((virtcol - width1) / width2) as c_int * width2 + width1
        } else {
            0
        };

        (left_col, right_col)
    }
}

/// Fill a fold column buffer with fold symbols.
///
/// This computes the fold column characters for a given line and fold info.
/// Returns the symbols as an array and the number of symbols.
///
/// @param level      Current fold level
/// @param closed     Whether the fold is closed
/// @param lnum       Current line number
/// @param fi_lnum    Line number where fold starts
/// @param fi_low_level Lowest fold level starting on same line
/// @param fdc        Fold column width
///
/// This is a pure computation extracted from `fill_foldcolumn()` in drawline.c.
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
fn compute_foldcolumn_symbols(
    wp: WinHandle,
    level: c_int,
    closed: bool,
    lnum: LinenrT,
    fi_lnum: LinenrT,
    fi_low_level: c_int,
    fdc: c_int,
) -> Vec<(ScharT, c_int)> {
    // If the column is too narrow, we start at the lowest level that
    // fits and use numbers to indicate the depth.
    let first_level = (level - fdc - c_int::from(closed) + 1).max(1);
    let closedcol = fdc.min(level);

    let capacity = if fdc > 0 { fdc as usize } else { 0 };
    let mut result = Vec::with_capacity(capacity);

    unsafe {
        for i in 0..fdc {
            let symbol = if i >= level {
                rs_schar_from_char(c_int::from(b' '))
            } else if i == closedcol - 1 && closed {
                nvim_win_get_fcs_foldclosed(wp)
            } else if fi_lnum == lnum && first_level + i >= fi_low_level {
                nvim_win_get_fcs_foldopen(wp)
            } else if first_level == 1 {
                nvim_win_get_fcs_foldsep(wp)
            } else {
                let foldinner = nvim_win_get_fcs_foldinner(wp);
                if foldinner != 0 {
                    foldinner
                } else if first_level + i <= 9 {
                    // Safe: first_level + i is guaranteed to be 1-9 here
                    rs_schar_from_char(c_int::from(b'0') + first_level + i)
                } else {
                    rs_schar_from_char(c_int::from(b'>'))
                }
            };

            // vcol: -1 = past fold level, -2 = closed fold, -3 = within fold
            let vcol = if i >= level {
                -1
            } else if i == closedcol - 1 && closed {
                -2
            } else {
                -3
            };

            result.push((symbol, vcol));
        }
    }

    result
}

/// Get the rightmost virtual column that needs to be drawn.
///
/// This determines the rightmost column for colorcolumn or cursorcolumn
/// highlighting. Returns 0 if neither feature is active.
///
/// @param wp          Window handle
/// @param color_cols  Pointer to -1 terminated array of colorcolumn positions, or null
/// @param color_cols_len  Number of elements in color_cols array (including terminator)
///
/// This is the Rust equivalent of `get_rightmost_vcol()` in drawline.c.
fn get_rightmost_vcol_impl(wp: WinHandle, color_cols: *const c_int) -> c_int {
    let mut ret = 0;

    unsafe {
        // Include cursor column if 'cursorcolumn' is set
        if nvim_win_get_p_cuc(wp) != 0 {
            ret = nvim_win_get_virtcol(wp);
        }

        // Find rightmost colorcolumn
        if !color_cols.is_null() {
            let mut i = 0;
            loop {
                let col = *color_cols.add(i);
                if col < 0 {
                    break;
                }
                if col > ret {
                    ret = col;
                }
                i += 1;
            }
        }
    }

    ret
}

// FFI exports

/// Get the 'listchars' "extends" character.
#[no_mangle]
pub extern "C" fn rs_get_lcs_ext(wp: WinHandle) -> ScharT {
    get_lcs_ext_impl(wp)
}

/// Get the rightmost virtual column that needs drawing.
///
/// # Safety
/// `color_cols` must be a valid pointer to a -1 terminated array, or null.
#[no_mangle]
pub extern "C" fn rs_get_rightmost_vcol(wp: WinHandle, color_cols: *const c_int) -> c_int {
    get_rightmost_vcol_impl(wp, color_cols)
}

/// Compute cursorline margins.
///
/// # Safety
/// `left_col` and `right_col` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_margin_columns_win(
    wp: WinHandle,
    left_col: *mut c_int,
    right_col: *mut c_int,
) {
    let (left, right) = margin_columns_win_impl(wp);
    if !left_col.is_null() {
        *left_col = left;
    }
    if !right_col.is_null() {
        *right_col = right;
    }
}

/// Fill fold column output buffer with symbols.
///
/// # Safety
/// `out_buffer` and `out_vcol` must be valid arrays of at least `fdc` elements,
/// or null (in which case this returns early).
#[no_mangle]
pub unsafe extern "C" fn rs_fill_foldcolumn_buffer(
    wp: WinHandle,
    foldinfo: FoldInfo,
    lnum: LinenrT,
    fdc: c_int,
    out_buffer: *mut ScharT,
    out_vcol: *mut ColnrT,
) {
    if out_buffer.is_null() || out_vcol.is_null() || fdc <= 0 {
        return;
    }

    let closed = foldinfo.fi_level != 0 && foldinfo.fi_lines > 0;
    let symbols = compute_foldcolumn_symbols(
        wp,
        foldinfo.fi_level,
        closed,
        lnum,
        foldinfo.fi_lnum,
        foldinfo.fi_low_level,
        fdc,
    );

    for (i, (symbol, vcol)) in symbols.into_iter().enumerate() {
        *out_buffer.add(i) = symbol;
        *out_vcol.add(i) = vcol;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foldinfo_layout() {
        // Verify FoldInfo struct has expected size
        assert!(std::mem::size_of::<FoldInfo>() > 0);
    }
}
