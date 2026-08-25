//! Where the popup menu goes and how big it is.
//!
//! The widths come from the items ([`pum_compute_size`]); the row and
//! height from the space above and below the anchor
//! ([`pum_compute_vertical_placement`]); the column and width from the
//! cursor column and what is left of the screen
//! ([`pum_compute_horizontal_placement`]). [`pum_position_at_mouse`] is
//! the `:popup` variant, anchored on the mouse instead of the cursor.
//!
//! Everything here writes the state cells in the parent rather than
//! answering a value: `pum_display` computes the placement in three steps
//! and the later ones read what the earlier ones decided.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

/// How many items the menu shows before `'pumheight'` has a say.
const PUM_DEF_HEIGHT: c_int = 10;

/// Apply an `OptInt`-typed upper bound, which zero and below switch off.
///
/// `'pumheight'`, `'pumwidth'` and `'pummaxwidth'` are all spelled this way.
fn clamp_to_option(value: c_int, limit: OptInt) -> c_int {
    if limit > 0 && OptInt::from(value) > limit {
        limit as c_int
    } else {
        value
    }
}

/// Measure the three item columns.
///
/// `pum_base_width` is the widest `word`. The kind and extra columns each
/// get one cell more than their widest entry, for the space that separates
/// them from what precedes.
///
/// # Safety
/// The item array must be the live one — `pum_display` and
/// `pum_show_popupmenu` both set it before calling.
pub(crate) unsafe fn pum_compute_size() {
    // SAFETY: the item strings belong to the caller of `pum_display` and stay
    // valid until `pum_undisplay`.
    unsafe {
        let (mut base, mut kind, mut extra) = (0, 0, 0);
        for item in pum_items() {
            if !item.pum_text.is_null() {
                base = base.max(vim_strsize(item.pum_text));
            }
            if !item.pum_kind.is_null() {
                kind = kind.max(vim_strsize(item.pum_kind) + 1);
            }
            if !item.pum_extra.is_null() {
                extra = extra.max(vim_strsize(item.pum_extra) + 1);
            }
        }
        pum_base_width.set(base);
        pum_kind_width.set(kind);
        pum_extra_width.set(extra);
    }
}

/// Decide `pum_row` and `pum_height`.
///
/// The menu goes below `pum_win_row` when there is room, and above it when
/// there is not and there is more space up there. A few lines of context are
/// left between the menu and the line being completed so that line stays
/// visible; a cmdline menu with no window behind it has none to leave.
///
/// `above_row`/`below_row` bound the area the menu may use (a preview window
/// moves them) and `pum_border_size` is the room `'pumborder'` needs.
///
/// # Safety
/// `target_win` must be live, or null — and null is only reached in cmdline
/// mode, which is the one case that never dereferences it.
pub(crate) unsafe fn pum_compute_vertical_placement(
    size: c_int,
    target_win: *mut win_T,
    pum_win_row: c_int,
    above_row: c_int,
    below_row: c_int,
    pum_border_size: c_int,
) {
    // SAFETY: `target_win` is live wherever it is read. `validate_cheight` is
    // the only call out of here and reaches nothing that reads the pum state,
    // which is why the row and height can be settled in locals first.
    unsafe {
        let cmdline_pum = State.get() & MODE_CMDLINE != 0 && target_win.is_null();
        let mut height = clamp_to_option(size.min(PUM_DEF_HEIGHT), p_ph.get());
        let mut row;

        if pum_win_row + 2 + pum_border_size >= below_row - height
            && pum_win_row - above_row > (below_row - above_row) / 2
        {
            // Above "pum_win_row", leaving two lines of context if possible.
            pum_above.set(true);
            let context_lines = if cmdline_pum {
                0
            } else {
                2.min((*target_win).w_wrow - (*target_win).w_cline_row)
            };

            if pum_win_row >= size + context_lines {
                row = pum_win_row - size - context_lines;
                height = size;
            } else {
                row = 0;
                height = pum_win_row - context_lines;
            }
            if p_ph.get() > 0 && OptInt::from(height) > p_ph.get() {
                // Losing rows off the top keeps the bottom where it was.
                row += height - p_ph.get() as c_int;
                height = p_ph.get() as c_int;
            }

            if pum_border_size > 0 && pum_border_size + row + height >= pum_win_row {
                if row < 2 {
                    height -= pum_border_size;
                } else {
                    row -= pum_border_size;
                }
            }
        } else {
            // Below "pum_win_row", leaving three lines of context if possible.
            pum_above.set(false);
            let context_lines = if cmdline_pum {
                0
            } else {
                validate_cheight(target_win);
                let cline_visible_offset =
                    (*target_win).w_cline_row + (*target_win).w_cline_height - (*target_win).w_wrow;
                3.min(cline_visible_offset)
            };

            row = pum_win_row + context_lines;
            height = clamp_to_option((below_row - row).min(size), p_ph.get());
            if row + height + pum_border_size >= cmdline_row.get() {
                height -= pum_border_size;
            }
        }

        // A preview window above must not be drawn over.
        if above_row > 0 && row < above_row && height > above_row {
            row = above_row;
            height = pum_win_row - above_row;
        }
        pum_row.set(row);
        pum_height.set(height);
    }
}

