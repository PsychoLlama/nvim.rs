//! The window arithmetic that touches no global and calls nothing back: how
//! small a frame may become, whether it ended up the size it was asked for,
//! where the cursor sits as a fraction of a window's height, and the two
//! additions that a `:resize` with an absurd count would otherwise overflow.
//!
//! These are the decisions the rest of the family makes between its window
//! writes, lifted out so they can be stated — and tested — on their own.
//! `tests/unit/window.rs` drives them directly, which is also how Miri sees
//! this half of the family. Nothing here reads an option or a global: the
//! values `frame_minheight()` and friends take from `'winheight'`,
//! `'winminheight'` and `curwin` arrive in a [`MinSize`], so a caller is what
//! decides *when* they are read — which matters, because `win_equal()` reads
//! them once and walks the tree many times.
//!
//! The one thing that is not a plain port is the arithmetic width: `:resize`
//! and `'winheight'` accept any `int`, and the C then adds a status line to it
//! and relies on the overflow wrapping. Three additions here saturate instead
//! (each says so), which turns a wrapped-negative height into the largest one
//! the layout can honour — the answer the clamps further down were going to
//! reach anyway.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::{FR_COL, FR_ROW, FRACTION_MULT, NOWIN};
use crate::src::nvim::types::win_T;
use crate::src::nvim::winlayer::Frame;

/// The C's `next_curwin` argument to [`frame_minheight`]/[`frame_minwidth`],
/// whose three states are three different rules — and which is why they are
/// not one `Option<Win>`:
///
/// * `NULL` asks for the minimum as things stand, and reserves a line (or a
///   column) for the *current* window when `'winminheight'` is zero;
/// * `NOWIN`, the `(win_T *)-1` sentinel `win_equal()` and `win_split_ins()`
///   pass, asks for the same minimum with **no** such reservation;
/// * a window asks for the minimum given that this window is about to become
///   current, so it gets `'winheight'`/`'winwidth'` rather than the minimum.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NextCurwin {
    /// The C's `NULL`.
    Unset,
    /// The C's `NOWIN`.
    NoWin,
    /// A window that is about to become current.
    Win(*mut win_T),
}

impl NextCurwin {
    /// The `win_T *` the C passes, read back as the three cases.
    pub fn of(win: *mut win_T) -> Self {
        if win.is_null() {
            Self::Unset
        } else if win == NOWIN {
            Self::NoWin
        } else {
            Self::Win(win)
        }
    }

    /// The `win_T *` back again, for the entry points that still hand one on.
    pub fn raw(self) -> *mut win_T {
        match self {
            Self::Unset => core::ptr::null_mut(),
            Self::NoWin => NOWIN,
            Self::Win(win) => win,
        }
    }

    /// Whether this asks about `win` in particular — the C's
    /// `topfrp->fr_win == next_curwin`, which neither sentinel can satisfy.
    fn is(self, win: *mut win_T) -> bool {
        self == Self::Win(win)
    }
}

/// What [`frame_minheight`] and [`frame_minwidth`] need from the options: the
/// size a `next_curwin` window is owed (`'winheight'` / `'winwidth'`), the
/// minimum any other window may shrink to (`'winminheight'` / `'winminwidth'`)
/// and the current window, for the one-line reservation the `NULL` case makes.
#[derive(Clone, Copy)]
pub struct MinSize {
    pub wanted: c_int,
    pub minimum: c_int,
    pub curwin: *mut win_T,
}

/// The minimal height of frame `topfrp`, from `frame_minheight()`.
///
/// A leaf costs its window's minimum plus the rows that are not text (window
/// bar, separator, status line); a row of frames costs the tallest of them and
/// a column the sum.
pub fn frame_minheight(topfrp: Frame, next_curwin: NextCurwin, opts: MinSize) -> c_int {
    if let Some(win) = topfrp.win() {
        // Combined height of window bar and separator column or status line.
        let extra_height = win.w_winbar_height + win.w_hsep_height + win.w_status_height;
        if next_curwin.is(win.raw()) {
            // Saturating: `'winheight'` is an unclamped option.
            return opts.wanted.saturating_add(extra_height);
        }
        let mut m = opts.minimum.saturating_add(extra_height);
        // Current window is minimal one line high.
        if win.raw() == opts.curwin && next_curwin == NextCurwin::Unset && opts.minimum == 0 {
            m += 1;
        }
        m
    } else if topfrp.fr_layout as c_int == FR_ROW {
        // The minimal height of the tallest frame in this row.
        topfrp.children().fold(0, |m, frp| {
            let n = frame_minheight(frp, next_curwin, opts);
            if n > m { n } else { m }
        })
    } else {
        // The minimal heights of every frame in this column, added up.
        topfrp.children().fold(0, |m, frp| {
            m.saturating_add(frame_minheight(frp, next_curwin, opts))
        })
    }
}

