//! Frame arithmetic -- giving a frame a new height or width.
//!
//! [`new_height`] and [`new_width`] distribute a frame's new size over its
//! children, recursing into rows and columns and stopping at the
//! `'winfix{height,width}'` pins; [`FrameRef::min_height`] and [`FrameRef::min_width`] answer how
//! small a frame may become, reading the options once and handing the walk
//! itself to [`arith`].  The `add_statusline`/`add_hsep`/`set_vsep` trio
//! adjusts the non-text rows and columns when the layout changes around them.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

// The whole file is frame arithmetic over identities now that
// `frame_new_height`'s raw-pointer shim is gone, so nothing here needs a
// dereference.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::arith::{MinSize, NextCurwin};
use super::*;
use crate::option::defs::{OptVal, OptionSetFlags};
use crate::option::set_option_value;
use crate::option::vars::{p_ch, p_wh, p_wiw, p_wmh, p_wmw};
use crate::options::kOptCmdheight;
use crate::types::OptInt;
use crate::ui::state::Rows;
use crate::winlayer::{FrameRef, Win};

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
        wanted: ::core::ffi::c_int::try_from(p_wh()).unwrap_or(::core::ffi::c_int::MAX),
        minimum: ::core::ffi::c_int::try_from(p_wmh()).unwrap_or(::core::ffi::c_int::MAX),
        curwin: Win::current_or_none().map(Win::id),
    }
}

/// `'winwidth'`, `'winminwidth'` and the current window.
fn width_opts() -> MinSize {
    MinSize {
        wanted: ::core::ffi::c_int::try_from(p_wiw()).unwrap_or(::core::ffi::c_int::MAX),
        minimum: ::core::ffi::c_int::try_from(p_wmw()).unwrap_or(::core::ffi::c_int::MAX),
        curwin: Win::current_or_none().map(Win::id),
    }
}

impl FrameRef {
    /// The minimal height of this frame, from `frame_minheight()`.
    pub(crate) fn min_height(self, next_curwin: NextCurwin) -> ::core::ffi::c_int {
        self.min_height_with(next_curwin, height_opts())
    }

    /// The minimal width of this frame, from `frame_minwidth()`.
    pub(crate) fn min_width(self, next_curwin: NextCurwin) -> ::core::ffi::c_int {
        self.min_width_with(next_curwin, width_opts())
    }

    /// The child of a column [`new_height`] starts giving or taking rows at: the
    /// first when `topfirst`, the last otherwise, skipping `'winfixheight'`
    /// frames when `wfh`. `None` when every child is pinned.
    fn column_end(self, topfirst: bool, wfh: bool) -> Option<FrameRef> {
        let mut frp = self.child()?;
        if wfh {
            while frp.fixed_height() {
                frp = frp.next()?;
            }
        }
        if !topfirst {
            while let Some(next) = frp.next() {
                frp = next;
            }
            if wfh {
                while frp.fixed_height() {
                    frp = frp.prev()?;
                }
            }
        }
        Some(frp)
    }

    /// The next child to take rows from, `'winfixheight'` frames skipped.
    fn step_over_fixed_height(self, topfirst: bool, wfh: bool) -> Option<FrameRef> {
        let mut next = if topfirst { self.next() } else { self.prev() };
        while wfh && next.is_some_and(FrameRef::fixed_height) {
            next = if topfirst { next?.next() } else { next?.prev() };
        }
        next
    }

    /// Whether `self` may not be given a new height: a leaf whose window has
    /// `'winfixheight'`, a row with any such frame in it, or a column of them.
    pub(crate) fn fixed_height(self) -> bool {
        if let Some(win) = self.win() {
            return win.w_onebuf_opt.wo_wfh != 0;
        }
        if ::core::ffi::c_int::from(self.fr_layout) == FR_ROW {
            // The row is fixed if one of the frames in it is fixed.
            return self.children().any(FrameRef::fixed_height);
        }
        // The column is fixed if all frames in it are fixed.
        self.children().all(FrameRef::fixed_height)
    }

