//! Cursor movement functions migrated from edit.c
//!
//! These handle single-character left/right movement, multi-line
//! up/down movement (with fold awareness), and beginning-of-line
//! positioning.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

/// Column number type (matches `colnr_T` in Neovim).
type ColnrT = i32;

/// Line number type (matches `linenr_T` in Neovim).
type LinenrT = i32;

/// Opaque handle to a window (`win_T *`).
type WinHandle = *mut c_void;

// ============================================================================
// C accessor functions
// ============================================================================

extern "C" {
    // Cursor position (curwin)
    fn nvim_curwin_get_cursor_col() -> ColnrT;
    fn nvim_curwin_set_cursor_col(col: ColnrT);
    fn nvim_curwin_get_cursor_lnum() -> LinenrT;

    // Cursor coladd (curwin)
    fn nvim_curwin_get_cursor_coladd() -> ColnrT;
    fn nvim_set_curwin_cursor_coladd(val: ColnrT);

    // w_curswant / w_set_curswant (curwin)
    fn nvim_curwin_get_w_curswant() -> ColnrT;
    fn nvim_curwin_set_w_set_curswant(val: bool);

    // Window-parameterized cursor access
    fn nvim_edit_win_get_cursor_lnum(wp: WinHandle) -> LinenrT;
    fn nvim_edit_win_set_cursor_lnum(wp: WinHandle, lnum: LinenrT);
    fn nvim_win_get_buf_line_count(wp: WinHandle) -> LinenrT;

    // Current window handle
    fn nvim_get_curwin() -> WinHandle;

    // Movement helpers
    fn nvim_coladvance(col: ColnrT);
    fn adjust_skipcol();
    fn getviscol() -> c_int;
    fn virtual_active(wp: WinHandle) -> bool;
    fn nvim_edit_get_ve_flags() -> c_int;

    // Character operations
    fn nvim_get_cursor_pos_ptr() -> *const c_char;
    fn get_cursor_line_ptr() -> *mut c_char;
    fn vim_isprintc(c: c_int) -> bool;
    fn ptr2cells(ptr: *const c_char) -> c_int;
    fn utf_ptr2char(ptr: *const u8) -> c_int;
    fn utfc_ptr2len(p: *const u8) -> c_int;
    fn mb_adjust_cursor();

    // Fold/conceal
    fn nvim_hasFolding(
        wp: WinHandle,
        lnum: LinenrT,
        firstp: *mut LinenrT,
        lastp: *mut LinenrT,
    ) -> c_int;
    fn nvim_edit_hasFoldingWin(
        wp: WinHandle,
        lnum: LinenrT,
        firstp: *mut LinenrT,
        lastp: *mut LinenrT,
    ) -> c_int;
    fn nvim_win_lines_concealed(wp: WinHandle) -> c_int;
    fn nvim_decor_conceal_line(wp: WinHandle, row: c_int, check_cursor: c_int) -> c_int;

    // State / option globals
    fn nvim_get_State() -> c_int;
    fn nvim_get_p_sol() -> c_int;
    fn nvim_edit_get_fdo_flags() -> c_int;

    // Topline
    fn nvim_excmds_update_topline_curwin();

    // ASCII helpers (Rust-implemented, callable via FFI)
    fn rs_ascii_iswhite(c: c_int) -> c_int;
}

// ============================================================================
// Constants (verified against C headers with `_Static_assert` in `edit.c`)
// ============================================================================

/// `BL_WHITE` from `edit.h` — cursor on first non-white in the line
const BL_WHITE: c_int = 1;

/// `BL_SOL` from `edit.h` — use 'sol' option
const BL_SOL: c_int = 2;

/// `BL_FIX` from `edit.h` — don't leave cursor on a NUL
const BL_FIX: c_int = 4;

/// `OK` from `vim_defs.h`
const OK: c_int = 1;

/// `FAIL` from `vim_defs.h`
const FAIL: c_int = 0;

/// `MODE_INSERT` from `state_defs.h`
const MODE_INSERT: c_int = 0x10;

/// `kOptFdoFlagAll` from generated `option_vars`
const K_OPT_FDO_FLAG_ALL: c_int = 0x01;

