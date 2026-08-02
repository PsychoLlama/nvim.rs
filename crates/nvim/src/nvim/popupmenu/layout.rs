//! Where the popup menu goes and how big it is.
//!
//! The widths come from the items ([`pum_compute_size`]); the row and
//! height from the space above and below the anchor
//! ([`pum_compute_vertical_placement`]); the column and width from the
//! cursor column and what is left of the screen
//! ([`pum_compute_horizontal_placement`]). [`pum_position_at_mouse`] is
//! the `:popup` variant, anchored on the mouse instead of the cursor.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn pum_compute_size() {
    unsafe {
        pum_base_width.set(0 as ::core::ffi::c_int);
        pum_kind_width.set(0 as ::core::ffi::c_int);
        pum_extra_width.set(0 as ::core::ffi::c_int);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < pum_size.get() {
            if !(*(*pum_array.ptr()).offset(i as isize)).pum_text.is_null() {
                let mut w: ::core::ffi::c_int =
                    vim_strsize((*(*pum_array.ptr()).offset(i as isize)).pum_text);
                if pum_base_width.get() < w {
                    pum_base_width.set(w);
                }
            }
            if !(*(*pum_array.ptr()).offset(i as isize)).pum_kind.is_null() {
                let mut w_0: ::core::ffi::c_int =
                    vim_strsize((*(*pum_array.ptr()).offset(i as isize)).pum_kind)
                        + 1 as ::core::ffi::c_int;
                if pum_kind_width.get() < w_0 {
                    pum_kind_width.set(w_0);
                }
            }
            if !(*(*pum_array.ptr()).offset(i as isize)).pum_extra.is_null() {
                let mut w_1: ::core::ffi::c_int =
                    vim_strsize((*(*pum_array.ptr()).offset(i as isize)).pum_extra)
                        + 1 as ::core::ffi::c_int;
                if pum_extra_width.get() < w_1 {
                    pum_extra_width.set(w_1);
                }
            }
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn pum_compute_vertical_placement(
    mut size: ::core::ffi::c_int,
    mut target_win: *mut win_T,
    mut pum_win_row: ::core::ffi::c_int,
    mut above_row: ::core::ffi::c_int,
    mut below_row: ::core::ffi::c_int,
    mut pum_border_size: ::core::ffi::c_int,
) {
    unsafe {
        let mut context_lines: ::core::ffi::c_int = 0;
        pum_height.set(if size < 10 as ::core::ffi::c_int {
            size
        } else {
            10 as ::core::ffi::c_int
        });
        if p_ph.get() > 0 as OptInt && pum_height.get() as OptInt > p_ph.get() {
            pum_height.set(p_ph.get() as ::core::ffi::c_int);
        }
        if pum_win_row + 2 as ::core::ffi::c_int + pum_border_size >= below_row - pum_height.get()
            && pum_win_row - above_row > (below_row - above_row) / 2 as ::core::ffi::c_int
        {
            pum_above.set(true_0 != 0);
            if State.get() & MODE_CMDLINE != 0 && target_win.is_null() {
                context_lines = 0 as ::core::ffi::c_int;
            } else {
                context_lines = if (2 as ::core::ffi::c_int)
                    < (*target_win).w_wrow - (*target_win).w_cline_row
                {
                    2 as ::core::ffi::c_int
                } else {
                    (*target_win).w_wrow - (*target_win).w_cline_row
                };
            }
            if pum_win_row >= size + context_lines {
                pum_row.set(pum_win_row - size - context_lines);
                pum_height.set(size);
            } else {
                pum_row.set(0 as ::core::ffi::c_int);
                pum_height.set(pum_win_row - context_lines);
            }
            if p_ph.get() > 0 as OptInt && pum_height.get() as OptInt > p_ph.get() {
                (*pum_row.ptr()) += pum_height.get() - p_ph.get() as ::core::ffi::c_int;
                pum_height.set(p_ph.get() as ::core::ffi::c_int);
            }
            if pum_border_size > 0 as ::core::ffi::c_int
                && pum_border_size + pum_row.get() + pum_height.get() >= pum_win_row
            {
                if pum_row.get() < 2 as ::core::ffi::c_int {
                    (*pum_height.ptr()) -= pum_border_size;
                } else {
                    (*pum_row.ptr()) -= pum_border_size;
                }
            }
        } else {
            pum_above.set(false_0 != 0);
            if State.get() & MODE_CMDLINE != 0 && target_win.is_null() {
                context_lines = 0 as ::core::ffi::c_int;
            } else {
                validate_cheight(target_win);
                let mut cline_visible_offset: ::core::ffi::c_int =
                    (*target_win).w_cline_row + (*target_win).w_cline_height - (*target_win).w_wrow;
                context_lines = if (3 as ::core::ffi::c_int) < cline_visible_offset {
                    3 as ::core::ffi::c_int
                } else {
                    cline_visible_offset
                };
            }
            pum_row.set(pum_win_row + context_lines);
            pum_height.set(if below_row - pum_row.get() < size {
                below_row - pum_row.get()
            } else {
                size
            });
            if p_ph.get() > 0 as OptInt && pum_height.get() as OptInt > p_ph.get() {
                pum_height.set(p_ph.get() as ::core::ffi::c_int);
            }
            if pum_row.get() + pum_height.get() + pum_border_size >= cmdline_row.get() {
                (*pum_height.ptr()) -= pum_border_size;
            }
        }
        if above_row > 0 as ::core::ffi::c_int
            && pum_row.get() < above_row
            && pum_height.get() > above_row
        {
            pum_row.set(above_row);
            pum_height.set(pum_win_row - above_row);
        }
    }
}

pub(crate) unsafe extern "C" fn set_pum_width_aligned_with_cursor(
    mut width: ::core::ffi::c_int,
    mut available_width: ::core::ffi::c_int,
) -> bool {
    let mut end_padding: bool = true_0 != 0;
    if (width as OptInt) < p_pw.get() {
        width = p_pw.get() as ::core::ffi::c_int;
        end_padding = false_0 != 0;
    }
    if p_pmw.get() > 0 as OptInt && width as OptInt > p_pmw.get() {
        width = p_pmw.get() as ::core::ffi::c_int;
        end_padding = false_0 != 0;
    }
    pum_width.set(
        width
            + (if end_padding as ::core::ffi::c_int != 0 && width as OptInt >= p_pw.get() {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }),
    );
    return available_width >= pum_width.get();
}

pub(crate) unsafe extern "C" fn pum_compute_horizontal_placement(
    mut target_win: *mut win_T,
    mut cursor_col: ::core::ffi::c_int,
    mut border_width: ::core::ffi::c_int,
) {
    unsafe {
        let mut max_col: ::core::ffi::c_int = if Columns.get()
            > (if !target_win.is_null() {
                (*target_win).w_wincol + (*target_win).w_view_width
            } else {
                0 as ::core::ffi::c_int
            }) {
            Columns.get()
        } else if !target_win.is_null() {
            (*target_win).w_wincol + (*target_win).w_view_width
        } else {
            0 as ::core::ffi::c_int
        };
        let mut desired_width: ::core::ffi::c_int =
            pum_base_width.get() + pum_kind_width.get() + pum_extra_width.get();
        let mut available_width: ::core::ffi::c_int = 0;
        if pum_rl.get() {
            available_width =
                cursor_col - pum_scrollbar.get() + 1 as ::core::ffi::c_int - border_width;
        } else {
            available_width = max_col - cursor_col - pum_scrollbar.get() - border_width;
        }
        pum_col.set(cursor_col);
        if set_pum_width_aligned_with_cursor(desired_width, available_width) {
            return;
        }
        if available_width as OptInt > p_pw.get() {
            pum_width.set(available_width);
            return;
        }
        if pum_rl.get() {
            available_width = max_col - pum_scrollbar.get() - border_width;
        } else {
            available_width += cursor_col;
        }
        if available_width as OptInt > p_pw.get() {
            pum_width.set(p_pw.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int);
            if pum_rl.get() {
                pum_col.set(pum_width.get() + pum_scrollbar.get() + border_width);
            } else {
                pum_col.set(max_col - pum_width.get() - pum_scrollbar.get() - border_width);
            }
            return;
        }
        if pum_rl.get() {
            pum_col.set(max_col - 1 as ::core::ffi::c_int);
        } else {
            pum_col.set(0 as ::core::ffi::c_int);
        }
        pum_width.set(max_col - pum_scrollbar.get() - border_width);
    }
}

pub(crate) unsafe extern "C" fn pum_position_at_mouse(mut min_width: ::core::ffi::c_int) {
    unsafe {
        let mut min_row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut min_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut max_row: ::core::ffi::c_int = Rows.get();
        let mut max_col: ::core::ffi::c_int = Columns.get();
        let mut grid: ::core::ffi::c_int = mouse_grid.get();
        let mut row: ::core::ffi::c_int = mouse_row.get();
        let mut col: ::core::ffi::c_int = mouse_col.get();
        pum_win_row_offset.set(0 as ::core::ffi::c_int);
        pum_win_col_offset.set(0 as ::core::ffi::c_int);
        if ui_has(kUIMultigrid) as ::core::ffi::c_int != 0 && grid == 0 as ::core::ffi::c_int {
            mouse_find_win_outer(&raw mut grid, &raw mut row, &raw mut col);
        }
        if grid > 1 as ::core::ffi::c_int {
            let mut wp: *mut win_T = get_win_by_grid_handle(grid as handle_T);
            if !wp.is_null() {
                row += (*wp).w_winrow;
                col += (*wp).w_wincol;
                pum_win_row_offset.set((*wp).w_winrow);
                pum_win_col_offset.set((*wp).w_wincol);
                if (*wp).w_view_height > 0 as ::core::ffi::c_int
                    || (*wp).w_view_width > 0 as ::core::ffi::c_int
                {
                    max_row = if Rows.get() - (*wp).w_winrow > (*wp).w_winrow + (*wp).w_view_height
                    {
                        Rows.get() - (*wp).w_winrow
                    } else {
                        (*wp).w_winrow + (*wp).w_view_height
                    };
                    max_col =
                        if Columns.get() - (*wp).w_wincol > (*wp).w_wincol + (*wp).w_view_width {
                            Columns.get() - (*wp).w_wincol
                        } else {
                            (*wp).w_wincol + (*wp).w_view_width
                        };
                }
            }
        }
        if (*pum_grid.ptr()).handle != 0 as ::core::ffi::c_int && grid == (*pum_grid.ptr()).handle {
            row += pum_row.get();
            col += pum_left_col.get();
        } else {
            pum_anchor_grid.set(grid);
        }
        let mut border_width: ::core::ffi::c_int = pum_border_width();
        let mut border_height: ::core::ffi::c_int = border_width;
        if max_row - row > pum_size.get() + border_height || max_row - row > row - min_row {
            pum_above.set(false_0 != 0);
            pum_row.set(row + 1 as ::core::ffi::c_int);
            if pum_height.get() + border_height > max_row - pum_row.get() {
                pum_height.set(max_row - pum_row.get() - border_height);
            }
        } else {
            pum_above.set(true_0 != 0);
            pum_row.set(row - pum_size.get() - border_height);
            if pum_row.get() < min_row {
                (*pum_height.ptr()) += pum_row.get() - min_row;
                pum_row.set(min_row);
            }
        }
        if pum_rl.get() {
            if col - min_col + 1 as ::core::ffi::c_int >= pum_base_width.get() + border_width
                || col - min_col + 1 as ::core::ffi::c_int > min_width + border_width
            {
                pum_col.set(col);
            } else {
                pum_col.set(
                    min_col
                        + (if pum_base_width.get() + border_width < min_width + border_width {
                            pum_base_width.get() + border_width
                        } else {
                            min_width + border_width
                        })
                        - 1 as ::core::ffi::c_int,
                );
            }
            pum_width.set(pum_col.get() - min_col + 1 as ::core::ffi::c_int - border_width);
        } else {
            if max_col - col >= pum_base_width.get() + border_width
                || max_col - col > min_width + border_width
            {
                pum_col.set(col);
            } else {
                pum_col.set(
                    max_col
                        - (if pum_base_width.get() + border_width < min_width + border_width {
                            pum_base_width.get() + border_width
                        } else {
                            min_width + border_width
                        }),
                );
            }
            pum_width.set(max_col - pum_col.get() - border_width);
        }
        pum_width.set(
            if pum_width.get() < pum_base_width.get() + 1 as ::core::ffi::c_int {
                pum_width.get()
            } else {
                pum_base_width.get() + 1 as ::core::ffi::c_int
            },
        );
    }
}
