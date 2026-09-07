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
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::{FR_COL, FR_ROW, FRACTION_MULT};
use crate::winlayer::{FrameRef, Win, WinId};

/// The C's `next_curwin` argument to [`frame_minheight`]/[`frame_minwidth`],
/// whose three states are three different rules — and which is why they are
/// not one `Option<Win>`:
///
/// * `NULL` asks for the minimum as things stand, and reserves a line (or a
///   column) for the *current* window when `'winminheight'` is zero;
/// * `NOWIN`, the `(Window *)-1` sentinel `win_equal()` and `win_split_ins()`
///   pass, asks for the same minimum with **no** such reservation;
/// * a window asks for the minimum given that this window is about to become
///   current, so it gets `'winheight'`/`'winwidth'` rather than the minimum.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum NextCurwin {
    /// The C's `NULL`.
    Unset,
    /// The C's `NOWIN`.
    NoWin,
    /// A window that is about to become current.
    Win(WinId),
}

impl NextCurwin {
    /// The window about to become current, as the C's third case.
    pub(crate) fn of(win: Win) -> Self {
        Self::Win(win.id())
    }

    /// Whether this asks about `win` in particular — the C's
    /// `topfrp->fr_win == next_curwin`, which neither sentinel can satisfy.
    fn is(self, win: Win) -> bool {
        self == Self::Win(win.id())
    }
}

/// What [`frame_minheight`] and [`frame_minwidth`] need from the options: the
/// size a `next_curwin` window is owed (`'winheight'` / `'winwidth'`), the
/// minimum any other window may shrink to (`'winminheight'` / `'winminwidth'`)
/// and the current window, for the one-line reservation the `NULL` case makes.
#[derive(Clone, Copy)]
pub(crate) struct MinSize {
    pub wanted: c_int,
    pub minimum: c_int,
    pub curwin: Option<WinId>,
}