/// `kOptVeFlagOnemore` from generated `option_vars`
const K_OPT_VE_FLAG_ONEMORE: c_int = 0x08;

/// NUL byte
const NUL: c_char = 0;

/// TAB character (`'\011'` = 9)
const TAB: c_char = 9;

// ============================================================================
// beginline
// ============================================================================

/// Move cursor to start of line.
///
/// - `BL_WHITE`: move to first non-white character
/// - `BL_SOL`: respect 'startofline' option
/// - `BL_FIX`: don't leave cursor on a NUL
unsafe fn beginline_impl(flags: c_int) {
    if (flags & BL_SOL) != 0 && nvim_get_p_sol() == 0 {
        nvim_coladvance(nvim_curwin_get_w_curswant());
    } else {
        nvim_curwin_set_cursor_col(0);
        nvim_set_curwin_cursor_coladd(0);

        if (flags & (BL_WHITE | BL_SOL)) != 0 {
            let mut ptr = get_cursor_line_ptr();
            while rs_ascii_iswhite(c_int::from(*ptr as u8)) != 0
                && !((flags & BL_FIX) != 0 && *ptr.add(1) == NUL)
            {
                nvim_curwin_set_cursor_col(nvim_curwin_get_cursor_col() + 1);
                ptr = ptr.add(1);
            }
        }
        nvim_curwin_set_w_set_curswant(true);
    }
    adjust_skipcol();
}

#[unsafe(export_name = "beginline")]
pub unsafe extern "C" fn rs_beginline(flags: c_int) {
    beginline_impl(flags);
}

// ============================================================================
// oneright
// ============================================================================

/// Move one character to the right.
///
/// Handles virtual editing mode and multi-byte characters.
/// Returns OK on success, FAIL at line boundary.
unsafe fn oneright_impl() -> c_int {
    if virtual_active(nvim_get_curwin()) {
        let prev_col = nvim_curwin_get_cursor_col();
        let prev_coladd = nvim_curwin_get_cursor_coladd();

        // Adjust for multi-wide char (excluding TAB)
        let ptr = nvim_get_cursor_pos_ptr();
        let viscol = getviscol();
        let advance = if *ptr != TAB && vim_isprintc(utf_ptr2char(ptr.cast::<u8>())) {
            ptr2cells(ptr)
        } else {
            1
        };
        nvim_coladvance(viscol + advance);
        nvim_curwin_set_w_set_curswant(true);

        // Return OK if the cursor moved, FAIL otherwise
        if prev_col != nvim_curwin_get_cursor_col()
            || prev_coladd != nvim_curwin_get_cursor_coladd()
        {
            return OK;
        }
        return FAIL;
    }

    let ptr = nvim_get_cursor_pos_ptr();
    if *ptr == NUL {
        return FAIL; // already at the very end
    }

    let l = utfc_ptr2len(ptr.cast::<u8>());

    // move "l" bytes right, but don't end up on the NUL, unless 'virtualedit'
    // contains "onemore".
    if *ptr.add(l as usize) == NUL && (nvim_edit_get_ve_flags() & K_OPT_VE_FLAG_ONEMORE) == 0 {
        return FAIL;
    }
    nvim_curwin_set_cursor_col(nvim_curwin_get_cursor_col() + l as ColnrT);

    nvim_curwin_set_w_set_curswant(true);
    adjust_skipcol();
    OK
}

#[must_use]
#[unsafe(export_name = "oneright")]
pub unsafe extern "C" fn rs_oneright() -> c_int {
    oneright_impl()
}

// ============================================================================
// oneleft
// ============================================================================

