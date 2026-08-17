//! Which window, row and column a screen position names --
//! `mouse_find_win()` and `mouse_comp_pos()`.
//!
//! [`comp_pos`] converts a window-relative row into a buffer line, walking
//! wrapped lines, folds and diff filler; [`find_win_inner`] and
//! [`find_win_outer`] walk the frame tree for the window containing a
//! screen position (the outer form counts the status line and separator as
//! belonging to the window above/left of them); `find_grid_win` is the
//! `ext_multigrid` half that maps a grid handle plus coordinates onto both.
//! [`vcol_to_col`] is the column half.
//!
//! The C's three `int *` out-parameters are one value, so the finders take a
//! [`MousePos`] and rewrite it in place.
//!
//! Original: `src/nvim/mouse.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;
use core::ptr;

#[allow(unused_imports)]
use super::*;
use crate::grid::get_win_by_grid_handle;
use crate::main::{firstwin, msg_grid, msg_grid_pos, pum_grid, topframe};
use crate::plines::{init_charsize_arg, win_charsize};
use crate::types::{CharsizeArg, handle_T, linenr_T};
use crate::ui_compositor::ui_comp_mouse_focus;
use crate::winlayer::{Frame, windows};

/// The screen row the first window starts at: the tab page line's height.
pub fn first_window_row() -> c_int {
    // SAFETY: the window list is live from startup to exit.
    unsafe { (*firstwin.get()).w_winrow }
}

// ---------------------------------------------------------------------------
// Screen position to window

/// Find the window at `pos`, rewriting it to be relative to the top-left of
/// that window's inner area.
///
/// Answers `None` when something is wrong -- including a click on the popup
/// menu, which has no window of its own.
pub fn find_win_inner(pos: &mut MousePos) -> Option<Win> {
    if let Some(win) = find_grid_win(pos) {
        return Some(win);
    } else if pos.grid > 1 {
        return None;
    }

    // SAFETY: the layout tree is live from startup to exit.
    let mut fp = unsafe { Frame::new(topframe.get()) };
    pos.row -= first_window_row();
    while fp.fr_layout as c_int != FR_LEAF {
        // Upstream dereferences `fr_child` unchecked: a non-leaf frame always
        // has children.  A missing one leaves `fp` non-leaf, whose `fr_win` is
        // null, and the search below then answers None.
        let Some(mut child) = fp.child() else { break };
        let by_column = fp.fr_layout as c_int == FR_ROW;
        // The last child is taken without a test, as the C's `for` is written.
        while let Some(sibling) = child.next() {
            if by_column {
                if pos.col < child.fr_width {
                    break;
                }
                pos.col -= child.fr_width;
            } else {
                if pos.row < child.fr_height {
                    break;
                }
                pos.row -= child.fr_height;
            }
            child = sibling;
        }
        fp = child;
    }

    // When using a timer that closes a window the window might not actually
    // exist.
    let win = windows().find(|wp| wp.raw() == fp.fr_win)?;
    pos.row -= win.w_winbar_height;
    Some(win)
}

/// [`find_win_inner`], with `pos` left relative to the top-left of the whole
/// window rather than of its inner area.
pub fn find_win_outer(pos: &mut MousePos) -> Option<Win> {
    let win = find_win_inner(pos)?;
    pos.row += win.w_winrow_off;
    pos.col += win.w_wincol_off;
    Some(win)
}

/// The `ext_multigrid` half: map a grid handle plus coordinates onto the
/// window that drew them, rewriting `pos` for the grid it settled on.
fn find_grid_win(pos: &mut MousePos) -> Option<Win> {
    if pos.grid == msg_grid.with(|grid| grid.handle) {
        pos.row += msg_grid_pos.get();
        pos.grid = DEFAULT_GRID_HANDLE;
    } else if pos.grid > 1 {
        // SAFETY: the handle table answers a live window or null.
        let wp = unsafe { get_win_by_grid_handle(pos.grid as handle_T) };
        if wp.is_null() {
            return None;
        }
        // SAFETY: as above.
        let win = unsafe { Win::new(wp) };
        if !win.w_grid_alloc.chars.is_null() && !(win.w_floating && !win.w_config.mouse) {
            pos.row = (pos.row - win.w_grid.row_offset).min(win.w_view_height - 1);
            pos.col = (pos.col - win.w_grid.col_offset).min(win.w_view_width - 1);
            return Some(win);
        }
    } else if pos.grid == 0 {
        // SAFETY: the compositor's layer stack is live; the grid it answers is
        // one of the layers, or null.
        let grid = unsafe { ui_comp_mouse_focus(pos.row, pos.col) };
        if pum_grid.with(|pum| ptr::eq(grid, pum)) {
            // SAFETY: the popup menu's grid is live.
            unsafe {
                pos.grid = (*grid).handle as c_int;
                (pos.row, pos.col) = (pos.row - (*grid).comp_row, pos.col - (*grid).comp_col);
            }
            // The popup menu doesn't have a window, so answer None.
            return None;
        }
        for win in windows() {
            if !ptr::eq(&raw const win.w_grid_alloc, grid) {
                continue;
            }
            // SAFETY: the grid a window drew on is live.
            pos.grid = unsafe { (*grid).handle } as c_int;
            pos.row -= win.w_winrow + win.w_grid.row_offset;
            pos.col -= win.w_wincol + win.w_grid.col_offset;
            return Some(win);
        }

        // No grid found, return the default grid. With multigrid this happens
        // for split separators for example.
        pos.grid = DEFAULT_GRID_HANDLE;
    }
    None
}

