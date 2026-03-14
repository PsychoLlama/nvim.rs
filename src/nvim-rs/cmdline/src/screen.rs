//! Command line screen position calculations
//!
//! This module provides utilities for calculating cursor positions and
//! screen coordinates in command-line mode.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int};

// =============================================================================
// C Function Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Global screen state
    fn nvim_get_columns() -> c_int;
    fn nvim_get_rows() -> c_int;
    fn nvim_get_key_typed() -> c_int;
    fn nvim_get_cmdline_star() -> c_int;
    fn nvim_get_cmdline_row() -> c_int;
    fn nvim_cmdline_win_is_active() -> c_int;
    fn nvim_cmdline_win_width() -> c_int;
    fn nvim_cmdline_win_height() -> c_int;

    // Command line state
    fn nvim_get_ccline_cmdpos() -> c_int;
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn nvim_get_ccline_cmdspos() -> c_int;
    fn nvim_get_ccline_cmdindent() -> c_int;
    fn nvim_get_ccline_cmdfirstc() -> c_int;
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;

    // Character utilities
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_ptr2cells(p: *const c_char) -> c_int;
    fn ptr2cells(p: *const c_char) -> c_int;
}

// =============================================================================
// Constants
// =============================================================================

/// Maximum column value (used for overflow protection)
pub const MAXCOL: c_int = 0x7FFF_FFFF;

/// NUL character
const NUL: c_int = 0;

// =============================================================================
// Screen Dimensions
// =============================================================================

/// Get the screen width in columns.
///
/// # Safety
///
/// Calls C function to access global.
#[must_use]
pub unsafe fn get_columns() -> c_int {
    nvim_get_columns()
}

/// Get the screen height in rows.
///
/// # Safety
///
/// Calls C function to access global.
#[must_use]
pub unsafe fn get_rows() -> c_int {
    nvim_get_rows()
}

/// Check if a key was typed (vs from a mapping).
///
/// # Safety
///
/// Calls C function to access global.
#[must_use]
pub unsafe fn key_was_typed() -> bool {
    nvim_get_key_typed() != 0
}

/// Get the cmdline_star flag (password mode).
///
/// # Safety
///
/// Calls C function to access global.
#[must_use]
pub unsafe fn get_cmdline_star() -> c_int {
    nvim_get_cmdline_star()
}

/// Check if password mode is active.
///
/// # Safety
///
/// Calls C function to access global.
#[must_use]
pub unsafe fn is_password_mode() -> bool {
    nvim_get_cmdline_star() > 0
}

/// Get the row where the command line starts.
///
/// # Safety
///
/// Calls C function to access global.
#[must_use]
pub unsafe fn get_cmdline_row() -> c_int {
    nvim_get_cmdline_row()
}

// =============================================================================
// Command Line Window
// =============================================================================

/// Check if a command-line window is active.
///
/// # Safety
///
/// Calls C function to check state.
#[must_use]
pub unsafe fn cmdline_win_active() -> bool {
    nvim_cmdline_win_is_active() != 0
}

/// Get the command-line window width (if active).
///
/// # Safety
///
/// Calls C function to access window state.
#[must_use]
pub unsafe fn cmdline_win_width() -> c_int {
    nvim_cmdline_win_width()
}

/// Get the command-line window height (if active).
///
/// # Safety
///
/// Calls C function to access window state.
#[must_use]
pub unsafe fn cmdline_win_height() -> c_int {
    nvim_cmdline_win_height()
}

// =============================================================================
// Position Calculations
// =============================================================================

/// Compute the starting column for the cursor (includes indent and prompt).
///
/// # Safety
///
/// Calls C functions to access ccline state.
#[must_use]
pub unsafe fn cmd_startcol() -> c_int {
    let cmdindent = nvim_get_ccline_cmdindent();
    let cmdfirstc = nvim_get_ccline_cmdfirstc();
    cmdindent + c_int::from(cmdfirstc != NUL)
}

/// Get the character width at a byte position in the command buffer.
///
/// If in password mode, always returns 1 (showing '*').
///
/// # Safety
///
/// `idx` must be a valid byte offset in the command buffer.
/// Calls C functions to access buffer and calculate width.
#[must_use]
pub unsafe fn cmdline_charsize(idx: c_int) -> c_int {
    if nvim_get_cmdline_star() > 0 {
        return 1;
    }
    let cmdbuff = nvim_get_ccline_cmdbuff();
    if cmdbuff.is_null() {
        return 1;
    }
    ptr2cells(cmdbuff.add(idx as usize))
}

