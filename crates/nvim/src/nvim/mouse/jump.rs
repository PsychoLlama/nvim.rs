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
//! Original: `src/nvim/mouse.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::cursor::coladvance;
use crate::src::nvim::drawscreen::{UPD_INVERTED, UPD_VALID, redraw_curbuf_later, redraw_later};
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    VIsual, VIsual_active, VIsual_reselect, cmdwin_type, cmdwin_win, curbuf, curwin, mouse_col,
    mouse_dragging, mouse_grid, mouse_past_bottom, mouse_past_eol, mouse_row, msg_silent, p_smd,
    redraw_cmdline,
};
use crate::src::nvim::r#move::{check_topfill, win_col_off};
use crate::src::nvim::normal::{end_visual_mode, may_start_select};
use crate::src::nvim::plines::{plines_win, win_get_fill};
use crate::src::nvim::statusline::stl_connected;
use crate::src::nvim::types::{colnr_T, linenr_T, pos_T, win_T};
use crate::src::nvim::window::{
    win_drag_status_line, win_drag_vsep_line, win_enter, win_fdccol_count,
};

pub unsafe extern "C" fn jump_to_mouse(
    mut flags: ::core::ffi::c_int,
    mut inclusive: *mut bool,
    mut which_button: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        static status_line_offset: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        static sep_line_offset: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        static on_status_line: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        static on_sep_line: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        static on_winbar: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        static on_statuscol: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        static prev_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
        static prev_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
        static did_drag: GlobalCell<::core::ffi::c_int> = GlobalCell::new(false_0);
        let mut count: ::core::ffi::c_int = 0;
        let mut first: bool = false;
        let mut row: ::core::ffi::c_int = mouse_row.get();
        let mut col: ::core::ffi::c_int = mouse_col.get();
        let mut grid: ::core::ffi::c_int = mouse_grid.get();
        let mut fdc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut keep_focus: bool = flags & MOUSE_FOCUS as ::core::ffi::c_int != 0;
        mouse_past_bottom.set(false_0 != 0);
        mouse_past_eol.set(false_0 != 0);
        if flags & MOUSE_RELEASED as ::core::ffi::c_int != 0 {
            if !(*dragwin.ptr()).is_null() && did_drag.get() == 0 {
                flags &=
                    !(MOUSE_FOCUS as ::core::ffi::c_int | MOUSE_DID_MOVE as ::core::ffi::c_int);
            }
            dragwin.set(::core::ptr::null_mut::<win_T>());
            did_drag.set(false_0);
        }
        if !(flags & MOUSE_DID_MOVE as ::core::ffi::c_int != 0
            && prev_row.get() == mouse_row.get()
            && prev_col.get() == mouse_col.get())
        {
            prev_row.set(mouse_row.get());
            prev_col.set(mouse_col.get());
            if flags & MOUSE_SETPOS as ::core::ffi::c_int == 0 {
                if row < 0 as ::core::ffi::c_int || col < 0 as ::core::ffi::c_int {
                    return IN_UNKNOWN as ::core::ffi::c_int;
                }
                let mut wp: *mut win_T =
                    mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
                if wp.is_null() {
                    return IN_UNKNOWN as ::core::ffi::c_int;
                }
                let mut below_window: bool =
                    grid == DEFAULT_GRID_HANDLE && row + (*wp).w_winbar_height >= (*wp).w_height;
                on_status_line.set(
                    below_window as ::core::ffi::c_int != 0
                        && row + (*wp).w_winbar_height - (*wp).w_height + 1 as ::core::ffi::c_int
                            == 1 as ::core::ffi::c_int,
                );
                on_sep_line.set(
                    grid == DEFAULT_GRID_HANDLE
                        && col >= (*wp).w_width
                        && col - (*wp).w_width + 1 as ::core::ffi::c_int == 1 as ::core::ffi::c_int,
                );
                on_winbar.set(
                    row < 0 as ::core::ffi::c_int
                        && row + (*wp).w_winbar_height >= 0 as ::core::ffi::c_int,
                );
                on_statuscol.set(
                    !below_window
                        && !on_status_line.get()
                        && !on_sep_line.get()
                        && !on_winbar.get()
                        && *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
                        && (if (*wp).w_onebuf_opt.wo_rl != 0 {
                            (col >= (*wp).w_view_width - win_col_off(wp)) as ::core::ffi::c_int
                        } else {
                            (col < win_col_off(wp)) as ::core::ffi::c_int
                        }) != 0,
                );
                if on_status_line.get() as ::core::ffi::c_int != 0
                    && on_sep_line.get() as ::core::ffi::c_int != 0
                {
                    if stl_connected(wp) {
                        on_sep_line.set(false_0 != 0);
                    } else {
                        on_status_line.set(false_0 != 0);
                    }
                }
                if keep_focus {
                    row = mouse_row.get();
                    col = mouse_col.get();
                    grid = mouse_grid.get();
                }
                let mut old_curwin: *mut win_T = curwin.get();
                let mut old_cursor: pos_T = (*curwin.get()).w_cursor;
                if !keep_focus {
                    if on_winbar.get() {
                        return IN_OTHER_WIN as ::core::ffi::c_int
                            | MOUSE_WINBAR as ::core::ffi::c_int;
                    }
                    if !on_statuscol.get() {
                        fdc = win_fdccol_count(wp);
                        dragwin.set(::core::ptr::null_mut::<win_T>());
                        if below_window {
                            status_line_offset.set(
                                row + (*wp).w_winbar_height - (*wp).w_height
                                    + 1 as ::core::ffi::c_int,
                            );
                            dragwin.set(wp);
                        } else {
                            status_line_offset.set(0 as ::core::ffi::c_int);
                        }
                        if grid == DEFAULT_GRID_HANDLE && col >= (*wp).w_width {
                            sep_line_offset.set(col - (*wp).w_width + 1 as ::core::ffi::c_int);
                            dragwin.set(wp);
                        } else {
                            sep_line_offset.set(0 as ::core::ffi::c_int);
                        }
                        if status_line_offset.get() != 0 && sep_line_offset.get() != 0 {
                            if stl_connected(wp) {
                                sep_line_offset.set(0 as ::core::ffi::c_int);
                            } else {
                                status_line_offset.set(0 as ::core::ffi::c_int);
                            }
                        }
                        if VIsual_active.get() as ::core::ffi::c_int != 0
                            && ((*wp).w_buffer != (*curwin.get()).w_buffer
                                || status_line_offset.get() == 0
                                    && sep_line_offset.get() == 0
                                    && (if (*wp).w_onebuf_opt.wo_rl != 0 {
                                        (col < (*wp).w_view_width - fdc) as ::core::ffi::c_int
                                    } else {
                                        (col >= fdc
                                            + (if wp != cmdwin_win.get() {
                                                0 as ::core::ffi::c_int
                                            } else {
                                                1 as ::core::ffi::c_int
                                            }))
                                            as ::core::ffi::c_int
                                    }) != 0
                                    && flags & MOUSE_MAY_STOP_VIS as ::core::ffi::c_int != 0)
                        {
                            end_visual_mode();
                            redraw_curbuf_later(UPD_INVERTED);
                        }
                        if cmdwin_type.get() != 0 as ::core::ffi::c_int && wp != cmdwin_win.get() {
                            sep_line_offset.set(0 as ::core::ffi::c_int);
                            row = 0 as ::core::ffi::c_int;
                            col += (*wp).w_wincol;
                            wp = cmdwin_win.get();
                        }
                        if (*dragwin.ptr()).is_null()
                            || flags & MOUSE_RELEASED as ::core::ffi::c_int != 0
                        {
                            win_enter(wp, true_0 != 0);
                        }
                        if curwin.get() != old_curwin {
                            set_mouse_topline(curwin.get());
                        }
                        if status_line_offset.get() != 0 {
                            if curwin.get() == old_curwin {
                                return IN_STATUS_LINE as ::core::ffi::c_int;
                            }
                            return IN_STATUS_LINE as ::core::ffi::c_int
                                | CURSOR_MOVED as ::core::ffi::c_int;
                        }
                        if sep_line_offset.get() != 0 {
                            if curwin.get() == old_curwin {
                                return IN_SEP_LINE as ::core::ffi::c_int;
                            }
                            return IN_SEP_LINE as ::core::ffi::c_int
                                | CURSOR_MOVED as ::core::ffi::c_int;
                        }
                        (*curwin.get()).w_cursor.lnum = (*curwin.get()).w_topline;
                    }
                } else if status_line_offset.get() != 0 {
                    if which_button == MOUSE_LEFT as ::core::ffi::c_int
                        && !(*dragwin.ptr()).is_null()
                    {
                        count = row - (*dragwin.get()).w_winrow - (*dragwin.get()).w_height
                            + 1 as ::core::ffi::c_int
                            - status_line_offset.get();
                        win_drag_status_line(dragwin.get(), count);
                        (*did_drag.ptr()) |= count;
                    }
                    return IN_STATUS_LINE as ::core::ffi::c_int;
                } else if sep_line_offset.get() != 0
                    && which_button == MOUSE_LEFT as ::core::ffi::c_int
                {
                    if !(*dragwin.ptr()).is_null() {
                        count = col - (*dragwin.get()).w_wincol - (*dragwin.get()).w_width
                            + 1 as ::core::ffi::c_int
                            - sep_line_offset.get();
                        win_drag_vsep_line(dragwin.get(), count);
                        (*did_drag.ptr()) |= count;
                    }
                    return IN_SEP_LINE as ::core::ffi::c_int;
                } else if on_status_line.get() as ::core::ffi::c_int != 0
                    && which_button == MOUSE_RIGHT as ::core::ffi::c_int
                {
                    return IN_STATUS_LINE as ::core::ffi::c_int;
                } else if on_winbar.get() as ::core::ffi::c_int != 0
                    && which_button == MOUSE_RIGHT as ::core::ffi::c_int
                {
                    return IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_WINBAR as ::core::ffi::c_int;
                } else if on_statuscol.get() as ::core::ffi::c_int != 0
                    && which_button == MOUSE_RIGHT as ::core::ffi::c_int
                {
                    return IN_OTHER_WIN as ::core::ffi::c_int
                        | MOUSE_STATUSCOL as ::core::ffi::c_int;
                } else {
                    if flags & MOUSE_MAY_STOP_VIS as ::core::ffi::c_int != 0 {
                        end_visual_mode();
                        redraw_curbuf_later(UPD_INVERTED);
                    }
                    if grid == 0 as ::core::ffi::c_int {
                        row -= (*curwin.get()).w_grid_alloc.comp_row
                            + (*curwin.get()).w_grid.row_offset;
                        col -= (*curwin.get()).w_grid_alloc.comp_col
                            + (*curwin.get()).w_grid.col_offset;
                    } else if grid != DEFAULT_GRID_HANDLE {
                        row -= (*curwin.get()).w_grid.row_offset;
                        col -= (*curwin.get()).w_grid.col_offset;
                    }
                    if row < 0 as ::core::ffi::c_int {
                        count = 0 as ::core::ffi::c_int;
                        first = true_0 != 0;
                        while (*curwin.get()).w_topline > 1 as linenr_T {
                            if (*curwin.get()).w_topfill
                                < win_get_fill(curwin.get(), (*curwin.get()).w_topline)
                            {
                                count += 1;
                            } else {
                                count += plines_win(
                                    curwin.get(),
                                    (*curwin.get()).w_topline - 1 as linenr_T,
                                    true_0 != 0,
                                );
                            }
                            if !first && count > -row {
                                break;
                            }
                            first = false_0 != 0;
                            hasFolding(
                                curwin.get(),
                                (*curwin.get()).w_topline,
                                &raw mut (*curwin.get()).w_topline,
                                ::core::ptr::null_mut::<linenr_T>(),
                            );
                            if (*curwin.get()).w_topfill
                                < win_get_fill(curwin.get(), (*curwin.get()).w_topline)
                            {
                                (*curwin.get()).w_topfill += 1;
                            } else {
                                (*curwin.get()).w_topline -= 1;
                                (*curwin.get()).w_topfill = 0 as ::core::ffi::c_int;
                            }
                        }
                        check_topfill(curwin.get(), false_0 != 0);
                        (*curwin.get()).w_valid &=
                            !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
                        redraw_later(curwin.get(), UPD_VALID);
                        row = 0 as ::core::ffi::c_int;
                    } else if row >= (*curwin.get()).w_view_height {
                        count = 0 as ::core::ffi::c_int;
                        first = true_0 != 0;
                        while (*curwin.get()).w_topline < (*curbuf.get()).b_ml.ml_line_count {
                            if (*curwin.get()).w_topfill > 0 as ::core::ffi::c_int {
                                count += 1;
                            } else {
                                count += plines_win(
                                    curwin.get(),
                                    (*curwin.get()).w_topline,
                                    true_0 != 0,
                                );
                            }
                            if !first
                                && count
                                    > row - (*curwin.get()).w_view_height + 1 as ::core::ffi::c_int
                            {
                                break;
                            }
                            first = false_0 != 0;
                            if (*curwin.get()).w_topfill > 0 as ::core::ffi::c_int {
                                (*curwin.get()).w_topfill -= 1;
                            } else {
                                if hasFolding(
                                    curwin.get(),
                                    (*curwin.get()).w_topline,
                                    ::core::ptr::null_mut::<linenr_T>(),
                                    &raw mut (*curwin.get()).w_topline,
                                ) as ::core::ffi::c_int
                                    != 0
                                    && (*curwin.get()).w_topline
                                        == (*curbuf.get()).b_ml.ml_line_count
                                {
                                    break;
                                }
                                (*curwin.get()).w_topline += 1;
                                (*curwin.get()).w_topfill =
                                    win_get_fill(curwin.get(), (*curwin.get()).w_topline);
                            }
                        }
                        check_topfill(curwin.get(), false_0 != 0);
                        redraw_later(curwin.get(), UPD_VALID);
                        (*curwin.get()).w_valid &=
                            !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
                        row = (*curwin.get()).w_view_height - 1 as ::core::ffi::c_int;
                    } else if row == 0 as ::core::ffi::c_int {
                        if mouse_dragging.get() > 0 as ::core::ffi::c_int
                            && (*curwin.get()).w_cursor.lnum
                                == (*(*curwin.get()).w_buffer).b_ml.ml_line_count
                            && (*curwin.get()).w_cursor.lnum == (*curwin.get()).w_topline
                        {
                            (*curwin.get()).w_valid &= !VALID_TOPLINE;
                        }
                    }
                }
                let mut col_from_screen: colnr_T = -1 as colnr_T;
                let mut mouse_fold_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                mouse_check_grid(&raw mut col_from_screen, &raw mut mouse_fold_flags);
                if mouse_comp_pos(
                    curwin.get(),
                    &raw mut row,
                    &raw mut col,
                    &raw mut (*curwin.get()).w_cursor.lnum,
                ) {
                    mouse_past_bottom.set(true_0 != 0);
                }
                if flags & MOUSE_MAY_VIS as ::core::ffi::c_int != 0 && !VIsual_active.get() {
                    VIsual.set(old_cursor);
                    VIsual_active.set(true_0 != 0);
                    VIsual_reselect.set(true_0);
                    may_start_select('o' as ::core::ffi::c_int);
                    setmouse();
                    if p_smd.get() != 0 && msg_silent.get() == 0 as ::core::ffi::c_int {
                        redraw_cmdline.set(true_0 != 0);
                    }
                }
                if col_from_screen >= 0 as ::core::ffi::c_int {
                    col = col_from_screen as ::core::ffi::c_int;
                }
                (*curwin.get()).w_curswant = col as colnr_T;
                (*curwin.get()).w_set_curswant = false_0;
                if !coladvance(curwin.get(), col as colnr_T) {
                    if !inclusive.is_null() {
                        *inclusive = true_0 != 0;
                    }
                    mouse_past_eol.set(true_0 != 0);
                } else if !inclusive.is_null() {
                    *inclusive = false_0 != 0;
                }
                count = if on_statuscol.get() as ::core::ffi::c_int != 0 {
                    IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_STATUSCOL as ::core::ffi::c_int
                } else {
                    IN_BUFFER as ::core::ffi::c_int
                };
                if curwin.get() != old_curwin
                    || (*curwin.get()).w_cursor.lnum != old_cursor.lnum
                    || (*curwin.get()).w_cursor.col != old_cursor.col
                {
                    count |= CURSOR_MOVED as ::core::ffi::c_int;
                }
                count |= mouse_fold_flags;
                return count;
            }
        }
        if status_line_offset.get() != 0 {
            return IN_STATUS_LINE as ::core::ffi::c_int;
        }
        if sep_line_offset.get() != 0 {
            return IN_SEP_LINE as ::core::ffi::c_int;
        }
        if on_winbar.get() {
            return IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_WINBAR as ::core::ffi::c_int;
        }
        if on_statuscol.get() {
            return IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_STATUSCOL as ::core::ffi::c_int;
        }
        if flags & MOUSE_MAY_STOP_VIS as ::core::ffi::c_int != 0 {
            end_visual_mode();
            redraw_curbuf_later(UPD_INVERTED);
        }
        return IN_BUFFER as ::core::ffi::c_int;
    }
}