/// Move one character to the left.
///
/// Handles virtual editing mode (accounting for 'showbreak'),
/// multi-byte characters, and adjusts cursor to first byte.
/// Returns OK on success, FAIL at line boundary.
unsafe fn oneleft_impl() -> c_int {
    if virtual_active(nvim_get_curwin()) {
        let v = getviscol();

        if v == 0 {
            return FAIL;
        }

        // We might get stuck on 'showbreak', skip over it.
        let mut width: ColnrT = 1;
        loop {
            nvim_coladvance(v - width);
            // getviscol() is slow, skip it when 'showbreak' is empty,
            // 'breakindent' is not set and there are no multi-byte characters
            if getviscol() < v {
                break;
            }
            width += 1;
        }

        if nvim_curwin_get_cursor_coladd() == 1 {
            // Adjust for multi-wide char (not a TAB)
            let ptr = nvim_get_cursor_pos_ptr();
            if *ptr != TAB && vim_isprintc(utf_ptr2char(ptr.cast::<u8>())) && ptr2cells(ptr) > 1 {
                nvim_set_curwin_cursor_coladd(0);
            }
        }

        nvim_curwin_set_w_set_curswant(true);
        adjust_skipcol();
        return OK;
    }

    if nvim_curwin_get_cursor_col() == 0 {
        return FAIL;
    }

    nvim_curwin_set_w_set_curswant(true);
    nvim_curwin_set_cursor_col(nvim_curwin_get_cursor_col() - 1);

    // if the character on the left of the current cursor is a multi-byte
    // character, move to its first byte
    mb_adjust_cursor();
    adjust_skipcol();
    OK
}

#[must_use]
#[unsafe(export_name = "oneleft")]
pub unsafe extern "C" fn rs_oneleft() -> c_int {
    oneleft_impl()
}

// ============================================================================
// cursor_up_inner
// ============================================================================

/// Move the cursor up "n" lines in window "wp".
///
/// Takes care of closed folds. Skips over concealed lines when
/// `skip_conceal` is true.
unsafe fn cursor_up_inner_impl(wp: WinHandle, mut n: LinenrT, skip_conceal: bool) {
    let mut lnum = nvim_edit_win_get_cursor_lnum(wp);

    if n >= lnum {
        lnum = 1;
    } else if nvim_win_lines_concealed(wp) != 0 {
        // Count each sequence of folded lines as one logical line.

        // go to the start of the current fold
        let mut first: LinenrT = 0;
        if nvim_hasFolding(wp, lnum, std::ptr::addr_of_mut!(first), ptr::null_mut()) != 0 {
            lnum = first;
        }

        loop {
            if n == 0 {
                break;
            }
            n -= 1;
            // move up one line
            lnum -= 1;
            if lnum <= 1 {
                break;
            }
            if skip_conceal {
                n += nvim_decor_conceal_line(wp, lnum - 1, 1);
            }
            // If we entered a fold, move to the beginning, unless in
            // Insert mode or when 'foldopen' contains "all": it will open
            // in a moment.
            if n > 0
                || !((nvim_get_State() & MODE_INSERT) != 0
                    || (nvim_edit_get_fdo_flags() & K_OPT_FDO_FLAG_ALL) != 0)
            {
                let mut fold_first: LinenrT = 0;
                if nvim_hasFolding(
                    wp,
                    lnum,
                    std::ptr::addr_of_mut!(fold_first),
                    ptr::null_mut(),
                ) != 0
                {
                    lnum = fold_first;
                }
            }
        }
        if lnum < 1 {
            lnum = 1;
        }
    } else {
        lnum -= n;
    }

    nvim_edit_win_set_cursor_lnum(wp, lnum);
}

/// Exported as the canonical C symbol, replacing the thin wrapper in `edit.c`.
#[unsafe(export_name = "cursor_up_inner")]
pub unsafe extern "C" fn rs_cursor_up_inner(wp: WinHandle, n: LinenrT, skip_conceal: bool) {
    cursor_up_inner_impl(wp, n, skip_conceal);
}

// ============================================================================
// cursor_up
// ============================================================================

/// Move cursor up "n" lines in curwin.
///
/// Tries to preserve the desired column position. Optionally
/// updates topline. Returns FAIL if already at line 1.
unsafe fn cursor_up_impl(n: LinenrT, upd_topline: bool) -> c_int {
    // This fails if the cursor is already in the first line.
    if n > 0 && nvim_curwin_get_cursor_lnum() <= 1 {
        return FAIL;
    }
    let curwin = nvim_get_curwin();
    cursor_up_inner_impl(curwin, n, false);

    // try to advance to the column we want to be at
    nvim_coladvance(nvim_curwin_get_w_curswant());

    if upd_topline {
        nvim_excmds_update_topline_curwin();
    }

    OK
}

