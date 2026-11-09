//! Setting a window's size explicitly -- `:resize`, and dragging a
//! separator.
//!
//! [`setheight_win`] and [`setwidth_win`] are the `:resize` /
//! `:vertical resize` entry points; [`FrameRef::set_height`] and
//! [`FrameRef::set_width`] are the recursive half, which takes the room from the
//! frames around the one being sized, respecting the minimum sizes and the
//! `'winfix*'` pins, and grows an ancestor when the siblings cannot pay.
//! [`win_drag_status_line`] and [`win_drag_vsep_line`] are the mouse forms,
//! and [`comp_positions`]/[`FrameRef::compute_position`] recompute every window's screen
//! position afterwards.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::narrow::number_as_int;
use core::ffi::c_int;

use super::arith::NextCurwin;
use super::*;
use crate::drawscreen::state::{cmdline_row, redraw_cmdline};
use crate::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, UPD_VALID, showmode};
use crate::option::defs::{OptError, OptSet};
use crate::option::vars::{P_WMH, P_WMW, p_ch, p_wmh, p_wmw};
use crate::types::{OptInt, kFloatRelativeWindow};
use crate::ui::state::{Columns, Rows};
use crate::winfloat::win_config_float;
use crate::winlayer::{FrameRef, Win, frames, frames_back, windows_back};

pub fn win_comp_pos() -> c_int {
    comp_positions()
}

/// Recompute every window's screen position from the frame tree, and answer
/// the row the command line starts on.
pub(crate) fn comp_positions() -> c_int {
    let mut row = tabline_rows();
    let mut col = 0;
    current_topframe().compute_position(&mut row, &mut col);
    // A float anchored to a window may have moved with it. Floats sit at the
    // end of the window list, so the walk back from `lastwin` stops at the
    // first window that is not one.
    for mut win in windows_back().take_while(|w| w.w_floating) {
        if win.w_config.relative == kFloatRelativeWindow {
            win.w_pos_changed = true;
        }
    }
    row + global_stl_rows()
}

impl FrameRef {
    /// Give this frame and everything in it its screen position, advancing
    /// `row` and `col` past it, from `frame_comp_pos()`.
    pub(crate) fn compute_position(self, row: &mut c_int, col: &mut c_int) {
        if let Some(mut wp) = self.win() {
            // Avoid an extra redraw when the position has not changed.
            if wp.w_winrow != *row || wp.w_wincol != *col {
                wp.w_winrow = *row;
                wp.w_wincol = *col;
                wp.redraw_later(UPD_NOT_VALID);
                wp.w_redr_status = true;
                wp.w_pos_changed = true;
            }
            let h = wp.w_height + wp.w_hsep_height + wp.w_status_height;
            *row += h.min(self.fr_height);
            *col += wp.w_width + wp.w_vsep_width;
        } else {
            let startrow = *row;
            let startcol = *col;
            for frp in self.children() {
                if c_int::from(self.fr_layout) == FR_ROW {
                    // All frames in a row start at the same row.
                    *row = startrow;
                } else {
                    // All frames in a column start at the same column.
                    *col = startcol;
                }
                frp.compute_position(row, col);
            }
        }
    }