/// Correct screen column for multi-byte characters that don't fit.
///
/// When a double-wide character doesn't fit at the end of a line,
/// we need to account for the '>' placeholder.
///
/// # Safety
///
/// `idx` must be a valid byte offset in the command buffer.
/// Calls C functions to access buffer and check character width.
pub unsafe fn correct_screencol(idx: c_int, cells: c_int, col: &mut c_int) {
    let cmdbuff = nvim_get_ccline_cmdbuff();
    if cmdbuff.is_null() {
        return;
    }
    let p = cmdbuff.add(idx as usize);
    let columns = nvim_get_columns();

    // If multi-byte char (>1 byte) is double-wide (2 cells)
    // and doesn't fit at end of line, increment column
    if utfc_ptr2len(p) > 1 && utf_ptr2cells(p) > 1 && (*col % columns) + cells > columns {
        *col += 1;
    }
}

/// Compute the column position for a byte position on the command line.
///
/// This is the main function for converting byte positions to screen positions.
///
/// # Safety
///
/// `bytepos` must be a valid byte offset in the command buffer.
/// Calls C functions to access buffer and screen state.
#[must_use]
pub unsafe fn cmd_screencol(bytepos: c_int) -> c_int {
    let columns = nvim_get_columns();
    let rows = nvim_get_rows();
    let mut col = cmd_startcol();

    // Calculate maximum displayable column
    let m = if key_was_typed() {
        if cmdline_win_active() {
            let wwidth = nvim_cmdline_win_width();
            let wheight = nvim_cmdline_win_height();
            let product = wwidth.saturating_mul(wheight);
            if product < 0 {
                MAXCOL
            } else {
                product
            }
        } else {
            let product = columns.saturating_mul(rows);
            if product < 0 {
                MAXCOL
            } else {
                product
            }
        }
    } else {
        MAXCOL
    };

    let cmdbuff = nvim_get_ccline_cmdbuff();
    if cmdbuff.is_null() {
        return col;
    }

    let cmdlen = nvim_get_ccline_cmdlen();
    let mut i = 0;

    while i < cmdlen && i < bytepos {
        let c = cmdline_charsize(i);

        // Count ">" for double-wide multi-byte char that doesn't fit
        correct_screencol(i, c, &mut col);

        // If the cmdline doesn't fit, show cursor on last visible char
        if col + c >= m {
            break;
        }
        col += c;

        // Move to next character
        i += utfc_ptr2len(cmdbuff.add(i as usize));
    }

    col
}

/// Get the screen position of the cursor.
///
/// # Safety
///
/// Calls C functions to access ccline state.
#[must_use]
pub unsafe fn cursor_screencol() -> c_int {
    cmd_screencol(nvim_get_ccline_cmdpos())
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Compute starting column for cursor on command line.
///
/// # Safety
///
/// Calls C functions to access ccline state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmd_startcol() -> c_int {
    cmd_startcol()
}

/// Get character width at position in command buffer.
///
/// # Safety
///
/// `idx` must be valid byte offset.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_charsize(idx: c_int) -> c_int {
    cmdline_charsize(idx)
}

/// Compute screen column for byte position.
///
/// Direct C replacement for `cmd_screencol()`.
///
/// # Safety
///
/// `bytepos` must be valid byte offset.
#[must_use]
#[export_name = "cmd_screencol"]
pub unsafe extern "C" fn rs_cmd_screencol(bytepos: c_int) -> c_int {
    cmd_screencol(bytepos)
}

/// Get cursor screen column.
///
/// # Safety
///
/// Calls C functions to access state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cursor_screencol() -> c_int {
    cursor_screencol()
}

/// Check if password mode is active.
///
/// # Safety
///
/// Calls C function to access global.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_password_mode() -> c_int {
    c_int::from(is_password_mode())
}

/// Direct C replacement for cmdline_is_password().
///
/// # Safety
///
/// Calls C function to access global.
#[must_use]
#[export_name = "cmdline_is_password"]
pub unsafe extern "C" fn cmdline_is_password_rs() -> bool {
    is_password_mode()
}

