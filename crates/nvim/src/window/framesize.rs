//! Frame arithmetic -- giving a frame a new height or width.
//!
//! [`new_height`] and [`new_width`] distribute a frame's new size over its
//! children, recursing into rows and columns and stopping at the
//! `'winfix{height,width}'` pins; [`minheight`] and [`minwidth`] answer how
//! small a frame may become, reading the options once and handing the walk
//! itself to [`arith`].  The `add_statusline`/`add_hsep`/`set_vsep` trio
//! adjusts the non-text rows and columns when the layout changes around them.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::arith::{MinSize, NextCurwin};
use super::*;
use crate::main::{Rows, curwin, p_ch, p_wh, p_wiw, p_wmh, p_wmw};
use crate::option::set_option_value;
use crate::options::kOptCmdheight;
use crate::types::{OptInt, OptVal, OptionSetFlags, frame_T};
use crate::winlayer::{Frame, Win};

// ---------------------------------------------------------------------------
// The minimum sizes
//
// The options are read here, once per call, and the walk that uses them is
// `arith`'s. `next_curwin` keeps all three of the C's cases: NULL, the NOWIN
// sentinel, and a window about to become current -- collapsing the first two
// would lose the one-line reservation only the NULL case makes.

/// `'winheight'`, `'winminheight'` and the current window, as
/// [`arith::frame_minheight`] wants them.
fn height_opts() -> MinSize {
    MinSize {
        wanted: p_wh.get() as ::core::ffi::c_int,
        minimum: p_wmh.get() as ::core::ffi::c_int,
        curwin: curwin.get(),
    }
}

/// `'winwidth'`, `'winminwidth'` and the current window.
fn width_opts() -> MinSize {
    MinSize {
        wanted: p_wiw.get() as ::core::ffi::c_int,
        minimum: p_wmw.get() as ::core::ffi::c_int,
        curwin: curwin.get(),
    }
}

/// The minimal height of frame `topfrp`, from `frame_minheight()`.
pub(crate) fn minheight(topfrp: Frame, next_curwin: NextCurwin) -> ::core::ffi::c_int {
    arith::frame_minheight(topfrp, next_curwin, height_opts())
}

/// The minimal width of frame `topfrp`, from `frame_minwidth()`.
pub(crate) fn minwidth(topfrp: Frame, next_curwin: NextCurwin) -> ::core::ffi::c_int {
    arith::frame_minwidth(topfrp, next_curwin, width_opts())
}

// ---------------------------------------------------------------------------
// New sizes

/// Give frame `topfrp` height `height`, from `frame_new_height()`.
///
/// `topfirst` takes the room from the top of a column rather than the bottom,
/// `wfh` skips `'winfixheight'` windows, and `set_ch` lets the top frame trade
/// rows with `'cmdheight'`.
pub(crate) fn new_height(
    topfrp: Frame,
    mut height: ::core::ffi::c_int,
    topfirst: bool,
    wfh: bool,
    set_ch: bool,
) {
    let mut topfrp = topfrp;
    if topfrp.parent().is_none() && set_ch {
        // The top frame's height is the screen's minus the command line, so
        // giving it a new one means giving 'cmdheight' the difference.
        let want_ch = p_ch.get() + topfrp.fr_height as OptInt - height as OptInt;
        let new_ch = min_set_ch.get().max(want_ch);
        if new_ch != p_ch.get() {
            let save_ch = min_set_ch.get();
            set_option_value(kOptCmdheight, OptVal::Number(new_ch), OptionSetFlags::NONE);
            min_set_ch.set(save_ch);
        }
        let room = Rows.get() as OptInt
            - p_ch.get()
            - tabline_rows() as OptInt
            - global_stl_rows() as OptInt;
        height = room.min(height as OptInt) as ::core::ffi::c_int;
    }
    if let Some(mut wp) = topfrp.win() {
        if is_bottom_window(wp) {
            wp.w_hsep_height = 0 as ::core::ffi::c_int;
        }
        new_win_height(wp, height - wp.w_hsep_height - wp.w_status_height);
    } else if topfrp.fr_layout as ::core::ffi::c_int == FR_ROW {
        // All frames in this row get the same new height. If one of them could
        // not fit its windows in it, take its height for the whole row and go
        // round again.
        loop {
            let mut grew = false;
            for frp in topfrp.children() {
                new_height(frp, height, topfirst, wfh, set_ch);
                if frp.fr_height > height {
                    height = frp.fr_height;
                    grew = true;
                    break;
                }
            }
            if !grew {
                break;
            }
        }
    } else {
        // A column: give the difference to one child and let it recurse, or
        // take it from as many as it needs when there is not enough.
        let Some(first) = column_end(topfrp, topfirst, wfh) else {
            return;
        };
        let mut frp = Some(first);
        let mut extra_lines = height - topfrp.fr_height;
        if extra_lines < 0 {
            while let Some(cur) = frp {
                let h = minheight(cur, NextCurwin::Unset);
                if cur.fr_height + extra_lines >= h {
                    new_height(cur, cur.fr_height + extra_lines, topfirst, wfh, set_ch);
                    break;
                }
                extra_lines += cur.fr_height - h;
                new_height(cur, h, topfirst, wfh, set_ch);
                frp = step_over_fixed(cur, topfirst, wfh);
                if frp.is_none() {
                    // Nothing left to take from: the column stays taller.
                    height -= extra_lines;
                }
            }
        } else if extra_lines > 0 {
            new_height(first, first.fr_height + extra_lines, topfirst, wfh, set_ch);
        }
    }
    topfrp.fr_height = height;
}