    /// Give this frame height `height`, resizing everything around it to fit,
    /// from `frame_setheight()`.
    ///
    /// A frame in a row needs its siblings resized too, so the whole row is sized
    /// instead. A frame in a column takes the room from the frames above and below
    /// it, growing the containing frame — or borrowing from the command line — when
    /// they cannot pay.
    fn set_height(self, height: c_int) {
        // If the height already is the desired value, nothing to do.
        if self.fr_height == height {
            return;
        }
        let Some(parent) = self.parent() else {
            // topframe: can only change the command line height.
            if height > 0 {
                new_height(self, height, false, false, true);
            }
            return;
        };
        if c_int::from(parent.fr_layout) == FR_ROW {
            // A row of frames: the frames left and right of this one need resizing
            // too, so size the row and let it distribute. Check their minimum
            // height first.
            let h = parent.min_height(NextCurwin::Unset);
            parent.set_height(height.max(h));
            return;
        }

        // A column of frames: try to change only frames in this column.
        let mut height = height;
        let mut room = 0;
        let mut room_cmdline = 0;
        let mut room_reserved = 0;
        // Do this twice: first compute the room available and, if it is not
        // enough, resize the containing frame; then compute it again and adjust
        // the height to it. Try not to reduce a 'winfixheight' window.
        for run in 1..=2 {
            room = 0;
            room_reserved = 0;
            for frp in parent.children() {
                if frp != self && frp.win().is_some_and(|w| w.w_onebuf_opt.wo_wfh != 0) {
                    room_reserved += frp.fr_height;
                }
                room += frp.fr_height;
                if frp != self {
                    room -= frp.min_height(NextCurwin::Unset);
                }
            }
            room_cmdline = if self.fr_width != Columns.get() {
                0
            } else {
                let wp = lastwin_nofloating(None);
                let below = wp.w_winrow + wp.w_height + wp.w_hsep_height + wp.w_status_height;
                (Rows.get() - number_as_int(p_ch()) - global_stl_rows() - below).max(0)
            };

            if height <= room + room_cmdline {
                break;
            }
            if run == 2 || self.fr_width == Columns.get() {
                height = room + room_cmdline;
                break;
            }
            let target = arith::parent_target(
                height,
                parent.min_height(NextCurwin::NoWin),
                number_as_int(p_wmh()),
            );
            parent.set_height(target);
            // NOTREACHED
        }

        // The number of lines to take from other frames (can be negative).
        let mut take = height - self.fr_height;
        // Without enough room, reduce a 'winfixheight' window as well.
        if height > room + room_cmdline - room_reserved {
            room_reserved = room + room_cmdline - height;
        }
        // With only a 'winfixheight' window, making this one smaller means making
        // the other one taller.
        if take < 0 && room - self.fr_height <= room_reserved {
            room_reserved = 0;
        }
        if take > 0 && room_cmdline > 0 {
            // Use lines from the command line first.
            let mut top = current_topframe();
            room_cmdline = room_cmdline.min(take);
            take -= room_cmdline;
            top.fr_height += room_cmdline;
        }

        // Set the current frame to the new height.
        new_height(self, height, false, false, true);

        // First take lines from the frames after the current frame; if that is not
        // enough, take them from the frames above it.
        for run in 0..2 {
            let start = if run == 0 { self.next() } else { self.prev() };
            let walk: Box<dyn Iterator<Item = FrameRef>> = if run == 0 {
                Box::new(frames(start))
            } else {
                Box::new(frames_back(start))
            };
            for frp in walk {
                if take == 0 {
                    break;
                }
                let h = frp.min_height(NextCurwin::Unset);
                if room_reserved > 0 && frp.win().is_some_and(|w| w.w_onebuf_opt.wo_wfh != 0) {
                    if room_reserved >= frp.fr_height {
                        room_reserved -= frp.fr_height;
                    } else {
                        if frp.fr_height - room_reserved > take {
                            room_reserved = frp.fr_height - take;
                        }
                        take -= frp.fr_height - room_reserved;
                        new_height(frp, room_reserved, false, false, true);
                        room_reserved = 0;
                    }
                } else if frp.fr_height - take < h {
                    take -= frp.fr_height - h;
                    new_height(frp, h, false, false, true);
                } else {
                    new_height(frp, frp.fr_height - take, false, false, true);
                    take = 0;
                }
            }
        }
    }