/// Get screen columns.
///
/// # Safety
///
/// Calls C function to access global.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_columns() -> c_int {
    nvim_get_columns()
}

/// Get screen rows.
///
/// # Safety
///
/// Calls C function to access global.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_rows() -> c_int {
    nvim_get_rows()
}

// =============================================================================
// Rendering/UI Function Migrations (Phase 3)
// =============================================================================

extern "C" {
    // Window accessors
    fn rs_lastwin_nofloating() -> *mut ();
    fn rs_global_stl_height() -> c_int;
    fn nvim_win_get_winrow(wp: *mut ()) -> c_int;
    fn nvim_win_get_w_height(wp: *mut ()) -> c_int;
    fn nvim_win_get_hsep_height(wp: *mut ()) -> c_int;
    fn nvim_win_get_status_height(wp: *mut ()) -> c_int;

    // Global state accessors
    fn nvim_get_exmode_active() -> bool;
    fn nvim_get_msg_scrolled() -> c_int;
    fn nvim_get_p_ch() -> i64;
    fn nvim_set_cmdline_row(val: c_int);
    fn nvim_set_lines_left(val: c_int);
    fn nvim_get_cmd_silent() -> c_int;
    fn nvim_set_msg_row(val: c_int);
    fn nvim_set_msg_col(val: c_int);
    fn ui_has(what: c_int) -> c_int;
    fn msg_cursor_goto(row: c_int, col: c_int);
    fn msg_start();
    fn msg_clr_eos();

    // For redrawcmdline
    fn nvim_set_need_wait_return(val: c_int);
    fn ui_cursor_shape();

    // For redrawcmdprompt
    fn msg_putchar(c: c_int);
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool);
    fn nvim_get_msg_col() -> c_int;
    fn nvim_get_msg_row() -> c_int;
    fn nvim_get_ccline_cmdprompt() -> *mut c_char;
    fn nvim_get_ccline_hl_id() -> c_int;
    fn nvim_set_ccline_cmdindent(val: c_int);
    fn nvim_set_ccline_redraw_state(val: c_int);

    // For draw_cmdline
    fn nvim_color_cmdline() -> bool;
    fn nvim_get_ccline_colors_size() -> usize;
    fn nvim_get_ccline_color_chunk(
        idx: usize,
        start_out: *mut c_int,
        end_out: *mut c_int,
        hl_id_out: *mut c_int,
    );
    fn msg_outtrans_len(msgstr: *const c_char, len: c_int, hl_id: c_int, hist: bool) -> c_int;

    // For redrawcmd
    fn sb_text_restart_cmdline();
    fn putcmdline(c: c_char, shift: bool);
    fn nvim_set_redrawing_cmdline(val: c_int);
    fn nvim_set_msg_no_more(val: c_int);
    fn nvim_set_msg_scroll(val: c_int);
    fn nvim_set_skip_redraw2(val: c_int);
    fn nvim_get_ccline_special_char() -> c_int;
    fn nvim_get_ccline_special_shift() -> c_int;
    fn nvim_set_ccline_special_char(val: c_int);
    fn nvim_set_ccline_cmdspos(val: c_int);
}

/// `kUICmdline` constant (from `ui_defs.h`).
const K_UI_CMDLINE: c_int = 32; // kUICmdline

/// Direct C replacement for compute_cmdrow().
///
/// Computes the row position for the command line.
///
/// # Safety
///
/// Calls C functions to access global state.
#[export_name = "compute_cmdrow"]
pub unsafe extern "C" fn compute_cmdrow_rs() {
    let rows = nvim_get_rows();
    let new_row = if nvim_get_exmode_active() || nvim_get_msg_scrolled() != 0 {
        rows - 1
    } else {
        let wp = rs_lastwin_nofloating();
        let winrow = nvim_win_get_winrow(wp);
        let height = nvim_win_get_w_height(wp);
        let hsep = nvim_win_get_hsep_height(wp);
        let status = nvim_win_get_status_height(wp);
        let stl = rs_global_stl_height();
        winrow + height + hsep + status + stl
    };

    let new_row = if new_row == rows && nvim_get_p_ch() > 0 {
        new_row - 1
    } else {
        new_row
    };

    nvim_set_cmdline_row(new_row);
    nvim_set_lines_left(new_row);
}