/// The child of a column [`new_height`] starts giving or taking rows at: the
/// first when `topfirst`, the last otherwise, skipping `'winfixheight'`
/// frames when `wfh`. `None` when every child is pinned.
fn column_end(topfrp: Frame, topfirst: bool, wfh: bool) -> Option<Frame> {
    let mut frp = topfrp.child()?;
    if wfh {
        while frame_fixed_height(frp) {
            frp = frp.next()?;
        }
    }
    if !topfirst {
        while let Some(next) = frp.next() {
            frp = next;
        }
        if wfh {
            while frame_fixed_height(frp) {
                frp = frp.prev()?;
            }
        }
    }
    Some(frp)
}

/// The next child to take rows from, `'winfixheight'` frames skipped.
fn step_over_fixed(frp: Frame, topfirst: bool, wfh: bool) -> Option<Frame> {
    let mut next = if topfirst { frp.next() } else { frp.prev() };
    while wfh && next.is_some_and(frame_fixed_height) {
        next = if topfirst { next?.next() } else { next?.prev() };
    }
    next
}

pub unsafe fn frame_new_height(
    topfrp: *mut frame_T,
    height: ::core::ffi::c_int,
    topfirst: bool,
    wfh: bool,
    set_ch: bool,
) {
    // SAFETY: the caller's promise -- a live frame.
    new_height(unsafe { Frame::new(topfrp) }, height, topfirst, wfh, set_ch);
}

/// Whether `frp` may not be given a new height: a leaf whose window has
/// `'winfixheight'`, a row with any such frame in it, or a column of them.
pub(crate) fn frame_fixed_height(frp: Frame) -> bool {
    if let Some(win) = frp.win() {
        return win.w_onebuf_opt.wo_wfh != 0;
    }
    if frp.fr_layout as ::core::ffi::c_int == FR_ROW {
        // The row is fixed if one of the frames in it is fixed.
        return frp.children().any(frame_fixed_height);
    }
    // The column is fixed if all frames in it are fixed.
    frp.children().all(frame_fixed_height)
}

/// [`frame_fixed_height`] with the axes exchanged: `'winfixwidth'`, and it is a
/// *column* that is fixed as soon as one child is.
pub(crate) fn frame_fixed_width(frp: Frame) -> bool {
    if let Some(win) = frp.win() {
        return win.w_onebuf_opt.wo_wfw != 0;
    }
    if frp.fr_layout as ::core::ffi::c_int == FR_COL {
        return frp.children().any(frame_fixed_width);
    }
    frp.children().all(frame_fixed_width)
}

/// Give the windows along the bottom of `frp` a status line, without changing
/// any height: the caller has already made room.
pub(crate) fn add_statusline(frp: Frame) {
    if let Some(mut win) = frp.win() {
        win.w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
    } else if frp.fr_layout as ::core::ffi::c_int == FR_ROW {
        // Handle all the frames in the row.
        frp.children().for_each(add_statusline);
    } else {
        debug_assert!(
            frp.fr_layout as ::core::ffi::c_int == FR_COL,
            "frp->fr_layout == FR_COL"
        );
        // Only the last frame in the column needs a status line.
        if let Some(last) = frp.children().last() {
            add_statusline(last);
        }
    }
}

