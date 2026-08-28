//! The lines between windows, and the glyphs where they meet.
//!
//! [`draw_vsep_win`] and [`draw_hsep_win`] draw the separator right of and below
//! one window. The interesting half is the corners: with the global statusline
//! (`'laststatus'` 3) a window boundary can be a T or a cross, and
//! [`draw_sep_connectors_win`] picks the right `'fillchars'` glyph for each of a
//! window's four corners by asking [`vsep_connected`] and [`hsep_connected`]
//! whether a neighbouring window's separator continues through it. Both walk the
//! frame tree to the neighbour at that row or column.
//!
//! [`win_redraw_signcols`] is here for a different reason: it is the one
//! per-window recomputation `win_update` does before deciding what to redraw,
//! and it answers whether the sign column changed width.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::decoration::SignCountHalf;
use crate::decoration::kMTMetaSignText;
use crate::grid::default_gridview;
use crate::winlayer::Win;

/// Which corner of a window a separator connector is being drawn in.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WindowCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl WindowCorner {
    /// Whether the corner is on the window's top edge rather than its bottom.
    fn is_top(self) -> bool {
        matches!(self, Self::TopLeft | Self::TopRight)
    }

    /// Whether the corner is on the window's left edge rather than its right.
    fn is_left(self) -> bool {
        matches!(self, Self::TopLeft | Self::BottomLeft)
    }
}

/// Recompute window `wp`'s sign-column width, and answer whether it changed.
///
/// A changed width means the whole window has to be redrawn: every line's
/// columns shift. `'statuscolumn'` is the second reason to answer true — the
/// expression can read the sign count, so a change to it invalidates the cached
/// width estimate even when the column itself did not move.
pub(crate) unsafe fn win_redraw_signcols(mut wp: Win) -> bool {
    // SAFETY: the caller's live window; its buffer is live with it.
    let mut buf = wp.buffer();

    // 'signcolumn' with a range, or a 'statuscolumn' that may ask for the
    // count, needs the per-line counts kept up to date from now on.
    if !buf.b_signcols.autom
        // SAFETY: the window's own 'statuscolumn' string, live with it.
        && (unsafe { *wp.w_onebuf_opt.wo_stc } != 0
            || (wp.w_maxscwidth > 1 && wp.w_minscwidth != wp.w_maxscwidth))
    {
        buf.b_signcols.autom = true;
        let last = buf.b_ml.ml_line_count - 1;
        // SAFETY: a live buffer, on the main thread.
        unsafe {
            buf_signcols_count_range(buf.raw(), 0, last, MAXLNUM as c_int, SignCountHalf::Both);
        }
    }

    // `b_signcols.max` is a high-water mark that nothing lowers as signs go
    // away, so trim the empty top buckets here.
    while buf.b_signcols.max > 0 && buf.b_signcols.count[(buf.b_signcols.max - 1) as usize] == 0 {
        buf.b_signcols.max -= 1;
    }

    let mut width = wp.w_maxscwidth.min(buf.b_signcols.max);
    // SAFETY: as above -- the window's own 'statuscolumn' string.
    let rebuild_stc =
        buf.b_signcols.max != buf.b_signcols.last_max && unsafe { *wp.w_onebuf_opt.wo_stc } != 0;

    if rebuild_stc {
        // Make `number_width` re-estimate the 'statuscolumn' width.
        wp.w_nrwidth_line_count = 0;
    } else if wp.w_minscwidth == 0 && wp.w_maxscwidth == 1 {
        // Plain `'signcolumn'` "auto": one column iff the buffer has any
        // sign text at all, which is cheaper than the per-line counts.
        width = c_int::from(buf_meta_total(buf, kMTMetaSignText) > 0);
    }

    let was = wp.w_scwidth;
    wp.w_scwidth = wp.w_minscwidth.max(0).max(width);
    wp.w_scwidth != was || rebuild_stc
}