/// Direct C replacement for cursorcmd().
///
/// Positions cursor on the command line.
///
/// # Safety
///
/// Calls C functions to access and set global state.
#[export_name = "cursorcmd"]
pub unsafe extern "C" fn cursorcmd_rs() {
    if nvim_get_cmd_silent() != 0 || ui_has(K_UI_CMDLINE) != 0 {
        return;
    }

    let rows = nvim_get_rows();
    let columns = nvim_get_columns();
    let cmdline_row = nvim_get_cmdline_row();
    let cmdspos = nvim_get_ccline_cmdspos();

    let mut msg_row_val = cmdline_row + (cmdspos / columns);
    let msg_col_val = cmdspos % columns;

    if msg_row_val >= rows {
        msg_row_val = rows - 1;
    }

    nvim_set_msg_row(msg_row_val);
    nvim_set_msg_col(msg_col_val);
    msg_cursor_goto(msg_row_val, msg_col_val);
}

/// Direct C replacement for gotocmdline().
///
/// Goes to the command line position.
///
/// # Safety
///
/// Calls C functions to access global state.
#[export_name = "gotocmdline"]
pub unsafe extern "C" fn gotocmdline_rs(clr: bool) {
    if ui_has(K_UI_CMDLINE) != 0 {
        return;
    }
    msg_start();
    nvim_set_msg_col(0); // always start in column 0
    if clr {
        msg_clr_eos(); // will reset clear_cmdline
    }
    msg_cursor_goto(nvim_get_cmdline_row(), 0);
}

/// Direct C replacement for the static correct_screencol().
///
/// Adjusts column for double-wide multi-byte chars that don't fit.
///
/// # Safety
///
/// `idx` must be a valid byte offset into the command buffer.
#[export_name = "correct_screencol"]
pub unsafe extern "C" fn correct_screencol_export(idx: c_int, cells: c_int, col: *mut c_int) {
    if col.is_null() {
        return;
    }
    correct_screencol(idx, cells, &mut *col);
}

/// CmdRedraw enum values
const K_CMD_REDRAW_ALL: c_int = 2;

/// Redraw the prompt portion of the command line.
///
/// This is the Rust implementation of `redrawcmdprompt()` from ex_getln.c.
///
/// # Safety
///
/// Calls C functions to access and set global state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_redrawcmdprompt() {
    if nvim_get_cmd_silent() != 0 {
        return;
    }
    if ui_has(K_UI_CMDLINE) != 0 {
        nvim_set_ccline_redraw_state(K_CMD_REDRAW_ALL);
        return;
    }
    let cmdfirstc = nvim_get_ccline_cmdfirstc();
    if cmdfirstc != NUL {
        msg_putchar(cmdfirstc);
    }
    let cmdprompt = nvim_get_ccline_cmdprompt();
    if cmdprompt.is_null() {
        let cmdindent = nvim_get_ccline_cmdindent();
        for _ in 0..cmdindent {
            msg_putchar(b' ' as c_int);
        }
    } else {
        msg_puts_hl(cmdprompt, nvim_get_ccline_hl_id(), false);
        let columns = nvim_get_columns();
        let msg_col = nvim_get_msg_col();
        let msg_row = nvim_get_msg_row();
        let cmdline_row = nvim_get_cmdline_row();
        let new_indent = msg_col + (msg_row - cmdline_row) * columns;
        // reverse of cmd_startcol(): subtract 1 if there's a firstc
        let new_indent = if cmdfirstc == NUL {
            new_indent
        } else {
            new_indent - 1
        };
        nvim_set_ccline_cmdindent(new_indent);
    }
}

// =============================================================================
// draw_cmdline: draw part of the command line
// =============================================================================

