//! `jump_to_mouse()` -- turning a screen position into a buffer position.
//!
//! The other half of the mouse: given a row and column it finds the window,
//! decides whether the click landed in the text, the status line, a vertical
//! separator, the winbar, the sign or fold column or the tabline, scrolls the
//! window when a drag has left it, and moves the cursor to the character under
//! the pointer -- with `'virtualedit'`, folds, `'conceal'` and multibyte
//! widths all taken into account.  Its answer is the `IN_*`/`CURSOR_MOVED`
//! bitmask the caller branches on.
//!
//! The stages are [`classify`] (what the click landed on, recorded in the
//! statics the *next* call reads), then one of [`enter_window`] (the caller
//! may change focus) and [`drag_or_extend`] (it may not), and finally the
//! cursor move both of them fall through to.  C reaches the last two through
//! `goto retnomove` and `goto foldclick`.
//!
//! Original: `src/nvim/mouse.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;
use core::ptr;

use super::*;
use crate::drawscreen::{UPD_INVERTED, UPD_VALID, redraw_curbuf_later};
use crate::global_cell::GlobalCell;
use crate::main::{
    VIsual, VIsual_active, VIsual_reselect, cmdwin_type, cmdwin_win, mouse_col, mouse_dragging,
    mouse_past_bottom, mouse_past_eol, mouse_row, msg_silent, p_smd, redraw_cmdline,
};
use crate::normal::{end_visual_mode, may_start_select};
use crate::types::pos_T;

// What the last event that could move focus landed on.  A drag or a release
// must act on the *same* status line or separator the press did, so these
// outlive the call that set them -- which is also what makes `retnomove`
// answerable without looking at the screen again.
static status_line_offset: GlobalCell<c_int> = GlobalCell::new(0);
static sep_line_offset: GlobalCell<c_int> = GlobalCell::new(0);
static on_status_line: GlobalCell<bool> = GlobalCell::new(false);
static on_sep_line: GlobalCell<bool> = GlobalCell::new(false);
static on_winbar: GlobalCell<bool> = GlobalCell::new(false);
static on_statuscol: GlobalCell<bool> = GlobalCell::new(false);
static prev_row: GlobalCell<c_int> = GlobalCell::new(-1);
static prev_col: GlobalCell<c_int> = GlobalCell::new(-1);
/// Whether a drag was noticed, so that the release may still move focus.
static did_drag: GlobalCell<c_int> = GlobalCell::new(0);

/// Move the cursor to the specified row and column on the screen.
/// Change current window if necessary. Returns an integer with the
/// `CURSOR_MOVED` bit set if the cursor has moved or unset otherwise.
///
/// The `MOUSE_FOLD_CLOSE` bit is set when clicked on the '-' in a fold column.
/// The `MOUSE_FOLD_OPEN` bit is set when clicked on the '+' in a fold column.
///
/// If flags has `MOUSE_FOCUS`, then the current window will not be changed, and
/// if the mouse is outside the window then the text will scroll, or if the
/// mouse was previously on a status line, then the status line may be dragged.
///
/// If flags has `MOUSE_MAY_VIS`, then Visual mode will be started before the
/// cursor is moved unless the cursor was on a status line or window bar.
/// This function returns one of `IN_UNKNOWN`, `IN_BUFFER`, `IN_STATUS_LINE` or
/// `IN_SEP_LINE` depending on where the cursor was clicked.
///
/// If flags has `MOUSE_MAY_STOP_VIS`, then Visual mode will be stopped, unless
/// the mouse is on the status line or window bar of the same window.
///
/// If flags has `MOUSE_DID_MOVE`, nothing is done if the mouse didn't move
/// since the last call.
///
/// If flags has `MOUSE_SETPOS`, nothing is done, only the current position is
/// remembered.
///
/// # Safety
/// `inclusive` must be writable or null.
pub unsafe fn jump_to_mouse(flags: c_int, inclusive: *mut bool, which_button: c_int) -> c_int {
    // SAFETY: the caller's promise.
    let inclusive = unsafe { inclusive.as_mut() };
    jump(flags, inclusive, which_button)
}