/// Walk from `wp`'s frame to the neighbouring frame across the given corner.
///
/// `layout` is the parent layout that puts frames side by side in the direction
/// being crossed — `FR_ROW` for a horizontal separator's left/right neighbour,
/// `FR_COL` for a vertical separator's above/below one. `before` picks the
/// previous sibling rather than the next.
///
/// Answers `None` when the walk reaches the root without finding a sibling, i.e.
/// when there is no neighbour on that side.
///
/// # Safety
/// `wp` must be a live window of the current layout.
unsafe fn neighbour_frame(wp: Win, layout: c_int, before: bool) -> Option<*mut frame_T> {
    // SAFETY: walking the window layout tree on the main thread.
    let mut fr = wp.w_frame;
    while !unsafe { (*fr).fr_parent }.is_null() {
        let sibling = if before {
            unsafe { (*fr).fr_prev }
        } else {
            unsafe { (*fr).fr_next }
        };
        if unsafe { (*(*fr).fr_parent).fr_layout } as c_int == layout && !sibling.is_null() {
            return Some(sibling);
        }
        fr = unsafe { (*fr).fr_parent };
    }
    None
}

/// Whether window `wp`'s horizontal separator at `corner` is continued by the
/// horizontal separator of the window on the other side of it.
///
/// Assumes the global statusline is enabled — without it a horizontal boundary
/// is a status line, not a separator.
pub(crate) unsafe fn hsep_connected(wp: Win, corner: WindowCorner) -> bool {
    // SAFETY: walking the window layout tree on the main thread.
    let before = corner.is_left();
    let sep_row = if corner.is_top() {
        wp.w_winrow - 1
    } else {
        unsafe { win_endrow(wp.raw()) }
    };

    // SAFETY: walking the layout tree of the caller's live window.
    let neighbour = unsafe { neighbour_frame(wp, FR_ROW, before) };
    let Some(mut fr) = neighbour else {
        return false;
    };

    // Descend to the leaf of that neighbour that touches `sep_row`. Going
    // left, the frame that touches it is the LAST child of every row frame
    // on the way down; otherwise it is the first child whose bottom edge
    // reaches the row.
    while unsafe { (*fr).fr_layout } as c_int != FR_LEAF {
        fr = unsafe { (*fr).fr_child };
        if unsafe { (*(*fr).fr_parent).fr_layout } as c_int == FR_ROW && before {
            while !unsafe { (*fr).fr_next }.is_null() {
                fr = unsafe { (*fr).fr_next };
            }
        } else {
            while !unsafe { (*fr).fr_next }.is_null()
                && unsafe { (*frame2win(fr)).w_winrow } + unsafe { (*fr).fr_height } < sep_row
            {
                fr = unsafe { (*fr).fr_next };
            }
        }
    }

    let other = unsafe { (*fr).fr_win };
    sep_row == unsafe { (*other).w_winrow } - 1 || sep_row == unsafe { win_endrow(other) }
}

/// Whether window `wp`'s vertical separator at `corner` is continued by the
/// vertical separator of the window above or below it.
pub(crate) unsafe fn vsep_connected(wp: Win, corner: WindowCorner) -> bool {
    // SAFETY: walking the window layout tree on the main thread.
    // The mirror image of `hsep_connected`: "before" is up rather than
    // left, and the sibling direction is a column rather than a row.
    let before = corner.is_top();
    let sep_col = if corner.is_left() {
        wp.w_wincol - 1
    } else {
        unsafe { win_endcol(wp.raw()) }
    };

    // SAFETY: walking the layout tree of the caller's live window.
    let neighbour = unsafe { neighbour_frame(wp, FR_COL, before) };
    let Some(mut fr) = neighbour else {
        return false;
    };

    while unsafe { (*fr).fr_layout } as c_int != FR_LEAF {
        fr = unsafe { (*fr).fr_child };
        if unsafe { (*(*fr).fr_parent).fr_layout } as c_int == FR_COL && before {
            while !unsafe { (*fr).fr_next }.is_null() {
                fr = unsafe { (*fr).fr_next };
            }
        } else {
            while !unsafe { (*fr).fr_next }.is_null()
                && unsafe { (*frame2win(fr)).w_wincol } + unsafe { (*fr).fr_width } < sep_col
            {
                fr = unsafe { (*fr).fr_next };
            }
        }
    }

    let other = unsafe { (*fr).fr_win };
    sep_col == unsafe { (*other).w_wincol } - 1 || sep_col == unsafe { win_endcol(other) }
}