/// Draw part of the command line starting at byte offset `start`, for `len` bytes.
///
/// Direct replacement for C `draw_cmdline()`.
///
/// # Safety
///
/// Calls C functions to access global state.
#[export_name = "draw_cmdline"]
pub unsafe extern "C" fn draw_cmdline_rs(start: c_int, len: c_int) {
    let cmdbuff = nvim_get_ccline_cmdbuff();
    // If cmdbuff is NULL or coloring fails, return early
    if cmdbuff.is_null() || !nvim_color_cmdline() {
        return;
    }

    if ui_has(K_UI_CMDLINE) != 0 {
        // In external UI mode, clear special_char and mark for full redraw
        nvim_set_ccline_special_char(NUL);
        nvim_set_ccline_redraw_state(K_CMD_REDRAW_ALL);
        return;
    }

    let star = nvim_get_cmdline_star();
    if star > 0 {
        // Password mode: show stars
        let mut i = 0;
        while i < len {
            msg_putchar(b'*' as c_int);
            let ptr = cmdbuff.add((start + i) as usize);
            let char_len = utfc_ptr2len(ptr);
            i += char_len;
        }
    } else {
        let colors_count = nvim_get_ccline_colors_size();
        if colors_count > 0 {
            // Colored mode: output each color chunk
            for idx in 0..colors_count {
                let mut chunk_start: c_int = 0;
                let mut chunk_end: c_int = 0;
                let mut chunk_hl: c_int = 0;
                nvim_get_ccline_color_chunk(
                    idx,
                    &raw mut chunk_start,
                    &raw mut chunk_end,
                    &raw mut chunk_hl,
                );
                if chunk_end <= start {
                    continue;
                }
                let effective_start = if chunk_start > start {
                    chunk_start
                } else {
                    start
                };
                let output_len = chunk_end - effective_start;
                if output_len > 0 {
                    let ptr = cmdbuff.add(effective_start as usize);
                    msg_outtrans_len(ptr, output_len, chunk_hl, false);
                }
            }
        } else {
            // No colors: output plain
            let ptr = cmdbuff.add(start as usize);
            msg_outtrans_len(ptr, len, 0, false);
        }
    }
}

// =============================================================================
// redrawcmd: redraw the entire command line
// =============================================================================

/// Redraw what is currently on the command line.
///
/// Direct replacement for C `redrawcmd()`.
///
/// # Safety
///
/// Calls C functions to access and set global state.
#[export_name = "redrawcmd"]
pub unsafe extern "C" fn redrawcmd_rs() {
    if nvim_get_cmd_silent() != 0 {
        return;
    }

    if ui_has(K_UI_CMDLINE) != 0 {
        draw_cmdline_rs(0, nvim_get_ccline_cmdlen());
        return;
    }

    // When 'incsearch' is set there may be no cmdbuff while redrawing
    let cmdbuff = nvim_get_ccline_cmdbuff();
    if cmdbuff.is_null() {
        msg_cursor_goto(nvim_get_cmdline_row(), 0);
        msg_clr_eos();
        return;
    }

    nvim_set_redrawing_cmdline(1);

    sb_text_restart_cmdline();
    msg_start();
    rs_redrawcmdprompt();

    // Don't use more prompt; truncate cmdline if it doesn't fit
    nvim_set_msg_no_more(1);
    draw_cmdline_rs(0, nvim_get_ccline_cmdlen());
    msg_clr_eos();
    nvim_set_msg_no_more(0);

    let new_cmdspos = cmd_screencol(nvim_get_ccline_cmdpos());
    nvim_set_ccline_cmdspos(new_cmdspos);

    let special_char = nvim_get_ccline_special_char();
    if special_char != NUL {
        let shift = nvim_get_ccline_special_shift() != 0;
        putcmdline(special_char as u8 as c_char, shift);
    }

    // An emsg() before may have set msg_scroll. Reset it in cmdline mode.
    nvim_set_msg_scroll(0);

    // Typing ':' at the more prompt may set skip_redraw. Reset it.
    nvim_set_skip_redraw2(0);

    nvim_set_redrawing_cmdline(0);
}

/// Direct C replacement for redrawcmdline().
///
/// Redraws the command line and updates the cursor.
///
/// # Safety
///
/// Calls C functions to access and set global state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_redrawcmdline() {
    if nvim_get_cmd_silent() != 0 {
        return;
    }
    nvim_set_need_wait_return(0);
    compute_cmdrow_rs();
    redrawcmd_rs();
    cursorcmd_rs();
    ui_cursor_shape();
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maxcol() {
        assert_eq!(MAXCOL, 0x7FFF_FFFF);
    }

    #[test]
    fn test_nul_constant() {
        assert_eq!(NUL, 0);
    }
}