    /// [`FrameRef::fixed_height`] with the axes exchanged: `'winfixwidth'`, and it is a
    /// *column* that is fixed as soon as one child is.
    pub(crate) fn fixed_width(self) -> bool {
        if let Some(win) = self.win() {
            return win.w_onebuf_opt.wo_wfw != 0;
        }
        if ::core::ffi::c_int::from(self.fr_layout) == FR_COL {
            return self.children().any(FrameRef::fixed_width);
        }
        self.children().all(FrameRef::fixed_width)
    }

    /// Give the windows along the bottom of `self` a status line, without changing
    /// any height: the caller has already made room.
    pub(crate) fn add_statusline(self) {
        if let Some(mut win) = self.win() {
            win.w_status_height = STATUS_HEIGHT;
        } else if ::core::ffi::c_int::from(self.fr_layout) == FR_ROW {
            // Handle all the frames in the row.
            self.children().for_each(FrameRef::add_statusline);
        } else {
            debug_assert!(
                ::core::ffi::c_int::from(self.fr_layout) == FR_COL,
                "frp->fr_layout == FR_COL"
            );
            // Only the last frame in the column needs a status line.
            if let Some(last) = self.children().last() {
                last.add_statusline();
            }
        }
    }

    /// [`FrameRef::column_end`] for a row of frames.
    fn row_end(self, leftfirst: bool, wfw: bool) -> Option<FrameRef> {
        let mut frp = self.child()?;
        if wfw {
            while frp.fixed_width() {
                frp = frp.next()?;
            }
        }
        if !leftfirst {
            while let Some(next) = frp.next() {
                frp = next;
            }
            if wfw {
                while frp.fixed_width() {
                    frp = frp.prev()?;
                }
            }
        }
        Some(frp)
    }

    /// [`FrameRef::step_over_fixed_height`] for a row of frames.
    fn step_over_fixed_width(self, leftfirst: bool, wfw: bool) -> Option<FrameRef> {
        let mut next = if leftfirst { self.next() } else { self.prev() };
        while wfw && next.is_some_and(FrameRef::fixed_width) {
            next = if leftfirst {
                next?.next()
            } else {
                next?.prev()
            };
        }
        next
    }

    /// [`FrameRef::add_statusline`] for the horizontal separator `'laststatus'` = 3 draws in
    /// a status line's place.
    pub(crate) fn add_hsep(self) {
        if let Some(mut win) = self.win() {
            win.w_hsep_height = 1 as ::core::ffi::c_int;
        } else if ::core::ffi::c_int::from(self.fr_layout) == FR_ROW {
            self.children().for_each(FrameRef::add_hsep);
        } else {
            debug_assert!(
                ::core::ffi::c_int::from(self.fr_layout) == FR_COL,
                "frp->fr_layout == FR_COL"
            );
            if let Some(last) = self.children().last() {
                last.add_hsep();
            }
        }
    }

