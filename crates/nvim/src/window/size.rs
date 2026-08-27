//! Setting a window's size explicitly -- `:resize`, and dragging a
//! separator.
//!
//! [`setheight_win`] and [`setwidth_win`] are the `:resize` /
//! `:vertical resize` entry points; [`set_frame_height`] and
//! [`set_frame_width`] are the recursive half, which takes the room from the
//! frames around the one being sized, respecting the minimum sizes and the
//! `'winfix*'` pins, and grows an ancestor when the siblings cannot pay.
//! [`win_drag_status_line`] and [`win_drag_vsep_line`] are the mouse forms,
//! and [`comp_positions`]/[`comp_pos`] recompute every window's screen
//! position afterwards.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::arith::NextCurwin;
use super::*;
use crate::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, UPD_VALID, showmode};
use crate::main::{
    Columns, Rows, cmdline_row, e_noroom, lastwin, p_ch, p_wmh, p_wmw, redraw_cmdline,
};
use crate::types::{OptInt, kFloatRelativeWindow, optset_T, win_T};
use crate::winfloat::win_config_float;
use crate::winlayer::{Frame, Win, frames, frames_back};

pub fn win_comp_pos() -> c_int {
    comp_positions()
}

/// Recompute every window's screen position from the frame tree, and answer
/// the row the command line starts on.
pub(crate) fn comp_positions() -> c_int {
    let mut row = tabline_rows();
    let mut col = 0;
    comp_pos(current_topframe(), &mut row, &mut col);
    // A float anchored to a window may have moved with it.
    // SAFETY: `lastwin` heads a live window list ending at a null `w_prev`.
    let mut wp = unsafe { Win::from_raw(lastwin.get()) };
    while let Some(mut win) = wp.filter(|w| w.w_floating) {
        if win.w_config.relative == kFloatRelativeWindow {
            win.w_pos_changed = true;
        }
        // SAFETY: a live window's `w_prev` is a live window or null.
        wp = unsafe { Win::from_raw(win.w_prev) };
    }
    row + global_stl_rows()
}

/// Give frame `topfrp` and everything in it its screen position, advancing
/// `row` and `col` past it, from `frame_comp_pos()`.
pub(crate) fn comp_pos(topfrp: Frame, row: &mut c_int, col: &mut c_int) {
    if let Some(mut wp) = topfrp.win() {
        // Avoid an extra redraw when the position has not changed.
        if wp.w_winrow != *row || wp.w_wincol != *col {
            wp.w_winrow = *row;
            wp.w_wincol = *col;
            wp.redraw_later(UPD_NOT_VALID);
            wp.w_redr_status = true;
            wp.w_pos_changed = true;
        }
        let h = wp.w_height + wp.w_hsep_height + wp.w_status_height;
        *row += h.min(topfrp.fr_height);
        *col += wp.w_width + wp.w_vsep_width;
    } else {
        let startrow = *row;
        let startcol = *col;
        for frp in topfrp.children() {
            if topfrp.fr_layout as c_int == FR_ROW {
                // All frames in a row start at the same row.
                *row = startrow;
            } else {
                // All frames in a column start at the same column.
                *col = startcol;
            }
            comp_pos(frp, row, col);
        }
    }
}

pub fn win_setheight(height: c_int) {
    setheight_win(height, cur_win());
}

pub unsafe fn win_setheight_win(height: c_int, win: *mut win_T) {
    // SAFETY: the caller's promise -- a live window.
    setheight_win(height, unsafe { Win::new(win) });
}

/// Give `win` height `height`, moving the other windows around it to fit.
pub(crate) fn setheight_win(height: c_int, win: Win) {
    let mut win = win;
    // Always keep the current window at least one line high, even when
    // 'winminheight' is zero; two when it has a window bar.
    let floor = if win.is_current() {
        p_wmh.get().max(1)
    } else {
        p_wmh.get()
    } as c_int
        + win.w_winbar_height;
    let height = height.max(floor);

    if win.w_floating {
        win.w_config.height = height.max(1);
        // SAFETY: a live window.
        win_config_float(win, win.w_config.clone());
        win.redraw_later(UPD_VALID);
    } else {
        let chrome = arith::height_with_chrome(height, win.w_hsep_height, win.w_status_height);
        set_frame_height(win.frame(), chrome);
        // Recompute the window positions.
        comp_positions();
        fix_scroll(true);
        redraw_all(UPD_NOT_VALID);
        redraw_cmdline.set(true);
    }
}