#[must_use]
#[unsafe(export_name = "cursor_up")]
pub unsafe extern "C" fn rs_cursor_up(n: LinenrT, upd_topline: bool) -> c_int {
    cursor_up_impl(n, upd_topline)
}

// ============================================================================
// cursor_down_inner
// ============================================================================

/// Move the cursor down "n" lines in window "wp".
///
/// Takes care of closed folds. Skips over concealed lines when
/// `skip_conceal` is true.
unsafe fn cursor_down_inner_impl(wp: WinHandle, mut n: c_int, skip_conceal: bool) {
    let mut lnum = nvim_edit_win_get_cursor_lnum(wp);
    let line_count = nvim_win_get_buf_line_count(wp);

    if lnum + n as LinenrT >= line_count {
        lnum = line_count;
    } else if nvim_win_lines_concealed(wp) != 0 {
        // count each sequence of folded lines as one logical line
        loop {
            if n == 0 {
                break;
            }
            n -= 1;
            let mut last: LinenrT = 0;
            if nvim_edit_hasFoldingWin(wp, lnum, ptr::null_mut(), std::ptr::addr_of_mut!(last)) != 0
            {
                lnum = last + 1;
            } else {
                lnum += 1;
            }
            if lnum >= line_count {
                break;
            }
            if skip_conceal {
                n += nvim_decor_conceal_line(wp, lnum - 1, 1);
            }
        }
        if lnum > line_count {
            lnum = line_count;
        }
    } else {
        lnum += n as LinenrT;
    }

    nvim_edit_win_set_cursor_lnum(wp, lnum);
}

/// Exported as the canonical C symbol, replacing the thin wrapper in `edit.c`.
#[unsafe(export_name = "cursor_down_inner")]
pub unsafe extern "C" fn rs_cursor_down_inner(wp: WinHandle, n: c_int, skip_conceal: bool) {
    cursor_down_inner_impl(wp, n, skip_conceal);
}

// ============================================================================
// cursor_down
// ============================================================================

/// Move cursor down "n" lines in curwin.
///
/// Tries to preserve the desired column position. Optionally
/// updates topline. Returns FAIL if already at last (folded) line.
unsafe fn cursor_down_impl(n: c_int, upd_topline: bool) -> c_int {
    let curwin = nvim_get_curwin();
    let mut lnum = nvim_curwin_get_cursor_lnum();
    // This fails if the cursor is already in the last (folded) line.
    let mut fold_last: LinenrT = 0;
    if nvim_edit_hasFoldingWin(
        curwin,
        lnum,
        ptr::null_mut(),
        std::ptr::addr_of_mut!(fold_last),
    ) != 0
    {
        lnum = fold_last;
    }
    if n > 0 && lnum >= nvim_win_get_buf_line_count(curwin) {
        return FAIL;
    }
    cursor_down_inner_impl(curwin, n, false);

    // try to advance to the column we want to be at
    nvim_coladvance(nvim_curwin_get_w_curswant());

    if upd_topline {
        nvim_excmds_update_topline_curwin();
    }

    OK
}

#[must_use]
#[unsafe(export_name = "cursor_down")]
pub unsafe extern "C" fn rs_cursor_down(n: c_int, upd_topline: bool) -> c_int {
    cursor_down_impl(n, upd_topline)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(BL_WHITE, 1);
        assert_eq!(BL_SOL, 2);
        assert_eq!(BL_FIX, 4);
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(MODE_INSERT, 0x10);
        assert_eq!(K_OPT_FDO_FLAG_ALL, 0x01);
        assert_eq!(K_OPT_VE_FLAG_ONEMORE, 0x08);
        assert_eq!(TAB as u8, 9);
    }

    #[test]
    fn test_bl_flags_are_distinct_bits() {
        // Ensure the BL_* flags can be combined without collision
        assert_eq!(BL_WHITE & BL_SOL, 0);
        assert_eq!(BL_WHITE & BL_FIX, 0);
        assert_eq!(BL_SOL & BL_FIX, 0);
        assert_eq!(BL_WHITE | BL_SOL | BL_FIX, 7);
    }
}