    /// [`FrameRef::set_height`] with the axes exchanged, from `frame_setwidth()` --
    /// with no command line to borrow from, and a top frame whose width is the
    /// screen's and so cannot change at all.
    pub(crate) fn set_width(self, width: c_int) {
        if self.fr_width == width {
            return;
        }
        let Some(parent) = self.parent() else {
            // topframe: can't change width.
            return;
        };
        if c_int::from(parent.fr_layout) == FR_COL {
            let w = parent.min_width(NextCurwin::Unset);
            parent.set_width(width.max(w));
            return;
        }

        let mut width = width;
        let mut room = 0;
        let mut room_reserved = 0;
        for run in 1..=2 {
            room = 0;
            room_reserved = 0;
            for frp in parent.children() {
                if frp != self && frp.win().is_some_and(|w| w.w_onebuf_opt.wo_wfw != 0) {
                    room_reserved += frp.fr_width;
                }
                room += frp.fr_width;
                if frp != self {
                    room -= frp.min_width(NextCurwin::Unset);
                }
            }
            if width <= room {
                break;
            }
            let rows_avail = OptInt::from(Rows.get())
                - p_ch()
                - OptInt::from(tabline_rows())
                - OptInt::from(global_stl_rows());
            if run == 2 || OptInt::from(self.fr_height) >= rows_avail {
                width = room;
                break;
            }
            let target = arith::parent_target(
                width,
                parent.min_width(NextCurwin::NoWin),
                number_as_int(p_wmw()),
            );
            parent.set_width(target);
        }

        let mut take = width - self.fr_width;
        if width > room - room_reserved {
            room_reserved = room - width;
        }
        if take < 0 && room - self.fr_width < room_reserved {
            room_reserved = 0;
        }

        new_width(self, width, false, false);

        for run in 0..2 {
            let start = if run == 0 { self.next() } else { self.prev() };
            let walk: Box<dyn Iterator<Item = FrameRef>> = if run == 0 {
                Box::new(frames(start))
            } else {
                Box::new(frames_back(start))
            };
            for frp in walk {
                if take == 0 {
                    break;
                }
                let w = frp.min_width(NextCurwin::Unset);
                if room_reserved > 0 && frp.win().is_some_and(|w| w.w_onebuf_opt.wo_wfw != 0) {
                    if room_reserved >= frp.fr_width {
                        room_reserved -= frp.fr_width;
                    } else {
                        if frp.fr_width - room_reserved > take {
                            room_reserved = frp.fr_width - take;
                        }
                        take -= frp.fr_width - room_reserved;
                        new_width(frp, room_reserved, false, false);
                        room_reserved = 0;
                    }
                } else if frp.fr_width - take < w {
                    take -= frp.fr_width - w;
                    new_width(frp, w, false, false);
                } else {
                    new_width(frp, frp.fr_width - take, false, false);
                    take = 0;
                }
            }
        }
    }
}

pub fn win_setheight(height: c_int) {
    setheight_win(height, Win::current());
}

pub fn win_setheight_win(height: c_int, win: Win) {
    setheight_win(height, win);
}

/// Give `win` height `height`, moving the other windows around it to fit.
pub(crate) fn setheight_win(height: c_int, win: Win) {
    let mut win = win;
    // Always keep the current window at least one line high, even when
    // 'winminheight' is zero; two when it has a window bar.
    let floor = number_as_int(if win.is_current() {
        p_wmh().max(1)
    } else {
        p_wmh()
    }) + win.w_winbar_height;
    let height = height.max(floor);

    if win.w_floating {
        win.w_config.height = height.max(1);
        win_config_float(win, win.w_config.clone());
        win.redraw_later(UPD_VALID);
    } else {
        let chrome = arith::height_with_chrome(height, win.w_hsep_height, win.w_status_height);
        win.frame().set_height(chrome);
        // Recompute the window positions.
        comp_positions();
        fix_scroll(true);
        redraw_all(UPD_NOT_VALID);
        redraw_cmdline.set(true);
    }
}

pub fn win_setwidth(width: c_int) {
    setwidth_win(width, Win::current());
}

pub fn win_setwidth_win(width: c_int, window: Win) {
    setwidth_win(width, window);
}

