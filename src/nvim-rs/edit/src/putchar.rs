//! `edit_putchar` / `edit_unputchar` -- put a character directly on screen.
//!
//! Used while handling CTRL-K, CTRL-V, CTRL-T, etc. in Insert mode.
//! The C statics (`pc_status`, `pc_schar`, `pc_attr`, `pc_row`, `pc_col`) are
//! moved here as Rust statics; the C accessor `nvim_set_pc_status_unset`
//! calls back into Rust to reset `pc_status`.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_uint};
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};

// ============================================================================
// pc_status constants
// ============================================================================

/// Nothing was put on screen.
const PC_STATUS_UNSET: i32 = 0;
/// Right half of double-wide char.
const PC_STATUS_RIGHT: i32 = 1;
/// Left half of double-wide char.
const PC_STATUS_LEFT: i32 = 2;
/// `pc_schar` was filled.
const PC_STATUS_SET: i32 = 3;

// ============================================================================
// Rust-owned statics (previously C statics in edit.c)
// ============================================================================

static PC_STATUS: AtomicI32 = AtomicI32::new(PC_STATUS_UNSET);
/// Saved `schar_T` (`uint32_t`).
static PC_SCHAR: AtomicU32 = AtomicU32::new(0);
static PC_ATTR: AtomicI32 = AtomicI32::new(0);
static PC_ROW: AtomicI32 = AtomicI32::new(0);
static PC_COL: AtomicI32 = AtomicI32::new(0);

// ============================================================================
// C accessor functions
// ============================================================================

extern "C" {
    /// True when curwin->w_grid_alloc.chars is non-NULL (`edit_shim.c`).
    fn nvim_curwin_grid_alloc_has_chars() -> bool;
    /// True when `default_grid.chars` is non-NULL (`drawscreen_shim.c`).
    fn nvim_default_grid_has_chars() -> bool;
    /// `update_topline(curwin)` (`eval_shim.c`).
    fn nvim_update_topline_curwin();
    /// `validate_cursor(curwin)` (`normal_shim.c`).
    fn nvim_validate_cursor_curwin_wrapper();
    /// `HL_ATTR(HLF_8)` (`edit_shim.c`).
    fn nvim_hl_attr_8() -> c_int;
    /// curwin->w_wrow (`normal_shim.c`).
    fn nvim_curwin_get_wrow() -> c_int;
    /// curwin->w_p_rl (`edit_shim.c`).
    fn nvim_curwin_get_p_rl() -> c_int;
    /// curwin->w_view_width (`ex_cmds_shim.c`).
    fn nvim_curwin_get_view_width() -> c_int;
    /// curwin->w_wcol (`normal_shim.c`).
    fn nvim_curwin_get_wcol() -> c_int;
    /// Set curwin->w_wcol (`normal_shim.c`).
    fn nvim_curwin_set_wcol(val: c_int);
    /// grid_line_start(&curwin->w_grid, row) (`edit_shim.c`).
    fn nvim_edit_grid_line_start(row: c_int);
    /// `grid_line_getchar(col`, `attr_out`) -> `uint32_t` (`edit_shim.c`).
    fn nvim_edit_grid_line_getchar(col: c_int, attr_out: *mut c_int) -> c_uint;
    /// `grid_line_put_schar(col`, schar, attr) (`edit_shim.c`).
    fn nvim_edit_grid_line_put_schar(col: c_int, schar: c_uint, attr: c_int);
    /// `grid_line_puts(col`, buf, len, attr) (`edit_shim.c`).
    fn nvim_edit_grid_line_puts(col: c_int, buf: *const c_char, len: c_int, attr: c_int) -> c_int;
    /// `grid_line_flush()` (`edit_shim.c`).
    fn nvim_edit_grid_line_flush();
    /// `schar_from_ascii`(' ') as `uint32_t` (`edit_shim.c`).
    fn nvim_schar_space() -> c_uint;
    fn utf_char2bytes(c: c_int, buf: *mut u8) -> c_int;
    /// redrawWinline(curwin, curwin->w_cursor.lnum) (`edit_shim.c`).
    fn nvim_redrawwinline_cursor();
}

