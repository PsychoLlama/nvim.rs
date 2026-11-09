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

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use super::*;
use crate::winlayer::Win;

/// How many items the menu shows before `'pumheight'` has a say.
const PUM_DEF_HEIGHT: c_int = 10;

/// Apply an `OptInt`-typed upper bound, which zero and below switch off.
///
/// `'pumheight'`, `'pumwidth'` and `'pummaxwidth'` are all spelled this way.
fn clamp_to_option(value: c_int, limit: OptInt) -> c_int {
    if limit > 0 && OptInt::from(value) > limit {
        // `limit` is below `value` here, so it fits.
        c_int::try_from(limit).unwrap_or(c_int::MAX)
    } else {
        value
    }
}

/// Measure the three item columns.
///
/// `PUM_BASE_WIDTH` is the widest `word`. The kind and extra columns each
/// get one cell more than their widest entry, for the space that separates
/// them from what precedes.
pub(crate) fn pum_compute_size() {
    let (mut base, mut kind, mut extra) = (0, 0, 0);
    for item in pum_items().iter() {
        base = base.max(strsize(&item.pum_text));
        if let Some(text) = &item.pum_kind {
            kind = kind.max(strsize(text) + 1);
        }
        if let Some(text) = &item.pum_extra {
            extra = extra.max(strsize(text) + 1);
        }
    }
    PUM_BASE_WIDTH.set(base);
    PUM_KIND_WIDTH.set(kind);
    PUM_EXTRA_WIDTH.set(extra);
}