    /// Add or remove the separator column along the right edge of `self`, taking
    /// the column it needs out of the windows' text.
    pub(crate) fn set_vsep(self, add: bool) {
        if let Some(mut win) = self.win() {
            if add && win.w_vsep_width == 0 {
                if win.w_width > 0 {
                    new_win_width(win, win.w_width - 1);
                }
                win.w_vsep_width = 1 as ::core::ffi::c_int;
            } else if !add && win.w_vsep_width == 1 {
                new_win_width(win, win.w_width + 1);
                win.w_vsep_width = 0 as ::core::ffi::c_int;
            }
        } else if ::core::ffi::c_int::from(self.fr_layout) == FR_COL {
            // Handle all the frames in the column.
            self.children().for_each(|child| child.set_vsep(add));
        } else {
            debug_assert!(
                ::core::ffi::c_int::from(self.fr_layout) == FR_ROW,
                "frp->fr_layout == FR_ROW"
            );
            // Only the last frame in the row needs a separator.
            if let Some(last) = self.children().last() {
                last.set_vsep(add);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// New sizes

/// Give frame `topfrp` height `height`, from `frame_new_height()`.
///
/// `topfirst` takes the room from the top of a column rather than the bottom,
/// `wfh` skips `'winfixheight'` windows, and `set_ch` lets the top frame trade
/// rows with `'cmdheight'`.
pub(crate) fn new_height(
    topfrp: FrameRef,
    mut height: ::core::ffi::c_int,
    topfirst: bool,
    wfh: bool,
    set_ch: bool,
) {
    let mut topfrp = topfrp;
    if topfrp.parent().is_none() && set_ch {
        // The top frame's height is the screen's minus the command line, so
        // giving it a new one means giving 'cmdheight' the difference.
        let want_ch = p_ch() + OptInt::from(topfrp.fr_height) - OptInt::from(height);
        let new_ch = min_set_ch.get().max(want_ch);
        if new_ch != p_ch() {
            let save_ch = min_set_ch.get();
            let _ = set_option_value(kOptCmdheight, OptVal::Number(new_ch), OptionSetFlags::NONE);
            min_set_ch.set(save_ch);
        }
        let room = OptInt::from(Rows.get())
            - p_ch()
            - OptInt::from(tabline_rows())
            - OptInt::from(global_stl_rows());
        let room = room.clamp(OptInt::from(::core::ffi::c_int::MIN), OptInt::from(height));
        height = ::core::ffi::c_int::try_from(room).expect("room was just clamped into c_int");
    }
    if let Some(mut wp) = topfrp.win() {
        if is_bottom_window(wp) {
            wp.w_hsep_height = 0 as ::core::ffi::c_int;
        }
        new_win_height(wp, height - wp.w_hsep_height - wp.w_status_height);
    } else if ::core::ffi::c_int::from(topfrp.fr_layout) == FR_ROW {
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
        let Some(first) = topfrp.column_end(topfirst, wfh) else {
            return;
        };
        let mut frp = Some(first);
        let mut extra_lines = height - topfrp.fr_height;
        if extra_lines < 0 {
            while let Some(cur) = frp {
                let h = cur.min_height(NextCurwin::Unset);
                if cur.fr_height + extra_lines >= h {
                    new_height(cur, cur.fr_height + extra_lines, topfirst, wfh, set_ch);
                    break;
                }
                extra_lines += cur.fr_height - h;
                new_height(cur, h, topfirst, wfh, set_ch);
                frp = cur.step_over_fixed_height(topfirst, wfh);
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

/// Give frame `topfrp` width `width`, from `frame_new_width()` -- [`new_height`]
/// with the axes exchanged, minus the `'cmdheight'` arm a width can never have.
pub(crate) fn new_width(
    topfrp: FrameRef,
    mut width: ::core::ffi::c_int,
    leftfirst: bool,
    wfw: bool,
) {
    let mut topfrp = topfrp;
    if ::core::ffi::c_int::from(topfrp.fr_layout) == FR_LEAF {
        let mut wp = topfrp.win().expect("a leaf frame holds a window");
        // Find out if there are any windows right of this one.
        let mut frp = topfrp;
        while let Some(parent) = frp.parent() {
            if ::core::ffi::c_int::from(parent.fr_layout) == FR_ROW && frp.next().is_some() {
                break;
            }
            frp = parent;
        }
        if frp.parent().is_none() {
            wp.w_vsep_width = 0 as ::core::ffi::c_int;
        }
        new_win_width(wp, width - wp.w_vsep_width);
    } else if ::core::ffi::c_int::from(topfrp.fr_layout) == FR_COL {
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
        let Some(first) = topfrp.row_end(leftfirst, wfw) else {
            return;
        };
        let mut frp = Some(first);
        let mut extra_cols = width - topfrp.fr_width;
        if extra_cols < 0 {
            while let Some(cur) = frp {
                let w = cur.min_width(NextCurwin::Unset);
                if cur.fr_width + extra_cols >= w {
                    new_width(cur, cur.fr_width + extra_cols, leftfirst, wfw);
                    break;
                }
                extra_cols += cur.fr_width - w;
                new_width(cur, w, leftfirst, wfw);
                frp = cur.step_over_fixed_width(leftfirst, wfw);
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

/// Set a leaf frame's width from the window it contains.
pub(crate) fn frame_fix_width(window: Win) {
    let mut frame = window.frame();
    frame.fr_width = window.w_width + window.w_vsep_width;
}

/// Set a leaf frame's height from the window it contains.
pub(crate) fn frame_fix_height(window: Win) {
    let mut frame = window.frame();
    frame.fr_height = window.w_height + window.w_hsep_height + window.w_status_height;
}