// ============================================================================
// edit_putchar
// ============================================================================

/// Put a character directly onto the screen. Not stored in a buffer.
/// Used while handling CTRL-K, CTRL-V, etc. in Insert mode.
///
/// # Safety
/// Calls C functions that access global `curwin/default_grid` state.
unsafe fn edit_putchar_impl(c: c_int, highlight: bool) {
    if !nvim_curwin_grid_alloc_has_chars() && !nvim_default_grid_has_chars() {
        return;
    }

    let attr: c_int = if highlight { nvim_hl_attr_8() } else { 0 };

    nvim_update_topline_curwin(); // just in case w_topline isn't valid
    nvim_validate_cursor_curwin_wrapper();

    let row = nvim_curwin_get_wrow();
    PC_ROW.store(row, Ordering::Relaxed);
    PC_STATUS.store(PC_STATUS_UNSET, Ordering::Relaxed);

    nvim_edit_grid_line_start(row);

    if nvim_curwin_get_p_rl() != 0 {
        let col = nvim_curwin_get_view_width() - 1 - nvim_curwin_get_wcol();
        PC_COL.store(col, Ordering::Relaxed);
        if nvim_edit_grid_line_getchar(col, std::ptr::null_mut()) == 0 {
            // NUL means right half of double-wide char
            let space = nvim_schar_space();
            nvim_edit_grid_line_put_schar(col - 1, space, attr);
            let wcol = nvim_curwin_get_wcol();
            nvim_curwin_set_wcol(wcol - 1);
            PC_STATUS.store(PC_STATUS_RIGHT, Ordering::Relaxed);
        }
    } else {
        let col = nvim_curwin_get_wcol();
        PC_COL.store(col, Ordering::Relaxed);
        if nvim_edit_grid_line_getchar(col + 1, std::ptr::null_mut()) == 0 {
            // col is the left half of a double-width char
            PC_STATUS.store(PC_STATUS_LEFT, Ordering::Relaxed);
        }
    }

    // Save the character to be able to put it back
    if PC_STATUS.load(Ordering::Relaxed) == PC_STATUS_UNSET {
        let mut saved_attr: c_int = 0;
        let col = PC_COL.load(Ordering::Relaxed);
        let schar = nvim_edit_grid_line_getchar(col, &raw mut saved_attr);
        PC_SCHAR.store(schar, Ordering::Relaxed);
        PC_ATTR.store(saved_attr, Ordering::Relaxed);
        PC_STATUS.store(PC_STATUS_SET, Ordering::Relaxed);
    }

    // MB_MAXCHAR + 1
    let mut buf = [0u8; 22];
    let col = PC_COL.load(Ordering::Relaxed);
    let len = utf_char2bytes(c, buf.as_mut_ptr());
    nvim_edit_grid_line_puts(col, buf.as_ptr().cast(), len, attr);
    nvim_edit_grid_line_flush();
}

/// # Safety
/// Calls C functions that access global state.
#[unsafe(export_name = "edit_putchar")]
pub unsafe extern "C" fn rs_edit_putchar(c: c_int, highlight: bool) {
    edit_putchar_impl(c, highlight);
}

// ============================================================================
// edit_unputchar
// ============================================================================

/// Undo the previous `edit_putchar()`.
///
/// # Safety
/// Calls C functions that access global curwin state.
unsafe fn edit_unputchar_impl() {
    let status = PC_STATUS.load(Ordering::Relaxed);
    if status == PC_STATUS_UNSET {
        return;
    }

    if status == PC_STATUS_RIGHT {
        let wcol = nvim_curwin_get_wcol();
        nvim_curwin_set_wcol(wcol + 1);
    }

    if status == PC_STATUS_RIGHT || status == PC_STATUS_LEFT {
        nvim_redrawwinline_cursor();
    } else {
        // PC_STATUS_SET: restore the saved character
        let row = PC_ROW.load(Ordering::Relaxed);
        let col = PC_COL.load(Ordering::Relaxed);
        let schar = PC_SCHAR.load(Ordering::Relaxed);
        let attr = PC_ATTR.load(Ordering::Relaxed);
        nvim_edit_grid_line_start(row);
        nvim_edit_grid_line_put_schar(col, schar, attr);
        nvim_edit_grid_line_flush();
    }
}