fn jump(mut flags: c_int, inclusive: Option<&mut bool>, which_button: c_int) -> c_int {
    let keep_focus = flags & MOUSE_FOCUS != 0;

    mouse_past_bottom.set(false);
    mouse_past_eol.set(false);

    if flags & MOUSE_RELEASED != 0 {
        // On button release we may change window focus if positioned on a
        // status line and no dragging happened.
        if !dragwin.get().is_null() && did_drag.get() == 0 {
            flags &= !(MOUSE_FOCUS | MOUSE_DID_MOVE);
        }
        dragwin.set(ptr::null_mut());
        did_drag.set(0);
    }

    if flags & MOUSE_DID_MOVE != 0
        && prev_row.get() == mouse_row.get()
        && prev_col.get() == mouse_col.get()
    {
        return no_move(flags);
    }

    prev_row.set(mouse_row.get());
    prev_col.set(mouse_col.get());

    if flags & MOUSE_SETPOS != 0 {
        return no_move(flags);
    }

    let mut pos = MousePos::current();
    if pos.row < 0 || pos.col < 0 {
        return IN_UNKNOWN; // check if it makes sense
    }

    // Find the window the row is in and adjust the position to be relative to
    // the top-left of the window inner area.
    let Some(win) = find_win_inner(&mut pos) else {
        return IN_UNKNOWN;
    };
    let below_window = classify(pos, win);

    if keep_focus {
        // If we can't change focus, set row, col and grid back to absolute
        // values, since the values relative to the window are only used when
        // keep_focus is false.
        pos = MousePos::current();
    }

    // SAFETY: `curwin` is live from startup to exit.
    let old_curwin = unsafe { Win::current() };
    let old_cursor = old_curwin.w_cursor;

    let answered = if keep_focus {
        drag_or_extend(&mut pos, flags, which_button)
    } else {
        enter_window(&mut pos, win, flags, below_window, old_curwin)
    };
    if let Some(answer) = answered {
        return answer;
    }

    move_cursor_there(pos, flags, inclusive, old_curwin, old_cursor)
}

/// The `retnomove` arm: the pointer has not moved (or the caller only wanted
/// the position remembered), so answer from what the last event recorded.
fn no_move(flags: c_int) -> c_int {
    // Before moving the cursor for a left click which is NOT in a status
    // line, stop Visual mode.
    if status_line_offset.get() != 0 {
        return IN_STATUS_LINE;
    }
    if sep_line_offset.get() != 0 {
        return IN_SEP_LINE;
    }
    if on_winbar.get() {
        return IN_OTHER_WIN | MOUSE_WINBAR;
    }
    if on_statuscol.get() {
        return IN_OTHER_WIN | MOUSE_STATUSCOL;
    }
    if flags & MOUSE_MAY_STOP_VIS != 0 {
        stop_visual();
    }
    IN_BUFFER
}

/// Leave Visual mode and delete the inversion.
fn stop_visual() {
    end_visual_mode();
    // SAFETY: only schedules a redraw of the current buffer.
    unsafe { redraw_curbuf_later(UPD_INVERTED) };
}

/// Record what the click landed on.  Answers whether it was at or below the
/// window's last text row.
///
/// The four `on_*` flags outlive the call: `retnomove` and the `keep_focus`
/// arms of a *later* drag or release read them back.
fn classify(pos: MousePos, win: Win) -> bool {
    let below_window =
        pos.grid == DEFAULT_GRID_HANDLE && pos.row + win.w_winbar_height >= win.w_height;
    on_status_line.set(below_window && pos.row + win.w_winbar_height - win.w_height + 1 == 1);
    on_sep_line.set(
        pos.grid == DEFAULT_GRID_HANDLE && pos.col >= win.w_width && pos.col - win.w_width == 0,
    );
    on_winbar.set(pos.row < 0 && pos.row + win.w_winbar_height >= 0);
    on_statuscol.set(
        !below_window
            && !on_status_line.get()
            && !on_sep_line.get()
            && !on_winbar.get()
            && !win.statuscolumn_empty()
            && in_statuscolumn(pos, win),
    );

    // The rightmost character of the status line might be a vertical
    // separator character if there is no connecting window to the right.
    if on_status_line.get() && on_sep_line.get() {
        if win.status_line_connected() {
            on_sep_line.set(false);
        } else {
            on_status_line.set(false);
        }
    }

    below_window
}