/// The minimal height of frame `topfrp`, from `frame_minheight()`.
///
/// A leaf costs its window's minimum plus the rows that are not text (window
/// bar, separator, status line); a row of frames costs the tallest of them and
/// a column the sum.
pub(crate) fn frame_minheight(topfrp: FrameRef, next_curwin: NextCurwin, opts: MinSize) -> c_int {
    if let Some(win) = topfrp.win() {
        // Combined height of window bar and separator column or status line.
        let extra_height = win.w_winbar_height + win.w_hsep_height + win.w_status_height;
        if next_curwin.is(win) {
            // Saturating: `'winheight'` is an unclamped option.
            return opts.wanted.saturating_add(extra_height);
        }
        let mut m = opts.minimum.saturating_add(extra_height);
        // Current window is minimal one line high.
        if Some(win.id()) == opts.curwin && next_curwin == NextCurwin::Unset && opts.minimum == 0 {
            m += 1;
        }
        m
    } else if c_int::from(topfrp.fr_layout) == FR_ROW {
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
pub(crate) fn frame_minwidth(topfrp: FrameRef, next_curwin: NextCurwin, opts: MinSize) -> c_int {
    if let Some(win) = topfrp.win() {
        if next_curwin.is(win) {
            // Saturating: `'winwidth'` is an unclamped option.
            return opts.wanted.saturating_add(win.w_vsep_width);
        }
        // Window: minimal width of the window plus separator column.
        let mut m = opts.minimum.saturating_add(win.w_vsep_width);
        // Current window is minimal one column wide.
        if opts.minimum == 0 && Some(win.id()) == opts.curwin && next_curwin == NextCurwin::Unset {
            m += 1;
        }
        m
    } else if c_int::from(topfrp.fr_layout) == FR_COL {
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
pub(crate) fn frame_check_height(topfrp: FrameRef, height: c_int) -> bool {
    if topfrp.fr_height != height {
        return false;
    }
    if c_int::from(topfrp.fr_layout) == FR_ROW {
        return topfrp.children().all(|frp| frp.fr_height == height);
    }
    true
}

/// Whether `topfrp` and its children are at width `width`, from
/// `frame_check_width()` — [`frame_check_height`] with the axes exchanged, so
/// it is a *column* whose children must match.
pub(crate) fn frame_check_width(topfrp: FrameRef, width: c_int) -> bool {
    if topfrp.fr_width != width {
        return false;
    }
    if c_int::from(topfrp.fr_layout) == FR_COL {
        return topfrp.children().all(|frp| frp.fr_width == width);
    }
    true
}

/// Sort `'colorcolumn'`'s columns ascending, which is what upstream's
/// `qsort(..., int_cmp)` does. Equal `int`s are indistinguishable, so an
/// unstable sort cannot be told from `qsort`'s.
pub(crate) fn sort_columns(columns: &mut [c_int]) {
    columns.sort_unstable();
}

/// A requested window height plus the rows below it that are not text, from
/// `win_setheight_win()`.
///
/// Saturating: `:resize 2147483647` reaches here, and the C's wrap turns the
/// tallest window anyone can ask for into the shortest. Everything downstream
/// clamps to the room actually available, so saturating and wrapping differ
/// only in which of the two answers comes out.
pub(crate) fn height_with_chrome(height: c_int, hsep_height: c_int, status_height: c_int) -> c_int {
    height
        .saturating_add(hsep_height)
        .saturating_add(status_height)
}

/// A requested window width plus its separator column, from
/// `win_setwidth_win()`. Saturating for the reason [`height_with_chrome`] is.
pub(crate) fn width_with_chrome(width: c_int, vsep_width: c_int) -> c_int {
    width.saturating_add(vsep_width)
}

/// The height (or width) to ask a parent frame for when the siblings cannot
/// pay for `size`: `frame_setheight()`/`frame_setwidth()`'s
/// `size + frame_min*(parent, NOWIN) - p_wm* - 1`. Saturating, as above —
/// `size` is the caller's unclamped request.
pub(crate) fn parent_target(size: c_int, parent_minimum: c_int, option_minimum: c_int) -> c_int {
    size.saturating_add(parent_minimum)
        .saturating_sub(option_minimum)
        .saturating_sub(1)
}

/// Where the cursor sits in a window, as sixteen-thousandths of its height,
/// from `set_fraction()`.
///
/// A cursor on the first line counts as halfway down it, so two lines give
/// 25%, three 17%, and the last line 75%, 83% and so on.
pub(crate) fn cursor_fraction(wrow: c_int, view_height: c_int) -> c_int {
    (wrow * FRACTION_MULT + FRACTION_MULT / 2) / view_height
}

/// The inverse, from `scroll_to_fraction()`: the screen row a saved
/// [`cursor_fraction`] puts the cursor on in a window `height` rows tall.
pub(crate) fn fraction_row(fraction: c_int, height: c_int) -> c_int {
    (fraction * height - 1) / FRACTION_MULT
}

#[cfg(test)]
mod tests {
    //! The layout arithmetic, driven directly.
    //!
    //! Each expectation is derived from `v0.12.4`'s `src/nvim/window.c` rather
    //! than from the port: the C function each case is aiming at is named in
    //! its comment. The trees are built here out of registered frames and
    //! [`bare_window`]s — `win_alloc`'s first three lines and none of its body
    //! — because a frame names its window by handle, so a tree over
    //! unregistered windows would read as a tree of empty leaves.
    //!
    //! These used to live in `crates/nvim/tests/unit/window.rs`, where the
    //! fixtures were boxed `MaybeUninit<Window>`s in no registry at all.
    //! Registering them from an integration test would have meant a `pub`
    //! constructor whose only caller was that test; in here the registries'
    //! own crate-internal entry points are reachable, and fifteen items left
    //! the crate's public surface with the move.

    use core::ffi::c_char;
    use std::sync::{Mutex, MutexGuard};

    use super::*;
    use crate::window::alloc::{bare_window, free_bare_window};
    use crate::window::{FR_COL, FR_LEAF, FR_ROW};
    use crate::winlayer::{FrameRef, free_frame, new_frame};

    /// The window and frame registries are process-wide and `cargo test` runs
    /// cases in parallel, so a case that files anything in one takes this
    /// first. The LuaJIT harness got the isolation by forking a child per
    /// case; here there is one process, so it is explicit. Poisoning is
    /// ignored — a panicking case has already reported its own failure, and
    /// each case's [`Tree`] gives back everything it registered.
    static REGISTRIES: Mutex<()> = Mutex::new(());

    /// A window with the chrome a leaf frame's minimum size is made of.
    struct Chrome {
        winbar: c_int,
        hsep: c_int,
        status: c_int,
        vsep: c_int,
    }

    /// A frame tree, and the registrations it has to give back.
    struct Tree {
        frames: Vec<FrameRef>,
        windows: Vec<Win>,
        _registries: MutexGuard<'static, ()>,
    }

    impl Drop for Tree {
        fn drop(&mut self) {
            for &frame in &self.frames {
                free_frame(frame);
            }
            for &win in &self.windows {
                free_bare_window(win);
            }
        }
    }

    impl Tree {
        fn new() -> Self {
            Tree {
                frames: Vec::new(),
                windows: Vec::new(),
                _registries: REGISTRIES.lock().unwrap_or_else(|e| e.into_inner()),
            }
        }

        /// A bare registered frame, linked to nothing.
        fn frame(&mut self) -> FrameRef {
            let frame = new_frame();
            self.frames.push(frame);
            frame
        }

        /// A leaf frame holding a window with the given chrome, `height` rows
        /// and `width` columns.
        fn leaf(&mut self, chrome: Chrome, height: c_int, width: c_int) -> FrameRef {
            let mut win = bare_window();
            self.windows.push(win);
            win.w_winbar_height = chrome.winbar;
            win.w_hsep_height = chrome.hsep;
            win.w_status_height = chrome.status;
            win.w_vsep_width = chrome.vsep;

            let mut fr = self.frame();
            fr.fr_layout = layout_byte(FR_LEAF);
            fr.fr_win = Some(win.id());
            fr.fr_height = height;
            fr.fr_width = width;
            fr
        }

        /// A row (`FR_ROW`) or column (`FR_COL`) of `children`, linked as the
        /// layout tree links them.
        fn branch(&mut self, layout: c_int, children: &[FrameRef]) -> FrameRef {
            let mut parent = self.frame();
            parent.fr_layout = layout_byte(layout);
            parent.fr_child = Some(children[0].id());
            for (i, &child) in children.iter().enumerate() {
                let mut child = child;
                child.fr_parent = Some(parent.id());
                child.fr_prev = (i > 0).then(|| children[i - 1].id());
                child.fr_next = children.get(i + 1).map(|next| next.id());
            }
            parent.fr_height = children[0].fr_height;
            parent.fr_width = children[0].fr_width;
            parent
        }

        fn set_size(&self, fr: FrameRef, height: c_int, width: c_int) {
            let mut fr = fr;
            fr.fr_height = height;
            fr.fr_width = width;
        }

        fn window_of(&self, fr: FrameRef) -> Win {
            fr.win().expect("a leaf frame holds a window")
        }
    }

    /// `'winheight'`/`'winwidth'` and `'winmin*'`, with no current window to
    /// reserve a line for.
    fn heights(wanted: c_int, minimum: c_int) -> MinSize {
        MinSize {
            wanted,
            minimum,
            curwin: None,
        }
    }

    /// `FR_LEAF`/`FR_ROW`/`FR_COL` as the byte a frame stores it in.
    fn layout_byte(layout: c_int) -> c_char {
        c_char::try_from(layout).expect("a layout is 0, 1 or 2")
    }

    const PLAIN: Chrome = Chrome {
        winbar: 0,
        hsep: 0,
        status: 0,
        vsep: 0,
    };

    // ------------------------------------------------------------ frame_minheight
    //
    // `frame_minheight()`:
    //
    //   if (topfrp->fr_win != NULL) {
    //     int extra = winbar + hsep + status;
    //     m = (topfrp->fr_win == next_curwin ? p_wh : p_wmh) + extra;
    //     if (fr_win == curwin && next_curwin == NULL && p_wmh == 0) m++;
    //   } else if (fr_layout == FR_ROW) { m = max over children }
    //   else { m = sum over children }

    #[test]
    fn a_leaf_costs_winminheight_plus_its_chrome() {
        let mut t = Tree::new();
        let status = Chrome { status: 1, ..PLAIN };
        let leaf = t.leaf(status, 5, 20);
        assert_eq!(frame_minheight(leaf, NextCurwin::NoWin, heights(10, 1)), 2);
    }

    #[test]
    fn a_leaf_with_a_window_bar_and_a_separator_costs_all_three() {
        let mut t = Tree::new();
        let chrome = Chrome {
            winbar: 1,
            hsep: 1,
            status: 1,
            vsep: 0,
        };
        let leaf = t.leaf(chrome, 5, 20);
        assert_eq!(frame_minheight(leaf, NextCurwin::NoWin, heights(10, 2)), 5);
    }

    #[test]
    fn the_next_curwin_leaf_costs_winheight_instead() {
        let mut t = Tree::new();
        let status = Chrome { status: 1, ..PLAIN };
        let leaf = t.leaf(status, 5, 20);
        let win = t.window_of(leaf);
        assert_eq!(
            frame_minheight(leaf, NextCurwin::Win(win.id()), heights(10, 1)),
            11
        );
    }

    #[test]
    fn another_window_being_next_curwin_changes_nothing() {
        let mut t = Tree::new();
        let leaf = t.leaf(PLAIN, 5, 20);
        let other = t.leaf(PLAIN, 5, 20);
        let win = t.window_of(other);
        assert_eq!(
            frame_minheight(leaf, NextCurwin::Win(win.id()), heights(10, 1)),
            1
        );
    }

    #[test]
    fn the_current_window_keeps_one_line_when_winminheight_is_zero() {
        // The `NULL` arm, and only it: `p_wmh == 0 && fr_win == curwin`.
        let mut t = Tree::new();
        let leaf = t.leaf(PLAIN, 5, 20);
        let win = t.window_of(leaf);
        let opts = MinSize {
            wanted: 10,
            minimum: 0,
            curwin: Some(win.id()),
        };
        assert_eq!(frame_minheight(leaf, NextCurwin::Unset, opts), 1);
    }

    #[test]
    fn nowin_makes_no_such_reservation() {
        // The trap `winsweep`'s `s3minx` exists for: `win_equal()` passes NOWIN,
        // not NULL, and only the NULL path bumps the current window.
        let mut t = Tree::new();
        let leaf = t.leaf(PLAIN, 5, 20);
        let win = t.window_of(leaf);
        let opts = MinSize {
            wanted: 10,
            minimum: 0,
            curwin: Some(win.id()),
        };
        assert_eq!(frame_minheight(leaf, NextCurwin::NoWin, opts), 0);
    }

    #[test]
    fn a_nonzero_winminheight_makes_no_reservation_either() {
        let mut t = Tree::new();
        let leaf = t.leaf(PLAIN, 5, 20);
        let win = t.window_of(leaf);
        let opts = MinSize {
            wanted: 10,
            minimum: 1,
            curwin: Some(win.id()),
        };
        assert_eq!(frame_minheight(leaf, NextCurwin::Unset, opts), 1);
    }

    #[test]
    fn a_row_costs_the_tallest_of_its_frames() {
        let mut t = Tree::new();
        let plain = t.leaf(PLAIN, 5, 10);
        let tall = t.leaf(
            Chrome {
                winbar: 1,
                hsep: 1,
                status: 1,
                vsep: 0,
            },
            5,
            10,
        );
        let row = t.branch(FR_ROW, &[plain, tall]);
        assert_eq!(frame_minheight(row, NextCurwin::NoWin, heights(10, 2)), 5);
    }

    #[test]
    fn a_column_costs_the_sum_of_its_frames() {
        let mut t = Tree::new();
        let a = t.leaf(Chrome { status: 1, ..PLAIN }, 5, 10);
        let b = t.leaf(Chrome { status: 1, ..PLAIN }, 5, 10);
        let c = t.leaf(PLAIN, 5, 10);
        let col = t.branch(FR_COL, &[a, b, c]);
        assert_eq!(
            frame_minheight(col, NextCurwin::NoWin, heights(10, 1)),
            2 + 2 + 1
        );
    }

    #[test]
    fn an_empty_row_and_column_both_cost_nothing() {
        let mut t = Tree::new();
        let mut row = t.frame();
        row.fr_layout = layout_byte(FR_ROW);
        assert_eq!(frame_minheight(row, NextCurwin::NoWin, heights(10, 1)), 0);
    }

    #[test]
    fn a_column_of_rows_adds_the_tallest_of_each() {
        let mut t = Tree::new();
        let a = t.leaf(Chrome { status: 1, ..PLAIN }, 5, 5);
        let b = t.leaf(PLAIN, 5, 5);
        let top = t.branch(FR_ROW, &[a, b]);
        let c = t.leaf(
            Chrome {
                winbar: 1,
                status: 1,
                ..PLAIN
            },
            5,
            10,
        );
        let col = t.branch(FR_COL, &[top, c]);
        // Row costs max(2, 1) = 2; the leaf below costs 1 + 1 + 1 = 3.
        assert_eq!(frame_minheight(col, NextCurwin::NoWin, heights(10, 1)), 5);
    }

    #[test]
    fn an_absurd_winheight_saturates_rather_than_wrapping() {
        // `:set winheight=2147483647` reaches `p_wh + extra_height`, which the C
        // lets overflow.
        let mut t = Tree::new();
        let leaf = t.leaf(Chrome { status: 1, ..PLAIN }, 5, 20);
        let win = t.window_of(leaf);
        let opts = heights(c_int::MAX, 1);
        assert_eq!(
            frame_minheight(leaf, NextCurwin::Win(win.id()), opts),
            c_int::MAX
        );
    }

    #[test]
    fn a_column_of_absurd_frames_saturates_too() {
        let mut t = Tree::new();
        let a = t.leaf(PLAIN, 5, 10);
        let b = t.leaf(PLAIN, 5, 10);
        let col = t.branch(FR_COL, &[a, b]);
        let opts = heights(10, c_int::MAX);
        assert_eq!(frame_minheight(col, NextCurwin::NoWin, opts), c_int::MAX);
    }

    // ------------------------------------------------------------- frame_minwidth
    //
    // `frame_minwidth()` is the same with the axes exchanged: a *column* takes the
    // widest child and a *row* the sum, and the separator column stands in for the
    // status line.

    #[test]
    fn a_leaf_costs_winminwidth_plus_its_separator() {
        let mut t = Tree::new();
        let leaf = t.leaf(Chrome { vsep: 1, ..PLAIN }, 5, 20);
        assert_eq!(frame_minwidth(leaf, NextCurwin::NoWin, heights(20, 1)), 2);
    }

    #[test]
    fn the_next_curwin_leaf_costs_winwidth() {
        let mut t = Tree::new();
        let leaf = t.leaf(Chrome { vsep: 1, ..PLAIN }, 5, 20);
        let win = t.window_of(leaf);
        assert_eq!(
            frame_minwidth(leaf, NextCurwin::Win(win.id()), heights(20, 1)),
            21
        );
    }

    #[test]
    fn the_current_window_keeps_one_column_when_winminwidth_is_zero() {
        let mut t = Tree::new();
        let leaf = t.leaf(PLAIN, 5, 20);
        let win = t.window_of(leaf);
        let opts = MinSize {
            wanted: 20,
            minimum: 0,
            curwin: Some(win.id()),
        };
        assert_eq!(frame_minwidth(leaf, NextCurwin::Unset, opts), 1);
        assert_eq!(frame_minwidth(leaf, NextCurwin::NoWin, opts), 0);
    }

    #[test]
    fn a_column_costs_the_widest_of_its_frames() {
        let mut t = Tree::new();
        let a = t.leaf(PLAIN, 5, 10);
        let b = t.leaf(Chrome { vsep: 1, ..PLAIN }, 5, 10);
        let col = t.branch(FR_COL, &[a, b]);
        assert_eq!(frame_minwidth(col, NextCurwin::NoWin, heights(20, 1)), 2);
    }

    #[test]
    fn a_row_costs_the_sum_of_its_frames() {
        let mut t = Tree::new();
        let a = t.leaf(Chrome { vsep: 1, ..PLAIN }, 5, 10);
        let b = t.leaf(Chrome { vsep: 1, ..PLAIN }, 5, 10);
        let c = t.leaf(PLAIN, 5, 10);
        let row = t.branch(FR_ROW, &[a, b, c]);
        assert_eq!(
            frame_minwidth(row, NextCurwin::NoWin, heights(20, 1)),
            2 + 2 + 1
        );
    }

    // --------------------------------------------------------- frame_check_height
    //
    // `frame_check_height()`:
    //
    //   if (topfrp->fr_height != height) return false;
    //   if (topfrp->fr_layout == FR_ROW)
    //     FOR_ALL_FRAMES(frp, topfrp->fr_child)
    //       if (frp->fr_height != height) return false;
    //   return true;

    #[test]
    fn a_leaf_at_the_right_height_checks_out() {
        let mut t = Tree::new();
        let leaf = t.leaf(PLAIN, 7, 20);
        assert!(frame_check_height(leaf, 7));
        assert!(!frame_check_height(leaf, 6));
    }

    #[test]
    fn a_rows_children_must_all_be_at_the_rows_height() {
        let mut t = Tree::new();
        let a = t.leaf(PLAIN, 7, 10);
        let b = t.leaf(PLAIN, 7, 10);
        let row = t.branch(FR_ROW, &[a, b]);
        t.set_size(row, 7, 20);
        assert!(frame_check_height(row, 7));
        t.set_size(b, 6, 10);
        assert!(!frame_check_height(row, 7));
    }

    #[test]
    fn a_columns_children_are_not_checked_at_all() {
        // Only a *row* shares its height with its children; a column's are
        // expected to differ, and upstream does not look at them.
        let mut t = Tree::new();
        let a = t.leaf(PLAIN, 3, 10);
        let b = t.leaf(PLAIN, 4, 10);
        let col = t.branch(FR_COL, &[a, b]);
        t.set_size(col, 7, 10);
        assert!(frame_check_height(col, 7));
    }

    #[test]
    fn a_columns_children_must_all_be_at_the_columns_width() {
        let mut t = Tree::new();
        let a = t.leaf(PLAIN, 3, 10);
        let b = t.leaf(PLAIN, 4, 10);
        let col = t.branch(FR_COL, &[a, b]);
        t.set_size(col, 7, 10);
        assert!(frame_check_width(col, 10));
        t.set_size(b, 4, 9);
        assert!(!frame_check_width(col, 10));
    }

    #[test]
    fn a_rows_children_are_not_width_checked() {
        let mut t = Tree::new();
        let a = t.leaf(PLAIN, 7, 10);
        let b = t.leaf(PLAIN, 7, 11);
        let row = t.branch(FR_ROW, &[a, b]);
        t.set_size(row, 7, 21);
        assert!(frame_check_width(row, 21));
    }

    // -------------------------------------------------------- the chrome additions
    //
    // `win_setheight_win()` adds the rows below the window to the height it was
    // asked for, and `win_setwidth_win()` the separator column. `:resize
    // 2147483647` reaches both, and the C's overflow wraps the tallest window
    // anyone can ask for into the shortest.

    #[test]
    fn an_ordinary_height_gains_its_chrome() {
        assert_eq!(height_with_chrome(10, 0, 1), 11);
        assert_eq!(height_with_chrome(10, 1, 1), 12);
        assert_eq!(height_with_chrome(10, 0, 0), 10);
    }

    #[test]
    fn an_absurd_height_saturates_instead_of_wrapping() {
        assert_eq!(height_with_chrome(c_int::MAX, 0, 1), c_int::MAX);
        assert_eq!(height_with_chrome(c_int::MAX, 1, 1), c_int::MAX);
    }

    #[test]
    fn a_negative_height_keeps_its_sign() {
        // `:resize -2147483647` is not clamped away either; `frame_setheight()`
        // takes it as "as small as possible".
        assert_eq!(height_with_chrome(-2147483647, 0, 1), -2147483646);
        assert_eq!(height_with_chrome(c_int::MIN, 0, 0), c_int::MIN);
    }

    #[test]
    fn a_width_gains_its_separator_and_saturates() {
        assert_eq!(width_with_chrome(10, 1), 11);
        assert_eq!(width_with_chrome(10, 0), 10);
        assert_eq!(width_with_chrome(c_int::MAX, 1), c_int::MAX);
    }

    // ------------------------------------------------------------- parent_target
    //
    // `frame_setheight()`'s first run, when the siblings cannot pay:
    //
    //   frame_setheight(curfrp->fr_parent,
    //                   height + frame_minheight(curfrp->fr_parent, NOWIN) - p_wmh - 1);

    #[test]
    fn the_parent_is_asked_for_the_size_plus_its_own_minimum() {
        assert_eq!(parent_target(20, 6, 1), 24);
        assert_eq!(parent_target(20, 6, 0), 25);
    }

    #[test]
    fn an_absurd_request_saturates_on_the_way_up() {
        assert_eq!(parent_target(c_int::MAX, 6, 1), c_int::MAX - 2);
        assert_eq!(parent_target(c_int::MAX, c_int::MAX, 1), c_int::MAX - 2);
    }

    #[test]
    fn a_very_negative_request_saturates_downwards() {
        assert_eq!(parent_target(c_int::MIN, 0, 0), c_int::MIN);
    }

    // ----------------------------------------------------------- the cursor's row
    //
    // `set_fraction()`: wp->w_fraction = (w_wrow * FRACTION_MULT + FRACTION_MULT/2)
    //                                    / w_view_height
    // `scroll_to_fraction()`: wp->w_wrow = (w_fraction * height - 1) / FRACTION_MULT

    const MULT: c_int = 16384;

    #[test]
    fn a_cursor_on_the_first_line_counts_as_halfway_down_it() {
        // Two lines: 25%. Three: about 17%.
        assert_eq!(cursor_fraction(0, 2), MULT / 4);
        assert_eq!(cursor_fraction(0, 3), MULT / 6);
    }

    #[test]
    fn a_cursor_on_the_last_line_counts_as_halfway_down_that() {
        // Two lines: 75%. Three: about 83%.
        assert_eq!(cursor_fraction(1, 2), 3 * MULT / 4);
        assert_eq!(cursor_fraction(2, 3), 5 * MULT / 6);
    }

    #[test]
    fn the_fraction_of_a_middle_row_is_its_share_of_the_height() {
        assert_eq!(cursor_fraction(5, 10), MULT / 2 + MULT / 20);
    }

    #[test]
    fn the_row_a_fraction_names_is_its_share_of_the_new_height() {
        // A cursor halfway down a ten-row window lands on row five of a ten-row
        // one, and on row two of a five-row one.
        assert_eq!(fraction_row(MULT / 2, 10), 4);
        assert_eq!(fraction_row(MULT / 2, 20), 9);
        // Integer division truncates towards zero, so `(0 * height - 1) / MULT` is
        // zero rather than -1.
        assert_eq!(fraction_row(0, 20), 0);
        assert_eq!(fraction_row(MULT, 10), 9);
    }

    #[test]
    fn a_full_fraction_lands_on_the_last_row_of_any_height() {
        for height in 1..40 {
            assert_eq!(fraction_row(MULT, height), height - 1);
        }
    }

    #[test]
    fn the_two_are_each_others_inverse_to_within_a_row() {
        for height in 2..40 {
            for row in 0..height {
                let back = fraction_row(cursor_fraction(row, height), height);
                assert!(
                    (back - row).abs() <= 1,
                    "row {row} of {height} came back as {back}"
                );
            }
        }
    }

    // -------------------------------------------------------------- sort_columns
    //
    // `check_colorcolumn()` sorts the parsed columns with `qsort(..., int_cmp)`
    // before dropping the duplicates.

    #[test]
    fn columns_come_back_ascending() {
        let mut cols = [30, 10, 20, 0, -1];
        sort_columns(&mut cols);
        assert_eq!(cols, [-1, 0, 10, 20, 30]);
    }

    #[test]
    fn duplicates_survive_the_sort_next_to_each_other() {
        // `check_colorcolumn()` drops them afterwards, and only trusts the sort to
        // have put them together.
        let mut cols = [20, 10, 20, 10, 20];
        sort_columns(&mut cols);
        assert_eq!(cols, [10, 10, 20, 20, 20]);
    }

    #[test]
    fn an_empty_or_single_column_list_is_left_alone() {
        let mut none: [c_int; 0] = [];
        sort_columns(&mut none);
        let mut one = [7];
        sort_columns(&mut one);
        assert_eq!(one, [7]);
    }
}