/// Give frame `curfrp` height `height`, resizing everything around it to fit,
/// from `frame_setheight()`.
///
/// A frame in a row needs its siblings resized too, so the whole row is sized
/// instead. A frame in a column takes the room from the frames above and below
/// it, growing the containing frame — or borrowing from the command line — when
/// they cannot pay.
fn set_frame_height(curfrp: Frame, height: c_int) {
    // If the height already is the desired value, nothing to do.
    if curfrp.fr_height == height {
        return;
    }
    let Some(parent) = curfrp.parent() else {
        // topframe: can only change the command line height.
        if height > 0 {
            new_height(curfrp, height, false, false, true);
        }
        return;
    };
    if parent.fr_layout as c_int == FR_ROW {
        // A row of frames: the frames left and right of this one need resizing
        // too, so size the row and let it distribute. Check their minimum
        // height first.
        let h = minheight(parent, NextCurwin::Unset);
        set_frame_height(parent, height.max(h));
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
            if frp != curfrp && frp.win().is_some_and(|w| w.w_onebuf_opt.wo_wfh != 0) {
                room_reserved += frp.fr_height;
            }
            room += frp.fr_height;
            if frp != curfrp {
                room -= minheight(frp, NextCurwin::Unset);
            }
        }
        room_cmdline = if curfrp.fr_width != Columns.get() {
            0
        } else {
            let wp = last_nonfloating(None);
            let below = wp.w_winrow + wp.w_height + wp.w_hsep_height + wp.w_status_height;
            (Rows.get() - p_ch.get() as c_int - global_stl_rows() - below).max(0)
        };

        if height <= room + room_cmdline {
            break;
        }
        if run == 2 || curfrp.fr_width == Columns.get() {
            height = room + room_cmdline;
            break;
        }
        let target = arith::parent_target(
            height,
            minheight(parent, NextCurwin::NoWin),
            p_wmh.get() as c_int,
        );
        set_frame_height(parent, target);
        // NOTREACHED
    }

    // The number of lines to take from other frames (can be negative).
    let mut take = height - curfrp.fr_height;
    // Without enough room, reduce a 'winfixheight' window as well.
    if height > room + room_cmdline - room_reserved {
        room_reserved = room + room_cmdline - height;
    }
    // With only a 'winfixheight' window, making this one smaller means making
    // the other one taller.
    if take < 0 && room - curfrp.fr_height <= room_reserved {
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
    new_height(curfrp, height, false, false, true);

    // First take lines from the frames after the current frame; if that is not
    // enough, take them from the frames above it.
    for run in 0..2 {
        let start = if run == 0 {
            curfrp.next()
        } else {
            curfrp.prev()
        };
        let walk: Box<dyn Iterator<Item = Frame>> = if run == 0 {
            Box::new(frames(start))
        } else {
            Box::new(frames_back(start))
        };
        for frp in walk {
            if take == 0 {
                break;
            }
            let h = minheight(frp, NextCurwin::Unset);
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

pub fn win_setwidth(width: c_int) {
    setwidth_win(width, cur_win());
}

pub unsafe fn win_setwidth_win(width: c_int, wp: *mut win_T) {
    // SAFETY: the caller's promise -- a live window.
    setwidth_win(width, unsafe { Win::new(wp) });
}

/// Give `wp` width `width`, moving the other windows around it to fit.
pub(crate) fn setwidth_win(width: c_int, wp: Win) {
    let mut wp = wp;
    // Always keep the current window at least one column wide, even when
    // 'winminwidth' is zero.
    let width = if wp.is_current() {
        width.max(p_wmw.get() as c_int).max(1)
    } else {
        width.max(0)
    };
    if wp.w_floating {
        wp.w_config.width = width;
        // SAFETY: a live window.
        win_config_float(wp, wp.w_config.clone());
        wp.redraw_later(UPD_NOT_VALID);
    } else {
        set_frame_width(wp.frame(), arith::width_with_chrome(width, wp.w_vsep_width));
        // Recompute the window positions.
        comp_positions();
        redraw_all(UPD_NOT_VALID);
    }
}

/// [`set_frame_height`] with the axes exchanged, from `frame_setwidth()` --
/// with no command line to borrow from, and a top frame whose width is the
/// screen's and so cannot change at all.
pub(crate) fn set_frame_width(curfrp: Frame, width: c_int) {
    if curfrp.fr_width == width {
        return;
    }
    let Some(parent) = curfrp.parent() else {
        // topframe: can't change width.
        return;
    };
    if parent.fr_layout as c_int == FR_COL {
        let w = minwidth(parent, NextCurwin::Unset);
        set_frame_width(parent, width.max(w));
        return;
    }

    let mut width = width;
    let mut room = 0;
    let mut room_reserved = 0;
    for run in 1..=2 {
        room = 0;
        room_reserved = 0;
        for frp in parent.children() {
            if frp != curfrp && frp.win().is_some_and(|w| w.w_onebuf_opt.wo_wfw != 0) {
                room_reserved += frp.fr_width;
            }
            room += frp.fr_width;
            if frp != curfrp {
                room -= minwidth(frp, NextCurwin::Unset);
            }
        }
        if width <= room {
            break;
        }
        let rows_avail = Rows.get() as OptInt
            - p_ch.get()
            - tabline_rows() as OptInt
            - global_stl_rows() as OptInt;
        if run == 2 || curfrp.fr_height as OptInt >= rows_avail {
            width = room;
            break;
        }
        let target = arith::parent_target(
            width,
            minwidth(parent, NextCurwin::NoWin),
            p_wmw.get() as c_int,
        );
        set_frame_width(parent, target);
    }

    let mut take = width - curfrp.fr_width;
    if width > room - room_reserved {
        room_reserved = room - width;
    }
    if take < 0 && room - curfrp.fr_width < room_reserved {
        room_reserved = 0;
    }

    new_width(curfrp, width, false, false);

    for run in 0..2 {
        let start = if run == 0 {
            curfrp.next()
        } else {
            curfrp.prev()
        };
        let walk: Box<dyn Iterator<Item = Frame>> = if run == 0 {
            Box::new(frames(start))
        } else {
            Box::new(frames_back(start))
        };
        for frp in walk {
            if take == 0 {
                break;
            }
            let w = minwidth(frp, NextCurwin::Unset);
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

pub unsafe fn did_set_winminheight(_args: *mut optset_T) -> *const c_char {
    let mut first = true;
    // Loop until there is a 'winminheight' that is possible.
    while p_wmh.get() > 0 as OptInt {
        let room = Rows.get() - p_ch.get() as c_int;
        if room >= min_rows_all_tabpages() {
            break;
        }
        p_wmh.set(p_wmh.get() - 1);
        if first {
            err(&raw const e_noroom as *const c_char);
            first = false;
        }
    }
    ::core::ptr::null()
}

pub unsafe fn did_set_winminwidth(_args: *mut optset_T) -> *const c_char {
    let mut first = true;
    while p_wmw.get() > 0 as OptInt {
        if Columns.get() >= minwidth(current_topframe(), NextCurwin::Unset) {
            break;
        }
        p_wmw.set(p_wmw.get() - 1);
        if first {
            err(&raw const e_noroom as *const c_char);
            first = false;
        }
    }
    ::core::ptr::null()
}

pub unsafe fn win_drag_status_line(dragwin: *mut win_T, offset: c_int) {
    // SAFETY: the caller's promise -- a live window.
    drag_status_line(unsafe { Win::new(dragwin) }, offset);
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
        if fr.fr_layout as c_int != FR_COL {
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
            room = fr.fr_height - minheight(fr, NextCurwin::Unset);
        } else {
            let mut sum = 0;
            fr = fr.child().expect("a row or column has a child");
            loop {
                sum += fr.fr_height - minheight(fr, NextCurwin::Unset);
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
            sum -= p_ch.get() as c_int + global_stl_rows();
        } else if min_set_ch.get() > 0 as OptInt {
            sum -= 1;
        }
        sum = sum.max(0);
        // Sum up the room of the frames below the current one.
        for frp in frames(curfr.next()) {
            sum += frp.fr_height - minheight(frp, NextCurwin::Unset);
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
        let n = minheight(fr, NextCurwin::Unset);
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
    // SAFETY: writes the mode message to the command line, which always exists.
    unsafe { showmode() };
}

pub unsafe fn win_drag_vsep_line(dragwin: *mut win_T, offset: c_int) {
    // SAFETY: the caller's promise -- a live window.
    drag_vsep_line(unsafe { Win::new(dragwin) }, offset);
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
    if fr.fr_layout as c_int != FR_ROW {
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
            room += fr.fr_width - minwidth(fr, NextCurwin::Unset);
            if fr == curfr {
                break;
            }
            fr = fr.next().expect("curfr is among these children");
        }
        curfr.next()
    } else {
        // Sum up the room of the frames right of the current one.
        for frp in frames(curfr.next()) {
            room += frp.fr_width - minwidth(frp, NextCurwin::Unset);
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
        let n = minwidth(fr, NextCurwin::Unset);
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