/// Decide `PUM_ROW` and `PUM_HEIGHT`.
///
/// The menu goes below `pum_win_row` when there is room, and above it when
/// there is not and there is more space up there. A few lines of context are
/// left between the menu and the line being completed so that line stays
/// visible; a cmdline menu with no window behind it has none to leave.
///
/// `above_row`/`below_row` bound the area the menu may use (a preview window
/// moves them) and `pum_border_size` is the room `'pumborder'` needs.
///
/// `target_win` is absent only in cmdline mode, which is the one case that
/// never reads it.
pub(crate) fn pum_compute_vertical_placement(
    size: c_int,
    target_win: Option<Win>,
    pum_win_row: c_int,
    above_row: c_int,
    below_row: c_int,
    pum_border_size: c_int,
) {
    // `validate_cheight` is the only call out of here and reaches nothing
    // that reads the pum state, which is why the row and height can be
    // settled in locals first.
    let cmdline_pum = STATE.get() & MODE_CMDLINE != 0 && target_win.is_none();
    let mut height = clamp_to_option(size.min(PUM_DEF_HEIGHT), p_ph());
    let mut row;

    if pum_win_row + 2 + pum_border_size >= below_row - height
        && pum_win_row - above_row > (below_row - above_row) / 2
    {
        // Above "pum_win_row", leaving two lines of context if possible.
        PUM_ABOVE.set(true);
        let context_lines = match target_win.filter(|_| !cmdline_pum) {
            None => 0,
            Some(target) => 2.min(target.w_wrow - target.w_cline_row),
        };

        if pum_win_row >= size + context_lines {
            row = pum_win_row - size - context_lines;
            height = size;
        } else {
            row = 0;
            height = pum_win_row - context_lines;
        }
        if p_ph() > 0 && OptInt::from(height) > p_ph() {
            // Losing rows off the top keeps the bottom where it was.
            row += height - c_int::try_from(p_ph()).unwrap_or(c_int::MAX);
            height = c_int::try_from(p_ph()).unwrap_or(c_int::MAX);
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
        PUM_ABOVE.set(false);
        let context_lines = match target_win.filter(|_| !cmdline_pum) {
            None => 0,
            Some(target) => {
                validate_cheight(target);
                let cline_visible_offset =
                    target.w_cline_row + target.w_cline_height - target.w_wrow;
                3.min(cline_visible_offset)
            }
        };

        row = pum_win_row + context_lines;
        height = clamp_to_option((below_row - row).min(size), p_ph());
        if row + height + pum_border_size >= CMDLINE_ROW.get() {
            height -= pum_border_size;
        }
    }

    // A preview window above must not be drawn over.
    if above_row > 0 && row < above_row && height > above_row {
        row = above_row;
        height = pum_win_row - above_row;
    }
    PUM_ROW.set(row);
    PUM_HEIGHT.set(height);
}

/// Set `PUM_WIDTH` to `width` bounded by `'pumwidth'` and `'pummaxwidth'`.
///
/// Answers whether the result fits in `available_width`. The extra cell is
/// the padding after the last column, dropped as soon as either bound has had
/// to move the width.
fn set_pum_width_aligned_with_cursor(width: c_int, available_width: c_int) -> bool {
    let mut width = width;
    let mut end_padding = true;

    if OptInt::from(width) < p_pw() {
        width = c_int::try_from(p_pw()).unwrap_or(c_int::MAX);
        end_padding = false;
    }
    if p_pmw() > 0 && OptInt::from(width) > p_pmw() {
        width = c_int::try_from(p_pmw()).unwrap_or(c_int::MAX);
        end_padding = false;
    }

    PUM_WIDTH.set(width + c_int::from(end_padding && OptInt::from(width) >= p_pw()));
    available_width >= PUM_WIDTH.get()
}

/// Decide `PUM_COL` and `PUM_WIDTH`.
///
/// The menu is aligned with `CURSOR_COL` when the three columns fit there. If
/// they do not it is shown truncated, still aligned, as long as that leaves
/// at least `'pumwidth'` cells; failing that it is pushed against the far
/// edge of the screen, and failing that it takes whatever the screen has.
///
pub(crate) fn pum_compute_horizontal_placement(
    target_win: Option<Win>,
    cursor_col: c_int,
    border_width: c_int,
) {
    let win_end_col = target_win.map_or(0, |t| t.w_wincol + t.w_view_width);
    let max_col = COLUMNS.get().max(win_end_col);
    let desired_width = PUM_BASE_WIDTH.get() + PUM_KIND_WIDTH.get() + PUM_EXTRA_WIDTH.get();

    let mut available_width = if PUM_RL.get() {
        cursor_col - PUM_SCROLLBAR.get() + 1 - border_width
    } else {
        max_col - cursor_col - PUM_SCROLLBAR.get() - border_width
    };

    // Align the menu with "cursor_col".
    PUM_COL.set(cursor_col);
    if set_pum_width_aligned_with_cursor(desired_width, available_width) {
        return;
    }

    // Show it truncated, provided it is at least 'pumwidth' wide.
    if OptInt::from(available_width) > p_pw() {
        PUM_WIDTH.set(available_width);
        return;
    }

    // A truncated menu is no longer aligned with "cursor_col".
    if PUM_RL.get() {
        available_width = max_col - PUM_SCROLLBAR.get() - border_width;
    } else {
        available_width += cursor_col;
    }

    if OptInt::from(available_width) > p_pw() {
        PUM_WIDTH.set(c_int::try_from(p_pw()).unwrap_or(c_int::MAX - 1) + 1); // truncate beyond 'pumwidth'
        if PUM_RL.get() {
            PUM_COL.set(PUM_WIDTH.get() + PUM_SCROLLBAR.get() + border_width);
        } else {
            PUM_COL.set(max_col - PUM_WIDTH.get() - PUM_SCROLLBAR.get() - border_width);
        }
        return;
    }

    // Not enough room anywhere: use what there is.
    PUM_COL.set(if PUM_RL.get() { max_col - 1 } else { 0 });
    PUM_WIDTH.set(max_col - PUM_SCROLLBAR.get() - border_width);
}

/// Place the whole menu around the mouse, for `:popup`.
///
/// Unlike the completion menu this one does not scroll, so it is sized to
/// `PUM_SIZE` outright and only shrinks when a screen edge says so.
/// `min_width` is the width the caller wants even for narrow items.
pub(crate) fn pum_position_at_mouse(min_width: c_int) {
    let pum_handle = pum_grid_ref().handle;
    let (min_row, min_col) = (0, 0);
    let mut max_row = ROWS.get();
    let mut max_col = COLUMNS.get();
    let mut pos = MousePos::current();
    PUM_WIN_ROW_OFFSET.set(0);
    PUM_WIN_COL_OFFSET.set(0);

    if ui_has(UI_MULTIGRID) && pos.grid == 0 {
        find_win_outer(&mut pos);
    }
    let (grid, mut row, mut col) = (pos.grid, pos.row, pos.col);
    if grid > 1
        && let Some(wp) = get_win_by_grid_handle(Handle::new(grid))
    {
        row += wp.w_winrow;
        col += wp.w_wincol;
        PUM_WIN_ROW_OFFSET.set(wp.w_winrow);
        PUM_WIN_COL_OFFSET.set(wp.w_wincol);

        if wp.w_view_height > 0 || wp.w_view_width > 0 {
            // The user asked for a different grid size; let the menu
            // extend to it.
            let (winrow, wincol) = (wp.w_winrow, wp.w_wincol);
            max_row = (ROWS.get() - winrow).max(winrow + wp.w_view_height);
            max_col = (COLUMNS.get() - wincol).max(wincol + wp.w_view_width);
        }
    }
    if pum_handle != Handle::NONE && grid == pum_handle.get() {
        // Repositioning the menu by right-clicking on itself.
        row += PUM_ROW.get();
        col += PUM_LEFT_COL.get();
    } else {
        PUM_ANCHOR_GRID.set(grid);
    }

    // Width and height are both 1 for a shadow border, otherwise 2.
    let border_height = pum_border_width();
    let border_width = border_height;
    if max_row - row > PUM_SIZE.get() + border_height || max_row - row > row - min_row {
        // Room below the mouse row, or more room below than above.
        PUM_ABOVE.set(false);
        PUM_ROW.set(row + 1);
        if PUM_HEIGHT.get() + border_height > max_row - PUM_ROW.get() {
            PUM_HEIGHT.set(max_row - PUM_ROW.get() - border_height);
        }
    } else {
        // Above the mouse row, shorter if it does not fit.
        PUM_ABOVE.set(true);
        PUM_ROW.set(row - PUM_SIZE.get() - border_height);
        if PUM_ROW.get() < min_row {
            PUM_HEIGHT.set(PUM_HEIGHT.get() + PUM_ROW.get() - min_row);
            PUM_ROW.set(min_row);
        }
    }

    // The fallback width when the mouse column has no room for the items.
    let aligned = (PUM_BASE_WIDTH.get() + border_width).min(min_width + border_width);
    if PUM_RL.get() {
        let fits = col - min_col + 1 >= PUM_BASE_WIDTH.get() + border_width
            || col - min_col + 1 > min_width + border_width;
        PUM_COL.set(if fits { col } else { min_col + aligned - 1 });
        PUM_WIDTH.set(PUM_COL.get() - min_col + 1 - border_width);
    } else {
        let fits = max_col - col >= PUM_BASE_WIDTH.get() + border_width
            || max_col - col > min_width + border_width;
        PUM_COL.set(if fits { col } else { max_col - aligned });
        PUM_WIDTH.set(max_col - PUM_COL.get() - border_width);
    }

    PUM_WIDTH.set(PUM_WIDTH.get().min(PUM_BASE_WIDTH.get() + 1));
}
