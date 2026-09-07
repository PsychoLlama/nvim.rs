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
#![allow(unsafe_code)]

use super::*;
use crate::decoration::SignCountHalf;
use crate::decoration::kMTMetaSignText;
use crate::grid::default_gridview;
use crate::winlayer::FrameRef;
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

/// Recompute window `window`'s sign-column width, and answer whether it changed.
///
/// A changed width means the whole window has to be redrawn: every line's
/// columns shift. `'statuscolumn'` is the second reason to answer true — the
/// expression can read the sign count, so a change to it invalidates the cached
/// width estimate even when the column itself did not move.
pub(crate) unsafe fn win_redraw_signcols(mut window: Win) -> bool {
    // SAFETY: the caller's live window; its buffer is live with it.
    let mut buf = window.buffer();

    // 'signcolumn' with a range, or a 'statuscolumn' that may ask for the
    // count, needs the per-line counts kept up to date from now on.
    if !buf.b_signcols.autom
        // SAFETY: the window's own 'statuscolumn' string, live with it.
        && (unsafe { *window.w_onebuf_opt.wo_stc } != 0
            || (window.w_maxscwidth > 1 && window.w_minscwidth != window.w_maxscwidth))
    {
        buf.b_signcols.autom = true;
        let last = buf.b_ml.ml_line_count - 1;
        buf_signcols_count_range(buf, 0, last, MAXLNUM, SignCountHalf::Both);
    }

    // `b_signcols.max` is a high-water mark that nothing lowers as signs go
    // away, so trim the empty top buckets here.
    while buf.b_signcols.max > 0 && buf.b_signcols.count[(buf.b_signcols.max - 1) as usize] == 0 {
        buf.b_signcols.max -= 1;
    }

    let mut width = window.w_maxscwidth.min(buf.b_signcols.max);
    // SAFETY: as above -- the window's own 'statuscolumn' string.
    let rebuild_stc = buf.b_signcols.max != buf.b_signcols.last_max
        && unsafe { *window.w_onebuf_opt.wo_stc } != 0;

    if rebuild_stc {
        // Make `number_width` re-estimate the 'statuscolumn' width.
        window.w_nrwidth_line_count = 0;
    } else if window.w_minscwidth == 0 && window.w_maxscwidth == 1 {
        // Plain `'signcolumn'` "auto": one column iff the buffer has any
        // sign text at all, which is cheaper than the per-line counts.
        width = c_int::from(buf_meta_total(buf, kMTMetaSignText) > 0);
    }

    let was = window.w_scwidth;
    window.w_scwidth = window.w_minscwidth.max(0).max(width);
    window.w_scwidth != was || rebuild_stc
}

/// Walk from `window`'s frame to the neighbouring frame across the given corner.
///
/// `layout` is the parent layout that puts frames side by side in the direction
/// being crossed — `FR_ROW` for a horizontal separator's left/right neighbour,
/// `FR_COL` for a vertical separator's above/below one. `before` picks the
/// previous sibling rather than the next.
///
/// Answers `None` when the walk reaches the root without finding a sibling, i.e.
/// when there is no neighbour on that side.
fn neighbour_frame(window: Win, layout: c_int, before: bool) -> Option<FrameRef> {
    let mut fr = window.frame();
    while let Some(parent) = fr.parent() {
        let sibling = if before { fr.prev() } else { fr.next() };
        if c_int::from(parent.fr_layout) == layout
            && let Some(sibling) = sibling
        {
            return Some(sibling);
        }
        fr = parent;
    }
    None
}

/// Whether window `window`'s horizontal separator at `corner` is continued by the
/// horizontal separator of the window on the other side of it.
///
/// Assumes the global statusline is enabled — without it a horizontal boundary
/// is a status line, not a separator.
pub(crate) fn hsep_connected(window: Win, corner: WindowCorner) -> bool {
    let before = corner.is_left();
    let sep_row = if corner.is_top() {
        window.w_winrow - 1
    } else {
        win_endrow(window)
    };

    let Some(mut fr) = neighbour_frame(window, FR_ROW, before) else {
        return false;
    };

    // Descend to the leaf of that neighbour that touches `sep_row`. Going
    // left, the frame that touches it is the LAST child of every row frame
    // on the way down; otherwise it is the first child whose bottom edge
    // reaches the row.
    while c_int::from(fr.fr_layout) != FR_LEAF {
        fr = fr.child().expect("a frame that is not a leaf has a child");
        let rowwise = fr
            .parent()
            .is_some_and(|parent| c_int::from(parent.fr_layout) == FR_ROW);
        while let Some(next) = fr.next() {
            if !(rowwise && before) && frame2window(fr).w_winrow + fr.fr_height >= sep_row {
                break;
            }
            fr = next;
        }
    }

    let other = fr.win().expect("a leaf frame holds a window");
    sep_row == other.w_winrow - 1 || sep_row == win_endrow(other)
}