/// Whether the column is inside the window's `'statuscolumn'`, which sits on
/// the right in a 'rightleft' window.
fn in_statuscolumn(pos: MousePos, win: Win) -> bool {
    if win.w_onebuf_opt.wo_rl != 0 {
        pos.col >= win.w_view_width - win.col_off()
    } else {
        pos.col < win.col_off()
    }
}

/// The path a press takes: work out what may be dragged, stop Visual mode when
/// the click leaves the selection, and move focus to the window that was hit.
///
/// Answers `Some` when the caller must not go on to move the cursor.  `None`
/// is C's fall-through to `foldclick`, which the `'statuscolumn'` arm reaches
/// by `goto` before any of this runs.
fn enter_window(
    pos: &mut MousePos,
    mut win: Win,
    flags: c_int,
    below_window: bool,
    old_curwin: Win,
) -> Option<c_int> {
    if on_winbar.get() {
        return Some(IN_OTHER_WIN | MOUSE_WINBAR);
    }
    if on_statuscol.get() {
        return None; // straight on to the fold click
    }

    let fdc = win.fdccol_count();
    dragwin.set(ptr::null_mut());

    // winpos and height may change in win_enter()!
    if below_window {
        // In (or below) status line
        status_line_offset.set(pos.row + win.w_winbar_height - win.w_height + 1);
        dragwin.set(win.raw());
    } else {
        status_line_offset.set(0);
    }

    if pos.grid == DEFAULT_GRID_HANDLE && pos.col >= win.w_width {
        // In separator line
        sep_line_offset.set(pos.col - win.w_width + 1);
        dragwin.set(win.raw());
    } else {
        sep_line_offset.set(0);
    }

    // The rightmost character of the status line might be a vertical
    // separator character if there is no connecting window to the right.
    if status_line_offset.get() != 0 && sep_line_offset.get() != 0 {
        if win.status_line_connected() {
            sep_line_offset.set(0);
        } else {
            status_line_offset.set(0);
        }
    }

    // Before jumping to another buffer, or moving the cursor for a left
    // click, stop Visual mode.  `old_curwin` is still the current window here.
    let past_columns = if win.w_onebuf_opt.wo_rl != 0 {
        pos.col < win.w_view_width - fdc
    } else {
        pos.col >= fdc + (win.raw() == cmdwin_win.get()) as c_int
    };
    if VIsual_active.get()
        && (win.buffer() != old_curwin.buffer()
            || (status_line_offset.get() == 0
                && sep_line_offset.get() == 0
                && past_columns
                && flags & MOUSE_MAY_STOP_VIS != 0))
    {
        stop_visual();
    }

    if cmdwin_type.get() != 0 && win.raw() != cmdwin_win.get() {
        // A click outside the command-line window: Use modeless selection if
        // possible.  Allow dragging the status lines.
        sep_line_offset.set(0);
        pos.row = 0;
        pos.col += win.w_wincol;
        // SAFETY: `cmdwin_win` is a live window while `cmdwin_type` is set.
        win = unsafe { Win::new(cmdwin_win.get()) };
    }

    // Only change window focus when not clicking on or dragging the status
    // line.  Do change focus when releasing the mouse button (MOUSE_FOCUS was
    // set above if we dragged first).
    if dragwin.get().is_null() || flags & MOUSE_RELEASED != 0 {
        win.enter(); // can make `win` invalid!
    }

    // SAFETY: `curwin` is live from startup to exit.
    let mut curwin_now = unsafe { Win::current() };
    // Set topline, to be able to check for double click ourselves.
    if curwin_now != old_curwin {
        set_mouse_topline(curwin_now);
    }
    // Don't use start_arrow() if we're in the same window.
    let moved = if curwin_now == old_curwin {
        0
    } else {
        CURSOR_MOVED
    };
    if status_line_offset.get() != 0 {
        // In (or below) status line
        return Some(IN_STATUS_LINE | moved);
    }
    if sep_line_offset.get() != 0 {
        return Some(IN_SEP_LINE | moved);
    }

    curwin_now.w_cursor.lnum = curwin_now.w_topline;
    None
}