// ---------------------------------------------------------------------------
// Screen position to buffer position

/// Compute the buffer line position from the screen position `row`/`col` in
/// window `win`, both rewritten to be relative to that line.
///
/// Answers the line, and whether the position is below the last one.
pub fn comp_pos(win: Win, row: &mut c_int, col: &mut c_int) -> (linenr_T, bool) {
    let mut screen_col = if win.w_onebuf_opt.wo_rl != 0 {
        win.w_view_width - 1 - *col
    } else {
        *col
    };
    let mut screen_row = *row;
    let mut below_last = false;
    let last_line = win.buffer().line_count();
    let mut lnum = win.w_topline;

    while screen_row > 0 {
        // Don't include filler lines in "count".
        let mut count = if win.may_fill() {
            screen_row -= if lnum == win.w_topline {
                win.w_topfill
            } else {
                win.fill_above(lnum)
            };
            win.plines_nofill(lnum, false)
        } else {
            win.plines(lnum, false)
        };

        if win.w_skipcol > 0 && lnum == win.w_topline {
            let (width1, width2) = win.text_widths();
            if width1 > 0 {
                // Adjust for 'smoothscroll' clipping the top screen lines.
                count -= skipped_top_lines(win.w_skipcol, width1, width2);
            }
        }

        if count > screen_row {
            break; // Position is in this buffer line.
        }

        lnum = win.fold_last(lnum);
        if lnum == last_line {
            below_last = true;
            break; // past end of file
        }
        screen_row -= count;
        lnum += 1;
    }

    // Mouse row reached, adjust lnum for concealed lines.
    while lnum < last_line && win.conceals_line(lnum - 1, false) {
        lnum = win.fold_last(lnum + 1);
    }

    if !below_last {
        // Compute the column without wrapping.
        let off = win.col_off() - win.col_off2();
        screen_col = screen_col.max(off) + screen_row * (win.w_view_width - off);
        // Add skip column for the topline.
        if lnum == win.w_topline {
            screen_col += win.w_skipcol;
        }
    }

    if win.w_onebuf_opt.wo_wrap == 0 {
        screen_col += win.w_leftcol;
    }

    // Skip the line number and fold column in front of the line.
    *col = (screen_col - win.col_off()).max(0);
    *row = screen_row;
    (lnum, below_last)
}

/// Convert a virtual (screen) column to a character column, the first column
/// being zero.  Answers the byte index and the columns left over inside the
/// character it landed in.
pub fn vcol_to_col(win: Win, lnum: linenr_T, vcol: colnr_T) -> (colnr_T, colnr_T) {
    // SAFETY: a live window, and `lnum` a line of the buffer it shows.
    let line = unsafe { win.buffer().line(lnum) };
    let mut csarg = CharsizeArg::default();
    // SAFETY: a live window and a NUL-terminated line of its buffer.
    let cstype = unsafe { init_charsize_arg(&mut csarg, win.raw(), lnum, line.raw()) };
    let mut ci = line.first_char();
    let mut cur_vcol: c_int = 0;
    // Try to advance to the specified column.
    // SAFETY: `ci` walks that line and the loop stops at its terminating NUL.
    unsafe {
        while cur_vcol < vcol && !line.ended(ci) {
            let width = win_charsize(cstype, cur_vcol, ci.ptr, ci.chr.value, &mut csarg).width;
            if cur_vcol + width > vcol {
                break;
            }
            cur_vcol += width;
            ci = line.next_char(ci);
        }
    }
    (line.index_of(ci), vcol - cur_vcol)
}

// ---------------------------------------------------------------------------
// The raw entry point the rest of the editor still calls

/// [`vcol_to_col`], writing the leftover columns through `coladdp`.
///
/// # Safety
/// `wp` must be a live window and `lnum` a line of the buffer it shows;
/// `coladdp` must be writable or null.
pub unsafe extern "C" fn vcol2col(
    wp: *mut win_T,
    lnum: linenr_T,
    vcol: colnr_T,
    coladdp: *mut colnr_T,
) -> colnr_T {
    // SAFETY: the caller's promise.
    let (col, coladd) = unsafe { vcol_to_col(Win::new(wp), lnum, vcol) };
    if !coladdp.is_null() {
        // SAFETY: as above.
        unsafe { *coladdp = coladd };
    }
    col
}