/// Whether window `window`'s vertical separator at `corner` is continued by the
/// vertical separator of the window above or below it.
pub(crate) fn vsep_connected(window: Win, corner: WindowCorner) -> bool {
    let before = corner.is_top();
    let sep_col = if corner.is_left() {
        window.w_wincol - 1
    } else {
        win_endcol(window)
    };

    let Some(mut fr) = neighbour_frame(window, FR_COL, before) else {
        return false;
    };

    while c_int::from(fr.fr_layout) != FR_LEAF {
        fr = fr.child().expect("a frame that is not a leaf has a child");
        let colwise = fr
            .parent()
            .is_some_and(|parent| c_int::from(parent.fr_layout) == FR_COL);
        while let Some(next) = fr.next() {
            if !(colwise && before) && frame2window(fr).w_wincol + fr.fr_width >= sep_col {
                break;
            }
            fr = next;
        }
    }

    let other = fr.win().expect("a leaf frame holds a window");
    sep_col == other.w_wincol - 1 || sep_col == win_endcol(other)
}

/// Draw the vertical separator right of window `window`.
pub(crate) unsafe fn draw_vsep_win(window: Win) {
    // SAFETY: a live window; the grid batch is opened and flushed per row.
    if window.w_vsep_width == 0 {
        return;
    }
    let attr = unsafe { win_hl_attr(window, HLF_C) };
    let col = win_endcol(window);
    let end_row = win_endrow(window);
    for row in (window.w_winrow)..end_row {
        unsafe { grid_line_start(default_gridview(), row) };
        grid_line_put_schar(col, window.w_p_fcs_chars.vert, attr);
        unsafe { grid_line_flush() };
    }
}

/// Draw the horizontal separator below window `window`.
pub(crate) unsafe fn draw_hsep_win(window: Win) {
    // SAFETY: a live window; the grid batch is opened and flushed here.
    if window.w_hsep_height == 0 {
        return;
    }
    unsafe { grid_line_start(default_gridview(), win_endrow(window)) };
    grid_line_fill(
        window.w_wincol,
        win_endcol(window),
        window.w_p_fcs_chars.horiz,
        unsafe { win_hl_attr(window, HLF_C) },
    );
    unsafe { grid_line_flush() };
}

/// The `'fillchars'` glyph for window `window`'s separators meeting at `corner`.
///
/// Two windows can be connected neither vertically nor horizontally, so if the
/// vertical separator does not continue through the corner the horizontal one
/// must — which is why the second half needs no test of its own.
fn get_corner_sep_connector(window: Win, corner: WindowCorner) -> ScreenChar {
    let fcs = &window.w_p_fcs_chars;
    if vsep_connected(window, corner) {
        if hsep_connected(window, corner) {
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

/// Draw the connecting glyphs at window `window`'s four corners.
///
/// Only with the global statusline: without it a horizontal window boundary is
/// a status line, which has no corners to connect. Corners on the edge of the
/// screen are skipped — there is nothing on the other side of them.
///
/// `update_screen` runs this for every window *after* all the window updates, so
/// that a connector is never overwritten by a neighbour's separator.
pub(crate) unsafe fn draw_sep_connectors_win(window: Win) {
    // SAFETY: a live window of the current layout; each grid batch is opened
    // and flushed here.
    if global_stl_height() == 0 || !(window.w_hsep_height == 1 || window.w_vsep_width == 1) {
        return;
    }

    let hl = unsafe { win_hl_attr(window, HLF_C) };

    // Which edges of the screen the window is on. Left and top are decided
    // by walking out to the root without finding a preceding sibling in the
    // relevant direction; right and bottom are simply "no separator there".
    let at_bottom = window.w_hsep_height == 0;
    let at_right = window.w_vsep_width == 0;
    let at_top = neighbour_frame(window, FR_COL, true).is_none();
    let at_left = neighbour_frame(window, FR_ROW, true).is_none();

    let top = window.w_winrow - 1;
    let bottom = win_endrow(window);
    let left = window.w_wincol - 1;
    let right = win_endcol(window);

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
            grid_line_put_schar(col, get_corner_sep_connector(window, corner), hl);
            unsafe { grid_line_flush() };
        }
    }
}