/// Draw the vertical separator right of window `wp`.
pub(crate) unsafe fn draw_vsep_win(wp: Win) {
    // SAFETY: a live window; the grid batch is opened and flushed per row.
    if wp.w_vsep_width == 0 {
        return;
    }
    let attr = unsafe { win_hl_attr(wp.raw(), HLF_C) };
    let col = unsafe { win_endcol(wp.raw()) };
    let end_row = unsafe { win_endrow(wp.raw()) };
    for row in (wp.w_winrow)..end_row {
        unsafe { grid_line_start(default_gridview(), row) };
        grid_line_put_schar(col, wp.w_p_fcs_chars.vert, attr);
        unsafe { grid_line_flush() };
    }
}

/// Draw the horizontal separator below window `wp`.
pub(crate) unsafe fn draw_hsep_win(wp: Win) {
    // SAFETY: a live window; the grid batch is opened and flushed here.
    if wp.w_hsep_height == 0 {
        return;
    }
    unsafe { grid_line_start(default_gridview(), win_endrow(wp.raw())) };
    grid_line_fill(
        wp.w_wincol,
        unsafe { win_endcol(wp.raw()) },
        wp.w_p_fcs_chars.horiz,
        unsafe { win_hl_attr(wp.raw(), HLF_C) },
    );
    unsafe { grid_line_flush() };
}

/// The `'fillchars'` glyph for window `wp`'s separators meeting at `corner`.
///
/// Two windows can be connected neither vertically nor horizontally, so if the
/// vertical separator does not continue through the corner the horizontal one
/// must — which is why the second half needs no test of its own.
unsafe fn get_corner_sep_connector(wp: Win, corner: WindowCorner) -> schar_T {
    // SAFETY: a live window of the current layout.
    let fcs = &wp.w_p_fcs_chars;
    if unsafe { vsep_connected(wp, corner) } {
        if unsafe { hsep_connected(wp, corner) } {
            fcs.verthoriz
        } else if corner.is_left() {
            fcs.vertright
        } else {
            fcs.vertleft
        }
    } else if corner.is_top() {
        fcs.horizdown
    } else {
        fcs.horizup
    }
}

/// Draw the connecting glyphs at window `wp`'s four corners.
///
/// Only with the global statusline: without it a horizontal window boundary is
/// a status line, which has no corners to connect. Corners on the edge of the
/// screen are skipped — there is nothing on the other side of them.
///
/// `update_screen` runs this for every window *after* all the window updates, so
/// that a connector is never overwritten by a neighbour's separator.
pub(crate) unsafe fn draw_sep_connectors_win(wp: Win) {
    // SAFETY: a live window of the current layout; each grid batch is opened
    // and flushed here.
    if global_stl_height() == 0 || !(wp.w_hsep_height == 1 || wp.w_vsep_width == 1) {
        return;
    }

    let hl = unsafe { win_hl_attr(wp.raw(), HLF_C) };

    // Which edges of the screen the window is on. Left and top are decided
    // by walking out to the root without finding a preceding sibling in the
    // relevant direction; right and bottom are simply "no separator there".
    let at_bottom = wp.w_hsep_height == 0;
    let at_right = wp.w_vsep_width == 0;
    let at_top = unsafe { neighbour_frame(wp, FR_COL, true) }.is_none();
    let at_left = unsafe { neighbour_frame(wp, FR_ROW, true) }.is_none();

    let top = wp.w_winrow - 1;
    let bottom = unsafe { win_endrow(wp.raw()) };
    let left = wp.w_wincol - 1;
    let right = unsafe { win_endcol(wp.raw()) };

    for (draw, row, col, corner) in [
        (!(at_top || at_left), top, left, WindowCorner::TopLeft),
        (!(at_top || at_right), top, right, WindowCorner::TopRight),
        (
            !(at_bottom || at_left),
            bottom,
            left,
            WindowCorner::BottomLeft,
        ),
        (
            !(at_bottom || at_right),
            bottom,
            right,
            WindowCorner::BottomRight,
        ),
    ] {
        if draw {
            unsafe { grid_line_start(default_gridview(), row) };
            grid_line_put_schar(col, unsafe { get_corner_sep_connector(wp, corner) }, hl);
            unsafe { grid_line_flush() };
        }
    }
}