/// Give `window` width `width`, moving the other windows around it to fit.
pub(crate) fn setwidth_win(width: c_int, window: Win) {
    let mut window = window;
    // Always keep the current window at least one column wide, even when
    // 'winminwidth' is zero.
    let width = if window.is_current() {
        width.max(number_as_int(p_wmw())).max(1)
    } else {
        width.max(0)
    };
    if window.w_floating {
        window.w_config.width = width;
        win_config_float(window, window.w_config.clone());
        window.redraw_later(UPD_NOT_VALID);
    } else {
        window
            .frame()
            .set_width(arith::width_with_chrome(width, window.w_vsep_width));
        // Recompute the window positions.
        comp_positions();
        redraw_all(UPD_NOT_VALID);
    }
}

pub fn did_set_winminheight(_args: &mut OptSet) -> Result<(), OptError> {
    let mut first = true;
    // Loop until there is a 'winminheight' that is possible.
    while p_wmh() > 0 as OptInt {
        let room = Rows.get() - number_as_int(p_ch());
        if room >= min_rows_all_tabpages() {
            break;
        }
        P_WMH.set(p_wmh() - 1);
        if first {
            err(WinError::NoRoom);
            first = false;
        }
    }
    Ok(())
}

pub fn did_set_winminwidth(_args: &mut OptSet) -> Result<(), OptError> {
    let mut first = true;
    while p_wmw() > 0 as OptInt {
        if Columns.get() >= current_topframe().min_width(NextCurwin::Unset) {
            break;
        }
        P_WMW.set(p_wmw() - 1);
        if first {
            err(WinError::NoRoom);
            first = false;
        }
    }
    Ok(())
}

pub fn win_drag_status_line(dragwin: Win, offset: c_int) {
    drag_status_line(dragwin, offset);
}

/// Move the status line below `dragwin` by `offset` rows, taking the room from
/// the frames below it (or above, for a negative offset).
fn drag_status_line(dragwin: Win, offset: c_int) {
    let top = current_topframe();
    let mut fr = dragwin.frame();
    let mut curfr = fr;
    if fr != top {
        // More than one window; when the parent frame is not a column of
        // frames, its parent should be.
        fr = fr.parent().expect("not the top frame");
        if c_int::from(fr.fr_layout) != FR_COL {
            curfr = fr;
            if fr != top {
                // Only a row of windows: may drag the status line.
                fr = fr.parent().expect("not the top frame");
            }
        }
    }
    // If this is the last frame in a column, resize the parent frame instead:
    // two levels up, to skip a row of frames.
    while curfr != top && curfr.next().is_none() {
        if fr != top {
            fr = fr.parent().expect("not the top frame");
        }
        curfr = fr;
        if fr != top {
            fr = fr.parent().expect("not the top frame");
        }
    }
    let up = offset < 0;
    let mut offset = offset;
    let room;
    // The frame that grows, which is nothing when the last status line is
    // dragged up.
    let grow;
    if up {
        offset = -offset;
        // Sum up the room of the current frame and the ones above it.
        if fr == curfr {
            // Only one window.
            room = fr.fr_height - fr.min_height(NextCurwin::Unset);
        } else {
            let mut sum = 0;
            fr = fr.child().expect("a row or column has a child");
            loop {
                sum += fr.fr_height - fr.min_height(NextCurwin::Unset);
                if fr == curfr {
                    break;
                }
                fr = fr.next().expect("curfr is among these children");
            }
            room = sum;
        }
        grow = curfr.next();
    } else {
        // Only dragging the last status line can reduce 'cmdheight'.
        let mut sum = Rows.get() - cmdline_row.get();
        if curfr.next().is_some() {
            sum -= number_as_int(p_ch()) + global_stl_rows();
        } else if min_set_ch.get() > 0 as OptInt {
            sum -= 1;
        }
        sum = sum.max(0);
        // Sum up the room of the frames below the current one.
        for frp in frames(curfr.next()) {
            sum += frp.fr_height - frp.min_height(NextCurwin::Unset);
        }
        room = sum;
        grow = Some(curfr);
    }

    // Without enough room, move as far as we can.
    offset = offset.min(room);
    if offset <= 0 {
        return;
    }
    if let Some(fr) = grow {
        new_height(fr, fr.fr_height + offset, up, false, true);
    }
    // Now make the other frames smaller.
    let mut next = if up { Some(curfr) } else { curfr.next() };
    while let Some(fr) = next.filter(|_| offset > 0) {
        let n = fr.min_height(NextCurwin::Unset);
        if fr.fr_height - offset <= n {
            offset -= fr.fr_height - n;
            new_height(fr, n, !up, false, true);
            next = if up { fr.prev() } else { fr.next() };
        } else {
            new_height(fr, fr.fr_height - offset, !up, false, true);
            break;
        }
    }
    comp_positions();
    fix_scroll(true);
    redraw_all(UPD_SOME_VALID);
    showmode();
}