/// The path a drag or a release takes, and a right click: the caller keeps its
/// window, so the event either resizes a window or scrolls the current one.
fn drag_or_extend(pos: &mut MousePos, flags: c_int, which_button: c_int) -> Option<c_int> {
    if status_line_offset.get() != 0 {
        if which_button == MOUSE_LEFT && !dragwin.get().is_null() {
            // SAFETY: `dragwin` holds a live window while a drag is in flight.
            let win = unsafe { Win::new(dragwin.get()) };
            // Drag the status line.
            let count = pos.row - win.w_winrow - win.w_height + 1 - status_line_offset.get();
            win.drag_status_line(count);
            did_drag.set(did_drag.get() | count);
        }
        return Some(IN_STATUS_LINE); // Cursor didn't move
    }
    if sep_line_offset.get() != 0 && which_button == MOUSE_LEFT {
        if !dragwin.get().is_null() {
            // SAFETY: as above.
            let win = unsafe { Win::new(dragwin.get()) };
            // Drag the separator column.
            let count = pos.col - win.w_wincol - win.w_width + 1 - sep_line_offset.get();
            win.drag_sep_line(count);
            did_drag.set(did_drag.get() | count);
        }
        return Some(IN_SEP_LINE); // Cursor didn't move
    }
    if on_status_line.get() && which_button == MOUSE_RIGHT {
        return Some(IN_STATUS_LINE);
    }
    if on_winbar.get() && which_button == MOUSE_RIGHT {
        // After a click on the window bar don't start Visual mode.
        return Some(IN_OTHER_WIN | MOUSE_WINBAR);
    }
    if on_statuscol.get() && which_button == MOUSE_RIGHT {
        // After a click on the status column don't start Visual mode.
        return Some(IN_OTHER_WIN | MOUSE_STATUSCOL);
    }

    // Before moving the cursor for a left click, stop Visual mode.
    if flags & MOUSE_MAY_STOP_VIS != 0 {
        stop_visual();
    }

    // SAFETY: `curwin` is live from startup to exit.
    let mut win = unsafe { Win::current() };
    if pos.grid == 0 {
        pos.row -= win.w_grid_alloc.comp_row + win.w_grid.row_offset;
        pos.col -= win.w_grid_alloc.comp_col + win.w_grid.col_offset;
    } else if pos.grid != DEFAULT_GRID_HANDLE {
        pos.row -= win.w_grid.row_offset;
        pos.col -= win.w_grid.col_offset;
    }

    // When clicking beyond the end of the window, scroll the screen.
    // Scroll by however many rows outside the window we are.
    if pos.row < 0 {
        scroll_back(win, pos.row);
        pos.row = 0;
    } else if pos.row >= win.w_view_height {
        scroll_forward(win, pos.row);
        pos.row = win.w_view_height - 1;
    } else if pos.row == 0
        && mouse_dragging.get() > 0
        && win.w_cursor.lnum == win.buffer().line_count()
        && win.w_cursor.lnum == win.w_topline
    {
        // When dragging the mouse, while the text has been scrolled up as far
        // as it goes, moving the mouse in the top line should scroll the text
        // down (done later when recomputing w_topline).
        win.w_valid &= !VALID_TOPLINE;
    }

    None
}