/// # Safety
/// Calls C functions that access global state.
#[unsafe(export_name = "edit_unputchar")]
pub unsafe extern "C" fn rs_edit_unputchar() {
    edit_unputchar_impl();
}

// ============================================================================
// C accessor: nvim_set_pc_status_unset (called from C and Rust)
// ============================================================================

/// Reset `pc_status` to `PC_STATUS_UNSET`.
///
/// Called from C (edit.c accessor) and Rust (editing.rs digraph code).
/// The C symbol `nvim_set_pc_status_unset` now updates the Rust static.
#[unsafe(export_name = "nvim_set_pc_status_unset")]
pub extern "C" fn rs_set_pc_status_unset() {
    PC_STATUS.store(PC_STATUS_UNSET, Ordering::Relaxed);
}

// ============================================================================
// display_dollar
// ============================================================================

// Additional C accessors for display_dollar.
extern "C" {
    /// `redrawing()` -> bool (`window_shim.c`).
    fn nvim_redrawing() -> c_int;
    /// curwin->w_cursor.col (`cursor_shim.c`).
    fn nvim_curwin_get_cursor_col() -> c_int;
    /// Set curwin->w_cursor.col (`cursor_shim.c`).
    fn nvim_curwin_set_cursor_col(col: c_int);
    /// `utf_head_off(cursor_line`, `cursor_line+col`) (`edit_shim.c`).
    fn nvim_edit_utf_head_off_cursor_col(col: c_int) -> c_int;
    /// `curs_columns(curwin`, false) (`edit_shim.c`).
    fn nvim_curs_columns_curwin_no_scroll();
    /// curwin->w_virtcol (edit.c).
    fn nvim_curwin_get_w_virtcol() -> c_int;
    /// `dollar_vcol` setter (edit.c).
    fn nvim_set_dollar_vcol(val: c_int);
}

/// Display a '$' at the end of the changed text when `cpo+=dollar`.
/// Only works when cursor is in the line that changes.
///
/// # Safety
/// Accesses global curwin state via C accessor functions.
unsafe fn display_dollar_impl(col_arg: c_int) {
    let col = col_arg.max(0);

    if nvim_redrawing() == 0 {
        return;
    }

    let save_col = nvim_curwin_get_cursor_col();
    nvim_curwin_set_cursor_col(col);

    // If on the last byte of a multi-byte char, move to the first byte.
    let head_off = nvim_edit_utf_head_off_cursor_col(col);
    let adjusted_col = col - head_off;
    nvim_curwin_set_cursor_col(adjusted_col);

    nvim_curs_columns_curwin_no_scroll(); // Recompute w_wrow and w_wcol

    if nvim_curwin_get_wcol() < nvim_curwin_get_view_width() {
        edit_putchar_impl(c_int::from(b'$'), false);
        nvim_set_dollar_vcol(nvim_curwin_get_w_virtcol());
    }

    nvim_curwin_set_cursor_col(save_col);
}

/// # Safety
/// Accesses global state.
#[unsafe(export_name = "display_dollar")]
pub unsafe extern "C" fn rs_display_dollar(col_arg: c_int) {
    display_dollar_impl(col_arg);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pc_status_constants() {
        assert_eq!(PC_STATUS_UNSET, 0);
        assert_eq!(PC_STATUS_RIGHT, 1);
        assert_eq!(PC_STATUS_LEFT, 2);
        assert_eq!(PC_STATUS_SET, 3);
    }

    #[test]
    fn test_pc_status_atomic_initial() {
        // PC_STATUS starts as UNSET
        assert_eq!(PC_STATUS.load(Ordering::Relaxed), PC_STATUS_UNSET);
    }
}