/// The minimal width of frame `topfrp`, from `frame_minwidth()`: the mirror of
/// [`frame_minheight`], with a column taking the widest child and a row the
/// sum, and the separator column standing in for the status line.
pub fn frame_minwidth(topfrp: Frame, next_curwin: NextCurwin, opts: MinSize) -> c_int {
    if let Some(win) = topfrp.win() {
        if next_curwin.is(win.raw()) {
            // Saturating: `'winwidth'` is an unclamped option.
            return opts.wanted.saturating_add(win.w_vsep_width);
        }
        // Window: minimal width of the window plus separator column.
        let mut m = opts.minimum.saturating_add(win.w_vsep_width);
        // Current window is minimal one column wide.
        if opts.minimum == 0 && win.raw() == opts.curwin && next_curwin == NextCurwin::Unset {
            m += 1;
        }
        m
    } else if topfrp.fr_layout as c_int == FR_COL {
        topfrp.children().fold(0, |m, frp| {
            let n = frame_minwidth(frp, next_curwin, opts);
            if m > n { m } else { n }
        })
    } else {
        topfrp.children().fold(0, |m, frp| {
            m.saturating_add(frame_minwidth(frp, next_curwin, opts))
        })
    }
}

/// Whether `topfrp` and its children are at height `height`, from
/// `frame_check_height()`.
///
/// Only a *row*'s children are checked, because only they share their parent's
/// height; a column's are expected to differ, and so are not looked at.
pub fn frame_check_height(topfrp: Frame, height: c_int) -> bool {
    if topfrp.fr_height != height {
        return false;
    }
    if topfrp.fr_layout as c_int == FR_ROW {
        return topfrp.children().all(|frp| frp.fr_height == height);
    }
    true
}

/// Whether `topfrp` and its children are at width `width`, from
/// `frame_check_width()` — [`frame_check_height`] with the axes exchanged, so
/// it is a *column* whose children must match.
pub fn frame_check_width(topfrp: Frame, width: c_int) -> bool {
    if topfrp.fr_width != width {
        return false;
    }
    if topfrp.fr_layout as c_int == FR_COL {
        return topfrp.children().all(|frp| frp.fr_width == width);
    }
    true
}

/// Sort `'colorcolumn'`'s columns ascending, which is what upstream's
/// `qsort(..., int_cmp)` does. Equal `int`s are indistinguishable, so an
/// unstable sort cannot be told from `qsort`'s.
pub fn sort_columns(columns: &mut [c_int]) {
    columns.sort_unstable();
}

/// A requested window height plus the rows below it that are not text, from
/// `win_setheight_win()`.
///
/// Saturating: `:resize 2147483647` reaches here, and the C's wrap turns the
/// tallest window anyone can ask for into the shortest. Everything downstream
/// clamps to the room actually available, so saturating and wrapping differ
/// only in which of the two answers comes out.
pub fn height_with_chrome(height: c_int, hsep_height: c_int, status_height: c_int) -> c_int {
    height
        .saturating_add(hsep_height)
        .saturating_add(status_height)
}

/// A requested window width plus its separator column, from
/// `win_setwidth_win()`. Saturating for the reason [`height_with_chrome`] is.
pub fn width_with_chrome(width: c_int, vsep_width: c_int) -> c_int {
    width.saturating_add(vsep_width)
}

/// The height (or width) to ask a parent frame for when the siblings cannot
/// pay for `size`: `frame_setheight()`/`frame_setwidth()`'s
/// `size + frame_min*(parent, NOWIN) - p_wm* - 1`. Saturating, as above —
/// `size` is the caller's unclamped request.
pub fn parent_target(size: c_int, parent_minimum: c_int, option_minimum: c_int) -> c_int {
    size.saturating_add(parent_minimum)
        .saturating_sub(option_minimum)
        .saturating_sub(1)
}

/// Where the cursor sits in a window, as sixteen-thousandths of its height,
/// from `set_fraction()`.
///
/// A cursor on the first line counts as halfway down it, so two lines give
/// 25%, three 17%, and the last line 75%, 83% and so on.
pub fn cursor_fraction(wrow: c_int, view_height: c_int) -> c_int {
    (wrow * FRACTION_MULT + FRACTION_MULT / 2) / view_height
}

/// The inverse, from `scroll_to_fraction()`: the screen row a saved
/// [`cursor_fraction`] puts the cursor on in a window `height` rows tall.
pub fn fraction_row(fraction: c_int, height: c_int) -> c_int {
    (fraction * height - 1) / FRACTION_MULT
}