/// Set `pum_width` to `width` bounded by `'pumwidth'` and `'pummaxwidth'`.
///
/// Answers whether the result fits in `available_width`. The extra cell is
/// the padding after the last column, dropped as soon as either bound has had
/// to move the width.
fn set_pum_width_aligned_with_cursor(width: c_int, available_width: c_int) -> bool {
    let mut width = width;
    let mut end_padding = true;

    if OptInt::from(width) < p_pw.get() {
        width = p_pw.get() as c_int;
        end_padding = false;
    }
    if p_pmw.get() > 0 && OptInt::from(width) > p_pmw.get() {
        width = p_pmw.get() as c_int;
        end_padding = false;
    }

    pum_width.set(width + c_int::from(end_padding && OptInt::from(width) >= p_pw.get()));
    available_width >= pum_width.get()
}

/// Decide `pum_col` and `pum_width`.
///
/// The menu is aligned with `cursor_col` when the three columns fit there. If
/// they do not it is shown truncated, still aligned, as long as that leaves
/// at least `'pumwidth'` cells; failing that it is pushed against the far
/// edge of the screen, and failing that it takes whatever the screen has.
///
/// # Safety
/// `target_win` must be live, or null.
pub(crate) unsafe fn pum_compute_horizontal_placement(
    target_win: *mut win_T,
    cursor_col: c_int,
    border_width: c_int,
) {
    // SAFETY: `target_win` is live when it is not null.
    unsafe {
        let win_end_col = if target_win.is_null() {
            0
        } else {
            (*target_win).w_wincol + (*target_win).w_view_width
        };
        let max_col = Columns.get().max(win_end_col);
        let desired_width = pum_base_width.get() + pum_kind_width.get() + pum_extra_width.get();

        let mut available_width = if pum_rl.get() {
            cursor_col - pum_scrollbar.get() + 1 - border_width
        } else {
            max_col - cursor_col - pum_scrollbar.get() - border_width
        };

        // Align the menu with "cursor_col".
        pum_col.set(cursor_col);
        if set_pum_width_aligned_with_cursor(desired_width, available_width) {
            return;
        }

        // Show it truncated, provided it is at least 'pumwidth' wide.
        if OptInt::from(available_width) > p_pw.get() {
            pum_width.set(available_width);
            return;
        }

        // A truncated menu is no longer aligned with "cursor_col".
        if pum_rl.get() {
            available_width = max_col - pum_scrollbar.get() - border_width;
        } else {
            available_width += cursor_col;
        }

        if OptInt::from(available_width) > p_pw.get() {
            pum_width.set(p_pw.get() as c_int + 1); // truncate beyond 'pumwidth'
            if pum_rl.get() {
                pum_col.set(pum_width.get() + pum_scrollbar.get() + border_width);
            } else {
                pum_col.set(max_col - pum_width.get() - pum_scrollbar.get() - border_width);
            }
            return;
        }

        // Not enough room anywhere: use what there is.
        pum_col.set(if pum_rl.get() { max_col - 1 } else { 0 });
        pum_width.set(max_col - pum_scrollbar.get() - border_width);
    }
}