pub fn win_drag_vsep_line(dragwin: Win, offset: c_int) {
    drag_vsep_line(dragwin, offset);
}

/// [`drag_status_line`] for a vertical separator, which has no command line to
/// borrow from and gives up when the layout has no row to work in.
///
/// The walk up the tree is upstream's own and is *not* the status line's: it
/// stops at the top frame rather than testing `curfr` against it, and it moves
/// `curfr` before `fr` rather than after.
fn drag_vsep_line(dragwin: Win, offset: c_int) {
    let top = current_topframe();
    let mut fr = dragwin.frame();
    if fr == top {
        // Only one window (cannot happen?).
        return;
    }
    let mut curfr = fr;
    fr = fr.parent().expect("not the top frame");
    // When the parent frame is not a row of frames, its parent should be.
    if c_int::from(fr.fr_layout) != FR_ROW {
        if fr == top {
            // Only a column of windows (cannot happen?).
            return;
        }
        curfr = fr;
        fr = fr.parent().expect("not the top frame");
    }
    // If this is the last frame in a row, resize a parent frame instead.
    while curfr.next().is_none() {
        if fr == top {
            break;
        }
        curfr = fr;
        fr = fr.parent().expect("not the top frame");
        if fr != top {
            curfr = fr;
            fr = fr.parent().expect("not the top frame");
        }
    }

    let left = offset < 0;
    let mut offset = offset;
    let mut room = 0;

    let grow = if left {
        offset = -offset;
        // Sum up the room of the current frame and the ones left of it.
        fr = fr.child().expect("a row has a child");
        loop {
            room += fr.fr_width - fr.min_width(NextCurwin::Unset);
            if fr == curfr {
                break;
            }
            fr = fr.next().expect("curfr is among these children");
        }
        curfr.next()
    } else {
        // Sum up the room of the frames right of the current one.
        for frp in frames(curfr.next()) {
            room += frp.fr_width - frp.min_width(NextCurwin::Unset);
        }
        Some(curfr)
    };

    // Without enough room, move as far as we can.
    offset = offset.min(room);
    if offset <= 0 {
        return;
    }
    // A safety check, which upstream says cannot happen.
    let Some(grow) = grow else {
        return;
    };
    new_width(grow, grow.fr_width + offset, left, false);
    let mut next = if left { Some(curfr) } else { curfr.next() };
    while let Some(fr) = next.filter(|_| offset > 0) {
        let n = fr.min_width(NextCurwin::Unset);
        if fr.fr_width - offset <= n {
            offset -= fr.fr_width - n;
            new_width(fr, n, !left, false);
            next = if left { fr.prev() } else { fr.next() };
        } else {
            new_width(fr, fr.fr_width - offset, !left, false);
            break;
        }
    }
    comp_positions();
    redraw_all(UPD_NOT_VALID);
}