/// Give frame `topfrp` width `width`, from `frame_new_width()` -- [`new_height`]
/// with the axes exchanged, minus the `'cmdheight'` arm a width can never have.
pub(crate) fn new_width(topfrp: Frame, mut width: ::core::ffi::c_int, leftfirst: bool, wfw: bool) {
    let mut topfrp = topfrp;
    if topfrp.fr_layout as ::core::ffi::c_int == FR_LEAF {
        let mut wp = topfrp.win().expect("a leaf frame holds a window");
        // Find out if there are any windows right of this one.
        let mut frp = topfrp;
        while let Some(parent) = frp.parent() {
            if parent.fr_layout as ::core::ffi::c_int == FR_ROW && frp.next().is_some() {
                break;
            }
            frp = parent;
        }
        if frp.parent().is_none() {
            wp.w_vsep_width = 0 as ::core::ffi::c_int;
        }
        new_win_width(wp, width - wp.w_vsep_width);
    } else if topfrp.fr_layout as ::core::ffi::c_int == FR_COL {
        loop {
            let mut grew = false;
            for frp in topfrp.children() {
                new_width(frp, width, leftfirst, wfw);
                if frp.fr_width > width {
                    width = frp.fr_width;
                    grew = true;
                    break;
                }
            }
            if !grew {
                break;
            }
        }
    } else {
        let Some(first) = row_end(topfrp, leftfirst, wfw) else {
            return;
        };
        let mut frp = Some(first);
        let mut extra_cols = width - topfrp.fr_width;
        if extra_cols < 0 {
            while let Some(cur) = frp {
                let w = minwidth(cur, NextCurwin::Unset);
                if cur.fr_width + extra_cols >= w {
                    new_width(cur, cur.fr_width + extra_cols, leftfirst, wfw);
                    break;
                }
                extra_cols += cur.fr_width - w;
                new_width(cur, w, leftfirst, wfw);
                frp = step_over_fixed_width(cur, leftfirst, wfw);
                if frp.is_none() {
                    width -= extra_cols;
                }
            }
        } else if extra_cols > 0 {
            new_width(first, first.fr_width + extra_cols, leftfirst, wfw);
        }
    }
    topfrp.fr_width = width;
}

/// [`column_end`] for a row of frames.
fn row_end(topfrp: Frame, leftfirst: bool, wfw: bool) -> Option<Frame> {
    let mut frp = topfrp.child()?;
    if wfw {
        while frame_fixed_width(frp) {
            frp = frp.next()?;
        }
    }
    if !leftfirst {
        while let Some(next) = frp.next() {
            frp = next;
        }
        if wfw {
            while frame_fixed_width(frp) {
                frp = frp.prev()?;
            }
        }
    }
    Some(frp)
}

/// [`step_over_fixed`] for a row of frames.
fn step_over_fixed_width(frp: Frame, leftfirst: bool, wfw: bool) -> Option<Frame> {
    let mut next = if leftfirst { frp.next() } else { frp.prev() };
    while wfw && next.is_some_and(frame_fixed_width) {
        next = if leftfirst {
            next?.next()
        } else {
            next?.prev()
        };
    }
    next
}

/// Add or remove the separator column along the right edge of `frp`, taking
/// the column it needs out of the windows' text.
pub(crate) fn set_vsep(frp: Frame, add: bool) {
    if let Some(mut win) = frp.win() {
        if add && win.w_vsep_width == 0 {
            if win.w_width > 0 {
                new_win_width(win, win.w_width - 1);
            }
            win.w_vsep_width = 1 as ::core::ffi::c_int;
        } else if !add && win.w_vsep_width == 1 {
            new_win_width(win, win.w_width + 1);
            win.w_vsep_width = 0 as ::core::ffi::c_int;
        }
    } else if frp.fr_layout as ::core::ffi::c_int == FR_COL {
        // Handle all the frames in the column.
        frp.children().for_each(|frp| set_vsep(frp, add));
    } else {
        debug_assert!(
            frp.fr_layout as ::core::ffi::c_int == FR_ROW,
            "frp->fr_layout == FR_ROW"
        );
        // Only the last frame in the row needs a separator.
        if let Some(last) = frp.children().last() {
            set_vsep(last, add);
        }
    }
}

/// [`add_statusline`] for the horizontal separator `'laststatus'` = 3 draws in
/// a status line's place.
pub(crate) fn add_hsep(frp: Frame) {
    if let Some(mut win) = frp.win() {
        win.w_hsep_height = 1 as ::core::ffi::c_int;
    } else if frp.fr_layout as ::core::ffi::c_int == FR_ROW {
        frp.children().for_each(add_hsep);
    } else {
        debug_assert!(
            frp.fr_layout as ::core::ffi::c_int == FR_COL,
            "frp->fr_layout == FR_COL"
        );
        if let Some(last) = frp.children().last() {
            add_hsep(last);
        }
    }
}

/// Set a leaf frame's width from the window it contains.
pub(crate) fn frame_fix_width(wp: Win) {
    let mut frame = wp.frame();
    frame.fr_width = wp.w_width + wp.w_vsep_width;
}

/// Set a leaf frame's height from the window it contains.
pub(crate) fn frame_fix_height(wp: Win) {
    let mut frame = wp.frame();
    frame.fr_height = wp.w_height + wp.w_hsep_height + wp.w_status_height;
}