/// Place the whole menu around the mouse, for `:popup`.
///
/// Unlike the completion menu this one does not scroll, so it is sized to
/// `pum_size` outright and only shrinks when a screen edge says so.
/// `min_width` is the width the caller wants even for narrow items.
///
/// # Safety
/// `pum_size`, `pum_height` and `pum_base_width` must already describe the
/// menu being shown.
pub(crate) unsafe fn pum_position_at_mouse(min_width: c_int) {
    let pum_handle = pum_grid_ref().handle;
    // SAFETY: `get_win_by_grid_handle` answers a live window or null.
    unsafe {
        let (min_row, min_col) = (0, 0);
        let mut max_row = Rows.get();
        let mut max_col = Columns.get();
        let mut pos = MousePos::current();
        pum_win_row_offset.set(0);
        pum_win_col_offset.set(0);

        if ui_has(kUIMultigrid) && pos.grid == 0 {
            find_win_outer(&mut pos);
        }
        let (grid, mut row, mut col) = (pos.grid, pos.row, pos.col);
        if grid > 1 {
            let wp = get_win_by_grid_handle(grid as handle_T);
            if !wp.is_null() {
                row += (*wp).w_winrow;
                col += (*wp).w_wincol;
                pum_win_row_offset.set((*wp).w_winrow);
                pum_win_col_offset.set((*wp).w_wincol);

                if (*wp).w_view_height > 0 || (*wp).w_view_width > 0 {
                    // The user asked for a different grid size; let the menu
                    // extend to it.
                    let (winrow, wincol) = ((*wp).w_winrow, (*wp).w_wincol);
                    max_row = (Rows.get() - winrow).max(winrow + (*wp).w_view_height);
                    max_col = (Columns.get() - wincol).max(wincol + (*wp).w_view_width);
                }
            }
        }
        if pum_handle != 0 && grid == pum_handle {
            // Repositioning the menu by right-clicking on itself.
            row += pum_row.get();
            col += pum_left_col.get();
        } else {
            pum_anchor_grid.set(grid);
        }

        // Width and height are both 1 for a shadow border, otherwise 2.
        let border_height = pum_border_width();
        let border_width = border_height;
        if max_row - row > pum_size.get() + border_height || max_row - row > row - min_row {
            // Room below the mouse row, or more room below than above.
            pum_above.set(false);
            pum_row.set(row + 1);
            if pum_height.get() + border_height > max_row - pum_row.get() {
                pum_height.set(max_row - pum_row.get() - border_height);
            }
        } else {
            // Above the mouse row, shorter if it does not fit.
            pum_above.set(true);
            pum_row.set(row - pum_size.get() - border_height);
            if pum_row.get() < min_row {
                pum_height.set(pum_height.get() + pum_row.get() - min_row);
                pum_row.set(min_row);
            }
        }

        // The fallback width when the mouse column has no room for the items.
        let aligned = (pum_base_width.get() + border_width).min(min_width + border_width);
        if pum_rl.get() {
            let fits = col - min_col + 1 >= pum_base_width.get() + border_width
                || col - min_col + 1 > min_width + border_width;
            pum_col.set(if fits { col } else { min_col + aligned - 1 });
            pum_width.set(pum_col.get() - min_col + 1 - border_width);
        } else {
            let fits = max_col - col >= pum_base_width.get() + border_width
                || max_col - col > min_width + border_width;
            pum_col.set(if fits { col } else { max_col - aligned });
            pum_width.set(max_col - pum_col.get() - border_width);
        }

        pum_width.set(pum_width.get().min(pum_base_width.get() + 1));
    }
}