/// Scroll back until the row `row` screen lines above the window is on screen.
fn scroll_back(mut win: Win, row: c_int) {
    let mut count = 0;
    let mut first = true;
    while win.w_topline > 1 {
        if win.w_topfill < win.fill_above(win.w_topline) {
            count += 1;
        } else {
            count += win.plines(win.w_topline - 1, true);
        }
        if !first && count > -row {
            break;
        }
        first = false;
        if let Some(fold_start) = win.fold_first(win.w_topline) {
            win.w_topline = fold_start;
        }
        if win.w_topfill < win.fill_above(win.w_topline) {
            win.w_topfill += 1;
        } else {
            win.w_topline -= 1;
            win.w_topfill = 0;
        }
    }
    win.check_topfill(false);
    win.w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
    win.redraw_later(UPD_VALID);
}

/// Scroll forward until row `row` -- which is at or below the window's last
/// text row -- is on screen.
fn scroll_forward(mut win: Win, row: c_int) {
    let last_line = win.buffer().line_count();
    let mut count = 0;
    let mut first = true;
    while win.w_topline < last_line {
        if win.w_topfill > 0 {
            count += 1;
        } else {
            count += win.plines(win.w_topline, true);
        }
        if !first && count > row - win.w_view_height + 1 {
            break;
        }
        first = false;

        if win.w_topfill > 0 {
            win.w_topfill -= 1;
        } else {
            let (folded, _, fold_end) = win.fold_span(win.w_topline);
            win.w_topline = fold_end;
            if folded && fold_end == last_line {
                break;
            }
            win.w_topline += 1;
            win.w_topfill = win.fill_above(win.w_topline);
        }
    }
    win.check_topfill(false);
    win.redraw_later(UPD_VALID);
    win.w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
}

/// C's `foldclick:` tail, which both paths above fall into: read the fold
/// markers off the drawn screen, turn the screen position into a buffer
/// position and put the cursor there.
fn move_cursor_there(
    mut pos: MousePos,
    flags: c_int,
    inclusive: Option<&mut bool>,
    old_curwin: Win,
    old_cursor: pos_T,
) -> c_int {
    let (col_from_screen, mouse_fold_flags) = mouse_check_grid();

    // SAFETY: `curwin` is live from startup to exit.
    let mut win = unsafe { Win::current() };
    // Compute the position in the buffer line from the position on the screen.
    let (lnum, below_last) = comp_pos(win, &mut pos.row, &mut pos.col);
    win.w_cursor.lnum = lnum;
    if below_last {
        mouse_past_bottom.set(true);
    }

    // Start Visual mode before coladvance(), for when 'sel' != "old"
    if flags & MOUSE_MAY_VIS != 0 && !VIsual_active.get() {
        VIsual.set(old_cursor);
        VIsual_active.set(true);
        VIsual_reselect.set(1);
        // If 'selectmode' contains "mouse", start Select mode.
        may_start_select('o' as c_int);
        setmouse();

        if p_smd.get() != 0 && msg_silent.get() == 0 {
            redraw_cmdline.set(true); // show visual mode later
        }
    }

    // Use the virtual column the screen recorded, which is accurate also
    // after concealed characters.
    let col = col_from_screen.unwrap_or(pos.col);
    win.w_curswant = col;
    win.w_set_curswant = 0; // May still have been true
    let past_eol = !win.coladvance(col);
    if let Some(inclusive) = inclusive {
        *inclusive = past_eol;
    }
    if past_eol {
        // Mouse click beyond end of line.
        mouse_past_eol.set(true);
    }

    let mut count = if on_statuscol.get() {
        IN_OTHER_WIN | MOUSE_STATUSCOL
    } else {
        IN_BUFFER
    };
    if win != old_curwin
        || win.w_cursor.lnum != old_cursor.lnum
        || win.w_cursor.col != old_cursor.col
    {
        count |= CURSOR_MOVED; // Cursor has moved
    }

    count | mouse_fold_flags
}
